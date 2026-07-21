# CodeFire Source Architecture

Status: canonical

Owner: Engineering Steward

Update trigger: crate/module ownership, command execution boundary, persistence layout, object kind, or a cross-cutting invariant changes.

This document explains stable source ownership and data flow using files and symbols, not line numbers. Source and tests remain authoritative for exact implementation. Run `python3 docs/development/tools/check_source_docs.py` after changing modules, command registration, or object kinds.

## Crate Boundaries

| Crate | Owns | Must not own |
|---|---|---|
| `codefire-core` | Atom extraction, Trace Graph and required-link policy, effective-policy provenance, change coverage, decision-work profiles/atomic ledger, Fires, resolutions, scan and verification domain values | filesystem repository layout, CLI rendering, remote transport |
| `codefire-store` | canonical bytes, full digest/object ID, immutable object records, object lookup, sealed-commit validation | open-workflow state, command parsing, policy UX |
| `codefire-cli` | parse/validate/plan/apply/render orchestration, repository/open state, automation JSON, remote/HTTP, recovery tools | alternate object identity or duplicated core verification rules |
| `codefire-util` | small shared primitives such as SHA-256 | domain policy and command workflows |

Dependency direction is CLI → core/store/util and core/store → util. Core and store do not depend on CLI.

## Five Connected Machines

1. **Workspace machine**: working files → `AtomIndex` → `TraceGraph` → changed Atoms/Fires → resolutions → verification.
2. **Sealing machine**: verified workspace → predeclared immutable blob/root/candidate/branch-snapshot roles → complete candidate validation → mutable branch-head publish.
3. **Repository machine**: branch/opened/active/lock/idempotency metadata → lifecycle state and recovery diagnostics.
4. **Remote machine**: URL/auth/signature/policy → object graph transfer → branch or merge-request mutation → audit/result.
5. **Automation machine**: parser → validated options → operation plan → apply → text or `codefire.command_result.v1` envelope.

No machine may infer a stronger claim than its inputs. In particular, an automation envelope renders the domain result; it does not decide that result.

## CLI Module Ownership

`main.rs` owns top-level dispatch and workflows that have not yet moved behind focused modules. New responsibilities belong in a module unless coupling to the orchestration transaction is essential.

| Module | Stable responsibility |
|---|---|
| `active_state` | pinned active-instance capability, bounded no-follow relative file I/O, identity rechecks, and deterministic transactional tree removal. | <!-- module: active_state -->
| `assessment_subject` | canonical live-open and sealed-pair assessment identities shared by context, explain, verification, review, and certificates. | <!-- module: assessment_subject -->
| `automation` | `codefire.command_result.v1`, diagnostics, bounded `next_actions`, state/scan/verification JSON. | <!-- module: automation -->
| `batch` | extinguish batch parsing, validation, planning, rendering, and shared active-ledger reducer integration. | <!-- module: batch -->
| `branch_refs` | pinned `.codefire`/branch/opened directory capabilities, membership/document capture, retained-byte ownership, and final subject revalidation for local branch listing. | <!-- module: branch_refs -->
| `cli_model` | shared command option/result types and the `OpenContextAuthority` proof carrier; no filesystem behavior. | <!-- module: cli_model -->
| `close_transaction` | durable close/discard evidence, phase validation, deterministic quarantine recovery, and ordered multi-record publication. | <!-- module: close_transaction -->
| `close_worktree` | capability-bound close/discard marker validation, exact-marker removal, and quarantined no-follow worktree deletion. | <!-- module: close_worktree -->
| `command_registry` | declared public command capabilities used by help/completion/metrics/capability output. | <!-- module: command_registry -->
| `compatibility` | one generated support manifest for readable/writable format versions, public schemas, runtime roles, and unknown-version/extension decisions. | <!-- module: compatibility -->
| `commit_active_reset` | byte-snapshot authority, ordered progress, preflight, and idempotent roll-forward for post-commit active-state reset. | <!-- module: commit_active_reset -->
| `command_execution` | streaming bounded child output, explicit environment profiles, timeout, and platform-reported process-tree termination. | <!-- module: command_execution -->
| `completion` | top-level help and shell completion rendering. | <!-- module: completion -->
| `context` | bounded read-only automation context packs. | <!-- module: context -->
| `doctor` | read-only repository diagnosis using shared open-snapshot and active-transaction validators. The request ledger starts before collection; `DoctorIssueCollection` accounts repository-layout, object-scan, transaction-marker, and malformed/missing-field opened-entry issue destinations across Store validation. Object scanning reuses the pinned Store subject/cache and publishes exact kind/message/path/slot ownership through a consuming work view; scanner sources and failed cache seeds drop before their scalars. Opened-entry issue payloads and the residual Vec slot are likewise reserved before materialization through the Store provisional ledger. `branch_heads::DoctorBranchSnapshot` owns sorted branch membership, bounded no-follow candidate reads, shared-session validation, and the final file/root fence. `opened_registry::DoctorOpenedRegistrySnapshot` owns recursive candidate membership, no-follow directory capabilities, absolute-open scratch, single-link evidence, bounded raw JSON, and parsed JSON `Value` through the post-Store membership/content fence on that same ledger. Parse cleanup and upper-level finish/abort keep the snapshot and ledger retry owner attached after cleanup failure. Marker authority and legacy issue order remain stable. Open-summary evidence work, active transactions, extracted opened fields, later diagnostics, active-state, and JSON collectors remain residuals. | <!-- module: doctor -->
| `evidence` | evidence/artifact capture, batch validation, and result rendering. | <!-- module: evidence -->
| `exit_code` | stable error-to-exit classification. | <!-- module: exit_code -->
| `explain` | read-only explanation of Fire/Atom/verification/storage targets. | <!-- module: explain -->
| `extinguish_ux` | interactive and recent-evidence input assistance. | <!-- module: extinguish_ux -->
| `fire` | manual/batch Fire validation and plan/apply behavior; its `active_ledger_transaction` child owns bounded Fire/Resolution postimage staging and replay. | <!-- module: fire -->
| `http` | HTTP client/server framing, ledger-bound parsed-body ownership, route dispatch, bounded pre-header JSON encoding, typed 413 transport, size/time/concurrency boundaries. | <!-- module: http -->
| `http_tls` | TLS transport configuration and key/certificate loading. | <!-- module: http_tls -->
| `idempotency` | common idempotency record identity/conflict helpers. | <!-- module: idempotency -->
| `import_workflow` | importing an existing directory into a new native open branch. | <!-- module: import_workflow -->
| `limited_yaml` | intentionally bounded YAML-like scalar/key parsing shared by batch inputs. | <!-- module: limited_yaml -->
| `link_batch` | Trace Link batch parsing, full validation, plan, and apply. | <!-- module: link_batch -->
| `merge_patch_idempotency` | merge and patch-import replay/conflict records. | <!-- module: merge_patch_idempotency -->
| `merge_resolution` | strict active merge metadata decoding, typed unresolved-conflict projection, and locked `merge --continue` validation/apply/render flow. | <!-- module: merge_resolution -->
| `metrics` | phase/counter values and attachment under `data.metrics`. | <!-- module: metrics -->
| `migration` | compatibility inspection, typed layout planning, and locked capability-relative application of registered auto-create directories. | <!-- module: migration -->
| `open_clone_idempotency` | open/clone replay and conflict records. | <!-- module: open_clone_idempotency -->
| `open_snapshot` | one typed repository/marker/branch/opened/active authority check, documented pending-window classification, and field-level divergence. | <!-- module: open_snapshot -->
| `open_transaction` | durable open evidence, exact-or-create artifact publication, phase recovery, and transaction archive/retirement. | <!-- module: open_transaction -->
| `pagination` | bounded `codefire.page.v1` metadata and collection/snapshot-bound `codefire.page_cursor.v1` encoding/validation. | <!-- module: pagination -->
| `remote` | file/HTTP remote models, strict authority components, descriptor-backed project bootstrap, ordered project gates, object transfer, and merge-request operations. | <!-- module: remote -->
| `repo_layout` | required versus auto-created repository directories shared by doctor/migrate. | <!-- module: repo_layout -->
| `secure_worktree` | capability-relative, no-follow sibling staging, abandoned-stage cleanup, and single-rename open-target publication below a pinned workspace boundary. | <!-- module: secure_worktree -->
| `secret_redaction` | secret-like command/argv/output and sensitive local-path redaction before evidence or diagnostic serialization. | <!-- module: secret_redaction -->
| `signatures` | commit/request signing, verification, key lifecycle, nonce/replay policy. | <!-- module: signatures -->
| `stable_lock` | persistent no-follow kernel-backed shared/exclusive locks, one operation deadline, and bounded atomic owner diagnostics. | <!-- module: stable_lock -->
| `storage` | bounded local/remote storage report and warnings. | <!-- module: storage -->
| `tests` | CLI-wide regression module compiled only for tests; focused module tests should stay near their owners. | <!-- module: tests -->
| `verification` | verify argument parsing plus text rendering of a core `Verification`. | <!-- module: verification -->
| `view` | show/diff/review-pack/patch views, algorithms, bounded payload handling. | <!-- module: view -->

