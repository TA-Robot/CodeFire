from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class BaselineRecord:
    baseline_id: str
    benchmark: str
    metric: str
    value: float
    source_type: str
    evidence_refs: tuple[str, ...] = field(default_factory=tuple)
    notes: str = ""


# cf-atom: CODE-BaselineRegistry
class BaselineRegistry:
    def __init__(self):
        self._records: dict[str, BaselineRecord] = {}

    def add(self, record: BaselineRecord) -> BaselineRecord:
        if record.baseline_id in self._records:
            raise ValueError(f"duplicate baseline_id: {record.baseline_id}")
        if record.source_type not in {"literature", "reproduced", "internal", "sanity_check"}:
            raise ValueError("source_type must be literature, reproduced, internal, or sanity_check")
        self._records[record.baseline_id] = record
        return record

    def get(self, baseline_id: str) -> BaselineRecord:
        return self._records[baseline_id]

    def all(self) -> list[BaselineRecord]:
        return list(self._records.values())

    def for_benchmark(self, benchmark: str, metric: str | None = None) -> list[BaselineRecord]:
        records = [record for record in self._records.values() if record.benchmark == benchmark]
        if metric is not None:
            records = [record for record in records if record.metric == metric]
        return records

    def best(self, *, benchmark: str, metric: str, maximize: bool = True) -> BaselineRecord | None:
        records = self.for_benchmark(benchmark, metric)
        if not records:
            return None
        return max(records, key=lambda record: record.value) if maximize else min(records, key=lambda record: record.value)
