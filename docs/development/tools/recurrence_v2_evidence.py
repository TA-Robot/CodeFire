#!/usr/bin/env python3
"""Evidence binding for the recurrence gate v2.

One responsibility: decide whether every declared artifact really is the blob it
claims to be, in a commit its role permits, and whether the frozen pre-repair
manifest and the committed timed-gate v2 results agree with the record.

There is no global "reachable from any ref" predicate.  A role is resolved
against a commit the topology or the rooted chain already proved, or against
candidate ancestry; a local branch or ref pointing at an object is never
evidence, and a role-incompatible commit fails even when `git cat-file` can read
it.
"""

from __future__ import annotations

import math
import re
from dataclasses import dataclass
from datetime import datetime, timezone
from decimal import Decimal
from pathlib import Path, PurePosixPath
from typing import Any

from ai_contracts import validate_json_schema
from recurrence_v2_authority import ChainResult
from recurrence_v2_git import GitError, GitRepo, sha256_bytes
from recurrence_v2_model import (
    AUTHORITY_SCHEMA_IDS,
    BLOB_MODE,
    LEGACY_ROOTS,
    LIVE_ARM_ANCHOR_SCHEMA_PATH,
    LIVE_ARM_ANCHOR_PATH,
    LIVE_ARM_ANCHOR_SCHEMA,
    MANIFEST_SCHEMA_PATH,
    OBSERVATION_ROLES,
    PORT_PACKETS,
    PORT_REQUIRED_REPAIR_PATHS,
    RECORD_SCHEMA_PATH,
    RESULT_SCHEMA_PATH,
    TIMED_SCHEMA,
    TIMED_SCHEMA_PATH,
    ArtifactRef,
    Finding,
    ManifestModel,
    RecordModel,
    duplicates,
    error,
    parse_manifest,
    path_components_safe,
    strict_decode_json,
    validate_schema_definition_identity,
)

GATE_SOURCE_PATHS = (
    "docs/development/tools/check_review_recurrence.py",
    "docs/development/tools/run_timed_gate.py",
    "docs/development/tools/recurrence_v2_model.py",
    "docs/development/tools/recurrence_v2_git.py",
    "docs/development/tools/recurrence_v2_topology.py",
    "docs/development/tools/recurrence_v2_authority.py",
    "docs/development/tools/recurrence_v2_evidence.py",
    "docs/development/tools/recurrence_v2_decision.py",
    "docs/development/tools/recurrence_v2_history.py",
    RECORD_SCHEMA_PATH,
    MANIFEST_SCHEMA_PATH,
    RESULT_SCHEMA_PATH,
    LIVE_ARM_ANCHOR_SCHEMA_PATH,
    TIMED_SCHEMA_PATH,
)


@dataclass(frozen=True)
class TimedFacts:
    validation_id: str
    result_revision: str
    result_tree: str
    status: str
    exit_code: int | None
    timed_out: bool
    log_complete: bool
    clean_before: bool
    clean_after: bool
    index_flags_clean: bool
    head_unchanged: bool
    log_inside_source_worktree: bool
    result_inside_source_worktree: bool
    log_bytes: int
    log_sha256: str

    def as_result(self) -> dict[str, Any]:
        """Render the exact closed timed-validation row retained by v2 results."""

        return {
            "result_revision": self.result_revision,
            "result_tree": self.result_tree,
            "status": self.status,
            "exit_code": self.exit_code,
            "timed_out": self.timed_out,
            "log_complete": self.log_complete,
            "clean_before": self.clean_before,
            "clean_after": self.clean_after,
            "index_flags_clean": self.index_flags_clean,
            "head_unchanged": self.head_unchanged,
            "log_inside_source_worktree": self.log_inside_source_worktree,
            "result_inside_source_worktree": self.result_inside_source_worktree,
            "log_bytes": self.log_bytes,
            "log_sha256": self.log_sha256,
            "passing": self.passing,
        }

    @property
    def passing(self) -> bool:
        return (
            self.status == "passed"
            and self.exit_code == 0
            and not self.timed_out
            and self.log_complete
            and self.clean_before
            and self.clean_after
            and self.index_flags_clean
            and self.head_unchanged
            and not self.log_inside_source_worktree
            and not self.result_inside_source_worktree
        )


_UTC_MICROSECOND = re.compile(
    r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:"
    r"[0-9]{2}\.[0-9]{6}Z$"
)
_TIMESTAMP_WALL_TOLERANCE_SECONDS = Decimal("0.000002")
_WALL_MONOTONIC_TOLERANCE_SECONDS = Decimal("0.000000000001")


def _finite_number(value: Any) -> bool:
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        return False
    try:
        return math.isfinite(value)
    except OverflowError:
        return False


def _parse_utc_microsecond(value: Any) -> datetime | None:
    if not isinstance(value, str) or _UTC_MICROSECOND.fullmatch(value) is None:
        return None
    try:
        parsed = datetime.strptime(value, "%Y-%m-%dT%H:%M:%S.%fZ")
    except ValueError:
        return None
    return parsed.replace(tzinfo=timezone.utc)


def _decimal_number(value: int | float) -> Decimal:
    return Decimal(str(value))


def _timedelta_seconds(value: Any) -> Decimal:
    microseconds = (
        (value.days * 86_400 + value.seconds) * 1_000_000 + value.microseconds
    )
    return Decimal(microseconds) / Decimal(1_000_000)


def _lexical_absolute_posix_parts(value: Any) -> tuple[str, ...] | None:
    """Return canonical absolute POSIX components without touching the host FS."""

    if not isinstance(value, str) or not value.startswith("/"):
        return None
    if any(
        character == "\\"
        or 0xD800 <= ord(character) <= 0xDFFF
        or ord(character) < 0x20
        or 0x7F <= ord(character) <= 0x9F
        for character in value
    ):
        return None
    if value == "/":
        return ()
    if value.endswith("/"):
        return None
    parts = tuple(value[1:].split("/"))
    if any(part in {"", ".", ".."} for part in parts):
        return None
    if PurePosixPath(value).as_posix() != value:
        return None
    return parts


def _component_contains(root: tuple[str, ...], candidate: tuple[str, ...]) -> bool:
    return len(candidate) >= len(root) and candidate[: len(root)] == root


def predecessor_path_presence(
    repo: GitRepo,
    record: RecordModel,
    location: str = "record.derivations",
) -> tuple[dict[str, bool], list[Finding]]:
    """Read one canonical predecessor tree-entry fact for both gate and replay."""

    present: dict[str, bool] = {}
    findings: list[Finding] = []
    for index, derivation in enumerate(record.derivations):
        item_location = f"{location}[{index}].path"
        if not path_components_safe(derivation.path):
            findings.append(
                error(
                    "unsafe-derivation-path",
                    item_location,
                    repr(derivation.path),
                )
            )
            continue
        if derivation.path in present:
            continue
        try:
            entry = repo.entry(
                record.subject.predecessor_candidate_revision,
                derivation.path.encode("utf-8"),
            )
        except (GitError, UnicodeEncodeError) as exc:
            findings.append(
                error(
                    "predecessor-path-probe-failed",
                    item_location,
                    str(exc),
                )
            )
            continue
        present[derivation.path] = entry is not None
    return present, findings


