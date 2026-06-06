from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import Hypothesis, HypothesisLink


@dataclass(frozen=True)
class MutationSpec:
    name: str
    rationale: str
    expected_gain_delta: float = 0.0
    novelty_delta: float = 0.0


@dataclass(frozen=True)
class AblationAxis:
    name: str
    removed_component: str
    rationale: str
    expected_gain_delta: float = -0.01


# cf-atom: CODE-CandidateEvolutionOperators
class CandidateEvolutionOperators:
    def mutate(self, parent: Hypothesis, spec: MutationSpec) -> Hypothesis:
        return Hypothesis(
            title=f"{parent.title} + {spec.name}",
            rationale=f"{parent.rationale} Mutation: {spec.rationale}",
            expected_gain=bounded_score(parent.expected_gain + spec.expected_gain_delta),
            novelty=bounded_score(parent.novelty + spec.novelty_delta),
        )

    def crossover(self, left: Hypothesis, right: Hypothesis, *, title: str | None = None) -> Hypothesis:
        merged_title = title or f"{left.title} x {right.title}"
        return Hypothesis(
            title=merged_title,
            rationale=(
                "Crossover combines mechanisms from two parents: "
                f"left=({left.rationale}) right=({right.rationale})"
            ),
            expected_gain=bounded_score((left.expected_gain + right.expected_gain) / 2.0),
            novelty=bounded_score(max(left.novelty, right.novelty) + 0.05),
        )

    def lineage(self, parent: Hypothesis, child: Hypothesis, relation: str) -> HypothesisLink:
        if relation not in {"mutates", "combines", "ablates"}:
            raise ValueError("unsupported evolution relation")
        return HypothesisLink(source_id=parent.uid, target_id=child.uid, relation=relation)


def bounded_score(value: float) -> float:
    return max(0.0, min(1.0, value))


# cf-atom: CODE-AblationGenerator
class AblationGenerator:
    def __init__(self, operators: CandidateEvolutionOperators | None = None):
        self.operators = operators or CandidateEvolutionOperators()

    def generate(self, parent: Hypothesis, axes: tuple[AblationAxis, ...]) -> list[Hypothesis]:
        return [
            self.operators.mutate(
                parent,
                MutationSpec(
                    name=f"ablate {axis.name}",
                    rationale=f"Remove {axis.removed_component} to test: {axis.rationale}",
                    expected_gain_delta=axis.expected_gain_delta,
                    novelty_delta=-0.05,
                ),
            )
            for axis in axes
        ]

    def lineage_links(self, parent: Hypothesis, ablations: list[Hypothesis]) -> list[HypothesisLink]:
        return [self.operators.lineage(parent, ablation, "ablates") for ablation in ablations]
