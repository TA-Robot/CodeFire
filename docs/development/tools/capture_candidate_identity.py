#!/usr/bin/env python3
"""Capture a fail-closed, manager-recomputed identity for an exact candidate."""

from __future__ import annotations

import argparse
import fnmatch
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

from ai_contracts import canonical_git_patch


def git_bytes(root: Path, *args: str, check: bool = True) -> bytes:
    result = subprocess.run(
        ["git", *args],
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if check and result.returncode != 0:
        message = result.stderr.decode("utf-8", errors="replace").strip()
        raise ValueError(message or f"git {' '.join(args)} failed")
    return result.stdout


def decode_path(value: bytes) -> str:
    return value.decode("utf-8", errors="surrogateescape")


def resolve_commit(root: Path, revision: str) -> str:
    return git_bytes(root, "rev-parse", "--verify", f"{revision}^{{commit}}").decode().strip()


def tree_entries(root: Path, revision: str) -> dict[str, dict[str, str]]:
    entries: dict[str, dict[str, str]] = {}
    raw = git_bytes(root, "ls-tree", "-rz", "--full-tree", revision)
    for record in raw.split(b"\0"):
        if not record:
            continue
        metadata, path_bytes = record.split(b"\t", 1)
        mode, object_type, object_id = metadata.decode("ascii").split(" ", 2)
        entries[decode_path(path_bytes)] = {
            "mode": mode,
            "type": object_type,
            "oid": object_id,
        }
    return entries


def changed_statuses(root: Path, base: str, head: str) -> dict[str, str]:
    raw = git_bytes(
        root,
        "diff",
        "--name-status",
        "-z",
        "--no-renames",
        base,
        head,
        "--",
    )
    fields = raw.split(b"\0")
    if fields and not fields[-1]:
        fields.pop()
    if len(fields) % 2:
        raise ValueError("git diff --name-status returned an incomplete record")
    return {
        decode_path(fields[index + 1]): fields[index].decode("ascii")
        for index in range(0, len(fields), 2)
    }


def changed_numstat(root: Path, base: str, head: str) -> dict[str, dict[str, int | None]]:
    raw = git_bytes(
        root,
        "diff",
        "--numstat",
        "-z",
        "--no-renames",
        base,
        head,
        "--",
    )
    result: dict[str, dict[str, int | None]] = {}
    for record in raw.split(b"\0"):
        if not record:
            continue
        added_raw, deleted_raw, path_bytes = record.split(b"\t", 2)

        def count(value: bytes) -> int | None:
            return None if value == b"-" else int(value)

        result[decode_path(path_bytes)] = {
            "added": count(added_raw),
            "deleted": count(deleted_raw),
        }
    return result


def repository_file(root: Path, path_ref: str) -> Path:
    unresolved = root / path_ref
    cursor = root
    for part in Path(path_ref).parts:
        cursor = cursor / part
        if cursor.is_symlink():
            raise ValueError(f"repository file path contains a symlink: {path_ref}")
    candidate = unresolved.resolve(strict=True)
    try:
        candidate.relative_to(root)
    except ValueError as exc:
        raise ValueError(f"repository file is outside the candidate worktree: {path_ref}") from exc
    if not candidate.is_file():
        raise ValueError(f"repository file is not regular: {path_ref}")
    return candidate


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_set_matches(path: str, pattern: str) -> bool:
    if pattern.endswith("/") and path.startswith(pattern):
        return True
    return path == pattern or fnmatch.fnmatchcase(path, pattern)


def load_manifest(
    root: Path,
    manifest_ref: str,
    task_id: str | None,
) -> tuple[dict[str, Any], dict[str, Any], str, str, str]:
    unresolved_manifest = Path(manifest_ref)
    if not unresolved_manifest.is_absolute():
        unresolved_manifest = root / unresolved_manifest
    if unresolved_manifest.is_symlink():
        raise ValueError(f"manifest path is a symlink: {manifest_ref}")
    manifest_path = unresolved_manifest.resolve(strict=True)
    if not manifest_path.is_file():
        raise ValueError(f"manifest is not a regular file: {manifest_ref}")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    tasks = manifest.get("tasks")
    if not isinstance(tasks, list) or not tasks:
        raise ValueError("manifest has no tasks")
    if task_id is None:
        if len(tasks) != 1:
            raise ValueError("--task-id is required for a multi-task manifest")
        task = tasks[0]
    else:
        matches = [item for item in tasks if item.get("id") == task_id]
        if len(matches) != 1:
            raise ValueError(f"manifest task id must match exactly once: {task_id}")
        task = matches[0]
    ticket_ref = task.get("ticket")
    if not isinstance(ticket_ref, str) or not ticket_ref:
        raise ValueError("selected manifest task has no ticket")
    ticket_path = repository_file(root, ticket_ref)
    return (
        manifest,
        task,
        str(manifest_path),
        file_sha256(manifest_path),
        file_sha256(ticket_path),
    )


def capture_candidate_identity(
    root: Path,
    base_revision: str,
    head_revision: str = "HEAD",
    *,
    manifest_ref: str | None = None,
    task_id: str | None = None,
) -> dict[str, Any]:
    root = root.resolve(strict=True)
    repository_root = Path(
        git_bytes(root, "rev-parse", "--show-toplevel").decode().strip()
    ).resolve(strict=True)
    if repository_root != root:
        raise ValueError(
            f"--repo must be the worktree root: expected {repository_root}, got {root}"
        )

    patch, changed_paths, base, head = canonical_git_patch(
        root, base_revision, head_revision
    )
    checkout_head = resolve_commit(root, "HEAD")
    tree = git_bytes(root, "rev-parse", f"{head}^{{tree}}").decode().strip()
    merge_base = git_bytes(root, "merge-base", base, head).decode().strip()
    parent_fields = (
        git_bytes(root, "rev-list", "--parents", "-n", "1", head).decode().split()
    )
    parents = parent_fields[1:]
    commits = [
        line
        for line in git_bytes(
            root, "rev-list", "--reverse", "--topo-order", f"{base}..{head}"
        )
        .decode()
        .splitlines()
        if line
    ]
    merge_commits = [
        line
        for line in git_bytes(
            root, "rev-list", "--min-parents=2", f"{base}..{head}"
        )
        .decode()
        .splitlines()
        if line
    ]
    branch = git_bytes(
        root, "symbolic-ref", "--short", "-q", "HEAD", check=False
    ).decode().strip()
    dirty_lines = [
        line
        for line in git_bytes(
            root,
            "-c",
            "core.quotepath=false",
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
        )
        .decode("utf-8", errors="surrogateescape")
        .splitlines()
        if line
    ]

    statuses = changed_statuses(root, base, head)
    numstat = changed_numstat(root, base, head)
    base_tree = tree_entries(root, base)
    head_tree = tree_entries(root, head)
    if changed_paths != set(statuses) or changed_paths != set(numstat):
        raise ValueError("canonical path, name-status, and numstat sets differ")
    changed = [
        {
            "path": path,
            "status": statuses[path],
            "numstat": numstat[path],
            "base_object": base_tree.get(path),
            "head_object": head_tree.get(path),
        }
        for path in sorted(changed_paths)
    ]

    blockers: list[str] = []
    if checkout_head != head:
        blockers.append("checkout HEAD differs from the requested candidate")
    if merge_base != base:
        blockers.append("task base is not the exact merge base")
    if not commits:
        blockers.append("candidate contains no additive commit")
    if merge_commits:
        blockers.append("candidate range contains a merge commit")
    if dirty_lines:
        blockers.append("candidate worktree has tracked or untracked changes")

    manifest_evidence: dict[str, Any] | None = None
    if manifest_ref is not None:
        manifest, task, manifest_path, manifest_digest, ticket_digest = load_manifest(
            root, manifest_ref, task_id
        )
        write_set = task.get("write_set")
        if not isinstance(write_set, list) or not all(
            isinstance(item, str) and item for item in write_set
        ):
            raise ValueError("selected manifest task has an invalid write_set")
        outside_write_set = [
            path
            for path in sorted(changed_paths)
            if not any(write_set_matches(path, pattern) for pattern in write_set)
        ]
        if manifest.get("base_revision") != base:
            blockers.append("manifest base_revision differs from the exact task base")
        if outside_write_set:
            blockers.append("candidate changed paths outside the declared write_set")
        manifest_evidence = {
            "path": manifest_path,
            "sha256": manifest_digest,
            "batch_id": manifest.get("batch_id"),
            "base_revision": manifest.get("base_revision"),
            "task_id": task.get("id"),
            "role": task.get("role"),
            "tier": task.get("tier"),
            "provider": task.get("provider"),
            "ticket": task.get("ticket"),
            "ticket_sha256": ticket_digest,
            "write_set": write_set,
            "outside_write_set": outside_write_set,
        }

    return {
        "schema": "codefire.candidate_identity.v1",
        "decision": "READY_FOR_REVIEW" if not blockers else "BLOCK",
        "blocking_reasons": blockers,
        "repository": str(root),
        "branch": branch or None,
        "base_revision": base,
        "merge_base": merge_base,
        "parent_revisions": parents,
        "head_revision": head,
        "checkout_head_revision": checkout_head,
        "tree": tree,
        "commit_count": len(commits),
        "commits": commits,
        "merge_commits": merge_commits,
        "patch_sha256": hashlib.sha256(patch).hexdigest(),
        "changed_paths": changed,
        "clean": not dirty_lines,
        "porcelain": dirty_lines,
        "manifest": manifest_evidence,
    }


def markdown_card(identity: dict[str, Any]) -> str:
    lines = [
        "# Candidate-ready identity",
        "",
        f"Status: `{identity['decision']}`",
        "",
        "| Field | Manager-recomputed value |",
        "|---|---|",
        f"| Repository | `{identity['repository']}` |",
        f"| Branch | `{identity['branch'] or 'DETACHED'}` |",
        f"| Base | `{identity['base_revision']}` |",
        f"| Merge base | `{identity['merge_base']}` |",
        f"| Parent(s) | `{' '.join(identity['parent_revisions'])}` |",
        f"| Head | `{identity['head_revision']}` |",
        f"| Tree | `{identity['tree']}` |",
        f"| Additive commits | `{identity['commit_count']}` |",
        f"| Binary patch SHA-256 | `{identity['patch_sha256']}` |",
        f"| Clean | `{str(identity['clean']).lower()}` |",
        "",
        "## Changed paths",
        "",
        "| Status | Path | Added | Deleted | Base object | Head object |",
        "|---|---|---:|---:|---|---|",
    ]
    for entry in identity["changed_paths"]:
        numstat = entry["numstat"]
        base_oid = (entry["base_object"] or {}).get("oid", "-")
        head_oid = (entry["head_object"] or {}).get("oid", "-")
        added = "-" if numstat["added"] is None else str(numstat["added"])
        deleted = "-" if numstat["deleted"] is None else str(numstat["deleted"])
        lines.append(
            f"| `{entry['status']}` | `{entry['path']}` | {added} | {deleted} | "
            f"`{base_oid}` | `{head_oid}` |"
        )
    lines.extend(["", "## Additive commits", ""])
    lines.extend(f"- `{commit}`" for commit in identity["commits"])
    if identity["manifest"] is not None:
        manifest = identity["manifest"]
        lines.extend(
            [
                "",
                "## Task binding",
                "",
                f"- manifest: `{manifest['path']}`",
                f"- manifest SHA-256: `{manifest['sha256']}`",
                f"- task: `{manifest['task_id']}`",
                f"- ticket: `{manifest['ticket']}`",
                f"- ticket SHA-256: `{manifest['ticket_sha256']}`",
                f"- outside declared write set: "
                f"`{json.dumps(manifest['outside_write_set'], ensure_ascii=True)}`",
            ]
        )
    lines.extend(["", "## Blocking reasons", ""])
    if identity["blocking_reasons"]:
        lines.extend(f"- {reason}" for reason in identity["blocking_reasons"])
    else:
        lines.append("- none")
    lines.extend(
        [
            "",
            "Validation evidence, provider/model/session/run identity, checkpoint "
            "history, and process deviations must be appended from independent "
            "manager evidence before review launch.",
            "",
        ]
    )
    return "\n".join(lines)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Recompute an exact candidate identity and fail closed on unsafe state."
    )
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", default="HEAD")
    parser.add_argument("--manifest")
    parser.add_argument("--task-id")
    parser.add_argument("--format", choices=("json", "markdown"), default="json")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        identity = capture_candidate_identity(
            args.repo,
            args.base,
            args.head,
            manifest_ref=args.manifest,
            task_id=args.task_id,
        )
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(f"candidate identity error: {exc}", file=sys.stderr)
        return 2
    if args.format == "markdown":
        print(markdown_card(identity), end="")
    else:
        print(json.dumps(identity, indent=2, sort_keys=True, ensure_ascii=True))
    return 0 if identity["decision"] == "READY_FOR_REVIEW" else 2


if __name__ == "__main__":
    raise SystemExit(main())
