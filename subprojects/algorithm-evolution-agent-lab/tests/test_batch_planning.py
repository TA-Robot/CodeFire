import unittest

from evoagent.batch_planning import BudgetedExperimentBatchPlanner
from evoagent.models import ExperimentPlan, Hypothesis
from evoagent.queueing import QueuedExperiment


# cf-atom: TEST-budgeted-experiment-batch-planner-respects-cost-and-diversity
class BudgetedExperimentBatchPlannerTests(unittest.TestCase):
    def test_budgeted_experiment_batch_planner_respects_cost_and_diversity(self):
        first_tabular = queued("first", "tabular", cost=0.2, gain=0.8)
        second_tabular = queued("second", "tabular", cost=0.1, gain=0.7)
        sequence = queued("sequence", "sequence", cost=0.3, gain=0.9)
        expensive = queued("expensive", "optimizer", cost=10.0, gain=10.0)

        batch = BudgetedExperimentBatchPlanner().plan(
            (first_tabular, second_tabular, sequence, expensive),
            max_total_cost=0.6,
            max_items=3,
            max_per_benchmark=1,
        )

        self.assertEqual([item.uid for item in batch.items], [second_tabular.uid, sequence.uid])
        self.assertAlmostEqual(batch.total_estimated_cost, 0.4)
        self.assertEqual(batch.deferred_ids, (first_tabular.uid, expensive.uid))


def queued(name: str, benchmark: str, *, cost: float, gain: float) -> QueuedExperiment:
    plan = ExperimentPlan(
        Hypothesis(name, f"{name} rationale", expected_gain=gain, novelty=0.2),
        benchmark,
        "baseline",
        "accuracy",
        cost,
        command=f"run-{name}",
    )
    return QueuedExperiment(
        plan=plan,
        priority=0.1,
        expected_information_gain=gain,
        estimated_cost=cost,
    )


if __name__ == "__main__":
    unittest.main()
