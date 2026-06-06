from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import CandidateAlgorithm
from evoagent.scoring import score_candidate


@dataclass(frozen=True)
class PortfolioPolicy:
    include_exploit: bool = True
    include_novelty: bool = True
    include_cheap_probe: bool = True


# cf-atom: CODE-PortfolioScheduler
class PortfolioScheduler:
    def __init__(self, policy: PortfolioPolicy | None = None):
        self.policy = policy or PortfolioPolicy()

    def schedule(self, candidates: list[CandidateAlgorithm], *, limit: int) -> list[CandidateAlgorithm]:
        if limit <= 0 or not candidates:
            return []
        selected: list[CandidateAlgorithm] = []
        if self.policy.include_exploit:
            append_unique(selected, max(candidates, key=exploit_score), limit)
        if self.policy.include_novelty:
            append_unique(selected, max(candidates, key=lambda candidate: candidate.hypothesis.novelty), limit)
        if self.policy.include_cheap_probe:
            append_unique(selected, min(candidates, key=lambda candidate: candidate.plan.estimated_cost), limit)
        for candidate in sorted(candidates, key=score_candidate, reverse=True):
            append_unique(selected, candidate, limit)
        return selected


def append_unique(selected: list[CandidateAlgorithm], candidate: CandidateAlgorithm, limit: int) -> None:
    if len(selected) >= limit:
        return
    if candidate not in selected:
        selected.append(candidate)


def exploit_score(candidate: CandidateAlgorithm) -> float:
    if candidate.result is None:
        return candidate.hypothesis.expected_gain
    return candidate.result.quality_delta * candidate.result.confidence
