# CFR-168 Rust File Doctor Zero-Residual Repair

Continue exact accepted candidate
`52a025ecda3fe2f5ad8b5b2a485562e7c16ab6de` and close every finding in
`docs/development/ai/swarm-results/cfr168-rust-file-doctor-review-claude-fallback-20260730/review.md`.
Use Claude Opus 5 (`claude-opus-5`). Work only in the assigned worktree and write set. Do not
rewrite or amend the accepted candidate or its review report; produce one or more new commits on
top and leave the worktree clean.

The review verdict is `ACCEPT` for the routing decision, but CFR-168 cannot close with its
Should/Could findings or Doctor-local residual risks left open. Treat all of them as required work,
not optional follow-up.

## Required production and test changes

1. Pin the production factory, not a test-local duplicate:
   - canonical `DecisionWorkProfile::local_v1()` phase set and 1 GiB retained limit;
   - `DecisionWorkCommandKind::LocalGraph`;
   - exact file-Doctor command label;
   - a discriminator stronger than the shared profile ID, so substituting `http_v1()` fails.
2. For every injected budget, deadline, subject-change, observer, encoding, and cleanup failure,
   compare the complete project tree byte-for-byte. Mask only unavoidable lock-owner entropy.
3. Attribute retained report growth to the phase where the growth occurs. A branch/object scan
   budget failure must not claim `publication_preflight`. Keep HTTP behavior and diagnostic
   precedence byte-identical, and assert the exact typed phase at boundaries.
4. Compare `project-gate.lock.owner.json` field-wise: `version == 1` and `mode == "shared"` are
   authoritative; only `pid` and `created_at` may be masked.
5. Extend the final subject fence beyond object records to every branch and merge-request document
   used by the report. Same-inode/same-size/same-mtime replacement must be detected. Quick mode and
   missing/unreachable-root paths must also fence the subjects they actually consumed before
   publication.
6. Bind the retired 131,072-record behavior explicitly. Preserve typed budget/deadline failure
   rather than silent truncation, add near-boundary evidence without constructing an impractical
   fixture, and document the compatibility decision where the behavior is defined.
7. Replace the three `release_owner(...)?` cleanup escapes with first-error-preserving cleanup:
   always notify the failure observer, assign the correct terminal reason, close permits/debt, and
   never replace the primary error with a secondary cleanup error.
8. Enforce zero retained/temporary debt and zero permits in release builds on every failure path;
   a `debug_assert!` is not sufficient. Add a fault-injection regression proving enforcement.
9. Remove production-only risk from the observer seam. Prefer test-only plumbing where possible;
   otherwise prove and assert that the shipped no-op observer cannot charge, allocate, mutate,
   change error precedence, or affect the HTTP caller.
10. Freeze the legacy parity oracle against independently pinned golden bytes/records so changing
    both the retired oracle and live implementation cannot silently redefine compatibility.

The report's broader CLI envelope/text-output accounting finding is intentionally owned by the
command-wide response-projection residual packet after the overlapping storage/main work is
integrated. Do not touch `main.rs` here and do not claim that broader cross-command item is closed.
All Doctor-local work listed above must be closed with no new residual.

## Invariants

- One command-scoped ledger and snapshot-bound validation/fence own the entire live file route.
- Quick/full text, JSON, issue ordering, status, local/file routing, and capability semantics remain
  exact except for the already-required typed resource/deadline/subject failures.
- Success and every failure return to zero debt and zero permits and leave semantic authority
  byte-identical.
- Do not raise public limits, weaken checks, truncate silently, hide fields from snapshots, or
  encode test-only behavior in production.
- Keep parse, validate, plan, apply, render, and write boundaries explicit.

## Required validation

Use:

```bash
export RUSTUP_HOME=/tmp/codefire-rustup/rustup
export CARGO_HOME=/tmp/codefire-rustup/cargo
export PATH="$CARGO_HOME/bin:$PATH"

cargo test -p codefire-cli --bin codefire-rs remote::doctor --locked
cargo test -p codefire-cli --bin codefire-rs doctor --locked
cargo test -p codefire-cli --bin codefire-rs --locked
cargo clippy -p codefire-cli --all-targets --locked -- -D warnings
cargo fmt --all -- --check
python3 docs/development/tools/check_source_docs.py
git diff --check
```

Prove every newly added test reverses against `52a025e…`, report exact counts and hashes, and
enumerate any cross-command dependency separately. Commit only the intended repair and source-doc
updates.
