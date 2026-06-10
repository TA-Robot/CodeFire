from __future__ import annotations

import hashlib
from dataclasses import dataclass
from enum import Enum


class RetrospectiveRecommendation(str, Enum):
    INCREASE_EXPLORATION = "increase_exploration"
    CONSOLIDATE = "consolidate"
    REDUCE_COST = "reduce_cost"
    MITIGATE_RISK = "mitigate_risk"
    ARCHIVE_STALE = "archive_stale"
    KEEP_POLICY = "keep_policy"


class RetrospectivePriority(str, Enum):
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"


class ResearchCyclePlanLane(str, Enum):
    MITIGATION = "mitigation"
    ACTIVE = "active"
    REVIEW = "review"
    ARCHIVE = "archive"
    DEFERRED = "deferred"


class ResearchCyclePlanLintSeverity(str, Enum):
    BLOCKER = "blocker"
    WARNING = "warning"


@dataclass(frozen=True)
class ResearchCycleSignal:
    cycle_id: str
    completed_runs: int
    improved_candidates: int
    regressed_candidates: int
    failed_runs: int
    blocked_items: int
    mean_cost: float
    remaining_budget: float
    high_frontier_drift: int = 0
    evidence_ready_claims: int = 0
    notes: str = ""


@dataclass(frozen=True)
class ResearchCycleRetrospectiveReport:
    cycle_ids: tuple[str, ...]
    completed_runs: int
    improvement_rate: float
    regression_rate: float
    failure_rate: float
    average_cost: float
    budget_pressure: float
    risk_pressure: float
    recommendations: tuple[RetrospectiveRecommendation, ...]
    priority: RetrospectivePriority
    rationale: tuple[str, ...]


@dataclass(frozen=True)
class ResearchCyclePlanItem:
    lane: ResearchCyclePlanLane
    recommendation: RetrospectiveRecommendation
    title: str
    action: str
    rationale: str
    budget_hint: float


@dataclass(frozen=True)
class ResearchCyclePlan:
    source_cycles: tuple[str, ...]
    priority: RetrospectivePriority
    active_capacity: int
    remaining_budget: float
    items: tuple[ResearchCyclePlanItem, ...]

    def lane(self, lane: ResearchCyclePlanLane) -> tuple[ResearchCyclePlanItem, ...]:
        return tuple(item for item in self.items if item.lane == lane)


@dataclass(frozen=True)
class ResearchCyclePlanLintFinding:
    severity: ResearchCyclePlanLintSeverity
    field: str
    message: str


@dataclass(frozen=True)
class ResearchCyclePlanLintReport:
    findings: tuple[ResearchCyclePlanLintFinding, ...]

    @property
    def ok(self) -> bool:
        return not any(finding.severity == ResearchCyclePlanLintSeverity.BLOCKER for finding in self.findings)

    @property
    def blocker_count(self) -> int:
        return sum(1 for finding in self.findings if finding.severity == ResearchCyclePlanLintSeverity.BLOCKER)

    @property
    def warning_count(self) -> int:
        return sum(1 for finding in self.findings if finding.severity == ResearchCyclePlanLintSeverity.WARNING)


@dataclass(frozen=True)
class ResearchCyclePlanningPacket:
    plan: ResearchCyclePlan
    lint_report: ResearchCyclePlanLintReport
    plan_markdown: str
    lint_markdown: str


@dataclass(frozen=True)
class ResearchCyclePlanningPacketManifestEntry:
    path: str
    content_sha256: str
    byte_count: int


@dataclass(frozen=True)
class ResearchCyclePlanningPacketManifest:
    source_cycles: tuple[str, ...]
    status: str
    entries: tuple[ResearchCyclePlanningPacketManifestEntry, ...]


@dataclass(frozen=True)
class ResearchCyclePlanningPacketManifestVerificationFinding:
    path: str
    message: str


@dataclass(frozen=True)
class ResearchCyclePlanningPacketManifestVerification:
    findings: tuple[ResearchCyclePlanningPacketManifestVerificationFinding, ...]

    @property
    def ok(self) -> bool:
        return not self.findings


@dataclass(frozen=True)
class ResearchCyclePlanningHandoffArtifact:
    path: str
    content: str


