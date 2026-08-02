#!/usr/bin/env python3
"""Run one gate without a shell and record monotonic, content-bound evidence."""

from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
import math
import os
import re
import select
import shlex
import shutil
import signal
import stat
import subprocess
import sys
import threading
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable, Mapping

POLICY_PYCACHE_PREFIX = "/dev/null"
sys.dont_write_bytecode = True
sys.pycache_prefix = POLICY_PYCACHE_PREFIX

DEFAULT_MAX_LOG_BYTES = 256 * 1024 * 1024
MAX_RESULT_BYTES = 1024 * 1024
MAX_JSON_STRING_CHARACTERS = 64 * 1024
MAX_JSON_ENTRIES = 100_000
PID_NAMESPACE_LAUNCHER = Path("/usr/bin/unshare")
PARENT_DEATH_LAUNCHER = Path("/usr/bin/setpriv")
PARENT_DEATH_HELPER = Path(__file__).resolve().with_name(
    "pdeath_exec.py"
)
ISOLATED_PYTHON = Path(sys.executable).resolve(strict=True)
MAX_LAUNCHER_BYTES = 64 * 1024**2
PROTECTED_RUST_TOOLS = frozenset(
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
SHELL_DISPATCHERS = frozenset({"bash", "dash", "ksh", "sh", "zsh"})
PREFIX_DISPATCHERS = frozenset({"nice", "nohup", "setsid"})
CODE_INTERPRETERS = frozenset(
    {"perl", "perl5", "python", "python3", "ruby"}
)
VERSIONED_PYTHON_RE = re.compile(
    r"^python(?:[0-9]+(?:\.[0-9]+)*)?$"
)
ENV_ASSIGNMENT_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
UNRESOLVED_WRAPPER_DISPATCH = "__codefire-unresolved-wrapper-dispatch__"
PR_SET_PDEATHSIG = 1


def descriptor_sha256(descriptor: int) -> str:
    digest = hashlib.sha256()
    offset = 0
    while True:
        chunk = os.pread(descriptor, 1024 * 1024, offset)
        if not chunk:
            break
        digest.update(chunk)
        offset += len(chunk)
    return digest.hexdigest()


def open_pinned_launchers() -> dict[str, dict[str, Any]]:
    paths = {
        "python": ISOLATED_PYTHON,
        "pdeath-helper": PARENT_DEATH_HELPER,
        "setpriv": PARENT_DEATH_LAUNCHER,
        "unshare": PID_NAMESPACE_LAUNCHER,
    }
    opened: dict[str, dict[str, Any]] = {}
    try:
        for name, path in sorted(paths.items()):
            lexical = path.absolute()
            if lexical.resolve(strict=True) != lexical:
                raise ValueError(
                    f"timed-gate launcher is not a real path: {name}"
                )
            descriptor = os.open(
                lexical,
                os.O_RDONLY
                | os.O_CLOEXEC
                | getattr(os, "O_NOFOLLOW", 0),
            )
            metadata = os.fstat(descriptor)
            observed = os.stat(lexical, follow_symlinks=False)
            if (
                not stat.S_ISREG(metadata.st_mode)
                or metadata.st_nlink != 1
                or metadata.st_mode & 0o022
                or metadata.st_size > MAX_LAUNCHER_BYTES
                or (
                    name != "pdeath-helper"
                    and not os.access(lexical, os.X_OK)
                )
                or (
                    name == "pdeath-helper"
                    and metadata.st_uid != os.geteuid()
                )
                or (metadata.st_dev, metadata.st_ino)
                != (observed.st_dev, observed.st_ino)
            ):
                os.close(descriptor)
                raise ValueError(
                    f"timed-gate launcher identity is unsafe: {name}"
                )
            opened[name] = {
                "descriptor": descriptor,
                "path": lexical,
                "identity": (
                    metadata.st_dev,
                    metadata.st_ino,
                    stat.S_IMODE(metadata.st_mode),
                    metadata.st_uid,
                    metadata.st_nlink,
                    metadata.st_size,
                    metadata.st_mtime_ns,
                    metadata.st_ctime_ns,
                    descriptor_sha256(descriptor),
                ),
            }
        return opened
    except BaseException:
        for item in opened.values():
            os.close(int(item["descriptor"]))
        raise


def verify_and_close_pinned_launchers(
    opened: dict[str, dict[str, Any]],
    *,
    verify: bool,
) -> None:
    error: BaseException | None = None
    try:
        if verify:
            for name, item in opened.items():
                descriptor = int(item["descriptor"])
                metadata = os.fstat(descriptor)
                path = Path(item["path"])
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
                    descriptor_sha256(descriptor),
                )
                if (
                    current != item["identity"]
                    or (observed.st_dev, observed.st_ino)
                    != current[:2]
                ):
                    raise ValueError(
                        "timed-gate launcher changed during execution: "
                        f"{name}"
                    )
    except BaseException as exc:
        error = exc
    finally:
        for item in opened.values():
            os.close(int(item["descriptor"]))
    if error is not None:
        raise error


def _set_parent_death_signal(expected_parent: int) -> None:
    libc = ctypes.CDLL(None, use_errno=True)
    if libc.prctl(
        PR_SET_PDEATHSIG, signal.SIGKILL, 0, 0, 0
    ) != 0:
        os._exit(126)
    if os.getppid() != expected_parent:
        os._exit(125)


