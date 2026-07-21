# CodeFire Command Source Trace

Status: canonical

Owner: CLI Maintainer

Update trigger: `COMMAND_CAPABILITIES`, dispatch, parser, plan/apply boundary, JSON rendering, persistent writes, or command tests change.

This is a symbol-level routing map. It intentionally omits fixed line numbers. The source-doc checker requires one marker for every registered top-level command and detects registry drift.

## Execution Contract

Every command follows as much of this path as its mode needs:

```text
argv
  -> help/capability routing before mutation
  -> typed parse
  -> repository/open/remote context resolution
  -> validation and lock/idempotency checks
  -> operation plan
  -> apply (skipped for read-only or dry-run)
  -> domain/result value
  -> text renderer or codefire.command_result.v1 envelope
```

`main.rs::run` is the top-level router. `local_workflow_handler` handles the compact local command family; the remaining arms call focused modules or still-local orchestration functions. `command_registry::COMMAND_CAPABILITIES` is the declared top-level surface, not a complete parser registry.

## Registered Commands

| Command | Capability | Parse / execute / render ownership | Persistent effect and principal evidence |
|---|---|---|---|
| `init` | text | `main` init parser/initializer, store/core initial roots | creates `.codefire`, initial sealed commit and main branch; init/layout/store tests. <!-- command: init --> |
| `import` | JSON | `import_workflow::{parse_import_args, import_existing_project, import_result_data_json}` | kernel-locks a persistent bootstrap provenance inode and content-immutable/exactly-one-link repository lock, publishes the common owner sidecar, resumes a payload-bound durable transaction, initializes/opens/scans the pinned target through bounded no-follow reads, and persists a v2 hash-only idempotency result; exact/conflicting/concurrent/crash-phase/reserved-temp/crafted-marker/I/O-provenance tests. <!-- command: import --> |
| `open` | JSON | `main::{parse_open_args, open_branch_from}`, `secure_worktree` | validates the sealed closed head, resolves the workspace boundary, materializes marker/content into a deterministic capability-pinned sibling stage with exact owner provenance, and publishes the complete worktree with one rename before opened/active/branch state writes; partial-cleanup/retry, abandoned-stage, path-race, idempotency, and corruption tests. <!-- command: open --> |
| `branch` | JSON, metrics for list | `main` branch list/show routing, `branch_refs::{RepositoryMetadataSnapshot,BranchRefSnapshot,OpenedRegistrySnapshot}`, Store validation session, metrics wire | list pins `.codefire`, captures branch/opened membership and documents relative/no-follow, validates all sealed heads on one ledger, revalidates the compound subject, precharges immutable projections and exact JSON/text bytes, then performs one final write. Subject-change, exact/+1, many-head shared-DAG, pretty/text, and cleanup tests cover it; there is no branch-create subcommand. <!-- command: branch --> |
| `status` | JSON + metrics | `main::read_status`, `open_snapshot`, `automation::status_*`, `metrics::status_metrics` | read-only derived state and scan prediction only after repository/marker/branch/opened/active authority agrees; divergence and pending recovery use typed no-write diagnostics. Bidirectional base-mismatch/pre-write/JSON/next-action tests cover it. <!-- command: status --> |
| `scan` | JSON + metrics | `main::{parse_path_json_args, run_scan_command, run_scan_with_lock_options, compute_scan_with_cache}`, `open_snapshot`, core scan builders, automation/metrics | live routing acquires the repository lock and an exact verified/recovery context, validates the sealed base and Trace endpoints, then uses subject-bound state-transaction v2 to write active scan/fire and registry/active/branch state while invalidating superseded verification. Exact phase retry recomputes; dry-run remains read-only; authority/corruption/lock/all-phase/freshness/non-Atom/bounded JSON tests cover it. <!-- command: scan --> |
| `fire` | JSON | `fire` plus `fire::active_ledger_transaction`, `main::run_fire_command` | validates every manual/batch Fire, stages the bounded `fires.json` postimage before publication, supports exact pending/completed replay, then uses state-transaction v2 for registry/active/branch state. Different pending requests/divergence fail closed and dry-run writes nothing; all-gap/replay/batch/identity tests cover it. <!-- command: fire --> |
| `extinguish` | JSON | `main::{parse_extinguish_args, run_extinguish}`, `batch`, `extinguish_ux`, `fire::active_ledger_transaction` | manual/batch paths update Fire status/`resolution_uid` and active Resolution records through one staged two-file reducer under lock, replay the durable original result, then publish target state with state-transaction v2; manual idempotency finishes afterward when requested. Refresh/all-gap/collision/divergence/no-follow/bounds/dry-run tests cover it. <!-- command: extinguish --> |
| `verify` | JSON + metrics | `verification::{parse_verify_args, render_verification}`, `main::{run_verify_command, run_verify_with_lock_options, compute_verify_with_cache}`, `assessment_subject`, `command_execution`, `secret_redaction`, `open_snapshot`, `merge_resolution::unresolved_merge_conflicts_from_state`, core effective-policy/change-coverage/decision, automation/metrics | policy/cwd/merge preflight precedes mutation; live routing binds a versioned scan/worktree assessment subject, effective rule provenance, and file-relation coverage, refuses a stale pre-execution worktree, uses a minimal or explicit inherited-snapshot environment, streams bounded output, records input/environment/worktree/resource profile, and blocks subject-changing commands before publishing verification/state through state-transaction v2. Commit rechecks that worktree subject through branch publication. JSON/text/diagnostic/explain/certificate share the subjects; authority/security/coverage/all-phase tests cover it. <!-- command: verify --> |
| `commit` | JSON | `main::{parse_commit_args, run_commit, commit_operation_plan, build_manifest_plan, plan_commit_transaction_objects, store_planned_commit_object}`, `open_snapshot`, `commit_active_reset`, core decision, store/signatures/automation | verified locked context and typed committable gate reject divergence or pending active-ledger authority before immutable/head publication. Success predeclares all object roles and exact reset byte snapshots, validates/stores the candidate and branch snapshot, publishes branch/opened state, then rolls the reset through every removal/state progress boundary before idempotency. Bidirectional authority, reset-gap/replay/divergence/legacy, safety, merge, and store-gap tests cover it. <!-- command: commit --> |
| `clone` | JSON | `main::{parse_clone_args, clone_branch}`, `remote::{project,locks,copy_object_graph,write_object_records}`, open/clone idempotency | snapshots a remote branch and complete sealed graph under the shared project gate, preserves every authenticated current/legacy object record ID during transfer, then creates a new local closed branch; it does not create an opened directory/state. Pending-reader/dry-run/idempotency/legacy-ID tests cover it. <!-- command: clone --> |
| `close` | JSON | `main::{parse_close_args, close_branch}`, `close_transaction`, `close_worktree`, `active_state` | validates the exact open subject, rejects pending active/open publication, publishes durable close authority, then advances worktree/active/registry/branch through identity-bound deterministic quarantine and object-first branch publication; exact retry heals every apply-before-phase boundary while dry-run is read-only. Phase/failpoint, sequential archive, doctor, marker/target/parent-swap, keep-dir, and dirty-work tests cover it. <!-- command: close --> |
| `discard` | JSON | same durable close path with explicit discard flag | intentionally removes unsealed open work through the same request-bound transaction; a foreign replacement or conflicting transaction is preserved and rejected, and only an exact retry may resume. <!-- command: discard --> |
| `merge` | JSON | `main::{parse_merge_args, merge_branch}`, `merge_resolution::{parse_merge_continue_args, run_merge_continue}`, `view` merge/diff, merge idempotency | merge apply plans/writes target changes and records text/binary/semantic conflict authority. `--continue` validates marker/sidecar/semantic resolution under the lock and pinned worktree before clearing blockers; dry-run/blocked paths are non-mutating. Text/binary/semantic/JSON/two-parent/malformed-path/idempotent-continue tests cover it. <!-- command: merge --> |
| `upload` | JSON | `main::parse_upload_args`, `remote::{project,locks,publication,operations,copy_object_graph,write_object_records}`, `http`, `signatures`, store exact-record publication | validates components/authenticated payload, applies one request ledger through overlay/graph/digest/signature/ancestry work, acquires the exclusive project gate, transfers exact records, then publishes generation and branch through one staged `remote_publication_transaction`; keyed v2 receipts replay across file/HTTP and Rust/Python. The Python fallback HTTP upload additionally validates publication actor/idempotency syntax before layout or temporary staging and validates sealed roots against the staged closure. Its closure planner owns the incoming lookup, seen set, and LIFO frontier; its stage-input projection owns both output lists. All collection bases, nodes/entries, capacity growth/overlap, and sort scratch are reserved on the request ledger before allocation, legacy collections are skipped on the accounted path, the stable closure phase is deadline-polled, and payload is dropped before release. Accounted staging strongly binds the exact records/closure objects, reserves Retained subject ownership before freezing tuples, and freezes incoming index/object ID/compact size/digest/private bytes plus existing ID/private bytes/device/inode/link-count/size/content-hash evidence. Immediately before `mkdtemp`, it revalidates list contents, record identity/size/digest, and each existing source with a second bounded read; both reads and incoming canonical revalidation are accounted. The Stage exposes no plan getter: one single-use method iterates and writes its private read-only payload views, clears write locals, and detaches payload-bearing failure tracebacks before close releases Retained/Temporary ownership. Thus caller mutation, CPython `id()` reuse, pathname replacement, successful final-iteration locals, and fallible write frames cannot redirect or outlive the accounted subject; any later failure remains confined to the cleaned temporary tree. For a missing project it freezes and validates a layout-independent generation/branch/optional nonce write proof before server namespace creation; after bootstrap reveals the random stable project ID, the full semantic proof revalidates and consumes that exact write plan under the gates. The response adapter freezes the exact success and fixed retry response pair before effects. Existing-project recovery/fast-forward planning and receipt/pending request-hash precedence remain unchanged. A single-use canonical publication snapshot binds semantic/result and every authority preimage/postimage to the reducer invocation; a fresh signed nonce-cache postimage is its final authority. Sealed/signature/verification/digest/ancestry and final publication allocation/deadline coverage, other Python POST routes, and full upload deadline polling remain outside these slices. Tamper/all-gap/replay/legacy/current-ID/component/gate/pre-effect fault tests cover it. <!-- command: upload --> |
| `list` | JSON | `main::{parse_list_args, list_branches/list_remote_branches}`, `remote::{project,locks}`, `http`, `pagination` | read-only local opened/branch or shared-gated remote branch view; a fixed pending publication blocks partial remote authority reads. HTTP branch listing scans and projects with deadline polling and validates all heads in one request-ledger Store session. Remote `--limit/--cursor` pages bind collection, ordered-result snapshot, offset, and limit; mutation makes replay explicitly stale. Bounded/sealed/page/JSON tests cover it. <!-- command: list --> |
| `request-merge` | JSON | `main::parse_request_merge_args`, `remote::{project,locks,publication,operations}`/`http`/`signatures` | acquires ordered source/target gates, reloads heads, copies a complete source graph into target escrow, binds the immutable versioned MR subject, and publishes it through the durable reducer. Dry-run/idempotency/signature/order/escrow tests cover it. <!-- command: request-merge --> |
| `request-list` | JSON | `main::parse_remote_project_args`, `remote::{project,locks}`, `pagination` | shared-gated target-local MR view with the same snapshot-bound cursor contract; fixed pending blocks partial publication and legacy cross-project records fail closed until ordered review materializes escrow. <!-- command: request-list --> |
| `request-review` | JSON | `main::parse_request_review_args`, `remote::{project,locks,publication,operations}`, remote review/signature path | parses routing fields without dereferencing source authority, acquires the target gate or ordered legacy gates, rereads/validates the MR, binds the decision to the immutable subject digest, and journals the MR postimage; stale rejection is read-only. <!-- command: request-review --> |
| `request-apply` | JSON | `main::parse_request_apply_args`, `remote::{project,locks,publication,operations}`, remote apply/signature path | validates the approved subject and target-local escrow under the exclusive gate, then journals generation, target branch, and MR with one identical reviewed head; it never substitutes the current source branch. <!-- command: request-apply --> |
| `show` | JSON | `main::parse_show_args`, `view::{show_commitish, show_commitish_data}`, `remote::{project,locks}` | remote branch/object graphs are snapshotted under the shared project gate before rendering; local/HTTP text/envelope/error/pending tests cover it. <!-- command: show --> |
| `diff` | JSON | `main::parse_diff_args`, `view::diff_commitish_with_options`, `remote::{project,locks}`, and algorithm modules | each remote operand is snapshotted under its shared project gate before bounded file/Atom/Trace/impact comparison; golden/Git/pending/large-file tests cover it. <!-- command: diff --> |
| `review-pack` | JSON metadata | `main::parse_review_pack_args`, `view::review_pack_with_options`, `assessment_subject::sealed_pair_subject` | validates both sealed commits and emits one immutable `sealed_validated` base/source subject before writing an explicit output artifact; bounded subject/metadata/content tests. <!-- command: review-pack --> |
| `patch` | JSON | `main` export/import parsers, `view` export, patch import/idempotency | export writes artifact; import validates then mutates open directory under lock/idempotency; round-trip/dry-run/bounds tests. <!-- command: patch --> |
| `doctor` | JSON | local `doctor::{parse_doctor_args, run_doctor, checks, branch_heads, opened_registry, report::AccountedDoctorReport}` or `remote::doctor` plus file/HTTP routing, `open_snapshot`, active-transaction/store/layout validators | target classification precedes local discovery. The request ledger owns layout, object-scan diagnostics, transaction-marker collection, branch authority, and opened-registry raw/parsed candidate ownership across Store validation. Object records reuse the pinned Store scan/cache. Branch heads and opened-registry candidates use sorted capability-pinned snapshots with fixed-size no-follow reads and final identity/content/membership fences. Opened JSON reserves serde Temporary plus parsed Retained before materialization, keeps exact `Value` ownership beside raw evidence, and restores snapshot+ledger on finish/abort cleanup failure for same-owner retry. Ordered issues and final report accounting remain stable. Open-summary evidence, active transactions, extracted opened fields, later opened/active-state diagnostics, and active-state JSON candidates remain CFR-168 residuals. File and HTTP(S) remotes return bounded typed reports and sealed-root validation. <!-- command: doctor --> |
| `storage` | JSON | `storage::{parse_storage_report_args, run_storage_report, *_json}` | read-only bounded local/remote inventory and artifact warnings; full local inventory uses the identity-rechecked capability-bound object scan, while quick mode avoids full record parsing; quick/full/limit tests. <!-- command: storage --> |
| `evidence` | JSON | `evidence::{parse_evidence_add_args, run_evidence_add/run_evidence_batch}`, `command_execution`, `secret_redaction` | validates command/artifact/batch, clears to a minimal environment, streams bounded output with process-tree timeout, redacts secret/path metadata, and stores profile/phase/portability plus subject/input/environment/resource evidence; large/infinite/descendant/failure/redaction tests cover it. <!-- command: evidence --> |
| `explain` | JSON | `explain::{parse_explain_args, run_explain}`, `assessment_subject` | read-only Fire/Atom/verify/storage explanation; Fire/verify views project the same live/verification subject used by context and certificate. Subject/target/error tests cover it. <!-- command: explain --> |
| `context` | JSON | `context::{parse_context_args, build_context_pack}`, `assessment_subject` | read-only bounded selection over active/repository objects plus versioned live subject; subject equality/truncation/freshness/path tests cover it. <!-- command: context --> |
| `migrate` | JSON | `migration` parse/inspect/locked apply, `repo_layout`, `RepoLock`, and retained no-follow layout capabilities | check/dry-run are read-only; apply performs a full locked reload and only registered auto-create directories, with replay/blocker/symlink/outside/parent-swap/concurrent-plan/profile-warning tests. Next actions are derived from mode and the current remaining postcondition, while planned/applied actions remain historical audit fields. Target formats are compatibility profiles, not format writers. <!-- command: migrate --> |
| `serve` | text process | `main::parse_serve_args`, `http::serve_http`, `http_tls`; Python `ReusableThreadingHTTPServer` | serves remote state and mutating routes under configured auth/signature/resource policy. Python process-global admission transactionally registers live servers, applies the minimum profile cap while grandfathering existing sockets, has zero application queue, precedes TLS/handler work, writes one fixed typed 503 only for plaintext HTTP, and releases a retryable permit on every handler/shutdown path; route/TLS/body/timeout/admission tests. <!-- command: serve --> |
| `completion` | text | `completion::{completion_script, bash_completion_script, zsh_completion_script}` | read-only generated shell script; registry/completion consistency tests. <!-- command: completion --> |
| `capabilities` | JSON | `command_registry::capabilities_data_json`, `compatibility::support_manifest_json`, automation envelope | generated read-only command targets, JSON/metrics/pagination modes, public schema IDs, format support decisions, semantic profile, and runtime roles; registry/help/parser/schema/conformance consistency tests. <!-- command: capabilities --> |
| `version` | text | `main` version arm | reports internal `codefire foundation 1`; release policy must not treat it as SemVer. <!-- command: version --> |

