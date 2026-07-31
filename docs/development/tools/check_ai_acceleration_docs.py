import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class Finding:
    level: str
    code: str
    path: str
    message: str


REQUIRED_AI_FILES = [
    "docs/development/ai/README.md",
    "docs/development/ai/context-packs.json",
    "docs/development/ai/operating-protocol.md",
    "docs/development/ai/role-routing-matrix.json",
    "docs/development/ai/provider-routing-matrix.json",
    "docs/development/ai/provider-routing-policy.md",
    "docs/development/ai/model-routing-policy.md",
    "docs/development/ai/acceleration-master-plan.md",
    "docs/development/ai/stage5-manual-control-runbook.md",
    "docs/development/ai/development-flow-kgi-kpi.md",
    "docs/development/ai/schemas/development-flow-record.v1.schema.json",
    "docs/development/ai/capability-registry.v1.json",
    "docs/development/ai/focused-test-map.v1.json",
    "docs/development/ai/evals/historical-cases.v1.json",
    "docs/development/ai/evals/live-routing-smoke-2026-07-11.v1.json",
    "docs/development/tools/prepare_historical_eval.py",
    "docs/development/tools/build_source_bound_rust.py",
    "docs/development/tools/capture_candidate_identity.py",
    "docs/development/tools/check_swarm_admission.py",
    "docs/development/tools/run_timed_gate.py",
    "docs/development/exploration/README.md",
    ".codex/config.toml",
    ".codex/agents/medium.toml",
    ".codex/agents/high.toml",
    ".codex/agents/ultra.toml",
]


REQUIRED_ROLES = {
    "manager",
    "architect",
    "implementer",
    "reviewer",
    "researcher",
    "docs_synchronizer",
    "validation_runner",
    "dogfood_runner",
    "classifier",
}


TASK_PACKET_SECTIONS = [
    "Status:",
    "## Objective",
    "## Scope",
    "## Non-Goals",
    "## Validation",
    "## Handoff",
]


ACTIVE_STATE_FILES = [
    "README.md",
    "AGENTS.md",
    "docs/development/README.md",
    "docs/development/project-management.md",
    "docs/development/current-cycle-burn-down-plan.md",
    "docs/development/codefire-improvement-cycle.md",
    "docs/development/todo-checklist.md",
    "docs/development/worktree-cleanup-plan.md",
    "docs/development/document-consolidation-plan.md",
]


STALE_PATTERNS = [
    (re.compile(r"\b104\s*\|\s*7\b"), "stale CFB 104/7 count table"),
    (re.compile(r"\b271\s*\|\s*264\s*\|\s*7\b"), "stale aggregate 271/264/7 count table"),
    (re.compile(r"Current detail issue status count"), "stale current detail status heading"),
    (re.compile(r"Active frozen issue batch"), "stale active frozen batch heading"),
]


DEV_SWARM_V1_STAGE5_FIELDS = frozenset(
    {
        "scheduling",
        "resource_class",
        "execution_profile",
        "progress_events",
        "hedge_of",
        "continuation_of",
        "supersedes",
        "candidate_event",
        "review_rollover",
        "validation_cache",
        "shadow_integration",
    }
)

ACTIVE_RUST_PROFILE_MARKERS = (
    "RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup",
    "CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo",
    "CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared",
    "CARGO_BUILD_JOBS=2",
    "RUST_TEST_THREADS=2",
    "CARGO_INCREMENTAL=0",
)


def path_has_glob(path_ref: str) -> bool:
    return any(char in path_ref for char in "*?[")


def reference_exists(root: Path, path_ref: str) -> bool:
    if path_has_glob(path_ref):
        return any(root.glob(path_ref))
    candidate = root / path_ref
    if path_ref.endswith("/"):
        return candidate.is_dir()
    return candidate.exists()


def load_json(root: Path, relative_path: str, findings: list[Finding]) -> dict[str, Any] | None:
    path = root / relative_path
    if not path.exists():
        findings.append(Finding("error", "missing-json", relative_path, "JSON file is missing"))
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        findings.append(Finding("error", "invalid-json", relative_path, f"{exc.msg} at line {exc.lineno}"))
        return None
    if not isinstance(data, dict):
        findings.append(Finding("error", "invalid-json-root", relative_path, "JSON root must be an object"))
        return None
    return data


