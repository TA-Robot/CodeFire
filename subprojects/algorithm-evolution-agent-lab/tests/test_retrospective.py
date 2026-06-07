import unittest

from evoagent.retrospective import (
    ResearchCycleRetrospective,
    ResearchCycleSignal,
    RetrospectivePlanningSummary,
    RetrospectivePriority,
    RetrospectiveRecommendation,
)


class ResearchCycleRetrospectiveTests(unittest.TestCase):
    # cf-atom: TEST-research-cycle-retrospective-recommends-policy-adjustments
    def test_research_cycle_retrospective_recommends_policy_adjustments(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-1",
                    completed_runs=4,
                    improved_candidates=2,
                    regressed_candidates=0,
                    failed_runs=1,
                    blocked_items=1,
                    mean_cost=3.0,
                    remaining_budget=1.5,
                    high_frontier_drift=1,
                    evidence_ready_claims=1,
                ),
                ResearchCycleSignal(
                    cycle_id="cycle-2",
                    completed_runs=3,
                    improved_candidates=1,
                    regressed_candidates=1,
                    failed_runs=2,
                    blocked_items=2,
                    mean_cost=2.0,
                    remaining_budget=1.0,
                    high_frontier_drift=1,
                    evidence_ready_claims=0,
                ),
            ]
        )

        self.assertEqual(report.cycle_ids, ("cycle-1", "cycle-2"))
        self.assertEqual(report.completed_runs, 7)
        self.assertAlmostEqual(report.improvement_rate, 3 / 7)
        self.assertIn(RetrospectiveRecommendation.MITIGATE_RISK, report.recommendations)
        self.assertIn(RetrospectiveRecommendation.REDUCE_COST, report.recommendations)
        self.assertIn(RetrospectiveRecommendation.CONSOLIDATE, report.recommendations)
        self.assertEqual(report.priority, RetrospectivePriority.HIGH)
        self.assertTrue(any("risk pressure" in reason for reason in report.rationale))

    def test_research_cycle_retrospective_validates_counts(self) -> None:
        with self.assertRaisesRegex(ValueError, "cycle_id is required"):
            ResearchCycleRetrospective().summarize(
                [
                    ResearchCycleSignal(
                        cycle_id="",
                        completed_runs=1,
                        improved_candidates=0,
                        regressed_candidates=0,
                        failed_runs=0,
                        blocked_items=0,
                        mean_cost=1.0,
                        remaining_budget=1.0,
                    )
                ]
            )

    # cf-atom: TEST-retrospective-planning-summary-renders-markdown
    def test_retrospective_planning_summary_renders_markdown(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-3",
                    completed_runs=5,
                    improved_candidates=0,
                    regressed_candidates=1,
                    failed_runs=1,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=6.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=0,
                )
            ]
        )

        markdown = RetrospectivePlanningSummary().render_markdown(report, title="Next Cycle Policy")

        self.assertIn("# Next Cycle Policy", markdown)
        self.assertIn("- Cycles: cycle-3", markdown)
        self.assertIn("- Priority: low", markdown)
        self.assertIn("## Recommendations", markdown)
        self.assertIn("- increase_exploration", markdown)
        self.assertIn("## Rationale", markdown)


if __name__ == "__main__":
    unittest.main()
