import unittest
from pathlib import Path

from evoagent.models import ExperimentPlan, ExperimentRun, Hypothesis, RunArtifact
from evoagent.run_identity import RunIdentityLedger


# cf-atom: TEST-run-identity-ledger-links-run-logs-artifacts-evidence
class RunIdentityLedgerTests(unittest.TestCase):
    def test_run_identity_ledger_links_run_logs_artifacts_evidence(self):
        run = make_run()
        ledger = RunIdentityLedger()

        record = ledger.record(run, config_ref="cfg_123", evidence_refs=("ev_1",), report_refs=("report.md",))

        self.assertEqual(record.run_id, run.run_id)
        self.assertEqual(record.plan_id, run.plan.uid)
        self.assertEqual(record.log_refs, ("stdout", "stderr"))
        self.assertEqual(record.artifact_refs, ("metrics.json",))
        self.assertEqual(ledger.get(run.run_id), record)
        self.assertEqual(ledger.for_plan(run.plan.uid), (record,))

    def test_run_identity_ledger_rejects_duplicate_runs(self):
        run = make_run()
        ledger = RunIdentityLedger()
        ledger.record(run, config_ref="cfg_123")

        with self.assertRaises(ValueError):
            ledger.record(run, config_ref="cfg_456")


def make_run() -> ExperimentRun:
    plan = ExperimentPlan(
        Hypothesis("identity", "link run assets", expected_gain=0.01, novelty=0.2),
        "toy-tabular",
        "baseline",
        "accuracy",
        0.1,
    )
    return ExperimentRun(
        run_id="run_abc",
        plan=plan,
        command="python experiment.py",
        cwd=Path.cwd(),
        returncode=0,
        stdout="ok",
        stderr="",
        duration_seconds=0.1,
        artifacts=(RunArtifact("metrics.json", True, 10),),
    )


if __name__ == "__main__":
    unittest.main()
