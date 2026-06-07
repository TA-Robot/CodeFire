# CodeFire Source Data Model Catalog

この文書は、CodeFireの実装を構成する主要なRust型、JSON payload、永続化ファイルをカタログ化する。目的は、ソースコードを開く前に「どのデータがどこから来て、どこへ保存され、どの関数が触るか」を想像できる状態にすることである。

## 1. Naming Rule

CodeFireのデータは概ね次の命名で読む。

| Name shape | Meaning | Example |
|---|---|---|
| `*Options` | parserが返すcommand input | `CommitOptions`, `EvidenceAddOptions` |
| `*Result` | runnerが返すcommand result | `CommitResult`, `UploadResult` |
| `*Execution` | domain resultとcontextを束ねた内部実行結果 | `ScanExecution`, `VerifyExecution` |
| domain noun | core semantic record | `Atom`, `Fire`, `Verification` |
| store noun | immutable object identity record | `ObjectRecord` |
| JSON helper | automation/text boundary | `scan_data_json`, `storage_report_data_json` |

Raw `serde_json::Value` は最後のrenderingまたはobject payloadのために使う。business logicの主語が長期間 `Value` のままなら、型が足りていない。

## 2. CLI Shared Types

Primary file: `crates/codefire-cli/src/cli_model.rs`

| Type | Fields to picture | Role |
|---|---|---|
| `Status` | `branch`, `state`, `base`, `open_fires` | `status` and automation summary |
| `Branch` | `name`, `head`, `state` | branch list row |
| `InitOptions` | `path`, `force` | init parser output |
| `InitResult` | `repo_root`, `main_commit` | init runner output |
| `OpenOptions` | `branch`, `path`, `dry_run`, `json_output`, `lock`, `idempotency_key` | open parser output |
| `OpenResult` | `branch`, `open_dir`, `plan` | open result/plan |
| `PathJsonOptions` | `path`, `json_output`, `metrics` | common read/workflow command options |
| `ExtinguishOptions` | `path`, `fire_id`, `resolution`, `rationale`, `evidence`, `evidence_refs`, dry-run/JSON/lock/idempotency/editor flags | single extinguish command input |
| `ExtinguishResult` | `display_id`, `fire_uid`, `plan` | extinguish result/plan |
| `CommitOptions` | `path`, `message`, dry-run/JSON/lock/idempotency/signature fields | commit command input |
| `CommitResult` | `commit_id`, `branch`, `plan` | commit result |
| `CloneOptions` / `CloneResult` | source/new branch plus dry-run/lock/idempotency, plan | branch clone |
| `MergeOptions` / `MergeResult` | source/target branch, heads, base, file actions, conflicts, applied/dry-run | local merge |
| `OpenContext` | `repo_root`, `open_dir`, `branch`, `registry_path`, `registry` | open-directory identity and active-state gateway |
| `ScanExecution` | `OpenContext`, `active_state_path`, `ScanResult`, fires | scan result plus writable state path |
| `VerifyExecution` | `ScanResult`, `Verification` | verification result bundle |

### Options/Result Placement Rule

Use `cli_model.rs` for shapes shared by several command paths or currently rooted in `main.rs`. Use command modules for vertical surfaces:

| Module | Local shapes |
|---|---|
| `fire.rs` | `FireOptions`, `FireResult` |
| `fire/batch.rs` | `FireBatchOptions` |
| `batch.rs` | `BatchExtinguishOptions`, `BatchExtinguishResult` |
| `evidence.rs` | `EvidenceAddOptions`, `EvidenceAddResult` |
| `storage.rs` | `StorageReportOptions`, `StorageReport`, stats structs |
| `doctor.rs` | `DoctorOptions`, `DoctorReport`, `DoctorIssue` |
| `migration.rs` | `MigrateOptions`, migration report structs |
| `remote.rs` | upload and merge request options/results |
| `view.rs` | `DiffOptions`, `ReviewPackOptions`, `PatchExportOptions` |
| `context.rs` | `ContextOptions`, context pack records |
| `explain.rs` | `ExplainOptions`, `ExplainResult` |

## 3. Core Domain Types

Primary file: `crates/codefire-core/src/lib.rs`

