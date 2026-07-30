# CFR-168 Python HTTP MR Mutation Independent Review (Claude Fallback)

Review date: 2026-07-30
Reviewer: Claude Opus 5 (`claude-opus-5`), fallback for the interrupted Fugu 1.1 Ultra session
Review worktree base: `9f1fe186df3002198a184fdced4691123037d71f`
Review batch: `docs/development/ai/swarm-batches/cfr168-python-http-mr-mutation-review-claude-fallback-20260730.md`
Continued contract: `docs/development/ai/swarm-batches/cfr168-python-http-mr-mutation-independent-review-20260730.md`
Lane contract: `docs/development/ai/swarm-batches/cfr168-python-http-mr-mutation-lane-20260729.md`

**Decision: `REQUEST_CHANGES`.**

## Exact subject

| Role | Revision |
|---|---|
| Baseline / task revision | `cab3126f216c595c0d2211174b364e8532a310e2` |
| Completed target under review | `21fe75175e997cc7c6f8c3ef2a7558144d850f94` |
| Pre-fix production snapshot named by the lane contract | `e2a218e015953f082245f36dbc6aee906b837e5a` |
| Target branch | `claude-agent/cfr168-python-http-mr-mutation-recovery-20260729-mr-mutation-restart` |

Ancestry is linear inside the lane, and the target is not an ancestor of this reviewer worktree:

```text
e2a218e015953f082245f36dbc6aee906b837e5a
  -> cab3126f216c595c0d2211174b364e8532a310e2
     -> 21fe75175e997cc7c6f8c3ef2a7558144d850f94
```

Cumulative diff `cab3126..21fe751`: `codefire` (+1605/-133) and
`tests/test_python_http_mr_mutation_decision_work.py` (+1246), 2718 insertions and 133 deletions
across two files. All `codefire` line references below are line numbers **at the target revision
`21fe751`**.

## Summary

The lane's *visible* architecture is correct and is a real improvement over `e2a218e`. All three
mutating routes now receive the handler's exact `http_v1` request ledger and
`HTTPRequestDeadline`, create no second `http_request` ledger, freeze the success and durable-retry
responses before their first effect, revalidate a byte-exact authority fence, and publish no partial
HTTP success. Both provisional ownership gaps flagged by the interrupted Fugu session are
**reproduced, not rejected**, and they are the reason for `REQUEST_CHANGES`:

1. Material, input-scaling decision work still runs outside the request ledger — in hidden
   `local_v1` "sealed validation" ledgers created per call by `validate_sealed_commit`, plus wholly
   unaccounted `read_json` / `collect_object_records` / `copy_object_graph` traversals in the
   authorization, record-validation, and subject-materialization phases. Measured live, the hidden
   ledgers read **2.3x to 3.4x more object bytes than the request ledger charges**, and the gap
   grows with input size. The route therefore claims one request-wide decision owner while doing the
   majority of its work in independent ledgers that neither the `http_v1` budget nor the 30 s
   request deadline governs.
2. The nonce semantic change is *required* atomicity, but its recovery/replay design is incomplete:
   all three routes accept and validate a fresh nonce on the replay path and then discard it without
   ever recording it, so a captured signed request whose idempotency key already has a receipt is
   replayable without bound inside the signature skew window.

Two further defects were found beyond the named review subjects: the legacy subject-less review path
converts a deterministic typed failure into a permanent `503 retry_same_request` while leaving added
escrow object bytes behind (Finding 3), and the final fence reports a same-path grow-tamper as a
bounded-limit error instead of its typed fence-changed diagnostic (Finding 5).

The reported 18/18 focused result is reproduced and is genuinely non-vacuous, but it is **evidence
only**: the module's own ledger-identity test deliberately admits nested non-`http_request` ledgers
(`tests/test_python_http_mr_mutation_decision_work.py:486-491`), which is precisely why the passing
suite does not discharge the lane contract's "never create a nested `local_v1` ledger" clause.

Per the memory-constrained instruction for this wave, no broad Python suite and no Cargo build were
run in this review, and neither is claimed to have passed.

## Findings, severity-ordered

### F1 — Blocking. Material decision work escapes the request ledger and the request deadline

