# CFR-168 / CFB-138 Python HTTP Upload — Third Repair Independent Rereview

Captured: 2026-07-31

Reviewer: Claude Opus 5 (`claude-opus-5`), independent report-only lane. No
subagent, no Codex, no Fugu, no Cargo/Rust, no install or dogfood step was used.

Reviewed subject: exact immutable candidate
`446cf4a97b63e46d011a03083ef59354b9d68178`.

Exact predecessor: `02e04c7e6458035d9c7d58de4d30b4a12e7cd594`.

## Decision

**`REQUEST_CHANGES`**

`BLOCKER-1`, `HIGH-3` and `MEDIUM-4` remain closed. `HIGH-2` is closed on two of
its three limbs — the direct wrapper is genuinely retryable and exactly once,
and the constructor unwind is closed *better* than the report asked, with a
trigger that needs no injection at all (an absent or unreadable object store).

Acceptance is nevertheless refused, on one decisive ground:

> On the two **live** HTTP upload routes, at the point **production itself
> reads the request ledger**, the candidate is indistinguishable from the
> predecessor. A real HTTP request carrying one real `close(2)` failure is
> observed by `make_http_handler`'s own request-loop `finally` as
> `retained_bytes = 11,456`, `open_permit_count = 0`,
> `terminal_reason = None` — the exact triple the prior report used to hold
> `HIGH-2` open — on `02e04c7` **and** on `446cf4a`.

The repair does drain the ledger, but only after the raised `OSError` is
released, which happens after the request has already been accounted, the
connection force-closed, and `terminate` skipped. Production never retries the
wrapper, and the finalizer cannot run while the propagating exception's
traceback pins the source. The oracle offered as closure evidence for this limb
passes only because `unittest.assertRaises` stores `exc.with_traceback(None)`;
with the traceback retained — production's state at its own checkpoint — the
same production route yields `11,456` on the candidate.

Literal prior-finding dispositions:

| Finding | Prior disposition (`b3de0d7`) | This review |
|---|---|---|
| `BLOCKER-1` — completed-quarantine recovery request ownership | CLOSED | **CLOSED (preserved)** |
| `HIGH-2` — commit-source cleanup ownership / release-failure contract | OPEN | **OPEN** (live-route limb only; direct-wrapper and constructor-unwind limbs CLOSED) |
| `HIGH-3` — atomically single-use publication-root proof | CLOSED | **CLOSED (preserved)** |
| `MEDIUM-4` — non-vacuous pre-allocation reversal matrix | CLOSED | **CLOSED (preserved)** |

No product, test, canonical source-document, packet, card, existing report,
issue, ledger or lifecycle byte was modified by this review. Every mutation was
applied to a detached export under `/tmp/rev3` and never to the worktree; every
probe lived under `/tmp/rev3/probes`. The only repository write is this report.

## Immutable Subject and Provenance

| Field | Recomputed value |
|---|---|
| Candidate HEAD | `446cf4a97b63e46d011a03083ef59354b9d68178` |
| Candidate tree | `3ac8fa8bc5aa38a1f137fd523db8285980d69692` |
| Predecessor HEAD | `02e04c7e6458035d9c7d58de4d30b4a12e7cd594` |
| Predecessor `codefire` SHA-256 | `63a9be232d064afa022432537f0686e57d47845b1f5654b929a69acec7a872fa` (matches the digest the prior report recorded) |
| Additive commits over the predecessor | `2` (`a531494` product+tests, `446cf4a` docs); `0` merges |
| Worktree at review start / end | clean |
| Interpreter | CPython `3.10.12` |

Additive history:

| Commit | Tree | Subject |
|---|---|---|
| `a531494be9b3ee5c6eef550521dc6f2d5b3176f7` | `5cdae973cf427e2a01cbd4d5d8564b20b3a4bc24` | `fix(http-upload): keep commit-source cleanup handles until cleanup succeeds` |
| `446cf4a97b63e46d011a03083ef59354b9d68178` | `3ac8fa8bc5aa38a1f137fd523db8285980d69692` | `docs(cfr168): record the HIGH-2 third repair and correct two over-claims` |

Exact changed paths, `02e04c7` → `446cf4a` (`398` insertions, `23` deletions):

