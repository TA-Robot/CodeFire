from __future__ import annotations

import json
import re
from typing import Protocol

from evoagent.models import ExperimentGoal, Hypothesis


class Planner(Protocol):
    def propose(self, goal: ExperimentGoal, count: int = 3) -> list[Hypothesis]:
        """Return candidate research hypotheses for a goal."""


def parse_hypotheses_json(text: str) -> list[Hypothesis]:
    payload = json.loads(extract_json_object(text))
    raw_hypotheses = payload.get("hypotheses")
    if not isinstance(raw_hypotheses, list):
        raise ValueError("planner output missing hypotheses list")

    hypotheses: list[Hypothesis] = []
    for index, item in enumerate(raw_hypotheses, start=1):
        if not isinstance(item, dict):
            raise ValueError(f"hypothesis {index} must be an object")
        title = require_string(item, "title", index)
        rationale = require_string(item, "rationale", index)
        expected_gain = require_score(item, "expected_gain", index)
        novelty = require_score(item, "novelty", index)
        hypotheses.append(
            Hypothesis(
                title=title,
                rationale=rationale,
                expected_gain=expected_gain,
                novelty=novelty,
            )
        )
    return hypotheses


def extract_json_object(text: str) -> str:
    stripped = text.strip()
    fenced = re.search(r"```(?:json)?\s*(\{.*?\})\s*```", stripped, re.DOTALL)
    if fenced:
        return fenced.group(1)
    start = stripped.find("{")
    end = stripped.rfind("}")
    if start == -1 or end == -1 or end < start:
        raise ValueError("planner output did not contain a JSON object")
    return stripped[start : end + 1]


def require_string(item: dict[str, object], key: str, index: int) -> str:
    value = item.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"hypothesis {index} missing string {key}")
    return value.strip()


def require_score(item: dict[str, object], key: str, index: int) -> float:
    value = item.get(key)
    if not isinstance(value, (int, float)):
        raise ValueError(f"hypothesis {index} missing numeric {key}")
    score = float(value)
    if score < 0.0 or score > 1.0:
        raise ValueError(f"hypothesis {index} {key} must be between 0 and 1")
    return score


from evoagent.codex_planner import CodexPlanner  # noqa: E402
from evoagent.static_planner import StaticPlanner  # noqa: E402

__all__ = [
    "CodexPlanner",
    "Planner",
    "StaticPlanner",
    "parse_hypotheses_json",
]
