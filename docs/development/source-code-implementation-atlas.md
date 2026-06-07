# CodeFire Source Code Implementation Atlas

この文書は、CodeFireのRust sourceを開かなくても「どのfileに何があり、どの関数が何を読み書きし、どの型がどのJSON/状態に対応するか」を具体的に想像するための実装アトラスである。

`source-code-map.md` は入口、`source-code-blueprint.md` は要求からsourceへの対応、`source-code-mental-model.md` は処理の流れ、`source-data-model-catalog.md` は型とJSONの辞書である。この文書はそれらをさらに実装寄りにして、crate、module、関数帯、主要型、永続化先を1枚の地図としてまとめる。

## 1. Repository Snapshot

現在の本流実装はRust workspaceである。Pythonの `codefire` はreference/fallbackとして残るが、新規機能の正本はRust側に置く。

```text
Cargo.toml
crates/
  codefire-core/       semantic domain: Atom, Trace, Fire, Verification
  codefire-store/      immutable object identity and sealed commit validation
  codefire-util/       hash, timestamp, path encoding helpers
  codefire-cli/        command dispatch, repository mutation, rendering, remote

docs/
  01_product_definition.md      product contract
  04_cli_spec.md                command contract
  05_consistency_model.md       commit/verification invariants
  08_object_store.md            object model
  09_index_trace_policy.md      Atom/Trace policy
  10_merge_remote_server.md     remote and merge behavior
  11_internal_architecture.md   architecture source of truth

docs/development/
  source-code-map.md
  source-code-implementation-atlas.md
  source-code-blueprint.md
  source-code-mental-model.md
  source-code-anatomy.md
  source-data-model-catalog.md
  command-source-trace.md
```

Implementation scale at the time of this atlas:

| Area | Approx lines | Meaning |
|---|---:|---|
| `crates/codefire-cli/src/main.rs` | 4,913 | command spine, repo/open workflow, remaining large orchestration |
| `crates/codefire-cli/src/tests.rs` | 5,558 | broad CLI regression suite |
| `crates/codefire-core/src/lib.rs` | 2,609 | semantic Atom/Trace/Fire/Verification engine |
| `crates/codefire-store/src/lib.rs` | 1,132 | object record and sealed commit engine |
| `crates/codefire-cli/src/remote.rs` | 1,487 | file-backed remote mutation and object graph copy |
| `crates/codefire-cli/src/http.rs` | 1,443 | HTTP/HTTPS transport server/client |
| `crates/codefire-cli/src/signatures.rs` | 956 | commit/request HMAC policy and nonce replay |
| command modules | 100-950 each | evidence, storage, context, doctor, migrate, view, batch |

This distribution matters: most architectural debt is not that logic is absent; it is that `main.rs` still owns several repository services that should become smaller modules.

## 2. Dependency Picture

```text
CLI argv
  |
  v
codefire-cli::main::run
  |
  +-- completion/help/error/exit-code
  +-- command parsers and Options in cli_model.rs or feature modules
  +-- local repo/open workflow in main.rs
  +-- feature modules
  |     evidence, fire, batch, link_batch, context, explain,
  |     doctor, migration, storage, view, remote, http, signatures
  |
  +-- codefire-core
  |     build_atom_index, current_trace_graph,
  |     build_scan_result, build_verification, build_resolution
  |
  +-- codefire-store
        object_record, store_object, read_object,
        commit_payload, validate_sealed_commit
```

Allowed direction:

```text
cli -> core
cli -> store
cli -> util
store -> util
core -> util
```

Forbidden direction:

```text
core -> cli rendering
store -> open-directory state
automation.rs -> filesystem mutation
parser -> filesystem writes
```

## 3. `main.rs` Spine

`crates/codefire-cli/src/main.rs` is still the central spine. Read it as bands.

