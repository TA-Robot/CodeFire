from __future__ import annotations

import ast
import json
from dataclasses import dataclass
from pathlib import Path


SKIP_DIRS = frozenset({"__pycache__", ".git", ".mypy_cache", ".pytest_cache", ".venv", "venv"})


@dataclass(frozen=True)
class NameHit:
    path: str
    line: int
    kind: str
    name: str
    length: int


@dataclass(frozen=True)
class NameFix:
    hit: NameHit
    new_name: str
    reason: str


@dataclass(frozen=True)
class NameStat:
    hits: int
    files: int
    max_len: int
    max_name: str


@dataclass(frozen=True)
class NameGateRes:
    ok: bool
    stat: NameStat


# cf-atom: CODE-NameLenAudit
class NameLenAudit:
    def __init__(self, max_len: int = 25) -> None:
        if max_len < 1:
            raise ValueError("max_len must be positive")
        self.max_len = max_len

    def scan_text(self, text: str, *, path: str = "<text>") -> tuple[NameHit, ...]:
        tree = ast.parse(text)
        hits: list[NameHit] = []
        for node in ast.walk(tree):
            if isinstance(node, (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)):
                self._add(hits, path, node.lineno, node.__class__.__name__, node.name)
            elif isinstance(node, ast.arg):
                self._add(hits, path, node.lineno, "arg", node.arg)
            elif isinstance(node, ast.Name) and isinstance(node.ctx, ast.Store):
                self._add(hits, path, node.lineno, "name", node.id)
        return tuple(sorted(set(hits), key=lambda hit: (-hit.length, hit.path, hit.line, hit.name)))

    def scan_file(self, path: Path) -> tuple[NameHit, ...]:
        return self.scan_text(path.read_text(encoding="utf-8"), path=str(path))

    def scan_paths(self, paths: tuple[Path, ...]) -> tuple[NameHit, ...]:
        hits: list[NameHit] = []
        for path in paths:
            hits.extend(self.scan_file(path))
        return tuple(sorted(set(hits), key=lambda hit: (-hit.length, hit.path, hit.line, hit.name)))

    def _add(self, hits: list[NameHit], path: str, line: int, kind: str, name: str) -> None:
        if len(name) > self.max_len:
            hits.append(NameHit(path=path, line=line, kind=kind, name=name, length=len(name)))


# cf-atom: CODE-NamePlan
class NamePlan:
    def __init__(self, max_len: int = 25) -> None:
        if max_len < 1:
            raise ValueError("max_len must be positive")
        self.max_len = max_len

    def plan(self, hits: tuple[NameHit, ...]) -> tuple[NameFix, ...]:
        used: set[str] = set()
        fixes: list[NameFix] = []
        for hit in hits:
            base = self._short(hit.name)
            name = base
            index = 2
            while name in used:
                suffix = str(index)
                name = base[: self.max_len - len(suffix)] + suffix
                index += 1
            used.add(name)
            fixes.append(NameFix(hit=hit, new_name=name, reason=f"{hit.length}>{self.max_len}"))
        return tuple(fixes)

    def _short(self, name: str) -> str:
        parts = self._parts(name)
        if not parts:
            return name[: self.max_len]
        if len(parts) == 1:
            return parts[0][: self.max_len]
        first = parts[0][:3]
        rest = "".join(part[:2].title() for part in parts[1:])
        short = first + rest
        if len(short) <= self.max_len:
            return short
        initials = parts[0][:3] + "".join(part[:1].title() for part in parts[1:])
        return initials[: self.max_len]

    def _parts(self, name: str) -> tuple[str, ...]:
        raw = name.replace("-", "_")
        split: list[str] = []
        for item in raw.split("_"):
            if not item:
                continue
            start = 0
            for index, char in enumerate(item[1:], start=1):
                prev = item[index - 1]
                if char.isupper() and (not prev.isupper()):
                    split.append(item[start:index].lower())
                    start = index
            split.append(item[start:].lower())
        return tuple(split)


