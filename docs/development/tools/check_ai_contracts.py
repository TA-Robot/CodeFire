#!/usr/bin/env python3
"""Validate AI packet contracts, packet instances, and routing evals."""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import asdict
from pathlib import Path

from ai_contracts import (
    canonical_git_patch,
    Finding,
    load_object,
    validate_context_catalog,
    validate_context_manifest,
    validate_historical_eval_corpus,
    validate_review_against_task,
    validate_review_packet,
    validate_routing_corpus,
    validate_task_packet,
    validate_json_schema,
)
from probe_validation_capabilities import CAPABILITY_IDS


SCHEMAS = {
    "docs/development/ai/schemas/task-packet.v1.schema.json": "https://codefire.dev/schemas/ai/task-packet.v1.schema.json",
    "docs/development/ai/schemas/context-manifest.v1.schema.json": "https://codefire.dev/schemas/ai/context-manifest.v1.schema.json",
    "docs/development/ai/schemas/review-packet.v1.schema.json": "https://codefire.dev/schemas/ai/review-packet.v1.schema.json",
    "docs/development/ai/schemas/development-flow-record.v1.schema.json": "https://codefire.dev/schemas/ai/development-flow-record.v1.schema.json",
}


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def read_object(root: Path, relative: str, findings: list[Finding]) -> dict | None:
    try:
        return load_object(root / relative)
    except FileNotFoundError:
        findings.append(Finding("error", "missing-file", relative, "file is missing"))
    except (json.JSONDecodeError, ValueError) as exc:
        findings.append(Finding("error", "invalid-json", relative, str(exc)))
    return None


