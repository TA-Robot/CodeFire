#!/usr/bin/env python3
"""Parse layer and frozen vocabularies for the recurrence gate v2.

This module owns one responsibility: turn already-schema-checked JSON into
immutable typed values, and hold the vocabularies and manager-fixed tables the
other v2 modules read.  It never touches Git, the filesystem, or the decision.

The v1 record shape stays untouched in ``check_review_recurrence.py``; nothing
here reads or rewrites it.
"""

from __future__ import annotations

import json
import math
import re
from dataclasses import dataclass
from pathlib import PurePosixPath
from typing import Any

from ai_contracts import Finding


RECORD_SCHEMA = "codefire.review_recurrence_control.v2"
MANIFEST_SCHEMA = "codefire.review_recurrence_repair_manifest.v1"
RESULT_SCHEMA = "codefire.review_recurrence_gate_result.v2"
TIMED_SCHEMA = "codefire.timed_gate.v2"
TIMED_SCHEMA_V1 = "codefire.timed_gate.v1"
LEGACY_RECORD_SCHEMA = "codefire.review_recurrence_control.v1"

RECORD_SCHEMA_PATH = (
    "docs/development/ai/schemas/review-recurrence-control.v2.schema.json"
)
MANIFEST_SCHEMA_PATH = (
    "docs/development/ai/schemas/review-recurrence-repair-manifest.v1.schema.json"
)
RESULT_SCHEMA_PATH = (
    "docs/development/ai/schemas/review-recurrence-gate-result.v2.schema.json"
)
LIVE_ARM_ANCHOR_SCHEMA_PATH = (
    "docs/development/ai/schemas/review-recurrence-live-arm-anchor.v1.schema.json"
)
TIMED_SCHEMA_PATH = "docs/development/ai/schemas/timed-gate-result.v2.schema.json"

AUTHORITY_SCHEMA_IDS: dict[str, str] = {
    RECORD_SCHEMA_PATH: (
        "https://codefire.dev/schemas/ai/"
        "review-recurrence-control.v2.schema.json"
    ),
    MANIFEST_SCHEMA_PATH: (
        "https://codefire.dev/schemas/ai/"
        "review-recurrence-repair-manifest.v1.schema.json"
    ),
    RESULT_SCHEMA_PATH: (
        "https://codefire.dev/schemas/ai/"
        "review-recurrence-gate-result.v2.schema.json"
    ),
    LIVE_ARM_ANCHOR_SCHEMA_PATH: (
        "https://codefire.dev/schemas/ai/"
        "review-recurrence-live-arm-anchor.v1.schema.json"
    ),
    TIMED_SCHEMA_PATH: (
        "https://codefire.dev/schemas/ai/timed-gate-result.v2.schema.json"
    ),
}

MAX_JSON_CONTAINER_DEPTH = 64

# N-3 extension point.  The anchor does not exist yet: a separate manager task
# must pin each live arm's exact manifest identity before a live arm runs.
LIVE_ARM_ANCHOR_PATH = (
    "docs/development/ai/review-recurrence/live-arm-anchor.v1.json"
)
LIVE_ARM_ANCHOR_SCHEMA = "codefire.review_recurrence_live_arm_anchor.v1"

TASK_PACKET_PREFIX = "docs/development/ai/task-packets/"
REVIEW_PACKET_PREFIXES = (
    "docs/development/ai/review-packets/",
    "docs/development/ai/task-packets/",
)
REPORT_PREFIX = "docs/development/ai/swarm-results/"
EVIDENCE_PREFIX = "docs/development/ai/review-recurrence/evidence/"
RECORD_PREFIX = "docs/development/ai/review-recurrence/records/"
RESULT_PREFIX = "docs/development/ai/review-recurrence/results/"

BLOB_MODE = "100644"

# The six field names the contract prohibits by name.  Semantic proxy detection
# beyond these names stays an independent review judgement and is not claimed.
PROHIBITED_QUOTA_FIELDS = (
    "target_finding_count",
    "minimum_findings",
    "expected_findings",
    "target_observations",
    "minimum_surfaces",
    "minimum_obligations",
)

EVIDENCE_STATES = (
    "DEFECT_REPRODUCED",
    "DEFECT_SOURCE_PROVEN",
    "DEFECT_INJECTION_ONLY",
    "CLOSURE_UNPROVEN",
    "ORACLE_VACUOUS",
    "CLAIM_FALSE",
    "SCOPE_BOUNDARY",
    "REQUIREMENT_UNSATISFIABLE",
    "CLOSED_PROVEN",
)

NEXT_ACTIONS = (
    "PRODUCT_REPAIR",
    "RUN_CLOSURE_EVIDENCE",
    "REPAIR_ORACLE",
    "REPAIR_CLAIM",
    "DOCUMENT_NON_REACHABILITY",
    "MANAGER_SCOPE_DECISION",
    "MANAGER_REQUIREMENT_DECISION",
    "NONE",
)

# F1h is distinct from the evidence-state routing table below. A derivation's
# disposition has exactly one admissible action, and every action except NONE
# is visible and blocking in I1 coverage.
DERIVATION_ROUTES: dict[str, str] = {
    "derivation_complete": "NONE",
    "derivation_contradicted": "REPAIR_CLAIM",
    "derivation_unavailable": "RUN_CLOSURE_EVIDENCE",
}

# I5: routing decides the kind of successor work, never integration authority.
STATE_ROUTES: dict[str, frozenset[str]] = {
    "DEFECT_REPRODUCED": frozenset({"PRODUCT_REPAIR"}),
    "DEFECT_SOURCE_PROVEN": frozenset({"PRODUCT_REPAIR"}),
    "DEFECT_INJECTION_ONLY": frozenset(
        {"DOCUMENT_NON_REACHABILITY", "PRODUCT_REPAIR"}
    ),
    "CLOSURE_UNPROVEN": frozenset({"RUN_CLOSURE_EVIDENCE"}),
    "ORACLE_VACUOUS": frozenset({"REPAIR_ORACLE"}),
    "CLAIM_FALSE": frozenset({"REPAIR_CLAIM"}),
    "SCOPE_BOUNDARY": frozenset({"MANAGER_SCOPE_DECISION"}),
    "REQUIREMENT_UNSATISFIABLE": frozenset({"MANAGER_REQUIREMENT_DECISION"}),
    "CLOSED_PROVEN": frozenset({"NONE"}),
}