# cf-atom: CODE-NameReport
class NameReport:
    def render(self, fixes: tuple[NameFix, ...]) -> str:
        rows = [
            "| Path | Line | Kind | Name | Suggestion | Reason |",
            "|---|---:|---|---|---|---|",
        ]
        if not fixes:
            rows.append("| none | 0 | none | none | none | clean |")
            return "\n".join(rows)
        for fix in fixes:
            hit = fix.hit
            rows.append(
                f"| {hit.path} | {hit.line} | {hit.kind} | {hit.name} | {fix.new_name} | {fix.reason} |"
            )
        return "\n".join(rows)


# cf-atom: CODE-NameFixJson
class NameFixJson:
    def render(self, fixes: tuple[NameFix, ...]) -> str:
        items = [
            {
                "kind": fix.hit.kind,
                "line": fix.hit.line,
                "name": fix.hit.name,
                "path": fix.hit.path,
                "reason": fix.reason,
                "suggestion": fix.new_name,
            }
            for fix in fixes
        ]
        return json.dumps(items, ensure_ascii=False, sort_keys=True)


# cf-atom: CODE-NameFixJsonRun
class NameFixJsonRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.planr = NamePlan(max_len)
        self.report = NameFixJson()

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        fixes = self.planr.plan(hits)
        return self.report.render(fixes)


# cf-atom: CODE-NameAuditRun
class NameAuditRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.planr = NamePlan(max_len)
        self.report = NameReport()

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        fixes = self.planr.plan(hits)
        return self.report.render(fixes)


# cf-atom: CODE-NameFiles
class NameFiles:
    def discover(self, roots: tuple[Path, ...]) -> tuple[Path, ...]:
        files: list[Path] = []
        for root in roots:
            if root.is_file():
                if root.suffix == ".py":
                    files.append(root)
                continue
            for path in root.rglob("*.py"):
                if self._skip(path, root):
                    continue
                files.append(path)
        return tuple(sorted(set(files), key=lambda path: path.as_posix()))

    def _skip(self, path: Path, root: Path) -> bool:
        try:
            rel = path.relative_to(root)
        except ValueError:
            rel = path
        return any(part in SKIP_DIRS or part.startswith(".") for part in rel.parts)


# cf-atom: CODE-NameProjRun
class NameProjRun:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameAuditRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameProjFixJson
class NameProjFixJson:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameFixJsonRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameStats
class NameStats:
    def summarize(self, hits: tuple[NameHit, ...]) -> NameStat:
        if not hits:
            return NameStat(hits=0, files=0, max_len=0, max_name="")
        paths = {hit.path for hit in hits}
        top = max(hits, key=lambda hit: (hit.length, hit.name))
        return NameStat(hits=len(hits), files=len(paths), max_len=top.length, max_name=top.name)


# cf-atom: CODE-NameGate
class NameGate:
    def __init__(self) -> None:
        self.stats = NameStats()

    def check(self, hits: tuple[NameHit, ...]) -> NameGateRes:
        stat = self.stats.summarize(hits)
        return NameGateRes(ok=stat.hits == 0, stat=stat)


# cf-atom: CODE-NameGateExit
class NameGateExit:
    def code(self, res: NameGateRes) -> int:
        return 0 if res.ok else 1


# cf-atom: CODE-NameGateExitRun
class NameGateExitRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.gate = NameGate()
        self.exit = NameGateExit()

    def code(self, paths: tuple[Path, ...]) -> int:
        hits = self.audit.scan_paths(paths)
        res = self.gate.check(hits)
        return self.exit.code(res)


# cf-atom: CODE-NameGateJson
class NameGateJson:
    def render(self, res: NameGateRes) -> str:
        item = {
            "ok": res.ok,
            "summary": {
                "files": res.stat.files,
                "hits": res.stat.hits,
                "max_len": res.stat.max_len,
                "max_name": res.stat.max_name,
            },
        }
        return json.dumps(item, ensure_ascii=False, sort_keys=True)


