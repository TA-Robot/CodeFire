from __future__ import annotations

import hashlib
import subprocess
import time
from pathlib import Path

from evoagent.models import ExperimentPlan, ExperimentRun, RunArtifact


# cf-atom: CODE-LocalExperimentRunner
class LocalExperimentRunner:
    def run(
        self,
        plan: ExperimentPlan,
        *,
        cwd: Path,
        command: str | None = None,
        timeout_seconds: float = 60.0,
    ) -> ExperimentRun:
        selected_command = command or plan.command
        if not selected_command:
            raise ValueError("experiment command is required")

        started = time.monotonic()
        try:
            proc = subprocess.run(
                selected_command,
                shell=True,
                cwd=cwd,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=timeout_seconds,
                check=False,
            )
            returncode = proc.returncode
            stdout = proc.stdout
            stderr = proc.stderr
        except subprocess.TimeoutExpired as exc:
            returncode = 124
            stdout = decode_timeout_output(exc.stdout)
            stderr = decode_timeout_output(exc.stderr)
            stderr = "\n".join(part for part in (stderr, f"timeout after {timeout_seconds:g}s") if part)
        duration = time.monotonic() - started
        artifacts = tuple(capture_artifact(cwd, path) for path in plan.artifact_paths)
        return ExperimentRun(
            run_id=run_id_for(plan, selected_command, started),
            plan=plan,
            command=selected_command,
            cwd=cwd,
            returncode=returncode,
            stdout=stdout,
            stderr=stderr,
            duration_seconds=duration,
            artifacts=artifacts,
        )


def capture_artifact(cwd: Path, path: str) -> RunArtifact:
    artifact = cwd / path
    exists = artifact.exists()
    return RunArtifact(
        path=path,
        exists=exists,
        size_bytes=artifact.stat().st_size if exists and artifact.is_file() else 0,
    )


def run_id_for(plan: ExperimentPlan, command: str, started: float) -> str:
    payload = f"{plan.hypothesis.title}|{plan.benchmark}|{plan.metric}|{command}|{started:.9f}"
    return "run_" + hashlib.sha1(payload.encode("utf-8")).hexdigest()[:12]


def decode_timeout_output(value: str | bytes | None) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return value
