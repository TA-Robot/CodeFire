from __future__ import annotations

import copy
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parent
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))

from ai_contracts import (  # noqa: E402
    canonical_git_patch,
    load_object,
    packet_digest,
    path_patterns_overlap,
    route_risk,
    validate_context_catalog,
    validate_context_manifest,
    validate_historical_eval_corpus,
    validate_review_against_task,
    validate_review_packet,
    validate_task_packet,
)
from classify_worktree import classify_path  # noqa: E402
from check_ai_acceleration_docs import STALE_PATTERNS, check_live_routing_smoke  # noqa: E402
from check_project_docs import check_current_claims  # noqa: E402
from build_source_bound_rust import (  # noqa: E402
    BINARY_PATH,
    build_source_manifest,
    validate_provenance_payload,
    validate_source_inputs_against_head,
)
from prepare_historical_eval import build_candidate_input  # noqa: E402
from probe_validation_capabilities import rust_binary_state  # noqa: E402
from resolve_context_pack import build_manifest, expand_reference  # noqa: E402
from run_ai_role import preflight_task_execution, resolve_packet_manifest  # noqa: E402
from run_dogfood_smoke import smoke_commands, validate_result  # noqa: E402
from suggest_validation import parse_porcelain_paths, suggestions  # noqa: E402
from validate_ai_runtime import strict_config_probe  # noqa: E402
from test_ai_runtime import RuntimeSeparationTests  # noqa: E402,F401
from test_grok_subscription_preflight import BillingStateTests, TransportTests  # noqa: E402,F401


ROOT = Path(__file__).resolve().parents[3]


class ContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.matrix = load_object(ROOT / "docs/development/ai/role-routing-matrix.json")
        cls.catalog = load_object(ROOT / "docs/development/ai/context-packs.json")
        catalog_findings, cls.context_ids = validate_context_catalog(cls.catalog)
        if catalog_findings:
            raise AssertionError(catalog_findings)
        cls.packet = load_object(
            ROOT / "docs/development/ai/task-packets/json/branch-list-timeout.json"
        )
        cls.task_schema = load_object(
            ROOT / "docs/development/ai/schemas/task-packet.v1.schema.json"
        )
        cls.review_schema = load_object(
            ROOT / "docs/development/ai/schemas/review-packet.v1.schema.json"
        )

    def test_repository_task_packet_is_valid(self) -> None:
        self.assertEqual(
            validate_task_packet(
                self.packet, self.matrix, self.context_ids, "packet", self.task_schema
            ), []
        )

    def test_wrong_score_and_traversal_are_rejected(self) -> None:
        packet = copy.deepcopy(self.packet)
        packet["risk"]["score"] = 1
        packet["scope"]["write_set"].append("../escape")
        codes = {
            finding.code
            for finding in validate_task_packet(
                packet, self.matrix, self.context_ids, "mutated-packet", self.task_schema
            )
        }
        self.assertIn("wrong-risk-score", codes)
        self.assertIn("unsafe-repository-path", codes)

    def test_mandatory_reason_overrides_numeric_score(self) -> None:
        dimensions = {name: 0 for name in self.matrix["risk_routing"]["dimensions"]}
        score, tier = route_risk(
            dimensions, ["canonical_document_family_deletion"], self.matrix
        )
        self.assertEqual(score, 0)
        self.assertEqual(tier, "ultra")

    def test_approval_requires_verified_claims_and_passing_evidence(self) -> None:
        digest = "a" * 64
        review = {
            "schema": "codefire.ai_review_packet.v1",
            "review_id": "review-001",
            "task_packet_id": "task-001",
            "task_packet_sha256": digest,
            "base_revision": "base",
            "head_revision": "head",
            "patch_sha256": "b" * 64,
            "reviewer_independent": True,
            "review_tier": "ultra",
            "files_changed": ["README.md"],
            "claims": [{"claim": "Docs agree", "evidence_refs": ["check"], "verified": True}],
            "validation_evidence": [{"validation_id": "check-01", "command": "check", "status": "passed", "summary": "ok", "log_sha256": "c" * 64}],
            "findings": [],
            "decision": "approve",
        }
        self.assertEqual(validate_review_packet(review, "review", self.review_schema), [])
        review["claims"][0]["verified"] = False
        codes = {
            finding.code
            for finding in validate_review_packet(review, "review", self.review_schema)
        }
        self.assertIn("approval-with-unverified-claim", codes)

    def test_schema_rejects_extra_property_and_wrong_optional_type(self) -> None:
        packet = copy.deepcopy(self.packet)
        packet["extra"] = True
        packet["validation"][0]["requires_capability"] = 42
        codes = {
            finding.code
            for finding in validate_task_packet(
                packet, self.matrix, self.context_ids, "mutated-packet", self.task_schema
            )
        }
        self.assertIn("schema-additional-property", codes)
        self.assertIn("schema-type", codes)

    def test_ready_packet_requires_gate_digest_and_unique_ids(self) -> None:
        packet = copy.deepcopy(self.packet)
        for validation in packet["validation"]:
            validation["required"] = False
        packet["handoff"]["evidence_digest_required"] = False
        packet["invariants"].append(copy.deepcopy(packet["invariants"][0]))
        codes = {
            finding.code
            for finding in validate_task_packet(
                packet, self.matrix, self.context_ids, "mutated-packet", self.task_schema
            )
        }
        self.assertIn("missing-required-validation", codes)
        self.assertIn("evidence-digest-required", codes)
        self.assertIn("duplicate-invariant-id", codes)

    def test_glob_write_and_disallowed_overlap_is_rejected(self) -> None:
        packet = copy.deepcopy(self.packet)
        packet["scope"]["write_set"] = ["docs/**"]
        packet["scope"]["disallowed_paths"] = ["docs/private/**"]
        codes = {
            finding.code
            for finding in validate_task_packet(
                packet, self.matrix, self.context_ids, "mutated-packet", self.task_schema
            )
        }
        self.assertIn("write-disallow-pattern-conflict", codes)

    def test_glob_overlap_fails_closed_without_literal_false_positive(self) -> None:
        self.assertTrue(path_patterns_overlap("[ab]/x", "a/*"))
        self.assertFalse(path_patterns_overlap("docs/a.md", "docs/a.md.bak"))

    def test_review_schema_rejects_empty_evidence_and_extra_property(self) -> None:
        review = {
            "schema": "codefire.ai_review_packet.v1",
            "review_id": "review-002",
            "task_packet_id": "task-002",
            "task_packet_sha256": "a" * 64,
            "base_revision": "base",
            "head_revision": "head",
            "patch_sha256": "b" * 64,
            "reviewer_independent": True,
            "review_tier": "ultra",
            "files_changed": [],
            "claims": [{"claim": "Claim", "evidence_refs": [], "verified": True}],
            "validation_evidence": [],
            "findings": [],
            "decision": "approve",
            "unexpected": True,
        }
        codes = {
            finding.code
            for finding in validate_review_packet(review, "review", self.review_schema)
        }
        self.assertIn("schema-min-items", codes)
        self.assertIn("schema-additional-property", codes)
        self.assertIn("approval-without-validation", codes)


