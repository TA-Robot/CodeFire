import unittest
from pathlib import Path

from evoagent.limitations import LimitationSectionGenerator
from evoagent.models import ExperimentPlan, ExperimentRun, Hypothesis
from evoagent.review import ReviewLedger


# cf-atom: TEST-limitation-section-includes-failures-gaps-and-objections
class LimitationSectionGeneratorTests(unittest.TestCase):
    def test_limitation_section_includes_failures_gaps_and_objections(self):
        review = ReviewLedger()
        objection = review.add_objection(
            claim_id="claim",
            summary="external baseline has not been reproduced",
            severity="blocking",
        )
        limitations = LimitationSectionGenerator().generate(
            runs=[run("failed", returncode=1)],
            expected_benchmarks=("toy-tabular", "synthetic-sequence"),
            covered_benchmarks=("toy-tabular",),
            open_objections=(objection,),
        )

        kinds = {limitation.kind for limitation in limitations}
        self.assertEqual(kinds, {"failed_runs", "benchmark_gap", "review_objection"})

    def test_limitation_section_records_empty_state(self):
        limitations = LimitationSectionGenerator().generate(
            runs=[],
            expected_benchmarks=(),
            covered_benchmarks=(),
            open_objections=(),
        )

        self.assertEqual(limitations[0].kind, "none_recorded")


def run(run_id: str, *, returncode: int) -> ExperimentRun:
    hypothesis = Hypothesis("candidate", "rationale", expected_gain=0.01, novelty=0.1)
    plan = ExperimentPlan(hypothesis, "toy-tabular", "baseline", "accuracy", estimated_cost=0.1)
    return ExperimentRun(
        run_id=run_id,
        plan=plan,
        command="run",
        cwd=Path("."),
        returncode=returncode,
        stdout="",
        stderr="",
        duration_seconds=1.0,
    )


if __name__ == "__main__":
    unittest.main()