## Core Domain Flow

Coreの独立domain moduleは、`effective_policy`がstable rule ID/source/value/applicability/safety classと順序非依存digest、`change_coverage`がmanifest changed pathとbase/current/changed Atom relation、file-only precision、tool migration、summary/digestを所有する。CLI/storeは同じ分類やdigestを再実装しない。

`trace_evaluation`はsource Link order、duplicate、自loop一件、direct-only Fire semanticsを保つ一回compileの
adjacency/required-count indexを所有する。`work_budget`は`codefire.decision-work.v1`、clone不能な
`DecisionWorkLedger`、command/phase applicability、stable counters、one-way terminal、複数dimensionを
effect前に一括preflight/commitする`DecisionWorkReservation`を所有する。ledgerを借用保持しないprepared ownerは
private numeric IDを公開しない`DecisionWorkLedgerIdentity`を保持し、permitのないexact/replay経路でも同一ledgerを照合してから
workとconcurrent ownershipを変更する。wall-clock deadlineとprocess-global
HTTP connection permitはdeterministic ledger外のavailability controlである。
Python fallbackのpermitはactive上限を全server/threadで共有し、異なるlive profileではregistryの最小capを同一lock上で適用する。
server constructorはsocket生成前のregistration ownerでregistry publicationをtransaction化し、partial publish、profile/TLS、
socket/bind、post-publishの失敗で以前のregistry/effective capを復元する。厳しいcapの追加時は既存socketをgrandfatherし、新規だけを
active数がcap未満になるまで止める。admissionはTLS/request/handler前、application queueはzeroで、plain rejectは固定503を一度だけ
書き、TLSはplaintextなしでcloseする。permitはrelease成功後だけownerをclearし、server closeとlast active releaseでregistry/
closing stateを回収する。このavailability permitはrequest ledger/counterを生成しない。
HTTP request parsing charges header increments and raw retained input before heap append, releases
header Temporary ownership before reserving the exact body, and transfers parsed JSON to Retained.
`HttpRequestBodyOwnership` binds that live `Value` and its exact scalar to the parser ledger. A
foreign ledger is rejected without consuming either; the owning release drops the `Value` first and
keeps a failed scalar release retryable. Upload/apply move this owner and the parser routing owner
into `HttpMutationRouteIntent`; superseded routing payloads/scalars are released only after the
destination-first derived owner is complete and before Store binding.
The same monotonic request deadline caps mutation-gate waits and is polled inside Store validation,
snapshot revalidation, object collection, upload preflight, signature history, and ancestry loops.

`codefire-core` exposes the domain stages:

```text
build_atom_index(open_dir)
  -> AtomIndex { version, hash_schema_version, atoms, duplicates }

current_trace_graph(open_dir)
  -> parse link syntax and construct typed links
  -> TraceGraph

build_scan_result_against_base(current, &base, trace, policy, prior fires, ...)
  -> changed_atoms + non_atom_changed_files + tool_migration + open_fires

build_change_coverage(manifest changes, current/base indexes, changed atoms, tool migration)
  -> non-Atom / deleted-Atom / out-of-Atom / Atom-change-with-unclassified-spans

parse_effective_verification_policy(open_dir)
  -> final VerificationPolicy + ordered EffectivePolicy rules/provenance/digest

build_verification(scan,
                   VerificationInputs { unresolved merge conflicts, missing links,
                                        stale resolutions, failed commands, evidence refs,
                                        assessment subject, execution records },
                   policy, verified_at)
  -> Verification { result, blocker collections, subject, execution,
                    effective_policy, change_coverage }

build_resolution(request, current atoms/links/policy)
  -> Resolution + content/link/policy basis

validate_sealed_snapshot_semantics(atom index, trace graph, fires, resolutions,
                                   verification, policy)
  -> authoritative schema-2 cross-root consistency decision
```

Hash schema 4 routes configured `.py` code/test artifacts through the bounded Python symbol
extractor. A linear lexical view masks single/continued/triple-quoted strings and identifies actual
comment starts before declaration/marker recognition, while original lines remain authoritative for
block hashing. It derives class, top-level def, and class-owned method IDs compatible with the
supported reference profile, and a nearby explicit `cf-atom` marker owns that symbol once. Other
source languages keep the explicit-marker Rust path until their derived profiles have equivalent
versioned fixtures.

`codefire_core::committable_decision` is the domain truth over raw `Verification` facts. It returns
typed ordered blocker/warning reasons, primary reason, outcome, and the effective-policy/change-coverage subject digests; persisted `Verification.result`
is a checked compatibility projection. Unresolved merge conflicts, required Fires, stale Resolutions, missing evidence,
duplicate Atom IDs, and failed required checks are a non-relaxable safety floor. Missing required
links alone may become warnings through `require_trace_completeness=false`. CLI exit, envelope,
diagnostics, actions, text, state, commit gate/certificate, and store validation consume that one
decision. ADR-013 owns the general boundary; ADR-014 owns active merge facts and their explicit
resolution transition. ADR-019 owns policy provenance and coverage inputs. Coverage is currently
`allow_with_disclosure`; it does not create a second committable predicate.

The sealed-snapshot predicate recomputes open required Fires, duplicate Atom IDs, missing required links, stale resolutions, policy trace-completeness, and the final typed outcome. It also validates effective-policy semantics/digest and verification-policy-root equality, validates change-coverage normalization/classification/summary/digest, and enforces unique Fire/active-Resolution identity and Fire-to-active-Resolution cardinality. Certificate policy/coverage digests must match verification. False legacy safety booleans cannot hide blocker facts, while a historical snapshot omitting both additive contracts remains readable without gaining a provenance/coverage claim. `codefire-store` must call this predicate after decoding current roots instead of duplicating these rules.

Sealed commit and signature-history walks use explicit heap stacks rather than process recursion while preserving
parent-array DFS/preorder and first-error precedence. HTTP bundle collection charges node/edge/depth/frontier,
record/aggregate/retained work before stack insertion, record read, or materialization; atomic bundle overflow has no
continuation and becomes exit 60 / HTTP 413 before a success header is published.
The Python fallback bundle route passes the request ledger through sealed validation, snapshot revalidation, LIFO reachable walk,
public-ID-ordered projection ownership, and canonical response freezing. Final revalidation binds record bytes and opened inode,
so append, truncate, and same-byte pathname replacement are subject changes. A budget- or subject-change-terminal ledger is not
reopened for diagnostics; route-owned memory first reaches zero apart from the request header, then a fixed 64KiB control-plane
response writes 413 or the canonical 400. Its optional deadline polls bounded branch/object read and SHA-256 chunks,
canonical hash/retained-size traversal, validation/final-fence, ordering/projection, response encoding, headers, and
publication. Exact expiry closes route ownership before the one-way `deadline_exceeded` terminal and fixed 503;
earlier budget/subject terminals or cleanup failures keep precedence. Contiguous stdlib `json.loads` remains
non-interruptible, so the complete Python bundle deadline slice is still rejected.

The Python fallback HTTP upload connects an upload-only mutation response owner to the same handler ledger. Actor and
idempotency syntax are validated before temporary staging or layout, and the staged closure validates the sealed root.
A missing project then freezes generation/branch/optional nonce authority paths, canonical postimages, bounds, and absent
preimages in a layout-independent single-use `RemoteJSONPublicationWritePreflight` before server namespace creation.
After bootstrap exposes the random stable project ID, the project/nonce gates revalidate and consume that exact write proof
into the full semantic `RemoteJSONPublicationPreflight`. Existing-project recovery/fast-forward planning and exact
pending/receipt request-hash precedence remain unchanged. The response adapter freezes the exact success and fixed retry
response before effects, and direct/adapter paths both append a fresh signed nonce as the reducer's last authority.
Generic publication preserves its historical validation precedence: write count, identity/root, request/intent, authority
proof, then result/consume. The stage-input closure projection now has two closed owners on the same request ledger.
`AccountedHTTPUploadPlanner` reserves all incoming-index `PyLong` owners before enumeration, runtime-reported
`dict`/`set`/`list` bases, ASCII string owners, collection-specific capacity, and old-plus-possible-new backing-table
scratch before every insertion because runtime rehash thresholds do not coincide with logical growth. It drops traversal
and index locals before clearing collections and releasing their scalars. `AccountedHTTPUploadClosure` reserves both output-list
bases, each retained entry, and both-list sort scratch before allocation. The accounted path allocates none of the legacy
dict/set/frontier/output collections and polls the same stable closure phase during validation/reference traversal/sort.
`AccountedHTTPUploadStage` then strongly binds those exact records/lists and reserves subject ownership before freezing
incoming index/ID/size/digest/private canonical bytes plus existing ID/private bytes/device/inode/link-count/size/content hash.
A fixed preflight Temporary owner and effect-free headroom reservation precede `mkdtemp`; immediately before that effect,
the route rechecks list contents, incoming canonical identity, and every existing source through a second bounded read.
The Stage exposes no plan getter. Its single-use internal writer consumes read-only payload views, clears entry/payload locals,
and detaches payload-bearing failure tracebacks before close drops the private plan and releases Retained/Temporary scalars.
Existing reads require a regular single-link inode at every initial/opened/after/named fence and writes poll in 64 KiB chunks.
Planner collections close before sorting and closure lists close after staging. Sealed validation,
signature/verification/digest/ancestry work, other POST routes, request-wide accounting of remaining upload business
allocations, complete deadline polling, request-header/body deadlines, and connection admission remain separate residuals.

