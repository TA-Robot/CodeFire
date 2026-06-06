from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import CandidateAlgorithm


@dataclass(frozen=True)
class AlgorithmFamilyReference:
    family: str
    keywords: tuple[str, ...]
    source: str


@dataclass(frozen=True)
class NoveltyFinding:
    candidate_name: str
    family: str
    overlap_terms: tuple[str, ...]
    source: str
    severity: str


# cf-atom: CODE-NoveltyReviewer
class NoveltyReviewer:
    def review(
        self,
        *,
        candidate: CandidateAlgorithm,
        known_families: list[AlgorithmFamilyReference],
        min_overlap: int = 2,
    ) -> list[NoveltyFinding]:
        if min_overlap <= 0:
            raise ValueError("min_overlap must be positive")
        text = f"{candidate.hypothesis.title} {candidate.hypothesis.rationale}".lower()
        findings: list[NoveltyFinding] = []
        for reference in known_families:
            overlap = tuple(keyword for keyword in reference.keywords if keyword.lower() in text)
            if len(overlap) >= min_overlap:
                findings.append(
                    NoveltyFinding(
                        candidate_name=candidate.name,
                        family=reference.family,
                        overlap_terms=overlap,
                        source=reference.source,
                        severity="review_required",
                    )
                )
        return findings
