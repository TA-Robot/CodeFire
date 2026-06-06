import unittest

from evoagent.models import ExperimentPlan, Hypothesis
from evoagent.queueing import ExperimentQueue


# cf-atom: TEST-experiment-queue-orders-ready-experiments
class ExperimentQueueTests(unittest.TestCase):
    def test_experiment_queue_orders_ready_experiments_by_value_and_constraints(self):
        queue = ExperimentQueue()
        prerequisite = make_plan("prerequisite", cost=0.1)
        cheap_probe = make_plan("cheap", cost=0.05)
        expensive = make_plan("expensive", cost=10.0)
        dependent = make_plan("dependent", cost=0.1)

        queue.add(prerequisite, priority=0.2, expected_information_gain=0.1)
        queue.add(cheap_probe, priority=0.5, expected_information_gain=0.3)
        queue.add(expensive, priority=10.0, expected_information_gain=1.0)
        queue.add(
            dependent,
            priority=5.0,
            expected_information_gain=0.5,
            dependency_ids=(prerequisite.uid,),
        )

        ready = queue.ready(completed_plan_ids=set(), remaining_cost_budget=1.0)

        self.assertEqual([item.uid for item in ready], [cheap_probe.uid, prerequisite.uid])
        self.assertEqual(queue.blockers(dependent.uid, completed_plan_ids=set()), (prerequisite.uid,))
        self.assertAlmostEqual(queue.total_estimated_cost, 10.25)

    def test_experiment_queue_pops_best_ready_item(self):
        queue = ExperimentQueue()
        low = make_plan("low", cost=1.0)
        high = make_plan("high", cost=0.5)
        queue.add(low, priority=0.1, expected_information_gain=0.1)
        queue.add(high, priority=0.2, expected_information_gain=1.0)

        selected = queue.pop_next(completed_plan_ids=set())

        self.assertEqual(selected.uid, high.uid)
        self.assertEqual([item.uid for item in queue.items], [low.uid])


def make_plan(name: str, *, cost: float) -> ExperimentPlan:
    return ExperimentPlan(
        Hypothesis(name, f"{name} rationale", expected_gain=0.01, novelty=0.2),
        "toy-tabular",
        "baseline",
        "accuracy",
        cost,
        command=f"run-{name}",
    )


if __name__ == "__main__":
    unittest.main()
