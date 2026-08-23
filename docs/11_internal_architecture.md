# 11. 内部アーキテクチャ設計

この文書は、Code Fireの内部設計である。実装時のmodule境界、データフロー、不変条件、エラー処理、テスト設計の正本として扱う。

実際のsource責務と実装経路は `docs/development/source-architecture.md`、`command-source-trace.md`、`source-data-model-catalog.md`、`change-playbook.md` に分離する。この文書はproduct内部設計、4文書はsource navigationと変更手順の正本である。

## 11.1 Architecture Principles

Code Fireの設計原則:

- sealed commitだけを正式履歴に置く。
- open directoryは作業場所であり、repository内checkpointではない。
- object identityはcanonical payloadから決まる。
- branch head更新はatomicである。
- commandはparse、validate、plan、apply、renderを分ける。
- human outputとmachine-readable JSONを混ぜない。
- policy downgradeされたwarningとblocking diagnosticを同じ扱いにしない。
- remoteは信頼境界であり、受信object graphを必ず検証する。
- cacheは派生データであり、壊れても正式データを壊さない。
- 大きなfile、深いcommit graph、大量Atomで計算量とメモリ使用量が破綻しないようにする。

## 11.2 Runtime Layers

```text
CLI Dispatch
  command selection, help, completion, top-level error handling

Command Contract Layer
  shared option parsing, path resolution, JSON envelope, exit codes, next_actions

Command Modules
  status, scan, verify, fire, extinguish, evidence, doctor, storage, migrate, diff, remote

Core Domain
  Atom extraction, Trace Graph, required links, scan result, verification model

Store Domain
  canonical JSON, object ID, immutable object records, sealed commit validation

Repository Services
  repo discovery, open registry, active state, branch head, locks, idempotency

View and Diff
  commitish resolution, manifest view, file/semantic diff, review pack

Remote Services
  file-backed remote, HTTP/HTTPS transport, permissions, signatures, merge requests

Diagnostics and Recovery
  doctor, migrate, storage report, explain
```

Dependency direction must generally flow downward. Command modules may call core/store/repository services. Core/store must not depend on CLI rendering.

## 11.3 Rust Workspace and Module Ownership

Current Rust workspace:

```text
crates/
  codefire-core/
    src/lib.rs

  codefire-store/
    src/lib.rs

  codefire-util/
    src/lib.rs

  codefire-cli/
    src/main.rs
    src/automation.rs
    src/cli_model.rs
    src/active_state.rs
    src/close_transaction.rs
    src/close_worktree.rs
    src/open_transaction.rs
    src/secure_worktree.rs
    src/context.rs
    src/verification.rs
    src/fire.rs
    src/fire/batch.rs
    src/extinguish_ux.rs
    src/link_batch.rs
    src/evidence.rs
    src/evidence/batch.rs
    src/doctor.rs
    src/doctor/checks.rs
    src/doctor/report.rs
    src/migration.rs
    src/storage.rs
    src/metrics.rs
    src/view.rs
    src/view/file_diff.rs
    src/view/semantic_diff.rs
    src/remote.rs
    src/remote/idempotency.rs
    src/remote/plans.rs
    src/http.rs
    src/http_tls.rs
    src/signatures.rs
    src/limited_yaml.rs
    src/exit_code.rs
    src/completion.rs
```

Ownership rules:

| Module | Owns | Must not own |
|---|---|---|
| `main.rs` | dispatch and thin orchestration | domain logic, JSON string building, large command implementations |
| `cli_model.rs` | CLI option/result/context types | object store invariants |
| `automation.rs` | JSON envelope, diagnostics rendering, next_actions | command mutation logic |
| `close_transaction.rs` | durable close/discard request, evidence matrix, phase/retry/archive ordering | generic tree traversal or CLI rendering |
| `close_worktree.rs` / `active_state.rs` | capability-bound worktree/active subjects and exact transactional removal | cross-record close phase policy |
| `secure_worktree.rs` | shared stable identity, no-follow capability, no-replace publication primitives | command-specific close/open policy |
| `codefire-core` | Atom, Trace Graph, scan, verification semantics | CLI parsing, filesystem mutation outside read/index operations |
| `codefire-store` | canonical JSON, object record, sealed commit validation | open directory state transitions |
| `doctor/*` | repository health diagnostics | migration write behavior |
| `migration.rs` | compatibility-profile inspection plus locked, capability-relative application of shared auto-create layout actions | independent layout definitions, arbitrary paths, or repository/object format rewriting |
| `storage.rs` | capacity and artifact observability | verification policy |
| `remote/*` | remote project mutation and validation | local open state mutation except via explicit service calls |

When a change adds a responsibility not listed here, update this document or `docs/development/module-boundaries.md` before implementation.

## 11.3.1 Source Shape Contract

内部設計は抽象設計だけでなく、実装の形も制約する。CodeFireのsource codeは、読む人が次の対応を推測できる構造でなければならない。

| Design concept | Expected source shape |
|---|---|
| command contract | parser, options/result type, runner, text renderer, JSON rendererが識別できる |
| local workflow | scan/verify/fire/extinguish/commitが同じOpenContextとactive state規則を使う |
| repository services | repo discovery、open registry、branch head、lock、atomic writeが分散しすぎない |
| domain semantics | Atom、Trace Graph、Fire、Resolution、Verificationは`codefire-core`に残る |
| object identity | object ID、record validation、sealed commit validationは`codefire-store`に残る |
| automation interface | machine-readable outputはenvelope helperとcommand-specific data helperに分かれる |
| remote boundary | remote mutation、HTTP transport、signature/TLS/idempotencyの境界がfileで分かる |
| recovery tooling | doctor/migrate/storageが同じlayout vocabularyを参照する |

Source documentation requirements:

- `docs/development/source-architecture.md` はcrate/module ownership、data flow、transaction、persistence/security境界を説明する。
- `docs/development/source-data-model-catalog.md` はRust型、persisted JSON、object、remote、evidence dataの対応を説明する。
- `docs/development/command-source-trace.md` はcommandごとのparse/validate/plan/apply/render/write/test経路を説明する。
- `docs/development/change-playbook.md` はreading route、変更recipe、invariant review、validation選択を説明する。
- 主要command、module、object kind、version定数を変えたchangeは、該当文書を更新してsource-doc checkerを通す。

この規則により、ドキュメントだけを読んでも「どのcrateに何があり、どの関数を起点に変更するか」を想像できる状態を維持する。

## 11.4 Data Model

### Object Store

Object records are immutable and content addressed.

```text
ObjectRecord
  id: CF-<TYPE>-<digest>
  hash: sha256(canonical(payload))
  type: object kind
  payload: object-specific data
```

Requirements:

- canonical JSON must be deterministic.
- final object path must match object kind and ID.
- writes must be publish-after-validate.
- legacy shorter IDs may be read only if full record validation passes.
- serialized records are capped at 64 MiB on write and read; readers use no-follow, non-blocking regular-file opens plus metadata and bounded-read checks.
- writer, ordinary reader, sealed traversal, and full-store scan open absolute root/subdirectory components without following symlinks; a symlink at the object root or an intermediate subdirectory is rejected.
- object publication uses a validated, synced create-new temp and no-replace hard link, then rechecks the pinned root/subdirectory identities; a full scan likewise succeeds only after its final identity recheck. Sealed consumers that cross a later mutable visibility point retain `ValidatedSealedCommit`, read commit/root/reference objects from the validation-bound handles, and recheck the same subject immediately before and after publication.

