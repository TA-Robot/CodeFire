from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class RoleComponent:
    role: str
    component_name: str
    capabilities: tuple[str, ...]


# cf-atom: CODE-RoleRegistry
class RoleRegistry:
    def __init__(self):
        self._components: dict[str, RoleComponent] = {}

    def register(self, component: RoleComponent) -> RoleComponent:
        if component.role in self._components:
            raise ValueError(f"duplicate role: {component.role}")
        if component.role not in {"planner", "implementer", "runner", "analyst", "reviewer", "critic", "curator"}:
            raise ValueError("unsupported role")
        self._components[component.role] = component
        return component

    def get(self, role: str) -> RoleComponent:
        return self._components[role]

    def supports(self, role: str, capability: str) -> bool:
        component = self._components.get(role)
        return component is not None and capability in component.capabilities

    def all(self) -> tuple[RoleComponent, ...]:
        return tuple(self._components.values())
