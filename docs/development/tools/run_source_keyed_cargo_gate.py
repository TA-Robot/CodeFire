#!/usr/bin/env python3
"""Run one Cargo gate in an exclusively locked, source-keyed target namespace."""

from __future__ import annotations

import argparse
import errno
import fcntl
import functools
import hashlib
import importlib.util
import json
import math
import os
import re
import secrets
import select
import shutil
import signal
import stat
import subprocess
import sys
import tarfile
import tempfile
import time
import types
from contextlib import contextmanager
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable, Iterator, Mapping

POLICY_PYCACHE_PREFIX = "/dev/null"
SOURCE_GATE_BOOTSTRAP_FD_ENV = "CODEFIRE_SOURCE_GATE_BOOTSTRAP_FD"
SOURCE_GATE_BOOTSTRAP_PYTHON_FD_ENV = (
    "CODEFIRE_SOURCE_GATE_BOOTSTRAP_PYTHON_FD"
)
SOURCE_GATE_BOOTSTRAP_PATH_ENV = "CODEFIRE_SOURCE_GATE_BOOTSTRAP_PATH"
SOURCE_GATE_BOOTSTRAP_MAX_BYTES = 1024 * 1024
SOURCE_GATE_BOOTSTRAP_MAX_PYTHON_BYTES = 256 * 1024**2


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
        raise RuntimeError("source-gate bootstrap file identity is unsafe")
    digest = hashlib.sha256()
    offset = 0
    while offset < metadata.st_size:
        chunk = os.pread(
            descriptor,
            min(1024 * 1024, metadata.st_size - offset),
            offset,
        )
        if not chunk:
            raise RuntimeError(
                "source-gate bootstrap file read was incomplete"
            )
        digest.update(chunk)
        offset += len(chunk)
    after = os.fstat(descriptor)
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
    if current != expected:
        raise RuntimeError(
            "source-gate bootstrap file changed while hashing"
        )
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


def _initial_supervisor_descriptor() -> tuple[int, Path]:
    argv_zero = sys.argv[0]
    prefix = "/proc/self/fd/"
    if argv_zero.startswith(prefix):
        raw_descriptor = argv_zero[len(prefix) :]
        if raw_descriptor.isdigit():
            descriptor = os.dup(int(raw_descriptor))
            target = os.readlink(f"/proc/self/fd/{descriptor}")
            if target.endswith(" (deleted)"):
                target = target[: -len(" (deleted)")]
            canonical = Path(target).absolute()
            return descriptor, canonical
    canonical = Path(__file__).absolute()
    if canonical.resolve(strict=True) != canonical:
        raise RuntimeError(
            "source-gate bootstrap path must not contain symlinks"
        )
    descriptor = os.open(
        canonical,
        os.O_RDONLY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    return descriptor, canonical


def _source_gate_bootstrap() -> dict[str, Any]:
    if not _canonical_python_startup():
        raise RuntimeError(
            "source gate requires exact Python flags: "
            "-I -S -B -X pycache_prefix=/dev/null"
        )
    raw_supervisor_fd = os.environ.get(SOURCE_GATE_BOOTSTRAP_FD_ENV)
    raw_python_fd = os.environ.get(
        SOURCE_GATE_BOOTSTRAP_PYTHON_FD_ENV
    )
    if raw_supervisor_fd is None and raw_python_fd is None:
        supervisor_fd, canonical_path = _initial_supervisor_descriptor()
        python_fd = os.open("/proc/self/exe", os.O_RDONLY)
        try:
            supervisor_identity = _bootstrap_file_identity(
                supervisor_fd,
                maximum_bytes=SOURCE_GATE_BOOTSTRAP_MAX_BYTES,
                owner_controlled=True,
            )
            python_identity = _bootstrap_file_identity(
                python_fd,
                maximum_bytes=SOURCE_GATE_BOOTSTRAP_MAX_PYTHON_BYTES,
                owner_controlled=False,
            )
            running_python = os.stat("/proc/self/exe")
            if (running_python.st_dev, running_python.st_ino) != (
                python_identity["device"],
                python_identity["inode"],
            ):
                raise RuntimeError(
                    "source-gate running Python identity changed"
                )
            os.set_inheritable(supervisor_fd, True)
            os.set_inheritable(python_fd, True)
            environment = dict(os.environ)
            environment[SOURCE_GATE_BOOTSTRAP_FD_ENV] = str(
                supervisor_fd
            )
            environment[SOURCE_GATE_BOOTSTRAP_PYTHON_FD_ENV] = str(
                python_fd
            )
            environment[SOURCE_GATE_BOOTSTRAP_PATH_ENV] = str(
                canonical_path
            )
            os.execve(
                python_fd,
                [
                    f"/proc/self/fd/{python_fd}",
                    "-I",
                    "-S",
                    "-B",
                    "-X",
                    f"pycache_prefix={POLICY_PYCACHE_PREFIX}",
                    f"/proc/self/fd/{supervisor_fd}",
                    *sys.argv[1:],
                ],
                environment,
            )
        except BaseException:
            os.close(supervisor_fd)
            os.close(python_fd)
            raise
        raise RuntimeError("source-gate bootstrap exec returned")
    if raw_supervisor_fd is None or raw_python_fd is None:
        raise RuntimeError("source-gate bootstrap descriptors are incomplete")
    try:
        supervisor_fd = int(raw_supervisor_fd)
        python_fd = int(raw_python_fd)
    except ValueError as exc:
        raise RuntimeError(
            "source-gate bootstrap descriptor is invalid"
        ) from exc
    if sys.argv[0] != f"/proc/self/fd/{supervisor_fd}":
        raise RuntimeError(
            "source-gate bootstrap argv does not name the held supervisor"
        )
    canonical_raw = os.environ.get(SOURCE_GATE_BOOTSTRAP_PATH_ENV)
    if not canonical_raw:
        raise RuntimeError("source-gate bootstrap path is unavailable")
    canonical_path = Path(canonical_raw).absolute()
    supervisor_identity = _bootstrap_file_identity(
        supervisor_fd,
        maximum_bytes=SOURCE_GATE_BOOTSTRAP_MAX_BYTES,
        owner_controlled=True,
    )
    python_identity = _bootstrap_file_identity(
        python_fd,
        maximum_bytes=SOURCE_GATE_BOOTSTRAP_MAX_PYTHON_BYTES,
        owner_controlled=False,
    )
    running_python = os.stat("/proc/self/exe")
    if (running_python.st_dev, running_python.st_ino) != (
        python_identity["device"],
        python_identity["inode"],
    ):
        raise RuntimeError(
            "source-gate bootstrap Python differs from /proc/self/exe"
        )
    return {
        "supervisor_descriptor": supervisor_fd,
        "supervisor_identity": supervisor_identity,
        "python_descriptor": python_fd,
        "python_identity": python_identity,
        "canonical_path": canonical_path,
    }


_RUNNING_SOURCE_GATE = (
    _source_gate_bootstrap() if __name__ == "__main__" else None
)
sys.dont_write_bytecode = True
sys.pycache_prefix = POLICY_PYCACHE_PREFIX

try:
    import tomllib
except ModuleNotFoundError:  # Python 3.10 compatibility
    tomllib = None  # type: ignore[assignment]

TOOLS_DIRECTORY = (
    Path(_RUNNING_SOURCE_GATE["canonical_path"]).parent
    if _RUNNING_SOURCE_GATE is not None
    else Path(__file__).resolve().parent
)
if str(TOOLS_DIRECTORY) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIRECTORY))

_PRELOADED_POLICY_MODULES: dict[str, dict[str, Any]] = {}


def _preload_policy_module(name: str, path: Path) -> None:
    lexical = path.absolute()
    if lexical.resolve(strict=True) != lexical:
        raise RuntimeError(f"policy module path is unsafe: {lexical}")
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
            raise RuntimeError(
                f"policy module identity is unsafe: {lexical}"
            )
        raw = os.pread(descriptor, metadata.st_size, 0)
        if len(raw) != metadata.st_size:
            raise RuntimeError(
                f"policy module read was incomplete: {lexical}"
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
            hashlib.sha256(raw).hexdigest(),
        )
        # A policy module already bound in this interpreter must be reused
        # rather than reloaded. Loading a second copy would leave two
        # divergent instances of the same policy state -- separate profile
        # dictionaries, separate lock bookkeeping -- and the caller that
        # imported first would keep enforcing the orphaned copy. Reuse is
        # admitted only when the live module was loaded from exactly this
        # pinned canonical path, whose bytes are verified above.
        existing = sys.modules.get(name)
        if existing is not None:
            origin = getattr(existing, "__file__", None)
            if origin is None or Path(origin).absolute() != lexical:
                raise RuntimeError(
                    f"policy module {name} is already bound to a foreign "
                    f"origin: {origin}"
                )
            reused = True
        else:
            spec = importlib.util.spec_from_loader(
                name, loader=None, origin=str(lexical)
            )
            if spec is None:
                raise RuntimeError(
                    f"policy module spec is unavailable: {name}"
                )
            module = importlib.util.module_from_spec(spec)
            module.__file__ = str(lexical)
            module.__cached__ = None
            sys.modules[name] = module
            try:
                code = compile(
                    raw,
                    str(lexical),
                    "exec",
                    dont_inherit=True,
                )
                exec(code, module.__dict__)
            except BaseException:
                sys.modules.pop(name, None)
                raise
            reused = False
        _PRELOADED_POLICY_MODULES[name] = {
            "descriptor": descriptor,
            "path": lexical,
            "identity": identity,
            "reused_existing_module": reused,
        }
    except BaseException:
        os.close(descriptor)
        raise


for _policy_module_name in (
    "ai_contracts",
    "build_source_bound_rust",
    "run_timed_gate",
    "check_rust_execution_profile",
):
    _preload_policy_module(
        _policy_module_name,
        TOOLS_DIRECTORY / f"{_policy_module_name}.py",
    )

from ai_contracts import canonical_json_bytes, sha256_json
from build_source_bound_rust import (
    GIT_CONTROL_CONFIG_ARGS,
    GATE_EVIDENCE_LOCK_NAME,
    GATE_EVIDENCE_RESULT_PUBLICATION_ENTRY_RESERVE,
    GATE_EVIDENCE_RESULT_PUBLICATION_RESERVE_BYTES,
    MAX_GATE_RESULT_BYTES,
    acquire_gate_evidence_lock,
    bounded_json_bytes,
    build_source_manifest,
    gate_evidence_admission,
    git_control_environment,
    release_gate_evidence_lock,
    validate_gate_evidence_lock_descriptor,
)
from check_rust_execution_profile import (
    DEFAULT_PROFILE,
    SOURCE_KEY_TARGET_DIRECTORY,
    evaluate_process,
)
from run_timed_gate import (
    open_exclusive_output,
    run_gate,
    verify_output_identity,
)


