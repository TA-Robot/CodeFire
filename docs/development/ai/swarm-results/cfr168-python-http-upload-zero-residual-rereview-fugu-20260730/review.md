# CFR-168 / CFB-138 Python HTTP Upload Zero-Residual Independent Re-review

Review completed: 2026-07-31 UTC

Reviewer lane: `cfr168-python-http-upload-zero-residual-rereview-fugu-20260730` /
task `upload-zero-residual-rereview`

Reviewer/model: independent Fugu worker agent using Sakana Fugu 1.1 Ultra
(`fugu-ultra-v1.1`). This review was performed independently from the repair implementer and from
the prior Claude fallback reviewer. The prior report was read as a mandatory finding checklist, but
the decision below is based on exact-candidate source inspection, fresh candidate exports,
reviewer-authored runtime probes, independent mutation runs, and re-executed gates.

Review scope: report-only. Product code, tests, canonical source docs, issue state, management
ledgers, and repair history were not modified.

## Decision

**`REQUEST_CHANGES`**

The candidate closes the central live-path scale defect and most of the original ordinary-upload
failure: the default profile admits records above 3 MiB, the 1,201-commit divergent history reaches
the exact 409 non-fast-forward decision, one normal HTTP upload uses one request-owned sealed
snapshot, and the final normal-path deadline fences prevent both project bootstrap and publication
after expiry.

Acceptance is nevertheless invalid for four independent reasons:

1. **F1 remains open at Blocker severity on completed-quarantine upload recovery.** The exact upload
   retry reaches `recover_remote_pending_quarantine`, which calls `validate_sealed_commit` without
   the request ledger or deadline, mints a fresh `local_graph` ledger, and returns 200 after the
   request clock is advanced to 1.0 seconds against a 100 ms deadline.
2. **The newly advertised caller-owned reusable API is unusable.** All of
   `validate_sealed_commit(..., ledger=..., deadline=...)`,
   `validate_remote_json_publication_identity(..., ledger=...)`, and
   `preflight_remote_json_publication(..., ledger=...)` fail because their source-memory owner
   reserves no `temporary_bytes` for `finalize`'s ordered projection.
3. **`RemoteJSONPublicationRootProof` is not single-use.** The same live proof accepts two
   consumptions before revocation, contradicting its contract and the mandatory F1 state-machine
   requirement.
4. **Required reversal evidence is incomplete.** Removing the ordered-projection claim or replacing
   production `_assert_prepared_coverage` with a no-op survives both the focused 22-test mutation
   run and a broader 33-test owner run. The layout-bootstrap deadline poll also lacks the required
   removal mutation. A forced-raise mutation proves the coverage function executes, but not that
   its checks are non-vacuously protected.

The strongest correctness concern is the upload retry's request-budget/deadline escape. The
strongest maintainability concern is that two accounting invariants explicitly required by this
ticket can be deleted without a candidate test changing result, while the public reusable path is
already broken by one of those invariants' dimension mismatch.

## Immutable subject and provenance

| Role | Exact revision / digest | Result |
|---|---|---|
| Reviewer worktree base | `de63a148c199ade7a05d3d7b68b60ef6af0d576a` | Confirmed |
| Canonical implementation snapshot | `e2a218e015953f082245f36dbc6aee906b837e5a` | Confirmed from packet |
| Reviewed scale target | `dece688afea815d1242e7751367052d1ceac4580` | Used for portable reversal |
| Prior report in this ancestry | `173cdf83583f8a677fe63a58c9787e8f8e884e9d` | Confirmed |
| Byte-identical original report commit | `0b1fd6f50fce2126a07238c422b1ff01b575c047` | Confirmed |
| Prior report Git blob at both commits | `32e0ead1727d6a0f745616f08149579990c60845` | Confirmed identical |
| Prior report SHA-256 | `534f098348e72fdc377b56ebf561a931e53284055a1f8a0e37d6d5393d956486` | Recomputed |
| Repair ticket / implementation base | `5f798990c313ea2e1b117bc74824de3f3ce028d2` | Confirmed |
| Repair ticket Git tree | `8f6cadbf9457081de27e485dbf00dd78f27293ff` | Confirmed |
| Repair ticket SHA-256 | `13a785e553545f111e4a0ab51583ff1ff23f6cd14ff60aa09b1f4ce06195db0d` | Recomputed |
| Exact repair candidate | `76baff84cb1192e11b26dcc669fb800f7b13fa30` | Reviewed |
| Candidate parent | `f6798cbfffda6ae181e934210a465ef7a7373539` | Confirmed |
| Candidate Git tree | `fcc65a4729f5029556e0266971d94b6321421f09` | Confirmed |
| Candidate patch SHA-256 | `0e0a0c35ae8c7e9852775199ade4172b737e6797bcbcc2425b2baba29aadd028` | Recomputed with required command |
| Merge base | `5f798990c313ea2e1b117bc74824de3f3ce028d2` | Exact |
| Commit count / merge count | `11` / `0` | Linear range confirmed |

The exact cumulative range contains these commits, in order:

| Revision | Exact subject | Review disposition |
|---|---|---|
| `600750cc070f076052b46fa39a32ed3eb19a8068` | `wip(http-upload): stream the canonical borrow encoder with bounded scratch` | Core encoding claim reproduced; PASS |
| `c33592a40d2099bdb3eea5b82a728ce643110b2d` | `wip(http-upload): reserve every sealed session's control state before allocation` | Live upload PASS; reusable caller-owned path FAILS temporary projection |
| `0b288985aa37b23567e8e28afb6505cc54b0ce2a` | `wip(http-upload): one request-owned sealed traversal and a final effect fence` | Ordinary upload PASS; completed-quarantine recovery escape remains |
| `6abc9916fd3b1f091d0c13e02ea0a4cc21f82206` | `wip(http-upload): consume the sealed proof inside its own reservation` | Reservation PASS; proof single-use contract FAILS |
| `c5d7e5e88da1dd734455913443b37c3eb68052ca` | `wip(http-upload): make the prepared-upper check a production invariant` | Function executes; no-op mutation remains undetected |
| `8830358bccbc5837b4874735c0a9805cf26b5c47` | `test(http-upload): bind the zero-residual repair with reversal and mutation evidence` | Focused tests pass; mandatory reversal matrix remains incomplete |
| `7d5d219da9ee28f37b151b3265cf55d4475d523f` | `test(http-upload): make the session probe portable to the reviewed target` | Candidate probe portable; full old-target repair suite is not non-vacuous |
| `49e044a53baabf731b70fd65ce83d0ab8b78445c` | `docs: record the HTTP upload zero-residual repair in the canonical source docs` | Docs checks pass; current-state claims at `source-architecture.md:391` are false |
| `668c44fa5871903491043a659ff253943678e908` | `fix(http-upload): release the borrow fingerprint when its document detaches` | Broad mutation detects omission; PASS |
| `f6798cbfffda6ae181e934210a465ef7a7373539` | `refactor(http-upload): drop the dead publication-root proof plumbing` | Removed plumbing has no reproduced residual; PASS |
| `76baff84cb1192e11b26dcc669fb800f7b13fa30` | `test(http-upload): pin the record-count ceiling as well as the record-size one` | Formula boundary reproduced; PASS |

The cumulative diff is exactly six files and `+2,227/-120`:

| Path | Added | Deleted | Candidate Git blob |
|---|---:|---:|---|
| `codefire` | 1,151 | 97 | `cc2c52aff4a11ebd80b2f22e37482b1700c0d738` |
| `tests/test_python_http_upload_accounted_traversal.py` | 1,006 | 20 | `ceeeebe8be2b581b2ba216dc349ced12624ad4c3` |
| `docs/development/change-playbook.md` | 17 | 0 | `fcb1514713644ae346e367358220852c64280725` |
| `docs/development/command-source-trace.md` | 13 | 2 | `847d19441cac15bc89217a002d9cab67891555ec` |
| `docs/development/source-architecture.md` | 37 | 0 | `6a4f8d4afb5312d18029ef0754fcedc16b6bab77` |
| `docs/development/source-data-model-catalog.md` | 3 | 1 | `ea357917bfa18e13e3a58fb3c4ce0092cb8a0241` |

No reviewed product, test, or canonical-source-doc byte differed from the immutable candidate.

## Findings

### BLOCKER-1 — Completed-quarantine upload recovery drops the request ledger and deadline

**Locations:** `codefire:3644`-`3697`, `codefire:2999`, `codefire:13089`-`13106`, and the HTTP
dispatch handoff at `codefire:15359`.

The ordinary upload route receives the handler's exact `DecisionWorkLedger` and
`HTTPRequestDeadline`. After its publication preflight and first-effect fence, it calls
`execute_remote_json_publication`. That API accepts neither ledger nor deadline. Its recovery
branch calls `recover_remote_pending_quarantine`, whose root loop executes:

```python
validate_sealed_commit(remote_dirs(project_root)["objects"], root_id)
```

with neither caller object. The no-ledger compatibility path therefore creates a fresh
`DecisionWorkProfile.local_v1()` ledger named `sealed validation`.

**Reviewer reproducer:** `PYTHONDONTWRITEBYTECODE=1 python3
/tmp/cfr168_recovery_probe.py` against the exact candidate export; PASS as a defect reproducer in
`1.521s`.

The probe first creates a completed upload transaction and injects a cleanup crash at
`remove_remote_transaction_stage`. It then constructs the completed quarantine using the same
shape as the candidate's recovery fixtures, starts the exact retry with a fresh HTTP request ledger
and a 100 ms deadline, and advances the clock from `0.0` to `1.0` immediately before the unthreaded
root validation.

Observed facts:

- request retry ledger id and deadline id were distinct, live objects;
- the recovery call passed `ledger=None`, `deadline=None`, and no phase;
- a fresh ledger was minted with `command="sealed validation"` and
  `command_kind="local_graph"`;
- one validation session used the request ledger for the staged temporary store, while a second
  used the fresh ledger for the durable project object store;
- the request deadline ended with `limit_ms=100`, `poll_count=2350`, and
  `expired_phase=None`, because no request poll occurred after the clock advance;
- the exact retry returned `(200, {"branch":"main", "head":...})` and removed the completed
  quarantine;
- the request ledger and the fresh ledger both ended with zero current debt and no open permits.

This is not a speculative neighboring owner. It is directly upload-reachable through:

```text
HTTP POST /branches/<name>/upload
  -> handle_http_upload
  -> execute_remote_json_publication
  -> recover_remote_pending_quarantine
  -> validate_sealed_commit(objects, root)  # fresh ledger, no request deadline
```

**State impact:** the reproduced completed authorities were unchanged, but the completed
quarantine was deleted and a success response was selected after the deadline had elapsed. That is
an externally visible recovery/cleanup effect and response-authority decision outside the request
deadline and request accounting contract. It also makes the canonical statement that the request
ledger is the only sealed-validation ledger a Python upload uses false.

**Minimal closure:** add an exact request ledger/deadline handoff through
`execute_remote_json_publication` and `recover_remote_pending_quarantine`; validate recovered roots
on that same ledger/deadline; poll immediately before recovery cleanup/response effects; add a
completed-quarantine retry test that advances the clock at the durable-root validation and asserts
typed deadline bytes, unchanged quarantine/authorities, no fresh ledger, one root traversal, zero
debt, and no open permits.

### HIGH-2 — The caller-owned sealed-validation API cannot complete

**Locations:** `codefire:7008`-`7034`, `codefire:22439`-`22450`,
`codefire:30028`-`30082`, and the advertised reusable contract at
`docs/development/source-architecture.md:391`-`397`.

`_AccountedUploadSourceMemory.acquire` reserves only `SEALED_SESSION_CONTROL_UPPER` under
`retained_bytes`. `_control_reservation_outstanding` returns zero for every dimension other than
`retained_bytes`. `SealedCommitValidationSession.finalize`, however, correctly claims its ordered
object-id projection under `temporary_bytes` before allocating and sorting it. The source owner
therefore rejects every caller-owned finalization at the claim, before the projection allocation.

On this runtime:

- prepared retained control upper: `11,456` bytes;
- prepared temporary control upper: `0` bytes;
- empty ordered projection: `56` bytes;
- one-record projection: `64` bytes;
- 27-record projection: `272` bytes.

**Reviewer reproducer:** `f3_api.py` / `f3_real.py` against the exact candidate export.

| Public path | Exact result |
|---|---|
| `validate_sealed_commit(..., ledger=..., deadline=...)` | `CFError: upload commit source exceeded its prepared temporary_bytes reservation` |
| `validate_remote_json_publication_identity(..., ledger=..., deadline=...)` | Same `CFError` |
| `preflight_remote_json_publication(..., ledger=..., deadline=...)` | Same `CFError` |
| Legacy identity validation with no ledger | Succeeds |

The three failing caller-owned paths each reconciled to
`current={retained_bytes: 0, temporary_bytes: 0}`, no open permits, and no terminal reason. Cleanup
is therefore balanced, but the promised reusable behavior does not exist. The direct three-path
probe took `1.346s`.

**State impact:** the API fails pre-effect and leaves no debt, but it makes the candidate's
cross-owner repair mechanism unusable. In particular, it cannot be used as written to repair the
neighboring HTTP owners documented for a later packet.

**Minimal closure:** reserve or acquire the exact ordered-projection temporary capacity before
`finalize` allocates it, keep acquisition publication and retryable release ordering, and add direct
tests for all three APIs with caller ledger only and with caller ledger plus deadline. Tests must
exercise empty, one-record, and multi-record closures; exact/minus-one capacity; constructor,
sort, and release faults; and zero debt/permits on every exit.

### HIGH-3 — `RemoteJSONPublicationRootProof` accepts repeated consumption

**Location:** `codefire:3347`-`3417`, especially `codefire:3416`.

The class describes a request-owned certificate and the ticket requires it to be
single-use/revocable. Its `validated_roots` method checks revocation, deadline, object-store
identity, and root coverage, but it never rejects a prior consumption. It only increments
`_consumptions`.

**Reviewer reproducer:** `PYTHONDONTWRITEBYTECODE=1 python3
/tmp/cfr168_proof_reuse.py`; defect reproduced in `0.644s`:

```text
attempt=1 accepted ... consumptions=1
attempt=2 accepted ... consumptions=2
after_revoke=rejected ... remote publication root proof is revoked
```

The owner prevents minting a second proof, but that does not make an aliased proof object itself
single-use. A caller holding the same proof reference may reuse it while the underlying snapshot is
still live.

