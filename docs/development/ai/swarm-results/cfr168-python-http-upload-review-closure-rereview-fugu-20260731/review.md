# CFR-168 / CFB-138 Python HTTP Upload Review-Closure Independent Rereview

Captured: 2026-07-31

Reviewer: fresh Sakana Fugu 1.1 Ultra (`fugu-ultra-v1.1`), independent
report-only lane.

Reviewed subject: exact immutable candidate
`467b0c27715e424ae18ffb5664da053b55915e2d` only.

## Decision

**`REQUEST_CHANGES`**

The exact successor closes the completed-quarantine request-budget escape and
the publication-root proof reuse defect. Nominal caller-owned sealed
validation now completes and the nine requested mutations all produce
non-green focused runs. Acceptance is nevertheless forbidden for two
independently reproduced residuals:

1. **HIGH-2 remains open.** `_AccountedUploadSourceMemory` and
   `SealedCommitValidationSession` erase claim/owner state before a fallible
   `DecisionWorkLedger.release`. A pre-effect release failure irrecoverably
   leaves `11,456` retained bytes on each public caller-owned API, or `432`
   temporary bytes on ordered-projection cleanup. The owner is already closed
   and unacquired with zero recorded claims; owner and API retries succeed but
   cannot release the debt.
2. **MEDIUM-4 remains open.** The ordered-projection and source-temporary
   reversal rows (`m1`, `m9`) fail only when the sort observer checks ledger
   state after `ordered_object_ids = list(self.documents)` has already made the
   unowned allocation. They therefore do not satisfy the ticket's required
   pre-allocation, non-vacuous oracle.

Literal prior-finding dispositions:

| Finding | Disposition |
|---|---|
| `BLOCKER-1` — completed-quarantine recovery request ownership | **CLOSED** |
| `HIGH-2` — usable caller-owned sealed-validation APIs | **OPEN** |
| `HIGH-3` — atomically single-use publication-root proof | **CLOSED** |
| `MEDIUM-4` — non-vacuous reversal matrix | **OPEN** |

No product, test, canonical source-document, packet, card, existing report,
issue, ledger, or lifecycle byte was modified by this review. Reviewer
mutations and probes were isolated under
`/tmp/cfr168-rereview-f8838c44a6ff-20260731`; the only repository write is
this report.

## Immutable Subject and Provenance

The exact candidate was checked out detached in a fresh scratch worktree
before behavioral review. No mutable branch tip or implementer worktree was
used for mutations.

| Field | Recomputed value |
|---|---|
| Candidate HEAD | `467b0c27715e424ae18ffb5664da053b55915e2d` |
| Parent | `f0ecd25ed18005d3f50b54dd254b7d05e678afb3` |
| Tree | `3b129b9e41ae45defe4d6e7c43da04b016f07404` |
| Continuation base / merge base | `b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b` / `b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b` |
| Ordered additive commits | `f2ebb32c26629cadca6a8c0967a212c1a91ff688`, `f0ecd25ed18005d3f50b54dd254b7d05e678afb3`, `467b0c27715e424ae18ffb5664da053b55915e2d` |
| Additive count / merge count | `3` / `0` |
| Binary full-index/no-renames patch | `91,788` bytes; SHA-256 `a71ea9276361cd426c0eecbf06c900034b5129849df281c8b8f881ffa0f63dc1` |
| Detached candidate scratch | clean; `HEAD (no branch)` |
| Detached predecessor scratch | `76baff84cb1192e11b26dcc669fb800f7b13fa30`; clean |
| Detached mutation scratch | restored to candidate bytes after every mutation; clean |
| Original implementer worktree | HEAD/tree exactly candidate values; branch `claude-agent/cfr168-python-http-upload-zero-residual-repair-20260730-upload-zero-residual-repair`; clean |

Additive history:

| Commit | Tree | Subject |
|---|---|---|
| `f2ebb32c26629cadca6a8c0967a212c1a91ff688` | `8c37630318ab3320c55dcf952fa4ea91edf2f7f1` | `wip(http-upload): close the three product findings of the zero-residual rereview` |
| `f0ecd25ed18005d3f50b54dd254b7d05e678afb3` | `df4cae6ad3a69085a16cf1bdff352b1ce686763d` | `test(http-upload): bind the review-closure repair with non-vacuous oracles` |
| `467b0c27715e424ae18ffb5664da053b55915e2d` | `3b129b9e41ae45defe4d6e7c43da04b016f07404` | `docs(cfr168): record the HTTP upload review-closure repair result` |

Exact changed paths:

| Status | Mode | Path | Added / deleted | Base blob | Head blob | Head bytes | Head SHA-256 |
|---|---|---|---:|---|---|---:|---|
| `M` | `100755` | `codefire` | `156 / 29` | `cc2c52aff4a11ebd80b2f22e37482b1700c0d738` | `e72b263b2717c45e84087ce0094f04cc2c74c2ca` | `1,402,507` | `8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93` |
| `A` | `100644` | `docs/development/ai/swarm-results/cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result.md` | `277 / 0` | — | `7100966bf76b1ada44d797e8787eb8377a18f6fd` | `16,259` | `a36149570fafcb7e38cc838738796b8e2b8c49095bab2325e1b136a3e31cb361` |
| `M` | `100644` | `docs/development/change-playbook.md` | `16 / 0` | `fcb1514713644ae346e367358220852c64280725` | `727fb6d81a28d7078e50ff9602be2821c3cd87dd` | `36,636` | `792fc05bebdc4621ea5092c3724a349ed5313956e8296a647b34cd5bebec6a65` |
| `M` | `100644` | `docs/development/command-source-trace.md` | `6 / 2` | `847d19441cac15bc89217a002d9cab67891555ec` | `6814208991d837d6992535576e44fe4e249978fb` | `40,678` | `986984327245a060d00fafcf9a12a85eb070a7bc8cc80685de12d0c1cbcec4e4` |
| `M` | `100644` | `docs/development/source-architecture.md` | `30 / 5` | `6a4f8d4afb5312d18029ef0754fcedc16b6bab77` | `fc707bd2aee8ce88b864a658383fbf27a53405af` | `103,662` | `a1e2f19fc13cf8985f1d03390ad8d6f7d8e7573a0677b5cb2d121e3d071e0e6e` |
| `M` | `100644` | `docs/development/source-data-model-catalog.md` | `2 / 2` | `ea357917bfa18e13e3a58fb3c4ce0092cb8a0241` | `619a70e7854ea14fd0942db3b4b2043eb8db113d` | `93,150` | `5ccab09f3c1f6a6ae84f3a7c6dc6b837fa5b2f699968dae6ba2457040606b9c0` |
| `M` | `100644` | `tests/test_python_http_upload_accounted_traversal.py` | `806 / 2` | `ceeeebe8be2b581b2ba216dc349ced12624ad4c3` | `010727908196179cb08c059e03e3cb54ddf7834c` | `143,181` | `c58135d4fb9341c39fe4ceaf335dab41db97b45e49ce512c30e6b3cdca7cda90` |

Evidence-chain binding:

| Evidence | Commit / blob | Bytes | SHA-256 |
|---|---|---:|---|
| Rejected predecessor | `76baff84cb1192e11b26dcc669fb800f7b13fa30`; tree `fcc65a4729f5029556e0266971d94b6321421f09` | — | — |
| Prior independent-review commit | `35983261e1abd32045e1ff6cbd7dd7dfff5e806d` | — | — |
| Prior report in candidate lineage | commit `fe7cf94388393bd9b0457564d0116406f2c8844e`; blob `1fd3de9b48e04d29b56f36d79584a0a03c22bcff` | `61,539` | `7632552eb0ca60addca0d83fe3ba53ead642a7cbee82937b1b7b37754f6cb2cf` |
| Continuation packet | commit `b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b`; blob `b7031007c17fcb78ea94491458a622b9841930f2` | `16,509` | `9c46f5585e1e83748651ced1e3384db4a4eb2201c966032de9211f79bc8ef64e` |
| Implementer result | candidate commit; blob `7100966bf76b1ada44d797e8787eb8377a18f6fd` | `16,259` | `a36149570fafcb7e38cc838738796b8e2b8c49095bab2325e1b136a3e31cb361` |
| Manager candidate card | blob `0b782797af9b311b2a21e97073340836fcddd056` | — | `b4a69230cd52d953bd177d9446f8429c7bbb60dc83598a3c2c2e514cb097e048` |

The original/resumed implementer session is
`7baa1275-0ee2-476a-af46-1907386ebceb`; provider and gate processes were
terminal before this review. No identity mismatch was found.

## BLOCKER-1 — CLOSED

### Production Route and Ownership Timeline

The live dispatcher constructs one request `DecisionWorkLedger` and one
`HTTPRequestDeadline`, then passes both at `codefire:15482`-`15490`:

```text
HTTP dispatch
  -> handle_http_upload(decision_work=request ledger,
                        deadline=request deadline)
  -> execute_remote_json_publication(ledger=request ledger,
                                     deadline=request deadline)
  -> recover_remote_pending_quarantine(ledger=request ledger,
                                       deadline=request deadline)
  -> poll("upload recovery sealed validation")
  -> validate_sealed_commit(...,
                            ledger=request ledger,
                            deadline=request deadline,
                            phase="upload recovery sealed validation")
  -> poll("upload recovery receipt")
  -> receipt replay and effect equality
  -> validate transaction-directory absence
  -> poll("upload recovery cleanup")
  -> quarantine cleanup / unlink / fsync
  -> recovered result becomes response authority
```

The exact handoff is at `codefire:13213`-`13229`,
`codefire:3777`-`3785`, and `codefire:2911`-`3031`. Each recovered root is
validated on the request ledger/deadline. The only preserved no-budget branch
is the explicit `ledger is None and deadline is None` call at
`codefire:3020`-`3023`; its production caller is remote GC at
`codefire:17214`, not HTTP upload dispatch.

The receipt poll is immediately before receipt read/replay at
`codefire:3088`-`3106`. The last cleanup poll is at `codefire:3116`; the next
operation is the first cleanup effect. No callback-capable or
input-proportional work intervenes between that poll and the effect sequence.

### Independent Counterexamples

The reviewer-authored production probe built a genuine completed quarantine,
advanced a monotonic clock from `0.0` to `1.0` seconds at recovery against a
`100` ms deadline, and drove `handle_http_upload`.

| Subject | Outcome | Fresh ledger minted | Durable snapshot | Quarantine | Ledger terminal state |
|---|---|---|---|---|---|
| Candidate | typed `RequestDeadlineExceeded` during `upload recovery sealed validation` | none | objects, branches, generation, nonce, pending marker, receipts, audit and quarantine byte-identical | preserved | zero retained/temporary debt; zero permits |
| Rejected predecessor | returned expired `200` | `sealed validation` / `local_graph` | changed | deleted | request ledger itself zero, but deadline never became authoritative |

Candidate live recovery then returned exact `200` with
`{"branch":"main","head":tip}`, removed only the intended quarantine,
validated the staged and recovered root once each on the request ledger,
left zero debt/permits, and replayed byte-identically.

Two independent reversals bind both required repairs:

- Removing only the `ledger`/`deadline` handoff (`m7`) makes the focused
  recovery oracle fail because `RequestDeadlineExceeded` is not raised. The
  direct handoff mutation reproduces the predecessor's fresh-ledger expired
  `200` and quarantine deletion.
- Removing only `poll("upload recovery cleanup")` in scratch leaves all
  other recovery code intact. Expiry injected after transaction-directory
  validation then returns `200`, deletes the quarantine and changes the durable
  snapshot. Candidate baseline instead raises during `upload recovery cleanup`
  with the entire snapshot unchanged. The mutation was restored to
  `codefire` SHA-256
  `8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93`.

Ordinary upload remains one request ledger, one deadline, one prepared sealed
snapshot, one proof consumption, one terminal response selection, and no
foreign validation session (`test_one_upload_owns_every_sealed_validation_ledger`,
the focused 29-test suite, and the live recovery probe).

## HIGH-2 — OPEN

### Nominal Repair and Exact Formula

The ordered projection is computed at `codefire:22564`-`22567` from runtime
collection footprint and actual document count:

```text
ordered_projection_bytes(N) = sys.getsizeof([]) + 8 * N
                            = 56 + 8N on this CPython runtime
```

The candidate calls `_acquire_control("temporary_bytes", projection_bytes,
"snapshot_revalidation")` before `list(self.documents)` and the in-place
heapsort. Exact observations are `56` bytes for zero documents, `64` for one,
and `272` for 27.

A reviewer-authored 407-document closure measured projection `3,312` bytes
and terminal temporary peak `5,318` bytes:

| Capacity | Result | Diagnostic / terminal state |
|---|---|---|
| Projection minus one: `3,311` | refused | `temporary_bytes`, phase `snapshot_revalidation`, observed `3,312`; zero debt/permits |
| Exact projection claim: `3,312` | projection claim itself fits | later scratch raises the full peak to `5,318` |
| Peak minus one: `5,317` | refused | observed `5,318`; zero debt/permits |
| Exact peak: `5,318` | completes | exact peak `5,318`; zero debt/permits |

Nominal public-path testing is otherwise successful:

- `validate_sealed_commit`, `validate_remote_json_publication_identity`, and
  `preflight_remote_json_publication` complete with a caller ledger and with a
  caller ledger plus deadline.
- Empty projection, one-record, multi-record, identity and preflight closures
  construct exactly one `_AccountedUploadSourceMemory`; nominal completion
  closes it with zero claims and leaves zero ledger debt/permits.
- The no-ledger compatibility path still completes, and invalid actor syntax
  retains its original precedence.
- The rejected predecessor reproduces
  `upload commit source exceeded its prepared temporary_bytes reservation` on
  caller-owned identity/preflight routes.

### Decisive Release-Failure Defect

The public API is not usable under its required release-failure contract.
Three source transitions consume ownership before a fallible release:

1. `codefire:7152`-`7160`: `_release_control_memory` decrements
   `_control_claimed[dimension]` before `self._ledger.release`.
2. `codefire:7162`-`7176`: `close` sets `_closed = True` and
   `_acquired = False` before releasing the `11,456`-byte retained session
   control upper.
3. `codefire:22667`-`22673`: `_release_control_scalar` clears
   `_ordered_projection_bytes` before delegating the release.

Reviewer-authored pre-effect `ledger.release` fault injection produced:

| Path | Injected release | First outcome | Owner after failure | Ledger after owner retry | Ledger after API retry |
|---|---:|---|---|---:|---:|
| `validate_sealed_commit` | `11,456` retained | `RuntimeError` | closed, unacquired, claims `0/0` | `11,456` retained | `11,456` retained; API returns |
| `validate_remote_json_publication_identity` | `11,456` retained | `RuntimeError` | closed, unacquired, claims `0/0` | `11,456` retained | `11,456` retained; API returns |
| `preflight_remote_json_publication` | `11,456` retained | `RuntimeError` | closed, unacquired, claims `0/0` | `11,456` retained | `11,456` retained; API returns |
| ordered-projection cleanup | `432` temporary | `RuntimeError` | closed, unacquired, claims `0/0`, peak temporary `432` | `432` temporary | `432` temporary; API returns |

All four cases have zero open permits and `terminal_reason = None`. The leak is
therefore silent, permanent on that caller ledger, and not recoverable by the
owner that originally held the claim. A later successful API call cannot know
that the earlier bytes still belong to it.

This violates the ticket's explicit constructor/allocation/sort/deadline/
cancellation/publication/**release-failure**/retry requirement and the more
general invariant that owner state is never erased before successful release.

Minimal closure:

- make scalar, exact-dimension, and retained-control release transitions
  retryable and exactly once;
- preserve the claim/amount and open owner state when `ledger.release` fails
  before effect;
- if a release can fail after changing ledger state, compare before/after and
  commit ownership only when the ledger change occurred, as the candidate's
  `_release_pair_claim` already does for internal claims;
- add direct pre-effect and post-effect release-failure regressions for all
  three public APIs and ordered-projection cleanup, asserting that owner retry
  or API cleanup reaches zero debt/permits without double release.

## HIGH-3 — CLOSED

`RemoteJSONPublicationRootProof` now has the explicit state machine required
by the prior review:

```text
available
  -> consuming                 (before the callback-capable deadline poll)
  -> consumed                  (before roots are returned)
  -> revoked                   (owner final fence or cleanup)

consuming -- refused check --> available
any state -- revoke ---------> revoked
```

At `codefire:3459`-`3495`, non-available use is rejected before the state is
claimed; deadline, foreign-store and uncovered-root failures restore
`available` unless a callback revoked the proof. One successful use increments
`consumptions` once. `revoke` is terminal and idempotent.

Independent direct results:

| Case | Candidate | Rejected predecessor |
|---|---|---|
| First use | roots returned; `consumed`; count `1` | roots returned; no state |
| Ordinary second use / alias | `CFError: ... already consumed`; count remains `1` | accepted |
| Callback reentry during deadline poll | inner call gets `already consuming`; outer succeeds; count `1` | inner accepted; total count `2` |
| Foreign store / uncovered root | typed error, remains retryable `available` | typed error; no state protocol |
| Expired deadline | typed deadline, remains `available`, count `0` | typed deadline; no state protocol |
| Revoke before/after use and repeated revoke | terminal `revoked`; no additional consumption | only legacy revoked bit behavior |

The focused mutation removing only the non-available guard (`m8`) accepts
ordinary second and alias use. Direct mutation capture returns roots twice
with `consumptions = 2`; callback capture accepts the reentrant call before the
outer call detects the changed state. The exact rejected predecessor also
accepts both reuse and reentry.

Proof bytes are claimed before allocation at `codefire:23783`-`23797`.
Normal finalization revokes the proof and releases its claim at
`codefire:24169`-`24177`; abort/owner cleanup performs the same terminal revoke
and release at `codefire:24639`-`24647`. `_release_pair_claim` clears its scalar
only after the internal claim release succeeds. Owner cleanup remains
retryable until `_closed` is committed, and `__del__` calls the same idempotent
path. The live focused case mints one proof, consumes it once, finishes in
`revoked`, traverses once for the prepared snapshot, preserves publication
ordering/wire bytes, and closes at zero debt/permits.

Operation, actor, idempotency-key and stale-identity mismatches are enforced
by the surrounding publication identity/preflight objects before effect; the
selected publication-precedence and generic-publication cases remain green.

## MEDIUM-4 — OPEN

### Nine Required Mutations

Each mutation was applied alone to a clean detached candidate export. Baseline
`codefire` SHA-256 was
`8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93`;
that exact SHA was restored before the next row. Commands used the named
single focused unittest under `PYTHONDONTWRITEBYTECODE=1`.

| Row | Exact edit | Expected assertion | Observed result | Duration | Mutated / patch / log SHA-256 |
|---|---|---|---|---:|---|
| `m1` | remove ordered-projection `_acquire_control`; keep list allocation/sort | refuse before unowned projection allocation | exit `1`; line `3002` observes ledger temporary `0 < 3,312` **after** `list(self.documents)` | `2.777665347s` | `0cb5a33a...` / `09146039...` / `0b2720a9...` |
| `m2` | make `_assert_prepared_coverage` a no-op | injected debt/peak/no-use conditions must be rejected | exit `1`; all three expected `CFError`s absent; healthy tail also differs because prior subtests mutated authority | `2.137856633s` | `7c42f873...` / `55bc0984...` / `9e6041da...` |
| `m3` | remove debt predicate only | retained-debt injection must fail pre-effect | exit `1`; retained-debt subtest reports `CFError not raised` | `1.780841637s` | `e93cf96e...` / `fe5481f9...` / `6d954150...` |
| `m4` | remove peak-versus-upper predicate only | peak-over-upper injection must fail pre-effect | exit `1`; peak predicate does not raise and later durable snapshot changes | `1.680546260s` | `0d273cdf...` / `cf29f427...` / `f6cd8bff...` |
| `m5` | remove pool-was-used predicate only | no-covered-use injection must fail | exit `1`; `CFError not raised` for no covered use | `1.883288535s` | `fba9fd9f...` / `9945b142...` / `70dac02d...` |
| `m6` | remove only `upload layout bootstrap` poll | elapsed request must be stopped before durable layout | exit `1`; expiry shifts to later `publication preflight` instead of layout fence, proving the layout fence was removed | `1.634401843s` | `b8925a36...` / `a99588f2...` / `6ac52737...` |
| `m7` | drop recovery ledger/deadline handoff | expired completed-quarantine retry must raise typed deadline | exit `1`; `RequestDeadlineExceeded not raised` | `1.668404112s` | `95b0caaa...` / `536499d8...` / `94332e80...` |
| `m8` | remove proof non-available reuse guard | second/alias/reentrant use must be rejected | exit `1`; second and alias accepted; callback state error; direct captures prove two successful consumptions | `1.549500831s` | `a446ea54...` / `73f4229c...` / `d787a2f7...` |
| `m9` | remove source owner's exact temporary dimension | refuse before unowned projection allocation | exit `1`; line `3002` observes ledger temporary `0 < 3,312` **after** `list(self.documents)` | `2.493328717s` | `be125ed0...` / `4cc009d4...` / `00975831...` |

Exact commands:

```bash
# m1, m9
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_ordered_projection_claim_is_exact_and_precedes_its_allocation -v

# m2, m3, m4, m5
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_prepared_coverage_predicates_are_load_bearing -v

# m6
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_removing_the_layout_bootstrap_poll_creates_the_layout -v

# m7
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_completed_quarantine_recovery_owns_the_request_budget -v

# m8
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_publication_root_proof_is_atomically_single_use -v
```

Exact one-at-a-time patch hunks:

```diff
# m1 — ordered projection claim removal
-            self._acquire_control(
-                "temporary_bytes", projection_bytes, "snapshot_revalidation"
-            )
-            self._ordered_projection_bytes = projection_bytes
+            self._ordered_projection_bytes = 0
             ordered_object_ids = list(self.documents)

# m2 — complete prepared-coverage no-op
     def _assert_prepared_coverage(self) -> None:
+        return
         for dimension, upper in (

# m3 — debt predicate removal
-            if self._claimed_memory[dimension]:
-                raise CFError(
-                    "HTTP upload sealed validation retains "
-                    f"{dimension} debt at its final fence"
-                )

# m4 — peak-versus-upper predicate removal
-            if self._claimed_memory_peak[dimension] > upper:
-                raise CFError(
-                    "HTTP upload sealed validation peak exceeded its prepared "
-                    f"{dimension}"
-                )

# m5 — pool-was-used predicate removal
-        if not self._claimed_memory_peak["retained_bytes"]:
-            raise CFError(
-                "HTTP upload sealed validation claimed no covered retained bytes"
-            )

# m6 — layout-bootstrap poll removal
-            fence_first_effect("upload layout bootstrap")
+            pass  # mutation m6 removes layout bootstrap poll

# m7 — recovery handoff removal
-        ledger=ledger,
-        deadline=deadline,

# m8 — publication-root proof reuse guard removal
-        if state != self.AVAILABLE:
-            raise CFError(
-                f"remote publication root proof is already {state}"
-            )

# m9 — source temporary dimension removal
-    EXACT_DIMENSIONS = frozenset({"temporary_bytes"})
+    EXACT_DIMENSIONS = frozenset()
```

Full exact hashes:

| Row | Mutated SHA-256 | Patch SHA-256 | Log SHA-256 | Restored SHA-256 |
|---|---|---|---|---|
| `m1` | `0cb5a33a9faef2bc239589711185ed9ea600f64ba8966bc458bef36a093fd2ce` | `091460391f0e6b958e15d6bb7b52c41f9a01c9013a5341441ad8e301bcd095a0` | `0b2720a931d1b59db3ac0902fb67237a9c1030261442c0ec5b2862c2caa4d853` | `8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93` |
| `m2` | `7c42f87389b27cdfc6a2e319fd20b2099c9472113a9debc6b20f3a1163767656` | `55bc0984f27fca94f5c5913e242498fef3bcc9d6d03a2f62a028a9f4301116c6` | `9e6041daf8fac3e1d5bcf004414721629c97d56ee433e5c8254c89afc5be3c89` | `8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93` |
| `m3` | `e93cf96e6bff2520cf6bc43aabbdf7d3fdd8370bebe907ba864f20e69926e24e` | `fe5481f99035de8dc1ae9e592753dafdf8b113ec39c68a2754b884e470c656a6` | `6d9541508e0e08d58f58d8e1dc574728902da3d0505fa4425ef7e705bdb176fc` | `8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93` |
| `m4` | `0d273cdf383c82529bcb95310f19e91ba42e42d4259df7ba4dd48e3a28c3beb3` | `cf29f42796f8f96f35d2d7c2e39aae835dfc893af353cd363510bf2ed9103f5b` | `f6cd8bff8f906d2b8cbb53995f29966c4430a530a1dd211b02a228f758f24531` | `8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93` |
| `m5` | `fba9fd9fd1286e050c93da71506c3e083014d118af5341b6a63404cd3e78271e` | `9945b1428dc5a020d7c68a440d811115be69401f919cf29d26f786ca1a2c3c2f` | `70dac02d5b6e36c4f9a8365161b5f83581e5eba07c8ddc765f1983603b63d62c` | `8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93` |
| `m6` | `b8925a365a0409a41adb5386438bd5578d63c59b2087253d2e5efd7648de190c` | `a99588f2576c13003c4b4580f753aeec755efe502023ba918e07188d4dcdcb3c` | `6ac5273767b24fce9bed4a3b770e1fd714dd3d4203b078d49a901ce30c928d51` | `8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93` |
| `m7` | `95b0caaa140a7944f195ab0fb58215363f14f15c811b2b2040f8e3cca0cfe95d` | `536499d8aabd3db03a7082079a3e6c6ac13a6a50eddda1ffd053dc722a171940` | `94332e8046e88eacbc32d848d0f9bd1fc1b3f0d0f481de4fd63eb4f801f71925` | `8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93` |
| `m8` | `a446ea546830e43e0ac3c47258a2530520b6b6caf4b5c82c8daa627896587856` | `73f4229c5ea1d6b4d7af9ad35488057888c1ab92c16d914a74c19b522920085d` | `d787a2f734fdc4b665174d9831834f1ccbe108409fede3597d2ee39951bf610d` | `8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93` |
| `m9` | `be125ed07a64b14be23e94b5f13afa3d510028c66532266d0ae9c23148e0c65b` | `4cc009d44dec7b3388b209f6bbccf4991a628ee78bf2c26eb12bd9879a1ecd90` | `00975831e495579ca31a04e447e9f74a885f96a85135d655bbb8c6a72cfe7305` | `8df9193e3700ce4d0e47f2c4ff8bef231c92d7b6a8d723f204ade9c7f1d0be93` |

### Why `m1` and `m9` Are Still Vacuous

Both edits leave `ordered_object_ids = list(self.documents)` at
`codefire:22572`. The focused test observes ownership only when
`_sort_strings_polling` is called on the already-created list. Its assertion at
`tests/test_python_http_upload_accounted_traversal.py:3002` therefore proves
that the sort sees no claim, but not that the list was prevented from being
allocated without an owner. The ticket explicitly rejects an oracle that
fails after the unowned allocation.

Minimal closure is to make the production allocation pass through a
charge-verified allocation boundary, or otherwise expose a production
precondition that is checked immediately before the `list` constructor. The
focused mutation must fail at that boundary before allocation, not at sort,
and must do so for both claim removal and loss of the source owner's temporary
dimension.

Supplemental `m10` removed only the final recovery cleanup poll. It returned
expired `200`, deleted the quarantine and changed the durable snapshot;
mutated/patch/log/restored SHA-256 values were
`e50f4690ea20842e55d0550907b964b157220ee9d5b5e174214e29421dc112d4`,
`70540c8683bd2187072919b82d6f187ffb2e3101134b4b5c0e82fa933e3ce525`,
`0777d4c78e1647844a6a519f96a2b4cce63b04d046a0fa0834a01539bef64368`,
and the candidate baseline respectively.