| Status | Path | +/− | Head blob | Head bytes | Head SHA-256 |
|---|---|---:|---|---:|---|
| `M` | `codefire` | `64 / 11` | `9bdf5aa8bc5d57024e23351729f04ab0b6898ec8` | `1,411,412` | `d0a84dab25828bedba8fcb745ad09c73cb55ec604de85a0f3275b8137c7f6ca8` |
| `M` | `tests/test_python_http_upload_accounted_traversal.py` | `206 / 0` | `d9f5bf96ac6c0737162aa56bf38fc7565b74f127` | `163,304` | `b917d8f0f96936bea0095b2bcfbe21b411a3f24afd2fffad60c6923acbafb819` |
| `M` | `docs/development/source-architecture.md` | `16 / 11` | `0b80e9f3b25f9378df470f4f7f4761912de40541` | `105,519` | `c0f72526866fd4b3bc74a1bd54020467a391a42ab613be48439055b881723c56` |
| `M` | `docs/development/command-source-trace.md` | `4 / 1` | `7970a91952e21a55c525ac93b241f731c1fe678e` | `41,067` | `96cf248ae775fb6da3600a156b477a0bd66494310bf675c5647cb75ed0e6d5ae` |
| `A` | `.../cfr168-python-http-upload-high2-third-repair-20260731/result.md` | `108 / 0` | `7cfa8bbe441b243e28fb8340fa249a9ed53a5005` | `6,814` | `65eca17968d0b5014465dfb95c1454d5d15a51b31ffc65b2c88364e769797287` |

Immutable inputs actually read, by digest:

| Input | Blob | Bytes | SHA-256 |
|---|---|---:|---|
| `b3de0d7:…second-review-closure-rereview-claude-fallback-20260731/review.md` | `7f21d42d69a10eaa6b33ba1872a1da6a9a947e48` | `25,480` | `3df1db92d6adcef9d318e23182fb9c62519f6eacb091266ced9616626f64cf1f` |
| `abfee6e:…cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md` | `783a3def680140076a36c74d6b8659355cbf9fbd` | — | `2392450e957c8437b323ded492fc7597dc77f5147b74f3056c97c0efd9329780` |
| `446cf4a:…cfr168-python-http-upload-high2-third-repair-20260731/result.md` | `7cfa8bbe441b243e28fb8340fa249a9ed53a5005` | `6,814` | `65eca17968d0b5014465dfb95c1454d5d15a51b31ffc65b2c88364e769797287` |

Reviewer probes (reproduce with `CF_BUILD=<tree>/codefire
PYTHONDONTWRITEBYTECODE=1 python3 <probe>`; each loads the exact build with
`runpy.run_path` and drives real production entry points against a real
`codefire init` store and a real HTTP project root):

| Probe | Purpose | SHA-256 |
|---|---|---|
| `h.py` | harness; real store, real ledger, real `close(2)`-shaped fault | `2170bcf40c87b2287d61b44287e3777c8582f23b9521eeeee05b555c243a8528` |
| `pA_direct.py` | direct wrapper: retry, drop, persistent fault | `3924e150a9031c74e8c874569e0cc4169b89ecc186e8102d81b592b540eb6593` |
| `pB_live.py` | both live routes via `handle_http_upload`, handles held | `9ff86b588458b201bf6c656d5ba045b62eca65e60eb27927af525539917d8d98` |
| `pC_server.py` | real socket request through `make_http_handler` | `cb959ee33792397fc7b16de54e606f11388033baad40299d2503a91325160993` |
| `pD_timing.py` | refcount vs cycle; exception liveness; captured request ledger | `c4211fd171aef184ebefc463a8e1c5d855428a17acadef5d7f330101e3d80abc` |
| `pE_newsurface.py` | diff-scoped hazards of the surface the candidate introduced | `458c543b76a6185763bd07314162c29848a97605570bf64c4c84be1b8400613c` |
| `pF_oracle.py` | why the checked-in live-route oracle passes | `63ca8e38da5863bd790346d1d60b47cfe548951071fb3bbd5658287e31547898` |
| `pG_public.py` | the caller-owned public APIs under a real `close(2)` fault | `4ee0f2aff73e493ea536869f898f7d90bbf38bdc5b1c082e5243bb7394ff3630` |
| `pH_realctor.py` | constructor unwind with **no** injection at all | `1d34ad4f35258737d1f64434e535f1ea6261b597ba89862eacc7bbb790f7d825` |
| `sweep.py` | AST sweep of the cleanup-owner defect shapes | `ed3ec370308cecc088ea2a4f4d0796e8aa0ff9d8121631769e25b9ebcc18cb93` |

