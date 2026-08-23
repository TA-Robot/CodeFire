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
fence. That upload route now polls the exact request deadline immediately before each of its three first effects --
missing-project layout bootstrap, object copy, and branch/generation publication -- after every identity and
final-subject check and before the response slot marks the effect started, so an already-elapsed request yields the
typed deadline outcome with the authority, objects, generation, nonce and receipt unchanged. Its publication-root
validation reuses one request-owned single-use proof of the already-validated closure, so parse/validate/plan/preflight/apply
stay separate while the same root is never traversed twice and no `local_v1` ledger is created inside the request.
The retry route `handle_http_upload -> execute_remote_json_publication -> recover_remote_pending_quarantine ->
validate_sealed_commit` carries the same exact ledger and deadline: each recovered durable root is validated on the
request ledger, and the deadline is polled before each recovered root, before the receipt replay, and immediately
before the quarantine cleanup that is that route's first externally visible effect. Expiry emits a fixed 503 only after route ownership closes. Contiguous stdlib JSON
parse/materialization remains non-interruptible, so full Python bundle deadline support is still rejected. This
coverage does not imply other POST response preflight, request-header/body, full upload-work deadline polling,
other-route deadline coverage, or connection admission.

Python uploadの後段候補は4つのsingle-use ownerへ分離した。production接続済みの
`AccountedServerVerification`はproject/policy/cwd capabilityを最初のopen前から所有し、policyとambient projectを各spawn直前に
再検証し、childへpinned project/cwd FDを渡す。production uploadはhandlerの同一request ledger/deadlineをownerへ渡し、
no-ledger callerだけlocal ledgerを維持する。全reservationはcaller-owned permitを`reserve_into`でowner-first登録し、FD番号は
close前にdetachして一度だけ閉じるためsame-number ABAをretryしない。全plan capabilityを最初のfault後もbounded sweepし、primary
operation errorをexact outer、最初のcleanup errorをexplicit `__cause__`にする。live `AccountedCommitSignaturePolicy`はpolicy/history/HMACを、
live `AccountedRemoteSignaturePolicyAuthority`はabsentまたはexact single-link raw policyをno-follow project capabilityへ
固定し、same-ledger/deadlineでpure ownerへ一度だけ貸してfinal identity/content fenceを行う。malformed JSON/UTF-8はraw payloadを
exceptionへ残さず、authority closeはchildを先に閉じる。別のproduction-unwired combined surfaceはsignature childの成功・close後だけ、
同じP0 bytesとduplicated project/policy capabilityを`AccountedServerVerification`へ貸し、ambient policy readなしでplan/cwdを作る。
partial childはfallible acquisition前にauthorityへ公開し、combined construction全域のcallback再入を拒否する。handlerはまだこのsurfaceを使わない。
production接続済み`AccountedHTTPUploadSealedValidation`はsealed Stageからincoming/existingのcount/aggregate/max-record scalar
boundsだけを一度moveし、staged session/FDをhandler ledger/deadlineへ閉じる。`validate_sealed_commit`はcaller ledger/deadlineを
任意で受け取り、budgetを持たないcallerだけが`local_v1` ledgerを作る。upload routeはvalidated snapshotから
`RemoteJSONPublicationRootProof`を作ってpublication identityへ渡し、publication preflightはその`RemoteJSONPublicationIdentity`を
再利用するので、同一rootのclosure traversalは1 requestで一度だけになる。`SealedCommitValidationSession`はPath/documents/
validated_commits/descriptor registry/reuse pool/subdirectory mapをallocation前にcontrol ownerへclaimし、closeしたdescriptor ownerは
poolへ戻すのでregistryはread数ではなく同時opened descriptor数で有界になる。`_AccountedUploadCommitSource`は
`_AccountedUploadSourceMemory`でその control upperをexact request ledgerへ先に確保する。session closeは全descriptorをfieldから
detachして各numericを一度だけcloseし、fault後もregistry全体をsweepしてpayloadとlogical controlを解放した後に最初のfaultを返す。
raw FDはretryしない。wrapperは互換の最大2回callを保つが、2回目が扱うのは残ったnon-descriptor scalarだけである。
`_AccountedUploadCommitSource._open`は既存`_memory`をpoll/allocation/acquire/session constructionより前にreconcileするため、persistent
constructor cleanup failureでも11,456-byte ownerを維持し、2つ目を上書きして22,912 bytesへ増やさない。live routeの
`is_ancestor_from_objects_accounted`と`enforce_remote_commit_signature_policy_accounted`はsource closeを
`_drained_commit_source_cleanup_error`で自ら同期実行し、同じsourceを最大2回callして最初のcleanup errorを得るので、request loopがrequestを精算する時点で
ledgerはすでにdrain済みになる。`make_http_handler`の`_decision_work_observer`で測ると、`close(2)`失敗を1回含むrequestは
current debt 0、permit 0、terminal `complete`で精算される(uploadは失敗しOSErrorはloop外へ抜けるので、`complete`はaccountingの
話でありrequestの成否ではない)。callerがどのerrorを見るかはshared-primitives `C14` — in-tree authorityは
[`shared-primitives preflight`](ai/swarm-results/cfr168-python-http-shared-primitives-preflight-claude-20260731/preflight.md)
で、per-clause registerが`C14`を、decision tableが`B2`を定義する: cleanup faultはprimary operation decisionを
決して置き換えない。traversal/policy decision自体が失敗した場合はそのdecisionがouter exceptionで、最初のcleanup errorが
`__cause__`として付く。cleanupの既存effective chain、primaryの既存effective chainの順にidentity dedupしてcycle-freeなexplicit
cause chainへ続け、payload traceback localsはrelease observerより前にdetachする。operationが成功した場合にのみcleanup error自体が表に出る。両routeは以前これを反転させており
(`raise cleanup_error from run_error`)、その同じshared-primitives preflightがbehaviour-open `B2`として記録していたものを
ここで解消する。release順
(commit source → authority)と「最初のcleanup errorを使う」規則は不変。constructor unwindも同じexact-primary契約を使う。
この同期保証はこの2つのcommit-source live routeにのみ及ぶ: `_AccountedUploadSourceMemory`は
`validate_sealed_commit`でも使われ、そちらはhelperの対象外で自前のreleaseを持つ。`__del__`は全callerに対してbest-effort
fallbackであり、どのcallerにも保証ではない: 対象がcollectibleになって初めて動き、raisedされたerrorのtracebackがsourceを
pinしている間は何も返さない。さらにdirect-live `AccountedHTTPUploadAncestry`はvalidated commit loaderを借用するfull BFS decisionのcollection/frame/deadlineを閉じ、match後も
legacy late-error orderのため全走査する。production-unwired `AccountedHTTPUploadSealedAncestry`はsealed ownerのpreallocated ancestry
handleを、callerへcompositeを返した後のsingle-use `prepare`で同じledger/deadlineのBFS loaderへ貸す。BFS payload/loaderを先にdropして
borrowをreleaseし、prepare/close faultはowner fieldへretry ownershipを残す。prepare中callback closeは外側frameでcleanupしてtyped
cancellationにする。direct signature/ancestry ownerはlive routeに接続済みだが、sealed/composite successorはupload/MR/publicationの
production pathに未接続であり、terminal fence、signature/digest/ancestry ordering、最終publicationの完全なallocation/deadline closureは残る。

