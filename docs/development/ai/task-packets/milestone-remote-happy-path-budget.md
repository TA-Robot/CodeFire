# AI Task Packet

Status: completed

Packet ID: MS-remote-happy-path-budget

Schema version: human-v1

Risk score: 5

Reasoning tier: high

## Objective

Classify the current implemented residuals of CFR-168 and CFB-136 against only the active M1 file-backed `upload -> request-merge -> request-apply` happy path, so the orchestrator admits no scale/HTTP/cache work that the path does not require.

This is node 2. It is read-only and runs in parallel with the public fixture/command probe.

## Scope

Read CFR-168, CFB-136, ADR-020, current source/architecture/command traces, existing focused tests, and historical implementation evidence only as needed to answer:

1. Which file-backed upload/MR/apply owners and decision-work/session paths are already implemented at current HEAD?
2. Does the smallest happy path traverse any unfinished CFR-168 counter/budget/closure surface?
3. Can CFB-136 same-inode mutation occur within one isolated happy-path command, and is its final revalidation already wired on that path?
4. What exact focused fixture/test can distinguish “path already works” from the smallest blocker?
5. Which unfinished HTTP, Python parity, scale, Doctor/storage, or broad graph work must remain parked?

## Non-Goals

- implementing, refactoring, running remote mutation, or widening tests;
- completing CFR-168 or changing issue status;
- treating “unfinished issue” as proof that the M1 happy path is blocked;
- reading or changing canonical dogfood state;
- starting adoption/release/product/ecosystem work.

## Validation

Set Status `completed` and classify each candidate slice `block-m1`, `park`, or `drop`. Run:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

No build, test, install, upload/request command, or product write is allowed.

## Handoff

- exact production ownership/path map: file-backed `cf://` only. Parse in `main.rs`; plan in `remote/plans.rs`; apply in `remote::{upload_branch,request_merge,apply_merge_request}`; layout/gate in `remote/{project,locks}.rs`; graph copy in `remote::{copy_object_graph,collect_object_records}`; durable writes in `remote/operations.rs` + `publication::{execute,replay}_json_publication`. HTTP twins in `http.rs` are out of this path.
- current implemented-vs-residual boundary: Store `validate_sealed_commit` already uses a per-call cache plus final retained-document revalidation. File-backed upload/MR/apply do not create a `DecisionWorkLedger`, do not share a command-wide session, and still call the unaccounted closure/digest/ancestry/publication twins. Accounted twins and `PreparedJsonPublication` are HTTP/test-wired, not this CLI path.
- whether either lies on the M1 critical path: no. CFR-168 `park`. CFB-136 `park`. Neither is `block-m1` or `drop`.
- smallest exact write set and focused fixture only if unavoidable: none. Distinguisher already exists: `crates/codefire-cli/src/tests.rs::file_remote_upload_clone_show_diff_and_merge_request_flow`. Do not add or run tests from this packet.
- conflict/dependency relationship with node 1: disjoint write sets. Implemented apply requires approved+subject-bound review; that is a public-sequence fact for node 1, not a CFR-168/CFB-136 residual. Fan-in must keep `cf://` and refuse HTTP, or this classification does not apply.
- next legal action after fan-in: node 3 admits a fixture/command sequence from node 1 without CFR-168/CFB-136 product writes unless a later isolated probe reproduces a failure on that exact path.

Commit only this packet.

## Result

Status **completed**. Classification at exact HEAD / base `aedce77ecc64c3c478d19a366e51b5425e7f6605`: **CFR-168 `park`**, **CFB-136 `park`**. Neither is `block-m1`. Neither is `drop`. No product write is unavoidable for this M1 slice.

Run by: Grok 4.6 individual agent, 2026-08-21, worktree `ms-remote-happy-path-budget-20260821`. Read-only implemented-vs-residual classification of the isolated file-backed `upload -> request-merge -> request-apply` path. No remote mutation, build, test, install, or implementation.

Active milestone: `MS-remote-happy-path`.
M1 exit criteria: one already sealed commit completes documented file-backed `upload -> request-merge -> request-apply` against an isolated remote; JSON identifies source/target/MR/applied head; a read-only replay confirms it; only the smallest *reproduced* CFR-168/CFB-136 blocker may be fixed.
Critical path: public remote-fixture/happy-path probe -> smallest blocker admission -> isolated remote run/fix -> read-only confirmation -> independent audit -> M1 exit.
This packet is parallel to the path (node 2), not the public-fixture probe.
This change is required because the orchestrator must not admit scale/HTTP/cache work the exact path does not require.
Parked instead of done: CFR-168 remaining ledger/HTTP/Python/scale/Doctor/storage/capability work; CFB-136 command-wide cache expansion; request-review as a CFR-168/CFB-136 admission.
Independent packets still unstarted: node 1 public fixture/command probe (parallel); node 3 fan-in waits on 1 and 2.
Validation actually run: `python3 docs/development/tools/check_project_docs.py`; `git diff --check`. No cargo/python tests, no `codefire upload|request-*`, no product write.
Stall risk: treating unfinished CFR-168/CFB-136 as M1 blockers, or admitting HTTP because accounted owners already exist there.
Next allowed action: wait for node 1 and orchestrator fan-in; do not implement.