class ContextManifestTests(unittest.TestCase):
    def test_manifest_is_deterministic_and_content_addressed(self) -> None:
        first = build_manifest(
            ROOT,
            ["pack-0-universal-orientation"],
            [],
            False,
            "test-base",
        )
        second = build_manifest(
            ROOT,
            ["pack-0-universal-orientation"],
            [],
            False,
            "test-base",
        )
        self.assertEqual(first, second)
        self.assertTrue(first["files"])
        self.assertTrue(first["manifest_id"].endswith(first["manifest_sha256"][:16]))

    def test_context_glob_rejects_symlink_escape(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / "context").mkdir()
            os.symlink("/etc/passwd", root / "context" / "leak.md")
            with self.assertRaisesRegex(ValueError, "symlink"):
                expand_reference(root, "context/*.md")

    def test_manifest_digest_tampering_is_rejected(self) -> None:
        manifest = build_manifest(
            ROOT,
            ["pack-0-universal-orientation"],
            [],
            False,
            "test-base",
        )
        manifest["manifest_sha256"] = "0" * 64
        schema = load_object(
            ROOT / "docs/development/ai/schemas/context-manifest.v1.schema.json"
        )
        codes = {
            finding.code
            for finding in validate_context_manifest(
                manifest,
                "manifest",
                schema,
                root=ROOT,
                verify_workspace=True,
            )
        }
        self.assertIn("context-manifest-digest-mismatch", codes)

    def test_packet_manifest_binds_existing_explicit_scope_inputs(self) -> None:
        packet = load_object(
            ROOT / "docs/development/ai/task-packets/json/branch-list-timeout.json"
        )
        manifest, unresolved = resolve_packet_manifest(
            ROOT,
            packet,
            verify_workspace=False,
        )
        paths = {item["path"] for item in manifest["files"]}
        self.assertIn("crates/codefire-cli/src/main.rs", paths)
        self.assertIn("crates/codefire-cli/src/cli_model.rs", paths)
        self.assertIn("crates/codefire-store/src/lib.rs", paths)
        self.assertIn(
            "subprojects/algorithm-evolution-agent-lab/.codefire/**",
            unresolved,
        )

    def test_declared_manifest_must_exactly_match_packet_context(self) -> None:
        packet = load_object(
            ROOT / "docs/development/ai/task-packets/json/branch-list-timeout.json"
        )
        packet["context"]["resolved_manifest"] = "docs/development/ai/context-packs.json"
        wrong_manifest = build_manifest(
            ROOT,
            ["pack-0-universal-orientation"],
            [],
            False,
            packet["base_revision"],
        )
        original_load = load_object

        def load_for_test(path: Path) -> dict:
            if path == ROOT / "docs/development/ai/context-packs.json":
                return wrong_manifest
            return original_load(path)

        with mock.patch("run_ai_role.load_object", side_effect=load_for_test):
            with self.assertRaisesRegex(ValueError, "does not exactly bind"):
                resolve_packet_manifest(ROOT, packet, verify_workspace=False)


class ExecutionPreflightTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.packet = load_object(
            ROOT / "docs/development/ai/task-packets/json/branch-list-timeout.json"
        )

    def test_stale_base_and_dirty_write_set_are_blocking(self) -> None:
        matches, dirty, errors = preflight_task_execution(
            self.packet,
            "different-head",
            {"crates/codefire-cli/src/main.rs"},
            "workspace-write",
        )
        self.assertFalse(matches)
        self.assertEqual(dirty, ["crates/codefire-cli/src/main.rs"])
        self.assertEqual(len(errors), 2)

    def test_clean_matching_read_only_preflight_passes(self) -> None:
        matches, dirty, errors = preflight_task_execution(
            self.packet,
            self.packet["base_revision"],
            {"README.md"},
            "read-only",
        )
        self.assertTrue(matches)
        self.assertEqual(dirty, [])
        self.assertEqual(errors, [])


