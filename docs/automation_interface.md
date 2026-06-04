# Automation Interface

CodeFire v0.6 exposes stable JSON output for state and diagnostic commands through a shared command result envelope.

## Command Result Envelope

```json
{
  "schema": "codefire.command_result.v1",
  "command": "verify",
  "ok": false,
  "exit_code": 1,
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
