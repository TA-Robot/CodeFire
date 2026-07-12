#!/usr/bin/env python3
"""Shared, dependency-free contract checks for CodeFire AI development tooling."""

from __future__ import annotations

import hashlib
import fnmatch
import json
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Iterable


TASK_SCHEMA = "codefire.ai_task_packet.v1"
REVIEW_SCHEMA = "codefire.ai_review_packet.v1"
CONTEXT_MANIFEST_SCHEMA = "codefire.ai_context_manifest.v1"
ROUTING_EVAL_SCHEMA = "codefire.ai_routing_eval.v1"
HISTORICAL_EVAL_SCHEMA = "codefire.ai_historical_eval.v1"
ROLE_MATRIX_SCHEMA = "codefire.ai_role_routing.v2"
CONTEXT_CATALOG_SCHEMA = "codefire.ai_context_packs.v1"
TIERS = ("medium", "high", "ultra")
TIER_RANK = {tier: index for index, tier in enumerate(TIERS)}
IDENTIFIER = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{1,127}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")


@dataclass(frozen=True)
class Finding:
    level: str
    code: str
    path: str
    message: str


def canonical_json_bytes(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def sha256_json(value: Any) -> str:
    return hashlib.sha256(canonical_json_bytes(value)).hexdigest()


def packet_digest(packet: dict[str, Any]) -> str:
    digestible = {key: value for key, value in packet.items() if key != "packet_sha256"}
    return sha256_json(digestible)


def load_object(path: Path) -> dict[str, Any]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError("JSON root must be an object")
    return data


def _schema_target(root_schema: dict[str, Any], reference: str) -> dict[str, Any]:
    if not reference.startswith("#/"):
        raise ValueError(f"only local JSON Schema references are supported: {reference}")
    target: Any = root_schema
    for raw_part in reference[2:].split("/"):
        part = raw_part.replace("~1", "/").replace("~0", "~")
        if not isinstance(target, dict) or part not in target:
            raise ValueError(f"unresolvable JSON Schema reference: {reference}")
        target = target[part]
    if not isinstance(target, dict):
        raise ValueError(f"JSON Schema reference is not an object: {reference}")
    return target


def _matches_schema_type(value: Any, expected: str) -> bool:
    if expected == "object":
        return isinstance(value, dict)
    if expected == "array":
        return isinstance(value, list)
    if expected == "string":
        return isinstance(value, str)
    if expected == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if expected == "number":
        return isinstance(value, (int, float)) and not isinstance(value, bool)
    if expected == "boolean":
        return isinstance(value, bool)
    if expected == "null":
        return value is None
    raise ValueError(f"unsupported JSON Schema type: {expected}")


def validate_json_schema(
    value: Any,
    schema: dict[str, Any],
    location: str,
    *,
    root_schema: dict[str, Any] | None = None,
) -> list[Finding]:
    """Evaluate the Draft 2020-12 keywords used by the repository schemas.

    This deliberately small evaluator prevents the executable contract from
    drifting below the checked-in schemas without adding a runtime dependency.
    Unsupported assertion keywords fail closed when they are introduced.
    """

    root = schema if root_schema is None else root_schema
    if "$ref" in schema:
        try:
            target = _schema_target(root, schema["$ref"])
        except ValueError as exc:
            return [Finding("error", "schema-definition-error", location, str(exc))]
        return validate_json_schema(value, target, location, root_schema=root)

    findings: list[Finding] = []
    supported_assertions = {
        "$ref", "type", "const", "enum", "required", "properties", "additionalProperties",
        "items", "minItems", "maxItems", "uniqueItems", "minLength", "maxLength", "pattern",
        "minimum", "maximum",
    }
    ignored_annotations = {"$schema", "$id", "$defs", "title", "description"}
    unknown = sorted(
        key for key in schema if key not in supported_assertions and key not in ignored_annotations
    )
    if unknown:
        findings.append(
            Finding("error", "unsupported-schema-keyword", location, ", ".join(unknown))
        )
        return findings

    expected_type = schema.get("type")
    if expected_type is not None:
        expected_types = [expected_type] if isinstance(expected_type, str) else expected_type
        if not isinstance(expected_types, list) or not all(isinstance(item, str) for item in expected_types):
            return [Finding("error", "schema-definition-error", location, "type must be a string or string list")]
        try:
            type_matches = any(_matches_schema_type(value, item) for item in expected_types)
        except ValueError as exc:
            return [Finding("error", "schema-definition-error", location, str(exc))]
        if not type_matches:
            findings.append(
                Finding("error", "schema-type", location, f"expected {expected_types}, got {type(value).__name__}")
            )
            return findings
    if "const" in schema and value != schema["const"]:
        findings.append(Finding("error", "schema-const", location, f"expected {schema['const']!r}"))
    if "enum" in schema and value not in schema["enum"]:
        findings.append(Finding("error", "schema-enum", location, f"value {value!r} is not allowed"))

    if isinstance(value, dict):
        required = schema.get("required", [])
        if not isinstance(required, list):
            findings.append(Finding("error", "schema-definition-error", location, "required must be a list"))
            return findings
        for key in required:
            if key not in value:
                findings.append(Finding("error", "schema-required", location, f"missing {key}"))
        properties = schema.get("properties", {})
        if not isinstance(properties, dict):
            findings.append(Finding("error", "schema-definition-error", location, "properties must be an object"))
            return findings
        for key, item in value.items():
            child_location = f"{location}.{key}"
            child_schema = properties.get(key)
            if isinstance(child_schema, dict):
                findings.extend(validate_json_schema(item, child_schema, child_location, root_schema=root))
            elif schema.get("additionalProperties") is False:
                findings.append(Finding("error", "schema-additional-property", child_location, "property is not allowed"))
            elif isinstance(schema.get("additionalProperties"), dict):
                findings.extend(
                    validate_json_schema(item, schema["additionalProperties"], child_location, root_schema=root)
                )

    if isinstance(value, list):
        if "minItems" in schema and len(value) < schema["minItems"]:
            findings.append(Finding("error", "schema-min-items", location, f"minimum is {schema['minItems']}"))
        if "maxItems" in schema and len(value) > schema["maxItems"]:
            findings.append(Finding("error", "schema-max-items", location, f"maximum is {schema['maxItems']}"))
        if schema.get("uniqueItems") is True:
            fingerprints = [canonical_json_bytes(item) for item in value]
            if len(fingerprints) != len(set(fingerprints)):
                findings.append(Finding("error", "schema-unique-items", location, "items must be unique"))
        item_schema = schema.get("items")
        if isinstance(item_schema, dict):
            for index, item in enumerate(value):
                findings.extend(
                    validate_json_schema(item, item_schema, f"{location}[{index}]", root_schema=root)
                )

    if isinstance(value, str):
        if "minLength" in schema and len(value) < schema["minLength"]:
            findings.append(Finding("error", "schema-min-length", location, f"minimum is {schema['minLength']}"))
        if "maxLength" in schema and len(value) > schema["maxLength"]:
            findings.append(Finding("error", "schema-max-length", location, f"maximum is {schema['maxLength']}"))
        if "pattern" in schema:
            try:
                matches = re.search(schema["pattern"], value) is not None
            except re.error as exc:
                findings.append(Finding("error", "schema-definition-error", location, f"invalid pattern: {exc}"))
            else:
                if not matches:
                    findings.append(Finding("error", "schema-pattern", location, f"does not match {schema['pattern']!r}"))

    if isinstance(value, (int, float)) and not isinstance(value, bool):
        if "minimum" in schema and value < schema["minimum"]:
            findings.append(Finding("error", "schema-minimum", location, f"minimum is {schema['minimum']}"))
        if "maximum" in schema and value > schema["maximum"]:
            findings.append(Finding("error", "schema-maximum", location, f"maximum is {schema['maximum']}"))
    return findings


def validate_context_manifest(
    manifest: dict[str, Any],
    location: str,
    json_schema: dict[str, Any] | None = None,
    *,
    root: Path | None = None,
    verify_workspace: bool = False,
) -> list[Finding]:
    """Validate manifest structure, its self-digest, and optionally live files."""

    findings = validate_json_schema(manifest, json_schema, location) if json_schema else []
    required = ("schema", "manifest_id", "base_revision", "pack_ids", "files", "manifest_sha256")
    if not require_keys(manifest, required, location, findings):
        return findings
    if manifest.get("schema") != CONTEXT_MANIFEST_SCHEMA:
        findings.append(
            Finding("error", "wrong-schema", location, f"expected {CONTEXT_MANIFEST_SCHEMA}")
        )
    files = manifest.get("files")
    if not isinstance(files, list):
        findings.append(Finding("error", "bad-context-files", f"{location}.files", "must be a list"))
        return findings
    paths: list[str] = []
    for index, item in enumerate(files):
        item_location = f"{location}.files[{index}]"
        if not isinstance(item, dict):
            findings.append(Finding("error", "bad-context-file", item_location, "must be an object"))
            continue
        path_ref = item.get("path")
        if not repository_path_valid(path_ref):
            findings.append(Finding("error", "unsafe-repository-path", item_location, repr(path_ref)))
            continue
        paths.append(path_ref)
        source_pack = item.get("source_pack")
        if source_pack != "additional" and source_pack not in manifest.get("pack_ids", []):
            findings.append(
                Finding("error", "unknown-manifest-source-pack", item_location, repr(source_pack))
            )
        if verify_workspace:
            if root is None:
                findings.append(
                    Finding("error", "manifest-verification-error", location, "root is required")
                )
                continue
            resolved_root = root.resolve()
            lexical = resolved_root / path_ref
            cursor = resolved_root
            symlink_found = False
            for part in PurePosixPath(path_ref).parts:
                cursor = cursor / part
                if cursor.is_symlink():
                    findings.append(
                        Finding("error", "context-file-symlink", item_location, path_ref)
                    )
                    symlink_found = True
                    break
            if symlink_found:
                continue
            try:
                resolved = lexical.resolve(strict=True)
                resolved.relative_to(resolved_root)
            except (FileNotFoundError, ValueError):
                findings.append(
                    Finding("error", "context-file-missing-or-outside", item_location, path_ref)
                )
                continue
            if not resolved.is_file():
                findings.append(Finding("error", "context-file-not-regular", item_location, path_ref))
                continue
            payload = resolved.read_bytes()
            if item.get("bytes") != len(payload):
                findings.append(
                    Finding("error", "context-file-size-mismatch", item_location, f"computed {len(payload)}")
                )
            digest = hashlib.sha256(payload).hexdigest()
            if item.get("sha256") != digest:
                findings.append(
                    Finding("error", "context-file-digest-mismatch", item_location, f"computed {digest}")
                )
    if len(paths) != len(set(paths)):
        findings.append(Finding("error", "duplicate-context-file", f"{location}.files", "paths must be unique"))
    digest_payload = {
        "schema": manifest.get("schema"),
        "base_revision": manifest.get("base_revision"),
        "pack_ids": manifest.get("pack_ids"),
        "files": manifest.get("files"),
    }
    digest = sha256_json(digest_payload)
    if manifest.get("manifest_sha256") != digest:
        findings.append(
            Finding("error", "context-manifest-digest-mismatch", location, f"computed {digest}")
        )
    expected_id = f"context-{digest[:16]}"
    if manifest.get("manifest_id") != expected_id:
        findings.append(
            Finding("error", "context-manifest-id-mismatch", location, f"expected {expected_id}")
        )
    return findings


def repository_path_valid(value: Any) -> bool:
    if not isinstance(value, str) or not value or "\\" in value or value.startswith("/"):
        return False
    path = PurePosixPath(value)
    return all(part not in ("", ".", "..") for part in path.parts)


def read_repository_artifact(root: Path, path_ref: str) -> bytes:
    if not repository_path_valid(path_ref):
        raise ValueError(f"unsafe repository artifact path: {path_ref!r}")
    resolved_root = root.resolve()
    cursor = resolved_root
    for part in PurePosixPath(path_ref).parts:
        cursor = cursor / part
        if cursor.is_symlink():
            raise ValueError(f"repository artifact path contains a symlink: {path_ref}")
    try:
        artifact = (resolved_root / path_ref).resolve(strict=True)
        artifact.relative_to(resolved_root)
    except (FileNotFoundError, ValueError) as exc:
        raise ValueError(f"repository artifact is missing or outside the repository: {path_ref}") from exc
    if not artifact.is_file():
        raise ValueError(f"repository artifact is not a regular file: {path_ref}")
    return artifact.read_bytes()


def canonical_git_patch(
    root: Path,
    base_revision: str,
    head_revision: str,
) -> tuple[bytes, set[str], str, str]:
    """Return deterministic committed patch bytes and the exact changed path set."""

    def resolve_commit(revision: str) -> str:
        result = subprocess.run(
            ["git", "rev-parse", "--verify", f"{revision}^{{commit}}"],
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            raise ValueError((result.stderr or result.stdout).strip() or f"invalid commit: {revision}")
        return result.stdout.strip()

    resolved_base = resolve_commit(base_revision)
    resolved_head = resolve_commit(head_revision)
    ancestor = subprocess.run(
        ["git", "merge-base", "--is-ancestor", resolved_base, resolved_head],
        cwd=root,
        capture_output=True,
        check=False,
    )
    if ancestor.returncode != 0:
        raise ValueError("review base_revision is not an ancestor of head_revision")
    common = [
        "git", "diff", "--no-ext-diff", "--no-textconv", "--no-renames",
        "--src-prefix=a/", "--dst-prefix=b/", resolved_base, resolved_head, "--",
    ]
    patch = subprocess.run(
        common[:2] + ["--binary", "--full-index"] + common[2:],
        cwd=root,
        capture_output=True,
        check=False,
    )
    if patch.returncode != 0:
        raise ValueError(patch.stderr.decode("utf-8", errors="replace").strip() or "git diff failed")
    names = subprocess.run(
        common[:2] + ["--name-only", "-z"] + common[2:],
        cwd=root,
        capture_output=True,
        check=False,
    )
    if names.returncode != 0:
        raise ValueError(names.stderr.decode("utf-8", errors="replace").strip() or "git diff --name-only failed")
    changed = {
        item.decode("utf-8", errors="surrogateescape")
        for item in names.stdout.split(b"\0")
        if item
    }
    return patch.stdout, changed, resolved_base, resolved_head


def path_patterns_overlap(first: str, second: str) -> bool:
    if fnmatch.fnmatchcase(first, second) or fnmatch.fnmatchcase(second, first):
        return True
    wildcard = re.compile(r"[*?[]")
    first_match = wildcard.search(first)
    second_match = wildcard.search(second)
    if first_match is None or second_match is None:
        # If either side is a literal, the fnmatch checks above are an exact
        # intersection test for that one path.
        return False
    first_prefix = first[: first_match.start()]
    second_prefix = second[: second_match.start()]
    if not first_prefix or not second_prefix:
        # Character classes or wildcards at the start make intersection hard
        # to disprove. Fail closed rather than permit an overlapping write.
        return True
    return first_prefix.startswith(second_prefix) or second_prefix.startswith(first_prefix)


def require_keys(
    value: Any,
    keys: Iterable[str],
    location: str,
    findings: list[Finding],
) -> bool:
    if not isinstance(value, dict):
        findings.append(Finding("error", "expected-object", location, "value must be an object"))
        return False
    missing = sorted(set(keys) - set(value))
    for key in missing:
        findings.append(Finding("error", "missing-key", location, f"missing required key: {key}"))
    return not missing


def check_string_list(
    value: Any,
    location: str,
    findings: list[Finding],
    *,
    nonempty: bool = False,
    paths: bool = False,
) -> list[str]:
    if not isinstance(value, list):
        findings.append(Finding("error", "expected-list", location, "value must be a list"))
        return []
    if nonempty and not value:
        findings.append(Finding("error", "empty-list", location, "list must not be empty"))
    valid: list[str] = []
    for index, item in enumerate(value):
        item_location = f"{location}[{index}]"
        if not isinstance(item, str) or not item:
            findings.append(Finding("error", "expected-string", item_location, "item must be a non-empty string"))
        elif paths and not repository_path_valid(item):
            findings.append(Finding("error", "unsafe-repository-path", item_location, repr(item)))
        else:
            valid.append(item)
    if len(valid) != len(set(valid)):
        findings.append(Finding("error", "duplicate-list-item", location, "items must be unique"))
    return valid


def route_risk(
    dimensions: dict[str, Any],
    mandatory_ultra: list[str],
    matrix: dict[str, Any],
) -> tuple[int, str]:
    policy = matrix["risk_routing"]
    dimension_names = policy["dimensions"]
    if set(dimensions) != set(dimension_names):
        missing = sorted(set(dimension_names) - set(dimensions))
        extra = sorted(set(dimensions) - set(dimension_names))
        raise ValueError(f"risk dimensions differ; missing={missing}, extra={extra}")
    values = list(dimensions.values())
    if any(isinstance(value, bool) or not isinstance(value, int) or not 0 <= value <= 2 for value in values):
        raise ValueError("each risk dimension must be an integer from 0 through 2")
    allowed_reasons = set(policy["mandatory_ultra"])
    unknown_reasons = sorted(set(mandatory_ultra) - allowed_reasons)
    if unknown_reasons:
        raise ValueError(f"unknown mandatory-Ultra reasons: {unknown_reasons}")
    score = sum(values)
    if mandatory_ultra:
        return score, "ultra"
    for tier in TIERS:
        bounds = policy["thresholds"][tier]
        if bounds["min"] <= score <= bounds["max"]:
            return score, tier
    raise ValueError(f"score {score} is not covered by routing thresholds")


def validate_context_catalog(data: dict[str, Any], location: str = "context-packs.json") -> tuple[list[Finding], set[str]]:
    findings: list[Finding] = []
    if data.get("schema") != CONTEXT_CATALOG_SCHEMA:
        findings.append(Finding("error", "wrong-schema", location, f"expected {CONTEXT_CATALOG_SCHEMA}"))
    packs = data.get("packs")
    if not isinstance(packs, list) or not packs:
        findings.append(Finding("error", "missing-context-packs", location, "packs must be a non-empty list"))
        return findings, set()
    ids: set[str] = set()
    parents: list[tuple[str, str]] = []
    for index, pack in enumerate(packs):
        pack_location = f"{location}:packs[{index}]"
        if not require_keys(pack, ("id", "title", "required_files"), pack_location, findings):
            continue
        pack_id = pack.get("id")
        if not isinstance(pack_id, str) or not IDENTIFIER.fullmatch(pack_id):
            findings.append(Finding("error", "bad-context-pack-id", pack_location, repr(pack_id)))
            continue
        if pack_id in ids:
            findings.append(Finding("error", "duplicate-context-pack-id", pack_location, pack_id))
        ids.add(pack_id)
        check_string_list(pack.get("required_files"), f"{pack_location}.required_files", findings, paths=True)
        check_string_list(pack.get("likely_files", []), f"{pack_location}.likely_files", findings, paths=True)
        for parent in check_string_list(pack.get("extends", []), f"{pack_location}.extends", findings):
            parents.append((pack_id, parent))
    for pack_id, parent in parents:
        if parent not in ids:
            findings.append(Finding("error", "missing-context-parent", location, f"{pack_id} extends {parent}"))
    return findings, ids


def validate_task_packet(
    packet: dict[str, Any],
    matrix: dict[str, Any],
    context_pack_ids: set[str],
    location: str,
    json_schema: dict[str, Any] | None = None,
) -> list[Finding]:
    findings: list[Finding] = (
        validate_json_schema(packet, json_schema, location) if json_schema is not None else []
    )
    required = (
        "schema", "packet_id", "status", "objective", "base_revision", "role", "risk",
        "context", "scope", "non_goals", "invariants", "validation", "handoff",
    )
    if not require_keys(packet, required, location, findings):
        return findings
    if packet.get("schema") != TASK_SCHEMA:
        findings.append(Finding("error", "wrong-schema", location, f"expected {TASK_SCHEMA}"))
    packet_id = packet.get("packet_id")
    if not isinstance(packet_id, str) or not IDENTIFIER.fullmatch(packet_id):
        findings.append(Finding("error", "bad-packet-id", location, repr(packet_id)))
    if packet.get("status") not in {"draft", "ready", "in_progress", "blocked", "completed", "cancelled"}:
        findings.append(Finding("error", "bad-packet-status", location, repr(packet.get("status"))))
    for key in ("objective", "base_revision"):
        if not isinstance(packet.get(key), str) or not packet[key].strip():
            findings.append(Finding("error", f"bad-{key}", location, f"{key} must be non-empty"))

    roles = {
        role.get("role"): role
        for role in matrix.get("roles", [])
        if isinstance(role, dict) and isinstance(role.get("role"), str)
    }
    role_name = packet.get("role")
    if role_name not in roles:
        findings.append(Finding("error", "unknown-role", location, repr(role_name)))

    risk = packet.get("risk")
    if require_keys(risk, ("dimensions", "mandatory_ultra", "score", "selected_tier", "rationale"), f"{location}.risk", findings):
        dimensions = risk.get("dimensions")
        mandatory = risk.get("mandatory_ultra")
        if isinstance(dimensions, dict) and isinstance(mandatory, list) and all(isinstance(item, str) for item in mandatory):
            try:
                computed_score, computed_tier = route_risk(dimensions, mandatory, matrix)
            except (KeyError, TypeError, ValueError) as exc:
                findings.append(Finding("error", "bad-risk-decision", f"{location}.risk", str(exc)))
            else:
                if risk.get("score") != computed_score:
                    findings.append(Finding("error", "wrong-risk-score", f"{location}.risk", f"expected {computed_score}"))
                if risk.get("selected_tier") != computed_tier:
                    findings.append(Finding("error", "wrong-risk-tier", f"{location}.risk", f"expected {computed_tier}"))
                baseline = roles.get(role_name, {}).get("reasoning_tier") if role_name in roles else None
                if baseline in TIER_RANK and TIER_RANK[computed_tier] < TIER_RANK[baseline]:
                    findings.append(Finding("error", "below-role-baseline", f"{location}.risk", f"{role_name} requires at least {baseline}"))
        else:
            findings.append(Finding("error", "bad-risk-shape", f"{location}.risk", "dimensions must be an object and mandatory_ultra a string list"))
        if not isinstance(risk.get("rationale"), str) or not risk["rationale"].strip():
            findings.append(Finding("error", "missing-risk-rationale", f"{location}.risk", "rationale is required"))

    context = packet.get("context")
    if require_keys(context, ("pack_ids", "additional_files", "evidence_refs"), f"{location}.context", findings):
        pack_ids = check_string_list(context.get("pack_ids"), f"{location}.context.pack_ids", findings)
        for pack_id in pack_ids:
            if pack_id not in context_pack_ids:
                findings.append(Finding("error", "unknown-context-pack", f"{location}.context.pack_ids", pack_id))
        check_string_list(context.get("additional_files"), f"{location}.context.additional_files", findings, paths=True)
        check_string_list(context.get("evidence_refs"), f"{location}.context.evidence_refs", findings)
        if "resolved_manifest" in context and not repository_path_valid(context["resolved_manifest"]):
            findings.append(Finding("error", "unsafe-repository-path", f"{location}.context.resolved_manifest", repr(context["resolved_manifest"])))

    scope = packet.get("scope")
    if require_keys(scope, ("read_set", "write_set", "disallowed_paths", "disallowed_actions"), f"{location}.scope", findings):
        read_set = check_string_list(scope.get("read_set"), f"{location}.scope.read_set", findings, paths=True)
        write_set = check_string_list(scope.get("write_set"), f"{location}.scope.write_set", findings, paths=True)
        disallowed = check_string_list(scope.get("disallowed_paths"), f"{location}.scope.disallowed_paths", findings, paths=True)
        check_string_list(scope.get("disallowed_actions"), f"{location}.scope.disallowed_actions", findings, nonempty=True)
        exact_conflicts = sorted(set(write_set) & set(disallowed))
        if exact_conflicts:
            findings.append(Finding("error", "write-disallow-conflict", f"{location}.scope", repr(exact_conflicts)))
        pattern_conflicts = sorted(
            (write, denied)
            for write in write_set
            for denied in disallowed
            if write != denied and path_patterns_overlap(write, denied)
        )
        if pattern_conflicts:
            findings.append(
                Finding("error", "write-disallow-pattern-conflict", f"{location}.scope", repr(pattern_conflicts))
            )

    check_string_list(packet.get("non_goals"), f"{location}.non_goals", findings, nonempty=True)
    invariants = packet.get("invariants")
    if not isinstance(invariants, list) or not invariants:
        findings.append(Finding("error", "missing-invariants", f"{location}.invariants", "at least one invariant is required"))
    else:
        invariant_ids: list[str] = []
        for index, invariant in enumerate(invariants):
            invariant_location = f"{location}.invariants[{index}]"
            if require_keys(invariant, ("id", "statement", "falsification_test"), invariant_location, findings):
                for key in ("id", "statement", "falsification_test"):
                    if not isinstance(invariant.get(key), str) or not invariant[key].strip():
                        findings.append(Finding("error", "bad-invariant", invariant_location, f"{key} must be non-empty"))
                if isinstance(invariant.get("id"), str):
                    invariant_ids.append(invariant["id"])
        if len(invariant_ids) != len(set(invariant_ids)):
            findings.append(Finding("error", "duplicate-invariant-id", f"{location}.invariants", "ids must be unique"))

    validations = packet.get("validation")
    if not isinstance(validations, list) or not validations:
        findings.append(Finding("error", "missing-validation", f"{location}.validation", "at least one command is required"))
    else:
        validation_ids: list[str] = []
        for index, validation in enumerate(validations):
            validation_location = f"{location}.validation[{index}]"
            if require_keys(validation, ("id", "command", "purpose", "required"), validation_location, findings):
                if not isinstance(validation.get("required"), bool):
                    findings.append(Finding("error", "bad-required-flag", validation_location, "required must be boolean"))
                for key in ("id", "command", "purpose"):
                    if not isinstance(validation.get(key), str) or not validation[key].strip():
                        findings.append(Finding("error", "bad-validation", validation_location, f"{key} must be non-empty"))
                if isinstance(validation.get("id"), str):
                    validation_ids.append(validation["id"])
        if len(validation_ids) != len(set(validation_ids)):
            findings.append(Finding("error", "duplicate-validation-id", f"{location}.validation", "ids must be unique"))
        if packet.get("status") in {"ready", "in_progress", "completed"} and not any(
            isinstance(item, dict) and item.get("required") is True for item in validations
        ):
            findings.append(
                Finding("error", "missing-required-validation", f"{location}.validation", "ready work needs a required gate")
            )

    handoff = packet.get("handoff")
    if require_keys(handoff, ("required_outputs", "evidence_digest_required"), f"{location}.handoff", findings):
        check_string_list(handoff.get("required_outputs"), f"{location}.handoff.required_outputs", findings, nonempty=True)
        if not isinstance(handoff.get("evidence_digest_required"), bool):
            findings.append(Finding("error", "bad-evidence-digest-flag", f"{location}.handoff", "must be boolean"))
        elif packet.get("status") in {"ready", "in_progress", "completed"} and handoff.get("evidence_digest_required") is not True:
            findings.append(
                Finding("error", "evidence-digest-required", f"{location}.handoff", "ready work must bind evidence")
            )

    declared_digest = packet.get("packet_sha256")
    if declared_digest is not None:
        if not isinstance(declared_digest, str) or not SHA256.fullmatch(declared_digest):
            findings.append(Finding("error", "bad-packet-digest", location, repr(declared_digest)))
        elif declared_digest != packet_digest(packet):
            findings.append(Finding("error", "packet-digest-mismatch", location, f"computed {packet_digest(packet)}"))
    return findings


def validate_review_packet(
    review: dict[str, Any],
    location: str,
    json_schema: dict[str, Any] | None = None,
) -> list[Finding]:
    findings: list[Finding] = (
        validate_json_schema(review, json_schema, location) if json_schema is not None else []
    )
    required = (
        "schema", "review_id", "task_packet_id", "task_packet_sha256", "base_revision",
        "head_revision", "patch_sha256", "reviewer_independent", "review_tier",
        "files_changed", "claims", "validation_evidence", "findings", "decision",
    )
    if not require_keys(review, required, location, findings):
        return findings
    if review.get("schema") != REVIEW_SCHEMA:
        findings.append(Finding("error", "wrong-schema", location, f"expected {REVIEW_SCHEMA}"))
    for key in ("task_packet_sha256", "patch_sha256"):
        value = review.get(key)
        if not isinstance(value, str) or not SHA256.fullmatch(value):
            findings.append(Finding("error", "bad-review-digest", f"{location}.{key}", repr(value)))
    if review.get("reviewer_independent") is not True:
        findings.append(Finding("error", "non-independent-review", location, "reviewer_independent must be true"))
    if review.get("review_tier") not in {"high", "ultra"}:
        findings.append(Finding("error", "bad-review-tier", location, repr(review.get("review_tier"))))
    check_string_list(review.get("files_changed"), f"{location}.files_changed", findings, paths=True)
    if review.get("decision") not in {"approve", "request_changes", "block"}:
        findings.append(Finding("error", "bad-review-decision", location, repr(review.get("decision"))))
    claims = review.get("claims")
    if not isinstance(claims, list) or not claims:
        findings.append(Finding("error", "missing-review-claims", f"{location}.claims", "at least one claim is required"))
    else:
        for index, claim in enumerate(claims):
            claim_location = f"{location}.claims[{index}]"
            if require_keys(claim, ("claim", "evidence_refs", "verified"), claim_location, findings):
                if not isinstance(claim.get("claim"), str) or not claim["claim"].strip():
                    findings.append(Finding("error", "bad-review-claim", claim_location, "claim must be non-empty"))
                check_string_list(claim.get("evidence_refs"), f"{claim_location}.evidence_refs", findings)
                if not isinstance(claim.get("verified"), bool):
                    findings.append(Finding("error", "bad-verified-flag", claim_location, "verified must be boolean"))
    evidence = review.get("validation_evidence")
    if not isinstance(evidence, list):
        findings.append(Finding("error", "bad-validation-evidence", f"{location}.validation_evidence", "must be a list"))
    else:
        evidence_ids: list[str] = []
        for index, item in enumerate(evidence):
            item_location = f"{location}.validation_evidence[{index}]"
            if isinstance(item, dict):
                validation_id = item.get("validation_id")
                if isinstance(validation_id, str):
                    evidence_ids.append(validation_id)
        if len(evidence_ids) != len(set(evidence_ids)):
            findings.append(
                Finding("error", "duplicate-validation-evidence-id", f"{location}.validation_evidence", "ids must be unique")
            )
    findings_data = review.get("findings")
    if not isinstance(findings_data, list):
        findings.append(Finding("error", "bad-review-findings", f"{location}.findings", "must be a list"))
    if review.get("decision") == "approve":
        if any(isinstance(item, dict) and item.get("severity") in {"block", "high"} for item in findings_data or []):
            findings.append(Finding("error", "approval-with-blocking-finding", location, "approve cannot retain block/high findings"))
        if any(isinstance(item, dict) and item.get("status") != "passed" for item in evidence or []):
            findings.append(Finding("error", "approval-with-incomplete-validation", location, "approve requires all recorded validation to pass"))
        if any(
            isinstance(item, dict)
            and item.get("status") == "passed"
            and not item.get("log_sha256")
            for item in evidence or []
        ):
            findings.append(
                Finding("error", "approval-validation-without-log-digest", location, "passed validation needs log_sha256")
            )
        if any(isinstance(item, dict) and item.get("verified") is not True for item in claims or []):
            findings.append(Finding("error", "approval-with-unverified-claim", location, "approve requires verified claims"))
        if not evidence:
            findings.append(Finding("error", "approval-without-validation", location, "approve requires validation evidence"))
        if any(
            isinstance(item, dict)
            and item.get("verified") is True
            and not item.get("evidence_refs")
            for item in claims or []
        ):
            findings.append(Finding("error", "approval-claim-without-evidence", location, "verified claims need evidence refs"))
    return findings


def validate_review_against_task(
    review: dict[str, Any],
    task: dict[str, Any],
    location: str,
    *,
    patch_bytes: bytes | None = None,
    actual_files: set[str] | None = None,
    resolved_base_revision: str | None = None,
    resolved_head_revision: str | None = None,
    post_validation_capabilities: set[str] | None = None,
    root: Path | None = None,
) -> list[Finding]:
    """Bind a review to its task, committed patch, changed paths, and evidence."""

    findings: list[Finding] = []
    post_validation_capabilities = post_validation_capabilities or set()
    if review.get("task_packet_id") != task.get("packet_id"):
        findings.append(
            Finding("error", "review-task-id-mismatch", location, repr(task.get("packet_id")))
        )
    expected_task_digest = packet_digest(task)
    if review.get("task_packet_sha256") != expected_task_digest:
        findings.append(
            Finding("error", "review-task-digest-mismatch", location, f"expected {expected_task_digest}")
        )
    if review.get("base_revision") != task.get("base_revision"):
        findings.append(
            Finding("error", "review-task-base-mismatch", location, repr(task.get("base_revision")))
        )
    decision = review.get("decision")
    approval = decision == "approve"
    if resolved_base_revision is not None and review.get("base_revision") != resolved_base_revision:
        findings.append(
            Finding("error", "review-base-not-canonical-commit", location, resolved_base_revision)
        )
    if resolved_head_revision is not None and review.get("head_revision") != resolved_head_revision:
        findings.append(
            Finding("error", "review-head-not-canonical-commit", location, resolved_head_revision)
        )
    elif approval and resolved_head_revision is None:
        findings.append(
            Finding("error", "approval-head-unverified", location, "canonical head commit is required")
        )
    task_tier = task.get("risk", {}).get("selected_tier") if isinstance(task.get("risk"), dict) else None
    review_tier = review.get("review_tier")
    minimum_review_tier = "ultra" if task_tier == "ultra" else "high"
    if review_tier in TIER_RANK and TIER_RANK[review_tier] < TIER_RANK[minimum_review_tier]:
        findings.append(
            Finding("error", "review-tier-below-task", location, f"requires {minimum_review_tier}")
        )

    if patch_bytes is None:
        if approval:
            findings.append(
                Finding("error", "approval-patch-unverified", location, "canonical patch bytes are required")
            )
    else:
        digest = hashlib.sha256(patch_bytes).hexdigest()
        if review.get("patch_sha256") != digest:
            findings.append(
                Finding("error", "review-patch-digest-mismatch", location, f"computed {digest}")
            )

    declared_files = {
        item for item in review.get("files_changed", []) if isinstance(item, str)
    }
    if actual_files is None:
        if approval:
            findings.append(
                Finding("error", "approval-files-unverified", location, "git changed paths are required")
            )
        files_to_check = declared_files
    else:
        files_to_check = actual_files
        missing = sorted(actual_files - declared_files)
        extra = sorted(declared_files - actual_files)
        if missing or extra:
            findings.append(
                Finding(
                    "error",
                    "review-files-changed-mismatch",
                    f"{location}.files_changed",
                    f"missing={missing}, extra={extra}",
                )
            )

    scope = task.get("scope") if isinstance(task.get("scope"), dict) else {}
    write_set = scope.get("write_set") if isinstance(scope.get("write_set"), list) else []
    disallowed = scope.get("disallowed_paths") if isinstance(scope.get("disallowed_paths"), list) else []
    for path_ref in sorted(files_to_check):
        path_location = f"{location}.files_changed"
        if not any(fnmatch.fnmatchcase(path_ref, pattern) for pattern in write_set):
            findings.append(
                Finding("error", "review-file-outside-write-set", path_location, path_ref)
            )
        if any(fnmatch.fnmatchcase(path_ref, pattern) for pattern in disallowed):
            findings.append(
                Finding("error", "review-file-disallowed", path_location, path_ref)
            )

    evidence_by_id = {
        item.get("validation_id"): item
        for item in review.get("validation_evidence", [])
        if isinstance(item, dict) and isinstance(item.get("validation_id"), str)
    }
    required_validations = [
        item
        for item in task.get("validation", [])
        if isinstance(item, dict) and item.get("required") is True
    ]
    for validation in required_validations:
        validation_id = validation.get("id")
        command = validation.get("command")
        evidence = evidence_by_id.get(validation_id)
        if evidence is None:
            if approval:
                findings.append(
                    Finding(
                        "error",
                        "review-missing-required-validation",
                        f"{location}.validation_evidence",
                        repr(validation_id),
                    )
                )
            continue
        elif evidence.get("command") != command:
            findings.append(
                Finding(
                    "error",
                    "review-validation-command-mismatch",
                    f"{location}.validation_evidence",
                    f"{validation_id!r} expected {command!r}",
                )
            )
        elif approval and evidence.get("status") != "passed":
            findings.append(
                Finding(
                    "error",
                    "review-required-validation-not-passed",
                    f"{location}.validation_evidence",
                    repr(command),
                )
            )
        required_capability = validation.get("requires_capability")
        if (
            approval
            and required_capability in post_validation_capabilities
        ):
            evidence_digest = evidence.get("capability_evidence_sha256")
            evidence_path = evidence.get("capability_evidence_path")
            if not isinstance(evidence_digest, str) or not SHA256.fullmatch(evidence_digest):
                findings.append(
                    Finding(
                        "error",
                        "review-missing-post-validation-evidence",
                        f"{location}.validation_evidence",
                        repr(required_capability),
                    )
                )
                continue
            if not isinstance(evidence_path, str):
                findings.append(
                    Finding(
                        "error",
                        "review-missing-capability-evidence-path",
                        f"{location}.validation_evidence",
                        repr(required_capability),
                    )
                )
                continue
            if root is None:
                findings.append(
                    Finding(
                        "error",
                        "review-capability-evidence-unverified",
                        f"{location}.validation_evidence",
                        "repository root is required",
                    )
                )
                continue
            try:
                raw_evidence = read_repository_artifact(root, evidence_path)
            except (OSError, ValueError) as exc:
                findings.append(
                    Finding("error", "review-capability-evidence-unavailable", evidence_path, str(exc))
                )
                continue
            computed_evidence_digest = hashlib.sha256(raw_evidence).hexdigest()
            if computed_evidence_digest != evidence_digest:
                findings.append(
                    Finding(
                        "error",
                        "review-capability-evidence-digest-mismatch",
                        evidence_path,
                        f"computed {computed_evidence_digest}",
                    )
                )
                continue
            try:
                evidence_payload = json.loads(raw_evidence)
            except json.JSONDecodeError as exc:
                findings.append(
                    Finding("error", "review-capability-evidence-invalid-json", evidence_path, str(exc))
                )
                continue
            if not isinstance(evidence_payload, dict):
                findings.append(
                    Finding("error", "review-capability-evidence-invalid-json", evidence_path, "root must be an object")
                )
                continue
            if required_capability == "rust-source-bound-binary":
                from build_source_bound_rust import validate_provenance_payload

                expected_revision = resolved_head_revision or str(review.get("head_revision"))
                for message in validate_provenance_payload(
                    root,
                    evidence_payload,
                    expected_revision=expected_revision,
                ):
                    findings.append(
                        Finding("error", "review-rust-provenance-invalid", evidence_path, message)
                    )
            else:
                findings.append(
                    Finding(
                        "error",
                        "review-capability-evidence-unsupported",
                        evidence_path,
                        repr(required_capability),
                    )
                )
    return findings


def validate_routing_corpus(data: dict[str, Any], matrix: dict[str, Any], location: str) -> list[Finding]:
    findings: list[Finding] = []
    if data.get("schema") != ROUTING_EVAL_SCHEMA:
        findings.append(Finding("error", "wrong-schema", location, f"expected {ROUTING_EVAL_SCHEMA}"))
    cases = data.get("cases")
    if not isinstance(cases, list) or not cases:
        findings.append(Finding("error", "missing-routing-cases", location, "cases must be a non-empty list"))
        return findings
    seen: set[str] = set()
    tier_coverage: set[str] = set()
    reason_coverage: set[str] = set()
    numeric_score_coverage: set[int] = set()
    for index, case in enumerate(cases):
        case_location = f"{location}:cases[{index}]"
        if not require_keys(case, ("id", "dimensions", "mandatory_ultra", "expected_score", "expected_tier", "rationale"), case_location, findings):
            continue
        case_id = case.get("id")
        if not isinstance(case_id, str) or not IDENTIFIER.fullmatch(case_id):
            findings.append(Finding("error", "bad-routing-case-id", case_location, repr(case_id)))
        elif case_id in seen:
            findings.append(Finding("error", "duplicate-routing-case", case_location, case_id))
        else:
            seen.add(case_id)
        dimensions = case.get("dimensions")
        mandatory = case.get("mandatory_ultra")
        if not isinstance(dimensions, dict) or not isinstance(mandatory, list) or not all(isinstance(item, str) for item in mandatory):
            findings.append(Finding("error", "bad-routing-case", case_location, "invalid dimensions or mandatory_ultra"))
            continue
        try:
            score, tier = route_risk(dimensions, mandatory, matrix)
        except (KeyError, TypeError, ValueError) as exc:
            findings.append(Finding("error", "bad-routing-case", case_location, str(exc)))
            continue
        if case.get("expected_score") != score:
            findings.append(Finding("error", "routing-score-mismatch", case_location, f"computed {score}"))
        if case.get("expected_tier") != tier:
            findings.append(Finding("error", "routing-tier-mismatch", case_location, f"computed {tier}"))
        tier_coverage.add(tier)
        reason_coverage.update(mandatory)
        if not mandatory:
            numeric_score_coverage.add(score)
    expected_reasons = set(matrix["risk_routing"]["mandatory_ultra"])
    if tier_coverage != set(TIERS):
        findings.append(Finding("error", "missing-tier-coverage", location, repr(sorted(set(TIERS) - tier_coverage))))
    if reason_coverage != expected_reasons:
        findings.append(Finding("error", "missing-mandatory-ultra-coverage", location, repr(sorted(expected_reasons - reason_coverage))))
    expected_scores = set(range(13))
    if numeric_score_coverage != expected_scores:
        findings.append(
            Finding(
                "error",
                "missing-numeric-score-coverage",
                location,
                repr(sorted(expected_scores - numeric_score_coverage)),
            )
        )
    return findings


def validate_historical_eval_corpus(
    data: dict[str, Any],
    matrix: dict[str, Any],
    root: Path,
    location: str,
) -> list[Finding]:
    """Validate replay cases while keeping candidate input and evaluator oracle distinct."""

    findings: list[Finding] = []
    if data.get("schema") != HISTORICAL_EVAL_SCHEMA:
        findings.append(Finding("error", "wrong-schema", location, f"expected {HISTORICAL_EVAL_SCHEMA}"))
    if not isinstance(data.get("purpose"), str) or not data["purpose"].strip():
        findings.append(Finding("error", "missing-eval-purpose", location, "purpose must be non-empty"))
    cases = data.get("cases")
    if not isinstance(cases, list) or len(cases) < 6:
        findings.append(Finding("error", "insufficient-historical-cases", location, "at least six cases are required"))
        return findings

    allowed_task_classes = {
        "completion_audit",
        "contract_review",
        "defect_diagnosis",
        "durability_review",
        "schema_drift_review",
        "security_review",
        "state_recovery_review",
        "trust_boundary_review",
    }
    seen: set[str] = set()
    source_families: set[str] = set()
    tier_coverage: set[str] = set()
    for index, case in enumerate(cases):
        case_location = f"{location}:cases[{index}]"
        required = ("id", "source_ids", "task_class", "prompt", "candidate_context", "risk", "oracle")
        if not require_keys(case, required, case_location, findings):
            continue
        if set(case) != set(required):
            findings.append(
                Finding(
                    "error",
                    "historical-case-fields",
                    case_location,
                    f"unexpected or missing fields: {sorted(set(case) ^ set(required))}",
                )
            )
        case_id = case.get("id")
        if not isinstance(case_id, str) or not IDENTIFIER.fullmatch(case_id):
            findings.append(Finding("error", "bad-historical-case-id", case_location, repr(case_id)))
        elif case_id in seen:
            findings.append(Finding("error", "duplicate-historical-case", case_location, case_id))
        else:
            seen.add(case_id)

        source_ids = check_string_list(
            case.get("source_ids"), f"{case_location}.source_ids", findings, nonempty=True
        )
        for source_id in source_ids:
            if source_id.startswith("CFB-"):
                source_families.add("CFB")
            if source_id.startswith("CFR-"):
                source_families.add("CFR")
        if case.get("task_class") not in allowed_task_classes:
            findings.append(
                Finding("error", "bad-historical-task-class", case_location, repr(case.get("task_class")))
            )
        if not isinstance(case.get("prompt"), str) or not case["prompt"].strip():
            findings.append(Finding("error", "missing-historical-prompt", case_location, "prompt is required"))

        candidate = case.get("candidate_context")
        if require_keys(candidate, ("observations", "context_files"), f"{case_location}.candidate_context", findings):
            if set(candidate) != {"observations", "context_files"}:
                findings.append(
                    Finding(
                        "error",
                        "candidate-context-fields",
                        f"{case_location}.candidate_context",
                        "only observations and context_files are allowed",
                    )
                )
            check_string_list(
                candidate.get("observations"),
                f"{case_location}.candidate_context.observations",
                findings,
                nonempty=True,
            )
            context_files = check_string_list(
                candidate.get("context_files"),
                f"{case_location}.candidate_context.context_files",
                findings,
                nonempty=True,
                paths=True,
            )
            for path_ref in context_files:
                if not (root / path_ref).exists():
                    findings.append(
                        Finding("error", "missing-eval-context", f"{case_location}.candidate_context", path_ref)
                    )

        risk = case.get("risk")
        risk_required = ("dimensions", "mandatory_ultra", "expected_score", "expected_tier")
        if require_keys(risk, risk_required, f"{case_location}.risk", findings):
            if set(risk) != set(risk_required):
                findings.append(
                    Finding("error", "historical-risk-fields", f"{case_location}.risk", "risk fields must be exact")
                )
            dimensions = risk.get("dimensions")
            mandatory = risk.get("mandatory_ultra")
            if not isinstance(dimensions, dict) or not isinstance(mandatory, list) or not all(
                isinstance(item, str) for item in mandatory
            ):
                findings.append(Finding("error", "bad-historical-risk", f"{case_location}.risk", "invalid shape"))
            else:
                try:
                    score, tier = route_risk(dimensions, mandatory, matrix)
                except (KeyError, TypeError, ValueError) as exc:
                    findings.append(Finding("error", "bad-historical-risk", f"{case_location}.risk", str(exc)))
                else:
                    if risk.get("expected_score") != score:
                        findings.append(
                            Finding("error", "historical-score-mismatch", f"{case_location}.risk", f"computed {score}")
                        )
                    if risk.get("expected_tier") != tier:
                        findings.append(
                            Finding("error", "historical-tier-mismatch", f"{case_location}.risk", f"computed {tier}")
                        )
                    tier_coverage.add(tier)

        oracle = case.get("oracle")
        oracle_required = (
            "source_files",
            "must_identify",
            "must_preserve",
            "disallowed_claims",
            "validation_refs",
        )
        if require_keys(oracle, oracle_required, f"{case_location}.oracle", findings):
            if set(oracle) != set(oracle_required):
                findings.append(
                    Finding("error", "historical-oracle-fields", f"{case_location}.oracle", "oracle fields must be exact")
                )
            source_files = check_string_list(
                oracle.get("source_files"),
                f"{case_location}.oracle.source_files",
                findings,
                nonempty=True,
                paths=True,
            )
            for path_ref in source_files:
                if not (root / path_ref).is_file():
                    findings.append(
                        Finding("error", "missing-eval-oracle-source", f"{case_location}.oracle", path_ref)
                    )
            for key in ("must_identify", "must_preserve", "disallowed_claims", "validation_refs"):
                check_string_list(
                    oracle.get(key), f"{case_location}.oracle.{key}", findings, nonempty=True
                )

    if source_families != {"CFB", "CFR"}:
        findings.append(
            Finding("error", "missing-historical-source-family", location, repr(sorted({"CFB", "CFR"} - source_families)))
        )
    if not {"high", "ultra"}.issubset(tier_coverage):
        findings.append(
            Finding("error", "missing-historical-tier", location, repr(sorted({"high", "ultra"} - tier_coverage)))
        )
    if not any(isinstance(case, dict) and case.get("task_class") == "completion_audit" for case in cases):
        findings.append(Finding("error", "missing-completion-audit-case", location, "completion audit is required"))
    return findings