class RustBuildProvenanceTests(unittest.TestCase):
    def test_source_manifest_is_stable_and_covers_workspace_inputs(self) -> None:
        first = build_source_manifest(ROOT)
        second = build_source_manifest(ROOT)
        self.assertEqual(first, second)
        paths = {item["path"] for item in first["files"]}
        self.assertIn("Cargo.toml", paths)
        self.assertIn("crates/codefire-cli/src/main.rs", paths)
        self.assertEqual(len(first["manifest_sha256"]), 64)

    def test_dirty_or_untracked_build_inputs_do_not_match_head(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / "crates/example/src").mkdir(parents=True)
            (root / "Cargo.toml").write_text("[workspace]\nmembers=[]\n", encoding="utf-8")
            (root / "Cargo.lock").write_text("# lock\n", encoding="utf-8")
            source = root / "crates/example/src/lib.rs"
            source.write_text("pub fn value() -> u8 { 1 }\n", encoding="utf-8")
            subprocess.run(["git", "init", "-q"], cwd=root, check=True)
            subprocess.run(["git", "config", "user.email", "ai-test@example.invalid"], cwd=root, check=True)
            subprocess.run(["git", "config", "user.name", "AI Test"], cwd=root, check=True)
            subprocess.run(["git", "add", "."], cwd=root, check=True)
            subprocess.run(["git", "commit", "-qm", "base"], cwd=root, check=True)
            head = subprocess.run(
                ["git", "rev-parse", "HEAD"], cwd=root, check=True, capture_output=True, text=True
            ).stdout.strip()
            self.assertEqual(validate_source_inputs_against_head(root, head), [])
            source.write_text("pub fn value() -> u8 { 2 }\n", encoding="utf-8")
            self.assertTrue(validate_source_inputs_against_head(root, head))
            source.write_text("pub fn value() -> u8 { 1 }\n", encoding="utf-8")
            (root / ".cargo").mkdir()
            (root / ".cargo/config.toml").write_text("[build]\nrustflags=[]\n", encoding="utf-8")
            self.assertTrue(validate_source_inputs_against_head(root, head))

    def test_review_verifies_provenance_payload_source_and_binary(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / "crates/example/src").mkdir(parents=True)
            (root / "Cargo.toml").write_text("[workspace]\nmembers=[]\n", encoding="utf-8")
            (root / "Cargo.lock").write_text("# lock\n", encoding="utf-8")
            (root / "crates/example/src/lib.rs").write_text(
                "pub fn value() -> u8 { 1 }\n", encoding="utf-8"
            )
            subprocess.run(["git", "init", "-q"], cwd=root, check=True)
            subprocess.run(["git", "config", "user.email", "ai-test@example.invalid"], cwd=root, check=True)
            subprocess.run(["git", "config", "user.name", "AI Test"], cwd=root, check=True)
            subprocess.run(["git", "add", "."], cwd=root, check=True)
            subprocess.run(["git", "commit", "-qm", "base"], cwd=root, check=True)
            head = subprocess.run(
                ["git", "rev-parse", "HEAD"], cwd=root, check=True, capture_output=True, text=True
            ).stdout.strip()
            binary = root / BINARY_PATH
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"current binary")
            source_manifest = build_source_manifest(root)
            provenance = {
                "schema": "codefire.rust_build_provenance.v1",
                "ok": True,
                "base_revision": head,
                "source_manifest_sha256": source_manifest["manifest_sha256"],
                "source_file_count": len(source_manifest["files"]),
                "build_command": "cargo build --release --locked -p codefire-cli --bin codefire-rs",
                "binary_path": BINARY_PATH.as_posix(),
                "binary_bytes": len(b"current binary"),
                "binary_sha256": hashlib.sha256(b"current binary").hexdigest(),
            }
            self.assertEqual(
                validate_provenance_payload(root, provenance, expected_revision=head), []
            )
            artifact = root / "target/codefire-ai/provenance.json"
            artifact.parent.mkdir(parents=True, exist_ok=True)
            raw = json.dumps(provenance, sort_keys=True, separators=(",", ":")).encode() + b"\n"
            artifact.write_bytes(raw)
            task = copy.deepcopy(
                load_object(ROOT / "docs/development/ai/task-packets/json/branch-list-timeout.json")
            )
            task["base_revision"] = head
            task["scope"]["write_set"] = []
            task["scope"]["disallowed_paths"] = []
            task["validation"] = [
                {
                    "id": "binary-smoke",
                    "command": "target/release/codefire-rs --help",
                    "purpose": "smoke current binary",
                    "required": True,
                    "requires_capability": "rust-source-bound-binary",
                }
            ]
            review = {
                "task_packet_id": task["packet_id"],
                "task_packet_sha256": packet_digest(task),
                "base_revision": head,
                "head_revision": head,
                "review_tier": "high",
                "files_changed": [],
                "patch_sha256": hashlib.sha256(b"").hexdigest(),
                "decision": "approve",
                "validation_evidence": [
                    {
                        "validation_id": "binary-smoke",
                        "command": "target/release/codefire-rs --help",
                        "status": "passed",
                        "capability_evidence_sha256": hashlib.sha256(raw).hexdigest(),
                        "capability_evidence_path": "target/codefire-ai/provenance.json",
                    }
                ],
            }
            self.assertEqual(
                validate_review_against_task(
                    review,
                    task,
                    "review",
                    patch_bytes=b"",
                    actual_files=set(),
                    resolved_base_revision=head,
                    resolved_head_revision=head,
                    post_validation_capabilities={"rust-source-bound-binary"},
                    root=root,
                ),
                [],
            )
            binary.write_bytes(b"tampered")
            codes = {
                finding.code
                for finding in validate_review_against_task(
                    review,
                    task,
                    "review",
                    patch_bytes=b"",
                    actual_files=set(),
                    resolved_base_revision=head,
                    resolved_head_revision=head,
                    post_validation_capabilities={"rust-source-bound-binary"},
                    root=root,
                )
            }
            self.assertIn("review-rust-provenance-invalid", codes)


class ReviewBindingTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.task = load_object(
            ROOT / "docs/development/ai/task-packets/json/branch-list-timeout.json"
        )

    def review(self) -> dict:
        return {
            "schema": "codefire.ai_review_packet.v1",
            "review_id": "review-binding-001",
            "task_packet_id": self.task["packet_id"],
            "task_packet_sha256": packet_digest(self.task),
            "base_revision": self.task["base_revision"],
            "head_revision": "head",
            "patch_sha256": hashlib.sha256(b"patch").hexdigest(),
            "reviewer_independent": True,
            "review_tier": "high",
            "files_changed": ["crates/codefire-cli/src/main.rs"],
            "claims": [{"claim": "Bound review", "evidence_refs": ["logs"], "verified": True}],
            "validation_evidence": [
                {
                    "validation_id": validation["id"],
                    "command": validation["command"],
                    "status": "passed",
                    "summary": "passed",
                    "log_sha256": "c" * 64,
                    **(
                        {"capability_evidence_sha256": "d" * 64}
                        if validation.get("requires_capability") == "rust-source-bound-binary"
                        else {}
                    ),
                }
                for validation in self.task["validation"]
                if validation["required"]
            ],
            "findings": [],
            "decision": "approve",
        }

    def test_review_binds_task_validations_and_patch(self) -> None:
        self.assertEqual(
            validate_review_against_task(
                self.review(),
                self.task,
                "review",
                patch_bytes=b"patch",
                actual_files={"crates/codefire-cli/src/main.rs"},
                resolved_base_revision=self.task["base_revision"],
                resolved_head_revision="head",
                post_validation_capabilities=set(),
            ),
            [],
        )

    def test_review_rejects_wrong_task_digest_and_missing_gate(self) -> None:
        review = self.review()
        review["task_packet_sha256"] = "0" * 64
        review["validation_evidence"].pop()
        codes = {
            finding.code
            for finding in validate_review_against_task(
                review,
                self.task,
                "review",
                post_validation_capabilities={"rust-source-bound-binary"},
            )
        }
        self.assertIn("review-task-digest-mismatch", codes)
        self.assertIn("review-missing-required-validation", codes)

    def test_approval_requires_canonical_patch_and_exact_git_file_set(self) -> None:
        review = self.review()
        review["files_changed"] = []
        review["patch_sha256"] = "0" * 64
        codes = {
            finding.code
            for finding in validate_review_against_task(
                review,
                self.task,
                "review",
                post_validation_capabilities=set(),
            )
        }
        self.assertIn("approval-patch-unverified", codes)
        self.assertIn("approval-files-unverified", codes)
        self.assertIn("approval-head-unverified", codes)

    def test_block_does_not_require_approval_only_post_validation_evidence(self) -> None:
        review = self.review()
        review["decision"] = "block"
        evidence = next(
            item
            for item in review["validation_evidence"]
            if item["validation_id"] == "dogfood-timeout"
        )
        evidence["status"] = "blocked"
        evidence.pop("capability_evidence_sha256")
        self.assertEqual(
            validate_review_against_task(
                review,
                self.task,
                "review",
                post_validation_capabilities={"rust-source-bound-binary"},
            ),
            [],
        )

    def test_canonical_git_patch_binds_head_and_exact_changed_files(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            target = root / "change.txt"
            target.write_text("base\n", encoding="utf-8")
            subprocess.run(["git", "init", "-q"], cwd=root, check=True)
            subprocess.run(["git", "config", "user.email", "ai-test@example.invalid"], cwd=root, check=True)
            subprocess.run(["git", "config", "user.name", "AI Test"], cwd=root, check=True)
            subprocess.run(["git", "add", "change.txt"], cwd=root, check=True)
            subprocess.run(["git", "commit", "-qm", "base"], cwd=root, check=True)
            base = subprocess.run(
                ["git", "rev-parse", "HEAD"], cwd=root, check=True, capture_output=True, text=True
            ).stdout.strip()
            target.write_text("head\n", encoding="utf-8")
            subprocess.run(["git", "add", "change.txt"], cwd=root, check=True)
            subprocess.run(["git", "commit", "-qm", "head"], cwd=root, check=True)
            head = subprocess.run(
                ["git", "rev-parse", "HEAD"], cwd=root, check=True, capture_output=True, text=True
            ).stdout.strip()
            patch, files, resolved_base, resolved_head = canonical_git_patch(root, base, head)
            self.assertTrue(patch)
            self.assertEqual(files, {"change.txt"})
            self.assertEqual(resolved_base, base)
            self.assertEqual(resolved_head, head)


class FocusedTestMapTests(unittest.TestCase):
    def test_rust_and_ai_changes_deduplicate_shared_commands(self) -> None:
        mapping = load_object(ROOT / "docs/development/ai/focused-test-map.v1.json")
        result = suggestions(
            mapping,
            ["crates/codefire-cli/src/main.rs", "docs/development/ai/README.md"],
        )
        commands = [item["command"] for item in result["commands"]]
        self.assertIn("cargo fmt --check", commands)
        self.assertIn("python3 docs/development/tools/check_ai_development.py", commands)
        self.assertEqual(len(commands), len(set(commands)))
        self.assertEqual(result["uncovered_paths"], [])


class DogfoodSmokeTests(unittest.TestCase):
    def test_read_only_mode_runs_only_status(self) -> None:
        commands = smoke_commands(Path("/repo/codefire"), Path("/repo/dogfood"), False)
        self.assertEqual(len(commands), 1)
        self.assertEqual(commands[0][1], "status")

    def test_mutating_mode_orders_status_scan_verify(self) -> None:
        commands = smoke_commands(Path("/repo/codefire"), Path("/repo/dogfood"), True)
        self.assertEqual([command[1] for command in commands], ["status", "scan", "verify"])

    def test_result_validation_rejects_process_payload_mismatch(self) -> None:
        payload = {
            "schema": "codefire.command_result.v1",
            "command": "status",
            "ok": True,
            "exit_code": 0,
        }
        result = validate_result("status", 2, json.dumps(payload), "")
        self.assertFalse(result["ok"])
        self.assertIn("payload exit_code differs from process return code", result["errors"])


class CapabilityProvenanceTests(unittest.TestCase):
    def test_valid_release_provenance_marks_only_release_binary_bound(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            release = root / BINARY_PATH
            release.parent.mkdir(parents=True)
            release.write_bytes(b"release")
            debug = root / "target/debug/codefire-rs"
            debug.parent.mkdir(parents=True)
            debug.write_bytes(b"debug")
            provenance = root / "target/codefire-ai/rust-build-provenance.json"
            provenance.parent.mkdir(parents=True)
            provenance.write_text(
                json.dumps({"base_revision": "a" * 40}), encoding="utf-8"
            )
            with mock.patch(
                "probe_validation_capabilities.git_head", return_value="a" * 40
            ), mock.patch(
                "probe_validation_capabilities.validate_provenance_payload", return_value=[]
            ):
                state = rust_binary_state(root)
        self.assertTrue(state["source_revision_bound"])
        by_path = {item["path"]: item for item in state["binaries"]}
        self.assertTrue(by_path[BINARY_PATH.as_posix()]["source_revision_bound"])
        self.assertFalse(by_path["target/debug/codefire-rs"]["source_revision_bound"])


class WorktreeClassifierTests(unittest.TestCase):
    def test_ai_config_adr_and_retired_root_docs_are_classified(self) -> None:
        self.assertEqual(classify_path(".codex/config.toml"), "ai_acceleration_docs")
        self.assertEqual(
            classify_path("adr/ADR-008-object-id-digest-length.md"),
            "strategy_architecture_docs",
        )
        self.assertEqual(
            classify_path("CodeFire_Design_Document_v0.2.pdf"),
            "document_cleanup",
        )
    def test_rename_porcelain_keeps_destination_and_source(self) -> None:
        self.assertEqual(
            parse_porcelain_paths(b"R  docs/new.md\0docs/old.md\0"),
            ["docs/new.md", "docs/old.md"],
        )

    def test_all_ai_tools_trigger_complete_gate(self) -> None:
        mapping = load_object(ROOT / "docs/development/ai/focused-test-map.v1.json")
        result = suggestions(
            mapping,
            ["docs/development/tools/probe_validation_capabilities.py"],
        )
        commands = [item["command"] for item in result["commands"]]
        self.assertIn("python3 docs/development/tools/check_ai_development.py", commands)

    def test_current_open_count_is_not_a_generic_stale_phrase(self) -> None:
        self.assertFalse(any(pattern.search("7 open") for pattern, _ in STALE_PATTERNS))
        self.assertTrue(any(pattern.search("104 | 7") for pattern, _ in STALE_PATTERNS))

    def test_project_docs_reject_stale_active_burn_down_count(self) -> None:
        ledgers = {
            "cfb": {
                "total": 137,
                "statuses": {"fixed": 135, "investigating": 2},
            },
            "cfr": {
                "total": 175,
                "statuses": {
                    "fixed": 159,
                    "fixed in v0.7": 10,
                    "investigating": 6,
                },
            },
        }
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            development = root / "docs/development"
            development.mkdir(parents=True)
            (development / "README.md").write_text(
                "CFB detail ledger: 137 total; 135 `fixed`, 2 `investigating`.\n"
                "CFR detail ledger: 175 total; 159 `fixed`, 10 `fixed in v0.7`, "
                "6 `investigating`.\n",
                encoding="utf-8",
            )
            (development / "current-cycle-burn-down-plan.md").write_text(
                "- CFB/CFR product/review issues: 8 open/investigating.\n"
                "- CFB detail ledger: 136 total; 134 `fixed`, 2 `investigating`.\n"
                "- CFR detail ledger: 175 total; 159 `fixed`, 10 `fixed in v0.7`, "
                "6 `investigating`.\n",
                encoding="utf-8",
            )
            (development / "codefire-improvement-cycle.md").write_text(
                "| CFB dogfood issues | 137 | 135 | 2 |\n"
                "| CFR code review issues | 175 | 169 | 6 |\n"
                "| Total | 312 | 304 | 8 |\n",
                encoding="utf-8",
            )
            findings = []
            check_current_claims(root, ledgers, findings)
        self.assertIn(
            "stale-active-cycle-ledger-claim",
            {finding.code for finding in findings},
        )


class RuntimeConfigTests(unittest.TestCase):
    def test_agent_layer_unknown_key_is_rejected_by_codex_strict_parser(self) -> None:
        codex = shutil.which("codex")
        if codex is None:
            self.skipTest("codex is unavailable")
        with tempfile.TemporaryDirectory() as temporary:
            config = Path(temporary) / "agent.toml"
            config.write_text(
                'model = "gpt-5.6-sol"\n'
                'model_reasoning_effort = "high"\n'
                'unknown_agent_layer_key = true\n',
                encoding="utf-8",
            )
            findings = []
            strict_config_probe(
                codex,
                config,
                ROOT,
                10.0,
                findings,
                copy_agent_layers=False,
            )
            self.assertIn(
                "config-layer-strict-unusable",
                {finding.code for finding in findings},
            )


class LiveRoutingEvidenceTests(unittest.TestCase):
    def test_repository_live_routing_smoke_is_valid(self) -> None:
        findings = []
        check_live_routing_smoke(ROOT, findings)
        self.assertEqual(findings, [])

    def test_live_routing_smoke_rejects_tier_response_drift(self) -> None:
        relative = Path("docs/development/ai/evals/live-routing-smoke-2026-07-11.v1.json")
        data = load_object(ROOT / relative)
        data["runs"][2]["observed_response"] = "not-ultra"
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / relative
            path.parent.mkdir(parents=True)
            path.write_text(json.dumps(data), encoding="utf-8")
            findings = []
            check_live_routing_smoke(root, findings)
            self.assertIn("live-routing-run-failed", {finding.code for finding in findings})


class HistoricalEvalTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.matrix = load_object(ROOT / "docs/development/ai/role-routing-matrix.json")
        cls.corpus = load_object(ROOT / "docs/development/ai/evals/historical-cases.v1.json")

    def test_repository_historical_corpus_is_valid(self) -> None:
        self.assertEqual(
            validate_historical_eval_corpus(
                self.corpus,
                self.matrix,
                ROOT,
                "historical-cases.v1.json",
            ),
            [],
        )

    def test_candidate_input_is_stable_and_redacts_reference_answer(self) -> None:
        first = build_candidate_input(self.corpus, "cfb-037-failed-command-evidence")
        second = build_candidate_input(self.corpus, "cfb-037-failed-command-evidence")
        self.assertEqual(first, second)
        serialized = json.dumps(first, sort_keys=True)
        reference = self.corpus["cases"][1]["oracle"]
        self.assertNotIn("oracle", serialized)
        for statement in reference["must_identify"]:
            self.assertNotIn(statement, serialized)
        self.assertEqual(len(first["reference_answer_sha256"]), 64)

    def test_historical_corpus_rejects_score_drift_and_missing_source(self) -> None:
        corpus = copy.deepcopy(self.corpus)
        corpus["cases"][0]["risk"]["expected_score"] = 0
        corpus["cases"][0]["oracle"]["source_files"] = ["docs/development/missing.md"]
        codes = {
            finding.code
            for finding in validate_historical_eval_corpus(
                corpus,
                self.matrix,
                ROOT,
                "mutated-corpus",
            )
        }
        self.assertIn("historical-score-mismatch", codes)
        self.assertIn("missing-eval-oracle-source", codes)


if __name__ == "__main__":
    unittest.main()
