import runpy
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class PythonResponseInputTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ns = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.ns["DecisionWorkLedger"]
        cls.Profile = cls.ns["DecisionWorkProfile"]
        cls.Owner = cls.ns["AccountedHTTPUploadResponseInput"]
        cls.Prepare = staticmethod(cls.ns["prepare_http_upload_response_input"])
        cls.CFError = cls.ns["CFError"]

    def ledger(self, label="response input"):
        return self.Ledger(
            self.Profile.http_v1(), label, command_kind="http_request"
        )

    def assert_zero(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def prepared(self):
        ledger = self.ledger()
        owner = self.Prepare(ledger, body_upper=32, header_upper=64)
        owner.prepare(
            ledger,
            200,
            b"success-header",
            b"success-body",
            b"durable-header",
            b"durable-body",
        )
        return ledger, owner

    def test_preparse_upper_reconciles_to_exact_immutable_pair(self):
        ledger = self.ledger()
        owner = self.Prepare(ledger, body_upper=32, header_upper=64)
        expected_upper = 2 * 32 + 2 * 64 + owner.FIXED_RETAINED_BYTES
        self.assertEqual(ledger.current["retained_bytes"], expected_upper)
        owner.prepare(
            ledger,
            201,
            b"success-header",
            b"success-body",
            b"durable-header",
            b"durable-body",
        )
        expected_exact = (
            len(b"success-header")
            + len(b"success-body")
            + len(b"durable-header")
            + len(b"durable-body")
            + owner.FIXED_RETAINED_BYTES
        )
        self.assertEqual(owner._state, owner.PREPARED)
        self.assertEqual(owner._retained_exact, expected_exact)
        self.assertEqual(ledger.current["retained_bytes"], expected_exact)
        self.assertEqual(ledger.cumulative["response_bytes"], 12)
        self.assertEqual(ledger.cumulative["response_header_bytes"], 14)
        for name in ("payload", "success", "durable", "select"):
            self.assertFalse(hasattr(owner, name))
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_body_or_header_overflow_is_pre_effect_and_retryable_by_close(self):
        ledger = self.ledger()
        owner = self.Prepare(ledger, body_upper=3, header_upper=4)
        with self.assertRaisesRegex(self.CFError, "preparse upper"):
            owner.prepare(ledger, 200, b"12345", b"123", b"1", b"12")
        self.assertEqual(owner._state, owner.RESERVED)
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_both_response_payloads_accept_the_exact_combined_upper(self):
        ledger = self.ledger()
        owner = self.Prepare(ledger, body_upper=4, header_upper=3)
        owner.prepare(ledger, 200, b"hhh", b"bbbb", b"hhh", b"bbbb")
        expected = 2 * 4 + 2 * 3 + owner.FIXED_RETAINED_BYTES
        self.assertEqual(owner._retained_exact, expected)
        self.assertEqual(ledger.current["retained_bytes"], expected)
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_prepare_is_single_use(self):
        ledger, owner = self.prepared()
        with self.assertRaisesRegex(self.CFError, "single-use"):
            owner.prepare(ledger, 200, b"h", b"b", b"h", b"b")
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_foreign_ledger_is_rejected(self):
        ledger, owner = self.prepared()
        foreign = self.ledger("foreign response input")
        with self.assertRaisesRegex(self.CFError, "another ledger"):
            owner.close(foreign)
        owner.close(ledger)
        self.assert_zero(ledger)
        self.assert_zero(foreign)

    def test_direct_constructor_is_rejected(self):
        ledger = self.ledger()
        transition = self.ns["_StablePermitTransition"](ledger)
        with self.assertRaisesRegex(self.CFError, "use prepare"):
            self.Owner(ledger, 1, 1, transition)
        self.assert_zero(ledger)

    def test_non_exact_bytes_and_status_are_rejected(self):
        ledger = self.ledger()
        owner = self.Prepare(ledger, body_upper=8, header_upper=8)
        with self.assertRaisesRegex(self.CFError, "exact response bytes"):
            owner.prepare(ledger, True, b"h", b"b", b"h", b"b")
        with self.assertRaisesRegex(self.CFError, "exact response bytes"):
            owner.prepare(ledger, 200, bytearray(b"h"), b"b", b"h", b"b")
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_baseline_values_require_prepared_exact_owner(self):
        ledger = self.ledger()
        owner = self.Prepare(ledger, body_upper=8, header_upper=8)
        with self.assertRaisesRegex(self.CFError, "baseline is unavailable"):
            owner._baseline_values(ledger)
        owner.prepare(ledger, 200, b"h", b"body", b"h", b"retry")
        transition, retained, upper = owner._baseline_values(ledger)
        self.assertIs(transition, owner._transition)
        self.assertEqual(retained, owner._retained_exact)
        self.assertGreaterEqual(upper, retained)
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_close_release_fault_retries_same_owner(self):
        ledger, owner = self.prepared()
        original = ledger.release
        calls = 0

        def fail_once(dimension, amount):
            nonlocal calls
            if dimension == "retained_bytes":
                calls += 1
                if calls == 1:
                    raise RuntimeError("response input release fault")
            return original(dimension, amount)

        ledger.release = fail_once
        with self.assertRaisesRegex(RuntimeError, "release fault"):
            owner.close(ledger)
        self.assertFalse(owner._closed)
        owner.close(ledger)
        del ledger.release
        self.assertGreaterEqual(calls, 2)
        self.assert_zero(ledger)

    def test_adopted_owner_rejects_stale_close(self):
        ledger, owner = self.prepared()
        owner._semantic_owner = object()
        with self.assertRaisesRegex(self.CFError, "adopted response"):
            owner.close(ledger)
        owner._semantic_owner = None
        owner.close(ledger)
        self.assert_zero(ledger)


if __name__ == "__main__":
    unittest.main()
