# CodeFire Source Code Reconstruction Guide

この文書は、CodeFireのソースコードをまだ開かなくても、実装の配置、主要型、関数の流れ、永続化境界、変更時に触る場所を頭の中で再構成できるようにするためのガイドである。

`source-code-map.md` は索引、`source-code-anatomy.md` は主要fileの解剖図、`source-data-model-catalog.md` は型とJSONの辞書である。この文書はそれらをつなぎ、「ソースコードはたぶんこう書かれている」と推測できる粒度まで落とす。

## 1. Repository As A Program

CodeFireは、ひとつの巨大CLIではなく、次の5つの機械が同じfilesystemを見て動くプログラムとして読む。

```text
Command machine
  argv, help, parser, output mode, exit code
  primary: crates/codefire-cli/src/main.rs, completion.rs, exit_code.rs

Workflow machine
  open directory, active state, scan/verify/fire/extinguish/commit
  primary: main.rs, cli_model.rs, fire.rs, batch.rs, verification.rs

Domain machine
  Atom, TraceGraph, Fire, Resolution, Verification
  primary: crates/codefire-core/src/lib.rs

Object machine
  canonical JSON, object ID, object record, sealed commit validation
  primary: crates/codefire-store/src/lib.rs

Tooling machine
  automation JSON, metrics, doctor, storage, migrate, explain, context
  primary: automation.rs, metrics.rs, doctor/*, storage.rs, migration.rs, context.rs, explain.rs
```

コードを読む前に、変更対象がどの機械に属するかを決める。機械をまたぐ変更は普通に必要だが、ひとつの関数が3つ以上の機械を直接抱え始めたら、設計が崩れている可能性が高い。

## 2. Workspace Tree You Should Be Able To Draw

実装はこの木として記憶する。

```text
crates/
  codefire-core/
    src/lib.rs
      semantic model
      config and policy parsers
      atom extractors
      scan and verification builders

  codefire-store/
    src/lib.rs
      object kind registry
      canonical JSON and object IDs
      object read/write
      sealed commit validation

  codefire-util/
    src/lib.rs
      sha256, hex, timestamp, path encoding utilities

  codefire-cli/
    src/main.rs
      dispatch, common parsers, open/repo helpers,
      scan/verify/extinguish/commit, merge/patch/open/clone

    src/cli_model.rs
      shared Options, Results, OpenContext, ScanExecution, VerifyExecution

    src/automation.rs
      command_result envelope, diagnostics, next_actions

    src/verification.rs
      verify parser and human text rendering

    src/fire.rs
    src/fire/batch.rs
    src/batch.rs
    src/extinguish_ux.rs
      manual fire and resolution UX

    src/evidence.rs
    src/evidence/batch.rs
      evidence capture and batch evidence add

    src/context.rs
    src/explain.rs
    src/storage.rs
    src/migration.rs
    src/doctor.rs
    src/doctor/checks.rs
    src/doctor/report.rs
      read-only tool and recovery surfaces

    src/view.rs
    src/view/file_diff.rs
    src/view/semantic_diff.rs
      show, diff, review-pack, patch export

    src/remote.rs
    src/remote/idempotency.rs
    src/remote/plans.rs
    src/http.rs
    src/http_tls.rs
    src/signatures.rs
      remote project, HTTP transport, TLS, signatures, nonce replay

    src/limited_yaml.rs
    src/metrics.rs
    src/idempotency.rs
    src/open_clone_idempotency.rs
    src/merge_patch_idempotency.rs
      shared support modules
```

この木にない大きな責務を追加したくなった場合は、新moduleを足すか、既存moduleの責務を文書で更新してから実装する。

## 3. `main.rs` Reconstructed

`main.rs` は長いが、ひとつの連続したアルゴリズムではない。次の帯として読む。

