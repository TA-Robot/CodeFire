from dataclasses import replace
import unittest

from evoagent.retrospective import (
    ResearchCycleRetrospective,
    ResearchCyclePlanningHandoffBundleBuilder,
    ResearchCyclePlanningHandoffBundleMarkdown,
    ResearchCyclePlanningHandoffReadinessGate,
    ResearchCyclePlanningHandoffReadinessMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummary,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummary,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactBuilder,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestBuilder,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactBuilder,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestBuilder,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestMarkdownVerificationMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestMarkdownVerifier,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownGate,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownGateMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerifier,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdownGate,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdownGateMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestBuilder,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestVerificationMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestVerifier,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdownGate,
    ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdownGateMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactBuilder,
    ResearchCyclePlanningHandoffReviewPacketArtifactManifestBuilder,
    ResearchCyclePlanningHandoffReviewPacketArtifactManifestMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactManifestVerificationMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactManifestVerifier,
    ResearchCyclePlanningHandoffReviewPacketBuilder,
    ResearchCyclePlanningHandoffReviewPacketMarkdown,
    ResearchCyclePlanningHandoffBundleSummary,
    ResearchCyclePlanLane,
    ResearchCyclePlan,
    ResearchCyclePlanItem,
    ResearchCyclePlanningPacketBuilder,
    ResearchCyclePlanningPacketManifestBuilder,
    ResearchCyclePlanningPacketManifestMarkdown,
    ResearchCyclePlanningPacketManifestVerification,
    ResearchCyclePlanningPacketManifestVerificationFinding,
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

    # cf-atom: TEST-research-cycle-planning-handoff-bundle-markdown-renders-index
    def test_research_cycle_planning_handoff_bundle_markdown_renders_index(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-15",
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
            packet_path="cycle-15/planning-packet.md",
            plan_path="cycle-15/plan.md",
            lint_path="cycle-15/lint.md",
            packet_title="Cycle 15 Packet",
        )

        markdown = ResearchCyclePlanningHandoffBundleMarkdown().render(
            bundle,
            title="Cycle 15 Handoff",
        )

        self.assertIn("# Cycle 15 Handoff", markdown)
        self.assertIn("- Source cycles: cycle-15", markdown)
        self.assertIn("- Packet status: ok", markdown)
        self.assertIn("- Manifest status: ok", markdown)
        self.assertIn("- Verification status: ok", markdown)
        self.assertIn("- Artifact count: 3", markdown)
        self.assertIn("| Path | Bytes |", markdown)
        self.assertIn("| `cycle-15/planning-packet.md` |", markdown)
        self.assertIn("- Manifest Markdown: included", markdown)
        self.assertIn("- Verification Markdown: included", markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-bundle-summary-reports-machine-readable-index
    def test_research_cycle_planning_handoff_bundle_summary_reports_machine_readable_index(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-16",
                    completed_runs=5,
                    improved_candidates=2,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=5.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=2,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=3.0,
            packet_path="cycle-16/planning-packet.md",
            plan_path="cycle-16/plan.md",
            lint_path="cycle-16/lint.md",
        )

        summary = ResearchCyclePlanningHandoffBundleSummary().summarize(bundle)

        self.assertEqual(summary["source_cycles"], ["cycle-16"])
        self.assertEqual(summary["packet_status"], "ok")
        self.assertEqual(summary["manifest_status"], "ok")
        self.assertEqual(summary["verification_status"], "ok")
        self.assertEqual(summary["artifact_count"], 3)
        self.assertEqual(summary["finding_count"], 0)
        self.assertEqual(summary["audits"], {"manifest_markdown": True, "verification_markdown": True})
        self.assertEqual(
            [artifact["path"] for artifact in summary["artifacts"]],  # type: ignore[index]
            ["cycle-16/planning-packet.md", "cycle-16/plan.md", "cycle-16/lint.md"],
        )
        first_artifact = summary["artifacts"][0]  # type: ignore[index]
        self.assertTrue(first_artifact["content_sha256"].startswith("sha256:"))  # type: ignore[index]
        self.assertGreater(first_artifact["byte_count"], 0)  # type: ignore[index]

    # cf-atom: TEST-research-cycle-planning-handoff-readiness-gate-blocks-drifted-bundle
    def test_research_cycle_planning_handoff_readiness_gate_blocks_drifted_bundle(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-17",
                    completed_runs=5,
                    improved_candidates=2,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=5.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=2,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=3.0,
            packet_path="cycle-17/planning-packet.md",
            plan_path="cycle-17/plan.md",
            lint_path="cycle-17/lint.md",
        )

        clean = ResearchCyclePlanningHandoffReadinessGate().evaluate(bundle)
        drifted = replace(
            bundle,
            verification=ResearchCyclePlanningPacketManifestVerification(
                (
                    ResearchCyclePlanningPacketManifestVerificationFinding(
                        path="cycle-17/plan.md",
                        message="sha256 digest mismatch",
                    ),
                )
            ),
            verification_markdown="",
        )
        blocked = ResearchCyclePlanningHandoffReadinessGate().evaluate(drifted)

        self.assertTrue(clean["ready"])
        self.assertEqual(clean["status"], "ready")
        self.assertFalse(blocked["ready"])
        self.assertEqual(blocked["status"], "blocked")
        self.assertIn("verification has 1 finding(s)", blocked["blockers"])
        self.assertIn("verification audit Markdown is missing", blocked["blockers"])
        self.assertEqual(blocked["artifact_count"], 3)
        self.assertEqual(blocked["finding_count"], 1)

    # cf-atom: TEST-research-cycle-planning-handoff-readiness-markdown-renders-status
    def test_research_cycle_planning_handoff_readiness_markdown_renders_status(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-18",
                    completed_runs=5,
                    improved_candidates=2,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=5.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=2,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=3.0,
            packet_path="cycle-18/planning-packet.md",
            plan_path="cycle-18/plan.md",
            lint_path="cycle-18/lint.md",
        )
        drifted = replace(
            bundle,
            verification=ResearchCyclePlanningPacketManifestVerification(
                (
                    ResearchCyclePlanningPacketManifestVerificationFinding(
                        path="cycle-18/plan.md",
                        message="sha256 digest mismatch",
                    ),
                )
            ),
        )

        clean_markdown = ResearchCyclePlanningHandoffReadinessMarkdown().render(
            ResearchCyclePlanningHandoffReadinessGate().evaluate(bundle),
            title="Cycle 18 Readiness",
        )
        blocked_markdown = ResearchCyclePlanningHandoffReadinessMarkdown().render(
            ResearchCyclePlanningHandoffReadinessGate().evaluate(drifted),
            title="Cycle 18 Readiness",
        )

        self.assertIn("# Cycle 18 Readiness", clean_markdown)
        self.assertIn("- Ready: yes", clean_markdown)
        self.assertIn("- Status: ready", clean_markdown)
        self.assertIn("- none", clean_markdown)
        self.assertIn("- Ready: no", blocked_markdown)
        self.assertIn("- Status: blocked", blocked_markdown)
        self.assertIn("- Finding count: 1", blocked_markdown)
        self.assertIn("- verification has 1 finding(s)", blocked_markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-bundles-summary-and-readiness
    def test_research_cycle_planning_handoff_review_packet_bundles_summary_and_readiness(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-19",
                    completed_runs=5,
                    improved_candidates=2,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=5.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=2,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=3.0,
            packet_path="cycle-19/planning-packet.md",
            plan_path="cycle-19/plan.md",
            lint_path="cycle-19/lint.md",
        )

        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(
            bundle,
            readiness_title="Cycle 19 Readiness",
        )

        self.assertIs(packet.bundle, bundle)
        self.assertEqual(packet.summary["source_cycles"], ["cycle-19"])
        self.assertEqual(packet.summary["artifact_count"], 3)
        self.assertTrue(packet.readiness["ready"])
        self.assertEqual(packet.readiness["status"], "ready")
        self.assertIn("# Cycle 19 Readiness", packet.readiness_markdown)
        self.assertIn("- Ready: yes", packet.readiness_markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-markdown-renders-review-document
    def test_research_cycle_planning_handoff_review_packet_markdown_renders_review_document(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-20",
                    completed_runs=6,
                    improved_candidates=3,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=6.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=3,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=4.0,
            packet_path="cycle-20/planning-packet.md",
            plan_path="cycle-20/plan.md",
            lint_path="cycle-20/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)

        markdown = ResearchCyclePlanningHandoffReviewPacketMarkdown().render(
            packet,
            title="Cycle 20 Review Packet",
        )

        self.assertIn("# Cycle 20 Review Packet", markdown)
        self.assertIn("- Source cycles: cycle-20", markdown)
        self.assertIn("- Ready: yes", markdown)
        self.assertIn("- Packet status: ok", markdown)
        self.assertIn("| `cycle-20/plan.md` |", markdown)
        self.assertIn("sha256:", markdown)
        self.assertIn("- Readiness Markdown: included", markdown)
        self.assertIn("- Manifest Markdown: included", markdown)
        self.assertIn("- Verification Markdown: included", markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-builder-packages-audits
    def test_research_cycle_planning_handoff_review_packet_artifact_builder_packages_audits(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-21",
                    completed_runs=6,
                    improved_candidates=3,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=6.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=3,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=4.0,
            packet_path="cycle-21/planning-packet.md",
            plan_path="cycle-21/plan.md",
            lint_path="cycle-21/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)

        artifacts = ResearchCyclePlanningHandoffReviewPacketArtifactBuilder().build(
            packet,
            review_path="cycle-21/review-packet.md",
            readiness_path="cycle-21/readiness.md",
            manifest_path="cycle-21/manifest.md",
            verification_path="cycle-21/verification.md",
        )

        self.assertEqual(
            [artifact.path for artifact in artifacts],
            [
                "cycle-21/review-packet.md",
                "cycle-21/readiness.md",
                "cycle-21/manifest.md",
                "cycle-21/verification.md",
            ],
        )
        self.assertIn("# Research Cycle Planning Handoff Review Packet", artifacts[0].content)
        self.assertIn("- Ready: yes", artifacts[1].content)
        self.assertIn("| `cycle-21/plan.md` |", artifacts[2].content)
        self.assertIn("Manifest Verification", artifacts[3].content)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-manifest-records-artifacts
    def test_research_cycle_planning_handoff_review_packet_artifact_manifest_records_artifacts(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-22",
                    completed_runs=7,
                    improved_candidates=4,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=7.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=4,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=4.0,
            packet_path="cycle-22/planning-packet.md",
            plan_path="cycle-22/plan.md",
            lint_path="cycle-22/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        artifacts = ResearchCyclePlanningHandoffReviewPacketArtifactBuilder().build(
            packet,
            review_path="cycle-22/review-packet.md",
            readiness_path="cycle-22/readiness.md",
            manifest_path="cycle-22/manifest.md",
            verification_path="cycle-22/verification.md",
        )

        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactManifestBuilder().build(
            packet,
            artifacts,
        )

        self.assertEqual(manifest.source_cycles, ("cycle-22",))
        self.assertEqual(manifest.status, "ok")
        self.assertEqual([entry.path for entry in manifest.entries], [artifact.path for artifact in artifacts])
        self.assertTrue(all(entry.content_sha256.startswith("sha256:") for entry in manifest.entries))
        self.assertEqual(
            [entry.byte_count for entry in manifest.entries],
            [len(artifact.content.encode("utf-8")) for artifact in artifacts],
        )

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-manifest-markdown-renders-table
    def test_research_cycle_planning_handoff_review_packet_artifact_manifest_markdown_renders_table(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-23",
                    completed_runs=7,
                    improved_candidates=4,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=7.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=4,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=4.0,
            packet_path="cycle-23/planning-packet.md",
            plan_path="cycle-23/plan.md",
            lint_path="cycle-23/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        artifacts = ResearchCyclePlanningHandoffReviewPacketArtifactBuilder().build(
            packet,
            review_path="cycle-23/review-packet.md",
            readiness_path="cycle-23/readiness.md",
            manifest_path="cycle-23/manifest.md",
            verification_path="cycle-23/verification.md",
        )
        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactManifestBuilder().build(
            packet,
            artifacts,
        )

        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactManifestMarkdown().render(
            manifest,
            title="Cycle 23 Artifact Manifest",
        )

        self.assertIn("# Cycle 23 Artifact Manifest", markdown)
        self.assertIn("- Source cycles: cycle-23", markdown)
        self.assertIn("- Status: ok", markdown)
        self.assertIn("- Artifact count: 4", markdown)
        self.assertIn("| `cycle-23/review-packet.md` |", markdown)
        self.assertIn("sha256:", markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-manifest-verifier-detects-drift
    def test_research_cycle_planning_handoff_review_packet_artifact_manifest_verifier_detects_drift(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-24",
                    completed_runs=7,
                    improved_candidates=4,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=1.0,
                    remaining_budget=7.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=4,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=4.0,
            packet_path="cycle-24/planning-packet.md",
            plan_path="cycle-24/plan.md",
            lint_path="cycle-24/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        artifacts = ResearchCyclePlanningHandoffReviewPacketArtifactBuilder().build(
            packet,
            review_path="cycle-24/review-packet.md",
            readiness_path="cycle-24/readiness.md",
            manifest_path="cycle-24/manifest.md",
            verification_path="cycle-24/verification.md",
        )
        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactManifestBuilder().build(
            packet,
            artifacts,
        )
        artifact_contents = {artifact.path: artifact.content for artifact in artifacts}
        clean = ResearchCyclePlanningHandoffReviewPacketArtifactManifestVerifier().verify(
            manifest,
            artifact_contents,
        )
        drifted_contents = dict(artifact_contents)
        drifted_contents["cycle-24/review-packet.md"] += "\nchanged\n"
        drifted_contents.pop("cycle-24/readiness.md")

        drifted = ResearchCyclePlanningHandoffReviewPacketArtifactManifestVerifier().verify(
            manifest,
            drifted_contents,
        )

        self.assertTrue(clean.ok)
        self.assertFalse(drifted.ok)
        self.assertIn("sha256 digest mismatch", [finding.message for finding in drifted.findings])
        self.assertIn("byte count mismatch", [finding.message for finding in drifted.findings])
        self.assertIn("artifact is missing", [finding.message for finding in drifted.findings])

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-manifest-verification-markdown-renders-findings
    def test_research_cycle_planning_handoff_review_packet_artifact_manifest_verification_markdown_renders_findings(
        self,
    ) -> None:
        clean = ResearchCyclePlanningPacketManifestVerification(findings=())
        drifted = ResearchCyclePlanningPacketManifestVerification(
            findings=(
                ResearchCyclePlanningPacketManifestVerificationFinding(
                    path="cycle-25/review-packet.md",
                    message="sha256 digest mismatch",
                ),
                ResearchCyclePlanningPacketManifestVerificationFinding(
                    path="cycle-25/readiness.md",
                    message="artifact is missing",
                ),
            )
        )

        clean_markdown = ResearchCyclePlanningHandoffReviewPacketArtifactManifestVerificationMarkdown().render(
            clean,
            title="Cycle 25 Artifact Manifest Verification",
        )
        drifted_markdown = ResearchCyclePlanningHandoffReviewPacketArtifactManifestVerificationMarkdown().render(
            drifted,
            title="Cycle 25 Artifact Manifest Verification",
        )

        self.assertIn("# Cycle 25 Artifact Manifest Verification", clean_markdown)
        self.assertIn("- Status: ok", clean_markdown)
        self.assertIn("- Finding count: 0", clean_markdown)
        self.assertIn("- none", clean_markdown)
        self.assertIn("- Status: blocked", drifted_markdown)
        self.assertIn("- Finding count: 2", drifted_markdown)
        self.assertIn("`cycle-25/review-packet.md`: sha256 digest mismatch", drifted_markdown)
        self.assertIn("`cycle-25/readiness.md`: artifact is missing", drifted_markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-builder-builds-final-audits
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_builder_builds_final_audits(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-26",
                    completed_runs=8,
                    improved_candidates=5,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.9,
                    remaining_budget=8.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=5,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=5.0,
            packet_path="cycle-26/planning-packet.md",
            plan_path="cycle-26/plan.md",
            lint_path="cycle-26/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)

        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-26/review-packet.md",
            readiness_path="cycle-26/readiness.md",
            manifest_path="cycle-26/manifest.md",
            verification_path="cycle-26/verification.md",
            manifest_title="Cycle 26 Artifact Manifest",
            verification_title="Cycle 26 Artifact Manifest Verification",
        )

        self.assertEqual(packet, archive.packet)
        self.assertEqual(4, len(archive.artifacts))
        self.assertEqual(4, len(archive.manifest.entries))
        self.assertTrue(archive.verification.ok)
        self.assertIn("# Cycle 26 Artifact Manifest", archive.manifest_markdown)
        self.assertIn("# Cycle 26 Artifact Manifest Verification", archive.verification_markdown)
        self.assertIn("- Status: ok", archive.verification_markdown)
        self.assertIn("- Finding count: 0", archive.verification_markdown)
        self.assertEqual(
            [
                "cycle-26/review-packet.md",
                "cycle-26/readiness.md",
                "cycle-26/manifest.md",
                "cycle-26/verification.md",
            ],
            [artifact.path for artifact in archive.artifacts],
        )

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-reports-index
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_reports_index(self) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-27",
                    completed_runs=9,
                    improved_candidates=6,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.8,
                    remaining_budget=9.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=6,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=6.0,
            packet_path="cycle-27/planning-packet.md",
            plan_path="cycle-27/plan.md",
            lint_path="cycle-27/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-27/review-packet.md",
            readiness_path="cycle-27/readiness.md",
            manifest_path="cycle-27/manifest.md",
            verification_path="cycle-27/verification.md",
        )

        summary = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummary().summarize(archive)

        self.assertEqual(["cycle-27"], summary["source_cycles"])
        self.assertEqual("ok", summary["archive_status"])
        self.assertEqual("ready", summary["readiness_status"])
        self.assertEqual(True, summary["ready"])
        self.assertEqual("ok", summary["manifest_status"])
        self.assertEqual("ok", summary["verification_status"])
        self.assertEqual(4, summary["artifact_count"])
        self.assertEqual(0, summary["finding_count"])
        self.assertEqual(
            {
                "manifest_markdown": True,
                "verification_markdown": True,
            },
            summary["audits"],
        )
        artifacts = summary["artifacts"]
        self.assertIsInstance(artifacts, list)
        self.assertEqual("cycle-27/review-packet.md", artifacts[0]["path"])
        self.assertGreater(artifacts[0]["byte_count"], 0)
        self.assertTrue(artifacts[0]["content_sha256"].startswith("sha256:"))

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-markdown-renders-index
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_markdown_renders_index(
        self,
    ) -> None:
        summary = {
            "source_cycles": ["cycle-28"],
            "archive_status": "ok",
            "ready": True,
            "readiness_status": "ready",
            "manifest_status": "ok",
            "verification_status": "ok",
            "artifact_count": 2,
            "finding_count": 0,
            "audits": {
                "manifest_markdown": True,
                "verification_markdown": True,
            },
            "artifacts": [
                {
                    "path": "cycle-28/review-packet.md",
                    "byte_count": 123,
                    "content_sha256": "sha256:abc123",
                },
                {
                    "path": "cycle-28/readiness.md",
                    "byte_count": 45,
                    "content_sha256": "sha256:def456",
                },
            ],
        }

        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdown().render(
            summary,
            title="Cycle 28 Archive Summary",
        )

        self.assertIn("# Cycle 28 Archive Summary", markdown)
        self.assertIn("- Source cycles: cycle-28", markdown)
        self.assertIn("- Archive status: ok", markdown)
        self.assertIn("- Ready: yes", markdown)
        self.assertIn("- Readiness status: ready", markdown)
        self.assertIn("- Manifest status: ok", markdown)
        self.assertIn("- Verification status: ok", markdown)
        self.assertIn("- Artifact count: 2", markdown)
        self.assertIn("- Finding count: 0", markdown)
        self.assertIn("| `cycle-28/review-packet.md` | 123 | `sha256:abc123` |", markdown)
        self.assertIn("- Manifest Markdown: included", markdown)
        self.assertIn("- Verification Markdown: included", markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-markdown-gate-blocks-drift
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_markdown_gate_blocks_drift(
        self,
    ) -> None:
        summary = {
            "source_cycles": ["cycle-29"],
            "archive_status": "ok",
            "ready": True,
            "readiness_status": "ready",
            "manifest_status": "ok",
            "verification_status": "ok",
            "artifact_count": 1,
            "finding_count": 0,
            "audits": {
                "manifest_markdown": True,
                "verification_markdown": True,
            },
            "artifacts": [
                {
                    "path": "cycle-29/review-packet.md",
                    "byte_count": 321,
                    "content_sha256": "sha256:cycle29",
                }
            ],
        }
        renderer = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdown()
        gate = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdownGate()

        clean = gate.evaluate(summary, renderer.render(summary, title="Cycle 29 Archive Summary"))

        self.assertEqual(
            {
                "ready": True,
                "status": "ready",
                "blockers": [],
                "artifact_count": 1,
                "finding_count": 0,
                "checked_artifact_count": 1,
            },
            clean,
        )

        drifted_summary = dict(summary)
        drifted_summary["artifact_count"] = 2
        drifted_markdown = renderer.render(summary).replace("- Verification status: ok\n", "")

        blocked = gate.evaluate(drifted_summary, drifted_markdown)

        self.assertEqual(False, blocked["ready"])
        self.assertEqual("blocked", blocked["status"])
        self.assertIn("verification status line is missing", blocked["blockers"])
        self.assertIn("artifact count 2 does not match 1 artifact record(s)", blocked["blockers"])

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-markdown-gate-markdown-renders-blockers
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_markdown_gate_markdown_renders_blockers(
        self,
    ) -> None:
        result = {
            "ready": False,
            "status": "blocked",
            "artifact_count": 2,
            "checked_artifact_count": 1,
            "finding_count": 0,
            "blockers": [
                "verification status line is missing",
                "artifact count 2 does not match 1 artifact record(s)",
            ],
        }

        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdownGateMarkdown().render(
            result,
            title="Cycle 30 Archive Summary Markdown Gate",
        )

        self.assertIn("# Cycle 30 Archive Summary Markdown Gate", markdown)
        self.assertIn("- Ready: no", markdown)
        self.assertIn("- Status: blocked", markdown)
        self.assertIn("- Artifact count: 2", markdown)
        self.assertIn("- Checked artifact count: 1", markdown)
        self.assertIn("- Finding count: 0", markdown)
        self.assertIn("- verification status line is missing", markdown)
        self.assertIn("- artifact count 2 does not match 1 artifact record(s)", markdown)

        clean_markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdownGateMarkdown().render(
            {"ready": True, "status": "ready", "blockers": []}
        )
        self.assertIn("- Ready: yes", clean_markdown)
        self.assertIn("- none", clean_markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-builder-packages-gated-summary
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_builder_packages_gated_summary(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-31",
                    completed_runs=10,
                    improved_candidates=7,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.7,
                    remaining_budget=10.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=7,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=7.0,
            packet_path="cycle-31/planning-packet.md",
            plan_path="cycle-31/plan.md",
            lint_path="cycle-31/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-31/review-packet.md",
            readiness_path="cycle-31/readiness.md",
            manifest_path="cycle-31/manifest.md",
            verification_path="cycle-31/verification.md",
        )

        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-31/archive-summary.md",
            gate_path="cycle-31/archive-summary-gate.md",
            summary_title="Cycle 31 Archive Summary",
            gate_title="Cycle 31 Archive Summary Gate",
        )

        self.assertEqual(archive, handoff.archive)
        self.assertEqual("ok", handoff.summary["archive_status"])
        self.assertEqual(True, handoff.gate_result["ready"])
        self.assertIn("# Cycle 31 Archive Summary", handoff.summary_markdown)
        self.assertIn("# Cycle 31 Archive Summary Gate", handoff.gate_markdown)
        self.assertEqual(
            [
                "cycle-31/archive-summary.md",
                "cycle-31/archive-summary-gate.md",
            ],
            [artifact.path for artifact in handoff.artifacts],
        )
        self.assertEqual(handoff.summary_markdown, handoff.artifacts[0].content)
        self.assertEqual(handoff.gate_markdown, handoff.artifacts[1].content)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-manifest-records-summary-artifacts
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_manifest_records_summary_artifacts(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-32",
                    completed_runs=11,
                    improved_candidates=8,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.6,
                    remaining_budget=11.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=8,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=8.0,
            packet_path="cycle-32/planning-packet.md",
            plan_path="cycle-32/plan.md",
            lint_path="cycle-32/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-32/review-packet.md",
            readiness_path="cycle-32/readiness.md",
            manifest_path="cycle-32/manifest.md",
            verification_path="cycle-32/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-32/archive-summary.md",
            gate_path="cycle-32/archive-summary-gate.md",
        )

        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestBuilder().build(
            handoff,
        )

        self.assertEqual(("cycle-32",), manifest.source_cycles)
        self.assertEqual("ok", manifest.status)
        self.assertEqual(
            [
                "cycle-32/archive-summary.md",
                "cycle-32/archive-summary-gate.md",
            ],
            [entry.path for entry in manifest.entries],
        )
        self.assertEqual(len(handoff.summary_markdown.encode("utf-8")), manifest.entries[0].byte_count)
        self.assertTrue(manifest.entries[0].content_sha256.startswith("sha256:"))
        self.assertEqual(len(handoff.gate_markdown.encode("utf-8")), manifest.entries[1].byte_count)
        self.assertTrue(manifest.entries[1].content_sha256.startswith("sha256:"))

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-manifest-markdown-renders-table
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_manifest_markdown_renders_table(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-33",
                    completed_runs=12,
                    improved_candidates=9,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.5,
                    remaining_budget=12.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=9,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=9.0,
            packet_path="cycle-33/planning-packet.md",
            plan_path="cycle-33/plan.md",
            lint_path="cycle-33/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-33/review-packet.md",
            readiness_path="cycle-33/readiness.md",
            manifest_path="cycle-33/manifest.md",
            verification_path="cycle-33/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-33/archive-summary.md",
            gate_path="cycle-33/archive-summary-gate.md",
        )
        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestBuilder().build(
            handoff,
        )

        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestMarkdown().render(
            manifest,
            title="Cycle 33 Archive Summary Artifact Manifest",
        )

        self.assertIn("# Cycle 33 Archive Summary Artifact Manifest", markdown)
        self.assertIn("- Source cycles: cycle-33", markdown)
        self.assertIn("- Status: ok", markdown)
        self.assertIn("- Artifact count: 2", markdown)
        self.assertIn("| Path | SHA-256 | Bytes |", markdown)
        self.assertIn("| `cycle-33/archive-summary.md` |", markdown)
        self.assertIn("| `cycle-33/archive-summary-gate.md` |", markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-manifest-verifier-detects-drift
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_manifest_verifier_detects_drift(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-34",
                    completed_runs=14,
                    improved_candidates=10,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.4,
                    remaining_budget=14.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=10,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=10.0,
            packet_path="cycle-34/planning-packet.md",
            plan_path="cycle-34/plan.md",
            lint_path="cycle-34/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-34/review-packet.md",
            readiness_path="cycle-34/readiness.md",
            manifest_path="cycle-34/manifest.md",
            verification_path="cycle-34/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-34/archive-summary.md",
            gate_path="cycle-34/archive-summary-gate.md",
        )
        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestBuilder().build(
            handoff,
        )
        artifact_contents = {artifact.path: artifact.content for artifact in handoff.artifacts}
        verifier = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestVerifier()

        clean = verifier.verify(manifest, artifact_contents)
        self.assertTrue(clean.ok)
        self.assertEqual((), clean.findings)

        drifted = dict(artifact_contents)
        drifted["cycle-34/archive-summary.md"] = "corrupted summary"
        drifted.pop("cycle-34/archive-summary-gate.md")
        result = verifier.verify(manifest, drifted)

        self.assertFalse(result.ok)
        self.assertEqual(
            [
                ("cycle-34/archive-summary.md", "sha256 digest mismatch"),
                ("cycle-34/archive-summary.md", "byte count mismatch"),
                ("cycle-34/archive-summary-gate.md", "artifact is missing"),
            ],
            [(finding.path, finding.message) for finding in result.findings],
        )

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-manifest-verification-markdown-renders-findings
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_manifest_verification_markdown_renders_findings(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-35",
                    completed_runs=15,
                    improved_candidates=11,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.4,
                    remaining_budget=15.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=11,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=11.0,
            packet_path="cycle-35/planning-packet.md",
            plan_path="cycle-35/plan.md",
            lint_path="cycle-35/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-35/review-packet.md",
            readiness_path="cycle-35/readiness.md",
            manifest_path="cycle-35/manifest.md",
            verification_path="cycle-35/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-35/archive-summary.md",
            gate_path="cycle-35/archive-summary-gate.md",
        )
        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestBuilder().build(
            handoff,
        )
        artifact_contents = {artifact.path: artifact.content for artifact in handoff.artifacts}
        artifact_contents["cycle-35/archive-summary.md"] = "corrupted summary"
        verification = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestVerifier().verify(
            manifest,
            artifact_contents,
        )

        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestVerificationMarkdown().render(
            verification,
            title="Cycle 35 Archive Summary Artifact Manifest Verification",
        )

        self.assertIn("# Cycle 35 Archive Summary Artifact Manifest Verification", markdown)
        self.assertIn("- Status: blocked", markdown)
        self.assertIn("- Finding count: 2", markdown)
        self.assertIn("- `cycle-35/archive-summary.md`: sha256 digest mismatch", markdown)
        self.assertIn("- `cycle-35/archive-summary.md`: byte count mismatch", markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-builder-builds-audits
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_builder_builds_audits(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-36",
                    completed_runs=16,
                    improved_candidates=12,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.4,
                    remaining_budget=16.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=12,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=12.0,
            packet_path="cycle-36/planning-packet.md",
            plan_path="cycle-36/plan.md",
            lint_path="cycle-36/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-36/review-packet.md",
            readiness_path="cycle-36/readiness.md",
            manifest_path="cycle-36/manifest.md",
            verification_path="cycle-36/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-36/archive-summary.md",
            gate_path="cycle-36/archive-summary-gate.md",
        )

        summary_archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder().build(
            handoff,
            manifest_title="Cycle 36 Archive Summary Artifact Manifest",
            verification_title="Cycle 36 Archive Summary Artifact Manifest Verification",
        )

        self.assertEqual(handoff, summary_archive.handoff)
        self.assertEqual(handoff.artifacts, summary_archive.artifacts)
        self.assertEqual(("cycle-36",), summary_archive.manifest.source_cycles)
        self.assertEqual("ok", summary_archive.manifest.status)
        self.assertEqual(2, len(summary_archive.manifest.entries))
        self.assertTrue(summary_archive.verification.ok)
        self.assertIn("# Cycle 36 Archive Summary Artifact Manifest", summary_archive.manifest_markdown)
        self.assertIn("- Artifact count: 2", summary_archive.manifest_markdown)
        self.assertIn("# Cycle 36 Archive Summary Artifact Manifest Verification", summary_archive.verification_markdown)
        self.assertIn("- Status: ok", summary_archive.verification_markdown)
        self.assertIn("- Finding count: 0", summary_archive.verification_markdown)
        self.assertIn("- none", summary_archive.verification_markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-reports-index
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_reports_index(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-37",
                    completed_runs=17,
                    improved_candidates=13,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.4,
                    remaining_budget=17.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=13,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=13.0,
            packet_path="cycle-37/planning-packet.md",
            plan_path="cycle-37/plan.md",
            lint_path="cycle-37/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-37/review-packet.md",
            readiness_path="cycle-37/readiness.md",
            manifest_path="cycle-37/manifest.md",
            verification_path="cycle-37/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-37/archive-summary.md",
            gate_path="cycle-37/archive-summary-gate.md",
        )
        summary_archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder().build(
            handoff,
        )

        summary = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummary().summarize(
            summary_archive,
        )

        self.assertEqual(["cycle-37"], summary["source_cycles"])
        self.assertEqual("ok", summary["parent_archive_status"])
        self.assertEqual("ok", summary["archive_status"])
        self.assertEqual("ok", summary["manifest_status"])
        self.assertEqual("ok", summary["verification_status"])
        self.assertEqual(2, summary["artifact_count"])
        self.assertEqual(0, summary["finding_count"])
        self.assertEqual(
            {"manifest_markdown": True, "verification_markdown": True},
            summary["audits"],
        )
        artifacts = summary["artifacts"]
        self.assertIsInstance(artifacts, list)
        self.assertEqual("cycle-37/archive-summary.md", artifacts[0]["path"])
        self.assertEqual(len(handoff.summary_markdown.encode("utf-8")), artifacts[0]["byte_count"])
        self.assertTrue(artifacts[0]["content_sha256"].startswith("sha256:"))
        self.assertEqual("cycle-37/archive-summary-gate.md", artifacts[1]["path"])

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-markdown-renders-index
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_markdown_renders_index(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-38",
                    completed_runs=18,
                    improved_candidates=14,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.4,
                    remaining_budget=18.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=14,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=14.0,
            packet_path="cycle-38/planning-packet.md",
            plan_path="cycle-38/plan.md",
            lint_path="cycle-38/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-38/review-packet.md",
            readiness_path="cycle-38/readiness.md",
            manifest_path="cycle-38/manifest.md",
            verification_path="cycle-38/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-38/archive-summary.md",
            gate_path="cycle-38/archive-summary-gate.md",
        )
        summary_archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder().build(
            handoff,
        )
        summary = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummary().summarize(
            summary_archive,
        )

        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdown().render(
            summary,
            title="Cycle 38 Archive Summary Artifact Archive Summary",
        )

        self.assertIn("# Cycle 38 Archive Summary Artifact Archive Summary", markdown)
        self.assertIn("- Source cycles: cycle-38", markdown)
        self.assertIn("- Parent archive status: ok", markdown)
        self.assertIn("- Archive status: ok", markdown)
        self.assertIn("- Manifest status: ok", markdown)
        self.assertIn("- Verification status: ok", markdown)
        self.assertIn("- Artifact count: 2", markdown)
        self.assertIn("- Finding count: 0", markdown)
        self.assertIn("| `cycle-38/archive-summary.md` |", markdown)
        self.assertIn("| `cycle-38/archive-summary-gate.md` |", markdown)
        self.assertIn("- Manifest Markdown: included", markdown)
        self.assertIn("- Verification Markdown: included", markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-markdown-gate-blocks-drift
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_markdown_gate_blocks_drift(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-39",
                    completed_runs=19,
                    improved_candidates=15,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.4,
                    remaining_budget=19.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=15,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=15.0,
            packet_path="cycle-39/planning-packet.md",
            plan_path="cycle-39/plan.md",
            lint_path="cycle-39/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-39/review-packet.md",
            readiness_path="cycle-39/readiness.md",
            manifest_path="cycle-39/manifest.md",
            verification_path="cycle-39/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-39/archive-summary.md",
            gate_path="cycle-39/archive-summary-gate.md",
        )
        summary_archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder().build(
            handoff,
        )
        summary = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummary().summarize(
            summary_archive,
        )
        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdown().render(
            summary,
        )
        gate = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdownGate()

        clean = gate.evaluate(summary, markdown)
        self.assertTrue(clean["ready"])
        self.assertEqual("ready", clean["status"])
        self.assertEqual([], clean["blockers"])
        self.assertEqual(2, clean["artifact_count"])
        self.assertEqual(2, clean["checked_artifact_count"])
        self.assertEqual(0, clean["finding_count"])

        drifted_summary = dict(summary)
        drifted_summary["artifact_count"] = 3
        drifted_markdown = markdown.replace("- Parent archive status: ok\n", "").replace(
            "| `cycle-39/archive-summary-gate.md` |",
            "| `cycle-39/missing-gate.md` |",
        )
        result = gate.evaluate(drifted_summary, drifted_markdown)

        self.assertFalse(result["ready"])
        self.assertEqual("blocked", result["status"])
        self.assertEqual(
            [
                "parent archive status line is missing",
                "artifact count line is missing",
                "artifact count 3 does not match 2 artifact record(s)",
                "artifact row is missing for cycle-39/archive-summary-gate.md",
            ],
            result["blockers"],
        )

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-markdown-gate-markdown-renders-blockers
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_markdown_gate_markdown_renders_blockers(
        self,
    ) -> None:
        result = {
            "ready": False,
            "status": "blocked",
            "artifact_count": 3,
            "checked_artifact_count": 2,
            "finding_count": 1,
            "blockers": [
                "parent archive status line is missing",
                "artifact row is missing for cycle-40/archive-summary-gate.md",
            ],
        }

        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdownGateMarkdown().render(
            result,
            title="Cycle 40 Archive Summary Artifact Archive Summary Markdown Gate",
        )

        self.assertIn("# Cycle 40 Archive Summary Artifact Archive Summary Markdown Gate", markdown)
        self.assertIn("- Ready: no", markdown)
        self.assertIn("- Status: blocked", markdown)
        self.assertIn("- Artifact count: 3", markdown)
        self.assertIn("- Checked artifact count: 2", markdown)
        self.assertIn("- Finding count: 1", markdown)
        self.assertIn("- parent archive status line is missing", markdown)
        self.assertIn("- artifact row is missing for cycle-40/archive-summary-gate.md", markdown)

        clean = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryMarkdownGateMarkdown().render(
            {"ready": True, "status": "ready", "artifact_count": 2, "checked_artifact_count": 2, "finding_count": 0},
        )
        self.assertIn("- Ready: yes", clean)
        self.assertIn("- none", clean)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-builder-packages-gated-summary
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_artifact_builder_packages_gated_summary(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-41",
                    completed_runs=20,
                    improved_candidates=16,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.4,
                    remaining_budget=20.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=16,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=16.0,
            packet_path="cycle-41/planning-packet.md",
            plan_path="cycle-41/plan.md",
            lint_path="cycle-41/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-41/review-packet.md",
            readiness_path="cycle-41/readiness.md",
            manifest_path="cycle-41/manifest.md",
            verification_path="cycle-41/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-41/archive-summary.md",
            gate_path="cycle-41/archive-summary-gate.md",
        )
        summary_archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder().build(
            handoff,
        )

        packaged = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactBuilder().build(
            summary_archive,
            summary_path="cycle-41/archive-summary-artifact-archive-summary.md",
            gate_path="cycle-41/archive-summary-artifact-archive-summary-gate.md",
            summary_title="Cycle 41 Archive Summary Artifact Archive Summary",
            gate_title="Cycle 41 Archive Summary Artifact Archive Summary Markdown Gate",
        )

        self.assertEqual(summary_archive, packaged.archive)
        self.assertEqual(["cycle-41"], packaged.summary["source_cycles"])
        self.assertEqual("ok", packaged.summary["archive_status"])
        self.assertTrue(packaged.gate_result["ready"])
        self.assertIn("# Cycle 41 Archive Summary Artifact Archive Summary", packaged.summary_markdown)
        self.assertIn("# Cycle 41 Archive Summary Artifact Archive Summary Markdown Gate", packaged.gate_markdown)
        self.assertEqual(
            ("cycle-41/archive-summary-artifact-archive-summary.md", "cycle-41/archive-summary-artifact-archive-summary-gate.md"),
            tuple(artifact.path for artifact in packaged.artifacts),
        )
        self.assertEqual(packaged.summary_markdown, packaged.artifacts[0].content)
        self.assertEqual(packaged.gate_markdown, packaged.artifacts[1].content)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-records-summary-artifacts
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_artifact_manifest_records_summary_artifacts(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-42",
                    completed_runs=21,
                    improved_candidates=17,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.38,
                    remaining_budget=21.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=17,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=17.0,
            packet_path="cycle-42/planning-packet.md",
            plan_path="cycle-42/plan.md",
            lint_path="cycle-42/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-42/review-packet.md",
            readiness_path="cycle-42/readiness.md",
            manifest_path="cycle-42/manifest.md",
            verification_path="cycle-42/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-42/archive-summary.md",
            gate_path="cycle-42/archive-summary-gate.md",
        )
        summary_archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder().build(
            handoff,
        )
        packaged = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactBuilder().build(
            summary_archive,
            summary_path="cycle-42/archive-summary-artifact-archive-summary.md",
            gate_path="cycle-42/archive-summary-artifact-archive-summary-gate.md",
        )

        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestBuilder().build(
            packaged,
        )

        self.assertEqual(("cycle-42",), manifest.source_cycles)
        self.assertEqual("ok", manifest.status)
        self.assertEqual(
            [
                "cycle-42/archive-summary-artifact-archive-summary.md",
                "cycle-42/archive-summary-artifact-archive-summary-gate.md",
            ],
            [entry.path for entry in manifest.entries],
        )
        self.assertEqual(len(packaged.summary_markdown.encode("utf-8")), manifest.entries[0].byte_count)
        self.assertTrue(manifest.entries[0].content_sha256.startswith("sha256:"))
        self.assertEqual(len(packaged.gate_markdown.encode("utf-8")), manifest.entries[1].byte_count)
        self.assertTrue(manifest.entries[1].content_sha256.startswith("sha256:"))

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-markdown-renders-table
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_artifact_manifest_markdown_renders_table(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-43",
                    completed_runs=22,
                    improved_candidates=18,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.36,
                    remaining_budget=22.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=18,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=18.0,
            packet_path="cycle-43/planning-packet.md",
            plan_path="cycle-43/plan.md",
            lint_path="cycle-43/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-43/review-packet.md",
            readiness_path="cycle-43/readiness.md",
            manifest_path="cycle-43/manifest.md",
            verification_path="cycle-43/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-43/archive-summary.md",
            gate_path="cycle-43/archive-summary-gate.md",
        )
        summary_archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder().build(
            handoff,
        )
        packaged = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactBuilder().build(
            summary_archive,
            summary_path="cycle-43/archive-summary-artifact-archive-summary.md",
            gate_path="cycle-43/archive-summary-artifact-archive-summary-gate.md",
        )
        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestBuilder().build(
            packaged,
        )

        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdown().render(
            manifest,
            title="Cycle 43 Archive Summary Artifact Archive Summary Artifact Manifest",
        )

        self.assertIn("# Cycle 43 Archive Summary Artifact Archive Summary Artifact Manifest", markdown)
        self.assertIn("- Source cycles: cycle-43", markdown)
        self.assertIn("- Status: ok", markdown)
        self.assertIn("- Artifact count: 2", markdown)
        self.assertIn("| Path | SHA-256 | Bytes |", markdown)
        for entry in manifest.entries:
            self.assertIn(f"| `{entry.path}` | `{entry.content_sha256}` | {entry.byte_count} |", markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-markdown-verifier-detects-drift
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_artifact_manifest_markdown_verifier_detects_drift(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-44",
                    completed_runs=23,
                    improved_candidates=19,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.34,
                    remaining_budget=23.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=19,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=19.0,
            packet_path="cycle-44/planning-packet.md",
            plan_path="cycle-44/plan.md",
            lint_path="cycle-44/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-44/review-packet.md",
            readiness_path="cycle-44/readiness.md",
            manifest_path="cycle-44/manifest.md",
            verification_path="cycle-44/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-44/archive-summary.md",
            gate_path="cycle-44/archive-summary-gate.md",
        )
        summary_archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder().build(
            handoff,
        )
        packaged = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactBuilder().build(
            summary_archive,
            summary_path="cycle-44/archive-summary-artifact-archive-summary.md",
            gate_path="cycle-44/archive-summary-artifact-archive-summary-gate.md",
        )
        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestBuilder().build(
            packaged,
        )
        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdown().render(
            manifest,
        )
        verifier = (
            ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerifier()
        )

        clean = verifier.verify(manifest, markdown)
        broken = verifier.verify(
            manifest,
            markdown.replace("- Artifact count: 2\n", "").replace(
                f"| `{manifest.entries[0].path}` | `{manifest.entries[0].content_sha256}` | {manifest.entries[0].byte_count} |\n",
                "",
            ),
        )

        self.assertTrue(clean.ok)
        self.assertFalse(broken.ok)
        self.assertEqual(
            [
                ("manifest.md", "artifact count line is missing"),
                (manifest.entries[0].path, "artifact row is missing"),
            ],
            [(finding.path, finding.message) for finding in broken.findings],
        )

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-markdown-verification-markdown-renders-findings
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_artifact_manifest_markdown_verification_markdown_renders_findings(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-45",
                    completed_runs=24,
                    improved_candidates=20,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.32,
                    remaining_budget=24.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=20,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=20.0,
            packet_path="cycle-45/planning-packet.md",
            plan_path="cycle-45/plan.md",
            lint_path="cycle-45/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-45/review-packet.md",
            readiness_path="cycle-45/readiness.md",
            manifest_path="cycle-45/manifest.md",
            verification_path="cycle-45/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-45/archive-summary.md",
            gate_path="cycle-45/archive-summary-gate.md",
        )
        summary_archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder().build(
            handoff,
        )
        packaged = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactBuilder().build(
            summary_archive,
            summary_path="cycle-45/archive-summary-artifact-archive-summary.md",
            gate_path="cycle-45/archive-summary-artifact-archive-summary-gate.md",
        )
        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestBuilder().build(
            packaged,
        )
        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdown().render(
            manifest,
        )
        verification = (
            ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerifier().verify(
                manifest,
                markdown.replace("| Path | SHA-256 | Bytes |\n", ""),
            )
        )

        rendered = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdown().render(
            verification,
            title="Cycle 45 Manifest Markdown Verification",
        )

        self.assertIn("# Cycle 45 Manifest Markdown Verification", rendered)
        self.assertIn("- Status: blocked", rendered)
        self.assertIn("- Finding count: 1", rendered)
        self.assertIn("- `manifest.md`: manifest table header is missing", rendered)
        self.assertNotIn("- none", rendered)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-markdown-verification-markdown-gate-blocks-drift
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_artifact_manifest_markdown_verification_markdown_gate_blocks_drift(
        self,
    ) -> None:
        report = ResearchCycleRetrospective().summarize(
            [
                ResearchCycleSignal(
                    cycle_id="cycle-46",
                    completed_runs=25,
                    improved_candidates=21,
                    regressed_candidates=0,
                    failed_runs=0,
                    blocked_items=0,
                    mean_cost=0.30,
                    remaining_budget=25.0,
                    high_frontier_drift=0,
                    evidence_ready_claims=21,
                )
            ]
        )
        bundle = ResearchCyclePlanningHandoffBundleBuilder().build(
            report,
            active_capacity=2,
            remaining_budget=21.0,
            packet_path="cycle-46/planning-packet.md",
            plan_path="cycle-46/plan.md",
            lint_path="cycle-46/lint.md",
        )
        packet = ResearchCyclePlanningHandoffReviewPacketBuilder().build(bundle)
        archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder().build(
            packet,
            review_path="cycle-46/review-packet.md",
            readiness_path="cycle-46/readiness.md",
            manifest_path="cycle-46/manifest.md",
            verification_path="cycle-46/verification.md",
        )
        handoff = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder().build(
            archive,
            summary_path="cycle-46/archive-summary.md",
            gate_path="cycle-46/archive-summary-gate.md",
        )
        summary_archive = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder().build(
            handoff,
        )
        packaged = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactBuilder().build(
            summary_archive,
            summary_path="cycle-46/archive-summary-artifact-archive-summary.md",
            gate_path="cycle-46/archive-summary-artifact-archive-summary-gate.md",
        )
        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestBuilder().build(
            packaged,
        )
        manifest_markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdown().render(
            manifest,
        )
        verification = (
            ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerifier().verify(
                manifest,
                manifest_markdown.replace("| Path | SHA-256 | Bytes |\n", ""),
            )
        )
        verification_markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdown().render(
            verification,
        )
        gate = (
            ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownGate()
        )

        clean = gate.evaluate(verification, verification_markdown)
        broken = gate.evaluate(
            verification,
            verification_markdown.replace("- Finding count: 1\n", "").replace(
                "- `manifest.md`: manifest table header is missing\n",
                "",
            ),
        )
        empty_verification = ResearchCyclePlanningPacketManifestVerification(findings=())
        empty_markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdown().render(
            empty_verification,
        )
        empty_clean = gate.evaluate(empty_verification, empty_markdown)

        self.assertTrue(clean["ready"])
        self.assertEqual("ready", clean["status"])
        self.assertEqual("blocked", clean["verification_status"])
        self.assertEqual(1, clean["finding_count"])
        self.assertFalse(broken["ready"])
        self.assertEqual(
            [
                "finding count line is missing",
                "finding line is missing for manifest.md",
            ],
            broken["blockers"],
        )
        self.assertTrue(empty_clean["ready"])
        self.assertEqual("ok", empty_clean["verification_status"])
        self.assertEqual(0, empty_clean["finding_count"])

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-markdown-verification-markdown-gate-markdown-renders-blockers
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_artifact_manifest_markdown_verification_markdown_gate_markdown_renders_blockers(
        self,
    ) -> None:
        gate_result = {
            "ready": False,
            "status": "blocked",
            "verification_status": "blocked",
            "finding_count": 2,
            "checked_finding_count": 2,
            "blockers": [
                "finding count line is missing",
                "finding line is missing for manifest.md",
            ],
        }

        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownGateMarkdown().render(
            gate_result,
            title="Cycle 47 Verification Markdown Gate",
        )
        clean = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownGateMarkdown().render(
            {
                "ready": True,
                "status": "ready",
                "verification_status": "ok",
                "finding_count": 0,
                "checked_finding_count": 0,
                "blockers": [],
            },
            title="Cycle 47 Verification Markdown Gate",
        )

        self.assertIn("# Cycle 47 Verification Markdown Gate", markdown)
        self.assertIn("- Ready: no", markdown)
        self.assertIn("- Status: blocked", markdown)
        self.assertIn("- Verification status: blocked", markdown)
        self.assertIn("- Finding count: 2", markdown)
        self.assertIn("- Checked finding count: 2", markdown)
        self.assertIn("- finding count line is missing", markdown)
        self.assertIn("- finding line is missing for manifest.md", markdown)
        self.assertIn("- Ready: yes", clean)
        self.assertIn("- Status: ready", clean)
        self.assertIn("- Verification status: ok", clean)
        self.assertIn("- none", clean)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-markdown-verification-markdown-artifact-builder-packages-gate-audit
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_artifact_manifest_markdown_verification_markdown_artifact_builder_packages_gate_audit(
        self,
    ) -> None:
        verification = ResearchCyclePlanningPacketManifestVerification(
            findings=(
                ResearchCyclePlanningPacketManifestVerificationFinding(
                    path="manifest.md",
                    message="manifest table header is missing",
                ),
            )
        )

        packaged = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactBuilder().build(
            verification,
            verification_path="cycle-48/manifest-markdown-verification.md",
            gate_path="cycle-48/manifest-markdown-verification-gate.md",
            verification_title="Cycle 48 Manifest Markdown Verification",
            gate_title="Cycle 48 Manifest Markdown Verification Gate",
        )

        self.assertEqual(
            (
                "cycle-48/manifest-markdown-verification.md",
                "cycle-48/manifest-markdown-verification-gate.md",
            ),
            tuple(artifact.path for artifact in packaged.artifacts),
        )
        self.assertIs(packaged.verification, verification)
        self.assertIn("# Cycle 48 Manifest Markdown Verification", packaged.verification_markdown)
        self.assertIn("- Status: blocked", packaged.verification_markdown)
        self.assertIn("- `manifest.md`: manifest table header is missing", packaged.verification_markdown)
        self.assertEqual(True, packaged.gate_result["ready"])
        self.assertEqual("ready", packaged.gate_result["status"])
        self.assertEqual("blocked", packaged.gate_result["verification_status"])
        self.assertIn("# Cycle 48 Manifest Markdown Verification Gate", packaged.gate_markdown)
        self.assertIn("- Ready: yes", packaged.gate_markdown)
        self.assertIn("- Verification status: blocked", packaged.gate_markdown)
        self.assertIn("- none", packaged.gate_markdown)
        self.assertEqual(packaged.verification_markdown, packaged.artifacts[0].content)
        self.assertEqual(packaged.gate_markdown, packaged.artifacts[1].content)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-markdown-verification-markdown-artifact-manifest-records-artifacts
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_artifact_manifest_markdown_verification_markdown_artifact_manifest_records_artifacts(
        self,
    ) -> None:
        verification = ResearchCyclePlanningPacketManifestVerification(
            findings=(
                ResearchCyclePlanningPacketManifestVerificationFinding(
                    path="manifest.md",
                    message="manifest table header is missing",
                ),
            )
        )
        packaged = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactBuilder().build(
            verification,
            verification_path="cycle-49/manifest-markdown-verification.md",
            gate_path="cycle-49/manifest-markdown-verification-gate.md",
        )

        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestBuilder().build(
            packaged,
            source_cycles=("cycle-49",),
        )

        self.assertEqual(("cycle-49",), manifest.source_cycles)
        self.assertEqual("ok", manifest.status)
        self.assertEqual(
            (
                "cycle-49/manifest-markdown-verification.md",
                "cycle-49/manifest-markdown-verification-gate.md",
            ),
            tuple(entry.path for entry in manifest.entries),
        )
        for artifact, entry in zip(packaged.artifacts, manifest.entries, strict=True):
            self.assertEqual(len(artifact.content.encode("utf-8")), entry.byte_count)
            self.assertTrue(entry.content_sha256.startswith("sha256:"))
            self.assertEqual(71, len(entry.content_sha256))

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-markdown-verification-markdown-artifact-manifest-markdown-renders-table
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_artifact_manifest_markdown_verification_markdown_artifact_manifest_markdown_renders_table(
        self,
    ) -> None:
        verification = ResearchCyclePlanningPacketManifestVerification(
            findings=(
                ResearchCyclePlanningPacketManifestVerificationFinding(
                    path="manifest.md",
                    message="manifest table header is missing",
                ),
            )
        )
        packaged = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactBuilder().build(
            verification,
            verification_path="cycle-50/manifest-markdown-verification.md",
            gate_path="cycle-50/manifest-markdown-verification-gate.md",
        )
        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestBuilder().build(
            packaged,
            source_cycles=("cycle-50",),
        )

        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestMarkdown().render(
            manifest,
            title="Cycle 50 Verification Markdown Artifact Manifest",
        )

        self.assertIn("# Cycle 50 Verification Markdown Artifact Manifest", markdown)
        self.assertIn("- Source cycles: cycle-50", markdown)
        self.assertIn("- Status: ok", markdown)
        self.assertIn("- Artifact count: 2", markdown)
        self.assertIn("| Path | SHA-256 | Bytes |", markdown)
        self.assertIn("| `cycle-50/manifest-markdown-verification.md` | `sha256:", markdown)
        self.assertIn("| `cycle-50/manifest-markdown-verification-gate.md` | `sha256:", markdown)

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-markdown-verification-markdown-artifact-manifest-markdown-verifier-detects-drift
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_artifact_manifest_markdown_verification_markdown_artifact_manifest_markdown_verifier_detects_drift(
        self,
    ) -> None:
        verification = ResearchCyclePlanningPacketManifestVerification(findings=())
        packaged = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactBuilder().build(
            verification,
            verification_path="cycle-51/manifest-markdown-verification.md",
            gate_path="cycle-51/manifest-markdown-verification-gate.md",
        )
        manifest = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestBuilder().build(
            packaged,
            source_cycles=("cycle-51",),
        )
        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestMarkdown().render(
            manifest,
            title="Cycle 51 Verification Markdown Artifact Manifest",
        )

        clean = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestMarkdownVerifier().verify(
            manifest,
            markdown,
        )
        broken = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestMarkdownVerifier().verify(
            manifest,
            markdown.replace("- Artifact count: 2\n", "").replace(
                "| `cycle-51/manifest-markdown-verification-gate.md` |",
                "| `cycle-51/missing.md` |",
            ),
        )

        self.assertTrue(clean.ok)
        self.assertEqual(
            (
                ("manifest.md", "artifact count line is missing"),
                ("cycle-51/manifest-markdown-verification-gate.md", "artifact row is missing"),
            ),
            tuple((finding.path, finding.message) for finding in broken.findings),
        )

    # cf-atom: TEST-research-cycle-planning-handoff-review-packet-artifact-archive-summary-artifact-archive-summary-artifact-manifest-markdown-verification-markdown-artifact-manifest-markdown-verification-markdown-renders-findings
    def test_research_cycle_planning_handoff_review_packet_artifact_archive_summary_artifact_archive_summary_artifact_manifest_markdown_verification_markdown_artifact_manifest_markdown_verification_markdown_renders_findings(
        self,
    ) -> None:
        verification = ResearchCyclePlanningPacketManifestVerification(
            findings=(
                ResearchCyclePlanningPacketManifestVerificationFinding(
                    path="cycle-52/manifest-markdown-verification-gate.md",
                    message="artifact row is missing",
                ),
            )
        )

        markdown = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestMarkdownVerificationMarkdown().render(
            verification,
            title="Cycle 52 Verification Markdown Artifact Manifest Markdown Verification",
        )
        clean = ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveSummaryArtifactManifestMarkdownVerificationMarkdownArtifactManifestMarkdownVerificationMarkdown().render(
            ResearchCyclePlanningPacketManifestVerification(findings=()),
            title="Cycle 52 Verification Markdown Artifact Manifest Markdown Verification",
        )

        self.assertIn("# Cycle 52 Verification Markdown Artifact Manifest Markdown Verification", markdown)
        self.assertIn("- Status: blocked", markdown)
        self.assertIn("- Finding count: 1", markdown)
        self.assertIn(
            "- `cycle-52/manifest-markdown-verification-gate.md`: artifact row is missing",
            markdown,
        )
        self.assertIn("- Status: ok", clean)
        self.assertIn("- Finding count: 0", clean)
        self.assertIn("- none", clean)


if __name__ == "__main__":
    unittest.main()
