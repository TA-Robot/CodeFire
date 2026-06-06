from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class HumanOverride:
    override_id: str
    target_id: str
    action: str
    rationale: str
    operator: str
    priority: float | None = None
    evidence_refs: tuple[str, ...] = ()


# cf-atom: CODE-HumanOverrideLedger
class HumanOverrideLedger:
    def __init__(self):
        self._records: list[HumanOverride] = []

    def add(
        self,
        *,
        target_id: str,
        action: str,
        rationale: str,
        operator: str,
        priority: float | None = None,
        evidence_refs: tuple[str, ...] = (),
    ) -> HumanOverride:
        if action not in {"priority_override", "freeze", "reject", "manual_evidence"}:
            raise ValueError("unsupported override action")
        if not rationale.strip():
            raise ValueError("rationale is required")
        record = HumanOverride(
            override_id=f"override-{len(self._records) + 1}",
            target_id=target_id,
            action=action,
            rationale=rationale,
            operator=operator,
            priority=priority,
            evidence_refs=evidence_refs,
        )
        self._records.append(record)
        return record

    def for_target(self, target_id: str) -> tuple[HumanOverride, ...]:
        return tuple(record for record in self._records if record.target_id == target_id)

    def active_freezes(self) -> tuple[HumanOverride, ...]:
        return tuple(record for record in self._records if record.action == "freeze")

    def all(self) -> tuple[HumanOverride, ...]:
        return tuple(self._records)
