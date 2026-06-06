from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class ContaminationSource:
    source_id: str
    dataset: str
    artifact_hashes: tuple[str, ...]
    notes: str = ""


@dataclass(frozen=True)
class ContaminationFinding:
    source_id: str
    kind: str
    detail: str


# cf-atom: CODE-ContaminationSourceRegistry
class ContaminationSourceRegistry:
    def __init__(self):
        self._sources: dict[str, ContaminationSource] = {}

    def add(self, source: ContaminationSource) -> ContaminationSource:
        if source.source_id in self._sources:
            raise ValueError(f"duplicate source_id: {source.source_id}")
        self._sources[source.source_id] = source
        return source

    def audit(
        self,
        *,
        dataset: str,
        artifact_hashes: tuple[str, ...],
    ) -> tuple[ContaminationFinding, ...]:
        findings: list[ContaminationFinding] = []
        query_hashes = set(artifact_hashes)
        for source in self._sources.values():
            if source.dataset == dataset:
                findings.append(
                    ContaminationFinding(
                        source.source_id,
                        "dataset_overlap",
                        f"{dataset} matches prohibited contamination source",
                    )
                )
            overlap = query_hashes.intersection(source.artifact_hashes)
            if overlap:
                findings.append(
                    ContaminationFinding(
                        source.source_id,
                        "artifact_overlap",
                        ",".join(sorted(overlap)),
                    )
                )
        return tuple(findings)

    def all(self) -> tuple[ContaminationSource, ...]:
        return tuple(self._sources.values())
