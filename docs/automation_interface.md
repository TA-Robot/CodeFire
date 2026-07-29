# Automation Interface

CodeFire v0.6 exposes stable JSON output for state and diagnostic commands through a shared command result envelope.
The schema identifier is implemented as `COMMAND_RESULT_SCHEMA` in `crates/codefire-cli/src/automation.rs` and currently has the value `codefire.command_result.v1`.

## Command Result Envelope

```json
{
  "schema": "codefire.command_result.v1",
  "command": "verify",
  "ok": false,
  "exit_code": 11,
  "repo": "/workspace/example",
  "data": {},
  "diagnostics": [],
  "next_actions": []
}
```

Fields:

```text
schema: stable schema identifier
command: command name without arguments
ok: true when the command result is successful
exit_code: intended process exit code for this result
repo: repository root path when known, otherwise null
data: command-specific result object
diagnostics: machine-readable diagnostics
next_actions: machine-readable remediation hints
```

Human output may change between releases. JSON output with the same schema version must remain backward compatible.
Suggested commands use the installed CLI display name `codefire`.

Checked base schemas live under `schemas/public/`. `codefire capabilities --json` advertises those
schema IDs and a generated `codefire.support.v1` manifest containing readable/current versions,
missing-version policy, canonical JSON and public-ID profiles, runtime roles, command targets, and
pagination mode. A schema is not advertised until it is checked in. Unknown persisted versions fail
closed unless that format explicitly declares a legacy missing-version case.

`next_actions` entries use this shape:

```json
{
  "kind": "context_atom",
  "command": "codefire context --atom REQ-session --depth 2 --json",
  "reason": "inspect the atom and nearby trace graph before adding the missing link",
  "target": {"atom_id": "REQ-session"}
}
```

Fields:

```text
kind: stable action class
command: suggested command or shell command
reason: short machine-readable remediation reason
target: action-specific target object
```

`next_actions` is bounded to at most 12 entries per command result envelope. When additional actions are available, the final entry has `kind: "next_actions_omitted"` and includes `target.omitted` plus `target.limit`.

## Exit Codes

CodeFire v0.6 uses a stable process exit code taxonomy. The `exit_code` field in JSON command result envelopes must match the intended process exit code for the same command result.

| Code | Meaning | Current implementation |
|---:|---|---|
| 0 | success | all successful commands |
| 1 | generic user-facing failure | IO and unclassified failures |
| 2 | invalid CLI usage or invalid config | CLI parse errors, invalid config, not inside a required repo/open directory |
| 10 | open fires block operation | `verify` with required open fires |
| 11 | blocking required links are missing | `verify` with missing required trace links |
| 12 | stale resolution blocks operation | `verify` with stale resolutions |
| 13 | duplicate Atom ID blocks operation | `verify` with duplicate Atom IDs |
| 14 | verification command failed | `verify` with failed configured checks |
| 15 | merge conflict blocks operation | `verify`, `commit`, and blocked `merge --continue` with unresolved text/binary/semantic facts |
| 20 | repository corruption detected | invalid marker/repository JSON or structural corruption |
| 21 | object hash or object reference invalid | object id/hash/reference validation failures, including missing evidence refs in `verify` |
| 22 | sealed commit validation failed | sealed commit graph/certificate validation failures |
| 30 | lock contention | local repository/resource lock already held or `--lock-timeout` elapsed |
| 31 | remote rejected request | HTTP remote error response |
| 32 | authentication or signature failure | reserved for signature enforcement |
| 33 | idempotency conflict | same idempotency key reused with a different payload |
| 40 | migration incompatibility | reserved for migration checks |
| 50 | external artifact missing or hash mismatch | reserved for artifact validation |
| 60 | deterministic resource budget exhausted | graph/object/HTTP decision-work limit reached before the attempted effect |

`resource_budget_exceeded` diagnostics preserve `profile`, `dimension`, `observed`, `limit`, `phase`,
`command`, and `continuation_supported` in local envelopes and through HTTP server/client transport. `observed`
is the first rejected `limit + 1` unit; the rejected atomic reservation does not partially advance domain work or
return `ok=true`. HTTP maps this outcome to 413 before a success header.

