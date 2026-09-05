# Orchestration Protocol

Status: canonical agent topology for CodeFire development

Owner: Orchestrator

Update trigger: role, provider entitlement, or invocation path changes.

This is an external development workflow, never a product runtime feature.
Scope is the M1 on [milestones](../milestones.md). Billing and selection follow
[provider policy](provider-routing-policy.md) and [matrix](provider-routing-matrix.json).

## Roles

| Role | Default | Responsibility |
|---|---|---|
| Orchestrator | current Codex subscription session | User outcome, bounded packets, dependencies, integration |
| Worker | Grok 4.6 High | Reproduction, implementation, focused tests, handoff |
| Scarce escalation | Fable 5.1, explicit only and verified included | Hard design/root cause or important review |
| Reviewer | Fresh session, other family if practical | Independent counterexamples and exact-diff verification |
| Monitor | Deterministic tools | Progress, evidence, quota, timeout and checkpoints |

For the recovery bootstrap, native current-session Codex workers/reviewer are explicitly admitted for
REC-02..05 while external billing remains unverified. No external API or paid fallback is admitted.

## Execution

1. Read the current board; name the user outcome and exit commands.
2. Write bounded packets with base, write/contract set, non-goals, invariants, validation and handoff.
3. Dispatch independent eligible packets concurrently. Billing-unverified or quota-paused is ineligible capacity, not idle capacity.
4. Reuse the implementing session for dependent work; keep review in a fresh session.
5. Start Grok through its official subscription CLI only after billing preflight, in a worktree under the configured workspace. Do not claim legacy dev-swarm schedules Grok.
6. Use explicit model/effort and bounded turns; do not let workers recursively spawn unbounded agents.
7. Gather diff, exact HEAD and results. Run integration validation, then independent review as appropriate to the final diff.
8. Close only on evidence and actual use; stage exit and experiment outcome are distinct.

High-risk data loss or integrity findings may trigger explicit M1 readmission regardless of old parking.
Routine findings stay parked. Two consecutive attempts without new evidence trigger a narrower question
or a handoff, not unlimited retries. On quota/failure, preserve branch, diff, session and next action.

## Capacity

One orchestrator plus two workers initially; Grok ceiling 2, Claude ceiling 1. Reserve quota for fixes and
integration. Fable is not a standing monitor. Review backlog of two stops new implementation dispatch.
Thirty-minute health checks use tools; model audits are event-triggered under [audit cadence](audit-cadence.md).
