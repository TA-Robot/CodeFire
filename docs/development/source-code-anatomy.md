# CodeFire Source Code Anatomy

この文書は、CodeFireのソースコードを開く前に、実装の中身まで具体的に想像できるようにするためのsource anatomyである。

`source-code-map.md` はfile索引、`source-code-mental-model.md` は処理の流れ、`command-source-trace.md` はcommand別trace、`source-code-blueprint.md` は要求からsourceへの対応表である。この文書はさらに一段具体化し、主要fileの内部にある型、関数群、状態、永続化境界、変更時の触り方を説明する。

## 1. Read This Before Opening Source

CodeFireは次の4層で読むと崩れにくい。

```text
CLI shell
  main.rs, completion.rs, cli_model.rs, exit_code.rs
  argvをOptionsへ変換し、Runnerを呼び、text/JSONへ戻す。

Workflow and repository shell
  main.rs, idempotency.rs, open_clone_idempotency.rs, merge_patch_idempotency.rs
  repo/open/branch/active state/lock/idempotencyを扱う。

Domain and object core
  codefire-core/src/lib.rs, codefire-store/src/lib.rs, codefire-util/src/lib.rs
  Atom/Trace/Fire/Verificationと、canonical object identityを扱う。

Feature modules
  evidence, fire, batch, context, doctor, migration, storage, view, remote, http, signatures
  commandごとの縦切り責務を持つ。
```

重要なのは、`main.rs` を「全部入りの実装」として読むのではなく、まだ抽出途中のorchestration layerとして読むことである。新規機能を追加するときは、`main.rs` に最小のdispatchと既存helper接続だけを置き、command固有のOptions/Result/Runner/Rendererはmoduleへ逃がす。

## 2. Workspace Reality

現在のRust実装はこの形である。

```text
crates/
  codefire-cli/
    src/main.rs                  process entry, dispatch, local workflow, repo/open helpers
    src/cli_model.rs             shared Options/Result/OpenContext types
    src/automation.rs            command_result envelope, diagnostics, next_actions
    src/completion.rs            help text and shell completion
    src/exit_code.rs             stable exit code taxonomy
    src/verification.rs          verify parser and human renderer
    src/fire.rs                  single manual fire
    src/fire/batch.rs            manual fire batch
    src/batch.rs                 extinguish batch
    src/extinguish_ux.rs         interactive/all-matching extinguish UX
    src/link_batch.rs            Trace Link batch mutation
    src/evidence.rs              single evidence add
    src/evidence/batch.rs        evidence add batch
    src/context.rs               read-only context packs
    src/explain.rs               read-only explanation commands
    src/doctor.rs                doctor option/report model
    src/doctor/checks.rs         doctor checks
    src/doctor/report.rs         doctor rendering/JSON/next_actions
    src/migration.rs             migration check/dry-run report
    src/storage.rs               local/remote/object/artifact capacity report
    src/view.rs                  show, diff, review-pack, patch export orchestration
    src/view/file_diff.rs        bounded file diff, rename/copy, line algorithms
    src/view/semantic_diff.rs    Atom/Trace/policy impact diff
    src/remote.rs                file-backed remote operations
    src/remote/idempotency.rs    remote idempotency records
    src/remote/plans.rs          remote dry-run plans
    src/http.rs                  HTTP client/server and endpoint dispatch
    src/http_tls.rs              TLS setup
    src/signatures.rs            HMAC signatures and nonce replay checks
    src/limited_yaml.rs          limited YAML helper parser
    src/metrics.rs               command metrics payloads
    src/tests.rs                 broad CLI regression tests
    src/tests/docs.rs            help/docs/completion consistency tests

  codefire-core/
    src/lib.rs                   Atom extraction, Trace Graph, scan, verification

  codefire-store/
    src/lib.rs                   canonical JSON, objects, sealed commit validation

  codefire-util/
    src/lib.rs                   hash, hex, time, path utilities
```

Python files at the repository root are legacy/reference surfaces. The default product direction is the Rust workspace.

## 3. Main Command Spine

`codefire-cli/src/main.rs` should be imagined as a command spine with five repeated moves.

```text
run(args)
  -> command_help_for_args(args) when --help is present
  -> command match arm
  -> parse_*_args
  -> run_* / compute_* / module runner
  -> print_* or command_result_envelope(...)
```

For mutating commands, the middle expands:

```text
parse options
  -> find repo/open context
  -> validate immutable inputs
  -> if dry-run: return operation plan
  -> acquire lock
  -> revalidate mutable inputs
  -> apply writes atomically
  -> save idempotency replay record when requested
  -> render typed result
```

