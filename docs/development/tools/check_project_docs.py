#!/usr/bin/env python3
"""Check CodeFire document links, ledgers, lifecycle, and active-state ownership."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections import Counter, defaultdict
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any, Iterable
from urllib.parse import unquote


REMOVED_PLAN_BASENAMES = {
    "completion-plan.md",
    "v0.6-readiness.md",
    "v0.6-rust-rewrite-plan.md",
    "v0.7-issue-improvement-plan.md",
    "v0.8-cfb-burn-down-plan.md",
    "v0.9-contract-scale-plan.md",
    "v1.0-contract-validation-plan.md",
}
REMOVED_REFERENCE_EXEMPT = {
    "docs/development/archive/deletion-manifest.md",
    "docs/development/document-consolidation-plan.md",
    "docs/development/history.md",
    "docs/development/todo-checklist.md",
}
LINK_PATTERN = re.compile(r"(?<!!)\[[^\]]*\]\(([^)]+)\)")
STATUS_PATTERN = re.compile(r"^\|\s*Status\s*\|\s*([^|]+?)\s*\|\s*$", re.MULTILINE)
INDEX_ROW_PATTERN = re.compile(r"^\|\s*((?:CFB|CFR)-\d{3})\s*\|\s*([^|]+?)\s*\|", re.MULTILINE)
CLOSED_ISSUE_STATUSES = {"fixed", "fixed in v0.7", "wontfix"}
TERMINAL_REVIEW_DISPOSITIONS = {"implemented", "falsified", "accepted_non_goal", "promoted"}


@dataclass(frozen=True)
class Finding:
    level: str
    code: str
    path: str
    message: str


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def git_paths(root: Path) -> list[str]:
    commands = (
        ["git", "ls-files", "-z"],
        ["git", "ls-files", "--others", "--exclude-standard", "-z"],
    )
    paths: set[str] = set()
    for command in commands:
        result = subprocess.run(command, cwd=root, capture_output=True, check=False)
        if result.returncode != 0:
            raise ValueError(result.stderr.decode("utf-8", errors="replace").strip() or "git file listing failed")
        for raw in result.stdout.split(b"\0"):
            if not raw:
                continue
            path = raw.decode("utf-8", errors="surrogateescape")
            if path.endswith(".md") and (root / path).is_file():
                paths.add(path)
    return sorted(paths)


def lifecycle(path: str) -> str:
    if path.startswith("docs/development/exploration/"):
        return "exploration"
    if path.startswith("docs/development/archive/") or path == "docs/development/history.md":
        return "historical"
    if "/bug-issues/" in path or "/code-review-issues/" in path:
        return "supporting"
    if path.startswith("docs/development/strategy/"):
        return "canonical"
    if path.startswith("docs/development/ai/"):
        return "supporting"
    if path.startswith("docs/development/"):
        return "supporting"
    if path.startswith("docs/") or path in {"README.md", "AGENTS.md"}:
        return "canonical"
    return "supporting"


def title_and_metadata(text: str) -> tuple[str | None, str | None, str | None, str | None]:
    title_match = re.search(r"^#\s+(.+?)\s*$", text, re.MULTILINE)
    status_match = re.search(r"^Status:\s*(.+?)\s*$", text, re.MULTILINE | re.IGNORECASE)
    owner_match = re.search(r"^Owner:\s*(.+?)\s*$", text, re.MULTILINE | re.IGNORECASE)
    trigger_match = re.search(r"^Update trigger:\s*(.+?)\s*$", text, re.MULTILINE | re.IGNORECASE)
    return (
        title_match.group(1) if title_match else None,
        status_match.group(1) if status_match else None,
        owner_match.group(1) if owner_match else None,
        trigger_match.group(1) if trigger_match else None,
    )


def parse_link_target(raw: str) -> str | None:
    value = raw.strip()
    if not value:
        return None
    if value.startswith("<") and ">" in value:
        value = value[1:value.index(">")]
    else:
        value = re.split(r"\s+[\"']", value, maxsplit=1)[0]
    lowered = value.lower()
    if lowered.startswith(("http://", "https://", "mailto:", "app://")) or value.startswith("#"):
        return None
    value = unquote(value.split("#", 1)[0].split("?", 1)[0])
    return value or None


def resolve_link(root: Path, source: Path, target: str) -> Path:
    if target.startswith("/workspace/project/"):
        return Path(target).resolve()
    if target == "/workspace/project":
        return root
    if target.startswith("/"):
        return (root / target.lstrip("/")).resolve()
    return (source.parent / target).resolve()


def markdown_links(text: str) -> Iterable[tuple[int, str]]:
    in_fence = False
    fence_marker = ""
    for line_number, line in enumerate(text.splitlines(), start=1):
        stripped = line.lstrip()
        if stripped.startswith(("```", "~~~")):
            marker = stripped[:3]
            if not in_fence:
                in_fence = True
                fence_marker = marker
            elif marker == fence_marker:
                in_fence = False
                fence_marker = ""
            continue
        if in_fence:
            continue
        for match in LINK_PATTERN.finditer(line):
            target = parse_link_target(match.group(1))
            if target is not None:
                yield line_number, target


def check_links(
    root: Path,
    markdown_paths: list[str],
    findings: list[Finding],
) -> dict[str, int]:
    inbound: Counter[str] = Counter()
    for relative in markdown_paths:
        source = root / relative
        text = source.read_text(encoding="utf-8")
        for line_number, target in markdown_links(text):
            if re.match(r"^[A-Za-z][A-Za-z0-9+.-]*:", target):
                findings.append(Finding("warning", "unsupported-link-scheme", f"{relative}:{line_number}", target))
                continue
            resolved = resolve_link(root, source, target)
            try:
                resolved_relative = resolved.relative_to(root).as_posix()
            except ValueError:
                findings.append(Finding("error", "link-outside-repository", f"{relative}:{line_number}", target))
                continue
            if not resolved.exists():
                level = "warning" if lifecycle(relative) in {"historical", "exploration"} else "error"
                findings.append(Finding(level, "broken-local-link", f"{relative}:{line_number}", target))
                continue
            inbound[resolved_relative] += 1
    return dict(inbound)


def detail_statuses(directory: Path, prefix: str, findings: list[Finding]) -> dict[str, str]:
    statuses: dict[str, str] = {}
    for path in sorted(directory.glob(f"{prefix.lower()}-[0-9][0-9][0-9].md")):
        issue_id = path.stem.upper()
        text = path.read_text(encoding="utf-8")
        matches = STATUS_PATTERN.findall(text)
        relative = path.relative_to(directory.parents[1]).as_posix()
        if len(matches) != 1:
            findings.append(Finding("error", "issue-status-cardinality", relative, f"found {len(matches)} Status rows"))
            continue
        statuses[issue_id] = matches[0].strip()
    return statuses


def index_statuses(path: Path, prefix: str, findings: list[Finding], root: Path) -> dict[str, str]:
    rows: dict[str, str] = {}
    duplicates: set[str] = set()
    for issue_id, status in INDEX_ROW_PATTERN.findall(path.read_text(encoding="utf-8")):
        if not issue_id.startswith(prefix):
            continue
        if issue_id in rows:
            duplicates.add(issue_id)
        rows[issue_id] = status.strip()
    for issue_id in sorted(duplicates):
        findings.append(Finding("error", "duplicate-index-row", path.relative_to(root).as_posix(), issue_id))
    return rows


def check_ledger_pair(
    root: Path,
    detail_dir: str,
    index_path: str,
    prefix: str,
    allowed_statuses: set[str],
    findings: list[Finding],
    *,
    index_has_status: bool,
) -> dict[str, Any]:
    details = detail_statuses(root / detail_dir, prefix, findings)
    index_text = (root / index_path).read_text(encoding="utf-8")
    if index_has_status:
        index = index_statuses(root / index_path, prefix, findings, root)
        for issue_id in sorted(set(details) - set(index)):
            findings.append(Finding("error", "detail-missing-from-index", index_path, issue_id))
        for issue_id in sorted(set(index) - set(details)):
            findings.append(Finding("error", "index-missing-detail", index_path, issue_id))
        for issue_id in sorted(set(details) & set(index)):
            if details[issue_id] != index[issue_id]:
                findings.append(
                    Finding(
                        "error",
                        "issue-status-mismatch",
                        issue_id,
                        f"detail={details[issue_id]!r}, index={index[issue_id]!r}",
                    )
                )
    else:
        referenced = set(re.findall(rf"\b{re.escape(prefix)}-\d{{3}}\b", index_text))
        for issue_id in sorted(set(details) - referenced):
            findings.append(Finding("error", "detail-missing-from-index", index_path, issue_id))
        declared_total = re.search(r"^- Total:\s*(\d+)\s*$", index_text, re.MULTILINE)
        declared_closed = re.search(r"^- Fixed / closed:\s*(\d+)\s*$", index_text, re.MULTILINE)
        declared_open = re.search(r"^- Remaining open:\s*(\d+)\s*$", index_text, re.MULTILINE)
        open_count = sum(1 for status in details.values() if status not in CLOSED_ISSUE_STATUSES)
        expected_summary = {
            "Total": (declared_total, len(details)),
            "Fixed / closed": (declared_closed, len(details) - open_count),
            "Remaining open": (declared_open, open_count),
        }
        for label, (match, expected_value) in expected_summary.items():
            if match is None or int(match.group(1)) != expected_value:
                actual = None if match is None else int(match.group(1))
                findings.append(
                    Finding(
                        "error",
                        "stale-index-count",
                        index_path,
                        f"{label}: expected {expected_value}, got {actual}",
                    )
                )
    for issue_id, status in sorted(details.items()):
        if status not in allowed_statuses:
            findings.append(Finding("error", "unknown-issue-status", issue_id, status))
    counts = Counter(details.values())
    return {
        "total": len(details),
        "statuses": dict(sorted(counts.items())),
        "_issue_statuses": details,
    }


def check_issue_root_cause_map(
    root: Path,
    issue_statuses: dict[str, str],
    findings: list[Finding],
) -> None:
    relative = "docs/development/issue-root-cause-map.json"
    path = root / relative
    if not path.is_file():
        findings.append(Finding("error", "missing-root-cause-map", relative, "file is missing"))
        return
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError, UnicodeError) as exc:
        findings.append(Finding("error", "invalid-root-cause-map", relative, str(exc)))
        return
    if not isinstance(payload, dict) or payload.get("type") != "codefire.issue_root_cause_map.v1":
        findings.append(
            Finding(
                "error",
                "invalid-root-cause-map-type",
                relative,
                "expected type codefire.issue_root_cause_map.v1",
            )
        )
        return
    raw_entries = payload.get("issues")
    if not isinstance(raw_entries, list):
        findings.append(Finding("error", "invalid-root-cause-map-issues", relative, "issues must be an array"))
        return

    entries: dict[str, dict[str, Any]] = {}
    duplicates: set[str] = set()
    for offset, raw_entry in enumerate(raw_entries):
        entry_path = f"{relative}:issues[{offset}]"
        if not isinstance(raw_entry, dict):
            findings.append(Finding("error", "invalid-root-cause-entry", entry_path, "entry must be an object"))
            continue
        issue_id = raw_entry.get("id")
        if not isinstance(issue_id, str) or not re.fullmatch(r"(?:CFB|CFR)-\d{3}", issue_id):
            findings.append(Finding("error", "invalid-root-cause-id", entry_path, repr(issue_id)))
            continue
        if issue_id in entries:
            duplicates.add(issue_id)
            continue
        entries[issue_id] = raw_entry
    for issue_id in sorted(duplicates):
        findings.append(Finding("error", "duplicate-root-cause-entry", relative, issue_id))

    open_ids = {
        issue_id for issue_id, status in issue_statuses.items() if status not in CLOSED_ISSUE_STATUSES
    }
    for issue_id in sorted(open_ids - set(entries)):
        findings.append(
            Finding(
                "error",
                "open-issue-missing-root-cause-entry",
                relative,
                issue_id,
            )
        )

    required_text = ("root_cause", "fix_group", "target_version", "owner_module")
    for issue_id, entry in sorted(entries.items()):
        ledger_status = issue_statuses.get(issue_id)
        if ledger_status is None:
            findings.append(Finding("error", "root-cause-entry-without-detail", relative, issue_id))
        elif entry.get("status") != ledger_status:
            findings.append(
                Finding(
                    "error",
                    "root-cause-status-mismatch",
                    issue_id,
                    f"map={entry.get('status')!r}, detail={ledger_status!r}",
                )
            )
        if issue_id not in open_ids:
            continue
        for field in required_text:
            value = entry.get(field)
            if not isinstance(value, str) or not value.strip():
                findings.append(Finding("error", "incomplete-open-root-cause-entry", issue_id, field))
        mapping = entry.get("next_plan_mapping")
        if not isinstance(mapping, dict):
            findings.append(
                Finding("error", "incomplete-open-root-cause-entry", issue_id, "next_plan_mapping")
            )
        else:
            for field in ("plan", "phase", "decision"):
                value = mapping.get(field)
                if not isinstance(value, str) or not value.strip():
                    findings.append(
                        Finding("error", "incomplete-open-root-cause-entry", issue_id, f"next_plan_mapping.{field}")
                    )
            plan = mapping.get("plan")
            if isinstance(plan, str) and plan.strip():
                resolved = (root / plan).resolve()
                try:
                    resolved.relative_to(root)
                except ValueError:
                    findings.append(Finding("error", "root-cause-plan-outside-repository", issue_id, plan))
                else:
                    if not resolved.is_file():
                        findings.append(Finding("error", "root-cause-plan-missing", issue_id, plan))
        evidence = entry.get("close_evidence")
        if not isinstance(evidence, list) or not evidence or any(
            not isinstance(item, str) or not item.strip() for item in evidence
        ):
            findings.append(Finding("error", "incomplete-open-root-cause-entry", issue_id, "close_evidence"))


def check_closed_issue_checklists(
    root: Path,
    issue_statuses: dict[str, str],
    findings: list[Finding],
) -> None:
    for issue_id, status in sorted(issue_statuses.items()):
        if status not in CLOSED_ISSUE_STATUSES:
            continue
        series = "bug-issues" if issue_id.startswith("CFB-") else "code-review-issues"
        relative = f"docs/development/{series}/{issue_id.lower()}.md"
        text = (root / relative).read_text(encoding="utf-8")
        if re.search(r"^- \[ \] ", text, re.MULTILINE):
            findings.append(
                Finding(
                    "error",
                    "closed-issue-has-unchecked-checklist",
                    relative,
                    f"{issue_id} is {status} but retains unchecked closure metadata",
                )
            )


def check_review_dispositions(
    root: Path,
    issue_statuses: dict[str, str],
    findings: list[Finding],
) -> dict[str, int]:
    relative = "docs/development/codefire-review-portfolio/review-dispositions.json"
    path = root / relative
    if not path.is_file():
        findings.append(Finding("error", "missing-review-disposition-ledger", relative, "file is missing"))
        return {}
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError, UnicodeError) as exc:
        findings.append(Finding("error", "invalid-review-disposition-ledger", relative, str(exc)))
        return {}
    if not isinstance(payload, dict) or payload.get("type") != "codefire.review_disposition_ledger.v1":
        findings.append(
            Finding(
                "error",
                "invalid-review-disposition-ledger-type",
                relative,
                "expected type codefire.review_disposition_ledger.v1",
            )
        )
        return {}
    raw_reviews = payload.get("reviews")
    if not isinstance(raw_reviews, list):
        findings.append(Finding("error", "invalid-review-disposition-reviews", relative, "reviews must be an array"))
        return {}

    portfolio = root / "docs/development/codefire-review-portfolio"
    expected: dict[str, Path] = {}
    for review_path in sorted((portfolio / "reviews").glob("[0-9][0-9][0-9]-*.md")):
        match = re.match(r"(\d{3})-", review_path.name)
        if match:
            expected[f"RVC-{match.group(1)}"] = review_path

    entries: dict[str, dict[str, Any]] = {}
    counts: Counter[str] = Counter()
    for offset, raw_entry in enumerate(raw_reviews):
        entry_path = f"{relative}:reviews[{offset}]"
        if not isinstance(raw_entry, dict):
            findings.append(Finding("error", "invalid-review-disposition-entry", entry_path, "entry must be an object"))
            continue
        review_id = raw_entry.get("id")
        if not isinstance(review_id, str) or not re.fullmatch(r"RVC-\d{3}", review_id):
            findings.append(Finding("error", "invalid-review-disposition-id", entry_path, repr(review_id)))
            continue
        if review_id in entries:
            findings.append(Finding("error", "duplicate-review-disposition", relative, review_id))
            continue
        entries[review_id] = raw_entry

        disposition = raw_entry.get("disposition")
        if disposition not in TERMINAL_REVIEW_DISPOSITIONS:
            findings.append(Finding("error", "nonterminal-review-disposition", review_id, repr(disposition)))
            continue
        counts[disposition] += 1
        for field in ("root_group", "rationale"):
            value = raw_entry.get(field)
            if not isinstance(value, str) or not value.strip():
                findings.append(Finding("error", "incomplete-review-disposition", review_id, field))

        declared_review = raw_entry.get("review")
        expected_path = expected.get(review_id)
        if expected_path is None:
            findings.append(Finding("error", "review-disposition-without-review", review_id, repr(declared_review)))
        else:
            expected_relative = expected_path.relative_to(portfolio).as_posix()
            if declared_review != expected_relative:
                findings.append(
                    Finding(
                        "error",
                        "review-disposition-path-mismatch",
                        review_id,
                        f"expected {expected_relative!r}, got {declared_review!r}",
                    )
                )
            review_text = expected_path.read_text(encoding="utf-8")
            status_match = STATUS_PATTERN.search(review_text)
            review_status = status_match.group(1).strip() if status_match else None
            if review_status != disposition:
                findings.append(
                    Finding(
                        "error",
                        "review-status-disposition-mismatch",
                        expected_path.relative_to(root).as_posix(),
                        f"file={review_status!r}, ledger={disposition!r}",
                    )
                )
            if re.search(r"^- \[ \] ", review_text, re.MULTILINE):
                findings.append(
                    Finding(
                        "error",
                        "terminal-review-has-unchecked-checklist",
                        expected_path.relative_to(root).as_posix(),
                        review_id,
                    )
                )

        evidence_paths = raw_entry.get("evidence_paths")
        if not isinstance(evidence_paths, list) or not evidence_paths:
            findings.append(Finding("error", "incomplete-review-disposition", review_id, "evidence_paths"))
        else:
            for evidence in evidence_paths:
                if not isinstance(evidence, str) or not evidence.strip():
                    findings.append(Finding("error", "invalid-review-evidence-path", review_id, repr(evidence)))
                    continue
                resolved = (root / evidence).resolve()
                try:
                    resolved.relative_to(root)
                except ValueError:
                    findings.append(Finding("error", "review-evidence-outside-repository", review_id, evidence))
                else:
                    if not resolved.exists():
                        findings.append(Finding("error", "missing-review-evidence", review_id, evidence))

        artifacts = raw_entry.get("formal_artifacts")
        if not isinstance(artifacts, list) or not artifacts:
            findings.append(Finding("error", "incomplete-review-disposition", review_id, "formal_artifacts"))
        else:
            issue_artifacts = 0
            for artifact in artifacts:
                if not isinstance(artifact, str) or not artifact.strip():
                    findings.append(Finding("error", "invalid-review-formal-artifact", review_id, repr(artifact)))
                    continue
                if re.fullmatch(r"(?:CFB|CFR)-\d{3}", artifact):
                    issue_artifacts += 1
                    if artifact not in issue_statuses:
                        findings.append(Finding("error", "missing-review-formal-issue", review_id, artifact))
                else:
                    resolved = (root / artifact).resolve()
                    try:
                        resolved.relative_to(root)
                    except ValueError:
                        findings.append(Finding("error", "review-artifact-outside-repository", review_id, artifact))
                    else:
                        if not resolved.exists():
                            findings.append(Finding("error", "missing-review-formal-artifact", review_id, artifact))
            if disposition == "promoted" and issue_artifacts == 0:
                findings.append(Finding("error", "promoted-review-without-formal-issue", review_id, repr(artifacts)))

    for review_id in sorted(set(expected) - set(entries)):
        findings.append(Finding("error", "review-missing-disposition", relative, review_id))
    for review_id in sorted(set(entries) - set(expected)):
        findings.append(Finding("error", "disposition-missing-review", relative, review_id))
    if len(expected) != 60:
        findings.append(Finding("error", "unexpected-review-portfolio-size", relative, f"expected 60, found {len(expected)}"))
    return dict(sorted(counts.items()))


def check_current_claims(root: Path, ledgers: dict[str, Any], findings: list[Finding]) -> None:
    hub_path = "docs/development/README.md"
    hub = (root / hub_path).read_text(encoding="utf-8")
    cfb = ledgers["cfb"]
    cfr = ledgers["cfr"]
    def ledger_claim(prefix: str, ledger: dict[str, Any]) -> str:
        statuses = ", ".join(
            f"{count} `{status}`" for status, count in ledger["statuses"].items()
        )
        return f"{prefix} detail ledger: {ledger['total']} total; {statuses}."

    expected = (
        ledger_claim("CFB", cfb),
        ledger_claim("CFR", cfr),
    )
    for claim in expected:
        if claim not in hub:
            findings.append(Finding("error", "stale-hub-ledger-claim", hub_path, f"expected: {claim}"))
    open_cfb = sum(count for status, count in cfb["statuses"].items() if status not in CLOSED_ISSUE_STATUSES)
    open_cfr = sum(count for status, count in cfr["statuses"].items() if status not in CLOSED_ISSUE_STATUSES)
    active_path = "docs/development/current-cycle-burn-down-plan.md"
    active = (root / active_path).read_text(encoding="utf-8")
    active_claims = (
        f"- CFB/CFR product/review issues: {open_cfb + open_cfr} open/investigating.",
        f"- {ledger_claim('CFB', cfb)}",
        f"- {ledger_claim('CFR', cfr)}",
    )
    for claim in active_claims:
        if claim not in active:
            findings.append(
                Finding(
                    "error",
                    "stale-active-cycle-ledger-claim",
                    active_path,
                    f"expected: {claim}",
                )
            )
    cycle_path = "docs/development/codefire-improvement-cycle.md"
    cycle = (root / cycle_path).read_text(encoding="utf-8")
    table_rows = (
        f"| CFB dogfood issues | {cfb['total']} | {cfb['total'] - open_cfb} | {open_cfb} |",
        f"| CFR code review issues | {cfr['total']} | {cfr['total'] - open_cfr} | {open_cfr} |",
        f"| Total | {cfb['total'] + cfr['total']} | {cfb['total'] + cfr['total'] - open_cfb - open_cfr} | {open_cfb + open_cfr} |",
    )
    for row in table_rows:
        if row not in cycle:
            findings.append(Finding("error", "stale-cycle-ledger-claim", cycle_path, f"expected row: {row}"))


def check_removed_plans(root: Path, markdown_paths: list[str], findings: list[Finding]) -> None:
    for basename in sorted(REMOVED_PLAN_BASENAMES):
        if (root / "docs/development" / basename).exists():
            findings.append(Finding("error", "superseded-plan-present", f"docs/development/{basename}", "file should be removed"))
    for relative in markdown_paths:
        if relative in REMOVED_REFERENCE_EXEMPT:
            continue
        text = (root / relative).read_text(encoding="utf-8")
        for basename in sorted(REMOVED_PLAN_BASENAMES):
            if basename in text:
                findings.append(Finding("error", "active-reference-to-removed-plan", relative, basename))


def check_strategy_metadata(root: Path, findings: list[Finding]) -> None:
    strategy_dir = root / "docs/development/strategy"
    required = {
        "README.md",
        "engineering-principles.md",
        "product-strategy.md",
        "modernization-roadmap.md",
        "release-and-format-roadmap.md",
        "decision-register.md",
    }
    actual = {path.name for path in strategy_dir.glob("*.md")}
    for name in sorted(required - actual):
        findings.append(Finding("error", "missing-strategy-document", f"docs/development/strategy/{name}", "file is missing"))
    for path in sorted(strategy_dir.glob("*.md")):
        text = path.read_text(encoding="utf-8")
        relative = path.relative_to(root).as_posix()
        title, status, owner, trigger = title_and_metadata(text)
        for field, value in (("title", title), ("status", status), ("owner", owner), ("update-trigger", trigger)):
            if value is None:
                findings.append(Finding("error", f"missing-strategy-{field}", relative, f"{field} is required"))


def build_inventory(root: Path, markdown_paths: list[str], inbound: dict[str, int]) -> list[dict[str, Any]]:
    inventory: list[dict[str, Any]] = []
    for relative in markdown_paths:
        path = root / relative
        text = path.read_text(encoding="utf-8")
        title, status, owner, trigger = title_and_metadata(text)
        inventory.append(
            {
                "path": relative,
                "title": title,
                "lines": len(text.splitlines()),
                "bytes": len(text.encode("utf-8")),
                "lifecycle": lifecycle(relative),
                "declared_status": status,
                "owner": owner,
                "update_trigger": trigger,
                "inbound_links": inbound.get(relative, 0),
            }
        )
    return inventory


def run(root: Path) -> tuple[list[Finding], dict[str, Any]]:
    findings: list[Finding] = []
    markdown_paths = git_paths(root)
    inbound = check_links(root, markdown_paths, findings)
    ledgers = {
        "cfb": check_ledger_pair(
            root,
            "docs/development/bug-issues",
            "docs/development/bug-backlog.md",
            "CFB",
            {"open", "investigating", "fixed", "wontfix"},
            findings,
            index_has_status=True,
        ),
        "cfr": check_ledger_pair(
            root,
            "docs/development/code-review-issues",
            "docs/development/code-review-issues-2026-06-06.md",
            "CFR",
            {"open", "investigating", "fixed", "fixed in v0.7", "wontfix"},
            findings,
            index_has_status=False,
        ),
    }
    issue_statuses = {
        issue_id: status
        for ledger in ledgers.values()
        for issue_id, status in ledger["_issue_statuses"].items()
    }
    check_issue_root_cause_map(root, issue_statuses, findings)
    check_closed_issue_checklists(root, issue_statuses, findings)
    review_dispositions = check_review_dispositions(root, issue_statuses, findings)
    check_current_claims(root, ledgers, findings)
    check_removed_plans(root, markdown_paths, findings)
    check_strategy_metadata(root, findings)
    current_cycle = root / "docs/development/current-cycle-burn-down-plan.md"
    if current_cycle.is_file() and len(current_cycle.read_text(encoding="utf-8").splitlines()) > 500:
        findings.append(
            Finding(
                "warning",
                "oversized-active-cycle",
                current_cycle.relative_to(root).as_posix(),
                "active cycle exceeds 500 lines; preserve uncommitted evidence before pruning",
            )
        )
    public_ledgers = {
        name: {key: value for key, value in ledger.items() if not key.startswith("_")}
        for name, ledger in ledgers.items()
    }
    payload = {
        "ledgers": public_ledgers,
        "review_dispositions": review_dispositions,
        "inventory": build_inventory(root, markdown_paths, inbound),
    }
    return findings, payload


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=repository_root())
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--inventory", action="store_true", help="Include the full document inventory in text/JSON output.")
    args = parser.parse_args()
    root = args.root.resolve()
    try:
        findings, details = run(root)
    except (OSError, UnicodeError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    errors = [finding for finding in findings if finding.level == "error"]
    payload: dict[str, Any] = {
        "schema": "codefire.project_docs_check.v1",
        "ok": not errors,
        "findings": [asdict(finding) for finding in findings],
        "ledgers": details["ledgers"],
        "review_dispositions": details["review_dispositions"],
        "document_count": len(details["inventory"]),
    }
    if args.inventory:
        payload["inventory"] = details["inventory"]
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True, ensure_ascii=False))
    else:
        state = "passed" if not errors else f"failed: {len(errors)} error(s)"
        print(f"Project docs check {state}; {payload['document_count']} Markdown files inventoried.")
        for finding in findings:
            print(f"- {finding.level}: {finding.code}: {finding.path}: {finding.message}")
        if args.inventory:
            for item in details["inventory"]:
                print(f"{item['lifecycle']:11} {item['lines']:5} {item['inbound_links']:3} {item['path']}")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
