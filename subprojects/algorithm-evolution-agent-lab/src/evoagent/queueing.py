from __future__ import annotations

from dataclasses import dataclass, replace

from evoagent.models import ExperimentPlan


@dataclass(frozen=True)
class QueuedExperiment:
    plan: ExperimentPlan
    priority: float
    expected_information_gain: float
    estimated_cost: float
    dependency_ids: tuple[str, ...] = ()
    reason: str = ""

    @property
    def uid(self) -> str:
        return self.plan.uid

    @property
    def scheduling_score(self) -> float:
        return self.priority + self.expected_information_gain / max(self.estimated_cost, 1e-9)


# cf-atom: CODE-ExperimentQueue
class ExperimentQueue:
    def __init__(self):
        self._items: dict[str, QueuedExperiment] = {}

    def add(
        self,
        plan: ExperimentPlan,
        *,
        priority: float,
        expected_information_gain: float,
        estimated_cost: float | None = None,
        dependency_ids: tuple[str, ...] = (),
        reason: str = "",
    ) -> QueuedExperiment:
        if priority < 0:
            raise ValueError("priority must be non-negative")
        if expected_information_gain < 0:
            raise ValueError("expected_information_gain must be non-negative")
        cost = plan.estimated_cost if estimated_cost is None else estimated_cost
        if cost < 0:
            raise ValueError("estimated_cost must be non-negative")
        item = QueuedExperiment(
            plan=plan,
            priority=priority,
            expected_information_gain=expected_information_gain,
            estimated_cost=cost,
            dependency_ids=dependency_ids,
            reason=reason,
        )
        self._items[item.uid] = item
        return item

    def update_priority(self, plan_id: str, priority: float) -> QueuedExperiment:
        if priority < 0:
            raise ValueError("priority must be non-negative")
        item = self._items[plan_id]
        updated = replace(item, priority=priority)
        self._items[plan_id] = updated
        return updated

    def ready(
        self,
        *,
        completed_plan_ids: set[str],
        remaining_cost_budget: float | None = None,
    ) -> tuple[QueuedExperiment, ...]:
        items = [
            item
            for item in self._items.values()
            if all(dependency_id in completed_plan_ids for dependency_id in item.dependency_ids)
            and (remaining_cost_budget is None or item.estimated_cost <= remaining_cost_budget)
        ]
        return tuple(sorted(items, key=lambda item: (-item.scheduling_score, item.uid)))

    def pop_next(
        self,
        *,
        completed_plan_ids: set[str],
        remaining_cost_budget: float | None = None,
    ) -> QueuedExperiment | None:
        ready_items = self.ready(
            completed_plan_ids=completed_plan_ids,
            remaining_cost_budget=remaining_cost_budget,
        )
        if not ready_items:
            return None
        selected = ready_items[0]
        del self._items[selected.uid]
        return selected

    def blockers(self, plan_id: str, completed_plan_ids: set[str]) -> tuple[str, ...]:
        item = self._items[plan_id]
        return tuple(
            dependency_id
            for dependency_id in item.dependency_ids
            if dependency_id not in completed_plan_ids
        )

    @property
    def items(self) -> tuple[QueuedExperiment, ...]:
        return tuple(sorted(self._items.values(), key=lambda item: item.uid))

    @property
    def total_estimated_cost(self) -> float:
        return sum(item.estimated_cost for item in self._items.values())
