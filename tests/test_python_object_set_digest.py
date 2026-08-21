import json
import gc
import runpy
import unittest
import weakref
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class PythonObjectSetDigestTests(unittest.TestCase):
    def setUp(self):
        self.namespace = runpy.run_path(str(CODEFIRE))
        self.Ledger = self.namespace["DecisionWorkLedger"]
        self.Profile = self.namespace["DecisionWorkProfile"]
        self.prepare = self.namespace["prepare_remote_object_set_digest"]
        self.legacy = self.namespace["merge_request_object_set_digest"]
        self.Owner = self.namespace["AccountedRemoteObjectSetDigest"]

    def ledger(self, overrides=None):
        return self.Ledger(
            self.Profile.http_v1(overrides),
            "HTTP object-set digest",
            command_kind="http_request",
        )

    def assert_closed(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    def source_upper(self, records):
        return len(
            self.namespace["canonical_json"](
                self.namespace["canonical_remote_object_records"](records)
            )
        )

    @staticmethod
    def records(count):
        return [
            {
                "object_id": f"CF-BLOB-{index:064x}",
                "type": "blob",
                "hash": f"{index:064x}",
                "payload": {
                    "unicode": f"値-{index}",
                    "nested": [index, True, None, {"z": 2, "a": "x"}],
                },
            }
            for index in range(count - 1, -1, -1)
        ]

    def test_digest_matches_legacy_without_reordering_input(self):
        for count in (0, 1, 3, 10):
            with self.subTest(count=count):
                records = self.records(count)
                before = json.dumps(records, ensure_ascii=False)
                ledger = self.ledger()
                source_upper = self.source_upper(records)
                owner = self.prepare(records, ledger, source_upper)
                try:
                    self.assertTrue(owner.matches_digest(self.legacy(records)))
                    self.assertFalse(owner.matches_digest("sha256:" + "0" * 64))
                    self.assertEqual(
                        json.dumps(records, ensure_ascii=False), before
                    )
                    self.assertEqual(
                        ledger.cumulative.get("aggregate_bytes", 0),
                        source_upper,
                    )
                    self.assertEqual(
                        ledger.counters.get("object_records", 0), count
                    )
                finally:
                    owner.close()
                self.assert_closed(ledger)

    def test_invalid_and_duplicate_order_matches_legacy(self):
        cases = [
            ([{"type": "blob"}], "without an object ID"),
            (
                [
                    {"object_id": "CF-BLOB-a"},
                    {"object_id": "CF-BLOB-a"},
                ],
                "duplicate object ID",
            ),
        ]
        for records, message in cases:
            with self.subTest(message=message):
                ledger = self.ledger()
                with self.assertRaisesRegex(
                    self.namespace["CFError"], message
                ):
                    self.prepare(records, ledger, 4096)
                with self.assertRaisesRegex(
                    self.namespace["CFError"], message
                ):
                    self.legacy(records)
                self.assert_closed(ledger)

    def test_budget_precedes_order_and_hash_allocations(self):
        records = self.records(3)
        total_bytes = self.source_upper(records)
        retained = (
            self.Owner.ORDERED_LIST_BASE_BYTES
            + self.Owner.ORDERED_LIST_SLOT_BYTES * len(records)
            + self.Owner.DIGEST_RETAINED_BYTES
        )
        temporary = (
            self.Owner.ENCODER_BASE_BYTES
            + self.Owner.ENCODER_BYTES_PER_SOURCE_BYTE * total_bytes
        )
        globals_ = self.prepare.__globals__
        hash_canonical = globals_["_hash_canonical_json_polling"]
        hash_calls = 0

        def observe_hash(*args, **kwargs):
            nonlocal hash_calls
            hash_calls += 1
            return hash_canonical(*args, **kwargs)

        globals_["_hash_canonical_json_polling"] = observe_hash
        try:
            temporary_rejected = self.ledger(
                {
                    "temporary_bytes": (
                        "concurrent",
                        temporary - 1,
                    )
                }
            )
            with self.assertRaises(
                self.namespace["ResourceBudgetExceeded"]
            ):
                self.prepare(records, temporary_rejected, total_bytes)
            self.assertEqual(hash_calls, 0)
            self.assert_closed(temporary_rejected)

            aggregate_rejected = self.ledger(
                {"aggregate_bytes": ("cumulative", total_bytes - 1)}
            )
            with self.assertRaises(
                self.namespace["ResourceBudgetExceeded"]
            ):
                self.prepare(records, aggregate_rejected, total_bytes)
            self.assertEqual(hash_calls, 0)
            self.assert_closed(aggregate_rejected)

            rejected = self.ledger(
                {"retained_bytes": ("concurrent", retained - 1)}
            )
            with self.assertRaises(
                self.namespace["ResourceBudgetExceeded"]
            ):
                self.prepare(records, rejected, total_bytes)
            self.assertEqual(hash_calls, 0)
            self.assert_closed(rejected)

            exact = self.ledger(
                {"retained_bytes": ("concurrent", retained)}
            )
            owner = self.prepare(records, exact, total_bytes)
            self.assertEqual(hash_calls, len(records))
            owner.close()
            self.assert_closed(exact)
        finally:
            globals_["_hash_canonical_json_polling"] = hash_canonical

    def test_deadline_and_cleanup_failure_keep_retry_state(self):
        records = self.records(10)
        now = [0.0]

        def clock():
            now[0] += 0.001
            return now[0]

        deadline = self.namespace["HTTPRequestDeadline"](3, clock)
        deadline_ledger = self.ledger()
        with self.assertRaises(
            self.namespace["RequestDeadlineExceeded"]
        ):
            self.prepare(
                records, deadline_ledger, self.source_upper(records), deadline
            )
        self.assert_closed(deadline_ledger)

        ledger = self.ledger()
        owner = self.prepare(records, ledger, self.source_upper(records))
        release = ledger.release
        failed = False

        def fail_retained_once(dimension, amount):
            nonlocal failed
            if dimension == "retained_bytes" and not failed:
                failed = True
                raise RuntimeError("injected digest release fault")
            return release(dimension, amount)

        ledger.release = fail_retained_once
        with self.assertRaisesRegex(RuntimeError, "release fault"):
            owner.close()
        self.assertFalse(owner._closed)
        self.assertIsNone(owner._digest)
        self.assertIsNone(owner._ordered)
        self.assertGreater(ledger.current.get("retained_bytes", 0), 0)
        ledger.release = release
        owner.close()
        self.assert_closed(ledger)

    def test_validation_precedes_payload_encoding_and_polls_deadline(self):
        cases = [
            ([{"payload": object()}], "without an object ID"),
            (
                [
                    {"object_id": "CF-BLOB-a", "payload": None},
                    {"object_id": "CF-BLOB-a", "payload": object()},
                ],
                "duplicate object ID",
            ),
        ]
        for records, message in cases:
            with self.subTest(message=message):
                ledger = self.ledger()
                with self.assertRaisesRegex(
                    self.namespace["CFError"], message
                ):
                    self.prepare(records, ledger, 4096)
                self.assert_closed(ledger)

        records = self.records(100)
        owner_heap_sort = self.Owner._heap_sort

        def skip_sort(owner, deadline):
            del owner, deadline

        self.Owner._heap_sort = skip_sort
        now = [0.0]

        def clock():
            now[0] += 0.001
            return now[0]

        try:
            ledger = self.ledger()
            with self.assertRaises(
                self.namespace["RequestDeadlineExceeded"]
            ):
                self.prepare(
                    list(reversed(records)),
                    ledger,
                    self.source_upper(records),
                    self.namespace["HTTPRequestDeadline"](3, clock),
                )
            self.assert_closed(ledger)
        finally:
            self.Owner._heap_sort = owner_heap_sort

    def test_deep_source_upper_precedes_hash_and_exact_limit_accepts(self):
        nested = 0
        for _ in range(12_000):
            nested = [nested]
        records = [{"object_id": "CF-BLOB-deep", "payload": nested}]
        source_upper = 50_000
        temporary = (
            self.Owner.ENCODER_BASE_BYTES
            + self.Owner.ENCODER_BYTES_PER_SOURCE_BYTE * source_upper
        )
        globals_ = self.prepare.__globals__
        original_hash = globals_["_hash_canonical_json_polling"]
        calls = 0

        def observe_hash(*args, **kwargs):
            nonlocal calls
            calls += 1
            return original_hash(*args, **kwargs)

        globals_["_hash_canonical_json_polling"] = observe_hash
        try:
            rejected = self.ledger(
                {"temporary_bytes": ("concurrent", temporary - 1)}
            )
            with self.assertRaises(
                self.namespace["ResourceBudgetExceeded"]
            ):
                self.prepare(records, rejected, source_upper)
            self.assertEqual(calls, 0)
            self.assert_closed(rejected)

            exact = self.ledger(
                {"temporary_bytes": ("concurrent", temporary)}
            )
            owner = self.prepare(records, exact, source_upper)
            self.assertEqual(calls, 1)
            owner.close()
            self.assert_closed(exact)
        finally:
            globals_["_hash_canonical_json_polling"] = original_hash

    def test_stream_accounting_includes_non_string_mapping_keys(self):
        records = [
            {
                "object_id": "CF-BLOB-infinity",
                "payload": {float("inf"): 1},
            }
        ]
        source_upper = self.source_upper(records)
        ledger = self.ledger()
        owner = self.prepare(records, ledger, source_upper)
        self.assertTrue(owner.matches_digest(self.legacy(records)))
        self.assertEqual(
            ledger.cumulative.get("aggregate_bytes", 0), source_upper
        )
        owner.close()
        self.assert_closed(ledger)

    def test_source_upper_contract_rejects_invalid_values(self):
        for value in (-1, True, 1.5, None):
            with self.subTest(value=value):
                ledger = self.ledger()
                with self.assertRaisesRegex(
                    self.namespace["CFError"], "source upper"
                ):
                    self.prepare([], ledger, value)
                self.assert_closed(ledger)

    def test_false_source_upper_stops_before_encoder_key_allocation(self):
        records = [
            {
                "object_id": "CF-BLOB-wide",
                "payload": {f"key-{index}": index for index in range(50_000)},
            }
        ]
        globals_ = self.prepare.__globals__
        original_hash = globals_["_hash_canonical_json_polling"]
        calls = 0

        def observe_hash(*args, **kwargs):
            nonlocal calls
            calls += 1
            return original_hash(*args, **kwargs)

        globals_["_hash_canonical_json_polling"] = observe_hash
        try:
            ledger = self.ledger()
            with self.assertRaisesRegex(
                self.namespace["CFError"], "exceed the source upper"
            ):
                self.prepare(records, ledger, 256)
            self.assertEqual(calls, 0)
            self.assert_closed(ledger)
        finally:
            globals_["_hash_canonical_json_polling"] = original_hash

    def test_hash_traceback_payload_dies_before_scalar_release(self):
        class Probe:
            pass

        records = self.records(1)
        globals_ = self.prepare.__globals__
        original_hash = globals_["_hash_canonical_json_polling"]
        reference = None

        def fail_hash(*args, **kwargs):
            nonlocal reference
            del args, kwargs
            try:
                probe = Probe()
                reference = weakref.ref(probe)
                raise ValueError("injected inner digest failure")
            except ValueError:
                raise RuntimeError("injected digest hash failure")

        globals_["_hash_canonical_json_polling"] = fail_hash
        ledger = self.ledger()
        original_release = ledger.release
        release_observations = []

        def observe_release(dimension, amount):
            release_observations.append((dimension, reference()))
            return original_release(dimension, amount)

        ledger.release = observe_release
        try:
            with self.assertRaisesRegex(RuntimeError, "hash failure"):
                self.prepare(records, ledger, self.source_upper(records))
        finally:
            globals_["_hash_canonical_json_polling"] = original_hash
            ledger.release = original_release
        self.assertTrue(release_observations)
        self.assertTrue(
            all(probe is None for _dimension, probe in release_observations)
        )
        self.assert_closed(ledger)

    def test_preconsume_cancel_fault_keeps_permit_retry_owner(self):
        records = self.records(1)
        ledger = self.ledger()
        original_consume = ledger.consume_reserved
        original_cancel = ledger.cancel_permit
        consume_failed = False
        cancel_failed = False

        def fail_consume_once(*args, **kwargs):
            nonlocal consume_failed
            if not consume_failed:
                consume_failed = True
                raise RuntimeError("injected digest consume failure")
            return original_consume(*args, **kwargs)

        def fail_cancel_once(permit):
            nonlocal cancel_failed
            if not cancel_failed:
                cancel_failed = True
                raise RuntimeError("injected digest cancel failure")
            return original_cancel(permit)

        ledger.consume_reserved = fail_consume_once
        ledger.cancel_permit = fail_cancel_once
        with self.assertRaisesRegex(RuntimeError, "cancel failure"):
            self.prepare(records, ledger, self.source_upper(records))
        ledger.consume_reserved = original_consume
        ledger.cancel_permit = original_cancel
        gc.collect()
        self.assertTrue(consume_failed)
        self.assertTrue(cancel_failed)
        self.assert_closed(ledger)


if __name__ == "__main__":
    unittest.main()
