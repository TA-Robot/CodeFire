from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class LearningCurveReport:
    trend: str
    best_value: float
    last_value: float
    slope: float
    unstable: bool
    plateau: bool


# cf-atom: CODE-LearningCurveAnalyzer
class LearningCurveAnalyzer:
    def analyze(
        self,
        metric_values: tuple[float, ...],
        *,
        min_delta: float,
        instability_tolerance: float,
    ) -> LearningCurveReport:
        if not metric_values:
            raise ValueError("metric_values are required")
        if min_delta < 0 or instability_tolerance < 0:
            raise ValueError("thresholds must be non-negative")

        best_value = max(metric_values)
        last_value = metric_values[-1]
        slope = last_value - metric_values[0] if len(metric_values) > 1 else 0.0
        reversals = count_reversals(metric_values)
        unstable = reversals > 1 or max(metric_values) - min(metric_values) > instability_tolerance
        plateau = len(metric_values) >= 3 and max(metric_values[-3:]) - min(metric_values[-3:]) < min_delta
        trend = "improving" if slope >= min_delta else "regressing" if slope <= -min_delta else "flat"
        if plateau:
            trend = "plateau"
        if unstable:
            trend = "unstable"

        return LearningCurveReport(
            trend=trend,
            best_value=best_value,
            last_value=last_value,
            slope=slope,
            unstable=unstable,
            plateau=plateau,
        )


def count_reversals(values: tuple[float, ...]) -> int:
    directions: list[int] = []
    for previous, current in zip(values, values[1:]):
        delta = current - previous
        if delta > 0:
            directions.append(1)
        elif delta < 0:
            directions.append(-1)
    return sum(1 for previous, current in zip(directions, directions[1:]) if previous != current)
