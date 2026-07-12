import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path


@dataclass(frozen=True)
class Entry:
    status: str
    path: str
    package: str


SAVE_ORDER = [
    "management_docs",
    "ai_acceleration_docs",
    "ai_acceleration_tools",
    "strategy_architecture_docs",
    "document_cleanup",
    "codefire_issue_metadata",
    "codefire_product",
    "algorithm_implementation",
    "algorithm_docs",
    "unknown",
]


ISSUE_METADATA_PREFIXES = [
    "docs/development/bug-issues/",
    "docs/development/code-review-issues/",
]


ISSUE_METADATA_FILES = {
    "docs/development/bug-backlog.md",
    "docs/development/code-review-issues-2026-06-06.md",
    "docs/development/issue-root-cause-map.json",
}


MANAGEMENT_FILES = {
    "README.md",
    "AGENTS.md",
    "docs/development/README.md",
    "docs/development/project-management.md",
    "docs/development/worktree-cleanup-plan.md",
    "docs/development/current-cycle-burn-down-plan.md",
    "docs/development/codefire-improvement-cycle.md",
    "docs/development/todo-checklist.md",
    "docs/development/history.md",
}


ALGORITHM_ROOT = "subprojects/algorithm-evolution-agent-lab/"

LEGACY_PRODUCT_ROOT_FILES = {
    "CodeFire_Design_Document_v0.2.docx",
    "CodeFire_Design_Document_v0.2.md",
    "CodeFire_Design_Document_v0.2.pdf",
    "IMPLEMENTATION_TODO.md",
}

DOCUMENT_CLEANUP_FILES = LEGACY_PRODUCT_ROOT_FILES | {
    "docs/12_mvp_implementation_plan.md",
    "docs/decisions.md",
    "docs/development/completion-plan.md",
    "docs/development/v0.6-readiness.md",
    "docs/development/v0.6-rust-rewrite-plan.md",
    "docs/development/v0.7-issue-improvement-plan.md",
    "docs/development/v0.8-cfb-burn-down-plan.md",
    "docs/development/v0.9-contract-scale-plan.md",
    "docs/development/v1.0-contract-validation-plan.md",
    "docs/development/source-code-anatomy.md",
    "docs/development/source-code-blueprint.md",
    "docs/development/source-code-executable-mental-model.md",
    "docs/development/source-code-implementation-atlas.md",
    "docs/development/source-code-map.md",
    "docs/development/source-code-mental-model.md",
    "docs/development/source-code-reconstruction-guide.md",
}

STRATEGY_ARCHITECTURE_FILES = {
    "adr/ADR-008-object-id-digest-length.md",
    "docs/development/change-playbook.md",
    "docs/development/command-source-trace.md",
    "docs/development/source-architecture.md",
    "docs/development/source-data-model-catalog.md",
}


def run_git_status(root: Path) -> list[str]:
    result = subprocess.run(
        ["git", "status", "--short", "--untracked-files=all"],
        cwd=root,
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "git status failed")
    return [line for line in result.stdout.splitlines() if line.strip()]


def parse_status_line(line: str) -> tuple[str, str]:
    status = line[:2]
    path = line[3:] if len(line) > 3 else ""
    if " -> " in path:
        path = path.split(" -> ", 1)[1]
    return status, path


def has_prefix(path: str, prefixes: list[str]) -> bool:
    return any(path.startswith(prefix) for prefix in prefixes)


def classify_path(path: str) -> str:
    if path in DOCUMENT_CLEANUP_FILES or path.startswith("docs/development/archive/") or path == "docs/development/document-consolidation-plan.md":
        return "document_cleanup"
    if path.startswith(".codex/"):
        return "ai_acceleration_docs"
    if path.startswith("docs/development/tools/"):
        return "ai_acceleration_tools"
    if path.startswith("docs/development/ai/"):
        return "ai_acceleration_docs"
    if path in ISSUE_METADATA_FILES or has_prefix(path, ISSUE_METADATA_PREFIXES):
        return "codefire_issue_metadata"
    if path in MANAGEMENT_FILES:
        return "management_docs"
    if (
        path in STRATEGY_ARCHITECTURE_FILES
        or path.startswith("docs/development/exploration/")
        or path.startswith("docs/development/strategy/")
    ):
        return "strategy_architecture_docs"
    if path.startswith(ALGORITHM_ROOT + "src/") or path.startswith(ALGORITHM_ROOT + "tests/"):
        return "algorithm_implementation"
    if (
        path.startswith(ALGORITHM_ROOT + "docs/")
        or path == ALGORITHM_ROOT + "AGENTS.md"
        or path == ALGORITHM_ROOT + "codefire.links.yaml"
    ):
        return "algorithm_docs"
    if (
        path.startswith("crates/")
        or path.startswith("tests/")
        or path == "codefire"
        or path == "install.sh"
        or path.startswith("adr/")
        or path.startswith("docs/")
    ):
        return "codefire_product"
    return "unknown"


def classify(root: Path) -> list[Entry]:
    entries: list[Entry] = []
    for line in run_git_status(root):
        status, path = parse_status_line(line)
        entries.append(Entry(status=status, path=path, package=classify_path(path)))
    return entries


def grouped(entries: list[Entry]) -> dict[str, list[Entry]]:
    packages = {package: [] for package in SAVE_ORDER}
    for entry in entries:
        packages.setdefault(entry.package, []).append(entry)
    return packages


def print_text(entries: list[Entry]) -> None:
    packages = grouped(entries)
    print("Dirty tree package summary:")
    for package in SAVE_ORDER:
        count = len(packages.get(package, []))
        if count:
            print(f"- {package}: {count}")
    print()
    for package in SAVE_ORDER:
        package_entries = packages.get(package, [])
        if not package_entries:
            continue
        print(f"[{package}]")
        for entry in package_entries:
            print(f"{entry.status} {entry.path}")
        print()
    unknown = packages.get("unknown", [])
    if unknown:
        print("Unknown files need human classification before packaging.")
    else:
        print("No unknown files.")


def main() -> int:
    parser = argparse.ArgumentParser(description="Classify CodeFire dirty-tree files into reviewable packages.")
    parser.add_argument("--root", default=".", help="Repository root. Defaults to current directory.")
    parser.add_argument("--json", action="store_true", help="Print machine-readable classification.")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    try:
        entries = classify(root)
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        return 2

    packages = grouped(entries)
    unknown = packages.get("unknown", [])
    if args.json:
        payload = {
            "ok": not unknown,
            "save_order": SAVE_ORDER,
            "packages": {
                package: [asdict(entry) for entry in packages.get(package, [])]
                for package in SAVE_ORDER
                if packages.get(package)
            },
            "unknown": [asdict(entry) for entry in unknown],
        }
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        print_text(entries)
    return 1 if unknown else 0


if __name__ == "__main__":
    sys.exit(main())
