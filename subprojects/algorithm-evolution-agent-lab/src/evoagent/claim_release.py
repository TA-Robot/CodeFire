from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from evoagent.claim_audit import ClaimAuditTrail
from evoagent.claim_review_queue import ClaimReviewDecision, ClaimReviewItem
from evoagent.evidence_pack import EvidenceReadiness, ExperimentEvidencePack


class ClaimReleaseDecision(str, Enum):
    ALLOW = "allow"
    HOLD = "hold"
    REJECT = "reject"


@dataclass(frozen=True)
class ClaimReleaseInput:
    claim_id: str
    candidate_id: str
    evidence_pack: ExperimentEvidencePack
    review_item: ClaimReviewItem
    audit_trail: ClaimAuditTrail
    required_artifacts: tuple[str, ...] = ()
    available_artifacts: tuple[str, ...] = ()


@dataclass(frozen=True)
class ClaimReleaseGateResult:
    claim_id: str
    candidate_id: str
    decision: ClaimReleaseDecision
    blocking_reasons: tuple[str, ...]
    missing_artifacts: tuple[str, ...]
    release_checklist: tuple[str, ...]


# cf-atom: CODE-ClaimReleaseGate
class ClaimReleaseGate:
    def evaluate(self, gate_input: ClaimReleaseInput) -> ClaimReleaseGateResult:
        validate_gate_input(gate_input)
        missing = missing_artifacts(gate_input.required_artifacts, gate_input.available_artifacts)
        blockers = blocking_reasons(gate_input)
        checklist = release_checklist(gate_input, missing)

        decision = ClaimReleaseDecision.ALLOW
        if blockers:
            decision = ClaimReleaseDecision.REJECT
        elif (
            gate_input.evidence_pack.readiness == EvidenceReadiness.REVIEW
            or gate_input.review_item.decision == ClaimReviewDecision.REVISE
            or gate_input.audit_trail.terminal_status != "allowed"
            or missing
        ):
            decision = ClaimReleaseDecision.HOLD

        return ClaimReleaseGateResult(
            claim_id=gate_input.claim_id,
            candidate_id=gate_input.candidate_id,
            decision=decision,
            blocking_reasons=blockers,
            missing_artifacts=missing,
            release_checklist=checklist,
        )


def validate_gate_input(gate_input: ClaimReleaseInput) -> None:
    if not gate_input.claim_id.strip():
        raise ValueError("claim_id must be nonempty")
    if not gate_input.candidate_id.strip():
        raise ValueError("candidate_id must be nonempty")
    for label, value in (
        ("evidence_pack.candidate_id", gate_input.evidence_pack.candidate_id),
        ("review_item.claim_id", gate_input.review_item.claim_id),
        ("review_item.candidate_id", gate_input.review_item.candidate_id),
        ("audit_trail.claim_id", gate_input.audit_trail.claim_id),
        ("audit_trail.candidate_id", gate_input.audit_trail.candidate_id),
    ):
        expected = gate_input.claim_id if label.endswith("claim_id") else gate_input.candidate_id
        if value != expected:
            raise ValueError(f"{label} must match release input")


def missing_artifacts(required: tuple[str, ...], available: tuple[str, ...]) -> tuple[str, ...]:
    available_set = set(available)
    return tuple(artifact for artifact in required if artifact not in available_set)


def blocking_reasons(gate_input: ClaimReleaseInput) -> tuple[str, ...]:
    reasons: list[str] = []
    if gate_input.evidence_pack.readiness == EvidenceReadiness.BLOCKED:
        reasons.append("evidence_blocked")
    if gate_input.review_item.decision == ClaimReviewDecision.BLOCK:
        reasons.append("review_blocked")
    if gate_input.audit_trail.terminal_status == "required":
        reasons.append("audit_follow_up_required")
    return tuple(reasons)


def release_checklist(gate_input: ClaimReleaseInput, missing: tuple[str, ...]) -> tuple[str, ...]:
    checklist: list[str] = []
    checklist.extend(gate_input.evidence_pack.mitigation_checklist)
    checklist.extend(gate_input.review_item.checklist)
    checklist.extend(f"attach:{artifact}" for artifact in missing)
    if gate_input.audit_trail.terminal_status != "allowed":
        checklist.append("resolve_audit_trail")
    return tuple(dict.fromkeys(item for item in checklist if item))
