from __future__ import annotations

import hashlib
from dataclasses import dataclass, field
from pathlib import Path


@dataclass(frozen=True)
class ExperimentGoal:
    objective: str
    target_metric: str = "validation_score"
    maximize: bool = True
    budget_hours: float = 1.0


@dataclass(frozen=True)
class Hypothesis:
    title: str
    rationale: str
    expected_gain: float
    novelty: float
    hypothesis_id: str = ""

    @property
    def uid(self) -> str:
        return self.hypothesis_id or stable_id("hyp", self.title, self.rationale)


# cf-atom: CODE-HypothesisLineage
@dataclass(frozen=True)
class HypothesisLink:
    source_id: str
    target_id: str
    relation: str


@dataclass(frozen=True)
class ExperimentPlan:
    hypothesis: Hypothesis
    benchmark: str
    baseline: str
    metric: str
    estimated_cost: float
    command: str = ""
    artifact_paths: tuple[str, ...] = field(default_factory=tuple)
    plan_id: str = ""

    @property
    def uid(self) -> str:
        return self.plan_id or stable_id(
            "plan",
            self.hypothesis.uid,
            self.benchmark,
            self.baseline,
            self.metric,
            self.command,
        )


@dataclass(frozen=True)
class ExperimentResult:
    plan: ExperimentPlan
    metric_value: float
    baseline_value: float
    confidence: float
    notes: str = ""

    @property
    def quality_delta(self) -> float:
        return self.metric_value - self.baseline_value


@dataclass(frozen=True)
class CandidateAlgorithm:
    name: str
    hypothesis: Hypothesis
    plan: ExperimentPlan
    result: ExperimentResult | None = None
    tags: tuple[str, ...] = field(default_factory=tuple)


@dataclass(frozen=True)
class RunArtifact:
    path: str
    exists: bool
    size_bytes: int


@dataclass(frozen=True)
class ExperimentRun:
    run_id: str
    plan: ExperimentPlan
    command: str
    cwd: Path
    returncode: int
    stdout: str
    stderr: str
    duration_seconds: float
    artifacts: tuple[RunArtifact, ...] = field(default_factory=tuple)

    @property
    def succeeded(self) -> bool:
        return self.returncode == 0


def stable_id(prefix: str, *parts: object) -> str:
    payload = "|".join(str(part) for part in parts)
    digest = hashlib.sha1(payload.encode("utf-8")).hexdigest()[:12]
    return f"{prefix}_{digest}"
