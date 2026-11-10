from __future__ import annotations

from typing import Any


# cf-atom: CODE-SeedSweepPlanner
class SeedSweepPlanner:
    @classmethod
    def from_range(cls, start: int, count: int) -> list[int]:
        if isinstance(count, bool) or not isinstance(count, int) or count <= 0:
            raise ValueError("count must be a positive integer")
        if isinstance(start, bool) or not isinstance(start, int) or start < 0:
            raise ValueError("start must be a non-negative integer")
        return [start + offset for offset in range(count)]

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
            if seed < 0:
                raise ValueError(f"seed must be non-negative: {seed}")
            if seed in unique:
                raise ValueError(f"duplicate seed: {seed}")
            unique.append(seed)
        return [{**base_config, "seed": seed} for seed in unique[:max_runs]]
