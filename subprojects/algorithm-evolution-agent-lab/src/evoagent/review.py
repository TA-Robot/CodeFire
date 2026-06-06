from __future__ import annotations

from dataclasses import dataclass, field, replace


@dataclass(frozen=True)
class ReviewObjection:
    objection_id: str
    claim_id: str
    summary: str
    severity: str
    evidence_refs: tuple[str, ...] = field(default_factory=tuple)
    resolved: bool = False
    resolution: str = ""


# cf-atom: CODE-ReviewLedger
class ReviewLedger:
    def __init__(self):
        self._objections: list[ReviewObjection] = []

    def add_objection(
        self,
        *,
        claim_id: str,
        summary: str,
        severity: str,
        evidence_refs: tuple[str, ...] = (),
    ) -> ReviewObjection:
        if severity not in {"low", "medium", "high", "blocking"}:
            raise ValueError("unsupported objection severity")
        objection = ReviewObjection(
            objection_id=f"objection-{len(self._objections) + 1}",
            claim_id=claim_id,
            summary=summary,
            severity=severity,
            evidence_refs=evidence_refs,
        )
        self._objections.append(objection)
        return objection

    def resolve(self, objection_id: str, *, resolution: str) -> ReviewObjection:
        if not resolution.strip():
            raise ValueError("resolution must be nonempty")
        for index, objection in enumerate(self._objections):
            if objection.objection_id == objection_id:
                resolved = replace(objection, resolved=True, resolution=resolution)
                self._objections[index] = resolved
                return resolved
        raise KeyError(objection_id)

    def open_objections(self, claim_id: str | None = None) -> tuple[ReviewObjection, ...]:
        return tuple(
            objection
            for objection in self._objections
            if not objection.resolved and (claim_id is None or objection.claim_id == claim_id)
        )

    def blocking_objections(self, claim_id: str | None = None) -> tuple[ReviewObjection, ...]:
        return tuple(
            objection
            for objection in self.open_objections(claim_id)
            if objection.severity == "blocking"
        )

    def all(self) -> tuple[ReviewObjection, ...]:
        return tuple(self._objections)
