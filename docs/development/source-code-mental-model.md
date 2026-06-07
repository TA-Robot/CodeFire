# CodeFire Source Code Mental Model

この文書は、CodeFireのソースコードをまだ開いていない状態でも、実装の形を頭に描けるようにするための詳細設計地図である。

`docs/development/source-code-map.md` は「どのfileを見るか」の索引、`docs/development/source-code-blueprint.md` は要求・設計・実装・テストの対応表、この文書は「そのfileの中で何がどうつながっているか」の説明である。関数名やmodule名はRust実装に合わせる。

## 1. Big Picture

CodeFireは、Gitのように作業treeを直接履歴化するのではなく、open directoryで作業し、Atom、Trace Link、Fire、Resolution、Verificationを検査してからsealed commitを作るCLIである。

実装の依存方向は次の形で読む。

```text
user / AI agent
  |
  v
codefire-cli/src/main.rs
  command dispatch, path resolution, locks, open-state orchestration
  |
  +--> command modules
  |      evidence.rs, storage.rs, doctor.rs, migration.rs, context.rs,
  |      fire.rs, batch.rs, verification.rs, view.rs, remote.rs, http.rs
  |
  +--> automation.rs / metrics.rs / exit_code.rs
  |      JSON envelope, diagnostics, next_actions, metrics, stable exit codes
  |
  +--> codefire-core
  |      Atom extraction, Trace Graph, scan, verification semantics
  |
  +--> codefire-store
         canonical JSON, object records, sealed commit validation
```

Rule of thumb:

- `codefire-core` decides what the project means.
- `codefire-store` decides what a persisted object is.
- `codefire-cli` decides how commands move data between filesystem, core, store, and output.
- `automation.rs` decides how machine-readable command output is shaped.

## 2. Workspace Shape

```text
crates/
  codefire-core/
    src/lib.rs
      Atom, TraceGraph, Fire, ScanResult, Verification
      config/policy/link parsing
      build_atom_index, current_trace_graph, build_scan_result, build_verification

  codefire-store/
    src/lib.rs
      ObjectRecord, canonical_json, object_id, store_object, read_object
      object_record_path, validate_sealed_commit, commit_payload

  codefire-util/
    src/lib.rs
      sha256, hex, UTC timestamp, percent path encoding

  codefire-cli/
    src/main.rs
      process entry, command match, common parsers, local workflow runners,
      open/clone/merge/commit orchestration, repo/open helpers, locks

    src/cli_model.rs
      shared CLI option/result/context structs

    src/automation.rs
      command_result_envelope and command-specific JSON data/diagnostics/actions

    src/verification.rs
      verify command parser and human rendering

    src/fire.rs, src/fire/batch.rs, src/batch.rs, src/extinguish_ux.rs
      manual fire, batch fire, batch extinguish, interactive/all extinguish UX

    src/evidence.rs, src/evidence/batch.rs
      evidence capture from artifact/URI/shell/argv and evidence batch

    src/storage.rs
      object store, artifact, remote storage report

    src/doctor.rs, src/doctor/checks.rs, src/doctor/report.rs, src/migration.rs
      health checking and migration reporting

    src/view.rs, src/view/file_diff.rs, src/view/semantic_diff.rs
      show, diff, review-pack, patch export/import support

    src/remote.rs, src/http.rs, src/http_tls.rs, src/signatures.rs
      file remote, HTTP/HTTPS server/client, signatures, replay protection
```

## 3. `main.rs` Anatomy

`main.rs` is still the densest file. Read it as bands rather than as one long file.

| Band | Approx area | What it owns |
|---|---:|---|
| module imports and shared helpers | top | module wiring and imported command surfaces |
| `scan_branch_state`, `status_state` | early | current local state labels |
| `main`, `run` | early | process entry and top-level command dispatch |
| help and command wrappers | after dispatch | `wants_help`, `subcommand_help`, `run_*_command` wrappers |
| output helpers | middle | text rendering, JSON envelope helpers, lock contention JSON |
| argument parsers | middle | `parse_*_args`, common mutation/path/lock parsing |
| init/open/clone/merge/patch | middle-late | repository materialization and branch/session operations |
| scan/verify/extinguish/commit | late | local workflow mutation and persistence |
| repository services | late | branch records, opened registry, manifest, open context |
| filesystem primitives | end | JSON atomic write, locks, repo discovery, base64, path helpers |

