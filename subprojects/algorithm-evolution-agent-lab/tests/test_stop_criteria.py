import unittest

from evoagent.stop_criteria import StopCriteria, StopCriteriaEvaluator, StopObservation


# cf-atom: TEST-stop-criteria-evaluator-blocks-run-continuation
class StopCriteriaEvaluatorTests(unittest.TestCase):
    def test_stop_criteria_evaluator_flags_all_blocking_reasons(self):
        observation = StopObservation(
            elapsed_seconds=10.0,
            cumulative_cost=2.5,
            recent_metric_values=(0.81, 0.811, 0.8105),
            output_valid=False,
            safety_violations=("disallowed_command",),
        )
        criteria = StopCriteria(
            timeout_seconds=5.0,
            max_cost=2.0,
            plateau_window=3,
            min_delta=0.01,
        )

        findings = StopCriteriaEvaluator().evaluate(observation, criteria)

        self.assertEqual(
            {finding.kind for finding in findings},
            {
                "timeout",
                "maximum_cost",
                "invalid_output",
                "safety_policy_violation",
                "metric_plateau",
            },
        )

    def test_stop_criteria_evaluator_allows_progress_when_thresholds_clear(self):
        observation = StopObservation(
            elapsed_seconds=1.0,
            cumulative_cost=0.2,
            recent_metric_values=(0.7, 0.75, 0.82),
        )
        criteria = StopCriteria(timeout_seconds=5.0, max_cost=2.0, plateau_window=3, min_delta=0.01)

        self.assertFalse(StopCriteriaEvaluator().should_stop(observation, criteria))


if __name__ == "__main__":
    unittest.main()
