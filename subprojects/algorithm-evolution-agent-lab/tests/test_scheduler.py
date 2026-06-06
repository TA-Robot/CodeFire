import unittest

from evoagent.models import CandidateAlgorithm, ExperimentPlan, Hypothesis
from evoagent.scheduler import PortfolioScheduler


# cf-atom: TEST-portfolio-scheduler-balances-exploit-explore-cost
class PortfolioSchedulerTests(unittest.TestCase):
    def test_portfolio_scheduler_balances_exploit_explore_cost(self):
        exploit = candidate("exploit", gain=0.12, novelty=0.2, cost=1.0)
        explore = candidate("explore", gain=0.02, novelty=0.95, cost=1.0)
        cheap = candidate("cheap", gain=0.03, novelty=0.3, cost=0.01)

        portfolio = PortfolioScheduler().schedule([cheap, explore, exploit], limit=3)

        self.assertEqual(portfolio[0], exploit)
        self.assertIn(explore, portfolio)
        self.assertIn(cheap, portfolio)


def candidate(name: str, *, gain: float, novelty: float, cost: float) -> CandidateAlgorithm:
    hypothesis = Hypothesis(name, f"{name} rationale", gain, novelty)
    plan = ExperimentPlan(hypothesis, "bench", "baseline", "accuracy", cost)
    return CandidateAlgorithm(name, hypothesis, plan)


if __name__ == "__main__":
    unittest.main()