Existing-project terminal characterizationは、signature compositeからancestry handoffへparent tokenを移管できず、project gateを
保持してもpolicy-first/sealed-firstの両final orderにcallback mutation窓が残ることを固定する。verification subprocess failureは
external effectを残し得るがfixtureではancestry/publicationへ進まない。sealed final後にambient recordsを読むpublication helperも
detached immutable inputではない。完全なterminal pairやhandler call siteは追加していない。

production-unwired `AccountedHTTPUploadExistingTerminalOwner`はexisting-projectのconsumer lifetimeとpair detach prerequisiteを単一ledger tokenへ閉じる。
exact signature handle→pinned verification run/close→exact ancestry handle→full BFSをproxyし、failureはabort-only、callback closeは
outer-frame typed cancellation、abortはchild/borrow→policy→sealedでstop/retryする。ancestry後はpolicy→sealedのcallback-rich prepare、
最後のdeadline callback、sealed→policyのcaptured-primitive probe、両probe成功後だけのfield-only commitを行う。logical detach後の
cleanupもpolicy→sealedでstop/retryしprobeを再実行しない。成功時はresource-zeroのlive `PAIR_DETACHED` ledger capabilityだけを保持し、
検証済みexact immutable branch/headへのprivate fixed-slot参照もcloneなしで保持する。explicit closeで両方を破棄する。stable project ID authority、
missing bootstrap、semantic detach、immutable publication input、effect/publication、handler call siteは追加していない。

