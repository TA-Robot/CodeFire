import unittest

from evoagent.dependencies import ExperimentDependencyGraph
from evoagent.models import ExperimentPlan, Hypothesis


# cf-atom: TEST-dependency-graph-unlocks-ready-plans
class ExperimentDependencyGraphTests(unittest.TestCase):
    def test_dependency_graph_unlocks_ready_plans(self):
        graph = ExperimentDependencyGraph()
        baseline = plan("baseline")
        ablation = plan("ablation")
        graph.add_dependency(prerequisite=baseline, dependent=ablation, relation="ablates")

        self.assertEqual(graph.ready_plans(completed_plan_ids=set()), [baseline])
        self.assertEqual(graph.blockers(ablation, completed_plan_ids=set()), (baseline.uid,))
        self.assertEqual(graph.ready_plans(completed_plan_ids={baseline.uid}), [ablation])

    def test_dependency_graph_rejects_cycles(self):
        graph = ExperimentDependencyGraph()
        first = plan("first")
        second = plan("second")
        graph.add_dependency(prerequisite=first, dependent=second, relation="requires")

        with self.assertRaises(ValueError):
            graph.add_dependency(prerequisite=second, dependent=first, relation="requires")


def plan(name: str) -> ExperimentPlan:
    hypothesis = Hypothesis(name, f"{name} rationale", expected_gain=0.01, novelty=0.2)
    return ExperimentPlan(
        hypothesis=hypothesis,
        benchmark="toy-tabular",
        baseline="baseline",
        metric="accuracy",
        estimated_cost=0.1,
        command=f"run-{name}",
    )


if __name__ == "__main__":
    unittest.main()
