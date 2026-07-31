#!/usr/bin/env python3
"""Current-gate semantic replay of every retained recurrence-v2 generation.

The authority reader first discovers and structurally roots the immutable
record/result chain.  This module then walks that chain oldest-first and reruns
artifact resolution, manifest equality, timed-evidence binding and the decision
derivation for every retained record.  A retained result is evidence of what a
historical gate said; it is never trusted as the source of predecessor states.

Historical gate identity remains explicit.  Replay uses the current gate's
semantics and therefore does not claim that a different historical gate was
semantically identical.  Structural replay findings block the current decision;
ordinary historical blockers (for example the mandatory bootstrap BLOCK) do
not, because they are precisely the obligations a successor is repairing.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any

from ai_contracts import Finding, validate_json_schema
from recurrence_v2_authority import ChainLink, ChainResult, generation_view
from recurrence_v2_decision import derive
from recurrence_v2_evidence import (
    bind_timed_validations,
    check_reference_integrity,
    load_manifest,
    resolve_artifacts,
    verify_canonical_port_packet,
)
from recurrence_v2_git import GitError, GitRepo
from recurrence_v2_model import LegacyRoot, RecordModel, error, parse_record, path_components_safe
from recurrence_v2_topology import verify_generation


@dataclass(frozen=True)
class HistoryReplay:
    findings: tuple[Finding, ...]
    generations: tuple[dict[str, Any], ...]
    chain: ChainResult


def _prefixed(rows: list[Finding], prefix: str) -> list[Finding]:
    return [
        Finding(row.level, row.code, f"{prefix}.{row.path}", row.message)
        for row in rows
    ]


def _load_json_blob(
    repo: GitRepo,
    commit: str,
    path: str,
    location: str,
) -> tuple[dict[str, Any] | None, list[Finding]]:
    if not path_components_safe(path):
        return None, [error("history-artifact-path-invalid", location, repr(path))]
    try:
        payload = repo.blob(commit, path.encode("utf-8"))
    except GitError as exc:
        return None, [error("history-artifact-unreadable", location, str(exc))]
    try:
        raw = json.loads(payload.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        return None, [error("history-artifact-not-json", location, str(exc))]
    if not isinstance(raw, dict):
        return None, [
            error("history-artifact-not-object", location, type(raw).__name__)
        ]
    return raw, []


def _predecessor_path_presence(
    repo: GitRepo,
    record: RecordModel,
) -> tuple[dict[str, bool], list[Finding]]:
    present: dict[str, bool] = {}
    findings: list[Finding] = []
    for index, derivation in enumerate(record.derivations):
        if not path_components_safe(derivation.path):
            findings.append(
                error(
                    "history-derivation-path-invalid",
                    f"record.derivations[{index}].path",
                    repr(derivation.path),
                )
            )
            continue
        try:
            entry = repo.entry(
                record.subject.predecessor_candidate_revision,
                derivation.path.encode("utf-8"),
            )
        except GitError as exc:
            findings.append(
                error(
                    "history-derivation-predecessor-unreadable",
                    f"record.derivations[{index}].path",
                    str(exc),
                )
            )
            continue
        present[derivation.path] = entry is not None
    return present, findings


def _check_retained_result_replay(
    result: dict[str, Any],
    record: RecordModel,
    link: ChainLink,
    *,
    structural_findings: list[Finding],
    decision_blockers: list[str],
) -> list[Finding]:
    """Compare authority-relevant result claims with current derived facts."""

    findings: list[Finding] = []
    expected_decision = (
        "ALLOW"
        if not structural_findings
        and not decision_blockers
        and record.review_decision == "accept"
        else "BLOCK"
    )
    checks = (
        ("control_id", result.get("control_id"), record.control_id),
        ("generation", result.get("generation"), record.generation),
        ("record_sha256", result.get("record_sha256"), link.record_sha256),
        ("review_decision", result.get("review_decision"), record.review_decision),
        ("integration_decision", result.get("integration_decision"), expected_decision),
    )
    for field, actual, expected in checks:
        if actual != expected:
            findings.append(
                error(
                    "retained-result-replay-mismatch",
                    f"result.{field}",
                    f"retained {actual!r}, current semantic replay derives {expected!r}",
                )
            )
    candidate = result.get("candidate")
    expected_candidate = {
        "predecessor_candidate_revision": (
            record.subject.predecessor_candidate_revision
        ),
        "candidate_revision": record.subject.candidate_revision,
        "candidate_tree": record.subject.candidate_tree,
        "patch_sha256": record.subject.patch_sha256,
        "changed_paths": sorted(record.subject.changed_paths),
    }
    if candidate != expected_candidate:
        findings.append(
            error(
                "retained-result-candidate-mismatch",
                "result.candidate",
                "retained candidate identity does not equal the replayed record",
            )
        )
    return findings


def revalidate_history(
    repo: GitRepo,
    chain: ChainResult,
    *,
    schema: dict[str, Any] | None,
    manifest_schema: dict[str, Any] | None,
    timed_schema: dict[str, Any] | None,
    legacy_roots: dict[str, LegacyRoot],
    discovery_valid: bool = True,
    enforce_canonical_ports: bool = False,
) -> HistoryReplay:
    """Replay retained generations oldest-first with a bounded progressive chain."""

    findings: list[Finding] = []
    summaries: list[dict[str, Any]] = []
    progressive_links: tuple[ChainLink, ...] = ()
    progressive_verified: frozenset[str] = frozenset()
    prefix_valid = discovery_valid and chain.rooted

    for link in reversed(chain.links):
        prefix = f"history.generation_{link.generation}"
        raw, load_findings = _load_json_blob(
            repo, link.retention_commit, link.record_path, f"{prefix}.record"
        )
        result, result_load_findings = _load_json_blob(
            repo, link.retention_commit, link.result_path, f"{prefix}.result"
        )
        findings.extend(load_findings)
        findings.extend(result_load_findings)
        if raw is None or result is None:
            continue

        local: list[Finding] = []
        if schema is not None:
            local.extend(validate_json_schema(raw, schema, "record"))
        record, parse_findings = parse_record(raw, "record")
        local.extend(parse_findings)
        if record is None:
            findings.extend(_prefixed(local, prefix))
            continue

        predecessor_chain = ChainResult(
            rooted=prefix_valid,
            root_arm=chain.root_arm,
            root_commit=chain.root_commit,
            links=progressive_links,
            verified_commits=progressive_verified,
            validator_divergences=chain.validator_divergences,
            depth=len(progressive_links) + 1,
        )
        view = generation_view(record)
        topology = None
        if view is None:
            local.append(
                error(
                    "history-link-artifact-reference-unresolved",
                    "record.subject",
                    "packet, manifest, review-packet and report references must resolve",
                )
            )
        else:
            topology, topology_findings = verify_generation(repo, view, "record")
            local.extend(topology_findings)
        artifacts, artifact_findings = resolve_artifacts(
            repo,
            record,
            predecessor_chain,
            legacy_roots,
            location="record.artifacts",
        )
        local.extend(artifact_findings)
        if enforce_canonical_ports:
            local.extend(
                verify_canonical_port_packet(
                    repo, record, artifacts, record.subject.authority.bootstrap_arm
                )
            )
        local.extend(check_reference_integrity(record, artifacts, "record"))
        manifest, manifest_findings = load_manifest(
            repo,
            record,
            artifacts,
            manifest_schema,
            location="manifest",
        )
        local.extend(manifest_findings)
        timed, timed_findings = bind_timed_validations(
            repo, record, manifest, artifacts, timed_schema
        )
        local.extend(timed_findings)
        predecessor_present, path_findings = _predecessor_path_presence(repo, record)
        local.extend(path_findings)
        decision = derive(
            record,
            manifest=manifest,
            chain=predecessor_chain,
            timed=timed,
            topology=topology,
            predecessor_path_present=predecessor_present,
        )
        local.extend(decision.findings)
        replay_findings = _check_retained_result_replay(
            result,
            record,
            link,
            structural_findings=local,
            decision_blockers=decision.blockers,
        )
        local.extend(replay_findings)
        findings.extend(_prefixed(local, prefix))
        summaries.append(
            {
                "generation": record.generation,
                "candidate_revision": record.subject.candidate_revision,
                "current_gate_structurally_valid": not local,
                "historical_blocker_count": len(decision.blockers),
                "retained_integration_decision": result.get("integration_decision"),
                "current_gate_replayed_integration_decision": (
                    "ALLOW"
                    if not local
                    and not decision.blockers
                    and record.review_decision == "accept"
                    else "BLOCK"
                ),
            }
        )
        if not local and prefix_valid:
            progressive_links = (link, *progressive_links)
            progressive_verified = frozenset(
                set(progressive_verified) | set(link.verified_commits)
            )
        else:
            prefix_valid = False

    verified_chain = ChainResult(
        rooted=prefix_valid,
        root_arm=chain.root_arm,
        root_commit=chain.root_commit,
        links=progressive_links,
        verified_commits=progressive_verified,
        validator_divergences=chain.validator_divergences,
        depth=len(progressive_links) + (1 if chain.root_commit is not None else 0),
    )
    return HistoryReplay(tuple(findings), tuple(summaries), verified_chain)
