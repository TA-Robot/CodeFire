from __future__ import annotations

from dataclasses import dataclass

from evoagent.triage import ExperimentTriageBoard, ExperimentTriageSignal, TriageAction


@dataclass(frozen=True)
class IterationLaneItem:
    plan_id: str
    action: TriageAction
    score: float
    estimated_cost: float
    reasons: tuple[str, ...]


@dataclass(frozen=True)
class ExperimentIterationPlan:
    active: tuple[IterationLaneItem, ...]
    review: tuple[IterationLaneItem, ...]
    deferred: tuple[IterationLaneItem, ...]
    budget_used: float

    @property
    def active_plan_ids(self) -> tuple[str, ...]:
        return tuple(item.plan_id for item in self.active)

    @property
    def review_plan_ids(self) -> tuple[str, ...]:
        return tuple(item.plan_id for item in self.review)

    @property
    def deferred_plan_ids(self) -> tuple[str, ...]:
        return tuple(item.plan_id for item in self.deferred)


# cf-atom: CODE-ExperimentIterationPlanner
class ExperimentIterationPlanner:
    def __init__(self, triage_board: ExperimentTriageBoard | None = None):
        self._triage_board = triage_board or ExperimentTriageBoard()

    def plan(
        self,
        signals: tuple[ExperimentTriageSignal, ...],
        *,
        remaining_budget: float,
        max_active: int,
        review_capacity: int,
    ) -> ExperimentIterationPlan:
        if remaining_budget < 0:
            raise ValueError("remaining_budget must be non-negative")
        if max_active <= 0:
            raise ValueError("max_active must be positive")
        if review_capacity < 0:
            raise ValueError("review_capacity must be non-negative")

        costs_by_plan = {signal.plan_id: signal.estimated_cost for signal in signals}
        ranked = self._triage_board.rank(signals, remaining_budget=remaining_budget)
        active: list[IterationLaneItem] = []
        review: list[IterationLaneItem] = []
        deferred: list[IterationLaneItem] = []
        budget_used = 0.0

        for item in ranked:
            cost = costs_by_plan[item.plan_id]
            lane_item = IterationLaneItem(
                plan_id=item.plan_id,
                action=item.action,
                score=item.score,
                estimated_cost=cost,
                reasons=item.reasons,
            )
            if item.action in {TriageAction.PROMOTE, TriageAction.REJECT}:
                if len(review) < review_capacity:
                    review.append(lane_item)
                else:
                    deferred.append(lane_item)
                continue
            if item.action in {TriageAction.RUN, TriageAction.REPLICATE, TriageAction.MUTATE}:
                if len(active) < max_active and budget_used + cost <= remaining_budget:
                    active.append(lane_item)
                    budget_used += cost
                else:
                    deferred.append(lane_item)
                continue
            deferred.append(lane_item)

        return ExperimentIterationPlan(
            active=tuple(active),
            review=tuple(review),
            deferred=tuple(deferred),
            budget_used=budget_used,
        )
