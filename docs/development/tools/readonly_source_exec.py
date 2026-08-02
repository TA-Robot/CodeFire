#!/usr/bin/env python3
"""Mount one exact source root read-only in a private namespace, then exec."""

from __future__ import annotations

import argparse
import errno
import hashlib
import json
import os
import stat
import subprocess
import sys
from pathlib import Path


SNAPSHOT_MARKER_NAME = ".codefire-source-snapshot.json"
MAX_PRIVATE_SOURCE_TMPFS_BYTES = 4 * 1024**3


def mount_points(mountinfo: str) -> list[Path]:
    result: list[Path] = []
    for line in mountinfo.splitlines():
        fields = line.split()
        if len(fields) >= 7 and "-" in fields:
            result.append(Path(unescape_mount_field(fields[4])))
    return result


def unescape_mount_field(value: str) -> str:
    for encoded, decoded in (
        ("\\040", " "),
        ("\\011", "\t"),
        ("\\012", "\n"),
        ("\\134", "\\"),
    ):
        value = value.replace(encoded, decoded)
    return value


def mount_propagation_tags(mountinfo: str) -> list[tuple[str, str]]:
    """Return (mount point, propagation tag) for every propagating mount.

    ``mountinfo`` optional fields sit between field 6 and the ``-``
    separator. ``shared:`` marks a peer group that propagates mounts
    outward; ``master:`` marks a slave that receives them from one.

    The separator is searched from index 6 rather than from the start of
    the line: the root and mount-point fields are not ``-``-escaped, so a
    path literally named ``-`` would otherwise be mistaken for the
    separator and silently hide that line's propagation tags.
    """
    tagged: list[tuple[str, str]] = []
    for line in mountinfo.splitlines():
        if not line.strip():
            continue
        fields = line.split()
        # Every rule below is "reject", never "skip": a propagation check
        # that silently passes over a line it cannot read reports a shared
        # mount as private, which is the one direction this must never
        # fail in. A short line takes the same path as a separator-less
        # one rather than being dropped by a length guard.
        if len(fields) < 7 or "-" not in fields[6:]:
            raise RuntimeError(
                f"unparseable mountinfo line: {line!r}"
            )
        separator = fields.index("-", 6)
        point = unescape_mount_field(fields[4])
        for field in fields[6:separator]:
            if field.startswith(("shared:", "master:")):
                tagged.append((point, field))
    return tagged


def prove_private_propagation(mountinfo: str) -> None:
    """Fail closed unless the whole namespace is private.

    The private source tmpfs is mounted at the candidate worktree's own
    path. Under shared propagation that mount would escape into the host
    namespace and mask the developer's real worktree with a read-only
    snapshot copy for as long as the gate ran. ``unshare --propagation
    private`` is requested explicitly, but a default is not evidence, so
    the guarantee is re-derived from the kernel's own mount table before
    any mount is created.
    """
    tagged = mount_propagation_tags(mountinfo)
    if tagged:
        raise RuntimeError(
            "mount namespace is not privately propagated: "
            + ", ".join(
                f"{point}[{tag}]" for point, tag in sorted(tagged)[:8]
            )
        )


def mount_is_read_only(source: Path, mountinfo: str) -> bool:
    expected = str(source)
    matches: list[bool] = []
    for line in mountinfo.splitlines():
        fields = line.split()
        if len(fields) < 7 or "-" not in fields:
            continue
        if unescape_mount_field(fields[4]) != expected:
            continue
        matches.append("ro" in fields[5].split(","))
    return bool(matches) and matches[-1]


def prove_read_only(source: Path) -> None:
    directory = os.open(
        source,
        os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | getattr(os, "O_NOFOLLOW", 0),
    )
    probe = f".codefire-readonly-probe-{os.getpid()}"
    try:
        try:
            descriptor = os.open(
                probe,
                os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC,
                0o600,
                dir_fd=directory,
            )
        except OSError as exc:
            if exc.errno != errno.EROFS:
                raise RuntimeError(
                    f"read-only source probe failed with {exc.errno}: {exc}"
                ) from exc
        else:
            os.close(descriptor)
            os.unlink(probe, dir_fd=directory)
            raise RuntimeError("source bind mount remained writable")
    finally:
        os.close(directory)


def prove_regular_read_only(path: Path) -> None:
    try:
        descriptor = os.open(
            path,
            os.O_WRONLY
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
        )
    except OSError as exc:
        if exc.errno != errno.EROFS:
            raise RuntimeError(
                f"read-only file probe failed with {exc.errno}: {path}: {exc}"
            ) from exc
    else:
        os.close(descriptor)
        raise RuntimeError(f"control file remained writable: {path}")