The fallback also has an effect-free `AccountedRemoteObjectSetDigest` prerequisite for the digest portion of that residual.
Its caller supplies an upper bound from canonical source bytes already owned by the request parser. The owner reserves an
ordered reference list, conservative source-proportional iterative-encoder/sort scratch, aggregate canonical bytes, object
count, and retained digest before copying or hashing. It heap-sorts references in place, validates IDs and the source upper
with deadline polling before encoder key-list allocation, and feeds each emitted canonical byte through the ledger into
SHA-256 without a full canonical list buffer. Encoder exceptions lose scratch-retaining tracebacks before cleanup, and every
permit is published to the retry owner before its first consumption. The digest cannot escape; only owner-scoped constant-time
comparison is exposed. Production upload/publication wiring and an accounted destination transfer remain open.

The frozen production handoff must not use the Stage's sealed-validation tuple as the digest source bound. That tuple covers
only reachable staged closure records, uses core records with `remote` removed, and includes the persisted newline; upload
semantics digest the canonical full request `objects` list, including unrelated request records and historical extensions.
The HTTP parser already retains the exact canonical full-request size. Production must pass that owned scalar as a conservative
object-list upper and let the digest owner's bounded sizing pass prove the subvalue fits before encoder allocation. A direct
same-ledger handler caller must provide equivalent parser-owned evidence; it must not materialize a new canonical list merely to
discover the upper.

Digest preparation remains after signature, server verification, publication identity, and missing-project write/response
preflight. On an existing project it remains at the current semantic-construction point under the continuous project gate;
moving it earlier would change first-error order. A missing project instead holds bootstrap authority, preselects the exact
project descriptor/stable ID, builds a private staged layout, and acquires that stage's project-gate inode before visibility.
Its typed bootstrap/publication proof pins the semantic/request hash, absent authority/current state, nonce/write plan, response
preseal, and immutable copy input before the terminal handoff.

The destination is a same-ledger upload-semantic owner allocated before digest work. It adopts the private digest, ordered
record references, source evidence, and parser scalar without a release/reacquire gap; no digest-string getter is permitted.
Immutable copy bytes have a separate Retained owner and construction Temporary overlap. After recovery/current, ancestry,
nonce/publication preflight, response preseal, and every other reject-capable callback, the owner revalidates the complete request
record set and performs a callback-free detach of both semantic and immutable-copy inputs. Effect code consumes only those pinned
inputs and never rereads ambient records. Existing-project order is continuous gate -> recovery/current -> ancestry -> preseal ->
terminal -> effect-start -> pinned copy -> reducer. Missing-project order is continuous bootstrap lock/private-stage gate/proof ->
terminal -> effect-start -> exact proof-consuming rename of the already-gated stage -> durable-only reducer under the same gate
token.

The production-unwired Python prerequisite now implements the local ownership portion of that contract. A caller-owned one-shot
`AccountedHTTPUploadParserScalar` performs the pre-parse Retained-upper acquisition and retry-safe post-parse exact shrink itself.
Generic `ledger.current >= amount` evidence is rejected because it cannot prove which owner acquired the scalar and permits duplicate
bind/double release. `AccountedHTTPUploadRequestSource` moves one exact reconciled token into the exact full request and objects list,
and rejects a second source. Caller-owned two-stage `AccountedHTTPUploadSemantic` adopts that exact source owner before digest work,
rejects a Stage-like false bound by remeasuring the full request, publishes the exact digest child and caller-created permit before
reservation, and privately transfers its digest into one stable semantic mapping. It separately preaccounts the same conservative
`4096 + 256L` canonical encoder Temporary envelope used by the digest owner and builds effect-ready core-plus-newline object bytes
while retaining the full record identities and full-record digest evidence, so historical `remote` fields remain semantic even
though they are absent from persisted copy bytes. Eight source-sized semantic work passes cover prepare size/fingerprint,
validation/hash, serialization, and their terminal size/fingerprint/full-record/copy-parity counterparts; digest child work and its
simultaneous encoder envelope remain separately charged. Deep/key-heavy request fixtures hold the exact combined Temporary upper
and reject exact-minus-one. The last deadline callback precedes callback-free full request fingerprint/list/ID/duplicate/digest/
copy-parity revalidation. Mutable request refs then drop before the parser scalar, construction Temporary releases with post-effect
reconciliation, and the preallocated read-only mapping proxy/copy tuple move into private effect slots by field assignment only; the
owner drops its direct underlying-dict reference. There is no digest or copy getter. The real handler must mint/adopt this token at
its parser-upper acquisition boundary, reconcile exact after parse, and never bind generic current. That parser-token transfer,
mapping lending through recovery and publication preflight, missing-project bootstrap proof, effect-start, and pinned-byte writer
remain unwired; this prerequisite does not change production first-error or effect ordering.

Server verification is isolated behind production `AccountedServerVerification`. It reserves capability/path and raw-policy-derived
AST/parse ownership before materialization, opens policy and cwd relative to a pinned project, and fences exact project/policy
identity and bytes before each spawn. Linux children receive the pinned project and cwd through `/proc/self/fd`; an ambient rename
after the fence cannot redirect either. Bootstrap skips policy only while the actual project path remains absent. Combined output is
bounded, escaping failures retain only a fixed sample/count/digest, and uncertain kill/reap keeps the exact process owner for retry.
All permits are stored in owner fields before one-shot `reserve_into` registration. FD fields are detached before exactly one
captured-builtin close invocation; an exception consumes numeric ownership because same-inode/same-number reuse cannot be
distinguished safely. Config/exception payloads are dropped before Retained release. These boundaries cannot roll back filesystem,
network, or service effects already performed by a configured command. Production HTTP upload passes the handler's same request
ledger and deadline through the compatibility wrapper into this owner; standalone/no-ledger callers retain the local-ledger path.
A production-unwired seeded construction path is reachable only through `AccountedRemoteSignaturePolicyAuthority`. After the
signature child verifies and closes, it borrows the identical immutable P0 bytes, duplicates the pinned project/policy descriptors,
and opens every cwd relative to that duplicate without an ambient policy parse. The two children never overlap. A partial verification
child is published to the authority before its first fallible acquisition, and a guard spans signature construction/verification/close
through verification construction so loader/dup callbacks cannot consume the final fence or close either owner early. The child must
run and close before the authority final fence. This is a terminal-coordinator prerequisite, not production handler wiring.

Production HTTP upload now moves only the sealed Stage's immutable record-count, aggregate-byte, and maximum-record-byte bounds
into `AccountedHTTPUploadSealedValidation`. The same handler ledger/deadline spans prepare, iterative validation, final staged-tree
fence, finish, and retryable close; the no-ledger direct path retains legacy validation. Incoming/existing payload plans do not
escape through this handoff, and later signature/verification/digest/ancestry/publication order is unchanged.

