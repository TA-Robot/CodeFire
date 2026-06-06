from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class ProductSurface(str, Enum):
    EXPERIMENT_AUTOMATION = "experiment_automation"
    MODEL_IDEA = "model_idea"
    BENCHMARK_RUNNER = "benchmark_runner"
    REPORTING = "reporting"
    GOVERNANCE = "governance"


@dataclass(frozen=True)
class ProductWorkItem:
    item_id: str
    title: str
    surface: ProductSurface
    has_experiment_plan: bool
    expected_research_impact: float
    effort: float


@dataclass(frozen=True)
class SurfacePriorityFinding:
    kind: str
    item_id: str
    detail: str


# cf-atom: CODE-ExperimentAutomationSurfaceGuard
class ExperimentAutomationSurfaceGuard:
    def prioritize(self, items: tuple[ProductWorkItem, ...]) -> tuple[ProductWorkItem, ...]:
        return tuple(sorted(items, key=lambda item: (-surface_score(item), item.item_id)))

    def findings(self, items: tuple[ProductWorkItem, ...]) -> tuple[SurfacePriorityFinding, ...]:
        findings: list[SurfacePriorityFinding] = []
        for item in items:
            if item.surface != ProductSurface.EXPERIMENT_AUTOMATION and not item.has_experiment_plan:
                findings.append(
                    SurfacePriorityFinding(
                        "missing_experiment_plan",
                        item.item_id,
                        "non-automation work must be tied to an experiment plan",
                    )
                )
            if item.effort < 0:
                findings.append(SurfacePriorityFinding("negative_effort", item.item_id, "effort must be non-negative"))
            if item.expected_research_impact < 0:
                findings.append(
                    SurfacePriorityFinding("negative_impact", item.item_id, "impact must be non-negative")
                )
        return tuple(findings)


def surface_score(item: ProductWorkItem) -> float:
    automation_bonus = 10.0 if item.surface == ProductSurface.EXPERIMENT_AUTOMATION else 0.0
    plan_bonus = 1.0 if item.has_experiment_plan else 0.0
    return automation_bonus + plan_bonus + item.expected_research_impact / max(item.effort, 1e-9)
