from dataclasses import replace
import unittest

from evoagent.loop import EvolutionAgent
from evoagent.models import ExperimentGoal, Hypothesis


# cf-atom: TEST-evolution-agent-selects-best-candidate
class EvolutionAgentTests(unittest.TestCase):
    def test_evolution_agent_selects_best_candidate(self):
        agent = EvolutionAgent(ExperimentGoal(objective="Improve sample efficiency"))
        candidates = agent.seed(
            [
                Hypothesis("low gain", "baseline variation", expected_gain=0.01, novelty=0.10),
                Hypothesis("high gain", "stronger research direction", expected_gain=0.10, novelty=0.60),
            ]
        )

        self.assertEqual(agent.next_candidate(), candidates[1])

    # cf-atom: TEST-evolution-agent-schedules-balanced-portfolio
    def test_evolution_agent_schedules_balanced_portfolio(self):
        agent = EvolutionAgent(ExperimentGoal(objective="Improve sample efficiency"))
        candidates = agent.seed(
            [
                Hypothesis("exploit", "high expected gain", expected_gain=0.12, novelty=0.20),
                Hypothesis("explore", "novel mechanism", expected_gain=0.02, novelty=0.95),
                Hypothesis("cheap", "cheap uncertainty reducer", expected_gain=0.03, novelty=0.30),
            ]
        )
        cheap_candidate = replace(
            candidates[2],
            plan=replace(candidates[2].plan, estimated_cost=0.01),
        )
        agent.candidates[2] = cheap_candidate

        portfolio = agent.portfolio(limit=3)

        self.assertEqual(portfolio[0], candidates[0])
        self.assertIn(candidates[1], portfolio)
        self.assertIn(cheap_candidate, portfolio)


if __name__ == "__main__":
    unittest.main()
