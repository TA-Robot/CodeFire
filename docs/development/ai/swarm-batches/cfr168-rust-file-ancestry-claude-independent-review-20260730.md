# CFR-168 Rust File Ancestry Claude Candidate Independent Review

Independently review the completed Claude candidate for Rust file-remote fast-forward ancestry
decision work. Use Fugu 1.1 Ultra (`fugu-ultra-v1.1`). This is candidate review only: do not modify
product code, tests, issue state, or management ledgers.

## Exact subject

- baseline/task revision: `fe54e60d22dfd081bf271593ba794e4f567ed915`
- completed target: `e93ebc2`
- target branch:
  `claude-agent/cfr168-rust-file-remote-ancestry-claude-hedge-20260730-file-remote-ancestry-hedge`
- cumulative path: `crates/codefire-cli/src/remote.rs`
- contract:
  `docs/development/ai/swarm-batches/cfr168-rust-file-remote-ancestry-20260729.md`

The original Fugu candidate remains separate. This report decides only whether `e93ebc2` is an
acceptable integration candidate; it must not select a winner merely because it finished first.

## Required review

1. Upload dry-run, upload publication, and merge-request apply fast-forward decisions each own one
   ledger/session from admitted subject through goal-directed traversal, final revalidation,
   pre-effect fence, terminal selection, and cleanup.
2. Queue/frontier/seen/raw-record/path owners are reserved before allocation or insertion; traversal
   is iterative, deterministic, visits each unique subject once, stops at the goal, and exposes
   stable exact counters on chain, diamond, cycle, missing, and corrupt fixtures.
3. Audit the reported residual duplicate `codefire_store::validate_sealed_commit` calls. Decide
   whether preserving an earlier unaccounted full traversal for error precedence violates the
   frozen one-decision-ledger contract or requires composition with another accepted predecessor.
4. Audit live upload ordering: pre-existing `copy_object_graph` may run before the ancestry decision.
   Determine whether any object/authority effect can occur before a budget/deadline/tamper rejection,
   and whether comparing against an already-mutated accepted rejection is sufficient evidence.
5. Exact/minus-one/plus-one, 1x/3x/10x, 600-depth, same-metadata tamper, deadline, failure cleanup,
   compatibility, and no-effect tests must reach production callers and be non-vacuous against the
   baseline. No public limit may be raised.
6. Independently run focused remote tests, strict Clippy, format/diff checks, and representative
   legacy upload/apply tests. Treat any unaccounted predecessor traversal or pre-decision effect that
   remains on the live route as `REQUEST_CHANGES`, even if the new nucleus itself is correct.

## Output

Write exactly:

`docs/development/ai/swarm-results/cfr168-rust-file-ancestry-claude-independent-review-20260730/review.md`

Include exact hashes, `ACCEPT` or `REQUEST_CHANGES`, severity-ordered findings with concrete
file/line evidence, a verdict for all six items, command results, comparison points for the
still-running Fugu candidate, and residual risk. Commit only the report and leave the reviewer
worktree clean.