# The only nonblocking pair, and only when closure is separately proven.
NONBLOCKING_PAIR = ("CLOSED_PROVEN", "NONE")

MANAGER_DECISION_ACTIONS = frozenset(
    {"MANAGER_SCOPE_DECISION", "MANAGER_REQUIREMENT_DECISION"}
)

ARTIFACT_ROLES = (
    "repair_packet",
    "repair_manifest",
    "candidate_artifact",
    "review_packet",
    "report",
    "timed_result",
    "timed_log",
    "terminal_event_evidence",
    "prior_v2_record",
    "prior_v2_result",
    "legacy_bootstrap_report",
    "routing_scope_decision",
    "prior_closure_evidence",
    "non_reachability_evidence",
)

# Roles admissible as observation, surface, or derivation evidence.  Authority
# and control roles are deliberately excluded so a record cannot cite its own
# packet as the proof of an observation.
OBSERVATION_ROLES = frozenset(
    {
        "candidate_artifact",
        "report",
        "timed_result",
        "timed_log",
        "terminal_event_evidence",
        "prior_closure_evidence",
        "non_reachability_evidence",
        "routing_scope_decision",
    }
)


@dataclass(frozen=True)
class LegacyRoot:
    """One manager-fixed bootstrap authority root.

    No other legacy binding may serve as a root without a new manager contract
    and an independent preflight.
    """

    arm: str
    predecessor_candidate: str
    commit: str
    report_path: str
    report_sha256: str
    review_generation: int
    obligation_origins: tuple[tuple[str, int], ...]

    def obligation_origin_map(self) -> dict[str, int]:
        """Return a fresh lookup without exposing mutable manager authority."""

        return dict(self.obligation_origins)


LEGACY_ROOTS: dict[str, LegacyRoot] = {
    "cargo": LegacyRoot(
        arm="cargo",
        predecessor_candidate="4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912",
        commit="130878d321f6be166f554ac2c68a59e2b36afa88",
        report_path=(
            "docs/development/ai/swarm-results/"
            "aia-source-keyed-cargo-gate-second-closure-rereview-claude-20260731/"
            "review.md"
        ),
        report_sha256=(
            "aad067b2b8feb80830c41db1608dd31f0e7e7d7ca1f37ab01d040336f788b4fd"
        ),
        review_generation=3,
        obligation_origins=(
            ("R1", 1),
            ("R2", 2),
            ("R3", 2),
            ("R4", 2),
            ("R5", 2),
        ),
    ),
    "kgi_kpi": LegacyRoot(
        arm="kgi_kpi",
        predecessor_candidate="753c8db103579d021795d3fe6a97c5d45d4a111a",
        commit="12f9d0b5c30ce2695ec6c96e5876703efd7b0d90",
        report_path=(
            "docs/development/ai/swarm-results/"
            "development-flow-kgi-kpi-r3-rereview-claude-20260731/review.md"
        ),
        report_sha256=(
            "23b59e01825e3f66d2d38b754348e3d51457f3c465def578ac99ce52729a204f"
        ),
        review_generation=4,
        obligation_origins=(
            ("A1-PREREQ", 3),
            ("A1-SUBJECT", 2),
            ("A2-DENOM", 2),
            ("A3-D1", 2),
            ("A3-D5", 2),
            ("A3-N1", 3),
            ("A4-STAGE", 2),
            ("A5-ECHO", 3),
            ("A5-SCOPE", 3),
            ("A6-N2", 3),
            ("A6-N4", 3),
            ("A7-C1", 3),
            ("A7-C2", 3),
            ("A7-C3", 3),
            ("A7-C4", 3),
            ("A8-C5", 3),
            ("A8-C6", 2),
            ("N3-MERGE-SEAL", 3),
            ("P1-RENAME-SHALLOW", 3),
            ("R1C-C-TRUST", 3),
        ),
    ),
}


@dataclass(frozen=True)
class PortPacket:
    """One canonical-port Markdown packet, fixed by path, commit and digest."""

    arm: str
    path: str
    source_commit: str
    sha256: str
    absent_at: str


PORT_PACKETS: dict[str, PortPacket] = {
    "cargo": PortPacket(
        arm="cargo",
        path=(
            "docs/development/ai/task-packets/"
            "cfr168-source-keyed-cargo-third-closure-repair-20260731.md"
        ),
        source_commit="8c51b712f1e3937fd462a95c70b8de79687b9289",
        sha256=(
            "7464f158cc04373af2cb5e0eacb37212233d920dc9233bf220e553361a43adc7"
        ),
        absent_at="4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912",
    ),
    "kgi_kpi": PortPacket(
        arm="kgi_kpi",
        path=(
            "docs/development/ai/task-packets/"
            "cfr168-development-flow-kgi-kpi-r4-repair-20260731.md"
        ),
        source_commit="a22fad0698b1623d8898c47699e7ba4e61294b5f",
        sha256=(
            "bbd6ef472903079238f24ad92bf567a9143fdbd1874975d5364ddd4eea8964ad"
        ),
        absent_at="753c8db103579d021795d3fe6a97c5d45d4a111a",
    ),
}

PORT_REQUIRED_REPAIR_PATHS: dict[str, frozenset[str]] = {
    "cargo": frozenset(),
    "kgi_kpi": frozenset(
        {
            "docs/development/ai/swarm-batches/"
            "development-flow-kgi-kpi-zero-residual-repair-r3-20260731.md",
            "docs/development/ai/swarm-batches/"
            "development-flow-kgi-kpi-zero-residual-repair-r4-20260731.md",
        }
    ),
}