Sealed ownerのproduction-unwired signature/ancestry handleは、retained validated commitだけをloaderなしで借用する。raw mutable
aliasはdestination-first canonical fingerprintでhandle close時とfinal fence時に検査する。disk callback完了後の最終checkはcallback
なしで行い、成功したdocumentを即session authorityからdetachしてからscalar cleanupへ進む。lookup/release/finishはsingle-flightで、
callback後かつpayload取得前にhandle/owner tokenを再確認する。

別のproduction-unwired二段階compositeは、このsealed ownerとpinned signature-policy authorityをsame ledger/identical deadlineで束縛する。
exact preallocated signature handleをpolicy loaderへ貸し、signature child close後、parent coordinator tokenがstale aliasを拒否している間に
handleをowner-first release/revalidateする。その成功後だけverification config/cwd/childを構築できる。signature/release failureは
verification config/spawn effect 0で、abort cleanupはactive borrow→policy authority→sealed authorityの順に最初の未完ownerで停止・retryする。
handlerはまだこのcompositeを使わず、verification実行、final fences、ancestry、bootstrap、publicationも行わない。

production-unwired digest handoffは、parser pre-parse upperをcaller-owned one-shot `AccountedHTTPUploadParserScalar`で取得し、
同owner内でpost-parse exact full-request scalarへretry-safeにshrinkしてから、そのexact tokenを
`AccountedHTTPUploadRequestSource`へmoveする。Stage closure/core/newline boundsは使わない。caller-owned二段階
`AccountedHTTPUploadSemantic`がsourceをrelease/reacquire gapなしでadoptし、digestを公開せずexact semantic mapping、
full-request fingerprint、元record identity、別Retainedのeffect-ready core-plus-newline bytesを保持する。semantic側8 source passと
`4096 + 256L` canonical scratch、digest child work/scratchを別々にaccountし、deep/key-heavy combined Temporary exact-minus-oneを
拒否する。last deadline callback後にfull request fingerprint/list/ID/duplicate/full-record digest/copy parityを再検証し、mutable AST
scalarをpayload-first releaseしてsemantic/copyをfield assignmentだけでread-only mapping proxy/copy tupleのprivate effect slotへ
detachする。同tokenのduplicate bindは拒否する。existingのcontinuous gate、missingのbootstrap-locked private-stage gate/proof、
mapping lending、effect consumer、handler parser release-slot移管は未接続である。

private `_HTTPUploadSemanticTerminalProbe`はこのsemantic ownerからcallerのlast callback前に構築・captureする補助境界である。
callback前にexact instance state、immutable child-indexed parser DAG、sorted record refs、copy memoryviewsを固定し、callback後は
sortingやidentity-map lookupをせずfull request hash、sorted full-record object-set hash、root `remote` omitted core-plus-newline
parityをpreallocated frameでstreamする。single-use closureはsuccess/reject/closeでviewとpinsをrevokeするが、production handler、
deadline callback、semantic detach、existing pair capability、effect/publicationへはまだ接続しない。