## Method

Per audit `abfee6e` `I1`, the predecessor's behaviour was re-derived from the
`02e04c7` tree itself before any candidate claim was read as evidence: probes
`pA`, `pB`, `pC`, `pD`, `pE`, `pF`, `pG`, `pH` all run unchanged against both
trees, so every "fixed after" statement below is a *difference measured on two
builds*, not a candidate self-report. Per `I2`, every symbol and branch the
candidate introduces was reviewed as unreviewed surface in its own right
(`pE`), separately from inherited-finding closure. Per `I4`/`§12.3`, oracles
were judged as properties — does the assertion fire at the boundary it names
(`E3`), does deleting the production line change a result (`E5`), is the
reversal behavioural rather than symbol-absence (`E6`), is the subject
production-reachable (`E8`) — and not by whether the implementation's
descriptor or handle enumeration looks right.

### Predecessor behaviour, re-derived from `02e04c7`

One real `close(2)` failure — the descriptor *is* closed, the call still raises
`OSError(EIO)` — armed only while `_AccountedUploadCommitSource.close` is on the
stack. No `DecisionWorkLedger.release` is patched anywhere.
`SEALED_SESSION_CONTROL_UPPER = 11,456`.

```text
pA on 02e04c7
  first close() outcome    OSError: [Errno 5] injected close(2) failure
  handles (session, memory) (None, None)
  retry close() outcome    returned None            <- silent no-op
  residual after retry     {'retained_bytes': 11456}  permits=False terminal=None
  drop + gc.collect()      {'retained_bytes': 11456}  <- no finalizer
```

That reproduces the prior report's decisive evidence independently, on the
predecessor tree, with the reviewer's own harness. Every claim below is scored
against this baseline.

## HIGH-2 — OPEN

### Closed limb 1 — the direct wrapper (`pA`, `pE`)

Independently confirmed on the candidate. `close` drops each handle only once
that handle's own cleanup succeeded, and does not attempt the reservation while
the session teardown still owes control debt.

| Case (`pA`, `pE` on `446cf4a`) | Result |
|---|---|
| one-shot `close(2)` fault, then explicit retry | handles **kept**; retry returns `None`; residual `{}`; a third `close` releases nothing again |
| one-shot fault, no retry, source dropped + `gc.collect()` | residual `{}` |
| persistent `close(2)` fault, three retries | `OSError`, `OSError`, then `None` once no descriptor remains to close; residual `{}` |
| post-effect ledger fault (release commits, then raises) | fault propagates; residual `{}`; retries release nothing again — exactly once preserved |
| healthy close ×3 + finalizer | residual `{}` throughout |
| finalizer after `ledger.terminate("budget_exceeded")` | residual `{}`, terminal preserved (predecessor: `11,456`) |

Mutation `m2` — restoring the pre-repair `self._session = None; self._memory =
None` at the top of `close` — makes
`test_commit_source_cleanup_survives_a_failing_close_syscall` fail behaviourally
(`assertIsNotNone(source._session)`), so this limb has a real reversal (`E6`).

### Closed limb 2 — the constructor unwind, closed better than required (`pH`)

The checked-in regression reaches the unwind by monkeypatching a
`SealedCommitValidationSession` subclass that raises and an
`_AccountedUploadSourceMemory` subclass whose `close` raises. That subject is
injected. A **production-reachable** trigger exists and needs no injection:
`SealedCommitValidationSession.__init__` claims its whole control state on the
owner (`codefire:21779`) and then opens the object-store root; the `os.open`
failure arm raises `CFError("object store is unavailable: …")` *outside* the
block that would release that claim, so the owner reaches the unwind already in
control debt.

`pH`, an absent object store and a `chmod 000` object store, no patching at all:

| Case | `02e04c7` | `446cf4a` |
|---|---|---|
| `_AccountedUploadCommitSource.payload`, absent store | caller sees `CFError: upload commit source closed with retained_bytes control debt`; handle dropped; residual `11,456` permanently | caller sees `CFError: object store is unavailable: …`; handle kept; explicit retry drains to `{}`; drop+gc drains to `{}` |
| same, `chmod 000` store | same wrong diagnostic; residual `11,456` permanently | correct diagnostic; drop+gc drains to `{}` |
| `validate_sealed_commit`, absent store | wrong diagnostic; residual `11,456` | correct diagnostic; residual `{}` at the point of observation |

This is a genuine, production-reachable correctness gain on both routes: the
predecessor replaced a real store-unavailability diagnostic with an internal
accounting error *and* leaked. Limb 2 is closed.

### Open limb 3 — the live routes, at production's own checkpoint (`pB`, `pC`, `pD`, `pF`)

`_AccountedUploadCommitSource` is built at `codefire:12040`
(`is_ancestor_from_objects_accounted`, reached from `handle_http_upload` at
`codefire:13173`) and at `codefire:7394`
(`enforce_remote_commit_signature_policy_accounted`, reached at
`codefire:13056`). Neither ever calls `close` a second time. So
on the live routes the new retryability is never exercised, and the only
recovery is the finalizer — which cannot run while the raised `OSError` is being
propagated, because its traceback holds the traversal frame that holds the
source.

`pB`, direct `handle_http_upload`, with every source the request constructs kept
strongly referenced (the state production is in while the error unwinds):

| Route | Build | Residual with sources referenced | After drop + `gc.collect()` |
|---|---|---|---|
| fast-forward ancestry | `02e04c7` | `{'retained_bytes': 11456}` | `{'retained_bytes': 11456}` |
| fast-forward ancestry | `446cf4a` | `{'retained_bytes': 11456}` | `{}` |
| commit-signature history | `02e04c7` | `{'retained_bytes': 11456}` | `{'retained_bytes': 11456}` |
| commit-signature history | `446cf4a` | `{'retained_bytes': 11456}` | `{}` |

`pC` / `pD3`, a **real socket request** to a real `make_http_handler` server,
read through `_decision_work_observer` — the same snapshot the request loop's
own `finally` (`codefire:15319`-`15346`) uses to decide `close_connection` and
whether to `terminate`:

```text
                                   02e04c7                446cf4a
close(2) faults fired              1                      1
HTTP response bytes                0                      0
PRODUCTION observation current     {'retained_bytes':11456} {'retained_bytes':11456}
PRODUCTION observation permits     0                      0
PRODUCTION terminal_reason         None                   None
request ledger read AFTER the run  {'retained_bytes':11456} {}
```

The candidate and the predecessor are **identical at the only point production
looks**. The ledger the candidate eventually drains is a per-request
`DecisionWorkLedger` minted in `handle_one_request` and discarded immediately
afterwards, so nothing in the build ever observes the drained state. The
production-visible consequences of the debt — `close_connection = True` and
`terminal_reason` left `None` instead of `complete` — are unchanged.

`pD1` pins the mechanism more precisely than the implementer's own note does.
With `gc.disable()`, dropping the last reference to the source does **not**
return the bytes; only `gc.collect()` does. The object graph is a cycle — the
raised `OSError` reaches `close`'s frame as `session_error`, and that frame is
in the exception's own traceback — so recovery depends on the *cyclic*
collector, not on refcounting. `_detach_exception_chain` clears `__context__`
and `__cause__`, not `__traceback__`.

`pF` isolates the single variable that decides the outcome, running the same
production route twice on the candidate:

```text
446cf4a, live handle_http_upload, one real close(2) failure
  exception stored with .with_traceback(None)  -> residual {}          <- what the test sees
  exception stored with its traceback          -> residual {'retained_bytes': 11456}
                                                  (and gc.collect() does not help)
02e04c7: 11,456 in both branches.
```

`unittest`'s `_AssertRaisesContext.__exit__` stores
`exc_value.with_traceback(None)`. The checked-in regression therefore always
takes the first branch; production always takes the second.

## Findings

Every item is classified, and states whether the reviewed diff created it.

### F1 — the live-route limb of `HIGH-2` is not closed at the production boundary

**Classification: reproduced production defect (inherited — the reviewed diff
did not create it, and did not close it). Blocking.**