# Emitted verbatim in every v2 result.  Each sentence is a limit the gate does
# not prove; none of them is softened elsewhere in the output.
PROVENANCE_LIMITS = (
    "Git proves topology in the presented history only; it does not prove when "
    "any text was authored and cannot detect a rewritten history that presents "
    "a conforming topology.",
    "A command that mutates and restores bytes during its own execution stays "
    "outside this provenance proof, even with empty clean-before, clean-after "
    "and index-flag states.",
    "The timed runner does not prove honest execution, semantic oracle "
    "quality, or that a test selector chose a nonzero corpus.",
    "Complete semantic symbol enumeration and whether a bound oracle "
    "semantically proves closure remain human review judgements.",
    "Whether the frozen manifest collections are complete is a manager and "
    "independent-preflight judgement, not a machine result.",
    "Retention integrity of this generation's record and result is verified "
    "only later, by the successor generation's authority resolution.",
    "Git binds an artifact's bytes and ancestry and cannot attest human "
    "authorship or independent acceptance of its content.",
)


def error(code: str, location: str, message: str) -> Finding:
    return Finding("error", code, location, message)


class _DuplicateJSONObjectKey(ValueError):
    def __init__(self, key: str) -> None:
        super().__init__(key)
        self.key = key


class _NonFiniteJSONNumber(ValueError):
    def __init__(self, token: str) -> None:
        super().__init__(token)
        self.token = token


class _JSONNumberOutOfRange(ValueError):
    pass


def _strict_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise _DuplicateJSONObjectKey(key)
        value[key] = item
    return value


def _strict_float(token: str) -> float:
    try:
        value = float(token)
    except (OverflowError, ValueError) as exc:
        raise _JSONNumberOutOfRange from exc
    if not math.isfinite(value):
        raise _NonFiniteJSONNumber(token)
    return value


def _strict_integer(token: str) -> int:
    try:
        return int(token)
    except (OverflowError, ValueError) as exc:
        raise _JSONNumberOutOfRange from exc


def _reject_json_constant(token: str) -> Any:
    raise _NonFiniteJSONNumber(token)


def _decoded_boundary_findings(value: Any, location: str) -> list[Finding]:
    """Check decoded text and container depth without recursive Python calls."""

    findings: list[Finding] = []
    initial_depth = 1 if isinstance(value, (dict, list)) else 0
    pending: list[tuple[Any, str, int]] = [(value, location, initial_depth)]
    while pending:
        item, item_location, depth = pending.pop()
        if isinstance(item, (dict, list)) and depth > MAX_JSON_CONTAINER_DEPTH:
            return [
                error(
                    "json-nesting-too-deep",
                    item_location,
                    f"container depth exceeds {MAX_JSON_CONTAINER_DEPTH}",
                )
            ]
        if isinstance(item, str):
            if any(0xD800 <= ord(character) <= 0xDFFF for character in item):
                findings.append(
                    error(
                        "unsafe-json-surrogate",
                        item_location,
                        f"text contains a lone surrogate: {item!r}",
                    )
                )
            continue
        if isinstance(item, float):
            # The parse hook rejects these. Keep a second, total boundary check
            # so a future decoder change cannot silently widen authority.
            if not math.isfinite(item):
                findings.append(
                    error(
                        "non-finite-json-number",
                        item_location,
                        "decoded JSON number is not finite",
                    )
                )
            continue
        if isinstance(item, list):
            child_depth = depth + 1
            pending.extend(
                (child, f"{item_location}[{index}]", child_depth)
                for index, child in reversed(tuple(enumerate(item)))
            )
            continue
        if isinstance(item, dict):
            child_depth = depth + 1
            for key, child in reversed(tuple(item.items())):
                key_location = f"{item_location}[{key!r}]"
                if any(0xD800 <= ord(character) <= 0xDFFF for character in key):
                    findings.append(
                        error(
                            "unsafe-json-surrogate",
                            item_location,
                            f"an object key contains a lone surrogate: {key!r}",
                        )
                    )
                pending.append((child, key_location, child_depth))
    return findings


def strict_decode_json(
    payload: bytes | str,
    location: str,
) -> tuple[Any | None, list[Finding]]:
    """Decode one v2 authority JSON boundary without throwing.

    This is the public decoder for current and retained records, manifests,
    timed results, gate results, live anchors, and their validator-owned schema
    blobs. It accepts exact UTF-8 bytes or already-decoded text. Repeated object
    members, non-finite numbers, unsupported integer magnitude, lone
    surrogates, and container depth above 64 are rejected before typed use.
    """

    if isinstance(payload, bytes):
        try:
            text = payload.decode("utf-8")
        except UnicodeDecodeError as exc:
            return None, [
                error(
                    "invalid-json-utf8",
                    location,
                    f"JSON is not valid UTF-8 at byte {exc.start}",
                )
            ]
    elif isinstance(payload, str):
        text = payload
    else:
        return None, [
            error(
                "invalid-json-input-type",
                location,
                f"expected bytes or str, got {type(payload).__name__}",
            )
        ]
    try:
        value = json.loads(
            text,
            object_pairs_hook=_strict_object,
            parse_constant=_reject_json_constant,
            parse_float=_strict_float,
            parse_int=_strict_integer,
        )
    except _DuplicateJSONObjectKey as exc:
        return None, [
            error(
                "duplicate-json-object-key",
                location,
                f"object member {exc.key!r} appears more than once",
            )
        ]
    except _NonFiniteJSONNumber as exc:
        return None, [
            error(
                "non-finite-json-number",
                location,
                f"JSON numeric token {exc.token!r} is not finite",
            )
        ]
    except _JSONNumberOutOfRange:
        return None, [
            error(
                "json-number-out-of-range",
                location,
                "JSON number is outside the supported decoder range",
            )
        ]
    except RecursionError:
        return None, [
            error(
                "json-nesting-too-deep",
                location,
                f"container depth exceeds {MAX_JSON_CONTAINER_DEPTH}",
            )
        ]
    except json.JSONDecodeError as exc:
        return None, [
            error(
                "invalid-json",
                location,
                f"{exc.msg} at line {exc.lineno} column {exc.colno}",
            )
        ]
    except (OverflowError, ValueError):
        # Python's integer digit guard and decoder numeric failures are
        # deliberately normalized instead of escaping as implementation detail.
        return None, [
            error(
                "json-number-out-of-range",
                location,
                "JSON number is outside the supported decoder range",
            )
        ]
    boundary_findings = _decoded_boundary_findings(value, location)
    return (None, boundary_findings) if boundary_findings else (value, [])