Sealed validation is a stronger consuming boundary than an ordinary object read. It opens and pins the canonical objects root, records the identity-or-missing state of the twelve built-in subdirectories and every currently present extension subdirectory, and reads commit/parent/root/reachable records relative to those directory handles with final-component no-follow semantics. At most 128 extension subdirectories may be bound; exceeding the cap fails as `InvalidSealedCommit` before handles enter the cache. The command-scoped cache key is `(root path + root identity + ordered subdirectory identity/missing set, commit ID)`, and successful entries are published only after the ambient root and bound subdirectories are rechecked. Root or child replacement therefore cannot reuse a validation result for the old subject. One cache retains only its current store subject: binding a different root/identity/subdirectory set evicts prior pins and validation entries. On platforms without stable directory identity, one traversal still uses pinned handles but cache results are not reused across calls.

This boundary assumes the cooperative object-store rule that a published object file is immutable. It binds directory subjects and read handles; it is not an in-place rewrite detector for an already cached object record. ADR-011 owns this decision and its root/subdirectory swap-back fixtures.

### Sealed Commit

Sealed commit references the complete consistency state.

```text
Commit
  version = 1
  root_schema_version = 2
  semantic_validation_profile = sealed-snapshot-v1
  parents
  manifest_root
  atom_index_root
  trace_graph_root
  fire_delta_root
  resolution_ledger_root
  verification_root
  policy_root
  certificate
  optional signature
```

Validation must check:

- commit object type.
- commit/root schema versions and compatibility mode.
- parent commit validity.
- root object existence and type.
- certificate structure and consistency counters.
- evidence/artifact references reachable from verification and resolutions.
- Atom-versus-Trace endpoint integrity for the commit's declared semantic profile.
- schema-2 cross-root Fire/Resolution/Atom/Trace/policy/verification semantics through the authoritative `codefire-core` predicate.

Current Rust commits use commit version 1, root schema 2, semantic profile `sealed-snapshot-v1`, and version-1 roots. Missing `root_schema_version` selects the legacy schema-1 required-root/shape contract; any schema other than 1 or 2 and any commit/root object version other than 1 fails closed. Missing semantic profile selects the explicitly weaker legacy-unversioned cross-root contract, while unknown/non-string profiles fail closed. Each parent is validated under its own profile, so a strict current commit may retain an unmodified legacy parent. For schema 2, empty `verification.missing_evidence_refs` and `verification.unresolved_merge_conflicts` arrays may be omitted. The missing-evidence certificate count may also be omitted only when the verification count is zero; unresolved merge conflicts have no certificate count because only zero may seal. ADR-012 defines the independent semantic-profile axis, and ADR-014 defines the merge extension.

For root schema 2, the store decodes Atom index, Trace Graph, Fire ledger, Resolution ledger, verification, and policy into `codefire-core` types and invokes `validate_sealed_snapshot_semantics`. Core consumes unresolved merge facts and recomputes the open required-Fire count, duplicate Atom IDs, missing required links, stale resolutions, trace-completeness flag, verification result, and Fire/active-Resolution UID/cardinality/identity agreement. The store then checks certificate result/count agreement and reachable evidence/artifact references. Commit/request signature generation and policy enforcement remain a CLI `signatures` responsibility at local commit and remote/security boundaries; the store must not claim that a structurally valid optional signature is cryptographically trusted.

### Active State

Active state is mutable metadata for one open directory.

```text
.codefire/active/<open_instance_id>/
  state.json
  scan.json
  fires.json
  resolutions.json
  verification.json
  state_transaction.json    # only while live scan/verify publication is pending
  commit_transaction.json   # only while commit is pending/interrupted
```

Allowed:

- fire metadata.
- resolution metadata.
- scan summary and hashes.
- verification result.
- open identity.

Forbidden:

- working file blobs.
- patch content that can restore work.
- stash/checkpoint snapshots.

## 11.5 State Model

Open branch state is a derived view, not a free-form string.

Inputs:

- branch head.
- open registry.
- current base commit.
- active scan freshness.
- changed atom count.
- non-Atom changed file count.
- open fire count.
- stale resolution count.
- latest verification result.
- pending merge/conflict state.

