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
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if args.command[:1] == ["--"]:
        args.command = args.command[1:]
    if not args.command:
        parser.error("a command is required after --")
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        payload = run_gate(
            args.command,
            args.cwd,
            args.log,
            args.timeout,
            drift_threshold_seconds=args.drift_threshold,
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
