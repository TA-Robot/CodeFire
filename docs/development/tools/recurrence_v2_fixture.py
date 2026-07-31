#!/usr/bin/env python3
"""Synthetic recurrence v2 histories for the tool tests.

Test-only helper: no production path imports this module.  It builds throwaway
Git histories in a temporary directory so every adversarial case is exercised
against real Git objects without touching the real Cargo or KGI/KPI histories.

`build_bootstrap` accepts one named `variant` per forgery so each topology
counterexample is one readable switch instead of a hand-copied history.
"""

from __future__ import annotations

import hashlib
import json
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from recurrence_v2_git import HERMETIC_CONFIG, PATCH_FLAGS
from recurrence_v2_model import LegacyRoot

# A well-formed, clean deciding-gate identity.  Tests inject it so gate
# behaviour is not coupled to whether this worktree happens to be committed;
# the real identity computation is exercised separately.
CLEAN_GATE: dict[str, Any] = {
    "source_root": "/synthetic/validator",
    "source_revision": "f" * 40,
    "source_tree": "e" * 40,
    "state": "clean_committed",
    "artifacts": [
        {
            "path": "docs/development/tools/check_review_recurrence.py",
            "mode": "100644",
            "sha256": "a" * 64,
            "committed_sha256": "a" * 64,
        }
    ],
}


def clean_gate(repo: "FixtureRepo", revision: str) -> dict[str, Any]:
    """Return a gate identity backed by an exact synthetic Git blob."""

    entry = repo.git("ls-tree", revision, "--", GATE_PATH).split()
    if len(entry) < 3:
        raise ValueError(f"synthetic gate is absent at {revision}")
    return {
        "source_root": str(repo.root),
        "source_revision": revision,
        "source_tree": repo.git("rev-parse", f"{revision}^{{tree}}"),
        "state": "clean_committed",
        "artifacts": [
            {
                "path": GATE_PATH,
                "mode": entry[0],
                "sha256": repo.blob_digest(revision, GATE_PATH),
                "committed_sha256": repo.blob_digest(revision, GATE_PATH),
            }
        ],
    }

VARIANTS = (
    "standard",
    "late_packet",
    "extra_packet_path",
    "mutate_revert",
    "control_rewrites_gate",
    "symlink_packet",
    "symlink_report",
    "symlink_evidence",
    "evidence_out_of_order",
    "invalid_utf8",
    "nonlinear_repair",
    "executable_report",
    "manifest_thin_prior",
    "manifest_thin_claim",
    "manifest_thin_validation",
    "byte_identical_locus",
)


def sha256_text(payload: bytes | str) -> str:
    if isinstance(payload, str):
        payload = payload.encode("utf-8")
    return hashlib.sha256(payload).hexdigest()


def artifact(ref_id: str, role: str, commit: str, path: str, digest: str) -> dict[str, Any]:
    return {
        "ref_id": ref_id,
        "role": role,
        "commit": commit,
        "path": path,
        "sha256": digest,
        "mode": "100644",
    }


@dataclass
class Built:
    record: dict[str, Any]
    roots: dict[str, LegacyRoot]
    commits: dict[str, str] = field(default_factory=dict)
    paths: dict[str, str] = field(default_factory=dict)