@dataclass(frozen=True)
class ResearchCyclePlanningHandoffBundle:
    packet: ResearchCyclePlanningPacket
    artifacts: tuple[ResearchCyclePlanningHandoffArtifact, ...]
    manifest: ResearchCyclePlanningPacketManifest
    manifest_markdown: str
    verification: ResearchCyclePlanningPacketManifestVerification
    verification_markdown: str


# cf-atom: CODE-ResearchCycleRetrospective
class ResearchCycleRetrospective:
    def summarize(self, signals: list[ResearchCycleSignal]) -> ResearchCycleRetrospectiveReport:
        if not signals:
            raise ValueError("at least one cycle signal is required")
        for signal in signals:
            validate_signal(signal)

        completed_runs = sum(signal.completed_runs for signal in signals)
        improved = sum(signal.improved_candidates for signal in signals)
        regressed = sum(signal.regressed_candidates for signal in signals)
        failed = sum(signal.failed_runs for signal in signals)
        blocked = sum(signal.blocked_items for signal in signals)
        high_drift = sum(signal.high_frontier_drift for signal in signals)
        evidence_ready = sum(signal.evidence_ready_claims for signal in signals)
        total_cost = sum(signal.mean_cost * max(signal.completed_runs, 1) for signal in signals)
        cost_weight = sum(max(signal.completed_runs, 1) for signal in signals)

        improvement_rate = safe_rate(improved, completed_runs)
        regression_rate = safe_rate(regressed, completed_runs)
        failure_rate = safe_rate(failed, completed_runs)
        average_cost = total_cost / cost_weight
        budget_pressure = budget_pressure_for(signals, average_cost)
        risk_pressure = risk_pressure_for(blocked, high_drift, completed_runs)
        recommendations = recommendations_for(
            improvement_rate=improvement_rate,
            regression_rate=regression_rate,
            failure_rate=failure_rate,
            budget_pressure=budget_pressure,
            risk_pressure=risk_pressure,
            evidence_ready=evidence_ready,
            completed_runs=completed_runs,
        )
        priority = priority_for(
            recommendations=recommendations,
            budget_pressure=budget_pressure,
            risk_pressure=risk_pressure,
            failure_rate=failure_rate,
        )
        return ResearchCycleRetrospectiveReport(
            cycle_ids=tuple(signal.cycle_id for signal in signals),
            completed_runs=completed_runs,
            improvement_rate=improvement_rate,
            regression_rate=regression_rate,
            failure_rate=failure_rate,
            average_cost=average_cost,
            budget_pressure=budget_pressure,
            risk_pressure=risk_pressure,
            recommendations=recommendations,
            priority=priority,
            rationale=rationale_for(recommendations, improvement_rate, failure_rate, budget_pressure, risk_pressure),
        )


def validate_signal(signal: ResearchCycleSignal) -> None:
    if not signal.cycle_id:
        raise ValueError("cycle_id is required")
    counts = (
        signal.completed_runs,
        signal.improved_candidates,
        signal.regressed_candidates,
        signal.failed_runs,
        signal.blocked_items,
        signal.high_frontier_drift,
        signal.evidence_ready_claims,
    )
    if any(value < 0 for value in counts):
        raise ValueError("cycle counts must be non-negative")
    if signal.mean_cost < 0 or signal.remaining_budget < 0:
        raise ValueError("cost and budget must be non-negative")
    if signal.improved_candidates + signal.regressed_candidates > max(signal.completed_runs, 1) + signal.failed_runs:
        raise ValueError("candidate outcome counts exceed observed cycle activity")


def safe_rate(count: int, total: int) -> float:
    if total <= 0:
        return 0.0
    return count / total


def budget_pressure_for(signals: list[ResearchCycleSignal], average_cost: float) -> float:
    remaining_budget = min(signal.remaining_budget for signal in signals)
    if average_cost == 0:
        return 0.0
    return clamp01(1.0 - (remaining_budget / (average_cost * 2.0)))


def risk_pressure_for(blocked: int, high_drift: int, completed_runs: int) -> float:
    denominator = max(completed_runs, 1)
    return clamp01((blocked + high_drift) / denominator)


