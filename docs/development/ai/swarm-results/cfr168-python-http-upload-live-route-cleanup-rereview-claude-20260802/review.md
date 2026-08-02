# CFR-168 / CFB-138 Python HTTP Upload — Live-Route Cleanup Repair, Exact Candidate Rereview

Captured: 2026-08-02

Reviewer: Claude Opus 5 (`claude-opus-5`), fresh independent report-only lane.
No subagent, no Codex, no Fugu, no Cargo/Rust, no install, no dogfood, no
network and no external write was used. Every mutation and probe lived under
throwaway `/tmp/rev4`; the candidate worktree was never modified to test it.
The only repository write is this report.

Reviewed subject: exact immutable candidate
`0f4341561ce95935600bf1398b8a5fbdb206daba`, over implementation base
`5790463864881ee28c372e9bd0b8cd8725c9dc70`.

## Decision

**`ACCEPT`**

Every claim this bounded candidate makes was independently reproduced with a
reviewer-owned harness that never calls the candidate's own test helpers, and
every one survived. The decisive prior blocker is closed at the boundary the
prior report named:

> At the point **production itself accounts the request** —
> `make_http_handler` → `handle_one_request`'s `finally`, read through
> `_decision_work_observer` (`codefire:15379`-`15420`) — a real socket POST
> carrying one real `close(2)` failure is now accounted
> `current = {}`, `open_permit_count = 0`, `terminal_reason = 'complete'`,
> `0` response bytes, zero durable bytes moved. The third-repair review
> measured `retained_bytes = 11,456 / 0 / None` there, and I reproduced that
> exact triple on the base product with the same probe before measuring the
> candidate.

The bound is real and not merely implied: with **eight** `close(2)` faults
armed and available, both live routes still make exactly **two** `close` calls
on the faulting source and exactly one on every other source the request built.
The C14 precedence is the corrected one the manager supplied, verified by
exception **object identity**, not by type or message alone.

