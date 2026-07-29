import json
import os
import runpy
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class GateDeadline:
    def __init__(self):
        self.callback = None
        self.match = None
        self.calls = []

    def check(self, phase):
        self.calls.append(phase)
        callback = self.callback
        if callback is not None and (
            self.match is None or self.match in phase
        ):
            self.callback = None
            callback(phase)


class ExistingProjectGateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ns = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.ns["DecisionWorkLedger"]
        cls.Profile = cls.ns["DecisionWorkProfile"]
        cls.Gate = cls.ns["AccountedHTTPUploadExistingProjectGate"]
        cls.PrepareGate = staticmethod(
            cls.ns["prepare_http_upload_existing_project_gate"]
        )
        cls.Carrier = cls.ns[
            "AccountedHTTPUploadExistingPairSemanticCarrier"
        ]
        cls.Authority = cls.ns["AccountedRemoteSignaturePolicyAuthority"]
        cls.CFError = cls.ns["CFError"]
        cls.ResourceBudget = cls.ns["ResourceBudgetExceeded"]

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp())
        self.org = "org"
        self.app = "app"
        self.stable_id = "CF-PROJECT-0123456789abcdef0123456789abcdef"
        self.project = self._make_project(self.tmp / self.org / self.app)

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def _descriptor(self, **updates):
        value = {
            "type": "codefire_remote_project",
            "version": 1,
            "stable_project_id": self.stable_id,
            "protocol_generation": 1,
            "organization": self.org,
            "application": self.app,
            "creation_provenance": "test",
            "created_at": "2026-07-22T00:00:00Z",
        }
        value.update(updates)
        return self.ns["canonical_json"](value) + b"\n"

    def _make_project(self, path, descriptor=None):
        path.mkdir(parents=True)
        (path / "locks").mkdir()
        (path / "locks/project-gate.lock").write_bytes(b"")
        (path / "remote_project.json").write_bytes(
            self._descriptor() if descriptor is None else descriptor
        )
        return path

    def _ledger(self, overrides=None):
        return self.Ledger(
            self.Profile.http_v1(overrides),
            "existing project gate",
            command_kind="http_request",
        )

    def _zero(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def test_exact_gate_is_private_and_close_releases_every_scalar(self):
        ledger = self._ledger()
        gate = self.PrepareGate(
            self.project, self.org, self.app, ledger
        )
        retained, temporary = self.Gate.live_upper(self.project)
        self.assertEqual(ledger.current["retained_bytes"], retained)
        self.assertEqual(ledger.current["temporary_bytes"], temporary)
        self.assertEqual(gate._stable_id, self.stable_id)
        for public_name in ("stable_id", "project_id", "payload", "gate"):
            self.assertFalse(hasattr(gate, public_name))
        gate.close(ledger)
        gate.close(ledger)
        self._zero(ledger)

    def test_construction_exact_limits_and_minus_one(self):
        retained, temporary = self.Gate.construction_upper(self.project)
        exact = self._ledger(
            {"retained_bytes": retained, "temporary_bytes": temporary}
        )
        gate = self.PrepareGate(self.project, self.org, self.app, exact)
        gate.close(exact)
        self._zero(exact)

        for dimension, limit in (
            ("retained_bytes", retained - 1),
            ("temporary_bytes", temporary - 1),
        ):
            with self.subTest(dimension=dimension):
                overrides = {
                    "retained_bytes": retained,
                    "temporary_bytes": temporary,
                }
                overrides[dimension] = limit
                ledger = self._ledger(overrides)
                with self.assertRaises(self.ResourceBudget):
                    self.PrepareGate(
                        self.project, self.org, self.app, ledger
                    )
                self._zero(ledger)

        physical = len((self.project / "remote_project.json").read_bytes())
        for limit, succeeds in ((2 * physical, True), (2 * physical - 1, False)):
            with self.subTest(aggregate_bytes=limit):
                ledger = self._ledger({"aggregate_bytes": limit})
                if succeeds:
                    gate = self.PrepareGate(
                        self.project, self.org, self.app, ledger
                    )
                    gate.close(ledger)
                else:
                    with self.assertRaises(self.ResourceBudget):
                        self.PrepareGate(
                            self.project, self.org, self.app, ledger
                        )
                self._zero(ledger)

    def test_gate_must_precede_live_owner_and_can_create_lock_inode(self):
        (self.project / "locks/project-gate.lock").unlink()
        ledger = self._ledger()
        gate = self.PrepareGate(
            self.project, self.org, self.app, ledger
        )
        self.assertTrue((self.project / "locks/project-gate.lock").is_file())
        gate.close(ledger)
        self._zero(ledger)

        ledger = self._ledger()
        ledger.acquire("retained_bytes", 1, "query")
        with self.assertRaisesRegex(self.CFError, "precede live owners"):
            self.PrepareGate(
                self.project, self.org, self.app, ledger
            )
        ledger.release("retained_bytes", 1)
        self._zero(ledger)

    def test_descriptor_16k_boundary_and_strict_identity(self):
        base = self._descriptor()
        exact = base + b" " * (self.Gate.MAX_DESCRIPTOR_BYTES - len(base))
        (self.project / "remote_project.json").write_bytes(exact)
        ledger = self._ledger()
        gate = self.PrepareGate(self.project, self.org, self.app, ledger)
        gate.close(ledger)
        self._zero(ledger)

        (self.project / "remote_project.json").write_bytes(exact + b" ")
        ledger = self._ledger()
        with self.assertRaisesRegex(self.CFError, "16 KiB"):
            self.PrepareGate(self.project, self.org, self.app, ledger)
        self._zero(ledger)

        duplicate = self._descriptor().rstrip()[:-1] + (
            b',"stable_project_id":"' + self.stable_id.encode() + b'"}\n'
        )
        (self.project / "remote_project.json").write_bytes(duplicate)
        ledger = self._ledger()
        with self.assertRaisesRegex(self.CFError, "duplicate"):
            self.PrepareGate(self.project, self.org, self.app, ledger)
        self._zero(ledger)

        (self.project / "remote_project.json").write_bytes(
            self._descriptor(stable_project_id="CF-PROJECT-" + "A" * 32)
        )
        ledger = self._ledger()
        with self.assertRaisesRegex(self.CFError, "descriptor is invalid"):
            self.PrepareGate(self.project, self.org, self.app, ledger)
        self._zero(ledger)

    def test_org_app_and_each_named_capability_are_no_follow(self):
        ledger = self._ledger()
        with self.assertRaisesRegex(self.CFError, "path does not match"):
            self.PrepareGate(self.project, "wrong", self.app, ledger)
        self._zero(ledger)

        for target in ("locks", "gate", "descriptor"):
            with self.subTest(target=target):
                root = self.tmp / f"case-{target}"
                project = root / self.org / self.app
                self._make_project(project)
                if target == "locks":
                    shutil.rmtree(project / "locks")
                    (project / "real-locks").mkdir()
                    (project / "locks").symlink_to("real-locks")
                elif target == "gate":
                    (project / "locks/project-gate.lock").unlink()
                    (project / "locks/real-gate").write_bytes(b"")
                    (project / "locks/project-gate.lock").symlink_to(
                        "real-gate"
                    )
                else:
                    (project / "remote_project.json").unlink()
                    (project / "real-descriptor").write_bytes(
                        self._descriptor()
                    )
                    (project / "remote_project.json").symlink_to(
                        "real-descriptor"
                    )
                ledger = self._ledger()
                with self.assertRaises((self.CFError, OSError)):
                    self.PrepareGate(project, self.org, self.app, ledger)
                self._zero(ledger)

        real = self.tmp / "real-app"
        self._make_project(real)
        linked = self.tmp / "linked" / self.org / self.app
        linked.parent.mkdir(parents=True)
        linked.symlink_to(real, target_is_directory=True)
        ledger = self._ledger()
        with self.assertRaises((self.CFError, OSError)):
            self.PrepareGate(linked, self.org, self.app, ledger)
        self._zero(ledger)

    def test_hardlinked_gate_and_descriptor_are_rejected(self):
        for target in ("gate", "descriptor"):
            with self.subTest(target=target):
                project = self._make_project(
                    self.tmp / f"hard-{target}" / self.org / self.app
                )
                selected = (
                    project / "locks/project-gate.lock"
                    if target == "gate"
                    else project / "remote_project.json"
                )
                os.link(selected, selected.with_name(selected.name + ".alias"))
                ledger = self._ledger()
                with self.assertRaisesRegex(self.CFError, "single-link"):
                    self.PrepareGate(project, self.org, self.app, ledger)
                self._zero(ledger)

    def test_exclusive_contention_rejects_and_retry_succeeds(self):
        first_ledger = self._ledger()
        first = self.PrepareGate(
            self.project, self.org, self.app, first_ledger
        )
        second_ledger = self._ledger()
        with self.assertRaisesRegex(self.CFError, "already held"):
            self.PrepareGate(
                self.project, self.org, self.app, second_ledger
            )
        self._zero(second_ledger)
        first.close(first_ledger)
        self._zero(first_ledger)
        second = self.PrepareGate(
            self.project, self.org, self.app, second_ledger
        )
        second.close(second_ledger)
        self._zero(second_ledger)

    def test_descriptor_and_project_replacement_break_probe(self):
        for target in ("descriptor", "project"):
            with self.subTest(target=target):
                root = self.tmp / f"mutate-{target}"
                project = self._make_project(root / self.org / self.app)
                ledger = self._ledger()
                gate = self.PrepareGate(project, self.org, self.app, ledger)
                if target == "descriptor":
                    replacement = project / "replacement.json"
                    replacement.write_bytes(self._descriptor())
                    os.replace(replacement, project / "remote_project.json")
                else:
                    moved = project.with_name("old-app")
                    project.rename(moved)
                    self._make_project(project)
                with self.assertRaisesRegex(
                    self.ns["ExistingProjectGateChanged"], "changed"
                ):
                    gate._probe_subject()
                gate.close(ledger)
                self._zero(ledger)

    def test_post_commit_accounting_faults_reconcile_and_retry(self):
        ledger = self._ledger()
        original_consume = ledger.consume_reserved
        fired = False

        def consume_then_raise(*args, **kwargs):
            nonlocal fired
            original_consume(*args, **kwargs)
            if not fired:
                fired = True
                raise RuntimeError("post-consume")

        ledger.consume_reserved = consume_then_raise
        gate = self.PrepareGate(self.project, self.org, self.app, ledger)
        ledger.consume_reserved = original_consume
        self.assertTrue(fired)

        original_release = ledger.release
        released = False

        def release_then_raise(*args, **kwargs):
            nonlocal released
            original_release(*args, **kwargs)
            if not released:
                released = True
                raise RuntimeError("post-release")

        ledger.release = release_then_raise
        gate.close(ledger)
        ledger.release = original_release
        gate.close(ledger)
        self.assertTrue(released)
        self._zero(ledger)

    def test_open_and_post_read_faults_leave_no_owner_debt(self):
        original_flock = self.Gate.FLOCK

        def fail_lock(_fd, _operation):
            raise OSError("flock fault")

        self.Gate.FLOCK = staticmethod(fail_lock)
        ledger = self._ledger()
        try:
            with self.assertRaisesRegex(OSError, "flock fault"):
                self.PrepareGate(
                    self.project, self.org, self.app, ledger
                )
        finally:
            self.Gate.FLOCK = staticmethod(original_flock)
        self._zero(ledger)

        original_read = self.Gate._read_subject

        def read_then_raise(selected):
            original_read(selected)
            raise RuntimeError("post-read fault")

        self.Gate._read_subject = read_then_raise
        ledger = self._ledger()
        try:
            with self.assertRaisesRegex(RuntimeError, "post-read fault"):
                self.PrepareGate(
                    self.project, self.org, self.app, ledger
                )
        finally:
            self.Gate._read_subject = original_read
        self._zero(ledger)

    def test_unlock_fault_preserves_retryable_gate_owner(self):
        ledger = self._ledger()
        gate = self.PrepareGate(
            self.project, self.org, self.app, ledger
        )
        original_flock = self.Gate.FLOCK
        fired = False

        def fail_first_unlock(fd, operation):
            nonlocal fired
            if operation == self.ns["fcntl"].LOCK_UN and not fired:
                fired = True
                raise RuntimeError("unlock fault")
            return original_flock(fd, operation)

        self.Gate.FLOCK = staticmethod(fail_first_unlock)
        try:
            with self.assertRaisesRegex(RuntimeError, "unlock fault"):
                gate.close(ledger)
        finally:
            self.Gate.FLOCK = staticmethod(original_flock)
        self.assertFalse(gate._closed)
        gate.close(ledger)
        self.assertTrue(fired)
        self._zero(ledger)

    def _prepared_carrier(self):
        subprocess.run(
            [str(CODEFIRE), "init"],
            cwd=self.tmp,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        objects = self.tmp / ".codefire/objects"
        initial = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text()
        )["head"]
        payload = self.ns["load_object_from_root"](objects, initial)
        child = json.loads(json.dumps(payload))
        child["message"] = "existing project gate"
        child["parents"] = [initial]
        key = "gate-secret"
        child = self.ns["sign_commit_payload"](child, "alice", key)
        head = self.ns["store_object"](self.tmp, "commit", child)
        (self.project / "server_policy.json").write_bytes(
            self.ns["canonical_json"](
                {
                    "commit_signatures": {
                        "required": True,
                        "keys": {"alice": key},
                    },
                    "verification": {"required": []},
                }
            )
        )
        ledger = self._ledger()
        deadline = GateDeadline()
        gate = self.PrepareGate(
            self.project, self.org, self.app, ledger, deadline
        )
        records = [
            json.loads(path.read_text())
            for path in sorted(objects.glob("*/*.json"))
        ]
        request = {"branch": "main", "head": head, "objects": records}
        exact = len(self.ns["canonical_json"](request))
        scalar = self.ns["prepare_http_upload_parser_scalar"](ledger, exact)
        scalar.acquire(ledger)
        scalar.reconcile_exact(ledger, exact)
        source = self.ns["bind_http_upload_request_source"](
            request, ledger, scalar
        )
        semantic = gate.prepare_semantic(
            source, ledger, "main", deadline
        )
        semantic.prepare(ledger)
        paths = list(objects.glob("*/*.json"))
        sizes = [path.stat().st_size for path in paths]
        sealed = self.ns["prepare_http_upload_sealed_validation"](
            ledger,
            objects,
            head,
            record_count_upper=len(sizes),
            aggregate_bytes_upper=sum(sizes),
            maximum_record_bytes_upper=max(sizes),
            deadline=deadline,
            _semantic_baseline_owner=semantic,
        )
        sealed.validate(ledger)
        policy = self.Authority(self.project, ledger, deadline)
        pair = self.ns["AccountedHTTPUploadExistingTerminalOwner"](
            sealed, policy, ledger
        )
        pair.verify_signature_and_server("main", head, ledger)
        self.assertTrue(pair.contains_ancestor(initial, head, ledger))
        return ledger, deadline, gate, pair, semantic

    def test_typed_gate_handoff_survives_terminal_and_closes_last(self):
        ledger, deadline, gate, pair, semantic = self._prepared_carrier()
        carrier = gate.prepare_carrier(pair, semantic, ledger)
        self.assertIsNone(gate._stable_id)
        self.assertEqual(carrier._stable_project_id, self.stable_id)
        with self.assertRaisesRegex(self.CFError, "adopted"):
            gate.close(ledger)
        carrier.finalize(ledger)
        self.assertEqual(carrier._state, carrier.READY)
        self.assertFalse(gate._closed)
        self.assertEqual(
            deadline.calls.count("upload terminal pair probe"), 1
        )
        carrier.close(ledger)
        self.assertTrue(gate._closed)
        self._zero(ledger)

    def test_fake_gate_proof_is_rejected_without_consuming_owner(self):
        ledger, _deadline, gate, pair, semantic = self._prepared_carrier()
        with self.assertRaisesRegex(self.CFError, "exact gate"):
            self.Carrier(pair, semantic, ledger)
        with self.assertRaisesRegex(self.CFError, "exact gate"):
            self.Carrier(
                pair,
                semantic,
                ledger,
                _stable_gate_owner=gate,
                _stable_gate_token=object(),
            )
        self.assertEqual(gate._state, gate.READY)
        carrier = gate.prepare_carrier(pair, semantic, ledger)
        carrier.close(ledger)
        self._zero(ledger)

    def test_last_callback_descriptor_mutation_rejects_before_commit(self):
        ledger, deadline, gate, pair, semantic = self._prepared_carrier()
        carrier = gate.prepare_carrier(pair, semantic, ledger)

        def replace_descriptor(_phase):
            changed = self._descriptor(
                stable_project_id=(
                    "CF-PROJECT-1123456789abcdef0123456789abcdef"
                )
            )
            (self.project / "remote_project.json").write_bytes(changed)

        deadline.callback = replace_descriptor
        deadline.match = "upload terminal pair probe"
        with self.assertRaises(self.ns["ExistingProjectGateChanged"]):
            carrier.finalize(ledger)
        self.assertFalse(carrier._input_committed)
        if not carrier._closed:
            carrier.close(ledger)
        self.assertTrue(gate._closed)
        self._zero(ledger)

    def _assert_gate_instance_type_drift_is_hook_free(self, hook_kind):
        ledger, deadline, gate, pair, semantic = self._prepared_carrier()
        carrier = gate.prepare_carrier(pair, semantic, ledger)
        hook_calls = []
        if hook_kind == "get":
            class HookGate(self.Gate):
                def __getattribute__(selected, name):
                    hook_calls.append(("get", name))
                    return object.__getattribute__(selected, name)
        else:
            class HookGate(self.Gate):
                def __setattr__(selected, name, value):
                    hook_calls.append(("set", name))
                    return object.__setattr__(selected, name, value)

        def replace_instance_type(_phase):
            object.__setattr__(gate, "__class__", HookGate)

        deadline.callback = replace_instance_type
        deadline.match = "upload terminal pair probe"
        try:
            with self.assertRaisesRegex(self.CFError, "proof changed"):
                carrier.finalize(ledger)
            self.assertEqual(hook_calls, [])
            self.assertFalse(object.__getattribute__(gate, "_closed"))
            self.assertFalse(object.__getattribute__(carrier, "_closed"))
            self.assertFalse(
                object.__getattribute__(carrier, "_input_committed")
            )
        finally:
            object.__setattr__(gate, "__class__", self.Gate)
        carrier.close(ledger)
        self._zero(ledger)

    def test_last_callback_gate_getattribute_type_drift_is_hook_free(self):
        self._assert_gate_instance_type_drift_is_hook_free("get")

    def test_last_callback_gate_setattr_type_drift_is_hook_free(self):
        self._assert_gate_instance_type_drift_is_hook_free("set")

    def test_last_callback_gate_dependency_drift_is_never_executed(self):
        ledger, deadline, gate, pair, semantic = self._prepared_carrier()
        carrier = gate.prepare_carrier(pair, semantic, ledger)
        class_mapping = type.__getattribute__(self.Gate, "__dict__")
        names = ("_read_exact", "_fingerprint", "CLOSE_CAPABILITY")
        originals = {
            name: type(class_mapping).__getitem__(class_mapping, name)
            for name in names
        }
        hook_calls = []

        def replacement(*_args, **_kwargs):
            hook_calls.append("called")
            raise AssertionError("mutated gate dependency executed")

        def replace_dependencies(_phase):
            type.__setattr__(self.Gate, "_read_exact", replacement)
            type.__setattr__(
                self.Gate, "_fingerprint", staticmethod(replacement)
            )
            type.__setattr__(
                self.Gate, "CLOSE_CAPABILITY", staticmethod(replacement)
            )

        deadline.callback = replace_dependencies
        deadline.match = "upload terminal pair probe"
        try:
            with self.assertRaisesRegex(self.CFError, "proof changed"):
                carrier.finalize(ledger)
            self.assertEqual(hook_calls, [])
            self.assertFalse(object.__getattribute__(gate, "_closed"))
            self.assertFalse(object.__getattribute__(carrier, "_closed"))
            self.assertFalse(
                object.__getattribute__(carrier, "_input_committed")
            )
        finally:
            for name, original in originals.items():
                type.__setattr__(self.Gate, name, original)
        carrier.close(ledger)
        self._zero(ledger)

    def test_last_callback_gate_global_drift_is_hook_free(self):
        ledger, deadline, gate, pair, semantic = self._prepared_carrier()
        carrier = gate.prepare_carrier(pair, semantic, ledger)
        gate_mapping = type.__getattribute__(self.Gate, "__dict__")
        raw_probe = type(gate_mapping).__getitem__(
            gate_mapping, "_probe_subject"
        )
        probe_globals = object.__getattribute__(raw_probe, "__globals__")
        names = ("_open_directory_nofollow", "_CODEFIRE_OS_PREAD")
        originals = {
            name: dict.__getitem__(probe_globals, name) for name in names
        }
        hook_calls = []

        def replacement(*_args, **_kwargs):
            hook_calls.append("called")
            raise AssertionError("mutated gate global executed")

        def replace_global(_phase):
            for name in names:
                dict.__setitem__(probe_globals, name, replacement)

        deadline.callback = replace_global
        deadline.match = "upload terminal pair probe"
        try:
            with self.assertRaisesRegex(self.CFError, "proof changed"):
                carrier.finalize(ledger)
            self.assertEqual(hook_calls, [])
            self.assertFalse(object.__getattribute__(gate, "_closed"))
            self.assertFalse(object.__getattribute__(carrier, "_closed"))
        finally:
            for name, original in originals.items():
                dict.__setitem__(probe_globals, name, original)
        carrier.close(ledger)
        self._zero(ledger)

    def test_last_callback_gate_os_primitive_drift_is_hook_free(self):
        ledger, deadline, gate, pair, semantic = self._prepared_carrier()
        carrier = gate.prepare_carrier(pair, semantic, ledger)
        gate_mapping = type.__getattribute__(self.Gate, "__dict__")
        raw_probe = type(gate_mapping).__getitem__(
            gate_mapping, "_probe_subject"
        )
        probe_globals = object.__getattribute__(raw_probe, "__globals__")
        os_module = dict.__getitem__(probe_globals, "os")
        os_mapping = object.__getattribute__(os_module, "__dict__")
        original_stat = dict.__getitem__(os_mapping, "stat")
        hook_calls = []

        def replacement(*args, **kwargs):
            hook_calls.append("called")
            return original_stat(*args, **kwargs)

        def replace_primitive(_phase):
            dict.__setitem__(os_mapping, "stat", replacement)

        deadline.callback = replace_primitive
        deadline.match = "upload terminal pair probe"
        try:
            with self.assertRaisesRegex(self.CFError, "proof changed"):
                carrier.finalize(ledger)
            self.assertEqual(hook_calls, [])
            self.assertFalse(object.__getattribute__(gate, "_closed"))
            self.assertFalse(object.__getattribute__(carrier, "_closed"))
        finally:
            dict.__setitem__(os_mapping, "stat", original_stat)
        carrier.close(ledger)
        self._zero(ledger)

    def test_last_callback_transferred_gate_lock_cannot_mint_ready(self):
        ledger, deadline, gate, pair, semantic = self._prepared_carrier()
        carrier = gate.prepare_carrier(pair, semantic, ledger)
        flock = self.ns["fcntl"].flock
        lock_ex = self.ns["fcntl"].LOCK_EX
        lock_nb = self.ns["fcntl"].LOCK_NB
        lock_un = self.ns["fcntl"].LOCK_UN
        holder = []

        def transfer_lock(_phase):
            flock(object.__getattribute__(gate, "_gate_fd"), lock_un)
            descriptor = os.open(
                self.project / "locks/project-gate.lock", os.O_RDWR
            )
            flock(descriptor, lock_ex | lock_nb)
            holder.append(descriptor)

        deadline.callback = transfer_lock
        deadline.match = "upload terminal pair probe"
        try:
            with self.assertRaisesRegex(
                self.ns["ExistingProjectGateChanged"], "exclusive_gate"
            ):
                carrier.finalize(ledger)
            self.assertFalse(carrier._input_committed)
            self.assertNotEqual(carrier._state, carrier.READY)
        finally:
            if holder:
                flock(holder[0], lock_un)
                os.close(holder[0])
        if not carrier._closed:
            carrier.close(ledger)
        self._zero(ledger)

    def _assert_reused_gate_descriptor_is_abandoned(self, attribute):
        ledger, deadline, gate, pair, semantic = self._prepared_carrier()
        carrier = gate.prepare_carrier(pair, semantic, ledger)
        foreign_path = self.tmp / f"foreign-{attribute}.bin"
        foreign_path.write_bytes(b"foreign descriptor")
        holder = []

        def reuse_numeric_descriptor(_phase):
            owned_descriptor = object.__getattribute__(gate, attribute)
            os.close(owned_descriptor)
            foreign_descriptor = os.open(foreign_path, os.O_RDWR)
            if foreign_descriptor != owned_descriptor:
                os.close(foreign_descriptor)
                raise AssertionError("numeric descriptor was not reused")
            holder.append(foreign_descriptor)

        deadline.callback = reuse_numeric_descriptor
        deadline.match = "upload terminal pair probe"
        with self.assertRaisesRegex(self.CFError, "ownership changed"):
            carrier.finalize(ledger)
        self.assertFalse(carrier._input_committed)
        self.assertFalse(carrier._closed)
        self.assertFalse(object.__getattribute__(gate, "_closed"))
        self.assertTrue(any(ledger.current.values()))
        foreign_descriptor = holder[0]
        foreign_metadata = os.fstat(foreign_descriptor)
        self.assertEqual(
            (foreign_metadata.st_dev, foreign_metadata.st_ino),
            (foreign_path.stat().st_dev, foreign_path.stat().st_ino),
        )
        observer = os.open(foreign_path, os.O_RDWR)
        try:
            self.ns["fcntl"].flock(
                observer,
                self.ns["fcntl"].LOCK_EX | self.ns["fcntl"].LOCK_NB,
            )
            self.ns["fcntl"].flock(observer, self.ns["fcntl"].LOCK_UN)
        finally:
            os.close(observer)
        self.assertEqual(os.pread(foreign_descriptor, 64, 0), b"foreign descriptor")
        os.close(foreign_descriptor)
        carrier.close(ledger)
        self._zero(ledger)

    def test_last_callback_reused_gate_fd_is_never_unlocked_or_closed(self):
        self._assert_reused_gate_descriptor_is_abandoned("_gate_fd")

    def test_last_callback_reused_descriptor_fd_is_never_closed(self):
        self._assert_reused_gate_descriptor_is_abandoned("_descriptor_fd")


if __name__ == "__main__":
    unittest.main()
