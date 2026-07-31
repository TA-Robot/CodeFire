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
from dataclasses import replace
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
from capture_candidate_identity import (  # noqa: E402
    capture_candidate_identity,
    markdown_card,
)
from check_ai_acceleration_docs import (  # noqa: E402
    ACTIVE_RUST_PROFILE_MARKERS,
    STALE_PATTERNS,
    check_active_rust_packet_profiles,
    check_dev_swarm_v1_stage5_fields,
    check_live_routing_smoke,
)
from check_swarm_admission import (  # noqa: E402
    DEFAULT_RESERVATIONS,
    DEFAULT_THRESHOLDS,
    evaluate_admission,
)
from check_rust_execution_profile import (  # noqa: E402
    DEFAULT_PROFILE,
    audit_recorded_command,
    build_result as build_rust_profile_result,
    evaluate_process,
)
from check_review_terminalization import (  # noqa: E402
    agent_texts,
    evaluate_terminalization,
)
from measure_recurrence_v2_burden import SHAPES, measure  # noqa: E402
from check_review_recurrence import (  # noqa: E402
    evaluate_v2,
    external_result_path,
    legacy_v1_block,
    load_authority_schema,
    main as run_review_recurrence_gate,
    validate_recurrence_control,
    validator_owned_path,
)
from build_source_bound_rust import (  # noqa: E402
    BINARY_PATH,
    build_source_manifest,
    validate_provenance_payload,
    validate_source_inputs_against_head,
)
from prepare_historical_eval import build_candidate_input  # noqa: E402
from probe_validation_capabilities import rust_binary_state  # noqa: E402
from resolve_context_pack import build_manifest, expand_reference  # noqa: E402
from run_ai_role import (  # noqa: E402
    codex_cli_adapter_key,
    preflight_task_execution,
    resolve_packet_manifest,
    select_codex_cli_adapter,
)
import recurrence_v2_authority as v2_authority  # noqa: E402
import recurrence_v2_evidence as v2_evidence  # noqa: E402
import recurrence_v2_history as v2_history  # noqa: E402
from recurrence_v2_evidence import (  # noqa: E402
    TimedFacts,
    load_manifest,
    predecessor_path_presence,
    resolve_artifacts,
    verify_canonical_port_packet,
    verify_retained_live_arm_anchor,
)
from recurrence_v2_git import PATCH_FLAGS, GitError, GitRepo  # noqa: E402
from recurrence_v2_fixture import (  # noqa: E402
    VARIANTS as RECURRENCE_V2_VARIANTS,
    FixtureRepo,
    build_bootstrap,
    build_second,
    clean_gate,
    retain_generation,
    timed_result,
)
from recurrence_v2_model import (  # noqa: E402
    LIVE_ARM_ANCHOR_PATH,
    LIVE_ARM_ANCHOR_SCHEMA,
    AUTHORITY_SCHEMA_IDS,
    NEXT_ACTIONS,
    PORT_PACKETS,
    PORT_REQUIRED_REPAIR_PATHS,
    ArtifactRef,
    Authority,
    parse_manifest,
    parse_record,
    strict_decode_json,
)
from run_timed_gate import (  # noqa: E402
    main as run_timed_gate_main,
    run_gate,
    run_gate_v2,
    timing_evidence,
)
from run_dogfood_smoke import smoke_commands, validate_result  # noqa: E402
from suggest_validation import parse_porcelain_paths, suggestions  # noqa: E402
from validate_ai_runtime import strict_config_probe  # noqa: E402


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


class CodexRuntimeAdapterTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.matrix = load_object(ROOT / "docs/development/ai/role-routing-matrix.json")

    def test_version_resolves_to_registered_major_minor_adapter(self) -> None:
        self.assertEqual(
            codex_cli_adapter_key("codex-cli 0.146.0\n"),
            "codex_cli_0_146",
        )
        key, adapter = select_codex_cli_adapter(
            self.matrix,
            "codex-cli 0.146.7",
        )
        self.assertEqual(key, "codex_cli_0_146")
        self.assertEqual(adapter["ultra"]["reasoning_effort"], "ultra")

    def test_unregistered_cli_version_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "codex_cli_0_147"):
            select_codex_cli_adapter(self.matrix, "codex-cli 0.147.0")

    def test_unparseable_cli_version_is_rejected(self) -> None:
        with self.assertRaisesRegex(ValueError, "cannot parse"):
            codex_cli_adapter_key("not-codex")


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


