#!/usr/bin/env python3
"""Distinguish a healthy long review from a stalled report terminalization."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


READY_RE = re.compile(
    r"(?:report|decision|content).{0,48}(?:ready|fixed|complete)"
    r"|(?:writing|write).{0,48}report.{0,24}(?:now|next)",
    re.IGNORECASE | re.DOTALL,
)
VERDICT_RE = re.compile(
    r"(?<![A-Z_])(?:REQUEST_CHANGES|ACCEPT|INSUFFICIENT_EVIDENCE)(?![A-Z_])"
)


def utc_now() -> str:
    return (
        datetime.now(timezone.utc)
        .isoformat(timespec="microseconds")
        .replace("+00:00", "Z")
    )


def agent_texts(event_path: Path) -> list[str]:
    texts: list[str] = []
    for line in event_path.read_text(encoding="utf-8").splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue
        if event.get("type") == "item.completed":
            item = event.get("item")
            if (
                isinstance(item, dict)
                and item.get("type") in {"agent_message", "reasoning"}
                and isinstance(item.get("text"), str)
            ):
                texts.append(item["text"])
            continue
        if event.get("type") != "assistant":
            continue
        message = event.get("message")
        content = message.get("content") if isinstance(message, dict) else None
        if not isinstance(content, list):
            continue
        for block in content:
            if (
                isinstance(block, dict)
                and block.get("type") == "text"
                and isinstance(block.get("text"), str)
            ):
                texts.append(block["text"])
    return texts


def git_report_state(report: Path, git_root: Path | None) -> dict[str, Any]:
    resolved = report.resolve()
    if not resolved.is_file():
        return {
            "state": "missing",
            "path": str(resolved),
            "bytes": 0,
            "sha256": None,
        }
    content = resolved.read_bytes()
    state = {
        "state": "present-unverified",
        "path": str(resolved),
        "bytes": len(content),
        "sha256": hashlib.sha256(content).hexdigest(),
    }
    if git_root is None:
        return state
    root = git_root.resolve()
    try:
        relative = resolved.relative_to(root)
    except ValueError:
        state["state"] = "outside-git-root"
        return state
    head_check = subprocess.run(
        ["git", "-C", str(root), "cat-file", "-e", f"HEAD:{relative.as_posix()}"],
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    clean_check = subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "diff",
            "--quiet",
            "HEAD",
            "--",
            relative.as_posix(),
        ],
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    state["state"] = (
        "committed"
        if head_check.returncode == 0 and clean_check.returncode == 0
        else "present-uncommitted"
    )
    return state


def process_alive(pid: int | None) -> bool | None:
    if pid is None:
        return None
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    return True


def evaluate_terminalization(
    texts: list[str],
    *,
    report: dict[str, Any],
    worker_alive: bool | None,
    max_ready_signals: int,
    max_events_after_ready: int,
    max_events_after_verdict: int,
) -> dict[str, Any]:
    ready_indices = [index for index, text in enumerate(texts) if READY_RE.search(text)]
    verdict_indices = [
        index for index, text in enumerate(texts) if VERDICT_RE.search(text)
    ]
    after_ready = (
        0 if not ready_indices else len(texts) - ready_indices[0] - 1
    )
    after_verdict = (
        0 if not verdict_indices else len(texts) - verdict_indices[0] - 1
    )

    report_state = report["state"]
    if report_state == "committed" and worker_alive is not True:
        state = "complete"
        decision = "COMPLETE"
        action = "collect and verify the immutable report identity"
    elif report_state == "committed":
        state = "report-committed-worker-live"
        decision = "INTERVENE"
        action = (
            "the immutable report is already durable; stop further evidence "
            "expansion after active child commands and log filters flush, "
            "then collect the exact report commit"
        )
    elif report_state in {
        "present-uncommitted",
        "present-unverified",
        "outside-git-root",
    }:
        state = "report-present-not-committed"
        decision = "INTERVENE"
        action = (
            "stop further evidence expansion; validate and commit only the "
            "declared report write set"
        )
    elif (
        len(ready_indices) >= max_ready_signals
        and after_ready >= max_events_after_ready
    ) or (
        verdict_indices
        and after_verdict >= max_events_after_verdict
        and ready_indices
    ):
        state = "terminalization-stalled"
        decision = "INTERVENE"
        action = (
            "preserve the exact session/transcript, terminate only after log "
            "flush, then resume that session with a compact report-only "
            "closure prompt"
        )
    elif worker_alive is False:
        state = "terminal-without-report"
        decision = "INTERVENE"
        action = (
            "preserve terminal logs and resume the exact session with a "
            "report-only recovery prompt"
        )
    else:
        state = "active"
        decision = "WAIT"
        action = "continue while unique trusted signals are still arriving"

    return {
        "schema": "codefire.review_terminalization.v1",
        "captured_at": utc_now(),
        "decision": decision,
        "state": state,
        "worker_alive": worker_alive,
        "agent_text_events": len(texts),
        "ready_signals": len(ready_indices),
        "verdict_signals": len(verdict_indices),
        "events_after_first_ready": after_ready,
        "events_after_first_verdict": after_verdict,
        "thresholds": {
            "max_ready_signals": max_ready_signals,
            "max_events_after_ready": max_events_after_ready,
            "max_events_after_verdict": max_events_after_verdict,
        },
        "report": report,
        "recommended_action": action,
    }


def atomic_write_json(path: Path, payload: dict[str, Any]) -> None:
    path = path.resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.tmp-{os.getpid()}")
    temporary.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    os.replace(temporary, path)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Detect a review that has settled its verdict but is not "
            "producing the required immutable report."
        )
    )
    parser.add_argument("--events", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--git-root", type=Path)
    parser.add_argument("--pid", type=int)
    parser.add_argument("--max-ready-signals", type=int, default=3)
    parser.add_argument("--max-events-after-ready", type=int, default=20)
    parser.add_argument("--max-events-after-verdict", type=int, default=80)
    parser.add_argument("--result", type=Path)
    args = parser.parse_args(argv)
    for name in (
        "max_ready_signals",
        "max_events_after_ready",
        "max_events_after_verdict",
    ):
        if getattr(args, name) < 1:
            parser.error(f"--{name.replace('_', '-')} must be positive")
    if args.pid is not None and args.pid < 1:
        parser.error("--pid must be positive")
    return args


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        texts = agent_texts(args.events)
        report = git_report_state(args.report, args.git_root)
        result = evaluate_terminalization(
            texts,
            report=report,
            worker_alive=process_alive(args.pid),
            max_ready_signals=args.max_ready_signals,
            max_events_after_ready=args.max_events_after_ready,
            max_events_after_verdict=args.max_events_after_verdict,
        )
        if args.result is not None:
            atomic_write_json(args.result, result)
    except (OSError, ValueError, KeyError) as exc:
        print(f"Review terminalization audit error: {exc}", file=sys.stderr)
        return 2
    print(json.dumps(result, indent=2, sort_keys=True))
    return 3 if result["decision"] == "INTERVENE" else 0


if __name__ == "__main__":
    raise SystemExit(main())