# cf-atom: CODE-NameGatePack
class NameGatePack:
    def __init__(self) -> None:
        self.gate = NameGate()

    def render(self, hits: tuple[NameHit, ...]) -> str:
        res = self.gate.check(hits)
        item = {
            "hits": [
                {
                    "kind": hit.kind,
                    "length": hit.length,
                    "line": hit.line,
                    "name": hit.name,
                    "path": hit.path,
                }
                for hit in hits
            ],
            "ok": res.ok,
            "summary": {
                "files": res.stat.files,
                "hits": res.stat.hits,
                "max_len": res.stat.max_len,
                "max_name": res.stat.max_name,
            },
        }
        return json.dumps(item, ensure_ascii=False, sort_keys=True)


# cf-atom: CODE-NameGateMd
class NameGateMd:
    def render(self, res: NameGateRes) -> str:
        status = "pass" if res.ok else "fail"
        rows = [
            "| Metric | Value |",
            "|---|---:|",
            f"| status | {status} |",
            f"| hits | {res.stat.hits} |",
            f"| files | {res.stat.files} |",
            f"| max_len | {res.stat.max_len} |",
            f"| max_name | {res.stat.max_name or 'none'} |",
        ]
        return "\n".join(rows)


# cf-atom: CODE-NameGateLine
class NameGateLine:
    def render(self, res: NameGateRes) -> str:
        status = "pass" if res.ok else "fail"
        name = res.stat.max_name or "none"
        return f"status={status} hits={res.stat.hits} files={res.stat.files} max_len={res.stat.max_len} max_name={name}"


# cf-atom: CODE-NameGateRun
class NameGateRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.gate = NameGate()
        self.report = NameGateMd()

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        res = self.gate.check(hits)
        return self.report.render(res)


# cf-atom: CODE-NameGateLineRun
class NameGateLineRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.gate = NameGate()
        self.report = NameGateLine()

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        res = self.gate.check(hits)
        return self.report.render(res)


# cf-atom: CODE-NameCiRun
class NameCiRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.gate = NameGate()
        self.exit = NameGateExit()
        self.line = NameGateLine()

    def run(self, paths: tuple[Path, ...]) -> tuple[int, str]:
        hits = self.audit.scan_paths(paths)
        res = self.gate.check(hits)
        return self.exit.code(res), self.line.render(res)


# cf-atom: CODE-NameCiJson
class NameCiJson:
    def render(self, code: int, line: str) -> str:
        item = {
            "exit_code": code,
            "line": line,
        }
        return json.dumps(item, ensure_ascii=False, sort_keys=True)


# cf-atom: CODE-NameCiStatusJson
class NameCiStatusJson:
    def render(self, code: int, line: str) -> str:
        item = {
            "exit_code": code,
            "line": line,
            "ok": code == 0,
        }
        return json.dumps(item, ensure_ascii=False, sort_keys=True)


# cf-atom: CODE-NameCiPack
class NameCiPack:
    def render(self, code: int, line: str, hits: tuple[NameHit, ...]) -> str:
        item = {
            "exit_code": code,
            "hits": [
                {
                    "kind": hit.kind,
                    "length": hit.length,
                    "line": hit.line,
                    "name": hit.name,
                    "path": hit.path,
                }
                for hit in hits
            ],
            "line": line,
            "ok": code == 0,
        }
        return json.dumps(item, ensure_ascii=False, sort_keys=True)


