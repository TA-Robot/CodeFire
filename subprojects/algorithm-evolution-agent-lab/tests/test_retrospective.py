import unittest

from evoagent.retrospective import (
    ResearchCycleRetrospective,
    ResearchCyclePlanningHandoffBundleBuilder,
    ResearchCyclePlanLane,
    ResearchCyclePlan,
    ResearchCyclePlanItem,
    ResearchCyclePlanningPacketBuilder,
    ResearchCyclePlanningPacketManifestBuilder,
    ResearchCyclePlanningPacketManifestMarkdown,
    ResearchCyclePlanningPacketManifestVerificationMarkdown,
    ResearchCyclePlanningPacketManifestVerifier,
    ResearchCyclePlanningPacketMarkdown,
    ResearchCyclePlanLint,
    ResearchCyclePlanLintMarkdown,
    ResearchCyclePlanLintSeverity,
    ResearchCyclePlanMarkdown,
    ResearchCyclePlanSynthesizer,
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

    # cf-atom: TEST-research-cycle-plan-synthesizer-builds-next-cycle-lanes
    def test_research_cycle_plan_synthesizer_builds_next_cycle_lanes(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-4",
                    completed_runs=8,
                    improved_candidates=3,
                    regressed_candidates=1,
                    failed_runs=2,
                    blocked_items=3,
                    mean_cost=4.0,
                    remaining_budget=1.0,
                    high_frontier_drift=2,
                    evidence_ready_claims=1,
                )
            ]
        )

        plan = ResearchCyclePlanSynthesizer().synthesize(
            report,
            active_capacity=1,
            remaining_budget=3.0,
        )

        self.assertEqual(plan.source_cycles, ("cycle-4",))
        self.assertEqual(plan.priority, RetrospectivePriority.HIGH)
        self.assertEqual(len(plan.lane(ResearchCyclePlanLane.MITIGATION)), 1)
        self.assertEqual(len(plan.lane(ResearchCyclePlanLane.ACTIVE)), 1)
        self.assertEqual(len(plan.lane(ResearchCyclePlanLane.REVIEW)), 1)
        self.assertIn(
            RetrospectiveRecommendation.MITIGATE_RISK,
            [item.recommendation for item in plan.items],
        )
        self.assertLessEqual(plan.lane(ResearchCyclePlanLane.ACTIVE)[0].budget_hint, 3.0)

    # cf-atom: TEST-research-cycle-plan-markdown-renders-lanes
    def test_research_cycle_plan_markdown_renders_lanes(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-5",
                    completed_runs=4,
                    improved_candidates=0,
                    regressed_candidates=0,
                    failed_runs=1,
                    blocked_items=0,
                    mean_cost=2.0,
                    remaining_budget=10.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=0,
                )
            ]
        )
        plan = ResearchCyclePlanSynthesizer().synthesize(
            report,
            active_capacity=2,
            remaining_budget=5.0,
        )

        markdown = ResearchCyclePlanMarkdown().render(plan, title="Cycle 5 Plan")

        self.assertIn("# Cycle 5 Plan", markdown)
        self.assertIn("- Source cycles: cycle-5", markdown)
        self.assertIn("## Active", markdown)
        self.assertIn("### Increase exploration diversity", markdown)
        self.assertIn("- Recommendation: increase_exploration", markdown)
        self.assertIn("- Budget hint: 2.00", markdown)

    # cf-atom: TEST-research-cycle-plan-lint-flags-invalid-plan
    def test_research_cycle_plan_lint_flags_invalid_plan(self) -> None:
        plan = ResearchCyclePlan(
            source_cycles=("",),
            priority=RetrospectivePriority.MEDIUM,
            active_capacity=1,
            remaining_budget=1.0,
            items=(
                ResearchCyclePlanItem(
                    lane=ResearchCyclePlanLane.ACTIVE,
                    recommendation=RetrospectiveRecommendation.REDUCE_COST,
                    title="",
                    action="draft cheaper probe",
                    rationale="",
                    budget_hint=1.0,
                ),
                ResearchCyclePlanItem(
                    lane=ResearchCyclePlanLane.ACTIVE,
                    recommendation=RetrospectiveRecommendation.INCREASE_EXPLORATION,
                    title="Explore alternate mechanism",
                    action="sample a different frontier",
                    rationale="low improvement",
                    budget_hint=1.0,
                ),
            ),
        )

        report = ResearchCyclePlanLint().lint(plan)

        self.assertFalse(report.ok)
        self.assertEqual(report.blocker_count, 4)
        self.assertEqual(report.warning_count, 1)
        self.assertEqual(report.findings[0].severity, ResearchCyclePlanLintSeverity.BLOCKER)
        self.assertIn("active lane exceeds active capacity", [finding.message for finding in report.findings])
        self.assertIn("active budget hints exceed remaining budget", [finding.message for finding in report.findings])
        self.assertIn("source cycle IDs must be non-empty", [finding.message for finding in report.findings])
        self.assertIn("title is required", [finding.message for finding in report.findings])

    def test_research_cycle_plan_lint_accepts_synthesized_plan(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-6",
                    completed_runs=5,
                    improved_candidates=2,
                    regressed_candidates=0,
                    failed_runs=1,
                    blocked_items=0,
                    mean_cost=2.0,
                    remaining_budget=6.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=1,
                )
            ]
        )
        plan = ResearchCyclePlanSynthesizer().synthesize(report, active_capacity=2, remaining_budget=5.0)

        lint = ResearchCyclePlanLint().lint(plan)

        self.assertTrue(lint.ok)
        self.assertEqual(lint.findings, ())

    # cf-atom: TEST-research-cycle-plan-lint-markdown-renders-findings
    def test_research_cycle_plan_lint_markdown_renders_findings(self) -> None:
        plan = ResearchCyclePlan(
            source_cycles=("cycle-7",),
            priority=RetrospectivePriority.LOW,
            active_capacity=1,
            remaining_budget=0.5,
            items=(
                ResearchCyclePlanItem(
                    lane=ResearchCyclePlanLane.ACTIVE,
                    recommendation=RetrospectiveRecommendation.INCREASE_EXPLORATION,
                    title="Increase exploration diversity",
                    action="sample an alternate frontier",
                    rationale="",
                    budget_hint=1.0,
                ),
            ),
        )
        lint = ResearchCyclePlanLint().lint(plan)

        markdown = ResearchCyclePlanLintMarkdown().render(lint, title="Cycle 7 Plan Lint")

        self.assertIn("# Cycle 7 Plan Lint", markdown)
        self.assertIn("- Status: blocked", markdown)
        self.assertIn("- Blockers: 1", markdown)
        self.assertIn("- Warnings: 1", markdown)
        self.assertIn("## Blockers", markdown)
        self.assertIn("`items.budget_hint`: active budget hints exceed remaining budget", markdown)
        self.assertIn("## Warnings", markdown)
        self.assertIn("`items[0].rationale`: rationale is empty", markdown)

    # cf-atom: TEST-research-cycle-planning-packet-builds-plan-lint-and-markdown
    def test_research_cycle_planning_packet_builds_plan_lint_and_markdown(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-8",
                    completed_runs=6,
                    improved_candidates=0,
                    regressed_candidates=0,
                    failed_runs=1,
                    blocked_items=0,
                    mean_cost=1.5,
                    remaining_budget=8.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=0,
                )
            ]
        )

        packet = ResearchCyclePlanningPacketBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=4.0,
            plan_title="Cycle 8 Plan",
            lint_title="Cycle 8 Lint",
        )

        self.assertEqual(packet.plan.source_cycles, ("cycle-8",))
        self.assertTrue(packet.lint_report.ok)
        self.assertIn("# Cycle 8 Plan", packet.plan_markdown)
        self.assertIn("### Increase exploration diversity", packet.plan_markdown)
        self.assertIn("# Cycle 8 Lint", packet.lint_markdown)
        self.assertIn("- Status: ok", packet.lint_markdown)
        self.assertIn("- none", packet.lint_markdown)

    # cf-atom: TEST-research-cycle-planning-packet-markdown-renders-review-document
    def test_research_cycle_planning_packet_markdown_renders_review_document(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-9",
                    completed_runs=4,
                    improved_candidates=0,
                    regressed_candidates=0,
                    failed_runs=1,
                    blocked_items=0,
                    mean_cost=1.25,
                    remaining_budget=6.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=0,
                )
            ]
        )
        packet = ResearchCyclePlanningPacketBuilder().build(
            report,
            active_capacity=1,
            remaining_budget=3.0,
            plan_title="Cycle 9 Plan",
            lint_title="Cycle 9 Lint",
        )

        markdown = ResearchCyclePlanningPacketMarkdown().render(packet, title="Cycle 9 Packet")

        self.assertIn("# Cycle 9 Packet", markdown)
        self.assertIn("- Source cycles: cycle-9", markdown)
        self.assertIn("- Status: ok", markdown)
        self.assertIn("- Active items: 1", markdown)
        self.assertIn("## Plan", markdown)
        self.assertIn("## Lint", markdown)
        self.assertIn("## Cycle 9 Plan", markdown)
        self.assertIn("## Cycle 9 Lint", markdown)
        self.assertIn("#### Increase exploration diversity", markdown)

    # cf-atom: TEST-research-cycle-planning-packet-manifest-records-artifact-hashes
    def test_research_cycle_planning_packet_manifest_records_artifact_hashes(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-10",
                    completed_runs=5,
                    improved_candidates=1,
                    regressed_candidates=0,
                    failed_runs=1,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=5.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=0,
                )
            ]
        )
        packet = ResearchCyclePlanningPacketBuilder().build(
            report,
            active_capacity=1,
            remaining_budget=2.0,
            plan_title="Cycle 10 Plan",
            lint_title="Cycle 10 Lint",
        )

        manifest = ResearchCyclePlanningPacketManifestBuilder().build(
            packet,
            packet_path="cycle-10/planning-packet.md",
            plan_path="cycle-10/plan.md",
            lint_path="cycle-10/lint.md",
            packet_title="Cycle 10 Packet",
        )

        self.assertEqual(manifest.source_cycles, ("cycle-10",))
        self.assertEqual(manifest.status, "ok")
        self.assertEqual(
            [entry.path for entry in manifest.entries],
            ["cycle-10/planning-packet.md", "cycle-10/plan.md", "cycle-10/lint.md"],
        )
        self.assertTrue(all(entry.content_sha256.startswith("sha256:") for entry in manifest.entries))
        self.assertTrue(all(entry.byte_count > 0 for entry in manifest.entries))
        with self.assertRaisesRegex(ValueError, "relative POSIX"):
            ResearchCyclePlanningPacketManifestBuilder().build(
                packet,
                packet_path="/tmp/planning-packet.md",
            )

    # cf-atom: TEST-research-cycle-planning-packet-manifest-markdown-renders-audit-table
    def test_research_cycle_planning_packet_manifest_markdown_renders_audit_table(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-11",
                    completed_runs=3,
                    improved_candidates=0,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=4.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=0,
                )
            ]
        )
        packet = ResearchCyclePlanningPacketBuilder().build(
            report,
            active_capacity=1,
            remaining_budget=2.0,
            plan_title="Cycle 11 Plan",
            lint_title="Cycle 11 Lint",
        )
        manifest = ResearchCyclePlanningPacketManifestBuilder().build(
            packet,
            packet_path="cycle-11/planning-packet.md",
            plan_path="cycle-11/plan.md",
            lint_path="cycle-11/lint.md",
            packet_title="Cycle 11 Packet",
        )

        markdown = ResearchCyclePlanningPacketManifestMarkdown().render(
            manifest,
            title="Cycle 11 Manifest",
        )

        self.assertIn("# Cycle 11 Manifest", markdown)
        self.assertIn("- Source cycles: cycle-11", markdown)
        self.assertIn("- Status: ok", markdown)
        self.assertIn("- Artifact count: 3", markdown)
        self.assertIn("| Path | SHA-256 | Bytes |", markdown)
        self.assertIn("| `cycle-11/planning-packet.md` | `sha256:", markdown)
        self.assertIn("| `cycle-11/lint.md` | `sha256:", markdown)

    # cf-atom: TEST-research-cycle-planning-packet-manifest-verifier-detects-drift
    def test_research_cycle_planning_packet_manifest_verifier_detects_drift(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-12",
                    completed_runs=3,
                    improved_candidates=1,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=4.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=1,
                )
            ]
        )
        packet = ResearchCyclePlanningPacketBuilder().build(
            report,
            active_capacity=1,
            remaining_budget=2.0,
            plan_title="Cycle 12 Plan",
            lint_title="Cycle 12 Lint",
        )
        packet_markdown = ResearchCyclePlanningPacketMarkdown().render(packet, title="Cycle 12 Packet")
        manifest = ResearchCyclePlanningPacketManifestBuilder().build(
            packet,
            packet_path="cycle-12/planning-packet.md",
            plan_path="cycle-12/plan.md",
            lint_path="cycle-12/lint.md",
            packet_title="Cycle 12 Packet",
        )

        verifier = ResearchCyclePlanningPacketManifestVerifier()
        clean = verifier.verify(
            manifest,
            {
                "cycle-12/planning-packet.md": packet_markdown,
                "cycle-12/plan.md": packet.plan_markdown,
                "cycle-12/lint.md": packet.lint_markdown,
            },
        )
        drifted = verifier.verify(
            manifest,
            {
                "cycle-12/planning-packet.md": packet_markdown + "\nchanged\n",
                "cycle-12/plan.md": packet.plan_markdown,
            },
        )

        self.assertTrue(clean.ok)
        self.assertFalse(drifted.ok)
        self.assertEqual(
            [(finding.path, finding.message) for finding in drifted.findings],
            [
                ("cycle-12/planning-packet.md", "sha256 digest mismatch"),
                ("cycle-12/planning-packet.md", "byte count mismatch"),
                ("cycle-12/lint.md", "artifact is missing"),
            ],
        )

    # cf-atom: TEST-research-cycle-planning-packet-manifest-verification-markdown-renders-findings
    def test_research_cycle_planning_packet_manifest_verification_markdown_renders_findings(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-13",
                    completed_runs=2,
                    improved_candidates=1,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=3.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=1,
                )
            ]
        )
        packet = ResearchCyclePlanningPacketBuilder().build(
            report,
            active_capacity=1,
            remaining_budget=2.0,
            plan_title="Cycle 13 Plan",
            lint_title="Cycle 13 Lint",
        )
        packet_markdown = ResearchCyclePlanningPacketMarkdown().render(packet, title="Cycle 13 Packet")
        manifest = ResearchCyclePlanningPacketManifestBuilder().build(
            packet,
            packet_path="cycle-13/planning-packet.md",
            plan_path="cycle-13/plan.md",
            lint_path="cycle-13/lint.md",
            packet_title="Cycle 13 Packet",
        )
        verifier = ResearchCyclePlanningPacketManifestVerifier()
        clean = verifier.verify(
            manifest,
            {
                "cycle-13/planning-packet.md": packet_markdown,
                "cycle-13/plan.md": packet.plan_markdown,
                "cycle-13/lint.md": packet.lint_markdown,
            },
        )
        drifted = verifier.verify(
            manifest,
            {
                "cycle-13/planning-packet.md": packet_markdown + "\nchanged\n",
                "cycle-13/plan.md": packet.plan_markdown,
            },
        )

        clean_markdown = ResearchCyclePlanningPacketManifestVerificationMarkdown().render(
            clean,
            title="Cycle 13 Manifest Verification",
        )
        drifted_markdown = ResearchCyclePlanningPacketManifestVerificationMarkdown().render(
            drifted,
            title="Cycle 13 Manifest Verification",
        )

        self.assertIn("# Cycle 13 Manifest Verification", clean_markdown)
        self.assertIn("- Status: ok", clean_markdown)
        self.assertIn("- Finding count: 0", clean_markdown)
        self.assertIn("- none", clean_markdown)
        self.assertIn("- Status: blocked", drifted_markdown)
        self.assertIn("- Finding count: 3", drifted_markdown)
        self.assertIn("`cycle-13/planning-packet.md`: sha256 digest mismatch", drifted_markdown)
        self.assertIn("`cycle-13/lint.md`: artifact is missing", drifted_markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-bundle-builds-artifacts-and-audits
    def test_research_cycle_planning_handoff_bundle_builds_artifacts_and_audits(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-14",
                    completed_runs=4,
                    improved_candidates=1,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=4.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=1,
                )
            ]
        )

        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=1,
            remaining_budget=2.0,
            packet_path="cycle-14/planning-packet.md",
            plan_path="cycle-14/plan.md",
            lint_path="cycle-14/lint.md",
            plan_title="Cycle 14 Plan",
            lint_title="Cycle 14 Lint",
            packet_title="Cycle 14 Packet",
            manifest_title="Cycle 14 Manifest",
            verification_title="Cycle 14 Verification",
        )

        self.assertEqual(bundle.packet.plan.source_cycles, ("cycle-14",))
        self.assertEqual(
            [artifact.path for artifact in bundle.artifacts],
            ["cycle-14/planning-packet.md", "cycle-14/plan.md", "cycle-14/lint.md"],
        )
        self.assertEqual([entry.path for entry in bundle.manifest.entries], [artifact.path for artifact in bundle.artifacts])
        self.assertTrue(bundle.verification.ok)
        self.assertIn("# Cycle 14 Packet", bundle.artifacts[0].content)
        self.assertIn("# Cycle 14 Manifest", bundle.manifest_markdown)
        self.assertIn("| `cycle-14/planning-packet.md` | `sha256:", bundle.manifest_markdown)
        self.assertIn("# Cycle 14 Verification", bundle.verification_markdown)
        self.assertIn("- Status: ok", bundle.verification_markdown)
        self.assertIn("- none", bundle.verification_markdown)


if __name__ == "__main__":
    unittest.main()