The production sealed owner owns staging-provided dimension bounds, the iterative validation session, borrowed documents, deadline
work, and every traversal FD.
Its FD contract also detaches before one close attempt; a synthetic pre-call fault may leak the original descriptor rather than risk
closing ambient authority after numeric reuse. It now preallocates production-unwired single-use signature/ancestry borrow handles.
They expose only already-retained validated commits and never invoke the loader. Before exposure the owner claims a canonical
full-record fingerprint destination, checks it at handle close, retains it through disk finalization, then performs a callback-free
final canonical check and immediately detaches the exposed document before scalar cleanup. Lookup, release, and finish are
single-flight; callback-rich work is followed by a callback-free handle/owner token recheck before the payload local is obtained.
Mutation or ambiguous activation makes the owner abort-only, and failed lookup tracebacks are detached before payload release.
Two additional fallback owners remain deliberately prerequisite-only.
`AccountedCommitSignaturePolicy` owns raw/parsed policy, borrowed keys, iterative history collections, and streaming HMAC work
through fixed-slot intrusive transitions that reconcile post-effect accounting faults and unlink fully released nodes without
allocation. A separate production-unwired `AccountedRemoteSignaturePolicyAuthority` pre-reserves path ownership, pins an absent
or exact regular single-link `server_policy.json` behind no-follow project capabilities, and lends its immutable raw bytes once to
the pure signature owner on the same ledger/deadline. Its combined one-shot surface can instead verify and close that child before
lending the same bytes and duplicated capabilities to server verification. Its final fence rechecks ambient project and exact policy identity/content;
child payload closes before raw/digest/FD/scalar ownership, and malformed JSON/UTF-8 errors retain no raw policy payload.
A production-unwired `AccountedHTTPUploadSignatureVerificationComposite` owns one validated sealed owner and one unused policy
authority on the same ledger and identical deadline. It passes the exact preallocated signature handle itself as loader—without
lambda, clone, session load, or store reopen—then closes the signature child and owner-first releases/revalidates the handle under
both parent guards. Only a successful midpoint release permits verification config/cwd construction or child publication. Failure
returns no child and retains both parents for explicit retry; cleanup releases an active borrow, then policy authority, then sealed
authority, stopping before sealed cleanup if policy cleanup fails. Coordinator tokens reject stale direct aliases for its lifetime.
`AccountedHTTPUploadAncestry` owns a full BFS seen/deque and frame lifecycle through a
borrowed validated-commit loader. A fixed owner gate accounts transition bytes before construction; child depth is created only after
frame acquisition; per-insertion retained/overlap bounds cover CPython growth and deque popleft churn. It preserves full traversal
after a match and drops loader/parent locals before retryable scalar release. The production-unwired
`AccountedHTTPUploadSealedAncestry` is returned before explicit single-use prepare activates only the sealed owner's preallocated
ancestry handle. It passes that exact callable and identical ledger/deadline into the BFS owner, then closes BFS payload/loader before
releasing the borrow. Both cleanup fields publish before fallible activation/construction. Partial decision/reentrant prepare is rejected;
callback close is carried through the active outer frame and returns typed cancellation instead of successful closed prepare. Faults
retain exact caller-owned retry state. It neither clones/reopens commits nor finishes the sealed owner. Signature integration,
handler/terminal fencing, ordered signature/digest/ancestry, publication, and complete upload deadline closure remain residuals.

These sealed-borrow and signature-authority prerequisites cannot be production-wired by choosing one final-fence order.
Policy-first leaves policy mutable during sealed callbacks; sealed-first leaves staged objects mutable during policy callbacks.
Scalar cleanup is LIFO, signature-time finish would close future ancestry, missing layout conflicts with the absent-policy fence,
and ambient server-verification policy rereads can select a command outside the pinned authority. The required boundary is an
upload terminal coordinator: own the verification plan from pinned policy, prove typed bootstrap transitions, keep consumers
through gate/recovery/ancestry/preflight, preconsume fallible callbacks/read charges, then callback-free detach both authorities
immediately before an effect consuming immutable staged/publication bytes. No complete terminal pair is claimed.
The signature composite closes only the retained-borrow-to-verification-plan prerequisite and is not that terminal coordinator.
The existing-project consumer characterization runs pinned signature verification and, through a test-only unified token, full
BFS ancestry under `remote_project_gate`. Production cannot make that transition: the signature composite retains both parent
tokens, the ancestry handoff rejects the sealed parent, and closing the composite destroys both parents. Verification failure may
already leave subprocess effects, while the composite has no later abort transition. Even after all consumers, the cooperative gate
does not suppress same-thread callbacks: policy-first permits a sealed callback to change policy, sealed-first permits a policy
callback to replace a staged object, and live policy scalars make sealed-first cleanup indeterminate. The current publication helper
also accepts ambient records created after sealed final. These six tests characterize the missing top-level token, callback-free
pair detach, abort transition, and immutable publication capability; they are not a coordinator implementation.

The production-unwired `AccountedHTTPUploadExistingTerminalOwner` now resolves only that consumer-lifetime token dead end for an
already-existing project. A request-scoped ledger registry blocks both exact parents before their diagnostic mirror slots are bound.
It returns no child: exact signature borrow, pinned server-verification run/close, and exact full-BFS ancestry borrow are proxied in
one owner. External-effect failure is abort-only, callback close becomes outer-frame typed cancellation, and cleanup stops/retries
child-before-borrow and policy-before-sealed. It has no missing-project bootstrap, policy/sealed pair recheck or detach, semantic
connection, immutable publication input, effect, or handler surface. The characterization tests therefore remain evidence for the
missing terminal pair/publication boundary rather than a claim that the complete coordinator exists.

## Store and Identity Flow

```text
payload
  -> canonical_json(payload)
  -> SHA-256(type_tag || NUL || canonical_bytes)
  -> full digest
  -> public object ID using the current 24-hex prefix
  -> ObjectRecord { object_id, type, hash, payload }
  -> immutable object subdirectory
```

Object writers and readers open the absolute object root and subdirectory component-by-component without following symlinks; a symlink root/intermediate directory fails closed. Ordinary readers validate each stored record's internal agreement among payload, full digest, public/legacy ID, and filename. Record serialization is capped at 64 MiB on write/read; readers open the final name no-follow and non-blocking, require a regular file, and enforce both metadata-size and bounded-read limits. Writers validate and sync a create-new temp, publish it with a no-replace hard link, then recheck root/subdirectory identity; existing-path reuse performs the same final recheck. Public prefixes never replace full identity: both the pre-existing-path and hard-link race branches validate the existing record first, then require requested/existing type, full digest, and canonical payload equality. A mismatch is the typed `ObjectCollision` error and never reuses the earlier object silently.

`store_object` constructs new records and always assigns the current 24-hex public ID.
`store_object_record_document` is the transfer/migration boundary: it accepts a complete record,
recomputes and validates its full identity, permits only the supported current or legacy ID width,
and publishes that exact authenticated public ID plus supported non-identity extension fields through
the same capability/no-replace/collision path. Existing immutable reuse requires the exact same
extension map. Unknown critical descriptors fail closed; noncritical descriptors are preserved, and
their declared object references participate in copy, bundle, Doctor, and GC reachability. File/HTTP
graph copy therefore never rewrites a sealed edge or silently drops an extension dependency.

Sealed validation opens a capability for the canonical objects root, binds the twelve built-in subdirectory states plus at most 128 currently present extension subdirectories as identity-or-missing, and opens records relative to retained subdirectory handles with final-component no-follow semantics. Exceeding the extension cap is an explicit `InvalidSealedCommit` before cache retention. A cache subject contains canonical root path, root identity, and the ordered subdirectory name/identity-or-missing set. Validation entries are staged until the ambient root and each bound child identity pass the final recheck; root/child replacement creates a different subject or fails validation. `validate_sealed_commit_pinned[_with_cache]` additionally returns `ValidatedSealedCommit`, which owns the exact bound handles beyond validation, reads the commit/roots/reachable objects through them with subject checks before and after consumption, and exposes an explicit identity check for both sides of a mutable publication point. Full-store scans use the same bound subject and final identity check. One cache retains only the current store subject and evicts prior pins/results when a different root or identity set is bound, while each live validated token retains one independently bounded handle set for its operation. Platforms without stable directory identity reuse within one traversal only and cannot claim a token publication recheck. This does not detect an in-place rewrite of an already cached object file, which remains forbidden by the immutable store contract. ADR-011 owns this capability boundary. ADR-012 independently dispatches cross-root semantics per commit: current `sealed-snapshot-v1` requires endpoint closure; profile-absent history retains only its legacy claim.

Current Rust sealing writes commit version 1, root schema 2, and version-1 roots. New verification/policy roots add the ADR-019 effective-policy/change-coverage claims and certificate digests without changing blocking semantics. Missing root schema selects the legacy schema-1 required roots; schema values other than 1/2 and commit/root versions other than 1 fail closed. Empty `missing_evidence_refs` and `unresolved_merge_conflicts` may be omitted from schema-2 verification. Only missing evidence has a compatibility certificate count; merge conflicts are active-only facts that must be empty before sealing. After shape/reference checks, schema 2 is decoded into core types and evaluated by the authoritative sealed-snapshot predicate before certificate agreement is accepted.

## Open Workspace Flow

`open` validates the sealed branch head and closed branch state, materializes the commit, then creates:

- `.codefire-open` in the opened directory with repository, branch, and open-instance identity;
- `.codefire/opened/<encoded-branch>.json` with path/base/active-state linkage;
- `.codefire/active/<open-instance>/state.json` and later scan/fire/resolution/verification state;
- branch state `open-clean`.

