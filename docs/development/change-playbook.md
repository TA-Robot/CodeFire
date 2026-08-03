# CodeFire Change Playbook

Status: canonical

Owner: Engineering Steward

Update trigger: a common change path, validation command, ownership boundary, or invariant checklist changes.

This is the task-oriented entry point for changing CodeFire. Architecture belongs in `source-architecture.md`, exact command routing in `command-source-trace.md`, and persisted/type contracts in `source-data-model-catalog.md`.

## Ten-Minute Orientation

1. Read `AGENTS.md`, the development hub, and the relevant strategy decision.
2. Run `git status --short --branch`; do not assume a clean tree.
3. Identify the command or domain symbol in `command-source-trace.md`.
4. Read the owning module and its focused tests before `main.rs` broadly.
5. Name the invariant and a regression that would fail if it broke.
6. Generate validation suggestions:

```bash
python3 docs/development/tools/suggest_validation.py <changed-path>...
python3 docs/development/tools/probe_validation_capabilities.py
```

7. If delegating, resolve a context pack and use a typed task packet.

## Read Routes

| Change | Read first | Then inspect |
|---|---|---|
| Atom extraction / Trace policy | `codefire-core` public structs and builders, `docs/09_index_trace_policy.md` | core tests, scan callers, hash-schema compatibility |
| Object identity / sealed commit | `codefire-store`, `docs/08_object_store.md` | commit construction, doctor/migration, compatibility fixtures |
| CLI option/help/capability | `command_registry`, `completion`, `cli_model` | parser/dispatch arm, docs tests, JSON failure path |
| State/scan/verify | `source-architecture.md`, `open_snapshot`, core scan/verification | state-transaction v2 orchestration, `verification`, automation renderers |
| Open/clone/close/import | relevant trace row, `open_snapshot` and lifecycle transaction owner | idempotency module, path policy, branch/opened/active records |
| Fire/extinguish/link/evidence batch | focused module; for Fire/Resolution also `fire::active_ledger_transaction` | all-items-before-write, all publication gaps, operation plans, state-transaction v2 |
| Commit | commit transaction and active-reset sections in architecture | `commit_active_reset`, open/ledger pending guards, store validators, signatures, doctor fixtures |
| Diff/show/review/patch/merge | `view` and its submodules | bounded output, object reads, merge/patch idempotency |
| Remote/HTTP/signatures | `remote`, `http`, `http_tls`, `signatures` | path/resource limits, nonce/key policy, structured remote errors |
| Python HTTP compound read route (Doctor/branch/bundle/MR list) | the route's operation owner, the shared `do_GET`/`do_POST` dispatcher, `read_json_accounted` and its authority fence | deferred per-record fault ownership, one typed filesystem-authority family, per-`(root, head)` escrow memo, poll-per-quantum policy, exact/±1 dimension bounds |
| Version/schema/capability/page | `compatibility`, `command_registry`, `pagination`, `schemas/public/` | checksum conformance corpus, Rust/Python/minimal-consumer decisions, parser/manifest agreement |
| Doctor/migration/layout | `repo_layout`, `doctor`, `migration`, shared transaction/snapshot validator | typed divergence/pending vocabulary, store registry, ledger-bound raw/parsed opened-registry snapshot ownership, recovery fixtures |
| AI development system | `ai/README.md`, schemas, model policy | strategy decision register and local AI gate |

## Recipe: Add or Change a Public Command

1. Decide whether it belongs in the product; keep AI orchestration external.
2. Add/update the shared option/result type in `cli_model` or a focused module.
3. Register the top-level capability in `command_registry`.
4. Add help/completion and ensure help returns before mutation/parser side effects.
5. Implement parse → validate → plan → apply → render; move domain logic out of dispatch.
6. Return `codefire.command_result.v1` for declared JSON support, including failures.
7. Add parser, help, text, JSON success/failure, dry-run/no-write, and path-context tests.
8. Update CLI spec, command trace, capability/source checker, and runbook if operational.

## Recipe: Change a Mutating Workflow

Write down before editing:

- validated base/head and expected state;
- the `VerifiedOpenSnapshot` subject/disposition and which exact operation, if any, may consume pending authority;
- lock/resource scope;
- idempotency payload and conflict key;
- full plan and no-write dry-run behavior;
- ordered writes and visibility point;
- interruption states and recovery/doctor output;
- JSON/text result and next action;
- focused failpoint/replay tests.

Never validate one batch item after earlier items have written. Never let rendering or dry-run take a weaker validation path.

For a durable phase marker, separate three claims: marker shape/current-phase validation, writer transition validation, and crash-window evidence. A retry must validate an existing marker and its subject/preimages before its first recovery write. Enforce adjacent transitions in the writer, but do not claim doctor can prove historical transition order from one persisted phase. For state publication, emit version 2 and bind branch/open-instance/base plus agreeing branch/registry/active pre-state. For multi-file Fire/Resolution publication, stage postimages before the first ledger write, distinguish pending request collision from current-image divergence, and test every file/progress/completion gap. Name the exact injected windows covered by tests instead of calling a partial matrix exhaustive.