Required invariant:

```text
if open_fire_count > 0:
  state = open-burning
```

State meanings:

| State | Meaning |
|---|---|
| `closed` | branch has no valid open directory |
| `open-clean` | open directory matches base and has no open fires |
| `open-burning` | changes or unresolved responsibilities exist and commit is blocked |
| `open-consistent` | changes exist, blocking diagnostics are zero, commit may proceed |

Status, scan, verify, commit, close must call the same state reducer. Individual commands must not invent state labels.

## 11.6 Command Lifecycle

Every mutating command follows this conceptual lifecycle. When validation depends on mutable repository/open metadata, lock acquisition and context reload move before that validation and plan construction; dry-run keeps the read-only path and does not acquire ownership merely to render a plan.

```text
parse args
resolve path/repo/open context
if dry-run:
  load/validate the read-only subject
  build and render the plan
  exit
acquire locks for the mutation subject
reload mutable context under the lock
load policy and current state
validate command-specific preconditions
build operation plan
if no longer applicable:
  render plan and exit
apply writes through atomic helpers
render result
```

Read-only commands follow:

```text
parse args
resolve path/repo/open context when needed
load validated source objects
build read model
render human or JSON output
```

No command should perform filesystem mutation while still discovering required inputs.

## 11.7 Command Result Contract

All JSON-capable commands return:

```json
{
  "schema": "codefire.command_result.v1",
  "command": "scan",
  "ok": true,
  "exit_code": 0,
  "repo": "/path/to/repo",
  "data": {},
  "diagnostics": [],
  "next_actions": []
}
```

Design requirements:

- JSON success and JSON failure use the same envelope.
- `exit_code` equals the process exit code.
- `diagnostics[].blocking` is authoritative for blocker filtering.
- `next_actions` must include enough machine-readable target data to be run outside the original cwd.
- collection samples preserve full total/omitted counts and bound every echoed untrusted string. For `link --batch`, diagnostics, plan items, and applied-link projections are independently capped at 100, with 256-byte UTF-8-safe string samples and explicit omission fields.
- envelope schema changes require explicit versioning.

## 11.8 Scan Design

Scan flow:

```text
1. find the repository, acquire RepoLock, and reload open context
2. validate the complete sealed base through the command cache
3. load codefire.yaml and build the current AtomIndex
4. load the base AtomIndex
5. load the Trace Graph and validate all endpoints against the current index
6. compare Atom hashes and non-Atom files against the base manifest
7. generate automatic fires from changed atoms and trace impact
8. preserve manual fires and remove obsolete automatic fires
9. validate any existing state-transaction marker before the first recovery write
10. publish verification invalidation, fires, scan, registry, active state, and branch through the scan phase sequence
11. verify the three state authorities agree and remove the marker
12. render changed atoms, non-Atom changes, open fires, metrics
```

Algorithm requirements:

- Atom lookup and required link checks should use maps/indexes, not repeated full graph scans.
- fire identity must be stable across scans for the same source/target/reason/basis.
- obsolete detection must not remove manual fires.
- scan must be safe on repositories with many files and large artifacts.

## 11.9 Verification Design

Verification combines internal and external checks.

Live verification preflights policy and strict active merge metadata before the locked scan may write, then computes the internal/external result. It validates any surviving state marker before writing a new verify marker, writes `verification.json`, advances registry/active/branch through the verify phase sequence, and removes the marker only after the three state authorities agree. Read-only preview callers use the compute boundary without these writes.

Internal checks:

- required open fires.
- missing required links.
- stale resolutions.
- missing evidence refs.
- duplicate Atom IDs.
- merge conflicts.
- policy/config shape errors.

External checks:

- configured verification commands.
- timeout handling.
- cwd/env restrictions where configured.
- captured evidence references when needed.

Diagnostic model:

```text
kind
severity
blocking
message
target
repairable
evidence
```

