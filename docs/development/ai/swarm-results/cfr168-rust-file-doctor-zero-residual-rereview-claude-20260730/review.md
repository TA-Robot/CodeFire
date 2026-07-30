# CFR-168 Rust File Doctor Zero-Residual — Independent Rereview (Claude)

Review date: 2026-07-30

Reviewer: Claude Opus 5 (`claude-opus-5`)

Batch: `cfr168-rust-file-doctor-zero-residual-rereview-claude-20260730`

Task: `file-doctor-zero-residual-rereview` (role `reviewer`, tier `ultra`)

Repair contract:
`docs/development/ai/swarm-batches/cfr168-rust-file-doctor-zero-residual-repair-20260730.md`

**Verdict: `ACCEPT`.**

All eight required residuals are closed on the live production route. Every claimed oracle was
re-executed and then deliberately regressed: eight independent single-line regressions of production
code were injected and each produced a targeted test failure, so no verdict below rests on a vacuous
assertion. No Must-level finding. Three low-severity notes and one contract-excluded item are
recorded; none is a material unowned finding in this scope.

This verdict was derived independently. The implementer's reported gate results were treated as
claims and re-run, not inherited.

## Recomputed subject identity and clean state

Every identity in the ticket was recomputed from the worktree before any review work.

| Claim | Ticket value | Recomputed | Match |
|---|---|---|---|
| Exact candidate | `1a04490806b59f8c39b5c1cf03f42d06aa750a98` | `git rev-parse HEAD` → same | yes |
| Candidate tree | `eafb2cbf20980944d16086a972100c321a8b56be` | `git rev-parse HEAD^{tree}` → same | yes |
| Repair task base | `dbeaac1526acc8c2e102ee2b137e8f2a708890b0` | `git merge-base --is-ancestor` → ancestor of HEAD | yes |
| Accepted predecessor | `52a025ecda3fe2f5ad8b5b2a485562e7c16ab6de` | `git merge-base --is-ancestor` → ancestor of HEAD | yes |
| Binary patch SHA-256 | `5f150dbe…c37427fac` | `git diff --binary dbeaac1 HEAD \| sha256sum` → `5f150dbe37b61b592fa5a52e9ce72a4367ba7c8de2c09eba79390a1c37427fac` | yes |
| Prior accepted-review report SHA-256 | `c1b39ffc…5b86ed8045` | `sha256sum …/cfr168-rust-file-doctor-review-claude-fallback-20260730/review.md` → `c1b39ffc6f25cf9f19007af3cd5336421400e467e6e3c28969de775b86ed8045` | yes |
| Repair-contract SHA-256 | `732f0703…cbefbc57abe6c7375` | `sha256sum …/cfr168-rust-file-doctor-zero-residual-repair-20260730.md` → `732f070322f5452619c8a492b34174399a2aa9d06935440cbefbc57abe6c7375` | yes |

`git diff --name-only dbeaac1 HEAD` is exactly the five declared paths, in the declared order:

```text
crates/codefire-cli/src/remote/doctor.rs
docs/development/change-playbook.md
docs/development/command-source-trace.md
docs/development/source-architecture.md
docs/development/source-data-model-catalog.md
```

`git diff --shortstat dbeaac1 HEAD` → `5 files changed, 2132 insertions(+), 266 deletions(-)`,
matching the ticket's 2,132/266 delta. `git status --porcelain` was empty at review start.

No moving substitute was reviewed. Eight deliberate regressions were applied and reverted during the
audit; after the last revert the patch digest was recomputed and is still
`5f150dbe37b61b592fa5a52e9ce72a4367ba7c8de2c09eba79390a1c37427fac`, `git rev-parse HEAD` is still
`1a04490…`, and `git status --porcelain` is empty. `crates/codefire-cli/src/remote/doctor.rs` hashes
`465ac999bb7eeb22775737a5f17ec30a8b8249aacefc50c8ab4e3e5fb6d2cf0f`. The candidate was never changed
to make a probe pass; the two scratch build directories (`target/clippy-review`,
`target/release-review`) were removed after use and are gitignored in any case.

All line citations below are line numbers in `crates/codefire-cli/src/remote/doctor.rs` at
`1a04490…`.

