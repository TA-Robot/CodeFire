#!/usr/bin/env python3
"""Run one gate without a shell and record monotonic, content-bound evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import signal
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable


def utc_timestamp(wall_ns: int) -> str:
    return datetime.fromtimestamp(wall_ns / 1_000_000_000, timezone.utc).isoformat(
        timespec="microseconds"
    ).replace("+00:00", "Z")


def sha256_file(path: Path) -> tuple[int, str]:
    digest = hashlib.sha256()
    size = 0
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            size += len(chunk)
            digest.update(chunk)
    return size, digest.hexdigest()


def timing_evidence(
    start_wall_ns: int,
    end_wall_ns: int,
    start_monotonic_ns: int,
    end_monotonic_ns: int,
    *,
    drift_threshold_seconds: float,
) -> dict[str, Any]:
    wall_seconds = (end_wall_ns - start_wall_ns) / 1_000_000_000
    monotonic_seconds = (end_monotonic_ns - start_monotonic_ns) / 1_000_000_000
    drift_seconds = wall_seconds - monotonic_seconds
    return {
        "started_at": utc_timestamp(start_wall_ns),
        "ended_at": utc_timestamp(end_wall_ns),
        "duration_seconds": monotonic_seconds,
        "duration_source": "time.monotonic_ns",
        "wall_elapsed_seconds": wall_seconds,
        "wall_minus_monotonic_seconds": drift_seconds,
        "wall_clock_discontinuity": abs(drift_seconds) > drift_threshold_seconds,
        "drift_threshold_seconds": drift_threshold_seconds,
    }


def terminate_process_group(process: subprocess.Popen[bytes]) -> None:
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    try:
        process.wait(timeout=5)
        return
    except subprocess.TimeoutExpired:
        pass
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        return
    process.wait()


def run_gate(
    argv: list[str],
    cwd: Path,
    log_path: Path,
    timeout_seconds: float,
    *,
    drift_threshold_seconds: float = 2.0,
    wall_clock_ns: Callable[[], int] = time.time_ns,
    monotonic_clock_ns: Callable[[], int] = time.monotonic_ns,
) -> dict[str, Any]:
    if not argv:
        raise ValueError("gate argv must not be empty")
    if timeout_seconds <= 0:
        raise ValueError("timeout must be positive")
    cwd = cwd.resolve(strict=True)
    if not cwd.is_dir():
        raise ValueError(f"gate cwd is not a directory: {cwd}")
    log_path = log_path.resolve()
    log_path.parent.mkdir(parents=True, exist_ok=True)

    start_wall_ns = wall_clock_ns()
    start_monotonic_ns = monotonic_clock_ns()
    timed_out = False
    with log_path.open("wb") as log:
        process = subprocess.Popen(
            argv,
            cwd=cwd,
            stdin=subprocess.DEVNULL,
            stdout=log,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        try:
            returncode = process.wait(timeout=timeout_seconds)
        except subprocess.TimeoutExpired:
            timed_out = True
            terminate_process_group(process)
            returncode = 124
        log.flush()
        os.fsync(log.fileno())
    end_monotonic_ns = monotonic_clock_ns()
    end_wall_ns = wall_clock_ns()
    log_bytes, log_sha256 = sha256_file(log_path)

    return {
        "schema": "codefire.timed_gate.v1",
        "argv": argv,
        "cwd": str(cwd),
        "timeout_seconds": timeout_seconds,
        "timed_out": timed_out,
        "exit_code": returncode,
        "status": "passed" if returncode == 0 and not timed_out else "failed",
        **timing_evidence(
            start_wall_ns,
            end_wall_ns,
            start_monotonic_ns,
            end_monotonic_ns,
            drift_threshold_seconds=drift_threshold_seconds,
        ),
        "log": {
            "path": str(log_path),
            "bytes": log_bytes,
            "sha256": log_sha256,
            "complete": True,
        },
    }


TIMED_V2_LIMITS = (
    "A command that mutates and restores bytes during its own execution stays outside "
    "this provenance proof, even when clean-before, clean-after and the index-flag set "
    "are all empty.",
    "This result does not prove honest execution, semantic oracle quality, or that a "
    "test selector chose a nonzero corpus; a validation-specific selection claim remains "
    "report evidence unless a dedicated parser verifies it.",
    "cwd, source.git_root and log.path are machine-specific absolute paths and are "
    "diagnostic only; identity is carried by source.revision, source.tree and the log "
    "digest with its byte count.",
)


def run_gate_v2(
    argv: list[str],
    cwd: Path,
    log_path: Path,
    timeout_seconds: float,
    *,
    result_path: Path,
    drift_threshold_seconds: float = 2.0,
    wall_clock_ns: Callable[[], int] = time.time_ns,
    monotonic_clock_ns: Callable[[], int] = time.monotonic_ns,
) -> dict[str, Any]:
    """Run one gate and bind it to a derived, clean source identity.

    The Git root containing `cwd`, the canonical `HEAD` revision and the `HEAD`
    tree are derived here rather than accepted as strings.  The porcelain status
    is captured immediately before and immediately after the command, and the
    index entries whose tag is not `H` are captured too, so an `assume-unchanged`
    or `skip-worktree` bit cannot hide a modification behind an empty status.

    Logs and results are written outside the source worktree and copied
    byte-identically into the evidence commit.  The required result path and the
    log path are both classified against the derived root; an in-tree path is
    emitted as such and cannot satisfy recurrence acceptance.
    """

    from recurrence_v2_git import GitRepo

    probe = GitRepo(cwd.resolve(strict=True))
    root = Path(probe.toplevel())
    repo = GitRepo(root)
    revision, tree = repo.head_identity()
    status_before = repo.worktree_status()
    payload = run_gate(
        argv,
        cwd,
        log_path,
        timeout_seconds,
        drift_threshold_seconds=drift_threshold_seconds,
        wall_clock_ns=wall_clock_ns,
        monotonic_clock_ns=monotonic_clock_ns,
    )
    status_after = repo.worktree_status()
    index_flagged = repo.index_flagged_entries()
    resolved_log = log_path.resolve()
    try:
        resolved_log.relative_to(root.resolve())
        log_inside = True
    except ValueError:
        log_inside = False
    try:
        result_path.resolve().relative_to(root.resolve())
        result_inside = True
    except ValueError:
        result_inside = False
    payload["schema"] = "codefire.timed_gate.v2"
    payload["source"] = {
        "git_root": str(root.resolve()),
        "revision": revision,
        "tree": tree,
        "derived": "git -C <cwd> rev-parse --show-toplevel HEAD HEAD^{tree}",
    }
    payload["worktree"] = {
        "clean_before": not status_before,
        "clean_after": not status_after,
        "status_before": list(status_before),
        "status_after": list(status_after),
        "index_flagged_entries": list(index_flagged),
        "index_flags_clean": not index_flagged,
        "log_inside_source_worktree": log_inside,
        "result_inside_source_worktree": result_inside,
        "status_command": "git status --porcelain=v1 -z --untracked-files=all",
        "index_command": "git ls-files -v -z",
    }
    payload["limits"] = list(TIMED_V2_LIMITS)
    return payload


def atomic_write_json(path: Path, payload: dict[str, Any]) -> None:
    path = path.resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.tmp-{os.getpid()}")
    temporary.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    os.replace(temporary, path)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run one argv gate and emit monotonic, digest-bound JSON evidence."
    )
    parser.add_argument("--cwd", type=Path, default=Path("."))
    parser.add_argument("--log", type=Path, required=True)
    parser.add_argument("--result", type=Path)
    parser.add_argument("--timeout", type=float, required=True)
    parser.add_argument("--drift-threshold", type=float, default=2.0)
    parser.add_argument(
        "--emit",
        choices=("v1", "v2"),
        default="v1",
        help=(
            "v1 keeps the historical result shape; v2 additionally derives the source "
            "revision/tree and the clean-before, clean-after and index-flag states that a "
            "recurrence v2 acceptance requires"
        ),
    )
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if args.command[:1] == ["--"]:
        args.command = args.command[1:]
    if not args.command:
        parser.error("a command is required after --")
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    runner = run_gate_v2 if args.emit == "v2" else run_gate
    try:
        if args.emit == "v2" and args.result is None:
            raise ValueError(
                "--emit v2 requires --result so the runner can prove that both the "
                "log and result were staged outside the source worktree"
            )
        runner_kwargs = {"drift_threshold_seconds": args.drift_threshold}
        if args.emit == "v2":
            runner_kwargs["result_path"] = args.result
        payload = runner(
            args.command,
            args.cwd,
            args.log,
            args.timeout,
            **runner_kwargs,
        )
        if args.result is not None:
            atomic_write_json(args.result, payload)
    except (OSError, ValueError) as exc:
        print(f"timed gate error: {exc}", file=sys.stderr)
        return 2
    print(json.dumps(payload, indent=2, sort_keys=True))
    return int(payload["exit_code"])


if __name__ == "__main__":
    raise SystemExit(main())
