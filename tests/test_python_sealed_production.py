import json
import runpy
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class PythonUploadSealedProductionTests(unittest.TestCase):
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
        self.local_objects = self.tmp / ".codefire/objects"
        self.head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(
                encoding="utf-8"
            )
        )["head"]
        self.records = self.reachable_records(self.local_objects, self.head)
        self.storage_root = self.tmp / "http-storage"
        self.globals = self.namespace["handle_http_upload"].__globals__

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def ledger(self, overrides=None, label="sealed production"):
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
        record = json.loads(path.read_text(encoding="utf-8"))
        return self.namespace["object_record_core"](record)

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

    def child_commit(self, parent, *, wrong_root=False):
        parent_record = self.object_record(self.local_objects, parent)
        payload = json.loads(json.dumps(parent_record["payload"]))
        payload["parents"] = [parent]
        payload["message"] = "sealed production child"
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

    def test_stage_moves_exact_mixed_bounds_once_and_production_uses_them(self):
        status, _ = self.upload("mixed", self.head, self.records)
        self.assertEqual(status, 200)
        project_objects = self.project_root("mixed") / "objects"
        child = self.child_commit(self.head)
        records = [child]

        ledger = self.ledger(label="stage bounds")
        closure = self.namespace["plan_http_upload_closure"](
            project_objects, child["object_id"], records, ledger, None
        )
        incoming, existing = closure
        self.assertEqual(incoming, [0])
        self.assertGreater(len(existing), 0)
        stage = self.namespace["AccountedHTTPUploadStage"](
            ledger, project_objects, records, incoming, existing, None
        )
        with self.assertRaisesRegex(
            self.namespace["CFError"], "not sealed for validation"
        ):
            stage.take_sealed_validation_bounds()
        sizes = [
            entry[3]
            for frozen in (stage._incoming_frozen, stage._existing_frozen)
            for entry in frozen
        ]
        expected = (len(sizes), sum(sizes), max(sizes))
        stage.seal_pre_effect()
        self.assertEqual(stage.take_sealed_validation_bounds(), expected)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "bounds are unavailable"
        ):
            stage.take_sealed_validation_bounds()
        stage.close()
        closure.close()
        self.assert_closed(ledger)

        observed = []
        prepare = self.globals["prepare_http_upload_sealed_validation"]

        def observe_prepare(active_ledger, objects_root, head, **kwargs):
            paths = list(objects_root.glob("*/*.json"))
            actual_sizes = [path.stat().st_size for path in paths]
            actual = (
                len(actual_sizes),
                sum(actual_sizes),
                max(actual_sizes, default=0),
            )
            supplied = (
                kwargs["record_count_upper"],
                kwargs["aggregate_bytes_upper"],
                kwargs["maximum_record_bytes_upper"],
            )
            observed.append((active_ledger, kwargs["deadline"], supplied, actual))
            return prepare(active_ledger, objects_root, head, **kwargs)

        self.globals["prepare_http_upload_sealed_validation"] = observe_prepare
        production_ledger = self.ledger(label="mixed production")
        try:
            status, _ = self.upload(
                "mixed", child["object_id"], records, ledger=production_ledger
            )
        finally:
            self.globals["prepare_http_upload_sealed_validation"] = prepare
        self.assertEqual(status, 200)
        self.assertEqual(len(observed), 1)
        self.assertIs(observed[0][0], production_ledger)
        self.assertIsNone(observed[0][1])
        self.assertEqual(observed[0][2], observed[0][3])
        self.assertEqual(observed[0][2], expected)
        self.assert_closed(production_ledger)

    def test_production_exact_minus_one_precedes_session_allocation(self):
        count = len(self.records)
        sizes = [
            len(self.namespace["_serialize_object_record_core"](record))
            for record in self.records
        ]
        aggregate = sum(sizes)
        maximum = max(sizes)
        retained, _temporary = self.namespace[
            "AccountedHTTPUploadSealedValidation"
        ].allocation_upper(count, aggregate, maximum)
        allocations = []
        Session = self.globals["SealedCommitValidationSession"]

        class ObservedSession(Session):
            def __init__(self, *args, **kwargs):
                allocations.append(True)
                super().__init__(*args, **kwargs)

        self.globals["SealedCommitValidationSession"] = ObservedSession
        ledger = self.ledger(
            {"retained_bytes": ("concurrent", retained - 1)},
            "production sealed exact minus one",
        )
        try:
            with self.assertRaises(self.namespace["ResourceBudgetExceeded"]):
                self.upload("exact-minus-one", self.head, self.records, ledger=ledger)
        finally:
            self.globals["SealedCommitValidationSession"] = Session
        self.assertEqual(allocations, [])
        self.assertFalse(self.project_root("exact-minus-one").exists())
        self.assert_closed(ledger)

    def test_invalid_root_keeps_error_order_and_missing_project_absent(self):
        bad = self.child_commit(self.head, wrong_root=True)
        ledger = self.ledger(label="production invalid root")
        with self.assertRaisesRegex(
            self.namespace["CFError"],
            "root content_manifest has type verification, expected content_manifest",
        ):
            self.upload(
                "invalid-root",
                bad["object_id"],
                [bad, *self.records],
                ledger=ledger,
            )
        project_root = self.project_root("invalid-root")
        self.assertFalse(project_root.exists())
        self.assertFalse((project_root / "request_nonce_cache.json").exists())
        self.assert_closed(ledger)

    def test_deadline_during_sealed_validation_has_no_layout_or_nonce(self):
        class SealedDeadline:
            def check(inner_self, phase):
                if phase == "upload sealed validation":
                    raise self.namespace["RequestDeadlineExceeded"](phase, 30_000)

        ledger = self.ledger(label="production sealed deadline")
        with self.assertRaises(self.namespace["RequestDeadlineExceeded"]):
            self.upload(
                "sealed-deadline",
                self.head,
                self.records,
                ledger=ledger,
                deadline=SealedDeadline(),
            )
        project_root = self.project_root("sealed-deadline")
        self.assertFalse(project_root.exists())
        self.assertFalse((project_root / "request_nonce_cache.json").exists())
        self.assert_closed(ledger)

    def test_final_revalidation_rejects_staged_swap_before_layout(self):
        prepare = self.globals["prepare_http_upload_sealed_validation"]

        def swap_before_finish(active_ledger, objects_root, head, **kwargs):
            owner = prepare(active_ledger, objects_root, head, **kwargs)
            finish = owner.finish

            def changed_finish(selected_ledger):
                path = self.namespace["object_record_path"](objects_root, head)
                self.assertIsNotNone(path)
                path.write_bytes(path.read_bytes() + b" ")
                return finish(selected_ledger)

            owner.finish = changed_finish
            return owner

        self.globals["prepare_http_upload_sealed_validation"] = swap_before_finish
        ledger = self.ledger(label="production sealed final fence")
        try:
            with self.assertRaises(self.namespace["SealedValidationSubjectChanged"]):
                self.upload("sealed-swap", self.head, self.records, ledger=ledger)
        finally:
            self.globals["prepare_http_upload_sealed_validation"] = prepare
        self.assertFalse(self.project_root("sealed-swap").exists())
        self.assert_closed(ledger)

    def test_constructor_and_release_faults_cleanup_before_layout(self):
        Session = self.globals["SealedCommitValidationSession"]

        class FailingSession(Session):
            def __init__(self, *args, **kwargs):
                super().__init__(*args, **kwargs)
                raise RuntimeError("injected production sealed constructor")

        self.globals["SealedCommitValidationSession"] = FailingSession
        constructor_ledger = self.ledger(label="production constructor fault")
        try:
            with self.assertRaisesRegex(RuntimeError, "sealed constructor"):
                self.upload(
                    "sealed-constructor",
                    self.head,
                    self.records,
                    ledger=constructor_ledger,
                )
        finally:
            self.globals["SealedCommitValidationSession"] = Session
        self.assertFalse(self.project_root("sealed-constructor").exists())
        self.assert_closed(constructor_ledger)

        prepare = self.globals["prepare_http_upload_sealed_validation"]
        armed = False

        def arm_release(active_ledger, objects_root, head, **kwargs):
            nonlocal armed
            owner = prepare(active_ledger, objects_root, head, **kwargs)
            armed = True
            return owner

        release_ledger = self.ledger(label="production release fault")
        release = release_ledger.release
        failed = False

        def fail_retained_once(dimension, amount):
            nonlocal failed
            if armed and dimension == "retained_bytes" and not failed:
                failed = True
                raise RuntimeError("injected production sealed release")
            return release(dimension, amount)

        self.globals["prepare_http_upload_sealed_validation"] = arm_release
        release_ledger.release = fail_retained_once
        try:
            with self.assertRaisesRegex(RuntimeError, "sealed release"):
                self.upload(
                    "sealed-release",
                    self.head,
                    self.records,
                    ledger=release_ledger,
                )
        finally:
            self.globals["prepare_http_upload_sealed_validation"] = prepare
            release_ledger.release = release
        self.assertTrue(failed)
        self.assertFalse(self.project_root("sealed-release").exists())
        self.assert_closed(release_ledger)

    def test_no_ledger_path_keeps_legacy_validation(self):
        legacy = self.globals["validate_sealed_commit"]
        prepare = self.globals["prepare_http_upload_sealed_validation"]
        calls = []

        def observe_legacy(objects_root, head, seen=None):
            calls.append((objects_root, head, seen))
            return legacy(objects_root, head, seen)

        def reject_accounted(*args, **kwargs):
            raise AssertionError("accounted owner reached without request ledger")

        self.globals["validate_sealed_commit"] = observe_legacy
        self.globals["prepare_http_upload_sealed_validation"] = reject_accounted
        try:
            status, _ = self.upload("legacy-direct", self.head, self.records)
        finally:
            self.globals["validate_sealed_commit"] = legacy
            self.globals["prepare_http_upload_sealed_validation"] = prepare
        self.assertEqual(status, 200)
        self.assertGreaterEqual(len(calls), 1)
        self.assertIn("codefire-http-upload-", str(calls[0][0]))


if __name__ == "__main__":
    unittest.main()

