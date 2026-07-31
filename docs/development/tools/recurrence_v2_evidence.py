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

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from ai_contracts import validate_json_schema
from recurrence_v2_authority import ChainResult
from recurrence_v2_git import GitError, GitRepo, sha256_bytes
from recurrence_v2_model import (
    BLOB_MODE,
    LEGACY_ROOTS,
    LIVE_ARM_ANCHOR_PATH,
    LIVE_ARM_ANCHOR_SCHEMA,
    MANIFEST_SCHEMA_PATH,
    OBSERVATION_ROLES,
    RECORD_SCHEMA_PATH,
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
)

GATE_SOURCE_PATHS = (
    "docs/development/tools/check_review_recurrence.py",
    "docs/development/tools/recurrence_v2_model.py",
    "docs/development/tools/recurrence_v2_git.py",
    "docs/development/tools/recurrence_v2_topology.py",
    "docs/development/tools/recurrence_v2_authority.py",
    "docs/development/tools/recurrence_v2_evidence.py",
    "docs/development/tools/recurrence_v2_decision.py",
    RECORD_SCHEMA_PATH,
    MANIFEST_SCHEMA_PATH,
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
    log_bytes: int
    log_sha256: str

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
        )


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


def load_manifest(
    repo: GitRepo,
    record: RecordModel,
    artifacts: dict[str, ArtifactRef],
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
    try:
        payload = repo.blob(
            manifest_artifact.commit, manifest_artifact.path.encode("utf-8")
        )
    except GitError as exc:
        return None, [error("manifest-unreadable", location, str(exc))]
    try:
        raw = json.loads(payload.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        return None, [error("manifest-not-json", location, str(exc))]
    if manifest_schema is not None:
        findings.extend(validate_json_schema(raw, manifest_schema, location))
    manifest, parse_findings = parse_manifest(raw, location)
    findings.extend(parse_findings)
    if manifest is None:
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
        if not mapped and record.subject.authority.kind == "legacy_bootstrap_report":
            findings.append(
                error(
                    "bootstrap-manifest-empty-closure-mapping",
                    f"{location}.prior_obligations",
                    f"{obligation_id} has no closure validation; a bootstrap manifest has no "
                    "predecessor v2 record and therefore no admissible exemption",
                )
            )
    return manifest, findings


def bind_timed_validations(
    repo: GitRepo,
    record: RecordModel,
    manifest: ManifestModel | None,
    artifacts: dict[str, ArtifactRef],
    timed_schema: dict[str, Any] | None,
) -> tuple[dict[str, TimedFacts], list[Finding]]:
    findings: list[Finding] = []
    facts: dict[str, TimedFacts] = {}
    argv_map = manifest.validation_argv() if manifest is not None else {}
    for index, row in enumerate(record.validations):
        location = f"record.validations[{index}]"
        result_artifact = artifacts.get(row.result_ref)
        log_artifact = artifacts.get(row.log_ref)
        if result_artifact is None or log_artifact is None:
            continue
        try:
            result_bytes = repo.blob(
                result_artifact.commit, result_artifact.path.encode("utf-8")
            )
            log_bytes = repo.blob(log_artifact.commit, log_artifact.path.encode("utf-8"))
        except GitError as exc:
            findings.append(error("timed-evidence-unreadable", location, str(exc)))
            continue
        try:
            payload = json.loads(result_bytes.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            findings.append(error("timed-result-not-json", location, str(exc)))
            continue
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
            continue
        if timed_schema is not None:
            findings.extend(
                validate_json_schema(payload, timed_schema, f"{location}.result")
            )
        source = payload.get("source") if isinstance(payload.get("source"), dict) else {}
        worktree = (
            payload.get("worktree") if isinstance(payload.get("worktree"), dict) else {}
        )
        log = payload.get("log") if isinstance(payload.get("log"), dict) else {}
        if source.get("revision") != record.subject.candidate_revision:
            findings.append(
                error(
                    "timed-result-revision-mismatch",
                    f"{location}.result",
                    f"result was produced at {source.get('revision')!r}, not the bound candidate "
                    f"{record.subject.candidate_revision}",
                )
            )
        if source.get("tree") != record.subject.candidate_tree:
            findings.append(
                error(
                    "timed-result-tree-mismatch",
                    f"{location}.result",
                    f"result tree {source.get('tree')!r} is not the candidate tree "
                    f"{record.subject.candidate_tree}",
                )
            )
        declared_argv = tuple(
            item for item in payload.get("argv", []) if isinstance(item, str)
        )
        expected_argv = argv_map.get(row.id)
        if expected_argv is not None and declared_argv != expected_argv:
            findings.append(
                error(
                    "timed-result-argv-mismatch",
                    f"{location}.result",
                    f"result argv {list(declared_argv)} is not the manifest-frozen argv "
                    f"{list(expected_argv)}",
                )
            )
        log_digest = sha256_bytes(log_bytes)
        if log.get("sha256") != log_digest:
            findings.append(
                error(
                    "timed-log-digest-mismatch",
                    f"{location}.log_ref",
                    f"result binds {log.get('sha256')!r}, committed log is {log_digest}",
                )
            )
        if log.get("bytes") != len(log_bytes):
            findings.append(
                error(
                    "timed-log-byte-count-mismatch",
                    f"{location}.log_ref",
                    f"result binds {log.get('bytes')!r}, committed log is {len(log_bytes)}",
                )
            )
        for field, declared, actual in (
            ("status", row.status, payload.get("status")),
            ("exit_code", row.exit_code, payload.get("exit_code")),
            ("timed_out", row.timed_out, payload.get("timed_out")),
            ("log_complete", row.log_complete, log.get("complete")),
        ):
            if declared != actual:
                findings.append(
                    error(
                        "record-timed-evidence-disagreement",
                        f"{location}.{field}",
                        f"record says {declared!r}, committed timed result says {actual!r}",
                    )
                )
        facts[row.id] = TimedFacts(
            validation_id=row.id,
            result_revision=str(source.get("revision")),
            result_tree=str(source.get("tree")),
            status=str(payload.get("status")),
            exit_code=payload.get("exit_code")
            if isinstance(payload.get("exit_code"), int)
            and not isinstance(payload.get("exit_code"), bool)
            else None,
            timed_out=payload.get("timed_out") is True,
            log_complete=log.get("complete") is True,
            clean_before=worktree.get("clean_before") is True,
            clean_after=worktree.get("clean_after") is True,
            index_flags_clean=worktree.get("index_flags_clean") is True,
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
        revision, _ = repo.head_identity()
    except GitError as exc:
        identity["detail"] = str(exc)
        return identity
    identity["source_revision"] = revision
    rows: list[dict[str, Any]] = []
    state = "clean_committed"
    for relative in GATE_SOURCE_PATHS:
        on_disk_path = validator_root / relative
        try:
            on_disk = sha256_bytes(on_disk_path.read_bytes())
        except OSError as exc:
            rows.append({"path": relative, "sha256": None, "committed_sha256": None})
            state = "unavailable"
            identity["detail"] = str(exc)
            continue
        try:
            committed = sha256_bytes(repo.blob(revision, relative.encode("utf-8")))
        except GitError:
            committed = None
            state = "untracked"
        if committed is not None and committed != on_disk:
            state = "uncommitted_modification"
        rows.append(
            {"path": relative, "sha256": on_disk, "committed_sha256": committed}
        )
    identity["artifacts"] = rows
    identity["state"] = state
    return identity


def load_live_arm_anchor(
    root: Path,
    arm: str,
    record: RecordModel,
    artifacts: dict[str, ArtifactRef],
) -> tuple[dict[str, Any] | None, list[Finding]]:
    """Consume the future manager-owned live-arm pin, fail-closed today.

    The anchor does not exist in the repository yet, so a `--live-arm` run can
    only block.  Nothing here fabricates a manifest identity.
    """

    location = LIVE_ARM_ANCHOR_PATH
    path = root / LIVE_ARM_ANCHOR_PATH
    if not path.is_file():
        return None, [
            error(
                "live-arm-anchor-unavailable",
                location,
                "no manager-owned live-arm anchor is pinned; a separate manager task must pin "
                "each arm's exact manifest path, commit, mode and digest before a live arm runs",
            )
        ]
    try:
        raw = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return None, [error("live-arm-anchor-unreadable", location, str(exc))]
    if not isinstance(raw, dict) or raw.get("schema") != LIVE_ARM_ANCHOR_SCHEMA:
        return None, [
            error(
                "live-arm-anchor-wrong-schema",
                location,
                f"expected {LIVE_ARM_ANCHOR_SCHEMA}",
            )
        ]
    rows = raw.get("arms") if isinstance(raw.get("arms"), list) else []
    match = next(
        (row for row in rows if isinstance(row, dict) and row.get("arm") == arm),
        None,
    )
    if match is None:
        return None, [
            error("live-arm-not-pinned", location, f"{arm} has no pinned anchor row")
        ]
    findings: list[Finding] = []
    packet = artifacts.get(record.subject.repair_packet_ref)
    manifest = artifacts.get(record.subject.repair_manifest_ref)
    for label, artifact in (("repair_packet", packet), ("repair_manifest", manifest)):
        pinned = match.get(label)
        if not isinstance(pinned, dict) or artifact is None:
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
    return match, findings
