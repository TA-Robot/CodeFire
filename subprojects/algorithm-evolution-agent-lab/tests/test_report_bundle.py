import sys
import tempfile
import unittest
from pathlib import Path

from evoagent.challenge import load_challenge_definition
from evoagent.models import CandidateAlgorithm, ExperimentGoal, ExperimentPlan, Hypothesis
from evoagent.program import ResearchProgram
from evoagent.report_bundle import SotaReportBundleGenerator


# cf-atom: TEST-sota-report-bundle-writes-reproduction-files
class SotaReportBundleTests(unittest.TestCase):
    def test_sota_report_bundle_writes_reproduction_files(self):
        program = ResearchProgram(ExperimentGoal("Improve toy tabular sample efficiency"))
        hypothesis = Hypothesis("calibrated thresholding", "reduce variance near the decision boundary", 0.03, 0.5)
        plan = ExperimentPlan(
            hypothesis,
            "toy-tabular",
            "regularized-polynomial",
            "validation_score",
            0.2,
            command=f"{sys.executable} -m evoagent.fixture_runner --fixture toy-tabular --candidate calibrated",
        )
        program.add_candidate(CandidateAlgorithm("candidate-calibrated", hypothesis, plan))
        challenge = load_challenge_definition("challenges/toy_tabular_sample_efficiency.json")

        with tempfile.TemporaryDirectory() as tmpdir:
            manifest = SotaReportBundleGenerator().write(
                program=program,
                challenge=challenge,
                output_dir=Path(tmpdir) / "bundle",
            )

            written = {path.name for path in manifest.files}
            self.assertEqual(written, {"report.md", "challenge.md", "reproduction.md", "limitations.md", "manifest.json"})
            self.assertIn("calibrated thresholding", (manifest.bundle_dir / "report.md").read_text(encoding="utf-8"))
            self.assertIn("regularized-polynomial", (manifest.bundle_dir / "challenge.md").read_text(encoding="utf-8"))
            self.assertIn("evoagent.fixture_runner", (manifest.bundle_dir / "reproduction.md").read_text(encoding="utf-8"))
            self.assertIn("validation labels", (manifest.bundle_dir / "limitations.md").read_text(encoding="utf-8"))
            self.assertIn("toy-tabular-sample-efficiency", (manifest.bundle_dir / "manifest.json").read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