## Internal Debug Surface

`atom-index`, `trace-graph`, and `missing-links` are read-only debug commands handled by `parse_read_only_debug_args`. They return structured envelopes with `--json` but are intentionally not entries in `COMMAND_CAPABILITIES`; changing that classification requires a registry/CLI-contract decision.

HTTP bundle routes validate the sealed head under the shared project gate, then use the atomic
`DecisionWorkLedger` object collector before response construction. The response uses one bounded JSON writer;
16 MiB limit-1/exact succeeds and limit+1 is rewritten to a typed 413 before any `200` bytes. The diagnostic keeps
profile/dimension/observed/limit/phase/command/continuation fields through server, client, and automation rendering.
The Python fallback preserves the handler-ledger identity across branch authority, iterative sealed validation,
reachable record chunked read/hash and canonical traversal, final subject fence, ID-ordered ownership, and response freeze. Append, truncate,
and pathname replacement terminate as `subject_changed`; one fixed bounded control response emits the existing
canonical 400 without reserving on the terminal ledger. The same optional deadline polls those bounded operations
through response publication; Python upload also polls while freezing its response pair and at its final pre-effect
fence. Expiry emits a fixed 503 only after route ownership closes. Contiguous stdlib JSON
parse/materialization remains non-interruptible, so full Python bundle deadline support is still rejected. This
coverage does not imply other POST response preflight, request-header/body, full upload-work deadline polling,
other-route deadline coverage, or connection admission.