`--blocking-only` filters diagnostics and filtered data views consistently. Non-blocking warnings may remain in all-data fields but must be separately named.

## 11.10 Fire and Resolution Design

Fire generation is impact analysis, not proof of inconsistency.

Fire record:

```text
fire_uid
display_id
source atom
target atom
reason
severity
origin
basis
created_at
```

Resolution record:

```text
resolution_uid
fire_uid
resolution_type
rationale
evidence_refs
basis
created_at
```

Stale detection compares current basis with resolution basis:

- source atom hash.
- target atom hash.
- trace link hash.
- policy hash.
- fire reason and kind where relevant.

Batch extinguish must validate every item before writing any resolution.

## 11.11 Commit Sealing Design

Commit flow:

```text
1. acquire the repository lock and reload open context
2. validate the complete sealed base and open identity
3. run fresh scan and verification under that lock
4. reject blockers
5. create commit_transaction.json
6. if policy requires a signature, acquire the key before the first immutable write
7. purely plan every manifest blob, seven sealed roots, signed/unsigned candidate, and next branch snapshot
8. publish all ordered role entries as planned before the first store
9. store blobs and roots, verify each ID, and mark each role entry stored
10. store/mark the candidate and validate the complete sealed candidate
11. store/mark the precomputed branch snapshot
12. publish the mutable branch ref, update opened registry, and reset active state
13. remove the transaction marker and render the summary
```

Write ordering:

- `objects[]` is an ordered role ledger with `role`, nullable repo-relative `path`, `object_type`, `object_id`, and `planned|stored` state. Duplicate blob IDs at different paths remain separate role entries.
- predeclare every immutable role before any object storage so every store-to-marker interruption is diagnosable.
- store the branch snapshot object after candidate validation but before publishing the mutable branch ref.
- update branch head only after all referenced objects exist and complete sealed validation passes.
- reset active state only after branch head update succeeds.

If commit fails before branch head update, no formal history is changed.

Current failpoint/corruption evidence covers the store-before-marker gap for a manifest blob, sealed root, candidate, and branch snapshot; interruption immediately before candidate validation; and rejection of invalid certificate, missing/wrong root, and invalid parent candidates. Each store-gap fixture proves the object is discoverable through its still-planned entry while branch/opened heads remain unchanged. It does not claim an exhaustive process-crash fixture at every write boundary.

## 11.12 Diff and Merge Design

Diff has multiple layers:

- manifest/file diff.
- binary summary.
- Atom diff.
- Trace Graph diff.
- policy impact diff.
- fire impact diff.

Diff algorithms:

- default bounded line diff.
- optional Myers/patience/histogram-style behavior.
- rename/copy detection with candidate limits.
- binary/non-UTF-8 files summarized by kind, size, hash.

Merge flow:

```text
1. validate source and target sealed heads
2. find common ancestor with bounded traversal
3. materialize merge into target open directory
4. detect file conflicts and binary conflicts
5. record text/binary/semantic unresolved authority
6. create merge_changed fires
7. leave target open-burning
8. require `merge --continue` after marker/sidecar resolution and semantic acknowledgement
9. verify, then seal through the normal two-parent commit transaction
```

Merge must not create a sealed commit directly. Human or external automation resolves file conflicts, removes binary sidecars after choosing the target result, explicitly acknowledges semantic candidates, runs `merge --continue`, resolves any derivative Fires, and verifies. Non-empty merge facts are the highest-priority `CommittableDecision` blocker and map to exit 15.

## 11.13 Remote Design

Remote project is a distribution boundary.

Remote operations:

- upload.
- list.
- clone.
- show/diff.
- request-merge.
- request-list.
- request-review.
- request-apply.
- doctor/gc.

Remote validation:

- incoming branch head is sealed commit.
- object graph is complete.
- parent history validates.
- root object types validate.
- permissions allow operation.
- token/signature policy passes.
- nonce replay cache rejects reused mutating requests.
- server-side verification runs when configured.