production-unwired `AccountedHTTPUploadExistingPairSemanticCarrier`は、carrier用semanticがsealed reservation前に取得した
parser/semantic/runtime-sized control scalarのexact baseline proofを、ancestry-complete pairへ結合する。ledger registryをpairからcarrierへ
先に移してstale aliasを遮断し、pairの最後のdeadline callback後はcaptured sealed/policy probe→semantic probe→registry/mirror/
verified subject/proof再検査→pair commit→fresh immutable mapping proxy/copy tuple commitだけを行う。callback前後のledger mutation
epoch/current identity/open-permitとpair/policy/sealed/semantic/carrier/ledger class method identityも固定し、exact carrier tokenとcaptured
probe/commit以外のprivate呼出しを拒否する。pair cleanupを先に完了するまでsemantic baselineを
保持し、closeはpayload/primaryの後にcontrol scalarを最後にreleaseする。payload getter、stable project ID、effect/publication、handler、
missing bootstrapは持たない。

その次のproduction-unwired destinationは
`prepare_http_upload_existing_effect_input` /
`AccountedHTTPUploadExistingEffectInput`である。semantic prepareはcarrierと
effect-input双方のruntime-sized controlをsealed baseline前に一括取得し、
owner allocationはそのpreaccount内で行う。pair last callback後のcaptured
commitはfresh mapping proxy/copy tupleを一度だけownerへfield moveし、
pair/semantic cleanup完了後にledger registryをcarrierからownerへ移す。
post-commit cleanup/finish failureはcallbackやprobeを再実行せずretryし、
closeはpayload→carrier/pair/semantic→combined control→stable gateの順で
処理し、ownerは最後にcontrol解放済みを検証する。公開getter、effect、
publication、response、handlerは持たない。

liveなUpload routeのresponse lifecycle自体はADR-021として実装済みである。
`handle_http_upload`は最後のpre-effect fenceでresponse permitをsealし、
その直後・first fallible effectの直前に一方向marker `EFFECT_MAY_HAVE_STARTED`
を立てる。両者の間にあるのはlocal state代入ひとつだけで、markerはdurable byte
の可視化を証明せず「no-effectをもう証明できない」ことだけを示す。selectionは
destination-firstで、exact selected ownerがslotへ移りdirectionがimmutableに
なった後でのみrejected ownerのfallible releaseが走る。`do_POST`は`take()`せず
slotを唯一のownerとして保持し、publishは凍結header/bodyをborrowして最初の
write前に`WIRE_STARTED`を記録するため、second documentは発生しえない。
`transport_boundary`は2回のsocket writeのみを覆い、write後のaccounting
`OSError`はtransport originにならない。pre-effect abortとpost-effect finishは
別operationで、abort中のdisposal faultはresponseを一切publishせずconnectionを
閉じ、post-effect faultはfrozen success/durable responseに対して常にsecondaryで
ある。failed releaseは同一の非消費ownerを保持し、failed boundaryごとに
request loopでちょうど1回だけsynchronous retryする。recoveryはbounded
diagnosticを1回だけ消費し、persistent failureはconnectionを閉じてreachable
ownerとexact debtを報告し、既存の`cleanup_incomplete` terminalを使う。
新しいpublication/wire terminalは追加しない。

