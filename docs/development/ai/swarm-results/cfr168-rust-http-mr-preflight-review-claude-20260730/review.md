# CFR-168 Rust HTTP MR Create/Review Response Preflight — Independent Claude Review

- Reviewer model: Claude Opus 5 (`claude-opus-5`)
- Review date: 2026-07-30
- Batch: `cfr168-rust-http-mr-preflight-review-claude-20260730`
- Task: `rust-http-mr-preflight-review-claude` (review-only; no product code, tests, issue state, or
  management ledger touched)

## Exact subject

| Item | Value |
| --- | --- |
| Baseline / task revision | `3c23e381f871724d31332bd179671db7cbb9d941` |
| Completed target | `71854e20689d2dd79246fa741e0e6da443d04d1a` |
| Target branch | `claude-agent/cfr168-rust-http-mr-preflight-claude-hedge-20260730-mr-create-review-preflight-hedge` |
| Reviewer base revision | `fcaa99b6808aad9cdbb217473217812d0cbb19e0` |
| Cumulative path | `crates/codefire-cli/src/http.rs` (sole file changed: +1403 / −47) |
| Contract | `docs/development/ai/swarm-batches/cfr168-rust-http-mr-preflight-20260729.md` |

Verified diff scope: `git diff --stat 3c23e38 71854e2` → `1 file changed, 1403 insertions(+),
47 deletions(-)`. Every hunk outside the two routes and the new owner type is `#[cfg(test)]`-gated
(`HttpRequestDeadline::check` line 44–47, thread-locals at 74–96, the encode counter at 1329–1337).
No production byte of the upload, apply, list, or shared-encoder paths changed.

Acceptance was **not** inferred from the implementation agent's reported 141/141 or 776/776. Every
validation below was re-run independently from a clean `git archive` extraction of `71854e2` into a
scratch tree (`/tmp/cfr168-verify`), and two adversarial probes were run that the packet's own tests
do not perform.

---

## Verdict

**REQUEST_CHANGES**

The production change is sound. I found **no product defect**: all six required behaviours hold under
code reading and under two independent probes, including a real post-seal publication failure that
none of the new tests exercise. The blocker is confined to the **test evidence**, and it is confirmed
by mutation, not inferred:

> Moving `MergeRequestResponsePreflight::prepare` to *after* `source_records.publish_with_work_and_poll`
> — i.e. reintroducing the exact defect class this packet exists to close, for the object-escrow
> effect — leaves **all eight new tests green** (19/19 `merge_request` tests; 775/776 full target,
> the single failure being an unrelated `import_workflow` concurrency flake).

Requirement 6 of this review explicitly mandates verifying production-path non-vacuity, and the
contract explicitly forbids accepting tests that do not establish the contract directly. The central
ordering guarantee for merge-create therefore ships without a regression barrier. The required
remediation is test-side and small (Findings 1 and 2); Findings 3–5 are hardening/quality and need
not block.

---

## Six required verdicts

| # | Required review | Verdict |
| --- | --- | --- |
| 1 | Create and review prebuild exact success + durable-retry status/body/header bytes on the request ledger before the first object/publication effect; writer paths never re-encode or select fallibly after an unrecorded effect | **PASS (product) / FAIL (evidence)** — ordering is correct by construction, but the create pre-effect evidence is vacuous w.r.t. the escrow (Finding 1) |
| 2 | Exact / minus-one / plus-one `response_bytes`, `response_header_bytes`, retained/temporary ownership, encoding failures, deadline phases, disconnects are pre-effect or recover to one durable byte-identical result, zero request debt, no open permit | **PASS with coverage gap** — verified for create and independently for a real post-seal failure; review lacks response/header boundaries (Finding 4) |
| 3 | Audit the reported pre-existing outer retained residual on merge-create success/error; prove intentional ownership through request termination or require repair | **PASS — and the reported success-path residual does not exist** (Finding 2) |
| 4 | Audit the second `MergeRequestResponsePreflight` owner beside `PreparedHttpMutationTransaction`: no duplicate encoding/reservation, split fence, or cleanup-order effect gap | **PASS with hardening required** — no reachable gap; the new owner's drop/seal guards are strictly weaker than the incumbent's (Finding 3) |
| 5 | Preserve authentication/nonce/signature, escrow, review materialization, error precedence, replay, post-commit 503 retry semantics, apply/upload neighbours, exact wire bytes | **PASS** |
| 6 | Production-path non-vacuity, updated existing tests, test-only hook isolation, full authority snapshots, strict Clippy, focused HTTP tests, representative cross-runtime cases | **FAIL on non-vacuity; PASS on all others** |

