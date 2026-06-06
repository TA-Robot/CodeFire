from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import ExperimentResult, ExperimentRun


@dataclass(frozen=True)
class FailureClassification:
    category: str
    confidence: float
    reason: str
    recommended_action: str


# cf-atom: CODE-FailureClassifier
class FailureClassifier:
    def classify(
        self,
        run: ExperimentRun,
        result: ExperimentResult | None = None,
    ) -> FailureClassification:
        if run.succeeded and result is not None:
            return classify_result(result)
        if run.succeeded:
            return FailureClassification(
                category="invalid_output",
                confidence=0.75,
                reason="run completed but did not produce an ingested result",
                recommended_action="check artifact contract and result ingestion before retrying",
            )

        text = f"{run.stdout}\n{run.stderr}".lower()
        if run.returncode == 124 or "timeout" in text or "time limit" in text:
            return FailureClassification(
                category="insufficient_budget",
                confidence=0.85,
                reason="run exceeded configured time or compute allowance",
                recommended_action="reduce experiment scope or allocate a larger budget",
            )
        if any(token in text for token in ("dataset", "split", "metric", "benchmark")):
            return FailureClassification(
                category="benchmark_mismatch",
                confidence=0.8,
                reason="failure references benchmark, split, dataset, or metric mismatch",
                recommended_action="verify benchmark definition and metric normalization",
            )
        if any(token in text for token in ("nan", "overflow", "diverg", "unstable", "seed")):
            return FailureClassification(
                category="instability",
                confidence=0.7,
                reason="failure looks like numerical or seed-dependent instability",
                recommended_action="replicate with guarded hyperparameters and capture diagnostics",
            )
        return FailureClassification(
            category="implementation_error",
            confidence=0.65,
            reason=first_observed_line(run) or "process exited unsuccessfully",
            recommended_action="fix runner command or experiment implementation before retrying",
        )


def classify_result(result: ExperimentResult) -> FailureClassification:
    if result.confidence < 0.35:
        return FailureClassification(
            category="inconclusive_result",
            confidence=0.7,
            reason="observed metric confidence is too low for a research decision",
            recommended_action="repeat the experiment or use a lower-variance benchmark",
        )
    if result.quality_delta < 0 and result.confidence >= 0.7:
        return FailureClassification(
            category="invalid_hypothesis",
            confidence=result.confidence,
            reason="candidate regressed against baseline with high confidence",
            recommended_action="archive as a negative result or mutate a different mechanism",
        )
    return FailureClassification(
        category="none",
        confidence=result.confidence,
        reason="run produced a usable result",
        recommended_action="continue according to the analysis note",
    )


def first_observed_line(run: ExperimentRun) -> str:
    for text in (run.stderr, run.stdout):
        for line in text.splitlines():
            stripped = line.strip()
            if stripped:
                return stripped
    return ""
