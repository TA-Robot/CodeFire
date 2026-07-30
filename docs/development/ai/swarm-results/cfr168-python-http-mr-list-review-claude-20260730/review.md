# CFR-168 Python HTTP Merge-Request List Independent Claude Review

Review date: 2026-07-30
Reviewer: Claude Opus 5 (`claude-opus-5`)
Review worktree base: `f13ddabfc521336c0056d7cb384d0269d39bf3b7`

**Decision: `REQUEST_CHANGES`.**

The recovered slice is a genuine, well-built fix for the named production defect. `do_GET` now
threads the handler's own `DecisionWorkLedger` and `HTTPRequestDeadline` through a single
merge-request-list operation owner; one `SealedCommitValidationSession` per distinct pinned object
root replaces the legacy per-reference `validate_sealed_commit` path; every retained path, parsed
record, frontier frame, escrow id, projected field, and response byte is reserved before it is
allocated; the projection is prepared in full before the final fence and before publication; and
every measured request closes with zero current debt and zero open permits — including on the
budget, deadline, subject-change, and uncaught-`OSError` paths. Request isolation, phase-flag reset,
and neighbouring-route non-interference all hold on the real socket protocol.

Two behaviour changes relative to the exact pre-fix route block acceptance as an integration-ready
package. Both were verified against the **actual** pre-fix `handle_http_request_list` loaded from
baseline `7327e2a8e72f6af84b0eadd8bb6b16233f86882a`, not against the test file's reimplementation:

1. Whole-snapshot authority capture reorders observable first-error precedence whenever more than
   one merge request is faulty. The contract requires that precedence be preserved, and the focused
   suite certifies "error precedence parity" using only single-fault fixtures.
2. Because the pinned reads are `O_NOFOLLOW` and `do_GET` catches only `ResourceBudgetExceeded`,
   `RequestDeadlineExceeded`, and `CFError`, a symlinked merge-request or branch authority record
   now produces a **zero-byte reply** where the pre-fix route returned `200 OK`.

Neither is a partial publication and neither leaks ledger state; both are diagnostic/compatibility
regressions that need an explicit decision plus test coverage before integration.

Per the memory-constrained wave instruction, the broad `test_python_*` discovery run and the Cargo
toolchain commands were **not** run in this review and are not claimed to have passed. The 18
focused tests and the targeted `test_codefire_cli` merge-request/request-list cases **were** run
independently here and passed.

## Revision landscape

| Subject | Value |
|---|---|
| Baseline / task revision | `7327e2a8e72f6af84b0eadd8bb6b16233f86882a` (`chore: add Python MR list session task`) |
| Completed target | `37f7427d46d2cf3ad66de3c1885ef4c0527cae70` (`probe: compose Python HTTP MR list session`) |
| Target branch | `claude-agent/cfr168-python-http-mr-list-session-20260730-mr-list-session` |
| Review worktree HEAD | `f13ddabfc521336c0056d7cb384d0269d39bf3b7` |
| Merge base (target ↔ review HEAD) | `16d012938730fdb2b092b7f8c5cd601b53b81a1f` |

The target is the tip of a linear **composition probe** lane, not an isolated slice:

```text
16d012938730fdb2b092b7f8c5cd601b53b81a1f   (chore: register Doctor and branch independent review)
  -> 5e3b0c7  probe: compose Python HTTP Doctor ledger
     -> 0703f71  probe: compose Python HTTP upload traversal
        -> 7e87665  probe: compose Python HTTP branch session
           -> 701469f  test: update branch-list deadline expectation
              -> 7327e2a8e72f6af84b0eadd8bb6b16233f86882a   (baseline / task)
                 -> 37f7427d46d2cf3ad66de3c1885ef4c0527cae70 (completed target)
```

Everything attributed below to the MR-list slice is taken from
`git diff 7327e2a8e72f6af84b0eadd8bb6b16233f86882a 37f7427d46d2cf3ad66de3c1885ef4c0527cae70`,
which is exactly:

```text
 codefire                                        |  742 ++++++++-
 tests/test_python_http_mr_list_decision_work.py | 1824 +++++++++++++++++++++++
 2 files changed, 2548 insertions(+), 18 deletions(-)
```

Cumulative reviewed paths: `codefire`, `tests/test_python_http_mr_list_decision_work.py`.
Existing affected test read but not modified: `tests/test_codefire_cli.py`.
Contract: `docs/development/ai/swarm-batches/cfr168-python-http-mr-list-session-20260730.md`.

Unqualified `codefire:N` line references below are at target `37f7427`; baseline references are
written `codefire@7327e2a:N`.

## Findings, severity ordered

### MUST-1 — Whole-snapshot authority capture changes observable first-error precedence

