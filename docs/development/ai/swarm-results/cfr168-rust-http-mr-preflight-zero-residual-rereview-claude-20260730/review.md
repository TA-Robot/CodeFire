# CFR-168 Rust HTTP MR Preflight Zero-Residual — Independent Claude Rereview

- Reviewer model: Claude Opus 5 (`claude-opus-5`)
- Review date: 2026-07-30
- Batch: `cfr168-rust-http-mr-preflight-zero-residual-rereview-claude-20260730`
- Task: `rust-http-mr-preflight-zero-residual-rereview` (review-only; no product code, test,
  source-contract, issue-state, or manager-ledger byte touched)
- Reviewer base revision: `56719771342e8c7b94e22ae53fb6551605d9342a`

---

## 1. Recomputed exact subject and clean state

Every identity below was recomputed in this worktree before any code was read. **All match.**

| Item | Declared | Recomputed | Result |
| --- | --- | --- | --- |
| Exact repaired candidate | `73bc5327ff6d750ce95f63333055ee39b693cb8c` | `73bc532` present, reachable from `HEAD` | match |
| Candidate parent | `7e4e685efbfc453a6d2209f37c99edb1ab46243d` | `git cat-file -p 73bc532` → `parent 7e4e685…` | match |
| Candidate tree | `430f2e2ccf9dca485575e7f9db0b639390d7e761` | `git cat-file -p 73bc532` → `tree 430f2e2…` | match |
| Binary patch SHA-256 | `216c12f8d9e0c6939bf6889856a999e8f45b56237eca9ff63cdef34105ef268d` | `git diff --binary 7e4e685 73bc532 \| sha256sum` → identical | match |
| Prior report SHA-256 | `f30c22aa34559829254b6ca305449abc874b653c8b3dbc767248fdbbc5fe0a5a` | `sha256sum …/cfr168-rust-http-mr-preflight-review-claude-20260730/review.md` → identical | match |
| Repair contract SHA-256 | `b1c8e65a2af80a776c58e550d24272753698fb337464729ee49880804ef95e1e` | `sha256sum …/cfr168-rust-http-mr-preflight-review-closure-repair-20260730.md` → identical | match |
| Original reviewed candidate | `71854e20689d2dd79246fa741e0e6da443d04d1a` | `git merge-base --is-ancestor 71854e2 7e4e685` → rc=0 | ancestor, intact |
| Diff volume | 2,763 insertions / 560 deletions | `git diff --shortstat 7e4e685 73bc532` → `6 files changed, 2763 insertions(+), 560 deletions(-)` | match |

Changed paths are **exactly** the six declared paths, no more and no fewer
(`git diff --name-only 7e4e685 73bc532`):

```text
crates/codefire-cli/src/http.rs                  +2437 / -557
crates/codefire-core/src/work_budget.rs          +237  / -1
docs/development/change-playbook.md              +26   / -0
docs/development/command-source-trace.md         +17   / -0
docs/development/source-architecture.md          +32   / -2
docs/development/source-data-model-catalog.md    +14   / -0
```

Clean state:

- `git status --porcelain` → empty at review start.
- `git diff --check` → clean (rc=0).
- Reviewer branch `claude-agent/cfr168-rust-http-mr-preflight-zero-residual-rereview-claude-20260730-rust-http-mr-preflight-zero-residual-rereview` at `5671977`;
  `git merge-base HEAD main` = `f1644122134d479333cf422fdb1d48b8fa92dd8e` = `main`, so the
  candidate line is a linear 24-commit descendant of `main`, not a divergent branch.
- Ancestry chain verified intact and additive:
  `3c23e38 → 71854e2 (reviewed candidate) → 185b7b7 (report copy) → 7e4e685 (repair contract freeze)
  → 73bc532 (exact repaired candidate) → 5671977 (this rereview's contract freeze)`.
- The candidate has exactly one parent (no merge commit).

All independent probes below were run against a `git archive 73bc532` extraction in
`/workspace/.cache/cfr168-rereview-scratch`, verified byte-identical to the candidate:
`git hash-object` on the extracted `http.rs` / `work_budget.rs` returned
`081e3cfbfc30d5df3b97d2ddbaab05a6f517328c` and `945a2f53b84786843b7c6aac04c9c47659d0b2f3`, equal to
`git rev-parse 73bc532:<path>` for both.

---

## 2. WIP-rewrite provenance reconciliation

The repair contract states: *"Add new commits; do not amend, rewrite, merge, or replace the reviewed
candidate or report"* and *"Create a durable WIP commit before long Cargo gates."* The implementation
lane produced temporary WIP commits and then delivered one squashed commit. Reconciliation:

**What the deviation did not damage.**

- The reviewed candidate `71854e2`, the candidate-rooted report copy `185b7b7`, and the contract
  freeze `7e4e685` are all still reachable ancestors of `HEAD`. Nothing reviewed was amended,
  replaced, or merged away.
- `73bc532`'s parent is exactly `7e4e685`, so the delivery is a strictly additive successor of the
  frozen taskbase, as required.
- No product bytes appear lost. The delivered tree, parent, and binary patch digest match the
  declared subject exactly; the diff touches exactly the six declared paths; and every one of the
  nine closure mechanisms claimed in the commit message is physically present in code and
  independently exercised (sections 4–5). `cargo fmt --all -- --check`, both Clippy gates, and all
  788 binary-target tests pass, so no edit was truncated or half-applied by the squash.

**What the deviation did cost — and it is not nothing.**

- The WIP commits are **not recoverable from this reviewer worktree**. `git fsck --dangling` on the
  shared object store lists dangling commits from other CFR-168 lanes (for example
  `62b212c` `cfr168-conflict-accounting-wip`, `c26f11b` `cfr168-session-release-wip`, and two
  `review: request changes on Python HTTP MR mutation lane` commits dated 2026-07-30) but **no
  Rust-repair WIP commit**. `git reflog show HEAD` on this worktree has two entries and does not
  reach them. I therefore cannot reconstruct any intermediate state independently; I can only verify
  the coherence of the final one.
- There is **no repair result document in the repository.**
  `docs/development/ai/swarm-results/` contains only `cfr168-rust-http-mr-preflight-review-claude-20260730`,
  `claude-cfr168-review-20260729`, and `current-state-closure-audit-20260729`. The repair contract's
  "Handoff" section required formulas, peaks, boundaries, state transitions, snapshot hashes,
  deliberate-regression commands/results, counts, durations, and exact hashes. None of that is a
  durable repository artifact; the only in-repo record of the lane's closure evidence is
  `73bc532`'s commit message.
- That commit message is **not fully accurate**: its claimed "57-byte review success body" is wrong
  (the actual encoded body is 60 bytes — Finding F-2). A single squashed commit message is therefore
  the whole evidence trail, and it contains a factual error that no test or document pins.

**Explicit conclusion, as required:** a clean single-commit history is *not* accepted here as proof
that the process contract was followed. The no-rewrite instruction was honoured with respect to the
reviewed candidate and report, and violated with respect to the lane's own durable WIP commits; the
durable-evidence half of the handoff requirement was not satisfied in-repo at all. No product bytes
were lost, but the intermediate evidence was.

---

## 3. Verdict

**REQUEST_CHANGES**

All nine required closure items — **F1–F5 and R1–R4 — are closed**, and I found **no product
defect**. The production change is correct under reading, under the full authorized gate ladder, and
under six deliberate regressions I authored and ran independently, four of which reproduce the exact
reversal the packet exists to prevent. F1's previously vacuous evidence is now genuinely
non-vacuous and I confirmed it by mutation.