## Recipe: Change Object or Format Semantics

This is mandatory Ultra work.

1. Add cross-runtime canonical/full-digest/public-ID vectors first.
2. State readable, writable, and rejected versions.
3. Preserve old field meaning; add a versioned extension or migration.
4. Test malformed payload, unknown/future version fail-closed, legacy readability, collision, filename/record mismatch, symlink root/subdirectory/final record, non-regular inputs, and size-cap enforcement including growth between metadata and read.
5. Update store validator, doctor, migration, data catalog, object spec, release/format roadmap, and compatibility matrix together.
6. Do not infer release SemVer from an internal schema increment.

For sealed snapshot changes, state commit version, root schema, every root object version, and empty-field omission rules independently. Current schema 2 must continue through the core authoritative predicate and certificate agreement; do not replace cross-root recomputation with shape-only checks. Keep schema-1 fixtures and reject unknown future schema/version values.

## Recipe: Optimize a Repository or Graph Command

1. Reproduce with a bounded fixture and record stable work counters.
2. Identify the repeated entity (object read, parse, hash, graph visit), not wall time alone.
3. Define the cache/memo key from every validity input.
   For filesystem subjects, a canonical path string alone is insufficient. Sealed-object cache changes must bind root identity plus the ordered identity-or-missing state of every bound object subdirectory, keep reads descriptor-relative/no-follow, stage entries until final identity recheck, cap extension subdirectories at 128, and retain only one current store subject. Test cap rejection without retained handles, subject-switch eviction, root/child swap-back, same-path replacement, failure-not-cached, and shared-DAG reuse. The cache still relies on immutable object files; do not claim it detects in-place rewrites of a cached record.
4. Compare success and corruption diagnostics before/after.
5. Add scale points that distinguish expected complexity.
6. Reject an optimization that weakens sealed validation or hides omissions.

For Rust full-wire upload/apply, Retained-own method/path before header scratch release, recognize
the fixed mutation route without heap vectors, and reserve decoded routing frames before
materialization. Move the canonical body and routing owners into the intent; do not clone the body.
Put every long-lived derived path/string/publication/response payload in that
typed owner, reserve its logical frame/compact bytes before materialization, and require the Store
snapshot to equal raw plus derived Retained with both routing owners already absent. Encode the response before dropping the source intent,
then release it before seal/selection. Move overlay documents from the Store proof into the
same-ledger typed batch only after planning: reserve the destination and additional Retained bytes
first, keep the documents in that batch through final raw revalidation and pre-seal staging, and do
not expose a plain `Vec` at the boundary. In the production stream adapter, authenticate the full
request and obtain opaque same-ledger payload evidence before consuming its objects. Reserve overlay
destination slots, move object values and their existing Retained scalar into Store, and prove at
final signature/policy/nonce validation that only the object array was consumed. Convert each owned
object map by moving its four core fields and reusing the residual extension map; do not clone
historical extension keys/values. Reserve one contiguous document-index slot per document before
Store duplicate validation, and keep input-order validation/first-repeat diagnostics.

For the Python fallback HTTP bundle route, keep one handler-created ledger across branch authority,
sealed validation, reachable collection, result ownership, and canonical response freezing. Preserve
the LIFO reference schedule and final public-ID order, and test limit boundaries, deep/shared DAG,
cycle, missing/corrupt record, projection failure, zero-current terminal, and no-partial-200 behavior.
Its optional deadline stays outside deterministic counters. Poll between bounded branch/object read and SHA-256
chunks, during canonical hash/retained-size walks, and at validation/order/projection/response boundaries. Test exact
expiry, mid-read/hash/canonical/final-hash cleanup, terminal precedence, and no partial 200. Contiguous stdlib
`json.loads` remains non-interruptible; do not claim full deadline support or reject otherwise-valid large single
string/float tokens. Do not infer mutation preflight, request-header/body, other-route, or connection-admission
deadline coverage from this read-only slice.

For Python fallback HTTP upload server verification, preserve sealed validation and signature policy before verification and
publication identity/digest/ancestry/layout/nonce after it. Pass the handler's exact request ledger and deadline into the existing
single-use `AccountedServerVerification` path; a no-ledger caller must retain the compatibility wrapper's local ledger. Test
same-ledger exact-minus-one before spawn, request deadline before layout/nonce, spawned-process cleanup, release-fault retry and
error chaining, legacy no-ledger execution, and earlier validation-error precedence. Preserve the exact operation exception as the
outer object; append the first cleanup exception as explicit cause, followed by cleanup then primary prior effective chains with
identity deduplication and no cycles. Before any release observer, detach traceback locals holding config/output/process payload.
Close every owned plan capability once in a bounded sweep even after an earlier fault, without retrying detached FD numbers.
Configured commands can create external
effects before failure; never describe their cleanup as publication rollback.

