from __future__ import annotations

from dataclasses import dataclass, field

from evoagent.models import ExperimentRun


@dataclass(frozen=True)
class RunIdentityRecord:
    run_id: str
    plan_id: str
    config_ref: str
    log_refs: tuple[str, ...] = field(default_factory=tuple)
    artifact_refs: tuple[str, ...] = field(default_factory=tuple)
    evidence_refs: tuple[str, ...] = field(default_factory=tuple)
    report_refs: tuple[str, ...] = field(default_factory=tuple)


# cf-atom: CODE-RunIdentityLedger
class RunIdentityLedger:
    def __init__(self):
        self._records: dict[str, RunIdentityRecord] = {}

    def record(
        self,
        run: ExperimentRun,
        *,
        config_ref: str,
        evidence_refs: tuple[str, ...] = (),
        report_refs: tuple[str, ...] = (),
    ) -> RunIdentityRecord:
        if not config_ref:
            raise ValueError("config_ref is required")
        if run.run_id in self._records:
            raise ValueError(f"duplicate run_id: {run.run_id}")
        record = RunIdentityRecord(
            run_id=run.run_id,
            plan_id=run.plan.uid,
            config_ref=config_ref,
            log_refs=("stdout", "stderr"),
            artifact_refs=tuple(artifact.path for artifact in run.artifacts),
            evidence_refs=evidence_refs,
            report_refs=report_refs,
        )
        self._records[run.run_id] = record
        return record

    def get(self, run_id: str) -> RunIdentityRecord:
        return self._records[run_id]

    def for_plan(self, plan_id: str) -> tuple[RunIdentityRecord, ...]:
        return tuple(record for record in self._records.values() if record.plan_id == plan_id)

    @property
    def records(self) -> tuple[RunIdentityRecord, ...]:
        return tuple(sorted(self._records.values(), key=lambda record: record.run_id))
