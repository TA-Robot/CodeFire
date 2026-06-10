from dataclasses import replace
import unittest

from evoagent.retrospective import (
    ResearchCycleRetrospective,
    ResearchCyclePlanningHandoffBundleBuilder,
    ResearchCyclePlanningHandoffBundleMarkdown,
    ResearchCyclePlanningHandoffReadinessGate,
    ResearchCyclePlanningHandoffReadinessMarkdown,
    ResearchCyclePlanningHandoffReviewPacketArtifactBuilder,
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


if __name__ == "__main__":
    unittest.main()
