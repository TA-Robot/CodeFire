# CFR-168 / CFB-138 Python HTTP Upload Staging Scale

Fix the newly admitted same-root blocker CFB-138 on the completed upload target
`cb13da08e82c2502920f735d27617e61f978527a`. Use Claude Opus 5
(`claude-opus-5`). Work only in the assigned worktree and write set. Do not change issue status,
management ledgers, unrelated routes, public defaults, or Rust code.

## Reproduced production defect

The live `handle_http_upload` path with an unmodified `DecisionWorkProfile.http_v1()` rejects a
root plus 1200 descendant commits before ancestry:

- phase: `semantic_validation`
- dimension: `retained_bytes`
- observed reservation: `130977816`
- default limit: `33554432`
- retained peak reached before refusal: `2274829`
- authority remains byte-identical and ledger debt returns to zero

The current recovery test raises the retained limit to 512 MiB only to reach ancestry. That is
valid characterization evidence, but it is not an acceptable product end state.

## Required behavior

- Make sealed validation account concurrent retained memory according to real owner lifetimes.
  Stream or chunk per-record validation state; do not reserve every record's heap upper as if all
  records were simultaneously live.
- Keep one exact request ledger and deadline from parse through sealed validation, signature,
  ancestry, response preflight, and publication.
- Preserve full validation, final snapshot revalidation, shared-DAG memo safety, same-inode
  mutation detection, error precedence, response bytes, replay behavior, and zero-debt cleanup.
- The 1201-commit fixture must reach the existing 409 non-fast-forward decision under the default
  profile. Do not solve this by raising defaults, lowering coverage, skipping records, weakening
  reservations, or changing the fixture to smaller work.
- All budget/deadline/subject-change failures remain pre-effect and leave branch, objects,
  generation, nonce, pending transaction, receipt, and response publication unchanged.

## Required production-path tests

- Remove the 512 MiB override from the deep fixture and prove the same exact ancestry counters and
  409 result under the default profile.
- Record 1x/3x/10x curves for graph nodes, record reads, logical depth, cumulative retained work,
  and concurrent retained peak. Assert a documented bound derived from the implementation.
- Add retained exact / minus-one / plus-one boundaries that reach the corrected validation owner.
- Retain shared-DAG determinism, iterative depth, signature policy/precedence, deadline phase,
  no-effect authority snapshots, and no-open-permit/current-zero checks.
- Prove the pre-fix target fails the new default-profile regression.

## Validation

At minimum run:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire \
  tests/test_python_http_upload_accounted_traversal.py
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests \
  -p 'test_python_http_upload_accounted_traversal.py'
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests \
  -p 'test_python_*.py'
python3 docs/development/tools/check_source_docs.py
git diff --check
```

Commit the implementation and tests, leave the worktree clean, and report the exact owner lifetime
change, before/after curves, default-profile reversal, fault/no-effect evidence, and every residual.
