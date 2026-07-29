# CFR-168 Canonical Python HTTP Upload Accounted Traversal

Implement this P0 production slice directly on immutable current-state snapshot
`e2a218e015953f082245f36dbc6aee906b837e5a`.

Use Claude Opus 5 (`claude-opus-5`). Work only in the assigned worktree and write set. Do not
merge any `aa5e57d`/alternative-design Python line, change issue status, weaken assertions, or
redesign unrelated routes.

## Reproduced defects

The independent current-state review reproduced both defects in the live Python upload route:

1. `handle_http_upload -> is_ancestor_from_objects -> ancestor_distances_from_objects` builds the
   entire reachable-history distance map without the request ledger or deadline. A 300-commit
   immediate-parent decision performed 300 uncharged object loads.
2. `handle_http_upload -> enforce_remote_commit_signature_policy` traverses signature history
   without the request ledger or deadline. Accounted ancestry/signature owners exist in this
   snapshot but have no production constructor/caller.

## Required implementation

- Pass the exact request-owned `DecisionWorkLedger` and `HTTPRequestDeadline` from the live upload
  handler into fast-forward ancestry and signature-policy history.
- Reuse or simplify only current-snapshot accounted primitives whose semantics match production.
  Do not import the later frozen response-bridge/reducer/preimage design family or add another
  production-unwired sidecar.
- Implement a goal-directed ancestor predicate: stop when the sought ancestor is found; never
  materialize a full distance map for a boolean decision.
- Before work/allocation, charge object reads/bytes, traversed edges/nodes, signature records,
  frontier/seen retained state, and terminal scalar/output ownership. Poll the exact deadline at
  deterministic phase and loop boundaries.
- Preserve no-limit decision, diagnostic precedence, traversal order, signature policy, response
  bytes, and publication semantics.
- Budget/deadline failure must occur before branch/object publication and leave no staged object,
  retained/temporary state, response slot, permit, or terminal debt.
- Remove or bypass dead owner code only when tests prove equivalent current-snapshot semantics.
- Do not claim Doctor, branch/MR/GC, Rust local/file transport, metrics, or other P0 surfaces fixed.

## Required production-path tests

Create `tests/test_python_http_upload_accounted_traversal.py` and exercise the real upload route:

- exact request ledger/deadline identity reaches ancestry and signature history;
- immediate-parent fast-forward on a 300-commit/deep fixture stops goal-directed rather than
  scanning all history;
- deep/shared DAG traversal is iterative, deterministic, and charged under one owner contract;
- exact and exact-minus-one (or exact/+1 observation) budgets return the typed resource outcome
  before mutation with complete counter/limit/current fields;
- deterministic deadline expiry in both phases fails before mutation and releases all ownership;
- required/optional signature policies retain current decisions and diagnostic precedence;
- the review's uncharged-read and dead-owner observations fail on the old production path and pass
  only through the new live path.

Use injected clocks; never sleep on wall time.

## Validation

At minimum run:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_upload_accounted_traversal -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_ancestry_owner \
  tests.test_python_signature_owner \
  tests.test_python_signature_terminal_freeze \
  tests.test_python_sealed_production \
  tests.test_python_external_production -v
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire \
  tests/test_python_http_upload_accounted_traversal.py
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Commit the implementation and leave the worktree clean.

## Handoff

State the exact live call chain sharing the ledger/deadline, before/after deep-history counts,
focused test counts, publication-atomicity evidence, dead code retained/removed with reason, and
all remaining CFR-168/CFB-136 surfaces.