| Type | Shape | Meaning |
|---|---|---|
| `Selector` | `type`, `value` | how an Atom is located inside an artifact |
| `Atom` | `atom_id`, `kind`, `artifact_path`, `selector`, `content_hash` | smallest tracked semantic unit |
| `AtomIndex` | `type`, `version`, `atoms`, `duplicate_atom_ids` | current or sealed set of Atoms |
| `TraceLink` | `from`, `to`, `type`, `link_id`, `link_hash` | edge between two Atoms |
| `TraceGraph` | `type`, `version`, `links` | graph of responsibility links |
| `RequiredLinkRule` | `type`, `target_kind`, `min` | policy requirement from one Atom kind to another |
| `TracePolicy` | `required_links` | scan-time missing-link policy |
| `VerificationPolicy` | booleans, `required_links`, `verification` commands | commit-gate policy |
| `VerificationCommand` | `id`, `command`, `cwd` | external check invoked during verify |
| `MissingRequiredLink` | `atom_id`, `required_type`, `target_kind`, `min`, `found` | policy gap |
| `FireAtomRef` | `atom_id`, optional `content_hash_at_fire` | fire endpoint with basis hash |
| `Fire` | IDs, status, severity, source, target, reason, trace path, creation metadata | unresolved responsibility |
| `ScanResult` | base commit, atom index, trace graph, changed atoms, open fires | complete scan output |
| `FailedCheck` | `id`, `command`, `output` | external verification failure |
| `StaleResolution` | `resolution_uid`, `reason` | resolution invalidated by changed basis |
| `MissingEvidenceRef` | `resolution_uid`, `evidence_id` | evidence reference that cannot be loaded |
| `Verification` | result, gate counters, missing links, stale resolutions, duplicate IDs, timestamp | commit gate result |
| `ResolutionBasis` | source atom basis, target atom basis, trace link bases, policy hash | immutable reason a resolution was valid |
| `Resolution` | resolution UID, fire UID, type, rationale, evidence, refs, basis, timestamp, status | active or sealed resolution record |

Core output types are serialized directly into active state and sealed object payloads. Field changes are contract changes.

## 4. Core Function Inputs And Outputs

| Function | Inputs | Output | Filesystem behavior |
|---|---|---|---|
| `build_atom_index(open_dir)` | open directory and `codefire.yaml` | `AtomIndex` | reads files only |
| `current_trace_graph(open_dir)` | `codefire.links.yaml` | `TraceGraph` | reads files only |
| `parse_links(path)` | link YAML path | `TraceLinkInput` list | reads one file |
| `parse_trace_policy(open_dir)` | `codefire.policy.yaml` | `TracePolicy` | reads policy file if present |
| `parse_verification_policy(open_dir)` | `codefire.policy.yaml` | `VerificationPolicy` | reads policy file if present |
| `changed_atoms(current, base)` | two `AtomIndex` values | atom ID list | pure |
| `required_link_missing(...)` | index, graph, policy | missing-link list | pure |
| `build_scan_result(...)` | current/base index, graph, policy, existing state | `ScanResult` | pure after inputs |
| `stale_resolutions(...)` | resolutions, current basis inputs | stale list | pure after inputs |
| `build_resolution(...)` | fire, atom index, graph, request | `Resolution` | pure after inputs |
| `build_verification(...)` | scan, resolutions, failed checks, policy | `Verification` | pure after inputs |
| `policy_hash(policy)` | verification policy | hash string | pure |

The CLI layer decides when to persist these outputs.

## 5. Object Store Types

Primary file: `crates/codefire-store/src/lib.rs`

| Type/Function | Shape | Meaning |
|---|---|---|
| `ObjectRecord` | `object_id`, `type`, `hash`, `payload` | immutable stored record |
| `canonical_json` | `Value -> bytes` | deterministic compact JSON |
| `object_digest` | `type_tag + payload -> sha256` | identity hash |
| `object_id` | type-specific prefix plus digest | public object reference |
| `object_prefix` | type tag to prefix | ID family |
| `object_subdir` | type tag to object subdir | storage location |
| `store_object` | write and validate record | immutable publish |
| `read_object` | load validated payload | trusted object read |
| `read_object_record` | load validated record | trusted object read with metadata |
| `validate_sealed_commit` | commit graph validation | branch/remote trust gate |
| `commit_payload` | roots/certificate to commit object payload | commit object builder |

Object record mental shape:

```json
{
  "object_id": "CF-COMMIT-...",
  "type": "commit",
  "hash": "64 lowercase hex sha256",
  "payload": {
    "type": "commit",
    "version": 1
  }
}
```

