# Next Roots Readiness Lane

## Objective

Determine what can safely proceed in parallel after CFR-168 and what must remain blocked. Focus on committed
evidence for CFR-169 through CFR-173 rather than roadmap labels alone.

## Evidence Sources

- `docs/development/current-cycle-burn-down-plan.md`
- `docs/development/code-review-issues/cfr-169.md` through `cfr-173.md`
- corresponding review portfolio entries and dispositions
- relevant task packets, history entries, source docs, and candidate branches:
  - `agent/cfr169-ref-registry`
  - `agent/cfr170-migrate-actions`
  - `agent/cfr171-claim-descriptor`
  - `agent/cfr172-readonly-experiment`
- all committed dependency evidence from CFR-168

Do not change implementation or canonical status. Do not read or modify the user's dirty main worktree.

## Deliverable

Create only:

`docs/development/ai/swarm-results/codefire-state-audit-20260729/04-next-roots-readiness.md`

For CFR-169..173, record:

- frozen problem and acceptance boundary;
- hard dependencies on CFR-168 or earlier roots;
- available committed implementation/test evidence;
- whether its existing branch is exploratory, partial, integration-ready, or stale;
- safe parallelizable sublanes with disjoint write sets;
- required independent review and deterministic gates;
- recommended action: `hold`, `packetize`, `review-candidate`, or `ready-after-cfr168`.

End with a dependency DAG and a proposed two-wave worker schedule that uses Fugu for bounded evidence,
Claude for implementation/review when authenticated, and no automatic Codex fallback.

Commit the report with a focused commit and leave the worktree clean.

## Validation

- Verify every cited branch and commit.
- Trace each claimed dependency back to a canonical issue, task packet, or roadmap statement.
