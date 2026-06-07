from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class FrontierAction(str, Enum):
    RUN_PROBE = "run_probe"
    MITIGATE_RISK = "mitigate_risk"
    REDESIGN = "redesign"
    DEFER = "defer"


@dataclass(frozen=True)
class ResearchFrontierSignal:
    frontier_id: str
    mechanism_family: str
    target: str
    baseline_gap: float
    expected_information_gain: float
    novelty_score: float
    estimated_cost: float
    challenge_alignment: float = 1.0
    negative_result_overlap: float = 0.0
    blocker_risks: int = 0
    rationale: str = ""


@dataclass(frozen=True)
class ResearchFrontierItem:
    frontier_id: str
    mechanism_family: str
    target: str
    action: FrontierAction
    score: float
    expected_evidence_gain: str
    primary_risk: str
    reasons: tuple[str, ...]
    rationale: str


# cf-atom: CODE-ResearchFrontierMap
class ResearchFrontierMap:
    def rank(
        self,
        signals: tuple[ResearchFrontierSignal, ...],
        *,
        remaining_budget: float,
    ) -> tuple[ResearchFrontierItem, ...]:
        if remaining_budget < 0:
            raise ValueError("remaining_budget must be non-negative")
        items = tuple(frontier_item(signal, remaining_budget=remaining_budget) for signal in signals)
        return tuple(sorted(items, key=lambda item: (-item.score, item.frontier_id)))


def frontier_item(
    signal: ResearchFrontierSignal,
    *,
    remaining_budget: float,
) -> ResearchFrontierItem:
    validate_signal(signal)
    reasons: list[str] = []
    action = choose_frontier_action(signal, remaining_budget=remaining_budget, reasons=reasons)
    score = frontier_score(signal, action, remaining_budget=remaining_budget)
    return ResearchFrontierItem(
        frontier_id=signal.frontier_id,
        mechanism_family=signal.mechanism_family,
        target=signal.target,
        action=action,
        score=score,
        expected_evidence_gain=evidence_gain_label(signal.expected_information_gain),
        primary_risk=primary_risk(signal),
        reasons=tuple(reasons),
        rationale=signal.rationale,
    )


def validate_signal(signal: ResearchFrontierSignal) -> None:
    if not signal.frontier_id:
        raise ValueError("frontier_id is required")
    if not signal.mechanism_family:
        raise ValueError("mechanism_family is required")
    if not signal.target:
        raise ValueError("target is required")
    for field_name, value in (
        ("baseline_gap", signal.baseline_gap),
        ("expected_information_gain", signal.expected_information_gain),
        ("novelty_score", signal.novelty_score),
        ("estimated_cost", signal.estimated_cost),
        ("challenge_alignment", signal.challenge_alignment),
        ("negative_result_overlap", signal.negative_result_overlap),
    ):
        if value < 0:
            raise ValueError(f"{field_name} must be non-negative")
    if signal.blocker_risks < 0:
        raise ValueError("blocker_risks must be non-negative")


def choose_frontier_action(
    signal: ResearchFrontierSignal,
    *,
    remaining_budget: float,
    reasons: list[str],
) -> FrontierAction:
    if signal.blocker_risks:
        reasons.append("blocker_risks")
        return FrontierAction.MITIGATE_RISK
    if signal.estimated_cost > remaining_budget:
        reasons.append("over_budget")
        return FrontierAction.DEFER
    if signal.negative_result_overlap >= 0.7:
        reasons.append("negative_result_overlap")
        return FrontierAction.REDESIGN
    if signal.baseline_gap > 0:
        reasons.append("baseline_gap")
    if signal.novelty_score >= 0.5:
        reasons.append("novel_mechanism")
    if signal.expected_information_gain >= 0.5:
        reasons.append("high_information_gain")
    if not reasons:
        reasons.append("low_but_executable_value")
    return FrontierAction.RUN_PROBE


def frontier_score(
    signal: ResearchFrontierSignal,
    action: FrontierAction,
    *,
    remaining_budget: float,
) -> float:
    value = (
        3.0 * signal.baseline_gap
        + 2.0 * signal.expected_information_gain
        + 1.5 * signal.novelty_score
        + signal.challenge_alignment
    )
    penalties = 2.0 * signal.negative_result_overlap + 3.0 * signal.blocker_risks
    if signal.estimated_cost > remaining_budget:
        penalties += signal.estimated_cost - remaining_budget
    action_bonus = {
        FrontierAction.RUN_PROBE: 3.0,
        FrontierAction.REDESIGN: 1.0,
        FrontierAction.MITIGATE_RISK: 0.0,
        FrontierAction.DEFER: -2.0,
    }[action]
    return action_bonus + value - penalties


def evidence_gain_label(expected_information_gain: float) -> str:
    if expected_information_gain >= 0.75:
        return "high"
    if expected_information_gain >= 0.35:
        return "medium"
    return "low"


def primary_risk(signal: ResearchFrontierSignal) -> str:
    if signal.blocker_risks:
        return "open_blocker_risk"
    if signal.negative_result_overlap >= 0.7:
        return "repeated_negative_result"
    if signal.estimated_cost > 0 and signal.expected_information_gain / signal.estimated_cost < 0.25:
        return "weak_information_gain_per_cost"
    return "execution_uncertainty"
