# CFR-168 / CFB-138 Python HTTP Upload — HIGH-2 Third Repair

Implementer: Claude Opus 5 (`claude-opus-5`), original implementer session
`7baa1275-0ee2-476a-af46-1907386ebceb`, resumed in its existing isolated worktree.

| Field | Value |
|---|---|
| Base (reviewed candidate) | `02e04c7e6458035d9c7d58de4d30b4a12e7cd594` |
| Report answered | `b3de0d7:docs/development/ai/swarm-results/cfr168-python-http-upload-second-review-closure-rereview-claude-fallback-20260731/review.md` |
| Report verdict | `REQUEST_CHANGES` — `HIGH-2` open; `BLOCKER-1`, `HIGH-3`, `MEDIUM-4` closed |

**Implementer evidence only. No acceptance is claimed.** This lane did not review its own candidate.
A new independent reviewer is requested.

## Scope

Narrowly bound third repair. Only the cleanup-ownership defect the report evidenced, at the two
sites its build-wide sweep returned, plus the primary-error precedence and reachable-recovery items
that share their root. No accounting was weakened, no ledger bypassed, and no other behaviour
touched.

## Reproduced before the repair

Against base `02e04c7`, with **no `DecisionWorkLedger.release` patched anywhere** — a real
`close(2)`-shaped failure (the descriptor *is* closed, the call still raises `OSError(EIO)`), armed
only while `_AccountedUploadCommitSource.close` is on the stack:

| Case | Observed on `02e04c7` |
|---|---|
| live fast-forward ancestry upload | raw `OSError` out of `handle_http_upload`; residual `{'retained_bytes': 11456}`, permits `False`, terminal `None` |
| live commit-signature history upload | same |
| direct `_AccountedUploadCommitSource.close` | handles `(session=None, memory=None)`; `source.close()` retry returns `None`; residual `11,456` after `gc.collect()` |
| `_open` constructor unwind | caller sees `RuntimeError: SECONDARY cleanup fault` instead of `ValueError: PRIMARY constructor decision`; residual `11,456` |

## Fixed after the repair

Same probe, same injection, against this candidate:

| Case | Observed now |
|---|---|
| live fast-forward ancestry upload | `OSError` still surfaces (the failure is real); residual `{0, 0}`, permits `False` |
| live commit-signature history upload | residual `{0, 0}`, permits `False` |
| direct `_AccountedUploadCommitSource.close` | handles **kept** (`session` and `memory` both live); retry `close()` succeeds; residual `{0, 0}`; a third close releases nothing again |
| `_open` constructor unwind | caller sees **`ValueError: PRIMARY constructor decision`**; handle kept; `source.close()` retry drains to `{0}`; dropping the source without a retry drains to `{0}` via the finalizer |

## Changes

| Path | Change |
|---|---|
| `codefire` | `_AccountedUploadCommitSource.close` drops each handle only once *its* cleanup succeeded, and does not attempt the reservation while the session teardown still owes control debt; `_open` and `validate_sealed_commit` constructor unwinds keep primary-error precedence; `_AccountedUploadCommitSource.__del__` and `_AccountedUploadSourceMemory.__del__` added as idempotent finalizers; one docstring over-claim corrected |
| `tests/test_python_http_upload_accounted_traversal.py` | `test_commit_source_cleanup_survives_a_failing_close_syscall`, `test_commit_source_constructor_unwind_keeps_the_primary_error` |
| `docs/development/source-architecture.md` | both over-claims the report named corrected; the repaired wrapper contract recorded |
| `docs/development/command-source-trace.md` | the retryable commit-source cleanup recorded |
| `docs/development/ai/swarm-results/cfr168-python-http-upload-high2-third-repair-20260731/result.md` | this record |

## Regressions

Both new tests drive a real `close(2)` failure, not a patched `ledger.release`, on **both** live
commit-source routes, and both **fail on the reviewed candidate `02e04c7`** (3 failures / 1 error)
and pass here. They assert: the first failure surfaces with `errno == 5`; the request ledger reaches
zero retained/temporary debt with no open permits; no durable authority byte changed and the branch
did not move; the handles survive the fault; the retry completes and is idempotent; and the
constructor unwind preserves the primary decision on both `_open` and `validate_sealed_commit`, with
recovery reachable by explicit retry and by the finalizer.

## Invariants held

- `BLOCKER-1`, `HIGH-3` and `MEDIUM-4` behaviour unchanged and green inside the 33-test focused suite.
- Accounting not weakened: no new bypass, no reservation removed, no limit raised. The reservation is
  still refused while control debt is outstanding — the repair changes *reachability of the retry*,
  not what is owed.
- Exactly-once preserved: repeated `close` after a successful release releases nothing again.
- Release still fails loudly; the finalizer only ever completes work genuinely still owed.

## Gates

| Command | Result |
|---|---|
| `py_compile codefire tests/test_python_http_upload_accounted_traversal.py tests/test_python_sealed_production.py tests/test_codefire_cli.py` | PASS |
| `unittest tests.test_python_http_upload_accounted_traversal` | **33/33 OK**, 98.743s |
| `unittest` ancestry / signature / signature-terminal-freeze / sealed-production / external-production | **59/59 OK**, 25.427s |
| `unittest discover -s tests -p 'test_python_*.py'` | **454/454 OK**, 203.036s |
| `check_source_docs.py` | PASS — 44 modules, 34 commands, 12 object kinds |
| `check_project_docs.py` | PASS — 532 Markdown files |
| `git diff --check` | PASS |

No Rust/Cargo, install, dogfood, Codex, or other agents.

## Remaining gaps

- The report's non-blocking hardening item is **not** addressed: `_control_charge_witness` still
  proves a non-reserved dimension with `min(claimed, ledger._current[dimension])`, an aggregate read.
  The reviewer could not reach the weak configuration from production and recorded it as hardening;
  this repair was bound to the cleanup-ownership defect, so it is left open deliberately.
- Recovery after a *persistently* failing cleanup depends on a later retry or on the finalizer; there
  is still no public surface that enumerates outstanding reservations for a caller to drain
  explicitly.
- The finalizer's timing is CPython refcount behaviour, not a guarantee of the language.
- The composition conflict with the active `cfr168-python-http-mr-list-review-closure-repair` lane
  persists in `execute_remote_json_publication` / `recover_remote_pending_quarantine` and now also in
  `_AccountedUploadCommitSource.close`.
- The 13 selected legacy CLI cases were not re-run in this iteration; the broad Python sweep and the
  adjacent owner suites were.

## Request

New independent review of the exact candidate below. No acceptance, integration, issue closure,
capability completion, or zero-residual claim is made from this implementer evidence.
