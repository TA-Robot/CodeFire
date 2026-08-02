from __future__ import annotations

import copy
import concurrent.futures
import ast
import errno
import fcntl
import hashlib
import io
import json
import os
import re
import signal
import shutil
import stat
import subprocess
import sys
import tempfile
import time
import types
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
    sha256_json,
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
    LIVE_SOURCE_EVIDENCE_SCHEMA,
    SOURCE_KEY_TARGET_DIRECTORY,
    audit_recorded_command,
    atomic_write_json as atomic_write_rust_profile_json,
    build_result as build_rust_profile_result,
    evaluate_process,
    read_process_observations,
    target_dir_is_admitted,
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
    main as run_review_recurrence_gate,
    validate_recurrence_control,
    validator_owned_path,
)
from build_source_bound_rust import (  # noqa: E402
    BINARY_PATH,
    BUILD_COMMAND,
    SOURCE_GATE_PYTHON,
    acquire_gate_evidence_lock,
    bound_binary_identity,
    build_source_manifest,
    prepare_gate_evidence_slot,
    provenance_destination_contract,
    run_bounded_source_gate,
    source_gate_launcher_environment,
    source_keyed_build_argv,
    release_gate_evidence_lock,
    validate_provenance_payload,
    validate_source_inputs_against_head,
    write_provenance,
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
import recurrence_v2_history as v2_history  # noqa: E402
from recurrence_v2_evidence import (  # noqa: E402
    TimedFacts,
    resolve_artifacts,
    verify_canonical_port_packet,
    verify_retained_live_arm_anchor,
)
from recurrence_v2_git import PATCH_FLAGS, GitRepo  # noqa: E402
from recurrence_v2_fixture import (  # noqa: E402
    VARIANTS as RECURRENCE_V2_VARIANTS,
    FixtureRepo,
    build_bootstrap,
    build_second,
    clean_gate,
    retain_generation,
)
from recurrence_v2_model import (  # noqa: E402
    LIVE_ARM_ANCHOR_PATH,
    LIVE_ARM_ANCHOR_SCHEMA,
    PORT_PACKETS,
    PORT_REQUIRED_REPAIR_PATHS,
    ArtifactRef,
    Authority,
    parse_record,
)
from run_timed_gate import (  # noqa: E402
    atomic_write_json,
    main as run_timed_gate_main,
    open_exclusive_output,
    protected_rust_dispatch,
    run_gate,
    run_gate_v2,
    timing_evidence,
    verify_output_identity,
)
from run_source_keyed_cargo_gate import (  # noqa: E402
    GateBlocked,
    MAX_EXPORT_SOURCE_ARTIFACT_LINKS,
    MAX_RETAINED_TEST_HARNESSES,
    MAX_TEST_HARNESS_SUMMARIES,
    MARKER_NAME,
    REGISTRY_LOCK_NAME,
    RUN_STATE_NAME,
    UNSHARE_ISOLATION_FLAGS,
    build_artifact_identity,
    build_source_identity,
    capture_source_state,
    cargo_test_contract,
    CargoTestLogAnalyzer,
    cleanup_ephemeral_namespace,
    cleanup_namespace_locked,
    cleanup_snapshot,
    create_clean_git_snapshot,
    evidence_path,
    exclusive_lock,
    export_bound_artifact,
    git_bytes,
    git_tree_entries,
    isolation_identity,
    main as source_keyed_cargo_main,
    namespace_usage,
    normalize_cargo_argv,
    pinned_tool_files,
    prepare_namespace,
    probe_toolchain,
    read_run_state,
    rendered_result,
    run_source_keyed_gate,
    toml_has_disabled_harness,
    write_run_state,
    write_retention_record,
)
from readonly_source_exec import (  # noqa: E402
    descriptors_retained_by_environment,
    mount_is_read_only,
    mount_points,
    mount_propagation_tags,
    prove_private_propagation,
)
from run_dogfood_smoke import smoke_commands, validate_result  # noqa: E402
from suggest_validation import parse_porcelain_paths, suggestions  # noqa: E402
from validate_ai_runtime import strict_config_probe  # noqa: E402
import run_source_keyed_cargo_gate as gate_module  # noqa: E402


ROOT = Path(__file__).resolve().parents[3]


class StdoutBytesCapture:
    """A stdout stand-in that accepts raw bytes and refuses text.

    `write` raising is the point of the fixture. The release wrapper
    compares the gate's captured stdout with the published result file
    byte for byte before it reads the decision, so the gate has to emit the
    durable bytes through `sys.stdout.buffer`. Any `print` of a second
    rendering fails here instead of silently restoring that mismatch.
    """

    def __init__(self) -> None:
        self.buffer = io.BytesIO()

    def flush(self) -> None:
        pass

    def write(self, text: str) -> int:
        raise AssertionError(
            f"the gate wrote a text representation to stdout: {text!r}"
        )


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
    def test_release_build_delegates_to_source_keyed_gate(self) -> None:
        argv = source_keyed_build_argv(
            Path("/candidate"),
            Path("/evidence/build.log"),
            Path("/evidence/build.json"),
            timeout=1200,
            lock_timeout=3600,
        )
        self.assertEqual(argv[0], str(SOURCE_GATE_PYTHON))
        self.assertEqual(
            argv[1:6],
            [
                "-I",
                "-S",
                "-B",
                "-X",
                "pycache_prefix=/dev/null",
            ],
        )
        self.assertTrue(
            argv[6].endswith("run_source_keyed_cargo_gate.py")
        )
        separator = argv.index("--")
        self.assertEqual(
            argv[separator + 1 :],
            [
                "cargo",
                "build",
                "--release",
                "--locked",
                "-p",
                "codefire-cli",
                "--bin",
                "codefire-rs",
            ],
        )
        self.assertIn("--export-artifact", argv[:separator])

    def test_provenance_destination_is_ignored_untracked_target_only(
        self,
    ) -> None:
        contract = provenance_destination_contract(
            ROOT, Path("target/codefire-ai/provenance-test.json")
        )
        self.assertEqual(
            contract["decision"],
            "UNTRACKED_IGNORED_REPOSITORY_TARGET",
        )
        with self.assertRaises(ValueError):
            provenance_destination_contract(ROOT, Path("Cargo.toml"))

    def test_gate_evidence_slots_preserve_active_and_reclaim_inactive(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            workspace = Path(temporary_raw).resolve()
            root = workspace / "candidate"
            root.mkdir()
            evidence = workspace / "evidence"
            evidence.mkdir(mode=0o700)
            output = Path(
                "target/codefire-ai/rust-build-provenance.json"
            )
            lock, _wait = acquire_gate_evidence_lock(evidence, 1)
            try:
                first = prepare_gate_evidence_slot(
                    root,
                    output,
                    evidence,
                    lock_descriptor=lock,
                )
            finally:
                release_gate_evidence_lock(lock)
            self.assertEqual(first["selected_slot"], 0)
            first_log = Path(first["log"])
            first_result = Path(first["result"])
            for path in (first_log, first_result):
                path.write_bytes(b"active\n")
                path.chmod(0o400)
            write_provenance(
                root,
                output,
                {
                    "schema": "codefire.rust_build_provenance.v1",
                    "ok": True,
                    "provenance_destination_contract": {
                        "decision": (
                            "UNTRACKED_IGNORED_REPOSITORY_TARGET"
                        ),
                        "relative_path": output.as_posix(),
                        "tracked": False,
                        "ignored": True,
                    },
                    "source_keyed_gate_evidence_root": str(evidence),
                    "build_log_path": str(first_log),
                    "source_keyed_gate_result_path": str(first_result),
                    "gate_evidence_retention": {
                        "schema": (
                            "codefire.rust_gate_evidence_slot.v1"
                        ),
                        "scope": first["scope"],
                        "selected_slot": first["selected_slot"],
                        "aggregate_budget_bytes": 8 * 1024**3,
                        "entry_budget": 10_000,
                        "attempt_reserve_bytes": 258 * 1024**2,
                    },
                },
            )
            stale_log = Path(
                str(first_log).replace("-0.log", "-1.log")
            )
            stale_result = Path(
                str(first_result).replace("-0.json", "-1.json")
            )
            for path in (stale_log, stale_result):
                path.write_bytes(b"stale\n")
                path.chmod(0o400)

            lock, _wait = acquire_gate_evidence_lock(evidence, 1)
            try:
                second = prepare_gate_evidence_slot(
                    root,
                    output,
                    evidence,
                    lock_descriptor=lock,
                )
            finally:
                release_gate_evidence_lock(lock)

            self.assertEqual(second["active_slot_before"], 0)
            self.assertEqual(second["selected_slot"], 1)
            self.assertTrue(first_log.exists())
            self.assertTrue(first_result.exists())
            self.assertFalse(stale_log.exists())
            self.assertFalse(stale_result.exists())
            self.assertEqual(
                set(second["removed_inactive_paths"]),
                {str(stale_log), str(stale_result)},
            )

    def test_gate_evidence_lock_is_finite_and_prior_corruption_preserves_slots(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            workspace = Path(temporary_raw).resolve()
            root = workspace / "candidate"
            root.mkdir()
            evidence = workspace / "evidence"
            evidence.mkdir(mode=0o700)
            output = Path(
                "target/codefire-ai/rust-build-provenance.json"
            )
            lock, _wait = acquire_gate_evidence_lock(evidence, 1)
            try:
                with self.assertRaises(TimeoutError):
                    acquire_gate_evidence_lock(evidence, 0)
                slot = prepare_gate_evidence_slot(
                    root,
                    output,
                    evidence,
                    lock_descriptor=lock,
                )
                slot_log = Path(slot["log"])
                slot_result = Path(slot["result"])
                slot_log.write_bytes(b"preserve-log\n")
                slot_result.write_bytes(b"preserve-result\n")
                for path in (slot_log, slot_result):
                    path.chmod(0o400)
                provenance = root / output
                provenance.parent.mkdir(parents=True)
                provenance.write_bytes(b"not-json\n")
                provenance.chmod(0o600)
                with self.assertRaises(json.JSONDecodeError):
                    prepare_gate_evidence_slot(
                        root,
                        output,
                        evidence,
                        lock_descriptor=lock,
                    )
                self.assertTrue(slot_log.exists())
                self.assertTrue(slot_result.exists())

                other_log = Path(
                    str(slot_log).replace("-0.log", "-1.log")
                )
                other_result = Path(
                    str(slot_result).replace("-0.json", "-1.json")
                )
                for path in (other_log, other_result):
                    path.write_bytes(b"also-preserve\n")
                    path.chmod(0o400)
                semantic_base = {
                    "schema": "codefire.rust_build_provenance.v1",
                    "ok": True,
                    "provenance_destination_contract": {
                        "decision": (
                            "UNTRACKED_IGNORED_REPOSITORY_TARGET"
                        ),
                        "relative_path": output.as_posix(),
                        "tracked": False,
                        "ignored": True,
                    },
                    "source_keyed_gate_evidence_root": str(evidence),
                    "build_log_path": str(slot_log),
                    "source_keyed_gate_result_path": str(slot_result),
                    "gate_evidence_retention": {
                        "schema": (
                            "codefire.rust_gate_evidence_slot.v1"
                        ),
                        "scope": slot["scope"],
                        "selected_slot": slot["selected_slot"],
                        "aggregate_budget_bytes": 8 * 1024**3,
                        "entry_budget": 10_000,
                        "attempt_reserve_bytes": 258 * 1024**2,
                    },
                }
                semantic_corruptions = []
                wrong_schema = dict(semantic_base)
                wrong_schema["schema"] = "wrong"
                semantic_corruptions.append(wrong_schema)
                mixed_slots = copy.deepcopy(semantic_base)
                mixed_slots["source_keyed_gate_result_path"] = str(
                    other_result
                )
                semantic_corruptions.append(mixed_slots)
                for payload in semantic_corruptions:
                    write_provenance(root, output, payload)
                    with self.assertRaisesRegex(
                        ValueError,
                        "no evidence was removed",
                    ):
                        prepare_gate_evidence_slot(
                            root,
                            output,
                            evidence,
                            lock_descriptor=lock,
                        )
                    for path in (
                        slot_log,
                        slot_result,
                        other_log,
                        other_result,
                    ):
                        self.assertTrue(path.exists())
            finally:
                release_gate_evidence_lock(lock)

    def test_source_gate_launcher_scrubs_pre_interpreter_overrides(
        self,
    ) -> None:
        environment = {
            "PATH": "/trusted/bin",
            "LD_PRELOAD": "/tmp/evil.so",
            "LD_AUDIT": "/tmp/audit.so",
            "GLIBC_TUNABLES": "glibc.malloc.check=3",
            "GCONV_PATH": "/tmp/gconv",
            "MALLOC_CHECK_": "3",
            "PYTHONHOME": "/tmp/python",
            "RUSTC_WRAPPER": "/tmp/wrapper",
            "RUSTUP_HOME": "/trusted/rustup",
        }
        launched = source_gate_launcher_environment(environment)
        self.assertEqual(launched["PATH"], "/usr/bin:/bin")
        self.assertEqual(launched["HOME"], "/nonexistent")
        self.assertEqual(launched["RUSTUP_HOME"], "/trusted/rustup")
        for name in (
            "LD_PRELOAD",
            "LD_AUDIT",
            "GLIBC_TUNABLES",
            "GCONV_PATH",
            "MALLOC_CHECK_",
            "PYTHONHOME",
            "RUSTC_WRAPPER",
        ):
            self.assertNotIn(name, launched)
        self.assertEqual(launched["PYTHONNOUSERSITE"], "1")

    def test_source_gate_timeout_kills_detached_descendants(self) -> None:
        probe = subprocess.run(
            [
                "/usr/bin/unshare",
                "-Urpf",
                "--mount-proc",
                "--kill-child=SIGKILL",
                "--",
                "/bin/true",
            ],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=5,
            check=False,
        )
        if probe.returncode != 0:
            self.skipTest("unprivileged PID namespaces unavailable")
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw).resolve()
            survivor = root / "detached-survivor"
            program = (
                "import subprocess,time,sys;"
                "subprocess.Popen("
                "[sys.executable,'-c',"
                f"\"import time;time.sleep(1);"
                f"open({str(survivor)!r},'w').write('alive')\"],"
                "start_new_session=True);"
                "time.sleep(10)"
            )
            with self.assertRaises(subprocess.TimeoutExpired):
                run_bounded_source_gate(
                    [str(SOURCE_GATE_PYTHON), "-c", program],
                    root=root,
                    environment=source_gate_launcher_environment({}),
                    timeout=0.25,
                )
            deadline = time.monotonic() + 1.5
            while time.monotonic() < deadline and not survivor.exists():
                time.sleep(0.05)
            self.assertFalse(survivor.exists())

    def test_parent_death_signal_kills_namespace_descendants(self) -> None:
        probe = subprocess.run(
            [
                "/usr/bin/unshare",
                "-Urpf",
                "--mount-proc",
                "--kill-child=SIGKILL",
                "--",
                "/usr/bin/true",
            ],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        if probe.returncode != 0:
            self.skipTest("unprivileged PID namespaces unavailable")
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw).resolve()
            ready = root / "ready"
            survivor = root / "survivor"
            child = (
                "import time;from pathlib import Path;"
                f"Path({str(ready)!r}).write_text('ready');"
                "time.sleep(1);"
                f"Path({str(survivor)!r}).write_text('alive')"
            )
            manager_program = (
                "import sys;"
                f"sys.path.insert(0,{str(TOOLS_DIR)!r});"
                "from pathlib import Path;"
                "from build_source_bound_rust import "
                "run_bounded_source_gate,source_gate_launcher_environment;"
                f"run_bounded_source_gate("
                f"{[str(SOURCE_GATE_PYTHON), '-c', child]!r},"
                f"root=Path({str(root)!r}),"
                "environment=source_gate_launcher_environment({}),"
                "timeout=30)"
            )
            manager = subprocess.Popen(
                [str(SOURCE_GATE_PYTHON), "-c", manager_program],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            try:
                deadline = time.monotonic() + 5
                while (
                    time.monotonic() < deadline
                    and not ready.exists()
                ):
                    time.sleep(0.02)
                self.assertTrue(ready.exists())
                os.kill(manager.pid, signal.SIGKILL)
                manager.wait(timeout=5)
                time.sleep(1.25)
                self.assertFalse(survivor.exists())
            finally:
                if manager.poll() is None:
                    manager.kill()
                    manager.wait()

    def test_provenance_publish_does_not_follow_legacy_temp_symlink(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw).resolve()
            parent = root / "target/codefire-ai"
            parent.mkdir(parents=True)
            destination = parent / "rust-build-provenance.json"
            victim = root / "victim"
            victim.write_bytes(b"must-remain")
            legacy_temporary = parent / (
                f".{destination.name}.tmp"
            )
            legacy_temporary.symlink_to(victim)
            payload = {"schema": "test", "writer": 1}
            published, digest = write_provenance(
                root,
                destination.relative_to(root),
                payload,
            )
            expected = (
                json.dumps(
                    payload,
                    ensure_ascii=False,
                    sort_keys=True,
                    separators=(",", ":"),
                )
                + "\n"
            ).encode()
            self.assertEqual(published, destination)
            self.assertEqual(destination.read_bytes(), expected)
            self.assertEqual(victim.read_bytes(), b"must-remain")
            self.assertEqual(digest, hashlib.sha256(expected).hexdigest())
            self.assertTrue(legacy_temporary.is_symlink())

    def test_concurrent_provenance_writers_publish_complete_record(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw).resolve()
            output = Path(
                "target/codefire-ai/rust-build-provenance.json"
            )
            payloads = [
                {"schema": "test", "writer": writer}
                for writer in range(2)
            ]
            with concurrent.futures.ThreadPoolExecutor(
                max_workers=2
            ) as executor:
                results = list(
                    executor.map(
                        lambda payload: write_provenance(
                            root, output, payload
                        ),
                        payloads,
                    )
                )
            rendered = {
                (
                    json.dumps(
                        payload,
                        ensure_ascii=False,
                        sort_keys=True,
                        separators=(",", ":"),
                    )
                    + "\n"
                ).encode()
                for payload in payloads
            }
            destination = root / output
            self.assertIn(destination.read_bytes(), rendered)
            self.assertEqual(
                {published for published, _ in results},
                {destination},
            )
            self.assertFalse(
                any(
                    path.name.startswith(
                        f".{destination.name}.tmp-"
                    )
                    for path in destination.parent.iterdir()
                )
            )

    def test_provenance_publish_rolls_back_on_parent_fsync_failure(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw).resolve()
            output = Path(
                "target/codefire-ai/rust-build-provenance.json"
            )
            destination, _ = write_provenance(
                root, output, {"schema": "test", "writer": "old"}
            )
            previous = destination.read_bytes()
            real_fsync = os.fsync
            failed = False

            # Target the durability barrier by observable state rather than
            # by counting fsync calls: fail the first directory fsync taken
            # once the destination already holds the new bytes. That is
            # exactly the post-rename barrier whose failure must roll the
            # previous generation back.
            def fail_after_rename(descriptor):
                nonlocal failed
                if (
                    not failed
                    and stat.S_ISDIR(os.fstat(descriptor).st_mode)
                    and destination.read_bytes() != previous
                ):
                    failed = True
                    raise OSError("synthetic parent fsync failure")
                return real_fsync(descriptor)

            with mock.patch(
                "build_source_bound_rust.os.fsync",
                side_effect=fail_after_rename,
            ), self.assertRaises(OSError):
                write_provenance(
                    root,
                    output,
                    {"schema": "test", "writer": "new"},
                )
            self.assertEqual(destination.read_bytes(), previous)

    def test_provenance_publication_lock_has_finite_timeout(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw).resolve()
            parent = root / "target/codefire-ai"
            parent.mkdir(parents=True)
            output = Path(
                "target/codefire-ai/rust-build-provenance.json"
            )
            lock = parent / ".rust-build-provenance.json.lock"
            descriptor = os.open(
                lock, os.O_RDWR | os.O_CREAT | os.O_CLOEXEC, 0o600
            )
            try:
                fcntl.flock(descriptor, fcntl.LOCK_EX)
                with self.assertRaises(TimeoutError):
                    write_provenance(
                        root,
                        output,
                        {"schema": "test"},
                        lock_timeout_seconds=0.05,
                    )
            finally:
                fcntl.flock(descriptor, fcntl.LOCK_UN)
                os.close(descriptor)

    def test_binary_identity_rejects_symlinked_ancestor(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root = temporary / "candidate"
            outside = temporary / "outside"
            root.mkdir()
            (outside / "release").mkdir(parents=True)
            (outside / "release/codefire-rs").write_bytes(b"forged")
            (outside / "release/codefire-rs").chmod(0o700)
            (root / "target").symlink_to(outside)
            with self.assertRaises(OSError):
                bound_binary_identity(root, BINARY_PATH)

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
            workspace = Path(temporary).resolve()
            root = workspace / "candidate"
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
            binary.chmod(0o700)
            source_manifest = build_source_manifest(root)
            binary_identity = bound_binary_identity(root, BINARY_PATH)
            provenance_relative = (
                "target/codefire-ai/provenance.json"
            )
            evidence_scope = hashlib.sha256(
                (
                    str(root)
                    + "\0"
                    + provenance_relative
                ).encode("utf-8")
            ).hexdigest()[:24]
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
                "provenance_destination_contract": {
                    "relative_path": provenance_relative,
                },
            }
            handcrafted_errors = validate_provenance_payload(
                root, provenance, expected_revision=head
            )
            self.assertTrue(
                any(
                    "evidence chain is unavailable" in error
                    for error in handcrafted_errors
                )
            )

            evidence = workspace / "evidence"
            evidence.mkdir(mode=0o700)
            evidence_lock, _wait = acquire_gate_evidence_lock(
                evidence, 1
            )
            release_gate_evidence_lock(evidence_lock)
            log = evidence / (
                f"rust-release-build-{evidence_scope}-0.log"
            )
            log_raw = b"bounded gate log\n"
            log.write_bytes(log_raw)
            log.chmod(0o400)
            source_key = "a" * 64
            artifact_key = "b" * 64
            namespace = (
                workspace
                / "shared-target/source-keys"
                / artifact_key
            )
            candidate = {
                "candidate_root": str(root),
                "head_revision": head,
                "dirty": False,
            }
            gate_result = {
                "schema": "codefire.source_keyed_cargo_gate.v1",
                "decision": "PASS",
                "blockers": [],
                "source_key": source_key,
                "artifact_key": artifact_key,
                "candidate": candidate,
                "candidate_after": candidate,
                "candidate_unchanged": True,
                "execution_profile": {
                    "admitted_evidence_root": str(evidence),
                },
                "evidence_retention": {
                    "schema": (
                        "codefire.cargo_gate_evidence_retention.v1"
                    ),
                    "admission": {
                        "aggregate_budget_bytes": 8 * 1024**3,
                        "reserve_bytes": 258 * 1024**2,
                        "entry_reserve": 2,
                    },
                    "postflight": {
                        "allocated_bytes": (
                            len(log_raw)
                        ),
                        "entry_count": 3,
                        "aggregate_budget_bytes": 8 * 1024**3,
                        "reserve_bytes": 2 * 1024**2,
                        "entry_reserve": 2,
                        "free_floor_bytes": 1024**3,
                        "filesystem_free_after_reserve_bytes": (
                            2 * 1024**3
                        ),
                    },
                },
                "argv": BUILD_COMMAND,
                "target": {
                    "namespace": str(namespace),
                    "artifact_key": artifact_key,
                },
                "retention": {
                    "decision": "PERSISTED",
                    "run_state_committed_sha256": "c" * 64,
                },
                "gate": {
                    "status": "passed",
                    "exit_code": 0,
                    "timed_out": False,
                    "runtime_guard": {
                        "enabled": True,
                        "decision": "PASS",
                    },
                    "log": {
                        "path": str(log),
                        "bytes": len(log_raw),
                        "sha256": hashlib.sha256(log_raw).hexdigest(),
                        "complete": True,
                    },
                },
                "log_binding": {
                    "path": str(log),
                    "bytes": len(log_raw),
                    "sha256": hashlib.sha256(log_raw).hexdigest(),
                    "timed_gate_evidence_matches": True,
                    "source_key_present_in_prefix": True,
                },
                "artifact_export": {
                    "decision": "EXPORTED",
                    "artifact_relative_path": "release/codefire-rs",
                    "target_artifact_path": str(
                        namespace / "release/codefire-rs"
                    ),
                    "export_path": str(root / BINARY_PATH),
                    "bytes": binary_identity["bytes"],
                    "sha256": binary_identity["sha256"],
                    "export_device": binary_identity["device"],
                    "export_inode": binary_identity["inode"],
                    "export_mode": binary_identity["mode"],
                    "export_nlink": binary_identity["nlink"],
                    "destination_contract": {
                        "decision": (
                            "UNTRACKED_IGNORED_CANDIDATE_TARGET"
                        ),
                        "relative_path": BINARY_PATH.as_posix(),
                        "tracked": False,
                        "ignored": True,
                    },
                },
            }
            result_path = evidence / (
                f"rust-release-build-{evidence_scope}-0.json"
            )
            result_raw = (
                json.dumps(
                    gate_result,
                    sort_keys=True,
                    separators=(",", ":"),
                )
                + "\n"
            ).encode()
            result_path.write_bytes(result_raw)
            result_path.chmod(0o400)
            provenance.update(
                {
                    "source_keyed_gate_evidence_root": str(evidence),
                    "source_keyed_gate_result_path": str(result_path),
                    "source_keyed_gate_result_bytes": len(result_raw),
                    "source_keyed_gate_result_sha256": hashlib.sha256(
                        result_raw
                    ).hexdigest(),
                    "build_log_path": str(log),
                    "build_log_bytes": len(log_raw),
                    "build_log_sha256": hashlib.sha256(
                        log_raw
                    ).hexdigest(),
                    "source_key": source_key,
                    "artifact_key": artifact_key,
                    "source_keyed_binary_path": str(
                        namespace / "release/codefire-rs"
                    ),
                    "source_gate_launcher_environment_sha256": "d" * 64,
                    "source_gate_launcher_environment_names": [
                        "HOME",
                        "LANG",
                        "LC_ALL",
                        "PATH",
                    ],
                    "gate_evidence_retention": {
                        "schema": (
                            "codefire.rust_gate_evidence_slot.v1"
                        ),
                        "scope": evidence_scope,
                        "selected_slot": 0,
                        "aggregate_budget_bytes": 8 * 1024**3,
                        "entry_budget": 10_000,
                        "attempt_reserve_bytes": 258 * 1024**2,
                    },
                }
            )
            self.assertEqual(
                validate_provenance_payload(
                    root, provenance, expected_revision=head
                ),
                [],
            )
            log.unlink()
            self.assertTrue(
                any(
                    "evidence chain is unavailable" in error
                    for error in validate_provenance_payload(
                        root, provenance, expected_revision=head
                    )
                )
            )
            log.write_bytes(log_raw)
            log.chmod(0o400)
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

            def rotate_before_consumer_lock(
                evidence_root: Path,
                timeout_seconds: float,
                *,
                create: bool = True,
            ) -> tuple[int, float]:
                artifact.write_bytes(raw + b" ")
                return acquire_gate_evidence_lock(
                    evidence_root,
                    timeout_seconds,
                    create=create,
                )

            with mock.patch(
                "build_source_bound_rust.acquire_gate_evidence_lock",
                side_effect=rotate_before_consumer_lock,
            ):
                race_codes = {
                    finding.code
                    for finding in validate_review_against_task(
                        review,
                        task,
                        "review",
                        patch_bytes=b"",
                        actual_files=set(),
                        resolved_base_revision=head,
                        resolved_head_revision=head,
                        post_validation_capabilities={
                            "rust-source-bound-binary"
                        },
                        root=root,
                    )
                }
            self.assertIn(
                "review-rust-provenance-changed-before-lock",
                race_codes,
            )
            artifact.write_bytes(raw)

            copied_artifact = (
                root / "target/codefire-ai/copied-provenance.json"
            )
            copied_artifact.write_bytes(raw)
            copied_review = copy.deepcopy(review)
            copied_review["validation_evidence"][0][
                "capability_evidence_path"
            ] = "target/codefire-ai/copied-provenance.json"
            copied_codes = {
                finding.code
                for finding in validate_review_against_task(
                    copied_review,
                    task,
                    "review",
                    patch_bytes=b"",
                    actual_files=set(),
                    resolved_base_revision=head,
                    resolved_head_revision=head,
                    post_validation_capabilities={
                        "rust-source-bound-binary"
                    },
                    root=root,
                )
            }
            self.assertIn(
                "review-rust-provenance-not-canonical",
                copied_codes,
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
    def test_direct_and_known_indirect_rust_dispatch_are_rejected(self) -> None:
        cases = (
            (["/canonical/bin/cargo", "test"], "cargo"),
            (["/usr/bin/env", "cargo", "test"], "cargo"),
            (["env", "-a", "fake", "cargo", "test"], "cargo"),
            (["env", "--argv0", "fake", "cargo", "test"], "cargo"),
            (["env", "-S", "cargo test --locked"], "cargo"),
            (["env", "PINNED=1", "/canonical/bin/rustc", "--version"], "rustc"),
            (["/canonical/bin/rustdoc", "--version"], "rustdoc"),
            (["/canonical/bin/rustfmt", "--version"], "rustfmt"),
            (["/canonical/bin/cargo-clippy", "--version"], "cargo-clippy"),
            (["/canonical/bin/cargo-fmt", "--version"], "cargo-fmt"),
            (["/usr/bin/timeout", "30m", "cargo", "check"], "cargo"),
            (["timeout", "--", "1", "cargo", "test"], "cargo"),
            (
                ["timeout", "--signal", "TERM", "30", "clippy-driver"],
                "clippy-driver",
            ),
            (["rustup", "run", "stable", "cargo", "test"], "cargo"),
            (
                [
                    "rustup",
                    "--verbose",
                    "run",
                    "stable",
                    "cargo",
                    "test",
                ],
                "cargo",
            ),
            (
                [
                    "rustup",
                    "run",
                    "--install",
                    "stable",
                    "cargo",
                    "test",
                ],
                "cargo",
            ),
            (["nice", "-n", "5", "cargo", "check"], "cargo"),
            (["taskset", "-c", "0", "rustc", "--version"], "rustc"),
            (["taskset", "-c0", "cargo", "test"], "cargo"),
            (["setsid", "cargo", "test"], "cargo"),
            (["nohup", "cargo", "test"], "cargo"),
            (
                ["/bin/bash", "-lc", "cargo test --locked"],
                "unresolved-wrapper",
            ),
            (
                ["sh", "-c", "/canonical/bin/rustc --version"],
                "unresolved-wrapper",
            ),
            (
                ["python3", "-c", "import os; os.execvp('cargo',['cargo'])"],
                "unresolved-wrapper",
            ),
            (
                [
                    "python3",
                    "-c",
                    "import subprocess; subprocess.call(['cargo','test'])",
                ],
                "unresolved-wrapper",
            ),
            (
                ["perl", "-e", "system('rustc --version')"],
                "unresolved-wrapper",
            ),
            (
                ["bash", "-c", r"c\argo test"],
                "unresolved-wrapper",
            ),
            (
                ["bash", "-c", 'ca""rgo test'],
                "unresolved-wrapper",
            ),
            (
                ["dash", "-c", "printf safe"],
                "unresolved-wrapper",
            ),
            (
                ["ksh", "-c", "printf safe"],
                "unresolved-wrapper",
            ),
            (
                ["bash", "-O", "extglob", "-c", "printf safe"],
                "unresolved-wrapper",
            ),
            (
                ["perl", "-e", "print 'safe'"],
                "unresolved-wrapper",
            ),
            (
                ["perl5", "-we", "print 'safe'"],
                "unresolved-wrapper",
            ),
            (
                ["ruby", "-e", "puts 'safe'"],
                "unresolved-wrapper",
            ),
            (
                ["python3.12", "-c", "print('safe')"],
                "unresolved-wrapper",
            ),
            (
                ["python3", "-X", "dev", "-c", "print('safe')"],
                "unresolved-wrapper",
            ),
            (
                ["env", "-a", "fake", "dash", "-c", "printf safe"],
                "unresolved-wrapper",
            ),
            (
                ["timeout", "-s", "TERM", "1", "ksh", "-c", "printf safe"],
                "unresolved-wrapper",
            ),
            (
                ["bash", "-c", "x=cargo; $x test"],
                "unresolved-wrapper",
            ),
            (
                ["bash", "-c", "x=ca;y=rgo;$x$y test"],
                "unresolved-wrapper",
            ),
            (
                [
                    "python3",
                    "-c",
                    (
                        "import subprocess;"
                        "subprocess.run(['ca'+'rgo','test'])"
                    ),
                ],
                "unresolved-wrapper",
            ),
            (
                [
                    "env",
                    "CARGO=/canonical/bin/cargo",
                    "sh",
                    "-c",
                    "$CARGO test",
                ],
                "unresolved-wrapper",
            ),
        )
        for argv, expected in cases:
            with self.subTest(argv=argv):
                self.assertEqual(protected_rust_dispatch(argv), expected)

    def test_static_non_rust_argv_with_rust_words_remains_admitted(self) -> None:
        cases = (
            ["/usr/bin/env", "LABEL=cargo", "/usr/bin/printf", "rustc"],
            ["timeout", "30", "/usr/bin/printf", "clippy-driver"],
        )
        for argv in cases:
            with self.subTest(argv=argv):
                self.assertIsNone(protected_rust_dispatch(argv))

    def test_symlink_alias_to_protected_tool_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw)
            cargo = temporary / "cargo"
            cargo.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            cargo.chmod(0o700)
            alias = temporary / "validator"
            alias.symlink_to(cargo)
            self.assertEqual(
                protected_rust_dispatch([str(alias), "test"]), "cargo"
            )

    def test_non_finite_limits_and_bounded_output_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw).resolve()
            for index, timeout in enumerate((float("nan"), float("inf"))):
                with self.subTest(timeout=timeout):
                    with self.assertRaisesRegex(ValueError, "timeout"):
                        run_gate(
                            [sys.executable, "-c", "pass"],
                            root,
                            root / f"nonfinite-{index}.log",
                            timeout,
                        )
            result = run_gate(
                [
                    sys.executable,
                    "-c",
                    "import os; os.write(1, b'x' * 4096)",
                ],
                root,
                root / "bounded.log",
                5,
                max_log_bytes=128,
            )
            self.assertEqual(result["status"], "failed")
            self.assertTrue(result["log_limit_exceeded"])
            self.assertFalse(result["log"]["complete"])
            self.assertEqual(result["log"]["bytes"], 128)

    def test_runtime_resource_guard_terminates_the_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw).resolve()
            checks = 0

            def guard() -> None:
                nonlocal checks
                checks += 1
                if checks >= 2:
                    raise RuntimeError("synthetic resource limit")

            result = run_gate(
                [
                    sys.executable,
                    "-I",
                    "-S",
                    "-c",
                    "import time; time.sleep(30)",
                ],
                root,
                root / "resource-guard.log",
                10,
                runtime_guard=guard,
                runtime_guard_interval_seconds=0.01,
            )
            self.assertEqual(result["status"], "failed")
            self.assertEqual(result["exit_code"], 126)
            self.assertEqual(
                result["runtime_guard"]["decision"], "BLOCK"
            )
            self.assertEqual(
                result["runtime_guard"]["error"]["error_type"],
                "RuntimeError",
            )
            self.assertGreaterEqual(
                result["runtime_guard"]["invocation_count"], 2
            )

    def test_output_parent_replacement_is_detected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw).resolve()
            evidence = root / "evidence"
            evidence.mkdir()
            output = evidence / "gate.log"
            path, descriptor, parent_descriptor = open_exclusive_output(
                output
            )
            displaced = root / "evidence-displaced"
            try:
                evidence.rename(displaced)
                evidence.mkdir()
                with self.assertRaisesRegex(
                    ValueError, "output parent changed"
                ):
                    verify_output_identity(
                        path, descriptor, parent_descriptor
                    )
            finally:
                os.close(descriptor)
                os.close(parent_descriptor)

    def test_atomic_result_parent_replacement_is_not_silently_accepted(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw).resolve()
            evidence = root / "evidence"
            evidence.mkdir()
            result = evidence / "result.json"
            atomic_write_json(result, {"decision": "PASS"})
            self.assertEqual(
                json.loads(result.read_text(encoding="utf-8")),
                {"decision": "PASS"},
            )

    def test_atomic_result_is_removed_when_parent_fsync_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            evidence = Path(temporary_raw).resolve() / "evidence"
            evidence.mkdir()
            result = evidence / "result.json"
            real_fsync = os.fsync
            parent_failure_injected = False

            def fail_first_parent_fsync(descriptor: int) -> None:
                nonlocal parent_failure_injected
                if (
                    stat.S_ISDIR(os.fstat(descriptor).st_mode)
                    and not parent_failure_injected
                ):
                    parent_failure_injected = True
                    raise OSError("injected parent fsync failure")
                real_fsync(descriptor)

            with mock.patch(
                "run_timed_gate.os.fsync",
                side_effect=fail_first_parent_fsync,
            ):
                with self.assertRaisesRegex(
                    OSError, "injected parent fsync failure"
                ):
                    atomic_write_json(result, {"decision": "PASS"})

            self.assertTrue(parent_failure_injected)
            self.assertFalse(result.exists())
            self.assertEqual(list(evidence.iterdir()), [])

    def test_atomic_result_rollback_preserves_replacement_inode(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            evidence = Path(temporary_raw).resolve() / "evidence"
            evidence.mkdir()
            result = evidence / "result.json"
            replacement = b'{"decision":"FOREIGN"}\n'
            real_fsync = os.fsync
            parent_failure_injected = False

            def replace_result_then_fail(descriptor: int) -> None:
                nonlocal parent_failure_injected
                if (
                    stat.S_ISDIR(os.fstat(descriptor).st_mode)
                    and not parent_failure_injected
                ):
                    parent_failure_injected = True
                    result.unlink()
                    result.write_bytes(replacement)
                    raise OSError("injected parent fsync failure")
                real_fsync(descriptor)

            with mock.patch(
                "run_timed_gate.os.fsync",
                side_effect=replace_result_then_fail,
            ):
                with self.assertRaisesRegex(
                    OSError, "injected parent fsync failure"
                ):
                    atomic_write_json(result, {"decision": "PASS"})

            self.assertTrue(parent_failure_injected)
            self.assertEqual(result.read_bytes(), replacement)
            self.assertEqual(list(evidence.iterdir()), [result])

    def test_mountinfo_parser_preserves_nested_and_read_only_evidence(
        self,
    ) -> None:
        mountinfo = (
            "24 20 0:1 / /candidate rw - ext4 /dev/root rw\n"
            "25 24 0:2 / /candidate/nested ro - tmpfs tmpfs ro\n"
            "26 20 0:3 / /snapshot\\040root ro - tmpfs tmpfs ro\n"
        )
        self.assertEqual(
            mount_points(mountinfo),
            [
                Path("/candidate"),
                Path("/candidate/nested"),
                Path("/snapshot root"),
            ],
        )
        self.assertFalse(mount_is_read_only(Path("/candidate"), mountinfo))
        self.assertTrue(
            mount_is_read_only(Path("/candidate/nested"), mountinfo)
        )

    def test_shared_propagation_is_rejected_before_any_mount(self) -> None:
        # Counterexample: the private source tmpfs is mounted at the
        # candidate worktree's own path. Under shared propagation it would
        # escape into the host namespace and mask the developer's real
        # worktree with a read-only snapshot copy for the whole gate.
        shared = (
            "24 20 0:1 / / rw shared:1 - ext4 /dev/root rw\n"
            "25 24 0:2 / /candidate rw - ext4 /dev/root rw\n"
        )
        self.assertEqual(
            mount_propagation_tags(shared), [("/", "shared:1")]
        )
        with self.assertRaisesRegex(
            RuntimeError, "not privately propagated"
        ):
            prove_private_propagation(shared)

    def test_slave_propagation_is_rejected(self) -> None:
        # A slave mount does not leak our mounts outward, but a host
        # unmount propagates inward and can unmask a protected path
        # mid-gate. --propagation private clears both, so both block.
        slave = (
            "24 20 0:1 / / rw - ext4 /dev/root rw\n"
            "25 24 0:2 / /toolchain ro master:7 - ext4 /dev/root ro\n"
        )
        self.assertEqual(
            mount_propagation_tags(slave), [("/toolchain", "master:7")]
        )
        with self.assertRaises(RuntimeError):
            prove_private_propagation(slave)

    def test_propagation_parser_is_not_fooled_by_a_dash_mount_point(
        self,
    ) -> None:
        # Root and mount-point fields are not "-"-escaped. Scanning for the
        # separator from the start of the line would stop at a path named
        # "-", slice away the optional fields, and report a shared mount as
        # private -- the one direction this check may never fail in.
        mountinfo = "24 20 0:1 /- /- rw shared:3 - ext4 /dev/root rw\n"
        self.assertEqual(
            mount_propagation_tags(mountinfo), [("/-", "shared:3")]
        )
        with self.assertRaises(RuntimeError):
            prove_private_propagation(mountinfo)

    def test_unparseable_mountinfo_line_is_not_read_as_private(
        self,
    ) -> None:
        # Counterexample: a length guard that `continue`s instead of
        # raising makes a short malformed line read as "not shared", which
        # is the one direction this check may never fail in. A line
        # carrying a shared: tag but too short to parse must block, not be
        # skipped -- so every malformed shape takes the same reject path.
        for malformed in (
            "24 20 0:1 / / rw noseparator\n",
            "24 20 0:1 / / shared:1\n",
            "24 20 0:1 / /\n",
            "x\n",
        ):
            with self.subTest(malformed=malformed):
                with self.assertRaisesRegex(RuntimeError, "unparseable"):
                    mount_propagation_tags(malformed)
                with self.assertRaises(RuntimeError):
                    prove_private_propagation(malformed)

    def test_blank_mountinfo_lines_are_tolerated(self) -> None:
        # A trailing newline is not a malformed record.
        self.assertEqual(
            mount_propagation_tags(
                "24 20 0:1 / / rw - ext4 /dev/root rw\n\n"
            ),
            [],
        )

    def test_real_mountinfo_parses(self) -> None:
        # The strict reject path must not reject the kernel's own format.
        mount_propagation_tags(
            Path("/proc/self/mountinfo").read_text(encoding="utf-8")
        )

    def test_private_propagation_is_accepted(self) -> None:
        private = (
            "24 20 0:1 / / rw - ext4 /dev/root rw\n"
            "25 24 0:2 / /candidate ro - tmpfs tmpfs ro\n"
            "26 24 0:3 / /unbindable rw unbindable - tmpfs tmpfs rw\n"
        )
        self.assertEqual(mount_propagation_tags(private), [])
        prove_private_propagation(private)

    def test_isolation_tool_descriptors_are_not_retained_for_cargo(
        self,
    ) -> None:
        # Counterexample: leaving the pinned mount/setpriv/python/helper
        # descriptors open would hand the build inherited executables it
        # never needs, reachable as /proc/self/fd/<n>. Only descriptors the
        # child environment actually names may survive the exec.
        environment = {
            "RUSTC": "/proc/self/fd/11",
            "RUSTDOC": "/proc/self/fd/12",
            "CARGO_TARGET_DIR": "/workspace/target",
            "PATH": "/usr/bin",
            "MALFORMED": "/proc/self/fd/not-a-number",
        }
        retained = descriptors_retained_by_environment(environment, 9)
        self.assertEqual(retained, {9, 11, 12})
        for isolation_descriptor in (3, 4, 5, 6):
            self.assertNotIn(isolation_descriptor, retained)

    def test_readonly_helper_mounts_snapshot_and_drops_namespace_caps(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            namespace_probe = subprocess.run(
                [
                    "/usr/bin/unshare",
                    "-Urnmpf",
                    "--mount-proc",
                    "--kill-child=SIGKILL",
                    "--",
                    "/usr/bin/true",
                ],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            if (
                namespace_probe.returncode != 0
                and b"Operation not permitted"
                in namespace_probe.stderr
            ):
                self.skipTest(
                    "unprivileged mount namespaces unavailable"
                )
            self.assertEqual(
                namespace_probe.returncode,
                0,
                namespace_probe.stderr.decode(
                    "utf-8", errors="replace"
                ),
            )
            source = temporary / "source"
            snapshot_root = temporary / "snapshots"
            snapshot = snapshot_root / "snapshot"
            toolchain = temporary / "toolchain"
            admitted_root = temporary / "admitted"
            evidence_root = temporary / "evidence"
            target = admitted_root / "source-keys" / ("a" * 64)
            for directory in (
                source,
                snapshot_root,
                snapshot,
                toolchain,
                admitted_root,
                evidence_root,
                target.parent,
                target,
            ):
                directory.mkdir()
            (target / ".codefire-source-key.json").write_text(
                "{}\n", encoding="utf-8"
            )
            (target / ".codefire-source-key.lock").touch()
            (target / ".codefire-source-key-run-state.json").write_text(
                "{}\n", encoding="utf-8"
            )
            (source / "live.txt").write_text("live\n", encoding="utf-8")
            (snapshot / "snap.txt").write_text(
                "snapshot\n", encoding="utf-8"
            )
            snapshot_manifest = [
                {
                    "path": "snap.txt",
                    "mode": "100644",
                    "bytes": len(b"snapshot\n"),
                    "sha256": hashlib.sha256(b"snapshot\n").hexdigest(),
                }
            ]
            snapshot_manifest_sha256 = hashlib.sha256(
                json.dumps(
                    snapshot_manifest,
                    ensure_ascii=False,
                    sort_keys=True,
                    separators=(",", ":"),
                ).encode("utf-8")
            ).hexdigest()
            marker_raw = b'{"schema":"test.snapshot.v1"}\n'
            (snapshot / ".codefire-source-snapshot.json").write_bytes(
                marker_raw
            )
            # The real manager pins more than this helper is told about by
            # name: pinned_tool_files passes the policy supervisor, the
            # parent-death helper, and every toolchain driver including the
            # ones this gate did not select. Pin representatives of each so
            # the inherited-set control is exercised against the same leak
            # surface the gate actually has, not just the descriptors the
            # helper receives as named arguments.
            executables = {
                "unshare": Path("/usr/bin/unshare"),
                "mount": Path("/usr/bin/mount"),
                "setpriv": Path("/usr/bin/setpriv"),
                "python": Path(sys.executable).resolve(),
                "helper": TOOLS_DIR / "readonly_source_exec.py",
                "primary": Path(sys.executable).resolve(),
                "pdeath-helper": TOOLS_DIR / "pdeath_exec.py",
                "policy-supervisor": (
                    TOOLS_DIR / "run_source_keyed_cargo_gate.py"
                ),
                "extra-toolchain-cargo-fmt": Path("/usr/bin/true"),
                "extra-toolchain-clippy-driver": Path("/usr/bin/true"),
            }
            descriptors: dict[str, int] = {}
            try:
                for name, path in executables.items():
                    descriptors[name] = os.open(path, os.O_RDONLY)
                for name, path in (
                    ("source", source),
                    ("snapshot", snapshot),
                    ("snapshot-root", snapshot_root),
                    ("toolchain", toolchain),
                    ("target", target),
                    ("admitted-root", admitted_root),
                    ("evidence-root", evidence_root),
                ):
                    descriptors[name] = os.open(
                        path, os.O_RDONLY | os.O_DIRECTORY
                    )
                for name, path in (
                    (
                        "target-marker",
                        target / ".codefire-source-key.json",
                    ),
                    (
                        "target-lock",
                        target / ".codefire-source-key.lock",
                    ),
                    (
                        "target-run-state",
                        target / ".codefire-source-key-run-state.json",
                    ),
                ):
                    descriptors[name] = os.open(path, os.O_RDONLY)
                inherited = tuple(descriptors.values())
                command = [
                    f"/proc/self/fd/{descriptors['unshare']}",
                    "-Urnmpf",
                    "--mount-proc",
                    "--kill-child=SIGKILL",
                    "--propagation",
                    "private",
                    "--",
                    f"/proc/self/fd/{descriptors['python']}",
                    "-I",
                    "-S",
                    f"/proc/self/fd/{descriptors['helper']}",
                    "--source",
                    str(source),
                    "--snapshot",
                    str(snapshot),
                    "--snapshot-root",
                    str(snapshot_root),
                    "--toolchain-root",
                    str(toolchain),
                    "--target",
                    str(target),
                    "--admitted-root",
                    str(admitted_root),
                    "--evidence-root",
                    str(evidence_root),
                    "--source-fd",
                    str(descriptors["source"]),
                    "--snapshot-fd",
                    str(descriptors["snapshot"]),
                    "--snapshot-root-fd",
                    str(descriptors["snapshot-root"]),
                    "--snapshot-marker-sha256",
                    hashlib.sha256(marker_raw).hexdigest(),
                    "--snapshot-file-manifest-sha256",
                    snapshot_manifest_sha256,
                    "--snapshot-file-count",
                    "1",
                    "--snapshot-logical-file-bytes",
                    str(len(b"snapshot\n")),
                    "--private-source-tmpfs-bytes",
                    str(64 * 1024**2),
                    "--toolchain-root-fd",
                    str(descriptors["toolchain"]),
                    "--target-fd",
                    str(descriptors["target"]),
                    "--admitted-root-fd",
                    str(descriptors["admitted-root"]),
                    "--evidence-root-fd",
                    str(descriptors["evidence-root"]),
                    "--target-marker-fd",
                    str(descriptors["target-marker"]),
                    "--target-lock-fd",
                    str(descriptors["target-lock"]),
                    "--target-run-state-fd",
                    str(descriptors["target-run-state"]),
                    "--mount-fd",
                    str(descriptors["mount"]),
                    "--setpriv-fd",
                    str(descriptors["setpriv"]),
                    "--python-fd",
                    str(descriptors["python"]),
                    "--helper-fd",
                    str(descriptors["helper"]),
                    "--primary-fd",
                    str(descriptors["primary"]),
                    "--pass-fds",
                    ",".join(str(item) for item in inherited),
                    "--",
                    str(Path(sys.executable).resolve()),
                    "-c",
                    (
                        "import os; from pathlib import Path; "
                        "assert os.getpid() == 1; "
                        "assert Path('snap.txt').read_bytes() == b'snapshot\\n'; "
                        "assert not Path('live.txt').exists(); "
                        "assert not Path('.codefire-source-snapshot.json').exists(); "
                        f"assert {str(snapshot)!r} not in "
                        "Path('/proc/self/mountinfo').read_text(); "
                        # Bound this by the property, not by an
                        # enumeration. Asserting that a listed handful of
                        # descriptors got closed cannot fail while other
                        # inherited descriptors survive, so it could only
                        # confirm the change already made. Instead read the
                        # command's *entire* inherited set from /proc and
                        # require it to equal the retained set exactly:
                        # stdio, plus the primary tool, plus whatever the
                        # child environment names. Anything else the
                        # manager pinned -- unshare, the parent-death
                        # helper, the policy supervisor, mount, setpriv,
                        # every unselected toolchain driver -- is a program
                        # a build.rs could exec through /proc/self/fd.
                        "expected = {0, 1, 2}\n"
                        f"expected.add({descriptors['primary']})\n"
                        "for value in os.environ.values():\n"
                        "    if value.startswith('/proc/self/fd/'):\n"
                        "        tail = value.rsplit('/', 1)[1]\n"
                        "        if tail.isdigit():\n"
                        "            expected.add(int(tail))\n"
                        "inherited = {\n"
                        "    int(name)\n"
                        "    for name in os.listdir('/proc/self/fd')\n"
                        "    if name.isdigit()\n"
                        "}\n"
                        # The listdir descriptor itself is already gone.
                        "inherited = {\n"
                        "    fd for fd in inherited\n"
                        "    if fd in expected or os.path.exists(f'/proc/self/fd/{fd}')\n"
                        "}\n"
                        "leaked = sorted(inherited - expected)\n"
                        "assert not leaked, (\n"
                        "    'inherited descriptors beyond the retained set: '\n"
                        "    + repr([\n"
                        "        (fd, os.readlink(f'/proc/self/fd/{fd}'))\n"
                        "        for fd in leaked\n"
                        "    ])\n"
                        ")\n"
                        # Nothing in the namespace may propagate: the source
                        # tmpfs sits on the candidate worktree's own path.
                        "info = Path('/proc/self/mountinfo').read_text()\n"
                        "assert 'shared:' not in info, info\n"
                        "assert 'master:' not in info, info\n"
                    ),
                ]
                environment = dict(os.environ)
                environment["CARGO_TARGET_DIR"] = str(target)
                result = subprocess.run(
                    command,
                    cwd=temporary,
                    env=environment,
                    stdin=subprocess.DEVNULL,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    pass_fds=inherited,
                    timeout=20,
                    check=False,
                )
            finally:
                for descriptor in descriptors.values():
                    os.close(descriptor)
            self.assertEqual(
                result.returncode,
                0,
                (result.stdout + result.stderr).decode(
                    "utf-8", errors="replace"
                ),
            )
            self.assertEqual(
                (source / "live.txt").read_text(encoding="utf-8"),
                "live\n",
            )
            self.assertFalse((source / "snap.txt").exists())

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
            self.assertEqual(stat.S_IMODE(log.stat().st_mode), 0o400)

    def test_gate_hash_includes_caller_prefix_and_uses_supplied_environment(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            log = root / "gate.log"
            prefix = b'{"source_key":"abc"}\n'
            environment = dict(os.environ)
            environment["CODEFIRE_GATE_TEST"] = "bound-value"
            result = run_gate(
                [
                    sys.executable,
                    "-c",
                    "import os; print(os.environ['CODEFIRE_GATE_TEST'])",
                ],
                root,
                log,
                5,
                environment=environment,
                log_prefix=prefix,
            )
            expected = prefix + b"bound-value\n"
            self.assertEqual(log.read_bytes(), expected)
            self.assertEqual(
                result["log"]["sha256"], hashlib.sha256(expected).hexdigest()
            )

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

class PolicyModulePreloadTests(unittest.TestCase):
    """The pinned policy preload must bind one instance, not a second copy."""

    def test_preloaded_policy_modules_reuse_the_live_instance(self) -> None:
        # Counterexample: this test module imports
        # check_rust_execution_profile before it imports the gate. A preload
        # that unconditionally re-executed the file would leave the gate
        # enforcing a second DEFAULT_PROFILE while every earlier importer
        # kept mutating the orphaned one -- so patching the profile in a
        # test, or overriding it in the release launcher, would silently
        # have no effect on the gate that actually runs Cargo.
        import check_rust_execution_profile
        import run_source_keyed_cargo_gate
        import run_timed_gate

        self.assertIs(
            check_rust_execution_profile.DEFAULT_PROFILE,
            run_source_keyed_cargo_gate.DEFAULT_PROFILE,
        )
        self.assertIs(
            sys.modules["check_rust_execution_profile"],
            check_rust_execution_profile,
        )
        self.assertIs(sys.modules["run_timed_gate"], run_timed_gate)
        for name in (
            "ai_contracts",
            "build_source_bound_rust",
            "check_rust_execution_profile",
            "run_timed_gate",
        ):
            record = run_source_keyed_cargo_gate._PRELOADED_POLICY_MODULES[
                name
            ]
            self.assertEqual(
                Path(sys.modules[name].__file__).resolve(),
                record["path"],
            )

    def test_policy_module_bound_to_a_foreign_origin_is_rejected(
        self,
    ) -> None:
        # Reuse is admitted only for a module already loaded from exactly
        # the pinned canonical path. A name hijacked from anywhere else
        # must fail closed rather than be silently replaced.
        import run_source_keyed_cargo_gate

        name = "ai_contracts"
        pinned = (
            run_source_keyed_cargo_gate.TOOLS_DIRECTORY / f"{name}.py"
        )
        impostor = types.ModuleType(name)
        impostor.__file__ = "/tmp/attacker/ai_contracts.py"
        original = sys.modules[name]
        sys.modules[name] = impostor
        try:
            with self.assertRaisesRegex(RuntimeError, "foreign origin"):
                run_source_keyed_cargo_gate._preload_policy_module(
                    name, pinned
                )
        finally:
            sys.modules[name] = original

    def test_parent_death_helper_refuses_an_orphaned_gate(self) -> None:
        # Counterexample for using bare `setpriv --pdeathsig SIGKILL`:
        # setpriv arms the parent-death signal only after the fork, so a
        # manager that dies in that window leaves an unsupervised Cargo
        # namespace with no one to kill it. The helper re-reads getppid
        # after the prctl call and refuses to exec instead.
        helper = (
            Path(__file__).resolve().with_name("pdeath_exec.py")
        )
        completed = subprocess.run(
            [
                sys.executable,
                "-I",
                "-S",
                str(helper),
                "--parent-pid",
                "1",
                "--",
                "/bin/echo",
                "should-not-run",
            ],
            capture_output=True,
            check=False,
        )
        self.assertEqual(completed.returncode, 125)
        self.assertEqual(completed.stdout, b"")

    def test_parent_death_helper_execs_under_the_expected_parent(
        self,
    ) -> None:
        helper = Path(__file__).resolve().with_name("pdeath_exec.py")
        completed = subprocess.run(
            [
                sys.executable,
                "-I",
                "-S",
                str(helper),
                "--parent-pid",
                str(os.getpid()),
                "--",
                "/bin/echo",
                "armed",
            ],
            capture_output=True,
            check=False,
        )
        self.assertEqual(completed.returncode, 0)
        self.assertEqual(completed.stdout.strip(), b"armed")

class HistoryTestCountConsistencyTests(unittest.TestCase):
    """Every test count in the permanent record must name a real commit.

    Twice now this row has carried a number that matches no tree: `157`,
    then `161` inside the fix for `157`. Both times a commit message
    claimed the count was corrected. Prose review did not catch either,
    so the count is checked mechanically here instead.

    The counts the row must state are literals in `CLAIMED`. A commit's
    test count is a fixed historical fact, so holding it as data keeps
    both row assertions active in a checkout where the historical objects
    are not reachable at all -- a `git archive` extraction, a shallow or
    single-branch clone. The separate re-derivation test below re-earns
    those literals from each named commit's own blob whenever the objects
    are present. That pairing is what keeps the control from going
    self-referential: the expected value never comes from the suite
    running the assertion, so adding tests in this or any later repair
    changes the current suite size but cannot change what `aaa39b6`
    defines.
    """

    HISTORY = ROOT / "docs/development/history.md"
    # Concise label the ledger row spells -> (full revision identifier,
    # the count the permanent record states for it). `git show` is given
    # the full identifier, because an abbreviation is unique only until
    # the object database grows into it; rows are matched on the short
    # label, because that is what the ledger has always written.
    CLAIMED = {
        "bfbbc2b": ("bfbbc2b2bdc22f88679e5e1dc794665d863304a1", 148),
        "aaa39b6": ("aaa39b61b37df273fe4d57e274ed8e1321e788ad", 159),
        "4e5c0e7": ("4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912", 168),
    }
    # Both spellings this ledger has used for a count.
    #
    # ASCII: "<N> test" / "<N> tests". `\b` does not fire between "test"
    # and a CJK character, so "148 test中" and "159 test全pass" would both
    # be missed; anchor on "not an ASCII letter" instead.
    #
    # Japanese: "<N>件全pass" -- the exact form *both* wrong counts were
    # written in ("...ものを157件全passへ", "候補は161件全pass"). An
    # ASCII-only extractor sees nothing in either, so re-adding the precise
    # recurrence this class exists to stop would pass unnoticed.
    #
    # "件" alone is not a count of tests: the same row carries
    # "MUST/CRITICAL 6件", "10件fail" and "12件に見える". Requiring the
    # "全pass" suffix keeps those out.
    COUNT_FORMS = r"(?:tests?(?![A-Za-z])|件全pass)"

    @classmethod
    def stated_counts(cls, row: str) -> set[int]:
        """Every test-count claim in one ledger row, in either spelling."""
        return {
            int(value)
            for value in re.findall(
                rf"(\d+)\s*{cls.COUNT_FORMS}", " ".join(row.split())
            )
        }

    @staticmethod
    def defined_test_count(revision: str) -> int | None:
        """Count test methods in one commit's test module, statically.

        Parsing the blob avoids executing a historical suite (slow, and it
        would need that commit's environment) while still producing a
        number that is independent of the assertion's own tree.

        Returns `None` when the object is not reachable from this checkout
        rather than raising: a missing historical blob is a property of the
        checkout, not a defect in the permanent record, and only the
        re-derivation test is allowed to react to it.
        """
        completed = subprocess.run(
            [
                "git",
                "show",
                f"{revision}:docs/development/tools/test_ai_tools.py",
            ],
            cwd=ROOT,
            capture_output=True,
            check=False,
        )
        if completed.returncode != 0:
            return None
        module = ast.parse(completed.stdout)
        total = 0
        for node in ast.walk(module):
            if not isinstance(node, ast.ClassDef):
                continue
            for item in node.body:
                if isinstance(
                    item, (ast.FunctionDef, ast.AsyncFunctionDef)
                ) and item.name.startswith("test"):
                    total += 1
        return total

    def test_static_count_agrees_with_the_recorded_runtime_count(
        self,
    ) -> None:
        # The literals in CLAIMED are only trustworthy if they reproduce
        # what `python3 -m unittest` reported for those trees. Measured:
        # bfbbc2b 148, aaa39b6 159, 4e5c0e7 168 -- all matching.
        #
        # This is the one test here that depends on the historical objects,
        # so it is the only one that may skip. The two row assertions run
        # on the literals and stay active either way.
        derived = {
            label: self.defined_test_count(revision)
            for label, (revision, _) in self.CLAIMED.items()
        }
        if all(count is None for count in derived.values()):
            self.skipTest(
                "no historical blob is reachable from this checkout "
                "(git archive extraction or shallow clone); the CLAIMED "
                "literals stay active in the two row assertions"
            )
        for label, (_, expected) in self.CLAIMED.items():
            with self.subTest(revision=label):
                if derived[label] is None:
                    self.skipTest(f"{label} unreachable from this checkout")
                self.assertEqual(derived[label], expected)

    def test_the_historical_wrong_count_forms_are_recognised(self) -> None:
        # Exact bytes from the two commits that carried a wrong count:
        # aaa39b6's row wrote "157件全passへ" and 4e5c0e7's wrote
        # "候補は161件全pass". Neither spells "test", so this is the
        # control that keeps the set-difference guard below from being
        # vacuous against the exact recurrence it is named for.
        self.assertEqual(
            self.stated_counts(
                "checkpoint時点でtest 148中10件failだったものを157件全passへ。"
            ),
            {157},
        )
        self.assertEqual(
            self.stated_counts("候補は161件全pass。"), {161}
        )
        # ASCII forms, including the CJK-adjacent ones.
        self.assertEqual(
            self.stated_counts(
                "`bfbbc2b`はGit worktree内で148 test中10件fail。"
                "`aaa39b6`で159 test全pass。`4e5c0e7`は168 tests。"
            ),
            {148, 159, 168},
        )
        # Counts of things that are not tests must stay out, or every row
        # would state counts matching no tree and the guard would be noise.
        self.assertEqual(
            self.stated_counts(
                "MUST/CRITICAL 6件を閉じた。errors=6。12件に見える。"
                "陳腐化していたtest 3件をより強い契約へ更新した"
            ),
            set(),
        )

    def source_keyed_rows(self) -> list[str]:
        """History rows that name at least one source-keyed lineage commit.

        `history.md` is a long ledger whose other rows carry unrelated
        counts for the product CLI and the algorithm lab. Scoping by the
        commits this lineage owns keeps the guard from adjudicating rows
        it knows nothing about.
        """
        rows = [
            row
            for row in self.HISTORY.read_text(
                encoding="utf-8"
            ).splitlines()
            if any(f"`{label}`" in row for label in self.CLAIMED)
        ]
        self.assertTrue(
            rows, "no source-keyed history row found; guard is vacuous"
        )
        return rows

    def test_history_states_the_right_count_for_each_named_commit(
        self,
    ) -> None:
        for row in self.source_keyed_rows():
            for label, (_, expected) in self.CLAIMED.items():
                if f"`{label}`" not in row:
                    continue
                with self.subTest(revision=label):
                    self.assertRegex(
                        " ".join(row.split()),
                        rf"`{label}`[^`]*?{expected}\s*{self.COUNT_FORMS}",
                        f"a history row names {label} but does not "
                        f"state its {expected}-test count next to it",
                    )

    def test_history_states_no_test_count_matching_no_tree(self) -> None:
        # The direct reversal for `157` and `161`: inside a source-keyed
        # row, every stated count must equal a count that a commit named
        # in that same row actually defines. 157 and 161 match none of
        # {148, 159, 168}, so either value fails here -- including in the
        # Japanese "件全pass" spelling both were originally written in.
        for row in self.source_keyed_rows():
            named = [
                label for label in self.CLAIMED if f"`{label}`" in row
            ]
            admitted = {self.CLAIMED[label][1] for label in named}
            claimed = self.stated_counts(row)
            with self.subTest(named=tuple(named)):
                self.assertTrue(
                    claimed,
                    "a source-keyed row states no test count; the guard "
                    "would be vacuous for it",
                )
                self.assertEqual(
                    claimed - admitted,
                    set(),
                    f"history row states test counts matching no tree "
                    f"it names: {sorted(claimed - admitted)}; "
                    f"admitted {sorted(admitted)} from {named}",
                )

class SourceKeyedGateDocumentationTests(unittest.TestCase):
    """The contract document must not outrun the implementation.

    Every finding these pin was a claim-versus-implementation defect rather
    than a missing control, so the regression has to be against the prose.
    """

    @property
    def contract(self) -> str:
        # Collapse wrapping so an assertion pins the sentence rather than
        # the column the sentence happened to break at.
        raw = (
            ROOT / "docs/development/ai/source-keyed-cargo-gates.md"
        ).read_text(encoding="utf-8")
        return " ".join(raw.split())

    def test_reclaim_is_not_described_as_exact_or_charged_back(
        self,
    ) -> None:
        text = self.contract
        self.assertNotIn("charged back against the", text)
        self.assertIn(
            "Nothing charges that figure back against the aggregate budget",
            text,
        )
        # The hard-link convention must be stated, not implied.
        self.assertIn("no inode de-duplication", text)
        self.assertIn("counted once per link", text)
        self.assertIn("upper bound", text)

    def test_descriptor_closure_is_described_as_a_complement(
        self,
    ) -> None:
        text = self.contract
        # The old wording promised an invariant the enumeration did not
        # implement. It must not come back.
        self.assertNotIn(
            "retaining only the descriptors the child environment itself names",
            text,
        )
        self.assertIn("complement of the retained set", text)
        self.assertIn("--pass-fds", text)
        # The survivor clause must be conditional on the declared set:
        # the close set is the complement over the parsed --pass-fds
        # string, so an undeclared inherited descriptor would survive.
        self.assertIn(
            "Of the descriptors declared in `--pass-fds`, what survives",
            text,
        )
        self.assertIn("complement over the *declared* set", text)

    def test_capability_drop_is_not_claimed_as_a_durable_boundary(
        self,
    ) -> None:
        text = self.contract
        self.assertIn(
            "The capability drop is not a durable capability boundary",
            text,
        )
        self.assertIn("CLONE_NEWUSER|CLONE_NEWNS", text)
        self.assertIn(
            "kernel's locking of mounts copied into a child user namespace",
            text,
        )

    def test_policy_module_reuse_claim_is_narrowed(self) -> None:
        text = self.contract
        self.assertIn(
            "after verifying the pinned file and the module's origin path",
            text,
        )
        self.assertIn(
            "not of the code the interpreter actually executed", text
        )

    def test_export_source_link_bound_is_stated_not_overclaimed(
        self,
    ) -> None:
        text = self.contract
        # The document claimed the export source was single-link. No real
        # `cargo build --release` artifact is: the live 2026-08-02 release
        # build blocked on exactly that. The claim must not return.
        self.assertNotIn("single-link executable", text)
        self.assertIn(
            "The export source is admitted with one or two links", text
        )
        self.assertIn("Cargo's uplift", text)
        # Relaxing the source bound must not be read as relaxing the
        # destination or the mutation check, so both have to stay written
        # down beside it.
        self.assertIn(
            "The published destination is separate and stays strictly "
            "single-link",
            text,
        )
        self.assertIn("artifact-changed-during-export", text)

    def test_stdout_result_agreement_is_documented_as_byte_equality(
        self,
    ) -> None:
        text = self.contract
        # "stdout ... agree" was true of the intent and false of the code:
        # the gate pretty-printed a second rendering, so the wrapper's
        # comparison failed on every live invocation. Say what the
        # agreement is and which paths owe it.
        self.assertIn(
            "That stdout agreement is byte equality, checked before the "
            "decision is read",
            text,
        )
        self.assertIn(
            "the reservation fallback after a failed publication", text
        )
        self.assertIn("never a second rendering of them", text)

    def documented_unshare_sequence(self) -> list[str]:
        """The flag sequence the contract document publishes, as tokens."""
        match = re.search(r"`unshare ([^`]+)`", self.contract)
        self.assertIsNotNone(
            match, "the contract no longer quotes an unshare launch line"
        )
        return match.group(1).split()

    def test_documented_unshare_sequence_equals_the_constant(
        self,
    ) -> None:
        # MUST-2 failed by the recorded list being a strict *prefix* of the
        # launched one. A containment assertion cannot see that: the doc
        # spells five flags, so it still contains the join of a shortened
        # three-flag constant and the test passes while the property is
        # broken. Compare the extracted token sequence for equality, which
        # fixes both the contents and the order.
        self.assertEqual(
            self.documented_unshare_sequence(),
            list(UNSHARE_ISOLATION_FLAGS),
        )
        # Equality alone would still be satisfied by shortening the doc and
        # the constant together, so pin the flag the propagation guarantee
        # depends on and its argument's position.
        documented = self.documented_unshare_sequence()
        self.assertIn("--propagation", documented)
        self.assertEqual(
            documented[documented.index("--propagation") + 1], "private"
        )

    def test_unshare_sequence_binds_identity_and_both_keys(
        self,
    ) -> None:
        # Two of the carriers MUST-2 named are covered here: the recorded
        # isolation identity, and each of the two keys. The launched argv
        # and the published result are covered by
        # `test_gate_uses_locked_source_namespace_and_source_keyed_log`,
        # which is the only test that runs a gate; nothing in this test
        # observes a launch, so it makes no argv claim.
        #
        # The keys are checked by mutating only the flag list and
        # requiring each key to move on its own. Uniqueness of the
        # (source, artifact) pair would not do: a pair is already distinct
        # when just one of its two members changes, so a key that stopped
        # binding the sequence entirely would still pass.
        isolation = isolation_identity()
        self.assertEqual(
            isolation["unshare_flags"], list(UNSHARE_ISOLATION_FLAGS)
        )

        source_state = {
            "candidate_root": "/candidate",
            "tree": "t" * 40,
            "rust_source_manifest_sha256": "a" * 64,
            "rust_source_file_count": 1,
        }
        profile = dict(DEFAULT_PROFILE)
        toolchain = {"executables": {}, "toolchain_root": "/toolchain"}
        source_keys: list[str] = []
        artifact_keys: list[str] = []
        for flags in (
            list(UNSHARE_ISOLATION_FLAGS),
            # The exact pre-fix list MUST-2 was raised on.
            ["-Urnmpf", "--mount-proc", "--kill-child=SIGKILL"],
            # Same members, different order.
            list(reversed(UNSHARE_ISOLATION_FLAGS)),
        ):
            mutated = dict(isolation)
            mutated["unshare_flags"] = flags
            source_key, _ = build_source_identity(
                source_state,
                environment_sha256="b" * 64,
                profile=profile,
                toolchain=toolchain,
                isolation=mutated,
            )
            artifact_key, _ = build_artifact_identity(
                source_state,
                environment_sha256="b" * 64,
                profile=profile,
                toolchain=toolchain,
                isolation=mutated,
            )
            source_keys.append(source_key)
            artifact_keys.append(artifact_key)

        # Projected independently, so each assertion can only be satisfied
        # by the key it names.
        self.assertEqual(
            len(set(source_keys)),
            len(source_keys),
            "the unshare flag sequence does not bind into the source key: "
            "shortening or reordering it left the source key unchanged",
        )
        self.assertEqual(
            len(set(artifact_keys)),
            len(artifact_keys),
            "the unshare flag sequence does not bind into the artifact "
            "key: shortening or reordering it left the artifact key "
            "unchanged",
        )

class ArtifactExportIdentityTests(unittest.TestCase):
    """Pin every export identity predicate around the uplift acceptance.

    `export_bound_artifact` used to require `st_nlink == 1` on the source,
    which no real `cargo build --release` artifact can satisfy: Cargo
    uplifts by hard-linking the compiled `deps/` inode onto the
    profile-root name. Relaxing a link count is the kind of edit that can
    quietly carry the neighbouring checks with it, so the acceptance and
    each protection it sits beside are asserted together. Every rejection
    below must also leave no destination and no staging residue.
    """

    COMPILED_NAME = "codefire_rs-0123456789abcdef"

    def make_scene(self, temporary: Path) -> tuple[Path, Path, Path]:
        target = temporary / "namespace"
        (target / "release/deps").mkdir(parents=True)
        candidate = temporary / "candidate"
        (candidate / "target/release").mkdir(parents=True)
        return target, candidate, candidate / "target/release/codefire-rs"

    def write_artifact(
        self,
        target: Path,
        payload: bytes = b"source-keyed-binary",
        *,
        links: int = 2,
        mode: int = 0o755,
    ) -> Path:
        artifact = target / "release/codefire-rs"
        if links == 1:
            artifact.write_bytes(payload)
            artifact.chmod(mode)
            return artifact
        # Written in Cargo's own direction: the bytes are produced under
        # `deps/` and the profile-root name is a second link to that exact
        # inode, not a copy of it.
        compiled = target / "release/deps" / self.COMPILED_NAME
        compiled.write_bytes(payload)
        compiled.chmod(mode)
        os.link(compiled, artifact)
        for index in range(links - 2):
            os.link(compiled, target / f"release/deps/alias-{index}")
        self.assertEqual(artifact.stat().st_nlink, links)
        return artifact

    def export(
        self, target: Path, candidate: Path, destination: Path
    ) -> dict[str, object]:
        return export_bound_artifact(
            target,
            "release/codefire-rs",
            candidate,
            destination,
            free_floor_bytes=0,
        )

    def staging_residue(self, destination: Path) -> list[str]:
        return sorted(
            path.name
            for path in destination.parent.iterdir()
            if path.name.startswith(f".{destination.name}.")
        )

    def assert_failed_closed(self, destination: Path) -> None:
        self.assertFalse(destination.exists())
        self.assertEqual(self.staging_residue(destination), [])

    def test_cargo_uplift_hardlinked_source_exports_a_private_copy(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as raw:
            temporary = Path(raw).resolve()
            target, candidate, destination = self.make_scene(temporary)
            artifact = self.write_artifact(target)
            source = artifact.stat()
            result = self.export(target, candidate, destination)
            self.assertEqual(result["decision"], "EXPORTED")
            self.assertEqual(result["source_nlink"], 2)
            self.assertTrue(result["source_cargo_uplift_hardlinked"])
            self.assertEqual(
                result["source_nlink_maximum"],
                MAX_EXPORT_SOURCE_ARTIFACT_LINKS,
            )
            self.assertEqual(
                destination.read_bytes(), b"source-keyed-binary"
            )
            self.assertEqual(
                result["sha256"],
                hashlib.sha256(b"source-keyed-binary").hexdigest(),
            )
            published = destination.stat()
            self.assertTrue(stat.S_ISREG(published.st_mode))
            self.assertEqual(published.st_nlink, 1)
            self.assertEqual(result["export_nlink"], 1)
            self.assertEqual(stat.S_IMODE(published.st_mode), 0o700)
            # The copy goes through a descriptor. A third link onto the
            # uplifted inode would satisfy "same bytes" and still leave the
            # exported binary writable from inside the artifact namespace.
            self.assertNotEqual(published.st_ino, source.st_ino)
            self.assertEqual(artifact.stat().st_nlink, 2)

    def test_over_linked_source_is_blocked(self) -> None:
        # The bound is the uplift pair and nothing wider. Assert the
        # constant too: widening it later must break this test rather than
        # silently admit an inode with more aliases than Cargo creates.
        self.assertEqual(MAX_EXPORT_SOURCE_ARTIFACT_LINKS, 2)
        for links in (3, 4):
            with self.subTest(links=links), tempfile.TemporaryDirectory() as raw:
                temporary = Path(raw).resolve()
                target, candidate, destination = self.make_scene(temporary)
                self.write_artifact(target, links=links)
                with self.assertRaises(GateBlocked) as raised:
                    self.export(target, candidate, destination)
                self.assertEqual(
                    raised.exception.code,
                    "artifact-not-private-executable",
                )
                self.assertIn(f"nlink={links}", raised.exception.detail)
                self.assert_failed_closed(destination)

    def test_symlinked_source_is_rejected_by_the_no_follow_open(
        self,
    ) -> None:
        # The symlink rejection is the kernel's, not a predicate the link
        # bound could weaken: O_NOFOLLOW fails the open before any stat is
        # taken. Pin the errno so a future rewrite cannot lose the flag and
        # keep passing on the link-count check alone.
        with tempfile.TemporaryDirectory() as raw:
            temporary = Path(raw).resolve()
            target, candidate, destination = self.make_scene(temporary)
            real = target / "release/deps" / self.COMPILED_NAME
            real.write_bytes(b"source-keyed-binary")
            real.chmod(0o755)
            (target / "release/codefire-rs").symlink_to(real)
            with self.assertRaises(OSError) as raised:
                self.export(target, candidate, destination)
            self.assertEqual(raised.exception.errno, errno.ELOOP)
            self.assert_failed_closed(destination)

    def test_nonregular_or_nonexecutable_source_is_blocked(self) -> None:
        cases = (
            ("directory", None, 0o700, "regular=False"),
            ("not-executable-uplifted", 2, 0o600, "mode=0o0600"),
            ("not-executable-single-link", 1, 0o600, "mode=0o0600"),
        )
        for name, links, mode, expected in cases:
            with self.subTest(case=name), tempfile.TemporaryDirectory() as raw:
                temporary = Path(raw).resolve()
                target, candidate, destination = self.make_scene(temporary)
                if links is None:
                    (target / "release/codefire-rs").mkdir(mode=mode)
                else:
                    self.write_artifact(target, links=links, mode=mode)
                with self.assertRaises(GateBlocked) as raised:
                    self.export(target, candidate, destination)
                self.assertEqual(
                    raised.exception.code,
                    "artifact-not-private-executable",
                )
                self.assertIn(expected, raised.exception.detail)
                self.assert_failed_closed(destination)

    def test_mutation_through_the_second_link_blocks_the_export(
        self,
    ) -> None:
        # This is the protection the single-link rule was standing in for.
        # With two links admitted, a writer holding the `deps/` name can
        # touch the inode mid-copy, so the pre/post identity comparison has
        # to be what actually fails closed. Both shapes are covered: an
        # equal-size rewrite, which only the timestamps reveal, and an
        # append, which also moves the size.
        for mutation in ("same-size-rewrite", "append"):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as raw:
                temporary = Path(raw).resolve()
                target, candidate, destination = self.make_scene(temporary)
                self.write_artifact(target)
                compiled = target / "release/deps" / self.COMPILED_NAME
                original = gate_module.write_all
                observed = {"mutated": False}

                def write_all_then_mutate(
                    descriptor: int,
                    payload: bytes,
                    _mutation: str = mutation,
                ) -> None:
                    original(descriptor, payload)
                    if observed["mutated"]:
                        return
                    observed["mutated"] = True
                    if _mutation == "append":
                        with open(compiled, "ab") as handle:
                            handle.write(b"-tampered")
                        return
                    with open(compiled, "r+b") as handle:
                        handle.write(b"X")
                    # Filesystem timestamp granularity is not a test input:
                    # force an observably different mtime so an equal-size
                    # rewrite cannot pass by landing inside the same tick
                    # as the pre-copy capture.
                    os.utime(compiled, ns=(1_000_000_000, 1_000_000_000))

                with mock.patch.object(
                    gate_module, "write_all", write_all_then_mutate
                ):
                    with self.assertRaises(GateBlocked) as raised:
                        self.export(target, candidate, destination)
                self.assertTrue(observed["mutated"])
                self.assertEqual(
                    raised.exception.code, "artifact-changed-during-export"
                )
                self.assert_failed_closed(destination)

    def test_unsafe_prior_destination_is_never_replaced(self) -> None:
        # The destination side keeps the strict single-link rule. A prior
        # export that is a symlink, or that carries a link the crash-residue
        # recovery cannot account for, must block with the prior bytes left
        # exactly as they were.
        cases = (
            ("symlink", 0, "unsafe-prior-artifact-export"),
            ("unaccounted-link", 1, "ambiguous-artifact-export-recovery"),
            ("over-linked", 2, "unsafe-prior-artifact-export"),
        )
        for name, extra_links, expected in cases:
            with self.subTest(case=name), tempfile.TemporaryDirectory() as raw:
                temporary = Path(raw).resolve()
                target, candidate, destination = self.make_scene(temporary)
                self.write_artifact(target)
                if name == "symlink":
                    elsewhere = candidate / "target/release/elsewhere"
                    elsewhere.write_bytes(b"prior")
                    destination.symlink_to(elsewhere)
                else:
                    destination.write_bytes(b"prior")
                    destination.chmod(0o700)
                    for index in range(extra_links):
                        os.link(
                            destination,
                            destination.parent / f"keep-{index}",
                        )
                with self.assertRaises(GateBlocked) as raised:
                    self.export(target, candidate, destination)
                self.assertEqual(raised.exception.code, expected)
                self.assertEqual(destination.read_bytes(), b"prior")
                self.assertEqual(self.staging_residue(destination), [])


class SourceKeyedCargoGateTests(unittest.TestCase):
    def make_repository(
        self, temporary: Path
    ) -> tuple[Path, dict[str, object], dict[str, str]]:
        root = temporary / "candidate"
        (root / "crates/example/src").mkdir(parents=True)
        (root / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["crates/example"]\n',
            encoding="utf-8",
        )
        (root / "Cargo.lock").write_text("# locked\n", encoding="utf-8")
        (root / ".gitignore").write_text("/target/\n", encoding="utf-8")
        (root / "crates/example/Cargo.toml").write_text(
            '[package]\nname = "example"\nversion = "0.1.0"\n',
            encoding="utf-8",
        )
        (root / "crates/example/src/lib.rs").write_text(
            "pub fn value() -> u8 { 1 }\n",
            encoding="utf-8",
        )
        subprocess.run(["git", "init", "-q"], cwd=root, check=True)
        subprocess.run(
            ["git", "config", "user.email", "gate-test@example.invalid"],
            cwd=root,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "Gate Test"],
            cwd=root,
            check=True,
        )
        subprocess.run(["git", "add", "."], cwd=root, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "candidate"], cwd=root, check=True
        )

        rustup = temporary / "rustup"
        cargo_home = temporary / "cargo-home"
        target = temporary / "shared-target"
        evidence = temporary / "evidence"
        snapshots = temporary / "snapshots"
        rustup.mkdir()
        (cargo_home / "bin").mkdir(parents=True)
        target.mkdir()
        evidence.mkdir(mode=0o700)
        snapshots.mkdir(mode=0o700)
        toolchain_name = "1.97.0-test-target"
        toolchain_bin = rustup / "toolchains" / toolchain_name / "bin"
        toolchain_bin.mkdir(parents=True)
        for name in (
            "cargo",
            "cargo-clippy",
            "cargo-fmt",
            "clippy-driver",
            "rustc",
            "rustdoc",
            "rustfmt",
        ):
            executable = toolchain_bin / name
            executable.write_text(
                f"#!/bin/sh\nprintf '%s test-version\\n' '{name}'\n",
                encoding="utf-8",
            )
            executable.chmod(0o700)
        cargo = cargo_home / "bin/cargo"
        cargo.write_text("#!/bin/sh\nexit 99\n", encoding="utf-8")
        cargo.chmod(0o700)
        profile: dict[str, object] = {
            "rustup_home": str(rustup),
            "cargo_home": str(cargo_home),
            "rustup_toolchain": toolchain_name,
            "target_dir": str(target),
            "evidence_root": str(evidence),
            "snapshot_root": str(snapshots),
            "max_build_jobs": 2,
            "max_test_threads": 2,
            "incremental": "0",
        }
        environment = {
            "HOME": str(temporary),
            "PATH": "/usr/bin",
        }
        return root, profile, environment

    def fake_runner(
        self,
        output: bytes,
        capture: dict[str, object] | None = None,
    ):
        def run(
            argv: list[str],
            cwd: Path,
            log_path: Path,
            timeout_seconds: float,
            *,
            environment: dict[str, str],
            log_prefix: bytes,
            log_binding_prefix: bytes,
            log_analyzer,
            pass_fds: tuple[int, ...],
            runtime_guard,
            pid_namespace_isolated: bool,
        ) -> dict[str, object]:
            log_path.parent.mkdir(parents=True, exist_ok=True)
            process_output = output
            if b"test result:" in output and b"Running " not in output:
                target = Path(environment["CARGO_TARGET_DIR"])
                process_output = (
                    f"     Running unittests src/main.rs "
                    f"({target}/debug/deps/example-abcd1234)\n".encode()
                    + output
                )
            payload = log_prefix + process_output
            log_path.write_bytes(payload)
            log_analyzer.feed(process_output)
            analysis = log_analyzer.finish()
            if capture is not None:
                capture.update(
                    {
                        "argv": argv,
                        "cwd": cwd,
                        "timeout": timeout_seconds,
                        "environment": environment,
                        "log_prefix": log_prefix,
                        "pass_fds": pass_fds,
                        "runtime_guard": runtime_guard,
                        "pid_namespace_isolated": pid_namespace_isolated,
                    }
                )
            return {
                "schema": "codefire.timed_gate.v1",
                "status": "passed",
                "exit_code": 0,
                "timed_out": False,
                "duration_seconds": 0.125,
                "duration_source": "time.monotonic_ns",
                "runtime_guard": {
                    "enabled": True,
                    "interval_seconds": 1.0,
                    "decision": "PASS",
                    "error": None,
                },
                "log": {
                    "path": str(log_path),
                    "bytes": len(payload),
                    "sha256": hashlib.sha256(payload).hexdigest(),
                    "bound_sha256": hashlib.sha256(
                        log_binding_prefix + payload
                    ).hexdigest(),
                    "complete": True,
                    "max_bytes": 256 * 1024 * 1024,
                },
                "log_analysis": analysis,
            }

        return run

    def test_source_key_binds_tree_dirty_state_and_worktree_root(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, _ = self.make_repository(temporary)
            toolchain = probe_toolchain(profile, "test")
            isolation = isolation_identity()
            clean = capture_source_state(root)
            first_key, _ = build_source_identity(
                clean,
                environment_sha256="a" * 64,
                profile=profile,
                toolchain=toolchain,
                isolation=isolation,
            )
            changed_root = dict(clean)
            changed_root["candidate_root"] = str(temporary / "other-worktree")
            other_key, _ = build_source_identity(
                changed_root,
                environment_sha256="a" * 64,
                profile=profile,
                toolchain=toolchain,
                isolation=isolation,
            )
            self.assertNotEqual(first_key, other_key)

            source = root / "crates/example/src/lib.rs"
            source.write_text(
                "pub fn value() -> u8 { 2 }\n", encoding="utf-8"
            )
            dirty = capture_source_state(root)
            self.assertTrue(dirty["dirty"])
            self.assertEqual(clean["tree"], dirty["tree"])
            self.assertNotEqual(clean["dirty_digest"], dirty["dirty_digest"])
            dirty_key, _ = build_source_identity(
                dirty,
                environment_sha256="a" * 64,
                profile=profile,
                toolchain=toolchain,
                isolation=isolation,
            )
            self.assertNotEqual(first_key, dirty_key)

            (root / "untracked-evidence.txt").write_text(
                "untracked bytes\n", encoding="utf-8"
            )
            with_untracked = capture_source_state(root)
            self.assertNotEqual(
                dirty["dirty_digest"], with_untracked["dirty_digest"]
            )

    def test_git_identity_ignores_ambient_redirection_and_fsmonitor(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, _, _ = self.make_repository(temporary)
            foreign = temporary / "foreign"
            foreign.mkdir()
            subprocess.run(["git", "init", "-q"], cwd=foreign, check=True)
            subprocess.run(
                ["git", "config", "user.email", "ai-test@example.invalid"],
                cwd=foreign,
                check=True,
            )
            subprocess.run(
                ["git", "config", "user.name", "AI Test"],
                cwd=foreign,
                check=True,
            )
            (foreign / "foreign").write_text(
                "foreign\n", encoding="utf-8"
            )
            subprocess.run(["git", "add", "."], cwd=foreign, check=True)
            subprocess.run(
                ["git", "commit", "-qm", "foreign"],
                cwd=foreign,
                check=True,
            )
            marker = temporary / "fsmonitor-ran"
            monitor = temporary / "fsmonitor.sh"
            monitor.write_text(
                f"#!/bin/sh\n/usr/bin/touch {marker}\n",
                encoding="utf-8",
            )
            monitor.chmod(0o700)
            subprocess.run(
                ["git", "config", "core.fsmonitor", str(monitor)],
                cwd=root,
                check=True,
            )
            expected_head = subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=root,
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            with mock.patch.dict(
                os.environ,
                {
                    "GIT_DIR": str(foreign / ".git"),
                    "GIT_WORK_TREE": str(foreign),
                    "GIT_CONFIG_COUNT": "1",
                    "GIT_CONFIG_KEY_0": "core.fsmonitor",
                    "GIT_CONFIG_VALUE_0": str(monitor),
                },
                clear=False,
            ):
                observed = capture_source_state(root)
            self.assertEqual(observed["candidate_root"], str(root))
            self.assertEqual(observed["head_revision"], expected_head)
            self.assertFalse(marker.exists())

    def test_git_identity_ignores_replace_refs(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, _, _ = self.make_repository(temporary)
            original_head = subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=root,
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            original_tree = subprocess.run(
                ["git", "rev-parse", "HEAD^{tree}"],
                cwd=root,
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            source = root / "crates/example/src/lib.rs"
            source.write_text(
                "pub fn replacement_value() -> u8 { 99 }\n",
                encoding="utf-8",
            )
            subprocess.run(["git", "add", "."], cwd=root, check=True)
            subprocess.run(
                ["git", "commit", "-qm", "replacement tree"],
                cwd=root,
                check=True,
            )
            replacement_commit = subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=root,
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            replacement_tree = subprocess.run(
                ["git", "rev-parse", "HEAD^{tree}"],
                cwd=root,
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            self.assertNotEqual(replacement_tree, original_tree)
            subprocess.run(
                ["git", "reset", "--hard", "-q", original_head],
                cwd=root,
                check=True,
            )
            subprocess.run(
                ["git", "replace", original_head, replacement_commit],
                cwd=root,
                check=True,
            )
            self.assertEqual(
                subprocess.run(
                    ["git", "rev-parse", "HEAD^{tree}"],
                    cwd=root,
                    check=True,
                    capture_output=True,
                    text=True,
                ).stdout.strip(),
                replacement_tree,
            )

            observed = capture_source_state(root)

            self.assertEqual(observed["head_revision"], original_head)
            self.assertEqual(observed["tree"], original_tree)
            self.assertFalse(observed["dirty"])

    def test_test_analyzer_rejects_fake_pass_followed_by_zero_harness(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            target = Path(temporary_raw).resolve() / "target"
            target.mkdir()
            analyzer = CargoTestLogAnalyzer(target, required=True)
            analyzer.feed(
                (
                    f"     Running unittests src/lib.rs "
                    f"({target}/debug/deps/first-a1)\n"
                    "running 1 test\n"
                    "test example::real ... ok\n"
                    "test result: ok. 1 passed; 0 failed; 0 ignored; "
                    "0 measured; 0 filtered out\n"
                    f"     Running unittests src/main.rs "
                    f"({target}/debug/deps/second-b2)\n"
                    "running 0 tests\n"
                    "test result: ok. 0 passed; 0 failed; 0 ignored; "
                    "0 measured; 794 filtered out\n"
                ).encode()
            )
            result = analyzer.finish()
            self.assertEqual(result["decision"], "BLOCK")
            self.assertEqual(result["summary_count"], 2)
            self.assertIn(
                "zero-selected-target-harness", result["parse_errors"]
            )

    def test_test_analyzer_state_and_result_remain_bounded(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            target = Path(temporary_raw).resolve() / "target"
            target.mkdir()
            malformed = CargoTestLogAnalyzer(target, required=True)
            malformed.feed(b"running 1 test\n" * 10_000)
            malformed_result = malformed.finish()
            self.assertEqual(
                malformed_result["parse_errors"],
                ["running-count-without-target-harness"],
            )
            self.assertEqual(len(malformed.errors), 1)
            self.assertEqual(malformed_result["harnesses"], [])

            numeric = CargoTestLogAnalyzer(target, required=True)
            numeric.feed(
                (
                    "     Running unittests src/lib.rs "
                    f"({target}/debug/deps/numeric)\n"
                    f"running {'9' * 10_000} tests\n"
                ).encode()
            )
            numeric_result = numeric.finish()
            self.assertEqual(numeric_result["decision"], "BLOCK")
            self.assertIn(
                "test-harness-count-exceeds-bound",
                numeric_result["parse_errors"],
            )

            analyzer = CargoTestLogAnalyzer(target, required=True)
            for index in range(MAX_TEST_HARNESS_SUMMARIES + 10):
                analyzer.feed(
                    (
                        "     Running unittests src/lib.rs "
                        f"({target}/debug/deps/example-{index})\n"
                        "running 1 test\n"
                        "test result: ok. 1 passed; 0 failed; 0 ignored; "
                        "0 measured; 0 filtered out\n"
                    ).encode()
                )
            result = analyzer.finish()
            self.assertEqual(result["decision"], "BLOCK")
            self.assertEqual(
                result["summary_count"],
                MAX_TEST_HARNESS_SUMMARIES + 10,
            )
            self.assertEqual(
                result["harnesses_retained"],
                MAX_RETAINED_TEST_HARNESSES,
            )
            self.assertEqual(
                len(result["harnesses"]),
                MAX_RETAINED_TEST_HARNESSES,
            )
            self.assertIn(
                "test-harness-summary-count-exceeds-bound",
                result["parse_errors"],
            )
            self.assertLess(
                len(rendered_result({"test_execution": result})),
                1024 * 1024,
            )

    def test_rendered_gate_result_has_a_hard_size_limit(self) -> None:
        with self.assertRaises(GateBlocked) as raised:
            rendered_result({"unbounded": "x" * (1024 * 1024)})
        self.assertEqual(
            raised.exception.code, "result-payload-exceeds-bound"
        )

    def test_python310_toml_fallback_rejects_ambiguous_quoted_keys(
        self,
    ) -> None:
        with mock.patch(
            "run_source_keyed_cargo_gate.tomllib", None
        ):
            self.assertTrue(
                toml_has_disabled_harness(
                    b'[[test]]\n"harness" = false\n'
                )
            )
            with self.assertRaisesRegex(
                ValueError, "escaped TOML strings require tomllib"
            ):
                toml_has_disabled_harness(
                    b'[[test]]\n"harn\\u0065ss" = false\n'
                )
            with self.assertRaisesRegex(
                ValueError, "multiline TOML strings require tomllib"
            ):
                toml_has_disabled_harness(
                    b'description = """ambiguous"""\n'
                )

    def test_artifact_key_ignores_commit_identity_but_binds_tree_and_root(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, _ = self.make_repository(temporary)
            toolchain = probe_toolchain(profile, "test")
            isolation = isolation_identity()
            initial = capture_source_state(root)
            initial_key, _ = build_artifact_identity(
                initial,
                environment_sha256="e" * 64,
                profile=profile,
                toolchain=toolchain,
                isolation=isolation,
            )
            subprocess.run(
                ["git", "commit", "--allow-empty", "-qm", "same tree"],
                cwd=root,
                check=True,
            )
            same_tree_state = capture_source_state(root)
            same_tree_key, _ = build_artifact_identity(
                same_tree_state,
                environment_sha256="e" * 64,
                profile=profile,
                toolchain=toolchain,
                isolation=isolation,
            )
            self.assertNotEqual(
                initial["head_revision"],
                same_tree_state["head_revision"],
            )
            self.assertEqual(initial["tree"], same_tree_state["tree"])
            self.assertEqual(initial_key, same_tree_key)

            source = root / "crates/example/src/lib.rs"
            source.write_text(
                "pub fn value() -> u8 { 5 }\n", encoding="utf-8"
            )
            subprocess.run(["git", "add", "."], cwd=root, check=True)
            subprocess.run(
                ["git", "commit", "-qm", "rust change"],
                cwd=root,
                check=True,
            )
            rust_state = capture_source_state(root)
            rust_key, _ = build_artifact_identity(
                rust_state,
                environment_sha256="e" * 64,
                profile=profile,
                toolchain=toolchain,
                isolation=isolation,
            )
            self.assertNotEqual(initial_key, rust_key)
            moved = dict(initial)
            moved["candidate_root"] = str(temporary / "other")
            moved_key, _ = build_artifact_identity(
                moved,
                environment_sha256="e" * 64,
                profile=profile,
                toolchain=toolchain,
                isolation=isolation,
            )
            self.assertNotEqual(initial_key, moved_key)

    def test_focused_tests_and_side_effecting_cargo_argv_fail_closed(
        self,
    ) -> None:
        profile = dict(DEFAULT_PROFILE)
        with self.assertRaises(GateBlocked) as focused:
            cargo_test_contract(
                ["cargo", "test", "--locked", "specific_filter"]
            )
        self.assertEqual(
            focused.exception.code,
            "focused-test-requires-explicit-harness",
        )
        with self.assertRaises(GateBlocked) as post_separator:
            cargo_test_contract(
                ["cargo", "test", "--locked", "--", "specific_filter"]
            )
        self.assertEqual(
            post_separator.exception.code,
            "focused-test-requires-explicit-harness",
        )
        cases = (
            (["cargo", "fmt"], "cargo-fmt-check-required"),
            (
                ["cargo", "test", "--locked", "--no-run"],
                "cargo-test-no-run-not-admitted",
            ),
            (
                ["cargo", "clippy", "--locked", "--fix"],
                "side-effecting-clippy-argv",
            ),
            (
                ["cargo", "doc", "--locked", "--open"],
                "side-effecting-cargo-doc-argv",
            ),
        )
        for command, code in cases:
            with self.subTest(command=command):
                with self.assertRaises(GateBlocked) as raised:
                    normalize_cargo_argv(command, profile=profile)
                self.assertEqual(raised.exception.code, code)
        normalized, subcommand = normalize_cargo_argv(
            ["cargo", "fmt", "--", "--check"], profile=profile
        )
        self.assertEqual(normalized, ["cargo", "fmt", "--", "--check"])
        self.assertEqual(subcommand, "fmt")

    def test_custom_test_harness_is_not_trusted_as_libtest_evidence(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as raw:
            temporary = Path(raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            manifest = root / "crates/example/Cargo.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8")
                + (
                    "\n[[test]]\n"
                    'name = "custom"\n'
                    'path = "tests/custom.rs"\n'
                    "harness = false\n"
                ),
                encoding="utf-8",
            )
            custom = root / "crates/example/tests/custom.rs"
            custom.parent.mkdir()
            custom.write_text("fn main() {}\n", encoding="utf-8")
            subprocess.run(["git", "add", "."], cwd=root, check=True)
            subprocess.run(
                ["git", "commit", "-qm", "custom harness"],
                cwd=root,
                check=True,
            )
            from run_source_keyed_cargo_gate import (
                custom_harness_contract as audit_snapshot,
            )

            original_manifest = manifest.read_bytes()

            def hide_live_custom_harness(snapshot, snapshot_identity):
                manifest.write_bytes(
                    original_manifest.replace(
                        b"harness = false", b"harness = true "
                    )
                )
                try:
                    return audit_snapshot(snapshot, snapshot_identity)
                finally:
                    manifest.write_bytes(original_manifest)

            with mock.patch(
                "run_source_keyed_cargo_gate.custom_harness_contract",
                side_effect=hide_live_custom_harness,
            ), self.assertRaises(GateBlocked) as raised:
                run_source_keyed_gate(
                    [
                        "cargo",
                        "test",
                        "--test",
                        "custom",
                        "--locked",
                    ],
                    root,
                    temporary / "evidence/custom-harness.log",
                    30,
                    environment=environment,
                    profile=profile,
                    new_namespace_free_floor_bytes=0,
                    gate_runner=self.fake_runner(
                        (
                            b"running 1 test\n"
                            b"test result: ok. 1 passed; 0 failed; "
                            b"0 ignored; 0 measured; 0 filtered out\n"
                        )
                    ),
                )
            self.assertEqual(
                raised.exception.code,
                "cargo-custom-test-harness-not-admitted",
            )
            self.assertEqual(manifest.read_bytes(), original_manifest)
            registry = (
                Path(str(profile["target_dir"]))
                / SOURCE_KEY_TARGET_DIRECTORY
            )
            self.assertEqual(list(registry.iterdir()), [])

    def test_loader_and_python_startup_overrides_are_rejected(
        self,
    ) -> None:
        for name in ("LD_PRELOAD", "LD_LIBRARY_PATH", "PYTHONPATH"):
            with self.subTest(name=name), tempfile.TemporaryDirectory() as raw:
                temporary = Path(raw).resolve()
                root, profile, environment = self.make_repository(temporary)
                environment[name] = str(temporary / "untrusted")
                with self.assertRaises(GateBlocked) as raised:
                    run_source_keyed_gate(
                        ["cargo", "check", "--locked"],
                        root,
                        temporary / "evidence/rejected.log",
                        30,
                        environment=environment,
                        profile=profile,
                        new_namespace_free_floor_bytes=0,
                        gate_runner=self.fake_runner(b""),
                    )
                self.assertEqual(
                    raised.exception.code,
                    "ambient-tool-or-runtime-override",
                )

    def test_unadmitted_credentials_are_not_exposed_to_gate(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            temporary = Path(raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            environment.update(
                {
                    "AWS_ACCESS_KEY_ID": "not-for-builds",
                    "AWS_SECRET_ACCESS_KEY": "not-for-builds",
                    "GOOGLE_APPLICATION_CREDENTIALS": (
                        str(temporary / "credentials.json")
                    ),
                    "KUBECONFIG": str(temporary / "kubeconfig"),
                    "NETRC": str(temporary / "netrc"),
                }
            )
            capture: dict[str, object] = {}
            result = run_source_keyed_gate(
                ["cargo", "check", "--locked"],
                root,
                temporary / "evidence/credential-scrub.log",
                30,
                environment=environment,
                profile=profile,
                ephemeral=True,
                new_namespace_free_floor_bytes=0,
                gate_runner=self.fake_runner(b"", capture),
            )
            self.assertEqual(result["decision"], "PASS")
            observed = capture["environment"]
            self.assertIsInstance(observed, dict)
            for name in (
                "AWS_ACCESS_KEY_ID",
                "AWS_SECRET_ACCESS_KEY",
                "GOOGLE_APPLICATION_CREDENTIALS",
                "KUBECONFIG",
                "NETRC",
            ):
                self.assertNotIn(name, observed)

    def test_admitted_target_root_must_be_owner_controlled(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            temporary = Path(raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            Path(str(profile["target_dir"])).chmod(0o777)
            with self.assertRaises(GateBlocked) as raised:
                run_source_keyed_gate(
                    ["cargo", "check", "--locked"],
                    root,
                    temporary / "evidence/unsafe-target.log",
                    30,
                    environment=environment,
                    profile=profile,
                    new_namespace_free_floor_bytes=0,
                    gate_runner=self.fake_runner(b""),
                )
            self.assertEqual(
                raised.exception.code, "unsafe-admitted-target-root"
            )

    def test_stale_snapshots_are_recovered_before_materialization(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as raw:
            temporary = Path(raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            snapshot_root = Path(str(profile["snapshot_root"]))
            stale_names = (".tmp-stale", "ready-stale")
            for name in stale_names:
                stale = snapshot_root / name
                stale.mkdir()
                (stale / "partial").write_bytes(b"partial")
            result = run_source_keyed_gate(
                ["cargo", "check", "--locked"],
                root,
                temporary / "evidence/recovered-snapshot.log",
                30,
                environment=environment,
                profile=profile,
                ephemeral=True,
                new_namespace_free_floor_bytes=0,
                gate_runner=self.fake_runner(b""),
            )
            self.assertEqual(result["decision"], "PASS")
            recovery = result["source_snapshot"]["root_recovery"]
            self.assertEqual(recovery["removed_count"], 2)
            for name in stale_names:
                self.assertFalse((snapshot_root / name).exists())

    def test_cleanup_measures_every_entry_over_the_premeasure_cap(
        self,
    ) -> None:
        # Two counterexamples in one namespace.
        #
        # First: an entry cap on destructive cleanup is a permanent wedge.
        # A crash-left namespace larger than the cap fails every cleanup
        # attempt identically, so it stays addressable forever, keeps
        # consuming the 32 GiB aggregate budget, and blocks every later
        # admission. Removal is progress, so only the wall deadline bounds
        # it.
        #
        # Second: the reclaimed figure must still cover every removed entry
        # when the pre-measure scan bound refuses the namespace. Dropping
        # to "unknown" there would leave the one number the retention
        # policy may not guess unmeasured. Note the deliberately weaker
        # word: covered, not exact. See the hard-link test below for what
        # this measurement does and does not claim.
        with tempfile.TemporaryDirectory() as raw:
            temporary = Path(raw).resolve()
            _, profile, _ = self.make_repository(temporary)
            admitted = Path(str(profile["target_dir"]))
            key = "1" * 64
            namespace, expected_marker, _, _, _ = prepare_namespace(
                admitted,
                key,
                {
                    "schema": "codefire.cargo_source_key.v1",
                    "candidate": {"tree": "over-cap"},
                },
                lock_timeout_seconds=0,
                new_namespace_free_floor_bytes=0,
            )
            nested = namespace / "debug/deps"
            nested.mkdir(parents=True)
            for index in range(12):
                (nested / f"artifact-{index}.rlib").write_bytes(
                    b"x" * 4096
                )
            expected = namespace_usage(namespace)
            self.assertGreater(expected["allocated_bytes"], 0)

            with mock.patch(
                "run_source_keyed_cargo_gate.MAX_NAMESPACE_ENTRY_COUNT",
                4,
            ):
                # The pre-measure scan refuses this namespace outright.
                with self.assertRaises(GateBlocked) as refused:
                    namespace_usage(namespace)
                self.assertEqual(
                    refused.exception.code,
                    "namespace-entry-count-limit-exceeded",
                )
                cleanup = cleanup_ephemeral_namespace(
                    admitted,
                    namespace,
                    expected_marker,
                    lock_timeout_seconds=0,
                )

            self.assertFalse(namespace.exists())
            self.assertEqual(cleanup["decision"], "EPHEMERAL_CLEANED")
            measurement = cleanup["reclaimed_measurement"]
            self.assertEqual(
                measurement["source"], "descriptor-relative-removal"
            )
            self.assertTrue(measurement["covers_every_removed_entry"])
            self.assertIsNone(cleanup["premeasured_allocated_bytes"])
            self.assertEqual(
                cleanup["removed_hardlinked_entry_count"], 0
            )
            self.assertEqual(
                cleanup["reclaimed_allocated_bytes"],
                expected["allocated_bytes"],
            )
            self.assertEqual(
                cleanup["reclaimed_logical_bytes"],
                expected["logical_bytes"],
            )
            self.assertEqual(
                cleanup["removed_entry_count"], expected["entry_count"]
            )
            self.assertTrue(cleanup["marker_removed_last"])
            self.assertEqual(
                [
                    name
                    for name in os.listdir(namespace.parent)
                    if name.startswith(".quarantine-")
                ],
                [],
            )

    def test_hardlinked_reclaim_is_an_upper_bound_and_says_so(
        self,
    ) -> None:
        # Counterexample to calling the reclaimed figure "exact" or saying
        # it is "charged back against the aggregate budget".
        #
        # Cargo's uplift gives every produced artifact a second link, so
        # the per-entry st_blocks convention counts that inode twice and
        # the reported figure is roughly double the allocation the removal
        # actually returns. That is the safe direction -- the measurement
        # and aggregate walks share the convention, so the budget stays
        # internally consistent and blocks earlier -- but it is an upper
        # bound, and the payload has to say so rather than let a consumer
        # read exactness into it. Nothing charges the figure back either:
        # the aggregate is always re-measured under the registry lock.
        with tempfile.TemporaryDirectory() as raw:
            temporary = Path(raw).resolve()
            _, profile, _ = self.make_repository(temporary)
            admitted = Path(str(profile["target_dir"]))
            key = "2" * 64
            namespace, expected_marker, _, _, _ = prepare_namespace(
                admitted,
                key,
                {
                    "schema": "codefire.cargo_source_key.v1",
                    "candidate": {"tree": "uplifted"},
                },
                lock_timeout_seconds=0,
                new_namespace_free_floor_bytes=0,
            )
            deps = namespace / "debug/deps"
            deps.mkdir(parents=True)
            payload = deps / "codefire_rs-abcd1234"
            payload.write_bytes(b"x" * (1024 * 1024))
            uplifted = namespace / "debug/codefire-rs"
            os.link(payload, uplifted)
            self.assertEqual(payload.stat().st_nlink, 2)

            deduplicated = int(
                subprocess.run(
                    ["du", "-s", "--block-size=1", str(namespace)],
                    capture_output=True,
                    text=True,
                    check=True,
                ).stdout.split()[0]
            )
            scanned = namespace_usage(namespace)
            cleanup = cleanup_ephemeral_namespace(
                admitted,
                namespace,
                expected_marker,
                lock_timeout_seconds=0,
            )

            # The removal walk and the aggregate scan agree with each
            # other, which is what keeps the budget self-consistent.
            self.assertEqual(
                cleanup["reclaimed_allocated_bytes"],
                scanned["allocated_bytes"],
            )
            # Both exceed the inode-deduplicated truth by one link, and
            # the payload quantifies that overstatement exactly rather
            # than leaving it to be inferred.
            self.assertGreater(
                cleanup["reclaimed_allocated_bytes"], deduplicated
            )
            self.assertEqual(
                cleanup["removed_hardlinked_entry_count"], 2
            )
            self.assertEqual(
                cleanup["reclaimed_allocated_bytes"]
                - cleanup[
                    "reclaimed_allocated_bytes_counted_more_than_once"
                ],
                deduplicated,
            )
            # The payload must state the convention, not imply exactness.
            measurement = cleanup["reclaimed_measurement"]
            self.assertEqual(
                measurement["convention"],
                "per-directory-entry-st-blocks-no-inode-dedup",
            )
            self.assertTrue(
                measurement["hardlinked_inodes_counted_once_per_link"]
            )
            self.assertTrue(
                measurement["is_upper_bound_on_freed_allocation"]
            )
            self.assertFalse(
                measurement["charged_back_against_aggregate_budget"]
            )
            self.assertNotIn("resource_accounting_complete", cleanup)

    def test_hardlinked_marker_overcount_is_reported(self) -> None:
        # The marker is skipped by the scandir walk and removed last, in a
        # branch that read st_nlink directly. By then the walk has already
        # unlinked the sibling link, so st_nlink has fallen back to 1: the
        # marker registered as un-linked, its allocation was still counted
        # twice by the per-entry convention, and the overcount field said
        # zero. That is a second wholly-inside-the-namespace case the
        # field's "exact" claim did not cover.
        #
        # Reachability: cleanup_namespace_locked normally validates the
        # marker first, and validate_marker_at rejects st_nlink != 1, so
        # the primary path is already fail-closed. The reachable surface
        # is recover_incomplete_namespaces, which passes expected_marker
        # =None precisely when the marker could not be read -- which is
        # what a hard-linked marker is.
        #
        # The assertion is the arithmetic property, not a symbol: on
        # 4e5c0e7 `walk - dup` overstates `du` by exactly one marker.
        with tempfile.TemporaryDirectory() as raw:
            temporary = Path(raw).resolve()
            _, profile, _ = self.make_repository(temporary)
            admitted = Path(str(profile["target_dir"]))
            namespace, _, _, _, _ = prepare_namespace(
                admitted,
                "3" * 64,
                {
                    "schema": "codefire.cargo_source_key.v1",
                    "candidate": {"tree": "marker-linked"},
                },
                lock_timeout_seconds=0,
                new_namespace_free_floor_bytes=0,
            )
            marker = namespace / MARKER_NAME
            os.link(marker, namespace / "marker-alias")
            self.assertEqual(marker.stat().st_nlink, 2)

            deduplicated = int(
                subprocess.run(
                    ["du", "-s", "--block-size=1", str(namespace)],
                    capture_output=True,
                    text=True,
                    check=True,
                ).stdout.split()[0]
            )
            scanned = namespace_usage(namespace)
            with exclusive_lock(admitted / REGISTRY_LOCK_NAME, 0):
                cleanup = cleanup_namespace_locked(
                    admitted, namespace, None, measure_before=False
                )

            self.assertFalse(namespace.exists())
            self.assertEqual(
                cleanup["reclaimed_allocated_bytes"],
                scanned["allocated_bytes"],
            )
            self.assertGreater(
                cleanup["reclaimed_allocated_bytes"], deduplicated
            )
            # Both links counted, and the overcount recovers du exactly.
            self.assertEqual(
                cleanup["removed_hardlinked_entry_count"], 2
            )
            self.assertEqual(
                cleanup["reclaimed_allocated_bytes"]
                - cleanup[
                    "reclaimed_allocated_bytes_counted_more_than_once"
                ],
                deduplicated,
            )

    def test_forward_guard_no_consumer_charges_reclaimed_bytes_back(
        self,
    ) -> None:
        """Forward guard, not a regression control.

        This passes unchanged on the predecessor `aaa39b6`, because no
        charge-back consumer existed there either. The prior `MUST-3(a)`
        defect was the *documentation* claiming one, and the control that
        actually fails on the predecessor is
        `test_reclaim_is_not_described_as_exact_or_charged_back`. Filing
        this as evidence that a predecessor regression was closed would
        overstate it.

        What it is good for: if a future change starts feeding the figure
        into an admission decision, the doc and payload wording must
        change with it, and this fails first.

        Scope limit, stated rather than implied: four literal read
        patterns over `docs/development/tools/*.py`. A read through an
        intermediate variable, or from outside that directory, is not
        seen. Immaterial today — the only Python outside it is `setup.py`
        and the disjoint `tests/` suite, neither of which touches gate
        result JSON.
        """
        tools = Path(__file__).resolve().parent
        # A read of the figure, as opposed to a site that produces it.
        reads = (
            '["reclaimed_allocated_bytes"]',
            "get(\"reclaimed_allocated_bytes\"",
            "['reclaimed_allocated_bytes']",
            "get('reclaimed_allocated_bytes'",
        )
        offenders = []
        for path in sorted(tools.glob("*.py")):
            if path.name == "test_ai_tools.py":
                continue
            for number, line in enumerate(
                path.read_text(encoding="utf-8").splitlines(), 1
            ):
                if any(read in line for read in reads):
                    offenders.append(
                        f"{path.name}:{number}: {line.strip()}"
                    )
        self.assertEqual(offenders, [])

    def test_incomplete_namespace_for_other_key_is_recovered(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as raw:
            temporary = Path(raw).resolve()
            _, profile, _ = self.make_repository(temporary)
            admitted = Path(str(profile["target_dir"]))
            stale_key = "a" * 64
            fresh_key = "b" * 64
            stale, _, _, _, _ = prepare_namespace(
                admitted,
                stale_key,
                {
                    "schema": "codefire.cargo_source_key.v1",
                    "candidate": {"tree": "stale"},
                },
                lock_timeout_seconds=0,
                new_namespace_free_floor_bytes=0,
            )
            (stale / RUN_STATE_NAME).unlink()
            fresh, _, _, _, admission = prepare_namespace(
                admitted,
                fresh_key,
                {
                    "schema": "codefire.cargo_source_key.v1",
                    "candidate": {"tree": "fresh"},
                },
                lock_timeout_seconds=0,
                new_namespace_free_floor_bytes=0,
            )
            self.assertFalse(stale.exists())
            self.assertTrue(fresh.exists())
            self.assertEqual(
                admission["incomplete_namespace_recovery"][
                    "removed_count"
                ],
                1,
            )
            self.assertEqual(
                read_run_state(fresh, source_key=fresh_key)["state"],
                "PREPARED",
            )

    def test_incomplete_namespace_over_entry_cap_is_recovered(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as raw:
            temporary = Path(raw).resolve()
            _, profile, _ = self.make_repository(temporary)
            admitted = Path(str(profile["target_dir"]))
            stale_key = "c" * 64
            fresh_key = "d" * 64
            stale, _, _, _, _ = prepare_namespace(
                admitted,
                stale_key,
                {
                    "schema": "codefire.cargo_source_key.v1",
                    "candidate": {"tree": "stale-over-cap"},
                },
                lock_timeout_seconds=0,
                new_namespace_free_floor_bytes=0,
            )
            (stale / RUN_STATE_NAME).unlink()
            for index in range(8):
                (stale / f"runaway-{index}").write_bytes(b"x")
            with mock.patch(
                "run_source_keyed_cargo_gate.MAX_NAMESPACE_ENTRY_COUNT",
                4,
            ):
                fresh, _, _, _, admission = prepare_namespace(
                    admitted,
                    fresh_key,
                    {
                        "schema": "codefire.cargo_source_key.v1",
                        "candidate": {"tree": "fresh-after-cap"},
                    },
                    lock_timeout_seconds=0,
                    new_namespace_free_floor_bytes=0,
                )
            self.assertFalse(stale.exists())
            self.assertTrue(fresh.exists())
            self.assertEqual(
                admission["incomplete_namespace_recovery"][
                    "removed_count"
                ],
                1,
            )

    def test_incomplete_namespace_recovery_ignores_corrupt_child_metadata(
        self,
    ) -> None:
        for corrupt_name in (
            ".codefire-source-key-retention.json",
            ".codefire-source-key-invalid.json",
        ):
            with self.subTest(corrupt_name=corrupt_name), tempfile.TemporaryDirectory() as raw:
                temporary = Path(raw).resolve()
                _, profile, _ = self.make_repository(temporary)
                admitted = Path(str(profile["target_dir"]))
                stale_key = "e" * 64
                fresh_key = "f" * 64
                stale, _, _, _, _ = prepare_namespace(
                    admitted,
                    stale_key,
                    {
                        "schema": "codefire.cargo_source_key.v1",
                        "candidate": {"tree": "corrupt-child-metadata"},
                    },
                    lock_timeout_seconds=0,
                    new_namespace_free_floor_bytes=0,
                )
                write_run_state(
                    stale,
                    source_key=stale_key,
                    state="RUNNING",
                    execution_key_value="1" * 64,
                )
                (stale / corrupt_name).write_bytes(b"not-json")
                fresh, _, _, _, admission = prepare_namespace(
                    admitted,
                    fresh_key,
                    {
                        "schema": "codefire.cargo_source_key.v1",
                        "candidate": {"tree": "fresh"},
                    },
                    lock_timeout_seconds=0,
                    new_namespace_free_floor_bytes=0,
                )
                self.assertFalse(stale.exists())
                self.assertTrue(fresh.exists())
                self.assertEqual(
                    admission["incomplete_namespace_recovery"][
                        "removed_count"
                    ],
                    1,
                )

    def test_interrupted_prepare_directory_is_recovered(self) -> None:
        with tempfile.TemporaryDirectory() as raw:
            temporary = Path(raw).resolve()
            _, profile, _ = self.make_repository(temporary)
            admitted = Path(str(profile["target_dir"]))
            registry = admitted / SOURCE_KEY_TARGET_DIRECTORY
            registry.mkdir()
            partial = registry / f".prepare-{'a' * 64}-stale"
            partial.mkdir()
            (partial / "partial-marker").write_bytes(b"partial")
            prepared, _, _, _, admission = prepare_namespace(
                admitted,
                "b" * 64,
                {
                    "schema": "codefire.cargo_source_key.v1",
                    "candidate": {"tree": "fresh"},
                },
                lock_timeout_seconds=0,
                new_namespace_free_floor_bytes=0,
            )
            self.assertFalse(partial.exists())
            self.assertTrue(prepared.exists())
            self.assertEqual(
                admission["registry_quarantine_recovery"][
                    "removed_count"
                ],
                1,
            )

    def test_clean_snapshot_is_exact_separate_inode_and_durably_removed(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, _ = self.make_repository(temporary)
            before = capture_source_state(root)
            source = root / "crates/example/src/lib.rs"
            original = source.read_bytes()
            snapshot, identity = create_clean_git_snapshot(
                root,
                before,
                "a" * 64,
                profile,
                free_floor_bytes=0,
            )
            snap_source = snapshot / "crates/example/src/lib.rs"
            self.assertEqual(snap_source.read_bytes(), original)
            self.assertNotEqual(
                source.stat().st_ino,
                snap_source.stat().st_ino,
            )
            source.write_text(
                "pub fn value() -> u8 { 99 }\n", encoding="utf-8"
            )
            self.assertEqual(snap_source.read_bytes(), original)
            cleanup = cleanup_snapshot(snapshot, identity)
            self.assertEqual(cleanup["decision"], "SNAPSHOT_CLEANED")
            self.assertFalse(snapshot.exists())
            self.assertFalse(cleanup["snapshot_exists_after"])
            self.assertFalse(cleanup["quarantine_exists_after"])

    def test_tracked_symlink_and_hardlinked_rust_input_are_rejected(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, _, _ = self.make_repository(temporary)
            symlink = root / "crates/example/src/linked.rs"
            symlink.symlink_to("lib.rs")
            subprocess.run(["git", "add", "."], cwd=root, check=True)
            subprocess.run(
                ["git", "commit", "-qm", "tracked symlink"],
                cwd=root,
                check=True,
            )
            head = subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=root,
                capture_output=True,
                text=True,
                check=True,
            ).stdout.strip()
            with self.assertRaises(GateBlocked) as tracked:
                git_tree_entries(root, head)
            self.assertEqual(
                tracked.exception.code, "unsupported-git-tree-entry"
            )

        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, _, _ = self.make_repository(temporary)
            source = root / "crates/example/src/lib.rs"
            os.link(source, root / "crates/example/src/hardlinked.rs")
            with self.assertRaisesRegex(ValueError, "hard-linked"):
                build_source_manifest(root)

    def test_pinned_tool_path_replacement_is_detected_after_execution(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            _, profile, _ = self.make_repository(temporary)
            toolchain = probe_toolchain(profile, "test")
            cargo_path = Path(toolchain["executables"]["cargo"]["path"])
            replacement = cargo_path.with_name("cargo.replacement")
            with self.assertRaises(GateBlocked) as raised:
                with pinned_tool_files(
                    toolchain, isolation_identity()
                ):
                    cargo_path.rename(cargo_path.with_name("cargo.original"))
                    replacement.write_text(
                        "#!/bin/sh\nprintf 'replacement\\n'\n",
                        encoding="utf-8",
                    )
                    replacement.chmod(0o700)
                    replacement.rename(cargo_path)
            self.assertIn(
                raised.exception.code,
                {
                    "pinned-tool-path-changed-during-gate",
                    "pinned-tool-changed-during-gate",
                },
            )

    def test_gate_uses_locked_source_namespace_and_source_keyed_log(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            capture: dict[str, object] = {}
            result = run_source_keyed_gate(
                ["cargo", "test", "--bin", "example", "--locked", "focused_filter"],
                root,
                temporary / "evidence/gate.log",
                30,
                environment=environment,
                profile=profile,
                new_namespace_free_floor_bytes=0,
                gate_runner=self.fake_runner(
                    (
                        b"running 1 test\n"
                        b"test example::focused_filter ... ok\n\n"
                        b"test result: ok. 1 passed; 0 failed; 0 ignored; "
                        b"0 measured; 7 filtered out\n"
                    ),
                    capture,
                ),
            )
            self.assertEqual(result["decision"], "PASS")
            self.assertEqual(result["test_execution"]["executed_tests"], 1)
            target = Path(result["target"]["namespace"])
            self.assertEqual(target.parent.name, SOURCE_KEY_TARGET_DIRECTORY)
            self.assertEqual(target.name, result["artifact_key"])
            self.assertEqual(
                result["target"]["artifact_key"], result["artifact_key"]
            )
            self.assertTrue((target / ".codefire-source-key.json").is_file())
            environment_used = capture["environment"]
            self.assertIsInstance(environment_used, dict)
            self.assertEqual(
                environment_used["CARGO_TARGET_DIR"],  # type: ignore[index]
                str(target),
            )
            for name in ("CARGO", "RUSTC", "RUSTDOC"):
                descriptor_path = environment_used[name]  # type: ignore[index]
                self.assertIsInstance(descriptor_path, str)
                self.assertTrue(
                    descriptor_path.startswith("/proc/self/fd/")
                )
                self.assertIn(
                    int(descriptor_path.rsplit("/", 1)[1]),
                    capture["pass_fds"],  # type: ignore[operator]
                )
                projected = result["runtime_environment_contract"][
                    "entries"
                ][name]
                self.assertEqual(
                    projected["kind"], "pinned-tool-fd"
                )
            self.assertEqual(
                result["artifact_environment_sha256"],
                sha256_json(result["runtime_environment_contract"]),
            )
            self.assertEqual(
                result["runtime_environment_contract"]["entries"][
                    "HOME"
                ]["kind"],
                "derived-runtime-scratch",
            )
            runtime_scratch = result["runtime_scratch"]
            self.assertEqual(
                environment_used["HOME"],  # type: ignore[index]
                runtime_scratch["admission"]["home"]["path"],
            )
            self.assertEqual(
                environment_used["TMPDIR"],  # type: ignore[index]
                runtime_scratch["admission"]["tmp"]["path"],
            )
            self.assertEqual(
                runtime_scratch["cleanup"]["decision"], "REMOVED"
            )
            self.assertFalse(
                Path(runtime_scratch["admission"]["root"]).exists()
            )
            prefix = capture["log_prefix"]
            self.assertIsInstance(prefix, bytes)
            self.assertIn(result["source_key"].encode("ascii"), prefix)
            self.assertTrue(
                result["log_binding"]["source_key_present_in_prefix"]
            )
            self.assertTrue(
                result["log_binding"]["timed_gate_evidence_matches"]
            )
            self.assertIn(
                "global_execution_lock_wait_seconds", result["target"]
            )
            launch = capture["argv"]
            self.assertIsInstance(launch, list)
            # The parent-death signal is armed by the pinned pdeath helper
            # before setpriv, not by setpriv alone: setpriv arms it after
            # the fork, so a manager that dies in that window would leave an
            # unsupervised Cargo namespace. The helper re-reads getppid
            # after the prctl call and refuses to exec if the parent is
            # already gone.
            self.assertEqual(
                launch[1:6],  # type: ignore[index]
                ["-I", "-S", "-B", "-X", "pycache_prefix=/dev/null"],
            )
            self.assertEqual(
                launch[7:9],  # type: ignore[index]
                ["--parent-pid", str(os.getpid())],
            )
            self.assertEqual(launch[9], "--")  # type: ignore[index]
            self.assertEqual(
                launch[11:13],  # type: ignore[index]
                ["--pdeathsig", "SIGKILL"],
            )
            # The recorded isolation identity is embedded verbatim in both
            # keys and in every published result, so it has to describe the
            # launch that actually happened. Assert the equality rather
            # than restating the flag list a third time: a recorded copy
            # that drifts behind the launched argv answers "was propagation
            # pinned?" with the pre-fix answer.
            self.assertEqual(
                launch[14 : 14 + len(UNSHARE_ISOLATION_FLAGS)],
                list(UNSHARE_ISOLATION_FLAGS),
            )
            self.assertEqual(
                result["readonly_isolation_identity"]["unshare_flags"],
                list(UNSHARE_ISOLATION_FLAGS),
            )
            # Counterexample: the private source tmpfs is mounted over the
            # candidate worktree's own path. If the namespace propagated
            # shared, that mount would escape to the host and mask the
            # developer's real worktree for the whole gate. util-linux
            # defaults to private, but the default is not evidence.
            self.assertIn("--propagation", UNSHARE_ISOLATION_FLAGS)
            self.assertEqual(
                UNSHARE_ISOLATION_FLAGS[
                    UNSHARE_ISOLATION_FLAGS.index("--propagation") + 1
                ],
                "private",
            )
            self.assertEqual(
                launch[14 + len(UNSHARE_ISOLATION_FLAGS)],  # type: ignore[index]
                "--",
            )
            # Every executable in the chain is dispatched by inherited
            # descriptor, never by a resolvable host path.
            for index in (0, 6, 10, 13):
                self.assertTrue(
                    launch[index].startswith(  # type: ignore[index]
                        "/proc/self/fd/"
                    ),
                    launch[index],  # type: ignore[index]
                )
                self.assertIn(
                    int(launch[index].rsplit("/", 1)[1]),  # type: ignore[index]
                    capture["pass_fds"],  # type: ignore[operator]
                )
            self.assertEqual(
                result["execution_profile"]["rustup_toolchain"],
                profile["rustup_toolchain"],
            )
            self.assertEqual(result["retention"]["decision"], "PERSISTED")
            self.assertEqual(
                result["retention"][
                    "persistent_aggregate_allocated_bytes_before"
                ],
                0,
            )
            self.assertEqual(
                result["retention"]["persistent_aggregate_budget_bytes"],
                32 * 1024**3,
            )
            self.assertEqual(
                result["retention"]["allocation_budget_basis"],
                {
                    "measured_max_namespace_allocated_bytes": 3_204_700_000,
                    "reference_candidate_count": 8,
                    "headroom_numerator": 5,
                    "headroom_denominator": 4,
                },
            )

    def test_release_artifact_is_exported_and_digest_bound_under_lock(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            output = root / "target/release/codefire-rs"
            output.parent.mkdir(parents=True)
            (output.parent / ".codefire-rs.tmp-stale").write_bytes(
                b"partial"
            )
            prior_residue = output.parent / ".codefire-rs.previous-stale"
            output.write_bytes(b"old")
            output.chmod(0o700)
            os.link(output, prior_residue)
            self.assertEqual(output.stat().st_nlink, 2)

            def runner(*args, **kwargs):
                target = Path(kwargs["environment"]["CARGO_TARGET_DIR"])
                artifact = target / "release/codefire-rs"
                artifact.parent.mkdir(parents=True, exist_ok=True)
                artifact.write_bytes(b"source-keyed-binary")
                artifact.chmod(0o700)
                return self.fake_runner(b"")(*args, **kwargs)

            result = run_source_keyed_gate(
                [
                    "cargo",
                    "build",
                    "--release",
                    "--locked",
                    "-p",
                    "example",
                ],
                root,
                temporary / "evidence/export.log",
                30,
                environment=environment,
                profile=profile,
                new_namespace_free_floor_bytes=0,
                export_artifact_relative="release/codefire-rs",
                export_path=output,
                gate_runner=runner,
            )
            self.assertEqual(result["decision"], "PASS")
            self.assertEqual(output.read_bytes(), b"source-keyed-binary")
            self.assertEqual(
                result["artifact_export"]["sha256"],
                hashlib.sha256(b"source-keyed-binary").hexdigest(),
            )
            self.assertEqual(
                result["artifact_export"]["export_path"], str(output)
            )
            self.assertEqual(
                result["artifact_export"]["destination_contract"][
                    "decision"
                ],
                "UNTRACKED_IGNORED_CANDIDATE_TARGET",
            )
            self.assertEqual(
                result["artifact_export"]["residue_recovery"][
                    "temporary_removed"
                ],
                1,
            )
            self.assertEqual(
                result["artifact_export"]["residue_recovery"][
                    "previous_recovered_or_removed"
                ],
                1,
            )
            self.assertFalse(
                any(
                    path.name.startswith(
                        (
                            ".codefire-rs.tmp-",
                            ".codefire-rs.previous-",
                        )
                    )
                    for path in output.parent.iterdir()
                )
            )
            committed = read_run_state(
                Path(result["target"]["namespace"]),
                source_key=result["artifact_key"],
            )
            self.assertEqual(committed["state"], "COMMITTED")
            self.assertEqual(
                committed["execution_key"], result["execution_key"]
            )

    def test_release_export_accepts_the_cargo_uplift_hardlink(
        self,
    ) -> None:
        # The 2026-08-02 live release build compiled for four minutes and
        # then blocked `artifact-not-private-executable` on
        # `release/codefire-rs`. Cargo uplifts a binary by hard-linking the
        # compiled `release/deps/<crate>-<metadata>` inode onto the
        # profile-root name, so the export source always arrives with two
        # links. Reproduce that exact on-disk shape through the whole gate,
        # not only through the export helper, because the helper is not
        # where the release lane failed.
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            output = root / "target/release/codefire-rs"
            output.parent.mkdir(parents=True)
            uplift: dict[str, int] = {}

            def runner(*args, **kwargs):
                target = Path(kwargs["environment"]["CARGO_TARGET_DIR"])
                compiled = (
                    target / "release/deps/example-0123456789abcdef"
                )
                compiled.parent.mkdir(parents=True, exist_ok=True)
                compiled.write_bytes(b"source-keyed-binary")
                compiled.chmod(0o755)
                artifact = target / "release/codefire-rs"
                os.link(compiled, artifact)
                observed = artifact.stat()
                uplift["nlink"] = observed.st_nlink
                uplift["inode"] = observed.st_ino
                return self.fake_runner(b"")(*args, **kwargs)

            result = run_source_keyed_gate(
                [
                    "cargo",
                    "build",
                    "--release",
                    "--locked",
                    "-p",
                    "example",
                ],
                root,
                temporary / "evidence/uplift.log",
                30,
                environment=environment,
                profile=profile,
                new_namespace_free_floor_bytes=0,
                export_artifact_relative="release/codefire-rs",
                export_path=output,
                gate_runner=runner,
            )
            self.assertEqual(uplift["nlink"], 2)
            self.assertEqual(result["decision"], "PASS")
            export = result["artifact_export"]
            self.assertEqual(export["decision"], "EXPORTED")
            self.assertEqual(export["source_nlink"], 2)
            self.assertTrue(export["source_cargo_uplift_hardlinked"])
            self.assertEqual(
                export["source_nlink_maximum"],
                MAX_EXPORT_SOURCE_ARTIFACT_LINKS,
            )
            self.assertEqual(
                export["sha256"],
                hashlib.sha256(b"source-keyed-binary").hexdigest(),
            )
            self.assertEqual(output.read_bytes(), b"source-keyed-binary")
            published = output.stat()
            self.assertEqual(published.st_nlink, 1)
            self.assertEqual(export["export_nlink"], 1)
            self.assertEqual(stat.S_IMODE(published.st_mode), 0o700)
            # Accepting a hard-linked *source* must not turn publication
            # into a link: the destination is a fresh inode written through
            # a descriptor, so nothing in the artifact namespace can still
            # reach the exported bytes.
            self.assertNotEqual(published.st_ino, uplift["inode"])
            self.assertEqual(export["export_inode"], published.st_ino)

    def test_tracked_or_nonignored_artifact_export_is_rejected(
        self,
    ) -> None:
        for tracked in (False, True):
            with self.subTest(tracked=tracked), tempfile.TemporaryDirectory() as raw:
                temporary = Path(raw).resolve()
                root, profile, environment = self.make_repository(temporary)
                output = root / "target/release/codefire-rs"
                if tracked:
                    output.parent.mkdir(parents=True)
                    output.write_bytes(b"tracked")
                    output.chmod(0o700)
                    subprocess.run(
                        ["git", "add", "-f", output.relative_to(root)],
                        cwd=root,
                        check=True,
                    )
                else:
                    (root / ".gitignore").write_text(
                        "/unrelated/\n", encoding="utf-8"
                    )
                    subprocess.run(
                        ["git", "add", ".gitignore"],
                        cwd=root,
                        check=True,
                    )
                subprocess.run(
                    ["git", "commit", "-qm", "unsafe export contract"],
                    cwd=root,
                    check=True,
                )
                with self.assertRaises(GateBlocked) as raised:
                    run_source_keyed_gate(
                        [
                            "cargo",
                            "build",
                            "--release",
                            "--locked",
                            "-p",
                            "example",
                        ],
                        root,
                        temporary / "evidence/export-rejected.log",
                        30,
                        environment=environment,
                        profile=profile,
                        new_namespace_free_floor_bytes=0,
                        export_artifact_relative="release/codefire-rs",
                        export_path=output,
                        gate_runner=self.fake_runner(b""),
                    )
                self.assertEqual(
                    raised.exception.code,
                    (
                        "artifact-export-destination-tracked"
                        if tracked
                        else "artifact-export-destination-not-ignored"
                    ),
                )

    def test_artifact_is_not_published_before_final_pass_predicates(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            output = root / "target/release/codefire-rs"
            output.parent.mkdir(parents=True)
            output.write_bytes(b"previous-known-good")
            output.chmod(0o700)

            def mutate_after_build(*args, **kwargs):
                target = Path(kwargs["environment"]["CARGO_TARGET_DIR"])
                artifact = target / "release/codefire-rs"
                artifact.parent.mkdir(parents=True, exist_ok=True)
                artifact.write_bytes(b"must-not-publish")
                artifact.chmod(0o700)
                result = self.fake_runner(b"")(*args, **kwargs)
                (root / "crates/example/src/lib.rs").write_text(
                    "pub fn value() -> u8 { 99 }\n",
                    encoding="utf-8",
                )
                return result

            result = run_source_keyed_gate(
                [
                    "cargo",
                    "build",
                    "--release",
                    "--locked",
                    "-p",
                    "example",
                ],
                root,
                temporary / "evidence/blocked-export.log",
                30,
                environment=environment,
                profile=profile,
                new_namespace_free_floor_bytes=0,
                export_artifact_relative="release/codefire-rs",
                export_path=output,
                gate_runner=mutate_after_build,
            )
            self.assertEqual(result["decision"], "BLOCK")
            self.assertEqual(output.read_bytes(), b"previous-known-good")
            self.assertEqual(
                result["artifact_export"]["decision"],
                "NOT_EXPORTED",
            )

    def test_clippy_and_fmt_use_pinned_primary_and_companion_tools(
        self,
    ) -> None:
        cases = (
            (
                ["cargo", "clippy", "--locked"],
                "RUSTC_WORKSPACE_WRAPPER",
                "toolchain:clippy-driver",
            ),
            (
                ["cargo", "fmt", "--", "--check"],
                "RUSTFMT",
                "toolchain:rustfmt",
            ),
        )
        for index, (command, environment_name, descriptor_name) in enumerate(
            cases
        ):
            with self.subTest(command=command), tempfile.TemporaryDirectory() as temporary_raw:
                temporary = Path(temporary_raw).resolve()
                root, profile, environment = self.make_repository(temporary)
                capture: dict[str, object] = {}
                result = run_source_keyed_gate(
                    command,
                    root,
                    temporary / f"evidence/tool-{index}.log",
                    30,
                    environment=environment,
                    profile=profile,
                    ephemeral=True,
                    new_namespace_free_floor_bytes=0,
                    gate_runner=self.fake_runner(b"", capture),
                )
                self.assertEqual(result["decision"], "PASS")
                used_environment = capture["environment"]
                self.assertIsInstance(used_environment, dict)
                toolchain = result["toolchain_identity"]["executables"]
                companion = descriptor_name.split(":", 1)[1]
                for pinned_environment_name in (
                    environment_name,
                    "CARGO",
                    "RUSTC",
                    "RUSTDOC",
                ):
                    descriptor_path = used_environment[
                        pinned_environment_name
                    ]  # type: ignore[index]
                    self.assertIsInstance(descriptor_path, str)
                    self.assertTrue(
                        descriptor_path.startswith("/proc/self/fd/")
                    )
                    self.assertIn(
                        int(descriptor_path.rsplit("/", 1)[1]),
                        capture["pass_fds"],  # type: ignore[operator]
                    )
                launch = capture["argv"]
                self.assertIsInstance(launch, list)
                primary_index = launch.index("--primary-fd") + 1  # type: ignore[union-attr]
                primary_descriptor = int(launch[primary_index])  # type: ignore[index]
                self.assertIn(
                    primary_descriptor,
                    capture["pass_fds"],  # type: ignore[operator]
                )
                self.assertIn(companion, toolchain)

    def test_successful_zero_test_filter_is_blocked(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            result = run_source_keyed_gate(
                ["cargo", "test", "--bin", "example", "--locked", "missing_filter"],
                root,
                temporary / "evidence/zero.log",
                30,
                environment=environment,
                profile=profile,
                new_namespace_free_floor_bytes=0,
                gate_runner=self.fake_runner(
                    (
                        b"running 0 tests\n\n"
                        b"test result: ok. 0 passed; 0 failed; 0 ignored; "
                        b"0 measured; 794 filtered out\n"
                    )
                ),
            )
            self.assertEqual(result["decision"], "BLOCK")
            self.assertIn(
                "cargo-test-executed-zero-tests", result["blockers"]
            )
            self.assertEqual(
                result["retention"]["decision"],
                "INCOMPLETE_GATE_CLEANED",
            )
            self.assertFalse(
                Path(result["target"]["namespace"]).exists()
            )

    def test_ignored_only_test_filter_is_blocked(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            result = run_source_keyed_gate(
                ["cargo", "test", "--bin", "example", "--locked", "ignored_filter"],
                root,
                temporary / "evidence/ignored.log",
                30,
                environment=environment,
                profile=profile,
                new_namespace_free_floor_bytes=0,
                gate_runner=self.fake_runner(
                    (
                        b"running 1 test\n"
                        b"test example::ignored_filter ... ignored\n"
                        b"test result: ok. 0 passed; 0 failed; 1 ignored; "
                        b"0 measured; 7 filtered out\n"
                    )
                ),
            )
            self.assertEqual(result["decision"], "BLOCK")
            self.assertEqual(result["test_execution"]["executed_tests"], 0)
            self.assertIn(
                "cargo-test-executed-zero-tests", result["blockers"]
            )

    def test_candidate_change_during_gate_invalidates_success(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            base_runner = self.fake_runner(
                (
                    b"running 1 test\n"
                    b"test example::focused ... ok\n"
                    b"test result: ok. 1 passed; 0 failed; 0 ignored; "
                    b"0 measured; 0 filtered out\n"
                )
            )

            def mutate_after_gate(*args, **kwargs):
                result = base_runner(*args, **kwargs)
                (root / "crates/example/src/lib.rs").write_text(
                    "pub fn value() -> u8 { 9 }\n", encoding="utf-8"
                )
                return result

            result = run_source_keyed_gate(
                ["cargo", "test", "--bin", "example", "--locked", "focused"],
                root,
                temporary / "evidence/mutated.log",
                30,
                environment=environment,
                profile=profile,
                new_namespace_free_floor_bytes=0,
                gate_runner=mutate_after_gate,
            )
            self.assertEqual(result["decision"], "BLOCK")
            self.assertFalse(result["candidate_unchanged"])
            self.assertIn("candidate-changed-during-gate", result["blockers"])
            # Counterexample: recapturing with the clean-only fast path
            # would raise a generic "requires a clean candidate" admission
            # error out of the gate instead of returning this result, so
            # the specific blocker and the evidence of what the candidate
            # became would both be lost.
            self.assertFalse(result["candidate"]["dirty"])
            self.assertTrue(result["candidate_after"]["dirty"])
            self.assertNotEqual(
                result["candidate_after"]["dirty_digest"],
                result["candidate"]["dirty_digest"],
            )
            self.assertEqual(
                result["candidate_after"]["tree"],
                result["candidate"]["tree"],
            )

    def test_snapshot_mutation_during_gate_blocks_and_is_cleaned(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            base_runner = self.fake_runner(b"validation output\n")

            def mutate_snapshot(*args, **kwargs):
                launch = args[0]
                snapshot = Path(launch[launch.index("--snapshot") + 1])
                source = snapshot / "crates/example/src/lib.rs"
                source.chmod(0o644)
                source.write_text(
                    "pub fn value() -> u8 { 200 }\n",
                    encoding="utf-8",
                )
                return base_runner(*args, **kwargs)

            with self.assertRaises(GateBlocked) as raised:
                run_source_keyed_gate(
                    ["cargo", "check", "--locked"],
                    root,
                    temporary / "evidence/snapshot-mutated.log",
                    30,
                    environment=environment,
                    profile=profile,
                    new_namespace_free_floor_bytes=0,
                    gate_runner=mutate_snapshot,
                )
            self.assertEqual(
                raised.exception.code,
                "snapshot-content-changed-during-gate",
            )
            self.assertEqual(
                list(Path(str(profile["snapshot_root"])).iterdir()),
                [],
            )
            registry = (
                Path(str(profile["target_dir"]))
                / SOURCE_KEY_TARGET_DIRECTORY
            )
            self.assertEqual(list(registry.iterdir()), [])

    def test_candidate_change_after_running_transition_is_cleaned(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            original_capture = capture_source_state
            call_count = 0

            def change_second_capture(candidate):
                nonlocal call_count
                call_count += 1
                observed = original_capture(candidate)
                if call_count == 2:
                    observed = dict(observed)
                    observed["tree"] = "f" * 40
                return observed

            with mock.patch(
                "run_source_keyed_cargo_gate.capture_source_state",
                side_effect=change_second_capture,
            ), self.assertRaises(GateBlocked) as raised:
                run_source_keyed_gate(
                    ["cargo", "check", "--locked"],
                    root,
                    temporary / "evidence/changed-before.log",
                    30,
                    environment=environment,
                    profile=profile,
                    new_namespace_free_floor_bytes=0,
                    gate_runner=self.fake_runner(b""),
                )
            self.assertEqual(
                raised.exception.code, "candidate-changed-before-gate"
            )
            registry = (
                Path(str(profile["target_dir"]))
                / SOURCE_KEY_TARGET_DIRECTORY
            )
            self.assertEqual(list(registry.iterdir()), [])

    def test_post_gate_bookkeeping_exception_cleans_running_namespace(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            base_runner = self.fake_runner(b"validation output\n")

            def corrupt_reserved_metadata(*args, **kwargs):
                target = Path(kwargs["environment"]["CARGO_TARGET_DIR"])
                (
                    target / ".codefire-source-key-invalid.json"
                ).write_bytes(b"invalid-child-bytes")
                return base_runner(*args, **kwargs)

            with self.assertRaises(GateBlocked) as raised:
                run_source_keyed_gate(
                    ["cargo", "check", "--locked"],
                    root,
                    temporary / "evidence/corrupt-bookkeeping.log",
                    30,
                    environment=environment,
                    profile=profile,
                    new_namespace_free_floor_bytes=0,
                    gate_runner=corrupt_reserved_metadata,
                )
            self.assertEqual(
                raised.exception.code, "namespace-invalidation-invalid"
            )
            registry = (
                Path(str(profile["target_dir"]))
                / SOURCE_KEY_TARGET_DIRECTORY
            )
            self.assertEqual(list(registry.iterdir()), [])

    def test_lockfile_and_config_overrides_are_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            cases = (
                (
                    ["cargo", "test", "--bin", "example", "focused"],
                    "cargo-lockfile-not-frozen",
                ),
                (
                    [
                        "cargo",
                        "test",
                        "--locked",
                        "--config",
                        "build.target-dir='/tmp/foreign'",
                    ],
                    "cargo-path-or-toolchain-override",
                ),
            )
            for index, (command, expected_code) in enumerate(cases):
                with self.subTest(command=command):
                    with self.assertRaises(GateBlocked) as raised:
                        run_source_keyed_gate(
                            command,
                            root,
                            temporary / f"evidence/rejected-{index}.log",
                            30,
                            environment=environment,
                            profile=profile,
                            gate_runner=self.fake_runner(b""),
                        )
                    self.assertEqual(raised.exception.code, expected_code)

    def test_dirty_candidate_is_not_trusted_even_when_ephemeral_is_requested(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            (root / "crates/example/src/lib.rs").write_text(
                "pub fn value() -> u8 { 3 }\n", encoding="utf-8"
            )
            with self.assertRaises(GateBlocked) as raised:
                run_source_keyed_gate(
                    ["cargo", "test", "--bin", "example", "--locked", "focused"],
                    root,
                    temporary / "evidence/dirty-blocked.log",
                    30,
                    environment=environment,
                    profile=profile,
                    gate_runner=self.fake_runner(b""),
                )
            self.assertEqual(
                raised.exception.code, "dirty-candidate-not-trusted"
            )
            with self.assertRaises(GateBlocked) as ephemeral_raised:
                run_source_keyed_gate(
                    [
                        "cargo",
                        "test",
                        "--bin",
                        "example",
                        "--locked",
                        "focused",
                    ],
                    root,
                    temporary / "evidence/dirty-ephemeral.log",
                    30,
                    environment=environment,
                    profile=profile,
                    ephemeral=True,
                    new_namespace_free_floor_bytes=0,
                    gate_runner=self.fake_runner(b""),
                )
            self.assertEqual(
                ephemeral_raised.exception.code,
                "dirty-candidate-not-trusted",
            )

    def test_persistent_aggregate_budget_blocks_unbounded_growth(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            _, profile, _ = self.make_repository(temporary)
            admitted = Path(str(profile["target_dir"]))
            first_payload = {
                "schema": "codefire.cargo_source_key.v1",
                "candidate": {"tree": "first"},
            }
            first_target, _, _, _, _ = prepare_namespace(
                admitted,
                "c" * 64,
                first_payload,
                lock_timeout_seconds=0,
                persistent_aggregate_budget_bytes=1_000_000,
                new_namespace_allocation_reserve_bytes=100,
                new_namespace_free_floor_bytes=0,
            )
            write_retention_record(
                first_target,
                source_key="c" * 64,
                usage={
                    "allocated_bytes": 100,
                    "logical_bytes": 50,
                    "entry_count": 2,
                },
            )
            write_run_state(
                first_target,
                source_key="c" * 64,
                state="COMMITTED",
                execution_key_value="e" * 64,
            )
            reused, _, _, _, admission = prepare_namespace(
                admitted,
                "c" * 64,
                first_payload,
                lock_timeout_seconds=0,
                persistent_aggregate_budget_bytes=1_000_000,
                new_namespace_allocation_reserve_bytes=1,
                new_namespace_free_floor_bytes=0,
            )
            observed = admission[
                "persistent_aggregate_allocated_bytes_before"
            ]
            self.assertGreater(observed, 0)
            with self.assertRaises(GateBlocked) as raised:
                prepare_namespace(
                    admitted,
                    "d" * 64,
                    {
                        "schema": "codefire.cargo_source_key.v1",
                        "candidate": {"tree": "second"},
                    },
                    lock_timeout_seconds=0,
                    persistent_aggregate_budget_bytes=observed,
                    new_namespace_allocation_reserve_bytes=1,
                    new_namespace_free_floor_bytes=0,
                )
            self.assertEqual(
                raised.exception.code,
                "persistent-aggregate-budget-exceeded",
            )
            self.assertEqual(reused, first_target)
            self.assertTrue(admission["namespace_preexisting"])
            with self.assertRaises(GateBlocked) as existing_raised:
                prepare_namespace(
                    admitted,
                    "c" * 64,
                    first_payload,
                    lock_timeout_seconds=0,
                    persistent_aggregate_budget_bytes=1,
                    new_namespace_allocation_reserve_bytes=1,
                    new_namespace_free_floor_bytes=0,
                )
            self.assertEqual(
                existing_raised.exception.code,
                "persistent-aggregate-budget-already-exceeded",
            )

    def test_post_gate_actual_allocation_overshoot_blocks_and_cleans_new_key(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            base_runner = self.fake_runner(
                (
                    b"running 1 test\n"
                    b"test example::focused ... ok\n"
                    b"test result: ok. 1 passed; 0 failed; 0 ignored; "
                    b"0 measured; 0 filtered out\n"
                )
            )

            def grow_namespace(*args, **kwargs):
                target = Path(kwargs["environment"]["CARGO_TARGET_DIR"])
                (target / "overshoot.bin").write_bytes(b"x" * (2 * 1024**2))
                return base_runner(*args, **kwargs)

            result = run_source_keyed_gate(
                ["cargo", "test", "--bin", "example", "--locked", "focused"],
                root,
                temporary / "evidence/overshoot.log",
                30,
                environment=environment,
                profile=profile,
                persistent_aggregate_budget_bytes=1024**2,
                new_namespace_allocation_reserve_bytes=1,
                new_namespace_free_floor_bytes=0,
                gate_runner=grow_namespace,
            )
            self.assertEqual(result["decision"], "BLOCK")
            self.assertIn(
                "post-gate-persistent-aggregate-budget-exceeded",
                result["blockers"],
            )
            self.assertEqual(
                result["retention"]["decision"],
                "POST_POLICY_EXCEEDED_CLEANED",
            )
            self.assertFalse(
                Path(result["target"]["namespace"]).exists()
            )
            self.assertFalse(
                result["retention"]["namespace_exists_after"]
            )

    def test_evidence_destination_and_parent_identity_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, _ = self.make_repository(temporary)
            state = capture_source_state(root)
            evidence_root = Path(str(profile["evidence_root"]))
            existing = evidence_root / "existing.log"
            existing.write_bytes(b"occupied")
            with self.assertRaises(GateBlocked) as occupied:
                evidence_path(
                    existing,
                    candidate_root=root,
                    admitted_root=Path(str(profile["target_dir"])),
                    git_common_dir=Path(state["git_common_dir"]),
                    git_dir=Path(state["git_dir"]),
                    profile=profile,
                    label="gate log",
                )
            self.assertEqual(
                occupied.exception.code, "evidence-path-exists"
            )

            nested_parent = evidence_root / "new/nested"
            with self.assertRaises(GateBlocked) as nested:
                evidence_path(
                    nested_parent / "gate.log",
                    candidate_root=root,
                    admitted_root=Path(str(profile["target_dir"])),
                    git_common_dir=Path(state["git_common_dir"]),
                    git_dir=Path(state["git_dir"]),
                    profile=profile,
                    label="gate log",
                )
            self.assertEqual(
                nested.exception.code, "unsafe-evidence-path"
            )
            self.assertFalse((evidence_root / "new").exists())

            outside = temporary / "outside"
            outside.mkdir()
            (evidence_root / "linked").symlink_to(outside, target_is_directory=True)
            with self.assertRaises(GateBlocked) as symlinked:
                evidence_path(
                    evidence_root / "linked/gate.log",
                    candidate_root=root,
                    admitted_root=Path(str(profile["target_dir"])),
                    git_common_dir=Path(state["git_common_dir"]),
                    git_dir=Path(state["git_dir"]),
                    profile=profile,
                    label="gate log",
                )
            self.assertEqual(
                symlinked.exception.code, "unsafe-evidence-path"
            )

            overlapping_profile = dict(profile)
            overlapping_profile["evidence_root"] = str(root)
            with self.assertRaises(GateBlocked) as overlapping:
                evidence_path(
                    root / "gate.log",
                    candidate_root=root,
                    admitted_root=Path(str(profile["target_dir"])),
                    git_common_dir=Path(state["git_common_dir"]),
                    git_dir=Path(state["git_dir"]),
                    profile=overlapping_profile,
                    label="gate log",
                )
            self.assertEqual(
                overlapping.exception.code,
                "unsafe-admitted-evidence-root",
            )

    def test_toolchain_mismatch_is_rejected_before_execution(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            environment["RUSTUP_TOOLCHAIN"] = "nightly"
            with self.assertRaises(GateBlocked) as raised:
                run_source_keyed_gate(
                    ["cargo", "test", "--bin", "example", "--locked", "focused"],
                    root,
                    temporary / "evidence/toolchain.log",
                    30,
                    environment=environment,
                    profile=profile,
                    gate_runner=self.fake_runner(b""),
                )
            self.assertEqual(
                raised.exception.code, "wrong-rustup-toolchain"
            )

    def test_non_finite_gate_and_lock_timeouts_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            cases = (
                (float("nan"), 1.0, "invalid-gate-timeout"),
                (30.0, float("inf"), "invalid-lock-timeout"),
            )
            for index, (timeout, lock_timeout, code) in enumerate(cases):
                with self.subTest(code=code):
                    with self.assertRaises(GateBlocked) as raised:
                        run_source_keyed_gate(
                            ["cargo", "check", "--locked"],
                            root,
                            temporary / f"evidence/nonfinite-{index}.log",
                            timeout,
                            lock_timeout_seconds=lock_timeout,
                            environment=environment,
                            profile=profile,
                            gate_runner=self.fake_runner(b""),
                        )
                    self.assertEqual(raised.exception.code, code)

    def test_git_identity_subprocess_has_a_wall_deadline(self) -> None:
        started = time.monotonic()
        with tempfile.TemporaryDirectory() as temporary_raw, mock.patch(
            "run_source_keyed_cargo_gate.GIT_EXECUTABLE",
            Path("/bin/sh"),
        ), mock.patch(
            "run_source_keyed_cargo_gate.GIT_CONTROL_CONFIG_ARGS",
            (),
        ), self.assertRaises(GateBlocked) as raised:
            git_bytes(
                Path(temporary_raw),
                "-c",
                "sleep 5",
                timeout_seconds=0.05,
            )
        self.assertEqual(
            raised.exception.code, "git-operation-timeout"
        )
        self.assertLess(time.monotonic() - started, 2)

    def test_block_decision_is_atomically_written_when_result_path_is_safe(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, _ = self.make_repository(temporary)
            result_path = temporary / "evidence/block.json"
            with mock.patch.dict(DEFAULT_PROFILE, profile, clear=True):
                returncode = source_keyed_cargo_main(
                    [
                        "--repo",
                        str(root),
                        "--log",
                        str(temporary / "evidence/block.log"),
                        "--result",
                        str(result_path),
                        "--timeout",
                        "30",
                        "--",
                        "cargo",
                        "test",
                        "focused",
                    ]
                )
            self.assertEqual(returncode, 3)
            payload = json.loads(result_path.read_text(encoding="utf-8"))
            self.assertEqual(payload["decision"], "BLOCK")
            self.assertEqual(
                payload["blockers"], ["cargo-lockfile-not-frozen"]
            )

    def test_unexpected_failure_is_durably_recorded_as_block(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, _ = self.make_repository(temporary)
            result_path = temporary / "evidence/internal-block.json"
            with mock.patch.dict(DEFAULT_PROFILE, profile, clear=True), mock.patch(
                "run_source_keyed_cargo_gate.run_source_keyed_gate",
                side_effect=RuntimeError("synthetic post-admission failure"),
            ):
                returncode = source_keyed_cargo_main(
                    [
                        "--repo",
                        str(root),
                        "--log",
                        str(temporary / "evidence/internal.log"),
                        "--result",
                        str(result_path),
                        "--timeout",
                        "30",
                        "--",
                        "cargo",
                        "check",
                        "--locked",
                    ]
                )
            self.assertEqual(returncode, 2)
            payload = json.loads(result_path.read_text(encoding="utf-8"))
            self.assertEqual(payload["decision"], "BLOCK")
            self.assertEqual(
                payload["blockers"],
                ["source-keyed-gate-internal-error"],
            )

    def test_result_is_reserved_before_gate_and_survives_publish_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, _ = self.make_repository(temporary)
            result_path = temporary / "evidence/reserved.json"
            with mock.patch.dict(
                DEFAULT_PROFILE, profile, clear=True
            ), mock.patch(
                "run_source_keyed_cargo_gate.run_source_keyed_gate",
                return_value={
                    "schema": "codefire.source_keyed_cargo_gate.v1",
                    "decision": "PASS",
                },
            ) as run_mock, mock.patch(
                "run_source_keyed_cargo_gate.publish_reserved_result",
                side_effect=OSError("synthetic publication failure"),
            ):
                returncode = source_keyed_cargo_main(
                    [
                        "--repo",
                        str(root),
                        "--log",
                        str(temporary / "evidence/reserved.log"),
                        "--result",
                        str(result_path),
                        "--timeout",
                        "30",
                        "--",
                        "cargo",
                        "check",
                        "--locked",
                    ]
                )
            self.assertEqual(returncode, 2)
            self.assertEqual(run_mock.call_count, 1)
            reserved = json.loads(
                result_path.read_text(encoding="utf-8")
            )
            self.assertEqual(reserved["decision"], "BLOCK")
            self.assertEqual(
                reserved["blockers"],
                ["result-publication-incomplete"],
            )
            self.assertFalse(reserved["artifact_export_accepted"])

    def main_capturing_stdout(
        self, argv: list[str]
    ) -> tuple[int, bytes]:
        captured = StdoutBytesCapture()
        with mock.patch.object(sys, "stdout", captured):
            returncode = source_keyed_cargo_main(argv)
        return returncode, captured.buffer.getvalue()

    def gate_argv(
        self, root: Path, temporary: Path, slot: str
    ) -> list[str]:
        return [
            "--repo",
            str(root),
            "--log",
            str(temporary / f"evidence/{slot}.log"),
            "--result",
            str(temporary / f"evidence/{slot}.json"),
            "--timeout",
            "30",
            "--",
            "cargo",
            "check",
            "--locked",
        ]

    def assert_stdout_is_the_durable_representation(
        self, stdout: bytes, durable: bytes
    ) -> dict[str, object]:
        # This is exactly the comparison `build_source_bound_rust.py`
        # makes, before it reads the decision at all. Every live wrapper
        # invocation failed it while stdout carried a pretty-printed second
        # rendering of the same payload.
        self.assertEqual(stdout, durable)
        self.assertTrue(stdout.endswith(b"\n"))
        self.assertNotIn(b"\n", stdout[:-1])
        payload = json.loads(stdout)
        self.assertIsInstance(payload, dict)
        self.assertEqual(stdout, rendered_result(payload))
        # Negative control: the representation the gate used to print. The
        # assertion is the reversal itself — if these ever became equal the
        # test above would stop distinguishing the fix from the defect.
        self.assertNotEqual(
            json.dumps(payload, indent=2, sort_keys=True).encode("utf-8")
            + b"\n",
            durable,
        )
        return payload

    def test_pass_stdout_is_the_published_result_byte_for_byte(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, _ = self.make_repository(temporary)
            result_path = temporary / "evidence/pass.json"
            with mock.patch.dict(
                DEFAULT_PROFILE, profile, clear=True
            ), mock.patch(
                "run_source_keyed_cargo_gate.run_source_keyed_gate",
                return_value={
                    "schema": "codefire.source_keyed_cargo_gate.v1",
                    "decision": "PASS",
                    "argv": ["cargo", "check", "--locked"],
                },
            ):
                returncode, stdout = self.main_capturing_stdout(
                    self.gate_argv(root, temporary, "pass")
                )
            self.assertEqual(returncode, 0)
            payload = self.assert_stdout_is_the_durable_representation(
                stdout, result_path.read_bytes()
            )
            self.assertEqual(payload["decision"], "PASS")

    def test_block_stdout_is_the_published_result_byte_for_byte(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, _ = self.make_repository(temporary)
            result_path = temporary / "evidence/block-bytes.json"
            argv = self.gate_argv(root, temporary, "block-bytes")
            # An unfrozen lockfile blocks after the result reservation, so
            # the BLOCK is the published payload rather than the reserved
            # one.
            argv[-1] = "focused"
            with mock.patch.dict(DEFAULT_PROFILE, profile, clear=True):
                returncode, stdout = self.main_capturing_stdout(argv)
            self.assertEqual(returncode, 3)
            payload = self.assert_stdout_is_the_durable_representation(
                stdout, result_path.read_bytes()
            )
            self.assertEqual(payload["decision"], "BLOCK")
            self.assertEqual(
                payload["blockers"], ["cargo-lockfile-not-frozen"]
            )

    def test_internal_error_stdout_is_the_published_result_byte_for_byte(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, _ = self.make_repository(temporary)
            result_path = temporary / "evidence/internal-bytes.json"
            with mock.patch.dict(
                DEFAULT_PROFILE, profile, clear=True
            ), mock.patch(
                "run_source_keyed_cargo_gate.run_source_keyed_gate",
                side_effect=RuntimeError("synthetic post-admission failure"),
            ):
                returncode, stdout = self.main_capturing_stdout(
                    self.gate_argv(root, temporary, "internal-bytes")
                )
            self.assertEqual(returncode, 2)
            payload = self.assert_stdout_is_the_durable_representation(
                stdout, result_path.read_bytes()
            )
            self.assertEqual(
                payload["blockers"],
                ["source-keyed-gate-internal-error"],
            )

    def test_publication_failure_stdout_is_the_reserved_bytes_on_disk(
        self,
    ) -> None:
        # When publication fails the payload falls back to the reservation.
        # stdout has to fall back with it: the reservation bytes are what
        # the result path still holds, so anything else reintroduces the
        # mismatch on precisely the path where evidence matters most.
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, _ = self.make_repository(temporary)
            result_path = temporary / "evidence/fallback.json"
            with mock.patch.dict(
                DEFAULT_PROFILE, profile, clear=True
            ), mock.patch(
                "run_source_keyed_cargo_gate.run_source_keyed_gate",
                return_value={
                    "schema": "codefire.source_keyed_cargo_gate.v1",
                    "decision": "PASS",
                },
            ), mock.patch(
                "run_source_keyed_cargo_gate.publish_reserved_result",
                side_effect=OSError("synthetic publication failure"),
            ):
                returncode, stdout = self.main_capturing_stdout(
                    self.gate_argv(root, temporary, "fallback")
                )
            self.assertEqual(returncode, 2)
            payload = self.assert_stdout_is_the_durable_representation(
                stdout, result_path.read_bytes()
            )
            self.assertEqual(payload["decision"], "BLOCK")
            self.assertEqual(
                payload["blockers"], ["result-publication-incomplete"]
            )
            self.assertEqual(payload["phase"], "RESERVED_BEFORE_EXECUTION")

    def test_pre_reservation_block_renders_raw_non_ascii_once(
        self,
    ) -> None:
        # Nothing is durable yet on this path, so the payload is rendered
        # once through the same bounded encoder. `ensure_ascii=False` means
        # a non-ASCII detail reaches stdout as UTF-8, not as `\u` escapes,
        # and a consumer comparing bytes has to see the same encoding the
        # publisher would have used.
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, _ = self.make_repository(temporary)
            nested = root / "サブ計画"
            nested.mkdir()
            result_path = temporary / "evidence/non-ascii.json"
            argv = self.gate_argv(root, temporary, "non-ascii")
            argv[1] = str(nested)
            with mock.patch.dict(
                DEFAULT_PROFILE, profile, clear=True
            ), mock.patch(
                "run_source_keyed_cargo_gate.run_source_keyed_gate"
            ) as run_mock:
                returncode, stdout = self.main_capturing_stdout(argv)
            self.assertEqual(returncode, 3)
            run_mock.assert_not_called()
            self.assertFalse(result_path.exists())
            payload = self.assert_stdout_is_the_durable_representation(
                stdout, stdout
            )
            self.assertEqual(
                payload["blockers"], ["candidate-not-worktree-root"]
            )
            self.assertIn("サブ計画".encode("utf-8"), stdout)
            self.assertNotIn(b"\\u", stdout)

    def test_preexisting_result_blocks_before_gate_execution(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, _ = self.make_repository(temporary)
            result_path = temporary / "evidence/preexisting.json"
            result_path.write_text("owner record\n", encoding="utf-8")
            with mock.patch.dict(
                DEFAULT_PROFILE, profile, clear=True
            ), mock.patch(
                "run_source_keyed_cargo_gate.run_source_keyed_gate"
            ) as run_mock:
                returncode = source_keyed_cargo_main(
                    [
                        "--repo",
                        str(root),
                        "--log",
                        str(temporary / "evidence/preexisting.log"),
                        "--result",
                        str(result_path),
                        "--timeout",
                        "30",
                        "--",
                        "cargo",
                        "check",
                        "--locked",
                    ]
                )
            self.assertEqual(returncode, 3)
            run_mock.assert_not_called()
            self.assertEqual(
                result_path.read_text(encoding="utf-8"),
                "owner record\n",
            )

    def test_target_outside_current_source_namespace_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            root, profile, environment = self.make_repository(temporary)
            environment["CARGO_TARGET_DIR"] = str(temporary / "foreign-target")
            with self.assertRaises(GateBlocked) as raised:
                run_source_keyed_gate(
                    ["cargo", "test", "--bin", "example", "--locked", "focused"],
                    root,
                    temporary / "evidence/rejected.log",
                    30,
                    environment=environment,
                    profile=profile,
                    gate_runner=self.fake_runner(b""),
                )
            self.assertEqual(
                raised.exception.code, "ambient-target-not-current-source"
            )

    def test_unmarked_crash_residue_is_quarantined_before_prepare(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            temporary = Path(temporary_raw).resolve()
            _, profile, _ = self.make_repository(temporary)
            admitted = Path(str(profile["target_dir"]))
            source_key = "b" * 64
            target = admitted / SOURCE_KEY_TARGET_DIRECTORY / source_key
            target.mkdir(parents=True)
            stale = target / "debug-artifact"
            stale.write_bytes(b"foreign")
            prepared, _, _, _, admission = prepare_namespace(
                admitted,
                source_key,
                {
                    "schema": "codefire.cargo_source_key.v1",
                    "candidate": {"tree": "replacement"},
                },
                lock_timeout_seconds=0,
                new_namespace_free_floor_bytes=0,
            )
            self.assertEqual(
                admission["incomplete_namespace_recovery"][
                    "removed_count"
                ],
                1,
            )
            self.assertEqual(prepared, target)
            self.assertFalse(stale.exists())
            self.assertEqual(
                read_run_state(target, source_key=source_key)["state"],
                "PREPARED",
            )

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
                "RUSTUP_TOOLCHAIN": DEFAULT_PROFILE["rustup_toolchain"],
                "CARGO_TARGET_DIR": target,
                "CARGO_BUILD_JOBS": "2",
                "RUST_TEST_THREADS": "2",
                "CARGO_INCREMENTAL": "0",
            },
        }

    def test_unkeyed_shared_root_process_is_blocked(self) -> None:
        codes = {
            item["code"] for item in evaluate_process(self.observation())
        }
        self.assertEqual(
            codes,
            {
                "unkeyed-live-target",
                "missing-or-wrong-source-key-gate-provenance",
                "missing-or-wrong-source-key-gate-key",
            },
        )

    def test_source_keyed_target_namespace_passes(self) -> None:
        observation = self.observation()
        source_target = (
            f"{DEFAULT_PROFILE['target_dir']}/{SOURCE_KEY_TARGET_DIRECTORY}/"
            f"{'a' * 64}"
        )
        environment = dict(observation["environment"])  # type: ignore[arg-type]
        environment["CARGO_TARGET_DIR"] = source_target
        environment[
            "CODEFIRE_CARGO_GATE_PROVENANCE"
        ] = "codefire.source_keyed_cargo_gate.v1"
        environment["CODEFIRE_CARGO_GATE_SOURCE_KEY"] = "a" * 64
        observation["environment"] = environment
        observation["argv"] = [
            f"{DEFAULT_PROFILE['rustup_home']}/toolchains/1.97.0/bin/rustc",
            "--out-dir",
            f"{source_target}/debug/deps",
        ]
        observation["runtime_source_evidence"] = {
            "schema": LIVE_SOURCE_EVIDENCE_SCHEMA,
            "status": "verified-structural",
            "source_key": "a" * 64,
            "run_state": "RUNNING",
            "execution_key": "b" * 64,
            "marker_sha256": "c" * 64,
            "run_state_sha256": "d" * 64,
            "control_files_read_only": True,
            "cryptographic_attestation": False,
        }
        self.assertEqual(evaluate_process(observation), [])
        self.assertTrue(
            target_dir_is_admitted(
                source_target,
                cwd=Path("/workspace/worktrees/candidate"),
            )
        )

    def test_forged_live_source_key_environment_is_not_attestation(
        self,
    ) -> None:
        observation = self.observation()
        source_target = (
            f"{DEFAULT_PROFILE['target_dir']}/{SOURCE_KEY_TARGET_DIRECTORY}/"
            f"{'a' * 64}"
        )
        environment = dict(observation["environment"])  # type: ignore[arg-type]
        environment["CARGO_TARGET_DIR"] = source_target
        environment[
            "CODEFIRE_CARGO_GATE_PROVENANCE"
        ] = "codefire.source_keyed_cargo_gate.v1"
        environment["CODEFIRE_CARGO_GATE_SOURCE_KEY"] = "a" * 64
        observation["environment"] = environment
        observation["argv"] = [
            f"{DEFAULT_PROFILE['rustup_home']}/toolchains/1.97.0/bin/rustc",
            "--out-dir",
            f"{source_target}/debug/deps",
        ]
        codes = {item["code"] for item in evaluate_process(observation)}
        self.assertEqual(
            codes,
            {"unverified-live-source-key-runtime-evidence"},
        )

    def test_live_reader_correlates_read_only_running_controls(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw).resolve()
            profile = dict(DEFAULT_PROFILE)
            profile["rustup_home"] = str(root / "rustup")
            profile["cargo_home"] = str(root / "cargo")
            profile["target_dir"] = str(root / "target")
            source_key = "a" * 64
            execution_key = "b" * 64
            target = (
                Path(profile["target_dir"])
                / SOURCE_KEY_TARGET_DIRECTORY
                / source_key
            )
            process_root = root / "process-root"
            process_target = process_root / target.relative_to("/")
            process_target.mkdir(parents=True)

            def control_bytes(payload: dict[str, object]) -> bytes:
                return (
                    json.dumps(
                        payload,
                        ensure_ascii=False,
                        sort_keys=True,
                        separators=(",", ":"),
                    ).encode()
                    + b"\n"
                )

            marker = {
                "schema": "codefire.cargo_target_namespace.v1",
                "source_key": source_key,
                "source_key_payload": {},
                "target_namespace": str(target),
            }
            run_state = {
                "schema": "codefire.cargo_namespace_run_state.v1",
                "source_key": source_key,
                "state": "RUNNING",
                "execution_key": execution_key,
            }
            (process_target / ".codefire-source-key.json").write_bytes(
                control_bytes(marker)
            )
            (
                process_target / ".codefire-source-key-run-state.json"
            ).write_bytes(control_bytes(run_state))

            proc = root / "proc"
            process = proc / "101"
            process.mkdir(parents=True)
            cwd = root / "workspace"
            cwd.mkdir()
            toolchain = (
                Path(profile["rustup_home"])
                / "toolchains"
                / str(profile["rustup_toolchain"])
                / "bin"
            )
            toolchain.mkdir(parents=True)
            rustc = toolchain / "rustc"
            rustc.write_bytes(b"not executed")
            (process / "cwd").symlink_to(cwd)
            (process / "root").symlink_to(
                process_root, target_is_directory=True
            )
            (process / "exe").symlink_to(rustc)
            (process / "cmdline").write_bytes(
                (
                    f"{rustc}\0--out-dir\0{target / 'debug/deps'}\0"
                ).encode()
            )
            environment = {
                "RUSTUP_HOME": str(profile["rustup_home"]),
                "CARGO_HOME": str(profile["cargo_home"]),
                "RUSTUP_TOOLCHAIN": str(profile["rustup_toolchain"]),
                "CARGO_TARGET_DIR": str(target),
                "CARGO_BUILD_JOBS": "2",
                "RUST_TEST_THREADS": "2",
                "CARGO_INCREMENTAL": "0",
                (
                    "CODEFIRE_CARGO_GATE_PROVENANCE"
                ): "codefire.source_keyed_cargo_gate.v1",
                "CODEFIRE_CARGO_GATE_SOURCE_KEY": source_key,
            }
            (process / "environ").write_bytes(
                b"\0".join(
                    f"{key}={value}".encode()
                    for key, value in environment.items()
                )
                + b"\0"
            )
            marker_mount = target / ".codefire-source-key.json"
            run_state_mount = (
                target / ".codefire-source-key-run-state.json"
            )
            (process / "mountinfo").write_text(
                (
                    f"31 20 0:1 / {marker_mount} ro - tmpfs tmpfs ro\n"
                    f"32 20 0:1 / {run_state_mount} ro - tmpfs tmpfs ro\n"
                ),
                encoding="utf-8",
            )
            observations = read_process_observations(
                proc, [cwd], profile=profile
            )
        self.assertEqual(len(observations), 1)
        evidence = observations[0]["runtime_source_evidence"]
        self.assertEqual(evidence["status"], "verified-structural")
        self.assertFalse(evidence["cryptographic_attestation"])
        self.assertEqual(evaluate_process(observations[0], profile), [])

    def test_fd_argv_is_identified_from_proc_executable(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw).resolve()
            proc = root / "proc"
            process = proc / "101"
            process.mkdir(parents=True)
            cwd = root / "workspace"
            cwd.mkdir()
            toolchain = (
                root
                / "rustup"
                / "toolchains"
                / "1.97.0-x86_64-unknown-linux-gnu"
                / "bin"
            )
            toolchain.mkdir(parents=True)
            rustc = toolchain / "rustc"
            rustc.write_bytes(b"not executed")
            (process / "cwd").symlink_to(cwd)
            (process / "exe").symlink_to(rustc)
            (process / "cmdline").write_bytes(
                b"/proc/self/fd/17\0--version\0"
            )
            (process / "environ").write_bytes(b"")
            observations = read_process_observations(proc, [cwd])
        self.assertEqual(len(observations), 1)
        self.assertEqual(observations[0]["tool"], "rustc")
        self.assertEqual(observations[0]["executable"], str(rustc))

    def test_cargo_companion_executables_are_live_rust_tools(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw).resolve()
            proc = root / "proc"
            cwd = root / "workspace"
            cwd.mkdir()
            toolchain = root / "toolchain"
            toolchain.mkdir()
            for pid, tool in ((101, "cargo-clippy"), (102, "cargo-fmt")):
                process = proc / str(pid)
                process.mkdir(parents=True)
                executable = toolchain / tool
                executable.write_bytes(b"not executed")
                (process / "cwd").symlink_to(cwd)
                (process / "exe").symlink_to(executable)
                (process / "cmdline").write_bytes(
                    f"/proc/self/fd/{pid}\0--version\0".encode()
                )
                (process / "environ").write_bytes(b"")
            observations = read_process_observations(proc, [cwd])
        self.assertEqual(
            {observation["tool"] for observation in observations},
            {"cargo-clippy", "cargo-fmt"},
        )

    def test_arbitrary_shared_target_descendant_is_blocked(self) -> None:
        observation = self.observation()
        environment = dict(observation["environment"])  # type: ignore[arg-type]
        environment["CARGO_TARGET_DIR"] = (
            f"{DEFAULT_PROFILE['target_dir']}/debug-reused-across-worktrees"
        )
        observation["environment"] = environment
        codes = {item["code"] for item in evaluate_process(observation)}
        self.assertIn("wrong-target-dir", codes)

    def test_exact_rustup_toolchain_is_required(self) -> None:
        observation = self.observation()
        environment = dict(observation["environment"])  # type: ignore[arg-type]
        environment["RUSTUP_TOOLCHAIN"] = "stable"
        observation["environment"] = environment
        codes = {item["code"] for item in evaluate_process(observation)}
        self.assertIn("wrong-rustup-toolchain", codes)

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
                "missing-or-wrong-source-key-gate-provenance",
                "missing-or-wrong-source-key-gate-key",
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
                "recorded-direct-rust-not-source-gated",
                "recorded-wrong-rustup-home",
                "recorded-missing-cargo-home",
                "recorded-missing-rustup-toolchain",
                "recorded-wrong-target-dir",
                "recorded-missing-source-key-gate-provenance",
                "recorded-missing-source-key-gate-key",
                "recorded-unsafe-build-jobs",
                "recorded-missing-test-threads",
                "recorded-missing-incremental",
            },
        )

    def test_recorded_literal_rust_tools_fail_closed_without_profile(
        self,
    ) -> None:
        for command in (
            "cargo test --workspace",
            "rustc --version",
            "cargo-clippy --version",
            "cargo-fmt --version",
            "(cargo test --workspace)",
            "if cargo test --workspace; then :; fi",
            "time cargo test --workspace",
            "bash -c 'x=cargo; $x test'",
            "bash -c 'x=ca;y=rgo;$x$y test'",
            (
                "python3 -c "
                "'import subprocess;"
                "subprocess.run([\"ca\"+\"rgo\",\"test\"])'"
            ),
            "env CARGO=/path/cargo sh -c '$CARGO test'",
        ):
            with self.subTest(command=command):
                codes = {
                    finding["code"]
                    for finding in audit_recorded_command(
                        command,
                        cwd=Path("/workspace/worktrees/candidate"),
                    )
                }
                self.assertIn("recorded-missing-rustup-home", codes)
                self.assertIn("recorded-missing-target-dir", codes)
                self.assertIn(
                    "recorded-missing-source-key-gate-provenance",
                    codes,
                )
                self.assertIn(
                    "recorded-direct-rust-not-source-gated",
                    codes,
                )

    def test_recorded_dynamic_dispatch_is_always_unresolved(self) -> None:
        commands = (
            r"bash -c 'c\argo test'",
            "bash -c 'ca\"\"rgo test'",
            "dash -c 'printf safe'",
            "ksh -c 'printf safe'",
            "sh -c 'printf safe'",
            "zsh -c 'printf safe'",
            "bash -O extglob -c 'printf safe'",
            "perl -e 'print \"safe\"'",
            "perl5 -we 'print \"safe\"'",
            "ruby -e 'puts \"safe\"'",
            "python -c 'print(\"safe\")'",
            "python3.12 -c 'print(\"safe\")'",
            "python3 -X dev -c 'print(\"safe\")'",
            "env -a fake dash -c 'printf safe'",
            "timeout -s TERM 1 ksh -c 'printf safe'",
            "nice -n 5 python3 -X dev -c 'print(\"safe\")'",
        )
        for command in commands:
            with self.subTest(command=command):
                codes = {
                    finding["code"]
                    for finding in audit_recorded_command(
                        command,
                        cwd=Path("/workspace/worktrees/candidate"),
                    )
                }
                self.assertIn(
                    "recorded-unresolved-dynamic-dispatch",
                    codes,
                )
                self.assertIn(
                    "recorded-direct-rust-not-source-gated",
                    codes,
                )

    def test_recorded_complete_profile_does_not_attest_direct_cargo(
        self,
    ) -> None:
        source_key = "a" * 64
        command = " ".join(
            (
                f"RUSTUP_HOME={DEFAULT_PROFILE['rustup_home']}",
                f"CARGO_HOME={DEFAULT_PROFILE['cargo_home']}",
                f"RUSTUP_TOOLCHAIN={DEFAULT_PROFILE['rustup_toolchain']}",
                (
                    f"CARGO_TARGET_DIR={DEFAULT_PROFILE['target_dir']}/"
                    f"{SOURCE_KEY_TARGET_DIRECTORY}/{source_key}"
                ),
                "CARGO_BUILD_JOBS=2",
                "RUST_TEST_THREADS=2",
                "CARGO_INCREMENTAL=0",
                (
                    "CODEFIRE_CARGO_GATE_PROVENANCE="
                    "codefire.source_keyed_cargo_gate.v1"
                ),
                f"CODEFIRE_CARGO_GATE_SOURCE_KEY={source_key}",
                "cargo test --workspace",
            )
        )
        self.assertEqual(
            {
                finding["code"]
                for finding in audit_recorded_command(
                    command,
                    cwd=Path("/workspace/worktrees/candidate"),
                )
            },
            {"recorded-direct-rust-not-source-gated"},
        )

    def test_recorded_isolated_source_gate_wrapper_is_attested(self) -> None:
        command = (
            f"{SOURCE_GATE_PYTHON} -I -S -B -X "
            "pycache_prefix=/dev/null "
            f"{TOOLS_DIR / 'run_source_keyed_cargo_gate.py'} "
            "--repo /workspace/worktrees/candidate "
            "--log /workspace/.cache/codefire-gate-evidence/gate.log "
            "--result /workspace/.cache/codefire-gate-evidence/gate.json "
            "-- cargo test --workspace"
        )
        self.assertEqual(
            audit_recorded_command(
                command,
                cwd=Path("/workspace/worktrees/candidate"),
            ),
            [],
        )
        nonisolated = command.replace(
            (
                f"{SOURCE_GATE_PYTHON} -I -S -B -X "
                "pycache_prefix=/dev/null"
            ),
            str(SOURCE_GATE_PYTHON),
            1,
        )
        codes = {
            finding["code"]
            for finding in audit_recorded_command(
                nonisolated,
                cwd=Path("/workspace/worktrees/candidate"),
            )
        }
        self.assertIn("recorded-missing-rustup-home", codes)
        self.assertIn(
            "recorded-direct-rust-not-source-gated",
            codes,
        )
        script = TOOLS_DIR / "run_source_keyed_cargo_gate.py"
        for bypass in (
            (
                f"{SOURCE_GATE_PYTHON} -I -S "
                "-c 'print(\"bypass\")' "
                f"{script} -- cargo test"
            ),
            (
                f"{SOURCE_GATE_PYTHON} -I -S "
                f"-m runpy {script} -- cargo test"
            ),
            (
                f"{SOURCE_GATE_PYTHON} -I -S "
                f"extra {script} -- cargo test"
            ),
            (
                f"{SOURCE_GATE_PYTHON} -I -S "
                f"\"$(printf %s {script})\" -- cargo test"
            ),
            (
                f"{SOURCE_GATE_PYTHON} -I -S "
                f"{script} -- cargo test "
                '"$(cargo --version)"'
            ),
            (
                f"{SOURCE_GATE_PYTHON} -I -S "
                f"{script} -- cargo test "
                '"`cargo --version`"'
            ),
            (
                f"{SOURCE_GATE_PYTHON} -I -S "
                f"{script} -- cargo test "
                "<(cargo --version)"
            ),
            (
                f"LD_PRELOAD=/tmp/evil.so "
                f"{SOURCE_GATE_PYTHON} -I -S "
                f"{script} -- cargo test"
            ),
            (
                f"python3 -I -S {script} -- cargo test"
            ),
        ):
            with self.subTest(bypass=bypass):
                bypass_codes = {
                    finding["code"]
                    for finding in audit_recorded_command(
                        bypass,
                        cwd=Path("/workspace/worktrees/candidate"),
                    )
                }
                self.assertIn(
                    "recorded-missing-rustup-home",
                    bypass_codes,
                )
                self.assertIn(
                    "recorded-direct-rust-not-source-gated",
                    bypass_codes,
                )

    def test_unknown_wrapper_option_arity_cannot_hide_rust_tools(
        self,
    ) -> None:
        commands = (
            "command -- cargo test",
            "command -p cargo test",
            "exec -a fake cargo test",
            "exec -- cargo test",
            "nohup -- cargo test",
            "env --argv0 fake cargo test",
            "env -C /tmp cargo test",
            "time -o /tmp/t cargo test",
            "timeout -s KILL 1 cargo test",
            "timeout -k 1 2 cargo test",
            "rustup run stable cargo test",
        )
        for command in commands:
            with self.subTest(command=command):
                codes = {
                    finding["code"]
                    for finding in audit_recorded_command(
                        command,
                        cwd=Path("/workspace/worktrees/candidate"),
                    )
                }
                self.assertIn("recorded-missing-rustup-home", codes)
                self.assertIn(
                    "recorded-direct-rust-not-source-gated",
                    codes,
                )

    def test_profile_result_publisher_rejects_final_symlink(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_raw:
            root = Path(temporary_raw)
            victim = root / "victim.json"
            victim.write_text("unchanged\n", encoding="utf-8")
            result = root / "result.json"
            result.symlink_to(victim)
            with self.assertRaises(FileExistsError):
                atomic_write_rust_profile_json(
                    result,
                    {"decision": "PASS"},
                )
            self.assertEqual(
                victim.read_text(encoding="utf-8"),
                "unchanged\n",
            )
            self.assertTrue(result.is_symlink())

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
                            "severity": "error",
                            "code": "forged-retained-finding",
                            "location": "result",
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
            # The accepted source-keyed hardening removed implicit
            # log-parent creation from the runner: `open_exclusive_output`
            # now requires an existing, symlink-free parent directory. The
            # in-tree evidence directory this case needs is therefore
            # created by the fixture rather than by the gate.
            (repo.root / "evidence").mkdir()
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
                self.assertIn("unsafe-authored-repository-path", codes)
                self.assertIn("unsafe-artifact-path", codes)

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
                    "mismatched-manifest",
                    lambda item: item["arms"][0]["repair_manifest"].__setitem__(
                        "sha256", "0" * 64
                    ),
                    "live-arm-pin-mismatch",
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
                ),
                [],
            )

            missing = copy.deepcopy(retained_result)
            missing["live_arm_pin"] = "unavailable"
            missing_findings = verify_retained_live_arm_anchor(
                validator_root,
                "cargo",
                record,
                artifacts,
                missing,
                historical_gate,
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
                lambda result: result.__setitem__("live_arm_pin", "unavailable"),
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
            (110, 18, 128),
        )
        self.assertEqual(
            (
                realistic["record"]["total"],
                realistic["manifest"]["total"],
                realistic["combined_total"],
            ),
            (312, 43, 355),
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
            payload = {"base_revision": "a" * 40}
            with mock.patch(
                "probe_validation_capabilities.git_head", return_value="a" * 40
            ), mock.patch(
                "probe_validation_capabilities."
                "validate_locked_canonical_provenance",
                return_value=(payload, {}, []),
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
