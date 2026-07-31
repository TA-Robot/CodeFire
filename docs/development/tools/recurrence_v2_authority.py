#!/usr/bin/env python3
"""Rooted predecessor-authority resolution for the recurrence gate v2.

`resolve_predecessor_authority_chain` is the contract's named machine reader.  A
prior v2 record is never trusted as a standalone JSON blob: the walk follows the
`predecessor_authority` chain transitively until one of the manager-fixed legacy
roots, and at every link it re-verifies generation continuity, candidate
identity, the whole packet/candidate/control/report/evidence/retention topology,
exact per-commit write sets, modes, blob digests, record/result digest equality,
the exported successor-obligation set, and the deciding-gate identity that
validated that link.

Cycles, missing links, a discontinuous generation, or a chain that does not
terminate at a manager-fixed root produce findings.  Ref existence is never a
substitute for this walk: nothing here resolves a branch or tag.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any

from ai_contracts import validate_json_schema
from recurrence_v2_git import GitError, GitRepo, sha256_bytes
from recurrence_v2_model import (
    BLOB_MODE,
    LEGACY_ROOTS,
    RESULT_SCHEMA,
    Authority,
    Finding,
    RecordModel,
    error,
    parse_record,
    path_components_safe,
)
from recurrence_v2_topology import GenerationView, verify_generation, verify_retention

MAX_CHAIN_LINKS = 64


@dataclass(frozen=True)
class ChainLink:
    generation: int
    candidate: str
    retention_commit: str
    evidence_commit: str
    record_path: str
    record_sha256: str
    result_path: str
    result_sha256: str
    successor_obligation_ids: tuple[str, ...]
    obligation_states: dict[str, str]
    obligation_origins: dict[str, int]
    deciding_gate: dict[str, Any]
    verified_commits: frozenset[str]


@dataclass(frozen=True)
class ChainResult:
    rooted: bool = False
    root_arm: str | None = None
    root_commit: str | None = None
    links: tuple[ChainLink, ...] = ()
    verified_commits: frozenset[str] = frozenset()
    validator_divergences: tuple[dict[str, Any], ...] = ()
    depth: int = 0

    @property
    def predecessor_link(self) -> ChainLink | None:
        """The immediate predecessor generation, if this chain has one."""

        return self.links[0] if self.links else None

    def summary(self) -> list[dict[str, Any]]:
        return [
            {
                "generation": link.generation,
                "candidate_revision": link.candidate,
                "retention_commit": link.retention_commit,
                "record_path": link.record_path,
                "record_sha256": link.record_sha256,
                "result_path": link.result_path,
                "result_sha256": link.result_sha256,
                "successor_obligation_ids": list(link.successor_obligation_ids),
                "deciding_gate": link.deciding_gate,
            }
            for link in self.links
        ]


def _evidence_paths(record: RecordModel) -> frozenset[str]:
    return frozenset(
        artifact.path
        for artifact in record.artifacts
        if artifact.role in {"timed_result", "timed_log", "terminal_event_evidence"}
    )


def generation_view(record: RecordModel) -> GenerationView | None:
    index = record.artifact_index()
    packet = index.get(record.subject.repair_packet_ref)
    manifest = index.get(record.subject.repair_manifest_ref)
    review = index.get(record.subject.review_packet_ref)
    report = index.get(record.subject.report_ref)
    if packet is None or manifest is None or review is None or report is None:
        return None
    return GenerationView(
        control_id=record.control_id,
        predecessor_candidate=record.subject.predecessor_candidate_revision,
        candidate=record.subject.candidate_revision,
        candidate_tree=record.subject.candidate_tree,
        patch_sha256=record.subject.patch_sha256,
        changed_paths=frozenset(record.subject.changed_paths),
        packet_commit=record.subject.repair_packet_commit,
        packet_path=packet.path,
        manifest_path=manifest.path,
        review_control_commit=record.subject.review_control_commit,
        review_packet_path=review.path,
        report_commit=record.subject.report_commit,
        report_path=report.path,
        evidence_commit=record.subject.evidence_commit,
        evidence_paths=_evidence_paths(record),
    )


def verify_legacy_root(
    repo: GitRepo,
    authority: Authority,
    predecessor_candidate: str,
    artifacts: dict[str, Any],
    location: str,
    roots: dict[str, Any] | None = None,
) -> tuple[bool, list[Finding]]:
    """Verify one manager-fixed bootstrap root by its pinned identity.

    `roots` exists so a synthetic history can be exercised end to end; the CLI
    never overrides it, so production always uses the two manager-fixed rows.
    """

    findings: list[Finding] = []
    roots = LEGACY_ROOTS if roots is None else roots
    arm = authority.bootstrap_arm
    root = roots.get(arm) if isinstance(arm, str) else None
    if root is None:
        findings.append(
            error(
                "unpinned-legacy-authority-root",
                f"{location}.bootstrap_arm",
                f"{arm!r} is not one of the manager-fixed roots "
                f"{sorted(roots)}; no other legacy binding may serve as a root",
            )
        )
        return False, findings
    if authority.commit != root.commit:
        findings.append(
            error(
                "legacy-root-commit-mismatch",
                f"{location}.commit",
                f"expected {root.commit} for arm {root.arm}",
            )
        )
        return False, findings
    if predecessor_candidate != root.predecessor_candidate:
        findings.append(
            error(
                "legacy-root-predecessor-mismatch",
                f"{location}.commit",
                f"arm {root.arm} pins predecessor {root.predecessor_candidate}",
            )
        )
    try:
        parents = repo.parents(root.commit)
        write_set = repo.write_set(root.commit)
        entry = repo.entry(root.commit, root.report_path.encode("utf-8"))
        blob = repo.blob(root.commit, root.report_path.encode("utf-8"))
    except GitError as exc:
        findings.append(error("legacy-root-git-binding-failed", location, str(exc)))
        return False, findings
    if parents != (root.predecessor_candidate,):
        findings.append(
            error(
                "legacy-root-parent-mismatch",
                location,
                f"parents are {list(parents)}, expected exactly [{root.predecessor_candidate}]",
            )
        )
    if write_set.raw != frozenset({root.report_path.encode("utf-8")}):
        findings.append(
            error(
                "legacy-root-write-set-mismatch",
                location,
                repr(write_set.rendered()),
            )
        )
    if entry is None or entry.mode != "100644" or entry.kind != "blob":
        findings.append(
            error(
                "legacy-root-report-not-a-regular-blob",
                location,
                repr(entry),
            )
        )
    digest = sha256_bytes(blob)
    if digest != root.report_sha256:
        findings.append(
            error("legacy-root-report-digest-mismatch", location, f"computed {digest}"),
        )
    declared = [
        artifact
        for artifact in artifacts.values()
        if artifact.role == "legacy_bootstrap_report"
    ]
    if len(declared) != 1:
        findings.append(
            error(
                "bootstrap-authority-artifact-set-mismatch",
                f"{location}.artifact_refs",
                f"expected exactly one legacy_bootstrap_report artifact, found {len(declared)}",
            )
        )
    else:
        bound = declared[0]
        if (
            bound.commit != root.commit
            or bound.path != root.report_path
            or bound.sha256 != root.report_sha256
        ):
            findings.append(
                error(
                    "bootstrap-authority-artifact-not-pinned",
                    f"{location}.artifact_refs",
                    f"{bound.ref_id} does not equal the pinned {root.arm} root binding",
                )
            )
    return not findings, findings


def _load_json_blob(
    repo: GitRepo,
    commit: str,
    path: str,
    location: str,
    findings: list[Finding],
) -> tuple[Any, str] | None:
    if not path_components_safe(path):
        findings.append(error("unsafe-retained-path", location, repr(path)))
        return None
    try:
        entry = repo.entry(commit, path.encode("utf-8"))
        payload = repo.blob(commit, path.encode("utf-8"))
    except GitError as exc:
        findings.append(error("retained-artifact-unreadable", location, str(exc)))
        return None
    if entry is None or entry.mode != "100644" or entry.kind != "blob":
        findings.append(
            error("retained-artifact-not-a-regular-blob", location, repr(entry))
        )
        return None
    try:
        parsed = json.loads(payload.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        findings.append(error("retained-artifact-not-json", location, str(exc)))
        return None
    return parsed, sha256_bytes(payload)


def _check_deciding_gate(
    raw: Any,
    repo: GitRepo,
    current_gate: dict[str, Any],
    location: str,
    findings: list[Finding],
) -> dict[str, Any]:
    """Verify a retained gate identity against committed Git blobs.

    A historical result may name a different known gate revision, but it may
    not invent its own source set or self-attest digests that Git cannot
    reproduce.  Semantic migration policy is recorded separately by the chain
    result; this reader only establishes exact identity.
    """

    if not isinstance(raw, dict):
        findings.append(
            error(
                "link-without-deciding-gate",
                location,
                "the retained result does not record which gate revision validated it",
            )
        )
        return {}
    required = ("state", "source_revision", "source_tree", "artifacts")
    missing = [key for key in required if key not in raw]
    if missing:
        findings.append(
            error("link-deciding-gate-incomplete", location, f"missing {missing}")
        )
        return {}
    if raw.get("state") != "clean_committed":
        findings.append(
            error(
                "link-deciding-gate-not-clean-committed",
                location,
                f"state is {raw.get('state')!r}",
            )
        )
    revision = raw.get("source_revision")
    if not isinstance(revision, str):
        findings.append(
            error("link-deciding-gate-revision-invalid", location, repr(revision))
        )
        return {}
    try:
        resolved_revision = repo.resolve_commit(revision)
    except GitError as exc:
        findings.append(
            error("link-deciding-gate-revision-unreadable", location, str(exc))
        )
        return {}
    if resolved_revision != revision:
        findings.append(
            error(
                "link-deciding-gate-revision-not-canonical",
                location,
                f"resolved {resolved_revision}",
            )
        )
    try:
        actual_tree = repo.tree_of(revision)
    except GitError as exc:
        findings.append(error("link-deciding-gate-tree-unreadable", location, str(exc)))
        actual_tree = None
    if raw.get("source_tree") != actual_tree:
        findings.append(
            error(
                "link-deciding-gate-tree-mismatch",
                f"{location}.source_tree",
                f"recorded {raw.get('source_tree')!r}, computed {actual_tree!r}",
            )
        )
    rows = raw.get("artifacts")
    if not isinstance(rows, list) or not rows:
        findings.append(
            error(
                "link-deciding-gate-artifacts-invalid",
                f"{location}.artifacts",
                "a nonempty artifact list is required",
            )
        )
        return {}
    current_rows = current_gate.get("artifacts")
    current_row_list = current_rows if isinstance(current_rows, list) else []
    expected_paths = {
        row.get("path")
        for row in current_row_list
        if isinstance(row, dict)
        and isinstance(row.get("path"), str)
    }
    seen: set[str] = set()
    for index, row in enumerate(rows):
        where = f"{location}.artifacts[{index}]"
        if not isinstance(row, dict):
            findings.append(error("link-gate-artifact-not-object", where, repr(row)))
            continue
        path = row.get("path")
        digest = row.get("sha256")
        committed_digest = row.get("committed_sha256")
        mode = row.get("mode")
        if not isinstance(path, str) or not path_components_safe(path):
            findings.append(error("link-gate-artifact-path-invalid", where, repr(path)))
            continue
        if path in seen:
            findings.append(error("link-gate-artifact-path-duplicate", where, path))
            continue
        seen.add(path)
        if mode != BLOB_MODE:
            findings.append(
                error("link-gate-artifact-mode-invalid", f"{where}.mode", repr(mode))
            )
        if digest != committed_digest:
            findings.append(
                error(
                    "link-gate-artifact-working-tree-drift",
                    where,
                    "recorded on-disk and committed digests differ",
                )
            )
        try:
            entry = repo.entry(revision, path.encode("utf-8"))
            payload = repo.blob(revision, path.encode("utf-8"))
        except GitError as exc:
            findings.append(error("link-gate-artifact-unreadable", where, str(exc)))
            continue
        if entry is None or entry.kind != "blob" or entry.mode != BLOB_MODE:
            findings.append(
                error(
                    "link-gate-artifact-not-regular-blob",
                    where,
                    f"expected {BLOB_MODE} blob, got {entry!r}",
                )
            )
        actual = sha256_bytes(payload)
        if digest != actual or committed_digest != actual:
            findings.append(
                error(
                    "link-gate-artifact-digest-mismatch",
                    where,
                    f"computed {actual}",
                )
            )
    if seen != expected_paths:
        findings.append(
            error(
                "link-deciding-gate-source-set-unknown",
                f"{location}.artifacts",
                f"historical-only={sorted(seen - expected_paths)}, "
                f"current-only={sorted(expected_paths - seen)}; a different source set needs "
                "an explicit compatibility authorization",
            )
        )
    return raw


def _gate_fingerprint(identity: dict[str, Any]) -> tuple[str, ...]:
    artifacts = identity.get("artifacts")
    rows = artifacts if isinstance(artifacts, list) else []
    artifact_rows = tuple(
        sorted(
            f"{row.get('path')}:{row.get('mode')}={row.get('sha256')}"
            for row in rows
            if isinstance(row, dict)
        )
    )
    return (
        f"state={identity.get('state')}",
        f"source_revision={identity.get('source_revision')}",
        f"source_tree={identity.get('source_tree')}",
        *artifact_rows,
    )


def resolve_predecessor_authority_chain(
    repo: GitRepo,
    record: RecordModel,
    *,
    schema: dict[str, Any] | None,
    deciding_gate: dict[str, Any],
    validator_repo: GitRepo | None = None,
    legacy_roots: dict[str, Any] | None = None,
    location: str = "record.subject.predecessor_authority",
) -> tuple[ChainResult, list[Finding]]:
    findings: list[Finding] = []
    links: list[ChainLink] = []
    verified: set[str] = set()
    divergences: list[dict[str, Any]] = []
    visited: set[str] = set()

    authority = record.subject.authority
    generation = record.generation
    predecessor_candidate = record.subject.predecessor_candidate_revision
    artifacts = record.artifact_index()
    current_location = location
    current_gate = _gate_fingerprint(deciding_gate)
    gate_repo = repo if validator_repo is None else validator_repo

    depth = 0
    while True:
        if depth > MAX_CHAIN_LINKS:
            findings.append(
                error(
                    "authority-chain-too-deep",
                    current_location,
                    f"more than {MAX_CHAIN_LINKS} links",
                )
            )
            return ChainResult(depth=depth), findings
        if authority.commit in visited:
            findings.append(
                error(
                    "authority-chain-cycle",
                    current_location,
                    f"{authority.commit} appears twice in the chain",
                )
            )
            return ChainResult(depth=depth), findings
        visited.add(authority.commit)
        depth += 1

        if authority.kind == "legacy_bootstrap_report":
            rooted, root_findings = verify_legacy_root(
                repo,
                authority,
                predecessor_candidate,
                artifacts,
                current_location,
                legacy_roots,
            )
            findings.extend(root_findings)
            if rooted:
                verified.add(authority.commit)
            return (
                ChainResult(
                    rooted=rooted,
                    root_arm=authority.bootstrap_arm,
                    root_commit=authority.commit,
                    links=tuple(links),
                    verified_commits=frozenset(verified),
                    validator_divergences=tuple(divergences),
                    depth=depth,
                ),
                findings,
            )
        if authority.kind != "prior_v2_record":
            findings.append(
                error(
                    "unknown-authority-kind",
                    f"{current_location}.kind",
                    repr(authority.kind),
                )
            )
            return ChainResult(depth=depth), findings

        record_refs = [
            artifact
            for artifact in artifacts.values()
            if artifact.role == "prior_v2_record"
        ]
        result_refs = [
            artifact
            for artifact in artifacts.values()
            if artifact.role == "prior_v2_result"
        ]
        if len(record_refs) != 1 or len(result_refs) != 1:
            findings.append(
                error(
                    "prior-authority-artifact-set-mismatch",
                    f"{current_location}.artifact_refs",
                    "exactly one prior_v2_record and one prior_v2_result artifact are required",
                )
            )
            return ChainResult(depth=depth), findings
        record_ref, result_ref = record_refs[0], result_refs[0]
        retention = authority.commit
        for artifact in (record_ref, result_ref):
            if artifact.commit != retention:
                findings.append(
                    error(
                        "prior-authority-artifact-off-retention-commit",
                        f"{current_location}.artifact_refs",
                        f"{artifact.ref_id} resolves at {artifact.commit}, not the retention commit {retention}",
                    )
                )
        loaded_record = _load_json_blob(
            repo, retention, record_ref.path, f"{current_location}.record", findings
        )
        loaded_result = _load_json_blob(
            repo, retention, result_ref.path, f"{current_location}.result", findings
        )
        if loaded_record is None or loaded_result is None:
            return ChainResult(depth=depth), findings
        prior_raw, prior_record_digest = loaded_record
        prior_result, prior_result_digest = loaded_result
        if prior_record_digest != record_ref.sha256:
            findings.append(
                error(
                    "prior-record-digest-mismatch",
                    f"{current_location}.record",
                    f"computed {prior_record_digest}",
                )
            )
        if prior_result_digest != result_ref.sha256:
            findings.append(
                error(
                    "prior-result-digest-mismatch",
                    f"{current_location}.result",
                    f"computed {prior_result_digest}",
                )
            )
        if schema is not None:
            findings.extend(
                validate_json_schema(
                    prior_raw, schema, f"{current_location}.record"
                )
            )
        prior, parse_findings = parse_record(prior_raw, f"{current_location}.record")
        findings.extend(parse_findings)
        if prior is None:
            return ChainResult(depth=depth), findings

        if not isinstance(prior_result, dict) or prior_result.get("schema") != RESULT_SCHEMA:
            findings.append(
                error(
                    "prior-result-wrong-schema",
                    f"{current_location}.result",
                    f"expected {RESULT_SCHEMA}",
                )
            )
            return ChainResult(depth=depth), findings
        if prior_result.get("record_sha256") != prior_record_digest:
            findings.append(
                error(
                    "prior-record-result-digest-disagreement",
                    f"{current_location}.result",
                    f"result binds {prior_result.get('record_sha256')!r}, retained record is {prior_record_digest}",
                )
            )
        if prior.generation != generation - 1:
            findings.append(
                error(
                    "authority-generation-discontinuous",
                    f"{current_location}.record",
                    f"link generation {prior.generation} does not precede {generation}",
                )
            )
        if prior.subject.candidate_revision != predecessor_candidate:
            findings.append(
                error(
                    "authority-candidate-mismatch",
                    f"{current_location}.record",
                    f"link candidate {prior.subject.candidate_revision} is not the declared "
                    f"predecessor candidate {predecessor_candidate}",
                )
            )
        view = generation_view(prior)
        if view is None:
            findings.append(
                error(
                    "link-artifact-reference-unresolved",
                    f"{current_location}.record",
                    "the retained record's own packet/manifest/review/report references do not resolve",
                )
            )
            return ChainResult(depth=depth), findings
        _, link_findings = verify_generation(repo, view, f"{current_location}.record")
        findings.extend(link_findings)
        findings.extend(
            verify_retention(
                repo,
                evidence_commit=prior.subject.evidence_commit,
                retention_commit=retention,
                record_path=record_ref.path,
                result_path=result_ref.path,
                location=f"{current_location}.retention",
            )
        )
        derived_successors = tuple(
            sorted(
                {row.id for row in prior.prior_obligations}
                | {row.id for row in prior.new_findings}
            )
        )
        exported = prior_result.get("successor_obligation_ids")
        exported_tuple = (
            tuple(sorted(item for item in exported if isinstance(item, str)))
            if isinstance(exported, list)
            else ()
        )
        if exported_tuple != derived_successors:
            findings.append(
                error(
                    "exported-successor-obligations-tampered",
                    f"{current_location}.result",
                    f"retained result exports {list(exported_tuple)}, the retained record derives "
                    f"{list(derived_successors)}",
                )
            )
        link_gate = _check_deciding_gate(
            prior_result.get("deciding_gate"),
            gate_repo,
            deciding_gate,
            f"{current_location}.result.deciding_gate",
            findings,
        )
        if link_gate and _gate_fingerprint(link_gate) != current_gate:
            divergences.append(
                {
                    "generation": prior.generation,
                    "historical_gate_revision": link_gate.get("source_revision"),
                    "current_gate_revision": deciding_gate.get("source_revision"),
                    "disposition": "compatibility_authorized",
                    "historical_decision_disposition": (
                        "retained_without_semantic_reproof"
                    ),
                    "inherits_current_authority": False,
                    "migration": "current_gate_full_semantic_replay",
                    "note": (
                        "the exact historical decision remains provenance only and does not "
                        "silently inherit current authority; the link can enter the current "
                        "authority chain only after the separate oldest-first semantic replay"
                    ),
                }
            )
        link_verified = {
            prior.subject.review_control_commit,
            prior.subject.report_commit,
            prior.subject.evidence_commit,
            retention,
        }
        verified |= link_verified
        links.append(
            ChainLink(
                generation=prior.generation,
                candidate=prior.subject.candidate_revision,
                retention_commit=retention,
                evidence_commit=prior.subject.evidence_commit,
                record_path=record_ref.path,
                record_sha256=prior_record_digest,
                result_path=result_ref.path,
                result_sha256=prior_result_digest,
                successor_obligation_ids=derived_successors,
                obligation_states={
                    row.id: row.evidence_state for row in prior.prior_obligations
                }
                | {row.id: row.evidence_state for row in prior.new_findings},
                obligation_origins={
                    row.id: row.origin_generation for row in prior.prior_obligations
                }
                | {row.id: prior.generation for row in prior.new_findings},
                deciding_gate=link_gate,
                verified_commits=frozenset(link_verified),
            )
        )
        authority = prior.subject.authority
        generation = prior.generation
        predecessor_candidate = prior.subject.predecessor_candidate_revision
        artifacts = prior.artifact_index()
        current_location = f"{current_location}.record.predecessor_authority"