class CandidateIdentityTests(unittest.TestCase):
    def make_repository(self, sandbox: Path) -> tuple[Path, str, str, Path]:
        root = sandbox / "repo"
        root.mkdir()
        packet_dir = root / "docs/development/ai/swarm-batches"
        packet_dir.mkdir(parents=True)
        ticket = packet_dir / "candidate-task.md"
        ticket.write_text("# Candidate task\n", encoding="utf-8")
        target = root / "product.txt"
        target.write_text("base\n", encoding="utf-8")
        subprocess.run(["git", "init", "-q"], cwd=root, check=True)
        subprocess.run(
            ["git", "config", "user.email", "ai-test@example.invalid"],
            cwd=root,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "AI Test"], cwd=root, check=True
        )
        subprocess.run(["git", "add", "."], cwd=root, check=True)
        subprocess.run(["git", "commit", "-qm", "base"], cwd=root, check=True)
        base = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        control = sandbox / "control"
        control.mkdir()
        manifest_path = control / "candidate-task.json"
        manifest_path.write_text(
            json.dumps(
                {
                    "schema": "dev-swarm.v1",
                    "batch_id": "candidate-batch",
                    "base_revision": base,
                    "tasks": [
                        {
                            "id": "candidate-task",
                            "role": "implementer",
                            "tier": "high",
                            "provider": "claude",
                            "ticket": ticket.relative_to(root).as_posix(),
                            "write_set": ["product.txt"],
                        }
                    ],
                }
            ),
            encoding="utf-8",
        )
        target.write_text("candidate\n", encoding="utf-8")
        subprocess.run(["git", "add", target], cwd=root, check=True)
        subprocess.run(["git", "commit", "-qm", "candidate"], cwd=root, check=True)
        head = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        return root, base, head, manifest_path

    def test_clean_linear_candidate_is_ready_and_exact(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, base, head, manifest_path = self.make_repository(
                Path(temporary).resolve()
            )
            identity = capture_candidate_identity(
                root,
                base,
                head,
                manifest_ref=str(manifest_path),
                task_id="candidate-task",
            )
            self.assertEqual(identity["decision"], "READY_FOR_REVIEW")
            self.assertEqual(identity["commit_count"], 1)
            self.assertEqual(
                [entry["path"] for entry in identity["changed_paths"]],
                ["product.txt"],
            )
            head_object = identity["changed_paths"][0]["head_object"]
            self.assertEqual(head_object["bytes"], len(b"candidate\n"))
            self.assertEqual(
                head_object["sha256"], hashlib.sha256(b"candidate\n").hexdigest()
            )
            self.assertEqual(identity["manifest"]["outside_write_set"], [])
            self.assertIn(identity["patch_sha256"], markdown_card(identity))

    def test_dirty_candidate_blocks_without_losing_identity(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, base, head, _ = self.make_repository(Path(temporary).resolve())
            (root / "untracked.txt").write_text("dirty\n", encoding="utf-8")
            identity = capture_candidate_identity(root, base, head)
            self.assertEqual(identity["decision"], "BLOCK")
            self.assertFalse(identity["clean"])
            self.assertIn(
                "candidate worktree has tracked or untracked changes",
                identity["blocking_reasons"],
            )

    def test_changed_path_outside_manifest_write_set_blocks(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, base, _, manifest_path = self.make_repository(
                Path(temporary).resolve()
            )
            outside = root / "outside.txt"
            outside.write_text("outside\n", encoding="utf-8")
            subprocess.run(["git", "add", outside], cwd=root, check=True)
            subprocess.run(["git", "commit", "-qm", "outside"], cwd=root, check=True)
            identity = capture_candidate_identity(
                root,
                base,
                "HEAD",
                manifest_ref=str(manifest_path),
                task_id="candidate-task",
            )
            self.assertEqual(identity["decision"], "BLOCK")
            self.assertIn(
                "candidate changed paths outside the declared write_set",
                identity["blocking_reasons"],
            )
            self.assertEqual(identity["manifest"]["outside_write_set"], ["outside.txt"])

    def test_report_mode_requires_role_and_every_declared_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root, base, head, manifest_path = self.make_repository(
                Path(temporary).resolve()
            )
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            manifest["tasks"][0]["role"] = "reviewer"
            manifest["tasks"][0]["write_set"].append("missing-report.md")
            manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
            identity = capture_candidate_identity(
                root,
                base,
                head,
                manifest_ref=str(manifest_path),
                task_id="candidate-task",
                expected_role="reviewer",
                require_exact_write_set=True,
            )
            self.assertEqual(identity["decision"], "BLOCK")
            self.assertIn(
                "declared write_set contains paths absent from the candidate",
                identity["blocking_reasons"],
            )
            self.assertEqual(
                identity["manifest"]["unmatched_write_set"], ["missing-report.md"]
            )


class TimedGateTests(unittest.TestCase):
    def test_gate_uses_monotonic_duration_and_binds_complete_log(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            log = root / "gate.log"
            result = run_gate(
                [sys.executable, "-c", "print('gate-ok')"],
                root,
                log,
                5,
            )
            self.assertEqual(result["status"], "passed")
            self.assertEqual(result["exit_code"], 0)
            self.assertEqual(result["duration_source"], "time.monotonic_ns")
            self.assertGreaterEqual(result["duration_seconds"], 0)
            self.assertEqual(log.read_text(encoding="utf-8"), "gate-ok\n")
            self.assertEqual(result["log"]["bytes"], len(b"gate-ok\n"))
            self.assertEqual(
                result["log"]["sha256"], hashlib.sha256(b"gate-ok\n").hexdigest()
            )
            self.assertTrue(result["log"]["complete"])

    def test_wall_clock_jump_is_explicit_and_not_the_duration(self) -> None:
        result = timing_evidence(
            1_000_000_000,
            11_000_000_000,
            20_000_000_000,
            22_000_000_000,
            drift_threshold_seconds=2.0,
        )
        self.assertEqual(result["duration_seconds"], 2.0)
        self.assertEqual(result["wall_elapsed_seconds"], 10.0)
        self.assertEqual(result["wall_minus_monotonic_seconds"], 8.0)
        self.assertTrue(result["wall_clock_discontinuity"])

    def test_v2_cli_requires_an_explicit_result_path(self) -> None:
        with mock.patch("sys.stderr"):
            status = run_timed_gate_main(
                [
                    "--emit",
                    "v2",
                    "--log",
                    "/tmp/recurrence-v2.log",
                    "--timeout",
                    "5",
                    "--",
                    sys.executable,
                    "-c",
                    "pass",
                ]
            )
        self.assertEqual(status, 2)

    def test_v2_cli_rejects_invalid_timing_before_subprocess(self) -> None:
        invalid_timeout = ("NaN", "Infinity", "-Infinity", "1e999", "-1e999", "0", "-1")
        invalid_threshold = ("NaN", "Infinity", "-Infinity", "1e999", "-1e999", "-1")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for flag, values in (
                ("--timeout", invalid_timeout),
                ("--drift-threshold", invalid_threshold),
            ):
                for value in values:
                    with self.subTest(flag=flag, value=value), mock.patch(
                        "subprocess.Popen"
                    ) as popen, mock.patch("sys.stderr"):
                        argv = [
                            "--emit",
                            "v2",
                            "--cwd",
                            str(root),
                            "--log",
                            str(root / "gate.log"),
                            "--result",
                            str(root / "gate.result.json"),
                            "--timeout",
                            "5",
                            "--drift-threshold",
                            "2",
                            "--",
                            sys.executable,
                            "-c",
                            "pass",
                        ]
                        argv[argv.index(flag) + 1] = value
                        self.assertEqual(run_timed_gate_main(argv), 2)
                        popen.assert_not_called()


class SwarmAdmissionTests(unittest.TestCase):
    def snapshot(self, **overrides: object) -> dict[str, object]:
        snapshot: dict[str, object] = {
            "effective_available_bytes": 12 * 1024**3,
            "swap_used_bytes": 0,
            "disk_available_bytes": 800 * 1024**3,
            "pids_headroom": 20_000,
            "psi": {
                "cpu": {"some": {"avg10": 0.0}},
                "memory": {"full": {"avg10": 0.0}},
                "io": {"full": {"avg10": 0.0}},
            },
        }
        snapshot.update(overrides)
        return snapshot

    def assess(
        self, snapshot: dict[str, object], **overrides: object
    ) -> dict[str, object]:
        arguments: dict[str, object] = {
            "resource_class": "light",
            "reservation_bytes": DEFAULT_RESERVATIONS["light"],
            "provider": "fugu",
            "provider_active": 1,
            "provider_window": 4,
        }
        arguments.update(overrides)
        return evaluate_admission(snapshot, **arguments)  # type: ignore[arg-type]

    def test_allows_another_lane_without_a_total_session_count(self) -> None:
        result = self.assess(self.snapshot())
        self.assertEqual(result["decision"], "ALLOW")
        self.assertEqual(
            result["basis"],
            "resource-and-provider-window; no fixed total session cap",
        )
        self.assertNotIn(
            "total-sessions", [item["id"] for item in result["checks"]]
        )

    def test_blocks_projected_memory_floor(self) -> None:
        result = self.assess(
            self.snapshot(
                effective_available_bytes=(
                    DEFAULT_THRESHOLDS["available_after_bytes"]
                    + DEFAULT_RESERVATIONS["light"]
                    - 1
                )
            )
        )
        self.assertEqual(result["decision"], "BLOCK")
        self.assertIn("projected-memory-floor", result["blockers"])

    def test_pending_launch_reservation_is_not_double_admitted(self) -> None:
        result = self.assess(
            self.snapshot(),
            outstanding_reservation_bytes=4 * 1024**3 + 1,
        )
        self.assertEqual(result["decision"], "BLOCK")
        self.assertIn("projected-memory-floor", result["blockers"])

    def test_provider_backoff_window_and_conflict_are_independent_blocks(
        self,
    ) -> None:
        result = self.assess(
            self.snapshot(),
            provider_active=4,
            provider_window=4,
            provider_backoff=True,
            conflict_reasons=["shared Cargo target has an active exclusive build"],
        )
        self.assertEqual(result["decision"], "BLOCK")
        self.assertEqual(
            set(result["blockers"]),
            {"provider-window", "provider-backoff", "conflict-domain"},
        )

    def test_pressure_blocks_even_with_free_bytes(self) -> None:
        snapshot = self.snapshot()
        snapshot["psi"] = {
            "cpu": {"some": {"avg10": DEFAULT_THRESHOLDS["cpu_some_avg10"] + 0.1}},
            "memory": {
                "full": {"avg10": DEFAULT_THRESHOLDS["memory_full_avg10"] + 0.1}
            },
            "io": {"full": {"avg10": DEFAULT_THRESHOLDS["io_full_avg10"] + 0.1}},
        }
        result = self.assess(snapshot)
        self.assertEqual(result["decision"], "BLOCK")
        self.assertEqual(
            set(result["blockers"]),
            {"cpu-pressure", "memory-pressure", "io-pressure"},
        )


class RustExecutionProfileTests(unittest.TestCase):
    def observation(self) -> dict[str, object]:
        target = DEFAULT_PROFILE["target_dir"]
        rustup = DEFAULT_PROFILE["rustup_home"]
        return {
            "pid": 101,
            "tool": "rustc",
            "cwd": "/workspace/worktrees/candidate",
            "argv": [
                f"{rustup}/toolchains/1.97.0/bin/rustc",
                "--out-dir",
                f"{target}/debug/deps",
            ],
            "environment": {
                "RUSTUP_HOME": DEFAULT_PROFILE["rustup_home"],
                "CARGO_HOME": DEFAULT_PROFILE["cargo_home"],
                "CARGO_TARGET_DIR": target,
                "CARGO_BUILD_JOBS": "2",
                "RUST_TEST_THREADS": "2",
                "CARGO_INCREMENTAL": "0",
            },
        }

    def test_shared_low_memory_process_passes(self) -> None:
        self.assertEqual(evaluate_process(self.observation()), [])

    def test_private_target_and_toolchain_are_blocked(self) -> None:
        observation = self.observation()
        observation["argv"] = [
            "/tmp/codefire-rustup/rustup/toolchains/stable/bin/rustc",
            "--out-dir",
            "/tmp/private-target/release/deps",
        ]
        environment = dict(observation["environment"])  # type: ignore[arg-type]
        environment["RUSTUP_HOME"] = "/tmp/codefire-rustup/rustup"
        environment["CARGO_TARGET_DIR"] = "/tmp/private-target"
        environment["CARGO_BUILD_JOBS"] = "8"
        observation["environment"] = environment
        codes = {
            finding["code"] for finding in evaluate_process(observation)
        }
        self.assertEqual(
            codes,
            {
                "wrong-rustup-home",
                "wrong-target-dir",
                "unsafe-build-jobs",
                "wrong-toolchain-executable",
                "private-out-dir",
            },
        )

    def test_recorded_explicit_override_is_durable_postflight_evidence(
        self,
    ) -> None:
        command = (
            "export RUSTUP_HOME=/tmp/rustup; "
            "CARGO_TARGET_DIR=target/release-review "
            "CARGO_BUILD_JOBS=7 cargo test --release"
        )
        codes = {
            finding["code"]
            for finding in audit_recorded_command(
                command, cwd=Path("/workspace/worktrees/candidate")
            )
        }
        self.assertEqual(
            codes,
            {
                "recorded-wrong-rustup-home",
                "recorded-wrong-target-dir",
                "recorded-unsafe-build-jobs",
            },
        )

    def test_recorded_violations_are_grouped_without_losing_count(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            events = Path(temporary) / "events.jsonl"
            events.write_text(
                "\n".join(
                    json.dumps(
                        {
                            "type": "item.completed",
                            "item": {
                                "type": "command_execution",
                                "command": (
                                    f"CARGO_TARGET_DIR=/tmp/private-{index} "
                                    "cargo test"
                                ),
                            },
                        }
                    )
                    for index in (1, 2)
                )
                + "\n",
                encoding="utf-8",
            )
            result = build_rust_profile_result(
                [],
                event_paths=[events],
                event_cwd=Path("/workspace/worktrees/candidate"),
            )
        self.assertEqual(result["decision"], "BLOCK")
        self.assertEqual(result["recorded_commands_scanned"], 2)
        self.assertEqual(result["recorded_violation_commands"], 2)
        self.assertEqual(
            sum(item["count"] for item in result["recorded_violations"]),
            2,
        )


class ReviewTerminalizationTests(unittest.TestCase):
    def report(self, state: str = "missing") -> dict[str, object]:
        return {
            "state": state,
            "path": "/repo/result/review.md",
            "bytes": 0,
            "sha256": None,
        }

    def evaluate(
        self,
        texts: list[str],
        *,
        report_state: str = "missing",
        worker_alive: bool | None = True,
    ) -> dict[str, object]:
        return evaluate_terminalization(
            texts,
            report=self.report(report_state),
            worker_alive=worker_alive,
            max_ready_signals=3,
            max_events_after_ready=4,
            max_events_after_verdict=8,
        )

    def test_long_review_with_unique_progress_remains_active(self) -> None:
        result = self.evaluate(
            [
                "I reproduced one counterexample.",
                "I am running the focused gate.",
                "The gate passed; now tracing the final owner.",
            ]
        )
        self.assertEqual(result["decision"], "WAIT")
        self.assertEqual(result["state"], "active")

    def test_repeated_ready_without_report_requires_intervention(self) -> None:
        result = self.evaluate(
            [
                "Decision is REQUEST_CHANGES.",
                "The decisive report is ready.",
                "Checking one more route.",
                "The report content is fixed.",
                "Checking one more timing.",
                "I am writing the complete report now.",
                "Checking one more name.",
                "Checking one more boundary.",
            ]
        )
        self.assertEqual(result["decision"], "INTERVENE")
        self.assertEqual(result["state"], "terminalization-stalled")

    def test_committed_report_is_complete(self) -> None:
        result = self.evaluate(
            ["The report is ready."],
            report_state="committed",
            worker_alive=False,
        )
        self.assertEqual(result["decision"], "COMPLETE")
        self.assertEqual(result["state"], "complete")

    def test_committed_report_with_live_worker_requires_collection(self) -> None:
        result = self.evaluate(
            ["The report is ready.", "Rerunning the broad gate again."],
            report_state="committed",
            worker_alive=True,
        )
        self.assertEqual(result["decision"], "INTERVENE")
        self.assertEqual(result["state"], "report-committed-worker-live")

    def test_command_output_does_not_create_agent_ready_signal(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            events = Path(temporary) / "events.jsonl"
            events.write_text(
                json.dumps(
                    {
                        "type": "item.completed",
                        "item": {
                            "type": "command_execution",
                            "aggregated_output": "The report is ready.",
                        },
                    }
                )
                + "\n"
                + json.dumps(
                    {
                        "type": "item.completed",
                        "item": {
                            "type": "agent_message",
                            "text": "Still gathering evidence.",
                        },
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            self.assertEqual(agent_texts(events), ["Still gathering evidence."])


class ReviewRecurrenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.schema = load_object(
            ROOT
            / "docs/development/ai/schemas/review-recurrence-control.v1.schema.json"
        )
        cls.v2_schema = load_object(
            ROOT
            / "docs/development/ai/schemas/review-recurrence-control.v2.schema.json"
        )
        cls.v2_manifest_schema = load_object(
            ROOT
            / "docs/development/ai/schemas/"
            "review-recurrence-repair-manifest.v1.schema.json"
        )
        cls.v2_timed_schema = load_object(
            ROOT / "docs/development/ai/schemas/timed-gate-result.v2.schema.json"
        )
        cls.v2_result_schema = load_object(
            ROOT
            / "docs/development/ai/schemas/review-recurrence-gate-result.v2.schema.json"
        )
        cls.v2_anchor_schema = load_object(
            ROOT
            / "docs/development/ai/schemas/review-recurrence-live-arm-anchor.v1.schema.json"
        )

    def evaluate_built_v2(
        self,
        repo: FixtureRepo,
        built: object,
        *,
        gate: dict[str, object] | None = None,
        verify_git: bool = True,
        live_arm: str | None = None,
        validator_root: Path | None = None,
    ) -> tuple[str, dict[str, object]]:
        payload = json.dumps(built.record, indent=2, sort_keys=True) + "\n"
        identity = gate or clean_gate(repo, built.commits["predecessor"])
        result = evaluate_v2(
            built.record,
            hashlib.sha256(payload.encode("utf-8")).hexdigest(),
            root=repo.root,
            validator_root=validator_root,
            schema=self.v2_schema,
            manifest_schema=self.v2_manifest_schema,
            timed_schema=self.v2_timed_schema,
            result_schema=self.v2_result_schema,
            anchor_schema=self.v2_anchor_schema,
            deciding_gate=identity,
            verify_git=verify_git,
            live_arm=live_arm,
            legacy_roots=built.roots,
        )
        return payload, result

    def amend_timed_result(
        self,
        repo: FixtureRepo,
        built: object,
        mutate: object,
    ) -> None:
        path = built.paths["timed_result"]
        payload = json.loads((repo.root / path).read_text(encoding="utf-8"))
        mutate(payload)
        repo.write(path, json.dumps(payload, indent=2, sort_keys=True) + "\n")
        old_evidence = built.commits["evidence"]
        repo.git("add", "--", path)
        repo.git("commit", "--amend", "-qm", "mutated timed evidence")
        new_evidence = repo.git("rev-parse", "HEAD")
        built.commits["evidence"] = new_evidence
        built.record["subject"]["evidence_commit"] = new_evidence
        for artifact in built.record["artifacts"]:
            if artifact["commit"] != old_evidence:
                continue
            artifact["commit"] = new_evidence
            artifact["sha256"] = repo.blob_digest(new_evidence, artifact["path"])

    def amend_timed_result_raw_number(
        self,
        repo: FixtureRepo,
        built: object,
        token: str,
    ) -> None:
        path = built.paths["timed_result"]
        payload = (repo.root / path).read_text(encoding="utf-8")
        needle = '"duration_seconds": 1.0'
        self.assertIn(needle, payload)
        repo.write(path, payload.replace(needle, f'"duration_seconds": {token}', 1))
        old_evidence = built.commits["evidence"]
        repo.git("add", "--", path)
        repo.git("commit", "--amend", "-qm", "mutated raw timed evidence")
        new_evidence = repo.git("rev-parse", "HEAD")
        built.commits["evidence"] = new_evidence
        built.record["subject"]["evidence_commit"] = new_evidence
        for artifact in built.record["artifacts"]:
            if artifact["commit"] != old_evidence:
                continue
            artifact["commit"] = new_evidence
            artifact["sha256"] = repo.blob_digest(new_evidence, artifact["path"])

    def live_anchor(self, built: object) -> dict[str, object]:
        artifacts = {
            artifact["ref_id"]: artifact for artifact in built.record["artifacts"]
        }

        def pin(ref_id: str) -> dict[str, object]:
            artifact = artifacts[ref_id]
            return {
                "commit": artifact["commit"],
                "path": artifact["path"],
                "sha256": artifact["sha256"],
                "mode": "100644",
            }

        return {
            "schema": LIVE_ARM_ANCHOR_SCHEMA,
            "arms": [
                {
                    "arm": "cargo",
                    "generation": built.record["generation"],
                    "repair_packet": pin("packet"),
                    "repair_manifest": pin("manifest"),
                    "predecessor_authority_commit": built.record["subject"][
                        "predecessor_authority"
                    ]["commit"],
                }
            ],
        }

    def mocked_authority_walk(
        self,
        template_raw: dict[str, object],
        link_count: int,
        *,
        cycle: bool = False,
    ) -> tuple[object, list[object]]:
        template, parse_findings = parse_record(template_raw, "template")
        self.assertIsNotNone(template)
        self.assertEqual(parse_findings, [])

        retention_for = {
            generation: f"{generation:040x}"
            for generation in range(1, link_count + 1)
        }

        def digest(kind: str, generation: int) -> str:
            return hashlib.sha256(f"{kind}-{generation}".encode()).hexdigest()

        def model(generation: int) -> object:
            if generation == 1:
                authority = Authority(
                    kind="legacy_bootstrap_report",
                    bootstrap_arm="cargo",
                    commit="f" * 40,
                    artifact_refs=("legacy-root",),
                )
                artifacts: tuple[ArtifactRef, ...] = ()
            else:
                retained_generation = generation - 1
                retention = retention_for[retained_generation]
                authority = Authority(
                    kind="prior_v2_record",
                    bootstrap_arm=None,
                    commit=retention,
                    artifact_refs=("prior-record", "prior-result"),
                )
                artifacts = (
                    ArtifactRef(
                        "prior-record",
                        "prior_v2_record",
                        retention,
                        f"records/{retained_generation}.json",
                        digest("record", retained_generation),
                        "100644",
                    ),
                    ArtifactRef(
                        "prior-result",
                        "prior_v2_result",
                        retention,
                        f"results/{retained_generation}.json",
                        digest("result", retained_generation),
                        "100644",
                    ),
                )
            subject = replace(
                template.subject,
                predecessor_candidate_revision=f"candidate-{generation - 1}",
                candidate_revision=f"candidate-{generation}",
                review_control_commit=f"control-{generation}",
                report_commit=f"report-{generation}",
                evidence_commit=f"evidence-{generation}",
                authority=authority,
            )
            return replace(
                template,
                generation=generation,
                subject=subject,
                artifacts=artifacts,
                prior_obligations=(),
                new_findings=(),
            )

        models = {
            generation: model(generation)
            for generation in range(1, link_count + 2)
        }
        if cycle:
            first = models[1]
            models[1] = replace(
                first,
                subject=replace(
                    first.subject,
                    authority=Authority(
                        kind="prior_v2_record",
                        bootstrap_arm=None,
                        commit=retention_for[1],
                        artifact_refs=("prior-record", "prior-result"),
                    ),
                ),
            )
        generation_by_retention = {
            retention: generation
            for generation, retention in retention_for.items()
        }
        gate = {
            "state": "clean_committed",
            "source_revision": "e" * 40,
            "source_tree": "d" * 40,
            "artifacts": [],
        }

        def load_blob(
            _repo: object,
            commit: str,
            path: str,
            _location: str,
            _findings: list[object],
        ) -> tuple[dict[str, object], str]:
            generation = generation_by_retention[commit]
            if path.startswith("records/"):
                return {"generation": generation}, digest("record", generation)
            return (
                {
                    "schema": v2_authority.RESULT_SCHEMA,
                    "record_sha256": digest("record", generation),
                    "successor_obligation_ids": [],
                    "deciding_gate": gate,
                    "live_arm_pin": {
                        "state": "unavailable",
                        "anchor": None,
                        "arm": None,
                    },
                },
                digest("result", generation),
            )

        def parse_link(
            raw: dict[str, object], _location: str
        ) -> tuple[object, list[object]]:
            return models[int(raw["generation"])], []

        with (
            mock.patch.object(v2_authority, "_load_json_blob", side_effect=load_blob),
            mock.patch.object(v2_authority, "parse_record", side_effect=parse_link),
            mock.patch.object(v2_authority, "generation_view", return_value=object()),
            mock.patch.object(v2_authority, "verify_generation", return_value=(None, [])),
            mock.patch.object(v2_authority, "verify_retention", return_value=[]),
            mock.patch.object(v2_authority, "_check_deciding_gate", return_value=gate),
            mock.patch.object(v2_authority, "verify_legacy_root", return_value=(True, [])),
        ):
            return v2_authority.resolve_predecessor_authority_chain(
                object(),
                models[link_count + 1],
                schema=None,
                deciding_gate=gate,
                legacy_roots={},
            )

    def record(self) -> dict[str, object]:
        digest = "a" * 64
        return {
            "schema": "codefire.review_recurrence_control.v1",
            "control_id": "review-control-001",
            "record_status": "final",
            "generation": 2,
            "subject": {
                "review_id": "review-002",
                "repair_packet_id": "repair-001",
                "repair_packet_path": (
                    "docs/development/ai/task-packets/repair.md"
                ),
                "repair_packet_revision": "repair-packet-commit",
                "repair_packet_sha256": digest,
                "repair_packet_timing": "ancestor_of_candidate",
                "repair_dispatch_event_ref": None,
                "repair_dispatch_event_sha256": None,
                "review_packet_id": "review-packet-002",
                "review_packet_path": (
                    "docs/development/ai/task-packets/review.md"
                ),
                "review_packet_sha256": digest,
                "base_revision": "base",
                "candidate_revision": "candidate",
                "candidate_tree": "tree",
                "patch_sha256": digest,
                "changed_paths": ["product.txt"],
                "review_control_revision": "control",
                "review_control_tree": "control-tree",
                "control_patch_sha256": digest,
                "control_changed_paths": [
                    "docs/development/ai/task-packets/repair.md"
                ],
                "report_commit": "report",
                "report_path": (
                    "docs/development/ai/swarm-results/review-002/review.md"
                ),
                "report_sha256": digest,
            },
            "coverage": {
                "finding_target_policy": "none",
                "stopping_rule": {
                    "method": "obligation_and_surface_saturation",
                    "prior_obligations_dispositioned": True,
                    "declared_surfaces_checked": True,
                    "repair_delta_checked": True,
                    "production_routes_checked": True,
                    "negative_controls_checked": True,
                    "unavailable_evidence_explicit": True,
                    "rationale": "All declared obligations and introduced surfaces ended.",
                },
                "declared_surfaces": ["product owner and terminal fence"],
                "out_of_scope": [],
            },
            "prior_findings": [
                {
                    "id": "F-001",
                    "observation_id": "OBS-002-001",
                    "effect_episode_id": "EFFECT-001",
                    "origin_generation": 1,
                    "continuity": "continuous_open",
                    "obligation": "The live route must terminate at the final fence.",
                    "closure_contract": {
                        "acceptance_criterion": "The final fence observes zero debt.",
                        "repair_boundary": "The product owner and its focused tests.",
                        "required_production_routes": ["normal and unwind routes"],
                        "anticipated_regressions": ["early release before final fence"],
                        "forbidden_shortcuts": ["test-only or claim-only closure"],
                    },
                    "blocking": True,
                    "disposition": "closed",
                    "causal_classes": ["bound_finding_not_closed"],
                    "before": {
                        "status": "reproduced",
                        "summary": "The predecessor leaks one unit.",
                        "evidence_refs": ["predecessor:test"],
                    },
                    "after": {
                        "status": "verified",
                        "summary": "The candidate reaches zero at the fence.",
                        "evidence_refs": ["candidate:test"],
                    },
                    "oracle": {
                        "property": "terminal debt is zero",
                        "independence_basis": "asserts externally visible state",
                        "assertion_boundary": "after the production final fence",
                        "positive_control_refs": ["candidate:test"],
                        "negative_control": {
                            "status": "killed",
                            "rationale": "Restoring the predecessor defect fails.",
                            "evidence_refs": ["mutation:test"],
                        },
                    },
                    "evidence_refs": ["source:owner", "log:focused"],
                    "owner": None,
                    "release_condition": None,
                    "route_authority": None,
                }
            ],
            "repair_delta": {
                "inserted_lines": 1,
                "deleted_lines": 0,
                "binary_paths": [],
                "changed_path_coverage": [
                    {
                        "path": "product.txt",
                        "required_by": ["F-001"],
                        "review_focus": [
                            "owner, effect, fence, and repair-created behavior"
                        ],
                        "disposition": "reviewed",
                        "evidence_refs": ["review:product-path"],
                    }
                ],
                "changed_surfaces": [
                    {
                        "path": "product.txt",
                        "symbol": "final_fence",
                        "kind": "private_symbol",
                        "change_type": "modified",
                        "required_by": "F-001",
                        "planned": True,
                        "plan_ref": "repair-001#authorized-change",
                        "disposition": "reviewed",
                        "evidence_refs": ["review:repair-delta"],
                    }
                ],
                "scope_expansion": "none",
                "scope_authority": None,
                "scope_rationale": "The change stays inside the authorized owner.",
            },
            "new_findings": [],
            "validations": [
                {
                    "id": "focused",
                    "command": "test focused",
                    "required": True,
                    "status": "passed",
                    "exit_code": 0,
                    "terminal": True,
                    "duration_ms": 10,
                    "log_ref": "log:focused",
                    "log_sha256": digest,
                    "result_summary": "passed",
                }
            ],
            "terminalization": {
                "state": "complete",
                "worker_alive": False,
                "child_processes_active": 0,
                "report_state": "committed",
                "event_log_ref": "events:review-002",
                "event_log_sha256": digest,
                "checked_at": "2026-07-31T12:00:00Z",
            },
            "outcomes": {
                "prior_findings_total": 1,
                "prior_findings_closed": 1,
                "prior_findings_open": 0,
                "prior_findings_routed": 0,
                "prior_findings_unavailable": 0,
                "candidate_created_findings": 0,
                "new_findings_total": 0,
                "new_findings_open": 0,
                "new_findings_routed": 0,
                "new_findings_unavailable": 0,
                "prior_observations_total": 1,
                "prior_effect_episodes_total": 1,
                "new_observations_total": 0,
                "new_effect_episodes_total": 0,
                "unplanned_surfaces": 0,
                "repair_service_ms": 20,
                "review_service_ms": 30,
                "verdict_to_report_ms": 5,
                "unavailable_metrics": [],
            },
            "review_decision": "accept",
        }

    def codes(self, record: dict[str, object]) -> set[str]:
        return {
            finding.code
            for finding in validate_recurrence_control(
                record,
                schema=self.schema,
            )
        }

    def test_valid_accept_record_is_count_free_and_complete(self) -> None:
        self.assertEqual(self.codes(self.record()), set())

    def test_closed_finding_requires_a_killed_negative_control(self) -> None:
        record = self.record()
        record["prior_findings"][0]["oracle"]["negative_control"]["status"] = (
            "survived"
        )
        self.assertIn("closure-without-negative-control", self.codes(record))

    def test_blocking_finding_cannot_be_routed(self) -> None:
        record = self.record()
        row = record["prior_findings"][0]
        row["disposition"] = "routed_open"
        row["owner"] = "successor"
        row["release_condition"] = "successor accepted"
        row["route_authority"] = "manager:decision"
        self.assertIn("blocking-finding-routed", self.codes(record))

    def test_passed_validation_must_be_terminal_and_content_addressed(self) -> None:
        record = self.record()
        validation = record["validations"][0]
        validation["terminal"] = False
        validation["log_ref"] = None
        validation["log_sha256"] = None
        codes = self.codes(record)
        self.assertIn("completed-validation-not-terminal", codes)
        self.assertIn("completed-validation-without-log-reference", codes)
        self.assertIn("completed-validation-without-log-digest", codes)

    def test_accept_rejects_new_blocking_finding(self) -> None:
        record = self.record()
        record["new_findings"] = [
            {
                "id": "F-NEW",
                "observation_id": "OBS-002-NEW",
                "effect_episode_id": "EFFECT-NEW",
                "continuity": "new_effect",
                "severity": "high",
                "blocking": True,
                "causal_classes": ["repair_introduced_regression"],
                "attribution": "created_by_candidate",
                "disposition": "open",
                "statement": "The repair introduced a new live-route failure.",
                "evidence_refs": ["candidate:new-failure"],
                "owner": "repair implementer",
                "release_condition": "next exact candidate closes the failure",
                "route_authority": None,
            }
        ]
        outcomes = record["outcomes"]
        outcomes["candidate_created_findings"] = 1
        outcomes["new_findings_total"] = 1
        outcomes["new_findings_open"] = 1
        outcomes["new_observations_total"] = 1
        outcomes["new_effect_episodes_total"] = 1
        self.assertIn("accept-with-unresolved-new-finding", self.codes(record))

    def test_unplanned_surface_needs_an_explicit_scope_decision(self) -> None:
        record = self.record()
        surface = record["repair_delta"]["changed_surfaces"][0]
        surface["planned"] = False
        surface["plan_ref"] = None
        record["outcomes"]["unplanned_surfaces"] = 1
        self.assertIn(
            "unplanned-surface-without-scope-decision",
            self.codes(record),
        )

    def test_accept_rejects_unreviewed_changed_path(self) -> None:
        record = self.record()
        record["repair_delta"]["changed_path_coverage"][0]["disposition"] = (
            "unreviewed"
        )
        self.assertIn(
            "accept-with-unreviewed-repair-path",
            self.codes(record),
        )

    def test_outcome_counts_cannot_collapse_effects_and_observations(self) -> None:
        record = self.record()
        record["outcomes"]["prior_effect_episodes_total"] = 2
        self.assertIn("outcome-count-mismatch", self.codes(record))

    def test_schema_rejects_a_finding_quota(self) -> None:
        record = self.record()
        record["coverage"]["target_finding_count"] = 100
        self.assertIn("schema-additional-property", self.codes(record))

    def test_schema_only_mode_never_allows_integration(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            record_path = Path(temporary) / "record.json"
            record_path.write_text(
                json.dumps(self.record()),
                encoding="utf-8",
            )
            with mock.patch("builtins.print"):
                exit_code = run_review_recurrence_gate(
                    [
                        "--record",
                        str(record_path),
                        "--schema",
                        str(
                            ROOT
                            / "docs/development/ai/schemas/"
                            "review-recurrence-control.v1.schema.json"
                        ),
                        "--no-git-verify",
                    ]
                )
            self.assertEqual(exit_code, 3)

    def test_git_binding_covers_candidate_control_report_and_task(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)

            def git(*args: str) -> str:
                completed = subprocess.run(
                    ["git", *args],
                    cwd=root,
                    check=True,
                    capture_output=True,
                    text=True,
                )
                return completed.stdout.strip()

            git("init", "-q")
            git("config", "user.name", "Review Test")
            git("config", "user.email", "review@example.invalid")
            (root / "README.md").write_text("base\n", encoding="utf-8")
            git("add", "README.md")
            git("commit", "-qm", "base")
            initial_base = git("rev-parse", "HEAD")
            repair_packet_path = Path(
                "docs/development/ai/task-packets/repair.md"
            )
            (root / repair_packet_path).parent.mkdir(parents=True)
            (root / repair_packet_path).write_text(
                "repair packet\n",
                encoding="utf-8",
            )
            git("add", repair_packet_path.as_posix())
            git("commit", "-qm", "repair contract")
            repair_packet_revision = git("rev-parse", "HEAD")
            base = repair_packet_revision
            (root / "product.txt").write_text("candidate\n", encoding="utf-8")
            git("add", "product.txt")
            git("commit", "-qm", "candidate")
            candidate = git("rev-parse", "HEAD")

            review_packet_path = Path(
                "docs/development/ai/task-packets/review.md"
            )
            (root / review_packet_path).write_text(
                "review packet\n",
                encoding="utf-8",
            )
            git("add", review_packet_path.as_posix())
            git("commit", "-qm", "review control")
            control = git("rev-parse", "HEAD")
            control_patch, control_paths, _, _ = canonical_git_patch(
                root,
                candidate,
                control,
            )

            report_path = Path(
                "docs/development/ai/swarm-results/review-002/review.md"
            )
            (root / report_path).parent.mkdir(parents=True)
            (root / report_path).write_text("review report\n", encoding="utf-8")
            git("add", report_path.as_posix())
            git("commit", "-qm", "review report")
            report = git("rev-parse", "HEAD")
            review_branch = git("branch", "--show-current")

            patch, paths, resolved_base, resolved_candidate = canonical_git_patch(
                root,
                base,
                candidate,
            )
            record = self.record()
            record["subject"].update(
                {
                    "repair_packet_path": repair_packet_path.as_posix(),
                    "repair_packet_revision": repair_packet_revision,
                    "repair_packet_sha256": hashlib.sha256(
                        (root / repair_packet_path).read_bytes()
                    ).hexdigest(),
                    "repair_packet_timing": "ancestor_of_candidate",
                    "repair_dispatch_event_ref": None,
                    "repair_dispatch_event_sha256": None,
                    "review_packet_path": review_packet_path.as_posix(),
                    "review_packet_sha256": hashlib.sha256(
                        (root / review_packet_path).read_bytes()
                    ).hexdigest(),
                    "base_revision": resolved_base,
                    "candidate_revision": resolved_candidate,
                    "candidate_tree": git("rev-parse", f"{candidate}^{{tree}}"),
                    "patch_sha256": hashlib.sha256(patch).hexdigest(),
                    "changed_paths": sorted(paths),
                    "review_control_revision": control,
                    "review_control_tree": git(
                        "rev-parse",
                        f"{control}^{{tree}}",
                    ),
                    "control_patch_sha256": hashlib.sha256(
                        control_patch
                    ).hexdigest(),
                    "control_changed_paths": sorted(control_paths),
                    "report_commit": report,
                    "report_path": report_path.as_posix(),
                    "report_sha256": hashlib.sha256(
                        (root / report_path).read_bytes()
                    ).hexdigest(),
                }
            )
            findings = validate_recurrence_control(
                record,
                schema=self.schema,
                root=root,
                verify_git=True,
            )
            self.assertEqual(findings, [])

            coverage = record["repair_delta"]["changed_path_coverage"]
            record["repair_delta"]["changed_path_coverage"] = []
            self.assertIn(
                "repair-path-coverage-mismatch",
                {
                    finding.code
                    for finding in validate_recurrence_control(
                        record,
                        schema=self.schema,
                        root=root,
                        verify_git=True,
                    )
                },
            )
            record["repair_delta"]["changed_path_coverage"] = coverage
            git("checkout", "-q", "-b", "external-contract", initial_base)
            (root / repair_packet_path).parent.mkdir(parents=True)
            (root / repair_packet_path).write_text(
                "repair packet\n",
                encoding="utf-8",
            )
            git("add", repair_packet_path.as_posix())
            git("commit", "-qm", "external repair contract")
            external_contract = git("rev-parse", "HEAD")
            git("checkout", "-q", review_branch)

            record["subject"]["repair_packet_revision"] = external_contract
            self.assertIn(
                "repair-packet-does-not-precede-candidate",
                {
                    finding.code
                    for finding in validate_recurrence_control(
                        record,
                        schema=self.schema,
                        root=root,
                        verify_git=True,
                    )
                },
            )
            record["subject"]["repair_packet_timing"] = (
                "dispatch_event_attested"
            )
            record["subject"]["repair_dispatch_event_ref"] = (
                "events:repair-dispatch"
            )
            record["subject"]["repair_dispatch_event_sha256"] = "b" * 64
            self.assertEqual(
                validate_recurrence_control(
                    record,
                    schema=self.schema,
                    root=root,
                    verify_git=True,
                ),
                [],
            )
            record["subject"]["repair_packet_revision"] = (
                repair_packet_revision
            )
            record["subject"]["repair_packet_timing"] = (
                "ancestor_of_candidate"
            )
            record["subject"]["repair_dispatch_event_ref"] = None
            record["subject"]["repair_dispatch_event_sha256"] = None
            record["repair_delta"]["inserted_lines"] = 2
            self.assertIn(
                "repair-delta-mismatch",
                {
                    finding.code
                    for finding in validate_recurrence_control(
                        record,
                        schema=self.schema,
                        root=root,
                        verify_git=True,
                    )
                },
            )
            record["repair_delta"]["inserted_lines"] = 1
            record["subject"]["report_sha256"] = "0" * 64
            self.assertIn(
                "report-digest-mismatch",
                {
                    finding.code
                    for finding in validate_recurrence_control(
                        record,
                        schema=self.schema,
                        root=root,
                        verify_git=True,
                    )
                },
            )

    def test_v2_bootstrap_blocks_and_rooted_second_generation_allows(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            self.assertEqual(first_result["integration_decision"], "BLOCK")
            self.assertTrue(first_result["record_valid"])
            self.assertEqual(first_result["finding_count"], 0)

            second = build_second(repo, first, first_payload, first_result)
            _, second_result = self.evaluate_built_v2(repo, second, gate=gate)
            self.assertEqual(second_result["integration_decision"], "ALLOW")
            self.assertEqual(second_result["finding_count"], 0)
            self.assertEqual(
                second_result["authority_chain"]["revalidated_generations"][0][
                    "current_gate_replayed_integration_decision"
                ],
                "BLOCK",
            )

    def test_v2_real_git_topology_counterexamples_are_structured_blocks(self) -> None:
        expected = {
            "late_packet": "packet-commit-not-direct-child",
            "extra_packet_path": "packet-commit-write-set-mismatch",
            "mutate_revert": "repair-commit-rewrites-packet",
            "control_rewrites_gate": "review-control-write-set-mismatch",
            "symlink_packet": "packet-artifact-not-a-regular-blob",
            "symlink_report": "report-not-a-regular-blob",
            "symlink_evidence": "evidence-not-a-regular-blob",
            "evidence_out_of_order": "evidence-not-direct-child",
            "invalid_utf8": "invalid-utf8-git-path",
            "nonlinear_repair": "repair-series-not-linear",
            "executable_report": "report-not-a-regular-blob",
            "manifest_thin_prior": "prior-obligation-set-mismatch",
            "manifest_thin_claim": "derivation-claim-set-mismatch",
            "manifest_thin_validation": "required-validation-set-mismatch",
        }
        self.assertEqual(
            set(expected) | {"standard", "byte_identical_locus"},
            set(RECURRENCE_V2_VARIANTS),
        )
        for variant, expected_code in expected.items():
            with self.subTest(variant=variant), tempfile.TemporaryDirectory() as temporary:
                repo = FixtureRepo(Path(temporary))
                built = build_bootstrap(repo, variant)
                _, result = self.evaluate_built_v2(repo, built)
                self.assertEqual(result["integration_decision"], "BLOCK")
                self.assertIn(
                    expected_code,
                    {finding["code"] for finding in result["findings"]},
                )

    def test_v2_manifest_sets_surfaces_and_count_free_shape_are_exact(self) -> None:
        mutations = (
            (
                "prior-obligation-thinned",
                lambda record: record["prior_obligations"].clear(),
                "prior-obligation-set-mismatch",
            ),
            (
                "derivation-thinned",
                lambda record: record["derivations"].clear(),
                "derivation-claim-set-mismatch",
            ),
            (
                "surface-thinned",
                lambda record: record["path_reviews"][0]["surfaces"].clear(),
                "schema-min-items",
            ),
            (
                "out-of-scope-escape",
                lambda record: record["path_reviews"][0].__setitem__(
                    "out_of_scope", True
                ),
                "schema-additional-property",
            ),
            (
                "finding-quota",
                lambda record: record.__setitem__("target_finding_count", 100),
                "schema-additional-property",
            ),
        )
        for name, mutate, expected_code in mutations:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                repo = FixtureRepo(Path(temporary))
                built = build_bootstrap(repo)
                mutate(built.record)
                _, result = self.evaluate_built_v2(repo, built)
                self.assertEqual(result["integration_decision"], "BLOCK")
                self.assertIn(
                    expected_code,
                    {finding["code"] for finding in result["findings"]},
                )

    def test_v2_rejects_narrowed_interval_and_author_transcribed_shortcuts(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            built = build_bootstrap(repo)
            original = copy.deepcopy(built.record)
            mutations = (
                (
                    "narrowed-predecessor-interval",
                    lambda record: record["subject"].update(
                        predecessor_candidate_revision=built.commits["packet"]
                    ),
                    {"legacy-root-predecessor-mismatch"},
                ),
                (
                    "branch-external-fake-dispatch",
                    lambda record: record["subject"].update(
                        repair_dispatch_event_ref="fake-dispatch",
                        repair_dispatch_event_sha256="0" * 64,
                    ),
                    {"schema-additional-property"},
                ),
                (
                    "no-surface-change-shortcut",
                    lambda record: record["path_reviews"][0].update(
                        no_surface_change=True
                    ),
                    {"schema-additional-property"},
                ),
                (
                    "author-transcribed-outcomes",
                    lambda record: record.update(
                        outcomes={"integration_ready": True, "finding_count": 0}
                    ),
                    {"schema-additional-property"},
                ),
                (
                    "finding-target-policy-not-none",
                    lambda record: record.update(finding_target_policy="at_least_one"),
                    {"schema-const"},
                ),
                (
                    "manifest-required-validation-omitted",
                    lambda record: record["validations"].clear(),
                    {
                        "required-validation-set-mismatch",
                        "manifest-required-validation-unbound",
                    },
                ),
                (
                    "author-relabelled-continuity",
                    lambda record: record["prior_obligations"][0].update(
                        continuity="continuous_open"
                    ),
                    {"schema-additional-property"},
                ),
            )
            for name, mutate, expected_codes in mutations:
                with self.subTest(name=name):
                    built.record = copy.deepcopy(original)
                    mutate(built.record)
                    _, result = self.evaluate_built_v2(repo, built)
                    codes = {finding["code"] for finding in result["findings"]}
                    self.assertEqual(result["integration_decision"], "BLOCK")
                    self.assertTrue(expected_codes <= codes, (name, codes))

    def test_v2_byte_identical_locus_still_requires_i1_derivation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            built = build_bootstrap(repo, "byte_identical_locus")
            before = subprocess.run(
                ["git", "show", f"{built.commits['predecessor']}:product.txt"],
                cwd=repo.root,
                check=True,
                capture_output=True,
                text=True,
            ).stdout
            after = subprocess.run(
                ["git", "show", f"{built.commits['candidate']}:product.txt"],
                cwd=repo.root,
                check=True,
                capture_output=True,
                text=True,
            ).stdout
            self.assertEqual(before.splitlines()[0], after.splitlines()[0])
            built.record["derivations"].clear()
            _, result = self.evaluate_built_v2(repo, built)
            self.assertEqual(result["integration_decision"], "BLOCK")
            self.assertIn(
                "derivation-claim-set-mismatch",
                {finding["code"] for finding in result["findings"]},
            )

    def test_v2_canonical_patch_flags_and_fixture_digest_are_pinned(self) -> None:
        self.assertEqual(
            PATCH_FLAGS,
            (
                "--no-ext-diff",
                "--no-textconv",
                "--no-renames",
                "--no-color",
                "-U3",
                "--inter-hunk-context=0",
                "--src-prefix=a/",
                "--dst-prefix=b/",
                "--binary",
                "--full-index",
            ),
        )
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            repo.write("product.txt", "before\n")
            predecessor = repo.commit("base", ["product.txt"])
            repo.write("product.txt", "after\n")
            candidate = repo.commit("candidate", ["product.txt"])
            expected = "50a19ad12629068d0a767ee77c8eafc315421c6dee52e702c40d3c42f57385a1"
            self.assertEqual(repo.patch_digest(predecessor, candidate), expected)
            repo.git("config", "diff.algorithm", "histogram")
            self.assertEqual(
                repo.patch_digest(predecessor, candidate),
                expected,
            )

    def test_v2_routing_states_never_become_integration_authority(self) -> None:
        mutations = (
            (
                "scope-without-manager-artifact",
                lambda row: row.update(
                    evidence_state="SCOPE_BOUNDARY",
                    next_action="MANAGER_SCOPE_DECISION",
                    routing_decision_ref=None,
                ),
                "manager-routing-without-decision-artifact",
            ),
            (
                "same-generation-closure",
                lambda row: row.update(
                    evidence_state="CLOSED_PROVEN", next_action="NONE"
                ),
                "same-generation-closure-of-new-finding",
            ),
            (
                "unavailable-with-evidence",
                lambda row: row["observations"].update(
                    negative_control="unavailable"
                ),
                "unavailable-state-with-evidence",
            ),
        )
        for name, mutate, expected_code in mutations:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                repo = FixtureRepo(Path(temporary))
                built = build_bootstrap(repo)
                mutate(built.record["new_findings"][0])
                _, result = self.evaluate_built_v2(repo, built)
                self.assertEqual(result["integration_decision"], "BLOCK")
                self.assertIn(
                    expected_code,
                    {finding["code"] for finding in result["findings"]},
                )

        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            built = build_bootstrap(repo)
            built.record["prior_obligations"][0].update(
                evidence_state="DEFECT_SOURCE_PROVEN",
                next_action="PRODUCT_REPAIR",
            )
            _, result = self.evaluate_built_v2(repo, built)
            self.assertEqual(result["integration_decision"], "BLOCK")
            row = next(row for row in result["rows"] if row["row_id"] == "OBL-1")
            self.assertEqual(row["evidence_state"], "DEFECT_SOURCE_PROVEN")
            self.assertTrue(row["blocking"])

    def test_v2_relabelled_origin_and_recurrence_after_closure_are_derived(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            first.record["prior_obligations"][0].update(
                evidence_state="CLOSED_PROVEN",
                next_action="NONE",
            )
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            self.assertEqual(first_result["finding_count"], 0)
            second = build_second(repo, first, first_payload, first_result)
            original = copy.deepcopy(second.record)

            second.record["prior_obligations"][0]["origin_generation"] = 1
            _, relabelled = self.evaluate_built_v2(repo, second, gate=gate)
            self.assertIn(
                "origin-generation-mismatch",
                {finding["code"] for finding in relabelled["findings"]},
            )

            second.record = copy.deepcopy(original)
            second.record["prior_obligations"][0].update(
                evidence_state="DEFECT_REPRODUCED",
                next_action="PRODUCT_REPAIR",
                closure_evidence_ref=None,
            )
            _, recurrence = self.evaluate_built_v2(repo, second, gate=gate)
            recurrence_row = next(
                row for row in recurrence["rows"] if row["row_id"] == "OBL-1"
            )
            self.assertEqual(
                recurrence_row["continuity"], "recurred_after_proven_closure"
            )
            self.assertIn(
                "recurrence-without-prior-closure-evidence",
                {finding["code"] for finding in recurrence["findings"]},
            )
            self.assertEqual(recurrence["integration_decision"], "BLOCK")

    def test_v2_injection_only_empty_mapping_needs_prior_bound_boundary(self) -> None:
        cases = (
            (False, True, "empty-closure-mapping-without-injection-predecessor"),
            (True, False, "injection-closure-without-non-reachability-boundary"),
            (True, True, None),
        )
        for predecessor_injection, bind_boundary, expected_code in cases:
            name = f"injection={predecessor_injection},boundary={bind_boundary}"
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                repo = FixtureRepo(Path(temporary))
                first = build_bootstrap(repo)
                if predecessor_injection:
                    first.record["prior_obligations"][0].update(
                        evidence_state="DEFECT_INJECTION_ONLY",
                        next_action="DOCUMENT_NON_REACHABILITY",
                    )
                gate = clean_gate(repo, first.commits["predecessor"])
                first_payload, first_result = self.evaluate_built_v2(
                    repo, first, gate=gate
                )
                self.assertEqual(first_result["finding_count"], 0)
                second = build_second(
                    repo,
                    first,
                    first_payload,
                    first_result,
                    injection_empty_mapping=True,
                    bind_non_reachability=bind_boundary,
                )
                _, result = self.evaluate_built_v2(repo, second, gate=gate)
                codes = {finding["code"] for finding in result["findings"]}
                if expected_code is None:
                    self.assertEqual(codes, set())
                    self.assertEqual(result["integration_decision"], "ALLOW")
                    row = next(
                        row for row in result["rows"] if row["row_id"] == "OBL-1"
                    )
                    self.assertTrue(row["closure_proven"])
                else:
                    self.assertIn(expected_code, codes)
                    self.assertEqual(result["integration_decision"], "BLOCK")

    def test_v2_closed_proven_rejects_missing_and_nonpassing_mapped_results(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            second = build_second(repo, first, first_payload, first_result)
            second.record["validations"].clear()
            _, missing = self.evaluate_built_v2(repo, second, gate=gate)
            self.assertIn(
                "closure-validation-missing",
                {finding["code"] for finding in missing["findings"]},
            )
            self.assertEqual(missing["integration_decision"], "BLOCK")

        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            second = build_second(repo, first, first_payload, first_result)

            def fail(payload: dict[str, object]) -> None:
                payload["status"] = "failed"
                payload["exit_code"] = 1

            self.amend_timed_result(repo, second, fail)
            second.record["validations"][0].update(status="failed", exit_code=1)
            _, failing = self.evaluate_built_v2(repo, second, gate=gate)
            self.assertIn(
                "closure-validation-not-passing",
                {finding["code"] for finding in failing["findings"]},
            )
            self.assertEqual(failing["integration_decision"], "BLOCK")

    def test_v2_history_replays_prior_timed_evidence_not_retained_claims(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            tampered = copy.deepcopy(first.record)
            tampered["validations"][0]["status"] = "failed"
            tampered_payload = json.dumps(tampered, indent=2, sort_keys=True) + "\n"
            forged_result = copy.deepcopy(first_result)
            forged_result["record_sha256"] = hashlib.sha256(
                tampered_payload.encode("utf-8")
            ).hexdigest()
            forged_result["record_valid"] = True
            forged_result["finding_count"] = 0
            retention, retained_result_payload = retain_generation(
                repo, first, tampered_payload, forged_result
            )
            second = build_second(
                repo,
                first,
                tampered_payload,
                forged_result,
                retention=retention,
                result_payload=retained_result_payload,
            )
            _, result = self.evaluate_built_v2(repo, second, gate=gate)
            self.assertEqual(result["integration_decision"], "BLOCK")
            self.assertIn(
                "record-timed-evidence-disagreement",
                {finding["code"] for finding in result["findings"]},
            )
            self.assertFalse(result["authority_chain"]["rooted"])

    def test_v2_history_rejects_self_contradictory_retained_result(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            retention, result_payload = retain_generation(
                repo,
                first,
                first_payload,
                first_result,
                result_mutation={
                    "record_valid": False,
                    "finding_count": 1,
                    "findings": [
                        {
                            "level": "error",
                            "code": "forged-retained-finding",
                            "path": "result",
                            "message": "this finding was not derived by replay",
                        }
                    ],
                },
            )
            second = build_second(
                repo,
                first,
                first_payload,
                first_result,
                retention=retention,
                result_payload=result_payload,
            )
            _, result = self.evaluate_built_v2(repo, second, gate=gate)
            self.assertEqual(result["integration_decision"], "BLOCK")
            replay_mismatches = [
                finding
                for finding in result["findings"]
                if finding["code"] == "retained-result-replay-mismatch"
            ]
            self.assertEqual(
                {finding["path"].rsplit(".", 1)[-1] for finding in replay_mismatches},
                {"record_valid", "finding_count", "findings"},
            )
            self.assertFalse(result["authority_chain"]["rooted"])

    def test_v2_prior_export_tamper_blocks_even_with_local_ref(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            retention, result_payload = retain_generation(
                repo,
                first,
                first_payload,
                first_result,
                result_mutation={"successor_obligation_ids": ["FORGED-LOCAL"]},
            )
            repo.git("branch", "forged-local-ref", retention)
            self.assertEqual(repo.git("rev-parse", "forged-local-ref"), retention)
            second = build_second(
                repo,
                first,
                first_payload,
                first_result,
                retention=retention,
                result_payload=result_payload,
            )
            _, result = self.evaluate_built_v2(repo, second, gate=gate)
            self.assertEqual(result["integration_decision"], "BLOCK")
            self.assertIn(
                "exported-successor-obligations-tampered",
                {finding["code"] for finding in result["findings"]},
            )

    def test_v2_retention_commit_must_contain_exactly_record_and_result(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            retention, result_payload = retain_generation(
                repo, first, first_payload, first_result, third_path=True
            )
            second = build_second(
                repo,
                first,
                first_payload,
                first_result,
                retention=retention,
                result_payload=result_payload,
            )
            _, result = self.evaluate_built_v2(repo, second, gate=gate)
            self.assertEqual(result["integration_decision"], "BLOCK")
            self.assertIn(
                "retention-write-set-mismatch",
                {finding["code"] for finding in result["findings"]},
            )

    def test_v2_gate_identity_is_git_verified_and_migration_is_explicit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate_one = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate_one
            )

            forged = copy.deepcopy(first_result)
            forged["deciding_gate"]["artifacts"][0]["sha256"] = "0" * 64
            forged["deciding_gate"]["artifacts"][0]["committed_sha256"] = "0" * 64
            retention, result_payload = retain_generation(
                repo, first, first_payload, forged
            )
            second = build_second(
                repo,
                first,
                first_payload,
                forged,
                retention=retention,
                result_payload=result_payload,
            )
            _, forged_result = self.evaluate_built_v2(repo, second, gate=gate_one)
            self.assertEqual(forged_result["integration_decision"], "BLOCK")
            self.assertIn(
                "link-gate-artifact-digest-mismatch",
                {finding["code"] for finding in forged_result["findings"]},
            )

        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate_one = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate_one
            )
            repo.git("checkout", "-q", "-b", "gate-two", first.commits["predecessor"])
            repo.write(
                "docs/development/tools/check_review_recurrence.py",
                "# synthetic gate revision two\n",
            )
            gate_two_commit = repo.commit(
                "gate revision two",
                ["docs/development/tools/check_review_recurrence.py"],
            )
            gate_two = clean_gate(repo, gate_two_commit)
            second = build_second(repo, first, first_payload, first_result)
            _, migrated_result = self.evaluate_built_v2(
                repo, second, gate=gate_two
            )
            self.assertEqual(migrated_result["integration_decision"], "ALLOW")
            divergence = migrated_result["authority_chain"][
                "validator_identity_divergences"
            ][0]
            self.assertEqual(
                divergence["disposition"], "compatibility_authorized"
            )
            self.assertEqual(
                divergence["historical_decision_disposition"],
                "retained_without_semantic_reproof",
            )
            self.assertFalse(divergence["inherits_current_authority"])
            self.assertEqual(
                divergence["migration"], "current_gate_full_semantic_replay"
            )
            replay = migrated_result["authority_chain"][
                "revalidated_generations"
            ][0]
            self.assertTrue(replay["current_gate_structurally_valid"])
            self.assertEqual(
                replay["current_gate_replayed_integration_decision"], "BLOCK"
            )

    def test_v2_schema_authority_cannot_be_redirected_to_candidate(self) -> None:
        canonical = validator_owned_path(None, ROOT, Path("schema.json"))
        self.assertEqual(canonical, (ROOT / "schema.json").resolve())
        with self.assertRaisesRegex(ValueError, "validator source"):
            validator_owned_path(
                Path("/tmp/candidate-weakened-schema.json"),
                ROOT,
                Path("schema.json"),
            )

    def test_v2_timed_runner_marks_in_tree_outputs_and_index_flags_nonpassing(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            repo.write(".gitignore", "evidence/\n")
            repo.write("tracked.txt", "tracked\n")
            repo.commit(
                "ignore synthetic evidence", [".gitignore", "tracked.txt"]
            )
            repo.git("update-index", "--skip-worktree", "tracked.txt")
            payload = run_gate_v2(
                [sys.executable, "-c", "pass"],
                repo.root,
                repo.root / "evidence/gate.log",
                10,
                result_path=repo.root / "evidence/gate.result.json",
            )
            self.assertTrue(payload["worktree"]["log_inside_source_worktree"])
            self.assertTrue(payload["worktree"]["result_inside_source_worktree"])
            self.assertFalse(payload["worktree"]["index_flags_clean"])
            facts = TimedFacts(
                validation_id="focused",
                result_revision=payload["source"]["revision"],
                result_tree=payload["source"]["tree"],
                status=payload["status"],
                exit_code=payload["exit_code"],
                timed_out=payload["timed_out"],
                log_complete=payload["log"]["complete"],
                clean_before=payload["worktree"]["clean_before"],
                clean_after=payload["worktree"]["clean_after"],
                index_flags_clean=payload["worktree"]["index_flags_clean"],
                head_unchanged=payload["worktree"]["head_unchanged"],
                log_inside_source_worktree=payload["worktree"][
                    "log_inside_source_worktree"
                ],
                result_inside_source_worktree=payload["worktree"][
                    "result_inside_source_worktree"
                ],
                log_bytes=payload["log"]["bytes"],
                log_sha256=payload["log"]["sha256"],
            )
            self.assertFalse(facts.passing)

    def test_v2_timed_runner_tracks_head_switch_and_transient_index_flags(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            repo_root = root / "repo"
            repo_root.mkdir()
            repo = FixtureRepo(repo_root)
            repo.write("tracked.txt", "base\n")
            repo.commit("base", ["tracked.txt"])
            repo.write("tracked.txt", "head\n")
            repo.commit("head", ["tracked.txt"])
            payload = run_gate_v2(
                ["git", "checkout", "-q", "HEAD^"],
                repo.root,
                root / "head-switch.log",
                10,
                result_path=root / "head-switch.result.json",
            )
            self.assertTrue(payload["worktree"]["clean_before"])
            self.assertTrue(payload["worktree"]["clean_after"])
            self.assertFalse(payload["worktree"]["head_unchanged"])
            self.assertNotEqual(
                payload["source"]["revision"],
                payload["worktree"]["head_revision_after"],
            )
            self.assertTrue(
                any("mutates and restores" in limit for limit in payload["limits"])
            )

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            repo_root = root / "repo"
            repo_root.mkdir()
            repo = FixtureRepo(repo_root)
            repo.write("tracked.txt", "tracked\n")
            repo.commit("tracked", ["tracked.txt"])
            repo.git("update-index", "--skip-worktree", "tracked.txt")
            payload = run_gate_v2(
                ["git", "update-index", "--no-skip-worktree", "tracked.txt"],
                repo.root,
                root / "index-union.log",
                10,
                result_path=root / "index-union.result.json",
            )
            self.assertTrue(payload["worktree"]["clean_before"])
            self.assertTrue(payload["worktree"]["clean_after"])
            self.assertFalse(payload["worktree"]["index_flags_clean"])
            self.assertTrue(payload["worktree"]["index_flagged_entries"])
            self.assertTrue(repo.git("ls-files", "-v", "tracked.txt").startswith("H "))

    def test_v2_dirty_before_and_after_timed_evidence_blocks_rooted_allow(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            second = build_second(repo, first, first_payload, first_result)

            def dirty(payload: dict[str, object]) -> None:
                worktree = payload["worktree"]
                worktree["clean_before"] = False
                worktree["status_before"] = [" M product.txt"]
                worktree["clean_after"] = False
                worktree["status_after"] = ["?? scratch.tmp"]

            self.amend_timed_result(repo, second, dirty)
            _, result = self.evaluate_built_v2(repo, second, gate=gate)
            self.assertEqual(result["integration_decision"], "BLOCK")
            self.assertIn(
                "closure-validation-not-passing",
                {finding["code"] for finding in result["findings"]},
            )
            self.assertTrue(
                any("clean-before" in blocker for blocker in result["blockers"])
            )

    def test_v2_malformed_control_path_returns_structured_block(self) -> None:
        for label, malformed in (
            ("nul", "bad\x00path"),
            ("lone-surrogate", "bad\udcffpath"),
        ):
            with self.subTest(label=label), tempfile.TemporaryDirectory() as temporary:
                repo = FixtureRepo(Path(temporary))
                built = build_bootstrap(repo)
                built.record["artifacts"][0]["path"] = malformed
                _, result = self.evaluate_built_v2(repo, built)
                self.assertEqual(result["integration_decision"], "BLOCK")
                codes = {finding["code"] for finding in result["findings"]}
                if label == "lone-surrogate":
                    self.assertIn("unsafe-json-surrogate", codes)
                else:
                    self.assertIn("schema-pattern", codes)
                self.assertIn("unsafe-authored-repository-path", codes)
                self.assertNotIn("unsafe-artifact-path", codes)

    def test_v2_result_path_must_be_outside_both_repositories(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            analysed = root / "analysed"
            validator = root / "validator"
            analysed.mkdir()
            validator.mkdir()

            with self.assertRaisesRegex(ValueError, "requires --result"):
                external_result_path(None, analysed, validator)
            with self.assertRaisesRegex(ValueError, "outside the analysed"):
                external_result_path(analysed / "result.json", analysed, validator)
            with self.assertRaisesRegex(ValueError, "outside the validator"):
                external_result_path(validator / "result.json", analysed, validator)

            outside = root / "staging" / "result.json"
            self.assertEqual(
                external_result_path(outside, analysed, validator),
                outside.resolve(),
            )

    def test_v2_live_anchor_requires_exact_separate_clean_validator_pin(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            analysed_root = root / "analysed"
            analysed_root.mkdir()
            analysed = FixtureRepo(analysed_root)
            built = build_bootstrap(analysed)
            anchor = self.live_anchor(built)

            for name, mutate, expected_code in (
                ("exact", lambda _anchor: None, None),
                (
                    "missing-generation",
                    lambda item: item["arms"][0].pop("generation"),
                    "schema-required",
                ),
                (
                    "wrong-generation",
                    lambda item: item["arms"][0].__setitem__("generation", 2),
                    "live-arm-generation-mismatch",
                ),
                (
                    "unknown-arm",
                    lambda item: item["arms"][0].__setitem__("arm", "unknown"),
                    "schema-enum",
                ),
                (
                    "extra-top-level",
                    lambda item: item.__setitem__("authority_override", True),
                    "schema-additional-property",
                ),
                (
                    "wrong-mode",
                    lambda item: item["arms"][0]["repair_packet"].__setitem__(
                        "mode", "100755"
                    ),
                    "schema-const",
                ),
                (
                    "mismatched-manifest",
                    lambda item: item["arms"][0]["repair_manifest"].__setitem__(
                        "sha256", "0" * 64
                    ),
                    "live-arm-pin-mismatch",
                ),
                (
                    "duplicate-arm",
                    lambda item: item["arms"].append(
                        {
                            **copy.deepcopy(item["arms"][0]),
                            "repair_manifest": {
                                **copy.deepcopy(
                                    item["arms"][0]["repair_manifest"]
                                ),
                                "sha256": "0" * 64,
                            },
                        }
                    ),
                    "duplicate-live-arm-anchor-arm",
                ),
            ):
                with self.subTest(name=name):
                    validator_root = root / f"validator-{name}"
                    validator_root.mkdir()
                    validator = FixtureRepo(validator_root)
                    candidate_anchor = copy.deepcopy(anchor)
                    mutate(candidate_anchor)
                    validator.write(
                        "docs/development/tools/check_review_recurrence.py",
                        "# clean committed validator gate\n",
                    )
                    validator.write(
                        LIVE_ARM_ANCHOR_PATH,
                        json.dumps(candidate_anchor, indent=2, sort_keys=True) + "\n",
                    )
                    validator_commit = validator.commit(
                        "pin live arm",
                        [
                            "docs/development/tools/check_review_recurrence.py",
                            LIVE_ARM_ANCHOR_PATH,
                        ],
                    )
                    gate = clean_gate(validator, validator_commit)
                    _, result = self.evaluate_built_v2(
                        analysed,
                        built,
                        gate=gate,
                        live_arm="cargo",
                        validator_root=validator_root,
                    )
                    codes = {finding["code"] for finding in result["findings"]}
                    if expected_code is None:
                        self.assertEqual(codes, set())
                        self.assertEqual(
                            result["live_arm_pin"]["anchor"]["validator_commit"],
                            validator_commit,
                        )
                    else:
                        self.assertIn(expected_code, codes)
                        self.assertEqual(result["integration_decision"], "BLOCK")

            validator_root = root / "validator-uncommitted-anchor"
            validator_root.mkdir()
            validator = FixtureRepo(validator_root)
            validator.write(
                "docs/development/tools/check_review_recurrence.py",
                "# clean committed validator gate\n",
            )
            validator.write(
                LIVE_ARM_ANCHOR_PATH,
                json.dumps(anchor, indent=2, sort_keys=True) + "\n",
            )
            validator_commit = validator.commit(
                "pin live arm",
                [
                    "docs/development/tools/check_review_recurrence.py",
                    LIVE_ARM_ANCHOR_PATH,
                ],
            )
            gate = clean_gate(validator, validator_commit)
            changed_anchor = copy.deepcopy(anchor)
            changed_anchor["arms"][0]["repair_manifest"]["sha256"] = "0" * 64
            validator.write(
                LIVE_ARM_ANCHOR_PATH,
                json.dumps(changed_anchor, indent=2, sort_keys=True) + "\n",
            )
            _, result = self.evaluate_built_v2(
                analysed,
                built,
                gate=gate,
                live_arm="cargo",
                validator_root=validator_root,
            )
            self.assertIn(
                "live-arm-anchor-uncommitted",
                {finding["code"] for finding in result["findings"]},
            )

    def test_v2_retained_live_anchor_rebinds_historical_validator_blob(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            analysed_root = root / "analysed"
            validator_root = root / "validator"
            analysed_root.mkdir()
            validator_root.mkdir()
            analysed = FixtureRepo(analysed_root)
            validator = FixtureRepo(validator_root)
            built = build_bootstrap(analysed)
            record, parse_findings = parse_record(built.record, "record")
            self.assertEqual(parse_findings, [])
            self.assertIsNotNone(record)
            artifacts = record.artifact_index()
            anchor_payload = self.live_anchor(built)
            validator.write(
                "docs/development/tools/check_review_recurrence.py",
                "# historical clean validator\n",
            )
            validator.write(
                LIVE_ARM_ANCHOR_PATH,
                json.dumps(anchor_payload, indent=2, sort_keys=True) + "\n",
            )
            validator_commit = validator.commit(
                "historical validator anchor",
                [
                    "docs/development/tools/check_review_recurrence.py",
                    LIVE_ARM_ANCHOR_PATH,
                ],
            )
            historical_gate = clean_gate(validator, validator_commit)
            valid_pin = {
                "state": "verified",
                "anchor": {
                    "validator_commit": validator_commit,
                    "path": LIVE_ARM_ANCHOR_PATH,
                    "mode": "100644",
                    "sha256": validator.blob_digest(
                        validator_commit, LIVE_ARM_ANCHOR_PATH
                    ),
                },
                "arm": anchor_payload["arms"][0],
            }
            retained_result = {
                "live_arm": "cargo",
                "live_arm_pin": valid_pin,
            }
            self.assertEqual(
                verify_retained_live_arm_anchor(
                    validator_root,
                    "cargo",
                    record,
                    artifacts,
                    retained_result,
                    historical_gate,
                    self.v2_anchor_schema,
                ),
                [],
            )

            missing = copy.deepcopy(retained_result)
            missing["live_arm_pin"] = {
                "state": "unavailable",
                "anchor": None,
                "arm": None,
            }
            missing_findings = verify_retained_live_arm_anchor(
                validator_root,
                "cargo",
                record,
                artifacts,
                missing,
                historical_gate,
                self.v2_anchor_schema,
            )
            self.assertIn(
                "retained-live-arm-pin-unavailable",
                {finding.code for finding in missing_findings},
            )

            forged = copy.deepcopy(retained_result)
            forged["live_arm_pin"]["anchor"]["sha256"] = "0" * 64
            forged_findings = verify_retained_live_arm_anchor(
                validator_root,
                "cargo",
                record,
                artifacts,
                forged,
                historical_gate,
                self.v2_anchor_schema,
            )
            self.assertIn(
                "retained-live-arm-anchor-digest-mismatch",
                {finding.code for finding in forged_findings},
            )

    def test_v2_history_replay_enforces_retained_live_anchor(self) -> None:
        cases = (
            ("valid", None, None),
            (
                "missing-pin",
                lambda result: result.__setitem__(
                    "live_arm_pin",
                    {"state": "unavailable", "anchor": None, "arm": None},
                ),
                "retained-live-arm-pin-unavailable",
            ),
            (
                "forged-digest",
                lambda result: result["live_arm_pin"]["anchor"].__setitem__(
                    "sha256", "0" * 64
                ),
                "retained-live-arm-anchor-digest-mismatch",
            ),
            ("validator-ancestry", None, "retained-live-arm-validator-revision-unrooted"),
        )
        for name, mutate_result, expected_code in cases:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                analysed_root = root / "analysed"
                validator_root = root / "validator"
                analysed_root.mkdir()
                validator_root.mkdir()
                analysed = FixtureRepo(analysed_root)
                validator = FixtureRepo(validator_root)
                first = build_bootstrap(analysed)
                anchor_payload = self.live_anchor(first)
                validator.write("README.md", "validator base\n")
                validator.commit("validator base", ["README.md"])
                validator.write(
                    "docs/development/tools/check_review_recurrence.py",
                    "# historical validator gate\n",
                )
                validator.write(
                    LIVE_ARM_ANCHOR_PATH,
                    json.dumps(anchor_payload, indent=2, sort_keys=True) + "\n",
                )
                historical_revision = validator.commit(
                    "historical validator anchor",
                    [
                        "docs/development/tools/check_review_recurrence.py",
                        LIVE_ARM_ANCHOR_PATH,
                    ],
                )
                historical_gate = clean_gate(validator, historical_revision)
                first_payload, first_result = self.evaluate_built_v2(
                    analysed,
                    first,
                    gate=historical_gate,
                    live_arm="cargo",
                    validator_root=validator_root,
                )
                self.assertEqual(first_result["finding_count"], 0)
                retained_result = copy.deepcopy(first_result)
                if mutate_result is not None:
                    mutate_result(retained_result)
                retention, result_payload = retain_generation(
                    analysed, first, first_payload, retained_result
                )
                second = build_second(
                    analysed,
                    first,
                    first_payload,
                    retained_result,
                    retention=retention,
                    result_payload=result_payload,
                )

                current_gate = historical_gate
                if name == "validator-ancestry":
                    validator_root_commit = validator.git(
                        "rev-list", "--max-parents=0", historical_revision
                    )
                    validator.git(
                        "checkout",
                        "-q",
                        "-b",
                        "current-validator",
                        validator_root_commit,
                    )
                    validator.write(
                        "docs/development/tools/check_review_recurrence.py",
                        "# unrelated current validator gate\n",
                    )
                    validator.write(
                        LIVE_ARM_ANCHOR_PATH,
                        json.dumps(anchor_payload, indent=2, sort_keys=True) + "\n",
                    )
                    current_revision = validator.commit(
                        "unrelated current validator",
                        [
                            "docs/development/tools/check_review_recurrence.py",
                            LIVE_ARM_ANCHOR_PATH,
                        ],
                    )
                    current_gate = clean_gate(validator, current_revision)

                record, parse_findings = parse_record(second.record, "record")
                self.assertEqual(parse_findings, [])
                self.assertIsNotNone(record)
                chain, chain_findings = (
                    v2_authority.resolve_predecessor_authority_chain(
                        GitRepo(analysed_root),
                        record,
                        schema=self.v2_schema,
                        result_schema=self.v2_result_schema,
                        deciding_gate=current_gate,
                        validator_repo=GitRepo(validator_root),
                        legacy_roots=second.roots,
                    )
                )
                self.assertEqual(chain_findings, [])
                self.assertEqual(
                    chain.links[0].deciding_gate["source_revision"],
                    historical_revision,
                )
                if name == "validator-ancestry":
                    self.assertFalse(
                        GitRepo(validator_root).is_ancestor(
                            historical_revision, current_gate["source_revision"]
                        )
                    )
                with mock.patch.object(
                    v2_history, "verify_canonical_port_packet", return_value=[]
                ):
                    replay = v2_history.revalidate_history(
                        GitRepo(analysed_root),
                        chain,
                        schema=self.v2_schema,
                        manifest_schema=self.v2_manifest_schema,
                        timed_schema=self.v2_timed_schema,
                        result_schema=self.v2_result_schema,
                        anchor_schema=self.v2_anchor_schema,
                        legacy_roots=second.roots,
                        enforce_canonical_ports=True,
                        validator_root=validator_root,
                        live_arm="cargo",
                    )
                codes = {finding.code for finding in replay.findings}
                if expected_code is None:
                    self.assertEqual(codes, set())
                    self.assertTrue(replay.chain.rooted)
                else:
                    self.assertIn(expected_code, codes)
                    self.assertFalse(replay.chain.rooted)

    def test_v2_replayed_chain_keeps_legacy_root_for_prior_evidence_roles(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            second = build_second(repo, first, first_payload, first_result)
            record, parse_findings = parse_record(second.record, "record")
            self.assertEqual(parse_findings, [])
            self.assertIsNotNone(record)
            chain, chain_findings = v2_authority.resolve_predecessor_authority_chain(
                GitRepo(repo.root),
                record,
                schema=self.v2_schema,
                result_schema=self.v2_result_schema,
                deciding_gate=gate,
                validator_repo=GitRepo(repo.root),
                legacy_roots=second.roots,
            )
            self.assertEqual(chain_findings, [])
            replay = v2_history.revalidate_history(
                GitRepo(repo.root),
                chain,
                schema=self.v2_schema,
                manifest_schema=self.v2_manifest_schema,
                timed_schema=self.v2_timed_schema,
                result_schema=self.v2_result_schema,
                anchor_schema=self.v2_anchor_schema,
                legacy_roots=second.roots,
            )
            root_commit = first.commits["legacy_root"]
            self.assertTrue(replay.chain.rooted)
            self.assertIn(root_commit, replay.chain.verified_commits)

            root_path = first.paths["legacy_report"]
            foreign_base = repo.git("rev-list", "--max-parents=0", root_commit)
            repo.git("checkout", "-q", "-b", "foreign-evidence", foreign_base)
            repo.write("foreign-evidence.txt", "unverified evidence\n")
            foreign_commit = repo.commit(
                "unverified evidence", ["foreign-evidence.txt"]
            )
            for role in ("prior_closure_evidence", "non_reachability_evidence"):
                with self.subTest(role=role):
                    permitted = ArtifactRef(
                        ref_id=f"legacy-{role}",
                        role=role,
                        commit=root_commit,
                        path=root_path,
                        sha256=repo.blob_digest(root_commit, root_path),
                        mode="100644",
                    )
                    permitted_record = replace(
                        record, artifacts=(*record.artifacts, permitted)
                    )
                    _, permitted_findings = resolve_artifacts(
                        GitRepo(repo.root),
                        permitted_record,
                        replay.chain,
                        second.roots,
                    )
                    self.assertEqual(permitted_findings, [])

                    rejected = replace(
                        permitted,
                        ref_id=f"foreign-{role}",
                        commit=foreign_commit,
                        path="foreign-evidence.txt",
                        sha256=repo.blob_digest(
                            foreign_commit, "foreign-evidence.txt"
                        ),
                    )
                    rejected_record = replace(
                        record, artifacts=(*record.artifacts, rejected)
                    )
                    _, rejected_findings = resolve_artifacts(
                        GitRepo(repo.root),
                        rejected_record,
                        replay.chain,
                        second.roots,
                    )
                    self.assertIn(
                        "artifact-not-on-ancestry-or-verified-generation",
                        {finding.code for finding in rejected_findings},
                    )

    def test_v2_timed_result_rejects_wrong_tree_and_v1_schema(self) -> None:
        mutations = (
            (
                "wrong-revision",
                lambda payload: payload["source"].__setitem__(
                    "revision", "0" * 40
                ),
                "timed-result-revision-mismatch",
            ),
            (
                "wrong-tree",
                lambda payload: payload["source"].__setitem__("tree", "0" * 40),
                "timed-result-tree-mismatch",
            ),
            (
                "fabricated-log-binding",
                lambda payload: payload["log"].__setitem__("sha256", "0" * 64),
                "timed-log-digest-mismatch",
            ),
            (
                "v1-result",
                lambda payload: payload.__setitem__(
                    "schema", "codefire.timed_gate.v1"
                ),
                "timed-result-wrong-schema",
            ),
        )
        for name, mutate, expected_code in mutations:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                repo = FixtureRepo(Path(temporary))
                built = build_bootstrap(repo)
                self.amend_timed_result(repo, built, mutate)
                _, result = self.evaluate_built_v2(repo, built)
                self.assertEqual(result["integration_decision"], "BLOCK")
                self.assertIn(
                    expected_code,
                    {finding["code"] for finding in result["findings"]},
                )

    def test_v2_references_reject_dangling_and_role_incompatible_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            built = build_bootstrap(repo)
            original = copy.deepcopy(built.record)
            mutations = (
                (
                    "dangling",
                    lambda record: record["path_reviews"][0]["surfaces"][0].update(
                        evidence_ref_ids=["missing-artifact"]
                    ),
                    "dangling-artifact-ref",
                ),
                (
                    "wrong-role",
                    lambda record: record["path_reviews"][0]["surfaces"][0].update(
                        evidence_ref_ids=["packet"]
                    ),
                    "role-incompatible-artifact-ref",
                ),
                (
                    "wrong-role-commit",
                    lambda record: record["artifacts"][3].update(
                        commit=record["subject"]["candidate_revision"]
                    ),
                    "artifact-role-commit-mismatch",
                ),
            )
            for name, mutate, expected_code in mutations:
                with self.subTest(name=name):
                    built.record = copy.deepcopy(original)
                    mutate(built.record)
                    _, result = self.evaluate_built_v2(repo, built)
                    self.assertEqual(result["integration_decision"], "BLOCK")
                    self.assertIn(
                        expected_code,
                        {finding["code"] for finding in result["findings"]},
                    )

    def test_v2_authority_generation_skip_and_unrooted_chain_block(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            skipped = build_second(
                repo, first, first_payload, first_result, generation=3
            )
            _, result = self.evaluate_built_v2(repo, skipped, gate=gate)
            self.assertEqual(result["integration_decision"], "BLOCK")
            self.assertIn(
                "authority-generation-discontinuous",
                {finding["code"] for finding in result["findings"]},
            )

        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            unrooted = build_second(repo, first, first_payload, first_result)
            unrooted.roots = {}
            repo.git(
                "branch", "unrooted-local-ref", unrooted.commits["retention"]
            )
            self.assertEqual(
                repo.git("rev-parse", "unrooted-local-ref"),
                unrooted.commits["retention"],
            )
            _, result = self.evaluate_built_v2(repo, unrooted, gate=gate)
            self.assertEqual(result["integration_decision"], "BLOCK")
            codes = {finding["code"] for finding in result["findings"]}
            self.assertIn("unpinned-legacy-authority-root", codes)
            self.assertIn("prior-authority-not-rooted", codes)

    def test_v2_authority_cycle_and_64_65_link_boundary(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            template = build_bootstrap(repo).record

        cycle_chain, cycle_findings = self.mocked_authority_walk(
            template, 1, cycle=True
        )
        self.assertFalse(cycle_chain.rooted)
        self.assertIn(
            "authority-chain-cycle", {finding.code for finding in cycle_findings}
        )

        accepted_chain, accepted_findings = self.mocked_authority_walk(template, 64)
        self.assertEqual(accepted_findings, [])
        self.assertTrue(accepted_chain.rooted)
        self.assertEqual(len(accepted_chain.links), 64)

        blocked_chain, blocked_findings = self.mocked_authority_walk(template, 65)
        self.assertFalse(blocked_chain.rooted)
        self.assertIn(
            "authority-chain-too-deep",
            {finding.code for finding in blocked_findings},
        )

    def test_v2_no_git_and_v1_normal_entry_are_always_blocks(self) -> None:
        v1 = legacy_v1_block(self.record(), "a" * 64)
        self.assertEqual(v1["integration_decision"], "BLOCK")
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            built = build_bootstrap(repo)
            _, result = self.evaluate_built_v2(repo, built, verify_git=False)
            self.assertEqual(result["integration_decision"], "BLOCK")
            self.assertTrue(
                any("--no-git-verify" in blocker for blocker in result["blockers"])
            )

    def test_v2_record_and_manifest_burden_measurement_is_reproducible(self) -> None:
        minimal = measure(SHAPES["minimal"])
        realistic = measure(SHAPES["realistic"])
        self.assertEqual(
            (
                minimal["record"]["total"],
                minimal["manifest"]["total"],
                minimal["combined_total"],
            ),
            (111, 19, 130),
        )
        self.assertEqual(
            (
                realistic["record"]["total"],
                realistic["manifest"]["total"],
                realistic["combined_total"],
            ),
            (314, 46, 360),
        )
        self.assertIn("do not prove", realistic["semantic_boundary"])

    def test_v2_canonical_port_packet_identities_match_git_objects(self) -> None:
        for arm, packet in PORT_PACKETS.items():
            with self.subTest(arm=arm):
                payload = subprocess.run(
                    ["git", "show", f"{packet.source_commit}:{packet.path}"],
                    cwd=ROOT,
                    check=True,
                    capture_output=True,
                ).stdout
                self.assertEqual(hashlib.sha256(payload).hexdigest(), packet.sha256)
                absent = subprocess.run(
                    ["git", "cat-file", "-e", f"{packet.absent_at}:{packet.path}"],
                    cwd=ROOT,
                    check=False,
                    capture_output=True,
                )
                self.assertNotEqual(absent.returncode, 0)

        required = frozenset(
            {
                "docs/development/ai/swarm-batches/"
                "development-flow-kgi-kpi-zero-residual-repair-r3-20260731.md",
                "docs/development/ai/swarm-batches/"
                "development-flow-kgi-kpi-zero-residual-repair-r4-20260731.md",
            }
        )
        self.assertEqual(PORT_REQUIRED_REPAIR_PATHS["kgi_kpi"], required)
        self.assertTrue(
            all(path.startswith("docs/development/ai/swarm-batches/") for path in required)
        )

        with tempfile.TemporaryDirectory() as temporary:
            fixture = build_bootstrap(FixtureRepo(Path(temporary)))
            record, parse_findings = parse_record(fixture.record, "record")
            self.assertEqual(parse_findings, [])
            self.assertIsNotNone(record)
            port = PORT_PACKETS["kgi_kpi"]
            subject = replace(
                record.subject,
                changed_paths=frozenset(),
                authority=replace(record.subject.authority, bootstrap_arm="kgi_kpi"),
            )
            narrowed = replace(record, subject=subject)
            packet = ArtifactRef(
                ref_id=narrowed.subject.repair_packet_ref,
                role="repair_packet",
                commit=port.source_commit,
                path=port.path,
                sha256=port.sha256,
                mode="100644",
            )
            findings = verify_canonical_port_packet(
                GitRepo(ROOT), narrowed, {packet.ref_id: packet}, "kgi_kpi"
            )
            self.assertIn(
                "canonical-port-required-repair-paths-missing",
                {finding.code for finding in findings},
            )
            covered = replace(
                narrowed,
                subject=replace(narrowed.subject, changed_paths=required),
            )
            covered_findings = verify_canonical_port_packet(
                GitRepo(ROOT), covered, {packet.ref_id: packet}, "kgi_kpi"
            )
            self.assertNotIn(
                "canonical-port-required-repair-paths-missing",
                {finding.code for finding in covered_findings},
            )

    def test_v2_strict_json_boundary_is_deterministic(self) -> None:
        cases = (
            (b'{"generation":1,"generation":2}', "duplicate-json-object-key"),
            (
                b'{"artifact":{"sha256":"a","sha256":"b"}}',
                "duplicate-json-object-key",
            ),
            (
                b'{"claim":{"kind":"parity","kind":"closure"}}',
                "duplicate-json-object-key",
            ),
            (
                b'{"worktree":{"clean_before":true,"clean_before":false}}',
                "duplicate-json-object-key",
            ),
            (
                b'{"candidate":{"candidate_tree":"a","candidate_tree":"b"}}',
                "duplicate-json-object-key",
            ),
            (
                b'{"pin":{"sha256":"a","sha256":"b"}}',
                "duplicate-json-object-key",
            ),
            (b'{"value":NaN}', "non-finite-json-number"),
            (b'{"value":Infinity}', "non-finite-json-number"),
            (b'{"value":-Infinity}', "non-finite-json-number"),
            (b'{"value":1e999}', "non-finite-json-number"),
            (b'{"value":-1e999}', "non-finite-json-number"),
            (b'{"value":"\\udcff"}', "unsafe-json-surrogate"),
            (("1" * 5000).encode("ascii"), "json-number-out-of-range"),
        )
        for payload, expected in cases:
            with self.subTest(expected=expected, payload=payload[:40]):
                value, findings = strict_decode_json(payload, "boundary")
                self.assertIsNone(value)
                self.assertEqual({finding.code for finding in findings}, {expected})

        depth_64 = ("[" * 64 + "0" + "]" * 64).encode("ascii")
        value, findings = strict_decode_json(depth_64, "boundary")
        self.assertEqual(findings, [])
        self.assertIsInstance(value, list)

        depth_65 = ("[" * 65 + "0" + "]" * 65).encode("ascii")
        value, findings = strict_decode_json(depth_65, "boundary")
        self.assertIsNone(value)
        self.assertEqual(
            {finding.code for finding in findings}, {"json-nesting-too-deep"}
        )

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            record_path = root / "duplicate-record.json"
            result_path = root / "diagnostic-result.json"
            record_path.write_bytes(b'{"schema":"x","schema":"y"}')
            with mock.patch("sys.stdout"):
                status = run_review_recurrence_gate(
                    ["--record", str(record_path), "--result", str(result_path)]
                )
            self.assertEqual(status, 3)
            diagnostic = json.loads(result_path.read_text(encoding="utf-8"))
            self.assertIn(
                "duplicate-json-object-key",
                {finding["code"] for finding in diagnostic["findings"]},
            )

        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            repo.write("retained.json", b'{"value":1,"value":2}')
            commit = repo.commit("duplicate retained object", ["retained.json"])
            retained_findings: list[object] = []
            loaded = v2_authority._load_json_blob(
                GitRepo(repo.root),
                commit,
                "retained.json",
                "history.result",
                retained_findings,
            )
            self.assertIsNone(loaded)
            self.assertIn(
                "duplicate-json-object-key",
                {finding.code for finding in retained_findings},
            )

        for relative, schema_id in AUTHORITY_SCHEMA_IDS.items():
            with self.subTest(schema=relative), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(
                    '{"$id":' + json.dumps(schema_id) + ',"$id":"forged"}',
                    encoding="utf-8",
                )
                with self.assertRaisesRegex(
                    ValueError, "duplicate-json-object-key"
                ):
                    load_authority_schema(None, root, Path(relative))

    def test_v2_path_fact_is_separate_from_locus_and_manifest_path_is_bound(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            built = build_bootstrap(repo, "byte_identical_locus")
            derivation = built.record["derivations"][0]
            derivation["locus_existed_in_predecessor"] = False
            _, result = self.evaluate_built_v2(repo, built)
            self.assertEqual(result["finding_count"], 0)
            coverage = result["metrics"]["i1"]["claim_coverage"]
            self.assertEqual(len(coverage), 1)
            self.assertTrue(coverage[0]["path_existed_in_predecessor"])
            self.assertFalse(coverage[0]["locus_existed_in_predecessor"])
            self.assertEqual(coverage[0]["locus_path"], "product.txt")

            derivation["path_existed_in_predecessor"] = False
            _, contradicted = self.evaluate_built_v2(repo, built)
            self.assertIn(
                "predecessor-path-existence-contradicted",
                {finding["code"] for finding in contradicted["findings"]},
            )

            derivation["path_existed_in_predecessor"] = True
            derivation["path"] = "README.md"
            _, mismatched = self.evaluate_built_v2(repo, built)
            self.assertIn(
                "manifest-derivation-path-mismatch",
                {finding["code"] for finding in mismatched["findings"]},
            )

    def test_v2_shared_path_probe_fails_identically_for_current_and_history(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            built = build_bootstrap(repo)
            record, parse_findings = parse_record(built.record, "record")
            self.assertEqual(parse_findings, [])
            self.assertIsNotNone(record)

            for spelling in (".", "a/./b", "a//b", "a/"):
                invalid = replace(
                    record,
                    derivations=(replace(record.derivations[0], path=spelling),),
                )
                current, current_findings = predecessor_path_presence(
                    GitRepo(repo.root), invalid, "record.derivations"
                )
                retained, retained_findings = predecessor_path_presence(
                    GitRepo(repo.root), invalid, "history.record.derivations"
                )
                self.assertEqual(current, {})
                self.assertEqual(retained, {})
                self.assertEqual(
                    {finding.code for finding in current_findings},
                    {"unsafe-derivation-path"},
                )
                self.assertEqual(
                    {finding.code for finding in retained_findings},
                    {"unsafe-derivation-path"},
                )

            failing_repo = mock.Mock()
            failing_repo.entry.side_effect = GitError("forced tree-entry failure")
            current, current_findings = predecessor_path_presence(
                failing_repo, record, "record.derivations"
            )
            retained, retained_findings = predecessor_path_presence(
                failing_repo, record, "history.record.derivations"
            )
            self.assertEqual(current, {})
            self.assertEqual(retained, {})
            self.assertEqual(
                {finding.code for finding in current_findings},
                {"predecessor-path-probe-failed"},
            )
            self.assertEqual(
                {finding.code for finding in retained_findings},
                {"predecessor-path-probe-failed"},
            )

    def test_v2_derivation_route_table_and_exact_i1_counts(self) -> None:
        cases = (
            ("derivation_complete", "REPAIR_CLAIM"),
            ("derivation_contradicted", "NONE"),
            ("derivation_unavailable", "NONE"),
        )
        for disposition, action in cases:
            with self.subTest(disposition=disposition, action=action), tempfile.TemporaryDirectory() as temporary:
                repo = FixtureRepo(Path(temporary))
                built = build_bootstrap(repo)
                row = built.record["derivations"][0]
                row["disposition"] = disposition
                row["next_action"] = action
                if disposition == "derivation_unavailable":
                    row["evidence"] = {"state": "unavailable", "ref_ids": []}
                _, result = self.evaluate_built_v2(repo, built)
                self.assertIn(
                    "derivation-action-route-mismatch",
                    {finding["code"] for finding in result["findings"]},
                )

        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            built = build_bootstrap(repo)
            _, result = self.evaluate_built_v2(repo, built)
            counts = result["metrics"]["i1"]["derivation_next_action_counts"]
            self.assertEqual(list(counts), list(NEXT_ACTIONS))
            self.assertEqual(counts["NONE"], 1)
            self.assertEqual(sum(counts.values()), 1)
            self.assertEqual(result["metrics"]["i1"]["blocking_derivations"], 0)

    def test_v2_semantic_ids_are_unique_before_indexing(self) -> None:
        for section, mutate, expected in (
            (
                "derivations",
                lambda row: row.update(locus="different:locus"),
                "duplicate-record-derivation-claim-id",
            ),
            (
                "validations",
                lambda row: row.update(status="failed", exit_code=1),
                "duplicate-record-validation-id",
            ),
        ):
            with self.subTest(section=section), tempfile.TemporaryDirectory() as temporary:
                repo = FixtureRepo(Path(temporary))
                built = build_bootstrap(repo)
                duplicate = copy.deepcopy(built.record[section][0])
                mutate(duplicate)
                built.record[section].append(duplicate)
                _, result = self.evaluate_built_v2(repo, built)
                self.assertEqual(result["integration_decision"], "BLOCK")
                self.assertIn(
                    expected, {finding["code"] for finding in result["findings"]}
                )

        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            built = build_bootstrap(repo)
            manifest = json.loads(
                subprocess.run(
                    [
                        "git",
                        "show",
                        f"{built.commits['packet']}:{built.paths['manifest']}",
                    ],
                    cwd=repo.root,
                    check=True,
                    capture_output=True,
                    text=True,
                ).stdout
            )
            cases = (
                (
                    "prior_obligations",
                    lambda row: row.update(closure_validation_ids=[]),
                    "duplicate-manifest-obligation-id",
                ),
                (
                    "parity_closure_claims",
                    lambda row: row.update(locus_path="README.md"),
                    "duplicate-manifest-claim-id",
                ),
                (
                    "required_validations",
                    lambda row: row.update(argv=["different"]),
                    "duplicate-manifest-validation-id",
                ),
            )
            for section, mutate, expected in cases:
                with self.subTest(manifest_section=section):
                    candidate = copy.deepcopy(manifest)
                    duplicate = copy.deepcopy(candidate[section][0])
                    mutate(duplicate)
                    candidate[section].append(duplicate)
                    parsed, findings = parse_manifest(candidate, "manifest")
                    self.assertIsNone(parsed)
                    self.assertIn(expected, {finding.code for finding in findings})

    def test_v2_timed_relations_fail_closed_without_partial_facts(self) -> None:
        mutations = (
            (
                "clean-summary",
                lambda payload: payload["worktree"].__setitem__("clean_before", False),
                "timed-clean-before-inconsistent",
            ),
            (
                "head-probe-shape",
                lambda payload: payload["worktree"].__setitem__(
                    "head_probe_error", "probe failed"
                ),
                "timed-head-probe-shape-inconsistent",
            ),
            (
                "half-head-identity",
                lambda payload: payload["worktree"].__setitem__(
                    "head_tree_after", None
                ),
                "timed-head-probe-shape-inconsistent",
            ),
            (
                "result-collision",
                lambda payload: payload["result"].__setitem__(
                    "path", payload["log"]["path"]
                ),
                "timed-log-result-path-collision",
            ),
            (
                "noncanonical-cwd",
                lambda payload: payload.__setitem__(
                    "cwd", "/synthetic/checkout/../escape"
                ),
                "timed-path-not-canonical",
            ),
            (
                "component-prefix-escape",
                lambda payload: payload.__setitem__(
                    "cwd", "/synthetic/checkout-sibling"
                ),
                "timed-cwd-outside-source-root",
            ),
            (
                "log-inside-with-false-summary",
                lambda payload: payload["log"].__setitem__(
                    "path", "/synthetic/checkout/focused.log"
                ),
                "timed-log-placement-inconsistent",
            ),
            (
                "detached-wall",
                lambda payload: payload.__setitem__("wall_elapsed_seconds", 0.5),
                "timed-wall-elapsed-inconsistent",
            ),
            (
                "negative-duration",
                lambda payload: payload.__setitem__("duration_seconds", -1.0),
                "schema-minimum",
            ),
            (
                "nonpositive-timeout",
                lambda payload: payload.__setitem__("timeout_seconds", 0.0),
                "timed-timing-value-invalid",
            ),
            (
                "wall-minus",
                lambda payload: payload.__setitem__(
                    "wall_minus_monotonic_seconds", 0.25
                ),
                "timed-wall-minus-monotonic-inconsistent",
            ),
            (
                "discontinuity",
                lambda payload: payload.__setitem__(
                    "wall_clock_discontinuity", True
                ),
                "timed-wall-clock-discontinuity-inconsistent",
            ),
            (
                "terminal",
                lambda payload: payload.__setitem__("status", "failed"),
                "timed-terminal-status-inconsistent",
            ),
            (
                "timeout-exit",
                lambda payload: payload.__setitem__("timed_out", True),
                "timed-timeout-exit-code-inconsistent",
            ),
        )
        for name, mutate, expected in mutations:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                repo = FixtureRepo(Path(temporary))
                built = build_bootstrap(repo)
                self.amend_timed_result(repo, built, mutate)
                _, result = self.evaluate_built_v2(repo, built)
                self.assertEqual(result["integration_decision"], "BLOCK")
                self.assertIn(
                    expected, {finding["code"] for finding in result["findings"]}
                )
                self.assertNotIn("focused", result.get("timed_validations", {}))

        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            self.amend_timed_result(
                repo,
                first,
                lambda payload: payload.__setitem__(
                    "wall_minus_monotonic_seconds", 0.25
                ),
            )
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            second = build_second(repo, first, first_payload, first_result)
            _, second_result = self.evaluate_built_v2(repo, second, gate=gate)
            self.assertEqual(second_result["integration_decision"], "BLOCK")
            self.assertIn(
                "timed-wall-minus-monotonic-inconsistent",
                {finding["code"] for finding in second_result["findings"]},
            )

    def test_v2_nonfinite_timed_numbers_fail_current_and_retained_decode(self) -> None:
        for token in ("NaN", "Infinity", "-Infinity", "1e999", "-1e999"):
            with self.subTest(token=token), tempfile.TemporaryDirectory() as temporary:
                repo = FixtureRepo(Path(temporary))
                built = build_bootstrap(repo)
                self.amend_timed_result_raw_number(repo, built, token)
                _, result = self.evaluate_built_v2(repo, built)
                self.assertEqual(result["integration_decision"], "BLOCK")
                self.assertIn(
                    "non-finite-json-number",
                    {finding["code"] for finding in result["findings"]},
                )
                self.assertEqual(result.get("timed_validations"), {})

        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            self.amend_timed_result_raw_number(repo, first, "1e999")
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            second = build_second(repo, first, first_payload, first_result)
            _, second_result = self.evaluate_built_v2(repo, second, gate=gate)
            self.assertEqual(second_result["integration_decision"], "BLOCK")
            self.assertIn(
                "non-finite-json-number",
                {finding["code"] for finding in second_result["findings"]},
            )

    def test_v2_timed_schema_failure_stops_typed_current_and_retained_use(self) -> None:
        for section in ("source", "worktree", "log", "result"):
            for invalid in (None, 7):
                with self.subTest(section=section, invalid=invalid), tempfile.TemporaryDirectory() as temporary:
                    repo = FixtureRepo(Path(temporary))
                    built = build_bootstrap(repo)
                    self.amend_timed_result(
                        repo,
                        built,
                        lambda payload, section=section, invalid=invalid: payload.__setitem__(
                            section, invalid
                        ),
                    )
                    _, result = self.evaluate_built_v2(repo, built)
                    self.assertEqual(result["integration_decision"], "BLOCK")
                    self.assertIn(
                        "schema-type",
                        {finding["code"] for finding in result["findings"]},
                    )
                    self.assertEqual(result.get("timed_validations"), {})

        for invalid in (None, 7):
            with self.subTest(argv=invalid), tempfile.TemporaryDirectory() as temporary:
                repo = FixtureRepo(Path(temporary))
                first = build_bootstrap(repo)
                self.amend_timed_result(
                    repo,
                    first,
                    lambda payload, invalid=invalid: payload.__setitem__(
                        "argv", invalid
                    ),
                )
                gate = clean_gate(repo, first.commits["predecessor"])
                first_payload, first_result = self.evaluate_built_v2(
                    repo, first, gate=gate
                )
                self.assertEqual(first_result.get("timed_validations"), {})
                second = build_second(repo, first, first_payload, first_result)
                _, second_result = self.evaluate_built_v2(repo, second, gate=gate)
                self.assertEqual(second_result["integration_decision"], "BLOCK")
                self.assertIn(
                    "schema-type",
                    {finding["code"] for finding in second_result["findings"]},
                )

    def test_v2_timed_tolerance_boundaries_and_signed_wall_jumps(self) -> None:
        base = timed_result(
            argv=["true"],
            revision="a" * 40,
            tree="b" * 40,
            log_bytes=b"ok\n",
        )
        for micros, should_pass in ((1, True), (2, True), (3, False)):
            with self.subTest(timestamp_microseconds=micros):
                payload = copy.deepcopy(base)
                payload["ended_at"] = (
                    f"2026-07-31T00:00:01.{micros:06d}Z"
                )
                _, findings = v2_evidence._timed_semantics(payload, "timed")
                codes = {finding.code for finding in findings}
                self.assertEqual(
                    "timed-wall-elapsed-inconsistent" not in codes,
                    should_pass,
                )

        for delta, should_pass in (
            (0.0000000000009, True),
            (0.0000000000010, True),
            (0.0000000000011, False),
        ):
            with self.subTest(wall_minus_delta=delta):
                payload = copy.deepcopy(base)
                payload["wall_minus_monotonic_seconds"] = delta
                _, findings = v2_evidence._timed_semantics(payload, "timed")
                codes = {finding.code for finding in findings}
                self.assertEqual(
                    "timed-wall-minus-monotonic-inconsistent" not in codes,
                    should_pass,
                )

        for started, ended, wall_elapsed, wall_minus in (
            (
                "2026-07-31T00:00:00.000000Z",
                "2026-07-31T00:00:03.000000Z",
                3.0,
                2.0,
            ),
            (
                "2026-07-31T00:00:01.000000Z",
                "2026-07-31T00:00:00.000000Z",
                -1.0,
                -2.0,
            ),
        ):
            payload = copy.deepcopy(base)
            payload.update(
                started_at=started,
                ended_at=ended,
                wall_elapsed_seconds=wall_elapsed,
                wall_minus_monotonic_seconds=wall_minus,
                drift_threshold_seconds=1.0,
                wall_clock_discontinuity=True,
            )
            _, findings = v2_evidence._timed_semantics(payload, "timed")
            self.assertEqual(findings, [])

    def test_v2_retained_result_is_closed_and_replays_all_stable_authority(self) -> None:
        cases = (
            (
                "input-schema",
                lambda result: result.__setitem__("input_schema", "forged.schema"),
                "schema-const",
            ),
            (
                "git-verified",
                lambda result: result.__setitem__("git_verified", False),
                "retained-result-replay-mismatch",
            ),
            (
                "timed-fact",
                lambda result: result["timed_validations"]["focused"].__setitem__(
                    "status", "failed"
                ),
                "retained-result-replay-mismatch",
            ),
            (
                "authority-root",
                lambda result: result["authority_chain"].__setitem__(
                    "root_arm", "kgi_kpi"
                ),
                "retained-result-authority-chain-mismatch",
            ),
            (
                "limits",
                lambda result: result["limits"].__setitem__(0, "forged limit"),
                "retained-result-replay-mismatch",
            ),
            (
                "unknown-top-level",
                lambda result: result.__setitem__("authority_override", True),
                "schema-additional-property",
            ),
            (
                "unknown-deciding-gate",
                lambda result: result["deciding_gate"].__setitem__(
                    "authority_override", True
                ),
                "schema-additional-property",
            ),
        )
        for name, mutate, expected in cases:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as temporary:
                repo = FixtureRepo(Path(temporary))
                first = build_bootstrap(repo)
                gate = clean_gate(repo, first.commits["predecessor"])
                first_payload, first_result = self.evaluate_built_v2(
                    repo, first, gate=gate
                )
                retained = copy.deepcopy(first_result)
                mutate(retained)
                retention, result_payload = retain_generation(
                    repo, first, first_payload, retained
                )
                second = build_second(
                    repo,
                    first,
                    first_payload,
                    retained,
                    retention=retention,
                    result_payload=result_payload,
                )
                _, result = self.evaluate_built_v2(repo, second, gate=gate)
                self.assertEqual(result["integration_decision"], "BLOCK")
                self.assertIn(
                    expected, {finding["code"] for finding in result["findings"]}
                )

        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            relocated = copy.deepcopy(first_result)
            relocated["deciding_gate"]["source_root"] = "/relocated/validator"
            retention, result_payload = retain_generation(
                repo, first, first_payload, relocated
            )
            second = build_second(
                repo,
                first,
                first_payload,
                relocated,
                retention=retention,
                result_payload=result_payload,
            )
            _, result = self.evaluate_built_v2(repo, second, gate=gate)
            self.assertEqual(result["integration_decision"], "ALLOW")

    def test_v2_manifest_origins_bind_bootstrap_and_predecessor_authority(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            record, parse_findings = parse_record(first.record, "record")
            self.assertEqual(parse_findings, [])
            self.assertIsNotNone(record)
            manifest = json.loads(
                subprocess.run(
                    [
                        "git",
                        "show",
                        f"{first.commits['packet']}:{first.paths['manifest']}",
                    ],
                    cwd=repo.root,
                    check=True,
                    capture_output=True,
                    text=True,
                ).stdout
            )
            manifest["prior_obligations"][0]["origin_generation"] = 1
            blob_repo = mock.Mock()
            blob_repo.blob.return_value = (
                json.dumps(manifest, sort_keys=True).encode("utf-8")
            )
            loaded, findings = load_manifest(
                blob_repo,
                record,
                record.artifact_index(),
                v2_authority.ChainResult(
                    rooted=True,
                    root_arm="cargo",
                    root_commit=first.commits["legacy_root"],
                    depth=1,
                ),
                self.v2_manifest_schema,
            )
            self.assertIsNotNone(loaded)
            self.assertIn(
                "manifest-record-origin-generation-mismatch",
                {finding.code for finding in findings},
            )
            self.assertIn(
                "bootstrap-manifest-origin-map-mismatch",
                {finding.code for finding in findings},
            )

        with tempfile.TemporaryDirectory() as temporary:
            repo = FixtureRepo(Path(temporary))
            first = build_bootstrap(repo)
            gate = clean_gate(repo, first.commits["predecessor"])
            first_payload, first_result = self.evaluate_built_v2(
                repo, first, gate=gate
            )
            second = build_second(repo, first, first_payload, first_result)
            second.record["prior_obligations"][0]["origin_generation"] = 1
            record, parse_findings = parse_record(second.record, "record")
            self.assertEqual(parse_findings, [])
            self.assertIsNotNone(record)
            chain, chain_findings = v2_authority.resolve_predecessor_authority_chain(
                GitRepo(repo.root),
                record,
                schema=self.v2_schema,
                result_schema=self.v2_result_schema,
                deciding_gate=gate,
                validator_repo=GitRepo(repo.root),
                legacy_roots=second.roots,
            )
            self.assertEqual(chain_findings, [])
            manifest = json.loads(
                subprocess.run(
                    [
                        "git",
                        "show",
                        f"{second.commits['packet']}:{second.paths['manifest']}",
                    ],
                    cwd=repo.root,
                    check=True,
                    capture_output=True,
                    text=True,
                ).stdout
            )
            manifest["prior_obligations"][1]["origin_generation"] = 1
            blob_repo = mock.Mock()
            blob_repo.blob.return_value = (
                json.dumps(manifest, sort_keys=True).encode("utf-8")
            )
            loaded, findings = load_manifest(
                blob_repo,
                record,
                record.artifact_index(),
                chain,
                self.v2_manifest_schema,
            )
            self.assertIsNotNone(loaded)
            self.assertIn(
                "manifest-predecessor-origin-generation-mismatch",
                {finding.code for finding in findings},
            )


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


class DevSwarmManifestSafetyTests(unittest.TestCase):
    def test_v1_rejects_inert_stage5_fields(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            batch_dir = root / "docs/development/ai/swarm-batches"
            batch_dir.mkdir(parents=True)
            manifest = {
                "schema": "dev-swarm.v1",
                "batch_id": "test",
                "tasks": [{"id": "task", "resource_class": "heavy"}],
                "scheduling": {"ready_order": "critical_path"},
            }
            (batch_dir / "test.json").write_text(
                json.dumps(manifest), encoding="utf-8"
            )
            findings = []
            check_dev_swarm_v1_stage5_fields(root, findings)
            locations = {finding.path for finding in findings}
            self.assertIn(
                "docs/development/ai/swarm-batches/test.json:scheduling",
                locations,
            )
            self.assertIn(
                "docs/development/ai/swarm-batches/test.json:tasks[0].resource_class",
                locations,
            )

    def test_safe_v1_and_future_v2_are_not_flagged(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            batch_dir = root / "docs/development/ai/swarm-batches"
            batch_dir.mkdir(parents=True)
            safe_v1 = {
                "schema": "dev-swarm.v1",
                "batch_id": "safe",
                "tasks": [{"id": "task", "provider": "claude"}],
            }
            future_v2 = {
                "schema": "dev-swarm.v2",
                "batch_id": "future",
                "scheduling": {"ready_order": "critical_path"},
                "tasks": [{"id": "task", "resource_class": "heavy"}],
            }
            (batch_dir / "safe.json").write_text(
                json.dumps(safe_v1), encoding="utf-8"
            )
            (batch_dir / "future.json").write_text(
                json.dumps(future_v2), encoding="utf-8"
            )
            findings = []
            check_dev_swarm_v1_stage5_fields(root, findings)
            self.assertEqual(findings, [])


class RustPacketProfileTests(unittest.TestCase):
    def test_active_cargo_packet_requires_exact_shared_profile(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            batch_dir = root / "docs/development/ai/swarm-batches"
            batch_dir.mkdir(parents=True)
            packet = batch_dir / "cfr168-rust-example-rereview.md"
            packet.write_text(
                "Status: ready for review\n\n## Validation\n\nRun `cargo test`.\n",
                encoding="utf-8",
            )
            findings = []
            check_active_rust_packet_profiles(root, findings)
            self.assertEqual(
                [finding.code for finding in findings],
                ["active-rust-packet-missing-shared-profile"],
            )

            packet.write_text(
                "Status: ready for review\n\n## Validation\n\n"
                + "\n".join(ACTIVE_RUST_PROFILE_MARKERS)
                + "\n\nRun `cargo test`.\n",
                encoding="utf-8",
            )
            findings = []
            check_active_rust_packet_profiles(root, findings)
            self.assertEqual(findings, [])


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
