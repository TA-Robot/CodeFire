# CFR-168 / CFB-138 Python HTTP Upload — Second Review-Closure Independent Rereview

Captured: 2026-07-31

Reviewer: Claude Opus 5 (`claude-opus-5`), independent report-only fallback lane.

Reviewed subject: exact immutable candidate
`02e04c7e6458035d9c7d58de4d30b4a12e7cd594` only.

Rejected predecessor: `467b0c27715e424ae18ffb5664da053b55915e2d`.

## Decision

**`REQUEST_CHANGES`**

`MEDIUM-4` is closed. The ordered object-id projection now passes through a
named pre-allocation boundary, and both required mutation shapes fail *before*
`list(self.documents)` runs, proved by an oracle that observes the memo's
`__iter__` directly. `BLOCKER-1` and `HIGH-3` remain closed. Every gate the
implementer claimed reproduces.

Acceptance is nevertheless forbidden because **`HIGH-2` is still open**. The
repair made `_AccountedUploadSourceMemory` release retryable and exactly once,
but it repaired only the three sites it named. The same owner is used by
`_AccountedUploadCommitSource`, the commit source of the two live HTTP-upload
routes (fast-forward ancestry and commit-signature history), whose `close`
**nulls both retry handles before the fallible cleanup**. On those routes the
new retryability is unreachable: a single failing `close(2)` — no ledger fault
injected anywhere — permanently strands `11,456` retained bytes on the live
request `DecisionWorkLedger`, with `terminal_reason = None`, no open permits,
and `source.close()` retry a silent no-op. The same constructor-unwind shape
also replaces the primary decision with the cleanup fault, in
`_AccountedUploadCommitSource._open` *and* in `validate_sealed_commit`, whose
main body did receive the primary-error precedence the repair advertises.

Literal prior-finding dispositions:

| Finding | Prior disposition | This review |
|---|---|---|
| `BLOCKER-1` — completed-quarantine recovery request ownership | CLOSED | **CLOSED (preserved)** |
| `HIGH-2` — usable caller-owned sealed-validation APIs / release-failure contract | OPEN | **OPEN** |
| `HIGH-3` — atomically single-use publication-root proof | CLOSED | **CLOSED (preserved)** |
| `MEDIUM-4` — non-vacuous pre-allocation reversal matrix | OPEN | **CLOSED** |

No product, test, canonical source-document, packet, card, existing report,
issue, ledger, or lifecycle byte was modified by this review. Every mutation
was applied to a detached export under `/tmp/mut` and reversed to the exact
candidate bytes; every probe lived under `/tmp/cfr168cf`. The only repository
write is this report.

## Prior-Lane Infrastructure Incident (not a product finding)

A prior Sakana Fugu rereview attempt against this exact candidate **failed
before writing any report**: its exec tool repeatedly could not create
processes. It produced no verdict, no evidence, and no disposition, and it is
recorded here only so the missing artifact is not mistaken for a silent
acceptance. That incident is an infrastructure failure of the reviewing lane
and carries no implication about the candidate.

The report actually read as input is the *earlier* Fugu review of the
**rejected predecessor** `467b0c2`
(`docs/development/ai/swarm-results/cfr168-python-http-upload-review-closure-rereview-fugu-20260731/review.md`,
commit `8bdc076`, blob `dd911a63bda6d54b02096affbec864c10922339e`, `44,609`
bytes, SHA-256
`57d6792106f09c17a1d154aedb5a64d342e5d19c23db7e9963c46061c868c132` — matching
the digest `result-2.md` cites). Its `HIGH-2` and `MEDIUM-4` claims were
treated strictly as hypotheses and were re-derived from scratch below.

## Immutable Subject and Provenance

| Field | Recomputed value |
|---|---|
| Candidate HEAD | `02e04c7e6458035d9c7d58de4d30b4a12e7cd594` |
| Candidate tree | `d1023263ee9da257336e1d3cdf437306f0df9299` |
| Parent (production fix) | `909bbea8239dc20b7f50cdabd1a02b86d9540896` |
| Rejected predecessor | `467b0c27715e424ae18ffb5664da053b55915e2d` |
| Continuation base / merge base | `b19c4346c0437ed28bdfe2fb6af1dd466abe9f8b` |
| Additive commits over the base | `5` (`f2ebb32`, `f0ecd25`, `467b0c2`, `909bbea`, `02e04c7`); `0` merges |
| Worktree at review start/end | clean |

