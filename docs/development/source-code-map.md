# CodeFire Source Code Map

この文書は、CodeFireのRust実装を読むためのsource mapである。要求定義や内部設計だけでは抽象的すぎるため、実際のfile、主要型、主要関数、責務境界をここで対応付ける。

## Reading Order

初めて実装を読む場合は、次の順で読む。

1. `crates/codefire-cli/src/main.rs`
   - command dispatch、local workflow handler、parse/run/renderの接続を見る。
2. `crates/codefire-cli/src/cli_model.rs`
   - CLIで共有されるOptions/Result/OpenContext型を見る。
3. `crates/codefire-cli/src/automation.rs`
   - JSON envelope、diagnostics、next_actionsの形を見る。
4. `crates/codefire-core/src/lib.rs`
   - Atom、Trace Graph、ScanResult、Verificationのdomain modelを見る。
5. `crates/codefire-store/src/lib.rs`
   - canonical JSON、object store、sealed commit validationを見る。
6. command-specific module
   - evidence、doctor、storage、migration、view、remoteなど、触るcommandに近いmoduleを見る。
7. `crates/codefire-cli/src/tests.rs` and `crates/codefire-cli/src/tests/docs.rs`
   - 既存behaviorとregression coverageを見る。

## Workspace Crates

| Crate | Role | Reads | Writes |
|---|---|---|---|
| `codefire-core` | Atom extraction、Trace Graph、scan/verification domain | working files, config, policy | domain structs only |
| `codefire-store` | object identity、canonical JSON、object read/write、sealed validation | `.codefire/objects` | immutable object records |
| `codefire-util` | shared utility for hash/hex/time/path encoding | none | none |
| `codefire-cli` | commands, repository mutation, remote, rendering | repo/open/remote files | active state, branch heads, objects, remote project files |

Design rule:

- `codefire-core` and `codefire-store` must remain usable without CLI rendering.
- `codefire-cli` may orchestrate core/store, but should not duplicate core/store invariants.

## CLI Entrypoint Map

| File | What to look for | Current responsibility |
|---|---|---|
| `main.rs` | `run`, command `match`, `local_workflow_handler` | top-level dispatch, legacy parse helpers, local workflow orchestration |
| `cli_model.rs` | `Status`, `OpenContext`, `*Options`, `*Result` | shared command data shapes |
| `exit_code.rs` | `ExitCode`, error mapping helpers | stable process exit code taxonomy |
| `completion.rs` | `help_text`, `completion_script` | top-level help and shell completion |
| `automation.rs` | `command_result_envelope`, `*_data_json`, `*_next_actions` | machine-readable command result contract |

Current architecture debt:

- `main.rs` still owns many parse helpers and operation implementations (`run_scan`, `run_verify`, `run_extinguish`, `run_commit`). v0.8 should extract state/contract/workflow modules without changing behavior.

## Module Responsibility Map

### Core workflow

| File | Main public surface | Responsibility |
|---|---|---|
| `main.rs` | `run_scan`, `run_verify`, `run_extinguish`, `run_commit` | main local workflow mutation/read orchestration |
| `verification.rs` | `VerifyOptions`, `parse_verify_args`, `print_verification`, `render_verification` | verify CLI options and human rendering |
| `fire.rs` | `FireOptions`, `run_fire`, `run_manual_fire_specs`, `fire_data_json` | manual fire creation and rendering data |
| `fire/batch.rs` | `parse_fire_batch_args`, `run_fire_batch`, `fire_batch_data_json` | batch manual fire validation and execution |
| `batch.rs` | `parse_extinguish_batch_args`, `run_extinguish_batch` | batch extinguish validation/execution |
| `extinguish_ux.rs` | interactive/all-matching parsers and runners | higher-level extinguish UX and evidence candidates |
| `link_batch.rs` | `parse_link_batch_args`, `run_link_batch` | Trace Link batch mutation |

### Automation and diagnostics

| File | Main public surface | Responsibility |
|---|---|---|
| `automation.rs` | `command_result_envelope`, `status_data_json`, `scan_data_json`, `verification_*` | JSON contract and next action generation |
| `metrics.rs` | `status_metrics`, `scan_metrics`, `verification_metrics`, `attach_metrics` | command metrics payload and text rendering |
| `explain.rs` | `parse_explain_args`, `run_explain`, `print_explain_result` | read-only explanation for fire/atom/verify/storage findings |

