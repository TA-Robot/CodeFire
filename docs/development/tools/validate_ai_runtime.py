#!/usr/bin/env python3
"""Validate CodeFire's project-scoped Codex model and role routing.

The check is intentionally local and network-free. It proves that repository
configuration and the routing manifest agree, and that the installed Codex
bundle exposes the expected GPT-5.6 Sol Ultra catalog strings. It does not
prove account entitlement or a successful remote model invocation.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

try:
    import tomllib  # type: ignore[import-not-found]
except ModuleNotFoundError:  # Python 3.10 compatibility in the dev container.
    tomllib = None


EXPECTED_MODEL = "gpt-5.6-sol"
EXPECTED_TIERS = {"medium", "high", "ultra"}
MATRIX_PATH = Path("docs/development/ai/role-routing-matrix.json")
PROJECT_CONFIG = Path(".codex/config.toml")
AGENT_CONFIG_DIR = Path(".codex/agents")


@dataclass(frozen=True)
class Finding:
    level: str
    code: str
    path: str
    message: str


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def load_json(path: Path, findings: list[Finding]) -> dict[str, Any] | None:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        findings.append(Finding("error", "missing-file", str(path), "file is missing"))
        return None
    except json.JSONDecodeError as exc:
        findings.append(
            Finding("error", "invalid-json", str(path), f"{exc.msg} at line {exc.lineno}")
        )
        return None
    if not isinstance(data, dict):
        findings.append(Finding("error", "invalid-json-root", str(path), "root must be an object"))
        return None
    return data


def load_toml(path: Path, findings: list[Finding]) -> dict[str, Any] | None:
    try:
        if tomllib is not None:
            with path.open("rb") as handle:
                data = tomllib.load(handle)
        else:
            data = parse_simple_toml(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        findings.append(Finding("error", "missing-file", str(path), "file is missing"))
        return None
    except (ValueError, getattr(tomllib, "TOMLDecodeError", ValueError)) as exc:
        findings.append(Finding("error", "invalid-toml", str(path), str(exc)))
        return None
    if not isinstance(data, dict):
        findings.append(Finding("error", "invalid-toml-root", str(path), "root must be a table"))
        return None
    return data


def parse_simple_toml(text: str) -> dict[str, Any]:
    """Parse the scalar/table subset used by repository Codex configs."""

    root: dict[str, Any] = {}
    table = root
    for line_number, raw_line in enumerate(text.splitlines(), start=1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            parts = line[1:-1].split(".")
            if not parts or any(not part for part in parts):
                raise ValueError(f"invalid table at line {line_number}")
            table = root
            for part in parts:
                child = table.setdefault(part, {})
                if not isinstance(child, dict):
                    raise ValueError(f"table conflicts with scalar at line {line_number}")
                table = child
            continue
        if "=" not in line:
            raise ValueError(f"expected assignment at line {line_number}")
        key, raw_value = (part.strip() for part in line.split("=", 1))
        if not key or key in table:
            raise ValueError(f"invalid or duplicate key at line {line_number}")
        value_text = raw_value.split("#", 1)[0].strip()
        if value_text.startswith('"') and value_text.endswith('"'):
            value: Any = json.loads(value_text)
        elif value_text in {"true", "false"}:
            value = value_text == "true"
        elif re.fullmatch(r"-?[0-9]+", value_text):
            value = int(value_text)
        else:
            raise ValueError(f"unsupported scalar at line {line_number}: {value_text!r}")
        table[key] = value
    return root


def flat_toml_string(path: Path, key: str) -> str | None:
    pattern = re.compile(rf"^{re.escape(key)}\s*=\s*\"([^\"]+)\"\s*(?:#.*)?$")
    for line in path.read_text(encoding="utf-8").splitlines():
        match = pattern.match(line.strip())
        if match:
            return match.group(1)
    return None


def check_project_config(root: Path, findings: list[Finding]) -> None:
    path = root / PROJECT_CONFIG
    if not path.is_file():
        findings.append(Finding("error", "missing-project-config", str(PROJECT_CONFIG), "file is missing"))
        return
    model = flat_toml_string(path, "model")
    effort = flat_toml_string(path, "model_reasoning_effort")
    if model != EXPECTED_MODEL:
        findings.append(
            Finding("error", "wrong-project-model", str(PROJECT_CONFIG), f"expected {EXPECTED_MODEL}, got {model!r}")
        )
    if effort != "ultra":
        findings.append(
            Finding("error", "wrong-project-effort", str(PROJECT_CONFIG), f"expected ultra, got {effort!r}")
        )


def check_matrix(root: Path, findings: list[Finding]) -> set[str]:
    path = root / MATRIX_PATH
    data = load_json(path, findings)
    if data is None:
        return set()
    if data.get("schema") != "codefire.ai_role_routing.v2":
        findings.append(Finding("error", "wrong-schema", str(MATRIX_PATH), "expected v2 schema"))

    runtime = data.get("runtime_policy")
    if not isinstance(runtime, dict):
        findings.append(Finding("error", "missing-runtime-policy", str(MATRIX_PATH), "runtime_policy is required"))
        return set()
    if runtime.get("primary_model") != EXPECTED_MODEL:
        findings.append(
            Finding("error", "wrong-primary-model", str(MATRIX_PATH), f"expected {EXPECTED_MODEL}")
        )
    tiers = runtime.get("policy_tiers")
    if not isinstance(tiers, list) or set(tiers) != EXPECTED_TIERS:
        findings.append(
            Finding("error", "wrong-policy-tiers", str(MATRIX_PATH), "tiers must be medium/high/ultra")
        )

    adapters = runtime.get("adapters")
    if not isinstance(adapters, dict):
        findings.append(Finding("error", "missing-adapters", str(MATRIX_PATH), "adapters are required"))
    else:
        expected_efforts = {
            "codex_cli_0_144": {"medium": "medium", "high": "high", "ultra": "ultra"},
            "codex_cli_0_146": {"medium": "medium", "high": "high", "ultra": "ultra"},
            "openai_responses_api": {"medium": "medium", "high": "high", "ultra": "max"},
            "portable_codex_fallback": {"medium": "medium", "high": "high", "ultra": "xhigh"},
        }
        for name, mapping in expected_efforts.items():
            adapter = adapters.get(name)
            if not isinstance(adapter, dict):
                findings.append(Finding("error", "wrong-adapter", str(MATRIX_PATH), f"missing {name}"))
                continue
            for tier, effort in mapping.items():
                tier_adapter = adapter.get(tier)
                if not isinstance(tier_adapter, dict) or tier_adapter.get("reasoning_effort") != effort:
                    findings.append(
                        Finding("error", "wrong-adapter", str(MATRIX_PATH), f"unexpected {name}.{tier}")
                    )
                if not isinstance(tier_adapter, dict) or not isinstance(tier_adapter.get("delegation"), str):
                    findings.append(
                        Finding("error", "missing-delegation", str(MATRIX_PATH), f"missing delegation for {name}.{tier}")
                    )
        for name in ("openai_responses_api", "portable_codex_fallback"):
            adapter = adapters.get(name)
            ultra = adapter.get("ultra") if isinstance(adapter, dict) else None
            if not isinstance(ultra, dict) or ultra.get("semantic_equivalence") is not False:
                findings.append(
                    Finding("error", "false-ultra-equivalence", str(MATRIX_PATH), f"{name}.ultra must be non-equivalent")
                )

    roles = data.get("roles")
    if not isinstance(roles, list) or not roles:
        findings.append(Finding("error", "missing-roles", str(MATRIX_PATH), "roles must be non-empty"))
        return set()

    seen: set[str] = set()
    used_tiers: set[str] = set()
    for index, role in enumerate(roles):
        role_path = f"{MATRIX_PATH}:roles[{index}]"
        if not isinstance(role, dict):
            findings.append(Finding("error", "invalid-role", role_path, "role must be an object"))
            continue
        name = role.get("role")
        if not isinstance(name, str) or not name:
            findings.append(Finding("error", "missing-role-name", role_path, "role name is required"))
            continue
        if name in seen:
            findings.append(Finding("error", "duplicate-role", role_path, name))
        seen.add(name)
        if role.get("model") != EXPECTED_MODEL:
            findings.append(Finding("error", "wrong-role-model", role_path, name))
        tier = role.get("reasoning_tier")
        if tier not in EXPECTED_TIERS:
            findings.append(Finding("error", "wrong-role-tier", role_path, f"{name}: {tier!r}"))
        else:
            used_tiers.add(tier)
        config_ref = role.get("config_file")
        if not isinstance(config_ref, str) or not (root / config_ref).is_file():
            findings.append(Finding("error", "missing-role-config", role_path, str(config_ref)))
        outputs = role.get("outputs")
        if not isinstance(outputs, list) or not outputs or not all(isinstance(item, str) and item for item in outputs):
            findings.append(Finding("error", "missing-role-outputs", role_path, name))

    if used_tiers != EXPECTED_TIERS:
        findings.append(
            Finding("error", "unused-policy-tier", str(MATRIX_PATH), f"used tiers: {sorted(used_tiers)}")
        )
    return used_tiers


def check_agent_configs(root: Path, findings: list[Finding]) -> None:
    for tier in sorted(EXPECTED_TIERS):
        path = root / AGENT_CONFIG_DIR / f"{tier}.toml"
        relative = path.relative_to(root).as_posix()
        if not path.is_file():
            findings.append(Finding("error", "missing-agent-config", relative, "file is missing"))
            continue
        model = flat_toml_string(path, "model")
        effort = flat_toml_string(path, "model_reasoning_effort")
        if model != EXPECTED_MODEL:
            findings.append(Finding("error", "wrong-agent-model", relative, repr(model)))
        if effort != tier:
            findings.append(Finding("error", "wrong-agent-effort", relative, repr(effort)))


def check_role_config_mapping(root: Path, findings: list[Finding]) -> None:
    project = load_toml(root / PROJECT_CONFIG, findings)
    matrix = load_json(root / MATRIX_PATH, findings)
    if project is None or matrix is None:
        return
    project_agents = project.get("agents")
    if not isinstance(project_agents, dict):
        findings.append(Finding("error", "missing-project-agents", str(PROJECT_CONFIG), "[agents] is required"))
        return
    matrix_roles = {
        role.get("role"): role
        for role in matrix.get("roles", [])
        if isinstance(role, dict) and isinstance(role.get("role"), str)
    }
    configured_roles = {
        name: value
        for name, value in project_agents.items()
        if isinstance(value, dict)
    }
    if set(configured_roles) != set(matrix_roles):
        findings.append(
            Finding(
                "error",
                "role-config-set-mismatch",
                str(PROJECT_CONFIG),
                f"missing={sorted(set(matrix_roles) - set(configured_roles))}, "
                f"extra={sorted(set(configured_roles) - set(matrix_roles))}",
            )
        )
    for name, role in matrix_roles.items():
        configured = configured_roles.get(name)
        if not isinstance(configured, dict):
            continue
        project_ref = configured.get("config_file")
        matrix_ref = role.get("config_file")
        if not isinstance(project_ref, str) or not isinstance(matrix_ref, str):
            findings.append(Finding("error", "bad-role-config-ref", str(PROJECT_CONFIG), name))
            continue
        project_path = (root / PROJECT_CONFIG.parent / project_ref).resolve()
        matrix_path = (root / matrix_ref).resolve()
        if project_path != matrix_path:
            findings.append(
                Finding("error", "role-config-ref-mismatch", str(PROJECT_CONFIG), f"{name}: {project_ref} != {matrix_ref}")
            )
            continue
        layer = load_toml(matrix_path, findings)
        if layer is None:
            continue
        if layer.get("model") != role.get("model"):
            findings.append(Finding("error", "role-layer-model-mismatch", matrix_ref, name))
        if layer.get("model_reasoning_effort") != role.get("reasoning_tier"):
            findings.append(Finding("error", "role-layer-tier-mismatch", matrix_ref, name))


def file_contains_all(path: Path, needles: tuple[bytes, ...]) -> bool:
    remaining = set(needles)
    overlap = max(len(needle) for needle in needles) - 1
    carry = b""
    with path.open("rb") as handle:
        while chunk := handle.read(1024 * 1024):
            data = carry + chunk
            remaining = {needle for needle in remaining if needle not in data}
            if not remaining:
                return True
            carry = data[-overlap:] if overlap else b""
    return False


def native_codex_binaries(codex: str) -> list[Path]:
    entrypoint = Path(codex).resolve()
    if entrypoint.name == "codex.js" and entrypoint.parent.name == "bin":
        package_root = entrypoint.parent.parent
        return sorted(
            path
            for path in package_root.glob("node_modules/@openai/codex-*/vendor/*/bin/codex*")
            if path.is_file()
        )
    return [entrypoint] if entrypoint.is_file() else []


def check_codex_installation(root: Path, findings: list[Finding], timeout: float) -> None:
    codex = shutil.which("codex")
    if codex is None:
        findings.append(Finding("error", "missing-codex", "PATH", "codex executable not found"))
        return
    try:
        result = subprocess.run(
            [codex, "--version"],
            cwd=root,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
    except subprocess.TimeoutExpired:
        findings.append(Finding("error", "codex-version-timeout", codex, f"exceeded {timeout}s"))
        return
    if result.returncode != 0:
        findings.append(
            Finding("error", "codex-version-failed", codex, (result.stderr or result.stdout).strip())
        )
        return

    version_match = re.search(r"codex-cli\s+(\d+)\.(\d+)(?:\.(\d+))?", result.stdout)
    if version_match is None:
        findings.append(Finding("error", "unparseable-codex-version", codex, result.stdout.strip()))
        return
    adapter_key = f"codex_cli_{version_match.group(1)}_{version_match.group(2)}"
    matrix = load_json(root / MATRIX_PATH, findings)
    adapters = matrix.get("runtime_policy", {}).get("adapters", {}) if matrix else {}
    if adapter_key not in adapters:
        findings.append(
            Finding("error", "missing-codex-version-adapter", str(MATRIX_PATH), adapter_key)
        )

    for tier in sorted(EXPECTED_TIERS):
        try:
            strict_probe = subprocess.run(
                [
                    codex,
                    "exec",
                    "--ignore-user-config",
                    "--strict-config",
                    "--model",
                    EXPECTED_MODEL,
                    "--config",
                    f'model_reasoning_effort="{tier}"',
                    "--sandbox",
                    "read-only",
                    "--cd",
                    str(root),
                    "--ephemeral",
                    "--json",
                    "-",
                ],
                cwd=root,
                input="",
                capture_output=True,
                text=True,
                timeout=timeout,
                check=False,
            )
        except subprocess.TimeoutExpired:
            findings.append(Finding("error", "codex-strict-probe-timeout", codex, f"{tier} exceeded {timeout}s"))
            continue
        strict_output = "\n".join(part for part in (strict_probe.stdout, strict_probe.stderr) if part)
        if "No prompt provided via stdin." not in strict_output:
            findings.append(
                Finding(
                    "error",
                    "codex-strict-config-unusable",
                    codex,
                    f"{tier}: " + (strict_output.strip() or f"unexpected exit {strict_probe.returncode}"),
                )
            )

    strict_config_probe(
        codex,
        root / PROJECT_CONFIG,
        root,
        timeout,
        findings,
        copy_agent_layers=True,
    )
    for tier in sorted(EXPECTED_TIERS):
        strict_config_probe(
            codex,
            root / AGENT_CONFIG_DIR / f"{tier}.toml",
            root,
            timeout,
            findings,
            copy_agent_layers=False,
        )

    try:
        catalog_result = subprocess.run(
            [codex, "debug", "models", "--bundled"],
            cwd=root,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
    except subprocess.TimeoutExpired:
        findings.append(Finding("error", "codex-catalog-timeout", codex, f"exceeded {timeout}s"))
        return
    if catalog_result.returncode != 0:
        findings.append(Finding("error", "codex-catalog-failed", codex, catalog_result.stderr.strip()))
        return
    try:
        catalog = json.loads(catalog_result.stdout)
    except json.JSONDecodeError as exc:
        findings.append(Finding("error", "codex-catalog-invalid-json", codex, str(exc)))
        return
    models = catalog.get("models") if isinstance(catalog, dict) else None
    model = next(
        (item for item in models or [] if isinstance(item, dict) and item.get("slug") == EXPECTED_MODEL),
        None,
    )
    if model is None:
        findings.append(Finding("error", "codex-model-missing", codex, EXPECTED_MODEL))
        return
    efforts = {
        item.get("effort"): item.get("description")
        for item in model.get("supported_reasoning_levels", [])
        if isinstance(item, dict)
    }
    if not EXPECTED_TIERS.issubset(efforts):
        findings.append(
            Finding("error", "codex-tier-missing", codex, repr(sorted(EXPECTED_TIERS - set(efforts))))
        )
    if "delegation" not in str(efforts.get("ultra", "")).lower():
        findings.append(
            Finding("error", "codex-ultra-delegation-missing", codex, repr(efforts.get("ultra")))
        )


def strict_config_probe(
    codex: str,
    config_path: Path,
    root: Path,
    timeout: float,
    findings: list[Finding],
    *,
    copy_agent_layers: bool,
) -> None:
    """Load one repository config layer as CODEX_HOME/config.toml under strict mode."""

    with tempfile.TemporaryDirectory(prefix="codefire-codex-home-") as temporary_home:
        home = Path(temporary_home)
        try:
            shutil.copy2(config_path, home / "config.toml")
            if copy_agent_layers:
                shutil.copytree(root / AGENT_CONFIG_DIR, home / "agents")
        except OSError as exc:
            findings.append(
                Finding("error", "config-layer-copy-failed", str(config_path), str(exc))
            )
            return
        try:
            probe = subprocess.run(
                [
                    codex,
                    "exec",
                    "--strict-config",
                    "--sandbox",
                    "read-only",
                    "--cd",
                    str(root),
                    "--ephemeral",
                    "--json",
                    "-",
                ],
                cwd=root,
                env={**os.environ, "CODEX_HOME": str(home)},
                input="",
                capture_output=True,
                text=True,
                timeout=timeout,
                check=False,
            )
        except subprocess.TimeoutExpired:
            findings.append(
                Finding("error", "config-layer-probe-timeout", str(config_path), f"exceeded {timeout}s")
            )
        else:
            output = "\n".join(part for part in (probe.stdout, probe.stderr) if part)
            if "No prompt provided via stdin." not in output:
                findings.append(
                    Finding("error", "config-layer-strict-unusable", str(config_path), output.strip())
                )


def run(root: Path, check_cli: bool, timeout: float) -> list[Finding]:
    findings: list[Finding] = []
    check_project_config(root, findings)
    check_matrix(root, findings)
    check_agent_configs(root, findings)
    check_role_config_mapping(root, findings)
    if check_cli:
        check_codex_installation(root, findings, timeout)
    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=repository_root())
    parser.add_argument("--skip-cli", action="store_true", help="Skip installed Codex strict-parser checks.")
    parser.add_argument("--timeout", type=float, default=20.0)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    root = args.root.resolve()
    findings = run(root, check_cli=not args.skip_cli, timeout=args.timeout)
    errors = [finding for finding in findings if finding.level == "error"]
    payload = {
        "ok": not errors,
        "model": EXPECTED_MODEL,
        "tiers": sorted(EXPECTED_TIERS),
        "cli_checked": not args.skip_cli,
        "findings": [asdict(finding) for finding in findings],
    }
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    elif errors:
        print(f"AI runtime validation failed: {len(errors)} error(s)")
        for finding in errors:
            print(f"- {finding.code}: {finding.path}: {finding.message}")
    else:
        print("AI runtime validation passed: gpt-5.6-sol with medium/high/ultra routing.")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
