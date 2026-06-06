from __future__ import annotations

import json

from evoagent.models import ExperimentResult, ExperimentRun


# cf-atom: CODE-ResultIngestor
class ResultIngestor:
    def ingest_metrics(
        self,
        run: ExperimentRun,
        *,
        metrics_path: str = "metrics.json",
        metric_key: str | None = None,
        baseline_key: str | None = None,
    ) -> ExperimentResult:
        if not run.succeeded:
            raise ValueError("cannot ingest metrics from a failed run")
        metric = metric_key or run.plan.metric
        baseline_metric = baseline_key or f"baseline_{metric}"
        artifact = next((item for item in run.artifacts if item.path == metrics_path and item.exists), None)
        if artifact is None:
            raise ValueError(f"missing metrics artifact: {metrics_path}")

        payload = json.loads((run.cwd / artifact.path).read_text(encoding="utf-8"))
        if metric not in payload:
            raise ValueError(f"missing metric value: {metric}")
        if baseline_metric not in payload:
            raise ValueError(f"missing baseline metric value: {baseline_metric}")

        return ExperimentResult(
            plan=run.plan,
            metric_value=float(payload[metric]),
            baseline_value=float(payload[baseline_metric]),
            confidence=float(payload.get("confidence", 0.5)),
            notes=str(payload.get("notes", "")),
        )
