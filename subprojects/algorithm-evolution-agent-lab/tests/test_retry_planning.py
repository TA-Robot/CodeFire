import unittest

from evoagent.failures import FailureClassification
from evoagent.retry_planning import RetryDecision, RetryEscalationPlanner


# cf-atom: TEST-retry-escalation-planner-routes-failures-to-next-actions
class RetryEscalationPlannerTests(unittest.TestCase):
    def test_retry_escalation_planner_routes_failures_to_next_actions(self):
        planner = RetryEscalationPlanner()

        retry = planner.recommend(FailureClassification("implementation_error", 0.7, "bad command", "fix"))
        review = planner.recommend(FailureClassification("insufficient_budget", 0.8, "timeout", "budget"))
        mutate = planner.recommend(FailureClassification("invalid_hypothesis", 0.9, "regressed", "mutate"))
        archive = planner.recommend(FailureClassification("unknown", 0.2, "unclear", "archive"))

        self.assertEqual(retry.decision, RetryDecision.RETRY)
        self.assertEqual(review.decision, RetryDecision.HUMAN_REVIEW)
        self.assertEqual(mutate.decision, RetryDecision.MUTATE)
        self.assertEqual(archive.decision, RetryDecision.ARCHIVE)


if __name__ == "__main__":
    unittest.main()
