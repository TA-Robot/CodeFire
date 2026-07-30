# CFR-168 Python HTTP Doctor + Branch Independent Review

Review date: 2026-07-30
Reviewer: Fugu 1.1 Ultra (`fugu-ultra-v1.1`)
Review worktree base: `8b87d07395a5336e2d4fe74b5d76e9709bfbbfbb`

**Decision: `REQUEST_CHANGES`.**

The Doctor and Branch product slices satisfy their route-ledger, deadline, ownership, cleanup,
serialization, ordering, and non-partial-publication contracts when reviewed at their exact target
commits. Their dedicated suites pass independently, fail against the named pre-fix baselines, and
pass together in a read-only composition probe after the response-routing conflict is resolved.

One Must-class integration blocker remains: the existing canonical CLI test at
`tests/test_codefire_cli.py:6273-6280` still asserts the pre-fix Branch behavior (`200 OK`, zero
deadline polls). The completed Branch target correctly returns the typed deadline outcome. Because
that file was outside the Branch write set, a separate corrective commit is mandatory before
integration. This is a legitimate contract update, not a product regression, but the target cannot
be accepted as an integration-ready package while a required existing test is known to fail.

The expensive full Python suite was not run in this review and is not claimed to have passed.

## Revision landscape

| Subject | Baseline | Interrupted production commit | Completed target | Exact target branch |
|---|---|---|---|---|
| Doctor | `fe70dd097ebed20adcf9756a080a0688bcf22864` | `d79be3ed71c5ec210851e404fbf07f2bf46b09b7` | `01a7c862fb9825878e580229d1b265ae3a48115a` | `claude-agent/cfr168-python-http-doctor-canonical-recovery-20260729-doctor-finish-after-reboot` |
| Branch list | `775366e3c3fe89dafe307b9beacab88eb9384d93` | `8265ea1102688d9df3d3a1c3b82f2dcd4c1b64d8` | `fbef572ef008059cbe39f12b123ac28d53b04f15` | `claude-agent/cfr168-python-http-branch-recovery-20260729-branch-finish-after-reboot` |

Ancestry is linear inside each lane:

```text
775366e3c3fe89dafe307b9beacab88eb9384d93
  -> fe70dd097ebed20adcf9756a080a0688bcf22864
     -> d79be3ed71c5ec210851e404fbf07f2bf46b09b7
        -> 01a7c862fb9825878e580229d1b265ae3a48115a

775366e3c3fe89dafe307b9beacab88eb9384d93
  -> 8265ea1102688d9df3d3a1c3b82f2dcd4c1b64d8
     -> fbef572ef008059cbe39f12b123ac28d53b04f15
```

The exact merge base of the two completed targets is
`775366e3c3fe89dafe307b9beacab88eb9384d93`.

Reviewed cumulative paths:

- Doctor: `codefire`, `tests/test_python_http_doctor_decision_work.py`
- Branch list: `codefire`, `tests/test_python_http_branch_decision_work.py`
- Existing affected test: `tests/test_codefire_cli.py:6169-6280`

## Findings, severity ordered

### MUST-1 — Canonical Branch deadline assertions still encode the defect

**Evidence.** At Branch target `fbef572ef008059cbe39f12b123ac28d53b04f15`,
`handle_http_list` polls `"branch list planning"` before project lookup and
`"branch list project gate"` before entering the shared project gate
(`codefire:11673-11698`). The live `do_GET` route passes the exact request ledger and deadline
(`codefire:14607-14620`). This is the behavior required by the Branch contract.

The existing method
`CodeFireCliTests.test_python_http_bundle_deadline_boundaries_and_poll_phases` still asserts at
`tests/test_codefire_cli.py:6273-6280` that a Branch request using `request_deadline_ms=0` returns
`200 OK` and performs zero deadline polls. Running that exact method at the Branch target fails at
line 6279.

The measured correct result is exact and stable:

```text
HTTP/1.0 503 Service Unavailable
kind: request_deadline_exceeded
exit_code: 60
diagnostic phase: branch list planning
deadline poll_count: 1
deadline expired_phase: branch list planning
terminal_reason: deadline_exceeded
open_permit_count: 0
current ownership: retained_bytes = 0; all other dimensions = 0/absent
```

**Classification.** This is a legitimate contract update: the old assertion explicitly preserves
the CFB-136 defect that `/branches` ignored the request deadline. It is not evidence of a product
regression. It is, however, an incomplete implementation package because a known canonical test
fails on the completed target.

**Required correction.** Replace the two old assertions with all of the following expectations:

