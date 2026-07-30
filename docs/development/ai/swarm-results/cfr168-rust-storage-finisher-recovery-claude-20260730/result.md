# CFR-168 Rust storage decision-work finisher — recovery evidence record

Status: **evidence record only. Not a review. Not an integration or Close authorization.**

Date: 2026-07-30
Branch: `claude-agent/cfr168-rust-storage-claude-finisher-20260730`
Agent: `cfr168-rust-storage-claude-finisher-20260730` (Claude)

This document makes durable the gate results that were actually recorded in this
session's event log before two host crashes. It records nothing that the log
does not contain, and it explicitly marks the last combined command as
interrupted. Every claim below is auditable against the log path and the Git
object names given here.

## 1. Subject, base, and range

| Role | Object |
| --- | --- |
| Subject (exact candidate, preserved) | `b3bc32a6bd4c792edc5a1302c61e16c624a9b648` |
| Subject tree | `42480dfd8f87a25548a980a5196269a3d44c4d18` |
| Frozen WIP root (finisher base) | `f5bce305da7ebfd08a840a4b281e29b0476874e0` |
| WIP parent (audit base for the whole packet) | `40c095de250df2ee9e9e34526e1926069b476e42` |
| Pre-CFR-168 reference for equivalence | `e2a218e` (`crates/codefire-cli/src/storage.rs`) |

Finisher delta range: `f5bce30..b3bc32a`
Full packet range: `40c095d..b3bc32a`

`f5bce30` was preserved: it was neither reset, discarded, replaced, nor amended.
`b3bc32a` is a separate commit on top of it and is itself preserved unamended by
this recovery step.

### Exact product content hashes at `b3bc32a`

| Path | Git blob |
| --- | --- |
| `crates/codefire-cli/src/main.rs` | `c71d0f5965d146f08624179d7693ff64c293490e` |
| `crates/codefire-cli/src/storage.rs` | `1a5ed9972cba6ccd282af817f0c1648f6dfbc6ae` |
| `crates/codefire-cli/src/tests.rs` | `57003cf20c954c55cdd0dfe62721cb7f5d776b5a` |
| `crates/codefire-cli/src/tests/storage_decision_work.rs` | `770801003e0115324f434b9ed3c90f940defdb89` |

### Changed paths

`f5bce30..b3bc32a` (the finisher delta, 2 files, +436 / -59):

```
M	crates/codefire-cli/src/storage.rs
M	crates/codefire-cli/src/tests/storage_decision_work.rs
```

`40c095d..b3bc32a` (the whole storage packet, 4 files, +2310 / -498):

```
M	crates/codefire-cli/src/main.rs
M	crates/codefire-cli/src/storage.rs
M	crates/codefire-cli/src/tests.rs
A	crates/codefire-cli/src/tests/storage_decision_work.rs
```

No file outside `crates/codefire-cli` is touched. `codefire-core`,
`codefire-store`, `codefire-util`, the Python runtime, and every schema are
unchanged by this packet.

## 2. Durable event log

All gate claims in section 3 are quoted from completed `tool_result` records in:

```
/workspace/worktrees/cfr168-aa5-probe/.fugu-second-agent/773723eb10c28988a9ce2c1863aa7df70e61559ee90f8e4c55e48911b7f44e66/worktrees/cfr168-rust-storage-recovery-20260729-storage-finish-after-reboot/.claude-second-agent/191250450d736b75461fc95410e9f0ff0e70b9c960b2b6eed801143e5d1d3a76/agents/cfr168-rust-storage-claude-finisher-20260730/logs/events.jsonl
```

The immutable audit subject is the log's first 648 LF-terminated records, exactly
2 719 170 bytes, with SHA-256
`6923c01a2bac3791c2ba03d3b1e7c46b2fec89da9b98140e97a1758e89c96a04`; that prefix
ended at mtime 2026-07-30 08:32. The same session log is append-only and gained
later recovery/report-writing events, so its current total line count and whole-file
digest are intentionally not evidence identities. The fixed prefix contains 78
`Bash` tool uses; 75 have a completed `tool_result`, and 3 have none. `seq NN`
below is the zero-based index of the tool use within that prefix, so each row can
be re-read directly.

Audit method used to build this table: pair every `tool_use` with its
`tool_use_id` in a later `tool_result`. A tool use with no matching result is
treated as interrupted and is **not** accepted as evidence, regardless of what
the command would have produced.

### Source-state fence

