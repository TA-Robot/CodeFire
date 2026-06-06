import unittest
from pathlib import Path

from evoagent.models import ExperimentGoal, ExperimentPlan, ExperimentResult, ExperimentRun, Hypothesis, RunArtifact
from evoagent.program import ResearchProgram
from evoagent.progress import ResearchProgressMeter


# cf-atom: TEST-research-progress-meter-scores-sustained-progress
class ResearchProgressMeterTests(unittest.TestCase):
    def test_research_progress_meter_scores_sustained_progress(self):
        program = ResearchProgram(ExperimentGoal("Improve sample efficiency"))
        hypothesis = program.add_hypothesis(Hypothesis("filter", "reduce variance", 0.03, 0.4))
        plan = ExperimentPlan(hypothesis, "toy-tabular", "baseline", "accuracy", 0.1)
        run = ExperimentRun(
            "run_progress",
            plan,
            "python experiment.py",
            Path.cwd(),
            0,
            "ok",
            "",
            0.1,
            artifacts=(RunArtifact("metrics.json", True, 10),),
        )
        program.add_run_evidence(claim_id="claim", run=run, summary="usable run", confidence=0.6)
        program.record_decision(
            summary="replicate",
            rationale="initial result improved",
            evidence_refs=("ev_ref",),
            rejected_alternatives=("stop now",),
            follow_up_work=("repeat seed",),
        )
        result = ExperimentResult(plan, metric_value=0.84, baseline_value=0.80, confidence=0.7)

        report = ResearchProgressMeter().evaluate(
            program,
            results=(result,),
            previous_uncertainty=0.9,
            current_uncertainty=0.5,
        )

        self.assertEqual(report.reproducible_runs, 1)
        self.assertEqual(report.improved_results, 1)
        self.assertEqual(report.bottlenecks, ())
        self.assertGreater(report.progress_score, 0.8)


if __name__ == "__main__":
    unittest.main()