SOURCE_KEY_SCHEMA = "codefire.cargo_source_key.v1"
ARTIFACT_KEY_SCHEMA = "codefire.cargo_artifact_key.v1"
MARKER_SCHEMA = "codefire.cargo_target_namespace.v1"
RESULT_SCHEMA = "codefire.source_keyed_cargo_gate.v1"
LOG_PREFIX_SCHEMA = "codefire.source_keyed_cargo_log_prefix.v1"
MARKER_NAME = ".codefire-source-key.json"
NAMESPACE_LOCK_NAME = ".codefire-source-key.lock"
REGISTRY_LOCK_NAME = ".codefire-source-key-registry.lock"
GLOBAL_EXECUTION_LOCK_NAME = ".codefire-source-key-execution.lock"
RETENTION_NAME = ".codefire-source-key-retention.json"
INVALIDATION_NAME = ".codefire-source-key-invalid.json"
RUN_STATE_NAME = ".codefire-source-key-run-state.json"
READONLY_HELPER = Path(__file__).resolve().with_name("readonly_source_exec.py")
PARENT_DEATH_HELPER = Path(__file__).resolve().with_name("pdeath_exec.py")
SUPERVISOR_POLICY_FILES = {
    "supervisor": TOOLS_DIRECTORY / "run_source_keyed_cargo_gate.py",
    "ai-contracts": Path(__file__).resolve().with_name(
        "ai_contracts.py"
    ),
    "build-source-bound": Path(__file__).resolve().with_name(
        "build_source_bound_rust.py"
    ),
    "execution-profile": Path(__file__).resolve().with_name(
        "check_rust_execution_profile.py"
    ),
    "timed-gate": Path(__file__).resolve().with_name(
        "run_timed_gate.py"
    ),
}
PRELOADED_POLICY_MODULE_NAMES = {
    "ai-contracts": "ai_contracts",
    "build-source-bound": "build_source_bound_rust",
    "execution-profile": "check_rust_execution_profile",
    "timed-gate": "run_timed_gate",
}
GIT_EXECUTABLE = Path("/usr/bin/git")
# One source of truth for the namespace launch. The recorded isolation
# identity is embedded verbatim in the source key, the artifact key, and
# every published result, so a reviewer auditing "was propagation pinned?"
# must read the flags that were actually launched. Duplicating the list
# once let the recorded copy drift behind the launched one; both now read
# this constant and the launch test asserts they agree.
#
# --propagation private: the private source tmpfs covers the candidate
# worktree's own path, so a shared namespace would mask the developer's
# real worktree on the host. util-linux defaults to private, but a default
# is not evidence.
UNSHARE_ISOLATION_FLAGS = (
    "-Urnmpf",
    "--mount-proc",
    "--kill-child=SIGKILL",
    "--propagation",
    "private",
)
MEASURED_MAX_NAMESPACE_ALLOCATED_BYTES = 3_204_700_000
MEASURED_REFERENCE_CANDIDATE_COUNT = 8
MEASURED_HEADROOM_NUMERATOR = 5
MEASURED_HEADROOM_DENOMINATOR = 4
DEFAULT_NEW_NAMESPACE_ALLOCATION_RESERVE_BYTES = (
    MEASURED_MAX_NAMESPACE_ALLOCATED_BYTES * MEASURED_HEADROOM_NUMERATOR
    + MEASURED_HEADROOM_DENOMINATOR
    - 1
) // MEASURED_HEADROOM_DENOMINATOR
DEFAULT_PERSISTENT_AGGREGATE_BUDGET_BYTES = 32 * 1024**3
DEFAULT_NEW_NAMESPACE_FREE_FLOOR_BYTES = 100 * 1024**3
MAX_NAMESPACE_ENTRY_COUNT = 1_000_000
MAX_ROOT_DIRECTORY_ENTRIES = 100_000
MAX_REGISTRY_ROOT_ENTRIES = 4_096
LIVE_NAMESPACE_SCAN_SECONDS = 0.75
REGISTRY_SCAN_SECONDS = 30.0
TREE_CLEANUP_SECONDS = 5 * 60.0
SNAPSHOT_VERIFY_SECONDS = 120.0
SOURCE_STATE_PREFLIGHT_SECONDS = 120.0
MAX_EXPORTED_ARTIFACT_LOGICAL_BYTES = 512 * 1024**2
ARTIFACT_EXPORT_TIMEOUT_SECONDS = 120.0
ARTIFACT_EXPORT_RECHECK_BYTES = 16 * 1024**2
MAX_ARTIFACT_EXPORT_RESIDUE_ENTRIES = 16
# Cargo uplifts a produced binary by hard-linking
# ``<profile>/deps/<crate>-<metadata>`` to ``<profile>/<name>``, so the
# requested export source normally carries exactly two links. Requiring a
# single link blocks that normal release export while proving nothing:
# what the copy rests on is the no-follow descriptor, the pre/post inode
# identity comparison, and the full-content re-read below, none of which
# depends on the link count. Two is the bound the uplift needs; a third
# link is an alias this gate never creates, so it stays a `BLOCK`. The
# exported destination is unrelated and remains strictly single-link.
MAX_EXPORT_SOURCE_ARTIFACT_LINKS = 2
ALLOWED_SUBCOMMANDS = frozenset({"build", "check", "clippy", "doc", "fmt", "test"})
LOCKED_SUBCOMMANDS = frozenset({"build", "check", "clippy", "doc", "test"})
TEST_RESULT_RE = re.compile(
    r"^test result: (ok|FAILED)\. "
    r"(\d+) passed; (\d+) failed; (\d+) ignored; "
    r"(\d+) measured; (\d+) filtered out(?:;.*)?$"
)
TEST_HARNESS_RE = re.compile(r"^\s*Running .+\(([^()]+)\)\s*$")
TEST_RUNNING_RE = re.compile(r"^running (\d+) tests?$")
ADMITTED_AMBIENT_ENVIRONMENT_NAMES = frozenset(
    {
        "AR",
        "CC",
        "CFLAGS",
        "CARGO_HTTP_CHECK_REVOKE",
        "CARGO_NET_OFFLINE",
        "CXX",
        "CXXFLAGS",
        "HOST",
        "LANG",
        "LC_ALL",
        "LC_CTYPE",
        "PATH",
        "PKG_CONFIG",
        "PKG_CONFIG_LIBDIR",
        "PKG_CONFIG_PATH",
        "RANLIB",
        "RUST_BACKTRACE",
        "RUSTFLAGS",
        "RUSTDOCFLAGS",
        "TARGET",
        "TZ",
    }
)
RUNTIME_SCRATCH_NAME = ".codefire-runtime-scratch"
RUNTIME_HOME_NAME = "home"
RUNTIME_TMP_NAME = "tmp"
ADMITTED_AMBIENT_ENVIRONMENT_PREFIXES = (
    "AR_",
    "CC_",
    "CFLAGS_",
    "CXX_",
    "CXXFLAGS_",
    "PKG_CONFIG_",
    "RANLIB_",
)
DANGEROUS_TOOL_OVERRIDE_NAMES = frozenset(
    {
        "CARGO",
        "CARGO_BUILD_RUSTC",
        "CARGO_BUILD_RUSTC_WRAPPER",
        "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
        "CLIPPY_DRIVER_PATH",
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTDOC",
        "RUSTFMT",
    }
)
DANGEROUS_RUNTIME_OVERRIDE_NAMES = frozenset(
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
SCRUBBED_ENVIRONMENT_PATTERNS = (
    re.compile(r"(?:^|_)(?:API_?KEY|PASSWORD|SECRET|TOKEN)$", re.IGNORECASE),
    re.compile(r"^(?:SSH_AUTH_SOCK|GPG_AGENT_INFO)$"),
)
RUNTIME_TOOL_ENVIRONMENT_TO_EXECUTABLE = {
    "CARGO": "cargo",
    "CLIPPY_DRIVER_PATH": "clippy-driver",
    "RUSTC": "rustc",
    "RUSTC_WORKSPACE_WRAPPER": "clippy-driver",
    "RUSTDOC": "rustdoc",
    "RUSTFMT": "rustfmt",
}
MAX_TEST_LOG_LINE_BYTES = 64 * 1024
MAX_TEST_HARNESS_SUMMARIES = 1024
MAX_RETAINED_TEST_HARNESSES = 128
MAX_TEST_COUNT_PER_HARNESS = 10_000_000
SNAPSHOT_MARKER_NAME = ".codefire-source-snapshot.json"
SNAPSHOT_MARKER_FIELDS = (
    "schema",
    "source_key",
    "candidate_root",
    "head_revision",
    "tree",
    "git_tree_entry_count",
    "git_tree_entries_sha256",
    "snapshot_logical_file_bytes",
    "snapshot_content_manifest_sha256",
    "snapshot_file_manifest_sha256",
    "rust_source_manifest_sha256",
    "snapshot_path",
)
DEFAULT_MAX_SNAPSHOT_LOGICAL_BYTES = 2 * 1024**3
DEFAULT_SNAPSHOT_ROOT_BUDGET_BYTES = 8 * 1024**3
DEFAULT_MAX_SNAPSHOT_FILES = 200_000
MAX_PRIVATE_SOURCE_TMPFS_BYTES = 4 * 1024**3
GIT_IDENTITY_TIMEOUT_SECONDS = 120.0
GIT_ARCHIVE_TIMEOUT_SECONDS = 15 * 60.0
MAX_GIT_STDERR_BYTES = 64 * 1024
MAX_TOOL_VERSION_OUTPUT_BYTES = 64 * 1024
TOOL_VERSION_TIMEOUT_SECONDS = 15.0
MAX_PINNED_TOOL_BYTES = 256 * 1024**2
TOOL_IDENTITY_TOTAL_SECONDS = 60.0


class GateBlocked(ValueError):
    """A fail-closed admission or provenance decision."""

    def __init__(self, code: str, detail: str):
        super().__init__(detail)
        self.code = code
        self.detail = detail


def terminate_git_process(process: subprocess.Popen[bytes]) -> None:
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


def consume_process_output(
    process: subprocess.Popen[bytes],
    *,
    label: str,
    timeout_seconds: float,
    stdout_consumer: Callable[[bytes], None],
) -> tuple[int, bytes]:
    if (
        not math.isfinite(timeout_seconds)
        or timeout_seconds <= 0
        or process.stdout is None
        or process.stderr is None
    ):
        terminate_git_process(process)
        raise GateBlocked(
            "git-operation-invalid",
            f"{label}: timeout={timeout_seconds}",
        )
    streams = {
        process.stdout.fileno(): "stdout",
        process.stderr.fileno(): "stderr",
    }
    for descriptor in streams:
        os.set_blocking(descriptor, False)
    stderr = bytearray()
    deadline = time.monotonic() + timeout_seconds
    try:
        while streams:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise GateBlocked(
                    "git-operation-timeout",
                    f"{label} exceeded {timeout_seconds}s",
                )
            try:
                readable, _, _ = select.select(
                    list(streams), [], [], remaining
                )
            except InterruptedError:
                continue
            if not readable:
                raise GateBlocked(
                    "git-operation-timeout",
                    f"{label} exceeded {timeout_seconds}s",
                )
            for descriptor in readable:
                try:
                    chunk = os.read(descriptor, 1024 * 1024)
                except BlockingIOError:
                    continue
                if not chunk:
                    streams.pop(descriptor, None)
                    continue
                if streams[descriptor] == "stdout":
                    stdout_consumer(chunk)
                else:
                    stderr.extend(chunk)
                    if len(stderr) > MAX_GIT_STDERR_BYTES:
                        raise GateBlocked(
                            "git-stderr-output-exceeded",
                            f"{label} exceeded "
                            f"{MAX_GIT_STDERR_BYTES} bytes",
                        )
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise GateBlocked(
                "git-operation-timeout",
                f"{label} exceeded {timeout_seconds}s",
            )
        returncode = process.wait(timeout=remaining)
        return returncode, bytes(stderr)
    except subprocess.TimeoutExpired as exc:
        raise GateBlocked(
            "git-operation-timeout",
            f"{label} exceeded {timeout_seconds}s",
        ) from exc
    except BaseException:
        terminate_git_process(process)
        raise
    finally:
        process.stdout.close()
        process.stderr.close()


def git_bytes(
    root: Path,
    *args: str,
    maximum_bytes: int = 256 * 1024 * 1024,
    timeout_seconds: float = GIT_IDENTITY_TIMEOUT_SECONDS,
) -> bytes:
    process = subprocess.Popen(
        [
            str(GIT_EXECUTABLE),
            *GIT_CONTROL_CONFIG_ARGS,
            *args,
        ],
        cwd=root,
        env=git_control_environment(),
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    output = bytearray()

    def collect(chunk: bytes) -> None:
        output.extend(chunk)
        if len(output) > maximum_bytes:
            raise GateBlocked(
                "git-identity-output-exceeded",
                f"git {' '.join(args)} exceeded {maximum_bytes} bytes",
            )

    returncode, stderr = consume_process_output(
        process,
        label=f"git {' '.join(args)}",
        timeout_seconds=timeout_seconds,
        stdout_consumer=collect,
    )
    if returncode != 0:
        message = stderr.decode("utf-8", errors="replace").strip()
        raise GateBlocked(
            "git-identity-failed",
            message or f"git {' '.join(args)} failed",
        )
    return bytes(output)


def git_status_bytes(
    root: Path,
    *args: str,
    maximum_bytes: int = 1024 * 1024,
    timeout_seconds: float = GIT_IDENTITY_TIMEOUT_SECONDS,
) -> tuple[int, bytes, bytes]:
    process = subprocess.Popen(
        [
            str(GIT_EXECUTABLE),
            *GIT_CONTROL_CONFIG_ARGS,
            *args,
        ],
        cwd=root,
        env=git_control_environment(),
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    output = bytearray()

    def collect(chunk: bytes) -> None:
        output.extend(chunk)
        if len(output) > maximum_bytes:
            raise GateBlocked(
                "git-identity-output-exceeded",
                f"git {' '.join(args)} exceeded {maximum_bytes} bytes",
            )

    returncode, stderr = consume_process_output(
        process,
        label=f"git {' '.join(args)}",
        timeout_seconds=timeout_seconds,
        stdout_consumer=collect,
    )
    return returncode, bytes(output), stderr


def repository_root(root: Path) -> Path:
    candidate = root.resolve(strict=True)
    if not candidate.is_dir():
        raise GateBlocked("candidate-not-directory", str(candidate))
    top = Path(
        git_bytes(candidate, "rev-parse", "--show-toplevel")
        .decode("utf-8", errors="strict")
        .strip()
    ).resolve(strict=True)
    if top != candidate:
        raise GateBlocked(
            "candidate-not-worktree-root",
            f"expected {top}, got {candidate}",
        )
    return candidate


def tagged_digest(parts: list[tuple[str, bytes]]) -> str:
    digest = hashlib.sha256()
    for label, payload in parts:
        label_bytes = label.encode("ascii")
        digest.update(len(label_bytes).to_bytes(4, "big"))
        digest.update(label_bytes)
        digest.update(len(payload).to_bytes(8, "big"))
        digest.update(payload)
    return digest.hexdigest()


def remaining_source_preflight_seconds(deadline: float) -> float:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise GateBlocked(
            "source-preflight-deadline-exceeded",
            str(SOURCE_STATE_PREFLIGHT_SECONDS),
        )
    return min(GIT_IDENTITY_TIMEOUT_SECONDS, remaining)


def git_output_identity(
    root: Path,
    *args: str,
    deadline: float | None = None,
) -> dict[str, Any]:
    process = subprocess.Popen(
        [
            str(GIT_EXECUTABLE),
            *GIT_CONTROL_CONFIG_ARGS,
            *args,
        ],
        cwd=root,
        env=git_control_environment(),
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    digest = hashlib.sha256()
    size = 0

    def consume(chunk: bytes) -> None:
        nonlocal size
        digest.update(chunk)
        size += len(chunk)

    returncode, _stderr = consume_process_output(
        process,
        label=f"git {' '.join(args)}",
        timeout_seconds=(
            remaining_source_preflight_seconds(deadline)
            if deadline is not None
            else GIT_IDENTITY_TIMEOUT_SECONDS
        ),
        stdout_consumer=consume,
    )
    if returncode != 0:
        raise GateBlocked(
            "git-identity-failed",
            f"git {' '.join(args)} failed with exit {returncode}",
        )
    return {"bytes": size, "sha256": digest.hexdigest()}


def file_content_identity(
    path: Path,
    *,
    deadline: float,
    maximum_bytes: int,
) -> tuple[int, str]:
    descriptor = os.open(
        path,
        os.O_RDONLY | os.O_CLOEXEC | getattr(os, "O_NOFOLLOW", 0),
    )
    try:
        before = os.fstat(descriptor)
        if not stat.S_ISREG(before.st_mode):
            raise GateBlocked(
                "unsafe-untracked-path", f"not a regular file: {path}"
            )
        if before.st_size > maximum_bytes:
            raise GateBlocked(
                "untracked-logical-bytes-exceeded",
                f"bytes={before.st_size}, remaining={maximum_bytes}",
            )
        digest = hashlib.sha256()
        size = 0
        while True:
            if time.monotonic() > deadline:
                raise GateBlocked(
                    "source-preflight-deadline-exceeded",
                    str(SOURCE_STATE_PREFLIGHT_SECONDS),
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
            raise GateBlocked(
                "untracked-file-changed-during-capture", str(path)
            )
        return size, digest.hexdigest()
    finally:
        os.close(descriptor)


def untracked_evidence(
    root: Path,
    *,
    deadline: float,
) -> list[dict[str, Any]]:
    raw = git_bytes(
        root,
        "ls-files",
        "-z",
        "--others",
        "--exclude-standard",
        "--",
        maximum_bytes=64 * 1024 * 1024,
        timeout_seconds=remaining_source_preflight_seconds(deadline),
    )
    evidence: list[dict[str, Any]] = []
    logical_bytes = 0
    paths = sorted(item for item in raw.split(b"\0") if item)
    if len(paths) > DEFAULT_MAX_SNAPSHOT_FILES:
        raise GateBlocked(
            "untracked-file-count-exceeded",
            f"files={len(paths)}, max={DEFAULT_MAX_SNAPSHOT_FILES}",
        )
    for path_bytes in paths:
        if time.monotonic() > deadline:
            raise GateBlocked(
                "source-preflight-deadline-exceeded",
                str(SOURCE_STATE_PREFLIGHT_SECONDS),
            )
        relative = path_bytes.decode("utf-8", errors="surrogateescape")
        lexical = root / relative
        try:
            lexical.parent.resolve(strict=True).relative_to(root)
        except (FileNotFoundError, ValueError) as exc:
            raise GateBlocked(
                "unsafe-untracked-path",
                f"untracked path escapes the candidate: {relative}",
            ) from exc
        metadata = lexical.lstat()
        if stat.S_ISREG(metadata.st_mode):
            kind = "file"
            if (
                logical_bytes + metadata.st_size
                > DEFAULT_MAX_SNAPSHOT_LOGICAL_BYTES
            ):
                raise GateBlocked(
                    "untracked-logical-bytes-exceeded",
                    f"maximum="
                    f"{DEFAULT_MAX_SNAPSHOT_LOGICAL_BYTES}",
                )
            payload_size, payload_sha256 = file_content_identity(
                lexical,
                deadline=deadline,
                maximum_bytes=(
                    DEFAULT_MAX_SNAPSHOT_LOGICAL_BYTES
                    - logical_bytes
                ),
            )
        elif stat.S_ISLNK(metadata.st_mode):
            kind = "symlink"
            payload = os.readlink(lexical).encode(
                "utf-8", errors="surrogateescape"
            )
            payload_size = len(payload)
            payload_sha256 = hashlib.sha256(payload).hexdigest()
        else:
            raise GateBlocked(
                "unsupported-untracked-input",
                f"untracked input is neither a file nor symlink: {relative}",
            )
        logical_bytes += payload_size
        if logical_bytes > DEFAULT_MAX_SNAPSHOT_LOGICAL_BYTES:
            raise GateBlocked(
                "untracked-logical-bytes-exceeded",
                f"maximum={DEFAULT_MAX_SNAPSHOT_LOGICAL_BYTES}",
            )
        evidence.append(
            {
                "path": relative,
                "kind": kind,
                "mode": stat.S_IMODE(metadata.st_mode),
                "bytes": payload_size,
                "sha256": payload_sha256,
            }
        )
    return evidence


def dirty_identity(
    root: Path,
    *,
    deadline: float | None = None,
) -> dict[str, Any]:
    if deadline is None:
        deadline = (
            time.monotonic() + SOURCE_STATE_PREFLIGHT_SECONDS
        )
    status_identity = git_output_identity(
        root,
        "-c",
        "core.quotepath=false",
        "status",
        "--porcelain=v2",
        "-z",
        "--untracked-files=all",
        deadline=deadline,
    )
    worktree_patch = git_output_identity(
        root,
        "diff",
        "--binary",
        "--full-index",
        "--no-ext-diff",
        "--no-textconv",
        "HEAD",
        "--",
        deadline=deadline,
    )
    index_patch = git_output_identity(
        root,
        "diff",
        "--cached",
        "--binary",
        "--full-index",
        "--no-ext-diff",
        "--no-textconv",
        "HEAD",
        "--",
        deadline=deadline,
    )
    untracked = untracked_evidence(root, deadline=deadline)
    untracked_bytes = canonical_json_bytes(untracked)
    return {
        "dirty": status_identity["bytes"] > 0,
        "dirty_digest": tagged_digest(
            [
                (
                    "porcelain-v2-z",
                    canonical_json_bytes(status_identity),
                ),
                (
                    "worktree-vs-head-binary-patch",
                    canonical_json_bytes(worktree_patch),
                ),
                (
                    "index-vs-head-binary-patch",
                    canonical_json_bytes(index_patch),
                ),
                ("untracked-content-manifest", untracked_bytes),
            ]
        ),
        "porcelain_sha256": status_identity["sha256"],
        "worktree_patch_sha256": worktree_patch["sha256"],
        "index_patch_sha256": index_patch["sha256"],
        "untracked_manifest_sha256": hashlib.sha256(untracked_bytes).hexdigest(),
        "untracked_file_count": len(untracked),
    }


def clean_dirty_identity(
    root: Path,
    *,
    deadline: float,
) -> dict[str, Any]:
    returncode, status, stderr = git_status_bytes(
        root,
        "-c",
        "core.quotepath=false",
        "status",
        "--porcelain=v2",
        "-z",
        "--untracked-files=all",
        maximum_bytes=1024 * 1024,
        timeout_seconds=remaining_source_preflight_seconds(deadline),
    )
    if returncode != 0:
        message = stderr.decode(
            "utf-8", errors="replace"
        ).strip()
        raise GateBlocked(
            "git-identity-failed",
            message or "git status failed",
        )
    if status:
        raise GateBlocked(
            "dirty-candidate-not-trusted",
            "trusted Cargo evidence requires a clean candidate; "
            f"porcelain_bytes={len(status)}, "
            f"porcelain_sha256={hashlib.sha256(status).hexdigest()}",
        )
    empty_identity = {
        "bytes": 0,
        "sha256": hashlib.sha256(b"").hexdigest(),
    }
    untracked_bytes = canonical_json_bytes([])
    return {
        "dirty": False,
        "dirty_digest": tagged_digest(
            [
                (
                    "porcelain-v2-z",
                    canonical_json_bytes(empty_identity),
                ),
                (
                    "worktree-vs-head-binary-patch",
                    canonical_json_bytes(empty_identity),
                ),
                (
                    "index-vs-head-binary-patch",
                    canonical_json_bytes(empty_identity),
                ),
                ("untracked-content-manifest", untracked_bytes),
            ]
        ),
        "porcelain_sha256": empty_identity["sha256"],
        "worktree_patch_sha256": empty_identity["sha256"],
        "index_patch_sha256": empty_identity["sha256"],
        "untracked_manifest_sha256": hashlib.sha256(
            untracked_bytes
        ).hexdigest(),
        "untracked_file_count": 0,
    }


def capture_source_state(
    root: Path,
    *,
    require_clean: bool = False,
    deadline: float | None = None,
) -> dict[str, Any]:
    if deadline is None:
        deadline = (
            time.monotonic() + SOURCE_STATE_PREFLIGHT_SECONDS
        )
    root = repository_root(root)
    dirty = (
        clean_dirty_identity(root, deadline=deadline)
        if require_clean
        else dirty_identity(root, deadline=deadline)
    )
    head = git_bytes(
        root,
        "rev-parse",
        "--verify",
        "HEAD^{commit}",
        timeout_seconds=remaining_source_preflight_seconds(deadline),
    ).decode().strip()
    tree = git_bytes(
        root,
        "rev-parse",
        f"{head}^{{tree}}",
        timeout_seconds=remaining_source_preflight_seconds(deadline),
    ).decode().strip()
    common_raw = git_bytes(
        root,
        "rev-parse",
        "--git-common-dir",
        timeout_seconds=remaining_source_preflight_seconds(deadline),
    ).decode().strip()
    common = Path(common_raw)
    if not common.is_absolute():
        common = root / common
    common = common.resolve(strict=True)
    git_dir_raw = git_bytes(
        root,
        "rev-parse",
        "--git-dir",
        timeout_seconds=remaining_source_preflight_seconds(deadline),
    ).decode().strip()
    git_dir = Path(git_dir_raw)
    if not git_dir.is_absolute():
        git_dir = root / git_dir
    git_dir = git_dir.resolve(strict=True)
    rust_manifest = build_source_manifest(root, deadline=deadline)
    return {
        "candidate_root": str(root),
        "git_common_dir": str(common),
        "git_dir": str(git_dir),
        "head_revision": head,
        "tree": tree,
        **dirty,
        "rust_source_manifest_sha256": rust_manifest["manifest_sha256"],
        "rust_source_file_count": len(rust_manifest["files"]),
    }


def git_tree_entries(root: Path, head: str) -> list[dict[str, Any]]:
    raw = git_bytes(
        root,
        "ls-tree",
        "-r",
        "-z",
        "-l",
        "--full-tree",
        head,
    )
    entries: list[dict[str, Any]] = []
    for record in (item for item in raw.split(b"\0") if item):
        try:
            metadata, path_raw = record.split(b"\t", 1)
            mode, object_type, oid, size_raw = metadata.split()
            relative = path_raw.decode("utf-8", errors="strict")
            relative_path = Path(relative)
            size = int(size_raw)
        except (ValueError, UnicodeDecodeError) as exc:
            raise GateBlocked(
                "git-tree-entry-invalid",
                hashlib.sha256(record).hexdigest(),
            ) from exc
        if (
            object_type != b"blob"
            or mode not in {b"100644", b"100755"}
            or relative_path.is_absolute()
            or not relative_path.parts
            or any(part in {"", ".", ".."} for part in relative_path.parts)
            or relative_path.as_posix() == SNAPSHOT_MARKER_NAME
            or re.fullmatch(rb"[0-9a-f]{40}|[0-9a-f]{64}", oid)
            is None
            or size < 0
        ):
            raise GateBlocked(
                "unsupported-git-tree-entry",
                f"path={relative!r}, mode={mode.decode(errors='replace')}, "
                f"type={object_type.decode(errors='replace')}",
            )
        entries.append(
            {
                "path": relative_path.as_posix(),
                "mode": mode.decode("ascii"),
                "oid": oid.decode("ascii"),
                "bytes": size,
            }
        )
    paths = [item["path"] for item in entries]
    if len(paths) != len(set(paths)):
        raise GateBlocked("duplicate-git-tree-path", head)
    if not entries:
        raise GateBlocked("empty-git-tree", head)
    return entries


def validate_private_root(path: Path, *, code: str) -> Path:
    lexical = path.absolute()
    try:
        resolved = lexical.resolve(strict=True)
    except FileNotFoundError as exc:
        raise GateBlocked(code, f"missing: {lexical}") from exc
    metadata = resolved.stat()
    if (
        resolved != lexical
        or not stat.S_ISDIR(metadata.st_mode)
        or metadata.st_uid != os.geteuid()
        or metadata.st_mode & 0o022
    ):
        raise GateBlocked(
            code, f"must be a real owner-controlled directory: {lexical}"
        )
    return resolved


def ensure_snapshot_directory_at(root: int, parts: tuple[str, ...]) -> int:
    current = os.dup(root)
    try:
        for part in parts:
            try:
                os.mkdir(part, mode=0o700, dir_fd=current)
                os.fsync(current)
            except FileExistsError:
                pass
            child = os.open(
                part,
                os.O_RDONLY
                | os.O_DIRECTORY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
                dir_fd=current,
            )
            metadata = os.fstat(child)
            if (
                not stat.S_ISDIR(metadata.st_mode)
                or metadata.st_uid != os.geteuid()
                or metadata.st_mode & 0o022
            ):
                os.close(child)
                raise GateBlocked(
                    "unsafe-snapshot-directory", "/".join(parts)
                )
            os.close(current)
            current = child
        return current
    except BaseException:
        os.close(current)
        raise


def fsync_tree_and_make_read_only(root: Path) -> None:
    root_descriptor = open_directory_no_follow(root)
    root_identity = os.fstat(root_descriptor)
    events: list[tuple[bool, str, int, int]] = [
        (
            False,
            "",
            root_identity.st_dev,
            root_identity.st_ino,
        )
    ]
    entries = 0
    deadline = time.monotonic() + SNAPSHOT_VERIFY_SECONDS
    try:
        while events:
            if time.monotonic() > deadline:
                raise GateBlocked(
                    "snapshot-finalization-deadline-exceeded",
                    str(SNAPSHOT_VERIFY_SECONDS),
                )
            (
                leaving,
                relative,
                expected_device,
                expected_inode,
            ) = events.pop()
            current = reopen_directory_at(
                root_descriptor,
                relative,
                code="snapshot-entry-identity-mismatch",
            )
            try:
                opened = os.fstat(current)
                if (opened.st_dev, opened.st_ino) != (
                    expected_device,
                    expected_inode,
                ):
                    raise GateBlocked(
                        "snapshot-entry-identity-mismatch",
                        relative or ".",
                    )
                if leaving:
                    os.fchmod(current, 0o555)
                    os.fsync(current)
                    continue
                child_directories: list[
                    tuple[str, int, int]
                ] = []
                names = bounded_directory_names_at(
                    current,
                    deadline=deadline,
                    code=(
                        "snapshot-finalization-deadline-exceeded"
                    ),
                    maximum_entries=MAX_NAMESPACE_ENTRY_COUNT,
                )
                for name in names:
                    if time.monotonic() > deadline:
                        raise GateBlocked(
                            "snapshot-finalization-deadline-exceeded",
                            str(SNAPSHOT_VERIFY_SECONDS),
                        )
                    entries += 1
                    if entries > MAX_NAMESPACE_ENTRY_COUNT:
                        raise GateBlocked(
                            "snapshot-entry-count-exceeded",
                            str(MAX_NAMESPACE_ENTRY_COUNT),
                        )
                    child_relative = (
                        f"{relative}/{name}" if relative else name
                    )
                    metadata = os.stat(
                        name,
                        dir_fd=current,
                        follow_symlinks=False,
                    )
                    if stat.S_ISDIR(metadata.st_mode):
                        child_directories.append(
                            (
                                child_relative,
                                metadata.st_dev,
                                metadata.st_ino,
                            )
                        )
                        continue
                    if stat.S_ISLNK(metadata.st_mode):
                        raise GateBlocked(
                            "snapshot-symlink-detected",
                            str(root / child_relative),
                        )
                    if (
                        not stat.S_ISREG(metadata.st_mode)
                        or metadata.st_nlink != 1
                    ):
                        raise GateBlocked(
                            "snapshot-file-identity-invalid",
                            str(root / child_relative),
                        )
                    descriptor = os.open(
                        name,
                        os.O_RDONLY
                        | os.O_CLOEXEC
                        | getattr(os, "O_NOFOLLOW", 0),
                        dir_fd=current,
                    )
                    try:
                        file_opened = os.fstat(descriptor)
                        if (
                            file_opened.st_dev,
                            file_opened.st_ino,
                            file_opened.st_size,
                        ) != (
                            metadata.st_dev,
                            metadata.st_ino,
                            metadata.st_size,
                        ):
                            raise GateBlocked(
                                "snapshot-entry-identity-mismatch",
                                child_relative,
                            )
                        os.fchmod(
                            descriptor,
                            (
                                0o555
                                if file_opened.st_mode & stat.S_IXUSR
                                else 0o444
                            ),
                        )
                        os.fsync(descriptor)
                    finally:
                        os.close(descriptor)
                events.append(
                    (
                        True,
                        relative,
                        expected_device,
                        expected_inode,
                    )
                )
                for child in reversed(child_directories):
                    events.append((False, *child))
            finally:
                os.close(current)
    finally:
        os.close(root_descriptor)


def create_clean_git_snapshot(
    root: Path,
    source_state: Mapping[str, Any],
    source_key: str,
    profile: Mapping[str, Any],
    *,
    max_logical_bytes: int = DEFAULT_MAX_SNAPSHOT_LOGICAL_BYTES,
    root_budget_bytes: int = DEFAULT_SNAPSHOT_ROOT_BUDGET_BYTES,
    max_files: int = DEFAULT_MAX_SNAPSHOT_FILES,
    free_floor_bytes: int = DEFAULT_NEW_NAMESPACE_FREE_FLOOR_BYTES,
) -> tuple[Path, dict[str, Any]]:
    if source_state["dirty"]:
        raise GateBlocked(
            "dirty-candidate-not-trusted",
            "trusted Cargo gates require a clean candidate and exact HEAD snapshot",
        )
    snapshot_root = validate_private_root(
        Path(str(profile["snapshot_root"])),
        code="unsafe-snapshot-root",
    )
    entries = git_tree_entries(root, str(source_state["head_revision"]))
    if len(entries) > max_files:
        raise GateBlocked(
            "snapshot-file-count-exceeded",
            f"files={len(entries)}, max={max_files}",
        )
    expected_logical = sum(int(item["bytes"]) for item in entries)
    if expected_logical > max_logical_bytes:
        raise GateBlocked(
            "snapshot-logical-size-exceeded",
            f"bytes={expected_logical}, max={max_logical_bytes}",
        )
    maximum_archive_bytes = (
        expected_logical + len(entries) * 4096 + 1024 * 1024
    )
    peak_materialization_reservation = (
        expected_logical + maximum_archive_bytes
    )
    root_usage = namespace_usage(snapshot_root)
    if (
        root_usage["allocated_bytes"] + peak_materialization_reservation
        > root_budget_bytes
    ):
        raise GateBlocked(
            "snapshot-root-budget-exceeded",
            f"observed={root_usage['allocated_bytes']}, "
            f"peak_reservation={peak_materialization_reservation}, "
            f"budget={root_budget_bytes}",
        )
    free = shutil.disk_usage(snapshot_root).free
    free_floor = int(free_floor_bytes)
    if free - peak_materialization_reservation < free_floor:
        raise GateBlocked(
            "snapshot-free-floor-after-reservation",
            f"free={free}, reservation={peak_materialization_reservation}, "
            f"floor={free_floor}",
        )

    nonce = f"{os.getpid()}-{time.monotonic_ns()}"
    temporary = snapshot_root / f".tmp-{source_key}-{nonce}"
    ready = snapshot_root / f"ready-{source_key}-{nonce}"
    temporary.mkdir(mode=0o700)
    temporary_descriptor: int | None = open_directory_no_follow(temporary)
    expected = {item["path"]: item for item in entries}
    observed: list[dict[str, Any]] = []
    process: subprocess.Popen[bytes] | None = None
    published = False
    archive_file = tempfile.TemporaryFile(dir=temporary)
    try:
        process = subprocess.Popen(
            [
                str(GIT_EXECUTABLE),
                *GIT_CONTROL_CONFIG_ARGS,
                "archive",
                "--format=tar",
                str(source_state["head_revision"]),
            ],
            cwd=root,
            env=git_control_environment(),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        archive_bytes = 0

        def spool_archive(chunk: bytes) -> None:
            nonlocal archive_bytes
            archive_bytes += len(chunk)
            if archive_bytes > maximum_archive_bytes:
                raise GateBlocked(
                    "snapshot-archive-size-limit-exceeded",
                    f"bytes={archive_bytes}, max={maximum_archive_bytes}",
                )
            write_all(archive_file.fileno(), chunk)

        returncode, stderr = consume_process_output(
            process,
            label="git archive",
            timeout_seconds=GIT_ARCHIVE_TIMEOUT_SECONDS,
            stdout_consumer=spool_archive,
        )
        if returncode != 0:
            raise GateBlocked(
                "snapshot-archive-failed",
                f"exit={returncode}, stderr_sha256="
                f"{hashlib.sha256(stderr).hexdigest()}",
            )
        archive_file.flush()
        archive_file.seek(0)
        with tarfile.open(fileobj=archive_file, mode="r:") as archive:
            for member in archive:
                relative = Path(member.name)
                if (
                    relative.is_absolute()
                    or not relative.parts
                    or any(
                        part in {"", ".", ".."} for part in relative.parts
                    )
                ):
                    raise GateBlocked(
                        "unsafe-snapshot-archive-path", member.name
                    )
                destination = temporary / relative
                if member.isdir():
                    directory = ensure_snapshot_directory_at(
                        temporary_descriptor, relative.parts
                    )
                    os.close(directory)
                    continue
                if not member.isfile() or member.name not in expected:
                    raise GateBlocked(
                        "unsupported-snapshot-archive-entry", member.name
                    )
                entry = expected[member.name]
                wanted_mode = 0o755 if entry["mode"] == "100755" else 0o644
                if member.size != entry["bytes"]:
                    raise GateBlocked(
                        "snapshot-archive-size-mismatch", member.name
                    )
                parent = ensure_snapshot_directory_at(
                    temporary_descriptor, relative.parts[:-1]
                )
                source = archive.extractfile(member)
                if source is None:
                    os.close(parent)
                    raise GateBlocked(
                        "snapshot-archive-content-missing", member.name
                    )
                digest = hashlib.sha256()
                object_digest = (
                    hashlib.sha1(usedforsecurity=False)
                    if len(entry["oid"]) == 40
                    else hashlib.sha256()
                )
                object_digest.update(
                    f"blob {member.size}\0".encode("ascii")
                )
                size = 0
                descriptor: int | None = None
                try:
                    descriptor = os.open(
                        relative.name,
                        os.O_WRONLY
                        | os.O_CREAT
                        | os.O_EXCL
                        | os.O_CLOEXEC
                        | getattr(os, "O_NOFOLLOW", 0),
                        wanted_mode,
                        dir_fd=parent,
                    )
                    while True:
                        chunk = source.read(1024 * 1024)
                        if not chunk:
                            break
                        write_all(descriptor, chunk)
                        digest.update(chunk)
                        object_digest.update(chunk)
                        size += len(chunk)
                    os.fsync(descriptor)
                finally:
                    if descriptor is not None:
                        os.close(descriptor)
                    os.close(parent)
                    source.close()
                if size != entry["bytes"]:
                    raise GateBlocked(
                        "snapshot-archive-size-mismatch", member.name
                    )
                if object_digest.hexdigest() != entry["oid"]:
                    raise GateBlocked(
                        "snapshot-archive-object-id-mismatch", member.name
                    )
                observed.append(
                    {
                        **entry,
                        "snapshot_bytes": size,
                        "snapshot_sha256": digest.hexdigest(),
                    }
                )
        archive_file.close()
        os.close(temporary_descriptor)
        temporary_descriptor = None
        if {item["path"] for item in observed} != set(expected):
            raise GateBlocked(
                "snapshot-path-set-mismatch",
                f"expected={len(expected)}, observed={len(observed)}",
            )
        observed.sort(key=lambda item: item["path"])
        rust_manifest = build_source_manifest(root)
        snapshot_by_path = {item["path"]: item for item in observed}
        snapshot_rust_files = [
            {
                "path": item["path"],
                "bytes": snapshot_by_path[item["path"]]["snapshot_bytes"],
                "sha256": snapshot_by_path[item["path"]][
                    "snapshot_sha256"
                ],
            }
            for item in rust_manifest["files"]
        ]
        snapshot_rust_sha256 = sha256_json(snapshot_rust_files)
        if (
            snapshot_rust_sha256 != rust_manifest["manifest_sha256"]
            or snapshot_rust_sha256
            != source_state["rust_source_manifest_sha256"]
        ):
            raise GateBlocked(
                "snapshot-rust-manifest-mismatch", snapshot_rust_sha256
            )
        snapshot_files = [
            {
                "path": item["path"],
                "mode": item["mode"],
                "bytes": item["snapshot_bytes"],
                "sha256": item["snapshot_sha256"],
            }
            for item in observed
        ]
        marker = {
            "schema": "codefire.clean_git_source_snapshot.v1",
            "source_key": source_key,
            "candidate_root": str(root),
            "head_revision": source_state["head_revision"],
            "tree": source_state["tree"],
            "git_tree_entry_count": len(entries),
            "git_tree_entries_sha256": sha256_json(entries),
            "snapshot_logical_file_bytes": expected_logical,
            "snapshot_content_manifest_sha256": sha256_json(observed),
            "snapshot_file_manifest_sha256": sha256_json(snapshot_files),
            "rust_source_manifest_sha256": snapshot_rust_sha256,
            "snapshot_path": str(ready),
        }
        marker_raw = marker_bytes(marker)
        marker_path = temporary / SNAPSHOT_MARKER_NAME
        marker_descriptor = os.open(
            marker_path,
            os.O_WRONLY
            | os.O_CREAT
            | os.O_EXCL
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            0o400,
        )
        try:
            write_all(marker_descriptor, marker_raw)
            os.fsync(marker_descriptor)
            os.fchmod(marker_descriptor, 0o400)
        finally:
            os.close(marker_descriptor)
        fsync_tree_and_make_read_only(temporary)
        usage = namespace_usage(temporary)
        root_usage_after = namespace_usage(snapshot_root)
        free_after_snapshot = shutil.disk_usage(snapshot_root).free
        if root_usage_after["allocated_bytes"] > root_budget_bytes:
            raise GateBlocked(
                "snapshot-root-budget-exceeded-after-materialization",
                f"observed={root_usage_after['allocated_bytes']}, "
                f"budget={root_budget_bytes}",
            )
        if free_after_snapshot < free_floor:
            raise GateBlocked(
                "snapshot-free-floor-crossed-after-materialization",
                f"free={free_after_snapshot}, floor={free_floor}",
            )
        root_descriptor = open_directory_no_follow(snapshot_root)
        try:
            os.rename(
                temporary.name,
                ready.name,
                src_dir_fd=root_descriptor,
                dst_dir_fd=root_descriptor,
            )
            published = True
            os.fsync(root_descriptor)
        finally:
            os.close(root_descriptor)
        return ready, {
            **marker,
            "marker_sha256": hashlib.sha256(marker_raw).hexdigest(),
            "allocated_bytes": usage["allocated_bytes"],
            "logical_bytes": usage["logical_bytes"],
            "entry_count": usage["entry_count"],
            "filesystem_free_bytes_before": free,
            "filesystem_free_bytes_after_materialization": (
                free_after_snapshot
            ),
            "root_allocated_bytes_before": root_usage["allocated_bytes"],
            "root_allocated_bytes_after_materialization": (
                root_usage_after["allocated_bytes"]
            ),
            "root_budget_bytes": root_budget_bytes,
        }
    except BaseException as original:
        if process is not None and process.poll() is None:
            terminate_git_process(process)
        if temporary_descriptor is not None:
            os.close(temporary_descriptor)
        if not archive_file.closed:
            archive_file.close()
        partial = ready if published else temporary
        try:
            cleanup_partial_snapshot(snapshot_root, partial)
        except BaseException as cleanup_error:
            raise GateBlocked(
                "partial-snapshot-cleanup-failed",
                f"{partial}: {cleanup_error}",
            ) from original
        raise


def path_value(value: str, cwd: Path) -> Path:
    path = Path(value)
    return (path if path.is_absolute() else cwd / path).resolve(strict=False)


def canonical_profile_environment(
    source: Mapping[str, str],
    *,
    cwd: Path,
    profile: Mapping[str, Any],
) -> tuple[dict[str, str], str | None]:
    ambient_target = source.get("CARGO_TARGET_DIR")
    present_overrides = sorted(
        name
        for name in (
            DANGEROUS_TOOL_OVERRIDE_NAMES
            | DANGEROUS_RUNTIME_OVERRIDE_NAMES
        )
        if name in source
    )
    if present_overrides:
        raise GateBlocked(
            "ambient-tool-or-runtime-override",
            "ambient tool/runtime overrides are not admitted: "
            + ",".join(present_overrides),
        )
    environment = {
        name: value
        for name, value in source.items()
        if (
            name in ADMITTED_AMBIENT_ENVIRONMENT_NAMES
            or name
            in {
                "CARGO_BUILD_JOBS",
                "CARGO_HOME",
                "CARGO_INCREMENTAL",
                "RUSTUP_HOME",
                "RUSTUP_TOOLCHAIN",
                "RUST_TEST_THREADS",
            }
            or name.startswith(ADMITTED_AMBIENT_ENVIRONMENT_PREFIXES)
        )
        and not any(
            pattern.search(name)
            for pattern in SCRUBBED_ENVIRONMENT_PATTERNS
        )
    }
    for key, profile_key in (
        ("RUSTUP_HOME", "rustup_home"),
        ("CARGO_HOME", "cargo_home"),
    ):
        wanted = Path(str(profile[profile_key])).resolve(strict=False)
        raw = environment.get(key)
        if raw is not None and path_value(raw, cwd) != wanted:
            raise GateBlocked(
                f"wrong-{key.lower().replace('_', '-')}",
                f"{key} must resolve to {wanted}",
            )
        environment[key] = str(wanted)

    expected_toolchain = str(profile["rustup_toolchain"])
    ambient_toolchain = environment.get("RUSTUP_TOOLCHAIN")
    if (
        ambient_toolchain is not None
        and ambient_toolchain != expected_toolchain
    ):
        raise GateBlocked(
            "wrong-rustup-toolchain",
            f"RUSTUP_TOOLCHAIN must be {expected_toolchain}",
        )
    environment["RUSTUP_TOOLCHAIN"] = expected_toolchain

    for key, profile_key in (
        ("CARGO_BUILD_JOBS", "max_build_jobs"),
        ("RUST_TEST_THREADS", "max_test_threads"),
    ):
        maximum = int(profile[profile_key])
        raw = environment.get(key)
        if raw is None:
            environment[key] = str(maximum)
            continue
        try:
            parsed = int(raw)
        except ValueError as exc:
            raise GateBlocked(
                f"unsafe-{key.lower().replace('_', '-')}",
                f"{key} must be an integer from 1 through {maximum}",
            ) from exc
        if parsed < 1 or parsed > maximum:
            raise GateBlocked(
                f"unsafe-{key.lower().replace('_', '-')}",
                f"{key} must be from 1 through {maximum}",
            )

    incremental = str(profile["incremental"])
    if environment.get("CARGO_INCREMENTAL", incremental) != incremental:
        raise GateBlocked(
            "unsafe-cargo-incremental",
            f"CARGO_INCREMENTAL must be {incremental}",
        )
    environment["CARGO_INCREMENTAL"] = incremental
    environment["CARGO_TERM_COLOR"] = "never"
    environment[
        "CODEFIRE_CARGO_GATE_PROVENANCE"
    ] = "codefire.source_keyed_cargo_gate.v1"
    environment.pop("CODEFIRE_CARGO_GATE_SOURCE_KEY", None)

    cargo_bin = str(Path(str(profile["cargo_home"])).resolve(strict=False) / "bin")
    path_parts = [
        part
        for part in environment.get("PATH", "").split(os.pathsep)
        if part and Path(part).resolve(strict=False) != Path(cargo_bin)
    ]
    environment["PATH"] = os.pathsep.join([cargo_bin, *path_parts])
    return environment, ambient_target


def stable_runtime_environment_contract(
    environment: Mapping[str, str],
    toolchain: Mapping[str, Any],
) -> dict[str, Any]:
    executables = toolchain.get("executables")
    if not isinstance(executables, Mapping):
        raise GateBlocked(
            "toolchain-environment-contract-missing",
            "toolchain executable identities are unavailable",
        )
    entries: dict[str, Any] = {}
    for key, value in sorted(environment.items()):
        if key in {
            "CARGO_TARGET_DIR",
            "CODEFIRE_CARGO_GATE_SOURCE_KEY",
            "HOME",
            "TMPDIR",
        }:
            continue
        tool_name = RUNTIME_TOOL_ENVIRONMENT_TO_EXECUTABLE.get(key)
        if tool_name is None:
            entries[key] = {"kind": "literal", "value": value}
            continue
        identity = executables.get(tool_name)
        if not isinstance(identity, Mapping):
            raise GateBlocked(
                "toolchain-environment-contract-missing",
                f"{key} requires pinned {tool_name}",
            )
        entries[key] = {
            "kind": "pinned-tool-fd",
            "tool": tool_name,
            "path": identity.get("path"),
            "sha256": identity.get("sha256"),
            "version_sha256": identity.get("version_sha256"),
        }
    entries.update(
        {
            "CARGO_TARGET_DIR": {
                "kind": "derived-artifact-namespace"
            },
            "CODEFIRE_CARGO_GATE_SOURCE_KEY": {
                "kind": "derived-artifact-key"
            },
            "HOME": {
                "kind": "derived-runtime-scratch",
                "relative_path": (
                    f"{RUNTIME_SCRATCH_NAME}/{RUNTIME_HOME_NAME}"
                ),
            },
            "TMPDIR": {
                "kind": "derived-runtime-scratch",
                "relative_path": (
                    f"{RUNTIME_SCRATCH_NAME}/{RUNTIME_TMP_NAME}"
                ),
            },
        }
    )
    return {
        "schema": "codefire.cargo_gate_runtime_environment.v1",
        "entries": entries,
    }


def project_runtime_environment(
    environment: Mapping[str, str],
    *,
    target: Path,
    artifact_key: str,
    runtime_scratch: Mapping[str, Any],
    toolchain: Mapping[str, Any],
    descriptors: Mapping[str, int],
) -> dict[str, Any]:
    projected_environment = dict(environment)
    if projected_environment.get("CARGO_TARGET_DIR") != str(target):
        raise GateBlocked(
            "runtime-target-environment-mismatch", str(target)
        )
    if (
        projected_environment.get("CODEFIRE_CARGO_GATE_SOURCE_KEY")
        != artifact_key
    ):
        raise GateBlocked(
            "runtime-source-key-environment-mismatch", artifact_key
        )
    if projected_environment.get("HOME") != runtime_scratch["home"]["path"]:
        raise GateBlocked(
            "runtime-home-environment-mismatch",
            str(projected_environment.get("HOME")),
        )
    if projected_environment.get("TMPDIR") != runtime_scratch["tmp"]["path"]:
        raise GateBlocked(
            "runtime-tmpdir-environment-mismatch",
            str(projected_environment.get("TMPDIR")),
        )
    for environment_name, tool_name in (
        RUNTIME_TOOL_ENVIRONMENT_TO_EXECUTABLE.items()
    ):
        if environment_name not in projected_environment:
            continue
        descriptor_name = f"toolchain:{tool_name}"
        if descriptor_name not in descriptors:
            raise GateBlocked(
                "runtime-tool-descriptor-missing", descriptor_name
            )
        expected = f"/proc/self/fd/{descriptors[descriptor_name]}"
        if projected_environment[environment_name] != expected:
            raise GateBlocked(
                "runtime-tool-descriptor-mismatch",
                f"{environment_name}: {projected_environment[environment_name]}",
            )
        identity = toolchain["executables"][tool_name]
        projected_environment[environment_name] = str(identity["path"])
    return stable_runtime_environment_contract(
        projected_environment, toolchain
    )


def artifact_environment_sha256(
    contract: Mapping[str, Any],
) -> str:
    return sha256_json(dict(contract))


def profile_contract(profile: Mapping[str, Any]) -> dict[str, Any]:
    return {
        "rustup_home": str(Path(str(profile["rustup_home"])).resolve(strict=False)),
        "cargo_home": str(Path(str(profile["cargo_home"])).resolve(strict=False)),
        "rustup_toolchain": str(profile["rustup_toolchain"]),
        "admitted_target_root": str(
            Path(str(profile["target_dir"])).resolve(strict=False)
        ),
        "admitted_evidence_root": str(
            Path(str(profile["evidence_root"])).resolve(strict=False)
        ),
        "admitted_snapshot_root": str(
            Path(str(profile["snapshot_root"])).resolve(strict=False)
        ),
        "max_build_jobs": int(profile["max_build_jobs"]),
        "max_test_threads": int(profile["max_test_threads"]),
        "incremental": str(profile["incremental"]),
        "runtime_scratch_policy": {
            "schema": "codefire.cargo_gate_runtime_scratch.v1",
            "namespace_relative_path": RUNTIME_SCRATCH_NAME,
            "home_relative_path": (
                f"{RUNTIME_SCRATCH_NAME}/{RUNTIME_HOME_NAME}"
            ),
            "tmp_relative_path": (
                f"{RUNTIME_SCRATCH_NAME}/{RUNTIME_TMP_NAME}"
            ),
            "lifecycle": "exclusive-per-run-removed-before-commit",
        },
    }


def preloaded_policy_identity(
    module_name: str,
    path: Path,
    *,
    deadline: float,
) -> dict[str, Any]:
    item = _PRELOADED_POLICY_MODULES.get(module_name)
    if item is None:
        raise GateBlocked(
            "preloaded-policy-module-missing", module_name
        )
    descriptor = int(item["descriptor"])
    expected = item["identity"]
    metadata = os.fstat(descriptor)
    observed = os.stat(path, follow_symlinks=False)
    current = (
        metadata.st_dev,
        metadata.st_ino,
        stat.S_IMODE(metadata.st_mode),
        metadata.st_uid,
        metadata.st_nlink,
        metadata.st_size,
        metadata.st_mtime_ns,
        metadata.st_ctime_ns,
        fd_sha256(
            descriptor,
            deadline=deadline,
            maximum_bytes=1024 * 1024,
        ),
    )
    if (
        current != expected
        or (observed.st_dev, observed.st_ino) != current[:2]
    ):
        raise GateBlocked(
            "preloaded-policy-module-changed", module_name
        )
    return {
        "path": str(path),
        "bytes": metadata.st_size,
        "sha256": current[-1],
        "device": metadata.st_dev,
        "inode": metadata.st_ino,
        "mode": stat.S_IMODE(metadata.st_mode),
        "uid": metadata.st_uid,
        "nlink": metadata.st_nlink,
        "mtime_ns": metadata.st_mtime_ns,
        "ctime_ns": metadata.st_ctime_ns,
    }


def running_supervisor_identity(
    *, deadline: float
) -> dict[str, Any]:
    if _RUNNING_SOURCE_GATE is None:
        return regular_file_identity(
            SUPERVISOR_POLICY_FILES["supervisor"],
            executable=False,
            deadline=deadline,
            owner_controlled=True,
        )
    descriptor = int(
        _RUNNING_SOURCE_GATE["supervisor_descriptor"]
    )
    expected = dict(_RUNNING_SOURCE_GATE["supervisor_identity"])
    current = _bootstrap_file_identity(
        descriptor,
        maximum_bytes=SOURCE_GATE_BOOTSTRAP_MAX_BYTES,
        owner_controlled=True,
    )
    if current != expected:
        raise GateBlocked(
            "running-supervisor-changed",
            str(SUPERVISOR_POLICY_FILES["supervisor"]),
        )
    return {
        "path": str(SUPERVISOR_POLICY_FILES["supervisor"]),
        **current,
    }


def running_python_identity(*, deadline: float) -> dict[str, Any]:
    if _RUNNING_SOURCE_GATE is None:
        return regular_executable_identity(
            Path(sys.executable).resolve(),
            version_args=["--version"],
            deadline=deadline,
        )
    descriptor = int(_RUNNING_SOURCE_GATE["python_descriptor"])
    expected = dict(_RUNNING_SOURCE_GATE["python_identity"])
    current = _bootstrap_file_identity(
        descriptor,
        maximum_bytes=SOURCE_GATE_BOOTSTRAP_MAX_PYTHON_BYTES,
        owner_controlled=False,
    )
    running = os.stat("/proc/self/exe")
    if (
        current != expected
        or (running.st_dev, running.st_ino)
        != (current["device"], current["inode"])
    ):
        raise GateBlocked(
            "running-python-changed", "/proc/self/exe"
        )
    version = sys.version.replace("\n", " ")
    return {
        "path": str(Path(sys.executable).resolve(strict=False)),
        **current,
        "version": version,
        "version_sha256": hashlib.sha256(
            version.encode("utf-8")
        ).hexdigest(),
    }


def regular_file_identity(
    path: Path,
    *,
    executable: bool,
    deadline: float | None = None,
    owner_controlled: bool = False,
) -> dict[str, Any]:
    lexical = path.absolute()
    try:
        resolved = lexical.resolve(strict=True)
    except OSError as exc:
        raise GateBlocked(
            "toolchain-executable-unavailable", f"{lexical}: {exc}"
        ) from exc
    if lexical != resolved:
        raise GateBlocked(
            "toolchain-executable-not-pinned",
            f"expected a real executable without symlinks: {lexical}",
        )
    descriptor = os.open(
        lexical,
        os.O_RDONLY | os.O_CLOEXEC | getattr(os, "O_NOFOLLOW", 0),
    )
    try:
        metadata = os.fstat(descriptor)
        observed = os.stat(lexical, follow_symlinks=False)
        if (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_nlink != 1
            or metadata.st_size > MAX_PINNED_TOOL_BYTES
            or (
                owner_controlled
                and (
                    metadata.st_uid != os.geteuid()
                    or metadata.st_mode & 0o022
                )
            )
            or (executable and not os.access(lexical, os.X_OK))
            or (metadata.st_dev, metadata.st_ino)
            != (observed.st_dev, observed.st_ino)
        ):
            raise GateBlocked(
                "toolchain-executable-not-pinned",
                f"expected a single-link regular executable: {lexical}",
            )
        return {
            "path": str(resolved),
            "bytes": metadata.st_size,
            "sha256": fd_sha256(
                descriptor,
                deadline=deadline,
                maximum_bytes=MAX_PINNED_TOOL_BYTES,
            ),
            "device": metadata.st_dev,
            "inode": metadata.st_ino,
            "mode": stat.S_IMODE(metadata.st_mode),
            "uid": metadata.st_uid,
            "nlink": metadata.st_nlink,
            "mtime_ns": metadata.st_mtime_ns,
            "ctime_ns": metadata.st_ctime_ns,
        }
    finally:
        os.close(descriptor)


def open_parent_death_helper() -> int:
    lexical = PARENT_DEATH_HELPER.absolute()
    if lexical.resolve(strict=True) != lexical:
        raise GateBlocked(
            "parent-death-helper-identity-unsafe", str(lexical)
        )
    descriptor = os.open(
        lexical,
        os.O_RDONLY | os.O_CLOEXEC | getattr(os, "O_NOFOLLOW", 0),
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
            raise GateBlocked(
                "parent-death-helper-identity-unsafe", str(lexical)
            )
        return descriptor
    except BaseException:
        os.close(descriptor)
        raise


def bounded_tool_version_output(
    target_descriptor: int,
    version_args: list[str],
    *,
    label: str,
    overall_deadline: float,
) -> tuple[int, bytes]:
    helper_descriptor = open_parent_death_helper()
    launch = [
        str(Path(sys.executable).resolve(strict=True)),
        "-I",
        "-S",
        "-B",
        "-X",
        f"pycache_prefix={POLICY_PYCACHE_PREFIX}",
        f"/proc/self/fd/{helper_descriptor}",
        "--parent-pid",
        str(os.getpid()),
        "--",
        "/usr/bin/setpriv",
        "--pdeathsig",
        "SIGKILL",
        "/usr/bin/unshare",
        "-Urpf",
        "--mount-proc",
        "--kill-child=SIGKILL",
        "--",
        f"/proc/self/fd/{target_descriptor}",
        *version_args,
    ]
    environment = {
        "HOME": "/nonexistent",
        "LANG": "C",
        "LC_ALL": "C",
        "PATH": "/usr/bin:/bin",
        "PYTHONDONTWRITEBYTECODE": "1",
        "PYTHONNOUSERSITE": "1",
        "XDG_CONFIG_HOME": "/nonexistent",
    }
    try:
        process = subprocess.Popen(
            launch,
            env=environment,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            pass_fds=(helper_descriptor, target_descriptor),
            start_new_session=True,
        )
    except OSError as exc:
        raise GateBlocked(
            "toolchain-version-probe-failed", f"{label}: {exc}"
        ) from exc
    finally:
        os.close(helper_descriptor)
    if process.stdout is None:
        terminate_git_process(process)
        raise GateBlocked(
            "toolchain-version-probe-failed",
            f"{label}: control pipe unavailable",
        )
    descriptor = process.stdout.fileno()
    os.set_blocking(descriptor, False)
    output = bytearray()
    deadline = min(
        overall_deadline,
        time.monotonic() + TOOL_VERSION_TIMEOUT_SECONDS,
    )
    try:
        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise GateBlocked(
                    "toolchain-version-probe-timeout",
                    f"{label}: exceeded {TOOL_VERSION_TIMEOUT_SECONDS}s",
                )
            try:
                readable, _, _ = select.select(
                    [descriptor], [], [], remaining
                )
            except InterruptedError:
                continue
            if not readable:
                raise GateBlocked(
                    "toolchain-version-probe-timeout",
                    f"{label}: exceeded {TOOL_VERSION_TIMEOUT_SECONDS}s",
                )
            try:
                chunk = os.read(
                    descriptor,
                    min(
                        64 * 1024,
                        MAX_TOOL_VERSION_OUTPUT_BYTES
                        + 1
                        - len(output),
                    ),
                )
            except BlockingIOError:
                continue
            if not chunk:
                break
            output.extend(chunk)
            if len(output) > MAX_TOOL_VERSION_OUTPUT_BYTES:
                raise GateBlocked(
                    "toolchain-version-probe-output-exceeded",
                    f"{label}: exceeded "
                    f"{MAX_TOOL_VERSION_OUTPUT_BYTES} bytes",
                )
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise GateBlocked(
                "toolchain-version-probe-timeout",
                f"{label}: exceeded {TOOL_VERSION_TIMEOUT_SECONDS}s",
            )
        try:
            returncode = process.wait(timeout=remaining)
        except subprocess.TimeoutExpired as exc:
            raise GateBlocked(
                "toolchain-version-probe-timeout",
                f"{label}: exceeded {TOOL_VERSION_TIMEOUT_SECONDS}s",
            ) from exc
        return returncode, bytes(output)
    except BaseException:
        terminate_git_process(process)
        raise
    finally:
        process.stdout.close()


def regular_executable_identity(
    path: Path,
    *,
    version_args: list[str],
    deadline: float | None = None,
) -> dict[str, Any]:
    if deadline is None:
        deadline = time.monotonic() + TOOL_IDENTITY_TOTAL_SECONDS
    identity = regular_file_identity(
        path, executable=True, deadline=deadline
    )
    resolved = Path(identity["path"])
    descriptor = os.open(
        resolved,
        os.O_RDONLY | os.O_CLOEXEC | getattr(os, "O_NOFOLLOW", 0),
    )
    try:
        metadata = os.fstat(descriptor)
        if (
            metadata.st_dev != identity["device"]
            or metadata.st_ino != identity["inode"]
            or metadata.st_size != identity["bytes"]
            or stat.S_IMODE(metadata.st_mode) != identity["mode"]
            or metadata.st_uid != identity["uid"]
            or metadata.st_nlink != identity["nlink"]
            or metadata.st_mtime_ns != identity["mtime_ns"]
            or metadata.st_ctime_ns != identity["ctime_ns"]
            or fd_sha256(
                descriptor,
                deadline=deadline,
                maximum_bytes=MAX_PINNED_TOOL_BYTES,
            )
            != identity["sha256"]
        ):
            raise GateBlocked(
                "toolchain-version-probe-identity-mismatch",
                str(resolved),
            )
        version_returncode, version_output = bounded_tool_version_output(
            descriptor,
            version_args,
            label=str(resolved),
            overall_deadline=deadline,
        )
        if version_returncode != 0:
            raise GateBlocked(
                "toolchain-version-probe-failed",
                f"{resolved}: exit={version_returncode}, "
                f"bytes={len(version_output)}",
            )
        after = os.fstat(descriptor)
        observed = os.stat(resolved, follow_symlinks=False)
        if (
            after.st_dev != identity["device"]
            or after.st_ino != identity["inode"]
            or after.st_size != identity["bytes"]
            or stat.S_IMODE(after.st_mode) != identity["mode"]
            or after.st_uid != identity["uid"]
            or after.st_nlink != identity["nlink"]
            or after.st_mtime_ns != identity["mtime_ns"]
            or after.st_ctime_ns != identity["ctime_ns"]
            or (observed.st_dev, observed.st_ino)
            != (identity["device"], identity["inode"])
            or observed.st_size != identity["bytes"]
            or stat.S_IMODE(observed.st_mode) != identity["mode"]
            or observed.st_uid != identity["uid"]
            or observed.st_nlink != identity["nlink"]
            or observed.st_mtime_ns != identity["mtime_ns"]
            or observed.st_ctime_ns != identity["ctime_ns"]
            or fd_sha256(
                descriptor,
                deadline=deadline,
                maximum_bytes=MAX_PINNED_TOOL_BYTES,
            )
            != identity["sha256"]
        ):
            raise GateBlocked(
                "toolchain-version-probe-changed", str(resolved)
            )
        return {
            **identity,
            "version_sha256": hashlib.sha256(version_output).hexdigest(),
            "version": version_output.decode(
                "utf-8", errors="replace"
            ).strip(),
        }
    finally:
        os.close(descriptor)


def probe_toolchain(
    profile: Mapping[str, Any], subcommand: str
) -> dict[str, Any]:
    deadline = time.monotonic() + TOOL_IDENTITY_TOTAL_SECONDS
    toolchain_root = (
        Path(str(profile["rustup_home"])).resolve(strict=False)
        / "toolchains"
        / str(profile["rustup_toolchain"])
    )
    bin_dir = toolchain_root / "bin"
    required: dict[str, list[str]] = {
        "cargo": ["--version", "--verbose"],
        "rustc": ["--version", "--verbose"],
        "rustdoc": ["--version"],
    }
    if subcommand == "clippy":
        required.update(
            {
                "cargo-clippy": ["--version"],
                "clippy-driver": ["--version"],
            }
        )
    if subcommand == "fmt":
        required.update(
            {
                "cargo-fmt": ["--version"],
                "rustfmt": ["--version"],
            }
        )
    executables = {
        name: regular_executable_identity(
            bin_dir / name,
            version_args=version_args,
            deadline=deadline,
        )
        for name, version_args in sorted(required.items())
    }
    payload = {
        "schema": "codefire.rust_toolchain_identity.v1",
        "toolchain": str(profile["rustup_toolchain"]),
        "toolchain_root": str(toolchain_root),
        "executables": executables,
    }
    payload["identity_sha256"] = sha256_json(payload)
    return payload


@functools.lru_cache(maxsize=1)
def isolation_identity() -> dict[str, Any]:
    deadline = time.monotonic() + TOOL_IDENTITY_TOTAL_SECONDS
    tools = {
        "helper": regular_file_identity(
            READONLY_HELPER,
            executable=False,
            deadline=deadline,
            owner_controlled=True,
        ),
        "pdeath-helper": regular_file_identity(
            PARENT_DEATH_HELPER,
            executable=False,
            deadline=deadline,
            owner_controlled=True,
        ),
        "git": regular_executable_identity(
            GIT_EXECUTABLE,
            version_args=["--version"],
            deadline=deadline,
        ),
        "mount": regular_executable_identity(
            Path("/usr/bin/mount"),
            version_args=["--version"],
            deadline=deadline,
        ),
        "python": running_python_identity(deadline=deadline),
        "unshare": regular_executable_identity(
            Path("/usr/bin/unshare"),
            version_args=["--version"],
            deadline=deadline,
        ),
        "setpriv": regular_executable_identity(
            Path("/usr/bin/setpriv"),
            version_args=["--version"],
            deadline=deadline,
        ),
    }
    for name, path in sorted(SUPERVISOR_POLICY_FILES.items()):
        tools[f"policy-{name}"] = (
            running_supervisor_identity(deadline=deadline)
            if name == "supervisor"
            else preloaded_policy_identity(
                PRELOADED_POLICY_MODULE_NAMES[name],
                path,
                deadline=deadline,
            )
            if name in PRELOADED_POLICY_MODULE_NAMES
            else regular_file_identity(
                path,
                executable=False,
                deadline=deadline,
                owner_controlled=True,
            )
        )
    payload = {
        "schema": "codefire.readonly_source_isolation.v1",
        "tools": tools,
        "unshare_flags": list(UNSHARE_ISOLATION_FLAGS),
        "manager_death_policy": (
            "pinned isolated Python runs the pinned parent-death helper, "
            "which sets SIGKILL and verifies the expected manager before "
            "exec; pinned setpriv preserves SIGKILL on pinned unshare; "
            "unshare kills namespace init and descendants"
        ),
        "supervisor_policy_bundle": {
            "schema": "codefire.cargo_gate_policy_bundle.v1",
            "members": sorted(
                name
                for name in tools
                if name.startswith("policy-")
                or name in {"helper", "pdeath-helper"}
            ),
            "binding": (
                "exact opened file identities are included in this "
                "isolation identity and revalidated after execution"
            ),
        },
        "companion_tool_dispatch": (
            "Cargo and all admitted Rust companion-tool environment paths "
            "use inherited pinned /proc/self/fd descriptors"
        ),
        "supplemental_group_policy": (
            "user-namespace setgroups is denied; post-check admits only "
            "mapped root gid and kernel overflowgid"
        ),
        "python_flags": [
            "-I",
            "-S",
            "-B",
            "-X",
            f"pycache_prefix={POLICY_PYCACHE_PREFIX}",
        ],
        "local_import_policy": (
            "repo __pycache__ is bypassed via explicit -B and "
            "-X pycache_prefix=/dev/null; exact pinned .py bytes are used"
        ),
        "git_control": {
            "config_args": list(GIT_CONTROL_CONFIG_ARGS),
            "environment": git_control_environment(),
        },
    }
    payload["identity_sha256"] = sha256_json(payload)
    return payload


def fd_sha256(
    descriptor: int,
    *,
    deadline: float | None = None,
    maximum_bytes: int = MAX_PINNED_TOOL_BYTES,
) -> str:
    metadata = os.fstat(descriptor)
    if metadata.st_size > maximum_bytes:
        raise GateBlocked(
            "bounded-file-size-exceeded",
            f"bytes={metadata.st_size}, maximum={maximum_bytes}",
        )
    digest = hashlib.sha256()
    offset = 0
    while True:
        if deadline is not None and time.monotonic() > deadline:
            raise GateBlocked(
                "bounded-file-hash-deadline-exceeded",
                f"maximum={maximum_bytes}",
            )
        chunk = os.pread(descriptor, 1024 * 1024, offset)
        if not chunk:
            break
        digest.update(chunk)
        offset += len(chunk)
        if offset > maximum_bytes:
            raise GateBlocked(
                "bounded-file-size-exceeded",
                f"bytes>{maximum_bytes}",
            )
    return digest.hexdigest()


@contextmanager
def pinned_tool_files(
    toolchain: Mapping[str, Any],
    isolation: Mapping[str, Any],
) -> Iterator[dict[str, Any]]:
    identities: dict[str, Mapping[str, Any]] = {}
    for name, identity in toolchain["executables"].items():
        identities[f"toolchain:{name}"] = identity
    for name, identity in isolation["tools"].items():
        identities[f"isolation:{name}"] = identity
    descriptors: dict[str, int] = {}
    held_descriptors: dict[str, int] = {}
    if _RUNNING_SOURCE_GATE is not None:
        held_descriptors = {
            "isolation:policy-supervisor": int(
                _RUNNING_SOURCE_GATE["supervisor_descriptor"]
            ),
            "isolation:python": int(
                _RUNNING_SOURCE_GATE["python_descriptor"]
            ),
        }
    held_names: set[str] = set()
    hash_deadline = time.monotonic() + TOOL_IDENTITY_TOTAL_SECONDS
    try:
        for name, identity in sorted(identities.items()):
            if name in held_descriptors:
                descriptor = os.dup(held_descriptors[name])
                held_names.add(name)
            else:
                descriptor = os.open(
                    str(identity["path"]),
                    os.O_RDONLY
                    | os.O_CLOEXEC
                    | getattr(os, "O_NOFOLLOW", 0),
                )
            metadata = os.fstat(descriptor)
            if (
                not stat.S_ISREG(metadata.st_mode)
                or metadata.st_nlink != identity["nlink"]
                or metadata.st_dev != identity["device"]
                or metadata.st_ino != identity["inode"]
                or stat.S_IMODE(metadata.st_mode) != identity["mode"]
                or metadata.st_uid != identity["uid"]
                or metadata.st_size != identity["bytes"]
                or metadata.st_mtime_ns != identity["mtime_ns"]
                or metadata.st_ctime_ns != identity["ctime_ns"]
                or fd_sha256(
                    descriptor,
                    deadline=hash_deadline,
                    maximum_bytes=MAX_PINNED_TOOL_BYTES,
                )
                != identity["sha256"]
            ):
                os.close(descriptor)
                raise GateBlocked("pinned-tool-identity-mismatch", name)
            descriptors[name] = descriptor
        yield {
            "descriptors": descriptors,
            "pass_fds": tuple(sorted(descriptors.values())),
        }
        hash_deadline = time.monotonic() + TOOL_IDENTITY_TOTAL_SECONDS
        for name, descriptor in descriptors.items():
            identity = identities[name]
            metadata = os.fstat(descriptor)
            if name in held_names:
                current_path = metadata
            else:
                try:
                    current_path = os.stat(
                        str(identity["path"]), follow_symlinks=False
                    )
                except OSError as exc:
                    raise GateBlocked(
                        "pinned-tool-path-changed-during-gate",
                        f"{name}: {exc}",
                    ) from exc
            if (
                metadata.st_dev != identity["device"]
                or metadata.st_ino != identity["inode"]
                or metadata.st_size != identity["bytes"]
                or stat.S_IMODE(metadata.st_mode) != identity["mode"]
                or metadata.st_uid != identity["uid"]
                or metadata.st_nlink != identity["nlink"]
                or metadata.st_mtime_ns != identity["mtime_ns"]
                or metadata.st_ctime_ns != identity["ctime_ns"]
                or current_path.st_dev != identity["device"]
                or current_path.st_ino != identity["inode"]
                or current_path.st_size != identity["bytes"]
                or stat.S_IMODE(current_path.st_mode) != identity["mode"]
                or current_path.st_uid != identity["uid"]
                or current_path.st_nlink != identity["nlink"]
                or current_path.st_mtime_ns != identity["mtime_ns"]
                or current_path.st_ctime_ns != identity["ctime_ns"]
                or fd_sha256(
                    descriptor,
                    deadline=hash_deadline,
                    maximum_bytes=MAX_PINNED_TOOL_BYTES,
                )
                != identity["sha256"]
            ):
                raise GateBlocked(
                    "pinned-tool-changed-during-gate", name
                )
    finally:
        for descriptor in descriptors.values():
            os.close(descriptor)


@contextmanager
def pinned_execution_directories(
    directories: Mapping[str, Path],
) -> Iterator[dict[str, Any]]:
    descriptors: dict[str, int] = {}
    identities: dict[str, tuple[int, int]] = {}
    try:
        for name, path in sorted(directories.items()):
            lexical = path.absolute()
            resolved = lexical.resolve(strict=True)
            if lexical != resolved:
                raise GateBlocked(
                    "pinned-directory-symlink", f"{name}: {lexical}"
                )
            descriptor = os.open(
                lexical,
                os.O_RDONLY
                | os.O_DIRECTORY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
            )
            opened = os.fstat(descriptor)
            observed = os.stat(lexical, follow_symlinks=False)
            identity = (opened.st_dev, opened.st_ino)
            if (
                not stat.S_ISDIR(observed.st_mode)
                or identity != (observed.st_dev, observed.st_ino)
            ):
                os.close(descriptor)
                raise GateBlocked(
                    "pinned-directory-identity-mismatch",
                    f"{name}: {lexical}",
                )
            descriptors[name] = descriptor
            identities[name] = identity
        yield {
            "descriptors": descriptors,
            "pass_fds": tuple(sorted(descriptors.values())),
        }
        for name, descriptor in descriptors.items():
            path = directories[name].absolute()
            opened = os.fstat(descriptor)
            try:
                observed = os.stat(path, follow_symlinks=False)
            except OSError as exc:
                raise GateBlocked(
                    "pinned-directory-path-changed-during-gate",
                    f"{name}: {exc}",
                ) from exc
            if (
                (opened.st_dev, opened.st_ino) != identities[name]
                or (observed.st_dev, observed.st_ino) != identities[name]
            ):
                raise GateBlocked(
                    "pinned-directory-changed-during-gate", name
                )
    finally:
        for descriptor in descriptors.values():
            os.close(descriptor)


@contextmanager
def pinned_control_files(
    files: Mapping[str, Path],
) -> Iterator[dict[str, Any]]:
    descriptors: dict[str, int] = {}
    identities: dict[str, tuple[int, int, int, int, int, int, str]] = {}
    try:
        for name, path in sorted(files.items()):
            lexical = path.absolute()
            if lexical.resolve(strict=True) != lexical:
                raise GateBlocked(
                    "pinned-control-file-symlink", f"{name}: {lexical}"
                )
            descriptor = os.open(
                lexical,
                os.O_RDONLY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
            )
            metadata = os.fstat(descriptor)
            if (
                not stat.S_ISREG(metadata.st_mode)
                or metadata.st_nlink != 1
                or metadata.st_uid != os.geteuid()
                or metadata.st_size > 1024 * 1024
            ):
                os.close(descriptor)
                raise GateBlocked(
                    "unsafe-pinned-control-file", f"{name}: {lexical}"
                )
            identity = (
                metadata.st_dev,
                metadata.st_ino,
                stat.S_IMODE(metadata.st_mode),
                metadata.st_uid,
                metadata.st_nlink,
                metadata.st_size,
                fd_sha256(descriptor),
            )
            observed = os.stat(lexical, follow_symlinks=False)
            if (observed.st_dev, observed.st_ino) != identity[:2]:
                os.close(descriptor)
                raise GateBlocked(
                    "pinned-control-file-identity-mismatch", name
                )
            descriptors[name] = descriptor
            identities[name] = identity
        yield {
            "descriptors": descriptors,
            "pass_fds": tuple(sorted(descriptors.values())),
        }
        for name, descriptor in descriptors.items():
            path = files[name].absolute()
            metadata = os.fstat(descriptor)
            try:
                observed = os.stat(path, follow_symlinks=False)
            except OSError as exc:
                raise GateBlocked(
                    "pinned-control-file-path-changed-during-gate",
                    f"{name}: {exc}",
                ) from exc
            current = (
                metadata.st_dev,
                metadata.st_ino,
                stat.S_IMODE(metadata.st_mode),
                metadata.st_uid,
                metadata.st_nlink,
                metadata.st_size,
                fd_sha256(descriptor),
            )
            if (
                current != identities[name]
                or (observed.st_dev, observed.st_ino) != current[:2]
            ):
                raise GateBlocked(
                    "pinned-control-file-changed-during-gate", name
                )
    finally:
        for descriptor in descriptors.values():
            os.close(descriptor)


def isolated_command(
    root: Path,
    snapshot: Path,
    snapshot_identity: Mapping[str, Any],
    admitted_root: Path,
    evidence_root: Path,
    target: Path,
    cargo_command: list[str],
    primary_tool_name: str,
    toolchain: Mapping[str, Any],
    isolation: Mapping[str, Any],
    pinned: Mapping[str, Any],
) -> list[str]:
    descriptors = pinned["descriptors"]
    primary_descriptor = descriptors[f"toolchain:{primary_tool_name}"]
    pass_fds = ",".join(str(item) for item in pinned["pass_fds"])
    return [
        f"/proc/self/fd/{descriptors['isolation:python']}",
        "-I",
        "-S",
        "-B",
        "-X",
        f"pycache_prefix={POLICY_PYCACHE_PREFIX}",
        f"/proc/self/fd/{descriptors['isolation:pdeath-helper']}",
        "--parent-pid",
        str(os.getpid()),
        "--",
        f"/proc/self/fd/{descriptors['isolation:setpriv']}",
        "--pdeathsig",
        "SIGKILL",
        f"/proc/self/fd/{descriptors['isolation:unshare']}",
        # Must stay identical to the recorded isolation identity; see
        # UNSHARE_ISOLATION_FLAGS.
        *UNSHARE_ISOLATION_FLAGS,
        "--",
        f"/proc/self/fd/{descriptors['isolation:python']}",
        "-I",
        "-S",
        f"/proc/self/fd/{descriptors['isolation:helper']}",
        "--source",
        str(root),
        "--snapshot",
        str(snapshot),
        "--snapshot-root",
        str(snapshot.parent),
        "--toolchain-root",
        str(toolchain["toolchain_root"]),
        "--target",
        str(target),
        "--admitted-root",
        str(admitted_root),
        "--evidence-root",
        str(evidence_root),
        "--source-fd",
        str(descriptors["directory:source"]),
        "--snapshot-fd",
        str(descriptors["directory:snapshot"]),
        "--snapshot-root-fd",
        str(descriptors["directory:snapshot-root"]),
        "--snapshot-marker-sha256",
        str(snapshot_identity["marker_sha256"]),
        "--snapshot-file-manifest-sha256",
        str(snapshot_identity["snapshot_file_manifest_sha256"]),
        "--snapshot-file-count",
        str(snapshot_identity["git_tree_entry_count"]),
        "--snapshot-logical-file-bytes",
        str(snapshot_identity["snapshot_logical_file_bytes"]),
        "--private-source-tmpfs-bytes",
        str(private_source_tmpfs_bytes(snapshot_identity)),
        "--toolchain-root-fd",
        str(descriptors["directory:toolchain"]),
        "--target-fd",
        str(descriptors["directory:target"]),
        "--admitted-root-fd",
        str(descriptors["directory:admitted-root"]),
        "--evidence-root-fd",
        str(descriptors["directory:evidence-root"]),
        "--target-marker-fd",
        str(descriptors["control:target-marker"]),
        "--target-lock-fd",
        str(descriptors["control:target-lock"]),
        "--target-run-state-fd",
        str(descriptors["control:target-run-state"]),
        "--mount-fd",
        str(descriptors["isolation:mount"]),
        "--setpriv-fd",
        str(descriptors["isolation:setpriv"]),
        "--python-fd",
        str(descriptors["isolation:python"]),
        "--helper-fd",
        str(descriptors["isolation:helper"]),
        "--primary-fd",
        str(primary_descriptor),
        "--pass-fds",
        pass_fds,
        "--",
        *cargo_command,
    ]


def private_source_tmpfs_bytes(
    snapshot_identity: Mapping[str, Any],
) -> int:
    logical = int(snapshot_identity["snapshot_logical_file_bytes"])
    file_count = int(snapshot_identity["git_tree_entry_count"])
    if logical < 0 or file_count < 1:
        raise GateBlocked(
            "invalid-private-source-size",
            f"logical={logical}, files={file_count}",
        )
    metadata_reserve = 16 * 1024**2 + file_count * 8192
    admitted = max(64 * 1024**2, logical + metadata_reserve)
    page = 4096
    admitted = ((admitted + page - 1) // page) * page
    if admitted > MAX_PRIVATE_SOURCE_TMPFS_BYTES:
        raise GateBlocked(
            "private-source-tmpfs-bound-exceeded",
            f"bytes={admitted}, max={MAX_PRIVATE_SOURCE_TMPFS_BYTES}",
        )
    return admitted


def build_source_identity(
    source_state: dict[str, Any],
    *,
    environment_sha256: str,
    profile: Mapping[str, Any],
    toolchain: Mapping[str, Any],
    isolation: Mapping[str, Any],
) -> tuple[str, dict[str, Any]]:
    payload = {
        "schema": SOURCE_KEY_SCHEMA,
        "candidate": source_state,
        "artifact_environment_sha256": environment_sha256,
        "execution_profile": profile_contract(profile),
        "toolchain_identity": toolchain,
        "readonly_isolation_identity": isolation,
    }
    return sha256_json(payload), payload


def build_artifact_identity(
    source_state: Mapping[str, Any],
    *,
    environment_sha256: str,
    profile: Mapping[str, Any],
    toolchain: Mapping[str, Any],
    isolation: Mapping[str, Any],
) -> tuple[str, dict[str, Any]]:
    payload = {
        "schema": ARTIFACT_KEY_SCHEMA,
        "candidate_root": source_state["candidate_root"],
        "tree": source_state["tree"],
        "rust_source_manifest_sha256": source_state[
            "rust_source_manifest_sha256"
        ],
        "rust_source_file_count": source_state[
            "rust_source_file_count"
        ],
        "artifact_environment_sha256": environment_sha256,
        "execution_profile": profile_contract(profile),
        "toolchain_identity": toolchain,
        "readonly_isolation_identity": isolation,
    }
    return sha256_json(payload), payload


def normalize_cargo_argv(
    command: list[str],
    *,
    profile: Mapping[str, Any],
) -> tuple[list[str], str]:
    if len(command) < 2 or command[0] != "cargo":
        raise GateBlocked(
            "noncanonical-cargo-argv",
            "command must begin with the literal argv entries: cargo <subcommand>",
        )
    if any("\0" in item for item in command):
        raise GateBlocked("nul-in-argv", "Cargo argv contains a NUL byte")
    subcommand = command[1]
    if subcommand not in ALLOWED_SUBCOMMANDS:
        raise GateBlocked(
            "cargo-subcommand-not-admitted",
            f"Cargo subcommand {subcommand!r} is not an admitted validation gate",
        )
    for index, item in enumerate(command[2:], 2):
        if (
            item in {"--config", "--manifest-path", "--target-dir"}
            or item.startswith("--config=")
            or item.startswith("--manifest-path=")
            or item.startswith("--target-dir=")
            or item.startswith("+")
        ):
            raise GateBlocked(
                "cargo-path-or-toolchain-override",
                f"Cargo argv[{index}] is not allowed: {item}",
            )
    if subcommand in LOCKED_SUBCOMMANDS and not (
        "--locked" in command or "--frozen" in command
    ):
        raise GateBlocked(
            "cargo-lockfile-not-frozen",
            f"cargo {subcommand} requires --locked or --frozen",
        )
    if subcommand == "test" and "--no-run" in command[2:]:
        raise GateBlocked(
            "cargo-test-no-run-not-admitted",
            "cargo test --no-run is compile-only and cannot satisfy the "
            "trusted test-execution contract",
        )
    if subcommand == "fmt":
        try:
            separator = command.index("--", 2)
        except ValueError as exc:
            raise GateBlocked(
                "cargo-fmt-check-required",
                "cargo fmt must pass -- --check",
            ) from exc
        if "--check" not in command[separator + 1 :]:
            raise GateBlocked(
                "cargo-fmt-check-required",
                "cargo fmt must pass -- --check",
            )
    if subcommand == "clippy" and any(
        item in {"--fix", "--allow-dirty", "--allow-staged"}
        or item.startswith(
            ("--fix=", "--allow-dirty=", "--allow-staged=")
        )
        for item in command[2:]
    ):
        raise GateBlocked(
            "side-effecting-clippy-argv",
            "cargo clippy fix/dirty/staged modes are not validation gates",
        )
    if subcommand == "doc" and any(
        item == "--open" or item.startswith("--open=")
        for item in command[2:]
    ):
        raise GateBlocked(
            "side-effecting-cargo-doc-argv",
            "cargo doc --open is not an admitted validation gate",
        )
    return list(command), subcommand


def validate_admitted_root(profile: Mapping[str, Any]) -> Path:
    lexical = Path(str(profile["target_dir"])).absolute()
    return validate_private_root(
        lexical, code="unsafe-admitted-target-root"
    )


def namespace_path(admitted_root: Path, source_key: str) -> Path:
    if re.fullmatch(r"[0-9a-f]{64}", source_key) is None:
        raise GateBlocked("invalid-source-key", source_key)
    target = admitted_root / SOURCE_KEY_TARGET_DIRECTORY / source_key
    try:
        target.resolve(strict=False).relative_to(admitted_root)
    except ValueError as exc:
        raise GateBlocked("target-outside-admitted-root", str(target)) from exc
    return target


def validate_ambient_target(
    raw_target: str | None,
    *,
    cwd: Path,
    admitted_root: Path,
    expected_target: Path,
) -> None:
    if raw_target is None:
        return
    observed = path_value(raw_target, cwd)
    if observed not in {admitted_root, expected_target}:
        raise GateBlocked(
            "ambient-target-not-current-source",
            f"CARGO_TARGET_DIR resolves to {observed}; expected {admitted_root} "
            f"or {expected_target}",
        )


@contextmanager
def exclusive_lock(
    path: Path,
    timeout_seconds: float,
    *,
    allow_parent_removal: bool = False,
    on_error: Callable[[], None] | None = None,
) -> Iterator[float]:
    if not math.isfinite(timeout_seconds) or timeout_seconds < 0:
        raise GateBlocked("invalid-lock-timeout", str(timeout_seconds))
    flags = os.O_RDWR | os.O_CREAT | os.O_CLOEXEC | getattr(os, "O_NOFOLLOW", 0)
    started = time.monotonic()
    descriptor: int | None = None
    identity: tuple[int, int] | None = None
    try:
        while True:
            try:
                descriptor = os.open(path, flags, 0o600)
            except OSError as exc:
                raise GateBlocked(
                    "lock-open-failed", f"{path}: {exc}"
                ) from exc
            metadata = os.fstat(descriptor)
            if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
                raise GateBlocked("lock-not-regular", str(path))
            identity = (metadata.st_dev, metadata.st_ino)
            while True:
                try:
                    fcntl.flock(
                        descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB
                    )
                    break
                except BlockingIOError:
                    waited = time.monotonic() - started
                    if waited >= timeout_seconds:
                        raise GateBlocked(
                            "exclusive-lock-timeout",
                            f"timed out waiting for {path} "
                            f"after {waited:.3f}s",
                        )
                    time.sleep(
                        min(
                            0.05,
                            max(0.0, timeout_seconds - waited),
                        )
                    )
            try:
                observed_before_entry = os.stat(
                    path, follow_symlinks=False
                )
            except OSError:
                observed_before_entry = None
            current = os.fstat(descriptor)
            if (
                observed_before_entry is not None
                and stat.S_ISREG(current.st_mode)
                and current.st_nlink == 1
                and (current.st_dev, current.st_ino) == identity
                and (
                    observed_before_entry.st_dev,
                    observed_before_entry.st_ino,
                )
                == identity
            ):
                break
            fcntl.flock(descriptor, fcntl.LOCK_UN)
            os.close(descriptor)
            descriptor = None
            if time.monotonic() - started >= timeout_seconds:
                raise GateBlocked(
                    "lock-path-changed-before-entry", str(path)
                )
        try:
            yield time.monotonic() - started
            assert descriptor is not None
            assert identity is not None
            current = os.fstat(descriptor)
            try:
                observed = os.stat(path, follow_symlinks=False)
            except OSError as exc:
                if (
                    allow_parent_removal
                    and isinstance(exc, FileNotFoundError)
                    and not path.parent.exists()
                ):
                    return
                raise GateBlocked(
                    "lock-path-changed-while-held", f"{path}: {exc}"
                ) from exc
            if (
                not stat.S_ISREG(current.st_mode)
                or current.st_nlink != 1
                or (current.st_dev, current.st_ino) != identity
                or (observed.st_dev, observed.st_ino) != identity
            ):
                raise GateBlocked(
                    "lock-path-changed-while-held", str(path)
                )
        except BaseException as original:
            if on_error is not None:
                try:
                    on_error()
                except BaseException as cleanup_error:
                    raise GateBlocked(
                        "post-admission-cleanup-failed",
                        f"{type(cleanup_error).__name__}:"
                        f"{cleanup_error}",
                    ) from original
            raise
    finally:
        if descriptor is not None:
            try:
                fcntl.flock(descriptor, fcntl.LOCK_UN)
            finally:
                os.close(descriptor)


@contextmanager
def already_held_lock(wait_seconds: float) -> Iterator[float]:
    yield wait_seconds


@contextmanager
def cargo_registry_locks(
    admitted_root: Path, timeout_seconds: float
) -> Iterator[tuple[float, float]]:
    with exclusive_lock(
        admitted_root / GLOBAL_EXECUTION_LOCK_NAME, timeout_seconds
    ) as global_wait:
        with exclusive_lock(
            admitted_root / REGISTRY_LOCK_NAME, timeout_seconds
        ) as registry_wait:
            yield global_wait, registry_wait


def marker_payload(
    source_key: str,
    source_key_payload: dict[str, Any],
    target: Path,
) -> dict[str, Any]:
    return {
        "schema": MARKER_SCHEMA,
        "source_key": source_key,
        "source_key_payload": source_key_payload,
        "target_namespace": str(target),
    }


def marker_bytes(payload: dict[str, Any]) -> bytes:
    return canonical_json_bytes(payload) + b"\n"


def write_marker(path: Path, payload: dict[str, Any]) -> None:
    temporary = path.with_name(
        f".{path.name}.tmp-{os.getpid()}-{time.monotonic_ns()}"
    )
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC
    descriptor = os.open(temporary, flags, 0o600)
    try:
        with os.fdopen(descriptor, "wb", closefd=False) as handle:
            handle.write(marker_bytes(payload))
            handle.flush()
            os.fsync(handle.fileno())
    finally:
        os.close(descriptor)
    os.replace(temporary, path)


def validate_marker(path: Path, expected: dict[str, Any]) -> str:
    parent = os.open(
        path.parent,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    try:
        return validate_marker_at(parent, path.name, expected, display=path)
    finally:
        os.close(parent)


def read_regular_at(
    directory: int,
    name: str,
    *,
    display: Path,
    maximum_bytes: int = 1024 * 1024,
) -> bytes:
    try:
        descriptor = os.open(
            name,
            os.O_RDONLY | os.O_CLOEXEC | getattr(os, "O_NOFOLLOW", 0),
            dir_fd=directory,
        )
    except OSError as exc:
        raise GateBlocked("source-marker-invalid", f"{display}: {exc}") from exc
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
            raise GateBlocked(
                "source-marker-invalid",
                f"{display} must be a single-link regular file",
            )
        chunks: list[bytes] = []
        size = 0
        while True:
            chunk = os.read(descriptor, min(64 * 1024, maximum_bytes + 1 - size))
            if not chunk:
                break
            chunks.append(chunk)
            size += len(chunk)
            if size > maximum_bytes:
                raise GateBlocked(
                    "source-marker-invalid", f"{display} exceeds size bound"
                )
        return b"".join(chunks)
    finally:
        os.close(descriptor)


def validate_marker_at(
    directory: int,
    name: str,
    expected: dict[str, Any],
    *,
    display: Path,
) -> str:
    raw = read_regular_at(directory, name, display=display)
    try:
        observed = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise GateBlocked("source-marker-invalid", f"{display}: {exc}") from exc
    if observed != expected or raw != marker_bytes(expected):
        raise GateBlocked(
            "source-marker-mismatch",
            f"target namespace is not bound to source key {expected['source_key']}",
        )
    return hashlib.sha256(raw).hexdigest()


def open_directory_no_follow(path: Path) -> int:
    descriptor = os.open(
        path,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    opened = os.fstat(descriptor)
    observed = os.stat(path, follow_symlinks=False)
    if (
        not stat.S_ISDIR(observed.st_mode)
        or (opened.st_dev, opened.st_ino) != (observed.st_dev, observed.st_ino)
    ):
        os.close(descriptor)
        raise GateBlocked("directory-identity-mismatch", str(path))
    return descriptor


def prepare_runtime_scratch(target: Path) -> dict[str, Any]:
    parent = open_directory_no_follow(target)
    scratch: int | None = None
    try:
        parent_metadata = os.fstat(parent)
        if (
            parent_metadata.st_uid != os.geteuid()
            or parent_metadata.st_mode & 0o022
        ):
            raise GateBlocked(
                "unsafe-runtime-scratch-parent", str(target)
            )
        try:
            os.mkdir(RUNTIME_SCRATCH_NAME, mode=0o700, dir_fd=parent)
            os.fsync(parent)
        except FileExistsError as exc:
            raise GateBlocked(
                "runtime-scratch-already-exists",
                str(target / RUNTIME_SCRATCH_NAME),
            ) from exc
        scratch = os.open(
            RUNTIME_SCRATCH_NAME,
            os.O_RDONLY
            | os.O_DIRECTORY
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            dir_fd=parent,
        )
        scratch_metadata = os.fstat(scratch)
        observed = os.stat(
            RUNTIME_SCRATCH_NAME,
            dir_fd=parent,
            follow_symlinks=False,
        )
        if (
            not stat.S_ISDIR(scratch_metadata.st_mode)
            or scratch_metadata.st_uid != os.geteuid()
            or scratch_metadata.st_mode & 0o077
            or (scratch_metadata.st_dev, scratch_metadata.st_ino)
            != (observed.st_dev, observed.st_ino)
        ):
            raise GateBlocked(
                "unsafe-runtime-scratch",
                str(target / RUNTIME_SCRATCH_NAME),
            )
        children: dict[str, dict[str, Any]] = {}
        for name in (RUNTIME_HOME_NAME, RUNTIME_TMP_NAME):
            os.mkdir(name, mode=0o700, dir_fd=scratch)
            child = os.open(
                name,
                os.O_RDONLY
                | os.O_DIRECTORY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
                dir_fd=scratch,
            )
            try:
                metadata = os.fstat(child)
                path_metadata = os.stat(
                    name, dir_fd=scratch, follow_symlinks=False
                )
                if (
                    not stat.S_ISDIR(metadata.st_mode)
                    or metadata.st_uid != os.geteuid()
                    or metadata.st_mode & 0o077
                    or (metadata.st_dev, metadata.st_ino)
                    != (path_metadata.st_dev, path_metadata.st_ino)
                ):
                    raise GateBlocked(
                        "unsafe-runtime-scratch-child",
                        str(target / RUNTIME_SCRATCH_NAME / name),
                    )
                os.fsync(child)
                children[name] = {
                    "path": str(
                        target / RUNTIME_SCRATCH_NAME / name
                    ),
                    "device": metadata.st_dev,
                    "inode": metadata.st_ino,
                    "mode": stat.S_IMODE(metadata.st_mode),
                    "uid": metadata.st_uid,
                }
            finally:
                os.close(child)
        os.fsync(scratch)
        return {
            "schema": "codefire.cargo_gate_runtime_scratch.v1",
            "root": str(target / RUNTIME_SCRATCH_NAME),
            "root_device": scratch_metadata.st_dev,
            "root_inode": scratch_metadata.st_ino,
            "home": children[RUNTIME_HOME_NAME],
            "tmp": children[RUNTIME_TMP_NAME],
        }
    except BaseException:
        if scratch is not None:
            try:
                remove_tree_contents_at(scratch)
            except BaseException:
                pass
            try:
                os.rmdir(RUNTIME_SCRATCH_NAME, dir_fd=parent)
                os.fsync(parent)
            except BaseException:
                pass
        raise
    finally:
        if scratch is not None:
            os.close(scratch)
        os.close(parent)


def cleanup_runtime_scratch(
    target: Path, expected: Mapping[str, Any]
) -> dict[str, Any]:
    parent = open_directory_no_follow(target)
    scratch: int | None = None
    try:
        observed = os.stat(
            RUNTIME_SCRATCH_NAME,
            dir_fd=parent,
            follow_symlinks=False,
        )
        if (
            not stat.S_ISDIR(observed.st_mode)
            or (observed.st_dev, observed.st_ino)
            != (
                int(expected["root_device"]),
                int(expected["root_inode"]),
            )
        ):
            raise GateBlocked(
                "runtime-scratch-identity-mismatch",
                str(target / RUNTIME_SCRATCH_NAME),
            )
        scratch = os.open(
            RUNTIME_SCRATCH_NAME,
            os.O_RDONLY
            | os.O_DIRECTORY
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            dir_fd=parent,
        )
        opened = os.fstat(scratch)
        if (opened.st_dev, opened.st_ino) != (
            observed.st_dev,
            observed.st_ino,
        ):
            raise GateBlocked(
                "runtime-scratch-identity-mismatch",
                str(target / RUNTIME_SCRATCH_NAME),
            )
        remove_tree_contents_at(scratch)
        os.close(scratch)
        scratch = None
        current = os.stat(
            RUNTIME_SCRATCH_NAME,
            dir_fd=parent,
            follow_symlinks=False,
        )
        if (current.st_dev, current.st_ino) != (
            observed.st_dev,
            observed.st_ino,
        ):
            raise GateBlocked(
                "runtime-scratch-changed-before-remove",
                str(target / RUNTIME_SCRATCH_NAME),
            )
        os.rmdir(RUNTIME_SCRATCH_NAME, dir_fd=parent)
        os.fsync(parent)
        try:
            os.stat(
                RUNTIME_SCRATCH_NAME,
                dir_fd=parent,
                follow_symlinks=False,
            )
        except FileNotFoundError:
            pass
        else:
            raise GateBlocked(
                "runtime-scratch-removal-incomplete",
                str(target / RUNTIME_SCRATCH_NAME),
            )
        return {
            "decision": "REMOVED",
            "path": str(target / RUNTIME_SCRATCH_NAME),
            "root_device": observed.st_dev,
            "root_inode": observed.st_ino,
        }
    finally:
        if scratch is not None:
            os.close(scratch)
        os.close(parent)


def reopen_directory_at(
    root: int,
    relative: str,
    *,
    code: str,
) -> int:
    current = os.dup(root)
    try:
        for part in relative.split("/") if relative else ():
            if part in {"", ".", ".."} or "/" in part:
                raise GateBlocked(code, relative)
            child = os.open(
                part,
                os.O_RDONLY
                | os.O_DIRECTORY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
                dir_fd=current,
            )
            metadata = os.fstat(child)
            if not stat.S_ISDIR(metadata.st_mode):
                os.close(child)
                raise GateBlocked(code, relative)
            os.close(current)
            current = child
        return current
    except BaseException:
        os.close(current)
        raise


def bounded_directory_names_at(
    directory: int,
    *,
    deadline: float,
    code: str,
    predicate: Callable[[str], bool] | None = None,
    maximum_entries: int = MAX_ROOT_DIRECTORY_ENTRIES,
) -> list[str]:
    selected: list[str] = []
    count = 0
    with os.scandir(directory) as scanner:
        for entry in scanner:
            count += 1
            if count > maximum_entries:
                raise GateBlocked(code, f"maximum={maximum_entries}")
            if time.monotonic() > deadline:
                raise GateBlocked(code, "deadline exceeded")
            if predicate is None or predicate(entry.name):
                selected.append(entry.name)
    selected.sort()
    return selected


def ordered_name_digest(names: list[str]) -> str:
    digest = hashlib.sha256()
    digest.update(b"[")
    for index, name in enumerate(names):
        if index:
            digest.update(b",")
        digest.update(
            json.dumps(
                name,
                ensure_ascii=False,
                separators=(",", ":"),
            ).encode("utf-8")
        )
    digest.update(b"]")
    return digest.hexdigest()


def ordered_name_summary(names: list[str]) -> str:
    return (
        f"count={len(names)}, "
        f"sha256={ordered_name_digest(names)}, "
        f"sample={names[:8]!r}"
    )


def namespace_usage_at(
    directory: int,
    *,
    traversal: dict[str, Any] | None = None,
) -> dict[str, int]:
    if traversal is None:
        traversal = {
            "entries": 0,
            "maximum_entries": MAX_NAMESPACE_ENTRY_COUNT,
            "deadline": None,
        }
    allocated = 0
    logical = 0
    entries = 0
    root_identity = os.fstat(directory)
    pending: list[tuple[str, int, int]] = [
        ("", root_identity.st_dev, root_identity.st_ino)
    ]
    while pending:
        relative, expected_device, expected_inode = pending.pop()
        current = reopen_directory_at(
            directory,
            relative,
            code="namespace-entry-identity-mismatch",
        )
        try:
            opened = os.fstat(current)
            if (opened.st_dev, opened.st_ino) != (
                expected_device,
                expected_inode,
            ):
                raise GateBlocked(
                    "namespace-entry-identity-mismatch",
                    relative or ".",
                )
            with os.scandir(current) as scanner:
                for entry in scanner:
                    name = entry.name
                    traversal["entries"] += 1
                    if (
                        traversal["entries"]
                        > traversal["maximum_entries"]
                    ):
                        raise GateBlocked(
                            "namespace-entry-count-limit-exceeded",
                            str(traversal["maximum_entries"]),
                        )
                    deadline = traversal.get("deadline")
                    if (
                        deadline is not None
                        and time.monotonic() > float(deadline)
                    ):
                        raise GateBlocked(
                            "live-namespace-scan-deadline-exceeded",
                            str(LIVE_NAMESPACE_SCAN_SECONDS),
                        )
                    metadata = os.stat(
                        name,
                        dir_fd=current,
                        follow_symlinks=False,
                    )
                    entries += 1
                    allocated += metadata.st_blocks * 512
                    logical += metadata.st_size
                    if stat.S_ISDIR(metadata.st_mode):
                        child_relative = (
                            f"{relative}/{name}"
                            if relative
                            else name
                        )
                        pending.append(
                            (
                                child_relative,
                                metadata.st_dev,
                                metadata.st_ino,
                            )
                        )
        finally:
            os.close(current)
    return {
        "allocated_bytes": allocated,
        "logical_bytes": logical,
        "entry_count": entries,
    }


def inspect_registry(
    registry: Path,
    *,
    measure_usage: bool = True,
) -> list[dict[str, Any]]:
    inventory: list[dict[str, Any]] = []
    if not registry.exists():
        return inventory
    if registry.is_symlink() or not registry.is_dir():
        raise GateBlocked("unsafe-source-key-registry", str(registry))
    registry_descriptor = open_directory_no_follow(registry)
    deadline = time.monotonic() + REGISTRY_SCAN_SECONDS
    traversal = {
        "entries": 0,
        "maximum_entries": MAX_NAMESPACE_ENTRY_COUNT,
        "deadline": deadline,
    }
    try:
        namespace_names = bounded_directory_names_at(
            registry_descriptor,
            deadline=deadline,
            code="registry-root-scan-limit-exceeded",
            maximum_entries=MAX_REGISTRY_ROOT_ENTRIES,
        )
    finally:
        os.close(registry_descriptor)
    for namespace_name in namespace_names:
        namespace = registry / namespace_name
        if (
            re.fullmatch(r"[0-9a-f]{64}", namespace.name) is None
            or namespace.is_symlink()
            or not namespace.is_dir()
        ):
            raise GateBlocked(
                "unknown-source-key-registry-entry", str(namespace)
            )
        namespace_descriptor = open_directory_no_follow(namespace)
        marker = namespace / MARKER_NAME
        try:
            try:
                raw = read_regular_at(
                    namespace_descriptor,
                    MARKER_NAME,
                    display=marker,
                )
                payload = json.loads(raw)
            except (json.JSONDecodeError, OSError) as exc:
                raise GateBlocked(
                    "source-marker-invalid", f"{marker}: {exc}"
                ) from exc
            if (
                not isinstance(payload, dict)
                or payload.get("schema") != MARKER_SCHEMA
                or payload.get("source_key") != namespace.name
                or payload.get("target_namespace") != str(namespace)
                or raw != marker_bytes(payload)
            ):
                raise GateBlocked(
                    "source-marker-registry-mismatch", str(marker)
                )
            retention = namespace / RETENTION_NAME
            last_used_at: str | None = None
            try:
                os.stat(
                    RETENTION_NAME,
                    dir_fd=namespace_descriptor,
                    follow_symlinks=False,
                )
            except FileNotFoundError:
                retained = None
            else:
                try:
                    retained_raw = read_regular_at(
                        namespace_descriptor,
                        RETENTION_NAME,
                        display=retention,
                    )
                    retained = json.loads(retained_raw)
                except (json.JSONDecodeError, OSError) as exc:
                    raise GateBlocked(
                        "retention-record-invalid", f"{retention}: {exc}"
                    ) from exc
            if retained is not None:
                if (
                    not isinstance(retained, dict)
                    or retained.get("schema")
                    != "codefire.cargo_namespace_retention.v1"
                    or retained.get("source_key") != namespace.name
                    or not isinstance(retained.get("last_used_at"), str)
                    or not isinstance(retained.get("allocated_bytes"), int)
                    or retained["allocated_bytes"] < 0
                    or retained_raw != marker_bytes(retained)
                ):
                    raise GateBlocked(
                        "retention-record-mismatch", str(retention)
                    )
                last_used_at = retained.get("last_used_at")
            invalidated = False
            try:
                os.stat(
                    INVALIDATION_NAME,
                    dir_fd=namespace_descriptor,
                    follow_symlinks=False,
                )
            except FileNotFoundError:
                pass
            else:
                invalidation_raw = read_regular_at(
                    namespace_descriptor,
                    INVALIDATION_NAME,
                    display=namespace / INVALIDATION_NAME,
                )
                try:
                    invalidation = json.loads(invalidation_raw)
                except json.JSONDecodeError as exc:
                    raise GateBlocked(
                        "namespace-invalidation-invalid",
                        f"{namespace / INVALIDATION_NAME}: {exc}",
                    ) from exc
                if (
                    not isinstance(invalidation, dict)
                    or invalidation.get("schema")
                    != "codefire.cargo_namespace_invalidation.v1"
                    or invalidation.get("source_key") != namespace.name
                    or invalidation_raw != marker_bytes(invalidation)
                ):
                    raise GateBlocked(
                        "namespace-invalidation-mismatch",
                        str(namespace / INVALIDATION_NAME),
                    )
                invalidated = True
            usage = (
                namespace_usage_at(
                    namespace_descriptor, traversal=traversal
                )
                if measure_usage
                else None
            )
        finally:
            os.close(namespace_descriptor)
        inventory.append(
            {
                "source_key": namespace.name,
                "last_used_at": last_used_at,
                "actual_allocated_bytes": (
                    usage["allocated_bytes"]
                    if usage is not None
                    else None
                ),
                "actual_logical_bytes": (
                    usage["logical_bytes"]
                    if usage is not None
                    else None
                ),
                "actual_entry_count": (
                    usage["entry_count"]
                    if usage is not None
                    else None
                ),
                "usage_measured": usage is not None,
                "invalidated": invalidated,
            }
        )
    return inventory


def namespace_usage(
    path: Path,
    *,
    live_deadline_seconds: float | None = None,
) -> dict[str, int]:
    descriptor = open_directory_no_follow(path)
    try:
        traversal = {
            "entries": 0,
            "maximum_entries": MAX_NAMESPACE_ENTRY_COUNT,
            "deadline": (
                time.monotonic() + live_deadline_seconds
                if live_deadline_seconds is not None
                else None
            ),
        }
        return namespace_usage_at(descriptor, traversal=traversal)
    finally:
        os.close(descriptor)


def utc_now() -> str:
    return (
        datetime.now(timezone.utc)
        .isoformat(timespec="microseconds")
        .replace("+00:00", "Z")
    )


def write_retention_record(
    target: Path,
    *,
    source_key: str,
    usage: dict[str, int],
) -> str:
    payload = {
        "schema": "codefire.cargo_namespace_retention.v1",
        "source_key": source_key,
        "target_namespace": str(target),
        "last_used_at": utc_now(),
        **usage,
    }
    raw = marker_bytes(payload)
    write_namespace_record(
        target,
        RETENTION_NAME,
        raw,
        replace=True,
    )
    return hashlib.sha256(raw).hexdigest()


def run_state_payload(
    source_key: str,
    state: str,
    execution_key_value: str | None,
) -> dict[str, Any]:
    if state not in {"PREPARED", "RUNNING", "COMMITTED"}:
        raise GateBlocked("invalid-run-state", state)
    return {
        "schema": "codefire.cargo_namespace_run_state.v1",
        "source_key": source_key,
        "state": state,
        "execution_key": execution_key_value,
    }


def write_run_state(
    target: Path,
    *,
    source_key: str,
    state: str,
    execution_key_value: str | None,
) -> str:
    payload = run_state_payload(
        source_key, state, execution_key_value
    )
    raw = marker_bytes(payload)
    write_namespace_record(
        target,
        RUN_STATE_NAME,
        raw,
        replace=True,
    )
    return hashlib.sha256(raw).hexdigest()


def read_run_state(
    target: Path,
    *,
    source_key: str,
) -> dict[str, Any] | None:
    directory = open_directory_no_follow(target)
    try:
        try:
            observed = os.stat(
                RUN_STATE_NAME,
                dir_fd=directory,
                follow_symlinks=False,
            )
        except FileNotFoundError:
            return None
        if not stat.S_ISREG(observed.st_mode) or observed.st_nlink != 1:
            raise GateBlocked(
                "namespace-run-state-invalid", str(target / RUN_STATE_NAME)
            )
        raw = read_regular_at(
            directory,
            RUN_STATE_NAME,
            display=target / RUN_STATE_NAME,
        )
    finally:
        os.close(directory)
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise GateBlocked(
            "namespace-run-state-invalid", str(target / RUN_STATE_NAME)
        ) from exc
    if (
        not isinstance(payload, dict)
        or payload.get("schema")
        != "codefire.cargo_namespace_run_state.v1"
        or payload.get("source_key") != source_key
        or payload.get("state")
        not in {"PREPARED", "RUNNING", "COMMITTED"}
        or (
            payload.get("execution_key") is not None
            and re.fullmatch(
                r"[0-9a-f]{64}",
                str(payload.get("execution_key")),
            )
            is None
        )
        or raw != marker_bytes(payload)
    ):
        raise GateBlocked(
            "namespace-run-state-invalid", str(target / RUN_STATE_NAME)
        )
    return payload


def write_all(descriptor: int, raw: bytes) -> None:
    offset = 0
    while offset < len(raw):
        written = os.write(descriptor, raw[offset:])
        if written < 1:
            raise OSError("short write")
        offset += written


def open_directory_path_at(
    root: int,
    parts: tuple[str, ...],
    *,
    create: bool,
) -> int:
    current = os.dup(root)
    try:
        for part in parts:
            if part in {"", ".", ".."} or "/" in part:
                raise GateBlocked("unsafe-artifact-path", repr(parts))
            if create:
                try:
                    os.mkdir(part, mode=0o700, dir_fd=current)
                    os.fsync(current)
                except FileExistsError:
                    pass
            child = os.open(
                part,
                os.O_RDONLY
                | os.O_DIRECTORY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
                dir_fd=current,
            )
            metadata = os.fstat(child)
            if (
                not stat.S_ISDIR(metadata.st_mode)
                or metadata.st_uid != os.geteuid()
                or metadata.st_mode & 0o022
            ):
                os.close(child)
                raise GateBlocked(
                    "unsafe-artifact-directory", "/".join(parts)
                )
            os.close(current)
            current = child
        return current
    except BaseException:
        os.close(current)
        raise


def artifact_export_destination_contract(
    candidate_root: Path,
    destination: Path,
) -> dict[str, Any]:
    lexical = destination.absolute()
    try:
        relative = lexical.relative_to(candidate_root)
    except ValueError as exc:
        raise GateBlocked(
            "artifact-export-outside-candidate", str(lexical)
        ) from exc
    if (
        len(relative.parts) < 2
        or relative.parts[0] != "target"
        or any(part in {"", ".", ".."} for part in relative.parts)
    ):
        raise GateBlocked(
            "artifact-export-outside-candidate-target", str(lexical)
        )
    relative_text = relative.as_posix()
    tracked = git_bytes(
        candidate_root,
        "ls-files",
        "-z",
        "--cached",
        "--",
        relative_text,
        maximum_bytes=1024 * 1024,
    )
    if tracked:
        raise GateBlocked(
            "artifact-export-destination-tracked", relative_text
        )
    returncode, ignored, stderr = git_status_bytes(
        candidate_root,
        "check-ignore",
        "--no-index",
        "--",
        relative_text,
        maximum_bytes=1024 * 1024,
    )
    expected = relative_text.encode("utf-8") + b"\n"
    if returncode == 1 and not ignored:
        raise GateBlocked(
            "artifact-export-destination-not-ignored", relative_text
        )
    if returncode != 0 or ignored != expected:
        raise GateBlocked(
            "artifact-export-ignore-check-failed",
            f"path={relative_text}, exit={returncode}, "
            f"stdout_sha256={hashlib.sha256(ignored).hexdigest()}, "
            f"stderr_sha256={hashlib.sha256(stderr).hexdigest()}",
        )
    return {
        "decision": "UNTRACKED_IGNORED_CANDIDATE_TARGET",
        "relative_path": relative_text,
        "tracked": False,
        "ignored": True,
        "git_control_sha256": sha256_json(
            {
                "config_args": list(GIT_CONTROL_CONFIG_ARGS),
                "environment": git_control_environment(),
            }
        ),
    }


def recover_artifact_export_residue(
    destination_parent: int,
    destination_name: str,
    *,
    deadline: float,
) -> dict[str, Any]:
    temporary_prefix = f".{destination_name}.tmp-"
    previous_prefix = f".{destination_name}.previous-"
    residue_names = bounded_directory_names_at(
        destination_parent,
        deadline=deadline,
        code="artifact-export-parent-scan-limit-exceeded",
        predicate=lambda name: name.startswith(
            (temporary_prefix, previous_prefix)
        ),
    )
    if len(residue_names) > MAX_ARTIFACT_EXPORT_RESIDUE_ENTRIES:
        raise GateBlocked(
            "artifact-export-residue-count-exceeded",
            ordered_name_summary(residue_names),
        )
    temporary_names = [
        name
        for name in residue_names
        if name.startswith(temporary_prefix)
    ]
    previous_names = [
        name
        for name in residue_names
        if name.startswith(previous_prefix)
    ]
    residue_metadata: dict[str, os.stat_result] = {}
    for name in [*temporary_names, *previous_names]:
        metadata = os.stat(
            name,
            dir_fd=destination_parent,
            follow_symlinks=False,
        )
        if (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_uid != os.geteuid()
            or metadata.st_nlink not in {1, 2}
        ):
            raise GateBlocked(
                "unsafe-artifact-export-residue", name
            )
        residue_metadata[name] = metadata
    changed = False
    for name in temporary_names:
        os.unlink(name, dir_fd=destination_parent)
        changed = True
    try:
        canonical = os.stat(
            destination_name,
            dir_fd=destination_parent,
            follow_symlinks=False,
        )
    except FileNotFoundError:
        canonical = None
    if canonical is not None:
        if (
            not stat.S_ISREG(canonical.st_mode)
            or canonical.st_uid != os.geteuid()
            or canonical.st_nlink not in {1, 2}
        ):
            raise GateBlocked(
                "unsafe-prior-artifact-export", destination_name
            )
        if canonical.st_nlink == 2:
            if len(previous_names) != 1:
                raise GateBlocked(
                    "ambiguous-artifact-export-recovery",
                    destination_name,
                )
            previous = residue_metadata[previous_names[0]]
            if (previous.st_dev, previous.st_ino) != (
                canonical.st_dev,
                canonical.st_ino,
            ):
                raise GateBlocked(
                    "artifact-export-residue-identity-mismatch",
                    destination_name,
                )
    if canonical is None and len(previous_names) > 1:
        raise GateBlocked(
            "ambiguous-artifact-export-recovery",
            destination_name,
        )
    if canonical is None and previous_names:
        os.rename(
            previous_names[0],
            destination_name,
            src_dir_fd=destination_parent,
            dst_dir_fd=destination_parent,
        )
        changed = True
    else:
        for name in previous_names:
            os.unlink(name, dir_fd=destination_parent)
            changed = True
    if changed:
        os.fsync(destination_parent)
    return {
        "decision": "ARTIFACT_EXPORT_RESIDUE_RECOVERED",
        "temporary_removed": len(temporary_names),
        "previous_recovered_or_removed": len(previous_names),
        "residue_names_sha256": ordered_name_digest(
            [*temporary_names, *previous_names]
        ),
    }


def exported_source_identity(
    metadata: os.stat_result,
) -> tuple[int, ...]:
    """The stat fields an export source must hold still across the copy."""
    return (
        metadata.st_dev,
        metadata.st_ino,
        metadata.st_mode,
        metadata.st_nlink,
        metadata.st_size,
        metadata.st_mtime_ns,
        metadata.st_ctime_ns,
    )


def export_bound_artifact(
    target: Path,
    artifact_relative: str,
    candidate_root: Path,
    destination: Path,
    *,
    free_floor_bytes: int,
) -> dict[str, Any]:
    export_deadline = (
        time.monotonic() + ARTIFACT_EXPORT_TIMEOUT_SECONDS
    )
    relative = Path(artifact_relative)
    if (
        relative.is_absolute()
        or not relative.parts
        or any(part in {"", ".", ".."} for part in relative.parts)
        or len(relative.parts) > 16
    ):
        raise GateBlocked("unsafe-artifact-path", artifact_relative)
    destination_lexical = destination.absolute()
    try:
        destination_relative = destination_lexical.relative_to(
            candidate_root
        )
    except ValueError as exc:
        raise GateBlocked(
            "artifact-export-outside-candidate", str(destination_lexical)
        ) from exc
    if (
        len(destination_relative.parts) < 2
        or destination_relative.parts[0] != "target"
        or destination_relative.name in {"", ".", ".."}
    ):
        raise GateBlocked(
            "artifact-export-outside-candidate-target",
            str(destination_lexical),
        )

    target_descriptor = open_directory_no_follow(target)
    source_parent: int | None = None
    source_descriptor: int | None = None
    candidate_descriptor: int | None = None
    destination_parent: int | None = None
    destination_descriptor: int | None = None
    temporary = (
        f".{destination_relative.name}.tmp-{os.getpid()}-"
        f"{time.monotonic_ns()}"
    )
    backup = (
        f".{destination_relative.name}.previous-{os.getpid()}-"
        f"{time.monotonic_ns()}"
    )
    destination_metadata: os.stat_result | None = None
    prior_metadata: os.stat_result | None = None
    backup_created = False
    published = False
    publication_committed = False
    backup_cleanup_durable = True
    residue_recovery: dict[str, Any] | None = None
    try:
        source_parent = open_directory_path_at(
            target_descriptor,
            relative.parts[:-1],
            create=False,
        )
        source_descriptor = os.open(
            relative.name,
            os.O_RDONLY
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            dir_fd=source_parent,
        )
        source_before = os.fstat(source_descriptor)
        if (
            not stat.S_ISREG(source_before.st_mode)
            or source_before.st_nlink < 1
            or source_before.st_nlink > MAX_EXPORT_SOURCE_ARTIFACT_LINKS
            or not source_before.st_mode & stat.S_IXUSR
        ):
            raise GateBlocked(
                "artifact-not-private-executable",
                f"{artifact_relative}: "
                f"regular={stat.S_ISREG(source_before.st_mode)}, "
                f"mode=0o{stat.S_IMODE(source_before.st_mode):04o}, "
                f"nlink={source_before.st_nlink}, "
                f"nlink_maximum={MAX_EXPORT_SOURCE_ARTIFACT_LINKS}",
            )
        if (
            source_before.st_size
            > MAX_EXPORTED_ARTIFACT_LOGICAL_BYTES
        ):
            raise GateBlocked(
                "artifact-export-logical-bytes-exceeded",
                f"bytes={source_before.st_size}, "
                f"maximum={MAX_EXPORTED_ARTIFACT_LOGICAL_BYTES}",
            )
        free_before_export = shutil.disk_usage(candidate_root).free
        if free_before_export - source_before.st_size < free_floor_bytes:
            raise GateBlocked(
                "insufficient-space-for-artifact-export",
                f"free={free_before_export}, "
                f"artifact={source_before.st_size}, "
                f"floor={free_floor_bytes}",
            )
        candidate_descriptor = open_directory_no_follow(candidate_root)
        destination_parent = open_directory_path_at(
            candidate_descriptor,
            destination_relative.parts[:-1],
            create=True,
        )
        residue_recovery = recover_artifact_export_residue(
            destination_parent,
            destination_relative.name,
            deadline=export_deadline,
        )
        destination_descriptor = os.open(
            temporary,
            os.O_WRONLY
            | os.O_CREAT
            | os.O_EXCL
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            0o700,
            dir_fd=destination_parent,
        )
        digest = hashlib.sha256()
        size = 0
        last_resource_check = 0
        staged_allocated_bytes = 0
        free_during_staging = free_before_export
        while True:
            if time.monotonic() > export_deadline:
                raise GateBlocked(
                    "artifact-export-deadline-exceeded",
                    str(ARTIFACT_EXPORT_TIMEOUT_SECONDS),
                )
            chunk = os.read(source_descriptor, 1024 * 1024)
            if not chunk:
                break
            if (
                size + len(chunk)
                > MAX_EXPORTED_ARTIFACT_LOGICAL_BYTES
            ):
                raise GateBlocked(
                    "artifact-export-logical-bytes-exceeded",
                    f"bytes>{MAX_EXPORTED_ARTIFACT_LOGICAL_BYTES}",
                )
            write_all(destination_descriptor, chunk)
            digest.update(chunk)
            size += len(chunk)
            if (
                size - last_resource_check
                >= ARTIFACT_EXPORT_RECHECK_BYTES
            ):
                staged = os.fstat(destination_descriptor)
                staged_allocated_bytes = staged.st_blocks * 512
                if (
                    staged_allocated_bytes
                    > MAX_EXPORTED_ARTIFACT_LOGICAL_BYTES
                ):
                    raise GateBlocked(
                        "artifact-export-allocated-bytes-exceeded",
                        f"allocated={staged_allocated_bytes}, "
                        f"maximum="
                        f"{MAX_EXPORTED_ARTIFACT_LOGICAL_BYTES}",
                    )
                free_during_staging = shutil.disk_usage(
                    candidate_root
                ).free
                if free_during_staging < free_floor_bytes:
                    raise GateBlocked(
                        "artifact-export-free-floor-crossed",
                        f"free={free_during_staging}, "
                        f"floor={free_floor_bytes}",
                    )
                last_resource_check = size
        source_after = os.fstat(source_descriptor)
        if (
            exported_source_identity(source_before)
            != exported_source_identity(source_after)
            or size != source_after.st_size
        ):
            raise GateBlocked(
                "artifact-changed-during-export", artifact_relative
            )
        # The stat tuple above is necessary and not sufficient. A writer
        # holding either link can overwrite the source mid-copy and restore
        # its mtime, and this filesystem advances ctime only about every
        # 3.8 ms: on attempt 75 of a 3 MiB two-link source, `source_before`
        # and `source_after` were identical across a mutation that had
        # already torn the staged copy into an A-prefix/B-suffix file equal
        # to neither the original nor the final source. The same hole is
        # reachable through a single link's own path, so the link bound is
        # not what closes it.
        #
        # Re-read the whole source through the same pinned descriptor and
        # require it to hash to exactly what was staged. Be exact about
        # what that establishes, because the tempting stronger reading is
        # false.
        #
        # It establishes: the staged byte sequence equals one complete
        # re-read of the source taken immediately after the copy, while
        # the before, after, and re-read stat identities all agree. So the
        # export cannot publish a view the gate never read as a whole --
        # the torn A-prefix/B-suffix file above equals no single complete
        # read. This is content equality, not a finer clock, so it does not
        # depend on winning a timestamp tick.
        #
        # It does not establish that every mutation before or during the
        # copy is detected. A writer that installs a complete, stable
        # replacement early enough for both reads to observe it identically
        # passes here, and the export then publishes that replacement
        # rather than what the source held when the gate started. Both
        # reads are also lock-free: nothing here excludes a concurrent
        # writer or proves the source was quiescent, so a writer that can
        # time both reads, or write the staged sequence back before the
        # re-read, is not excluded either. Each of those needs the same-UID
        # actor the contract's boundary section already places outside this
        # gate; none of them is closed by this check, and no comment or
        # contract statement may say otherwise.
        staged_sha256 = digest.hexdigest()
        os.lseek(source_descriptor, 0, os.SEEK_SET)
        reread_digest = hashlib.sha256()
        reread_bytes = 0
        while True:
            if time.monotonic() > export_deadline:
                raise GateBlocked(
                    "artifact-export-deadline-exceeded",
                    str(ARTIFACT_EXPORT_TIMEOUT_SECONDS),
                )
            chunk = os.read(source_descriptor, 1024 * 1024)
            if not chunk:
                break
            reread_bytes += len(chunk)
            if reread_bytes > MAX_EXPORTED_ARTIFACT_LOGICAL_BYTES:
                raise GateBlocked(
                    "artifact-export-logical-bytes-exceeded",
                    f"bytes>{MAX_EXPORTED_ARTIFACT_LOGICAL_BYTES}",
                )
            reread_digest.update(chunk)
        source_reread = os.fstat(source_descriptor)
        source_reread_sha256 = reread_digest.hexdigest()
        if (
            reread_bytes != size
            or source_reread_sha256 != staged_sha256
            or exported_source_identity(source_reread)
            != exported_source_identity(source_before)
            or exported_source_identity(source_reread)
            != exported_source_identity(source_after)
        ):
            raise GateBlocked(
                "artifact-changed-during-export",
                f"{artifact_relative}: staged_bytes={size}, "
                f"reread_bytes={reread_bytes}, "
                f"staged_sha256={staged_sha256}, "
                f"reread_sha256={source_reread_sha256}",
            )
        os.fchmod(destination_descriptor, 0o700)
        if time.monotonic() > export_deadline:
            raise GateBlocked(
                "artifact-export-deadline-exceeded",
                str(ARTIFACT_EXPORT_TIMEOUT_SECONDS),
            )
        os.fsync(destination_descriptor)
        destination_metadata = os.fstat(destination_descriptor)
        staged_allocated_bytes = destination_metadata.st_blocks * 512
        if (
            staged_allocated_bytes
            > MAX_EXPORTED_ARTIFACT_LOGICAL_BYTES
        ):
            raise GateBlocked(
                "artifact-export-allocated-bytes-exceeded",
                f"allocated={staged_allocated_bytes}, "
                f"maximum={MAX_EXPORTED_ARTIFACT_LOGICAL_BYTES}",
            )
        free_after_staging = shutil.disk_usage(candidate_root).free
        if free_after_staging < free_floor_bytes:
            raise GateBlocked(
                "artifact-export-free-floor-crossed",
                f"free={free_after_staging}, floor={free_floor_bytes}",
            )
        os.close(destination_descriptor)
        destination_descriptor = None
        candidate_opened = os.fstat(candidate_descriptor)
        candidate_observed = os.stat(
            candidate_root, follow_symlinks=False
        )
        if (candidate_opened.st_dev, candidate_opened.st_ino) != (
            candidate_observed.st_dev,
            candidate_observed.st_ino,
        ):
            raise GateBlocked(
                "artifact-export-candidate-root-changed",
                str(candidate_root),
            )
        reopened_parent = open_directory_path_at(
            candidate_descriptor,
            destination_relative.parts[:-1],
            create=False,
        )
        try:
            if (
                os.fstat(reopened_parent).st_dev,
                os.fstat(reopened_parent).st_ino,
            ) != (
                os.fstat(destination_parent).st_dev,
                os.fstat(destination_parent).st_ino,
            ):
                raise GateBlocked(
                    "artifact-export-parent-changed",
                    str(destination_lexical.parent),
                )
        finally:
            os.close(reopened_parent)
        try:
            prior_metadata = os.stat(
                destination_relative.name,
                dir_fd=destination_parent,
                follow_symlinks=False,
            )
        except FileNotFoundError:
            prior_metadata = None
        else:
            if (
                not stat.S_ISREG(prior_metadata.st_mode)
                or prior_metadata.st_uid != os.geteuid()
                or prior_metadata.st_nlink != 1
            ):
                raise GateBlocked(
                    "unsafe-prior-artifact-export",
                    str(destination_lexical),
                )
            os.link(
                destination_relative.name,
                backup,
                src_dir_fd=destination_parent,
                dst_dir_fd=destination_parent,
                follow_symlinks=False,
            )
            backup_created = True
            backup_metadata = os.stat(
                backup,
                dir_fd=destination_parent,
                follow_symlinks=False,
            )
            if (backup_metadata.st_dev, backup_metadata.st_ino) != (
                prior_metadata.st_dev,
                prior_metadata.st_ino,
            ):
                raise GateBlocked(
                    "artifact-export-backup-identity-mismatch",
                    str(destination_lexical),
                )
            os.fsync(destination_parent)
        os.rename(
            temporary,
            destination_relative.name,
            src_dir_fd=destination_parent,
            dst_dir_fd=destination_parent,
        )
        published = True
        exported = os.stat(
            destination_relative.name,
            dir_fd=destination_parent,
            follow_symlinks=False,
        )
        if (
            not stat.S_ISREG(exported.st_mode)
            or exported.st_nlink != 1
            or (exported.st_dev, exported.st_ino)
            != (destination_metadata.st_dev, destination_metadata.st_ino)
            or exported.st_size != size
        ):
            raise GateBlocked(
                "artifact-export-identity-mismatch",
                str(destination_lexical),
            )
        os.fsync(destination_parent)
        publication_committed = True
        if backup_created:
            try:
                backup_observed = os.stat(
                    backup,
                    dir_fd=destination_parent,
                    follow_symlinks=False,
                )
                if (
                    prior_metadata is None
                    or (backup_observed.st_dev, backup_observed.st_ino)
                    != (prior_metadata.st_dev, prior_metadata.st_ino)
                ):
                    raise GateBlocked(
                        "artifact-export-backup-changed",
                        str(destination_lexical),
                    )
                os.unlink(backup, dir_fd=destination_parent)
                backup_created = False
                os.fsync(destination_parent)
            except (GateBlocked, OSError):
                # The new canonical artifact was already made durable. A
                # hidden prior-version hardlink is housekeeping evidence,
                # not a reason to report BLOCK after publication.
                backup_cleanup_durable = False
        return {
            "decision": "EXPORTED",
            "artifact_relative_path": relative.as_posix(),
            "target_artifact_path": str(target / relative),
            "export_path": str(destination_lexical),
            "bytes": size,
            "sha256": digest.hexdigest(),
            "source_reread_bytes": reread_bytes,
            "source_reread_sha256": source_reread_sha256,
            "source_device": source_before.st_dev,
            "source_inode": source_before.st_ino,
            "source_mode": stat.S_IMODE(source_before.st_mode),
            "source_nlink": source_before.st_nlink,
            "source_nlink_maximum": MAX_EXPORT_SOURCE_ARTIFACT_LINKS,
            "source_cargo_uplift_hardlinked": (
                source_before.st_nlink > 1
            ),
            "export_device": exported.st_dev,
            "export_inode": exported.st_ino,
            "export_mode": stat.S_IMODE(exported.st_mode),
            "export_nlink": exported.st_nlink,
            "replaced_prior_artifact": prior_metadata is not None,
            "prior_backup_cleanup_durable": backup_cleanup_durable,
            "filesystem_free_bytes_before": free_before_export,
            "filesystem_free_bytes_after_staging": free_after_staging,
            "filesystem_free_floor_bytes": free_floor_bytes,
            "artifact_logical_bytes_limit": (
                MAX_EXPORTED_ARTIFACT_LOGICAL_BYTES
            ),
            "artifact_staged_allocated_bytes": (
                staged_allocated_bytes
            ),
            "artifact_export_deadline_seconds": (
                ARTIFACT_EXPORT_TIMEOUT_SECONDS
            ),
            "residue_recovery": residue_recovery,
        }
    except BaseException as original:
        rollback_errors: list[str] = []
        if (
            destination_parent is not None
            and published
            and not publication_committed
            and destination_metadata is not None
        ):
            try:
                current = os.stat(
                    destination_relative.name,
                    dir_fd=destination_parent,
                    follow_symlinks=False,
                )
                if (current.st_dev, current.st_ino) != (
                    destination_metadata.st_dev,
                    destination_metadata.st_ino,
                ):
                    raise GateBlocked(
                        "artifact-export-rollback-foreign-inode",
                        str(destination_lexical),
                    )
                os.unlink(
                    destination_relative.name,
                    dir_fd=destination_parent,
                )
                published = False
            except BaseException as rollback_error:
                rollback_errors.append(str(rollback_error))
        if (
            destination_parent is not None
            and backup_created
            and not publication_committed
            and prior_metadata is not None
        ):
            try:
                backup_observed = os.stat(
                    backup,
                    dir_fd=destination_parent,
                    follow_symlinks=False,
                )
                if (backup_observed.st_dev, backup_observed.st_ino) != (
                    prior_metadata.st_dev,
                    prior_metadata.st_ino,
                ):
                    raise GateBlocked(
                        "artifact-export-rollback-backup-changed",
                        str(destination_lexical),
                    )
                try:
                    current = os.stat(
                        destination_relative.name,
                        dir_fd=destination_parent,
                        follow_symlinks=False,
                    )
                except FileNotFoundError:
                    current = None
                if current is None:
                    os.rename(
                        backup,
                        destination_relative.name,
                        src_dir_fd=destination_parent,
                        dst_dir_fd=destination_parent,
                    )
                elif (current.st_dev, current.st_ino) == (
                    prior_metadata.st_dev,
                    prior_metadata.st_ino,
                ):
                    os.unlink(backup, dir_fd=destination_parent)
                else:
                    raise GateBlocked(
                        "artifact-export-rollback-foreign-inode",
                        str(destination_lexical),
                    )
                backup_created = False
            except BaseException as rollback_error:
                rollback_errors.append(str(rollback_error))
        if destination_parent is not None:
            try:
                os.fsync(destination_parent)
            except OSError as rollback_error:
                rollback_errors.append(str(rollback_error))
        if rollback_errors:
            raise GateBlocked(
                "artifact-export-rollback-failed",
                "; ".join(rollback_errors),
            ) from original
        raise
    finally:
        if destination_descriptor is not None:
            os.close(destination_descriptor)
        if destination_parent is not None:
            try:
                os.unlink(temporary, dir_fd=destination_parent)
            except FileNotFoundError:
                pass
            if backup_created and not publication_committed:
                try:
                    backup_observed = os.stat(
                        backup,
                        dir_fd=destination_parent,
                        follow_symlinks=False,
                    )
                    if (
                        prior_metadata is not None
                        and (backup_observed.st_dev, backup_observed.st_ino)
                        == (prior_metadata.st_dev, prior_metadata.st_ino)
                    ):
                        os.unlink(backup, dir_fd=destination_parent)
                except FileNotFoundError:
                    pass
            os.close(destination_parent)
        if candidate_descriptor is not None:
            os.close(candidate_descriptor)
        if source_descriptor is not None:
            os.close(source_descriptor)
        if source_parent is not None:
            os.close(source_parent)
        os.close(target_descriptor)


def write_namespace_record(
    target: Path,
    name: str,
    raw: bytes,
    *,
    replace: bool,
) -> None:
    directory = open_directory_no_follow(target)
    temporary = f".{name}.tmp-{os.getpid()}-{time.monotonic_ns()}"
    descriptor: int | None = None
    try:
        descriptor = os.open(
            temporary,
            os.O_WRONLY
            | os.O_CREAT
            | os.O_EXCL
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            0o600,
            dir_fd=directory,
        )
        write_all(descriptor, raw)
        os.fsync(descriptor)
        os.close(descriptor)
        descriptor = None
        if replace:
            try:
                existing = os.stat(
                    name, dir_fd=directory, follow_symlinks=False
                )
            except FileNotFoundError:
                pass
            else:
                if (
                    not stat.S_ISREG(existing.st_mode)
                    or existing.st_nlink != 1
                ):
                    raise GateBlocked(
                        "namespace-record-unsafe", str(target / name)
                    )
            os.rename(
                temporary,
                name,
                src_dir_fd=directory,
                dst_dir_fd=directory,
            )
        else:
            os.link(
                temporary,
                name,
                src_dir_fd=directory,
                dst_dir_fd=directory,
                follow_symlinks=False,
            )
            os.unlink(temporary, dir_fd=directory)
        os.fsync(directory)
    finally:
        if descriptor is not None:
            os.close(descriptor)
        try:
            os.unlink(temporary, dir_fd=directory)
        except FileNotFoundError:
            pass
        os.close(directory)


def invalidate_namespace(
    target: Path,
    *,
    source_key: str,
    reason: str,
    before: Mapping[str, Any],
    after: Mapping[str, Any] | None,
) -> dict[str, Any]:
    payload = {
        "schema": "codefire.cargo_namespace_invalidation.v1",
        "source_key": source_key,
        "reason": reason,
        "invalidated_at": utc_now(),
        "candidate_before_sha256": sha256_json(dict(before)),
        "candidate_after_sha256": (
            sha256_json(dict(after)) if after is not None else None
        ),
    }
    raw = marker_bytes(payload)
    try:
        write_namespace_record(
            target,
            INVALIDATION_NAME,
            raw,
            replace=False,
        )
    except FileExistsError:
        directory = open_directory_no_follow(target)
        try:
            raw = read_regular_at(
                directory,
                INVALIDATION_NAME,
                display=target / INVALIDATION_NAME,
            )
        finally:
            os.close(directory)
        try:
            existing_payload = json.loads(raw)
        except json.JSONDecodeError as exc:
            raise GateBlocked(
                "namespace-invalidation-invalid",
                str(target / INVALIDATION_NAME),
            ) from exc
        if (
            not isinstance(existing_payload, dict)
            or existing_payload.get("schema")
            != "codefire.cargo_namespace_invalidation.v1"
            or existing_payload.get("source_key") != source_key
            or raw != marker_bytes(existing_payload)
        ):
            raise GateBlocked(
                "namespace-invalidation-mismatch",
                str(target / INVALIDATION_NAME),
            )
    return {
        "decision": "INVALIDATED",
        "reason": reason,
        "record_sha256": hashlib.sha256(raw).hexdigest(),
        "namespace_exists_after": True,
    }


def remove_tree_contents_at(
    directory: int,
    *,
    marker_name_last: str | None = None,
    traversal: dict[str, Any] | None = None,
) -> dict[str, int]:
    """Remove one quarantined tree and measure exactly what it reclaimed.

    Removal is bounded only by a finite wall deadline, never by an entry
    count. An entry cap is the right bound for a measurement scan, which
    makes no progress when repeated, but the wrong bound for destructive
    cleanup: a crash-left namespace larger than the cap would fail every
    cleanup attempt identically and stay addressable forever, consuming the
    aggregate budget and blocking all later admissions. Every unlink here is
    progress, so a deadline stop leaves a strictly smaller tree that the next
    run's quarantine recovery resumes.

    Sizes are accumulated from the same no-follow ``stat`` that decides how
    each entry is removed, so every removed entry is measured without a
    second traversal of a namespace that can hold gigabytes of Cargo
    output.

    Allocated bytes follow the same convention as ``namespace_usage_at``:
    ``st_blocks * 512`` summed per *directory entry*, with no inode
    de-duplication. A hard-linked inode is therefore counted once per link,
    and Cargo's uplift routinely gives every produced artifact a second
    link, so this figure is an upper bound on the allocation a removal
    actually returns to the filesystem -- roughly double for the uplifted
    files. The convention is deliberate rather than incidental: it matches
    the aggregate scan the retention policy compares against, so budget
    decisions stay internally consistent and err toward blocking earlier,
    and the measured sizing basis was derived under it. Non-directory entries
    with ``st_nlink > 1`` are counted separately so the overstatement is
    visible in the evidence instead of being implied; directories are
    excluded because they always carry a link per child and cannot be hard
    linked.
    """
    if traversal is None:
        traversal = {
            "entries": 0,
            "maximum_entries": None,
            "deadline": time.monotonic() + TREE_CLEANUP_SECONDS,
        }
    allocated_bytes = 0
    logical_bytes = 0
    removed_entries = 0
    hardlinked_entries = 0
    duplicate_allocated_bytes = 0
    # Only inodes that actually carry extra links are tracked, so this set
    # holds the uplifted artifacts rather than one entry per file.
    seen_multilink_inodes: set[tuple[int, int]] = set()
    root_identity = os.fstat(directory)
    events: list[tuple[bool, str, int, int]] = [
        (False, "", root_identity.st_dev, root_identity.st_ino)
    ]
    while events:
        if time.monotonic() > float(traversal["deadline"]):
            raise GateBlocked(
                "cleanup-deadline-exceeded",
                str(TREE_CLEANUP_SECONDS),
            )
        leaving, relative, expected_device, expected_inode = (
            events.pop()
        )
        current = reopen_directory_at(
            directory,
            relative,
            code="cleanup-entry-identity-mismatch",
        )
        try:
            opened = os.fstat(current)
            if (opened.st_dev, opened.st_ino) != (
                expected_device,
                expected_inode,
            ):
                raise GateBlocked(
                    "cleanup-entry-identity-mismatch",
                    relative or ".",
                )
            if not leaving:
                os.fchmod(current, 0o700)
                children: list[tuple[str, int, int]] = []
                with os.scandir(current) as scanner:
                    for entry in scanner:
                        if time.monotonic() > float(
                            traversal["deadline"]
                        ):
                            raise GateBlocked(
                                "cleanup-deadline-exceeded",
                                str(TREE_CLEANUP_SECONDS),
                            )
                        name = entry.name
                        if (
                            not relative
                            and marker_name_last is not None
                            and name == marker_name_last
                        ):
                            continue
                        traversal["entries"] += 1
                        maximum_entries = traversal.get(
                            "maximum_entries"
                        )
                        if (
                            maximum_entries is not None
                            and traversal["entries"] > maximum_entries
                        ):
                            raise GateBlocked(
                                "cleanup-entry-count-limit-exceeded",
                                str(maximum_entries),
                            )
                        metadata = entry.stat(
                            follow_symlinks=False
                        )
                        removed_entries += 1
                        allocated_bytes += metadata.st_blocks * 512
                        logical_bytes += metadata.st_size
                        # Directories legitimately carry st_nlink >= 2
                        # (".." from each child), and cannot be hard
                        # linked. Counting them here would inflate the
                        # figure that exists to expose hard-link
                        # double-counting.
                        #
                        # st_nlink drops as this walk unlinks, so a link
                        # count observed later is not comparable with one
                        # observed earlier. Identity is stable, so repeats
                        # are detected by (st_dev, st_ino) instead: for
                        # links wholly inside this namespace that yields
                        # the exact number of bytes the per-entry
                        # convention counted more than once.
                        if not stat.S_ISDIR(metadata.st_mode):
                            inode = (metadata.st_dev, metadata.st_ino)
                            if inode in seen_multilink_inodes:
                                duplicate_allocated_bytes += (
                                    metadata.st_blocks * 512
                                )
                                hardlinked_entries += 1
                            elif metadata.st_nlink > 1:
                                seen_multilink_inodes.add(inode)
                                hardlinked_entries += 1
                        child_relative = (
                            f"{relative}/{name}"
                            if relative
                            else name
                        )
                        if stat.S_ISDIR(metadata.st_mode):
                            children.append(
                                (
                                    child_relative,
                                    metadata.st_dev,
                                    metadata.st_ino,
                                )
                            )
                        else:
                            os.unlink(name, dir_fd=current)
                events.append(
                    (
                        True,
                        relative,
                        expected_device,
                        expected_inode,
                    )
                )
                for child_relative, device, inode in reversed(
                    children
                ):
                    events.append(
                        (False, child_relative, device, inode)
                    )
                continue
            os.fsync(current)
        finally:
            os.close(current)

        if relative:
            if "/" in relative:
                parent_relative, name = relative.rsplit("/", 1)
            else:
                parent_relative, name = "", relative
            parent = reopen_directory_at(
                directory,
                parent_relative,
                code="cleanup-entry-identity-mismatch",
            )
            try:
                observed = os.stat(
                    name, dir_fd=parent, follow_symlinks=False
                )
                if (observed.st_dev, observed.st_ino) != (
                    expected_device,
                    expected_inode,
                ):
                    raise GateBlocked(
                        "cleanup-entry-identity-mismatch", relative
                    )
                os.rmdir(name, dir_fd=parent)
                os.fsync(parent)
            finally:
                os.close(parent)
            continue

        if marker_name_last is not None:
            root = reopen_directory_at(
                directory,
                "",
                code="cleanup-entry-identity-mismatch",
            )
            try:
                try:
                    marker = os.stat(
                        marker_name_last,
                        dir_fd=root,
                        follow_symlinks=False,
                    )
                except FileNotFoundError:
                    pass
                else:
                    if stat.S_ISDIR(marker.st_mode):
                        raise GateBlocked(
                            "cleanup-marker-not-regular",
                            marker_name_last,
                        )
                    os.unlink(marker_name_last, dir_fd=root)
                    traversal["entries"] += 1
                    removed_entries += 1
                    allocated_bytes += marker.st_blocks * 512
                    logical_bytes += marker.st_size
                    # The marker is skipped by the scandir walk and removed
                    # here, last, so it needs the walk's own repeat test
                    # rather than a bare st_nlink read. By this point the
                    # walk has already unlinked any sibling link, so the
                    # marker's st_nlink has fallen back to 1 and would
                    # register as un-linked; only the inode identity still
                    # says otherwise. Without this the per-entry convention
                    # double-counts a linked marker while the overcount
                    # field reports zero.
                    marker_inode = (marker.st_dev, marker.st_ino)
                    if not stat.S_ISDIR(marker.st_mode):
                        if marker_inode in seen_multilink_inodes:
                            duplicate_allocated_bytes += (
                                marker.st_blocks * 512
                            )
                            hardlinked_entries += 1
                        elif marker.st_nlink > 1:
                            seen_multilink_inodes.add(marker_inode)
                            hardlinked_entries += 1
                os.fsync(root)
            finally:
                os.close(root)
    return {
        "allocated_bytes": allocated_bytes,
        "logical_bytes": logical_bytes,
        "entry_count": removed_entries,
        "hardlinked_entry_count": hardlinked_entries,
        "duplicate_allocated_bytes": duplicate_allocated_bytes,
    }


def cleanup_partial_snapshot(
    snapshot_root: Path,
    snapshot: Path,
    *,
    traversal: dict[str, Any] | None = None,
) -> bool:
    if snapshot.parent != snapshot_root or not snapshot.name.startswith(
        (".tmp-", "ready-")
    ):
        raise GateBlocked("unsafe-partial-snapshot-path", str(snapshot))
    root_descriptor = open_directory_no_follow(snapshot_root)
    snapshot_descriptor: int | None = None
    try:
        try:
            observed = os.stat(
                snapshot.name,
                dir_fd=root_descriptor,
                follow_symlinks=False,
            )
        except FileNotFoundError:
            return False
        if not stat.S_ISDIR(observed.st_mode):
            raise GateBlocked(
                "partial-snapshot-not-directory", str(snapshot)
            )
        snapshot_descriptor = os.open(
            snapshot.name,
            os.O_RDONLY
            | os.O_DIRECTORY
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            dir_fd=root_descriptor,
        )
        opened = os.fstat(snapshot_descriptor)
        if (opened.st_dev, opened.st_ino) != (
            observed.st_dev,
            observed.st_ino,
        ):
            raise GateBlocked(
                "partial-snapshot-identity-mismatch", str(snapshot)
            )
        remove_tree_contents_at(
            snapshot_descriptor, traversal=traversal
        )
        current = os.stat(
            snapshot.name,
            dir_fd=root_descriptor,
            follow_symlinks=False,
        )
        if (opened.st_dev, opened.st_ino) != (
            current.st_dev,
            current.st_ino,
        ):
            raise GateBlocked(
                "partial-snapshot-changed-before-remove", str(snapshot)
            )
        os.close(snapshot_descriptor)
        snapshot_descriptor = None
        os.rmdir(snapshot.name, dir_fd=root_descriptor)
        os.fsync(root_descriptor)
        return True
    finally:
        if snapshot_descriptor is not None:
            os.close(snapshot_descriptor)
        os.close(root_descriptor)


def recover_snapshot_root(snapshot_root: Path) -> dict[str, Any]:
    root = validate_private_root(
        snapshot_root, code="unsafe-snapshot-root"
    )
    root_descriptor = open_directory_no_follow(root)
    deadline = time.monotonic() + TREE_CLEANUP_SECONDS
    try:
        names = bounded_directory_names_at(
            root_descriptor,
            deadline=deadline,
            code="snapshot-root-scan-limit-exceeded",
            predicate=lambda name: name.startswith(
                (".tmp-", "ready-")
            ),
        )
    finally:
        os.close(root_descriptor)
    # Destructive cleanup is deadline-bounded only; an entry cap here
    # would make an oversized crash-left tree permanently unremovable.
    # See remove_tree_contents_at.
    traversal = {
        "entries": 0,
        "maximum_entries": None,
        "deadline": deadline,
    }
    recovered: list[str] = []
    for name in names:
        cleanup_partial_snapshot(
            root, root / name, traversal=traversal
        )
        recovered.append(name)
    return {
        "decision": "SNAPSHOT_ROOT_RECOVERED",
        "removed_count": len(recovered),
        "removed_names_sha256": ordered_name_digest(recovered),
    }


def recover_registry_quarantines(registry: Path) -> dict[str, Any]:
    if not registry.exists():
        return {
            "decision": "REGISTRY_QUARANTINES_RECOVERED",
            "removed_count": 0,
            "removed_names_sha256": sha256_json([]),
        }
    descriptor = open_directory_no_follow(registry)
    deadline = time.monotonic() + TREE_CLEANUP_SECONDS
    # Destructive cleanup is deadline-bounded only; an entry cap here
    # would make an oversized crash-left tree permanently unremovable.
    # See remove_tree_contents_at.
    traversal = {
        "entries": 0,
        "maximum_entries": None,
        "deadline": deadline,
    }
    removed: list[str] = []
    try:
        names = bounded_directory_names_at(
            descriptor,
            deadline=deadline,
            code="registry-root-scan-limit-exceeded",
            predicate=lambda name: name.startswith(
                (".quarantine-", ".prepare-")
            ),
            maximum_entries=MAX_REGISTRY_ROOT_ENTRIES,
        )
        for name in names:
            observed = os.stat(
                name, dir_fd=descriptor, follow_symlinks=False
            )
            if not stat.S_ISDIR(observed.st_mode):
                raise GateBlocked(
                    "unsafe-registry-recovery-entry", str(registry / name)
                )
            child = os.open(
                name,
                os.O_RDONLY
                | os.O_DIRECTORY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
                dir_fd=descriptor,
            )
            try:
                opened = os.fstat(child)
                if (opened.st_dev, opened.st_ino) != (
                    observed.st_dev,
                    observed.st_ino,
                ):
                    raise GateBlocked(
                        "registry-quarantine-identity-mismatch", name
                    )
                remove_tree_contents_at(
                    child, traversal=traversal
                )
            finally:
                os.close(child)
            os.rmdir(name, dir_fd=descriptor)
            os.fsync(descriptor)
            removed.append(name)
    finally:
        os.close(descriptor)
    return {
        "decision": "REGISTRY_QUARANTINES_RECOVERED",
        "removed_count": len(removed),
        "removed_names_sha256": ordered_name_digest(removed),
    }


def recover_incomplete_namespaces(
    admitted_root: Path,
    registry: Path,
) -> dict[str, Any]:
    recovered: list[str] = []
    if not registry.exists():
        return {
            "decision": "INCOMPLETE_NAMESPACES_RECOVERED",
            "removed_count": 0,
            "removed_source_keys_sha256": sha256_json([]),
        }
    descriptor = open_directory_no_follow(registry)
    deadline = time.monotonic() + TREE_CLEANUP_SECONDS
    try:
        names = bounded_directory_names_at(
            descriptor,
            deadline=deadline,
            code="registry-root-scan-limit-exceeded",
            maximum_entries=MAX_REGISTRY_ROOT_ENTRIES,
        )
    finally:
        os.close(descriptor)
    # Destructive cleanup is deadline-bounded only; an entry cap here
    # would make an oversized crash-left tree permanently unremovable.
    # See remove_tree_contents_at.
    cleanup_traversal = {
        "entries": 0,
        "maximum_entries": None,
        "deadline": deadline,
    }
    for name in names:
        target = registry / name
        source_key = target.name
        if (
            re.fullmatch(r"[0-9a-f]{64}", source_key) is None
            or target.is_symlink()
            or not target.is_dir()
        ):
            raise GateBlocked(
                "unknown-source-key-registry-entry", str(target)
            )
        target = registry / source_key
        directory = open_directory_no_follow(target)
        expected_marker: dict[str, Any] | None = None
        invalidated = False
        try:
            try:
                raw = read_regular_at(
                    directory,
                    MARKER_NAME,
                    display=target / MARKER_NAME,
                )
                parsed_marker = json.loads(raw)
                if (
                    not isinstance(parsed_marker, dict)
                    or parsed_marker.get("schema") != MARKER_SCHEMA
                    or parsed_marker.get("source_key") != source_key
                    or parsed_marker.get("target_namespace") != str(target)
                    or raw != marker_bytes(parsed_marker)
                ):
                    raise GateBlocked(
                        "source-marker-registry-mismatch",
                        str(target / MARKER_NAME),
                    )
                expected_marker = parsed_marker
            except (GateBlocked, json.JSONDecodeError, OSError):
                expected_marker = None
            try:
                os.stat(
                    INVALIDATION_NAME,
                    dir_fd=directory,
                    follow_symlinks=False,
                )
            except FileNotFoundError:
                pass
            else:
                invalidated = True
        finally:
            os.close(directory)
        try:
            state = read_run_state(target, source_key=source_key)
        except (GateBlocked, OSError):
            state = None
        if (
            expected_marker is None
            or invalidated
            or state is None
            or state.get("state") != "COMMITTED"
        ):
            cleanup_namespace_locked(
                admitted_root,
                target,
                expected_marker,
                cleanup_traversal=cleanup_traversal,
                measure_before=False,
            )
            recovered.append(source_key)
    return {
        "decision": "INCOMPLETE_NAMESPACES_RECOVERED",
        "removed_count": len(recovered),
        "removed_source_keys_sha256": ordered_name_digest(recovered),
    }


def snapshot_file_manifest_at(
    directory: int,
    *,
    prefix: str = "",
) -> list[dict[str, Any]]:
    files: list[dict[str, Any]] = []
    root_identity = os.fstat(directory)
    pending: list[tuple[str, str, int, int]] = [
        ("", prefix, root_identity.st_dev, root_identity.st_ino)
    ]
    logical_bytes = 0
    deadline = time.monotonic() + SNAPSHOT_VERIFY_SECONDS
    while pending:
        if time.monotonic() > deadline:
            raise GateBlocked(
                "snapshot-verification-deadline-exceeded",
                str(SNAPSHOT_VERIFY_SECONDS),
            )
        (
            traversal_relative,
            display_prefix,
            expected_device,
            expected_inode,
        ) = pending.pop()
        current = reopen_directory_at(
            directory,
            traversal_relative,
            code="snapshot-entry-identity-mismatch",
        )
        try:
            opened_directory = os.fstat(current)
            if (opened_directory.st_dev, opened_directory.st_ino) != (
                expected_device,
                expected_inode,
            ):
                raise GateBlocked(
                    "snapshot-entry-identity-mismatch",
                    display_prefix or ".",
                )
            child_directories: list[
                tuple[str, str, int, int]
            ] = []
            names = bounded_directory_names_at(
                current,
                deadline=deadline,
                code="snapshot-verification-deadline-exceeded",
                maximum_entries=MAX_NAMESPACE_ENTRY_COUNT,
            )
            for name in names:
                if time.monotonic() > deadline:
                    raise GateBlocked(
                        "snapshot-verification-deadline-exceeded",
                        str(SNAPSHOT_VERIFY_SECONDS),
                    )
                traversal_child = (
                    f"{traversal_relative}/{name}"
                    if traversal_relative
                    else name
                )
                relative = (
                    f"{display_prefix}/{name}"
                    if display_prefix
                    else name
                )
                metadata = os.stat(
                    name,
                    dir_fd=current,
                    follow_symlinks=False,
                )
                if stat.S_ISDIR(metadata.st_mode):
                    child_directories.append(
                        (
                            traversal_child,
                            relative,
                            metadata.st_dev,
                            metadata.st_ino,
                        )
                    )
                    continue
                if relative == SNAPSHOT_MARKER_NAME:
                    continue
                if (
                    not stat.S_ISREG(metadata.st_mode)
                    or metadata.st_nlink != 1
                ):
                    raise GateBlocked(
                        "snapshot-file-identity-invalid", relative
                    )
                if len(files) >= DEFAULT_MAX_SNAPSHOT_FILES:
                    raise GateBlocked(
                        "snapshot-file-count-exceeded",
                        str(DEFAULT_MAX_SNAPSHOT_FILES),
                    )
                if (
                    logical_bytes + metadata.st_size
                    > DEFAULT_MAX_SNAPSHOT_LOGICAL_BYTES
                ):
                    raise GateBlocked(
                        "snapshot-logical-bytes-exceeded",
                        str(DEFAULT_MAX_SNAPSHOT_LOGICAL_BYTES),
                    )
                descriptor = os.open(
                    name,
                    os.O_RDONLY
                    | os.O_CLOEXEC
                    | getattr(os, "O_NOFOLLOW", 0),
                    dir_fd=current,
                )
                try:
                    opened = os.fstat(descriptor)
                    if (
                        (opened.st_dev, opened.st_ino)
                        != (metadata.st_dev, metadata.st_ino)
                        or opened.st_nlink != 1
                    ):
                        raise GateBlocked(
                            "snapshot-entry-identity-mismatch",
                            relative,
                        )
                    digest = hashlib.sha256()
                    size = 0
                    while True:
                        if time.monotonic() > deadline:
                            raise GateBlocked(
                                "snapshot-verification-deadline-exceeded",
                                str(SNAPSHOT_VERIFY_SECONDS),
                            )
                        chunk = os.read(descriptor, 1024 * 1024)
                        if not chunk:
                            break
                        digest.update(chunk)
                        size += len(chunk)
                    after = os.fstat(descriptor)
                    if (
                        (
                            opened.st_dev,
                            opened.st_ino,
                            opened.st_size,
                        )
                        != (
                            after.st_dev,
                            after.st_ino,
                            after.st_size,
                        )
                        or size != after.st_size
                    ):
                        raise GateBlocked(
                            "snapshot-file-changed-during-verification",
                            relative,
                        )
                    logical_bytes += size
                    files.append(
                        {
                            "path": relative,
                            "mode": (
                                "100755"
                                if opened.st_mode & stat.S_IXUSR
                                else "100644"
                            ),
                            "bytes": size,
                            "sha256": digest.hexdigest(),
                        }
                    )
                finally:
                    os.close(descriptor)
            for child in reversed(child_directories):
                pending.append(child)
        finally:
            os.close(current)
    files.sort(key=lambda item: str(item["path"]))
    return files


def verify_snapshot_contents(
    snapshot: Path,
    snapshot_identity: Mapping[str, Any],
) -> dict[str, Any]:
    descriptor = open_directory_no_follow(snapshot)
    try:
        expected_marker = {
            key: snapshot_identity[key] for key in SNAPSHOT_MARKER_FIELDS
        }
        marker_raw = read_regular_at(
            descriptor,
            SNAPSHOT_MARKER_NAME,
            display=snapshot / SNAPSHOT_MARKER_NAME,
        )
        if (
            json.loads(marker_raw) != expected_marker
            or marker_raw != marker_bytes(expected_marker)
            or hashlib.sha256(marker_raw).hexdigest()
            != snapshot_identity["marker_sha256"]
        ):
            raise GateBlocked(
                "snapshot-marker-mismatch",
                str(snapshot / SNAPSHOT_MARKER_NAME),
            )
        files = snapshot_file_manifest_at(descriptor)
    finally:
        os.close(descriptor)
    observed_sha256 = sha256_json(files)
    if (
        observed_sha256
        != snapshot_identity["snapshot_file_manifest_sha256"]
    ):
        raise GateBlocked(
            "snapshot-content-changed-during-gate", observed_sha256
        )
    return {
        "decision": "SNAPSHOT_CONTENT_VERIFIED",
        "file_count": len(files),
        "file_manifest_sha256": observed_sha256,
        "marker_sha256": hashlib.sha256(marker_raw).hexdigest(),
    }


def cleanup_snapshot(
    snapshot: Path,
    snapshot_identity: Mapping[str, Any],
) -> dict[str, Any]:
    snapshot_root = snapshot.parent
    root_descriptor = open_directory_no_follow(snapshot_root)
    snapshot_descriptor: int | None = None
    quarantine = (
        f".quarantine-{snapshot.name}-{os.getpid()}-{time.monotonic_ns()}"
    )
    marker_expected = {
        key: snapshot_identity[key] for key in SNAPSHOT_MARKER_FIELDS
    }
    try:
        snapshot_descriptor = open_directory_no_follow(snapshot)
        marker_raw = read_regular_at(
            snapshot_descriptor,
            SNAPSHOT_MARKER_NAME,
            display=snapshot / SNAPSHOT_MARKER_NAME,
        )
        if (
            json.loads(marker_raw) != marker_expected
            or marker_raw != marker_bytes(marker_expected)
            or hashlib.sha256(marker_raw).hexdigest()
            != snapshot_identity["marker_sha256"]
        ):
            raise GateBlocked(
                "snapshot-marker-mismatch",
                str(snapshot / SNAPSHOT_MARKER_NAME),
            )
        opened = os.fstat(snapshot_descriptor)
        observed = os.stat(
            snapshot.name,
            dir_fd=root_descriptor,
            follow_symlinks=False,
        )
        if (opened.st_dev, opened.st_ino) != (
            observed.st_dev,
            observed.st_ino,
        ):
            raise GateBlocked("snapshot-identity-mismatch", str(snapshot))
        os.rename(
            snapshot.name,
            quarantine,
            src_dir_fd=root_descriptor,
            dst_dir_fd=root_descriptor,
        )
        os.fsync(root_descriptor)
        quarantined = os.stat(
            quarantine,
            dir_fd=root_descriptor,
            follow_symlinks=False,
        )
        if (opened.st_dev, opened.st_ino) != (
            quarantined.st_dev,
            quarantined.st_ino,
        ):
            raise GateBlocked(
                "snapshot-quarantine-identity-mismatch", quarantine
            )
        remove_tree_contents_at(
            snapshot_descriptor, marker_name_last=SNAPSHOT_MARKER_NAME
        )
        current = os.stat(
            quarantine,
            dir_fd=root_descriptor,
            follow_symlinks=False,
        )
        if (opened.st_dev, opened.st_ino) != (
            current.st_dev,
            current.st_ino,
        ):
            raise GateBlocked(
                "snapshot-quarantine-changed-before-remove", quarantine
            )
        os.close(snapshot_descriptor)
        snapshot_descriptor = None
        os.rmdir(quarantine, dir_fd=root_descriptor)
        os.fsync(root_descriptor)
        exists_after = False
        for name in (snapshot.name, quarantine):
            try:
                os.stat(name, dir_fd=root_descriptor, follow_symlinks=False)
            except FileNotFoundError:
                continue
            exists_after = True
        if exists_after:
            raise GateBlocked(
                "snapshot-cleanup-incomplete", str(snapshot)
            )
    finally:
        if snapshot_descriptor is not None:
            os.close(snapshot_descriptor)
        os.close(root_descriptor)
    return {
        "decision": "SNAPSHOT_CLEANED",
        "snapshot_exists_after": False,
        "quarantine_exists_after": False,
        "marker_removed_last": True,
        "reclaimed_allocated_bytes": snapshot_identity["allocated_bytes"],
    }


def cleanup_namespace_locked(
    admitted_root: Path,
    target: Path,
    expected_marker: dict[str, Any] | None,
    *,
    cleanup_traversal: dict[str, Any] | None = None,
    measure_before: bool = True,
) -> dict[str, Any]:
    registry = target.parent
    registry_descriptor = open_directory_no_follow(registry)
    target_descriptor: int | None = None
    quarantine_name = (
        f".quarantine-{target.name}-{os.getpid()}-{time.monotonic_ns()}"
    )
    try:
        target_descriptor = open_directory_no_follow(target)
        if expected_marker is not None:
            validate_marker_at(
                target_descriptor,
                MARKER_NAME,
                expected_marker,
                display=target / MARKER_NAME,
            )
        if measure_before:
            try:
                before: dict[str, int] | None = namespace_usage_at(
                    target_descriptor
                )
            except GateBlocked as exc:
                if exc.code != "namespace-entry-count-limit-exceeded":
                    raise
                # The admission scan bound does not stop cleanup, and it no
                # longer costs exact accounting either: the removal walk
                # below measures every entry it unlinks.
                before = None
        else:
            before = None
        opened = os.fstat(target_descriptor)
        observed = os.stat(
            target.name,
            dir_fd=registry_descriptor,
            follow_symlinks=False,
        )
        if (opened.st_dev, opened.st_ino) != (
            observed.st_dev,
            observed.st_ino,
        ):
            raise GateBlocked("cleanup-target-identity-mismatch", str(target))
        try:
            os.stat(
                quarantine_name,
                dir_fd=registry_descriptor,
                follow_symlinks=False,
            )
        except FileNotFoundError:
            pass
        else:
            raise GateBlocked(
                "cleanup-quarantine-collision", quarantine_name
            )
        os.rename(
            target.name,
            quarantine_name,
            src_dir_fd=registry_descriptor,
            dst_dir_fd=registry_descriptor,
        )
        os.fsync(registry_descriptor)
        try:
            os.stat(
                target.name,
                dir_fd=registry_descriptor,
                follow_symlinks=False,
            )
        except FileNotFoundError:
            pass
        else:
            raise GateBlocked(
                "cleanup-source-key-still-addressable", str(target)
            )
        quarantined = os.stat(
            quarantine_name,
            dir_fd=registry_descriptor,
            follow_symlinks=False,
        )
        if (opened.st_dev, opened.st_ino) != (
            quarantined.st_dev,
            quarantined.st_ino,
        ):
            raise GateBlocked(
                "cleanup-quarantine-identity-mismatch", quarantine_name
            )
        removed = remove_tree_contents_at(
            target_descriptor,
            marker_name_last=MARKER_NAME,
            traversal=cleanup_traversal,
        )
        current = os.stat(
            quarantine_name,
            dir_fd=registry_descriptor,
            follow_symlinks=False,
        )
        if (opened.st_dev, opened.st_ino) != (
            current.st_dev,
            current.st_ino,
        ):
            raise GateBlocked(
                "cleanup-quarantine-changed-before-remove",
                quarantine_name,
            )
        os.close(target_descriptor)
        target_descriptor = None
        os.rmdir(quarantine_name, dir_fd=registry_descriptor)
        os.fsync(registry_descriptor)
        try:
            os.stat(
                quarantine_name,
                dir_fd=registry_descriptor,
                follow_symlinks=False,
            )
        except FileNotFoundError:
            quarantine_exists_after = False
        else:
            quarantine_exists_after = True
        try:
            os.stat(
                target.name,
                dir_fd=registry_descriptor,
                follow_symlinks=False,
            )
        except FileNotFoundError:
            namespace_exists_after = False
        else:
            namespace_exists_after = True
        if namespace_exists_after or quarantine_exists_after:
            raise GateBlocked(
                "cleanup-removal-not-durable",
                f"target={namespace_exists_after}, "
                f"quarantine={quarantine_exists_after}",
            )
    finally:
        if target_descriptor is not None:
            os.close(target_descriptor)
        os.close(registry_descriptor)
    # Reclaimed bytes come from the removal walk, so every removed entry is
    # measured even when the separate admission-scan bound refused to
    # pre-measure the namespace. "Every entry measured" is not the same as
    # "exact bytes returned to the filesystem", and this payload does not
    # claim the stronger property: the per-entry convention counts a
    # hard-linked inode once per link, and nothing charges this figure back
    # against the aggregate budget -- the aggregate is always re-measured
    # under the registry lock. The convention and the hard-link count are
    # reported so a consumer can see the bound rather than infer exactness.
    return {
        "decision": "EPHEMERAL_CLEANED",
        "reclaimed_measurement": {
            "source": "descriptor-relative-removal",
            "covers_every_removed_entry": True,
            "convention": "per-directory-entry-st-blocks-no-inode-dedup",
            "hardlinked_inodes_counted_once_per_link": True,
            "is_upper_bound_on_freed_allocation": True,
            "charged_back_against_aggregate_budget": False,
        },
        "reclaimed_allocated_bytes": removed["allocated_bytes"],
        "reclaimed_logical_bytes": removed["logical_bytes"],
        "removed_entry_count": removed["entry_count"],
        "removed_hardlinked_entry_count": removed[
            "hardlinked_entry_count"
        ],
        # Exact for links wholly inside the namespace, which is the Cargo
        # uplift case; a lower bound when the other link is outside it.
        "reclaimed_allocated_bytes_counted_more_than_once": removed[
            "duplicate_allocated_bytes"
        ],
        "premeasured_allocated_bytes": (
            before["allocated_bytes"] if before is not None else None
        ),
        "namespace_exists_after": False,
        "quarantine_exists_after": False,
        "marker_removed_last": True,
    }


def cleanup_ephemeral_namespace(
    admitted_root: Path,
    target: Path,
    expected_marker: dict[str, Any],
    *,
    lock_timeout_seconds: float,
) -> dict[str, Any]:
    with exclusive_lock(
        admitted_root / REGISTRY_LOCK_NAME, lock_timeout_seconds
    ) as registry_wait:
        result = cleanup_namespace_locked(
            admitted_root, target, expected_marker
        )
    result["registry_lock_wait_seconds"] = registry_wait
    return result


def prepare_namespace(
    admitted_root: Path,
    source_key: str,
    source_key_payload: dict[str, Any],
    *,
    lock_timeout_seconds: float,
    ephemeral: bool = False,
    persistent_aggregate_budget_bytes: int = (
        DEFAULT_PERSISTENT_AGGREGATE_BUDGET_BYTES
    ),
    new_namespace_allocation_reserve_bytes: int = (
        DEFAULT_NEW_NAMESPACE_ALLOCATION_RESERVE_BYTES
    ),
    new_namespace_free_floor_bytes: int = (
        DEFAULT_NEW_NAMESPACE_FREE_FLOOR_BYTES
    ),
    registry_lock_already_held: bool = False,
    registry_lock_wait_seconds: float = 0.0,
) -> tuple[Path, dict[str, Any], str, float, dict[str, Any]]:
    if persistent_aggregate_budget_bytes < 1:
        raise GateBlocked(
            "invalid-persistent-aggregate-budget",
            str(persistent_aggregate_budget_bytes),
        )
    if new_namespace_allocation_reserve_bytes < 0:
        raise GateBlocked(
            "invalid-new-namespace-allocation-reserve",
            str(new_namespace_allocation_reserve_bytes),
        )
    if new_namespace_free_floor_bytes < 0:
        raise GateBlocked(
            "invalid-new-namespace-free-floor",
            str(new_namespace_free_floor_bytes),
        )
    target = namespace_path(admitted_root, source_key)
    expected_marker = marker_payload(source_key, source_key_payload, target)
    registry = admitted_root / SOURCE_KEY_TARGET_DIRECTORY
    lock_context = (
        already_held_lock(registry_lock_wait_seconds)
        if registry_lock_already_held
        else exclusive_lock(
            admitted_root / REGISTRY_LOCK_NAME, lock_timeout_seconds
        )
    )
    with lock_context as registry_wait:
        if registry.exists():
            if registry.is_symlink() or not registry.is_dir():
                raise GateBlocked("unsafe-source-key-registry", str(registry))
        else:
            registry.mkdir(mode=0o700)
        quarantine_recovery = recover_registry_quarantines(registry)
        incomplete_recovery = recover_incomplete_namespaces(
            admitted_root, registry
        )
        inventory = inspect_registry(registry)
        target_preexisting = target.exists()
        filesystem_free = shutil.disk_usage(admitted_root).free
        aggregate_allocated = sum(
            int(item["actual_allocated_bytes"])
            for item in inventory
        )
        current_inventory = next(
            (
                item
                for item in inventory
                if item["source_key"] == source_key
            ),
            None,
        )
        current_actual_allocated = (
            int(current_inventory["actual_allocated_bytes"])
            if current_inventory is not None
            else 0
        )
        if target_preexisting:
            if target.is_symlink() or not target.is_dir():
                raise GateBlocked(
                    "unsafe-source-key-namespace", str(target)
                )
            marker = target / MARKER_NAME
            if not marker.exists():
                target_descriptor = open_directory_no_follow(target)
                try:
                    existing = bounded_directory_names_at(
                        target_descriptor,
                        deadline=(
                            time.monotonic()
                            + REGISTRY_SCAN_SECONDS
                        ),
                        code=(
                            "unmarked-namespace-scan-limit-exceeded"
                        ),
                    )
                finally:
                    os.close(target_descriptor)
                raise GateBlocked(
                    "unmarked-source-key-namespace",
                    f"{target}: {ordered_name_summary(existing)}",
                )
            validate_marker(marker, expected_marker)
            run_state = read_run_state(
                target, source_key=source_key
            )
            if (
                current_inventory is not None
                and current_inventory["invalidated"]
            ) or (
                run_state is None
                or run_state.get("state") != "COMMITTED"
            ):
                raise GateBlocked(
                    "unfinished-or-invalidated-namespace-survived-recovery",
                    str(target),
                )
        projected_allocated = (
            aggregate_allocated
            if target_preexisting
            else aggregate_allocated
            + new_namespace_allocation_reserve_bytes
        )
        if aggregate_allocated > persistent_aggregate_budget_bytes:
            raise GateBlocked(
                "persistent-aggregate-budget-already-exceeded",
                f"observed={aggregate_allocated}, "
                f"budget={persistent_aggregate_budget_bytes}",
            )
        if (
            not ephemeral
            and projected_allocated > persistent_aggregate_budget_bytes
        ):
            raise GateBlocked(
                "persistent-aggregate-budget-exceeded",
                f"observed={aggregate_allocated}, "
                f"current={current_actual_allocated}, "
                f"reserve={new_namespace_allocation_reserve_bytes}, "
                f"projected={projected_allocated}, "
                f"budget={persistent_aggregate_budget_bytes}",
            )
        reservation_for_free_floor = (
            new_namespace_allocation_reserve_bytes
            if not target_preexisting
            else 0
        )
        free_after_reservation = (
            filesystem_free - reservation_for_free_floor
        )
        if free_after_reservation < new_namespace_free_floor_bytes:
            raise GateBlocked(
                "insufficient-space-after-source-namespace-reservation",
                f"free={filesystem_free}, "
                f"reservation={reservation_for_free_floor}, "
                f"remaining={free_after_reservation}, "
                f"floor={new_namespace_free_floor_bytes}",
            )
        if not target_preexisting:
            preparation = registry / (
                f".prepare-{source_key}-{os.getpid()}-"
                f"{time.monotonic_ns()}"
            )
            preparation.mkdir(mode=0o700)
            try:
                write_marker(
                    preparation / MARKER_NAME, expected_marker
                )
                write_run_state(
                    preparation,
                    source_key=source_key,
                    state="PREPARED",
                    execution_key_value=None,
                )
                preparation_descriptor = open_directory_no_follow(
                    preparation
                )
                try:
                    os.fsync(preparation_descriptor)
                finally:
                    os.close(preparation_descriptor)
                registry_descriptor = open_directory_no_follow(registry)
                try:
                    try:
                        os.stat(
                            target.name,
                            dir_fd=registry_descriptor,
                            follow_symlinks=False,
                        )
                    except FileNotFoundError:
                        pass
                    else:
                        raise GateBlocked(
                            "source-key-namespace-appeared-during-prepare",
                            str(target),
                        )
                    os.rename(
                        preparation.name,
                        target.name,
                        src_dir_fd=registry_descriptor,
                        dst_dir_fd=registry_descriptor,
                    )
                    os.fsync(registry_descriptor)
                finally:
                    os.close(registry_descriptor)
            except BaseException:
                if preparation.exists():
                    recover_registry_quarantines(registry)
                if target.exists():
                    cleanup_namespace_locked(
                        admitted_root, target, expected_marker
                    )
                raise
        marker = target / MARKER_NAME
        marker_sha256 = validate_marker(marker, expected_marker)
    admission = {
        "mode": "ephemeral" if ephemeral else "persistent",
        "namespace_preexisting": target_preexisting,
        "persistent_namespace_count_observed": len(inventory),
        "persistent_aggregate_allocated_bytes_before": aggregate_allocated,
        "persistent_aggregate_budget_bytes": (
            persistent_aggregate_budget_bytes
        ),
        "current_namespace_actual_allocated_bytes_before": (
            current_actual_allocated
        ),
        "new_namespace_allocation_reserve_bytes": (
            new_namespace_allocation_reserve_bytes
        ),
        "new_persistent_projected_allocated_bytes": projected_allocated,
        "allocation_budget_basis": {
            "measured_max_namespace_allocated_bytes": (
                MEASURED_MAX_NAMESPACE_ALLOCATED_BYTES
            ),
            "reference_candidate_count": MEASURED_REFERENCE_CANDIDATE_COUNT,
            "headroom_numerator": MEASURED_HEADROOM_NUMERATOR,
            "headroom_denominator": MEASURED_HEADROOM_DENOMINATOR,
        },
        "filesystem_free_bytes_before": filesystem_free,
        "filesystem_free_bytes_after_reservation": free_after_reservation,
        "free_space_reservation_bytes": reservation_for_free_floor,
        "new_namespace_free_floor_bytes": new_namespace_free_floor_bytes,
        "registry_quarantine_recovery": quarantine_recovery,
        "incomplete_namespace_recovery": incomplete_recovery,
    }
    return target, expected_marker, marker_sha256, registry_wait, admission


def validated_evidence_root(
    *,
    candidate_root: Path,
    admitted_root: Path,
    git_common_dir: Path,
    git_dir: Path,
    profile: Mapping[str, Any],
) -> Path:
    evidence_lexical = Path(str(profile["evidence_root"])).absolute()
    try:
        evidence_root = evidence_lexical.resolve(strict=True)
    except FileNotFoundError as exc:
        raise GateBlocked(
            "admitted-evidence-root-missing", str(evidence_lexical)
        ) from exc
    metadata = os.stat(evidence_root, follow_symlinks=False)
    if (
        evidence_root != evidence_lexical
        or not stat.S_ISDIR(metadata.st_mode)
        or metadata.st_uid != os.geteuid()
        or metadata.st_mode & 0o022
    ):
        raise GateBlocked(
            "unsafe-admitted-evidence-root",
            "evidence root must be a real, owner-controlled directory: "
            f"{evidence_lexical}",
        )
    for forbidden in (
        candidate_root,
        admitted_root,
        git_common_dir,
        git_dir,
    ):
        forbidden = forbidden.resolve(strict=False)
        if (
            evidence_root == forbidden
            or forbidden in evidence_root.parents
            or evidence_root in forbidden.parents
        ):
            raise GateBlocked(
                "unsafe-admitted-evidence-root",
                f"evidence root overlaps forbidden path {forbidden}",
            )
    descriptor = os.open(
        evidence_root,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    try:
        opened = os.fstat(descriptor)
        observed = os.stat(evidence_root, follow_symlinks=False)
        if (
            not stat.S_ISDIR(opened.st_mode)
            or opened.st_uid != os.geteuid()
            or opened.st_mode & 0o022
            or (opened.st_dev, opened.st_ino)
            != (metadata.st_dev, metadata.st_ino)
            or (opened.st_dev, opened.st_ino)
            != (observed.st_dev, observed.st_ino)
        ):
            raise GateBlocked(
                "unsafe-admitted-evidence-root",
                f"evidence root identity changed: {evidence_root}",
            )
    except OSError as exc:
        raise GateBlocked(
            "unsafe-admitted-evidence-root",
            f"{evidence_root}: {exc}",
        ) from exc
    finally:
        os.close(descriptor)
    return evidence_root


def evidence_path(
    path: Path,
    *,
    candidate_root: Path,
    admitted_root: Path,
    git_common_dir: Path,
    git_dir: Path,
    profile: Mapping[str, Any],
    label: str,
) -> Path:
    evidence_root = validated_evidence_root(
        candidate_root=candidate_root,
        admitted_root=admitted_root,
        git_common_dir=git_common_dir,
        git_dir=git_dir,
        profile=profile,
    )
    lexical = path.absolute()
    if (
        lexical.parent != evidence_root
        or lexical.name in {"", ".", ".."}
    ):
        raise GateBlocked(
            "unsafe-evidence-path",
            f"{label} must be a direct child of {evidence_root}: {lexical}",
        )

    descriptor = os.open(
        evidence_root,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    try:
        try:
            os.stat(
                lexical.name,
                dir_fd=descriptor,
                follow_symlinks=False,
            )
        except FileNotFoundError:
            pass
        else:
            raise GateBlocked("evidence-path-exists", str(lexical))
    except OSError as exc:
        raise GateBlocked("unsafe-evidence-path", f"{lexical}: {exc}") from exc
    finally:
        os.close(descriptor)
    return lexical


def execution_key(
    source_key: str,
    command: list[str],
    timeout_seconds: float,
    environment_sha256: str,
) -> str:
    return sha256_json(
        {
            "schema": "codefire.cargo_gate_execution_key.v1",
            "source_key": source_key,
            "argv": command,
            "timeout_seconds": timeout_seconds,
            "artifact_environment_sha256": environment_sha256,
        }
    )


class CargoTestLogAnalyzer:
    """Correlate bounded libtest summaries with target-bound Cargo harnesses."""

    def __init__(
        self,
        target: Path,
        *,
        required: bool,
        expected_harness_name: str | None = None,
        expected_harness_count: int | None = None,
    ):
        self.target = target.resolve(strict=False)
        self.required = required
        self.expected_harness_name = expected_harness_name
        self.expected_harness_count = expected_harness_count
        self.buffer = bytearray()
        self.current_harness: dict[str, Any] | None = None
        self.pending_count: int | None = None
        self.harnesses: list[dict[str, Any]] = []
        self.harness_digest = hashlib.sha256()
        self.summary_count = 0
        self.executed_tests = 0
        self.passed = 0
        self.failed = 0
        self.ignored = 0
        self.measured = 0
        self.filtered_out = 0
        self.failed_status = False
        self.errors: set[str] = set()

    def _error(self, code: str) -> None:
        # Every error is selected from a finite internal vocabulary. A set
        # prevents a bounded log full of repeated malformed lines from
        # becoming an unbounded Python list.
        self.errors.add(code)

    def _bounded_count(self, raw: str) -> int | None:
        if len(raw) > len(str(MAX_TEST_COUNT_PER_HARNESS)):
            self._error("test-harness-count-exceeds-bound")
            return None
        value = int(raw)
        if value > MAX_TEST_COUNT_PER_HARNESS:
            self._error("test-harness-count-exceeds-bound")
            return None
        return value

    def feed(self, chunk: bytes) -> None:
        if not self.required:
            return
        self.buffer.extend(chunk)
        if len(self.buffer) > MAX_TEST_LOG_LINE_BYTES and b"\n" not in self.buffer:
            self._error("test-log-line-exceeds-bound")
            self.buffer.clear()
            return
        while b"\n" in self.buffer:
            raw, _, rest = self.buffer.partition(b"\n")
            self.buffer = bytearray(rest)
            self._line(raw.rstrip(b"\r"))

    def _line(self, raw: bytes) -> None:
        if len(raw) > MAX_TEST_LOG_LINE_BYTES:
            self._error("test-log-line-exceeds-bound")
            return
        line = raw.decode("utf-8", errors="replace")
        harness_match = TEST_HARNESS_RE.fullmatch(line)
        if harness_match is not None:
            if self.current_harness is not None:
                self._error("target-harness-missing-summary")
            harness_path = Path(harness_match.group(1)).resolve(strict=False)
            if (
                harness_path != self.target
                and self.target not in harness_path.parents
            ):
                self.current_harness = None
                self.pending_count = None
                self._error("cargo-test-harness-outside-target")
                return
            basename = harness_path.name
            if self.expected_harness_name is not None:
                normalized = self.expected_harness_name.replace("-", "_")
                if not (
                    basename == normalized
                    or basename.startswith(f"{normalized}-")
                ):
                    self._error("unexpected-target-test-harness")
            self.current_harness = {
                "path_sha256": hashlib.sha256(
                    str(harness_path).encode("utf-8")
                ).hexdigest()
            }
            self.pending_count = None
            return
        running_match = TEST_RUNNING_RE.fullmatch(line.strip())
        if running_match is not None:
            if self.current_harness is None:
                self._error("running-count-without-target-harness")
                return
            if self.pending_count is not None:
                self._error("duplicate-running-count")
            self.pending_count = self._bounded_count(running_match.group(1))
            return
        summary_match = TEST_RESULT_RE.fullmatch(line.strip())
        if summary_match is None:
            return
        if self.current_harness is None or self.pending_count is None:
            self._error("test-summary-without-running-harness")
            return
        status = summary_match.group(1)
        parsed = [
            self._bounded_count(summary_match.group(index))
            for index in range(2, 7)
        ]
        if any(value is None for value in parsed):
            self.current_harness = None
            self.pending_count = None
            return
        passed, failed, ignored, measured, filtered = (
            int(value) for value in parsed
        )
        selected = passed + failed + ignored + measured
        if selected != self.pending_count:
            self._error("test-summary-running-count-mismatch")
            self.current_harness = None
            self.pending_count = None
            return
        if self.pending_count == 0:
            self._error("zero-selected-target-harness")
        summary = {
            **self.current_harness,
            "status": status,
            "running": self.pending_count,
            "passed": passed,
            "failed": failed,
            "ignored": ignored,
            "measured": measured,
            "filtered_out": filtered,
        }
        self.summary_count += 1
        if self.summary_count > MAX_TEST_HARNESS_SUMMARIES:
            self._error("test-harness-summary-count-exceeds-bound")
        self.harness_digest.update(canonical_json_bytes(summary))
        self.harness_digest.update(b"\n")
        if len(self.harnesses) < MAX_RETAINED_TEST_HARNESSES:
            self.harnesses.append(summary)
        self.executed_tests += passed + failed + measured
        self.passed += passed
        self.failed += failed
        self.ignored += ignored
        self.measured += measured
        self.filtered_out += filtered
        self.failed_status = self.failed_status or status == "FAILED"
        self.current_harness = None
        self.pending_count = None

    def finish(self) -> dict[str, Any]:
        if not self.required:
            return {"required": False, "decision": "NOT_APPLICABLE"}
        if self.buffer:
            self._line(bytes(self.buffer).rstrip(b"\r"))
            self.buffer.clear()
        if self.current_harness is not None or self.pending_count is not None:
            self._error("target-harness-missing-summary")
        if (
            self.expected_harness_count is not None
            and self.summary_count != self.expected_harness_count
        ):
            self._error("unexpected-target-harness-count")
        decision = (
            "PASS"
            if self.summary_count
            and self.executed_tests > 0
            and self.failed == 0
            and not self.failed_status
            and not self.errors
            else "BLOCK"
        )
        return {
            "required": True,
            "decision": decision,
            "summary_count": self.summary_count,
            "summary_limit": MAX_TEST_HARNESS_SUMMARIES,
            "executed_tests": self.executed_tests,
            "passed": self.passed,
            "failed": self.failed,
            "ignored": self.ignored,
            "measured": self.measured,
            "filtered_out": self.filtered_out,
            "parse_errors": sorted(self.errors),
            "harnesses_sha256": self.harness_digest.hexdigest(),
            "harnesses_retained": len(self.harnesses),
            "harnesses_truncated": (
                self.summary_count - len(self.harnesses)
            ),
            "harnesses": self.harnesses,
        }


def cargo_test_contract(command: list[str]) -> dict[str, Any]:
    selector_name: str | None = None
    selector_seen = False
    value_indices: set[int] = set()
    value_options = {
        "-j",
        "-p",
        "--bin",
        "--example",
        "--exclude",
        "--features",
        "--jobs",
        "--package",
        "--target",
        "--test",
    }
    for index, item in enumerate(command[2:], 2):
        if item in value_options and index + 1 < len(command):
            value_indices.add(index + 1)
        if item == "--lib":
            selector_seen = True
        if item in {"--bin", "--test"} and index + 1 < len(command):
            selector_seen = True
            selector_name = command[index + 1]
        elif item.startswith("--bin=") or item.startswith("--test="):
            selector_seen = True
            selector_name = item.split("=", 1)[1]
    separator = (
        command.index("--", 2) if "--" in command[2:] else len(command)
    )
    positional = [
        item
        for index, item in enumerate(command[2:separator], 2)
        if index not in value_indices and not item.startswith("-")
    ]
    harness_value_options = {
        "--color",
        "--format",
        "--skip",
        "--test-threads",
        "-Z",
    }
    harness_value_indices: set[int] = set()
    for index, item in enumerate(command[separator + 1 :], separator + 1):
        if item in harness_value_options and index + 1 < len(command):
            harness_value_indices.add(index + 1)
    harness_positional = [
        item
        for index, item in enumerate(command[separator + 1 :], separator + 1)
        if index not in harness_value_indices and not item.startswith("-")
    ]
    focused = positional[0] if positional else (
        harness_positional[0] if harness_positional else None
    )
    if focused is not None and not selector_seen:
        raise GateBlocked(
            "focused-test-requires-explicit-harness",
            "a filtered cargo test must select --lib, --bin, or --test",
        )
    return {
        "expected_harness_name": selector_name,
        "expected_harness_count": 1 if selector_seen else None,
        "focused_filter": focused,
        "minimum_tests_per_harness": 1,
    }


def toml_has_disabled_harness(raw: bytes) -> bool:
    text = raw.decode("utf-8", errors="strict")
    if tomllib is not None:
        payload = tomllib.loads(text)
        lib = payload.get("lib")
        if isinstance(lib, dict) and lib.get("harness") is False:
            return True
        for table_name in ("bin", "example", "test", "bench"):
            tables = payload.get(table_name, [])
            if isinstance(tables, list) and any(
                isinstance(table, dict)
                and table.get("harness") is False
                for table in tables
            ):
                return True
        return False

    # Python 3.10 has no stdlib TOML parser. This small conservative lexer
    # recognizes any `harness = false` key, including dotted and inline-table
    # forms, while skipping comments and quoted values. It intentionally
    # rejects the setting outside Cargo target tables rather than risk a
    # false admission.
    token_window: list[tuple[str, str]] = []

    def record_token(kind: str, value: str) -> bool:
        token_window.append((kind, value))
        if len(token_window) > 3:
            del token_window[0]
        if len(token_window) != 3:
            return False
        key_token, equals_token, value_token = token_window
        return (
            key_token[0] in {"bare", "string"}
            and key_token[1] == "harness"
            and equals_token == ("punctuation", "=")
            and value_token == ("bare", "false")
        )

    index = 0
    while index < len(text):
        character = text[index]
        if character.isspace():
            index += 1
            continue
        if character == "#":
            newline = text.find("\n", index + 1)
            index = len(text) if newline < 0 else newline + 1
            continue
        if character in {'"', "'"}:
            quote = character
            triple = text.startswith(quote * 3, index)
            if triple:
                raise ValueError(
                    "multiline TOML strings require tomllib for a "
                    "fail-closed harness audit"
                )
            delimiter = quote
            cursor = index + len(delimiter)
            content_start = cursor
            while cursor < len(text):
                if text.startswith(delimiter, cursor):
                    break
                if quote == '"' and text[cursor] == "\\":
                    cursor += 2
                else:
                    cursor += 1
            if cursor >= len(text):
                raise ValueError("unterminated TOML string")
            content = text[content_start:cursor]
            if quote == '"' and "\\" in content:
                raise ValueError(
                    "escaped TOML strings require tomllib for a "
                    "fail-closed harness audit"
                )
            if record_token("string", content):
                return True
            index = cursor + len(delimiter)
            continue
        if character.isalnum() or character in "_-":
            cursor = index + 1
            while cursor < len(text) and (
                text[cursor].isalnum() or text[cursor] in "_-"
            ):
                cursor += 1
            if record_token("bare", text[index:cursor]):
                return True
            index = cursor
            continue
        if record_token("punctuation", character):
            return True
        index += 1
    return False


def custom_harness_contract(
    snapshot: Path,
    snapshot_identity: Mapping[str, Any],
) -> dict[str, Any]:
    root_descriptor = open_directory_no_follow(snapshot)
    try:
        files = snapshot_file_manifest_at(root_descriptor)
        observed_manifest_sha256 = sha256_json(files)
        if (
            observed_manifest_sha256
            != snapshot_identity["snapshot_file_manifest_sha256"]
        ):
            raise GateBlocked(
                "snapshot-content-changed-before-harness-audit",
                observed_manifest_sha256,
            )
    finally:
        os.close(root_descriptor)

    rejected_count = 0
    rejected_digest = hashlib.sha256()
    rejected_sample: list[str] = []
    parsed_count = 0
    for item in files:
        relative = str(item["path"])
        if Path(relative).name != "Cargo.toml":
            continue
        root_descriptor = open_directory_no_follow(snapshot)
        parent: int | None = None
        descriptor: int | None = None
        try:
            parent = open_directory_path_at(
                root_descriptor,
                Path(relative).parts[:-1],
                create=False,
            )
            descriptor = os.open(
                Path(relative).name,
                os.O_RDONLY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
                dir_fd=parent,
            )
            before = os.fstat(descriptor)
            chunks: list[bytes] = []
            size = 0
            while True:
                chunk = os.read(descriptor, 1024 * 1024)
                if not chunk:
                    break
                chunks.append(chunk)
                size += len(chunk)
                if size > 8 * 1024 * 1024:
                    raise GateBlocked(
                        "cargo-manifest-too-large-for-harness-audit",
                        relative,
                    )
            after = os.fstat(descriptor)
            raw = b"".join(chunks)
            if (
                not stat.S_ISREG(before.st_mode)
                or before.st_nlink != 1
                or (
                    before.st_dev,
                    before.st_ino,
                    before.st_size,
                    before.st_mtime_ns,
                    before.st_ctime_ns,
                )
                != (
                    after.st_dev,
                    after.st_ino,
                    after.st_size,
                    after.st_mtime_ns,
                    after.st_ctime_ns,
                )
                or size != after.st_size
                or size != item["bytes"]
                or hashlib.sha256(raw).hexdigest() != item["sha256"]
            ):
                raise GateBlocked(
                    "cargo-manifest-changed-during-harness-audit",
                    relative,
                )
        finally:
            if descriptor is not None:
                os.close(descriptor)
            if parent is not None:
                os.close(parent)
            os.close(root_descriptor)
        try:
            disabled_harness = toml_has_disabled_harness(raw)
        except (UnicodeDecodeError, ValueError) as exc:
            raise GateBlocked(
                "cargo-manifest-harness-audit-failed", relative
            ) from exc
        parsed_count += 1
        if disabled_harness:
            rejected_count += 1
            raw_relative = relative.encode(
                "utf-8", errors="surrogatepass"
            )
            rejected_digest.update(
                len(raw_relative).to_bytes(4, "big")
            )
            rejected_digest.update(raw_relative)
            if len(rejected_sample) < 8:
                rejected_sample.append(relative)
    if rejected_count:
        raise GateBlocked(
            "cargo-custom-test-harness-not-admitted",
            f"count={rejected_count}, "
            f"sha256={rejected_digest.hexdigest()}, "
            f"sample={rejected_sample!r}",
        )
    return {
        "decision": "STANDARD_LIBTEST_HARNESSES_ONLY",
        "manifest_count": parsed_count,
        "snapshot_file_manifest_sha256": observed_manifest_sha256,
        "rust_source_manifest_sha256": snapshot_identity[
            "rust_source_manifest_sha256"
        ],
    }


def run_source_keyed_gate(
    command: list[str],
    candidate: Path,
    log_path: Path,
    timeout_seconds: float,
    *,
    environment: Mapping[str, str] | None = None,
    profile: Mapping[str, Any] | None = None,
    lock_timeout_seconds: float = 6 * 60 * 60,
    ephemeral: bool = False,
    persistent_aggregate_budget_bytes: int = (
        DEFAULT_PERSISTENT_AGGREGATE_BUDGET_BYTES
    ),
    new_namespace_allocation_reserve_bytes: int = (
        DEFAULT_NEW_NAMESPACE_ALLOCATION_RESERVE_BYTES
    ),
    new_namespace_free_floor_bytes: int = (
        DEFAULT_NEW_NAMESPACE_FREE_FLOOR_BYTES
    ),
    export_artifact_relative: str | None = None,
    export_path: Path | None = None,
    gate_runner: Callable[..., dict[str, Any]] = run_gate,
) -> dict[str, Any]:
    if not math.isfinite(timeout_seconds) or timeout_seconds <= 0:
        raise GateBlocked("invalid-gate-timeout", str(timeout_seconds))
    if (
        not math.isfinite(lock_timeout_seconds)
        or lock_timeout_seconds < 0
    ):
        raise GateBlocked("invalid-lock-timeout", str(lock_timeout_seconds))
    if (export_artifact_relative is None) != (export_path is None):
        raise GateBlocked(
            "incomplete-artifact-export-request",
            "artifact relative path and export path must be provided together",
        )
    if ephemeral and export_artifact_relative is not None:
        raise GateBlocked(
            "ephemeral-artifact-export-not-admitted",
            "artifact export requires a retained source-key namespace",
        )
    expected_profile = dict(DEFAULT_PROFILE if profile is None else profile)
    root = repository_root(candidate)
    admitted_root = validate_admitted_root(expected_profile)
    source_environment = os.environ if environment is None else environment
    gate_environment, ambient_target = canonical_profile_environment(
        source_environment,
        cwd=root,
        profile=expected_profile,
    )
    normalized_command, subcommand = normalize_cargo_argv(
        command, profile=expected_profile
    )
    test_contract = (
        cargo_test_contract(normalized_command)
        if subcommand == "test" and "--no-run" not in normalized_command
        else {
            "expected_harness_name": None,
            "expected_harness_count": None,
            "focused_filter": None,
            "minimum_tests_per_harness": None,
        }
    )
    toolchain = probe_toolchain(expected_profile, subcommand)
    isolation = isolation_identity()
    toolchain_bin = str(
        Path(toolchain["toolchain_root"]) / "bin"
    )
    existing_path = gate_environment.get("PATH", "")
    gate_environment["PATH"] = os.pathsep.join(
        [
            toolchain_bin,
            *[
                item
                for item in existing_path.split(os.pathsep)
                if item and Path(item).resolve(strict=False) != Path(toolchain_bin)
            ],
        ]
    )
    gate_environment["RUSTC"] = str(
        toolchain["executables"]["rustc"]["path"]
    )
    gate_environment["RUSTDOC"] = str(
        toolchain["executables"]["rustdoc"]["path"]
    )
    gate_environment["CARGO"] = str(
        toolchain["executables"]["cargo"]["path"]
    )
    if subcommand == "clippy":
        gate_environment["CLIPPY_DRIVER_PATH"] = str(
            toolchain["executables"]["clippy-driver"]["path"]
        )
        gate_environment["RUSTC_WORKSPACE_WRAPPER"] = str(
            toolchain["executables"]["clippy-driver"]["path"]
        )
    if subcommand == "fmt":
        gate_environment["RUSTFMT"] = str(
            toolchain["executables"]["rustfmt"]["path"]
        )
    runtime_environment_contract = stable_runtime_environment_contract(
        gate_environment, toolchain
    )
    environment_sha256 = artifact_environment_sha256(
        runtime_environment_contract
    )
    # Capture the full dirty identity rather than the fast clean-only form.
    # Cleanliness is an admission predicate with its own blocker below, and
    # the same complete identity has to be recapturable after the gate for
    # the candidate-unchanged comparison to mean anything.
    before = capture_source_state(root)
    log = evidence_path(
        log_path,
        candidate_root=root,
        admitted_root=admitted_root,
        git_common_dir=Path(before["git_common_dir"]),
        git_dir=Path(before["git_dir"]),
        profile=expected_profile,
        label="gate log",
    )
    if before["dirty"]:
        raise GateBlocked(
            "dirty-candidate-not-trusted",
            "trusted Cargo evidence requires a clean candidate materialized "
            "from exact HEAD Git blobs; dirty candidates are not executed",
        )
    export_destination_contract: dict[str, Any] | None = None
    if export_path is not None:
        export_destination_contract = (
            artifact_export_destination_contract(
                root, Path(str(export_path))
            )
        )
    source_key, _source_payload = build_source_identity(
        before,
        environment_sha256=environment_sha256,
        profile=expected_profile,
        toolchain=toolchain,
        isolation=isolation,
    )
    artifact_key, artifact_payload = build_artifact_identity(
        before,
        environment_sha256=environment_sha256,
        profile=expected_profile,
        toolchain=toolchain,
        isolation=isolation,
    )
    target = namespace_path(admitted_root, artifact_key)
    validate_ambient_target(
        ambient_target,
        cwd=root,
        admitted_root=admitted_root,
        expected_target=target,
    )
    gate_environment["CARGO_TARGET_DIR"] = str(target)
    gate_environment["CODEFIRE_CARGO_GATE_SOURCE_KEY"] = artifact_key
    cargo_command = [
        str(toolchain["executables"]["cargo"]["path"]),
        *normalized_command[1:],
    ]
    primary_tool_name = {
        "clippy": "cargo-clippy",
        "fmt": "cargo-fmt",
    }.get(subcommand, "cargo")
    synthetic_profile = dict(expected_profile)
    profile_violations = evaluate_process(
        {
            "cwd": str(root),
            "argv": cargo_command,
            "environment": gate_environment,
        },
        synthetic_profile,
    )
    if profile_violations:
        raise GateBlocked(
            "execution-profile-rejected",
            json.dumps(profile_violations, sort_keys=True, separators=(",", ":")),
        )

    gate_execution_key = execution_key(
        source_key,
        normalized_command,
        timeout_seconds,
        environment_sha256,
    )
    prefix_payload = {
        "schema": LOG_PREFIX_SCHEMA,
        "source_key": source_key,
        "artifact_key": artifact_key,
        "execution_key": gate_execution_key,
        "candidate_root": before["candidate_root"],
        "head_revision": before["head_revision"],
        "tree": before["tree"],
        "dirty_digest": before["dirty_digest"],
    }
    log_prefix = canonical_json_bytes(prefix_payload) + b"\n"

    with cargo_registry_locks(
        admitted_root, lock_timeout_seconds
    ) as (global_execution_wait, registry_wait):
        (
            target,
            expected_marker,
            marker_sha256,
            _,
            retention_admission,
        ) = prepare_namespace(
            admitted_root,
            artifact_key,
            artifact_payload,
            lock_timeout_seconds=lock_timeout_seconds,
            ephemeral=ephemeral,
            persistent_aggregate_budget_bytes=(
                persistent_aggregate_budget_bytes
            ),
            new_namespace_allocation_reserve_bytes=(
                new_namespace_allocation_reserve_bytes
            ),
            new_namespace_free_floor_bytes=(
                new_namespace_free_floor_bytes
            ),
            registry_lock_already_held=True,
            registry_lock_wait_seconds=registry_wait,
        )
        running_transitioned = False

        def cleanup_running_namespace_on_error() -> None:
            if not running_transitioned or not target.exists():
                return
            invalidation_error: BaseException | None = None
            if not ephemeral:
                try:
                    invalidate_namespace(
                        target,
                        source_key=artifact_key,
                        reason="namespace-lock-body-exception",
                        before=before,
                        after=None,
                    )
                except BaseException as exc:
                    invalidation_error = exc
            try:
                cleanup_namespace_locked(
                    admitted_root, target, expected_marker
                )
            except BaseException as cleanup_error:
                detail = (
                    f"namespace:{type(cleanup_error).__name__}:"
                    f"{cleanup_error}"
                )
                if invalidation_error is not None:
                    detail += (
                        f"; invalidation:"
                        f"{type(invalidation_error).__name__}:"
                        f"{invalidation_error}"
                    )
                raise GateBlocked(
                    "post-admission-cleanup-failed", detail
                ) from cleanup_error

        with exclusive_lock(
            target / NAMESPACE_LOCK_NAME,
            lock_timeout_seconds,
            allow_parent_removal=True,
            on_error=cleanup_running_namespace_on_error,
        ) as namespace_wait:
            current_marker_sha256 = validate_marker(
                target / MARKER_NAME, expected_marker
            )
            if current_marker_sha256 != marker_sha256:
                raise GateBlocked(
                    "source-marker-changed",
                    f"marker changed before execution: {target / MARKER_NAME}",
                )
            prior_run_state = read_run_state(
                target, source_key=artifact_key
            )
            if (
                prior_run_state is None
                or prior_run_state.get("state")
                not in {"PREPARED", "COMMITTED"}
            ):
                raise GateBlocked(
                    "namespace-not-ready-for-execution",
                    str(target / RUN_STATE_NAME),
                )
            running_state_sha256 = write_run_state(
                target,
                source_key=artifact_key,
                state="RUNNING",
                execution_key_value=gate_execution_key,
            )
            running_transitioned = True
            committed_state_sha256: str | None = None
            snapshot: Path | None = None
            snapshot_identity: dict[str, Any] | None = None
            snapshot_verification: dict[str, Any] | None = None
            snapshot_cleanup: dict[str, Any] | None = None
            snapshot_root_recovery: dict[str, Any] | None = None
            artifact_export: dict[str, Any] | None = None
            runtime_scratch: dict[str, Any] | None = None
            runtime_scratch_cleanup: dict[str, Any] | None = None
            try:
                if capture_source_state(root) != before:
                    raise GateBlocked(
                        "candidate-changed-before-gate",
                        "candidate identity changed while waiting for the "
                        "exclusive locks",
                    )
                snapshot_root_recovery = recover_snapshot_root(
                    Path(str(expected_profile["snapshot_root"]))
                )
                snapshot, snapshot_identity = create_clean_git_snapshot(
                    root,
                    before,
                    source_key,
                    expected_profile,
                    free_floor_bytes=new_namespace_free_floor_bytes,
                )
                if subcommand == "test":
                    test_contract["custom_harness_audit"] = (
                        custom_harness_contract(
                            snapshot, snapshot_identity
                        )
                    )
                runtime_scratch = prepare_runtime_scratch(target)
                with pinned_tool_files(toolchain, isolation) as tool_pins, (
                    pinned_execution_directories(
                        {
                            "directory:admitted-root": admitted_root,
                            "directory:evidence-root": Path(
                                str(expected_profile["evidence_root"])
                            ),
                            "directory:source": root,
                            "directory:snapshot": snapshot,
                            "directory:snapshot-root": snapshot.parent,
                            "directory:runtime-home": Path(
                                str(runtime_scratch["home"]["path"])
                            ),
                            "directory:runtime-scratch": Path(
                                str(runtime_scratch["root"])
                            ),
                            "directory:runtime-tmp": Path(
                                str(runtime_scratch["tmp"]["path"])
                            ),
                            "directory:toolchain": Path(
                                str(toolchain["toolchain_root"])
                            ),
                            "directory:target": target,
                        }
                    )
                ) as directory_pins, pinned_control_files(
                    {
                        "control:target-marker": target / MARKER_NAME,
                        "control:target-lock": target / NAMESPACE_LOCK_NAME,
                        "control:target-run-state": target / RUN_STATE_NAME,
                    }
                ) as control_pins:
                    pinned = {
                        "descriptors": {
                            **tool_pins["descriptors"],
                            **directory_pins["descriptors"],
                            **control_pins["descriptors"],
                        },
                        "pass_fds": tuple(
                            sorted(
                                {
                                    *tool_pins["pass_fds"],
                                    *directory_pins["pass_fds"],
                                    *control_pins["pass_fds"],
                                }
                            )
                        ),
                    }
                    runtime_environment = dict(gate_environment)
                    runtime_tool_descriptors = {
                        "CARGO": "cargo",
                        "RUSTC": "rustc",
                        "RUSTDOC": "rustdoc",
                    }
                    if subcommand == "clippy":
                        runtime_tool_descriptors.update(
                            {
                                "CLIPPY_DRIVER_PATH": (
                                    "clippy-driver"
                                ),
                                "RUSTC_WORKSPACE_WRAPPER": (
                                    "clippy-driver"
                                ),
                            }
                        )
                    if subcommand == "fmt":
                        runtime_tool_descriptors["RUSTFMT"] = "rustfmt"
                    for environment_name, tool_name in (
                        runtime_tool_descriptors.items()
                    ):
                        runtime_environment[environment_name] = (
                            "/proc/self/fd/"
                            f"{pinned['descriptors'][f'toolchain:{tool_name}']}"
                        )
                    runtime_environment["HOME"] = str(
                        runtime_scratch["home"]["path"]
                    )
                    runtime_environment["TMPDIR"] = str(
                        runtime_scratch["tmp"]["path"]
                    )
                    observed_runtime_environment_contract = (
                        project_runtime_environment(
                            runtime_environment,
                            target=target,
                            artifact_key=artifact_key,
                            runtime_scratch=runtime_scratch,
                            toolchain=toolchain,
                            descriptors=pinned["descriptors"],
                        )
                    )
                    if (
                        observed_runtime_environment_contract
                        != runtime_environment_contract
                    ):
                        raise GateBlocked(
                            "runtime-environment-contract-mismatch",
                            artifact_key,
                        )
                    analyzer = CargoTestLogAnalyzer(
                        target,
                        required=(
                            subcommand == "test"
                            and "--no-run" not in normalized_command
                        ),
                        expected_harness_name=test_contract[
                            "expected_harness_name"
                        ],
                        expected_harness_count=test_contract[
                            "expected_harness_count"
                        ],
                    )
                    launch_argv = isolated_command(
                        root,
                        snapshot,
                        snapshot_identity,
                        admitted_root,
                        Path(str(expected_profile["evidence_root"])),
                        target,
                        cargo_command,
                        primary_tool_name,
                        toolchain,
                        isolation,
                        pinned,
                    )
                    aggregate_other_namespaces = max(
                        0,
                        int(
                            retention_admission[
                                "persistent_aggregate_allocated_bytes_before"
                            ]
                        )
                        - int(
                            retention_admission[
                                "current_namespace_actual_allocated_bytes_before"
                            ]
                        ),
                    )
                    live_namespace_limit = max(
                        new_namespace_allocation_reserve_bytes,
                        int(
                            retention_admission[
                                "current_namespace_actual_allocated_bytes_before"
                            ]
                        )
                        + new_namespace_allocation_reserve_bytes,
                    )

                    def runtime_resource_guard() -> None:
                        live_usage: dict[str, int] | None = None
                        for _attempt in range(2):
                            try:
                                live_usage = namespace_usage(
                                    target,
                                    live_deadline_seconds=(
                                        LIVE_NAMESPACE_SCAN_SECONDS
                                    ),
                                )
                                break
                            except FileNotFoundError:
                                continue
                        if live_usage is None:
                            raise GateBlocked(
                                "live-target-usage-unavailable",
                                str(target),
                            )
                        live_allocated = int(
                            live_usage["allocated_bytes"]
                        )
                        if live_allocated > live_namespace_limit:
                            raise GateBlocked(
                                "live-namespace-allocation-limit-exceeded",
                                f"observed={live_allocated}, "
                                f"limit={live_namespace_limit}",
                            )
                        live_aggregate = (
                            aggregate_other_namespaces + live_allocated
                        )
                        if (
                            live_aggregate
                            > persistent_aggregate_budget_bytes
                        ):
                            raise GateBlocked(
                                "live-persistent-aggregate-budget-exceeded",
                                f"observed={live_aggregate}, "
                                f"budget={persistent_aggregate_budget_bytes}",
                            )
                        live_free = shutil.disk_usage(admitted_root).free
                        if live_free < new_namespace_free_floor_bytes:
                            raise GateBlocked(
                                "live-filesystem-free-floor-crossed",
                                f"free={live_free}, "
                                f"floor={new_namespace_free_floor_bytes}",
                            )

                    gate = gate_runner(
                        launch_argv,
                        root,
                        log,
                        timeout_seconds,
                        environment=runtime_environment,
                        log_prefix=log_prefix,
                        log_binding_prefix=(
                            source_key.encode("ascii") + b"\0"
                        ),
                        log_analyzer=analyzer,
                        pass_fds=pinned["pass_fds"],
                        runtime_guard=runtime_resource_guard,
                        pid_namespace_isolated=True,
                    )
                    snapshot_verification = verify_snapshot_contents(
                        snapshot, snapshot_identity
                    )
                    post_gate_marker_sha256 = validate_marker(
                        target / MARKER_NAME, expected_marker
                    )
                    if post_gate_marker_sha256 != marker_sha256:
                        raise GateBlocked(
                            "source-marker-changed-during-gate",
                            str(target / MARKER_NAME),
                        )
                runtime_scratch_cleanup = cleanup_runtime_scratch(
                    target, runtime_scratch
                )
                # A gate that dirtied the candidate is a BLOCK with
                # candidate-changed-during-gate evidence, not a raised
                # admission error: the result has to record what the
                # candidate became, so the recapture must tolerate dirt.
                after = capture_source_state(root)
                snapshot_cleanup = cleanup_snapshot(
                    snapshot, snapshot_identity
                )
                snapshot = None
            except BaseException as exc:
                cleanup_errors: list[str] = []
                if (
                    snapshot is not None
                    and snapshot_identity is not None
                    and snapshot.exists()
                ):
                    try:
                        cleanup_snapshot(snapshot, snapshot_identity)
                    except BaseException as cleanup_exc:
                        cleanup_errors.append(
                            f"snapshot:{type(cleanup_exc).__name__}:"
                            f"{cleanup_exc}"
                        )
                if not ephemeral and target.exists():
                    try:
                        invalidate_namespace(
                            target,
                            source_key=artifact_key,
                            reason=(
                                "post-admission-exception:"
                                f"{type(exc).__name__}"
                            ),
                            before=before,
                            after=None,
                        )
                    except BaseException as cleanup_exc:
                        cleanup_errors.append(
                            f"invalidate:{type(cleanup_exc).__name__}:"
                            f"{cleanup_exc}"
                        )
                if target.exists():
                    try:
                        cleanup_namespace_locked(
                            admitted_root, target, expected_marker
                        )
                    except BaseException as cleanup_exc:
                        cleanup_errors.append(
                            f"namespace:{type(cleanup_exc).__name__}:"
                            f"{cleanup_exc}"
                        )
                if cleanup_errors:
                    raise GateBlocked(
                        "post-admission-cleanup-failed",
                        "; ".join(cleanup_errors),
                    ) from exc
                raise

            aggregate_before = int(
                retention_admission[
                    "persistent_aggregate_allocated_bytes_before"
                ]
            )
            current_before_bytes = int(
                retention_admission[
                    "current_namespace_actual_allocated_bytes_before"
                ]
            )
            usage = namespace_usage(target)
            measured_aggregate = (
                aggregate_before
                - current_before_bytes
                + int(usage["allocated_bytes"])
            )
            free_before_retention = shutil.disk_usage(admitted_root).free
            post_policy_codes: list[str] = []
            if measured_aggregate > persistent_aggregate_budget_bytes:
                post_policy_codes.append(
                    "post-gate-persistent-aggregate-budget-exceeded"
                )
            if free_before_retention < new_namespace_free_floor_bytes:
                post_policy_codes.append(
                    "post-gate-filesystem-free-floor-crossed"
                )

            if after != before:
                if ephemeral:
                    try:
                        retention = cleanup_namespace_locked(
                            admitted_root, target, expected_marker
                        )
                    except (GateBlocked, OSError) as exc:
                        retention = {
                            "decision": "CLEANUP_FAILED",
                            "error": str(exc),
                            "namespace_exists_after": target.exists(),
                        }
                else:
                    retention = invalidate_namespace(
                        target,
                        source_key=artifact_key,
                        reason="candidate-changed-during-gate",
                        before=before,
                        after=after,
                    )
                    retention.update(usage)
            elif ephemeral:
                try:
                    retention = cleanup_namespace_locked(
                        admitted_root, target, expected_marker
                    )
                except (GateBlocked, OSError) as exc:
                    retention = {
                        "decision": "CLEANUP_FAILED",
                        "error": str(exc),
                        "namespace_exists_after": target.exists(),
                    }
            elif (
                post_policy_codes
                and not retention_admission["namespace_preexisting"]
            ):
                try:
                    cleanup = cleanup_namespace_locked(
                        admitted_root, target, expected_marker
                    )
                    retention = {
                        **cleanup,
                        "decision": "POST_POLICY_EXCEEDED_CLEANED",
                        "post_policy_blockers": post_policy_codes,
                        "measured_allocated_bytes_before_cleanup": usage[
                            "allocated_bytes"
                        ],
                        "measured_logical_bytes_before_cleanup": usage[
                            "logical_bytes"
                        ],
                    }
                except (GateBlocked, OSError) as exc:
                    retention = {
                        "decision": "POST_POLICY_CLEANUP_FAILED",
                        "post_policy_blockers": post_policy_codes,
                        "error": str(exc),
                        **usage,
                        "namespace_exists_after": target.exists(),
                    }
            elif post_policy_codes:
                try:
                    retention_sha256 = write_retention_record(
                        target,
                        source_key=artifact_key,
                        usage=usage,
                    )
                    retention = {
                        "decision": "POST_POLICY_EXCEEDED_RETAINED",
                        "post_policy_blockers": post_policy_codes,
                        **usage,
                        "record_sha256": retention_sha256,
                        "namespace_exists_after": True,
                    }
                except (GateBlocked, OSError) as exc:
                    retention = {
                        "decision": "POST_POLICY_RETENTION_RECORD_FAILED",
                        "post_policy_blockers": post_policy_codes,
                        "error": str(exc),
                        **usage,
                        "namespace_exists_after": target.exists(),
                    }
            else:
                try:
                    usage = namespace_usage(target)
                    retention_sha256 = write_retention_record(
                        target,
                        source_key=artifact_key,
                        usage=usage,
                    )
                    retention = {
                        "decision": "PERSISTED",
                        **usage,
                        "record_sha256": retention_sha256,
                        "namespace_exists_after": True,
                    }
                except (GateBlocked, OSError) as exc:
                    retention = {
                        "decision": "RETENTION_RECORD_FAILED",
                        "error": str(exc),
                        "namespace_exists_after": target.exists(),
                    }

            fresh_inventory = inspect_registry(target.parent)
            fresh_aggregate = sum(
                int(item["actual_allocated_bytes"])
                for item in fresh_inventory
            )
            fresh_free = shutil.disk_usage(admitted_root).free
            fresh_policy_codes: list[str] = []
            fresh_current = next(
                (
                    item
                    for item in fresh_inventory
                    if item["source_key"] == artifact_key
                ),
                None,
            )
            if not ephemeral and (
                fresh_current is None
                or fresh_current.get("invalidated") is True
            ):
                fresh_policy_codes.append(
                    "final-current-namespace-invalidated"
                )
            if fresh_aggregate > persistent_aggregate_budget_bytes:
                fresh_policy_codes.append(
                    "final-persistent-aggregate-budget-exceeded"
                )
            if fresh_free < new_namespace_free_floor_bytes:
                fresh_policy_codes.append(
                    "final-filesystem-free-floor-crossed"
                )
            for code in fresh_policy_codes:
                if code not in post_policy_codes:
                    post_policy_codes.append(code)
            measured_aggregate = fresh_aggregate
            if fresh_policy_codes and target.exists():
                if not retention_admission["namespace_preexisting"]:
                    cleanup = cleanup_namespace_locked(
                        admitted_root, target, expected_marker
                    )
                    retention = {
                        **cleanup,
                        "decision": "POST_POLICY_EXCEEDED_CLEANED",
                        "post_policy_blockers": list(post_policy_codes),
                        "measured_allocated_bytes_before_cleanup": usage[
                            "allocated_bytes"
                        ],
                        "measured_logical_bytes_before_cleanup": usage[
                            "logical_bytes"
                        ],
                    }
                    measured_aggregate = sum(
                        int(item["actual_allocated_bytes"])
                        for item in inspect_registry(target.parent)
                    )
                else:
                    retention = {
                        **retention,
                        "decision": "POST_POLICY_EXCEEDED_RETAINED",
                        "post_policy_blockers": list(post_policy_codes),
                        "namespace_exists_after": True,
                    }

            in_lock_log = gate.get("log", {})
            in_lock_log_matches = (
                in_lock_log.get("path") == str(log)
                and isinstance(in_lock_log.get("bytes"), int)
                and in_lock_log.get("bytes", -1) >= len(log_prefix)
                and re.fullmatch(
                    r"[0-9a-f]{64}",
                    str(in_lock_log.get("sha256")),
                )
                is not None
                and re.fullmatch(
                    r"[0-9a-f]{64}",
                    str(in_lock_log.get("bound_sha256")),
                )
                is not None
                and in_lock_log.get("complete") is True
            )
            in_lock_analysis = gate.get("log_analysis")
            in_lock_analysis_passed = (
                not isinstance(in_lock_analysis, dict)
                or in_lock_analysis.get("decision") != "BLOCK"
            )
            in_lock_runtime_guard = gate.get("runtime_guard")
            in_lock_runtime_guard_passed = (
                isinstance(in_lock_runtime_guard, dict)
                and in_lock_runtime_guard.get("enabled") is True
                and in_lock_runtime_guard.get("decision") == "PASS"
            )
            commit_ready = (
                gate.get("status") == "passed"
                and gate.get("exit_code") == 0
                and not gate.get("timed_out", False)
                and after == before
                and in_lock_log_matches
                and in_lock_analysis_passed
                and in_lock_runtime_guard_passed
                and not post_policy_codes
                and retention["decision"] == "PERSISTED"
            )
            if commit_ready:
                committed_state_sha256 = write_run_state(
                    target,
                    source_key=artifact_key,
                    state="COMMITTED",
                    execution_key_value=gate_execution_key,
                )
                committed_state = read_run_state(
                    target, source_key=artifact_key
                )
                if (
                    committed_state is None
                    or committed_state.get("state") != "COMMITTED"
                    or committed_state.get("execution_key")
                    != gate_execution_key
                ):
                    raise GateBlocked(
                        "namespace-run-state-commit-failed",
                        str(target / RUN_STATE_NAME),
                    )
                post_commit_inventory = inspect_registry(target.parent)
                post_commit_aggregate = sum(
                    int(item["actual_allocated_bytes"])
                    for item in post_commit_inventory
                )
                post_commit_free = shutil.disk_usage(admitted_root).free
                measured_aggregate = post_commit_aggregate
                post_commit_policy_codes: list[str] = []
                if (
                    post_commit_aggregate
                    > persistent_aggregate_budget_bytes
                ):
                    post_commit_policy_codes.append(
                        "post-commit-persistent-aggregate-budget-exceeded"
                    )
                if post_commit_free < new_namespace_free_floor_bytes:
                    post_commit_policy_codes.append(
                        "post-commit-filesystem-free-floor-crossed"
                    )
                if post_commit_policy_codes:
                    post_policy_codes.extend(
                        code
                        for code in post_commit_policy_codes
                        if code not in post_policy_codes
                    )
                    write_run_state(
                        target,
                        source_key=artifact_key,
                        state="RUNNING",
                        execution_key_value=gate_execution_key,
                    )
                    committed_state_sha256 = None
                    if not retention_admission["namespace_preexisting"]:
                        cleanup = cleanup_namespace_locked(
                            admitted_root, target, expected_marker
                        )
                        retention = {
                            **cleanup,
                            "decision": "POST_POLICY_EXCEEDED_CLEANED",
                            "post_policy_blockers": list(
                                post_policy_codes
                            ),
                            "measured_allocated_bytes_before_cleanup": (
                                usage["allocated_bytes"]
                            ),
                            "measured_logical_bytes_before_cleanup": (
                                usage["logical_bytes"]
                            ),
                        }
                    else:
                        retention = {
                            **retention,
                            "decision": "POST_POLICY_EXCEEDED_RETAINED",
                            "post_policy_blockers": list(
                                post_policy_codes
                            ),
                            "namespace_exists_after": True,
                        }
                elif export_artifact_relative is not None:
                    try:
                        current_export_contract = (
                            artifact_export_destination_contract(
                                root, Path(str(export_path))
                            )
                        )
                        if (
                            current_export_contract
                            != export_destination_contract
                        ):
                            raise GateBlocked(
                                "artifact-export-destination-contract-changed",
                                str(export_path),
                            )
                        artifact_export = export_bound_artifact(
                            target,
                            export_artifact_relative,
                            root,
                            Path(str(export_path)),
                            free_floor_bytes=(
                                new_namespace_free_floor_bytes
                            ),
                        )
                        artifact_export["destination_contract"] = (
                            current_export_contract
                        )
                    except BaseException:
                        write_run_state(
                            target,
                            source_key=artifact_key,
                            state="RUNNING",
                            execution_key_value=gate_execution_key,
                        )
                        committed_state_sha256 = None
                        cleanup_namespace_locked(
                            admitted_root, target, expected_marker
                        )
                        raise
            if (
                export_artifact_relative is not None
                and artifact_export is None
            ):
                artifact_export = {
                    "decision": "NOT_EXPORTED",
                    "reason": "final-pass-predicates-not-satisfied",
                }
            if (
                not ephemeral
                and committed_state_sha256 is None
                and target.exists()
            ):
                incomplete_cleanup = cleanup_namespace_locked(
                    admitted_root, target, expected_marker
                )
                retention = {
                    **incomplete_cleanup,
                    "decision": "INCOMPLETE_GATE_CLEANED",
                    "namespace_exists_after": False,
                }
                measured_aggregate = sum(
                    int(item["actual_allocated_bytes"])
                    for item in inspect_registry(target.parent)
                )
            if retention["decision"] == "PERSISTED":
                retention[
                    "persistent_aggregate_allocated_bytes_after"
                ] = measured_aggregate
            elif retention["decision"] == "EPHEMERAL_CLEANED":
                retention[
                    "persistent_aggregate_allocated_bytes_after"
                ] = aggregate_before - current_before_bytes
            elif retention["decision"] == "POST_POLICY_EXCEEDED_CLEANED":
                retention[
                    "persistent_aggregate_allocated_bytes_after"
                ] = aggregate_before
            else:
                retention[
                    "persistent_aggregate_allocated_bytes_after"
                ] = measured_aggregate
            retention[
                "persistent_aggregate_allocated_bytes_measured_after_gate"
            ] = measured_aggregate
            retention["filesystem_free_bytes_after"] = shutil.disk_usage(
                admitted_root
            ).free
            retention["final_registry_allocated_bytes"] = (
                measured_aggregate
            )
            retention["run_state_before_sha256"] = (
                running_state_sha256
            )
            retention["run_state_committed_sha256"] = (
                committed_state_sha256
            )

    gate_log = gate.get("log", {})
    log_evidence_matches = (
        gate_log.get("path") == str(log)
        and isinstance(gate_log.get("bytes"), int)
        and gate_log.get("bytes", -1) >= len(log_prefix)
        and re.fullmatch(r"[0-9a-f]{64}", str(gate_log.get("sha256")))
        is not None
        and re.fullmatch(
            r"[0-9a-f]{64}", str(gate_log.get("bound_sha256"))
        )
        is not None
        and gate_log.get("complete") is True
    )
    test_summary = gate.get("log_analysis")
    if not isinstance(test_summary, dict):
        test_summary = {
            "required": subcommand == "test",
            "decision": "BLOCK",
            "parse_errors": ["missing-pinned-log-analysis"],
        }
    candidate_unchanged = after == before
    gate_passed = (
        gate.get("status") == "passed"
        and gate.get("exit_code") == 0
        and not gate.get("timed_out", False)
    )
    runtime_guard_evidence = gate.get("runtime_guard")
    runtime_guard_passed = (
        isinstance(runtime_guard_evidence, dict)
        and runtime_guard_evidence.get("enabled") is True
        and runtime_guard_evidence.get("decision") == "PASS"
    )
    passed = (
        gate_passed
        and runtime_guard_passed
        and candidate_unchanged
        and log_evidence_matches
        and test_summary["decision"] != "BLOCK"
        and not post_policy_codes
        and retention["decision"] in {"PERSISTED", "EPHEMERAL_CLEANED"}
        and (ephemeral or committed_state_sha256 is not None)
        and (
            export_artifact_relative is None
            or (
                artifact_export is not None
                and artifact_export.get("decision") == "EXPORTED"
            )
        )
    )
    blockers: list[str] = []
    if not gate_passed:
        blockers.append("cargo-gate-failed")
    if not runtime_guard_passed:
        blockers.append("runtime-resource-guard-not-passed")
    if not candidate_unchanged:
        blockers.append("candidate-changed-during-gate")
    if not log_evidence_matches:
        blockers.append("timed-gate-log-evidence-mismatch")
    if test_summary["decision"] == "BLOCK":
        if test_summary.get("parse_errors"):
            blockers.append("cargo-test-log-not-correlated")
        if int(test_summary.get("failed", 0)) > 0:
            blockers.append("cargo-test-summary-failed")
        if int(test_summary.get("executed_tests", 0)) == 0:
            blockers.append("cargo-test-executed-zero-tests")
    blockers.extend(post_policy_codes)
    if retention["decision"] not in {"PERSISTED", "EPHEMERAL_CLEANED"}:
        blockers.append("target-retention-or-cleanup-failed")
    if not ephemeral and committed_state_sha256 is None:
        blockers.append("target-run-state-not-committed")
    if export_artifact_relative is not None and (
        artifact_export is None
        or artifact_export.get("decision") != "EXPORTED"
    ):
        blockers.append("requested-artifact-not-exported")

    return {
        "schema": RESULT_SCHEMA,
        "decision": "PASS" if passed else "BLOCK",
        "blockers": blockers,
        "source_key": source_key,
        "artifact_key": artifact_key,
        "artifact_key_payload": artifact_payload,
        "execution_key": gate_execution_key,
        "candidate": before,
        "candidate_after": after,
        "candidate_unchanged": candidate_unchanged,
        "artifact_environment_sha256": environment_sha256,
        "runtime_environment_contract": (
            runtime_environment_contract
        ),
        "execution_profile": profile_contract(expected_profile),
        "toolchain_identity": toolchain,
        "readonly_isolation_identity": isolation,
        "source_snapshot": {
            "root_recovery": snapshot_root_recovery,
            "identity": snapshot_identity,
            "post_gate_verification": snapshot_verification,
            "cleanup": snapshot_cleanup,
        },
        "runtime_scratch": {
            "admission": runtime_scratch,
            "cleanup": runtime_scratch_cleanup,
        },
        "target": {
            "admitted_root": str(admitted_root),
            "namespace": str(target),
            "artifact_key": artifact_key,
            "marker": str(target / MARKER_NAME),
            "marker_sha256": marker_sha256,
            "registry_lock_wait_seconds": registry_wait,
            "global_execution_lock_wait_seconds": global_execution_wait,
            "namespace_lock_wait_seconds": namespace_wait,
            "cleanup_performed": retention["decision"]
            in {
                "EPHEMERAL_CLEANED",
                "INCOMPLETE_GATE_CLEANED",
                "POST_POLICY_EXCEEDED_CLEANED",
            },
        },
        "retention": {
            **retention_admission,
            **retention,
        },
        "argv": normalized_command,
        "artifact_export": artifact_export,
        "test_execution": test_summary,
        "test_contract": test_contract,
        "gate": gate,
        "log_binding": {
            "path": str(log),
            "bytes": gate_log.get("bytes"),
            "sha256": gate_log.get("sha256"),
            "prefix_bytes": len(log_prefix),
            "prefix_sha256": hashlib.sha256(log_prefix).hexdigest(),
            "source_keyed_sha256": gate_log.get("bound_sha256"),
            "source_key_present_in_prefix": source_key.encode("ascii")
            in log_prefix,
            "timed_gate_evidence_matches": log_evidence_matches,
        },
    }


def rendered_result(payload: Mapping[str, Any]) -> bytes:
    try:
        return bounded_json_bytes(
            dict(payload),
            maximum_bytes=MAX_GATE_RESULT_BYTES,
            label="gate result payload",
        )
    except ValueError as exc:
        raise GateBlocked(
            "result-payload-exceeds-bound",
            f"limit={MAX_GATE_RESULT_BYTES}: {exc}",
        ) from exc


def unreserved_result_bytes(payload: Mapping[str, Any]) -> bytes:
    """Render a terminal payload that no reservation ever made durable.

    Used only before `reserve_result_path` succeeds, so there is no file
    for a consumer to compare stdout against. The payload is still bounded
    exactly like a published one; if it is refused, a fixed minimal BLOCK
    is rendered instead, because a terminal handler must not raise a second
    exception on its way out.
    """
    try:
        return rendered_result(payload)
    except GateBlocked:
        return rendered_result(
            {
                "schema": RESULT_SCHEMA,
                "decision": "BLOCK",
                "blockers": ["result-payload-exceeds-bound"],
            }
        )


def emit_durable_result(raw: bytes) -> None:
    """Write the exact bytes a consumer will re-read from the result file.

    `build_source_bound_rust.py` compares captured stdout to the published
    result byte for byte *before* it reads the decision, so stdout has to
    carry the durable representation and nothing else. A second rendering
    here — pretty-printed, re-encoded, or re-sorted — makes every release
    invocation fail that comparison whatever the gate itself decided.
    """
    sys.stdout.flush()
    sys.stdout.buffer.write(raw)
    sys.stdout.buffer.flush()


def reserve_result_path(path: Path) -> dict[str, Any]:
    reservation_payload = {
        "schema": RESULT_SCHEMA,
        "decision": "BLOCK",
        "blockers": ["result-publication-incomplete"],
        "phase": "RESERVED_BEFORE_EXECUTION",
        "artifact_export_accepted": False,
    }
    raw = rendered_result(reservation_payload)
    lexical, descriptor, parent = open_exclusive_output(path)
    identity: os.stat_result | None = None
    try:
        write_all(descriptor, raw)
        os.fsync(descriptor)
        os.fchmod(descriptor, 0o400)
        identity = os.fstat(descriptor)
        verify_output_identity(lexical, descriptor, parent)
        os.fsync(parent)
    except BaseException:
        if identity is not None:
            try:
                observed = os.stat(
                    lexical.name,
                    dir_fd=parent,
                    follow_symlinks=False,
                )
            except FileNotFoundError:
                pass
            else:
                if (observed.st_dev, observed.st_ino) == (
                    identity.st_dev,
                    identity.st_ino,
                ):
                    try:
                        os.unlink(lexical.name, dir_fd=parent)
                        os.fsync(parent)
                    except OSError:
                        pass
        os.close(descriptor)
        os.close(parent)
        raise
    return {
        "path": lexical,
        "descriptor": descriptor,
        "parent_descriptor": parent,
        "identity": identity,
        "payload": reservation_payload,
        "raw": raw,
    }


def close_result_reservation(reservation: Mapping[str, Any]) -> None:
    for key in ("descriptor", "parent_descriptor"):
        descriptor = reservation.get(key)
        if isinstance(descriptor, int):
            try:
                os.close(descriptor)
            except OSError:
                pass


def publish_reserved_result(
    reservation: Mapping[str, Any],
    payload: Mapping[str, Any],
) -> bytes:
    """Publish `payload` and return the exact bytes now on disk.

    The caller needs those bytes verbatim: the release wrapper compares
    them with the gate's stdout, so re-rendering the same payload a second
    time at the print site would be a separate representation to keep in
    step. Returning the committed bytes makes that impossible to get wrong.
    """
    path = Path(str(reservation["path"]))
    descriptor = int(reservation["descriptor"])
    parent = int(reservation["parent_descriptor"])
    expected = reservation["identity"]
    if not isinstance(expected, os.stat_result):
        raise ValueError("result reservation identity is unavailable")
    raw = rendered_result(payload)
    temporary = f".{path.name}.tmp-{secrets.token_hex(16)}"
    backup = f".{path.name}.reserved-{secrets.token_hex(16)}"
    temporary_descriptor: int | None = None
    temporary_identity: os.stat_result | None = None
    backup_created = False
    publication_committed = False
    try:
        current = os.fstat(descriptor)
        if (
            (current.st_dev, current.st_ino)
            != (expected.st_dev, expected.st_ino)
            or current.st_nlink != 1
        ):
            raise ValueError("result reservation descriptor changed")
        verify_output_identity(path, descriptor, parent)
        temporary_descriptor = os.open(
            temporary,
            os.O_WRONLY
            | os.O_CREAT
            | os.O_EXCL
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            0o400,
            dir_fd=parent,
        )
        write_all(temporary_descriptor, raw)
        os.fsync(temporary_descriptor)
        temporary_identity = os.fstat(temporary_descriptor)
        if (
            not stat.S_ISREG(temporary_identity.st_mode)
            or temporary_identity.st_nlink != 1
        ):
            raise ValueError("result temporary identity is unsafe")
        os.close(temporary_descriptor)
        temporary_descriptor = None
        verify_output_identity(path, descriptor, parent)
        os.link(
            path.name,
            backup,
            src_dir_fd=parent,
            dst_dir_fd=parent,
            follow_symlinks=False,
        )
        backup_created = True
        os.fsync(parent)
        os.rename(
            temporary,
            path.name,
            src_dir_fd=parent,
            dst_dir_fd=parent,
        )
        temporary = ""
        published = os.stat(
            path.name, dir_fd=parent, follow_symlinks=False
        )
        if (
            not stat.S_ISREG(published.st_mode)
            or published.st_nlink != 1
            or (published.st_dev, published.st_ino)
            != (temporary_identity.st_dev, temporary_identity.st_ino)
        ):
            raise ValueError("result publication identity mismatch")
        os.fsync(parent)
        publication_committed = True
        try:
            os.unlink(backup, dir_fd=parent)
            backup_created = False
            os.fsync(parent)
        except OSError:
            # The canonical result is already durably committed. A private
            # old-BLOCK residue is harmless and can be removed manually.
            pass
        return raw
    except BaseException as original:
        rollback_errors: list[str] = []
        if backup_created and not publication_committed:
            try:
                backup_metadata = os.stat(
                    backup, dir_fd=parent, follow_symlinks=False
                )
                if (backup_metadata.st_dev, backup_metadata.st_ino) != (
                    expected.st_dev,
                    expected.st_ino,
                ):
                    raise ValueError(
                        "result reservation backup identity mismatch"
                    )
                try:
                    canonical = os.stat(
                        path.name,
                        dir_fd=parent,
                        follow_symlinks=False,
                    )
                except FileNotFoundError:
                    canonical = None
                if canonical is not None and (
                    canonical.st_dev,
                    canonical.st_ino,
                ) == (expected.st_dev, expected.st_ino):
                    os.unlink(backup, dir_fd=parent)
                elif (
                    canonical is not None
                    and temporary_identity is not None
                    and (canonical.st_dev, canonical.st_ino)
                    == (
                        temporary_identity.st_dev,
                        temporary_identity.st_ino,
                    )
                ):
                    os.unlink(path.name, dir_fd=parent)
                    os.rename(
                        backup,
                        path.name,
                        src_dir_fd=parent,
                        dst_dir_fd=parent,
                    )
                elif canonical is None:
                    os.rename(
                        backup,
                        path.name,
                        src_dir_fd=parent,
                        dst_dir_fd=parent,
                    )
                else:
                    raise ValueError(
                        "result rollback found a foreign canonical inode"
                    )
                os.fsync(parent)
                backup_created = False
            except BaseException as rollback_error:
                rollback_errors.append(str(rollback_error))
        if rollback_errors:
            raise GateBlocked(
                "result-publication-rollback-failed",
                "; ".join(rollback_errors),
            ) from original
        raise
    finally:
        if temporary_descriptor is not None:
            os.close(temporary_descriptor)
        if temporary:
            try:
                os.unlink(temporary, dir_fd=parent)
            except FileNotFoundError:
                pass


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--log", type=Path, required=True)
    parser.add_argument("--result", type=Path, required=True)
    parser.add_argument("--timeout", type=float, required=True)
    parser.add_argument("--lock-timeout", type=float, default=6 * 60 * 60)
    parser.add_argument("--export-artifact")
    parser.add_argument("--export-path", type=Path)
    parser.add_argument(
        "--ephemeral",
        action="store_true",
        help=(
            "Remove this clean candidate's artifact namespace under the same "
            "locks after the gate; dirty candidates are always rejected."
        ),
    )
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if args.command[:1] == ["--"]:
        args.command = args.command[1:]
    if not args.command:
        parser.error("a Cargo command is required after --")
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    result_path: Path | None = None
    reservation: dict[str, Any] | None = None
    evidence_lock_descriptor: int | None = None
    evidence_lock_inherited = False
    durable_raw: bytes | None = None
    try:
        if not math.isfinite(args.timeout) or args.timeout <= 0:
            raise GateBlocked(
                "invalid-gate-timeout", str(args.timeout)
            )
        if (
            not math.isfinite(args.lock_timeout)
            or args.lock_timeout < 0
        ):
            raise GateBlocked(
                "invalid-lock-timeout", str(args.lock_timeout)
            )
        root = repository_root(args.repo)
        initial_state = capture_source_state(
            root, require_clean=True
        )
        admitted_root = validate_admitted_root(DEFAULT_PROFILE)
        evidence_root = validated_evidence_root(
            candidate_root=root,
            admitted_root=admitted_root,
            git_common_dir=Path(initial_state["git_common_dir"]),
            git_dir=Path(initial_state["git_dir"]),
            profile=DEFAULT_PROFILE,
        )
        inherited_lock_raw = os.environ.get(
            "CODEFIRE_GATE_EVIDENCE_LOCK_FD"
        )
        if inherited_lock_raw is None:
            (
                evidence_lock_descriptor,
                evidence_lock_wait_seconds,
            ) = acquire_gate_evidence_lock(
                evidence_root, args.lock_timeout
            )
        else:
            try:
                evidence_lock_descriptor = int(inherited_lock_raw)
            except ValueError as exc:
                raise GateBlocked(
                    "invalid-inherited-evidence-lock",
                    inherited_lock_raw,
                ) from exc
            validate_gate_evidence_lock_descriptor(
                evidence_root, evidence_lock_descriptor
            )
            evidence_lock_inherited = True
            evidence_lock_wait_seconds = 0.0
        evidence_admission = gate_evidence_admission(
            evidence_root, reserve_attempt=True
        )
        candidate_log_path = evidence_path(
            args.log,
            candidate_root=root,
            admitted_root=admitted_root,
            git_common_dir=Path(initial_state["git_common_dir"]),
            git_dir=Path(initial_state["git_dir"]),
            profile=DEFAULT_PROFILE,
            label="gate log",
        )
        candidate_result_path = evidence_path(
            args.result,
            candidate_root=root,
            admitted_root=admitted_root,
            git_common_dir=Path(initial_state["git_common_dir"]),
            git_dir=Path(initial_state["git_dir"]),
            profile=DEFAULT_PROFILE,
            label="gate result",
        )
        if candidate_result_path == candidate_log_path:
            raise GateBlocked(
                "evidence-path-collision",
                "gate log and result JSON must use different paths",
            )
        result_path = candidate_result_path
        reservation = reserve_result_path(result_path)
        payload = run_source_keyed_gate(
            args.command,
            args.repo,
            candidate_log_path,
            args.timeout,
            lock_timeout_seconds=args.lock_timeout,
            ephemeral=args.ephemeral,
            export_artifact_relative=args.export_artifact,
            export_path=args.export_path,
        )
        evidence_postflight = gate_evidence_admission(
            evidence_root,
            reserve_attempt=False,
            additional_reserve_bytes=(
                GATE_EVIDENCE_RESULT_PUBLICATION_RESERVE_BYTES
            ),
            additional_entry_reserve=(
                GATE_EVIDENCE_RESULT_PUBLICATION_ENTRY_RESERVE
            ),
        )
        payload["evidence_retention"] = {
            "schema": "codefire.cargo_gate_evidence_retention.v1",
            "lock_file": str(
                evidence_root / GATE_EVIDENCE_LOCK_NAME
            ),
            "lock_inherited": evidence_lock_inherited,
            "lock_wait_seconds": evidence_lock_wait_seconds,
            "admission": evidence_admission,
            "postflight": evidence_postflight,
        }
        durable_raw = publish_reserved_result(reservation, payload)
        close_result_reservation(reservation)
        reservation = None
    except GateBlocked as exc:
        payload = {
            "schema": RESULT_SCHEMA,
            "decision": "BLOCK",
            "blockers": [exc.code],
            "error": exc.detail,
        }
        if reservation is not None:
            try:
                durable_raw = publish_reserved_result(reservation, payload)
            except (OSError, ValueError) as write_exc:
                print(
                    f"source-keyed Cargo gate result write failed: {write_exc}",
                    file=sys.stderr,
                )
                payload = dict(reservation["payload"])
                # The reservation bytes are what the path still holds, so
                # stdout stays the durable representation on this path too.
                durable_raw = bytes(reservation["raw"])
            finally:
                close_result_reservation(reservation)
                reservation = None
        else:
            durable_raw = unreserved_result_bytes(payload)
        emit_durable_result(durable_raw)
        return 3
    except Exception as exc:
        failure = {
            "schema": RESULT_SCHEMA,
            "decision": "BLOCK",
            "blockers": ["source-keyed-gate-internal-error"],
            "error_type": type(exc).__name__,
            "error": str(exc),
        }
        if reservation is not None:
            try:
                durable_raw = publish_reserved_result(reservation, failure)
            except (OSError, ValueError) as write_exc:
                print(
                    f"source-keyed Cargo gate result write failed: {write_exc}",
                    file=sys.stderr,
                )
                failure = dict(reservation["payload"])
                durable_raw = bytes(reservation["raw"])
            finally:
                close_result_reservation(reservation)
                reservation = None
        else:
            durable_raw = unreserved_result_bytes(failure)
        print(f"source-keyed Cargo gate error: {exc}", file=sys.stderr)
        emit_durable_result(durable_raw)
        return 2
    finally:
        if evidence_lock_descriptor is not None:
            if evidence_lock_inherited:
                os.close(evidence_lock_descriptor)
            else:
                release_gate_evidence_lock(
                    evidence_lock_descriptor
                )
    emit_durable_result(durable_raw)
    if payload["decision"] == "PASS":
        return 0
    gate_code = payload.get("gate", {}).get("exit_code")
    return int(gate_code) if isinstance(gate_code, int) and gate_code != 0 else 3


if __name__ == "__main__":
    raise SystemExit(main())