def _schema_reference_target(
    root: dict[str, Any], reference: Any
) -> tuple[dict[str, Any] | None, str | None]:
    if not isinstance(reference, str) or not reference.startswith("#/"):
        return None, f"only local string JSON Schema references are supported: {reference!r}"
    target: Any = root
    for raw_part in reference[2:].split("/"):
        part = raw_part.replace("~1", "/").replace("~0", "~")
        if not isinstance(target, dict) or part not in target:
            return None, f"unresolvable JSON Schema reference: {reference}"
        target = target[part]
    if not isinstance(target, dict):
        return None, f"JSON Schema reference is not an object: {reference}"
    return target, None


def _schema_definition_structure_findings(
    root: dict[str, Any], location: str
) -> list[Finding]:
    """Check every operand the repository's compact evaluator may consume."""

    supported_assertions = {
        "$ref",
        "type",
        "const",
        "enum",
        "required",
        "properties",
        "additionalProperties",
        "items",
        "minItems",
        "maxItems",
        "uniqueItems",
        "minLength",
        "maxLength",
        "pattern",
        "minimum",
        "maximum",
    }
    annotations = {"$schema", "$id", "$defs", "title", "description"}
    allowed_types = {
        "object",
        "array",
        "string",
        "integer",
        "number",
        "boolean",
        "null",
    }
    findings: list[Finding] = []
    active: set[int] = set()
    validated: set[int] = set()

    def invalid(path: str, message: str) -> None:
        findings.append(error("schema-definition-error", path, message))

    def visit(node: Any, path: str) -> None:
        if not isinstance(node, dict):
            invalid(path, f"schema node must be an object, got {type(node).__name__}")
            return
        identity = id(node)
        if identity in active:
            invalid(path, "recursive JSON Schema references are not supported")
            return
        if identity in validated:
            return
        active.add(identity)

        unknown = sorted(
            key
            for key in node
            if key not in supported_assertions and key not in annotations
        )
        if unknown:
            invalid(path, f"unsupported schema keywords: {', '.join(unknown)}")

        if "$ref" in node:
            target, problem = _schema_reference_target(root, node["$ref"])
            if problem is not None:
                invalid(f"{path}.$ref", problem)
            elif target is not None:
                visit(target, f"{path}.$ref")

        expected_type = node.get("type")
        if expected_type is not None:
            expected_types = (
                [expected_type] if isinstance(expected_type, str) else expected_type
            )
            if (
                not isinstance(expected_types, list)
                or not expected_types
                or not all(
                    isinstance(item, str) and item in allowed_types
                    for item in expected_types
                )
                or len(set(expected_types)) != len(expected_types)
            ):
                invalid(
                    f"{path}.type",
                    "type must be a nonempty unique supported string or string list",
                )

        enum = node.get("enum")
        if "enum" in node and (not isinstance(enum, list) or not enum):
            invalid(f"{path}.enum", "enum must be a nonempty array")

        required = node.get("required")
        if "required" in node and (
            not isinstance(required, list)
            or not all(isinstance(item, str) for item in required)
            or len(set(required)) != len(required)
        ):
            invalid(
                f"{path}.required",
                "required must be an array of unique strings",
            )

        properties = node.get("properties")
        if "properties" in node:
            if not isinstance(properties, dict):
                invalid(f"{path}.properties", "properties must be an object")
            else:
                for name, child in properties.items():
                    visit(child, f"{path}.properties.{name}")

        additional = node.get("additionalProperties")
        if "additionalProperties" in node:
            if isinstance(additional, dict):
                visit(additional, f"{path}.additionalProperties")
            elif not isinstance(additional, bool):
                invalid(
                    f"{path}.additionalProperties",
                    "additionalProperties must be a boolean or schema object",
                )

        if "items" in node:
            items = node.get("items")
            if isinstance(items, dict):
                visit(items, f"{path}.items")
            else:
                invalid(f"{path}.items", "items must be a schema object")

        for keyword in ("minItems", "maxItems", "minLength", "maxLength"):
            if keyword not in node:
                continue
            bound = node[keyword]
            if not isinstance(bound, int) or isinstance(bound, bool) or bound < 0:
                invalid(f"{path}.{keyword}", f"{keyword} must be a nonnegative integer")
        for lower, upper in (("minItems", "maxItems"), ("minLength", "maxLength")):
            if (
                isinstance(node.get(lower), int)
                and not isinstance(node.get(lower), bool)
                and isinstance(node.get(upper), int)
                and not isinstance(node.get(upper), bool)
                and node[lower] > node[upper]
            ):
                invalid(path, f"{lower} must not exceed {upper}")

        if "uniqueItems" in node and not isinstance(node["uniqueItems"], bool):
            invalid(f"{path}.uniqueItems", "uniqueItems must be a boolean")

        if "pattern" in node:
            pattern = node["pattern"]
            if not isinstance(pattern, str):
                invalid(f"{path}.pattern", "pattern must be a string")
            else:
                try:
                    re.compile(pattern)
                except re.error as exc:
                    invalid(f"{path}.pattern", f"invalid pattern: {exc}")

        for keyword in ("minimum", "maximum"):
            if keyword not in node:
                continue
            bound = node[keyword]
            if (
                not isinstance(bound, (int, float))
                or isinstance(bound, bool)
                or (isinstance(bound, float) and not math.isfinite(bound))
            ):
                invalid(f"{path}.{keyword}", f"{keyword} must be a finite number")
        if (
            isinstance(node.get("minimum"), (int, float))
            and not isinstance(node.get("minimum"), bool)
            and isinstance(node.get("maximum"), (int, float))
            and not isinstance(node.get("maximum"), bool)
            and node["minimum"] > node["maximum"]
        ):
            invalid(path, "minimum must not exceed maximum")

        definitions = node.get("$defs")
        if "$defs" in node:
            if not isinstance(definitions, dict):
                invalid(f"{path}.$defs", "$defs must be an object")
            else:
                for name, child in definitions.items():
                    visit(child, f"{path}.$defs.{name}")

        for annotation in ("$schema", "$id", "title", "description"):
            if annotation in node and not isinstance(node[annotation], str):
                invalid(f"{path}.{annotation}", f"{annotation} must be a string")

        active.remove(identity)
        validated.add(identity)

    visit(root, location)
    return findings