Current desired direction:

- command-specific orchestration should gradually move out of `main.rs`;
- thin wrappers in `main.rs` should parse, call module functions, and render;
- shared behavior such as path parsing, state derivation, and command help should become common services.

## 4. Command Execution Pattern

Most commands should fit this mental template.

```text
run(args)
  -> select command
  -> parse_*_args
  -> run_* or compute_*
  -> if --json:
       automation::command_result_envelope(...)
     else:
       print_* or render_*
```

Mutating commands add dry-run, lock, and idempotency gates.

```text
parse
  -> resolve repo/open context
  -> validate stable inputs
  -> if dry-run: build operation plan and return
  -> acquire lock
  -> revalidate mutable inputs
  -> write objects / active state / branch head atomically
  -> save idempotency result if requested
  -> render result
```

This pattern is visible in:

- `run_extinguish_command` -> `parse_extinguish_args` -> `run_extinguish`
- `run_commit_command` -> `parse_commit_args` -> `run_commit`
- `evidence add` dispatch -> `parse_evidence_add_args` -> `run_evidence_add`
- remote request commands -> parse -> plan/dry-run -> lock -> remote mutation

## 4.1 In-Memory Shapes While A Command Runs

CodeFireのcommandは、文字列やJSONを直接持ち回すのではなく、できるだけ次の中間構造を経由して読む。

```text
argv
  -> *Options
  -> OpenContext or repo_root
  -> domain result
  -> command result struct
  -> human text or automation envelope
```

Typical examples:

| Command | Options | Context | Domain result | Render source |
|---|---|---|---|---|
| `status` | `PathJsonOptions` | repo/open discovery inside `read_status` | `Status` | `print_status`, `status_data_json` |
| `scan` | `PathJsonOptions` | `OpenContext` in `compute_scan` | `ScanExecution`, `ScanResult` | `render_scan`, `scan_data_json` |
| `verify` | `VerifyOptions` | `OpenContext` in `compute_verify` | `VerifyExecution`, `Verification` | `render_verification`, `verification_data_json` |
| `fire` | `FireOptions` | command module resolves open context | `FireResult` | `print_fire_result`, `fire_data_json` |
| `extinguish` | `ExtinguishOptions` | `OpenContext` | `ExtinguishResult` | text helper in `main.rs`, automation data in `main.rs` |
| `commit` | `CommitOptions` | `OpenContext` | `CommitResult` | text helper in `main.rs`, plan/data envelope |
| `evidence add` | `EvidenceAddOptions` | repo root plus optional open context | `EvidenceAddResult` | `print_evidence_add_result`, `evidence_add_data_json` |
| `storage` | `StorageReportOptions` | repo root and remote URLs | `StorageReport` | `print_storage_report`, `storage_report_data_json` |

Design smell:

- if a runner returns raw `serde_json::Value` before mutation has finished, the business result model is probably missing;
- if a parser writes files or a renderer performs validation, the command lifecycle has leaked across layers;
- if `automation.rs` needs to know internal lock or filesystem steps, the command result is not expressive enough.

## 4.2 Function-Level Flow For Local Workflow

Use this as a concrete call graph when reading the local workflow code.

```text
run(args)
  -> local_workflow_handler(command)
  -> run_scan_command / run_verify_command / run_fire_command /
     run_extinguish_command / run_commit_command
```

Scan:

```text
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
       -> set_open_state(...)
       -> write active scan/fires when persist=true
  -> render_scan or command_result_envelope(scan_data_json)
```

Verify:

