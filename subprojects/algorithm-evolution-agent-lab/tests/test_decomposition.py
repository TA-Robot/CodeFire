import unittest

from evoagent.decomposition import ObjectiveDecomposer
from evoagent.models import ExperimentGoal


# cf-atom: TEST-objective-decomposer-creates-measurable-subgoals
class ObjectiveDecomposerTests(unittest.TestCase):
    def test_objective_decomposer_creates_measurable_subgoals(self):
        goal = ExperimentGoal(
            objective="Improve sample efficiency",
            target_metric="validation_score",
            budget_hours=2.0,
        )

        subgoals = ObjectiveDecomposer().decompose(
            goal,
            benchmarks=("toy-tabular", "synthetic-sequence"),
            target_improvement=0.02,
            acceptable_risk="low",
        )

        self.assertEqual(len(subgoals), 2)
        self.assertEqual(subgoals[0].benchmark, "toy-tabular")
        self.assertEqual(subgoals[0].metric, "validation_score")
        self.assertEqual(subgoals[0].target_improvement, 0.02)
        self.assertEqual(subgoals[0].budget_hours, 1.0)
        self.assertEqual(subgoals[0].acceptable_risk, "low")

    def test_objective_decomposer_requires_benchmark(self):
        with self.assertRaises(ValueError):
            ObjectiveDecomposer().decompose(
                ExperimentGoal("Improve sample efficiency"),
                benchmarks=(),
                target_improvement=0.01,
            )


if __name__ == "__main__":
    unittest.main()
