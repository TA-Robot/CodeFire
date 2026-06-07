from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from evoagent.evidence_pack import EvidenceReadiness, ExperimentEvidencePack


class ClaimReviewDecision(str, Enum):
    APPROVE = "approve"
    REVISE = "revise"
    BLOCK = "block"


@dataclass(frozen=True)
class ClaimReviewRequest:
    claim_id: str
    candidate_id: str
    evidence_pack: ExperimentEvidencePack
    open_objection_count: int = 0
    blocking_objection_count: int = 0
    assigned_reviewer_count: int = 1


@dataclass(frozen=True)
class ClaimReviewItem:
    claim_id: str
    candidate_id: str
    decision: ClaimReviewDecision
    priority_score: float
    reasons: tuple[str, ...]
    checklist: tuple[str, ...]


# cf-atom: CODE-ClaimReviewQueue
class ClaimReviewQueue:
    def build(self, requests: tuple[ClaimReviewRequest, ...]) -> tuple[ClaimReviewItem, ...]:
        items = tuple(self._build_item(request) for request in requests)
        return tuple(
            sorted(
                items,
                key=lambda item: (
                    -decision_rank(item.decision),
                    -item.priority_score,
                    item.claim_id,
                    item.candidate_id,
                ),
            )
        )

    def _build_item(self, request: ClaimReviewRequest) -> ClaimReviewItem:
        validate_request(request)
        reasons = self._reasons(request)
        checklist = self._checklist(request)
        decision = self._decision(request)
        return ClaimReviewItem(
            claim_id=request.claim_id,
            candidate_id=request.candidate_id,
            decision=decision,
            priority_score=priority_score(request, decision),
            reasons=reasons,
            checklist=checklist,
        )

    def _decision(self, request: ClaimReviewRequest) -> ClaimReviewDecision:
        if request.evidence_pack.readiness == EvidenceReadiness.BLOCKED or request.blocking_objection_count > 0:
            return ClaimReviewDecision.BLOCK
        if (
            request.evidence_pack.readiness == EvidenceReadiness.REVIEW
            or request.open_objection_count > 0
            or request.assigned_reviewer_count == 0
        ):
            return ClaimReviewDecision.REVISE
        return ClaimReviewDecision.APPROVE

    def _reasons(self, request: ClaimReviewRequest) -> tuple[str, ...]:
        reasons: list[str] = []
        if request.blocking_objection_count > 0:
            reasons.append(f"blocking_objections:{request.blocking_objection_count}")
        reasons.extend(request.evidence_pack.blocking_reasons)
        if request.evidence_pack.readiness == EvidenceReadiness.REVIEW:
            reasons.append("evidence_pack_requires_review")
        if request.open_objection_count > 0:
            reasons.append(f"open_objections:{request.open_objection_count}")
        if request.assigned_reviewer_count == 0:
            reasons.append("no_assigned_reviewer")
        if not reasons:
            reasons.append("ready_for_approval")
        return tuple(reasons)

    def _checklist(self, request: ClaimReviewRequest) -> tuple[str, ...]:
        checklist = list(request.evidence_pack.mitigation_checklist)
        if request.blocking_objection_count > 0 or request.open_objection_count > 0:
            checklist.append("resolve_reviewer_objections")
        if request.assigned_reviewer_count == 0:
            checklist.append("assign_reviewer")
        return tuple(checklist)


def validate_request(request: ClaimReviewRequest) -> None:
    if not request.claim_id.strip():
        raise ValueError("claim_id must be nonempty")
    if not request.candidate_id.strip():
        raise ValueError("candidate_id must be nonempty")
    for field_name in ("open_objection_count", "blocking_objection_count", "assigned_reviewer_count"):
        value = getattr(request, field_name)
        if value < 0:
            raise ValueError(f"{field_name} must be non-negative")
    if request.blocking_objection_count > request.open_objection_count:
        raise ValueError("blocking_objection_count cannot exceed open_objection_count")


def priority_score(request: ClaimReviewRequest, decision: ClaimReviewDecision) -> float:
    base = {
        ClaimReviewDecision.BLOCK: 100.0,
        ClaimReviewDecision.REVISE: 60.0,
        ClaimReviewDecision.APPROVE: 20.0,
    }[decision]
    objection_pressure = request.open_objection_count * 4.0 + request.blocking_objection_count * 8.0
    evidence_gap = (1.0 - request.evidence_pack.completeness_score) * 20.0
    review_gap = 5.0 if request.assigned_reviewer_count == 0 else 0.0
    return round(base + objection_pressure + evidence_gap + review_gap, 4)


def decision_rank(decision: ClaimReviewDecision) -> int:
    return {
        ClaimReviewDecision.BLOCK: 3,
        ClaimReviewDecision.REVISE: 2,
        ClaimReviewDecision.APPROVE: 1,
    }[decision]