1. response starts with `HTTP/1.0 503 Service Unavailable`;
2. no `HTTP/1.0 200 OK` appears;
3. payload `kind == "request_deadline_exceeded"`;
4. payload `exit_code == 60`;
5. first diagnostic phase is `"branch list planning"`;
6. deadline `poll_count == 1`;
7. request `terminal_reason == "deadline_exceeded"`;
8. `open_permit_count == 0` and `assert_http_work_closed` succeeds.

The read-only composition probe commit
`701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714` contains exactly that correction and the method passes
there. The manager must land an equivalent separate corrective commit before integrating the
Branch target; the probe is evidence, not an integration instruction.

No other Must, Should, or Could finding was found in the two declared product/test slices.

## Required-review verdicts

### 1. Doctor request ledger/deadline ownership — `PASS`

- `do_GET` marks Doctor response phases active and passes `self.decision_work` plus
  `self.request_deadline` into `handle_http_doctor` (`codefire@01a7c862:14414-14447`).
- `handle_http_doctor` forwards the same objects through `remote_doctor_payload`
  (`codefire@01a7c862:11491-11512`).
- `remote_doctor_payload` uses the supplied ledger verbatim; only the no-ledger direct/file/library
  compatibility path constructs one `local_v1` ledger
  (`codefire@01a7c862:14991-15011`).
- Failure cleanup records the request ledger's pre-existing retained/temporary baselines and releases
  only Doctor's positive deltas; it terminates only a locally owned ledger
  (`codefire@01a7c862:15012-15046`).
- The payload owner terminates only locally owned compatibility ledgers; request-owned payloads leave
  request terminal state to the handler (`codefire@01a7c862:14911-14941`).
- Production-path identity, hidden-second-ledger absence, direct compatibility, supplied-ledger
  preservation, and pre-existing scalar preservation are asserted at
  `tests/test_python_http_doctor_decision_work.py:176-240` and `:480-554`.

Verdict: the exact request ledger and deadline reach route, snapshot/semantic work, diagnostics,
encoding, headers, and publication. Direct compatibility deliberately owns one local ledger, and
request cleanup neither releases pre-existing ownership nor terminates the request ledger.

### 2. Doctor corrupted-path projection and structured reports — `PASS`

- `scan_object_hash_errors()` output is converted once with
  `bad_objects = [str(path) for path in bad_paths]` before accounting
  (`codefire@01a7c862:15367-15377`).
- That same list is used to build ordered issues, retained in the payload, transferred to the payload
  owner, and released with that owner (`codefire@01a7c862:15388-15451`). No `Path` object reaches
  canonical JSON sizing or publication.
- Direct and HTTP corrupted-object tests assert string-only paths, one ordered
  `remote_object_invalid` issue before sealed-reference diagnostics, charged/published object
  identity, byte equivalence, exact/minus-one retained accounting, typed failure, and zero debt
  (`tests/test_python_http_doctor_decision_work.py:556-700`).

Verdict: projection, charging, publication, issue order, direct/HTTP structure, and cleanup are
consistent; the reproduced `PosixPath is not JSON serializable` failure is closed.

### 3. Branch one-ledger/session/final-fence traversal — `PASS`

- The live route supplies the handler's request ledger/deadline
  (`codefire@fbef572e:14607-14620`), and `handle_http_list` forwards them inside the shared project
  gate (`codefire@fbef572e:11673-11698`).
- `collect_http_branch_list_payload` captures sorted no-follow authority paths, reads and retains every
  branch record, validates every head under one `SealedCommitValidationSession`, projects the result,
  and calls `session.finalize(terminate=False)` before returning the accounted payload
  (`codefire@fbef572e:11518-11631`).
- The legacy per-head `validate_sealed_commit` path is not used. The identity test patches it to raise
  and still gets a successful response while observing exactly one request ledger, one session, and
  one projection (`tests/test_python_http_branch_decision_work.py:404-520`).
- Twenty-six heads sharing a depth-12 spine produce 37 object reads/revalidations and 31 graph nodes,
  versus the measured cold per-head costs of 336/336/192; order remains sorted and stable
  (`tests/test_python_http_branch_decision_work.py:526-592`).
- Same-inode/same-size/same-mtime content changes and extension-only changes before the final fence
  fail with the typed `digest_changed` subject-change outcome and no partial response
  (`tests/test_python_http_branch_decision_work.py:630-740`).

Verdict: one request ledger/deadline owns query/authority, shared Store validation, projection,
final revalidation, and publication; no legacy or nested traversal remains.

### 4. Production ownership, exact bounds, deadlines, and non-vacuity — `PASS`

Doctor target:

- dedicated suite: 19/19 pass;
- exact/minus-one `object_records` and retained-byte tests pass/fail at the measured owner;
- deterministic route/authority/object/payload-accounting deadline failures are typed and debt-free;
- selected target tests copied onto baseline `fe70dd097...` do not pass: request-ledger phases remain
  absent and corrupted `Path` accounting raises before any wire response.