`ACCEPT` is scoped exactly as the packet scopes it. It means only that this
bounded candidate is fit for later composition. It does **not** close CFR-168,
CFB-138, all of Upload, global C14, integration, Install or Dogfood, and it
does not clear the separately routed composition blockers listed in
[Residual routing](#residual-routing).

Five non-blocking observations are recorded in [Observations](#observations).
None of them is a must-fix trigger: none is a product regression, none is a
false closure claim, and each is either inherited or a documentation/oracle
narrowing that the candidate's own text already bounds correctly.

Literal prior-finding dispositions (report `3475a8a`):

| Finding | Prior disposition | This review |
|---|---|---|
| `BLOCKER-1` — completed-quarantine recovery request ownership | CLOSED | **CLOSED (preserved)** |
| `HIGH-2` — commit-source cleanup ownership / release-failure contract | OPEN (live-route limb) | **CLOSED** — all three limbs |
| `HIGH-3` — atomically single-use publication-root proof | CLOSED | **CLOSED (preserved)** |
| `MEDIUM-4` — non-vacuous pre-allocation reversal matrix | CLOSED | **CLOSED (preserved)** |

## Immutable Subject and Provenance

Every field below was recomputed from the repository before the review began
and again immediately before the verdict; both computations agree, and both
agree with the exact subject given to this lane.

| Field | Recomputed value | Matches subject |
|---|---|---|
| Candidate HEAD | `0f4341561ce95935600bf1398b8a5fbdb206daba` | yes |
| Candidate tree | `da356c2a041e57eea9192ab917f3a293239c60e7` | yes |
| Candidate parent | `e7312ef3d25e27b89db35bf5080048e0e4a04a34` (tree `5718defd2e6e767bb4f9ec175a9a197b6e9417cd`) | yes |
| Implementation base | `5790463864881ee28c372e9bd0b8cd8725c9dc70` | yes |
| Base tree | `d8c09084f8ec1ee0c08f8d503b3b934f9cad3973` | — |
| Base subject | `docs(cfr168): freeze Upload live-route cleanup repair` (docs-only, `+140` in the packet) | — |
| Additive commits over base | `2` — `e7312ef`, `0f43415` | yes |
| Merges in `5790463..0f43415` | `0` | yes |
| Changed paths | `4` (all `M`) | yes |
| `git diff --binary 5790463..0f43415` SHA-256 | `4ba34a6d92c5a6084f52fcdb978c0a4f0a5687b3fcac4bf019e8b2982dbf2fcb` | yes |
| Worktree at review start / at verdict | clean / clean apart from this report | yes |
| Interpreter | CPython `3.10.12` (`main, Mar 3 2026, 11:56:32`) `[GCC 11.4.0]` | — |

Additive history:

| Commit | Tree | Subject |
|---|---|---|
| `e7312ef3d25e27b89db35bf5080048e0e4a04a34` | `5718defd2e6e767bb4f9ec175a9a197b6e9417cd` | `fix(http-upload): drain commit-source cleanup at both live call sites` |
| `0f4341561ce95935600bf1398b8a5fbdb206daba` | `da356c2a041e57eea9192ab917f3a293239c60e7` | `fix(http-upload): close C14 cleanup precedence and bound the live-route retry` |

Exact changed paths, `5790463` → `0f43415` (`657` insertions, `54` deletions):

| Status | Path | +/− | Head blob | Head bytes | Head SHA-256 |
|---|---|---:|---|---:|---|
| `M` | `codefire` | `90 / 30` | `b984c1e7e393af658a618c96270d33d39bf8eeaa` | `1,414,728` | `8ff05edd67c9247a1f23ad142d990f03ea0f455fcfcd3fbf4257040ec054d56d` |
| `M` | `tests/test_python_http_upload_accounted_traversal.py` | `531 / 19` | `829836bc0411bf82030286e6f1b0e3d58e574e14` | `186,957` | `31fb356171c2ef08b691bcdc467827321a6c89a9d969574537970acf7effc7fa` |
| `M` | `docs/development/source-architecture.md` | `22 / 3` | `cac0adb178be417fc197af1c3187fa14805f3082` | `107,627` | `23b62bf8a88e8f274a8d37676ec8a3fd1f223703c5692efbdc79c6afb49461b7` |
| `M` | `docs/development/command-source-trace.md` | `14 / 2` | `809629ddb96aed3d026d813f9f123fe2482dc1f7` | `42,867` | `d23654e44d2954f42b9118242940271185d92f528f9317d23c38a59249dbe892` |

The write set is exactly the packet's closed write set. No issue status,
backlog, history, cycle state, prior packet, prior report, card, ledger,
Doctor/Branch/MR-list product or test, Rust, profile, packaging, Install,
Dogfood or `subprojects/` byte was touched.

### Immutable authorities, by digest

| Input | Blob | Bytes | SHA-256 |
|---|---|---:|---|
| `…/swarm-batches/cfr168-python-http-upload-live-route-cleanup-repair-20260802.md` | `250703ae50afcf842359abebc813cea717679c6a` | `6,395` | `756bf200138f6066dc1f2ac13d4933cf3e4a4228742bfc079d0c1ce3da2dabe9` |
| `…/swarm-results/cfr168-python-http-upload-third-repair-rereview-claude-20260731/review.md` | `91bd5a92ea9f2ec8a9ccb1760851274724bb66b5` | `34,297` | `f131284ea7d441781790481f1d356ca73a763d1ddb41ae878a6f7179c88ec649` |

Both were read in full. Both are byte-identical at `3475a8a`, at `5790463` and
at the candidate tree — the candidate rewrote no authority:

```text
review.md blob at 3475a8a = 91bd5a92ea9f2ec8a9ccb1760851274724bb66b5
review.md blob at 5790463 = 91bd5a92ea9f2ec8a9ccb1760851274724bb66b5
review.md blob at 0f43415 = 91bd5a92ea9f2ec8a9ccb1760851274724bb66b5
packet   blob at 5790463 = 250703ae50afcf842359abebc813cea717679c6a
packet   blob at 0f43415 = 250703ae50afcf842359abebc813cea717679c6a
```

`AGENTS.md` and both complete commit messages were also read in full.

**Shared-primitives preflight `C14`/`B2`.** This authority does not exist in
the repository. `git grep` over the whole candidate tree returns exactly three
hits for `shared-primitives`, and all three are lines this diff added to the
two canonical source docs; there is no `C14` record, no `B2` record and no
preflight document anywhere under `docs/`, and none existed at the base. The
packet does not mention `C14` or `B2` either. I therefore reviewed the
**manager-supplied contract** as authoritative — cleanup faults never displace
the primary operation decision, including constructor unwind, in every build;
when run/decision and cleanup both fail the exact run/decision exception object
is outer with the first cleanup error as its cause; cleanup alone is outer only
when the operation succeeded — and recorded the unresolvable in-tree citation
as observation [O1](#o1--canonical-source-docs-cite-an-authority-that-does-not-exist-in-tree).

### The base product is byte-identical to the rejected candidate

| Build | `codefire` blob | `codefire` SHA-256 |
|---|---|---|
| `446cf4a` (rejected by `3475a8a`) | `9bdf5aa8bc5d57024e23351729f04ab0b6898ec8` | `d0a84dab25828bedba8fcb745ad09c73cb55ec604de85a0f3275b8137c7f6ca8` |
| `5790463` (this candidate's base) | `9bdf5aa8bc5d57024e23351729f04ab0b6898ec8` | `d0a84dab25828bedba8fcb745ad09c73cb55ec604de85a0f3275b8137c7f6ca8` |

The digest matches the one report `3475a8a` recorded for `446cf4a`. Every
"before" number below is therefore a measurement of the exact product the prior
report rejected, taken by me, on that tree, with my own harness — not a
candidate self-report.

### Reviewer probes

Reproduce with `CF_BUILD=<tree>/codefire PYTHONDONTWRITEBYTECODE=1 python3
<probe>`. Each loads the exact build with `runpy.run_path` and drives real
production entry points against a real `codefire init` store and a real HTTP
project root. None of them imports or reuses the candidate's test module.

| Probe | Purpose | SHA-256 |
|---|---|---|
| `h.py` | harness: real store, real request ledger, real `close(2)`-shaped fault, per-source close counting, whole-project-root digest | `444682c9193194b13450a448f7df57664bb0c62a93de1a7ac6162c8f74fd5cbe` |
| `p1_bounded.py` | both live routes, one-shot and persistent faults: bound, no spin, first-error precedence, held traceback | `e1633bbea2ad13385cc6f60df1b885dac8a3619b10f7bf097cc1479072ce5d58` |
| `p2_c14.py` | real deadline + real `close(2)` together: C14 outer/cause by object identity, cleanup order | `5d6571556d7cda1409b17d82c9387b113678f3ee55212fe664f26a2f4ee600ce` |
| `p3_socket.py` | real socket request to a real `make_http_handler`; `locks/` enumeration | `6ab3e5623d14ca31ce64ea7edb9de6e3e0a27b40e9f8ad30179e4a2231f5f5da` |
| `p4_ctor_nolegder.py` | constructor unwind with the primary error **held**; no-ledger default-arg compatibility | `a38eb1e917133b7d5532aa7b8afd4c818ab7a75897fa76c9537758f8420cb46a` |
| `p5_ctor.py` | constructor unwind with **no injection at all** (absent / `chmod 000` store), both builds | `c2ba25d4840a574cb9be49dad0f6f6d7e3395e6401df81a1547fa5642a6033bd` |
| `p6_live_ctor.py` / `p6b.py` | the store-unavailable arm driven through the **live** upload route | `1a89c7aa65fd28f00f7a90c76500c22b06655eb96c799a5f38d4fdd5e5e5c912` / `46cb57607de2fffd47ac497b196421f532977262a284ee3a364aa0197c19fd77` |
| `p7_hazards.py` | double release, descriptor double-close, cause-cycle and uncollectable-owner audit | `f5afb9ffaac75b42e830adea261c21446fabe77a2e2676ac3eac2d3ff53ad195` |
| `mutate.py` | builds the nine reversal exports; each anchor asserted to match exactly once | `cdc12cc9ed0cdd8743a60cee948f7033e4ae07c77737630f99de81aefa889893` |
| `run_mut.sh` | runs the five boundary tests on one mutated export | `45e554ad196b950c44db7eeb8f87dbb7ea7cfea2c62d9a369fd39c3dd9f772f9` |

## Method

1. **Identity first.** All subject fields recomputed before reading any claim,
   and again before the verdict. Any mismatch or product mutation would have
   been `BLOCK`; there was none.
2. **Baseline before candidate.** Every behavioural probe was run on the base
   tree (`/tmp/rev4/predx`, `codefire` SHA-256 `d0a84dab…7c7f6ca8`) before it
   was run on the candidate, so each "fixed after" statement is a difference
   measured on two builds.
3. **Independent instrumentation.** The harness re-implements bootstrapping,
   fault injection, per-source close counting and the durable digest from
   scratch. Where the candidate's oracle and my probe agree, that is two
   independent measurements, not one.
4. **Oracles judged as properties.** For each new assertion: does it fire at
   the boundary it names, does deleting the production line change the result,
   is the failure behavioural rather than symbol-absence, is the observed state
   production-reachable. Nine single-edit reversals answer this.
5. **Diff read as unreviewed surface.** Every changed line was inspected in its
   own right for unbounded retry, swallowed `BaseException`, traceback/cause
   cycles, double release, leaked handles, false docs, monkeypatch leakage,
   flaky timing/PID assumptions, weak oracles and overclaim, separately from
   inherited-finding closure.
6. **No mutation of the candidate.** All nine reversals, the predecessor
   reversal and every probe ran against detached exports under `/tmp/rev4`.

The reviewed change is small and strictly local. `git diff -U0` hunk headers on
`codefire`, base → candidate:

```text
@@ _AccountedUploadSourceMemory.__del__      docstring
@@ _AccountedUploadCommitSource.__del__      docstring
@@ +34 lines: new _drained_commit_source_cleanup_error
@@ enforce_remote_commit_signature_policy_accounted  cleanup block
@@ is_ancestor_from_objects_accounted                cleanup block
```

Nothing else in the 35,129-line build changed. `_control_charge_witness`,
`_release_ledger_exactly_once`, `_AccountedUploadCommitSource._open`,
`_AccountedUploadCommitSource.close`, `_AccountedUploadSourceMemory.close`,
`SealedCommitValidationSession`, `validate_sealed_commit`,
`run_server_verification` and the request loop are byte-identical to the base.
There are exactly two `_AccountedUploadCommitSource(` construction sites
(`codefire:7438`, `codefire:12093`) and exactly two callers of the new helper
(`codefire:7462`, `codefire:12107`) — the two owned wrappers and no third.

## Falsification of the implementer's claims

Each row states what the candidate claims, what I measured independently, and
on which build.

### 1. Bounded synchronous retry, no spin, no reliance on `__del__`

`p1_bounded.py`, both live routes, one real `close(2)`-shaped fault (the
descriptor really closes, the call still raises `OSError(EIO)`), armed only
while `_AccountedUploadCommitSource.close` is on the stack. No ledger call is
patched anywhere. Every source the request built is held strongly by the
counting wrapper, so no source finalizer can contribute.

```text
                                candidate 0f43415        base 5790463
ancestry   close calls/source   [1, 2]                   [1, 1]
           current (tb held)    {}                       {'retained_bytes': 11456}
signature  close calls/source   [2]                      [1]
           current (tb held)    {}                       {'retained_bytes': 11456}
both       traceback_retained   True                     True
both       durable diff         none (all bytes)         none (all bytes)
```

**The bound is decisive, not incidental.** In the persistent variant I armed
**eight** faults. Both routes still fired exactly **two**:

```text
0f43415, persistent fault (8 available), ancestry
  faults actually fired      2
  close calls per source     [1, 2]
  surfaced error             OSError errno 5 "…failure #1"     <- the FIRST
  current                    {'retained_bytes': 11456}          <- documented, not claimed recovered
```

A build that looped until the teardown succeeded, or retried twice, would have
fired three or more. It fires two. Reversal `r4` confirms the oracle catches
the difference (`[1,3] != [1,2]`, `[3] != [2]`).

The ancestry request builds two sources; exactly one is closed twice and the
unrelated one exactly once. The signature request builds one, closed twice.

### 2. C14 precedence with a real operation failure and a real cleanup failure

`p2_c14.py`: a real `RequestDeadlineExceeded` raised mid-traversal by
production's own deadline poll (the borrow succeeds first, so a real descriptor
is open), plus the one-shot `close(2)` fault in the cleanup that follows.

```text
0f43415, ancestry
  OUTER   RequestDeadlineExceeded: HTTP request deadline of 100 ms exceeded during upload ancestry
          same exception OBJECT the deadline poll raised: True
  CAUSE   OSError errno 5 '[Errno 5] injected close(2) failure #1'
  close calls per source  [1, 2]      traceback retained  True
  current {}   permits False   terminal None   expired_phase 'upload ancestry'
  cleanup order  source.close … authority.close
  durable diff (all bytes, incl. locks)  none

0f43415, signature-history
  OUTER   RequestDeadlineExceeded: … during commit signature history load
          same exception OBJECT: True
  CAUSE   OSError errno 5 '…failure #1'
  close calls per source  [2]         traceback retained  True
  current {}   permits False   terminal None   expired_phase 'commit signature history load'
  cleanup order  source.close … authority.close
  durable diff (all bytes, incl. locks)  none
```

This is the corrected contract, not the legacy cleanup-outer behaviour: the
operation decision is outer **by object identity**, and the first cleanup error
is its `__cause__`. Reading the code confirms the shape at both sites is
literally

```python
if run_error is not None:
    if cleanup_error is not None:
        raise run_error from cleanup_error
    raise run_error from None
if cleanup_error is not None:
    raise cleanup_error from None
```

On the base product the same probe run raises the `OSError` as the outer error
(the inverted `raise cleanup_error from run_error`), which is why the new
regression **errors** rather than fails on that tree.

### 3. Cleanup-only case still surfaces the *first* cleanup error

`p1_bounded.py` persistent variant, first fault `errno 5`, retry fault
`errno 13`. The caller sees `OSError errno 5 "…failure #1"` on both routes.
The retry's error is dropped, never substituted. Confirmed by code: the helper
returns `first_error` from the outer `except`, and the inner `except` only
detaches.

With the operation succeeding and only the cleanup failing, the cleanup error
is the outer exception with `__cause__ = None` — measured, and asserted by the
candidate's own test.

### 4. Release order and exactly-once for unrelated sources

Order is `source` then `authority` at the signature site by construction
(`codefire:7462` before `codefire:7466`) and was observed in that order on both
routes in `p2_c14.py`. Unrelated sources are closed exactly once
(`[1, 2]` / `[2]`, above; `[1, 1, 2]` on the three-source case in
`p7_hazards.py`).

Exactly-once and no double release, `p7_hazards.py`:

```text
clean close x3   #1 -> None current={}   #2 -> None current={}   #3 -> None current={}
                 all ledger dimensions non-negative throughout
```

No descriptor double-close is possible: `_SealedValidationFileDescriptor.close`
sets `_fd = -1` **before** calling `_CODEFIRE_OS_CLOSE`, so the retry finds the
owner already drained and never re-closes the fd. That is exactly why the retry
succeeds after a `close(2)` fault.

### 5. The real socket observer is production's checkpoint and is non-vacuous

`_decision_work_observer` is invoked inside `handle_one_request`'s `finally`
(`codefire:15397`), **after** the `close_connection` / `terminate` decision at
`codefire:15384`-`15396` — i.e. it reads the exact snapshot production itself
uses. `p3_socket.py` drives a real `POST … /upload HTTP/1.0` over a real socket
to a real `ReusableThreadingHTTPServer`:

```text
                                   base 5790463             candidate 0f43415
close(2) faults fired              1                        1
PRODUCTION observation current     {'retained_bytes':11456} {}
PRODUCTION observation permits     0                        0
PRODUCTION terminal_reason         None                     'complete'
budget / deadline diagnostic       None / None              None / None
HTTP response bytes                0                        0
error escaping the request loop    OSError errno 5          OSError errno 5
close calls per source             [1, 1] / [1]             [1, 2] / [2]
durable diff, ALL bytes incl locks none                     none
```

The base column reproduces the prior report's decisive `11,456 / 0 / None`
triple exactly, independently. Non-vacuity is proved by reversal: `r1` and `r2`
each restore `{'retained_bytes': 11456} != {}` in this very test.

`terminal_reason = 'complete'` is not a mis-accounting. The loop reaches its
`terminate` arm only because there is no debt, no open permit and no deadline
error; the same arm already fires for any fully accounted request that the
application rejected. The upload still fails, the `OSError` still escapes, the
client still receives zero bytes, and both canonical docs say so in as many
words.

### 6. Traceback retention does not reopen the debt

Every measurement above was taken **inside the raised error's own handler with
`__traceback__` retained** (`traceback_retained: True` in `p1`/`p2`, and the
socket observer runs while the `OSError` is propagating). This is the single
variable the prior report isolated as the reason the old oracle passed and
production did not. On the candidate the held-traceback branch reads `{}`; on
the base it reads `11,456`.

`p7_hazards.py` also checks the new cause link for cycles:

```text
chain = RequestDeadlineExceeded(tb=True) <- OSError(tb=False)
self-referential: False
cyclic-garbage owners after collect: 0
```

The attached cause carries no traceback (`_detach_*` cleared it before it was
returned), so the C14 link adds no frame retention and no cycle.

### 7. Constructor unwind

With **no injection at all** — an absent object store and a `chmod 000` object
store, exactly the trigger the prior report found — the unwind keeps its
primary decision on both builds:

```text
p5_ctor.py, candidate and base, identical output
  PRIMARY  CFError: 'object store is unavailable: …/.codefire/objects'
  after unwind    current={'retained_bytes': 11456}  memory handle kept
  retry #1 close() -> None   current={}
  retry #2 close() -> None   current={}
  after drop + gc            current={}   permits=False
```

Primary decision preserved, retryable, drains. But this is **only** true when
the primary error is not held. Holding it (`p4_ctor_nolegder.py`) pins the
partially constructed `SealedCommitValidationSession`, whose control claim is
therefore never released, so `_AccountedUploadSourceMemory.close` refuses and
every subsequent `close()` raises `upload commit source closed with
retained_bytes control debt`. See [O4](#o4--the-constructor-unwind-limb-is-retryable-only-while-the-primary-error-is-not-held).

I did **not** over-credit the checked-in
`test_commit_source_constructor_unwind_keeps_the_primary_error`: it is
inherited, unchanged by this diff, driven by monkeypatched subclasses, and
measured after `assertRaises` stripped the traceback. It is not evidence for
this limb; `p5_ctor.py` is.

No-ledger compatibility (`ledger=None` default argument) was exercised on both
wrappers and both returned normally.

### 8. Reversal matrix — nine single edits, all behavioural

Each mutation was applied **alone** to a detached export under
`/tmp/rev4/mut/*`; every anchor was asserted to match exactly once; every
mutated build was `py_compile`d before use. The five boundary tests were then
run on each.

| Row | Exact edit | Result | Failing assertion — behavioural, at the named boundary |
|---|---|---|---|
| `r1` | ancestry call site: retry removed (plain `source.close()`) | **FAILED (3)** | `close calls per source: [1, 1]` ≠ `[1, 2]` ×2; socket observer `{'retained_bytes': 11456} != {}` |
| `r2` | signature call site: retry removed | **FAILED (3)** | `[1]` ≠ `[2]` ×2; socket observer `{'retained_bytes': 11456} != {}` |
| `r3` | `_AccountedUploadCommitSource.__del__` renamed away | **FAILED (1)** | `assert_closed(ledger)` at test line `4291` → `assertTrue(all(value == 0 …))` |
| `r4` | helper retries twice instead of once | **FAILED (6)** | `[1, 3]` ≠ `[1, 2]` and `[3]` ≠ `[2]`, on all three route tests |
| `r5a` | ancestry: `raise cleanup_error from run_error` restored | **FAILED (errors=1)** | the `OSError` escapes as the outer error, so the C14 test never sees its deadline error |
| `r5b` | signature: same inversion | **FAILED (errors=1)** | same |
| `r6a` | ancestry: `from cleanup_error` → `from None` | **FAILED (1)** | `assertIsInstance(observed["cause"], OSError)` → `None` |
| `r6b` | signature: same | **FAILED (1)** | same |
| `m2` | `self._session = None; self._memory = None` restored at the top of `close` | **FAILED (2)** | `assertIsNotNone(source._session)` at line `4032`; finalizer test at line `4286` |

Every one of the nine reversals the candidate's commit message claims was
reproduced, with the same shape the message states. `r3`'s failure is at
`assert_closed`, exactly as claimed.

### 9. Reversal onto the exact base product

`/tmp/rev4/predx` = the `5790463` tree with the candidate's test file only
(`codefire` SHA-256 `d0a84dab…7c7f6ca8`):

```text
Ran 5 tests -- FAILED (failures=4, errors=2)
  FAIL   …cleanup_survives_a_failing_close_syscall (ancestry)          [1, 1] != [1, 2]
  FAIL   …cleanup_survives_a_failing_close_syscall (signature-history) [1]    != [2]
  FAIL   …request_loop_accounts_a_drained_cleanup_fault (ancestry)     {'retained_bytes': 11456} != {}
  FAIL   …request_loop_accounts_a_drained_cleanup_fault (signature-history) {'retained_bytes': 11456} != {}
  ERROR  …live_cleanup_fault_never_displaces_the_operation_decision (ancestry)          OSError escapes
  ERROR  …live_cleanup_fault_never_displaces_the_operation_decision (signature-history) OSError escapes
```

On the candidate the same five are **OK**. `…finalizer_settles_an_abandoned_retry`
and `…constructor_unwind_keeps_the_primary_error` pass on both, correctly:
neither finalizer nor the unwind is new — only their oracles are, and `r3`/`m2`
show both oracles are load-bearing.

### 10. `durable_snapshot` classification, inspected independently

The helper excludes the **bytes** of files under `locks/` while keeping every
such path as a snapshot key. I enumerated what actually exists there after a
real request (`p3_socket.py`):

```text
locks/project-gate.lock             0 bytes   b''
locks/project-gate.lock.owner.json  82 bytes  {"created_at":"2026-08-02T14:53:46Z",
                                               "mode":"exclusive","pid":979229,"version":1}
```

- `*.lock` files are created `O_RDWR|O_CREAT` and never written — always empty.
- `*.owner.json` is written by `_publish_remote_lock_owner`
  (`codefire:1042`-`1072`), whose own comment reads *"Owner metadata is
  diagnostic and never authoritative."* Its whole payload is `version`, `pid`,
  `mode`, `created_at`.
- Every authoritative artifact lives **outside** `locks/` and is still fully
  digested: `gc_state.json` (`remote_generation_path`),
  `request_nonce_cache.json` (`remote_request_nonce_cache_path`),
  `remote_project.json`, `server_policy.json`, `branches/`, `objects/`,
  `transactions/`, `idempotency/`, `audit/`, `merge_requests/`, `gc/`.
- Path presence is still compared, so a leaked, missing or newly created lock
  is still caught.

The exclusion therefore hides nothing authoritative. Independently, I ran every
failing-path probe with **all** bytes compared including `locks/` and measured
zero durable difference in every case (`p1`, `p2`, `p3`), so nothing was hidden
in practice either. The residual breadth of the prefix rule is
[O5](#o5--durable_snapshot-excludes-all-locks-bytes-not-only-owner-metadata).

### 11. Preserved behaviour

All of the following ran green on the exact candidate tree in the 36-test
focused suite, and none of their code was touched by the diff:

| Preserved item | Oracle | Result |
|---|---|---|
| default `http_v1` limits, record shape/size curves | `test_default_profile_record_shape_and_size_curves`, `test_exact_and_exact_minus_one_budgets_precede_mutation` | OK |
| CFB-138 1201-commit default-profile reversal | `test_reproduced_all_record_reservation_fails_the_default_profile`, `test_new_live_path_replaces_the_reproduced_uncharged_traversals` | OK |
| 1x / 3x / 10x accounting curves | `test_scale_curves_stay_within_the_owner_lifetime_bound`, `test_graph_shape_curves_reach_the_same_owner`, `test_record_shape_curves_reach_the_same_owner` | OK |
| `BLOCKER-1` | `test_completed_quarantine_recovery_owns_the_request_budget`, `test_dropping_the_recovery_handoff_publishes_an_expired_retry` | OK |
| `HIGH-3` | `test_publication_root_proof_is_atomically_single_use` | OK |
| `MEDIUM-4` | `test_ordered_projection_allocation_requires_a_proved_charge`, `test_ordered_projection_claim_is_exact_and_precedes_its_allocation` | OK |
| control release retryable + exactly once | `test_control_release_is_retryable_and_exactly_once` | OK |
| prior signature error order / precedence | `test_accounted_signature_policy_matches_the_legacy_first_error`, `test_signature_policy_decisions_and_precedence_are_preserved` | OK |
| no-ledger compatibility | `p4_ctor_nolegder.py`, both wrappers with `ledger=None` | OK |

Doctor / Branch / MR-list behaviour and the docs boundaries around them are
untouched: the diff changes no Doctor/Branch/MR-list product or test file, and
the two canonical source-doc edits are confined to the
`_AccountedUploadCommitSource` paragraphs.

## Adjudication of the four pre-review gaps closed by `0f43415`

| Gap | Verdict | Evidence |
|---|---|---|
| 1. exact bounded-call oracle | **CLOSED** | `assert_one_bounded_retry` requires exactly two calls on the faulting source and one on every other; `r4` fails it (`[1,3]`/`[3]`); independently, 8 armed faults still produce exactly 2 calls (`p1`). It is also non-vacuous on an empty call list (`[] != [2]`). |
| 2. corrected C14 precedence | **CLOSED** | Outer error asserted by type, message **and object identity**; `r5a`/`r5b` (inversion) and `r6a`/`r6b` (cause dropped) all fail. Independently reproduced in `p2_c14.py` on both routes. Matches the manager-supplied contract exactly. |
| 3. real socket observer | **CLOSED** | The assertion reads `_decision_work_observer`, invoked at `codefire:15397` inside `handle_one_request`'s `finally` after the `terminate` decision. `r1`/`r2` and the base product each restore `{'retained_bytes': 11456}` there. Independently reproduced in `p3_socket.py`. |
| 4. narrowed docs / docstrings | **CLOSED, with caveats** | Both `__del__` docstrings and both canonical docs now scope the synchronous guarantee to the two live routes, name `validate_sealed_commit` as owning its own release (verified: `codefire:30389`, it does), state that the finalizer needs collectibility and returns nothing while an error's traceback pins the owner, and state that a persistently failing teardown is recovered by neither finalizer. I reproduced that persistent-failure residual (`11,456` retained) and the claim is accurate. Caveats: [O1](#o1--canonical-source-docs-cite-an-authority-that-does-not-exist-in-tree) and [O5](#o5--durable_snapshot-excludes-all-locks-bytes-not-only-owner-metadata). |

## Prior finding disposition, in detail

| Prior finding (`3475a8a`) | Disposition | Basis |
|---|---|---|
| `F1` live-route debt at the production checkpoint (blocking) | **CLOSED** | Base `11,456 / 0 / None` vs candidate `{} / 0 / 'complete'` at `_decision_work_observer`, measured by me on both trees. Reversal restores the base triple. |
| `F2` traceback-stripping false oracle (blocking) | **CLOSED** | `observe_live_close_fault` measures inside the live error's own handler and asserts `traceback_retained`; the outer checkpoint is measured over a real socket. Both fail on the base product. Residual: [O2](#o2--the-live-route-oracles-remain-insensitive-to-m2). |
| `F3` load-bearing finalizer with no reversal | **CLOSED** | `test_commit_source_finalizer_settles_an_abandoned_retry`; `r3` fails it at `assert_closed`, deterministically (the session is held over the drop, so it is not a finalizer race). |
| `F4` unconditional finalizer guarantee in canonical docs/docstring | **CLOSED** | Both docs and both docstrings narrowed; each narrowed claim independently verified, including the persistent-teardown non-recovery. |
| `F5` refcount wording understates the mechanism | **CLOSED** | `source-architecture.md` now says the source "sits in a cycle through the raised error's traceback, so it needs the cyclic collector and returns nothing at all while that error is still held" — which is what I measure. |
| `F6` persistent cleanup failure has no recovery | **Documented, not repaired** — as the packet routed it (`nonblocking_improvement`). Verified: a persistent fault leaves `11,456` on both routes, and the docs say recovery "is not claimed". No infinite retry or framework was added. |
| `F7` `_open` retry orphans a live handle | **Not expanded** — as routed. `_open` is byte-identical to the base. |
| `F8` docs gate not run on the exact tree | **DISCHARGED** | Both checkers re-run on the exact frozen candidate: PASS, `535` Markdown files. |
| `F9` `_control_charge_witness` aggregate read | **Out of scope** — byte-identical, still deferred pending a recorded manager decision. Not promoted. |

## Implementer-claim remeasurement

| Claim (from the two commit messages) | My measurement | Verdict |
|---|---|---|
| "`assert_one_bounded_retry` requires exactly two calls on the faulting source and exactly one on every other source" | `[1, 2]` ancestry, `[2]` signature, `[1, 1, 2]` three-source; `r4` → `[1,3]`/`[3]` | **Confirmed** |
| "Zero-debt alone permitted any retry count … it no longer does" | 8 armed faults → 2 calls; `r4` caught | **Confirmed** |
| "raise the operation decision with the first cleanup error as its `__cause__`" | Outer is the same exception **object** the deadline poll raised; cause is `OSError errno 5 "…#1"` | **Confirmed** |
| "a cleanup fault with no operation error still surfaces the first cleanup error on its own" | `OSError errno 5`, `__cause__ = None` | **Confirmed** |
| "Release order (source, then authority) and exactly-once are unchanged" | Order observed on both routes; triple close is a no-op; ledger never negative | **Confirmed** |
| "`run_server_verification` … is untouched" | Not present in the diff at all | **Confirmed** |
| "sees current zero, permits zero, terminal `complete`, empty response and unchanged durable bytes" | `{}`, `0`, `'complete'`, `0` bytes, zero durable diff **including** all `locks/` bytes | **Confirmed** |
| "The third-repair review measured 11,456 / 0 / None there" | Reproduced by me on the base tree: `{'retained_bytes': 11456}`, `0`, `None` | **Confirmed** |
| "`_AccountedUploadSourceMemory` also serves `validate_sealed_commit`, which owns its own release" | `codefire:30389`; it releases both session and memory itself and is not routed through the helper | **Confirmed** |
| "`durable_snapshot` keeps every lock path as a key but excludes lock-owner bytes" | Accurate in effect; the code excludes *all* `locks/` bytes, and the only other file there is an always-empty `.lock` — see [O5](#o5--durable_snapshot-excludes-all-locks-bytes-not-only-owner-metadata) | **Confirmed, wording narrow** |
| "…which carry pid and an ISO timestamp and are documented in-tree as diagnostic and never authoritative" | `codefire:1068`, verbatim | **Confirmed** |
| "15/15 clean after" (snapshot determinism) | Not reproduced as 15 runs; the snapshot was stable across every run in this lane | **Not independently counted** |
| All nine reversals `r1`–`r6b`, `m2` | All nine reproduced with the stated shapes | **Confirmed** |

## Observations

Each is classified by evidence, product impact, must-fix trigger, actionability
and provenance. **None is a must-fix trigger for this bounded candidate.**

### O1 — canonical source docs cite an authority that does not exist in-tree

**Evidence.** `git grep shared-primitives` over the candidate tree returns
exactly three lines, all added by this diff:
`source-architecture.md:459` ("shared-primitives `C14`"),
`source-architecture.md:463` ("the shared-primitives preflight recorded as
behaviour-open `B2`") and `command-source-trace.md:119`. There is no
shared-primitives document, no `C14` record and no `B2` record anywhere in the
repository, and none at the base. The packet does not mention them either.

**Product impact.** None — the rule as stated is correct and matches the
manager-supplied contract and the shipped behaviour. The defect is
traceability: a reader of the canonical source docs cannot resolve the cited
authority.

**Must-fix trigger.** No. It is not a false behavioural claim and not a false
closure claim. Both docs checkers pass.

**Actionability.** High and cheap: either land the shared-primitives authority
in-tree and link it, or restate the rule without the `C14`/`B2` labels.

**Provenance.** Candidate-created. Recommended for the composition step, not
for this candidate.

### O2 — the live-route oracles remain insensitive to `m2`

**Evidence.** Under `m2` (erase-before-release restored) both live-route
subtests still **pass**; only the direct-wrapper assertion (line `4032`) and
the new finalizer test (line `4286`) fail. My `p1` probe on the `m2` build
shows why: with the handles erased, the session becomes unreachable as soon as
`close` returns, so `SealedCommitValidationSession.__del__` clears the control
debt and `_AccountedUploadSourceMemory.__del__` then releases — reaching
`current = {}` without the wrapper contributing. Holding the *sources* (which
the counting wrapper does) does not hold the session and memory owners on a
build that erased the references to them.

The test's inline comment — "`calls` holds every source, so nothing here could
have been settled by a finalizer" — is **true of the shipped build**, because
there the source keeps `_session` and `_memory`, so holding the source holds
them transitively. It is not true of an arbitrary build.

**Product impact.** None. This is an oracle-strength observation.

**Must-fix trigger.** No. The repair actually under review is the synchronous
retry, and it is pinned at both checkpoints by `r1`, `r2` and `r4`; `m2` is
pinned by two other assertions in the same file. The prior `F2` concern — that
the live-route assertions pinned *nothing* — no longer holds.

**Actionability.** Low. If wanted, hold the session and memory owners as well
as the source, or soften the comment to "no *source* finalizer".

**Provenance.** The comment is candidate-created; the oracle shape is
inherited.

### O3 — the retry error is swallowed unconditionally, including `BaseException`

**Evidence.** `codefire:7415`-`7416`:

```python
except BaseException as retry_error:
    detach(retry_error)
```

A `KeyboardInterrupt` or `SystemExit` raised during the retry is discarded and
the first cleanup error is returned in its place.

**Product impact.** Very low. This implements the packet's requirement 3 (the
*first* cleanup error must be propagated even when the retry drains everything)
and matches the established convention in this file — both `__del__`s and the
pre-existing release loop already used `except BaseException` with first-error
precedence.

**Must-fix trigger.** No — the alternative would violate the packet.

**Actionability.** Low; would need a project-wide decision about asynchronous
exceptions during cleanup, not a change here.

**Provenance.** Candidate-created instance of an inherited convention. Routed.

### O4 — the constructor-unwind limb is retryable only while the primary error is not held

**Evidence.** `SealedCommitValidationSession.__init__` claims its whole control
state on the owner and then opens the object-store root; the `os.open` failure
arm raises `CFError("object store is unavailable: …")` **outside** the
`try/except` that releases that claim (`codefire:21860`-`21871`). The owner
therefore reaches the unwind in control debt, and
`_AccountedUploadSourceMemory.close` refuses while that debt exists.

With the primary error **dropped** (`p5_ctor.py`), the partially constructed
session becomes collectible, its `__del__` releases the claim, and the explicit
retry then drains to `{}` — identical on base and candidate. With the primary
error **held** (`p4_ctor_nolegder.py`), the session stays pinned and every
subsequent `close()` raises `upload commit source closed with retained_bytes
control debt`; the reservation is not recovered.

**Product impact.** Not observed on either live upload route. Three attempts to
drive this arm through `handle_http_upload` with a real filesystem trigger
(`chmod 000` at the 1st, 2nd and 3rd `_open`) all ended with the request
returning `200` and `current = {}`, **identically on base and candidate**.

**Must-fix trigger.** No. `_open`, both `close` methods and the session
constructor are byte-identical to the base, so the diff neither created nor
regressed this; it falls under the packet's `F6` (`nonblocking_improvement`,
"document as not guaranteed"), and the candidate's docs and docstrings state
exactly that ("a session teardown that keeps failing is recovered by neither
finalizer … Recovery from a permanently failing teardown is not claimed").

**Actionability.** Medium, out of this write set: move the `os.open` arm inside
the releasing block in `SealedCommitValidationSession.__init__`.

**Provenance.** Inherited. Routed.

### O5 — `durable_snapshot` excludes all `locks/` bytes, not only owner metadata

**Evidence.** The docstring justifies the exclusion in terms of "lock owner
metadata"; the code applies `key.startswith("locks/")`. Today the only files
there are an always-empty `*.lock` and the diagnostic `*.owner.json`, so
nothing authoritative is hidden (see §10), and I confirmed zero durable
difference with `locks/` bytes **included** on every failing path.

**Product impact.** None today. The rule would go blind if an authoritative
artifact were ever placed under `locks/`.

**Must-fix trigger.** No.

**Actionability.** Cheap: narrow the predicate to `*.owner.json`, or digest the
owner payload with `pid`/`created_at` normalised out so the bytes stay compared.

**Provenance.** Candidate-created.

### O6 — the intermediate commit carries the inverted precedence

**Evidence.** `e7312ef` introduced the helper but kept
`raise cleanup_error from run_error` at both sites; `0f43415` corrected it.

**Product impact.** None as delivered — the two commits are additive with no
merge and land together, and only `0f43415` is the product.

**Must-fix trigger.** No. Informational, and relevant only if the range is ever
split or bisected.

**Provenance.** Candidate-created. Informational.

## Residual routing

Recorded honestly; none was modified by this diff, none is newly regressed by
it, and the candidate's docs claim none of them closed. The
`source-architecture.md` text is explicit: *"That synchronous guarantee is
scoped to those two commit-source routes and to nothing else."*

| Residual | State on the candidate | Routing |
|---|---|---|
| `AccountedServerVerification` | untouched; not in the diff | separately routed composition blocker |
| handler `POST` / request-loop `finally` | untouched; only *read* by the new observer test | separately routed |
| signature / server / ancestry owners outside the two wrappers | untouched | separately routed |
| planner / stage / sealed factories | untouched | separately routed |
| production-unwired successor owners (`AccountedHTTPUploadAncestry`, `AccountedHTTPUploadSealedAncestry`) | untouched | separately routed |
| `validate_sealed_commit` (`codefire:30389`) | already C14-correct on precedence (`raise run_error` then `cleanup_error`); no cause link, no synchronous retry; owns its own release, as the docs say | outside the two wrappers; routed |
| constructor unwinds do not attach the cleanup error as `__cause__` | inherited and unchanged in both `_open` and `validate_sealed_commit`; the docs claim only that they "already kept that precedence", which is true | routed — the cause-link half of the C14 contract is not yet realised there |
| `F6` persistent teardown | no recovery; reproduced (`11,456` retained); documented as not claimed | routed `nonblocking_improvement` |
| `F7` `_open` handle orphaning | byte-identical to the base | routed owner-hardening |
| `F9` `_control_charge_witness` | byte-identical to the base | deferred pending a recorded manager decision |

## Gates

All run on the exact candidate worktree (`0f43415`, tree `da356c2a…`), clean
before and after. A skipped or unavailable gate is recorded as such and is
never reported as PASS.

| Command | Result |
|---|---|
| `python3 -m py_compile codefire tests/test_python_http_upload_accounted_traversal.py tests/test_python_sealed_production.py tests/test_codefire_cli.py` | **PASS** |
| `python3 -m unittest tests.test_python_http_upload_accounted_traversal -v` | **36/36 OK**, `105.432s` (base suite was 33; the diff adds exactly 3) |
| `python3 -m unittest tests.test_python_ancestry_owner tests.test_python_signature_owner tests.test_python_signature_terminal_freeze tests.test_python_sealed_production tests.test_python_external_production -v` | **59/59 OK**, `31.734s` |
| `python3 -m unittest discover -s tests -p 'test_python_*.py'` | **457/457 OK**, `301.396s` |
| `python3 -m unittest tests.test_codefire_cli -k upload` | **27/27 OK** (27 selected), `251.044s` |
| `python3 docs/development/tools/check_source_docs.py` | **PASS** — 44 modules, 34 commands, 12 object kinds |
| `python3 docs/development/tools/check_project_docs.py` | **PASS** — **535** Markdown files on the exact candidate tree, and **PASS** at **536** with this report present |
| `git diff --check` | **PASS** |
| `git status --short` | clean (before this report) |
| the five boundary tests on the exact base product (`/tmp/rev4/predx`) | **4 failures / 2 errors** — reproduces `F1` and the precedence inversion |
| nine single-edit reversals `r1`–`r6b`, `m2` | **all FAIL**, each at the boundary it names |
| three-run stability check of the three new tests (real socket, real server thread, `InjectedClock`) | **3×3/3 OK**, `2.569s` / `3.109s` / `3.324s` — no timing, port or PID flake observed |

No Cargo/Rust, Docker, install, dogfood, network or repository-wide validation
was run in this lane. `./demo.sh` was not run and is recorded as not run.

## Verdict

`ACCEPT`

This bounded candidate is fit for later composition. It does not close CFR-168,
CFB-138, all of Upload, global C14, integration, Install or Dogfood, and the
residuals above remain open and routed.