`capabilities --json` publishes a `decision_work_profile` entry in `support.formats` with the exact
serialized local and HTTP `codefire.decision-work.v1` profiles, including limits, applicable phases,
exit 60 semantics, and current coverage. Keeping this as a versioned format entry preserves the
`codefire.capabilities.v1` object shape. Coverage is deliberately `partial`: only listed metric
commands and the listed Rust request/response transport boundary are claimed. Python transport,
handler-internal work, deadline, connection admission, and unlisted surfaces remain explicit non-claims
until their integrated runtime gates are complete.

When multiple `verify` blockers are present, CodeFire reports the first blocker class in this order: unresolved merge conflicts, open fires, missing required links, stale resolutions, missing evidence refs, duplicate Atom IDs, failed verification commands.

Mutating commands that acquire repository or remote resource locks—including live `scan` and `verify`—accept `--wait-lock` and `--lock-timeout <duration>`.
`--lock-timeout` accepts seconds (`2`, `2s`) or milliseconds (`250ms`) and returns exit code 30 when elapsed. Repository lock files are persistent, content-opaque kernel-lock inodes; contention is determined by the kernel lock, not pathname existence. Local and import-created `repo.lock` files are accepted only as no-follow regular files with exactly one provable filesystem link, their bytes are never truncated/written by acquisition, and pathname identity/provenance is rechecked after locking. Best-effort `pid` / `created_at` diagnostics come from the atomically replaced sibling `repo.lock.owner.json`, read no-follow with a 16 KiB cap. Invalid sidecars are ignored and never define ownership.
When a remote mutator is run with `--json`, lock contention prints a `codefire.command_result.v1` envelope with a `lock_contention` diagnostic and `retry_with_wait_lock` next action before exiting with code 30.

Idempotency keys are implemented incrementally for mutating commands:

```text
import --idempotency-key <key>
open --idempotency-key <key>
clone --idempotency-key <key>
merge --idempotency-key <key>
patch import --idempotency-key <key>
upload --idempotency-key <key>
request-merge --idempotency-key <key>
request-review --idempotency-key <key>
request-apply --idempotency-key <key>
commit --idempotency-key <key>
extinguish --idempotency-key <key>
```

Same key + same payload returns the stored result. Same key + different payload returns exit code 33.

`import` uses a version-2 hash-only record: operation, normalized target scope, key, payload version/hash/mode, and original result are persisted without the raw canonical payload. Replay also validates the persistent bootstrap lock, durable import transaction, repository initialization provenance, sealed recorded base, and current marker/opened/branch subject. A result-only legacy marker is not replayable. `.codefire-import.lock` and `.codefire/locks/repo.lock` are persistent files whose kernel locks express ownership; `repo.lock.owner.json` is only diagnostic metadata. Automation must not delete any of them or treat existence as contention.

Import parses bootstrap, transaction, idempotency, and other operational JSON through a 1 MiB no-follow/non-blocking regular-file reader with metadata and limit-plus-one checks. Repository byte-copy reads are capped at 64 MiB. Imported user files are capped at 49,545,216 bytes (47.25 MiB) so their base64 blob records remain within the object store's 64 MiB serialized-record limit; oversized, growing, symlinked, or non-regular inputs fail closed.

## Supported Commands

```bash
codefire status --json
codefire status --json --metrics
codefire scan --json
codefire scan --json --metrics
codefire verify --json
codefire verify --details --json
codefire verify --details --blocking-only --json
codefire verify --details --json --metrics
codefire context --branch --json
codefire context --changed --json
codefire context --atom REQ-session --depth 2 --json
codefire context --fire FIRE-001 --json
```

`status --json` data:

```json
{
  "branch": "main",
  "state": "open-clean",
  "base": "CF-COMMIT-...",
  "open_fires": 0
}
```

`scan --json` data:

```json
{
  "branch_state": "open-burning",
  "base_commit": "CF-COMMIT-...",
  "full": false,
  "sample_limit": 50,
  "changed_count": 1,
  "changed_atom_count": 1,
  "non_atom_changed_file_count": 0,
  "open_fire_count": 0,
  "changed_atoms_omitted": 0,
  "non_atom_changed_files_omitted": 0,
  "open_fires_omitted": 0,
  "changed_atom_ids": ["REQ-session"],
  "changed_atoms": ["REQ-session"],
  "changed_atoms_sample": [
    {
      "atom_id": "REQ-session",
      "kind": "requirement",
      "path": "docs/spec/session.md",
      "selector": {"type": "heading", "value": "REQ-session"},
      "content_hash": "sha256:..."
    }
  ],
  "non_atom_changed_files": [],
  "open_fires": []
}
```

