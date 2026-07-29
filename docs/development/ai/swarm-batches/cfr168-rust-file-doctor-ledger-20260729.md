# CFR-168 Rust File-Remote Doctor Ledger Adoption

Unify the live Rust `cf remote doctor cf://…` path with the accepted accounted Doctor
implementation on immutable current-state snapshot
`e2a218e015953f082245f36dbc6aee906b837e5a`.

Use Fugu 1.1 Ultra (`fugu-ultra-v1.1`). Work only in the assigned worktree and write set. Do not
change issue status, edit management ledgers, or widen this packet into HTTP, storage, scan, upload,
merge, metrics, or capability work.

## Production defect

`run_remote_doctor` sends HTTP targets through the request-accounted implementation, but file
targets still call the duplicate unaccounted `diagnose_remote_project`. The module already has the
accepted `diagnose_remote_project_with_work_and_poll` path, including accounted report ownership,
Store scanner reuse, shared sealed-validation session, final revalidation, and typed accounting
errors.

## Required behavior

- For `cf://` targets, create exactly one command-scoped `DecisionWorkLedger` using the canonical
  local profile before project/layout inspection.
- Route the live file command through `diagnose_remote_project_with_work_and_poll`; do not maintain
  two semantically independent Doctor implementations.
- Preserve exact quick/full report content, issue and skipped-check ordering, diagnostics, exit
  behavior, JSON/text output, missing-project behavior, and transport/project fields.
- Keep one shared sealed-validation session for all discovered roots and one final subject
  revalidation. Same-path, same-inode, same-size, same-mtime content mutation must invalidate the
  result.
- Charge report/scanner/session/response work before allocation or publication. Finish the report,
  preflight its projected output, finalize the ledger, and publish only after every fallible
  decision boundary succeeds.
- Budget/deadline/final-fence failure must be typed, release all concurrent ownership, and publish
  no partial success output. File Doctor is read-only and must not alter the remote project.
- Remove or reduce the legacy duplicate only when compatibility and tests prove it safe. Any
  retained direct compatibility helper must be clearly outside the production command path and
  must not become an unaccounted fallback.

## Required production-path tests

Add focused tests in `crates/codefire-cli/src/remote/doctor.rs` which invoke the live file command
path and prove:

- the exact ledger identity reaches the accounted scanner and sealed-validation session;
- no second ledger is created;
- quick/full byte and diagnostic equivalence with the previous successful behavior;
- exact / limit-minus-one / plus-one boundaries for meaningful dimensions;
- deterministic cancellation at entry, scan, validation, final revalidation, response preflight,
  and pre-publication;
- 1x/3x/10x stable counter curves for file Doctor's selected work;
- shared-DAG reuse, deterministic ordering, and same-metadata tamper invalidation;
- zero retained/concurrent ownership on success and every injected failure;
- no output bytes are published on typed failure.

Avoid helper-only tests and assertions that merely prove a counter is nonzero.

## Validation

At minimum run:

```bash
export RUSTUP_HOME=/tmp/codefire-rustup/rustup
export CARGO_HOME=/tmp/codefire-rustup/cargo
export PATH="$CARGO_HOME/bin:$PATH"
cargo fmt --all -- --check
cargo test -p codefire-cli --bin codefire-rs remote::doctor --locked
cargo clippy -p codefire-cli --bin codefire-rs --locked -- -D warnings
python3 docs/development/tools/check_source_docs.py
git diff --check
```

Run narrower exact tests while iterating, then the full `codefire-cli` binary target if time
permits. Commit the implementation and leave the worktree clean.

## Handoff

Report the live call chain, ledger/session identity, legacy-path disposition, exact boundaries,
counter observations, subject-change/no-output evidence, validation commands/counts, and every
remaining CFR-168 Doctor surface outside this packet.
