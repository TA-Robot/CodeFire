from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from evoagent.models import ExperimentPlan, Hypothesis


class FrontierAction(str, Enum):
    RUN_PROBE = "run_probe"
    MITIGATE_RISK = "mitigate_risk"
    REDESIGN = "redesign"
    DEFER = "defer"


class FrontierFeedbackOutcome(str, Enum):
    IMPROVED = "improved"
    REPLICATED = "replicated"
    REGRESSED = "regressed"
    FAILED = "failed"
    BLOCKED = "blocked"
    INCONCLUSIVE = "inconclusive"


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
class FrontierFeedback:
    frontier_id: str
    outcome: FrontierFeedbackOutcome
    metric_delta: float = 0.0
    confidence: float = 0.0
    observed_cost: float = 0.0
    blocker_count: int = 0
    notes: str = ""


@dataclass(frozen=True)
class FrontierFeedbackUpdate:
    frontier_id: str
    prior_signal: ResearchFrontierSignal | None
    updated_signal: ResearchFrontierSignal | None
    applied_outcome_count: int
    reasons: tuple[str, ...]


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


@dataclass(frozen=True)
class FrontierPlanDraft:
    frontier_id: str
    plan: ExperimentPlan
    analysis_criteria: tuple[str, ...]
    reasons: tuple[str, ...]


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


# cf-atom: CODE-FrontierExperimentPlanner
class FrontierExperimentPlanner:
    def build_plans(
        self,
        frontiers: tuple[ResearchFrontierItem, ...],
        *,
        baseline: str,
        metric: str,
        remaining_budget: float,
        max_plans: int,
    ) -> tuple[FrontierPlanDraft, ...]:
        if not baseline:
            raise ValueError("baseline is required")
        if not metric:
            raise ValueError("metric is required")
        if remaining_budget < 0:
            raise ValueError("remaining_budget must be non-negative")
        if max_plans < 0:
            raise ValueError("max_plans must be non-negative")

        drafts: list[FrontierPlanDraft] = []
        spent = 0.0
        for frontier in frontiers:
            if len(drafts) >= max_plans:
                break
            if frontier.action != FrontierAction.RUN_PROBE:
                continue
            estimated_cost = min(max(frontier.score / 10.0, 0.1), max(remaining_budget - spent, 0.0))
            if estimated_cost <= 0:
                break
            plan = ExperimentPlan(
                hypothesis=Hypothesis(
                    title=f"Probe {frontier.mechanism_family} for {frontier.target}",
                    rationale=frontier.rationale or "; ".join(frontier.reasons),
                    expected_gain=max(frontier.score / 100.0, 0.0),
                    novelty=1.0 if "novel_mechanism" in frontier.reasons else 0.3,
                ),
                benchmark=frontier.target,
                baseline=baseline,
                metric=metric,
                estimated_cost=estimated_cost,
                command=frontier_command(frontier),
                artifact_paths=(f"artifacts/{frontier.frontier_id}/metrics.json",),
            )
            drafts.append(
                FrontierPlanDraft(
                    frontier_id=frontier.frontier_id,
                    plan=plan,
                    analysis_criteria=(
                        "compare against baseline",
                        "record confidence and caveats",
                        f"inspect primary risk: {frontier.primary_risk}",
                    ),
                    reasons=frontier.reasons,
                )
            )
            spent += estimated_cost
        return tuple(drafts)


# cf-atom: CODE-FrontierFeedbackIntegrator
class FrontierFeedbackIntegrator:
    def integrate(
        self,
        signals: tuple[ResearchFrontierSignal, ...],
        feedback: tuple[FrontierFeedback, ...],
    ) -> tuple[FrontierFeedbackUpdate, ...]:
        by_frontier: dict[str, list[FrontierFeedback]] = {}
        for item in feedback:
            validate_feedback(item)
            by_frontier.setdefault(item.frontier_id, []).append(item)

        updates: list[FrontierFeedbackUpdate] = []
        known_ids = {signal.frontier_id for signal in signals}
        for signal in signals:
            updated = signal
            reasons: list[str] = []
            applied = 0
            for item in by_frontier.get(signal.frontier_id, ()):
                updated = apply_frontier_feedback(updated, item, reasons)
                applied += 1
            updates.append(
                FrontierFeedbackUpdate(
                    frontier_id=signal.frontier_id,
                    prior_signal=signal,
                    updated_signal=updated,
                    applied_outcome_count=applied,
                    reasons=tuple(reasons) if reasons else ("no_feedback",),
                )
            )

        for item in feedback:
            if item.frontier_id not in known_ids:
                updates.append(
                    FrontierFeedbackUpdate(
                        frontier_id=item.frontier_id,
                        prior_signal=None,
                        updated_signal=None,
                        applied_outcome_count=0,
                        reasons=("unknown_frontier", item.outcome.value),
                    )
                )
        return tuple(updates)


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


