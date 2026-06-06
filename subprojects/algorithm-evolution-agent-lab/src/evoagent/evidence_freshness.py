from __future__ import annotations

from dataclasses import dataclass

from evoagent.evidence import EvidenceRecord


@dataclass(frozen=True)
class VersionedEvidenceRef:
    evidence: EvidenceRecord
    benchmark_version: str
    dataset_version: str
    protocol_version: str


@dataclass(frozen=True)
class EvidenceFreshnessFinding:
    evidence_id: str
    kind: str
    detail: str


# cf-atom: CODE-EvidenceFreshnessAudit
class EvidenceFreshnessAudit:
    def audit(
        self,
        refs: tuple[VersionedEvidenceRef, ...],
        *,
        current_benchmark_version: str,
        current_dataset_version: str,
        current_protocol_version: str,
    ) -> tuple[EvidenceFreshnessFinding, ...]:
        findings: list[EvidenceFreshnessFinding] = []
        for ref in refs:
            if ref.benchmark_version != current_benchmark_version:
                findings.append(
                    EvidenceFreshnessFinding(
                        ref.evidence.evidence_id,
                        "stale_benchmark",
                        f"{ref.benchmark_version} != {current_benchmark_version}",
                    )
                )
            if ref.dataset_version != current_dataset_version:
                findings.append(
                    EvidenceFreshnessFinding(
                        ref.evidence.evidence_id,
                        "stale_dataset",
                        f"{ref.dataset_version} != {current_dataset_version}",
                    )
                )
            if ref.protocol_version != current_protocol_version:
                findings.append(
                    EvidenceFreshnessFinding(
                        ref.evidence.evidence_id,
                        "stale_protocol",
                        f"{ref.protocol_version} != {current_protocol_version}",
                    )
                )
        return tuple(findings)
