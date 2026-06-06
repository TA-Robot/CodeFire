from __future__ import annotations

from dataclasses import dataclass

from evoagent.challenge import ComputeBudget
from evoagent.models import ExperimentRun


@dataclass(frozen=True)
class ComputeAccount:
    total_runs: int
    successful_runs: int
    failed_runs: int
    total_hours: float
    max_hours: float
    max_trials: int

    @property
    def remaining_hours(self) -> float:
        return max(0.0, self.max_hours - self.total_hours)

    @property
    def remaining_trials(self) -> int:
        return max(0, self.max_trials - self.total_runs)

    @property
    def over_budget(self) -> bool:
        return self.total_hours > self.max_hours or self.total_runs > self.max_trials


# cf-atom: CODE-ComputeAccountant
class ComputeAccountant:
    def summarize(self, runs: list[ExperimentRun], budget: ComputeBudget) -> ComputeAccount:
        total_hours = sum(max(run.duration_seconds, 0.0) for run in runs) / 3600.0
        successful_runs = sum(1 for run in runs if run.succeeded)
        return ComputeAccount(
            total_runs=len(runs),
            successful_runs=successful_runs,
            failed_runs=len(runs) - successful_runs,
            total_hours=total_hours,
            max_hours=budget.max_hours,
            max_trials=budget.max_trials,
        )
