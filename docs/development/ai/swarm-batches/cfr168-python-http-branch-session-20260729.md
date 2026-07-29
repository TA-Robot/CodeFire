# CFR-168/CFB-136 Python HTTP Branch-List Request Session

Implement this P0 production slice directly on immutable current-state snapshot
`e2a218e015953f082245f36dbc6aee906b837e5a`.

Use Claude Opus 5 (`claude-opus-5`). Work only in the assigned worktree and write set. Do not
merge the later Python candidate line, change issue status, weaken assertions, or redesign
unrelated routes.

## Reproduced defect

The live Python `GET /branches` route calls `handle_http_list`, then validates every branch head
through an independent `validate_sealed_commit` cache. It passes neither the request
`DecisionWorkLedger` nor `HTTPRequestDeadline`, does not own one pinned branch-authority snapshot
plus one shared sealed-validation session, and performs response preparation only after the
handler returns. This is the production shape named by CFB-136.

## Required implementation

- Pass the exact request-owned `DecisionWorkLedger` and `HTTPRequestDeadline` through the live
  branch-list route and all branch authority/validation/projection work.
- Capture branch authority records under the current no-follow/project-gate contract. Account
  directory entries, record bytes/JSON, retained names/heads/state, and result projection before
  work/allocation.
- Use one request-scoped `SealedCommitValidationSession` for every head. Reuse immutable shared DAG
  validation within that session and perform one final snapshot/digest revalidation before response
  publication.
- A same-path, same-inode, same-size, same-mtime content change between capture/validation and the
  final fence must fail closed with the existing typed subject-changed diagnostic. Never authorize
  reuse from pathname/mtime alone.
- Poll deterministic authority, per-head, shared-DAG, final-revalidation, encoding, header, and
  publication boundaries.
- Preserve no-limit branch order, JSON/text-equivalent domain fields, diagnostics, status, and
  compatibility behavior.
- Budget/deadline/subject-change failure must publish no partial response and leave no retained or
  temporary debt, open session, permit, or hidden second ledger.
- Do not claim Doctor, MR/GC, upload ancestry/signature, Rust local/file transport, metrics, or
  other P0 surfaces fixed.

Reuse the current-snapshot Python session primitives where their live Store semantics match. Do
not create a test-only cache or another uncalled factory.

## Required production-path tests

Create `tests/test_python_http_branch_decision_work.py` and exercise the real handler:

- exact request ledger/deadline identity and absence of any nested local ledger;
- 24 or more heads sharing a deep DAG validate under one session, with stable order and shared work
  counters rather than one cold traversal per head;
- cold and reused-session results/diagnostics/order are identical;
- same-inode/size/mtime extension-only or same-length content mutation before finalization is
  rejected on the live route;
- exact and exact-minus-one (or exact/+1 observation) limits for authority records, object work,
  retained projection, and response encoding produce the complete typed resource outcome;
- deterministic deadline expiry at authority, validation, final revalidation, and response phases;
- empty, one-head, many-head, malformed authority, invalid sealed head, and shared-parent failure
  precedence;
- all failure paths produce no partial bytes and zero concurrent ownership debt.

Do not use wall-clock sleeps.

## Validation

At minimum run:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_branch_decision_work -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_codefire_cli.CodeFireCliTests.test_http_remote_branch_list \
  tests.test_codefire_cli.CodeFireCliTests.test_remote_branch_list -v
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire \
  tests/test_python_http_branch_decision_work.py
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Locate exact neighboring test names if they differ. Commit the implementation and leave the
worktree clean.

## Handoff

Report the live call chain, authority/session/revalidation ownership, cold/shared work counts,
same-metadata mutation result, exact/+1 and deadline evidence, focused test counts, and remaining
P0 surfaces.
