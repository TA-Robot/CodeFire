from __future__ import annotations

from dataclasses import dataclass, field

from evoagent.models import ExperimentResult
from evoagent.program import ResearchProgram


@dataclass(frozen=True)
class CampaignMilestone:
    milestone_id: str
    title: str
    required_evidence_tags: tuple[str, ...] = field(default_factory=tuple)
    min_successful_runs: int = 0
    min_improved_results: int = 0


@dataclass(frozen=True)
class CampaignProgress:
    completed_milestones: tuple[str, ...]
    blocked_milestones: tuple[str, ...]
    next_milestone: str
    completion_ratio: float


# cf-atom: CODE-ExperimentCampaignTracker
class ExperimentCampaignTracker:
    def evaluate(
        self,
        program: ResearchProgram,
        milestones: tuple[CampaignMilestone, ...],
        *,
        results: tuple[ExperimentResult, ...] = (),
    ) -> CampaignProgress:
        successful_runs = sum(1 for run in program.runs if run.succeeded)
        improved_results = sum(1 for result in results if result.quality_delta > 0 and result.confidence >= 0.5)
        evidence_tags = {tag for record in program.evidence.all() for tag in record.tags}

        completed: list[str] = []
        blocked: list[str] = []
        for milestone in milestones:
            if milestone_complete(
                milestone,
                successful_runs=successful_runs,
                improved_results=improved_results,
                evidence_tags=evidence_tags,
            ):
                completed.append(milestone.milestone_id)
            else:
                blocked.append(milestone.milestone_id)

        next_milestone = blocked[0] if blocked else ""
        completion_ratio = len(completed) / len(milestones) if milestones else 1.0
        return CampaignProgress(
            completed_milestones=tuple(completed),
            blocked_milestones=tuple(blocked),
            next_milestone=next_milestone,
            completion_ratio=completion_ratio,
        )


def milestone_complete(
    milestone: CampaignMilestone,
    *,
    successful_runs: int,
    improved_results: int,
    evidence_tags: set[str],
) -> bool:
    return (
        successful_runs >= milestone.min_successful_runs
        and improved_results >= milestone.min_improved_results
        and set(milestone.required_evidence_tags).issubset(evidence_tags)
    )