### 1. File-backed owners already implemented at this HEAD

Default CLI is Rust. File vs HTTP is chosen by URL (`parse_cf_url` / `is_cf_http_url`) inside the same command functions; this classification is only the `cf://` arm.

| Command | Parse | Plan | File-backed apply | Persistent effect |
|---|---|---|---|---|
| `upload` | `main::parse_upload_args` | `remote/plans.rs::upload_operation_plan` | `remote::upload_branch` | `ensure_remote_layout` / `RemoteProjectGate::acquire_exclusive`; `copy_object_graph`; `operations::upload_publication` + `upload_writes`; `publication::{replay,execute}_json_publication` |
| `request-merge` | `main::parse_request_merge_args` | `request_merge_operation_plan` | `remote::request_merge` | `RemoteProjectGates::acquire_for_merge`; source then target `validate_sealed_commit`; `collect_object_records` + `source_object_set_digest`; immutable MR subject; escrow `copy_object_graph`; `request_merge_publication` + `execute_json_publication` |
| `request-apply` | `main::parse_request_apply_args` | `request_apply_operation_plan` | `remote::apply_merge_request` | exclusive target gate; approved+subject-bound check; source/target `validate_sealed_commit`; `is_ancestor_in_objects`; `apply_publication` + `execute_json_publication` |

Supporting owners already on this path:

- `remote/project.rs`: `cf://` decode, stable project ID, contained authority paths, missing-project `ensure_remote_project_layout` (no request ledger).
- `remote/locks.rs`: exclusive/shared project gate and ordered source/target acquisition.
- `remote/operations.rs`: canonical upload/MR/apply `PublicationRequest` + authority write sets.
- `remote/publication.rs`: keyed v2 receipt replay and durable reducer used by file-backed CLI (`execute_json_publication` / `replay_json_publication`).
- `signatures.rs::enforce_remote_commit_signature_policy`: iterative history walk; no-ops unless `server_policy.json` sets `commit_signatures.required`. Isolated default fixtures do not enter that policy.
- JSON envelope render: `main::print_plan_result_json` wrapping the operation plan (`transport: "file"`).

`request-review` (`remote::review_merge_request`) is implemented and is a production prerequisite of apply (`status == "approved"` and `merge_request_has_effective_approval`). That is not a CFR-168/CFB-136 residual. Node 1 owns whether the public sequence names it.

Accounted / session owners exist at this HEAD and are **not** the file-backed CLI mutators:

- `DecisionWorkLedger` and `begin_sealed_commit_validation_session(_with_ledger)` — local `branch list`, HTTP bundle/MR/upload, Doctor, Store tests.
- `collect_object_records_with_owned_work_and_poll`, `source_object_set_digest_ordered_with_poll`, `is_ancestor_in_provisional_subject_with_poll`, `enforce_remote_commit_signature_policy_in_provisional_subject_with_poll`.
- `PreparedJsonPublication` / `prepare_json_publication_plan_lazy_with_nonce_and_provisional_roots` — production call in `http.rs` only.
- `prepare_unpublished_remote_project_with_ledger` — marked HTTP bootstrap integration, not the file-backed `ensure_remote_project_layout` used by `upload_branch`.

### 2. Smallest happy path vs unfinished CFR-168 counters/budgets/closure

The file-backed mutators traverse the **unaccounted twins** of CFR-168 surfaces:

- closure: `collect_object_records(..., ledger=None)` and `copy_object_graph` (no ledger, LIFO reachable walk, per-record validate+store);
- digest: `source_object_set_digest` (legacy no-ledger);
- ancestry: `is_ancestor_in_objects` → `ancestor_distances` full FIFO map, not the provisional-subject/ledger early-stop helper;
- publication: `execute_json_publication` / `replay_json_publication`, which re-call `validate_sealed_commit` per `object_roots` without a request-wide ledger or `DecisionWorkPermit`;
- bootstrap: missing `cf://` project creation through `ensure_remote_project_layout`, not the HTTP ledger footprint owner.

Those twins are implemented production code, not missing handlers. CFR-168 residuals are attaching one request ledger, typed counters, HTTP combined publication, Python parity, scale/capability, and Doctor/storage session reuse. A one-commit isolated fixture never consults those unfinished limits, so unfinished counters cannot be the smallest M1 blocker on current evidence.

`unfinished issue` is not proof the happy path is blocked. Historical in-process coverage of the same file-backed owners already exists (`file_remote_upload_clone_show_diff_and_merge_request_flow`). This packet did not rerun it.