def recommendations_for(
    *,
    improvement_rate: float,
    regression_rate: float,
    failure_rate: float,
    budget_pressure: float,
    risk_pressure: float,
    evidence_ready: int,
    completed_runs: int,
) -> tuple[RetrospectiveRecommendation, ...]:
    recommendations: list[RetrospectiveRecommendation] = []
    append_if(recommendations, risk_pressure >= 0.4, RetrospectiveRecommendation.MITIGATE_RISK)
    append_if(recommendations, budget_pressure >= 0.5, RetrospectiveRecommendation.REDUCE_COST)
    append_if(
        recommendations,
        improvement_rate >= 0.35 and evidence_ready > 0,
        RetrospectiveRecommendation.CONSOLIDATE,
    )
    append_if(
        recommendations,
        completed_runs >= 3 and improvement_rate < 0.2 and failure_rate < 0.35,
        RetrospectiveRecommendation.INCREASE_EXPLORATION,
    )
    append_if(
        recommendations,
        regression_rate >= 0.25 or failure_rate >= 0.5,
        RetrospectiveRecommendation.ARCHIVE_STALE,
    )
    if not recommendations:
        recommendations.append(RetrospectiveRecommendation.KEEP_POLICY)
    return tuple(recommendations)


def append_if(items: list[RetrospectiveRecommendation], condition: bool, item: RetrospectiveRecommendation) -> None:
    if condition and item not in items:
        items.append(item)


def priority_for(
    *,
    recommendations: tuple[RetrospectiveRecommendation, ...],
    budget_pressure: float,
    risk_pressure: float,
    failure_rate: float,
) -> RetrospectivePriority:
    if (
        RetrospectiveRecommendation.MITIGATE_RISK in recommendations
        and risk_pressure >= 0.6
        or RetrospectiveRecommendation.ARCHIVE_STALE in recommendations
        and failure_rate >= 0.6
    ):
        return RetrospectivePriority.HIGH
    if budget_pressure >= 0.5 or len(recommendations) >= 2:
        return RetrospectivePriority.MEDIUM
    return RetrospectivePriority.LOW


def rationale_for(
    recommendations: tuple[RetrospectiveRecommendation, ...],
    improvement_rate: float,
    failure_rate: float,
    budget_pressure: float,
    risk_pressure: float,
) -> tuple[str, ...]:
    lines: list[str] = []
    if RetrospectiveRecommendation.MITIGATE_RISK in recommendations:
        lines.append(f"risk pressure is {risk_pressure:.2f}; resolve blockers before expanding execution")
    if RetrospectiveRecommendation.REDUCE_COST in recommendations:
        lines.append(f"budget pressure is {budget_pressure:.2f}; prefer cheaper probes")
    if RetrospectiveRecommendation.CONSOLIDATE in recommendations:
        lines.append(f"improvement rate is {improvement_rate:.2f}; consolidate evidence-ready candidates")
    if RetrospectiveRecommendation.INCREASE_EXPLORATION in recommendations:
        lines.append("low improvement with manageable failure rate; increase exploration diversity")
    if RetrospectiveRecommendation.ARCHIVE_STALE in recommendations:
        lines.append(f"failure rate is {failure_rate:.2f}; archive stale or repeatedly failing paths")
    if RetrospectiveRecommendation.KEEP_POLICY in recommendations:
        lines.append("no dominant pressure detected; keep the current planning policy")
    return tuple(lines)


def clamp01(value: float) -> float:
    return min(1.0, max(0.0, value))


# cf-atom: CODE-RetrospectivePlanningSummary
class RetrospectivePlanningSummary:
    def render_markdown(
        self,
        report: ResearchCycleRetrospectiveReport,
        *,
        title: str = "Research Cycle Retrospective",
    ) -> str:
        lines = [
            f"# {title}",
            "",
            f"- Cycles: {', '.join(report.cycle_ids)}",
            f"- Priority: {report.priority.value}",
            f"- Completed runs: {report.completed_runs}",
            f"- Improvement rate: {report.improvement_rate:.2f}",
            f"- Regression rate: {report.regression_rate:.2f}",
            f"- Failure rate: {report.failure_rate:.2f}",
            f"- Average cost: {report.average_cost:.2f}",
            f"- Budget pressure: {report.budget_pressure:.2f}",
            f"- Risk pressure: {report.risk_pressure:.2f}",
            "",
            "## Recommendations",
            "",
        ]
        lines.extend(f"- {recommendation.value}" for recommendation in report.recommendations)
        lines.extend(["", "## Rationale", ""])
        lines.extend(f"- {reason}" for reason in report.rationale)
        return "\n".join(lines).rstrip() + "\n"


