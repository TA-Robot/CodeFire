import unittest
from pathlib import Path

from evoagent.challenge import load_challenge_definition
from evoagent.models import CandidateAlgorithm, ExperimentGoal, ExperimentPlan, ExperimentRun, Hypothesis, RunArtifact
from evoagent.program import ResearchProgram
from evoagent.reproduction import ReproductionChecklistBuilder


# cf-atom: TEST-reproduction-checklist-detects-missing-evidence
class ReproductionChecklistTests(unittest.TestCase):
    def test_reproduction_checklist_passes_with_repeated_artifact_runs(self):
        program = program_with_candidate(command="python -m evoagent.fixture_runner")
        program.record_run(run("run-1"))
        program.record_run(run("run-2"))
        challenge = load_challenge_definition("challenges/toy_tabular_sample_efficiency.json")

        checklist = ReproductionChecklistBuilder().build(program=program, challenge=challenge)

        self.assertTrue(checklist.passed)
        self.assertEqual(checklist.failures, ())

    def test_reproduction_checklist_detects_missing_evidence(self):
        program = program_with_candidate(command="")
        challenge = load_challenge_definition("challenges/toy_tabular_sample_efficiency.json")

        checklist = ReproductionChecklistBuilder().build(program=program, challenge=challenge)

        failed_names = {item.name for item in checklist.failures}
        self.assertIn("candidate_commands", failed_names)
        self.assertIn("run_artifacts", failed_names)
        self.assertIn("repeated_runs", failed_names)


def program_with_candidate(command: str) -> ResearchProgram:
    program = ResearchProgram(ExperimentGoal("Improve toy tabular sample efficiency"))
    hypothesis = Hypothesis("calibrated thresholding", "reduce variance near the boundary", 0.03, 0.5)
    plan = ExperimentPlan(
        hypothesis,
        "toy-tabular",
        "regularized-polynomial",
        "validation_score",
        0.2,
        command=command,
        artifact_paths=("metrics.json",),
    )
    program.add_candidate(CandidateAlgorithm("candidate-calibrated", hypothesis, plan))
    return program


def run(run_id: str) -> ExperimentRun:
    hypothesis = Hypothesis("calibrated thresholding", "reduce variance near the boundary", 0.03, 0.5)
    plan = ExperimentPlan(
        hypothesis,
        "toy-tabular",
        "regularized-polynomial",
        "validation_score",
        0.2,
        command="python -m evoagent.fixture_runner",
        artifact_paths=("metrics.json",),
    )
    return ExperimentRun(
        run_id=run_id,
        plan=plan,
        command=plan.command,
        cwd=Path("."),
        returncode=0,
        stdout="ok",
        stderr="",
        duration_seconds=0.1,
        artifacts=(RunArtifact("metrics.json", exists=True, size_bytes=128),),
    )


if __name__ == "__main__":
    unittest.main()
