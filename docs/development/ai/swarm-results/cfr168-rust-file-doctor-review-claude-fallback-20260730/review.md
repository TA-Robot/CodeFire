# CFR-168 Rust File Doctor — Independent Review (Claude Fallback)

Review date: 2026-07-30

Reviewer: Claude Opus 5 (`claude-opus-5`)

Batch: `cfr168-rust-file-doctor-review-claude-fallback-20260730`

Continued contract:
`docs/development/ai/swarm-batches/cfr168-rust-file-doctor-independent-review-20260730.md`

**Verdict: `ACCEPT`.**

No Must-level finding. The live `cf remote doctor cf://…` route is unified onto the accounted
`diagnose_remote_project_with_work_and_poll` implementation under exactly one command-scoped
`DecisionWorkLedger`; the legacy unaccounted duplicate is `#[cfg(test)]`-only and therefore cannot
exist in a production build; all six required-review items were re-derived from the target and pass.
Four Should-level and four Could-level findings are recorded below; none of them makes the reviewed
target unsafe to integrate, and three of them are inherited from the previously accepted accounted
implementation that the HTTP route already ships.

This verdict was derived independently. The interrupted Fugu 1.1 Ultra session produced no report and
was not used as evidence; the implementation agent's reported clean commit and 775/775 result were
treated as claims and re-run, not inherited.

## Exact subject

| Role | Revision | Disposition |
|---|---|---|
| Baseline / task revision | `ea8f2a641b68d06fe4a266f304dd03daa1f4a7de` | comparison base |
| Completed target | `52a025ecda3fe2f5ad8b5b2a485562e7c16ab6de` | reviewed and accepted |
| Target branch | `fugu-agent/cfr168-rust-file-doctor-recovery-20260729-file-doctor-restart` | contains the target as tip |
| Contract | `docs/development/ai/swarm-batches/cfr168-rust-file-doctor-ledger-20260729.md` | requirements source |
| Reviewer worktree base | `9f1fe186df3002198a184fdced4691123037d71f` | report-only |

`ea8f2a6…` is a first-parent ancestor of `52a025e…`; the range is the single commit
`52a025e Adopt accounted ledger for live file remote doctor`. The whole diff is one file:

```text
crates/codefire-cli/src/remote/doctor.rs | 1304 +++++++++++--- (1272 insertions, 32 deletions)
```

The reviewer worktree base carries `crates/codefire-cli/src/remote/doctor.rs` byte-identical to the
baseline `ea8f2a6…`; the target is **not** an ancestor of `main` and is not integrated. All line
citations below are line numbers in `crates/codefire-cli/src/remote/doctor.rs` at `52a025e…`.

### Review environment disclosure

The exact target was already checked out, clean and detached at `52a025e…`, in the abandoned Fugu
reviewer worktree
`…/.fugu-second-agent/…/worktrees/cfr168-rust-file-doctor-independent-review-20260730-file-doctor-independent-review`.
Reviewer validations were executed there so they run against the exact target rather than a
reconstruction. No tracked file in that worktree was read-write touched; only gitignored `target/`
build artifacts were produced, and `git status --porcelain` at `52a025e…` is empty after every run.
The Claude reviewer worktree contains only this report.

## Required-review verdicts

| # | Item | Verdict |
|---|---|---|
| 1 | Single command ledger owns the whole live file route; no unobserved second decision owner | **PASS** |
| 2 | Every retained collection/path/document/projection/buffer/Store handoff reserved before use; exact / −1 / +1 are typed budget outcomes | **PASS** |
| 3 | Final subject fence detects same-metadata replacement before publication; every injected failure leaves zero debt, no permit, no semantic mutation | **PASS** (see SHOULD-2, COULD-1) |
| 4 | Quick/full behavior, text/JSON bytes, issue ordering, precedence, local/file compatibility, capability semantics unchanged | **PASS** (see COULD-2) |
| 5 | Test snapshot excludes only nondeterministic lock ownership metadata and compares every semantic byte | **PASS** (see SHOULD-4) |
| 6 | Reviewer independently ran focused Doctor tests, strict Clippy, format/diff checks, representative legacy Doctor tests | **PASS** (reviewer-executed) |

### Item 1 — single command ledger — PASS

