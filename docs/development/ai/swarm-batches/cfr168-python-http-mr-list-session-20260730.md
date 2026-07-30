# CFR-168 Python HTTP Merge-Request List Session

Close the remaining Python `GET /v1/projects/{org}/{app}/merge-requests` request-ledger gap after
the branch-list session result. Use Claude Opus 5 (`claude-opus-5`). Work only in the assigned
worktree and write set. Do not modify MR mutation routes, GC, Rust, issue state, management
ledgers, or public budget defaults.

## Production defect

`do_GET` currently calls `handle_http_request_list(storage_root, org, app)` without the handler's
`DecisionWorkLedger` or `HTTPRequestDeadline`. The live function then:

- builds a plain sorted `glob` list;
- loads merge-request and target-branch JSON through unaccounted helpers;
- calls `validate_http_merge_request_record`, which opens independent sealed-validation work per
  reference;
- appends a plain unowned response list;
- performs no deterministic deadline polls or final snapshot fence before response publication.

This is one of the independently reviewed CFR-168 production-surface blockers.

## Required behavior

- Thread the exact request ledger and deadline from `do_GET` through one MR-list operation owner,
  authority capture, JSON reads, URL/subject validation, sealed-reference validation, projection,
  final revalidation, response encoding/header/publication, and cleanup.
- Capture MR authority and each consulted branch authority with bounded no-follow accounted reads.
  Preserve the existing deterministic MR filename order, status calculation, error precedence,
  legacy/cross-project rejection, source/target URL checks, JSON bytes, and 404/400 behavior.
- Reuse one request-ledger-bound `SealedCommitValidationSession` per distinct pinned object-root
  subject. Do not call the legacy independent `validate_sealed_commit` path from the live route.
  Finalize every session only after the complete projection is prepared and before publication.
- Account every retained path, parsed record, validation frontier/memo, projected field/list, and
  response byte before allocation/publication. All owners must close with zero current debt and no
  open permit.
- Add an exact MR-list response phase to the shared read-route phase dispatcher and reset its flag
  for every request. A deadline/budget/subject change must publish only the existing typed terminal
  diagnostic, never a partial list.
- Do not weaken validation, raise a default, drop cross-project compatibility checks, or materialize
  an unbounded side list as a shortcut.

## Required production-path tests

Create `tests/test_python_http_mr_list_decision_work.py` and drive the real socket handler:

- exact request ledger/deadline identity; no nested ledger or legacy sealed validator;
- absent/empty/one/many MR authority and deterministic compatible bytes/order;
- shared source/target DAG validation once per object-root session with exact counters;
- open/stale status parity and legacy/cross-project/error-precedence parity;
- same-path/inode/size/mtime MR, branch, and reachable-record mutation before the final fence;
- files/bytes/records/nodes/edges/retained/temporary/response exact, minus-one, and plus-one;
- deterministic deadline expiry at authority, validation, projection, final revalidation,
  response encoding, headers, and publication;
- malformed/unparseable/future/invalid sealed references fail closed with no partial response;
- every success/failure closes all sessions, permits, and retained/temporary ownership;
- prove the pre-fix target reaches the unaccounted/legacy path.

Retain the Doctor, branch-list, bundle, upload, and MR mutation neighboring behavior.

## Validation

At minimum run:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire \
  tests/test_python_http_mr_list_decision_work.py
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_mr_list_decision_work -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests \
  -p 'test_python_*.py'
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Commit the implementation/tests, leave the worktree clean, and report the call chain, ledger and
session identities, exact/deadline/no-effect evidence, compatibility results, and residual routes.
