# State Audit Synthesis Lane

## Objective

Synthesize the four independently committed audit reports already merged into this worktree. Preserve
disagreement instead of averaging it away. This lane prepares an integration handoff; it does not merge to
main or declare CFR-168 complete.

## Inputs

- `docs/development/ai/swarm-results/codefire-state-audit-20260729/01-state-inventory.md`
- `docs/development/ai/swarm-results/codefire-state-audit-20260729/02-cfr168-contract-audit.md`
- `docs/development/ai/swarm-results/codefire-state-audit-20260729/03-integration-map.md`
- `docs/development/ai/swarm-results/codefire-state-audit-20260729/04-next-roots-readiness.md`

## Deliverable

Create only:

`docs/development/ai/swarm-results/codefire-state-audit-20260729/05-manager-handoff.md`

The handoff must include:

1. immutable base and all input report commits;
2. facts agreed by multiple lanes;
3. conflicting or low-confidence conclusions;
4. a prioritized next-action queue with provider, role, timeout, dependencies, write set, and acceptance
   evidence for each task;
5. the precise checkpoint at which Claude authentication is required;
6. any genuinely Ultra decision that could justify guarded Codex `gpt-5.6-sol`, otherwise explicitly state
   that no Codex lane is needed;
7. a no-main-merge warning and exact manager verification commands.

Commit the handoff with a focused commit. Confirm the worktree is clean. Do not edit any input report.

## Validation

- Confirm all four input paths are present and committed.
- Confirm the handoff cites the actual input branch heads visible in Git.
- Check that every proposed parallel write set is disjoint.