def validate_feedback(feedback: FrontierFeedback) -> None:
    if not feedback.frontier_id:
        raise ValueError("frontier_id is required")
    if feedback.confidence < 0:
        raise ValueError("confidence must be non-negative")
    if feedback.observed_cost < 0:
        raise ValueError("observed_cost must be non-negative")
    if feedback.blocker_count < 0:
        raise ValueError("blocker_count must be non-negative")


def apply_frontier_feedback(
    signal: ResearchFrontierSignal,
    feedback: FrontierFeedback,
    reasons: list[str],
) -> ResearchFrontierSignal:
    confidence = clamp01(feedback.confidence)
    baseline_gap = signal.baseline_gap
    expected_information_gain = signal.expected_information_gain
    novelty_score = signal.novelty_score
    challenge_alignment = signal.challenge_alignment
    negative_result_overlap = signal.negative_result_overlap
    blocker_risks = signal.blocker_risks

    if feedback.outcome == FrontierFeedbackOutcome.IMPROVED:
        improvement = max(feedback.metric_delta, 0.0) * max(confidence, 0.25)
        baseline_gap += improvement
        expected_information_gain += 0.12 * confidence
        challenge_alignment += 0.05 * confidence
        negative_result_overlap -= 0.08 * confidence
        reasons.append("improved")
    elif feedback.outcome == FrontierFeedbackOutcome.REPLICATED:
        expected_information_gain += 0.08 * confidence
        challenge_alignment += 0.08 * confidence
        negative_result_overlap -= 0.05 * confidence
        reasons.append("replicated")
    elif feedback.outcome == FrontierFeedbackOutcome.REGRESSED:
        regression = abs(min(feedback.metric_delta, 0.0))
        baseline_gap -= 0.5 * regression
        expected_information_gain -= 0.18 * max(confidence, 0.5)
        negative_result_overlap += 0.15 + 0.5 * regression
        reasons.append("regressed")
    elif feedback.outcome == FrontierFeedbackOutcome.FAILED:
        expected_information_gain -= 0.14 * max(confidence, 0.5)
        negative_result_overlap += 0.2
        reasons.append("failed")
    elif feedback.outcome == FrontierFeedbackOutcome.BLOCKED:
        blocker_risks += max(feedback.blocker_count, 1)
        expected_information_gain -= 0.1
        reasons.append("blocked")
    elif feedback.outcome == FrontierFeedbackOutcome.INCONCLUSIVE:
        expected_information_gain -= 0.05
        reasons.append("inconclusive")

    if feedback.observed_cost > signal.estimated_cost:
        overrun_penalty = (feedback.observed_cost - signal.estimated_cost) / max(feedback.observed_cost, 1.0)
        expected_information_gain -= min(0.25, overrun_penalty)
        reasons.append("over_budget")

    rationale = signal.rationale
    if feedback.notes:
        rationale = f"{rationale}; {feedback.notes}" if rationale else feedback.notes

    return ResearchFrontierSignal(
        frontier_id=signal.frontier_id,
        mechanism_family=signal.mechanism_family,
        target=signal.target,
        baseline_gap=clamp01(baseline_gap),
        expected_information_gain=clamp01(expected_information_gain),
        novelty_score=clamp01(novelty_score),
        estimated_cost=max(signal.estimated_cost, 0.0),
        challenge_alignment=clamp01(challenge_alignment),
        negative_result_overlap=clamp01(negative_result_overlap),
        blocker_risks=blocker_risks,
        rationale=rationale,
    )


def clamp01(value: float) -> float:
    return min(max(value, 0.0), 1.0)


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


def frontier_command(frontier: ResearchFrontierItem) -> str:
    mechanism = frontier.mechanism_family.replace(" ", "_")
    target = frontier.target.replace(" ", "_")
    return (
        "python -m evoagent.fixture_runner "
        f"--mechanism {mechanism} "
        f"--benchmark {target} "
        f"--output artifacts/{frontier.frontier_id}/metrics.json"
    )