`_HTTPMergeRequestLane` (`codefire:12162`) threads the handler ledger and deadline into its own
owners, but the dominant I/O of every route is performed by shared helpers that take neither.

Hidden ledger construction:

- `validate_sealed_commit` (`codefire:30118-30129`) constructs
  `DecisionWorkLedger(DecisionWorkProfile.local_v1(), "sealed validation")` per call and passes no
  deadline to `SealedCommitValidationSession`.
- It is reached on the live path of all three routes from
  `preflight_remote_json_publication` -> `validate_remote_json_publication_identity`
  (`codefire:3237-3245`, `codefire:3171`), once per object root.
- Review and apply reach it again from `validate_http_merge_request_record` (`codefire:12417`),
  once per referenced ref (`source_head`, `target_head_at_request`, and `applied_head` when set).

Wholly unaccounted traversals:

- `collect_object_records` (`codefire:8812-8829`) — no ledger, no poll, unbounded `read_json` per
  record — is called by `validate_merge_request_subject` (`codefire:21808`) and therefore twice per
  review and twice per apply.
- `copy_object_graph` (`codefire:8799-8809`) — no ledger, no poll — is create's first durable
  effect (`codefire:12643-12647`) and is also the legacy review path's first effect via
  `materialize_merge_request_subject` (`codefire:21856-21861`).
- `read_json` (`codefire:1429-1432`) is `path.read_text()` + `json.loads` with **no size limit and
  no charging**, and the whole authorization phase runs on it:
  `authorize_remote_without_nonce` (`codefire:6811-6831`) ->
  `check_remote_permission` / `check_remote_auth` (`codefire:6737-6784`) ->
  `verify_remote_request_signature` (`codefire:3934`) -> `request_nonce_cache_postimage`
  (`codefire:3776-3846`), which materialises the entire nonce cache (default cap 10,000 entries)
  into a fresh dict. The success path materialises that cache **again** at publication time
  (`codefire:12930-12939` review, `codefire:13234-13243` apply, create equivalently).

Live measurement, `chain_length=3`, one fixture, real sockets through `make_http_handler`:

| Route | `http_request` ledgers | nested `local_v1` ledgers | hidden `record_reads` | hidden `aggregate_bytes` | charged `record_reads` | charged `aggregate_bytes` | unaccounted `collect_object_records` | unaccounted `copy_object_graph` |
|---|---|---|---|---|---|---|---|---|
| create | 1 | 5 | 44 | 22,319 | 10 | 6,491 | 0 | 1 |
| review | 1 | 3 | 27 | 13,995 | 10 | 7,935 | 2 (10 records each) | 0 |
| apply | 1 | 5 | 41 | 19,301 | 3 | 7,688 | 2 (10 records each) | 0 |

Apply, scaled 1x / 3x / 10x on the same probe:

| `chain_length` | nested ledgers | hidden `record_reads` | hidden `aggregate_bytes` | charged `record_reads` | charged `aggregate_bytes` | unaccounted collect sizes |
|---|---|---|---|---|---|---|
| 1 | 5 | 37 | 15,277 | 1 | 5,676 | 8, 8 |
| 3 | 5 | 41 | 19,301 | 3 | 7,688 | 10, 10 |
| 10 | 5 | 55 | 33,385 | 10 | 14,730 | 17, 17 |

Hidden work grows monotonically with the input graph, so this is not a fixed constant overhead.

Failure scenario: a client posts an apply for a merge request whose escrow graph is large but whose
charged frontier is small. The request ledger's `object_records`, `aggregate_bytes`, `record_reads`
and `retained_bytes` all stay inside the `http_v1` profile, so no `413` is raised, while the process
actually reads and materialises roughly 2.3x-3.4x that byte volume under `local_v1` limits. Because
`validate_sealed_commit` receives no deadline, the 30 s `HTTPRequestDeadline` is only observed at the
lane's named phase boundaries and cannot interrupt a long sealed-validation walk; the request
overruns its deadline without a typed `503`. The same request also reads the whole escrow object set
three times per review (twice unaccounted, once charged) and up to three unbounded `read_json` reads
of `server_policy.json` plus two full materialisations of the nonce cache.

