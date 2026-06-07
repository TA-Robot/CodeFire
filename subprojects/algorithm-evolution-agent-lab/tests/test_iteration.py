import unittest

from evoagent.iteration import ExperimentIterationPlanner
from evoagent.triage import ExperimentTriageSignal


# cf-atom: TEST-experiment-iteration-planner-builds-bounded-plan
class ExperimentIterationPlannerTests(unittest.TestCase):
    def test_experiment_iteration_planner_builds_bounded_plan(self):
        planner = ExperimentIterationPlanner()

        plan = planner.plan(
            (
                ExperimentTriageSignal(
                    plan_id="promote",
                    priority=0.5,
                    expected_information_gain=0.2,
                    estimated_cost=0.1,
                    promotion_decision="promote",
                    learning_trend="improving",
                ),
                ExperimentTriageSignal(
                    plan_id="run-cheap",
                    priority=1.0,
                    expected_information_gain=1.0,
                    estimated_cost=0.3,
                ),
                ExperimentTriageSignal(
                    plan_id="replicate",
                    priority=0.8,
                    expected_information_gain=0.4,
                    estimated_cost=0.4,
                    promotion_decision="replicate",
                    learning_trend="unstable",
                ),
                ExperimentTriageSignal(
                    plan_id="run-expensive",
                    priority=0.7,
                    expected_information_gain=1.0,
                    estimated_cost=0.8,
                ),
                ExperimentTriageSignal(
                    plan_id="blocked",
                    priority=10.0,
                    expected_information_gain=10.0,
                    estimated_cost=0.1,
                    open_blockers=1,
                ),
            ),
            remaining_budget=0.7,
            max_active=2,
            review_capacity=1,
        )

        self.assertEqual(plan.review_plan_ids, ("promote",))
        self.assertEqual(plan.active_plan_ids, ("run-cheap", "replicate"))
        self.assertLessEqual(plan.budget_used, 0.7)
        self.assertIn("blocked", plan.deferred_plan_ids)
        self.assertIn("run-expensive", plan.deferred_plan_ids)

    def test_experiment_iteration_planner_validates_capacity(self):
        planner = ExperimentIterationPlanner()

        with self.assertRaises(ValueError):
            planner.plan((), remaining_budget=1.0, max_active=0, review_capacity=1)


if __name__ == "__main__":
    unittest.main()
