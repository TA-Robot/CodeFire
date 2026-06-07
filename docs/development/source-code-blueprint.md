# CodeFire Source Code Blueprint

この文書は、CodeFireの要求定義、内部設計、Rust実装、テスト、運用ドキュメントを同じ頭で読めるようにするための実装blueprintである。

`source-code-map.md` はfile索引、`source-code-mental-model.md` は処理の流れ、`command-source-trace.md` はcommand別traceである。この文書はそれらを統合し、「この機能はどの層に存在し、どのfileをどう変えるべきか」を示す。

## 1. Product Requirement To Source

| Requirement area | Product docs | Primary source | Supporting source | Tests |
|---|---|---|---|---|
| repository creation | `docs/01_product_definition.md` FR-001, `docs/07_local_repository_model.md` | `crates/codefire-cli/src/main.rs` init path | `codefire-store/src/lib.rs`, `doctor/checks.rs`, `migration.rs` | `crates/codefire-cli/src/tests.rs` init/layout tests |
| branch open/close | FR-002, FR-003, consistency model | `main.rs` open/close helpers | `cli_model.rs`, `open_clone_idempotency.rs`, `idempotency.rs` | open registry, copied directory, close safety tests |
| scan and impact detection | FR-005, `docs/09_index_trace_policy.md` | `codefire-core/src/lib.rs` | `main.rs::compute_scan`, `automation.rs`, `metrics.rs` | core Atom/Trace tests, CLI scan tests |
| manual and generated fire | FR-006, `docs/06_fire_extinguish.md` | `fire.rs`, `fire/batch.rs`, `codefire-core/src/lib.rs` | `main.rs`, `limited_yaml.rs` | manual fire and batch validation tests |
| resolution and extinguish | FR-007 | `main.rs::run_extinguish`, `batch.rs`, `extinguish_ux.rs` | `cli_model.rs`, `automation.rs` | stale resolution, batch all-or-nothing tests |
| verification and certificate | FR-008, verification requirements | `main.rs::compute_verify`, `codefire-core/src/lib.rs` | `verification.rs`, `exit_code.rs`, `automation.rs` | verify diagnostics and policy tests |
| sealed commit | FR-009, `docs/08_object_store.md` | `main.rs::run_commit`, `codefire-store/src/lib.rs` | manifest/fire delta/verification builders in `main.rs` | commit blocker, object tamper, sealed validation tests |
| merge and diff | FR-010, FR-011 | `main.rs`, `view.rs`, `view/file_diff.rs`, `view/semantic_diff.rs` | `merge_patch_idempotency.rs` | merge conflict, deep ancestor, diff tests |
| remote circulation | FR-012 | `remote.rs`, `http.rs` | `http_tls.rs`, `signatures.rs`, `remote/plans.rs` | file remote, HTTP, TLS, signature, nonce tests |
| health and migration | FR-013, FR-014 | `doctor.rs`, `doctor/checks.rs`, `migration.rs` | `storage.rs`, `explain.rs` | doctor/migrate layout and repairability tests |
| storage observability | FR-015 | `storage.rs` | `codefire-store`, `remote.rs` | object grouping, large object, remote storage tests |
| evidence | FR-016 | `evidence.rs`, `evidence/batch.rs` | `codefire-store`, `automation.rs` | artifact ref, command capture, argv, batch tests |
| context/explain | FR-017, FR-018 | `context.rs`, `explain.rs` | `automation.rs`, active scan readers | context pack and explanation tests |
| installation/completion | FR-019 | `install.sh`, `completion.rs` | `tests/test_codefire_cli.py`, README/runbook | installer smoke and docs consistency tests |

The important reading rule:

- product docs describe the contract;
- architecture docs describe the allowed shape;
- source files implement one slice of the contract;
- tests must pin the contract at the smallest useful layer.

## 2. Layered Runtime Picture

CodeFire is easiest to imagine as five cooperating machines.

```text
Command Machine
  parse argv, select command, resolve path, choose JSON/text, assign exit code

Repository Machine
  find .codefire, find .codefire-open, validate branch/open/base identity,
  hold locks, write active state, update branch heads

Domain Machine
  extract Atoms, parse Trace Links and policy, compute changed Atoms,
  generate fires, evaluate verification blockers

Object Machine
  canonicalize payloads, compute object IDs, write immutable records,
  validate sealed commit object graphs

Automation Machine
  wrap command results, diagnostics, metrics, and next_actions into stable JSON
```

Concrete ownership:

| Machine | Main files | Must stay out of |
|---|---|---|
| Command | `codefire-cli/src/main.rs`, `completion.rs`, `verification.rs`, command modules | object identity details |
| Repository | `main.rs`, `cli_model.rs`, `idempotency.rs`, `open_clone_idempotency.rs` | semantic Atom/Trace decisions |
| Domain | `codefire-core/src/lib.rs` | CLI rendering and process exit codes |
| Object | `codefire-store/src/lib.rs`, `codefire-util/src/lib.rs` | open-directory state transitions |
| Automation | `automation.rs`, `metrics.rs`, `exit_code.rs` | filesystem mutation |