---

## Findings (severity-ordered)

### F1 — MEDIUM · test-coverage · CONFIRMED by mutation
**The merge-create "pre-effect" evidence does not constrain the preflight's position relative to the
object escrow, and the publication-fence case asserts "no effect" after an effect has occurred.**

Evidence — fixture, `crates/codefire-cli/src/http.rs:13679-13697`:

```rust
let head = write_http_test_commit(&source_objects);
for record in collect_object_records(&source_objects, &head).unwrap() {
    let document = codefire_store::object_record_document_from_value(record).unwrap();
    codefire_store::store_object_record_document(&target_objects, document).unwrap();   // :13689
}
for root in [&source_root, &target_root] {                                             // :13691
    write_json_atomic(&remote_branch_path(root, "main"),
        &json!({"version": 1, "name": "main", "head": head})).unwrap();                // same head
}
```

`write_merge_create_fixture` gives source and target the **same head** and pre-copies **every source
object record into the target object store**. The escrow at `:9142`
(`source_records.publish_with_work_and_poll(&target_objects, …)`) is therefore byte-idempotent in
every create test, and `remote_project_snapshot` (`:13627`, files only — directories are never
recorded) cannot observe it.

Two consequences:

1. `merge_request_routes_poll_the_deadline_at_every_preflight_fence` case
   `("fence", "merge-request publication preflight")` (`:14106`) asserts
   `"{label} deadline must leave every authority byte-identical"` (`:14127`). That fence
   (`:9226`) sits **after** the escrow, so a durable object effect has already happened when this
   assertion claims none did. The assertion passes only because the fixture pre-seeded the objects.
2. The pre-effect claims in `merge_request_create_response_limits_are_exact_and_pre_effect` and
   `injected_merge_request_preflight_failure_is_typed_and_pre_effect` do not distinguish
   "failed before the escrow" from "failed after the escrow".

**Mutation probe (decisive).** In the scratch tree I swapped the two blocks so that the escrow runs
first and `MergeRequestResponsePreflight::prepare` runs after it, with the escrow-failure path
cleaning up without the preflight:

```
cargo test -p codefire-cli --bin codefire-rs --locked merge_request
  → test result: ok. 19 passed; 0 failed  (all 8 new tests green)
cargo test -p codefire-cli --bin codefire-rs --locked
  → test result: FAILED. 775 passed; 1 failed
     (failure: import_workflow::tests::concurrent_exact_import_with_wait_replays_the_winner —
      unrelated concurrency test; passes 3/3 in isolation and in both pristine full runs)
```

No test in the 776-test target detects the reintroduced defect.

**Not a product defect.** Code order at `:9120` (prepare) → `:9142` (escrow) is correct, and
requirement 1 is satisfied by construction. The gap is that the contract's requested regression
barrier ("filesystem authority snapshots proving no effect on every typed failure") is absent for the
object effect. It *is* load-bearing for the publication authority: an MR record, receipt, and
transaction records would appear in `remote_project_snapshot` and fail `assert_authority_unchanged`.

**Required remediation.** Give the create fixture a target head that does **not** already contain the
source objects (distinct heads, or a target object store missing at least one source record) so the
escrow is observable, and assert the target object store byte-set explicitly in the pre-effect cases.
Separately, correct the fence case's assertion to what it actually proves (no *authority* effect,
escrow already committed), or move the fence before the escrow.

---

### F2 — LOW/MEDIUM · correctness-of-claim · CONFIRMED by probe
**The documented "pre-existing outer retained residual on merge-create success" does not exist; the
create test discards a residual that is provably zero, while the review test asserts it.**

`crates/codefire-cli/src/http.rs:13595-13606`:

```rust
/// The merge routes deliberately hand a few unrelated read owners to
/// the outer publisher, which releases them after the response is
/// written. What must never survive is a prepared-response permit or a
/// temporary charge.
fn assert_no_open_work(&self) { … TemporaryBytes == 0; !has_open_permits() … }
```

`RetainedBytes` is deliberately excluded, and the create call site discards the residual
(`:13911`, `let (status, body, selected_address, _) = run.into_selected_response();`) whereas the
review call site asserts it is zero (`:13941-13955`).

**Probe result** (`review_probe_merge_create_residual_is_absorbed_by_the_outer_publisher`, added only
in the scratch tree):

