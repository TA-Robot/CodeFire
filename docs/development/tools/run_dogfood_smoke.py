#!/usr/bin/env python3
"""Run bounded CodeFire dogfood checks with mutation disabled by default."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


COMMAND_RESULT_SCHEMA = "codefire.command_result.v1"


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def contained_directory(root: Path, value: Path) -> Path:
    candidate = value if value.is_absolute() else root / value
    resolved = candidate.resolve(strict=True)
    resolved.relative_to(root)
    if not resolved.is_dir():
        raise ValueError(f"dogfood path is not a directory: {value}")
    return resolved


def executable_file(root: Path, value: Path) -> Path:
    candidate = value if value.is_absolute() else root / value
    if candidate.is_symlink():
        raise ValueError(f"binary path must not be a symlink: {value}")
    resolved = candidate.resolve(strict=True)
    resolved.relative_to(root)
    if not resolved.is_file() or not os.access(resolved, os.X_OK):
        raise ValueError(f"binary is not an executable regular file: {value}")
    return resolved


def smoke_commands(binary: Path, target: Path, allow_mutation: bool) -> list[list[str]]:
    commands = [[str(binary), "status", "--path", str(target), "--json"]]
    if allow_mutation:
        commands.extend(
            [
                [str(binary), "scan", "--path", str(target), "--json"],
                [str(binary), "verify", "--path", str(target), "--json"],
            ]
        )
    return commands


def validate_result(command_name: str, returncode: int, stdout: str, stderr: str) -> dict[str, Any]:
    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError as exc:
        return {
            "command": command_name,
            "ok": False,
            "returncode": returncode,
            "error": f"stdout is not JSON: {exc}",
            "stderr_tail": stderr[-2000:],
        }
    errors = []
    if not isinstance(payload, dict):
        errors.append("result is not an object")
    else:
        if payload.get("schema") != COMMAND_RESULT_SCHEMA:
            errors.append("unexpected command-result schema")
        if payload.get("command") != command_name:
            errors.append("command field does not match invocation")
        if payload.get("exit_code") != returncode:
            errors.append("payload exit_code differs from process return code")
        if payload.get("ok") is not True or returncode != 0:
            errors.append("command did not report success")
    return {
        "command": command_name,
        "ok": not errors,
        "returncode": returncode,
        "errors": errors,
        "result": payload,
        "stderr_tail": stderr[-2000:] if stderr else "",
    }


def run_smoke(
    root: Path,
    binary: Path,
    target: Path,
    *,
    allow_mutation: bool,
    timeout: float,
) -> dict[str, Any]:
    commands = smoke_commands(binary, target, allow_mutation)
    results = []
    started = time.monotonic()
    for command in commands:
        command_name = command[1]
        command_started = time.monotonic()
        try:
            completed = subprocess.run(
                command,
                cwd=root,
                capture_output=True,
                text=True,
                timeout=timeout,
                check=False,
            )
        except subprocess.TimeoutExpired as exc:
            results.append(
                {
                    "command": command_name,
                    "ok": False,
                    "error": f"timeout after {exc.timeout}s",
                }
            )
            break
        result = validate_result(
            command_name,
            completed.returncode,
            completed.stdout,
            completed.stderr,
        )
        result["duration_ms"] = round((time.monotonic() - command_started) * 1000)
        results.append(result)
        if not result["ok"]:
            break
    return {
        "schema": "codefire.dogfood_smoke.v1",
        "ok": len(results) == len(commands) and all(result["ok"] for result in results),
        "mode": "mutating" if allow_mutation else "read-only",
        "binary": binary.relative_to(root).as_posix(),
        "target": target.relative_to(root).as_posix(),
        "duration_ms": round((time.monotonic() - started) * 1000),
        "results": results,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=repository_root())
    parser.add_argument(
        "--binary", type=Path, default=Path("target/release/codefire-rs")
    )
    parser.add_argument(
        "--path", type=Path, default=Path("subprojects/algorithm-evolution-agent-lab")
    )
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument(
        "--allow-mutation",
        action="store_true",
        help="also run scan and verify, which update CodeFire active state",
    )
    args = parser.parse_args()
    try:
        if args.timeout <= 0:
            raise ValueError("timeout must be positive")
        root = args.root.resolve(strict=True)
        binary = executable_file(root, args.binary)
        target = contained_directory(root, args.path)
        payload = run_smoke(
            root,
            binary,
            target,
            allow_mutation=args.allow_mutation,
            timeout=args.timeout,
        )
    except (FileNotFoundError, OSError, ValueError) as exc:
        payload = {
            "schema": "codefire.dogfood_smoke.v1",
            "ok": False,
            "error": str(exc),
        }
    print(json.dumps(payload, indent=2, sort_keys=True))
    return 0 if payload["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