One real `close(2)` failure inside the commit-source cleanup on either live
upload route leaves the request ledger holding `11,456` retained bytes with
`terminal_reason = None` at the request loop's own accounting checkpoint, on the
candidate exactly as on the predecessor (`pC`, `pD3`). A raw `OSError` — not a
typed HTTP status — still escapes `handle_http_upload`; no durable authority
byte changes and the branch does not move, so the blast radius remains the
request budget and the error surface. The prior report's closure item 4 required
"regressions on the two live commit-source routes … asserting that the request
ledger reaches zero debt and zero permits". It does not, where production reads
it.

The prior report's minimal closure did allow "an idempotent finalizer … in the
shape the other owners on this route already use" as an alternative to a public
retry surface. That allowance is honoured in form; it does not discharge the
requirement in substance, because on this route the finalizer is structurally
unable to run before the measurement point.

### F2 — the two new live-route assertions do not fire at the boundary they name

**Classification: claim/enforcement mismatch. Created by the reviewed diff.
Blocking (it is the evidence offered for F1).**

`test_commit_source_cleanup_survives_a_failing_close_syscall` asserts
`assert_closed(ledger)` after an `assertRaises` block that has already stripped
the traceback (`pF`). Two independent checks show the live-route half of that
oracle pins nothing about the repair:

- Mutation `m2` restores the pre-repair erase-before-release ordering in
  `_AccountedUploadCommitSource.close`. Both live-route subtests still **pass**;
  only the direct-wrapper assertion at line `3776` fails. The live-route
  assertions are satisfied by the finalizers alone — the pre-existing
  `SealedCommitValidationSession.__del__` clearing the control debt, then
  `_AccountedUploadSourceMemory.__del__` releasing — with the wrapper's handle
  retention, which is the actual repair, contributing nothing.
- The same production route, measured with the exception retained, yields
  `11,456` on the candidate.

This violates `E3` (fire at or before the boundary named — here "the live
request ledger") and `E8` (the observed state is not production-reachable at
that point).

### F3 — `_AccountedUploadCommitSource.__del__` is load-bearing but unpinned

**Classification: claim/enforcement mismatch (test-coverage gap). Created by the
reviewed diff. Non-blocking on its own.**

Mutation `m3` renames the new finalizer out of existence. The full 33-test
focused suite is **33/33 OK** and both new regressions pass. Yet on that same
build `pA`'s drop case regresses to `{'retained_bytes': 11456}` and `pB`'s
commit-signature history route regresses to `{'retained_bytes': 11456}` after
drop + `gc.collect()` — because without the source finalizer nothing closes the
session first, so `_AccountedUploadSourceMemory.close` refuses with control debt
and its own `__del__` swallows the refusal. A new production symbol that is the
only recovery on one live route has no oracle at all (`E5`).

### F4 — the corrected over-claims are replaced by a new one

**Classification: claim/enforcement mismatch. Created by the reviewed diff.
Non-blocking.**

The two corrections the prior report demanded are correct and verified: the
`_charged_allocation_boundary` claim is now scoped to its single caller (in
`source-architecture.md` and in the docstring at `codefire:21841`), and the
blanket "control releases are retryable and exactly once" is now qualified.

The replacement text over-claims in turn. `source-architecture.md`: "both owners
carry the idempotent finalizer … so a reservation nobody successfully closed is
still returned", and `command-source-trace.md`: "idempotent finalizerがどのclose
でも回収されなかった分を返す". Both are unconditional and both are falsified:

- by F1, at the production checkpoint on the live routes;
- by `pE2`, where a persistently failing session teardown leaves `12,249`
  retained bytes that neither retry nor either finalizer ever returns, because
  `_AccountedUploadSourceMemory.close` refuses with control debt and `__del__`
  swallows the refusal. Identical on the predecessor, so the *behaviour* is
  inherited; the *claim* is new.

### F5 — remaining-limitation 3 understates the mechanism

**Classification: claim/enforcement mismatch. Created by the reviewed diff
(report text). Non-blocking.**

`result.md`: "The finalizer's timing is CPython refcount behaviour, not a
guarantee of the language." `pD1` shows refcount alone does not recover the
reservation at all — the source sits in a cycle through the raised exception's
traceback, so the *cyclic* collector is required; and while the exception is
retained, as it is throughout the production error path, no collection recovers
it. The stated limitation is weaker than the real one, and the real one is what
makes F1 decisive.