def validate_schema_definition_identity(
    value: Any,
    expected_id: str,
    location: str,
) -> list[Finding]:
    """Validate exact registry identity and the compact evaluator's operands."""

    if not isinstance(value, dict):
        return [
            error(
                "schema-definition-not-an-object",
                location,
                f"expected object, got {type(value).__name__}",
            )
        ]
    if value.get("$id") != expected_id:
        return [
            error(
                "wrong-schema-id",
                location,
                f"expected {expected_id}, got {value.get('$id')!r}",
            )
        ]
    return _schema_definition_structure_findings(value, location)


def validate_live_arm_pin_semantics(
    value: Any,
    location: str,
) -> list[Finding]:
    """Enforce the closed result pin's state/null cross-field contract."""

    if not isinstance(value, dict):
        return [
            error(
                "live-arm-pin-not-an-object",
                location,
                f"expected object, got {type(value).__name__}",
            )
        ]
    state = value.get("state")
    anchor = value.get("anchor")
    arm = value.get("arm")
    if state == "unavailable" and anchor is None and arm is None:
        return []
    if state == "verified" and isinstance(anchor, dict) and isinstance(arm, dict):
        return []
    return [
        error(
            "live-arm-pin-state-inconsistent",
            location,
            "unavailable requires null anchor and arm; verified requires both objects",
        )
    ]


def validate_deciding_gate_semantics(
    value: Any,
    location: str,
    *,
    code_prefix: str = "deciding-gate",
) -> list[Finding]:
    """Bind ``clean_committed`` to its non-path gate identity facts."""

    if not isinstance(value, dict):
        return [
            error(
                f"{code_prefix}-not-an-object",
                location,
                f"expected object, got {type(value).__name__}",
            )
        ]
    if value.get("state") != "clean_committed":
        return [
            error(
                f"{code_prefix}-not-clean-committed",
                location,
                f"state is {value.get('state')!r}",
            )
        ]

    findings: list[Finding] = []
    for field in ("source_revision", "source_tree"):
        commit = value.get(field)
        if not (
            isinstance(commit, str)
            and len(commit) == 40
            and all(character in "0123456789abcdef" for character in commit)
        ):
            findings.append(
                error(
                    f"{code_prefix}-clean-{field.replace('_', '-')}-invalid",
                    f"{location}.{field}",
                    "clean_committed requires one canonical 40-hex Git identity",
                )
            )

    artifacts = value.get("artifacts")
    if not isinstance(artifacts, list) or not artifacts:
        findings.append(
            error(
                f"{code_prefix}-clean-artifacts-invalid",
                f"{location}.artifacts",
                "clean_committed requires a nonempty gate artifact set",
            )
        )

    worktree = value.get("worktree")
    expected_worktree_keys = {"status", "index_flagged_entries"}
    if not isinstance(worktree, dict):
        findings.append(
            error(
                f"{code_prefix}-worktree-invalid",
                f"{location}.worktree",
                "clean_committed requires its closed worktree state",
            )
        )
    else:
        actual_worktree_keys = set(worktree)
        if actual_worktree_keys != expected_worktree_keys:
            findings.append(
                error(
                    f"{code_prefix}-worktree-shape-invalid",
                    f"{location}.worktree",
                    f"expected keys {sorted(expected_worktree_keys)}, "
                    f"got {sorted(actual_worktree_keys)}",
                )
            )
        if (
            worktree.get("status") != []
            or worktree.get("index_flagged_entries") != []
        ):
            findings.append(
                error(
                    f"{code_prefix}-worktree-not-clean",
                    f"{location}.worktree",
                    "clean_committed requires empty status and index_flagged_entries",
                )
            )
    if "detail" in value:
        findings.append(
            error(
                f"{code_prefix}-clean-detail-present",
                f"{location}.detail",
                "clean_committed gate identities must not retain an error detail",
            )
        )
    return findings


def path_components_safe(value: str) -> bool:
    """Accept one canonical repository-relative lexical POSIX spelling."""

    if (
        not isinstance(value, str)
        or not value
        or value.startswith("/")
        or "\\" in value
        or any(
            ord(character) < 32
            or ord(character) == 127
            or 0xD800 <= ord(character) <= 0xDFFF
            for character in value
        )
    ):
        return False
    parts = value.split("/")
    return (
        all(part not in ("", ".", "..") for part in parts)
        and PurePosixPath(value).as_posix() == value
    )


@dataclass(frozen=True)
class ArtifactRef:
    ref_id: str
    role: str
    commit: str
    path: str
    sha256: str
    mode: str


@dataclass(frozen=True)
class Surface:
    surface_id: str
    kind: str
    scope_disposition: str
    evidence_ref_ids: tuple[str, ...]
    scope_routing_ref: str | None


@dataclass(frozen=True)
class PathReview:
    path: str
    disposition: str
    surfaces: tuple[Surface, ...]


@dataclass(frozen=True)
class Observations:
    before_ref_ids: tuple[str, ...]
    after_ref_ids: tuple[str, ...]
    negative_control: str
    negative_control_ref_ids: tuple[str, ...]


@dataclass(frozen=True)
class ObligationRow:
    id: str
    origin_generation: int
    evidence_state: str
    next_action: str
    closure_evidence_ref: str | None
    non_reachability_ref: str | None
    routing_decision_ref: str | None
    observations: Observations


@dataclass(frozen=True)
class FindingRow:
    id: str
    evidence_state: str
    next_action: str
    routing_decision_ref: str | None
    observations: Observations


@dataclass(frozen=True)
class DerivationRow:
    claim_id: str
    path: str
    locus: str
    path_existed_in_predecessor: bool
    locus_existed_in_predecessor: bool
    predecessor_behaviour: str
    candidate_behaviour: str
    evidence_state: str
    evidence_ref_ids: tuple[str, ...]
    disposition: str
    next_action: str


@dataclass(frozen=True)
class ValidationRow:
    id: str
    result_ref: str
    log_ref: str
    status: str
    exit_code: int | None
    timed_out: bool
    log_complete: bool


