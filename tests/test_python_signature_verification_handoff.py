import json
import os
import runpy
import shutil
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class RecordingDeadline:
    def __init__(self):
        self.phases = []
        self.limit_ms = None
        self.started_at = 0.0
        self.last_observed = 0.0

    def check(self, phase):
        self.phases.append(phase)


class PythonSignatureVerificationHandoffTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.namespace = runpy.run_path(str(CODEFIRE))
        cls.Authority = cls.namespace[
            "AccountedRemoteSignaturePolicyAuthority"
        ]
        cls.Verification = cls.namespace["AccountedServerVerification"]
        cls.Ledger = cls.namespace["DecisionWorkLedger"]
        cls.Profile = cls.namespace["DecisionWorkProfile"]

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.project = self.tmp / "storage/org/app"
        self.project.mkdir(parents=True)
        self.policy_path = self.project / "server_policy.json"
        self.commit_id = "CF-COMMIT-" + "a" * 24

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def ledger(self, label="signature verification handoff"):
        return self.Ledger(
            self.Profile.http_v1(), label, command_kind="http_request"
        )

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def write_policy(self, *, signatures=None, checks=None):
        policy = {
            "commit_signatures": (
                {"required": False} if signatures is None else signatures
            ),
            "verification": {"required": [] if checks is None else checks},
        }
        raw = json.dumps(policy, separators=(",", ":")).encode("utf-8")
        self.policy_path.write_bytes(raw)
        return raw

    @staticmethod
    def forbidden_loader(_object_id):
        raise AssertionError("disabled signature policy called its loader")

    def test_exact_snapshot_is_lent_without_ambient_policy_read(self):
        self.write_policy(
            checks=[{"id": "p0", "command": "exit 0", "cwd": "."}]
        )
        ledger = self.ledger()
        deadline = RecordingDeadline()
        authority = self.Authority(self.project, ledger, deadline)
        original_read = self.Verification._read_policy
        reads = []

        def forbidden_read(*args, **kwargs):
            reads.append((args, kwargs))
            raise AssertionError("seeded verification reread ambient policy")

        self.Verification._read_policy = forbidden_read
        try:
            owner = authority.prepare_verified_server_verification(
                self.forbidden_loader,
                "main",
                self.commit_id,
                ledger,
            )
        finally:
            self.Verification._read_policy = original_read
        self.assertEqual(reads, [])
        self.assertIs(owner._policy_raw, authority._policy_raw)
        self.assertEqual(owner._plans[0][0:2], ("p0", "exit 0"))
        self.assertIs(owner._ledger, ledger)
        self.assertIs(owner._deadline, deadline)
        self.assertNotEqual(owner._project_fd, authority._project_fd)
        self.assertEqual(
            os.fstat(owner._project_fd).st_ino,
            os.fstat(authority._project_fd).st_ino,
        )
        self.assertNotEqual(owner._policy_fd, authority._policy_fd)
        self.assertEqual(
            os.fstat(owner._policy_fd).st_ino,
            os.fstat(authority._policy_fd).st_ino,
        )
        owner.run()
        owner.close()
        authority.revalidate(ledger)
        authority.close(ledger)
        self.assertTrue(deadline.phases)
        self.assert_closed(ledger)

    def test_signature_failure_precedes_plan_cwd_and_spawn(self):
        self.write_policy(
            signatures={"required": True, "keys": {"alice": "secret"}},
            checks=[{"id": "must-not-plan", "command": "exit 0"}],
        )
        unsigned = {"type": "commit", "version": 1, "parents": []}

        def loader(_object_id):
            return unsigned, self.namespace["canonical_json_size"](unsigned)

        ledger = self.ledger("signature failure before verification")
        authority = self.Authority(self.project, ledger)
        original_cwd = self.Verification._open_cwd_capability
        original_spawn = self.Verification._spawn_process
        cwd_calls = []
        spawn_calls = []

        def observe_cwd(*args):
            cwd_calls.append(args)
            return original_cwd(*args)

        def observe_spawn(*args):
            spawn_calls.append(args)
            return original_spawn(*args)

        self.Verification._open_cwd_capability = staticmethod(observe_cwd)
        self.Verification._spawn_process = staticmethod(observe_spawn)
        try:
            with self.assertRaisesRegex(
                self.namespace["CFError"], "signature is required"
            ):
                authority.prepare_verified_server_verification(
                    loader, "main", self.commit_id, ledger
                )
        finally:
            self.Verification._open_cwd_capability = staticmethod(original_cwd)
            self.Verification._spawn_process = staticmethod(original_spawn)
        authority.close(ledger)
        self.assertEqual(cwd_calls, [])
        self.assertEqual(spawn_calls, [])
        self.assert_closed(ledger)

    def test_public_begin_wrapper_fault_cannot_strand_construction_guard(self):
        self.write_policy()
        for after_effect in (False, True):
            with self.subTest(after_effect=after_effect):
                ledger = self.ledger(
                    f"public begin {'post' if after_effect else 'pre'} fault"
                )
                authority = self.Authority(self.project, ledger)
                original_begin = authority._begin_verified_server_verification

                def fail_begin(*args, **kwargs):
                    if after_effect:
                        original_begin(*args, **kwargs)
                    raise KeyboardInterrupt("synthetic public begin failure")

                authority._begin_verified_server_verification = fail_begin
                try:
                    with self.assertRaisesRegex(
                        KeyboardInterrupt, "public begin failure"
                    ):
                        authority.prepare_verified_server_verification(
                            self.forbidden_loader,
                            "main",
                            self.commit_id,
                            ledger,
                        )
                finally:
                    authority._begin_verified_server_verification = original_begin
                self.assertFalse(authority._verification_handoff_open)
                self.assertFalse(authority._verification_constructing)
                authority.close(ledger)
                self.assert_closed(ledger)

    def test_signature_loader_cannot_reenter_final_or_close(self):
        self.write_policy(
            signatures={"required": True, "keys": {"alice": "secret"}},
            checks=[{"id": "must-not-plan", "command": "exit 0"}],
        )
        unsigned = {"type": "commit", "version": 1, "parents": []}
        ledger = self.ledger("signature loader handoff reentry")
        authority = self.Authority(self.project, ledger)
        reentries = []

        def loader(_object_id):
            for operation in (
                lambda: authority.revalidate(ledger),
                lambda: authority.close(ledger),
            ):
                with self.assertRaisesRegex(
                    self.namespace["CFError"], "construction is in progress"
                ):
                    operation()
            reentries.append(True)
            return unsigned, self.namespace["canonical_json_size"](unsigned)

        with self.assertRaisesRegex(
            self.namespace["CFError"], "signature is required"
        ):
            authority.prepare_verified_server_verification(
                loader, "main", self.commit_id, ledger
            )
        self.assertEqual(reentries, [True])
        self.assertFalse(authority._verification_constructing)
        self.assertFalse(authority._revalidated)
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_children_are_nonoverlapping_and_each_single_use(self):
        self.write_policy(checks=[])
        ledger = self.ledger("one-shot child ordering")
        authority = self.Authority(self.project, ledger)
        owner = authority.prepare_verified_server_verification(
            self.forbidden_loader, "main", self.commit_id, ledger
        )
        self.assertIsNone(authority._policy_owner)
        self.assertIs(authority._verification_owner, owner)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "authority is single-use"
        ):
            authority.prepare_policy(self.forbidden_loader, ledger)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "must complete and close"
        ):
            authority.revalidate(ledger)
        owner.run()
        with self.assertRaisesRegex(
            self.namespace["CFError"], "must complete and close"
        ):
            authority.revalidate(ledger)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "owner is single-use"
        ):
            owner.run()
        owner.close()
        authority.revalidate(ledger)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "authority is single-use"
        ):
            authority.revalidate(ledger)
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_failed_verification_cannot_authorize_final_fence(self):
        self.write_policy(
            checks=[{"id": "failure", "command": "exit 7"}]
        )
        ledger = self.ledger("failed verification final fence")
        authority = self.Authority(self.project, ledger)
        owner = authority.prepare_verified_server_verification(
            self.forbidden_loader, "main", self.commit_id, ledger
        )
        with self.assertRaisesRegex(
            self.namespace["CFError"], "server verification failed"
        ):
            owner.run()
        owner.close()
        with self.assertRaisesRegex(
            self.namespace["CFError"], "must complete and close"
        ):
            authority.revalidate(ledger)
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_child_release_failure_retries_before_final_fence(self):
        self.write_policy(checks=[])
        ledger = self.ledger("verification child release retry")
        authority = self.Authority(self.project, ledger)
        owner = authority.prepare_verified_server_verification(
            self.forbidden_loader, "main", self.commit_id, ledger
        )
        owner.run()
        original_release = ledger.release
        raised = []

        def fail_after_release(dimension, amount):
            original_release(dimension, amount)
            if not raised:
                raised.append((dimension, amount))
                raise RuntimeError("synthetic child release failure")

        ledger.release = fail_after_release
        try:
            with self.assertRaisesRegex(RuntimeError, "child release failure"):
                owner.close()
        finally:
            ledger.release = original_release
        self.assertEqual(len(raised), 1)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "must complete and close"
        ):
            authority.revalidate(ledger)
        owner.close()
        authority.revalidate(ledger)
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_policy_or_project_change_rejects_before_spawn(self):
        for change in ("policy", "project"):
            with self.subTest(change=change):
                self.write_policy(
                    checks=[{"id": "p0", "command": "exit 0"}]
                )
                ledger = self.ledger(f"{change} subject change")
                authority = self.Authority(self.project, ledger)
                owner = authority.prepare_verified_server_verification(
                    self.forbidden_loader, "main", self.commit_id, ledger
                )
                if change == "policy":
                    replacement = self.policy_path.with_suffix(".replacement")
                    replacement.write_bytes(
                        json.dumps(
                            {
                                "commit_signatures": {"required": False},
                                "verification": {
                                    "required": [
                                        {"id": "p1", "command": "exit 9"}
                                    ]
                                },
                            },
                            separators=(",", ":"),
                        ).encode("utf-8")
                    )
                    os.replace(replacement, self.policy_path)
                else:
                    moved = self.project.with_name("app-original")
                    self.project.rename(moved)
                    self.project.mkdir()
                original_spawn = self.Verification._spawn_process
                spawn_calls = []

                def observe_spawn(*args):
                    spawn_calls.append(args)
                    return original_spawn(*args)

                self.Verification._spawn_process = staticmethod(observe_spawn)
                try:
                    with self.assertRaisesRegex(
                        self.namespace["CFError"], "changed before execution"
                    ):
                        owner.run()
                finally:
                    self.Verification._spawn_process = staticmethod(
                        original_spawn
                    )
                    owner.close()
                    authority.close(ledger)
                self.assertEqual(spawn_calls, [])
                self.assert_closed(ledger)
                if change == "project":
                    shutil.rmtree(self.project)
                    moved.rename(self.project)

    def test_missing_project_is_explicit_noop_and_appearance_fails(self):
        missing = self.tmp / "missing/org/app"
        missing.parent.mkdir(parents=True)
        ledger = self.ledger("missing bootstrap handoff")
        authority = self.Authority(missing, ledger)
        owner = authority.prepare_verified_server_verification(
            self.forbidden_loader,
            "main",
            self.commit_id,
            ledger,
            allow_missing_project=True,
        )
        self.assertEqual(owner._plans, [])
        missing.mkdir()
        with self.assertRaisesRegex(
            self.namespace["CFError"], "project changed before execution"
        ):
            owner.run()
        owner.close()
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_foreign_ledger_and_existing_as_missing_do_not_transfer(self):
        self.write_policy(checks=[])
        ledger = self.ledger("handoff ledger")
        foreign = self.ledger("foreign handoff ledger")
        authority = self.Authority(self.project, ledger)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "different ledger"
        ):
            authority.prepare_verified_server_verification(
                self.forbidden_loader,
                "main",
                self.commit_id,
                foreign,
            )
        with self.assertRaisesRegex(
            self.namespace["CFError"], "project changed before execution"
        ):
            authority.prepare_verified_server_verification(
                self.forbidden_loader,
                "main",
                self.commit_id,
                ledger,
                allow_missing_project=True,
            )
        authority.close(ledger)
        self.assert_closed(ledger)
        self.assert_closed(foreign)

    def test_seeded_capability_dup_failure_closes_partial_child(self):
        self.write_policy(
            checks=[{"id": "dup-failure", "command": "exit 0"}]
        )
        ledger = self.ledger("seeded capability duplication failure")
        authority = self.Authority(self.project, ledger)
        original_dup = os.dup
        duplicated = []

        def fail_second_dup(descriptor):
            if duplicated:
                raise OSError("synthetic policy dup failure")
            result = original_dup(descriptor)
            duplicated.append(result)
            return result

        os.dup = fail_second_dup
        try:
            with self.assertRaisesRegex(OSError, "policy dup failure"):
                authority.prepare_verified_server_verification(
                    self.forbidden_loader, "main", self.commit_id, ledger
                )
        finally:
            os.dup = original_dup
        self.assertEqual(len(duplicated), 1)
        with self.assertRaises(OSError):
            os.fstat(duplicated[0])
        self.assertIsNotNone(authority._policy_raw)
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_verification_config_failure_does_not_escape_or_replace_p0(self):
        raw = self.write_policy(
            checks=[{"id": "invalid-command", "command": ["not", "scalar"]}]
        )
        ledger = self.ledger("seeded verification config failure")
        authority = self.Authority(self.project, ledger)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "command must be a string"
        ):
            authority.prepare_verified_server_verification(
                self.forbidden_loader, "main", self.commit_id, ledger
            )
        self.assertEqual(authority._policy_raw, raw)
        self.assertIsNotNone(authority._verification_owner)
        self.assertTrue(authority._verification_owner._closed)
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_seeded_constructor_cannot_bypass_signature_handoff(self):
        self.write_policy(checks=[])
        ledger = self.ledger("internal handoff capability")
        authority = self.Authority(self.project, ledger)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "handoff is unavailable"
        ):
            self.Verification(
                self.project,
                "main",
                self.commit_id,
                ledger,
                _signature_policy_authority=authority,
            )
        self.assertFalse(authority._policy_bound)
        authority.close(ledger)
        self.assert_closed(ledger)

    def test_partial_constructor_cleanup_is_published_for_retry(self):
        self.write_policy(
            checks=[{"id": "bad", "command": ["not", "a", "string"]}]
        )
        ledger = self.ledger("partial child constructor cleanup")
        authority = self.Authority(self.project, ledger)
        original_release = ledger.release
        original_close_plans = self.Verification._close_plan_capabilities
        release_calls = []
        cleanup_armed = []

        def arm_constructor_cleanup(owner, plans):
            cleanup_armed.append(True)
            return original_close_plans(owner, plans)

        def fail_before_release(dimension, amount):
            if cleanup_armed:
                release_calls.append((dimension, amount))
                raise RuntimeError("synthetic constructor release failure")
            original_release(dimension, amount)

        ledger.release = fail_before_release
        self.Verification._close_plan_capabilities = arm_constructor_cleanup
        try:
            with self.assertRaisesRegex(
                RuntimeError, "constructor release failure"
            ) as raised:
                authority.prepare_verified_server_verification(
                    self.forbidden_loader, "main", self.commit_id, ledger
                )
        finally:
            ledger.release = original_release
            self.Verification._close_plan_capabilities = original_close_plans
        self.assertTrue(release_calls)
        self.assertIsInstance(raised.exception.__cause__, self.namespace["CFError"])
        child = authority._verification_owner
        self.assertIsNotNone(child)
        self.assertFalse(child._closed)
        authority.close(ledger)
        self.assertTrue(child._closed)
        self.assert_closed(ledger)

    def test_dup_reentry_cannot_consume_or_close_final_authority(self):
        self.write_policy(
            checks=[{"id": "reentry", "command": "exit 0"}]
        )
        ledger = self.ledger("dup reentry final fence")
        authority = self.Authority(self.project, ledger)
        original_dup = os.dup
        reentries = []

        def reenter_once(descriptor):
            if not reentries:
                child = authority._verification_owner
                self.assertIsNotNone(child)
                for operation in (
                    lambda: authority.revalidate(ledger),
                    lambda: authority.close(ledger),
                    child.run,
                    child.close,
                ):
                    with self.assertRaisesRegex(
                        self.namespace["CFError"], "construction is in progress"
                    ):
                        operation()
                reentries.append(True)
            return original_dup(descriptor)

        os.dup = reenter_once
        try:
            owner = authority.prepare_verified_server_verification(
                self.forbidden_loader, "main", self.commit_id, ledger
            )
        finally:
            os.dup = original_dup
        self.assertEqual(reentries, [True])
        self.assertFalse(authority._revalidated)
        self.assertIs(authority._verification_owner, owner)
        owner.run()
        owner.close()
        authority.revalidate(ledger)
        authority.close(ledger)
        self.assert_closed(ledger)


if __name__ == "__main__":
    unittest.main()
