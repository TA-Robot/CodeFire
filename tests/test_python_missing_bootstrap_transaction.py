import fcntl
import json
import os
import runpy
import shutil
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class BootstrapDeadline:
    def __init__(self):
        self.callback = None
        self.calls = []

    def check(self, phase):
        self.calls.append(phase)
        callback = self.callback
        self.callback = None
        if callback is not None:
            callback(phase)


class MissingProjectBootstrapTransactionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ns = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.ns["DecisionWorkLedger"]
        cls.Profile = cls.ns["DecisionWorkProfile"]
        cls.Owner = cls.ns[
            "AccountedHTTPUploadMissingProjectBootstrapTransaction"
        ]
        cls.Prepare = staticmethod(
            cls.ns[
                "prepare_http_upload_missing_project_bootstrap_transaction"
            ]
        )
        cls.CFError = cls.ns["CFError"]
        cls.ResourceBudget = cls.ns["ResourceBudgetExceeded"]

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.projects = self.tmp / "projects"
        self.org = "org"
        self.app = "app"
        self.project = self.projects / self.org / self.app
        self.project.parent.mkdir(parents=True)
        self.stable_id = "CF-PROJECT-0123456789abcdef0123456789abcdef"
        self.reductions = []

        def reducer(capability):
            self.reductions.append(capability)

        self.reducer = reducer

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def _ledger(self, overrides=None):
        return self.Ledger(
            self.Profile.http_v1(overrides),
            "missing project bootstrap",
            command_kind="http_request",
        )

    def _prepare(self, ledger=None, *, deadline=None, reducer=None):
        selected = self._ledger() if ledger is None else ledger
        owner = self.Prepare(
            self.project,
            self.org,
            self.app,
            self.stable_id,
            selected,
            self.reducer if reducer is None else reducer,
            deadline,
        )
        return selected, owner

    def _zero(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def _terminal(self, owner, ledger):
        ledger.seal_prepublication()
        return owner.prepare_terminal(ledger)

    def test_prepared_stage_is_hidden_gated_and_close_removes_it(self):
        ledger, owner = self._prepare()
        stage = owner._stage_path
        descriptor = json.loads(
            (stage / "remote_project.json").read_bytes()
        )
        self.assertEqual(owner._state, owner.PREPARED_HIDDEN)
        self.assertFalse(self.project.exists())
        self.assertTrue((stage / "locks/project-gate.lock").is_file())
        self.assertEqual(descriptor["stable_project_id"], self.stable_id)
        self.assertEqual(
            descriptor["creation_provenance"],
            "authenticated_bootstrap_transaction",
        )
        retained, temporary = self.Owner.live_upper(self.project)
        self.assertEqual(ledger.current["retained_bytes"], retained)
        self.assertEqual(ledger.current["temporary_bytes"], temporary)
        owner.close(ledger)
        self.assertFalse(stage.exists())
        self.assertFalse(self.project.exists())
        self._zero(ledger)

    def test_live_limits_are_exact_and_minus_one_stops_before_stage(self):
        retained, live_temporary = self.Owner.live_upper(self.project)
        _construction_retained, construction_temporary = (
            self.Owner.construction_upper(self.project)
        )
        exact = self._ledger(
            {
                "retained_bytes": retained,
                "temporary_bytes": construction_temporary,
            }
        )
        owner = self._prepare(exact)[1]
        owner.close(exact)
        self._zero(exact)

        for dimension, limit in (
            ("retained_bytes", retained - 1),
            ("temporary_bytes", construction_temporary - 1),
        ):
            with self.subTest(dimension=dimension):
                overrides = {
                    "retained_bytes": retained,
                    "temporary_bytes": construction_temporary,
                }
                overrides[dimension] = limit
                ledger = self._ledger(overrides)
                with self.assertRaises(self.ResourceBudget):
                    self._prepare(ledger)
                self.assertFalse(self.project.exists())
                self.assertFalse(any(self.project.parent.glob(".bootstrap-*")))
                self._zero(ledger)

    def test_invalid_identity_existing_target_and_reducer_leave_no_debt(self):
        cases = (
            ("CF-PROJECT-" + "A" * 32, self.org, self.app),
            (self.stable_id, "wrong", self.app),
            (self.stable_id, self.org, "wrong"),
        )
        for stable_id, organization, application in cases:
            with self.subTest(stable_id=stable_id, organization=organization):
                ledger = self._ledger()
                with self.assertRaises(self.CFError):
                    self.Prepare(
                        self.project,
                        organization,
                        application,
                        stable_id,
                        ledger,
                        self.reducer,
                    )
                self._zero(ledger)

        ledger = self._ledger()
        with self.assertRaisesRegex(self.CFError, "exact function"):
            self.Prepare(
                self.project,
                self.org,
                self.app,
                self.stable_id,
                ledger,
                object(),
            )
        self._zero(ledger)

        self.project.mkdir()
        ledger = self._ledger()
        with self.assertRaisesRegex(self.CFError, "already exists"):
            self._prepare(ledger)
        self.assertTrue(self.project.is_dir())
        self._zero(ledger)

    def test_hardlinked_bootstrap_lock_rejects_without_fd_or_stage_leak(self):
        ledger, owner = self._prepare()
        lock = self.projects / ".bootstrap-locks" / owner._bootstrap_lock_name
        owner.close(ledger)
        self._zero(ledger)
        os.link(lock, lock.with_name(lock.name + ".alias"))
        before = len(tuple(Path("/proc/self/fd").iterdir()))
        retry = self._ledger()
        with self.assertRaisesRegex(self.CFError, "single-link"):
            self._prepare(retry)
        after = len(tuple(Path("/proc/self/fd").iterdir()))
        self.assertEqual(after, before)
        self.assertFalse(self.project.exists())
        self.assertFalse(any(self.project.parent.glob(".bootstrap-*")))
        self._zero(retry)

    def test_stage_open_fault_uses_named_identity_for_cleanup(self):
        original = self.ns["_open_directory_at"]

        def fail_stage(directory_fd, name):
            if name.startswith(".bootstrap-"):
                raise OSError("stage open fault")
            return original(directory_fd, name)

        self.ns["_open_directory_at"].__globals__[
            "_open_directory_at"
        ] = fail_stage
        ledger = self._ledger()
        try:
            with self.assertRaisesRegex(OSError, "stage open fault"):
                self._prepare(ledger)
        finally:
            self.ns["_open_directory_at"].__globals__[
                "_open_directory_at"
            ] = original
        self.assertFalse(self.project.exists())
        self.assertFalse(any(self.project.parent.glob(".bootstrap-*")))
        self._zero(ledger)

    def test_terminal_is_same_ledger_sealed_and_single_use(self):
        ledger, owner = self._prepare()
        foreign = self._ledger()
        with self.assertRaisesRegex(self.CFError, "another ledger"):
            owner.prepare_terminal(foreign)
        with self.assertRaisesRegex(self.CFError, "sealed ledger"):
            owner.prepare_terminal(ledger)
        handoff = self._terminal(owner, ledger)
        self.assertEqual(owner._state, owner.TERMINAL_READY)
        with self.assertRaisesRegex(self.CFError, "unavailable"):
            owner.prepare_terminal(ledger)
        with self.assertRaisesRegex(self.CFError, "another ledger"):
            handoff.start_effect(foreign)
        owner.close(ledger)
        handoff.close(ledger)
        self._zero(ledger)
        self._zero(foreign)

    def test_fake_handoff_and_duplicate_transitions_are_rejected(self):
        ledger, owner = self._prepare()
        handoff = self._terminal(owner, ledger)
        fake_type = self.ns["_HTTPUploadMissingBootstrapHandoff"]
        fake = fake_type(owner, owner._token)
        with self.assertRaisesRegex(self.CFError, "proof changed"):
            fake.start_effect(ledger)
        handoff.start_effect(ledger)
        with self.assertRaisesRegex(self.CFError, "unavailable"):
            handoff.start_effect(ledger)
        handoff.rename_visible(ledger)
        with self.assertRaisesRegex(self.CFError, "unavailable"):
            handoff.rename_visible(ledger)
        handoff.reduce(ledger)
        with self.assertRaisesRegex(self.CFError, "unavailable"):
            handoff.reduce(ledger)
        handoff.close(ledger)
        self._zero(ledger)

    def test_exact_state_sequence_keeps_gate_and_publishes_same_inode(self):
        ledger, owner = self._prepare()
        stage_inode = owner._stage_identity
        gate_fd = owner._gate_fd
        gate_identity = owner._gate_identity
        handoff = self._terminal(owner, ledger)
        handoff.start_effect(ledger)
        self.assertEqual(owner._state, owner.EFFECT_STARTED)
        handoff.rename_visible(ledger)
        self.assertEqual(owner._state, owner.RENAMED_VISIBLE)
        self.assertEqual(
            (self.project.stat().st_dev, self.project.stat().st_ino),
            stage_inode,
        )
        self.assertEqual(owner._gate_fd, gate_fd)
        self.assertEqual(owner._gate_identity, gate_identity)
        self.assertFalse(owner._stage_path.exists())
        handoff.reduce(ledger)
        self.assertEqual(owner._state, owner.COMPLETE)
        self.assertEqual(len(self.reductions), 1)
        handoff.close(ledger)
        self.assertTrue(self.project.is_dir())
        self._zero(ledger)

    def test_gate_contention_is_continuous_before_and_after_rename(self):
        ledger, owner = self._prepare()
        handoff = self._terminal(owner, ledger)
        gate_path = owner._stage_path / "locks/project-gate.lock"
        contender = os.open(gate_path, os.O_RDWR | os.O_NOFOLLOW)
        try:
            with self.assertRaises(BlockingIOError):
                fcntl.flock(contender, fcntl.LOCK_EX | fcntl.LOCK_NB)
        finally:
            os.close(contender)
        handoff.start_effect(ledger)
        handoff.rename_visible(ledger)
        contender = os.open(
            self.project / "locks/project-gate.lock",
            os.O_RDWR | os.O_NOFOLLOW,
        )
        try:
            with self.assertRaises(BlockingIOError):
                fcntl.flock(contender, fcntl.LOCK_EX | fcntl.LOCK_NB)
        finally:
            os.close(contender)
        handoff.reduce(ledger)
        handoff.close(ledger)
        self._zero(ledger)

    def test_set_then_raise_rename_reconciles_without_second_rename(self):
        ledger, owner = self._prepare()
        handoff = self._terminal(owner, ledger)
        handoff.start_effect(ledger)
        original = self.Owner.RENAME_NOREPLACE
        calls = 0

        def rename_then_raise(*args):
            nonlocal calls
            calls += 1
            original(*args)
            raise RuntimeError("post-rename fault")

        self.Owner.RENAME_NOREPLACE = staticmethod(rename_then_raise)
        try:
            handoff.rename_visible(ledger)
        finally:
            self.Owner.RENAME_NOREPLACE = staticmethod(original)
        self.assertEqual(calls, 1)
        self.assertEqual(owner._state, owner.RENAMED_VISIBLE)
        handoff.reduce(ledger)
        handoff.close(ledger)
        self.assertTrue(self.project.exists())
        self._zero(ledger)

    def test_exact_relative_rename_uses_pinned_org_descriptor(self):
        ledger, owner = self._prepare()
        handoff = self._terminal(owner, ledger)
        handoff.start_effect(ledger)
        original = self.Owner.RENAME_NOREPLACE
        observed = []

        def observe(directory_fd, source, destination):
            observed.append((directory_fd, source, destination))
            return original(directory_fd, source, destination)

        self.Owner.RENAME_NOREPLACE = staticmethod(observe)
        try:
            handoff.rename_visible(ledger)
        finally:
            self.Owner.RENAME_NOREPLACE = staticmethod(original)
        self.assertEqual(
            observed,
            [(owner._org_fd, owner._stage_name, self.app)],
        )
        handoff.reduce(ledger)
        handoff.close(ledger)
        self._zero(ledger)

    def test_foreign_collision_is_preserved_and_rename_can_retry(self):
        ledger, owner = self._prepare()
        handoff = self._terminal(owner, ledger)
        handoff.start_effect(ledger)
        self.project.mkdir()
        marker = self.project / "foreign"
        marker.write_text("keep")
        with self.assertRaises((self.CFError, FileExistsError, OSError)):
            handoff.rename_visible(ledger)
        self.assertEqual(owner._state, owner.EFFECT_STARTED)
        self.assertEqual(marker.read_text(), "keep")
        shutil.rmtree(self.project)
        handoff.rename_visible(ledger)
        handoff.reduce(ledger)
        handoff.close(ledger)
        self._zero(ledger)

    def test_reducer_failure_stays_retryable_without_rerename(self):
        attempts = 0

        def flaky_reducer(_capability):
            nonlocal attempts
            attempts += 1
            if attempts == 1:
                raise RuntimeError("reducer fault")

        ledger, owner = self._prepare(reducer=flaky_reducer)
        handoff = self._terminal(owner, ledger)
        handoff.start_effect(ledger)
        original = self.Owner.RENAME_NOREPLACE
        rename_calls = 0

        def counted_rename(*args):
            nonlocal rename_calls
            rename_calls += 1
            return original(*args)

        self.Owner.RENAME_NOREPLACE = staticmethod(counted_rename)
        try:
            handoff.rename_visible(ledger)
            with self.assertRaisesRegex(RuntimeError, "reducer fault"):
                handoff.reduce(ledger)
            self.assertEqual(owner._state, owner.REDUCING)
            with self.assertRaisesRegex(self.CFError, "incomplete"):
                handoff.close(ledger)
            handoff.reduce(ledger)
        finally:
            self.Owner.RENAME_NOREPLACE = staticmethod(original)
        self.assertEqual(rename_calls, 1)
        self.assertEqual(attempts, 2)
        handoff.close(ledger)
        self.assertTrue(self.project.exists())
        self._zero(ledger)

    def test_reducer_state_mutation_is_rejected_and_retryable(self):
        owner_slot = []
        original_transition = []
        attempts = 0

        def mutating_reducer(_capability):
            nonlocal attempts
            attempts += 1
            if attempts == 1:
                owner = owner_slot[0]
                original_transition.append(owner._transition)
                owner._transition = None

        ledger, owner = self._prepare(reducer=mutating_reducer)
        owner_slot.append(owner)
        handoff = self._terminal(owner, ledger)
        handoff.start_effect(ledger)
        handoff.rename_visible(ledger)
        with self.assertRaisesRegex(self.CFError, "dependency changed"):
            handoff.reduce(ledger)
        self.assertEqual(owner._state, owner.REDUCING)
        owner._transition = original_transition[0]
        handoff.reduce(ledger)
        self.assertEqual(attempts, 2)
        handoff.close(ledger)
        self.assertTrue(self.project.exists())
        self._zero(ledger)

    def test_reducer_capability_mutation_is_rejected_and_retryable(self):
        owner_slot = []
        attempts = 0

        def mutating_reducer(capability):
            nonlocal attempts
            attempts += 1
            if attempts == 1:
                capability._owner = None

        ledger, owner = self._prepare(reducer=mutating_reducer)
        owner_slot.append(owner)
        handoff = self._terminal(owner, ledger)
        handoff.start_effect(ledger)
        handoff.rename_visible(ledger)
        with self.assertRaisesRegex(self.CFError, "dependency changed"):
            handoff.reduce(ledger)
        self.assertEqual(owner._state, owner.REDUCING)
        owner._reducer_capability._owner = owner_slot[0]
        handoff.reduce(ledger)
        self.assertEqual(attempts, 2)
        handoff.close(ledger)
        self.assertTrue(self.project.exists())
        self._zero(ledger)

    def test_close_refuses_effect_gap_but_pre_effect_cleanup_is_retryable(self):
        ledger, owner = self._prepare()
        handoff = self._terminal(owner, ledger)
        handoff.start_effect(ledger)
        with self.assertRaisesRegex(self.CFError, "incomplete"):
            owner.close(ledger)
        handoff.rename_visible(ledger)
        handoff.reduce(ledger)
        owner.close(ledger)
        self._zero(ledger)

        self.app = "other"
        self.project = self.projects / self.org / self.app
        other_ledger, other = self._prepare()
        original = self.Owner.REMOVE_HIDDEN
        fired = False

        def fail_once(path):
            nonlocal fired
            if not fired:
                fired = True
                raise RuntimeError("cleanup fault")
            original(path)

        self.Owner.REMOVE_HIDDEN = staticmethod(fail_once)
        try:
            with self.assertRaisesRegex(RuntimeError, "cleanup fault"):
                other.close(other_ledger)
            self.assertFalse(other._closed)
            self.assertTrue(other._stage_path.exists())
        finally:
            self.Owner.REMOVE_HIDDEN = staticmethod(original)
        other.close(other_ledger)
        self.assertFalse(other._stage_path.exists())
        self._zero(other_ledger)

    def test_hidden_stage_identity_swap_is_preserved_until_reconciled(self):
        ledger, owner = self._prepare()
        stage = owner._stage_path
        moved = stage.with_name(stage.name + ".owned")
        stage.rename(moved)
        stage.mkdir()
        marker = stage / "foreign"
        marker.write_text("keep")
        with self.assertRaisesRegex(self.CFError, "identity changed"):
            owner.close(ledger)
        self.assertEqual(marker.read_text(), "keep")
        self.assertTrue(moved.exists())
        shutil.rmtree(stage)
        moved.rename(stage)
        owner.close(ledger)
        self.assertFalse(stage.exists())
        self._zero(ledger)

    def test_foreign_numeric_fd_reuse_is_rejected_and_not_closed(self):
        ledger, owner = self._prepare()
        descriptor_number = owner._descriptor_fd
        os.close(descriptor_number)
        foreign_path = self.tmp / "foreign-fd"
        foreign_path.write_bytes(b"foreign")
        foreign = os.open(foreign_path, os.O_RDONLY)
        if foreign != descriptor_number:
            os.dup2(foreign, descriptor_number)
            os.close(foreign)
            foreign = descriptor_number
        ledger.seal_prepublication()
        with self.assertRaisesRegex(self.CFError, "ownership changed"):
            owner.prepare_terminal(ledger)
        owner.close(ledger)
        self.assertEqual(os.pread(foreign, 7, 0), b"foreign")
        os.close(foreign)
        self._zero(ledger)

    def test_terminal_rejects_dependency_mutation_without_dispatch(self):
        deadline = BootstrapDeadline()
        ledger, owner = self._prepare(deadline=deadline)
        original = self.Owner._all_fds_owned
        replacement_called = False

        def replacement(_owner):
            nonlocal replacement_called
            replacement_called = True
            raise AssertionError("mutated class probe dispatched")

        def mutate_class(_phase):
            self.Owner._all_fds_owned = replacement

        deadline.callback = mutate_class
        ledger.seal_prepublication()
        try:
            with self.assertRaisesRegex(self.CFError, "dependency changed"):
                owner.prepare_terminal(ledger)
        finally:
            self.Owner._all_fds_owned = original
        self.assertFalse(replacement_called)
        self.assertEqual(owner._state, owner.PREPARED_HIDDEN)
        owner.close(ledger)
        self._zero(ledger)

    def test_terminal_rejects_reducer_mutation_without_dispatch(self):
        deadline = BootstrapDeadline()
        ledger, owner = self._prepare(deadline=deadline)
        replacement_called = False

        def replacement(_capability):
            nonlocal replacement_called
            replacement_called = True

        original = owner._durable_reducer

        def mutate_instance(_phase):
            owner._durable_reducer = replacement

        deadline.callback = mutate_instance
        ledger.seal_prepublication()
        try:
            with self.assertRaisesRegex(self.CFError, "dependency changed"):
                owner.prepare_terminal(ledger)
        finally:
            owner._durable_reducer = original
        self.assertFalse(replacement_called)
        self.assertEqual(owner._state, owner.PREPARED_HIDDEN)
        owner.close(ledger)
        self._zero(ledger)

    def test_terminal_rejects_later_capability_mutation_without_dispatch(self):
        deadline = BootstrapDeadline()
        ledger, owner = self._prepare(deadline=deadline)
        original = self.Owner.RENAME_NOREPLACE
        replacement_called = False

        def replacement(*_args):
            nonlocal replacement_called
            replacement_called = True

        def mutate_class(_phase):
            self.Owner.RENAME_NOREPLACE = staticmethod(replacement)

        deadline.callback = mutate_class
        ledger.seal_prepublication()
        try:
            with self.assertRaisesRegex(self.CFError, "dependency changed"):
                owner.prepare_terminal(ledger)
        finally:
            self.Owner.RENAME_NOREPLACE = staticmethod(original)
        self.assertFalse(replacement_called)
        self.assertEqual(owner._state, owner.PREPARED_HIDDEN)
        owner.close(ledger)
        self._zero(ledger)

    def test_terminal_rejects_ledger_current_mutation(self):
        deadline = BootstrapDeadline()
        ledger, owner = self._prepare(deadline=deadline)
        expected_retained = ledger._current["retained_bytes"]

        def mutate_ledger(_phase):
            ledger._current["retained_bytes"] -= 1

        deadline.callback = mutate_ledger
        ledger.seal_prepublication()
        try:
            with self.assertRaisesRegex(self.CFError, "dependency changed"):
                owner.prepare_terminal(ledger)
        finally:
            ledger._current["retained_bytes"] = expected_retained
        self.assertEqual(owner._state, owner.PREPARED_HIDDEN)
        owner.close(ledger)
        self._zero(ledger)

    def test_last_callback_mutation_rejects_before_effect(self):
        deadline = BootstrapDeadline()
        ledger, owner = self._prepare(deadline=deadline)
        descriptor = owner._stage_path / "remote_project.json"

        def mutate(_phase):
            descriptor.write_bytes(b"{}\n")

        deadline.callback = mutate
        ledger.seal_prepublication()
        with self.assertRaisesRegex(self.CFError, "descriptor"):
            owner.prepare_terminal(ledger)
        self.assertEqual(owner._state, owner.PREPARED_HIDDEN)
        self.assertFalse(self.project.exists())
        owner.close(ledger)
        self._zero(ledger)


if __name__ == "__main__":
    unittest.main()
