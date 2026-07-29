# CFR-168 / CFB-136 Independent Review of the Actual Current State

| Field | Value |
|---|---|
| Task | `cfr168-current-review` (batch `current-state-closure-audit-20260729`) |
| Reviewer | Claude Opus 5, independent reviewer lane |
| Snapshot | `e2a218e015953f082245f36dbc6aee906b837e5a` (`chore: checkpoint current classified worktree`) |
| Contracts reviewed | `docs/development/code-review-issues/cfr-168.md`, `docs/development/bug-issues/cfb-136.md`, `adr/ADR-020-decision-equivalent-work-budget.md`, `docs/development/ai/task-packets/json/cfr-168-decision-work-budget.json` |
| Decision | **`REQUEST_CHANGES`** |
| Confidence | High for the Rust/Python production-wiring findings (static tracing plus executed runtime probes); Medium for the metrics/documentation findings; Medium-Low for the two boundary items explicitly marked *unproven reachability* |

## 0. Scope, method, and what this review does not claim

The snapshot is verified as the assigned base: `git log --oneline -3` in the assigned worktree
resolves `e2a218e` on top of `f164412`, and `git status --short` is clean at start and at handoff
(except the two deliverables). The user's dirty main worktree was never read or touched, and no
existing ref was created, moved, or deleted.

