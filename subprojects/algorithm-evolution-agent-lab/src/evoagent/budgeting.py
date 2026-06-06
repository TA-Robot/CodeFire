from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import CandidateAlgorithm
from evoagent.scoring import score_candidate


@dataclass(frozen=True)
class BudgetPlan:
    selected: tuple[CandidateAlgorithm, ...]
    total_cost: float
    remaining_budget: float


# cf-atom: CODE-BudgetAwarePlanner
class BudgetAwarePlanner:
    def plan(self, candidates: list[CandidateAlgorithm], *, budget_hours: float) -> BudgetPlan:
        if budget_hours < 0.0:
            raise ValueError("budget_hours must be non-negative")
        selected: list[CandidateAlgorithm] = []
        total_cost = 0.0
        for candidate in sorted(candidates, key=information_per_cost, reverse=True):
            cost = max(candidate.plan.estimated_cost, 0.0)
            if total_cost + cost <= budget_hours:
                selected.append(candidate)
                total_cost += cost
        return BudgetPlan(
            selected=tuple(selected),
            total_cost=total_cost,
            remaining_budget=max(0.0, budget_hours - total_cost),
        )


def information_per_cost(candidate: CandidateAlgorithm) -> float:
    cost = max(candidate.plan.estimated_cost, 0.01)
    return score_candidate(candidate) / cost
