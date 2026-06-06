import unittest
from pathlib import Path

from evoagent.challenge import ComputeBudget
from evoagent.compute import ComputeAccountant
from evoagent.models import ExperimentPlan, ExperimentRun, Hypothesis


# cf-atom: TEST-compute-accountant-counts-failed-attempts
class ComputeAccountantTests(unittest.TestCase):
    def test_compute_accountant_counts_failed_attempts(self):
        runs = [run("run-1", returncode=0, duration_seconds=1800), run("run-2", returncode=1, duration_seconds=900)]

        account = ComputeAccountant().summarize(runs, ComputeBudget(max_hours=1.0, max_trials=3))

        self.assertEqual(account.total_runs, 2)
        self.assertEqual(account.successful_runs, 1)
        self.assertEqual(account.failed_runs, 1)
        self.assertAlmostEqual(account.total_hours, 0.75)
        self.assertAlmostEqual(account.remaining_hours, 0.25)
        self.assertEqual(account.remaining_trials, 1)
        self.assertFalse(account.over_budget)

    def test_compute_accountant_flags_budget_overrun(self):
        runs = [run("run-1", returncode=0, duration_seconds=4000), run("run-2", returncode=0, duration_seconds=10)]

        account = ComputeAccountant().summarize(runs, ComputeBudget(max_hours=1.0, max_trials=1))

        self.assertTrue(account.over_budget)


def run(run_id: str, *, returncode: int, duration_seconds: float) -> ExperimentRun:
    hypothesis = Hypothesis("candidate", "rationale", expected_gain=0.01, novelty=0.2)
    plan = ExperimentPlan(hypothesis, "bench", "baseline", "accuracy", estimated_cost=0.1)
    return ExperimentRun(
        run_id=run_id,
        plan=plan,
        command="run",
        cwd=Path("."),
        returncode=returncode,
        stdout="",
        stderr="",
        duration_seconds=duration_seconds,
    )


if __name__ == "__main__":
    unittest.main()
