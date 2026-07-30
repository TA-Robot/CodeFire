# CFR-168 Python HTTP MR Mutation Independent Review

Independently review the completed Python HTTP merge-request create/review/apply decision-work lane.
Use Fugu 1.1 Ultra (`fugu-ultra-v1.1`). This is review-only: do not modify product code, tests,
issue state, or management ledgers.

## Exact subject

- baseline/task revision: `cab3126f216c595c0d2211174b364e8532a310e2`
- completed target: `21fe751`
- target branch:
  `claude-agent/cfr168-python-http-mr-mutation-recovery-20260729-mr-mutation-restart`
- cumulative paths: `codefire`, `tests/test_python_http_mr_mutation_decision_work.py`
- contract:
  `docs/development/ai/swarm-batches/cfr168-python-http-mr-mutation-lane-20260729.md`

Do not infer acceptance from the implementation agent's 18 focused passes or clean commit. The
reported nonce timing, legacy materialization ordering, and nested sealed validation are
load-bearing review subjects.

## Required review

1. Create, review, and apply use the exact handler request ledger/deadline through authorization,
   authority capture, escrow/signature/ancestry validation, response ownership, final fence,
   publication, replay, and cleanup. No lane-local `http_request` ledger or hidden legacy traversal
   may escape accounting.
2. Every retained collection, authority snapshot, record read, frontier insertion, digest, response
   byte/header, and permit is charged before allocation/read/growth. Exact/minus-one/plus-one and
   1x/3x/10x counters are deterministic and production-path non-vacuous.
3. Audit the nonce semantic change. A failed signed request no longer burns its nonce because nonce
   publication joins the transaction. Decide whether this is required atomicity, a compatibility
   regression, or an incomplete recovery/replay design; prove the answer with live retries.
4. Audit legacy subject-less review: response sealing precedes `materialize_merge_request_subject`,
   whose object copy is the first effect. Encoding/header/publication/preflight failures must be
   impossible after that first effect or recover to one durable byte-identical result.
5. Audit the remaining nested `validate_sealed_commit` local ledger and off-goal corruption
   precedence. The route must not claim one request-wide decision owner while performing material
   decision work in a hidden independent ledger.
6. Independently run the 18 focused tests, the affected legacy MR/request-signature/replay tests,
   Python compilation and diff checks. The reported broad suite's two Cargo-PATH failures are
   environment-only only if the same cases pass later with the configured Rust toolchain.

## Output

Write exactly:

`docs/development/ai/swarm-results/cfr168-python-http-mr-mutation-independent-review-20260730/review.md`

Include exact hashes, `ACCEPT` or `REQUEST_CHANGES`, severity-ordered findings with concrete
file/line evidence, a verdict for all six items, commands/results, required integration
predecessors, and residual risk. Commit only the report and leave the reviewer worktree clean.
