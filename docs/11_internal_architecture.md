# 11. 内部アーキテクチャ設計

この文書は、Code Fireの内部設計である。実装時のmodule境界、データフロー、不変条件、エラー処理、テスト設計の正本として扱う。

実際のsource file、主要関数、commandごとの実装経路は、補助文書として `docs/development/source-code-map.md`、`docs/development/source-code-blueprint.md`、`docs/development/source-code-mental-model.md`、`docs/development/source-code-reconstruction-guide.md`、`docs/development/source-data-model-catalog.md`、`docs/development/source-code-anatomy.md`、`docs/development/command-source-trace.md` に展開する。この文書は設計の正本、補助文書はsource navigationの正本である。

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
| `codefire-core` | Atom, Trace Graph, scan, verification semantics | CLI parsing, filesystem mutation outside read/index operations |
| `codefire-store` | canonical JSON, object record, sealed commit validation | open directory state transitions |
| `doctor/*` | repository health diagnostics | migration write behavior |
| `migration.rs` | migration plan/apply | independent layout definitions that diverge from doctor |
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

- `docs/development/source-code-map.md` はfile/module/type/function責務の索引である。
- `docs/development/source-code-mental-model.md` は関数内のdata flowと永続化境界を説明する。
- `docs/development/source-code-reconstruction-guide.md` はsourceを開く前にworkspace tree、command skeleton、`main.rs` の帯、永続化境界、変更recipeを再構成するための実装復元ガイドである。
- `docs/development/source-data-model-catalog.md` はRust型、JSON payload、active state、object record、remote/storage/evidence dataの辞書である。
- `docs/development/source-code-anatomy.md` は主要fileの内部にある型、関数群、状態、永続化境界、変更時の触り方を説明する。
- `docs/development/source-code-blueprint.md` は要求、設計、source、persistence、testの対応表である。
- `docs/development/command-source-trace.md` はCLI commandからparse/run/render/writeまでのtraceである。
- 主要commandの実装経路を変えたcommitは、上記文書の少なくとも1つを更新する。

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

### Sealed Commit

Sealed commit references the complete consistency state.

```text
Commit
  parents
  manifest_root
  atom_index_root
  trace_graph_root
  fire_delta_root
  verification_root
  policy_root
  certificate
  optional signature
```

Validation must check:

- commit object type.
- parent commit validity.
- root object existence and type.
- certificate structure and consistency counters.
- evidence/artifact references reachable from verification and resolutions.
- optional signature policy.

### Active State

Active state is mutable metadata for one open directory.

```text
.codefire/active/<open_instance_id>/
  state.json
  scan.json
  fires.json
  resolutions.json
  verification.json
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

Every mutating command follows the same lifecycle.

```text
parse args
resolve path/repo/open context
load policy and current state
validate command-specific preconditions
build operation plan
if dry-run:
  render plan and exit
acquire locks
revalidate mutable preconditions
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
- envelope schema changes require explicit versioning.

## 11.8 Scan Design

Scan flow:

```text
1. resolve open context
2. validate marker/open registry/base/head
3. load codefire.yaml
4. build current AtomIndex
5. load base AtomIndex
6. compare Atom hashes
7. compare non-Atom files against manifest
8. load Trace Graph and policy
9. generate automatic fires from changed atoms and trace impact
10. preserve manual fires
11. remove obsolete automatic fires
12. write active scan/fires/state
13. render changed atoms, non-Atom changes, open fires, metrics
```

Algorithm requirements:

- Atom lookup and required link checks should use maps/indexes, not repeated full graph scans.
- fire identity must be stable across scans for the same source/target/reason/basis.
- obsolete detection must not remove manual fires.
- scan must be safe on repositories with many files and large artifacts.

## 11.9 Verification Design

Verification combines internal and external checks.

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
1. acquire repo and branch locks
2. validate open identity
3. run or load fresh scan
4. run verification
5. reject blockers
6. build content manifest from working files
7. store blobs and derived objects
8. store verification and fire delta
9. build certificate
10. optionally sign commit
11. store commit object
12. atomically update branch head
13. reset active state to clean base
14. render summary
```

Write ordering:

- store immutable objects first.
- update branch head only after all referenced objects exist and validate.
- reset active state only after branch head update succeeds.

If commit fails before branch head update, no formal history is changed.

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
5. create merge_changed fires
6. leave target open-burning
```

Merge must not create a sealed commit directly. Human or external automation must resolve fires and verify.

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
branch-<encoded-name>.lock
object-store.lock
active-<open_instance_id>.lock
remote resource locks
remote generation.lock
```

Rules:

- acquire locks in stable order.
- timeouts return stable lock contention exit code.
- lock files should include owner metadata when available.
- writes use temp file, fsync where appropriate, validation, atomic rename or link finalization.
- after lock acquisition, revalidate mutable preconditions.

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
- command capture uses bounded output and timeout.
- argv mode avoids shell expansion when requested.
- secrets should not be stored in object payloads or diagnostics.
- signatures and nonce replay protect mutating remote operations when policy requires them.

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

## 11.21 Recovery Model

Recovery strategy:

- immutable valid objects are never rewritten.
- invalid temp records can be removed.
- missing optional directories can be recreated.
- branch head corruption is blocking and requires explicit repair.
- open registry/marker mismatch is blocking for mutation.
- cache can be deleted and rebuilt.
- active state corruption can be diagnosed; recovery must not fabricate file content.

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
- evidence command failure semantics need stricter trust handling.
- fire volume needs grouping and batch template generation.
- non-Atom changed files need first-class scan visibility.
- metrics should distinguish unmeasured from measured zero-cost phases.
- source navigation docs must stay close enough to code that a contributor can infer module shape and data flow before opening the implementation.

These are not separate product directions. They are architecture debt against the requirements in `docs/01_product_definition.md`.
