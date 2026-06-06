import unittest
from pathlib import Path

from evoagent.artifacts import ArtifactCollectionContract, DeclaredArtifact
from evoagent.models import ExperimentPlan, ExperimentRun, Hypothesis, RunArtifact


# cf-atom: TEST-artifact-collection-contract-detects-missing-required-artifacts
class ArtifactCollectionContractTests(unittest.TestCase):
    def test_artifact_collection_contract_detects_missing_required_artifacts(self):
        run = make_run()
        declarations = (
            DeclaredArtifact("metrics.json", "metrics"),
            DeclaredArtifact("config.json", "configuration"),
            DeclaredArtifact("plot.png", "plot", required=False),
        )

        report = ArtifactCollectionContract().evaluate(
            run,
            declarations,
            environment_metadata={"python": "3.11"},
        )

        self.assertFalse(report.complete)
        self.assertEqual(report.collected_paths, ("metrics.json",))
        self.assertEqual(report.missing_required_paths, ("config.json",))
        self.assertEqual(report.stdout_ref, "stdout")
        self.assertEqual(report.stderr_ref, "stderr")
        self.assertEqual(report.environment_metadata, (("python", "3.11"),))


def make_run() -> ExperimentRun:
    plan = ExperimentPlan(
        Hypothesis("artifacts", "collect declared outputs", expected_gain=0.01, novelty=0.2),
        "toy-tabular",
        "baseline",
        "accuracy",
        0.1,
    )
    return ExperimentRun(
        run_id="run_artifacts",
        plan=plan,
        command="python experiment.py",
        cwd=Path.cwd(),
        returncode=0,
        stdout="ok",
        stderr="",
        duration_seconds=0.1,
        artifacts=(RunArtifact("metrics.json", True, 20), RunArtifact("config.json", False, 0)),
    )


if __name__ == "__main__":
    unittest.main()
