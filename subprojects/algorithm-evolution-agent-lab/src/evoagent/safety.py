from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class ClaimReadiness(str, Enum):
    IDEA = "idea"
    LOCAL_IMPROVEMENT = "local_improvement"
    REPRODUCED_IMPROVEMENT = "reproduced_improvement"
    BENCHMARK_CANDIDATE = "benchmark_candidate"
    PAPER_READY = "paper_ready"
    EXTERNAL_SOTA_CLAIM = "external_sota_claim"


@dataclass(frozen=True)
class ClaimEvidenceSummary:
    valid_runs: int = 0
    baseline_comparisons: int = 0
    repeated_runs: int = 0
    benchmark_protocol_satisfied: bool = False
    ablations_included: bool = False
    limitations_included: bool = False
    reproduction_bundle_ready: bool = False
    external_baseline_comparison: bool = False
    reviewer_objections_open: int = 0


# cf-atom: CODE-ClaimReadinessGate
def claim_readiness(summary: ClaimEvidenceSummary) -> ClaimReadiness:
    if summary.valid_runs <= 0 or summary.baseline_comparisons <= 0:
        return ClaimReadiness.IDEA

    readiness = ClaimReadiness.LOCAL_IMPROVEMENT
    if summary.repeated_runs >= 2:
        readiness = ClaimReadiness.REPRODUCED_IMPROVEMENT
    if readiness_at_least(readiness, ClaimReadiness.REPRODUCED_IMPROVEMENT) and summary.benchmark_protocol_satisfied:
        readiness = ClaimReadiness.BENCHMARK_CANDIDATE
    if (
        readiness_at_least(readiness, ClaimReadiness.BENCHMARK_CANDIDATE)
        and summary.ablations_included
        and summary.limitations_included
        and summary.reproduction_bundle_ready
    ):
        readiness = ClaimReadiness.PAPER_READY
    if (
        readiness == ClaimReadiness.PAPER_READY
        and summary.external_baseline_comparison
        and summary.reviewer_objections_open == 0
        and can_claim_sota(
            benchmark_results=summary.valid_runs,
            baseline_comparisons=summary.baseline_comparisons,
            reproducible=summary.repeated_runs >= 2,
        )
    ):
        readiness = ClaimReadiness.EXTERNAL_SOTA_CLAIM

    if summary.reviewer_objections_open > 0 and readiness_at_least(readiness, ClaimReadiness.PAPER_READY):
        return ClaimReadiness.BENCHMARK_CANDIDATE
    return readiness


# cf-atom: CODE-SotaClaimGate
def can_claim_sota(*, benchmark_results: int, baseline_comparisons: int, reproducible: bool) -> bool:
    """Return whether evidence is sufficient to allow a SOTA-ready label."""
    return reproducible and benchmark_results > 0 and baseline_comparisons > 0


def readiness_at_least(actual: ClaimReadiness, threshold: ClaimReadiness) -> bool:
    return readiness_rank(actual) >= readiness_rank(threshold)


def readiness_rank(readiness: ClaimReadiness) -> int:
    order = (
        ClaimReadiness.IDEA,
        ClaimReadiness.LOCAL_IMPROVEMENT,
        ClaimReadiness.REPRODUCED_IMPROVEMENT,
        ClaimReadiness.BENCHMARK_CANDIDATE,
        ClaimReadiness.PAPER_READY,
        ClaimReadiness.EXTERNAL_SOTA_CLAIM,
    )
    return order.index(readiness)
