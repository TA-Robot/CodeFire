# CodeFire Source and Persisted Data Model Catalog

Status: canonical

Owner: Core and Store Maintainers

Update trigger: a Rust domain/CLI type, object kind, persisted JSON shape, version/default, or compatibility rule changes.

This catalog records current shapes at the level required for safe changes. Unless a row says otherwise, “current writer” means the Rust-first/default CLI, not the Python fallback. Exact serde/constructor behavior in source and golden tests remains authoritative; examples are not promises that every legacy record or fallback writer contains every field.

## Version Axes

| Axis | Rust-first/default writer | Compatibility note |
|---|---|---|
| Core model `VERSION` | `1` | Most core serialized structs carry `version: 1`. <!-- constant: VERSION=1 --> |
| Atom hash schema | scan/commit `AtomIndex` emits `4` | Rust init's empty Atom index and Python fallback currently omit it; missing value follows the explicit legacy/default path. <!-- constant: ATOM_HASH_SCHEMA_VERSION=4 --> |
| Repository layout | `repo.json.version = 1` | Migration validates unsupported values explicitly. |
| Commit root schema | Rust init/commit payloads emit `2` with commit/root object version `1` | Missing schema selects legacy schema 1 with the old required-root/shape contract; only schema 1/2 and commit/root version 1 are readable. Unknown future values fail closed. |
| Public object digest prefix | 24 hex | Legacy 12-hex IDs remain readable when full record hash agrees. <!-- constant: OBJECT_ID_DIGEST_HEX_LENGTH=24 --> |
| Automation envelope | Rust declared JSON commands use `codefire.command_result.v1` | Python fallback is primarily a human-text CLI and does not provide a shared `--json` surface or common envelope; isolated JSON-producing commands are separate contracts. Payload types are independently versioned inside Rust `data`. |
| Capability/support manifest | `codefire.capabilities.v1` plus `codefire.support.v1` | Unknown versions and critical extensions deny; noncritical object extensions preserve. Checked schemas and the checksum corpus live under `schemas/public/` and `schemas/conformance/`. |
| Collection page/cursor | `codefire.page.v1` / `codefire.page_cursor.v1` | Cursor binds collection, complete sorted-result snapshot, offset, and limit. Mutation causes explicit stale-cursor rejection; cursor payload is bounded and canonical-hex encoded. |

Product/package/SemVer is independent; see `strategy/release-and-format-roadmap.md`.

### Rust/Python Writer Boundary

| Surface | Rust-first/default CLI | Python fallback |
|---|---|---|
| New public Object ID | 24-hex digest prefix | 24-hex digest prefix; authenticated transferred 12-hex history remains readable |
| Atom hash schema | normal scan/commit emits 4; initial empty roots currently omit | omitted |
| Sealed semantic profile | current init/commit emits `sealed-snapshot-v1` | omitted and therefore read as `legacy-unversioned` |
| Commit root schema | emits 2 | omitted |
| JSON command envelope | `codefire.command_result.v1` for declared JSON support | No shared `--json`/envelope surface; output is primarily human text, with isolated command-specific JSON outputs treated separately |

This is an observed compatibility gap, not a promise that either writer can safely rewrite every repository produced by the other. The release/format fixture matrix owns that decision.

## Core Domain Types