The last product source or test mutation in the log is `seq 78`
(`Edit crates/codefire-cli/src/tests/storage_decision_work.rs`), immediately
followed by `seq 79` which applied `cargo fmt --all`. There is **no** `Edit`,
`Write`, or source-rewriting `Bash` tool use after `seq 79`. Therefore every
gate at `seq >= 79` ran against the exact source state now committed as
`b3bc32a`, which `seq 89` independently confirmed by content probe and which
`git status --porcelain` reports as a clean worktree.

## 3. Completed gates accepted as evidence

Two toolchains appear in the log. Both are Rust 1.97.x and both satisfy the
workspace MSRV of 1.75.

- **TC-A** (pre-crash, ephemeral): cargo 1.97.1 / rustc 1.97.1,
  `RUSTUP_HOME=/tmp/codefire-rustup/rustup`, `CARGO_HOME=/tmp/codefire-rustup/cargo`.
- **TC-B** (post-crash, persistent): cargo 1.97.0 (`c980f4866`) / rustc 1.97.0
  (`2d8144b78`), `RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup`,
  `CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo`,
  `CARGO_BUILD_JOBS=2`, `RUST_TEST_THREADS=2`.

| # | seq | TC | Command | Recorded result |
| --- | --- | --- | --- | --- |
| G1 | 84 | A | `cargo test --package codefire-cli --bin codefire-rs` | `test result: ok. 779 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 40.59s` |
| G2 | 83 | A | `cargo test ... --bin codefire-rs remote::` then `http::` | `83 passed; 0 failed; ... 696 filtered out` and `133 passed; 0 failed; ... 646 filtered out` |
| G3 | 86 | — | `python3 docs/development/tools/check_source_docs.py` | `Source docs check passed: 44 modules, 34 commands, 12 object kinds.` `EXIT=0` |
| G4 | 79 | A | `cargo fmt --all && cargo test ... storage_decision_work` | `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 768 filtered out; finished in 0.68s` |
| G5 | 81 | A | `cargo test ... -- --exact` × 4 legacy storage tests | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 775 filtered out; finished in 0.10s` |
| G6 | 90 | B | `cargo fmt --all -- --check` | `FMT_OK` (no diff) |
| G7 | 90 | B | `git diff --check f5bce30 b3bc32a` | `DIFFCHECK_OK` (no whitespace/conflict defect) |
| G8 | 91 | B | `cargo clippy --package codefire-cli --all-targets --locked -- -D warnings` | `Finished \`dev\` profile ... in 49.62s`; zero `error:`/`warning:` lines under `--message-format short` |
| G9 | 92 | B | `cargo test --package codefire-cli --bin codefire-rs --locked tests::storage_decision_work::` | `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 768 filtered out; finished in 0.65s` |
| G10 | 93 | B | `cargo test ... --locked -- --exact` × 4 legacy storage tests | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 775 filtered out; finished in 0.07s` |

### Filters demonstrably select tests

Every filtered run reports a non-zero selected count together with a non-zero
filtered-out count, so no gate silently matched nothing:

- `tests::storage_decision_work::` → 11 selected / 768 filtered out (G9).
- 4 fully-qualified `--exact` names → 4 selected / 775 filtered out (G10).
- `remote::` → 83 selected / 696 filtered out; `http::` → 133 selected / 646
  filtered out (G2).
- The unfiltered binary suite reports `0 filtered out` over 779 tests (G1),
  which bounds the suite size the filtered counts are taken from.

### The 11 focused `storage_decision_work` tests (G9, TC-B, all `ok`)

```
storage_decision_work_preserves_quick_and_full_json_bytes
storage_decision_work_preserves_reference_text_bytes
storage_malformed_record_failure_paths_finish_without_debt
storage_integrity_failure_keeps_reference_parsed_count_and_kind
storage_object_discovery_read_and_parse_limits_accept_exact_and_reject_minus_one
storage_remote_integrity_failure_keeps_reference_kind
storage_reports_remote_warnings_after_local_object_warnings
storage_order_and_bounded_largest_state_are_stable_with_many_candidates
storage_response_preflight_is_exact_and_never_publishes_partial_output
storage_retained_aggregation_accepts_exact_and_rejects_minus_one
storage_selected_remote_work_accepts_exact_and_rejects_minus_one
```

### The 4 fully-qualified legacy storage tests (G5, G10, all `ok`)

```
tests::storage_report_counts_objects_by_type_and_warns_large_objects
tests::storage_report_counts_file_remote_project_storage
storage::tests::active_state_scan_does_not_follow_outside_symlinks
storage::tests::active_state_scan_rejects_symlinked_root
```

### Read-only checks re-run during this recovery step

These use Git and text only. They create no build artifacts and invoke no Cargo
directly or indirectly.

| Check | Result |
| --- | --- |
| `git rev-parse HEAD` | `b3bc32a6bd4c792edc5a1302c61e16c624a9b648` |
| `git status --porcelain` (before this report) | empty (clean) |
| `git diff --check` (worktree) | empty (clean) |
| `git diff --check 40c095d b3bc32a` (whole packet range) | empty (clean) |

## 4. Interrupted commands — explicitly not evidence

Three tool uses in the log have **no** completed `tool_result`. Nothing is
inferred from the fact that they started.

| seq | TC | Command | Disposition |
| --- | --- | --- | --- |
| 87 | A | `cargo fmt --all -- --check; cargo clippy --package codefire-cli --all-targets -- -D warnings; git diff --check; git diff --check 40c095d; git status --porcelain` | **Interrupted** by the first host OOM restart. Not evidence. Superseded on TC-B by G6, G7, G8 and by the read-only range check in section 3. |
| 94 | B | `cargo test ... --locked remote::` && `cargo test ... --locked http::` && `python3 docs/development/tools/check_source_docs.py` && `cargo build --package codefire-cli --bin codefire-rs --locked` | **Interrupted** by the second host crash (disk exhaustion; 91.8 GiB of reproducible Cargo targets were then removed). **Not evidence.** This is the combined remote/http/source-doc/build command referred to in the recovery instruction. Its four constituent gates are covered on TC-A only, by G2, G3, and — for the binary target — by the earlier successful `cargo build --package codefire-cli --bin codefire-rs` (`seq 13`) plus the compile that G1 necessarily performed. They have **not** been re-confirmed on TC-B. |
| 103 | — | log-analysis Python one-liner (no product effect) | Interrupted; re-run successfully in this step. |

## 5. Delta contents at `b3bc32a` and the invariants it targets

The finisher delta (`f5bce30..b3bc32a`) makes six substantive changes plus test
coverage. Each is stated with the invariant it serves, so a reviewer can check
the claim against the code rather than against this document.

### 5.1 Gate-blocking defects in the frozen WIP

1. **MSRV violation.** `storage.rs` used `std::io::ErrorKind::NotADirectory`
   (stable 1.83) against a declared workspace MSRV of 1.75. Replaced with
   `ErrorKind::InvalidInput`, matching the convention already used across
   `secure_worktree.rs`, `close_transaction.rs`, and `import_workflow.rs`. The
   message text is unchanged.
2. **`clippy::result_large_err` ×5.** `StorageReportFailure` carried an inline
   `DecisionWorkSnapshot` (seven `BTreeMap`s) making every fallible signature's
   `Err` ≥272 bytes. The test-only snapshot field is now `Box`ed.
3. **`clippy::result_large_err` ×2 on Store visitor closures.** Annotated with
   `#[expect(clippy::result_large_err, reason = "the Store scanner visitor
   contract requires StoreError")]`, matching the existing annotation in
   `remote/doctor.rs`.