Other state-machine checks behaved correctly in reviewer probes:

- foreign object store rejected;
- uncovered root rejected;
- revoked proof rejected;
- `RemoteJSONPublicationIdentity` rejected wrong operation, actor, idempotency key, roots, and
  object store;
- a deadline created before the clock was advanced correctly rejected consumption with
  `RequestDeadlineExceeded` after the clock advanced.

The earlier `f1_proof.py` line that printed an accepted expired proof is not evidence of a defect:
that script advanced its clock before constructing the deadline. The corrected construction-then-
advance probe rejected in `0.664s` with one poll and phase `identity`.

**State impact:** the current normal upload consumes the proof once and later revokes it, so this
probe did not produce a durable upload mutation. The defect is nevertheless a live authority-state
violation, permits alias/reentry reuse, and fails an explicit F1 acceptance condition.

**Minimal closure:** atomically reject any call after the first successful consumption, preserve
deadline and identity-check precedence, and add tests for second use, aliased use, callback reentry,
failure-before-consumption, failure-after-state-transition, revoke-before-use, and stale snapshot
revocation.

### MEDIUM-4 — Mandatory reversal coverage is still vacuous for two accounting invariants

**Locations:** ordered projection at `codefire:22439`-`22450`; production coverage check at
`codefire:24032`-`24050`; focused tests in
`tests/test_python_http_upload_accounted_traversal.py`.

Independent temporary mutations produced these results:

| Mutation | Focused 22 tests | Broader 33 owner tests | Disposition |
|---|---:|---:|---|
| Remove the ordered-projection temporary claim while retaining list allocation/sort | `22/22` PASS | `33/33` PASS | Required reversal not detected |
| Replace `_assert_prepared_coverage` with a no-op | `22/22` PASS | `33/33` PASS | Required reversal not detected |
| Force `_assert_prepared_coverage` to raise | 43 errors / 3 failures | Not needed | Confirms production execution only |

The product currently contains both checks, but the ticket requires evidence that deleting them
changes a focused assertion from passing to failing. That evidence does not exist. The missing
projection test is especially material because HIGH-2 is the same projection/dimension boundary on
the new public path.

The candidate also mutates the `upload publication effect` poll and detects publication after
expiry, but there is no corresponding removal mutation for the distinct `upload layout bootstrap`
poll. Behavior at the layout fence was independently reproduced, but the mandatory reversal row is
still incomplete.

**Minimal closure:** add direct, focused assertions on ordered-projection acquisition-before-list,
projection release, and prepared-coverage high-water/use checks; add a layout-poll removal mutation;
and make the old-target comparison fail on behavior rather than missing symbols or line-number
differences.

## Mandatory prior-finding disposition

| Prior finding | Disposition on exact candidate | Independent evidence |
|---|---|---|
| **F1 / Blocker — duplicate request-bypassing traversals and missing final deadline fence** | **OPEN** | Ordinary upload is substantially repaired: one request-ledger session, no foreign ledger, one root traversal, 2,345 request polls, typed existing-project and bootstrap deadline failures, and no normal-path effect. Overall F1 is not closed because completed-quarantine upload recovery creates a fresh ledger with no deadline, and the root proof is reusable. |
| **F2 / High — false per-record retained/temporary admission cliff** | **CLOSED for product behavior** | Formula re-derived from claim sites; streaming matches canonical bytes; default profile admits 3 MiB and 3 MiB+1 records; first live refusal is staging; formula record/count ceilings are pinned. No public profile was raised and no validation bypass was observed. |
| **F3 / High — commit-source session constructed without a control owner** | **OPEN overall; live upload path closed** | Normal upload covered 18/18 control-claim sites with a matching owner and zero debt. The reusable caller-owned source owner reserves no temporary projection capacity, so its final construction phase always fails and its required retry matrix cannot complete. |
| **F4 / Medium — descriptor retention and unclaimed ordered projection** | **Product implementation closed on live path; mandatory proof incomplete** | Descriptor registry peak remains 10 and reuse-pool peak 1 through 814 reads. The projection is claimed before allocation in source. Removing that claim, however, passes both mutation suites, so the required non-vacuity condition is not closed. |

### F1 requirement-by-requirement

| Mandatory condition | Result |
|---|---|
| Exact handler ledger/deadline from body through cleanup | PASS on ordinary upload; FAIL on completed-quarantine retry |
| No live upload creates another sealed-validation ledger | PASS on ordinary upload; FAIL on recovery (`sealed validation` / `local_graph`) |
| Each publication root closure traversed exactly once | PASS on ordinary upload; FAIL overall because recovery revalidates the durable root on a fresh session |
| Root proof derived from request snapshot and rejects foreign store/root | PASS |
| Root proof single-use/revocable | **FAIL: two pre-revocation consumptions accepted** |
| Identity rejects foreign operation/actor/key/root/store | PASS in mismatch matrix |
| Preflight reuses identity without skipping final identity/deadline checks | PASS on ordinary upload |
| Final poll before layout bootstrap | PASS behavior; required removal mutation missing |
| Final poll before object copy and branch/generation publication | PASS behavior and publication-poll mutation |
| Existing-project 1.0 s / 100 ms leaves authority byte-identical | PASS |
| Missing-project 1.0 s / 100 ms creates no layout | PASS |
| Response effect-start transition occurs after its fence | PASS by source order and watched-effect probe on ordinary path; recovery still returns success after the bypass |

### Earlier residual-risk disposition

| Prior residual risk | Candidate disposition |
|---|---|
| Record-count ceiling unpinned | Resolved: 4,959 records at 805 B/record admitted; 4,960 first binds retained |
| Reservation slack large at scale | Still present but bounded: measured retained claims are about 14.7%-16.1% of reservation on the 1x/3x/10x divergent fixtures; maintainability risk remains |
| `_claimed_memory_peak` observation-only | Partly addressed by production `_assert_prepared_coverage`; deletion of the assertion remains undetected, so regression risk remains |
| Unclaimed structures invisible to coverage | Descriptor and projection claim sites exist; projection-claim deletion remains invisible to tests, and the reusable API exposes a real projection dimension bug |
| Nested-traversal blast radius beyond upload | Audited: four HTTP-reachable owner functions are exhaustive; one of them, recovery, is upload-reachable and still defective |
| Rust/local/file/other frozen surfaces not assessed | Still outside this report-only Python HTTP lane; full integration gate remains pending |

## Candidate-surface disposition