| Type | Essential fields / meaning |
|---|---|
| `Atom` | stable `atom_id`, kind/source location, content hash and metadata used for identity/trace. |
| `AtomIndex` | `version`, `hash_schema_version`, Atom collection, duplicate IDs; identity-producing snapshot. |
| `TraceLink` / `TraceGraph` | typed source/target relation; `validate_trace_graph_endpoints(AtomIndex, TraceGraph)` is the shared direct-edit/batch predicate and rejects `missing_atom` or `ambiguous_atom_id` endpoints. `TraceGraphEndpointError` retains total/omitted counts, at most 100 stable details, and UTF-8-safe 256-byte `link_id`/`atom_id` samples with truncation metadata. |
| `TracePolicy` / `RequiredLinkRule` | link completeness rules and blocking behavior. |
| `VerificationPolicy` | commit policy (including the legacy-defaulted `require_commit_signature`) and constrained verification commands; signature generation/verification remains owned by the CLI `signatures` module. |
| `EffectivePolicy` / `EffectivePolicyRule` | `effective-verification-policy-v1`; stable rule ID, default/repository/CLI source, effective JSON value, applicability, safety class, ordered rule set, and digest. |
| `VerificationCommand` | `id`, command, cwd, timeout, output limit, environment/inheritance controls, and `allow_failure`; there is no `required` field on this type. |
| `AssessmentSubject` | versioned live-open or sealed-pair identity with trust kind, subject/content digest, optional live worktree-manifest digest, immutability, and applicable repository/branch/open/base/source coordinates. |
| `VerificationExecutionRecord` | command profile/phase, assessment/input/environment/worktree digests, cwd/environment policy, resource counters, result/timeout/termination scope, and redaction count; no raw environment values. |
| `Fire` | stable UID/display ID, source/target Atom refs, reason/status, trace path, creator/time, key, and optional resolution/obsolete fields. |
| `Resolution` / `ResolutionBasis` | decision/rationale/evidence refs plus source/target content, Trace Link, and policy basis. |
| `ChangeCoverage` / `ChangeCoverageEntry` | `file-relation-v1`; every manifest-changed path is non-Atom, deleted-Atom, out-of-Atom, or Atom-changed with unclassified spans. It records normalized Atom/blob facts, summary, optional tool migration, `file_only_no_span_claim`, and digest. |
| `ScanResult` | current index/graph, changed Atoms, compatibility non-Atom changed-file projection, typed change coverage, tool migration, open Fires. |
| `Verification` | result plus unresolved merge conflicts, open required Fires, failed checks, missing links, stale resolutions, missing evidence refs, duplicates, optional assessment subject/execution records, effective policy, and change coverage. Historical omission of the latter two makes no claim. |
| `VerificationInputs` | one construction packet for merge, link, check, resolution, evidence findings, assessment subject, and execution records before the core derives a Verification. |
| `UnresolvedMergeConflict` / `MergeConflictKind` | active text path, binary path, or semantic Atom subject projected into the sealing decision. |
| `validate_sealed_snapshot_semantics[_with_contract]` | authoritative schema-2 cross-root predicate: recomputes Fire count, duplicate Atoms, required links, stale resolutions, trace-completeness/result, validates policy/coverage contracts and projections, and checks Fire/active-Resolution UID/cardinality/identity agreement. |
| `DecisionWorkProfile` / `DecisionWorkLedger` | advertised `codefire.decision-work.v1` limits and one non-cloneable command/request ledger. Command kind and stable phase are applicable-profile inputs; terminal reason is one-way (`complete`, `early_match`, `budget_exceeded`, `corrupt_subject`, `subject_changed`, or `deadline_exceeded`). |
| `DecisionWorkReservation` / `DecisionWorkSnapshot` | one logical effect's cumulative/maximum/concurrent dimensions and correctness counters are preflighted together and committed atomically. Snapshot exposes bounded phase deltas, peaks, counters, and terminal reason without controlling domain semantics. |
| `DecisionWorkLedgerIdentity` / `DecisionWorkLedger::{identity,matches_identity}` | opaque, non-serializable binding for a prepared owner that cannot retain a ledger borrow. The numeric ledger ID stays private; Store/CLI owners compare the token before applying work, consuming permits, or releasing concurrent ownership, including permit-free exact/replay paths. |
| `DecisionWorkPermit` / `DecisionWorkLedger::{reserve,reserve_into,consume_reserved,consume_reserved_pair,cancel_reserved,seal_prepublication}` | non-serializable ledger-bound publication headroom. Reservation does not inflate actual counters; consumption cannot exceed its phase/surface/dimension/counter upper bound, and two different-phase permits can be preflighted and committed atomically. Python `reserve_into` lets a fallible owner store a caller-created permit before registration; after capacity preflight it binds once to that ledger before the fault-injectable registration boundary. Cross-ledger registration and reuse after cancel/full consume are rejected. A foreign permit is rejected, ordinary accounting is disabled after the final publication seal, `has_open_permits` is a side-effect-free transfer guard, and terminal completion requires no open permit. |
| `DecisionWorkBudgetExceeded` | blocking `resource_budget_exceeded` diagnostic with exit 60, profile, dimension, saturating-u64 `observed=limit+1`, limit, phase, command, and surface-specific continuation support. HTTP maps it to 413 without partial success. |
| `store::{DiagnosticScanWork,DiagnosticScanVisitOutcome}` / `doctor::checks::DoctorObjectIssueSpec` | non-persisted local Doctor object-diagnostic handoff. The consuming work view keeps its Retained permit inside an RAII scope, performs fallible kind/message/path/slot materialization, consumes the exact scalar, invokes one infallible publisher, and yields only a Store-constructed opaque outcome. Error or unwind cancels the permit; a published owner cannot be followed by a separate visitor error. Failed prepared cache seeds drop before their scalar and visitor-growth rollback. This owner covers object-scan diagnostics only. |
| `HttpRequestRoutingOwnership` / `HttpRequestBodyOwnership` | non-serializable Rust parser owners for method/path Strings and one raw JSON `Value`, each with exact Retained scalar and ledger identity. Destinations are acquired before header/body scratch release, malformed/truncated cleanup preserves unrelated owners, and payload always drops before scalar. |
| `HttpMutationRoutingOwnership` / `HttpMutationRouteIntent` / `HttpMutationDerivedIntent` | non-serializable decoded org/app/branch-or-MR routing, moved canonical request, and real upload/apply `String`/`PathBuf`/`Value`/publication/apply-identity payloads. Mutation shape selection allocates no route vectors; decoded and derived destinations reserve first. Both routing owners must be absent and derived structure complete before Store, whose current Retained must equal raw plus derived ownership. Production upload additionally reserves overlay destination slots, moves object values out of the request, removes core fields from each owned JSON map, reuses the residual map as extensions without cloning historical keys/values, and transfers the same compact Retained scalar to Store. Missing bootstrap permits remain separate external owners. |
| `RemoteRequestPayloadEvidence` | opaque non-serializable proof created only by successful full-request signature preflight. It binds the validated payload digest and the canonical request with `objects` elided to one ledger identity. Final upload validation accepts only an empty consumed object array plus an otherwise identical request; the streaming digest polls and accounts bytes before hashing without retaining a serialized copy. |
| Python `_ProcessHTTPConnectionAdmission` / `_HTTPServerRegistration` / `_HTTPConnectionPermit` | process-global, lock-protected, zero-application-queue availability gate outside the deterministic ledger. A transactional live-server owner restores the prior registry/effective cap after partial constructor publication; the registry selects the minimum live profile cap under the same lock as active ownership. A stricter cap grandfathers admitted sockets but blocks new admission. Last release removes closing state, and one retryable permit binds each accepted plain/TLS socket through handler and shutdown cleanup. Plain over-cap gets one fixed typed 503 before dispatch-owned shutdown; TLS closes before handshake without plaintext. |
| Python `AccountedHTTPBundle` | fallback-only owner for one read-only branch bundle projection. It is bound to the request ledger and keeps compact canonical branch/object bytes retained through response freezing. Close validates that binding and becomes closed/cleared only after retained release succeeds; foreign-ledger or release failure preserves payload, amount, and retry/drop cleanup. It is neither a persisted format nor a mutation capability. |
| Python `AccountedHTTPUploadPlanner` | fallback-only transient owner for upload closure working collections. It binds the incoming lookup, seen set, and LIFO frontier to one request ledger; reserves all incoming-index `PyLong` owners from the record count before enumeration, runtime `sys.getsizeof` bases for each distinct collection, ASCII string ownership, collection-specific capacity growth, and old-plus-possible-new backing-table overlap before every insertion because runtime rehash counts differ from logical growth counts. It sizes validated ASCII object IDs without allocating encoded copies. Frontier/traversal/index locals are dropped before planner scalar release on success or failure. Close clears all collections before Temporary/Retained release and keeps retryable scalar state on cleanup failure. |
| Python `AccountedHTTPUploadClosure` | fallback-only transient owner for one HTTP upload stage-input closure projection. It binds incoming record indexes and existing object IDs to the handler request ledger, reserves both list bases, each retained entry, and sort scratch before allocation, and polls the deadline during collection/reference traversal and sorting. The accounted path does not allocate legacy output lists. Close clears both lists before releasing retained/temporary charges and preserves scalar cleanup state for retry. Incoming lookup and seen/frontier belong to `AccountedHTTPUploadPlanner`; record payloads and publication state remain outside this owner. |
| Python `AccountedHTTPUploadStage` | fallback-only transient owner for staging projection/serialization. It strongly owns the exact records and closure lists, reserves Retained subject storage before tuple allocation, and freezes incoming index/object ID/compact size/digest/read-only canonical payload plus existing ID/payload/device/inode/link-count/size/content-hash evidence. Aggregate work covers initial serialization, incoming canonical revalidation, and two bounded existing reads. Immediately before temporary-root creation it revalidates caller list contents, incoming identity/size/digest, and existing evidence while fixing count/aggregate/max-record serialized-byte bounds across both private tuples. No payload plan escapes: a single-use writer clears iteration locals and strips failure tracebacks; a separate seal-after exactly-once accessor moves only the immutable scalar bounds to sealed validation. Existing reads require a regular single-link inode at every initial/opened/after/named fence. Close drops strong/private payload ownership before retryable scalar release. |
| Python `AccountedRemoteObjectSetDigest` | fallback-only, non-persisted prerequisite owner for canonical object-set digest work. Its caller provides an upper bound derived from parser-owned canonical source bytes. Before list allocation or hashing it reserves a source-proportional iterative-encoder envelope, ordered references, digest, aggregate bytes, and object count. It heap-sorts references, validates IDs and the source upper before encoder mapping-key allocation, and accounts every emitted canonical byte into SHA-256. Input order and the legacy digest remain unchanged. Hash tracebacks are detached before payload-first cleanup, and pre-consumption permits remain owner-held across cancellation faults. No digest string escapes; only owner-scoped constant-time comparison is public. Production destination transfer is not wired. |
| Python `AccountedHTTPUploadParserScalar` / `AccountedHTTPUploadRequestSource` / `AccountedHTTPUploadSemantic` / `_HTTPUploadSemanticTerminalProbe` | non-serialized, production-unwired two-stage implementation of the frozen upload-semantic ownership boundary. A caller-owned one-shot token acquires the parser pre-parse Retained upper and retry-safely reconciles it in-place to the exact post-parse scalar; the source moves that exact token into one request/objects identity and rejects unreconciled or duplicate bind, rather than trusting forgeable generic ledger current. The caller receives the destination before single-use prepare; it adopts the exact source owner, rejects Stage-like false bounds, destination-publishes the digest child/caller-created permit, keeps the digest and full-request fingerprint private, and separately preaccounts eight source-sized semantic work passes, a conservative `4096 + 256L` canonical scratch envelope, and effect-ready core-plus-newline immutable copy bytes. Carrier-intended prepare additionally acquires one runtime-sized, page-rounded conservative Retained control upper before sealed reservation. The private terminal probe is constructed/captured before a caller-owned final callback, discards its mutable builder, and uses an immutable child-indexed DAG plan, captured instance-dictionary descriptors, five preallocated frame arrays, and pinned memoryviews for callback-free full-request/object-set/copy-parity streaming. It has exact single-use revocation and complete partial/late-construction view cleanup, but invokes no deadline callback and mints no detach/effect capability. The semantic owner's standalone terminal path releases the parser scalar and field-detaches a preallocated read-only mapping proxy/copy tuple into private effect slots. No digest/copy getter exists. Handler token mint/reconcile/transfer, recovery/publication mapping lending, missing bootstrap proof, and effect consumption remain unwired. |
| Python `AccountedHTTPUploadExistingProjectGate` / `ExistingProjectGateChanged` | fallback-only, non-serialized, production-unwired owner for one existing remote project's stable identity and exclusive persistent gate. `prepare_http_upload_existing_project_gate` acquires path Temporary and then the complete owner/capability/raw-descriptor/JSON/scratch Retained+Temporary upper before allocating the owner or opening a path. It requires zero live concurrent request owners, walks the absolute parent without following links, pins exact project and `locks` directories, opens or creates a persistent single-link gate inode, takes nonblocking exclusive `flock`, and pins a single-link descriptor no larger than 16 KiB. Exact type/version/generation, canonical org/app, lowercase 32-hex stable ID, opened/named inode, metadata, bounded bytes, and final content are fenced. A private single-use semantic handoff places the gate scalars into the parser/semantic/control sealed baseline and proves gate acquisition preceded sealed construction. The exact gate/token and stable ID then move into a gate-backed pair-semantic carrier; terminal revalidation first rejects exact-instance/class/global primitive drift without dispatch, classifies all five numeric FDs against captured owned inode identities, then uses captured operations to recheck subject bytes and exact-FD exclusive lock ownership before semantic/pair commit. Automatic cleanup is deferred when FD ownership is unconfirmed; explicit cleanup unlocks/closes exact original inodes only and abandons foreign/EBADF numerics. Successful close releases the gate last. No stable-ID/payload getter, production handler, effect/publication, or missing-project bootstrap is provided. |
| Python `AccountedHTTPUploadExistingPairSemanticCarrier` | fallback-only, non-serialized, production-unwired payload-opaque carrier for one ancestry-complete existing pair and one carrier-intended prepared semantic on the exact same ledger/deadline. Sealed construction accepts the semantic/parser/control scalars, plus an exact gate baseline when present, as the complete live baseline and pins transition identities, amounts, permit state, and release flags. Adoption moves the request registry pair→carrier before mirror publication and rolls partial bind back to the pair. One exact carrier token binds the pair mirror and captured semantic probe/commit identities; a gate-backed semantic additionally requires the exact typed gate/token and pre-sealed acquisition epoch. Around the pair's last callback, the ledger mutation epoch/current identity and empty permit set plus ownership class identities are pinned. Captured sealed/policy probes, optional gate probe, and callback-free semantic probe all pass before pair field commits; captured exact instance-state checks then guard registry, mirrors, verified subject, proof, private stable ID, and a fresh private mapping proxy plus copy tuple commit. No source mapping backing alias is retained. Pair cleanup precedes semantic parser/Temporary cleanup; close drops payload/primary ownership, runtime-sized control, then the gate, with stop/retry reconciliation and no probe rerun. The page-rounded control value is derived from runtime `sys.getsizeof` shells (16384 bytes in the canonical test environment), not a wire-format constant. It exposes no payload/effect/stable-ID getter, publication capability, handler, or missing-project bootstrap. |
| Python `AccountedHTTPUploadExistingEffectInput` | fallback-only, non-serialized, production-unwired destination-first owner for the existing-project carrier's terminal mapping proxy and immutable copy tuple. An explicitly opted-in semantic preaccounts this owner's runtime-sized control together with carrier control before sealed construction. `prepare_http_upload_existing_effect_input` allocates within that scalar before the pair's final callback and binds captured exact carrier/owner/class/registry commit and finish slots. Terminal success stages the original carrier commit result without a getter, then moves registry ownership only after pair/semantic cleanup; either post-commit boundary retries without another callback. Direct carrier close is rejected while adopted. Owner close drops the input, closes pair/semantic and the optional stable gate, and verifies the combined control release. It exposes no effect/publication/response/handler capability and has no missing-project claim. |
| Python `AccountedHTTPUploadResponseInput` | fallback-only, non-serialized, production-unwired preparse owner for immutable success/durable response bytes. Its factory reserves maximum response/header work and acquires both bodies, both headers, and runtime control before owner allocation. Single-use prepare accepts exact `bytes`, charges the maximum selectable body/header, cancels unused work, and shrinks the same Retained scalar to both payloads plus control. Opt-in semantic adoption same-current reconciles the parser predecessor from upper to exact; sealed baseline pins response owner/token/transition/scalar, and the pair carrier invokes a captured exact instance/class/key/byte identity probe after the last callback. Class drift defers automatic cleanup. It exposes no payload or selection getter and drops bytes before retryable release. Effect-input single-token composition, publication, and handler wiring remain separate prerequisites. |
| Python `AccountedHTTPUploadExistingPublicationInput` | fallback-only, non-serialized, production-unwired first destination for an existing-upload publication plan. Its factory acquires the caller-declared plan upper, optional semantic upper, runtime control, runtime-measured depth-128 validation/encoding Temporary, and bounded 8-pass plan plus explicit identity/semantic/request-envelope work before owner allocation; a ledger-held bootstrap transition makes pre-owner cleanup faults retryable through the same factory. Single-use prepare bounds outer counts first, accepts only exact JSON trees, proves each remaining canonical size before materialization or legacy subject/path validation, validates authority components against relative-path traversal, and freezes 1..64 ordered authority/before/after triples plus operation/actor/key/roots/result as private canonical bytes and hashes. In request-bound mode the same still-open transition requires a fully initialized exact prepared semantic/source/allocation on the same ledger, binds it through a private token, streams the legacy canonical request domain from frozen identity and bounded semantic bytes, then releases Temporary. Semantic cleanup owns the bound input; sealed validation pins its exact owner/token/transition/scalar/hash identity while preserving the established response proof tail; and the pair carrier adds a callback-free exact class/instance/plan/result/hash probe to the combined terminal fence. It neither reads ambient project/preimage state nor exposes a payload/effect/hash API; pinned gate-relative preimage proof, shared effect/response terminal-token adoption, reducer, and handler wiring remain required. |
| Python `AccountedHTTPUploadMissingProjectBootstrapTransaction` | fallback-only, non-serialized, production-unwired owner for the local missing-project namespace transaction. `prepare_http_upload_missing_project_bootstrap_transaction` preaccounts path construction and the complete owner upper on the request ledger, holds a persistent target bootstrap lock, creates one exact `.bootstrap-<stable-id>` stage below a pinned organization, writes its bounded descriptor/base layout, and acquires the stage's persistent single-link project gate before terminal. Its private single-use handoff enforces `PREPARED_HIDDEN -> TERMINAL_READY -> EFFECT_STARTED -> RENAMED_VISIBLE -> REDUCING -> COMPLETE`; visibility uses one pinned-org descriptor-relative no-replace rename, reconciles set-then-raise, and never reopens or reacquires the gate. Only the caller-supplied exact-function durable reducer receives the private capability after rename; reducer failure remains retryable without another rename. Pre-effect close identity-fences and removes only the hidden stage, post-effect close is refused until reducer completion, and explicit FD cleanup closes only captured device/inode identities. It does not yet adopt semantic/request-hash/write/response/copy proofs and is not connected to the production handler. |
| Python `AccountedServerVerification` | fallback single-use production owner for project/policy/cwd capabilities, parsed policy and check plans, bounded combined output, process-group cleanup, and ledger state. Production HTTP upload passes the handler's same request ledger/deadline through the wrapper; standalone/no-ledger callers retain a local ledger. Its normal path pins and rereads single-link policy bytes from the project capability. A production-unwired seeded path, reachable only from the signature authority after signature success/close, borrows identical P0 bytes and duplicates pinned project/policy capabilities without `_read_policy`. Both paths fence project/policy evidence before every spawn and pass pinned project/cwd FDs through `/proc/self/fd`. Owner-first `reserve_into`, parent-first partial-child publication, construction reentry guards, and fixed pending states reconcile faults. A close sweeps all currently owned plan capabilities once after the first fault; FD ownership is detached before one captured-builtin close and is never retried. The exact operation failure remains outer, the first cleanup failure is its explicit cause after both effective prior chains, and config/process traceback payloads drop before scalar release. Bootstrap is no-policy only while the actual project path remains absent. Command external effects are not rollbackable. |
| Python `AccountedRemoteSignaturePolicyAuthority` / `SignaturePolicySubjectChanged` | fallback-only direct-live owner for one remote signature-policy subject; only its combined signature-to-server-verification successor is production-unwired. It pre-reserves path materialization, walks absolute components without following links, and pins an absent project/policy or one regular single-link policy descriptor with exact metadata, raw bytes, and SHA-256. Raw bytes have no public accessor. Its signature-only surface lends them once to `AccountedCommitSignaturePolicy`; its combined surface verifies and closes that child before lending the identical bytes and duplicated capabilities to `AccountedServerVerification` on the same ledger/deadline. Child lifetimes do not overlap, partial verification construction is owner-first, and callback reentry cannot consume the final fence. Malformed JSON/UTF-8 becomes payload-free `CFError`. Final validation fences ambient parent/project plus named/descriptor policy identity/content. Children close before raw/digest and detached FDs/scalars; cleanup never retries a consumed numeric descriptor. Fully restored unobserved rename history remains indistinguishable without a journal. |
| Python `AccountedHTTPUploadSignatureVerificationComposite` | fallback-only, production-unwired two-stage controller owning one validated sealed owner and one unused signature-policy authority on the same ledger and identical deadline. `prepare` activates the exact preallocated signature borrow and passes that object itself as loader without wrapper, clone, session load, or store reopen. Signature verification and child close precede owner-first borrow revoke/revalidation under both parent guards; only successful midpoint release may construct or publish the verification child. Failure returns no child, marks the controller abort-only, and retains both parents. Retryable cleanup releases active borrow, then policy authority, then sealed authority, stopping before sealed cleanup if policy cleanup fails. It does not run verification, finish either final fence, perform ancestry/bootstrap, or authorize publication. |
| Python `AccountedHTTPUploadExistingTerminalOwner` | fallback-only, production-unwired existing-project consumer-lifetime and pair-detach owner. One ledger registry token blocks stale aliases of its exact validated sealed owner and unused pinned policy authority before either mirrored parent token can run a callback. It proxies the exact signature handle through signature verification and revoke, constructs/runs/closes the pinned server-verification child without returning it, then separately proxies the exact ancestry handle through one full BFS and close. Fixed control slots pin clone-free references to the exact immutable verified branch/head. After ancestry it prepares policy then sealed under callbacks, performs one last deadline callback, probes sealed then policy through pre-callback-captured functions/primitives and preowned/accounted buffers/FD slots, and commits logical detach only after both probes pass. Cleanup is policy-before-sealed stop/retry and never reruns probes. Success leaves one live resource-zero `PAIR_DETACHED` ledger capability and the private subject refs until explicit close. Failure, including a subprocess effect or half-detach injection, is abort-only. Stable project identity, missing-project bootstrap, semantic input, immutable publication input, effect/publication, and handler wiring are absent. |
| Python `AccountedCommitSignaturePolicy` | fallback-only direct-live pure owner for raw/parsed signature policy, borrowed keys, iterative `(commit_id,is_head)` frontier, seen membership, and chunked HMAC work. Fixed-slot intrusive transitions publish permits before `reserve_into`, separate HMAC storage/work, reconcile reserve/consume/cancel/release faults, and unlink fully released nodes in O(1). Legacy head/history and signature decisions are preserved. The direct signature-enforcement route is production-connected; sealed/composite successor wiring to MR/publication remains open. |
| Python `AccountedHTTPUploadSealedValidation` / `_HTTPUploadSealedCommitBorrow` | fallback post-staging owner wired to production HTTP upload when a request ledger exists; its commit-borrow surface remains production-unwired. It receives the sealed Stage's exactly-once record-count/aggregate/max-record scalar bounds, registers owner-first before session allocation, borrows staged capability/documents, preserves iterative DFS/root first-error/final fences, and polls the handler deadline. Purpose-specific preallocated handles expose only already-retained validated commits without loader/reopen. A destination-first per-record canonical fingerprint detects synchronous, same-size nested, and post-close raw-alias mutation; disk finalization retains documents until a callback-free memory check immediately detaches each exposed authority. Lookup/release/finish guards and post-callback token checks prevent revoked payload return and teardown reentry. Failed lookup tracebacks and all session payloads drop before scalar release. Traversal FDs detach before one captured-builtin close; exceptions consume numeric ownership, so a synthetic pre-call fault may leak the original FD rather than risk same-number reuse damage. The no-ledger direct path retains legacy validation. Its reservation carries no per-record heap multiplier: retained terms are the owner's fixed fields, two borrow handles, the session control upper, the publication-proof upper, a fixed control allowance plus borrow fingerprint per record, the staged aggregate twice, and the pair-probe plan, each matching an actual retained claim; the temporary upper takes the largest of the three non-overlapping per-record scratch sets rather than their sum. Every canonical fingerprint, final detach, and terminal pair probe streams through `canonical_json_stream_digest`, so no whole-record encode is reserved or held. It mints one single-use `RemoteJSONPublicationRootProof` from `validated_commits`, claimed inside the reservation and revoked at the final fence, which lets publication identity and publication preflight reuse one traversal. The final fence asserts in production that the covered pool settled to zero, that each high-water sits inside the admitted reservation, and that the pool was used. |
| Python `RemoteJSONPublicationRootProof` / `RemoteJSONPublicationIdentity` | fallback-only, non-serialized request-scoped certificates. The root proof records the absolutised store path, the ordered roots a request-owned sealed traversal already validated, and the request deadline; every consumption polls that deadline and refuses a different store, an uncovered root, or a revoked proof. It is atomically single-use over `available`/`consuming`/`consumed`/`revoked`: the consuming claim is taken before the deadline poll, which is the only callback it can reach, so an alias, a callback reentry, or a second call all receive the exact typed state error, while any refused check restores `available` and consumes nothing. `revoke` is terminal and idempotent from every state. The identity records one completed publication-identity decision and re-proves operation/actor/key/roots/store against the arguments of any reuse, so `preflight_remote_json_publication` skips nothing silently. Both are allocated inside the minting owner's prepared reservation and consumed inside its lifetime, so neither outlives its own accounting. |
| Python `_AccountedUploadSourceMemory` | fallback-only control-state owner for one commit-source validation session. It acquires the retained session control upper -- absolutised store `Path`, document memo, validated-commit set, descriptor registry, descriptor reuse pool, subdirectory map, and every descriptor owner -- on the caller's exact ledger in `graph_discovery` before the constructor that allocates them, refuses any retained control claim without an outstanding reservation, and refuses to close with control debt in either dimension. `temporary_bytes` is instead an exactly acquired dimension: the route's only temporary control state is `finalize`'s ordered object-id projection, sized `sizeof([]) + pointer_bytes * documents` (56 B empty, 64 B for one record, 272 B for 27), so it is acquired for that exact amount on the caller's ledger and the claim's own `snapshot_revalidation` phase before the list exists, with the caller ledger raising its own typed budget diagnostic, rather than covered by a count-independent reservation. Document, traversal-frame and read-scratch claims stay on the same ledger and phases they already used, so established traversal budgets are unchanged. Session close detaches and attempts every descriptor exactly once, continues after faults, drops payloads, and releases logical control before returning the first fault; raw descriptor numbers are never retried. Scalar releases remain exactly once. Before `_AccountedUploadCommitSource._open` performs any poll/allocation/acquisition/session construction it reconciles an existing memory owner, so persistent cleanup preserves the same 11,456-byte owner instead of accumulating 22,912 bytes. `_control_charge_witness` distinguishes `RESERVED_DIMENSIONS`, whose upper is already on the ledger, from every other dimension, which must be visible on that ledger before the structure it pays for may be allocated. |
| Python `AccountedHTTPUploadAncestry` | fallback-only direct-live owner for one full breadth-first commit-ancestry decision. A caller supplies one ledger, deadline, and borrowed validated-commit loader. It reserves seen/deque bases, frames/seen entries, per-insertion storage and old-plus-new overlap before allocation. A fixed control slot consumes transition bytes before construction and a child frame before `depth + 1`; actual CPython set/deque growth and popleft churn are fixture-bounded. It accounts nodes, edges, depth, and frontier, finishes traversal after a match for legacy late-error order, reconciles post-effect faults, and drops loader/commit/parent/loop payloads before scalar release. Sealed-session borrowing is available only through the production-unwired composite below; handler/final-fence and ordered signature/digest wiring remain open. |
| Python `AccountedHTTPUploadSealedAncestry` | fallback-only, production-unwired two-stage composite binding one validated sealed session to one ancestry decision. Its factory returns caller ownership before explicit single-use prepare. Prepare publishes cleanup fields first, activates the sealed owner's preallocated `ancestry` handle, and constructs `AccountedHTTPUploadAncestry` with that exact callable and identical ledger/deadline. No loader/reopen/clone path exists. Close is LIFO: ancestry collections, frames, and loader drop before borrow verification/revocation; prepare or close faults retain the exact owner for caller retry. Partial decision/reentrant prepare is rejected and callback close is carried through the active outer frame; a closed prepare returns typed cancellation rather than success. The composite never calls the sealed final fence and is not connected to handler, signature, digest, or publication. |
| Python `_PreparedHTTPMutationResponses` / `_HTTPMutationResponseSlot` | fallback-only transient upload response owner (ADR-021). It binds exact canonical success and fixed retry body/header candidates to one ledger, reserves then consumes their maximum response/header charges before effects, and rejects foreign ledgers without state change. The pair carries the explicit states `open`, `sealed`, `selecting_success`/`selecting_durable`, `selected_success`/`selected_durable`, `cleanup_pending`, `closed`; selection is an immutable destination-first handoff that runs nothing fallible, after which the pair owns only the rejected owner and `release_rejected` is the single fallible step, so no generic pair cleanup can reach a selected response. The slot carries `empty`, `prepared`, `sealed`, `effect_may_have_started`, `selected`, `wire_started`, `wire_complete`, `cleanup_pending`, `closed` plus a separate `not_started`/`started`/`complete` wire state, the one-way `EFFECT_MAY_HAVE_STARTED` marker, an attempt counter bounded at two per failed release boundary, and bounded cleanup-fault text. It exposes `abort_before_effect` and `finish_after_effect` as distinct operations, keeps ownership through the borrowed wire attempt, and projects a truthful observation of selection, wire state, attempts, reachable owner, exact debt, and permits. It is not persisted, adds no decision-work terminal variant, and does not cover other POST routes. |
| Python `RemoteJSONPublicationWritePreflight` / `RemoteJSONPublicationPreflight` | fallback-only single-use effect-free proofs. The layout-independent write proof binds project root and ordered write digest, resolves/deduplicates authority paths, verifies fresh preimages, and freezes bounded canonical postimage bytes/hashes before missing-project namespace effects. After bootstrap it is revalidated and consumed under the project/nonce gates. The full proof additionally binds operation, actor, idempotency key, sealed roots, semantic/result, request hash, and exact caller-owned objects before one reducer consumption. Neither proof is serialized or reusable. |
| Python `HTTPRequestDeadline` / `RequestDeadlineExceeded` | fallback-only handler-owned monotonic deadline for one read-only bundle/clone GET plus bounded upload fences. Disabled mode does not read the clock; enabled mode expires exactly at `elapsed >= limit` and polls bounded branch/object reads, raw SHA-256, canonical hash/retained-size traversal, validation, projection, response publication, and upload closure collection/reference/sort before staging. Route ownership closes before one-way `deadline_exceeded` and its fixed 503; earlier terminal or cleanup failure retains precedence. Contiguous stdlib JSON parse/materialization is not interruptible, so full deadline support remains rejected. Other upload business phases, other routes, and full upload-work polling are not implied. |
| `doctor::{DoctorReportAccountingTransfer,report::AccountedDoctorReport}` | non-persisted local Doctor report handoff and owner. The transfer binds the preaccounted repository-layout byte count to its funding ledger identity; a foreign ledger is rejected before report sizing or accounting. The owner acquires only the unowned remainder, accounts data/diagnostic/action projection and JSON/text/stdout handoff, releases payloads before accounting, and exposes only allocation-free success inspection plus consuming write or cancellation. Encoding, cancellation, drop, and stdout errors leave current memory and permits at zero; partial stdout failure is non-success, never `Complete`. Later non-layout collectors remain CFR-168 residuals. |
| `doctor::checks::{DoctorIssueCollection,AccountedDoctorPath,DoctorTransactionMarkerSpec,DoctorTransactionMarkerPresence,DoctorTransactionIssueSpec}` / `secure_worktree::StableFilePathIdentity` | private non-persisted collection and read-authority owners. The collection binds its funding ledger, accounts layout plus ordered transaction segments, and rejects foreign operations before allocation/effect. One static-spec collector owns top-level open/close marker fixed-limit bounded read/parse scratch and exact successful read counters. Open authority prepares both path owners, reads canonical then removal quarantine, counts only realized records, and does not retroactively adopt a candidate appearing after its ordered read. Primary presence classifies simultaneous authority without reconstructing marker file paths or querying metadata again; the separate report kind/message/`.codefire` path/ordered slot allocate only after exact Retained reservation. Its read token retains original parent and file identities; closing capability/ambient validation detects parent replacement even if the same marker inode moves under the public name. Final Vec allocation precedes payload movement and transferred ownership is not charged twice. Open-summary evidence work and later collectors remain residuals. |
| `doctor::branch_heads::DoctorBranchSnapshot` | private local Doctor branch authority owner binding `.codefire`, optional `branches`, sorted membership, stable entry identities, regular/symlink content evidence, extracted heads, paths, and exact Retained ownership to the Store-session ledger. Regular reads allocate only metadata size, fill that fixed Vec, then probe EOF with stack `[u8;1]`; a `size+1` permit consumes actual read bytes and hashes only a complete record. Malformed/symlink/special/oversize candidates remain ordered diagnostics without poisoning later valid heads. Finalization rechecks every candidate, membership, directory slot, and ambient parent. Opened and active collectors are outside this owner. |
| `doctor::opened_registry::DoctorOpenedRegistrySnapshot` | private, non-persisted opened-registry candidate owner. The same Store-session ledger reserves Retained parent/root capabilities, display paths, sorted recursive membership, candidate paths, entry kind/identity, regular size, raw JSON Vec/payload, parsed JSON `Value`, and collection slots before allocation. Initial and final absolute parent resolution has a separately checked Temporary owner for the root path, normal-component Vec/frames/payloads, and overlapping traversal handles; its limit failure precedes the open hook and its scalar is released on every open result. Root, nested directories, and candidate files are opened no-follow and descriptor-relative after queue/handle and physical-read reservation. A candidate must prove single-link authority before and after the fixed read through the same file handle; allocation-free typed unavailable evidence distinguishes open, metadata, identity, bounded-size, multiple-link, and unprovable-link failures for ordered diagnostics. A metadata-sized fixed destination read plus stack EOF sentinel prevents Vec growth. JSON parse jointly reserves serde Temporary scratch and a parsed-Retained upper before materialization, transfers exact compact ownership into the snapshot, and preserves any failed cleanup scalar for same-owner abort retry. The upper sealed-validation coordinator restores a taken snapshot with its detached ledger after finish/abort cleanup failure. The final fence repeats the read and compares bytes, identity, link evidence, size, and membership, closing hardlinks added before or during the final read, same-inode rewrite/growth, and ambient root/nested read/restore ABA. Entry checks borrow paths/raw slices, parsed values, or typed unavailable evidence while the owner remains attached. Abort, foreign, limit, unreadable, cancellation, release-retry, and changed-subject paths drop payloads before scalar release. Extracted field `String`/`PathBuf` values remain outside this owner. |
| `SealedCommitValidationSession::{validate_commit_with_cancellation,finalize_with_cancellation}` | caller-polled iterative validation/final revalidation. Overlay input preparation reserves one checked Temporary `usize` slot per document before allocating its contiguous duplicate index, sorts it in place without heap allocation, uses no per-ID clone or measured-size vector, and releases scratch after drop. Cancellation makes the session abort-only or finalizes it with `deadline_exceeded`; retained/temporary ownership is released before ledger detachment. |
| `AccountedObjectStoreReader` | non-serializable request-scoped exact reader. One ledger identity owns its preaccounted root path, pinned root/subdirectory set, file-name/path scratch, opened-file/name identity fences, and final ambient root/subdirectory fence across all record reads. |
| `PreparedObjectRecordBatch` / `.codefire-stage-v1.*.tmp` | non-serializable same-ledger object publication owner. It can receive overlay documents directly from the validation proof after destination/additional-Retained reservation, retaining them through final raw revalidation without a second JSON clone or plain-vector escape. A stable single-link kernel lock serializes stage writers and crash recovery. Recovery accounts and validates every reserved-name candidate before unlink, rechecks name identity at effect time, preserves unknown/future/malformed/multi-link/content-mismatched files, and may abandon only private stages after a shared seal while releasing retry ownership. |
| `BoundPrepublicationValidationSession` / `PrepublicationSealedValidationProof` | non-serializable RAII transition from one successful unfinalized Store session and private root binding to the detached proof plus its same ledger. After detachment, the proof checks the exact ledger and bound retained roots before lending a non-ledger provisional planning view; it can then move overlay documents and their existing compact-byte ownership into the same-ledger batch while retaining filesystem fingerprints for final raw-document/identity revalidation immediately before the shared seal. Ordinary cancellation returns a finished zero-current ledger. The unfinished variant releases only Store ownership, rejects terminal or non-Retained current state, preserves caller Retained/permits, and returns a retry owner on accounting failure. |
| `AccountedObjectRecords` / `rendered_object_record_document_size` | graph collection owns compact-canonical retained bytes explicitly after a `2P` raw/parse Temporary reservation. The owner is either released after digest validation or consumed by ledger-accounted escrow publication; pretty physical size is measured without allocating the publication buffer. |