| Line anchor | Band | What it contains | Source smell when it grows |
|---:|---|---|---|
| 13 | module wiring | imports command modules | adding a module without source-map/docs updates |
| 105 | state helpers | `scan_branch_state`, `status_state` | state rules duplicated elsewhere |
| 121 | process entry | `main` | command side effects before help/error routing |
| 130 | top-level dispatch | `run(args)` command match | large command-specific logic in match arms |
| 767 | help routing | `wants_help`, `command_help_for_args`, `subcommand_help` | per-command help aliases drifting |
| 927 | error model | `CliError` and conversions | plain text errors escaping JSON mode |
| 1024 | local workflow table | `local_workflow_handler` | scan/verify/fire/extinguish/commit dispatch divergence |
| 1035 | workflow wrappers | `run_scan_command` ... `run_commit_command` | parse/run/render mixed with domain logic |
| 1238 | text/JSON rendering helpers | status, branch, scan, plan/data envelope | JSON manually constructed as text |
| 1368 | parser band | `parse_*_args`, common option helpers | parser starts mutating files |
| 2428 | repo lifecycle | `init_repo`, `open_branch`, `clone_branch`, `merge_branch` | branch/open invariants duplicated |
| 3293 | scan/verify | `compute_scan`, `compute_verify` | core semantics embedded in CLI |
| 3421 | extinguish | `run_extinguish` | resolution basis/evidence validation spread out |
| 3642 | commit | `run_commit` | object write ordering hidden or changed without tests |
| 4021 | base/object helpers | `load_base_atom_index`, commit/object helpers | sealed validation bypassed |
| 4151 | filesystem primitives | `write_json_atomic`, `read_json`, locks | partial writes or unsynced publication |
| 4192 | status/read state | `read_status` | state labels invented outside reducer |
| 4389 | open context | `open_context` | copied open dirs accepted without registry validation |
| 4522 | manifest | `build_manifest` | blob/object storage duplicated |
| 4846 | discovery | `find_repo_root`, `find_open_marker` | path discovery differs across commands |

The mental skeleton is:

```text
main
  -> run
     -> optional help short-circuit
     -> local_workflow_handler for scan/verify/fire/extinguish/commit
     -> command match for other commands
        -> parse typed options
        -> call runner
        -> render text or JSON envelope
```

For mutating commands:

```text
parse
  -> discover repo/open context
  -> validate immutable inputs
  -> if dry-run: return plan
  -> lock
  -> revalidate mutable inputs
  -> write objects/state/branch heads atomically
  -> store idempotency result when requested
  -> render typed result
```

## 4. CLI Command Ownership

| Command | Dispatch owner | Parser/model owner | Runner owner | JSON owner | Writes |
|---|---|---|---|---|---|
| `init` | `main.rs::run` | `main.rs::parse_init_args`, `InitOptions` | `main.rs::init_repo` | text only currently | `.codefire`, initial objects, main branch |
| `open` | `main.rs::run` | `OpenOptions` | `main.rs::open_branch_from` | plan envelope | open dir, `.codefire-open`, opened registry |
| `branch list` | `main.rs::run` | `PathJsonOptions` | `main.rs::list_branches` | `command_result_envelope` | none |
| `status` | `main.rs::run` | `PathJsonOptions` | `main.rs::read_status` | `automation::status_data_json` | none |
| `scan` | `local_workflow_handler` | `PathJsonOptions` | `compute_scan(persist=true)` | `automation::scan_data_json` | active `scan.json`, `fires.json`, `state.json` |
| `verify` | `local_workflow_handler` | `verification.rs::parse_verify_args` | `compute_verify(persist=true)` | `automation::verification_data_json` | active `verification.json`, `state.json` |
| `fire` | `local_workflow_handler` | `fire.rs` | `fire.rs::run_fire` | `fire_data_json` | active `fires.json` |
| `extinguish` | `local_workflow_handler` | `ExtinguishOptions` | `main.rs::run_extinguish` | result plan in `main.rs` | active `resolutions.json`, `fires.json` |
| `commit` | `local_workflow_handler` | `CommitOptions` | `main.rs::run_commit` | result plan in `main.rs` | objects, branch head, active reset |
| `clone` | `main.rs::run` | `CloneOptions` | `main.rs::clone_branch` | plan envelope | new branch/open dir |
| `merge` | `main.rs::run` | `MergeOptions` | `main.rs::merge_branch` | plan envelope | target open dir and active state |
| `show` | `main.rs::run` | positional target | `view.rs::show_commitish` | text currently | none |
| `diff` | `main.rs::run` | `DiffArgs` | `view.rs::diff_commitish_with_options` | text currently | none |
| `review-pack` | `main.rs::run` | `ReviewPackArgs` | `view.rs::review_pack_with_options` | text/file currently | optional output file |
| `patch export` | `main.rs::run` | `PatchExportArgs` | `view.rs::patch_export_with_options` | text/file currently | optional output file |
| `patch import` | `main.rs::run` | `PatchImportOptions` | `main.rs::import_patch` | data envelope | open dir and active state |
| `link` | `main.rs::run` | `link_batch.rs` | `link_batch.rs::run_link_batch` | `link_batch_data_json` | `codefire.links.yaml` |
| `evidence add` | `main.rs::run` | `evidence.rs` | `evidence.rs::run_evidence_add` or batch | `evidence_add_data_json` | evidence/artifact objects |
| `context` | `main.rs::run` | `context.rs` | `context.rs::build_context_pack` | command envelope | none |
| `explain` | `main.rs::run` | `explain.rs` | `explain.rs::run_explain` | command envelope | none |
| `storage report` | `main.rs::run` | `storage.rs` | `storage.rs::run_storage_report` | `storage_report_data_json` | none |
| `doctor` | `main.rs::run` | `doctor.rs` | `doctor.rs::run_doctor` | doctor report JSON | none |
| `migrate check/dry-run` | `main.rs::run` | `migration.rs` | `migration.rs::run_migrate` | migration report JSON | none for current modes |
| `upload` | `main.rs::run` | `remote.rs::UploadOptions` | `remote.rs::upload_branch` | plan envelope | remote objects/branch |
| `list` | `main.rs::run` | `RemoteProjectOptions` | `remote.rs::list_remote_branches` | text currently | none |
| `request-*` | `main.rs::run` | `remote.rs` | `remote.rs` runners | plan/text | remote MR records and branch heads |
| `serve` | `main.rs::run` | `ServeOptions` | `http.rs::serve_http` | HTTP JSON | remote storage root |