The live route is `run_remote_doctor` → `run_file_remote_doctor` →
`run_file_remote_doctor_with_factory_and_hook` → `diagnose_remote_project_with_work_and_poll_and_observer`.

- `run_remote_doctor` line 866-867 replaces the former `Ok(Some(diagnose_remote_project(…)))` with
  `run_file_remote_doctor(target, quick).map(Some)`. The `cf://` arm has exactly one production
  callee.
- `run_file_remote_doctor` line 808-829 constructs one `DecisionWorkLedger` from
  `DecisionWorkProfile::local_v1()`, `DecisionWorkCommandKind::LocalGraph`, and
  `FILE_REMOTE_DOCTOR_COMMAND` (line 15). `local_v1().applicable_commands` contains `LocalGraph`
  (`crates/codefire-core/src/work_budget.rs:273-276`), so construction is valid.
- The ledger is created at line 615 **before** `parse_cf_project_url` (line 638) and therefore before
  target discovery, project gate acquisition, and layout inspection — as the contract requires.
- Ledger ownership is threaded, never duplicated: the object scanner receives `ledger.as_mut()`
  (line 1683-1686) and hands the same `&mut` to its visitor; the sealed-validation session takes the
  ledger by value via `ledger.take()` (line 1870-1871) and returns it via `take_decision_work_ledger`
  (line 1972, 1975) or `discard_and_take_decision_work_ledger` on every failure edge.
- `diagnose_remote_project_with_work_and_poll` (line 1979) is retained as a thin, behavior-preserving
  delegation with a no-op observer (line 1990-2000), so the two HTTP call sites
  (`crates/codefire-cli/src/http.rs:5811,14142,14169`) are unchanged. I verified the observer seam is
  genuinely inert on the no-op path: `with_provisional_decision_work_ledger`
  (`crates/codefire-store/src/lib.rs:6604-6684`) only samples retained/temporary/terminal baselines
  and re-checks them, charging nothing on success.
- No compatibility helper can be a production fallback. `diagnose_remote_project_legacy` (line 873),
  `sorted_json_paths`, `inspect_branches`, `inspect_merge_requests`, `inspect_objects`,
  `inspect_sealed_roots`, `RemoteDoctorReport::push_issue`, `MAX_REMOTE_RECORDS`, and the `BTreeSet`
  import are all `#[cfg(test)]`. `git grep` over the target tree shows no non-test caller of the
  legacy helper, and strict Clippy on the non-test `--bin` build is clean, which independently proves
  the release binary contains no second Doctor implementation.

`live_file_doctor_uses_one_local_ledger_across_scanner_and_validation` (line 2913) proves the factory
is called exactly once and that `matches_identity` holds at Entry, Scan, Validation,
FinalRevalidation, ResponsePreflight, and Prepublication, with the observed stage sequence pinned in
order, `is_prepublication_sealed()` true, and terminal reason `Complete`.

### Item 2 — reservation before allocation, typed boundaries — PASS

The report owner reserves before every growth: `reserve_growth` (line 303) charges `RetainedBytes`
before `set_project_exists`, `insert_layout`, `increment_counter`, `push_issue`, and
`push_skipped_check` mutate the report, and `release_shrink` (line 322) balances every shrink.
Scanner identifier maps (`object_ids`, `references`, `encountered_ids`, `roots`) are charged through
`retain_scanned_identifier` and released unconditionally after the scan closure (line 1817-1823),
which is why no failure path leaks them. Directory iteration charges a per-entry path frame through
`doctor_directory_entries` (line 973) before growth.

The publication sequence in `run_file_remote_doctor_with_factory_and_hook` is correctly ordered and
balanced:

1. `report_bytes = report.retained_bytes()` (line 680);
2. `PublicationPreflight` acquire of `report_bytes` plus a `ResponseBytesEncoded` count
   (line 688-700) — charged **before** the projection is computed or cloned;
3. `compact_json_len(report.report())` (line 709) and the equality check against `report_bytes`
   (line 730) reject any owner/payload mismatch as `InvalidRepository`;
4. `report.report().clone()` (line 746) is covered by the step-2 charge;
5. `report.release_owner` (line 747) then `seal_prepublication` (line 765), then release of
   `report_bytes` (line 777) and `ensure_file_remote_doctor_unowned` (line 787) before
   `finish(Complete)` (line 792).

