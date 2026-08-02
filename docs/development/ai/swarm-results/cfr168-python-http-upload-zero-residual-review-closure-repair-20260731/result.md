# CFR-168 / CFB-138 Python HTTP Upload Review-Closure Repair — Implementer Result

Implementer: Claude Opus 5 (`claude-opus-5`), original implementer session
`7baa1275-0ee2-476a-af46-1907386ebceb`, resumed in its existing isolated worktree.

Packet: `docs/development/ai/swarm-batches/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731.md`

Immutable review:
`docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-rereview-fugu-20260730/review.md`

**This is implementer evidence only. It is not an acceptance claim.** A fresh independent
exact-candidate rereview is requested.

## Immutable input verification

| Item | Expected | Observed | Result |
|---|---|---|---|
| Review Git blob | `1fd3de9b48e04d29b56f36d79584a0a03c22bcff` | identical | PASS |
| Review SHA-256 | `7632552eb0ca60addca0d83fe3ba53ead642a7cbee82937b1b7b37754f6cb2cf` | identical | PASS |
| Rejected candidate tree | `fcc65a4729f5029556e0266971d94b6321421f09` | identical | PASS |
| Rejected candidate | `76baff84cb1192e11b26dcc669fb800f7b13fa30` | ancestor of head, unamended | PASS |

Nothing in the rejected candidate, the review, the packet, or history was amended, rebased,
squashed, reset, or merged. Every commit below is additive above the packet commit.

## Candidate identity

| Item | Value |
|---|---|
| Packet/parent base | `b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b` |
| Repair commits | see table below |
| Changed paths | `codefire`, `tests/test_python_http_upload_accounted_traversal.py`, four canonical source docs, this result record |

Cumulative `b19c434..HEAD` diff: `codefire` +156/−29,
`tests/test_python_http_upload_accounted_traversal.py` +806/−2,
`docs/development/source-architecture.md` +30/−5,
`docs/development/change-playbook.md` +16/−0,
`docs/development/command-source-trace.md` +6/−2,
`docs/development/source-data-model-catalog.md` +2/−2.

## Finding closure

### BLOCKER-1 — completed-quarantine recovery keeps the request ledger and deadline

`handle_http_upload` passes its exact `DecisionWorkLedger` and `HTTPRequestDeadline` to
`execute_remote_json_publication`, which forwards both to `recover_remote_pending_quarantine`.
Recovered durable roots are validated through
`validate_sealed_commit(..., ledger=..., deadline=..., phase="upload recovery sealed validation")`.
The deadline is polled before each recovered root, before the receipt replay
(`upload recovery receipt`), and immediately before the quarantine cleanup
(`upload recovery cleanup`); no fallible or input-proportional work runs between that last poll and
`remove_remote_transaction_dir_quarantine` / the quarantine unlink / the recovered response. A
caller with no request budget — remote GC at the second call site — keeps the legacy positional
call verbatim.

Exact call chains:

```text
ordinary : handle_http_upload -> prepare_http_upload_sealed_validation -> validate
           -> publication_root_proof -> validate_remote_json_publication_identity
           -> finish (final revalidation) -> signature -> server verification
           -> fence("upload final subject fence") -> [fence("upload layout bootstrap")
           -> ensure_remote_layout] -> gate -> ancestry -> preflight_remote_json_publication
           -> seal_for_effect -> fence("upload publication effect")
           -> write_object_records -> execute_remote_json_publication

recovery : handle_http_upload -> ... -> execute_remote_json_publication(ledger, deadline)
           -> recover_remote_pending_quarantine(ledger, deadline)
           -> poll("upload recovery sealed validation")
           -> validate_sealed_commit(ledger, deadline, phase=...)
           -> poll("upload recovery receipt") -> receipt replay
           -> poll("upload recovery cleanup") -> quarantine cleanup -> recovered result
```

Reversal, 1.0 s advanced against a 100 ms request at the recovered-root validation:

| Observation | Reviewed candidate `76baff84` | This repair |
|---|---|---|
| Outcome | `(200, {...})` | typed `RequestDeadlineExceeded` |
| `expired_phase` | `None` | `upload recovery sealed validation` |
| Ledgers minted in the request | 1 (`sealed validation` / `local_graph`) | 0 |
| Completed quarantine | **deleted** | present, byte-identical |
| Objects / branches / generation / nonce / pending / receipts / audit | unchanged | unchanged |
| Ledger current / permits | zero / none | zero / none |

The live retry still completes: `(200, {"branch": "main", "head": ...})`, quarantine cleared, zero
ledgers minted, two root traversals on the request ledger (staged head, recovered durable root),
zero debt; the idempotent replay returns the identical status and body.

### HIGH-2 — the caller-owned sealed-validation API completes

`_AccountedUploadSourceMemory` keeps `retained_bytes` as a fixed upper reserved once before the
session constructor, and treats `temporary_bytes` as an exactly acquired dimension. `finalize`'s
ordered object-id projection is charged on the caller's ledger, on the claim's own
`snapshot_revalidation` phase, before the list exists. The caller ledger raises its own typed budget
diagnostic; the owner no longer shadows it.

Formulas:

```text
retained control upper  = SEALED_SESSION_CONTROL_UPPER = 11,456   (reserved once, count-independent)
temporary control claim = ordered_projection_upper(N)
                        = sizeof([]) + pointer_bytes * N = 56 + 8N   (acquired exactly)
                          N=0 -> 56      N=1 -> 64      N=27 -> 272
```

Those three values reproduce the review's runtime observations exactly. Measured on a 407-document
closure: projection `3,312`, request temporary peak `5,300`.

| Public path | Reviewed candidate | This repair (ledger, and ledger + deadline) |
|---|---|---|
| `validate_sealed_commit(..., ledger=..., deadline=...)` | `CFError: exceeded its prepared temporary_bytes reservation` | completes |
| `validate_remote_json_publication_identity(..., ledger=..., deadline=...)` | same `CFError` | completes |
| `preflight_remote_json_publication(..., ledger=..., deadline=...)` | same `CFError` | completes |
| no-ledger identity | succeeds | succeeds, precedence unchanged |

Every path ends with `current={retained_bytes: 0, temporary_bytes: 0}`, no open permits, and no
terminal reason. Empty (memoised), one-record and multi-record closures are covered.

Boundary matrix on the 407-document closure:

| Temporary limit | Outcome |
|---:|---|
| `3,311` (projection − 1) | refused, `snapshot_revalidation` / `temporary_bytes`, observed `3,312` |
| `3,312` (projection) | admitted, refused later at observed `3,854` |
| `5,299` (peak − 1) | refused, observed `5,300` |
| `5,300` (peak) | completes, zero debt |

### HIGH-3 — the publication-root proof is atomically single-use

`RemoteJSONPublicationRootProof` gains `available / consuming / consumed / revoked`. The
`CONSUMING` claim is taken **before** the deadline poll — the only callback the object can reach —
and `CONSUMED` before the roots are returned.

| Case | State after | Result |
|---|---|---|
| first use | `consumed`, consumptions 1 | accepted |
| second use / aliased use | `consumed`, consumptions 1 | `remote publication root proof is already consumed` |
| callback reentry through the injected clock | `consumed`, consumptions **1** | inner call gets `... is already consuming` |
| foreign object store | `available`, consumptions 0 | typed store error, retryable |
| uncovered root | `available`, consumptions 0 | typed coverage error, retryable |
| construction then deadline expiry (×2) | `available`, consumptions 0 | `RequestDeadlineExceeded` |
| revoke before use / after use / twice | `revoked` | `... is revoked`, idempotent |
| live upload | `revoked`, consumptions 1 | one traversal, unchanged wire bytes |

The reviewed candidate accepted two consumptions before revocation and two under callback reentry.

### MEDIUM-4 — reversal evidence is non-vacuous

Focused suite: 22 → **29** tests. Each mutation was applied one at a time to a clean isolated export
of the head tree at `/tmp/mut`, run, reversed, and the tree proved byte-identical to baseline
`codefire` SHA-256 `8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93` before the
next one.