If a later Python upload owner borrows validated commits, use a purpose-specific single-use handle from the sealed owner and allow
only commits already retained in the validated session; never widen authority through `load_payload`. Because the payload is a raw
mutable alias, claim its canonical-fingerprint destination before exposure, verify at handle close, retain it through disk
revalidation, then perform a callback-free final canonical check and immediately detach that document before scalar cleanup.
Make lookup, release, and finish single-flight; after any callback-rich fingerprint work, recheck handle identity and owner/handle
tokens before obtaining the payload local. Test same-size nested and post-close mutation, callback reentry, stale/revoked handles,
foreign/unknown authority, payload-before-release, final-fence ordering, and retry faults.
When composing the signature purpose with pinned policy/verification planning, use a two-stage owner so prepare failure leaves retry
authority with the caller. Bind both parents with coordinator tokens, pass the exact preallocated handle as the signature loader,
and keep the authority construction guard active through signature-child close and owner-first handle release/revalidation.
Verification config/cwd/child publication may start only after that midpoint release. Abort cleanup is active borrow, policy authority,
then sealed authority and stops before sealed cleanup if policy cleanup fails. This remains production-unwired until a terminal
coordinator owns verification execution, both final fences, ancestry, bootstrap, and publication ordering.

Do not compose the signature and ancestry handoffs by clearing parent tokens or closing the signature composite. A valid
existing-project terminal owner must acquire one top-level token before either child, proxy signature/verification/ancestry,
and retain both parents through one gate-held terminal phase. Verification failure is abort-only even though subprocess effects
may already exist. Run every reject-capable read/poll/accounting/preflight before a callback-free pair recheck and detach; neither
`policy.revalidate` then `sealed.finish` nor the reverse is sufficient, even under the cooperative project gate. Abort cleanup is
ancestry child/borrow, verification child, policy authority, then sealed authority with stop/retry. Without that pair API, keep
production wiring frozen and add only characterization or a production-unwired prerequisite.

The production-unwired existing-project prerequisite closes consumer lifetime and the pair-detach primitive with
`AccountedHTTPUploadExistingTerminalOwner`: claim the shared ledger registry before either mirrored parent token, return no child,
and stop before verification spawn if callback close was already requested. Every later failure is abort-only. Close ancestry child
before its borrow, verification/partial signature children before the residual signature borrow, and policy before sealed; preserve
the exact field and stop on the first cleanup fault. After ancestry, perform callback-rich preparation policy then sealed, place the
last deadline callback before the critical section, probe sealed then policy using only captured primitives and preowned/accounted
destinations, and commit neither side until both probes pass. Logical-detach cleanup is policy then sealed, stop/retry, and must not
rerun probes. Resolve every probe/commit/helper/FD function before the first prepare callback; no mutable class lookup belongs in the
critical interval. Keep the successful resource-zero `PAIR_DETACHED` owner and its clone-free exact immutable verified branch/head
references as the exact ledger capability until explicit close. Capture those refs before preparation, recheck the sealed head after
the last callback, and restore only the captured refs during field-only commit. This owner still has no stable project ID authority, semantic input,
bootstrap, immutable effect input, effect, publication, or handler method.

For the existing-project stable-identity prerequisite, call `prepare_http_upload_existing_project_gate` before parser, semantic, or
sealed ownership exists on the request ledger. Keep its two-stage path/full-owner preaccount factory: owner allocation, project open,
descriptor read/parse, scratch, and persistent descriptors must not move ahead of their upper. Validate canonical org/app and exact
lowercase stable ID from a bounded single-link descriptor, hold a no-follow single-link exclusive gate, and compare named/opened
project/locks/gate/descriptor identities and bytes at the terminal probe. Stable identity flows only through the gate's semantic
factory; a gate-backed semantic must never enter the legacy carrier constructor without the exact gate/token. Include the gate
Retained/Temporary scalars in the sealed baseline and prove the gate ledger epoch precedes sealed construction. Invoke the captured
gate probe after the pair's last callback but before semantic probe and pair commits. Abort/close order remains pair, semantic payload,
parser/Temporary/primary/control, then gate. Capture exact gate instance/class/global/OS dependencies before that callback; after it,
perform a hook-free drift check before any instance dispatch and use only captured primitives for the subject/content probe and exact-FD
exclusive-lock reassertion. Before `flock`, read, unlock, or close, classify every retained numeric FD by captured device/inode.
Unconfirmed ownership defers automatic cleanup; explicit cleanup touches exact original inodes only and field-abandons foreign/EBADF
numerics. If class/state drift makes cleanup dispatch unsafe, retain ownership until restoration and explicit retry. Keep
handler/effect/publication and missing bootstrap unwired in this slice.