```text
run_verify_command
  -> verification::parse_verify_args
  -> run_verify
  -> compute_verify(persist=true)
       -> compute_scan(persist=true)
       -> parse_verification_policy(open_dir)
       -> run_verification_commands(open_dir, policy)
       -> build_verification(...)
       -> persist_verification_result_state
       -> set_open_state(verification_open_state)
  -> render_verification or command_result_envelope(verification_data_json)
```

Commit:

```text
run_commit_command
  -> parse_commit_args
  -> run_commit
       -> open_context
       -> lock repo and branch
       -> compute_verify(persist=true)
       -> reject blocking diagnostics
       -> build_manifest
       -> store blob/manifest/atom_index/trace_graph/fire_delta/verification/policy
       -> commit_payload
       -> store commit object
       -> save_branch_record
       -> reset_active
  -> human summary or JSON result
```

When these flows change, update `docs/development/command-source-trace.md` in the same commit.

## 5. Local Workflow Data Flow

The local workflow is the core loop: scan, verify, extinguish, commit.

```text
open directory files
  |
  v
codefire_core::build_atom_index(open_dir)
  |
  +--> current AtomIndex
  |
  v
main.rs::compute_scan
  |
  +--> load base AtomIndex from sealed commit
  +--> codefire_core::current_trace_graph(open_dir)
  +--> codefire_core::build_scan_result(...)
  |
  +--> persist active scan.json / fires.json when requested
  |
  v
ScanResult
```

`verify` intentionally starts by computing scan state again.

```text
run_verify
  -> compute_scan(start, persist=true)
  -> parse_verification_policy(open_dir)
  -> run_verification_commands(open_dir, policy)
  -> codefire_core::build_verification(scan, resolutions, failed_checks, policy)
  -> persist_verification_result_state
  -> return Verification
```

`commit` consumes the current active state and writes sealed objects.

```text
run_commit
  -> open_context
  -> compute_verify
  -> reject blockers
  -> build_manifest(open_dir)
  -> store manifest / atom index / trace graph / fire delta / verification / policy
  -> codefire_store::commit_payload(...)
  -> codefire_store::store_object(type="commit", payload)
  -> save branch head
  -> reset active state
```

Important invariant:

- scan and verify may update active state;
- only commit updates a branch head;
- sealed objects are immutable once written.

## 6. State and Persistence Model

CodeFire has three persistence zones.

```text
.codefire/objects/
  immutable content-addressed object records

.codefire/branches/
  mutable branch head records

.codefire/active/<open_instance_id>/
  mutable open-directory state: scan, fires, resolutions, verification
```

And one open marker in the working directory:

```text
.codefire-open
  repo root
  branch
  open instance id
  current base commit
```

The common read path is:

```text
find_open_marker(start)
  -> read .codefire-open
  -> find_repo_root
  -> validate opened registry
  -> return OpenContext
```

The branch read path is:

```text
find_repo_root(start)
  -> .codefire/branches/<branch>.json
  -> head commit id
  -> codefire_store::validate_sealed_commit(objects, head)
```

When a command acts on an open directory, start by locating the `OpenContext`; when it acts on a repository, start by locating the repo root.

## 7. Core Domain Flow

`codefire-core/src/lib.rs` is a domain module with several internal zones.

```text
files/config
  -> Config and path patterns
  -> rel_files
  -> extract_markdown_atoms / extract_explicit_cf_atoms / language extractors
  -> AtomIndex

codefire.links.yaml
  -> parse_links
  -> TraceLinkInput
  -> TraceGraph

codefire.policy.yaml
  -> parse_trace_policy
  -> parse_verification_policy
  -> RequiredLinkRule / VerificationPolicy

AtomIndex + base AtomIndex + TraceGraph + resolutions
  -> changed_atoms
  -> required_link_missing
  -> build_scan_result
  -> Fire list

ScanResult + failed external checks + policy
  -> build_verification
  -> Verification
```

Core structs to recognize:

| Struct | Mental role |
|---|---|
| `Atom` | one requirement/design/code/test/API/schema unit |
| `AtomIndex` | all Atoms found in an open directory or sealed commit |
| `TraceLink` / `TraceGraph` | directed responsibility edges between Atoms |
| `RequiredLinkRule` / `TracePolicy` | which links should exist |
| `Fire` | unresolved responsibility created by changed/missing/stale relation |
| `Resolution` | claim that a fire is handled, with basis and evidence refs |
| `ScanResult` | changed Atoms and currently open fires |
| `Verification` | complete gate result for commit |

Core should not know about `--json`, terminal text, or command names.

Main public functions:

| Function | Reads | Produces | Used by |
|---|---|---|---|
| `build_atom_index(open_dir)` | source files plus `codefire.yaml` patterns | `AtomIndex` | scan, context, fire, verify, commit |
| `current_trace_graph(open_dir)` | `codefire.links.yaml` | `TraceGraph` | scan, context, diff, commit |
| `parse_trace_policy(open_dir)` | `codefire.policy.yaml` | `TracePolicy` | scan and missing-link checks |
| `parse_verification_policy(open_dir)` | `codefire.policy.yaml` | `VerificationPolicy` | verify and commit |
| `build_scan_result(...)` | current/base indexes, trace graph, policy, existing fires/resolutions | `ScanResult` | scan, verify, commit |
| `build_verification(...)` | scan, resolutions, failed checks, policy | `Verification` | verify and commit |
| `build_resolution(...)` | fire, atom index, trace graph, evidence refs | `Resolution` | extinguish |

Internal shape:

- extractor functions create `Atom` values with `id`, `kind`, `path`, `selector`, and content hash;
- `changed_atoms` compares current and base `AtomIndex` maps by Atom ID and hash;
- Trace Link IDs and fire UIDs are deterministic hashes of semantic inputs;
- verification is a pure summary over scan/fires/resolutions/policy plus external failed checks.

Core must not mutate `.codefire/` or print output. If a future core function needs to write active state, that responsibility belongs in CLI repository services instead.

## 7.1 Extractor Mental Model

Atom extraction follows this order:

```text
codefire.yaml / default patterns
  -> rel_files(open_dir)
  -> classify file kind by pattern
  -> format-specific extractor
  -> AtomIndex { atoms: Vec<Atom> }
```

Current extractor categories:

| Category | Source shape | Atom ID style | Typical consumer |
|---|---|---|---|
| markdown headings | heading line and following block | documented requirement/design IDs | requirement, design, docs trace |
| explicit `cf-atom` markers | marker metadata plus nearby content | explicit Atom ID | code and mixed files |
| code/path fallback | configured path patterns | path-derived IDs | implementation trace and broad impact |

Any extractor change must answer:

- how duplicate IDs are diagnosed;
- whether content hash reflects the intended semantic unit;
- whether non-Atom file changes remain visible in scan;
- which Trace Link policies should apply to the new Atom kind.

## 8. Store Domain Flow

`codefire-store/src/lib.rs` owns identity and validation.

```text
payload Value
  -> canonical_json(payload)
  -> sha256
  -> CF-<TYPE>-<digest>
  -> ObjectRecord { id, hash, type, payload }
  -> write .codefire/objects/<subdir>/<id>.json through temp file
```

Read path:

```text
object id
  -> object_record_path(objects_root, id)
  -> read JSON
  -> validate object id, hash, type, subdir
  -> return payload
```

Sealed commit validation:

```text
validate_sealed_commit(objects, commit_id)
  -> read commit object
  -> validate parents recursively
  -> validate each root object type
  -> validate references inside manifest/resolution/verification/evidence
```

If a new object type is introduced, update:

- `object_kind_by_type`
- object prefix/subdir mapping
- `known_object_subdirs`
- doctor layout checks
- storage grouping
- sealed validation if commits can reference it

Main public functions:

| Function | Responsibility |
|---|---|
| `object_record(type_tag, payload)` | create deterministic record with ID and hash |
| `store_object(objects_root, type_tag, payload)` | publish immutable object record |
| `read_object` / `read_object_record` | load and validate object files |
| `object_record_path` | resolve object ID to known subdir/path |
| `validate_sealed_commit` | prove a commit object graph is structurally valid |
| `commit_payload` | build the commit object payload from root object IDs and metadata |

