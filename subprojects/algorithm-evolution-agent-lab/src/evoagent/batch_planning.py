from __future__ import annotations

from dataclasses import dataclass

from evoagent.queueing import QueuedExperiment


@dataclass(frozen=True)
class ExperimentBatch:
    items: tuple[QueuedExperiment, ...]
    total_estimated_cost: float
    deferred_ids: tuple[str, ...]


# cf-atom: CODE-BudgetedExperimentBatchPlanner
class BudgetedExperimentBatchPlanner:
    def plan(
        self,
        ready_items: tuple[QueuedExperiment, ...],
        *,
        max_total_cost: float,
        max_items: int,
        max_per_benchmark: int = 1,
    ) -> ExperimentBatch:
        if max_total_cost < 0:
            raise ValueError("max_total_cost must be non-negative")
        if max_items < 0:
            raise ValueError("max_items must be non-negative")
        if max_per_benchmark <= 0:
            raise ValueError("max_per_benchmark must be positive")

        selected: list[QueuedExperiment] = []
        deferred: list[str] = []
        cost = 0.0
        by_benchmark: dict[str, int] = {}
        for item in sorted(ready_items, key=lambda candidate: (-candidate.scheduling_score, candidate.uid)):
            benchmark_count = by_benchmark.get(item.plan.benchmark, 0)
            would_fit = (
                len(selected) < max_items
                and cost + item.estimated_cost <= max_total_cost
                and benchmark_count < max_per_benchmark
            )
            if would_fit:
                selected.append(item)
                cost += item.estimated_cost
                by_benchmark[item.plan.benchmark] = benchmark_count + 1
            else:
                deferred.append(item.uid)

        return ExperimentBatch(
            items=tuple(selected),
            total_estimated_cost=cost,
            deferred_ids=tuple(deferred),
        )
