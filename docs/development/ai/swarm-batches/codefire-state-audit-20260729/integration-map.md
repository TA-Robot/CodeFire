# Integration Map Lane

## Objective

Produce a mechanically grounded integration map for the committed CFR-168 branch family. The result must
make it possible for a later manager to choose an integration base without replaying dozens of branches by
guesswork.

## Scope

- Analyze merge bases, patch identities, unique commits, changed paths, and likely conflicts.
- Start from these candidate tips and expand only when ancestry requires it:
  - `agent/cfr168-execution-evidence-accounting`
  - `agent/cfr168-response-input`
  - `agent/cfr168-reducer-current-integration`
  - `agent/cfr168-main-integration-audit`
  - `agent/cfr168-typed-reducer-predecessor`
  - `agent/cfr168-recovery-authoritative-classifier`
  - `agent/cfr168-response-bridge-owner-fix`
- Detect commits already reachable from another tip; do not prescribe redundant cherry-picks.
- Treat documentation-only commits and implementation/test commits separately.
- Do not merge, cherry-pick, rebase, or alter any existing branch.

## Deliverable

Create only:

`docs/development/ai/swarm-results/codefire-state-audit-20260729/03-integration-map.md`

Include:

1. exact branch tips and merge bases;
2. reachability/duplicate-patch matrix;
3. path-level conflict forecast;
4. safest candidate integration base with alternatives;
5. ordered integration operations expressed as a proposal, never executed;
6. validation gates after each proposed stage;
7. rollback/checkpoint boundaries and unresolved risks.

Every command in the proposal must use full branch or commit identifiers and be safe to copy only after a
manager confirms the selected plan.

Commit the report with a focused commit and leave the worktree clean.

## Validation

- Confirm proposed non-redundancy with `git merge-base --is-ancestor`, `git cherry`, or stable patch IDs.
- Confirm all conflict predictions against actual changed-path intersections.
