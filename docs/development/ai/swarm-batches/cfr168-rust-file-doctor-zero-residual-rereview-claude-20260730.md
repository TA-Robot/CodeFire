# CFR-168 Rust File Doctor Zero-Residual Independent Rereview

Use Claude Opus 5 (`claude-opus-5`) as an independent reviewer. Review only;
do not modify product code, tests, source contracts, issue state, or manager
ledgers. Commit exactly one immutable review report.

## Exact subject

- accepted product predecessor:
  `52a025ecda3fe2f5ad8b5b2a485562e7c16ab6de`
- repair task base:
  `dbeaac1526acc8c2e102ee2b137e8f2a708890b0`
- exact candidate:
  `1a04490806b59f8c39b5c1cf03f42d06aa750a98`
- candidate tree: `eafb2cbf20980944d16086a972100c321a8b56be`
- binary patch SHA-256:
  `5f150dbe37b61b592fa5a52e9ce72a4367ba7c8de2c09eba79390a1c37427fac`
- prior accepted-review report SHA-256:
  `c1b39ffc6f25cf9f19007af3cd5336421400e467e6e3c28969de775b86ed8045`
- repair-contract SHA-256:
  `732f070322f5452619c8a492b34174399a2aa9d06935440cbefbc57abe6c7375`

Changed paths are exactly:

```text
crates/codefire-cli/src/remote/doctor.rs
docs/development/change-playbook.md
docs/development/command-source-trace.md
docs/development/source-architecture.md
docs/development/source-data-model-catalog.md
```

First recompute every identity, the clean state, candidate ancestry, changed
paths, and patch digest. A mismatch is `REQUEST_CHANGES`; do not review a
moving substitute.

## Required independent falsification

Re-audit the eight residuals from
`docs/development/ai/swarm-results/cfr168-rust-file-doctor-review-claude-fallback-20260730/review.md`
against the live production route and the repair contract:

1. the production factory, profile, command kind, and label are asserted
   through the real File Doctor entry point rather than a helper-only owner;
2. every injected budget, deadline, cancellation, observer, fence, gate,
   report-growth, cleanup, and publication failure leaves a byte-identical
   semantic repository tree, zero debt, no open permit, and one terminal
   observation;
3. each report-growth charge is attributed to its real bounded phase rather
   than a convenient generic phase;
4. gate acquisition, wait, exclusion, stable identity, release, and terminal
   fields are independently observable and non-vacuous;
5. the final subject fence covers every authoritative branch,
   merge-request, object, generation, signature, and Doctor input claimed by
   the source contracts, with mutation hooks after the last observation;
6. retiring the 131,072-record cap remains bounded by the declared
   decision-work/deadline/resource contract at exact, minus-one, plus-one,
   cancellation, and large-scale boundaries;
7. every fallible cleanup/release path preserves the primary result, remains
   retryable or completes deterministically, records failure, and cannot
   bypass terminalization;
8. zero-debt and zero-permit closure is enforced in production behavior, not
   only by `debug_assert!` or test-only inspection.

For every claimed oracle, perform or inspect a deliberate regression that
would violate the exact property. Reject helper-scoped stage jumps, deleted
whole helpers, constant stage labels, or snapshots that omit a claimed
authority as proof. Trace the actual `cf file doctor` call graph and separate
test seams from production behavior.

Also inspect the 2,132-insertion/266-deletion delta for duplicated owners,
unbounded retained state, hidden allocations, panic paths, dead compatibility
code, changed wire/CLI behavior, error-precedence drift, and source-document
overclaim. Preserve the previously accepted routing decision and all
unrelated File/HTTP behavior.

## Validation boundary

Run the smallest independent focused cases needed to reproduce each property,
then the authorized Rust CLI/core/Clippy/document gates if the environment is
source-bound and clean. Record exact commands, counts, durations, toolchain,
target path, and failures. Passing worker-reported gates are context, not
independent evidence. Do not change the candidate to make a probe pass.

## Decision and output

Write exactly:

`docs/development/ai/swarm-results/cfr168-rust-file-doctor-zero-residual-rereview-claude-20260730/review.md`

The report must include:

- recomputed subject identity and clean-state evidence;
- one verdict for each of the eight residuals;
- production reachability and deliberate-regression evidence;
- severity-ordered findings with concrete file/line and reproducer;
- commands/results and any unavailable evidence;
- exact integration/write-overlap map;
- `ACCEPT` only if every required residual is closed with no material
  unowned finding in this scope; otherwise `REQUEST_CHANGES`.

Commit only the report and leave the reviewer worktree clean.
