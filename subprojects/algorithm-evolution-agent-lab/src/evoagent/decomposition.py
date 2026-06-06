from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import ExperimentGoal


@dataclass(frozen=True)
class ResearchSubgoal:
    name: str
    benchmark: str
    metric: str
    target_improvement: float
    budget_hours: float
    acceptable_risk: str


# cf-atom: CODE-ObjectiveDecomposer
class ObjectiveDecomposer:
    def decompose(
        self,
        goal: ExperimentGoal,
        *,
        benchmarks: tuple[str, ...],
        target_improvement: float,
        acceptable_risk: str = "medium",
    ) -> tuple[ResearchSubgoal, ...]:
        if not benchmarks:
            raise ValueError("at least one benchmark is required")
        if target_improvement <= 0.0:
            raise ValueError("target_improvement must be positive")
        per_benchmark_budget = goal.budget_hours / len(benchmarks)
        return tuple(
            ResearchSubgoal(
                name=f"{goal.objective} on {benchmark}",
                benchmark=benchmark,
                metric=goal.target_metric,
                target_improvement=target_improvement,
                budget_hours=per_benchmark_budget,
                acceptable_risk=acceptable_risk,
            )
            for benchmark in benchmarks
        )