The requested target is resolved below the canonical repository-parent workspace boundary with parent traversal, existing symlink components, repository overlap, and existing-open overlap rejected. Resolution retains no-follow capabilities for the workspace boundary and every deepest existing target ancestor; an ancestor that is renamed/replaced before staging fails identity validation, and a previously absent component is created with create-new semantics rather than adopting a concurrent replacement. Apply derives a deterministic `.codefire-open-stage-<hash>` sibling and an exact `.owner.json` provenance record from repository path/identity, branch, target, and sealed base. `secure_worktree` materializes every file plus `.codefire-open` into the pinned stage and makes the complete worktree visible with one sibling rename. A failed materialization drops its stage and owner without making the target non-empty; a retry with the same owner can remove and rebuild an abandoned stage, while missing/mismatched provenance fails closed. After rename, the sealed content manifest is recomputed through the retained target capability with bounded no-follow traversal; symlink, non-regular, and non-UTF-8 entries fail closed, every file/directory name is rebound after use, and the ambient target identity is rechecked. The same target capability and sealed-store token remain live through repository metadata publication. A platform that cannot atomically replace a pre-existing empty target fails without a remove-then-rename fallback.

The worktree rename is the open command's materialization visibility point; it is not an atomic publication of all open state. Before a live open, `.codefire/open_transaction.json` is created with atomic no-replace semantics and becomes repository authority. Exact retries advance adjacent durable phases `prepared -> target_published -> registry_written -> active_state_written -> branch_written -> idempotency_written`. Each phase classifies the target, opened registry, active state, branch object/reference, and optional idempotency artifact as exact preimage/postimage evidence; divergent or impossible combinations fail closed. The branch object is stored and identity-checked before its mutable reference. Target/store identities are checked immediately before and after every later registry, active-state, branch-object/reference, and idempotency visibility point. Completion writes an exact-or-create `active/<open-instance>/open_transaction.completed.<transaction-suffix>.json` archive, quarantines the identity-bound pending marker as `open_transaction.removing.json`, and only then unlinks it. A crash can still expose an intermediate phase, but an exact live retry reconciles it and doctor reports valid pending evidence; recovery is retry-driven rather than a background daemon. Dry-run never recovers or mutates a pending transaction. Failures use `OpenTargetResolution` and render `open_target_resolution_failed` with reason, repo/branch, requested/resolved path, workspace boundary, and a safe-target dry-run next action. ADR-009 records the target dependency and replacement boundary.

`close` and `discard` bind the recorded open directory through parent/target capabilities before reading `.codefire-open`. The marker read is no-follow, non-blocking, regular-file-only, and capped at 1 MiB; repository path/ID, branch, opened path, and open-instance ID must agree with repository state. A live operation rejects pending active commit/state publication, then publishes version-1 `.codefire/close_transaction.json` with no-replace semantics before its first destructive write. The marker records the exact request, branch/opened/marker preimages, target and parent identities, deterministic quarantine components, branch postimage/object ID, and result.

Recovery advances adjacent phases `prepared -> worktree_applied -> active_state_removed -> registry_removed -> branch_written`. `--keep-dir` removes only the identity-bound marker; full close/discard quarantines and recursively removes the pinned tree without following symlinks. Active state and the opened registry are then removed through their own transaction-derived, identity-bound sibling quarantines. Every phase accepts only its exact artifact state or the immediate apply-before-phase crash gap; a divergent replacement fails closed. Branch publication stores and validates the immutable closed-branch object before replacing the mutable reference, so the postimage object may safely preexist any prepublication phase. Completion exact-or-creates `.codefire/close_transaction.completed.json`, then quarantines and unlinks the pending marker; a later distinct close retires the previous fixed archive before starting. The exact live request resumes a pending marker, dry-run reports it without mutation, and doctor reports valid pending or conflicting open/close authority. Pending open and close transactions mutually block each other.

`load_verified_open_context` now constructs one `open_snapshot::VerifiedOpenSnapshot` before ordinary consumers may use an open workspace. Its locator check binds the canonical repository path/ID, marker/branch/opened names, opened path, open-instance ID, original-open commit, and the exact `.codefire/active/<open-instance>` path. Its identity/state check then requires opened current base versus published branch head and branch/opened/active state agreement. The resulting `OpenContextAuthority::Verified` token is carried by `OpenContext`; the exact legacy Python status shape remains a separate read-only `LegacyPythonStatus` exception and cannot authorize mutation.

A mismatch is `OpenSnapshotDivergence`, not a boolean or generic state error. The JSON/doctor contract carries `phase`, `authority`, `expected`, `actual`, both record paths, repository, and branch, and offers only the read-only `inspect_open_snapshot_divergence` action. A structurally and subject-valid pending commit, version-2 state transaction, or validated unreleased active-ledger descriptor is classified as `OpenSnapshotDisposition::PendingCommit`, `PendingState`, or `PendingActiveLedger` instead of steady-state corruption. The state/ledger pair is accepted only when operation/request hash agree and the publication statuses permit the recorded state phase. Ordinary/read-only consumers return typed `open_recovery_pending`; only a live command holding `RepoLock` and naming the exact owning operation may receive that pending context and run its reducer. Foreign, malformed, cross-operation, simultaneous, or impossible phase/evidence combinations remain divergence and are preserved. Documented commit publication windows may temporarily admit branch-new/opened-old, but only when the pending candidate, parents, stored-role ledger, sealed candidate, phase, and subject prove that exact window.

The snapshot retains an `ActiveStateDir` bound to the active parent and instance directory identities. Active files are one normal relative component; reads are no-follow/non-blocking and bounded, while writes use synced create-new temps and capability-relative publication. Reads, writes, file removal, and tree removal recheck the ambient parent/instance subject at use boundaries. A post-validation instance rename plus outside symlink therefore returns the typed `active_state_capability_invalid` diagnostic without using an ambient path fallback. Doctor runs the same snapshot and transaction validators and exposes the same typed divergence fields. A valid pending commit/state/active-ledger issue additionally carries typed authority, operation, phase, intended state when applicable, and a bounded request identity: commit transaction ID, active-ledger request hash, or version-2 state subject. It never projects the stored request payload, durable ledger result, commit message, operation plan, or idempotency key. Its only executable action is read-only `doctor`; `codefire_pending_authority_target` names the exact owner/mode and `exact_request_only` retry contract with `automatic_repair=false`. Active-state consistency counts only Fire records whose validated `status` is `open`; extinguished history is compatible with `open-clean`/`open-consistent`. Doctor and storage inventory bind the active root and children no-follow rather than recursively following ambient names. [ADR-015](../../adr/ADR-015-verified-open-authority-and-active-transitions.md) owns this authority/recovery boundary and [CFR-163](code-review-issues/cfr-163.md) tracks its delivery gate.

### Import bootstrap and replay

`import_workflow` normalizes the target and complete semantic request before any lookup. Live import atomically publishes a fully formed versioned `.codefire-import.lock` provenance record, takes its stable kernel lock, and advances a separate `.codefire-import-transaction.json` through planned, initialized, opened, scanned, and completed phases. The target directory is pinned as a no-follow capability; repository and open-state reads/writes remain relative to that capability, and every success path rechecks the ambient target identity. Import creates and owns the persistent repository kernel lock before making `repo.json` visible, requires a regular no-follow inode with exactly one provable link, leaves its contents untouched, proves the lock pathname still names the held file at acquisition/use boundaries, and atomically publishes owner diagnostics to the common sidecar only while holding it. It then publishes the sealed object graph before the initial branch ref and persists scan state with the shared scan transaction phase validator. Repository `repo.json` carries exact matching import/open provenance, so retry can resume a repository created by the same transaction without adopting an unrelated `codefire init`.

The exact root lock, transaction, and open marker names plus the reserved root bootstrap/atomic-write temp patterns are excluded from Atom/content scans; matching nested or merely similar user files remain ordinary project content. Bootstrap publication first writes `.codefire-import.lock.bootstrap-*.part`, then hard-links the fully synced bytes to the deterministic `.codefire-import.lock.bootstrap-*.tmp` candidate before linking the canonical lock. `.part` is unpublished scratch: discovery never parses it as authority and recovery preserves it unless it can prove that it is another name for the canonical lock. A malformed published `.tmp` is preserved and rejected; a different fully formed contender remains owned by its process and is not unlinked by the winner. Atomic-write remnants use `.codefire-import.write-<transaction-id>-*.tmp`; only matching IDs are removed during retry, while foreign IDs are preserved and rejected.