### Important Function Clusters

| Cluster | Functions | Mental picture |
|---|---|---|
| entry and dispatch | `main`, `run`, `command_help_for_args`, `subcommand_help` | argv is still raw strings; no command should mutate before this layer finishes help/error routing |
| workflow dispatch | `local_workflow_handler`, `run_scan_command`, `run_verify_command`, `run_fire_command`, `run_extinguish_command`, `run_commit_command` | common shape for open-directory commands |
| parser band | `parse_init_args`, `parse_open_args`, `parse_path_json_args`, `parse_extinguish_args`, `parse_commit_args`, remote/view parsers | argv becomes typed Options; parsers must not write files |
| repo lifecycle | `init_repo`, `open_branch_from`, `clone_branch`, `materialize_commit`, `open_operation_plan`, `clone_operation_plan` | closed branch and open directory are connected here |
| merge/patch | `merge_branch`, `build_merge_result`, `apply_merge_result`, `import_patch`, `apply_patch_entry` | sealed commit inputs become working tree changes or patch objects |
| scan/verify | `compute_scan`, `compute_verify`, `run_verification_commands`, `persist_verification_result_state` | source files become ScanResult/Verification and active state |
| extinguish/commit | `run_extinguish`, `run_commit`, idempotency helpers, operation plan helpers | active fires/resolutions become resolved state or sealed commit |
| object and branch helpers | `build_manifest`, `load_base_atom_index`, `commit_parents_for_open`, `load_branch_record`, `save_branch_record` | mutable branch records point at immutable objects |
| filesystem primitives | `write_json_atomic`, `sync_directory_best_effort`, lock structs, `find_repo_root`, `find_open_marker`, `read_json` | low-level durability and discovery; should remain boring and reusable |

### How To Judge A `main.rs` Change

Good `main.rs` changes usually do one of these:

- add or route a command to an existing feature module;
- connect a typed result to text/JSON output;
- call repository services and domain/store functions in a clear order;
- remove duplicated parsing, state, or rendering logic.

Risky `main.rs` changes usually do one of these:

- embed new domain rules that belong in `codefire-core`;
- build object IDs or sealed validation manually instead of using `codefire-store`;
- print JSON by hand instead of using result helpers and `automation.rs`;
- add large feature-specific logic that should live in a command module;
- write files inside parser/helper code before validation is complete.

## 4. Shared CLI Data Types

`cli_model.rs` is the shared vocabulary between parsers, runners, and renderers.

| Type | Meaning | Typical owner |
|---|---|---|
| `Status` | summarized repo/open state | `read_status`, `automation::status_data_json` |
| `Branch` | branch list row | branch loader and branch list output |
| `InitOptions` / `InitResult` | repository initialization contract | `parse_init_args`, `init_repo` |
| `OpenOptions` / `OpenResult` | branch materialization contract | `parse_open_args`, `open_branch_from` |
| `PathJsonOptions` | common path/json/metrics command options | status, scan, storage-like commands |
| `ExtinguishOptions` / `ExtinguishResult` | active fire resolution mutation | `parse_extinguish_args`, `run_extinguish` |
| `CommitOptions` / `CommitResult` | sealed commit mutation | `parse_commit_args`, `run_commit` |
| `CloneOptions` / `CloneResult` | branch clone/open copy flow | clone parser/runner |
| `DiffArgs`, `ReviewPackArgs`, `PatchExportArgs`, `PatchImportOptions` | view/patch CLI contracts | view and patch routing |
| `ServeOptions` | HTTP server configuration | `parse_serve_args`, `http::serve_http` |
| `MergeOptions` / `MergeResult` | local merge plan/apply result | merge runner and JSON renderer |
| `LockOptions` | common wait/timeout settings | local and remote mutators |
| `OpenContext` | repo root, open dir, branch, base commit, active state path | every open-directory workflow |
| `ScanExecution` / `VerifyExecution` | domain result plus context | scan/verify/commit orchestration |

If a command has more than trivial arguments or returns data used by JSON output, it should have a typed Options/Result shape. Raw `serde_json::Value` should be the last rendering step, not the primary business model.

## 5. Domain Core Anatomy

`codefire-core/src/lib.rs` is the semantic engine. It is dense, but it has a stable mental shape.