Use `scan --json --full` only when the complete changed atom/open fire arrays are needed. Default JSON is bounded for large initial imports.

When `--metrics` is present on `status`, `scan`, `verify`, or `branch list`, the command data includes:

```json
{
  "metrics": {
    "type": "codefire_metrics",
    "version": 1,
    "command": "scan",
    "phase_timings": {
      "total_ms": 12,
      "scan_pipeline_ms": 12,
      "atom_extraction_ms": 0,
      "trace_parse_ms": 0,
      "fire_build_ms": 0
    },
    "phases": [
      {"name": "total", "elapsed_ms": 12, "measured": true},
      {"name": "scan_pipeline", "elapsed_ms": 12, "measured": true},
      {"name": "atom_extraction", "elapsed_ms": 0, "measured": false}
    ],
    "counters": [{"name": "atoms", "value": 24}],
    "cache": {
      "status": "unimplemented",
      "enabled": false,
      "implementation": "none",
      "disabled_reason": "not_implemented",
      "entry_count": 0,
      "hit_count": 0,
      "miss_count": 0
    }
  }
}
```

`phase_timings` keeps stable `<phase>_ms` keys for scripts. `phases` carries the same values with `measured` metadata so uninstrumented subphases are not confused with measured zero-cost work. Cache fields are present even while cache is unimplemented to keep the future cache-enabled schema compatible.

`verify --json` data:

```json
{
  "result": "failed",
  "profile": "local_verification_v1",
  "phase": "post_execution",
  "subject": {
    "type": "codefire_assessment_subject",
    "version": 1,
    "kind": "live_open",
    "trust_kind": "local_unisolated",
    "subject_digest": "sha256:...",
    "content_digest": "sha256:...",
    "worktree_digest": "sha256:...",
    "immutable": false
  },
  "execution": [
    {
      "id": "unit-tests",
      "profile": "trusted_local_shell_v1",
      "phase": "verification_command",
      "subject_digest": "sha256:...",
      "input_digest": "sha256:...",
      "environment_digest": "sha256:...",
      "environment_policy": "minimal_allowlist_with_explicit_overrides",
      "stdout_bytes_seen": 120,
      "stdout_bytes_stored": 120,
      "stdout_truncated": false,
      "timed_out": false,
      "termination_scope": "process_group"
    }
  ],
  "trace_completeness_required": true,
  "open_required_fires": 1,
  "unresolved_merge_conflicts": [],
  "missing_required_links": [],
  "stale_resolutions": [],
  "missing_evidence_refs": [],
  "duplicate_atom_ids": [],
  "failed_checks": [],
  "diagnostic_filter": "blocking_only"
}
```

`verify --json` sets `exit_code` to the matching blocker code from the exit code taxonomy. For example, unresolved merge conflicts produce code `15`, missing required links produce code `11`, missing evidence refs produce code `21`, and failed configured verification commands produce code `14`.
`diagnostic_filter` is `all` by default and `blocking_only` when `--blocking-only` is present. Verification diagnostics include `blocking`; `missing_required_link` is `severity=warning` and `blocking=false` when `trace_completeness_required` is false.
`subject` is the exact versioned assessment identity shared with context/explain and a later certificate. `execution` never carries raw environment values; its digests, profile/phase, cwd classification, resource counters, timeout/termination scope, worktree before/after, and redaction count describe the trust boundary.

`status --json`, `scan --json`, `verify --json`, `doctor --json`, `storage report --json`, and `migrate ... --json` include remediation-oriented `next_actions`:

```text
status: verify, scan, context_changed, commit
scan: context_changed, context_fire, extinguish_fire, verify
verify: continue_merge, context_changed, scan, context_atom, refresh_resolution, repair_evidence_ref, rerun_check, commit
doctor: inspect_object_store_corruption, repair_branch_head, reopen_branch_workspace, inspect_doctor_report
storage report: inspect_storage_warnings
migrate: review_migration_plan, apply_migration_plan, retry_remaining_migration_actions, inspect_migration_blockers
```