Commit eligibility must derive from one `Verification`, not independent recounts in renderers.

## CLI Boundary Types

`cli_model` owns shared options/results for init/import/open/status/branch/extinguish/commit/clone/close/diff/review/patch/serve/merge/lock. Focused modules own their specialized types, including context, doctor, evidence, explain, fire/link batch, migration, remote, storage, and view algorithms. The capability support manifest's versioned `decision_work_profile` format entry serializes the Core-owned local/HTTP `DecisionWorkProfile` values directly and labels coverage as partial; automation must treat unlisted work surfaces as unclaimed rather than inferring enforcement from profile availability.

Options carry explicit path/JSON/dry-run/lock/idempotency fields where supported. Result types return domain identity plus an operation plan or data payload; rendering is separate.

`PageRequest` carries optional bounded limit and cursor. `Page` contains the selected values and
`codefire_page` version 1 metadata: collection, complete-result `sha256:` snapshot, offset, limit,
returned/total counts, and optional next cursor. Remote `list` and `request-list` expose it; an
unpaged request preserves the historical full-result behavior.

`MigrationReport` is a non-persisted command result with `mode`, requested compatibility-profile
`target_format`, repository/check counts, blockers/warnings, and separate
`planned_actions`/`applied_actions`/`skipped_actions`/`remaining_actions`. Its JSON projection includes
the corresponding counts, `would_write`, `did_write`,
`target_format_semantics=compatibility_profile_only`, and explicit false
`format_write_supported`/`format_write_performed` fields. Only shared-registry `create_directory`
actions marked `AutoCreate` can be applied. `v0.6` acceptance therefore does not claim a v0.6 writer
or a repository-format transition. Persistent `repo.lock`/owner diagnostics are operational lock state,
not migration actions counted by `did_write`.

