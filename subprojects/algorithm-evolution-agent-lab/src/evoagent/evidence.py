from __future__ import annotations

import hashlib
from dataclasses import dataclass, field
from datetime import datetime, timezone


@dataclass(frozen=True)
class EvidenceRecord:
    evidence_id: str
    claim_id: str
    source_type: str
    source_ref: str
    summary: str
    confidence: float
    created_at: str
    tags: tuple[str, ...] = field(default_factory=tuple)


# cf-atom: CODE-EvidenceLedger
class EvidenceLedger:
    def __init__(self):
        self._records: list[EvidenceRecord] = []

    def add(
        self,
        *,
        claim_id: str,
        source_type: str,
        source_ref: str,
        summary: str,
        confidence: float,
        tags: tuple[str, ...] = (),
    ) -> EvidenceRecord:
        if confidence < 0.0 or confidence > 1.0:
            raise ValueError("confidence must be between 0 and 1")
        record = EvidenceRecord(
            evidence_id=evidence_id_for(claim_id, source_type, source_ref, len(self._records)),
            claim_id=claim_id,
            source_type=source_type,
            source_ref=source_ref,
            summary=summary,
            confidence=confidence,
            created_at=now_iso(),
            tags=tags,
        )
        self._records.append(record)
        return record

    def for_claim(self, claim_id: str) -> list[EvidenceRecord]:
        return [record for record in self._records if record.claim_id == claim_id]

    def all(self) -> list[EvidenceRecord]:
        return list(self._records)

    def claim_confidence(self, claim_id: str) -> float:
        records = self.for_claim(claim_id)
        if not records:
            return 0.0
        return max(record.confidence for record in records)


def evidence_id_for(claim_id: str, source_type: str, source_ref: str, index: int) -> str:
    payload = f"{claim_id}|{source_type}|{source_ref}|{index}"
    return "ev_" + hashlib.sha1(payload.encode("utf-8")).hexdigest()[:12]


def now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
