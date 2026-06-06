from __future__ import annotations

from evoagent.models import CandidateAlgorithm, ExperimentGoal, ExperimentPlan, Hypothesis
from evoagent.scheduler import PortfolioScheduler
from evoagent.scoring import score_candidate


# cf-atom: CODE-EvolutionAgent
class EvolutionAgent:
    def __init__(self, goal: ExperimentGoal):
        self.goal = goal
        self.candidates: list[CandidateAlgorithm] = []

    def seed(self, hypotheses: list[Hypothesis]) -> list[CandidateAlgorithm]:
        created = []
        for index, hypothesis in enumerate(hypotheses, start=1):
            plan = ExperimentPlan(
                hypothesis=hypothesis,
                benchmark="initial-benchmark",
                baseline="current-best-baseline",
                metric=self.goal.target_metric,
                estimated_cost=min(self.goal.budget_hours, 1.0),
            )
            candidate = CandidateAlgorithm(
                name=f"candidate-{index}",
                hypothesis=hypothesis,
                plan=plan,
            )
            self.candidates.append(candidate)
            created.append(candidate)
        return created

    def next_candidate(self) -> CandidateAlgorithm:
        if not self.candidates:
            raise ValueError("no candidates available")
        return max(self.candidates, key=score_candidate)

    def portfolio(self, limit: int) -> list[CandidateAlgorithm]:
        return PortfolioScheduler().schedule(self.candidates, limit=limit)