class FixtureRepo:
    """Minimal Git wrapper for building synthetic histories."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.name", "Recurrence V2 Fixture")
        self.git("config", "user.email", "recurrence-v2@example.invalid")

    def git(self, *args: str) -> str:
        done = subprocess.run(
            ["git", *args],
            cwd=self.root,
            check=True,
            capture_output=True,
            text=True,
        )
        return done.stdout.strip()

    def write(self, relative: str, content: bytes | str) -> None:
        target = self.root / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(content, str):
            content = content.encode("utf-8")
        target.write_bytes(content)

    def commit(self, message: str, paths: list[str]) -> str:
        self.git("add", "-A", "--", *paths)
        self.git("commit", "-qm", message)
        return self.git("rev-parse", "HEAD")

    def digest(self, relative: str) -> str:
        return sha256_text((self.root / relative).read_bytes())

    def blob_digest(self, commit: str, relative: str) -> str:
        payload = subprocess.run(
            ["git", "show", f"{commit}:{relative}"],
            cwd=self.root,
            check=True,
            capture_output=True,
        ).stdout
        return sha256_text(payload)

    def changed(self, base: str, head: str) -> list[str]:
        raw = subprocess.run(
            ["git", "diff", "--name-only", "-z", "--no-renames", base, head, "--"],
            cwd=self.root,
            check=True,
            capture_output=True,
        ).stdout
        return sorted(
            item.decode("utf-8", "backslashreplace")
            for item in raw.split(b"\0")
            if item
        )

    def patch_digest(self, base: str, head: str) -> str:
        payload = subprocess.run(
            ["git", *HERMETIC_CONFIG, "diff", *PATCH_FLAGS, base, head, "--"],
            cwd=self.root,
            check=True,
            capture_output=True,
        ).stdout
        return sha256_text(payload)


def timed_result(
    *,
    argv: list[str],
    revision: str,
    tree: str,
    log_bytes: bytes,
    schema: str = "codefire.timed_gate.v2",
    clean_before: bool = True,
    clean_after: bool = True,
    index_flags_clean: bool = True,
    status: str = "passed",
    exit_code: int = 0,
) -> dict[str, Any]:
    return {
        "schema": schema,
        "argv": argv,
        "cwd": "/synthetic/checkout",
        "timeout_seconds": 600.0,
        "timed_out": False,
        "exit_code": exit_code,
        "status": status,
        "started_at": "2026-07-31T00:00:00.000000Z",
        "ended_at": "2026-07-31T00:00:01.000000Z",
        "duration_seconds": 1.0,
        "duration_source": "time.monotonic_ns",
        "wall_elapsed_seconds": 1.0,
        "wall_minus_monotonic_seconds": 0.0,
        "wall_clock_discontinuity": False,
        "drift_threshold_seconds": 2.0,
        "source": {
            "git_root": "/synthetic/checkout",
            "revision": revision,
            "tree": tree,
            "derived": "git -C <cwd> rev-parse --show-toplevel HEAD HEAD^{tree}",
        },
        "worktree": {
            "clean_before": clean_before,
            "clean_after": clean_after,
            "status_before": [] if clean_before else [" M product.txt"],
            "status_after": [] if clean_after else ["?? scratch.tmp"],
            "index_flagged_entries": [] if index_flags_clean else ["S\tproduct.txt"],
            "index_flags_clean": index_flags_clean,
            "head_revision_after": revision,
            "head_tree_after": tree,
            "head_unchanged": True,
            "head_probe_error": None,
            "log_inside_source_worktree": False,
            "result_inside_source_worktree": False,
            "status_command": "git status --porcelain=v1 -z --untracked-files=all",
            "index_command": "git ls-files -v -z",
        },
        "log": {
            "path": "/synthetic/logs/focused.log",
            "bytes": len(log_bytes),
            "sha256": sha256_text(log_bytes),
            "complete": True,
        },
        "limits": [
            "A command that mutates and restores bytes during its own execution stays "
            "outside this provenance proof."
        ],
    }


PACKET_PATH = "docs/development/ai/task-packets/synthetic-repair.md"
MANIFEST_PATH = "docs/development/ai/task-packets/synthetic-repair.manifest.json"
REVIEW_PACKET_PATH = "docs/development/ai/review-packets/synthetic-review.md"
REPORT_PATH = "docs/development/ai/swarm-results/synthetic-successor-review/review.md"
LEGACY_REPORT_PATH = (
    "docs/development/ai/swarm-results/synthetic-legacy-rereview/review.md"
)
GATE_PATH = "docs/development/tools/check_review_recurrence.py"
ARGV = ["python3", "-m", "unittest", "focused"]


def build_bootstrap(repo: FixtureRepo, variant: str = "standard") -> Built:
    """One bootstrap generation whose authority is a synthetic pinned root."""

    if variant not in VARIANTS:
        raise ValueError(f"unknown fixture variant: {variant}")
    control_id = "cfr168-v2-gen1"
    repo.write("README.md", "base\n")
    repo.write(GATE_PATH, "# synthetic gate stand-in\n")
    predecessor_paths = ["README.md", GATE_PATH]
    if variant == "byte_identical_locus":
        repo.write("product.txt", "stable locus\nbefore repair\n")
        predecessor_paths.append("product.txt")
    predecessor = repo.commit("predecessor candidate", predecessor_paths)

    repo.write(LEGACY_REPORT_PATH, "legacy rereview REQUEST_CHANGES\n")
    legacy_commit = repo.commit("legacy report", [LEGACY_REPORT_PATH])
    legacy_digest = repo.digest(LEGACY_REPORT_PATH)
    roots = {
        "cargo": LegacyRoot(
            arm="cargo",
            predecessor_candidate=predecessor,
            commit=legacy_commit,
            report_path=LEGACY_REPORT_PATH,
            report_sha256=legacy_digest,
        )
    }

    repo.git("checkout", "-q", "-b", "repair", predecessor)
    packet_paths = [PACKET_PATH, MANIFEST_PATH]
    if variant == "extra_packet_path":
        packet_paths.append("docs/development/ai/task-packets/synthetic-extra.md")

    def write_packet_set() -> str:
        repo.write(PACKET_PATH, "# Synthetic repair packet\n")
        if variant == "symlink_packet":
            (repo.root / PACKET_PATH).unlink()
            (repo.root / PACKET_PATH).symlink_to("../../../../README.md")
        if variant == "extra_packet_path":
            repo.write(packet_paths[2], "# extra manager control artifact\n")
        manifest = {
            "schema": "codefire.review_recurrence_repair_manifest.v1",
            "manifest_id": "synthetic-manifest-1",
            "generation": 1,
            "predecessor_candidate_revision": predecessor,
            "predecessor_authority": {
                "kind": "legacy_bootstrap_report",
                "bootstrap_arm": "cargo",
                "commit": legacy_commit,
            },
            "repair_packet_path": PACKET_PATH,
            "repair_packet_sha256": repo.digest(PACKET_PATH),
            "finding_target_policy": "none",
            "prior_obligations": [
                {"obligation_id": "OBL-1", "closure_validation_ids": ["focused"]}
            ],
            "parity_closure_claims": [
                {
                    "claim_id": "CLAIM-1",
                    "kind": "parity",
                    "obligation_id": "OBL-1",
                    "locus_path": "product.txt",
                }
            ],
            "required_validations": [{"validation_id": "focused", "argv": ARGV}],
        }
        if variant == "manifest_thin_prior":
            manifest["prior_obligations"].clear()
        elif variant == "manifest_thin_claim":
            manifest["parity_closure_claims"].clear()
        elif variant == "manifest_thin_validation":
            manifest["required_validations"].clear()
        repo.write(MANIFEST_PATH, json.dumps(manifest, indent=2, sort_keys=True) + "\n")
        return repo.commit("repair packet and manifest", packet_paths)

    if variant == "late_packet":
        repo.write("product.txt", "repaired\n")
        repo.commit("repair", ["product.txt"])
        packet_commit = write_packet_set()
        candidate = packet_commit
        # The packet is committed at/after the repair, so its parent is a repair
        # commit and the candidate carries no repair after it.
        repo.write("product.txt", "repaired again\n")
        candidate = repo.commit("repair after the packet", ["product.txt"])
    else:
        packet_commit = write_packet_set()
        if variant == "mutate_revert":
            repo.write(PACKET_PATH, "# rewritten packet\n")
            repo.commit("intermediate packet rewrite", [PACKET_PATH])
            repo.write(PACKET_PATH, "# Synthetic repair packet\n")
            repo.commit("restore the packet", [PACKET_PATH])
        if variant == "invalid_utf8":
            repo.write("product\udcff.txt".encode("utf-8", "surrogateescape").decode(
                "utf-8", "surrogateescape"
            ), "repaired\n")
        repo.write(
            "product.txt",
            (
                "stable locus\nafter repair\n"
                if variant == "byte_identical_locus"
                else "repaired\n"
            ),
        )
        candidate = repo.commit("repair", ["."])
        if variant == "nonlinear_repair":
            repo.git("checkout", "-q", "-b", "side", packet_commit)
            repo.write("side.txt", "side\n")
            repo.commit("side repair", ["side.txt"])
            repo.git("checkout", "-q", "repair")
            repo.git("merge", "-q", "--no-ff", "-m", "merge side repair", "side")
            candidate = repo.git("rev-parse", "HEAD")
    candidate_tree = repo.git("rev-parse", f"{candidate}^{{tree}}")

    repo.write(REVIEW_PACKET_PATH, "# Synthetic review packet\n")
    control_paths = [REVIEW_PACKET_PATH]
    if variant == "control_rewrites_gate":
        repo.write(GATE_PATH, "# gate rewritten by review control\n")
        control_paths.append(GATE_PATH)
    control_commit = repo.commit("review control", control_paths)

    if variant == "symlink_report":
        target = repo.root / REPORT_PATH
        target.parent.mkdir(parents=True, exist_ok=True)
        target.symlink_to(repo.root / "README.md")
    else:
        repo.write(REPORT_PATH, "# Synthetic successor review\n")
    report_commit = repo.commit("review report", [REPORT_PATH])
    if variant == "executable_report":
        repo.git("update-index", "--chmod=+x", REPORT_PATH)
        repo.git("commit", "-qm", "make the report executable")
        report_commit = repo.git("rev-parse", "HEAD")

    evidence_dir = f"docs/development/ai/review-recurrence/evidence/{control_id}"
    log_path = f"{evidence_dir}/focused.log"
    result_path = f"{evidence_dir}/focused.result.json"
    event_path = f"{evidence_dir}/terminal-events.log"
    log_bytes = b"focused validation ok\n"
    if variant == "evidence_out_of_order":
        repo.git("checkout", "-q", control_commit)
        repo.git("checkout", "-q", "-b", "evidence-first")
    repo.write(log_path, log_bytes)
    if variant == "symlink_evidence":
        (repo.root / log_path).unlink()
        (repo.root / log_path).symlink_to(repo.root / "README.md")
    repo.write(
        result_path,
        json.dumps(
            timed_result(
                argv=ARGV,
                revision=candidate,
                tree=candidate_tree,
                log_bytes=log_bytes,
            ),
            indent=2,
            sort_keys=True,
        )
        + "\n",
    )
    repo.write(event_path, "worker stopped; zero children\n")
    evidence_commit = repo.commit("evidence", [log_path, result_path, event_path])

    changed = repo.changed(predecessor, candidate)
    record = {
        "schema": "codefire.review_recurrence_control.v2",
        "control_id": control_id,
        "generation": 1,
        "finding_target_policy": "none",
        "predecessor_obligation_completeness": "unavailable",
        "subject": {
            "predecessor_candidate_revision": predecessor,
            "candidate_revision": candidate,
            "candidate_tree": candidate_tree,
            "patch_sha256": repo.patch_digest(predecessor, candidate),
            "changed_paths": changed,
            "repair_packet_commit": packet_commit,
            "review_control_commit": control_commit,
            "report_commit": report_commit,
            "evidence_commit": evidence_commit,
            "repair_packet_ref": "packet",
            "repair_manifest_ref": "manifest",
            "review_packet_ref": "review-packet",
            "report_ref": "report",
            "predecessor_authority": {
                "kind": "legacy_bootstrap_report",
                "bootstrap_arm": "cargo",
                "commit": legacy_commit,
                "artifact_refs": ["legacy-root"],
            },
        },
        "artifacts": [
            artifact(
                "packet",
                "repair_packet",
                packet_commit,
                PACKET_PATH,
                repo.blob_digest(packet_commit, PACKET_PATH),
            ),
            artifact(
                "manifest",
                "repair_manifest",
                packet_commit,
                MANIFEST_PATH,
                repo.blob_digest(packet_commit, MANIFEST_PATH),
            ),
            artifact(
                "review-packet",
                "review_packet",
                control_commit,
                REVIEW_PACKET_PATH,
                repo.blob_digest(control_commit, REVIEW_PACKET_PATH),
            ),
            artifact(
                "report",
                "report",
                report_commit,
                REPORT_PATH,
                repo.blob_digest(report_commit, REPORT_PATH),
            ),
            artifact(
                "timed-result",
                "timed_result",
                evidence_commit,
                result_path,
                repo.blob_digest(evidence_commit, result_path),
            ),
            artifact(
                "timed-log",
                "timed_log",
                evidence_commit,
                log_path,
                sha256_text(log_bytes),
            ),
            artifact(
                "terminal-events",
                "terminal_event_evidence",
                evidence_commit,
                event_path,
                repo.blob_digest(evidence_commit, event_path),
            ),
            artifact(
                "legacy-root",
                "legacy_bootstrap_report",
                legacy_commit,
                LEGACY_REPORT_PATH,
                legacy_digest,
            ),
        ],
        "path_reviews": [
            {
                "path": path,
                "disposition": "reviewed",
                "surfaces": [
                    {
                        "surface_id": f"S1-{index}",
                        "kind": "documentation" if path.startswith("docs/") else "product",
                        "scope_disposition": "reviewed_in_scope",
                        "evidence_ref_ids": ["report"],
                        "scope_routing_ref": None,
                    }
                ],
            }
            for index, path in enumerate(changed)
        ],
        "prior_obligations": [
            {
                "id": "OBL-1",
                "origin_generation": 0,
                "evidence_state": "DEFECT_REPRODUCED",
                "next_action": "PRODUCT_REPAIR",
                "closure_evidence_ref": None,
                "non_reachability_ref": None,
                "routing_decision_ref": None,
                "observations": {
                    "before_ref_ids": ["report"],
                    "after_ref_ids": ["timed-result"],
                    "negative_control": "killed",
                    "negative_control_ref_ids": ["timed-result"],
                },
            }
        ],
        "new_findings": [
            {
                "id": "NF-1",
                "evidence_state": "CLOSURE_UNPROVEN",
                "next_action": "RUN_CLOSURE_EVIDENCE",
                "routing_decision_ref": None,
                "observations": {
                    "before_ref_ids": ["report"],
                    "after_ref_ids": [],
                    "negative_control": "not_applicable",
                    "negative_control_ref_ids": ["report"],
                },
            }
        ],
        "derivations": [
            {
                "claim_id": "CLAIM-1",
                "path": "product.txt",
                "locus": "product.txt:final_fence",
                "locus_existed_in_predecessor": variant == "byte_identical_locus",
                "predecessor_behaviour": "the locus is absent from the predecessor tree",
                "candidate_behaviour": "the candidate introduces the repaired fence",
                "evidence": {"state": "available", "ref_ids": ["report"]},
                "disposition": "derivation_complete",
                "next_action": "NONE",
            }
        ],
        "validations": [
            {
                "id": "focused",
                "result_ref": "timed-result",
                "log_ref": "timed-log",
                "status": "passed",
                "exit_code": 0,
                "timed_out": False,
                "log_complete": True,
            }
        ],
        "terminalization": {
            "state": "complete",
            "worker_alive": False,
            "child_processes_active": 0,
            "event_evidence_ref": "terminal-events",
        },
        "review_decision": "accept",
    }
    return Built(
        record=record,
        roots=roots,
        commits={
            "predecessor": predecessor,
            "legacy_root": legacy_commit,
            "packet": packet_commit,
            "candidate": candidate,
            "control": control_commit,
            "report": report_commit,
            "evidence": evidence_commit,
        },
        paths={
            "packet": PACKET_PATH,
            "manifest": MANIFEST_PATH,
            "review_packet": REVIEW_PACKET_PATH,
            "report": REPORT_PATH,
            "legacy_report": LEGACY_REPORT_PATH,
            "timed_result": result_path,
            "timed_log": log_path,
            "terminal_events": event_path,
        },
    )


SECOND_PACKET_PATH = "docs/development/ai/task-packets/synthetic-repair-2.md"
SECOND_MANIFEST_PATH = (
    "docs/development/ai/task-packets/synthetic-repair-2.manifest.json"
)
SECOND_REVIEW_PACKET_PATH = "docs/development/ai/review-packets/synthetic-review-2.md"
SECOND_REPORT_PATH = (
    "docs/development/ai/swarm-results/synthetic-successor-review-2/review.md"
)
RECORD_RETENTION_PATH = "docs/development/ai/review-recurrence/records/gen1.json"
RESULT_RETENTION_PATH = "docs/development/ai/review-recurrence/results/gen1.json"


def retain_generation(
    repo: FixtureRepo,
    first: Built,
    first_payload: str,
    first_result: dict[str, Any],
    *,
    third_path: bool = False,
    parent: str | None = None,
    result_mutation: dict[str, Any] | None = None,
) -> tuple[str, str]:
    """Commit generation one's record and result, optionally malformed."""

    repo.git("checkout", "-q", parent or first.commits["evidence"])
    repo.git("checkout", "-q", "-B", "retain")
    result_payload = json.dumps(
        {**first_result, **(result_mutation or {})}, indent=2, sort_keys=True
    ) + "\n"
    repo.write(RECORD_RETENTION_PATH, first_payload)
    repo.write(RESULT_RETENTION_PATH, result_payload)
    paths = [RECORD_RETENTION_PATH, RESULT_RETENTION_PATH]
    if third_path:
        repo.write("docs/development/ai/review-recurrence/records/extra.json", "{}\n")
        paths.append("docs/development/ai/review-recurrence/records/extra.json")
    retention = repo.commit("retain generation one", paths)
    return retention, result_payload


