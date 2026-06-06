import sys
import tempfile
import unittest
from pathlib import Path

from evoagent.models import CandidateAlgorithm, ExperimentGoal, ExperimentPlan, Hypothesis
from evoagent.program import ResearchProgram
from evoagent.reporting import LivingResearchReport
from evoagent.runner import LocalExperimentRunner


# cf-atom: TEST-living-research-report-summarizes-program
class LivingResearchReportTests(unittest.TestCase):
    def test_living_research_report_summarizes_program(self):
        program = ResearchProgram(ExperimentGoal("Find stronger low-budget tabular algorithms"))
        hypothesis = Hypothesis("feature bagging", "reduce variance across cheap models", 0.05, 0.7)
        plan = ExperimentPlan(
            hypothesis,
            "toy-tabular",
            "logistic-regression",
            "accuracy",
            0.2,
            command=f"{sys.executable} -c \"print('ok')\"",
        )
        candidate = CandidateAlgorithm("candidate-feature-bagging", hypothesis, plan)
        program.add_candidate(candidate)

        with tempfile.TemporaryDirectory() as tmp:
            run = LocalExperimentRunner().run(plan, cwd=Path(tmp))

        evidence = program.add_run_evidence(
            claim_id="claim-local-improvement",
            run=run,
            summary="Toy run completed against logistic baseline.",
            confidence=0.45,
        )
        program.record_decision(
            summary="Keep candidate for replication",
            rationale="The first signal is weak but actionable.",
            evidence_refs=(evidence.evidence_id,),
        )

        report = LivingResearchReport().render(program)

        self.assertIn("# Living Research Report", report)
        self.assertIn("Find stronger low-budget tabular algorithms", report)
        self.assertIn("| candidate-feature-bagging | feature bagging |", report)
        self.assertIn("| " + run.run_id + " | toy-tabular | accuracy | logistic-regression | passed |", report)
        self.assertIn("Low-confidence evidence remains for: claim-local-improvement", report)
        self.assertIn("Keep candidate for replication", report)

    def test_living_research_report_writes_markdown_file(self):
        program = ResearchProgram(ExperimentGoal("Improve optimizer benchmark"))

        with tempfile.TemporaryDirectory() as tmp:
            path = LivingResearchReport().write(program, Path(tmp) / "reports" / "living.md")

            self.assertTrue(path.exists())
            self.assertIn("Improve optimizer benchmark", path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