def check_required_files(root: Path, findings: list[Finding]) -> None:
    for relative_path in REQUIRED_AI_FILES:
        if not (root / relative_path).is_file():
            findings.append(Finding("error", "missing-ai-file", relative_path, "Required AI acceleration file is missing"))


def check_context_packs(root: Path, findings: list[Finding]) -> None:
    relative_path = "docs/development/ai/context-packs.json"
    data = load_json(root, relative_path, findings)
    if data is None:
        return
    if data.get("schema") != "codefire.ai_context_packs.v1":
        findings.append(Finding("error", "bad-schema", relative_path, "Unexpected context-pack schema"))
    packs = data.get("packs")
    if not isinstance(packs, list) or not packs:
        findings.append(Finding("error", "missing-packs", relative_path, "packs must be a non-empty list"))
        return
    ids: set[str] = set()
    for index, pack in enumerate(packs):
        path = f"{relative_path}:packs[{index}]"
        if not isinstance(pack, dict):
            findings.append(Finding("error", "bad-pack", path, "pack must be an object"))
            continue
        pack_id = pack.get("id")
        if not isinstance(pack_id, str) or not pack_id:
            findings.append(Finding("error", "missing-pack-id", path, "pack id is required"))
        elif pack_id in ids:
            findings.append(Finding("error", "duplicate-pack-id", path, f"Duplicate pack id: {pack_id}"))
        else:
            ids.add(pack_id)
        for key in ("required_files", "likely_files"):
            refs = pack.get(key, [])
            if refs is None:
                refs = []
            if not isinstance(refs, list):
                findings.append(Finding("error", "bad-path-list", path, f"{key} must be a list"))
                continue
            for path_ref in refs:
                if not isinstance(path_ref, str) or not path_ref:
                    findings.append(Finding("error", "bad-path-ref", path, f"{key} contains an invalid path"))
                    continue
                if not reference_exists(root, path_ref):
                    findings.append(Finding("error", "missing-path-ref", path_ref, f"Referenced by {pack_id or path} {key}"))
    for index, pack in enumerate(packs):
        if not isinstance(pack, dict):
            continue
        for parent in pack.get("extends", []):
            if parent not in ids:
                findings.append(Finding("error", "missing-pack-parent", f"{relative_path}:packs[{index}]", f"Unknown parent pack: {parent}"))


def check_role_routing(root: Path, findings: list[Finding]) -> None:
    relative_path = "docs/development/ai/role-routing-matrix.json"
    data = load_json(root, relative_path, findings)
    if data is None:
        return
    if data.get("schema") != "codefire.ai_role_routing.v2":
        findings.append(Finding("error", "bad-schema", relative_path, "Unexpected role-routing schema"))
    runtime_policy = data.get("runtime_policy")
    if not isinstance(runtime_policy, dict):
        findings.append(Finding("error", "missing-runtime-policy", relative_path, "runtime_policy is required"))
    else:
        if runtime_policy.get("primary_model") != "gpt-5.6-sol":
            findings.append(Finding("error", "bad-primary-model", relative_path, "primary_model must be gpt-5.6-sol"))
        tiers = runtime_policy.get("policy_tiers")
        if not isinstance(tiers, list) or set(tiers) != {"medium", "high", "ultra"}:
            findings.append(Finding("error", "bad-policy-tiers", relative_path, "policy_tiers must be medium/high/ultra"))
        adapters = runtime_policy.get("adapters")
        if not isinstance(adapters, dict):
            findings.append(Finding("error", "missing-runtime-adapters", relative_path, "runtime adapters are required"))
    roles = data.get("roles")
    if not isinstance(roles, list) or not roles:
        findings.append(Finding("error", "missing-roles", relative_path, "roles must be a non-empty list"))
        return
    seen: set[str] = set()
    for index, role in enumerate(roles):
        path = f"{relative_path}:roles[{index}]"
        if not isinstance(role, dict):
            findings.append(Finding("error", "bad-role", path, "role must be an object"))
            continue
        role_name = role.get("role")
        if not isinstance(role_name, str) or not role_name:
            findings.append(Finding("error", "missing-role-name", path, "role name is required"))
            continue
        if role_name in seen:
            findings.append(Finding("error", "duplicate-role", path, f"Duplicate role: {role_name}"))
        seen.add(role_name)
        if role.get("model") != "gpt-5.6-sol":
            findings.append(Finding("error", "bad-role-model", path, f"{role_name} must use gpt-5.6-sol"))
        tier = role.get("reasoning_tier")
        if tier not in {"medium", "high", "ultra"}:
            findings.append(Finding("error", "missing-reasoning-tier", path, f"{role_name} needs medium/high/ultra reasoning_tier"))
        config_file = role.get("config_file")
        if not isinstance(config_file, str) or not config_file or not (root / config_file).is_file():
            findings.append(Finding("error", "missing-role-config", path, f"{role_name} needs an existing config_file"))
        outputs = role.get("outputs")
        if not isinstance(outputs, list) or not outputs or not all(isinstance(item, str) and item for item in outputs):
            findings.append(Finding("error", "missing-role-outputs", path, f"{role_name} needs non-empty string outputs"))
    missing_roles = sorted(REQUIRED_ROLES - seen)
    for role_name in missing_roles:
        findings.append(Finding("error", "missing-required-role", relative_path, f"Missing role: {role_name}"))


