#!/usr/bin/env python3
"""Build or explicitly execute a guarded Codex task for CodeFire development."""

from __future__ import annotations

import argparse
import fnmatch
import hashlib
import json
import re
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any

from ai_contracts import (
    load_object,
    packet_digest,
    validate_context_catalog,
    validate_context_manifest,
    validate_task_packet,
)
from probe_validation_capabilities import probe
from resolve_context_pack import build_manifest, expand_reference


MATRIX_PATH = Path("docs/development/ai/role-routing-matrix.json")
CAPABILITY_REGISTRY_PATH = Path("docs/development/ai/capability-registry.v1.json")
CODEX_CLI_VERSION_PATTERN = re.compile(r"codex-cli\s+(\d+)\.(\d+)(?:\.(\d+))?")


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def load_matrix(root: Path) -> dict[str, Any]:
    data = json.loads((root / MATRIX_PATH).read_text(encoding="utf-8"))
    if data.get("schema") != "codefire.ai_role_routing.v2":
        raise ValueError("role-routing matrix must use codefire.ai_role_routing.v2")
    return data


def select_role(matrix: dict[str, Any], name: str) -> dict[str, Any]:
    roles = matrix.get("roles", [])
    for role in roles:
        if isinstance(role, dict) and role.get("role") == name:
            return role
    available = sorted(role.get("role") for role in roles if isinstance(role, dict) and isinstance(role.get("role"), str))
    raise ValueError(f"unknown role {name!r}; choose one of: {', '.join(available)}")


def codex_cli_adapter_key(version_output: str) -> str:
    match = CODEX_CLI_VERSION_PATTERN.search(version_output)
    if match is None:
        raise ValueError(f"cannot parse Codex CLI version: {version_output.strip()!r}")
    return f"codex_cli_{match.group(1)}_{match.group(2)}"


def select_codex_cli_adapter(
    matrix: dict[str, Any],
    version_output: str,
) -> tuple[str, dict[str, Any]]:
    adapter_key = codex_cli_adapter_key(version_output)
    adapters = matrix.get("runtime_policy", {}).get("adapters", {})
    adapter = adapters.get(adapter_key) if isinstance(adapters, dict) else None
    if not isinstance(adapter, dict):
        raise ValueError(
            f"Codex CLI {adapter_key.removeprefix('codex_cli_').replace('_', '.')} "
            f"has no registered runtime adapter ({adapter_key})"
        )
    return adapter_key, adapter