## Object Record and Identity

Current object record:

```json
{
  "object_id": "CF-...-<24 hex>",
  "type": "<type tag>",
  "hash": "<full 64-hex SHA-256>",
  "payload": {}
}
```

The envelope may also carry non-identity top-level metadata. A standardized `extensions` array uses
`{namespace,name,version,critical,references,payload?}`. Unknown `critical=true` is rejected before
semantic use. Unknown noncritical descriptors and legacy top-level metadata are retained as JSON
values. Transfer publication requires exact extension-map equality on immutable reuse, and declared
`CF-*` references participate in graph completeness and reachability without changing the payload digest.

Digest input is `type_tag`, one NUL byte, then the canonical JSON bytes of `payload`. `read_object_record` recomputes the full hash and current/legacy ID and validates the filename/record identity. Existing-path reuse additionally compares the requested and stored type, complete digest, and canonical payload; a shared truncated ID with any mismatch is `ObjectCollision`. `validate_sealed_commit` is the stronger consuming boundary that validates commit/root shapes, Atom/Trace references, and reachable parent history.

`MAX_OBJECT_RECORD_BYTES` is 64 MiB and applies to serialization and reading. Writer and reader roots/subdirectories are opened from an absolute path component-by-component with no-follow semantics, so a symlink root or intermediate directory is rejected. `read_object_record_from_directory` opens the final name with no-follow/non-blocking options, requires a regular file, rejects an oversized metadata length, and also reads through a limit-plus-one guard so concurrent growth cannot bypass the cap. Oversize is the typed `ObjectRecordTooLarge {path,size,limit}` error. Writer publication validates/syncs a create-new temp, hard-links it no-replace, and rechecks ambient root/subdirectory identity after publication or existing-record reuse.

