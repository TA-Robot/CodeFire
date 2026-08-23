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


class PythonSignatureTerminalFreezeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.namespace = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.namespace["DecisionWorkLedger"]
        cls.Profile = cls.namespace["DecisionWorkProfile"]
        cls.PolicyAuthority = cls.namespace[
            "AccountedRemoteSignaturePolicyAuthority"
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
        self.head = json.loads(
            (self.tmp / ".codefire/branches/main.json").read_text(
                encoding="utf-8"
            )
        )["head"]
        self.project = self.tmp / "remote-project"
        self.project.mkdir()
        self.policy_path = self.project / "server_policy.json"
        self.policy_path.write_bytes(b"{}")

    def tearDown(self):
        shutil.rmtree(self.tmp)

    def ledger(self, label):
        return self.Ledger(
            self.Profile.http_v1(), label, command_kind="http_request"
        )

    def prepare_sealed(self, ledger):
        paths = list(self.objects.glob("*/*.json"))
        sizes = [path.stat().st_size for path in paths]
        return self.namespace["prepare_http_upload_sealed_validation"](
            ledger,
            self.objects,
            self.head,
            record_count_upper=len(sizes),
            aggregate_bytes_upper=sum(sizes),
            maximum_record_bytes_upper=max(sizes),
            deadline=None,
        )

    def prepare_disabled_policy(self, ledger):
        authority = self.PolicyAuthority(self.project, ledger)

        def forbidden_loader(_object_id):
            raise AssertionError("disabled signature policy called its loader")

        policy = authority.prepare_policy(forbidden_loader, ledger)
        policy.verify(self.head)
        return authority

    def borrow_signature_once(self, owner, ledger):
        handle = owner.borrow_commits(ledger, "signature")
        payload, upper = handle(self.head)
        self.assertEqual(payload.get("type"), "commit")
        self.assertGreater(upper, 0)
        payload = None
        handle.close(ledger)

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def test_policy_first_leaves_sealed_callback_policy_window(self):
        ledger = self.ledger("policy-first terminal freeze")
        sealed = self.prepare_sealed(ledger)
        sealed.validate(ledger)
        self.borrow_signature_once(sealed, ledger)
        policy = self.prepare_disabled_policy(ledger)
        policy.revalidate(ledger)
        original = self.policy_path.read_bytes()
        mutations = []

        class MutatePolicy:
            def check(inner_self, _phase):
                if mutations:
                    return
                self.policy_path.write_bytes(
                    b'{"commit_signatures":{"required":true}}'
                )
                mutations.append(True)

        sealed._deadline = MutatePolicy()
        with self.assertRaisesRegex(
            self.namespace["CFError"], "indeterminate scalars"
        ):
            sealed.finish(ledger)
        self.assertEqual(mutations, [True])
        self.assertNotEqual(self.policy_path.read_bytes(), original)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "single-use"
        ):
            policy.revalidate(ledger)
        policy.close(ledger)
        sealed.close(ledger)
        self.assert_closed(ledger)

    def test_sealed_first_leaves_policy_callback_staged_window(self):
        ledger = self.ledger("sealed-first terminal freeze")
        sealed = self.prepare_sealed(ledger)
        sealed.validate(ledger)
        self.borrow_signature_once(sealed, ledger)
        policy = self.prepare_disabled_policy(ledger)
        with self.assertRaisesRegex(
            self.namespace["CFError"], "indeterminate scalars"
        ):
            sealed.finish(ledger)
        object_path = self.namespace["object_record_path"](
            self.objects, self.head
        )
        original = object_path.read_bytes()
        original_inode = object_path.stat().st_ino
        mutations = []

        class MutateStagedObject:
            def check(inner_self, _phase):
                if mutations:
                    return
                replacement = object_path.with_name(
                    object_path.name + ".replacement"
                )
                replacement.write_bytes(original)
                os.replace(replacement, object_path)
                mutations.append(True)

        policy._deadline = MutateStagedObject()
        policy.revalidate(ledger)
        self.assertEqual(mutations, [True])
        self.assertNotEqual(object_path.stat().st_ino, original_inode)
        self.assertIsNone(sealed._session)
        policy.close(ledger)
        sealed.close(ledger)
        self.assert_closed(ledger)

    def test_signature_only_finish_precludes_future_ancestry_borrow(self):
        ledger = self.ledger("signature-only finish freeze")
        sealed = self.prepare_sealed(ledger)
        sealed.validate(ledger)
        self.borrow_signature_once(sealed, ledger)
        sealed.finish(ledger)
        with self.assertRaisesRegex(self.namespace["CFError"], "closed"):
            sealed.borrow_commits(ledger, "ancestry")
        self.assert_closed(ledger)

    def test_missing_project_bootstrap_invalidates_policy_authority(self):
        missing = self.tmp / "missing/org/app"
        missing.parent.mkdir(parents=True)
        ledger = self.ledger("missing project terminal freeze")
        authority = self.PolicyAuthority(missing, ledger)

        def forbidden_loader(_object_id):
            raise AssertionError("missing policy called its loader")

        policy = authority.prepare_policy(forbidden_loader, ledger)
        policy.verify(self.head)
        missing.mkdir()
        with self.assertRaises(
            self.namespace["SignaturePolicySubjectChanged"]
        ):
            authority.revalidate(ledger)
        authority.close(ledger)
        self.assert_closed(ledger)


if __name__ == "__main__":
    unittest.main()
