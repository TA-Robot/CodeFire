# CFR-168 Python HTTP Merge-Request Mutation Lane

Adopt one request-wide decision-work/deadline/publication lane for Python HTTP merge-request
create, review, and apply on immutable current-state snapshot
`e2a218e015953f082245f36dbc6aee906b837e5a`.

Use Claude Opus 5 (`claude-opus-5`). Work only in the assigned worktree and write set. Do not
change issue status, edit canonical management documents, modify the upload/GC/list/Doctor routes,
or import the rejected `aa5e57d` Python line wholesale.

## Production defect

The three live mutating MR routes do not receive the handler's `http_v1`
`DecisionWorkLedger`/`HTTPRequestDeadline`. They also build the success response only after the
handler has committed durable state. A response-body/header/retained limit can therefore fail after
publication, and the route work is absent from the request ledger.

The upload route already demonstrates `_HTTPMutationResponseSlot` and request-owned response
preflight. Reuse its stable public contract where appropriate, but do not entangle or regress the
upload implementation.

## Required scope

- `POST …/merge-requests` (create);
- `POST …/merge-requests/{id}/review`;
- `POST …/merge-requests/{id}/apply`.

## Required behavior

- Thread the exact handler request ledger and deadline from `do_POST` into all three live handlers
  and every selected filesystem/signature/validation/ancestry/publication phase.
- Never create a nested `local_v1` ledger or reopen work in an unaccounted compatibility fallback.
- Preflight/freeze the exact success response and every durable/recovery response before the first
  effect, using request-owned response/body/header/retained accounting.
- Preserve the existing authorization, idempotency/recovery, status/error precedence, response
  JSON bytes, MR subject binding, approval semantics, fast-forward result, audit records, and
  success-side file contents.
- Create/review/apply must each use a pinned subject and final revalidation appropriate to their
  authorities. Apply's signature history and ancestry must be iterative, goal-directed, charged,
  deadline-polled, and bound to the same request ledger.
- Poll deterministic phase boundaries through response preflight, gate acquisition, recovery,
  validation, signature/ancestry, final fence, publication, and response handoff.
- Any budget/deadline/final-fence/encoding failure must be typed and leave branch, generation, MR,
  object, nonce, idempotency, transaction, audit, and recovery state byte-identical to pre-call
  state. Publish no partial HTTP success.
- Replay must return the immutable recorded result without repeating validation or mutation while
  still observing request/response accounting and deadline rules.
- Cleanup must release only this lane's ownership and leave the request header ownership for the
  outer handler. No open permit or concurrent debt may remain on success or failure.

## Required production-path tests

Add a dedicated test module that drives real `serve` HTTP requests and the live handlers:

- exact request ledger identity and absence of any second ledger for all three routes;
- baseline success/replay byte equivalence and unchanged status/error precedence;
- response-body, response-header, retained, object/file/byte, node/edge/depth/frontier,
  signature/ancestry, and snapshot-revalidation exact / minus-one / plus-one boundaries where
  applicable;
- deterministic deadline expiry at each named phase;
- 1x/3x/10x counter curves for create/review/apply selected work;
- deep/shared-DAG apply with an immediate ancestor proving early goal-directed exit;
- same-path/same-size/same-mtime subject tamper at the final fence;
- pre/post filesystem snapshots proving no effect for every typed failure;
- one terminal reason, zero open permits, and zero route-owned concurrent debt.

Tests must fail against `e2a218e` for the reproduced production defect and exercise live routing,
not only owner scaffolding. Avoid nonzero-only or tautological metric assertions.

## Validation

At minimum run:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_mr_mutation_decision_work -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_codefire_cli.CodeFireCliTests.test_http_merge_request_escrow_survives_source_project_removal
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  codefire tests/test_python_http_mr_mutation_decision_work.py
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Locate and run the exact neighboring MR tests, then run the full Python suite. Commit the
implementation and leave the worktree clean.

## Handoff

Report each live call chain, exact ledger/deadline identity, response-slot handoff, pre-effect
authority order, tested boundaries/counter curves, no-effect/tamper/replay observations, full test
counts, and every remaining Python HTTP route outside this packet.
