from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class BenchmarkDomainDefinition:
    benchmark_id: str
    domain: str
    dataset: str
    metric: str


# cf-atom: CODE-BenchmarkDomainCatalog
class BenchmarkDomainCatalog:
    def __init__(self):
        self._benchmarks: dict[str, BenchmarkDomainDefinition] = {}

    def add(self, benchmark: BenchmarkDomainDefinition) -> BenchmarkDomainDefinition:
        if benchmark.benchmark_id in self._benchmarks:
            raise ValueError(f"duplicate benchmark_id: {benchmark.benchmark_id}")
        if not benchmark.domain:
            raise ValueError("domain is required")
        self._benchmarks[benchmark.benchmark_id] = benchmark
        return benchmark

    def by_domain(self, domain: str) -> tuple[BenchmarkDomainDefinition, ...]:
        return tuple(
            benchmark
            for benchmark in self._benchmarks.values()
            if benchmark.domain == domain
        )

    def domains(self) -> tuple[str, ...]:
        return tuple(sorted({benchmark.domain for benchmark in self._benchmarks.values()}))

    def all(self) -> tuple[BenchmarkDomainDefinition, ...]:
        return tuple(self._benchmarks.values())