**Evidence.** The pre-fix route interleaved read and validation per record in filename order
(`codefire@7327e2a:12090-12105`): read record *i* (which version-validates it through
`read_remote_merge_request_path`, `codefire@7327e2a:2367-2372`), then parse its URLs, then the
project-ownership check, then `reject_legacy_cross_project_merge_request_read`, then
`validate_http_merge_request_record`, then the branch read — and only then move to record *i+1*.

The target splits that into two whole-authority phases. `capture_authority` (`codefire:12215-12243`)
reads **and version-validates every** merge-request record before any URL, sealed-reference, or
subject validation runs:

- `codefire:12229-12234` — `read_json_accounted` per path, raising
  `merge request record is unavailable` / `invalid JSON: <path>` during capture;
- `codefire:12239` — `validate_persisted_record_version(mr, "remote_merge_request")` per record,
  still inside the capture loop;
- `validate_records` (`codefire:12309-12343`) only then walks the pinned list to do URL parsing,
  the project-ownership check, legacy/cross-project rejection, `validate_record`, and the branch
  read.

Consequently any read- or version-class fault anywhere in the directory now pre-empts a
validation-class fault in an **earlier** filename.

**Failure scenario (measured).** Ten multi-fault fixtures were driven through the live socket route
at the target and, for the same storage tree, through the pre-fix `handle_http_request_list`
imported from the baseline commit's `codefire`. Eight of ten diverge. Representative rows:

```text
first(sorted)=source_head:7        last(sorted)=unparseable record
  pre-fix: CFError: merge request validation failed: source_head must be a commit object id
  target : 400 Bad Request  {"error": "invalid JSON: .../merge_requests/MR-d66faf8bda25.json: ..."}

first(sorted)=source_head:7        last(sorted)=version 2
  pre-fix: CFError: merge request validation failed: source_head must be a commit object id
  target : 400 Bad Request  {"error": "unsupported remote_merge_request.version 2; readable version is 1"}

first(sorted)=source_head:7        mid(sorted)=version 2
  pre-fix: CFError: merge request validation failed: source_head must be a commit object id
  target : 400 Bad Request  {"error": "unsupported remote_merge_request.version 2; readable version is 1"}

first(sorted)=foreign target_url   last(sorted)=version 2
  pre-fix: CFError: merge request target does not belong to the listed project
  target : 400 Bad Request  {"error": "unsupported remote_merge_request.version 2; readable version is 1"}

first(sorted)=diverged subject_digest  last(sorted)=version 2
  pre-fix: CFError: merge request subject digest diverged
  target : 400 Bad Request  {"error": "unsupported remote_merge_request.version 2; readable version is 1"}
```

Controls behave correctly and confirm the mechanism rather than a fixture artefact: when the
**earliest** filename carries the read/version fault, or when both faults are the same class, the
two routes agree exactly (`first=version 2, last=source_head:7` → both report the version error;
`first=source_head:7, last=source_head:7` → both report the sealed-reference error).

Every divergent case still fails closed: HTTP 400, no `"merge_requests"` on the wire, all current
ownership dimensions zero, `open_permit_count == 0`.

**Why it blocks.** The contract requires the target to "Preserve the existing deterministic MR
filename order, status calculation, **error precedence**, legacy/cross-project rejection,
source/target URL checks, JSON bytes, and 404/400 behavior." Multi-fault first-error precedence is
not preserved. The contract *also* mandates a pinned authority snapshot and a final snapshot fence,
so the two requirements are in genuine tension: restoring interleaved reads would defeat the pinned
snapshot the rest of the design depends on.