def _permitted_commits(
    record: RecordModel,
    chain: ChainResult,
    role: str,
    legacy_roots: dict[str, Any],
) -> tuple[frozenset[str] | None, str]:
    """Return the commits a role may resolve at, or ``None`` for a rule check."""

    subject = record.subject
    fixed = {
        "repair_packet": frozenset({subject.repair_packet_commit}),
        "repair_manifest": frozenset({subject.repair_packet_commit}),
        "candidate_artifact": frozenset({subject.candidate_revision}),
        "review_packet": frozenset({subject.review_control_commit}),
        "report": frozenset({subject.report_commit}),
        "timed_result": frozenset({subject.evidence_commit}),
        "timed_log": frozenset({subject.evidence_commit}),
        "terminal_event_evidence": frozenset({subject.evidence_commit}),
        "prior_v2_record": frozenset({subject.authority.commit}),
        "prior_v2_result": frozenset({subject.authority.commit}),
    }
    if role in fixed:
        return fixed[role], "declared topology commit"
    if role == "legacy_bootstrap_report":
        arm = subject.authority.bootstrap_arm
        root = legacy_roots.get(arm) if isinstance(arm, str) else None
        return (
            frozenset({root.commit}) if root is not None else frozenset(),
            "manager-fixed legacy root",
        )
    return None, "candidate ancestry or a verified earlier generation"


def _own_review_commits(record: RecordModel) -> frozenset[str]:
    return frozenset(
        {
            record.subject.review_control_commit,
            record.subject.report_commit,
            record.subject.evidence_commit,
        }
    )


def _check_ancestry_role(
    repo: GitRepo,
    record: RecordModel,
    chain: ChainResult,
    artifact: ArtifactRef,
    location: str,
    findings: list[Finding],
) -> None:
    """Routing, prior-closure and non-reachability roles.

    Routing/scope artifacts resolve at or before the candidate on its ancestry.
    Prior-closure and non-reachability evidence may additionally resolve inside
    the fully verified report/evidence/retention topology of an earlier
    generation whose chain roots at a manager-fixed legacy root.  None of the
    three may resolve inside this generation's own review-control, report or
    evidence commits, so the boundary that `CLOSED_PROVEN` depends on can never
    be self-authored by the reviewing lane.
    """

    if artifact.commit in _own_review_commits(record):
        findings.append(
            error(
                "artifact-inside-own-review-commit",
                location,
                f"{artifact.role} may not resolve inside this generation's "
                f"review-control, report or evidence commit ({artifact.commit})",
            )
        )
        return
    on_ancestry = artifact.commit == record.subject.candidate_revision or repo.is_ancestor(
        artifact.commit, record.subject.candidate_revision
    )
    if on_ancestry:
        return
    if artifact.role in {"prior_closure_evidence", "non_reachability_evidence"}:
        if chain.rooted and artifact.commit in chain.verified_commits:
            return
        findings.append(
            error(
                "artifact-not-on-ancestry-or-verified-generation",
                location,
                f"{artifact.commit} is neither at/before the candidate on its ancestry nor "
                "inside a verified earlier generation of a rooted authority chain",
            )
        )
        return
    findings.append(
        error(
            "artifact-not-on-candidate-ancestry",
            location,
            f"{artifact.commit} is not the candidate or an ancestor of it",
        )
    )


def resolve_artifacts(
    repo: GitRepo,
    record: RecordModel,
    chain: ChainResult,
    legacy_roots: dict[str, Any] | None = None,
    location: str = "record.artifacts",
) -> tuple[dict[str, ArtifactRef], list[Finding]]:
    legacy_roots = LEGACY_ROOTS if legacy_roots is None else legacy_roots
    findings: list[Finding] = []
    resolved: dict[str, ArtifactRef] = {}
    repeated = duplicates([artifact.ref_id for artifact in record.artifacts])
    if repeated:
        findings.append(error("duplicate-artifact-ref-id", location, repr(repeated)))
    for index, artifact in enumerate(record.artifacts):
        row = f"{location}[{index}]"
        resolved.setdefault(artifact.ref_id, artifact)
        if artifact.mode != BLOB_MODE:
            findings.append(
                error("artifact-mode-not-regular-file", f"{row}.mode", artifact.mode)
            )
            continue
        if not path_components_safe(artifact.path):
            findings.append(
                error("unsafe-artifact-path", f"{row}.path", repr(artifact.path))
            )
            continue
        permitted, description = _permitted_commits(
            record, chain, artifact.role, legacy_roots
        )
        if permitted is None:
            _check_ancestry_role(repo, record, chain, artifact, row, findings)
        elif artifact.commit not in permitted:
            findings.append(
                error(
                    "artifact-role-commit-mismatch",
                    f"{row}.commit",
                    f"role {artifact.role} resolves only at {sorted(permitted)} "
                    f"({description}), not {artifact.commit}",
                )
            )
            continue
        try:
            entry = repo.entry(artifact.commit, artifact.path.encode("utf-8"))
            payload = repo.blob(artifact.commit, artifact.path.encode("utf-8"))
        except GitError as exc:
            findings.append(error("artifact-unreadable", row, str(exc)))
            continue
        if entry is None or entry.kind != "blob" or entry.mode != BLOB_MODE:
            findings.append(
                error(
                    "artifact-not-a-regular-blob",
                    row,
                    f"{artifact.path} at {artifact.commit} is {entry!r}; symlinks, trees, "
                    "gitlinks and executable blobs are rejected",
                )
            )
            continue
        digest = sha256_bytes(payload)
        if digest != artifact.sha256:
            findings.append(
                error("artifact-digest-mismatch", f"{row}.sha256", f"computed {digest}")
            )
    if record.subject.authority.kind == "prior_v2_record" and not chain.rooted:
        findings.append(
            error(
                "prior-authority-not-rooted",
                "record.subject.predecessor_authority",
                "a prior v2 record is admissible only through a chain that terminates at a "
                "manager-fixed legacy root",
            )
        )
    return resolved, findings