This makes the true retained peak `2 × report_bytes` during steps 2-5.
`live_file_doctor_accepts_limit_minus_one_and_exact_then_rejects_plus_one` (line 3022) derives the
observed peak from a real baseline run and pins limit−1 / limit / limit+1 for both
`AggregateBytes` (cumulative) and `RetainedBytes` (concurrent, at that real peak), asserting the
typed `CliError::ResourceBudget` with exact `dimension`, `observed`, and `limit`, terminal reason
`BudgetExceeded`, and zero added ownership on the accepted side. Exact and minus-one succeed; only
plus-one fails. No corruption and no partial report is produced on the failing side, because the
report is released before the error escapes.

### Item 3 — final subject fence and failure cleanup — PASS

The fence is the sealed-validation session finalize: `session.finalize_with_cancellation`
(line 1964-1975) runs after every root has been validated and **before** the report is finished,
preflighted, or published — the publication sequence above contains no filesystem read after the
fence. The `FinalRevalidation` observation point (line 1946-1962) is placed before the finalize, so a
tamper injected there is exactly what the fence must catch.

`live_file_doctor_rejects_same_metadata_tamper_without_output_or_ownership` (line 3372) rewrites a
fixed-length marker in place and restores `atime`/`mtime`, then asserts identical `dev`, `ino`, `len`,
and `modified()` before and after — a true same-path/same-inode/same-size/same-mtime replacement —
and requires `StoreError::SealedValidationSubjectChanged { kind: DigestChanged, object_id: head }`,
no published bytes, zero current debt, zero open permits, and terminal reason `SubjectChanged`.

Failure-path invariants hold structurally, not only by assertion:

- `file_remote_doctor_terminal_reason` (line 511) maps deadline → `DeadlineExceeded`, budget →
  `BudgetExceeded`, `SealedValidationSubjectChanged` → `SubjectChanged`, else `CorruptSubject`.
- `finish_file_remote_doctor_failure` (line 531) is idempotent against a terminal reason already set
  by `fail_budget`.
- `DecisionWorkLedger::finish` rejects open permits (`work_budget.rs:1164-1170`), and
  `ensure_file_remote_doctor_unowned` (line 568) independently checks `RetainedBytes`,
  `TemporaryBytes`, and `open_permit_count` against the entry baseline — `finish` alone does not
  check current debt, so this separate check is load-bearing on the success path.
- `live_file_doctor_cancels_every_stage_without_publishing_or_leaking` (line 3168) injects a deadline
  at all six stages, and for each asserts the stage was actually reached, the error is typed
  `RequestDeadlineExceeded`, the rendered output buffer is empty, retained/temporary current are back
  to baseline, permits are zero, and terminal reason is `DeadlineExceeded`.

File Doctor performs no repository write on any path: it acquires only
`RemoteProjectGate::acquire_shared` (line 2063) — the same shared acquisition the retired legacy
helper already performed (line 897) — so the packet introduces no new mutation surface. See SHOULD-2
for the coverage gap on this property and COULD-1 for the fence's scope.

### Item 4 — behavior parity — PASS

`live_file_doctor_preserves_quick_full_bytes_diagnostics_and_missing_reports` (line 2974) builds a
fixture containing a healthy branch, a future-version branch record, and a truncated policy object,
then for both `quick` values compares the **live** `run_remote_doctor` result against
`diagnose_remote_project_legacy` on four axes: full struct equality (which pins issue ordering and
every counter), `serde_json::to_vec` byte equality, `remote_doctor_diagnostics_json` equality, and
rendered text byte equality. It then repeats the comparison for a missing project and pins
`issues[0].kind == "remote_project_missing"`. Transport stays `"file"` and `project` stays the
original URL.

I re-derived the parity surfaces the fixture does not reach and found no divergence:

- Truncation: legacy `RemoteDoctorReport::push_issue` (line 77-91) and accounted `push_issue`
  (line 416-455) both cap at `MAX_DIAGNOSTIC_SAMPLES = 128` and both set `diagnostics_truncated`.
  The public schema pins `issues.maxItems: 128` and requires `diagnostics_truncated`
  (`schemas/public/remote-doctor-v1.schema.json`).
