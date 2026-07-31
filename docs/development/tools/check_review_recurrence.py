#!/usr/bin/env python3
"""Gate a successor review without using a finding-count target.

The record keeps report observations, effect episodes, generation
opportunities, repair-created surface, validation terminality, and routing
separate.  It is a manager-side integration gate: a structurally valid
REQUEST_CHANGES record still returns BLOCK for integration.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
from dataclasses import asdict
from pathlib import Path
from typing import Any

from ai_contracts import (
    Finding,
    canonical_git_patch,
    load_object,
    repository_path_valid,
    validate_json_schema,
)


SCHEMA_NAME = "codefire.review_recurrence_control.v1"
RESULT_SCHEMA = "codefire.review_recurrence_gate_result.v1"
SCHEMA_PATH = Path(
    "docs/development/ai/schemas/review-recurrence-control.v1.schema.json"
)
EXPANSIVE_SURFACE_KINDS = {
    "public_api",
    "state_machine",
    "classifier",
    "persisted_shape",
}
CONTROL_PATH_PREFIXES = (
    "docs/development/ai/task-packets/",
    "docs/development/ai/schemas/",
    "docs/development/tools/",
)
STOPPING_CONDITIONS = (
    "prior_obligations_dispositioned",
    "declared_surfaces_checked",
    "repair_delta_checked",
    "production_routes_checked",
    "negative_controls_checked",
    "unavailable_evidence_explicit",
)


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def add(
    findings: list[Finding],
    code: str,
    location: str,
    message: str,
) -> None:
    findings.append(Finding("error", code, location, message))


def nonempty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def unique_field(
    rows: Any,
    field: str,
    location: str,
    findings: list[Finding],
) -> set[str]:
    if not isinstance(rows, list):
        return set()
    seen: set[str] = set()
    for index, row in enumerate(rows):
        if not isinstance(row, dict):
            continue
        value = row.get(field)
        if not isinstance(value, str):
            continue
        if value in seen:
            add(
                findings,
                f"duplicate-{field.replace('_', '-')}",
                f"{location}[{index}].{field}",
                value,
            )
        seen.add(value)
    return seen


def require_route(
    row: dict[str, Any],
    location: str,
    findings: list[Finding],
) -> None:
    for field in ("owner", "release_condition", "route_authority"):
        if not nonempty(row.get(field)):
            add(
                findings,
                "incomplete-open-route",
                f"{location}.{field}",
                "routed work needs an owner, release condition, and authority",
            )
    if row.get("blocking") is True:
        add(
            findings,
            "blocking-finding-routed",
            location,
            "routing never closes or de-blocks a blocking finding",
        )


def validate_coverage(
    coverage: Any,
    *,
    accepting: bool,
    findings: list[Finding],
) -> None:
    if not isinstance(coverage, dict):
        return
    stopping = coverage.get("stopping_rule")
    if not isinstance(stopping, dict):
        return
    if accepting:
        incomplete = [
            field for field in STOPPING_CONDITIONS if stopping.get(field) is not True
        ]
        if incomplete:
            add(
                findings,
                "accept-before-coverage-saturation",
                "record.coverage.stopping_rule",
                f"incomplete conditions: {incomplete}",
            )


def validate_prior_findings(
    rows: Any,
    *,
    generation: int | None,
    findings: list[Finding],
) -> set[str]:
    location = "record.prior_findings"
    ids = unique_field(rows, "id", location, findings)
    unique_field(rows, "observation_id", location, findings)
    if not isinstance(rows, list):
        return ids
    for index, row in enumerate(rows):
        if not isinstance(row, dict):
            continue
        row_location = f"{location}[{index}]"
        origin = row.get("origin_generation")
        if (
            isinstance(origin, int)
            and isinstance(generation, int)
            and origin >= generation
        ):
            add(
                findings,
                "non-prior-origin-generation",
                f"{row_location}.origin_generation",
                f"origin {origin} must precede generation {generation}",
            )
        disposition = row.get("disposition")
        before = row.get("before") if isinstance(row.get("before"), dict) else {}
        after = row.get("after") if isinstance(row.get("after"), dict) else {}
        oracle = row.get("oracle") if isinstance(row.get("oracle"), dict) else {}
        negative = (
            oracle.get("negative_control")
            if isinstance(oracle.get("negative_control"), dict)
            else {}
        )
        if disposition == "closed":
            if before.get("status") not in {"reproduced", "not_applicable"}:
                add(
                    findings,
                    "closure-without-before-proof",
                    f"{row_location}.before.status",
                    "closure needs a reproduced pre-repair observation or a justified non-applicable state",
                )
            if after.get("status") != "verified":
                add(
                    findings,
                    "closure-without-after-proof",
                    f"{row_location}.after.status",
                    "closed finding must be verified on the candidate",
                )
            for phase, observation in (("before", before), ("after", after)):
                if len(observation.get("evidence_refs", [])) == 0:
                    add(
                        findings,
                        "closure-observation-without-evidence",
                        f"{row_location}.{phase}.evidence_refs",
                        "closed finding observations need exact evidence",
                    )
            if negative.get("status") not in {"killed", "not_applicable"}:
                add(
                    findings,
                    "closure-without-negative-control",
                    f"{row_location}.oracle.negative_control.status",
                    "negative control must be killed or explicitly inapplicable",
                )
            if (
                negative.get("status") == "not_applicable"
                and len(negative.get("evidence_refs", [])) == 0
            ):
                add(
                    findings,
                    "inapplicable-control-without-evidence",
                    f"{row_location}.oracle.negative_control",
                    "claim/policy controls still need exact evidence",
                )
            for field in ("owner", "release_condition", "route_authority"):
                if row.get(field) is not None:
                    add(
                        findings,
                        "closed-finding-has-open-route",
                        f"{row_location}.{field}",
                        "closed rows do not retain routing fields",
                    )
        elif disposition == "open":
            if after.get("status") not in {"open", "unavailable"}:
                add(
                    findings,
                    "open-finding-with-closed-observation",
                    f"{row_location}.after.status",
                    repr(after.get("status")),
                )
            if not nonempty(row.get("owner")) or not nonempty(
                row.get("release_condition")
            ):
                add(
                    findings,
                    "open-finding-without-owner",
                    row_location,
                    "open finding needs an owner and release condition",
                )
            if row.get("route_authority") is not None:
                add(
                    findings,
                    "open-finding-has-route-authority",
                    f"{row_location}.route_authority",
                    "unrouted open work must not claim route authority",
                )
        elif disposition == "routed_open":
            require_route(row, row_location, findings)
            if after.get("status") not in {"open", "unavailable"}:
                add(
                    findings,
                    "routed-finding-with-closed-observation",
                    f"{row_location}.after.status",
                    repr(after.get("status")),
                )
        elif disposition == "unavailable":
            if not nonempty(row.get("owner")) or not nonempty(
                row.get("release_condition")
            ):
                add(
                    findings,
                    "unavailable-without-next-owner",
                    row_location,
                    "unavailable evidence needs an owner and release condition",
                )
            if row.get("route_authority") is not None:
                add(
                    findings,
                    "unavailable-finding-has-route-authority",
                    f"{row_location}.route_authority",
                    "unavailable is not a manager-authorized route",
                )
    return ids


def validate_new_findings(
    rows: Any,
    findings: list[Finding],
) -> tuple[set[str], int]:
    location = "record.new_findings"
    ids = unique_field(rows, "id", location, findings)
    unique_field(rows, "observation_id", location, findings)
    candidate_created = 0
    if not isinstance(rows, list):
        return ids, candidate_created
    for index, row in enumerate(rows):
        if not isinstance(row, dict):
            continue
        row_location = f"{location}[{index}]"
        if row.get("attribution") == "created_by_candidate":
            candidate_created += 1
            if "repair_introduced_regression" not in row.get(
                "causal_classes",
                [],
            ):
                add(
                    findings,
                    "candidate-created-without-repair-cause",
                    f"{row_location}.causal_classes",
                    "candidate-created findings must name the repair-introduced cause",
                )
        if (
            row.get("attribution") == "pre_existing"
            and "repair_introduced_regression" in row.get("causal_classes", [])
        ):
            add(
                findings,
                "pre-existing-with-repair-cause",
                f"{row_location}.causal_classes",
                "a pre-existing effect cannot be attributed to this repair",
            )
        if "bound_finding_not_closed" in row.get("causal_classes", []):
            add(
                findings,
                "prior-obligation-misclassified-as-new",
                f"{row_location}.causal_classes",
                "an unclosed bound obligation belongs in prior_findings",
            )
        disposition = row.get("disposition")
        if disposition == "open":
            if not nonempty(row.get("owner")) or not nonempty(
                row.get("release_condition")
            ):
                add(
                    findings,
                    "new-open-finding-without-owner",
                    row_location,
                    "open finding needs an owner and release condition",
                )
            if row.get("route_authority") is not None:
                add(
                    findings,
                    "new-open-finding-has-route-authority",
                    f"{row_location}.route_authority",
                    "unrouted open work must not claim route authority",
                )
        elif disposition == "routed_open":
            require_route(row, row_location, findings)
        elif disposition == "unavailable":
            if not nonempty(row.get("owner")) or not nonempty(
                row.get("release_condition")
            ):
                add(
                    findings,
                    "new-unavailable-without-next-owner",
                    row_location,
                    "unavailable evidence needs an owner and release condition",
                )
            if row.get("route_authority") is not None:
                add(
                    findings,
                    "new-unavailable-has-route-authority",
                    f"{row_location}.route_authority",
                    "unavailable is not a manager-authorized route",
                )
    return ids, candidate_created


def validate_repair_delta(
    delta: Any,
    finding_ids: set[str],
    findings: list[Finding],
) -> int:
    if not isinstance(delta, dict):
        return 0
    location = "record.repair_delta"
    surfaces = delta.get("new_surfaces")
    unique_field(surfaces, "symbol", f"{location}.new_surfaces", findings)
    expansive = False
    unplanned = 0
    if isinstance(surfaces, list):
        for index, surface in enumerate(surfaces):
            if not isinstance(surface, dict):
                continue
            surface_location = f"{location}.new_surfaces[{index}]"
            required_by = surface.get("required_by")
            if required_by not in finding_ids:
                add(
                    findings,
                    "new-surface-without-finding-owner",
                    f"{surface_location}.required_by",
                    repr(required_by),
                )
            if surface.get("kind") in EXPANSIVE_SURFACE_KINDS:
                expansive = True
            if surface.get("planned") is False:
                unplanned += 1
                if surface.get("plan_ref") is not None:
                    add(
                        findings,
                        "unplanned-surface-has-plan-reference",
                        f"{surface_location}.plan_ref",
                        "unplanned repair surface must not claim a pre-repair plan reference",
                    )
            elif not nonempty(surface.get("plan_ref")):
                add(
                    findings,
                    "planned-surface-without-plan-reference",
                    f"{surface_location}.plan_ref",
                    "planned repair surface needs an exact pre-repair reference",
                )
    expansion = delta.get("scope_expansion")
    authority = delta.get("scope_authority")
    if expansive and expansion not in {"manager_approved", "blocked"}:
        add(
            findings,
            "expansive-surface-without-manager-approval",
            f"{location}.scope_expansion",
            "public API/state/schema/classifier additions need explicit approval",
        )
    if expansion == "manager_approved" and not nonempty(authority):
        add(
            findings,
            "scope-expansion-without-authority",
            f"{location}.scope_authority",
            "manager-approved expansion needs an exact authority",
        )
    if expansion != "manager_approved" and authority is not None:
        add(
            findings,
            "unexpected-scope-authority",
            f"{location}.scope_authority",
            "authority is only valid for manager-approved expansion",
        )
    if unplanned and expansion not in {"manager_approved", "blocked"}:
        add(
            findings,
            "unplanned-surface-without-scope-decision",
            f"{location}.scope_expansion",
            "unplanned repair surface needs manager approval or a blocked decision",
        )
    return unplanned


def validate_validations(rows: Any, findings: list[Finding]) -> None:
    location = "record.validations"
    unique_field(rows, "id", location, findings)
    if not isinstance(rows, list):
        return
    for index, row in enumerate(rows):
        if not isinstance(row, dict):
            continue
        row_location = f"{location}[{index}]"
        if row.get("status") in {"passed", "failed"}:
            if row.get("terminal") is not True:
                add(
                    findings,
                    "completed-validation-not-terminal",
                    f"{row_location}.terminal",
                    "background progress or truncated output is not a completed validation",
                )
            if not nonempty(row.get("log_ref")):
                add(
                    findings,
                    "completed-validation-without-log-reference",
                    f"{row_location}.log_ref",
                    "completed validation needs an exact log reference",
                )
            if not nonempty(row.get("log_sha256")):
                add(
                    findings,
                    "completed-validation-without-log-digest",
                    f"{row_location}.log_sha256",
                    "completed validation needs a complete log digest",
                )
        if row.get("status") == "passed":
            if row.get("exit_code") != 0:
                add(
                    findings,
                    "passed-validation-nonzero-exit",
                    f"{row_location}.exit_code",
                    repr(row.get("exit_code")),
                )
        if row.get("status") == "failed" and row.get("exit_code") in {None, 0}:
            add(
                findings,
                "failed-validation-without-nonzero-exit",
                f"{row_location}.exit_code",
                repr(row.get("exit_code")),
            )


def validate_outcomes(
    outcomes: Any,
    prior_rows: Any,
    new_rows: Any,
    candidate_created: int,
    unplanned_surfaces: int,
    findings: list[Finding],
) -> None:
    if (
        not isinstance(outcomes, dict)
        or not isinstance(prior_rows, list)
        or not isinstance(new_rows, list)
    ):
        return
    counts = {
        "prior_findings_total": len(prior_rows),
        "prior_findings_closed": sum(
            isinstance(row, dict) and row.get("disposition") == "closed"
            for row in prior_rows
        ),
        "prior_findings_open": sum(
            isinstance(row, dict) and row.get("disposition") == "open"
            for row in prior_rows
        ),
        "prior_findings_routed": sum(
            isinstance(row, dict) and row.get("disposition") == "routed_open"
            for row in prior_rows
        ),
        "prior_findings_unavailable": sum(
            isinstance(row, dict) and row.get("disposition") == "unavailable"
            for row in prior_rows
        ),
        "candidate_created_findings": candidate_created,
        "new_findings_total": len(new_rows),
        "new_findings_open": sum(
            isinstance(row, dict) and row.get("disposition") == "open"
            for row in new_rows
        ),
        "new_findings_routed": sum(
            isinstance(row, dict) and row.get("disposition") == "routed_open"
            for row in new_rows
        ),
        "new_findings_unavailable": sum(
            isinstance(row, dict) and row.get("disposition") == "unavailable"
            for row in new_rows
        ),
        "prior_observations_total": len(
            {
                row.get("observation_id")
                for row in prior_rows
                if isinstance(row, dict) and isinstance(row.get("observation_id"), str)
            }
        ),
        "prior_effect_episodes_total": len(
            {
                row.get("effect_episode_id")
                for row in prior_rows
                if isinstance(row, dict)
                and isinstance(row.get("effect_episode_id"), str)
            }
        ),
        "new_observations_total": len(
            {
                row.get("observation_id")
                for row in new_rows
                if isinstance(row, dict) and isinstance(row.get("observation_id"), str)
            }
        ),
        "new_effect_episodes_total": len(
            {
                row.get("effect_episode_id")
                for row in new_rows
                if isinstance(row, dict)
                and isinstance(row.get("effect_episode_id"), str)
            }
        ),
        "unplanned_surfaces": unplanned_surfaces,
    }
    for field, expected in counts.items():
        if outcomes.get(field) != expected:
            add(
                findings,
                "outcome-count-mismatch",
                f"record.outcomes.{field}",
                f"declared {outcomes.get(field)!r}, computed {expected}",
            )
    unavailable = outcomes.get("unavailable_metrics")
    unavailable_set = set(unavailable) if isinstance(unavailable, list) else set()
    for field in ("repair_service_ms", "review_service_ms", "verdict_to_report_ms"):
        if outcomes.get(field) is None and field not in unavailable_set:
            add(
                findings,
                "null-metric-without-unavailable-marker",
                f"record.outcomes.{field}",
                field,
            )
        if outcomes.get(field) is not None and field in unavailable_set:
            add(
                findings,
                "available-metric-marked-unavailable",
                f"record.outcomes.unavailable_metrics",
                field,
            )


def git_output(root: Path, *args: str) -> bytes:
    completed = subprocess.run(
        ["git", *args],
        cwd=root,
        check=False,
        capture_output=True,
    )
    if completed.returncode != 0:
        raise ValueError(
            completed.stderr.decode("utf-8", errors="replace").strip()
            or f"git {' '.join(args)} failed"
        )
    return completed.stdout


def verify_git_subject(
    subject: Any,
    repair_delta: Any,
    root: Path,
    findings: list[Finding],
) -> None:
    if not isinstance(subject, dict):
        return
    location = "record.subject"
    try:
        patch, changed, base, candidate = canonical_git_patch(
            root,
            str(subject.get("base_revision")),
            str(subject.get("candidate_revision")),
        )
    except ValueError as exc:
        add(findings, "candidate-git-binding-failed", location, str(exc))
        return
    if subject.get("base_revision") != base:
        add(
            findings,
            "base-revision-not-canonical",
            f"{location}.base_revision",
            f"resolved {base}",
        )
    if subject.get("candidate_revision") != candidate:
        add(
            findings,
            "candidate-revision-not-canonical",
            f"{location}.candidate_revision",
            f"resolved {candidate}",
        )
    patch_sha = hashlib.sha256(patch).hexdigest()
    if subject.get("patch_sha256") != patch_sha:
        add(
            findings,
            "candidate-patch-digest-mismatch",
            f"{location}.patch_sha256",
            f"computed {patch_sha}",
        )
    declared_paths = set(subject.get("changed_paths", []))
    unsafe_candidate_paths = sorted(
        path for path in changed if not repository_path_valid(path)
    )
    if unsafe_candidate_paths:
        add(
            findings,
            "candidate-has-unsafe-path",
            f"{location}.changed_paths",
            repr(unsafe_candidate_paths),
        )
    if declared_paths != changed:
        add(
            findings,
            "candidate-changed-paths-mismatch",
            f"{location}.changed_paths",
            f"declared-only={sorted(declared_paths - changed)}, actual-only={sorted(changed - declared_paths)}",
        )
    try:
        tree = git_output(root, "rev-parse", f"{candidate}^{{tree}}").decode().strip()
        control = (
            git_output(
                root,
                "rev-parse",
                "--verify",
                f"{subject.get('review_control_revision')}^{{commit}}",
            )
            .decode()
            .strip()
        )
        repair_packet_revision = (
            git_output(
                root,
                "rev-parse",
                "--verify",
                f"{subject.get('repair_packet_revision')}^{{commit}}",
            )
            .decode()
            .strip()
        )
        report_commit = (
            git_output(
                root,
                "rev-parse",
                "--verify",
                f"{subject.get('report_commit')}^{{commit}}",
            )
            .decode()
            .strip()
        )
        control_patch, actual_control_paths, control_base, control_head = (
            canonical_git_patch(
                root,
                candidate,
                control,
            )
        )
        control_tree = (
            git_output(root, "rev-parse", f"{control}^{{tree}}").decode().strip()
        )
        control_patch_sha = hashlib.sha256(control_patch).hexdigest()
        declared_control_paths = set(subject.get("control_changed_paths", []))
        if control_base != candidate:
            raise ValueError("review control does not descend from the exact candidate")
        if subject.get("review_control_revision") != control_head:
            add(
                findings,
                "review-control-revision-not-canonical",
                f"{location}.review_control_revision",
                f"resolved {control_head}",
            )
        if subject.get("review_control_tree") != control_tree:
            add(
                findings,
                "review-control-tree-mismatch",
                f"{location}.review_control_tree",
                f"computed {control_tree}",
            )
        if subject.get("control_patch_sha256") != control_patch_sha:
            add(
                findings,
                "review-control-patch-digest-mismatch",
                f"{location}.control_patch_sha256",
                f"computed {control_patch_sha}",
            )
        if declared_control_paths != actual_control_paths:
            add(
                findings,
                "review-control-paths-mismatch",
                f"{location}.control_changed_paths",
                f"declared-only={sorted(declared_control_paths - actual_control_paths)}, actual-only={sorted(actual_control_paths - declared_control_paths)}",
            )
        unsafe_control_paths = sorted(
            path
            for path in actual_control_paths
            if not repository_path_valid(path)
            or not any(path.startswith(prefix) for prefix in CONTROL_PATH_PREFIXES)
        )
        if unsafe_control_paths:
            add(
                findings,
                "review-control-touches-non-control-path",
                f"{location}.control_changed_paths",
                repr(unsafe_control_paths),
            )
        report_parents = (
            git_output(root, "rev-list", "--parents", "-n", "1", report_commit)
            .decode()
            .strip()
            .split()
        )
        if len(report_parents) != 2 or report_parents[1] != control:
            raise ValueError(
                "report commit must be a single-parent direct child of review control"
            )
        report_path = subject.get("report_path")
        if not repository_path_valid(report_path):
            raise ValueError(f"unsafe report path: {report_path!r}")
        if not report_path.startswith("docs/development/ai/swarm-results/"):
            raise ValueError("report path is outside the review result namespace")
        if report_path in actual_control_paths:
            raise ValueError("report path was already changed by review control")
        report_bytes = git_output(root, "show", f"{report_commit}:{report_path}")
        repair_packet_path = subject.get("repair_packet_path")
        if not repository_path_valid(repair_packet_path):
            raise ValueError(f"unsafe repair packet path: {repair_packet_path!r}")
        if not repair_packet_path.startswith(
            "docs/development/ai/task-packets/"
        ):
            raise ValueError(
                "repair packet path is outside the task-packet namespace"
            )
        repair_packet_bytes = git_output(
            root,
            "show",
            f"{repair_packet_revision}:{repair_packet_path}",
        )
        review_packet_path = subject.get("review_packet_path")
        if not repository_path_valid(review_packet_path):
            raise ValueError(f"unsafe review packet path: {review_packet_path!r}")
        if not review_packet_path.startswith(
            "docs/development/ai/task-packets/"
        ):
            raise ValueError(
                "review packet path is outside the task-packet namespace"
            )
        review_packet_bytes = git_output(
            root,
            "show",
            f"{control}:{review_packet_path}",
        )
        report_changed = {
            value.decode("utf-8", errors="surrogateescape")
            for value in git_output(
                root,
                "diff",
                "--name-only",
                "-z",
                "--no-renames",
                control,
                report_commit,
                "--",
            ).split(b"\0")
            if value
        }
    except ValueError as exc:
        add(findings, "report-git-binding-failed", location, str(exc))
        return
    if subject.get("candidate_tree") != tree:
        add(
            findings,
            "candidate-tree-mismatch",
            f"{location}.candidate_tree",
            f"computed {tree}",
        )
    report_sha = hashlib.sha256(report_bytes).hexdigest()
    if subject.get("report_sha256") != report_sha:
        add(
            findings,
            "report-digest-mismatch",
            f"{location}.report_sha256",
            f"computed {report_sha}",
        )
    if subject.get("repair_packet_revision") != repair_packet_revision:
        add(
            findings,
            "repair-packet-revision-not-canonical",
            f"{location}.repair_packet_revision",
            f"resolved {repair_packet_revision}",
        )
    repair_packet_sha = hashlib.sha256(repair_packet_bytes).hexdigest()
    if subject.get("repair_packet_sha256") != repair_packet_sha:
        add(
            findings,
            "repair-packet-digest-mismatch",
            f"{location}.repair_packet_sha256",
            f"computed {repair_packet_sha}",
        )
    review_packet_sha = hashlib.sha256(review_packet_bytes).hexdigest()
    if subject.get("review_packet_sha256") != review_packet_sha:
        add(
            findings,
            "review-packet-digest-mismatch",
            f"{location}.review_packet_sha256",
            f"computed {review_packet_sha}",
        )
    repair_timing = subject.get("repair_packet_timing")
    repair_is_ancestor = subprocess.run(
        [
            "git",
            "merge-base",
            "--is-ancestor",
            repair_packet_revision,
            candidate,
        ],
        cwd=root,
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode == 0
    dispatch_ref = subject.get("repair_dispatch_event_ref")
    dispatch_sha = subject.get("repair_dispatch_event_sha256")
    if repair_timing == "ancestor_of_candidate":
        if not repair_is_ancestor:
            add(
                findings,
                "repair-packet-does-not-precede-candidate",
                f"{location}.repair_packet_revision",
                "declared ancestor timing is false",
            )
        if dispatch_ref is not None or dispatch_sha is not None:
            add(
                findings,
                "ancestor-timing-has-dispatch-attestation",
                f"{location}.repair_packet_timing",
                "ancestor timing does not use external dispatch evidence",
            )
    elif repair_timing == "dispatch_event_attested":
        if not nonempty(dispatch_ref) or not nonempty(dispatch_sha):
            add(
                findings,
                "repair-dispatch-attestation-incomplete",
                f"{location}.repair_packet_timing",
                "branch-external pre-repair packets need an event reference and digest",
            )
    if report_changed != {report_path}:
        add(
            findings,
            "report-commit-write-set-mismatch",
            f"{location}.report_commit",
            f"changed paths: {sorted(report_changed)}",
        )
    if isinstance(repair_delta, dict):
        try:
            entries = [
                entry
                for entry in git_output(
                    root,
                    "diff",
                    "--numstat",
                    "-z",
                    "--no-renames",
                    base,
                    candidate,
                    "--",
                ).split(b"\0")
                if entry
            ]
            inserted = 0
            deleted = 0
            binary_paths: set[str] = set()
            for entry in entries:
                added, removed, path_bytes = entry.split(b"\t", 2)
                path = path_bytes.decode("utf-8", errors="surrogateescape")
                if added == b"-" or removed == b"-":
                    binary_paths.add(path)
                else:
                    inserted += int(added)
                    deleted += int(removed)
        except (ValueError, UnicodeError) as exc:
            add(
                findings,
                "repair-delta-git-binding-failed",
                "record.repair_delta",
                str(exc),
            )
        else:
            expected_delta = {
                "inserted_lines": inserted,
                "deleted_lines": deleted,
                "binary_paths": binary_paths,
            }
            for field, actual in expected_delta.items():
                declared = (
                    set(repair_delta.get(field, []))
                    if field == "binary_paths"
                    else repair_delta.get(field)
                )
                if declared != actual:
                    add(
                        findings,
                        "repair-delta-mismatch",
                        f"record.repair_delta.{field}",
                        f"declared {declared!r}, computed {actual!r}",
                    )


def validate_recurrence_control(
    record: dict[str, Any],
    *,
    schema: dict[str, Any] | None = None,
    root: Path | None = None,
    verify_git: bool = False,
    location: str = "record",
) -> list[Finding]:
    findings = (
        validate_json_schema(record, schema, location) if schema is not None else []
    )
    if record.get("schema") != SCHEMA_NAME:
        add(findings, "wrong-schema", location, f"expected {SCHEMA_NAME}")
    generation = record.get("generation")
    decision = record.get("review_decision")
    validate_coverage(
        record.get("coverage"),
        accepting=decision == "accept",
        findings=findings,
    )
    prior_rows = record.get("prior_findings")
    prior_ids = validate_prior_findings(
        prior_rows,
        generation=generation if isinstance(generation, int) else None,
        findings=findings,
    )
    new_rows = record.get("new_findings")
    new_ids, candidate_created = validate_new_findings(
        new_rows,
        findings,
    )
    overlap = prior_ids & new_ids
    if overlap:
        add(
            findings,
            "finding-id-reused-across-generations",
            location,
            repr(sorted(overlap)),
        )
    prior_observations = {
        row.get("observation_id")
        for row in (prior_rows if isinstance(prior_rows, list) else [])
        if isinstance(row, dict) and isinstance(row.get("observation_id"), str)
    }
    new_observations = {
        row.get("observation_id")
        for row in (new_rows if isinstance(new_rows, list) else [])
        if isinstance(row, dict) and isinstance(row.get("observation_id"), str)
    }
    observation_overlap = prior_observations & new_observations
    if observation_overlap:
        add(
            findings,
            "observation-id-reused-across-sections",
            location,
            repr(sorted(observation_overlap)),
        )
    prior_effects = {
        row.get("effect_episode_id")
        for row in (prior_rows if isinstance(prior_rows, list) else [])
        if isinstance(row, dict) and isinstance(row.get("effect_episode_id"), str)
    }
    new_effects = {
        row.get("effect_episode_id")
        for row in (new_rows if isinstance(new_rows, list) else [])
        if isinstance(row, dict) and isinstance(row.get("effect_episode_id"), str)
    }
    effect_overlap = prior_effects & new_effects
    if effect_overlap:
        add(
            findings,
            "prior-effect-reclassified-as-new",
            location,
            repr(sorted(effect_overlap)),
        )
    unplanned_surfaces = validate_repair_delta(
        record.get("repair_delta"),
        prior_ids | new_ids,
        findings,
    )
    validate_validations(record.get("validations"), findings)
    validate_outcomes(
        record.get("outcomes"),
        prior_rows,
        new_rows,
        candidate_created,
        unplanned_surfaces,
        findings,
    )
    if verify_git:
        if root is None:
            add(findings, "git-root-required", location, "--git-root is required")
        else:
            verify_git_subject(
                record.get("subject"),
                record.get("repair_delta"),
                root.resolve(),
                findings,
            )

    if decision == "accept":
        for index, row in enumerate(prior_rows if isinstance(prior_rows, list) else []):
            if not isinstance(row, dict):
                continue
            disposition = row.get("disposition")
            valid_routed = (
                disposition == "routed_open" and row.get("blocking") is False
            )
            if disposition != "closed" and not valid_routed:
                add(
                    findings,
                    "accept-with-unresolved-prior-finding",
                    f"record.prior_findings[{index}]",
                    str(row.get("id")),
                )
        for index, row in enumerate(
            record.get("new_findings")
            if isinstance(record.get("new_findings"), list)
            else []
        ):
            if not isinstance(row, dict):
                continue
            if (
                row.get("disposition") != "routed_open"
                or row.get("blocking") is not False
            ):
                add(
                    findings,
                    "accept-with-unresolved-new-finding",
                    f"record.new_findings[{index}]",
                    str(row.get("id")),
                )
        repair_delta = (
            record.get("repair_delta")
            if isinstance(record.get("repair_delta"), dict)
            else {}
        )
        for index, surface in enumerate(
            repair_delta.get("new_surfaces")
            if isinstance(repair_delta.get("new_surfaces"), list)
            else []
        ):
            if (
                isinstance(surface, dict)
                and surface.get("disposition") != "reviewed"
            ):
                add(
                    findings,
                    "accept-with-unreviewed-repair-surface",
                    f"record.repair_delta.new_surfaces[{index}]",
                    str(surface.get("symbol")),
                )
        if repair_delta.get("scope_expansion") == "blocked":
            add(
                findings,
                "accept-with-blocked-scope-expansion",
                "record.repair_delta.scope_expansion",
                "blocked scope expansion cannot be integrated",
            )
        for index, row in enumerate(
            record.get("validations")
            if isinstance(record.get("validations"), list)
            else []
        ):
            if (
                isinstance(row, dict)
                and row.get("required") is True
                and row.get("status") != "passed"
            ):
                add(
                    findings,
                    "accept-with-incomplete-required-validation",
                    f"record.validations[{index}]",
                    str(row.get("id")),
                )
        terminal = (
            record.get("terminalization")
            if isinstance(record.get("terminalization"), dict)
            else {}
        )
        if (
            terminal.get("state") != "complete"
            or terminal.get("worker_alive") is not False
            or terminal.get("child_processes_active") != 0
            or terminal.get("report_state") != "committed"
            or not nonempty(terminal.get("event_log_ref"))
            or not nonempty(terminal.get("event_log_sha256"))
        ):
            add(
                findings,
                "accept-before-review-terminal",
                "record.terminalization",
                "accept requires a committed report, stopped worker, zero active children, and exact event-log evidence",
            )
    return findings


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Validate one successor-review recurrence record and gate "
            "integration without a finding-count target."
        )
    )
    parser.add_argument("--record", type=Path, required=True)
    parser.add_argument("--schema", type=Path)
    parser.add_argument("--git-root", type=Path)
    parser.add_argument("--no-git-verify", action="store_true")
    parser.add_argument("--result", type=Path)
    return parser.parse_args(argv)


def atomic_write(path: Path, payload: dict[str, Any]) -> None:
    path = path.resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    temporary.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    temporary.replace(path)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    root = (args.git_root or repository_root()).resolve()
    schema_path = (args.schema or root / SCHEMA_PATH).resolve()
    try:
        record = load_object(args.record.resolve())
        schema = load_object(schema_path)
        findings = validate_recurrence_control(
            record,
            schema=schema,
            root=root,
            verify_git=not args.no_git_verify,
            location=args.record.as_posix(),
        )
    except (OSError, ValueError, json.JSONDecodeError) as exc:
        print(f"Review recurrence gate error: {exc}", file=sys.stderr)
        return 2
    errors = [finding for finding in findings if finding.level == "error"]
    git_verified = not args.no_git_verify
    integration_allowed = (
        git_verified and not errors and record.get("review_decision") == "accept"
    )
    result = {
        "schema": RESULT_SCHEMA,
        "record": str(args.record.resolve()),
        "record_valid": not errors,
        "git_verified": git_verified,
        "review_decision": record.get("review_decision"),
        "integration_decision": "ALLOW" if integration_allowed else "BLOCK",
        "finding_count": len(findings),
        "findings": [asdict(finding) for finding in findings],
    }
    if args.result is not None:
        atomic_write(args.result, result)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if integration_allowed else 3


if __name__ == "__main__":
    raise SystemExit(main())
