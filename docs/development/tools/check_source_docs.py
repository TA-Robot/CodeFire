#!/usr/bin/env python3
"""Check canonical source docs against Rust modules, commands, object kinds, and versions."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


OLD_SOURCE_DOCS = {
    "source-code-map.md",
    "source-code-blueprint.md",
    "source-code-mental-model.md",
    "source-code-anatomy.md",
    "source-code-implementation-atlas.md",
    "source-code-executable-mental-model.md",
    "source-code-reconstruction-guide.md",
}
REFERENCE_EXEMPT = {
    "docs/development/archive/deletion-manifest.md",
    "docs/development/document-consolidation-plan.md",
    "docs/development/history.md",
}
CANONICAL_DOCS = {
    "docs/development/source-architecture.md",
    "docs/development/command-source-trace.md",
    "docs/development/source-data-model-catalog.md",
    "docs/development/change-playbook.md",
}


@dataclass(frozen=True)
class Finding:
    level: str
    code: str
    path: str
    message: str


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def git_markdown_paths(root: Path) -> list[str]:
    paths: set[str] = set()
    for command in (
        ["git", "ls-files", "-z", "*.md"],
        ["git", "ls-files", "--others", "--exclude-standard", "-z", "*.md"],
    ):
        result = subprocess.run(command, cwd=root, capture_output=True, check=False)
        if result.returncode != 0:
            raise ValueError(result.stderr.decode("utf-8", errors="replace").strip())
        for value in result.stdout.split(b"\0"):
            if value:
                path = value.decode("utf-8", errors="surrogateescape")
                if (root / path).is_file():
                    paths.add(path)
    return sorted(paths)


def compare_sets(
    source: set[str],
    documented: set[str],
    location: str,
    kind: str,
    findings: list[Finding],
) -> None:
    for item in sorted(source - documented):
        findings.append(Finding("error", f"missing-{kind}-marker", location, item))
    for item in sorted(documented - source):
        findings.append(Finding("error", f"stale-{kind}-marker", location, item))


def run(root: Path) -> tuple[list[Finding], dict[str, object]]:
    findings: list[Finding] = []
    for relative in sorted(CANONICAL_DOCS):
        path = root / relative
        if not path.is_file():
            findings.append(Finding("error", "missing-canonical-source-doc", relative, "file is missing"))
            continue
        text = path.read_text(encoding="utf-8")
        for label in ("Status:", "Owner:", "Update trigger:"):
            if label not in text:
                findings.append(Finding("error", "missing-source-doc-metadata", relative, label))
        fixed_lines = sorted(set(re.findall(r"\b[\w./-]+\.rs:\d+\b", text)))
        if fixed_lines:
            findings.append(
                Finding("error", "fixed-source-line-reference", relative, ", ".join(fixed_lines[:10]))
            )

    main_text = (root / "crates/codefire-cli/src/main.rs").read_text(encoding="utf-8")
    architecture = (root / "docs/development/source-architecture.md").read_text(encoding="utf-8")
    source_modules = set(re.findall(r"^mod\s+([a-z_]+);\s*$", main_text, re.MULTILINE))
    documented_modules = set(re.findall(r"<!--\s*module:\s*([a-z_]+)\s*-->", architecture))
    compare_sets(source_modules, documented_modules, "docs/development/source-architecture.md", "module", findings)

    registry_text = (root / "crates/codefire-cli/src/command_registry.rs").read_text(encoding="utf-8")
    trace = (root / "docs/development/command-source-trace.md").read_text(encoding="utf-8")
    source_commands = set(re.findall(r'capability\("([a-z-]+)"\s*,', registry_text))
    documented_commands = set(re.findall(r"<!--\s*command:\s*([a-z-]+)\s*-->", trace))
    compare_sets(source_commands, documented_commands, "docs/development/command-source-trace.md", "command", findings)

    store_text = (root / "crates/codefire-store/src/lib.rs").read_text(encoding="utf-8")
    catalog = (root / "docs/development/source-data-model-catalog.md").read_text(encoding="utf-8")
    source_object_kinds = set(re.findall(r'ObjectKind::new\("([a-z_]+)"\s*,', store_text))
    documented_object_kinds = set(re.findall(r"<!--\s*object-kind:\s*([a-z_]+)\s*-->", catalog))
    compare_sets(
        source_object_kinds,
        documented_object_kinds,
        "docs/development/source-data-model-catalog.md",
        "object-kind",
        findings,
    )

    core_text = (root / "crates/codefire-core/src/lib.rs").read_text(encoding="utf-8")
    expected_constants: dict[str, str] = {}
    for name, source_text in (
        ("VERSION", core_text),
        ("ATOM_HASH_SCHEMA_VERSION", core_text),
        ("OBJECT_ID_DIGEST_HEX_LENGTH", store_text),
    ):
        match = re.search(rf"pub const {name}:\s*\w+\s*=\s*(\d+);", source_text)
        if match is None:
            findings.append(Finding("error", "missing-source-constant", "Rust source", name))
        else:
            expected_constants[name] = match.group(1)
    documented_constants = dict(
        re.findall(r"<!--\s*constant:\s*([A-Z0-9_]+)=(\d+)\s*-->", catalog)
    )
    if documented_constants != expected_constants:
        findings.append(
            Finding(
                "error",
                "source-constant-drift",
                "docs/development/source-data-model-catalog.md",
                f"source={expected_constants}, docs={documented_constants}",
            )
        )

    markdown_paths = git_markdown_paths(root)
    for old_name in sorted(OLD_SOURCE_DOCS):
        if (root / "docs/development" / old_name).exists():
            findings.append(
                Finding("error", "superseded-source-doc-present", f"docs/development/{old_name}", "remove after consolidation")
            )
    for relative in markdown_paths:
        if relative in REFERENCE_EXEMPT or relative.startswith("docs/development/exploration/"):
            continue
        text = (root / relative).read_text(encoding="utf-8")
        for old_name in sorted(OLD_SOURCE_DOCS):
            if old_name in text:
                findings.append(Finding("error", "active-reference-to-old-source-doc", relative, old_name))

    details: dict[str, object] = {
        "modules": sorted(source_modules),
        "commands": sorted(source_commands),
        "object_kinds": sorted(source_object_kinds),
        "constants": expected_constants,
    }
    return findings, details


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=repository_root())
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    try:
        findings, details = run(args.root.resolve())
    except (OSError, UnicodeError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    errors = [finding for finding in findings if finding.level == "error"]
    payload = {
        "schema": "codefire.source_docs_check.v1",
        "ok": not errors,
        "findings": [asdict(finding) for finding in findings],
        **details,
    }
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    elif errors:
        print(f"Source docs check failed: {len(errors)} error(s)")
        for finding in findings:
            print(f"- {finding.code}: {finding.path}: {finding.message}")
    else:
        print(
            "Source docs check passed: "
            f"{len(details['modules'])} modules, {len(details['commands'])} commands, "
            f"{len(details['object_kinds'])} object kinds."
        )
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
