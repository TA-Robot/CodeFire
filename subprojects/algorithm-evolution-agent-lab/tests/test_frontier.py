import unittest

from evoagent.frontier import (
    FrontierFeedback,
    FrontierFeedbackIntegrator,
    FrontierFeedbackOutcome,
    FrontierAction,
    FrontierExperimentPlanner,
    ResearchFrontierItem,
    ResearchFrontierMap,
    ResearchFrontierSignal,
)


# cf-atom: TEST-research-frontier-map-ranks-opportunities
class ResearchFrontierMapTests(unittest.TestCase):
    def test_research_frontier_map_ranks_opportunities(self):
        frontier = ResearchFrontierMap()

        ranked = frontier.rank(
            (
                ResearchFrontierSignal(
                    frontier_id="safe-low-value",
                    mechanism_family="regularized_baseline_tuning",
                    target="toy-tabular",
                    baseline_gap=0.05,
                    expected_information_gain=0.2,
                    novelty_score=0.1,
                    estimated_cost=0.2,
                ),
                ResearchFrontierSignal(
                    frontier_id="novel-gap",
                    mechanism_family="adaptive_feature_selector",
                    target="toy-tabular",
                    baseline_gap=0.4,
                    expected_information_gain=0.8,
                    novelty_score=0.9,
                    estimated_cost=0.5,
                    rationale="large gap and novel mechanism",
                ),
                ResearchFrontierSignal(
                    frontier_id="blocked",
                    mechanism_family="expensive_meta_learner",
                    target="toy-tabular",
                    baseline_gap=0.6,
                    expected_information_gain=0.9,
                    novelty_score=0.8,
                    estimated_cost=0.6,
                    blocker_risks=1,
                ),
            ),
            remaining_budget=1.0,
        )

        by_id = {item.frontier_id: item for item in ranked}
        self.assertEqual(ranked[0].frontier_id, "novel-gap")
        self.assertEqual(by_id["novel-gap"].action, FrontierAction.RUN_PROBE)
        self.assertEqual(by_id["novel-gap"].expected_evidence_gain, "high")
        self.assertEqual(by_id["blocked"].action, FrontierAction.MITIGATE_RISK)
        self.assertIn("blocker_risks", by_id["blocked"].reasons)

    def test_research_frontier_map_defers_or_redesigns_unsafe_work(self):
        frontier = ResearchFrontierMap()

        ranked = frontier.rank(
            (
                ResearchFrontierSignal(
                    frontier_id="over-budget",
                    mechanism_family="large_ensemble_search",
                    target="toy-tabular",
                    baseline_gap=0.3,
                    expected_information_gain=0.7,
                    novelty_score=0.6,
                    estimated_cost=3.0,
                ),
                ResearchFrontierSignal(
                    frontier_id="known-bad",
                    mechanism_family="leaky_target_encoding",
                    target="toy-tabular",
                    baseline_gap=0.5,
                    expected_information_gain=0.8,
                    novelty_score=0.7,
                    estimated_cost=0.3,
                    negative_result_overlap=0.9,
                ),
            ),
            remaining_budget=1.0,
        )

        by_id = {item.frontier_id: item for item in ranked}
        self.assertEqual(by_id["over-budget"].action, FrontierAction.DEFER)
        self.assertIn("over_budget", by_id["over-budget"].reasons)
        self.assertEqual(by_id["known-bad"].action, FrontierAction.REDESIGN)
        self.assertEqual(by_id["known-bad"].primary_risk, "repeated_negative_result")


# cf-atom: TEST-frontier-experiment-planner-builds-runnable-plans
class FrontierExperimentPlannerTests(unittest.TestCase):
    def test_frontier_experiment_planner_builds_runnable_plans(self):
        planner = FrontierExperimentPlanner()
        frontiers = (
            ResearchFrontierItem(
                frontier_id="frontier-a",
                mechanism_family="adaptive selector",
                target="toy-tabular",
                action=FrontierAction.RUN_PROBE,
                score=8.0,
                expected_evidence_gain="high",
                primary_risk="execution_uncertainty",
                reasons=("baseline_gap", "novel_mechanism"),
                rationale="probe a compact adaptive selector",
            ),
            ResearchFrontierItem(
                frontier_id="blocked",
                mechanism_family="unsafe shortcut",
                target="toy-tabular",
                action=FrontierAction.REDESIGN,
                score=6.0,
                expected_evidence_gain="medium",
                primary_risk="repeated_negative_result",
                reasons=("negative_result_overlap",),
                rationale="known bad path",
            ),
        )

        drafts = planner.build_plans(
            frontiers,
            baseline="baseline-v1",
            metric="validation_score",
            remaining_budget=1.0,
            max_plans=2,
        )

        self.assertEqual(len(drafts), 1)
        self.assertEqual(drafts[0].frontier_id, "frontier-a")
        self.assertEqual(drafts[0].plan.benchmark, "toy-tabular")
        self.assertEqual(drafts[0].plan.baseline, "baseline-v1")
        self.assertEqual(drafts[0].plan.metric, "validation_score")
        self.assertIn("--mechanism adaptive_selector", drafts[0].plan.command)
        self.assertEqual(drafts[0].plan.artifact_paths, ("artifacts/frontier-a/metrics.json",))
        self.assertIn("inspect primary risk: execution_uncertainty", drafts[0].analysis_criteria)

    def test_frontier_experiment_planner_respects_budget_and_limit(self):
        planner = FrontierExperimentPlanner()
        frontiers = tuple(
            ResearchFrontierItem(
                frontier_id=f"frontier-{index}",
                mechanism_family="cheap probe",
                target="toy-tabular",
                action=FrontierAction.RUN_PROBE,
                score=5.0,
                expected_evidence_gain="medium",
                primary_risk="execution_uncertainty",
                reasons=("baseline_gap",),
                rationale="cheap",
            )
            for index in range(3)
        )

        drafts = planner.build_plans(
            frontiers,
            baseline="baseline-v1",
            metric="validation_score",
            remaining_budget=0.6,
            max_plans=1,
        )

        self.assertEqual(len(drafts), 1)
        self.assertLessEqual(drafts[0].plan.estimated_cost, 0.6)


