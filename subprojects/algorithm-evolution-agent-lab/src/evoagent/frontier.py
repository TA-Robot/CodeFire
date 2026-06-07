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


class FrontierDriftCategory(str, Enum):
    NEW = "new"
    REMOVED = "removed"
    RISING = "rising"
    FALLING = "falling"
    STABLE = "stable"
    ACTION_CHANGED = "action_changed"


class FrontierDriftSeverity(str, Enum):
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"


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


@dataclass(frozen=True)
class FrontierDriftRecord:
    frontier_id: str
    category: FrontierDriftCategory
    severity: FrontierDriftSeverity
    previous_rank: int | None
    current_rank: int | None
    rank_delta: int | None
    previous_score: float | None
    current_score: float | None
    score_delta: float | None
    previous_action: FrontierAction | None
    current_action: FrontierAction | None
    recommendation: str
    reasons: tuple[str, ...]


@dataclass(frozen=True)
class FrontierDriftReport:
    records: tuple[FrontierDriftRecord, ...]
    high_severity_count: int
    medium_severity_count: int
    low_severity_count: int
    summary: str


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


# cf-atom: CODE-FrontierDriftReporter
class FrontierDriftReporter:
    def report(
        self,
        previous: tuple[ResearchFrontierItem, ...],
        current: tuple[ResearchFrontierItem, ...],
        *,
        significant_rank_delta: int = 2,
        significant_score_delta: float = 1.0,
    ) -> FrontierDriftReport:
        if significant_rank_delta < 0:
            raise ValueError("significant_rank_delta must be non-negative")
        if significant_score_delta < 0:
            raise ValueError("significant_score_delta must be non-negative")

        previous_by_id = {item.frontier_id: item for item in previous}
        current_by_id = {item.frontier_id: item for item in current}
        previous_rank = {item.frontier_id: index + 1 for index, item in enumerate(previous)}
        current_rank = {item.frontier_id: index + 1 for index, item in enumerate(current)}

        records = tuple(
            drift_record(
                frontier_id,
                previous_by_id.get(frontier_id),
                current_by_id.get(frontier_id),
                previous_rank.get(frontier_id),
                current_rank.get(frontier_id),
                significant_rank_delta=significant_rank_delta,
                significant_score_delta=significant_score_delta,
            )
            for frontier_id in sorted(set(previous_by_id) | set(current_by_id))
        )
        ordered = tuple(sorted(records, key=drift_sort_key))
        high = sum(1 for record in ordered if record.severity == FrontierDriftSeverity.HIGH)
        medium = sum(1 for record in ordered if record.severity == FrontierDriftSeverity.MEDIUM)
        low = sum(1 for record in ordered if record.severity == FrontierDriftSeverity.LOW)
        return FrontierDriftReport(
            records=ordered,
            high_severity_count=high,
            medium_severity_count=medium,
            low_severity_count=low,
            summary=drift_summary(high, medium, low),
        )


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


def drift_record(
    frontier_id: str,
    previous: ResearchFrontierItem | None,
    current: ResearchFrontierItem | None,
    previous_rank: int | None,
    current_rank: int | None,
    *,
    significant_rank_delta: int,
    significant_score_delta: float,
) -> FrontierDriftRecord:
    previous_score = previous.score if previous is not None else None
    current_score = current.score if current is not None else None
    score_delta = None
    if previous_score is not None and current_score is not None:
        score_delta = current_score - previous_score
    rank_delta = None
    if previous_rank is not None and current_rank is not None:
        rank_delta = previous_rank - current_rank

    previous_action = previous.action if previous is not None else None
    current_action = current.action if current is not None else None
    category = drift_category(
        previous,
        current,
        rank_delta=rank_delta,
        score_delta=score_delta,
        significant_rank_delta=significant_rank_delta,
        significant_score_delta=significant_score_delta,
    )
    severity = drift_severity(
        category,
        previous_action=previous_action,
        current_action=current_action,
        rank_delta=rank_delta,
        score_delta=score_delta,
        significant_rank_delta=significant_rank_delta,
        significant_score_delta=significant_score_delta,
    )
    reasons = drift_reasons(
        category,
        previous_action=previous_action,
        current_action=current_action,
        rank_delta=rank_delta,
        score_delta=score_delta,
    )
    return FrontierDriftRecord(
        frontier_id=frontier_id,
        category=category,
        severity=severity,
        previous_rank=previous_rank,
        current_rank=current_rank,
        rank_delta=rank_delta,
        previous_score=previous_score,
        current_score=current_score,
        score_delta=score_delta,
        previous_action=previous_action,
        current_action=current_action,
        recommendation=drift_recommendation(category, severity, current_action),
        reasons=reasons,
    )


