#!/usr/bin/env python3
"""Report local validation capabilities without executing product tests."""

from __future__ import annotations

import argparse
import json
import shutil
import sys
from pathlib import Path


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
    mtimes = [path.stat().st_mtime for path in paths if path.is_file()]
    return max(mtimes) if mtimes else None


def rust_binary_state(root: Path) -> dict[str, object]:
    sources = list((root / "crates").rglob("*.rs"))
    sources.extend([root / "Cargo.toml", root / "Cargo.lock"])
    source_mtime = newest_mtime(sources)
    candidates = [root / "target/release/codefire-rs", root / "target/debug/codefire-rs"]
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
                "source_revision_bound": False,
            }
        )
    return {
        "available": bool(binaries),
        "source_revision_bound": False,
        "note": "mtime ordering is informational and never proves that a binary was built from this worktree",
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