Store should not know open directories, branch state names, CLI flags, or remote permissions. It only knows object records and references.

## 9. Automation JSON Model

Automation JSON is centralized around `automation::command_result_envelope`.

```json
{
  "type": "codefire.command_result.v1",
  "ok": true,
  "command": "scan",
  "exit_code": 0,
  "schema": "...",
  "data": {},
  "diagnostics": [],
  "next_actions": []
}
```

Command-specific payloads should be built by one of these functions or a command module equivalent:

- `status_data_json`
- `scan_data_json`
- `verification_data_json_with_filter`
- `fire_data_json`
- `evidence_add_data_json`
- `storage_report_data_json`
- `doctor_report_data_json`
- `migration_report_data_json`

The key separation:

- `data` is the stable machine payload;
- `diagnostics` is prioritized problems/warnings;
- `next_actions` is operational guidance for humans and tools;
- text output is a separate renderer.

## 10. Evidence Capture Flow

Single evidence capture now has a clear plan/apply split.

```text
parse_evidence_add_args
  -> EvidenceAddOptions
  -> run_evidence_add
       -> find_repo_root
       -> validate_evidence_add_options
       -> if dry_run:
            evidence_add_operation_plan
            return EvidenceAddResult { dry_run: true, plan: ... }
       -> store_artifact_ref if artifact path/URI exists
       -> capture_shell_command or capture_argv_command if requested
       -> store evidence object
       -> return EvidenceAddResult with id, object_path, summaries, diagnostics
```

Command capture stores bounded summaries:

- `command_stdout_summary`
- `command_stdout_truncated`
- `command_stderr_summary`
- `command_stderr_truncated`
- `command_exit_code`
- `command_timed_out`

Batch evidence reuses the single-result shape through `evidence/batch.rs`.

## 11. Metrics and Storage Flow

Metrics are attached after command execution.

```text
run_scan_command
  -> started = Instant::now()
  -> run_scan
  -> scan_metrics(started.elapsed(), scan)
  -> attach_metrics(scan_data_json(scan), metrics)
```

Unmeasured phases are represented as `null` in timing maps; future/unimplemented cache details are reported as unavailable rather than as normal operational telemetry.

Storage report flow:

```text
parse_storage_report_args
  -> StorageReportOptions
  -> run_storage_report
       -> find_repo_root
       -> scan .codefire/objects
       -> scan active/opened/branch/remotes/idempotency areas
       -> scan external artifact refs
       -> scan file remote project storage when configured
  -> storage_report_data_json or print_storage_report
```

The user-facing alias `codefire storage [path] [--json]` delegates to the same report path as `codefire storage report ...`.

## 12. Command Surface Table

| Command | Parser | Runner | JSON data builder | Human renderer | Persistence |
|---|---|---|---|---|---|
| `status` | `parse_path_json_args` | `read_status` | `status_data_json` | `print_status` | read-only |
| `scan` | `parse_path_json_args` | `run_scan` / `compute_scan` | `scan_data_json` | `print_scan` | active scan/fires |
| `verify` | `parse_verify_args` | `run_verify` / `compute_verify` | `verification_data_json_with_filter` | `print_verification` | active verification/state |
| `fire` | `parse_fire_args` | `run_fire` | `fire_data_json` | `print_fire_result` | active fires |
| `extinguish` | `parse_extinguish_args` | `run_extinguish` | plan/result helpers | inline/text helpers | active resolutions |
| `commit` | `parse_commit_args` | `run_commit` | plan/result helpers | inline/text helpers | objects, branch head, active reset |
| `evidence add` | `parse_evidence_add_args` | `run_evidence_add` | `evidence_add_data_json` | `print_evidence_add_result` | evidence/artifact objects |
| `storage` | `parse_storage_report_args` | `run_storage_report` | `storage_report_data_json` | `print_storage_report` | read-only |
| `doctor` | `parse_doctor_args` | `run_doctor` | `doctor_report_data_json` | `print_doctor_report` | read-only |
| `migrate` | `parse_migrate_args` | `run_migrate` | `migration_report_data_json` | `print_migration_report` | mostly read-only/planned |
| `diff` | `parse_diff_args` | `view::diff_commitish_with_options` | view diff payload | text diff renderer | read-only |
| `branch list` | `parse_path_json_args` | `list_branches` | `branch_json` in envelope | `print_branches` | read-only |

