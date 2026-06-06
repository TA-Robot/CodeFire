from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import ExperimentRun
from evoagent.review import ReviewObjection


@dataclass(frozen=True)
class Limitation:
    kind: str
    detail: str


# cf-atom: CODE-LimitationSectionGenerator
class LimitationSectionGenerator:
    def generate(
        self,
        *,
        runs: list[ExperimentRun],
        expected_benchmarks: tuple[str, ...],
        covered_benchmarks: tuple[str, ...],
        open_objections: tuple[ReviewObjection, ...],
    ) -> tuple[Limitation, ...]:
        limitations: list[Limitation] = []
        failed_runs = [run for run in runs if not run.succeeded]
        if failed_runs:
            limitations.append(Limitation("failed_runs", f"{len(failed_runs)} failed run(s) remain in the record"))

        missing_benchmarks = tuple(sorted(set(expected_benchmarks) - set(covered_benchmarks)))
        if missing_benchmarks:
            limitations.append(
                Limitation("benchmark_gap", "missing benchmark coverage: " + ", ".join(missing_benchmarks))
            )

        for objection in open_objections:
            limitations.append(Limitation("review_objection", objection.summary))

        if not limitations:
            limitations.append(Limitation("none_recorded", "no current failed runs, benchmark gaps, or reviewer objections"))
        return tuple(limitations)
