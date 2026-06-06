from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import ExperimentResult
from evoagent.program import ResearchProgram


@dataclass(frozen=True)
class ResearchProgressReport:
    reproducible_runs: int
    improved_results: int
    hypothesis_count: int
    decision_count: int
    evidence_count: int
    uncertainty_reduction: float
    progress_score: float
    bottlenecks: tuple[str, ...]


# cf-atom: CODE-ResearchProgressMeter
class ResearchProgressMeter:
    def evaluate(
        self,
        program: ResearchProgram,
        *,
        results: tuple[ExperimentResult, ...] = (),
        previous_uncertainty: float = 1.0,
        current_uncertainty: float = 1.0,
    ) -> ResearchProgressReport:
        if previous_uncertainty < 0 or current_uncertainty < 0:
            raise ValueError("uncertainty values must be non-negative")

        reproducible_runs = sum(1 for run in program.runs if run.succeeded and run.artifacts)
        improved_results = sum(1 for result in results if result.quality_delta > 0 and result.confidence >= 0.5)
        evidence_count = len(program.evidence.all())
        uncertainty_reduction = max(0.0, previous_uncertainty - current_uncertainty)
        bottlenecks = bottlenecks_for(
            reproducible_runs=reproducible_runs,
            improved_results=improved_results,
            evidence_count=evidence_count,
            decision_count=len(program.decisions),
        )
        progress_score = (
            reproducible_runs * 0.25
            + improved_results * 0.35
            + len(program.hypotheses) * 0.05
            + evidence_count * 0.15
            + len(program.decisions) * 0.1
            + uncertainty_reduction * 0.1
        )
        return ResearchProgressReport(
            reproducible_runs=reproducible_runs,
            improved_results=improved_results,
            hypothesis_count=len(program.hypotheses),
            decision_count=len(program.decisions),
            evidence_count=evidence_count,
            uncertainty_reduction=uncertainty_reduction,
            progress_score=progress_score,
            bottlenecks=bottlenecks,
        )


def bottlenecks_for(
    *,
    reproducible_runs: int,
    improved_results: int,
    evidence_count: int,
    decision_count: int,
) -> tuple[str, ...]:
    bottlenecks: list[str] = []
    if reproducible_runs == 0:
        bottlenecks.append("no_reproducible_runs")
    if improved_results == 0:
        bottlenecks.append("no_improved_results")
    if evidence_count == 0:
        bottlenecks.append("no_evidence")
    if decision_count == 0:
        bottlenecks.append("no_decisions")
    return tuple(bottlenecks)