When a command changes, update this table if its owner, JSON contract, or write targets change.

## 5. Core Semantic Engine

`crates/codefire-core/src/lib.rs` owns semantic meaning. It does not own process output or repository mutation.

| Zone | Anchor | Main structs/functions | Output |
|---|---:|---|---|
| object wrapper | 14 | `ObjectId` | typed ID wrapper |
| domain structs | 58 | `Selector`, `Atom`, `AtomIndex`, `TraceLink`, `TraceGraph` | source/trace data |
| policy structs | 101 | `RequiredLinkRule`, `TracePolicy`, `VerificationPolicy`, `VerificationCommand` | trace and verification policy |
| fire/verification structs | 133 | `MissingRequiredLink`, `Fire`, `ScanResult`, `Verification` | scan and commit gate data |
| resolution structs | 220 | `ResolutionBasis`, `Resolution`, `ResolutionRequest` | extinguish basis and evidence refs |
| Atom indexer | 266 | `build_atom_index` | `AtomIndex` |
| Trace Link parser | 307 | `current_trace_graph`, `parse_links` | `TraceGraph` |
| policy parser | 420 | `parse_trace_policy`, `parse_verification_policy` | policy structs |
| verification builder | 792 | `build_verification` | `Verification` |
| stale resolution detection | 836 | `stale_resolutions` | stale resolution list |
| resolution builder | 891 | `build_resolution` | `Resolution` |
| required-link engine | 947 | `required_link_missing`, `current_required_link_missing` | missing link diagnostics |
| scan builder | 1024 | `build_scan_result` | `ScanResult` |
| extractor internals | 1346 | Markdown and explicit `cf-atom:` extractors | `Atom` values |
| config/path internals | 1643 | `Config`, pattern matching, file traversal | extractor inputs |

Data flow:

```text
open_dir files
  -> Config::parse
  -> rel_files and extractors
  -> AtomIndex
  -> current_trace_graph
  -> TracePolicy / VerificationPolicy
  -> build_scan_result
  -> build_verification
```

Change rule:

- New Atom kind: update config/extractor, docs/09, core tests, scan JSON expectations.
- New fire reason: update `Fire`, scan builder, automation diagnostics, verify/commit tests.
- New verification blocker: update policy, `build_verification`, exit code, automation docs/tests.

## 6. Store and Sealed Identity

`crates/codefire-store/src/lib.rs` owns immutable identity. Its central invariant is:

```text
object_id = object_prefix(type_tag) + "-" + first 24 hex chars of
            sha256(type_tag + NUL + canonical_json(payload))
```

The current object kind registry:

| Type tag | ID prefix | Subdir |
|---|---|---|
| `blob` | `CF-BLOB` | `blobs` |
| `content_manifest` | `CF-MANIFEST` | `content_manifests` |
| `atom_index` | `CF-ATOMINDEX` | `atom_indexes` |
| `trace_graph` | `CF-TRACE` | `trace_graphs` |
| `fire_ledger` | `CF-FIRELEDGER` | `fire_ledgers` |
| `resolution_ledger` | `CF-RESOLUTION` | `resolution_ledgers` |
| `verification` | `CF-VERIFY` | `verifications` |
| `policy` | `CF-POLICY` | `policies` |
| `artifact_ref` | `CF-ARTIFACT` | `artifact_refs` |
| `evidence` | `CF-EVIDENCE` | `evidence` |
| `commit` | `CF-COMMIT` | `commits` |
| `branch` | `CF-BRANCH` | `branches` |

Function groups:

| Zone | Anchor | Functions | Responsibility |
|---|---:|---|---|
| registry | 22 | `OBJECT_KINDS`, `object_prefix`, `object_subdir` | type/prefix/path map |
| errors | 47 | `StoreError` | corruption and validation taxonomy |
| record | 103 | `ObjectRecord` | wrapper schema |
| identity | 111 | `canonical_json`, `object_digest`, `object_id` | deterministic IDs |
| validation | 182 | `validate_object_record` | record/hash/filename consistency |
| persistence | 237 | `object_record_path`, `store_object`, `read_object` | publish/read records |
| sealed commit | 354 | `validate_sealed_commit` and reference validators | commit graph trust |
| commit builder | 617 | `commit_payload` | commit root/certificate shape |

If an object kind is added, update all of these together: store registry, doctor checks, storage report, remote object copy, internal architecture docs, object store docs, tests.

## 7. Shared CLI Types

`crates/codefire-cli/src/cli_model.rs` is the vocabulary for parser/runner/rendering handoff.

```text
Status, Branch
  read-only state rows

InitOptions/OpenOptions/CloneOptions/MergeOptions/PatchImportOptions
  repository lifecycle and mutating command inputs

PathJsonOptions
  common --path/--json/--metrics model

ExtinguishOptions/CommitOptions
  local workflow mutation inputs

OpenContext
  repo_root, open_dir, branch, registry_path, registry

ScanExecution/VerifyExecution
  domain result plus context needed by rendering/persistence
```

Design rule: a command that has nontrivial arguments or JSON output should return a typed result before it becomes `serde_json::Value`.

## 8. Feature Module Cards

| Module | Internal shape | Reads | Writes | Key risk |
|---|---|---|---|---|
| `automation.rs` | envelope, data helpers, diagnostics, next_actions | command/domain results | none | public JSON drift |
| `metrics.rs` | `CommandMetrics`, phase/counter/cache JSON | elapsed times, result counts | none | fake precision or unstable fields |
| `exit_code.rs` | `ExitCode`, mapper functions | verification/core/store errors | process code only | inconsistent JSON/text exit semantics |
| `completion.rs` | help text and shell completion | static command surface | stdout | help/docs/completion drift |
| `verification.rs` | verify parser and human renderer | policy/result | stdout | JSON/text filter mismatch |
| `fire.rs`, `fire/batch.rs` | manual fire options, batch validation | active fires, atom index | active fires | partial batch writes |
| `batch.rs` | extinguish batch | active fires/resolutions | active resolutions/fires | partial writes or stale basis |
| `link_batch.rs` | Trace Link batch parser/writer | batch YAML, links file | `codefire.links.yaml` | invalid YAML append location |
| `evidence.rs`, `evidence/batch.rs` | evidence capture, artifact refs, command capture | artifact files, shell/argv command | evidence/artifact objects | empty dry-run IDs, absolute path leakage |
| `context.rs` | context pack selector and active scan snapshot | active scan/fires, working tree | none | stale context or unbounded payload |
| `explain.rs` | fire/atom/verify/storage explanations | active state, storage report | none | says passed while blockers exist |
| `doctor/*` | health checks and report rendering | layout, records, branches, active state | none | missing repairability or corruption category |
| `migration.rs` | compatibility report/action plan | layout, objects, branches | none currently | migration and doctor layout drift |
| `storage.rs` | object/artifact/remote capacity scan | objects, remote dirs, nonce/idempotency records | none | object type misclassification |
| `view.rs` | show/diff/review-pack/patch export orchestration | sealed commits and manifests | optional output file | unvalidated commitish or raw JSON drift |
| `view/file_diff.rs` | bounded diff and rename/copy logic | manifest blobs | none | O(n^2) behavior on large files |
| `view/semantic_diff.rs` | Atom/Trace/policy impact diff | sealed roots | none | semantic impact under-reporting |
| `remote.rs`, `remote/*` | file-backed remote, plans, idempotency | local and remote objects | remote objects/branches/MRs | trust boundary bypass |
| `http.rs`, `http_tls.rs` | transport and TLS | HTTP requests, cert/key files | remote project through handlers | auth/signature divergence from file remote |
| `signatures.rs` | commit/request signatures, nonce replay | policy, env keys, commits | nonce cache | time/window parsing and replay |
| `limited_yaml.rs` | shared limited YAML parser | scalar/key-value YAML subset | none | parser variants diverging |

