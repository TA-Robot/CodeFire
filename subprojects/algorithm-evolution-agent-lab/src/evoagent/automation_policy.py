from __future__ import annotations

from dataclasses import dataclass

from evoagent.models import ExperimentPlan


@dataclass(frozen=True)
class AutomationPolicy:
    allowed_command_prefixes: tuple[str, ...]
    max_estimated_cost: float
    require_artifacts: bool = True


@dataclass(frozen=True)
class AutomationPolicyFinding:
    kind: str
    detail: str


# cf-atom: CODE-SafeAutomationPolicy
class SafeAutomationPolicy:
    def check(self, plan: ExperimentPlan, policy: AutomationPolicy) -> tuple[AutomationPolicyFinding, ...]:
        findings: list[AutomationPolicyFinding] = []
        if not plan.command:
            findings.append(AutomationPolicyFinding("missing_command", "experiment command is required"))
        elif not plan.command.startswith(policy.allowed_command_prefixes):
            findings.append(
                AutomationPolicyFinding(
                    "disallowed_command",
                    "experiment command is outside configured runner prefixes",
                )
            )
        if plan.estimated_cost > policy.max_estimated_cost:
            findings.append(
                AutomationPolicyFinding(
                    "cost_limit",
                    f"estimated cost {plan.estimated_cost:g} exceeds limit {policy.max_estimated_cost:g}",
                )
            )
        if policy.require_artifacts and not plan.artifact_paths:
            findings.append(AutomationPolicyFinding("missing_artifacts", "artifact contract is required"))
        return tuple(findings)

    def allowed(self, plan: ExperimentPlan, policy: AutomationPolicy) -> bool:
        return not self.check(plan, policy)
