# Progress Monitoring and Independent Review

Status: canonical

Owner: Orchestrator

Update trigger: observed stalls, quota pressure, or a changed integration gate.

## Cadence

Check running lanes at least every 30 minutes with deterministic status, diffs, test results and quota
where available. Do not spend Fable on periodic status prose. A monitoring interval is not a model call.

Request a bounded independent model review at integration, a high-risk contract change, or a real stall.
Use a fresh Grok session by default, an explicitly selected Fable session for critical judgments if quota
and subscription inclusion are verified. The native Codex recovery exception is in provider-routing-policy.md.

## Scope and evidence

Read only the current board, packet, exact diff and validation evidence. Reproduce a blocker when possible.
Return `block-m1`, `park`, or `drop`; do not turn ordinary suggestions into another mandatory batch.
A data-loss/integrity failure is explicitly readmitted even if its issue was parked.

Each review records base/head, commands actually run, blockers and at most five next actions.
Implementation and review never share a session. No same-session self-approval.
A failed or unexecuted required test cannot be approved away.

## Stalls

No new diff, test or concrete blocker for 30 minutes triggers investigation. Two repeated unsuccessful
attempts trigger narrowing or handoff. A billing-unverified or quota-paused lane is a documented capacity
constraint. Preserve its checkpoint, proceed with independent eligible work, and never enable paid fallback.