Remote mutation must be idempotent where keys are provided. Same key and same payload replays safely; same key and different payload fails.

## 11.14 Evidence and Artifact Design

Evidence records support:

- command capture.
- argv command capture without shell expansion.
- artifact reference.
- external artifact URI.
- batch creation.
- dry-run.

Design requirements:

- non-zero command exit must not look like successful proof unless explicitly allowed.
- stdout/stderr capture is bounded.
- truncation is recorded.
- artifact paths are repo-relative when possible and redacted when external.
- large artifacts generate diagnostics and storage warnings.
- evidence object path and IDs are returned in JSON.

## 11.15 Doctor, Migrate, Storage

These commands must share layout definitions.

Doctor:

- read-only by default.
- reports blocking, warning, repairable, category, path, target.
- distinguishes corruption from missing optional/autocreatable directories.

Migrate:

- check reports compatibility.
- dry-run reports planned actions.
- apply performs bounded layout/policy changes.
- planned actions correspond to doctor diagnostics.

Storage:

- reports `.codefire/objects`, active state, idempotency, remotes.
- groups objects by type.
- reports large objects and artifact refs.
- exposes quick/full modes.

Doctor and migrate must not contradict each other on whether a repo is usable.

## 11.16 Locking and Atomic Writes

Locks:

```text
repo.lock
repo.lock.owner.json
branch-<encoded-name>.lock
object-store.lock
active-<open_instance_id>.lock
remote resource locks
remote generation.lock
```

Rules:

- acquire locks in stable order.
- timeouts return stable lock contention exit code.
- local/import `repo.lock` is a persistent regular no-follow file with exactly one provable filesystem link; ownership is its kernel advisory lock, not pathname existence. Acquisition never truncates or writes its contents. File identity/link provenance is checked before locking, immediately after locking through the canonical pathname, and again at the mutation use boundary. Process exit releases ownership, while normal unlock never unlinks the inode.
- repository owner metadata `{version,pid,created_at}` is atomically replaced only after ownership in the common `repo.lock.owner.json` sidecar. Contention reads are no-follow, regular-file only, and capped at 16 KiB; invalid sidecars are ignored. Unlock leaves both files in place.
- import uses that same repository-lock/sidecar rule plus a persistent root `.codefire-import.lock` whose immutable payload binds bootstrap provenance. Mutable import recovery phase lives in the separately atomically replaced `.codefire-import-transaction.json`.
- the older specialized remote/resource `FileLock` still uses create-new/unlink semantics and is outside ADR-010 until migrated with its own fixtures.
- writes use temp file, fsync where appropriate, validation, atomic rename or link finalization.
- after lock acquisition, revalidate mutable preconditions.

Generic CLI repository JSON uses one symmetric 64 MiB serialization/read cap. The writer rejects oversized rendered JSON before temp publication. The reader opens the absolute parent component-by-component without following symlinks, opens the final name no-follow/non-blocking, requires a regular file, and enforces both metadata size and a limit-plus-one read. Specialized transaction/import/owner formats may use smaller documented limits.

Close/discard publishes a 1 MiB-capped pending marker before deletion and uses only transaction-derived single-component quarantine names. Worktree, active state, opened registry, branch reference, and their parents are checked against recorded stable identities at their use boundaries. The closed branch object is immutable and precedes the mutable branch reference. Completion publishes an exact fixed archive before identity-bound removal of pending authority; a subsequent distinct close retires that archive explicitly.

Import reserves only these root namespaces: exact `.codefire-import.lock`, `.codefire-import-transaction.json`, and `.codefire-open`; `.codefire-import.lock.bootstrap-*.part` unpublished staging; fully written `.codefire-import.lock.bootstrap-*.tmp` publication candidates; and `.codefire-import.write-<transaction-id>-*.tmp` atomic-write remnants. `.part` files are not parsed as authority and are preserved unless identity proves that they are another name for the canonical lock. A valid/malformed foreign bootstrap candidate is never blindly deleted; malformed published `.tmp` and foreign write-temp IDs fail closed. Matching write temps are recoverable. Nested or merely similar names remain user content.