| Surface | Result | Evidence |
|---|---|---|
| `_canonical_escaped_text`, `canonical_json_stream`, `canonical_json_stream_digest` | PASS | 15 explicit shapes plus 5,000 fuzz values; zero byte/digest mismatches |
| Bounded stream scratch | PASS | 71,368-276,003 B measured peaks, all below 524,288 B constant; no whole-record duplicate peak |
| `RemoteJSONPublicationRootProof` | **FAIL** | Reuse accepted twice; revoke/store/root/deadline checks otherwise pass |
| `RemoteJSONPublicationIdentity` | PASS for exact mismatch checks | Wrong operation, actor, key, roots, and store rejected |
| `_AccountedUploadSourceMemory` | **FAIL on public caller-owned path** | Retained upper 11,456; temporary upper 0; finalize needs at least 56 temporary bytes |
| Upload `fence_first_effect` | PASS ordinary behavior; incomplete reversal | Existing and missing project probes stop before effects; recovery route bypasses deadline elsewhere |
| Request-owned proof/identity wiring | PASS ordinary route; proof contract FAIL | One root traversal and identity reuse observed; proof remains reusable |
| `SealedCommitValidationSession` control claims/releases | PASS live route | 18/18 claim sites owned; zero live-path debt |
| Descriptor registry/reuse pool | PASS | Peak 10 / 1 across linear, fan-in, and deep-chain fixtures |
| Ordered projection | Present in product; evidence incomplete | Claim precedes list/sort, but removal mutation passes |
| Retained/temporary formulas | PASS | Re-derived and boundary-measured below |
| Production `_assert_prepared_coverage` | Executes; protection incomplete | Forced raise reaches production; no-op mutation passes |
| Caller-supplied `ledger`/`deadline` in `validate_sealed_commit` | **FAIL** | Ordered projection exceeds zero temporary reservation |
| Caller-ledger identity/preflight helpers | **FAIL** | Same failure through both public helpers |
| Detached-document borrow-fingerprint release | PASS | Omitting release produces nine errors in broad owner suite |
| Dead publication-root proof plumbing removal | PASS | No residual call or state dependency reproduced |
| Nine focused tests and final count ceiling | Partial | Core cases pass; proof reuse, reusable API, recovery escape, projection claim, coverage no-op, and layout-poll reversal are not caught |
| Parse/validate/plan/preflight/apply/render/write separation | PASS ordinary route; recovery handoff incomplete | Normal route remains explicit; recovery API omits request execution context |
| Exception precedence and exact response behavior | PASS in executed suites | Focused precedence and selected legacy methods pass |
| Callback reentry / single-use transitions | **FAIL for root proof** | Proof object accepts repeated use |
| Coverage assertion after durable effect | No reproduced suppression | Assertion runs in sealed-owner finish before the first publication effect |

## Accounting formulas and owner lifetimes

Let `N` be staged record count, `A` aggregate canonical record bytes, and `M` maximum record bytes.
The exact candidate formulas are:

```text
retained =
    64 KiB
  + BORROW_CONTROL_BYTES
  + SEALED_SESSION_CONTROL_UPPER
  + PUBLICATION_PROOF_BYTES
  + PUBLICATION_PROOF_ROOT_BYTES * 64
  + (RETAINED_RECORD_BYTES + BORROW_FINGERPRINT_BYTES) * N
  + 2 * A
  + pair_plan_upper(N)

temporary =
    64 KiB
  + PAIR_PROBE_TEMPORARY_BASE_BYTES
  + pair_builder_temporary_upper(N)
  + ordered_projection_upper(N)
  + TEMPORARY_RECORD_BYTES * N
  + max(
        2 * M,
        stream_scratch_upper(M),
        M + stream_scratch_upper(M)
    )

stream_scratch_upper(M) = 512 KiB + 3 * M
ordered_projection_upper(N) = sizeof([]) + pointer_bytes * N
```

On the reviewed CPython runtime, `sizeof([])=56` and `pointer_bytes=8`.

| Formula term | Actual simultaneously-live owner / phase | Lifetime and overlap conclusion |
|---|---|---|
| `64 KiB` retained base | Sealed owner fixed fields | Whole sealed-owner lifetime |
| `BORROW_CONTROL_BYTES` | Preallocated signature/ancestry borrow handles | Whole sealed-owner lifetime |
| `SEALED_SESSION_CONTROL_UPPER` (`11,456` B) | Path, document memo, validated set, descriptor registry/pool, subdirectory map, root and bounded descriptor owners | Acquired before session constructor; whole session lifetime |
| Publication proof base + 64 root slots | `RemoteJSONPublicationRootProof` and certified IDs | Mint through final snapshot fence; released on revoke |
| `(4,096 + 1,024) * N` | `RetainedObjectDocument`/frame control plus borrow fingerprints | Documents and fingerprints may all coexist through final detach |
| First `A` | Retained canonical documents | All staged documents retained for final revalidation |
| Second `A` | Variable traversal-frame IDs | Active frames coexist with retained documents; formula is conservative |
| `pair_plan_upper(N)` | Terminal pair tuples and descriptor slots | Pair-probe preparation/cleanup |
| `64 KiB + 4 KiB` temporary bases | Owner bookkeeping and terminal pair probe | Temporary owner lifetime |
| `pair_builder_temporary_upper(N)` | Largest one of ordered document, borrowed-document, or subdirectory builder lists | Builders are alternatives; maximum, not sum |
| `ordered_projection_upper(N)` | `finalize` list and in-place heapsort | Whole snapshot-revalidation pass; must be acquired before list allocation |
| `128 * N` | Session temporary per-record bookkeeping | Concurrent with projection/pair base |
| `2*M` | Growing read buffer plus immutable snapshot | Released after one record read |
| `512 KiB + 3*M` | Canonical stream chunks and sorted-key projections | Released after one streamed record |
| `M + stream_scratch` | Terminal physical read buffer plus streamed canonical fingerprint | Pair-probe maximum; does not overlap earlier read/fingerprint owners |

The scratch maxima are correctly combined with `max`, not summed. The retained formula contains no
`maximum_record_bytes` multiplier. No reviewed public limit changed.

The defect in HIGH-2 is outside this sealed-owner formula: standalone `_AccountedUploadSourceMemory`
reserves retained session control but no temporary ordered projection.

## Scale evidence

All scale probes used the unmodified `DecisionWorkProfile.http_v1()`:

```text
retained_bytes       33,554,432 concurrent
temporary_bytes      16,777,216 concurrent
request_body_bytes   16,777,216 cumulative
aggregate_bytes       1,073,741,824 cumulative
graph_nodes             100,000 cumulative
graph_edges             400,000 cumulative
logical_depth           100,000 maximum
maximum_frontier        100,000 maximum
response_bytes       16,777,216 cumulative
request/response headers   65,536 cumulative each
MAX_OBJECT_RECORD_BYTES 67,108,864
request deadline          30,000 ms
```

Reviewer-authored exact candidate probe command:

```bash
PYTHONDONTWRITEBYTECODE=1 EXPORT=/tmp/cfr168_cand.W3YBDT python3 <scale-probe>
```

Result: PASS in `15.920s`. Every row returned the exact 409 body
`{"error":"upload rejected: remote branch is not an ancestor of local branch head"}`
(`82` canonical bytes), kept the branch unchanged, left the divergent tip absent, released all
current debt, and held no permits.

| Factor | Records | Aggregate / max record B | Ancestry nodes / reads | Request graph edges | Depth / max frontier | Retained reservation = ledger peak | Temporary reservation = ledger peak | Claimed retained / temporary peak |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1x | 127 | 99,647 / 813 | 121 / 121 | 966 | 120 / 121 | 951,382 | 615,580 | 139,517 / 2,698 |
| 3x | 367 | 294,767 / 813 | 361 / 361 | 2,886 | 360 / 361 | 2,574,262 | 650,140 | 403,997 / 4,618 |
| 10x | 1,207 | 979,088 / 815 | 1,201 / 1,201 | 9,606 | 1,200 / 1,201 | 8,257,144 | 771,108 | 1,331,078 / 11,342 |

Observed slopes per additional staged record:

- retained reservation: `6,762.0` B from 1x to 3x and `6,765.3` B from 3x to 10x;
- measured retained claim: `1,102.0` B and `1,103.7` B;
- temporary reservation: `144.0` B and `144.01` B;
- formula sensitivity to aggregate: exactly `2` retained bytes per additional canonical byte.

The graph-shape focused test also reaches wide fan-in, stacked/shared diamonds, signed history,
shared DAG, and deep chain. The independent descriptor probe below covers wide fan-in and a
400-node deep chain.

## Record, encoding, and boundary evidence

### Live record-size curve

Reviewer-authored live uploads on the exact candidate produced:

| Message-field size | Result |
|---:|---|
| 64 KiB | 200 |
| 98,304 B | 200 |
| 126,976 B | 200 |
| 128 KiB | 200 |
| 256 KiB | 200 |
| 1 MiB | 200 |
| 3 MiB | 200 |
| 3 MiB + 1 | 200 |
| 4 MiB | Typed `ResourceBudgetExceeded` in `staging` |
| 8 MiB, 12 MiB, 16,000,000 B | Typed `ResourceBudgetExceeded` in `staging` |

A binary search measured the largest admitted message field as `3,354,531` B and the smallest
refused field as `3,354,532` B. The first refusal was the named staging temporary owner:

```text
resource budget exceeded ... during staging: 16,777,221 observed, limit 16,777,216
```

This reproduces the canonical-doc claim that a single record above 3 MiB is admitted and that the
first live refusal is staging rather than sealed validation. The boundary probe took `100.806s`;
the size curve took `34.056s`.

### Formula ceilings

Under the unchanged default profile:

| Boundary | Last admitted | First refused | First binding owner |
|---|---:|---:|---|
| Single-record formula fixture (`N=101`, aggregate `66,000 + M`) | `M=4,042,158` | `M=4,042,159` | sealed temporary; `16,777,216` then `16,777,220` |
| Record-count formula fixture (`805` B/record) | `N=4,959` | `N=4,960` | sealed retained; `33,553,230` then `33,559,976` |

The single-record formula ceiling is not the same quantity as the live message-field boundary: the
formula uses the supplied record-byte upper and a fixed 101-record aggregate fixture, while the
live probe exercises the whole HTTP staging ownership chain.

### Canonical streaming

`canonical_json_stream_digest` matched `canonical_json` for 15 explicit nested/text shapes and
5,000 generated values, with zero mismatch. Explicit content included ASCII, CJK, emoji,
quote/backslash/newline/tab-heavy text, control characters, mixed Unicode, nested maps/lists, empty
values, and key-heavy mappings.

| Shape | Stream peak B | Whole-record encode peak B | Canonical size B | Below 512 KiB scratch |
|---|---:|---:|---:|---|
| 720 KiB control-heavy | 276,003 | 7,928,827 | 3,962,888 | Yes |
| 2 MiB control-heavy | 272,723 | 22,545,336 | 11,272,200 | Yes |
| 2 MiB ASCII | 71,368 | 4,195,256 | 2,097,160 | Yes |
| 2 MiB CJK | 173,505 | 3,584,379 | 2,150,408 | Yes |

The stream identity probe took `2.333s`; the scratch comparison took `8.996s`.

### Descriptor and ordered-projection evidence

Independent descriptor probe command completed in `8.180s`:

| Shape | Records | Reads | Descriptor registry peak | Reuse-pool peak | Cleanup |
|---|---:|---:|---:|---:|---|
| 1x linear 20 | 27 | 54 | 10 | 1 | Zero current |
| 3x linear 60 | 67 | 134 | 10 | 1 | Zero current |
| 10x linear 200 | 207 | 414 | 10 | 1 | Zero current |
| Wide fan-in 40 | 48 | 96 | 10 | 1 | Zero current |
| Deep chain 400 | 407 | 814 | 10 | 1 | Zero current |

The source shows `finalize` claiming `56 + 8*N` temporary bytes before `list(self.documents)` and
the in-place sort. The required mutation removing that claim is not detected, as recorded above.

### Exact/minus-one/plus-one boundary matrix

| Boundary named by ticket | Candidate evidence executed | Reviewer conclusion |
|---|---|---|
| Request body bytes | Python-pattern sweep includes HTTP request admission/budget tests; public limit confirmed 16 MiB | No independent full 16 MiB exact/-1/+1 fixture was re-authored in this lane |
| Record/file bytes | Live reviewer size curve plus hard `MAX_OBJECT_RECORD_BYTES=64 MiB`; focused exact record curves | Core F2 boundary reproduced |
| Aggregate bytes | Object-set and stage owner exact/minus-one tests passed in 443 sweep | Not independently remeasured at the 1 GiB public ceiling |
| Retained bytes | Focused `test_retained_boundaries_reach_the_streaming_validation_owner` | Exact/minus-one/plus-one reached sealed owner |
| Temporary bytes | Focused `test_temporary_boundaries_reach_the_streaming_validation_owner` | Exact/minus-one/plus-one reached sealed owner |
| Graph nodes / edges | Neighbor suites include exact graph limit tests | Passed; 1x/3x/10x work counts reproduced |
| Depth / frontier | Neighbor ancestry tests bind frame growth, depth, and frontier | Passed |
| Response body / headers | Response-input exact/overflow tests passed in 443 sweep | No independent raw-wire 16 MiB/64 KiB probe in this lane |
| Session control | 18/18 live control sites covered | Reusable source path fails its temporary projection |
| Publication proof | Proof allocation/revoke behavior exercised | Single-use fails; exact/-1/+1 byte limit not independently re-authored |
| Publication-response owners | Existing publication/response owner exact/minus-one tests passed in 443 sweep | Selected upload identity/preflight legacy methods pass |

The missing independently-authored rows are evidence limits, not passing acceptance evidence.

## Durable-state evidence

### Ordinary existing-project final-effect deadline

The exact 1.0 s / 100 ms injection at `upload publication effect` produced
`RequestDeadlineExceeded`, retained the old branch head, left the new object absent, changed no
tracked authority path, ended current debt at zero, and held no permits. The phase tail was:

```text
upload ancestry -> publication preflight -> upload publication effect
```

### Missing-project layout deadline

The exact injection at `upload layout bootstrap` produced `RequestDeadlineExceeded`, did not create
the project root, ended current debt at zero, and held no permits. The immediately preceding phases
included server verification and `upload final subject fence`.

### Portable old-target reversal

The same preflight-return clock advance on `dece688afea815d1242e7751367052d1ceac4580`
returned 200, moved the branch, published the child object, and changed
`branches/main.json` plus the object-store digest. On the exact candidate it raised typed
`RequestDeadlineExceeded` during `upload publication effect`, left the branch/object unchanged,
changed no path, and reconciled the ledger to zero. This comparison is behaviorally non-vacuous and
does not depend on a new symbol.

### Completed-quarantine retry snapshot

The exact recovery defect run recorded these before/after digests:

| Authority | Before | After | Changed |
|---|---|---|---|
| Objects | `aca9130967b1c6d7931731c758589988ea6d23422dd044b7160b31b9f8a48fd7` | same | No |
| Branches | `612a1a766805426dc7f6e0bd46dcbd40e2312647ab8ce6baed43faad5fa37997` | same | No |
| Generation | `b135d967e76db82c2a6212be7ff8d3c2b4d74b585ca957763b3eb5f68d807120` | same | No |
| Nonce | absent | absent | No |
| Pending transaction | absent | absent | No |
| Receipts | `104252bf349233633a4898bdab1569a0991e8e3f3223107f411a2796d19a5d57` | same | No |
| Audit | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | same | No |
| Completed quarantine | `1785e9b0da88a764665dbe8934927f39eb79b9579881ec3780d72d2048adcc5d` | absent | **Yes** |

The retry returned 200 after the clock advance. Both involved ledgers reconciled to zero. The probe
constructs the completed quarantine by injecting a cleanup crash and applying the candidate's
existing recovery-fixture transition; it is not a full operating-system crash replay. It is still
an exact production retry through the defective call at `codefire:2999`.

### Other failure-state coverage

The focused and legacy suites preserve exact 200/409/error/retry behavior for signature required
and optional precedence, fast-forward and non-fast-forward decisions, malformed/corrupt/missing
history, nonce races, receipt replay, final-subject rejection, completed quarantine, Doctor,
branch protection, and merge-request list/review/apply. Reviewer-authored exact durable digests were
not captured for every budget/tamper/encoding/fault permutation; those rows rely on the passing
candidate tests and are not elevated to independent acceptance evidence.

## Reversal and non-vacuity matrix

| Required reversal | Exact counterexample / changed assertion | Result |
|---|---|---|
| Restore nested publication-root traversal | Candidate mutation path mints `sealed validation`, traverses child root twice, and finds a foreign-ledger session | **Detected** |
| Remove publication-effect deadline poll | `test_removing_the_final_effect_poll_publishes_an_expired_request`: status changes from typed deadline to 200, branch moves, object appears | **Detected** |
| Remove layout-bootstrap deadline poll | No removal mutation executed; behavior-only bootstrap probe passes | **Not closed** |
| Omit source owner | Focused mutation: 7 failures, including zero covered source sessions and zero outstanding control bytes | **Detected** |
| Omit each individual control reservation | Whole-owner omission detected; individual reservation-site deletion matrix not executed | **Partial** |
| Restore old per-record multiplier | Focused mutation: 25 failures; 256 KiB-3 MiB admissions revert to semantic-validation refusal | **Detected** |
| Retain one descriptor owner per read | Focused mutation: 2 failures / 11 errors; registry bounds and traversal behavior fail | **Detected** |
| Remove ordered-projection claim | Focused `22/22` PASS; broad `33/33` PASS | **Not detected** |
| Change one emitted canonical byte | Focused mutation: 2 failures in streamed canonical identity test | **Detected** |
| Replace bounded stream with whole-record encode | No dedicated whole-encode source mutation; tracemalloc boundary would be expected to detect the measured peak | **Partial** |
| Disable production `_assert_prepared_coverage` | Focused `22/22` PASS; broad `33/33` PASS | **Not detected** |
| Force production `_assert_prepared_coverage` to raise | 43 errors / 3 failures | Confirms call executes, not check non-vacuity |
| Accept reused/revoked/foreign proof or identity | Reviewer reuse probe is accepted twice; revoked/foreign cases reject | **Candidate defect reproduced** |
| Omit detached-document fingerprint release | Focused `22/22` PASS, but broader 33-test run reports 9 retained-debt errors | **Detected only by broad matrix** |

Mutation-run timings and exact totals:

| Mutation | Result | Test time |
|---|---|---:|
| `m3_omit_source_owner` | 7 failures | 105.821s |
| `m4_restore_multiplier` | 25 failures | 81.750s |
| `m5_no_recycle` | 2 failures / 11 errors | 67.603s |
| `m6_no_projection_claim` focused | 22 PASS | 82.703s |
| `m7_change_canonical_byte` | 2 failures | 83.011s |
| `m8_disable_coverage` focused | 22 PASS | 101.759s |
| `m8b_force_raise` | 3 failures / 43 errors | 26.759s |
| `m10_no_fp_release` focused | 22 PASS | 102.516s |
| `m6_no_projection_claim` broad | 33 PASS | 29.933s |
| `m8_disable_coverage` broad | 33 PASS | 33.058s |
| `m10_no_fp_release` broad | 9 errors | 36.504s |

The complete repair-specific candidate suite was not accepted as old-target reversal evidence when
run against `dece688a`: many failures would be caused only by missing candidate symbols/fixtures or
line changes. The portable deadline probe above is admissible; a complete non-vacuous old-target
repair matrix remains pending.

## Neighboring HTTP call-site audit

An AST inventory found exactly 28 production `validate_sealed_commit` call occurrences.

| Group | Occurrences | Lines | Shortest dispatch route / role |
|---|---:|---|---|
| `open_context` | 2 | 1808, 1816 | Local `scan/status/fire/extinguish/verify/commit` helpers |
| Completed-quarantine recovery | 1 | 2999 | HTTP upload or other remote publication -> `execute_remote_json_publication` -> recovery |
| Reusable publication-identity helper | 2 | 3499, 3501 | Helper fallback; normal upload uses a root proof, caller-ledger path currently fails HIGH-2 |
| HTTP list owner | 1 | 12133 | GET branch list -> `handle_http_list` |
| HTTP remote-branch owner | 1 | 12226 | HTTP MR merge/apply -> `http_remote_branch` |
| HTTP merge-request record owner | 1 | 12254 | HTTP MR list/review/apply -> `validate_http_merge_request_record` |
| Upload legacy no-budget fallback | 1 | 12778 | Direct `handle_http_upload(..., decision_work=None)` only; real HTTP dispatch supplies ledger/deadline |
| File/CLI merge-request validation | 1 | 21337 | Local request list/review/apply validation |
| `resolve_commitish` | 2 | 32919, 32922 | Local `show` / `diff` resolution |
| `resolve_commitish_any` | 3 | 32940, 32951, 32957 | HTTP/file/local remote `show` / `diff` resolution |
| `cmd_branch_list` | 1 | 33109 | Local branch list |
| `cmd_open` | 1 | 33118 | Local open |
| `cmd_clone` | 3 | 33196, 33216, 33235 | Local/file/HTTP clone paths |
| `cmd_upload` | 1 | 33255 | CLI upload source branch |
| `cmd_request_merge` | 2 | 33471, 33472 | File-remote MR creation |
| `cmd_request_list` | 1 | 33594 | File-remote MR list |
| `cmd_request_apply` | 2 | 33857, 33858 | File-remote escrow source/target |
| `cmd_merge` | 2 | 34202, 34203 | Local merge source/target |
| **Total** | **28** |  |  |

The four HTTP-reachable owner functions outside the ordinary upload repair are exhaustive:

| Owner | Receives an upload request ledger/deadline? | Candidate behavior |
|---|---|---|
| `recover_remote_pending_quarantine` | **Yes, by route; silently drops both** | **Defective, BLOCKER-1** |
| `handle_http_list` | No upload request context is passed to this endpoint | Existing behavior unchanged in selected tests |
| `http_remote_branch` | No upload request context is passed | Existing MR/branch behavior unchanged in selected tests |
| `validate_http_merge_request_record` | No upload request context is passed | Existing MR list/review/apply behavior unchanged in selected tests |

