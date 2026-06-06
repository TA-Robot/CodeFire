import unittest

from evoagent.models import CandidateAlgorithm, ExperimentPlan, ExperimentResult, Hypothesis
from evoagent.regression import RegressionDetector


# cf-atom: TEST-regression-detector-flags-shared-benchmark-drop
class RegressionDetectorTests(unittest.TestCase):
    def test_regression_detector_flags_shared_benchmark_drop(self):
        reference = candidate("accepted", metric_value=0.82)
        regressed = candidate("new-variant", metric_value=0.78)

        findings = RegressionDetector().detect(candidate=regressed, references=[reference], tolerance=0.01)

        self.assertEqual(len(findings), 1)
        self.assertEqual(findings[0].candidate_name, "new-variant")
        self.assertEqual(findings[0].reference_name, "accepted")
        self.assertAlmostEqual(findings[0].observed_gap, -0.04)

    def test_regression_detector_ignores_unrelated_metric(self):
        reference = candidate("accepted", metric_value=0.82, metric="accuracy")
        regressed = candidate("new-variant", metric_value=0.30, metric="loss")

        findings = RegressionDetector().detect(candidate=regressed, references=[reference])

        self.assertEqual(findings, [])


def candidate(name: str, *, metric_value: float, metric: str = "accuracy") -> CandidateAlgorithm:
    hypothesis = Hypothesis(name, f"{name} rationale", expected_gain=0.01, novelty=0.3)
    plan = ExperimentPlan(hypothesis, "toy-tabular", "baseline", metric, 0.1)
    result = ExperimentResult(plan, metric_value=metric_value, baseline_value=0.80, confidence=0.8)
    return CandidateAlgorithm(name, hypothesis, plan, result=result)


if __name__ == "__main__":
    unittest.main()
