# REC-07: Repair bootstrap cleanup work reservation (CFB-138)

Status: patched; awaiting root full-suite integration

## Objective

Fix the reproduced mismatch between reserved bootstrap rollback capacity and actual bounded cleanup work.
Current product base is 83b4522, worktree /workspace/project/.codex-worktrees/recovery-20260905. This is explicitly admitted M1 product work after the baseline failed. User prioritizes Grok, Fable scarce, no metered billing.

## Scope

Only crates/codefire-cli/src/remote/project.rs (implementation + its focused tests), and if strictly required existing bootstrap tests in remote/locks.rs or http.rs. You may update this packet with diagnosis/results. Root owns issue ledgers/governance. No other worker changes these source files. Read-only context: remote locks, core work_budget. Do not modify core ledger semantics or general budgets without returning a blocker first.

## Invariants

Preserve bounded recovery ceiling, effect-before-reservation prohibition, cancellation/permit ownership, quarantine/retry behavior, and no mutation of foreign/replaced paths. No blanket numeric budget increase and no weakening tests. Tests must cover the actual missed cost term or traversal bound and real overflow rejection.

## Validation

Source docs/development/tools/rust_env.sh before cargo. Logs target/recovery-rust/test.log and focused-results.json show six deterministic cleanup failures (45732/47136 byte limits). Reproduce remote::locks::tests::ledger_bootstrap_cancel_consumes_permits_and_releases_current alone. Inspect bootstrap_rollback_reservation, BootstrapRecoveryBudget accounting and cleanup traversal before editing. Run focused bootstrap tests, cargo fmt --check and strict clippy for touched crate. Root will run the integrated full suite with bounded test concurrency after your patch. Do not start full workspace suite yourself. If method/module/persistence contracts materially change, identify canonical source docs updates required.

## Non-Goals

Import timeout fix; first-seal UX; Python; base scripts; toolchain; provider routing; external network or model calls; main mutation; commits; reading secrets; other issue roots. No nested agents.

## Diagnosis

Cleanup traversal charged `metadata.len()` for every child, including directories. In the reproduced environment directory `st_size` contributed filesystem allocation bytes (for example 4096), so a 23-entry fixed layout already exceeded the reserved single-pass byte limit. The reservation never budgeted host allocator blocks: it reserved logical name frames plus file payload (descriptor / lock / owner / caller footprint). tmpfs directories are tens of bytes, so the same tests can pass there and fail on ext4.

Reproduced limits are exactly the single-pass formula, not the 16 MiB recovery ceiling:

- empty footprint: `E=23`, `B=23*1276+16384=45732`
- cancel footprint `(1,128)`: `E=24`, `B=24*1276+16384+128=47136`

## Corrected formula

Unchanged ceilings: `BOOTSTRAP_RECOVERY_ENTRY_LIMIT=4096`, `BOOTSTRAP_RECOVERY_BYTE_LIMIT=16MiB`.

Single-pass cleanup limits (`bootstrap_cleanup_pass_limits`):

- `E = 23 + footprint.entries`
- `B = 16384 + footprint.payload_bytes + E * 1276`
  - `1276 = LOGICAL_FRAME_BASE_BYTES(256) + 4*255` worst-case name frame
  - `16384` is the reserved file-payload lump, not directory blocks

Rollback ledger reservation: `2E` graph nodes and `2B` aggregate bytes (scan + remove observations are summed). `TemporaryBytes` is a peak (`2*16384 + 1276`) and is reserved once.

Traversal cost per child:

1. Admit the entry: charge `256 + name_len` **before** `symlink_metadata`.
2. Then `stat`. Directory payload = 0; file/symlink payload = `metadata.len()`.
3. At an exhausted entry/byte limit the walk still stops before `stat`, so a vanished or unreadable child remains a deterministic budget error, not IO.

No constant inflation. Source-doc contracts are unchanged (internal accounting only).

## Results

Patch is in `crates/codefire-cli/src/remote/project.rs` only (plus this packet). No commit.

Independent review of the first patch: 47 bootstrap tests plus both non-bootstrap-named failing locks cases passed. One `block-m1`: scan/remove/`remove_one_bootstrap_child_for_test` called `symlink_metadata` before the name-frame charge. Ordering is restored; directory `st_size` exclusion and shared pass-limit formula kept.

Grok revalidation after the ordering fix (sourced `docs/development/tools/rust_env.sh`; no workspace suite):

```bash
cargo fmt --check -- crates/codefire-cli/src/remote/project.rs
cargo test --locked -p codefire-cli --bin codefire-rs remote::project::tests::bootstrap_cleanup
cargo test --locked -p codefire-cli --bin codefire-rs remote::project::tests::bootstrap_recovery_reports_typed_entry_bound -- --exact
cargo test --locked -p codefire-cli --bin codefire-rs remote::project::tests::bootstrap_removal_keeps_descriptor_until_retry_can_finish -- --exact
```

All five named tests passed. Root owns clippy/full-suite with bounded concurrency. Import lock timeout remains out of scope.

## Handoff

Product diff is ready for final independent review and integrated source validation. Root owns issue closure and commit; focused intermediate passes do not mark M1 complete.
