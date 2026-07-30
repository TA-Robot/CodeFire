# CFR-168 Python HTTP Doctor + Branch Independent Review

Independently review the completed adjacent Python HTTP Doctor and branch-list request-ledger
lanes. Use Fugu 1.1 Ultra (`fugu-ultra-v1.1`). This is review-only: do not modify product code,
tests, issue state, or management ledgers.

## Exact review subjects

Doctor:

- baseline: `fe70dd097ebed20adcf9756a080a0688bcf22864`
- interrupted production commit: `d79be3ed71c5ec210851e404fbf07f2bf46b09b7`
- completed target: `01a7c862fb9825878e580229d1b265ae3a48115a`
- cumulative paths: `codefire`, `tests/test_python_http_doctor_decision_work.py`
- contract:
  `docs/development/ai/swarm-batches/cfr168-python-http-doctor-canonical-port-20260729.md`

Branch list:

- baseline: `775366e3c3fe89dafe307b9beacab88eb9384d93`
- interrupted production commit: `8265ea1102688d9df3d3a1c3b82f2dcd4c1b64d8`
- completed target: `fbef572ef008059cbe39f12b123ac28d53b04f15`
- cumulative paths: `codefire`, `tests/test_python_http_branch_decision_work.py`
- contract:
  `docs/development/ai/swarm-batches/cfr168-python-http-branch-session-20260729.md`

The exact target branches are:

- `claude-agent/cfr168-python-http-doctor-canonical-recovery-20260729-doctor-finish-after-reboot`
- `claude-agent/cfr168-python-http-branch-recovery-20260729-branch-finish-after-reboot`

Do not infer acceptance from either implementation manager's `passed` state. In particular:

- the Doctor agent's full-suite background task was stopped when its Claude session exited, so no
  full-suite result exists;
- the Branch agent reports that existing assertions at
  `tests/test_codefire_cli.py:6279-6280` now expect the wrong deadline phase/status, but that file
  was outside its write set. Determine whether this is a legitimate contract update, a product
  regression, or an incomplete implementation.

## Required review

1. Doctor uses exactly the request `DecisionWorkLedger` and deadline through route, snapshot,
   diagnostics, encoding, headers, and publication; direct/file compatibility deliberately owns
   one local ledger and request cleanup does not terminate or release unrelated ownership.
2. Corrupted object paths are projected once to stable strings before sizing and the exact retained
   projection is published/released. Direct and HTTP paths produce structured unhealthy reports,
   preserve diagnostic order, and cannot leak untyped `Path` serialization errors.
3. Branch list uses one request ledger/deadline across query parsing, authority capture, Store
   session/revalidation, projection, and response publication, with no legacy/nested traversal.
4. Both test suites reach the intended production owner for exact/minus-one and deadline failures,
   prove no partial wire/debt, and are non-vacuous against the respective pre-fix baselines.
5. The two slices compose safely in adjacent `do_GET` dispatch: per-request flags reset, response
   phase labels are exact, quick/bundle/mutation behavior is not changed, and integration order
   will not silently drop either route's ledger/deadline state.
6. Inspect and run the two affected `tests/test_codefire_cli.py` cases. State the exact correct
   expectations and whether a separate corrective commit is mandatory before integration.

Run both dedicated focused files, relevant neighboring CLI cases, Python compilation, and
`git diff --check`. It is acceptable to defer the expensive full suite to the exact integrated
revision, but the report must not claim it passed here.

## Output

Write exactly:

`docs/development/ai/swarm-results/cfr168-python-http-doctor-branch-independent-review-20260730/review.md`

Include exact hashes, `ACCEPT` or `REQUEST_CHANGES`, severity-ordered findings with concrete
file/line evidence, verdicts for all six items, commands/results, the exact integration conflict
map, and residual risk. Missing evidence or a mandatory out-of-write-set test correction requires
`REQUEST_CHANGES`. Commit only the report and leave the reviewer worktree clean.