Migration actions are derived from the current command mode and postcondition. A successful or exact-replay
`migrate apply` with no `remaining_actions` returns no action even though its historical `planned_actions` audit
is retained. A failed/partial apply only recommends a bounded revalidation retry when current remaining actions
exist; check and dry-run continue to project review and apply actions respectively.

`doctor --json` data:

```json
{
  "type": "codefire_doctor_report",
  "version": 1,
  "ok": false,
  "mode": "full",
  "skipped_checks": [],
  "checked": {"objects": 12, "branches": 1, "opened": 1, "active_files": 4},
  "issue_counts": {"total": 1, "blocking": 1},
  "issues": [
    {
      "severity": "error",
      "category": "object_store",
      "repairable": false,
      "blocking": true,
      "kind": "object_integrity_error",
      "message": "object hash mismatch: expected ..., got ...",
      "path": "/repo/.codefire/objects/commits/CF-COMMIT-....json"
    }
  ]
}
```

`doctor --quick --json` keeps layout, branch, opened registry, and active state shape checks, but skips full object store integrity scanning. The skipped check names are returned in `data.skipped_checks`.

Doctor layout diagnostics use the shared repository layout registry. Required layout gaps are blocking errors. Auto-creatable layout gaps are non-blocking repairable warnings and produce a `review_migration_plan` next action that points to `migrate dry-run --json`.

Doctor active state diagnostics include `missing_active_state_file` when an active state directory lacks `state.json`, and `active_state_invariant_violation` when `open-clean` / `open-consistent` coexists with non-empty `fires.json`.

For `cf://`, `cf+http://`, or `cf+https://` targets, `doctor --json` keeps the command-result envelope
and places a `codefire.remote_doctor.v1` report in `data`. The report identifies project and transport,
layout and semantic profile, checked/count fields, bounded issues, and skipped quick checks. A missing
remote project is a typed unsuccessful report, not a local-path or transport traceback. Rust clients
accept the same report shape from the Python HTTP fallback.

Remote `list` and `request-list` accept `--limit 1..1000` and `--cursor`. Paginated results include a
`codefire.page.v1` object in `data.page` with `returned`, `total`, `snapshot`, and nullable
`next_cursor`. The bounded `codefire.page_cursor.v1` binds collection, snapshot, offset, and limit;
cross-collection reuse, changed limit, malformed/future cursor, or collection drift fails closed.
Without pagination options, the historical full-result shape remains available.

`storage report --json` data:

```json
{
  "type": "codefire_storage_report",
  "version": 1,
  "mode": "full",
  "skipped_checks": [],
  "coverage": {
    "object_json_validation": true,
    "object_type_breakdown": true,
    "external_artifact_refs": true,
    "remote_layout_validation": true,
    "largest_object_type_source": "object_record",
    "requires_full_for": []
  },
  "objects": {
    "files": 12,
    "bytes": 4096,
    "scan": {
      "mode": "full",
      "elapsed_ms": 3,
      "files_seen": 12,
      "bytes_seen": 4096,
      "json_records_parsed": 12,
      "invalid_records": 0
    },
    "by_type": [{"type": "commit", "files": 1, "bytes": 512}],
    "largest_limit": 10,
    "largest": [{"object_id": "CF-COMMIT-...", "type": "commit", "type_confidence": "object_record", "path": "/repo/.codefire/objects/commits/CF-COMMIT-....json", "bytes": 512}]
  },
  "active_state": {"files": 1, "bytes": 28},
  "idempotency": {"files": 0, "bytes": 0},
  "external_artifacts": {"refs": 0, "referenced_bytes": 0, "payload_bytes_stored": 0},
  "remotes": [
    {
      "url": "cf:///srv/codefire/org/app",
      "status": "ok",
      "project_root": "/srv/codefire/.codefire-server/projects/org/app",
      "missing_required_paths": [],
      "objects": {"files": 20, "bytes": 8192},
      "branches": {"files": 2, "bytes": 256},
      "merge_requests": {"files": 1, "bytes": 512},
      "idempotency": {"files": 0, "bytes": 0},
      "retention": {"retention_seconds": 86400, "retention_generations": 2, "current_generation": 7},
      "objects_by_generation": [{"generation": 7, "files": 20, "bytes": 8192}]
    }
  ],
  "warnings": []
}
```

