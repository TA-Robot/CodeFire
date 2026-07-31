#!/usr/bin/env python3
"""Exact Git chronology for one recurrence generation.

One responsibility: prove the presented topology.

```text
predecessor -> packet commit (exactly two paths) -> repair series -> candidate
            -> review control (one path) -> report (one path) -> evidence
            -> [retention, verified later by the successor generation]
```

The same function verifies the current generation and every earlier link the
authority chain walks, so the rule exists once.  Nothing here reads an evidence
state or decides integration, and nothing here encodes how many children the
predecessor candidate has: each declared commit is checked against its own
parent and write set, so an abandoned sibling attempt is invisible rather than
fatal.

Git proves relationships in the history presented to the gate.  It does not
prove when text was authored and cannot detect a rewritten history that
presents a conforming topology.
"""

from __future__ import annotations

from dataclasses import dataclass

from recurrence_v2_git import GitError, GitRepo, PathSet, render_path, sha256_bytes
from recurrence_v2_model import (
    BLOB_MODE,
    EVIDENCE_PREFIX,
    RECORD_PREFIX,
    REPORT_PREFIX,
    RESULT_PREFIX,
    REVIEW_PACKET_PREFIXES,
    TASK_PACKET_PREFIX,
    Finding,
    error,
    path_components_safe,
)


@dataclass(frozen=True)
class GenerationView:
    """Everything one generation's record declares about its own chronology."""

    control_id: str
    predecessor_candidate: str
    candidate: str
    candidate_tree: str
    patch_sha256: str
    changed_paths: frozenset[str]
    packet_commit: str
    packet_path: str
    manifest_path: str
    review_control_commit: str
    review_packet_path: str
    report_commit: str
    report_path: str
    evidence_commit: str
    evidence_paths: frozenset[str]


@dataclass(frozen=True)
class TopologyFacts:
    repair_surface: frozenset[bytes]
    repair_commits: tuple[str, ...]
    inserted_lines: int
    deleted_lines: int
    binary_paths: tuple[str, ...]
    invalid_utf8_paths: tuple[str, ...]

    @property
    def repair_surface_paths(self) -> tuple[str, ...]:
        return tuple(sorted(render_path(item) for item in self.repair_surface))


def scan_interval_paths(
    repo: GitRepo,
    predecessor: str,
    candidate: str,
    location: str,
) -> tuple[tuple[str, ...], list[Finding]]:
    """Detect invalid-UTF-8 repository paths before schema validation.

    The offending bytes are rendered with backslash escapes so the structured
    finding stays JSON-writable; a lone surrogate would not be.
    """

    try:
        changed = repo.changed_paths(
            repo.resolve_commit(predecessor), repo.resolve_commit(candidate)
        )
    except GitError as exc:
        return (), [error("interval-git-binding-failed", location, str(exc))]
    invalid = changed.invalid_utf8
    if invalid:
        return invalid, [
            error(
                "invalid-utf8-git-path",
                f"{location}.changed_paths",
                "Git reports paths whose bytes are not valid UTF-8: "
                + ", ".join(invalid),
            )
        ]
    return (), []


def _write_set(
    repo: GitRepo,
    commit: str,
    expected: frozenset[str],
    code: str,
    location: str,
    findings: list[Finding],
) -> PathSet | None:
    try:
        actual = repo.write_set(commit)
    except GitError as exc:
        findings.append(error(code, location, str(exc)))
        return None
    wanted = frozenset(path.encode("utf-8") for path in expected)
    if actual.raw != wanted:
        findings.append(
            error(
                code,
                location,
                "declared-only="
                + repr(sorted(render_path(item) for item in wanted - actual.raw))
                + ", actual-only="
                + repr(sorted(render_path(item) for item in actual.raw - wanted)),
            )
        )
    return actual


def _blob_mode(
    repo: GitRepo,
    commit: str,
    path: str,
    code: str,
    location: str,
    findings: list[Finding],
) -> None:
    if not path_components_safe(path):
        findings.append(error(code, location, f"unsafe repository path: {path!r}"))
        return
    try:
        entry = repo.entry(commit, path.encode("utf-8"))
    except GitError as exc:
        findings.append(error(code, location, str(exc)))
        return
    if entry is None:
        findings.append(error(code, location, f"{path} is absent at {commit}"))
        return
    if entry.mode != BLOB_MODE or entry.kind != "blob":
        findings.append(
            error(
                code,
                location,
                f"{path} at {commit} is mode {entry.mode} {entry.kind}, not a {BLOB_MODE} blob",
            )
        )


def _single_parent(
    repo: GitRepo,
    child: str,
    parent: str,
    code: str,
    location: str,
    findings: list[Finding],
) -> None:
    try:
        parents = repo.parents(child)
    except GitError as exc:
        findings.append(error(code, location, str(exc)))
        return
    if parents != (parent,):
        findings.append(
            error(
                code,
                location,
                f"parents of {child} are {list(parents)}, expected exactly [{parent}]",
            )
        )