4. **`clippy::too_many_arguments`.** `consider_largest_object` took 8 arguments.
   Introduced a borrowed `LargestObjectCandidate<'_>` so the identity travels as
   one value; no behaviour change.

Before this delta, strict Clippy failed with 9 errors (4 in the bin target, 9 in
the all-targets test build), so the frozen WIP could not have passed its own
declared gate. After it, G8 is clean.

### 5.2 Reference-equivalence defects (text/JSON results and diagnostic order)

5. **`json_records_parsed` undercounted, and integrity warnings were
   misclassified.** The pre-CFR-168 implementation read/parsed a record
   (`json_records_parsed += 1`) and then validated it in a *second* step, so a
   digest/id/filename/incomplete failure was counted as **parsed and invalid**
   and reported as `object_integrity_error` (remote: `remote_object_integrity_error`).
   The accounted scanner
   `codefire_store::scan_object_store_record_documents_with_work_and_cancellation`
   performs read, parse, *and* `validate_object_record` internally and surfaces a
   single `entry.document = Err(..)`, so the WIP counted such a record as *not
   parsed* and, for `IncompleteRecord`, labelled it `invalid_object_record`.
   That is a divergence in emitted JSON (`objects.scan.json_records_parsed`) and
   in the warning taxonomy.

   Fixed by `classify_object_record_failure(&error, &path)`, which restores the
   two-stage distinction without touching `codefire-store`:
   `ObjectIdMismatch`, `ObjectHashMismatch`, and `FilenameMismatch` are produced
   only by that validator; `IncompleteRecord` is shared with the bounded reader,
   and the validator is its only producer whose context is the *bare* record
   path (every reader/parser context carries a message prefix). Local and remote
   kinds are derived from one classification so the two surfaces cannot drift.
   Covered by `storage_integrity_failure_keeps_reference_parsed_count_and_kind`
   (hash mismatch, `payload: null` → post-parse `IncompleteRecord`, and a
   truncated record) and `storage_remote_integrity_failure_keeps_reference_kind`.