def check_reference_integrity(
    record: RecordModel,
    artifacts: dict[str, ArtifactRef],
    location: str = "record",
) -> list[Finding]:
    """Every reference resolves, has a compatible role, and nothing is dead."""

    findings: list[Finding] = []
    used: set[str] = set()

    def require(ref: str | None, roles: frozenset[str], where: str) -> None:
        if ref is None:
            return
        used.add(ref)
        artifact = artifacts.get(ref)
        if artifact is None:
            findings.append(
                error("dangling-artifact-ref", where, f"{ref} is not in the artifacts table")
            )
            return
        if artifact.role not in roles:
            findings.append(
                error(
                    "role-incompatible-artifact-ref",
                    where,
                    f"{ref} has role {artifact.role}; {sorted(roles)} is required here",
                )
            )

    subject = record.subject
    require(subject.repair_packet_ref, frozenset({"repair_packet"}), "record.subject.repair_packet_ref")
    require(
        subject.repair_manifest_ref,
        frozenset({"repair_manifest"}),
        "record.subject.repair_manifest_ref",
    )
    require(subject.review_packet_ref, frozenset({"review_packet"}), "record.subject.review_packet_ref")
    require(subject.report_ref, frozenset({"report"}), "record.subject.report_ref")
    authority_roles = (
        frozenset({"legacy_bootstrap_report"})
        if subject.authority.kind == "legacy_bootstrap_report"
        else frozenset({"prior_v2_record", "prior_v2_result"})
    )
    for ref in subject.authority.artifact_refs:
        require(ref, authority_roles, "record.subject.predecessor_authority.artifact_refs")
    declared_authority_roles = {
        artifacts[ref].role for ref in subject.authority.artifact_refs if ref in artifacts
    }
    if declared_authority_roles and declared_authority_roles != set(authority_roles):
        findings.append(
            error(
                "authority-artifact-role-set-mismatch",
                "record.subject.predecessor_authority.artifact_refs",
                f"declared {sorted(declared_authority_roles)}, required {sorted(authority_roles)}",
            )
        )
    for index, review in enumerate(record.path_reviews):
        for surface_index, surface in enumerate(review.surfaces):
            where = f"record.path_reviews[{index}].surfaces[{surface_index}]"
            for ref in surface.evidence_ref_ids:
                require(ref, OBSERVATION_ROLES, f"{where}.evidence_ref_ids")
            require(
                surface.scope_routing_ref,
                frozenset({"routing_scope_decision"}),
                f"{where}.scope_routing_ref",
            )
    for where, row in record.rows():
        for ref in row.observations.before_ref_ids:
            require(ref, OBSERVATION_ROLES, f"{where}.observations.before_ref_ids")
        for ref in row.observations.after_ref_ids:
            require(ref, OBSERVATION_ROLES, f"{where}.observations.after_ref_ids")
        for ref in row.observations.negative_control_ref_ids:
            require(
                ref,
                OBSERVATION_ROLES,
                f"{where}.observations.negative_control_ref_ids",
            )
        require(
            row.routing_decision_ref,
            frozenset({"routing_scope_decision"}),
            f"{where}.routing_decision_ref",
        )
        require(
            getattr(row, "non_reachability_ref", None),
            frozenset({"non_reachability_evidence"}),
            f"{where}.non_reachability_ref",
        )
        require(
            getattr(row, "closure_evidence_ref", None),
            frozenset({"prior_closure_evidence"}),
            f"{where}.closure_evidence_ref",
        )
    for index, derivation in enumerate(record.derivations):
        for ref in derivation.evidence_ref_ids:
            require(
                ref,
                OBSERVATION_ROLES,
                f"record.derivations[{index}].evidence.ref_ids",
            )
    for index, validation in enumerate(record.validations):
        require(
            validation.result_ref,
            frozenset({"timed_result"}),
            f"record.validations[{index}].result_ref",
        )
        require(
            validation.log_ref,
            frozenset({"timed_log"}),
            f"record.validations[{index}].log_ref",
        )
    require(
        record.terminalization.event_evidence_ref,
        frozenset({"terminal_event_evidence"}),
        "record.terminalization.event_evidence_ref",
    )
    unused = sorted(set(artifacts) - used)
    if unused:
        findings.append(
            error(
                "unreferenced-artifact",
                f"{location}.artifacts",
                f"{unused} are bound but never used; every artifact must have a consumer",
            )
        )
    return findings


def verify_canonical_port_packet(
    repo: GitRepo,
    record: RecordModel,
    artifacts: dict[str, ArtifactRef],
    arm: str | None,
    location: str = "record.subject.repair_packet_ref",
) -> list[Finding]:
    """Bind a bootstrap port to the contract-fixed historical packet bytes."""

    if record.subject.authority.kind != "legacy_bootstrap_report":
        return []
    port = PORT_PACKETS.get(arm) if arm is not None else None
    if port is None:
        return [error("canonical-port-arm-unknown", location, repr(arm))]
    findings: list[Finding] = []
    packet = artifacts.get(record.subject.repair_packet_ref)
    if packet is None:
        return [error("canonical-port-packet-unresolved", location, port.path)]
    if packet.path != port.path or packet.sha256 != port.sha256:
        findings.append(
            error(
                "canonical-port-packet-identity-mismatch",
                location,
                f"{arm} requires {port.path} at digest {port.sha256}",
            )
        )
    try:
        entry = repo.entry(port.source_commit, port.path.encode("utf-8"))
        source = repo.blob(port.source_commit, port.path.encode("utf-8"))
    except GitError as exc:
        findings.append(error("canonical-port-source-unreadable", location, str(exc)))
    else:
        if entry is None or entry.kind != "blob" or entry.mode != BLOB_MODE:
            findings.append(
                error(
                    "canonical-port-source-not-regular-blob",
                    location,
                    repr(entry),
                )
            )
        if sha256_bytes(source) != port.sha256:
            findings.append(
                error(
                    "canonical-port-source-digest-mismatch",
                    location,
                    f"computed {sha256_bytes(source)}",
                )
            )
    try:
        predecessor_entry = repo.entry(port.absent_at, port.path.encode("utf-8"))
    except GitError as exc:
        findings.append(error("canonical-port-predecessor-unreadable", location, str(exc)))
    else:
        if predecessor_entry is not None:
            findings.append(
                error(
                    "canonical-port-packet-not-absent-at-predecessor",
                    location,
                    f"{port.path} exists at {port.absent_at}",
                )
            )
    required_paths = PORT_REQUIRED_REPAIR_PATHS.get(arm, frozenset())
    missing_paths = sorted(required_paths - record.subject.changed_paths)
    if missing_paths:
        findings.append(
            error(
                "canonical-port-required-repair-paths-missing",
                "record.subject.changed_paths",
                repr(missing_paths),
            )
        )
    return findings