This is not a shared-helper limitation. The repository already contains the accounted pattern the
lane needed: the `do_GET` bundle path builds `SealedCommitValidationSession(objects_root, ledger)`
and passes `poll=` for both validation and collection (`codefire:11335-11346`), and the upload
mutation route owns its sealed validation on the request ledger through
`prepare_http_upload_sealed_validation(decision_work, ...)` and
`sealed_validation_owner.validate(decision_work)` (`codefire:13398-13408`), closing it against the
same ledger in its `finally` (`codefire:13678-13685`). The lane contract's "never create a nested
`local_v1` ledger or reopen work in an unaccounted compatibility fallback" is therefore both
explicit and achievable here.

### F2 — Blocking (test coverage). The exact / minus-one / plus-one boundaries do not bound the request

The boundary harness itself is sound: `run_boundary`
(`tests/test_python_http_mr_mutation_decision_work.py:894-972`) drives real requests at the exact
limit and at limit-1 (and limit+1 for `response_bytes`), asserts `published` versus `rejected`, checks
the typed `413` diagnostic dimension, asserts `terminal_reason == "budget_exceeded"`, compares the
full pre/post storage tree for byte equality, and proves gate release by a following successful
request. The 1x/3x/10x curves are real and the module fails hard against `e2a218e`.

The defect is scope, not mechanics. The ledger-identity test explicitly tolerates the hidden
ledgers:

```python
# No compatibility fallback reopened work on a second HTTP ledger.
for ledger in record.ledgers:
    if ledger is request_ledger:
        continue
    self.assertNotEqual(ledger.command_kind, "http_request")
    self.assertEqual(ledger.profile.request_deadline_ms, None)
```

(`tests/test_python_http_mr_mutation_decision_work.py:486-491`.) Every `local_v1` sealed-validation
ledger satisfies both assertions, so the suite asserts something strictly weaker than the contract:
"no second *HTTP* ledger" instead of "no nested ledger". Failure scenario: F1 can be fixed, partially
fixed, or regressed later without any test in this module changing colour, and the "exact" counters
that reviewers and the asymptotic/order table will consume under-report the route's real record
reads by 77% (create), 73% (review) and 93% (apply) at `chain_length=3`.

### F3 — Medium. Legacy subject-less review: post-effect preflight failure yields a permanent `503` and non-restored escrow bytes

Ordering is as reported and is correct as far as it goes: for a record without `subject`, the lane
revalidates the fence, freezes the response and seals the effect *before*
`materialize_merge_request_subject` (`codefire:12872-12888`), so encoding and header failures are
impossible after the first effect. But `preflight_remote_json_publication` runs *after* that first
effect (`codefire:12948-12959`), and any failure there is routed through
`_HTTPMergeRequestLane.recover_from_effect_failure` (`codefire:12355-12369`), which sees
`effect_in_flight` true and `publication_succeeded` false and returns
`503 HTTP_MUTATION_RETRY_PAYLOAD`.

Probe: a legacy record whose escrow copy was removed, with one injected typed `CFError` from
`preflight_remote_json_publication`:

```text
status: 503
body  : {"error":"the durable mutation completed without a confirmed response; retry the same request",
         "kind":"retry_same_request","retry_same_request":true}
materialize_merge_request_subject ran: True
escrow object files added: ['commits/CF-COMMIT-339d83356a02b528e70d6639.json']
target authority entries changed: ['objects/commits/CF-COMMIT-339d83356a02b528e70d6639.json']
durable state byte-identical: False
terminal_reason: complete
open permits: 0
retry status/body: 503 (identical bytes)
retry durable state identical to first failure: True
```

Two clauses fail. The lane contract requires that a typed failure leave **object** state
byte-identical to pre-call state; one escrow commit record remains. And the contract requires that
existing status/error precedence be preserved: a *deterministic* preflight failure — for example a
sealed-commit validation error on the escrow set — was a `400` with its diagnostic at `e2a218e`, and
is now a `503` instructing the client to "retry the same request" forever. The probe confirms the
retry reproduces the identical `503` indefinitely.

The narrower required-review item 4 is *satisfied* on its second disjunct: the failure recovers to
one durable byte-identical result, and repeated retries converge. `terminal_reason` is single and
there are zero open permits. So this is a real but non-catastrophic ordering defect: the fix is to
move the publication preflight (and its sealed validation) ahead of materialization, or to classify
non-retryable preflight errors so they are not masked as retryable.

