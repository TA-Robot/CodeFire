import json
import runpy
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class SelectiveDeadline:
    def __init__(self):
        self.limit_ms = None
        self.started_at = 0.0
        self.last_observed = 0.0
        self.callback = None
        self.match = None

    def check(self, phase):
        callback = self.callback
        if callback is not None and (
            self.match is None or self.match in phase
        ):
            self.callback = None
            callback(phase)


class PythonExistingTerminalOwnerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.namespace = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.namespace["DecisionWorkLedger"]
        cls.Profile = cls.namespace["DecisionWorkProfile"]
        cls.Authority = cls.namespace[
            "AccountedRemoteSignaturePolicyAuthority"
        ]
        cls.Sealed = cls.namespace["AccountedHTTPUploadSealedValidation"]
        cls.Owner = cls.namespace[
            "AccountedHTTPUploadExistingTerminalOwner"
        ]

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        subprocess.run(
            [str(CODEFIRE), "init"],
            cwd=self.tmp,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        self.objects = self.tmp / ".codefire/objects"
        self.initial = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(
                encoding="utf-8"
            )
        )["head"]
        payload = self.namespace["load_object_from_root"](
            self.objects, self.initial
        )
        child = json.loads(json.dumps(payload))
        child["message"] = "single existing terminal owner"
        child["parents"] = [self.initial]
        self.key = "existing-terminal-secret"
        child = self.namespace["sign_commit_payload"](
            child, "alice", self.key
        )
        self.head = self.namespace["store_object"](
            self.tmp, "commit", child
        )
        self.project = self.tmp / "remote/org/app"
        self.project.mkdir(parents=True)
        self.policy_path = self.project / "server_policy.json"

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def ledger(self, label="existing terminal owner"):
        return self.Ledger(
            self.Profile.http_v1(), label, command_kind="http_request"
        )

    def policy(self, command=""):
        value = {
            "commit_signatures": {
                "required": True,
                "keys": {"alice": self.key},
            },
            "verification": {"required": []},
        }
        if command:
            value["verification"]["required"].append(
                {"id": "existing-terminal", "command": command}
            )
        self.policy_path.write_bytes(
            json.dumps(value, separators=(",", ":")).encode("utf-8")
        )

    def prepare_sealed(self, ledger, deadline):
        paths = list(self.objects.glob("*/*.json"))
        sizes = [path.stat().st_size for path in paths]
        sealed = self.namespace["prepare_http_upload_sealed_validation"](
            ledger,
            self.objects,
            self.head,
            record_count_upper=len(sizes),
            aggregate_bytes_upper=sum(sizes),
            maximum_record_bytes_upper=max(sizes),
            deadline=deadline,
        )
        sealed.validate(ledger)
        return sealed

    def prepare_owner(self, command="", label="existing terminal owner"):
        self.policy(command)
        ledger = self.ledger(label)
        deadline = SelectiveDeadline()
        sealed = self.prepare_sealed(ledger, deadline)
        authority = self.Authority(self.project, ledger, deadline)
        owner = self.Owner(sealed, authority, ledger)
        return ledger, deadline, sealed, authority, owner

    def assert_zero(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))
        self.assertIsNone(ledger._upload_terminal_owner)

    def test_exact_children_do_not_escape_and_full_bfs_is_single_use(self):
        ledger, _deadline, sealed, authority, owner = self.prepare_owner()
        signature = sealed._borrow_handles["signature"]
        ancestry_handle = sealed._borrow_handles["ancestry"]
        policy_type = self.namespace["AccountedCommitSignaturePolicy"]
        ancestry_type = self.namespace["AccountedHTTPUploadAncestry"]
        original_policy_init = policy_type.__init__
        original_ancestry_init = ancestry_type.__init__
        observed = []

        def policy_init(child, raw, selected, loader, deadline):
            observed.append(("signature", loader is signature))
            return original_policy_init(child, raw, selected, loader, deadline)

        def ancestry_init(child, selected, loader, deadline, **kwargs):
            observed.append(("ancestry", loader is ancestry_handle))
            return original_ancestry_init(
                child, selected, loader, deadline, **kwargs
            )

        policy_type.__init__ = policy_init
        ancestry_type.__init__ = ancestry_init
        try:
            self.assertIsNone(
                owner.verify_signature_and_server("main", self.head, ledger)
            )
            self.assertTrue(
                owner.contains_ancestor(self.initial, self.head, ledger)
            )
        finally:
            policy_type.__init__ = original_policy_init
            ancestry_type.__init__ = original_ancestry_init
        self.assertEqual(observed, [("signature", True), ("ancestry", True)])
        self.assertIsNone(owner._verification)
        self.assertIsNone(owner._ancestry)
        self.assertEqual(signature._token, 0)
        self.assertEqual(ancestry_handle._token, 0)
        self.assertEqual(owner._state, owner.ANCESTRY_DONE)
        self.assertIs(ledger._upload_terminal_owner, owner)
        self.assertIs(sealed._upload_terminal_owner, owner)
        self.assertIs(authority._upload_terminal_owner, owner)
        for operation in (
            lambda: owner.verify_signature_and_server(
                "main", self.head, ledger
            ),
            lambda: owner.contains_ancestor(self.initial, self.head, ledger),
        ):
            with self.assertRaises(self.namespace["CFError"]):
                operation()
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_stale_parent_aliases_and_old_composites_are_rejected(self):
        ledger, deadline, sealed, authority, owner = self.prepare_owner()
        operations = (
            lambda: sealed.borrow_commits(ledger, "signature"),
            lambda: sealed.finish(ledger),
            lambda: sealed.close(ledger),
            lambda: authority.prepare_policy(lambda _oid: None, ledger),
            lambda: authority.revalidate(ledger),
            lambda: authority.close(ledger),
            lambda: self.namespace[
                "AccountedHTTPUploadSignatureVerificationComposite"
            ](sealed, authority, ledger),
            lambda: self.namespace["prepare_http_upload_sealed_ancestry"](
                sealed, ledger, deadline
            ).prepare(ledger),
        )
        for operation in operations:
            with self.assertRaisesRegex(
                self.namespace["CFError"], "upload terminal|already owns"
            ):
                operation()
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_bind_faults_rollback_registry_and_both_parents(self):
        self.policy()
        owner_type = self.Owner
        original_bind = owner_type._bind_parent_tokens
        for fault_point in ("pre", "after_sealed", "post"):
            with self.subTest(fault_point=fault_point):
                ledger = self.ledger(f"bind {fault_point}")
                deadline = SelectiveDeadline()
                sealed = self.prepare_sealed(ledger, deadline)
                authority = self.Authority(self.project, ledger, deadline)

                def fail_bind(selected, policy, owner):
                    if fault_point == "pre":
                        raise KeyboardInterrupt("synthetic bind fault")
                    if fault_point == "after_sealed":
                        object.__setattr__(
                            selected, "_upload_terminal_owner", owner
                        )
                        with self.assertRaisesRegex(
                            self.namespace["CFError"], "binding"
                        ):
                            owner.verify_signature_and_server(
                                "main", self.head, ledger
                            )
                        raise KeyboardInterrupt("synthetic bind fault")
                    original_bind(selected, policy, owner)
                    raise KeyboardInterrupt("synthetic bind fault")

                owner_type._bind_parent_tokens = staticmethod(fail_bind)
                try:
                    with self.assertRaisesRegex(
                        KeyboardInterrupt, "synthetic bind fault"
                    ):
                        self.Owner(sealed, authority, ledger)
                finally:
                    owner_type._bind_parent_tokens = staticmethod(
                        original_bind
                    )
                self.assertIsNone(ledger._upload_terminal_owner)
                self.assertIsNone(sealed._upload_terminal_owner)
                self.assertIsNone(authority._upload_terminal_owner)
                authority.close(ledger)
                sealed.close(ledger)
                self.assert_zero(ledger)

    def test_binding_reentry_close_cancels_constructor_after_pair_bind(self):
        self.policy()
        ledger = self.ledger("binding reentry close")
        deadline = SelectiveDeadline()
        sealed = self.prepare_sealed(ledger, deadline)
        authority = self.Authority(self.project, ledger, deadline)
        owner_type = self.Owner
        original_bind = owner_type._bind_parent_tokens
        observations = []

        def bind_then_reenter(selected, policy, owner):
            original_bind(selected, policy, owner)
            with self.assertRaisesRegex(
                self.namespace["CFError"], "binding"
            ):
                owner.verify_signature_and_server(
                    "main", self.head, ledger
                )
            observations.append(owner.close(ledger))

        owner_type._bind_parent_tokens = staticmethod(bind_then_reenter)
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"], "binding was cancelled"
            ):
                self.Owner(sealed, authority, ledger)
        finally:
            owner_type._bind_parent_tokens = staticmethod(original_bind)
        self.assertEqual(observations, [None])
        self.assertFalse(sealed._borrow_handles)
        self.assertTrue(sealed._closed)
        self.assertTrue(authority._closed)
        self.assert_zero(ledger)

    def test_parent_mirror_setters_are_never_called(self):
        self.policy()
        ledger = self.ledger("callback-free parent mirrors")
        deadline = SelectiveDeadline()
        sealed = self.prepare_sealed(ledger, deadline)
        authority = self.Authority(self.project, ledger, deadline)
        sealed_type = type(sealed)
        authority_type = type(authority)
        original_sealed_setattr = sealed_type.__setattr__
        original_authority_setattr = authority_type.__setattr__
        calls = []

        def reject_sealed(selected, name, value):
            if selected is sealed and name == "_upload_terminal_owner":
                calls.append("sealed")
                raise KeyboardInterrupt("replaceable sealed mirror")
            return original_sealed_setattr(selected, name, value)

        def reject_authority(selected, name, value):
            if selected is authority and name == "_upload_terminal_owner":
                calls.append("policy")
                raise KeyboardInterrupt("replaceable policy mirror")
            return original_authority_setattr(selected, name, value)

        sealed_type.__setattr__ = reject_sealed
        authority_type.__setattr__ = reject_authority
        try:
            owner = self.Owner(sealed, authority, ledger)
        finally:
            sealed_type.__setattr__ = original_sealed_setattr
            authority_type.__setattr__ = original_authority_setattr
        self.assertEqual(calls, [])
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_operation_guards_are_callback_free_under_setattr_hooks(self):
        for operation_name in ("verification", "ancestry"):
            with self.subTest(operation=operation_name):
                ledger, _deadline, _sealed, _authority, owner = (
                    self.prepare_owner(label=f"guard {operation_name}")
                )
                if operation_name == "ancestry":
                    owner.verify_signature_and_server(
                        "main", self.head, ledger
                    )
                owner_type = type(owner)
                original_setattr = owner_type.__setattr__
                intercepted = []

                def reject_guards(selected, name, value):
                    if selected is owner and name in {
                        "_state",
                        "_operating",
                    }:
                        intercepted.append((name, value))
                        raise KeyboardInterrupt("replaceable guard setter")
                    return original_setattr(selected, name, value)

                owner_type.__setattr__ = reject_guards
                try:
                    if operation_name == "verification":
                        owner.verify_signature_and_server(
                            "main", self.head, ledger
                        )
                    else:
                        self.assertTrue(
                            owner.contains_ancestor(
                                self.initial, self.head, ledger
                            )
                        )
                finally:
                    owner_type.__setattr__ = original_setattr
                self.assertEqual(intercepted, [])
                owner.close(ledger)
                self.assert_zero(ledger)

    def test_finish_callback_cannot_bind_in_progress_sealed_parent(self):
        self.policy()
        ledger = self.ledger("finish callback bind")
        deadline = SelectiveDeadline()
        sealed = self.prepare_sealed(ledger, deadline)
        authority = self.Authority(self.project, ledger, deadline)
        rejected = []

        def callback(_phase):
            try:
                self.Owner(sealed, authority, ledger)
            except self.namespace["CFError"] as exc:
                rejected.append(str(exc))
            authority.close(ledger)

        deadline.callback = callback
        deadline.match = "upload sealed validation"
        sealed.finish(ledger)
        self.assertEqual(len(rejected), 1)
        self.assertIn("idle sealed authority", rejected[0])
        self.assertIsNone(ledger._upload_terminal_owner)
        self.assertIsNone(sealed._upload_terminal_owner)
        self.assertIsNone(authority._upload_terminal_owner)
        self.assert_zero(ledger)

    def test_preused_or_active_sealed_authority_is_rejected_before_binding(self):
        self.policy()
        ledger = self.ledger("preused terminal owner")
        deadline = SelectiveDeadline()
        sealed = self.prepare_sealed(ledger, deadline)
        authority = self.Authority(self.project, ledger, deadline)
        handle = sealed.borrow_commits(ledger, "signature")
        with self.assertRaisesRegex(
            self.namespace["CFError"], "idle sealed authority"
        ):
            self.Owner(sealed, authority, ledger)
        self.assertIsNone(ledger._upload_terminal_owner)
        handle.close(ledger)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "unused consumer handles"
        ):
            self.Owner(sealed, authority, ledger)
        authority.close(ledger)
        sealed.close(ledger)
        self.assert_zero(ledger)

    def test_missing_project_authority_is_out_of_scope_before_binding(self):
        self.policy()
        ledger = self.ledger("missing existing project")
        deadline = SelectiveDeadline()
        sealed = self.prepare_sealed(ledger, deadline)
        missing = self.tmp / "absent/project"
        authority = self.Authority(missing, ledger, deadline)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "existing project"
        ):
            self.Owner(sealed, authority, ledger)
        self.assertIsNone(ledger._upload_terminal_owner)
        self.assertFalse(sealed._borrow_handles["signature"]._used)
        self.assertFalse(sealed._borrow_handles["ancestry"]._used)
        authority.close(ledger)
        sealed.close(ledger)
        self.assert_zero(ledger)

    def test_post_activation_signature_fault_is_abort_only_and_reconciled(self):
        ledger, _deadline, sealed, _authority, owner = self.prepare_owner()
        signature = sealed._borrow_handles["signature"]
        ancestry = sealed._borrow_handles["ancestry"]
        original = sealed.borrow_commits

        def fail_after_activation(*args, **kwargs):
            original(*args, **kwargs)
            raise KeyboardInterrupt("post activation")

        sealed.borrow_commits = fail_after_activation
        with self.assertRaisesRegex(KeyboardInterrupt, "post activation"):
            owner.verify_signature_and_server("main", self.head, ledger)
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self.assertEqual(signature._token, 0)
        self.assertFalse(ancestry._used)
        self.assertTrue(owner._closed)
        self.assert_zero(ledger)

    def test_verification_external_effect_failure_is_abort_only(self):
        ledger, _deadline, sealed, _authority, owner = self.prepare_owner(
            "touch failed.marker; exit 1", "terminal external failure"
        )
        ancestry = sealed._borrow_handles["ancestry"]
        with self.assertRaisesRegex(
            self.namespace["CFError"], "server verification failed"
        ):
            owner.verify_signature_and_server("main", self.head, ledger)
        self.assertTrue((self.project / "failed.marker").exists())
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self.assertFalse(ancestry._used)
        with self.assertRaises(self.namespace["CFError"]):
            owner.contains_ancestor(self.initial, self.head, ledger)
        self.assert_zero(ledger)

    def test_callback_close_before_spawn_becomes_typed_cancellation(self):
        ledger, deadline, _sealed, _authority, owner = self.prepare_owner(
            "touch must-not-run.marker", "terminal pre-spawn cancellation"
        )
        deadline.callback = lambda _phase: owner.close(ledger)
        deadline.match = "commit signature"
        with self.assertRaisesRegex(
            self.namespace["CFError"], "cancelled"
        ):
            owner.verify_signature_and_server("main", self.head, ledger)
        self.assertFalse((self.project / "must-not-run.marker").exists())
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self.assert_zero(ledger)

    def test_callback_close_during_run_cancels_after_external_effect(self):
        ledger, deadline, _sealed, _authority, owner = self.prepare_owner(
            "touch ran.marker", "terminal run cancellation"
        )
        deadline.callback = lambda _phase: owner.close(ledger)
        deadline.match = "verification execution"
        with self.assertRaisesRegex(
            self.namespace["CFError"], "cancelled"
        ):
            owner.verify_signature_and_server("main", self.head, ledger)
        self.assertTrue((self.project / "ran.marker").exists())
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self.assert_zero(ledger)

    def test_ancestry_post_activation_fault_does_not_escape_bool(self):
        ledger, _deadline, sealed, _authority, owner = self.prepare_owner()
        owner.verify_signature_and_server("main", self.head, ledger)
        handle = sealed._borrow_handles["ancestry"]
        original = sealed.borrow_commits

        def fail_after_activation(*args, **kwargs):
            original(*args, **kwargs)
            raise KeyboardInterrupt("ancestry post activation")

        sealed.borrow_commits = fail_after_activation
        with self.assertRaisesRegex(
            KeyboardInterrupt, "ancestry post activation"
        ):
            owner.contains_ancestor(self.initial, self.head, ledger)
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self.assertEqual(handle._token, 0)
        self.assert_zero(ledger)

    def test_policy_cleanup_fault_stops_before_sealed_and_retries(self):
        ledger, _deadline, sealed, authority, owner = self.prepare_owner()
        owner.verify_signature_and_server("main", self.head, ledger)
        owner.contains_ancestor(self.initial, self.head, ledger)
        original = authority.close
        calls = []

        def fail_policy(*_args, **_kwargs):
            calls.append("policy")
            raise KeyboardInterrupt("policy cleanup fault")

        authority.close = fail_policy
        with self.assertRaisesRegex(
            KeyboardInterrupt, "policy cleanup fault"
        ):
            owner.close(ledger)
        self.assertEqual(calls, ["policy"])
        self.assertFalse(sealed._closed)
        self.assertIs(owner._policy_authority, authority)
        authority.close = original
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_partial_verification_child_is_published_and_retried(self):
        ledger, _deadline, _sealed, authority, owner = self.prepare_owner()
        verification_type = self.namespace["AccountedServerVerification"]
        original_acquire = verification_type._acquire_capability_owner
        original_close = verification_type.close
        children = []
        close_calls = []

        def fail_acquire(child, *_args, **_kwargs):
            children.append(
                (
                    child,
                    owner._verification is child,
                    authority._verification_owner is child,
                )
            )
            raise KeyboardInterrupt("partial verification construction")

        def fail_first_close(child):
            close_calls.append(child)
            if len(close_calls) == 1:
                raise KeyboardInterrupt("partial verification close")
            return original_close(child)

        verification_type._acquire_capability_owner = fail_acquire
        verification_type.close = fail_first_close
        try:
            with self.assertRaisesRegex(
                KeyboardInterrupt, "partial verification close"
            ):
                owner.verify_signature_and_server("main", self.head, ledger)
        finally:
            verification_type._acquire_capability_owner = original_acquire
            verification_type.close = original_close
        self.assertEqual(len(children), 1)
        child, owner_published, authority_published = children[0]
        self.assertTrue(owner_published)
        self.assertTrue(authority_published)
        self.assertTrue(child._closed)
        self.assertIsNone(authority._verification_owner)
        self.assertGreaterEqual(len(close_calls), 2)
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self.assert_zero(ledger)

    def test_verification_wrapper_and_post_effect_close_never_escape_child(self):
        ledger, _deadline, _sealed, authority, owner = self.prepare_owner()
        original_core = authority._prepare_verified_server_verification_core
        published = []

        def fail_after_publish(*args, **kwargs):
            child = original_core(*args, **kwargs)
            published.append(child)
            raise KeyboardInterrupt("post verification publish")

        authority._prepare_verified_server_verification_core = fail_after_publish
        with self.assertRaisesRegex(
            KeyboardInterrupt, "post verification publish"
        ):
            owner.verify_signature_and_server("main", self.head, ledger)
        self.assertEqual(len(published), 1)
        self.assertTrue(published[0]._closed)
        self.assertIsNone(owner._verification)
        self.assert_zero(ledger)

        ledger, _deadline, _sealed, _authority, owner = self.prepare_owner(
            label="verification post-close"
        )
        verification_type = self.namespace["AccountedServerVerification"]
        original_close = verification_type.close
        close_calls = []

        def close_then_fail(child):
            result = original_close(child)
            close_calls.append(child)
            if len(close_calls) == 1:
                raise KeyboardInterrupt("post verification close")
            return result

        verification_type.close = close_then_fail
        try:
            with self.assertRaisesRegex(
                KeyboardInterrupt, "post verification close"
            ):
                owner.verify_signature_and_server("main", self.head, ledger)
        finally:
            verification_type.close = original_close
        self.assertTrue(close_calls[0]._closed)
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self.assert_zero(ledger)

    def test_partial_ancestry_construction_and_auto_close_are_reconciled(self):
        ledger, _deadline, _sealed, _authority, owner = self.prepare_owner()
        owner.verify_signature_and_server("main", self.head, ledger)
        ancestry_type = self.namespace["AccountedHTTPUploadAncestry"]
        original_allocate = ancestry_type._allocate_seen
        original_close = ancestry_type.close
        close_calls = []

        def fail_allocate():
            raise KeyboardInterrupt("partial ancestry construction")

        def fail_first_close(child, selected=None):
            close_calls.append(child)
            if len(close_calls) == 1:
                raise KeyboardInterrupt("partial ancestry close")
            return original_close(child, selected)

        ancestry_type._allocate_seen = staticmethod(fail_allocate)
        ancestry_type.close = fail_first_close
        try:
            with self.assertRaisesRegex(
                KeyboardInterrupt, "partial ancestry close"
            ):
                owner.contains_ancestor(self.initial, self.head, ledger)
        finally:
            ancestry_type._allocate_seen = staticmethod(original_allocate)
            ancestry_type.close = original_close
        self.assertGreaterEqual(len(close_calls), 2)
        self.assertTrue(close_calls[0]._closed)
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self.assert_zero(ledger)

        ledger, _deadline, _sealed, _authority, owner = self.prepare_owner(
            label="ancestry auto-close"
        )
        owner.verify_signature_and_server("main", self.head, ledger)
        original_close = ancestry_type.close
        auto_closed = []

        def close_then_fail(child, selected=None):
            result = original_close(child, selected)
            auto_closed.append(child)
            if len(auto_closed) == 1:
                raise KeyboardInterrupt("post ancestry auto-close")
            return result

        ancestry_type.close = close_then_fail
        try:
            with self.assertRaisesRegex(
                KeyboardInterrupt, "post ancestry auto-close"
            ):
                owner.contains_ancestor(self.initial, self.head, ledger)
        finally:
            ancestry_type.close = original_close
        self.assertTrue(auto_closed[0]._closed)
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self.assert_zero(ledger)

    def test_abort_cleanup_hard_stops_child_then_borrow_then_parents(self):
        ledger, _deadline, sealed, authority, owner = self.prepare_owner()
        owner.verify_signature_and_server("main", self.head, ledger)
        borrow = sealed._borrow_handles["ancestry"]
        owner._ancestry_borrow = borrow
        selected = sealed.borrow_commits(
            ledger, "ancestry", _upload_terminal_owner=owner
        )
        self.assertIs(selected, borrow)
        ancestry_type = self.namespace["AccountedHTTPUploadAncestry"]
        ancestry_type(ledger, borrow, owner._deadline, _handoff_owner=owner)
        ancestry = owner._ancestry
        original_ancestry_close = ancestry.close
        original_borrow_close = borrow.close
        borrow_calls = []

        def fail_child(*_args, **_kwargs):
            raise KeyboardInterrupt("ancestry child cleanup")

        def observe_borrow(*args, **kwargs):
            borrow_calls.append(True)
            return original_borrow_close(*args, **kwargs)

        ancestry.close = fail_child
        borrow.close = observe_borrow
        with self.assertRaisesRegex(
            KeyboardInterrupt, "ancestry child cleanup"
        ):
            owner.close(ledger)
        self.assertEqual(borrow_calls, [])
        self.assertFalse(authority._closed)
        ancestry.close = original_ancestry_close

        def fail_borrow(*_args, **_kwargs):
            raise KeyboardInterrupt("ancestry borrow cleanup")

        borrow.close = fail_borrow
        with self.assertRaisesRegex(
            KeyboardInterrupt, "ancestry borrow cleanup"
        ):
            owner.close(ledger)
        self.assertFalse(authority._closed)
        borrow.close = original_borrow_close
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_signature_child_and_borrow_cleanup_faults_stop_before_policy(self):
        ledger, _deadline, sealed, authority, owner = self.prepare_owner()
        signature_type = self.namespace["AccountedCommitSignaturePolicy"]
        original_verify = signature_type.verify
        original_close = signature_type.close
        signature_borrow = sealed._borrow_handles["signature"]

        def fail_verify(*_args, **_kwargs):
            raise KeyboardInterrupt("signature verify fault")

        def fail_child_close(*_args, **_kwargs):
            raise KeyboardInterrupt("signature child cleanup")

        signature_type.verify = fail_verify
        signature_type.close = fail_child_close
        try:
            with self.assertRaisesRegex(
                KeyboardInterrupt, "signature child cleanup"
            ):
                owner.verify_signature_and_server("main", self.head, ledger)
        finally:
            signature_type.verify = original_verify
            signature_type.close = original_close
        self.assertFalse(authority._closed)
        self.assertNotEqual(signature_borrow._token, 0)
        owner.close(ledger)
        self.assert_zero(ledger)

        ledger, _deadline, sealed, authority, owner = self.prepare_owner(
            label="signature borrow cleanup"
        )
        signature_borrow = sealed._borrow_handles["signature"]
        original_borrow = sealed.borrow_commits
        original_borrow_close = signature_borrow.close

        def fail_after_activation(*args, **kwargs):
            original_borrow(*args, **kwargs)
            raise KeyboardInterrupt("signature activation wrapper")

        def fail_borrow_close(*_args, **_kwargs):
            raise KeyboardInterrupt("signature borrow cleanup")

        sealed.borrow_commits = fail_after_activation
        signature_borrow.close = fail_borrow_close
        with self.assertRaisesRegex(
            KeyboardInterrupt, "signature borrow cleanup"
        ):
            owner.verify_signature_and_server("main", self.head, ledger)
        self.assertFalse(authority._closed)
        self.assertNotEqual(signature_borrow._token, 0)
        signature_borrow.close = original_borrow_close
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_ancestry_callback_close_and_reentry_are_typed_cancellation(self):
        ledger, deadline, _sealed, _authority, owner = self.prepare_owner()
        owner.verify_signature_and_server("main", self.head, ledger)
        reentry = []

        def callback(_phase):
            owner.close(ledger)
            for operation in (
                lambda: owner.contains_ancestor(
                    self.initial, self.head, ledger
                ),
                lambda: owner.verify_signature_and_server(
                    "main", self.head, ledger
                ),
            ):
                try:
                    operation()
                except self.namespace["CFError"] as exc:
                    reentry.append(str(exc))

        deadline.callback = callback
        deadline.match = "upload ancestry"
        with self.assertRaisesRegex(
            self.namespace["CFError"], "cancelled"
        ):
            owner.contains_ancestor(self.initial, self.head, ledger)
        self.assertEqual(len(reentry), 2)
        self.assertTrue(any("operation" in value for value in reentry))
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self.assert_zero(ledger)

    def test_surface_has_no_terminal_detach_or_publication_api(self):
        ledger, _deadline, _sealed, _authority, owner = self.prepare_owner()
        for name in (
            "finish",
            "terminal_detach",
            "effect_input",
            "publish",
            "pair_probe",
            "detach_parents",
        ):
            self.assertFalse(hasattr(owner, name), name)
        owner.close(ledger)
        self.assert_zero(ledger)


if __name__ == "__main__":
    unittest.main()
