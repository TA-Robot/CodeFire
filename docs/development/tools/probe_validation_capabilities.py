#!/usr/bin/env python3
"""Report local validation capabilities without executing product tests."""

from __future__ import annotations

import argparse
import json
import shutil
import sys
from pathlib import Path

sys.dont_write_bytecode = True
sys.pycache_prefix = "/dev/null"

from build_source_bound_rust import (
    BINARY_PATH,
    CanonicalProvenanceChanged,
    DEFAULT_OUTPUT,
    git_head,
    source_paths,
    validate_locked_canonical_provenance,
)


CAPABILITY_IDS = {
    "bash",
    "cargo",
    "codefire",
    "codex",
    "git",
    "python3",
    "rust-binary-present",
    "rust-source-bound-binary",
    "timeout",
}


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def newest_mtime(paths: list[Path]) -> float | None:
    newest: float | None = None
    for path in paths:
        if path.is_file():
            observed = path.stat().st_mtime
            newest = (
                observed if newest is None else max(newest, observed)
            )
    return newest


def rust_binary_state(root: Path) -> dict[str, object]:
    try:
        source_mtime = newest_mtime(source_paths(root))
    except (
        FileNotFoundError,
        OSError,
        TimeoutError,
        UnicodeDecodeError,
        ValueError,
    ):
        source_mtime = None
    provenance_path = root / DEFAULT_OUTPUT
    provenance_errors: list[str] = []
    provenance: dict[str, object] | None = None
    provenance_present = provenance_path.exists() or provenance_path.is_symlink()
    if provenance_present:
        try:
            loaded, _identity, provenance_errors = (
                validate_locked_canonical_provenance(
                    root,
                    relative=DEFAULT_OUTPUT,
                    expected_revision=git_head(root),
                )
            )
            provenance = loaded
        except (
            CanonicalProvenanceChanged,
            FileNotFoundError,
            json.JSONDecodeError,
            OSError,
            TimeoutError,
            UnicodeDecodeError,
            ValueError,
        ) as exc:
            provenance_errors = [str(exc)]
    source_revision_bound = provenance is not None and not provenance_errors
    candidates = [root / BINARY_PATH, root / "target/debug/codefire-rs"]
    binaries: list[dict[str, object]] = []
    for path in candidates:
        if not path.is_file():
            continue
        mtime = path.stat().st_mtime
        binaries.append(
            {
                "path": path.relative_to(root).as_posix(),
                "bytes": path.stat().st_size,
                "newer_than_sources": source_mtime is not None and mtime >= source_mtime,
                "source_revision_bound": source_revision_bound and path == root / BINARY_PATH,
            }
        )
    return {
        "available": bool(binaries),
        "source_revision_bound": source_revision_bound,
        "note": "source binding requires a valid clean-HEAD provenance artifact; mtime ordering is informational only",
        "provenance": {
            "path": DEFAULT_OUTPUT.as_posix(),
            "present": provenance_present,
            "valid": source_revision_bound,
            "base_revision": provenance.get("base_revision") if provenance else None,
            "errors": provenance_errors,
        },
        "binaries": binaries,
    }


def probe(root: Path) -> dict[str, object]:
    executables = {
        name: shutil.which(name)
        for name in ("bash", "cargo", "codefire", "codex", "git", "python3", "rustc", "timeout")
    }
    binary = rust_binary_state(root)
    capabilities = {
        "bash": executables["bash"] is not None,
        "cargo": executables["cargo"] is not None and executables["rustc"] is not None,
        "codefire": executables["codefire"] is not None,
        "codex": executables["codex"] is not None,
        "rust-binary-present": bool(binary["available"]),
        "rust-source-bound-binary": bool(binary["source_revision_bound"]),
        "git": executables["git"] is not None,
        "python3": executables["python3"] is not None,
        "timeout": executables["timeout"] is not None,
    }
    if set(capabilities) != CAPABILITY_IDS:
        raise AssertionError("capability probe and registry vocabulary differ")
    return {
        "schema": "codefire.validation_capabilities.v1",
        "ok": True,
        "capabilities": capabilities,
        "executables": executables,
        "rust_binary": binary,
        "gates": {
            "ai_development": capabilities["python3"] and capabilities["codex"],
            "docs": capabilities["python3"],
            "python_reference": capabilities["python3"],
            "rust_source": capabilities["cargo"],
            "rust_binary_smoke": capabilities["rust-binary-present"],
            "rust_current_source_binary": capabilities["rust-source-bound-binary"],
        },
        "limitations": [
            message
            for condition, message in (
                (not capabilities["cargo"], "Rust source gates are unavailable because cargo/rustc are missing."),
                (not capabilities["rust-source-bound-binary"], "No Rust binary is cryptographically or provenance-bound to the current worktree; existing binaries are smoke-only."),
                (not capabilities["codefire"], "The installed codefire command is unavailable; use an explicit binary path for dogfood."),
            )
            if condition
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=repository_root())
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    payload = probe(args.root.resolve())
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        for gate, ready in payload["gates"].items():
            print(f"{'READY' if ready else 'BLOCKED'} {gate}")
        for limitation in payload["limitations"]:
            print(f"- {limitation}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