```text
module imports and constants
  -> command modules and shared crates are wired here

state labels
  -> scan_branch_state, status_state

entry and dispatch
  -> main
  -> run
  -> command_help_for_args
  -> command match arms

command wrappers
  -> run_scan_command
  -> run_verify_command
  -> run_fire_command
  -> run_extinguish_command
  -> run_commit_command
  -> local_workflow_handler

render and envelope helpers
  -> print_status, print_branches, JSON helpers, lock error JSON

parser band
  -> parse_init_args
  -> parse_open_args
  -> parse_path_json_args
  -> parse_extinguish_args
  -> parse_commit_args
  -> parse remote/view/patch/merge args

repository lifecycle
  -> init_repo
  -> open_branch_from
  -> clone_branch
  -> materialize_commit
  -> open_operation_plan, clone_operation_plan

merge and patch
  -> merge_branch
  -> apply_merge_result
  -> import_patch

local workflow
  -> compute_scan
  -> compute_verify
  -> run_extinguish
  -> run_commit

repository services
  -> open_context
  -> list_branches
  -> load_branch_record
  -> save_branch_record
  -> opened_registry_path
  -> build_manifest
  -> load_base_atom_index
  -> commit_parents_for_open

filesystem primitives
  -> write_json_atomic
  -> reset_active
  -> locks
  -> find_repo_root
  -> find_open_marker
  -> read_json
  -> required_string
```

`main.rs` へ新しい処理を足すときの判断基準:

| Want to add | Prefer |
|---|---|
| command dispatch only | `main.rs` match arm and thin wrapper |
| command-specific options/result | command module or `cli_model.rs` |
| domain rule | `codefire-core/src/lib.rs` |
| object identity or commit graph validation | `codefire-store/src/lib.rs` |
| JSON automation payload | `automation.rs` or command module JSON helper |
| repository layout operation | repository helper in `main.rs`, future repo service |
| health/recovery observation | `doctor/*`, `storage.rs`, `migration.rs`, `explain.rs` |

## 4. Command Skeletons

ほとんどのcommandは、コード上で次の骨格に分解できる。

```text
public CLI command
  -> parser returns *Options
  -> runner returns *Result or domain execution model
  -> text renderer or automation JSON renderer
```

Mutating commandはさらにこうなる。

```text
parse argv
  -> resolve repo/open context
  -> validate stable inputs
  -> build dry-run plan
  -> if dry-run: return plan
  -> acquire lock
  -> revalidate mutable inputs
  -> write through store or atomic JSON helper
  -> save idempotency result if configured
  -> render typed result
```

Read-only commandはこうなる。

```text
parse argv
  -> resolve repo/open/remote target
  -> validate referenced objects if sealed history is involved
  -> build read model
  -> render text or JSON envelope
```

### Core Local Workflow

```text
scan:
  run_scan_command
    -> parse_path_json_args
    -> run_scan
    -> compute_scan(persist=true)
         -> open_context
         -> build_atom_index(open_dir)
         -> load_base_atom_index(objects, base_commit)
         -> current_trace_graph(open_dir)
         -> parse_trace_policy(open_dir)
         -> build_scan_result(...)
         -> write active scan.json and fires.json
         -> set_open_state(...)
    -> render_scan or automation envelope

verify:
  run_verify_command
    -> verification::parse_verify_args
    -> run_verify
    -> compute_verify(persist=true)
         -> compute_scan(persist=true)
         -> parse_verification_policy(open_dir)
         -> run_verification_commands(open_dir, policy)
         -> build_verification(...)
         -> persist verification.json
         -> set_open_state(...)
    -> print_verification or automation envelope

extinguish:
  run_extinguish_command
    -> parse_extinguish_args or UX module parser
    -> run_extinguish
         -> open_context
         -> load active fires
         -> validate target fire and evidence refs
         -> build_resolution(...)
         -> update fires.json
         -> append/update resolutions.json
         -> set_open_state(...)
    -> result JSON or text

commit:
  run_commit_command
    -> parse_commit_args
    -> run_commit
         -> open_context
         -> acquire repo/branch locks
         -> compute_verify(persist=true)
         -> reject blockers
         -> build_manifest(open_dir)
         -> store content_manifest, atom_index, trace_graph,
            fire_ledger, resolution_ledger, verification, policy
         -> commit_payload(...)
         -> store commit object
         -> save_branch_record
         -> update opened registry base commit
         -> reset_active(open-clean)
    -> result JSON or text
```

