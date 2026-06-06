import sys
import tempfile
import unittest
from pathlib import Path

from evoagent.analysis import RunAnalyst
from evoagent.ingestion import ResultIngestor
from evoagent.models import ExperimentPlan, Hypothesis
from evoagent.runner import LocalExperimentRunner


# cf-atom: TEST-run-analyst-writes-structured-note
class RunAnalystTests(unittest.TestCase):
    def test_run_analyst_writes_structured_note_for_ingested_result(self):
        hypothesis = Hypothesis("analysis", "explain metric movement", 0.02, 0.3)
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
                    "Path('metrics.json').write_text('{\\\"accuracy\\\": 0.84, "
                    "\\\"baseline_accuracy\\\": 0.80, \\\"confidence\\\": 0.65, "
                    "\\\"notes\\\": \\\"feature filter reduced variance\\\"}')\""
                ),
            )
            result = ResultIngestor().ingest_metrics(run)

        note = RunAnalyst().analyze(run, result)

        self.assertIn("improved accuracy by 0.0400", note.summary)
        self.assertEqual(note.metric_comparison, "0.8400 vs baseline 0.8000")
        self.assertEqual(note.failure_classification, "none")
        self.assertEqual(note.next_action, "replicate with another seed")
        self.assertEqual(note.claim_readiness_update, "local improvement candidate")

    def test_run_analyst_classifies_failed_run(self):
        hypothesis = Hypothesis("analysis", "explain failure", 0.02, 0.3)
        plan = ExperimentPlan(hypothesis, "toy-tabular", "logistic", "accuracy", 0.1)

        with tempfile.TemporaryDirectory() as tmp:
            run = LocalExperimentRunner().run(
                plan,
                cwd=Path(tmp),
                command=f"{sys.executable} -c \"import sys; print('bad config'); sys.exit(2)\"",
            )

        note = RunAnalyst().analyze(run)

        self.assertEqual(note.failure_classification, "implementation_error")
        self.assertEqual(note.confidence, 0.0)
        self.assertEqual(note.claim_readiness_update, "no claim progress")

    def test_run_analyst_uses_failure_classifier_for_timeout(self):
        hypothesis = Hypothesis("analysis", "explain timeout", 0.02, 0.3)
        plan = ExperimentPlan(hypothesis, "toy-tabular", "logistic", "accuracy", 0.1)

        with tempfile.TemporaryDirectory() as tmp:
            run = LocalExperimentRunner().run(
                plan,
                cwd=Path(tmp),
                command=f"{sys.executable} -c \"import time; time.sleep(0.2)\"",
                timeout_seconds=0.01,
            )

        note = RunAnalyst().analyze(run)

        self.assertEqual(note.failure_classification, "insufficient_budget")
        self.assertIn("budget", note.next_action)


if __name__ == "__main__":
    unittest.main()
