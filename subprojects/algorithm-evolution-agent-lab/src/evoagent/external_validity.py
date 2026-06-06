from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import ExperimentResult


@dataclass(frozen=True)
class BenchmarkDomain:
    benchmark: str
    domain: str


@dataclass(frozen=True)
class ExternalValidityReport:
    improved_benchmarks: tuple[str, ...]
    improved_domains: tuple[str, ...]
    mean_delta: float
    transferable: bool


# cf-atom: CODE-ExternalValidityEvaluator
class ExternalValidityEvaluator:
    def evaluate(
        self,
        *,
        results: list[ExperimentResult],
        domains: list[BenchmarkDomain],
        min_domains: int = 2,
    ) -> ExternalValidityReport:
        if min_domains <= 0:
            raise ValueError("min_domains must be positive")
        domain_by_benchmark = {item.benchmark: item.domain for item in domains}
        improvements = [result for result in results if result.quality_delta > 0]
        improved_benchmarks = tuple(sorted({result.plan.benchmark for result in improvements}))
        improved_domains = tuple(sorted({domain_by_benchmark.get(result.plan.benchmark, "unknown") for result in improvements}))
        mean_delta = sum(result.quality_delta for result in results) / len(results) if results else 0.0
        return ExternalValidityReport(
            improved_benchmarks=improved_benchmarks,
            improved_domains=improved_domains,
            mean_delta=mean_delta,
            transferable=len(improved_domains) >= min_domains,
        )
