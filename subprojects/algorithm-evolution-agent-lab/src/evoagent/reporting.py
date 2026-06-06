from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from evoagent.models import CandidateAlgorithm, ExperimentRun
from evoagent.program import ResearchProgram
from evoagent.scoring import score_candidate


@dataclass(frozen=True)
class ReportOptions:
    title: str = "Living Research Report"
    max_recommendations: int = 3


# cf-atom: CODE-LivingResearchReport
class LivingResearchReport:
    def __init__(self, options: ReportOptions | None = None):
        self.options = options or ReportOptions()

    def render(self, program: ResearchProgram) -> str:
        lines = [
            f"# {self.options.title}",
            "",
            "## Objective",
            "",
            program.goal.objective,
            "",
            "## Current Best Candidates",
            "",
            *self._candidate_table(program.candidates),
            "",
            "## Benchmark Status",
            "",
            *self._run_table(program.runs),
            "",
            "## Evidence Ledger",
            "",
            *self._evidence_table(program),
            "",
            "## Rejected Paths",
            "",
            *self._rejected_paths(program.candidates),
            "",
            "## Unresolved Risks",
            "",
            *self._risks(program),
            "",
            "## Recommended Next Experiments",
            "",
            *self._recommendations(program.candidates),
            "",
            "## Decision History",
            "",
            *self._decision_table(program),
            "",
        ]
        return "\n".join(lines)

    def write(self, program: ResearchProgram, path: Path) -> Path:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(self.render(program), encoding="utf-8")
        return path

    def _candidate_table(self, candidates: list[CandidateAlgorithm]) -> list[str]:
        if not candidates:
            return ["No candidates recorded yet."]
        lines = ["| Candidate | Hypothesis | Score | Status |", "|---|---|---:|---|"]
        for candidate in sorted(candidates, key=score_candidate, reverse=True):
            status = "rejected" if "rejected" in candidate.tags else "active"
            lines.append(
                f"| {escape_cell(candidate.name)} | {escape_cell(candidate.hypothesis.title)} | "
                f"{score_candidate(candidate):.4f} | {status} |"
            )
        return lines

    def _run_table(self, runs: list[ExperimentRun]) -> list[str]:
        if not runs:
            return ["No experiment runs recorded yet."]
        lines = ["| Run | Benchmark | Metric | Baseline | Status | Artifacts |", "|---|---|---|---|---|---:|"]
        for run in runs:
            status = "passed" if run.succeeded else "failed"
            lines.append(
                f"| {escape_cell(run.run_id)} | {escape_cell(run.plan.benchmark)} | "
                f"{escape_cell(run.plan.metric)} | {escape_cell(run.plan.baseline)} | "
                f"{status} | {len(run.artifacts)} |"
            )
        return lines

    def _evidence_table(self, program: ResearchProgram) -> list[str]:
        records = program.evidence.all()
        if not records:
            return ["No evidence records linked yet."]
        lines = ["| Evidence | Claim | Source | Confidence | Summary |", "|---|---|---|---:|---|"]
        for record in records:
            source = f"{record.source_type}:{record.source_ref}"
            lines.append(
                f"| {escape_cell(record.evidence_id)} | {escape_cell(record.claim_id)} | "
                f"{escape_cell(source)} | {record.confidence:.2f} | {escape_cell(record.summary)} |"
            )
        return lines

    def _rejected_paths(self, candidates: list[CandidateAlgorithm]) -> list[str]:
        rejected = [candidate for candidate in candidates if "rejected" in candidate.tags]
        if not rejected:
            return ["No rejected paths recorded yet."]
        return [f"- {candidate.name}: {candidate.hypothesis.title}" for candidate in rejected]

    def _risks(self, program: ResearchProgram) -> list[str]:
        risks: list[str] = []
        if not program.runs:
            risks.append("- No experiment runs have been recorded.")
        failed = [run for run in program.runs if not run.succeeded]
        if failed:
            risks.append(f"- {len(failed)} experiment run(s) failed and need failure analysis.")
        weak_claims = [
            record.claim_id
            for record in program.evidence.all()
            if record.confidence < 0.5
        ]
        if weak_claims:
            unique_claims = ", ".join(sorted(set(weak_claims)))
            risks.append(f"- Low-confidence evidence remains for: {unique_claims}.")
        if not risks:
            risks.append("- No unresolved risks recorded by the current report rules.")
        return risks

    def _recommendations(self, candidates: list[CandidateAlgorithm]) -> list[str]:
        active = [candidate for candidate in candidates if "rejected" not in candidate.tags]
        if not active:
            return ["No active candidates available for scheduling."]
        ranked = sorted(active, key=score_candidate, reverse=True)[: self.options.max_recommendations]
        return [
            f"- Run `{candidate.plan.command or 'configured runner'}` for {candidate.name} on {candidate.plan.benchmark}."
            for candidate in ranked
        ]

    def _decision_table(self, program: ResearchProgram) -> list[str]:
        if not program.decisions:
            return ["No decisions recorded yet."]
        lines = ["| Decision | Summary | Evidence |", "|---|---|---|"]
        for decision in program.decisions:
            evidence = ", ".join(decision.evidence_refs) if decision.evidence_refs else "-"
            lines.append(
                f"| {escape_cell(decision.decision_id)} | {escape_cell(decision.summary)} | {escape_cell(evidence)} |"
            )
        return lines


def escape_cell(value: object) -> str:
    return str(value).replace("|", "\\|").replace("\n", " ")
