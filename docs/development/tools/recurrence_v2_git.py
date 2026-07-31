#!/usr/bin/env python3
"""Byte-safe Git plumbing for the recurrence gate v2 and the timed runner.

Paths are kept as ``bytes`` end to end.  A repository path is allowed to hold
bytes that are not valid UTF-8, so every comparison is a byte comparison and
every rendered form is JSON-safe: invalid bytes become ``\\xNN`` escapes rather
than lone surrogates that would make the result document unwritable.

The canonical patch is produced with an explicit, hermetic diff configuration so
its digest cannot drift with a user's Git settings.  Traversals are bounded and
carry an explicit visited set.
"""

from __future__ import annotations

import hashlib
import subprocess
from dataclasses import dataclass
from pathlib import Path

MAX_SERIES_COMMITS = 4096

# Hermetic configuration: every setting that could change diff or path output.
HERMETIC_CONFIG = (
    "-c",
    "core.autocrlf=false",
    "-c",
    "core.safecrlf=false",
    "-c",
    "core.quotePath=false",
    "-c",
    "diff.algorithm=myers",
    "-c",
    "diff.mnemonicPrefix=false",
    "-c",
    "diff.noprefix=false",
    "-c",
    "diff.suppressBlankEmpty=false",
    "-c",
    "diff.indentHeuristic=false",
    "-c",
    "diff.renames=false",
)

# Shared by every diff query.  Context and prefix flags belong only to the
# patch query: with --name-only or --numstat they would append patch bytes to
# the NUL-separated record stream.
DIFF_FLAGS = (
    "--no-ext-diff",
    "--no-textconv",
    "--no-renames",
    "--no-color",
)

PATCH_FLAGS = DIFF_FLAGS + (
    "-U3",
    "--inter-hunk-context=0",
    "--src-prefix=a/",
    "--dst-prefix=b/",
    "--binary",
    "--full-index",
)


class GitError(ValueError):
    """A Git query failed or returned something the caller cannot use."""


def render_path(raw: bytes) -> str:
    """Return a JSON-safe rendering of a repository path."""

    return raw.decode("utf-8", "backslashreplace")


def is_utf8(raw: bytes) -> bool:
    try:
        raw.decode("utf-8")
    except UnicodeDecodeError:
        return False
    return True


@dataclass(frozen=True)
class PathSet:
    """An exact set of repository paths, held as bytes."""

    raw: frozenset[bytes]

    @property
    def invalid_utf8(self) -> tuple[str, ...]:
        return tuple(sorted(render_path(item) for item in self.raw if not is_utf8(item)))

    def rendered(self) -> tuple[str, ...]:
        return tuple(sorted(render_path(item) for item in self.raw))

    def __len__(self) -> int:
        return len(self.raw)


@dataclass(frozen=True)
class TreeEntry:
    mode: str
    kind: str
    oid: str


def declared_paths(paths: frozenset[str] | tuple[str, ...]) -> frozenset[bytes]:
    return frozenset(path.encode("utf-8") for path in paths)