No fifth HTTP owner was found. The two helper-internal sites at 3499/3501 are the reusable API, not
a fifth dispatch owner. The upload-local site at 12778 is guarded by `decision_work is None`; real
HTTP dispatch passes both `self.decision_work` and `self.request_deadline` at line 15359. The normal
HTTP upload probe observed no call to that fallback.

The reusable API shape is adequate for the separately tracked list/branch/MR repairs, but its
implementation is not currently usable because HIGH-2 fails every caller-owned finalization.

## Validation on the exact candidate

The exact candidate was exported to `/tmp/cfr168_cand.W3YBDT`. All six candidate blobs were verified
against Git before execution. `PYTHONDONTWRITEBYTECODE=1` was used for Python commands.

| Command | Exact result | Time / evidence |
|---|---|---|
| `python3 -m py_compile codefire tests/test_python_http_upload_accounted_traversal.py tests/test_codefire_cli.py` | PASS | 0.775s |
| `python3 -m unittest tests.test_python_http_upload_accounted_traversal -v` | 22/22 PASS | unittest 82.910s; wall metadata 83.129s |
| `python3 -m unittest tests.test_python_ancestry_owner tests.test_python_signature_owner tests.test_python_signature_terminal_freeze tests.test_python_sealed_production tests.test_python_external_production -v` | 59/59 PASS | unittest 26.231s; wall metadata 27.004s |
| `python3 -m unittest discover -s tests -p 'test_python_*.py'` | 443/443 PASS | unittest 304.748s; wall metadata 305.774s |
| `python3 docs/development/tools/check_source_docs.py` | PASS: 44 modules, 34 commands, 12 object kinds | 0.249s |
| `python3 docs/development/tools/check_project_docs.py` | PASS: 528 Markdown files | 0.909s |
| `git diff --check 5f798990...76baff84` | PASS | 0.076s |
| Selected legacy methods below | 13/13 PASS | unittest 202.701s |

The exact gate commands, with candidate-export working directory unless noted, were:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire \
  tests/test_python_http_upload_accounted_traversal.py tests/test_codefire_cli.py
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_upload_accounted_traversal -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_ancestry_owner \
  tests.test_python_signature_owner \
  tests.test_python_signature_terminal_freeze \
  tests.test_python_sealed_production \
  tests.test_python_external_production -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover \
  -s tests -p 'test_python_*.py'
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check \
  5f798990c313ea2e1b117bc74824de3f3ce028d2..\
76baff84cb1192e11b26dcc669fb800f7b13fa30
```

The exact 22 focused traversal methods were:

```text
test_accounted_signature_policy_matches_the_legacy_first_error
test_ancestry_collection_budget_precedes_its_allocators
test_deadline_expiry_in_both_phases_precedes_mutation
test_deep_divergent_history_traversal_is_iterative_and_charged
test_default_profile_record_shape_and_size_curves
test_exact_and_exact_minus_one_budgets_precede_mutation
test_exact_request_ledger_and_deadline_reach_both_traversals
test_expired_request_cannot_publish_or_bootstrap_a_layout
test_graph_shape_curves_reach_the_same_owner
test_immediate_parent_fast_forward_stops_goal_directed
test_new_live_path_replaces_the_reproduced_uncharged_traversals
test_one_upload_owns_every_sealed_validation_ledger
test_record_shape_curves_reach_the_same_owner
test_removing_the_final_effect_poll_publishes_an_expired_request
test_reproduced_all_record_reservation_fails_the_default_profile
test_retained_boundaries_reach_the_streaming_validation_owner
test_scale_curves_stay_within_the_owner_lifetime_bound
test_shared_dag_traversal_is_deterministic_and_charged_once
test_signature_policy_decisions_and_precedence_are_preserved
test_source_session_allocations_prove_an_outstanding_reservation
test_streamed_canonical_encoder_matches_the_whole_record_encoder
test_temporary_boundaries_reach_the_streaming_validation_owner
```

The exact 59 neighboring methods were:

```text
tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_allocation_and_post_effect_faults_reconcile
tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_bfs_full_walk_preserves_parent_and_late_error_order
tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_child_depth_is_materialized_only_after_frame_acquisition
tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_collection_and_graph_limits_precede_allocators_and_loaders
tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_collection_growth_uppers_cover_cpython_through_100k
tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_deadline_polls_before_loader_and_between_nodes
tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_deep_chain_is_iterative
tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_diamond_revisit_is_bfs_and_loads_each_commit_once
tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_frame_growth_depth_and_frontier_limits_precede_append
tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_invalid_subject_closes_without_calling_loader
tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_payload_precedes_retryable_release_and_foreign_close_is_rejected
tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_registration_consume_release_and_cancel_post_effect_faults
tests.test_python_ancestry_owner.PythonAncestryOwnerTests.test_transition_handoff_faults_release_the_fixed_gate
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_active_frontier_frame_stays_owned_through_node_loader
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_cleanup_retry_foreign_ledger_and_zero_permits
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_dense_and_deep_policy_upper_exact_and_minus_one
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_diamond_is_lifo_and_counts_revisited_edge_once
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_disabled_policy_has_zero_graph_work
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_false_loader_upper_rejects_before_serializer_collections
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_frontier_and_seen_allocation_faults_rollback_and_retry_cleanup
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_frontier_and_seen_growth_hold_temporary_overlap_before_insert
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_graph_node_and_edge_limits_are_exact
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_head_and_history_required_preserve_legacy_unsigned_rules
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_hmac_failure_traceback_drops_serializer_payload_before_release
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_hmac_key_sort_is_deadline_polled
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_invalid_revoked_and_expired_key_keep_first_parent_error
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_mid_history_and_hmac_deadlines_release_every_owner
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_one_three_and_ten_node_curves_charge_actual_work
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_original_cancel_then_raise_preserves_business_error_every_path
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_original_consume_then_raise_reconciles_every_permit_path
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_original_release_then_raise_preserves_collection_rollback_error
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_original_release_then_raise_reconciles_and_keeps_business_error
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_permit_consume_and_cancel_failures_keep_retry_ownership
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_policy_ast_and_collection_upper_is_current_before_parse
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_policy_failure_tracebacks_drop_parser_payload_before_release
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_policy_retained_and_hmac_temporary_upper_exact_and_minus_one
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_post_consume_memory_error_is_recovered_by_transition_scalar_owner
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_post_registration_fault_keeps_every_caller_owned_permit_recoverable
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_pre_registration_failure_retires_every_published_transition
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_release_state_does_not_depend_on_fallible_integer_subtraction
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_streaming_hmac_matches_canonical_json_for_nested_escaped_payload
tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_thousand_node_chain_retires_scalar_free_transitions_immediately
tests.test_python_signature_terminal_freeze.PythonSignatureTerminalFreezeTests.test_missing_project_bootstrap_invalidates_policy_authority
tests.test_python_signature_terminal_freeze.PythonSignatureTerminalFreezeTests.test_policy_first_leaves_sealed_callback_policy_window
tests.test_python_signature_terminal_freeze.PythonSignatureTerminalFreezeTests.test_sealed_first_leaves_policy_callback_staged_window
tests.test_python_signature_terminal_freeze.PythonSignatureTerminalFreezeTests.test_signature_only_finish_precludes_future_ancestry_borrow
tests.test_python_sealed_production.PythonUploadSealedProductionTests.test_constructor_and_release_faults_cleanup_before_layout
tests.test_python_sealed_production.PythonUploadSealedProductionTests.test_deadline_during_sealed_validation_has_no_layout_or_nonce
tests.test_python_sealed_production.PythonUploadSealedProductionTests.test_final_revalidation_rejects_staged_swap_before_layout
tests.test_python_sealed_production.PythonUploadSealedProductionTests.test_invalid_root_keeps_error_order_and_missing_project_absent
tests.test_python_sealed_production.PythonUploadSealedProductionTests.test_no_ledger_path_keeps_legacy_validation
tests.test_python_sealed_production.PythonUploadSealedProductionTests.test_production_exact_minus_one_precedes_session_allocation
tests.test_python_sealed_production.PythonUploadSealedProductionTests.test_stage_moves_exact_mixed_bounds_once_and_production_uses_them
tests.test_python_external_production.PythonExternalProductionTests.test_deadline_has_no_layout_or_nonce
tests.test_python_external_production.PythonExternalProductionTests.test_invalid_root_precedes_verification_effect
tests.test_python_external_production.PythonExternalProductionTests.test_no_ledger_keeps_local_wrapper_and_policy_effect
tests.test_python_external_production.PythonExternalProductionTests.test_process_read_fault_reaps_before_return
tests.test_python_external_production.PythonExternalProductionTests.test_release_retry_preserves_run_error_and_accounting
tests.test_python_external_production.PythonExternalProductionTests.test_same_ledger_exact_minus_one_rejects_before_spawn
```

Selected legacy methods executed exactly:

1. `test_upload_rejects_non_fast_forward`
2. `test_http_remote_upload_list_and_clone`
3. `test_python_upload_nonce_race_stays_pre_effect_and_exactly_retryable`
4. `test_remote_upload_receipt_replays_across_file_and_http_transports`
5. `test_python_completed_pending_quarantine_preserves_missing_receipt`
6. `test_python_remote_publication_recovers_authoritative_staging_and_replays_receipt`
7. `test_remote_doctor_detects_invalid_sealed_commit_references`
8. `test_python_remote_list_and_doctor_use_one_zero_current_ledger`
9. `test_remote_list_rejects_invalid_branch_head`
10. `test_request_list_and_review_reject_invalid_merge_request_refs`
11. `test_remote_branch_protection_gates_upload_and_apply`
12. `test_python_upload_publication_identity_rejects_before_effect`
13. `test_python_upload_publication_plan_rejects_before_effect`

The selected legacy command was:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_codefire_cli.CodeFireCliTests.test_upload_rejects_non_fast_forward \
  tests.test_codefire_cli.CodeFireCliTests.test_http_remote_upload_list_and_clone \
  tests.test_codefire_cli.CodeFireCliTests.test_python_upload_nonce_race_stays_pre_effect_and_exactly_retryable \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_upload_receipt_replays_across_file_and_http_transports \
  tests.test_codefire_cli.CodeFireCliTests.test_python_completed_pending_quarantine_preserves_missing_receipt \
  tests.test_codefire_cli.CodeFireCliTests.test_python_remote_publication_recovers_authoritative_staging_and_replays_receipt \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_doctor_detects_invalid_sealed_commit_references \
  tests.test_codefire_cli.CodeFireCliTests.test_python_remote_list_and_doctor_use_one_zero_current_ledger \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_list_rejects_invalid_branch_head \
  tests.test_codefire_cli.CodeFireCliTests.test_request_list_and_review_reject_invalid_merge_request_refs \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_branch_protection_gates_upload_and_apply \
  tests.test_codefire_cli.CodeFireCliTests.test_python_upload_publication_identity_rejects_before_effect \
  tests.test_codefire_cli.CodeFireCliTests.test_python_upload_publication_plan_rejects_before_effect -v
```

