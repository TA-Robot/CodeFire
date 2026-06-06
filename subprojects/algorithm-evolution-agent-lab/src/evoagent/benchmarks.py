from __future__ import annotations

import math
from dataclasses import dataclass, field


@dataclass(frozen=True)
class BenchmarkMetric:
    name: str
    maximize: bool = True
    target: float | None = None
    tolerance: float = 0.0

    def normalize_delta(self, value: float, baseline: float) -> float:
        delta = value - baseline if self.maximize else baseline - value
        return 0.0 if abs(delta) <= self.tolerance else delta


@dataclass(frozen=True)
class BenchmarkDefinition:
    benchmark_id: str
    task: str
    dataset: str
    split: str
    metric: BenchmarkMetric
    baseline_ids: tuple[str, ...] = field(default_factory=tuple)
    pitfalls: tuple[str, ...] = field(default_factory=tuple)
    validation_protocol: str = ""
    version: int = 1


@dataclass(frozen=True)
class RepeatedRunSummary:
    metric: BenchmarkMetric
    values: tuple[float, ...]
    baseline: float

    @property
    def mean(self) -> float:
        return sum(self.values) / len(self.values)

    @property
    def sample_stddev(self) -> float:
        if len(self.values) < 2:
            return 0.0
        mean = self.mean
        variance = sum((value - mean) ** 2 for value in self.values) / (len(self.values) - 1)
        return math.sqrt(variance)

    @property
    def standard_error(self) -> float:
        return self.sample_stddev / math.sqrt(len(self.values))

    @property
    def normalized_delta(self) -> float:
        return self.metric.normalize_delta(self.mean, self.baseline)

    @property
    def confidence(self) -> float:
        if len(self.values) < 2:
            return 0.25 if self.normalized_delta > 0 else 0.0
        uncertainty = self.standard_error + self.metric.tolerance
        if uncertainty == 0:
            return 1.0 if self.normalized_delta > 0 else 0.0
        signal_to_noise = self.normalized_delta / uncertainty
        return max(0.0, min(1.0, signal_to_noise / 3.0))

    @property
    def promoted(self) -> bool:
        return len(self.values) >= 3 and self.normalized_delta > 0 and self.confidence >= 0.5


# cf-atom: CODE-BenchmarkRegistry
class BenchmarkRegistry:
    def __init__(self):
        self._benchmarks: dict[str, BenchmarkDefinition] = {}

    def add(self, benchmark: BenchmarkDefinition) -> BenchmarkDefinition:
        if benchmark.benchmark_id in self._benchmarks:
            raise ValueError(f"duplicate benchmark_id: {benchmark.benchmark_id}")
        validate_benchmark(benchmark)
        self._benchmarks[benchmark.benchmark_id] = benchmark
        return benchmark

    def refresh(self, benchmark: BenchmarkDefinition) -> BenchmarkDefinition:
        existing = self._benchmarks.get(benchmark.benchmark_id)
        if existing is not None and benchmark.version <= existing.version:
            raise ValueError("refreshed benchmark version must increase")
        validate_benchmark(benchmark)
        self._benchmarks[benchmark.benchmark_id] = benchmark
        return benchmark

    def get(self, benchmark_id: str) -> BenchmarkDefinition:
        return self._benchmarks[benchmark_id]

    def all(self) -> list[BenchmarkDefinition]:
        return list(self._benchmarks.values())

    def by_metric(self, metric_name: str) -> list[BenchmarkDefinition]:
        return [
            benchmark
            for benchmark in self._benchmarks.values()
            if benchmark.metric.name == metric_name
        ]


def summarize_repeated_runs(
    metric: BenchmarkMetric,
    values: tuple[float, ...] | list[float],
    *,
    baseline: float,
) -> RepeatedRunSummary:
    if not values:
        raise ValueError("at least one repeated run value is required")
    return RepeatedRunSummary(metric=metric, values=tuple(float(value) for value in values), baseline=baseline)


def validate_benchmark(benchmark: BenchmarkDefinition) -> None:
    if not benchmark.benchmark_id:
        raise ValueError("benchmark_id is required")
    if not benchmark.task:
        raise ValueError("task is required")
    if not benchmark.dataset:
        raise ValueError("dataset is required")
    if not benchmark.split:
        raise ValueError("split is required")
    if not benchmark.metric.name:
        raise ValueError("metric name is required")
    if benchmark.metric.tolerance < 0:
        raise ValueError("metric tolerance must be non-negative")
    if benchmark.version < 1:
        raise ValueError("benchmark version must be positive")