## Remote Project Identity and Gate

A file-backed remote project owns a bounded `remote_project.json` descriptor with `type=codefire_remote_project`, version 1, a random `CF-PROJECT-<32-hex>` stable identity, protocol generation 1, canonical organization/application components, creation provenance, and timestamp. Organization, application, branch, merge-request ID, and HTTP path components share one Rust/Python contract after strict percent decoding: empty/current/parent, absolute or prefixed, slash/backslash, colon, control, multi-component, invalid UTF-8/escape, and values above 255 UTF-8 bytes are rejected before nonce/layout/owner publication. A new project is staged below its organization and renamed under persistent namespace/target bootstrap locks. Ledger-aware preparation owns a fixed-work permit plus an optional `BootstrapStageFootprint { entries, payload_bytes }` rollback permit, rejects foreign ledgers and footprints above the 4,096-entry/16-MiB recovery cap before content effects, and exposes the hidden project only through its already-held gate. Explicit cancellation performs bounded removal; fail-safe drop only quarantines the owned stage, which the next ledger-accounted bootstrap recovery removes. The final project name appears only through one no-replace rename after all fallible identity and label preparation. A descriptor-less project containing mutable authority is preserved and rejected until an administratively quiescent explicit migration exists; a generation marker is not treated as proof that legacy writers are absent.

For HTTP upload, the generic request-body model (`RequestBodyBytes`, 16 MiB) dominates before route
dispatch. A missing-project `BootstrapStageFootprint` is then prevalidated before namespace or gate
creation; an existing project bypasses this bootstrap-only physical recovery cap. HTTP's realizable
payload footprint grows in two-byte file/rollback quanta, while the footprint value itself retains
the exact gate-level limit-1/exact/+1 contract. Parsed JSON is held by a ledger-bound
`HttpRequestBodyOwnership`; upload/apply move it with the parser routing owners into the combined
typed prepared intent, complete the Store-bound derived payloads, and release only the superseded
routing owners before Store binding.

`locks/project-gate.lock` is a persistent exactly-one-link regular inode. Rust uses `fs2` and Unix Python uses `fcntl.flock`; neither runtime unlinks it at unlock. The diagnostic `.owner.json` sidecar is atomic, non-authoritative, and bounded to 16 KiB. Mutation and GC take the project gate exclusively. File/HTTP list, clone, show/diff snapshot, bundle, merge-request list, and remote-doctor readers take it shared and reject a present bounded `pending_transaction.json` as recovery-pending. A legacy cross-project MR without target-local subject escrow cannot be listed under only the target gate; list fails closed until an exclusive ordered review materializes its subject. Cross-project request creation and legacy review collapse equal project IDs and otherwise acquire gates in stable project-ID order; Rust applies one monotonic timeout to the whole set.

Every upload, request-merge, review, and apply uses a version-1 `remote_publication_transaction`, including calls without a user idempotency key. The fixed pending marker binds operation, actor/key scope, request hash, ordered typed authorities, exact preimage/postimage hashes, sealed object roots, immutable result, bounded staged postimages, and published progress. Recovery accepts only the exact owner and either the recorded preimage or postimage at each adjacent step. A keyed completion additionally publishes a version-2 `remote_idempotency_receipt`; its hashed filename scopes actor, operation, and key, and its exact result/effect hashes replay across Rust/Python and file/HTTP transports. Completed pending or transaction-directory quarantine is removed only after exact schema, hash, authority uniqueness, owner, result/effect, receipt when keyed, and current postimage validation.

Rust `PreparedJsonPublication` is a non-persisted, ledger-bound publication owner. It stores an
opaque `DecisionWorkLedgerIdentity`, an optional reducer plan, an optional publication permit, exact
reserved work, the concrete optional `ValidatedPublicationRoots` token behind that retained charge,
and pre-seal revalidation state. Foreign-ledger operations are
rejected before reads or accounting; failed preconditions preserve the plan, and successful cancel
or commit drops the reducer/token payload before releasing its retained charge and leaves the owner
inert before drop. This type does not replace the persisted transaction
or receipt formats above.
The production provisional-planning variant leaves that optional legacy token empty and borrows the
unfinalized Store subject only during construction; its request-wide owner retains the detached
`PrepublicationSealedValidationProof` instead, avoiding a duplicate owned root list.

A version-1 `codefire_merge_request_subject` binds the MR ID, stable source/target project IDs, branch identities, immutable `source_head`, `target_head_at_request`, and canonical source object-set digest. Reviews bind its digest. Cross-project creation copies that graph into target-local escrow before MR publication; apply validates and publishes only the reviewed escrow head, even if the source branch advances or disappears.

`codefire.support.v1` is generated from the Rust compatibility registry and advertised inside
`codefire.capabilities.v1`. It lists readable/current versions for object, commit/root, remote
descriptor/branch/MR/publication/receipt, command envelope, capabilities, and remote Doctor formats;
missing/future behavior is explicit. `codefire.remote_doctor.v1` reports target/transport/profile,
layout, bounded issue samples, object/branch/MR/reference counters, skipped quick checks, and outcome.
The checked checksum manifest executes the same accept/deny vectors in Rust, Python fallback, and a
stdlib-only independent consumer.

Python GC owns a version-1 `remote_gc_transaction` under the same exclusive gate. Its bounded stage contains typed roots, inventory chunks, policy/root/inventory/plan fingerprints, retention pre/postimages, exact candidate identities, immutable audit, and ordered delete progress. `planned_at_ns` must be a nonnegative integer no later than the trusted resume wall clock before it may influence age retention. Candidate deletion first moves the exact record to a plan-owned quarantine, reopens and rechecks identity/hash, then unlinks while compliant writers remain excluded by the project gate; a quarantine interruption is resumable. The exact `gc/GC-*.json` record is audit authority, while `audit/gc.jsonl` is a deterministically rebuilt derived view. `retention_index.json` is separately versioned, bounded, and generation-monotonic. Unknown/future pending, root, object, policy, or retention versions are preserved and block collection.

New payload construction uses `store_object` and current 24-hex IDs. Remote/migration transfer uses
`store_object_record`: the supplied `ObjectRecord` must independently validate against its full
digest and a supported 12- or 24-hex public ID, after which the exact ID is published. Existing-path
and link-race reuse still require type, full digest, and canonical payload equality; exact-record
publication is not a weaker raw-file copy primitive. The Python fallback follows the same rule:
new construction emits 24 hex, while complete transferred records may retain an authenticated
legacy 12-hex or current 24-hex ID.

`ObjectStoreSubject` contains the canonical root path, stable root identity, and an ordered set of the twelve built-in subdirectory states plus at most 128 currently present extension subdirectories, each with identity or explicit absence. A 129th extension directory is `InvalidSealedCommit` before any new handles enter the cache. `BoundObjectStore` retains root/subdirectory capabilities, and record files are opened relative to those handles with no-follow semantics. `SealedCommitValidationCache` keys a successful commit by that full subject plus commit ID, stages entries until final root/child identity rechecks, and retains only the current store subject; binding a different root/identity/subdirectory set evicts prior pins and validation entries. Cancellation during staged-subject promotion invalidates every partial promotion and marks the session failed, so no finalized payload can be borrowed. `ValidatedSealedCommit` is the non-serializable consuming token returned by `validate_sealed_commit_pinned[_with_cache]`; it owns clones of that exact bound handle set, exposes commit/root/reachable-object reads with identity checks before and after, and can recheck the same subject immediately before and after mutable publication. The full object scan uses the same bound subject and final identity recheck. Stable identity unavailable means no reuse across calls and no token publication proof. The cache/token rely on the immutable object-file contract and do not detect an in-place rewrite of an already validated file. See ADR-011.

