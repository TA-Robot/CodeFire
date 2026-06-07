# Command Source Trace

この文書は、主要CLI commandがどのsource file、主要関数、domain model、persistent stateを通るかを示す。ソースコードを読む前に、commandの実装経路を頭に描くための索引である。

## Trace Notation

```text
CLI command
  -> parse function
  -> run function
  -> domain/core/store calls
  -> render function
  -> persistent state
```

関数名は現在のRust実装に合わせる。v0.8で関数が移動した場合、この文書も同じcommitで更新する。

## init

```text
codefire init [path]
  -> main.rs::parse_init_args
  -> main.rs command match arm
  -> repository/object initialization helpers in main.rs and store
  -> optional JSON plan/result rendering in main.rs
  -> writes .codefire/, initial objects, branches/main
```

Primary files:

- `crates/codefire-cli/src/main.rs`
- `crates/codefire-cli/src/cli_model.rs`
- `crates/codefire-store/src/lib.rs`

Design risk:

- layout definitions must not diverge from doctor/migrate/storage.

## open

```text
codefire open <branch> <path>
  -> main.rs::parse_open_args
  -> main.rs open match arm
  -> open materialization and marker/registry write helpers
  -> optional plan/result JSON
  -> writes .codefire-open, .codefire/opened/*, .codefire/active/*
```

Primary files:

- `main.rs`
- `cli_model.rs` (`OpenOptions`, `OpenResult`, `OpenContext`)
- `open_clone_idempotency.rs`
- `idempotency.rs`

Design risk:

- open must reject copied directories and same-branch multiple open.

## branch list

```text
codefire branch list [--json] [--path <repo>]
  -> main.rs branch/list match arm
  -> main.rs::parse_path_json_args(command="branch list")
  -> branch loading and sealed head validation
  -> if --json:
       automation.rs::command_result_envelope(data={type: codefire_branch_list, branches: [...]})
     else:
       text rendering in main.rs::print_branches
  -> reads .codefire/branches/*
```

Primary files:

- `main.rs`
- `codefire-store/src/lib.rs`

Remaining v0.8 gap:

- `branch show --json` is still not implemented.
- unsupported branch subcommands with `--json` still need a structured failure envelope.

## status

```text
codefire status [path|--path <path>] [--json] [--metrics]
  -> main.rs::parse_path_json_args(command="status")
  -> status loading in main.rs
  -> automation.rs::status_data_json
  -> automation.rs::status_next_actions
  -> metrics.rs::status_metrics
  -> reads .codefire-open, opened registry, active state, branch head
```

Primary files:

- `main.rs`
- `cli_model.rs` (`Status`, `PathJsonOptions`)
- `automation.rs`
- `metrics.rs`

Current behavior:

- `--path <path>` and `--path=<path>` are accepted by the shared path/json parser for commands that use `parse_path_json_args`.

Remaining v0.8 gap:

- status state must be derived by a shared reducer.

## scan

```text
codefire scan [path|--path <path>] [--json] [--metrics]
  -> main.rs::local_workflow_handler("scan")
  -> main.rs::run_scan_command
  -> main.rs::parse_path_json_args(command="scan")
  -> main.rs::run_scan
  -> codefire_core scan functions
  -> automation.rs::scan_data_json
  -> automation.rs::scan_diagnostics_json
  -> automation.rs::scan_next_actions
  -> metrics.rs::scan_metrics
  -> writes active scan.json and fires.json
```

Primary files:

- `main.rs`
- `codefire-core/src/lib.rs`
- `automation.rs`
- `metrics.rs`

Persistent state:

- `.codefire/active/<open>/scan.json`
- `.codefire/active/<open>/fires.json`
- active state marker/status files

v0.8 gap:

- non-Atom changed files need first-class output.
- batch extinguish next_actions should be generated when multiple fires exist.

## context

```text
codefire context --changed|--atom|--fire|--branch --json
  -> context.rs::parse_context_args
  -> context.rs::build_context_pack
  -> context.rs::print_context_summary or automation envelope in main.rs
  -> reads active scan/fires, Atom index, Trace Graph
```

Primary files:

- `context.rs`
- `automation.rs`
- `main.rs`

v0.8 gap:

- `context --changed --json` top-level data should mirror scan summary fields.

## verify

```text
codefire verify [path] [--details] [--blocking-only] [--json] [--metrics]
  -> main.rs::local_workflow_handler("verify")
  -> main.rs::run_verify_command
  -> verification.rs::parse_verify_args
  -> main.rs::run_verify
  -> codefire_core verification model
  -> main.rs::run_verification_commands for external checks
  -> verification.rs::print_verification or automation JSON
  -> metrics.rs::verification_metrics
  -> writes active verification.json and may update active state
```

Primary files:

- `main.rs`
- `verification.rs`
- `codefire-core/src/lib.rs`
- `automation.rs`
- `exit_code.rs`
- `metrics.rs`

Persistent state:

- `.codefire/active/<open>/verification.json`
- active state file.

v0.8 gap:

- blocking-only data and diagnostics must agree.
- verify must not degrade clean state.

## fire

```text
codefire fire <source> --to <target> ...
  -> main.rs::local_workflow_handler("fire")
  -> main.rs::run_fire_command
  -> fire.rs::parse_fire_args
  -> fire.rs::run_fire
  -> fire.rs::run_manual_fire_specs
  -> fire.rs::fire_data_json or print_fire_result
  -> writes active fires.json
```

Batch:

```text
codefire fire --batch <file>
  -> fire/batch.rs::parse_fire_batch_args
  -> fire/batch.rs::run_fire_batch
  -> fire.rs::run_manual_fire_specs
```

Primary files:

- `fire.rs`
- `fire/batch.rs`
- `limited_yaml.rs`
- `main.rs`

Design invariant:

- all batch items validate before any write.

## extinguish

```text
codefire extinguish <fire-id> --resolution <type> ...
  -> main.rs::local_workflow_handler("extinguish")
  -> main.rs::run_extinguish_command
  -> main.rs::parse_extinguish_args
  -> extinguish_ux.rs::prepare_extinguish_options
  -> main.rs::run_extinguish
  -> writes active resolutions.json and updates fire status
```

Batch:

```text
codefire extinguish --batch <file>
  -> batch.rs::parse_extinguish_batch_args
  -> batch.rs::run_extinguish_batch
  -> main.rs::run_extinguish per validated item
```

Interactive/all-matching:

```text
codefire extinguish --interactive
  -> extinguish_ux.rs::parse_interactive_extinguish_args
  -> extinguish_ux.rs::run_interactive_extinguish

codefire extinguish --all-matching ...
  -> extinguish_ux.rs::parse_all_matching_extinguish_args
  -> extinguish_ux.rs::run_all_matching_extinguish
```

Primary files:

- `main.rs`
- `batch.rs`
- `extinguish_ux.rs`
- `cli_model.rs`

v0.8 gap:

- success output should include atom/link/evidence summary.
- batch template generation should avoid manual YAML construction.

## evidence add

```text
codefire evidence add ...
  -> main.rs evidence/add match arm
  -> evidence.rs::parse_evidence_add_args
  -> evidence.rs::run_evidence_add
       -> dry-run returns evidence_add_operation_plan without writes
       -> apply stores optional artifact_ref and evidence object
  -> evidence.rs::evidence_add_data_json
  -> evidence.rs::print_evidence_add_result
  -> writes evidence object and optional artifact_ref object unless dry-run
```

Batch:

```text
codefire evidence add --batch <file>
  -> evidence.rs::parse_evidence_add_args
  -> evidence/batch.rs::run_evidence_batch
  -> evidence/batch.rs::evidence_batch_data_json
```

Primary files:

- `evidence.rs`
- `evidence/batch.rs`
- `storage.rs`
- `remote.rs` for artifact/evidence graph copy.

Current behavior:

- single-item `--dry-run` returns a validated operation plan and does not write an evidence object.
- JSON result includes evidence object path and bounded stdout/stderr summaries with truncation flags.

Remaining v0.8 gap:

- failed command capture should require explicit allowance.

## commit

```text
codefire commit -m <message>
  -> main.rs::local_workflow_handler("commit")
  -> main.rs::run_commit_command
  -> main.rs::parse_commit_args
  -> main.rs::run_commit
  -> run scan/verify or use fresh active state
  -> codefire-store object writes
  -> branch head atomic update
  -> active state reset
  -> CommitResult rendering
```

Primary files:

- `main.rs`
- `cli_model.rs` (`CommitOptions`, `CommitResult`)
- `codefire-store/src/lib.rs`
- `idempotency.rs`
- `signatures.rs`

Persistent state:

- `.codefire/objects/*`
- `.codefire/branches/<branch>.json`
- active state reset under `.codefire/active/<open>/`

v0.8 gap:

- blocked `commit --dry-run --json` must return JSON envelope.
- success output should summarize changed atoms, resolved fires, evidence, non-Atom files.

## show / diff / review-pack / patch

```text
codefire show <branch-or-url>
  -> view.rs::show_commitish

codefire diff [options] <left> <right>
  -> main.rs::parse_diff_args
  -> view.rs::diff_commitish_with_options
  -> view/file_diff.rs
  -> view/semantic_diff.rs

codefire review-pack ...
  -> main.rs::parse_review_pack_args
  -> view.rs::review_pack_with_options

codefire patch export ...
  -> main.rs::parse_patch_export_args
  -> view.rs::patch_export_with_options

codefire patch import ...
  -> main.rs::parse_patch_import_args
  -> patch import implementation in main.rs
  -> merge_patch_idempotency.rs
```

Primary files:

- `view.rs`
- `view/file_diff.rs`
- `view/semantic_diff.rs`
- `merge_patch_idempotency.rs`
- golden fixtures under `crates/codefire-cli/tests/fixtures/`

v0.8 gap:

- `show --json` and `diff --json` should return command result envelope.
- diff options must match emitted sections.