def load_manifest(
    repo: GitRepo,
    record: RecordModel,
    artifacts: dict[str, ArtifactRef],
    chain: ChainResult,
    manifest_schema: dict[str, Any] | None,
    location: str = "manifest",
) -> tuple[ManifestModel | None, list[Finding]]:
    """Read the frozen manifest from the packet commit and prove set equality."""

    findings: list[Finding] = []
    manifest_artifact = artifacts.get(record.subject.repair_manifest_ref)
    packet_artifact = artifacts.get(record.subject.repair_packet_ref)
    if manifest_artifact is None or packet_artifact is None:
        return None, [
            error(
                "manifest-artifact-unresolved",
                location,
                "the record does not bind both packet artifacts",
            )
        ]
    if manifest_schema is None:
        findings.append(
            error(
                "manifest-schema-unavailable",
                location,
                "the registered repair-manifest schema is required before typed use",
            )
        )
        return None, findings
    definition_findings = validate_schema_definition_identity(
        manifest_schema,
        AUTHORITY_SCHEMA_IDS[MANIFEST_SCHEMA_PATH],
        f"{location}.schema_definition",
    )
    findings.extend(definition_findings)
    if definition_findings:
        return None, findings
    try:
        payload = repo.blob(
            manifest_artifact.commit, manifest_artifact.path.encode("utf-8")
        )
    except GitError as exc:
        return None, [error("manifest-unreadable", location, str(exc))]
    raw, decode_findings = strict_decode_json(payload, location)
    findings.extend(decode_findings)
    if raw is None or decode_findings:
        return None, findings
    schema_findings = validate_json_schema(raw, manifest_schema, location)
    findings.extend(schema_findings)
    if schema_findings:
        return None, findings
    manifest, parse_findings = parse_manifest(raw, location)
    findings.extend(parse_findings)
    if manifest is None or parse_findings:
        return None, findings
    if manifest.generation != record.generation:
        findings.append(
            error(
                "manifest-generation-mismatch",
                f"{location}.generation",
                f"manifest {manifest.generation}, record {record.generation}",
            )
        )
    if (
        manifest.predecessor_candidate_revision
        != record.subject.predecessor_candidate_revision
    ):
        findings.append(
            error(
                "manifest-predecessor-mismatch",
                f"{location}.predecessor_candidate_revision",
                manifest.predecessor_candidate_revision,
            )
        )
    authority = record.subject.authority
    if (
        manifest.authority_kind != authority.kind
        or manifest.authority_commit != authority.commit
        or manifest.authority_bootstrap_arm != authority.bootstrap_arm
    ):
        findings.append(
            error(
                "manifest-authority-mismatch",
                f"{location}.predecessor_authority",
                f"manifest binds {manifest.authority_kind}/{manifest.authority_bootstrap_arm}/"
                f"{manifest.authority_commit}",
            )
        )
    if manifest.repair_packet_path != packet_artifact.path:
        findings.append(
            error(
                "manifest-packet-path-mismatch",
                f"{location}.repair_packet_path",
                f"record binds {packet_artifact.path}",
            )
        )
    if manifest.repair_packet_sha256 != packet_artifact.sha256:
        findings.append(
            error(
                "manifest-packet-digest-mismatch",
                f"{location}.repair_packet_sha256",
                f"record binds {packet_artifact.sha256}",
            )
        )
    obligation_map = manifest.obligation_map()
    manifest_origins = manifest.obligation_origin_map()
    manifest_ids = frozenset(obligation_map)
    record_ids = frozenset(row.id for row in record.prior_obligations)
    if manifest_ids != record_ids:
        findings.append(
            error(
                "prior-obligation-set-mismatch",
                "record.prior_obligations",
                f"manifest-only={sorted(manifest_ids - record_ids)}, "
                f"record-only={sorted(record_ids - manifest_ids)}",
            )
        )
    repeated_record_ids = duplicates([row.id for row in record.prior_obligations])
    record_origins: dict[str, int] = {}
    if repeated_record_ids:
        findings.append(
            error(
                "duplicate-prior-obligation-id",
                "record.prior_obligations",
                repr(repeated_record_ids),
            )
        )
    else:
        record_origins = {
            row.id: row.origin_generation for row in record.prior_obligations
        }
        for obligation_id, manifest_origin in sorted(manifest_origins.items()):
            record_origin = record_origins.get(obligation_id)
            if record_origin is not None and record_origin != manifest_origin:
                findings.append(
                    error(
                        "manifest-record-origin-generation-mismatch",
                        f"{location}.prior_obligations",
                        f"{obligation_id} has manifest origin {manifest_origin}, "
                        f"record origin {record_origin}",
                    )
                )
            if manifest_origin >= manifest.generation:
                findings.append(
                    error(
                        "manifest-obligation-origin-not-prior",
                        f"{location}.prior_obligations",
                        f"{obligation_id} origin {manifest_origin} is not before "
                        f"generation {manifest.generation}",
                    )
                )
    if record.subject.authority.kind == "prior_v2_record":
        predecessor = chain.predecessor_link
        predecessor_origins = (
            predecessor.obligation_origins if predecessor is not None else {}
        )
        for obligation_id, manifest_origin in sorted(manifest_origins.items()):
            predecessor_origin = predecessor_origins.get(obligation_id)
            if predecessor_origin is None:
                findings.append(
                    error(
                        "manifest-predecessor-origin-unavailable",
                        f"{location}.prior_obligations",
                        f"{obligation_id} has no exported origin in the verified predecessor",
                    )
                )
            elif predecessor_origin != manifest_origin:
                findings.append(
                    error(
                        "manifest-predecessor-origin-generation-mismatch",
                        f"{location}.prior_obligations",
                        f"{obligation_id} has manifest origin {manifest_origin}, "
                        f"predecessor origin {predecessor_origin}",
                    )
                )
    elif record.subject.authority.kind == "legacy_bootstrap_report":
        arm = record.subject.authority.bootstrap_arm
        fixed_root = LEGACY_ROOTS.get(arm) if isinstance(arm, str) else None
        expected_bootstrap_origins = record_origins
        if (
            fixed_root is not None
            and record.subject.authority.commit == fixed_root.commit
        ):
            if record.generation != fixed_root.review_generation:
                findings.append(
                    error(
                        "bootstrap-review-generation-mismatch",
                        "record.generation",
                        f"{arm} requires first live review generation "
                        f"{fixed_root.review_generation}, got {record.generation}",
                    )
                )
            expected_bootstrap_origins = fixed_root.obligation_origin_map()
        if manifest_origins != expected_bootstrap_origins:
            findings.append(
                error(
                    "bootstrap-manifest-origin-map-mismatch",
                    f"{location}.prior_obligations",
                    f"{arm} requires {sorted(expected_bootstrap_origins.items())}, got "
                    f"{sorted(manifest_origins.items())}",
                )
            )
    validation_argv = manifest.validation_argv()
    manifest_validations = frozenset(validation_argv)
    record_validations = frozenset(row.id for row in record.validations)
    if manifest_validations != record_validations:
        findings.append(
            error(
                "required-validation-set-mismatch",
                "record.validations",
                f"manifest-only={sorted(manifest_validations - record_validations)}, "
                f"record-only={sorted(record_validations - manifest_validations)}",
            )
        )
    claim_ids = frozenset(claim.claim_id for claim in manifest.claims)
    derivation_ids = frozenset(row.claim_id for row in record.derivations)
    if claim_ids != derivation_ids:
        findings.append(
            error(
                "derivation-claim-set-mismatch",
                "record.derivations",
                f"manifest-only={sorted(claim_ids - derivation_ids)}, "
                f"record-only={sorted(derivation_ids - claim_ids)}",
            )
        )
    for claim in manifest.claims:
        if claim.obligation_id is not None and claim.obligation_id not in manifest_ids:
            findings.append(
                error(
                    "manifest-claim-obligation-unknown",
                    f"{location}.parity_closure_claims",
                    f"{claim.claim_id} maps to unknown obligation {claim.obligation_id}",
                )
            )
    for obligation_id, mapped in sorted(obligation_map.items()):
        unknown = sorted(mapped - manifest_validations)
        if unknown:
            findings.append(
                error(
                    "closure-validation-not-required",
                    f"{location}.prior_obligations",
                    f"{obligation_id} maps to {unknown}, which are not in required_validations",
                )
            )
        if not mapped:
            predecessor = chain.predecessor_link
            predecessor_state = (
                predecessor.obligation_states.get(obligation_id)
                if predecessor is not None
                else None
            )
            if predecessor_state != "DEFECT_INJECTION_ONLY":
                code = (
                    "bootstrap-manifest-empty-closure-mapping"
                    if record.subject.authority.kind == "legacy_bootstrap_report"
                    else "manifest-empty-closure-mapping-without-injection-predecessor"
                )
                findings.append(
                    error(
                        code,
                        f"{location}.prior_obligations",
                        f"{obligation_id} has no closure validation and the verified "
                        f"predecessor state is {predecessor_state!r}; only a rooted "
                        "DEFECT_INJECTION_ONLY predecessor row can authorize an empty mapping",
                    )
                )
    return manifest, findings