`review-pack --json` returns metadata only:

```json
{
  "type": "codefire_review_pack_result",
  "version": 1,
  "output_path": "review-pack.json",
  "bytes": 12000,
  "included_sections": ["base", "source", "options", "file_diff", "semantic_diff", "verification", "next_actions"],
  "payload_in_envelope": false
}
```

`patch export --json` returns metadata only. Large write content is omitted by default when it exceeds the configured caps:

```json
{
  "type": "codefire_patch_export_result",
  "version": 1,
  "output_path": "change.cfpatch.json",
  "summary": {"entries": 3, "omitted_entries": 1, "max_file_bytes": 1048576, "max_payload_bytes": 4194304},
  "omissions": [{"path": "model.bin", "reason": "file_exceeds_max_file_bytes", "bytes": 8388608, "sha256": "..."}],
  "payload_in_envelope": false
}
```

`evidence add --json` data:

```json
{
  "type": "codefire_evidence_add_result",
  "version": 1,
  "evidence_id": "CF-EVIDENCE-...",
  "artifact_ref_id": "CF-ARTIFACT-...",
  "command_exit_code": 0,
  "command_timed_out": false,
  "command_cwd": "/workspace/project",
  "command_cwd_source": "caller_cwd",
  "repo_relative_command_cwd": null
}
```

`link --batch <file> --json` data uses `type=codefire_link_batch_result`, includes `dry_run`, `item_count`, `links_file`, `valid`, bounded `diagnostics`, `diagnostic_total`, `diagnostics_omitted`, a `codefire_operation_plan`, and bounded applied-link output. `--dry-run` validates Atom references and duplicate links without modifying `codefire.links.yaml`. Detail diagnostics are capped at 100; a `diagnostics_omitted` summary reports additional issues. `data.plan.item_count` is the full planned count, `plan.items` contains at most 100 projections, and `plan.items_omitted` reports the remainder. Live success returns the full count in `data.added_link_count`, at most 100 projections in `data.added_links`, and the remainder in `data.added_links_omitted`. Endpoint/link `from`/`to`/`type` samples are UTF-8-safe and capped at 256 bytes per field with `*_truncated` and `*_omitted_bytes` metadata, so neither plan nor apply result echoes an unbounded batch string.

Mutating commands that still expose legacy operation/merge plan payloads return the same plan under `data.plan` in a `codefire.command_result.v1` envelope with `data.type=codefire_plan_result`. This applies to `open`, `clone`, `extinguish` variants, `upload`, `request-merge`, `request-review`, `request-apply`, and merge apply/planning. `merge --continue --json` instead returns `data.type=codefire_merge_continue_result` with `blocked`, `applied`, `already_resolved`, typed conflicts, and validation blockers; blocked output is one envelope with exit 15. `patch import --json` returns its patch import result directly as envelope `data`.

`commit --json` returns typed `data.type=codefire_commit_result` rather than the legacy generic plan result. Dry-run, blocked live, and applied results include `branch`, `open_dir`, `changed_atom_count`, bounded `changed_atoms`, `changed_atoms_omitted`, `verification`, and the operation `plan`. A blocked dry-run or live attempt is still one JSON document: top-level `ok=false`, the typed verification exit code, `data.blocked=true`, `data.applied=false`, and a verify/repair next action. Applied commit results have top-level `repo` populated from execution context and do not return apply-only next actions.

Live commit recovery state is not embedded in the successful result. If signature policy requires a key and none is available, the command fails before any immutable write and leaves a `phase=started`, zero-object marker. Otherwise the writer computes every manifest-blob, seven-root, candidate-commit, and next-branch-snapshot ID before the first store, then publishes ordered `objects[]` entries containing `role`, nullable `path`, `object_type`, `object_id`, and `state=planned`. Each exact-ID store changes only its role entry to `stored`; duplicate blob IDs on different paths remain separate entries. Candidate validation precedes branch-snapshot storage, and both precede mutable branch-ref publication. `doctor --json` reports a survivor as `kind=pending_commit_transaction` with its `path` and a message containing phase plus stored/planned role-entry counts. Entry details and candidate alias remain in the marker rather than separate doctor issue fields; legacy string entries count as stored.