Before the existing-project pair's final callback, preaccount the effect-input
control together with the carrier control in the semantic secondary scalar,
then bind `AccountedHTTPUploadExistingEffectInput` as a destination-first owner.
Replace the carrier's private semantic commit only with captured closures that
first validate the exact carrier/owner/class/registry slots, run the original
callback-free commit, and move the fresh mapping proxy/copy tuple without a
getter.  Post-commit cleanup and the final slot move must be separately
retryable without another deadline callback.  Registry ownership moves to the
effect-input owner only after pair/semantic cleanup succeeds; its close drops
the input, lets carrier shutdown release the combined preaccounted control
after pair/semantic payload cleanup, closes the stable gate last, and then
verifies the completed control release.  Keep effect execution, publication, response
selection, missing bootstrap, and handler wiring outside this prerequisite.

Do not attach the current mutable HTTP mutation response pair directly before
or after that terminal boundary. Preparing it before terminal leaves its
public header/body fields outside the last callback recheck; preparing it after
terminal runs the response deadline callback after the project subject was
accepted. A normally accounted response allocated after sealed construction
also invalidates its exact ledger baseline. Freeze the next composition until
one destination-first owner preaccounts the success/durable response pair in
that sealed baseline and owns it before the last callback,
adds captured exact response state to the callback-free terminal probe, and
adopts both response selection and effect input under one token. Publication
preflight evidence must join the same final probe before this owner can expose
an effect-start transition.

Use `prepare_http_upload_response_input` as the preparse response prerequisite.
Its factory acquires both success/durable body and header uppers plus runtime
control before allocating the owner, while leaving exact response/header work
reservations open. After parsing, `prepare` accepts exact immutable bytes,
charges the maximum selected body/header work, cancels unused reservation, and
shrinks that same scalar to both payloads plus control. It exposes neither
bytes nor selection. When opted in, semantic adoption reconciles the parser
transition's predecessor from response upper to exact without changing ledger
current, then sealed proof pins the response transition/scalar/token. The pair
carrier captures response instance/class/byte identities before the last
callback and invokes that callback-free probe before commit. Class drift must
defer automatic cleanup until restoration; never dispatch a replacement.

For the missing-project namespace transaction prerequisite, construct
`AccountedHTTPUploadMissingProjectBootstrapTransaction` only after the organization exists and before the final project name does.
Keep target bootstrap-lock, hidden stage, stage gate, descriptor, and their accounting on one request ledger. Seal first, then use
only the returned typed handoff for effect-start, the exact pinned-directory no-replace rename, and durable reducer. Never close and
reopen the gate around rename, never rerun rename after the final inode is visible, and never delete the final project during reducer
retry. Before effect-start, cancellation may remove only the identity-matching hidden stage; after effect-start, close must fail until
the reducer reaches `COMPLETE`. Keep semantic/request-hash/write/response/copy proof adoption and handler wiring outside this local
transaction slice.

For Python upload object-set digest, do not use Stage sealed bounds: they cover closure/core/newline, not the full request object list.
Transfer the parser-owned exact full-request canonical scalar to a destination-first same-ledger semantic owner with no digest getter.
Reserve separate Retained bytes and a source-proportional canonical-encoder scratch envelope for immutable copy/fingerprint input.
Existing projects keep a continuous gate through
recovery/current/ancestry/preseal→terminal→effect. Missing projects hold bootstrap authority, build and gate a private stage before
terminal, then effect-start→exact rename→durable reducer with the same gate token. Terminal detaches semantic and immutable copy
inputs; post-terminal copy never rereads ambient records. The production-unwired Python prerequisite acquires the pre-parse upper
into a caller-owned one-shot `AccountedHTTPUploadParserScalar`, reconciles it in-place to the exact post-parse scalar, moves that
exact token through `AccountedHTTPUploadRequestSource`, returns `AccountedHTTPUploadSemantic` before explicit single-use prepare,
and keeps the digest, exact mapping, record identities, and effect-ready core-plus-newline bytes private. Its last deadline callback
precedes callback-free full-request fingerprint/record revalidation and field-only detach. A future handler wire must move the real
parser release slot into this token chain exactly once and consume the read-only mapping proxy/copy tuple without adding a
digest/copy getter.
For the callback-free semantic prerequisite, construct and capture `_HTTPUploadSemanticTerminalProbe` before the caller's final
deadline callback. Prelink every container child to an index in an exact immutable tuple plan; do not retain a mutable identity map
or sort after the callback. Validate captured instance-dictionary key identities before any fixed-key lookup, then stream the full
request, full sorted records, and root-`remote`-omitted core bytes through preallocated frames. Keep it single-use, revoke views on
success/failure/close, roll back partial publication, and do not treat this production-unwired probe as detach, effect authority, or
handler wiring.

## Recipe: Synchronize Documentation