def _timed_semantics(
    payload: dict[str, Any],
    location: str,
) -> tuple[dict[str, bool], list[Finding]]:
    """Recompute every runner summary and validate its frozen relations."""

    findings: list[Finding] = []
    source = payload["source"]
    worktree = payload["worktree"]
    log = payload["log"]
    result = payload["result"]

    clean_before = worktree["status_before"] == []
    clean_after = worktree["status_after"] == []
    index_flags_clean = worktree["index_flagged_entries"] == []
    for field, actual, expected in (
        ("clean_before", worktree["clean_before"], clean_before),
        ("clean_after", worktree["clean_after"], clean_after),
        ("index_flags_clean", worktree["index_flags_clean"], index_flags_clean),
    ):
        if actual != expected:
            findings.append(
                error(
                    f"timed-{field.replace('_', '-')}-inconsistent",
                    f"{location}.worktree.{field}",
                    f"declared {actual!r}, recomputed {expected!r}",
                )
            )

    revision_after = worktree["head_revision_after"]
    tree_after = worktree["head_tree_after"]
    probe_error = worktree["head_probe_error"]
    probe_shape_valid = (
        (probe_error is None and revision_after is not None and tree_after is not None)
        or (probe_error is not None and revision_after is None and tree_after is None)
    )
    if not probe_shape_valid:
        findings.append(
            error(
                "timed-head-probe-shape-inconsistent",
                f"{location}.worktree",
                "head_probe_error is null iff both post-command identities are present; "
                "an error requires both identities to be null",
            )
        )
    head_unchanged = (
        probe_error is None
        and revision_after == source["revision"]
        and tree_after == source["tree"]
    )
    if worktree["head_unchanged"] != head_unchanged:
        findings.append(
            error(
                "timed-head-unchanged-inconsistent",
                f"{location}.worktree.head_unchanged",
                f"declared {worktree['head_unchanged']!r}, recomputed {head_unchanged!r}",
            )
        )

    path_values = {
        "cwd": payload["cwd"],
        "source.git_root": source["git_root"],
        "log.path": log["path"],
        "result.path": result["path"],
    }
    path_parts: dict[str, tuple[str, ...]] = {}
    for field, value in path_values.items():
        parts = _lexical_absolute_posix_parts(value)
        if parts is None:
            findings.append(
                error(
                    "timed-path-not-canonical",
                    f"{location}.{field}",
                    f"expected one lexical absolute POSIX path, got {value!r}",
                )
            )
        else:
            path_parts[field] = parts

    if {"cwd", "source.git_root"} <= path_parts.keys() and not _component_contains(
        path_parts["source.git_root"], path_parts["cwd"]
    ):
        findings.append(
            error(
                "timed-cwd-outside-source-root",
                f"{location}.cwd",
                f"{payload['cwd']!r} is not within {source['git_root']!r}",
            )
        )

    log_inside = False
    if {"log.path", "source.git_root"} <= path_parts.keys():
        log_inside = _component_contains(
            path_parts["source.git_root"], path_parts["log.path"]
        )
        if worktree["log_inside_source_worktree"] != log_inside:
            findings.append(
                error(
                    "timed-log-placement-inconsistent",
                    f"{location}.worktree.log_inside_source_worktree",
                    f"declared {worktree['log_inside_source_worktree']!r}, "
                    f"recomputed {log_inside!r}",
                )
            )
    result_inside = False
    if {"result.path", "source.git_root"} <= path_parts.keys():
        result_inside = _component_contains(
            path_parts["source.git_root"], path_parts["result.path"]
        )
        if worktree["result_inside_source_worktree"] != result_inside:
            findings.append(
                error(
                    "timed-result-placement-inconsistent",
                    f"{location}.worktree.result_inside_source_worktree",
                    f"declared {worktree['result_inside_source_worktree']!r}, "
                    f"recomputed {result_inside!r}",
                )
            )
    if (
        "log.path" in path_parts
        and "result.path" in path_parts
        and path_parts["log.path"] == path_parts["result.path"]
    ):
        findings.append(
            error(
                "timed-log-result-path-collision",
                f"{location}.result.path",
                "log.path and result.path must be distinct",
            )
        )

    timeout = payload["timeout_seconds"]
    duration = payload["duration_seconds"]
    wall_elapsed = payload["wall_elapsed_seconds"]
    wall_minus = payload["wall_minus_monotonic_seconds"]
    threshold = payload["drift_threshold_seconds"]
    for field, value, positive in (
        ("timeout_seconds", timeout, True),
        ("duration_seconds", duration, False),
        ("drift_threshold_seconds", threshold, False),
    ):
        valid = _finite_number(value) and (value > 0 if positive else value >= 0)
        if not valid:
            findings.append(
                error(
                    "timed-timing-value-invalid",
                    f"{location}.{field}",
                    f"{field} must be finite and {'positive' if positive else 'nonnegative'}",
                )
            )
    for field, value in (
        ("wall_elapsed_seconds", wall_elapsed),
        ("wall_minus_monotonic_seconds", wall_minus),
    ):
        if not _finite_number(value):
            findings.append(
                error(
                    "timed-timing-value-invalid",
                    f"{location}.{field}",
                    f"{field} must be finite",
                )
            )

    started = _parse_utc_microsecond(payload["started_at"])
    ended = _parse_utc_microsecond(payload["ended_at"])
    if started is None:
        findings.append(
            error(
                "timed-timestamp-invalid",
                f"{location}.started_at",
                "expected YYYY-MM-DDTHH:MM:SS.ffffffZ with a valid UTC datetime",
            )
        )
    if ended is None:
        findings.append(
            error(
                "timed-timestamp-invalid",
                f"{location}.ended_at",
                "expected YYYY-MM-DDTHH:MM:SS.ffffffZ with a valid UTC datetime",
            )
        )
    if started is not None and ended is not None and _finite_number(wall_elapsed):
        timestamp_elapsed = _timedelta_seconds(ended - started)
        wall_elapsed_decimal = _decimal_number(wall_elapsed)
        if (
            abs(timestamp_elapsed - wall_elapsed_decimal)
            > _TIMESTAMP_WALL_TOLERANCE_SECONDS
        ):
            findings.append(
                error(
                    "timed-wall-elapsed-inconsistent",
                    f"{location}.wall_elapsed_seconds",
                    f"timestamp delta {timestamp_elapsed!r} differs from "
                    f"{wall_elapsed!r} beyond {_TIMESTAMP_WALL_TOLERANCE_SECONDS!r}",
                )
            )
    recomputed_wall_minus: float | None = None
    if _finite_number(wall_elapsed) and _finite_number(duration):
        recomputed_wall_minus = wall_elapsed - duration
        within_wall_minus_tolerance = False
        if _finite_number(wall_minus):
            within_wall_minus_tolerance = (
                wall_minus == recomputed_wall_minus
                or abs(
                    _decimal_number(wall_minus)
                    - _decimal_number(recomputed_wall_minus)
                )
                <= _WALL_MONOTONIC_TOLERANCE_SECONDS
            )
        if (
            not _finite_number(wall_minus)
            or not within_wall_minus_tolerance
        ):
            findings.append(
                error(
                    "timed-wall-minus-monotonic-inconsistent",
                    f"{location}.wall_minus_monotonic_seconds",
                    f"declared {wall_minus!r}, recomputed {recomputed_wall_minus!r}",
                )
            )
    if recomputed_wall_minus is not None and _finite_number(threshold):
        discontinuity = abs(recomputed_wall_minus) > threshold
        if payload["wall_clock_discontinuity"] != discontinuity:
            findings.append(
                error(
                    "timed-wall-clock-discontinuity-inconsistent",
                    f"{location}.wall_clock_discontinuity",
                    f"declared {payload['wall_clock_discontinuity']!r}, "
                    f"recomputed {discontinuity!r}",
                )
            )

    passed = payload["exit_code"] == 0 and payload["timed_out"] is False
    if (payload["status"] == "passed") != passed:
        findings.append(
            error(
                "timed-terminal-status-inconsistent",
                f"{location}.status",
                "status is passed iff exit_code is zero and timed_out is false",
            )
        )
    if payload["timed_out"] is True and payload["exit_code"] != 124:
        findings.append(
            error(
                "timed-timeout-exit-code-inconsistent",
                f"{location}.exit_code",
                "a timed-out runner result must use exit code 124",
            )
        )
    return (
        {
            "clean_before": clean_before,
            "clean_after": clean_after,
            "index_flags_clean": index_flags_clean,
            "head_unchanged": head_unchanged,
            "log_inside_source_worktree": log_inside,
            "result_inside_source_worktree": result_inside,
        },
        findings,
    )


