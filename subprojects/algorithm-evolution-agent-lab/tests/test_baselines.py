import unittest

from evoagent.baselines import BaselineRecord, BaselineRegistry


# cf-atom: TEST-baseline-registry-selects-best
class BaselineRegistryTests(unittest.TestCase):
    def test_baseline_registry_selects_best_for_metric_direction(self):
        registry = BaselineRegistry()
        registry.add(
            BaselineRecord(
                baseline_id="tabular-logreg",
                benchmark="toy-tabular",
                metric="accuracy",
                value=0.78,
                source_type="reproduced",
                evidence_refs=("ev_1",),
            )
        )
        registry.add(
            BaselineRecord(
                baseline_id="tabular-tree",
                benchmark="toy-tabular",
                metric="accuracy",
                value=0.82,
                source_type="internal",
            )
        )
        registry.add(
            BaselineRecord(
                baseline_id="optimizer-default",
                benchmark="toy-optimizer",
                metric="loss",
                value=0.25,
                source_type="sanity_check",
            )
        )

        self.assertEqual(registry.best(benchmark="toy-tabular", metric="accuracy").baseline_id, "tabular-tree")
        self.assertEqual(
            registry.best(benchmark="toy-optimizer", metric="loss", maximize=False).baseline_id,
            "optimizer-default",
        )
        self.assertEqual(len(registry.for_benchmark("toy-tabular")), 2)

    def test_baseline_registry_rejects_duplicate_ids(self):
        registry = BaselineRegistry()
        record = BaselineRecord("baseline", "benchmark", "metric", 1.0, "literature")
        registry.add(record)

        with self.assertRaises(ValueError):
            registry.add(record)

    def test_baseline_registry_rejects_unknown_source_type(self):
        with self.assertRaises(ValueError):
            BaselineRegistry().add(BaselineRecord("baseline", "benchmark", "metric", 1.0, "unknown"))


if __name__ == "__main__":
    unittest.main()
