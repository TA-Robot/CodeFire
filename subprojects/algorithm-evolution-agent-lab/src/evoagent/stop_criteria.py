from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class StopCriteria:
    timeout_seconds: float | None = None
    max_cost: float | None = None
    plateau_window: int = 0
    min_delta: float = 0.0
    require_valid_output: bool = True
    block_on_safety_violation: bool = True


@dataclass(frozen=True)
class StopObservation:
    elapsed_seconds: float
    cumulative_cost: float
    recent_metric_values: tuple[float, ...] = ()
    output_valid: bool = True
    safety_violations: tuple[str, ...] = ()


@dataclass(frozen=True)
class StopFinding:
    kind: str
    detail: str


# cf-atom: CODE-StopCriteriaEvaluator
class StopCriteriaEvaluator:
    def evaluate(
        self,
        observation: StopObservation,
        criteria: StopCriteria,
    ) -> tuple[StopFinding, ...]:
        findings: list[StopFinding] = []
        if criteria.timeout_seconds is not None and observation.elapsed_seconds >= criteria.timeout_seconds:
            findings.append(
                StopFinding(
                    "timeout",
                    f"elapsed {observation.elapsed_seconds:g}s reached limit {criteria.timeout_seconds:g}s",
                )
            )
        if criteria.max_cost is not None and observation.cumulative_cost >= criteria.max_cost:
            findings.append(
                StopFinding(
                    "maximum_cost",
                    f"cost {observation.cumulative_cost:g} reached limit {criteria.max_cost:g}",
                )
            )
        if criteria.require_valid_output and not observation.output_valid:
            findings.append(StopFinding("invalid_output", "required output contract is invalid"))
        if criteria.block_on_safety_violation and observation.safety_violations:
            findings.append(
                StopFinding(
                    "safety_policy_violation",
                    ", ".join(observation.safety_violations),
                )
            )
        if plateaued(observation.recent_metric_values, criteria.plateau_window, criteria.min_delta):
            findings.append(
                StopFinding(
                    "metric_plateau",
                    f"last {criteria.plateau_window:g} metric values improved by less than {criteria.min_delta:g}",
                )
            )
        return tuple(findings)

    def should_stop(self, observation: StopObservation, criteria: StopCriteria) -> bool:
        return bool(self.evaluate(observation, criteria))


def plateaued(values: tuple[float, ...], window: int, min_delta: float) -> bool:
    if window <= 1 or len(values) < window:
        return False
    tail = values[-window:]
    return max(tail) - min(tail) < min_delta
