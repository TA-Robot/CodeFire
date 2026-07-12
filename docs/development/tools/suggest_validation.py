#!/usr/bin/env python3
"""Suggest deduplicated validation commands for changed repository paths."""

from __future__ import annotations

import argparse
import fnmatch
import json
import subprocess
import sys
from pathlib import Path, PurePosixPath
from typing import Any

from ai_contracts import load_object, repository_path_valid


MAP_PATH = Path("docs/development/ai/focused-test-map.v1.json")


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def changed_paths(root: Path) -> list[str]:
    result = subprocess.run(
        ["git", "status", "--porcelain=v1", "-z", "--untracked-files=all"],
        cwd=root,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise ValueError(result.stderr.decode("utf-8", errors="replace").strip() or "git status failed")
    return parse_porcelain_paths(result.stdout)


def parse_porcelain_paths(payload: bytes) -> list[str]:
    """Return both destination and source for NUL-delimited rename/copy records."""

    records = payload.split(b"\0")
    paths: list[str] = []
    index = 0
    while index < len(records):
        record = records[index]
        index += 1
        if not record:
            continue
        text = record.decode("utf-8", errors="surrogateescape")
        if len(text) < 4:
            raise ValueError(f"unexpected porcelain record: {text!r}")
        status = text[:2]
        path = text[3:]
        paths.append(PurePosixPath(path).as_posix())
        if "R" in status or "C" in status:
            if index >= len(records) or not records[index]:
                raise ValueError("rename/copy record is missing its second path")
            source_path = records[index].decode("utf-8", errors="surrogateescape")
            index += 1
            paths.append(PurePosixPath(source_path).as_posix())
    return sorted(set(paths))


def path_matches(path: str, pattern: str) -> bool:
    if pattern.endswith("/**"):
        prefix = pattern[:-3]
        return path == prefix or path.startswith(f"{prefix}/")
    return fnmatch.fnmatchcase(path, pattern)


def suggestions(mapping: dict[str, Any], paths: list[str]) -> dict[str, Any]:
    if mapping.get("schema") != "codefire.ai_focused_test_map.v1":
        raise ValueError("unexpected focused-test map schema")
    commands: list[dict[str, Any]] = []
    command_index: dict[tuple[str, str], int] = {}
    path_rules: dict[str, list[str]] = {path: [] for path in paths}
    for rule in mapping.get("rules", []):
        rule_id = rule["id"]
        matched = [
            path for path in paths if any(path_matches(path, pattern) for pattern in rule["paths"])
        ]
        if not matched:
            continue
        for path in matched:
            path_rules[path].append(rule_id)
        for command in rule["commands"]:
            cwd = command.get("cwd", ".")
            key = (cwd, command["command"])
            if key in command_index:
                commands[command_index[key]]["rules"].append(rule_id)
                continue
            item = {
                "command": command["command"],
                "cwd": cwd,
                "capability": command["capability"],
                "purpose": command["purpose"],
                "rules": [rule_id],
            }
            command_index[key] = len(commands)
            commands.append(item)
    return {
        "schema": "codefire.ai_validation_suggestions.v1",
        "paths": paths,
        "path_rules": path_rules,
        "uncovered_paths": [path for path, rules in path_rules.items() if not rules],
        "commands": commands,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*")
    parser.add_argument("--root", type=Path, default=repository_root())
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    try:
        paths = args.paths or changed_paths(root)
        unsafe = [path for path in paths if not repository_path_valid(path)]
        if unsafe:
            raise ValueError(f"paths must be repository-relative: {unsafe}")
        payload = suggestions(load_object(root / MAP_PATH), sorted(set(paths)))
    except (FileNotFoundError, KeyError, json.JSONDecodeError, TypeError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        for item in payload["commands"]:
            prefix = "" if item["cwd"] == "." else f"(cd {item['cwd']} &&) "
            print(f"{prefix}{item['command']}")
        if payload["uncovered_paths"]:
            print("Uncovered paths:", file=sys.stderr)
            for path in payload["uncovered_paths"]:
                print(f"- {path}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