The blocker is confined to **evidence and claim defects on two items this ticket explicitly told me
to falsify**, both cheap to fix and neither affecting shipped behaviour:

1. **The 1x/3x/10x scaling oracle is vacuous** (Finding F-1). The required evidence item is
   *"exact 1x/3x/10x counter curves where work scales with object or response count."*
   `merge_request_response_work_is_flat_while_object_work_scales` varies only *unreachable* source
   objects, so the escrowed closure — and therefore every counter in the curve — is identical at 1x,
   3x, and 10x. The test's own passing assertion
   (`object_records == base_objects`, "only the source closure is escrowed, and it is the same
   closure") is the proof that the axis never moved, while its doc comment claims *"object work
   scales exactly with it."* Neither half of the claim is exercised.
2. **The reconciled response bytes do not match the implementation's report** (Finding F-2). Measured
   on the candidate: create success **126** bytes ✓, durable retry **149** bytes ✓, review success
   **60** bytes — not the claimed 57.

Given that this lane's only durable evidence artifact is the commit message (section 2), a false
number in it and a vacuous oracle presented as a required counter curve are exactly the class of
defect the prior review blocked on, so I hold to the same standard. Remediation is test/claim-side
and small; the product needs no change.

---

## 4. Required verdicts — F1 through F5

| # | Required item | Verdict |
| --- | --- | --- |
| F1 | Observable first-effect ordering, non-vacuous closure, fail-closed complete snapshots, actual reversal, pre-escrow vs post-escrow distinction | **PASS — closed** |
| F2 | Exact residual ownership; zero after body release; named formulas; publisher closes at zero/zero/one | **PASS — closed** |
| F3 | Fail-closed response owner across seal, select, cancel, unwind/drop, release builds, primary-error preservation | **PASS — closed** |
| F4 | Independent create *and* review exact / −1 / +1 boundaries; success body separated from durable body; each failure reaches the intended owner | **PASS — closed** (one narrow displacement, §4.4) |
| F5 | Dead-code and owner cleanup; strict Clippy without replacement suppressions; no staged/duplicate owner claims | **PASS — closed** (one residual, §4.5) |

### 4.1 F1 — observable first-effect ordering · CLOSED

- **Non-vacuous source/target closure.** `write_merge_create_fixture` (`crates/codefire-cli/src/http.rs:14226-14282`)
  now writes two commits with different `verification` markers, so source and target heads differ
  (`assert_ne!`, `:14237`), and it computes `escrowed_object_ids` as the exact set difference and
  asserts it is non-empty (`:14263`), that the target head's own closure is present (`:14267`), and
  that the source head is *absent* from the target (`:14271`). The reviewed candidate's
  byte-idempotent escrow is gone.
- **Complete fail-closed snapshots.** `remote_project_snapshot_filtered` (`:14091-14145`) records
  `Directory`, `File(bytes)`, and `Symlink(target)` entries, uses `symlink_metadata` so links are not
  followed, and **panics** on every `read_dir`, iterate, `symlink_metadata`, `read_link`, and `read`
  failure, plus on a missing root (`:14129`). Authority and object snapshots are split
  (`remote_authority_snapshot` `:14160`, `remote_object_snapshot` `:14165`).
- **Pre-escrow no-effect vs post-escrow publication fence are now distinct guarantees.**
  `merge_request_preflight_precedes_the_observable_object_escrow` (`:14646`) asserts *both* authority
  and object stores byte-identical after an injected preflight failure, and asserts the escrowed ids
  are still absent (`:14702`). `merge_request_publication_fence_guarantees_authority_and_retries_exactly`
  (`:14748`) asserts authority unchanged, then explicitly asserts the escrow **did** commit and
  committed exactly the missing closure (`:14781`), then proves exact retry/recovery by re-running the
  identical request to a 200 with the object store rewritten byte-identically (`:14804`). The
  production comment at `:9521-9531` states the same limited promise and why the fence cannot move
  before the escrow (the escrow charges the ledger; sealing blocks exactly that).
- **Actual reversal — independently reproduced.** I moved
  `MergeRequestResponsePreflight::prepare` to after `source_records.publish_with_work_and_poll`,
  preserving every nearby helper call including `record_http_cleanup_error` and
  `source_records.release` (mutation **M1**). Result: `21 passed; 4 failed` on the focused
  `merge_request` filter. See §6.

### 4.2 F2 — exact residual ownership · CLOSED

- **Create success ends at zero after the selected body is released.**
  `into_selected_response` (`:13997`) releases the prepared body and returns the residual;
  `assert_eq!(residual, 0)` is asserted on create success in
  `merge_request_preflight_precedes_the_observable_object_escrow` (`:14722`),
  `merge_request_publication_fence_guarantees_authority_and_retries_exactly` (`:14800`),
  `merge_request_response_work_is_flat_while_object_work_scales` (`:16228`), and for review in
  `review_response_limits_are_exact_and_pre_effect` (`:15106`). My own probe confirms it directly:
  `PROBE create status=200 len=126 residual=0`.
- **The stale comment is replaced by the real invariant.** `assert_no_open_work` (`:13968-13992`) now
  documents that retained ownership is asserted separately and always by an exact named formula, and
  states why: both routes retire every owner before their first authoritative effect.
- **Every intentional handoff has a named formula.** The one remaining transferred residual is the
  pre-precommit-release failure, and it is pinned as a *named sum* measured independently of the
  route: `merge_create_pre_preflight_residual` (`:14305`) equals
  `merge_create_branch_head_frame_bytes(&fixture)` (`:14326`, measured by driving
  `read_http_branch_head_accounted` on a separate ledger), asserted at `:14656` and `:15333`.
  `merge_create_digest_destination_bytes` (`:14366`) independently measures the destination
  reservation the route now retires precommit, asserted `> 0` at `:14661`.
- **The publisher's close is proven, not swept.** `into_written_wire` (`:14007`) drives the real
  `write_http_dispatch_outcome`, and `assert_request_finished_clean` (`:14038-14060`) asserts every
  dimension of `snapshot.current` is 0, `has_open_permits()` is false, and `terminal_reason` is
  exactly one expected value. Applied to create preflight failure (`CorruptSubject`, `:15320`), the
  transferred-owner path (`DeadlineExceeded`, `:15348`), review preflight failure (`:15388`), double
  selection failure (`:15554`), and both post-commit absorptions (`:15611`, `:15642`).
  The blanket terminal sweep is no longer used as ownership evidence for any unnamed owner.
- **Production side.** Create's `precommit_release` block (`:9399-9416`) retires the digest
  destinations and both branch-head frames *before* the escrow, so the only fallible step after the
  first authoritative effect is the frozen-response handoff (`:9557-9564`).

### 4.3 F3 — fail-closed response owner · CLOSED

| Concern | Where | Independent check |
| --- | --- | --- |
| Second seal | state guard `:9005-9009` | mutation **M2** removes the witness → test fails |
| Externally completed seal | `is_prepublication_sealed()` witness `:9010-9019` | mutation **M2**; matches `revalidate_and_seal` (`:3518`) |
| Release-build drop | ownership moved to `HttpRouteContext::drop` (`:4282-4299`), which holds the ledger and calls `responses.cancel(ledger)`, escalating to `fail_closed_http_mutation_drop` (`:4256`) | `merge_request_response_preflight_cell_releases_every_permit_on_drop` (`:15737`) uses `std::mem::forget(preflight)` (`:15834`) so the handle's `debug_assert` cannot do the work |
| Drop after every state transition | 7 exits: `WhilePrepared`, `WhileSealed`, `AfterCancel`, `AfterDoubleCancel`, `AfterSelect`, `AfterRejectedSecondSelect`, `AfterRejectedReEncode` (`:15751-15758`) | each asserts zero retained and zero open permits |
| Double / partial select | `select` rejects unless `Sealed` (`:9073`); `PreparedSelectionFault::{BeforeValidation,BeforeCommit,AfterCommit}` (`:1446-1467`) | `partially_failed_success_selection_still_selects_the_frozen_durable_response` (`:15443`) |
| Cancel, early return, primary-error preservation | `cancel` restores the cell on failure (`:9141-9144`); `cancel_with` (`:9151`) and `record_http_cleanup_error` keep the first error | `double_failed_selection_preserves_the_primary_outcome_and_ends_at_zero` (`:15527`) asserts the *first* failure is the reported one |
| Both response bodies **and** permits closed | `PreparedMutationResponses::cancel` (`:3029-3074`) cancels both permits then releases both bodies | cell-drop test asserts `retained_http_bytes == 0` and `open_permit_count() == 0` |
| Duplicate owner deleted | `PreparedMutationResponsesOwner` has **zero** references in the tree | `grep -n PreparedMutationResponsesOwner` → no hits |
| `work_budget.rs` invariant | `:729-755` names exactly two authorized typed sealers and forbids simultaneous owners | I enumerated every `seal_prepublication()` call site: production sealers are exactly `http.rs:3584` / `:3990` (`PreparedHttpMutationTransaction`) and `http.rs:9025` (`MergeRequestResponsePreflight`); `publication.rs:243` is `#[cfg(test)]`-gated, and all other sites are test bodies. The documented claim is accurate. |

`PreparedMutationResponses::Drop` (`:3077-3087`) is still a `debug_assert`. That is now correct by
construction rather than by luck: every production holder is a ledger-holding fail-closed owner
(`HttpRouteContext`, `PreparedHttpMutationTransaction` `:3978`, `PreparedHttpMutationTerminal`
`:4230`), and the cell's doc comment (`:4268-4279`) states exactly why a ledger-less owner cannot do
better than assert. Recorded as a residual, not a finding (§8, R-3).

### 4.4 F4 — independent response boundaries · CLOSED

| Route | `response_bytes` | `response_header_bytes` | `retained_bytes` | `temporary_bytes` |
| --- | --- | --- | --- | --- |
| create | exact/−1/+1 `:14878-14991` | exact/−1/+1 same loop | exact/−1/+1 same loop, filler injected *inside* the preflight (`:14919`) | not a charged dimension of the preflight; ownership asserted `== 0` on every exit (`:13978`) |
| review | exact/−1/+1 `:15060-15147` | exact/−1/+1 same loop | exact/−1/+1 in the pre-existing `review_retained_limit_minus_exact_plus_one_is_effect_safe` (`:13814`) | as above |

- **Success body separated from the larger durable body.** Both tests assert
  `durable_bytes > success_bytes` and `response_upper == durable_bytes` (`:14863`, `:15043`), assert
  `durable_bytes == 149` as a cross-runtime constant (`:14859`), and then pin the *exact cumulative
  charge* separately — `baseline_success_charge == success_bytes` (`:14868`, `:15048`) and, for
  review, `baseline_header_charge == http_json_header_len(200, success_body.len())` (`:15052`). A
  success-body-only regression moves neither response boundary but does move those charges, which is
  precisely the gap F4.2 named.
- **Each failure reaches the intended owner.** Every `plus-one` case asserts
  `merge_request_preflight_was_reached()` (new `#[cfg(test)]` observation set at the top of
  `encode`, `:8906-8907`), `merge_request_preflight_retained_before() > 0`, and
  `error.diagnostic()["dimension"] == name`, together with a 413 status and both snapshots
  unchanged. For the response/header dimensions the filler is a *cumulative* precharge, so the
  preflight's reservation is the only acquisition of that dimension in the route.
- **Failure/deadline/disconnect/replay/cleanup with exact bytes and snapshots.**
  `merge_request_create_disconnect_and_replay_are_byte_exact` (`:15860`),
  `merge_request_disconnect_releases_the_prepared_owner_without_debt` (`:15961`),
  `merge_request_routes_poll_the_deadline_at_every_preflight_fence` (`:15152`),
  `merge_request_publication_handoff_cannot_be_lost_to_a_late_deadline` (`:15246`).
- **Narrow displacement, not a gap in coverage:** the review route's `retained_bytes` exact/−1/+1
  boundary lives in the pre-existing test, which asserts `prepared.is_none()` and an unchanged MR
  record but does *not* assert `merge_request_preflight_was_reached()`. Every other review boundary
  does. Recorded as residual R-1.

### 4.5 F5 — dead-code and owner cleanup · CLOSED

- `#[allow(dead_code)]` count in `http.rs`: **17 → 1**
  (`git show 7e4e685:crates/codefire-cli/src/http.rs | grep -c allow(dead_code)` = 17;
  candidate = 1). Every suppression on the production-wired response-preparation surface
  (`prepare_mutation_responses_with_deadline`, `PreparedMutationResponses` and its impl,
  `RETRY_SAME_REQUEST_RESPONSE`, `encode_prepared_mutation_response`,
  `http_response_work_reservation`, `release_prepared_mutation_response`,
  `release_prepared_mutation_pair`) is gone.
- The "intentionally staged before its production route wiring" comment is gone.
- `PreparedMutationResponsesOwner` and its test surface are deleted; no route holds two nearly
  identical owners. Both production wrappers now share one `PreparedMutationResponses` state
  machine with one commit point (F3.5 satisfied).
- **No replacement suppression was added.** `#[expect(...)]` count is 1 in both the parent and the
  candidate (the pre-existing `clippy::result_large_err` at `:5179`); the diff adds no `allow`/`expect`
  attribute. `cargo clippy -p codefire-cli --bin codefire-rs --locked -- -D warnings` and
  `cargo clippy -p codefire-core --lib --locked -- -D warnings` both pass with zero warnings.
- Finding F-4: the surviving attribute at `:3192` is on the whole
  `impl PreparedHttpMutationTransaction` block — 28 methods — while its new comment claims it covers
  only `attach_object_batch`, `attach_publication`, and `attach_responses`. It is pre-existing, but
  the comment is narrower than the attribute. Separately (residual R-2), the ticket's stronger reading
  ("strict Clippy must *reject* replacement suppressions") is not achievable with `-D warnings` alone:
  no gate in the ladder would flag a re-added redundant `allow`.

---

## 5. Required verdicts — R1 through R4

| # | Required item | Verdict |
| --- | --- | --- |
| R1 | Post-commit cleanup cannot replace a committed response; inject before and after the first effect | **PASS — closed** (evidence gap on the "before" injection *mechanism*, §5.1) |
| R2 | Transactional selection; fail after each internal transfer; double failure preserves the first outcome | **PASS — closed** |
| R3 | Directory-complete fail-closed snapshots with deliberate directory/symlink/unreadable fixtures | **PASS — closed** |
| R4 | Typed unique deadline phases; independent selection; stable public strings; duplicate identity fails | **PASS — closed** on its create/review obligations (in-file overclaim, §5.4 / F-3) |

### 5.1 R1 — post-commit cleanup · CLOSED

- **Create has no fallible accounting release after its first effect at all.** All three
  precommit-completable releases were moved into one block before the escrow (`:9399-9416`), and the
  production comment says exactly why (`:9391-9398`).
- **Review genuinely cannot move its releases** — the authority clone, locked authority JSON, and
  planned record/status/subject growth are read by `execute_json_publication` itself — so its
  post-publication chain (`:10184-10224`) records a cleanup failure as the request's single terminal
  condition and publishes the already frozen document (`:10225-10240`,
  `record_http_post_commit_cleanup_failure` `:10251-10263`).
- **Exactly one document.** `post_commit_cleanup_failure_preserves_the_frozen_response` (`:15574`)
  asserts a 200 result, `wire.starts_with("HTTP/1.1 200 OK")`, and
  `wire.matches("HTTP/1.1").count() == 1`, plus terminal reason `CorruptSubject`, byte content
  (`"decision":"approve"`), the durable record, and `assert_request_finished_clean`. The same test
  covers **create** through `PreparedSelectionFault::AfterCommit` (`:15628`), the transition where a
  committed handoff must never become a new error document.
- **`write_http_dispatch_outcome` deliberately does *not* absorb.** Its error arm releases the
  planned body and writes the error document (`:947-966`), with a comment explaining that only the
  owner knows whether the effect became durable. That is the correct division and is asserted by the
  pre-effect tests, which all check `wire.starts_with("HTTP/1.1 400"/"503")`.
- **Evidence gap (residual R-4):** the "inject before the first effect" half is exercised as a
  *preflight* failure, a *deadline* trip, and a *budget* rejection — never as an accounting-release
  failure. Create's `precommit_release` error arm (`:9417-9429`) has no test, and it re-calls
  `destination_ownership.release` after the closure may already have released it; the second call
  returns the typed "already released" diagnostic, which `record_http_cleanup_error` correctly
  absorbs as a secondary error, so the primary outcome survives. Unreachable except through an
  accounting bug (`release` fails only on underflow).

### 5.2 R2 — transactional fallback · CLOSED

- **One commit point.** `PreparedMutationResponses::select` (`:2904-2988`) only *reads* until
  `consume_selected_work` (`:2993-3027`), which calls the new
  `DecisionWorkLedger::consume_reserved_pair_exact_and_release`
  (`crates/codefire-core/src/work_budget.rs:1006-1053`). That method stages both actuals **and** the
  unselected body's release on a clone, revalidates every other outstanding permit's headroom against
  the staged state, and only then assigns `self.snapshot` and removes both permits. Everything after
  the commit in `select` is two infallible moves, and the code says so (`:2975-2976`).
- **Failure points after each internal transition.** `BeforeValidation` (`:2911`), `BeforeCommit`
  (`:2966` — after validation, after both frozen reservations are cloned, before the ledger commit),
  `AfterCommit` (`:9094` — after the body already belongs to the writer). The doc comment
  (`:1446-1462`) states there is deliberately no point *between* permit consumption and the body
  handoff because the atomic ledger call makes that one operation — which I verified by reading:
  no reachable intermediate state exists there.
- **A partially failed success selection really does fall back.**
  `partially_failed_success_selection_still_selects_the_frozen_durable_response` (`:15443`) runs both
  fault points through the production review route to a 503 with the exact 149-byte body and zero
  residual, then checks the raw owner directly: after a commit-point rejection both bodies, both
  permits, and both frozen work reservations survive (`:15501-15509`), retained bytes and permit count
  are unchanged, `cumulative` is still empty, and `select_durable` then succeeds (`:15511`). This is
  the exact scenario the prior review flagged as advertised-but-undemonstrated.
- **Double failure preserves the primary outcome.**
  `double_failed_selection_preserves_the_primary_outcome_and_ends_at_zero` (`:15527`) asserts the
  reported error is the *first* failure, the wire is a single 400, and the ledger closes at zero with
  one terminal reason.
- **Core-level atomicity is independently pinned.**
  `publication_permit_pair_exact_release_commits_or_changes_nothing` (`work_budget.rs:1546-1687`)
  drives over-consumption and release-underflow rejections, asserting `snapshot` unchanged and both
  permits still open, then the successful commit's exact charges, discarded remainders, and
  double-consume rejection. Mutation **M4** breaks the atomicity and this test fails (§6).

### 5.3 R3 — directory-complete snapshots · CLOSED

`remote_project_snapshot_records_directories_symlinks_and_fails_closed` (`:16034-16130`) deliberately
creates and removes a directory, creates a valid symlink and a **dangling** symlink, removes the
snapshot root, and makes a subtree unreadable (`0o000`, guarded by an `enforced` check so the case is
skipped rather than falsely passing when the mode is not enforced for this user). Each is asserted to
be *observed* or to *fail closed*, using `catch_unwind` so the panic is the assertion. Files,
directories, symlink kind and target, and exact bytes are all recorded; `locks/` is excluded with a
stated reason (acquiring a gate is not an authority effect). Mutation **M5** makes the walker return
silently on `read_dir` and this test fails (§6). File mode/ownership is not recorded — no route in
scope changes it, so this is noted rather than charged.

### 5.4 R4 — typed unique deadline phases · CLOSED on its create/review obligations

- **Public strings are byte-identical to the parent.** I diffed the full set of
  `.check("…")`/`.remaining_ms("…")` literals between `7e4e685` and `73bc532`: **empty diff.** R4.2
  satisfied exactly.
- **Independently validated uniqueness table.** `HTTP_DEADLINE_PHASE_TABLE` (`:85-186`) has 100
  entries; I recomputed `len == unique == 100`.
  `http_deadline_phase_table_is_unique_and_covers_every_poll` (`:16137-16196`) rejects a duplicate
  entry, asserts stable index identity for all 100, feeds
  `http_deadline_phase_id_in` a deliberately duplicated table and asserts the rejection, asserts an
  unknown phase has no identity, and scans the file for polled literals missing from the table.
  Mutation **M6** adds a duplicate entry and this test fails (§6).
- **Each create/review phase is independently selectable.** None of the merge-request or review phase
  names is used at more than one poll site, so arming one trips exactly one fence;
  `merge_request_routes_poll_the_deadline_at_every_preflight_fence` (`:15152`) exercises them and
  asserts the exact phase in the returned diagnostic.
- **In-file overclaim (Finding F-3).** The table's doc comment (`:74-83`) claims fault injection
  *"resolves a phase name to its unique index here rather than comparing strings, so two phases that
  happened to share a name could no longer be tripped together."* That is false: the index is a pure
  function of the string, so co-named sites still share one identity. Nine names in this file are
  polled at 2–4 distinct sites (`upload object-set digest` ×4; `response encoding`, `request
  dispatch`, `apply planning`, `apply ancestry validation`, `apply subject validation`, `apply
  publication preflight`, `upload request signature preflight`, `upload request signature validation`
  ×2 each), and `response encoding` and `request dispatch` are both on the create/review paths.
  Separately, the coverage scan only sees literal `.check("…")` / `.remaining_ms("…")` sites — 60 of
  the 100 entries. The other 40, **including every merge/review response- and publication-preflight
  phase**, reach `check` through a `&'static str` parameter and are validated only at run time, and
  only once some trip is armed (`http_deadline_trips_at` short-circuits on `armed.is_none()`,
  `:236-237`).

---

## 6. Production reachability and deliberate-regression evidence

### 6.1 Live call graph through the production dispatcher

`dispatch_http_with_ledger` builds the real `HttpRouteContext` — including the new
`response_preflight: None` cell (`:5900-5905`) — and routes:

- `("POST", ["merge-requests"])` → `handle_http_request_merge` (`:5972`)
- `("POST", ["merge-requests", mr_id, "review"])` → `handle_http_request_review` (`:5974`)

Both handlers construct `MergeRequestResponsePreflight` on that context (`:9449`, `:10147`), park the
pair in the cell (`:8973`), seal (`:9532`, `:10153`), publish through
`remote::publication::execute_json_publication`, and hand the frozen buffer to
`write_http_dispatch_outcome`'s prepared branch (`:968-983`), which writes it without a second
encoding. The context is dropped before the ledger is reused, because it borrows the ledger mutably —
so its `Drop` always runs first.

End-to-end reachability is exercised over the real wire, not just at route level:
`merge_request_create_and_review_publish_the_exact_frozen_preflighted_response` (`:14456`) builds raw
HTTP bytes and calls `handle_http_stream` (`:14467`) for create, create-replay, review, and
review-replay, asserting byte-identical replays, `merge_request_preflight_encodings() == 1`,
`http_response_encodings() == 2` (the frozen pair only), the exact escrowed id set, and unchanged
snapshots. `tests::file_remote_upload_clone_show_diff_and_merge_request_flow` and
`tests::merge_request_subject_matches_cross_runtime_golden_vector` also pass on the candidate.

### 6.2 Deliberate regressions (authored and run by me, in the scratch tree only)

Each mutation breaks one exact property while preserving nearby helper calls. Pristine sources were
restored and re-verified after every run.

| # | Property broken | Mutation | Result |
| --- | --- | --- | --- |
| **M1** | F1.4 first-effect ordering | `MergeRequestResponsePreflight::prepare` moved after `source_records.publish_with_work_and_poll`; the escrow-failure arm keeps `record_http_cleanup_error` + `source_records.release` | **FAIL as required** — `21 passed; 4 failed` on `merge_request`. Failures: `merge_request_preflight_precedes_the_observable_object_escrow` (`:14697`, target object store no longer byte-identical), `injected_merge_request_preflight_failure_is_typed_and_pre_effect` (`:15304`), `merge_request_create_response_limits_are_exact_and_pre_effect` (`:14965`), `merge_request_routes_poll_the_deadline_at_every_preflight_fence` (`:15184`). Reproduces the implementation's claimed 4-test reversal exactly. |
| **M2** | F3.1 fail-closed seal | the `is_prepublication_sealed()` witness check deleted from `seal` | **FAIL as required** — `merge_request_response_preflight_seal_fails_closed_on_a_second_or_external_seal` panics at `:15693` (`expect_err("a second or external seal must be rejected")`), then the handle's own `Drop` assertion fires at `:9156` during unwind |
| **M3** | R1 committed-response preservation | review's absorption arm returns `Err(cleanup_error)` instead of `Ok(response)`, keeping the `record_http_post_commit_cleanup_failure` call | **FAIL as required** — `post_commit_cleanup_failure_preserves_the_frozen_response` panics at `:15591` ("a post-commit release failure must not replace the committed success") |
| **M4** | R2 selection atomicity | `consume_reserved_pair_exact_and_release` assigns `self.snapshot` and removes both permits before the release half validates | **FAIL as required** — `publication_permit_pair_exact_release_commits_or_changes_nothing` panics at `work_budget.rs:1626` (`"{label} must change nothing"` on the release-underflow case) |
| **M5** | R3 fail-closed snapshot | walker returns silently on `read_dir` error instead of panicking | **FAIL as required** — `remote_project_snapshot_records_directories_symlinks_and_fails_closed` panics at `:16055` ("a created directory must be observed"), then `:16066`, `:14130`, `:16117` |
| **M6** | R4 phase-identity uniqueness | `"merge-request response preflight"` duplicated in the table | **FAIL as required** — `http_deadline_phase_table_is_unique_and_covers_every_poll` panics at `:16141` ("listed twice; two phases sharing a name would trip together") |

After the last mutation the scratch sources were restored and re-verified:
`sha256sum` returned `1300d62b…3a98` (`http.rs`) and `50ce68b1…6419` (`work_budget.rs`), identical to
the values captured at extraction from `73bc532`.

Mutation classes the ticket told me to reject as insufficient — deleted helper, constant label,
helper counter, test-only stage jump, incomplete snapshot — were **not** used: M1 reorders real
production statements and keeps every helper call; M2/M3 delete a real guard and a real absorption
arm; M4 changes real ledger commit ordering; M5 changes the walker's failure mode, not its call
sites; M6 changes only the identity table.

---

## 7. Byte reconciliation and scaling audit

Measured on the candidate with an added print-only probe in the scratch tree
(`rereview_probe_exact_response_bytes`), through the production routes:

```text
PROBE create status=200 len=126 residual=0
  body={"id":"MR-16603f968381","source_head":"CF-COMMIT-8b399d8cc553fafa06a86759","target_head":"CF-COMMIT-563f0343a4a98c00474553a6"}
PROBE review status=200 len=60  residual=0
  body={"decision":"approve","id":"MR-probe","reviewer":"reviewer"}
PROBE durable len=149
  body={"error":"the durable mutation completed without a confirmed response; retry the same request","kind":"retry_same_request","retry_same_request":true}
```

| Claim | Measured | Result |
| --- | --- | --- |
| 126-byte create success | 126 | **match** (`{"id","source_head","target_head"}` with `MR-`+12 and two `CF-COMMIT-`+24 ids) |
| 149-byte / 503 durable retry | 149, status 503 | **match**, and pinned in-tree by `assert_eq!(durable_bytes, 149)` (`:14859`) |
| 57-byte review success | **60** | **mismatch — Finding F-2.** `{"decision","id","reviewer"}`; length varies with the MR id and reviewer strings, so 57 is not a constant of any fixture in the suite. No test or document asserts 57; only `73bc532`'s commit message does. |

Simultaneous peaks are stated as formulas and asserted, not recorded as constants:
`response_upper = max(success, durable)` and `header_upper = max(header(200,success), header(503,durable))`
(`:14851-14854`, `:15035-15038`); the preflight's retained peak is
`retained_upper = retained_before_preflight + success_bytes + durable_bytes` (`:14874`). Cumulative
response/header charges are pinned separately to the *selected* body's exact bytes.

**The 1x/3x/10x curve is the one piece of required evidence that does not hold up.** I reproduced the
candidate's own scaling loop with print-only instrumentation (`rereview_probe_scaling_axis`, scratch
tree, production route):

