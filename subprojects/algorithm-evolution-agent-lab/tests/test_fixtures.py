import sys
import tempfile
import unittest
from pathlib import Path

from evoagent.fixtures import ToyTabularFixture
from evoagent.ingestion import ResultIngestor
from evoagent.models import Hypothesis
from evoagent.runner import LocalExperimentRunner


# cf-atom: TEST-local-benchmark-fixture-produces-artifacts
class LocalBenchmarkFixtureTests(unittest.TestCase):
    def test_local_benchmark_fixture_produces_artifacts(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = ToyTabularFixture().write_run_artifacts(
                Path(tmp),
                candidate="linear-threshold",
                seed=3,
            )

            self.assertGreater(result.metrics["accuracy"], result.metrics["baseline_accuracy"])
            for artifact in ("metrics.json", "config.json", "environment.json", "run.log", "analysis.md"):
                self.assertTrue((Path(tmp) / artifact).exists(), artifact)

    # cf-atom: TEST-local-benchmark-fixture-runs-through-runner
    def test_local_benchmark_fixture_runs_through_runner_and_ingestor(self):
        hypothesis = Hypothesis("threshold baseline probe", "cheap sanity benchmark", 0.02, 0.2)
        plan = ToyTabularFixture().plan_for(hypothesis, candidate="linear-threshold", seed=4)
        src_root = Path(__file__).resolve().parents[1] / "src"
        with tempfile.TemporaryDirectory() as tmp:
            run = LocalExperimentRunner().run(
                plan,
                cwd=Path(tmp),
                command=(
                    f"PYTHONPATH={src_root} {sys.executable} -m evoagent.fixture_runner "
                    "--fixture toy-tabular --candidate linear-threshold --seed 4"
                ),
            )
            result = ResultIngestor().ingest_metrics(run)

        self.assertTrue(run.succeeded)
        self.assertGreater(result.metric_value, result.baseline_value)


if __name__ == "__main__":
    unittest.main()