def bind_timed_validations(
    repo: GitRepo,
    record: RecordModel,
    manifest: ManifestModel | None,
    artifacts: dict[str, ArtifactRef],
    timed_schema: dict[str, Any] | None,
) -> tuple[dict[str, TimedFacts], list[Finding]]:
    findings: list[Finding] = []
    facts: dict[str, TimedFacts] = {}
    if timed_schema is None:
        return facts, [
            error(
                "timed-result-schema-unavailable",
                "schemas.timed_result",
                "the registered timed-result schema is required before typed use",
            )
        ]
    definition_findings = validate_schema_definition_identity(
        timed_schema,
        AUTHORITY_SCHEMA_IDS[TIMED_SCHEMA_PATH],
        "schemas.timed_result",
    )
    if definition_findings:
        return facts, definition_findings
    argv_map = manifest.validation_argv() if manifest is not None else {}
    duplicate_ids = frozenset(duplicates([row.id for row in record.validations]))
    if duplicate_ids:
        findings.append(
            error(
                "duplicate-record-validation-id",
                "record.validations",
                repr(sorted(duplicate_ids)),
            )
        )
    for index, row in enumerate(record.validations):
        location = f"record.validations[{index}]"
        item_findings: list[Finding] = []
        result_artifact = artifacts.get(row.result_ref)
        log_artifact = artifacts.get(row.log_ref)
        if result_artifact is None or log_artifact is None:
            continue
        try:
            result_bytes = repo.blob(
                result_artifact.commit, result_artifact.path.encode("utf-8")
            )
            log_bytes = repo.blob(log_artifact.commit, log_artifact.path.encode("utf-8"))
        except (GitError, UnicodeEncodeError) as exc:
            findings.append(error("timed-evidence-unreadable", location, str(exc)))
            continue
        payload, decode_findings = strict_decode_json(
            result_bytes, f"{location}.result"
        )
        findings.extend(decode_findings)
        if payload is None or decode_findings:
            continue
        schema_findings = validate_json_schema(
            payload, timed_schema, f"{location}.result"
        )
        findings.extend(schema_findings)
        if not isinstance(payload, dict) or payload.get("schema") != TIMED_SCHEMA:
            findings.append(
                error(
                    "timed-result-wrong-schema",
                    f"{location}.result_ref",
                    f"expected {TIMED_SCHEMA}, got "
                    f"{payload.get('schema') if isinstance(payload, dict) else type(payload).__name__!r}; "
                    "a v1 timed result is historical data and cannot satisfy a v2 accept",
                )
            )
        if schema_findings or not isinstance(payload, dict) or payload.get("schema") != TIMED_SCHEMA:
            continue

        derived, semantic_findings = _timed_semantics(
            payload, f"{location}.result"
        )
        item_findings.extend(semantic_findings)
        source = payload["source"]
        log = payload["log"]
        if source["revision"] != record.subject.candidate_revision:
            item_findings.append(
                error(
                    "timed-result-revision-mismatch",
                    f"{location}.result",
                    f"result was produced at {source['revision']!r}, not the bound candidate "
                    f"{record.subject.candidate_revision}",
                )
            )
        if source["tree"] != record.subject.candidate_tree:
            item_findings.append(
                error(
                    "timed-result-tree-mismatch",
                    f"{location}.result",
                    f"result tree {source['tree']!r} is not the candidate tree "
                    f"{record.subject.candidate_tree}",
                )
            )
        declared_argv = tuple(payload["argv"])
        expected_argv = argv_map.get(row.id)
        if expected_argv is not None and declared_argv != expected_argv:
            item_findings.append(
                error(
                    "timed-result-argv-mismatch",
                    f"{location}.result",
                    f"result argv {list(declared_argv)} is not the manifest-frozen argv "
                    f"{list(expected_argv)}",
                )
            )
        log_digest = sha256_bytes(log_bytes)
        if log["sha256"] != log_digest:
            item_findings.append(
                error(
                    "timed-log-digest-mismatch",
                    f"{location}.log_ref",
                    f"result binds {log['sha256']!r}, committed log is {log_digest}",
                )
            )
        if log["bytes"] != len(log_bytes):
            item_findings.append(
                error(
                    "timed-log-byte-count-mismatch",
                    f"{location}.log_ref",
                    f"result binds {log['bytes']!r}, committed log is {len(log_bytes)}",
                )
            )
        for field, declared, actual in (
            ("status", row.status, payload["status"]),
            ("exit_code", row.exit_code, payload["exit_code"]),
            ("timed_out", row.timed_out, payload["timed_out"]),
            ("log_complete", row.log_complete, log["complete"]),
        ):
            if declared != actual:
                item_findings.append(
                    error(
                        "record-timed-evidence-disagreement",
                        f"{location}.{field}",
                        f"record says {declared!r}, committed timed result says {actual!r}",
                    )
                )
        findings.extend(item_findings)
        if item_findings or row.id in duplicate_ids:
            continue
        facts[row.id] = TimedFacts(
            validation_id=row.id,
            result_revision=source["revision"],
            result_tree=source["tree"],
            status=payload["status"],
            exit_code=payload["exit_code"],
            timed_out=payload["timed_out"],
            log_complete=log["complete"],
            clean_before=derived["clean_before"],
            clean_after=derived["clean_after"],
            index_flags_clean=derived["index_flags_clean"],
            head_unchanged=derived["head_unchanged"],
            log_inside_source_worktree=derived["log_inside_source_worktree"],
            result_inside_source_worktree=derived["result_inside_source_worktree"],
            log_bytes=len(log_bytes),
            log_sha256=log_digest,
        )
    return facts, findings