## Production reachability

The `cf … doctor` call graph was traced rather than assumed:

```text
main.rs:1115  run_remote_doctor(&target, options.quick)?
  doctor.rs:1414  is_cf_http_url  → HTTP delegation (unchanged)
  doctor.rs:1445  target.starts_with("cf://")
    doctor.rs:1394  run_file_remote_doctor
      doctor.rs:1395  run_file_remote_doctor_with_factory_and_observer(
                        target, quick,
                        new_file_remote_doctor_ledger,      // doctor.rs:1385
                        InertFileRemoteDoctorObserver)      // doctor.rs:603
```

`grep` over `crates/` confirms `run_remote_doctor` has exactly one non-test caller (`main.rs:1115`)
and that `new_file_remote_doctor_ledger` (doctor.rs:1385) is the sole ledger factory on the `cf://`
route. The HTTP server side enters through `diagnose_remote_project_with_work_and_poll`
(doctor.rs:2654), which passes `InertFileRemoteDoctorObserver` and
`DoctorRecordSubjectFence::Disabled` — its accepted accounting is preserved by construction.

Test seams are separated from production behavior and are all `#[cfg(test)]`:
`HookFileRemoteDoctorObserver` (620), `run_file_remote_doctor_with_factory_and_hook` (3280),
`diagnose_remote_project_legacy` (1452), `MAX_REMOTE_RECORDS` (33). The production observer is a
zero-sized type whose `INERT` associated constant is `true`, and every observation point is behind
`if O::INERT { return Ok(()) }` (doctor.rs:641-653), so the shipped build contains no observation
call. Critically, the test helper `local_doctor_ledger()` (3274) is no longer a test-local duplicate
— it now calls `new_file_remote_doctor_ledger()` directly, so the whole suite drives the production
factory.

## Verdicts for the eight required residuals

| # | Residual | Verdict |
|---|---|---|
| 1 | production factory / profile / command kind / label asserted through the real entry point | **PASS** |
| 2 | byte-identical tree, zero debt, no open permit, one terminal observation on every injected failure | **PASS** |
| 3 | report-growth charged to its real bounded phase | **PASS** |
| 4 | gate fields independently observable and non-vacuous | **PASS** (scope boundary noted) |
| 5 | final subject fence covers every claimed authority, hooks after last observation | **PASS** |
| 6 | retired 131,072-record cap bounded by the declared contract at every boundary | **PASS** |
| 7 | fallible cleanup preserves the primary result and cannot bypass terminalization | **PASS** |
| 8 | zero-debt / zero-permit closure enforced in production, not only `debug_assert!` | **PASS** |

### Residual 1 — production factory, profile, command kind, label — PASS

`file_remote_doctor_profile()` (1380) and `new_file_remote_doctor_ledger()` (1385) are the single
production owners; `FILE_REMOTE_DOCTOR_COMMAND_KIND` (38) and `FILE_REMOTE_DOCTOR_COMMAND` (35) are
named constants rather than restated literals.

`live_file_doctor_pins_its_production_ledger_factory` (4342) obtains its ledger from the production
factory and uses a discriminator strictly stronger than the profile ID. It first asserts
`local.profile_id() == http.profile_id()` (4347) — proving the ID cannot discriminate — then asserts
whole-profile structural equality `ledger.profile() == &local` **and** `ledger.profile() != &http`
(4348-4349). It additionally pins the exact eight-phase list (4352-4364), the absence of `Encoding`
(4365-4368), the 1 GiB `RetainedBytes` limit against `http_v1()`'s 32 MiB (4369-4383), the command
kind, the literal label `"file remote doctor"`, and `applicable_commands` membership (4384-4392).
Finally it observes both label and profile end to end through a real typed `ResourceBudget` outcome
raised on the live route (4397-4409).

**Deliberate regressions.** R1: `file_remote_doctor_profile()` → `DecisionWorkProfile::http_v1()` →
4 tests fail (`…pins_its_production_ledger_factory`, `…attributes_report_growth_to_its_own_phase`,
`…uses_one_local_ledger_across_scanner_and_validation`,
`…accepts_limit_minus_one_and_exact_then_rejects_plus_one`). R7: `FILE_REMOTE_DOCTOR_COMMAND` →
`"remote doctor"` → 1 test fails. The prior review's exact failure scenario is now caught.