def installed_codex_cli_adapter(
    matrix: dict[str, Any],
    root: Path,
    timeout: float = 10.0,
) -> tuple[str, dict[str, Any]]:
    try:
        result = subprocess.run(
            ["codex", "--version"],
            cwd=root,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
    except subprocess.TimeoutExpired as exc:
        raise ValueError(f"codex --version exceeded {timeout}s") from exc
    if result.returncode != 0:
        raise ValueError(
            (result.stderr or result.stdout).strip() or "codex --version failed"
        )
    return select_codex_cli_adapter(matrix, result.stdout)


def contained_path(root: Path, value: Path) -> Path:
    resolved = value.resolve()
    try:
        resolved.relative_to(root)
    except ValueError as exc:
        raise ValueError(f"path must stay inside repository: {resolved}") from exc
    return resolved


def task_text(root: Path, packet: Path | None, prompt: str | None) -> tuple[str, Path | None]:
    parts: list[str] = []
    packet_path: Path | None = None
    if packet is not None:
        packet_path = contained_path(root, packet if packet.is_absolute() else root / packet)
        parts.append(
            f"Task packet: {packet_path.relative_to(root).as_posix()}\n\n"
            f"{packet_path.read_text(encoding='utf-8')}"
        )
    if prompt:
        parts.append(f"Additional instruction:\n\n{prompt}")
    if not parts and not sys.stdin.isatty():
        stdin_text = sys.stdin.read().strip()
        if stdin_text:
            parts.append(stdin_text)
    if not parts:
        raise ValueError("provide --packet, --prompt, or piped stdin")
    return "\n\n---\n\n".join(parts), packet_path


def validated_json_packet(
    root: Path,
    path: Path | None,
    matrix: dict[str, Any],
) -> dict[str, Any] | None:
    if path is None or path.suffix.lower() != ".json":
        return None
    packet = load_object(path)
    catalog = load_object(root / "docs/development/ai/context-packs.json")
    task_schema = load_object(root / "docs/development/ai/schemas/task-packet.v1.schema.json")
    catalog_findings, context_ids = validate_context_catalog(catalog)
    findings = catalog_findings + validate_task_packet(
        packet, matrix, context_ids, path.relative_to(root).as_posix(), task_schema
    )
    errors = [finding for finding in findings if finding.level == "error"]
    if errors:
        details = "; ".join(f"{finding.code}: {finding.message}" for finding in errors)
        raise ValueError(f"invalid JSON task packet: {details}")
    return packet


def current_head(root: Path) -> str:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise ValueError((result.stderr or result.stdout).strip() or "git rev-parse HEAD failed")
    return result.stdout.strip()


def dirty_paths(root: Path) -> set[str]:
    result = subprocess.run(
        ["git", "status", "--porcelain=v1", "-z", "--untracked-files=all"],
        cwd=root,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise ValueError(result.stderr.decode("utf-8", errors="replace").strip() or "git status failed")
    records = result.stdout.decode("utf-8", errors="surrogateescape").split("\0")
    paths: set[str] = set()
    index = 0
    while index < len(records):
        record = records[index]
        if not record:
            index += 1
            continue
        if len(record) < 4 or record[2] != " ":
            raise ValueError(f"unexpected git status record: {record!r}")
        status = record[:2]
        paths.add(record[3:])
        if "R" in status or "C" in status:
            index += 1
            if index < len(records) and records[index]:
                paths.add(records[index])
        index += 1
    return paths


def dirty_write_paths(packet: dict[str, Any], changed: set[str]) -> list[str]:
    write_set = packet.get("scope", {}).get("write_set", [])
    return sorted(
        path
        for path in changed
        if any(fnmatch.fnmatchcase(path, pattern) for pattern in write_set)
    )


def preflight_task_execution(
    packet: dict[str, Any],
    head_revision: str,
    changed: set[str],
    sandbox: str,
) -> tuple[bool, list[str], list[str]]:
    base_matches = packet.get("base_revision") == head_revision
    changed_write_paths = dirty_write_paths(packet, changed)
    errors: list[str] = []
    if not base_matches:
        errors.append(
            f"task base_revision {packet.get('base_revision')!r} differs from HEAD {head_revision!r}"
        )
    if sandbox == "workspace-write" and changed_write_paths:
        errors.append(
            "declared write_set intersects existing worktree changes: "
            + ", ".join(changed_write_paths)
        )
    return base_matches, changed_write_paths, errors


def resolve_packet_manifest(
    root: Path,
    packet: dict[str, Any],
    *,
    verify_workspace: bool,
) -> tuple[dict[str, Any], list[str]]:
    context = packet["context"]
    scope = packet.get("scope", {})
    scope_references: list[str] = []
    unresolved_read_references: list[str] = []
    for reference in scope.get("read_set", []):
        try:
            expand_reference(root, reference)
        except ValueError:
            unresolved_read_references.append(reference)
        else:
            scope_references.append(reference)
    for reference in scope.get("write_set", []):
        try:
            expand_reference(root, reference)
        except ValueError as exc:
            message = str(exc)
            if "matched no files" not in message and "file is missing" not in message:
                raise
        else:
            scope_references.append(reference)
    if verify_workspace and unresolved_read_references:
        raise ValueError(
            "declared read_set references are unavailable: "
            + ", ".join(sorted(unresolved_read_references))
        )
    expected = build_manifest(
        root,
        context["pack_ids"],
        list(dict.fromkeys(context["additional_files"] + scope_references)),
        False,
        packet["base_revision"],
    )
    declared = context.get("resolved_manifest")
    if declared:
        manifest_path = contained_path(root, root / declared)
        manifest = load_object(manifest_path)
        schema = load_object(root / "docs/development/ai/schemas/context-manifest.v1.schema.json")
        findings = validate_context_manifest(
            manifest,
            manifest_path.relative_to(root).as_posix(),
            schema,
            root=root,
            verify_workspace=verify_workspace,
        )
        errors = [finding for finding in findings if finding.level == "error"]
        if errors:
            details = "; ".join(f"{finding.code}: {finding.message}" for finding in errors)
            raise ValueError(f"invalid resolved context manifest: {details}")
        if manifest.get("base_revision") != packet.get("base_revision"):
            raise ValueError("context manifest base_revision differs from task packet")
        if manifest.get("manifest_sha256") != expected.get("manifest_sha256"):
            raise ValueError(
                "resolved context manifest does not exactly bind packet packs, additional files, "
                "and existing read/write inputs"
            )
        return manifest, sorted(unresolved_read_references)
    return expected, sorted(unresolved_read_references)


def capability_phases(root: Path) -> dict[str, str]:
    registry = load_object(root / CAPABILITY_REGISTRY_PATH)
    return {
        item["id"]: item["phase"]
        for item in registry.get("capabilities", [])
        if isinstance(item, dict)
        and isinstance(item.get("id"), str)
        and isinstance(item.get("phase"), str)
    }


def build_command(
    matrix: dict[str, Any],
    adapter_key: str,
    role: dict[str, Any],
    tier: str,
    root: Path,
    sandbox: str,
    persist: bool,
    json_events: bool,
) -> list[str]:
    runtime = matrix["runtime_policy"]
    adapter = runtime["adapters"][adapter_key][tier]
    effort = adapter["reasoning_effort"]
    command = [
        "codex",
        "exec",
        "--ignore-user-config",
        "--strict-config",
        "--model",
        role["model"],
        "--config",
        f'model_reasoning_effort="{effort}"',
        "--sandbox",
        sandbox,
        "--cd",
        str(root),
    ]
    if not persist:
        command.append("--ephemeral")
    if json_events:
        command.append("--json")
    command.append("-")
    return command


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--role", help="Role name. Optional when a JSON task packet declares role.")
    parser.add_argument("--packet", type=Path)
    parser.add_argument("--prompt")
    parser.add_argument("--root", type=Path, default=repository_root())
    parser.add_argument("--sandbox", choices=("read-only", "workspace-write"), default="read-only")
    parser.add_argument("--execute", action="store_true", help="Run guarded Codex. The default only prints a manifest.")
    parser.add_argument(
        "--allow-codex",
        action="store_true",
        help="Explicitly authorize Codex usage after provider routing selected an Ultra/final task.",
    )
    parser.add_argument(
        "--allow-untyped-execution",
        action="store_true",
        help="Explicitly permit execution without a validated JSON task packet.",
    )
    parser.add_argument(
        "--allow-unenforced-write-set",
        action="store_true",
        help="Acknowledge that Codex workspace-write confines the repository, not the packet write-set.",
    )
    parser.add_argument("--persist", action="store_true", help="Persist the Codex session instead of using --ephemeral.")
    parser.add_argument("--json-events", action="store_true")
    args = parser.parse_args()

    try:
        root = args.root.resolve()
        matrix = load_matrix(root)
        prompt, packet_path = task_text(root, args.packet, args.prompt)
        packet = validated_json_packet(root, packet_path, matrix)
        if args.execute and packet is not None and args.prompt:
            raise ValueError("typed packet execution cannot add --prompt outside the validated contract")
        packet_role = packet.get("role") if packet is not None else None
        if args.role is not None and packet_role is not None and args.role != packet_role:
            raise ValueError(
                f"--role {args.role!r} conflicts with JSON task packet role {packet_role!r}"
            )
        role_name = args.role or packet_role
        if role_name is None:
            raise ValueError("provide --role unless a JSON task packet declares it")
        role = select_role(matrix, role_name)
        tier = packet["risk"]["selected_tier"] if packet is not None else role["reasoning_tier"]
        if args.execute and not args.allow_codex:
            raise ValueError(
                "Codex execution is guarded by the Low-Codex policy; pass --allow-codex only "
                "after provider routing selected an Ultra/final task"
            )
        if args.execute and packet is not None and packet.get("status") not in {"ready", "in_progress"}:
            raise ValueError("only ready or in_progress JSON task packets can execute")
        if args.execute and packet is None and not args.allow_untyped_execution:
            raise ValueError("execution requires a validated JSON task packet or --allow-untyped-execution")
        if args.sandbox == "workspace-write" and packet is None:
            raise ValueError("workspace-write requires a validated JSON task packet")
        if args.sandbox == "workspace-write" and not args.allow_unenforced_write_set:
            raise ValueError(
                "workspace-write is repository-scoped, not packet-write-set-scoped; "
                "pass --allow-unenforced-write-set to acknowledge this limitation"
            )
        head_revision = current_head(root)
        task_base_matches_head = packet is None or packet.get("base_revision") == head_revision
        changed_paths = dirty_paths(root) if packet is not None else set()
        preflight_errors: list[str] = []
        changed_write_paths: list[str] = []
        if packet is not None:
            task_base_matches_head, changed_write_paths, preflight_errors = preflight_task_execution(
                packet,
                head_revision,
                changed_paths,
                args.sandbox,
            )
        if args.execute and preflight_errors:
            raise ValueError("; ".join(preflight_errors))
        context_result = (
            resolve_packet_manifest(root, packet, verify_workspace=args.execute)
            if packet is not None
            else None
        )
        context_manifest = context_result[0] if context_result is not None else None
        unresolved_context_references = context_result[1] if context_result is not None else []
        if context_manifest is not None:
            prompt += (
                "\n\n---\n\nResolved context manifest (read only these files first; "
                "request escalation before expanding context):\n\n"
                + json.dumps(context_manifest, indent=2, sort_keys=True)
            )
        capability_payload = probe(root)
        phases = capability_phases(root)
        required_capabilities = sorted(
            {
                item["requires_capability"]
                for item in packet.get("validation", [])
                if isinstance(item, dict)
                and item.get("required") is True
                and isinstance(item.get("requires_capability"), str)
            }
        ) if packet is not None else []
        preflight_capabilities = [
            item for item in required_capabilities if phases.get(item) == "preflight"
        ]
        post_validation_capabilities = [
            item for item in required_capabilities if phases.get(item) == "post_validation"
        ]
        unavailable_capabilities = [
            item for item in preflight_capabilities if capability_payload["capabilities"].get(item) is not True
        ]
        if args.execute and unavailable_capabilities:
            raise ValueError(
                "required validation capabilities are unavailable: "
                + ", ".join(unavailable_capabilities)
            )
        runtime_adapter_key, runtime_adapter = installed_codex_cli_adapter(matrix, root)
        command = build_command(
            matrix,
            runtime_adapter_key,
            role,
            tier,
            root,
            args.sandbox,
            args.persist,
            args.json_events,
        )
    except (FileNotFoundError, json.JSONDecodeError, KeyError, TypeError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    runtime = matrix["runtime_policy"]
    manifest = {
        "schema": "codefire.ai_run_manifest.v1",
        "role": role["role"],
        "model": role["model"],
        "reasoning_tier": tier,
        "runtime_adapter": runtime_adapter_key,
        "effective_effort": runtime_adapter[tier]["reasoning_effort"],
        "delegation": runtime_adapter[tier]["delegation"],
        "sandbox": args.sandbox,
        "cwd": str(root),
        "prompt_sha256": hashlib.sha256(prompt.encode("utf-8")).hexdigest(),
        "prompt_bytes": len(prompt.encode("utf-8")),
        "command": shlex.join(command),
        "execute": args.execute,
        "codex_usage_explicitly_allowed": args.allow_codex,
        "strict_config": True,
        "ignore_user_config": True,
        "head_revision": head_revision,
        "write_set_enforcement": (
            "repository_sandbox_only_acknowledged"
            if args.sandbox == "workspace-write"
            else "read_only"
        ),
    }
    if packet is not None:
        manifest.update(
            {
                "task_packet_id": packet["packet_id"],
                "task_packet_sha256": packet_digest(packet),
                "task_base_revision": packet["base_revision"],
                "task_base_matches_head": task_base_matches_head,
                "risk_score": packet["risk"]["score"],
                "declared_write_set": packet["scope"]["write_set"],
                "dirty_write_set_paths": changed_write_paths,
                "context_manifest_id": context_manifest["manifest_id"],
                "context_manifest_sha256": context_manifest["manifest_sha256"],
                "unresolved_context_read_references": unresolved_context_references,
                "required_capabilities": required_capabilities,
                "preflight_capabilities": preflight_capabilities,
                "post_validation_capabilities": post_validation_capabilities,
                "unavailable_capabilities": unavailable_capabilities,
                "execution_ready_in_environment": (
                    task_base_matches_head
                    and not unavailable_capabilities
                    and not unresolved_context_references
                    and (args.sandbox != "workspace-write" or not changed_write_paths)
                ),
            }
        )
    print(json.dumps(manifest, indent=2, sort_keys=True))
    if not args.execute:
        return 0

    result = subprocess.run(command, cwd=root, input=prompt, text=True, check=False)
    return result.returncode


if __name__ == "__main__":
    sys.exit(main())
