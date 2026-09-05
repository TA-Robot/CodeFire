"""Offline regression tests for native/external role isolation and recovery policy."""

from __future__ import annotations

import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import validate_ai_runtime as runtime
from check_ai_acceleration_docs import check_provider_routing, check_role_routing
from run_ai_role import build_command


ROOT = Path(__file__).resolve().parents[3]
PROVIDER_MATRIX = Path("docs/development/ai/provider-routing-matrix.json")


class RuntimeSeparationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix=".runtime-test-", dir=ROOT)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        for relative in (runtime.MATRIX_PATH, PROVIDER_MATRIX, runtime.PROJECT_CONFIG):
            destination = self.root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / relative, destination)
        for relative in (runtime.AGENT_CONFIG_DIR, runtime.EXTERNAL_CONFIG_DIR):
            shutil.copytree(ROOT / relative, self.root / relative)
        self.matrix = json.loads((self.root / runtime.MATRIX_PATH).read_text())

    def save_matrix(self) -> None:
        (self.root / runtime.MATRIX_PATH).write_text(json.dumps(self.matrix))

    def codes(self) -> set[str]:
        self.save_matrix()
        return {item.code for item in runtime.run(self.root, check_cli=False, timeout=1)}

    def external(self) -> dict:
        return next(role for role in self.matrix["roles"] if role["role"] == "implementer")

    def test_native_and_external_profiles_pass_without_cli(self) -> None:
        self.assertEqual(self.codes(), set())

    def test_provider_model_mismatch_is_rejected(self) -> None:
        self.external()["provider"] = "claude"
        self.assertIn("wrong-role-model", self.codes())

    def test_external_role_cannot_generate_codex_command(self) -> None:
        with self.assertRaisesRegex(ValueError, "verified subscription launcher"):
            build_command(self.matrix, self.external(), "high", self.root, "read-only", False, True)

    def test_native_role_keeps_native_command(self) -> None:
        role = next(role for role in self.matrix["roles"] if role["role"] == "manager")
        command = build_command(self.matrix, role, "ultra", self.root, "read-only", False, True)
        self.assertEqual(command[:2], ["codex", "exec"])
        self.assertEqual(command[command.index("--model") + 1], "gpt-5.6-sol")
        self.assertIn('model_reasoning_effort="ultra"', command)

    def test_external_dry_run_refuses_before_any_cli_probe(self) -> None:
        import run_ai_role
        with patch.object(sys, "argv", ["run_ai_role.py", "--root", str(self.root), "--role", "implementer", "--prompt", "test"]), patch.object(run_ai_role, "probe") as probe, patch.object(run_ai_role.subprocess, "run") as execute, patch.object(sys, "stderr") as stderr:
            self.assertEqual(run_ai_role.main(), 2)
        probe.assert_not_called()
        execute.assert_not_called()
        self.assertIn("verified subscription launcher", "".join(str(call.args[0]) for call in stderr.write.call_args_list))

    def test_execute_refuses_even_explicit_allow_before_any_cli_probe(self) -> None:
        import run_ai_role
        for role in ("manager", "architect"):
            with self.subTest(role=role), patch.object(sys, "argv", [
                "run_ai_role.py", "--root", str(self.root), "--role", role,
                "--prompt", "test", "--execute", "--allow-codex", "--allow-untyped-execution",
            ]), patch.object(run_ai_role, "load_matrix") as load_matrix, patch.object(run_ai_role, "probe") as probe, patch.object(run_ai_role.subprocess, "run") as execute, patch.object(sys, "stderr") as stderr:
                self.assertEqual(run_ai_role.main(), 2)
            load_matrix.assert_not_called()
            probe.assert_not_called()
            execute.assert_not_called()
            self.assertIn("verified Codex billing preflight", "".join(str(call.args[0]) for call in stderr.write.call_args_list))

    def test_missing_provider_is_not_inferred(self) -> None:
        del self.external()["provider"]
        self.assertIn("wrong-role-provider", self.codes())

    def test_external_profile_cannot_be_registered_as_native(self) -> None:
        path = self.root / runtime.PROJECT_CONFIG
        path.write_text(path.read_text() + '\n[agents.implementer]\nconfig_file = "../docs/development/ai/agents/grok-4.6.toml"\n')
        self.assertIn("role-config-set-mismatch", self.codes())

    def test_native_role_cannot_reference_external_directory(self) -> None:
        self.matrix["roles"][0]["config_file"] = self.external()["config_file"]
        self.assertIn("wrong-role-config-scope", self.codes())

    def test_external_role_cannot_reference_native_layer(self) -> None:
        self.external()["config_file"] = ".codex/agents/high.toml"
        self.assertIn("wrong-role-config-scope", self.codes())

    def test_symlink_cannot_escape_profile_directory(self) -> None:
        path = self.root / runtime.EXTERNAL_CONFIG_DIR / "escaped.toml"
        path.symlink_to(self.root / runtime.AGENT_CONFIG_DIR / "high.toml")
        self.external()["config_file"] = path.relative_to(self.root).as_posix()
        self.assertIn("wrong-role-config-scope", self.codes())

    def test_external_layer_provider_and_unknown_keys_are_rejected(self) -> None:
        path = self.root / self.external()["config_file"]
        path.write_text('model = "grok-4.6"\nprovider = "claude"\nrole_class = "individual_or_audit"\nmodel_reasoning_effort = "high"\n')
        self.assertTrue({"role-layer-provider-mismatch", "unknown-external-profile-key"}.issubset(self.codes()))

    def test_malformed_external_toml_is_rejected(self) -> None:
        (self.root / self.external()["config_file"]).write_text('provider = [')
        self.assertIn("invalid-toml", self.codes())

    def test_native_tier_mismatch_is_rejected(self) -> None:
        path = self.root / runtime.AGENT_CONFIG_DIR / "ultra.toml"
        path.write_text('model = "gpt-5.6-sol"\nmodel_reasoning_effort = "high"\n')
        self.assertIn("role-layer-tier-mismatch", self.codes())

    def test_fable_alternate_requires_explicit_selection(self) -> None:
        self.external()["alternate_requires_explicit_selection"] = False
        self.assertIn("unguarded-alternate", self.codes())

    def test_alternate_profile_is_validated(self) -> None:
        (self.root / self.external()["alternate_config_file"]).write_text('model = "grok-4.6"\nprovider = "claude"\nrole_class = "individual_or_audit"\n')
        self.assertIn("role-layer-model-mismatch", self.codes())

    def test_malformed_role_collections_report_findings(self) -> None:
        for value in (None, {}, "roles"):
            with self.subTest(value=value):
                self.matrix["roles"] = value
                self.assertIn("missing-roles", self.codes())

    def test_unhashable_provider_and_tiers_report_findings(self) -> None:
        self.external()["provider"] = []
        self.external()["reasoning_tier"] = {}
        self.matrix["runtime_policy"]["policy_tiers"] = [[]]
        self.assertTrue({"wrong-role-provider", "wrong-role-tier", "wrong-policy-tiers"}.issubset(self.codes()))

    def test_missing_installed_adapter_remains_an_error(self) -> None:
        del self.matrix["runtime_policy"]["adapters"]["codex_cli_0_153"]
        self.assertIn("wrong-adapter", self.codes())

    def test_cli_catalog_and_strict_probes_are_required(self) -> None:
        calls = []

        def command(args, **kwargs):
            calls.append(args)
            if args[1:] == ["--version"]:
                return subprocess.CompletedProcess(args, 0, "codex-cli 0.153.0", "")
            if args[1:3] == ["debug", "models"]:
                catalog = {"models": [{"slug": runtime.EXPECTED_MODEL, "supported_reasoning_levels": [{"effort": "medium"}, {"effort": "high"}]}]}
                return subprocess.CompletedProcess(args, 0, json.dumps(catalog), "")
            self.assertEqual(kwargs["input"], "")
            self.assertIn("--strict-config", args)
            return subprocess.CompletedProcess(args, 1, "", "unknown strict config")

        with patch.object(runtime.shutil, "which", return_value="codex"), patch.object(runtime.subprocess, "run", side_effect=command):
            findings = []
            runtime.check_codex_installation(self.root, findings, 1)
        codes = {finding.code for finding in findings}
        self.assertTrue({"codex-strict-config-unusable", "config-layer-strict-unusable", "codex-tier-missing"}.issubset(codes))
        self.assertEqual(len(calls), 9)

    def test_role_policy_rejects_mismatched_provider_even_for_valid_model(self) -> None:
        self.external()["provider"] = "claude"
        self.save_matrix()
        findings = []
        check_role_routing(self.root, findings)
        self.assertIn("bad-role-model", {item.code for item in findings})

    def test_provider_policy_passes_and_rejects_paid_or_automatic_fable(self) -> None:
        path = self.root / PROVIDER_MATRIX
        data = json.loads(path.read_text())
        findings = []
        check_provider_routing(self.root, findings)
        self.assertEqual(findings, [])
        data["billing"]["extra_usage_allowed"] = True
        data["tier_routes"]["medium"]["alternate"]["requires_explicit_selection"] = False
        path.write_text(json.dumps(data))
        findings = []
        check_provider_routing(self.root, findings)
        self.assertTrue({"unsafe-billing-policy", "unguarded-fable-alternate"}.issubset({item.code for item in findings}))


if __name__ == "__main__":
    unittest.main()