The idempotency v2 record stores a hash-only canonical request binding and the original result. Replay verifies the stable repository identity, import provenance, sealed recorded base, marker/opened/branch subject agreement, and coherent current sealed head. Mutable branch state may advance after a successful import without invalidating the original replay. Foreign/partial lock provenance, malformed transactions, legacy result-only markers, and a reused key with a different request fail closed.

Every import input read is capability-relative/no-follow, non-blocking, regular-file-only, metadata-size checked, and bounded again with a limit-plus-one read. Parsed operational/control JSON is capped at 1 MiB; repository byte-copy paths are capped at 64 MiB; ordinary imported user files are capped at 49,545,216 bytes (47.25 MiB) so base64 encoding and reserved record overhead cannot exceed the object store's 64 MiB serialized-record cap.

## Scan and Verify Flow

`run_scan_command` acquires `RepoLock`, reloads the open context under that lock, validates the complete sealed current base through a command-scoped store cache, builds the current Atom index/Trace Graph, applies the shared endpoint validator, compares the full manifest and base/current indexes, and derives Fires plus typed `ChangeCoverage`. The legacy-named non-Atom list is a compatibility projection of non-Atom, deleted-Atom, and out-of-Atom entries; a changed Atom path is represented by `atom_change_with_unclassified_spans` and never claims whole-file span coverage. `ScanExecution` carries the same `ValidatedSealedCommit` capability into verify; verify does not perform a fresh ambient validation that could silently adopt an equal-content replacement store. The token is checked before state publication, before completion, and after completion. `run_verify_command` uses the same lock-owning boundary and builds on that scan before executing constrained commands and persisting the core `Verification` with effective policy and coverage. Policy and active merge metadata are both preflighted before the scan phase may write; malformed conflict paths/state therefore fail before active scan/verification mutation. Nested commit/batch paths pass their existing lock capability; `compute_scan`/`compute_verify` remain explicitly read-only for status, explain, dry-run, and prediction callers. A lock is bound to one canonical repository root and cannot authorize another repository's state writes.

`assessment_subject` derives a version-1 live subject from repository/branch/open-instance/base, canonical scan content, and the canonical worktree-manifest digest. Context, explain, verification, automation JSON, and the later commit certificate project that same digest. Review pack instead uses an immutable `sealed_commit_pair` subject after both commits pass sealed validation. Verification command cwd is canonical and contained below the open root before active publication. Default execution clears the caller environment and uses fixed PATH/LANG/LC_ALL/TZ plus explicit policy overrides; `inherit_env=true` materializes a Unicode snapshot and remains explicitly weaker. `command_execution` drains both pipes concurrently, stores only the configured prefixes, and terminates the Unix process group on timeout. Each record binds assessment subject, profile/phase, input/environment/worktree digests, resource counters, termination scope, and redaction count. Verification refuses a stale subject before command execution; an exit-zero command whose before/after worktree digests differ is blocking. Commit rechecks the subject after verification, before decision/transaction publication, after manifest planning, and immediately before branch publication. Store validation enforces the exact live/sealed subject variant and rejects unbound version-1 fields. ADR-016 owns this trust boundary.

Local merge apply records pending parent/base plus text, binary, and semantic conflict arrays in active state. `merge_resolution` decodes those arrays strictly and projects every entry into `Verification.unresolved_merge_conflicts`. Recorded conflicts remain blocking even after a user edits files until `merge --continue` verifies that generated text markers are absent, recorded binary sidecars are removed, and semantic candidates are explicitly acknowledged. Live continue takes `RepoLock`, repeats validation against a pinned no-follow `ExistingWorktree`, invalidates stale verification, records a version-1 resolution acknowledgement, clears the unresolved arrays, and reruns locked scan while preserving the pending merge parent for the later two-parent commit. Dry-run and blocked continue paths write nothing.

Live scan/verify persist a version-2 `state_transaction.json` marker before their first active-state write. In addition to operation, intended state, phase, and timestamps, version 2 binds `{branch, open_instance_id, current_base_commit}` and an agreeing branch/registry/active pre-state triple. Scan advances `started -> verification_invalidated -> fires_written -> scan_written -> registry_written -> active_state_written -> branch_written`; verify advances `started -> verification_written -> registry_written -> active_state_written -> branch_written`. The writer rejects non-adjacent, skipped, reversed, or cross-operation updates. Exact retry requires the same operation, intended state, and subject before its first recovery write, then recomputes the derived files instead of trusting them. A live scan invalidates prior `verification.json` because that verification belongs to the superseded scan subject. Completion rechecks persisted Fire/scan/verification facts and the sealed base, then requires all three state authorities to equal the derived intended state before removing the marker.

Fire and Resolution publication has two coordinated reducers. `fire::active_ledger_transaction` first writes bounded version-1 `active_ledger_transaction.json` authority containing an operation/request hash, durable result, up to two exact Fire/Resolution preimage hashes, staged postimages, and `published_count`. It accepts only `fires.json` and `resolutions.json`, uses the retained `ActiveStateDir`, and rolls forward the immediate file-write-before-progress gap. A completed record is compacted to the request/result hashes, durable result, effect hashes, and `state_publication=pending`; exact replay verifies those current effects and returns the original JSON result. Only completion of the matching state transaction changes that field to `released` with a completion timestamp. A different request cannot replace a ledger- or state-publication-pending authority, while an unexpected current hash is preserved as divergence. Manual and batch `fire` and `extinguish` all use this reducer; dry-run only invokes the read-only pending guard.

After ledger publication, the same operation uses version-2 `state_transaction.json` to publish its target state through `started -> registry_written -> active_state_written -> branch_written`. Exact retry resumes only the same operation/subject, validates each documented stable/intermediate state tuple through `VerifiedOpenSnapshot`, and removes the state marker only after registry, active, and branch all equal the intended state; it then releases the matching active-ledger authority. Version-1 state markers remain shape-readable for legacy diagnosis, but they lack the subject/pre-state proof required for a recoverable `VerifiedOpenSnapshot` pending disposition. `validate_active_ledger_descriptor` is the shared read-only authority for commands, open snapshot, close, and doctor: it validates pending recovery images or complete effects and returns operation, phase, request hash, and `LedgerPending|StatePublicationPending|Released`. Doctor turns the first two into typed `pending_active_ledger_transaction` issues and an inspect-only exact-owner target; `Released` is nonblocking, while malformed evidence is `invalid_active_ledger_transaction`.

Failpoint evidence covers every scan, verify, and active-ledger state publication phase; every Fire/Resolution journal/file/progress/completion gap; exact replay; different-request collision; malformed/divergent postimages; no-follow and byte limits; and dry-run/no-write. This evidence proves convergence for the recorded reducers, not atomic multi-file visibility to readers that ignore their pending authority.

Repository locks use persistent, kernel-backed advisory lock files opened as regular no-follow files with exactly one provable link. Their contents are never truncated or written on acquisition; identity/link provenance is checked before/after locking and at the mutation use boundary. File existence is not ownership, process exit releases the kernel lock, and the inode is never unlinked during unlock. Best-effort owner metadata is atomically replaced after lock acquisition in `repo.lock.owner.json`, read no-follow with a 16 KiB cap on contention, and left in place after unlock. ADR-010 records crash/ABA behavior and the remaining specialized-lock boundary.

Read-only preview paths must not write scan state. Dry-run and live mutations must share validation/plan semantics.

Generic repository JSON outside smaller specialized formats uses a symmetric 64 MiB read/write cap. Writes reject an oversized serialization before temp publication; reads open every absolute parent component no-follow and the final regular file no-follow/non-blocking, with metadata and limit-plus-one bounds.

## Commit Transaction Flow

Production `run_commit` constructs the commit payload directly; it does not call the store test/helper `commit_payload`.

```text
open_context + scan + verification + active fires/resolutions + policy
  -> operation plan
  -> reject or dry-run if not applicable
  -> commit_transaction.json(status=pending, phase=started,
                             active_reset=exact file/state byte snapshots)
  -> resolve any required signing key
  -> pure-plan manifest blobs + seven roots + signed/unsigned candidate + branch snapshot
  -> publish every ordered role/path/type/ID entry as planned before the first store
  -> store blobs and roots, checking each ID and marking each role entry stored
  -> store/mark the candidate commit
  -> validate the complete sealed candidate with the command-scoped cache
  -> authenticate any present/required candidate signature with the same trusted local key
  -> store/mark the precomputed branch snapshot object
  -> publish the mutable branch ref
  -> update opened registry base/state
  -> phase=resetting_active_state
  -> remove each recorded active file with ordered durable progress
  -> publish and authenticate the recorded open-clean state.json postimage
  -> remove commit_transaction.json
  -> persist idempotency result
```