Python uploadの後段候補は4つのsingle-use ownerへ分離した。production接続済みの
`AccountedServerVerification`はproject/policy/cwd capabilityを最初のopen前から所有し、policyとambient projectを各spawn直前に
再検証し、childへpinned project/cwd FDを渡す。production uploadはhandlerの同一request ledger/deadlineをownerへ渡し、
no-ledger callerだけlocal ledgerを維持する。全reservationはcaller-owned permitを`reserve_into`でowner-first登録し、FD番号は
close前にdetachして一度だけ閉じるためsame-number ABAをretryしない。pure `AccountedCommitSignaturePolicy`はpolicy/history/HMACを、
そのproduction-unwired `AccountedRemoteSignaturePolicyAuthority`はabsentまたはexact single-link raw policyをno-follow project capabilityへ
固定し、same-ledger/deadlineでpure ownerへ一度だけ貸してfinal identity/content fenceを行う。malformed JSON/UTF-8はraw payloadを
exceptionへ残さず、authority closeはchildを先に閉じる。
production接続済み`AccountedHTTPUploadSealedValidation`はsealed Stageからincoming/existingのcount/aggregate/max-record scalar
boundsだけを一度moveし、staged session/FDをhandler ledger/deadlineへ閉じる。さらにproduction-unwired
`AccountedHTTPUploadAncestry`はvalidated commit loaderを借用するfull BFS decisionのcollection/frame/deadlineを閉じ、match後も
legacy late-error orderのため全走査する。signatureとancestryはまだupload/MR/publicationのproduction pathに未接続であり、sealed
session borrow、signature/digest/ancestry ordering、最終publicationの完全なallocation/deadline closureは残る。

