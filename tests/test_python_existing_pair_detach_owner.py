import json
import os
import runpy
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class PairDeadline:
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


class PythonExistingPairDetachTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ns = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.ns["DecisionWorkLedger"]
        cls.Profile = cls.ns["DecisionWorkProfile"]
        cls.Authority = cls.ns["AccountedRemoteSignaturePolicyAuthority"]
        cls.Owner = cls.ns["AccountedHTTPUploadExistingTerminalOwner"]

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
            (self.tmp / ".codefire/branches/main.json").read_text()
        )["head"]
        payload = self.ns["load_object_from_root"](self.objects, self.initial)
        child = json.loads(json.dumps(payload))
        child["message"] = "existing pair detach"
        child["parents"] = [self.initial]
        self.key = "pair-detach-secret"
        child = self.ns["sign_commit_payload"](child, "alice", self.key)
        self.head = self.ns["store_object"](self.tmp, "commit", child)
        self.project = self.tmp / "remote/org/app"
        self.project.mkdir(parents=True)
        self.policy_path = self.project / "server_policy.json"

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def _write_policy(self):
        policy = {
            "commit_signatures": {
                "required": True,
                "keys": {"alice": self.key},
            },
            "verification": {"required": []},
        }
        self.policy_path.write_bytes(
            json.dumps(policy, separators=(",", ":")).encode()
        )

    def _prepare(self, label="existing pair detach", branch="main"):
        self._write_policy()
        ledger = self.Ledger(
            self.Profile.http_v1(), label, command_kind="http_request"
        )
        deadline = PairDeadline()
        paths = list(self.objects.glob("*/*.json"))
        sizes = [path.stat().st_size for path in paths]
        sealed = self.ns["prepare_http_upload_sealed_validation"](
            ledger,
            self.objects,
            self.head,
            record_count_upper=len(sizes),
            aggregate_bytes_upper=sum(sizes),
            maximum_record_bytes_upper=max(sizes),
            deadline=deadline,
        )
        sealed.validate(ledger)
        policy = self.Authority(self.project, ledger, deadline)
        owner = self.Owner(sealed, policy, ledger)
        owner.verify_signature_and_server(branch, self.head, ledger)
        self.assertTrue(owner.contains_ancestor(self.initial, self.head, ledger))
        return ledger, deadline, sealed, policy, owner

    def _assert_zero(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))
        self.assertIsNone(ledger._upload_terminal_owner)

    def _assert_pair_live(self, ledger, owner):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))
        self.assertIs(ledger._upload_terminal_owner, owner)
        self.assertFalse(owner._closed)
        self.assertTrue(owner._pair_resources_released)

    def test_success_orders_probes_then_commits_and_closes(self):
        ledger, _deadline, sealed, policy, owner = self._prepare()
        calls = []
        sealed_probe = sealed._probe_upload_terminal_pair
        policy_probe = policy._probe_upload_terminal_pair
        sealed_commit = sealed._commit_upload_terminal_pair
        policy_commit = policy._commit_upload_terminal_pair
        sealed_type = type(sealed)
        policy_type = type(policy)
        sealed_type._probe_upload_terminal_pair = lambda selected, token, *args: (
            calls.append("sealed-probe"), sealed_probe(token, *args)
        )[-1]
        policy_type._probe_upload_terminal_pair = lambda selected, token, *args: (
            calls.append("policy-probe"), policy_probe(token, *args)
        )[-1]
        sealed_type._commit_upload_terminal_pair = lambda selected, token: (
            calls.append("sealed-commit"), sealed_commit(token)
        )[-1]
        policy_type._commit_upload_terminal_pair = lambda selected, token: (
            calls.append("policy-commit"), policy_commit(token)
        )[-1]
        try:
            owner.finalize_pair(ledger)
        finally:
            sealed_type._probe_upload_terminal_pair = sealed_probe.__func__
            policy_type._probe_upload_terminal_pair = policy_probe.__func__
            sealed_type._commit_upload_terminal_pair = sealed_commit.__func__
            policy_type._commit_upload_terminal_pair = policy_commit.__func__
        self.assertEqual(
            calls,
            ["sealed-probe", "policy-probe", "sealed-commit", "policy-commit"],
        )
        self.assertEqual(owner._state, owner.PAIR_DETACHED)
        self._assert_pair_live(ledger, owner)
        owner.close(ledger)
        self._assert_zero(ledger)

    def test_last_callback_cannot_replace_captured_critical_functions(self):
        ledger, deadline, sealed, policy, owner = self._prepare(
            "pair captured critical functions"
        )
        object_path = self.ns["object_record_path"](self.objects, self.head)
        original_object = object_path.read_bytes()
        sealed_type = type(sealed)
        policy_type = type(policy)
        slot_type = self.ns["_CallbackFreeFD"]
        attributes = (
            (sealed_type, "_probe_upload_terminal_pair"),
            (policy_type, "_probe_upload_terminal_pair"),
            (sealed_type, "_commit_upload_terminal_pair"),
            (policy_type, "_commit_upload_terminal_pair"),
            (sealed_type, "_probe_pair_store"),
            (policy_type, "_probe_pair_project"),
            (sealed_type, "_pair_changed"),
            (sealed_type, "_borrow_mutation_failure"),
            (policy_type, "_fingerprint"),
            (policy_type, "_safe_policy_metadata"),
            (slot_type, "open"),
            (slot_type, "close"),
        )
        originals = {
            (target, name): vars(target)[name] for target, name in attributes
        }
        forged_commits = []

        def mutate_and_replace(_phase):
            replacement = object_path.with_suffix(".replacement")
            replacement.write_bytes(original_object)
            os.replace(replacement, object_path)
            self.policy_path.write_bytes(
                b'{"commit_signatures":{"required":false}}'
            )
            sealed_type._probe_upload_terminal_pair = lambda *_args: None
            policy_type._probe_upload_terminal_pair = lambda *_args: None
            sealed_type._commit_upload_terminal_pair = (
                lambda *_args: forged_commits.append("sealed")
            )
            policy_type._commit_upload_terminal_pair = (
                lambda *_args: forged_commits.append("policy")
            )
            sealed_type._probe_pair_store = lambda *_args: None
            policy_type._probe_pair_project = lambda *_args: None
            sealed_type._pair_changed = lambda *_args: self.ns["CFError"](
                "forged sealed change"
            )
            sealed_type._borrow_mutation_failure = (
                lambda *_args: self.ns["CFError"]("forged borrow change")
            )
            policy_type._fingerprint = lambda _metadata: ()
            policy_type._safe_policy_metadata = lambda _metadata: True
            slot_type.open = lambda *_args, **_kwargs: -1
            slot_type.close = lambda *_args: None

        deadline.callback = mutate_and_replace
        deadline.match = "upload terminal pair probe"
        try:
            with self.assertRaisesRegex(
                self.ns["SealedValidationSubjectChanged"], "identity_changed"
            ):
                owner.finalize_pair(ledger)
        finally:
            for (target, name), original in originals.items():
                setattr(target, name, original)
        self.assertEqual(forged_commits, [])
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self.assertIsNone(owner._verified_branch)
        self.assertIsNone(owner._verified_head)
        self._assert_zero(ledger)

    def test_verified_subject_refs_survive_pair_until_explicit_close(self):
        branch = "".join(("ma", "in"))
        ledger, _deadline, _sealed, _policy, owner = self._prepare(
            "pair subject refs", branch=branch
        )
        self.assertIs(owner._verified_branch, branch)
        self.assertIs(owner._verified_head, self.head)
        owner.finalize_pair(ledger)
        self.assertIs(owner._verified_branch, branch)
        self.assertEqual(owner._verified_head, self.head)
        self._assert_pair_live(ledger, owner)
        owner.close(ledger)
        self.assertIsNone(owner._verified_branch)
        self.assertIsNone(owner._verified_head)
        self._assert_zero(ledger)

    def test_pair_rejects_pinned_head_drift_and_drops_subject_refs(self):
        ledger, _deadline, sealed, _policy, owner = self._prepare(
            "pair pinned subject drift"
        )
        sealed._head = self.initial
        with self.assertRaisesRegex(self.ns["CFError"], "pair subject changed"):
            owner.finalize_pair(ledger)
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self.assertIsNone(owner._verified_branch)
        self.assertIsNone(owner._verified_head)
        self._assert_zero(ledger)

    def test_last_callback_mutating_both_sides_reports_sealed_first(self):
        ledger, deadline, sealed, policy, owner = self._prepare(
            "pair diagnostic order"
        )
        object_path = self.ns["object_record_path"](self.objects, self.head)
        original_object = object_path.read_bytes()
        policy_calls = []
        original_policy_probe = policy._probe_upload_terminal_pair

        def mutate(_phase):
            replacement = object_path.with_suffix(".replacement")
            replacement.write_bytes(original_object)
            os.replace(replacement, object_path)
            self.policy_path.write_bytes(
                b'{"commit_signatures":{"required":false}}'
            )

        def observe_policy(token):
            policy_calls.append(True)
            return original_policy_probe(token)

        policy._probe_upload_terminal_pair = observe_policy
        deadline.callback = mutate
        deadline.match = "upload terminal pair probe"
        with self.assertRaisesRegex(
            self.ns["SealedValidationSubjectChanged"], "identity_changed"
        ):
            owner.finalize_pair(ledger)
        self.assertEqual(policy_calls, [])
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self._assert_zero(ledger)

    def test_last_callback_mutating_borrow_alias_is_rejected(self):
        ledger, deadline, sealed, _policy, owner = self._prepare(
            "pair borrowed alias"
        )
        document = sealed._session.documents[self.head]

        def mutate(_phase):
            document.record["payload"]["message"] = "same alias changed"

        deadline.callback = mutate
        deadline.match = "upload terminal pair probe"
        with self.assertRaisesRegex(
            self.ns["CFError"], "borrowed commit changed"
        ):
            owner.finalize_pair(ledger)
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self._assert_zero(ledger)

    def test_close_and_reentry_before_probe_are_typed_cancellation(self):
        ledger, deadline, sealed, policy, owner = self._prepare(
            "pair pre-probe cancellation"
        )
        probes = []
        sealed_probe = sealed._probe_upload_terminal_pair
        policy_probe = policy._probe_upload_terminal_pair
        sealed._probe_upload_terminal_pair = lambda token: (
            probes.append("sealed"), sealed_probe(token)
        )[-1]
        policy._probe_upload_terminal_pair = lambda token: (
            probes.append("policy"), policy_probe(token)
        )[-1]
        reentry = []

        def cancel(_phase):
            try:
                owner.finalize_pair(ledger)
            except self.ns["CFError"] as error:
                reentry.append(str(error))
            owner.close(ledger)

        deadline.callback = cancel
        deadline.match = "upload terminal pair probe"
        with self.assertRaisesRegex(self.ns["CFError"], "cancelled"):
            owner.finalize_pair(ledger)
        self.assertEqual(probes, [])
        self.assertEqual(len(reentry), 1)
        self.assertIn("in progress", reentry[0])
        self._assert_zero(ledger)

    def test_second_probe_failure_mints_no_success(self):
        ledger, deadline, sealed, policy, owner = self._prepare(
            "pair second probe failure"
        )
        before_cumulative = dict(ledger.cumulative)
        before_counters = dict(ledger.counters)
        documents = tuple(sealed._session.documents.values())
        disk_documents = tuple(
            document for document in documents if document.path is not None
        )
        borrowed_documents = tuple(
            document for document in documents if document.borrow_digest is not None
        )
        expected_disk_bytes = sum(
            document.physical_bytes for document in disk_documents
        )
        expected_borrowed_bytes = sum(
            document.retained_bytes for document in borrowed_documents
        )
        expected_total_bytes = expected_disk_bytes + 2 * expected_borrowed_bytes
        deadline.callback = lambda _phase: self.policy_path.write_bytes(
            b'{"commit_signatures":{"required":false}}'
        )
        deadline.match = "upload terminal pair probe"
        with self.assertRaises(self.ns["SignaturePolicySubjectChanged"]):
            owner.finalize_pair(ledger)
        self.assertFalse(sealed._pair_detached)
        self.assertFalse(policy._pair_detached)
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self.assertEqual(
            ledger.cumulative["snapshot_revalidation_records"]
            - before_cumulative.get("snapshot_revalidation_records", 0),
            len(disk_documents),
        )
        self.assertEqual(
            ledger.cumulative["snapshot_revalidation_bytes"]
            - before_cumulative.get("snapshot_revalidation_bytes", 0),
            expected_total_bytes,
        )
        self.assertEqual(
            ledger.counters["snapshot_revalidation_bytes"]
            - before_counters.get("snapshot_revalidation_bytes", 0),
            expected_disk_bytes,
        )
        self._assert_zero(ledger)

    def test_fault_after_first_commit_cleans_half_detach_without_success(self):
        ledger, _deadline, sealed, policy, owner = self._prepare(
            "pair half detach"
        )
        sealed_type = type(sealed)
        original_commit = sealed_type._commit_upload_terminal_pair

        def fail_after_commit(selected, token):
            original_commit(selected, token)
            raise KeyboardInterrupt("sealed commit post-effect")

        sealed_type._commit_upload_terminal_pair = fail_after_commit
        try:
            with self.assertRaisesRegex(
                KeyboardInterrupt, "sealed commit post-effect"
            ):
                owner.finalize_pair(ledger)
        finally:
            sealed_type._commit_upload_terminal_pair = original_commit
        self.assertTrue(sealed._pair_detached)
        self.assertFalse(policy._pair_detached)
        self.assertEqual(owner._state, owner.ABORT_ONLY)
        self._assert_zero(ledger)

    def test_pair_plan_bounds_cover_odd_lists_and_slot_objects(self):
        sealed_type = self.ns["AccountedHTTPUploadSealedValidation"]
        slot_type = self.ns["_CallbackFreeFD"]
        self.assertFalse(hasattr(slot_type(), "__dict__"))
        pointer_bytes = sys.getsizeof((None,)) - sys.getsizeof(tuple())
        for record_count in (0, 1, 3):
            with self.subTest(record_count=record_count):
                subdir_count = min(
                    record_count, len(self.ns["OBJECT_PREFIXES"])
                )
                document_tuple = tuple([None] * record_count)
                borrowed_tuple = tuple([None] * record_count)
                subdir_slots = tuple(
                    (str(index), None, slot_type())
                    for index in range(subdir_count)
                )
                root_slot = slot_type()
                file_slot = slot_type()
                actual_retained = (
                    sys.getsizeof(document_tuple)
                    + sys.getsizeof(borrowed_tuple)
                    + sys.getsizeof(subdir_slots)
                    + sum(sys.getsizeof(item) for item in subdir_slots)
                    + sum(
                        sys.getsizeof(slot)
                        for slot in (
                            root_slot,
                            file_slot,
                            *(item[2] for item in subdir_slots),
                        )
                    )
                )
                self.assertLessEqual(
                    actual_retained,
                    sealed_type._pair_plan_upper(record_count),
                )
                odd_dict = dict.fromkeys(range(record_count))
                actual_builder = sys.getsizeof(list(odd_dict))
                self.assertLessEqual(
                    actual_builder,
                    sealed_type._pair_builder_temporary_upper(record_count),
                )
                expected_tuple_bytes = (
                    sys.getsizeof(tuple()) + pointer_bytes * record_count
                )
                self.assertEqual(sys.getsizeof(document_tuple), expected_tuple_bytes)

    def test_policy_pair_slots_are_covered_from_construction_and_drop_first(self):
        ledger, _deadline, _sealed, policy, owner = self._prepare(
            "policy pair retained owner"
        )
        before = ledger.current.get("retained_bytes", 0)
        self.Authority._prepare_upload_terminal_pair(policy, owner)
        retained = self.Authority.pair_probe_retained_upper()
        self.assertEqual(
            ledger.current.get("retained_bytes", 0) - before,
            0,
        )
        self.assertEqual(
            retained,
            sys.getsizeof((None, None, None))
            + 3 * sys.getsizeof(object.__new__(self.ns["_CallbackFreeFD"])),
        )
        absolute = Path(os.path.abspath(self.project))
        path_bytes = len(os.fsencode(absolute))
        components = max(len(absolute.parts) - 1, 1)
        capability_retained, _ = self.Authority.capability_upper(self.project)
        self.assertEqual(
            capability_retained,
            self.Authority.CAPABILITY_RETAINED_BASE_BYTES
            + self.Authority.CAPABILITY_RETAINED_COMPONENT_BYTES
            * (components + 2)
            + path_bytes
            + retained,
        )
        fixed_probe_peak = (
            sys.getsizeof(bytearray())
            + sys.getsizeof(memoryview(bytearray()))
            + sys.getsizeof(self.ns["hashlib"].sha256())
        )
        self.assertLessEqual(
            fixed_probe_peak,
            self.Authority.PAIR_PROBE_TEMPORARY_BASE_BYTES,
        )
        transition = policy._capability_transition
        transition_type = type(transition)
        original_release = transition_type.release_all
        observed = []

        def observe_release(selected):
            if selected is transition:
                observed.append(
                    (
                        policy._pair_policy_scratch,
                        policy._pair_probe_fd_slots,
                    )
                )
            return original_release(selected)

        transition_type.release_all = observe_release
        try:
            policy.close(ledger, _upload_terminal_owner=owner)
        finally:
            transition_type.release_all = original_release
        self.assertEqual(observed, [(None, None)])
        owner.close(ledger)
        self._assert_zero(ledger)

    def test_overlay_document_is_not_counted_as_disk_revalidation(self):
        ledger, _deadline, sealed, policy, owner = self._prepare(
            "pair overlay accounting"
        )
        documents = tuple(sealed._session.documents.values())
        self.assertGreaterEqual(len(documents), 2)
        documents[0].path = None
        disk_documents = tuple(
            document for document in documents if document.path is not None
        )
        borrowed_documents = tuple(
            document for document in documents if document.borrow_digest is not None
        )
        disk_bytes = sum(document.physical_bytes for document in disk_documents)
        borrowed_bytes = sum(
            document.retained_bytes for document in borrowed_documents
        )
        policy_bytes = 0 if policy._policy_missing else len(policy._policy_raw)
        before_cumulative = dict(ledger.cumulative)
        before_counters = dict(ledger.counters)
        owner.finalize_pair(ledger)
        self.assertEqual(
            ledger.cumulative["snapshot_revalidation_records"]
            - before_cumulative.get("snapshot_revalidation_records", 0),
            len(disk_documents),
        )
        self.assertEqual(
            ledger.cumulative["snapshot_revalidation_bytes"]
            - before_cumulative.get("snapshot_revalidation_bytes", 0),
            disk_bytes + 2 * borrowed_bytes,
        )
        for counter, expected in (
            ("snapshot_revalidation_records", len(disk_documents)),
            ("snapshot_revalidation_bytes", disk_bytes),
            ("files_read", len(disk_documents) + (1 if policy_bytes else 0)),
            ("file_bytes_read", disk_bytes + policy_bytes),
            ("bytes_hashed", disk_bytes + 2 * borrowed_bytes),
        ):
            self.assertEqual(
                ledger.counters[counter] - before_counters.get(counter, 0),
                expected,
            )
        self._assert_pair_live(ledger, owner)
        owner.close(ledger)
        self._assert_zero(ledger)

    def test_instance_probe_wrappers_cannot_enter_critical_section(self):
        ledger, _deadline, sealed, policy, owner = self._prepare(
            "pair concrete critical calls"
        )
        calls = []
        sealed._probe_upload_terminal_pair = lambda _token: calls.append(
            "sealed-probe"
        )
        policy._probe_upload_terminal_pair = lambda _token: calls.append(
            "policy-probe"
        )
        sealed._probe_pair_store = lambda _session: calls.append(
            "sealed-helper"
        )
        policy._probe_pair_project = lambda: calls.append("policy-helper")
        owner.finalize_pair(ledger)
        self.assertEqual(calls, [])
        self._assert_pair_live(ledger, owner)
        owner.close(ledger)
        self._assert_zero(ledger)

    def test_probe_fd_slot_owns_post_open_fault(self):
        slot_type = self.ns["_CallbackFreeFD"]
        original_open = slot_type.open
        for after_effect in (False, True):
            with self.subTest(after_effect=after_effect):
                ledger, _deadline, _sealed, _policy, owner = self._prepare(
                    f"pair descriptor fault {after_effect}"
                )
                descriptors = []

                def fail_open(slot, *args, **kwargs):
                    if after_effect:
                        descriptor = original_open(slot, *args, **kwargs)
                        descriptors.append(descriptor)
                    raise KeyboardInterrupt("pair descriptor open")

                slot_type.open = fail_open
                try:
                    with self.assertRaisesRegex(
                        KeyboardInterrupt, "pair descriptor open"
                    ):
                        owner.finalize_pair(ledger)
                finally:
                    slot_type.open = original_open
                for descriptor in descriptors:
                    with self.assertRaises(OSError):
                        os.fstat(descriptor)
                self.assertEqual(owner._state, owner.ABORT_ONLY)
                self._assert_zero(ledger)

    def test_pair_claim_release_reconciles_post_effect_fault(self):
        for after_effect in (False, True):
            with self.subTest(after_effect=after_effect):
                ledger, _deadline, sealed, _policy, owner = self._prepare(
                    f"pair claim release {after_effect}"
                )
                original_release = sealed._release_memory_claim
                failed = False

                def fail_target(dimension, amount):
                    nonlocal failed
                    target = sealed._pair_probe_retained
                    if (
                        not failed
                        and dimension == "retained_bytes"
                        and target
                        and amount == target
                    ):
                        failed = True
                        if after_effect:
                            original_release(dimension, amount)
                        raise KeyboardInterrupt("pair claim release")
                    return original_release(dimension, amount)

                sealed._release_memory_claim = fail_target
                with self.assertRaisesRegex(
                    KeyboardInterrupt, "pair claim release"
                ):
                    owner.finalize_pair(ledger)
                self.assertTrue(failed)
                self.assertEqual(owner._state, owner.PAIR_DETACHED)
                self.assertEqual(
                    sealed._pair_probe_retained == 0,
                    after_effect,
                )
                sealed._release_memory_claim = original_release
                owner.finalize_pair(ledger)
                self._assert_pair_live(ledger, owner)
                owner.close(ledger)
                self._assert_zero(ledger)

    def test_borrowed_alias_probe_keeps_insertion_diagnostic_order(self):
        ledger, deadline, sealed, _policy, owner = self._prepare(
            "pair borrowed diagnostic order"
        )
        borrowed = [
            document
            for document in sealed._session.documents.values()
            if document.borrow_digest is not None
        ]
        self.assertGreaterEqual(len(borrowed), 2)
        expected = borrowed[0].object_id

        def mutate(_phase):
            for index, document in enumerate(borrowed[:2]):
                document.record["payload"]["pair-order"] = index

        deadline.callback = mutate
        deadline.match = "upload terminal pair probe"
        with self.assertRaisesRegex(self.ns["CFError"], expected):
            owner.finalize_pair(ledger)
        self._assert_zero(ledger)

    def test_policy_cleanup_faults_do_not_rerun_probes(self):
        for after_effect in (False, True):
            with self.subTest(after_effect=after_effect):
                ledger, _deadline, sealed, policy, owner = self._prepare(
                    f"pair policy cleanup {after_effect}"
                )
                original_close = policy.close
                calls = []

                def fail_once(*args, **kwargs):
                    calls.append(True)
                    if len(calls) == 1:
                        if after_effect:
                            original_close(*args, **kwargs)
                        raise KeyboardInterrupt("policy pair cleanup")
                    return original_close(*args, **kwargs)

                policy.close = fail_once
                with self.assertRaisesRegex(
                    KeyboardInterrupt, "policy pair cleanup"
                ):
                    owner.finalize_pair(ledger)
                self.assertEqual(owner._state, owner.PAIR_DETACHED)
                self.assertFalse(sealed._closed)
                self.assertTrue(sealed._pair_detached)
                self.assertTrue(policy._pair_detached)
                owner.finalize_pair(ledger)
                self._assert_pair_live(ledger, owner)
                owner.close(ledger)
                self._assert_zero(ledger)

    def test_sealed_cleanup_faults_do_not_rerun_probes(self):
        for after_effect in (False, True):
            with self.subTest(after_effect=after_effect):
                ledger, _deadline, sealed, policy, owner = self._prepare(
                    f"pair sealed cleanup {after_effect}"
                )
                original_close = sealed.close
                calls = []

                def fail_once(*args, **kwargs):
                    calls.append(True)
                    if len(calls) == 1:
                        if after_effect:
                            original_close(*args, **kwargs)
                        raise KeyboardInterrupt("sealed pair cleanup")
                    return original_close(*args, **kwargs)

                sealed.close = fail_once
                with self.assertRaisesRegex(
                    KeyboardInterrupt, "sealed pair cleanup"
                ):
                    owner.finalize_pair(ledger)
                self.assertTrue(policy._closed)
                self.assertTrue(sealed._pair_detached)
                self.assertTrue(policy._pair_detached)
                owner.finalize_pair(ledger)
                self._assert_pair_live(ledger, owner)
                owner.close(ledger)
                self._assert_zero(ledger)

    def test_stale_aliases_repeated_use_and_surface_are_closed(self):
        ledger, _deadline, sealed, policy, owner = self._prepare(
            "pair closed surface"
        )
        for operation in (
            lambda: sealed.finish(ledger),
            lambda: policy.revalidate(ledger),
        ):
            with self.assertRaisesRegex(self.ns["CFError"], "upload terminal"):
                operation()
        owner.finalize_pair(ledger)
        self._assert_pair_live(ledger, owner)
        with self.assertRaisesRegex(self.ns["CFError"], "single-use"):
            owner.finalize_pair(ledger)
        for name in (
            "sealed_owner",
            "policy_authority",
            "capability",
            "borrow",
            "publication",
            "semantic",
        ):
            self.assertFalse(hasattr(owner, name))
        owner.close(ledger)
        with self.assertRaisesRegex(self.ns["CFError"], "closed"):
            owner.finalize_pair(ledger)
        self._assert_zero(ledger)


if __name__ == "__main__":
    unittest.main()