これを読めば、local workflowのコードは「作業treeを読む、active stateを書く、commit時だけsealed objectとbranch headを書く」という形に見えるはずである。

## 5. Data Crossing Points

CodeFireで重要なのは、データが層を越える地点である。

| Crossing | From | To | Boundary object |
|---|---|---|---|
| argv to typed command | raw strings | CLI runner | `*Options` |
| open path to workflow | filesystem | local workflow | `OpenContext` |
| source files to semantics | open directory | core domain | `AtomIndex`, `TraceGraph` |
| scan to verification | core scan | core verification | `ScanResult`, resolutions, failed checks |
| active state to sealed history | mutable open state | object store | content manifest, fire ledger, resolution ledger, verification |
| object payload to identity | JSON payload | immutable store | `ObjectRecord` |
| result to tools | typed result | JSON public API | command result envelope |

When debugging, inspect these crossing objects before reading every helper.

## 6. Persistence You Should Picture

The repository has immutable and mutable zones.

```text
.codefire/
  objects/
    commits/
    content_manifests/
    atom_indexes/
    trace_graphs/
    fire_ledgers/
    resolution_ledgers/
    verifications/
    policies/
    blobs/
    evidence/
    artifact_refs/
    branches/

  branches/
    <branch>.json

  opened/
    <branch>.json

  active/
    <open_instance_id>/
      state.json
      scan.json
      fires.json
      resolutions.json
      verification.json

  locks/
  idempotency/
```

Open directory:

```text
<open-dir>/
  .codefire-open
  user files...
```

Rules:

- `objects/*` is immutable content-addressed history.
- `branches/*.json` is the mutable head pointer for each branch.
- `opened/*.json` connects a branch to an open directory and active state.
- `active/*` is mutable derived state for the open directory.
- `.codefire-open` is the working directory marker that points back to the repo/open identity.
- Working file blobs are not stored in `active/*`; they are sealed only through commit objects.

## 7. Object Store Reconstructed

Object identity has this shape.

```text
payload Value
  -> canonical_json(payload)
  -> object_digest(type_tag + NUL + canonical payload)
  -> object_id(prefix + shortened digest)
  -> ObjectRecord { object_id, type, hash, payload }
  -> .codefire/objects/<subdir>/<object_id>.json
```

Read validation does not merely parse JSON. It checks:

- object ID matches payload digest;
- `hash` matches payload digest;
- filename stem matches object ID;
- object subdirectory matches object kind;
- sealed commit roots have the expected object types;
- nested references such as manifest blobs and evidence artifact refs exist.

Therefore, code that wants an object must call `read_object` or `read_object_record`, not manually read a JSON file and trust it.

## 8. Core Domain Reconstructed

`codefire-core/src/lib.rs` reads as four pipelines.

```text
Atom pipeline:
  Config::parse(open_dir)
    -> rel_files(open_dir)
    -> extract_markdown_atoms
    -> extract_explicit_cf_atoms
    -> path/config fallback atoms
    -> AtomIndex

Trace pipeline:
  codefire.links.yaml
    -> parse_links
    -> TraceLinkInput
    -> TraceLink with deterministic IDs/hashes
    -> TraceGraph

Policy pipeline:
  codefire.policy.yaml
    -> parse_trace_policy
    -> parse_verification_policy
    -> RequiredLinkRule and VerificationCommand lists

Scan/verification pipeline:
  current AtomIndex + base AtomIndex + TraceGraph + TracePolicy + resolutions
    -> changed_atoms
    -> missing required links
    -> Fire list
    -> ScanResult
    -> Verification with blockers/warnings
```

Core structs are serializable domain records, not CLI view models. If a field is added there, downstream JSON, sealed commit objects, docs, and tests probably need changes.

## 9. Automation Reconstructed

Automation output is public API. Mentally reconstruct it as:

```text
typed result or domain execution
  -> command-specific data_json helper
  -> diagnostics_json helper
  -> next_actions helper
  -> command_result_envelope
```

The envelope has this role split:

| Field | Meaning |
|---|---|
| `ok` | command success at process/contract level |
| `exit_code` | stable process exit taxonomy |
| `command` | canonical CLI command label |
| `schema` | command-specific data schema |
| `data` | stable payload for tools |
| `diagnostics` | problems, warnings, and blockers |
| `next_actions` | executable operational suggestions |
| `metrics` | optional command measurement and counts |

Do not put mutation behavior in `automation.rs`; it should translate already-computed results.

## 10. Feature Modules Reconstructed

### Evidence

```text
parse_evidence_add_args
  -> EvidenceAddOptions
  -> run_evidence_add
       -> validate one capture mode
       -> dry-run plan
       -> optional artifact_ref object
       -> optional command capture
       -> evidence object
  -> evidence_add_data_json or text
```

Evidence code owns bounded command output, timeout handling, artifact references, and batch validation.

### Storage

```text
parse_storage_report_args
  -> StorageReportOptions
  -> run_storage_report
       -> find repo root
       -> scan .codefire/objects
       -> scan active/idempotency areas
       -> inspect evidence artifact refs
       -> optionally scan remote projects
  -> storage_report_data_json or text
```

Storage must count retained bytes and warn; it must not repair.

### Doctor And Migration

```text
doctor:
  parse options
  -> run_checks
       -> layout
       -> objects
       -> branch heads
       -> opened registry
       -> active state shape
  -> report JSON/text

migrate:
  parse options
  -> inspect format compatibility
  -> dry-run/check report
```

Doctor says what is wrong. Migration says what would need to change. Neither should silently mutate unless an explicit future apply command is introduced.

### View, Diff, Review, Patch

```text
commitish string
  -> resolve local or remote commitish
  -> validate sealed commit
  -> load roots and manifests
  -> file diff and semantic diff
  -> show/diff/review-pack/patch payload
```

`view.rs` is the coordinator; file-level delta belongs in `view/file_diff.rs`; Atom/Trace/policy impact belongs in `view/semantic_diff.rs`.

### Remote And HTTP

```text
file remote operation
  -> parse cf:// or path-like remote
  -> dry-run plan
  -> optional signature/idempotency
  -> lock remote project
  -> validate object graph
  -> mutate remote branches or merge request records

HTTP operation
  -> parse HTTP URL
  -> client JSON request
  -> server route in http.rs
  -> call same remote semantics
```

Security-sensitive work is split across `http_tls.rs` and `signatures.rs`; remote business mutation should not hide transport trust checks.

## 11. Change Reconstruction Recipes

### Add A New CLI Command

You should expect to touch:

1. `docs/04_cli_spec.md`
2. `completion.rs`
3. parser/options/result in a command module or `cli_model.rs`
4. `main.rs` dispatch and thin wrapper
5. runner module
6. text renderer
7. JSON helper and `automation_interface.md`
8. command-source trace docs
9. focused tests plus docs/help tests

### Add A New Scan Rule

You should expect to touch:

1. trace/index policy docs
2. `codefire-core/src/lib.rs`
3. `compute_scan` input plumbing only if new data is required
4. automation diagnostics and next_actions
5. source code blueprint/map
6. core tests and scan JSON tests

### Add A New Object Root

You should expect to touch:

1. object store docs
2. `codefire-store/src/lib.rs` kind/root validation
3. writer in commit/evidence/feature module
4. doctor checks
5. storage grouping
6. remote object graph copy
7. sealed commit validation tests

### Add A New Remote Mutation

You should expect to touch:

1. remote/server docs
2. `remote/plans.rs`
3. `remote.rs`
4. `http.rs`
5. `signatures.rs` if trust boundary changes
6. remote idempotency helpers
7. file-remote and HTTP tests

## 12. Reading Checklist

Before implementing, the developer should be able to answer:

- Which command owns the user-facing contract?
- Which `*Options` type represents argv?
- Which runner owns mutation?
- Which domain struct represents the semantic result?
- Which files under `.codefire/` are read and written?
- Which object records are sealed into commit history?
- Which automation payload and diagnostics change?
- Which tests prove the behavior at parser, domain, repository, and JSON layers?

If these answers are not obvious from docs, update the source docs before or during implementation.