### F6 — remaining-limitation 2 understates persistence

**Classification: routable risk (inherited; the reviewed diff did not create the
behaviour). Non-blocking.**

`result.md`: "Recovery after a *persistently* failing cleanup depends on a later
retry or on the finalizer." `pE2` shows neither works in that case: three
retries and a finalizer all leave `12,249` retained bytes, on both trees. The
honest statement is that a persistently failing session teardown has no recovery
at all, which is arguably **requirement-unsatisfiable** while the structures the
reservation covers still exist — but it should be recorded as that, not as
covered by the finalizer.

### F7 — `_open` retry now orphans a live handle

**Classification: routable risk. The over-reservation is inherited; the
orphaning step is created by the reviewed diff. Net effect is an improvement.
Non-blocking.**

`pE1`: after a constructor failure whose cleanup also failed, a caller that
simply asks for another payload re-enters `_open`, which mints a **new**
`_AccountedUploadSourceMemory`, acquires again, and overwrites the retained
`self._memory`. The ledger then holds `22,912` retained bytes — twice
`SEALED_SESSION_CONTROL_UPPER` — and the first owner is reachable only through
the collector. The predecessor also reaches `22,912`, but permanently and with
the wrong error; the candidate drains to `{}` on `close()` plus collection. Worth
recording because the diff introduced the "retain a handle that a later `_open`
silently replaces" step, which is the same ownership hazard family the repair
was bound to.

### F8 — one recorded gate was not run on the exact candidate tree

**Classification: claim/enforcement mismatch (evidence provenance). Created by
the reviewed diff. Non-blocking.**

`result.md` records `check_project_docs.py` as "PASS — 532 Markdown files". The
predecessor tree has `532` tracked Markdown files; the candidate tree has `533`,
because `result.md` is itself one. The recorded run therefore predates the
committed tree. I re-ran it on the exact candidate: PASS, `533` files.

### F9 — `_control_charge_witness` aggregate read

**Classification: non-production-reachable. Not created by the reviewed diff.
Correctly deferred.**

Unchanged from the predecessor byte for byte. The prior report classified it
non-blocking and could not reach the weak configuration from production; the
implementer declared it as deliberately left open. Audit `abfee6e` §10 item 10
says not to promote a reviewer-labelled non-blocking residual into mandatory
closure work without a recorded manager decision. This review does not promote
it, and does not expand scope to it.

### Adjudication of the implementer's remaining-gap list

| Stated gap | This review |
|---|---|
| `_control_charge_witness` not addressed | **Accepted deferral** — F9; non-production-reachable, correctly declared |
| persistent-failure recovery "depends on a later retry or on the finalizer" | **Rejected as written** — F6; neither recovers |
| "the finalizer's timing is CPython refcount behaviour" | **Rejected as written** — F5; it needs the cyclic collector, and never runs while the exception is held |
| composition conflict with the active MR-list lane | Informational; outside this review's scope, not assessed |
| 13 selected legacy CLI cases not re-run | **Partially discharged** — the upload-facing CLI subset is 27/27 OK on both trees; the full 228-test comparison is recorded unavailable (Gates) |

## BLOCKER-1, HIGH-3 and MEDIUM-4 — preserved CLOSED

The candidate's `codefire` diff is six hunks carrying five changes: the two new
`__del__`s, the retained-handle branches in
`_AccountedUploadCommitSource._open` / `.close`, the guarded cleanup arm in
`validate_sealed_commit`'s constructor unwind, and one docstring.
`_control_charge_witness` and `_release_ledger_exactly_once` are byte-identical
to the predecessor. Nothing else in the build changed, so `BLOCKER-1` and `HIGH-3` code
is untouched and `MEDIUM-4` code is untouched apart from the docstring
correction. Confirmed behaviourally rather than by inspection alone:

```bash
python3 -m unittest \
  …test_completed_quarantine_recovery_owns_the_request_budget \
  …test_dropping_the_recovery_handoff_publishes_an_expired_retry \
  …test_publication_root_proof_is_atomically_single_use \
  …test_ordered_projection_allocation_requires_a_proved_charge \
  …test_control_release_is_retryable_and_exactly_once -v
# Ran 5 tests -- OK
```