`fire --json` data uses `type=codefire_fire_result`, includes `dry_run`, `item_count`, `branch`, `open_dir`, a `codefire_operation_plan`, and applied manual fires. `fire --batch <file> --json` uses `type=codefire_fire_batch_result` with the same shape and all-item validation. `--dry-run` validates Atom references and duplicate fire keys without modifying `fires.json` or branch state.

Manual fire identity uses the same digest family as scan fires: `display_id` is `FIRE-<12 uppercase hex>` and `fire_uid` is `fire_sha256_<32 lowercase hex>`. The semantic key excludes the base commit; the current base commit is reported separately in `plan.current_base_commit`.

Fire operation plans are bounded by default. `plan.item_count` is the full count, `plan.items` and `plan.sample_items` contain up to `plan.sample_limit` entries, `plan.items_omitted` reports omitted entries, and `plan.truncated` is true when the default sample is incomplete. Use `--full` when complete item detail is required. Plan `next_actions` include `--path <open_dir>` commands and targets with `branch`, `open_dir`, and `repo_root`.

`evidence add --batch <file> --json` data uses `type=codefire_evidence_batch_result`, includes `dry_run`, `item_count`, a `codefire_operation_plan`, and per-item evidence results when applied.

Evidence capture stores command output as a sealed `evidence` object and external artifact metadata as an `artifact_ref` object. Artifact payload bytes are not copied into `.codefire/objects`; storage report exposes their referenced bytes separately from stored payload bytes.

`extinguish --evidence-ref <CF-EVIDENCE-...>` links a resolution to sealed evidence through `resolution.evidence_refs`. `verify` reports `missing_evidence_refs` when a referenced evidence object is absent or not an evidence object. Remote object graph copy follows `resolution_ledger -> evidence_refs -> evidence -> artifact_ref` so evidence-linked resolutions survive upload/clone/request workflows.

`explain --json` data:

```json
{
  "type": "codefire_explain",
  "version": 1,
  "target": {"kind": "fire", "value": "FIRE-001"},
  "summary": "FIRE-001 links REQ-session to DES-session because atom_changed",
  "fire": {
    "display_id": "FIRE-001",
    "source_atom": "REQ-session",
    "target_atom": "DES-session",
    "reason": "atom_changed"
  }
}
```

Supported explain targets are `fire <id>`, `atom <id>`, `verify-failure`, and `storage-warning`. The command is read-only and returns diagnostics plus next_actions appropriate to the target.

`migrate check --json` / `migrate dry-run --json` / `migrate apply --json` data:

```json
{
  "type": "codefire_migration_report",
  "version": 1,
  "mode": "check",
  "target_format": "current",
  "target_format_semantics": "compatibility_profile_only",
  "current_format": "current",
  "supported_target_formats": ["current", "v0.6"],
  "repository_version": 1,
  "compatible": true,
  "scan_mode": "full",
  "skipped_checks": [],
  "checked_objects": 12,
  "checked_branches": 1,
  "blockers": [],
  "warnings": [],
  "planned_actions": [{"kind": "create_directory", "path": "/repo/.codefire/idempotency", "description": "create .codefire/idempotency"}],
  "applied_actions": [],
  "skipped_actions": [],
  "remaining_actions": [{"kind": "create_directory", "path": "/repo/.codefire/idempotency", "description": "create .codefire/idempotency"}],
  "applied_count": 0,
  "skipped_count": 0,
  "remaining_count": 1,
  "would_write": false,
  "did_write": false,
  "format_write_supported": false,
  "format_write_performed": false
}
```

`migrate check --quick --json` skips object record integrity scanning, sets `scan_mode:"quick"`, returns `checked_objects:0`, and lists `object_record_integrity` in `skipped_checks`.

