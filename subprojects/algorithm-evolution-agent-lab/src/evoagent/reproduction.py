from __future__ import annotations

from dataclasses import dataclass

from evoagent.challenge import SotaChallengeDefinition
from evoagent.program import ResearchProgram


@dataclass(frozen=True)
class ReproductionChecklistItem:
    name: str
    passed: bool
    evidence: str


@dataclass(frozen=True)
class ReproductionChecklist:
    items: tuple[ReproductionChecklistItem, ...]

    @property
    def passed(self) -> bool:
        return all(item.passed for item in self.items)

    @property
    def failures(self) -> tuple[ReproductionChecklistItem, ...]:
        return tuple(item for item in self.items if not item.passed)


# cf-atom: CODE-ReproductionChecklist
class ReproductionChecklistBuilder:
    def build(self, *, program: ResearchProgram, challenge: SotaChallengeDefinition) -> ReproductionChecklist:
        succeeded_runs = [run for run in program.runs if run.succeeded]
        artifact_runs = [
            run
            for run in succeeded_runs
            if run.artifacts and all(artifact.exists and artifact.size_bytes >= 0 for artifact in run.artifacts)
        ]
        commands = [candidate.plan.command for candidate in program.candidates if candidate.plan.command]
        items = (
            ReproductionChecklistItem(
                "challenge_baselines",
                bool(challenge.baselines),
                f"{len(challenge.baselines)} baseline(s) defined",
            ),
            ReproductionChecklistItem(
                "candidate_commands",
                bool(commands),
                f"{len(commands)} runnable candidate command(s) recorded",
            ),
            ReproductionChecklistItem(
                "run_artifacts",
                len(artifact_runs) == len(succeeded_runs) and bool(succeeded_runs),
                f"{len(artifact_runs)}/{len(succeeded_runs)} successful run(s) have declared artifacts",
            ),
            ReproductionChecklistItem(
                "repeated_runs",
                len(succeeded_runs) >= 2,
                f"{len(succeeded_runs)} successful run(s) recorded",
            ),
            ReproductionChecklistItem(
                "disallowed_shortcuts",
                bool(challenge.disallowed_shortcuts),
                f"{len(challenge.disallowed_shortcuts)} shortcut guardrail(s) listed",
            ),
            ReproductionChecklistItem(
                "compute_budget",
                challenge.allowed_compute.max_hours > 0 and challenge.allowed_compute.max_trials > 0,
                f"{challenge.allowed_compute.max_hours:g} hour(s), {challenge.allowed_compute.max_trials} trial(s)",
            ),
        )
        return ReproductionChecklist(items=items)