```
PROBE create success: body_len=126 residual_after_body_release=0
```

On merge-create success, after releasing the prepared body, retained current is **exactly 0**. The
126 bytes are the frozen success body itself — the one owner that is *supposed* to be handed to the
publisher. So the create route releases every owner it opens, and the comment's "a few unrelated read
owners" is inaccurate for the success path.

**Error paths.** The residual there is real and equals the two branch-head frame owners
(`source_head_frame_bytes` + `target_head_frame_bytes`, released only on the success/replay paths):

```
PROBE create-error retained residual handed to publisher = 580   (injected preflight failure)
PROBE post-seal current = {RetainedBytes: 580, TemporaryBytes: 0} sealed=true
```

580 is exactly the number `merge_create_pre_preflight_residual` (`:13716`) measures and every
preflight-fault case pins (`residual + precharged_retained`), so the frozen pair itself provably
leaks nothing.

**Item-3 disposition: intentionally owned through request termination — no repair required.** Proven
by probe on both paths: after `write_http_dispatch_outcome`, every dimension of `snapshot.current` is
0, `has_open_permits()` is false, and exactly one terminal reason is set (`Complete` on success,
`CorruptSubject` on the error paths). Mechanism: `release_all_http_current_memory` at `:822` on the
prepared path and `terminate_http_request` at `:895-921` on the error path. Baseline provenance is
not the reason it is exempt; termination ownership is.

Caveat worth recording: both mechanisms are a **blanket** `release(dimension, current)` sweep, not a
named per-owner release. The request-level invariant therefore cannot distinguish "route correctly
handed a live owner to the publisher" from "route leaked an owner" — which is precisely why F2's
missing `assert_eq!(residual, 0)` on create matters.

**Required remediation.** Assert `residual == 0` for create as the review case already does, and fix
the `assert_no_open_work` doc comment.

---

### F3 — LOW · robustness · item 4 (second typed owner)
**The new owner's drop and seal guards are strictly weaker than the incumbent owner's for the same
ledger resource.**

| Concern | `PreparedHttpMutationTransaction` (upload/apply) | `MergeRequestResponsePreflight` (merge/review) |
| --- | --- | --- |
| Drop | `:3837-3897` fail-closed: cancels, verifies zero current **and** no open permits, aborts the process otherwise | `:8877-8884` `debug_assert!` only — no cancellation |
| Seal guard | `:3381` rejects if `self.sealed` **or** `ledger.is_prepublication_sealed()` | `:8772-8783` checks only `self.responses.is_none() \|\| self.sealed` |

`seal_prepublication` (`crates/codefire-core/src/work_budget.rs:978-984`) is idempotent, so a second
sealer succeeds silently. In release builds a future early return between `prepare` and
`select`/`cancel` leaks both permits and both frozen bodies with no diagnostic; the leak then
surfaces only as `finish` → `OpenPermits` (`work_budget.rs:1168`) inside `write_http_dispatch_outcome`
— **after** the body has already been written to the socket at `:818`. Note also that the existing
`PreparedMutationResponsesOwner` (`:2907-2951`) already provides the safer Drop-cancels-automatically
variant and the new owner does not use it.

`crates/codefire-core/src/work_budget.rs:730-732` still documents the invariant this change
supersedes: *"only the combined transaction owner may seal."*

**No reachable defect today.** I audited every path in both routes; the preflight is selected or
cancelled on all of them, and no route constructs both owners, so there is no double-seal, no
duplicate encoding, and no split fence in practice. Recommended hardening: mirror the incumbent's
already-sealed check in `seal`, and make `Drop` cancel rather than merely assert. Also update the
codefire-core doc comment.

---

### F4 — LOW · test-coverage
**The review route has no `response_bytes` / `response_header_bytes` exact / minus-one / plus-one
boundary, and the create boundary pins the durable body rather than the success body.**

`merge_request_create_response_limits_are_exact_and_pre_effect` (`:13961+`) covers all three
dimensions for create only. Review gets only `RetainedBytes`, via the updated
`review_retained_limit_minus_exact_plus_one_is_effect_safe` (`:13454-13574`), which asserts
`prepared.is_none()` and an unchanged MR record at limit+1 but does not prove the failure lands at the
preflight rather than at any earlier retained acquisition.

Measured sizes: frozen durable body = **149 bytes**
(`{"error":"the durable mutation completed without a confirmed response; retry the same request","kind":"retry_same_request","retry_same_request":true}`),
create success body = **126 bytes**, review success body ≈ 60 bytes. So
`response_upper = max(success, durable)` is the **durable** body on both routes. Consequences:

- The `response_bytes` / `response_header_bytes` boundaries in the create test pin the *durable*
  body's accounting; a regression affecting only the success body's size would not move them. The
  `retained_bytes` case does capture both (`retained_upper = retained_before + success + durable`,
  `:13993-14002`).
- Both routes now reserve a **larger** ResponseBytes/HeaderBytes peak than baseline (the durable body
  instead of the success body). Cumulative charge is unchanged — see the non-regression note below —
  but the peak reservation is a genuine behaviour change with no boundary test on review.

---

### F5 — LOW · quality
**Stale `#[allow(dead_code)]` blankets and a stale staging comment now that the raw API is
production-wired.**

`crates/codefire-cli/src/http.rs:1078-1080`:

```rust
// This raw API is intentionally staged before its production route wiring so
// publication and Store owners can adopt it without introducing ledger borrows.
#[allow(dead_code)]
fn prepare_mutation_responses_with_deadline(…)
```

This is now false: merge and review wire it in production. The same stale `#[allow(dead_code)]`
remains on `PreparedMutationResponses` and its impl (`:2663`, `:2681`), `RETRY_SAME_REQUEST_RESPONSE`
(`:1218`), `encode_prepared_mutation_response` (`:1159`), `http_response_work_reservation` (`:1225`),
`release_prepared_mutation_response` (`:1244`), and `release_prepared_mutation_pair` (`:1253`). The
blanket also suppresses reporting of `PreparedMutationResponsesOwner` (`:2907`), which is now
genuinely unused in production. Clippy `-D warnings` cannot catch stale allows.

---

## Independently verified positives

- **Wire byte equivalence and replay.**
  `merge_request_create_and_review_publish_the_exact_frozen_preflighted_response` (`:13795+`) compares
  full raw HTTP bytes including status line and headers, for both routes, and asserts the replay is
  byte-identical *and* leaves the authority snapshot unchanged. Non-vacuous and strong.
- **No second encoding.** `http_response_encodings() == 2` (exactly the frozen pair) on both routes,
  plus pointer identity: `merge_request_preflight_body_address()` is compared against the address of
  the buffer that reaches the writer (`:13911-13924`). This is real owner identity, not
  `prepared.is_some()`.
- **Owner is the request ledger.** `merge_request_preflight_ledger()` is checked against
  `ledger.identity()` and against a second request's distinct identity (`:13927-13946`).
- **Publisher deadline poll is unreachable on the prepared path.**
  `merge_request_publication_handoff_cannot_be_lost_to_a_late_deadline` (`:14183+`) trips
  `"response publication"` and still gets `HTTP/1.1 200 OK` with the MR committed — confirming
  `write_http_dispatch_outcome`'s prepared branch (`:814-829`) bypasses the writer's deadline and
  encoding failure surfaces entirely.
- **No cumulative double-charge.** `encode_prepared_mutation_response` encodes with
  `charge_response_bytes = false` (`:1169`, writer at `:1392-1395`), so only
  `consume_reserved_pair` charges `ResponseBytes`/`ResponseHeaderBytes`, with the *selected* body's
  exact bytes, and the unused permit remainder is cancelled (`:2829-2838`). Cumulative response
  accounting is therefore byte-for-byte identical to baseline.
- **Replay releases before charging.** `replace_with_recorded_result` (`:8758-8770`) is
  `cancel` → `encode`, so the planned pair is released before the recorded pair is charged; retained
  peak is `max(planned, recorded)`, not the sum.
- **Publishing the frozen body instead of `published` is sound.** `execute_json_publication`
  (`crates/codefire-cli/src/remote/publication.rs:3246-3312`) can return a value other than `result`
  only via `recover_completed_pending_quarantine`, a pre-existing pending marker, or
  `load_completed_receipt` — and `replay_json_publication` (`:3314+`) drains all three first under the
  same continuously-held exclusive gate on both routes. `serde_json` is built without
  `preserve_order`, so any disk round-trip re-serialises to identical bytes. The guarantee is
  nevertheless enforced only by `debug_assert_eq!` (`:9250-9255`, `:9876-9881`), where baseline was
  correct by construction; a release-build assertion would be defence-in-depth.