```text
config and policy
  codefire.yaml, codefire.links.yaml, codefire.policy.yaml

extractors
  source files -> Atom values -> AtomIndex

trace graph
  link inputs -> TraceLink values -> TraceGraph

scan
  current AtomIndex + base AtomIndex + TraceGraph + policy + resolutions
  -> changed atoms + fires + missing required links

verification
  ScanResult + resolutions + external check failures + verification policy
  -> Verification
```

### Core Concepts

| Concept | What it is | Should be changed when |
|---|---|---|
| `Atom` | a requirement/design/code/test/API/schema unit with ID, kind, path, selector, hash | extraction semantics or Atom metadata changes |
| `AtomIndex` | collection/index of current or sealed Atoms | lookup or duplicate handling changes |
| `TraceLink` / `TraceGraph` | directed relation between Atoms | link schema or graph traversal changes |
| `TracePolicy` / `RequiredLinkRule` | required relation policy | missing-link rule changes |
| `Fire` | unresolved responsibility for changed/missing/stale relation | scan impact semantics changes |
| `Resolution` | claim that a fire was handled with basis and evidence | extinguish/staleness semantics changes |
| `ScanResult` | current changed Atoms and open fires | scan output contract changes |
| `Verification` | complete commit gate result | commit blocker/warning policy changes |

### Core Function Shape

| Function family | Reads | Produces | Must not do |
|---|---|---|---|
| extractors | working files and config patterns | `AtomIndex` | write active state or print output |
| trace parsing | `codefire.links.yaml` | `TraceGraph` | infer CLI next actions |
| policy parsing | `codefire.policy.yaml` | trace/verification policy | mutate repository layout |
| scan builders | current/base indexes, links, policy, prior resolutions | `ScanResult` and fires | decide exit codes |
| verification builders | scan, resolutions, failed external checks, policy | `Verification` | create object records |
| resolution basis builders | fire, Atom index, Trace Graph, evidence refs | `Resolution` | validate object store layout |

The core module may read source files to index them. It must not know whether the caller is `scan`, `verify`, `commit`, or a future automation command except through explicit inputs.

## 6. Object Store Anatomy

`codefire-store/src/lib.rs` is the identity and sealed-history engine.

```text
payload JSON
  -> canonical_json
  -> sha256 digest
  -> object_id(type, digest)
  -> ObjectRecord { object_id, hash, type, payload }
  -> .codefire/objects/<kind>/<object_id>.json
```

Read path:

```text
object_id
  -> object_record_path
  -> read_object_record
  -> validate object_id, type, hash, filename, payload digest
  -> return payload
```

Sealed validation path:

```text
validate_sealed_commit(objects, commit_id)
  -> read commit object
  -> validate parent commits recursively
  -> validate required roots
  -> validate manifest references
  -> validate resolution/evidence/verification references
```

### Store Function Groups

| Group | Functions | Invariant |
|---|---|---|
| kind registry | `object_prefix`, `object_subdir`, `known_object_subdirs`, internal kind lookup | object type and path mapping are centralized |
| identity | `canonical_json`, `object_digest`, `object_id`, `object_record` | ID comes from canonical payload, not display text |
| persistence | `store_object`, `read_object`, `read_object_record`, `object_record_path` | published objects validate after write |
| validation | `validate_object_record`, `validate_sealed_commit`, reference validators | commit graph can be trusted before branch head points at it |
| commit payload | `commit_payload` | root/certificate shape is constructed consistently |

When adding an object kind, update store, doctor, storage, remote copying, internal architecture docs, and tests together.

## 7. Feature Module Anatomy

### Evidence

```text
parse_evidence_add_args
  -> EvidenceAddOptions
  -> run_evidence_add
       -> validate mode: artifact, URI, shell command, argv command
       -> dry-run: evidence_add_operation_plan
       -> apply: optional artifact_ref object + evidence object
  -> evidence_add_data_json / print_evidence_add_result
```

The batch path in `evidence/batch.rs` validates all items before applying. Evidence command capture stores bounded stdout/stderr summaries and truncation flags; never treat captured output as unbounded payload.

### Fire and Extinguish

Manual fire creation is in `fire.rs`; manual fire batch is in `fire/batch.rs`; extinguish batch is in `batch.rs`; interactive/all-matching UX is in `extinguish_ux.rs`.

```text
manual fire
  options/spec
  -> load open context and current Atom/Trace state
  -> build deterministic manual fire IDs
  -> write active fires

extinguish
  options
  -> validate active fire and evidence refs
  -> build resolution basis
  -> write active resolutions and update fires
```

All batch paths should validate the complete batch before writing the first item.

### Context and Explain

