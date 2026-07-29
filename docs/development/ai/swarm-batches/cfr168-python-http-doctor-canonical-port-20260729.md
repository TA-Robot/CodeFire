# CFR-168 Canonical Python HTTP Doctor Ledger Port

Port the independently tested Doctor request-ledger slice onto immutable current-state snapshot
`e2a218e015953f082245f36dbc6aee906b837e5a`.

Use Claude Opus 5 (`claude-opus-5`). Work only in the assigned worktree and write set. Do not merge
the `aa5e57d` Python candidate line, copy its frozen alternative reducers, change issue status, or
edit canonical management documents.

## Source prototype

The source implementation is the committed tip of:

```text
claude-agent/cfr168-python-http-doctor-ledger-20260729-doctor-request-ledger
```

Use its `codefire` delta and `tests/test_python_http_doctor_decision_work.py` as a reviewed
prototype, then reapply/review them against this canonical snapshot. The manager has already
verified that the current code delta is textually applicable to `e2a218e`; semantic and regression
validation are still required here.

## Required implementation

- Pass the exact request `DecisionWorkLedger` and `HTTPRequestDeadline` through
  `do_GET -> handle_http_doctor -> remote_doctor_payload -> _remote_doctor_payload ->
  _remote_doctor_payload_owned`.
- Never create a nested `local_v1` ledger after a request ledger was supplied. Preserve the
  deliberate direct/file/library compatibility path that owns a local ledger.
- Poll deterministic Doctor phase boundaries through snapshot revalidation and response
  encoding/header/publication.
- Preserve quick/full result, ordering, JSON bytes, diagnostics, direct compatibility, cleanup,
  and request-owned terminal state when no limit/deadline expires.
- Budget/deadline failure must release only Doctor's acquisitions, preserve pre-existing request
  ownership, and publish no partial response.

## Additional reproduced defect to close

On a corrupted object record, `doctor_session.scan_object_hash_errors()` returns `Path` values.
The production path currently calls `canonical_json_size([bad_paths, ...])` before converting those
paths to strings, which raises `TypeError: PosixPath is not JSON serializable`. Fix this on the
canonical path:

- construct one stable string projection for `bad_objects` before accounting;
- charge the exact serializable projection that is retained/projected;
- preserve issue ordering, subject strings, counts, and text/JSON payloads;
- prove corrupted-object Doctor returns the intended structured unhealthy report, never an
  untyped exception, in direct and HTTP full mode;
- include exact/minus-one retained accounting and zero-debt cleanup for the non-empty diagnostic.

## Required tests

Port and retain all production-path tests from
`tests/test_python_http_doctor_decision_work.py`, including exact ledger identity, second-ledger
absence, quick/full equality, exact/-1 limits, deterministic deadline phases, and cleanup.

Add the corrupted object path regression described above. It must exercise the live direct and
HTTP handlers, assert stable JSON-serializable string paths, typed status/diagnostics, and no
ownership debt.

## Validation

At minimum run:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_doctor_decision_work -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_doctor_reports_corruption \
  tests.test_codefire_cli.CodeFireCliTests.test_http_remote_doctor -v
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire \
  tests/test_python_http_doctor_decision_work.py
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Locate the exact neighboring test names if they differ. Run the full Python suite after focused
tests. Commit the implementation and leave the worktree clean.

## Handoff

Report the canonical production call chain, source-prototype commit, tests/counts, exact/-1 and
deadline observations, corrupted-object regression, cleanup/byte-equivalence evidence, and every
remaining P0 surface.