### Residual 2 — byte-identical semantic tree, zero ownership, one terminal observation — PASS

`assert_read_only_project_tree` (3494) excludes **nothing** wholesale. It asserts both gate
bookkeeping files still exist, asserts the gate lock inode is empty, compares the owner descriptor
field-wise, and then compares the complete remaining tree as two `BTreeMap<PathBuf, Vec<u8>>` — so
an added, removed, or mutated file anywhere under the project root fails. `expected_mutations` names
only paths the *test itself* rewrote, with their expected bytes, so an injected tamper is still
compared byte-for-byte everywhere else (3537-3554). A `GateAcquisition::NotAcquired` variant
(3520-3529) additionally requires that a run aborting before gate acquisition rewrites neither gate
file.

Coverage is now on every injected-failure suite, not just the success path:

- `live_file_doctor_cancels_every_stage_without_publishing_or_leaking` (4004) — all 7 stages
  (`Entry`, `Scan`, `Validation`, `FinalRevalidation`, `RecordFence`, `ResponsePreflight`,
  `Prepublication`) × quick/full, each with a fresh `before` snapshot and a full tree comparison
  (4027, 4066-4077).
- `live_file_doctor_accepts_limit_minus_one_and_exact_then_rejects_plus_one` (3832) — tree compared at
  4000 after all six budget boundary runs.
- `live_file_doctor_binds_retired_record_cap_to_typed_budget` (4821) — tree compared at 4915.
- `live_file_doctor_fences_every_record_document_before_publication` (4578) — tree compared per tampered
  document at 4680-4691.

Zero ownership and the single terminal observation are checked from the observation itself, not from
a test-only side channel: `record_failure_evidence` (3374) captures the ledger snapshot and
`open_permit_count()` at exactly the `Failure` stage, and `assert_failure_unowned` (3352) asserts
`RetainedBytes == baseline`, `TemporaryBytes == 0`, and `open_permits == 0`. Terminal reasons are
pinned per failure class (`DeadlineExceeded`, `BudgetExceeded`, `SubjectChanged`). Terminalization
happens at most once: `finish_file_remote_doctor_failure` (939) only finishes when
`terminal_reason.is_none()`, and every failure edge returns immediately after notifying.

**Deliberate regression.** R5 injected the prior review's exact failure scenario — a stray
`doctor-abort.tmp` written under the project root on the diagnosis failure edge (2899) — and 5 tests
failed. Under the predecessor this write was invisible to the suite.

### Residual 3 — report-growth phase attribution — PASS

`DoctorGrowthSite` (157) names where a growth actually happens and `DoctorGrowthSite::phase()` (173)
maps it: `ReportFrame`/`LayoutInspection` → `Validation`, `RecordScan` → `RecordRead`,
`ReferenceGraph` → `GraphDiscovery`, `SealedValidation` → `SemanticValidation`, `SkippedCheck` →
`Query`. `AccountedRemoteDoctorReport::growth_phase` (345) applies the site phase unless the profile
declares the HTTP `Encoding` phase, so the accepted HTTP attribution stays byte-identical. Every
growth site in production code passes a real site rather than one constant — all 31 occurrences were
enumerated: `LayoutInspection` at 419/444/2737/2757/2776/2801, `SkippedCheck` at 546, `RecordScan`
across branch and merge-request scanning (1936-2034, 2096-2201) and the object scanner
(2358/2398/2405/2435), `ReferenceGraph` at 2449/2460, `SealedValidation` at 2511/2522/2594/2601, and
`ReportFrame` at 324. The two generic helpers `increment_counter` (472) and `push_issue` (529) take
the site as a parameter, so no call site can silently fall back to a default.