If a function mixes three or more machines, it is a refactor candidate.

## 3. Source File Responsibility Cards

### `crates/codefire-cli/src/main.rs`

This is the orchestration spine. It still contains too much implementation, but it should be read in stable bands:

1. module imports and shared constants;
2. branch/open state helpers;
3. `main` / `run` command dispatch;
4. help and command wrapper functions;
5. JSON/text output helpers;
6. argument parsers;
7. local workflow runners: scan, verify, extinguish, commit;
8. repository/open/branch/object helper services;
9. filesystem primitives.

Design intent:

- keep command match arms shallow;
- move command-specific models into modules;
- keep core/store invariants delegated downward;
- make every mutation path obvious: parse, resolve, validate, plan, lock, revalidate, write, render.

Current concrete anchors:

| Anchor | Meaning | Do not hide |
|---|---|---|
| `run(args)` | top-level command selection | command aliases, help behavior, JSON failure policy |
| `local_workflow_handler` | common entry for scan/verify/fire/extinguish/commit | workflow command dispatch changes |
| `compute_scan` | source tree to `ScanResult` and active scan state | Atom/base/trace/policy comparison |
| `compute_verify` | scan plus policy plus external checks | verification blocker semantics |
| `run_extinguish` | active fire to resolution mutation | stale basis and evidence reference validation |
| `run_commit` | open state to sealed commit and branch head | object write ordering, branch atomicity, idempotency |
| `open_context` | marker/registry/base validation | copied open directory and branch identity checks |
| `write_json_atomic` | active/branch JSON publication | partial write and directory sync behavior |

When a new feature touches one of these anchors, the implementation should first add a narrow test around the anchor, then refactor only enough to keep the anchor readable.

### `crates/codefire-core/src/lib.rs`

This is the semantic engine. It answers:

- which files produce Atoms;
- which Atom IDs and hashes exist now;
- which Trace Links exist;
- which required links are missing;
- which fires should exist;
- which verification blockers remain.

It should not answer:

- what CLI text says;
- what exit code is returned;
- where active state JSON is written;
- how a remote request is authenticated.

Source zones inside the file:

| Zone | Primary structs/functions | Change examples |
|---|---|---|
| domain structs | `Atom`, `TraceGraph`, `Fire`, `ScanResult`, `Verification`, `Resolution` | add field to persisted semantic output |
| config and policy parsing | `parse_trace_policy`, `parse_verification_policy` | new required-link or verification policy |
| scan semantics | `changed_atoms`, `required_link_missing`, `build_scan_result` | new fire reason, better grouping |
| verification semantics | `build_verification`, `stale_resolutions` | new blocker or warning category |
| resolution basis | `build_resolution` | new stale-basis input |
| extractors | `build_atom_index`, `extract_markdown_atoms`, `extract_explicit_cf_atoms` | new Atom kind or content hash rule |

The core module is allowed to read source files for indexing. It is not allowed to write repository state, decide exit codes, or format human/JSON command output.

### `crates/codefire-store/src/lib.rs`

This is the identity engine. It answers:

- what canonical JSON is;
- how an object ID is calculated;
- where object records live;
- whether a commit object graph is valid.

Any feature that adds a commit root, object kind, or reference-bearing payload must update store validation, doctor visibility, and storage reporting together.

Source zones inside the file:

| Zone | Primary functions | Change examples |
|---|---|---|
| object kind registry | `object_kind_by_type`, `object_prefix`, `object_subdir` | new object type |
| identity | `canonical_json`, `object_digest`, `object_id`, `object_record` | object identity versioning |
| persistence | `store_object`, `read_object_record`, `object_record_path` | write durability or lookup performance |
| sealed validation | `validate_sealed_commit`, root/reference validators | new commit root or evidence reference |
| commit builder | `commit_payload` | certificate/root shape changes |

If object identity changes, treat it as a compatibility migration, not a local refactor.

### `crates/codefire-cli/src/automation.rs`

This is the machine contract. It translates domain/command results into:

- `codefire.command_result.v1` envelopes;
- stable `data` payloads;
- bounded `diagnostics`;
- executable `next_actions`.

Do not hide command-specific write logic in this file. If `automation.rs` needs to know too much about mutation internals, the command result model is probably missing a proper result struct.

Source zones inside the file:

| Zone | Functions | Purpose |
|---|---|---|
| envelope | `command_result_envelope` | stable wrapper for tool consumption |
| command names | `cli_command` | canonical command text in next_actions |
| status/scan data | `status_data_json`, `scan_data_json` | lifecycle state summaries |
| verification data | `verification_data_json_with_filter`, diagnostic helpers | blocker/warning machine payload |
| next actions | `*_next_actions`, `bounded_next_actions` | action suggestions with caps |

Automation output is a public API. Changing field names requires docs and compatibility tests.

### Command Modules

Command modules own one vertical surface.

| Module | Typical contents |
|---|---|
| `evidence.rs` | options, validation, dry-run plan, command/artifact capture, result JSON |
| `storage.rs` | read-only capacity model, local/remote object grouping, warnings |
| `doctor/*` | health check model, check execution, JSON/text report |
| `migration.rs` | compatibility check and migration plan |
| `context.rs` | active scan based context pack |
| `view/*` | commitish resolution, file diff, semantic diff, review pack |
| `remote/*` | file remote mutation, dry-run plans, idempotency |
| `http*` | transport boundary and TLS |
| `signatures.rs` | HMAC signatures, key policy, nonce replay |

## 3.1 Feature Change Blueprint

Use this table to understand which files should normally change together.

| Feature change | Contract docs | Source files | Persistence | Tests |
|---|---|---|---|---|
| new CLI flag | `docs/04_cli_spec.md`, `docs/automation_interface.md` if JSON changes | parser in `main.rs` or command module, completion docs tests | none unless flag mutates state | parser/help/docs consistency plus command behavior |
| new scan/fire rule | `docs/09_index_trace_policy.md`, `docs/06_fire_extinguish.md` | `codefire-core/src/lib.rs`, `main.rs::compute_scan`, `automation.rs` | active scan/fires, commit fire delta | core unit plus scan CLI regression |
| new verification blocker | `docs/05_consistency_model.md`, `docs/04_cli_spec.md` | `codefire-core`, `verification.rs`, `automation.rs`, `exit_code.rs` | active verification, commit certificate | verification JSON and commit rejection tests |
| new object kind | `docs/08_object_store.md`, `docs/11_internal_architecture.md` | `codefire-store`, writer module, `doctor`, `storage`, `remote` | `.codefire/objects/<kind>` | object tamper, doctor, storage, remote copy tests |
| remote mutation change | `docs/10_merge_remote_server.md`, automation docs | `remote.rs`, `remote/plans.rs`, `http.rs`, `signatures.rs` | remote branches/MRs/idempotency/nonce | file remote and HTTP remote tests |
| evidence behavior | `docs/06_fire_extinguish.md`, `docs/04_cli_spec.md` | `evidence.rs`, `evidence/batch.rs`, `storage.rs`, `automation.rs` | evidence objects, artifact refs | command capture, dry-run, batch, storage tests |
| install/completion | README/runbook, CLI spec | `install.sh`, `completion.rs`, docs tests | install prefix only | shell/Python smoke and docs consistency |

Any row that changes JSON output must update `docs/automation_interface.md` and add a test that inspects the actual JSON shape.

## 3.2 Repository State Blueprint

The source code should make these state transitions visible.

```text
closed branch
  -- open --> open-clean
  -- edit/scan/fire --> open-burning
  -- extinguish/verify with no blockers --> open-consistent
  -- commit --> open-clean at new base commit
  -- close --> closed
```

State inputs:

| Input | Source owner | Persistent source |
|---|---|---|
| branch head | `main.rs`, future repository service | `.codefire/branches/<branch>.json` |
| open identity | `open_context`, opened registry helpers | `.codefire-open`, `.codefire/opened/<branch>.json` |
| base commit | `OpenContext` | open marker and opened registry |
| changed atoms | `codefire-core::build_scan_result` | active `scan.json` |
| open fires | scan/fire/extinguish runners | active `fires.json` |
| stale resolutions | `codefire-core::stale_resolutions` | active `resolutions.json`, current basis |
| verification blockers | `build_verification`, external checks | active `verification.json` |

The same state reducer should eventually feed status, scan, verify, commit, close, and automation next_actions.

## 4. Open Directory Lifecycle

The core lifecycle is:

```text
init
  creates .codefire repository and initial sealed commit

open
  materializes branch into a directory
  writes .codefire-open and .codefire/opened entry

edit files
  changes live only in the working directory

scan
  builds current Atom index and Trace Graph
  compares with base commit
  writes active scan/fires

extinguish
  writes active resolutions

verify
  re-runs scan, external checks, policy checks
  writes active verification

commit
  rejects blockers
  writes immutable objects
  updates branch head
  resets active state
```

The repository intentionally does not store a WIP snapshot. If a future change stores file content inside `.codefire/active`, it violates the product model unless a product-level requirement changes first.

## 5. Command Implementation Recipe

When implementing or changing a command, use this source-level recipe.