# cf-atom: CODE-NameCiSummary
class NameCiSummary:
    def __init__(self) -> None:
        self.stats = NameStats()
        self.kinds = NameKinds()

    def render(self, hits: tuple[NameHit, ...]) -> str:
        stat = self.stats.summarize(hits)
        item = {
            "files": [
                {"count": count, "path": path}
                for path, count in self._file_counts(hits)
            ],
            "kinds": [{"count": count, "kind": kind} for kind, count in self.kinds.group(hits)],
            "longest": self._longest(hits),
            "summary": {
                "files": stat.files,
                "hits": stat.hits,
                "max_len": stat.max_len,
                "max_name": stat.max_name,
            },
        }
        return json.dumps(item, ensure_ascii=False, sort_keys=True)

    def _file_counts(self, hits: tuple[NameHit, ...]) -> tuple[tuple[str, int], ...]:
        counts: dict[str, int] = {}
        for hit in hits:
            counts[hit.path] = counts.get(hit.path, 0) + 1
        return tuple(sorted(counts.items(), key=lambda item: (-item[1], item[0])))

    def _longest(self, hits: tuple[NameHit, ...]) -> dict[str, int | str] | None:
        if not hits:
            return None
        hit = sorted(hits, key=lambda row: (-row.length, row.path, row.line, row.name))[0]
        return {
            "kind": hit.kind,
            "length": hit.length,
            "line": hit.line,
            "name": hit.name,
            "path": hit.path,
        }


# cf-atom: CODE-NameCiNote
class NameCiNote:
    def render(self, hits: tuple[NameHit, ...]) -> str:
        if not hits:
            return "clean"
        rows = [
            f"{hit.path}:{hit.line}: {hit.kind} {hit.name} length={hit.length}"
            for hit in hits
        ]
        return "\n".join(rows)


# cf-atom: CODE-NameCiNoteTop
class NameCiNoteTop:
    def __init__(self) -> None:
        self.top = NameTop()

    def render(self, hits: tuple[NameHit, ...], limit: int = 5) -> str:
        if not hits:
            return "clean"
        rows = [
            f"{hit.path}:{hit.line}: {hit.kind} {hit.name} length={hit.length}"
            for hit in self.top.rank(hits, limit)
        ]
        omitted = len(hits) - len(rows)
        if omitted:
            rows.append(f"omitted={omitted}")
        return "\n".join(rows)


# cf-atom: CODE-NameCiNoteMd
class NameCiNoteMd:
    def render(self, note: str) -> str:
        body = note.rstrip("\n")
        return f"```text\n{body}\n```"


# cf-atom: CODE-NameCiNoteSec
class NameCiNoteSec:
    def __init__(self) -> None:
        self.note_md = NameCiNoteMd()

    def render(self, title: str, note: str) -> str:
        heading = title.strip() or "Identifier naming notes"
        return f"### {heading}\n\n{self.note_md.render(note)}"


# cf-atom: CODE-NameCiStatusRun
class NameCiStatusRun:
    def __init__(self, max_len: int = 25) -> None:
        self.runner = NameCiRun(max_len)
        self.report = NameCiStatusJson()

    def render(self, paths: tuple[Path, ...]) -> str:
        code, line = self.runner.run(paths)
        return self.report.render(code, line)


# cf-atom: CODE-NameCiPackRun
class NameCiPackRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.gate = NameGate()
        self.exit = NameGateExit()
        self.line = NameGateLine()
        self.report = NameCiPack()

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        res = self.gate.check(hits)
        code = self.exit.code(res)
        return self.report.render(code, self.line.render(res), hits)


# cf-atom: CODE-NameCiSummaryRun
class NameCiSummaryRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.report = NameCiSummary()

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        return self.report.render(hits)


# cf-atom: CODE-NameCiNoteRun
class NameCiNoteRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.report = NameCiNote()

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        return self.report.render(hits)


# cf-atom: CODE-NameCiNoteTopRun
class NameCiNoteTopRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.report = NameCiNoteTop()

    def render(self, paths: tuple[Path, ...], limit: int = 5) -> str:
        hits = self.audit.scan_paths(paths)
        return self.report.render(hits, limit)


# cf-atom: CODE-NameCiNoteMdRun
class NameCiNoteMdRun:
    def __init__(self, max_len: int = 25) -> None:
        self.runner = NameCiNoteTopRun(max_len)
        self.report = NameCiNoteMd()

    def render(self, paths: tuple[Path, ...], limit: int = 5) -> str:
        note = self.runner.render(paths, limit)
        return self.report.render(note)