Additive history since the rejected predecessor:

| Commit | Tree | Subject |
|---|---|---|
| `909bbea8239dc20b7f50cdabd1a02b86d9540896` | `767cc06b0d90371d4f41236dc559728eb8d08176` | `fix(http-upload): make control release retryable and prove the projection charge` |
| `02e04c7e6458035d9c7d58de4d30b4a12e7cd594` | `d1023263ee9da257336e1d3cdf437306f0df9299` | `docs(cfr168): record the second HTTP upload review-closure repair` |

Exact changed paths, `467b0c2` → `02e04c7` (`581` insertions, `26` deletions):

| Status | Path | +/− | Head blob | Head bytes | Head SHA-256 |
|---|---|---:|---|---:|---|
| `M` | `codefire` | `192` | `2851947f5856153f5e3100896402cd2b03269f02` | `1,408,820` | `63a9be232d064afa022432537f0686e57d47845b1f5654b929a69acec7a872fa` |
| `M` | `tests/test_python_http_upload_accounted_traversal.py` | `288` | `0dec20de1c77b367ce8abb37606e1e30371bbe94` | `154,579` | `f0dd71624c096308365e392bfc4611f616c25537ca71848578efa0f6e05d46a3` |
| `A` | `.../cfr168-python-http-upload-zero-residual-review-closure-repair-20260731/result-2.md` | `102` | — | — | `f3f2dcd5d345427d683a5a197c692ada7371cc9863f464e6b160cffe5ee67bae` |
| `M` | `docs/development/change-playbook.md` | `11` | — | — | — |
| `M` | `docs/development/source-architecture.md` | `12` | — | — | — |
| `M` | `docs/development/source-data-model-catalog.md` | `2 / 2` | — | — | — |

Reviewer probe scripts (reproduce with `CF_BUILD=<candidate>/codefire
PYTHONDONTWRITEBYTECODE=1 python3 <probe>`):

| Probe | SHA-256 |
|---|---|
| `harness.py` | `ff11b06c49d86f02ac66a2d13d383ba38f6fffd5ce76a202a20fe5008513e18d` |
| `p1_public_retry.py` | `cb4789484f2620c9c0975a02beef07811d53712316cad1f788ee28564bde80f1` |
| `p2_source_wrapper.py` | `171faa647b63adb060a3006838034eec633eb846ca3b75374bbdabc93effd907` |
| `p3_boundary.py` | `2a08724d3c499e693f6c1d8b4399673039b721117542d4059ca94852d869aa66` |
| `p4_high2_matrix.py` | `95621c35f1bd5440aa0e3bfa3d14cc1a7865a5d61f240f009e3403aa00ca6bac` |
| `p5_witness_reach.py` | `7bf705224c42937a323f0050602198bb77cc026c3acf26ca41a97e15f1ca2d97` |
| `p6_realistic.py` | `8b251b76a8144a5e5d80083f6e7e2245c10c098c16ca4201c17ed63d8c988728` |
| `p7b_live_http.py` | `3cc821cee0a4df27a39f5e42d26f69579e0b5d7a2bedccb7816151a88f81d25b` |
| `mutate.py` | `c4c7607d9eb0ba308eaa1dd37f129282b73acbc0c326a1e698180362be0cd9d8` |

Each probe loads the candidate `codefire` with `runpy.run_path`, the same way
the checked-in suite does, and drives real production entry points against a
real `codefire init` store and a real HTTP project root.

## HIGH-2 — OPEN

### What the repair actually fixed (independently confirmed)

`_release_ledger_exactly_once` (`codefire:7175`-`7196`) reads the caller
ledger's scalar before the release, runs the caller's `consume` only when that
scalar really moved, and always propagates. `close` (`codefire:7207`-`7236`),
`_release_control_memory` (`codefire:7198`-`7209`) and
`SealedCommitValidationSession._release_control_scalar`
(`codefire:22765`-`22793`) are built on it. Reviewer-authored fault matrix on
the exact candidate (`p4_high2_matrix.py`), `SEALED_SESSION_CONTROL_UPPER =
11,456`, projection `352` bytes at `37` documents:

| # | Case | Result |
|---|---|---|
| 1 | post-effect `retained_bytes` fault (ledger commits, then raises) | fault propagates; ledger `{}`; owner `closed`, unacquired; two further `close()` calls release nothing again — **exactly once** |
| 2 | persistent `retained_bytes` fault | three owner retries each raise; debt stays exactly `{'retained_bytes': 11456}`; heals to `{}` on the first retry after the fault is removed |
| 3 | pre-effect `temporary_bytes` projection fault | the session's own cleanup retries the still-owed release; ledger `{}` without any owner retry |
| 4 | primary/secondary precedence on the main body (`finalize` raises `PRIMARY`, cleanup raises) | caller sees the `PRIMARY` decision |
| 6 | repeated `close()` on a healthy owner (×3) | no double release; ledger `{}` |

The corresponding mutation reversal is load-bearing: restoring the pre-repair
`self._acquired = False; self._closed = True; self._ledger.release(...)`
ordering makes `test_control_release_is_retryable_and_exactly_once` fail with
three failures, one per public caller-owned path (`mB` below).

### Decisive residual — the wrapper discards the retry handle

`_AccountedUploadCommitSource` owns exactly this `_AccountedUploadSourceMemory`
on two **live** HTTP-upload routes:

```text
handle_http_upload
  -> codefire:13127  is_ancestor_from_objects_accounted(...)          # fast-forward ancestry
       -> _AccountedUploadCommitSource(objects_root, ledger, deadline, "upload ancestry")
  -> codefire:13010  enforce_remote_commit_signature_policy_accounted(...)
       -> _AccountedUploadCommitSource(..., "commit signature history load")
```

Its cleanup consumes both handles before anything fallible runs
(`codefire:7306`-`7328`):

```python
    def close(self) -> None:
        session = self._session
        memory = self._memory
        self._session = None          # <-- retry handles consumed here ...
        self._memory = None
        ...
                session.close(None)   # <-- ... before the fallible releases
        ...
                memory.close()
```

`_open`'s constructor unwind does the same (`codefire:7289`-`7292`):
`self._memory = None` precedes `memory.close()`, and a `memory.close()` failure
replaces the constructor's own exception.

`_AccountedUploadSourceMemory` has no `__del__`, so there is no finalizer
safety net either (the sweep for this pattern over the whole build returns
exactly these two sites and nothing else).

Reviewer counterexamples (`p2_source_wrapper.py`):

| Case | First outcome | Handles after fault | Retry | Residual after retry + `gc.collect()` |
|---|---|---|---|---|
| `_AccountedUploadCommitSource.close`, one-shot pre-effect `release` fault | `RuntimeError` | `session=None`, `memory=None` | `source.close()` returns `None` | `{'retained_bytes': 11456}` |
| `is_ancestor_from_objects_accounted`, same fault | `RuntimeError` | — | second public call succeeds | `{'retained_bytes': 11456}` |
| `_open` constructor unwind (`PRIMARY` ctor error + cleanup fault) | caller sees `RuntimeError: SECONDARY cleanup fault` | `memory=None` | — | `{'retained_bytes': 11456}` |

### The trigger is realistic, and it reaches the live request

No `DecisionWorkLedger.release` is patched in the following. A single
`close(2)` raises `OSError(EIO)` — the fd *is* closed, the call still fails,
exactly as close(2) may — while `_AccountedUploadCommitSource.close` is on the
stack. That aborts `SealedCommitValidationSession.close`'s release sequence, so
the source-memory owner then legitimately refuses to close with control debt;
by then both handles are already `None`.

`p6_realistic.py`, direct:

```text
injected close(2) failures = 1
first outcome  = OSError: [Errno 5] injected close(2) failure
handles        = (None, None)
retry outcome  = None            # source.close() again is a silent no-op
residual       = {'retained_bytes': 11456}  permits=False  terminal=None
```

`p7b_live_http.py`, through the production handler:

```text
faulted upload outcome  = OSError: [Errno 5] injected close(2) failure
ledger residual         = {'retained_bytes': 11456}
permits                 = False
terminal_reason         = None
durable paths changed   = []
```

So one failing `close(2)` inside the upload ancestry commit source escapes
`handle_http_upload` as a raw `OSError` — not a typed HTTP status — and leaves
the request budget permanently `11,456` bytes in debt with `terminal_reason =
None`. The leak is silent, unrecoverable from any surface the caller can
reach, and stable across further calls. No durable authority byte changed
(`remote_project.json`, `generation.json`, `branches/main.json`,
`request_nonce_cache.json`, `pending_transaction.json` all identical), so the
blast radius is the request budget and the error surface, not publication.

