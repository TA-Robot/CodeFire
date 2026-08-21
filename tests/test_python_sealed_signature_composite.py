import gc
import json
import os
import runpy
import shutil
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class ReentryDeadline:
    def __init__(self):
        self.limit_ms = None
        self.started_at = 0.0
        self.last_observed = 0.0
        self.armed = False
        self.callback = None
        self.phases = []

    def check(self, phase):
        self.phases.append(phase)
        if self.armed and self.callback is not None:
            callback = self.callback
            self.callback = None
            callback(phase)


class PythonSealedSignatureCompositeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.namespace = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.namespace["DecisionWorkLedger"]
        cls.Profile = cls.namespace["DecisionWorkProfile"]
        cls.Authority = cls.namespace[
            "AccountedRemoteSignaturePolicyAuthority"
        ]
        cls.Composite = cls.namespace[
            "AccountedHTTPUploadSignatureVerificationComposite"
        ]

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.namespace["subprocess"].run(
            [str(CODEFIRE), "init"],
            cwd=self.tmp,
            check=True,
            stdout=self.namespace["subprocess"].PIPE,
            stderr=self.namespace["subprocess"].PIPE,
            text=True,
        )
        self.objects = self.tmp / ".codefire/objects"
        self.unsigned_head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(
                encoding="utf-8"
            )
        )["head"]
        self.project = self.tmp / "remote-project"
        self.project.mkdir()
        self.policy_path = self.project / "server_policy.json"
        self.key = "sealed-signature-secret"

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def ledger(self, label="sealed signature composite"):
        return self.Ledger(
            self.Profile.http_v1(), label, command_kind="http_request"
        )

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def signed_head(self):
        payload = self.namespace["load_object_from_root"](
            self.objects, self.unsigned_head
        )
        signed = self.namespace["sign_commit_payload"](
            payload, "alice", self.key
        )
        return self.namespace["store_object"](self.tmp, "commit", signed)

    def write_policy(self, *, required=True, checks=None):
        policy = {
            "commit_signatures": {
                "required": required,
                "keys": {"alice": self.key},
            },
            "verification": {"required": [] if checks is None else checks},
        }
        self.policy_path.write_bytes(
            json.dumps(policy, separators=(",", ":")).encode("utf-8")
        )

    def prepare_sealed(self, ledger, head, deadline=None):
        paths = list(self.objects.glob("*/*.json"))
        sizes = [path.stat().st_size for path in paths]
        owner = self.namespace["prepare_http_upload_sealed_validation"](
            ledger,
            self.objects,
            head,
            record_count_upper=len(sizes),
            aggregate_bytes_upper=sum(sizes),
            maximum_record_bytes_upper=max(sizes),
            deadline=deadline,
        )
        owner.validate(ledger)
        return owner

    def complete_success(self, ledger, composite, verification):
        verification.close()
        composite.close(ledger)
        self.assert_closed(ledger)

    def test_exact_retained_payload_is_loader_without_clone_or_reopen(self):
        head = self.signed_head()
        self.write_policy()
        ledger = self.ledger()
        sealed = self.prepare_sealed(ledger, head)
        authority = self.Authority(self.project, ledger)
        composite = self.Composite(sealed, authority, ledger)
        document_payload = sealed._session.documents[head].record["payload"]
        original_borrow = sealed._borrow_commit
        original_loader = self.namespace["load_object_from_root"]
        policy_type = self.namespace["AccountedCommitSignaturePolicy"]
        original_policy_init = policy_type.__init__
        observations = []
        loader_observations = []

        def observe_borrow(handle, token, mode, object_id):
            result = original_borrow(handle, token, mode, object_id)
            observations.append(
                (object_id, result[0] is document_payload, result[1])
            )
            return result

        def forbidden_reopen(*_args, **_kwargs):
            raise AssertionError("composite reopened the object store")

        def observe_policy_init(
            policy_owner, raw, selected, loader, deadline
        ):
            loader_observations.append(loader)
            original_policy_init(
                policy_owner, raw, selected, loader, deadline
            )

        sealed._borrow_commit = observe_borrow
        self.namespace["load_object_from_root"] = forbidden_reopen
        policy_type.__init__ = observe_policy_init
        try:
            verification = (
                composite.prepare("main", head, ledger)
            )
        finally:
            policy_type.__init__ = original_policy_init
            self.namespace["load_object_from_root"] = original_loader
            sealed._borrow_commit = original_borrow
        self.assertEqual(len(observations), 1)
        self.assertEqual(observations[0][0], head)
        self.assertTrue(observations[0][1])
        self.assertGreater(observations[0][2], 0)
        handle = sealed._borrow_handles["signature"]
        self.assertEqual(loader_observations, [handle])
        self.assertTrue(handle._used)
        self.assertEqual(handle._token, 0)
        self.assertEqual(sealed._active_borrow_token, 0)
        self.assertFalse(sealed._upload_terminal_active)
        self.complete_success(ledger, composite, verification)

    def test_ledger_deadline_head_and_validation_must_match_before_use(self):
        head = self.signed_head()
        self.write_policy()
        deadline = ReentryDeadline()
        ledger = self.ledger("composite identity")
        foreign = self.ledger("foreign composite identity")
        sealed = self.prepare_sealed(ledger, head, deadline)
        authority = self.Authority(self.project, ledger, deadline)
        composite = self.Composite(sealed, authority, ledger)
        handle = sealed._borrow_handles["signature"]
        for selected, commit_id, expected in (
            (foreign, head, "different ledger"),
            (ledger, self.unsigned_head, "validated head"),
        ):
            with self.subTest(expected=expected):
                with self.assertRaisesRegex(self.namespace["CFError"], expected):
                    composite.prepare("main", commit_id, selected)
                self.assertFalse(handle._used)
                self.assertFalse(authority._verification_bound)
        other_authority = self.Authority(
            self.project, ledger, ReentryDeadline()
        )
        with self.assertRaisesRegex(
            self.namespace["CFError"], "same deadline"
        ):
            self.Composite(sealed, other_authority, ledger)
        self.assertFalse(handle._used)
        other_authority.close(ledger)
        composite.close(ledger)
        self.assert_closed(ledger)
        self.assert_closed(foreign)

    def test_handle_release_precedes_verification_child_construction(self):
        head = self.signed_head()
        self.write_policy()
        deadline = ReentryDeadline()
        ledger = self.ledger("composite midpoint release")
        sealed = self.prepare_sealed(ledger, head, deadline)
        authority = self.Authority(self.project, ledger, deadline)
        composite = self.Composite(sealed, authority, ledger)
        handle = sealed._borrow_handles["signature"]
        verification_type = self.namespace["AccountedServerVerification"]
        original_close = handle.close
        original_cwd = verification_type._open_cwd_capability
        released = []
        release_callbacks = []

        def during_release(_phase):
            release_callbacks.append(True)
            self.assertIsNone(authority._verification_owner)
            with self.assertRaisesRegex(
                self.namespace["CFError"], "operation is in progress"
            ):
                composite.close(ledger)

        def observe_close(selected=None):
            self.assertIsNone(authority._verification_owner)
            deadline.callback = during_release
            deadline.armed = True
            result = original_close(selected)
            released.append(True)
            return result

        def observe_cwd(*args):
            self.assertEqual(released, [True])
            self.assertEqual(handle._token, 0)
            return original_cwd(*args)

        handle.close = observe_close
        verification_type._open_cwd_capability = staticmethod(observe_cwd)
        try:
            verification = composite.prepare("main", head, ledger)
        finally:
            verification_type._open_cwd_capability = staticmethod(original_cwd)
            handle.close = original_close
        self.assertEqual(released, [True])
        self.assertEqual(release_callbacks, [True])
        self.complete_success(ledger, composite, verification)

    def test_constructor_parent_mirrors_are_callback_free(self):
        head = self.signed_head()
        self.write_policy()
        authority_type = self.Authority
        sealed_type = self.namespace["AccountedHTTPUploadSealedValidation"]
        original_sealed_setattr = sealed_type.__setattr__
        original_setattr = authority_type.__setattr__
        ledger = self.ledger("composite callback-free token bind")
        sealed = self.prepare_sealed(ledger, head)
        authority = self.Authority(self.project, ledger)
        calls = []

        def observe_sealed(owner, name, value):
            if owner is sealed and name == "_upload_terminal_owner":
                calls.append("sealed")
                raise KeyboardInterrupt("replaceable sealed mirror")
            return original_sealed_setattr(owner, name, value)

        def observe_policy(owner, name, value):
            if owner is authority and name == "_upload_terminal_owner":
                calls.append("policy")
                raise KeyboardInterrupt("replaceable policy mirror")
            return original_setattr(owner, name, value)

        sealed_type.__setattr__ = observe_sealed
        authority_type.__setattr__ = observe_policy
        try:
            composite = self.Composite(sealed, authority, ledger)
        finally:
            sealed_type.__setattr__ = original_sealed_setattr
            authority_type.__setattr__ = original_setattr
        self.assertEqual(calls, [])
        self.assertIs(sealed._upload_terminal_owner, composite)
        self.assertIs(authority._upload_terminal_owner, composite)
        composite.close(ledger)
        gc.collect()
        self.assert_closed(ledger)

    def test_parent_coordinator_tokens_reject_stale_aliases(self):
        head = self.signed_head()
        self.write_policy()
        ledger = self.ledger("composite parent token")
        sealed = self.prepare_sealed(ledger, head)
        authority = self.Authority(self.project, ledger)
        composite = self.Composite(sealed, authority, ledger)

        operations = (
            lambda: sealed.borrow_commits(ledger, "signature"),
            lambda: sealed.finish(ledger),
            lambda: sealed.close(ledger),
            lambda: authority.prepare_policy(lambda _commit: None, ledger),
            lambda: authority.prepare_verified_server_verification(
                lambda _commit: None, "main", head, ledger
            ),
            lambda: authority.revalidate(ledger),
            lambda: authority.close(ledger),
        )
        for operation in operations:
            with self.assertRaisesRegex(
                self.namespace["CFError"], "upload terminal composite owns"
            ):
                operation()
        self.assertFalse(sealed._borrow_handles["signature"]._used)
        self.assertFalse(authority._policy_bound)
        composite.close(ledger)
        self.assert_closed(ledger)

    def test_operation_guard_assignment_faults_remain_closeable(self):
        head = self.signed_head()
        self.write_policy()
        composite_type = self.Composite
        original_setattr = composite_type.__setattr__

        for guard_name in ("_operating", "_closing"):
            for after_effect in (False, True):
                with self.subTest(
                    guard_name=guard_name, after_effect=after_effect
                ):
                    ledger = self.ledger(
                        f"composite {guard_name} "
                        f"{'post' if after_effect else 'pre'}"
                    )
                    sealed = self.prepare_sealed(ledger, head)
                    authority = self.Authority(self.project, ledger)
                    composite = self.Composite(sealed, authority, ledger)

                    def fail_guard(owner, name, value):
                        if (
                            owner is composite
                            and name == guard_name
                            and value is True
                        ):
                            if after_effect:
                                original_setattr(owner, name, value)
                            raise KeyboardInterrupt(
                                "synthetic operation guard failure"
                            )
                        return original_setattr(owner, name, value)

                    composite_type.__setattr__ = fail_guard
                    try:
                        with self.assertRaisesRegex(
                            KeyboardInterrupt, "operation guard failure"
                        ):
                            if guard_name == "_operating":
                                composite.prepare("main", head, ledger)
                            else:
                                composite.close(ledger)
                    finally:
                        composite_type.__setattr__ = original_setattr
                    self.assertFalse(composite._operating)
                    self.assertFalse(composite._closing)
                    composite.close(ledger)
                    self.assert_closed(ledger)

        sealed_type = self.namespace["AccountedHTTPUploadSealedValidation"]
        original_sealed_setattr = sealed_type.__setattr__
        for after_effect in (False, True):
            with self.subTest(
                guard_name="sealed_active", after_effect=after_effect
            ):
                ledger = self.ledger(
                    f"composite sealed guard "
                    f"{'post' if after_effect else 'pre'}"
                )
                sealed = self.prepare_sealed(ledger, head)
                authority = self.Authority(self.project, ledger)
                composite = self.Composite(sealed, authority, ledger)

                def fail_sealed_guard(owner, name, value):
                    if (
                        owner is sealed
                        and name == "_upload_terminal_active"
                        and value is True
                    ):
                        if after_effect:
                            original_sealed_setattr(owner, name, value)
                        raise KeyboardInterrupt(
                            "synthetic sealed operation guard failure"
                        )
                    return original_sealed_setattr(owner, name, value)

                sealed_type.__setattr__ = fail_sealed_guard
                try:
                    with self.assertRaisesRegex(
                        KeyboardInterrupt, "sealed operation guard failure"
                    ):
                        composite.prepare("main", head, ledger)
                finally:
                    sealed_type.__setattr__ = original_sealed_setattr
                self.assertFalse(composite._operating)
                self.assertFalse(sealed._upload_terminal_active)
                composite.close(ledger)
                self.assert_closed(ledger)

    def test_authority_guard_clear_faults_are_callback_free(self):
        head = self.signed_head()
        self.write_policy()
        authority_type = self.Authority
        original_setattr = authority_type.__setattr__

        for after_effect in (False, True):
            with self.subTest(after_effect=after_effect):
                ledger = self.ledger(
                    f"composite authority guard "
                    f"{'post' if after_effect else 'pre'}"
                )
                sealed = self.prepare_sealed(ledger, head)
                authority = self.Authority(self.project, ledger)
                composite = self.Composite(sealed, authority, ledger)

                def fail_handoff_close(owner, name, value):
                    if (
                        owner is authority
                        and name == "_verification_handoff_open"
                        and value is False
                    ):
                        if after_effect:
                            original_setattr(owner, name, value)
                        raise KeyboardInterrupt(
                            "synthetic authority guard clear failure"
                        )
                    return original_setattr(owner, name, value)

                authority_type.__setattr__ = fail_handoff_close
                try:
                    with self.assertRaisesRegex(
                        KeyboardInterrupt, "authority guard clear failure"
                    ):
                        composite.prepare("main", head, ledger)
                finally:
                    authority_type.__setattr__ = original_setattr
                self.assertFalse(authority._verification_handoff_open)
                self.assertFalse(authority._verification_constructing)
                self.assertFalse(sealed._upload_terminal_active)
                self.assertFalse(composite._operating)
                composite.close(ledger)
                self.assert_closed(ledger)

    def test_authority_cleanup_failure_blocks_sealed_cleanup_until_retry(self):
        head = self.signed_head()
        self.write_policy()
        ledger = self.ledger("composite cleanup order")
        sealed = self.prepare_sealed(ledger, head)
        authority = self.Authority(self.project, ledger)
        composite = self.Composite(sealed, authority, ledger)
        original_authority_close = authority.close
        original_sealed_close = sealed.close
        calls = []

        def fail_authority_once(selected=None, **kwargs):
            calls.append("authority")
            if calls == ["authority"]:
                raise RuntimeError("synthetic authority cleanup failure")
            return original_authority_close(selected, **kwargs)

        def observe_sealed_close(selected, **kwargs):
            calls.append("sealed")
            return original_sealed_close(selected, **kwargs)

        authority.close = fail_authority_once
        sealed.close = observe_sealed_close
        try:
            with self.assertRaisesRegex(RuntimeError, "authority cleanup failure"):
                composite.close(ledger)
            self.assertEqual(calls, ["authority"])
            self.assertFalse(sealed._closed)
            composite.close(ledger)
        finally:
            authority.close = original_authority_close
            sealed.close = original_sealed_close
        self.assertEqual(calls, ["authority", "authority", "sealed"])
        self.assert_closed(ledger)

    def test_signature_failure_releases_handle_before_any_verification_plan(self):
        self.write_policy(required=True, checks=[{"command": "exit 0"}])
        ledger = self.ledger("composite signature failure")
        sealed = self.prepare_sealed(ledger, self.unsigned_head)
        authority = self.Authority(self.project, ledger)
        composite = self.Composite(sealed, authority, ledger)
        verification_type = self.namespace["AccountedServerVerification"]
        original_cwd = verification_type._open_cwd_capability
        original_spawn = verification_type._spawn_process
        cwd_calls = []
        spawn_calls = []

        def observe_cwd(*args):
            cwd_calls.append(args)
            return original_cwd(*args)

        def observe_spawn(*args):
            spawn_calls.append(args)
            return original_spawn(*args)

        verification_type._open_cwd_capability = staticmethod(observe_cwd)
        verification_type._spawn_process = staticmethod(observe_spawn)
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"], "signature is required"
            ):
                composite.prepare("main", self.unsigned_head, ledger)
        finally:
            verification_type._open_cwd_capability = staticmethod(original_cwd)
            verification_type._spawn_process = staticmethod(original_spawn)
        self.assertEqual(cwd_calls, [])
        self.assertEqual(spawn_calls, [])
        self.assertEqual(sealed._borrow_handles["signature"]._token, 0)
        self.assertFalse(sealed._upload_terminal_active)
        composite.close(ledger)
        self.assert_closed(ledger)

    def test_config_failure_keeps_partial_child_and_releases_handle(self):
        head = self.signed_head()
        self.write_policy(checks=[{"id": "bad", "command": ["not", "text"]}])
        ledger = self.ledger("composite config failure")
        sealed = self.prepare_sealed(ledger, head)
        authority = self.Authority(self.project, ledger)
        composite = self.Composite(sealed, authority, ledger)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "command must be a string"
        ):
            composite.prepare("main", head, ledger)
        child = authority._verification_owner
        self.assertIsNotNone(child)
        self.assertTrue(child._closed)
        self.assertEqual(sealed._borrow_handles["signature"]._token, 0)
        self.assertFalse(sealed._upload_terminal_active)
        composite.close(ledger)
        self.assert_closed(ledger)

    def test_callbacks_cannot_close_or_finalize_either_owner(self):
        head = self.signed_head()
        self.write_policy()
        deadline = ReentryDeadline()
        ledger = self.ledger("composite callback reentry")
        sealed = self.prepare_sealed(ledger, head, deadline)
        authority = self.Authority(self.project, ledger, deadline)
        composite = self.Composite(sealed, authority, ledger)
        observations = []

        def reenter(_phase):
            for operation in (
                lambda: authority.revalidate(ledger),
                lambda: authority.close(ledger),
                lambda: sealed.finish(ledger),
                lambda: sealed.close(ledger),
                lambda: composite.close(ledger),
            ):
                with self.assertRaisesRegex(
                    self.namespace["CFError"],
                    "composite|handoff|construction|in progress",
                ):
                    operation()
            observations.append(
                (
                    authority._closed,
                    authority._revalidated,
                    sealed._closed,
                    sealed._upload_terminal_active,
                )
            )

        deadline.callback = reenter
        deadline.armed = True
        verification = composite.prepare("main", head, ledger)
        self.assertEqual(observations, [(False, False, False, True)])
        self.assertFalse(sealed._upload_terminal_active)
        self.complete_success(ledger, composite, verification)

    def test_handle_activation_failure_leaves_both_parents_closeable(self):
        head = self.signed_head()
        self.write_policy()
        ledger = self.ledger("composite activation failure")
        sealed = self.prepare_sealed(ledger, head)
        authority = self.Authority(self.project, ledger)
        composite = self.Composite(sealed, authority, ledger)
        handle = sealed._borrow_handles["signature"]
        original_activate = handle._activate

        def fail_after_activate(token):
            original_activate(token)
            raise RuntimeError("synthetic activation failure")

        handle._activate = fail_after_activate
        try:
            with self.assertRaisesRegex(RuntimeError, "activation failure"):
                composite.prepare("main", head, ledger)
        finally:
            handle._activate = original_activate
        self.assertTrue(sealed._borrow_failed)
        self.assertEqual(sealed._active_borrow_token, 0)
        self.assertFalse(sealed._upload_terminal_active)
        self.assertFalse(authority._verification_constructing)
        composite.close(ledger)
        self.assert_closed(ledger)

    def test_handle_release_failure_is_reachable_for_retry(self):
        head = self.signed_head()
        self.write_policy()
        ledger = self.ledger("composite handle release retry")
        sealed = self.prepare_sealed(ledger, head)
        authority = self.Authority(self.project, ledger)
        composite = self.Composite(sealed, authority, ledger)
        handle = sealed._borrow_handles["signature"]
        original_close = handle.close
        failed = []

        def fail_twice(selected=None):
            failed.append(True)
            if len(failed) <= 2:
                raise RuntimeError("synthetic handle close failure")
            return original_close(selected)

        handle.close = fail_twice
        try:
            with self.assertRaisesRegex(RuntimeError, "handle close failure"):
                composite.prepare("main", head, ledger)
        finally:
            handle.close = original_close
        self.assertEqual(failed, [True, True])
        self.assertNotEqual(handle._token, 0)
        self.assertFalse(sealed._upload_terminal_active)
        self.assertIsNone(authority._verification_owner)
        composite.close(ledger)
        self.assert_closed(ledger)

    def test_silent_release_noop_cannot_publish_verification_child(self):
        head = self.signed_head()
        self.write_policy()
        ledger = self.ledger("composite silent release")
        sealed = self.prepare_sealed(ledger, head)
        authority = self.Authority(self.project, ledger)
        composite = self.Composite(sealed, authority, ledger)
        handle = sealed._borrow_handles["signature"]
        original_close = handle.close
        verification_type = self.namespace["AccountedServerVerification"]
        original_cwd = verification_type._open_cwd_capability
        close_calls = []
        cwd_calls = []

        def silent_close(_selected=None):
            close_calls.append(True)

        def observe_cwd(*args):
            cwd_calls.append(args)
            return original_cwd(*args)

        handle.close = silent_close
        verification_type._open_cwd_capability = staticmethod(observe_cwd)
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"], "borrow release is incomplete"
            ):
                composite.prepare("main", head, ledger)
        finally:
            verification_type._open_cwd_capability = staticmethod(original_cwd)
            handle.close = original_close
        self.assertEqual(close_calls, [True, True])
        self.assertEqual(cwd_calls, [])
        self.assertNotEqual(handle._token, 0)
        self.assertIsNone(authority._verification_owner)
        self.assertTrue(composite._abort_only)
        composite.close(ledger)
        self.assert_closed(ledger)

    def test_foreign_verification_return_is_never_published_to_caller(self):
        head = self.signed_head()
        self.write_policy()
        ledger = self.ledger("composite foreign verification return")
        sealed = self.prepare_sealed(ledger, head)
        authority = self.Authority(self.project, ledger)
        composite = self.Composite(sealed, authority, ledger)
        original_core = authority._prepare_verified_server_verification_core
        foreign = object()

        def substitute_return(*args, **kwargs):
            original_core(*args, **kwargs)
            return foreign

        authority._prepare_verified_server_verification_core = substitute_return
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"], "foreign verification owner"
            ):
                composite.prepare("main", head, ledger)
        finally:
            authority._prepare_verified_server_verification_core = original_core
        real_child = authority._verification_owner
        self.assertIsInstance(
            real_child, self.namespace["AccountedServerVerification"]
        )
        self.assertIsNot(real_child, foreign)
        self.assertEqual(sealed._borrow_handles["signature"]._token, 0)
        composite.close(ledger)
        self.assertTrue(real_child._closed)
        self.assert_closed(ledger)

    def test_post_activation_wrapper_failure_cannot_strand_borrow(self):
        head = self.signed_head()
        self.write_policy()
        ledger = self.ledger("composite post activation failure")
        sealed = self.prepare_sealed(ledger, head)
        authority = self.Authority(self.project, ledger)
        composite = self.Composite(sealed, authority, ledger)
        handle = sealed._borrow_handles["signature"]
        original_borrow = sealed.borrow_commits

        def fail_after_borrow(*args, **kwargs):
            original_borrow(*args, **kwargs)
            raise RuntimeError("synthetic post activation failure")

        sealed.borrow_commits = fail_after_borrow
        try:
            with self.assertRaisesRegex(RuntimeError, "post activation failure"):
                composite.prepare("main", head, ledger)
        finally:
            sealed.borrow_commits = original_borrow
        self.assertTrue(handle._used)
        self.assertEqual(handle._token, 0)
        self.assertEqual(sealed._active_borrow_token, 0)
        self.assertIsNone(authority._verification_owner)
        composite.close(ledger)
        self.assert_closed(ledger)

    def test_post_begin_wrapper_failure_cannot_strand_authority_guard(self):
        head = self.signed_head()
        self.write_policy()
        ledger = self.ledger("composite post begin failure")
        sealed = self.prepare_sealed(ledger, head)
        authority = self.Authority(self.project, ledger)
        composite = self.Composite(sealed, authority, ledger)
        original_begin = authority._begin_verified_server_verification

        def fail_after_begin(*args, **kwargs):
            original_begin(*args, **kwargs)
            raise KeyboardInterrupt("synthetic post begin failure")

        authority._begin_verified_server_verification = fail_after_begin
        try:
            with self.assertRaisesRegex(KeyboardInterrupt, "post begin failure"):
                composite.prepare("main", head, ledger)
        finally:
            authority._begin_verified_server_verification = original_begin
        self.assertFalse(authority._verification_constructing)
        self.assertFalse(sealed._upload_terminal_active)
        self.assertFalse(sealed._borrow_handles["signature"]._used)
        composite.close(ledger)
        self.assert_closed(ledger)

    def test_lookup_failure_drops_payload_traceback_and_releases_handle(self):
        head = self.signed_head()
        self.write_policy()
        ledger = self.ledger("composite lookup failure")
        sealed = self.prepare_sealed(ledger, head)
        authority = self.Authority(self.project, ledger)
        composite = self.Composite(sealed, authority, ledger)
        original_borrow = sealed._borrow_commit

        def fail_lookup(*_args):
            raise RuntimeError("synthetic sealed lookup failure")

        sealed._borrow_commit = fail_lookup
        try:
            with self.assertRaisesRegex(RuntimeError, "lookup failure"):
                composite.prepare("main", head, ledger)
        finally:
            sealed._borrow_commit = original_borrow
        self.assertEqual(sealed._borrow_handles["signature"]._token, 0)
        self.assertFalse(sealed._upload_terminal_active)
        composite.close(ledger)
        self.assert_closed(ledger)


if __name__ == "__main__":
    unittest.main()
