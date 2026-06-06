from __future__ import annotations

from dataclasses import dataclass, field

from evoagent.evidence import EvidenceLedger
from evoagent.models import CandidateAlgorithm, ExperimentGoal, ExperimentRun, Hypothesis, HypothesisLink


@dataclass(frozen=True)
class DecisionRecord:
    decision_id: str
    summary: str
    rationale: str
    evidence_refs: tuple[str, ...] = field(default_factory=tuple)
    rejected_alternatives: tuple[str, ...] = field(default_factory=tuple)
    follow_up_work: tuple[str, ...] = field(default_factory=tuple)


@dataclass(frozen=True)
class DecisionRecordFinding:
    kind: str
    detail: str


# cf-atom: CODE-DecisionRecordValidator
class DecisionRecordValidator:
    def validate(self, decision: DecisionRecord) -> tuple[DecisionRecordFinding, ...]:
        findings: list[DecisionRecordFinding] = []
        if not decision.summary.strip():
            findings.append(DecisionRecordFinding("missing_summary", "decision summary is required"))
        if not decision.rationale.strip():
            findings.append(DecisionRecordFinding("missing_rationale", "decision rationale is required"))
        if not decision.evidence_refs:
            findings.append(DecisionRecordFinding("missing_evidence", "decision should reference evidence"))
        if not decision.rejected_alternatives:
            findings.append(
                DecisionRecordFinding("missing_rejected_alternatives", "decision should record rejected alternatives")
            )
        if not decision.follow_up_work:
            findings.append(DecisionRecordFinding("missing_follow_up", "decision should record expected follow-up work"))
        return tuple(findings)


# cf-atom: CODE-ResearchProgram
class ResearchProgram:
    def __init__(self, goal: ExperimentGoal):
        self.goal = goal
        self.hypotheses: list[Hypothesis] = []
        self.hypothesis_links: list[HypothesisLink] = []
        self.candidates: list[CandidateAlgorithm] = []
        self.runs: list[ExperimentRun] = []
        self.evidence = EvidenceLedger()
        self.decisions: list[DecisionRecord] = []

    def add_hypothesis(self, hypothesis: Hypothesis) -> Hypothesis:
        self.hypotheses.append(hypothesis)
        return hypothesis

    def add_candidate(self, candidate: CandidateAlgorithm) -> CandidateAlgorithm:
        if candidate.hypothesis not in self.hypotheses:
            self.hypotheses.append(candidate.hypothesis)
        self.candidates.append(candidate)
        return candidate

    def link_hypotheses(self, source: Hypothesis, target: Hypothesis, relation: str) -> HypothesisLink:
        if relation not in {"derived_from", "mutates", "combines", "ablates", "contradicts", "supersedes"}:
            raise ValueError("unsupported hypothesis relation")
        for hypothesis in (source, target):
            if hypothesis not in self.hypotheses:
                self.hypotheses.append(hypothesis)
        link = HypothesisLink(source_id=source.uid, target_id=target.uid, relation=relation)
        self.hypothesis_links.append(link)
        return link

    def record_run(self, run: ExperimentRun) -> ExperimentRun:
        self.runs.append(run)
        return run

    def add_run_evidence(self, *, claim_id: str, run: ExperimentRun, summary: str, confidence: float):
        self.record_run(run)
        return self.evidence.add(
            claim_id=claim_id,
            source_type="experiment_run",
            source_ref=run.run_id,
            summary=summary,
            confidence=confidence,
            tags=("run", run.plan.benchmark, run.plan.metric),
        )

    def record_decision(
        self,
        *,
        summary: str,
        rationale: str,
        evidence_refs: tuple[str, ...] = (),
        rejected_alternatives: tuple[str, ...] = (),
        follow_up_work: tuple[str, ...] = (),
    ) -> DecisionRecord:
        decision = DecisionRecord(
            decision_id=f"decision-{len(self.decisions) + 1}",
            summary=summary,
            rationale=rationale,
            evidence_refs=evidence_refs,
            rejected_alternatives=rejected_alternatives,
            follow_up_work=follow_up_work,
        )
        self.decisions.append(decision)
        return decision
