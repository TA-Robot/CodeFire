# CFR-168 Python Upload Live-Route Cleanup Repair

Status: authorized for one bounded implementation candidate

Provider/model: Claude Opus 5 (`claude-opus-5`)

Role: implementation repair; this lane cannot approve or integrate its work

## Exact inputs

- rejected candidate: `446cf4a97b63e46d011a03083ef59354b9d68178`
- immutable independent report: `3475a8a19a244c266e0a5e158b712eff7cab5d37`
- report path:
  `docs/development/ai/swarm-results/cfr168-python-http-upload-third-repair-rereview-claude-20260731/review.md`
- implementation base: the clean commit containing this packet
- root issues: CFR-168 and CFB-138; both remain nonterminal

The report and prior candidate are immutable. Add commits above this packet.
Do not amend, rebase, squash, reset, merge, or rewrite their bytes.

## Objective

Make both production HTTP Upload commit-source routes drain a one-shot cleanup
failure before the request ledger is observed, while preserving the first
cleanup error, zero durable mutation and the already-proven default-profile
1201-commit CFB-138 reversal.

## Manager actionability adjudication

| Finding | Disposition | Repair authority |
|---|---|---|
| F1 live-route debt at the production checkpoint | `required_correction`; MF-1/MF-4 | repair both owned call sites |
| F2 traceback-stripping false oracle | `required_correction`; MF-5 | replace with production-state oracle |
| F3 load-bearing finalizer has no reversal | `required_correction`; MF-5 | add one behavioral deletion reversal |
| F4 unconditional finalizer guarantee in canonical docs/docstring | `required_correction`; MF-1 | narrow to the actual guarantee |
| F5 refcount wording | duplicate evidence for F4 | correct only in successor evidence; do not rewrite history |
| F6 persistent cleanup failure | `nonblocking_improvement` | document as not guaranteed; no infinite retry/framework |
| F7 retry may replace an older failed owner | `nonblocking_improvement` | retain as routed owner-hardening; do not expand this repair |
| F8 non-exact docs gate | required evidence gap, no product defect | rerun on the exact frozen candidate |
| F9 aggregate witness | `out_of_scope`; production reachability unproved | no repair |

No other report observation authorizes mutation.

## Closed write set

1. `codefire`
2. `tests/test_python_http_upload_accounted_traversal.py`
3. `docs/development/source-architecture.md`
4. `docs/development/command-source-trace.md`

Do not edit issue status, backlog, history, current-cycle state, prior packets or
reports, Doctor/Branch/MR-list product/tests, Rust, profiles, packaging,
Install, Dogfood or subprojects. Return an exact dependency instead of widening
the set.

## Required implementation

1. In `is_ancestor_from_objects_accounted`, when the first `source.close()`
   raises after clearing the control debt, retry the same source exactly once
   before propagating the original error.
2. Apply the same bounded rule in
   `enforce_remote_commit_signature_policy_accounted`.
3. Preserve the primary operation/cleanup precedence and propagate the first
   cleanup error even when the retry drains all ownership.
4. Do not add a public API, persisted state, new ledger, new HTTP status or a
   generic cleanup framework.
5. Replace the `assertRaises`-after-the-fact live-route oracle with one that
   retains the raised exception traceback at the production observation point.
6. Cover ancestry and signature-history routes, exactly one injected
   `close(2)` fault, zero retained/temporary debt, zero permits, unchanged
   authority/branch/object/nonce/transaction/receipt state and unchanged
   terminal precedence.
7. Add a behavioral reversal that fails if the synchronous retry is removed,
   and a separate fallback reversal that fails if
   `_AccountedUploadCommitSource.__del__` is removed.
8. Narrow source docs/docstrings: synchronous caller retry owns production
   correctness; the finalizer is best effort only after the object becomes
   collectible and does not guarantee in-flight or persistent-failure cleanup.

## Protected invariants and non-goals

- keep one request ledger and the existing request deadline;
- never erase a cleanup handle before its own release succeeds;
- never convert cleanup failure into success;
- publish no branch, object, nonce, transaction, receipt or response on the
  failing paths;
- keep default `http_v1` limits unchanged;
- preserve the 1201-commit default-profile 409 result and 1x/3x/10x/exact
  accounting evidence;
- preserve accepted Doctor/Branch and MR-list behavior; and
- do not claim recovery from permanently failing teardown.

## Validation

Run focused checks before broad checks:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  codefire \
  tests/test_python_http_upload_accounted_traversal.py \
  tests/test_python_sealed_production.py \
  tests/test_codefire_cli.py

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_upload_accounted_traversal -v

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_ancestry_owner \
  tests.test_python_signature_owner \
  tests.test_python_signature_terminal_freeze \
  tests.test_python_sealed_production \
  tests.test_python_external_production -v

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_codefire_cli -k upload

PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  discover -s tests -p 'test_python_*.py'

python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

If a broad test is too long, checkpoint the clean implementation/focused-test
candidate first, then continue in the same session. A skipped or unavailable
gate is explicit risk, never PASS.

## Candidate Freeze and handoff

Commit only the four allowed paths and leave the worktree clean. Report exact
base/head/tree, changed paths, patch digest, focused/broad commands and results,
the two deliberate reversals, authority hashes and remaining routed risks.
Request one fresh report-only exact repair review. Do not self-approve, port,
integrate, Install, Dogfood, change issue status or claim CFR-168/CFB-138 Close.

The experimental recurrence live arms do not include Python product repair, so
they are not an integration prerequisite for this candidate. Standard exact
repair review and manager adjudication remain mandatory; no finding-count
target applies.