1. Classify the target as canonical, supporting, exploration, historical, generated, or superseded.
2. Update the one authority; do not copy current counts into stable entry pages.
3. For deletion, record unique-content disposition and inbound migration in the deletion manifest.
4. Run project docs and source docs checks.
5. Preserve old event facts in append-only history; do not leave old instructions live.

## Invariant Review Checklist

### Identity and storage

- canonical serialization and full digest unchanged or versioned;
- current and legacy public IDs handled explicitly;
- immutable records validated before use;
- commit/root schema versions fail closed except for explicit schema-1 compatibility;
- schema-2 Atom/Trace/Fire/Resolution/policy/verification pass the core authoritative predicate, and certificate agrees with verification;
- committable validity comes from the typed core decision; safety-floor facts cannot be disabled by
  policy, every reason has an exhaustive exit/diagnostic/action/text mapping, and only missing-link
  completeness may be a warning;
- active text/binary/semantic merge conflict metadata is a typed exit-15 blocker; only the locked,
  pinned-worktree `merge --continue` transition may clear it after marker/sidecar/acknowledgement checks;
- `missing_evidence_refs` omission is accepted only as the documented empty compatibility case;
- object writer/read/sealed/full-scan roots and subdirectories are no-follow capabilities, records are capped at 64 MiB, and publication/snapshot identity is rechecked;
- a multi-object commit predeclares every blob/root/candidate/branch-snapshot role before its first store and marks each exact ID stored before the mutable visibility point;
- no reachable object deletion or silent alias.

### Branch and workspace state

- branch head sealed;
- branch/open marker/opened registry/active state identities agree through one `VerifiedOpenSnapshot`, including base/head and state agreement;
- a documented pending commit/state window is typed separately from steady state; only the exact lock-owning reducer may consume it, while other consumers return `open_recovery_pending` without writes;
- open work is never silently discarded or rematerialized over;
- failed open materialization leaves the target absent or preserves the same empty target, cleans only an exactly owned sibling stage, and can retry without adopting a foreign stage;
- close/discard reads the marker through the pinned target, removes only the bound marker for keep-dir, and quarantines plus post-validates the exact target before recursive no-follow deletion; identity failure preserves replacement content and precedes repository-state cleanup;
- dry-run does not mutate;
- worktree publish/replace has one supported-platform rename visibility point; separately assess the later opened/active/branch publication and its interruption states.
- commit active reset authenticates every removal preimage and the clean state postimage before branch/opened publication can be finalized; a legacy partial reset without those snapshots is inspection-only.

### Automation

- success and failure envelopes use `schema`, `command`, `ok`, `exit_code`, `repo`, `data`, `diagnostics`, `next_actions`;
- payload-specific `type` stays inside `data`;
- capability target/schema/pagination claims come from the generated registry and checked public schemas;
- page cursors bind collection plus complete-result snapshot and reject mutation, cross-collection replay, malformed/oversized encoding, or limit changes;
- metrics stay under `data.metrics`;
- bounded output reports truncation/omission;
- exit code and diagnostics describe the same failure.

### Security and recovery

- repository containment rechecked after path resolution;
- filesystem writes use a pinned/no-follow subject where a check/use rename or symlink swap can redirect an ambient path;
- Fire/Resolution dependent files use one bounded staged ledger reducer; exact replay validates effect hashes, another pending request is a conflict, and an unexpected image is preserved rather than overwritten;
- symlink/temporary-file/lock races considered, including crash release, persistent-lock inode ABA, hard-link provenance, immutable lock contents, bounded/no-follow owner sidecars, and foreign reserved-file collisions;
- reserved temp names distinguish unpublished scratch from published authority: import `.part` is preserved/non-authoritative, bootstrap `.tmp` is fully formed, and write `.tmp` is recoverable only when its transaction ID matches;
- evidence commands default to a minimal environment, redact command/argv/output and sensitive local paths, store no raw environment values, and label profile/phase/trust/portability plus subject/input/environment digests;
- remote mutation signature v2 authenticates the full canonical payload plus actor/operation/target/key/idempotency/time/nonce; both key-window edges fail before nonce publication, and cache saturation never evicts a live replay entry;
- remote authority components are validated before any layout/nonce/lock write; all mutators use the exclusive project gate plus durable publication, all compound readers use the shared gate plus pending check, and cross-project gates use stable order;
- remote reviews bind the versioned immutable MR subject, apply consumes only target-local reviewed escrow, and upload/apply generation/branch/MR/result heads agree;
- remote branch/MR/object/public records pass the shared support decision before semantic fields are consumed; unknown critical extensions fail closed while noncritical descriptors and references survive transfer;
- GC planning binds bounded policy/root/inventory/retention fingerprints and a non-future trusted planning time; exact candidate quarantine/progress and exact JSON audit recover before pending release, while JSONL remains derived;
- verification/review/context/explain/certificate use the applicable versioned assessment subject; live subjects bind the canonical worktree manifest and are rechecked through branch publication, while certificate/verification subject identity and exact variant fields are sealed and cross-checked;
- resource/command limits drain both streams during execution, report bytes seen/stored/truncation, and test timeout cleanup including descendants where the platform claims process-group termination;
- a compound read route that pins a whole authority snapshot must not let a later record's read,
  no-follow, parse, or version fault pre-empt an earlier record's validation fault: retain each
  record's own capture outcome and raise it in filename order at the point the per-record route
  observed it. Multi-fault fixtures in both orders, not single-fault fixtures, are the evidence;