Reviewer-authored probe timings:

| Probe | Result | Time |
|---|---|---:|
| Ordinary upload ledger/session/poll inventory | PASS: one session, no foreign ledger, 2,345 polls | 1.521s |
| Existing-project final-effect deadline | PASS: typed deadline, no effect | 1.345s |
| Missing-project layout deadline | PASS: no layout | 1.345s |
| Canonical stream identity | PASS: 15 + 5,000 cases | 2.333s |
| Stream scratch comparison | PASS | 8.996s |
| Live size curve | PASS as boundary evidence | 34.056s |
| Binary-searched live staging boundary | PASS as boundary evidence | 100.806s |
| Live source control claims | PASS: 18/18 sites | 1.258s |
| Caller-owned public API | **Defect reproduced** | 1.346s |
| Proof reuse | **Defect reproduced** | 0.644s |
| Corrected expired-proof construction | PASS: typed deadline | 0.664s |
| Completed-quarantine recovery | **Defect reproduced** | 1.521s |
| Exact 1x/3x/10x scale table | PASS | 15.920s |
| Descriptor registry/reuse curves | PASS | 8.180s |

The malformed `elapsed_ms` value in `/tmp/cfr168_rerun_legacy.meta` was not used; unittest's
`202.701s` is the recorded legacy duration. An attempted timing wrapper using absent
`/usr/bin/time` did not execute a gate and is not cited as evidence. A preliminary scale wrapper
printed valid rows but exited after an auxiliary slope helper was constructed without `setUp`; the
corrected `15.920s` scale run above is the cited result.

## Skipped and pending gates

- The full `AGENTS.md` all-language/all-test gate was not run in this reviewer lane.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace` remain pending for the manager integration gate.
- The repository-wide non-pattern Python unittest discovery and `./demo.sh` remain pending for the
  manager integration gate.
- The 443-test `test_python_*.py` sweep is not represented as an all-language or complete repository
  gate.
- The complete repair-specific suite was not accepted as a baseline run on `dece688a` where missing
  candidate symbols or fixture imports made failures vacuous.

## Minimal closure requirements

1. Thread the exact upload request ledger and deadline through completed-quarantine recovery; add a
   clock-advanced exact-retry regression with quarantine and response bytes unchanged.
2. Give `_AccountedUploadSourceMemory` temporary ownership for the ordered projection and directly
   test every public caller-ledger helper on empty, one-record, and multi-record closures.
3. Make `RemoteJSONPublicationRootProof` atomically single-use and add alias/reentry/revoke tests.
4. Add non-vacuous tests that fail when the ordered-projection claim or any production coverage
   check is removed; add the missing layout-bootstrap poll reversal.
5. Re-run the exact candidate gate and a behaviorally portable old-target reversal matrix; do not
   count missing-symbol failures as evidence.
6. Update the canonical source docs only after the implementation makes the “only upload ledger”
   and reusable API statements true.

## Residual risk

- The ordinary upload repair is large (`+2,227/-120`) and crosses streaming, memory ownership,
  publication identity, durable effect fencing, recovery, and source docs. The undetected mutation
  rows show that passing candidate tests alone is not sufficient evidence for those boundaries.
- The retained reservation remains conservative by roughly 6x-7x over measured live retained
  claims at the tested scales. This is safe, but future scale targets may encounter another formula
  cliff well before actual pressure.
- The proof object and public helper APIs are reusable surfaces, not upload-local implementation
  details. Their current defects increase the risk of a neighboring repair appearing accounted
  while still failing or accepting stale authority.
- Rust, local/file transport, GC/metrics, and unrelated frozen CFR-168 surfaces were not reassessed
  in this report-only lane.

No conditional acceptance is offered.
