# Claude CFR-168 Independent Review

Review the committed candidate `agent/cfr168-execution-evidence-accounting`
(`aa5e57d8b1e22fb83792412c68f255ceaf603e39`) independently against the CFR-168/CFB-136
contracts visible on that branch.

This is a real reviewer lane, not a connectivity-only prompt.

## Required work

- Establish exact base, candidate, merge base, and unique commit range.
- Read the candidate's CFR-168 issue, CFB-136 issue, ADR-020, task packet, tests, and affected
  canonical source documents.
- Identify Must/Should/Nice findings with exact commit/path evidence.
- Distinguish implementation defects from missing integration, install, dogfood, or review evidence.
- Do not inspect or modify the user's dirty main worktree.
- Do not merge or alter any existing branch.

## Deliverable

Create only:

`docs/development/ai/swarm-results/claude-cfr168-review-20260729/review.md`

Commit it with a focused commit. Record commands run, validations, skipped expensive gates, confidence,
and one of `ACCEPT_FOR_INTEGRATION_REVIEW`, `REQUEST_CHANGES`, or `INSUFFICIENT_EVIDENCE`.
Leave the assigned worktree clean.
