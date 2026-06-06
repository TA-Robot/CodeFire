from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import ExperimentPlan


@dataclass(frozen=True)
class ExperimentDependency:
    prerequisite_id: str
    dependent_id: str
    relation: str


# cf-atom: CODE-ExperimentDependencyGraph
class ExperimentDependencyGraph:
    def __init__(self):
        self._plans: dict[str, ExperimentPlan] = {}
        self._dependencies: list[ExperimentDependency] = []

    def add_plan(self, plan: ExperimentPlan) -> ExperimentPlan:
        self._plans[plan.uid] = plan
        return plan

    def add_dependency(
        self,
        *,
        prerequisite: ExperimentPlan,
        dependent: ExperimentPlan,
        relation: str,
    ) -> ExperimentDependency:
        if relation not in {"requires", "replicates", "ablates", "unlocks"}:
            raise ValueError("unsupported dependency relation")
        self.add_plan(prerequisite)
        self.add_plan(dependent)
        dependency = ExperimentDependency(prerequisite.uid, dependent.uid, relation)
        if has_path(self._dependencies + [dependency], start=dependent.uid, target=prerequisite.uid):
            raise ValueError("dependency would create a cycle")
        self._dependencies.append(dependency)
        return dependency

    def ready_plans(self, completed_plan_ids: set[str]) -> list[ExperimentPlan]:
        return [
            plan
            for plan_id, plan in self._plans.items()
            if plan_id not in completed_plan_ids and not self.blockers(plan, completed_plan_ids)
        ]

    def blockers(self, plan: ExperimentPlan, completed_plan_ids: set[str]) -> tuple[str, ...]:
        return tuple(
            dependency.prerequisite_id
            for dependency in self._dependencies
            if dependency.dependent_id == plan.uid and dependency.prerequisite_id not in completed_plan_ids
        )

    @property
    def dependencies(self) -> tuple[ExperimentDependency, ...]:
        return tuple(self._dependencies)


def has_path(dependencies: list[ExperimentDependency], *, start: str, target: str) -> bool:
    stack = [start]
    seen: set[str] = set()
    while stack:
        current = stack.pop()
        if current == target:
            return True
        if current in seen:
            continue
        seen.add(current)
        stack.extend(
            dependency.dependent_id
            for dependency in dependencies
            if dependency.prerequisite_id == current
        )
    return False
