# CFR-168 Python HTTP Doctor Request-Ledger Wiring

Implement one bounded production slice from the current CFR-168 integration candidate. The actual
swarm base is `b21d5aa6aedb5f2acee178bf43e389ac81559852`; its parent product state is
`c37b3be82f31f4a54088e5cdbdd2e8d3012bda4b`.

Use Claude Opus 5. Work only in the assigned worktree and write set. Do not merge, change issue
status, weaken an existing assertion, or redesign unrelated HTTP routes.

## Defect

The independent current-state review reproduced that Python `GET /doctor` creates a second
`DecisionWorkLedger(DecisionWorkProfile.local_v1(), "remote doctor")`. The HTTP request's
`http_v1` ledger and deadline are not passed through `do_GET -> handle_http_doctor ->
remote_doctor_payload`, so the route can consume a second, much larger budget and the request
ledger remains empty.

## Required implementation

- Pass the exact request-owned `DecisionWorkLedger` and `HTTPRequestDeadline` from the live
  `do_GET` Doctor route through every production layer that performs Doctor work.
- Make `remote_doctor_payload` and `AccountedDoctorPayload` use that same ledger when supplied.
  Preserve a deliberate local/direct compatibility path only where no request ledger exists.
- Poll the supplied deadline at bounded, deterministic Doctor phase boundaries. Do not create a
  nested ledger or silently fall back after a request ledger was supplied.
- Keep quick/full result, issue ordering, JSON bytes, status, diagnostic precedence, and cleanup
  behavior unchanged when no limit/deadline is exceeded.
- On budget/deadline failure, leave no retained/temporary scalar, open permit, payload owner, or
  terminal-state debt in either a hidden or request ledger.
- Do not claim that branch-list, MR, GC, upload ancestry/signature, Rust local/file transport, or
  other CFR-168 surfaces are fixed by this slice.

## Required tests

Create `tests/test_python_http_doctor_decision_work.py` and exercise the real production route, not
only a helper:

- the returned Doctor payload is owned by the exact HTTP request ledger;
- the request ledger records Doctor work and no second `local_v1` ledger is constructed;
- quick and full modes preserve domain output and order;
- retained/work limit exact and limit-minus-one (or exact/+1 observation) give the typed resource
  outcome without partial response or debt;
- a deterministic deadline expiry is propagated through the production route and cleans up;
- direct/local compatibility still works with its intentionally local ledger;
- a regression probe equivalent to the independent review's empty-request-ledger observation now
  fails before the fix and passes after it.

Use module loading and injected clocks/filesystem fixtures already established by the CFR-168
Python tests. Do not use wall-clock sleeps.

## Validation

At minimum run:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_python_http_doctor_decision_work -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  tests.test_codefire_cli.CodeFireCliTests.test_python_doctor_accounted_payload_limit_minus_exact_plus_one \
  tests.test_codefire_cli.CodeFireCliTests.test_python_doctor_report_and_render_share_one_accounted_graph -v
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire \
  tests/test_python_http_doctor_decision_work.py
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

If the named neighboring tests differ on this base, locate and run the exact equivalent tests and
record their names. Commit the implementation and leave the worktree clean.

## Handoff

In the final message and issue evidence, state:

- exact production call chain now sharing the request ledger/deadline;
- focused test counts and commands;
- compatibility behavior retained;
- all remaining CFR-168/CFB-136 surfaces explicitly still open.
