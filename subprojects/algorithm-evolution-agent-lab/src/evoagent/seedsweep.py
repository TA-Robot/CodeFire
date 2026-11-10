from __future__ import annotations

from typing import Any


# cf-atom: CODE-SeedSweepPlanner
class SeedSweepPlanner:
    def plan(
        self,
        base_config: dict[str, Any],
        seeds: list[int],
        max_runs: int,
    ) -> list[dict[str, Any]]:
        if isinstance(max_runs, bool) or not isinstance(max_runs, int) or max_runs <= 0:
            raise ValueError("max_runs must be a positive integer")
        unique: list[int] = []
        for seed in seeds:
            if isinstance(seed, bool) or not isinstance(seed, int):
                raise ValueError("seeds must be integers")
            if seed not in unique:
                unique.append(seed)
        return [{**base_config, "seed": seed} for seed in unique[:max_runs]]