# cf-atom: CODE-ResearchCyclePlanSynthesizer
class ResearchCyclePlanSynthesizer:
    def synthesize(
        self,
        report: ResearchCycleRetrospectiveReport,
        *,
        active_capacity: int,
        remaining_budget: float,
    ) -> ResearchCyclePlan:
        if active_capacity <= 0:
            raise ValueError("active_capacity must be positive")
        if remaining_budget < 0:
            raise ValueError("remaining_budget must be non-negative")

        items = [plan_item_for(recommendation, report, remaining_budget) for recommendation in report.recommendations]
        items.sort(key=plan_item_sort_key)
        return ResearchCyclePlan(
            source_cycles=report.cycle_ids,
            priority=report.priority,
            active_capacity=active_capacity,
            remaining_budget=remaining_budget,
            items=tuple(bound_active_items(items, active_capacity)),
        )


# cf-atom: CODE-ResearchCyclePlanMarkdown
class ResearchCyclePlanMarkdown:
    def render(
        self,
        plan: ResearchCyclePlan,
        *,
        title: str = "Next Research Cycle Plan",
    ) -> str:
        lines = [
            f"# {title}",
            "",
            f"- Source cycles: {', '.join(plan.source_cycles)}",
            f"- Priority: {plan.priority.value}",
            f"- Active capacity: {plan.active_capacity}",
            f"- Remaining budget: {plan.remaining_budget:.2f}",
            "",
        ]
        for lane in ResearchCyclePlanLane:
            lane_items = plan.lane(lane)
            if not lane_items:
                continue
            lines.extend([f"## {lane.value.title()}", ""])
            for item in lane_items:
                lines.extend(
                    [
                        f"### {item.title}",
                        "",
                        f"- Recommendation: {item.recommendation.value}",
                        f"- Action: {item.action}",
                        f"- Budget hint: {item.budget_hint:.2f}",
                        f"- Rationale: {item.rationale}",
                        "",
                    ]
                )
        return "\n".join(lines).rstrip() + "\n"


# cf-atom: CODE-ResearchCyclePlanLint
class ResearchCyclePlanLint:
    def lint(self, plan: ResearchCyclePlan) -> ResearchCyclePlanLintReport:
        findings: list[ResearchCyclePlanLintFinding] = []
        add_if(findings, not plan.source_cycles, ResearchCyclePlanLintSeverity.BLOCKER, "source_cycles", "at least one source cycle is required")
        add_if(
            findings,
            any(not cycle_id for cycle_id in plan.source_cycles),
            ResearchCyclePlanLintSeverity.BLOCKER,
            "source_cycles",
            "source cycle IDs must be non-empty",
        )
        add_if(
            findings,
            plan.active_capacity <= 0,
            ResearchCyclePlanLintSeverity.BLOCKER,
            "active_capacity",
            "active capacity must be positive",
        )
        add_if(
            findings,
            plan.remaining_budget < 0,
            ResearchCyclePlanLintSeverity.BLOCKER,
            "remaining_budget",
            "remaining budget must be non-negative",
        )
        add_if(findings, not plan.items, ResearchCyclePlanLintSeverity.WARNING, "items", "plan has no lane items")

        active_items = plan.lane(ResearchCyclePlanLane.ACTIVE)
        add_if(
            findings,
            len(active_items) > max(plan.active_capacity, 0),
            ResearchCyclePlanLintSeverity.BLOCKER,
            "items",
            "active lane exceeds active capacity",
        )
        active_budget = sum(item.budget_hint for item in active_items)
        add_if(
            findings,
            active_budget > plan.remaining_budget,
            ResearchCyclePlanLintSeverity.BLOCKER,
            "items.budget_hint",
            "active budget hints exceed remaining budget",
        )

        seen_recommendations: set[RetrospectiveRecommendation] = set()
        for index, item in enumerate(plan.items):
            prefix = f"items[{index}]"
            add_if(findings, not item.title, ResearchCyclePlanLintSeverity.BLOCKER, f"{prefix}.title", "title is required")
            add_if(findings, not item.action, ResearchCyclePlanLintSeverity.BLOCKER, f"{prefix}.action", "action is required")
            add_if(
                findings,
                not item.rationale,
                ResearchCyclePlanLintSeverity.WARNING,
                f"{prefix}.rationale",
                "rationale is empty",
            )
            add_if(
                findings,
                item.budget_hint < 0,
                ResearchCyclePlanLintSeverity.BLOCKER,
                f"{prefix}.budget_hint",
                "budget hint must be non-negative",
            )
            add_if(
                findings,
                item.recommendation in seen_recommendations,
                ResearchCyclePlanLintSeverity.WARNING,
                f"{prefix}.recommendation",
                "recommendation appears more than once",
            )
            seen_recommendations.add(item.recommendation)

        return ResearchCyclePlanLintReport(findings=tuple(sorted(findings, key=lint_finding_sort_key)))