- `ok` semantics: legacy sets `ok = issues.is_empty()` at the end; accounted `finish` (line 474)
  computes the same predicate with a balanced 1-byte release for the `false`→`true` narrowing.
- Unreadable branch directory: the accounted path pushes `remote_branch_directory_invalid` with the
  inner `std::io::Error`, legacy with `CliError::Io(e).to_string()`. These are the same string —
  `CliError::Io(error) => write!(f, "{error}")` (`crates/codefire-cli/src/main.rs:2142`).
- Text rendering: `print_remote_doctor_report` (line 2518) is refactored to `writeln!` over
  `write_remote_doctor_report` (line 2476) so tests can capture bytes. Line content and order are
  unchanged; the only difference is one stdout lock for the whole report instead of one per line,
  which is strictly more atomic.
- Capability/profile: `semantic_profile` still comes from
  `codefire_store::CURRENT_SEMANTIC_VALIDATION_PROFILE` (line 58, 244), and the HTTP client arm's
  capability/profile/mode cross-checks (line 848-863) are untouched.
- Local-target routing is unaffected: `run_remote_doctor` still returns `Ok(None)` for non-URL
  targets (line 869), and `tests::remote_doctor::local_doctor_target_is_not_claimed_by_remote_router`
  passes.

Exit behavior changes only in the failure direction, as the contract requires: a budget, deadline, or
fence failure now yields a typed `CliError` instead of an unaccounted report. That error shape already
existed on this function for the HTTP arm, so `main.rs:1115` handles a `?` from
`run_remote_doctor` on both transports identically.

### Item 5 — project-gate snapshot treatment — PASS

`snapshot_project_files` (line 2682) walks the entire project root recursively and records full byte
content per relative path. `assert_read_only_except_gate_bookkeeping` (line 2725) asserts both gate
bookkeeping files are still present after the run, then compares the two maps with **every other
file's bytes**, so an added file, a removed file, a renamed file, or any single changed byte outside
the two excluded paths fails the test. A leftover atomic-write temp file would also fail it, since
`publish_owner_in_directory` (`crates/codefire-cli/src/stable_lock.rs:218-240`) writes through a
uniquely named temp before rename.

The exclusion is genuinely necessary rather than cosmetic: the fixture's `before` snapshot precedes
any gate acquisition, so `locks/project-gate.lock` and `locks/project-gate.lock.owner.json` are
created by the run and would otherwise fail the file-set comparison. The excluded owner descriptor
carries only lock ownership metadata — `{version, pid, mode, created_at}` — and no semantic project
state, which is within item 5's allowance. The `.lock` file itself is always empty. See SHOULD-4 for
the one place the exclusion is broader than it needs to be.

### Item 6 — reviewer-executed validation — PASS

All results below were produced by this reviewer against `52a025e…`. The implementation agent's
775-case claim is corroborated, not inherited: I re-ran the full binary target myself.

## Commands and results

Toolchain: `cargo 1.97.1 (c980f4866 2026-06-30)`, `rustc 1.97.1 (8bab26f4f 2026-07-14)`, with
`RUSTUP_HOME=/tmp/codefire-rustup/rustup`, `CARGO_HOME=/tmp/codefire-rustup/cargo`.

| Command | Result |
|---|---|
| `cargo test -p codefire-cli --bin codefire-rs remote::doctor --locked` | **8 passed, 0 failed**, 767 filtered |
| `cargo test -p codefire-cli --bin codefire-rs doctor --locked` | **119 passed, 0 failed**, 656 filtered |
| `cargo test -p codefire-cli --bin codefire-rs --locked` | **775 passed, 0 failed**, 0 ignored |
| `cargo clippy -p codefire-cli --bin codefire-rs --locked -- -D warnings` | exit 0, no warning |
| `cargo fmt --all -- --check` | exit 0 |
| `git diff --check` | exit 0 |
| `python3 docs/development/tools/check_source_docs.py` | passed: 44 modules, 34 commands, 12 object kinds |
| `git status --porcelain` (exact-target worktree) | empty |
| `git rev-parse HEAD` (exact-target worktree) | `52a025ecda3fe2f5ad8b5b2a485562e7c16ab6de` |

The eight focused live-path cases, all passing:

```text
remote::doctor::tests::live_file_doctor_uses_one_local_ledger_across_scanner_and_validation
remote::doctor::tests::live_file_doctor_preserves_quick_full_bytes_diagnostics_and_missing_reports
remote::doctor::tests::live_file_doctor_accepts_limit_minus_one_and_exact_then_rejects_plus_one
remote::doctor::tests::live_file_doctor_cancels_every_stage_without_publishing_or_leaking
remote::doctor::tests::live_file_doctor_has_stable_one_three_ten_counter_curves
remote::doctor::tests::live_file_doctor_reuses_shared_dag_and_orders_missing_roots_deterministically
remote::doctor::tests::live_file_doctor_rejects_same_metadata_tamper_without_output_or_ownership
remote::doctor::tests::legacy_doctor_continues_one_session_after_a_corrupt_root
```

The representative legacy Doctor set includes all five `tests::remote_doctor` cases that drive the
live router (`file_remote_doctor_accepts_empty_healthy_current_layout`,
`file_remote_doctor_routes_urls_and_reports_missing_without_local_discovery`,
`file_remote_doctor_rejects_future_branch_record_before_semantic_use`,
`file_remote_doctor_reports_corrupt_object_as_bounded_typed_issue`,
`local_doctor_target_is_not_claimed_by_remote_router`) plus the local `doctor::` module suites — 119
cases in total, all passing on the exact target.

## Findings (severity-ordered)

### MUST — none

### SHOULD-1 — the production ledger's profile, command kind, and label are asserted by no test

`run_file_remote_doctor` (line 808-829) is the only place the canonical local profile is chosen, and
nothing pins it. The identity test does not use it: `run_live_with_ledger` (line 2576) and
`local_doctor_ledger` (line 2567) build a test-local duplicate of the production factory, and the
one profile assertion (line 2943-2946) checks `profile_id() == "codefire.decision-work.v1"`, which
cannot discriminate — `http_v1()` is built from `local_v1()` and never overwrites `profile_id`
(`work_budget.rs:294`), so both profiles report that same id. The tests that do enter through the
production factory (`run_remote_doctor`, line 2991/3014) only compare report content; the production
ledger is dropped behind a `debug_assert_eq!` on terminal reason (line 823-828) and is never
inspected.

Failure scenario: change `run_file_remote_doctor` to `DecisionWorkProfile::http_v1()`. Construction
still succeeds (`http_v1` inherits `LocalGraph` in `applicable_commands`), `ownership_phase` silently
flips to `Encoding`, and the `RetainedBytes` limit drops from 1 GiB to 32 MiB — a 32× tightening of
the live file Doctor's memory budget. Every existing test stays green, including the parity tests,
because small fixtures never approach either limit. The contract's "using the canonical local
profile" requirement is therefore unenforced.

Suggested closure: assert `profile_id`, `applicable_phases`, the `RetainedBytes` limit, and the
command label on a ledger obtained from the production factory itself, or expose the factory so a
test can pin it directly.

### SHOULD-2 — no failure injection asserts "no semantic repository mutation"

Item 3 requires that *every* injected budget/deadline/tamper/observer failure leave "no semantic
repository mutation," but the read-only snapshot assertion is used only on the success path:
`snapshot_project_files`/`assert_read_only_except_gate_bookkeeping` appear exactly once, at line 2986
and 3007 inside the quick/full parity test. The budget test (line 3022), the six-stage cancellation
test (line 3168), and the tamper test (line 3372) assert ledger, error-type, and output properties
only; the tamper test's filesystem assertions cover just the single tampered file's `dev`, `ino`,
`len`, and `mtime`.

Failure scenario: a later change that writes a repair, quarantine, or `.tmp` artifact on a Doctor
abort path — for example inside the session discard in `validate_doctor_roots_with_work`
(line 1900, 1937-1941, 1960) or in `observe_file_remote_doctor_failure` (line 592) — would leave a
stray file under the project root on every cancelled run and no test in the suite would fail. I found no such
write on the current target, so this is a coverage gap rather than a live defect.

Suggested closure: snapshot the project before each injected failure in the cancellation, budget, and
tamper tests and reuse `assert_read_only_except_gate_bookkeeping` on each.

### SHOULD-3 — local report growth is attributed to `PublicationPreflight`, misreporting the failing phase

