import runpy
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEFIRE = ROOT / "codefire"


class PythonExistingPublicationInputTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ns = runpy.run_path(str(CODEFIRE))
        cls.Ledger = cls.ns["DecisionWorkLedger"]
        cls.Profile = cls.ns["DecisionWorkProfile"]
        cls.Owner = cls.ns["AccountedHTTPUploadExistingPublicationInput"]
        cls.Prepare = staticmethod(
            cls.ns["prepare_http_upload_existing_publication_input"]
        )
        cls.CFError = cls.ns["CFError"]
        cls.BudgetError = cls.ns["ResourceBudgetExceeded"]
        cls.canonical_json = staticmethod(cls.ns["canonical_json"])
        cls.U64_MAX = cls.ns["U64_MAX"]

    def ledger(self, label="existing publication input", overrides=None):
        return self.Ledger(
            self.Profile.http_v1(overrides or {}),
            label,
            command_kind="http_request",
        )

    def assert_zero(self, ledger):
        self.assertFalse(ledger.has_open_permits())
        self.assertTrue(all(value == 0 for value in ledger.current.values()))

    @staticmethod
    def plan(count=1):
        writes = [
            (
                {"kind": "branch", "name": f"branch-{index}"},
                None,
                {"version": 1, "head": f"sha256:{index:064x}"},
            )
            for index in range(count)
        ]
        return {
            "project_root": Path("/tmp/codefire-publication-input"),
            "operation": "upload",
            "actor": "alice",
            "writes": writes,
            "result": {"status": "uploaded", "write_count": count},
            "idempotency_key": "request-1",
            "object_roots": [f"{index:064x}" for index in range(count)],
        }

    def canonical_upper(self, plan):
        values = [
            plan["operation"],
            plan["actor"],
            plan["idempotency_key"],
            plan["object_roots"],
            plan["result"],
        ]
        for authority, before, after in plan["writes"]:
            values.extend((authority, before, after))
        return sum(len(self.canonical_json(value)) for value in values)

    def prepared(self, count=1):
        plan = self.plan(count)
        upper = self.canonical_upper(plan)
        ledger = self.ledger()
        owner = self.Prepare(ledger, canonical_upper=upper)
        owner.prepare(ledger, **plan)
        return ledger, owner, plan, upper

    def test_exact_upper_freezes_one_private_plan_and_reconciles(self):
        ledger, owner, _plan, upper = self.prepared()
        expected = upper + owner.FIXED_RETAINED_BYTES
        self.assertEqual(owner._state, owner.PREPARED)
        self.assertEqual(owner._retained_exact, expected)
        self.assertEqual(ledger.current["retained_bytes"], expected)
        self.assertEqual(ledger.current["temporary_bytes"], 0)
        self.assertEqual(
            ledger.cumulative["aggregate_bytes"],
            self.Owner.WORK_BYTES_PER_CANONICAL_BYTE * upper,
        )
        expected_hashed = sum(
            len(self.canonical_json(after))
            + (0 if before is None else len(self.canonical_json(before)))
            for _authority, before, after in _plan["writes"]
        )
        self.assertEqual(ledger.counters["bytes_hashed"], expected_hashed)
        for name in ("payload", "writes", "result", "effect", "publish"):
            self.assertFalse(hasattr(owner, name))
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_exact_minus_one_fails_before_publication_and_closes(self):
        plan = self.plan()
        upper = self.canonical_upper(plan)
        ledger = self.ledger()
        owner = self.Prepare(ledger, canonical_upper=upper - 1)
        with self.assertRaisesRegex(self.CFError, "exceeds its upper"):
            owner.prepare(ledger, **plan)
        self.assertEqual(owner._state, owner.RESERVED)
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_write_count_accepts_one_and_sixty_four_but_rejects_bounds(self):
        for count in (1, 64):
            ledger, owner, _plan, _upper = self.prepared(count)
            owner.close(ledger)
            self.assert_zero(ledger)

        for count in (0, 65):
            plan = self.plan(count)
            upper = self.canonical_upper(plan)
            ledger = self.ledger(f"write-count-{count}")
            owner = self.Prepare(ledger, canonical_upper=upper)
            with self.assertRaisesRegex(self.CFError, "write count"):
                owner.prepare(ledger, **plan)
            owner.close(ledger)
            self.assert_zero(ledger)

    def test_duplicate_authority_is_rejected(self):
        plan = self.plan(2)
        plan["writes"][1] = (
            dict(plan["writes"][0][0]),
            None,
            {"version": 1, "head": "sha256:" + "f" * 64},
        )
        upper = self.canonical_upper(plan)
        ledger = self.ledger()
        owner = self.Prepare(ledger, canonical_upper=upper)
        with self.assertRaisesRegex(self.CFError, "duplicate authority"):
            owner.prepare(ledger, **plan)
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_caller_mutation_cannot_change_frozen_bytes(self):
        ledger, owner, plan, _upper = self.prepared(2)
        identity_before = owner._identity_parts
        writes_before = owner._write_plan
        result_before = owner._result_bytes
        plan["object_roots"].clear()
        plan["result"]["status"] = "changed"
        plan["writes"][0][0]["name"] = "changed"
        plan["writes"][0][2]["head"] = "changed"
        self.assertEqual(owner._identity_parts, identity_before)
        self.assertEqual(owner._write_plan, writes_before)
        self.assertEqual(owner._result_bytes, result_before)
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_nested_non_exact_json_is_rejected_before_encoding(self):
        class DictSubclass(dict):
            pass

        plan = self.plan()
        plan["result"]["nested"] = DictSubclass(value=1)
        upper = self.canonical_upper(plan)
        ledger = self.ledger()
        owner = self.Prepare(ledger, canonical_upper=upper)
        with self.assertRaisesRegex(self.CFError, "exact JSON"):
            owner.prepare(ledger, **plan)
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_large_input_with_small_upper_is_rejected_before_encoding(self):
        plan = self.plan()
        plan["idempotency_key"] = None
        plan["object_roots"] = []
        plan["result"] = {"large": "x" * 100_000}
        ledger = self.ledger()
        owner = self.Prepare(ledger, canonical_upper=64)
        with self.assertRaisesRegex(self.CFError, "exceeds its upper"):
            owner.prepare(ledger, **plan)
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_large_identity_and_authority_reject_inside_small_upper(self):
        cases = []
        actor_plan = self.plan()
        actor_plan["actor"] = "x" * 100_000
        cases.append(actor_plan)
        roots_plan = self.plan()
        roots_plan["object_roots"][0] = "x" * 100_000
        cases.append(roots_plan)
        authority_plan = self.plan()
        authority_plan["idempotency_key"] = None
        authority_plan["object_roots"] = []
        authority_plan["result"] = {}
        authority_plan["writes"][0][0]["name"] = "x" * 100_000
        cases.append(authority_plan)

        for index, plan in enumerate(cases):
            ledger = self.ledger(f"large-identity-{index}")
            owner = self.Prepare(ledger, canonical_upper=64)
            with self.assertRaisesRegex(self.CFError, "exceeds its upper"):
                owner.prepare(ledger, **plan)
            owner.close(ledger)
            self.assert_zero(ledger)

    def test_json_depth_is_bounded_inside_measured_temporary_base(self):
        nested = None
        for _ in range(self.Owner.MAX_JSON_DEPTH + 1):
            nested = [nested]
        plan = self.plan()
        plan["result"] = {"nested": nested}
        upper = self.canonical_upper(plan)
        ledger = self.ledger()
        owner = self.Prepare(ledger, canonical_upper=upper)
        with self.assertRaisesRegex(self.CFError, "JSON depth"):
            owner.prepare(ledger, **plan)
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_merge_request_authority_cannot_escape_relative_namespace(self):
        plan = self.plan()
        plan["writes"][0] = (
            {"kind": "merge_request", "id": "../../outside"},
            None,
            {"version": 1},
        )
        upper = self.canonical_upper(plan)
        ledger = self.ledger()
        owner = self.Prepare(ledger, canonical_upper=upper)
        with self.assertRaisesRegex(self.CFError, "invalid remote merge-request"):
            owner.prepare(ledger, **plan)
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_factory_cleanup_failure_is_retried_from_ledger_bootstrap(self):
        plan = self.plan()
        upper = self.canonical_upper(plan)
        ledger = self.ledger()
        owner_init = self.Owner.__init__
        release = ledger.release

        def fail_init(_owner, *_args, **_kwargs):
            raise RuntimeError("publication owner allocation fault")

        def fail_release(dimension, amount):
            if dimension == "retained_bytes":
                raise RuntimeError("publication bootstrap release fault")
            return release(dimension, amount)

        self.Owner.__init__ = fail_init
        ledger.release = fail_release
        try:
            with self.assertRaisesRegex(RuntimeError, "bootstrap release fault"):
                self.Prepare(ledger, canonical_upper=upper)
            bootstrap = ledger.__dict__.get(
                "_upload_publication_input_bootstrap"
            )
            self.assertIsNotNone(bootstrap)
            self.assertGreater(ledger.current["retained_bytes"], 0)
        finally:
            self.Owner.__init__ = owner_init
            del ledger.release

        owner = self.Prepare(ledger, canonical_upper=upper)
        self.assertNotIn("_upload_publication_input_bootstrap", ledger.__dict__)
        owner.prepare(ledger, **plan)
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_foreign_ledger_is_rejected_without_cross_release(self):
        ledger, owner, _plan, _upper = self.prepared()
        foreign = self.ledger("foreign publication input")
        with self.assertRaisesRegex(self.CFError, "another ledger"):
            owner.close(foreign)
        self.assert_zero(foreign)
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_close_release_fault_retries_same_owner(self):
        ledger, owner, _plan, _upper = self.prepared()
        original = ledger.release
        failed = False

        def fail_once(dimension, amount):
            nonlocal failed
            if dimension == "retained_bytes" and not failed:
                failed = True
                raise RuntimeError("publication input release fault")
            return original(dimension, amount)

        ledger.release = fail_once
        with self.assertRaisesRegex(RuntimeError, "release fault"):
            owner.close(ledger)
        self.assertFalse(owner._closed)
        self.assertIsNone(owner._write_plan)
        owner.close(ledger)
        del ledger.release
        self.assertTrue(failed)
        self.assert_zero(ledger)

    def test_retained_and_temporary_limits_are_acquired_before_owner(self):
        plan = self.plan()
        upper = self.canonical_upper(plan)
        fixed = self.Owner.FIXED_RETAINED_BYTES
        retained = upper + fixed
        temporary = self.Owner.TEMPORARY_BASE_BYTES + 4 * upper
        for dimension, limit in (
            ("retained_bytes", retained - 1),
            ("temporary_bytes", temporary - 1),
        ):
            ledger = self.ledger(f"{dimension}-limit", {dimension: limit})
            with self.assertRaises(self.BudgetError):
                self.Prepare(ledger, canonical_upper=upper)
            self.assert_zero(ledger)

        ledger = self.ledger(
            "exact-limits",
            {"retained_bytes": retained, "temporary_bytes": temporary},
        )
        owner = self.Prepare(ledger, canonical_upper=upper)
        owner.prepare(ledger, **plan)
        owner.close(ledger)
        self.assert_zero(ledger)

    def test_u64_and_direct_constructor_are_rejected_without_owner(self):
        ledger = self.ledger()
        with self.assertRaisesRegex(self.CFError, "exceeds u64"):
            self.Prepare(ledger, canonical_upper=self.U64_MAX)
        transition = self.ns["_StablePermitTransition"](ledger)
        with self.assertRaisesRegex(self.CFError, "use prepare"):
            self.Owner(ledger, 1, transition)
        self.assert_zero(ledger)


if __name__ == "__main__":
    unittest.main()
