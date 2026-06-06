from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class ChallengeBaseline:
    name: str
    metric_value: float
    source: str


@dataclass(frozen=True)
class ComputeBudget:
    max_hours: float
    max_trials: int


@dataclass(frozen=True)
class SotaChallengeDefinition:
    challenge_id: str
    task: str
    dataset: str
    metric: str
    split: str
    target_improvement: float
    allowed_compute: ComputeBudget
    baselines: tuple[ChallengeBaseline, ...]
    disallowed_shortcuts: tuple[str, ...]

    @property
    def best_baseline(self) -> ChallengeBaseline:
        return max(self.baselines, key=lambda baseline: baseline.metric_value)


# cf-atom: CODE-SotaChallengeDefinition
def load_challenge_definition(path: str | Path) -> SotaChallengeDefinition:
    data = json.loads(Path(path).read_text(encoding="utf-8"))
    required = {
        "challenge_id",
        "task",
        "dataset",
        "metric",
        "split",
        "target_improvement",
        "allowed_compute",
        "baselines",
        "disallowed_shortcuts",
    }
    missing = sorted(required - set(data))
    if missing:
        raise ValueError(f"missing challenge fields: {', '.join(missing)}")

    baselines = tuple(
        ChallengeBaseline(
            name=expect_nonempty_text(baseline, "name"),
            metric_value=float(baseline["metric_value"]),
            source=expect_nonempty_text(baseline, "source"),
        )
        for baseline in data["baselines"]
    )
    if not baselines:
        raise ValueError("challenge must include at least one baseline")

    allowed_compute = data["allowed_compute"]
    definition = SotaChallengeDefinition(
        challenge_id=expect_nonempty_text(data, "challenge_id"),
        task=expect_nonempty_text(data, "task"),
        dataset=expect_nonempty_text(data, "dataset"),
        metric=expect_nonempty_text(data, "metric"),
        split=expect_nonempty_text(data, "split"),
        target_improvement=float(data["target_improvement"]),
        allowed_compute=ComputeBudget(
            max_hours=float(allowed_compute["max_hours"]),
            max_trials=int(allowed_compute["max_trials"]),
        ),
        baselines=baselines,
        disallowed_shortcuts=tuple(str(item) for item in data["disallowed_shortcuts"]),
    )
    validate_challenge(definition)
    return definition


def validate_challenge(definition: SotaChallengeDefinition) -> None:
    if definition.target_improvement <= 0.0:
        raise ValueError("target_improvement must be positive")
    if definition.allowed_compute.max_hours <= 0.0:
        raise ValueError("allowed_compute.max_hours must be positive")
    if definition.allowed_compute.max_trials <= 0:
        raise ValueError("allowed_compute.max_trials must be positive")
    if not definition.disallowed_shortcuts:
        raise ValueError("challenge must list disallowed shortcuts")


def expect_nonempty_text(data: dict[str, object], key: str) -> str:
    value = str(data[key]).strip()
    if not value:
        raise ValueError(f"{key} must be nonempty")
    return value