def check_provider_routing(root: Path, findings: list[Finding]) -> None:
    relative_path = "docs/development/ai/provider-routing-matrix.json"
    data = load_json(root, relative_path, findings)
    if data is None:
        return
    if data.get("schema") != "codefire.ai_provider_routing.v1":
        findings.append(Finding("error", "bad-provider-schema", relative_path, "Unexpected provider-routing schema"))
    routes = data.get("tier_routes")
    expected = {"medium": "fugu", "high": "claude", "ultra": "codex"}
    if not isinstance(routes, dict):
        findings.append(Finding("error", "missing-tier-routes", relative_path, "tier_routes is required"))
    else:
        for tier, provider in expected.items():
            route = routes.get(tier)
            if not isinstance(route, dict) or route.get("provider") != provider:
                findings.append(
                    Finding(
                        "error",
                        "bad-provider-tier-route",
                        relative_path,
                        f"{tier} must route to {provider}",
                    )
                )
        medium = routes.get("medium")
        if isinstance(medium, dict) and medium.get("model") != "fugu-ultra-v1.1":
            findings.append(
                Finding(
                    "error",
                    "bad-fugu-provider-model",
                    relative_path,
                    "Medium Fugu must use fugu-ultra-v1.1",
                )
            )
        high = routes.get("high")
        if isinstance(high, dict):
            models = high.get("models")
            expected_claude_models = {
                "implementation": "claude-opus-5",
                "review_or_research": "claude-opus-5",
            }
            if not isinstance(models, dict) or models != expected_claude_models:
                findings.append(
                    Finding(
                        "error",
                        "bad-claude-provider-models",
                        relative_path,
                        "High Claude implementation and review/research must use claude-opus-5",
                    )
                )
        ultra = routes.get("ultra")
        if isinstance(ultra, dict):
            if ultra.get("model") != "gpt-5.6-sol":
                findings.append(Finding("error", "bad-codex-provider-model", relative_path, "Ultra Codex must use gpt-5.6-sol"))
            if ultra.get("requires_explicit_allow_codex") is not True:
                findings.append(Finding("error", "unguarded-codex-route", relative_path, "Ultra Codex must require explicit allow"))
    concurrency = data.get("concurrency")
    if not isinstance(concurrency, dict):
        findings.append(Finding("error", "missing-provider-concurrency", relative_path, "concurrency is required"))
    else:
        for provider in ("fugu", "claude", "codex"):
            value = concurrency.get(provider)
            if not isinstance(value, int) or isinstance(value, bool) or value < 1:
                findings.append(Finding("error", "bad-provider-concurrency", relative_path, f"{provider} must be a positive integer"))
        if isinstance(concurrency.get("codex"), int) and concurrency["codex"] != 1:
            findings.append(Finding("error", "codex-concurrency-not-one", relative_path, "Codex control plane must stay at one lane"))
    concurrency_control = data.get("concurrency_control")
    if not isinstance(concurrency_control, dict):
        findings.append(
            Finding(
                "error",
                "missing-concurrency-control",
                relative_path,
                "adaptive concurrency_control is required",
            )
        )
    else:
        if concurrency_control.get("mode") != "adaptive_window":
            findings.append(
                Finding(
                    "error",
                    "fixed-provider-concurrency",
                    relative_path,
                    "provider concurrency must use adaptive_window",
                )
            )
        if concurrency_control.get("total_session_cap") is not None:
            findings.append(
                Finding(
                    "error",
                    "fixed-total-session-cap",
                    relative_path,
                    "total_session_cap must stay null",
                )
            )
        admission_tool = concurrency_control.get("admission_tool")
        if (
            not isinstance(admission_tool, str)
            or not admission_tool
            or not (root / admission_tool).is_file()
        ):
            findings.append(
                Finding(
                    "error",
                    "missing-admission-tool",
                    relative_path,
                    "concurrency_control must reference the admission tool",
                )
            )
    fallback = data.get("fallback")
    if not isinstance(fallback, dict) or fallback.get("automatic_codex_fallback") is not False:
        findings.append(Finding("error", "automatic-codex-fallback", relative_path, "automatic Codex fallback must be false"))
    role_defaults = data.get("role_defaults")
    if not isinstance(role_defaults, dict):
        findings.append(Finding("error", "missing-provider-role-defaults", relative_path, "role_defaults is required"))
    else:
        required = {
            "implementer": "claude",
            "reviewer": "claude",
            "classifier": "fugu",
            "validation_runner": "fugu",
            "integrator": "codex",
        }
        for role, provider in required.items():
            if role_defaults.get(role) != provider:
                findings.append(
                    Finding(
                        "error",
                        "bad-provider-role-default",
                        relative_path,
                        f"{role} must default to {provider}",
                    )
                )