def verify_generation(
    repo: GitRepo,
    view: GenerationView,
    location: str,
) -> tuple[TopologyFacts | None, list[Finding]]:
    findings: list[Finding] = []
    try:
        predecessor = repo.resolve_commit(view.predecessor_candidate)
        candidate = repo.resolve_commit(view.candidate)
        packet_commit = repo.resolve_commit(view.packet_commit)
        control = repo.resolve_commit(view.review_control_commit)
        report = repo.resolve_commit(view.report_commit)
        evidence = repo.resolve_commit(view.evidence_commit)
    except GitError as exc:
        return None, [error("topology-git-binding-failed", location, str(exc))]
    for declared, resolved, field in (
        (view.predecessor_candidate, predecessor, "predecessor_candidate_revision"),
        (view.candidate, candidate, "candidate_revision"),
        (view.packet_commit, packet_commit, "repair_packet_commit"),
        (view.review_control_commit, control, "review_control_commit"),
        (view.report_commit, report, "report_commit"),
        (view.evidence_commit, evidence, "evidence_commit"),
    ):
        if declared != resolved:
            findings.append(
                error(
                    "revision-not-canonical",
                    f"{location}.subject.{field}",
                    f"resolved {resolved}",
                )
            )

    # Packet commit: direct child of the predecessor, exactly the two packet
    # artifacts, no repair byte.  The admissible manager-control set in this
    # pre-repair window is exactly these two paths.
    _single_parent(
        repo,
        packet_commit,
        predecessor,
        "packet-commit-not-direct-child",
        f"{location}.subject.repair_packet_commit",
        findings,
    )
    packet_paths = frozenset({view.packet_path, view.manifest_path})
    if len(packet_paths) != 2:
        findings.append(
            error(
                "packet-and-manifest-share-a-path",
                f"{location}.subject.repair_packet_commit",
                view.packet_path,
            )
        )
    _write_set(
        repo,
        packet_commit,
        packet_paths,
        "packet-commit-write-set-mismatch",
        f"{location}.subject.repair_packet_commit",
        findings,
    )
    for path in sorted(packet_paths):
        if not path.startswith(TASK_PACKET_PREFIX):
            findings.append(
                error(
                    "packet-artifact-outside-namespace",
                    f"{location}.subject.repair_packet_commit",
                    f"{path} is outside {TASK_PACKET_PREFIX}",
                )
            )
        _blob_mode(
            repo,
            packet_commit,
            path,
            "packet-artifact-not-a-regular-blob",
            f"{location}.subject.repair_packet_commit",
            findings,
        )

    # Candidate interval, derived over exactly predecessor..candidate.
    try:
        changed = repo.changed_paths(predecessor, candidate)
        patch = repo.canonical_patch(predecessor, candidate)
        tree = repo.tree_of(candidate)
        inserted, deleted, binary_paths = repo.numstat(predecessor, candidate)
    except GitError as exc:
        return None, findings + [
            error("interval-git-binding-failed", location, str(exc))
        ]
    declared_changed = frozenset(
        path.encode("utf-8") for path in view.changed_paths
    )
    if declared_changed != changed.raw:
        findings.append(
            error(
                "changed-paths-mismatch",
                f"{location}.subject.changed_paths",
                "declared-only="
                + repr(sorted(render_path(i) for i in declared_changed - changed.raw))
                + ", actual-only="
                + repr(sorted(render_path(i) for i in changed.raw - declared_changed)),
            )
        )
    unsafe = sorted(
        render_path(item)
        for item in changed.raw
        if not path_components_safe(render_path(item))
    )
    if unsafe:
        findings.append(
            error("unsafe-changed-path", f"{location}.subject.changed_paths", repr(unsafe))
        )
    if view.candidate_tree != tree:
        findings.append(
            error(
                "candidate-tree-mismatch",
                f"{location}.subject.candidate_tree",
                f"computed {tree}",
            )
        )
    patch_digest = sha256_bytes(patch)
    if view.patch_sha256 != patch_digest:
        findings.append(
            error(
                "candidate-patch-digest-mismatch",
                f"{location}.subject.patch_sha256",
                f"computed {patch_digest}",
            )
        )

    repair_surface = changed.raw - frozenset(
        path.encode("utf-8") for path in packet_paths
    )
    if not repair_surface:
        findings.append(
            error(
                "empty-repair-surface",
                f"{location}.subject.changed_paths",
                "the presented interval contains no repair path outside the two packet artifacts",
            )
        )

    # Every commit after the packet commit is inspected separately: a net diff
    # alone would let an intermediate commit rewrite the packet and restore it.
    repair_commits: tuple[str, ...] = ()
    try:
        repair_commits = repo.linear_series(packet_commit, candidate)
    except GitError as exc:
        findings.append(
            error(
                "repair-series-not-linear",
                f"{location}.subject.candidate_revision",
                str(exc),
            )
        )
    else:
        if not repair_commits:
            findings.append(
                error(
                    "no-repair-commit",
                    f"{location}.subject.candidate_revision",
                    "the candidate is the packet commit itself",
                )
            )
        packet_bytes = frozenset(path.encode("utf-8") for path in packet_paths)
        for commit in repair_commits:
            try:
                touched = repo.write_set(commit)
            except GitError as exc:
                findings.append(
                    error("repair-commit-not-inspectable", f"{location}.repair_series", str(exc))
                )
                continue
            rewritten = touched.raw & packet_bytes
            if rewritten:
                findings.append(
                    error(
                        "repair-commit-rewrites-packet",
                        f"{location}.repair_series",
                        f"{commit} touches "
                        + repr(sorted(render_path(i) for i in rewritten)),
                    )
                )
            outside = touched.raw - repair_surface - packet_bytes
            if outside:
                findings.append(
                    error(
                        "repair-commit-outside-surface",
                        f"{location}.repair_series",
                        f"{commit} touches "
                        + repr(sorted(render_path(i) for i in outside)),
                    )
                )

    # Review control -> report -> evidence, each an exact direct child.
    _single_parent(
        repo,
        control,
        candidate,
        "review-control-not-direct-child",
        f"{location}.subject.review_control_commit",
        findings,
    )
    _write_set(
        repo,
        control,
        frozenset({view.review_packet_path}),
        "review-control-write-set-mismatch",
        f"{location}.subject.review_control_commit",
        findings,
    )
    if not any(
        view.review_packet_path.startswith(prefix) for prefix in REVIEW_PACKET_PREFIXES
    ):
        findings.append(
            error(
                "review-packet-outside-namespace",
                f"{location}.subject.review_packet_commit",
                f"{view.review_packet_path} is outside {list(REVIEW_PACKET_PREFIXES)}",
            )
        )
    _blob_mode(
        repo,
        control,
        view.review_packet_path,
        "review-packet-not-a-regular-blob",
        f"{location}.subject.review_control_commit",
        findings,
    )
    _single_parent(
        repo,
        report,
        control,
        "report-not-direct-child",
        f"{location}.subject.report_commit",
        findings,
    )
    _write_set(
        repo,
        report,
        frozenset({view.report_path}),
        "report-write-set-mismatch",
        f"{location}.subject.report_commit",
        findings,
    )
    if not view.report_path.startswith(REPORT_PREFIX):
        findings.append(
            error(
                "report-outside-namespace",
                f"{location}.subject.report_commit",
                f"{view.report_path} is outside {REPORT_PREFIX}",
            )
        )
    _blob_mode(
        repo,
        report,
        view.report_path,
        "report-not-a-regular-blob",
        f"{location}.subject.report_commit",
        findings,
    )
    _single_parent(
        repo,
        evidence,
        report,
        "evidence-not-direct-child",
        f"{location}.subject.evidence_commit",
        findings,
    )
    _write_set(
        repo,
        evidence,
        view.evidence_paths,
        "evidence-write-set-mismatch",
        f"{location}.subject.evidence_commit",
        findings,
    )
    namespace = f"{EVIDENCE_PREFIX}{view.control_id}/"
    for path in sorted(view.evidence_paths):
        if not path.startswith(namespace):
            findings.append(
                error(
                    "evidence-outside-namespace",
                    f"{location}.subject.evidence_commit",
                    f"{path} is outside {namespace}",
                )
            )
        _blob_mode(
            repo,
            evidence,
            path,
            "evidence-not-a-regular-blob",
            f"{location}.subject.evidence_commit",
            findings,
        )
    facts = TopologyFacts(
        repair_surface=repair_surface,
        repair_commits=repair_commits,
        inserted_lines=inserted,
        deleted_lines=deleted,
        binary_paths=binary_paths,
        invalid_utf8_paths=changed.invalid_utf8,
    )
    return facts, findings