```text
PROBE scale x1 : source_total= 9 source_closure=8 target_before=8 target_after=10 object_records=32 aggregate_bytes=13852 response_bytes=126 retained_peak=41943 residual=0
PROBE scale x3 : source_total=11 source_closure=8 target_before=8 target_after=10 object_records=32 aggregate_bytes=13852 response_bytes=126 retained_peak=41943 residual=0
PROBE scale x10: source_total=18 source_closure=8 target_before=8 target_after=10 object_records=32 aggregate_bytes=13852 response_bytes=126 retained_peak=41943 residual=0
```

The source object store does grow (9 → 11 → 18 objects), but `source_closure` stays at **8**, the
escrow copies the same **2** records every time (target 8 → 10), and **every counter is byte-identical
at 1x, 3x, and 10x**: `object_records` 32, `aggregate_bytes` 13852, `response_bytes` 126,
`retained_peak` 41943. The three iterations are the same request run three times. `object_records` is
non-zero, so the counter is real — it simply never moves. See Finding F-1.

---

## 8. Findings, severity-ordered

### F-1 — MEDIUM · test-coverage / vacuous oracle · CONFIRMED by probe

**`merge_request_response_work_is_flat_while_object_work_scales`
(`crates/codefire-cli/src/http.rs:16198-16258`) varies nothing observable, so the required 1x/3x/10x
counter curve is vacuous — and its doc comment claims the opposite of what it asserts.**