# cf-atom: CODE-NameCiNoteSecRun
class NameCiNoteSecRun:
    def __init__(self, max_len: int = 25) -> None:
        self.runner = NameCiNoteTopRun(max_len)
        self.report = NameCiNoteSec()

    def render(self, paths: tuple[Path, ...], title: str, limit: int = 5) -> str:
        note = self.runner.render(paths, limit)
        return self.report.render(title, note)


# cf-atom: CODE-NameProjCiNoteTopRun
class NameProjCiNoteTopRun:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameCiNoteTopRun(max_len)

    def render(self, roots: tuple[Path, ...], limit: int = 5) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths, limit)


# cf-atom: CODE-NameProjCiNoteMdRun
class NameProjCiNoteMdRun:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameCiNoteMdRun(max_len)

    def render(self, roots: tuple[Path, ...], limit: int = 5) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths, limit)


# cf-atom: CODE-NameProjCiNoteSecRun
class NameProjCiNoteSecRun:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameCiNoteSecRun(max_len)

    def render(self, roots: tuple[Path, ...], title: str, limit: int = 5) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths, title, limit)


# cf-atom: CODE-NameOpsGuide
class NameOpsGuide:
    def render(self, max_len: int = 25, limit: int = 5) -> str:
        return "\n".join(
            [
                "# Identifier Naming Audit",
                "",
                "## Purpose",
                "",
                "Use the naming audit before review to catch Python identifiers that are hard to scan in CI logs.",
                "",
                "## Defaults",
                "",
                f"- Maximum identifier length: {max_len}",
                f"- Review note limit: {limit}",
                "- Audited syntax: class names, function names, arguments, and assigned names",
                "",
                "## Review Workflow",
                "",
                "1. Discover Python files from the project root.",
                "2. Run the length audit with the agreed maximum identifier length.",
                "3. Publish a bounded Markdown section in the review thread.",
                "4. Rename only identifiers whose shorter names preserve the research concept.",
                "",
                "## CI Payloads",
                "",
                "- `NameProjCiStatusRun` returns machine-readable pass/fail status.",
                "- `NameProjCiPackRun` returns status plus every finding for annotations.",
                "- `NameProjCiNoteSecRun` returns a bounded Markdown section for reviewers.",
            ]
        )


# cf-atom: CODE-NameProjOpsDoc
class NameProjOpsDoc:
    def __init__(self, max_len: int = 25) -> None:
        self.guide = NameOpsGuide()
        self.note = NameProjCiNoteSecRun(max_len)
        self.max_len = max_len

    def render(self, roots: tuple[Path, ...], limit: int = 5) -> str:
        return "\n\n".join(
            [
                self.guide.render(self.max_len, limit),
                self.note.render(roots, "Current naming findings", limit),
            ]
        )


# cf-atom: CODE-NameCiJsonRun
class NameCiJsonRun:
    def __init__(self, max_len: int = 25) -> None:
        self.runner = NameCiRun(max_len)
        self.report = NameCiJson()

    def render(self, paths: tuple[Path, ...]) -> str:
        code, line = self.runner.run(paths)
        return self.report.render(code, line)


# cf-atom: CODE-NameGateJsonRun
class NameGateJsonRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.gate = NameGate()
        self.report = NameGateJson()

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        res = self.gate.check(hits)
        return self.report.render(res)


# cf-atom: CODE-NameGatePackRun
class NameGatePackRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.report = NameGatePack()

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        return self.report.render(hits)


# cf-atom: CODE-NameProjGate
class NameProjGate:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameGateRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameProjGateJson
class NameProjGateJson:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameGateJsonRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameProjGatePack
class NameProjGatePack:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameGatePackRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameProjGateExit
class NameProjGateExit:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameGateExitRun(max_len)

    def code(self, roots: tuple[Path, ...]) -> int:
        paths = self.files.discover(roots)
        return self.runner.code(paths)