# cf-atom: CODE-ResearchCyclePlanLintMarkdown
class ResearchCyclePlanLintMarkdown:
    def render(
        self,
        report: ResearchCyclePlanLintReport,
        *,
        title: str = "Research Cycle Plan Lint",
    ) -> str:
        lines = [
            f"# {title}",
            "",
            f"- Status: {'ok' if report.ok else 'blocked'}",
            f"- Blockers: {report.blocker_count}",
            f"- Warnings: {report.warning_count}",
            "",
        ]
        for severity in ResearchCyclePlanLintSeverity:
            severity_findings = tuple(finding for finding in report.findings if finding.severity == severity)
            if not severity_findings:
                continue
            lines.extend([f"## {severity.value.title()}s", ""])
            lines.extend(f"- `{finding.field}`: {finding.message}" for finding in severity_findings)
            lines.append("")
        if not report.findings:
            lines.extend(["## Findings", "", "- none", ""])
        return "\n".join(lines).rstrip() + "\n"


# cf-atom: CODE-ResearchCyclePlanningPacketBuilder
class ResearchCyclePlanningPacketBuilder:
    def build(
        self,
        report: ResearchCycleRetrospectiveReport,
        *,
        active_capacity: int,
        remaining_budget: float,
        plan_title: str = "Next Research Cycle Plan",
        lint_title: str = "Research Cycle Plan Lint",
    ) -> ResearchCyclePlanningPacket:
        plan = ResearchCyclePlanSynthesizer().synthesize(
            report,
            active_capacity=active_capacity,
            remaining_budget=remaining_budget,
        )
        lint_report = ResearchCyclePlanLint().lint(plan)
        return ResearchCyclePlanningPacket(
            plan=plan,
            lint_report=lint_report,
            plan_markdown=ResearchCyclePlanMarkdown().render(plan, title=plan_title),
            lint_markdown=ResearchCyclePlanLintMarkdown().render(lint_report, title=lint_title),
        )


# cf-atom: CODE-ResearchCyclePlanningPacketMarkdown
class ResearchCyclePlanningPacketMarkdown:
    def render(
        self,
        packet: ResearchCyclePlanningPacket,
        *,
        title: str = "Research Cycle Planning Packet",
    ) -> str:
        lines = [
            f"# {title}",
            "",
            f"- Source cycles: {', '.join(packet.plan.source_cycles)}",
            f"- Priority: {packet.plan.priority.value}",
            f"- Status: {'ok' if packet.lint_report.ok else 'blocked'}",
            f"- Active items: {len(packet.plan.lane(ResearchCyclePlanLane.ACTIVE))}",
            f"- Remaining budget: {packet.plan.remaining_budget:.2f}",
            f"- Blockers: {packet.lint_report.blocker_count}",
            f"- Warnings: {packet.lint_report.warning_count}",
            "",
            "## Plan",
            "",
            demote_markdown_headings(packet.plan_markdown).rstrip(),
            "",
            "## Lint",
            "",
            demote_markdown_headings(packet.lint_markdown).rstrip(),
        ]
        return "\n".join(lines).rstrip() + "\n"


# cf-atom: CODE-ResearchCyclePlanningPacketManifestBuilder
class ResearchCyclePlanningPacketManifestBuilder:
    def build(
        self,
        packet: ResearchCyclePlanningPacket,
        *,
        packet_path: str = "planning-packet.md",
        plan_path: str = "plan.md",
        lint_path: str = "lint.md",
        packet_title: str = "Research Cycle Planning Packet",
    ) -> ResearchCyclePlanningPacketManifest:
        packet_markdown = ResearchCyclePlanningPacketMarkdown().render(packet, title=packet_title)
        entries = (
            manifest_entry(packet_path, packet_markdown),
            manifest_entry(plan_path, packet.plan_markdown),
            manifest_entry(lint_path, packet.lint_markdown),
        )
        return ResearchCyclePlanningPacketManifest(
            source_cycles=packet.plan.source_cycles,
            status="ok" if packet.lint_report.ok else "blocked",
            entries=entries,
        )