- no route may answer an unusable filesystem authority with a zero-byte reply. Map every no-follow
  rejection, symlinked/directory authority, unavailable path, permission denial, metadata failure, and
  failed read into one typed 400 family, and keep the shared `do_GET`/`do_POST` arm so an escaping
  `OSError` cannot close the connection silently; only client transport failures and commit-source
  cleanup faults still abandon it, and a cleanup fault does so only after the request loop has
  accounted the drained ledger. Following an authority symlink is not compatibility worth preserving;
- memoize repeated pinned traversal/digest work by its exact pinned identity, prove the object
  traversal and hash counters become constant rather than linear at several cardinalities, and prove
  the constancy claim non-vacuous by removing the memo. Keep the final fence over every reused result;
- reduce duplicate deadline polls to one per bounded work quantum without reducing cancellation
  coverage: prove a long single traversal still cancels at many boundaries with exact phase
  attribution, and prove each production poll is required by silencing it. Enumerate poll sites from
  live production frames, not from the phase helper that implements them, and silence exactly one site
  at a time: replacing a whole helper cannot distinguish two adjacent redundant polls;
- never infer a missing leaf from an unavailable parent. Pin the parent directory capability
  no-follow, record its identity in the capture, and record absence only when that proved parent
  returns the supported leaf-absence result. Reprove the parent at the final fence: an unreadable
  parent is never evidence of continued absence, and a stale record must fail closed rather than be
  silently downgraded;
- classify failure origin at the boundary or owner that raised it, over one taxonomy that is exhaustive
  across transport, filesystem and commit-source cleanup. `TimeoutError`/`ConnectionError` are raised
  both by a client socket and by a remote filesystem, so mark transport at the real socket transfers,
  filesystem at route execution, and commit-cleanup at the cleanup owner that drained the fault, and
  share one classifier between GET and POST — extend that classifier with a new origin rather than
  adding a second route-local one. A string, errno or
  exception-class heuristic is not evidence and does not survive dispatcher composition;
- cleanup keeps retry ownership until its release succeeds. Never zero a scalar, remove a session,
  detach an owner, or clear a map first; make cleanup idempotent so a second call releases only the
  remainder; never swallow `BaseException`; and never publish a success while route ownership,
  temporary bytes, or open permits survive — name the residual terminal state instead;
- reprove every captured metadata field after the final byte read, the EOF probe, and all interruptible
  digest work, and bind the digest to that terminal observation. Mutate mode, hardlink add/remove,
  symlink replacement, size, identity, and content at each boundary; where a boundary is physically
  undetectable because the fence already owns and hashed those bytes, document that outcome as the
  pinned snapshot instead of claiming a rejection;
- prepare and encode the complete accounted response before the terminal authority/session/project
  fence, run cleanup before publication, and leave nothing authority-dependent, callback-capable, or
  input-proportional between that fence and the first response byte. Do not claim the authority fence is
  the last *fallible* boundary while a pinned lock or descriptor is still unlocked or closed after it:
  place the final publishable proof after every context-manager teardown, owned by the caller that
  actually publishes, and prove the ordering with an oracle that observes `flock`/`close` and not only
  named methods. Define the publication point exactly and transfer one prepared-response owner without a
  second encoding or ledger;
- make the publishable proof a production transition, not an advisory check. A guard whose only effect is
  a flag some test reads cannot enforce anything: gate the owner transfer on the proved state, so an
  unproved response cannot reach the wire. On a failed proof, release the prepared bytes, make the ledger
  terminal for the exact cleanup reason *before* selecting any response, keep the typed diagnostic
  observable, and publish nothing rather than a success;
- displace a prepared response before selecting another one. Every error arm reached while a response
  slot already owns bytes must dispose of that owner first, or a post-route fault is silently replaced by
  the older prepared success. Specify the precedence when disposal itself fails — unknown ledger state
  means nothing is published and the fault is recorded — and keep the shape identical across GET and
  POST so a successor cannot inherit only half of it;
- replace defensive silence in accounting with a typed fail-closed error. An exact release larger than
  the owner still holds is a defect: returning quietly masks the next double-release. Making it typed
  found a real aggregate-plus-exact double release in session cleanup that the silent form had hidden;
