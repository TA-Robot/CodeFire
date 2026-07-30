# CFR-168 Python HTTP Doctor + Branch Composition Re-review

Independently re-review the corrected Doctor and branch-list composition. Use Fugu 1.1 Ultra
(`fugu-ultra-v1.1`). This is review-only: do not modify product code, tests, issue state, or
management ledgers.

## Exact subject

- Corrected composition target:
  `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714`
- Pre-composition management/product base:
  `16d0129`
- Doctor composition commit:
  `5e3b0c769f84e10a57592c3c9bf229e61cd37ebe`
- Adjacent upload composition commit, outside this verdict:
  `0703f71b9800ff19ce52d01bcd46c8eddb323df2`
- Branch composition/conflict-resolution commit:
  `7e87665b8158a37f1d25b449de7831cf9420f545`
- Mandatory canonical test correction:
  `701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714`
- Prior review:
  `docs/development/ai/swarm-results/cfr168-python-http-doctor-branch-independent-review-20260730/review.md`

Review the exact target tree and the Doctor/Branch/corrective deltas. Do not accept or reject the
adjacent upload product slice in this report; it has a separate independent review and CFB-138
gate.

## Required re-review

1. Confirm prior `MUST-1` is closed exactly: an expired branch-list request returns typed 503
   `request_deadline_exceeded`, phase `branch list planning`, one poll, terminal
   `deadline_exceeded`, no 200, no permit, and zero debt.
2. Confirm the six `codefire` conflict blocks use the prior review's required combined
   `read_route_phase` resolution and reset Doctor/branch/bundle/mutation flags on every request.
3. Confirm neither Doctor nor Branch production ownership, exact/deadline behavior, diagnostic
   order, final revalidation, bundle behavior, or mutation response behavior was dropped by the
   composition.
4. Run both dedicated suites, the corrected affected CLI method, representative bundle/mutation
   neighbors, Python compilation, source/project docs checks, and `git diff --check`.
5. Inspect the manager's exact-target full-suite evidence: Python discovery at `701469f` ran
   463/463 in 296.500 seconds. Do not claim you ran it yourself unless you do; state whether the
   exact revision and command are sufficient corroborating evidence.

Any new blocking finding or incomplete correction is `REQUEST_CHANGES`. Otherwise return `ACCEPT`
for the Doctor+Branch corrected composition only.

## Output

Write exactly:

`docs/development/ai/swarm-results/cfr168-python-http-doctor-branch-composition-rereview-20260730/review.md`

Include exact hashes, verdict, finding disposition, commands/results, conflict-resolution verdict,
full-suite evidence disposition, and residual risk. Commit only this report and leave the reviewer
worktree clean.