# cf-atom: CODE-NameProjGateLine
class NameProjGateLine:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameGateLineRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameProjCiRun
class NameProjCiRun:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameCiRun(max_len)

    def run(self, roots: tuple[Path, ...]) -> tuple[int, str]:
        paths = self.files.discover(roots)
        return self.runner.run(paths)


# cf-atom: CODE-NameProjCiJsonRun
class NameProjCiJsonRun:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameCiJsonRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameProjCiStatusRun
class NameProjCiStatusRun:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameCiStatusRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameProjCiPackRun
class NameProjCiPackRun:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameCiPackRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameProjCiSummaryRun
class NameProjCiSummaryRun:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameCiSummaryRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameProjCiNoteRun
class NameProjCiNoteRun:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameCiNoteRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameStatMd
class NameStatMd:
    def render(self, stat: NameStat) -> str:
        rows = [
            "| Metric | Value |",
            "|---|---:|",
            f"| hits | {stat.hits} |",
            f"| files | {stat.files} |",
            f"| max_len | {stat.max_len} |",
            f"| max_name | {stat.max_name or 'none'} |",
        ]
        return "\n".join(rows)


# cf-atom: CODE-NameStatJson
class NameStatJson:
    def render(self, stat: NameStat) -> str:
        item = {
            "files": stat.files,
            "hits": stat.hits,
            "max_len": stat.max_len,
            "max_name": stat.max_name,
        }
        return json.dumps(item, ensure_ascii=False, sort_keys=True)


# cf-atom: CODE-NameKinds
class NameKinds:
    def group(self, hits: tuple[NameHit, ...]) -> tuple[tuple[str, int], ...]:
        counts: dict[str, int] = {}
        for hit in hits:
            counts[hit.kind] = counts.get(hit.kind, 0) + 1
        return tuple(sorted(counts.items(), key=lambda item: (-item[1], item[0])))


# cf-atom: CODE-NameKindMd
class NameKindMd:
    def render(self, rows: tuple[tuple[str, int], ...]) -> str:
        lines = [
            "| Kind | Count |",
            "|---|---:|",
        ]
        if not rows:
            lines.append("| none | 0 |")
            return "\n".join(lines)
        for kind, count in rows:
            lines.append(f"| {kind} | {count} |")
        return "\n".join(lines)


# cf-atom: CODE-NameDash
class NameDash:
    def __init__(self) -> None:
        self.stats = NameStats()
        self.stat_md = NameStatMd()
        self.kinds = NameKinds()
        self.kind_md = NameKindMd()

    def render(self, hits: tuple[NameHit, ...]) -> str:
        stat = self.stats.summarize(hits)
        rows = self.kinds.group(hits)
        return "\n\n".join((self.stat_md.render(stat), self.kind_md.render(rows)))


# cf-atom: CODE-NameDashJson
class NameDashJson:
    def __init__(self) -> None:
        self.stats = NameStats()
        self.kinds = NameKinds()

    def render(self, hits: tuple[NameHit, ...]) -> str:
        stat = self.stats.summarize(hits)
        item = {
            "kinds": [{"count": count, "kind": kind} for kind, count in self.kinds.group(hits)],
            "summary": {
                "files": stat.files,
                "hits": stat.hits,
                "max_len": stat.max_len,
                "max_name": stat.max_name,
            },
        }
        return json.dumps(item, ensure_ascii=False, sort_keys=True)


# cf-atom: CODE-NameDashJsonRun
class NameDashJsonRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.report = NameDashJson()

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        return self.report.render(hits)


# cf-atom: CODE-NameDashRun
class NameDashRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.dash = NameDash()

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        return self.dash.render(hits)


# cf-atom: CODE-NameProjDash
class NameProjDash:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameDashRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NamePjDashJson
class NamePjDashJson:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameDashJsonRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameTop
class NameTop:
    def rank(self, hits: tuple[NameHit, ...], limit: int = 5) -> tuple[NameHit, ...]:
        if limit < 1:
            raise ValueError("limit must be positive")
        rows = sorted(hits, key=lambda hit: (-hit.length, hit.path, hit.line, hit.name))
        return tuple(rows[:limit])