Import's parsed operational/control JSON is read no-follow and non-blocking with a 1 MiB cap, regular-file requirement, metadata precheck, and limit-plus-one guard. Repository byte-copy reads are capped at 64 MiB. Ordinary imported user files are capped at 49,545,216 bytes (47.25 MiB) so base64 serialization plus reserved record overhead remains inside the object store's 64 MiB record cap.

## 11.17 Error and Exit Code Design

Error classes:

- usage/config error.
- verification blocker.
- repository corruption.
- object/hash/reference invalid.
- sealed commit validation failed.
- lock contention.
- remote rejection.
- auth/signature failure.
- idempotency conflict.
- migration incompatibility.
- external artifact validation failure.

Design requirements:

- user-facing text should be concise and actionable.
- JSON diagnostics should be structured and stable.
- multiple verification blockers choose exit code by documented priority.
- internal invariant violation should not be disguised as usage error.

## 11.18 Performance Design

Expected pressure points:

- Atom extraction over many files.
- required link evaluation.
- diff over large or repeated files.
- object graph traversal.
- remote upload graph copy.
- doctor full object validation.
- storage full object parsing.

Required mitigations:

- build maps and indexes once per command.
- bound candidate pairs in rename detection.
- bound output size and context lines.
- provide quick/full modes.
- expose phase metrics.
- avoid reading large artifacts into memory when hashing.
- keep cache optional and rebuildable.

## 11.19 Security and Trust Boundaries

Trust boundaries:

- local user filesystem.
- remote project storage.
- HTTP/HTTPS transport.
- external verification command.
- evidence command capture.
- artifact paths and URIs.
- signing keys and request keys.

Requirements:

- remote server validates all received object graphs.
- HTTP path parsing rejects traversal and invalid encodings.
- TLS insecure mode is explicit and local/test only.
- verification/evidence command capture drains stdout/stderr while the process runs, stores only each configured prefix, reports bytes seen/stored/truncation, and records whether timeout terminated a Unix process group or only a direct child.
- argv mode avoids shell expansion when requested.
- command environments are default-deny/minimal and their values are bound by digest rather than serialized; secret-like values and external/home path literals are redacted before evidence or diagnostics are stored.
- verification cwd is contained below its open subject; the live subject binds the canonical worktree manifest, stale pre-execution subjects are rejected, passing commands that change that subject are blockers, and execution records carry profile/phase/subject/input/environment/worktree identities.
- request signature v2 authenticates a versioned subject containing the complete payload digest, actor/operation/target/key/idempotency/time/nonce; key validity precedes nonce publication and cache saturation preserves live replay memory by failing new requests closed.
- context, explain, review pack, verification, and commit certificate project one versioned assessment subject appropriate to their live or sealed trust boundary. Commit rechecks a live worktree subject before its transaction, after manifest planning, and immediately before mutable branch publication; recovery recomputes candidate/open agreement before completing an interrupted visibility step.
- open materialization resolves the target below the canonical repository-parent boundary, rejects parent traversal/symlink/overlap, writes relative to pinned no-follow capabilities, and rechecks the ambient target identity before repository metadata publication.
- sealed object validation reads through pinned root/subdirectory capabilities and binds cache reuse to both root and child directory subjects.

## 11.20 Test Architecture

Test domains:

| Domain | Location |
|---|---|
| docs/help/completion consistency | `crates/codefire-cli/src/tests/docs.rs` |
| command contract and JSON envelope | `src/tests/cli_contract.rs` or equivalent module |
| local workflow | local workflow test module |
| fire/extinguish/verify | workflow/verification test module |
| doctor/migrate/storage | health/recovery test module |
| evidence | evidence test module |
| diff/merge | view/diff/merge test module |
| remote/http/signature | remote/http/signature test modules |
| core extractor/trace policy | `codefire-core` unit tests |
| store canonical/object validation | `codefire-store` unit tests |