## Object Kinds

| Type tag | Prefix | Subdirectory | Role |
|---|---|---|---|
| `blob` | `CF-BLOB` | `blobs` | file content. <!-- object-kind: blob --> |
| `content_manifest` | `CF-MANIFEST` | `content_manifests` | path → blob/metadata sealed snapshot. <!-- object-kind: content_manifest --> |
| `atom_index` | `CF-ATOMINDEX` | `atom_indexes` | serialized Atom index. <!-- object-kind: atom_index --> |
| `trace_graph` | `CF-TRACE` | `trace_graphs` | serialized Trace Graph. <!-- object-kind: trace_graph --> |
| `fire_ledger` | `CF-FIRELEDGER` | `fire_ledgers` | sealed Fire set/delta. <!-- object-kind: fire_ledger --> |
| `resolution_ledger` | `CF-RESOLUTION` | `resolution_ledgers` | sealed resolutions and evidence refs. <!-- object-kind: resolution_ledger --> |
| `verification` | `CF-VERIFY` | `verifications` | core verification result. <!-- object-kind: verification --> |
| `policy` | `CF-POLICY` | `policies` | policy used for the seal. <!-- object-kind: policy --> |
| `artifact_ref` | `CF-ARTIFACT` | `artifact_refs` | external/local artifact identity and metadata. <!-- object-kind: artifact_ref --> |
| `evidence` | `CF-EVIDENCE` | `evidence` | captured/referenced evidence subject. <!-- object-kind: evidence --> |
| `commit` | `CF-COMMIT` | `commits` | parent/root/certificate/history object. <!-- object-kind: commit --> |
| `branch` | `CF-BRANCH` | `branches` | object-store branch payload kind; repository branch heads also have active metadata records. <!-- object-kind: branch --> |

## Verification and Committable Decision

Persisted verification remains version 1 with raw fact collections and string `result`. CFR-162 adds
optional-empty `unresolved_merge_conflicts` entries with `kind=text|binary|semantic` and `subject`;
historical omission means empty. CFR-164 adds optional `codefire_assessment_subject` version 1 and
optional-empty `VerificationExecutionRecord` entries; historical omission remains readable. A live subject binds
repository/branch/open instance/base, scan content, and canonical worktree manifest under `local_unisolated`. Execution binds
`trusted_local_shell_v1` profile, `verification_command` phase, subject/input/environment/worktree digests,
repo-relative cwd, limits/counters, exit/timeout/termination scope, and redaction count. Core derives:

```text
CommittableDecision {
  outcome: passed | failed,
  primary_reason: optional CommittableReason,
  blocking_reasons: ordered reason classes,
  warning_reasons: ordered reason classes
}
```

Reason order is unresolved merge conflicts, open required Fires, required missing links, stale Resolutions, missing evidence,
duplicate Atom IDs, failed required checks. The four version-1 policy fields
`require_no_required_fires`, `require_no_stale_resolutions`, `reject_duplicate_atom_ids`, and
`require_verification_success` remain serialized for compatibility but Rust working config accepts
only `true`; programmatic/deserialized false cannot relax facts. `require_trace_completeness=false`
keeps missing-link facts as warnings. Merge conflicts have no policy relaxation and map to exit 15.
Verify automation adds `data.decision` while retaining `data.result` as the outcome projection.
ADR-013 defines historical clean-policy readability; ADR-014 defines merge resolution.

## Current Commit Payload

Current production `run_commit` writes:

```json
{
  "type": "commit",
  "version": 1,
  "root_schema_version": 2,
  "semantic_validation_profile": "sealed-snapshot-v1",
  "parents": ["CF-COMMIT-..."],
  "message": "...",
  "roots": {
    "content_manifest": "CF-MANIFEST-...",
    "atom_index": "CF-ATOMINDEX-...",
    "trace_graph": "CF-TRACE-...",
    "fire_delta": "CF-FIRELEDGER-...",
    "resolution_ledger": "CF-RESOLUTION-...",
    "verification": "CF-VERIFY-...",
    "policy": "CF-POLICY-..."
  },
  "certificate": {
    "result": "passed",
    "profile": "local_verification_v1",
    "phase": "pre_commit",
    "subject": {"type": "codefire_assessment_subject", "version": 1, "subject_digest": "sha256:...", "worktree_digest": "sha256:..."},
    "open_required_fires": 0,
    "failed_checks": 0,
    "missing_required_links": 0,
    "stale_resolutions": 0,
    "missing_evidence_refs": 0,
    "duplicate_atom_ids": 0,
    "effective_policy_digest": "<sha256 hex>",
    "change_coverage_digest": "<sha256 hex>"
  },
  "changed_atoms": [],
  "extinguished_fires": [],
  "created_at": "...",
  "signature": {
    "type": "commit_signature",
    "version": 1,
    "algorithm": "hmac-sha256",
    "signer": "...",
    "key_id": "...",
    "signed_at": "...",
    "signature": "..."
  }
}
```

`semantic_validation_profile` is independent of `root_schema_version`. Current Rust writers emit
`sealed-snapshot-v1`; a missing field is an explicitly weaker `legacy-unversioned` read contract,
and unknown/non-string values fail closed. Each commit in a parent chain is interpreted under its
own profile, so a strict bridge commit can retain legacy parents without rewriting their object IDs.
ADR-012 owns this boundary.

The store's sealed-commit validator checks required roots, root object types/shapes, Atom/Trace endpoints, reachable references, and parent history. For root schema 2 it decodes Atom/Trace/Fire/Resolution/verification/policy into core types, calls `validate_sealed_snapshot_semantics_with_contract`, then matches certificate result/counts and ADR-019 policy/coverage digests to verification. That predicate uses ADR-013's safety-floor decision, so false policy fields cannot hide blocker facts; ADR-014's non-empty merge facts also force failed and cannot be sealed. When present, effective-policy profile/order/IDs/values/digest must match the raw policy and both roots; coverage paths/Atom facts/classification/summary/digest must be self-consistent. Historical records omitting both contracts remain readable and do not gain those claims. A one-sided root or certificate claim is invalid. Missing schema retains legacy schema-1 readability. Unknown root schema, commit version other than 1, and root object version other than 1 fail closed. Schema 2 permits `missing_evidence_refs` omission only for an effective empty list/count and `unresolved_merge_conflicts` omission only as an empty list; the latter has no certificate count because a writer may seal only zero. Commit/request signature policy is enforced separately by the CLI `signatures` module at local commit and remote/security boundaries. The `signature` member is optional and, when present, is an object rather than a string. When policy requires a local commit signature, the writer obtains the key after creating `commit_transaction.json` but before planning/storing any immutable commit role. After sealed candidate validation, the CLI rereads the candidate through the retained token and verifies the present/required HMAC with that same trusted key before branch-snapshot or reference publication; a malformed/tampered signature leaves the transaction at `validating_candidate_commit`.

When verification carries an assessment subject, sealed validation checks its type/version, exact live/sealed
variant fields, digest labels, and absence of unbound fields, recomputes the subject digest from identity fields,
and requires the certificate subject to be exactly equal. A live subject requires `worktree_digest`; a sealed pair
omits it.
Historical verification/certificate pairs with neither subject remain readable; one-sided, mismatched, or tampered
subjects fail closed. Request signature version 2 is a separate CLI boundary that binds the full canonical payload
digest plus actor/operation/target/key/idempotency/time/nonce. Version 1 is not interpreted as payload
authentication. ADR-016 owns both trust-subject extensions.

## Repository Layout

```text
.codefire/
  repo.json                  # optional import initialization provenance
  open_transaction.json     # only while open publication/recovery is pending
  open_transaction.removing.json # only during identity-bound marker retirement
  close_transaction.json    # only while close/discard publication is pending
  close_transaction.completed.json  # exact last-completed close authority
  objects/<object-kind-subdir>/<object-id>.json
  branches/<encoded-branch>.json
  opened/<encoded-branch>.json
  active/<open-instance>/
    state.json
    scan.json
    fires.json
    resolutions.json
    verification.json
    active_ledger_transaction.json # pending staged Fire/Resolution postimages or latest compact receipt
    state_transaction.json    # only while scan/verify/Fire/Resolution state publication is pending
    commit_transaction.json   # only while interrupted/in progress
    open_transaction.completed.<transaction-suffix>.json # immutable completion archive
  cache/
  locks/
    repo.lock                # persistent content-opaque inode; ownership is its kernel lock
    repo.lock.owner.json     # atomically replaced best-effort pid/timestamp diagnostics
  remotes/
  idempotency/
```

An imported project root additionally contains `.codefire-import.lock` (immutable bootstrap provenance and stable advisory-lock inode) and `.codefire-import-transaction.json` (atomically replaced phase/result state). Its `.codefire/locks/repo.lock` follows the same content-immutable/exactly-one-link rule as an ordinary repository lock and publishes diagnostics to `.codefire/locks/repo.lock.owner.json` only after ownership is acquired. Atom and content-manifest scans exclude these exact root names plus `.codefire-open`, root bootstrap candidates `.codefire-import.lock.bootstrap-*.tmp`, unpublished staging `.codefire-import.lock.bootstrap-*.part`, and root atomic-write remnants `.codefire-import.write-<transaction-id>-*.tmp`. The namespaces are reserved only at the imported root; similarly named non-matching and nested user files remain tracked.

Bootstrap publication writes/syncs a random `.part`, hard-links it to the deterministic fully formed `.tmp`, removes its staging name, then hard-links the `.tmp` to the canonical lock. Discovery never parses `.part` as authority. Recovery preserves unlinked/stalled `.part` scratch, removes it only when file identity proves it is linked to the canonical lock, can resume a valid fully formed `.tmp`, preserves/rejects malformed published `.tmp`, and never unlinks another fully formed contender solely because it lost the canonical-link race. Import atomic replacement removes only write `.tmp` files bound to the canonical transaction ID; foreign IDs are preserved and fail closed.

`repo_layout::REPO_DIRS` distinguishes required from auto-created directories. Object subdirectory requirements come from the store registry.