The loop adds `multiplier` extra objects to the *source* object store:

```rust
for index in 0..multiplier {
    codefire_store::store_object(&source_objects, "trace_graph",
        json!({"type": "trace_graph", "version": 1, "links": [], "pad": index})).unwrap();   // :16210
}
```

Those objects have `"links": []` and are not referenced by the fixture's commit, so they are outside
the closure of `source_head`. The create route only ever collects the closure
(`collect_object_records_with_owned_work_and_poll` from `remote.rs:3050`), so the escrowed set — and
every counter derived from it — is identical at 1x, 3x, and 10x. **Measured** (§7): `source_closure`
stays at 8 while the source store grows 9 → 11 → 18; the escrow copies the same 2 records each time;
and `object_records` (32), `aggregate_bytes` (13852), `response_bytes` (126), and the retained peak
(41943) are byte-identical across all three multipliers. The test's own passing assertion says the
same thing:

```rust
assert_eq!(*object_records, base_objects,
    "{multiplier}x: only the source closure is escrowed, and it is the same closure");   // :16253
```

while the doc comment two lines above the `#[test]` says:

```rust
/// The contract's 1x/3x/10x curve: response work is independent of the
/// escrowed object count, while object work scales exactly with it.               // :16198-16199
```

Both halves are unexercised. "Object work scales exactly with it" is contradicted by the assertion
directly below it. "Response work is independent of the escrowed object count" is not demonstrated
either, because the escrowed object count never changed — the three iterations are the same request
run three times.

