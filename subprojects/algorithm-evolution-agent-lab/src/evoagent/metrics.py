from __future__ import annotations

from dataclasses import dataclass

from evoagent.benchmarks import BenchmarkMetric
from evoagent.models import ExperimentResult


@dataclass(frozen=True)
class NormalizedMetricResult:
    metric_name: str
    raw_value: float
    baseline_value: float
    normalized_delta: float
    relative_delta: float
    improved: bool


# cf-atom: CODE-MetricNormalizer
class MetricNormalizer:
    def normalize(self, result: ExperimentResult, metric: BenchmarkMetric) -> NormalizedMetricResult:
        normalized_delta = metric.normalize_delta(result.metric_value, result.baseline_value)
        denominator = abs(result.baseline_value) if result.baseline_value != 0 else 1.0
        return NormalizedMetricResult(
            metric_name=metric.name,
            raw_value=result.metric_value,
            baseline_value=result.baseline_value,
            normalized_delta=normalized_delta,
            relative_delta=normalized_delta / denominator,
            improved=normalized_delta > 0.0,
        )

    def score(self, result: ExperimentResult, metric: BenchmarkMetric) -> float:
        return self.normalize(result, metric).relative_delta * result.confidence