## 13. How To Add Or Change A Command

Use this checklist before editing.

1. Define the command contract.
   - Arguments, path behavior, `--json`, `--metrics`, `--dry-run`, lock/idempotency behavior.
   - Update `docs/04_cli_spec.md` and `docs/automation_interface.md` if the contract changes.

2. Find the correct owner.
   - CLI-only parse/render: `codefire-cli`.
   - Atom/Trace/Verification semantics: `codefire-core`.
   - Object identity or sealed validation: `codefire-store`.
   - Health/reporting: `doctor`, `migration`, `storage`, `explain`.

3. Implement parse/run/render as separate functions.
   - Parser returns an options struct.
   - Runner returns a result struct or domain value.
   - Renderer takes the result; it must not mutate state.

4. Add JSON envelope support.
   - Stable `data`.
   - Bounded `diagnostics`.
   - Useful `next_actions`.
   - Failure paths should remain machine-readable when `--json` was requested.

5. Add tests at the smallest useful layer.
   - Parser tests for option forms.
   - Runner tests for persistence and invariants.
   - JSON tests for schema and automation behavior.
   - Golden tests only when human output stability matters.

6. Update navigation docs.
   - `source-code-map.md` for new modules/types.
   - `command-source-trace.md` for command paths.
   - This document if a new layer or lifecycle appears.

## 14. Invariants To Keep In Mind

High-risk invariants:

- object IDs are derived from canonical payload, not filenames or display text;
- branch head changes only after verification passes;
- active state may be mutable, sealed objects may not;
- dry-run must not write objects, branches, active state, or remote project data;
- `--json` output must not mix human text before or after JSON;
- non-blocking diagnostics must not cause blocking exit codes;
- lock acquisition must happen before mutable writes and after enough validation to know the target;
- path handling should be consistent across commands that accept a repository/open-directory target;
- output truncation must report that truncation occurred.

## 15. Where Bugs Usually Come From

Common failure zones:

| Symptom | Likely source |
|---|---|
| command accepts one path form but not another | parser helpers in `main.rs` or command module |
| text says clean but JSON says burning | state derivation split between `main.rs` and `automation.rs` |
| object exists but cannot be read | `codefire-store` object ID/subdir/type validation |
| doctor and migrate disagree | duplicated layout definitions |
| automation cannot recover | missing JSON envelope, diagnostics, or next_actions |
| evidence is hard to audit | `evidence_add_data_json` missing summary/path fields |
| storage size surprises users | artifact refs and object grouping not surfaced by `storage.rs` |
| tests are slow or broad | behavior added only to `tests.rs` instead of module-local tests |

## 16. Reading Exercises

To understand the current implementation quickly, trace these paths in order.

1. `codefire scan . --json --metrics`
   - `run` -> `local_workflow_handler` -> `run_scan_command` -> `compute_scan` -> `automation.rs` -> `metrics.rs`.

2. `codefire verify . --details --blocking-only --json`
   - `run_verify_command` -> `parse_verify_args` -> `compute_verify` -> `verification_data_json_with_filter`.

3. `codefire evidence add --from-command '...' --dry-run --json`
   - `parse_evidence_add_args` -> `run_evidence_add` -> `evidence_add_operation_plan` -> `evidence_add_data_json`.

4. `codefire storage . --json`
   - `parse_storage_report_args` -> `run_storage_report_command` -> `run_storage_report` -> `storage_report_data_json`.

5. `codefire commit -m '...'`
   - `parse_commit_args` -> `run_commit` -> `build_manifest` -> `codefire_store::store_object` -> `save_branch_record`.

If these five traces are clear, most of the CodeFire codebase will feel navigable.