def drift_category(
    previous: ResearchFrontierItem | None,
    current: ResearchFrontierItem | None,
    *,
    rank_delta: int | None,
    score_delta: float | None,
    significant_rank_delta: int,
    significant_score_delta: float,
) -> FrontierDriftCategory:
    if previous is None:
        return FrontierDriftCategory.NEW
    if current is None:
        return FrontierDriftCategory.REMOVED
    if previous.action != current.action:
        return FrontierDriftCategory.ACTION_CHANGED
    if rank_delta is not None and rank_delta >= significant_rank_delta and significant_rank_delta > 0:
        return FrontierDriftCategory.RISING
    if rank_delta is not None and -rank_delta >= significant_rank_delta and significant_rank_delta > 0:
        return FrontierDriftCategory.FALLING
    if score_delta is not None and score_delta >= significant_score_delta and significant_score_delta > 0:
        return FrontierDriftCategory.RISING
    if score_delta is not None and -score_delta >= significant_score_delta and significant_score_delta > 0:
        return FrontierDriftCategory.FALLING
    return FrontierDriftCategory.STABLE


def drift_severity(
    category: FrontierDriftCategory,
    *,
    previous_action: FrontierAction | None,
    current_action: FrontierAction | None,
    rank_delta: int | None,
    score_delta: float | None,
    significant_rank_delta: int,
    significant_score_delta: float,
) -> FrontierDriftSeverity:
    if category in (FrontierDriftCategory.NEW, FrontierDriftCategory.REMOVED):
        action = current_action or previous_action
        if action == FrontierAction.RUN_PROBE:
            return FrontierDriftSeverity.HIGH
        return FrontierDriftSeverity.MEDIUM
    if category == FrontierDriftCategory.ACTION_CHANGED:
        if current_action in (FrontierAction.MITIGATE_RISK, FrontierAction.REDESIGN, FrontierAction.DEFER):
            return FrontierDriftSeverity.HIGH
        return FrontierDriftSeverity.MEDIUM
    rank_magnitude = abs(rank_delta or 0)
    score_magnitude = abs(score_delta or 0.0)
    if rank_magnitude >= max(significant_rank_delta * 2, 1):
        return FrontierDriftSeverity.HIGH
    if score_magnitude >= max(significant_score_delta * 2.0, 0.01):
        return FrontierDriftSeverity.HIGH
    if category in (FrontierDriftCategory.RISING, FrontierDriftCategory.FALLING):
        return FrontierDriftSeverity.MEDIUM
    return FrontierDriftSeverity.LOW


def drift_reasons(
    category: FrontierDriftCategory,
    *,
    previous_action: FrontierAction | None,
    current_action: FrontierAction | None,
    rank_delta: int | None,
    score_delta: float | None,
) -> tuple[str, ...]:
    reasons = [category.value]
    if previous_action != current_action:
        reasons.append(f"action:{action_value(previous_action)}->{action_value(current_action)}")
    if rank_delta:
        direction = "rank_up" if rank_delta > 0 else "rank_down"
        reasons.append(f"{direction}:{abs(rank_delta)}")
    if score_delta:
        direction = "score_up" if score_delta > 0 else "score_down"
        reasons.append(f"{direction}:{abs(score_delta):.3f}")
    return tuple(reasons)


def drift_recommendation(
    category: FrontierDriftCategory,
    severity: FrontierDriftSeverity,
    current_action: FrontierAction | None,
) -> str:
    if category == FrontierDriftCategory.NEW:
        return "triage_new_frontier"
    if category == FrontierDriftCategory.REMOVED:
        return "archive_or_explain_removed_frontier"
    if current_action == FrontierAction.MITIGATE_RISK:
        return "mitigate_before_next_run"
    if current_action == FrontierAction.REDESIGN:
        return "redesign_hypothesis"
    if current_action == FrontierAction.DEFER:
        return "defer_until_budget_or_blocker_changes"
    if severity == FrontierDriftSeverity.HIGH:
        return "review_priority_shift"
    if category == FrontierDriftCategory.RISING:
        return "consider_next_iteration"
    if category == FrontierDriftCategory.FALLING:
        return "reduce_priority_or_recheck_evidence"
    return "monitor"


def drift_sort_key(record: FrontierDriftRecord) -> tuple[int, int, float, str]:
    severity_rank = {
        FrontierDriftSeverity.HIGH: 0,
        FrontierDriftSeverity.MEDIUM: 1,
        FrontierDriftSeverity.LOW: 2,
    }[record.severity]
    return (
        severity_rank,
        -abs(record.rank_delta or 0),
        -abs(record.score_delta or 0.0),
        record.frontier_id,
    )


def drift_summary(high: int, medium: int, low: int) -> str:
    if high:
        return f"{high} high severity frontier drift item(s) need review"
    if medium:
        return f"{medium} medium severity frontier drift item(s) should be checked"
    return f"{low} low severity frontier drift item(s) observed"


def action_value(action: FrontierAction | None) -> str:
    return action.value if action is not None else "none"


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
