#!/usr/bin/env python3
"""Prepare a deterministic historical eval input without exposing its reference answer."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from ai_contracts import (
    load_object,
    sha256_json,
    validate_historical_eval_corpus,
)


CORPUS_PATH = Path("docs/development/ai/evals/historical-cases.v1.json")
MATRIX_PATH = Path("docs/development/ai/role-routing-matrix.json")


def repository_root() -> Path:
    return Path(__file__).resolve().parents[3]


def build_candidate_input(corpus: dict[str, Any], case_id: str) -> dict[str, Any]:
    cases = corpus.get("cases", [])
    selected = next(
        (case for case in cases if isinstance(case, dict) and case.get("id") == case_id),
        None,
    )
    if selected is None:
        raise KeyError(case_id)
    public_case = {
        "schema": "codefire.ai_historical_eval_input.v1",
        "case_id": selected["id"],
        "task_class": selected["task_class"],
        "required_tier": selected["risk"]["expected_tier"],
        "prompt": selected["prompt"],
        "candidate_context": selected["candidate_context"],
        "response_requirements": [
            "State the diagnosis and why it follows from the observations.",
            "List protected invariants and the smallest safe change or decision.",
            "List deterministic validation evidence and unresolved unknowns.",
            "Do not claim that unavailable tests passed.",
        ],
        "corpus_sha256": sha256_json(corpus),
        "reference_answer_sha256": sha256_json(selected["oracle"]),
    }
    public_case["input_sha256"] = sha256_json(public_case)
    return public_case


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=repository_root())
    parser.add_argument("--case", help="Historical case id to materialize.")
    parser.add_argument("--list", action="store_true", help="List available cases without their reference answers.")
    args = parser.parse_args()
    root = args.root.resolve()
    try:
        corpus = load_object(root / CORPUS_PATH)
        matrix = load_object(root / MATRIX_PATH)
    except (FileNotFoundError, json.JSONDecodeError, ValueError) as exc:
        print(f"failed to load eval inputs: {exc}", file=sys.stderr)
        return 1
    findings = validate_historical_eval_corpus(corpus, matrix, root, CORPUS_PATH.as_posix())
    errors = [finding for finding in findings if finding.level == "error"]
    if errors:
        for finding in errors:
            print(f"{finding.code}: {finding.path}: {finding.message}", file=sys.stderr)
        return 1
    if args.list:
        payload = {
            "schema": "codefire.ai_historical_eval_index.v1",
            "corpus_sha256": sha256_json(corpus),
            "cases": [
                {
                    "id": case["id"],
                    "task_class": case["task_class"],
                    "required_tier": case["risk"]["expected_tier"],
                }
                for case in corpus["cases"]
            ],
        }
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0
    if not args.case:
        parser.error("--case is required unless --list is used")
    try:
        payload = build_candidate_input(corpus, args.case)
    except KeyError:
        print(f"unknown historical eval case: {args.case}", file=sys.stderr)
        return 2
    print(json.dumps(payload, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