def deciding_gate_identity(validator_root: Path) -> dict[str, Any]:
    """Bind the gate revision and blob digests that produced this decision.

    The validator source repository is deliberately distinct from the analysed
    `--git-root`.  A decision is only allowed to authorize integration when the
    validator artifacts match their declared clean committed revision, so a
    retained decision names an exact, reproducible gate.
    """

    repo = GitRepo(validator_root)
    identity: dict[str, Any] = {
        "source_root": str(validator_root.resolve()),
        "source_revision": None,
        "state": "unavailable",
        "artifacts": [],
    }
    try:
        revision, tree = repo.head_identity()
        status = repo.worktree_status()
        index_flags = repo.index_flagged_entries()
    except GitError as exc:
        identity["detail"] = str(exc)
        return identity
    identity["source_revision"] = revision
    identity["source_tree"] = tree
    identity["worktree"] = {
        "status": list(status),
        "index_flagged_entries": list(index_flags),
    }
    rows: list[dict[str, Any]] = []
    state = "clean_committed"
    for relative in GATE_SOURCE_PATHS:
        on_disk_path = validator_root / relative
        if on_disk_path.is_symlink() or not on_disk_path.is_file():
            rows.append(
                {
                    "path": relative,
                    "mode": None,
                    "sha256": None,
                    "committed_sha256": None,
                }
            )
            state = "unavailable"
            identity["detail"] = f"{relative} is not an on-disk regular file"
            continue
        try:
            on_disk = sha256_bytes(on_disk_path.read_bytes())
        except OSError as exc:
            rows.append(
                {
                    "path": relative,
                    "mode": None,
                    "sha256": None,
                    "committed_sha256": None,
                }
            )
            state = "unavailable"
            identity["detail"] = str(exc)
            continue
        try:
            entry = repo.entry(revision, relative.encode("utf-8"))
            committed = sha256_bytes(repo.blob(revision, relative.encode("utf-8")))
        except GitError:
            entry = None
            committed = None
            state = "untracked"
        if entry is not None and (entry.kind != "blob" or entry.mode != BLOB_MODE):
            state = "untracked"
        if committed is not None and committed != on_disk:
            state = "uncommitted_modification"
        rows.append(
            {
                "path": relative,
                "mode": entry.mode if entry is not None else None,
                "sha256": on_disk,
                "committed_sha256": committed,
            }
        )
    if state == "clean_committed" and (status or index_flags):
        state = "uncommitted_worktree"
    identity["artifacts"] = rows
    identity["state"] = state
    return identity


def _decode_live_arm_anchor(
    payload: bytes,
    anchor_schema: dict[str, Any] | None,
    location: str,
) -> tuple[dict[str, Any] | None, list[Finding]]:
    """Strictly decode and schema-check an anchor before any row selection."""

    raw, findings = strict_decode_json(payload, location)
    if raw is None or findings:
        return None, findings
    if anchor_schema is None:
        return None, [
            error(
                "live-arm-anchor-schema-unavailable",
                location,
                "the registered live-arm-anchor schema is required before typed use",
            )
        ]
    definition_findings = validate_schema_definition_identity(
        anchor_schema,
        AUTHORITY_SCHEMA_IDS[LIVE_ARM_ANCHOR_SCHEMA_PATH],
        f"{location}.schema_definition",
    )
    if definition_findings:
        return None, definition_findings
    schema_findings = validate_json_schema(raw, anchor_schema, location)
    if schema_findings:
        return None, schema_findings
    if not isinstance(raw, dict) or raw.get("schema") != LIVE_ARM_ANCHOR_SCHEMA:
        return None, [
            error(
                "live-arm-anchor-wrong-schema",
                location,
                f"expected {LIVE_ARM_ANCHOR_SCHEMA}",
            )
        ]
    rows = raw["arms"]
    repeated = duplicates([row["arm"] for row in rows])
    if repeated:
        return None, [
            error(
                "duplicate-live-arm-anchor-arm",
                f"{location}.arms",
                repr(repeated),
            )
        ]
    path_findings: list[Finding] = []
    for index, row in enumerate(rows):
        for label in ("repair_packet", "repair_manifest"):
            value = row[label]["path"]
            if not path_components_safe(value):
                path_findings.append(
                    error(
                        "unsafe-live-arm-anchor-path",
                        f"{location}.arms[{index}].{label}.path",
                        repr(value),
                    )
                )
    if path_findings:
        return None, path_findings
    return raw, []


def load_live_arm_anchor(
    validator_root: Path,
    analysed_root: Path,
    arm: str,
    record: RecordModel,
    artifacts: dict[str, ArtifactRef],
    deciding_gate: dict[str, Any],
    anchor_schema: dict[str, Any] | None,
) -> tuple[dict[str, Any] | None, list[Finding]]:
    """Consume a manager-owned live-arm pin from the clean validator source.

    The anchor does not exist in the repository yet, so a `--live-arm` run can
    only block.  Nothing here fabricates a manifest identity.
    """

    location = LIVE_ARM_ANCHOR_PATH
    findings: list[Finding] = []
    validator_root = validator_root.resolve()
    analysed_root = analysed_root.resolve()
    if validator_root == analysed_root:
        findings.append(
            error(
                "live-arm-anchor-from-analysed-repository",
                location,
                "the manager anchor must come from a separate clean validator source "
                "repository, never from the analysed --git-root",
            )
        )
    if deciding_gate.get("state") != "clean_committed":
        findings.append(
            error(
                "live-arm-validator-not-clean-committed",
                location,
                f"validator gate identity state is {deciding_gate.get('state')!r}",
            )
        )
    revision = deciding_gate.get("source_revision")
    if not isinstance(revision, str):
        findings.append(
            error(
                "live-arm-validator-revision-unavailable",
                location,
                "the validator gate identity has no committed source revision",
            )
        )
        return None, findings
    repo = GitRepo(validator_root)
    try:
        head, _ = repo.head_identity()
    except GitError as exc:
        findings.append(error("live-arm-validator-unreadable", location, str(exc)))
        return None, findings
    if head != revision:
        findings.append(
            error(
                "live-arm-validator-head-mismatch",
                location,
                f"validator HEAD is {head}, deciding gate binds {revision}",
            )
        )
    path = validator_root / LIVE_ARM_ANCHOR_PATH
    if path.is_symlink() or not path.is_file():
        return None, [
            *findings,
            error(
                "live-arm-anchor-unavailable",
                location,
                "no manager-owned live-arm anchor is pinned; a separate manager task must pin "
                "each arm's exact manifest path, commit, mode and digest before a live arm runs",
            )
        ]
    try:
        entry = repo.entry(revision, LIVE_ARM_ANCHOR_PATH.encode("utf-8"))
        payload = repo.blob(revision, LIVE_ARM_ANCHOR_PATH.encode("utf-8"))
        on_disk = path.read_bytes()
    except (GitError, OSError) as exc:
        return None, [
            *findings,
            error("live-arm-anchor-unreadable", location, str(exc)),
        ]
    if entry is None or entry.kind != "blob" or entry.mode != BLOB_MODE:
        findings.append(
            error(
                "live-arm-anchor-not-regular-blob",
                location,
                f"expected committed mode {BLOB_MODE}, got {entry!r}",
            )
        )
    digest = sha256_bytes(payload)
    if on_disk != payload:
        findings.append(
            error(
                "live-arm-anchor-uncommitted",
                location,
                "on-disk anchor bytes differ from the validator HEAD blob",
            )
        )
    raw, anchor_findings = _decode_live_arm_anchor(
        payload, anchor_schema, location
    )
    findings.extend(anchor_findings)
    if raw is None:
        return None, findings
    rows = raw["arms"]
    match = next((row for row in rows if row["arm"] == arm), None)
    if match is None:
        return None, [
            *findings,
            error("live-arm-not-pinned", location, f"{arm} has no pinned anchor row")
        ]
    if match["generation"] != record.generation:
        findings.append(
            error(
                "live-arm-generation-mismatch",
                f"{location}.arms",
                f"anchor pins generation {match['generation']}, record is "
                f"generation {record.generation}",
            )
        )
    packet = artifacts.get(record.subject.repair_packet_ref)
    manifest = artifacts.get(record.subject.repair_manifest_ref)
    for label, artifact in (("repair_packet", packet), ("repair_manifest", manifest)):
        pinned = match[label]
        if artifact is None:
            findings.append(
                error("live-arm-pin-incomplete", location, f"{label} is not pinned")
            )
            continue
        if (
            pinned.get("commit") != artifact.commit
            or pinned.get("path") != artifact.path
            or pinned.get("sha256") != artifact.sha256
            or pinned.get("mode") != BLOB_MODE
        ):
            findings.append(
                error(
                    "live-arm-pin-mismatch",
                    location,
                    f"{label} does not equal the pinned {arm} identity",
                )
            )
    if match.get("predecessor_authority_commit") != record.subject.authority.commit:
        findings.append(
            error(
                "live-arm-authority-mismatch",
                location,
                "the record's predecessor authority is not the pinned one",
            )
        )
    if findings:
        return None, findings
    return (
        {
            "state": "verified",
            "anchor": {
                "validator_commit": revision,
                "path": LIVE_ARM_ANCHOR_PATH,
                "mode": BLOB_MODE,
                "sha256": digest,
            },
            "arm": match,
        },
        [],
    )