def check_task_packets(root: Path, findings: list[Finding]) -> None:
    packet_dir = root / "docs/development/ai/task-packets"
    if not packet_dir.is_dir():
        findings.append(Finding("error", "missing-task-packet-dir", "docs/development/ai/task-packets", "Task packet directory is missing"))
        return
    packets = sorted(path for path in packet_dir.glob("*.md") if path.name != "README.md")
    if not packets:
        findings.append(Finding("error", "missing-task-packets", "docs/development/ai/task-packets", "No task packets found"))
        return
    for path in packets:
        text = path.read_text(encoding="utf-8")
        relative_path = path.relative_to(root).as_posix()
        for section in TASK_PACKET_SECTIONS:
            if section not in text:
                findings.append(Finding("error", "missing-task-section", relative_path, f"Missing section marker: {section}"))


def check_live_routing_smoke(root: Path, findings: list[Finding]) -> None:
    relative_path = "docs/development/ai/evals/live-routing-smoke-2026-07-11.v1.json"
    data = load_json(root, relative_path, findings)
    if data is None:
        return
    if data.get("schema") != "codefire.ai_live_routing_smoke.v1":
        findings.append(Finding("error", "bad-live-routing-schema", relative_path, repr(data.get("schema"))))
    if data.get("model") != "gpt-5.6-sol":
        findings.append(Finding("error", "bad-live-routing-model", relative_path, repr(data.get("model"))))
    if data.get("sandbox") != "read-only":
        findings.append(Finding("error", "unsafe-live-routing-sandbox", relative_path, repr(data.get("sandbox"))))
    runs = data.get("runs")
    if not isinstance(runs, list):
        findings.append(Finding("error", "bad-live-routing-runs", relative_path, "runs must be an array"))
        return
    by_tier = {
        item.get("policy_tier"): item
        for item in runs
        if isinstance(item, dict) and isinstance(item.get("policy_tier"), str)
    }
    if set(by_tier) != {"medium", "high", "ultra"} or len(runs) != 3:
        findings.append(
            Finding("error", "bad-live-routing-tier-set", relative_path, f"observed {sorted(by_tier)}")
        )
    for tier in ("medium", "high", "ultra"):
        item = by_tier.get(tier)
        if item is None:
            continue
        expected = f"{tier}-live-ok"
        if item.get("cli_reasoning_effort") != tier:
            findings.append(Finding("error", "live-routing-effort-mismatch", relative_path, tier))
        if item.get("exit_code") != 0 or item.get("expected_response") != expected or item.get("observed_response") != expected:
            findings.append(Finding("error", "live-routing-run-failed", relative_path, tier))
        usage = item.get("usage")
        if not isinstance(usage, dict):
            findings.append(Finding("error", "missing-live-routing-usage", relative_path, tier))
            continue
        for key in ("input_tokens", "cached_input_tokens", "output_tokens", "reasoning_output_tokens"):
            value = usage.get(key)
            minimum = 0 if key in {"cached_input_tokens", "reasoning_output_tokens"} else 1
            if not isinstance(value, int) or isinstance(value, bool) or value < minimum:
                findings.append(Finding("error", "bad-live-routing-usage", relative_path, f"{tier}.{key}"))
        if isinstance(usage.get("input_tokens"), int) and isinstance(usage.get("cached_input_tokens"), int):
            if usage["cached_input_tokens"] > usage["input_tokens"]:
                findings.append(Finding("error", "bad-live-routing-cache-count", relative_path, tier))
    limitations = data.get("does_not_prove")
    if not isinstance(limitations, list) or len(limitations) < 3:
        findings.append(Finding("error", "missing-live-routing-limitations", relative_path, "bounded claims are required"))


