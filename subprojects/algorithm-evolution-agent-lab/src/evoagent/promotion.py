from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class PromotionDecision(str, Enum):
    PROMOTE = "promote"
    REPLICATE = "replicate"
    MUTATE = "mutate"
    REJECT = "reject"
    HOLD = "hold"


@dataclass(frozen=True)
class CandidatePromotionInput:
    quality_delta: float
    confidence: float
    replication_count: int
    open_blockers: int = 0
    novelty_review_passed: bool = True


@dataclass(frozen=True)
class CandidatePromotionResult:
    decision: PromotionDecision
    reasons: tuple[str, ...]


# cf-atom: CODE-CandidatePromotionPolicy
class CandidatePromotionPolicy:
    def decide(
        self,
        candidate: CandidatePromotionInput,
        *,
        min_delta: float,
        min_confidence: float,
        min_replications: int,
    ) -> CandidatePromotionResult:
        if candidate.open_blockers > 0:
            return CandidatePromotionResult(PromotionDecision.HOLD, ("open_blockers",))
        if not candidate.novelty_review_passed:
            return CandidatePromotionResult(PromotionDecision.MUTATE, ("novelty_review_failed",))
        if candidate.quality_delta < 0 and candidate.confidence >= min_confidence:
            return CandidatePromotionResult(PromotionDecision.REJECT, ("confident_regression",))
        if candidate.quality_delta < min_delta:
            return CandidatePromotionResult(PromotionDecision.MUTATE, ("insufficient_improvement",))
        if candidate.confidence < min_confidence or candidate.replication_count < min_replications:
            return CandidatePromotionResult(PromotionDecision.REPLICATE, ("needs_replication",))
        return CandidatePromotionResult(PromotionDecision.PROMOTE, ("meets_delta_confidence_and_replication",))
