import json
import runpy
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class PythonExternalProductionTests(unittest.TestCase):
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
        self.namespace = runpy.run_path(str(CODEFIRE))
        self.globals = self.namespace["handle_http_upload"].__globals__
        self.Owner = self.namespace["AccountedServerVerification"]
        self.local_objects = self.tmp / ".codefire/objects"
        self.head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(
                encoding="utf-8"
            )
        )["head"]
        self.records = self.reachable_records(self.local_objects, self.head)
        self.storage_root = self.tmp / "http-storage"

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def ledger(self, overrides=None, label="external production"):
        return self.namespace["DecisionWorkLedger"](
            self.namespace["DecisionWorkProfile"].http_v1(overrides),
            label,
            command_kind="http_request",
        )

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def project_root(self, app):
        return self.namespace["http_project_root"](
            self.storage_root, "org", app
        )

    def upload(self, app, head, records, *, ledger=None, deadline=None):
        return self.namespace["handle_http_upload"](
            self.storage_root,
            "org",
            app,
            "main",
            {"branch": "main", "head": head, "objects": records},
            decision_work=ledger,
            deadline=deadline,
        )

    def object_record(self, objects_root, object_id):
        path = self.namespace["object_record_path"](objects_root, object_id)
        self.assertIsNotNone(path, object_id)
        return self.namespace["object_record_core"](
            json.loads(path.read_text(encoding="utf-8"))
        )

    def reachable_records(self, objects_root, head):
        records = []
        seen = set()
        stack = [head]
        while stack:
            object_id = stack.pop()
            if object_id in seen:
                continue
            seen.add(object_id)
            record = self.object_record(objects_root, object_id)
            records.append(record)
            stack.extend(self.namespace["object_record_references"](record))
        return sorted(records, key=lambda record: record["object_id"])

    def bootstrap(self, app):
        status, _ = self.upload(app, self.head, self.records)
        self.assertEqual(status, 200)
        return self.project_root(app)

    def write_policy(self, project_root, checks):
        (project_root / "server_policy.json").write_text(
            json.dumps({"verification": {"required": checks}}),
            encoding="utf-8",
        )

    def authority_snapshot(self, project_root):
        paths = (
            "remote_project.json",
            "generation.json",
            "branches/main.json",
            "request_nonce_cache.json",
            "pending_transaction.json",
        )
        return {
            path: (
                (project_root / path).read_bytes()
                if (project_root / path).is_file()
                else None
            )
            for path in paths
        }

    def child_commit(self, parent, *, wrong_root=False):
        parent_record = self.object_record(self.local_objects, parent)
        payload = json.loads(json.dumps(parent_record["payload"]))
        payload["parents"] = [parent]
        payload["message"] = "external production child"
        if wrong_root:
            payload["roots"]["content_manifest"] = payload["roots"][
                "verification"
            ]
        object_id = self.namespace["object_id"]("commit", payload)
        return {
            "object_id": object_id,
            "type": "commit",
            "hash": self.namespace["object_digest"]("commit", payload),
            "payload": payload,
        }

    def test_same_ledger_exact_minus_one_rejects_before_spawn(self):
        project_root = self.bootstrap("exact-minus-one")
        self.write_policy(
            project_root, [{"id": "must-not-spawn", "command": "exit 0"}]
        )
        authority_before = self.authority_snapshot(project_root)
        output_temporary = (
            self.Owner.OUTPUT_MEMORY_BASE_BYTES
            + self.Owner.OUTPUT_MEMORY_MULTIPLIER
            * (self.Owner.DEFAULT_OUTPUT_LIMIT_BYTES + 1)
            + self.Owner.OUTPUT_READ_CHUNK_BYTES
        )
        ledger = self.ledger(
            {"temporary_bytes": ("concurrent", output_temporary - 1)},
            "external production exact minus one",
        )
        prepare = self.globals["prepare_server_verification"]
        spawn = self.Owner._spawn_process
        observed = []
        spawn_calls = 0

        def observe_prepare(*args, **kwargs):
            observed.append((args[3], args[4]))
            return prepare(*args, **kwargs)

        def observe_spawn(*args):
            nonlocal spawn_calls
            spawn_calls += 1
            return spawn(*args)

        self.globals["prepare_server_verification"] = observe_prepare
        self.Owner._spawn_process = staticmethod(observe_spawn)
        try:
            with self.assertRaises(self.namespace["ResourceBudgetExceeded"]):
                self.upload(
                    "exact-minus-one", self.head, self.records, ledger=ledger
                )
        finally:
            self.globals["prepare_server_verification"] = prepare
            self.Owner._spawn_process = staticmethod(spawn)
        self.assertEqual(observed, [(ledger, None)])
        self.assertEqual(spawn_calls, 0)
        self.assertEqual(self.authority_snapshot(project_root), authority_before)
        self.assert_closed(ledger)

    def test_deadline_has_no_layout_or_nonce(self):
        class VerificationDeadline:
            def check(inner_self, phase):
                if phase == "upload server verification execution":
                    raise self.namespace["RequestDeadlineExceeded"](phase, 30_000)

        ledger = self.ledger(label="external production deadline")
        with self.assertRaises(self.namespace["RequestDeadlineExceeded"]):
            self.upload(
                "deadline",
                self.head,
                self.records,
                ledger=ledger,
                deadline=VerificationDeadline(),
            )
        project_root = self.project_root("deadline")
        self.assertFalse(project_root.exists())
        self.assertFalse((project_root / "request_nonce_cache.json").exists())
        self.assert_closed(ledger)

    def test_process_read_fault_reaps_before_return(self):
        project_root = self.bootstrap("process-fault")
        self.write_policy(
            project_root,
            [{"id": "read-fault", "command": "printf x; sleep 30"}],
        )
        authority_before = self.authority_snapshot(project_root)
        ledger = self.ledger(label="external production process fault")
        spawn = self.Owner._spawn_process
        read_pipe = self.Owner._read_pipe
        spawned = []

        def observe_spawn(*args):
            process = spawn(*args)
            spawned.append(process)
            return process

        def fail_read(*_args):
            raise OSError("injected production verification read fault")

        self.Owner._spawn_process = staticmethod(observe_spawn)
        self.Owner._read_pipe = staticmethod(fail_read)
        try:
            with self.assertRaisesRegex(OSError, "verification read fault"):
                self.upload(
                    "process-fault", self.head, self.records, ledger=ledger
                )
        finally:
            self.Owner._spawn_process = staticmethod(spawn)
            self.Owner._read_pipe = staticmethod(read_pipe)
        self.assertEqual(len(spawned), 1)
        self.assertIsNotNone(spawned[0].poll())
        self.assertEqual(self.authority_snapshot(project_root), authority_before)
        self.assert_closed(ledger)

    def test_release_retry_preserves_run_error_and_accounting(self):
        project_root = self.bootstrap("release-retry")
        self.write_policy(
            project_root, [{"id": "run-error", "command": "exit 7"}]
        )
        authority_before = self.authority_snapshot(project_root)
        ledger = self.ledger(label="external production release retry")
        prepare = self.globals["prepare_server_verification"]
        release = ledger.release
        owners = []
        armed = False
        failed = False
        cleanup_fault = RuntimeError(
            "injected production verification release"
        )
        run_faults = []
        failure = self.Owner._failure

        def observe_failure(owner, message):
            error = failure(owner, message)
            run_faults.append(error)
            return error

        def observe_prepare(*args, **kwargs):
            nonlocal armed
            owner = prepare(*args, **kwargs)
            owners.append(owner)
            armed = True
            return owner

        def fail_retained_once(dimension, amount):
            nonlocal failed
            if armed and dimension == "retained_bytes" and not failed:
                failed = True
                raise cleanup_fault
            return release(dimension, amount)

        self.globals["prepare_server_verification"] = observe_prepare
        self.Owner._failure = observe_failure
        ledger.release = fail_retained_once
        try:
            with self.assertRaises(self.namespace["CFError"]) as caught:
                self.upload(
                    "release-retry", self.head, self.records, ledger=ledger
                )
        finally:
            self.globals["prepare_server_verification"] = prepare
            self.Owner._failure = failure
            ledger.release = release
        self.assertTrue(failed)
        self.assertEqual(len(owners), 1)
        self.assertIs(caught.exception, run_faults[0])
        self.assertIs(caught.exception.__cause__, cleanup_fault)
        self.assertIn("server verification failed: run-error", str(caught.exception))
        self.assertFalse(owners[0]._closed)
        owners[0].close()
        self.assertEqual(self.authority_snapshot(project_root), authority_before)
        self.assert_closed(ledger)

    def test_no_ledger_keeps_local_wrapper_and_policy_effect(self):
        project_root = self.bootstrap("legacy")
        marker = project_root / "legacy-verification-marker"
        self.write_policy(
            project_root,
            [{"id": "legacy", "command": "touch legacy-verification-marker"}],
        )
        prepare = self.globals["prepare_server_verification"]
        observed = []

        def observe_prepare(*args, **kwargs):
            observed.append((args[3], args[4]))
            return prepare(*args, **kwargs)

        self.globals["prepare_server_verification"] = observe_prepare
        try:
            status, _ = self.upload("legacy", self.head, self.records)
        finally:
            self.globals["prepare_server_verification"] = prepare
        self.assertEqual(status, 200)
        self.assertTrue(marker.is_file())
        self.assertEqual(len(observed), 1)
        local_ledger, observed_deadline = observed[0]
        self.assertIsNone(observed_deadline)
        self.assertEqual(local_ledger._command, "server verification")
        self.assertEqual(
            local_ledger._profile.limit("retained_bytes"), 1024 * 1024 * 1024
        )
        self.assert_closed(local_ledger)

    def test_invalid_root_precedes_verification_effect(self):
        project_root = self.bootstrap("precedence")
        marker = project_root / "must-not-exist"
        self.write_policy(
            project_root,
            [{"id": "must-not-run", "command": "touch must-not-exist"}],
        )
        bad = self.child_commit(self.head, wrong_root=True)
        ledger = self.ledger(label="external production precedence")
        authority_before = self.authority_snapshot(project_root)
        with self.assertRaisesRegex(
            self.namespace["CFError"],
            "root content_manifest has type verification, expected content_manifest",
        ):
            self.upload("precedence", bad["object_id"], [bad], ledger=ledger)
        self.assertFalse(marker.exists())
        self.assertEqual(self.authority_snapshot(project_root), authority_before)
        self.assert_closed(ledger)


if __name__ == "__main__":
    unittest.main()
