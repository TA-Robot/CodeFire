import unittest

from evoagent.budgeting import BudgetAwarePlanner
from evoagent.models import CandidateAlgorithm, ExperimentPlan, Hypothesis


# cf-atom: TEST-budget-aware-planner-prefers-cheap-informative-candidates
class BudgetAwarePlannerTests(unittest.TestCase):
    def test_budget_aware_planner_prefers_cheap_informative_candidates(self):
        cheap = candidate("cheap", gain=0.05, novelty=0.5, cost=0.2)
        expensive = candidate("expensive", gain=0.08, novelty=0.5, cost=2.0)
        medium = candidate("medium", gain=0.04, novelty=0.4, cost=0.5)

        plan = BudgetAwarePlanner().plan([expensive, medium, cheap], budget_hours=0.7)

        self.assertEqual(plan.selected, (cheap, medium))
        self.assertAlmostEqual(plan.total_cost, 0.7)
        self.assertAlmostEqual(plan.remaining_budget, 0.0)

    def test_budget_aware_planner_rejects_negative_budget(self):
        with self.assertRaises(ValueError):
            BudgetAwarePlanner().plan([], budget_hours=-1.0)


def candidate(name: str, *, gain: float, novelty: float, cost: float) -> CandidateAlgorithm:
    hypothesis = Hypothesis(name, f"{name} rationale", expected_gain=gain, novelty=novelty)
    plan = ExperimentPlan(hypothesis, "bench", "baseline", "accuracy", estimated_cost=cost)
    return CandidateAlgorithm(name, hypothesis, plan)


if __name__ == "__main__":
    unittest.main()
