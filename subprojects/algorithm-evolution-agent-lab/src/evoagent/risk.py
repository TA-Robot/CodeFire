from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class RiskSeverity(str, Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


class RiskStatus(str, Enum):
    OPEN = "open"
    MITIGATING = "mitigating"
    ACCEPTED = "accepted"
    CLOSED = "closed"


@dataclass(frozen=True)
class ExperimentRisk:
    risk_id: str
    plan_id: str
    category: str
    severity: RiskSeverity
    probability: float
    impact: float
    status: RiskStatus = RiskStatus.OPEN
    mitigation: str = ""


@dataclass(frozen=True)
class RiskPriority:
    risk_id: str
    plan_id: str
    category: str
    severity: RiskSeverity
    status: RiskStatus
    score: float
    mitigation: str


# cf-atom: CODE-ExperimentRiskRegister
class ExperimentRiskRegister:
    def prioritize(self, risks: tuple[ExperimentRisk, ...]) -> tuple[RiskPriority, ...]:
        priorities = tuple(risk_priority(risk) for risk in risks if risk.status in active_statuses())
        return tuple(sorted(priorities, key=lambda item: (-item.score, item.risk_id)))

    def blockers(self, risks: tuple[ExperimentRisk, ...]) -> tuple[RiskPriority, ...]:
        return tuple(
            item
            for item in self.prioritize(risks)
            if item.severity in {RiskSeverity.CRITICAL, RiskSeverity.HIGH}
        )

    def mitigation_checklist(self, risks: tuple[ExperimentRisk, ...]) -> tuple[str, ...]:
        return tuple(item.mitigation for item in self.prioritize(risks) if item.mitigation)


def active_statuses() -> set[RiskStatus]:
    return {RiskStatus.OPEN, RiskStatus.MITIGATING}


def risk_priority(risk: ExperimentRisk) -> RiskPriority:
    validate_risk(risk)
    severity_weight = {
        RiskSeverity.LOW: 1.0,
        RiskSeverity.MEDIUM: 2.0,
        RiskSeverity.HIGH: 4.0,
        RiskSeverity.CRITICAL: 8.0,
    }[risk.severity]
    status_weight = 1.25 if risk.status == RiskStatus.OPEN else 1.0
    score = severity_weight * risk.probability * risk.impact * status_weight
    return RiskPriority(
        risk_id=risk.risk_id,
        plan_id=risk.plan_id,
        category=risk.category,
        severity=risk.severity,
        status=risk.status,
        score=score,
        mitigation=risk.mitigation,
    )


def validate_risk(risk: ExperimentRisk) -> None:
    if not risk.risk_id:
        raise ValueError("risk_id is required")
    if not risk.plan_id:
        raise ValueError("plan_id is required")
    if not risk.category:
        raise ValueError("category is required")
    if not 0.0 <= risk.probability <= 1.0:
        raise ValueError("probability must be between 0 and 1")
    if risk.impact < 0:
        raise ValueError("impact must be non-negative")