This is the same invariant the repair states in its own docstrings — "the owner
stays open and acquired until the ledger has actually given the reservation
back, so a failure before that release can be retried" — defeated one frame up.

### Supporting residual — public-API retry cannot drain the debt

Even on the three repaired public caller-owned APIs, the owner that holds the
retry is a **local** in `validate_sealed_commit` (`codefire:30316`) and in
`_open`, never returned, never registered. `p1_public_retry.py`, with a
*one-shot* fault that is gone by the time of the retry, the exception dropped,
and `gc.collect()` run:

| Public API | After fault | Public retry | After retry | After a third call |
|---|---|---|---|---|
| `validate_sealed_commit` | `{'retained_bytes': 11456}` | returns normally | `{'retained_bytes': 11456}` | `{'retained_bytes': 11456}` |
| `validate_remote_json_publication_identity` | `{'retained_bytes': 11456}` | returns normally | `{'retained_bytes': 11456}` | `{'retained_bytes': 11456}` |
| `preflight_remote_json_publication` | `{'retained_bytes': 11456}` | returns normally | `{'retained_bytes': 11456}` | `{'retained_bytes': 11456}` |

The checked-in regression
`test_control_release_is_retryable_and_exactly_once` obtains its `owner` by
monkeypatching `_AccountedUploadSourceMemory` in the module globals with a
recording subclass. It therefore pins owner-level retryability but pins nothing
that a production caller can reach. The prior review's minimal closure did
allow "owner retry **or** API cleanup", so this alone would not block; it is
recorded because it is the reason the wrapper defect above has no fallback.

### Supporting residual — constructor-unwind precedence

`validate_sealed_commit` gained explicit primary-error precedence in its main
body but its constructor unwind (`codefire:30323`-`30326`) still lets the
cleanup fault escape in place of the constructor's decision. `p4` case 5:

```text
5 ctor unwind: caller sees RuntimeError: injected release fault
               (expected ValueError: PRIMARY constructor decision)
               residual = {'retained_bytes': 11456}
```

### Minimal closure for HIGH-2

- Do not consume `self._session` / `self._memory` in
  `_AccountedUploadCommitSource.close` or `_open` until the corresponding
  cleanup has actually succeeded, so `source.close()` is retryable the way the
  owner beneath it now is.
- Give the constructor unwind in both `_open` and `validate_sealed_commit` the
  same primary-error precedence the main body already has.
- Provide a recovery a production caller can reach — either a public retry
  surface for the still-owed reservation, or an idempotent finalizer on
  `_AccountedUploadSourceMemory` in the shape the other owners on this route
  already use (`AccountedHTTPBundle.__del__`, `_SealedSessionDescriptor.__del__`).
- Add regressions on the two live commit-source routes, driven by a failing
  `close(2)` rather than a patched `ledger.release`, asserting that the request
  ledger reaches zero debt and zero permits and that the primary error survives.

## MEDIUM-4 — CLOSED

`_charged_ordered_projection` (`codefire:21811`-`21831`) charges, calls the
named `_charged_allocation_boundary` (`codefire:21790`-`21809`), records
`_ordered_projection_bytes`, and only then evaluates `list(self.documents)`.
Nothing fallible separates the proof from the allocation.

Reviewer-authored boundary matrix (`p3_boundary.py`), oracle = the memo's
`__iter__`, which `list(self.documents)` is the only route caller of; a
37-document closure charges `352` temporary bytes:

| Case | Outcome | Memo iterated |
|---|---|---|
| healthy | completes, ledger `{}` | once |
| projection charge removed | `CFError: sealed validation ordered object-id projection is not charged before allocation: 0 of 352 temporary_bytes proved` | **never** |
| source owner's exact `temporary_bytes` dimension removed | same typed boundary error | **never** |
| `_charged_allocation_boundary` neutralised | completes with the unowned allocation — the boundary is what refuses | once |
| sort fault | typed propagation, ledger `{}` | — |

The `m9` shape (`EXACT_DIMENSIONS = frozenset()`) is refused **before
allocation on all three caller-owned routes** under their natural ledger state
(`p5_witness_reach.py`). This is the pre-allocation, non-vacuous oracle the
ticket required, and it is what the prior review found missing.

### Non-blocking robustness note

