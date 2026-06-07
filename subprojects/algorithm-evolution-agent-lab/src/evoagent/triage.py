from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class TriageAction(str, Enum):
    RUN = "run"
    REPLICATE = "replicate"
    MUTATE = "mutate"
    PROMOTE = "promote"
    REJECT = "reject"
    HOLD = "hold"


@dataclass(frozen=True)
class ExperimentTriageSignal:
    plan_id: str
    priority: float
    expected_information_gain: float
    estimated_cost: float
    promotion_decision: str = "run"
    learning_trend: str = "unknown"
    open_blockers: int = 0
    rationale: str = ""


@dataclass(frozen=True)
class ExperimentTriageItem:
    plan_id: str
    action: TriageAction
    score: float
    reasons: tuple[str, ...]
    rationale: str


# cf-atom: CODE-ExperimentTriageBoard
class ExperimentTriageBoard:
    def rank(
        self,
        signals: tuple[ExperimentTriageSignal, ...],
        *,
        remaining_budget: float,
    ) -> tuple[ExperimentTriageItem, ...]:
        if remaining_budget < 0:
            raise ValueError("remaining_budget must be non-negative")
        items = tuple(triage_item(signal, remaining_budget=remaining_budget) for signal in signals)
        return tuple(sorted(items, key=lambda item: (-item.score, item.plan_id)))


def triage_item(signal: ExperimentTriageSignal, *, remaining_budget: float) -> ExperimentTriageItem:
    validate_signal(signal)
    reasons: list[str] = []
    action = choose_action(signal, remaining_budget=remaining_budget, reasons=reasons)
    score = triage_score(signal, action)
    return ExperimentTriageItem(
        plan_id=signal.plan_id,
        action=action,
        score=score,
        reasons=tuple(reasons),
        rationale=signal.rationale,
    )


def validate_signal(signal: ExperimentTriageSignal) -> None:
    if not signal.plan_id:
        raise ValueError("plan_id is required")
    if signal.priority < 0:
        raise ValueError("priority must be non-negative")
    if signal.expected_information_gain < 0:
        raise ValueError("expected_information_gain must be non-negative")
    if signal.estimated_cost < 0:
        raise ValueError("estimated_cost must be non-negative")
    if signal.open_blockers < 0:
        raise ValueError("open_blockers must be non-negative")


def choose_action(
    signal: ExperimentTriageSignal,
    *,
    remaining_budget: float,
    reasons: list[str],
) -> TriageAction:
    if signal.open_blockers:
        reasons.append("open_blockers")
        return TriageAction.HOLD
    if signal.estimated_cost > remaining_budget:
        reasons.append("over_budget")
        return TriageAction.HOLD

    decision = signal.promotion_decision
    trend = signal.learning_trend
    if decision == "reject":
        reasons.append("promotion_reject")
        return TriageAction.REJECT
    if decision == "promote" and trend not in {"unstable", "regressing"}:
        reasons.append("promotion_ready")
        return TriageAction.PROMOTE
    if decision == "replicate" or trend == "unstable":
        reasons.append("needs_replication")
        return TriageAction.REPLICATE
    if decision == "mutate" or trend in {"plateau", "regressing"}:
        reasons.append("needs_mutation")
        return TriageAction.MUTATE

    reasons.append("ready_value")
    return TriageAction.RUN


def triage_score(signal: ExperimentTriageSignal, action: TriageAction) -> float:
    if action == TriageAction.HOLD:
        return -10.0 - signal.open_blockers - signal.estimated_cost
    value = signal.priority + signal.expected_information_gain / max(signal.estimated_cost, 0.01)
    action_bonus = {
        TriageAction.PROMOTE: 4.0,
        TriageAction.RUN: 3.0,
        TriageAction.REPLICATE: 2.0,
        TriageAction.MUTATE: 1.0,
        TriageAction.REJECT: 0.5,
        TriageAction.HOLD: -10.0,
    }[action]
    return action_bonus + value
