import unittest

from evoagent.triage import ExperimentTriageBoard, ExperimentTriageSignal, TriageAction


# cf-atom: TEST-experiment-triage-board-ranks-actions
class ExperimentTriageBoardTests(unittest.TestCase):
    def test_experiment_triage_board_ranks_actions(self):
        board = ExperimentTriageBoard()

        ranked = board.rank(
            (
                ExperimentTriageSignal(
                    plan_id="blocked",
                    priority=10.0,
                    expected_information_gain=10.0,
                    estimated_cost=0.1,
                    open_blockers=1,
                    rationale="blocked despite high value",
                ),
                ExperimentTriageSignal(
                    plan_id="promote",
                    priority=0.5,
                    expected_information_gain=0.2,
                    estimated_cost=0.2,
                    promotion_decision="promote",
                    learning_trend="improving",
                ),
                ExperimentTriageSignal(
                    plan_id="replicate",
                    priority=0.4,
                    expected_information_gain=0.8,
                    estimated_cost=0.5,
                    promotion_decision="replicate",
                    learning_trend="unstable",
                ),
                ExperimentTriageSignal(
                    plan_id="mutate",
                    priority=0.4,
                    expected_information_gain=0.2,
                    estimated_cost=0.5,
                    promotion_decision="run",
                    learning_trend="plateau",
                ),
            ),
            remaining_budget=1.0,
        )

        by_id = {item.plan_id: item for item in ranked}
        self.assertEqual(by_id["promote"].action, TriageAction.PROMOTE)
        self.assertEqual(by_id["replicate"].action, TriageAction.REPLICATE)
        self.assertEqual(by_id["mutate"].action, TriageAction.MUTATE)
        self.assertEqual(by_id["blocked"].action, TriageAction.HOLD)
        self.assertEqual(ranked[0].plan_id, "promote")
        self.assertIn("open_blockers", by_id["blocked"].reasons)

    def test_experiment_triage_board_holds_over_budget_work(self):
        board = ExperimentTriageBoard()

        ranked = board.rank(
            (
                ExperimentTriageSignal(
                    plan_id="expensive",
                    priority=1.0,
                    expected_information_gain=2.0,
                    estimated_cost=4.0,
                ),
            ),
            remaining_budget=1.0,
        )

        self.assertEqual(ranked[0].action, TriageAction.HOLD)
        self.assertIn("over_budget", ranked[0].reasons)


if __name__ == "__main__":
    unittest.main()