`migrate dry-run` sets `would_write=true` when a compatible repository has registered auto-create
actions and returns `apply_migration_plan`. `migrate apply` requires a full scan, acquires the shared
repository lock, reloads the plan, and applies only those repo-contained actions through retained
no-follow parent capabilities. A successful exact replay has zero applied/remaining actions and
`did_write=false`; operational `repo.lock` owner diagnostics are not counted as migration writes.
`current` and `v0.6` are compatibility-profile selectors only: neither requests a repository/object
format rewrite. A missing branch-head semantic profile is returned as the non-blocking
`legacy_semantic_validation_profile` warning.

When compatibility or apply precondition blockers exist, migrate JSON sets `ok=false` and
`exit_code=40`. Initial compatibility blockers are returned before lock bootstrap or layout writes.

`context --json` data:

```json
{
  "type": "codefire_context_pack",
  "version": 1,
  "selector": {"kind": "atom", "value": "REQ-session", "depth": 1},
  "branch": {
    "name": "main",
    "open_dir": "/workspace/example-open",
    "base_commit": "CF-COMMIT-..."
  },
  "scan": {
    "branch_state": "open-burning",
    "snapshot_source": "active_scan",
    "active_scan_path": "/workspace/repo/.codefire/active/main/scan.json",
    "preview_recomputed": false,
    "fire_source": "active",
    "changed_atoms": ["REQ-session"],
    "open_fires": []
  },
  "atoms": [],
  "trace_links": [],
  "fires": []
}
```

Context selectors:

```text
--branch: branch/open-directory state summary
--changed: changed Atom set and directly related trace/fire context
--atom <atom-id> [--depth <n>]: selected Atom neighborhood through TraceGraph links
--fire <fire-id>: selected fire, source/target atoms, trace path, and matching links
```

`scan.snapshot_source` is `active_scan` when an active `scan.json` was read. If no active scan exists, it is `preview_recomputed`, `preview_recomputed` is true, and `fire_source` is `preview`; context remains read-only in both cases.

Mutating command dry-runs use `codefire_operation_plan` data when available:

```json
{
  "type": "codefire_operation_plan",
  "version": 1,
  "command": "commit",
  "dry_run": true,
  "would_apply": false,
  "operations": [],
  "next_actions": []
}
```

Current Rust operation plan coverage:

```text
commit --dry-run --json
extinguish --dry-run --json
extinguish --batch <file> --dry-run --json
open --dry-run --json
clone --dry-run --json
upload --dry-run --json
merge --dry-run --json
patch import --dry-run --json
request-merge --dry-run --json
request-review --dry-run --json
request-apply --dry-run --json
```

## Diagnostics

Diagnostics are objects with at least:

```text
kind
severity
```

Selected current diagnostic kinds include:

```text
open_fire
open_required_fires
missing_required_link
stale_resolution
duplicate_atom_id
failed_check
trace_graph_endpoint_integrity
open_target_resolution_failed
object_id_collision
object_not_found
sealed_commit_invalid
lock_contention
```

`trace_graph_endpoint_integrity` is blocking and uses exit code 21. It carries `repo`, `path`, `total_issues`, `issues_omitted`, and at most 100 typed `issues`; each endpoint issue distinguishes `missing_atom` from `ambiguous_atom_id` and carries bounded IDs plus omitted-byte metadata.

`open_target_resolution_failed` carries `reason`, `repo`, `branch`, `requested_path`, nullable `resolved_path`, and `workspace_boundary`. Automation should follow its `choose_safe_open_target` next action rather than retrying the same path. `object_id_collision` carries requested/existing type and full hashes plus `canonical_payload_matches`; `sealed_commit_invalid` and `object_not_found` retain repo/path context when the caller has it.

Doctor transaction findings live under doctor result issues. `pending_state_transaction` exposes path and a message with operation/phase/intended state; an unsupported or malformed marker is `invalid_active_state_shape`, not a valid pending transaction. Retry validates the marker before its first recovery write. Adjacent phase transitions are enforced by the writer; a doctor issue reports only the persisted current phase and does not prove the marker's historical transition sequence.

Each verification diagnostic has a stable `severity` and `blocking` flag. Automation should use `blocking=true` for commit-blocker filtering instead of assuming every verification diagnostic is an error.

`next_actions` is present in every envelope. Empty arrays mean no action is currently suggested for that command result.
