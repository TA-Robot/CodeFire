from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import CandidateAlgorithm


@dataclass(frozen=True)
class PolicyEvaluationReport:
    policy_name: str
    selected_names: tuple[str, ...]
    oracle_names: tuple[str, ...]
    hit_rate: float
    selected_mean_delta: float
    oracle_mean_delta: float
    regret: float
    selected_cost: float


# cf-atom: CODE-PolicyEvaluator
class PolicyEvaluator:
    def evaluate(
        self,
        *,
        policy_name: str,
        candidates: list[CandidateAlgorithm],
        selected: list[CandidateAlgorithm],
        limit: int,
    ) -> PolicyEvaluationReport:
        if limit <= 0:
            raise ValueError("limit must be positive")
        evaluated = [candidate for candidate in candidates if candidate.result is not None]
        if not evaluated:
            raise ValueError("at least one candidate must have a result")

        k = min(limit, len(evaluated))
        selected_evaluated = [candidate for candidate in selected if candidate.result is not None][:k]
        if not selected_evaluated:
            raise ValueError("at least one selected candidate must have a result")

        oracle = sorted(evaluated, key=outcome_score, reverse=True)[:k]
        selected_names = tuple(candidate.name for candidate in selected_evaluated)
        oracle_names = tuple(candidate.name for candidate in oracle)
        selected_mean = mean_delta(selected_evaluated)
        oracle_mean = mean_delta(oracle)
        return PolicyEvaluationReport(
            policy_name=policy_name,
            selected_names=selected_names,
            oracle_names=oracle_names,
            hit_rate=overlap_rate(selected_names, oracle_names),
            selected_mean_delta=selected_mean,
            oracle_mean_delta=oracle_mean,
            regret=max(0.0, oracle_mean - selected_mean),
            selected_cost=sum(max(candidate.plan.estimated_cost, 0.0) for candidate in selected_evaluated),
        )


def outcome_score(candidate: CandidateAlgorithm) -> float:
    assert candidate.result is not None
    return candidate.result.quality_delta * candidate.result.confidence


def mean_delta(candidates: list[CandidateAlgorithm]) -> float:
    return sum(candidate.result.quality_delta for candidate in candidates if candidate.result is not None) / len(candidates)


def overlap_rate(selected_names: tuple[str, ...], oracle_names: tuple[str, ...]) -> float:
    if not oracle_names:
        return 0.0
    return len(set(selected_names) & set(oracle_names)) / len(oracle_names)
