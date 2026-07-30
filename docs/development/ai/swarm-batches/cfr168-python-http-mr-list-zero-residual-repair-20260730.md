# CFR-168 Python HTTP MR List Zero-Residual Repair

Repair exact composition target `37f7427d46d2cf3ad66de3c1885ef4c0527cae70` against every finding and
MR-list/shared-read-route residual in
`docs/development/ai/swarm-results/cfr168-python-http-mr-list-review-claude-20260730/review.md`.
Use Claude Opus 5 (`claude-opus-5`). Work only in the assigned worktree and write set. Add new
commits on top; do not amend the reviewed target or report.

The verdict is `REQUEST_CHANGES`. Preserve the target's single request ledger, deadline, one-session
per distinct object root, exact pre-allocation ownership, final fence, zero-debt cleanup, and
request isolation. Close both Must findings and every Should/Could/residual; none are optional under
the CFR-168 zero-residual rule.

## Required behavior and architecture

1. Preserve exact legacy first-error precedence for multi-fault repositories without weakening the
   pinned authority snapshot:
   - capture raw bytes/metadata/descriptor ownership without eagerly raising later-record parse or
     version faults;
   - retain deferred per-record capture/read/parse/version outcomes;
   - process and raise them in deterministic filename order at the same point the pre-fix route
     would have observed each fault;
   - keep final byte/metadata revalidation over every consumed authority.
   Do not merely document the current precedence regression when it can be removed with deferred
   capture.
2. Fail closed with a complete typed HTTP response for every filesystem failure:
   - map `O_NOFOLLOW`, symlink, directory, unavailable, permission, metadata, and read failures into
     one stable diagnostic/status family;
   - add shared `do_GET` and `do_POST` handling so no Doctor, branch, MR-list, bundle, upload, MR,
     or GC route can close the connection with a zero-byte reply;
   - preserve primary errors and close the exact request ledger once.
   Following authority symlinks is not required compatibility; explicit typed rejection is.
3. Memoize escrow traversal and digest by the exact pinned `(session/root, source_head)` identity.
   Many merge requests sharing one head must reuse ordered IDs and digest ownership without
   duplicate physical reads, traversal, hashing, or uncharged retained copies. Final session
   revalidation still fences every reused result.
4. Replace `stable_project_id`'s pre-read `stat` plus reopen with one accounted pinned descriptor/raw
   read. Charge actual bytes consumed, parse from the owned bytes, memoize once, and detect
   same-metadata replacement at the final fence.
5. Reduce duplicate deadline polling without reducing cancellation coverage. Use one stable polling
   policy per file/record/frontier or bounded work quantum; prove a long single traversal remains
   interruptible and phase attribution remains exact.
6. Keep the shared dispatcher/reset/freezer composition deterministic. Exactly one read-route phase
   flag may be active, every flag resets before the next request, and an adjacent route's
   filesystem failure must not leak MR-list phase, debt, or response state.

## Required evidence

- Add multi-fault fixtures spanning read/unavailable/symlink/invalid JSON/version, URL ownership,
  cross-project, sealed reference, subject digest, escrow digest, and branch faults. Compare exact
  target output against the real baseline implementation for both fault orders.
- Add symlinked MR-record and branch-record tests plus shared Doctor/branch/bundle/POST cases. Every
  one must receive a typed status/body, zero partial payload, zero debt, and zero permits.
- Pin one-session/no-duplicate-read behavior and memoized escrow work at 1/8/24/80/200 merge
  requests. For one shared head, object traversal/hash counters must become constant rather than
  linear in MR count; only per-MR record/projection work may scale linearly.
- Cover exact / minus-one / plus-one for every charged dimension after memoization and deferred
  fault ownership. Counters and response bytes remain deterministic.
- Add mutation/non-vacuity checks that fail if eager whole-directory version validation returns,
  an `OSError` escapes the dispatcher, memoization is removed, descriptor bytes are reopened, or a
  required production poll is removed.
- Re-run the eight-request isolation sequence, HTTP/1.0 pipelining proof, the local `cf://`
  `cmd_request_list` parity cases, and all affected legacy request-list/MR cases.

## Invariants

- No public budget/deadline is raised, no validation or nofollow check is weakened, no partial list
  is published, and no later error replaces the deterministic first error.
- Deterministic filename order, status calculation, open/stale behavior, legacy/cross-project
  rejection, URL checks, JSON fields, and 404/400/503 semantics remain exact except for the required
  typed rejection of unsafe filesystem authority.
- Every owner is reserved before read/allocation/insertion and every path ends at zero
  retained/temporary debt and zero permits.
- The MR-mutation and GC routes remain separately reviewed successors; shared dispatcher corrections
  must be reusable by them and must not claim those product packets complete.

## Required validation

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile \
  codefire tests/test_python_http_mr_list_decision_work.py tests/test_codefire_cli.py
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_mr_list_decision_work -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p 'test_python_*.py'
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Run broad discovery only when the manager grants the single heavy-test slot. Prove every new
regression fails against `37f7427…`, and every legacy-precedence case matches `7327e2a…`. Report
exact hashes, counts, counter curves, poll counts, before/after authority hashes, and integration
conflicts. Commit only intended product/tests/source docs and leave the worktree clean.