If signature policy requires a key, `run_commit` resolves that key immediately after creating the pending marker and before the first immutable write. Missing key leaves a `started` marker with an empty object list and no candidate or branch publication. Store sealed validation intentionally validates graph/semantic integrity rather than HMAC trust, so the CLI rereads the validated candidate through its pinned token and verifies a present or required signature with the same local signing key before storing the branch snapshot or publishing a mutable reference. A tampered signature leaves a typed authentication/signature error and a diagnosable `validating_candidate_commit` marker.

The version-1 marker's current `objects[]` is an ordered role ledger: `{role,path,object_type,object_id,state}`. Duplicate blob IDs at different paths remain separate entries. A candidate alias mirrors the candidate entry; legacy string entries remain readable as stored objects. Immutable objects written before publication may be orphaned safely, and even a still-`planned` entry can identify the object in a store-to-marker interruption. Mutable branch-ref publish is the visibility point and occurs only after complete candidate validation and branch-snapshot storage.

The same marker owns a version-1 `active_reset` authority from `phase=started`: exact presence/byte-count/SHA-256 preimages for `fires.json`, `resolutions.json`, `scan.json`, `verification.json`, `active_ledger_transaction.json`, and `state_transaction.json`, plus exact preimage and canonical `{"state":"open-clean"}` postimage evidence for `state.json`. Reset progress is an ordered `removed` prefix followed by `state_file.progress=written`. Before any continuation, `commit_active_reset` validates the complete authority and every current target without writing. It accepts only the exact recorded state or the one remove/write-before-progress gap, records progress after each apply, rejects reappearance/replacement, and verifies the final postimage. Legacy `publishing_branch` markers may be upgraded only while reset is provably unstarted; a legacy partial reset without authenticated preimages remains inspection-only. A live or dry-run commit also rejects a pending active-ledger transaction before commit planning; a successful commit removes the validated compact receipt with the other active artifacts, so reset cannot orphan staged Fire/Resolution postimages or carry their replay authority into the new base.

Current evidence covers store-before-marker interruption for manifest blob, sealed root, candidate, and branch snapshot, plus the pre-validation window and invalid certificate/missing root/wrong root type/invalid parent/invalid present signature rejection. Each store-gap fixture proves discovery through the planned ID while branch/opened heads remain unchanged. Do not generalize those fixtures into an exhaustive failpoint at every write.

## Repository Layout Ownership

`repo_layout::REPO_DIRS` is the source of truth shared by doctor and migration:

- required: `objects`, `branches`, `opened`, `active`;
- auto-created: `cache`, `locks`, `remotes`, `idempotency`;
- object subdirectories come from the store registry; `artifact_refs` and `evidence` are auto-created while other current object subdirectories are required.

Additional state includes `repo.json`, encoded branch/opened records, active state files, locks, remotes, and idempotency records. Exact shapes belong in `source-data-model-catalog.md`.

Local Doctor captures `.codefire/branches` with one ledger-bound snapshot. It pins the parent and optional branch directory, retains sorted membership and candidate evidence, validates valid heads through the recoverable shared Store session, then rechecks entry identity/content, membership, directory slot, and ambient parent before report handoff. Regular-file append races cannot grow the read Vec: a fixed-size read is followed by a separate one-byte EOF probe, with actual read and hash counters consumed from a `size+1` permit.

Opened-registry discovery uses a separate `DoctorOpenedRegistrySnapshot`. It reserves Retained root/path/membership/raw ownership before opening the no-follow `.codefire`/`opened` capabilities or allocating sorted candidate paths and fixed record buffers. Each initial and final absolute parent open first computes an allocation-free checked Temporary upper for the root `PathBuf`, all normal-component frames and bytes, their Vec, and overlapping traversal handles; exhaustion stops before the ambient root or any child handle is opened. Recursive directories and JSON files are opened descriptor-relative and no-follow only after their queue/handle and physical-read reservations. Candidate bytes require single-link metadata both before and after the fixed read from the same open file, use a metadata-sized destination reservation, and probe EOF with a separate stack sentinel; typed allocation-free unavailable evidence preserves open/metadata/identity/size/link rejection detail for ordered `invalid_open_registry` diagnostics. The snapshot parses lent raw slices without an ambient reopen after jointly reserving serde Temporary scratch and a parsed-Retained upper. Exact compact `Value` ownership remains beside the raw evidence through Store finalization. Valid and malformed transitions drop unpublished payloads before rollback; any failed scalar release stays on the snapshot, while finish/abort restores a taken snapshot and detached ledger for same-owner retry. Finalization repeats the pinned read and rejects membership, identity, link count, size, or byte changes, including hardlinks added after capture or inside the final read window, same-inode rewrites, and growth. Root/nested replacement cannot redirect the payload read. Success and changed-subject paths drop parsed/raw bytes, capabilities, membership, and paths before releasing their scalar. Extracted field clones and later marker/snapshot/active-state diagnostics remain CFR-168 residuals.

The preceding object-record scan uses that same Store session and ledger. Scanner-owned path/document/error evidence stays borrowed until the report destination has an exact Retained reservation. `DiagnosticScanWork::publish_retained` consumes its work view and closes fallible materialization, exact permit consumption, and one infallible destination publish into a Store-constructed opaque outcome, so an owner-bearing success cannot be replaced by a later visitor error. A failed visit destroys any prepared cache seed before releasing that seed scalar and then rolls back other visitor growth. Malformed, unreadable, and corrupt records preserve bytewise order and later cache priming without a second read/hash. Opened-registry entry diagnostic destinations for malformed JSON and five missing required fields are also accounted separately; extracted field owners, later opened diagnostics, and active-state collectors remain CFR-168 residuals.

`migrate check` and `migrate dry-run` never acquire the repository lock or mutate repository state.
`migrate apply` first completes a full read-only compatibility pass. Only a compatible repository may
bootstrap a missing `locks` directory through the retained `.codefire` capability; it then acquires
the ordinary persistent repository lock and recomputes the authoritative plan. The bootstrap is the
minimal prerequisite for taking that same lock and is not permission to repair another missing path.
Every executable action is reconstructed from the shared `AutoCreate` registry, matched to its exact
repo-contained absolute path, prevalidated as missing, and created no-follow relative to the retained
`.codefire` or `objects` parent. Repository, `.codefire`, and `objects` identities are rechecked before
and after use. A blocker, symlink/non-directory, forged outside action, parent replacement, or locked
plan change fails closed. A directory that is already present before planning is an idempotent no-op;
appearance after the locked plan is a concurrency failure rather than silent adoption.

Migration target names `current` and `v0.6` select accepted compatibility profiles only. This command
does not write repository/object format metadata, and reports that limitation explicitly. A validated
branch head without `semantic_validation_profile` is readable legacy debt and produces a warning; a
current-profile bridge head clears that warning even when its parent remains legacy.

## Remote Publication Flow

`remote::project` owns strict decoded components, stable project identity, contained authority paths,
and bounded no-follow project I/O. `remote::locks` owns the persistent shared/exclusive project gate
and stable cross-project order. `remote::operations` derives canonical upload/MR/review/apply write
sets and immutable results; `remote::publication` is the only Rust reducer allowed to publish those
mutable authorities.

A missing remote project is prepared below the projects namespace as an unpublished, gate-locked
stage. The ledger-aware preparation charges bounded stale-stage recovery on the request ledger and
reserves fixed bootstrap work before creating the stage. Its visibility owner retains the opaque
ledger identity plus fixed and caller-supplied object/authority cleanup permits. Explicit cancel uses
bounded capability-relative quarantine/removal; the fail-safe `Drop` path only performs an
identity-fenced quarantine and leaves recursive work to the next bounded recovery. The hidden gate
and visibility owner are borrowed separately so publication planning never requires a self-referential
owner. Publication prepares every identity and final path label before its sole `NOREPLACE` rename;
the post-rename tail only flips ownership state and adopts prepared labels.

HTTP upload validates the generic 16 MiB wire body before dispatch. For a missing project it then
validates the caller-supplied `BootstrapStageFootprint` after the existing-project fast path and
before namespace preparation, so staging overflow returns typed 413 without creating the server
namespace, hidden gate, project layout, nonce, object stage, branch, or generation. The parser's
live-body Retained charge now shares a ledger-identity owner with the `Value`; method/path use a
parallel exact owner before header scratch release. Production upload/apply select their fixed route
without heap vectors, pre-reserve decoded routing strings, and move body plus routing into
`HttpMutationRouteIntent` without cloning. The intent also owns destination-first accounted
upload/apply path, string, publication, response, and apply-identity payloads. It releases both
routing owners before Store; binding structurally rejects live routing or incomplete derived payloads
and compares Retained with the exact canonical-request plus derived-owner sum. The combined
transaction adopts the real payload owner for post-gate signature/nonce and Fresh-only planning,
then encodes responses and drops the intent before seal/selection. Missing-project
creation/rollback/visibility permits remain explicit external owners across this Store boundary.
For production upload, auth-only preflight of the full canonical request also returns opaque evidence
bound to that ledger. After planning, the intent reserves typed destination slots before allocation,
moves object values out of its request into Store overlay documents, and transfers the same Retained
scalar. Store conversion removes the four core fields from each owned JSON map and reuses the remaining
map as extensions, so neither core strings nor unbounded historical extension keys/values are cloned.
Final signature/policy/nonce validation accepts the now-empty object array only when a streaming
canonical digest proves every other request field is unchanged. The borrowed unit-test adapter cannot
consume its caller and therefore remains a separate pre-reserved clone path. Store reserves one checked
Temporary `usize` slot per document before allocating its contiguous duplicate index, sorts those slots
in place without heap allocation, and preserves input-order validation/first-repeat diagnostics through
binary lookup. It no longer materializes a per-ID clone or separate per-document size vector.

