from __future__ import annotations


# cf-atom: CODE-RunLabeler
class RunLabeler:
    @staticmethod
    def format_run_label(experiment: str, seed: int) -> str:
        return f"{experiment}-seed{seed:04d}"
