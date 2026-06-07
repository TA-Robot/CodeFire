import unittest

from evoagent.claim_audit import ClaimAuditTrail
from evoagent.claim_release import ClaimReleaseDecision, ClaimReleaseGate, ClaimReleaseInput
from evoagent.claim_review_queue import ClaimReviewDecision, ClaimReviewItem
from evoagent.evidence_pack import EvidenceReadiness, ExperimentEvidencePack


def evidence(readiness: EvidenceReadiness = EvidenceReadiness.READY) -> ExperimentEvidencePack:
    return ExperimentEvidencePack(
        plan_id="plan-a",
        candidate_id="candidate-a",
        readiness=readiness,
        completeness_score=1.0 if readiness == EvidenceReadiness.READY else 0.4,
        blocking_reasons=() if readiness != EvidenceReadiness.BLOCKED else ("missing_artifacts:1",),
        review_notes=() if readiness == EvidenceReadiness.READY else ("stale_evidence:1",),
        mitigation_checklist=() if readiness == EvidenceReadiness.READY else ("refresh evidence",),
    )


def review(decision: ClaimReviewDecision = ClaimReviewDecision.APPROVE) -> ClaimReviewItem:
    return ClaimReviewItem(
        claim_id="claim-a",
        candidate_id="candidate-a",
        decision=decision,
        priority_score=20.0,
        reasons=("ready_for_approval",),
        checklist=() if decision == ClaimReviewDecision.APPROVE else ("resolve reviewer objection",),
    )


def audit(status: str = "allowed") -> ClaimAuditTrail:
    return ClaimAuditTrail(
        claim_id="claim-a",
        candidate_id="candidate-a",
        events=(),
        terminal_status=status,
    )


# cf-atom: TEST-claim-release-gate-allows-complete-claims
class ClaimReleaseGateTests(unittest.TestCase):
    def test_claim_release_gate_allows_complete_claims(self):
        result = ClaimReleaseGate().evaluate(
            ClaimReleaseInput(
                claim_id="claim-a",
                candidate_id="candidate-a",
                evidence_pack=evidence(),
                review_item=review(),
                audit_trail=audit(),
                required_artifacts=("report.md", "reproduction.md"),
                available_artifacts=("reproduction.md", "report.md", "metrics.json"),
            )
        )

        self.assertEqual(result.decision, ClaimReleaseDecision.ALLOW)
        self.assertEqual(result.blocking_reasons, ())
        self.assertEqual(result.missing_artifacts, ())

    def test_claim_release_gate_holds_missing_artifacts(self):
        result = ClaimReleaseGate().evaluate(
            ClaimReleaseInput(
                claim_id="claim-a",
                candidate_id="candidate-a",
                evidence_pack=evidence(),
                review_item=review(),
                audit_trail=audit(),
                required_artifacts=("report.md", "reproduction.md"),
                available_artifacts=("report.md",),
            )
        )

        self.assertEqual(result.decision, ClaimReleaseDecision.HOLD)
        self.assertEqual(result.missing_artifacts, ("reproduction.md",))
        self.assertIn("attach:reproduction.md", result.release_checklist)

    def test_claim_release_gate_rejects_blocked_audit(self):
        result = ClaimReleaseGate().evaluate(
            ClaimReleaseInput(
                claim_id="claim-a",
                candidate_id="candidate-a",
                evidence_pack=evidence(EvidenceReadiness.BLOCKED),
                review_item=review(ClaimReviewDecision.BLOCK),
                audit_trail=audit("required"),
            )
        )

        self.assertEqual(result.decision, ClaimReleaseDecision.REJECT)
        self.assertEqual(
            result.blocking_reasons,
            ("evidence_blocked", "review_blocked", "audit_follow_up_required"),
        )


if __name__ == "__main__":
    unittest.main()