def prove_writable_directory(path: Path) -> None:
    directory = os.open(
        path,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    probe = f".codefire-writable-probe-{os.getpid()}"
    try:
        descriptor = os.open(
            probe,
            os.O_WRONLY
            | os.O_CREAT
            | os.O_EXCL
            | os.O_CLOEXEC,
            0o600,
            dir_fd=directory,
        )
        os.close(descriptor)
        os.unlink(probe, dir_fd=directory)
    finally:
        try:
            os.unlink(probe, dir_fd=directory)
        except (FileNotFoundError, OSError):
            pass
        os.close(directory)


def canonical_json_sha256(value: object) -> str:
    raw = json.dumps(
        value,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    return hashlib.sha256(raw).hexdigest()


def stable_identity(metadata: os.stat_result) -> tuple[int, ...]:
    return (
        metadata.st_dev,
        metadata.st_ino,
        metadata.st_mode,
        metadata.st_nlink,
        metadata.st_uid,
        metadata.st_size,
        metadata.st_mtime_ns,
        metadata.st_ctime_ns,
    )


def write_all(descriptor: int, raw: bytes) -> None:
    offset = 0
    while offset < len(raw):
        written = os.write(descriptor, raw[offset:])
        if written < 1:
            raise RuntimeError("short write while copying private source")
        offset += written


def read_regular_at(
    directory: int,
    name: str,
    *,
    maximum_bytes: int = 64 * 1024,
) -> bytes:
    descriptor = os.open(
        name,
        os.O_RDONLY | os.O_CLOEXEC | getattr(os, "O_NOFOLLOW", 0),
        dir_fd=directory,
    )
    try:
        before = os.fstat(descriptor)
        if (
            not stat.S_ISREG(before.st_mode)
            or before.st_nlink != 1
            or before.st_size > maximum_bytes
        ):
            raise RuntimeError(f"invalid private-source input: {name}")
        chunks: list[bytes] = []
        size = 0
        while True:
            chunk = os.read(descriptor, min(64 * 1024, maximum_bytes + 1 - size))
            if not chunk:
                break
            chunks.append(chunk)
            size += len(chunk)
            if size > maximum_bytes:
                raise RuntimeError(f"private-source input exceeds bound: {name}")
        after = os.fstat(descriptor)
        if stable_identity(before) != stable_identity(after) or size != after.st_size:
            raise RuntimeError(f"private-source input changed while read: {name}")
        return b"".join(chunks)
    finally:
        os.close(descriptor)


def copy_snapshot_tree(
    source: int,
    destination: int,
    *,
    expected_file_count: int,
    expected_logical_bytes: int,
    expected_manifest_sha256: str,
    prefix: str = "",
) -> list[dict[str, object]]:
    def open_directory_components(
        root: int, components: tuple[str, ...]
    ) -> int:
        current = os.dup(root)
        try:
            for component in components:
                child = os.open(
                    component,
                    os.O_RDONLY
                    | os.O_DIRECTORY
                    | os.O_CLOEXEC
                    | getattr(os, "O_NOFOLLOW", 0),
                    dir_fd=current,
                )
                os.close(current)
                current = child
            return current
        except BaseException:
            os.close(current)
            raise

    def relative_path(components: tuple[str, ...]) -> str:
        suffix = "/".join(components)
        if prefix and suffix:
            return f"{prefix}/{suffix}"
        return prefix or suffix

    def directory_object_identity(
        metadata: os.stat_result,
    ) -> tuple[int, int]:
        return metadata.st_dev, metadata.st_ino

    files: list[dict[str, object]] = []
    logical_bytes = 0
    directory_file_counts: dict[tuple[str, ...], int] = {}
    # Each event stores component names rather than an open descriptor. A
    # directory is reopened from the pinned root for each event, so live file
    # descriptors remain constant even for a source deeper than Python's
    # recursion limit.
    stack: list[
        tuple[
            str,
            tuple[str, ...],
            tuple[int, ...] | None,
            tuple[int, ...] | None,
            tuple[tuple[str, ...], ...],
            int,
        ]
    ] = [("enter", (), None, None, (), 0)]
    while stack:
        (
            action,
            components,
            expected_source_identity,
            expected_destination_identity,
            child_components,
            direct_file_count,
        ) = stack.pop()
        relative_directory = relative_path(components)
        if action == "leave":
            source_directory = open_directory_components(source, components)
            destination_directory = open_directory_components(
                destination, components
            )
            try:
                if stable_identity(
                    os.fstat(source_directory)
                ) != expected_source_identity:
                    raise RuntimeError(
                        "snapshot directory changed during copy: "
                        f"{relative_directory}"
                    )
                if directory_object_identity(
                    os.fstat(destination_directory)
                ) != expected_destination_identity:
                    raise RuntimeError(
                        "private-source directory changed during copy: "
                        f"{relative_directory}"
                    )
                subtree_file_count = direct_file_count + sum(
                    directory_file_counts.pop(child)
                    for child in child_components
                )
                if components and subtree_file_count == 0:
                    raise RuntimeError(
                        "snapshot contains an untracked empty directory: "
                        f"{relative_directory}"
                    )
                if components:
                    os.fchmod(destination_directory, 0o555)
                directory_file_counts[components] = subtree_file_count
            finally:
                os.close(destination_directory)
                os.close(source_directory)
            continue

        source_directory = open_directory_components(source, components)
        destination_directory = open_directory_components(
            destination, components
        )
        children: list[
            tuple[
                tuple[str, ...],
                tuple[int, ...],
                tuple[int, ...],
            ]
        ] = []
        direct_files = 0
        try:
            source_opened = os.fstat(source_directory)
            destination_opened = os.fstat(destination_directory)
            source_identity = stable_identity(source_opened)
            destination_identity = directory_object_identity(
                destination_opened
            )
            if (
                expected_source_identity is not None
                and source_identity != expected_source_identity
            ):
                raise RuntimeError(
                    "snapshot directory changed before copy: "
                    f"{relative_directory}"
                )
            if (
                expected_destination_identity is not None
                and destination_identity != expected_destination_identity
            ):
                raise RuntimeError(
                    "private-source directory changed before copy: "
                    f"{relative_directory}"
                )
            for name in sorted(os.listdir(source_directory)):
                child_path = (*components, name)
                relative = relative_path(child_path)
                if relative == SNAPSHOT_MARKER_NAME:
                    continue
                metadata = os.stat(
                    name,
                    dir_fd=source_directory,
                    follow_symlinks=False,
                )
                if stat.S_ISDIR(metadata.st_mode):
                    os.mkdir(
                        name,
                        mode=0o700,
                        dir_fd=destination_directory,
                    )
                    source_child = os.open(
                        name,
                        os.O_RDONLY
                        | os.O_DIRECTORY
                        | os.O_CLOEXEC
                        | getattr(os, "O_NOFOLLOW", 0),
                        dir_fd=source_directory,
                    )
                    destination_child: int | None = None
                    try:
                        destination_child = os.open(
                            name,
                            os.O_RDONLY
                            | os.O_DIRECTORY
                            | os.O_CLOEXEC
                            | getattr(os, "O_NOFOLLOW", 0),
                            dir_fd=destination_directory,
                        )
                        child_source_identity = stable_identity(
                            os.fstat(source_child)
                        )
                        if child_source_identity != stable_identity(metadata):
                            raise RuntimeError(
                                "snapshot directory changed before copy: "
                                f"{relative}"
                            )
                        child_destination_identity = directory_object_identity(
                            os.fstat(destination_child)
                        )
                    finally:
                        if destination_child is not None:
                            os.close(destination_child)
                        os.close(source_child)
                    children.append(
                        (
                            child_path,
                            child_source_identity,
                            child_destination_identity,
                        )
                    )
                    continue
                if (
                    not stat.S_ISREG(metadata.st_mode)
                    or metadata.st_nlink != 1
                ):
                    raise RuntimeError(
                        f"snapshot contains an unsupported entry: {relative}"
                    )
                source_file = os.open(
                    name,
                    os.O_RDONLY
                    | os.O_CLOEXEC
                    | getattr(os, "O_NOFOLLOW", 0),
                    dir_fd=source_directory,
                )
                destination_file: int | None = None
                try:
                    opened = os.fstat(source_file)
                    if stable_identity(opened) != stable_identity(metadata):
                        raise RuntimeError(
                            "snapshot file changed before copy: "
                            f"{relative}"
                        )
                    destination_file = os.open(
                        name,
                        os.O_WRONLY
                        | os.O_CREAT
                        | os.O_EXCL
                        | os.O_CLOEXEC
                        | getattr(os, "O_NOFOLLOW", 0),
                        0o600,
                        dir_fd=destination_directory,
                    )
                    digest = hashlib.sha256()
                    size = 0
                    while True:
                        chunk = os.read(source_file, 1024 * 1024)
                        if not chunk:
                            break
                        size += len(chunk)
                        logical_bytes += len(chunk)
                        if logical_bytes > expected_logical_bytes:
                            raise RuntimeError(
                                "snapshot logical bytes exceeded the "
                                "admitted manifest"
                            )
                        write_all(destination_file, chunk)
                        digest.update(chunk)
                    after = os.fstat(source_file)
                    if (
                        stable_identity(opened) != stable_identity(after)
                        or size != after.st_size
                    ):
                        raise RuntimeError(
                            f"snapshot file changed during copy: {relative}"
                        )
                    os.fsync(destination_file)
                    executable = bool(opened.st_mode & stat.S_IXUSR)
                    os.fchmod(
                        destination_file,
                        0o555 if executable else 0o444,
                    )
                    files.append(
                        {
                            "path": relative,
                            "mode": "100755" if executable else "100644",
                            "bytes": size,
                            "sha256": digest.hexdigest(),
                        }
                    )
                    direct_files += 1
                finally:
                    if destination_file is not None:
                        os.close(destination_file)
                    os.close(source_file)
            if stable_identity(os.fstat(source_directory)) != source_identity:
                raise RuntimeError(
                    "snapshot directory changed during copy: "
                    f"{relative_directory}"
                )
            if (
                directory_object_identity(os.fstat(destination_directory))
                != destination_identity
            ):
                raise RuntimeError(
                    "private-source directory changed during copy: "
                    f"{relative_directory}"
                )
        finally:
            os.close(destination_directory)
            os.close(source_directory)
        child_paths = tuple(child[0] for child in children)
        stack.append(
            (
                "leave",
                components,
                source_identity,
                destination_identity,
                child_paths,
                direct_files,
            )
        )
        for (
            child_path,
            child_source_identity,
            child_destination_identity,
        ) in reversed(children):
            stack.append(
                (
                    "enter",
                    child_path,
                    child_source_identity,
                    child_destination_identity,
                    (),
                    0,
                )
            )
    if not prefix:
        files.sort(key=lambda item: str(item["path"]))
        observed_bytes = sum(int(item["bytes"]) for item in files)
        if len(files) != expected_file_count:
            raise RuntimeError(
                "private source file count does not match admitted manifest"
            )
        if observed_bytes != expected_logical_bytes:
            raise RuntimeError(
                "private source size does not match admitted manifest"
            )
        observed_sha256 = canonical_json_sha256(files)
        if observed_sha256 != expected_manifest_sha256:
            raise RuntimeError(
                "private source content does not match admitted manifest"
            )
    return files


def descriptors_retained_by_environment(
    environment: dict[str, str], primary_fd: int
) -> set[int]:
    """Descriptors the exec'd tool still needs, read from its own argv/env.

    Cargo reaches rustc, rustdoc, clippy-driver and rustfmt through
    ``/proc/self/fd/<n>`` values in its environment, so those descriptors
    must survive. Everything else the manager pinned is closed before the
    exec, and membership is derived from the environment actually being
    passed rather than from an assumption that the numbers are distinct.
    """
    retained = {primary_fd}
    for value in environment.values():
        if not value.startswith("/proc/self/fd/"):
            continue
        try:
            retained.add(int(value.rsplit("/", 1)[1]))
        except ValueError:
            continue
    return retained


def pinned_fd(value: int) -> int:
    if value < 0:
        raise RuntimeError("pinned descriptor must be non-negative")
    os.fstat(value)
    return value


def verify_directory_fd(
    descriptor: int,
    path: Path,
    *,
    label: str,
) -> None:
    opened = os.fstat(descriptor)
    observed = os.stat(path, follow_symlinks=False)
    if (
        not path.is_dir()
        or (opened.st_dev, opened.st_ino)
        != (observed.st_dev, observed.st_ino)
    ):
        raise RuntimeError(f"{label} descriptor does not match {path}")


def verify_privileges_dropped() -> None:
    status = {}
    for line in Path("/proc/self/status").read_text(encoding="utf-8").splitlines():
        if ":" in line:
            key, value = line.split(":", 1)
            status[key] = value.strip()
    for key in ("CapAmb", "CapBnd", "CapEff", "CapInh", "CapPrm"):
        if int(status.get(key, "1"), 16) != 0:
            raise RuntimeError(f"{key} was not cleared")
    if status.get("NoNewPrivs") != "1":
        raise RuntimeError("no_new_privs was not enabled")
    groups = {
        int(value) for value in status.get("Groups", "").split()
    }
    try:
        overflow_gid = int(
            Path("/proc/sys/kernel/overflowgid").read_text(
                encoding="ascii"
            ).strip()
        )
    except (OSError, ValueError) as exc:
        raise RuntimeError(
            "kernel overflowgid is unavailable"
        ) from exc
    admitted_groups = {0, overflow_gid}
    if not groups.issubset(admitted_groups):
        raise RuntimeError(
            "supplemental groups escape the map-root/overflow boundary: "
            f"{sorted(groups)}"
        )


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--snapshot", type=Path, required=True)
    parser.add_argument("--snapshot-root", type=Path, required=True)
    parser.add_argument("--toolchain-root", type=Path, required=True)
    parser.add_argument("--target", type=Path, required=True)
    parser.add_argument("--admitted-root", type=Path, required=True)
    parser.add_argument("--evidence-root", type=Path, required=True)
    parser.add_argument("--source-fd", type=int, required=True)
    parser.add_argument("--snapshot-fd", type=int, required=True)
    parser.add_argument("--snapshot-root-fd", type=int, required=True)
    parser.add_argument("--snapshot-marker-sha256", required=True)
    parser.add_argument(
        "--snapshot-file-manifest-sha256", required=True
    )
    parser.add_argument("--snapshot-file-count", type=int, required=True)
    parser.add_argument(
        "--snapshot-logical-file-bytes", type=int, required=True
    )
    parser.add_argument("--private-source-tmpfs-bytes", type=int, required=True)
    parser.add_argument("--toolchain-root-fd", type=int, required=True)
    parser.add_argument("--target-fd", type=int, required=True)
    parser.add_argument("--admitted-root-fd", type=int, required=True)
    parser.add_argument("--evidence-root-fd", type=int, required=True)
    parser.add_argument("--target-marker-fd", type=int, required=True)
    parser.add_argument("--target-lock-fd", type=int, required=True)
    parser.add_argument("--target-run-state-fd", type=int, required=True)
    parser.add_argument("--mount-fd", type=int, required=True)
    parser.add_argument("--setpriv-fd", type=int, required=True)
    parser.add_argument("--python-fd", type=int, required=True)
    parser.add_argument("--helper-fd", type=int, required=True)
    parser.add_argument("--primary-fd", type=int, required=True)
    parser.add_argument("--pass-fds", required=True)
    parser.add_argument("--post-drop", action="store_true")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    if args.command[:1] == ["--"]:
        args.command = args.command[1:]
    if not args.command:
        parser.error("an absolute command is required after --")
    for value, label in (
        (args.snapshot_marker_sha256, "snapshot marker digest"),
        (
            args.snapshot_file_manifest_sha256,
            "snapshot file manifest digest",
        ),
    ):
        if len(value) != 64 or any(
            character not in "0123456789abcdef" for character in value
        ):
            parser.error(f"{label} must be lowercase SHA-256")
    if args.snapshot_file_count < 1:
        parser.error("snapshot file count must be positive")
    if args.snapshot_logical_file_bytes < 0:
        parser.error("snapshot logical file bytes must be non-negative")
    if (
        args.private_source_tmpfs_bytes
        < args.snapshot_logical_file_bytes + 1024 * 1024
        or args.private_source_tmpfs_bytes
        > MAX_PRIVATE_SOURCE_TMPFS_BYTES
    ):
        parser.error("private source tmpfs size is outside the admitted bound")
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    source_lexical = args.source.absolute()
    snapshot_lexical = args.snapshot.absolute()
    snapshot_root_lexical = args.snapshot_root.absolute()
    toolchain_lexical = args.toolchain_root.absolute()
    target_lexical = args.target.absolute()
    admitted_root_lexical = args.admitted_root.absolute()
    evidence_root_lexical = args.evidence_root.absolute()
    source = source_lexical.resolve(strict=True)
    snapshot = snapshot_lexical.resolve(strict=True)
    snapshot_root = snapshot_root_lexical.resolve(strict=True)
    toolchain_root = toolchain_lexical.resolve(strict=True)
    target = target_lexical.resolve(strict=True)
    admitted_root = admitted_root_lexical.resolve(strict=True)
    evidence_root = evidence_root_lexical.resolve(strict=True)
    if source_lexical != source or not source.is_dir():
        raise RuntimeError("source must be a real directory without symlinks")
    for lexical, path, label in (
        (snapshot_lexical, snapshot, "snapshot"),
        (snapshot_root_lexical, snapshot_root, "snapshot root"),
        (toolchain_lexical, toolchain_root, "toolchain root"),
        (target_lexical, target, "target"),
        (admitted_root_lexical, admitted_root, "admitted root"),
        (evidence_root_lexical, evidence_root, "evidence root"),
    ):
        if lexical != path or not path.is_dir():
            raise RuntimeError(
                f"{label} must be a real directory without symlinks"
            )
    if snapshot.parent != snapshot_root:
        raise RuntimeError("snapshot must be an immediate child of snapshot root")
    try:
        target.relative_to(admitted_root)
    except ValueError as exc:
        raise RuntimeError("target must be below admitted root") from exc
    protected = (source, snapshot_root, toolchain_root, evidence_root)
    if any(
        target == path
        or path in target.parents
        or target in path.parents
        for path in protected
    ):
        raise RuntimeError("target must be outside source, snapshot, and toolchain")
    expected_target = os.environ.get("CARGO_TARGET_DIR")
    if expected_target is None or Path(expected_target).resolve() != target:
        raise RuntimeError("CARGO_TARGET_DIR does not match isolated target")
    inherited = tuple(
        pinned_fd(int(value))
        for value in args.pass_fds.split(",")
        if value
    )
    source_fd = pinned_fd(args.source_fd)
    snapshot_fd = pinned_fd(args.snapshot_fd)
    snapshot_root_fd = pinned_fd(args.snapshot_root_fd)
    toolchain_root_fd = pinned_fd(args.toolchain_root_fd)
    target_fd = pinned_fd(args.target_fd)
    admitted_root_fd = pinned_fd(args.admitted_root_fd)
    evidence_root_fd = pinned_fd(args.evidence_root_fd)
    target_marker_fd = pinned_fd(args.target_marker_fd)
    target_lock_fd = pinned_fd(args.target_lock_fd)
    target_run_state_fd = pinned_fd(args.target_run_state_fd)
    mount_fd = pinned_fd(args.mount_fd)
    setpriv_fd = pinned_fd(args.setpriv_fd)
    python_fd = pinned_fd(args.python_fd)
    helper_fd = pinned_fd(args.helper_fd)
    primary_fd = pinned_fd(args.primary_fd)
    for descriptor, path, label in (
        (snapshot_fd, snapshot, "snapshot"),
        (snapshot_root_fd, snapshot_root, "snapshot root"),
        (toolchain_root_fd, toolchain_root, "toolchain root"),
        (target_fd, target, "target"),
        (admitted_root_fd, admitted_root, "admitted root"),
        (evidence_root_fd, evidence_root, "evidence root"),
    ):
        verify_directory_fd(descriptor, path, label=label)
    for descriptor, path, label in (
        (
            target_marker_fd,
            target / ".codefire-source-key.json",
            "target marker",
        ),
        (
            target_lock_fd,
            target / ".codefire-source-key.lock",
            "target lock",
        ),
        (
            target_run_state_fd,
            target / ".codefire-source-key-run-state.json",
            "target run state",
        ),
    ):
        opened = os.fstat(descriptor)
        observed = os.stat(path, follow_symlinks=False)
        if (
            not stat.S_ISREG(opened.st_mode)
            or opened.st_nlink != 1
            or (opened.st_dev, opened.st_ino)
            != (observed.st_dev, observed.st_ino)
        ):
            raise RuntimeError(f"{label} descriptor does not match {path}")
    if not args.post_drop:
        verify_directory_fd(source_fd, source, label="source")
    mount_executable = f"/proc/self/fd/{mount_fd}"
    mountinfo = Path("/proc/self/mountinfo").read_text(
        encoding="utf-8", errors="strict"
    )
    prove_private_propagation(mountinfo)
    if args.post_drop:
        verify_privileges_dropped()
        for protected_path in (
            source,
            snapshot_root,
            toolchain_root,
            admitted_root,
            evidence_root,
            target / ".codefire-source-key.json",
            target / ".codefire-source-key.lock",
            target / ".codefire-source-key-run-state.json",
        ):
            remount = subprocess.run(
                [
                    mount_executable,
                    "-o",
                    "remount,bind,rw",
                    str(protected_path),
                ],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                pass_fds=inherited,
                check=False,
            )
            if remount.returncode == 0:
                raise RuntimeError(
                    "unprivileged remount unexpectedly succeeded: "
                    f"{protected_path}"
                )
        mountinfo = Path("/proc/self/mountinfo").read_text(
            encoding="utf-8", errors="strict"
        )
        prove_private_propagation(mountinfo)
        if not mount_is_read_only(source, mountinfo):
            raise RuntimeError("source mount lost read-only state after capability drop")
        if not mount_is_read_only(snapshot_root, mountinfo):
            raise RuntimeError(
                "snapshot root mask lost read-only state after capability drop"
            )
        if not mount_is_read_only(admitted_root, mountinfo):
            raise RuntimeError(
                "admitted root lost read-only state after capability drop"
            )
        if not mount_is_read_only(evidence_root, mountinfo):
            raise RuntimeError(
                "evidence root mask lost read-only state after capability drop"
            )
        if mount_is_read_only(target, mountinfo):
            raise RuntimeError("Cargo target mount unexpectedly became read-only")
        for control_path in (
            target / ".codefire-source-key.json",
            target / ".codefire-source-key.lock",
            target / ".codefire-source-key-run-state.json",
        ):
            if not mount_is_read_only(control_path, mountinfo):
                raise RuntimeError(
                    f"control file mount is not read-only: {control_path}"
                )
        verify_directory_fd(
            source_fd, source, label="private mounted source"
        )
        verify_directory_fd(
            snapshot_fd, snapshot, label="masked snapshot"
        )
        verify_directory_fd(
            snapshot_root_fd,
            snapshot_root,
            label="masked snapshot root",
        )
        verify_directory_fd(
            toolchain_root_fd,
            toolchain_root,
            label="mounted toolchain root",
        )
        verify_directory_fd(target_fd, target, label="target")
        verify_directory_fd(
            admitted_root_fd, admitted_root, label="admitted root"
        )
        verify_directory_fd(
            evidence_root_fd,
            evidence_root,
            label="masked evidence root",
        )
        prove_read_only(source)
        prove_writable_directory(target)
        prove_regular_read_only(
            target / ".codefire-source-key.json"
        )
        prove_regular_read_only(target / ".codefire-source-key.lock")
        prove_regular_read_only(
            target / ".codefire-source-key-run-state.json"
        )
        # Cargo keeps only what its own environment names: the primary tool
        # and the pinned rustc/rustdoc/clippy-driver/rustfmt descriptors.
        #
        # The close set is derived from the inherited set, not from an
        # enumeration of the descriptors this helper happens to be told
        # about by name. The manager pins more than it names here -- the
        # policy supervisor, `unshare`, the parent-death helper, and every
        # toolchain driver including the ones this gate did not select --
        # and each of those is a program a build script could exec through
        # /proc/self/fd. Enumerating leaves exactly those behind, so close
        # the complement of `retained` over `--pass-fds` instead. stdio is
        # never a member and is excluded defensively.
        environment = dict(os.environ)
        retained = descriptors_retained_by_environment(
            environment, primary_fd
        )
        closed: set[int] = set()
        for descriptor in sorted(
            {
                *inherited,
                source_fd,
                snapshot_fd,
                snapshot_root_fd,
                toolchain_root_fd,
                target_fd,
                admitted_root_fd,
                evidence_root_fd,
                target_marker_fd,
                target_lock_fd,
                target_run_state_fd,
                mount_fd,
                setpriv_fd,
                python_fd,
                helper_fd,
            }
        ):
            if (
                descriptor in retained
                or descriptor in closed
                or descriptor in (0, 1, 2)
            ):
                continue
            try:
                os.close(descriptor)
            except OSError as exc:
                if exc.errno != errno.EBADF:
                    raise
            closed.add(descriptor)
        os.chdir(source)
        os.execve(primary_fd, args.command, environment)
        return 127

    points = mount_points(mountinfo)
    for protected_root in (
        source,
        snapshot_root,
        toolchain_root,
        admitted_root,
        evidence_root,
    ):
        descendants = [
            str(point)
            for point in points
            if protected_root in point.parents
        ]
        if descendants:
            raise RuntimeError(
                f"nested mount below {protected_root}: {sorted(descendants)}"
            )
    marker_raw = read_regular_at(snapshot_fd, SNAPSHOT_MARKER_NAME)
    if hashlib.sha256(marker_raw).hexdigest() != args.snapshot_marker_sha256:
        raise RuntimeError("snapshot marker does not match admitted identity")
    source_mount = Path(f"/proc/self/fd/{source_fd}")
    subprocess.run(
        [
            mount_executable,
            "-t",
            "tmpfs",
            "-o",
            (
                f"size={args.private_source_tmpfs_bytes},"
                "mode=0700,nosuid,nodev"
            ),
            "codefire-private-source",
            str(source_mount),
        ],
        stdin=subprocess.DEVNULL,
        pass_fds=inherited,
        check=True,
    )
    mounted_source_fd = os.open(
        source,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    try:
        copy_snapshot_tree(
            snapshot_fd,
            mounted_source_fd,
            expected_file_count=args.snapshot_file_count,
            expected_logical_bytes=args.snapshot_logical_file_bytes,
            expected_manifest_sha256=(
                args.snapshot_file_manifest_sha256
            ),
        )
        os.fsync(mounted_source_fd)
        os.fchmod(mounted_source_fd, 0o555)
    except BaseException:
        os.close(mounted_source_fd)
        raise
    subprocess.run(
        [
            mount_executable,
            "--bind",
            str(source),
            str(source),
        ],
        stdin=subprocess.DEVNULL,
        pass_fds=inherited,
        check=True,
    )
    subprocess.run(
        [
            mount_executable,
            "-o",
            "remount,bind,ro",
            str(source),
        ],
        stdin=subprocess.DEVNULL,
        pass_fds=inherited,
        check=True,
    )
    os.dup2(mounted_source_fd, source_fd, inheritable=True)
    os.close(mounted_source_fd)

    snapshot_root_mount = Path(f"/proc/self/fd/{snapshot_root_fd}")
    subprocess.run(
        [
            mount_executable,
            "-t",
            "tmpfs",
            "-o",
            "size=1048576,mode=0711,nosuid,nodev,noexec",
            "codefire-hidden-snapshots",
            str(snapshot_root_mount),
        ],
        stdin=subprocess.DEVNULL,
        pass_fds=inherited,
        check=True,
    )
    masked_snapshot_root_fd = os.open(
        snapshot_root,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    masked_snapshot_fd: int | None = None
    try:
        os.mkdir(snapshot.name, mode=0o700, dir_fd=masked_snapshot_root_fd)
        masked_snapshot_fd = os.open(
            snapshot.name,
            os.O_RDONLY
            | os.O_DIRECTORY
            | os.O_CLOEXEC
            | getattr(os, "O_NOFOLLOW", 0),
            dir_fd=masked_snapshot_root_fd,
        )
        os.fchmod(masked_snapshot_fd, 0o700)
        os.fchmod(masked_snapshot_root_fd, 0o111)
        subprocess.run(
            [
                mount_executable,
                "--bind",
                str(snapshot_root),
                str(snapshot_root),
            ],
            stdin=subprocess.DEVNULL,
            pass_fds=inherited,
            check=True,
        )
        subprocess.run(
            [
                mount_executable,
                "-o",
                "remount,bind,ro",
                str(snapshot_root),
            ],
            stdin=subprocess.DEVNULL,
            pass_fds=inherited,
            check=True,
        )
        os.dup2(
            masked_snapshot_root_fd,
            snapshot_root_fd,
            inheritable=True,
        )
        os.dup2(masked_snapshot_fd, snapshot_fd, inheritable=True)
    finally:
        if masked_snapshot_fd is not None:
            os.close(masked_snapshot_fd)
        os.close(masked_snapshot_root_fd)

    evidence_root_mount = Path(f"/proc/self/fd/{evidence_root_fd}")
    subprocess.run(
        [
            mount_executable,
            "-t",
            "tmpfs",
            "-o",
            "size=1048576,mode=0700,nosuid,nodev,noexec",
            "codefire-hidden-evidence",
            str(evidence_root_mount),
        ],
        stdin=subprocess.DEVNULL,
        pass_fds=inherited,
        check=True,
    )
    masked_evidence_root_fd = os.open(
        evidence_root,
        os.O_RDONLY
        | os.O_DIRECTORY
        | os.O_CLOEXEC
        | getattr(os, "O_NOFOLLOW", 0),
    )
    try:
        os.fchmod(masked_evidence_root_fd, 0o000)
        subprocess.run(
            [
                mount_executable,
                "--bind",
                str(evidence_root),
                str(evidence_root),
            ],
            stdin=subprocess.DEVNULL,
            pass_fds=inherited,
            check=True,
        )
        subprocess.run(
            [
                mount_executable,
                "-o",
                "remount,bind,ro",
                str(evidence_root),
            ],
            stdin=subprocess.DEVNULL,
            pass_fds=inherited,
            check=True,
        )
        os.dup2(
            masked_evidence_root_fd,
            evidence_root_fd,
            inheritable=True,
        )
    finally:
        os.close(masked_evidence_root_fd)

    toolchain_mount = Path(f"/proc/self/fd/{toolchain_root_fd}")
    subprocess.run(
        [
            mount_executable,
            "--bind",
            str(toolchain_mount),
            str(toolchain_mount),
        ],
        stdin=subprocess.DEVNULL,
        pass_fds=inherited,
        check=True,
    )
    subprocess.run(
        [
            mount_executable,
            "-o",
            "remount,bind,ro",
            str(toolchain_mount),
        ],
        stdin=subprocess.DEVNULL,
        pass_fds=inherited,
        check=True,
    )

    admitted_root_mount = Path(f"/proc/self/fd/{admitted_root_fd}")
    subprocess.run(
        [
            mount_executable,
            "--bind",
            str(admitted_root_mount),
            str(admitted_root_mount),
        ],
        stdin=subprocess.DEVNULL,
        pass_fds=inherited,
        check=True,
    )
    subprocess.run(
        [
            mount_executable,
            "-o",
            "remount,bind,ro",
            str(admitted_root_mount),
        ],
        stdin=subprocess.DEVNULL,
        pass_fds=inherited,
        check=True,
    )
    target_mount = Path(f"/proc/self/fd/{target_fd}")
    subprocess.run(
        [
            mount_executable,
            "--bind",
            str(target_mount),
            str(target),
        ],
        stdin=subprocess.DEVNULL,
        pass_fds=inherited,
        check=True,
    )
    subprocess.run(
        [
            mount_executable,
            "-o",
            "remount,bind,rw",
            str(target),
        ],
        stdin=subprocess.DEVNULL,
        pass_fds=inherited,
        check=True,
    )
    for control_fd, control_path in (
        (target_marker_fd, target / ".codefire-source-key.json"),
        (target_lock_fd, target / ".codefire-source-key.lock"),
        (
            target_run_state_fd,
            target / ".codefire-source-key-run-state.json",
        ),
    ):
        subprocess.run(
            [
                mount_executable,
                "--bind",
                f"/proc/self/fd/{control_fd}",
                str(control_path),
            ],
            stdin=subprocess.DEVNULL,
            pass_fds=inherited,
            check=True,
        )
        subprocess.run(
            [
                mount_executable,
                "-o",
                "remount,bind,ro",
                str(control_path),
            ],
            stdin=subprocess.DEVNULL,
            pass_fds=inherited,
            check=True,
        )

    mountinfo = Path("/proc/self/mountinfo").read_text(
        encoding="utf-8", errors="strict"
    )
    prove_private_propagation(mountinfo)
    if not mount_is_read_only(source, mountinfo):
        raise RuntimeError("private source mount is not read-only")
    if not mount_is_read_only(snapshot_root, mountinfo):
        raise RuntimeError("snapshot root mask is not read-only")
    if not mount_is_read_only(toolchain_root, mountinfo):
        raise RuntimeError("toolchain bind mount is not read-only")
    if not mount_is_read_only(admitted_root, mountinfo):
        raise RuntimeError("admitted root bind mount is not read-only")
    if not mount_is_read_only(evidence_root, mountinfo):
        raise RuntimeError("evidence root mask is not read-only")
    if mount_is_read_only(target, mountinfo):
        raise RuntimeError("Cargo target bind mount is not writable")
    for control_path in (
        target / ".codefire-source-key.json",
        target / ".codefire-source-key.lock",
        target / ".codefire-source-key-run-state.json",
    ):
        if not mount_is_read_only(control_path, mountinfo):
            raise RuntimeError(
                f"control file bind mount is not read-only: {control_path}"
            )
    prove_read_only(source)
    prove_read_only(snapshot)
    prove_read_only(toolchain_root)
    prove_writable_directory(target)
    prove_regular_read_only(target / ".codefire-source-key.json")
    prove_regular_read_only(target / ".codefire-source-key.lock")
    prove_regular_read_only(
        target / ".codefire-source-key-run-state.json"
    )

    post_drop = [
        f"/proc/self/fd/{python_fd}",
        "-I",
        "-S",
        "-B",
        "-X",
        "pycache_prefix=/dev/null",
        f"/proc/self/fd/{helper_fd}",
        "--source",
        str(source),
        "--snapshot",
        str(snapshot),
        "--snapshot-root",
        str(snapshot_root),
        "--toolchain-root",
        str(toolchain_root),
        "--target",
        str(target),
        "--admitted-root",
        str(admitted_root),
        "--evidence-root",
        str(evidence_root),
        "--source-fd",
        str(source_fd),
        "--snapshot-fd",
        str(snapshot_fd),
        "--snapshot-root-fd",
        str(snapshot_root_fd),
        "--snapshot-marker-sha256",
        args.snapshot_marker_sha256,
        "--snapshot-file-manifest-sha256",
        args.snapshot_file_manifest_sha256,
        "--snapshot-file-count",
        str(args.snapshot_file_count),
        "--snapshot-logical-file-bytes",
        str(args.snapshot_logical_file_bytes),
        "--private-source-tmpfs-bytes",
        str(args.private_source_tmpfs_bytes),
        "--toolchain-root-fd",
        str(toolchain_root_fd),
        "--target-fd",
        str(target_fd),
        "--admitted-root-fd",
        str(admitted_root_fd),
        "--evidence-root-fd",
        str(evidence_root_fd),
        "--target-marker-fd",
        str(target_marker_fd),
        "--target-lock-fd",
        str(target_lock_fd),
        "--target-run-state-fd",
        str(target_run_state_fd),
        "--mount-fd",
        str(mount_fd),
        "--setpriv-fd",
        str(setpriv_fd),
        "--python-fd",
        str(python_fd),
        "--helper-fd",
        str(helper_fd),
        "--primary-fd",
        str(primary_fd),
        "--pass-fds",
        args.pass_fds,
        "--post-drop",
        "--",
        *args.command,
    ]
    setpriv_argv = [
        "setpriv",
        "--no-new-privs",
        "--bounding-set=-all",
        "--inh-caps=-all",
        "--ambient-caps=-all",
        "--",
        *post_drop,
    ]
    os.execve(setpriv_fd, setpriv_argv, dict(os.environ))
    return 127


if __name__ == "__main__":
    raise SystemExit(main())
