#!/usr/bin/env python3
"""Audit live and recorded Rust work against the admitted execution profile."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable


DEFAULT_PROFILE = {
    "rustup_home": "/home/devuser/.cache/codefire-rust-1.97/rustup",
    "cargo_home": "/home/devuser/.cache/codefire-rust-1.97/cargo",
    "target_dir": "/workspace/.cache/cargo-target-shared",
    "max_build_jobs": 2,
    "max_test_threads": 2,
    "incremental": "0",
}

RUST_TOOLS = frozenset({"cargo", "rustc", "clippy-driver"})
PROFILE_KEYS = (
    "RUSTUP_HOME",
    "CARGO_HOME",
    "CARGO_TARGET_DIR",
    "CARGO_BUILD_JOBS",
    "RUST_TEST_THREADS",
    "CARGO_INCREMENTAL",
)
ASSIGNMENT_RE = re.compile(
    r"(?:^|[\s;])(?:export\s+)?"
    r"(RUSTUP_HOME|CARGO_HOME|CARGO_TARGET_DIR|CARGO_BUILD_JOBS|"
    r"RUST_TEST_THREADS|CARGO_INCREMENTAL)="
    r"(?:\"([^\"]*)\"|'([^']*)'|([^\s;&|]+))"
)
CARGO_COMMAND_RE = re.compile(r"(?:^|[\s;&|])(?:[^\s;&|]*/)?cargo(?:\s|$)")


def utc_now() -> str:
    return (
        datetime.now(timezone.utc)
        .isoformat(timespec="microseconds")
        .replace("+00:00", "Z")
    )


def path_within(path: Path, roots: Iterable[Path]) -> bool:
    resolved = path.resolve(strict=False)
    return any(
        resolved == root.resolve(strict=False)
        or root.resolve(strict=False) in resolved.parents
        for root in roots
    )


def resolved_value(value: str, cwd: Path) -> str:
    path = Path(value)
    return str((path if path.is_absolute() else cwd / path).resolve(strict=False))


def parse_positive_limit(
    value: str | None,
    *,
    maximum: int,
    missing_code: str,
    invalid_code: str,
) -> list[str]:
    if value is None:
        return [missing_code]
    try:
        parsed = int(value)
    except ValueError:
        return [invalid_code]
    if parsed < 1 or parsed > maximum:
        return [invalid_code]
    return []


def evaluate_process(
    observation: dict[str, Any],
    profile: dict[str, Any] | None = None,
) -> list[dict[str, Any]]:
    expected = DEFAULT_PROFILE if profile is None else profile
    cwd = Path(str(observation["cwd"]))
    environment = dict(observation.get("environment", {}))
    argv = list(observation.get("argv", []))
    violations: list[dict[str, Any]] = []

    def add(code: str, observed: Any, expected_value: Any) -> None:
        violations.append(
            {
                "code": code,
                "observed": observed,
                "expected": expected_value,
            }
        )

    exact_paths = (
        ("RUSTUP_HOME", "rustup_home", "rustup-home"),
        ("CARGO_HOME", "cargo_home", "cargo-home"),
        ("CARGO_TARGET_DIR", "target_dir", "target-dir"),
    )
    for environment_key, profile_key, code in exact_paths:
        raw = environment.get(environment_key)
        if raw is None:
            add(f"missing-{code}", None, expected[profile_key])
            continue
        observed = resolved_value(str(raw), cwd)
        wanted = str(Path(str(expected[profile_key])).resolve(strict=False))
        if observed != wanted:
            add(f"wrong-{code}", observed, wanted)

    for code in parse_positive_limit(
        environment.get("CARGO_BUILD_JOBS"),
        maximum=int(expected["max_build_jobs"]),
        missing_code="missing-build-jobs",
        invalid_code="unsafe-build-jobs",
    ):
        add(code, environment.get("CARGO_BUILD_JOBS"), expected["max_build_jobs"])
    for code in parse_positive_limit(
        environment.get("RUST_TEST_THREADS"),
        maximum=int(expected["max_test_threads"]),
        missing_code="missing-test-threads",
        invalid_code="unsafe-test-threads",
    ):
        add(
            code,
            environment.get("RUST_TEST_THREADS"),
            expected["max_test_threads"],
        )
    if environment.get("CARGO_INCREMENTAL") != str(expected["incremental"]):
        add(
            "unsafe-incremental",
            environment.get("CARGO_INCREMENTAL"),
            expected["incremental"],
        )

    if argv:
        raw_executable = Path(str(argv[0]))
        expected_rustup = Path(str(expected["rustup_home"])).resolve(strict=False)
        if raw_executable.is_absolute():
            executable = raw_executable.resolve(strict=False)
            if not path_within(executable, [expected_rustup]):
                add(
                    "wrong-toolchain-executable",
                    str(executable),
                    str(expected_rustup),
                )
        if "--out-dir" in argv:
            index = argv.index("--out-dir")
            if index + 1 < len(argv):
                out_dir = Path(resolved_value(str(argv[index + 1]), cwd))
                expected_target = Path(str(expected["target_dir"])).resolve(strict=False)
                if not path_within(out_dir, [expected_target]):
                    add("private-out-dir", str(out_dir), str(expected_target))
    return violations


def read_process_observations(
    proc_root: Path,
    scope_roots: list[Path],
) -> list[dict[str, Any]]:
    observations: list[dict[str, Any]] = []
    for process_dir in sorted(
        (path for path in proc_root.iterdir() if path.name.isdigit()),
        key=lambda path: int(path.name),
    ):
        try:
            argv = [
                value.decode("utf-8", errors="replace")
                for value in (process_dir / "cmdline").read_bytes().split(b"\0")
                if value
            ]
            if not argv or Path(argv[0]).name not in RUST_TOOLS:
                continue
            cwd = Path(os.readlink(process_dir / "cwd")).resolve(strict=False)
            if not path_within(cwd, scope_roots):
                continue
            environment: dict[str, str] = {}
            for item in (process_dir / "environ").read_bytes().split(b"\0"):
                if not item or b"=" not in item:
                    continue
                key, value = item.split(b"=", 1)
                decoded_key = key.decode("utf-8", errors="replace")
                if decoded_key in PROFILE_KEYS:
                    environment[decoded_key] = value.decode(
                        "utf-8", errors="replace"
                    )
        except (FileNotFoundError, PermissionError, ProcessLookupError, OSError):
            continue
        observations.append(
            {
                "pid": int(process_dir.name),
                "tool": Path(argv[0]).name,
                "cwd": str(cwd),
                "argv": argv,
                "environment": environment,
            }
        )
    return observations


def assignment_values(command: str) -> dict[str, str]:
    values: dict[str, str] = {}
    for match in ASSIGNMENT_RE.finditer(command):
        values[match.group(1)] = next(
            value
            for value in (match.group(2), match.group(3), match.group(4))
            if value is not None
        )
    return values


def extract_agent_commands(event_path: Path) -> list[str]:
    commands: list[str] = []
    for line in event_path.read_text(encoding="utf-8").splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue

        def visit(value: Any) -> None:
            if isinstance(value, dict):
                if (
                    value.get("type") == "command_execution"
                    and isinstance(value.get("command"), str)
                ):
                    commands.append(value["command"])
                if (
                    value.get("name") in {"Bash", "Shell"}
                    and isinstance(value.get("input"), dict)
                    and isinstance(value["input"].get("command"), str)
                ):
                    commands.append(value["input"]["command"])
                for nested in value.values():
                    visit(nested)
            elif isinstance(value, list):
                for nested in value:
                    visit(nested)

        visit(event)
    return list(dict.fromkeys(commands))


def audit_recorded_command(
    command: str,
    *,
    cwd: Path,
    profile: dict[str, Any] | None = None,
) -> list[dict[str, Any]]:
    expected = DEFAULT_PROFILE if profile is None else profile
    if CARGO_COMMAND_RE.search(command) is None:
        return []
    assignments = assignment_values(command)
    violations: list[dict[str, Any]] = []

    def add(code: str, observed: Any, expected_value: Any) -> None:
        violations.append(
            {
                "code": code,
                "observed": observed,
                "expected": expected_value,
            }
        )

    for environment_key, profile_key, code in (
        ("RUSTUP_HOME", "rustup_home", "rustup-home"),
        ("CARGO_HOME", "cargo_home", "cargo-home"),
        ("CARGO_TARGET_DIR", "target_dir", "target-dir"),
    ):
        if environment_key not in assignments:
            continue
        observed = resolved_value(assignments[environment_key], cwd)
        wanted = str(Path(str(expected[profile_key])).resolve(strict=False))
        if observed != wanted:
            add(f"recorded-wrong-{code}", observed, wanted)
    for environment_key, maximum, code in (
        ("CARGO_BUILD_JOBS", int(expected["max_build_jobs"]), "build-jobs"),
        (
            "RUST_TEST_THREADS",
            int(expected["max_test_threads"]),
            "test-threads",
        ),
    ):
        if environment_key not in assignments:
            continue
        value = assignments[environment_key]
        try:
            safe = 1 <= int(value) <= maximum
        except ValueError:
            safe = False
        if not safe:
            add(f"recorded-unsafe-{code}", value, maximum)
    if (
        "CARGO_INCREMENTAL" in assignments
        and assignments["CARGO_INCREMENTAL"] != str(expected["incremental"])
    ):
        add(
            "recorded-unsafe-incremental",
            assignments["CARGO_INCREMENTAL"],
            expected["incremental"],
        )
    return violations


def build_result(
    observations: list[dict[str, Any]],
    *,
    event_paths: list[Path],
    event_cwd: Path,
    profile: dict[str, Any] | None = None,
) -> dict[str, Any]:
    expected = dict(DEFAULT_PROFILE if profile is None else profile)
    live: list[dict[str, Any]] = []
    for observation in observations:
        violations = evaluate_process(observation, expected)
        if violations:
            live.append(
                {
                    "pid": observation["pid"],
                    "tool": observation["tool"],
                    "cwd": observation["cwd"],
                    "violations": violations,
                }
            )

    recorded_groups: dict[tuple[str, str], dict[str, Any]] = {}
    recorded_count = 0
    recorded_violation_count = 0
    for event_path in event_paths:
        for command in extract_agent_commands(event_path):
            recorded_count += 1
            violations = audit_recorded_command(
                command, cwd=event_cwd, profile=expected
            )
            if violations:
                recorded_violation_count += 1
                event_name = str(event_path.resolve())
                signature = json.dumps(
                    violations, sort_keys=True, separators=(",", ":")
                )
                key = (event_name, signature)
                group = recorded_groups.setdefault(
                    key,
                    {
                        "events": event_name,
                        "count": 0,
                        "command_sha256": [],
                        "violations": violations,
                    },
                )
                group["count"] += 1
                group["command_sha256"].append(
                    hashlib.sha256(command.encode("utf-8")).hexdigest()
                )

    recorded: list[dict[str, Any]] = []
    for group in recorded_groups.values():
        command_hashes = sorted(set(group.pop("command_sha256")))
        group["command_set_sha256"] = hashlib.sha256(
            ("\n".join(command_hashes) + "\n").encode("ascii")
        ).hexdigest()
        group["command_sha256_samples"] = command_hashes[:5]
        recorded.append(group)
    recorded.sort(
        key=lambda item: (
            item["events"],
            json.dumps(item["violations"], sort_keys=True),
        )
    )

    blockers = sorted(
        {
            violation["code"]
            for item in live + recorded
            for violation in item["violations"]
        }
    )
    return {
        "schema": "codefire.rust_execution_profile_audit.v1",
        "captured_at": utc_now(),
        "decision": "PASS" if not blockers else "BLOCK",
        "profile": expected,
        "live_processes_scanned": len(observations),
        "recorded_commands_scanned": recorded_count,
        "recorded_violation_commands": recorded_violation_count,
        "live_violations": live,
        "recorded_violations": recorded,
        "blockers": blockers,
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
        description=(
            "Fail closed when live or recorded CodeFire Rust work overrides "
            "the admitted shared low-memory execution profile."
        )
    )
    parser.add_argument(
        "--scope-root",
        action="append",
        type=Path,
        default=[],
        help="Process cwd root to audit; repeatable (default: /workspace/worktrees).",
    )
    parser.add_argument(
        "--events",
        action="append",
        type=Path,
        default=[],
        help="Agent JSONL event log to audit for explicit profile overrides.",
    )
    parser.add_argument(
        "--event-cwd",
        type=Path,
        default=Path("/workspace"),
        help="Base for resolving relative paths found in recorded commands.",
    )
    parser.add_argument("--proc-root", type=Path, default=Path("/proc"))
    parser.add_argument("--result", type=Path)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    scope_roots = (
        [Path("/workspace/worktrees")]
        if not args.scope_root
        else [path.resolve() for path in args.scope_root]
    )
    try:
        observations = read_process_observations(args.proc_root, scope_roots)
        result = build_result(
            observations,
            event_paths=args.events,
            event_cwd=args.event_cwd.resolve(),
        )
        if args.result is not None:
            atomic_write_json(args.result, result)
    except (OSError, ValueError, KeyError) as exc:
        print(f"Rust execution-profile audit error: {exc}", file=sys.stderr)
        return 2
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["decision"] == "PASS" else 3


if __name__ == "__main__":
    raise SystemExit(main())