`AccountedRemoteDoctorReport::new` (line 229-237) selects `ownership_phase = Encoding` when the
profile declares that phase and `PublicationPreflight` otherwise. `local_v1().applicable_phases`
omits `Encoding` (`work_budget.rs:277-286`), so on the file route every report retained-byte
reservation — layout insertion, counter widening, and each issue pushed during branch, merge-request,
object, and sealed-root inspection — is charged in phase `PublicationPreflight`, long before any
publication work begins. HTTP is unaffected: `http_v1` includes `Encoding`
(`work_budget.rs:327-333`), so that route's phase attribution is byte-for-byte what it was.

Failure scenario: a real `RetainedBytes` exhaustion while Doctor is scanning branch records produces
`CliError::ResourceBudget` whose `DecisionWorkBudgetExceeded.phase` is `publication_preflight`
(`apply_direct` passes `reservation.phase` straight into `fail_budget`, `work_budget.rs:768-786`).
That field is serialized into the CLI error payload, so an operator diagnosing an oversized project
is pointed at output publication rather than at record scanning. Secondarily,
`snapshot.phase_counters[PublicationPreflight]` now conflates report growth with the genuine
response-preflight charge at line 688-700, so the two cannot be separated after the fact. The
boundary test asserts `dimension`, `observed`, and `limit` but not `phase` (line 3093-3101,
3151-3159), so the misattribution is unasserted in either direction.

`local_v1` already declares `Validation`, `Query`, `GraphDiscovery`, `RecordRead`,
`SemanticValidation`, `Staging`, and `SnapshotRevalidation`; any of these would attribute
report-growth debt more truthfully than `PublicationPreflight`.

### SHOULD-4 — the gate exclusion is file-scoped, so the gate's deterministic fields go unchecked

`GATE_BOOKKEEPING_FILES` (line 2715-2718) excludes `locks/project-gate.lock.owner.json` in whole. The
descriptor is `{version, pid, mode, created_at}` (`stable_lock.rs:224-229`), of which only `pid` and
`created_at` are nondeterministic — `version` is a constant `1` and `mode` must always be `"shared"`
for a read-only Doctor.

Failure scenario: change line 2063 to `RemoteProjectGate::acquire_exclusive`. The live file Doctor
would then take an exclusive gate — serializing itself against all concurrent readers and against the
HTTP server's shared-gate paths, a real availability regression for a read-only diagnostic — and
`assert_read_only_except_gate_bookkeeping` would still pass, because the only file recording the
acquired mode is excluded wholesale. Nothing else in the suite pins the gate mode of the file route.

The in-code justification is also inaccurate on this point: the comment at line 2707-2714 states that
the descriptor's "`mode`, `pid`, and `created_at` fields legitimately change when the gate is
re-acquired." `pid` and `created_at` do; `mode` does not and must not. This is the single place where
item 5's "only nondeterministic lock ownership metadata" allowance is applied more broadly than the
data requires.

Suggested closure: compare `locks/project-gate.lock.owner.json` field-wise — assert `version == 1`
and `mode == "shared"`, and exclude only `pid` and `created_at`.

### COULD-1 — the final subject fence is object-store-scoped; branch and merge-request records sit outside it

The sealed-validation session's subject is the object store passed at line 1873-1877, so the finalize
fence revalidates commit records only. Branch and merge-request documents are read earlier by
`inspect_branches_with_work` (line 1292) and `inspect_merge_requests_with_work`, before the session
exists, and are never re-checked.

Failure scenario: after Doctor reads `branches/main.json` but before the report is published, replace
its `head` with a same-length value and restore `mtime`. The published report then describes the
sealed state of the *old* head and asserts `ok`, with no fence detecting the swap. This is inherited
from the accounted implementation already accepted for the HTTP route rather than introduced by this
packet, and item 3's fence requirement is met for the Store subject the session actually binds; I
record it so the boundary is explicit rather than assumed.

### COULD-2 — retiring the 131 072-record cap changes file-route behavior at extreme scale

`MAX_REMOTE_RECORDS` is now `#[cfg(test)]` (line 12-13) and the accounted branch/merge-request
inspection applies no `.take(MAX_REMOTE_RECORDS)` (line 1319 onward); scale is bounded by ledger
budgets instead.

