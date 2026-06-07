from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from evoagent.claim_review_queue import ClaimReviewDecision, ClaimReviewItem
from evoagent.evidence_pack import EvidenceReadiness, ExperimentEvidencePack


class ClaimAuditStage(str, Enum):
    EVIDENCE = "evidence"
    REVIEW = "review"
    EXTERNAL_CLAIM = "external_claim"
    FOLLOW_UP = "follow_up"


@dataclass(frozen=True)
class ClaimAuditEvent:
    event_id: str
    claim_id: str
    candidate_id: str
    stage: ClaimAuditStage
    status: str
    source_refs: tuple[str, ...]
    follow_up_actions: tuple[str, ...] = ()


@dataclass(frozen=True)
class ClaimAuditTrail:
    claim_id: str
    candidate_id: str
    events: tuple[ClaimAuditEvent, ...]
    terminal_status: str


# cf-atom: CODE-ClaimAuditTrailBuilder
class ClaimAuditTrailBuilder:
    def build(
        self,
        *,
        claim_id: str,
        evidence_pack: ExperimentEvidencePack,
        review_item: ClaimReviewItem,
        external_claim_allowed: bool,
        note_refs: tuple[str, ...] = (),
    ) -> ClaimAuditTrail:
        if claim_id != review_item.claim_id:
            raise ValueError("claim_id must match review item")
        if evidence_pack.candidate_id != review_item.candidate_id:
            raise ValueError("evidence pack candidate must match review item")

        events = [
            evidence_event(claim_id, evidence_pack),
            review_event(review_item),
            external_claim_event(
                claim_id=claim_id,
                candidate_id=review_item.candidate_id,
                review_item=review_item,
                external_claim_allowed=external_claim_allowed,
                note_refs=note_refs,
            ),
        ]
        if review_item.checklist:
            events.append(follow_up_event(claim_id, review_item))

        for event in events:
            validate_event(event)

        ordered = tuple(sorted(events, key=lambda event: (stage_rank(event.stage), event.event_id)))
        return ClaimAuditTrail(
            claim_id=claim_id,
            candidate_id=review_item.candidate_id,
            events=ordered,
            terminal_status=ordered[-1].status,
        )


def evidence_event(claim_id: str, pack: ExperimentEvidencePack) -> ClaimAuditEvent:
    return ClaimAuditEvent(
        event_id=f"{claim_id}:evidence",
        claim_id=claim_id,
        candidate_id=pack.candidate_id,
        stage=ClaimAuditStage.EVIDENCE,
        status=pack.readiness.value,
        source_refs=(pack.plan_id,),
        follow_up_actions=pack.blocking_reasons + pack.review_notes + pack.mitigation_checklist,
    )


def review_event(item: ClaimReviewItem) -> ClaimAuditEvent:
    return ClaimAuditEvent(
        event_id=f"{item.claim_id}:review",
        claim_id=item.claim_id,
        candidate_id=item.candidate_id,
        stage=ClaimAuditStage.REVIEW,
        status=item.decision.value,
        source_refs=(f"review-score:{item.priority_score:.4f}",),
        follow_up_actions=item.reasons + item.checklist,
    )


def external_claim_event(
    *,
    claim_id: str,
    candidate_id: str,
    review_item: ClaimReviewItem,
    external_claim_allowed: bool,
    note_refs: tuple[str, ...],
) -> ClaimAuditEvent:
    approved = review_item.decision == ClaimReviewDecision.APPROVE and external_claim_allowed
    status = "allowed" if approved else "not_allowed"
    follow_up = () if approved else ("resolve_review_or_evidence_blockers",)
    refs = note_refs or (f"review:{review_item.claim_id}",)
    return ClaimAuditEvent(
        event_id=f"{claim_id}:external_claim",
        claim_id=claim_id,
        candidate_id=candidate_id,
        stage=ClaimAuditStage.EXTERNAL_CLAIM,
        status=status,
        source_refs=refs,
        follow_up_actions=follow_up,
    )


def follow_up_event(claim_id: str, item: ClaimReviewItem) -> ClaimAuditEvent:
    return ClaimAuditEvent(
        event_id=f"{claim_id}:follow_up",
        claim_id=claim_id,
        candidate_id=item.candidate_id,
        stage=ClaimAuditStage.FOLLOW_UP,
        status="required",
        source_refs=(f"review:{item.claim_id}",),
        follow_up_actions=item.checklist,
    )


def validate_event(event: ClaimAuditEvent) -> None:
    if not event.event_id.strip():
        raise ValueError("event_id must be nonempty")
    if not event.claim_id.strip():
        raise ValueError("claim_id must be nonempty")
    if not event.candidate_id.strip():
        raise ValueError("candidate_id must be nonempty")
    if not event.status.strip():
        raise ValueError("status must be nonempty")
    if not event.source_refs:
        raise ValueError("source_refs must be nonempty")


def stage_rank(stage: ClaimAuditStage) -> int:
    return {
        ClaimAuditStage.EVIDENCE: 1,
        ClaimAuditStage.REVIEW: 2,
        ClaimAuditStage.EXTERNAL_CLAIM: 3,
        ClaimAuditStage.FOLLOW_UP: 4,
    }[stage]