*Failure scenario.* Someone makes response work depend on the escrowed object count — for example by
folding the escrowed id set or record count into the response payload or into a per-record retained
charge. `response_bytes`, `header_bytes`, and `encodings` stay identical across all three multipliers
because the escrowed closure is identical, so this test still passes and the regression ships with the
contract's "1x/3x/10x counter curve" evidence item reported as satisfied.

*Reproducer.* `git archive 73bc532` into a scratch tree, duplicate the scaling loop with a print of
`source_closure`, `target_after`, `object_records`, `aggregate_bytes`, `response_bytes`, and the
retained peak, then
`cargo test -p codefire-cli --bin codefire-rs --locked rereview_probe_scaling_axis -- --nocapture`.
Output is in §7: nothing moves.

*Repair check.* Make the added objects reachable from `source_head` (link them into the commit's
graph, or write `multiplier` extra reachable records), then assert `object_records` *grows linearly*
with the multiplier while `response_bytes`, `header_bytes`, and `encodings` stay flat. Note that the
assertion at `:16253` would then have to be inverted — which is itself the tell that the present
version is vacuous.

*Scope.* This is a required-evidence item of the repair contract ("exact 1x/3x/10x counter curves
where work scales with object or response count") and an explicit audit item of this rereview
("Audit simultaneous peak formulas and 1x/3x/10x scaling, not only cumulative charges"). It is not one
of F1–F5 or R1–R4, all of which are closed.

### F-2 — LOW/MEDIUM · claim accuracy · CONFIRMED by probe

**The repaired candidate's only durable evidence artifact states a review success body size that is
wrong.** `73bc532` commit message: *"the exact 126-byte create success body, 57-byte review success
body, and 149-byte/503 durable-retry body are unchanged."* Measured: **60** bytes, not 57
(§7). The 126 and 149 figures are correct, and 149 is pinned by an in-tree assertion; 60 is pinned by
nothing.

*Failure scenario.* A later lane, a manager composing candidates, or the Python counterpart lane takes
57 as the Rust review-route success size — for a cross-runtime golden vector, a limit calculation, or a
regression baseline — and derives a wrong constant. Because there is no repair result document in the
repository (§2), this commit message is the authoritative record.

*Reproducer.* `git archive 73bc532` into a scratch tree, add a print of
`run_review(...).into_selected_response().1.len()` for the standard review fixture, run
`cargo test -p codefire-cli --bin codefire-rs --locked <probe>` → `len=60`.

*Remediation.* Correct the figure in whatever durable record the lane produces, and if 60 is worth
freezing, assert it next to the existing `assert_eq!(durable_bytes, 149)`.

### F-3 — LOW · claim accuracy in changed code · CONFIRMED by grep

**The phase-table comment (`crates/codefire-cli/src/http.rs:74-83`) claims a property the typed
identity does not deliver.** The index is a pure function of the phase string, so two poll sites that
share a name still share one identity and still trip together. Nine names in this file are polled at
2–4 sites; `response encoding` and `request dispatch` are among them and are on the create/review
paths. The uniqueness oracle validates the *table*, not the source's use of it.

*Failure scenario.* A future lane adds a second `check("review response preflight")` site (or reuses
any existing shared name for a new create/review fence) believing the table now prevents co-tripping.
Arming that phase trips both sites; whichever runs first wins, and a test intended to isolate the
later fence silently exercises the earlier one. The uniqueness test still passes, because the table
still holds one entry.

*Reproducer.*
`grep -o '\.check("[^"]*")' crates/codefire-cli/src/http.rs | sort | uniq -c | sort -rn | head`
→ `4 .check("upload object-set digest")`, then eight names at 2 each.

*Remediation.* Either state the real property (the table gives each *name* one identity and the
regression check rejects a duplicated table entry; co-named sites are still indistinguishable), or
key injection on a per-site identity — e.g. a `(phase, occurrence)` pair or distinct phase constants —
and extend the coverage scan to reject a name appearing at more than one site.

### F-4 — LOW · quality · residual suppression scope

`#[allow(dead_code)]` at `crates/codefire-cli/src/http.rs:3192` is attached to the entire
`impl PreparedHttpMutationTransaction` block (`:3192-3976`, 28 methods), while the comment introduced
by this candidate (`:3186-3191`) claims it covers only `attach_object_batch`, `attach_publication`, and
`attach_responses`.

*Failure scenario.* Any of the other 25 methods — including production-critical ones such as
`revalidate_and_seal`, `commit_effects_after_seal`, `select_durable_after_commit_error` — loses its
last caller during a later refactor and the dead code is never reported, because the block-level
allow suppresses it. Clippy `-D warnings` cannot catch a stale allow.

*Reproducer.* Delete the only production call to any method in that impl and run
`cargo clippy -p codefire-cli --bin codefire-rs --locked -- -D warnings` → still rc=0.

*Remediation.* Move the attribute onto the three attach methods, matching what the comment claims.
The attribute is pre-existing, so this is hardening rather than a regression this candidate
introduced.

---

## 9. Residual risks (owned, not blocking)

1. **R-1 — review's `retained_bytes` boundary does not assert owner attribution.** The review
   response and header boundaries assert `merge_request_preflight_was_reached()`; the pre-existing
   `review_retained_limit_minus_exact_plus_one_is_effect_safe` (`:13814`) does not. It does pin
   `diagnostic["dimension"] == "retained_bytes"`, `observed == limit + 1`, `limit`, `prepared.is_none()`,
   and an unchanged MR record, which is strong — but `observed == limit + 1` identifies the overshoot,
   not the acquisition that caused it, so this one boundary still does not prove the rejection landed
   at the response preflight rather than at an earlier retained acquisition. Every other boundary on
   both routes does.
2. **R-2 — `#[expect]`/`allow` re-addition is ungated.** No gate in the authorized ladder rejects a
   redundant suppression, so F5.4's stronger reading cannot be enforced mechanically. The candidate
   itself adds none (17 → 1 `allow`, `expect` unchanged at 1).
3. **R-3 — `PreparedMutationResponses::Drop` remains a `debug_assert`.** Safe by construction today:
   every production holder is a ledger-holding fail-closed owner. There is no test that panics inside
   a route to exercise the unwind path specifically; Drop-on-unwind is a language guarantee, and
   `fail_closed_http_mutation_drop` aborts rather than double-panics (`:4256-4261`).
4. **R-4 — no injected accounting-release failure before the first effect.** Create's
   `precommit_release` error arm (`:9417-9429`) is untested and re-calls
   `destination_ownership.release` after the closure may already have released it; the redundant call
   returns the typed "already released" diagnostic, which is absorbed as a secondary error so the
   primary outcome survives. Reachable only through an accounting bug.
5. **R-5 — `record_http_post_commit_cleanup_failure` degrades to a no-op in release builds.**
   `crates/codefire-cli/src/http.rs:10257-10262`: if `finish` is rejected the failure is only
   `debug_assert!(false)`. `finish` can only be rejected while a permit is open
   (`work_budget.rs:1255`), which a selected response precludes — but if it ever happened in a release
   build the request could finish with *no* terminal reason rather than exactly one. Pre-existing in
   kind.
6. **R-6 — the 149-byte/503 durable body is declared a cross-runtime wire contract with no
   cross-runtime oracle.** `docs/development/source-data-model-catalog.md:187-189` now states it
   *"must agree byte-for-byte with the Python HTTP path."* The Python side contains only negative
   assertions (`tests/test_codefire_cli.py:3799, 4115, 4339, 4624` assert `kind !=
   "retry_same_request"`); no golden vector pins the body across runtimes. Carried unchanged from the
   prior review's integration note; the repair contract did not require closing it.
7. **R-7 — snapshot metadata is bytes + kind only.** File mode and ownership are not recorded, so a
   permission-only change would not register. No route in scope changes them.

---

## 10. Additional product audit — results

| Audit item | Result |
| --- | --- |
| Second ledger, response owner, sealer, cleanup owner, or terminalizer | **None.** Exactly one request ledger; one shared `PreparedMutationResponses`; exactly two production sealers, matching the documented invariant (§4.3); every `finish` site in `http.rs` (`:979`, `:1071`, `:1154`, `:4031`, `:4246`, `:10257`) is guarded by `terminal_reason.is_none()` or tolerates `TerminalAlreadySet` |
| Hidden/unreserved allocation, unbounded retained state | **None.** The pair holds two `Vec<u8>` bodies, both charged as `RetainedBytes` before the effect; `response_upper`/`header_upper` are `u64`; the phase table is a `#[cfg(test)]` `&'static [&'static str]`; no new production collection grows with input |
| Permit consumption that can partially commit | **Not reachable.** `consume_reserved_pair_exact_and_release` stages actuals, releases, and other-permit headroom on clones and assigns `self.snapshot` only after all validate; both permits are removed in the same step; proven by the core test and by mutation **M4** |
| Panic/abort from malformed or resource-limited input | **None found.** New production `expect`s are all on `context.ledger`, which no route empties (`grep` for `context.ledger.take()` / `= None` → no hits); `fail_closed_http_mutation_drop`'s ledger-less branch is structurally unreachable; `http_deadline_phase_id`'s `expect` is `#[cfg(test)]`; budget/limit rejections are typed 413 `ResourceBudget`, deadline 503, malformed 400 |
| Response-after-effect, replay, nonce, escrow, signature, authentication, error precedence | **Preserved.** `verify_remote_request_signature` is still the first thing both routes do (`:9865` review; create unchanged above the first touched hunk); the escrow position is unchanged; replay drains through `replay_json_publication` before any write and republishes byte-identically; error precedence is preserved by `record_http_cleanup_error` / `cancel_with` / `(Err(error), Err(_)) => Err(error)` |
| Upload / apply / list / bundle / doctor behaviour and wire bytes | **Unchanged.** The only neighbour edits are mechanical consequences of `HttpRouteContext` gaining a `Drop`: `response_preflight: None` added at the two construction sites (`:5904`, `:6415`), `context.payload_owner` → `.take()` (`:6008`), and `handle_http_upload`'s inline temporary named as a `let` binding. Both have identical drop timing. All 153 `http::tests` and all 788 binary-target tests pass |
| Source-document overclaim; test-only behaviour shown as production | Docs are accurate on the two claims I could falsify: exactly two sealers, and the 149-byte durable body. **Overclaims found:** the phase-table comment (F-3) and the scaling-test comment (F-1), both in `http.rs`. Every injection hook is `#[cfg(test)]` + `thread_local!` and `reset_merge_request_preflight_observations` (`:8756`) clears all nine of them plus the deadline and encode counters. R-6 is a normative doc statement with no oracle |
| Duplicate typed phases, unreachable compatibility code, suppressed warnings | Table is unique (100/100); public phase strings byte-identical to the parent; 40 of the 100 entries have no literal poll site because they reach `check` through a `&'static str` parameter (§5.4), which weakens the coverage scan but is not dead code; `PreparedMutationResponsesOwner` deleted; suppressions 17 → 1 (F-4) |

---

## 11. Commands, counts, durations, unavailable evidence

Toolchain: `cargo 1.97.0 (c980f4866 2026-06-30)`, `rustc 1.97.0 (2d8144b78 2026-07-07)`.
Environment exactly as authorized:

```bash
export RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup
export CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=2
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared
```

Shared target path `/workspace/.cache/cargo-target-shared` (749 MB at review start), 14 cores.

**Focused probes first, then the authorized ladder — all on the pristine candidate tree:**

| # | Command | Result | Duration |
| --- | --- | --- | --- |
| 1 | `cargo test -p codefire-cli --bin codefire-rs --locked merge_request` | **pass** — `25 passed; 0 failed; 763 filtered out` | 109 s total (105 s compile, 4.00 s tests) |
| 2 | `cargo fmt --all -- --check` | **pass** (rc=0, no output) | 4 s |
| 3 | `cargo test -p codefire-core --lib --locked work_budget` | **pass** — `26 passed; 0 failed; 64 filtered out`, incl. `publication_permit_pair_exact_release_commits_or_changes_nothing` | 12 s (0.01 s tests) |
| 4 | `cargo test -p codefire-cli --bin codefire-rs --locked http::tests` | **pass** — `153 passed; 0 failed; 635 filtered out` (141 on the reviewed candidate → 12 new) | 38 s (37.72 s tests) |
| 5 | `cargo test -p codefire-cli --bin codefire-rs --locked` | **pass** — `788 passed; 0 failed; 0 filtered out` (776 on the reviewed candidate) | 221 s (72.33 s tests) |
| 6 | `cargo clippy -p codefire-cli --bin codefire-rs --locked -- -D warnings` | **pass** — rc=0, zero warnings | 40 s |
| 7 | `cargo clippy -p codefire-core --lib --locked -- -D warnings` | **pass** — rc=0, zero warnings | 7 s |
| 8 | `python3 docs/development/tools/check_source_docs.py` | **pass** — `44 modules, 34 commands, 12 object kinds` | 0 s |
| 9 | `python3 docs/development/tools/check_project_docs.py` | **pass** | 1 s |
| 10 | `git diff --check` | **clean** | 0 s |

No flake was observed: the full binary target passed `788/788` in one run (the prior review's
`import_workflow` flake did not recur).

**Scratch-tree probes** (`/workspace/.cache/cfr168-rereview-scratch`, own target dir
`/workspace/.cache/cfr168-rereview-target`, candidate blobs verified by `git hash-object`; pristine
sources restored and re-verified after every mutation):

| Probe | Result | Duration |
| --- | --- | --- |
| `rereview_probe_exact_response_bytes` (print-only) | create 126 / review 60 / durable 149 (§7) | 162 s (first build in a cold target dir) |
| **M1** preflight after escrow | `21 passed; 4 failed` — F1 barrier confirmed | 104 s |
| **M2** seal witness removed | seal test fails at `:15693` | 112 s |
| **M3** post-commit absorption removed | absorption test fails at `:15591` | 125 s |
| **M5** snapshot fails open | snapshot test fails at `:16055` | 108 s |
| **M6** duplicate phase identity | uniqueness test fails at `:16141` | 117 s |
| **M4** non-atomic pair commit | core atomicity test fails at `work_budget.rs:1626` | 18 s |
| `rereview_probe_scaling_axis` (print-only) | every counter identical at 1x/3x/10x (§7) — F-1 confirmed | 115 s |

**Unavailable / not run, and why:**

- **The repair lane's WIP commits and its handoff evidence.** Not present in this worktree's object
  store (§2). I could not verify any intermediate state, only the final one.
- **No repair result document exists** in `docs/development/ai/swarm-results/`, so the lane's claimed
  formulas, snapshot hashes, and deliberate-regression results could not be checked against mine
  item by item; I re-derived them independently instead.
- **Release-build behaviour** was not measured. Every `debug_assert`-dependent claim
  (`PreparedMutationResponses::Drop`, `record_http_post_commit_cleanup_failure`, the publication
  `debug_assert_eq!` at `:9550` and `:10164`) is assessed by reading, plus the cell-drop test's
  `std::mem::forget` which removes the handle assertion from the path. A `--release` test run is not
  in the authorized ladder.
- **Docker, install, dogfood, unrestricted all-language discovery, and the full Rust workspace** were
  not run, per the validation boundary. Python-side behaviour was inspected by grep only.
- **Worker-reported gate results were treated as context, not evidence.** Every number in the table
  above is from a command I ran in this worktree.

---

## 12. Integration and write-overlap map

**This review's write set.**
`docs/development/ai/swarm-results/cfr168-rust-http-mr-preflight-zero-residual-rereview-claude-20260730/review.md`
only — a new directory that exists on no other branch. **Zero overlap** with product code, tests,
source contracts, issue state, or manager ledgers. One commit; worktree left clean.

**The candidate's own overlap surface.**

| Path | Overlap risk |
| --- | --- |
| `crates/codefire-cli/src/http.rs` | **High.** ±2,437/−557 across `HttpRequestDeadline`, the response-preparation helpers, `PreparedMutationResponses`, `HttpRouteContext`, both merge routes, and ~4,000 lines of tests. Any concurrent `http.rs` lane will conflict textually. |
| `crates/codefire-core/src/work_budget.rs` | **Medium.** One new public method (`consume_reserved_pair_exact_and_release`, `:1006`), one doc-comment rewrite (`:729-755`), one new test. Additive; no existing signature or limit changed. |
| `docs/development/change-playbook.md`, `command-source-trace.md`, `source-architecture.md`, `source-data-model-catalog.md` | **Low.** Additive paragraphs, except two replaced lines in `source-architecture.md`. Both source-document gates pass. |

- `main` = `f1644122134d479333cf422fdb1d48b8fa92dd8e`; `git merge-base HEAD main` = `main`, so this line
  is a linear 24-commit descendant. The candidate applies with no conflict on its own line by
  construction (it is already committed there).
- **Every other `agent/cfr168-*` branch that touches `http.rs` or `work_budget.rs`** —
  `capability-metrics`, `consumer-owner`, `digest-integration`, `digest-owner`,
  `doctor-branch-accounting`, `doctor-conflict-accounting`, `doctor-consumer`,
  `doctor-report-accounting`, `execution-evidence-accounting`, `execution-evidence-phase-a`,
  `existing-effect-input`, `existing-filesystem-reducer-packet`, `existing-reducer-freeze`,
  `http-bootstrap`, `main-integration-audit`, and the `python-*` set — shares merge-base `main` and is
  a stale predecessor of this integration line, whose `http.rs` already contains strictly more work.
  Landing any of them requires a rebase, not a merge. Same conclusion as the prior review, re-derived.
- **Semantic adjacency to watch:** `agent/cfr168-python-mutation-response-preflight` implements the
  same contract for the Python HTTP path. `source-data-model-catalog.md:187-189` now declares
  `RETRY_SAME_REQUEST_RESPONSE` a cross-runtime wire contract; nothing pins it across runtimes
  (residual R-6). The manager should treat the 149-byte body and its 503 status as a shared constant
  when composing the two lanes.
- **Shared-symbol adjacency:** `work_budget.rs:729-755` now names the *only two* owners permitted to
  seal a request ledger. Any later route adopting a prepared terminal response must reuse one of them
  rather than introduce a third sealer; `MergeRequestResponsePreflight::seal`'s witness check
  (regressed as M2) is the guard that would catch a violation.

---

## 13. Decision

**REQUEST_CHANGES**

- **F1, F2, F3, F4, F5, R1, R2, R3, R4: all closed.** No product defect found under reading, the full
  authorized gate ladder, or six independent deliberate regressions.
- **Blocking, and confined to evidence/claims:**
  - **F-1 (MEDIUM)** — the required 1x/3x/10x counter curve is vacuous: the varied objects are
    unreachable from `source_head`, so no counter moves at 1x, 3x, or 10x, and the test's doc comment
    claims object work scales while its own assertion proves it did not.
  - **F-2 (LOW/MEDIUM)** — the candidate's reported 57-byte review success body is wrong (60), and
    because no repair result document exists in-repo, that commit message is the whole evidence
    trail.
- **Non-blocking:** F-3 (phase-identity comment overclaim), F-4 (impl-wide `allow(dead_code)` scope),
  residuals R-1 through R-7.
- **Provenance:** the WIP rewrite did not damage the reviewed candidate, the report, the contract
  freeze, or any product byte, but it did leave the lane's intermediate evidence unrecoverable and no
  durable evidence document in the repository. A clean single-commit history is explicitly **not**
  accepted here as proof the process contract was followed.
- Remediation is test- and claim-side and small: fix the scaling fixture so the object axis actually
  moves and invert the assertion accordingly, correct the byte figure in a durable record, and
  optionally correct the two overclaiming comments. The production change needs no modification.

Reviewer worktree left clean; only this report was committed.