During `open`, the target parent can temporarily contain the following deterministic sibling names:

```text
.codefire-open-stage-<48-bit-operation-hash>/
.codefire-open-stage-<48-bit-operation-hash>.owner.json
```

The owner record is capped at 16 KiB and binds `type=codefire_open_staging_owner`, version 1, repository path and repository ID, branch, target, and sealed base commit. It is published no-replace and read as a bounded regular no-follow file. Exact matching provenance permits cleanup/rebuild of an abandoned stage; a stage without the owner or a mismatched owner is foreign and is preserved/rejected. The stage already contains the final `.codefire-open` marker before one sibling rename publishes it as the target. Normal failure removes the unpublished stage and owner. These sibling names are not repository/opened authority. The repository transaction separately records the initial boundary/existing-ancestor identity chain and the actual staged publication-parent identity after CodeFire creates any missing suffix; recovery must match both before cleaning or rebuilding a stage.

Close/discard derives every quarantine from its validated `close_tx_<32 hex>` identity: `.codefire-close-marker-<suffix>` inside a kept worktree, `.codefire-close-worktree-<suffix>` beside a removed worktree, `.codefire-active-remove-<suffix>` beside active state, `.codefire-close-registry-<suffix>` beside the opened record, and `.codefire-close-transaction-finish-<suffix>` beside the pending marker. These names are not independent authority: the version-1 pending/completed record binds each single component to the exact target and parent filesystem identities. A retry resumes `original -> quarantine -> removed` and never guesses from a name or blindly restores a partially deleted tree. Quarantines and the fixed completed archive must not be manually treated as stale temp files.

## Open Marker and Registry

Current `.codefire-open` writer shape:

```json
{
  "version": 1,
  "repository": {"path": "/repo/.codefire", "repository_id": "..."},
  "branch": {"name": "feature", "opened_from_commit": "CF-COMMIT-..."},
  "open": {"open_instance_id": "open_...", "opened_path": "/work", "opened_at": "..."}
}
```

Current repository-side opened record:

```json
{
  "version": 1,
  "branch": {"name": "feature"},
  "open": {
    "open_instance_id": "open_...",
    "path": "/work",
    "opened_from_commit": "CF-COMMIT-...",
    "current_base_commit": "CF-COMMIT-...",
    "active_state_path": "/repo/.codefire/active/open_..."
  },
  "state": {"last_known": "open-clean"}
}
```

`active/.../state.json` is currently `{"state":"open-clean|open-burning|open-consistent"}` with no `type` or `version`. Consumers must not invent fields from old documentation.

## Verified Open Snapshot and Recovery Classification

`open_snapshot::VerifiedOpenSnapshot` is a source-level proof carried as `OpenContextAuthority::Verified`; it is not another persisted JSON file. It binds the canonical repository path and ID, marker/opened/branch names, opened path, open-instance ID, original-open commit, exact active-state path, published branch head/current opened base, and branch/opened/active state. The exact historical Python status-only shape is represented separately as `OpenContextAuthority::LegacyPythonStatus` and cannot authorize mutation.

The validator returns `OpenSnapshotDivergence { phase, authority, expected, actual, expected_record, actual_record, repo_root, branch }`. CLI and doctor JSON preserve those fields under `kind=open_snapshot_divergence`; the safe action is read-only inspection rather than an inferred repair. A valid subject-bound transaction or unreleased validated ledger descriptor yields `OpenSnapshotDisposition::PendingCommit { phase }`, `PendingState { operation, phase }`, or `PendingActiveLedger { operation, phase, request_hash }`. A simultaneous state/ledger pair is accepted only for the same operation/request hash and a publication status allowed at that state phase. Ordinary consumers surface `open_recovery_pending` with operation/phase/repository/branch/open directory. Only a lock-owning exact operation may consume that disposition; malformed, foreign, conflicting, or impossible pending evidence is divergence.

The branch-new/opened-old tuple is accepted only in the recorded commit publication window: the pending transaction must match branch/open directory, its candidate and every role must be stored, the candidate must validate as sealed with the recorded parents, and its phase/current branch/opened tuple must be one of the explicit publication/reset/idempotency states. State-transition windows similarly require version-2 subject and pre-state fields plus a state tuple permitted at the recorded operation/phase. These rules are the persisted-authority contract selected by [ADR-015](../../adr/ADR-015-verified-open-authority-and-active-transitions.md) and tracked by [CFR-163](code-review-issues/cfr-163.md).

## Open Transaction Marker

The version-1 repository-wide `.codefire/open_transaction.json` is a bounded pending authority for one open request. It records a random `open_tx_<32-hex>` transaction ID; repository path/ID; exact request payload/hash/idempotency key; branch preimage/postimage and branch-object ID; target/workspace/staging/open-marker data; initial target-resolution identities and the actual staged publication-parent identity; expected opened-registry and active-state records; optional idempotency record; and stable result/plan. Initial creation uses atomic no-replace publication, so a foreign racer is preserved rather than overwritten.

Adjacent phases are `prepared`, `target_published`, `registry_written`, `active_state_written`, `branch_written`, and `idempotency_written`. Retry classifies every artifact against the recorded preimage/postimage. The branch object may exist while its mutable reference is still at the preimage; the inverse (a published reference without its object) is corruption. A target is exact only after its marker and sealed content manifest are verified through a retained, bounded no-follow worktree capability. Symlink, non-regular, and non-UTF-8 entries fail closed, and every file/directory name is rebound after use. The same target capability and sealed-store token are rechecked around later visibility points. Dry-run reports pending authority without reconciling it.

Completion adds `completed_at`, writes `active/<open-instance>/open_transaction.completed.<transaction-suffix>.json` with exact-or-create semantics, then identity-quarantines the pending marker as `.codefire/open_transaction.removing.json` before unlink. A divergent existing archive, simultaneous canonical/quarantine markers, or marker identity replacement fails closed. A quarantined marker is valid only at `idempotency_written`. Crash recovery is driven by rerunning the exact live request; doctor verifies target materialization and reports valid pending evidence or `invalid_open_transaction`.

## Close Transaction Marker

`.codefire/close_transaction.json` is a version-1 `codefire_close_transaction` record with `status=pending`, a `close_tx_...` transaction ID, phase, repository identity, exact close/discard/keep-dir request, branch before/after/object identity, worktree marker and identities, active-state and opened-registry preimage identities, deterministic quarantine names, result, and timestamps. Valid phases are `prepared`, `worktree_applied`, `active_state_removed`, `registry_removed`, and `branch_written`. The branch postimage differs from its supported open preimage only by `state=closed`; only `open-clean`, `open-burning`, and `open-consistent` are valid preimages.

Success writes the same record as `status=completed` to fixed `.codefire/close_transaction.completed.json`, adds `completed_at` plus the pending marker/parent/quarantine identities, and only then removes the pending marker. Exact archive reuse makes a crash between archive publication and marker removal retryable. A later different close first moves/removes the old archive through `.codefire-close-transaction-completed-retiring`; simultaneous primary and retiring archives are corruption. Neither a dry-run nor another mutation may consume or replace pending authority.

## Commit Transaction Marker

While committing, `active/.../commit_transaction.json` contains version, transaction ID, `pending` status, current phase, branch/open directory, parents, message, timestamps, an ordered `objects` role ledger, a candidate alias, and version-1 `active_reset` authority. A required-signature key failure occurs before planning/storage and leaves `phase=started`, an empty objects list, and no candidate field, while its already-recorded reset preimages remain available for later validation.

Current version-1 object entries are:

```json
{
  "role": "manifest_blob|content_manifest|atom_index|trace_graph|fire_ledger|resolution_ledger|verification|policy|candidate_commit|branch_snapshot",
  "path": "repo-relative manifest path or null",
  "object_type": "blob|content_manifest|atom_index|trace_graph|fire_ledger|resolution_ledger|verification|policy|commit|branch",
  "object_id": "CF-...",
  "state": "planned|stored"
}
```

Every manifest-blob path entry, seven sealed roots, candidate, and branch snapshot is published as `planned` before the first store. Equal blob content may repeat an object ID but each path keeps an independent role entry/state. After each exact-ID store, only that entry changes to `stored`; the candidate alias changes with its candidate entry. The candidate is validated before branch-snapshot storage, and the mutable branch ref is published only after that snapshot is stored. Success removes the marker. Doctor keeps stored/planned role-entry counts in the human message and projects typed `authority=commit_transaction`, `operation=commit`, phase, and a bounded `transaction_id` request identity. It does not copy message, request payload, operation plan, or idempotency key out of the marker. Legacy string entries are accepted as stored for diagnosis; an unsafe/oversized legacy identifier is represented as a withheld identity rather than echoed.

Phase vocabulary follows the role groups: `objects_planned`; repeated `storing_manifest_blobs` / `stored_manifest_blob`; repeated `storing_commit_roots` / the existing `stored_<root-role>` phases; `storing_candidate_commit` / `validating_candidate_commit`; `storing_branch_snapshot` / `stored_branch_snapshot`; `publishing_branch`; and `resetting_active_state`. A store-gap failure intentionally leaves the phase in `storing_*` and its object entry in `planned`, even though the exact planned ID may already exist in the immutable store.

`active_reset.removals` is an ordered fixed list for `fires.json`, `resolutions.json`, `scan.json`, `verification.json`, `active_ledger_transaction.json`, and `state_transaction.json`. Each entry stores `{path, preimage, progress}`; a present preimage is `{present:true, byte_count, sha256}` and an absent preimage is exactly `{present:false}`. Progress must be one `removed` prefix followed by `planned` entries. `active_reset.state_file` stores the same exact preimage evidence, an authenticated canonical open-clean postimage, and `progress=planned|written`. The reset reducer preflights every entry before writing, accepts only exact recorded bytes/missing state or the immediate apply-before-progress gap, records one progress step per atomic update, and rejects a removed file that reappears. Completion requires every removal plus the exact state postimage. A pre-authority legacy marker can auto-upgrade only before reset begins; legacy partial reset is preserved for manual inspection.

`active/.../state_transaction.json` is a pending marker for live scan/verify and Fire/Resolution state publication. The current writer emits version 2 with status, operation, current phase, intended state, timestamps, `subject={branch,open_instance_id,current_base_commit}`, and `pre_state={branch,registry,active}`. All pre-state values must agree and use one supported open state. Version 1 remains shape-readable for legacy diagnosis, but it lacks the subject/pre-state proof required for a recoverable `VerifiedOpenSnapshot` pending disposition.

