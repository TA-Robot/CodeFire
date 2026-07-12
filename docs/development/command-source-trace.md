# CodeFire Command Source Trace

Status: canonical

Owner: CLI Maintainer

Update trigger: `COMMAND_CAPABILITIES`, dispatch, parser, plan/apply boundary, JSON rendering, persistent writes, or command tests change.

This is a symbol-level routing map. It intentionally omits fixed line numbers. The source-doc checker requires one marker for every registered top-level command and detects registry drift.

## Execution Contract

Every command follows as much of this path as its mode needs:

```text
argv
  -> help/capability routing before mutation
  -> typed parse
  -> repository/open/remote context resolution
  -> validation and lock/idempotency checks
  -> operation plan
  -> apply (skipped for read-only or dry-run)
  -> domain/result value
  -> text renderer or codefire.command_result.v1 envelope
```

`main.rs::run` is the top-level router. `local_workflow_handler` handles the compact local command family; the remaining arms call focused modules or still-local orchestration functions. `command_registry::COMMAND_CAPABILITIES` is the declared top-level surface, not a complete parser registry.

## Registered Commands

| Command | Capability | Parse / execute / render ownership | Persistent effect and principal evidence |
|---|---|---|---|
| `init` | text | `main` init parser/initializer, store/core initial roots | creates `.codefire`, initial sealed commit and main branch; init/layout/store tests. <!-- command: init --> |
| `import` | JSON | `import_workflow::{parse_import_args, import_existing_project, import_result_data_json}` | creates a native repo/open branch from existing files; its current result-only idempotency marker does not reject a different request under the same key (CFB-126); dry-run, path, idempotency tests. <!-- command: import --> |
| `open` | JSON | `main::{parse_open_args, open_branch_from}` | validates sealed closed head, materializes files, writes marker/opened/active/branch state; open/idempotency/corruption tests. <!-- command: open --> |
| `branch` | JSON, metrics for list | `main` branch list/show routing and branch record helpers | read-only list/show of encoded branch records; there is no branch-create subcommand. Sealed-head and branch-show envelope tests cover the surface. <!-- command: branch --> |
| `status` | JSON + metrics | `main::read_status`, `automation::status_*`, `metrics::status_metrics` | read-only derived state and scan prediction; state/JSON/next-action tests. <!-- command: status --> |
| `scan` | JSON + metrics | `main::{parse_path_json_args, run_scan_command, run_scan}`, core scan builders, automation/metrics | writes active scan/fire/state on live scan; direct production routing currently lacks `RepoLock` (CFB-124); read-only preview remains separate; freshness/non-Atom/bounded JSON tests. <!-- command: scan --> |
| `fire` | JSON | `fire` module plus `main::run_fire_command` | validates all manual/batch Fires before active ledger writes; dry-run/batch/identity tests. <!-- command: fire --> |
| `extinguish` | JSON | `main::{parse_extinguish_args, run_extinguish}`, `batch`, `extinguish_ux` | updates Fire status and appends resolution basis/evidence refs under lock/idempotency; refresh/batch/dry-run tests. <!-- command: extinguish --> |
| `verify` | JSON + metrics | `verification::{parse_verify_args, render_verification}`, `main::{run_verify_command, compute_verify}`, core verification, automation/metrics | writes active scan/verification/state and executes constrained commands; direct production routing currently lacks `RepoLock` (CFB-124); `run_verify` is a test-only helper. Blocker/filter/exit tests cover production routing. <!-- command: verify --> |
| `commit` | JSON | `main::{parse_commit_args, run_commit, commit_operation_plan}`, store/signatures/automation | pending transaction, root objects, commit/certificate, branch publish, opened/active reset, idempotency; failpoint/sealed/certificate tests. <!-- command: commit --> |
| `clone` | JSON | `main::{parse_clone_args, clone_branch}`, open/clone idempotency | validates the source sealed head and creates a new closed branch record; it does not create an opened directory/state. Dry-run/idempotency tests cover it. <!-- command: clone --> |
| `close` | JSON | `main::{parse_close_args, close_branch}` | removes active/opened state and closes the branch; normally removes the open directory, while `--keep-dir` preserves the directory and removes its marker. Dirty-work rejection/dry-run tests cover it. <!-- command: close --> |
| `discard` | JSON | same close path with explicit discard flag | intentionally removes open work after validation/plan; discard safety tests. <!-- command: discard --> |
| `merge` | JSON | `main::{parse_merge_args, merge_branch}`, `view` merge/diff, merge idempotency | plans/applies target open-directory changes and state; conflict/binary/semantic/dry-run tests. <!-- command: merge --> |
| `upload` | JSON | `main::parse_upload_args`, `remote`, `http`, `signatures` | transfers validated object graph and conditionally updates remote branch under resource lock/idempotency; file/HTTP/auth tests. <!-- command: upload --> |
| `list` | JSON | `main::{parse_list_args, list_branches/list_remote_branches}` | read-only local opened/branch or remote branch view; bounded/sealed/JSON tests. <!-- command: list --> |
| `request-merge` | JSON | `main::parse_request_merge_args`, `remote`/`http`/`signatures` | creates a merge request after resolving and validating sealed source/target heads; request signing is a separate transport/auth concern. Dry-run/idempotency tests cover it. <!-- command: request-merge --> |
| `request-list` | JSON | `main::parse_remote_project_args`, remote list | read-only merge-request list; file/HTTP envelope tests. <!-- command: request-list --> |
| `request-review` | JSON | `main::parse_request_review_args`, remote review/signature path | appends review decision under lock/idempotency; signed request and dry-run tests. <!-- command: request-review --> |
| `request-apply` | JSON | `main::parse_request_apply_args`, remote apply/signature path | validates approved/current heads and publishes target remote head; replay/fast-forward/dry-run tests. <!-- command: request-apply --> |
| `show` | JSON | `main::parse_show_args`, `view::{show_commitish, show_commitish_data}` | read-only commit/object/branch resolution; text/envelope/error tests. <!-- command: show --> |
| `diff` | JSON | `main::parse_diff_args`, `view::diff_commitish_with_options` and algorithm modules | read-only bounded file/Atom/Trace/impact diff; golden/Git comparison/large-file tests. <!-- command: diff --> |
| `review-pack` | JSON metadata | `main::parse_review_pack_args`, `view::review_pack_with_options` | writes explicit output artifact when requested; bounded review-pack metadata/content tests. <!-- command: review-pack --> |
| `patch` | JSON | `main` export/import parsers, `view` export, patch import/idempotency | export writes artifact; import validates then mutates open directory under lock/idempotency; round-trip/dry-run/bounds tests. <!-- command: patch --> |
| `doctor` | JSON | `doctor::{parse_doctor_args, run_doctor, report renderers}` plus store/layout validators | read-only diagnosis by default; reports pending transaction/layout/object/open corruption; doctor fixtures. <!-- command: doctor --> |
| `storage` | JSON | `storage::{parse_storage_report_args, run_storage_report, *_json}` | read-only bounded local/remote inventory and artifact warnings; quick/full/limit tests. <!-- command: storage --> |
| `evidence` | JSON | `evidence::{parse_evidence_add_args, run_evidence_add/run_evidence_batch}` | validates command/artifact/batch then stores artifact_ref/evidence objects; failure/timeout/redaction-boundary tests. <!-- command: evidence --> |
| `explain` | JSON | `explain::{parse_explain_args, run_explain}` | read-only Fire/Atom/verify/storage explanation and next action; target/error tests. <!-- command: explain --> |
| `context` | JSON | `context::{parse_context_args, build_context_pack}` | read-only bounded selection over active/repository objects; truncation/freshness/path tests. <!-- command: context --> |
| `migrate` | JSON | `migration` parser/check/dry-run and `repo_layout` | currently inspects/plans compatibility/layout action; version/limit/doctor agreement tests. <!-- command: migrate --> |
| `serve` | text process | `main::parse_serve_args`, `http::serve_http`, `http_tls` | serves remote state and mutating routes under configured auth/signature/resource policy; route/TLS/body/timeout tests. <!-- command: serve --> |
| `completion` | text | `completion::{completion_script, bash_completion_script, zsh_completion_script}` | read-only generated shell script; registry/completion consistency tests. <!-- command: completion --> |
| `capabilities` | JSON | `command_registry::capabilities_data_json`, automation envelope | read-only declared command feature surface; registry/help/completion/metrics consistency tests. <!-- command: capabilities --> |
| `version` | text | `main` version arm | reports internal `codefire foundation 1`; release policy must not treat it as SemVer. <!-- command: version --> |

