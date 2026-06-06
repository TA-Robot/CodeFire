from __future__ import annotations

from dataclasses import dataclass

from evoagent.failures import FailureClassifier
from evoagent.models import ExperimentResult, ExperimentRun


@dataclass(frozen=True)
class AnalysisNote:
    summary: str
    metric_comparison: str
    confidence: float
    failure_classification: str
    suspected_mechanism: str
    next_action: str
    claim_readiness_update: str


# cf-atom: CODE-RunAnalyst
class RunAnalyst:
    def __init__(self, failure_classifier: FailureClassifier | None = None):
        self.failure_classifier = failure_classifier or FailureClassifier()

    def analyze(self, run: ExperimentRun, result: ExperimentResult | None = None) -> AnalysisNote:
        if not run.succeeded:
            failure = self.failure_classifier.classify(run)
            return AnalysisNote(
                summary=f"Run {run.run_id} failed before producing a usable result.",
                metric_comparison="not available",
                confidence=0.0,
                failure_classification=failure.category,
                suspected_mechanism=failure.reason,
                next_action=failure.recommended_action,
                claim_readiness_update="no claim progress",
            )
        if result is None:
            failure = self.failure_classifier.classify(run)
            return AnalysisNote(
                summary=f"Run {run.run_id} completed but has not been ingested.",
                metric_comparison="not ingested",
                confidence=0.1,
                failure_classification=failure.category,
                suspected_mechanism=failure.reason,
                next_action=failure.recommended_action,
                claim_readiness_update="no claim progress",
            )

        delta = result.quality_delta
        failure = self.failure_classifier.classify(run, result)
        direction = "improved" if delta > 0 else "regressed" if delta < 0 else "matched"
        next_action = "replicate with another seed" if delta > 0 else "mutate or reject this candidate"
        readiness = "local improvement candidate" if delta > 0 and result.confidence >= 0.5 else "hypothesis only"
        return AnalysisNote(
            summary=f"Run {run.run_id} {direction} {result.plan.metric} by {delta:.4f}.",
            metric_comparison=f"{result.metric_value:.4f} vs baseline {result.baseline_value:.4f}",
            confidence=result.confidence,
            failure_classification=failure.category,
            suspected_mechanism=result.notes or failure.reason,
            next_action=next_action,
            claim_readiness_update=readiness,
        )


def first_nonempty_line(text: str) -> str:
    for line in text.splitlines():
        stripped = line.strip()
        if stripped:
            return stripped
    return ""