一方、既存の`_PreparedHTTPMutationResponses`を上記production-unwired
effect-input ownerの前後へ単純接続する案は引き続き
Freezeされている。terminal前にresponseをprepareするとlast callbackによる
header/body差替えがterminal probe外に残り、terminal後にprepareするとresponse
deadline callbackが検証済みgate subjectを変更できる。またsealed後の正規response
会計はexact ledger baselineを壊し、両ownerに共有adoption tokenもない。後続compositionは
success/durable responseをsealed baselineでpreaccountし、last callback前に
destination-firstで所有して、そのexact stateとpublication preflight proofをterminal
probeへ含め、effect-inputとresponse selectionを単一tokenへ移す必要がある。

そのpreparse prerequisiteとして`prepare_http_upload_response_input` /
`AccountedHTTPUploadResponseInput`を追加した。factoryはowner allocation前に
success/durable双方のbody/header upperとruntime controlを取得し、exact bytes
確定後にmax body/header workをchargeして同じRetained scalarを両payload＋control
へ縮小する。bytes/selection getterはない。opt-in semantic adoptionはparser
predecessorをupper→exactへsame-current reconcileし、sealed baselineはresponse
transition/scalar/tokenをproof化する。carrierはlast callback前にexact response
instance/class/byte identity probeをcaptureし、commit前にhook-free実行する。
class drift中はauto cleanupを延期する。effect-inputとの単一token化とhandler接続は次段である。

existing-project stable identity/exclusive gateのproduction-unwired prerequisiteは
`prepare_http_upload_existing_project_gate` / `AccountedHTTPUploadExistingProjectGate`が所有する。factoryはpath scratch後、owner
allocationとfilesystem openより先にfull Retained/Temporary upperを取得する。project/`locks`/persistent gate/16 KiB以下の
`remote_project.json`をno-follow capabilityとsingle-link inodeへ固定し、canonical org/app、version/type/generation、lowercase
`CF-PROJECT-<32-hex>`を検証してexclusive nonblocking `flock`を保持する。private semantic factoryだけがstable ID/tokenを渡し、
gate scalarをparser/semantic/controlと同じsealed baselineへ含める。gate-backed carrierではpair last callback後にgate probe→semantic
probe→pair commitsの順で再検査する。gate probeはexact instance/class/global primitiveをhook-freeに照合し、captured primitiveだけで
5本のretained FDをcaptured dev/inodeに照合してからsubject/contentとexact gate FDのexclusive lockを再assertする。driftやFD ownership不明中は
auto cleanupを延期し、explicit closeはexact owned inodeだけをunlock/close、foreign/EBADF numericをabandonしてgateを最後に解放する。production handler、
effect/publication、missing bootstrapには未接続である。

missing-project namespace transactionのproduction-unwired境界は
`prepare_http_upload_missing_project_bootstrap_transaction` /
`AccountedHTTPUploadMissingProjectBootstrapTransaction`が所有する。persistent target bootstrap lockの下でhidden stageを作成し、
そのstageのpersistent gateをexclusive保持してからterminalへ進む。typed handoff後はeffect-start→pinned-org relative
no-replace rename→same-gate durable-only reducerの一方向で、rename/reducer faultは再renameなしにreconcile/retryする。
pre-effect closeだけがexact hidden identityを削除し、effect後はreducer完了までcloseを拒否する。`upload` handlerの既存/missing
分岐、semantic/write/response/copy proof、publication reducerには未接続である。

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

### Filesystem authority failures on read and mutation routes

- Every no-follow rejection, symlinked or directory authority, unavailable path, permission denial,
  metadata failure, and failed read maps into one typed diagnostic family, `filesystem authority is
  unusable: <target>: <reason>`, published as 400.
- The shared Python `do_GET`/`do_POST` dispatchers publish that family for any filesystem-origin
  `OSError` escaping a route, so Doctor, branch list, merge-request list, bundle, upload,
  merge-request, and GC can never answer a filesystem fault with a zero-byte reply.
