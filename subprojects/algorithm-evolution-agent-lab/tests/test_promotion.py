import unittest

from evoagent.promotion import CandidatePromotionInput, CandidatePromotionPolicy, PromotionDecision


# cf-atom: TEST-candidate-promotion-policy-gates-promotion
class CandidatePromotionPolicyTests(unittest.TestCase):
    def test_candidate_promotion_policy_gates_promotion(self):
        policy = CandidatePromotionPolicy()

        promote = policy.decide(
            CandidatePromotionInput(quality_delta=0.04, confidence=0.8, replication_count=3),
            min_delta=0.02,
            min_confidence=0.7,
            min_replications=2,
        )
        replicate = policy.decide(
            CandidatePromotionInput(quality_delta=0.04, confidence=0.6, replication_count=1),
            min_delta=0.02,
            min_confidence=0.7,
            min_replications=2,
        )
        hold = policy.decide(
            CandidatePromotionInput(quality_delta=0.04, confidence=0.8, replication_count=3, open_blockers=1),
            min_delta=0.02,
            min_confidence=0.7,
            min_replications=2,
        )

        self.assertEqual(promote.decision, PromotionDecision.PROMOTE)
        self.assertEqual(replicate.decision, PromotionDecision.REPLICATE)
        self.assertEqual(hold.decision, PromotionDecision.HOLD)


if __name__ == "__main__":
    unittest.main()
