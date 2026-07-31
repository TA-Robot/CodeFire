#!/usr/bin/env python3
"""Parse layer and frozen vocabularies for the recurrence gate v2.

This module owns one responsibility: turn already-schema-checked JSON into
immutable typed values, and hold the vocabularies and manager-fixed tables the
other v2 modules read.  It never touches Git, the filesystem, or the decision.

The v1 record shape stays untouched in ``check_review_recurrence.py``; nothing
here reads or rewrites it.
"""

from __future__ import annotations

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
TIMED_SCHEMA_PATH = "docs/development/ai/schemas/timed-gate-result.v2.schema.json"

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


def path_components_safe(value: str) -> bool:
    """Reject absolute paths, backslashes and empty/dot/dot-dot components."""

    if not value or value.startswith("/") or "\\" in value:
        return False
    return all(
        part not in ("", ".", "..") for part in PurePosixPath(value).parts
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
    for index, item in enumerate(
        raw.get("artifacts") if isinstance(raw.get("artifacts"), list) else []
    ):
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
        raw.get("path_reviews") if isinstance(raw.get("path_reviews"), list) else []
    ):
        if not isinstance(item, dict):
            findings.append(
                error("path-review-not-an-object", f"{location}.path_reviews[{index}]", "")
            )
            continue
        surfaces: list[Surface] = []
        for surface in (
            item.get("surfaces") if isinstance(item.get("surfaces"), list) else []
        ):
            if not isinstance(surface, dict):
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
    for item in (
        raw.get("prior_obligations")
        if isinstance(raw.get("prior_obligations"), list)
        else []
    ):
        if not isinstance(item, dict):
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
    for item in (
        raw.get("new_findings") if isinstance(raw.get("new_findings"), list) else []
    ):
        if not isinstance(item, dict):
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
    for item in (
        raw.get("derivations") if isinstance(raw.get("derivations"), list) else []
    ):
        if not isinstance(item, dict):
            continue
        evidence = item.get("evidence") if isinstance(item.get("evidence"), dict) else {}
        derivations.append(
            DerivationRow(
                claim_id=_str(item.get("claim_id")),
                path=_str(item.get("path")),
                locus=_str(item.get("locus")),
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
    for item in (
        raw.get("validations") if isinstance(raw.get("validations"), list) else []
    ):
        if not isinstance(item, dict):
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
    terminal_raw = (
        raw.get("terminalization")
        if isinstance(raw.get("terminalization"), dict)
        else {}
    )
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
    return record, findings


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
    authority = (
        raw.get("predecessor_authority")
        if isinstance(raw.get("predecessor_authority"), dict)
        else {}
    )
    obligations = tuple(
        ManifestObligation(
            obligation_id=_str(item.get("obligation_id")),
            closure_validation_ids=frozenset(_tuple(item.get("closure_validation_ids"))),
        )
        for item in (
            raw.get("prior_obligations")
            if isinstance(raw.get("prior_obligations"), list)
            else []
        )
        if isinstance(item, dict)
    )
    claims = tuple(
        ManifestClaim(
            claim_id=_str(item.get("claim_id")),
            kind=_str(item.get("kind")),
            obligation_id=_opt_str(item.get("obligation_id")),
            locus_path=_str(item.get("locus_path")),
        )
        for item in (
            raw.get("parity_closure_claims")
            if isinstance(raw.get("parity_closure_claims"), list)
            else []
        )
        if isinstance(item, dict)
    )
    validations = tuple(
        ManifestValidation(
            validation_id=_str(item.get("validation_id")),
            argv=_tuple(item.get("argv")),
        )
        for item in (
            raw.get("required_validations")
            if isinstance(raw.get("required_validations"), list)
            else []
        )
        if isinstance(item, dict)
    )
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
    return manifest, findings


def duplicates(values: tuple[str, ...] | list[str]) -> list[str]:
    seen: set[str] = set()
    repeated: set[str] = set()
    for value in values:
        if value in seen:
            repeated.add(value)
        seen.add(value)
    return sorted(repeated)