1. Update the contract.
   - `docs/04_cli_spec.md`
   - `docs/automation_interface.md` for JSON/exit/next_actions
   - requirement/design docs if behavior is product-level

2. Add or update the options/result types.
   - Prefer `cli_model.rs` or a command module.
   - Avoid returning raw `serde_json::Value` from business logic.

3. Implement parser and validation separately.
   - Parser handles argv shape.
   - Validation handles semantic preconditions.

4. Build a dry-run plan before mutation.
   - Plan must include target repo/open, branch, command-specific inputs, and expected writes.
   - Dry-run must not write objects, branches, active state, remote files, or idempotency records.

5. Acquire locks before mutable writes.
   - Revalidate branch/open/object state after lock acquisition.
   - Keep lock scope narrow but complete.

6. Persist through a small set of helpers.
   - Objects go through `codefire-store`.
   - JSON files use atomic write helpers.
   - Branch heads update only after validation.

7. Render through dedicated code.
   - Text renderer for humans.
   - JSON renderer/envelope for automation.
   - Failure with `--json` still returns a JSON envelope.

8. Add tests.
   - parser/validation tests for command shape;
   - runner tests for state mutation;
   - JSON tests for automation contract;
   - integration tests for lifecycle regressions.

## 6. State Files And Writers

| Persistent path | Writer | Reader | Notes |
|---|---|---|---|
| `.codefire/objects/*` | `codefire-store`, evidence/commit/remote callers | store validation, show/diff/doctor/storage | immutable after write |
| `.codefire/branches/*.json` | commit, clone, merge/apply, remote sync | status/open/upload/diff | must point to sealed commit |
| `.codefire/opened/*.json` | open/close | status, open-context resolution, doctor | guards copied open directories |
| `.codefire/active/*/scan.json` | scan/verify/commit preview | context, verify, status, commit | derived metadata |
| `.codefire/active/*/fires.json` | scan/fire/extinguish | verify, context, commit | unresolved responsibility state |
| `.codefire/active/*/resolutions.json` | extinguish | verify, commit | basis must become stale when source relation changes |
| `.codefire/active/*/verification.json` | verify/commit preview | status, commit | must not imply commit if clean and unchanged |
| `.codefire/idempotency/*` | local mutators | local mutators | should not store sensitive full plans unnecessarily |
| remote project files | remote/http commands | remote/http/doctor/storage | remote is a trust boundary |

## 7. Cross-Cutting Invariants

| Invariant | Source places to check when changing |
|---|---|
| branch head is sealed | `main.rs`, `codefire-store`, `remote.rs`, `doctor/checks.rs` |
| JSON envelope is stable | `automation.rs`, `exit_code.rs`, command modules, docs tests |
| no WIP snapshot in repo | active state writers in `main.rs`, `context.rs`, evidence/storage docs |
| dry-run has no side effects | command parser/runner, idempotency helpers, tests |
| batch validate before write | `batch.rs`, `fire/batch.rs`, `evidence/batch.rs`, `link_batch.rs` |
| path behavior is consistent | shared parser helpers, completion/help, CFB path issues |
| diagnostics match exit codes | `exit_code.rs`, `automation.rs`, verify/doctor/migrate modules |
| large outputs are bounded | scan/context/commit dry-run/batch JSON builders |
| object graph validation is complete | `codefire-store`, `doctor`, `remote`, `show/diff` |
| metrics are honest | `metrics.rs`, command runners, docs/automation contract |

## 8. How To Review A CodeFire Change

Use this review route.

1. Identify the touched command or domain.
2. Read the relevant row in `source-code-map.md`.
3. Follow the command path in `command-source-trace.md`.
4. Check whether the change crosses machines from section 2.
5. Verify product/architecture docs were updated if the contract changed.
6. Check tests at the smallest layer and lifecycle layer.
7. Run or request the relevant verification commands.
8. If behavior affects dogfooding, update `bug-backlog.md`, issue detail files, or improvement-cycle docs.

## 9. Current Architecture Pressure Points

These are known areas where the source shape is not yet ideal.

| Pressure point | Why it matters | Direction |
|---|---|---|
| `main.rs` still owns too many runners | parsing, state mutation, and rendering are harder to reason about | extract state reducer and workflow services before more command growth |
| path option parsing is not fully centralized | commands accept different path forms | shared command metadata/parser layer |
| JSON payloads can become too large | automation and dogfooding suffer on initial imports | summarized/default output plus explicit verbose/detail flags |
| state labels are duplicated | `open-clean`, `open-burning`, `open-consistent` can disagree | single state reducer |
| doctor/migrate/storage layout knowledge can diverge | recovery tools may disagree | shared layout definition |
| active-state next_actions can be stale | automation may perform useless commit/verify steps | next_actions generated from reducer and changed-count context |

Treat these as design debt to retire during v0.8/v0.9 planning, not as reasons to add workaround code.