| # | Mutation | Focused result | Failing test | unittest time |
|---|---|---|---|---:|
| m1 | remove the ordered-projection temporary claim, keep list + sort | FAILED (1) | `test_ordered_projection_claim_is_exact_and_precedes_its_allocation` | 115.824s |
| m2 | replace `_assert_prepared_coverage` with a no-op | FAILED (4) | `test_prepared_coverage_predicates_are_load_bearing` | 101.438s |
| m3 | remove only the debt predicate | FAILED (4) | same | 108.399s |
| m4 | remove only the peak-vs-upper predicate | FAILED (3) | same | 106.974s |
| m5 | remove only the pool-was-used predicate | FAILED (2) | same | 116.366s |
| m6 | remove the `upload layout bootstrap` poll, keep the publication poll | FAILED (1) | `test_removing_the_layout_bootstrap_poll_creates_the_layout` | 90.129s |
| m7 | drop the recovery ledger/deadline handoff | FAILED (1) | `test_completed_quarantine_recovery_owns_the_request_budget` | 104.066s |
| m8 | remove the proof's reuse guard | FAILED (2 f / 1 e) | `test_publication_root_proof_is_atomically_single_use` | 112.571s |
| m9 | remove the source owner's temporary dimension | FAILED (1) | `test_ordered_projection_claim_is_exact_and_precedes_its_allocation` | 102.355s |

Every mutation was removed afterwards and restoration verified by SHA-256. Timings were measured
while an unrelated `cfr168-python-http-mr-list-review-closure-repair` lane ran a full
`tests.test_codefire_cli` suite on the same host, so they are inflated relative to an idle machine.

Oracle shapes, as the packet requires:

- the projection oracle asserts acquisition before the sort, the exact dimension, phase and size,
  the owner's peak use, its release, and terminal zero, and binds projection exact/minus-one;
- the coverage oracle creates the exact state each predicate rejects immediately before the
  production check and then delegates to it, so a no-op replacement stops the raise — it is not a
  forced raise;
- the layout-poll mutation produces a behavioural difference: the durable layout of a project that
  had none is created by an already-elapsed request.

## Preserved accepted properties

| Property | Evidence |
|---|---|
| 1,201-commit divergent history reaches the exact 409 | 10x row below; 82-byte body |
| 1x/3x/10x curves | table below; reservation equals the ledger retained peak at every factor |
| one request-owned sealed snapshot, no foreign ledger, ordinary route | `test_one_upload_owns_every_sealed_validation_ledger` |
| streamed canonical encoder byte-identical and bounded | `test_streamed_canonical_encoder_matches_the_whole_record_encoder` |
| existing-project and missing-project final fences pre-effect | `test_expired_request_cannot_publish_or_bootstrap_a_layout` |
| descriptor registry/reuse bounds | `test_source_session_allocations_prove_an_outstanding_reservation` |
| detached-document fingerprint release | broad owner suites |
| exact 200/409 response bytes, nonce/idempotency | 13 selected legacy cases; recovery replay above |
| record size/count ceilings | `test_default_profile_record_shape_and_size_curves` |

1x/3x/10x, unmodified `DecisionWorkProfile.http_v1()`:

| Factor | Depth | Records | Aggregate / max | Status | Retained reservation = ledger peak | Temporary reservation | Claimed retained peak | Nodes / edges / reads | Polls |
|---|---:|---:|---:|---|---:|---:|---:|---|---:|
| 1x | 120 | 127 | 98,437 / 803 | 409 | 948,962 | 615,540 | 138,307 | 242 / 966 / 248 | 156,930 |
| 3x | 360 | 367 | 291,157 / 803 | 409 | 2,567,042 | 650,100 | 400,387 | 722 / 2,886 / 728 | 465,758 |
| 10x | 1,200 | 1,207 | 967,078 / 805 | 409 | 8,233,124 | 771,068 | 1,319,068 | 2,402 / 9,606 / 2,408 | 1,548,053 |

Every row: zero current retained/temporary debt, zero open permits, branch unchanged, tip absent.

## `validate_sealed_commit` production inventory

AST inventory of the head tree: **29** call occurrences (was 28; `recover_remote_pending_quarantine`
now has two — the accounted call and the preserved no-budget call).