Sealed ownerのproduction-unwired signature/ancestry handleは、retained validated commitだけをloaderなしで借用する。raw mutable
aliasはdestination-first canonical fingerprintでhandle close時とfinal fence時に検査する。disk callback完了後の最終checkはcallback
なしで行い、成功したdocumentを即session authorityからdetachしてからscalar cleanupへ進む。lookup/release/finishはsingle-flightで、
callback後かつpayload取得前にhandle/owner tokenを再確認する。

production-unwired digest handoffは、parser-owned full-request scalarをcaller-owned one-shot `AccountedHTTPUploadParserScalar`で取得し、
そのexact tokenを`AccountedHTTPUploadRequestSource`へmoveする。Stage closure/core/newline boundsは使わない。caller-owned二段階
`AccountedHTTPUploadSemantic`がsourceをrelease/reacquire gapなしでadoptし、
digestを公開せずexact semantic mapping、元record identity、別Retainedのeffect-ready core-plus-newline bytesを保持する。last deadline callback後に
full request/list/ID/duplicate/full-record digest/copy parityを再検証し、mutable AST scalarをpayload-first releaseしてsemantic/copyをfield assignment
だけでread-only mapping proxy/copy tupleのprivate effect slotへdetachする。同tokenのduplicate bindは拒否する。existingのcontinuous gate、missingのbootstrap-locked private-stage gate/proof、mapping lending、
effect consumer、handler parser release-slot移管は未接続である。

