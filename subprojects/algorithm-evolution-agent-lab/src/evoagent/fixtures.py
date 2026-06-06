from __future__ import annotations

import json
import platform
import sys
from dataclasses import dataclass
from pathlib import Path

from evoagent.benchmarks import BenchmarkDefinition, BenchmarkMetric
from evoagent.models import ExperimentPlan, Hypothesis


@dataclass(frozen=True)
class FixtureResult:
    metrics: dict[str, float | str]
    config: dict[str, int | str]
    analysis: str


# cf-atom: CODE-LocalBenchmarkFixtures
class ToyTabularFixture:
    benchmark_id = "toy-tabular-v1"

    def definition(self) -> BenchmarkDefinition:
        return BenchmarkDefinition(
            benchmark_id=self.benchmark_id,
            task="binary classification",
            dataset="deterministic synthetic tabular",
            split="seeded-80-20",
            metric=BenchmarkMetric("accuracy", maximize=True, tolerance=0.001),
            baseline_ids=("majority-class",),
            pitfalls=("small validation set", "linear threshold bias"),
            validation_protocol="run at least three seeds before promotion",
            version=1,
        )

    def plan_for(self, hypothesis: Hypothesis, *, candidate: str, seed: int = 0) -> ExperimentPlan:
        command = (
            f"{sys.executable} -m evoagent.fixture_runner "
            f"--fixture toy-tabular --candidate {candidate} --seed {seed}"
        )
        return ExperimentPlan(
            hypothesis=hypothesis,
            benchmark=self.benchmark_id,
            baseline="majority-class",
            metric="accuracy",
            estimated_cost=0.02,
            command=command,
            artifact_paths=("metrics.json", "config.json", "environment.json", "run.log", "analysis.md"),
        )

    def evaluate(self, *, candidate: str, seed: int = 0) -> FixtureResult:
        rows = synthetic_rows(seed=seed, count=80)
        train = rows[:60]
        validation = rows[60:]
        baseline_prediction = majority_label(train)
        baseline_accuracy = accuracy([baseline_prediction for _ in validation], [row[2] for row in validation])
        predictions = [predict(candidate, row[0], row[1], train) for row in validation]
        candidate_accuracy = accuracy(predictions, [row[2] for row in validation])
        delta = candidate_accuracy - baseline_accuracy
        metrics: dict[str, float | str] = {
            "accuracy": candidate_accuracy,
            "baseline_accuracy": baseline_accuracy,
            "confidence": 0.55 if delta > 0 else 0.2,
            "candidate": candidate,
            "fixture": self.benchmark_id,
        }
        config: dict[str, int | str] = {
            "seed": seed,
            "candidate": candidate,
            "train_rows": len(train),
            "validation_rows": len(validation),
        }
        direction = "improved" if delta > 0 else "matched" if delta == 0 else "regressed"
        analysis = f"{candidate} {direction} accuracy by {delta:.4f} on {self.benchmark_id}."
        return FixtureResult(metrics=metrics, config=config, analysis=analysis)

    def write_run_artifacts(self, output_dir: Path, *, candidate: str, seed: int = 0) -> FixtureResult:
        output_dir.mkdir(parents=True, exist_ok=True)
        result = self.evaluate(candidate=candidate, seed=seed)
        (output_dir / "metrics.json").write_text(json.dumps(result.metrics, sort_keys=True), encoding="utf-8")
        (output_dir / "config.json").write_text(json.dumps(result.config, sort_keys=True), encoding="utf-8")
        (output_dir / "environment.json").write_text(
            json.dumps(
                {
                    "python": sys.version.split()[0],
                    "platform": platform.platform(),
                },
                sort_keys=True,
            ),
            encoding="utf-8",
        )
        (output_dir / "run.log").write_text(result.analysis + "\n", encoding="utf-8")
        (output_dir / "analysis.md").write_text("# Fixture analysis\n\n" + result.analysis + "\n", encoding="utf-8")
        return result


def synthetic_rows(*, seed: int, count: int) -> list[tuple[float, float, int]]:
    rows = []
    state = seed + 17
    for _ in range(count):
        state = (1103515245 * state + 12345) % (2**31)
        x1 = ((state % 2000) / 1000.0) - 1.0
        state = (1103515245 * state + 12345) % (2**31)
        x2 = ((state % 2000) / 1000.0) - 1.0
        label = 1 if x1 + (0.6 * x2) > 0.05 else 0
        rows.append((x1, x2, label))
    return rows


def majority_label(rows: list[tuple[float, float, int]]) -> int:
    positives = sum(label for _, _, label in rows)
    return 1 if positives >= (len(rows) - positives) else 0


def predict(candidate: str, x1: float, x2: float, train: list[tuple[float, float, int]]) -> int:
    if candidate == "majority":
        return majority_label(train)
    if candidate == "linear-threshold":
        return 1 if x1 + (0.6 * x2) > 0.0 else 0
    if candidate == "x1-threshold":
        return 1 if x1 > 0.0 else 0
    raise ValueError(f"unsupported fixture candidate: {candidate}")


def accuracy(predictions: list[int], labels: list[int]) -> float:
    if not labels:
        raise ValueError("labels must not be empty")
    correct = sum(1 for predicted, label in zip(predictions, labels) if predicted == label)
    return correct / len(labels)
