import sys
import tempfile
import unittest
from pathlib import Path

from evoagent.ingestion import ResultIngestor
from evoagent.models import ExperimentPlan, Hypothesis
from evoagent.runner import LocalExperimentRunner


# cf-atom: TEST-result-ingestor-parses-metrics-artifact
class ResultIngestorTests(unittest.TestCase):
    def test_result_ingestor_parses_metrics_artifact(self):
        hypothesis = Hypothesis("metric parser", "turn metrics into result", 0.02, 0.3)
        plan = ExperimentPlan(
            hypothesis,
            "toy-tabular",
            "logistic",
            "accuracy",
            0.1,
            artifact_paths=("metrics.json",),
        )

        with tempfile.TemporaryDirectory() as tmp:
            run = LocalExperimentRunner().run(
                plan,
                cwd=Path(tmp),
                command=(
                    f"{sys.executable} -c "
                    "\"from pathlib import Path; "
                    "Path('metrics.json').write_text('{\\\"accuracy\\\": 0.83, "
                    "\\\"baseline_accuracy\\\": 0.78, \\\"confidence\\\": 0.7, "
                    "\\\"notes\\\": \\\"clean probe\\\"}')\""
                ),
            )

            result = ResultIngestor().ingest_metrics(run)

        self.assertEqual(result.metric_value, 0.83)
        self.assertEqual(result.baseline_value, 0.78)
        self.assertEqual(result.confidence, 0.7)
        self.assertEqual(result.notes, "clean probe")
        self.assertAlmostEqual(result.quality_delta, 0.05)

    def test_result_ingestor_rejects_missing_metrics_artifact(self):
        hypothesis = Hypothesis("metric parser", "turn metrics into result", 0.02, 0.3)
        plan = ExperimentPlan(hypothesis, "toy-tabular", "logistic", "accuracy", 0.1)

        with tempfile.TemporaryDirectory() as tmp:
            run = LocalExperimentRunner().run(
                plan,
                cwd=Path(tmp),
                command=f"{sys.executable} -c \"print('no metrics')\"",
            )

        with self.assertRaises(ValueError):
            ResultIngestor().ingest_metrics(run)


if __name__ == "__main__":
    unittest.main()
