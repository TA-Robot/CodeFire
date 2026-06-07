import unittest

from evoagent.claim_audit import ClaimAuditStage, ClaimAuditTrailBuilder, validate_event
from evoagent.claim_review_queue import ClaimReviewDecision, ClaimReviewItem
from evoagent.evidence_pack import EvidenceReadiness, ExperimentEvidencePack


def evidence_pack(readiness: EvidenceReadiness = EvidenceReadiness.READY) -> ExperimentEvidencePack:
    return ExperimentEvidencePack(
        plan_id="plan-a",
        candidate_id="candidate-a",
        readiness=readiness,
        completeness_score=1.0 if readiness == EvidenceReadiness.READY else 0.6,
        blocking_reasons=() if readiness != EvidenceReadiness.BLOCKED else ("missing_artifacts:1",),
        review_notes=() if readiness == EvidenceReadiness.READY else ("stale_evidence:1",),
        mitigation_checklist=() if readiness == EvidenceReadiness.READY else ("refresh evidence",),
    )


def review_item(decision: ClaimReviewDecision = ClaimReviewDecision.APPROVE) -> ClaimReviewItem:
    return ClaimReviewItem(
        claim_id="claim-a",
        candidate_id="candidate-a",
        decision=decision,
        priority_score=20.0,
        reasons=("ready_for_approval",) if decision == ClaimReviewDecision.APPROVE else ("evidence_pack_requires_review",),
        checklist=() if decision == ClaimReviewDecision.APPROVE else ("refresh evidence",),
    )


# cf-atom: TEST-claim-audit-trail-records-review-decision
class ClaimAuditTrailBuilderTests(unittest.TestCase):
    def test_claim_audit_trail_records_review_decision(self):
        trail = ClaimAuditTrailBuilder().build(
            claim_id="claim-a",
            evidence_pack=evidence_pack(),
            review_item=review_item(),
            external_claim_allowed=True,
            note_refs=("decision-1",),
        )

        self.assertEqual([event.stage for event in trail.events], [
            ClaimAuditStage.EVIDENCE,
            ClaimAuditStage.REVIEW,
            ClaimAuditStage.EXTERNAL_CLAIM,
        ])
        self.assertEqual(trail.terminal_status, "allowed")
        self.assertEqual(trail.events[1].status, "approve")
        self.assertEqual(trail.events[2].source_refs, ("decision-1",))

    def test_claim_audit_trail_requires_follow_up_for_revision(self):
        trail = ClaimAuditTrailBuilder().build(
            claim_id="claim-a",
            evidence_pack=evidence_pack(EvidenceReadiness.REVIEW),
            review_item=review_item(ClaimReviewDecision.REVISE),
            external_claim_allowed=False,
        )

        self.assertEqual(trail.events[-1].stage, ClaimAuditStage.FOLLOW_UP)
        self.assertEqual(trail.terminal_status, "required")
        self.assertIn("refresh evidence", trail.events[-1].follow_up_actions)
        self.assertEqual(trail.events[2].status, "not_allowed")

    def test_claim_audit_trail_validates_matching_candidate(self):
        bad_pack = ExperimentEvidencePack(
            plan_id="plan-b",
            candidate_id="candidate-b",
            readiness=EvidenceReadiness.READY,
            completeness_score=1.0,
            blocking_reasons=(),
            review_notes=(),
            mitigation_checklist=(),
        )

        with self.assertRaises(ValueError):
            ClaimAuditTrailBuilder().build(
                claim_id="claim-a",
                evidence_pack=bad_pack,
                review_item=review_item(),
                external_claim_allowed=True,
            )


if __name__ == "__main__":
    unittest.main()
