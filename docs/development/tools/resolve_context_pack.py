#!/usr/bin/env python3
"""Resolve context-pack inheritance into a content-addressed file manifest."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

from ai_contracts import (
    CONTEXT_MANIFEST_SCHEMA,
    canonical_json_bytes,
    load_object,
    repository_path_valid,
    validate_context_manifest,
)


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def current_revision(root: Path) -> str:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=root, capture_output=True, text=True, check=False
    )
    if result.returncode != 0:
        raise ValueError((result.stderr or result.stdout).strip() or "git rev-parse HEAD failed")
    return result.stdout.strip()


def resolve_pack_ids(packs: dict[str, dict[str, Any]], requested: list[str]) -> list[str]:
    resolved: list[str] = []
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(pack_id: str) -> None:
        if pack_id in visited:
            return
        if pack_id in visiting:
            raise ValueError(f"context-pack inheritance cycle at {pack_id}")
        pack = packs.get(pack_id)
        if pack is None:
            raise ValueError(f"unknown context pack: {pack_id}")
        visiting.add(pack_id)
        for parent in pack.get("extends", []):
            visit(parent)
        visiting.remove(pack_id)
        visited.add(pack_id)
        resolved.append(pack_id)

    for pack_id in requested:
        visit(pack_id)
    return resolved


def expand_reference(root: Path, reference: str) -> list[Path]:
    if not repository_path_valid(reference):
        raise ValueError(f"unsafe repository path: {reference!r}")
    if any(character in reference for character in "*?["):
        paths = sorted(path for path in root.glob(reference) if path.is_file() or path.is_symlink())
        if not paths:
            raise ValueError(f"context glob matched no files: {reference}")
        return [safe_repository_file(root, path, reference) for path in paths]
    return [safe_repository_file(root, root / reference, reference)]


def safe_repository_file(root: Path, candidate: Path, reference: str) -> Path:
    root = root.resolve()
    lexical = candidate if candidate.is_absolute() else root / candidate
    try:
        relative = lexical.relative_to(root)
    except ValueError as exc:
        raise ValueError(f"context path escapes repository: {reference}") from exc
    cursor = root
    for part in relative.parts:
        cursor = cursor / part
        if cursor.is_symlink():
            raise ValueError(f"context path contains a symlink: {reference}")
    try:
        path = lexical.resolve(strict=True)
    except FileNotFoundError as exc:
        raise ValueError(f"context file is missing: {reference}")
    try:
        path.relative_to(root)
    except ValueError as exc:
        raise ValueError(f"context path resolves outside repository: {reference}") from exc
    if not path.is_file():
        raise ValueError(f"context path is not a regular file: {reference}")
    return path


def build_manifest(
    root: Path,
    requested: list[str],
    additional: list[str],
    include_likely: bool,
    base_revision: str,
) -> dict[str, Any]:
    catalog = load_object(root / "docs/development/ai/context-packs.json")
    packs = {pack["id"]: pack for pack in catalog["packs"]}
    resolved_ids = resolve_pack_ids(packs, requested)
    sources: dict[str, str] = {}
    for pack_id in resolved_ids:
        keys = ["required_files"] + (["likely_files"] if include_likely else [])
        for key in keys:
            for reference in packs[pack_id].get(key, []):
                for path in expand_reference(root, reference):
                    relative = path.relative_to(root).as_posix()
                    sources.setdefault(relative, pack_id)
    for reference in additional:
        for path in expand_reference(root, reference):
            sources.setdefault(path.relative_to(root).as_posix(), "additional")

    files: list[dict[str, Any]] = []
    for relative in sorted(sources):
        safe_path = safe_repository_file(root, root / relative, relative)
        payload = safe_path.read_bytes()
        files.append(
            {
                "path": relative,
                "sha256": hashlib.sha256(payload).hexdigest(),
                "bytes": len(payload),
                "source_pack": sources[relative],
            }
        )
    if not files:
        raise ValueError("resolved context is empty")
    digest_payload = {
        "schema": CONTEXT_MANIFEST_SCHEMA,
        "base_revision": base_revision,
        "pack_ids": resolved_ids,
        "files": files,
    }
    digest = hashlib.sha256(canonical_json_bytes(digest_payload)).hexdigest()
    manifest = {
        "schema": CONTEXT_MANIFEST_SCHEMA,
        "manifest_id": f"context-{digest[:16]}",
        "base_revision": base_revision,
        "pack_ids": resolved_ids,
        "files": files,
        "manifest_sha256": digest,
    }
    schema = load_object(root / "docs/development/ai/schemas/context-manifest.v1.schema.json")
    schema_findings = validate_context_manifest(
        manifest,
        "context-manifest",
        schema,
        root=root,
        verify_workspace=True,
    )
    if schema_findings:
        raise ValueError(
            "generated context manifest violates schema: "
            + "; ".join(f"{finding.path}: {finding.message}" for finding in schema_findings)
        )
    return manifest


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack_id", nargs="+")
    parser.add_argument("--root", type=Path, default=repository_root())
    parser.add_argument("--additional", action="append", default=[])
    parser.add_argument("--include-likely", action="store_true")
    parser.add_argument("--base-revision")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--force", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    try:
        base_revision = args.base_revision or current_revision(root)
        manifest = build_manifest(
            root, args.pack_id, args.additional, args.include_likely, base_revision
        )
        rendered = json.dumps(manifest, indent=2, sort_keys=True) + "\n"
        if args.output is None:
            print(rendered, end="")
        else:
            output = (root / args.output).resolve() if not args.output.is_absolute() else args.output.resolve()
            try:
                output.relative_to(root)
            except ValueError as exc:
                raise ValueError("output must stay inside repository") from exc
            if output.exists() and not args.force:
                raise ValueError(f"output exists; pass --force to replace: {output}")
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text(rendered, encoding="utf-8")
    except (FileNotFoundError, KeyError, json.JSONDecodeError, TypeError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