`MEDIUM-4`'s oracle is still non-vacuous: mutation `m1` deletes the
`_charged_allocation_boundary(...)` call from `_charged_ordered_projection` and
`test_ordered_projection_allocation_requires_a_proved_charge` fails
behaviourally (`E5`, `E6`).

## Mutation Matrix

Each mutation was applied alone to a detached export of the candidate under
`/tmp/rev3/mut_*`; the worktree was never touched.

| Row | Exact edit | Oracle run | Result | Reading |
|---|---|---|---|---|
| `m1` | delete the `_charged_allocation_boundary(...)` call in `_charged_ordered_projection` | `test_ordered_projection_allocation_requires_a_proved_charge` | **FAIL** (1) | `MEDIUM-4` oracle still load-bearing |
| `m2` | restore `self._session = None; self._memory = None` at the top of `_AccountedUploadCommitSource.close` | both new regressions | **FAIL** (1) — direct-wrapper assertion only; both live-route subtests pass | the live-route half pins nothing (F2) |
| `m3` | rename `_AccountedUploadCommitSource.__del__` away | both new regressions; then the full focused suite | **OK** / **33/33 OK** | a load-bearing new symbol with no oracle (F3) |
| `m4` | rename `_AccountedUploadSourceMemory.__del__` away | both new regressions | **FAIL** (1) — constructor-unwind finalizer case | that finalizer is pinned |

Reversal of the two new regressions on the exact predecessor (`/tmp/rev3/predx`
= `02e04c7` tree with the candidate's test file, `codefire` SHA-256
`63a9be23…a872fa`):

```text
Ran 2 tests -- FAILED (failures=3, errors=1)
  FAIL  …cleanup_survives_a_failing_close_syscall (route='ancestry')     assert_closed
  FAIL  …cleanup_survives_a_failing_close_syscall (route='signature-history') assert_closed
  FAIL  …cleanup_survives_a_failing_close_syscall                        assertIsNotNone(source._session)
  ERROR …constructor_unwind_keeps_the_primary_error                      SECONDARY cleanup fault
```

This matches the implementer's claim of "3 failures / 1 error" exactly, and the
failures are behavioural, not symbol-absence (`E6`). On the candidate the same
two tests are **OK**. The reversal is therefore admissible; F2 concerns what the
passing side proves, not whether the reversal is real.

## Same-Root Shared-Owner Sweep

An AST sweep (`sweep.py`) over both whole builds for the three defect shapes —
erase-before-release, finalizer-swallowed-release, and unguarded fallible
cleanup immediately before a bare `raise`:

| Shape | `02e04c7` | `446cf4a` |
|---|---|---|
| erase-before-release | `_AccountedUploadCommitSource.close` ×2 (`self._session = None` @7309 → `session.close()` @7314; `self._memory = None` @7310 → `memory.close()` @7322), plus one false positive | **0** real sites; only the same false positive |
| unguarded cleanup then bare `raise` | includes `_AccountedUploadCommitSource._open: memory.close() @7291` | that site is gone; the remaining hits are byte-identical to the predecessor |
| finalizer `__del__` swallowing its release | 33 owners | 35 — the two new ones join an established, consistent pattern |

The one remaining shape-1 hit,
`AccountedHTTPUploadExistingTerminalOwner.close` (`codefire:26035`/`26065`), is a
false positive of the heuristic: reading it shows the correct
release-then-clear-only-if-released ordering, and the region is byte-identical
on both trees. So the candidate does eliminate both real sites and introduces no
new one of that shape. No double-release, no reservation-order inversion and no
error-displacement path was found in the new code: `pE3` confirms exactly-once
survives a post-effect ledger fault, `pE4` confirms repeated `close` releases
nothing again, and skipping `memory.close()` while the session teardown owes
control debt is the correct ordering, not a displacement.

The two genuinely new hazards are the finalizer-only recovery path on the live
routes (F1) and the `_open` handle-orphaning step (F7).

## Gates

All run on the exact candidate worktree, clean before and after.