## Preserved Upload Properties

The exact candidate retained the previously accepted upload behavior in all
executed nominal gates. Deliberate release-failure debt is the exception
described under HIGH-2.

Owner-lifetime formulas remain:

```text
retained =
    64 KiB
  + BORROW_CONTROL_BYTES
  + SEALED_SESSION_CONTROL_UPPER
  + PUBLICATION_PROOF_BYTES
  + PUBLICATION_PROOF_ROOT_BYTES * 64
  + (RETAINED_RECORD_BYTES + BORROW_FINGERPRINT_BYTES) * N
  + 2 * aggregate_bytes
  + pair_plan_upper(N)

temporary =
    64 KiB
  + PAIR_PROBE_TEMPORARY_BASE_BYTES
  + pair_builder_temporary_upper(N)
  + ordered_projection_upper(N)
  + TEMPORARY_RECORD_BYTES * N
  + max(2M, stream_scratch_upper(M), M + stream_scratch_upper(M))

stream_scratch_upper(M) = 512 KiB + 3M
ordered_projection_upper(N) = 56 + 8N
```

The scratch maxima are alternatives and remain combined with `max`, not sum;
the retained formula has no `maximum_record_bytes` multiplier.

Default-profile scale rows reproduced by the focused suite:

| Factor | Depth | Records | Aggregate / max B | Result | Retained reservation / peak | Temporary reservation | Claimed retained peak | Nodes / edges / reads | Polls |
|---|---:|---:|---:|---|---:|---:|---:|---|---:|
| `1x` | `120` | `127` | `98,437 / 803` | exact `409` | `948,962` | `615,540` | `138,307` | `242 / 966 / 248` | `156,930` |
| `3x` | `360` | `367` | `291,157 / 803` | exact `409` | `2,567,042` | `650,100` | `400,387` | `722 / 2,886 / 728` | `465,758` |
| `10x` | `1,200` | `1,207` | `967,078 / 805` | exact `409` | `8,233,124` | `771,068` | `1,319,068` | `2,402 / 9,606 / 2,408` | `1,548,053` |

Every row leaves zero current retained/temporary debt, zero permits, unchanged
remote branch and absent divergent tip. The reviewer production probe also
completed a 1,201-commit fast-forward with exact `200`, then returned exact
`409` and
`upload rejected: remote branch is not an ancestor of local branch head` for
the divergent case.

Record and count behavior remains under the unchanged
`DecisionWorkProfile.http_v1()`:

- live records through `3 MiB + 1` are admitted; the independently recorded
  live message-field boundary remains `3,354,531` admitted and `3,354,532`
  refused by staging temporary ownership;
- the single-record formula fixture admits `M = 4,042,158` and refuses
  `4,042,159` at the exact `16,777,216` temporary limit;
- the `805` B/record formula fixture admits `N = 4,959` and refuses `4,960`
  at the retained limit;
- no public request, object-record, aggregate, graph, response or header limit
  changed.

Canonical streaming remains byte-identical for ASCII, CJK, emoji,
quote/backslash/newline/tab-heavy, control-character and mixed Unicode shapes.
The independent production probe returned `true` for all six named shapes.
Focused streaming tests retain bounded scratch and no whole-record duplicate.

Descriptor registry/reuse, shared-DAG and shape coverage remain green for wide
fan-in, deep chain, signed history and stacked diamonds. The established
registry peak remains `10`, reuse-pool peak `1`, with cleanup to zero; detached
document fingerprints are released by the owner suites.

Existing-project publication, missing-project layout, recovery sealed-root,
receipt and cleanup deadline fences all remain pre-effect in candidate
baseline. Selected legacy tests preserve exact authentication, signature,
nonce, idempotency, receipt, recovery, fast-forward and diagnostic precedence,
and exact `200`/`409`/error/retry bytes.

## Call-Site and Composition Residual Audit

AST enumeration found exactly **29** production
`validate_sealed_commit` occurrences:

| Enclosing function | Count | Lines | Ownership / shortest route |
|---|---:|---|---|
| `open_context` | `2` | `1808`, `1816` | local context commands |
| `recover_remote_pending_quarantine` | `2` | `3021`, `3025` | no-budget remote GC branch; accounted HTTP upload branch with `ledger`, `deadline`, `phase` |
| `validate_remote_json_publication_identity` | `2` | `3577`, `3579` | compatibility branch; caller-owned branch |
| `handle_http_list` | `1` | `12250` | separately owned HTTP list surface |
| `http_remote_branch` | `1` | `12343` | separately owned MR branch surface |
| `validate_http_merge_request_record` | `1` | `12371` | separately owned MR list/review/apply surface |
| `handle_http_upload` | `1` | `12895` | direct no-budget compatibility fallback; real HTTP dispatch supplies request owner |
| `validate_merge_request_record` | `1` | `21460` | non-HTTP shared validation |
| `resolve_commitish` | `2` | `33046`, `33049` | CLI/local resolution |
| `resolve_commitish_any` | `3` | `33067`, `33078`, `33084` | CLI local/remote resolution |
| `cmd_branch_list` | `1` | `33236` | CLI |
| `cmd_open` | `1` | `33245` | CLI |
| `cmd_clone` | `3` | `33323`, `33343`, `33362` | CLI local/remote clone |
| `cmd_upload` | `1` | `33382` | CLI upload prevalidation |
| `cmd_request_merge` | `2` | `33598`, `33599` | CLI MR creation |
| `cmd_request_list` | `1` | `33721` | CLI MR list |
| `cmd_request_apply` | `2` | `33984`, `33985` | CLI MR apply |
| `cmd_merge` | `2` | `34329`, `34330` | CLI merge |