- Origin is decided by the boundary or owner that raised the error, not by its exception class, and the
  one shared taxonomy is exhaustive over three origins: transport, filesystem, and commit-cleanup.
  `transport_boundary()` wraps the request-line, header, body, and response-byte transfers and marks
  what they raise; `route_filesystem_boundary()` wraps route execution and marks every unmarked
  `OSError` it lets escape; `_drained_commit_source_cleanup_error` marks the standalone `OSError` its
  synchronous drain returns as commit-cleanup origin, at that owner and nowhere else.
  `classify_route_os_error` is the one shared decision both dispatch arms
  use, and it refuses the commit-cleanup origin exactly as it refuses transport. A real socket timeout,
  reset, broken pipe, or disconnect abandons the connection without
  fabricating a second response, and the abandoned request still closes its ledger with zero debt and
  zero open permits; a commit-source cleanup fault likewise escapes rather than publishing a typed
  authority, after the request loop has already accounted the drained ledger;
  a filesystem `ETIMEDOUT` publishes the typed family on both GET and POST.
  Request-deadline cancellation keeps its typed 503 and `MemoryError` stays outside the arm.
- A leaf authority is absent only through a proved parent directory capability. `_open_authority_parent`
  pins the parent no-follow and records its identity in the capture; a missing, replaced, symlinked,
  denied, unreadable, or metadata-failing parent is a typed filesystem-authority failure, never
  absence, so a stale merge request is never silently downgraded to open.
- Following an authority symlink is not preserved compatibility. A symlinked merge-request, branch, or
  descriptor record is rejected explicitly where the pre-fix route silently followed it.
- A pinned authority record replaced during a request is rejected as `pinned authority record changed:
  <path> (<reason>)`, with reasons `unreadable`, `appeared`, `parent_changed`, `identity_changed`,
  `mode_changed`, `link_count_changed`, `size_changed`, and `digest_changed`.
- Cleanup releases are retryable and idempotent: no scalar, session, owner, or map entry is dropped
  before its exact release succeeds, and a cleanup that cannot reach zero terminates the request ledger
  as `cleanup_incomplete` rather than publishing a success with residual current work. An exact release
  larger than the owner still holds is not absorbed: it fails closed as `decision-work owner
  over-release: <owner> owns <n>, release requested <m>`. Session index ownership is released per key,
  and never as an aggregate that a still-unreleased key would double-release on the next attempt.
- A cleanup fault that a retry repaired is consumed, not stored unread: it becomes a note on the primary
  diagnostic when the request already failed, and otherwise reaches the handler's own error log as
  `recovered cleanup fault: <bounded json>` with its attempt count and fault text. A clean request emits
  nothing.
- The read routes' publication point is `_HTTPReadResponseSlot.prove_and_take` followed by
  `prepared.publish`, and it runs in `do_GET` *after* the route and both of its context-manager
  teardowns. The pinned lock unlock and the four pinned descriptor closes therefore precede the final
  proof rather than following it. A slot that was not proved refuses to hand over its bytes
  (`prepared read response was not proved publishable`), and a failed proof publishes no success byte,
  logs `prepared read response cannot publish with unaccounted current work`, and terminates the ledger
  as `cleanup_incomplete`.
- A filesystem-origin `OSError` raised by the post-route unlock or descriptor close reaches the same
  typed 400 family. The already-prepared success is displaced first, so a proven-but-superseded response
  can never replace the typed diagnostic. If that disposal fails, the ledger state is unknown: nothing is
  published, the disposal fault is recorded on the primary diagnostic and logged, and the connection
  closes.
- One MR-list request performs exactly five physical `remote_project.json` descriptor reads with named
  owners: the project gate pins the descriptor before locking, reproves it once on acquisition and once
  at its terminal fence, `stable_project_id` charges and parses the single accounted copy the subject
  comparison uses, and the final authority fence reproves those exact owned bytes.
- The dispatcher arm is a shared correction the merge-request mutation and GC routes reuse. It does not
  make their own product work complete: those routes remain separately reviewed successors and still
  owe their own ledger/deadline/accounting evidence.

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
