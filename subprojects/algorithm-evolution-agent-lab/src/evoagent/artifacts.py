from __future__ import annotations

from dataclasses import dataclass, field

from evoagent.models import ExperimentRun


@dataclass(frozen=True)
class DeclaredArtifact:
    path: str
    kind: str
    required: bool = True


@dataclass(frozen=True)
class ArtifactCollectionReport:
    run_id: str
    collected_paths: tuple[str, ...]
    missing_required_paths: tuple[str, ...]
    stdout_ref: str
    stderr_ref: str
    environment_metadata: tuple[tuple[str, str], ...] = field(default_factory=tuple)

    @property
    def complete(self) -> bool:
        return not self.missing_required_paths


# cf-atom: CODE-ArtifactCollectionContract
class ArtifactCollectionContract:
    def evaluate(
        self,
        run: ExperimentRun,
        declarations: tuple[DeclaredArtifact, ...],
        *,
        environment_metadata: dict[str, object] | None = None,
    ) -> ArtifactCollectionReport:
        existing = {artifact.path for artifact in run.artifacts if artifact.exists}
        missing_required = tuple(
            declaration.path
            for declaration in declarations
            if declaration.required and declaration.path not in existing
        )
        return ArtifactCollectionReport(
            run_id=run.run_id,
            collected_paths=tuple(sorted(existing)),
            missing_required_paths=missing_required,
            stdout_ref="stdout",
            stderr_ref="stderr",
            environment_metadata=normalize_metadata(environment_metadata or {}),
        )


def normalize_metadata(values: dict[str, object]) -> tuple[tuple[str, str], ...]:
    return tuple(sorted((str(key), str(value)) for key, value in values.items()))