6. **Scan order regression changed diagnostic precedence.** The reference
   evaluated `objects → remotes → active_state → idempotency` (the last two fall
   out of struct-literal field order). The WIP scanned active-state and
   idempotency *before* remotes, which changes which diagnostic is reported first
   when more than one area fails, and changes the cumulative charge sequence that
   decides which budget dimension trips first. Restored to reference order;
   warning concatenation (object warnings then remote warnings) was already
   correct and is now pinned by
   `storage_reports_remote_warnings_after_local_object_warnings`.

### 5.3 Accounting gaps (charge before allocation)

7. **Remote generation vector was uncharged.** `scan_remote_object_generations`
   drained its `BTreeMap` into `Vec<RemoteGenerationStats>` with no charge, while
   the sibling object-type path charged the equivalent transient duplicate as
   Temporary. Now mirrors it: reserve `len * size_of::<RemoteGenerationStats>()`
   as Temporary before the vector is allocated, release after the map is gone.
8. **Per-remote path scratch under-declared by 5×.** One remote scope
   concurrently owns the parsed project root *plus* the four derived layout
   directories, each bounded by the same 16 KiB path envelope, but only one
   envelope was reserved. Now `REMOTE_PATH_OWNER_COUNT = 5` envelopes are
   reserved before `parse_cf_project_url` allocates anything.
9. **Vacuous assertion removed.** A `debug_assert!(processed)` on a value that
   was unconditionally `true` was replaced by explicit match arms that document
   the reference skip semantics and keep the budget passthrough.

### 5.4 Test-coverage gaps closed

10. **No human-readable equivalence oracle existed.** The pre-existing
    equivalence test only compared the encoder's bytes against a
    re-serialization of the *same* report, so the text surface had no oracle at
    all after `print_storage_report` was replaced by `write_storage_report`.
    Added `reference_text_bytes`, an independent re-implementation of the former
    `println!` rendering (including its own `format_bytes`), exercised over
    quick and full modes with a populated remote and non-empty warnings by
    `storage_decision_work_preserves_reference_text_bytes`.

Focused module count went 7 → 11.

### 5.5 Invariants exercised by the accepted gates

- Exact legacy text bytes: G9 (`..._preserves_reference_text_bytes`).
- Exact legacy JSON bytes, quick and full: G9
  (`..._preserves_quick_and_full_json_bytes`).
- One bounded decision-work owner for local *and* remote reporting: a single
  `DecisionWorkLedger` is created in `prepare_storage_report` and threaded by
  `&mut` through discovery, both object scans, every remote, encoding, and
  publication; no nested ledger is constructed. `StorageReportRun` is
  `#[must_use]` and its `Drop` asserts zero retained and zero temporary debt.
- Charge before allocation/read/growth for paths, generation vectors,
  raw/parsed records, response buffers, and retained/temporary owners: G9
  boundary tests plus items 7 and 8 above.
- Success and every failure class closes to zero with no partial output and no
  authority change: G9 (`..._never_publishes_partial_output` asserts the output
  buffer is empty on overflow; `..._finish_without_debt` and
  `assert_zero_concurrent_debt` assert zero residual current bytes; budget
  failures assert `terminal_reason = BudgetExceeded`, `observed = limit + 1`,
  exit 60, `continuation_supported = false`).
- Public limits not raised and tests not weakened: the delta adds no limit
  constant and relaxes no assertion; `DecisionWorkProfile::local_v1()` is
  unmodified, and the only new constant (`REMOTE_PATH_OWNER_COUNT`) *increases*
  a charge. G1/G2/G5/G10 show no legacy regression was loosened.

## 6. Residual