Failure scenario: a project with more than 131 072 branch records previously produced a successful
report that silently inspected only the first 131 072 while reporting the full `branch_count`; it now
runs until a typed `AggregateBytes` or `ObjectRecords` budget outcome and publishes nothing. The new
behavior is what the contract asks for — typed outcomes instead of silent truncation, and it removes
an unbounded-memory scan that the old file route performed with no budget at all — but it is a
user-visible change on the file route that no test pins at or near the retired cap.

### COULD-3 — three cleanup paths use `?` on `release_owner`, bypassing terminalization and the failure observer

At line 682-685, 703-706, and 718-721 the code releases the report owner with
`report.release_owner(active_ledger).map_err(…)?`. If that release ever failed, the function would
return the *cleanup* error without calling `observe_file_remote_doctor_failure`, leaving the ledger
with no terminal reason and discarding the original cause. Every other failure edge in the function
terminalizes first. The paths are unreachable on the current target — the release amount was charged
immediately before and `release` only fails on measurement mismatch or underflow — so this is a
symmetry/robustness note, not a live defect.

### COULD-4 — the zero-debt check on failure paths is `debug_assert!` only

`observe_file_remote_doctor_failure` verifies zero retained debt via
`debug_assert!(ensure_file_remote_doctor_unowned(…).is_ok())` (line 601), which compiles out in
release builds. The explicit releases on each failure edge make the check redundant for correctness,
and the tests observe the same property through `assert_failure_unowned` on the recorded snapshot, so
this only means the invariant is unpoliced in a release binary. The success path uses the real
`ensure_file_remote_doctor_unowned` call at line 787, so publication is protected in all profiles.

### Note — retained test-only legacy oracle

Roughly 250 lines of `#[cfg(test)]` legacy Doctor code are retained purely as the parity oracle for
line 2988-2990 and 3013, plus the one legacy regression test at line 3462-3463. The contract
explicitly permits a retained compatibility helper that is clearly outside the production path, and
`#[cfg(test)]` is the strongest available form of that guarantee. The residual cost is that the
oracle is a mutable copy of retired behavior rather than a frozen golden fixture, so it can drift
with future edits without any signal.

## Residual risk

1. **Published bytes are broader than the preflighted projection.** The preflight charges and counts
   `compact_json_len(report)` (line 687-700), but `main.rs:1124-1135` publishes either the pretty
   `command_result_envelope` (larger than the compact report) or the text rendering (a different size
   entirely), after the ledger has been finished and dropped. The contract asks only that the report's
   own projected output be preflighted, which is satisfied; full CLI-envelope accounting is outside
   this packet and outside its write set.
2. **No fence in quick mode or when no root is reachable.** Quick runs skip the object graph
   entirely and record `object_hash_and_reference_graph` in `skipped_checks`; the
   `!has_encountered_root` branch (line 1840-1866) emits missing-root diagnostics without opening a
   session. Both are honest about what they did not check, and both predate this packet, but neither
   has a final subject fence.
3. **Observer plumbing is production-compiled and test-driven.** The `FileRemoteDoctorStage` seam,
   `run_file_remote_doctor_with_factory_and_hook`, and the two
   `with_provisional_decision_work_ledger` observation points exist in the shipped binary with
   inert `|_, _| Ok(())` closures. I verified inertness on both the CLI side and the Store side, and
   Clippy is clean, but the mechanism is exercised only by tests, so a future non-trivial hook would
   change the accounted behavior of the HTTP route as well as the file route.
4. **Verdict scope.** This report accepts `52a025e…` for the live file-remote Doctor route only. It
   does not accept the branch's integration into `main`, and it makes no statement about other
   CFR-168 Doctor surfaces (HTTP Doctor route, Python parity, capability advertisement) beyond
   confirming this change leaves the HTTP accounting path byte-identical.
5. **Not re-verified.** Workspace-wide `cargo test` beyond the `codefire-cli` binary target, the
   Python suites, and any cross-runtime parity gate were not run here; the ticket's focused and
   representative scope was completed in full.

## Next action

Manager owns integration. Recommended: integrate `52a025e…` as-is, and schedule SHOULD-1 through
SHOULD-4 as a follow-up test/attribution packet on `crates/codefire-cli/src/remote/doctor.rs` —
none of them requires reopening the production routing decision accepted here.
