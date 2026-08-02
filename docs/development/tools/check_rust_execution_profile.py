#!/usr/bin/env python3
"""Audit live and recorded Rust work against the admitted execution profile."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shlex
import stat
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable

sys.dont_write_bytecode = True
sys.pycache_prefix = "/dev/null"

from run_timed_gate import atomic_write_json as publish_atomic_json


DEFAULT_PROFILE = {
    "rustup_home": "/home/devuser/.cache/codefire-rust-1.97/rustup",
    "cargo_home": "/home/devuser/.cache/codefire-rust-1.97/cargo",
    "rustup_toolchain": "1.97.0-x86_64-unknown-linux-gnu",
    "target_dir": "/workspace/.cache/cargo-target-shared",
    "evidence_root": "/workspace/.cache/codefire-gate-evidence",
    "snapshot_root": "/workspace/.cache/codefire-source-snapshots",
    "max_build_jobs": 2,
    "max_test_threads": 2,
    "incremental": "0",
}

RUST_TOOLS = frozenset(
    {
        "cargo",
        "cargo-clippy",
        "cargo-fmt",
        "rustc",
        "rustdoc",
        "rustfmt",
        "clippy-driver",
    }
)
SOURCE_KEY_TARGET_DIRECTORY = "source-keys"
SOURCE_KEY_PATTERN = re.compile(r"^[0-9a-f]{64}$")
SOURCE_KEY_GATE_PROVENANCE = "codefire.source_keyed_cargo_gate.v1"
SOURCE_KEY_MARKER_NAME = ".codefire-source-key.json"
SOURCE_KEY_MARKER_SCHEMA = "codefire.cargo_target_namespace.v1"
SOURCE_KEY_RUN_STATE_NAME = ".codefire-source-key-run-state.json"
SOURCE_KEY_RUN_STATE_SCHEMA = "codefire.cargo_namespace_run_state.v1"
LIVE_SOURCE_EVIDENCE_SCHEMA = "codefire.live_source_key_runtime_evidence.v1"
SOURCE_GATE_PYTHON = Path(sys.executable).resolve(strict=True)
SOURCE_GATE_PYTHON_FLAGS = [
    "-I",
    "-S",
    "-B",
    "-X",
    "pycache_prefix=/dev/null",
]
SOURCE_GATE_LAUNCHER_OVERRIDE_NAMES = frozenset(
    {
        "BASH_ENV",
        "ENV",
        "LD_AUDIT",
        "LD_LIBRARY_PATH",
        "LD_PRELOAD",
        "PYTHONHOME",
        "PYTHONINSPECT",
        "PYTHONPATH",
        "PYTHONSTARTUP",
    }
)
PROFILE_KEYS = (
    "RUSTUP_HOME",
    "CARGO_HOME",
    "RUSTUP_TOOLCHAIN",
    "CARGO_TARGET_DIR",
    "CARGO_BUILD_JOBS",
    "RUST_TEST_THREADS",
    "CARGO_INCREMENTAL",
    "CODEFIRE_CARGO_GATE_PROVENANCE",
    "CODEFIRE_CARGO_GATE_SOURCE_KEY",
)
RUST_COMMAND_RE = re.compile(
    r"(?:^|[\s;&|])(?:[^\s;&|]*/)?"
    r"(?:cargo|cargo-clippy|cargo-fmt|rustc|rustdoc|rustfmt|clippy-driver)"
    r"(?:\s|$)"
)


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


def target_expectation(profile: dict[str, Any]) -> str:
    root = Path(str(profile["target_dir"])).resolve(strict=False)
    return f"{root} or {root}/{SOURCE_KEY_TARGET_DIRECTORY}/<64-hex-source-key>"


def target_dir_is_admitted(
    value: str,
    *,
    cwd: Path,
    profile: dict[str, Any] | None = None,
    allow_shared_root: bool = True,
) -> bool:
    """Accept the shared root or one canonical source-key namespace below it."""

    expected = DEFAULT_PROFILE if profile is None else profile
    root = Path(str(expected["target_dir"])).resolve(strict=False)
    observed = Path(resolved_value(value, cwd))
    if observed == root:
        return allow_shared_root
    try:
        relative = observed.relative_to(root)
    except ValueError:
        return False
    return (
        len(relative.parts) == 2
        and relative.parts[0] == SOURCE_KEY_TARGET_DIRECTORY
        and SOURCE_KEY_PATTERN.fullmatch(relative.parts[1]) is not None
    )


def canonical_control_bytes(payload: dict[str, Any]) -> bytes:
    return (
        json.dumps(
            payload,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8")
        + b"\n"
    )


def mountinfo_path(value: str) -> str:
    return re.sub(
        r"\\([0-7]{3})",
        lambda match: chr(int(match.group(1), 8)),
        value,
    )


def read_only_mount_points(raw: str) -> set[str]:
    mounts: set[str] = set()
    for line in raw.splitlines():
        fields = line.split()
        if len(fields) < 7 or "-" not in fields:
            continue
        separator = fields.index("-")
        if separator < 6:
            continue
        options = set(fields[5].split(","))
        if "ro" in options:
            mounts.add(mountinfo_path(fields[4]))
    return mounts


def read_runtime_control(directory: int, name: str) -> bytes:
    descriptor = os.open(
        name,
        os.O_RDONLY
        | os.O_NONBLOCK
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
        dir_fd=directory,
    )
    try:
        before = os.fstat(descriptor)
        if (
            not stat.S_ISREG(before.st_mode)
            or before.st_nlink != 1
            or before.st_size > 1024 * 1024
        ):
            raise ValueError("runtime control is not a bounded regular file")
        chunks: list[bytes] = []
        size = 0
        while True:
            chunk = os.read(descriptor, min(64 * 1024, 1024 * 1024 + 1 - size))
            if not chunk:
                break
            chunks.append(chunk)
            size += len(chunk)
            if size > 1024 * 1024:
                raise ValueError("runtime control exceeds size bound")
        after = os.fstat(descriptor)
        identity_fields = (
            "st_dev",
            "st_ino",
            "st_mode",
            "st_nlink",
            "st_size",
            "st_mtime_ns",
            "st_ctime_ns",
        )
        if any(
            getattr(before, field) != getattr(after, field)
            for field in identity_fields
        ):
            raise ValueError("runtime control changed while read")
        raw = b"".join(chunks)
        if len(raw) != before.st_size:
            raise ValueError("runtime control size changed while read")
        return raw
    finally:
        os.close(descriptor)


def open_process_directory(process_dir: Path, target: Path) -> int:
    if not target.is_absolute():
        raise ValueError("runtime target must be absolute")
    descriptor = os.open(
        process_dir / "root",
        os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC,
    )
    try:
        for component in target.parts[1:]:
            child = os.open(
                component,
                os.O_RDONLY
                | os.O_DIRECTORY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
                dir_fd=descriptor,
            )
            os.close(descriptor)
            descriptor = child
        return descriptor
    except BaseException:
        os.close(descriptor)
        raise


def unverified_runtime_source_evidence(reason: str) -> dict[str, Any]:
    return {
        "schema": LIVE_SOURCE_EVIDENCE_SCHEMA,
        "status": "unverified",
        "reason": reason,
        "cryptographic_attestation": False,
    }


def capture_runtime_source_evidence(
    process_dir: Path,
    *,
    target: Path,
    source_key: str,
) -> dict[str, Any]:
    """Corroborate source binding structurally inside one process mount namespace.

    The evidence is deliberately not called cryptographic attestation.  It
    proves only that the observed namespace exposes the gate's canonical
    RUNNING controls as read-only mount points while the process is sampled.
    """

    try:
        directory = open_process_directory(process_dir, target)
        try:
            marker_raw = read_runtime_control(
                directory, SOURCE_KEY_MARKER_NAME
            )
            run_state_raw = read_runtime_control(
                directory, SOURCE_KEY_RUN_STATE_NAME
            )
        finally:
            os.close(directory)
        marker = json.loads(marker_raw)
        run_state = json.loads(run_state_raw)
        if (
            not isinstance(marker, dict)
            or marker.get("schema") != SOURCE_KEY_MARKER_SCHEMA
            or marker.get("source_key") != source_key
            or marker.get("target_namespace") != str(target)
            or not isinstance(marker.get("source_key_payload"), dict)
            or marker_raw != canonical_control_bytes(marker)
        ):
            return unverified_runtime_source_evidence("marker-mismatch")
        execution_key = (
            run_state.get("execution_key")
            if isinstance(run_state, dict)
            else None
        )
        if (
            not isinstance(run_state, dict)
            or run_state.get("schema") != SOURCE_KEY_RUN_STATE_SCHEMA
            or run_state.get("source_key") != source_key
            or run_state.get("state") != "RUNNING"
            or not isinstance(execution_key, str)
            or SOURCE_KEY_PATTERN.fullmatch(execution_key) is None
            or run_state_raw != canonical_control_bytes(run_state)
        ):
            return unverified_runtime_source_evidence("run-state-mismatch")
        mountinfo = (process_dir / "mountinfo").read_text(
            encoding="utf-8", errors="strict"
        )
        read_only = read_only_mount_points(mountinfo)
        required_mounts = {
            str(target / SOURCE_KEY_MARKER_NAME),
            str(target / SOURCE_KEY_RUN_STATE_NAME),
        }
        if not required_mounts.issubset(read_only):
            return unverified_runtime_source_evidence(
                "controls-not-read-only-mounts"
            )
        return {
            "schema": LIVE_SOURCE_EVIDENCE_SCHEMA,
            "status": "verified-structural",
            "source_key": source_key,
            "run_state": "RUNNING",
            "execution_key": execution_key,
            "marker_sha256": hashlib.sha256(marker_raw).hexdigest(),
            "run_state_sha256": hashlib.sha256(run_state_raw).hexdigest(),
            "control_files_read_only": True,
            "cryptographic_attestation": False,
        }
    except (
        FileNotFoundError,
        PermissionError,
        ProcessLookupError,
        OSError,
        UnicodeError,
        ValueError,
        json.JSONDecodeError,
    ):
        return unverified_runtime_source_evidence(
            "process-runtime-controls-unavailable"
        )


def runtime_source_evidence_is_verified(
    evidence: Any,
    *,
    source_key: str,
) -> bool:
    return (
        isinstance(evidence, dict)
        and evidence.get("schema") == LIVE_SOURCE_EVIDENCE_SCHEMA
        and evidence.get("status") == "verified-structural"
        and evidence.get("source_key") == source_key
        and evidence.get("run_state") == "RUNNING"
        and isinstance(evidence.get("execution_key"), str)
        and SOURCE_KEY_PATTERN.fullmatch(evidence["execution_key"]) is not None
        and isinstance(evidence.get("marker_sha256"), str)
        and SOURCE_KEY_PATTERN.fullmatch(evidence["marker_sha256"]) is not None
        and isinstance(evidence.get("run_state_sha256"), str)
        and SOURCE_KEY_PATTERN.fullmatch(evidence["run_state_sha256"]) is not None
        and evidence.get("control_files_read_only") is True
        and evidence.get("cryptographic_attestation") is False
    )


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

    raw_toolchain = environment.get("RUSTUP_TOOLCHAIN")
    expected_toolchain = str(expected["rustup_toolchain"])
    if raw_toolchain is None:
        add("missing-rustup-toolchain", None, expected_toolchain)
    elif raw_toolchain != expected_toolchain:
        add("wrong-rustup-toolchain", raw_toolchain, expected_toolchain)

    raw_target = environment.get("CARGO_TARGET_DIR")
    if raw_target is None:
        add("missing-target-dir", None, target_expectation(expected))
        observed_target = Path(str(expected["target_dir"])).resolve(strict=False)
        target_admitted = False
    else:
        observed_target = Path(resolved_value(str(raw_target), cwd))
        target_admitted = target_dir_is_admitted(
            str(raw_target),
            cwd=cwd,
            profile=expected,
            allow_shared_root=False,
        )
        if not target_admitted:
            code = (
                "unkeyed-live-target"
                if observed_target
                == Path(str(expected["target_dir"])).resolve(strict=False)
                else "wrong-target-dir"
            )
            add(
                code,
                str(observed_target),
                (
                    f"{Path(str(expected['target_dir'])).resolve(strict=False)}/"
                    f"{SOURCE_KEY_TARGET_DIRECTORY}/<64-hex-source-key>"
                ),
            )
    source_key = (
        observed_target.name
        if target_admitted
        and observed_target.parent.name == SOURCE_KEY_TARGET_DIRECTORY
        else None
    )
    if (
        environment.get("CODEFIRE_CARGO_GATE_PROVENANCE")
        != SOURCE_KEY_GATE_PROVENANCE
    ):
        add(
            "missing-or-wrong-source-key-gate-provenance",
            environment.get("CODEFIRE_CARGO_GATE_PROVENANCE"),
            SOURCE_KEY_GATE_PROVENANCE,
        )
    if (
        source_key is None
        or environment.get("CODEFIRE_CARGO_GATE_SOURCE_KEY") != source_key
    ):
        add(
            "missing-or-wrong-source-key-gate-key",
            environment.get("CODEFIRE_CARGO_GATE_SOURCE_KEY"),
            source_key,
        )
    if (
        target_admitted
        and observation.get("pid") is not None
        and not runtime_source_evidence_is_verified(
            observation.get("runtime_source_evidence"),
            source_key=source_key or "",
        )
    ):
        evidence = observation.get("runtime_source_evidence")
        observed_evidence = (
            {
                "status": evidence.get("status"),
                "reason": evidence.get("reason"),
            }
            if isinstance(evidence, dict)
            else None
        )
        add(
            "unverified-live-source-key-runtime-evidence",
            observed_evidence,
            (
                "structurally verified read-only RUNNING controls in the "
                "sampled process mount namespace; this is not "
                "cryptographic attestation"
            ),
        )

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
        raw_executable = Path(
            str(observation.get("executable", argv[0]))
        )
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
                expected_target = (
                    observed_target
                    if target_admitted
                    else Path(str(expected["target_dir"])).resolve(strict=False)
                )
                if not path_within(out_dir, [expected_target]):
                    add("private-out-dir", str(out_dir), str(expected_target))
    return violations


def read_process_observations(
    proc_root: Path,
    scope_roots: list[Path],
    profile: dict[str, Any] | None = None,
) -> list[dict[str, Any]]:
    expected = DEFAULT_PROFILE if profile is None else profile
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
            if not argv:
                continue
            executable_raw = os.readlink(process_dir / "exe")
            executable = Path(executable_raw).resolve(strict=False)
            argv_tool = Path(argv[0]).name
            executable_tool = executable.name
            if (
                argv_tool not in RUST_TOOLS
                and executable_tool not in RUST_TOOLS
            ):
                continue
            tool = (
                executable_tool
                if executable_tool in RUST_TOOLS
                else argv_tool
            )
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
            target_raw = environment.get("CARGO_TARGET_DIR")
            source_key = environment.get("CODEFIRE_CARGO_GATE_SOURCE_KEY")
            runtime_evidence = unverified_runtime_source_evidence(
                "missing-or-invalid-source-key-target"
            )
            if (
                target_raw is not None
                and source_key is not None
                and target_dir_is_admitted(
                    target_raw,
                    cwd=cwd,
                    profile=expected,
                    allow_shared_root=False,
                )
            ):
                target = Path(resolved_value(target_raw, cwd))
                if source_key == target.name:
                    runtime_evidence = capture_runtime_source_evidence(
                        process_dir,
                        target=target,
                        source_key=source_key,
                    )
        except (FileNotFoundError, PermissionError, ProcessLookupError, OSError):
            continue
        observations.append(
            {
                "pid": int(process_dir.name),
                "tool": tool,
                "executable": str(executable),
                "cwd": str(cwd),
                "argv": argv,
                "environment": environment,
                "runtime_source_evidence": runtime_evidence,
            }
        )
    return observations


SHELL_ASSIGNMENT_TOKEN_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
PYTHON_EXECUTABLE_RE = re.compile(r"^python(?:[0-9]+(?:\.[0-9]+)*)?$")
RECORDED_SHELL_DISPATCHERS = frozenset(
    {"bash", "dash", "ksh", "sh", "zsh"}
)
RECORDED_CODE_INTERPRETERS = frozenset(
    {"perl", "perl5", "ruby"}
)
PROFILE_KEY_SET = frozenset(PROFILE_KEYS)
SHELL_SUBSTITUTION_RE = re.compile(r"\$\(|`|<\(|>\(")


def assignment_token_value(token: str) -> tuple[str, str] | None:
    if "=" not in token:
        return None
    key, value = token.split("=", 1)
    if key not in PROFILE_KEY_SET:
        return None
    return key, value


def shell_segments(command: str) -> list[list[str]]:
    lexer = shlex.shlex(
        command,
        posix=True,
        punctuation_chars="();&|",
    )
    lexer.whitespace_split = True
    lexer.commenters = ""
    segments: list[list[str]] = []
    current: list[str] = []
    for token in lexer:
        if token and all(character in "();&|" for character in token):
            if current:
                segments.append(current)
                current = []
            continue
        current.append(token)
    if current:
        segments.append(current)
    return segments


def shell_has_command_payload(tokens: list[str], index: int) -> bool:
    """Return whether a known shell consumes a command string via ``-c``."""

    for item in tokens[index + 1 :]:
        if item == "--":
            return False
        if (
            item.startswith("-")
            and item != "-"
            and not item.startswith("--")
            and "c" in item[1:]
        ):
            return True
    return False


def is_recorded_code_interpreter(executable: str) -> bool:
    return (
        executable in RECORDED_CODE_INTERPRETERS
        or PYTHON_EXECUTABLE_RE.fullmatch(executable) is not None
    )


def interpreter_has_code_payload(tokens: list[str], index: int) -> bool:
    executable = Path(tokens[index]).name
    perl_or_ruby = executable in RECORDED_CODE_INTERPRETERS
    for item in tokens[index + 1 :]:
        if item == "--":
            return False
        if item in {"-c", "-e"} or item.startswith(("-c", "-e")):
            return True
        if (
            perl_or_ruby
            and item.startswith("-")
            and item != "-"
            and not item.startswith("--")
            and "e" in item[1:]
        ):
            return True
    return False


def executable_index(tokens: list[str]) -> int | None:
    index = 0
    while index < len(tokens) and SHELL_ASSIGNMENT_TOKEN_RE.match(
        tokens[index]
    ):
        index += 1
    while index < len(tokens):
        wrapper = Path(tokens[index]).name
        if wrapper in {
            "!",
            "{",
            "do",
            "elif",
            "else",
            "if",
            "then",
            "until",
            "while",
        }:
            index += 1
            continue
        if wrapper == "command":
            index += 1
            while index < len(tokens):
                item = tokens[index]
                if item == "--":
                    index += 1
                    break
                if item.startswith("-"):
                    index += 1
                    continue
                break
            continue
        if wrapper == "exec":
            index += 1
            while index < len(tokens):
                item = tokens[index]
                if item == "--":
                    index += 1
                    break
                if item in {"-a", "--argv0"}:
                    index += 2
                    continue
                if item.startswith("--argv0=") or (
                    item.startswith("-a") and item != "-a"
                ):
                    index += 1
                    continue
                if item.startswith("-"):
                    index += 1
                    continue
                break
            continue
        if wrapper in {"nohup", "setsid"}:
            index += 1
            while index < len(tokens):
                item = tokens[index]
                if item == "--":
                    index += 1
                    break
                if item.startswith("-"):
                    index += 1
                    continue
                break
            continue
        if wrapper == "time":
            index += 1
            while index < len(tokens):
                item = tokens[index]
                if item == "--":
                    index += 1
                    break
                if item in {"-o", "--output", "-f", "--format"}:
                    index += 2
                    continue
                if item.startswith(("--output=", "--format=")):
                    index += 1
                    continue
                if item.startswith("-"):
                    index += 1
                    continue
                break
            continue
        if wrapper == "env":
            index += 1
            while index < len(tokens):
                item = tokens[index]
                if item == "--":
                    index += 1
                    break
                if SHELL_ASSIGNMENT_TOKEN_RE.match(item):
                    index += 1
                    continue
                if item in {
                    "-u",
                    "--unset",
                    "-C",
                    "--chdir",
                    "-a",
                    "--argv0",
                }:
                    index += 2
                    continue
                if item.startswith(
                    (
                        "--unset=",
                        "--chdir=",
                        "--argv0=",
                    )
                ) or (item.startswith("-a") and item != "-a"):
                    index += 1
                    continue
                if item.startswith("-"):
                    index += 1
                    continue
                break
            continue
        if wrapper == "timeout":
            index += 1
            while index < len(tokens):
                item = tokens[index]
                if item == "--":
                    index += 1
                    break
                if item in {
                    "-k",
                    "--kill-after",
                    "-s",
                    "--signal",
                }:
                    index += 2
                    continue
                if item.startswith(("--kill-after=", "--signal=")):
                    index += 1
                    continue
                if item.startswith("-"):
                    index += 1
                    continue
                break
            if index < len(tokens):
                index += 1
            continue
        if wrapper == "nice":
            index += 1
            while index < len(tokens):
                item = tokens[index]
                if item == "--":
                    index += 1
                    break
                if item in {"-n", "--adjustment"}:
                    index += 2
                    continue
                if item.startswith("--adjustment="):
                    index += 1
                    continue
                if item.startswith("-"):
                    index += 1
                    continue
                break
            continue
        break
    return index if index < len(tokens) else None


def source_gate_segment_is_attested(
    tokens: list[str],
    *,
    cwd: Path,
) -> bool:
    if any(SHELL_SUBSTITUTION_RE.search(token) for token in tokens):
        return False
    index = executable_index(tokens)
    if index is None:
        return False
    python = Path(tokens[index])
    if (
        not python.is_absolute()
        or Path(os.path.abspath(python)) != SOURCE_GATE_PYTHON
    ):
        return False
    for token in tokens[:index]:
        if (
            SHELL_ASSIGNMENT_TOKEN_RE.match(token)
            and token.split("=", 1)[0]
            in SOURCE_GATE_LAUNCHER_OVERRIDE_NAMES
        ):
            return False
    if any(
        token.split("=", 1)[0] in SOURCE_GATE_LAUNCHER_OVERRIDE_NAMES
        for token in tokens
        if SHELL_ASSIGNMENT_TOKEN_RE.match(token)
    ):
        return False
    expected_script = Path(__file__).with_name(
        "run_source_keyed_cargo_gate.py"
    ).resolve(strict=False)
    script_index: int | None = None
    for candidate_index in range(index + 1, len(tokens)):
        candidate = Path(tokens[candidate_index])
        lexical = (
            candidate if candidate.is_absolute() else cwd / candidate
        )
        absolute_lexical = Path(os.path.abspath(lexical))
        if absolute_lexical == expected_script:
            script_index = candidate_index
            break
    if script_index is None:
        return False
    if (
        tokens[index + 1 : script_index]
        != SOURCE_GATE_PYTHON_FLAGS
    ):
        return False
    try:
        separator = tokens.index("--", script_index + 1)
    except ValueError:
        return False
    return (
        separator + 1 < len(tokens)
        and Path(tokens[separator + 1]).name in RUST_TOOLS
    )


def recorded_rust_invocations(
    command: str,
    *,
    cwd: Path,
    depth: int = 0,
) -> list[str]:
    """Classify commands at the trusted recorded-command boundary.

    Known shell and interpreter code strings are unresolved regardless of
    their apparent content because this parser does not prove those language
    semantics. This is not an execve monitor for arbitrary executable files,
    script files, or descendants they may launch.
    """

    if depth > 2:
        return ["unparsed-literal"] if RUST_COMMAND_RE.search(command) else []
    try:
        segments = shell_segments(command)
    except ValueError:
        return ["unparsed-literal"] if RUST_COMMAND_RE.search(command) else []
    invocations: list[str] = []
    for tokens in segments:
        index = executable_index(tokens)
        if index is None:
            continue
        executable = Path(tokens[index]).name
        if "$" in tokens[index] or "`" in tokens[index]:
            invocations.append("unresolved-dynamic-dispatch")
            continue
        if executable in RUST_TOOLS:
            invocations.append("direct")
            continue
        if source_gate_segment_is_attested(tokens, cwd=cwd):
            invocations.append("source-keyed-gate")
            continue
        if (
            executable in RECORDED_SHELL_DISPATCHERS
            and shell_has_command_payload(tokens, index)
        ):
            invocations.append("unresolved-dynamic-dispatch")
            continue
        if (
            is_recorded_code_interpreter(executable)
            and interpreter_has_code_payload(tokens, index)
        ):
            invocations.append("unresolved-dynamic-dispatch")
            continue
        if (
            any(
                Path(token).name == "run_source_keyed_cargo_gate.py"
                for token in tokens
            )
            and "--" in tokens
        ):
            separator = tokens.index("--")
            if any(
                Path(token).name in RUST_TOOLS
                for token in tokens[separator + 1 :]
            ):
                invocations.append("unattested-source-gate")
                continue
        if (
            PYTHON_EXECUTABLE_RE.fullmatch(executable) is not None
            and "--" in tokens[index + 1 :]
        ):
            separator = tokens.index("--", index + 1)
            if any(
                Path(token).name in RUST_TOOLS
                for token in tokens[separator + 1 :]
            ):
                invocations.append("unattested-python-rust-wrapper")
                continue
        if any(Path(token).name in RUST_TOOLS for token in tokens):
            # Unknown wrapper option arity must not hide a literal Rust
            # executable. Only the exact source-gate form above is attested.
            invocations.append("unparsed-literal")
    return invocations


def scoped_recorded_assignments(command: str) -> dict[str, str]:
    """Return assignments that actually scope every parsed direct Rust command."""

    try:
        segments = shell_segments(command)
    except ValueError:
        return {}
    exported: dict[str, str] = {}
    direct_environments: list[dict[str, str]] = []
    for tokens in segments:
        if tokens[:1] == ["export"]:
            for token in tokens[1:]:
                parsed = assignment_token_value(token)
                if parsed is not None:
                    exported[parsed[0]] = parsed[1]
            continue
        index = executable_index(tokens)
        if index is None or Path(tokens[index]).name not in RUST_TOOLS:
            continue
        environment = dict(exported)
        for token in tokens[:index]:
            parsed = assignment_token_value(token)
            if parsed is not None:
                environment[parsed[0]] = parsed[1]
        direct_environments.append(environment)
    if not direct_environments:
        return {}
    common_keys = set(direct_environments[0])
    for environment in direct_environments[1:]:
        common_keys.intersection_update(environment)
    return {
        key: direct_environments[0][key]
        for key in common_keys
        if all(
            environment[key] == direct_environments[0][key]
            for environment in direct_environments[1:]
        )
    }


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
    invocations = recorded_rust_invocations(command, cwd=cwd)
    if not invocations or all(
        invocation == "source-keyed-gate" for invocation in invocations
    ):
        return []
    assignments = scoped_recorded_assignments(command)
    violations: list[dict[str, Any]] = []

    def add(code: str, observed: Any, expected_value: Any) -> None:
        violations.append(
            {
                "code": code,
                "observed": observed,
                "expected": expected_value,
            }
        )

    non_source_gated = sorted(
        {
            invocation
            for invocation in invocations
            if invocation != "source-keyed-gate"
        }
    )
    if non_source_gated:
        add(
            "recorded-direct-rust-not-source-gated",
            non_source_gated,
            (
                f"{SOURCE_GATE_PYTHON} "
                f"{' '.join(SOURCE_GATE_PYTHON_FLAGS)} "
                "<exact run_source_keyed_cargo_gate.py> "
                "... -- <Rust tool>"
            ),
        )
    if "unresolved-dynamic-dispatch" in non_source_gated:
        add(
            "recorded-unresolved-dynamic-dispatch",
            "shell/interpreter command string",
            "statically attested direct argv",
        )

    unattested_wrappers = sorted(
        {
            invocation
            for invocation in invocations
            if invocation.startswith("unattested-")
        }
    )
    if unattested_wrappers:
        add(
            "recorded-unattested-source-gate-wrapper",
            unattested_wrappers,
            (
                f"{SOURCE_GATE_PYTHON} "
                f"{' '.join(SOURCE_GATE_PYTHON_FLAGS)} "
                "<exact run_source_keyed_cargo_gate.py> "
                "... -- <Rust tool>"
            ),
        )

    for environment_key, profile_key, code in (
        ("RUSTUP_HOME", "rustup_home", "rustup-home"),
        ("CARGO_HOME", "cargo_home", "cargo-home"),
    ):
        if environment_key not in assignments:
            add(
                f"recorded-missing-{code}",
                None,
                expected[profile_key],
            )
            continue
        observed = resolved_value(assignments[environment_key], cwd)
        wanted = str(Path(str(expected[profile_key])).resolve(strict=False))
        if observed != wanted:
            add(f"recorded-wrong-{code}", observed, wanted)
    if "RUSTUP_TOOLCHAIN" not in assignments:
        add(
            "recorded-missing-rustup-toolchain",
            None,
            expected["rustup_toolchain"],
        )
    elif assignments["RUSTUP_TOOLCHAIN"] != str(expected["rustup_toolchain"]):
        add(
            "recorded-wrong-rustup-toolchain",
            assignments["RUSTUP_TOOLCHAIN"],
            expected["rustup_toolchain"],
        )
    recorded_source_key: str | None = None
    if "CARGO_TARGET_DIR" not in assignments:
        add(
            "recorded-missing-target-dir",
            None,
            (
                f"{Path(str(expected['target_dir'])).resolve(strict=False)}/"
                f"{SOURCE_KEY_TARGET_DIRECTORY}/<64-hex-source-key>"
            ),
        )
    else:
        observed = resolved_value(assignments["CARGO_TARGET_DIR"], cwd)
        if not target_dir_is_admitted(
            assignments["CARGO_TARGET_DIR"],
            cwd=cwd,
            profile=expected,
            allow_shared_root=False,
        ):
            add(
                "recorded-wrong-target-dir",
                observed,
                (
                    f"{Path(str(expected['target_dir'])).resolve(strict=False)}/"
                    f"{SOURCE_KEY_TARGET_DIRECTORY}/<64-hex-source-key>"
                ),
            )
        else:
            recorded_source_key = Path(observed).name
    recorded_provenance = assignments.get(
        "CODEFIRE_CARGO_GATE_PROVENANCE"
    )
    if recorded_provenance is None:
        add(
            "recorded-missing-source-key-gate-provenance",
            None,
            SOURCE_KEY_GATE_PROVENANCE,
        )
    elif recorded_provenance != SOURCE_KEY_GATE_PROVENANCE:
        add(
            "recorded-wrong-source-key-gate-provenance",
            recorded_provenance,
            SOURCE_KEY_GATE_PROVENANCE,
        )
    assigned_source_key = assignments.get(
        "CODEFIRE_CARGO_GATE_SOURCE_KEY"
    )
    if assigned_source_key is None:
        add(
            "recorded-missing-source-key-gate-key",
            None,
            recorded_source_key or "<64-hex-source-key>",
        )
    elif (
        SOURCE_KEY_PATTERN.fullmatch(assigned_source_key) is None
        or assigned_source_key != recorded_source_key
    ):
        add(
            "recorded-wrong-source-key-gate-key",
            assigned_source_key,
            recorded_source_key or "<64-hex-source-key>",
        )
    for environment_key, maximum, code in (
        ("CARGO_BUILD_JOBS", int(expected["max_build_jobs"]), "build-jobs"),
        (
            "RUST_TEST_THREADS",
            int(expected["max_test_threads"]),
            "test-threads",
        ),
    ):
        if environment_key not in assignments:
            add(f"recorded-missing-{code}", None, maximum)
            continue
        value = assignments[environment_key]
        try:
            safe = 1 <= int(value) <= maximum
        except ValueError:
            safe = False
        if not safe:
            add(f"recorded-unsafe-{code}", value, maximum)
    if "CARGO_INCREMENTAL" not in assignments:
        add(
            "recorded-missing-incremental",
            None,
            expected["incremental"],
        )
    elif assignments["CARGO_INCREMENTAL"] != str(expected["incremental"]):
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
        "live_source_key_judgment": {
            "classification": "structural-advisory",
            "fail_closed_without_runtime_controls": True,
            "cryptographic_attestation": False,
        },
        "live_processes_scanned": len(observations),
        "recorded_commands_scanned": recorded_count,
        "recorded_violation_commands": recorded_violation_count,
        "live_violations": live,
        "recorded_violations": recorded,
        "blockers": blockers,
    }


def atomic_write_json(path: Path, payload: dict[str, Any]) -> None:
    publish_atomic_json(path, payload)


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
