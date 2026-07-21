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


class ArmedDeadline:
    def __init__(self):
        self.limit_ms = None
        self.started_at = 0.0
        self.last_observed = 0.0
        self.callback = None

    def check(self, phase):
        callback = self.callback
        if callback is not None:
            self.callback = None
            callback(phase)


class PythonExistingConsumerTerminalFreezeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.namespace = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.namespace["DecisionWorkLedger"]
        cls.Profile = cls.namespace["DecisionWorkProfile"]
        cls.Authority = cls.namespace[
            "AccountedRemoteSignaturePolicyAuthority"
        ]
        cls.SignatureComposite = cls.namespace[
            "AccountedHTTPUploadSignatureVerificationComposite"
        ]
        cls.Ancestry = cls.namespace["AccountedHTTPUploadAncestry"]

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
        self.initial_head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(
                encoding="utf-8"
            )
        )["head"]
        initial = self.namespace["load_object_from_root"](
            self.objects, self.initial_head
        )
        child = json.loads(json.dumps(initial))
        child["message"] = "existing consumer terminal freeze"
        child["parents"] = [self.initial_head]
        self.key = "existing-consumer-secret"
        signed = self.namespace["sign_commit_payload"](
            child, "alice", self.key
        )
        self.head = self.namespace["store_object"](
            self.tmp, "commit", signed
        )

        self.project = self.tmp / "remote/org/app"
        self.project.mkdir(parents=True)
        (self.project / "locks").mkdir()
        (self.project / "remote_project.json").write_text(
            "{}\n", encoding="utf-8"
        )
        self.policy_path = self.project / "server_policy.json"

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def ledger(self, label):
        return self.Ledger(
            self.Profile.http_v1(), label, command_kind="http_request"
        )

    def write_policy(self, command="exit 0"):
        policy = {
            "commit_signatures": {
                "required": True,
                "keys": {"alice": self.key},
            },
            "verification": {
                "required": [{"id": "consumer", "command": command}]
            },
        }
        self.policy_path.write_bytes(
            json.dumps(policy, separators=(",", ":")).encode("utf-8")
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

    def prepare_signature(self, ledger, deadline):
        sealed = self.prepare_sealed(ledger, deadline)
        authority = self.Authority(self.project, ledger, deadline)
        composite = self.SignatureComposite(sealed, authority, ledger)
        verification = composite.prepare("main", self.head, ledger)
        return sealed, authority, composite, verification

    def run_ancestry_with_unified_test_token(
        self, ledger, deadline, sealed, composite
    ):
        with self.namespace["remote_project_gate"](self.project):
            borrow = sealed.borrow_commits(
                ledger,
                "ancestry",
                _signature_composite_owner=composite,
            )
            ancestry = self.Ancestry(ledger, borrow, deadline)
            try:
                self.assertTrue(ancestry.contains(self.initial_head, self.head))
            finally:
                ancestry.close(ledger)
                borrow.close(ledger)
        self.assertEqual(borrow._token, 0)
        self.assertEqual(sealed._active_borrow_token, 0)

    @staticmethod
    def detach_test_token(composite, sealed, authority):
        # Characterization-only stand-in for the missing successful transfer
        # API. Production code must not clear coordinator tokens this way.
        object.__setattr__(sealed, "_signature_composite_owner", None)
        object.__setattr__(authority, "_signature_composite_owner", None)
        object.__setattr__(composite, "_sealed_owner", None)
        object.__setattr__(composite, "_policy_authority", None)
        object.__setattr__(composite, "_closed", True)

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def test_current_signature_owner_cannot_transfer_to_ancestry_handoff(self):
        self.write_policy()
        ledger = self.ledger("existing consumer ancestry transfer freeze")
        deadline = ArmedDeadline()
        sealed, authority, composite, verification = self.prepare_signature(
            ledger, deadline
        )
        verification.run()
        verification.close()
        handoff = self.namespace["prepare_http_upload_sealed_ancestry"](
            sealed, ledger, deadline
        )
        with self.assertRaisesRegex(
            self.namespace["CFError"], "composite owns sealed authority"
        ):
            handoff.prepare(ledger)
        self.assertFalse(sealed._borrow_handles["ancestry"]._used)
        handoff.close(ledger)
        composite.close(ledger)
        self.assert_closed(ledger)

    def test_verification_failure_has_external_effect_but_no_abort_transition(self):
        self.write_policy("touch verification-failed.marker; exit 1")
        ledger = self.ledger("existing consumer verification failure freeze")
        deadline = ArmedDeadline()
        sealed, _authority, composite, verification = self.prepare_signature(
            ledger, deadline
        )
        with self.assertRaisesRegex(
            self.namespace["CFError"], "server verification failed"
        ):
            verification.run()
        verification.close()
        self.assertTrue((self.project / "verification-failed.marker").exists())
        self.assertFalse(composite._abort_only)
        self.assertFalse(sealed._borrow_handles["ancestry"]._used)
        composite.close(ledger)
        self.assert_closed(ledger)

    def test_policy_first_final_after_consumers_leaves_policy_callback_window(self):
        self.write_policy()
        ledger = self.ledger("existing consumer policy-first freeze")
        deadline = ArmedDeadline()
        sealed, authority, composite, verification = self.prepare_signature(
            ledger, deadline
        )
        verification.run()
        verification.close()
        self.run_ancestry_with_unified_test_token(
            ledger, deadline, sealed, composite
        )
        self.detach_test_token(composite, sealed, authority)

        original = self.policy_path.read_bytes()
        mutations = []

        def mutate_policy(_phase):
            self.policy_path.write_bytes(
                b'{"commit_signatures":{"required":false}}'
            )
            mutations.append(True)

        with self.namespace["remote_project_gate"](self.project):
            authority.revalidate(ledger)
            deadline.callback = mutate_policy
            with self.assertRaisesRegex(
                self.namespace["CFError"], "indeterminate scalars"
            ):
                sealed.finish(ledger)
        self.assertEqual(mutations, [True])
        self.assertNotEqual(self.policy_path.read_bytes(), original)
        with self.assertRaisesRegex(self.namespace["CFError"], "single-use"):
            authority.revalidate(ledger)
        authority.close(ledger)
        sealed.close(ledger)
        self.assert_closed(ledger)

    def test_sealed_first_final_after_consumers_leaves_staged_callback_window(self):
        self.write_policy()
        ledger = self.ledger("existing consumer sealed-first freeze")
        deadline = ArmedDeadline()
        sealed, authority, composite, verification = self.prepare_signature(
            ledger, deadline
        )
        verification.run()
        verification.close()
        self.run_ancestry_with_unified_test_token(
            ledger, deadline, sealed, composite
        )
        self.detach_test_token(composite, sealed, authority)

        object_path = self.namespace["object_record_path"](
            self.objects, self.head
        )
        original = object_path.read_bytes()
        original_inode = object_path.stat().st_ino
        mutations = []

        def mutate_staged(_phase):
            replacement = object_path.with_name(object_path.name + ".replacement")
            replacement.write_bytes(original)
            os.replace(replacement, object_path)
            mutations.append(True)

        with self.namespace["remote_project_gate"](self.project):
            with self.assertRaisesRegex(
                self.namespace["CFError"], "indeterminate scalars"
            ):
                sealed.finish(ledger)
            deadline.callback = mutate_staged
            authority.revalidate(ledger)
        self.assertEqual(mutations, [True])
        self.assertNotEqual(object_path.stat().st_ino, original_inode)
        self.assertIsNone(sealed._session)
        authority.close(ledger)
        sealed.close(ledger)
        self.assert_closed(ledger)

    def test_publication_helper_accepts_ambient_records_after_sealed_final(self):
        ledger = self.ledger("existing consumer ambient publication freeze")
        deadline = ArmedDeadline()
        sealed = self.prepare_sealed(ledger, deadline)
        sealed.finish(ledger)
        self.assert_closed(ledger)

        payload = {"version": 1, "content": "created after sealed final"}
        digest = self.namespace["object_digest"]("blob", payload)
        object_id = self.namespace["object_id"]("blob", payload)
        ambient_records = [
            {
                "object_id": object_id,
                "type": "blob",
                "hash": digest,
                "payload": payload,
            }
        ]
        relative = Path(self.namespace["object_subdir"]("blob")) / f"{object_id}.json"
        self.assertFalse((self.objects / relative).exists())

        published_objects = self.tmp / "ambient-publication"
        self.namespace["write_object_records"](
            published_objects,
            ambient_records,
        )
        self.assertTrue((published_objects / relative).is_file())

    def test_abort_cleanup_after_consumers_stops_at_policy_then_retries(self):
        self.write_policy()
        ledger = self.ledger("existing consumer cleanup freeze")
        deadline = ArmedDeadline()
        sealed, authority, composite, verification = self.prepare_signature(
            ledger, deadline
        )
        verification.run()
        verification.close()
        self.run_ancestry_with_unified_test_token(
            ledger, deadline, sealed, composite
        )
        original_authority_close = authority.close
        original_sealed_close = sealed.close
        calls = []

        def fail_policy_once(selected=None, **kwargs):
            calls.append("policy")
            if calls == ["policy"]:
                raise RuntimeError("synthetic policy cleanup failure")
            return original_authority_close(selected, **kwargs)

        def observe_sealed(selected, **kwargs):
            calls.append("sealed")
            return original_sealed_close(selected, **kwargs)

        authority.close = fail_policy_once
        sealed.close = observe_sealed
        try:
            with self.assertRaisesRegex(RuntimeError, "policy cleanup failure"):
                composite.close(ledger)
            self.assertEqual(calls, ["policy"])
            self.assertFalse(sealed._closed)
            composite.close(ledger)
        finally:
            authority.close = original_authority_close
            sealed.close = original_sealed_close
        self.assertEqual(calls, ["policy", "policy", "sealed"])
        self.assert_closed(ledger)


if __name__ == "__main__":
    unittest.main()
