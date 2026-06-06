from __future__ import annotations

from evoagent.models import ExperimentGoal, Hypothesis


# cf-atom: CODE-StaticPlanner
class StaticPlanner:
    def propose(self, goal: ExperimentGoal, count: int = 3) -> list[Hypothesis]:
        hypotheses = [
            Hypothesis(
                title="adaptive regularization schedule",
                rationale="Small datasets may benefit from lower early regularization and stronger late regularization.",
                expected_gain=0.08,
                novelty=0.45,
            ),
            Hypothesis(
                title="feature interaction sparsifier",
                rationale="Sparse interaction search can improve tabular generalization under tight sample budgets.",
                expected_gain=0.05,
                novelty=0.72,
            ),
            Hypothesis(
                title="confidence-weighted ensemble pruning",
                rationale="Cheap ensembles can be pruned by uncertainty contribution to improve cost-adjusted performance.",
                expected_gain=0.04,
                novelty=0.62,
            ),
        ]
        return hypotheses[: max(count, 0)]