Reviewed surface (source of truth, not the issue log's self-report):

- Rust: `crates/codefire-core/src/{work_budget.rs,lib.rs}`, `crates/codefire-store/src/lib.rs`,
  `crates/codefire-cli/src/{main.rs,http.rs,remote.rs,remote/doctor.rs,remote/project.rs,signatures.rs,storage.rs,metrics.rs,compatibility.rs,doctor/checks.rs}`.
- Python fallback: `codefire` (33,456 lines), `tests/` (31,005 lines across 32 files).
- Docs/contracts: `docs/{04_cli_spec.md,09_index_trace_policy.md,10_merge_remote_server.md,automation_interface.md}`,
  `adr/ADR-020`, `docs/development/current-cycle-burn-down-plan.md`, `docs/development/history.md`.

This review deliberately treats the ~1,068-line "Implementation Evidence" log in `cfr-168.md` as a
*claim* to be checked, not as evidence. Every finding below is anchored to a path and symbol at this
snapshot, and the four Must findings that concern reachable production behaviour are supported by
executed probes, not only by reading.

Explicitly **not** claimed: this is not an exhaustive line-by-line audit of 156k lines of Rust plus
33k lines of Python. I did not attempt to falsify every internal ownership/cancellation invariant of
the accepted Core/Store slices; the Core and Store nuclei are the best-tested parts of the change and
I found no defect in them. The findings are concentrated where the contract's own acceptance criteria
are checkable end-to-end: which production command surfaces are actually bounded by a ledger, and
whether the tests that are green exercise those surfaces.

### Validations executed in this worktree

| Validation | Command | Result |
|---|---|---|
| Core unit | `cargo test -p codefire-core --lib` | **89 passed, 0 failed** |
| Store unit | `cargo test -p codefire-store --lib` | **196 passed, 0 failed** |
| Util unit | `cargo test -p codefire-util --lib` | **3 passed, 0 failed** |
| CLI unit | `cargo test -p codefire-cli --bins` | **768 passed, 0 failed** (42.6 s) |
| Format | `cargo fmt --check` | pass (exit 0) |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | pass (exit 0, 0 warnings, 1m15s) |
| Project docs | `python3 docs/development/tools/check_project_docs.py` | pass (522 Markdown files) |
| Source docs | `python3 docs/development/tools/check_source_docs.py` | pass (44 modules / 34 commands / 12 object kinds) |
| Python production-path focused | `python3 -m unittest tests.test_python_sealed_production tests.test_python_external_production` | **13 passed** (13.4 s) |
| Python unwired-owner focused | `python3 -m unittest tests.test_python_ancestry_owner` | **13 passed** (7.6 s) |
| Runtime probe A | Python HTTP doctor route ledger identity (script in `/tmp`, repo untouched) | reproduced M2 |
| Runtime probe B | Python production upload ancestry read count (script in `/tmp`, repo untouched) | reproduced M1 |

Deliberately skipped as expensive and already covered by the manager's baseline: the full
`python3 -m unittest discover -s tests` sweep (~650 s per `history.md`), `demo.sh`, and the
integration test binaries. The Rust unit counts I measured (CLI 768 / Core 89 / Store 196 / Util 3)
match the most recent numbers recorded in `docs/development/history.md`, so the green baseline is
independently confirmed rather than restated.

**The green baseline is real. It is also not sufficient, and section 1 explains why: the suites that
pass largely do not cover the surfaces that the frozen acceptance criteria require to be bounded.**

---

## 1. Implementation defects (Must)

### M1 — Python production HTTP upload still answers fast-forward with a full reachable-history map, unbudgeted and undeadlined

`handle_http_upload` is the wired Python upload handler; it receives `decision_work` and `deadline`
from `do_POST` (`codefire:14601`). Inside the project gate it decides fast-forward with:

- `codefire:12205-12207` — `is_ancestor_from_objects(tmp_objects, current["head"], head)`
- `codefire:11189-11190` — `is_ancestor_from_objects` → `ancestor in ancestor_distances_from_objects(...)`
- `codefire:11175-11187` — `ancestor_distances_from_objects`: BFS over the **entire** reachable set,
  one `load_object_from_root` per node, **no ledger parameter, no deadline parameter, no budget**

This is verbatim the defect the root was opened to fix. `cfr-168.md:56` freezes it as
characterization: *"`remote::is_ancestor_in_objects`は一件のancestor判定にも`ancestor_distances`で全到達mapを作る"*.

Executed probe (300-commit linear history built with the module's own `object_id`/`object_digest`,
`load_object_from_root` counted by rebinding it in `ancestor_distances_from_objects.__globals__`):

```
is_ancestor(immediate parent, head) = True
object reads performed: 300 for reachable history of 300
```

The answer is decidable from one read of `head`. 300 reads were performed, none of them charged.

The accounted replacement exists and is dead: `AccountedHTTPUploadAncestry` occupies
`codefire:10672-11174` (503 lines) and has **zero** production constructors — its only non-test
construction sites are `codefire:23991` (inside `AccountedHTTPUploadExistingTerminalOwner`, itself
unwired) and `codefire:28654` (inside `AccountedHTTPUploadSealedAncestry`, also unwired).

Violated: CFR-168 AC 2 ("output limitに達する前の内部read/validation/materializationもbudgetで停止する");
ADR-020 invariant `bounded-before-effect`; task-packet invariant `no-recursive-history`'s
"visit each subject at most once per command session" intent — the visit happens, but outside any session.

By contrast the Rust HTTP upload/apply routes **are** correct here: `http.rs:7419` and `http.rs:9698`
use `is_ancestor_in_provisional_subject_with_poll` (`remote.rs:3729`), which charges frames and polls
the deadline. So this is a Rust/Python parity defect on a wired production route, not an
"everything is equally unfinished" observation.

### M2 — Python HTTP `GET /doctor` runs on a second, `local_v1` ledger, defeating the request-wide HTTP bound

- `codefire:14402-14405` — `do_GET` calls `handle_http_doctor(storage_root, org, app, quick=...)`.
  It passes **neither** `self.decision_work` **nor** `deadline`. (Compare `codefire:14411-14418`,
  where the bundle route does pass both.)
- `codefire:11491-11499` — `handle_http_doctor` → `remote_doctor_payload(project_root, url, quick=...)`.
- `codefire:14950-14953` — `remote_doctor_payload` constructs its own ledger:
  `DecisionWorkLedger(DecisionWorkProfile.local_v1(), "remote doctor")`.
- `codefire:14882-14907` — the returned `AccountedDoctorPayload` binds its retained bytes to *that*
  ledger and terminates *that* ledger in `close()`.

Executed probe:

```
status: 200
payload type: AccountedDoctorPayload
request-ledger current BEFORE: {}
request-ledger current AFTER : {}
payload has its own ledger: True   is request ledger: False
payload ledger profile retained limit: 1073741824   (local_v1, 1 GiB)
request ledger profile retained limit:   33554432   (http_v1, 32 MiB)
payload retained bytes charged: 695
```

The HTTP request ledger records **zero** work for a route that read the project layout, enumerated
branches and merge requests, validated sealed roots, and built a report. The work is charged to a
throwaway ledger whose concurrent `retained_bytes` ceiling is **32× larger** than the HTTP profile's.

Violated: ADR-020 §1's single shared per-request ledger; ADR-020's required proof *"one cumulative
ledger proves that N heads/phases cannot each consume the full command budget"* — here a single phase
gets a second full budget; CFR-168 AC 2. The Rust server does this correctly at `http.rs:5811-5818`
(`diagnose_remote_project_with_work_and_poll(..., context.ledger, || deadline.check(...))`), which
makes this a concrete cross-runtime divergence on a live route.

### M3 — Seven of nine Python HTTP routes have no request-ledger accounting, no deadline, and (for the mutating ones) no pre-effect response preflight

Only two of the Python routes receive `self.decision_work`:

| Route | Call site | Ledger | Deadline |
|---|---|---|---|
| `GET  …/branches` | `codefire:14397` | ✗ | ✗ |
| `GET  …/doctor` | `codefire:14402` | ✗ | ✗ |
| `GET  …/branches/{b}/bundle` | `codefire:14411` | ✓ | ✓ |
| `GET  …/merge-requests` | `codefire:14420` | ✗ | ✗ |
| `POST …/branches/{b}/upload` | `codefire:14601` | ✓ | ✓ |
| `POST …/gc` | `codefire:14613` | ✗ | ✗ |
| `POST …/merge-requests` | `codefire:14615` | ✗ | ✗ |
| `POST …/merge-requests/{id}/review` | `codefire:14617` | ✗ | ✗ |
| `POST …/merge-requests/{id}/apply` | `codefire:14619` | ✗ | ✗ |

Two consequences are specific and checkable:

1. **`GET …/branches` still exhibits the exact CFB-136 shape the bug report names.**
   `codefire:11483-11487` loops the branch directory and calls
   `validate_sealed_commit(dirs["objects"], branch["head"])` per head. `validate_sealed_commit`
   (Rust twin at `codefire-store/src/lib.rs:5042-5045`, Python equivalent) builds a **fresh** cache
   per head. CFB-136's Observation says *"現行branch listはheadごとにfresh cacheを作るため cross-head
   reuseを十分活用していない"* and its AC 4 requires cache hit/miss/revalidated/invalidated counters
   plus *"Rust/Python、branch/Doctor/storage/ancestryで同じmutation fixtureとdiagnosticを使う"*. The Rust
   local `branch list` was genuinely fixed (`main.rs:11467-11560`: one
   `begin_sealed_commit_validation_session_with_recoverable_ledger`, all heads validated, single
   `finalize()`); the Rust HTTP branch list was genuinely fixed (`http.rs:5929-…`:
   accounted directory scan + `with_http_sealed_validation`). The Python side has neither a shared
   session nor counters, so the CFB-136 parity criterion is unmet, not merely partially met.

2. **The three mutating MR routes commit durable authority with no pre-effect response ownership.**
   In `do_POST` the response is only built *after* the handler returns: `codefire:14680-14687`
   (`if prepared is None: prepared = self.prepare_json(status, payload)`), and that call sits
   **outside** the `except ResourceBudgetExceeded` / `except CFError` handlers that precede it. Only
   the upload route freezes success and durable-503 bodies before the effect, via the
   `_HTTPMutationResponseSlot` (`codefire:14599-14609`). ADR-020 §5 requires the whole publication
   lane to stop *before* publication; three of four mutating Python routes are outside that lane.

### M4 — Named local and file-transport Rust surfaces remain entirely unbudgeted, with the original characterization defects intact

CFR-168's Required Fix says *"command-scoped validated graph/cacheを branch/storage/doctor/ancestryで
共有する"*. Only `branch list` and local `doctor` are wired. Every `DecisionWorkLedger::new` in the CLI
crate outside `#[cfg(test)]` code is: `main.rs:11468` (`list_branches`), `doctor/checks.rs:1403`
(`run_checks_with_sealed_validation_hook`), and `http.rs:{345,710,5213}` (HTTP request/limiter/parse).
`crates/codefire-cli/src/remote.rs` contains **no** production ledger at all — its only
`DecisionWorkLedger::new` is `remote.rs:3937`, inside `#[cfg(test)] mod object_set_digest_owner_tests`.

Concretely:

- **`cf merge`** — `main.rs:5227` `common_ancestor(&objects, &source_head, &target_head)` →
  `main.rs:10756-10758` → `main.rs:10767-10793` `ancestor_distances`: full reachable
  `HashMap<String, AncestorInfo>`, one `commit_info` read per node, no ledger, no bound. Same shape as
  M1, in Rust, on a local command.
- **`cf remote upload` (file transport)** — `remote.rs:170-173` `upload_branch` takes no ledger;
  `remote.rs:178` `validate_sealed_commit` (fresh cache); `remote.rs:293-297`
  `enforce_remote_commit_signature_policy` (`signatures.rs:137-141`, no ledger — the accounted twin
  `enforce_remote_commit_signature_policy_in_provisional_subject_with_poll` at `signatures.rs:177` is
  used only by HTTP); `remote.rs:311` and `remote.rs:372` `is_ancestor_in_objects`
  (`remote.rs:3721-3727` → `ancestor_distances`); `remote.rs:341` `copy_object_graph`.
- **`cf remote doctor cf://…`** — `remote/doctor.rs:471-514` `run_remote_doctor` routes the file
  transport to `diagnose_remote_project` (`remote/doctor.rs:518`), a **second, unaccounted
  implementation of the same diagnostic surface**; the accounted
  `diagnose_remote_project_with_work_and_poll` (`remote/doctor.rs:1573`) is reachable only from the
  HTTP route (`http.rs:5811`). Note in fairness: the legacy path's `inspect_sealed_roots`
  (`remote/doctor.rs:1966-2010`) *does* use one shared session with `finalize()`, so it is
  snapshot-safe for CFB-136; it is unbudgeted, not unsafe.
- **`cf storage`** — `storage.rs:466-560` `scan_object_store_record_documents` with unbounded
  `warnings`, `by_type`, and `largest_objects` growth and no ledger. `storage` is named verbatim in
  the Required Fix.
- **`cf scan`** — `main.rs:6400-6408` takes a `RepoLock` and a fresh
  `SealedCommitValidationCache::default()`, then `main.rs:6457` `validate_sealed_commit_pinned_with_cache`,
  with no ledger. Separately, the indexed Trace evaluator *is* implemented and *is* on the production
  path (`codefire-core/src/lib.rs:2264` `TraceEvaluationIndex::compile`, reached through
  `build_scan_result` → `build_scan_result_against_base_with_work`), but its `TraceWorkCounters` are
  discarded: `build_scan_result_against_base_with_work` has **no CLI caller**
  (`grep build_scan_result_against_base_with_work crates/codefire-cli` → no hits), and
  `metrics.rs:212-232` `scan_metrics` still reports `cache_unimplemented()`
  (`metrics.rs:481-491`) even though scan does use a validation cache.

### M5 — Python production upload's signature-history enforcement is unaccounted, and both accounted signature owners are dead code

`codefire:12111` — `handle_http_upload` calls `enforce_remote_commit_signature_policy(project_root, tmp_objects, head)`.
`codefire:6587-6617` is iterative (explicit LIFO stack, so `no-recursive-history` holds) but takes no
ledger and no deadline: the policy `read_json`, one `load_object_from_root` per commit, and the
unbounded `seen` set and `stack` are all uncharged.

The accounted owners are present and unreachable:

- `AccountedRemoteSignaturePolicyAuthority` — `codefire:4505-5739` (1,235 lines), no production constructor.
- `AccountedCommitSignaturePolicy` — `codefire:5740-6570` (831 lines), no production constructor.
- The only bridge into them is `AccountedServerVerification.__init__`'s private
  `_signature_policy_authority` kwarg (`codefire:6910-6913`), which
  `prepare_server_verification`/`run_server_verification` (`codefire:8405-8465`) never supply.

Note in fairness: `AccountedServerVerification` itself **is** wired — `codefire:12114-12120` passes
`ledger=decision_work, deadline=deadline`. So external verification is bounded; signature-policy
enforcement immediately before it is not.

### M6 — The Python test suite's centre of mass is code with zero production callers, so a green Python gate cannot evidence CFR-168 closure

Eleven public factory functions have **no non-test caller anywhere in `codefire`**
(verified by grepping each name followed by `(` and excluding its own `def`):

`prepare_remote_object_set_digest` (`codefire:16801`), `prepare_http_upload_existing_publication_input`
(`:17570`), `prepare_http_upload_response_input` (`:18287`), `prepare_http_upload_parser_scalar`
(`:18569`), `bind_http_upload_request_source` (`:18694`), `prepare_http_upload_semantic` (`:20400`),
`prepare_http_upload_existing_project_gate` (`:25704`),
`prepare_http_upload_missing_project_bootstrap_transaction` (`:26689`),
`prepare_http_upload_existing_pair_semantic_carrier` (`:28278`),
`prepare_http_upload_existing_effect_input` (`:28287`), `prepare_http_upload_sealed_ancestry` (`:28768`).

Measured spans of the owner classes reachable only from those factories or from each other:

| Class | Lines | Span |
|---|---|---|
| `AccountedRemoteSignaturePolicyAuthority` | 1,235 | 4505–5739 |
| `AccountedHTTPUploadExistingProjectGate` | 1,093 | 24611–25703 |
| `AccountedHTTPUploadExistingPairSemanticCarrier` | 1,048 | 26774–27821 |
| `_HTTPUploadSemanticTerminalProbe` | 956 | 19444–20399 |
| `AccountedHTTPUploadExistingTerminalOwner` | 952 | 23650–24601 |
| `AccountedHTTPUploadMissingProjectBootstrapTransaction` | 847 | 25842–26688 |
| `AccountedCommitSignaturePolicy` | 831 | 5740–6570 |
| `AccountedHTTPUploadExistingPublicationInput` | 757 | 16813–17569 |
| `AccountedHTTPUploadSemantic` | 741 | 18703–19443 |
| `AccountedHTTPUploadResponseInput` | 602 | 17685–18286 |
| `AccountedHTTPUploadAncestry` | 503 | 10672–11174 |
| `AccountedHTTPUploadExistingEffectInput` | 456 | 27822–28277 |
| `AccountedRemoteObjectSetDigest` | 364 | 16437–16800 |
| `AccountedHTTPUploadSignatureVerificationComposite` | 339 | 23311–23649 |
| `AccountedHTTPUploadPlanner` | 277 | 8832–9108 |
| `AccountedHTTPUploadParserScalar` | 224 | 18345–18568 |
| `AccountedHTTPUploadSealedAncestry` | 203 | 28565–28767 |
| `AccountedHTTPUploadRequestSource` | 118 | 18576–18693 |
| `_HTTPUploadSealedValidationReservation` | 97 | 28314–28410 |
| `_HTTPUploadMissingBootstrapHandoff` + reducer capability | 44 | 25798–25841 |
| **Total** | **11,687** | **≈35% of `codefire`** |

On the test side, 22 of the 30 `tests/test_python_*.py` files (**≥12,547 of 17,433 lines, ≈72%**)
reference at least one of these unwired symbols. The true figure is higher: three of the eight files
I could not classify by symbol name import another file's harness
(`tests/test_python_existing_response_composition.py:3` does
`from tests import test_python_existing_effect_input as effect_harness`), and
`tests/test_python_sealed_borrow.py` targets the sealed **borrow** API which no production consumer
receives.

The pairing that makes this a blocker rather than a style note:
`python3 -m unittest tests.test_python_ancestry_owner` passes **13/13** in this worktree, while the
production ancestry path it nominally protects performs the unbudgeted 300-read traversal of M1. The
review contract's requirement — *"Verify that tests exercise production paths and cannot pass by
testing only scaffolding"* — fails here in the most literal way available.

To be fair to the work: this is not accidental. `cfr-168.md` and `history.md` repeatedly and honestly
label these slices "production-unwired" and maintain a Freeze. The finding is that the accumulated
green numbers (`canonical Python 631/631` etc.) are therefore **not** closure evidence, and must not
be read as such at the Close gate.

---

## 2. Missing or weak tests (Should)

### S1 — `branch list` cache metrics are self-inconsistent and omit a counter CFB-136 requires

`metrics.rs:196-208` sets `entry_count: saturating_usize(store_work.document_reads)` and
`miss_count: saturating_usize(store_work.document_reads)` — the two fields are *always equal by
construction*, and `entry_count` is a count of reads, not of cache entries. No test asserts either
field: the only `entry_count` assertions in the CLI crate are for the unrelated nonce cache
(`signatures.rs:3333`, `tests.rs:8400`).

Additionally, CFB-136 AC 4 requires *"cache countersはhit/miss/revalidated/invalidatedをaggregateで示し"*.
`grep -n invalidated crates/codefire-store/src/lib.rs crates/codefire-cli/src/metrics.rs` returns
**no hits**: `SealedValidationWork` (`codefire-store/src/lib.rs:5528-5537`) has no `invalidated`
counter and `CacheMetrics` (`metrics.rs:33-41`) has no `revalidated`/`invalidated` field, even though
`revalidated_documents`/`revalidated_bytes` are tracked and are projected as *decision* counters at
`metrics.rs:167-186`.

### S2 — The metrics-orthogonality test is structurally unable to falsify its invariant

`tests.rs:627-648` constructs `without_metrics` synthetically from the *same* `result.branches` value
that `with_metrics` is built from, then asserts `with_metrics["branches"] == without_metrics["branches"]`.
That is a projection test. The task packet's `metrics-orthogonal` falsification test is *"Compare
metrics-disabled and metrics-enabled command results after removing only the metrics member"* — i.e.
two independent executions. In practice the invariant holds structurally (the ledger is created
unconditionally in `list_branches` and metrics is computed post hoc from a snapshot), so I rate this
Should, not Must — but the test as written would not catch a regression that broke it.

### S3 — No asymptotic order table in any canonical document

CFR-168 AC 1 requires *"selected operationsの時間/メモリorderをdocsとscale fixturesで固定し"*, and the
task packet's `handoff.required_outputs` includes `complexity_table`.
`grep -rn "O(" docs/*.md docs/development/source-architecture.md` returns **no hits**. The only order
statement I found anywhere is a Rust doc comment (`codefire-core/src/lib.rs:2253-2255`:
`O(atoms + links + changed_atoms + adjacent_entries)`). `docs/09_index_trace_policy.md:1-6` describes
the index qualitatively. ADR-020's "Required Proof Before Adoption" (lines 256-273) lists obligations
but no orders.

The *scale fixture* half of AC 1 is satisfied for Trace:
`codefire-core/src/lib.rs:4612-4661` `trace_work_counters_have_linear_1x_3x_10x_slope` asserts exact
`TraceWorkCounters` at scales 1/3/10 against a production-path evaluator. That is good, real evidence.
The docs half is absent.

### S4 — Rust MR create/review lack the pre-effect response preflight that upload/apply have (reachability unproven)

`*context.prepared = Some(...)` occurs only in `handle_http_upload_with_request_source`
(`http.rs:7068`, `:7078`, `:7123`) and `handle_http_request_apply_with_request_source`
(`http.rs:10924`). `handle_http_request_merge` (`http.rs:8506`) and `handle_http_request_review`
(`http.rs:9103`) leave `context.prepared` empty, so their responses are encoded after the mutation
commits, via `write_http_json_with_ledger` (`http.rs:807-814`). A `ResponseBytes` /
`ResponseHeaderBytes` / concurrent `RetainedBytes` failure there would return a typed 413 for an
already-committed mutation.

**I could not demonstrate this is reachable at the current limits** (16 MiB `ResponseBytes`, 32 MiB
concurrent `RetainedBytes`): the MR create/review response payloads I traced are small relative to
those ceilings. I am reporting it as an unmet ADR-020 §5 contract item with an unproven exploit, not
as a confirmed partial-effect bug. The same structural gap exists in Python for the same routes and is
folded into M3.

### S5 — Decision-work metrics projection covers exactly one command

`grep -n "^pub(crate) fn .*_metrics" crates/codefire-cli/src/metrics.rs` yields `status_metrics`,
`branch_list_metrics`, `scan_metrics`, `verification_metrics`. Only `branch_list_metrics` takes a
`&DecisionWorkSnapshot`. CFR-168's Required Fix asks for *"stable work countersとbudget-exceeded typed
outcomeをmetrics/diagnosticsへ出す"*. `compatibility.rs:132-161` discloses this honestly
(`"metrics_commands": ["branch list"]`, `"coverage": {"status": "partial"}`, and explicit
`not_fully_claimed` / `not_claimed` entries), and `compatibility.rs:185` tests the non-overclaim. The
disclosure is correct and is the right behaviour for an open root; it is simultaneously proof that the
criterion is not met.

One stale claim worth correcting in the manifest before Close: `compatibility.rs` still reports
`"python_http_transport": "not_fully_claimed"`, but the Python HTTP transport, bundle GET, connection
admission, and upload response preflight were subsequently landed and independently accepted per
`cfr-168.md:556-563`, `:603-613`, `:614-622`, `:656-663`. The manifest now understates Python
transport while overstating nothing — harmless today, but it will misdescribe the shipped surface if
it is not refreshed at Close.

---

## 3. Missing current-snapshot Install / Dogfood / Retrospect evidence

None of the three gates has any evidence at `e2a218e`. This is not an inference from absence — the
project's own tracking says so:

- `docs/development/current-cycle-burn-down-plan.md:63` — row 6 for `CFR-168 + CFB-136` reads:
  *"Implement in progress; Store/session, HTTP apply, and existing-project upload preflight accepted;
  ledger-bound atomic bootstrap, remaining surfaces, capability/metrics, **Install/Dogfood remain**"*.
  Compare rows 1-5 (`:58-62`), each of which records *"fresh Install, … Dogfood, independent ACCEPT"*.
- `docs/development/code-review-issues/cfr-168.md:8` — `Status | investigating`;
  `:50` — the Triage checklist item *"実装、scale gate、Install/Dogfood、Retrospectを完了した"* is unchecked.
- `docs/development/bug-issues/cfb-136.md:8` — `Status | investigating`; `:79` — *"root fix、full gate、
  fresh Install、Dogfood、Retrospectを完了した"* unchecked; `:53` — AC explicitly forbids `fixed`
  before those gates.
- `grep -n "CFR-168" docs/development/history.md` returns 61 rows, none of which is an Install,
  Dogfood, Retrospect, or Close entry — all 61 are Implement/Freeze/characterization slices.

Concretely missing for this snapshot:

1. **Fresh isolated Install** from `e2a218e` with recorded source/installed SHA-256 (the pattern used
   at `current-cycle-burn-down-plan.md:33` for CFR-167).
2. **Installed scale/budget reversal** — the packet's `fresh-install-dogfood` validation requires
   *"installed scale/budget reversals"*: at minimum an installed limit-1 / exact / limit+1 triple
   producing exit 60 with the full typed diagnostic, plus an installed x10 counter-curve check
   against AC 4.
3. **Canonical Dogfood** on `subprojects/algorithm-evolution-agent-lab` — Algorithm 281 tests plus
   installed `status`/`scan`/`verify`/`doctor`.
4. **Full cross-runtime counter corpus** — `cfr-168.md:239-240` still lists the Rust/Python counter
   corpus as outstanding, and ADR-020:268 requires *"Rust/Python common-ancestor, required-link,
   sealed/signature, object, and HTTP golden vectors agree"*. No such corpus exists at this snapshot;
   the Python side of the surfaces in M1/M3/M5 has no counters to compare against.
5. **Gates for the surfaces in M4** — `cf merge`, `cf storage`, `cf scan`, `cf remote upload` (file),
   and `cf remote doctor cf://…` currently have no budget to gate.
6. **Retrospect** — no new-issue-or-none record for CFR-168 or CFB-136.
7. **Capability re-advertisement at full coverage** — `cfr-168.md:126-127` committed to not
   re-advertising until every surface/schema/conformance item is present. The manifest is currently
   advertised with `coverage.status: "partial"`, which is the honest interim state, but Close requires
   flipping it and refreshing the stale `python_http_transport` value (S5).

---

## 4. Later-root work that must not be pulled into CFR-168

Recording this explicitly so the Close gate does not grow:

| Out-of-scope item observed | Owning root |
|---|---|
| `http.rs` 18,639 lines, `main.rs` 12,647, `codefire-store/src/lib.rs` 20,539, `tests.rs` 13,069; and the 30-file / 17,433-line Python owner-test sprawl **as an architecture problem** | **CFR-169** (`cfr-169.md:13-17`, `:22-23` — explicitly *"line countそのものではなく"*, and `:29-31` requires splitting test helpers/fixtures). M6 above is scoped to *production reachability*, not module size. |
| Typed executable automation outcomes / next-action vocabulary / migration | CFR-170 |
| Machine-readable consistency-claim and certificate-proof descriptors | CFR-171 |
| Read-only adoption and workspace-UX bounded experiments | CFR-172 |
| Progress / Fire-value / product stop-rule pilot protocol | CFR-173 |
| A resumable / paged bundle protocol with its own versioned snapshot + object-set token | Explicitly deferred by ADR-020:250-253 — *"A future resumable bundle protocol requires its own versioned snapshot/object-set token and is not left as an untracked CFR-168 residual."* Do not accept a streaming bundle as CFR-168 scope. |
| Python-only `POST …/gc` route (`codefire:14613`) with no Rust server counterpart (`grep '"gc"' crates/codefire-cli/src/http.rs` → no hits) and no Rust HTTP client | Pre-existing transport asymmetry, unrelated to decision-work. Should be triaged separately, **not** folded in. |
| `CFB-137` (wall-clock `Date` header test flake) | Already `fixed` (`cfb-137.md:8`); no CFR-168 action. |
| `CFR-174` / `CFR-175` | Already `fixed`; do not reopen. |

---

## 5. What is genuinely done, and done well

A reviewer that only lists defects misrepresents the state. The following are real, verified at this
snapshot, and should not be re-litigated:

- **Core `DecisionWorkLedger`** (`codefire-core/src/work_budget.rs`, 2,430 lines) — non-clonable,
  atomic multi-dimension `DecisionWorkReservation`, ledger-bound `DecisionWorkPermit` with
  `consume_reserved_pair`, opaque `DecisionWorkLedgerIdentity`, one-way terminal reasons, stable
  counter vocabulary, canonical-JSON goldens. 89/89 tests, clippy-clean.
- **Profile parity is exact.** `local_v1` / `http_v1` limits match byte-for-byte between
  `work_budget.rs:224-335` and `codefire:203-256`: graph_nodes 100k, graph_edges 400k, logical_depth
  100k, aggregate/retained/temporary 1 GiB local, object_records 100k, snapshot_revalidation 100k/1 GiB,
  maximum_frontier 100k, counter_values `U64_MAX-1`; HTTP overrides 64 KiB headers, 16 MiB
  request/response body, 32 MiB retained, 16 MiB temporary, 30 s deadline, 32 active / 0 queued.
- **Iterative traversal, no process stack dependence** — Store `iterative_sealed_validation_handles_deep_linear_history`
  passes; Python sealed/signature/ancestry traversals are deque/LIFO based.
- **Indexed Trace on the production path with a real slope fixture** —
  `TraceEvaluationIndex::compile` (`lib.rs:2264`) is reached by `cf scan`, and
  `trace_work_counters_have_linear_1x_3x_10x_slope` (`lib.rs:4612`) pins exact counters at 1x/3x/10x.
- **Rust local `branch list` closes CFB-136 properly** — one session, one ledger, provisional
  validation of all heads, one `finalize()` (`main.rs:11467-11560`;
  `codefire-store/src/lib.rs:6737-6807` performs the full retained-document revalidation and maps
  failure to `SubjectChanged` / `CorruptSubject` / `DeadlineExceeded` terminals).
- **Rust HTTP request lane is genuinely request-wide** — `handle_http_stream` (`http.rs:703-762`)
  creates one ledger, `dispatch_http_with_ledger` threads it through routing, intent, and route
  handlers, and `write_http_dispatch_outcome` (`http.rs:764-815`) releases and terminates it. Upload
  and apply run through `PreparedHttpMutationTransaction` (`http.rs:2912`, production entries at
  `:6782` and `:10711`) with pre-effect response preparation.
- **Rust tests do exercise the production wire** — 13 call sites drive
  `handle_http_stream` directly (e.g. `http.rs:13130` `full_wire_apply_preserves_authority_at_parser_store_handoff_boundary`,
  `:13189`, `:13271`, `:13536`), so the Rust HTTP findings cannot be dismissed as scaffolding-only,
  and neither can the Rust successes.
- **Honest self-disclosure** — the capability manifest declares `partial` coverage and enumerates
  `not_claimed` surfaces, with a test that guards against overclaiming (`compatibility.rs:185`).
  Combined with the Freeze notes in `cfr-168.md`, the project's own account of its state is accurate.
  That materially raised my confidence in the parts I did not exhaustively re-derive.

---

## 6. Decision

**`REQUEST_CHANGES`.**

Not `INSUFFICIENT_EVIDENCE`: the evidence available was sufficient to reach a firm conclusion. The
Rust unit gates, fmt, clippy, and docs checks all pass in this worktree and match the recorded
baseline; static tracing plus two executed runtime probes settled the production-wiring questions
without ambiguity.

Not `ACCEPT_FOR_CLOSE_GATE`, for two independent reasons, either of which alone is disqualifying:

1. **Frozen acceptance criteria are unmet on reachable production paths.** The characterization
   defect that opened the root — deciding a single ancestor question by materialising the whole
   reachable history — is still live and uncharged in the Python HTTP upload handler (M1, reproduced:
   300 reads), in `cf merge` (M4), and in file-transport `cf remote upload` (M4). A wired Python HTTP
   route runs on a second ledger with a 32×-larger ceiling than the request profile permits (M2,
   reproduced). Seven of nine Python HTTP routes have no request accounting at all, and three of them
   mutate durable authority with no pre-effect response ownership (M3). `storage`, named verbatim in
   the Required Fix, has no budget (M4).

2. **The green test numbers do not evidence the contract.** ~11,700 lines of Python owner code have
   no production caller, and ≥72% of the Python owner-test corpus exercises that code (M6). The
   decisive demonstration: `tests.test_python_ancestry_owner` passes 13/13 while the production
   ancestry path it names is the defect in M1.

The implementation quality inside the accepted slices is high and the project's own status tracking is
accurate — `cfr-168.md` says `investigating`, `cfb-136.md` says `investigating`, and
`current-cycle-burn-down-plan.md:63` says Install/Dogfood remain. **This review's finding is that
those statuses are correct and should not be advanced.** The gap to Close is integration and surface
coverage, not a defective nucleus.

### Ordered path to a re-review

1. Wire the existing accounted Python owners into `handle_http_upload` so that ancestry (M1),
   signature policy (M5), and the object-set digest run on the request ledger and deadline. Add a
   production fixture asserting the read count for "immediate parent is an ancestor" is bounded, so
   the M1 probe becomes a regression test.
2. Pass `self.decision_work` and `deadline` into the remaining seven Python HTTP routes; give
   `handle_http_list` one shared session, and extend `_HTTPMutationResponseSlot` to MR
   create/review/apply (M2, M3).
3. Wire `cf merge`, `cf storage`, `cf scan`, file-transport `cf remote upload`, and file-transport
   `cf remote doctor` to command ledgers; delete or unify the duplicate unaccounted
   `diagnose_remote_project` (M4).
4. Delete, or wire, every one of the eleven zero-caller Python factories. Whatever remains unwired at
   Close must be justified in `cfr-168.md` as a named later-root item, not carried as green test
   weight (M6).
5. Add the `invalidated` counter, fix `entry_count`, and project decision work beyond `branch list`
   (S1, S5); publish the complexity/order table (S3); either close S4 or record it as an accepted,
   documented limitation with the reachability analysis.
6. Only then run the Close gate: full Rust + Python + `demo.sh` + docs, fresh isolated Install with
   recorded SHA-256, installed budget reversals, x10 counter curve, cross-runtime counter corpus,
   canonical Algorithm Dogfood, and Retrospect.

### Risks if this snapshot were closed as-is

- Both original resource-safety defects (unbounded ancestry materialisation, unbudgeted signature
  history) would ship unfixed on the Python fallback and on three Rust local/file surfaces, with the
  issue marked `fixed` — the exact outcome CFB-136's AC 5 was written to prevent.
- A Python HTTP `GET /doctor` against a large project can retain up to the local 1 GiB ceiling inside
  a request the operator believes is capped at 32 MiB.
- `cfr-168.md`'s 1,068-line evidence log would become the project's record of a closed root while
  ~35% of the Python code it describes has no caller, making future reasoning about what is actually
  enforced unreliable.
