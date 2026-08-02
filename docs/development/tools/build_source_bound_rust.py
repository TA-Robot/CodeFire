#!/usr/bin/env python3
"""Build the Rust CLI and emit provenance bound to unchanged source inputs."""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import importlib.util
import json
import math
import os
import secrets
import select
import signal
import shutil
import stat
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

POLICY_PYCACHE_PREFIX = "/dev/null"
BUILD_BOOTSTRAP_FD_ENV = "CODEFIRE_BUILD_BOOTSTRAP_FD"
BUILD_BOOTSTRAP_PYTHON_FD_ENV = "CODEFIRE_BUILD_BOOTSTRAP_PYTHON_FD"
BUILD_BOOTSTRAP_PATH_ENV = "CODEFIRE_BUILD_BOOTSTRAP_PATH"
BUILD_BOOTSTRAP_MAX_BYTES = 1024 * 1024
BUILD_BOOTSTRAP_MAX_PYTHON_BYTES = 256 * 1024**2


def _canonical_python_startup() -> bool:
    flags = sys.flags
    return (
        flags.isolated == 1
        and flags.ignore_environment == 1
        and flags.no_user_site == 1
        and flags.no_site == 1
        and flags.dont_write_bytecode == 1
        and flags.optimize == 0
        and not flags.dev_mode
        and sys._xoptions.get("pycache_prefix")
        == POLICY_PYCACHE_PREFIX
        and sys.pycache_prefix == POLICY_PYCACHE_PREFIX
    )


def _bootstrap_file_identity(
    descriptor: int,
    *,
    maximum_bytes: int,
    owner_controlled: bool,
) -> dict[str, Any]:
    metadata = os.fstat(descriptor)
    if (
        not stat.S_ISREG(metadata.st_mode)
        or metadata.st_nlink != 1
        or metadata.st_size > maximum_bytes
        or metadata.st_mode & 0o022
        or (owner_controlled and metadata.st_uid != os.geteuid())
    ):
        raise RuntimeError("build bootstrap file identity is unsafe")
    digest = hashlib.sha256()
    offset = 0
    while offset < metadata.st_size:
        chunk = os.pread(
            descriptor,
            min(1024 * 1024, metadata.st_size - offset),
            offset,
        )
        if not chunk:
            raise RuntimeError("build bootstrap file read was incomplete")
        digest.update(chunk)
        offset += len(chunk)
    after = os.fstat(descriptor)
    expected = (
        metadata.st_dev,
        metadata.st_ino,
        stat.S_IMODE(metadata.st_mode),
        metadata.st_uid,
        metadata.st_nlink,
        metadata.st_size,
        metadata.st_mtime_ns,
        metadata.st_ctime_ns,
    )
    current = (
        after.st_dev,
        after.st_ino,
        stat.S_IMODE(after.st_mode),
        after.st_uid,
        after.st_nlink,
        after.st_size,
        after.st_mtime_ns,
        after.st_ctime_ns,
    )
    if current != expected:
        raise RuntimeError("build bootstrap file changed while hashing")
    return {
        "bytes": metadata.st_size,
        "sha256": digest.hexdigest(),
        "device": metadata.st_dev,
        "inode": metadata.st_ino,
        "mode": stat.S_IMODE(metadata.st_mode),
        "uid": metadata.st_uid,
        "nlink": metadata.st_nlink,
        "mtime_ns": metadata.st_mtime_ns,
        "ctime_ns": metadata.st_ctime_ns,
    }


def _initial_build_descriptor() -> tuple[int, Path]:
    prefix = "/proc/self/fd/"
    if sys.argv[0].startswith(prefix):
        raw = sys.argv[0][len(prefix) :]
        if raw.isdigit():
            descriptor = os.dup(int(raw))
            target = os.readlink(f"/proc/self/fd/{descriptor}")
            if target.endswith(" (deleted)"):
                target = target[: -len(" (deleted)")]
            return descriptor, Path(target).absolute()
    canonical = Path(__file__).absolute()
    if canonical.resolve(strict=True) != canonical:
        raise RuntimeError("build bootstrap path must not contain symlinks")
    return (
        os.open(
            canonical,
            os.O_RDONLY
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
        ),
        canonical,
    )


def _build_bootstrap() -> dict[str, Any]:
    if not _canonical_python_startup():
        raise RuntimeError(
            "build wrapper requires exact Python flags: "
            "-I -S -B -X pycache_prefix=/dev/null"
        )
    raw_build_fd = os.environ.get(BUILD_BOOTSTRAP_FD_ENV)
    raw_python_fd = os.environ.get(BUILD_BOOTSTRAP_PYTHON_FD_ENV)
    if raw_build_fd is None and raw_python_fd is None:
        build_fd, canonical_path = _initial_build_descriptor()
        python_fd = os.open("/proc/self/exe", os.O_RDONLY)
        try:
            build_identity = _bootstrap_file_identity(
                build_fd,
                maximum_bytes=BUILD_BOOTSTRAP_MAX_BYTES,
                owner_controlled=True,
            )
            python_identity = _bootstrap_file_identity(
                python_fd,
                maximum_bytes=BUILD_BOOTSTRAP_MAX_PYTHON_BYTES,
                owner_controlled=False,
            )
            running_python = os.stat("/proc/self/exe")
            if (running_python.st_dev, running_python.st_ino) != (
                python_identity["device"],
                python_identity["inode"],
            ):
                raise RuntimeError("running Python identity changed")
            os.set_inheritable(build_fd, True)
            os.set_inheritable(python_fd, True)
            environment = dict(os.environ)
            environment[BUILD_BOOTSTRAP_FD_ENV] = str(build_fd)
            environment[BUILD_BOOTSTRAP_PYTHON_FD_ENV] = str(python_fd)
            environment[BUILD_BOOTSTRAP_PATH_ENV] = str(canonical_path)
            os.execve(
                python_fd,
                [
                    f"/proc/self/fd/{python_fd}",
                    "-I",
                    "-S",
                    "-B",
                    "-X",
                    f"pycache_prefix={POLICY_PYCACHE_PREFIX}",
                    f"/proc/self/fd/{build_fd}",
                    *sys.argv[1:],
                ],
                environment,
            )
        except BaseException:
            os.close(build_fd)
            os.close(python_fd)
            raise
        raise RuntimeError("build bootstrap exec returned")
    if raw_build_fd is None or raw_python_fd is None:
        raise RuntimeError("build bootstrap descriptors are incomplete")
    try:
        build_fd = int(raw_build_fd)
        python_fd = int(raw_python_fd)
    except ValueError as exc:
        raise RuntimeError("build bootstrap descriptor is invalid") from exc
    if sys.argv[0] != f"/proc/self/fd/{build_fd}":
        raise RuntimeError(
            "build bootstrap argv does not name the held wrapper"
        )
    canonical_raw = os.environ.get(BUILD_BOOTSTRAP_PATH_ENV)
    if not canonical_raw:
        raise RuntimeError("build bootstrap path is unavailable")
    build_identity = _bootstrap_file_identity(
        build_fd,
        maximum_bytes=BUILD_BOOTSTRAP_MAX_BYTES,
        owner_controlled=True,
    )
    python_identity = _bootstrap_file_identity(
        python_fd,
        maximum_bytes=BUILD_BOOTSTRAP_MAX_PYTHON_BYTES,
        owner_controlled=False,
    )
    running_python = os.stat("/proc/self/exe")
    if (running_python.st_dev, running_python.st_ino) != (
        python_identity["device"],
        python_identity["inode"],
    ):
        raise RuntimeError("build bootstrap Python differs from /proc/self/exe")
    return {
        "build_descriptor": build_fd,
        "build_identity": build_identity,
        "python_descriptor": python_fd,
        "python_identity": python_identity,
        "canonical_path": Path(canonical_raw).absolute(),
    }


_RUNNING_BUILD_WRAPPER = (
    _build_bootstrap() if __name__ == "__main__" else None
)
sys.dont_write_bytecode = True
sys.pycache_prefix = POLICY_PYCACHE_PREFIX

TOOLS_DIRECTORY = (
    Path(_RUNNING_BUILD_WRAPPER["canonical_path"]).parent
    if _RUNNING_BUILD_WRAPPER is not None
    else Path(__file__).resolve().parent
)
if str(TOOLS_DIRECTORY) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIRECTORY))