`live_file_doctor_attributes_report_growth_to_its_own_phase` (4488) is non-vacuous in the way the
ticket requires: `report_growth_failure_phase` (4437) exhausts the retained budget and reads the
phase out of the **real typed budget payload** (`error.budget_exceeded()…phase`), not out of the
enum. It checks all six sites under `local_v1()`, asserts each `!= PublicationPreflight`, and
asserts all six map to `Encoding` under `http_v1()`. End to end it then asserts
`phases[PublicationPreflight][RetainedBytes] == compact_json_len(report)` exactly — publication
preflight carries only the projected response and nothing else (4556-4560) — while `Validation` and
`RecordRead` are non-zero and `Encoding` is absent entirely.

**Deliberate regression.** R2: `Self::RecordScan => …PublicationPreflight` → 1 targeted failure.

### Residual 4 — gate acquisition and terminal fields — PASS (with a stated scope boundary)

The wholesale exclusion is gone. `GATE_OWNER_MASKED_FIELDS` is exactly `["pid", "created_at"]`
(3438) and `assert_shared_gate_owner_descriptor` (3448) asserts the descriptor's exact key set
`["created_at","mode","pid","version"]` (so a gained or lost field fails), `version == 1`, and `mode
== "shared"` with the message *"a read-only file doctor must acquire the project gate shared"*. The
lock inode is asserted present and empty (3512-3515). The prior review's inaccurate in-code comment
claiming `mode` legitimately varies has been replaced by a correct one (3435-3437).

