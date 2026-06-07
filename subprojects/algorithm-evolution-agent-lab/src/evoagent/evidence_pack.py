from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class EvidenceReadiness(str, Enum):
    READY = "ready"
    REVIEW = "review"
    BLOCKED = "blocked"


@dataclass(frozen=True)
class ExperimentEvidenceSignal:
    plan_id: str
    candidate_id: str
    improvement: float
    confidence: float
    replication_count: int
    stale_evidence_count: int = 0
    blocker_risk_count: int = 0
    missing_artifact_count: int = 0
    mitigation_notes: tuple[str, ...] = ()


@dataclass(frozen=True)
class ExperimentEvidencePack:
    plan_id: str
    candidate_id: str
    readiness: EvidenceReadiness
    completeness_score: float
    blocking_reasons: tuple[str, ...]
    review_notes: tuple[str, ...]
    mitigation_checklist: tuple[str, ...]


# cf-atom: CODE-ExperimentEvidencePackBuilder
class ExperimentEvidencePackBuilder:
    def __init__(
        self,
        *,
        minimum_improvement: float = 0.0,
        minimum_confidence: float = 0.8,
        minimum_replications: int = 2,
    ):
        if minimum_confidence < 0 or minimum_confidence > 1:
            raise ValueError("minimum_confidence must be between 0 and 1")
        if minimum_replications < 0:
            raise ValueError("minimum_replications must be non-negative")
        self._minimum_improvement = minimum_improvement
        self._minimum_confidence = minimum_confidence
        self._minimum_replications = minimum_replications

    def build(self, signal: ExperimentEvidenceSignal) -> ExperimentEvidencePack:
        validate_signal(signal)
        blocking: list[str] = []
        review: list[str] = []

        if signal.missing_artifact_count > 0:
            blocking.append(f"missing_artifacts:{signal.missing_artifact_count}")
        if signal.blocker_risk_count > 0:
            blocking.append(f"blocker_risks:{signal.blocker_risk_count}")
        if signal.improvement < self._minimum_improvement:
            blocking.append("insufficient_improvement")

        if signal.confidence < self._minimum_confidence:
            review.append("low_confidence")
        if signal.replication_count < self._minimum_replications:
            review.append("insufficient_replication")
        if signal.stale_evidence_count > 0:
            review.append(f"stale_evidence:{signal.stale_evidence_count}")

        readiness = EvidenceReadiness.READY
        if blocking:
            readiness = EvidenceReadiness.BLOCKED
        elif review:
            readiness = EvidenceReadiness.REVIEW

        return ExperimentEvidencePack(
            plan_id=signal.plan_id,
            candidate_id=signal.candidate_id,
            readiness=readiness,
            completeness_score=completeness_score(signal, blocking, review),
            blocking_reasons=tuple(blocking),
            review_notes=tuple(review),
            mitigation_checklist=tuple(note for note in signal.mitigation_notes if note),
        )


def validate_signal(signal: ExperimentEvidenceSignal) -> None:
    if not signal.plan_id:
        raise ValueError("plan_id is required")
    if not signal.candidate_id:
        raise ValueError("candidate_id is required")
    if signal.confidence < 0 or signal.confidence > 1:
        raise ValueError("confidence must be between 0 and 1")
    if signal.replication_count < 0:
        raise ValueError("replication_count must be non-negative")
    if signal.stale_evidence_count < 0:
        raise ValueError("stale_evidence_count must be non-negative")
    if signal.blocker_risk_count < 0:
        raise ValueError("blocker_risk_count must be non-negative")
    if signal.missing_artifact_count < 0:
        raise ValueError("missing_artifact_count must be non-negative")


def completeness_score(
    signal: ExperimentEvidenceSignal,
    blocking: list[str],
    review: list[str],
) -> float:
    penalty = 0.0
    penalty += 0.3 * len(blocking)
    penalty += 0.1 * len(review)
    penalty += max(0.0, 1.0 - signal.confidence) * 0.2
    penalty += min(signal.stale_evidence_count, 5) * 0.03
    penalty += min(signal.missing_artifact_count, 5) * 0.05
    return max(0.0, round(1.0 - penalty, 3))
