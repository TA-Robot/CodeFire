# CFR-168 Python HTTP MR-list Final-publication Closure Repair

Status: Ready

## Objective

Close every blocking and recorded residual in the exact independent review of
candidate `4930d25178be11e064060487f8c254802b09868d`, without weakening any of
the seven prior closures. The result must make the final publishability proof
effective on the production GET path, preserve typed post-route failures, and
make success-with-debt impossible.

Use the exact original Claude Opus 5 implementer session
`3dd468cb-d9a9-42be-b266-befde4fba6fd` in its clean original worktree.

## Exact subject and review

- Rejected product candidate:
  `4930d25178be11e064060487f8c254802b09868d`
- Review task base:
  `8143f437342cf7bf11322d82f6ca63ab888382e8`
- Immutable reviewer commit:
  `0aa0d3d0bfbaafcc9d67bccdc90207810a60fe4b`
- Report-only continuation commit in this worktree:
  `6ee3646`
- Canonical report commit:
  `909978c`
- Report blob SHA-256:
  `dc4f66ba9649e4333980996c6f83d7f28b0f5f4bd165683adac55bf6d9704b2e`
- Report patch SHA-256:
  `2848022a51e55fc449dafacacb3349ca4177387eb9b1176d6616d60a0403d7ce`
- Decision: `REQUEST_CHANGES`
- Report:
  `docs/development/ai/swarm-results/cfr168-python-http-mr-list-review-closure-rereview-claude-20260730/review.md`

Read the complete report before editing. It independently confirms that all
seven inherited findings are closed and then reproduces F1--F3 plus two
recorded cleanup/accounting observations. The report is immutable evidence,
not an editable task artifact.

## Scope

Allowed product/test/source-document files:

- `codefire`
- `tests/test_python_http_mr_list_decision_work.py`
- `docs/development/change-playbook.md`
- `docs/development/command-source-trace.md`
- `docs/development/source-architecture.md`
- `docs/development/source-data-model-catalog.md`
- this task packet

Do not edit the review report. Do not import unrelated Upload, MR-write,
Doctor, Rust, integration, KGI/KPI, or foundation changes.

## Required closure

### F1 — real final proof after teardown

The exact review observed five fallible operations after the claimed terminal
fence: `flock(LOCK_UN)` plus four descriptor closes from
`stable_remote_lock` and `remote_project_gate`.

Move or split the production lifecycle so the final publishability proof runs
only after the route and both context-manager teardowns have completed
successfully. The authority fence may remain inside its pinned authority
scope, but it must not be described as the last fallible boundary. After the
final handler-owned proof, only non-fallible owner transfer and the actual
wire publication may remain.

Extend the production ordering oracle to observe `fcntl.flock` and every
relevant `os.close`; a named-method-only trace is insufficient. A deliberate
regression that moves the proof back inside either context must fail for the
intended ordering reason.

### F2 — post-route filesystem failure is never discarded

Drive a real filesystem-origin `OSError(EIO)` from the production unlock/close
boundary. It must not be replaced by an older prepared `200`, silently
dropped, or omitted from the typed/logged outcome.

Every `do_GET` error arm reached after a read slot owns a prepared response
must dispose of that owner before selecting/publishing a different response.
Match the safe ownership semantics of `do_POST` without duplicating another
route-local protocol. Specify and test deterministic precedence when disposal
itself fails.

### F3 — publishability guard must enforce zero debt

When retained/temporary ownership or a permit survives at the proof point,
`require_publishable_state` must prevent a success response from reaching the
wire. The terminal reason must be `cleanup_incomplete`, the typed error must
remain observable, and every owner that can be released must be released
exactly once.

`state_proved` cannot be test-only evidence. Either make it part of the
production publication transition or remove it in favor of a typed state that
the production branch must consume.

### Recorded residuals

The zero-residual goal includes the review's non-blocking observations:

1. `cleanup_recoveries` is incremented but unread, and `cleanup_faults` is an
   unread property. Remove dead state or route it to a stable production
   diagnostic/decision that a non-vacuous test observes. Do not retain
   write-only evidence.
2. `_release_exact` silently returns when `amount > owned`. Replace that
   defensive silence with a typed fail-closed accounting error and prove a
   deliberate over-release cannot be masked.
3. Preserve the independently confirmed `remaining_response_bytes` fix and
   reserve-before-allocation behavior; it is not permission to restore the
   full-limit double charge.

## Invariants

- All seven prior findings and every prior residual remain closed.
- Parent authority, leaf metadata/content, project descriptor, and final
  publication fences retain their exact typed diagnostics and precedence.
- Filesystem/transport origin classification remains shared across GET/POST.
- No success byte may publish with current retained/temporary debt or an open
  permit.
- No error path may publish both an old prepared success and a replacement
  error.
- Cleanup retries once where contracted, records secondary failure
  observably, and never erases local ownership before release succeeds.
- Response allocation remains bounded and reserved before allocation.
- The exact cardinality/transport-harness flake fixes in `4930d25` remain.
- The report and canonical issue state are not modified by this worker.

## Counterexamples and reversal

Add production-route tests for:

1. unlock/close ordering relative to the final proof and first byte;
2. genuine `EIO` at the unlock boundary with exact typed/logged outcome;
3. residual retained ownership at the proof with zero success bytes;
4. prepared-owner disposal failure while replacing a response;
5. `_release_exact` over-release;
6. dead/write-only cleanup evidence removal or production consumption.

Run the new F1--F3 tests on exact rejected candidate `4930d25` and record that
they fail for the intended reasons. Also run them on predecessor `38da898`
where applicable and explain every pass/absence honestly; a legacy pass does
not erase a candidate-added false guarantee. Each deliberate regression of
the fixed successor must fail for its named owner/fence/error reason rather
than an unrelated exception.

## Validation

Run focused gates first with monotonic timing:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_mr_list_decision_work
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/check_project_docs.py
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/check_source_docs.py
git diff --check 4930d25178be11e064060487f8c254802b09868d..HEAD
```

Then run the same affected CLI scope recorded in the immutable review and the
broad Python-pattern suite on the exact final head. Cargo/Install failures
that reproduced identically at the review task base are environment evidence,
not a reason to weaken Python closure or fabricate a pass.

Use:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/run_timed_gate.py \
  --cwd "$PWD" \
  --timeout 3600 \
  --log /tmp/cfr168-mr-list-final-python.log \
  --result /tmp/cfr168-mr-list-final-python.json \
  -- python3 -m unittest discover -s tests -p 'test_python_*.py'
```

Do not use wall-clock duration from the 2026-07-31 clock-discontinuity window.

## Handoff

Commit small additive changes; do not amend or rebase. The final handoff must
provide:

- exact base, task, product head, parent, and tree;
- changed paths and write-set result;
- F1--F3 plus recorded-residual disposition;
- production owner/effect/fence timeline;
- rejected-candidate, legacy, and deliberate-regression results;
- focused, affected CLI, broad Python, project/source docs, and diff-check
  results with monotonic evidence;
- any new issue found while closing these roots;
- clean-worktree confirmation; and
- a fresh independent-review requirement. Worker success never authorizes
  integration.