CFR-168 candidate slices:

| Slice | Decision | Why |
|---|---|---|
| File-backed CLI ledger/session/permit wiring for upload/MR/apply | `park` | Path already has unaccounted owners; wiring is CFR-168 completion, not M1 exit |
| Accounted closure/digest/ancestry on this CLI path | `park` | Same; small isolated graphs do not require budget stop |
| `PreparedJsonPublication` on file-backed reducer | `park` | Legacy reducer is the file-backed production path; HTTP combined owner is a different transport |
| HTTP request-wide owner, nonce, 413/deadline, bootstrap ledger | `park` | M1 names isolated file-backed `cf://` only |
| Python decision-work parity | `park` | Default CLI is Rust; Python is fallback |
| 1x/3x/10x scale, capability re-advertisement, Install/Dogfood/Retrospect | `park` | Explicit M1 non-goals |
| Doctor/storage/local branch-list remaining charges | `park` | Not on upload/MR/apply |

### 3. CFB-136 same-inode mutation inside one happy-path command

`codefire_store::validate_sealed_commit` allocates a **fresh** `SealedCommitValidationCache`, binds the store, validates one commit, then `finalize_sealed_validation_subject` → `revalidate_retained_object_documents`. Final revalidation of that per-call cache **is already wired** on every file-backed validation, including publication `validate_object_roots`.

CFB-136's reproduced shape is command-wide cache reuse across heads: validate head A, rewrite a shared ancestor in place (same inode/path/size/mtime, extension-only), then validate head B from the same cache and get a validated hit. That does **not** occur on this path:

- `upload_branch`: one local-head `validate_sealed_commit`, cache dropped; later `collect_object_records` / `copy_object_graph` re-read without that cache.
- `request_merge`: source then target `validate_sealed_commit` with **two independent** caches even when both branches share one project object store.
- `apply_merge_request`: two more independent per-call caches on the target-local escrow store, then uncached `is_ancestor_in_objects`.

Same-inode rewrite **during** one `validate_sealed_commit` is the Store-nucleus case already covered by final revalidation (`sealed_validation_session_detects_extension_only_same_metadata_rewrite` is the session fixture; the per-call API uses the same finalize). Post-validation copy/collection does not reuse that cache.

Command-wide sessions used today (`branch list`, HTTP MR list, Doctor) are off this mutator path. Expanding those sessions onto upload/MR/apply without the existing session+final-revalidation contract would *create* CFB-136. Current owners avoid it by not sharing the cache.

CFB-136: `park`. Not `block-m1` (path does not share the failing cache). Not `drop` (same-root CFR-168 residual remains valid for session consumers).

### 4. Focused fixture that distinguishes “path already works” from the smallest blocker

Do not add tests. Do not run them from this packet. Existing focused coverage of the implemented owners:

- `crates/codefire-cli/src/tests.rs::file_remote_upload_clone_show_diff_and_merge_request_flow` — isolated temp `cf://` server, `upload` main then feature, `request-merge`, `request-review --decision approve`, `request-apply`, plus list/clone/show/diff. This is the in-process distinguisher for “owners already complete the file-backed flow”.
- `tests/test_codefire_cli.py::RemoteFlowTests.test_upload_list_remote_clone_and_request_merge` — subprocess CLI of the same sequence; apply uses the reviewed head after a later source upload.

Smallest later isolated probe (node 3+, not this packet): temp root disjoint from canonical Algorithm; one sealed local head; `cf://$server/org/app/{main,feature}`; `upload --json` → `request-merge --json` → `request-review --decision approve --json` → `request-apply --json` → read-only `list` / `show` / `request-list`. Interpret:

- completes with applied remote head matching the reviewed source head → CFR-168/CFB-136 remain `park`;
- one command reports validated success after an in-command same-inode rewrite of a memoized record → CFB-136 `block-m1`;
- command fails for missing ledger, HTTP, scale, Doctor, or Python parity → that is new reproduction, not current evidence, and is not admitted from this packet.

### 5. Fan-in

Node 1 (public fixture/command probe) writes only `milestone-remote-happy-path-probe.md`. This packet writes only itself. No shared file.

Dependencies for node 3:

- keep transport `cf://`; HTTP would pull parked CFR-168 publication/ledger residuals this classification excluded;
- include `request-review` in the admitted sequence if apply is the terminal mutator; omitting it is a public-contract miss for node 1, not a CFR-168/CFB-136 blocker;
- do not admit CFR-168 or CFB-136 product edits, new focused tests, or scale/HTTP/Python/Doctor work on this evidence;
- next legal product action after fan-in is the isolated run of the admitted public sequence, not a budget/cache implementation.

Issue ledgers stay `investigating`. This packet does not change issue status, the board, or canonical dogfood state.
