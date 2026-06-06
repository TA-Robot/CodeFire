from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass, field

from evoagent.models import ExperimentPlan


@dataclass(frozen=True)
class ExperimentConfiguration:
    plan_id: str
    benchmark: str
    baseline: str
    metric: str
    hyperparameters: tuple[tuple[str, str], ...] = field(default_factory=tuple)
    seed: int | None = None
    dataset_version: str = ""
    benchmark_split: str = ""
    code_reference: str = ""
    environment: tuple[tuple[str, str], ...] = field(default_factory=tuple)

    @property
    def fingerprint(self) -> str:
        payload = json.dumps(
            {
                "plan_id": self.plan_id,
                "benchmark": self.benchmark,
                "baseline": self.baseline,
                "metric": self.metric,
                "hyperparameters": self.hyperparameters,
                "seed": self.seed,
                "dataset_version": self.dataset_version,
                "benchmark_split": self.benchmark_split,
                "code_reference": self.code_reference,
                "environment": self.environment,
            },
            sort_keys=True,
        )
        return "cfg_" + hashlib.sha1(payload.encode("utf-8")).hexdigest()[:12]


@dataclass(frozen=True)
class ConfigurationFinding:
    kind: str
    detail: str


# cf-atom: CODE-ConfigurationCapture
class ConfigurationCapture:
    def capture(
        self,
        plan: ExperimentPlan,
        *,
        hyperparameters: dict[str, object] | None = None,
        seed: int | None = None,
        dataset_version: str = "",
        benchmark_split: str = "",
        code_reference: str = "",
        environment: dict[str, object] | None = None,
    ) -> ExperimentConfiguration:
        return ExperimentConfiguration(
            plan_id=plan.uid,
            benchmark=plan.benchmark,
            baseline=plan.baseline,
            metric=plan.metric,
            hyperparameters=normalize_mapping(hyperparameters or {}),
            seed=seed,
            dataset_version=dataset_version,
            benchmark_split=benchmark_split,
            code_reference=code_reference,
            environment=normalize_mapping(environment or {}),
        )

    def validate(self, config: ExperimentConfiguration) -> tuple[ConfigurationFinding, ...]:
        findings: list[ConfigurationFinding] = []
        if config.seed is None:
            findings.append(ConfigurationFinding("missing_seed", "experiment seed should be captured"))
        if not config.dataset_version:
            findings.append(ConfigurationFinding("missing_dataset_version", "dataset version should be captured"))
        if not config.benchmark_split:
            findings.append(ConfigurationFinding("missing_benchmark_split", "benchmark split should be captured"))
        if not config.code_reference:
            findings.append(ConfigurationFinding("missing_code_reference", "code reference should be captured"))
        return tuple(findings)


def normalize_mapping(values: dict[str, object]) -> tuple[tuple[str, str], ...]:
    return tuple(sorted((str(key), str(value)) for key, value in values.items()))