## Cross-Cutting Paths

### JSON

- `automation::command_result_envelope` owns the top-level shape.
- verify `data.decision` exposes the typed `outcome`, `primary_reason`, ordered
  `blocking_reasons`, `warning_reasons`, `effective_policy_digest`, and
  `change_coverage_digest`; legacy `data.result` is its string projection. Full contracts are
  siblings in verification data; each actual verification diagnostic binds their digests under
  `decision_contract` without duplicating unbounded rule/path collections.
- operation-plan commands put their plan under `data.plan` with a result type.
- payload-specific `type` stays in `data`; metrics are attached under `data.metrics`.
- `print_cli_error_json` maps `CliError` to the same envelope and stable exit class.
- `open_snapshot_divergence` preserves phase/authority/expected/actual/expected-record/actual-record/repository/branch; `open_recovery_pending` preserves operation/phase/repository/branch/open directory. Neither diagnostic proposes a mutating repair.
- generic repository JSON serialization and reads share a 64 MiB cap; reads are component-wise/final no-follow, non-blocking, regular-file-only, and limit-plus-one bounded. Smaller transaction/import/owner readers keep their specialized limits.

### Lock and idempotency

- mutable repository metadata/open-state mutations acquire a repository-root-bound persistent kernel `RepoLock` through a content-immutable regular no-follow inode with exactly one provable link; identity/provenance is rechecked after acquisition and at use, and optional wait/timeout owner diagnostics come only from the separately atomically replaced, no-follow/16 KiB-bounded `repo.lock.owner.json`; pathname existence is not ownership, direct live scan/verify and nested callers use one lock owner, while immutable content-addressed writes such as `run_evidence_add` can publish through the object store without that repository-wide lock;
- ordinary open consumers require `VerifiedOpenSnapshot::Steady`; only a lock-owning exact operation may receive its own `PendingCommit`/`PendingState`/`PendingActiveLedger` disposition. Fire/Resolution state and ledger authorities must agree on operation/request hash and publication status so another pending request cannot be adopted, and commit rejects any unreleased active-ledger reducer before reset;
- remote mutations acquire project/branch/request resource locks;
- idempotency records bind operation, scope/key, and payload hash so replay returns the recorded result and mismatch is a conflict; import v2 follows the same rule and rejects legacy result-only markers.
- import separately owns a persistent root bootstrap provenance lock, a durable phase transaction, and root-only reserved `.part`/`.tmp` namespaces; unpublished `.part` is not authority, fully written bootstrap `.tmp` may be recovered, and only transaction-matching write `.tmp` is removed.

