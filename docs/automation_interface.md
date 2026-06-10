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
| 15 | merge conflict blocks operation | reserved for blocking operation plans |
| 20 | repository corruption detected | invalid marker/repository JSON or structural corruption |
| 21 | object hash or object reference invalid | object id/hash/reference validation failures, including missing evidence refs in `verify` |
| 22 | sealed commit validation failed | sealed commit graph/certificate validation failures |
| 30 | lock contention | local repository/resource lock already held or `--lock-timeout` elapsed |
| 31 | remote rejected request | HTTP remote error response |
| 32 | authentication or signature failure | reserved for signature enforcement |
| 33 | idempotency conflict | same idempotency key reused with a different payload |
| 40 | migration incompatibility | reserved for migration checks |
| 50 | external artifact missing or hash mismatch | reserved for artifact validation |

When multiple `verify` blockers are present, CodeFire reports the first blocker class in this order: open fires, missing required links, stale resolutions, missing evidence refs, duplicate Atom IDs, failed verification commands.

Mutating commands that acquire repository or remote resource locks accept `--wait-lock` and `--lock-timeout <duration>`.
`--lock-timeout` accepts seconds (`2`, `2s`) or milliseconds (`250ms`) and returns exit code 30 when elapsed. If the lock file contains `pid` or `created_at`, human diagnostics include that owner metadata.
When a remote mutator is run with `--json`, lock contention prints a `codefire.command_result.v1` envelope with a `lock_contention` diagnostic and `retry_with_wait_lock` next action before exiting with code 30.

Idempotency keys are implemented incrementally for mutating commands:

```text
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
  "open_fire_count": 0,
  "changed_atoms_omitted": 0,
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
  "open_fires": []
}
```

Use `scan --json --full` only when the complete changed atom/open fire arrays are needed. Default JSON is bounded for large initial imports.

When `--metrics` is present on `status`, `scan`, or `verify`, the command data includes:

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
  "trace_completeness_required": true,
  "open_required_fires": 1,
  "missing_required_links": [],
  "stale_resolutions": [],
  "missing_evidence_refs": [],
  "duplicate_atom_ids": [],
  "failed_checks": [],
  "diagnostic_filter": "blocking_only"
}
```

`verify --json` sets `exit_code` to the matching blocker code from the exit code taxonomy. For example, missing required links produce code `11`, missing evidence refs produce code `21`, and failed configured verification commands produce code `14`.
`diagnostic_filter` is `all` by default and `blocking_only` when `--blocking-only` is present. Verification diagnostics include `blocking`; `missing_required_link` is `severity=warning` and `blocking=false` when `trace_completeness_required` is false.

`status --json`, `scan --json`, `verify --json`, `doctor --json`, `storage report --json`, and `migrate ... --json` include remediation-oriented `next_actions`:

```text
status: verify, scan, context_changed, commit
scan: context_changed, context_fire, extinguish_fire, verify
verify: context_changed, scan, context_atom, refresh_resolution, repair_evidence_ref, rerun_check, commit
doctor: inspect_object_store_corruption, repair_branch_head, reopen_branch_workspace, inspect_doctor_report
storage report: inspect_storage_warnings
migrate: review_migration_plan, inspect_migration_blockers
```

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

`link --batch <file> --json` data uses `type=codefire_link_batch_result`, includes `dry_run`, `item_count`, `links_file`, a `codefire_operation_plan`, and applied `added_links`. `--dry-run` validates Atom references and duplicate links without modifying `codefire.links.yaml`.

Mutating commands that still expose legacy operation/merge plan payloads return the same plan under `data.plan` in a `codefire.command_result.v1` envelope with `data.type=codefire_plan_result`. This applies to `open`, `clone`, `commit`, `extinguish` variants, `upload`, `request-merge`, `request-review`, `request-apply`, and `merge`. `patch import --json` returns its patch import result directly as envelope `data`.

`fire --json` data uses `type=codefire_fire_result`, includes `dry_run`, `item_count`, `branch`, `open_dir`, a `codefire_operation_plan`, and applied manual fires. `fire --batch <file> --json` uses `type=codefire_fire_batch_result` with the same shape and all-item validation. `--dry-run` validates Atom references and duplicate fire keys without modifying `fires.json` or branch state.

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

`migrate check --json` / `migrate dry-run --json` data:

```json
{
  "type": "codefire_migration_report",
  "version": 1,
  "mode": "check",
  "target_format": "current",
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
  "would_write": false
}
```

`migrate check --quick --json` skips object record integrity scanning, sets `scan_mode:"quick"`, returns `checked_objects:0`, and lists `object_record_integrity` in `skipped_checks`.

When compatibility blockers exist, `migrate check --json` sets `ok=false` and `exit_code=40`.

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

Current diagnostic kinds:

```text
open_fire
open_required_fires
missing_required_link
stale_resolution
duplicate_atom_id
failed_check
```

Each verification diagnostic has a stable `severity` and `blocking` flag. Automation should use `blocking=true` for commit-blocker filtering instead of assuming every verification diagnostic is an error.

`next_actions` is present in every envelope. Empty arrays mean no action is currently suggested for that command result.