| Enclosing function | n | Lines | Keyword arguments |
|---|---:|---|---|
| `open_context` | 2 | 1808, 1816 | — |
| `recover_remote_pending_quarantine` | 2 | 3021, 3025 | `(deadline, ledger, phase)` and `()` |
| `validate_remote_json_publication_identity` | 2 | 3577, 3579 | `(deadline, ledger, phase)` and `()` |
| `handle_http_list` | 1 | 12250 | — |
| `http_remote_branch` | 1 | 12343 | — |
| `validate_http_merge_request_record` | 1 | 12371 | — |
| `handle_http_upload` (no-budget fallback) | 1 | 12895 | — |
| `validate_merge_request_record` | 1 | 21460 | — |
| `resolve_commitish` | 2 | 33046, 33049 | — |
| `resolve_commitish_any` | 3 | 33067, 33078, 33084 | — |
| `cmd_branch_list` | 1 | 33236 | — |
| `cmd_open` | 1 | 33245 | — |
| `cmd_clone` | 3 | 33323, 33343, 33362 | — |
| `cmd_upload` | 1 | 33382 | — |
| `cmd_request_merge` | 2 | 33598, 33599 | — |
| `cmd_request_list` | 1 | 33721 | — |
| `cmd_request_apply` | 2 | 33984, 33985 | — |
| `cmd_merge` | 2 | 34329, 34330 | — |

Upload-reachable recovery is now accounted. The three remaining HTTP owner functions
(`handle_http_list`, `http_remote_branch`, `validate_http_merge_request_record`) carry no request
ledger or deadline in their own handlers; the reusable caller-owned API they need now completes on
all three of its public entry points.

## Gates

All with `PYTHONDONTWRITEBYTECODE=1`, in the implementation worktree.

| Command | Result | unittest time |
|---|---|---:|
| `python3 -m py_compile codefire tests/test_python_http_upload_accounted_traversal.py tests/test_python_sealed_production.py tests/test_codefire_cli.py` | PASS | — |
| `python3 -m unittest tests.test_python_http_upload_accounted_traversal` | 29/29 PASS | 156.953s |
| `python3 -m unittest tests.test_python_ancestry_owner tests.test_python_signature_owner tests.test_python_signature_terminal_freeze tests.test_python_sealed_production tests.test_python_external_production` | 59/59 PASS | 29.597s |
| `python3 -m unittest discover -s tests -p 'test_python_*.py'` | 450/450 PASS | 333.387s |
| 13 selected legacy CLI cases from the review | 13/13 PASS | 169.748s |
| `python3 docs/development/tools/check_source_docs.py` | PASS — 44 modules, 34 commands, 12 object kinds | — |
| `python3 docs/development/tools/check_project_docs.py` | PASS — 531 Markdown files | — |
| `git diff --check` | PASS | — |

The 13 selected legacy cases are the exact list named by the immutable review, including
`test_python_completed_pending_quarantine_preserves_missing_receipt` and
`test_python_remote_publication_recovers_authoritative_staging_and_replays_receipt`, which cover the
quarantine/recovery surface this repair changes.

## Unavailable / not-run evidence

- Cargo, Docker, install, dogfood, `./demo.sh`, unrestricted all-language discovery and
  repository-wide validation were not run in this lane, per the packet.
- Independent raw-wire 16 MiB request-body and 64 KiB header exact/±1 fixtures were not re-authored
  here; those rows continue to rest on the existing owner suites.
- Timings were taken under concurrent load from another lane, as noted above.

## Conflicts with active Python HTTP composition work

An active `cfr168-python-http-mr-list-review-closure-repair` lane is running against the same
`codefire`. This repair changes the signatures of `execute_remote_json_publication` (added optional
`ledger`/`deadline`) and `recover_remote_pending_quarantine` (same), both of which are reachable
from `handle_http_request_merge` / `_review` / `_apply`. It does not modify
`validate_http_merge_request_record`, `handle_http_list`, or `http_remote_branch`. The manager
should expect a textual conflict in those two function headers and in the neighbouring
`validate_sealed_commit` call-site inventory when composing the two lanes.

## Request

Fresh independent exact-candidate rereview. No acceptance, integration, issue closure, capability
completion, or zero-residual claim is made from this implementer evidence.