**Deliberate regression.** R3: production `RemoteProjectGate::acquire_shared` → `acquire_exclusive`
at line 2746 (the legacy `#[cfg(test)]` oracle's identical call at 1476 left alone) → **6 tests
fail**, each on the exact assertion `a read-only file doctor must acquire the project gate shared`.
The prior review's availability-regression scenario is now caught.

*Scope boundary:* gate *wait*, *exclusion*, and *stable identity/release* mechanics are implemented
in `crates/codefire-cli/src/remote/locks.rs` and `stable_lock.rs`, which are outside this write set
and byte-unchanged by this candidate. What this packet can and does make independently observable is
the Doctor's own acquisition mode, the descriptor's authoritative fields, and the gate's read-only
footprint. I record this so the boundary is explicit rather than assumed.

### Residual 5 — final subject fence coverage — PASS

`DoctorRecordSubjectFence` (697) pins each consumed document by read length, `StableFsIdentity`,
**and** SHA-256 of the exact bytes that were parsed — the digest is computed from the same buffer
the parse consumes (1763-1783), so pin and parse cannot disagree. `verify` (778) re-reads through
the same bounded no-follow reader and rejects `Unreadable`, `SizeChanged`, identity change, and
`DigestChanged` as a typed `StoreError::SealedValidationSubjectChanged`, which
`file_remote_doctor_terminal_reason` (919) maps to `SubjectChanged`. Unparseable documents are
fenced too (1797-1799), because the report's "invalid record" conclusion derives from those exact
bytes.

Placement is correct: `fence.verify` runs at 2875, after the last subject read, guarded by a
`RecordFence` observation at 2870, and **before** `report.finish` (2877), the response preflight
(1178), and publication. Quick mode and the unreachable-root path both reach it — neither opens a
sealed-validation session, so this is their only fence.

The tests use mutation hooks placed after the last observation, which is exactly the shape the
ticket demands. `live_file_doctor_fences_every_record_document_before_publication` (4578) rewrites
the document *at the `RecordFence` stage* (4624-4626), then asserts `dev`, `ino`, `len`, and `mtime`
are all unchanged (4636-4639) — a genuine same-inode/same-size/same-mtime replacement — and that the
run publishes nothing, returns the typed change with the correct
`object_id`/`path`/`kind`/`expected_size`/`actual_size`, terminalizes `SubjectChanged`, and leaves
the tree otherwise byte-identical. It runs for both branch and merge-request documents and for both
quick and full modes. `live_file_doctor_fences_record_documents_without_a_reachable_root` (4702)
proves the unreachable-root path is fenced and asserts (by panicking in the hook, 4725-4727) that no
validation session is opened there. `record_subject_fence_pins_every_consumed_document_and_balances`
(4758) proves the fence pins exactly the consumed set (`fence.len() == 2`) and balances its own
retained ownership.

Object records, generations, and signatures remain fenced by the object-store scanner's
membership/identity fence and the sealed-validation session's retained-document digest fence — the
mechanism accepted in `52a025e…` and unchanged here. The candidate closes the branch/merge-request
gap the prior review recorded as COULD-1, and the source documents describe the split accurately
rather than claiming a single fence owns everything.

**Deliberate regression.** R4: live route `DoctorRecordSubjectFence::enabled()` → `Disabled` (1143)
→ 4 tests fail, including `http_delegation_keeps_encoding_attribution_and_records_no_fence`, which
proves the HTTP/file split itself is asserted in both directions.

### Residual 6 — retired 131,072-record cap — PASS

`MAX_REMOTE_RECORDS` is `#[cfg(test)]` (32-33) and the compatibility decision is documented at the
constant (12-31), which is where the contract asked for it.
`live_file_doctor_binds_retired_record_cap_to_typed_budget` (4821) asserts the canonical profile's
`ObjectRecords` limit is `cumulative(100_000)`, that `MAX_REMOTE_RECORDS == 131_072`, and —
decisively — that `100_000 < 131_072`, so the retired cap is *strictly unreachable* and silent
truncation cannot happen. It also asserts the retired `remote_record_budget_exceeded` diagnostic
kind is never emitted.

Boundaries are pinned without an impractical fixture, as the contract permits:

- `ObjectRecords`: exact (`remaining == consumed`, publishes and lands exactly on the limit, report equal
  to baseline), minus-one (`consumed - 1`), and first-read exhaustion (`remaining == 0`, phase pinned to
  `RecordRead`). Both failures assert `dimension`, `limit`, `observed == limit + 1`,
  `phase != PublicationPreflight`, empty published output, zero ownership, and `BudgetExceeded`.
- `AggregateBytes` and `RetainedBytes`: exact, minus-one, and plus-one each
  (`live_file_doctor_accepts_limit_minus_one_and_exact_then_rejects_plus_one`, 3832), with the failing
  phase asserted to be the read that actually charged it (`SnapshotRevalidation`), not publication.
- Cancellation at scale is covered by the seven-stage cancellation suite.

**Deliberate regression.** R8: removing `.charge(ObjectRecords, 1)` from `read_doctor_json` (1756) →
2 tests fail, proving the record accounting that bounds the retired cap is itself asserted.

### Residual 7 — fallible cleanup preserves the primary result — PASS

All three `report.release_owner(active_ledger).map_err(…)?` escapes the prior review flagged are
gone (confirmed in the deletion side of the diff). They are replaced by
`release_failed_file_remote_doctor_report` (1352) and `release_failed_file_remote_doctor_projection`
(1362), which `drop` the secondary result so the primary cause is never replaced. Every failure edge
in `run_file_remote_doctor_with_factory_and_observer` now follows one order — release what it can,
terminalize, drain, notify the observer, return the original error — at 1109, 1123, 1150, 1170,
1194, 1211, 1230, 1253, 1273, 1289, 1304, 1314, 1327. The diagnosis-level edge does the same
(2890-2902), and any residual a failed release leaves behind is repaired by the drain before the
observer is notified, so cleanup failure cannot bypass terminalization.

The single release that is genuinely the *primary* operation rather than cleanup — releasing the
report owner after a successful projection (1242) — is documented as such (1239-1241) and still
terminalizes and notifies like every other edge. That distinction is correct, not an escape.

### Residual 8 — zero-debt / zero-permit closure enforced in production — PASS

The `debug_assert!` is gone. `enforce_file_remote_doctor_unowned` (1008) is unconditional production
code: it computes the residual above the entry baseline for `RetainedBytes` and `TemporaryBytes`,
issues real `ledger.release` calls (legal after terminalization), reports what it drained via
`FileRemoteDoctorDebtDrain`, and re-checks with `ensure_file_remote_doctor_unowned`. Open permits
cannot be closed without their token, so an unrepairable violation aborts loudly *after* the
observer has been notified (1078-1083) rather than being returned in place of the cause.

**Decisive release-build evidence.** `cargo test --release -p codefire-cli --bin codefire-rs
remote::doctor --locked` → 20 passed, 0 failed. The critical one is
`file_remote_doctor_failure_cleanup_rejects_unrepairable_ownership` (5009), a `#[should_panic]`
test: under the predecessor's `debug_assert!`-only enforcement the panic compiles out in release and
this test would **fail**. It passes, so the invariant is genuinely policed in a shipped binary.
`file_remote_doctor_failure_cleanup_drains_injected_residual_debt` (4922) additionally proves the
observer already sees the drained ledger and that a balanced edge drains nothing.

**Deliberate regression.** R6: neutering the drain (early `return Ok(drain)`) → both cleanup tests
fail.

### Contract items 9 and 10 (not among the eight, verified for completeness)

Item 9 (observer seam): the fallible closure plumbing is now `#[cfg(test)]`-only; production ships
the zero-sized `InertFileRemoteDoctorObserver` behind a compile-time `INERT` guard.
`file_remote_doctor_inert_observer_cannot_affect_the_route` (5032) proves inertness by `const`
assertion (`INERT`, `size_of == 0`, `!needs_drop`) and proves behavioral equivalence by asserting
the shipped route and an observed route agree on both the published report and the *complete* ledger
snapshot.

Item 10 (frozen parity oracle): `GOLDEN_HEALTHY_FULL_JSON`, `GOLDEN_HEALTHY_QUICK_JSON`,
`GOLDEN_MISSING_FULL_JSON`, and `GOLDEN_HEALTHY_FULL_TEXT` (5164-5167) are independently pinned
literals compared against **both** the live route and the legacy oracle (5180, 5220), so changing
the oracle and the implementation together can no longer silently redefine compatibility. The golden
tests enter through `run_remote_doctor`, the true production entry point.

## Additional delta inspection

Inspected across the 2,132/266 delta:

- **Duplicated owners** — none introduced; the reverse happened. `local_doctor_ledger()` (3274) no longer
  duplicates the production factory, and `file_remote_doctor_profile()` is the single profile owner.
- **Dead compatibility code** — `MAX_REMOTE_RECORDS`, `diagnose_remote_project_legacy`, and the four
  legacy inspectors are all `#[cfg(test)]`; Clippy with `-D warnings` on `--all-targets` is clean, so no
  unreachable production item survives.
- **Deleted helpers** — the deletion side of the diff is the three `?` cleanup escapes, the
  `debug_assert!`, and the hook→observer refactor. No whole helper was deleted to make an assertion pass.
- **Unbounded retained state** — the fence's `Vec<DoctorRecordSubject>` charges every pin to
  `RetainedBytes` before pushing (751-770) and is bounded both by the 1 GiB retained limit and by the
  100,000 cumulative `ObjectRecords` limit that every record read charges.
- **Changed wire/CLI behavior** — the HTTP branch of `run_remote_doctor` (1414-1443) is untouched; JSON
  and text output are frozen against golden literals and confirmed byte-identical to the legacy oracle.
- **Error-precedence drift** — `read_doctor_json` inlines `read_json` in order to hash the exact parsed
  bytes. I verified this is not a behavior change: `main.rs:12620`'s `read_json` calls
  `secure_worktree::read_file_nofollow_bounded`, which is itself a one-line wrapper over
  `read_file_nofollow_bounded_with_identity` (`secure_worktree.rs:1285-1287`) — same reader, same
  `REPOSITORY_JSON_BYTE_LIMIT`, same two typed failures in the same order.
- **Source-document overclaim** — the four document edits were read against the code. The specific claims
  ("single production factory before URL parsing/gate/layout", "record limit below the retired cap",
  "phase that performs it", "quick and unreachable-root modes", "object records stay fenced by the
  scanner and session", "HTTP delegation keeps `encoding` and records no fence", "drains in every build")
  are each backed by code and a test verified above. The `command-source-trace.md` entry correctly leaves
  the other CFR-168 Doctor residuals listed as still open.

## Findings (severity-ordered)

### MUST — none

### SHOULD — none

### NOTE-1 — new production `panic!` on the failure-cleanup edge

`observe_file_remote_doctor_failure` (doctor.rs:1082) aborts the process when the drain cannot
restore zero ownership. This is a deliberate, documented consequence of closing residual 8 in
release builds, and its ordering is right: terminalize → drain → notify observer → abort.
Reachability is narrow — the CLI Doctor production path takes no permit of its own (`grep` finds no
`ledger.reserve(` in the module's production code; the ledger is only lent to Store calls that
return it), and the retained/temporary drain computes `current - baseline`, which cannot underflow.
So the panic fires only on an internal accounting bug or a Store call returning a ledger with an
open permit.

*Reproducer:* `file_remote_doctor_failure_cleanup_rejects_unrepairable_ownership` (doctor.rs:5009)
holds a permit across the edge and observes the abort. Behavior in debug builds is unchanged from
the predecessor (the old `debug_assert!` also panicked); only release builds changed, from silently
continuing to aborting. Aborting is the safer of the two given the invariant, so I record this as a
boundary the manager should be aware of rather than a defect.

### NOTE-2 — one transient unreserved allocation per fenced record

`read_doctor_json` allocates the 64-byte hex digest at doctor.rs:1781-1783 before
`DoctorRecordSubjectFence::record` reserves its retained frame at doctor.rs:751; likewise
`path.display().to_string()` at doctor.rs:737 is allocated inside the measurement call. The
unreserved window holds exactly one such value at a time and it is charged immediately afterward via
`logical_retained_frame_bytes`, so nothing accumulates. This mirrors the pre-existing accepted read
path rather than being introduced by this packet.

*Reproducer:* read doctor.rs:1779-1783 against doctor.rs:736-762. No test can currently observe it
because the amount is fixed and immediately charged.

### NOTE-3 — a document unreadable at scan time is not fenced

`read_doctor_json` returns `DoctorJsonRead::Invalid` without recording a fence subject when
`fs::symlink_metadata` fails (doctor.rs:1729-1731) or when the bounded read fails
(doctor.rs:1767-1777); only the *unparseable* case is fenced (doctor.rs:1797-1799). A document that
was unreadable during the scan and becomes readable before publication would therefore leave its
"invalid"/absent conclusion unrevalidated.

*Reproducer:* make `branches/main.json` unreadable, run the live route to get the invalid-record
diagnostic, then restore permissions before the `RecordFence` stage; the report still publishes.
There are no parsed bytes to pin in this case, so I read this as a scope boundary of a
content-digest fence rather than a defect — recorded so it is explicit rather than assumed closed.

### Contract-excluded — broader CLI envelope / text-output accounting

The prior review's residual 1 (published envelope bytes are broader than the preflighted projection,
`main.rs:1124-1135`) is intentionally owned by the command-wide response-projection packet. The
repair contract forbids touching `main.rs` here and forbids claiming it closed; the candidate honors
both. Not counted against this verdict.

## Commands and results

Environment: `RUSTUP_HOME=/tmp/codefire-rustup/rustup`, `CARGO_HOME=/tmp/codefire-rustup/cargo`,
`PATH="$CARGO_HOME/bin:$PATH"`. Toolchain `cargo 1.97.1 (c980f4866 2026-06-30)`, `rustc 1.97.1
(8bab26f4f 2026-07-14)`. 14 CPUs. Default target path is the reviewer worktree's `target/`; Clippy
and the release run used `target/clippy-review` and `target/release-review`, both deleted after use.

Focused cases first, then the authorized gates:

| # | Command | Result | Duration |
|---|---|---|---|
| 1 | `cargo test -p codefire-cli --bin codefire-rs remote::doctor --locked` | ok — 20 passed, 0 failed, 767 filtered | 0.62 s (2 m 20 s incl. build) |
| 2 | `cargo test -p codefire-cli --bin codefire-rs doctor --locked` | ok — 131 passed, 0 failed, 656 filtered | 2.32 s (32 s total) |
| 3 | `cargo test -p codefire-cli --bin codefire-rs --locked` | ok — 787 passed, 0 failed, 0 filtered | 30.50 s |
| 4 | `cargo clippy -p codefire-cli --all-targets --locked -- -D warnings` | clean | 1 m 32 s |
| 5 | `cargo fmt --all -- --check` | clean (exit 0) | — |
| 6 | `python3 docs/development/tools/check_source_docs.py` | passed — 44 modules, 34 commands, 12 object kinds | — |
| 7 | `git diff --check` | clean (exit 0) | — |
| 8 | `cargo test --release -p codefire-cli --bin codefire-rs remote::doctor --locked` | ok — 20 passed, 0 failed, 766 filtered | 0.18 s (6 m 13 s incl. build) |

The 787 (debug) vs 786 (release) binary-wide count differs by exactly one `#[cfg(debug_assertions)]`
test at `crates/codefire-cli/src/http.rs:17514`, outside Doctor scope.

### Deliberate regressions (each applied to production code, then reverted)

| # | Injected regression | Site | Result |
|---|---|---|---|
| R1 | `file_remote_doctor_profile()` → `http_v1()` | doctor.rs:1381 | 4 failed / 16 passed |
| R2 | `DoctorGrowthSite::RecordScan` → `PublicationPreflight` | doctor.rs:178 | 1 failed / 19 passed |
| R3 | production gate `acquire_shared` → `acquire_exclusive` | doctor.rs:2746 | 6 failed / 14 passed |
| R4 | live route fence `enabled()` → `Disabled` | doctor.rs:1143 | 4 failed / 16 passed |
| R5 | stray `doctor-abort.tmp` written on the diagnosis failure edge | doctor.rs:2899 | 5 failed / 15 passed |
| R6 | zero-debt drain neutered (early return) | doctor.rs:1012 | 2 failed / 18 passed |
| R7 | command label → `"remote doctor"` | doctor.rs:35 | 1 failed / 19 passed |
| R8 | record read no longer charges `ObjectRecords` | doctor.rs:1756 | 2 failed / 18 passed |

Every regression produced a targeted failure; none was silently absorbed. After the final revert the
candidate's patch digest, tree, and `git status` were re-verified identical to the recomputed
subject.

### Evidence not obtained

- Workspace-wide `cargo test` beyond the `codefire-cli` binary target, the Python suites, and any
  cross-runtime parity gate were not run. They are outside the ticket's focused and authorized scope; the
  prior accepted review recorded the same boundary.
- Gate *wait* and *exclusion* semantics were not exercised concurrently (owner: `remote/locks.rs` /
  `stable_lock.rs`, unchanged and outside this write set) — see the residual 4 scope boundary.
- Passing worker-reported gates were treated as context only; every result in the table above was
  executed by this reviewer against the exact candidate.

## Integration and write-overlap map

| Path | Candidate `1a04490…` | This review | Overlap |
|---|---|---|---|
| `crates/codefire-cli/src/remote/doctor.rs` | modified | read-only | none |
| `docs/development/change-playbook.md` | modified | read-only | none |
| `docs/development/command-source-trace.md` | modified | read-only | none |
| `docs/development/source-architecture.md` | modified | read-only | none |
| `docs/development/source-data-model-catalog.md` | modified | read-only | none |
| `docs/development/ai/swarm-results/cfr168-rust-file-doctor-zero-residual-rereview-claude-20260730/review.md` | untouched | **created** | none |

The candidate's five changed paths are exactly the `write_set` declared in
`docs/development/ai/swarm-batches/cfr168-rust-file-doctor-zero-residual-repair-20260730.json`;
nothing was written outside it. This reviewer's `write_set` is the single report above and does not
intersect the candidate's. No product code, test, source contract, issue state, or manager ledger
was modified by this review. The candidate is not integrated into `main`; the manager owns
integration.

Cross-command dependency: the response-projection/CLI-envelope accounting item touches
`crates/codefire-cli/src/main.rs`, which is outside both this packet and this review.

## Next action

Manager owns integration. Recommended: integrate `1a04490…` as-is. CFR-168's file-Doctor-local scope
is closed — all four Should and all four Could findings from `c1b39ffc…` are resolved with
production-reachable, deliberately regressed oracles. The remaining CFR-168 Doctor work
(command-wide response projection / CLI envelope accounting, and the other Doctor surfaces listed in
`command-source-trace.md`) stays with its own packets. NOTE-1 through NOTE-3 need no rework in this
scope; NOTE-3 is the only one worth carrying forward as a one-line entry if a future packet widens
the fence to unreadable subjects.