# cf-atom: CODE-ResearchCyclePlanningPacketManifestMarkdown
class ResearchCyclePlanningPacketManifestMarkdown:
    def render(
        self,
        manifest: ResearchCyclePlanningPacketManifest,
        *,
        title: str = "Research Cycle Planning Packet Manifest",
    ) -> str:
        lines = [
            f"# {title}",
            "",
            f"- Source cycles: {', '.join(manifest.source_cycles)}",
            f"- Status: {manifest.status}",
            f"- Artifact count: {len(manifest.entries)}",
            "",
            "| Path | SHA-256 | Bytes |",
            "|---|---|---:|",
        ]
        for entry in manifest.entries:
            lines.append(f"| `{entry.path}` | `{entry.content_sha256}` | {entry.byte_count} |")
        return "\n".join(lines).rstrip() + "\n"


# cf-atom: CODE-ResearchCyclePlanningPacketManifestVerifier
class ResearchCyclePlanningPacketManifestVerifier:
    def verify(
        self,
        manifest: ResearchCyclePlanningPacketManifest,
        artifact_contents: dict[str, str],
    ) -> ResearchCyclePlanningPacketManifestVerification:
        findings: list[ResearchCyclePlanningPacketManifestVerificationFinding] = []
        for entry in manifest.entries:
            content = artifact_contents.get(entry.path)
            if content is None:
                findings.append(
                    ResearchCyclePlanningPacketManifestVerificationFinding(
                        path=entry.path,
                        message="artifact is missing",
                    )
                )
                continue
            payload = content.encode("utf-8")
            content_sha256 = f"sha256:{hashlib.sha256(payload).hexdigest()}"
            if content_sha256 != entry.content_sha256:
                findings.append(
                    ResearchCyclePlanningPacketManifestVerificationFinding(
                        path=entry.path,
                        message="sha256 digest mismatch",
                    )
                )
            if len(payload) != entry.byte_count:
                findings.append(
                    ResearchCyclePlanningPacketManifestVerificationFinding(
                        path=entry.path,
                        message="byte count mismatch",
                    )
                )
        return ResearchCyclePlanningPacketManifestVerification(findings=tuple(findings))


# cf-atom: CODE-ResearchCyclePlanningPacketManifestVerificationMarkdown
class ResearchCyclePlanningPacketManifestVerificationMarkdown:
    def render(
        self,
        verification: ResearchCyclePlanningPacketManifestVerification,
        *,
        title: str = "Research Cycle Planning Packet Manifest Verification",
    ) -> str:
        lines = [
            f"# {title}",
            "",
            f"- Status: {'ok' if verification.ok else 'blocked'}",
            f"- Finding count: {len(verification.findings)}",
            "",
            "## Findings",
            "",
        ]
        if not verification.findings:
            lines.append("- none")
        else:
            for finding in verification.findings:
                lines.append(f"- `{finding.path}`: {finding.message}")
        return "\n".join(lines).rstrip() + "\n"


# cf-atom: CODE-ResearchCyclePlanningHandoffBundleBuilder
class ResearchCyclePlanningHandoffBundleBuilder:
    def build(
        self,
        report: ResearchCycleRetrospectiveReport,
        *,
        active_capacity: int,
        remaining_budget: float,
        packet_path: str = "planning-packet.md",
        plan_path: str = "plan.md",
        lint_path: str = "lint.md",
        plan_title: str = "Research Cycle Plan",
        lint_title: str = "Research Cycle Plan Lint",
        packet_title: str = "Research Cycle Planning Packet",
        manifest_title: str = "Research Cycle Planning Packet Manifest",
        verification_title: str = "Research Cycle Planning Packet Manifest Verification",
    ) -> ResearchCyclePlanningHandoffBundle:
        packet = ResearchCyclePlanningPacketBuilder().build(
            report,
            active_capacity=active_capacity,
            remaining_budget=remaining_budget,
            plan_title=plan_title,
            lint_title=lint_title,
        )
        packet_markdown = ResearchCyclePlanningPacketMarkdown().render(packet, title=packet_title)
        artifacts = (
            ResearchCyclePlanningHandoffArtifact(packet_path, packet_markdown),
            ResearchCyclePlanningHandoffArtifact(plan_path, packet.plan_markdown),
            ResearchCyclePlanningHandoffArtifact(lint_path, packet.lint_markdown),
        )
        manifest = ResearchCyclePlanningPacketManifestBuilder().build(
            packet,
            packet_path=packet_path,
            plan_path=plan_path,
            lint_path=lint_path,
            packet_title=packet_title,
        )
        artifact_contents = {artifact.path: artifact.content for artifact in artifacts}
        verification = ResearchCyclePlanningPacketManifestVerifier().verify(manifest, artifact_contents)
        return ResearchCyclePlanningHandoffBundle(
            packet=packet,
            artifacts=artifacts,
            manifest=manifest,
            manifest_markdown=ResearchCyclePlanningPacketManifestMarkdown().render(
                manifest,
                title=manifest_title,
            ),
            verification=verification,
            verification_markdown=ResearchCyclePlanningPacketManifestVerificationMarkdown().render(
                verification,
                title=verification_title,
            ),
        )