`context.rs` and `explain.rs` are read-only. They should summarize existing scan/fire/verification/storage state for tools and humans. They are not allowed to repair, extinguish, or commit.

### Doctor, Migration, Storage

These three modules are health and observability surfaces.

| Module | Reads | Writes | Main risk |
|---|---|---|---|
| `doctor/*` | repository layout, objects, branches, opened registry, active state | none | diverging from actual layout rules |
| `migration.rs` | repository layout and object records | currently plans/checks; future apply must be explicit | hiding incompatible format changes |
| `storage.rs` | local objects, active/opened/idempotency areas, artifact refs, remotes | none | undercounting artifacts or remote retained data |

They should share the same layout vocabulary as init/open/commit/store. If doctor and migration disagree, the layout model is duplicated incorrectly.

### View, Diff, Patch

`view.rs` resolves branch names, commit IDs, and remote commitish references. `view/file_diff.rs` renders file deltas with bounded output and multiple line algorithms. `view/semantic_diff.rs` compares Atom/Trace/verification-policy impact.

```text
diff command
  -> resolve left/right commitish
  -> load manifests and root objects
  -> file diff for bytes/text
  -> semantic diff for Atom/Trace/policy impact
  -> text or JSON-like review payload
```

Patch import still routes through `main.rs`; patch export/review-pack primarily use `view.rs`.

### Remote, HTTP, Signatures

File remote and HTTP remote share the same conceptual operations:

```text
upload
  local sealed branch head + object graph
  -> validate and copy object graph
  -> update remote branch

merge request
  source head + target head snapshot
  -> request record
  -> review/apply validates staleness and object graph
```

Security-sensitive code is split:

| Concern | File |
|---|---|
| file-backed remote mutation | `remote.rs` |
| remote dry-run plan shape | `remote/plans.rs` |
| remote idempotency replay | `remote/idempotency.rs` |
| HTTP parsing, routing, body limits, server loop | `http.rs` |
| TLS configuration | `http_tls.rs` |
| HMAC signing, allowed signers, nonce replay | `signatures.rs` |

Do not mix signature validation into business mutation code unless the caller boundary is explicit.

## 8. Persistence Anatomy

The `.codefire` directory has mutable and immutable zones.

```text
.codefire/
  objects/                         immutable object records
    commits/
    manifests/
    atom_indexes/
    trace_graphs/
    fire_deltas/
    verifications/
    policies/
    resolutions/
    evidence/
    artifacts/

  branches/<branch>.json           mutable branch head pointer
  opened/<branch-or-open>.json      mutable open registry
  active/<open_instance_id>/        mutable working state
    state.json
    scan.json
    fires.json
    resolutions.json
    verification.json

  idempotency/                      local replay records
  locks/                            local lock files
```

The open working directory contains:

```text
.codefire-open
  repo root
  branch
  open instance id
  base commit
```

State rule:

- active state is allowed to change during scan, fire, extinguish, verify, and commit preparation;
- branch head changes only during successful commit, merge apply, or remote branch update flows;
- object records are immutable after publication;
- dry-run should not change any of these zones.

## 9. Command Anatomy Table

| Command | Options source | Runner source | Domain/store source | JSON source | Persistent effect |
|---|---|---|---|---|---|
| `init` | `main.rs::parse_init_args` | `main.rs::init_repo` | `codefire-store` initial objects | `main.rs` plan/data envelope | create repo layout, initial branch |
| `open` | `parse_open_args` | `open_branch_from` | `validate_sealed_commit`, `materialize_commit` | `main.rs` plan/data envelope | working tree, open marker, opened/active records |
| `clone` | `parse_clone_args` | `clone_branch` | store validation and materialization | `main.rs` | copied open directory |
| `status` | `parse_path_json_args` | `read_status` | branch/open validation | `automation::status_data_json` | read-only |
| `scan` | `parse_path_json_args` | `compute_scan` | `codefire-core` scan builders | `automation::scan_data_json` | active scan/fires/state |
| `verify` | `verification::parse_verify_args` | `compute_verify` | core verification + external checks | `automation::verification_*` | active verification/state |
| `fire` | `fire::parse_fire_args` | `fire::run_fire` | core fire basis helpers | `fire::fire_data_json` | active fires |
| `extinguish` | `parse_extinguish_args` / `extinguish_ux` | `run_extinguish` / batch runners | core resolution basis | result/plan helpers | active resolutions/fires |
| `commit` | `parse_commit_args` | `run_commit` | core verify + store object writes | result/plan helpers | object records, branch head, active reset |
| `evidence add` | `evidence::parse_evidence_add_args` | `evidence::run_evidence_add` | store evidence/artifact objects | `evidence_add_data_json` | evidence/artifact objects |
| `link --batch` | `link_batch::parse_link_batch_args` | `link_batch::run_link_batch` | limited YAML/link mutation | module JSON | `codefire.links.yaml` |
| `doctor` | `doctor::parse_doctor_args` | `doctor::run_doctor` | store validation | `doctor/report.rs` | read-only |
| `migrate` | `migration::parse_migrate_args` | `migration::run_migrate` | store/layout checks | `migration.rs` | read-only/planned |
| `storage` | `storage::parse_storage_report_args` | `storage::run_storage_report` | store object grouping | `storage_report_data_json` | read-only |
| `show/diff/review-pack/patch export` | `main.rs` parsers | `view.rs` | store/core semantic diff | view/module helpers | read-only or patch artifact |
| `patch import` | `parse_patch_import_args` | `import_patch` | merge/patch idempotency | `main.rs` | working tree/active state |
| `upload/list/request-*` | `main.rs` remote parsers | `remote.rs` or `http.rs` | store validation + signatures | remote/module helpers | remote project |
| `serve` | `parse_serve_args` | `http::serve_http` | remote handlers + signatures | HTTP JSON | remote project |

