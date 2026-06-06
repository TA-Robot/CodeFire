from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from evoagent.failures import FailureClassification


class RetryDecision(str, Enum):
    RETRY = "retry"
    MUTATE = "mutate"
    ARCHIVE = "archive"
    HUMAN_REVIEW = "human_review"


@dataclass(frozen=True)
class RetryRecommendation:
    decision: RetryDecision
    rationale: str
    required_changes: tuple[str, ...]


# cf-atom: CODE-RetryEscalationPlanner
class RetryEscalationPlanner:
    def recommend(self, failure: FailureClassification) -> RetryRecommendation:
        if failure.category in {"implementation_error", "invalid_output"}:
            return RetryRecommendation(
                RetryDecision.RETRY,
                "failure is operational and should be retried after fixing execution or artifacts",
                ("fix_command_or_artifacts",),
            )
        if failure.category == "insufficient_budget":
            return RetryRecommendation(
                RetryDecision.HUMAN_REVIEW,
                "budget increase or experiment shrink decision needs review",
                ("adjust_budget", "confirm_value_of_information"),
            )
        if failure.category in {"instability", "inconclusive_result"}:
            return RetryRecommendation(
                RetryDecision.RETRY,
                "result is noisy and should be repeated with variance controls",
                ("repeat_seed", "tighten_hyperparameter_bounds"),
            )
        if failure.category in {"invalid_hypothesis", "benchmark_mismatch"}:
            return RetryRecommendation(
                RetryDecision.MUTATE,
                "scientific direction or benchmark fit should change before another run",
                ("mutate_hypothesis", "revise_benchmark_alignment"),
            )
        return RetryRecommendation(
            RetryDecision.ARCHIVE,
            "failure category is not actionable enough for immediate retry",
            ("archive_negative_result",),
        )
