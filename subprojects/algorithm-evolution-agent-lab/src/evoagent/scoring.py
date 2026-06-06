from __future__ import annotations

from evoagent.models import CandidateAlgorithm


# cf-atom: CODE-CandidateScoring
def score_candidate(candidate: CandidateAlgorithm) -> float:
    """Rank candidates for the next experiment.

    This is a scheduling score, not a scientific claim.
    """
    quality = candidate.hypothesis.expected_gain
    confidence = 0.25
    if candidate.result is not None:
        quality = candidate.result.quality_delta
        confidence = candidate.result.confidence

    novelty = candidate.hypothesis.novelty
    cost = max(candidate.plan.estimated_cost, 0.0)
    return (0.55 * quality) + (0.25 * novelty) + (0.20 * confidence) - (0.10 * cost)
