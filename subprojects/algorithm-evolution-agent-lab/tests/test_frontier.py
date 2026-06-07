import unittest

from evoagent.frontier import FrontierAction, ResearchFrontierMap, ResearchFrontierSignal


# cf-atom: TEST-research-frontier-map-ranks-opportunities
class ResearchFrontierMapTests(unittest.TestCase):
    def test_research_frontier_map_ranks_opportunities(self):
        frontier = ResearchFrontierMap()

        ranked = frontier.rank(
            (
                ResearchFrontierSignal(
                    frontier_id="safe-low-value",
                    mechanism_family="regularized_baseline_tuning",
                    target="toy-tabular",
                    baseline_gap=0.05,
                    expected_information_gain=0.2,
                    novelty_score=0.1,
                    estimated_cost=0.2,
                ),
                ResearchFrontierSignal(
                    frontier_id="novel-gap",
                    mechanism_family="adaptive_feature_selector",
                    target="toy-tabular",
                    baseline_gap=0.4,
                    expected_information_gain=0.8,
                    novelty_score=0.9,
                    estimated_cost=0.5,
                    rationale="large gap and novel mechanism",
                ),
                ResearchFrontierSignal(
                    frontier_id="blocked",
                    mechanism_family="expensive_meta_learner",
                    target="toy-tabular",
                    baseline_gap=0.6,
                    expected_information_gain=0.9,
                    novelty_score=0.8,
                    estimated_cost=0.6,
                    blocker_risks=1,
                ),
            ),
            remaining_budget=1.0,
        )

        by_id = {item.frontier_id: item for item in ranked}
        self.assertEqual(ranked[0].frontier_id, "novel-gap")
        self.assertEqual(by_id["novel-gap"].action, FrontierAction.RUN_PROBE)
        self.assertEqual(by_id["novel-gap"].expected_evidence_gain, "high")
        self.assertEqual(by_id["blocked"].action, FrontierAction.MITIGATE_RISK)
        self.assertIn("blocker_risks", by_id["blocked"].reasons)

    def test_research_frontier_map_defers_or_redesigns_unsafe_work(self):
        frontier = ResearchFrontierMap()

        ranked = frontier.rank(
            (
                ResearchFrontierSignal(
                    frontier_id="over-budget",
                    mechanism_family="large_ensemble_search",
                    target="toy-tabular",
                    baseline_gap=0.3,
                    expected_information_gain=0.7,
                    novelty_score=0.6,
                    estimated_cost=3.0,
                ),
                ResearchFrontierSignal(
                    frontier_id="known-bad",
                    mechanism_family="leaky_target_encoding",
                    target="toy-tabular",
                    baseline_gap=0.5,
                    expected_information_gain=0.8,
                    novelty_score=0.7,
                    estimated_cost=0.3,
                    negative_result_overlap=0.9,
                ),
            ),
            remaining_budget=1.0,
        )

        by_id = {item.frontier_id: item for item in ranked}
        self.assertEqual(by_id["over-budget"].action, FrontierAction.DEFER)
        self.assertIn("over_budget", by_id["over-budget"].reasons)
        self.assertEqual(by_id["known-bad"].action, FrontierAction.REDESIGN)
        self.assertEqual(by_id["known-bad"].primary_risk, "repeated_negative_result")


if __name__ == "__main__":
    unittest.main()
