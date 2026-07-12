#!/usr/bin/env python3
"""Build the Rust CLI and emit provenance bound to unchanged source inputs."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

from ai_contracts import sha256_json


BUILD_COMMAND = [
    "cargo", "build", "--release", "--locked", "-p", "codefire-cli", "--bin", "codefire-rs",
]
BINARY_PATH = Path("target/release/codefire-rs")
DEFAULT_OUTPUT = Path("target/codefire-ai/rust-build-provenance.json")


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def command_text(command: list[str]) -> str:
    return " ".join(command)


def run_text(root: Path, command: list[str], timeout: float) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=root,
        capture_output=True,
        text=True,
        timeout=timeout,
        check=False,
    )


def source_paths(root: Path) -> list[Path]:
    result = subprocess.run(
        [
            "git",
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
        cwd=root,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise ValueError(result.stderr.decode("utf-8", errors="replace").strip() or "git ls-files failed")
    paths: list[Path] = []
    for raw in result.stdout.split(b"\0"):
        if not raw:
            continue
        relative = raw.decode("utf-8", errors="strict")
        candidate = root / relative
        cursor = root
        for part in Path(relative).parts:
            cursor = cursor / part
            if cursor.is_symlink():
                raise ValueError(f"Rust build input contains a symlink: {relative}")
        resolved = candidate.resolve(strict=True)
        resolved.relative_to(root)
        if not resolved.is_file():
            raise ValueError(f"Rust build input is not a regular file: {relative}")
        paths.append(resolved)
    if not paths:
        raise ValueError("Rust source input set is empty")
    return sorted(set(paths), key=lambda path: path.relative_to(root).as_posix())


def build_source_manifest(root: Path) -> dict[str, Any]:
    files = []
    for path in source_paths(root):
        payload = path.read_bytes()
        files.append(
            {
                "path": path.relative_to(root).as_posix(),
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    return {
        "schema": "codefire.rust_build_inputs.v1",
        "files": files,
        "manifest_sha256": sha256_json(files),
    }


def git_head(root: Path) -> str:
    result = run_text(root, ["git", "rev-parse", "HEAD"], 10.0)
    if result.returncode != 0:
        raise ValueError((result.stderr or result.stdout).strip() or "git rev-parse failed")
    return result.stdout.strip()


def validate_source_inputs_against_head(root: Path, expected_revision: str) -> list[str]:
    """Require every build input to be tracked and byte-identical to expected HEAD."""

    head = subprocess.run(
        ["git", "rev-parse", "--verify", f"{expected_revision}^{{commit}}"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    if head.returncode != 0:
        return [(head.stderr or head.stdout).strip() or "expected revision is not a commit"]
    resolved_head = head.stdout.strip()
    if resolved_head != expected_revision:
        return [f"expected revision must be the canonical commit SHA {resolved_head}"]
    listed = subprocess.run(
        [
            "git", "ls-tree", "-r", "--name-only", "-z", resolved_head, "--",
            "Cargo.toml", "Cargo.lock", ".cargo", "rust-toolchain",
            "rust-toolchain.toml", "crates",
        ],
        cwd=root,
        capture_output=True,
        check=False,
    )
    if listed.returncode != 0:
        return [listed.stderr.decode("utf-8", errors="replace").strip() or "git ls-tree failed"]
    head_paths = {
        item.decode("utf-8", errors="strict")
        for item in listed.stdout.split(b"\0")
        if item
    }
    try:
        current_paths = {
            path.relative_to(root).as_posix()
            for path in source_paths(root)
        }
    except (FileNotFoundError, UnicodeDecodeError, ValueError) as exc:
        return [str(exc)]
    errors: list[str] = []
    if current_paths != head_paths:
        errors.append(
            "build input path set differs from HEAD: "
            f"untracked_or_extra={sorted(current_paths - head_paths)}, "
            f"missing={sorted(head_paths - current_paths)}"
        )
    for relative in sorted(current_paths & head_paths):
        blob = subprocess.run(
            ["git", "show", f"{resolved_head}:{relative}"],
            cwd=root,
            capture_output=True,
            check=False,
        )
        if blob.returncode != 0:
            errors.append(f"HEAD blob is unavailable: {relative}")
            continue
        if (root / relative).read_bytes() != blob.stdout:
            errors.append(f"build input differs from HEAD: {relative}")
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
        candidate = root / BINARY_PATH
        if candidate.is_symlink():
            errors.append("provenance binary path is a symlink")
        else:
            try:
                binary = candidate.resolve(strict=True)
                binary.relative_to(root.resolve())
                binary_bytes = binary.read_bytes()
            except (FileNotFoundError, OSError, ValueError) as exc:
                errors.append(f"provenance binary is unavailable: {exc}")
            else:
                if payload.get("binary_bytes") != len(binary_bytes):
                    errors.append("provenance binary size differs from the current binary")
                if payload.get("binary_sha256") != hashlib.sha256(binary_bytes).hexdigest():
                    errors.append("provenance binary digest differs from the current binary")
    if payload.get("build_command") != command_text(BUILD_COMMAND):
        errors.append("provenance build command is not the canonical release build")
    return errors


def write_provenance(root: Path, output: Path, payload: dict[str, Any]) -> tuple[Path, str]:
    destination = (root / output).resolve() if not output.is_absolute() else output.resolve()
    destination.relative_to(root)
    destination.parent.mkdir(parents=True, exist_ok=True)
    rendered = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
    temporary = destination.with_name(f".{destination.name}.tmp")
    temporary.write_text(rendered, encoding="utf-8")
    temporary.replace(destination)
    return destination, hashlib.sha256(rendered.encode("utf-8")).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=repository_root())
    parser.add_argument("--timeout", type=float, default=1200.0)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()
    root = args.root.resolve()
    started = time.monotonic()
    try:
        base_revision = git_head(root)
        source_errors = validate_source_inputs_against_head(root, base_revision)
        if source_errors:
            raise ValueError("; ".join(source_errors))
        before = build_source_manifest(root)
        cargo_version = run_text(root, ["cargo", "--version"], 10.0)
        rustc_version = run_text(root, ["rustc", "--version"], 10.0)
        if cargo_version.returncode != 0 or rustc_version.returncode != 0:
            raise ValueError("cargo and rustc versions must both be available")
        build = run_text(root, BUILD_COMMAND, args.timeout)
        build_log = build.stdout.encode("utf-8") + b"\0" + build.stderr.encode("utf-8")
        if build.returncode != 0:
            payload = {
                "schema": "codefire.rust_build_provenance.v1",
                "ok": False,
                "build_command": command_text(BUILD_COMMAND),
                "build_returncode": build.returncode,
                "build_log_sha256": hashlib.sha256(build_log).hexdigest(),
                "stdout_tail": build.stdout[-4000:],
                "stderr_tail": build.stderr[-4000:],
            }
            print(json.dumps(payload, indent=2, sort_keys=True))
            return 1
        after = build_source_manifest(root)
        if after != before:
            raise ValueError("Rust source inputs changed during the build; provenance rejected")
        if git_head(root) != base_revision:
            raise ValueError("HEAD changed during the build; provenance rejected")
        source_errors = validate_source_inputs_against_head(root, base_revision)
        if source_errors:
            raise ValueError("; ".join(source_errors))
        binary = (root / BINARY_PATH).resolve(strict=True)
        binary.relative_to(root)
        binary_bytes = binary.read_bytes()
        payload = {
            "schema": "codefire.rust_build_provenance.v1",
            "ok": True,
            "base_revision": base_revision,
            "source_manifest_sha256": before["manifest_sha256"],
            "source_file_count": len(before["files"]),
            "cargo_version": cargo_version.stdout.strip(),
            "rustc_version": rustc_version.stdout.strip(),
            "build_command": command_text(BUILD_COMMAND),
            "build_log_sha256": hashlib.sha256(build_log).hexdigest(),
            "binary_path": BINARY_PATH.as_posix(),
            "binary_bytes": len(binary_bytes),
            "binary_sha256": hashlib.sha256(binary_bytes).hexdigest(),
            "duration_ms": round((time.monotonic() - started) * 1000),
        }
        output, artifact_sha256 = write_provenance(root, args.output, payload)
        payload["provenance_path"] = output.relative_to(root).as_posix()
        payload["provenance_sha256"] = artifact_sha256
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0
    except subprocess.TimeoutExpired as exc:
        print(json.dumps({"schema": "codefire.rust_build_provenance.v1", "ok": False, "error": f"timeout after {exc.timeout}s"}, indent=2, sort_keys=True))
        return 1
    except (FileNotFoundError, UnicodeDecodeError, ValueError) as exc:
        print(json.dumps({"schema": "codefire.rust_build_provenance.v1", "ok": False, "error": str(exc)}, indent=2, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