### Paths

Repository discovery, open-marker validation, explicit `--path`, remote URLs, output paths, and symlink policy are separate inputs. Preserve the originating repo/open path in diagnostics and `next_actions`. `open` path failures use the typed `OpenTargetResolution` path so JSON retains reason, requested/resolved path, workspace boundary, and a safe-target dry-run action instead of collapsing capability failures into generic I/O.

### Trace and batch diagnostics

`link_batch::{validate_links, link_batch_operation_plan, link_batch_data_json}` composes the shared core endpoint predicate with self/duplicate/existing-link checks. Core endpoint issues retain `missing_atom` versus `ambiguous_atom_id`, total/omitted counts, and UTF-8-safe 256-byte ID samples. Link-batch output keeps at most 100 detail diagnostics plus an omission summary and exposes `diagnostic_total` / `diagnostics_omitted`. Plan `items` and live `added_links` are independently capped at 100 projected links, keep full `item_count` / `added_link_count`, and report `items_omitted` / `added_links_omitted`; projection strings use the same UTF-8-safe 256-byte metadata. Live endpoint failure is rendered through `CoreContext` as `trace_graph_endpoint_integrity` with repo/path context.

### Docs and test synchronization

When a command changes, update the registry, help/completion, CLI spec, this table, JSON/text tests, and the focused validation map together. `check_source_docs.py` detects command set drift, but semantic routing still needs review.