`_control_charge_witness` (`codefire:7160`-`7173`) proves a non-reserved
dimension with `min(claimed, ledger._current[dimension])` — an aggregate read
of the caller ledger, not a proof that *these* bytes were charged. With the
`m9` defect shape present **and** at least `amount` unrelated `temporary_bytes`
concurrently held on the same caller ledger, the boundary is satisfied and the
projection is allocated with no charge of its own:

```text
(e) exact dim removed + 4096 unrelated temporary held:
    outcome=None  allocated=[37]  residual={'temporary_bytes': 4096}
    -> the projection (352 B) was allocated with NO temporary charge of its own
```

That preload is reviewer-supplied and I could not reach it from production.
Instrumenting the boundary shows no slack anywhere: all three caller-owned
routes observe exactly `352` of `352` live temporary bytes, and the live
`handle_http_upload` route uses `AccountedHTTPUploadSealedValidation`, whose
whole temporary upper is genuinely pre-acquired on the request ledger, so its
self-witnessing is sound.

```text
validate_sealed_commit:                        (_AccountedUploadSourceMemory, temporary_bytes, 352, 352)
validate_remote_json_publication_identity:     (_AccountedUploadSourceMemory, temporary_bytes, 352, 352)
preflight_remote_json_publication:             (_AccountedUploadSourceMemory, temporary_bytes, 352, 352)
handle_http_upload (200):                      (AccountedHTTPUploadSealedValidation, temporary_bytes, 120, 598424)
```

So the witness is currently exact in every reachable configuration.
Strengthening it to a delta measured across this owner's own charge (rather
than an aggregate level read) would make the property independent of what else
the request happens to hold. Recorded as a hardening item, not a blocker.

## BLOCKER-1 and HIGH-3 — preserved CLOSED

Re-run on the exact candidate:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_completed_quarantine_recovery_owns_the_request_budget \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_dropping_the_recovery_handoff_publishes_an_expired_retry \
  tests.test_python_http_upload_accounted_traversal.PythonHTTPUploadAccountedTraversalTests.test_publication_root_proof_is_atomically_single_use -v
