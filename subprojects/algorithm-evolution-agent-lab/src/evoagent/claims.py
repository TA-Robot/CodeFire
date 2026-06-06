from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class ClaimType(str, Enum):
    INTERNAL_SCORE = "internal_score"
    BENCHMARK_IMPROVEMENT = "benchmark_improvement"
    LEADERBOARD_IMPROVEMENT = "leaderboard_improvement"
    EXTERNAL_SOTA = "external_sota"


@dataclass(frozen=True)
class ClaimDisciplineResult:
    claim_type: ClaimType
    allowed: bool
    required_evidence: tuple[str, ...]
    missing_evidence: tuple[str, ...]


# cf-atom: CODE-ClaimDisciplineClassifier
class ClaimDisciplineClassifier:
    def classify(self, claim_type: ClaimType, evidence: tuple[str, ...]) -> ClaimDisciplineResult:
        required = required_evidence_for(claim_type)
        evidence_set = set(evidence)
        missing = tuple(item for item in required if item not in evidence_set)
        return ClaimDisciplineResult(
            claim_type=claim_type,
            allowed=not missing,
            required_evidence=required,
            missing_evidence=missing,
        )


def required_evidence_for(claim_type: ClaimType) -> tuple[str, ...]:
    if claim_type == ClaimType.INTERNAL_SCORE:
        return ("score_formula",)
    if claim_type == ClaimType.BENCHMARK_IMPROVEMENT:
        return ("benchmark_definition", "baseline_comparison", "valid_run")
    if claim_type == ClaimType.LEADERBOARD_IMPROVEMENT:
        return ("benchmark_definition", "external_baseline", "submission_record", "anti_overfitting_check")
    if claim_type == ClaimType.EXTERNAL_SOTA:
        return (
            "benchmark_definition",
            "external_baseline",
            "reproduction_bundle",
            "statistical_confidence",
            "reviewer_clearance",
            "limitations",
        )
    raise ValueError(f"unsupported claim type: {claim_type}")