# cf-atom: CODE-ResearchCyclePlanningHandoffBundleMarkdown
class ResearchCyclePlanningHandoffBundleMarkdown:
    def render(
        self,
        bundle: ResearchCyclePlanningHandoffBundle,
        *,
        title: str = "Research Cycle Planning Handoff Bundle",
    ) -> str:
        lines = [
            f"# {title}",
            "",
            f"- Source cycles: {', '.join(bundle.packet.plan.source_cycles)}",
            f"- Packet status: {'ok' if bundle.packet.lint_report.ok else 'blocked'}",
            f"- Manifest status: {bundle.manifest.status}",
            f"- Verification status: {'ok' if bundle.verification.ok else 'blocked'}",
            f"- Artifact count: {len(bundle.artifacts)}",
            "",
            "## Artifacts",
            "",
            "| Path | Bytes |",
            "|---|---:|",
        ]
        for artifact in bundle.artifacts:
            lines.append(f"| `{artifact.path}` | {len(artifact.content.encode('utf-8'))} |")
        lines.extend(
            [
                "",
                "## Audits",
                "",
                "- Manifest Markdown: included",
                "- Verification Markdown: included",
            ]
        )
        return "\n".join(lines).rstrip() + "\n"


def manifest_entry(path: str, content: str) -> ResearchCyclePlanningPacketManifestEntry:
    validate_manifest_path(path)
    payload = content.encode("utf-8")
    return ResearchCyclePlanningPacketManifestEntry(
        path=path,
        content_sha256=f"sha256:{hashlib.sha256(payload).hexdigest()}",
        byte_count=len(payload),
    )


def validate_manifest_path(path: str) -> None:
    if not path or path.startswith("/") or "\\" in path:
        raise ValueError("manifest paths must be relative POSIX paths")
    if any(part in {"", ".", ".."} for part in path.split("/")):
        raise ValueError("manifest paths must not contain empty, current, or parent segments")


def demote_markdown_headings(markdown: str) -> str:
    lines: list[str] = []
    for line in markdown.rstrip().splitlines():
        heading_level = len(line) - len(line.lstrip("#"))
        if heading_level > 0 and heading_level < len(line) and line[heading_level] == " ":
            lines.append(f"#{line}")
            continue
        lines.append(line)
    return "\n".join(lines).rstrip() + "\n"


def add_if(
    findings: list[ResearchCyclePlanLintFinding],
    condition: bool,
    severity: ResearchCyclePlanLintSeverity,
    field: str,
    message: str,
) -> None:
    if condition:
        findings.append(ResearchCyclePlanLintFinding(severity=severity, field=field, message=message))


def lint_finding_sort_key(finding: ResearchCyclePlanLintFinding) -> tuple[int, str, str]:
    severity_rank = {ResearchCyclePlanLintSeverity.BLOCKER: 0, ResearchCyclePlanLintSeverity.WARNING: 1}
    return (severity_rank[finding.severity], finding.field, finding.message)