## 10. Change Recipes

### Add Or Change A CLI Flag

Update in this order:

1. `docs/04_cli_spec.md` for human command contract.
2. parser in `main.rs` or command module.
3. typed Options/Result if the flag changes behavior.
4. `completion.rs` and `tests/docs.rs` for help/completion.
5. `docs/automation_interface.md` if JSON changes.
6. source navigation docs when the command path changes.

### Add A New Domain Rule

Update in this order:

1. product or design doc that defines the rule.
2. `codefire-core/src/lib.rs` domain model and builder.
3. CLI runner that passes inputs to core.
4. automation diagnostics/next_actions if users or tools must react.
5. tests at core level and CLI JSON level.

### Add A New Object Kind

Update in this order:

1. `docs/08_object_store.md` and `docs/11_internal_architecture.md`.
2. `codefire-store/src/lib.rs` kind registry, path mapping, validation.
3. writer module.
4. `doctor/checks.rs`, `storage.rs`, remote object graph copying.
5. tamper/compatibility tests.

### Add A New Remote Operation

Update in this order:

1. `docs/10_merge_remote_server.md` and automation docs.
2. `remote/plans.rs` dry-run plan.
3. `remote.rs` file-backed implementation.
4. `http.rs` endpoint and request parsing.
5. `signatures.rs` policy if trust boundary changes.
6. remote idempotency and tests.

## 11. Current Structural Debt

These are known gaps between the desired anatomy and current implementation.

| Debt | Current symptom | Preferred direction |
|---|---|---|
| `main.rs` is still too large | parse, workflow, repo services, merge, patch, commit all coexist | extract command modules and repository services gradually |
| shared command registry is partial | help routing and completion are not a single metadata source | introduce command metadata without changing product behavior |
| state derivation is duplicated | status, scan, verify, automation next_actions can drift | create a shared state reducer/service |
| layout vocabulary is repeated | init, doctor, migration, storage each know parts of `.codefire` | centralize layout constants and health vocabulary |
| JSON failure path is uneven | some errors return text even when tools need structured recovery | route failures through common envelope and exit code mapping |
| tests are too concentrated | `tests.rs` remains a broad integration bucket | move touched areas into domain-specific test modules |

Structural debt must be paid down when nearby behavior is touched. Do not create a large unrelated refactor just to satisfy this table.

## 12. Documentation Sync Rule

When source shape changes, update docs by blast radius:

| Source change | Required doc update |
|---|---|
| command route/parser/runner changes | `command-source-trace.md` and possibly `source-code-map.md` |
| module responsibility changes | `source-code-map.md`, this file, `module-boundaries.md` |
| data flow or persistence changes | `source-code-mental-model.md`, `docs/11_internal_architecture.md` |
| product/automation contract changes | `docs/04_cli_spec.md`, `docs/automation_interface.md`, `source-code-blueprint.md` |
| object identity or store layout changes | `docs/08_object_store.md`, `docs/11_internal_architecture.md`, this file |
| issue-driven version planning changes | relevant `docs/development/v*-*.md`, backlog/detail issue files, `history.md` |

The target state is that a contributor can read docs first, name the exact module and function cluster to change, and only then open the source to implement.
