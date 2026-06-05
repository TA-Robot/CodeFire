# Automation Interface

CodeFire v0.6 exposes stable JSON output for state and diagnostic commands through a shared command result envelope.

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

`next_actions` entries use this shape:

```json
{
  "kind": "context_atom",
  "command": "codefire-rs context --atom REQ-session --depth 2 --json",
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
| 21 | object hash or object reference invalid | object id/hash/reference validation failures |
| 22 | sealed commit validation failed | sealed commit graph/certificate validation failures |
| 30 | lock contention | local repository/resource lock already held or `--lock-timeout` elapsed |
| 31 | remote rejected request | HTTP remote error response |
| 32 | authentication or signature failure | reserved for signature enforcement |
| 33 | idempotency conflict | same idempotency key reused with a different payload |
| 40 | migration incompatibility | reserved for migration checks |
| 50 | external artifact missing or hash mismatch | reserved for artifact validation |

When multiple `verify` blockers are present, CodeFire reports the first blocker class in this order: open fires, missing required links, stale resolutions, duplicate Atom IDs, failed verification commands.

Local mutating commands that acquire `repo.lock` accept `--wait-lock` and `--lock-timeout <duration>`.
`--lock-timeout` accepts seconds (`2`, `2s`) or milliseconds (`250ms`) and returns exit code 30 when elapsed. If the lock file contains `pid` or `created_at`, human diagnostics include that owner metadata.

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
  "changed_atoms": ["REQ-session"],
  "open_fires": []
}
```

When `--metrics` is present on `status`, `scan`, or `verify`, the command data includes:

```json
{
  "metrics": {
    "type": "codefire_metrics",
    "version": 1,
    "command": "scan",
    "phase_timings": {"total_ms": 12},
    "counters": [{"name": "atoms", "value": 24}],
    "cache": {"enabled": false}
  }
}
```

`verify --json` data:

```json
{
  "result": "failed",
  "open_required_fires": 1,
  "missing_required_links": [],
  "stale_resolutions": [],
  "duplicate_atom_ids": [],
  "failed_checks": []
}
```

`verify --json` sets `exit_code` to the matching blocker code from the exit code taxonomy. For example, missing required links produce code `11`; failed configured verification commands produce code `14`.

`status --json`, `scan --json`, `verify --json`, and `storage report --json` include remediation-oriented `next_actions`:

```text
status: verify, scan, context_changed, commit
scan: context_changed, context_fire, extinguish_fire, verify
verify: context_changed, scan, context_atom, refresh_resolution, rerun_check, commit
storage report: inspect_storage_warnings
migrate: review_migration_plan, inspect_migration_blockers
```

Rust `doctor` is still planned separately; its `next_actions` are added when that command surface lands.

`storage report --json` data:

```json
{
  "type": "codefire_storage_report",
  "version": 1,
  "objects": {
    "files": 12,
    "bytes": 4096,
    "by_type": [{"type": "commit", "files": 1, "bytes": 512}],
    "largest": [{"object_id": "CF-COMMIT-...", "type": "commit", "path": "/repo/.codefire/objects/commits/CF-COMMIT-....json", "bytes": 512}]
  },
  "active_state": {"files": 1, "bytes": 28},
  "idempotency": {"files": 0, "bytes": 0},
  "external_artifacts": {"refs": 0, "referenced_bytes": 0, "payload_bytes_stored": 0},
  "warnings": []
}
```

`evidence add --json` data:

```json
{
  "type": "codefire_evidence_add_result",
  "version": 1,
  "evidence_id": "CF-EVIDENCE-...",
  "artifact_ref_id": "CF-ARTIFACT-...",
  "command_exit_code": 0
}
```

Evidence capture stores command output as a sealed `evidence` object and external artifact metadata as an `artifact_ref` object. Artifact payload bytes are not copied into `.codefire/objects`; storage report exposes their referenced bytes separately from stored payload bytes.

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
  "target_format": "v0.6",
  "repository_version": 1,
  "compatible": true,
  "checked_objects": 12,
  "checked_branches": 1,
  "blockers": [],
  "warnings": [],
  "planned_actions": [{"kind": "create_directory", "path": "/repo/.codefire/idempotency", "description": "create .codefire/idempotency"}],
  "would_write": false
}
```

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

`next_actions` is present in every envelope. Empty arrays mean no action is currently suggested for that command result.