def _close_fds_except(kept: set[int]) -> None:
    for raw in os.listdir("/proc/self/fd"):
        try:
            descriptor = int(raw)
        except ValueError:
            continue
        if descriptor in kept:
            continue
        try:
            os.close(descriptor)
        except OSError:
            pass


class RuntimeGuardWorker:
    def __init__(self, guard: Callable[[], None]) -> None:
        command_read, command_write = os.pipe2(os.O_CLOEXEC)
        result_read, result_write = os.pipe2(os.O_CLOEXEC)
        expected_parent = os.getpid()
        try:
            pid = os.fork()
        except BaseException:
            for descriptor in (
                command_read,
                command_write,
                result_read,
                result_write,
            ):
                os.close(descriptor)
            raise
        if pid == 0:
            try:
                os.close(command_write)
                os.close(result_read)
                os.setsid()
                _set_parent_death_signal(expected_parent)
                _close_fds_except({command_read, result_write})
                while True:
                    command = os.read(command_read, 1)
                    if not command or command == b"Q":
                        os._exit(0)
                    if command != b"R":
                        os._exit(2)
                    try:
                        guard()
                    except BaseException as exc:
                        payload = json.dumps(
                            {
                                "error_type": type(exc).__name__[:128],
                                "code": str(
                                    getattr(
                                        exc,
                                        "code",
                                        "runtime-resource-guard-blocked",
                                    )
                                )[:512],
                            },
                            separators=(",", ":"),
                        ).encode("utf-8")
                        os.write(result_write, b"E" + payload + b"\n")
                    else:
                        os.write(result_write, b"P\n")
            except BaseException:
                os._exit(3)
            os._exit(0)
        os.close(command_read)
        os.close(result_write)
        self.pid = pid
        self.command = command_write
        self.result = result_read
        self.buffer = bytearray()
        self.closed = False

    def invoke(self, deadline: float) -> dict[str, str] | None:
        os.write(self.command, b"R")
        while True:
            newline = self.buffer.find(b"\n")
            if newline >= 0:
                raw = bytes(self.buffer[:newline])
                del self.buffer[: newline + 1]
                if raw == b"P":
                    return None
                if not raw.startswith(b"E"):
                    raise RuntimeError(
                        "runtime guard worker protocol failed"
                    )
                loaded = json.loads(raw[1:])
                if not isinstance(loaded, dict):
                    raise RuntimeError(
                        "runtime guard worker response is invalid"
                    )
                return {
                    "error_type": str(
                        loaded.get("error_type", "RuntimeError")
                    )[:128],
                    "code": str(
                        loaded.get(
                            "code",
                            "runtime-resource-guard-blocked",
                        )
                    )[:512],
                }
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError(
                    "runtime guard exceeded the gate deadline"
                )
            readable, _, _ = select.select(
                [self.result], [], [], remaining
            )
            if not readable:
                raise TimeoutError(
                    "runtime guard exceeded the gate deadline"
                )
            chunk = os.read(self.result, 2048)
            if not chunk:
                raise RuntimeError(
                    "runtime guard worker exited unexpectedly"
                )
            self.buffer.extend(chunk)
            if len(self.buffer) > 2048:
                raise RuntimeError(
                    "runtime guard worker output exceeded 2 KiB"
                )

    def close(self, *, force: bool = False) -> None:
        if self.closed:
            return
        self.closed = True
        try:
            if force:
                try:
                    os.killpg(self.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            else:
                try:
                    os.write(self.command, b"Q")
                except (BrokenPipeError, OSError):
                    pass
            deadline = time.monotonic() + 0.25
            while True:
                try:
                    observed, _status = os.waitpid(
                        self.pid, os.WNOHANG
                    )
                except ChildProcessError:
                    break
                if observed == self.pid:
                    break
                if time.monotonic() >= deadline:
                    try:
                        os.killpg(self.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    try:
                        os.waitpid(self.pid, 0)
                    except ChildProcessError:
                        pass
                    break
                time.sleep(0.005)
        finally:
            os.close(self.command)
            os.close(self.result)


def bounded_json_bytes(payload: Any) -> bytes:
    stack: list[tuple[Any, int]] = [(payload, 0)]
    entries = 0
    while stack:
        value, depth = stack.pop()
        if depth > 64:
            raise ValueError("result JSON depth exceeded")
        if isinstance(value, str):
            if len(value) > MAX_JSON_STRING_CHARACTERS:
                raise ValueError(
                    "result contains an oversized string"
                )
        elif isinstance(value, dict):
            entries += len(value)
            if entries > MAX_JSON_ENTRIES:
                raise ValueError("result JSON entry bound exceeded")
            for key, item in value.items():
                if (
                    not isinstance(key, str)
                    or len(key) > MAX_JSON_STRING_CHARACTERS
                ):
                    raise ValueError(
                        "result contains an invalid object key"
                    )
                stack.append((item, depth + 1))
        elif isinstance(value, (list, tuple)):
            entries += len(value)
            if entries > MAX_JSON_ENTRIES:
                raise ValueError("result JSON entry bound exceeded")
            stack.extend((item, depth + 1) for item in value)
        elif value is not None and not isinstance(
            value, (bool, int, float)
        ):
            raise ValueError("result contains a non-JSON value")
    encoder = json.JSONEncoder(
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
        allow_nan=False,
    )
    output = bytearray()
    for text in encoder.iterencode(payload):
        raw = text.encode("utf-8")
        if len(output) + len(raw) + 1 > MAX_RESULT_BYTES:
            raise ValueError(
                f"result exceeds {MAX_RESULT_BYTES} bytes"
            )
        output.extend(raw)
    output.extend(b"\n")
    return bytes(output)


def shell_command_string(argv: list[str]) -> str | None:
    """Return a shell -c payload without interpreting or executing it."""

    for index, item in enumerate(argv[1:], 1):
        if item == "--":
            return None
        if item.startswith("-") and item != "-":
            short_options = item[1:] if not item.startswith("--") else ""
            if "c" in short_options:
                return argv[index + 1] if index + 1 < len(argv) else ""
    return None


def split_env_command(value: str) -> list[str]:
    try:
        return shlex.split(value, posix=True)
    except ValueError:
        return [UNRESOLVED_WRAPPER_DISPATCH]


def env_command(
    argv: list[str],
    environment: Mapping[str, str],
    *,
    _depth: int = 0,
) -> tuple[list[str], dict[str, str]]:
    """Return the command portion of a conventional env invocation."""

    if _depth > 8:
        return [UNRESOLVED_WRAPPER_DISPATCH], dict(environment)
    child_environment = dict(environment)
    index = 1
    while index < len(argv):
        item = argv[index]
        if item == "--":
            return argv[index + 1 :], child_environment
        if ENV_ASSIGNMENT_RE.match(item):
            key, value = item.split("=", 1)
            child_environment[key] = value
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
            if item in {"-u", "--unset"} and index + 1 < len(argv):
                child_environment.pop(argv[index + 1], None)
            index += 2
            continue
        if item.startswith("--argv0=") or (
            item.startswith("-a") and item != "-a"
        ):
            index += 1
            continue
        if item in {"-S", "--split-string"}:
            split_command = (
                split_env_command(argv[index + 1])
                if index + 1 < len(argv)
                else [UNRESOLVED_WRAPPER_DISPATCH]
            )
            return env_command(
                ["env", *split_command],
                child_environment,
                _depth=_depth + 1,
            )
        if item.startswith("--split-string="):
            return env_command(
                ["env", *split_env_command(item.split("=", 1)[1])],
                child_environment,
                _depth=_depth + 1,
            )
        if item.startswith("-S") and item != "-S":
            return env_command(
                ["env", *split_env_command(item[2:])],
                child_environment,
                _depth=_depth + 1,
            )
        if item.startswith(("-", "--")):
            index += 1
            continue
        return argv[index:], child_environment
    return [], child_environment


def timeout_command(argv: list[str]) -> list[str]:
    """Return the command portion of a conventional timeout invocation."""

    index = 1
    while index < len(argv):
        item = argv[index]
        if item == "--":
            index += 1
            if index < len(argv):
                index += 1  # duration after the option terminator
            break
        if item in {"-k", "--kill-after", "-s", "--signal"}:
            index += 2
            continue
        if item.startswith("-"):
            index += 1
            continue
        index += 1  # duration
        break
    return argv[index:]


def executable_names(
    value: str, environment: Mapping[str, str]
) -> set[str]:
    names = {Path(value).name}
    if os.sep in value:
        resolved = Path(value).resolve(strict=False)
    else:
        located = shutil.which(value, path=environment.get("PATH"))
        resolved = (
            Path(located).resolve(strict=False)
            if located is not None
            else Path(value)
        )
    names.add(resolved.name)
    return names


def first_prefix_command(
    argv: list[str],
    *,
    options_with_values: frozenset[str] = frozenset(),
) -> list[str]:
    index = 1
    while index < len(argv):
        item = argv[index]
        if item == "--":
            return argv[index + 1 :]
        if item in options_with_values:
            index += 2
            continue
        if item.startswith("-"):
            index += 1
            continue
        return argv[index:]
    return []


def taskset_command(argv: list[str]) -> list[str]:
    index = 1
    mask_seen = False
    while index < len(argv):
        item = argv[index]
        if item == "--":
            index += 1
            break
        if item.startswith("--cpu-list="):
            index += 1
            break
        if item.startswith("-c") and item != "-c":
            index += 1
            break
        if item in {"-c", "--cpu-list"}:
            index += 1
            continue
        if item in {"-p", "--pid"}:
            return []
        if item.startswith("-"):
            index += 1
            continue
        if not mask_seen:
            mask_seen = True
            index += 1
            break
    return argv[index:]


def code_interpreter_names(names: set[str]) -> set[str]:
    return {
        name
        for name in names
        if name in CODE_INTERPRETERS
        or VERSIONED_PYTHON_RE.fullmatch(name) is not None
    }


def has_interpreter_payload(argv: list[str], names: set[str]) -> bool:
    interpreters = code_interpreter_names(names)
    perl_or_ruby = bool(interpreters & {"perl", "perl5", "ruby"})
    for item in argv[1:]:
        if item == "--":
            return False
        if item in {"-c", "-e"} or item.startswith(("-c", "-e")):
            return True
        if (
            perl_or_ruby
            and item.startswith("-")
            and "e" in item[1:]
        ):
            return True
    return False


def protected_rust_dispatch(
    argv: list[str],
    environment: Mapping[str, str] | None = None,
    *,
    _depth: int = 0,
) -> str | None:
    """Classify dispatch at the trusted direct-argv admission boundary.

    Known shell/code-string dispatch is unresolved even when its payload
    appears harmless: shell quoting and interpreter semantics are not proved
    here. This is not an execve monitor for arbitrary trusted binaries or
    script files, nor for descendants they may launch.
    """

    if not argv:
        return None
    if _depth > 16:
        return "unresolved-wrapper"
    child_environment = dict(os.environ if environment is None else environment)
    if "$" in argv[0] or "`" in argv[0]:
        return "unresolved-wrapper"
    names = executable_names(argv[0], child_environment)
    if UNRESOLVED_WRAPPER_DISPATCH in names:
        return "unresolved-wrapper"
    direct = names & PROTECTED_RUST_TOOLS
    if direct:
        return sorted(direct)[0]
    if "env" in names:
        command, env_environment = env_command(argv, child_environment)
        return protected_rust_dispatch(
            command, env_environment, _depth=_depth + 1
        )
    if "timeout" in names:
        return protected_rust_dispatch(
            timeout_command(argv), child_environment, _depth=_depth + 1
        )
    prefix = names & PREFIX_DISPATCHERS
    if prefix:
        options = (
            frozenset({"-n", "--adjustment"})
            if "nice" in prefix
            else frozenset()
        )
        return protected_rust_dispatch(
            first_prefix_command(argv, options_with_values=options),
            child_environment,
            _depth=_depth + 1,
        )
    if "taskset" in names:
        return protected_rust_dispatch(
            taskset_command(argv), child_environment, _depth=_depth + 1
        )
    if "rustup" in names:
        try:
            run_index = argv.index("run", 1)
        except ValueError:
            return None
        toolchain_index = run_index + 1
        while toolchain_index < len(argv):
            item = argv[toolchain_index]
            if item == "--":
                toolchain_index += 1
                break
            if item == "--install":
                toolchain_index += 1
                continue
            if item.startswith("-"):
                return "unresolved-wrapper"
            break
        if toolchain_index + 1 < len(argv):
            return protected_rust_dispatch(
                argv[toolchain_index + 1 :],
                child_environment,
                _depth=_depth + 1,
            )
        return None
    if names & SHELL_DISPATCHERS:
        command = shell_command_string(argv)
        if command is None:
            return None
        return "unresolved-wrapper"
    if code_interpreter_names(names):
        return (
            "unresolved-wrapper"
            if has_interpreter_payload(argv, names)
            else None
        )
    return None


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


def descriptor_log_identity(
    descriptor: int,
    *,
    binding_prefix: bytes,
) -> tuple[int, str, str]:
    before = os.fstat(descriptor)
    if not stat.S_ISREG(before.st_mode) or before.st_nlink != 1:
        raise ValueError("gate log descriptor is not a private regular file")
    digest = hashlib.sha256()
    bound_digest = hashlib.sha256(binding_prefix)
    size = 0
    offset = 0
    while True:
        chunk = os.pread(descriptor, 1024 * 1024, offset)
        if not chunk:
            break
        digest.update(chunk)
        bound_digest.update(chunk)
        size += len(chunk)
        offset += len(chunk)
    after = os.fstat(descriptor)
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
        or size != after.st_size
    ):
        raise ValueError("gate log changed while it was rehashed")
    return size, digest.hexdigest(), bound_digest.hexdigest()


def open_exclusive_output(path: Path) -> tuple[Path, int, int]:
    lexical = path.absolute()
    if lexical.name in {"", ".", ".."}:
        raise ValueError(f"invalid output filename: {lexical}")
    parent_lexical = lexical.parent.absolute()
    parent = parent_lexical.resolve(strict=True)
    if parent != parent_lexical or not parent.is_dir():
        raise ValueError(
            f"output parent must be a real directory without symlinks: "
            f"{parent_lexical}"
        )
    parent_descriptor = os.open(
        parent,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    descriptor: int | None = None
    created_identity: tuple[int, int] | None = None
    try:
        descriptor = os.open(
            lexical.name,
            os.O_RDWR
            | os.O_CREAT
            | os.O_EXCL
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            0o600,
            dir_fd=parent_descriptor,
        )
        metadata = os.fstat(descriptor)
        created_identity = (metadata.st_dev, metadata.st_ino)
        if (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_nlink != 1
        ):
            raise ValueError(
                f"output is not a private regular file: {lexical}"
            )
        verify_parent_identity(lexical, parent_descriptor)
        return lexical, descriptor, parent_descriptor
    except BaseException:
        try:
            if created_identity is not None:
                try:
                    observed = os.stat(
                        lexical.name,
                        dir_fd=parent_descriptor,
                        follow_symlinks=False,
                    )
                except FileNotFoundError:
                    pass
                else:
                    if (observed.st_dev, observed.st_ino) == (
                        created_identity
                    ):
                        os.unlink(
                            lexical.name,
                            dir_fd=parent_descriptor,
                        )
                        os.fsync(parent_descriptor)
        finally:
            try:
                if descriptor is not None:
                    os.close(descriptor)
            finally:
                os.close(parent_descriptor)
        raise


def verify_parent_identity(path: Path, parent_descriptor: int) -> None:
    parent_lexical = path.absolute().parent
    opened = os.fstat(parent_descriptor)
    try:
        observed = os.stat(parent_lexical, follow_symlinks=False)
        resolved = parent_lexical.resolve(strict=True)
    except OSError as exc:
        raise ValueError(
            f"output parent changed during write: {parent_lexical}"
        ) from exc
    if (
        resolved != parent_lexical
        or not stat.S_ISDIR(observed.st_mode)
        or (observed.st_dev, observed.st_ino)
        != (opened.st_dev, opened.st_ino)
    ):
        raise ValueError(
            f"output parent changed during write: {parent_lexical}"
        )


def verify_output_identity(
    path: Path, descriptor: int, parent_descriptor: int
) -> None:
    verify_parent_identity(path, parent_descriptor)
    opened = os.fstat(descriptor)
    observed = os.stat(
        path.name, dir_fd=parent_descriptor, follow_symlinks=False
    )
    lexical_observed = os.stat(path, follow_symlinks=False)
    if (
        not stat.S_ISREG(observed.st_mode)
        or observed.st_nlink != 1
        or (observed.st_dev, observed.st_ino)
        != (opened.st_dev, opened.st_ino)
        or (lexical_observed.st_dev, lexical_observed.st_ino)
        != (opened.st_dev, opened.st_ino)
    ):
        raise ValueError(f"output identity changed during write: {path}")


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


def process_group_exists(process_group: int) -> bool:
    try:
        os.killpg(process_group, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def terminate_process_group(
    process: subprocess.Popen[bytes],
    *,
    grace_seconds: float = 1.0,
) -> None:
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    deadline = time.monotonic() + max(0.0, grace_seconds)
    while process_group_exists(process.pid) and time.monotonic() < deadline:
        time.sleep(0.01)
    if process_group_exists(process.pid):
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    if process.poll() is None:
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            try:
                process.kill()
            except ProcessLookupError:
                pass
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
    environment: Mapping[str, str] | None = None,
    log_prefix: bytes = b"",
    log_binding_prefix: bytes = b"",
    log_analyzer: Any | None = None,
    max_log_bytes: int = DEFAULT_MAX_LOG_BYTES,
    pass_fds: tuple[int, ...] = (),
    runtime_guard: Callable[[], None] | None = None,
    runtime_guard_interval_seconds: float = 1.0,
    pid_namespace_isolated: bool = False,
) -> dict[str, Any]:
    if not argv:
        raise ValueError("gate argv must not be empty")
    if not math.isfinite(timeout_seconds) or timeout_seconds <= 0:
        raise ValueError("timeout must be positive")
    if (
        not math.isfinite(drift_threshold_seconds)
        or drift_threshold_seconds < 0
    ):
        raise ValueError("drift threshold must be finite and non-negative")
    if not isinstance(log_prefix, bytes):
        raise ValueError("log prefix must be bytes")
    if len(log_prefix) > 64 * 1024:
        raise ValueError("log prefix exceeds 64 KiB")
    if not isinstance(log_binding_prefix, bytes):
        raise ValueError("log binding prefix must be bytes")
    if max_log_bytes < len(log_prefix) or max_log_bytes < 1:
        raise ValueError("max log bytes must include the complete log prefix")
    if any(not isinstance(item, int) or item < 0 for item in pass_fds):
        raise ValueError("pass_fds must contain non-negative integers")
    if not isinstance(pid_namespace_isolated, bool):
        raise ValueError("pid_namespace_isolated must be boolean")
    if (
        not math.isfinite(runtime_guard_interval_seconds)
        or runtime_guard_interval_seconds <= 0
    ):
        raise ValueError(
            "runtime guard interval must be finite and positive"
        )
    cwd = cwd.resolve(strict=True)
    if not cwd.is_dir():
        raise ValueError(f"gate cwd is not a directory: {cwd}")
    log_path, log_descriptor, parent_descriptor = open_exclusive_output(
        log_path
    )

    start_wall_ns = wall_clock_ns()
    start_monotonic_ns = monotonic_clock_ns()
    timed_out = False
    log_limit_exceeded = False
    runtime_guard_error: dict[str, str] | None = None
    runtime_guard_invocations = 0
    runtime_guard_total_seconds = 0.0
    runtime_guard_max_seconds = 0.0
    runtime_guard_unfinished_at_timeout = False
    pump_errors: list[BaseException] = []
    log_digest = hashlib.sha256()
    bound_log_digest = hashlib.sha256(log_binding_prefix)
    log_bytes = 0
    process: subprocess.Popen[bytes] | None = None
    log = None
    launcher_files: dict[str, dict[str, Any]] = {}
    guard_worker: RuntimeGuardWorker | None = None
    log_committed = False
    try:
        log = os.fdopen(log_descriptor, "w+b", closefd=False)
        log.write(log_prefix)
        log.flush()
        log_digest.update(log_prefix)
        bound_log_digest.update(log_prefix)
        log_bytes = len(log_prefix)
        if runtime_guard is not None:
            guard_worker = RuntimeGuardWorker(runtime_guard)
        if pid_namespace_isolated:
            launch_argv = argv
            launch_pass_fds = pass_fds
        else:
            launcher_files = open_pinned_launchers()
            descriptors = {
                name: int(item["descriptor"])
                for name, item in launcher_files.items()
            }
            launch_argv = [
                f"/proc/self/fd/{descriptors['python']}",
                "-I",
                "-S",
                "-B",
                "-X",
                f"pycache_prefix={POLICY_PYCACHE_PREFIX}",
                f"/proc/self/fd/{descriptors['pdeath-helper']}",
                "--parent-pid",
                str(os.getpid()),
                "--",
                f"/proc/self/fd/{descriptors['setpriv']}",
                "--pdeathsig",
                "SIGKILL",
                f"/proc/self/fd/{descriptors['unshare']}",
                "-Urpf",
                "--mount-proc",
                "--kill-child=SIGKILL",
                # --mount-proc implies --mount, so a fresh procfs is
                # mounted here. Pin propagation rather than inherit the
                # util-linux default, so that mount can never propagate
                # back onto the host's /proc.
                "--propagation",
                "private",
                "--",
                *argv,
            ]
            launch_pass_fds = tuple(
                sorted({*pass_fds, *descriptors.values()})
            )
        process = subprocess.Popen(
            launch_argv,
            cwd=cwd,
            env=None if environment is None else dict(environment),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            start_new_session=True,
            pass_fds=launch_pass_fds,
        )
        if process.stdout is None:
            raise RuntimeError("gate output pipe was not created")

        def pump_output() -> None:
            nonlocal log_bytes, log_limit_exceeded
            try:
                for chunk in iter(lambda: process.stdout.read(64 * 1024), b""):
                    remaining = max_log_bytes - log_bytes
                    admitted = chunk[: max(0, remaining)]
                    if admitted:
                        log.write(admitted)
                        log_digest.update(admitted)
                        bound_log_digest.update(admitted)
                        log_bytes += len(admitted)
                        if log_analyzer is not None:
                            log_analyzer.feed(admitted)
                    if len(admitted) != len(chunk):
                        log_limit_exceeded = True
                        terminate_process_group(process)
            except BaseException as exc:
                pump_errors.append(exc)
                terminate_process_group(process, grace_seconds=0.0)

        pump = threading.Thread(
            target=pump_output,
            name="codefire-gate-log-pump",
            daemon=True,
        )
        pump.start()
        deadline = time.monotonic() + timeout_seconds
        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                timed_out = True
                terminate_process_group(process, grace_seconds=0.0)
                returncode = 124
                break
            if runtime_guard is not None:
                guard_started = time.monotonic()
                try:
                    if guard_worker is None:
                        raise RuntimeError(
                            "runtime guard worker is unavailable"
                        )
                    observed_guard_error = guard_worker.invoke(
                        deadline
                    )
                except TimeoutError:
                    runtime_guard_unfinished_at_timeout = True
                    timed_out = True
                    guard_worker.close(force=True)
                    guard_worker = None
                    terminate_process_group(
                        process, grace_seconds=0.0
                    )
                    returncode = 124
                    break
                guard_duration = time.monotonic() - guard_started
                runtime_guard_invocations += 1
                runtime_guard_total_seconds += guard_duration
                runtime_guard_max_seconds = max(
                    runtime_guard_max_seconds, guard_duration
                )
                if time.monotonic() >= deadline:
                    timed_out = True
                    if guard_worker is not None:
                        guard_worker.close(force=True)
                        guard_worker = None
                    terminate_process_group(
                        process, grace_seconds=0.0
                    )
                    returncode = 124
                    break
                if observed_guard_error is not None:
                    runtime_guard_error = observed_guard_error
                    terminate_process_group(process)
                    returncode = 126
                    break
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                timed_out = True
                terminate_process_group(process, grace_seconds=0.0)
                returncode = 124
                break
            try:
                returncode = process.wait(
                    timeout=min(
                        remaining,
                        runtime_guard_interval_seconds,
                    )
                    if runtime_guard is not None
                    else remaining
                )
                if time.monotonic() > deadline:
                    timed_out = True
                    terminate_process_group(
                        process, grace_seconds=0.0
                    )
                    returncode = 124
                break
            except subprocess.TimeoutExpired:
                continue
        # A successful leader is not sufficient: close the namespace/process
        # group so descendants cannot retain stdout or outlive the gate.
        terminate_process_group(
            process,
            grace_seconds=0.0 if timed_out else 1.0,
        )
        pump.join(timeout=10)
        if pump.is_alive():
            terminate_process_group(process, grace_seconds=0.0)
            pump.join(timeout=10)
        if pump.is_alive():
            raise RuntimeError("gate output pump did not terminate")
        if pump_errors:
            raise RuntimeError(f"gate output pump failed: {pump_errors[0]}")
        process.stdout.close()
        if log_limit_exceeded:
            returncode = 125
        log.flush()
        os.fsync(log.fileno())
        verify_output_identity(log_path, log_descriptor, parent_descriptor)
        (
            observed_log_bytes,
            observed_log_sha256,
            observed_bound_sha256,
        ) = descriptor_log_identity(
            log_descriptor,
            binding_prefix=log_binding_prefix,
        )
        if (
            observed_log_bytes != log_bytes
            or observed_log_sha256 != log_digest.hexdigest()
            or observed_bound_sha256 != bound_log_digest.hexdigest()
        ):
            raise ValueError(
                "gate log bytes differ from the captured output stream"
            )
        os.fchmod(log_descriptor, 0o400)
        os.fsync(log_descriptor)
        verify_output_identity(log_path, log_descriptor, parent_descriptor)
        os.fsync(parent_descriptor)
        analysis = (
            log_analyzer.finish() if log_analyzer is not None else None
        )
        if analysis is not None:
            bounded_json_bytes({"log_analysis": analysis})
        if launcher_files:
            verify_and_close_pinned_launchers(
                launcher_files, verify=True
            )
            launcher_files = {}
        end_monotonic_ns = monotonic_clock_ns()
        end_wall_ns = wall_clock_ns()
        result = {
            "schema": "codefire.timed_gate.v1",
            "argv": argv,
            "cwd": str(cwd),
            "timeout_seconds": timeout_seconds,
            "timed_out": timed_out,
            "log_limit_exceeded": log_limit_exceeded,
            "runtime_guard": {
                "enabled": runtime_guard is not None,
                "interval_seconds": (
                    runtime_guard_interval_seconds
                    if runtime_guard is not None
                    else None
                ),
                "decision": (
                    "BLOCK"
                    if (
                        runtime_guard_error is not None
                        or runtime_guard_unfinished_at_timeout
                    )
                    else "PASS"
                ),
                "error": runtime_guard_error,
                "invocation_count": runtime_guard_invocations,
                "total_seconds": runtime_guard_total_seconds,
                "maximum_seconds": runtime_guard_max_seconds,
                "unfinished_at_timeout": (
                    runtime_guard_unfinished_at_timeout
                ),
            },
            "pid_namespace": {
                "isolated": True,
                "provided_by_caller": pid_namespace_isolated,
                "kill_descendants_on_exit": True,
            },
            "exit_code": returncode,
            "status": (
                "passed"
                if (
                    returncode == 0
                    and not timed_out
                    and not log_limit_exceeded
                )
                else "failed"
            ),
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
                "sha256": log_digest.hexdigest(),
                "bound_sha256": bound_log_digest.hexdigest(),
                "complete": not log_limit_exceeded,
                "max_bytes": max_log_bytes,
            },
        }
        if analysis is not None:
            result["log_analysis"] = analysis
        bounded_json_bytes(result)
        log_committed = True
    except BaseException as original:
        if process is not None and process.poll() is None:
            terminate_process_group(process)
        cleanup_error: BaseException | None = None
        if not log_committed:
            try:
                opened = os.fstat(log_descriptor)
                observed = os.stat(
                    log_path.name,
                    dir_fd=parent_descriptor,
                    follow_symlinks=False,
                )
                if (opened.st_dev, opened.st_ino) != (
                    observed.st_dev,
                    observed.st_ino,
                ):
                    raise ValueError(
                        "failed gate log cleanup identity changed"
                    )
                os.unlink(log_path.name, dir_fd=parent_descriptor)
                os.fsync(parent_descriptor)
            except FileNotFoundError:
                pass
            except BaseException as exc:
                cleanup_error = exc
        if cleanup_error is not None:
            raise RuntimeError(
                f"failed gate log cleanup: {cleanup_error}"
            ) from original
        raise
    finally:
        try:
            try:
                if launcher_files:
                    verify_and_close_pinned_launchers(
                        launcher_files, verify=False
                    )
                    launcher_files = {}
            finally:
                try:
                    if guard_worker is not None:
                        guard_worker.close(force=True)
                        guard_worker = None
                finally:
                    if log is not None:
                        log.close()
        finally:
            try:
                os.close(log_descriptor)
            finally:
                os.close(parent_descriptor)
    return result


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

    from recurrence_v2_git import GitError, GitRepo

    try:
        probe = GitRepo(cwd.resolve(strict=True))
        root = Path(probe.toplevel())
        repo = GitRepo(root)
        revision, tree = repo.head_identity()
        status_before = repo.worktree_status()
        index_flagged_before = repo.index_flagged_entries()
    except GitError as exc:
        raise ValueError(f"pre-command Git identity probe failed: {exc}") from exc
    payload = run_gate(
        argv,
        cwd,
        log_path,
        timeout_seconds,
        drift_threshold_seconds=drift_threshold_seconds,
        wall_clock_ns=wall_clock_ns,
        monotonic_clock_ns=monotonic_clock_ns,
    )
    try:
        status_after = repo.worktree_status()
    except GitError as exc:
        status_after = (f"<post-command git status failed: {exc}>",)
    try:
        index_flagged_after = repo.index_flagged_entries()
    except GitError as exc:
        index_flagged_after = (f"<post-command index probe failed: {exc}>",)
    index_flagged = tuple(sorted(set(index_flagged_before) | set(index_flagged_after)))
    try:
        revision_after, tree_after = repo.head_identity()
        head_probe_error = None
    except GitError as exc:
        revision_after = None
        tree_after = None
        head_probe_error = str(exc)
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
        "head_revision_after": revision_after,
        "head_tree_after": tree_after,
        "head_unchanged": (
            head_probe_error is None
            and revision_after == revision
            and tree_after == tree
        ),
        "head_probe_error": head_probe_error,
        "log_inside_source_worktree": log_inside,
        "result_inside_source_worktree": result_inside,
        "status_command": "git status --porcelain=v1 -z --untracked-files=all",
        "index_command": "git ls-files -v -z",
    }
    payload["limits"] = list(TIMED_V2_LIMITS)
    return payload