def verify_retention(
    repo: GitRepo,
    *,
    evidence_commit: str,
    retention_commit: str,
    record_path: str,
    result_path: str,
    location: str,
) -> list[Finding]:
    """Retention is a direct child of the evidence commit and writes two paths."""

    findings: list[Finding] = []
    _single_parent(
        repo,
        retention_commit,
        evidence_commit,
        "retention-not-direct-child",
        location,
        findings,
    )
    if record_path == result_path:
        findings.append(
            error("retention-paths-collide", location, record_path)
        )
    _write_set(
        repo,
        retention_commit,
        frozenset({record_path, result_path}),
        "retention-write-set-mismatch",
        location,
        findings,
    )
    if not record_path.startswith(RECORD_PREFIX):
        findings.append(
            error(
                "retained-record-outside-namespace",
                location,
                f"{record_path} is outside {RECORD_PREFIX}",
            )
        )
    if not result_path.startswith(RESULT_PREFIX):
        findings.append(
            error(
                "retained-result-outside-namespace",
                location,
                f"{result_path} is outside {RESULT_PREFIX}",
            )
        )
    for path in (record_path, result_path):
        _blob_mode(
            repo,
            retention_commit,
            path,
            "retained-artifact-not-a-regular-blob",
            location,
            findings,
        )
    return findings
