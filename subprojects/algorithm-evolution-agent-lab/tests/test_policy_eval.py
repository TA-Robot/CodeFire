import unittest

from evoagent.models import CandidateAlgorithm, ExperimentPlan, ExperimentResult, Hypothesis
from evoagent.policy_eval import PolicyEvaluator


# cf-atom: TEST-policy-evaluator-measures-selection-regret
class PolicyEvaluatorTests(unittest.TestCase):
    def test_policy_evaluator_measures_selection_regret(self):
        winner = candidate("winner", gain=0.10, observed=0.12, confidence=0.9, cost=1.0)
        runner_up = candidate("runner-up", gain=0.08, observed=0.08, confidence=0.8, cost=1.0)
        cheap_probe = candidate("cheap-probe", gain=0.02, observed=0.01, confidence=0.9, cost=0.1)

        report = PolicyEvaluator().evaluate(
            policy_name="balanced-portfolio",
            candidates=[cheap_probe, runner_up, winner],
            selected=[winner, cheap_probe],
            limit=2,
        )

        self.assertEqual(report.oracle_names, ("winner", "runner-up"))
        self.assertEqual(report.selected_names, ("winner", "cheap-probe"))
        self.assertEqual(report.hit_rate, 0.5)
        self.assertAlmostEqual(report.selected_mean_delta, 0.065)
        self.assertAlmostEqual(report.oracle_mean_delta, 0.10)
        self.assertAlmostEqual(report.regret, 0.035)
        self.assertAlmostEqual(report.selected_cost, 1.1)


def candidate(name: str, *, gain: float, observed: float, confidence: float, cost: float) -> CandidateAlgorithm:
    hypothesis = Hypothesis(name, f"{name} rationale", gain, novelty=0.5)
    plan = ExperimentPlan(hypothesis, "bench", "baseline", "accuracy", cost)
    result = ExperimentResult(plan, metric_value=1.0 + observed, baseline_value=1.0, confidence=confidence)
    return CandidateAlgorithm(name, hypothesis, plan, result=result)


if __name__ == "__main__":
    unittest.main()