def check_dev_swarm_v1_stage5_fields(root: Path, findings: list[Finding]) -> None:
    batch_dir = root / "docs/development/ai/swarm-batches"
    if not batch_dir.is_dir():
        return

    for path in sorted(batch_dir.glob("*.json")):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if not isinstance(data, dict) or data.get("schema") != "dev-swarm.v1":
            continue

        relative_path = path.relative_to(root).as_posix()

        def visit(value: Any, pointer: str) -> None:
            if isinstance(value, dict):
                for key, child in value.items():
                    child_pointer = f"{pointer}.{key}" if pointer else key
                    if key in DEV_SWARM_V1_STAGE5_FIELDS:
                        findings.append(
                            Finding(
                                "error",
                                "unsupported-stage5-field-in-dev-swarm-v1",
                                f"{relative_path}:{child_pointer}",
                                "Stage 5 controls require fail-closed dev-swarm.v2; current v1 may ignore this field",
                            )
                        )
                    visit(child, child_pointer)
            elif isinstance(value, list):
                for index, child in enumerate(value):
                    visit(child, f"{pointer}[{index}]")

        visit(data, "")


def check_active_rust_packet_profiles(root: Path, findings: list[Finding]) -> None:
    batch_dir = root / "docs/development/ai/swarm-batches"
    if not batch_dir.is_dir():
        return
    for path in sorted(batch_dir.glob("cfr168-rust-*.md")):
        text = path.read_text(encoding="utf-8")
        if not re.search(r"^Status: (?:blocked|ready)\b", text, flags=re.MULTILINE):
            continue
        if "## Validation" not in text or "cargo " not in text.lower():
            continue
        missing = [marker for marker in ACTIVE_RUST_PROFILE_MARKERS if marker not in text]
        if missing:
            findings.append(
                Finding(
                    "error",
                    "active-rust-packet-missing-shared-profile",
                    path.relative_to(root).as_posix(),
                    f"missing exact shared low-memory marker(s): {', '.join(missing)}",
                )
            )


def check_stale_active_state(root: Path, findings: list[Finding]) -> None:
    active_paths = [root / path for path in ACTIVE_STATE_FILES]
    active_paths.extend((root / "docs/development/ai").rglob("*.md"))
    for path in sorted(set(active_paths)):
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        relative_path = path.relative_to(root).as_posix()
        for pattern, message in STALE_PATTERNS:
            if pattern.search(text):
                findings.append(Finding("error", "stale-active-state", relative_path, message))


def run(root: Path) -> list[Finding]:
    findings: list[Finding] = []
    check_required_files(root, findings)
    check_context_packs(root, findings)
    check_role_routing(root, findings)
    check_provider_routing(root, findings)
    check_task_packets(root, findings)
    check_live_routing_smoke(root, findings)
    check_dev_swarm_v1_stage5_fields(root, findings)
    check_active_rust_packet_profiles(root, findings)
    check_stale_active_state(root, findings)
    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description="Check CodeFire AI acceleration docs.")
    parser.add_argument("--root", default=".", help="Repository root. Defaults to current directory.")
    parser.add_argument("--json", action="store_true", help="Print machine-readable findings.")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    findings = run(root)
    errors = [finding for finding in findings if finding.level == "error"]

    if args.json:
        print(json.dumps({"ok": not errors, "findings": [asdict(finding) for finding in findings]}, indent=2, sort_keys=True))
    elif errors:
        print(f"AI acceleration docs check failed: {len(errors)} error(s)")
        for finding in errors:
            print(f"- {finding.code}: {finding.path}: {finding.message}")
    else:
        print("AI acceleration docs check passed.")

    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