| Command | Result |
|---|---|
| `python3 -m py_compile codefire tests/test_python_http_upload_accounted_traversal.py tests/test_python_sealed_production.py tests/test_codefire_cli.py` | PASS |
| `python3 -m unittest tests.test_python_http_upload_accounted_traversal` | **33/33 OK**, `85.503s` |
| the two new regressions, on `02e04c7` | **3 failures / 1 error** (reproduces the implementer's claim) |
| the two new regressions, on `446cf4a` | **2/2 OK** |
| the five inherited-closure tests | **5/5 OK** |
| adjacent owners: ancestry ×2, signature ×3, sealed ×3, external-production | **121/121 OK**, `58.135s` |
| `python3 -m unittest discover -s tests -p 'test_python_*.py'` | **454/454 OK**, `240.812s` |
| `python3 docs/development/tools/check_source_docs.py` | PASS — 44 modules, 34 commands, 12 object kinds |
| `python3 docs/development/tools/check_project_docs.py` | PASS — **533** Markdown files on the exact candidate tree (`result.md` records 532; see F8), and PASS at 534 with this report present |
| `git diff --check` | PASS |
| `git status --short` | clean apart from this report, which is the only repository write |
| `python3 -m unittest tests.test_codefire_cli -k upload` | **27/27 OK** (27 selected), `220.448s` |
| `tests/test_codefire_cli.py` in full, both trees | **not completed — recorded unavailable**, see below |

### The legacy CLI suite — the implementer's last stated gap

`result.md` records that "the 13 selected legacy CLI cases were not re-run in
this iteration". This lane attempted the stronger check — the whole
`tests/test_codefire_cli.py` (`228` tests) on the candidate *and* on the
predecessor — and **that comparison did not complete**. Both processes were
terminated by session teardown; both logs end mid-run in progress marks, with
no unittest summary and no exit status. No exit status is inferred from them
here, and the incomplete runs are not offered as a gate.

What the two partial logs do support, from their content alone, is a prefix
comparison. Their progress-mark sequences are character-for-character identical
over their entire common prefix — `219` of `228` marks under a relaxed parse,
`106` under a strict whole-line parse — and the only two failures sit at the
same ordinals (`11` and `35`) on both trees. So the two failures that were
observed are **pre-existing on `02e04c7`** and not introduced by the candidate.
The last ~9 tests were never compared.

Bounded replacement, run to completion in the foreground on both trees:

| Command | `02e04c7` | `446cf4a` |
|---|---|---|
| `python3 -m unittest tests.test_codefire_cli -k upload` (**27 selected**, non-zero) | **27/27 OK**, `257.606s` | **27/27 OK**, `220.448s` |

The upload-facing legacy CLI surface — the surface this diff touches — is
therefore green and identical on both trees. The full `228`-test comparison
remains **unavailable** in this lane and is recorded as such. It is not one of
this review's required gates, and the verdict does not rest on it: `F1` is
established by `pC`/`pD3`/`pF` against the production observer, and `F2` by the
mutation matrix.

No Cargo/Rust, Docker, install, dogfood or repository-wide validation was run in
this lane.

## Minimal Closure for the Next Iteration

Bounded, and inside the surface this repair already owns — no new public API and
no new state machine is needed.

1. Make the two live routes drain **before** `handle_http_upload` returns or
   raises. Both already own the source at the exact point the cleanup fails —
   `is_ancestor_from_objects_accounted` at `codefire:12051`-`12057` and
   `enforce_remote_commit_signature_policy_accounted`'s release loop at
   `codefire:7415` — so completing the release there (retry `source.close()`
   once the session teardown has cleared its control debt, then re-raise the
   primary) removes the dependence on the exception being dropped. Recovery
   must not require the caller to discard the error it was just handed.
2. Re-point the live-route oracle at the boundary it names: assert through
   `make_http_handler`'s `_decision_work_observer`, or with the raised exception
   still held, rather than after `assertRaises` has stripped the traceback. The
   oracle must fail when mutation `m2` is applied.
3. Add one assertion that fails when `_AccountedUploadCommitSource.__del__` is
   deleted (F3).
4. Correct the two new documentation sentences and remaining-limitations 2 and 3
   to what the build guarantees: the finalizer needs the cyclic collector, it
   cannot run while the error is in flight, and a persistently failing session
   teardown has no recovery (F4, F5, F6).

`BLOCKER-1`, `HIGH-3`, `MEDIUM-4` and the two closed `HIGH-2` limbs need no
further work, and `_control_charge_witness` should stay deferred pending a
recorded manager decision.

## Verdict

`REQUEST_CHANGES`
