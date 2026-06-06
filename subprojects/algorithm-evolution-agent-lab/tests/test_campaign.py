import unittest
from pathlib import Path

from evoagent.campaign import CampaignMilestone, ExperimentCampaignTracker
from evoagent.models import ExperimentGoal, ExperimentPlan, ExperimentResult, ExperimentRun, Hypothesis
from evoagent.program import ResearchProgram


# cf-atom: TEST-experiment-campaign-tracker-identifies-next-milestone
class ExperimentCampaignTrackerTests(unittest.TestCase):
    def test_experiment_campaign_tracker_identifies_next_milestone(self):
        program = ResearchProgram(ExperimentGoal("Improve sample efficiency"))
        hypothesis = Hypothesis("campaign", "track milestones", expected_gain=0.02, novelty=0.3)
        plan = ExperimentPlan(hypothesis, "toy-tabular", "baseline", "accuracy", 0.1)
        run = ExperimentRun("run_campaign", plan, "python exp.py", Path.cwd(), 0, "ok", "", 0.1)
        program.add_run_evidence(
            claim_id="claim",
            run=run,
            summary="baseline comparison complete",
            confidence=0.7,
        )
        result = ExperimentResult(plan, metric_value=0.83, baseline_value=0.80, confidence=0.7)
        milestones = (
            CampaignMilestone("m1", "first successful run", min_successful_runs=1),
            CampaignMilestone("m2", "confirmed improvement", min_improved_results=1),
            CampaignMilestone("m3", "review ready", required_evidence_tags=("review",)),
        )

        progress = ExperimentCampaignTracker().evaluate(program, milestones, results=(result,))

        self.assertEqual(progress.completed_milestones, ("m1", "m2"))
        self.assertEqual(progress.blocked_milestones, ("m3",))
        self.assertEqual(progress.next_milestone, "m3")
        self.assertAlmostEqual(progress.completion_ratio, 2 / 3)


if __name__ == "__main__":
    unittest.main()