def plan_item_for(
    recommendation: RetrospectiveRecommendation,
    report: ResearchCycleRetrospectiveReport,
    remaining_budget: float,
) -> ResearchCyclePlanItem:
    rationale = rationale_for_recommendation(recommendation, report)
    if recommendation == RetrospectiveRecommendation.MITIGATE_RISK:
        return ResearchCyclePlanItem(
            lane=ResearchCyclePlanLane.MITIGATION,
            recommendation=recommendation,
            title="Resolve blocking risks before expanding execution",
            action="build or update the mitigation checklist for the highest-risk campaign items",
            rationale=rationale,
            budget_hint=0.0,
        )
    if recommendation == RetrospectiveRecommendation.REDUCE_COST:
        return ResearchCyclePlanItem(
            lane=ResearchCyclePlanLane.ACTIVE,
            recommendation=recommendation,
            title="Run cheaper information-gain probes",
            action="draft bounded low-cost experiments and defer expensive replications",
            rationale=rationale,
            budget_hint=min(remaining_budget, max(report.average_cost * 0.5, 0.0)),
        )
    if recommendation == RetrospectiveRecommendation.CONSOLIDATE:
        return ResearchCyclePlanItem(
            lane=ResearchCyclePlanLane.REVIEW,
            recommendation=recommendation,
            title="Consolidate evidence-ready candidates",
            action="prepare evidence packs, review queue items, and release gate inputs for promising candidates",
            rationale=rationale,
            budget_hint=0.0,
        )
    if recommendation == RetrospectiveRecommendation.INCREASE_EXPLORATION:
        return ResearchCyclePlanItem(
            lane=ResearchCyclePlanLane.ACTIVE,
            recommendation=recommendation,
            title="Increase exploration diversity",
            action="sample frontier mechanisms that are not covered by recent negative results",
            rationale=rationale,
            budget_hint=min(remaining_budget, max(report.average_cost, 0.0)),
        )
    if recommendation == RetrospectiveRecommendation.ARCHIVE_STALE:
        return ResearchCyclePlanItem(
            lane=ResearchCyclePlanLane.ARCHIVE,
            recommendation=recommendation,
            title="Archive stale or repeatedly failing paths",
            action="write negative-result records and remove stale paths from the active queue",
            rationale=rationale,
            budget_hint=0.0,
        )
    return ResearchCyclePlanItem(
        lane=ResearchCyclePlanLane.DEFERRED,
        recommendation=recommendation,
        title="Keep current planning policy",
        action="carry forward the current queue and monitor for stronger signals",
        rationale=rationale,
        budget_hint=0.0,
    )


def rationale_for_recommendation(
    recommendation: RetrospectiveRecommendation,
    report: ResearchCycleRetrospectiveReport,
) -> str:
    for reason in report.rationale:
        if recommendation == RetrospectiveRecommendation.MITIGATE_RISK and "risk pressure" in reason:
            return reason
        if recommendation == RetrospectiveRecommendation.REDUCE_COST and "budget pressure" in reason:
            return reason
        if recommendation == RetrospectiveRecommendation.CONSOLIDATE and "improvement rate" in reason:
            return reason
        if recommendation == RetrospectiveRecommendation.INCREASE_EXPLORATION and "increase exploration" in reason:
            return reason
        if recommendation == RetrospectiveRecommendation.ARCHIVE_STALE and "failure rate" in reason:
            return reason
        if recommendation == RetrospectiveRecommendation.KEEP_POLICY and "keep the current" in reason:
            return reason
    return "recommendation came from the retrospective summary"


def plan_item_sort_key(item: ResearchCyclePlanItem) -> tuple[int, str]:
    lane_rank = {
        ResearchCyclePlanLane.MITIGATION: 0,
        ResearchCyclePlanLane.ACTIVE: 1,
        ResearchCyclePlanLane.REVIEW: 2,
        ResearchCyclePlanLane.ARCHIVE: 3,
        ResearchCyclePlanLane.DEFERRED: 4,
    }
    return (lane_rank[item.lane], item.recommendation.value)


def bound_active_items(
    items: list[ResearchCyclePlanItem],
    active_capacity: int,
) -> list[ResearchCyclePlanItem]:
    active_count = 0
    bounded: list[ResearchCyclePlanItem] = []
    for item in items:
        if item.lane != ResearchCyclePlanLane.ACTIVE:
            bounded.append(item)
            continue
        active_count += 1
        if active_count <= active_capacity:
            bounded.append(item)
            continue
        bounded.append(
            ResearchCyclePlanItem(
                lane=ResearchCyclePlanLane.DEFERRED,
                recommendation=item.recommendation,
                title=item.title,
                action=item.action,
                rationale=f"deferred because active capacity is {active_capacity}",
                budget_hint=0.0,
            )
        )
    return bounded
