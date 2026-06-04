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
| 30 | lock contention | local repository/resource lock already held |
| 31 | remote rejected request | HTTP remote error response |
| 32 | authentication or signature failure | reserved for signature enforcement |
| 33 | idempotency conflict | reserved for idempotency keys |
| 40 | migration incompatibility | reserved for migration checks |
| 50 | external artifact missing or hash mismatch | reserved for artifact validation |

When multiple `verify` blockers are present, CodeFire reports the first blocker class in this order: open fires, missing required links, stale resolutions, duplicate Atom IDs, failed verification commands.

## Supported Commands

```bash
codefire status --json
codefire scan --json
codefire verify --json
codefire verify --details --json
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

`next_actions` is present in every envelope. CF-218 defines the stable location and array shape; later diagnostics tasks populate richer remediation actions across more commands.