Branch target:

- dedicated suite: 13/13 pass;
- exact/minus-one authority, object work, graph work, retained/temporary projection, revalidation,
  and response-byte cases are typed, atomic, and debt-free
  (`tests/test_python_http_branch_decision_work.py:743-801`);
- all authority/validation/fence/response deadline phase labels are reached and can be expired
  deterministically (`tests/test_python_http_branch_decision_work.py:807-867`);
- selected target tests copied onto baseline `775366e3...` cannot pass because the production
  `collect_http_branch_list_payload` owner/session entry point does not exist.

Verdict: the tests reach live production owners, prove no partial success wire/debt, and are
non-vacuous against the named pre-fix baselines.

### 5. Adjacent `do_GET` composition — `PASS`, with mandatory conflict resolution

The slices are semantically composable, but both edit the same response-phase helper and per-request
flag initialization. A raw merge of the completed tips has conflicts in `codefire`; choosing either
side wholesale would drop the other route's response deadline labels or active flag.

A read-only composition probe applied the exact Doctor delta, the adjacent upload delta, the Branch
delta with the resolution below, and the canonical test correction:

```text
5e3b0c769f84e10a57592c3c9bf229e61cd37ebe  Doctor delta
0703f71b9800ff19ce52d01bcd46c8eddb323df2  adjacent upload delta
7e87665b8158a37f1d25b449de7831cf9420f545  Branch delta + response conflict resolution
701469f91dd0c4c52c5628fb3e9d9e9d0ab0c714  canonical Branch deadline test correction
```

On that combined tree, the 32 Doctor+Branch dedicated tests pass, the corrected neighboring deadline
test passes, bundle ownership/work-limit tests pass, and an upload preflight/mutation response test
passes. Doctor quick/full equivalence remains covered by the Doctor suite. Per-request flags are
initialized for every `handle_one_request`, so no route state carries to another request.

Verdict: the product slices compose safely if and only if the exact conflict resolution below is
used. The existing bundle and mutation arms must remain unchanged.

### 6. Existing affected CLI case — `REQUEST_CHANGES`

The exact affected method is
`CodeFireCliTests.test_python_http_bundle_deadline_boundaries_and_poll_phases`, not a separately named
Branch-list test. At Branch target `fbef572e`, it fails solely at the stale `200 OK` assertion.

Correct expectation: typed `503 request_deadline_exceeded`, phase `branch list planning`, one poll,
`deadline_exceeded`, zero permits, zero ownership debt. This is a contract correction, not a product
rollback. A separate corrective commit is mandatory before integration because
`tests/test_codefire_cli.py` was outside the Branch lane write set.

## Exact integration conflict map

Command used:

```bash
git merge-tree \
  775366e3c3fe89dafe307b9beacab88eb9384d93 \
  01a7c862fb9825878e580229d1b265ae3a48115a \
  fbef572ef008059cbe39f12b123ac28d53b04f15
```

It reports `codefire` changed in both and six conflict blocks. The dedicated test files are disjoint.

| Conflict surface | Doctor side | Branch side | Required integrated resolution |
|---|---|---|---|
| Read-route phase helper | `response_deadline_route()` handles bundle + Doctor (`codefire@01a7c862:13806-13812`) | `read_route_phase(suffix)` handles bundle + Branch (`codefire@fbef572e:14002-14008`) | Keep Branch's generalized `read_route_phase(suffix)` shape and arms in this order: bundle, Doctor, Branch list. Return exact `"<route> <suffix>"` labels. |
| Response body freeze poll | Builds a Doctor/bundle route label then falls back to upload (`codefire@01a7c862:13814-13827`) | Uses `read_route_phase("response encoding")`, then upload fallback (`codefire@fbef572e:14010-14023`) | Keep the generalized phase variable; preserve upload fallback exactly. |
| Pre-encoding deadline poll | Doctor uses `response_route` (`codefire@01a7c862:14050-14052`) | Branch derives `encoding_phase` (`codefire@fbef572e:14245-14249`) | Derive encoding/header/publication phases once from the combined helper and poll each exact label. |
| Pre-header deadline poll | Doctor string-formats its route (`codefire@01a7c862:14067-14070`) | Branch uses `header_phase` | Use combined `header_phase`; do not remove Doctor or Branch response-header polling. |
| Pre-publication deadline poll | Doctor string-formats its route (`codefire@01a7c862:14080-14083`) | Branch uses `publication_phase` | Use combined `publication_phase`; do not publish after either route deadline expires. |
| Per-request flag initialization | `_doctor_deadline_active = False` (`codefire@01a7c862:14272-14274`) | `_list_deadline_active = False` (`codefire@fbef572e:14465-14467`) | Initialize bundle, Doctor, Branch-list, and mutation flags all to `False` on every request. |