Every reducer call, keyed or unkeyed, stages bounded postimages and publishes one fixed pending
transaction before its first generation/branch/MR write. Retry validates exact owner, request,
pre/postimages, object roots, progress, and immutable result; keyed success adds a scoped v2 receipt.
File and HTTP are adapters over that model, and the Python fallback writes and replays the same
persisted contract. Cross-runtime replay tests exercise actual Rust and Python processes in both
directions rather than relying only on a digest fixture.

The core work ledger now exposes an opaque identity for ledger-independent prepared owners and
ledger-bound publication permits: upper-bound reservation
protects capacity without recording work, exact consumption records the realized work, cancellation
returns unused capacity, and two different-phase permits can be consumed through one atomic pair
preflight. The prepublication seal disables ordinary accounting, while a side-effect-free open-permit
witness guards ownership transfer to the response writer. Existing-project HTTP upload owns
nonce-cache, object-buffer, reducer postimages, Store proof, and dual responses in one prepared
mutation owner. Missing-project upload uses the same request ledger to preflight the exact hidden
object/authority footprint, commit the hidden effects, select the terminal response, and arm the
bootstrap cleanup/visibility permits. Its callback creates a private validated handoff before the
sole no-replace rename; after that visibility point the tail only adopts prepared labels and moves
the validated ledger/response parts, so no fallible validation or allocation remains. Full-wire
signed upload reaches durable object/branch/nonce/receipt publication instead of the former
parser/Store 400 boundary.
The raw Rust JSON reducer also stores that opaque ledger identity and checks it before any pre-seal
authority reread or accounting. Its plan and permit remain in an in-place owner on rejected
preconditions. The owner retains the concrete validated-root token rather than transferring only its
byte count; cancellation and commit drop both reducer plan and token before releasing retained
ownership, while cancellation preserves a retryable retained amount if release fails. The
request-wide owner, not a bare raw plan, is the eventual production boundary.
Production planning accepts a borrowed provisional Store subject and therefore does not clone a
second root token: the bound session is first detached into the Store proof plus its ledger, then the
proof lends a non-ledger provisional view to the concrete JSON planner. That proof remains the
combined owner's root evidence. The finalized root-token adapter is
retained only for compatibility callers that have already completed a read-only Store session.
Store exposes that transition as a `BoundPrepublicationValidationSession`: it privately owns the
session and binding and cleans both on drop. Detachment yields
`PrepublicationSealedValidationProof` plus the ledger; the proof verifies the exact ledger, active
state, seal state, bound-root membership, and retained documents before lending the planning view.
Once planning is complete, the proof can move its overlay documents into a same-ledger
`PreparedObjectRecordBatch`. The destination and batch-only Retained bytes are acquired before the
move; compact document ownership is transferred as an existing scalar, and no raw document vector
escapes Store. The proof keeps filesystem fingerprints while the batch keeps the documents alive
through batch and proof pre-seal revalidation, so the former second JSON clone is unnecessary.
Explicit cancellation drops binding/session payloads before releasing each accounting dimension and
returns the finished ledger. A failed release returns a recoverable owner with scalar-tail retry;
unhandled raw-drop cleanup failure panics, or aborts during an existing unwind, rather than losing an
unfinished ledger in release builds. The narrower unfinished cancellation variant returns only after
Store-owned payloads/scalars are gone, the ledger is nonterminal, and every non-Retained current
dimension is zero; caller-owned Retained intent and bootstrap permits remain external owners.
Multi-record graph
planning separately uses one `AccountedObjectStoreReader`, so every exact record read shares a
pinned root/subdirectory set and one closing ambient identity fence.
Prepared object batches additionally hold one stable kernel-locked
`.codefire-stage-v1.lock` per object root from recovery through commit/cancel/abandon. Versioned
private stages are recovered only after a bounded, polled, ledger-accounted, no-follow scan has
validated every candidate; unknown/future/malformed, multi-link, content-mismatched, or name-swapped
candidates remain untouched. The Store uses the workspace's existing exact `fs2` version rather than
maintaining separate Unix and Windows locking implementations.

Local branch listing owns one compound operation across a pinned `.codefire` capability, relative
no-follow branch capture, present/absent opened-registry snapshot, immutable result projections,
Store sealed-head validation, final identity/revalidation fences, metrics projection, and encoded
output. JSON/text output is counted before allocation, reserves exact Retained bytes, and performs
one final stdout write. Subject drift releases every snapshot/projection and records
`subject_changed`; the root traversal depth is zero.

MR creation materializes a versioned immutable subject and target-local source graph escrow while
holding ordered project gates. Review binds the subject digest. Apply consumes only the reviewed
escrow head and publishes generation, target branch, and MR as one transaction. Shared readers check
the pending marker before dereferencing authority; show/diff snapshot the gated graph into a private
temporary object store before longer rendering work.

Python GC is the current executor for the same gate's durable GC transaction. Planning is bounded and
fingerprinted from current policy, typed roots, complete inventory, retention preimage, and trusted
planning time. Deletion uses a plan-owned quarantine and progress journal; exact per-plan JSON is the
audit authority and JSONL is derived. Non-cooperating same-UID filesystem mutation outside the project
gate is not claimed as an isolation boundary; the supported local threat model requires all CodeFire
writers to participate in the gate, while no-follow/identity checks fail closed on detected swaps.

## Required Invariants and Current Gaps

- Canonical bytes/full digest/public ID must agree across every writer and reader. Existing-object reuse compares the complete requested and stored identities; a shared prefix alone is never sufficient.
- A branch head must name a valid sealed commit and remote mutation must not weaken this. Scan validates its consumed base, while local commit validates its complete candidate before the branch-head visibility point. Request signature v2 binds the full canonical mutation payload and identity fields; key-window/HMAC validation precedes nonce publication, and nonce-capacity failure never evicts a live replay entry.
- Validation happens before plan application; batch inputs validate all items before the first write.
- Branch/opened/open-marker/active identities must agree through `VerifiedOpenSnapshot` or the command returns structured corruption/recovery guidance before ordinary mutation.
- Mutable metadata and open-state mutations must acquire the relevant lock and use collision-resistant temporary writes plus replace/sync behavior. Direct live scan/verify and nested mutable callers share one explicit lock owner. Their state transaction removes its marker only after persisted fires/scan/verification match the current computed authority, scan base agrees with registry base and branch head, and the derived state agrees across registry/active/branch. Some immutable content-addressed evidence writes intentionally rely on store publication rather than a repository-wide lock.
- Idempotency keys must replay the same payload/result and reject a different payload. Import binds operation, normalized target scope, branch/mode semantic payload hash, key, and result; remote v2 receipts bind actor/operation/key plus request/result/effect hashes and replay across transports/runtimes; legacy result-only markers fail closed.
- JSON success/failure uses the stable envelope and exit taxonomy; bounded output reports omission.
- Trace endpoint failures retain typed `missing_atom`/`ambiguous_atom_id` taxonomy while bounding detail count to 100 and each ID sample to 256 UTF-8-safe bytes; link batch adds total/omitted fields rather than silently dropping excess detail. Its plan items and live added-link output are also separate 100-item samples with full count/omitted fields and the same bounded projections.
- Root/path/symlink containment must be rechecked at the use boundary. Open target resolution rejects traversal, symlinks, boundary-prefix collisions, repository overlap, and existing-open overlap; capability-relative writes and final identity checks close the validation/use race.
- Verification/evidence command execution obeys subject/input/environment/cwd/resource profiles, redacts portable output, and records the platform cleanup scope; lack of OS isolation remains explicit as `local_unisolated`.
- AI orchestration remains outside these modules.

## Review Hotspots

Treat changes to store identity, `run_commit`, branch/open lifecycle, remote mutation, signatures, path helpers, automation envelopes, or repository layout as Ultra review. Require focused regression evidence before refactoring a semantic path. Use `change-playbook.md` for read sets and validation selection.