# Ran 3 tests in 2.421s -- OK
```

Both closures are also exercised inside the green 31-test focused suite. The
recovery handoff (`ledger`/`deadline` into
`recover_remote_pending_quarantine`) and the
`available/consuming/consumed/revoked` proof state machine are unchanged by
this candidate's diff, and the prior lane's mutation evidence for them
(`m7`, `m8`) is not disturbed.

## Mutation Matrix

Each mutation was applied alone to a clean detached export at `/tmp/mut`, run
against one named focused test, reversed, and the tree proved byte-identical to
the candidate baseline `codefire` SHA-256
`63a9be232d064afa022432537f0686e57d47845b1f5654b929a69acec7a872fa` before the
next row.

| Row | Exact edit | Focused test | Result | Restored |
|---|---|---|---|---|
| `mA` | remove the `_charged_allocation_boundary(...)` call from `_charged_ordered_projection` | `test_ordered_projection_allocation_requires_a_proved_charge` | exit `1`; the removed-charge shape now reports the late `upload commit source memory claim underflow` instead of the boundary error | MATCH |
| `mB` | restore the pre-repair erase-before-release ordering in `_AccountedUploadSourceMemory.close` | `test_control_release_is_retryable_and_exactly_once` | exit `1`; `3` failures, one per public path, all `assertTrue(owner._acquired)` | MATCH |
| `mC` | remove the ordered-projection `_acquire_control` (keep list/sort) | `test_ordered_projection_allocation_requires_a_proved_charge` | exit `1`; healthy case raises the boundary error `0 of 352 temporary_bytes proved` — refusal is **pre**-allocation | MATCH |
| `mD` | `EXACT_DIMENSIONS = frozenset()` | `test_ordered_projection_allocation_requires_a_proved_charge` | exit `1`; same pre-allocation boundary error | MATCH |

| Row | Mutated SHA-256 | Restored SHA-256 |
|---|---|---|
| `mA` | `33de5582a2558e2782670a5e8e0a6a06c34696e7be95b472cb841031d4ec763e` | `63a9be23…a872fa` |
| `mB` | (in-place, single hunk in `close`) | `63a9be23…a872fa` |
| `mC` | `629aad64a3b85b1ffaa20f02f4ca9f08b22f2bad58253702dce8b4667449495d` | `63a9be23…a872fa` |
| `mD` | `0af67f9d10151691ad446f274462c6a9597e9cad1f20a70468be5eb79bbd2ace` | `63a9be23…a872fa` |

`mC`/`mD` are the two rows the prior review found vacuous. They now fail
pre-allocation, and `p3_boundary.py` independently asserts `allocated == []` for
both shapes, so the oracle is non-vacuous in the sense the ticket requires.

## Gates

All run on the exact candidate worktree, clean before and after.

| Command | Result |
|---|---|
| `python3 -m py_compile codefire tests/test_python_http_upload_accounted_traversal.py tests/test_python_sealed_production.py tests/test_codefire_cli.py` | PASS |
| `python3 -m unittest tests.test_python_http_upload_accounted_traversal -v` | **31/31 OK**, `81.296s` |
| `python3 -m unittest discover -s tests -p 'test_python_*.py'` | **452/452 OK**, `275.842s` |
| `python3 docs/development/tools/check_source_docs.py` | PASS — 44 modules, 34 commands, 12 object kinds |
| `python3 docs/development/tools/check_project_docs.py` | PASS — 532 Markdown files |
| `git diff --check` | PASS |
| `git status --short` | clean |

No Cargo/Rust, Docker, install, dogfood, or repository-wide validation was run
in this lane, and no Codex or subagent was used. All probes were checked-in-style
test-local Python scripts driven by small shell commands.

## Documentation Accuracy

Two statements added by the candidate over-state the mechanism and should be
corrected alongside the `HIGH-2` repair:

- `docs/development/source-architecture.md`: "Every accounted collection
  allocation on this route passes through `_charged_allocation_boundary`". The
  boundary has exactly one call site in the build — `_charged_ordered_projection`
  (`codefire:21822`). The same over-claim appears in the method's own docstring
  at `codefire:21794`. Other accounted collections on this route (descriptor
  registry, subdirectory map, pair-plan/pair-builder collections) do not pass
  through it.
- `docs/development/source-architecture.md`: "Control releases are retryable and
  exactly once." True for `_AccountedUploadSourceMemory` and
  `SealedCommitValidationSession._release_control_scalar`; false for the
  `_AccountedUploadCommitSource` wrapper on the two live upload routes, per the
  evidence above.

The `source-data-model-catalog.md` row for `_AccountedUploadSourceMemory` is
accurate as written, because it scopes its claim to that owner.

## Same-Root Shared-Owner Residual Search

A build-wide sweep for the defect shape "consume the handle or the scalar
before the fallible release" returns exactly two sites, both in
`_AccountedUploadCommitSource`:

```text
7290: self._memory = None    -->  7291: memory.close()
7309: self._session = None   -->  7314: session.close(None)
```

`SealedCommitValidationSession._release_memory_scalar` already uses the
before/after commit-detection pattern, and `AccountedHTTPBundle.close` releases
before clearing and carries a `__del__`. So the residual is localized: the
repair's transition is correct and the two unrepaired call sites are the only
ones sharing its root.

## Strongest Concerns and Required Next Action

1. **`HIGH-2` blocking.** `_AccountedUploadCommitSource.close` and `_open`
   discard the retry handles before the fallible cleanup, on the two live
   HTTP-upload commit-source routes. Reproduced with a realistic single
   `close(2)` failure and no ledger fault injection at all: `11,456` retained
   bytes permanently stranded on the live request ledger, `terminal_reason =
   None`, retry a silent no-op, and a raw `OSError` out of
   `handle_http_upload`.
2. **`HIGH-2` supporting.** No production caller can reach the owner that holds
   the retry, on any of the three repaired public APIs; there is no `__del__`
   fallback; and the checked-in regression reaches the owner only through a
   monkeypatched subclass.
3. **`HIGH-2` supporting.** Constructor-unwind precedence in `_open` and in
   `validate_sealed_commit` still replaces the primary decision with the
   cleanup fault.
4. **Non-blocking.** The `_control_charge_witness` aggregate read, and the two
   documentation over-claims.

`MEDIUM-4`, `BLOCKER-1` and `HIGH-3` need no further work. Closing item 1
(with 2 and 3, which share its root and its fix) plus regressions driven by a
`close(2)` failure rather than a patched `ledger.release` is what this
candidate needs before it can be accepted.

## Verdict

`REQUEST_CHANGES`