**Product residual: zero.** No known unfixed correctness, accounting, or test
defect remains in the storage packet at `b3bc32a`. No defect was documented in
place of being fixed.

**Evidence residual (process, not product):** the following gates are confirmed
on TC-A (Rust 1.97.1) against this exact source state but were **not**
re-confirmed on TC-B (Rust 1.97.0) because command `seq 94` was interrupted and
the recovery instruction forbids re-running Cargo:

- full `codefire-cli` binary suite (G1, 779/779),
- `remote::` and `http::` module suites (G2, 83/83 and 133/133),
- `cargo build --bin codefire-rs` as a standalone step (covered on TC-A by
  `seq 13` and implicitly by G1's compile),
- `check_source_docs.py` (G3) — re-runnable at zero build cost; it shells out
  only to `git ls-files`.

Both toolchains are Rust 1.97.x and satisfy MSRV 1.75, and the source state is
byte-identical across them (section 2 fence, plus G6/G8/G9/G10 passing on TC-B).
A cross-toolchain divergence in those four gates is therefore judged unlikely,
but it is **not proven**, and this record does not claim it.

**Not covered by this packet at all:** the Python fallback runtime, cross-runtime
conformance vectors, `cargo test --workspace`, the Python unittest suite, and the
Install/Dogfood gates. The packet changes no Python and no shared crate, but that
is an argument, not a measurement.

## 7. Standing and next action

This document is an **evidence record produced by the implementing agent**. It is
explicitly **not** an independent review, and it does **not** authorize
integration, merge, or Close for CFR-168 or for ADR-020 adoption.

**Remaining next action: independent adversarial review** of `b3bc32a` against
`f5bce30` and `40c095d`, by an agent that did not write this delta. Suggested
focus, in priority order:

1. The `IncompleteRecord`-by-bare-path discriminator in
   `object_record_failed_semantic_validation`. It is exact for every producer
   present in `codefire-store` today, but it is coupled to message-context
   shape rather than to a structural signal. A reviewer should decide whether
   `codefire-store` ought to expose the failing stage explicitly instead.
2. Whether the WIP's symlink hardening should be kept as reference-divergent
   behaviour or reverted. `remote_layout_issues` moved from `fs::metadata`
   (follows symlinks) to `symlink_metadata` + explicit symlink rejection, and the
   quick object scan moved from a symlink-following `read_dir` walk to a
   capability no-follow walk. Both make quick mode agree with full mode and
   downgrade a would-be hard `CliError::Io` into a reported `invalid_layout`, but
   both do change results for a symlinked directory. This delta preserved the WIP
   choice deliberately; it is a judgement call a reviewer should ratify or reject.
3. The declared portable accounting units — `STORAGE_OWNER_FRAME_BYTES`,
   the 4 KiB iterator/text scratch, `STORAGE_JSON_PARSE_BASE_BYTES`, the
   `2 * P + min(P, 64 KiB)` parse reservation, and using `retained_bytes` as the
   proxy for the JSON `Value` tree in `encode_json_output` — against ADR-020's
   logical-bytes definition.
4. Error precedence when a visitor error and a Store scan error coincide:
   `scan_objects` and `scan_remote_object_generations` return the visitor error
   and drop `scan_result`, which can mask a budget error arising during the
   scanner's own release path.
5. Confirmation of G1/G2/G3 on TC-B once Cargo targets can be rebuilt.

## 8. Reproduction

```sh
export RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup
export CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2

git checkout b3bc32a6bd4c792edc5a1302c61e16c624a9b648

cargo fmt --all -- --check
cargo clippy --package codefire-cli --all-targets --locked -- -D warnings
git diff --check 40c095d b3bc32a
python3 docs/development/tools/check_source_docs.py

cargo test --package codefire-cli --bin codefire-rs --locked tests::storage_decision_work::
cargo test --package codefire-cli --bin codefire-rs --locked -- --exact \
  tests::storage_report_counts_objects_by_type_and_warns_large_objects \
  tests::storage_report_counts_file_remote_project_storage \
  storage::tests::active_state_scan_does_not_follow_outside_symlinks \
  storage::tests::active_state_scan_rejects_symlinked_root
cargo test --package codefire-cli --bin codefire-rs --locked remote::
cargo test --package codefire-cli --bin codefire-rs --locked http::
cargo test --package codefire-cli --bin codefire-rs --locked
cargo build --package codefire-cli --bin codefire-rs --locked
```