## 9. Persistent State Layout

Local repository:

```text
.codefire/
  repo.json
  objects/
    blobs/
    content_manifests/
    atom_indexes/
    trace_graphs/
    fire_ledgers/
    resolution_ledgers/
    verifications/
    policies/
    artifact_refs/
    evidence/
    commits/
    branches/
  branches/<encoded-branch>.json
  opened/<encoded-branch>.json
  active/<open_instance_id>/
    state.json
    scan.json
    fires.json
    resolutions.json
    verification.json
  locks/
  idempotency/
```

Open directory marker:

```text
.codefire-open
  repo_root
  branch
  open_instance_id
  base_commit
```

Remote project:

```text
<remote-root>/<org>/<app>/
  objects/
  branches/
  merge_requests/
  idempotency/
  nonce_cache/
  generations/
  audit/
  server_policy.json
```

Commit writes:

```text
working tree files
  -> blob objects
  -> content_manifest object
  -> atom_index object
  -> trace_graph object
  -> fire_ledger object
  -> resolution_ledger object
  -> verification object
  -> policy object
  -> commit object
  -> branch head JSON
  -> active state reset
```

## 10. Change Recipes

### Add A New Read-Only Command

Touch:

1. `completion.rs` for help/completion.
2. `main.rs::run` dispatch.
3. New or existing command module with `Options`, `Result`, parser, runner, renderer.
4. `automation.rs` if JSON is public.
5. `docs/04_cli_spec.md`, `docs/automation_interface.md`, this atlas.
6. CLI tests and docs/help consistency tests.

Do not write files in parser code. Do not emit raw JSON outside envelope unless the command is intentionally a debug-only exception and documented as such.

### Add A New Mutating Command

Touch:

1. typed Options/Result in `cli_model.rs` or feature module;
2. dry-run plan type;
3. lock/idempotency path;
4. validation before and after lock;
5. atomic write path;
6. JSON envelope and next_actions;
7. doctor/storage/migration visibility if new state appears;
8. tests for dry-run, JSON, lock conflict, idempotency, rollback/non-partial write.

### Add A New Object Kind

Touch:

1. `codefire-store::OBJECT_KINDS`;
2. object writer;
3. sealed commit validator if referenced from commits;
4. remote copy/reference traversal;
5. doctor object validation;
6. storage report grouping;
7. object store docs and schema examples;
8. tamper and missing reference tests.

### Add A New Atom Extractor

Touch:

1. `codefire-core::Config` defaults and parser;
2. extractor function and Atom ID/hash tests;
3. trace policy docs;
4. scan/verify regression;
5. automation/context behavior if new Atom kind affects next_actions.

### Fix A JSON Contract Bug

Touch:

1. command parser to accept shared `--path`, `--json`, `--metrics` where applicable;
2. runner result model if current output is raw text;
3. `automation.rs` envelope/data/diagnostic helper;
4. `exit_code.rs` mapping for failure mode;
5. docs/automation interface;
6. one golden-ish test asserting `type`, `ok`, `command`, `data`, `diagnostics`, `next_actions`.

## 11. Review Checklist

Before merging source changes, answer these questions from docs alone:

- Which command owns the user-facing contract?
- Which parser converts argv to typed options?
- Which runner can mutate state?
- Which core/store invariant applies?
- Which `.codefire` path can change?
- Is JSON produced through `command_result_envelope`?
- Are human output and JSON output generated separately?
- Are errors mapped to stable exit codes?
- Does doctor/storage/migrate need to know about the new data?
- Does remote copy/validation need to know about the new object/reference?
- Is there a test at the smallest useful layer?
- Did the source navigation docs change when source ownership changed?

If the answer requires opening source for basic ownership, this atlas is incomplete and should be updated in the same change.
