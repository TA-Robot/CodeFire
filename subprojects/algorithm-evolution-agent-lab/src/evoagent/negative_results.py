from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class NegativeResult:
    result_id: str
    hypothesis_id: str
    failure_mode: str
    summary: str
    avoid_conditions: tuple[str, ...]
    evidence_refs: tuple[str, ...]


# cf-atom: CODE-NegativeResultArchive
class NegativeResultArchive:
    def __init__(self):
        self._results: list[NegativeResult] = []

    def add(
        self,
        *,
        hypothesis_id: str,
        failure_mode: str,
        summary: str,
        avoid_conditions: tuple[str, ...],
        evidence_refs: tuple[str, ...] = (),
    ) -> NegativeResult:
        if not hypothesis_id or not failure_mode:
            raise ValueError("hypothesis_id and failure_mode are required")
        result = NegativeResult(
            result_id=f"negative-{len(self._results) + 1}",
            hypothesis_id=hypothesis_id,
            failure_mode=failure_mode,
            summary=summary,
            avoid_conditions=avoid_conditions,
            evidence_refs=evidence_refs,
        )
        self._results.append(result)
        return result

    def by_failure_mode(self, failure_mode: str) -> tuple[NegativeResult, ...]:
        return tuple(result for result in self._results if result.failure_mode == failure_mode)

    def should_avoid(self, condition: str) -> bool:
        normalized = condition.lower()
        return any(
            normalized in avoid_condition.lower()
            for result in self._results
            for avoid_condition in result.avoid_conditions
        )

    def all(self) -> tuple[NegativeResult, ...]:
        return tuple(self._results)