Testing rules:

- new command behavior needs both text behavior where relevant and JSON contract tests.
- any bug fix gets a regression test named after behavior, not after the issue number only.
- large algorithmic changes need boundary tests for size, depth, repeated data, malformed input.
- docs/help/completion must remain consistent.
- transaction claims name the exact tested windows. State-transaction fixtures currently cover scan after `fires_written`, verify after `verification_written`, cross-operation retry, and invalid-operation/phase rejection before recovery writes; they are not evidence for every possible metadata-write interruption.

## 11.21 Recovery Model

Recovery strategy:

- immutable valid objects are never rewritten.
- invalid temp records can be removed.
- missing optional directories can be recreated.
- branch head corruption is blocking and requires explicit repair.
- open registry/marker mismatch is blocking for mutation.
- cache can be deleted and rebuilt.
- active state corruption can be diagnosed; recovery must not fabricate file content.
- pending `commit_transaction.json` markers are diagnostic evidence for interrupted multi-object commits. Current `objects[]` entries carry role/path/type/ID plus planned/stored state; a candidate alias mirrors the candidate entry. Doctor exposes marker path and a message containing phase plus stored/planned role-entry counts, not separate entry/candidate fields. Legacy string entries remain readable and count as stored.
- pending `state_transaction.json` markers identify interrupted live scan/verify publication. Retry validates marker shape and the current operation/phase membership before its first recovery write, then recomputes derived state. Only the writer can enforce adjacent phase transitions; one persisted current phase cannot prove historical transition order.
- pending `.codefire/close_transaction.json` is the retry authority for interrupted close/discard. Only the exact request may resume its five adjacent phases. Deterministic worktree/active/registry quarantine plus branch before/object/after evidence distinguishes every tested publication gap; divergent replacements and simultaneous pending open/close authority fail closed. Dry-run never recovers it, and completion archives before removing it.
- import retry accepts only the request-bound bootstrap/transaction/repository provenance for the same normalized subject. It preserves unpublished `.part` scratch, can recover fully formed matching `.tmp` publication, removes only transaction-bound write temps, and rejects foreign/malformed published provenance without adopting a normal `init` repository.

`doctor` explains the state. `migrate` changes layout/policy when the change is known and bounded. Neither should silently mutate by default.

## 11.22 Extension Rules

Adding a new object type:

1. define payload schema.
2. add known object subdirectory mapping.
3. add canonical object tests.
4. add sealed graph reference extraction if referenced.
5. add doctor/storage visibility.
6. document migration and compatibility.

Adding a new command:

1. define path mode and JSON support.
2. add help/completion.
3. use shared command result envelope if JSON exists.
4. implement dry-run if mutating.
5. define exit codes and diagnostics.
6. add tests and docs.

Adding a new Atom extractor:

1. define supported syntax subset.
2. avoid unbounded parsing.
3. produce stable Atom IDs.
4. report invalid explicit Atom IDs with artifact/line.
5. add duplicate ID and content hash tests.

## 11.23 Current Architecture Gaps

Known gaps that drive v0.8 planning:

- command surface contract is still uneven across show/list/branch/diff/commit failure paths.
- state derivation needs one reducer shared by status/scan/verify/commit.
- doctor and migrate need a shared layout definition.
- evidence command execution is local and unsandboxed; `trust_kind=local_unisolated` and platform termination scope must not be interpreted as OS isolation.
- fire volume needs grouping and batch template generation.
- non-Atom changed files need first-class scan visibility.
- metrics should distinguish unmeasured from measured zero-cost phases.
- source navigation docs must stay close enough to code that a contributor can infer module shape and data flow before opening the implementation.

These are not separate product directions. They are architecture debt against the requirements in `docs/01_product_definition.md`.