- scope every oracle to what it measures. If a claim is route-wide, instrument every physical read or
  poll site and assert the exact protocol with named owners; otherwise name the helper boundary in the
  assertion. An overclaimed test is not evidence;
- do not keep write-only evidence. A counter that is only incremented and a property that is never read
  prove nothing: consume the diagnostic on the production path — as a note on the primary error, or in the
  handler's own error log — and assert both that a faulted request emits it and that a clean request does
  not;
- record every new regression that passes on the revision it was meant to falsify, with the reason. A
  legacy pass does not erase a candidate-added false guarantee, and an unexplained pass reads as a hidden
  gap;
- never let a fixture-derived identity choose which record an expectation covers. Record IDs derived
  from content digests over real commit objects vary between fixture builds, so selecting a subject by
  sorted-ID order and then asserting a persisted field reproduces only intermittently. Derive the
  expectation from the same rule the route applies and add a deterministic case for the interesting
  record. One such latent expectation in the merge-request-list cardinality case was the identified
  cause of an intermittent focused-module failure: it published the correct `stale` status while the
  test expected the persisted `open`, and it reproduced roughly once in five whole-module runs;
- a live-socket harness must tolerate a deliberately abandoned transport. When the server abandons the
  connection, the client's half-close can raise `ENOTCONN` and its read can raise `ECONNRESET`; treat
  both as end-of-stream and keep asserting the exact wire bytes, or the harness itself becomes the
  intermittent failure;
- a terminal HTTP ledger is never reopened for diagnostics: route-owned payloads first return to request-header-only current/open-zero, then one fixed bounded control-plane response is written; append, truncate, replacement, encoder/writer failure, and budget-vs-subject precedence remain fixtures;
- pending/stale/corrupt states lead to safe, explicit recovery; doctor reuses the typed snapshot/transaction validators and recommends inspection plus only the exact owning retry.

For Python fallback upload ancestry, preserve legacy breadth-first parent order and complete traversal after a match; an early
boolean return changes which later corrupt parent wins. Pre-reserve seen/deque bases, every frame/seen entry, conservative
per-insertion storage, and old-plus-possible-new backing overlap. A transition and child-depth integer are allocation payloads:
consume their upper through the owner's fixed pre-existing gate before constructing them. Keep CPython set/deque growth and
popleft-churn probes aligned with those constants. Production must borrow commits from a still-live sealed-validation session,
complete its final subject fence before publication, and clear loader/parent/loop payloads before releasing same-ledger scalars.

For production Python upload sealed validation, move only immutable scalar bounds from a sealed Stage: record count, aggregate
serialized bytes, and maximum serialized record bytes must cover both incoming and existing staged records and move exactly once.
Prepare, validate, finish, and retryable close stay on the handler request ledger/deadline. Preserve the no-ledger legacy path and
the later signature/verification/digest/ancestry/publication order; test mixed bounds, exact-minus-one, deadline no-effect, final
staged-tree fence, and cleanup faults before claiming production coverage.

Never create a `DecisionWorkLedger` inside a live HTTP request. `validate_sealed_commit` takes an optional caller ledger/deadline;
thread the request's exact pair, or -- when the request already validated the same closure -- pass the request-owned
`RemoteJSONPublicationRootProof` and reuse the resulting `RemoteJSONPublicationIdentity`, so no root is traversed twice. A caller
with nothing to thread must keep the legacy positional call shape verbatim, because fault tests stub that signature. Put a deadline
poll immediately before *every* first externally visible effect, including the missing-project layout bootstrap, and before the
response slot marks the effect started -- otherwise an elapsed request answers 503 or 200 instead of the typed deadline outcome.

Give every `SealedCommitValidationSession` an explicit owner and claim its `Path`, memo, sets, registry, reuse pool, map entries and
descriptor owners before the allocator that creates them; recycle released descriptor owners so the registry cannot grow per read,
and reserve `finalize`'s ordered-id projection before allocating it. One close must detach and attempt every descriptor exactly once,
continue the bounded sweep after faults, drop payloads, release logical control, and only then return the first cleanup failure; never
retry a raw FD number. The source wrapper may keep its two-call compatibility shape, but a second call may only finish remaining
non-descriptor scalar cleanup. Before `_AccountedUploadCommitSource._open` performs a poll, allocation, acquisition, or session
construction, reconcile any published memory owner so a persistent failure retains one 11,456-byte charge rather than overwriting it
and growing to 22,912. Derive a reservation term only from a claim site that exists:
a per-record heap multiplier with no matching claim becomes an admission cliff far below the declared record limit. Stream and hash
canonical bytes with bounded scratch through `canonical_json_stream_digest` instead of reserving a whole-record encode, and prove
byte-identity against `canonical_json` when touching it. Note two frozen constraints: `_HTTPUploadSealedValidationReservation.close`
requires the ledger's scalars to be exactly its baseline plus upper, and the server-verification owner snapshots absolute ledger
scalars -- so the sealed owner's final fence must run while no other owner holds retained or temporary bytes on that ledger.
Record the real admissible record size and count, and prove the first refusal is a named resource owner rather than formula slack.

