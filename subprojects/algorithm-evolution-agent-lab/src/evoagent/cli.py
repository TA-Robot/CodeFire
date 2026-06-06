from __future__ import annotations

import argparse

from evoagent.loop import EvolutionAgent
from evoagent.models import ExperimentGoal
from evoagent.planner import CodexPlanner, StaticPlanner
from evoagent.scoring import score_candidate


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="evoagent")
    parser.add_argument("--goal", required=True)
    parser.add_argument("--planner", choices=["static", "codex"], default="static")
    parser.add_argument("--ideas", type=int, default=2)
    parser.add_argument("--codex-model")
    parser.add_argument("--codex-timeout", type=float, default=120.0)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    goal = ExperimentGoal(objective=args.goal)
    planner = (
        CodexPlanner(model=args.codex_model, timeout_seconds=args.codex_timeout)
        if args.planner == "codex"
        else StaticPlanner()
    )
    agent = EvolutionAgent(goal)
    agent.seed(planner.propose(goal, count=args.ideas))
    selected = agent.next_candidate()
    print(f"selected: {selected.name}")
    print(f"title: {selected.hypothesis.title}")
    print(f"score: {score_candidate(selected):.4f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