## Internal Debug Surface

`atom-index`, `trace-graph`, and `missing-links` are read-only debug commands handled by `parse_read_only_debug_args`. They return structured envelopes with `--json` but are intentionally not entries in `COMMAND_CAPABILITIES`; changing that classification requires a registry/CLI-contract decision.

## Cross-Cutting Paths

### JSON

- `automation::command_result_envelope` owns the top-level shape.
- operation-plan commands put their plan under `data.plan` with a result type.
- payload-specific `type` stays in `data`; metrics are attached under `data.metrics`.
- `print_cli_error_json` maps `CliError` to the same envelope and stable exit class.

### Lock and idempotency

- mutable repository metadata/open-state mutations normally acquire `RepoLock` with optional wait/timeout diagnostics; direct live scan/verify are the documented CFB-124 exception to fix, while immutable content-addressed writes such as `run_evidence_add` can publish through the object store without that repository-wide lock;
- remote mutations acquire project/branch/request resource locks;
- idempotency records are required to bind operation, scope/key, and payload hash so replay returns the recorded result and mismatch is a conflict; import is the current CFB-126 exception because its marker stores key/result without the request hash.

### Paths

Repository discovery, open-marker validation, explicit `--path`, remote URLs, output paths, and symlink policy are separate inputs. Preserve the originating repo/open path in diagnostics and `next_actions`.

### Docs and test synchronization

When a command changes, update the registry, help/completion, CLI spec, this table, JSON/text tests, and the focused validation map together. `check_source_docs.py` detects command set drift, but semantic routing still needs review.