### F4 — Medium. Recovery/replay paths accept a valid nonce and never record it

`verify_remote_request_signature(..., defer_nonce=True)` (`codefire:4049-4079`) returns an
unpublished `nonce_plan` and preserves replay/capacity/malformed-cache precedence, and the routes
join the nonce postimage to the publication transaction. On every replay path, however, the nonce
write is dropped:

- create: `if nonce_plan is not None and not recovering` (nonce pre/post images stay `None`);
- review: the nonce write is appended only inside `if not recovering` (`codefire:12929-12946`);
- apply: the recovery branch (`codefire:13081-13162`) contains no nonce write at all.

Live probe, signed policy with `required: true`:

```text
first signed create  (idem-1, nonce r1) -> 200; nonce entries ['r1']
replay create        (idem-1, nonce r2) -> 200, byte-identical body; nonce entries ['r1']
same signature again (idem-1, nonce r2) -> 200
write_json targets during replay: []   storage bytes identical: True
```

Failure scenario: an attacker who captures one signed retry whose idempotency key already has a
receipt can re-submit that exact request without bound for the whole skew window (default 300 s),
each time receiving the recorded result, because the nonce it presented is validated and then thrown
away. Impact is bounded — the replay performs no `write_json` at all and leaves durable bytes
byte-identical, so this is a replay-*protection* contract gap and a recorded-result disclosure
primitive, not a state-divergence hole. That is exactly the "incomplete recovery/replay design"
branch of required-review item 3.

### F5 — Low. A same-path grow-tamper at the fence is reported as a bounded-limit error, not a fence change

`AccountedHTTPMergeRequestFence._observe` (`codefire:11732-11777`) takes `declared` from `lstat`,
then calls `_read_pinned_authority_bytes(path, max(declared, 1), self._poll)`
(`codefire:11623-11686`), which raises
`CFError("pinned merge request authority exceeds its bounded limit: ...")` when the reopened file's
`st_size` exceeds that limit. Failure scenario: an authority file grows between the `lstat` and the
`open` inside `revalidate()`; the fence correctly refuses to proceed and no effect occurs, but the
client receives a bounded-limit `400` rather than the route's own
`merge_request_fence_changed` typed diagnostic with `expected_digest` / `actual_digest`. Detection is
intact; only the typing and the diagnostic are wrong. The advertised same-path/same-size/same-mtime
tamper case is correctly typed and is covered by
`test_final_fence_rejects_a_same_size_same_mtime_subject_tamper`.

### F6 — Informational. Nonce timing is now inconsistent across `POST` routes

`handle_http_gc` (`codefire:11502-11514`) still calls `authorize_remote`, which burns the nonce at
authorization through `record_request_nonce` (`codefire:4078`), and it still receives no ledger,
deadline, or response slot. After this lane lands, `POST .../gc` consumes a nonce on a failed request
while upload and the three MR routes do not. This is the already-ledgered residual gc packet, not a
defect of this diff, but it must not be left unresolved across a release boundary.

## Verdicts for all six required-review items

| # | Item | Verdict |
|---|---|---|
| 1 | Exact handler ledger/deadline through every phase; no lane-local `http_request` ledger or hidden legacy traversal escaping accounting | **FAIL (blocking)** |
| 2 | Everything charged before allocation/read/growth; exact/±1 and 1x/3x/10x deterministic and non-vacuous | **PARTIAL — mechanics accept, coverage fail (blocking)** |
| 3 | Nonce semantic change: required atomicity, compatibility regression, or incomplete recovery/replay design | **Required atomicity, with an incomplete recovery/replay design (medium)** |
| 4 | Legacy subject-less review materialization ordering | **PARTIAL — item 4 as written is satisfied; the lane contract's no-effect and status/error-precedence clauses fail (medium)** |
| 5 | Remaining nested `validate_sealed_commit` local ledger and off-goal corruption precedence | **FAIL (blocking) — reproduced** |
| 6 | Independent focused, legacy MR / request-signature / replay, compilation and diff checks | **ACCEPT for every gate run; the two broad-suite Cargo failures are identified but not re-verified against the toolchain in this wave** |

