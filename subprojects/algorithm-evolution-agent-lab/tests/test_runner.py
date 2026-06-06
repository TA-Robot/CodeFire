import sys
import tempfile
import unittest
from pathlib import Path

from evoagent.models import ExperimentPlan, Hypothesis
from evoagent.runner import LocalExperimentRunner


# cf-atom: TEST-local-runner-captures-artifacts
class LocalExperimentRunnerTests(unittest.TestCase):
    def test_local_runner_captures_artifacts(self):
        hypothesis = Hypothesis("probe", "check runner", expected_gain=0.01, novelty=0.10)
        plan = ExperimentPlan(
            hypothesis=hypothesis,
            benchmark="toy",
            baseline="zero",
            metric="accuracy",
            estimated_cost=0.01,
            artifact_paths=("metrics.json",),
        )
        with tempfile.TemporaryDirectory() as tmp:
            cwd = Path(tmp)
            command = (
                f"{sys.executable} -c "
                "\"from pathlib import Path; "
                "Path('metrics.json').write_text('{\\\"accuracy\\\": 0.75}'); "
                "print('done')\""
            )

            run = LocalExperimentRunner().run(plan, cwd=cwd, command=command)

        self.assertTrue(run.succeeded)
        self.assertIn("done", run.stdout)
        self.assertEqual(run.artifacts[0].path, "metrics.json")
        self.assertTrue(run.artifacts[0].exists)
        self.assertGreater(run.artifacts[0].size_bytes, 0)

    def test_local_runner_requires_command(self):
        hypothesis = Hypothesis("probe", "check runner", expected_gain=0.01, novelty=0.10)
        plan = ExperimentPlan(hypothesis, "toy", "zero", "accuracy", 0.01)

        with self.assertRaises(ValueError):
            LocalExperimentRunner().run(plan, cwd=Path.cwd())


if __name__ == "__main__":
    unittest.main()
