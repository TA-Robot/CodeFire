# CFR-168 / CFB-138 Python HTTP Upload Zero-Residual Repair

Repair exact scale target `dece688afea815d1242e7751367052d1ceac4580` against every finding in
`docs/development/ai/swarm-results/cfr168-python-http-upload-scale-review-claude-fallback-20260730/review.md`.
Use Claude Opus 5 (`claude-opus-5`). Work only in the assigned worktree and write set. Keep the
valid 1201-commit scale correction; add new commits on top and do not amend the target or report.

The review verdict is `REQUEST_CHANGES`. Every Blocker/High/Medium finding and upload-local residual
risk is required closure work.

## Required architecture

1. One exact HTTP request ledger and `HTTPRequestDeadline` must own body parsing, authentication,
   nonce/policy capture, sealed validation, signature policy, ancestry, publication identity,
   response preparation, final fence, object staging, branch/generation publication, and cleanup.
2. Replace the two request-bypassing publication-root `validate_sealed_commit` traversals with one
   request-owned prepared sealed-validation subject/session. Reuse its validated snapshot through
   identity preflight and final revalidation; do not mint a `local_v1()` ledger inside a live HTTP
   request and do not validate the same root twice.
3. Poll the exact request deadline immediately before the first externally visible effect, after
   every identity/final-subject check. An already-expired request must never write an object, move a
   branch/generation, consume a nonce, publish a receipt/transaction, or emit a success response.
4. Give `_AccountedUploadCommitSource` and every `SealedCommitValidationSession` an explicit memory
   owner before constructing its `Path`, `documents`, `validated_commits`, `_fd_owners`,
   `_subdir_fds`, root descriptor, or any later list/dict/set/descriptor entry. Reuse or remove
   closed descriptor owners so the list cannot grow per read; reserve the final ordered-id
   projection before allocating/sorting it.
5. Re-derive retained and temporary uppers from actual simultaneously-live owners. Remove the
   `128×`/`129× maximum_record_bytes` admission cliff unless an observed owner truly needs it.
   Stream/hash/encode with bounded scratch and transfer already-accounted request-AST/record
   ownership instead of duplicating whole-record heap reservations.
6. Make the prepared-upper check a production invariant, not observation-only test state. Every
   allocation/growth on the covered route must be able to prove which outstanding reservation
   owns it.

Audit every production HTTP caller of `validate_sealed_commit`. If the same fresh-`local_v1()`
escape exists on branch, MR, escrow, or publication paths, route it through a caller-owned
ledger/deadline or expose the reusable accounted API needed by the adjacent repair packet. Do not
silently leave an equivalent HTTP escape behind.

## Required reversal and boundary evidence

- Reproduce and reverse the expired-before-second-validation case: elapsed 1.0 s against a 100 ms
  request must return the typed deadline outcome with zero effect and byte-identical authority.
- Assert that one upload creates no sealed-validation ledger other than the exact request ledger,
  performs no duplicate closure traversal, and polls throughout validation.
- Instrument every source-session list/dict/set/path/FD construction and insertion; each must see an
  outstanding matching reservation before allocation.
- Preserve the default-profile 1201-commit 409 reversal and exact 1x/3x/10x curves.
- Add default-profile record-shape curves at 64 KiB, 126,976 B, 128 KiB, 256 KiB, 1 MiB, and a
  practical near-request-body boundary. Include ASCII, CJK, emoji, escape-heavy, control-character,
  wide fan-in, signed history, diamond, and deep-chain shapes.
- Bind exact / minus-one / plus-one for request body, record/file bytes, aggregate, retained,
  temporary, graph/depth/frontier, response body/header, and publication-owner boundaries.
- Record the real maximum admissible record/count under unchanged public profiles and prove failure
  occurs only at a documented resource owner, never from formula slack.
- Snapshot objects, branches, generations, nonce, pending transaction, receipt, audit and response
  bytes before every budget/deadline/tamper/encoding/publication failure.
- Add mutation/non-vacuity evidence: restoring either nested validation, removing the final deadline
  poll, omitting the source owner, or reinstating the max-record multiplier must fail a focused test.

## Invariants

- Do not raise public limits, reduce the 1201-commit fixture, skip validation, change signature or
  fast-forward precedence, weaken same-metadata subject detection, or hide allocation sites.
- Preserve exact 200/409/error/retry response bytes and nonce/idempotency behavior.
- Success and every failure end with zero retained/temporary debt and zero permits.
- Keep parse, validate, plan, preflight, apply, render, and write boundaries explicit and iterative.

## Required validation

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire \
  tests/test_python_http_upload_accounted_traversal.py tests/test_codefire_cli.py
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_upload_accounted_traversal -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_ancestry_owner \
  tests.test_python_signature_owner \
  tests.test_python_signature_terminal_freeze \
  tests.test_python_sealed_production \
  tests.test_python_external_production -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p 'test_python_*.py'
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Run the selected legacy upload/nonce/receipt/Doctor/branch cases named by the review. Report exact
counts, hashes, before/after owner formulas and curves, prove each new regression fails on
`dece688…`, commit only the intended repair/docs, and leave the worktree clean.
