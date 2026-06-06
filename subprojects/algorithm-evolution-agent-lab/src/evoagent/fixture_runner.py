from __future__ import annotations

import argparse
from pathlib import Path

from evoagent.fixtures import ToyTabularFixture


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="python -m evoagent.fixture_runner")
    parser.add_argument("--fixture", choices=["toy-tabular"], required=True)
    parser.add_argument("--candidate", choices=["majority", "linear-threshold", "x1-threshold"], required=True)
    parser.add_argument("--seed", type=int, default=0)
    parser.add_argument("--output-dir", default=".")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.fixture != "toy-tabular":
        raise ValueError(f"unsupported fixture: {args.fixture}")
    result = ToyTabularFixture().write_run_artifacts(
        Path(args.output_dir),
        candidate=args.candidate,
        seed=args.seed,
    )
    print(result.analysis)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