Item-by-item detail:

**1 — FAIL.** The positive half holds and is proven: exactly one `http_request` ledger per route on
every live route (probe, and `test_every_live_route_binds_the_exact_request_ledger_and_deadline`),
`lane.ledger is request_ledger`, every fence/ancestry/borrow/signature owner bound to that same
ledger and that same deadline object, `_HTTPMutationResponseSlot` handed off through
`mutation_slot.take()` in `do_POST` (`codefire:15912-15958`) with the slot closed in the handler's
`finally` (`codefire:16018-16020`), authority capture and escrow/signature/ancestry validation
charged, a final fence, publication, replay, and clean cleanup. The negative half fails: the hidden
legacy traversal does escape accounting — see F1's unaccounted `collect_object_records` and
`copy_object_graph` call sites and the uncharged authorization phase.

**2 — PARTIAL, blocking on coverage.** For the dimensions the lane does own, charging is genuinely
pre-allocation: `AccountedHTTPMergeRequestFence._observe` acquires scratch and applies
`snapshot_revalidation_*` and `aggregate_bytes` *before* `_read_pinned_authority_bytes`
(`codefire:11732-11777`); `AccountedHTTPMergeRequestAncestry._push` / `_mark_seen` charge
`graph_edges` / `graph_nodes`, observe `maximum_frontier` and `logical_depth`, and acquire the frame
and seen-entry retained bytes before mutating the frontier or the seen set (`codefire:11996-12037`);
`AccountedRemoteCommitBorrow.load_sized` borrows exactly one record at a time and releases the prior
one first (`codefire:11878-11914`); `lane.collect_records` charges each record read and each edge and
releases its sort scratch (`codefire:12244-12305`). Fence pin counts are deterministic and exactly
match: 6 pins x 2 observations = 12 `snapshot_revalidation_records` for create and apply, 5 x 2 = 10
for review. The counter curves are non-vacuous. The blocking problem is that these exact bounds
apply to a minority of the route's real reads (F1, F2), so "charged before read" is true of what is
charged and false of the request as a whole.

**3 — Required atomicity, incomplete on recovery.** Deferral is not a regression to be reverted: the
lane contract requires that a typed failure leave nonce state byte-identical to pre-call state, and
recording a nonce is itself a durable write, so joining nonce publication to the transaction is the
only way to satisfy that clause. Proven by live retries:

```text
signed review to a nonexistent MR -> 404 "merge request not found"   (authorization already passed)
nonce cache entries after that failure: []                            (nonce not burned)
later reuse of that same nonce      -> 200                            (nonce is reusable)
replay of a *successful* nonce      -> 400 "request signature validation failed: nonce has already been used"
```

So the invariant changed from "a nonce is accepted at most once ever" to "a nonce is consumed at most
once by a *publishing* mutation". The residual replay window this opens for non-publishing failures
is inherent to the required atomicity and is bounded by `max_skew_seconds`; I accept it. What is not
acceptable is F4: the recovery path accepts a valid nonce and never records it even though that
request *does* return a durable result.

**4 — PARTIAL.** Response sealing does precede `materialize_merge_request_subject`, so encoding,
header and retained-limit failures are impossible after the first effect, and the failure does
recover to one durable byte-identical result across retries. Preflight and publication failures
remain possible after the first effect and are masked as retryable; escrow object bytes are not
restored. See F3.