### Evidence and storage

| File | Main public surface | Responsibility |
|---|---|---|
| `evidence.rs` | `EvidenceAddOptions`, `run_evidence_add`, `evidence_add_data_json` | single evidence capture from artifact, URI, shell command, argv command |
| `evidence/batch.rs` | `run_evidence_batch`, `evidence_batch_data_json` | batch evidence validation and execution |
| `storage.rs` | `StorageReportOptions`, `run_storage_report`, `storage_report_data_json` | local/remote capacity reporting, object type grouping, artifact refs |

### Health and migration

| File | Main public surface | Responsibility |
|---|---|---|
| `doctor.rs` | `DoctorOptions`, `run_doctor`, `DoctorReport`, `DoctorIssue` | health check entry and report model |
| `doctor/checks.rs` | `run_checks` | repository layout, object, branch, opened registry, active state checks |
| `doctor/report.rs` | `doctor_report_*`, `print_doctor_report` | doctor JSON/text rendering |
| `migration.rs` | `MigrateOptions`, `run_migrate`, `MigrationReport` | migration check/dry-run and planned actions |

### View, diff, patch, merge

| File | Main public surface | Responsibility |
|---|---|---|
| `view.rs` | `show_commitish`, `diff_commitish_with_options`, `review_pack_with_options`, `patch_export_with_options` | commitish resolution, manifest view, diff/review/patch orchestration |
| `view/file_diff.rs` | file diff internals | bounded text/binary diff, rename/copy detection, patience/histogram anchors |
| `view/semantic_diff.rs` | semantic diff internals | Atom diff, Trace Graph diff, verification/policy impact |
| `merge_patch_idempotency.rs` | merge/patch replay helpers | idempotent result serialization/deserialization |

### Remote, HTTP, security

| File | Main public surface | Responsibility |
|---|---|---|
| `remote.rs` | `upload_branch`, `list_remote_branches`, `request_merge`, `review_merge_request`, `apply_merge_request` | file-backed remote project mutation and validation |
| `remote/idempotency.rs` | idempotency helpers | remote mutator idempotency records |
| `remote/plans.rs` | plan builders | remote operation dry-run plans |
| `http.rs` | `http_json`, `serve_http`, URL parsers | HTTP/HTTPS client/server routing and request parsing |
| `http_tls.rs` | TLS client/server helpers | rustls setup, cert/key loading, insecure mode guard |
| `signatures.rs` | signing/verification helpers | commit signatures, remote request signatures, nonce replay |

### Shared parsing and helpers

| File | Main public surface | Responsibility |
|---|---|---|
| `limited_yaml.rs` | limited YAML parser utilities | shared scalar/comment/key-value parsing for batch/config-like files |
| `idempotency.rs` | local idempotency helpers | local mutator idempotency payload/record verification |
| `open_clone_idempotency.rs` | open/clone replay helpers | idempotent open/clone results |

## Core Domain Map

`crates/codefire-core/src/lib.rs` is currently dense. Conceptually it owns:

| Concept | Responsibility |
|---|---|
| config parsing | `codefire.yaml`, links, policy limited parsing |
| Atom extraction | Markdown, explicit `cf-atom`, code/API/DB extractors |
| Atom identity | Atom ID validation, content hash calculation |
| Trace Graph | links, required link policy, link IDs |
| Scan | current/base Atom comparison, fire generation, stale/obsolete concepts |
| Verification | missing links, stale resolutions, duplicate Atom IDs, failed checks model |

When editing core:

- avoid adding CLI-specific error text;
- add tests near core behavior;
- preserve Atom ID and hash compatibility unless a migration is documented;
- use maps/indexes for repeated graph checks.

## Store Domain Map

`crates/codefire-store/src/lib.rs` owns:

| Concept | Responsibility |
|---|---|
| canonical JSON | deterministic serialization for object identity |
| object digest | hash calculation and object ID formatting |
| object write/read | immutable object record storage and validation |
| known object kinds | prefix/subdir/type mapping |
| sealed commit validation | commit/root/parent/certificate/object graph checks |
| legacy compatibility | reading validated legacy object IDs |

When editing store:

- never use display strings as identity input unless the schema says so;
- validate before publishing final object paths;
- add golden or tamper tests for identity changes;
- update doctor/storage visibility for new object kinds.

## Data Shape Map