@dataclass(frozen=True)
class Terminalization:
    state: str
    worker_alive: bool
    child_processes_active: int
    event_evidence_ref: str


@dataclass(frozen=True)
class Authority:
    kind: str
    bootstrap_arm: str | None
    commit: str
    artifact_refs: tuple[str, ...]


@dataclass(frozen=True)
class Subject:
    predecessor_candidate_revision: str
    candidate_revision: str
    candidate_tree: str
    patch_sha256: str
    changed_paths: frozenset[str]
    repair_packet_commit: str
    review_control_commit: str
    report_commit: str
    evidence_commit: str
    repair_packet_ref: str
    repair_manifest_ref: str
    review_packet_ref: str
    report_ref: str
    authority: Authority


@dataclass(frozen=True)
class RecordModel:
    control_id: str
    generation: int
    predecessor_obligation_completeness: str
    review_decision: str
    subject: Subject
    artifacts: tuple[ArtifactRef, ...]
    path_reviews: tuple[PathReview, ...]
    prior_obligations: tuple[ObligationRow, ...]
    new_findings: tuple[FindingRow, ...]
    derivations: tuple[DerivationRow, ...]
    validations: tuple[ValidationRow, ...]
    terminalization: Terminalization

    def artifact_index(self) -> dict[str, ArtifactRef]:
        return {artifact.ref_id: artifact for artifact in self.artifacts}

    def rows(self) -> tuple[tuple[str, Any], ...]:
        """Every dispositioned row, tagged with its record location."""

        prior = tuple(
            (f"record.prior_obligations[{index}]", row)
            for index, row in enumerate(self.prior_obligations)
        )
        new = tuple(
            (f"record.new_findings[{index}]", row)
            for index, row in enumerate(self.new_findings)
        )
        return prior + new


@dataclass(frozen=True)
class ManifestObligation:
    obligation_id: str
    origin_generation: int
    closure_validation_ids: frozenset[str]


@dataclass(frozen=True)
class ManifestClaim:
    claim_id: str
    kind: str
    obligation_id: str | None
    locus_path: str


@dataclass(frozen=True)
class ManifestValidation:
    validation_id: str
    argv: tuple[str, ...]


@dataclass(frozen=True)
class ManifestModel:
    manifest_id: str
    generation: int
    predecessor_candidate_revision: str
    authority_kind: str
    authority_bootstrap_arm: str | None
    authority_commit: str
    repair_packet_path: str
    repair_packet_sha256: str
    obligations: tuple[ManifestObligation, ...]
    claims: tuple[ManifestClaim, ...]
    validations: tuple[ManifestValidation, ...]

    def obligation_map(self) -> dict[str, frozenset[str]]:
        return {row.obligation_id: row.closure_validation_ids for row in self.obligations}

    def obligation_origin_map(self) -> dict[str, int]:
        return {row.obligation_id: row.origin_generation for row in self.obligations}

    def validation_argv(self) -> dict[str, tuple[str, ...]]:
        return {row.validation_id: row.argv for row in self.validations}


def _str(value: Any) -> str:
    return value if isinstance(value, str) else ""


def _opt_str(value: Any) -> str | None:
    return value if isinstance(value, str) else None


def _tuple(value: Any) -> tuple[str, ...]:
    if not isinstance(value, list):
        return ()
    return tuple(item for item in value if isinstance(item, str))


def _int(value: Any) -> int:
    return value if isinstance(value, int) and not isinstance(value, bool) else -1


def _observations(raw: Any) -> Observations:
    data = raw if isinstance(raw, dict) else {}
    return Observations(
        before_ref_ids=_tuple(data.get("before_ref_ids")),
        after_ref_ids=_tuple(data.get("after_ref_ids")),
        negative_control=_str(data.get("negative_control")),
        negative_control_ref_ids=_tuple(data.get("negative_control_ref_ids")),
    )


def _array_items(
    raw: dict[str, Any],
    key: str,
    location: str,
    findings: list[Finding],
) -> list[Any]:
    value = raw.get(key)
    if not isinstance(value, list):
        findings.append(
            error(
                "record-array-invalid",
                f"{location}.{key}",
                f"expected array, got {type(value).__name__}",
            )
        )
        return []
    return value