def _preload_ai_contracts() -> dict[str, Any]:
    path = TOOLS_DIRECTORY / "ai_contracts.py"
    descriptor = os.open(
        path,
        os.O_RDONLY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    try:
        identity = _bootstrap_file_identity(
            descriptor,
            maximum_bytes=BUILD_BOOTSTRAP_MAX_BYTES,
            owner_controlled=True,
        )
        raw = os.pread(descriptor, identity["bytes"], 0)
        if len(raw) != identity["bytes"]:
            raise RuntimeError("ai_contracts read was incomplete")
        spec = importlib.util.spec_from_loader(
            "ai_contracts", loader=None, origin=str(path)
        )
        if spec is None:
            raise RuntimeError("ai_contracts spec is unavailable")
        module = importlib.util.module_from_spec(spec)
        module.__file__ = str(path)
        module.__cached__ = None
        sys.modules["ai_contracts"] = module
        try:
            exec(
                compile(raw, str(path), "exec", dont_inherit=True),
                module.__dict__,
            )
        except BaseException:
            sys.modules.pop("ai_contracts", None)
            raise
        return {
            "descriptor": descriptor,
            "path": path,
            "identity": identity,
        }
    except BaseException:
        os.close(descriptor)
        raise


_BUILD_AI_CONTRACTS = _preload_ai_contracts()
from ai_contracts import sha256_json


POLICY_IDENTITY_FIELDS = (
    "bytes",
    "sha256",
    "device",
    "inode",
    "mode",
    "uid",
    "nlink",
    "mtime_ns",
    "ctime_ns",
)


def running_build_wrapper_identity() -> dict[str, Any]:
    if _RUNNING_BUILD_WRAPPER is None:
        descriptor = os.open(
            Path(__file__).resolve(strict=True),
            os.O_RDONLY | os.O_CLOEXEC,
        )
        try:
            identity = _bootstrap_file_identity(
                descriptor,
                maximum_bytes=BUILD_BOOTSTRAP_MAX_BYTES,
                owner_controlled=True,
            )
        finally:
            os.close(descriptor)
        return identity
    descriptor = int(_RUNNING_BUILD_WRAPPER["build_descriptor"])
    current = _bootstrap_file_identity(
        descriptor,
        maximum_bytes=BUILD_BOOTSTRAP_MAX_BYTES,
        owner_controlled=True,
    )
    if current != _RUNNING_BUILD_WRAPPER["build_identity"]:
        raise ValueError("running build wrapper changed")
    return current


def running_build_python_identity() -> dict[str, Any]:
    if _RUNNING_BUILD_WRAPPER is None:
        descriptor = os.open("/proc/self/exe", os.O_RDONLY)
        try:
            return _bootstrap_file_identity(
                descriptor,
                maximum_bytes=BUILD_BOOTSTRAP_MAX_PYTHON_BYTES,
                owner_controlled=False,
            )
        finally:
            os.close(descriptor)
    descriptor = int(_RUNNING_BUILD_WRAPPER["python_descriptor"])
    current = _bootstrap_file_identity(
        descriptor,
        maximum_bytes=BUILD_BOOTSTRAP_MAX_PYTHON_BYTES,
        owner_controlled=False,
    )
    if current != _RUNNING_BUILD_WRAPPER["python_identity"]:
        raise ValueError("running build Python changed")
    return current


def running_ai_contracts_identity() -> dict[str, Any]:
    descriptor = int(_BUILD_AI_CONTRACTS["descriptor"])
    current = _bootstrap_file_identity(
        descriptor,
        maximum_bytes=BUILD_BOOTSTRAP_MAX_BYTES,
        owner_controlled=True,
    )
    if current != _BUILD_AI_CONTRACTS["identity"]:
        raise ValueError("loaded ai_contracts policy changed")
    return current


def gate_policy_identity(
    gate_result: dict[str, Any], name: str
) -> dict[str, Any]:
    isolation = gate_result.get("readonly_isolation_identity")
    tools = (
        isolation.get("tools")
        if isinstance(isolation, dict)
        else None
    )
    identity = tools.get(name) if isinstance(tools, dict) else None
    if not isinstance(identity, dict):
        raise ValueError(f"source gate policy identity is missing: {name}")
    return identity


def validate_outer_policy_binding(
    gate_result: dict[str, Any],
) -> dict[str, Any]:
    expected = {
        "policy-build-source-bound": running_build_wrapper_identity(),
        "policy-ai-contracts": running_ai_contracts_identity(),
        "python": running_build_python_identity(),
    }
    bound: dict[str, Any] = {}
    for name, outer_identity in expected.items():
        gate_identity = gate_policy_identity(gate_result, name)
        for field in POLICY_IDENTITY_FIELDS:
            if gate_identity.get(field) != outer_identity.get(field):
                raise ValueError(
                    "source gate policy differs from outer build policy: "
                    f"{name}.{field}"
                )
        bound[name] = {
            field: outer_identity[field]
            for field in POLICY_IDENTITY_FIELDS
        }
    payload = {
        "schema": "codefire.rust_build_outer_policy_binding.v1",
        "members": bound,
        "python_flags": [
            "-I",
            "-S",
            "-B",
            "-X",
            f"pycache_prefix={POLICY_PYCACHE_PREFIX}",
        ],
    }
    payload["identity_sha256"] = sha256_json(payload)
    return payload


BUILD_COMMAND = [
    "cargo", "build", "--release", "--locked", "-p", "codefire-cli", "--bin", "codefire-rs",
]
BINARY_PATH = Path("target/release/codefire-rs")
DEFAULT_OUTPUT = Path("target/codefire-ai/rust-build-provenance.json")
GIT_EXECUTABLE = "/usr/bin/git"
SOURCE_KEYED_CARGO_GATE = Path(__file__).resolve().with_name(
    "run_source_keyed_cargo_gate.py"
)
PARENT_DEATH_HELPER = Path(__file__).resolve().with_name(
    "pdeath_exec.py"
)
SOURCE_GATE_PYTHON = Path(sys.executable).resolve(strict=True)
DEFAULT_GATE_EVIDENCE_ROOT = Path(
    "/workspace/.cache/codefire-gate-evidence"
)
SOURCE_GATE_LAUNCHER_ADMITTED_NAMES = frozenset(
    {
        "AR",
        "CC",
        "CFLAGS",
        "CARGO_BUILD_JOBS",
        "CARGO_HOME",
        "CARGO_HTTP_CHECK_REVOKE",
        "CARGO_INCREMENTAL",
        "CARGO_NET_OFFLINE",
        "CARGO_TARGET_DIR",
        "CODEFIRE_GATE_EVIDENCE_LOCK_FD",
        "CXX",
        "CXXFLAGS",
        "HOST",
        "PKG_CONFIG",
        "PKG_CONFIG_LIBDIR",
        "PKG_CONFIG_PATH",
        "RANLIB",
        "RUSTUP_HOME",
        "RUSTUP_TOOLCHAIN",
        "RUST_BACKTRACE",
        "RUSTFLAGS",
        "RUSTDOCFLAGS",
        "RUST_TEST_THREADS",
        "TARGET",
        "TZ",
    }
)
SOURCE_GATE_LAUNCHER_ADMITTED_PREFIXES = (
    "AR_",
    "CC_",
    "CFLAGS_",
    "CXX_",
    "CXXFLAGS_",
    "PKG_CONFIG_",
    "RANLIB_",
)
GIT_CONTROL_TIMEOUT_SECONDS = 120.0
MAX_GIT_CONTROL_STDOUT_BYTES = 256 * 1024 * 1024
MAX_GIT_CONTROL_STDERR_BYTES = 64 * 1024
GIT_CONTROL_CONFIG_ARGS = (
    "-c",
    "core.fsmonitor=false",
    "-c",
    "core.hooksPath=/dev/null",
    "-c",
    "core.attributesFile=/dev/null",
    "-c",
    "core.excludesFile=/dev/null",
    "-c",
    "core.untrackedCache=false",
    "-c",
    "maintenance.auto=false",
)
SOURCE_GATE_PID_LAUNCHER = Path("/usr/bin/unshare")
SOURCE_GATE_PARENT_DEATH_LAUNCHER = Path("/usr/bin/setpriv")
SOURCE_GATE_PARENT_DEATH_ARGS = ("--pdeathsig", "SIGKILL")
SOURCE_GATE_PID_NAMESPACE_ARGS = (
    "-Urpf",
    "--mount-proc",
    "--kill-child=SIGKILL",
    # --mount-proc implies --mount, so this launcher does create a mount
    # namespace and does mount a fresh procfs. Private propagation is the
    # util-linux default, but a default is not evidence: request it so the
    # namespace can never push that procfs back onto the host.
    "--propagation",
    "private",
    "--",
)
SOURCE_GATE_CONTROL_OVERHEAD_SECONDS = (
    15 * 60 + 12 * GIT_CONTROL_TIMEOUT_SECONDS + 10 * 60
)
MAX_GATE_EVIDENCE_ALLOCATED_BYTES = 8 * 1024**3
MAX_GATE_EVIDENCE_ENTRIES = 10_000
GATE_EVIDENCE_ATTEMPT_RESERVE_BYTES = 258 * 1024**2
MAX_GATE_RESULT_BYTES = 1024**2
GATE_EVIDENCE_RESULT_PUBLICATION_RESERVE_BYTES = 2 * 1024**2
GATE_EVIDENCE_RESULT_PUBLICATION_ENTRY_RESERVE = 2
GATE_EVIDENCE_SLOT_COUNT = 2
GATE_EVIDENCE_LOCK_NAME = ".codefire-rust-gate-evidence.lock"
GATE_EVIDENCE_FREE_FLOOR_BYTES = 1024**3
MAX_BUILD_SOURCE_FILES = 200_000
MAX_BUILD_SOURCE_LOGICAL_BYTES = 2 * 1024**3
SOURCE_PREFLIGHT_TIMEOUT_SECONDS = 120.0
PATH_ERROR_SAMPLE_COUNT = 8
MAX_JSON_STRING_CHARACTERS = 64 * 1024
MAX_JSON_CONTAINER_ENTRIES = 100_000
MAX_JSON_DEPTH = 64
MAX_PROVENANCE_PARENT_ENTRIES = 10_000
MAX_PROVENANCE_RESIDUES = 32
PROVENANCE_RECOVERY_TIMEOUT_SECONDS = 10.0
GATE_EVIDENCE_SCAN_TIMEOUT_SECONDS = 10.0


def git_control_environment() -> dict[str, str]:
    return {
        "GIT_ATTR_NOSYSTEM": "1",
        "GIT_CONFIG_GLOBAL": "/dev/null",
        "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_CONFIG_SYSTEM": "/dev/null",
        "GIT_DISCOVERY_ACROSS_FILESYSTEM": "0",
        "GIT_NO_REPLACE_OBJECTS": "1",
        "GIT_OPTIONAL_LOCKS": "0",
        "GIT_TERMINAL_PROMPT": "0",
        "HOME": "/nonexistent",
        "LANG": "C",
        "LC_ALL": "C",
        "PATH": "/usr/bin:/bin",
        "XDG_CONFIG_HOME": "/nonexistent",
    }


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def command_text(command: list[str]) -> str:
    return " ".join(command)


def remaining_preflight_seconds(deadline: float) -> float:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise TimeoutError("Rust source preflight deadline exceeded")
    return min(GIT_CONTROL_TIMEOUT_SECONDS, remaining)


def path_set_summary(label: str, paths: set[str]) -> str:
    ordered = sorted(paths)
    digest = hashlib.sha256()
    for item in ordered:
        raw = item.encode("utf-8", errors="surrogatepass")
        digest.update(len(raw).to_bytes(4, "big"))
        digest.update(raw)
    return (
        f"{label}_count={len(ordered)}, "
        f"{label}_sha256={digest.hexdigest()}, "
        f"{label}_sample={ordered[:PATH_ERROR_SAMPLE_COUNT]!r}"
    )


def bounded_json_bytes(
    payload: Any,
    *,
    maximum_bytes: int,
    label: str,
    trailing_newline: bool = True,
) -> bytes:
    stack: list[tuple[Any, int]] = [(payload, 0)]
    entries = 0
    while stack:
        value, depth = stack.pop()
        if depth > MAX_JSON_DEPTH:
            raise ValueError(f"{label} exceeds JSON depth bound")
        if isinstance(value, str):
            if len(value) > MAX_JSON_STRING_CHARACTERS:
                raise ValueError(
                    f"{label} contains an oversized string"
                )
        elif isinstance(value, dict):
            entries += len(value)
            if entries > MAX_JSON_CONTAINER_ENTRIES:
                raise ValueError(
                    f"{label} exceeds JSON entry bound"
                )
            for key, item in value.items():
                if (
                    not isinstance(key, str)
                    or len(key) > MAX_JSON_STRING_CHARACTERS
                ):
                    raise ValueError(
                        f"{label} contains an invalid object key"
                    )
                stack.append((item, depth + 1))
        elif isinstance(value, (list, tuple)):
            entries += len(value)
            if entries > MAX_JSON_CONTAINER_ENTRIES:
                raise ValueError(
                    f"{label} exceeds JSON entry bound"
                )
            stack.extend(
                (item, depth + 1) for item in value
            )
        elif value is not None and not isinstance(
            value, (bool, int, float)
        ):
            raise ValueError(
                f"{label} contains a non-JSON value"
            )
    encoder = json.JSONEncoder(
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
        allow_nan=False,
    )
    output = bytearray()
    for text in encoder.iterencode(payload):
        raw = text.encode("utf-8")
        if (
            len(output)
            + len(raw)
            + (1 if trailing_newline else 0)
            > maximum_bytes
        ):
            raise ValueError(
                f"{label} exceeds {maximum_bytes} bytes"
            )
        output.extend(raw)
    if trailing_newline:
        output.extend(b"\n")
    return bytes(output)


def bounded_git(
    root: Path,
    args: list[str],
    *,
    timeout: float = GIT_CONTROL_TIMEOUT_SECONDS,
    maximum_stdout_bytes: int = MAX_GIT_CONTROL_STDOUT_BYTES,
) -> subprocess.CompletedProcess[bytes]:
    command = [
        GIT_EXECUTABLE,
        *GIT_CONTROL_CONFIG_ARGS,
        *args,
    ]
    process = subprocess.Popen(
        command,
        cwd=root,
        env=git_control_environment(),
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    if process.stdout is None or process.stderr is None:
        process.kill()
        process.wait()
        raise ValueError(f"Git control pipes unavailable: {args}")
    streams = {
        process.stdout.fileno(): "stdout",
        process.stderr.fileno(): "stderr",
    }
    for descriptor in streams:
        os.set_blocking(descriptor, False)
    stdout = bytearray()
    stderr = bytearray()
    deadline = time.monotonic() + timeout
    try:
        while streams:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError(f"Git operation timed out: {args}")
            try:
                readable, _, _ = select.select(
                    list(streams), [], [], remaining
                )
            except InterruptedError:
                continue
            if not readable:
                raise TimeoutError(f"Git operation timed out: {args}")
            for descriptor in readable:
                try:
                    chunk = os.read(descriptor, 1024 * 1024)
                except BlockingIOError:
                    continue
                if not chunk:
                    streams.pop(descriptor, None)
                    continue
                output = (
                    stdout
                    if streams[descriptor] == "stdout"
                    else stderr
                )
                output.extend(chunk)
                limit = (
                    maximum_stdout_bytes
                    if output is stdout
                    else MAX_GIT_CONTROL_STDERR_BYTES
                )
                if len(output) > limit:
                    raise ValueError(
                        f"Git control output exceeded {limit} bytes: "
                        f"{args}"
                    )
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError(f"Git operation timed out: {args}")
        returncode = process.wait(timeout=remaining)
    except BaseException:
        if process.poll() is None:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        raise
    finally:
        process.stdout.close()
        process.stderr.close()
    return subprocess.CompletedProcess(
        command, returncode, bytes(stdout), bytes(stderr)
    )


def source_paths(
    root: Path,
    *,
    deadline: float | None = None,
) -> list[Path]:
    if deadline is None:
        deadline = time.monotonic() + SOURCE_PREFLIGHT_TIMEOUT_SECONDS
    result = bounded_git(
        root,
        [
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--",
            "Cargo.toml",
            "Cargo.lock",
            ".cargo",
            "rust-toolchain",
            "rust-toolchain.toml",
            "crates",
        ],
        timeout=remaining_preflight_seconds(deadline),
        maximum_stdout_bytes=64 * 1024 * 1024,
    )
    if result.returncode != 0:
        raise ValueError(result.stderr.decode("utf-8", errors="replace").strip() or "git ls-files failed")
    paths: list[Path] = []
    raw_paths = [raw for raw in result.stdout.split(b"\0") if raw]
    if len(raw_paths) > MAX_BUILD_SOURCE_FILES:
        raise ValueError(
            "Rust source file count exceeded "
            f"{MAX_BUILD_SOURCE_FILES}"
        )
    logical_bytes = 0
    for raw in raw_paths:
        if time.monotonic() > deadline:
            raise TimeoutError(
                "Rust source preflight deadline exceeded"
            )
        relative = raw.decode("utf-8", errors="strict")
        candidate = root / relative
        cursor = root
        for part in Path(relative).parts:
            cursor = cursor / part
            if cursor.is_symlink():
                raise ValueError(f"Rust build input contains a symlink: {relative}")
        resolved = candidate.resolve(strict=True)
        resolved.relative_to(root)
        metadata = resolved.stat()
        if not stat.S_ISREG(metadata.st_mode):
            raise ValueError(f"Rust build input is not a regular file: {relative}")
        if metadata.st_nlink != 1:
            raise ValueError(
                f"Rust build input must not be hard-linked: {relative}"
            )
        logical_bytes += metadata.st_size
        if logical_bytes > MAX_BUILD_SOURCE_LOGICAL_BYTES:
            raise ValueError(
                "Rust source logical bytes exceeded "
                f"{MAX_BUILD_SOURCE_LOGICAL_BYTES}"
            )
        paths.append(resolved)
    if not paths:
        raise ValueError("Rust source input set is empty")
    return sorted(set(paths), key=lambda path: path.relative_to(root).as_posix())


def build_source_manifest(
    root: Path,
    *,
    deadline: float | None = None,
) -> dict[str, Any]:
    if deadline is None:
        deadline = time.monotonic() + SOURCE_PREFLIGHT_TIMEOUT_SECONDS
    files = []
    logical_bytes = 0
    for path in source_paths(root, deadline=deadline):
        if time.monotonic() > deadline:
            raise TimeoutError(
                "Rust source preflight deadline exceeded"
            )
        descriptor = os.open(
            path,
            os.O_RDONLY
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
        )
        try:
            before = os.fstat(descriptor)
            if (
                not stat.S_ISREG(before.st_mode)
                or before.st_nlink != 1
                or (
                    logical_bytes + before.st_size
                    > MAX_BUILD_SOURCE_LOGICAL_BYTES
                )
            ):
                raise ValueError(
                    f"Rust build input identity is unsafe: {path}"
                )
            digest = hashlib.sha256()
            size = 0
            while True:
                if time.monotonic() > deadline:
                    raise TimeoutError(
                        "Rust source preflight deadline exceeded"
                    )
                chunk = os.read(descriptor, 1024 * 1024)
                if not chunk:
                    break
                digest.update(chunk)
                size += len(chunk)
            after = os.fstat(descriptor)
            if (
                (before.st_dev, before.st_ino, before.st_size)
                != (after.st_dev, after.st_ino, after.st_size)
                or size != after.st_size
            ):
                raise ValueError(
                    f"Rust build input changed while hashing: {path}"
                )
        finally:
            os.close(descriptor)
        logical_bytes += size
        files.append(
            {
                "path": path.relative_to(root).as_posix(),
                "bytes": size,
                "sha256": digest.hexdigest(),
            }
        )
    return {
        "schema": "codefire.rust_build_inputs.v1",
        "files": files,
        "manifest_sha256": sha256_json(files),
    }


def git_head(root: Path) -> str:
    result = bounded_git(
        root, ["rev-parse", "HEAD"], timeout=10.0
    )
    if result.returncode != 0:
        message = (result.stderr or result.stdout).decode(
            "utf-8", errors="replace"
        ).strip()
        raise ValueError(message or "git rev-parse failed")
    return result.stdout.decode("utf-8", errors="strict").strip()


def validate_source_inputs_against_head(
    root: Path,
    expected_revision: str,
    *,
    deadline: float | None = None,
) -> list[str]:
    """Require every build input to be tracked and byte-identical to expected HEAD."""

    if deadline is None:
        deadline = time.monotonic() + SOURCE_PREFLIGHT_TIMEOUT_SECONDS
    try:
        head = bounded_git(
            root,
            [
            "rev-parse",
            "--verify",
            f"{expected_revision}^{{commit}}",
            ],
            timeout=remaining_preflight_seconds(deadline),
        )
    except (OSError, TimeoutError, ValueError) as exc:
        return [str(exc)]
    if head.returncode != 0:
        message = (head.stderr or head.stdout).decode(
            "utf-8", errors="replace"
        ).strip()
        return [message or "expected revision is not a commit"]
    resolved_head = head.stdout.decode("utf-8", errors="strict").strip()
    if resolved_head != expected_revision:
        return [f"expected revision must be the canonical commit SHA {resolved_head}"]
    try:
        listed = bounded_git(
            root,
            [
            "ls-tree", "-r", "--name-only", "-z", resolved_head, "--",
            "Cargo.toml", "Cargo.lock", ".cargo", "rust-toolchain",
            "rust-toolchain.toml", "crates",
            ],
            timeout=remaining_preflight_seconds(deadline),
            maximum_stdout_bytes=64 * 1024 * 1024,
        )
    except (OSError, TimeoutError, ValueError) as exc:
        return [str(exc)]
    if listed.returncode != 0:
        return [listed.stderr.decode("utf-8", errors="replace").strip() or "git ls-tree failed"]
    raw_head_paths = [
        item for item in listed.stdout.split(b"\0") if item
    ]
    if len(raw_head_paths) > MAX_BUILD_SOURCE_FILES:
        return [
            "HEAD Rust source file count exceeded "
            f"{MAX_BUILD_SOURCE_FILES}"
        ]
    head_paths = {
        item.decode("utf-8", errors="strict")
        for item in raw_head_paths
    }
    try:
        current_paths = {
            path.relative_to(root).as_posix()
            for path in source_paths(root, deadline=deadline)
        }
    except (FileNotFoundError, UnicodeDecodeError, ValueError) as exc:
        return [str(exc)]
    errors: list[str] = []
    if current_paths != head_paths:
        extra = current_paths - head_paths
        missing = head_paths - current_paths
        errors.append(
            "build input path set differs from HEAD: "
            f"{path_set_summary('untracked_or_extra', extra)}; "
            f"{path_set_summary('missing', missing)}"
        )
    try:
        compared = bounded_git(
            root,
            [
                "diff",
                "--quiet",
                "--no-ext-diff",
                "--no-textconv",
                resolved_head,
                "--",
                "Cargo.toml",
                "Cargo.lock",
                ".cargo",
                "rust-toolchain",
                "rust-toolchain.toml",
                "crates",
            ],
            timeout=remaining_preflight_seconds(deadline),
            maximum_stdout_bytes=64 * 1024,
        )
    except (OSError, TimeoutError, ValueError) as exc:
        errors.append(str(exc))
    else:
        if compared.returncode == 1:
            errors.append("tracked Rust build inputs differ from HEAD")
        elif compared.returncode != 0:
            message = compared.stderr.decode(
                "utf-8", errors="replace"
            ).strip()
            errors.append(
                message or "Git could not compare build inputs to HEAD"
            )
    return errors


def gate_evidence_root_contract(root: Path, raw: object) -> Path:
    if not isinstance(raw, str) or not raw:
        raise ValueError("source-keyed gate evidence root is missing")
    lexical = Path(raw).absolute()
    if not Path(raw).is_absolute():
        raise ValueError("source-keyed gate evidence root must be absolute")
    resolved = lexical.resolve(strict=True)
    metadata = os.stat(lexical, follow_symlinks=False)
    canonical_root = root.resolve(strict=True)
    if (
        lexical != resolved
        or not stat.S_ISDIR(metadata.st_mode)
        or metadata.st_uid != os.geteuid()
        or metadata.st_mode & 0o022
        or resolved == canonical_root
        or resolved in canonical_root.parents
        or canonical_root in resolved.parents
    ):
        raise ValueError(
            "source-keyed gate evidence root must be a real, "
            "owner-controlled directory outside the repository"
        )
    return resolved


def validate_source_keyed_gate_chain(
    root: Path,
    payload: dict[str, Any],
    *,
    expected_revision: str,
) -> list[str]:
    errors: list[str] = []
    try:
        evidence_root = gate_evidence_root_contract(
            root, payload.get("source_keyed_gate_evidence_root")
        )
        result_path_raw = payload.get(
            "source_keyed_gate_result_path"
        )
        log_path_raw = payload.get("build_log_path")
        if not isinstance(result_path_raw, str):
            raise ValueError("source-keyed gate result path is missing")
        if not isinstance(log_path_raw, str):
            raise ValueError("source-keyed gate log path is missing")
        result_path = Path(result_path_raw)
        log_path = Path(log_path_raw)
        retention = payload.get("gate_evidence_retention")
        destination_contract = payload.get(
            "provenance_destination_contract"
        )
        relative_output = (
            destination_contract.get("relative_path")
            if isinstance(destination_contract, dict)
            else None
        )
        selected_slot = (
            retention.get("selected_slot")
            if isinstance(retention, dict)
            else None
        )
        if (
            not isinstance(relative_output, str)
            or selected_slot not in {0, 1}
        ):
            raise ValueError(
                "gate evidence slot binding is incomplete"
            )
        expected_scope = hashlib.sha256(
            (
                str(root.resolve(strict=True))
                + "\0"
                + relative_output
            ).encode("utf-8")
        ).hexdigest()[:24]
        if (
            retention.get("scope") != expected_scope
            or log_path.name
            != (
                f"rust-release-build-{expected_scope}-"
                f"{selected_slot}.log"
            )
            or result_path.name
            != (
                f"rust-release-build-{expected_scope}-"
                f"{selected_slot}.json"
            )
        ):
            raise ValueError(
                "gate evidence paths differ from retained scope/slot"
            )
        result_evidence = bound_gate_evidence(
            evidence_root,
            result_path,
            maximum_bytes=MAX_GATE_RESULT_BYTES,
            include_bytes=True,
        )
        log_evidence = bound_gate_evidence(
            evidence_root,
            log_path,
            maximum_bytes=256 * 1024 * 1024,
            include_bytes=False,
        )
        if (
            payload.get("source_keyed_gate_result_bytes")
            != result_evidence["bytes"]
            or payload.get("source_keyed_gate_result_sha256")
            != result_evidence["sha256"]
        ):
            errors.append(
                "provenance gate result digest/size differs from durable evidence"
            )
        if (
            payload.get("build_log_bytes") != log_evidence["bytes"]
            or payload.get("build_log_sha256")
            != log_evidence["sha256"]
        ):
            errors.append(
                "provenance build log digest/size differs from durable evidence"
            )
        raw = result_evidence["raw"]
        if not isinstance(raw, bytes):
            raise ValueError("source-keyed gate result bytes are unavailable")
        gate_result = json.loads(raw)
        if not isinstance(gate_result, dict):
            raise ValueError("source-keyed gate result root is not an object")
    except (
        FileNotFoundError,
        json.JSONDecodeError,
        OSError,
        TypeError,
        UnicodeDecodeError,
        ValueError,
    ) as exc:
        return [f"source-keyed gate evidence chain is unavailable: {exc}"]

    if gate_result.get("schema") != "codefire.source_keyed_cargo_gate.v1":
        errors.append("unexpected source-keyed gate result schema")
    if gate_result.get("decision") != "PASS":
        errors.append("source-keyed gate result is not PASS")
    if gate_result.get("blockers") != []:
        errors.append("source-keyed gate result contains blockers")
    if gate_result.get("argv") != BUILD_COMMAND:
        errors.append("source-keyed gate argv differs from release build")
    if gate_result.get("candidate_unchanged") is not True:
        errors.append("source-keyed gate candidate was not stable")
    candidate = gate_result.get("candidate")
    candidate_after = gate_result.get("candidate_after")
    if (
        not isinstance(candidate, dict)
        or candidate.get("candidate_root") != str(root)
        or candidate.get("head_revision") != expected_revision
        or candidate.get("dirty") is not False
        or candidate_after != candidate
    ):
        errors.append(
            "source-keyed gate candidate evidence differs from current request"
        )
    source_key = gate_result.get("source_key")
    artifact_key = gate_result.get("artifact_key")
    if (
        not isinstance(source_key, str)
        or not isinstance(artifact_key, str)
        or payload.get("source_key") != source_key
        or payload.get("artifact_key") != artifact_key
    ):
        errors.append("provenance source/artifact keys differ from gate result")
    execution_profile = gate_result.get("execution_profile")
    if (
        not isinstance(execution_profile, dict)
        or execution_profile.get("admitted_evidence_root")
        != str(evidence_root)
    ):
        errors.append("gate result evidence-root contract differs")
    gate_evidence_retention = gate_result.get("evidence_retention")
    evidence_admission = (
        gate_evidence_retention.get("admission")
        if isinstance(gate_evidence_retention, dict)
        else None
    )
    evidence_postflight = (
        gate_evidence_retention.get("postflight")
        if isinstance(gate_evidence_retention, dict)
        else None
    )
    if (
        not isinstance(gate_evidence_retention, dict)
        or gate_evidence_retention.get("schema")
        != "codefire.cargo_gate_evidence_retention.v1"
        or not isinstance(evidence_admission, dict)
        or not isinstance(evidence_postflight, dict)
        or evidence_admission.get("aggregate_budget_bytes")
        != MAX_GATE_EVIDENCE_ALLOCATED_BYTES
        or evidence_admission.get("reserve_bytes")
        != GATE_EVIDENCE_ATTEMPT_RESERVE_BYTES
        or evidence_admission.get("entry_reserve") != 2
        or evidence_postflight.get("aggregate_budget_bytes")
        != MAX_GATE_EVIDENCE_ALLOCATED_BYTES
        or evidence_postflight.get("reserve_bytes")
        != GATE_EVIDENCE_RESULT_PUBLICATION_RESERVE_BYTES
        or evidence_postflight.get("entry_reserve")
        != GATE_EVIDENCE_RESULT_PUBLICATION_ENTRY_RESERVE
        or evidence_postflight.get("projected_allocated_bytes", -1)
        > MAX_GATE_EVIDENCE_ALLOCATED_BYTES
        or not isinstance(
            evidence_postflight.get("allocated_bytes"), int
        )
        or evidence_postflight.get("allocated_bytes", -1)
        > MAX_GATE_EVIDENCE_ALLOCATED_BYTES
        or not isinstance(
            evidence_postflight.get("entry_count"), int
        )
        or evidence_postflight.get("entry_count", -1)
        > MAX_GATE_EVIDENCE_ENTRIES
        or evidence_postflight.get("free_floor_bytes")
        != GATE_EVIDENCE_FREE_FLOOR_BYTES
        or not isinstance(
            evidence_postflight.get(
                "filesystem_free_after_reserve_bytes"
            ),
            int,
        )
        or evidence_postflight.get(
            "filesystem_free_after_reserve_bytes", -1
        )
        < GATE_EVIDENCE_FREE_FLOOR_BYTES
    ):
        errors.append("source-keyed gate evidence retention is invalid")
    gate = gate_result.get("gate")
    gate_log = gate.get("log") if isinstance(gate, dict) else None
    runtime_guard = (
        gate.get("runtime_guard") if isinstance(gate, dict) else None
    )
    if (
        not isinstance(gate, dict)
        or gate.get("status") != "passed"
        or gate.get("exit_code") != 0
        or gate.get("timed_out", False)
        or not isinstance(runtime_guard, dict)
        or runtime_guard.get("enabled") is not True
        or runtime_guard.get("decision") != "PASS"
    ):
        errors.append("source-keyed gate process/runtime guard did not pass")
    if (
        not isinstance(gate_log, dict)
        or gate_log.get("path") != str(log_path)
        or gate_log.get("bytes") != log_evidence["bytes"]
        or gate_log.get("sha256") != log_evidence["sha256"]
        or gate_log.get("complete") is not True
    ):
        errors.append("gate result log evidence differs from durable log")
    log_binding = gate_result.get("log_binding")
    if (
        not isinstance(log_binding, dict)
        or log_binding.get("path") != str(log_path)
        or log_binding.get("bytes") != log_evidence["bytes"]
        or log_binding.get("sha256") != log_evidence["sha256"]
        or log_binding.get("timed_gate_evidence_matches") is not True
        or log_binding.get("source_key_present_in_prefix") is not True
    ):
        errors.append("source-keyed log binding is incomplete")
    retention = gate_result.get("retention")
    if (
        not isinstance(retention, dict)
        or retention.get("decision") != "PERSISTED"
        or not isinstance(
            retention.get("run_state_committed_sha256"), str
        )
    ):
        errors.append("source-keyed namespace was not durably committed")
    target = gate_result.get("target")
    namespace = (
        target.get("namespace") if isinstance(target, dict) else None
    )
    if (
        not isinstance(target, dict)
        or target.get("artifact_key") != artifact_key
        or not isinstance(namespace, str)
    ):
        errors.append("source-keyed target evidence is incomplete")
    artifact_export = gate_result.get("artifact_export")
    export_contract = (
        artifact_export.get("destination_contract")
        if isinstance(artifact_export, dict)
        else None
    )
    binary: dict[str, Any] | None = None
    try:
        binary = bound_binary_identity(root, BINARY_PATH)
    except (FileNotFoundError, OSError, ValueError) as exc:
        errors.append(f"source-keyed binary is unavailable: {exc}")
    if (
        not isinstance(artifact_export, dict)
        or artifact_export.get("decision") != "EXPORTED"
        or artifact_export.get("artifact_relative_path")
        != "release/codefire-rs"
        or artifact_export.get("target_artifact_path")
        != (
            str(Path(namespace) / "release/codefire-rs")
            if isinstance(namespace, str)
            else None
        )
        or artifact_export.get("export_path")
        != str(root / BINARY_PATH)
        or not isinstance(export_contract, dict)
        or export_contract.get("decision")
        != "UNTRACKED_IGNORED_CANDIDATE_TARGET"
        or export_contract.get("relative_path") != BINARY_PATH.as_posix()
        or export_contract.get("tracked") is not False
        or export_contract.get("ignored") is not True
        or payload.get("source_keyed_binary_path")
        != artifact_export.get("target_artifact_path")
    ):
        errors.append("source-keyed artifact export evidence differs")
    if binary is not None:
        for evidence_name, identity_name in (
            ("bytes", "bytes"),
            ("sha256", "sha256"),
            ("export_device", "device"),
            ("export_inode", "inode"),
            ("export_mode", "mode"),
            ("export_nlink", "nlink"),
        ):
            if (
                not isinstance(artifact_export, dict)
                or artifact_export.get(evidence_name)
                != binary[identity_name]
            ):
                errors.append(
                    "source-keyed artifact export identity differs "
                    f"for {evidence_name}"
                )
    return errors


def validate_provenance_payload(
    root: Path,
    payload: dict[str, Any],
    *,
    expected_revision: str,
) -> list[str]:
    """Verify that provenance still describes the current source and binary."""

    errors: list[str] = []
    if payload.get("schema") != "codefire.rust_build_provenance.v1":
        errors.append("unexpected provenance schema")
    if payload.get("ok") is not True:
        errors.append("provenance does not record a successful build")
    if payload.get("base_revision") != expected_revision:
        errors.append("provenance base_revision differs from review head_revision")
    try:
        current_head = git_head(root)
    except (FileNotFoundError, ValueError) as exc:
        errors.append(str(exc))
    else:
        if current_head != expected_revision:
            errors.append("review head_revision is not the current checked-out HEAD")
    errors.extend(validate_source_inputs_against_head(root, expected_revision))

    try:
        source_manifest = build_source_manifest(root)
    except (FileNotFoundError, UnicodeDecodeError, ValueError) as exc:
        errors.append(f"source manifest could not be recomputed: {exc}")
    else:
        if payload.get("source_manifest_sha256") != source_manifest["manifest_sha256"]:
            errors.append("provenance source manifest differs from the current source inputs")
        if payload.get("source_file_count") != len(source_manifest["files"]):
            errors.append("provenance source file count differs from the current source inputs")

    binary_path = payload.get("binary_path")
    if binary_path != BINARY_PATH.as_posix():
        errors.append(f"unexpected provenance binary path: {binary_path!r}")
    else:
        try:
            binary = bound_binary_identity(root, BINARY_PATH)
        except (FileNotFoundError, OSError, ValueError) as exc:
            errors.append(f"provenance binary is unavailable: {exc}")
        else:
            if payload.get("binary_bytes") != binary["bytes"]:
                errors.append(
                    "provenance binary size differs from the current binary"
                )
            if payload.get("binary_sha256") != binary["sha256"]:
                errors.append(
                    "provenance binary digest differs from the current binary"
                )
    if payload.get("build_command") != command_text(BUILD_COMMAND):
        errors.append("provenance build command is not the canonical release build")
    launcher_digest = payload.get(
        "source_gate_launcher_environment_sha256"
    )
    launcher_names = payload.get(
        "source_gate_launcher_environment_names"
    )
    if (
        not isinstance(launcher_digest, str)
        or len(launcher_digest) != 64
        or not isinstance(launcher_names, list)
        or not all(isinstance(name, str) for name in launcher_names)
        or launcher_names != sorted(set(launcher_names))
        or not {"HOME", "LANG", "LC_ALL", "PATH"}.issubset(
            set(launcher_names)
        )
        or any(
            name
            not in (
                SOURCE_GATE_LAUNCHER_ADMITTED_NAMES
                | {
                    "HOME",
                    "LANG",
                    "LC_ALL",
                    "PATH",
                    "PYTHONDONTWRITEBYTECODE",
                    "PYTHONNOUSERSITE",
                    "XDG_CONFIG_HOME",
                }
            )
            and not name.startswith(
                SOURCE_GATE_LAUNCHER_ADMITTED_PREFIXES
            )
            for name in launcher_names
        )
    ):
        errors.append("source-gate launcher environment contract is invalid")
    evidence_retention = payload.get("gate_evidence_retention")
    if (
        not isinstance(evidence_retention, dict)
        or evidence_retention.get("schema")
        != "codefire.rust_gate_evidence_slot.v1"
        or evidence_retention.get("selected_slot") not in {0, 1}
        or evidence_retention.get("aggregate_budget_bytes")
        != MAX_GATE_EVIDENCE_ALLOCATED_BYTES
        or evidence_retention.get("entry_budget")
        != MAX_GATE_EVIDENCE_ENTRIES
        or evidence_retention.get("attempt_reserve_bytes")
        != GATE_EVIDENCE_ATTEMPT_RESERVE_BYTES
    ):
        errors.append("gate evidence retention contract is invalid")
    errors.extend(
        validate_source_keyed_gate_chain(
            root, payload, expected_revision=expected_revision
        )
    )
    return errors


def provenance_destination_contract(
    root: Path, output: Path
) -> dict[str, Any]:
    destination = (
        (root / output).absolute()
        if not output.is_absolute()
        else output.absolute()
    )
    try:
        relative = destination.relative_to(root)
    except ValueError as exc:
        raise ValueError("provenance output must remain inside the repository") from exc
    if (
        len(relative.parts) < 2
        or relative.parts[0] != "target"
        or any(part in {"", ".", ".."} for part in relative.parts)
    ):
        raise ValueError(
            "provenance output must be under the repository target/ directory"
        )
    relative_text = relative.as_posix()
    tracked = bounded_git(
        root,
        ["ls-files", "-z", "--cached", "--", relative_text],
        maximum_stdout_bytes=1024 * 1024,
    )
    if tracked.returncode != 0:
        raise ValueError("could not verify provenance output tracking state")
    if tracked.stdout:
        raise ValueError("provenance output must not be tracked by Git")
    ignored = bounded_git(
        root,
        ["check-ignore", "--no-index", "--", relative_text],
        maximum_stdout_bytes=1024 * 1024,
    )
    if (
        ignored.returncode != 0
        or ignored.stdout != relative_text.encode("utf-8") + b"\n"
    ):
        raise ValueError("provenance output must be ignored by Git")
    return {
        "decision": "UNTRACKED_IGNORED_REPOSITORY_TARGET",
        "relative_path": relative_text,
        "tracked": False,
        "ignored": True,
    }


def _write_all(descriptor: int, raw: bytes) -> None:
    offset = 0
    while offset < len(raw):
        written = os.write(descriptor, raw[offset:])
        if written < 1:
            raise OSError("short provenance write")
        offset += written


def _open_provenance_parent(root: Path, destination: Path) -> int:
    relative = destination.relative_to(root)
    if relative.name in {"", ".", ".."}:
        raise ValueError(f"invalid provenance filename: {destination}")
    current = os.open(
        root,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    try:
        for part in relative.parts[:-1]:
            if part in {"", ".", ".."} or "/" in part:
                raise ValueError(
                    f"invalid provenance parent component: {part!r}"
                )
            try:
                os.mkdir(part, mode=0o700, dir_fd=current)
            except FileExistsError:
                pass
            else:
                os.fsync(current)
            child = os.open(
                part,
                os.O_RDONLY
                | os.O_DIRECTORY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
                dir_fd=current,
            )
            metadata = os.fstat(child)
            observed = os.stat(
                part, dir_fd=current, follow_symlinks=False
            )
            if (
                not stat.S_ISDIR(metadata.st_mode)
                or metadata.st_uid != os.geteuid()
                or metadata.st_mode & 0o022
                or (metadata.st_dev, metadata.st_ino)
                != (observed.st_dev, observed.st_ino)
            ):
                os.close(child)
                raise ValueError(
                    f"unsafe provenance parent directory: "
                    f"{destination.parent}"
                )
            os.close(current)
            current = child
        return current
    except BaseException:
        os.close(current)
        raise


def _verify_provenance_parent(
    destination: Path, parent_descriptor: int
) -> None:
    opened = os.fstat(parent_descriptor)
    observed = os.stat(destination.parent, follow_symlinks=False)
    if (
        destination.parent.resolve(strict=True) != destination.parent
        or not stat.S_ISDIR(observed.st_mode)
        or (opened.st_dev, opened.st_ino)
        != (observed.st_dev, observed.st_ino)
    ):
        raise ValueError(
            f"provenance parent changed during publish: "
            f"{destination.parent}"
        )


def _provenance_residue_identity(
    parent: int,
    name: str,
) -> os.stat_result:
    descriptor = os.open(
        name,
        os.O_RDONLY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
        dir_fd=parent,
    )
    try:
        metadata = os.fstat(descriptor)
        observed = os.stat(
            name, dir_fd=parent, follow_symlinks=False
        )
        if (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_uid != os.geteuid()
            or metadata.st_nlink not in {1, 2}
            or (metadata.st_dev, metadata.st_ino)
            != (observed.st_dev, observed.st_ino)
        ):
            raise ValueError(
                f"unsafe provenance publication residue: {name}"
            )
        return metadata
    finally:
        os.close(descriptor)


def _recover_provenance_residue(
    parent: int,
    destination_name: str,
) -> None:
    temporary_prefix = f".{destination_name}.tmp-"
    previous_prefix = f".{destination_name}.previous-"
    deadline = (
        time.monotonic() + PROVENANCE_RECOVERY_TIMEOUT_SECONDS
    )
    temporary_names: list[str] = []
    previous_names: list[str] = []
    entries = 0
    with os.scandir(parent) as scanner:
        for entry in scanner:
            entries += 1
            if entries > MAX_PROVENANCE_PARENT_ENTRIES:
                raise ValueError(
                    "provenance parent entry bound exceeded"
                )
            if time.monotonic() > deadline:
                raise TimeoutError(
                    "provenance residue recovery timed out"
                )
            if entry.name.startswith(temporary_prefix):
                temporary_names.append(entry.name)
            elif entry.name.startswith(previous_prefix):
                previous_names.append(entry.name)
    temporary_names.sort()
    previous_names.sort()
    if (
        len(temporary_names) + len(previous_names)
        > MAX_PROVENANCE_RESIDUES
    ):
        raise ValueError(
            "provenance residue count exceeded "
            f"{MAX_PROVENANCE_RESIDUES}"
        )
    for name in temporary_names:
        metadata = _provenance_residue_identity(parent, name)
        if metadata.st_nlink != 1:
            raise ValueError(
                f"unsafe provenance temporary residue: {name}"
            )
        os.unlink(name, dir_fd=parent)
    try:
        canonical = os.stat(
            destination_name,
            dir_fd=parent,
            follow_symlinks=False,
        )
    except FileNotFoundError:
        canonical = None
    if canonical is None and len(previous_names) > 1:
        raise ValueError(
            "ambiguous provenance backup recovery"
        )
    if canonical is None and previous_names:
        restored = _provenance_residue_identity(
            parent, previous_names[0]
        )
        if restored.st_nlink != 1:
            raise ValueError(
                "provenance backup has unexpected hard links"
            )
        os.rename(
            previous_names[0],
            destination_name,
            src_dir_fd=parent,
            dst_dir_fd=parent,
        )
    else:
        if canonical is not None and (
            not stat.S_ISREG(canonical.st_mode)
            or canonical.st_uid != os.geteuid()
            or canonical.st_nlink not in {1, 2}
        ):
            raise ValueError(
                "unsafe canonical provenance during recovery"
            )
        for name in previous_names:
            backup = _provenance_residue_identity(parent, name)
            if backup.st_nlink == 2 and (
                canonical is None
                or (backup.st_dev, backup.st_ino)
                != (canonical.st_dev, canonical.st_ino)
            ):
                raise ValueError(
                    "provenance backup has a foreign hard link"
                )
            os.unlink(name, dir_fd=parent)
    if temporary_names or previous_names:
        os.fsync(parent)


def write_provenance(
    root: Path,
    output: Path,
    payload: dict[str, Any],
    *,
    lock_timeout_seconds: float = 120.0,
) -> tuple[Path, str]:
    root = root.resolve(strict=True)
    destination = (
        (root / output).absolute()
        if not output.is_absolute()
        else output.absolute()
    )
    destination.relative_to(root)
    raw = bounded_json_bytes(
        payload,
        maximum_bytes=1024 * 1024,
        label="provenance payload",
    )

    parent = _open_provenance_parent(root, destination)
    lock_name = f".{destination.name}.lock"
    temporary_prefix = f".{destination.name}.tmp-"
    lock_descriptor: int | None = None
    temporary_descriptor: int | None = None
    temporary_name: str | None = None
    temporary_metadata: os.stat_result | None = None
    backup_name: str | None = None
    backup_created = False
    renamed = False
    publication_committed = False
    existing_identity: tuple[int, int] | None = None
    try:
        lock_descriptor = os.open(
            lock_name,
            os.O_RDWR
            | os.O_CREAT
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            0o600,
            dir_fd=parent,
        )
        lock_metadata = os.fstat(lock_descriptor)
        if (
            not stat.S_ISREG(lock_metadata.st_mode)
            or lock_metadata.st_uid != os.geteuid()
            or lock_metadata.st_nlink != 1
        ):
            raise ValueError("unsafe provenance publication lock")
        if (
            not math.isfinite(lock_timeout_seconds)
            or lock_timeout_seconds < 0
        ):
            raise ValueError("invalid provenance lock timeout")
        lock_started = time.monotonic()
        while True:
            try:
                fcntl.flock(
                    lock_descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB
                )
                break
            except BlockingIOError:
                waited = time.monotonic() - lock_started
                if waited >= lock_timeout_seconds:
                    raise TimeoutError(
                        "timed out waiting for provenance publication lock"
                    )
                time.sleep(
                    min(
                        0.05,
                        max(0.0, lock_timeout_seconds - waited),
                    )
                )
        lock_observed = os.stat(
            lock_name, dir_fd=parent, follow_symlinks=False
        )
        if (lock_metadata.st_dev, lock_metadata.st_ino) != (
            lock_observed.st_dev,
            lock_observed.st_ino,
        ):
            raise ValueError("provenance publication lock changed")
        _verify_provenance_parent(destination, parent)

        _recover_provenance_residue(
            parent, destination.name
        )

        try:
            existing = os.stat(
                destination.name,
                dir_fd=parent,
                follow_symlinks=False,
            )
        except FileNotFoundError:
            existing_identity = None
        else:
            if (
                not stat.S_ISREG(existing.st_mode)
                or existing.st_uid != os.geteuid()
                or existing.st_nlink != 1
            ):
                raise ValueError(
                    f"unsafe existing provenance file: {destination}"
                )
            existing_identity = (existing.st_dev, existing.st_ino)

        for _ in range(16):
            candidate = f"{temporary_prefix}{secrets.token_hex(16)}"
            try:
                temporary_descriptor = os.open(
                    candidate,
                    os.O_WRONLY
                    | os.O_CREAT
                    | os.O_EXCL
                    | os.O_CLOEXEC
                    | getattr(os, "O_NOFOLLOW", 0),
                    0o600,
                    dir_fd=parent,
                )
            except FileExistsError:
                continue
            temporary_name = candidate
            break
        if temporary_descriptor is None or temporary_name is None:
            raise FileExistsError(
                "could not allocate a private provenance temporary"
            )
        _write_all(temporary_descriptor, raw)
        os.fsync(temporary_descriptor)
        temporary_metadata = os.fstat(temporary_descriptor)
        if (
            not stat.S_ISREG(temporary_metadata.st_mode)
            or temporary_metadata.st_nlink != 1
        ):
            raise ValueError(
                "provenance temporary is not a private regular file"
            )
        os.close(temporary_descriptor)
        temporary_descriptor = None

        try:
            current = os.stat(
                destination.name,
                dir_fd=parent,
                follow_symlinks=False,
            )
        except FileNotFoundError:
            current_identity = None
        else:
            current_identity = (current.st_dev, current.st_ino)
        if current_identity != existing_identity:
            raise ValueError(
                "provenance destination changed before publish"
            )
        _verify_provenance_parent(destination, parent)
        if existing_identity is not None:
            backup_name = (
                f".{destination.name}.previous-"
                f"{secrets.token_hex(16)}"
            )
            os.link(
                destination.name,
                backup_name,
                src_dir_fd=parent,
                dst_dir_fd=parent,
                follow_symlinks=False,
            )
            backup_created = True
            os.fsync(parent)
        os.rename(
            temporary_name,
            destination.name,
            src_dir_fd=parent,
            dst_dir_fd=parent,
        )
        renamed = True
        temporary_name = None
        published = os.stat(
            destination.name,
            dir_fd=parent,
            follow_symlinks=False,
        )
        if (
            not stat.S_ISREG(published.st_mode)
            or published.st_nlink != 1
            or (published.st_dev, published.st_ino)
            != (temporary_metadata.st_dev, temporary_metadata.st_ino)
        ):
            raise ValueError("provenance identity changed during publish")
        _verify_provenance_parent(destination, parent)
        os.fsync(parent)
        publication_committed = True
        if backup_created and backup_name is not None:
            try:
                os.unlink(backup_name, dir_fd=parent)
                backup_created = False
                os.fsync(parent)
            except OSError:
                pass
    except BaseException as original:
        rollback_errors: list[str] = []
        if not publication_committed:
            try:
                try:
                    canonical = os.stat(
                        destination.name,
                        dir_fd=parent,
                        follow_symlinks=False,
                    )
                except FileNotFoundError:
                    canonical = None
                if renamed:
                    if (
                        canonical is None
                        or temporary_metadata is None
                        or (canonical.st_dev, canonical.st_ino)
                        != (
                            temporary_metadata.st_dev,
                            temporary_metadata.st_ino,
                        )
                    ):
                        raise ValueError(
                            "provenance rollback found a foreign canonical"
                        )
                    os.unlink(destination.name, dir_fd=parent)
                    canonical = None
                if backup_created and backup_name is not None:
                    backup = os.stat(
                        backup_name,
                        dir_fd=parent,
                        follow_symlinks=False,
                    )
                    if existing_identity is None or (
                        backup.st_dev,
                        backup.st_ino,
                    ) != existing_identity:
                        raise ValueError(
                            "provenance backup identity mismatch"
                        )
                    if canonical is None:
                        os.rename(
                            backup_name,
                            destination.name,
                            src_dir_fd=parent,
                            dst_dir_fd=parent,
                        )
                    elif (
                        canonical.st_dev,
                        canonical.st_ino,
                    ) == existing_identity:
                        os.unlink(backup_name, dir_fd=parent)
                    else:
                        raise ValueError(
                            "provenance rollback destination changed"
                        )
                    backup_created = False
                os.fsync(parent)
            except BaseException as rollback_error:
                rollback_errors.append(str(rollback_error))
        if rollback_errors:
            raise ValueError(
                "provenance publication rollback failed: "
                + "; ".join(rollback_errors)
            ) from original
        raise
    finally:
        cleanup_errors: list[str] = []
        try:
            if temporary_descriptor is not None:
                try:
                    os.close(temporary_descriptor)
                except OSError as exc:
                    cleanup_errors.append(str(exc))
            if temporary_name is not None:
                try:
                    observed = os.stat(
                        temporary_name,
                        dir_fd=parent,
                        follow_symlinks=False,
                    )
                    if (
                        temporary_metadata is not None
                        and (observed.st_dev, observed.st_ino)
                        != (
                            temporary_metadata.st_dev,
                            temporary_metadata.st_ino,
                        )
                    ):
                        raise ValueError(
                            "provenance temporary cleanup identity changed"
                        )
                    os.unlink(temporary_name, dir_fd=parent)
                except FileNotFoundError:
                    pass
                except (OSError, ValueError) as exc:
                    cleanup_errors.append(str(exc))
            if (
                backup_created
                and publication_committed
                and backup_name is not None
            ):
                try:
                    observed = os.stat(
                        backup_name,
                        dir_fd=parent,
                        follow_symlinks=False,
                    )
                    if (
                        existing_identity is None
                        or (observed.st_dev, observed.st_ino)
                        != existing_identity
                    ):
                        raise ValueError(
                            "provenance backup cleanup identity changed"
                        )
                    os.unlink(backup_name, dir_fd=parent)
                except FileNotFoundError:
                    pass
                except (OSError, ValueError) as exc:
                    cleanup_errors.append(str(exc))
        finally:
            try:
                if lock_descriptor is not None:
                    try:
                        fcntl.flock(
                            lock_descriptor, fcntl.LOCK_UN
                        )
                    except OSError as exc:
                        cleanup_errors.append(str(exc))
                    finally:
                        try:
                            os.close(lock_descriptor)
                        except OSError as exc:
                            cleanup_errors.append(str(exc))
            finally:
                try:
                    os.close(parent)
                except OSError as exc:
                    cleanup_errors.append(str(exc))
        if cleanup_errors:
            raise ValueError(
                "provenance publication cleanup failed: "
                + "; ".join(cleanup_errors[:8])
            )
    return destination, hashlib.sha256(raw).hexdigest()


def source_keyed_build_argv(
    root: Path,
    log: Path,
    result: Path,
    *,
    timeout: float,
    lock_timeout: float,
) -> list[str]:
    return [
        str(SOURCE_GATE_PYTHON),
        "-I",
        "-S",
        "-B",
        "-X",
        f"pycache_prefix={POLICY_PYCACHE_PREFIX}",
        str(SOURCE_KEYED_CARGO_GATE),
        "--repo",
        str(root),
        "--log",
        str(log),
        "--result",
        str(result),
        "--timeout",
        str(timeout),
        "--lock-timeout",
        str(lock_timeout),
        "--export-artifact",
        "release/codefire-rs",
        "--export-path",
        str(root / BINARY_PATH),
        "--",
        *BUILD_COMMAND,
    ]


def source_gate_launcher_environment(
    environment: dict[str, str] | None = None,
) -> dict[str, str]:
    source = os.environ if environment is None else environment
    admitted = {
        name: value
        for name, value in source.items()
        if name in SOURCE_GATE_LAUNCHER_ADMITTED_NAMES
        or name.startswith(SOURCE_GATE_LAUNCHER_ADMITTED_PREFIXES)
    }
    admitted.update(
        {
            "HOME": "/nonexistent",
            "LANG": "C",
            "LC_ALL": "C",
            "PATH": "/usr/bin:/bin",
            "PYTHONDONTWRITEBYTECODE": "1",
            "PYTHONNOUSERSITE": "1",
            "XDG_CONFIG_HOME": "/nonexistent",
        }
    )
    return admitted


def open_parent_death_helper() -> int:
    lexical = PARENT_DEATH_HELPER.absolute()
    if lexical.resolve(strict=True) != lexical:
        raise ValueError(
            f"parent-death helper identity is unsafe: {lexical}"
        )
    descriptor = os.open(
        lexical,
        os.O_RDONLY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    try:
        metadata = os.fstat(descriptor)
        observed = os.stat(lexical, follow_symlinks=False)
        if (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_uid != os.geteuid()
            or metadata.st_nlink != 1
            or metadata.st_mode & 0o022
            or metadata.st_size > 64 * 1024
            or (metadata.st_dev, metadata.st_ino)
            != (observed.st_dev, observed.st_ino)
        ):
            raise ValueError(
                f"parent-death helper identity is unsafe: {lexical}"
            )
        return descriptor
    except BaseException:
        os.close(descriptor)
        raise


def open_source_gate_supervisor() -> tuple[int, tuple[Any, ...]]:
    lexical = SOURCE_KEYED_CARGO_GATE.absolute()
    if lexical.resolve(strict=True) != lexical:
        raise ValueError(
            f"source gate supervisor identity is unsafe: {lexical}"
        )
    descriptor = os.open(
        lexical,
        os.O_RDONLY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    try:
        metadata = os.fstat(descriptor)
        observed = os.stat(lexical, follow_symlinks=False)
        if (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_uid != os.geteuid()
            or metadata.st_nlink != 1
            or metadata.st_mode & 0o022
            or metadata.st_size > 1024 * 1024
            or (metadata.st_dev, metadata.st_ino)
            != (observed.st_dev, observed.st_ino)
        ):
            raise ValueError(
                f"source gate supervisor identity is unsafe: {lexical}"
            )
        identity = (
            metadata.st_dev,
            metadata.st_ino,
            stat.S_IMODE(metadata.st_mode),
            metadata.st_uid,
            metadata.st_nlink,
            metadata.st_size,
            metadata.st_mtime_ns,
            metadata.st_ctime_ns,
            hashlib.sha256(
                os.pread(descriptor, metadata.st_size, 0)
            ).hexdigest(),
        )
        return descriptor, identity
    except BaseException:
        os.close(descriptor)
        raise


def verify_source_gate_supervisor(
    descriptor: int,
    identity: tuple[Any, ...],
) -> None:
    metadata = os.fstat(descriptor)
    observed = os.stat(
        SOURCE_KEYED_CARGO_GATE, follow_symlinks=False
    )
    current = (
        metadata.st_dev,
        metadata.st_ino,
        stat.S_IMODE(metadata.st_mode),
        metadata.st_uid,
        metadata.st_nlink,
        metadata.st_size,
        metadata.st_mtime_ns,
        metadata.st_ctime_ns,
        hashlib.sha256(
            os.pread(descriptor, metadata.st_size, 0)
        ).hexdigest(),
    )
    if (
        current != identity
        or (observed.st_dev, observed.st_ino) != current[:2]
    ):
        raise ValueError(
            "source gate supervisor changed during execution"
        )


def run_bounded_source_gate(
    argv: list[str],
    *,
    root: Path,
    environment: dict[str, str],
    timeout: float,
    maximum_output_bytes: int = 1024 * 1024,
    pass_fds: tuple[int, ...] = (),
) -> subprocess.CompletedProcess[bytes]:
    supervisor_descriptor: int | None = None
    supervisor_identity: tuple[Any, ...] | None = None
    launched_argv = list(argv)
    running_python_descriptor = (
        int(_RUNNING_BUILD_WRAPPER["python_descriptor"])
        if _RUNNING_BUILD_WRAPPER is not None
        else None
    )
    if running_python_descriptor is not None and launched_argv:
        launched_argv[0] = (
            f"/proc/self/fd/{running_python_descriptor}"
        )
    try:
        supervisor_index = launched_argv.index(
            str(SOURCE_KEYED_CARGO_GATE)
        )
    except ValueError:
        pass
    else:
        (
            supervisor_descriptor,
            supervisor_identity,
        ) = open_source_gate_supervisor()
        launched_argv[supervisor_index] = (
            f"/proc/self/fd/{supervisor_descriptor}"
        )
    try:
        helper_descriptor = open_parent_death_helper()
    except BaseException:
        if supervisor_descriptor is not None:
            os.close(supervisor_descriptor)
        raise
    launch = [
        (
            f"/proc/self/fd/{running_python_descriptor}"
            if running_python_descriptor is not None
            else str(SOURCE_GATE_PYTHON)
        ),
        "-I",
        "-S",
        "-B",
        "-X",
        f"pycache_prefix={POLICY_PYCACHE_PREFIX}",
        f"/proc/self/fd/{helper_descriptor}",
        "--parent-pid",
        str(os.getpid()),
        "--",
        str(SOURCE_GATE_PARENT_DEATH_LAUNCHER),
        *SOURCE_GATE_PARENT_DEATH_ARGS,
        str(SOURCE_GATE_PID_LAUNCHER),
        *SOURCE_GATE_PID_NAMESPACE_ARGS,
        *launched_argv,
    ]
    try:
        process = subprocess.Popen(
            launch,
            cwd=root,
            env=environment,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            pass_fds=tuple(
                sorted(
                    {
                        *pass_fds,
                        helper_descriptor,
                        *(
                            (running_python_descriptor,)
                            if running_python_descriptor is not None
                            else ()
                        ),
                        *(
                            (supervisor_descriptor,)
                            if supervisor_descriptor is not None
                            else ()
                        ),
                    }
                )
            ),
            start_new_session=True,
        )
    except BaseException:
        if supervisor_descriptor is not None:
            os.close(supervisor_descriptor)
        raise
    finally:
        os.close(helper_descriptor)
    if process.stdout is None or process.stderr is None:
        process.kill()
        process.wait()
        if supervisor_descriptor is not None:
            os.close(supervisor_descriptor)
            supervisor_descriptor = None
        raise ValueError("source gate control pipes are unavailable")
    streams = {
        process.stdout.fileno(): "stdout",
        process.stderr.fileno(): "stderr",
    }
    for descriptor in streams:
        os.set_blocking(descriptor, False)
    stdout = bytearray()
    stderr = bytearray()
    deadline = time.monotonic() + timeout
    try:
        while streams:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise subprocess.TimeoutExpired(launch, timeout)
            try:
                readable, _, _ = select.select(
                    list(streams), [], [], remaining
                )
            except InterruptedError:
                continue
            if not readable:
                raise subprocess.TimeoutExpired(launch, timeout)
            for descriptor in readable:
                try:
                    chunk = os.read(descriptor, 64 * 1024)
                except BlockingIOError:
                    continue
                if not chunk:
                    streams.pop(descriptor, None)
                    continue
                output = (
                    stdout
                    if streams[descriptor] == "stdout"
                    else stderr
                )
                output.extend(chunk)
                if len(output) > maximum_output_bytes:
                    raise ValueError(
                        "source-keyed gate control output exceeded "
                        f"{maximum_output_bytes} bytes"
                    )
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise subprocess.TimeoutExpired(launch, timeout)
        returncode = process.wait(timeout=remaining)
    except BaseException:
        if process.poll() is None:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        raise
    finally:
        try:
            process.stdout.close()
            process.stderr.close()
        finally:
            if supervisor_descriptor is not None:
                try:
                    if supervisor_identity is not None:
                        verify_source_gate_supervisor(
                            supervisor_descriptor,
                            supervisor_identity,
                        )
                finally:
                    os.close(supervisor_descriptor)
                    supervisor_descriptor = None
    return subprocess.CompletedProcess(
        launch, returncode, bytes(stdout), bytes(stderr)
    )


def bound_gate_evidence(
    evidence_root: Path,
    path: Path,
    *,
    maximum_bytes: int,
    include_bytes: bool,
) -> dict[str, Any]:
    root = evidence_root.resolve(strict=True)
    lexical = path.absolute()
    if lexical.parent != root or lexical.name in {"", ".", ".."}:
        raise ValueError(
            f"gate evidence is not a direct child of {root}: {lexical}"
        )
    parent = os.open(
        root,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    descriptor: int | None = None
    try:
        descriptor = os.open(
            lexical.name,
            os.O_RDONLY
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            dir_fd=parent,
        )
        before = os.fstat(descriptor)
        observed = os.stat(
            lexical.name, dir_fd=parent, follow_symlinks=False
        )
        if (
            not stat.S_ISREG(before.st_mode)
            or before.st_uid != os.geteuid()
            or before.st_nlink != 1
            or stat.S_IMODE(before.st_mode) != 0o400
            or (before.st_dev, before.st_ino)
            != (observed.st_dev, observed.st_ino)
            or before.st_size > maximum_bytes
        ):
            raise ValueError(f"unsafe gate evidence identity: {lexical}")
        digest = hashlib.sha256()
        chunks: list[bytes] = []
        size = 0
        while True:
            chunk = os.read(descriptor, 1024 * 1024)
            if not chunk:
                break
            size += len(chunk)
            if size > maximum_bytes:
                raise ValueError(
                    f"gate evidence exceeds {maximum_bytes} bytes: "
                    f"{lexical}"
                )
            digest.update(chunk)
            if include_bytes:
                chunks.append(chunk)
        after = os.fstat(descriptor)
        current = os.stat(
            lexical.name, dir_fd=parent, follow_symlinks=False
        )
        if (
            (
                before.st_dev,
                before.st_ino,
                before.st_mode,
                before.st_nlink,
                before.st_size,
                before.st_mtime_ns,
                before.st_ctime_ns,
            )
            != (
                after.st_dev,
                after.st_ino,
                after.st_mode,
                after.st_nlink,
                after.st_size,
                after.st_mtime_ns,
                after.st_ctime_ns,
            )
            or (after.st_dev, after.st_ino)
            != (current.st_dev, current.st_ino)
            or size != after.st_size
        ):
            raise ValueError(
                f"gate evidence changed while rehashed: {lexical}"
            )
        return {
            "path": str(lexical),
            "bytes": size,
            "sha256": digest.hexdigest(),
            "raw": b"".join(chunks) if include_bytes else None,
        }
    finally:
        if descriptor is not None:
            os.close(descriptor)
        os.close(parent)


def read_bound_provenance(
    root: Path,
    relative: Path = DEFAULT_OUTPUT,
    *,
    maximum_bytes: int = 1024 * 1024,
) -> tuple[dict[str, Any], dict[str, Any]]:
    root = root.resolve(strict=True)
    if (
        relative.is_absolute()
        or len(relative.parts) < 2
        or relative.parts[0] != "target"
        or any(part in {"", ".", ".."} for part in relative.parts)
    ):
        raise ValueError(f"unsafe provenance path: {relative}")
    directory = os.open(
        root,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    descriptor: int | None = None
    try:
        for part in relative.parts[:-1]:
            child = os.open(
                part,
                os.O_RDONLY
                | os.O_DIRECTORY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
                dir_fd=directory,
            )
            metadata = os.fstat(child)
            observed = os.stat(
                part, dir_fd=directory, follow_symlinks=False
            )
            if (
                not stat.S_ISDIR(metadata.st_mode)
                or metadata.st_uid != os.geteuid()
                or metadata.st_mode & 0o022
                or (metadata.st_dev, metadata.st_ino)
                != (observed.st_dev, observed.st_ino)
            ):
                os.close(child)
                raise ValueError(
                    f"unsafe provenance ancestor: {root / relative}"
                )
            os.close(directory)
            directory = child
        descriptor = os.open(
            relative.name,
            os.O_RDONLY
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            dir_fd=directory,
        )
        before = os.fstat(descriptor)
        observed = os.stat(
            relative.name, dir_fd=directory, follow_symlinks=False
        )
        if (
            not stat.S_ISREG(before.st_mode)
            or before.st_uid != os.geteuid()
            or before.st_nlink != 1
            or before.st_size > maximum_bytes
            or (before.st_dev, before.st_ino)
            != (observed.st_dev, observed.st_ino)
        ):
            raise ValueError(
                f"unsafe provenance identity: {root / relative}"
            )
        digest = hashlib.sha256()
        chunks: list[bytes] = []
        size = 0
        while True:
            chunk = os.read(descriptor, 1024 * 1024)
            if not chunk:
                break
            size += len(chunk)
            if size > maximum_bytes:
                raise ValueError(
                    f"provenance exceeds {maximum_bytes} bytes"
                )
            digest.update(chunk)
            chunks.append(chunk)
        after = os.fstat(descriptor)
        current = os.stat(
            relative.name, dir_fd=directory, follow_symlinks=False
        )
        if (
            (
                before.st_dev,
                before.st_ino,
                before.st_mode,
                before.st_nlink,
                before.st_size,
                before.st_mtime_ns,
                before.st_ctime_ns,
            )
            != (
                after.st_dev,
                after.st_ino,
                after.st_mode,
                after.st_nlink,
                after.st_size,
                after.st_mtime_ns,
                after.st_ctime_ns,
            )
            or (after.st_dev, after.st_ino)
            != (current.st_dev, current.st_ino)
            or size != after.st_size
        ):
            raise ValueError("provenance changed while it was read")
        raw = b"".join(chunks)
        loaded = json.loads(raw)
        if not isinstance(loaded, dict):
            raise ValueError("provenance root must be an object")
        return loaded, {
            "path": relative.as_posix(),
            "bytes": size,
            "sha256": digest.hexdigest(),
            "device": after.st_dev,
            "inode": after.st_ino,
            "mode": stat.S_IMODE(after.st_mode),
            "nlink": after.st_nlink,
        }
    finally:
        if descriptor is not None:
            os.close(descriptor)
        os.close(directory)


def bound_binary_identity(root: Path, relative: Path) -> dict[str, Any]:
    if (
        relative.is_absolute()
        or len(relative.parts) < 2
        or relative.parts[0] != "target"
        or any(part in {"", ".", ".."} for part in relative.parts)
    ):
        raise ValueError(f"unsafe binary path: {relative}")
    root_descriptor = os.open(
        root,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    parent_descriptor = os.dup(root_descriptor)
    descriptor: int | None = None
    try:
        for part in relative.parts[:-1]:
            child = os.open(
                part,
                os.O_RDONLY
                | os.O_DIRECTORY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
                dir_fd=parent_descriptor,
            )
            metadata = os.fstat(child)
            if (
                not stat.S_ISDIR(metadata.st_mode)
                or metadata.st_uid != os.geteuid()
                or metadata.st_mode & 0o022
            ):
                os.close(child)
                raise ValueError(
                    f"unsafe binary ancestor: {root / relative}"
                )
            os.close(parent_descriptor)
            parent_descriptor = child
        descriptor = os.open(
            relative.name,
            os.O_RDONLY
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            dir_fd=parent_descriptor,
        )
        before = os.fstat(descriptor)
        if (
            not stat.S_ISREG(before.st_mode)
            or before.st_nlink != 1
            or not before.st_mode & stat.S_IXUSR
        ):
            raise ValueError("bound binary is not a private executable")
        digest = hashlib.sha256()
        size = 0
        while True:
            chunk = os.read(descriptor, 1024 * 1024)
            if not chunk:
                break
            digest.update(chunk)
            size += len(chunk)
        after = os.fstat(descriptor)
        observed = os.stat(
            relative.name,
            dir_fd=parent_descriptor,
            follow_symlinks=False,
        )
        if (
            (
                before.st_dev,
                before.st_ino,
                before.st_mode,
                before.st_nlink,
                before.st_size,
                before.st_mtime_ns,
                before.st_ctime_ns,
            )
            != (
                after.st_dev,
                after.st_ino,
                after.st_mode,
                after.st_nlink,
                after.st_size,
                after.st_mtime_ns,
                after.st_ctime_ns,
            )
            or (observed.st_dev, observed.st_ino)
            != (after.st_dev, after.st_ino)
            or size != after.st_size
        ):
            raise ValueError("bound binary changed while it was verified")
        return {
            "path": str(root / relative),
            "bytes": size,
            "sha256": digest.hexdigest(),
            "device": after.st_dev,
            "inode": after.st_ino,
            "mode": stat.S_IMODE(after.st_mode),
            "nlink": after.st_nlink,
        }
    finally:
        if descriptor is not None:
            os.close(descriptor)
        os.close(parent_descriptor)
        os.close(root_descriptor)


def gate_evidence_usage(evidence_root: Path) -> dict[str, int]:
    entries = 0
    allocated = 0
    logical = 0
    stack = [evidence_root]
    deadline = (
        time.monotonic() + GATE_EVIDENCE_SCAN_TIMEOUT_SECONDS
    )
    while stack:
        directory = stack.pop()
        with os.scandir(directory) as scanner:
            for entry in scanner:
                if time.monotonic() > deadline:
                    raise TimeoutError(
                        "gate evidence usage scan timed out"
                    )
                entries += 1
                if entries > MAX_GATE_EVIDENCE_ENTRIES:
                    raise ValueError(
                        "gate evidence root entry limit exceeded"
                    )
                metadata = entry.stat(follow_symlinks=False)
                if stat.S_ISLNK(metadata.st_mode):
                    raise ValueError(
                        f"gate evidence root contains a symlink: "
                        f"{entry.path}"
                    )
                allocated += metadata.st_blocks * 512
                logical += metadata.st_size
                if stat.S_ISDIR(metadata.st_mode):
                    stack.append(Path(entry.path))
                elif not stat.S_ISREG(metadata.st_mode):
                    raise ValueError(
                        "gate evidence root contains an unsupported "
                        f"entry: {entry.path}"
                    )
    return {
        "entry_count": entries,
        "allocated_bytes": allocated,
        "logical_bytes": logical,
    }


def acquire_gate_evidence_lock(
    evidence_root: Path,
    timeout_seconds: float,
    *,
    create: bool = True,
) -> tuple[int, float]:
    if not math.isfinite(timeout_seconds) or timeout_seconds < 0:
        raise ValueError("gate evidence lock timeout must be finite")
    root = evidence_root.resolve(strict=True)
    flags = (
        os.O_RDWR
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0)
    )
    if create:
        flags |= os.O_CREAT
    try:
        descriptor = os.open(
            root / GATE_EVIDENCE_LOCK_NAME,
            flags,
            0o600,
        )
    except FileNotFoundError as exc:
        raise ValueError(
            "gate evidence lock is missing"
        ) from exc
    metadata = os.fstat(descriptor)
    observed = os.stat(
        root / GATE_EVIDENCE_LOCK_NAME, follow_symlinks=False
    )
    if (
        not stat.S_ISREG(metadata.st_mode)
        or metadata.st_uid != os.geteuid()
        or metadata.st_nlink != 1
        or metadata.st_mode & 0o077
        or (metadata.st_dev, metadata.st_ino)
        != (observed.st_dev, observed.st_ino)
    ):
        os.close(descriptor)
        raise ValueError("gate evidence lock identity is unsafe")
    started = time.monotonic()
    deadline = started + timeout_seconds
    while True:
        try:
            fcntl.flock(
                descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB
            )
            return descriptor, time.monotonic() - started
        except BlockingIOError:
            if time.monotonic() >= deadline:
                os.close(descriptor)
                raise TimeoutError(
                    "gate evidence lock acquisition timed out"
                )
            time.sleep(min(0.05, max(0.0, deadline - time.monotonic())))


def validate_gate_evidence_lock_descriptor(
    evidence_root: Path, descriptor: int
) -> None:
    root = evidence_root.resolve(strict=True)
    metadata = os.fstat(descriptor)
    observed = os.stat(
        root / GATE_EVIDENCE_LOCK_NAME, follow_symlinks=False
    )
    if (
        not stat.S_ISREG(metadata.st_mode)
        or metadata.st_uid != os.geteuid()
        or metadata.st_nlink != 1
        or metadata.st_mode & 0o077
        or (metadata.st_dev, metadata.st_ino)
        != (observed.st_dev, observed.st_ino)
    ):
        raise ValueError(
            "inherited gate evidence lock identity is unsafe"
        )
    fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)


def release_gate_evidence_lock(descriptor: int) -> None:
    try:
        fcntl.flock(descriptor, fcntl.LOCK_UN)
    finally:
        os.close(descriptor)


class CanonicalProvenanceChanged(ValueError):
    def __init__(self, phase: str) -> None:
        self.phase = phase
        super().__init__(
            f"canonical provenance changed {phase}"
        )


class GateEvidenceLockUnavailable(ValueError):
    pass


def validate_locked_canonical_provenance(
    root: Path,
    *,
    relative: Path = DEFAULT_OUTPUT,
    expected_revision: str,
    expected_sha256: str | None = None,
    expected_bytes: int | None = None,
    lock_timeout_seconds: float = 120.0,
) -> tuple[dict[str, Any], dict[str, Any], list[str]]:
    """Validate one canonical provenance generation under its evidence lock."""

    initial_payload, initial_identity = read_bound_provenance(
        root, relative
    )
    if (
        expected_sha256 is not None
        and initial_identity["sha256"] != expected_sha256
    ) or (
        expected_bytes is not None
        and initial_identity["bytes"] != expected_bytes
    ):
        raise CanonicalProvenanceChanged("before-lock")
    destination_contract = initial_payload.get(
        "provenance_destination_contract"
    )
    canonical_path = (
        destination_contract.get("relative_path")
        if isinstance(destination_contract, dict)
        else None
    )
    if canonical_path != relative.as_posix():
        raise ValueError(
            "provenance was not read from its bound canonical path: "
            f"{canonical_path!r}"
        )
    evidence_root = gate_evidence_root_contract(
        root,
        initial_payload.get("source_keyed_gate_evidence_root"),
    )
    try:
        lock_descriptor, lock_wait_seconds = (
            acquire_gate_evidence_lock(
                evidence_root,
                lock_timeout_seconds,
                create=False,
            )
        )
    except (OSError, TimeoutError, ValueError) as exc:
        raise GateEvidenceLockUnavailable(str(exc)) from exc
    try:
        locked_payload, locked_identity = read_bound_provenance(
            root, relative
        )
        if (
            locked_identity["sha256"]
            != initial_identity["sha256"]
            or locked_identity["bytes"] != initial_identity["bytes"]
            or locked_payload != initial_payload
            or (
                expected_sha256 is not None
                and locked_identity["sha256"] != expected_sha256
            )
            or (
                expected_bytes is not None
                and locked_identity["bytes"] != expected_bytes
            )
        ):
            raise CanonicalProvenanceChanged("before-lock")
        source_before_head = git_head(root)
        source_before_manifest = build_source_manifest(root)
        errors = validate_provenance_payload(
            root,
            locked_payload,
            expected_revision=expected_revision,
        )
        try:
            final_binary = bound_binary_identity(root, BINARY_PATH)
        except (FileNotFoundError, OSError, ValueError) as exc:
            errors.append(
                f"provenance binary is unavailable after validation: {exc}"
            )
        else:
            if (
                locked_payload.get("binary_bytes")
                != final_binary["bytes"]
                or locked_payload.get("binary_sha256")
                != final_binary["sha256"]
            ):
                errors.append(
                    "provenance binary changed during locked validation"
                )
        source_after_head = git_head(root)
        source_after_manifest = build_source_manifest(root)
        final_source_errors = validate_source_inputs_against_head(
            root, expected_revision
        )
        if (
            source_before_head != source_after_head
            or source_before_manifest != source_after_manifest
        ):
            raise CanonicalProvenanceChanged(
                "during-source-validation"
            )
        if source_after_head != expected_revision:
            errors.append(
                "source HEAD changed during locked validation"
            )
        if (
            locked_payload.get("source_manifest_sha256")
            != source_after_manifest["manifest_sha256"]
            or locked_payload.get("source_file_count")
            != len(source_after_manifest["files"])
        ):
            errors.append(
                "provenance source manifest changed during validation"
            )
        errors.extend(final_source_errors)
        final_payload, final_identity = read_bound_provenance(
            root, relative
        )
        if (
            final_identity["sha256"] != locked_identity["sha256"]
            or final_identity["bytes"] != locked_identity["bytes"]
            or final_payload != locked_payload
        ):
            raise CanonicalProvenanceChanged("under-lock")
        return locked_payload, {
            **locked_identity,
            "gate_evidence_lock_wait_seconds": lock_wait_seconds,
        }, errors
    finally:
        release_gate_evidence_lock(lock_descriptor)


def gate_evidence_admission(
    evidence_root: Path,
    *,
    reserve_attempt: bool,
    additional_reserve_bytes: int = 0,
    additional_entry_reserve: int = 0,
) -> dict[str, int]:
    if additional_reserve_bytes < 0 or additional_entry_reserve < 0:
        raise ValueError("gate evidence reserves must be non-negative")
    usage = gate_evidence_usage(evidence_root)
    reserve = (
        GATE_EVIDENCE_ATTEMPT_RESERVE_BYTES
        if reserve_attempt
        else 0
    ) + additional_reserve_bytes
    entry_reserve = (
        2 if reserve_attempt else 0
    ) + additional_entry_reserve
    projected = usage["allocated_bytes"] + reserve
    free = shutil.disk_usage(evidence_root).free
    free_after_reserve = free - reserve
    if projected > MAX_GATE_EVIDENCE_ALLOCATED_BYTES:
        raise ValueError(
            "gate evidence allocation budget exceeded: "
            f"observed={usage['allocated_bytes']}, "
            f"reserve={reserve}, projected={projected}, "
            f"budget={MAX_GATE_EVIDENCE_ALLOCATED_BYTES}"
        )
    if usage["entry_count"] + entry_reserve > (
        MAX_GATE_EVIDENCE_ENTRIES
    ):
        raise ValueError("gate evidence entry budget would be exceeded")
    if free_after_reserve < GATE_EVIDENCE_FREE_FLOOR_BYTES:
        raise ValueError(
            "gate evidence filesystem free floor would be crossed: "
            f"free={free}, reserve={reserve}, "
            f"remaining={free_after_reserve}, "
            f"floor={GATE_EVIDENCE_FREE_FLOOR_BYTES}"
        )
    return {
        **usage,
        "reserve_bytes": reserve,
        "entry_reserve": entry_reserve,
        "projected_allocated_bytes": projected,
        "filesystem_free_bytes": free,
        "filesystem_free_after_reserve_bytes": free_after_reserve,
        "aggregate_budget_bytes": MAX_GATE_EVIDENCE_ALLOCATED_BYTES,
        "entry_budget": MAX_GATE_EVIDENCE_ENTRIES,
        "free_floor_bytes": GATE_EVIDENCE_FREE_FLOOR_BYTES,
    }


def remove_gate_evidence_file(
    evidence_root: Path, path: Path
) -> bool:
    root = evidence_root.resolve(strict=True)
    lexical = path.absolute()
    if lexical.parent != root or lexical.name in {"", ".", ".."}:
        raise ValueError(
            f"gate evidence cleanup escaped its root: {lexical}"
        )
    parent = os.open(
        root,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    descriptor: int | None = None
    try:
        try:
            descriptor = os.open(
                lexical.name,
                os.O_RDONLY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
                dir_fd=parent,
            )
        except FileNotFoundError:
            return False
        opened = os.fstat(descriptor)
        observed = os.stat(
            lexical.name, dir_fd=parent, follow_symlinks=False
        )
        if (
            not stat.S_ISREG(opened.st_mode)
            or opened.st_uid != os.geteuid()
            or opened.st_nlink != 1
            or (opened.st_dev, opened.st_ino)
            != (observed.st_dev, observed.st_ino)
        ):
            raise ValueError(
                f"unsafe gate evidence cleanup identity: {lexical}"
            )
        os.unlink(lexical.name, dir_fd=parent)
        os.fsync(parent)
        return True
    finally:
        if descriptor is not None:
            os.close(descriptor)
        os.close(parent)


def recover_gate_result_publication_residue(
    evidence_root: Path,
    result_paths: list[Path],
) -> dict[str, Any]:
    root = evidence_root.resolve(strict=True)
    parent = os.open(
        root,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    deadline = time.monotonic() + 10.0
    prefixes = {
        path.name: (
            f".{path.name}.tmp-",
            f".{path.name}.reserved-",
        )
        for path in result_paths
    }
    residues: dict[str, dict[str, list[str]]] = {
        name: {"temporary": [], "reserved": []}
        for name in prefixes
    }
    total_entries = 0
    try:
        with os.scandir(parent) as scanner:
            for entry in scanner:
                total_entries += 1
                if total_entries > MAX_GATE_EVIDENCE_ENTRIES:
                    raise ValueError(
                        "gate evidence root entry limit exceeded"
                    )
                if time.monotonic() > deadline:
                    raise TimeoutError(
                        "gate result residue recovery timed out"
                    )
                for canonical, (
                    temporary_prefix,
                    reserved_prefix,
                ) in prefixes.items():
                    if entry.name.startswith(temporary_prefix):
                        residues[canonical]["temporary"].append(
                            entry.name
                        )
                        break
                    if entry.name.startswith(reserved_prefix):
                        residues[canonical]["reserved"].append(
                            entry.name
                        )
                        break
        residue_count = sum(
            len(kind)
            for grouped in residues.values()
            for kind in grouped.values()
        )
        if residue_count > 64:
            raise ValueError(
                "gate result publication residue count exceeded 64"
            )
        changed = False
        for canonical, grouped in sorted(residues.items()):
            temporary_names = sorted(grouped["temporary"])
            reserved_names = sorted(grouped["reserved"])
            for name in temporary_names:
                metadata = _provenance_residue_identity(
                    parent, name
                )
                if metadata.st_nlink != 1:
                    raise ValueError(
                        f"unsafe gate result temporary: {name}"
                    )
                os.unlink(name, dir_fd=parent)
                changed = True
            try:
                canonical_metadata = os.stat(
                    canonical,
                    dir_fd=parent,
                    follow_symlinks=False,
                )
            except FileNotFoundError:
                canonical_metadata = None
            if canonical_metadata is None and len(reserved_names) > 1:
                raise ValueError(
                    "ambiguous gate result reservation recovery"
                )
            if canonical_metadata is None and reserved_names:
                restored = _provenance_residue_identity(
                    parent, reserved_names[0]
                )
                if restored.st_nlink != 1:
                    raise ValueError(
                        "gate result reservation has foreign links"
                    )
                os.rename(
                    reserved_names[0],
                    canonical,
                    src_dir_fd=parent,
                    dst_dir_fd=parent,
                )
                changed = True
                continue
            if canonical_metadata is not None and (
                not stat.S_ISREG(canonical_metadata.st_mode)
                or canonical_metadata.st_uid != os.geteuid()
                or canonical_metadata.st_nlink not in {1, 2}
            ):
                raise ValueError(
                    f"unsafe canonical gate result: {canonical}"
                )
            for name in reserved_names:
                backup = _provenance_residue_identity(
                    parent, name
                )
                if backup.st_nlink == 2 and (
                    canonical_metadata is None
                    or (backup.st_dev, backup.st_ino)
                    != (
                        canonical_metadata.st_dev,
                        canonical_metadata.st_ino,
                    )
                ):
                    raise ValueError(
                        "gate result reservation has a foreign link"
                    )
                os.unlink(name, dir_fd=parent)
                changed = True
        if changed:
            os.fsync(parent)
        return {
            "decision": "GATE_RESULT_RESIDUE_RECOVERED",
            "residue_count": residue_count,
            "changed": changed,
        }
    finally:
        os.close(parent)


def prepare_gate_evidence_slot(
    root: Path,
    output: Path,
    evidence_root: Path,
    *,
    lock_descriptor: int,
) -> dict[str, Any]:
    validate_gate_evidence_lock_descriptor(
        evidence_root, lock_descriptor
    )
    canonical_root = root.resolve(strict=True)
    output_absolute = (
        (canonical_root / output).absolute()
        if not output.is_absolute()
        else output.absolute()
    )
    output_relative = output_absolute.relative_to(canonical_root)
    scope = hashlib.sha256(
        (
            str(canonical_root)
            + "\0"
            + output_relative.as_posix()
        ).encode("utf-8")
    ).hexdigest()[:24]
    names = [
        {
            "slot": slot,
            "log": evidence_root
            / f"rust-release-build-{scope}-{slot}.log",
            "result": evidence_root
            / f"rust-release-build-{scope}-{slot}.json",
        }
        for slot in range(GATE_EVIDENCE_SLOT_COUNT)
    ]
    residue_recovery = recover_gate_result_publication_residue(
        evidence_root,
        [Path(str(item["result"])) for item in names],
    )
    active_slot: int | None = None
    try:
        prior, _identity = read_bound_provenance(
            canonical_root, output_relative
        )
    except FileNotFoundError:
        prior = None
    if prior is not None:
        destination_contract = prior.get(
            "provenance_destination_contract"
        )
        retention = prior.get("gate_evidence_retention")
        selected_slot = (
            retention.get("selected_slot")
            if isinstance(retention, dict)
            else None
        )
        selected_prior = (
            names[selected_slot]
            if type(selected_slot) is int
            and 0 <= selected_slot < len(names)
            else None
        )
        if (
            prior.get("schema") != "codefire.rust_build_provenance.v1"
            or prior.get("ok") is not True
            or not isinstance(destination_contract, dict)
            or destination_contract.get("decision")
            != "UNTRACKED_IGNORED_REPOSITORY_TARGET"
            or destination_contract.get("relative_path")
            != output_relative.as_posix()
            or destination_contract.get("tracked") is not False
            or destination_contract.get("ignored") is not True
            or prior.get("source_keyed_gate_evidence_root")
            != str(evidence_root)
            or not isinstance(retention, dict)
            or retention.get("schema")
            != "codefire.rust_gate_evidence_slot.v1"
            or retention.get("scope") != scope
            or retention.get("aggregate_budget_bytes")
            != MAX_GATE_EVIDENCE_ALLOCATED_BYTES
            or retention.get("entry_budget")
            != MAX_GATE_EVIDENCE_ENTRIES
            or retention.get("attempt_reserve_bytes")
            != GATE_EVIDENCE_ATTEMPT_RESERVE_BYTES
            or selected_prior is None
            or prior.get("build_log_path")
            != str(selected_prior["log"])
            or prior.get("source_keyed_gate_result_path")
            != str(selected_prior["result"])
        ):
            raise ValueError(
                "prior canonical provenance does not identify one "
                "complete evidence slot; no evidence was removed"
            )
        active_slot = int(selected_slot)
    selected_slot = (
        0 if active_slot is None else 1 - active_slot
    )
    selected = names[selected_slot]
    removed = [
        str(path)
        for path in (selected["log"], selected["result"])
        if remove_gate_evidence_file(evidence_root, path)
    ]
    admission = gate_evidence_admission(
        evidence_root, reserve_attempt=True
    )
    return {
        "schema": "codefire.rust_gate_evidence_slot.v1",
        "scope": scope,
        "active_slot_before": active_slot,
        "selected_slot": selected_slot,
        "log": selected["log"],
        "result": selected["result"],
        "removed_inactive_paths": removed,
        "publication_residue_recovery": residue_recovery,
        "usage_before": admission,
        "attempt_reserve_bytes": (
            GATE_EVIDENCE_ATTEMPT_RESERVE_BYTES
        ),
        "aggregate_budget_bytes": (
            MAX_GATE_EVIDENCE_ALLOCATED_BYTES
        ),
        "entry_budget": MAX_GATE_EVIDENCE_ENTRIES,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=repository_root())
    parser.add_argument("--timeout", type=float, default=1200.0)
    parser.add_argument("--lock-timeout", type=float, default=6 * 60 * 60)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()
    root = args.root.resolve(strict=True)
    started = time.monotonic()
    evidence_lock_descriptor: int | None = None
    try:
        if (
            not math.isfinite(args.timeout)
            or args.timeout <= 0
            or not math.isfinite(args.lock_timeout)
            or args.lock_timeout < 0
        ):
            raise ValueError(
                "timeouts must be finite and non-negative"
            )
        preflight_deadline = (
            time.monotonic() + SOURCE_PREFLIGHT_TIMEOUT_SECONDS
        )
        base_revision = git_head(root)
        source_errors = validate_source_inputs_against_head(
            root,
            base_revision,
            deadline=preflight_deadline,
        )
        if source_errors:
            raise ValueError("; ".join(source_errors))
        before = build_source_manifest(
            root, deadline=preflight_deadline
        )
        output_contract = provenance_destination_contract(
            root, args.output
        )
        requested_evidence_root = DEFAULT_GATE_EVIDENCE_ROOT.absolute()
        requested_evidence_root.mkdir(
            mode=0o700, parents=True, exist_ok=True
        )
        evidence_root = gate_evidence_root_contract(
            root, str(requested_evidence_root)
        )
        (
            evidence_lock_descriptor,
            evidence_lock_wait_seconds,
        ) = acquire_gate_evidence_lock(
            evidence_root, args.lock_timeout
        )
        evidence_slot = prepare_gate_evidence_slot(
            root,
            args.output,
            evidence_root,
            lock_descriptor=evidence_lock_descriptor,
        )
        gate_log = Path(str(evidence_slot["log"]))
        gate_result_path = Path(str(evidence_slot["result"]))
        launcher_environment = source_gate_launcher_environment()
        launcher_environment[
            "CODEFIRE_GATE_EVIDENCE_LOCK_FD"
        ] = str(evidence_lock_descriptor)
        gate_process = run_bounded_source_gate(
            source_keyed_build_argv(
                root,
                gate_log,
                gate_result_path,
                timeout=args.timeout,
                lock_timeout=args.lock_timeout,
            ),
            root=root,
            environment=launcher_environment,
            timeout=(
                args.timeout
                + args.lock_timeout
                + SOURCE_GATE_CONTROL_OVERHEAD_SECONDS
            ),
            pass_fds=(evidence_lock_descriptor,),
        )
        try:
            result_evidence = bound_gate_evidence(
                evidence_root,
                gate_result_path,
                maximum_bytes=MAX_GATE_RESULT_BYTES,
                include_bytes=True,
            )
            gate_raw = result_evidence["raw"]
            gate_result = json.loads(gate_raw)
        except (FileNotFoundError, json.JSONDecodeError, OSError) as exc:
            raise ValueError(
                f"source-keyed build result is unavailable: {exc}"
            ) from exc
        if gate_process.stdout != gate_raw:
            raise ValueError(
                "source-keyed gate result file differs from captured stdout"
            )
        if gate_process.returncode == 0 and gate_process.stderr:
            raise ValueError(
                "source-keyed gate emitted unexpected control stderr"
            )
        if (
            gate_process.returncode != 0
            or not isinstance(gate_result, dict)
            or gate_result.get("decision") != "PASS"
            or gate_result.get("argv") != BUILD_COMMAND
        ):
            payload = {
                "schema": "codefire.rust_build_provenance.v1",
                "ok": False,
                "build_command": command_text(BUILD_COMMAND),
                "source_keyed_gate_returncode": gate_process.returncode,
                "source_keyed_gate_decision": (
                    gate_result.get("decision")
                    if isinstance(gate_result, dict)
                    else None
                ),
                "source_keyed_gate_result_sha256": hashlib.sha256(
                    gate_raw
                ).hexdigest(),
                "source_keyed_gate_result_path": str(gate_result_path),
            }
            print(json.dumps(payload, indent=2, sort_keys=True))
            return 1
        outer_policy_binding = validate_outer_policy_binding(
            gate_result
        )
        gate_log_evidence = bound_gate_evidence(
            evidence_root,
            gate_log,
            maximum_bytes=256 * 1024 * 1024,
            include_bytes=False,
        )
        recorded_log = gate_result.get("gate", {}).get("log", {})
        if (
            not isinstance(recorded_log, dict)
            or recorded_log.get("path") != str(gate_log)
            or recorded_log.get("bytes")
            != gate_log_evidence["bytes"]
            or recorded_log.get("sha256")
            != gate_log_evidence["sha256"]
            or recorded_log.get("complete") is not True
        ):
            raise ValueError(
                "source-keyed gate log changed or differs from result evidence"
            )
        candidate_evidence = gate_result.get("candidate")
        if (
            not isinstance(candidate_evidence, dict)
            or candidate_evidence.get("candidate_root") != str(root)
            or candidate_evidence.get("head_revision") != base_revision
            or candidate_evidence.get("dirty") is not False
        ):
            raise ValueError(
                "source-keyed gate candidate evidence does not match the build request"
            )
        target = gate_result.get("target")
        if not isinstance(target, dict):
            raise ValueError("source-keyed gate target evidence is missing")
        namespace = Path(str(target.get("namespace", ""))).resolve(
            strict=True
        )
        admitted_root = Path(
            str(
                gate_result.get("execution_profile", {}).get(
                    "admitted_target_root", ""
                )
            )
        ).resolve(strict=True)
        try:
            namespace.relative_to(admitted_root)
        except ValueError as exc:
            raise ValueError(
                "source-keyed build namespace is outside the admitted root"
            ) from exc
        destination_binary = root / BINARY_PATH
        artifact_export = gate_result.get("artifact_export")
        export_contract = (
            artifact_export.get("destination_contract")
            if isinstance(artifact_export, dict)
            else None
        )
        if (
            not isinstance(artifact_export, dict)
            or artifact_export.get("decision") != "EXPORTED"
            or artifact_export.get("artifact_relative_path")
            != "release/codefire-rs"
            or artifact_export.get("target_artifact_path")
            != str(namespace / "release/codefire-rs")
            or artifact_export.get("export_path")
            != str(destination_binary)
            or not isinstance(export_contract, dict)
            or export_contract.get("decision")
            != "UNTRACKED_IGNORED_CANDIDATE_TARGET"
            or export_contract.get("relative_path")
            != BINARY_PATH.as_posix()
            or export_contract.get("tracked") is not False
            or export_contract.get("ignored") is not True
        ):
            raise ValueError(
                "source-keyed gate artifact export evidence is missing or mismatched"
            )
        binary_identity = bound_binary_identity(root, BINARY_PATH)
        for evidence_name, identity_name in (
            ("bytes", "bytes"),
            ("sha256", "sha256"),
            ("export_device", "device"),
            ("export_inode", "inode"),
            ("export_mode", "mode"),
            ("export_nlink", "nlink"),
        ):
            if artifact_export.get(evidence_name) != binary_identity[
                identity_name
            ]:
                raise ValueError(
                    "exported binary differs from source-keyed artifact evidence"
                )
        binary_size = int(binary_identity["bytes"])
        binary_sha256 = str(binary_identity["sha256"])
        postflight_deadline = (
            time.monotonic() + SOURCE_PREFLIGHT_TIMEOUT_SECONDS
        )
        after = build_source_manifest(
            root, deadline=postflight_deadline
        )
        if after != before:
            raise ValueError("Rust source inputs changed during the build; provenance rejected")
        if git_head(root) != base_revision:
            raise ValueError("HEAD changed during the build; provenance rejected")
        source_errors = validate_source_inputs_against_head(
            root,
            base_revision,
            deadline=postflight_deadline,
        )
        if source_errors:
            raise ValueError("; ".join(source_errors))
        toolchain = gate_result.get("toolchain_identity")
        if not isinstance(toolchain, dict) or not isinstance(
            toolchain.get("executables"), dict
        ):
            raise ValueError("source-keyed toolchain evidence is missing")
        executables = toolchain["executables"]
        evidence_postflight = gate_evidence_admission(
            evidence_root, reserve_attempt=False
        )
        payload = {
            "schema": "codefire.rust_build_provenance.v1",
            "ok": True,
            "base_revision": base_revision,
            "source_manifest_sha256": before["manifest_sha256"],
            "source_file_count": len(before["files"]),
            "cargo_version": executables["cargo"]["version"],
            "rustc_version": executables["rustc"]["version"],
            "build_command": command_text(BUILD_COMMAND),
            "build_log_path": str(gate_log),
            "build_log_bytes": gate_log_evidence["bytes"],
            "build_log_sha256": gate_log_evidence["sha256"],
            "source_keyed_gate_evidence_root": str(evidence_root),
            "source_keyed_gate_result_path": str(gate_result_path),
            "source_keyed_gate_result_bytes": result_evidence["bytes"],
            "source_keyed_gate_result_sha256": result_evidence["sha256"],
            "source_gate_launcher_environment_sha256": sha256_json(
                launcher_environment
            ),
            "source_gate_launcher_environment_names": sorted(
                launcher_environment
            ),
            "outer_policy_binding": outer_policy_binding,
            "gate_evidence_retention": {
                key: value
                for key, value in evidence_slot.items()
                if key not in {"log", "result"}
            }
            | {
                "lock_wait_seconds": evidence_lock_wait_seconds,
                "usage_after_gate": evidence_postflight,
            },
            "source_key": gate_result["source_key"],
            "artifact_key": gate_result["artifact_key"],
            "source_keyed_binary_path": artifact_export[
                "target_artifact_path"
            ],
            "binary_path": BINARY_PATH.as_posix(),
            "binary_bytes": binary_size,
            "binary_sha256": binary_sha256,
            "provenance_destination_contract": output_contract,
            "duration_ms": round((time.monotonic() - started) * 1000),
        }
        if provenance_destination_contract(root, args.output) != output_contract:
            raise ValueError(
                "provenance output contract changed during the build"
            )
        output, artifact_sha256 = write_provenance(root, args.output, payload)
        publication_deadline = (
            time.monotonic() + SOURCE_PREFLIGHT_TIMEOUT_SECONDS
        )
        if (
            build_source_manifest(
                root, deadline=publication_deadline
            )
            != before
            or git_head(root) != base_revision
            or validate_source_inputs_against_head(
                root,
                base_revision,
                deadline=publication_deadline,
            )
        ):
            raise ValueError(
                "source inputs changed during provenance publication"
            )
        payload["provenance_path"] = output.relative_to(root).as_posix()
        payload["provenance_sha256"] = artifact_sha256
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0
    except subprocess.TimeoutExpired as exc:
        print(json.dumps({"schema": "codefire.rust_build_provenance.v1", "ok": False, "error": f"timeout after {exc.timeout}s"}, indent=2, sort_keys=True))
        return 1
    except (
        FileNotFoundError,
        KeyError,
        OSError,
        UnicodeDecodeError,
        ValueError,
    ) as exc:
        print(json.dumps({"schema": "codefire.rust_build_provenance.v1", "ok": False, "error": str(exc)}, indent=2, sort_keys=True))
        return 1
    finally:
        if evidence_lock_descriptor is not None:
            release_gate_evidence_lock(evidence_lock_descriptor)


if __name__ == "__main__":
    sys.exit(main())