def recover_atomic_json_temporaries(
    parent: int,
    destination_name: str,
) -> None:
    prefix = f".{destination_name}.tmp-"
    deadline = time.monotonic() + 10.0
    names: list[str] = []
    entries = 0
    with os.scandir(parent) as scanner:
        for entry in scanner:
            entries += 1
            if entries > 10_000:
                raise ValueError(
                    "result parent entry bound exceeded"
                )
            if time.monotonic() > deadline:
                raise TimeoutError(
                    "result temporary recovery timed out"
                )
            if entry.name.startswith(prefix):
                names.append(entry.name)
    if len(names) > 32:
        raise ValueError(
            "result temporary residue count exceeded 32"
        )
    for name in sorted(names):
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
                or metadata.st_nlink != 1
                or (metadata.st_dev, metadata.st_ino)
                != (observed.st_dev, observed.st_ino)
            ):
                raise ValueError(
                    f"unsafe result temporary residue: {name}"
                )
            os.unlink(name, dir_fd=parent)
        finally:
            os.close(descriptor)
    if names:
        os.fsync(parent)


def atomic_write_json(path: Path, payload: dict[str, Any]) -> None:
    raw = bounded_json_bytes(payload)
    lexical = path.absolute()
    parent_lexical = lexical.parent.absolute()
    parent = parent_lexical.resolve(strict=True)
    if parent != parent_lexical:
        raise ValueError("result parent must not contain symlinks")
    parent_descriptor = os.open(
        parent,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    temporary_name = (
        f".{lexical.name}.tmp-{os.getpid()}-{time.monotonic_ns()}"
    )
    descriptor: int | None = None
    metadata: os.stat_result | None = None
    try:
        verify_parent_identity(lexical, parent_descriptor)
        recover_atomic_json_temporaries(
            parent_descriptor, lexical.name
        )
        try:
            os.stat(
                lexical.name,
                dir_fd=parent_descriptor,
                follow_symlinks=False,
            )
        except FileNotFoundError:
            pass
        else:
            raise FileExistsError(f"result already exists: {lexical}")
        descriptor = os.open(
            temporary_name,
            os.O_WRONLY
            | os.O_CREAT
            | os.O_EXCL
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            0o600,
            dir_fd=parent_descriptor,
        )
        metadata = os.fstat(descriptor)
        if (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_nlink != 1
        ):
            raise ValueError(
                "result temporary is not a private regular file"
            )
        offset = 0
        while offset < len(raw):
            written = os.write(descriptor, raw[offset:])
            if written < 1:
                raise OSError("short result write")
            offset += written
        os.fsync(descriptor)
        after_write = os.fstat(descriptor)
        if (
            not stat.S_ISREG(after_write.st_mode)
            or after_write.st_nlink != 1
            or (after_write.st_dev, after_write.st_ino)
            != (metadata.st_dev, metadata.st_ino)
        ):
            raise ValueError("result temporary is not a private regular file")
        metadata = after_write
        os.close(descriptor)
        descriptor = None
        verify_parent_identity(lexical, parent_descriptor)
        os.link(
            temporary_name,
            lexical.name,
            src_dir_fd=parent_descriptor,
            dst_dir_fd=parent_descriptor,
            follow_symlinks=False,
        )
        os.unlink(temporary_name, dir_fd=parent_descriptor)
        observed = os.stat(
            lexical.name,
            dir_fd=parent_descriptor,
            follow_symlinks=False,
        )
        if (
            not stat.S_ISREG(observed.st_mode)
            or observed.st_nlink != 1
            or (observed.st_dev, observed.st_ino)
            != (metadata.st_dev, metadata.st_ino)
        ):
            raise ValueError("result identity changed during publish")
        lexical_observed = os.stat(lexical, follow_symlinks=False)
        if (lexical_observed.st_dev, lexical_observed.st_ino) != (
            metadata.st_dev,
            metadata.st_ino,
        ):
            raise ValueError("result parent changed during publish")
        verify_parent_identity(lexical, parent_descriptor)
        os.fsync(parent_descriptor)
    except BaseException:
        if metadata is not None:
            try:
                published = os.stat(
                    lexical.name,
                    dir_fd=parent_descriptor,
                    follow_symlinks=False,
                )
            except FileNotFoundError:
                pass
            else:
                if (published.st_dev, published.st_ino) == (
                    metadata.st_dev,
                    metadata.st_ino,
                ):
                    try:
                        os.unlink(
                            lexical.name,
                            dir_fd=parent_descriptor,
                        )
                    except FileNotFoundError:
                        pass
                    else:
                        try:
                            os.fsync(parent_descriptor)
                        except OSError:
                            pass
        raise
    finally:
        cleanup_errors: list[str] = []
        try:
            if descriptor is not None:
                try:
                    os.close(descriptor)
                except OSError as exc:
                    cleanup_errors.append(str(exc))
            try:
                observed = os.stat(
                    temporary_name,
                    dir_fd=parent_descriptor,
                    follow_symlinks=False,
                )
                if metadata is not None and (
                    observed.st_dev,
                    observed.st_ino,
                ) != (metadata.st_dev, metadata.st_ino):
                    raise ValueError(
                        "result temporary cleanup identity changed"
                    )
                os.unlink(
                    temporary_name, dir_fd=parent_descriptor
                )
            except FileNotFoundError:
                pass
            except (OSError, ValueError) as exc:
                cleanup_errors.append(str(exc))
        finally:
            try:
                os.close(parent_descriptor)
            except OSError as exc:
                cleanup_errors.append(str(exc))
        if cleanup_errors:
            raise RuntimeError(
                "result publication cleanup failed: "
                + "; ".join(cleanup_errors[:8])
            )


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
    protected_tool = protected_rust_dispatch(args.command)
    if protected_tool is not None:
        print(
            f"timed gate error: {protected_tool} dispatch is not admitted; "
            "Cargo gates must use run_source_keyed_cargo_gate.py",
            file=sys.stderr,
        )
        return 2
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