Upload-owned recovery now uses the request ledger/deadline. The compatible
no-budget recovery call is reachable from remote GC only. The three separately
owned HTTP surfaces remain unchanged and still require their own tracked
repair; nominal caller-owned APIs are structurally reusable, but HIGH-2 means
their cleanup contract is not yet safe.

No second upload proof, traversal, ledger, response owner or effect path was
found around the repaired ordinary route. Deadline/accounting exceptions are
not swallowed into stale prepared success by the reviewed upload/recovery
arms. Nominal shared-primitives tests for MR merge/review/apply, remote GC,
recovery, signature and publication remain green.

There is exact textual and semantic overlap with the active MR-list candidate
in monolithic `codefire`: the signatures and call inventory around
`execute_remote_json_publication`, `recover_remote_pending_quarantine`, and
`validate_sealed_commit`, plus the four canonical source-contract documents.
This decision is upload-scoped. It cannot substitute for Python composition
review or authorize conflict resolution/integration.

## Validation Evidence

All Python commands used `PYTHONDONTWRITEBYTECODE=1`, reviewer-specific
`/tmp/cfr168-rereview-f8838c44a6ff-20260731` paths, and unique logs. The
wrapper uses `time.perf_counter_ns()` for duration; wrapper SHA-256 is
`8af35fd03a3de538a6d5a9d3fe50d3b61889fabae31ecd0fe248b69b3cb16296`.

Runtime/toolchain:

```text
Linux 6.6.87.2-microsoft-standard-WSL2 x86_64
Python 3.10.12 (/usr/bin/python3; GCC 11.4.0 build)
Git 2.34.1
candidate export HEAD 467b0c27715e424ae18ffb5664da053b55915e2d
```

Required gates:

| Gate | Exact result | Wrapper duration | Log bytes | Log SHA-256 |
|---|---|---:|---:|---|
| `python3 -m py_compile codefire tests/test_python_http_upload_accounted_traversal.py tests/test_python_sealed_production.py tests/test_codefire_cli.py` | PASS | `0.753166576s` | `0` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `python3 -m unittest tests.test_python_http_upload_accounted_traversal -v` | `29/29` PASS | `122.449485514s` | `5,872` | `44b726f3556301ef89644b0998396bf8fe887e2d03d94c296609295b2cc2a244` |
| five neighboring owner suites, `-v` | `59/59` PASS | `31.007462893s` | `8,053` | `5daa5f370091d13aa4314041111d9b353df17ea6b421ce9237ddb436875dfb9b` |
| `python3 -m unittest discover -s tests -p 'test_python_*.py'` | `450/450` PASS | `286.720903347s` | `552` | `c8a88f556eb831efc077cfadc127f268bc50c7031cc9560d74f364a863932dbc` |
| exact 13 selected legacy cases | `13/13` PASS | `168.487320024s` | `1,516` | `3bf4b346e227e416f2208235e5b670d910c45fe1e937c54e27fea11d2a8fbf83` |
| selected shared MR/GC/recovery/signature/publication cases | `14/14` PASS | `221.785864913s` | `1,608` | `bcd1b8b21c4f256f3275457a097f8a4d60aab68e10987ce825468fd9e6d4b62d` |
| `python3 docs/development/tools/check_source_docs.py` | PASS: 44 modules, 34 commands, 12 object kinds | `0.215793870s` | `68` | `a3679b77701e02f0697f91c6c53a362f7ab9d372ed700d042c1a73ec329cdb03` |
| `python3 docs/development/tools/check_project_docs.py` on candidate export | PASS: 531 Markdown files | `0.764880800s` | `59` | `f8c8f92f48a2fcdf073c6dead702eb1fd0a7e59fe3b40b463fbdaa4a40753954` |
| `git diff --check b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b..467b0c27715e424ae18ffb5664da053b55915e2d` | PASS | `0.029108074s` | `0` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |

Neighbor command:

```bash
python3 -m unittest \
  tests.test_python_ancestry_owner \
  tests.test_python_signature_owner \
  tests.test_python_signature_terminal_freeze \
  tests.test_python_sealed_production \
  tests.test_python_external_production -v
```

Exact selected legacy command:

```bash
python3 -m unittest \
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

Selected shared-primitives command:

```bash
python3 -m unittest \
  tests.test_codefire_cli.CodeFireCliTests.test_upload_list_remote_clone_and_request_merge \
  tests.test_codefire_cli.CodeFireCliTests.test_http_merge_request_escrow_survives_source_project_removal \
  tests.test_codefire_cli.CodeFireCliTests.test_request_list_and_review_reject_invalid_merge_request_refs \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_branch_protection_gates_upload_and_apply \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_gc_removes_unreachable_objects \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_gc_recovers_delete_before_progress_and_exact_audit \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_gc_blocks_on_invalid_sealed_refs \
  tests.test_codefire_cli.CodeFireCliTests.test_python_remote_publication_recovers_authoritative_staging_and_replays_receipt \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_requires_and_verifies_commit_signature \
  tests.test_codefire_cli.CodeFireCliTests.test_http_remote_requires_commit_signature_on_upload \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_request_signatures_gate_mutating_operations \
  tests.test_codefire_cli.CodeFireCliTests.test_python_generic_publication_preserves_identity_root_precedence \
  tests.test_codefire_cli.CodeFireCliTests.test_python_upload_publication_identity_rejects_before_effect \
  tests.test_codefire_cli.CodeFireCliTests.test_python_upload_publication_plan_rejects_before_effect -v