- **Real post-seal publication failure recovers cleanly (probe; not covered by any new test).** Two
  creates for the same source/target/head within one `now_iso_utc()` tick collide on `mr_id`, producing
  a genuine post-seal, pre-commit publication rejection:
  ```
  PROBE post-seal publication failure = Some(InvalidRepository(
    "remote publication authority diverged from its exact pre/postimage: …/merge_requests/MR-232fad9c4335.json"))
  PROBE post-seal current = {RetainedBytes: 580, TemporaryBytes: 0} sealed=true
  PROBE post-seal terminal = Some(CorruptSubject)
  ```
  Typed pre-existing diagnostic, `prepared.is_none()`, no open permit, residual absorbed at
  termination, exactly one terminal reason. Requirement 2 holds on a path the packet does not test.
- **Test-only hook isolation.** All injection state is `#[cfg(test)]` + `thread_local!`
  (`:74-96`, `:1294-1312`, `:8556-8626`); `reset_merge_request_preflight_observations` (`:8565`)
  clears deadline and encode counters too. No production byte is affected.
- **Preserved semantics (item 5).** Signature/nonce verification order untouched
  (`:8913-8920`, `:9578-9585`); escrow position unchanged; review subject materialization is
  in-memory only (`&mut mr`) and still precedes the preflight; the pre-existing
  `"review publication preflight"` deadline phase string is preserved by `seal` at `:9866`;
  post-commit 503 retry semantics implemented via `select_after_commit` (`:8836-8851`) and verified
  by `committed_merge_request_falls_back_to_the_frozen_durable_retry_response` (`:14286+`).
- **Bootstrap writes before the preflight are out of contract scope.** `ensure_remote_layout`
  (`:8921`) is the only durable write before the preflight; it is the idempotent layout bootstrap and
  runs before all planning. `remote_project_stable_id` is read-only
  (`remote/project.rs:1443-1459`, `:3176-3178`) — it does **not** initialise a descriptor.
- **Existing tests updated rather than bypassed.** `ReviewBoundaryRunResult` gained the prepared owner
  (`:11634`), and the pre-existing accounted-review and review-boundary tests now take and assert the
  prepared response (`:13166-13173` (`take_prepared_route_response` at `:13167`), `:13499-13502`, `:13531-13539`).

## Residual risk

1. **Post-commit accounting-release failures still produce an error response, not the frozen durable
   body.** In review, the post-closure chain at `:9891-9923`
   (`release_http_temporary` / `release_http_retained` × 4) uses `?` *after* `select_after_commit` has
   already succeeded; in create, `destination_ownership.release(…)?` inside the handoff closure
   (`:9259-9264`) runs before `publication?`. Either failing turns a committed mutation into an error
   response, which is the class the frozen durable body exists to prevent. Only reachable through an
   accounting bug (all are pure `ledger.release` underflow checks), and the behaviour is unchanged
   from baseline — but the new machinery is not used to cover it.
2. **The durable-retry fallback's partial-failure path is untested.**
   `inject_merge_request_success_selection_failure` returns *before* `self.select(…)` touches any
   state (`:8793-8801`), so the tested fallback never exercises a `select_success` that fails
   midway. If it did, `consume_selected_work` would already have cleared `response_permit`/
   `header_permit` (`:2839-2840`), so the subsequent `select_durable` would fail with
   "prepared mutation response-byte permit is missing" rather than falling back, and `cancel` would
   then run against a half-consumed pair. Practically unreachable, but the fallback's advertised
   guarantee is not demonstrated for a real failure.
3. **Directory-level effects are invisible to the authority snapshots.**
   `remote_project_snapshot` (`:13627-13649`) records files only and silently returns on
   `read_dir` errors, so created/removed directories and unreadable subtrees do not register.
4. `#[cfg(test)]` deadline injection keys on `&'static str` phase equality; two phases sharing a
   string would trip together. None currently collide, but new phase names must stay distinct.

## Commands and results

All commands run against a clean `git archive 71854e2` extraction in `/tmp/cfr168-verify` with
`CARGO_TARGET_DIR=/tmp/cfr168-verify-target`, toolchain `cargo 1.97.1`:

```bash
export RUSTUP_HOME=/tmp/codefire-rustup/rustup
export CARGO_HOME=/tmp/codefire-rustup/cargo
export PATH="$CARGO_HOME/bin:$PATH"
```

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | **pass** (no output, rc=0) |
| `cargo test -p codefire-cli --bin codefire-rs http::tests --locked` | **pass** — `141 passed; 0 failed; 635 filtered out` in 29.77 s; all 8 new tests present and green |
| `cargo clippy -p codefire-cli --bin codefire-rs --locked -- -D warnings` | **pass** — rc=0, zero warnings |
| `cargo test -p codefire-cli --bin codefire-rs --locked` (full binary target) | **pass** — `776 passed; 0 failed` in 34.95 s |
| `python3 docs/development/tools/check_source_docs.py` | **pass** — `44 modules, 34 commands, 12 object kinds` |
| `git diff --check 3c23e38 71854e2` | **clean** |
| implementation worktree `git status --short` | **clean** |
| Cross-runtime representative cases | `tests::merge_request_subject_matches_cross_runtime_golden_vector`, `tests::file_remote_upload_clone_show_diff_and_merge_request_flow`, `tests::merge_request_target_escrow_survives_source_project_removal` — all green |

Adversarial probes (scratch tree only; pristine file restored and byte-verified afterwards):

| Probe | Result |
| --- | --- |
| Mutation: preflight moved after the object escrow | `19 passed; 0 failed` on `merge_request`; `775 passed; 1 failed` full (unrelated `import_workflow` flake) → **F1 confirmed** |
| `concurrent_exact_import_with_wait_replays_the_winner` on pristine tree, ×3 isolated + 2 full runs | pass every time → flake, not a target regression |
| Residual attribution on create success | `body_len=126 residual_after_body_release=0` → **F2 confirmed** |
| Outer absorption of create success + error residual | zero current, no open permits, one terminal reason on both → **item 3 satisfied** |
| Real post-seal publication failure | typed diagnostic, `prepared.is_none()`, no open permit, 580 residual absorbed, one terminal reason → **requirement 2 satisfied on an untested path** |

## Integration conflict map

- **Sole file:** `crates/codefire-cli/src/http.rs`. No `Cargo.toml`, schema, doc, or ledger change,
  so no cross-crate or documentation integration surface.
- **Applies cleanly onto the current integration tip.** `crates/codefire-cli/src/http.rs` at
  `fcaa99b` (integration base) is byte-identical to the task baseline `3c23e38`
  (`git diff 3c23e38 HEAD -- crates/codefire-cli/src/http.rs` → empty).
  `git merge-tree 3c23e38 fcaa99b 71854e2` produces **no conflict markers**.
- **Other open branches touching `http.rs`:** `agent/cfr168-http-bootstrap`,
  `agent/cfr168-python-http`, `agent/cfr168-python-http-work`,
  `agent/cfr168-python-mutation-response-preflight`. All four share merge-base `f164412` (main) and
  are stale predecessors of the current integration line — the integration branch's `http.rs` already
  contains far more work than any of them. They are not live competitors for this hunk, but any
  attempt to land them would require a rebase, not a merge.
- **Semantic adjacency to watch:** the Python-side counterpart
  (`agent/cfr168-python-mutation-response-preflight`) implements the same contract for the Python HTTP
  path. The frozen durable-retry body text and the 503 status are now a **cross-runtime wire
  contract** (`RETRY_SAME_REQUEST_RESPONSE`, 149 bytes, `http.rs:1218-1223`); the two runtimes must
  agree byte-for-byte. No golden vector currently pins it across runtimes.
- **Shared-symbol adjacency:** this change is the second production consumer of
  `PreparedMutationResponses` / `seal_prepublication`. Any later route that adopts
  `PreparedHttpMutationTransaction` for merge or review must not also construct
  `MergeRequestResponsePreflight` — see F3, which is the guard that would catch it.

## Recommended remediation before acceptance

1. **(F1, required)** Make the create fixture's escrow observable — distinct source/target heads, or a
   target object store missing at least one source record — and assert the target object byte-set in
   every pre-effect case. Re-check the mutation probe fails afterwards. Correct or relocate the
   `"merge-request publication preflight"` fence case's byte-identical claim.
2. **(F2, required)** Assert `residual == 0` on create success as review already does, and fix the
   `assert_no_open_work` doc comment — the create route leaves no unrelated retained owner.
3. **(F3, recommended)** Mirror `revalidate_and_seal`'s already-sealed check in
   `MergeRequestResponsePreflight::seal`; make its `Drop` cancel rather than `debug_assert`; update
   `work_budget.rs:730-732`.
4. **(F4, recommended)** Add review-route `response_bytes` / `response_header_bytes` exact /
   minus-one / plus-one cases.
5. **(F5, optional)** Drop the now-stale `#[allow(dead_code)]` attributes and the "staged before its
   production route wiring" comment; decide whether `PreparedMutationResponsesOwner` stays.