| Data shape | Type/file | Persistent location |
|---|---|---|
| open context | `OpenContext` in `cli_model.rs` | `.codefire-open`, `.codefire/opened/*` |
| status | `Status` in `cli_model.rs` | branch/open registry plus active state |
| scan result | `codefire_core::ScanResult` | `.codefire/active/<open>/scan.json`, `fires.json` |
| verification | `codefire_core::Verification` | `.codefire/active/<open>/verification.json`, commit object roots |
| manual fire result | `FireResult` in `fire.rs` | active `fires.json` |
| extinguish result | `ExtinguishResult` in `cli_model.rs` | active `resolutions.json` |
| evidence result | `EvidenceAddResult` in `evidence.rs` | `.codefire/objects/evidence`, optional `artifact_refs` |
| doctor report | `DoctorReport` in `doctor.rs` | read-only |
| migration report | `MigrationReport` in `migration.rs` | read-only unless apply mode is added/used |
| storage report | `StorageReport` in `storage.rs` | read-only |
| remote branch | `RemoteBranch` in `remote.rs` | remote project `branches/` |
| merge request | request structs in `remote.rs` | remote project `merge_requests/` |

## Filesystem Layout Map

| Path | Owner | Meaning |
|---|---|---|
| `.codefire/repo.json` or repo metadata | init/doctor/migrate | repository identity and format |
| `.codefire/objects/` | store/commit/evidence/remote | immutable object records |
| `.codefire/branches/` | branch/open/commit/remote | local branch heads |
| `.codefire/opened/` | open/close/status | open branch registry |
| `.codefire/active/<open>/` | scan/fire/extinguish/verify/commit | mutable open-directory metadata |
| `.codefire/idempotency/` | local mutators | idempotency records |
| `.codefire/locks/` | local mutators | local resource locks |
| remote `objects/branches/merge_requests` | remote/http | file-backed or served remote project state |

## Test Map

| Test file | Current role |
|---|---|
| `crates/codefire-cli/src/tests.rs` | broad CLI integration-style and regression tests |
| `crates/codefire-cli/src/tests/docs.rs` | docs/help/completion consistency |
| feature-local `mod tests` | module-specific algorithms such as diff, HTTP, signatures, limited YAML |
| `crates/codefire-core/src/lib.rs` tests | Atom extraction, policy, trace graph, scan/verification domain |
| `crates/codefire-store/src/lib.rs` tests | canonical JSON, object identity, sealed validation |
| `tests/test_codefire_cli.py` | Python reference and installer/fallback compatibility |

For new v0.8 work, prefer domain-specific test modules instead of further expanding `tests.rs`.

## Change Navigation Cheatsheet

| Change target | Start here | Also inspect |
|---|---|---|
| JSON envelope or next_actions | `automation.rs` | `docs/automation_interface.md`, CLI tests |
| status/scan/verify state labels | `main.rs`, `automation.rs` | planned `state.rs`, `docs/05_consistency_model.md` |
| scan fire generation | `codefire-core/src/lib.rs` | `main.rs::run_scan`, `automation.rs::scan_*` |
| verification blocker behavior | `codefire-core/src/lib.rs`, `verification.rs` | `exit_code.rs`, `automation.rs` |
| extinguish behavior | `main.rs::run_extinguish`, `batch.rs`, `extinguish_ux.rs` | `codefire-core` stale resolution model |
| evidence capture | `evidence.rs` | `evidence/batch.rs`, `storage.rs`, `remote.rs` |
| doctor/migrate consistency | `doctor/*`, `migration.rs` | `storage.rs`, repo layout docs |
| storage report | `storage.rs` | `evidence.rs`, `remote.rs` |
| diff output | `view.rs`, `view/file_diff.rs`, `view/semantic_diff.rs` | golden fixtures |
| remote upload/MR | `remote.rs`, `http.rs` | `signatures.rs`, `remote/idempotency.rs` |
| install behavior | `install.sh` | `README.md`, Python tests |

## Known Refactor Targets

These are structural observations, not immediate implementation instructions:

- Extract state derivation from `main.rs`/`automation.rs` into `state.rs`.
- Extract scan/verify/commit command orchestration out of `main.rs`.
- Centralize path option parsing and subcommand help metadata.
- Share layout definitions between `init`, `doctor`, `migration`, and `storage`.
- Split `tests.rs` into domain modules as touched.
- Make JSON failure paths use a common renderer.
