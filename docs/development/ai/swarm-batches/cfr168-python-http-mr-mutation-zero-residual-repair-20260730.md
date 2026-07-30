# CFR-168 Python HTTP MR Mutation Zero-Residual Repair

Repair exact candidate `21fe75175e997cc7c6f8c3ef2a7558144d850f94` against every finding and
MR-mutation-local residual in
`docs/development/ai/swarm-results/cfr168-python-http-mr-mutation-review-claude-fallback-20260730/review.md`.
Use Claude Opus 5 (`claude-opus-5`). Work only in the assigned worktree and write set. Add new
commits on top; do not amend the candidate or independent report.

The verdict is `REQUEST_CHANGES`. All Blocking/Medium/Low findings are required closure work. The
visible request-ledger/response-slot architecture is worth preserving, but the route is not
request-accounted while sealed validation, authorization, nonce-cache materialization, collection,
copying, and legacy subject materialization escape into nested ledgers or unaccounted helpers.

## Required architecture and behavior

1. One exact handler-created `http_v1` ledger and `HTTPRequestDeadline` must own every create,
   review, and apply phase:
   - request parsing, policy/auth/signature and nonce-cache reads;
   - sealed validation, object collection/copy, signature subject and escrow validation;
   - ancestry, final fence, response preparation, publication/recovery, and cleanup.
   No live route may construct a nested `local_v1` or other compatibility ledger.
2. Replace every live `validate_sealed_commit` escape with a caller-owned
   `SealedCommitValidationSession` bound to the request ledger and request poll. Reuse one validated
   snapshot/session per root across authorization, MR validation, identity preflight, final
   revalidation, collection, materialization, and publication. Do not validate or physically read
   the same closure again through `collect_object_records`, `copy_object_graph`, or `read_json`.
3. Make authorization input bounded and accounted before read/allocation:
   - `server_policy.json`, key material, signature subject, and nonce cache;
   - every `Path`, raw byte buffer, parsed object, list/dict/set entry, descriptor, projection, and
     canonical encoding;
   - the nonce cache must not be fully materialized twice on success.
   Thread the deadline poll into every iterative record and cache step.
4. Tighten production non-vacuity: the ledger-identity test must reject every nested ledger, not
   merely a second `http_request` ledger. Instrument hidden record reads, aggregate bytes,
   collection, copy, and JSON reads so an unowned operation fails the focused suite.
5. Repair legacy subject-less review ordering:
   - all deterministic publication/sealed-validation preflight must finish before escrow
     materialization;
   - otherwise stage materialization in a discardable transaction whose complete pre-call object
     snapshot is restored on rejection;
   - preserve the original typed 4xx diagnostic and precedence instead of converting a permanent
     failure into endless `503 retry_same_request`;
   - every typed pre-effect failure must leave objects and all authority byte-identical.
6. Complete nonce recovery/replay semantics. A replay request whose signature presents a fresh nonce
   must not be accepted and then silently discard that nonce. Either atomically record the accepted
   nonce with the authoritative replay transaction or reject it before returning the recorded
   result. Preserve exact idempotent result bytes, signature skew/key-window rules, nonce capacity,
   and no-partial-publication behavior. Prove a captured signed replay is not reusable without bound.
7. Translate same-path grow/shrink/replacement failures inside
   `AccountedHTTPMergeRequestFence` into the typed `merge_request_fence_changed` diagnostic with
   expected/actual evidence. A generic bounded-limit error may be used only for an input already
   beyond the public bound at initial capture.
8. Make all subject validation and ancestry precedence goal-directed. An inadmissible status/stale
   request must not pay unbounded off-goal hidden work, while legacy corruption/error precedence
   remains exact.
9. Recover the exact request ledger from every session discard, double-close, observer, response
   selection, publication, recovery, and cleanup failure. Emit one terminal observation, preserve
   the primary typed outcome, and finish with zero debt and zero permits.

## Required reversal and scale evidence

- Reproduce the reviewer tables for create/review/apply at 1x/3x/10x and prove:
  one request ledger, zero nested ledgers, zero unaccounted collection/copy/JSON reads, and stable
  linear counters.
- Exact / minus-one / plus-one boundaries must cover every charged dimension and the newly owned
  authorization, nonce, validation, collection, copy, projection, response, and publication owners.
- Full pre/post snapshots must include object records, branches, generations, MR records, nonce
  cache, transaction/pending/quarantine/receipt/audit state, and exact response bytes.
- Add mutation checks that fail when:
  a nested sealed-validation ledger is restored, any poll is removed, an authorization/cache owner
  is omitted, publication preflight is moved after legacy materialization, or replay again drops a
  fresh nonce.
- Preserve the focused module's failure against `e2a218e…`; additionally prove all new regressions
  fail against `21fe751…`.

## Invariants

- Do not raise a public profile limit, weaken sealed validation/signature/ancestry/fence checks,
  hide an allocation from instrumentation, or classify a deterministic permanent failure as
  retryable.
- Create/review/apply retain exact success, error, durable-retry, idempotency, and cross-runtime
  semantics except for the required typed corrections above.
- Parse, authorize, validate, plan, preflight, apply, render, write, and cleanup remain explicit.
- Success and every failure end at zero retained/temporary debt and zero open permits.
- `POST .../gc` remains a separately tracked successor packet; do not claim uniform POST nonce
  timing until that packet is complete.

## Required validation

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  codefire tests/test_python_http_mr_mutation_decision_work.py tests/test_codefire_cli.py
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_mr_mutation_decision_work -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p 'test_python_*.py'
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Run the exact neighboring legacy MR/signature/nonce/replay/escrow/branch-protection cases named in
the review. The full Python suite and Cargo-dependent install cases are manager-serialized after the
upload predecessor is composed. Report exact hashes, counts, counter curves, owner formulas,
reversal results, and integration conflicts. Commit only intended product/tests/source docs and
leave the worktree clean.
