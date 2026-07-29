# CFR-168 Rust File-Remote Ancestry Adoption

Adopt the accepted decision-work/session primitives in the live Rust file-remote ancestry
decisions on immutable current-state snapshot
`e2a218e015953f082245f36dbc6aee906b837e5a`.

Use Fugu 1.1 Ultra (`fugu-ultra-v1.1`). Work only in the assigned worktree and write set. Do not
change issue status, merge the rejected Python candidate line, edit management ledgers, or widen
this packet into HTTP, storage, local merge, or capability work.

## Production scope

The current `remote::is_ancestor_in_objects` materializes a complete
`ancestor_distances` map for a boolean decision and is called by:

- file-remote upload dry-run fast-forward validation;
- file-remote upload live fast-forward validation;
- file-remote merge-request apply fast-forward validation.

The same module already contains the accepted
`is_ancestor_in_provisional_subject_with_poll` nucleus and decision-work/session primitives. Trace
their real ownership contracts before editing; reuse or safely adapt them rather than adding a
second accounting vocabulary.

## Required behavior

- Start one command-scoped `DecisionWorkLedger` before the selected file-remote decision work.
- Use one pinned/validated object-root subject and one goal-directed iterative ancestry traversal.
  Stop at the requested ancestor; do not materialize every reachable distance for a boolean query.
- Charge before work/allocation across applicable dimensions: object records/files/bytes,
  graph nodes/edges, logical depth/frontier, retained scalar/bytes, and final revalidation.
- Preserve the existing fast-forward truth value, parent order, error precedence, dry-run payload,
  live output bytes, and success-side filesystem mutations.
- Revalidate the exact subject after traversal and before any upload/apply effect. A same-path,
  same-inode, same-size, same-mtime content rewrite must invalidate the result.
- Limit/deadline/final-fence failure must be typed, release only this operation's ownership, and
  leave branch, generation, merge-request, object, idempotency, and publication state unchanged.
- No nested ledger, unaccounted fallback, partial publication, cache reopening, or second full
  traversal is allowed.
- Keep direct helpers compatible for callers outside this packet. If a compatibility wrapper must
  own a local ledger, production callers in scope must demonstrably use the command ledger path.

## Required production-path tests

Add focused tests in `crates/codefire-cli/src/remote.rs` that exercise the live file-remote callers,
not only the helper:

- exact / limit-minus-one / plus-one boundaries for the meaningful dimensions;
- deterministic deadline expiry at entry, traversal, final revalidation, and pre-effect;
- 1x/3x/10x counters proving goal-directed linear work and early exit;
- deep linear history and shared-DAG traversal without recursion or duplicate physical reads;
- same-metadata content tamper at the final fence;
- dry-run and live fast-forward/non-fast-forward semantic and diagnostic byte equivalence;
- zero retained/concurrent ownership after success and every injected failure;
- no mutation or partial publication on every typed failure.

Use the strongest existing fixtures and exact accepted APIs already in the module. Avoid assertions
that merely prove a counter is nonzero.

## Validation

At minimum run:

```bash
export RUSTUP_HOME=/tmp/codefire-rustup/rustup
export CARGO_HOME=/tmp/codefire-rustup/cargo
export PATH="$CARGO_HOME/bin:$PATH"
cargo fmt --all -- --check
cargo test -p codefire-cli --bin codefire-rs remote --locked
cargo clippy -p codefire-cli --bin codefire-rs --locked -- -D warnings
python3 docs/development/tools/check_source_docs.py
git diff --check
```

Run narrower exact tests while iterating, then the full `codefire-cli` binary test target if time
permits. Commit the implementation and leave the worktree clean.

## Handoff

Report the exact production callers, ledger/session identity, charge order, final fence, tested
dimensions and counts, 1x/3x/10x observations, mutation/no-effect evidence, compatibility decision,
and every residual CFR-168 Rust ancestry surface outside this packet.