# cf-atom: CODE-NameTopMd
class NameTopMd:
    def render(self, hits: tuple[NameHit, ...]) -> str:
        lines = [
            "| Path | Line | Kind | Name | Length |",
            "|---|---:|---|---|---:|",
        ]
        if not hits:
            lines.append("| none | 0 | none | none | 0 |")
            return "\n".join(lines)
        for hit in hits:
            lines.append(f"| {hit.path} | {hit.line} | {hit.kind} | {hit.name} | {hit.length} |")
        return "\n".join(lines)


# cf-atom: CODE-NameTopJson
class NameTopJson:
    def __init__(self) -> None:
        self.top = NameTop()
        self.report = NameJson()

    def render(self, hits: tuple[NameHit, ...], limit: int = 5) -> str:
        rows = self.top.rank(hits, limit)
        return self.report.render(rows)


# cf-atom: CODE-NameTopJsonRun
class NameTopJsonRun:
    def __init__(self, max_len: int = 25, limit: int = 5) -> None:
        self.audit = NameLenAudit(max_len)
        self.report = NameTopJson()
        self.limit = limit

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        return self.report.render(hits, self.limit)


# cf-atom: CODE-NameTopRun
class NameTopRun:
    def __init__(self, max_len: int = 25, limit: int = 5) -> None:
        self.audit = NameLenAudit(max_len)
        self.top = NameTop()
        self.markdown = NameTopMd()
        self.limit = limit

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        rows = self.top.rank(hits, self.limit)
        return self.markdown.render(rows)


# cf-atom: CODE-NameProjTop
class NameProjTop:
    def __init__(self, max_len: int = 25, limit: int = 5) -> None:
        self.files = NameFiles()
        self.runner = NameTopRun(max_len, limit)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameProjTopJson
class NameProjTopJson:
    def __init__(self, max_len: int = 25, limit: int = 5) -> None:
        self.files = NameFiles()
        self.runner = NameTopJsonRun(max_len, limit)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameFull
class NameFull:
    def __init__(self, limit: int = 5) -> None:
        self.dash = NameDash()
        self.top = NameTop()
        self.top_md = NameTopMd()
        self.limit = limit

    def render(self, hits: tuple[NameHit, ...]) -> str:
        rows = self.top.rank(hits, self.limit)
        return "\n\n".join((self.dash.render(hits), self.top_md.render(rows)))


# cf-atom: CODE-NameFullRun
class NameFullRun:
    def __init__(self, max_len: int = 25, limit: int = 5) -> None:
        self.audit = NameLenAudit(max_len)
        self.report = NameFull(limit)

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        return self.report.render(hits)


# cf-atom: CODE-NameProjFull
class NameProjFull:
    def __init__(self, max_len: int = 25, limit: int = 5) -> None:
        self.files = NameFiles()
        self.runner = NameFullRun(max_len, limit)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)


# cf-atom: CODE-NameJson
class NameJson:
    def render(self, hits: tuple[NameHit, ...]) -> str:
        items = [
            {
                "kind": hit.kind,
                "length": hit.length,
                "line": hit.line,
                "name": hit.name,
                "path": hit.path,
            }
            for hit in hits
        ]
        return json.dumps(items, ensure_ascii=False, sort_keys=True)


# cf-atom: CODE-NameJsonRun
class NameJsonRun:
    def __init__(self, max_len: int = 25) -> None:
        self.audit = NameLenAudit(max_len)
        self.report = NameJson()

    def render(self, paths: tuple[Path, ...]) -> str:
        hits = self.audit.scan_paths(paths)
        return self.report.render(hits)


# cf-atom: CODE-NameProjJson
class NameProjJson:
    def __init__(self, max_len: int = 25) -> None:
        self.files = NameFiles()
        self.runner = NameJsonRun(max_len)

    def render(self, roots: tuple[Path, ...]) -> str:
        paths = self.files.discover(roots)
        return self.runner.render(paths)
