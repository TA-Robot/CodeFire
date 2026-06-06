from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import CandidateAlgorithm


@dataclass(frozen=True)
class RegressionFinding:
    candidate_name: str
    reference_name: str
    benchmark: str
    metric: str
    observed_gap: float
    tolerance: float


# cf-atom: CODE-RegressionDetector
class RegressionDetector:
    def detect(
        self,
        *,
        candidate: CandidateAlgorithm,
        references: list[CandidateAlgorithm],
        tolerance: float = 0.0,
        maximize: bool = True,
    ) -> list[RegressionFinding]:
        if candidate.result is None:
            return []
        findings: list[RegressionFinding] = []
        for reference in references:
            if reference.result is None or not comparable(candidate, reference):
                continue
            gap = candidate.result.metric_value - reference.result.metric_value
            normalized_gap = gap if maximize else -gap
            if normalized_gap < -abs(tolerance):
                findings.append(
                    RegressionFinding(
                        candidate_name=candidate.name,
                        reference_name=reference.name,
                        benchmark=candidate.plan.benchmark,
                        metric=candidate.plan.metric,
                        observed_gap=normalized_gap,
                        tolerance=abs(tolerance),
                    )
                )
        return findings


def comparable(candidate: CandidateAlgorithm, reference: CandidateAlgorithm) -> bool:
    assert candidate.result is not None
    assert reference.result is not None
    return (
        candidate.plan.benchmark == reference.plan.benchmark
        and candidate.plan.metric == reference.plan.metric
        and candidate.result.plan.benchmark == reference.result.plan.benchmark
        and candidate.result.plan.metric == reference.result.plan.metric
    )