Known object families are centralized in the store crate. A new object type must be visible to object path resolution, doctor, storage, remote copy, and commit validation if commits can reference it.

## 6. Sealed Commit Payload

A commit payload should be pictured as:

```json
{
  "type": "commit",
  "version": 1,
  "parents": ["CF-COMMIT-..."],
  "message": "...",
  "roots": {
    "content_manifest": "CF-MANIFEST-...",
    "atom_index": "CF-ATOMINDEX-...",
    "trace_graph": "CF-TRACE-...",
    "fire_delta": "CF-FIRELEDGER-...",
    "resolution_ledger": "CF-RESOLUTION-...",
    "verification": "CF-VERIFY-...",
    "policy": "CF-POLICY-..."
  },
  "certificate": {
    "result": "passed",
    "open_required_fires": 0,
    "failed_checks": 0,
    "missing_required_links": 0,
    "stale_resolutions": 0
  }
}
```

The exact certificate can evolve, but validation must keep the invariant that branch heads point only at structurally valid sealed commit graphs.

## 7. Repository Files

### `.codefire/branches/<branch>.json`

Typical shape:

```json
{
  "type": "branch",
  "version": 1,
  "name": "main",
  "head": "CF-COMMIT-...",
  "state": "open-clean"
}
```

Writers:

- init;
- commit;
- clone/open state updates;
- merge apply;
- remote sync.

Readers:

- status;
- branch list;
- open;
- merge;
- diff/show;
- upload;
- doctor.

### `.codefire/opened/<branch>.json`

Typical shape:

```json
{
  "type": "opened",
  "version": 1,
  "branch": {
    "name": "main",
    "opened_from_commit": "CF-COMMIT-..."
  },
  "open": {
    "open_instance_id": "open-...",
    "opened_path": "/abs/path/to/open",
    "opened_at": "2026-06-07T00:00:00Z",
    "active_state_path": "/abs/path/to/repo/.codefire/active/open-...",
    "current_base_commit": "CF-COMMIT-..."
  },
  "state": {
    "last_known": "open-clean"
  }
}
```

This record ties the branch to one open directory and one active state directory.

### `<open-dir>/.codefire-open`

Typical shape:

```json
{
  "type": "codefire_open",
  "version": 1,
  "repo": {
    "root": "/abs/path/to/repo"
  },
  "branch": {
    "name": "main",
    "opened_from_commit": "CF-COMMIT-..."
  },
  "open": {
    "open_instance_id": "open-...",
    "opened_path": "/abs/path/to/open"
  }
}
```

`open_context(start)` cross-checks this marker with the opened registry and branch state.

## 8. Active State Files

Active state is mutable, derived, and per open directory.

### `state.json`

```json
{
  "type": "active_state",
  "version": 1,
  "state": "open-clean"
}
```

Writers:

- open initialization;
- scan;
- verify;
- extinguish;
- merge/patch import;
- commit reset.

### `scan.json`

Usually stores serialized `ScanResult`.

Important fields:

- `base_commit`;
- `atom_index`;
- `trace_graph`;
- `changed_atoms`;
- `open_fires`.

Writers:

- scan;
- verify;
- commit preview through verification path.

### `fires.json`

Usually stores `Vec<Fire>`.

Important fields:

- `fire_uid`;
- `display_id`;
- `status`;
- source/target atom refs;
- `reason`;
- `key`;
- optional resolution linkage.

Writers:

- scan;
- manual fire;
- extinguish;
- merge/patch when marking state burning.

### `resolutions.json`

Usually stores `Vec<Resolution>`.

Important fields:

- `resolution_uid`;
- `fire_uid`;
- `resolution_type`;
- `rationale`;
- `evidence`;
- `evidence_refs`;
- `basis`;
- `status`.

Writers:

- extinguish;
- batch extinguish;
- interactive/all-matching extinguish.

### `verification.json`

Usually stores serialized `Verification`.

Important fields:

- `result`;
- `open_required_fires`;
- `failed_checks`;
- `missing_required_links`;
- `stale_resolutions`;
- `missing_evidence_refs`;
- `duplicate_atom_ids`;
- `verified_at`.

Writers:

- verify;
- commit verification path.

## 9. Automation Envelope

Machine output should look like:

```json
{
  "type": "codefire.command_result.v1",
  "ok": true,
  "command": "verify",
  "exit_code": 0,
  "schema": "codefire.verify.v1",
  "data": {},
  "diagnostics": [],
  "next_actions": [],
  "metrics": {}
}
```

Field rules:

| Field | Rule |
|---|---|
| `ok` | success/failure of the command contract, not necessarily "no blockers" |
| `command` | canonical label from `cli_command` or equivalent |
| `exit_code` | must align with `exit_code.rs` |
| `schema` | stable payload identifier |
| `data` | command-specific structured result |
| `diagnostics` | ordered warnings/errors/blockers |
| `next_actions` | executable suggestions, preferably with target path/repo |
| `metrics` | optional, must not change command semantics |

## 10. Evidence Data

Evidence command data has two layers:

| Layer | Object | Meaning |
|---|---|---|
| artifact ref | `artifact_ref` object | points to file path or URI and records hashes/sizes where possible |
| evidence | `evidence` object | records label, command/artifact source, summaries, and artifact ref |

`EvidenceAddResult` includes:

- repo root;
- dry-run flag;
- evidence ID;
- optional artifact ref ID;
- optional object path;
- command exit status;
- timeout/truncation flags;
- diagnostics;
- optional plan.

The command stores bounded summaries, not unbounded stdout/stderr.

## 11. Storage Data

`StorageReport` summarizes:

- repo root;
- object area files/bytes;
- active state files/bytes;
- idempotency files/bytes;
- object types by count/bytes;
- largest objects;
- external artifact refs and referenced bytes;
- remote storage stats;
- warnings;
- quick/full mode.

Storage report structs are read models. They must not become repair or migration state.

## 12. Remote Data

Remote operations use:

| Shape | Purpose |
|---|---|
| `UploadOptions` / `UploadResult` | copy local sealed branch head and object graph to remote |
| `RemoteProjectOptions` | list/read remote project |
| `RequestMergeOptions` / `RequestMergeResult` | create merge request record |
| `RequestReviewOptions` / `RequestReviewResult` | record reviewer decision |
| `RequestApplyOptions` / `RequestApplyResult` | apply accepted merge request to remote target branch |

Remote project files are a separate trust boundary. Always picture remote mutation as:

```text
parse URL
  -> load local/remote branch heads
  -> validate object graph
  -> verify signature/nonce if configured
  -> lock remote project
  -> write remote objects/branches/MR/idempotency
```

## 13. Test Data Map

Primary file: `crates/codefire-cli/src/tests.rs`

The test file is broad, but test names reveal source behavior:

| Test name pattern | Behavior covered |
|---|---|
| `parse_*_args_*` | parser contract |
| `*_dry_run_*without_writing*` | dry-run side-effect guard |
| `*_idempotency_*` | replay/conflict behavior |
| `doctor_*` | layout/object/open/active health |
| `storage_*` | capacity and remote/object counting |
| `evidence_*` | evidence object and command capture |
| `remote_*` / `http_*` | remote project, HTTP, TLS, signatures |
| `merge_*` | branch merge, conflicts, ancestors |
| `verify_*` | verification diagnostics and state behavior |

When adding a feature, tests should exist at the smallest layer that owns the invariant and at the CLI JSON layer if automation behavior changes.

## 14. Quick Debugging Matrix

| Symptom | First data to inspect | Likely source |
|---|---|---|
| wrong state label | branch record, opened registry, `active/state.json` | `status_state`, `scan_branch_state`, state writers |
| scan misses change | current/base `AtomIndex`, `codefire.yaml`, extractor output | `codefire-core::build_atom_index`, `changed_atoms` |
| missing link not reported | `TraceGraph`, `TracePolicy`, atom kinds | `required_link_missing`, policy parser |
| verify permits bad commit | `Verification`, `VerificationPolicy`, failed checks | `build_verification`, `compute_verify`, `run_commit` |
| commit object invalid | commit roots, object records | `codefire-store`, `run_commit` root storage |
| JSON missing field | command data helper, envelope helper | `automation.rs` or module `*_data_json` |
| dry-run wrote files | operation plan path vs write path | command runner and idempotency helpers |
| remote accepts bad graph | remote copied objects, branch head validation | `remote.rs`, `http.rs`, `validate_sealed_commit` |

This matrix should be kept current as modules are extracted from `main.rs`.
