#!/usr/bin/env python3
"""Run the complete local, non-mutating CodeFire AI-development gate."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def run_json_check(root: Path, name: str, command: list[str], timeout: float) -> dict[str, Any]:
    try:
        result = subprocess.run(
            command,
            cwd=root,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
            env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"},
        )
    except subprocess.TimeoutExpired as exc:
        return {"name": name, "ok": False, "returncode": None, "error": f"timeout after {exc.timeout}s"}
    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError:
        return {
            "name": name,
            "ok": False,
            "returncode": result.returncode,
            "error": "check did not return JSON",
            "stdout": result.stdout[-4000:],
            "stderr": result.stderr[-4000:],
        }
    return {
        "name": name,
        "ok": result.returncode == 0 and payload.get("ok") is True,
        "returncode": result.returncode,
        "payload": payload,
        "stderr": result.stderr[-4000:] if result.stderr else "",
    }


def run_tool_tests(root: Path, timeout: float) -> dict[str, Any]:
    command = [sys.executable, "-m", "unittest", "docs/development/tools/test_ai_tools.py"]
    try:
        result = subprocess.run(
            command,
            cwd=root,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
            env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"},
        )
    except subprocess.TimeoutExpired as exc:
        return {"name": "tool_tests", "ok": False, "returncode": None, "error": f"timeout after {exc.timeout}s"}
    return {
        "name": "tool_tests",
        "ok": result.returncode == 0,
        "returncode": result.returncode,
        "stdout": result.stdout[-4000:],
        "stderr": result.stderr[-4000:],
    }


def run_capability_probe(root: Path, timeout: float) -> dict[str, Any]:
    return run_json_check(
        root,
        "validation_capabilities",
        [sys.executable, "docs/development/tools/probe_validation_capabilities.py", "--json"],
        timeout,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=repository_root())
    parser.add_argument("--skip-cli", action="store_true", help="Skip local Codex catalog inspection.")
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    checks = [
        run_json_check(
            root,
            "acceleration_docs",
            [sys.executable, "docs/development/tools/check_ai_acceleration_docs.py", "--json"],
            args.timeout,
        ),
        run_json_check(
            root,
            "contracts",
            [sys.executable, "docs/development/tools/check_ai_contracts.py", "--json"],
            args.timeout,
        ),
        run_json_check(
            root,
            "project_docs",
            [sys.executable, "docs/development/tools/check_project_docs.py", "--json"],
            args.timeout,
        ),
        run_json_check(
            root,
            "source_docs",
            [sys.executable, "docs/development/tools/check_source_docs.py", "--json"],
            args.timeout,
        ),
        run_json_check(
            root,
            "runtime",
            [
                sys.executable,
                "docs/development/tools/validate_ai_runtime.py",
                "--json",
                *(["--skip-cli"] if args.skip_cli else []),
            ],
            args.timeout,
        ),
        run_tool_tests(root, args.timeout),
        run_capability_probe(root, args.timeout),
    ]
    payload = {
        "schema": "codefire.ai_development_check.v1",
        "ok": all(check["ok"] for check in checks),
        "checks": checks,
    }
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        for check in checks:
            state = "PASS" if check["ok"] else "FAIL"
            print(f"{state} {check['name']}")
        print("AI development gate passed." if payload["ok"] else "AI development gate failed.")
    return 0 if payload["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