def verify_retained_live_arm_anchor(
    validator_root: Path,
    arm: str,
    record: RecordModel,
    artifacts: dict[str, ArtifactRef],
    retained_result: dict[str, Any],
    historical_gate: dict[str, Any],
    anchor_schema: dict[str, Any] | None,
) -> list[Finding]:
    """Re-prove a retained generation's manager anchor from validator Git.

    Each generation may have been validated at a different clean validator
    commit.  Its retained result therefore carries the exact anchor blob and
    selected arm row used at that time.  The current validator repository must
    still contain that commit on the ancestry of its current HEAD; candidate
    Git and a self-authored retained JSON claim are never anchor authority.
    """

    location = "result.live_arm_pin"
    findings: list[Finding] = []
    if retained_result.get("live_arm") != arm:
        findings.append(
            error(
                "retained-live-arm-mismatch",
                "result.live_arm",
                f"retained {retained_result.get('live_arm')!r}, rooted arm is {arm!r}",
            )
        )
    pin = retained_result.get("live_arm_pin")
    if not isinstance(pin, dict):
        return [
            *findings,
            error(
                "retained-live-arm-pin-unavailable",
                location,
                "every retained live generation must bind its validator-owned anchor",
            ),
        ]
    if pin.get("state") != "verified":
        return [
            *findings,
            error(
                "retained-live-arm-pin-unavailable",
                f"{location}.state",
                "every retained live generation requires state 'verified'",
            ),
        ]
    anchor = pin.get("anchor")
    retained_arm = pin.get("arm")
    if not isinstance(anchor, dict) or not isinstance(retained_arm, dict):
        return [
            *findings,
            error(
                "retained-live-arm-pin-incomplete",
                location,
                "anchor identity and selected arm row are both required",
            ),
        ]
    revision = historical_gate.get("source_revision")
    if not isinstance(revision, str):
        return [
            *findings,
            error(
                "retained-live-arm-gate-revision-unavailable",
                location,
                "the retained deciding gate has no canonical source revision",
            ),
        ]
    expected_anchor = {
        "validator_commit": revision,
        "path": LIVE_ARM_ANCHOR_PATH,
        "mode": BLOB_MODE,
    }
    for field, expected in expected_anchor.items():
        if anchor.get(field) != expected:
            findings.append(
                error(
                    "retained-live-arm-anchor-identity-mismatch",
                    f"{location}.anchor.{field}",
                    f"retained {anchor.get(field)!r}, expected {expected!r}",
                )
            )
    repo = GitRepo(validator_root.resolve())
    try:
        head, _ = repo.head_identity()
        resolved = repo.resolve_commit(revision)
        entry = repo.entry(revision, LIVE_ARM_ANCHOR_PATH.encode("utf-8"))
        payload = repo.blob(revision, LIVE_ARM_ANCHOR_PATH.encode("utf-8"))
    except GitError as exc:
        return [
            *findings,
            error("retained-live-arm-anchor-unreadable", location, str(exc)),
        ]
    if resolved != revision:
        findings.append(
            error(
                "retained-live-arm-validator-revision-not-canonical",
                location,
                f"resolved {resolved}",
            )
        )
    if revision != head and not repo.is_ancestor(revision, head):
        findings.append(
            error(
                "retained-live-arm-validator-revision-unrooted",
                location,
                f"historical validator commit {revision} is not on current HEAD {head}",
            )
        )
    if entry is None or entry.kind != "blob" or entry.mode != BLOB_MODE:
        findings.append(
            error(
                "retained-live-arm-anchor-not-regular-blob",
                location,
                repr(entry),
            )
        )
    digest = sha256_bytes(payload)
    if anchor.get("sha256") != digest:
        findings.append(
            error(
                "retained-live-arm-anchor-digest-mismatch",
                f"{location}.anchor.sha256",
                f"computed {digest}",
            )
        )
    raw, anchor_findings = _decode_live_arm_anchor(
        payload, anchor_schema, f"{location}.historical_anchor"
    )
    findings.extend(anchor_findings)
    if raw is None:
        return findings
    rows = raw["arms"]
    match = next((row for row in rows if row["arm"] == arm), None)
    if match is None:
        return [
            *findings,
            error("retained-live-arm-not-pinned", location, f"{arm} has no anchor row"),
        ]
    if match["generation"] != record.generation:
        findings.append(
            error(
                "retained-live-arm-generation-mismatch",
                f"{location}.arm.generation",
                f"anchor pins generation {match['generation']}, retained record is "
                f"generation {record.generation}",
            )
        )
    if retained_arm != match:
        findings.append(
            error(
                "retained-live-arm-row-mismatch",
                f"{location}.arm",
                "retained arm row differs from the committed historical anchor",
            )
        )
    packet = artifacts.get(record.subject.repair_packet_ref)
    manifest = artifacts.get(record.subject.repair_manifest_ref)
    for label, artifact in (("repair_packet", packet), ("repair_manifest", manifest)):
        pinned = match.get(label)
        if not isinstance(pinned, dict) or artifact is None:
            findings.append(
                error("retained-live-arm-pin-incomplete", location, f"{label} is not pinned")
            )
            continue
        expected = {
            "commit": artifact.commit,
            "path": artifact.path,
            "sha256": artifact.sha256,
            "mode": BLOB_MODE,
        }
        if pinned != expected:
            findings.append(
                error(
                    "retained-live-arm-pin-mismatch",
                    location,
                    f"{label} does not equal the retained generation artifact",
                )
            )
    if match.get("predecessor_authority_commit") != record.subject.authority.commit:
        findings.append(
            error(
                "retained-live-arm-authority-mismatch",
                location,
                "the retained generation authority is not the historical pinned authority",
            )
        )
    return findings
