from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import ExperimentPlan


@dataclass(frozen=True)
class ExperimentPlanContract:
    plan_id: str
    hypothesis_id: str
    benchmark: str
    baseline: str
    metric: str
    expected_outcome: str
    budget: float
    runner_command: str
    artifact_contract: tuple[str, ...]
    analysis_criteria: tuple[str, ...]


@dataclass(frozen=True)
class PlanContractFinding:
    kind: str
    detail: str


# cf-atom: CODE-ExperimentPlanContract
class ExperimentPlanContractBuilder:
    def build(
        self,
        plan: ExperimentPlan,
        *,
        analysis_criteria: tuple[str, ...],
    ) -> ExperimentPlanContract:
        return ExperimentPlanContract(
            plan_id=plan.uid,
            hypothesis_id=plan.hypothesis.uid,
            benchmark=plan.benchmark,
            baseline=plan.baseline,
            metric=plan.metric,
            expected_outcome=f"expected gain {plan.hypothesis.expected_gain:g}",
            budget=plan.estimated_cost,
            runner_command=plan.command,
            artifact_contract=plan.artifact_paths,
            analysis_criteria=analysis_criteria,
        )

    def validate(self, contract: ExperimentPlanContract) -> tuple[PlanContractFinding, ...]:
        findings: list[PlanContractFinding] = []
        required_text_fields = (
            ("plan_id", contract.plan_id),
            ("hypothesis_id", contract.hypothesis_id),
            ("benchmark", contract.benchmark),
            ("baseline", contract.baseline),
            ("metric", contract.metric),
            ("expected_outcome", contract.expected_outcome),
            ("runner_command", contract.runner_command),
        )
        for name, value in required_text_fields:
            if not value.strip():
                findings.append(PlanContractFinding(f"missing_{name}", f"{name} is required"))
        if contract.budget < 0:
            findings.append(PlanContractFinding("negative_budget", "budget must be non-negative"))
        if not contract.artifact_contract:
            findings.append(PlanContractFinding("missing_artifact_contract", "declared artifacts are required"))
        if not contract.analysis_criteria:
            findings.append(PlanContractFinding("missing_analysis_criteria", "analysis criteria are required"))
        return tuple(findings)