def build_second(
    repo: FixtureRepo,
    first: Built,
    first_payload: str,
    first_result: dict[str, Any],
    *,
    retention: str | None = None,
    result_payload: str | None = None,
    generation: int = 2,
    injection_empty_mapping: bool = False,
    bind_non_reachability: bool = False,
) -> Built:
    """The rooted second generation, where machine ALLOW first becomes reachable."""

    if retention is None or result_payload is None:
        retention, result_payload = retain_generation(
            repo, first, first_payload, first_result
        )
    control_id = "cfr168-v2-gen2"
    predecessor = first.record["subject"]["candidate_revision"]
    repo.git("checkout", "-q", predecessor)
    repo.git("checkout", "-q", "-B", "repair2")
    repo.write(SECOND_PACKET_PATH, "# Synthetic second repair packet\n")
    manifest = {
        "schema": "codefire.review_recurrence_repair_manifest.v1",
        "manifest_id": "synthetic-manifest-2",
        "generation": generation,
        "predecessor_candidate_revision": predecessor,
        "predecessor_authority": {
            "kind": "prior_v2_record",
            "bootstrap_arm": None,
            "commit": retention,
        },
        "repair_packet_path": SECOND_PACKET_PATH,
        "repair_packet_sha256": repo.digest(SECOND_PACKET_PATH),
        "finding_target_policy": "none",
        "prior_obligations": [
            {"obligation_id": "NF-1", "closure_validation_ids": ["focused"]},
            {
                "obligation_id": "OBL-1",
                "closure_validation_ids": (
                    [] if injection_empty_mapping else ["focused"]
                ),
            },
        ],
        "parity_closure_claims": [
            {
                "claim_id": "CLAIM-2",
                "kind": "closure",
                "obligation_id": "OBL-1",
                "locus_path": "product.txt",
            }
        ],
        "required_validations": [{"validation_id": "focused", "argv": ARGV}],
    }
    repo.write(
        SECOND_MANIFEST_PATH, json.dumps(manifest, indent=2, sort_keys=True) + "\n"
    )
    packet_commit = repo.commit(
        "second repair packet and manifest", [SECOND_PACKET_PATH, SECOND_MANIFEST_PATH]
    )
    repo.write("product.txt", "repaired twice\n")
    candidate = repo.commit("second repair", ["product.txt"])
    candidate_tree = repo.git("rev-parse", f"{candidate}^{{tree}}")

    repo.write(SECOND_REVIEW_PACKET_PATH, "# Second review packet\n")
    control_commit = repo.commit("review control 2", [SECOND_REVIEW_PACKET_PATH])
    repo.write(SECOND_REPORT_PATH, "# Second successor review\n")
    report_commit = repo.commit("review report 2", [SECOND_REPORT_PATH])

    evidence_dir = f"docs/development/ai/review-recurrence/evidence/{control_id}"
    log_path = f"{evidence_dir}/focused.log"
    result_path = f"{evidence_dir}/focused.result.json"
    event_path = f"{evidence_dir}/terminal-events.log"
    log_bytes = b"second focused validation ok\n"
    repo.write(log_path, log_bytes)
    repo.write(
        result_path,
        json.dumps(
            timed_result(
                argv=ARGV,
                revision=candidate,
                tree=candidate_tree,
                log_bytes=log_bytes,
            ),
            indent=2,
            sort_keys=True,
        )
        + "\n",
    )
    repo.write(event_path, "worker stopped; zero children\n")
    evidence_commit = repo.commit("evidence 2", [log_path, result_path, event_path])

    changed = repo.changed(predecessor, candidate)
    closed_row = {
        "origin_generation": 0,
        "evidence_state": "CLOSED_PROVEN",
        "next_action": "NONE",
        "closure_evidence_ref": None,
        "non_reachability_ref": None,
        "routing_decision_ref": None,
        "observations": {
            "before_ref_ids": ["report"],
            "after_ref_ids": ["timed-result"],
            "negative_control": "killed",
            "negative_control_ref_ids": ["timed-result"],
        },
    }
    record = {
        "schema": "codefire.review_recurrence_control.v2",
        "control_id": control_id,
        "generation": generation,
        "finding_target_policy": "none",
        "predecessor_obligation_completeness": "verified",
        "subject": {
            "predecessor_candidate_revision": predecessor,
            "candidate_revision": candidate,
            "candidate_tree": candidate_tree,
            "patch_sha256": repo.patch_digest(predecessor, candidate),
            "changed_paths": changed,
            "repair_packet_commit": packet_commit,
            "review_control_commit": control_commit,
            "report_commit": report_commit,
            "evidence_commit": evidence_commit,
            "repair_packet_ref": "packet",
            "repair_manifest_ref": "manifest",
            "review_packet_ref": "review-packet",
            "report_ref": "report",
            "predecessor_authority": {
                "kind": "prior_v2_record",
                "bootstrap_arm": None,
                "commit": retention,
                "artifact_refs": ["prior-record", "prior-result"],
            },
        },
        "artifacts": [
            artifact(
                "packet",
                "repair_packet",
                packet_commit,
                SECOND_PACKET_PATH,
                repo.blob_digest(packet_commit, SECOND_PACKET_PATH),
            ),
            artifact(
                "manifest",
                "repair_manifest",
                packet_commit,
                SECOND_MANIFEST_PATH,
                repo.blob_digest(packet_commit, SECOND_MANIFEST_PATH),
            ),
            artifact(
                "review-packet",
                "review_packet",
                control_commit,
                SECOND_REVIEW_PACKET_PATH,
                repo.blob_digest(control_commit, SECOND_REVIEW_PACKET_PATH),
            ),
            artifact(
                "report",
                "report",
                report_commit,
                SECOND_REPORT_PATH,
                repo.blob_digest(report_commit, SECOND_REPORT_PATH),
            ),
            artifact(
                "timed-result",
                "timed_result",
                evidence_commit,
                result_path,
                repo.blob_digest(evidence_commit, result_path),
            ),
            artifact(
                "timed-log", "timed_log", evidence_commit, log_path, sha256_text(log_bytes)
            ),
            artifact(
                "terminal-events",
                "terminal_event_evidence",
                evidence_commit,
                event_path,
                repo.blob_digest(evidence_commit, event_path),
            ),
            artifact(
                "prior-record",
                "prior_v2_record",
                retention,
                RECORD_RETENTION_PATH,
                sha256_text(first_payload),
            ),
            artifact(
                "prior-result",
                "prior_v2_result",
                retention,
                RESULT_RETENTION_PATH,
                sha256_text(result_payload),
            ),
        ],
        "path_reviews": [
            {
                "path": path,
                "disposition": "reviewed",
                "surfaces": [
                    {
                        "surface_id": f"S2-{index}",
                        "kind": "documentation" if path.startswith("docs/") else "product",
                        "scope_disposition": "reviewed_in_scope",
                        "evidence_ref_ids": ["report"],
                        "scope_routing_ref": None,
                    }
                ],
            }
            for index, path in enumerate(changed)
        ],
        "prior_obligations": [
            {**closed_row, "id": "OBL-1", "origin_generation": 0},
            {
                **closed_row,
                "id": "NF-1",
                "origin_generation": 1,
                "observations": {
                    "before_ref_ids": ["report"],
                    "after_ref_ids": ["timed-result"],
                    "negative_control": "not_applicable",
                    "negative_control_ref_ids": ["report"],
                },
            },
        ],
        "new_findings": [],
        "derivations": [
            {
                "claim_id": "CLAIM-2",
                "path": "product.txt",
                "locus": "product.txt:final_fence",
                "locus_existed_in_predecessor": True,
                "predecessor_behaviour": (
                    "the predecessor releases the fence before the terminal check"
                ),
                "candidate_behaviour": "the candidate holds it to the terminal check",
                "evidence": {"state": "available", "ref_ids": ["report"]},
                "disposition": "derivation_complete",
                "next_action": "NONE",
            }
        ],
        "validations": [
            {
                "id": "focused",
                "result_ref": "timed-result",
                "log_ref": "timed-log",
                "status": "passed",
                "exit_code": 0,
                "timed_out": False,
                "log_complete": True,
            }
        ],
        "terminalization": {
            "state": "complete",
            "worker_alive": False,
            "child_processes_active": 0,
            "event_evidence_ref": "terminal-events",
        },
        "review_decision": "accept",
    }
    if bind_non_reachability:
        prior_report = next(
            item for item in first.record["artifacts"] if item["ref_id"] == "report"
        )
        record["artifacts"].append(
            artifact(
                "non-reachability",
                "non_reachability_evidence",
                prior_report["commit"],
                prior_report["path"],
                prior_report["sha256"],
            )
        )
        record["prior_obligations"][0]["non_reachability_ref"] = "non-reachability"
    return Built(
        record=record,
        roots=first.roots,
        commits={
            "predecessor": predecessor,
            "retention": retention,
            "packet": packet_commit,
            "candidate": candidate,
            "control": control_commit,
            "report": report_commit,
            "evidence": evidence_commit,
        },
        paths={
            "packet": SECOND_PACKET_PATH,
            "manifest": SECOND_MANIFEST_PATH,
            "report": SECOND_REPORT_PATH,
            "timed_result": result_path,
            "timed_log": log_path,
            "record_retention": RECORD_RETENTION_PATH,
            "result_retention": RESULT_RETENTION_PATH,
        },
    )
