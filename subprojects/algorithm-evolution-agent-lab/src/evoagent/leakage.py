from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class DatasetSplitAudit:
    train_ids: frozenset[str]
    validation_ids: frozenset[str]
    test_ids: frozenset[str]
    observed_metric: str
    expected_metric: str


@dataclass(frozen=True)
class LeakageFinding:
    kind: str
    severity: str
    detail: str


# cf-atom: CODE-LeakageChecker
class LeakageChecker:
    def check(self, audit: DatasetSplitAudit) -> list[LeakageFinding]:
        findings: list[LeakageFinding] = []
        for left_name, left, right_name, right in (
            ("train", audit.train_ids, "validation", audit.validation_ids),
            ("train", audit.train_ids, "test", audit.test_ids),
            ("validation", audit.validation_ids, "test", audit.test_ids),
        ):
            overlap = left & right
            if overlap:
                findings.append(
                    LeakageFinding(
                        kind="split_overlap",
                        severity="blocking",
                        detail=f"{left_name}/{right_name} share {len(overlap)} sample id(s)",
                    )
                )
        if audit.observed_metric != audit.expected_metric:
            findings.append(
                LeakageFinding(
                    kind="metric_misuse",
                    severity="blocking",
                    detail=f"observed metric {audit.observed_metric} does not match expected {audit.expected_metric}",
                )
            )
        return findings
