from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class MemoryRecord:
    record_id: str
    kind: str
    title: str
    summary: str
    tags: tuple[str, ...]


# cf-atom: CODE-ResearchMemoryIndex
class ResearchMemoryIndex:
    def __init__(self):
        self._records: list[MemoryRecord] = []

    def add(self, *, kind: str, title: str, summary: str, tags: tuple[str, ...]) -> MemoryRecord:
        if not kind or not title:
            raise ValueError("kind and title are required")
        record = MemoryRecord(
            record_id=f"memory-{len(self._records) + 1}",
            kind=kind,
            title=title,
            summary=summary,
            tags=normalize_tags(tags),
        )
        self._records.append(record)
        return record

    def search(self, *, tag: str | None = None, kind: str | None = None, text: str | None = None) -> tuple[MemoryRecord, ...]:
        normalized_tag = tag.lower() if tag else None
        normalized_text = text.lower() if text else None
        return tuple(
            record
            for record in self._records
            if (kind is None or record.kind == kind)
            and (normalized_tag is None or normalized_tag in record.tags)
            and (
                normalized_text is None
                or normalized_text in record.title.lower()
                or normalized_text in record.summary.lower()
            )
        )

    def all(self) -> tuple[MemoryRecord, ...]:
        return tuple(self._records)


def normalize_tags(tags: tuple[str, ...]) -> tuple[str, ...]:
    return tuple(sorted({tag.strip().lower() for tag in tags if tag.strip()}))