## doctor / migrate / storage / explain

```text
codefire doctor [path] [--quick|--full] [--json]
  -> doctor.rs::parse_doctor_args
  -> doctor.rs::run_doctor
  -> doctor/checks.rs::run_checks
  -> doctor/report.rs render helpers

codefire migrate check|dry-run [path] [--json]
  -> migration.rs::parse_migrate_args
  -> migration.rs::run_migrate
  -> migration report render helpers

codefire storage [path] [--json]
codefire storage report [path] [--json]
codefire storage --path <path> --json
  -> main.rs::run_storage_report_command
  -> storage.rs::parse_storage_report_args
  -> storage.rs::run_storage_report
  -> storage report render helpers

codefire explain fire|atom|verify-failure|storage-warning ...
  -> explain.rs::parse_explain_args
  -> explain.rs::run_explain
  -> explain.rs::print_explain_result
```

Primary files:

- `doctor.rs`, `doctor/checks.rs`, `doctor/report.rs`
- `migration.rs`
- `storage.rs`
- `explain.rs`

Current behavior:

- `storage`, `storage [path]`, `storage --json`, and `storage report ...` all route to the same storage report runner.

Remaining v0.8 gap:

- doctor/migrate must share layout definitions and repair vocabulary.

## remote and HTTP

```text
codefire upload <branch> <remote-url>
  -> main.rs::parse_upload_args
  -> remote.rs::upload_branch
  -> remote/plans.rs
  -> remote/idempotency.rs
  -> signatures.rs

codefire list <remote-project-url>
  -> main.rs::parse_remote_project_args
  -> remote.rs::list_remote_branches

codefire request-merge ...
  -> main.rs::parse_request_merge_args
  -> remote.rs::request_merge

codefire request-list ...
  -> main.rs::parse_remote_project_args
  -> remote.rs::list_merge_requests

codefire request-review ...
  -> main.rs::parse_request_review_args
  -> remote.rs::review_merge_request

codefire request-apply ...
  -> main.rs::parse_request_apply_args
  -> remote.rs::apply_merge_request
```

HTTP path:

```text
http.rs::http_json
http.rs::serve_http
http.rs::dispatch_http
http.rs::handle_http_upload
http.rs::handle_http_request_merge
http.rs::handle_http_request_list
http.rs::handle_http_request_review
http.rs::handle_http_request_apply
```

Primary files:

- `remote.rs`
- `remote/plans.rs`
- `remote/idempotency.rs`
- `http.rs`
- `http_tls.rs`
- `signatures.rs`

Security-sensitive code:

- HTTP path normalization.
- request body size/read timeout.
- TLS key/cert parsing.
- client CA configuration.
- request signature verification.
- nonce replay checks.

## serve

```text
codefire serve <storage-root> ...
  -> main.rs::parse_serve_args
  -> http.rs::serve_http
  -> http_tls.rs when TLS is enabled
  -> http.rs request parser and dispatch handlers
```

Primary files:

- `http.rs`
- `http_tls.rs`
- `signatures.rs`
- `remote.rs`

## completion and version/help

```text
codefire completion bash|zsh
  -> completion.rs::completion_script

codefire --help / help
  -> completion.rs::help_text

codefire --version / version
  -> main.rs version arm
```

Primary files:

- `completion.rs`
- `main.rs`
- `docs/04_cli_spec.md`
- `crates/codefire-cli/src/tests/docs.rs`

## Cross-Cutting Flows

### JSON rendering flow

```text
command run result
  -> command-specific data_json helper
  -> diagnostics helper
  -> next_actions helper
  -> automation.rs::command_result_envelope
  -> serde_json::to_string_pretty
```

If a command prints raw JSON directly, it is a candidate for v0.8 contract cleanup.

### Lock and idempotency flow

```text
parse --wait-lock / --lock-timeout / --idempotency-key
  -> build payload hash
  -> check replay or conflict
  -> acquire lock
  -> revalidate
  -> write
  -> save idempotency record
```

Primary files:

- `idempotency.rs`
- `open_clone_idempotency.rs`
- `merge_patch_idempotency.rs`
- `remote/idempotency.rs`

### Remote request signing flow

```text
client request payload
  -> signatures.rs remote request signature
  -> remote/http transport
  -> server verifies key, timestamp skew, nonce replay
  -> handler applies mutation
```

Primary files:

- `signatures.rs`
- `remote.rs`
- `http.rs`

## How To Add A New Command

Implementation checklist:

1. Add option/result structs to `cli_model.rs` or a command module.
2. Add parse function.
3. Add run function that returns a typed result.
4. Add JSON data helper and diagnostics/next_actions if `--json` is supported.
5. Route from `main.rs` with minimal orchestration.
6. Add help/completion entry.
7. Add CLI spec and automation docs.
8. Add tests for text behavior, JSON envelope, errors, and docs/help consistency.

Avoid:

- embedding JSON strings by hand.
- writing files before full validation.
- adding domain logic directly inside a large `match` arm.
- adding command help without parser support.