**Required correction (owner's choice, then coverage either way).** Either

1. accept the new precedence as an intentional, documented compatibility change — record it in the
   contract/batch note, and replace the single-fault-only parity assertion with explicit multi-fault
   fixtures that assert the *new* rule ("any authority read/version fault pre-empts any validation
   fault, regardless of filename order"); or
2. preserve pre-fix precedence by deferring per-record `validate_persisted_record_version` out of
   `capture_authority` into `validate_records`, and by hoisting nothing else — this recovers parity
   for the version-fault cases but **not** for the unparseable/unavailable cases, which are
   unavoidable under a pinned pre-read; those must then be documented as (1).

Option 1 is the recommendation. What cannot stand is the current state: a contract-listed property
is violated while the suite asserts it holds.

### MUST-2 — A symlinked authority record now yields a zero-byte reply where the pre-fix route returned `200 OK`

**Evidence.** The pinned reads use `read_json_accounted`, which opens with `O_NOFOLLOW` and only
maps `FileNotFoundError` to the default (`codefire@7327e2a:1454-1460`; unchanged at target). A
symlinked regular file therefore raises `OSError: [Errno 40] Too many levels of symbolic links`.
`do_GET` catches only `ResourceBudgetExceeded` (`codefire:15554`), `RequestDeadlineExceeded`
(`codefire:15557`), and `CFError` (`codefire:15559`) — so that `OSError` escapes the route and the
connection closes having published nothing.

The pre-fix route reached these files through `read_json` / `read_remote_branch_record`
(`codefire@7327e2a:12097`, `codefire@7327e2a:21108-21112`), which follow symlinks.

**Failure scenario (measured).** Replace one merge-request record, or the consulted `main` branch
record, with a symlink to a sibling file holding the identical bytes:

```text
MR record is a symlink       pre-fix: 200 OK list[8]   target: <EMPTY WIRE> (0 bytes)
branch record is a symlink   pre-fix: 200 OK list[8]   target: <EMPTY WIRE> (0 bytes)
control, clean authority     pre-fix: 200 OK list[8]   target: 200 OK, 1296 bytes
```

The client receives no status line, no diagnostic, and cannot distinguish this from a crash. Ledger
hygiene is nonetheless intact on this path: `open_permit_count == 0` and every current dimension is
zero.

**Scope.** This is a shared-dispatcher gap, not an MR-list-only coding error. The same symlinked
branch record produces the identical zero-byte reply on the neighbouring routes at this target:

```text
GET /v1/projects/org/app/merge-requests  -> <EMPTY WIRE>
GET /v1/projects/org/app/branches        -> <EMPTY WIRE>
GET /v1/projects/org/app/doctor          -> <EMPTY WIRE>
```

It is therefore pre-existing on Doctor and branch-list and **newly reachable** on
`/merge-requests`, where it is a `200 OK` → empty-wire regression.

**Required correction.** Add an `except OSError` arm to `do_GET` (and the `do_POST` equivalent) that
publishes a typed terminal diagnostic — the natural mapping is a `CFError`-class 400 carrying the
offending path, matching `read_json_accounted`'s own
`bounded JSON file is invalid: <path>` wording, which is what a directory-in-place-of-a-record
already produces correctly. Because `do_GET` is shared with Doctor, branch-list, bundle, and upload,
this belongs in a separate corrective commit owned by the manager, outside the MR-list write set —
the same shape as MUST-1 in the Doctor/branch independent review. Add one MR-list test asserting a
symlinked MR record and a symlinked branch record both publish a typed diagnostic with no partial
list.

### SHOULD-1 — The escrow object-set digest is recomputed per merge request, not per pinned source head

**Evidence.** `validate_subject` calls `_collect_escrow_object_ids` (`codefire:12486-12620`) and
`_digest_escrow_records` (`codefire:12621-12655`) once per merge request. Both are keyed only by
the session and `source_head`; neither result is memoized. When *N* merge requests share one
`source_head` — the common "many MRs against one target from one branch tip" shape — the route
re-traverses and re-digests the identical escrow set *N* times.

**Measured slope** (one shared escrow root, one shared `source_head`, 29-object fixture, budgets
raised so only the slope is observed):

| MRs | sessions | files_read | object_records | memo_misses | graph_nodes | revalidation | graph_edges | bytes_hashed | aggregate_bytes | peak_retained | elapsed |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 1 | 47 | 22 | 22 | 9 | 22 | 142 | 42,558 | 29,006 | 17,707 | 0.25 s |
| 8 | 1 | 54 | 22 | 22 | 9 | 22 | 646 | 130,170 | 123,737 | 25,655 | 0.99 s |
| 24 | 1 | 70 | 22 | 22 | 9 | 22 | 1,798 | 330,426 | 340,265 | 42,359 | 2.68 s |
| 80 | 1 | 126 | 22 | 22 | 9 | 22 | 5,830 | 1,031,322 | 1,098,113 | 102,504 | 9.96 s |

The physical-read side is exactly right: `files_read` rises by exactly one per added merge request
(its own record), and `object_records`, `memo_misses`, `graph_nodes`, `semantic_validations` (31),
and `snapshot_revalidation_records` are all **constant**. An instrumented `read_json_accounted`
recorded **zero** paths read more than once at every cardinality. Session count stays at 1.

What scales linearly is CPU and hash/aggregate accounting: `bytes_hashed` ≈ 12.5 KB per added MR,
`graph_edges` ≈ 71 per added MR, `revisits` 52 → 4,239.

**Not a regression.** At default `http_v1` budgets the target is materially *faster* than the
pre-fix route, because session reuse removes the per-reference re-reads:

```text
N=80   pre-fix 200 OK in 11.36 s   target 200 OK in  9.63 s
N=200  pre-fix 200 OK in 56.10 s   target 200 OK in 24.60 s
```

**Why it still matters.** `request_deadline_ms` defaults to 30,000 (`codefire:250`) and the target
now enforces it, so the per-MR constant sets the cardinality at which a previously-slow-but-200
listing becomes a typed 503. At this fixture's escrow size that boundary is a few hundred merge
requests. Memoizing `(session, source_head) -> (ordered, digest)` collapses the dominant term
without weakening any validation, since the escrow set is already fenced once per session by
`finalize_sessions` (`codefire:12711-12717`). Recommended as a follow-up in the product lane, not a
blocker.

### SHOULD-2 — The suite certifies error-precedence parity it does not test, and never tests symlinked authority

**Evidence.** `tests/test_python_http_mr_list_decision_work.py:1025-1123`
(`test_legacy_cross_project_and_error_precedence_parity`) injects exactly **one** fault per run —
each case mutates a single merge request (indices 0, 1, 2, 4, 6, 7) and compares against
`legacy_request_list`. Under that fixture family the target and the pre-fix route always agree, so
the test passes while the general property asserted by its name and docstring is false (MUST-1).

`test_malformed_and_invalid_sealed_references_fail_closed`
(`tests/test_python_http_mr_list_decision_work.py:1125-1201`) asserts target-side messages only; it
performs no legacy comparison, so it cannot detect the `500`/uncaught → `400` and `200` → empty-wire
shifts. No test in the file exercises a symlinked merge-request, branch, or descriptor record, which
is why MUST-2 survived an otherwise thorough suite.

Also worth noting for the record: the file's `legacy_request_list`
(`tests/test_python_http_mr_list_decision_work.py:530-583`) is a faithful, line-for-line
reproduction of `codefire@7327e2a:12090-12105` that resolves every helper through the production
module globals. It is sound evidence. This review nevertheless re-derived both MUST findings against
the real baseline function, and the two agree.

### COULD-1 — `stable_project_id` charges from a pre-read `stat`, then re-opens the descriptor

`stable_project_id` (`codefire:12266-12308`) stats `remote_project.json` with
`follow_symlinks=False`, charges `aggregate_bytes`/`temporary_bytes`/`files_read`/`file_bytes_read`/
`record_reads` from that observed size, and then calls `remote_project_stable_id`
(`codefire@7327e2a:2199-2210`), which re-opens and re-reads the file. A size change between the stat
and the read makes the charge diverge from the bytes actually consumed. The window is inside the
shared project gate and the result is memoized once per request, so the exposure is small. Related:
the descriptor is physically read twice per request — once by `remote_project_gate`, once here —
which is pre-existing behaviour also present in the pre-fix route through
`validate_merge_request_subject`.

### COULD-2 — Deadline-poll density

A 200-merge-request listing performed **1,496,274** deadline polls (~7,481 per merge request);
`_collect_escrow_object_ids` polls per frontier pop and per visited reference, and
`_digest_escrow_records` polls twice per escrow record per merge request. Correctness is unaffected
and the clock reads are cheap individually, but at scale the polling itself is a visible share of
the route's wall clock. Worth revisiting alongside SHOULD-1.

No other Must, Should, or Could finding was found in the two declared paths.

## Required-review verdicts

### 1. Exact handler ledger/deadline owns every stage, no legacy or locally-created traversal — `PASS`

- `do_GET` marks the MR-list response phase active and passes `self.decision_work` and
  `self.request_deadline` positionally into `handle_http_request_list` (`codefire:15544-15550`).
- `handle_http_request_list` (`codefire:12800-12822`) polls `merge request list planning` before
  project lookup and `merge request list project gate` before entering the shared gate, then hands
  the same two objects to `collect_http_merge_request_list_payload` (`codefire:12763-12797`), which
  constructs exactly one `_HTTPMergeRequestListOperation` and no ledger of its own.
- Authority capture (`codefire:12215-12243`), consulted-branch reads (`codefire:12245-12264`),
  descriptor identity (`codefire:12266-12308`), URL/subject validation (`codefire:12309-12484`),
  escrow traversal and both digests (`codefire:12486-12667`), projection (`codefire:12669-12709`),
  the final fence (`codefire:12711-12717`), and cleanup (`codefire:12733-12760`) all take
  `self.ledger` / `self.deadline` — there is no second `DecisionWorkLedger` anywhere in the slice.
- Response bytes, headers, and publication are named through the shared read-route dispatcher
  (`codefire:14899`), and `AccountedHTTPMergeRequestList` (`codefire:11441-11454`) is registered in
  the bounded freezer's owner set (`codefire:14153`).
- Independently measured: a successive-request sequence produced **8 distinct ledger objects for 8
  requests** and, for every MR-list request, 9 `merge request list *` phases and **zero** foreign
  phases.
- The suite's `forbidden_legacy_helpers` guard
  (`tests/test_python_http_mr_list_decision_work.py:388-411`) poisons
  `validate_sealed_commit`, `validate_http_merge_request_record`,
  `validate_merge_request_subject`, `read_remote_merge_request_path`,
  `read_remote_branch_record`, `collect_object_records`, `merge_request_object_set_digest`,
  `merge_request_subject_digest`, and `read_json`; the live route reaches none of them. Source
  inspection agrees — `remote_project_stable_id` reads through `_open_directory_nofollow`, not
  `read_json`.

### 2. Reservation before allocation; exact / minus-one / plus-one and stable counters reach the live route and close to zero — `PASS`

- Every owner reserves first: the pinned entry slot is acquired **before** the record it will own is
  parsed (`codefire:12227-12233`); session index entries before session construction, with rollback
  on failure (`codefire:12191-12213`); branch and descriptor index entries before their reads
  (`codefire:12253-12256`, `codefire:12288-12295`); the frontier frame before each push and the
  `seen`/`ordered` slots before insertion (`codefire:12492-12508`, `codefire:12547-12560`); the
  subject field-set and the per-record core view before they are built (`codefire:12451-12461`,
  `codefire:12634-12641`); the envelope before the projection list and each entry's exact canonical
  bytes — plus the separator byte — before it is appended (`codefire:12673-12695`).
- Ownership handoff is correct, not double-charged: `_AccountedHTTPProjection.__init__`
  (`codefire:11377-11387`) **takes over** already-acquired bytes rather than re-acquiring, and
  `project()` zeroes `_projection_retained` only after the payload exists (`codefire:12696-12702`),
  so a raising constructor leaves the debt with `close()`.
- `test_exact_minus_one_and_plus_one_limits_bound_every_stage`
  (`tests/test_python_http_mr_list_decision_work.py:1327-1396`) drives 11 dimensions —
  `aggregate_bytes`, `object_records`, `graph_nodes`, `graph_edges`, `maximum_frontier`,
  `snapshot_revalidation_records`, `snapshot_revalidation_bytes`, `retained_bytes`,
  `temporary_bytes`, `counter_values`, `response_bytes` — at exact, exact+1 (byte-identical
  response, no diagnostic, `terminal_reason == "complete"`), and exact−1 (413, exact
  `dimension`/`limit`/`observed`, `terminal_reason == "budget_exceeded"`, no partial list). All pass
  at the target.
- Counters are exact equations, not bounds
  (`tests/test_python_http_mr_list_decision_work.py:1398-1446`): `files_read ==
  2·unique_objects + MR_COUNT + 2`, `file_bytes_read == 2·object_bytes + mr_bytes + branch_bytes +
  descriptor_bytes`, `record_reads == unique_objects + 1`, `memo_misses == unique_objects`,
  `edge_occurrences == graph_edges`, `snapshot_revalidation_bytes == object_bytes`,
  `cumulative response_bytes == Content-Length`.
- Closure to zero was confirmed independently on **every** path this review exercised — success,
  400 validation failure, 400 invalid-JSON, 503 deadline, subject-change, and the uncaught-`OSError`
  empty-wire path: all current dimensions zero and `open_permit_count == 0` in every observation.

### 3. Deterministic order, compatible bytes/status, single-fault precedence, legacy/cross-project, open/stale — `PASS with one exception`

- Order is the pre-fix order: `accounted_sorted_json_paths`
  (`codefire@7327e2a:1507-1551`) scandirs, filters `*.json`, and sorts names under an accounted sort
  scratch — the same set and order as the pre-fix `sorted(glob("*.json"))`.
- Bytes and status match: `test_cold_and_reused_route_publish_identical_bytes_and_work`
  (`tests/test_python_http_mr_list_decision_work.py:895-921`) compares the live body field-for-field
  against `legacy_request_list`; this review independently confirmed `200 OK`, identical list length,
  and byte-identical stable responses across cold/warm and across a
  failure→repair→success sequence.
- Within a single record the check order is preserved exactly: `validate_records`
  (`codefire:12309-12343`) does source URL parse → target URL parse → project-ownership → legacy
  cross-project → `validate_record` → branch read → status, and `validate_record`
  (`codefire:12344-12411`) does id component → sealed refs in `source_head`,
  `target_head_at_request`, `applied_head` order → endpoint equality → subject, matching
  `codefire@7327e2a:11968-11998` conjunct for conjunct, including the short-circuit position of
  `target_project_id` and `source_object_set_digest`.
- Single-fault parity confirmed independently across twelve fault classes, including six not in the
  suite: JSON-list record (`remote_merge_request must be an object`), missing `source_head`,
  subject with an extra field, bad branch `version`, `applied_head` pointing at an absent object,
  and a non-`.json` neighbour file (correctly ignored, `200 OK`). All produced the pre-fix message
  verbatim. An absent consulted branch record still yields `200 OK` with persisted status on both
  routes.
- Legacy/cross-project rejection, mismatched endpoints, foreign target, diverged subject digest, and
  drifted escrow digest all keep their exact pre-fix diagnostics
  (`tests/test_python_http_mr_list_decision_work.py:1025-1123`).
- Open/stale parity holds (`tests/test_python_http_mr_list_decision_work.py:974-1023`); the status
  computation at `codefire:12332-12342` is the pre-fix comparison unchanged.
- **The exception** is response-status compatibility on symlinked authority records: `200 OK` →
  zero-byte reply (MUST-2). A directory in place of a record, by contrast, is handled correctly
  (typed 400, `bounded JSON file is invalid: <path>`).

### 4. Reported whole-snapshot capture risk — `FAIL` (the risk is real and confirmed)

Multi-fault fixtures were constructed as required and the answer is unambiguous: **yes, the target
changes observable first-error precedence relative to the pre-fix route.** Eight of ten multi-fault
fixtures diverge; the divergence is caused by `capture_authority` performing every read and every
`validate_persisted_record_version` before `validate_records` performs any URL/sealed/subject check
(`codefire:12215-12243` vs `codefire:12309-12343`). Controls in which the earliest filename carries
the read/version fault, or in which both faults are the same class, agree exactly — confirming the
mechanism rather than a fixture artefact. Every divergent case still fails closed with no partial
publication and no ownership debt. Full evidence and the correction options are in MUST-1.

### 5. Aggregate accounting slope, session reuse, memo hits, 1×/3×/10× work, duplicate reads — `PASS` (with SHOULD-1)

- Exactly **one** session for the whole request across every cardinality tested (1, 8, 24, 80 merge
  requests sharing one escrow/object root), and one session in the 8-MR default fixture where all
  source and target references resolve to the same pinned root.
- **Zero** duplicate physical reads at every cardinality, measured by instrumenting
  `read_json_accounted` and recording every path: no path was read twice. The consulted branch head
  is read exactly once per request, not once per merge request
  (`tests/test_python_http_mr_list_decision_work.py:873-893`), and the project descriptor is
  memoized per project root (`codefire:12266-12274`).
- No hidden validation: `object_records` (22), `memo_misses` (22), `graph_nodes` (9),
  `semantic_validations` (31), and `snapshot_revalidation_records` (22) are **constant** from 1 to
  80 merge requests; `memo_hits` rises 90 → 1,986, which is the memo doing its job.
- 1×/3×/10× (8 → 24 → 80): `files_read` 54 → 70 → 126, i.e. exactly one added read per added merge
  request — its own record.
- No regression versus the defect: at default budgets the target beats the pre-fix route
  (9.63 s vs 11.36 s at N=80; 24.60 s vs 56.10 s at N=200), and the suite's cold per-reference probe
  shows the pre-fix shape re-reading the shared spine once per merge request
  (`tests/test_python_http_mr_list_decision_work.py:837-870`).
- The one soft spot is the un-memoized per-MR escrow re-traversal and re-digest, linear in merge
  requests even when every merge request shares one `source_head` — SHOULD-1.

### 6. Request isolation on the actual server protocol — `PASS`

- Wire pipelining is genuinely unreachable: the handler advertises `protocol_version == "HTTP/1.0"`,
  and two MR-list requests sent in one `sendall` on one connection produced **1** response, **1**
  decision-work observation, and no debt. The suite makes the same point and then proves isolation
  where it is actually decided — by poisoning `_mr_list_deadline_active` immediately before
  `handle_one_request` dispatches
  (`tests/test_python_http_mr_list_decision_work.py:1627-1721`); every neighbouring route observed
  the flag as `False`, i.e. reset, never inherited.
- The flag is reset per request in `handle_one_request` alongside every sibling flag
  (`codefire:15359-15362`), set only on the MR-list branch (`codefire:15544`), and consulted only
  through the ordered dispatcher (`codefire:14891-14901`).
- Independently measured eight-request sequence on one live server, with a deadline poisoned only
  during request 1 and an escrow object corrupted only for request 3:

```text
1 mr-list, deadline poisoned   503 request_deadline_exceeded  expired_phase="merge request list planning"
                               terminal_reason=deadline_exceeded  permits=0  debt={}  foreign_phases=[]
2 mr-list, clean               200 OK  merge_requests  permits=0  debt={}  9 MR phases  foreign_phases=[]
3 mr-list, corrupt escrow      400 invalid object JSON: .../CF-BLOB-4bea....json
                               permits=0  debt={}  7 MR phases  foreign_phases=[]
4 mr-list, repaired            200 OK  merge_requests  permits=0  debt={}  foreign_phases=[]
5 GET /branches                200 OK  branches   0 MR phases, only "branch list *" phases
6 GET /doctor                  200 OK  doctor     0 MR phases, only "doctor *" phases
7 POST .../main/upload         400 upload request must include branch, head, and objects
                               0 MR phases, only "upload response encoding"
8 mr-list final                200 OK  merge_requests  permits=0  debt={}  foreign_phases=[]
```

  Eight distinct ledger objects for eight requests; requests 2, 4, and 8 are byte-identical modulo
  `Date`. No ledger, deadline, phase, or ownership state crossed a request boundary — including
  across a deadline failure, a subject/content failure, and the neighbouring Doctor, branch-list,
  and upload routes.

## Commands and results

All commands run at target `37f7427d46d2cf3ad66de3c1885ef4c0527cae70` in the session worktree
(`.../worktrees/cfr168-python-http-mr-list-session-20260730-mr-list-session`), which was clean
before and after.

| Command | Result |
|---|---|
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest tests.test_python_http_mr_list_decision_work -v` | **OK — Ran 18 tests in 74.739s**, 18/18 pass (independently run here) |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest -v` on the 8 affected `test_codefire_cli` cases | **OK — Ran 8 tests in 229.267s** (see list below) |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile codefire tests/test_python_http_mr_list_decision_work.py tests/test_codefire_cli.py` | passed |
| `python3 docs/development/tools/check_source_docs.py` | `Source docs check passed: 44 modules, 34 commands, 12 object kinds.` |
| `python3 docs/development/tools/check_project_docs.py` | `Project docs check passed; 540 Markdown files inventoried.` |
| `git diff --check` | clean, rc=0 |
| `git status --porcelain` (session worktree) | empty before and after all runs |
| Multi-fault precedence probe vs real baseline `codefire` (10 fixtures) | ran; **8/10 diverge** → MUST-1 |
| Single-fault parity probe (12 fault classes) | ran; 9 exact-parity, 1 improved (typed 400 for a directory), **2 empty-wire** → MUST-2 |
| Symlinked-authority probe across `/merge-requests`, `/branches`, `/doctor` | ran; zero-byte reply on all three → MUST-2 scope |
| Aggregate slope probe (1/8/24/80 MRs, one escrow root) | ran; 1 session, 0 duplicate reads, constant validation counters → verdict 5 / SHOULD-1 |
| Default-budget scale probe (80 / 200 MRs, `http_v1` defaults) | ran; `200 OK` both, target faster than pre-fix → SHOULD-1 context |
| Isolation probe (8-request sequence + pipelining) | ran; 8 distinct ledgers, no leakage → verdict 6 |

The `test_codefire_cli` cases run: `test_upload_list_remote_clone_and_request_merge`,
`test_merge_request_subject_escrow_survives_source_removal_and_rejects_tamper`,
`test_http_remote_upload_list_and_clone`,
`test_request_list_and_review_reject_invalid_merge_request_refs`,
`test_https_remote_upload_list_clone_and_show`,
`test_http_remote_payload_tamper_writes_no_layout_or_nonce`,
`test_http_remote_request_signature_rejects_replayed_nonce`,
`test_python_http_bundle_deadline_boundaries_and_poll_phases`.

The last of those is the method the Doctor/branch independent review recorded as MUST-1; it **passes
at this target** because the corrective commit `701469f` ("test: update branch-list deadline
expectation") is in this lane's ancestry. That correction is still owed to `main`.

Selection note: `cmd_request_list` serves `cf://` URLs through a separate local code path
(`codefire:33482-33515`, unchanged by this slice) and only `cf+http`/`cf+https` URLs reach
`handle_http_request_list`. The eight cases above are the complete affected set — the five `cf+http`
tests plus the three `request-list`/merge-request tests that exercise the shared authority, escrow,
and validation code.

### Not run in this review

- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p 'test_python_*.py'` — the
  Python-pattern discovery run. The manager reports 481/481; this review did **not** run it and does
  not claim it. Skipped under the memory-constrained wave instruction.
- `python3 -m unittest discover -s tests -v` (full suite) and `./demo.sh` — not run, same reason.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` — **not runnable**: `cargo` and `rustc` are both absent from this sandbox
  (`cargo --version` → `command not found`), a capability gap already documented in
  `docs/development/ai/swarm-results/claude-cfr168-review-20260729/review.md:332-344`. The reviewed
  diff touches zero Rust files (`codefire` and one Python test only), so no Rust behaviour is at
  risk; the commands remain owed at integration on a Rust-capable host.

## Integration conflict map

The target is a composition probe tip, so the manager must extract the MR-list slice
(`7327e2a..37f7427`) rather than merging the branch. Within `codefire` the slice touches six
regions; four are shared with lanes already in this probe's ancestry and will conflict when the
lanes are integrated in a different order.

| Region at `37f7427` | Change | Conflict risk |
|---|---|---|
| `codefire:11441-11454` | new `AccountedHTTPMergeRequestList`, inserted directly after `AccountedHTTPBranchList` | **High** — `AccountedHTTPBranchList` is added by the branch lane (`7e87665`). Textually adjacent insert. |
| `codefire:12104-12822` | `MERGE_REQUEST_LIST_ENVELOPE_FIELD`, `_HTTPMergeRequestListOperation`, `collect_http_merge_request_list_payload`, rewritten `handle_http_request_list` | **Low** — replaces the 16-line pre-fix body; no other lane touches it. |
| `codefire:14153` | `AccountedHTTPMergeRequestList` added to `_BoundedCanonicalJSONFreezer`'s owner set | **High** — the same set literal is extended by the Doctor, bundle, and branch lanes. |
| `codefire:14891-14901` | `read_route_phase` gains the `_mr_list_deadline_active` arm, ordered after `branch list` | **High** — the dispatcher itself is introduced/extended by the Doctor and branch lanes. Arm order is not semantically load-bearing (at most one flag is ever set), but the hunk will conflict. |
| `codefire:15359-15362` | `_mr_list_deadline_active = False` in the `handle_one_request` reset block | **High** — same block is extended by every read-route lane. |
| `codefire:15544-15550` | `do_GET` `merge-requests` arm sets the flag and passes ledger + deadline | **Medium** — the `do_GET` route table is edited by the Doctor, bundle, and branch lanes; this arm is distinct. |
| `tests/test_python_http_mr_list_decision_work.py` | new file, 1824 lines | **None**. |

Cross-lane obligations the manager still owns:

- `tests/test_codefire_cli.py:6273-6280` — the branch-list deadline correction (`701469f`) is
  present in this lane's ancestry but not in `main`. It must land with, or before, the branch lane.
- **MUST-2's fix belongs in `do_GET`/`do_POST`**, which are outside the MR-list write set and shared
  with Doctor, branch-list, bundle, and upload. It should be one separate corrective commit covering
  all read routes at once, not four per-lane edits.
- MUST-1's chosen resolution touches
  `docs/development/ai/swarm-batches/cfr168-python-http-mr-list-session-20260730.md` (the
  "preserve error precedence" clause) and
  `tests/test_python_http_mr_list_decision_work.py:1025-1123`. Both are inside the MR-list lane.

## Residual risk

- **Multi-fault diagnostics (MUST-1).** Until resolved, operators debugging a repository with more
  than one broken merge request will be shown a different first error than the pre-fix route showed,
  and the suite will keep asserting parity. No correctness or disclosure risk: every path fails
  closed at 400 with no partial list and no ownership debt.
- **Symlinked authority (MUST-2).** Any `/merge-requests`, `/branches`, or `/doctor` request whose
  pinned authority record is a symlink returns nothing at all. Operationally this looks like a
  crash and will be diagnosed as one. No state leaks and no permit is held open, so it is not a
  resource-exhaustion vector.
- **Cardinality (SHOULD-1).** The per-merge-request escrow re-traversal keeps `bytes_hashed`,
  `aggregate_bytes`, and `graph_edges` linear in merge-request count. Cumulative budgets are far
  away (1 GiB `aggregate_bytes`, 400,000 `graph_edges`), but the enforced 30 s default deadline is
  reachable at a few hundred merge requests on a shared root — where the pre-fix route returned a
  slow `200 OK`. Failure is a typed 503 with no partial list, which is the contract-intended
  behaviour, so this is a capacity-planning risk rather than a defect.
- **Unverified suites.** The Python-pattern discovery run (481/481), the full Python suite,
  `./demo.sh`, and all three Cargo commands were not run here. The Cargo gap is environmental and
  the diff contains no Rust; the Python gaps are this wave's memory constraint and should be closed
  before integration on a host with headroom.
- **Composition, not isolation.** Every measurement above was taken at `37f7427`, which already
  contains the Doctor, upload, and branch slices. The MR-list slice was never exercised against a
  `main` that lacks them. Once the shared dispatcher, reset block, freezer owner set, and `do_GET`
  route table are reconciled during integration, the 18 focused tests and the eight
  `test_codefire_cli` cases must be re-run on the integrated tree.
- **Unaudited neighbours.** MR mutation routes (`request-merge`, `request-review`, apply), GC, and
  the `cf://` local `cmd_request_list` path were read only where they bear on this route. They are
  unchanged by this slice and were not independently audited.