**5 — FAIL, reproduced.** The nested `local_v1` ledger is present on the live path of all three
routes and carries the larger share of the work (F1's tables). On off-goal precedence: apply runs
`validate_http_merge_request_record` at `codefire:13051`, which sealed-validates up to three commit
graphs, *before* the admissibility checks at `codefire:13163` (`status != "approved"` -> 409) and
`codefire:13171` (stale -> 409). Ordering is unchanged from `e2a218e`, so this is not a precedence
regression, but it means an inadmissible request still pays for full off-goal graph validation on a
hidden ledger with no deadline — the availability half of F1.

**6 — ACCEPT for what was run.** Results in the next section. `git diff --check` over
`cab3126..21fe751` is clean. The 18 focused tests pass at the target and fail hard against
`e2a218e` (14 failures + 23 errors), so the module is production-path non-vacuous. The two
broad-suite failures are identified precisely as `test_install_script_installs_codefire` and
`test_completion_command_and_install_completion`, both failing with exactly
`error: cargo is required to build the Rust codefire binary; pass --binary PATH to use a prebuilt
binary`, and neither exercises `codefire` or any MR route. Re-verification with the configured Cargo
toolchain was started and then killed by the host OOM restart; it produced no output and no binary,
and this wave is memory constrained, so it is **not claimed as verified here**. The
Cargo-PATH-environment conclusion rests on the reproduced failure text plus the toolchain probe
already recorded in `docs/development/ai/swarm-batches/cfr168-recovery-execution-ledger-20260730.md`.

## Commands and results

All product-side commands ran against a read-only `git archive` extraction of `21fe751` in a scratch
directory. No product file, test, issue record, or management ledger was modified.

| Command | Result |
|---|---|
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests.test_python_http_mr_mutation_decision_work -v` | **OK — 18 tests, 98.030s** |
| Same module against `e2a218e015953f082245f36dbc6aee906b837e5a` | **FAILED — 14 failures, 23 errors, 18 tests, 61.883s (reproduces the production defect)** |
| `python3 -m unittest -v` over 18 neighboring legacy MR / request-signature / nonce / replay / escrow / branch-protection cases in `tests.test_codefire_cli.CodeFireCliTests` | **OK — 18 tests, 527.051s** |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire tests/test_python_http_mr_mutation_decision_work.py` | **OK** |
| `git diff --check cab3126f216c595c0d2211174b364e8532a310e2 21fe75175e997cc7c6f8c3ef2a7558144d850f94` | **clean — no whitespace errors** |
| Reviewer probe: hidden ledgers, uncharged reads, uncharged traversals per route | **OK — F1 table** |
| Reviewer probe: apply hidden-work scaling at 1x/3x/10x | **OK — F1 scaling table** |
| Reviewer probe: failed signed request does not burn its nonce; successful nonce still refused | **OK — item 3 evidence** |
| Reviewer probe: recovery replay never burns its fresh nonce; unbounded reuse; zero `write_json` | **OK — F4 evidence** |
| Reviewer probe: legacy review post-effect preflight failure | **OK — F3 evidence** |
| `tests.test_codefire_cli.CodeFireCliTests.test_install_script_installs_codefire` (no Cargo on `PATH`) | **FAILED — `cargo is required to build the Rust codefire binary`** |
| `tests.test_codefire_cli.CodeFireCliTests.test_completion_command_and_install_completion` (no Cargo on `PATH`) | **FAILED — same message** |
| `test_install_script_dry_run_bad_binary_and_prefix_safety`, `test_install_script_completion_failure_does_not_install_binary`, `test_pyproject_installs_codefire_script` | **OK — install surface otherwise healthy without Cargo** |
| Same two Cargo cases with `RUSTUP_HOME`/`CARGO_HOME`/configured `PATH` | **NOT COMPLETED — killed by the host OOM restart, no output, no binary; not re-run in this memory-constrained wave** |

The neighboring set run in full: `test_http_merge_request_escrow_survives_source_project_removal`,
`test_http_remote_payload_tamper_writes_no_layout_or_nonce`,
`test_http_remote_request_signature_rejects_replayed_nonce`,
`test_http_remote_request_signature_required_on_upload`,
`test_merge_request_subject_escrow_survives_source_removal_and_rejects_tamper`,
`test_merge_request_subject_matches_cross_runtime_golden_vector`,
`test_python_remote_nonce_capacity_preserves_live_replay_memory`,
`test_python_remote_publication_recovers_authoritative_staging_and_replays_receipt`,
`test_python_remote_request_signature_enforces_default_skew_in_utc`,
`test_python_remote_request_signature_rejects_payload_field_tamper_and_key_windows`,
`test_python_upload_signed_invalid_identity_does_not_consume_nonce`,
`test_python_upload_nonce_race_stays_pre_effect_and_exactly_retryable`,
`test_remote_branch_protection_gates_upload_and_apply`,
`test_remote_request_signature_matches_shared_golden_vector`,
`test_remote_request_signatures_gate_mutating_operations`,
`test_remote_upload_receipt_replays_across_file_and_http_transports`,
`test_request_list_and_review_reject_invalid_merge_request_refs`,
`test_upload_list_remote_clone_and_request_merge`.

Not run and not claimed: the full Python suite, `check_source_docs.py` / `check_project_docs.py`
against the target tree (they require a git checkout and the diff touches no documentation), and any
Cargo build.

## Required integration predecessors

1. **Python upload response-owner invariants must be integrated first.** This target depends on
   symbols introduced by the upload line — `_HTTPMutationResponseSlot` (`codefire:14724`),
   `_PreparedHTTPMutationResponses`, `HTTP_MUTATION_RETRY_PAYLOAD`,
   `authorize_remote_without_nonce` (`codefire:6811`), `verify_remote_request_signature(...,
   defer_nonce=True)`, and the `preflight=` parameter of `execute_remote_json_publication`
   (`codefire:3295`). Integrating MR mutation ahead of upload is not possible. This matches
   serialized integration step 3 in
   `docs/development/ai/swarm-batches/cfr168-recovery-execution-ledger-20260730.md`.
2. **Resolve the `do_POST` and `make_http_handler` hunks against the Doctor, branch-session and
   upload changes** so exactly one request ledger and one deadline survive. This diff adds
   `self._mutation_phase` to the per-request initialisation in `handle_one_request`
   (`codefire:15552`) and reads it in the response-encoding and mutation-publication deadline labels
   (`codefire:15098`, `codefire:15251`), and it adds four adjacent route arms with
   `mutation_slot.take()` (`codefire:15899-15958`). These are the same hunks the Doctor/branch review
   flagged for adjacent-conflict resolution.
3. **F1 and F2 must be fixed before integration**, using the in-repo accounted pattern
   (`codefire:11335-11346` and `codefire:13398-13408`), and the ledger-identity test tightened from
   "no second `http_request` ledger" to "no nested ledger" so the fix is defended.
4. **`POST .../gc` must follow in the same release**, so nonce-consumption timing is uniform across
   `POST` routes (F6). It is already predecessor-gated on this result in the recovery ledger.

Not a predecessor, but required by the same contract wave: F3's ordering fix touches
`materialize_merge_request_subject` call ordering only inside the review lane, so it does not
enlarge the write set.

## Residual risk

- **Unbounded uncharged memory in the authorization phase.** `read_json` (`codefire:1429`) has no
  size limit, and the nonce cache is materialised up to twice per successful signed mutation. A
  large or adversarially grown `request_nonce_cache.json` or `server_policy.json` is a memory
  amplification path that no `http_v1` dimension observes. This predates the diff but the diff
  doubles the nonce-cache materialisation count on the success path.
- **Counter authority.** Until F1 is fixed, the lane's counters must not be promoted into the
  asymptotic/order table or the cross-runtime corpus: they under-report record reads by 73%-93% at
  `chain_length=3` and the shortfall grows with input.
- **Deadline granularity.** Even after F1, the deadline is polled at phase boundaries; a single
  helper call over a very large graph is only interruptible if `poll=` is threaded all the way down,
  as the `do_GET` bundle path does.
- **Legacy escrow residue.** F3 leaves content-addressed escrow objects behind on a post-effect
  failure. They are harmless and GC-reachable, but the "no effect for every typed failure" claim is
  not literally true for the legacy review path and should not be asserted in downstream conformance
  documents until fixed.
- **Cargo verification not re-run.** The environment-only conclusion for
  `test_install_script_installs_codefire` and `test_completion_command_and_install_completion` rests
  on the reproduced `cargo is required` failure text plus the previously recorded toolchain probe,
  not on a fresh build in this wave.
- **Unreviewed adjacent surfaces.** `handle_http_gc` and the `do_GET` routes outside this packet were
  read only far enough to establish F6 and the integration conflicts; they are not reviewed here.
- **Interpreter-level exceptions.** `recover_from_effect_failure` (`codefire:12355`) converts any
  `BaseException` after a successful publication into `200`. This matches the established upload
  contract (`codefire:13662-13675`) exactly, so it is recorded as accepted behaviour rather than a
  finding, but it does mean `SystemExit` and `MemoryError` are suppressed once the mutation is
  durable.