Adjacent route arms do not conflict textually, but they are part of the semantic resolution:

- Branch arm must set `_list_deadline_active = True` and pass the exact request ledger/deadline.
- Doctor arm must set `_doctor_deadline_active = True` and pass the exact request ledger/deadline.
- Bundle arm and mutation response behavior must remain unchanged.
- `AccountedHTTPBranchList` must remain an accepted bounded-freezer mapping type.

The safe resolved shape is demonstrated at composition-probe
`codefire:14193-14209`, `:14424-14480`, `:14658-14661`, and `:14801-14841`.

Separately, land the mandatory `tests/test_codefire_cli.py:6273-6280` correction described in
MUST-1. Omitting it leaves the integrated suite known-red even if the product merge is otherwise
correct.

## Validation evidence

Immutable target snapshots were produced with `git archive` and validated without modifying the
review worktree.

| Command | Result |
|---|---|
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests.test_python_http_doctor_decision_work -v` at `01a7c862...` | **19 tests OK**, 21.500s |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests.test_python_http_branch_decision_work -v` at `fbef572e...` | **13 tests OK**, 16.882s |
| Same two modules on the resolved composition probe | **32 tests OK**, 36.073s |
| Seven neighboring Doctor CLI methods: request-ledger/direct cleanup, failure cleanup, missing project, object corruption, invalid sealed refs | **7 tests OK**, 118.507s |
| Five neighboring Branch CLI methods: HTTP upload/list/clone, invalid remote head, list/clone/MR, remote list+Doctor ledger, bundle work limits | **5 tests OK**, 274.092s |
| Existing affected method at exact Branch target | **1 failure**, line 6279 stale `200 OK` assertion, 1.239s |
| Corrected affected method on composition probe | **1 test OK**, 2.744s |
| Composition neighbors: bundle ledger/wire, bundle work limits, upload response preflight | **3 tests OK**, 10.835s |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire tests/test_python_http_doctor_decision_work.py tests/test_codefire_cli.py` at Doctor target | pass |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire tests/test_python_http_branch_decision_work.py tests/test_codefire_cli.py` at Branch target | pass |
| Combined `py_compile` for `codefire`, both dedicated files, and `tests/test_codefire_cli.py` | pass |
| `python3 docs/development/tools/check_source_docs.py` on composition probe | pass: 44 modules, 34 commands, 12 object kinds |
| `python3 docs/development/tools/check_project_docs.py` on composition probe | pass: 539 Markdown files inventoried |
| `git diff --check fe70dd097...01a7c862 -- codefire tests/test_python_http_doctor_decision_work.py` | pass |
| `git diff --check 775366e3...fbef572e -- codefire tests/test_python_http_branch_decision_work.py` | pass |
| `git diff --check 16d0129...701469f` on composition probe | pass |
| Three representative Doctor target tests copied to `fe70dd097...` | **3/3 do not pass**: hidden ledger phases absent; corrupted `Path` charge aborts wire |
| Three representative Branch target tests copied to `775366e3...` | **3/3 do not pass**: production request-session collector absent |

The contract's example neighboring names
`test_remote_doctor_reports_corruption`, `test_http_remote_doctor`,
`test_http_remote_branch_list`, and `test_remote_branch_list` do not exist at these revisions. The
actual neighboring methods above were located and run. The affected deadline method was inspected
and run explicitly.

No full-suite result exists for the Doctor target: its background full-suite task ended with the
Claude session. This review also defers the expensive full Python suite to the exact integrated
revision and makes no full-suite pass claim.

## Residual risk

1. The exact integrated revision does not yet exist. The composition probe proves the resolution is
   viable, but the manager must reproduce it without dropping Doctor, Branch, bundle, or mutation
   state.
2. The canonical stale deadline assertions remain uncorrected on the completed Branch target; this
   is the current integration blocker.
3. The full Python suite remains unexecuted on the exact integrated revision. It must run after the
   conflict resolution and test correction, because the Doctor background run has no result and the
   Branch target knowingly fails one existing case before correction.
4. This review does not accept or make claims about Rust HTTP, MR/GC, upload signature/ancestry beyond
   the adjacent composition checks, metrics, or any other CFR-168 surface outside the declared paths.

## Required next action

1. Land a separate corrective commit for `tests/test_codefire_cli.py:6273-6280` with the exact typed
   Branch deadline expectations above.
2. Integrate the two product deltas using the conflict map, preserving all four per-request flags and
   the generalized response-phase helper.
3. Re-run both dedicated files, the corrected neighboring method, relevant bundle/mutation neighbors,
   Python compilation, doc checks, and `git diff --check` on the exact integrated revision.
4. Run the full Python suite on that exact revision before declaring the two slices accepted.
