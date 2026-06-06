import unittest
from pathlib import Path

from evoagent.failures import FailureClassifier
from evoagent.models import ExperimentPlan, ExperimentResult, ExperimentRun, Hypothesis


# cf-atom: TEST-failure-classifier-categorizes-run-outcomes
class FailureClassifierTests(unittest.TestCase):
    def test_failure_classifier_detects_budget_and_benchmark_failures(self):
        plan = make_plan()
        timeout_run = make_run(plan, returncode=124, stderr="timeout after 1s")
        mismatch_run = make_run(plan, returncode=2, stderr="dataset split does not match benchmark")

        classifier = FailureClassifier()

        self.assertEqual(classifier.classify(timeout_run).category, "insufficient_budget")
        self.assertEqual(classifier.classify(mismatch_run).category, "benchmark_mismatch")

    def test_failure_classifier_distinguishes_inconclusive_and_invalid_hypothesis(self):
        plan = make_plan()
        run = make_run(plan, returncode=0)
        inconclusive = ExperimentResult(plan, metric_value=0.81, baseline_value=0.80, confidence=0.2)
        regressed = ExperimentResult(plan, metric_value=0.72, baseline_value=0.80, confidence=0.9)

        classifier = FailureClassifier()

        self.assertEqual(classifier.classify(run, inconclusive).category, "inconclusive_result")
        self.assertEqual(classifier.classify(run, regressed).category, "invalid_hypothesis")


def make_plan() -> ExperimentPlan:
    return ExperimentPlan(
        Hypothesis("failure", "classify failed run", expected_gain=0.02, novelty=0.4),
        "toy-tabular",
        "baseline",
        "accuracy",
        0.1,
    )


def make_run(plan: ExperimentPlan, *, returncode: int, stderr: str = "") -> ExperimentRun:
    return ExperimentRun(
        run_id="run_failure",
        plan=plan,
        command="python experiment.py",
        cwd=Path.cwd(),
        returncode=returncode,
        stdout="",
        stderr=stderr,
        duration_seconds=0.1,
    )


if __name__ == "__main__":
    unittest.main()
