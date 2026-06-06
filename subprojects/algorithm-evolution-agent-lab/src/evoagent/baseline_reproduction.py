from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class BaselineReproduction:
    baseline_id: str
    benchmark: str
    metric: str
    value: float
    source: str
    reproduced: bool
    evidence_refs: tuple[str, ...]


# cf-atom: CODE-BaselineReproductionLedger
class BaselineReproductionLedger:
    def __init__(self):
        self._records: list[BaselineReproduction] = []

    def add(self, record: BaselineReproduction) -> BaselineReproduction:
        if not record.baseline_id or not record.benchmark or not record.metric:
            raise ValueError("baseline_id, benchmark, and metric are required")
        self._records.append(record)
        return record

    def reproduced_for(self, *, benchmark: str, metric: str) -> tuple[BaselineReproduction, ...]:
        return tuple(
            record
            for record in self._records
            if record.benchmark == benchmark and record.metric == metric and record.reproduced
        )

    def has_reproduced_baseline(self, *, benchmark: str, metric: str) -> bool:
        return bool(self.reproduced_for(benchmark=benchmark, metric=metric))

    def all(self) -> tuple[BaselineReproduction, ...]:
        return tuple(self._records)
