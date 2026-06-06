import unittest

from evoagent.benchmarks import (
    BenchmarkDefinition,
    BenchmarkMetric,
    BenchmarkRegistry,
    summarize_repeated_runs,
)


# cf-atom: TEST-benchmark-registry-refreshes-versioned-definitions
class BenchmarkRegistryTests(unittest.TestCase):
    def test_benchmark_registry_refreshes_versioned_definitions(self):
        registry = BenchmarkRegistry()
        metric = BenchmarkMetric("accuracy", maximize=True, tolerance=0.001)
        registry.add(
            BenchmarkDefinition(
                benchmark_id="toy-tabular",
                task="binary classification",
                dataset="synthetic-v1",
                split="seeded-80-20",
                metric=metric,
                baseline_ids=("logreg-v1",),
                validation_protocol="three seeds minimum",
                version=1,
            )
        )

        refreshed = registry.refresh(
            BenchmarkDefinition(
                benchmark_id="toy-tabular",
                task="binary classification",
                dataset="synthetic-v2",
                split="seeded-80-20",
                metric=metric,
                baseline_ids=("logreg-v2",),
                validation_protocol="five seeds minimum",
                version=2,
            )
        )

        self.assertEqual(registry.get("toy-tabular"), refreshed)
        self.assertEqual(registry.by_metric("accuracy"), [refreshed])
        with self.assertRaises(ValueError):
            registry.refresh(refreshed)

    # cf-atom: TEST-repeated-run-summary-requires-signal-over-noise
    def test_repeated_run_summary_requires_signal_over_noise(self):
        metric = BenchmarkMetric("loss", maximize=False, tolerance=0.001)
        summary = summarize_repeated_runs(metric, [0.70, 0.69, 0.71, 0.70], baseline=0.75)

        self.assertAlmostEqual(summary.mean, 0.70)
        self.assertGreater(summary.normalized_delta, 0.0)
        self.assertGreaterEqual(summary.confidence, 0.5)
        self.assertTrue(summary.promoted)

    def test_single_run_does_not_promote_noisy_improvement(self):
        metric = BenchmarkMetric("accuracy", maximize=True, tolerance=0.001)
        summary = summarize_repeated_runs(metric, [0.81], baseline=0.80)

        self.assertEqual(summary.confidence, 0.25)
        self.assertFalse(summary.promoted)


if __name__ == "__main__":
    unittest.main()
