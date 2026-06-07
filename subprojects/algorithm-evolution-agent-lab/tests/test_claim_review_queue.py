import unittest

from evoagent.claim_review_queue import ClaimReviewDecision, ClaimReviewQueue, ClaimReviewRequest
from evoagent.evidence_pack import (
    EvidenceReadiness,
    ExperimentEvidencePack,
)


def pack(
    *,
    plan_id: str,
    candidate_id: str,
    readiness: EvidenceReadiness,
    score: float,
    blocking_reasons: tuple[str, ...] = (),
    review_notes: tuple[str, ...] = (),
    checklist: tuple[str, ...] = (),
) -> ExperimentEvidencePack:
    return ExperimentEvidencePack(
        plan_id=plan_id,
        candidate_id=candidate_id,
        readiness=readiness,
        completeness_score=score,
        blocking_reasons=blocking_reasons,
        review_notes=review_notes,
        mitigation_checklist=checklist,
    )


# cf-atom: TEST-claim-review-queue-prioritizes-blocked-claims
class ClaimReviewQueueTests(unittest.TestCase):
    def test_claim_review_queue_prioritizes_blocked_claims(self):
        queue = ClaimReviewQueue()

        items = queue.build(
            (
                ClaimReviewRequest(
                    claim_id="claim-ready",
                    candidate_id="candidate-a",
                    evidence_pack=pack(
                        plan_id="plan-a",
                        candidate_id="candidate-a",
                        readiness=EvidenceReadiness.READY,
                        score=1.0,
                    ),
                ),
                ClaimReviewRequest(
                    claim_id="claim-blocked",
                    candidate_id="candidate-b",
                    evidence_pack=pack(
                        plan_id="plan-b",
                        candidate_id="candidate-b",
                        readiness=EvidenceReadiness.BLOCKED,
                        score=0.45,
                        blocking_reasons=("missing_artifacts:1",),
                        checklist=("attach metrics artifact",),
                    ),
                    open_objection_count=2,
                    blocking_objection_count=1,
                ),
            )
        )

        self.assertEqual(items[0].claim_id, "claim-blocked")
        self.assertEqual(items[0].decision, ClaimReviewDecision.BLOCK)
        self.assertIn("missing_artifacts:1", items[0].reasons)
        self.assertIn("resolve_reviewer_objections", items[0].checklist)
        self.assertEqual(items[1].decision, ClaimReviewDecision.APPROVE)

    def test_claim_review_queue_marks_unassigned_review_for_revision(self):
        queue = ClaimReviewQueue()

        item = queue.build(
            (
                ClaimReviewRequest(
                    claim_id="claim-review",
                    candidate_id="candidate-c",
                    evidence_pack=pack(
                        plan_id="plan-c",
                        candidate_id="candidate-c",
                        readiness=EvidenceReadiness.REVIEW,
                        score=0.7,
                        review_notes=("stale_evidence:1",),
                    ),
                    assigned_reviewer_count=0,
                ),
            )
        )[0]

        self.assertEqual(item.decision, ClaimReviewDecision.REVISE)
        self.assertEqual(item.reasons, ("evidence_pack_requires_review", "no_assigned_reviewer"))
        self.assertEqual(item.checklist, ("assign_reviewer",))

    def test_claim_review_queue_validates_objection_counts(self):
        queue = ClaimReviewQueue()

        with self.assertRaises(ValueError):
            queue.build(
                (
                    ClaimReviewRequest(
                        claim_id="claim-invalid",
                        candidate_id="candidate-d",
                        evidence_pack=pack(
                            plan_id="plan-d",
                            candidate_id="candidate-d",
                            readiness=EvidenceReadiness.READY,
                            score=1.0,
                        ),
                        open_objection_count=0,
                        blocking_objection_count=1,
                    ),
                )
            )


if __name__ == "__main__":
    unittest.main()