def run(root: Path, extra_tasks: list[Path], extra_reviews: list[Path]) -> list[Finding]:
    findings: list[Finding] = []
    loaded_schemas: dict[str, dict] = {}
    for relative, schema_id in SCHEMAS.items():
        data = read_object(root, relative, findings)
        if data is not None and data.get("$id") != schema_id:
            findings.append(Finding("error", "wrong-schema-id", relative, f"expected {schema_id}"))
        if data is not None:
            loaded_schemas[relative] = data

    matrix_path = "docs/development/ai/role-routing-matrix.json"
    catalog_path = "docs/development/ai/context-packs.json"
    corpus_path = "docs/development/ai/evals/routing-cases.v1.json"
    historical_corpus_path = "docs/development/ai/evals/historical-cases.v1.json"
    capability_path = "docs/development/ai/capability-registry.v1.json"
    test_map_path = "docs/development/ai/focused-test-map.v1.json"
    matrix = read_object(root, matrix_path, findings)
    catalog = read_object(root, catalog_path, findings)
    corpus = read_object(root, corpus_path, findings)
    historical_corpus = read_object(root, historical_corpus_path, findings)
    capability_registry = read_object(root, capability_path, findings)
    focused_test_map = read_object(root, test_map_path, findings)
    capability_ids: set[str] = set()
    post_validation_capabilities: set[str] = set()
    if capability_registry is not None:
        if capability_registry.get("schema") != "codefire.ai_capability_registry.v1":
            findings.append(Finding("error", "wrong-schema", capability_path, "unexpected capability schema"))
        entries = capability_registry.get("capabilities")
        if not isinstance(entries, list):
            findings.append(Finding("error", "bad-capability-registry", capability_path, "capabilities must be a list"))
        else:
            for index, entry in enumerate(entries):
                if not isinstance(entry, dict) or not isinstance(entry.get("id"), str):
                    findings.append(Finding("error", "bad-capability", f"{capability_path}[{index}]", repr(entry)))
                    continue
                capability_id = entry["id"]
                if capability_id in capability_ids:
                    findings.append(Finding("error", "duplicate-capability", capability_path, capability_id))
                capability_ids.add(capability_id)
                kind = entry.get("kind")
                phase = entry.get("phase")
                if kind not in {"executable", "compound", "derived", "evidence"}:
                    findings.append(Finding("error", "bad-capability-kind", capability_path, repr(kind)))
                if phase not in {"preflight", "post_validation"}:
                    findings.append(Finding("error", "bad-capability-phase", capability_path, repr(phase)))
                if kind == "evidence" and phase != "post_validation":
                    findings.append(Finding("error", "evidence-used-as-preflight", capability_path, capability_id))
                if kind != "evidence" and phase != "preflight":
                    findings.append(Finding("error", "capability-phase-mismatch", capability_path, capability_id))
                if phase == "post_validation":
                    post_validation_capabilities.add(capability_id)
            if capability_ids != CAPABILITY_IDS:
                findings.append(
                    Finding(
                        "error",
                        "capability-probe-drift",
                        capability_path,
                        f"registry={sorted(capability_ids)}, probe={sorted(CAPABILITY_IDS)}",
                    )
                )
    if focused_test_map is not None:
        if focused_test_map.get("schema") != "codefire.ai_focused_test_map.v1":
            findings.append(Finding("error", "wrong-schema", test_map_path, "unexpected focused-test schema"))
        for rule_index, rule in enumerate(focused_test_map.get("rules", [])):
            for command_index, command in enumerate(rule.get("commands", []) if isinstance(rule, dict) else []):
                capability = command.get("capability") if isinstance(command, dict) else None
                if capability not in capability_ids:
                    findings.append(
                        Finding(
                            "error",
                            "unknown-capability",
                            f"{test_map_path}:rules[{rule_index}].commands[{command_index}]",
                            repr(capability),
                        )
                    )
    context_ids: set[str] = set()
    if catalog is not None:
        catalog_findings, context_ids = validate_context_catalog(catalog, catalog_path)
        findings.extend(catalog_findings)
    if matrix is None:
        return findings
    if corpus is not None:
        findings.extend(validate_routing_corpus(corpus, matrix, corpus_path))
    if historical_corpus is not None:
        findings.extend(
            validate_historical_eval_corpus(
                historical_corpus,
                matrix,
                root,
                historical_corpus_path,
            )
        )

    flow_schema = loaded_schemas.get(
        "docs/development/ai/schemas/development-flow-record.v1.schema.json"
    )
    flow_paths = sorted((root / "docs/development/ai/flow-metrics").glob("*.json"))
    if not flow_paths:
        findings.append(
            Finding(
                "error",
                "missing-development-flow-records",
                "docs/development/ai/flow-metrics",
                "at least one active or final KGI/KPI record is required",
            )
        )
    flow_record_ids: set[str] = set()
    for path in flow_paths:
        relative = path.relative_to(root).as_posix()
        data = read_object(root, relative, findings)
        if data is None or flow_schema is None:
            continue
        flow_findings = validate_json_schema(data, flow_schema, relative)
        findings.extend(flow_findings)
        record_id = data.get("record_id")
        if isinstance(record_id, str):
            if record_id in flow_record_ids:
                findings.append(
                    Finding("error", "duplicate-development-flow-record-id", relative, record_id)
                )
            else:
                flow_record_ids.add(record_id)

    task_paths = sorted((root / "docs/development/ai/task-packets/json").glob("*.json"))
    task_paths.extend(path.resolve() for path in extra_tasks)
    if not task_paths:
        findings.append(Finding("error", "missing-json-task-packets", "docs/development/ai/task-packets/json", "no packet instances"))
    tasks_by_id: dict[str, dict] = {}
    for path in task_paths:
        try:
            relative = path.relative_to(root).as_posix()
        except ValueError:
            findings.append(Finding("error", "packet-outside-repository", str(path), "task packet must stay inside repository"))
            continue
        data = read_object(root, relative, findings)
        if data is not None:
            task_findings = validate_task_packet(
                data,
                matrix,
                context_ids,
                relative,
                loaded_schemas.get("docs/development/ai/schemas/task-packet.v1.schema.json"),
            )
            findings.extend(task_findings)
            packet_id = data.get("packet_id")
            if isinstance(packet_id, str):
                if packet_id in tasks_by_id:
                    findings.append(Finding("error", "duplicate-task-packet-id", relative, packet_id))
                elif not any(finding.level == "error" for finding in task_findings):
                    tasks_by_id[packet_id] = data
            for validation_index, validation in enumerate(data.get("validation", [])):
                capability = validation.get("requires_capability") if isinstance(validation, dict) else None
                if capability is not None and capability not in capability_ids:
                    findings.append(
                        Finding(
                            "error",
                            "unknown-capability",
                            f"{relative}.validation[{validation_index}]",
                            repr(capability),
                        )
                    )
        markdown_view = root / "docs/development/ai/task-packets" / f"{path.stem}.md"
        if path.parent == root / "docs/development/ai/task-packets/json" and not markdown_view.is_file():
            findings.append(Finding("warning", "missing-markdown-task-view", relative, markdown_view.relative_to(root).as_posix()))

    review_paths = sorted((root / "docs/development/ai/review-packets/json").glob("*.json"))
    review_paths.extend(path.resolve() for path in extra_reviews)
    for path in review_paths:
        try:
            relative = path.relative_to(root).as_posix()
        except ValueError:
            findings.append(Finding("error", "review-outside-repository", str(path), "review packet must stay inside repository"))
            continue
        data = read_object(root, relative, findings)
        if data is not None:
            review_findings = validate_review_packet(
                data,
                relative,
                loaded_schemas.get("docs/development/ai/schemas/review-packet.v1.schema.json"),
            )
            findings.extend(review_findings)
            task_id = data.get("task_packet_id")
            task = tasks_by_id.get(task_id) if isinstance(task_id, str) else None
            if task is None:
                findings.append(Finding("error", "review-task-not-found", relative, repr(task_id)))
            elif not any(finding.level == "error" for finding in review_findings):
                patch_bytes = None
                actual_files = None
                resolved_base = None
                resolved_head = None
                if data.get("decision") == "approve":
                    try:
                        patch_bytes, actual_files, resolved_base, resolved_head = canonical_git_patch(
                            root,
                            str(data.get("base_revision")),
                            str(data.get("head_revision")),
                        )
                    except ValueError as exc:
                        findings.append(
                            Finding("error", "approval-canonical-patch-unavailable", relative, str(exc))
                        )
                findings.extend(
                    validate_review_against_task(
                        data,
                        task,
                        relative,
                        patch_bytes=patch_bytes,
                        actual_files=actual_files,
                        resolved_base_revision=resolved_base,
                        resolved_head_revision=resolved_head,
                        post_validation_capabilities=post_validation_capabilities,
                        root=root,
                    )
                )
    manifest_schema = loaded_schemas.get(
        "docs/development/ai/schemas/context-manifest.v1.schema.json"
    )
    for path in sorted((root / "docs/development/ai/context-manifests").glob("*.json")):
        relative = path.relative_to(root).as_posix()
        data = read_object(root, relative, findings)
        if data is not None and manifest_schema is not None:
            findings.extend(
                validate_context_manifest(
                    data,
                    relative,
                    manifest_schema,
                    root=root,
                    verify_workspace=True,
                )
            )
    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=repository_root())
    parser.add_argument("--task", type=Path, action="append", default=[])
    parser.add_argument("--review", type=Path, action="append", default=[])
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    findings = run(root, args.task, args.review)
    errors = [finding for finding in findings if finding.level == "error"]
    payload = {"ok": not errors, "findings": [asdict(finding) for finding in findings]}
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    elif errors:
        print(f"AI contract check failed: {len(errors)} error(s)")
        for finding in findings:
            print(f"- {finding.level}: {finding.code}: {finding.path}: {finding.message}")
    else:
        print("AI contract check passed.")
        for finding in findings:
            if finding.level == "warning":
                print(f"- warning: {finding.code}: {finding.path}: {finding.message}")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