def parse_record(
    raw: Any,
    location: str = "record",
) -> tuple[RecordModel | None, list[Finding]]:
    """Build the immutable record model.

    Structural failures return ``None`` so every downstream verifier can assume
    a well-formed model instead of re-checking types.  Schema validation runs
    first in the gate; this is the fail-closed second line.
    """

    findings: list[Finding] = []
    if not isinstance(raw, dict):
        return None, [error("record-not-an-object", location, type(raw).__name__)]
    if raw.get("schema") != RECORD_SCHEMA:
        findings.append(
            error(
                "wrong-record-schema",
                f"{location}.schema",
                f"expected {RECORD_SCHEMA}, got {raw.get('schema')!r}",
            )
        )
    subject_raw = raw.get("subject")
    authority_raw = (
        subject_raw.get("predecessor_authority")
        if isinstance(subject_raw, dict)
        else None
    )
    if not isinstance(subject_raw, dict) or not isinstance(authority_raw, dict):
        findings.append(
            error("record-subject-missing", f"{location}.subject", "subject is required")
        )
        return None, findings
    authority = Authority(
        kind=_str(authority_raw.get("kind")),
        bootstrap_arm=_opt_str(authority_raw.get("bootstrap_arm")),
        commit=_str(authority_raw.get("commit")),
        artifact_refs=_tuple(authority_raw.get("artifact_refs")),
    )
    subject = Subject(
        predecessor_candidate_revision=_str(
            subject_raw.get("predecessor_candidate_revision")
        ),
        candidate_revision=_str(subject_raw.get("candidate_revision")),
        candidate_tree=_str(subject_raw.get("candidate_tree")),
        patch_sha256=_str(subject_raw.get("patch_sha256")),
        changed_paths=frozenset(_tuple(subject_raw.get("changed_paths"))),
        repair_packet_commit=_str(subject_raw.get("repair_packet_commit")),
        review_control_commit=_str(subject_raw.get("review_control_commit")),
        report_commit=_str(subject_raw.get("report_commit")),
        evidence_commit=_str(subject_raw.get("evidence_commit")),
        repair_packet_ref=_str(subject_raw.get("repair_packet_ref")),
        repair_manifest_ref=_str(subject_raw.get("repair_manifest_ref")),
        review_packet_ref=_str(subject_raw.get("review_packet_ref")),
        report_ref=_str(subject_raw.get("report_ref")),
        authority=authority,
    )
    artifacts: list[ArtifactRef] = []
    for index, item in enumerate(_array_items(raw, "artifacts", location, findings)):
        if not isinstance(item, dict):
            findings.append(
                error("artifact-not-an-object", f"{location}.artifacts[{index}]", "")
            )
            continue
        artifacts.append(
            ArtifactRef(
                ref_id=_str(item.get("ref_id")),
                role=_str(item.get("role")),
                commit=_str(item.get("commit")),
                path=_str(item.get("path")),
                sha256=_str(item.get("sha256")),
                mode=_str(item.get("mode")),
            )
        )
    path_reviews: list[PathReview] = []
    for index, item in enumerate(
        _array_items(raw, "path_reviews", location, findings)
    ):
        if not isinstance(item, dict):
            findings.append(
                error("path-review-not-an-object", f"{location}.path_reviews[{index}]", "")
            )
            continue
        surfaces: list[Surface] = []
        raw_surfaces = item.get("surfaces")
        if not isinstance(raw_surfaces, list):
            findings.append(
                error(
                    "path-review-surfaces-invalid",
                    f"{location}.path_reviews[{index}].surfaces",
                    f"expected array, got {type(raw_surfaces).__name__}",
                )
            )
            raw_surfaces = []
        for surface_index, surface in enumerate(raw_surfaces):
            if not isinstance(surface, dict):
                findings.append(
                    error(
                        "surface-not-an-object",
                        f"{location}.path_reviews[{index}].surfaces[{surface_index}]",
                        f"got {type(surface).__name__}",
                    )
                )
                continue
            surfaces.append(
                Surface(
                    surface_id=_str(surface.get("surface_id")),
                    kind=_str(surface.get("kind")),
                    scope_disposition=_str(surface.get("scope_disposition")),
                    evidence_ref_ids=_tuple(surface.get("evidence_ref_ids")),
                    scope_routing_ref=_opt_str(surface.get("scope_routing_ref")),
                )
            )
        path_reviews.append(
            PathReview(
                path=_str(item.get("path")),
                disposition=_str(item.get("disposition")),
                surfaces=tuple(surfaces),
            )
        )
    obligations: list[ObligationRow] = []
    for index, item in enumerate(
        _array_items(raw, "prior_obligations", location, findings)
    ):
        if not isinstance(item, dict):
            findings.append(
                error(
                    "prior-obligation-not-an-object",
                    f"{location}.prior_obligations[{index}]",
                    f"got {type(item).__name__}",
                )
            )
            continue
        obligations.append(
            ObligationRow(
                id=_str(item.get("id")),
                origin_generation=_int(item.get("origin_generation")),
                evidence_state=_str(item.get("evidence_state")),
                next_action=_str(item.get("next_action")),
                closure_evidence_ref=_opt_str(item.get("closure_evidence_ref")),
                non_reachability_ref=_opt_str(item.get("non_reachability_ref")),
                routing_decision_ref=_opt_str(item.get("routing_decision_ref")),
                observations=_observations(item.get("observations")),
            )
        )
    new_findings: list[FindingRow] = []
    for index, item in enumerate(
        _array_items(raw, "new_findings", location, findings)
    ):
        if not isinstance(item, dict):
            findings.append(
                error(
                    "new-finding-not-an-object",
                    f"{location}.new_findings[{index}]",
                    f"got {type(item).__name__}",
                )
            )
            continue
        new_findings.append(
            FindingRow(
                id=_str(item.get("id")),
                evidence_state=_str(item.get("evidence_state")),
                next_action=_str(item.get("next_action")),
                routing_decision_ref=_opt_str(item.get("routing_decision_ref")),
                observations=_observations(item.get("observations")),
            )
        )
    derivations: list[DerivationRow] = []
    for index, item in enumerate(
        _array_items(raw, "derivations", location, findings)
    ):
        if not isinstance(item, dict):
            findings.append(
                error(
                    "derivation-not-an-object",
                    f"{location}.derivations[{index}]",
                    f"got {type(item).__name__}",
                )
            )
            continue
        raw_evidence = item.get("evidence")
        if not isinstance(raw_evidence, dict):
            findings.append(
                error(
                    "derivation-evidence-not-an-object",
                    f"{location}.derivations[{index}].evidence",
                    f"got {type(raw_evidence).__name__}",
                )
            )
            raw_evidence = {}
        evidence = raw_evidence
        derivations.append(
            DerivationRow(
                claim_id=_str(item.get("claim_id")),
                path=_str(item.get("path")),
                locus=_str(item.get("locus")),
                path_existed_in_predecessor=item.get(
                    "path_existed_in_predecessor"
                )
                is True,
                locus_existed_in_predecessor=item.get("locus_existed_in_predecessor")
                is True,
                predecessor_behaviour=_str(item.get("predecessor_behaviour")),
                candidate_behaviour=_str(item.get("candidate_behaviour")),
                evidence_state=_str(evidence.get("state")),
                evidence_ref_ids=_tuple(evidence.get("ref_ids")),
                disposition=_str(item.get("disposition")),
                next_action=_str(item.get("next_action")),
            )
        )
    validations: list[ValidationRow] = []
    for index, item in enumerate(
        _array_items(raw, "validations", location, findings)
    ):
        if not isinstance(item, dict):
            findings.append(
                error(
                    "validation-not-an-object",
                    f"{location}.validations[{index}]",
                    f"got {type(item).__name__}",
                )
            )
            continue
        exit_code = item.get("exit_code")
        validations.append(
            ValidationRow(
                id=_str(item.get("id")),
                result_ref=_str(item.get("result_ref")),
                log_ref=_str(item.get("log_ref")),
                status=_str(item.get("status")),
                exit_code=exit_code
                if isinstance(exit_code, int) and not isinstance(exit_code, bool)
                else None,
                timed_out=item.get("timed_out") is True,
                log_complete=item.get("log_complete") is True,
            )
        )
    terminal_value = raw.get("terminalization")
    if not isinstance(terminal_value, dict):
        findings.append(
            error(
                "terminalization-not-an-object",
                f"{location}.terminalization",
                f"got {type(terminal_value).__name__}",
            )
        )
        terminal_value = {}
    terminal_raw = terminal_value
    record = RecordModel(
        control_id=_str(raw.get("control_id")),
        generation=_int(raw.get("generation")),
        predecessor_obligation_completeness=_str(
            raw.get("predecessor_obligation_completeness")
        ),
        review_decision=_str(raw.get("review_decision")),
        subject=subject,
        artifacts=tuple(artifacts),
        path_reviews=tuple(path_reviews),
        prior_obligations=tuple(obligations),
        new_findings=tuple(new_findings),
        derivations=tuple(derivations),
        validations=tuple(validations),
        terminalization=Terminalization(
            state=_str(terminal_raw.get("state")),
            worker_alive=terminal_raw.get("worker_alive") is not False,
            child_processes_active=_int(terminal_raw.get("child_processes_active")),
            event_evidence_ref=_str(terminal_raw.get("event_evidence_ref")),
        ),
    )
    repeated_derivations = duplicates([row.claim_id for row in record.derivations])
    if repeated_derivations:
        findings.append(
            error(
                "duplicate-record-derivation-claim-id",
                f"{location}.derivations",
                repr(repeated_derivations),
            )
        )
    repeated_validations = duplicates([row.id for row in record.validations])
    if repeated_validations:
        findings.append(
            error(
                "duplicate-record-validation-id",
                f"{location}.validations",
                repr(repeated_validations),
            )
        )
    return (None, findings) if findings else (record, [])