```

New regressions run individually:

| Regression | Result | Duration | Log SHA-256 |
|---|---|---:|---|
| completed-quarantine request ownership | PASS | `2.724445745s` | `07bade4881f5f0a3a90063e853c145b2bbac37e04d917169a68402fc3f732495` |
| recovery handoff mutation | PASS | `2.463989565s` | `70b09ebf3da7e6996bf2d1e314fb66842c94e68574a1791ead82e1757e4c5629` |
| caller-owned public APIs | PASS | `2.996113397s` | `7583a1106d18f52b547e4ce3071f0606f57447370332270b525281d633f0af47` |
| ordered-projection claim/boundaries | PASS | `5.231470582s` | `d57425c3f1f494b91a90cbe64bc8c688acd88402058d98918c82b553e8b862c9` |
| proof state protocol | PASS | `2.425026969s` | `372a0be36ecc3f5f9b8d6e4eefae26a088d724f1066eaf173a5920bf02849852` |
| prepared-coverage predicates | PASS | `2.550498334s` | `541aceb391abd031d800fcd84fcd897f07cc599bbbd94dd62dae2910aaf4f095` |
| layout-bootstrap poll | PASS | `2.381813454s` | `2c6c8e57e8040290bfc9840777b724a1950ccb145733b84339f3bcb9cb3b31e2` |

Reviewer-authored production probes:

| Probe | Subject/result | Duration | Log SHA-256 |
|---|---|---:|---|
| completed quarantine | candidate typed deadline + preserved snapshot; predecessor expired `200` | `1.605256872s` / `1.756997856s` | `926c40b8de4652eeb177b6813a29623acf23617233b24ad384b0e342149307b2` / `1feb73f543f13c7980abd5acfd718dea657e49d12e280dd41dcecf1912c5ea7b` |
| final cleanup poll | candidate typed cleanup deadline + preserved snapshot | `0.610574447s` | `2f07eba642750ae37f40cbd3706cbf03d124931da7abab64c105578bcc122714` |
| usable API nominal paths | candidate completes; predecessor temporary-reservation refusal | `1.418015705s` / `1.231480649s` | `ada0cc174b9124fd482bf62bef9ae14e356d3c410ebd1bc416063335b46454ff` / `7bd5a5281e85d21fe0fa547b4efc6ce9aa59309ae11ff526303c91e94007621d` |
| projection exact/minus-one | candidate `407` documents, `3,312` projection, `5,318` peak | `1.304325478s` | `508d990670120111b69ac5fd5fea54c37ed8d7ae2fd53e17f1ed080e5e1f85a2` |
| proof direct state | candidate single-use; predecessor reusable | `0.338685308s` / `0.372623172s` | `4fa6602a5cedf55bac7e599a13de27a2566196f52472d23e863bd32eb4546968` / `5bafb0469f628c5244b3c8b4f338c47ee532ec4d6bee374b1a301aa61392445f` |
| proof callback reentry | candidate rejects; predecessor accepts | `0.347424503s` / `0.388102760s` | `e5480236c8b7f6574356fc2e0a7b75dd6353816b62635470e841df60988331de` / `7b6049fc9ab377d2748f814a9e2d0a343022c866a348386249a3f48bee50e853` |
| preserved 1,201/canonical behavior | exact results described above | `9.804337835s` | `4b3ec0a7a6013b18143e9015ee4eb0099b68dac5c4d28e1d4b69cca183c6464b` |
| retained release failure | all three APIs leak `11,456` | `1.562986416s` | `3e823e8b0903cd718832500f6b41d055637190f14645e09a79872ac52a3496db` |
| temporary release failure | projection cleanup leaks `432` | `1.582434314s` | `5e5e4aca97577f49ad1e7581bc4d0ff437e2bbaaf0b840a9b8c907a06f4e76db` |

The direct `m8` captures additionally logged a second successful consumption
(`3f95c4eb5b44df4f7bc4ea9572b29163d573d9f25097b8c203d9cf5248ca95cc`)
and accepted callback reentry before outer failure
(`d0fcf3a2aa022201eff77b99a23584d5a8500d02e7ce5110073387015ae907d2`).

## Skipped and Pending Gates

Per the authoritative lane boundary, Cargo/Rust, Docker, Install, Dogfood,
`./demo.sh`, unrestricted all-language discovery, repository-wide integration,
and Python composition were not run. They remain manager-owned and pending or
unavailable in this lane. No toolchain was installed and the environment was
not mutated.

Independent raw-wire 16 MiB request-body and 64 KiB header exact/±1 fixtures
were not re-authored here; the executed broad/legacy owner suites are the only
evidence cited for those unchanged public limits. The Python-pattern sweep is
not evidence for Cargo, Install, Dogfood, composition, or release readiness.

## Strongest Concerns and Required Next Action

The strongest correctness concern is not nominal budget sizing but failure
atomicity: comments explicitly justify erasing owner state before a fallible
release to avoid double release, but that ordering makes pre-effect release
failure unrecoverable. The implementation needs an explicit release state
transition, not optimistic state destruction.

The strongest maintainability concern is that the ordered-projection test
uses a sort callback as a proxy for a prior allocation. It looks load-bearing
because mutations are red, while still allowing the exact unowned allocation
the test is intended to prevent. The allocation/claim boundary should become
one named production operation with a directly testable precondition.

Manager next action: return exact candidate
`467b0c27715e424ae18ffb5664da053b55915e2d` for a minimal additive repair of
HIGH-2 and MEDIUM-4, then create a new immutable candidate and request another
fresh independent review. Do **not** integrate this candidate.

## Verdict

`REQUEST_CHANGES`
