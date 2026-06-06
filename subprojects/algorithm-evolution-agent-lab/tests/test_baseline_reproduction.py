import unittest

from evoagent.baseline_reproduction import BaselineReproduction, BaselineReproductionLedger


# cf-atom: TEST-baseline-reproduction-ledger-requires-reproduced-baseline
class BaselineReproductionLedgerTests(unittest.TestCase):
    def test_baseline_reproduction_ledger_requires_reproduced_baseline(self):
        ledger = BaselineReproductionLedger()
        imported = ledger.add(
            BaselineReproduction(
                baseline_id="paper-baseline",
                benchmark="toy-tabular",
                metric="accuracy",
                value=0.80,
                source="paper",
                reproduced=False,
                evidence_refs=("paper-table-1",),
            )
        )
        reproduced = ledger.add(
            BaselineReproduction(
                baseline_id="local-baseline",
                benchmark="toy-tabular",
                metric="accuracy",
                value=0.79,
                source="local-run",
                reproduced=True,
                evidence_refs=("run-1",),
            )
        )

        self.assertEqual(ledger.all(), (imported, reproduced))
        self.assertEqual(ledger.reproduced_for(benchmark="toy-tabular", metric="accuracy"), (reproduced,))
        self.assertTrue(ledger.has_reproduced_baseline(benchmark="toy-tabular", metric="accuracy"))
        self.assertFalse(ledger.has_reproduced_baseline(benchmark="toy-tabular", metric="loss"))

    def test_baseline_reproduction_ledger_requires_identity(self):
        with self.assertRaises(ValueError):
            BaselineReproductionLedger().add(
                BaselineReproduction("", "bench", "metric", 0.0, "local", True, ())
            )


if __name__ == "__main__":
    unittest.main()