A request has more than one route to an effect. `handle_http_upload` reaches
`execute_remote_json_publication -> recover_remote_pending_quarantine -> validate_sealed_commit` on a retry, so that chain carries the
same exact ledger and deadline and polls before each recovered root, before the receipt replay, and immediately before the quarantine
cleanup. When auditing "one request, one ledger", enumerate the recovery and error routes too, not only the ordinary one. Owners that
reserve a fixed upper for one dimension must still cover the others: reserve what is count-independent, and acquire exactly what is
sized by data the traversal discovers -- a dimension left at zero silently rejects every caller at the first claim. Let the caller
ledger raise its own typed budget diagnostic for an exactly acquired dimension instead of shadowing it with an owner-level refusal.
Single-use authority objects must take their state transition *before* any callback they can reach, including a deadline poll, or an
alias or reentrant consumer borrows the same authority twice.

An accounting invariant only counts as covered when deleting it changes a focused assertion. For a claim, bind the exact/minus-one
capacity so the claim itself is the refusal and its dimension, phase and exact size are asserted. For a check, create the state the
check exists to reject immediately before it runs and delegate to the production function, so replacing that function with a no-op
stops the raise; forcing it to raise only proves it is reached. For a deadline poll, expire the clock in the gap that only that poll
guards and assert the resulting layout/effect or response difference.

Release is the other half of accounting and has its own ordering rule: never erase the claim, the amount, or the owner's
open/acquired state before the fallible ledger call. Consume ownership only for a release the ledger really made — compare its scalar
before and after — and always let the failure propagate. Otherwise a pre-effect fault strands the bytes with nothing left owed, and
the retry is a silent no-op. Keep primary-error precedence in cleanup so the fault that caused the unwind is not replaced by the
release that failed while handling it. Preserve exact exception identity and append existing cleanup-chain nodes before existing
primary-chain nodes, deduplicated by identity into one explicit cycle-free cause chain; later cleanup faults do not displace the first.

An allocation is only accounted if the charge can be *witnessed* at the moment of allocation. Route accounted collection allocations
through a named pre-allocation boundary that proves the charge — in the dimension the caller's ledger actually sees, not merely in
the owner's bookkeeping — and record what is owed only after that proof. A test that notices the missing charge after the allocation
already happened is evidence of the wrong thing.

When adding a dependency for one of these boundaries, record the reason, direct/transitive impact, rejected alternatives, and replacement/removal condition in an ADR. Current worktree, lock, and sealed-object capability decisions are ADR-009, ADR-010, and ADR-011. Local open authority plus active transition recovery is [ADR-015](../../adr/ADR-015-verified-open-authority-and-active-transitions.md), with implementation closure tracked by [CFR-163](code-review-issues/cfr-163.md). Versioned remote/assessment subjects and execution profiles are [ADR-016](../../adr/ADR-016-versioned-trust-subject-and-execution-profile.md), with delivery tracked by [CFR-164](code-review-issues/cfr-164.md). Remote project containment, publication, MR escrow, and GC recovery are [ADR-017](../../adr/ADR-017-remote-publication-transaction.md), tracked by [CFR-165](code-review-issues/cfr-165.md), [CFR-174](code-review-issues/cfr-174.md), and [CFR-175](code-review-issues/cfr-175.md). Version/schema/capability negotiation, extension criticality, conformance corpus, cursor snapshots, and remote Doctor routing are [ADR-018](../../adr/ADR-018-version-capability-conformance-contract.md), tracked by [CFR-166](code-review-issues/cfr-166.md) and [CFB-134](bug-issues/cfb-134.md). Effective verification-policy provenance, file-only ChangeCoverage, and their sealed cross-root binding are [ADR-019](../../adr/ADR-019-effective-policy-and-change-coverage-contract.md), tracked by [CFR-167](code-review-issues/cfr-167.md).

## Focused Validation Order

1. format/compile for the changed language;
2. the narrow regression protecting the invariant;
3. owning crate/module tests;
4. cross-crate/workspace or Python compatibility tests;
5. CLI/installer/dogfood smoke when the boundary changed;
6. docs/source/AI gates;
7. release gates only from a clean, source-bound environment.

When `cargo` is unavailable, report Rust source validation as blocked. A pre-existing binary may reproduce user behavior but is not evidence for modified Rust source.

## Handoff

Every implementation handoff names:

- objective and issue/task packet;
- files changed and intentionally untouched;
- invariant and design choice;
- focused and broad commands with exact result;
- blocked capabilities/environment;
- residual risk and next evidence.

For High/Ultra changes, use an independent reviewer bound to the task and patch subjects.