Operation phases are:

- scan: `started -> verification_invalidated -> fires_written -> scan_written -> registry_written -> active_state_written -> branch_written`;
- verify: `started -> verification_written -> registry_written -> active_state_written -> branch_written`;
- fire, fire-batch, extinguish, extinguish-batch: `started -> registry_written -> active_state_written -> branch_written`.

The writer enforces adjacent transitions. Exact retry requires the same version-2 operation, intended state, branch/open instance/base subject, and agreeing recorded pre-state before its first write. `VerifiedOpenSnapshot` accepts only the preimage, documented write-before-phase tuple, or completed intended-state tuple for the current phase; cross-operation, foreign subject, or impossible tuples are divergence. Scan retry recomputes derived files and invalidates superseded verification rather than trusting partial output. Completion rechecks persisted Fire/scan/verification authority as applicable and requires registry/active/branch state agreement before marker removal. Doctor exposes marker path plus typed state-transaction authority, operation, phase, intended state, and either the validated active-ledger `request_hash`, the bounded version-2 branch/open-instance/base subject, or an explicit legacy-unbound/withheld identity. Its next action executes only read-only doctor inspection; the typed target identifies the owning command/mode and exact-request-only contract and forbids automatic repair. Ordinary non-owning consumers return typed `open_recovery_pending` without writes.

## Active Fire/Resolution Ledger Transaction

`active/.../active_ledger_transaction.json` is version 1 with `type=codefire_active_ledger_transaction`. A pending record contains `status=pending`, operation (`fire`, `fire-batch`, `extinguish`, or `extinguish-batch`), `phase=prepared|publishing`, `published_count`, canonical request/result hashes, the durable command result, timestamps, and one or two ordered writes. Only `fires.json` and `resolutions.json` are legal, duplicate paths are rejected, and identical pre/post hashes are invalid. Each write records path, preimage presence/hash, postimage hash, and the staged JSON postimage. Individual ledger files retain the 64 MiB repository JSON cap; the combined journal is bounded to three such budgets plus 1 MiB overhead and is read/written through `ActiveStateDir` without following symlinks.

Recovery first validates the whole marker and all current images without writing. Entries before `published_count` must equal their postimage; the current entry may equal its preimage or postimage to model one atomic write-before-progress gap; later entries must equal preimages. It then rolls forward and atomically advances progress. Ledger completion replaces the staged record with `status=complete`, `phase=complete`, request/result hashes, durable result, effect hashes, `state_publication=pending`, and `completed_at`. Exact replay verifies every current effect before returning the original result. The matching state transaction alone changes publication to `released` and adds `state_publication_completed_at`; only then may a different request replace the receipt after validating its preimages. A different request while either ledger or state publication is pending is an idempotency conflict, and any unexpected hash is preserved as repository divergence. `validate_active_ledger_descriptor` validates pending recovery images or complete effects and returns only operation, phase, request hash, and typed publication status for open snapshot, close, and doctor. Doctor emits a non-repairable typed pending authority for `LedgerPending`/`StatePublicationPending`, exposes no durable result or postimages, and offers only read-only inspection plus the exact owner contract; `Released` is healthy and produces no pending issue. Fire/Resolution dry-runs and commit use the same read-only pending guard; commit later removes a validated released receipt through `active_reset`. Active-state Doctor invariants count only validated Fire entries with `status=open`, so retained extinguished history does not contradict `open-clean`.

## Import Transaction and Idempotency

The version-1 import transaction binds `transaction_id`, normalized target/scope, branch, payload version/hash, initial-file count, deterministic open identity, phase, optional base commit, and optional result. Phase values are `planned`, `initialized`, `opened`, `scanned`, and `completed`. `.codefire-import.lock` keeps the planned immutable seed on a persistent advisory-lock inode; mutable phase/result lives only in the transaction file so atomic replacement never changes the locked inode. `repo.json` is published with exact version, repository ID, transaction timestamp, and initialization provenance; initialization repeats the immutable transaction/target/branch/payload/open identity used to distinguish recovery from an unrelated repository. Import also creates/locks the persistent repository lock before making `repo.json` visible, requires exactly one provable filesystem link, verifies that the pathname still names the held file, leaves its bytes untouched, and writes owner diagnostics to the separate common sidecar. Staged initialization publishes the sealed object graph before the mutable branch reference, so no visible initial head can point to a partially published graph and retries can fill a pre-branch interruption safely.

Import control and parsed operational JSON use a 1 MiB no-follow/non-blocking regular-file read limit with both metadata-size and limit-plus-one checks. Repository byte-copy reads use 64 MiB. Ordinary imported user files use a 49,545,216-byte (47.25 MiB) limit, reserving 1 MiB of the 64 MiB object-record budget and accounting for base64 expansion. These are product limits, not merely test fixture sizes.

Import idempotency records are version 2 and hash-only: they store record type/version, operation, scope, key, payload version/hash/mode, and the original result. They intentionally do not persist the raw canonical payload. Replay additionally validates repository ID/initialization provenance, the sealed recorded base, marker/opened/branch agreement, and the current sealed head; normal post-import sealed advancement does not invalidate the original result. Version-1 result-only markers are not treated as bound replays and fail closed.

## Automation Envelope

Every Rust command with declared JSON support uses this top-level shape:

```json
{
  "schema": "codefire.command_result.v1",
  "command": "scan",
  "ok": true,
  "exit_code": 0,
  "repo": "/repo-or-null",
  "data": {"type": "payload-specific", "metrics": "optional under data"},
  "diagnostics": [],
  "next_actions": []
}
```

There is no top-level payload `type`. `next_actions` are normalized and bounded; command-specific payloads/plans live under `data`.

Stage 1 contextual failures preserve their subject: `OpenTargetResolution` renders `open_target_resolution_failed` with reason/repo/branch/requested/resolved/boundary fields; `CoreContext` renders endpoint failures as `trace_graph_endpoint_integrity` with repo/path and bounded total/omitted issues; `StoreContext` renders `sealed_commit_invalid`, `object_not_found`, or full-identity `object_id_collision` with repo/path when known. Link-batch dry-run data separately exposes `diagnostic_total` and `diagnostics_omitted` beside its bounded detail array. Its plan uses full `item_count`, at most 100 projected `items`, and `items_omitted`; live success uses full `added_link_count`, at most 100 projected `added_links`, and `added_links_omitted`. Projected from/to/type strings reuse the 256-byte UTF-8-safe truncation fields.

## Pinned Authority Capture and Fence

A compound read route that pins a JSON authority file captures, per file, the pinned parent-directory
identity, the exact descriptor identity (`st_dev`/`st_ino`), physical size, link count, mode, and a
SHA-256 digest of the owned bytes, or records the file as absent together with that same pinned parent
identity. The capture is transient request state, never persisted, and is charged as retained ownership
before it is filled. An absence is recorded only through a proved, no-follow parent directory: a
missing, replaced, symlinked, denied, unreadable, or metadata-failing parent is a typed
filesystem-authority failure instead.

The final fence reproves every capture from a fresh no-follow read: parent identity, descriptor
identity, regular-file type, mode, link count, size, byte content, and — for an absent capture — the
same parent plus continued absence, so an unreadable parent is never accepted as evidence of absence.
Every captured field is reproved again after the final byte read, after the EOF probe, and after the
interruptible content digest, and the parent identity is reproved once more before the digest is
compared, so no mode or link-count change between the last observation and publication can escape.
`read_json_accounted` binds the same protocol at capture time: its terminal `fstat` after the digest
must still show every field the capture records. Because the digest is part of the capture, a
replacement that preserves path, inode, size, and mtime is still rejected. Fence work is charged as
`snapshot_revalidation_records` / `snapshot_revalidation_bytes` alongside the sealed object store's own
revalidation, so one request's fence total is objects plus consumed authority files.

One read route's prepared response is transient request state with an explicit publication state, never
a persisted field. `_HTTPReadResponseSlot.state` is one of `empty`, `prepared`, `closing`, `proved`,
`published`, or `disposed`. Only the publishable proof moves `prepared → proved`, only `proved` may move
to `published`, and any other transition out of `prepared` is `disposed`, which releases the frozen bytes
exactly once. The state is production-consumed: the owner transfer refuses to yield bytes that are not
`proved`, so the proof cannot be an inert observation. A recovered-cleanup diagnostic carried alongside it
is bounded to the first four faults plus an exact `fault_count`.

Link count is captured and reproved, not constrained on arrival. A stable pre-existing hardlink on a
merge-request or branch authority record remains compatible authority and is published normally; only a
change to the captured count during the request fails closed as `link_count_changed`. The project
descriptor keeps the legacy baseline's own exactly-one-link rule in `_read_regular_bounded_at`, which
has applied to every bounded authority file since `7327e2a`; the accounted route adds no second,
incidental link-count rejection of its own.

The merge-request list route additionally retains, per pinned record, its deferred read/parse/version
outcome. That outcome is request state, not a persisted field: it is raised in filename order at the
point the pre-fix per-record route would have observed it, so the observable first error is unchanged
while the snapshot stays pinned.

## Compatibility Rules

- Missing/legacy Atom hash schema defaults only where source explicitly defines it. Rust scan/commit emits 4; Python fallback currently omits it. Schema 4 adds string/comment-aware Python lexical masking so declaration-like fixture text is not identity-producing source.
- Legacy 12-hex object IDs are accepted only when the record's full hash and payload recompute correctly.
- Remote copy/receive preserves that validated 12-hex public identity exactly; only newly constructed objects advance to the 24-hex writer.
- Rust current commits emit root schema 2 and semantic profile `sealed-snapshot-v1`; missing root schema is the explicit legacy schema-1 path, while missing semantic profile is the separate legacy-unversioned cross-root path. Unknown profile, schema values other than 1/2, and commit/root object versions other than 1 are rejected rather than interpreted as current.
- In schema 2, omitted `verification.missing_evidence_refs` means an empty list; the matching certificate count may be omitted only when that effective list is empty.
- Omitted `verification.unresolved_merge_conflicts` means an empty list. Non-empty is always a failed decision and has no certificate-count extension.
- Stage 2 required rule: an unknown required schema or capability must fail explicitly and must never downgrade a blocker to success.
- Commit/root-schema enforcement now follows that rule; other persisted/public schema families still require their own explicit compatibility fixtures before the rule can be claimed globally.
- Rust/Python behavior and writable/readable combinations require real fixtures; see release/format roadmap.
- A source change to any marker in this file requires source-doc checker and focused product tests.
