# CFR-168 Python HTTP Upload Independent Review

Independently review the completed Python HTTP upload ancestry/signature lane. Use Fugu 1.1 Ultra
(`fugu-ultra-v1.1`). This is a review-only packet: do not modify product code, tests, issue state,
or management ledgers.

## Exact review subject

- Completed target: `cb13da08e82c2502920f735d27617e61f978527a`
- Product baseline: `5254e3dc8e3cbab3330c2a34469eb56750edf626`
- Interrupted production commit included in the target:
  `3153d75a20414a12a37fcce847b79e13d0616c75`
- Recovery test commit included in the target:
  `cb13da08e82c2502920f735d27617e61f978527a`
- Contract packet:
  `docs/development/ai/swarm-batches/cfr168-python-http-upload-traversal-canonical-20260729.md`
- Review the cumulative diff
  `5254e3dc8e3cbab3330c2a34469eb56750edf626..cb13da08e82c2502920f735d27617e61f978527a`,
  restricted to `codefire` and
  `tests/test_python_http_upload_accounted_traversal.py`.

The target branch is
`claude-agent/cfr168-python-http-upload-recovery-20260729-upload-fix-three-tests`.
Do not infer acceptance from the implementation agent's report or from test names.

## Required review

Establish from live production paths and decisive tests whether all of the following are true:

1. HTTP upload ancestry discovery and legacy signature-history traversal use the one request
   `DecisionWorkLedger` and request deadline; no nested or unbudgeted traversal remains.
2. Frontier/seen/list growth is reserved before allocation, recursion is absent, work is
   goal-directed where valid, and shared-DAG accounting has the documented meaning.
3. Response and staged-object owners are prepared before the first durable effect. Budget,
   deadline, subject-change, signature, and publication failure leave repository state,
   transaction/idempotency state, and response behavior atomic.
4. Exact/minus-one/plus-one tests reach the intended production owner rather than failing in an
   unrelated predecessor. Deep and shared-DAG fixtures are non-vacuous and their expected counters
   follow from the implementation.
5. Existing success bytes, diagnostic precedence, signature policy semantics, replay behavior, and
   zero-debt cleanup are preserved.
6. The recovery-only test changes do not relax a product requirement merely to turn a red test
   green.

Run at least the focused upload test file, `git diff --check`, and Python compilation. Inspect the
actual cumulative diff and relevant shared owners. Additional focused neighboring tests are
encouraged when they can disprove the contract.

## Output

Write exactly one report:

`docs/development/ai/swarm-results/cfr168-python-http-upload-independent-review-20260730/review.md`

The report must contain:

- exact reviewed baseline and target hashes;
- `ACCEPT` or `REQUEST_CHANGES`;
- findings ordered by severity, with file/line and concrete counterexample;
- an explicit verdict for each of the six required-review items;
- commands and results actually run;
- any residual risk or follow-up that belongs to a different frozen packet.

If any required claim lacks direct evidence, return `REQUEST_CHANGES`; do not silently narrow the
packet. Commit only the report and leave the reviewer worktree clean.