class GitRepo:
    """Read-only Git access rooted at one analysed repository."""

    def __init__(self, root: Path) -> None:
        self.root = root.resolve()
        self._blobs: dict[tuple[str, bytes], bytes] = {}
        self._entries: dict[tuple[str, bytes], TreeEntry | None] = {}

    def _run(
        self,
        args: tuple[str | bytes, ...],
        *,
        hermetic: bool = False,
    ) -> bytes:
        # Path arguments stay bytes so an invalid-UTF-8 repository path is
        # passed to Git exactly as Git stores it.
        command: list[str | bytes] = [
            "git",
            *(HERMETIC_CONFIG if hermetic else ()),
            *args,
        ]
        completed = subprocess.run(
            command,
            cwd=self.root,
            check=False,
            capture_output=True,
        )
        if completed.returncode != 0:
            detail = completed.stderr.decode("utf-8", "replace").strip()
            printable = " ".join(
                item if isinstance(item, str) else render_path(item) for item in args
            )
            raise GitError(detail or f"git {printable} failed")
        return completed.stdout

    def _text(self, args: tuple[str | bytes, ...]) -> str:
        return self._run(args).decode("utf-8", "replace").strip()

    def resolve_commit(self, revision: str) -> str:
        if not revision:
            raise GitError("empty revision")
        return self._text(("rev-parse", "--verify", f"{revision}^{{commit}}"))

    def tree_of(self, commit: str) -> str:
        return self._text(("rev-parse", "--verify", f"{commit}^{{tree}}"))

    def parents(self, commit: str) -> tuple[str, ...]:
        line = self._text(("rev-list", "--parents", "-n", "1", commit)).split()
        if not line:
            raise GitError(f"no commit for {commit}")
        return tuple(line[1:])

    def is_ancestor(self, ancestor: str, descendant: str) -> bool:
        completed = subprocess.run(
            ["git", "merge-base", "--is-ancestor", ancestor, descendant],
            cwd=self.root,
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        return completed.returncode == 0

    def changed_paths(self, base: str, head: str) -> PathSet:
        raw = self._run(
            ("diff", *DIFF_FLAGS, "--name-only", "-z", base, head, "--"),
            hermetic=True,
        )
        return PathSet(frozenset(item for item in raw.split(b"\0") if item))

    def write_set(self, commit: str) -> PathSet:
        """Exact paths one single-parent commit changes."""

        parents = self.parents(commit)
        if len(parents) != 1:
            raise GitError(
                f"{commit} has {len(parents)} parents; a single-parent commit is required"
            )
        return self.changed_paths(parents[0], commit)

    def canonical_patch(self, base: str, head: str) -> bytes:
        return self._run(("diff", *PATCH_FLAGS, base, head, "--"), hermetic=True)

    def numstat(self, base: str, head: str) -> tuple[int, int, tuple[str, ...]]:
        raw = self._run(
            ("diff", *DIFF_FLAGS, "--numstat", "-z", base, head, "--"),
            hermetic=True,
        )
        fields = [item for item in raw.split(b"\0") if item]
        inserted = 0
        deleted = 0
        binary: list[str] = []
        index = 0
        while index < len(fields):
            entry = fields[index]
            parts = entry.split(b"\t")
            if len(parts) < 3:
                raise GitError(f"unparsable numstat entry: {entry!r}")
            added, removed, path = parts[0], parts[1], b"\t".join(parts[2:])
            if added == b"-" or removed == b"-":
                binary.append(render_path(path))
            else:
                try:
                    inserted += int(added)
                    deleted += int(removed)
                except ValueError as exc:
                    raise GitError(f"unparsable numstat counts: {entry!r}") from exc
            index += 1
        return inserted, deleted, tuple(sorted(binary))

    def entry(self, commit: str, path: bytes) -> TreeEntry | None:
        key = (commit, path)
        if key in self._entries:
            return self._entries[key]
        raw = self._run(("ls-tree", "-z", commit, "--", path))
        result: TreeEntry | None = None
        for record in raw.split(b"\0"):
            if not record:
                continue
            meta, _, name = record.partition(b"\t")
            if name != path:
                continue
            pieces = meta.split(b" ")
            if len(pieces) != 3:
                continue
            result = TreeEntry(
                mode=pieces[0].decode("ascii", "replace"),
                kind=pieces[1].decode("ascii", "replace"),
                oid=pieces[2].decode("ascii", "replace"),
            )
        self._entries[key] = result
        return result

    def blob(self, commit: str, path: bytes) -> bytes:
        key = (commit, path)
        if key in self._blobs:
            return self._blobs[key]
        found = self.entry(commit, path)
        if found is None or found.kind != "blob":
            raise GitError(
                f"{render_path(path)} is not a blob at {commit}"
            )
        payload = self._run(("cat-file", "blob", found.oid))
        self._blobs[key] = payload
        return payload

    def linear_series(self, base: str, head: str) -> tuple[str, ...]:
        """Commits from ``base`` (exclusive) to ``head`` (inclusive).

        Every commit must have exactly one parent so each step is inspectable.
        The visited set makes the traversal terminate on any malformed graph
        instead of running to the commit bound silently.
        """

        series: list[str] = []
        visited: set[str] = set()
        cursor = head
        while cursor != base:
            if cursor in visited:
                raise GitError(f"cycle detected while walking to {base}")
            if len(series) >= MAX_SERIES_COMMITS:
                raise GitError(
                    f"more than {MAX_SERIES_COMMITS} commits between {base} and {head}"
                )
            visited.add(cursor)
            series.append(cursor)
            parents = self.parents(cursor)
            if len(parents) != 1:
                raise GitError(
                    f"{cursor} has {len(parents)} parents; the reviewed series must be linear"
                )
            cursor = parents[0]
        series.reverse()
        return tuple(series)

    def worktree_status(self) -> tuple[str, ...]:
        raw = self._run(
            ("status", "--porcelain=v1", "-z", "--untracked-files=all"),
        )
        return tuple(
            render_path(item) for item in raw.split(b"\0") if item
        )

    def index_flagged_entries(self) -> tuple[str, ...]:
        """Index entries whose tag is not ``H``.

        This is the stronger exact equivalent of naming ``assume-unchanged``
        (lowercase tags) and ``skip-worktree`` (``S``): any other tag is
        reported too, so the clean-status proof cannot be defeated by an index
        bit that suppresses modification reporting.
        """

        raw = self._run(("ls-files", "-v", "-z"))
        flagged: list[str] = []
        for record in raw.split(b"\0"):
            if not record or len(record) < 2:
                continue
            tag = record[0:1]
            path = record[2:] if record[1:2] == b" " else record[1:]
            if tag != b"H":
                flagged.append(f"{tag.decode('ascii', 'replace')}\t{render_path(path)}")
        return tuple(sorted(flagged))

    def toplevel(self) -> str:
        return self._text(("rev-parse", "--show-toplevel"))

    def head_identity(self) -> tuple[str, str]:
        return self.resolve_commit("HEAD"), self.tree_of("HEAD")


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()
