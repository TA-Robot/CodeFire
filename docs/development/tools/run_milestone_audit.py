#!/usr/bin/env python3
"""Build a bounded brief for an event-triggered milestone audit."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def git_status(root: Path) -> str:
    result = subprocess.run(
        ["git", "status", "--short", "--branch"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        stderr = (result.stderr or "").strip() or "git status failed"
        return f"(unavailable: {stderr})"
    return (result.stdout or "").strip() or "(clean)"


def read_text(path: Path, limit: int = 4000) -> str:
    if not path.is_file():
        return f"(missing {path})"
    text = path.read_text(encoding="utf-8")
    if len(text) > limit:
        return text[:limit] + "\n...[truncated for audit brief]...\n"
    return text


def build_brief(root: Path) -> str:
    board = root / "docs/development/milestones.md"
    model = root / "docs/development/milestone-operating-model.md"
    previous = root / "docs/development/ai/audit-latest.md"
    parts = [
        "# Milestone audit brief",
        "",
        "Read this brief, then `docs/development/milestones.md` and",
        "`docs/development/milestone-operating-model.md`. Do not implement.",
        "Do not open parked CFB/CFR issues. Write `docs/development/ai/audit-latest.md`.",
        "",
        "## Git status",
        "",
        "```text",
        git_status(root),
        "```",
        "",
        "## Previous audit",
        "",
        read_text(previous, limit=2000) if previous.is_file() else "(none)",
        "",
        "## Board excerpt",
        "",
        read_text(board, limit=3000),
        "",
        "## Operating model excerpt",
        "",
        read_text(model, limit=2000),
    ]
    return "\n".join(parts).rstrip() + "\n"


def build_prompt(root: Path) -> str:
    brief = build_brief(root)
    return (
        "You are the CodeFire milestone audit agent in a fresh session, independent "
        "of the implementer. Grok 4.6 is the default; scarce Fable 5.1 requires "
        "explicit selection and verified subscription entitlement. Current-session "
        "Codex independent review is allowed only for explicitly admitted recovery "
        "bootstrap packets. Run a bounded read-only audit of plan vs progress "
        "when an integration, failure, stall, or capacity event warrants it; do not "
        "consume model quota for periodic status checks.\n\n"
        "Rules:\n"
        "- Use subscription capacity only: no metered billing, extra usage, "
        "automatic top-ups, or unverified external model execution.\n"
        "- Only advise how to reach the current M1 exit.\n"
        "- Park non-blocking review and extra tests.\n"
        "- Treat implementation of parked issues as stall.\n"
        "- Write docs/development/ai/audit-latest.md in the required shape "
        "from docs/development/ai/audit-cadence.md.\n"
        "- Do not edit product code, issue ledgers, or later-milestone docs.\n\n"
        f"{brief}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=None, help="CodeFire repository root")
    parser.add_argument(
        "--prompt",
        action="store_true",
        help="Print the full auditor prompt instead of the brief.",
    )
    args = parser.parse_args()
    root = (args.root or repository_root()).resolve()
    sys.stdout.write(build_prompt(root) if args.prompt else build_brief(root))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