# cf-atom: TEST-frontier-feedback-integrator-updates-frontier-signals
class FrontierFeedbackIntegratorTests(unittest.TestCase):
    def test_frontier_feedback_integrator_updates_frontier_signals(self):
        integrator = FrontierFeedbackIntegrator()
        base = ResearchFrontierSignal(
            frontier_id="adaptive-selector",
            mechanism_family="adaptive selector",
            target="toy-tabular",
            baseline_gap=0.2,
            expected_information_gain=0.4,
            novelty_score=0.7,
            estimated_cost=0.5,
            challenge_alignment=0.6,
            negative_result_overlap=0.2,
        )

        updates = integrator.integrate(
            (base,),
            (
                FrontierFeedback(
                    frontier_id="adaptive-selector",
                    outcome=FrontierFeedbackOutcome.IMPROVED,
                    metric_delta=0.12,
                    confidence=0.8,
                    observed_cost=0.4,
                    notes="stable local gain",
                ),
                FrontierFeedback(
                    frontier_id="adaptive-selector",
                    outcome=FrontierFeedbackOutcome.REPLICATED,
                    metric_delta=0.08,
                    confidence=0.7,
                    observed_cost=0.5,
                ),
            ),
        )

        self.assertEqual(len(updates), 1)
        update = updates[0]
        self.assertEqual(update.applied_outcome_count, 2)
        self.assertIn("improved", update.reasons)
        self.assertIn("replicated", update.reasons)
        self.assertIsNotNone(update.updated_signal)
        assert update.updated_signal is not None
        self.assertGreater(update.updated_signal.baseline_gap, base.baseline_gap)
        self.assertGreater(
            update.updated_signal.expected_information_gain,
            base.expected_information_gain,
        )
        self.assertLess(
            update.updated_signal.negative_result_overlap,
            base.negative_result_overlap,
        )
        self.assertIn("stable local gain", update.updated_signal.rationale)

    def test_frontier_feedback_integrator_penalizes_failed_and_blocked_feedback(self):
        integrator = FrontierFeedbackIntegrator()
        base = ResearchFrontierSignal(
            frontier_id="expensive-meta",
            mechanism_family="meta learner",
            target="toy-tabular",
            baseline_gap=0.5,
            expected_information_gain=0.8,
            novelty_score=0.8,
            estimated_cost=0.5,
        )

        update = integrator.integrate(
            (base,),
            (
                FrontierFeedback(
                    frontier_id="expensive-meta",
                    outcome=FrontierFeedbackOutcome.FAILED,
                    confidence=0.9,
                    observed_cost=0.9,
                ),
                FrontierFeedback(
                    frontier_id="expensive-meta",
                    outcome=FrontierFeedbackOutcome.BLOCKED,
                    blocker_count=2,
                ),
            ),
        )[0]

        self.assertIn("failed", update.reasons)
        self.assertIn("blocked", update.reasons)
        self.assertIn("over_budget", update.reasons)
        self.assertIsNotNone(update.updated_signal)
        assert update.updated_signal is not None
        self.assertGreater(
            update.updated_signal.negative_result_overlap,
            base.negative_result_overlap,
        )
        self.assertLess(
            update.updated_signal.expected_information_gain,
            base.expected_information_gain,
        )
        self.assertEqual(update.updated_signal.blocker_risks, 2)

        ranked = ResearchFrontierMap().rank((update.updated_signal,), remaining_budget=1.0)
        self.assertEqual(ranked[0].action, FrontierAction.MITIGATE_RISK)

    def test_frontier_feedback_integrator_reports_unknown_frontier(self):
        updates = FrontierFeedbackIntegrator().integrate(
            (),
            (
                FrontierFeedback(
                    frontier_id="missing-frontier",
                    outcome=FrontierFeedbackOutcome.INCONCLUSIVE,
                    notes="no matching frontier",
                ),
            ),
        )

        self.assertEqual(len(updates), 1)
        self.assertEqual(updates[0].frontier_id, "missing-frontier")
        self.assertIsNone(updates[0].prior_signal)
        self.assertIsNone(updates[0].updated_signal)
        self.assertEqual(updates[0].applied_outcome_count, 0)
        self.assertEqual(updates[0].reasons, ("unknown_frontier", "inconclusive"))


if __name__ == "__main__":
    unittest.main()
