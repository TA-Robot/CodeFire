import unittest

from evoagent.benchmarks import BenchmarkMetric
from evoagent.metrics import MetricNormalizer
from evoagent.models import ExperimentPlan, ExperimentResult, Hypothesis


# cf-atom: TEST-metric-normalizer-aligns-direction-and-scale
class MetricNormalizerTests(unittest.TestCase):
    def test_metric_normalizer_aligns_direction_and_scale(self):
        hypothesis = Hypothesis("loss reducer", "reduce validation loss", expected_gain=0.02, novelty=0.3)
        plan = ExperimentPlan(hypothesis, "bench", "baseline", "loss", estimated_cost=0.1)
        result = ExperimentResult(plan, metric_value=0.40, baseline_value=0.50, confidence=0.8)

        normalized = MetricNormalizer().normalize(result, BenchmarkMetric("loss", maximize=False))

        self.assertEqual(normalized.metric_name, "loss")
        self.assertAlmostEqual(normalized.normalized_delta, 0.10)
        self.assertAlmostEqual(normalized.relative_delta, 0.20)
        self.assertTrue(normalized.improved)
        self.assertAlmostEqual(MetricNormalizer().score(result, BenchmarkMetric("loss", maximize=False)), 0.16)

    def test_metric_normalizer_applies_tolerance(self):
        hypothesis = Hypothesis("small gain", "minor score change", expected_gain=0.01, novelty=0.1)
        plan = ExperimentPlan(hypothesis, "bench", "baseline", "accuracy", estimated_cost=0.1)
        result = ExperimentResult(plan, metric_value=0.801, baseline_value=0.800, confidence=0.9)

        normalized = MetricNormalizer().normalize(result, BenchmarkMetric("accuracy", tolerance=0.002))

        self.assertEqual(normalized.normalized_delta, 0.0)
        self.assertFalse(normalized.improved)


if __name__ == "__main__":
    unittest.main()