def parse_manifest(
    raw: Any,
    location: str,
) -> tuple[ManifestModel | None, list[Finding]]:
    findings: list[Finding] = []
    if not isinstance(raw, dict):
        return None, [error("manifest-not-an-object", location, type(raw).__name__)]
    if raw.get("schema") != MANIFEST_SCHEMA:
        findings.append(
            error(
                "wrong-manifest-schema",
                f"{location}.schema",
                f"expected {MANIFEST_SCHEMA}, got {raw.get('schema')!r}",
            )
        )
        return None, findings
    authority_value = raw.get("predecessor_authority")
    if not isinstance(authority_value, dict):
        findings.append(
            error(
                "manifest-authority-not-an-object",
                f"{location}.predecessor_authority",
                f"got {type(authority_value).__name__}",
            )
        )
        authority_value = {}
    authority = authority_value
    obligation_rows = _array_items(raw, "prior_obligations", location, findings)
    obligations_list: list[ManifestObligation] = []
    for index, item in enumerate(obligation_rows):
        if not isinstance(item, dict):
            findings.append(
                error(
                    "manifest-obligation-not-an-object",
                    f"{location}.prior_obligations[{index}]",
                    f"got {type(item).__name__}",
                )
            )
            continue
        obligations_list.append(
            ManifestObligation(
                obligation_id=_str(item.get("obligation_id")),
                origin_generation=_int(item.get("origin_generation")),
                closure_validation_ids=frozenset(
                    _tuple(item.get("closure_validation_ids"))
                ),
            )
        )
    obligations = tuple(obligations_list)
    claim_rows = _array_items(raw, "parity_closure_claims", location, findings)
    claims_list: list[ManifestClaim] = []
    for index, item in enumerate(claim_rows):
        if not isinstance(item, dict):
            findings.append(
                error(
                    "manifest-claim-not-an-object",
                    f"{location}.parity_closure_claims[{index}]",
                    f"got {type(item).__name__}",
                )
            )
            continue
        claims_list.append(
            ManifestClaim(
                claim_id=_str(item.get("claim_id")),
                kind=_str(item.get("kind")),
                obligation_id=_opt_str(item.get("obligation_id")),
                locus_path=_str(item.get("locus_path")),
            )
        )
    claims = tuple(claims_list)
    validation_rows = _array_items(raw, "required_validations", location, findings)
    validations_list: list[ManifestValidation] = []
    for index, item in enumerate(validation_rows):
        if not isinstance(item, dict):
            findings.append(
                error(
                    "manifest-validation-not-an-object",
                    f"{location}.required_validations[{index}]",
                    f"got {type(item).__name__}",
                )
            )
            continue
        validations_list.append(
            ManifestValidation(
                validation_id=_str(item.get("validation_id")),
                argv=_tuple(item.get("argv")),
            )
        )
    validations = tuple(validations_list)
    manifest = ManifestModel(
        manifest_id=_str(raw.get("manifest_id")),
        generation=_int(raw.get("generation")),
        predecessor_candidate_revision=_str(
            raw.get("predecessor_candidate_revision")
        ),
        authority_kind=_str(authority.get("kind")),
        authority_bootstrap_arm=_opt_str(authority.get("bootstrap_arm")),
        authority_commit=_str(authority.get("commit")),
        repair_packet_path=_str(raw.get("repair_packet_path")),
        repair_packet_sha256=_str(raw.get("repair_packet_sha256")),
        obligations=obligations,
        claims=claims,
        validations=validations,
    )
    for code, item_location, repeated in (
        (
            "duplicate-manifest-obligation-id",
            f"{location}.prior_obligations",
            duplicates([row.obligation_id for row in manifest.obligations]),
        ),
        (
            "duplicate-manifest-claim-id",
            f"{location}.parity_closure_claims",
            duplicates([row.claim_id for row in manifest.claims]),
        ),
        (
            "duplicate-manifest-validation-id",
            f"{location}.required_validations",
            duplicates([row.validation_id for row in manifest.validations]),
        ),
    ):
        if repeated:
            findings.append(error(code, item_location, repr(repeated)))
    return (None, findings) if findings else (manifest, [])


def duplicates(values: tuple[str, ...] | list[str]) -> list[str]:
    seen: set[str] = set()
    repeated: set[str] = set()
    for value in values:
        if value in seen:
            repeated.add(value)
        seen.add(value)
    return sorted(repeated)
