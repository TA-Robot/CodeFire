# CodeFire Source and Persisted Data Model Catalog

Status: canonical

Owner: Core and Store Maintainers

Update trigger: a Rust domain/CLI type, object kind, persisted JSON shape, version/default, or compatibility rule changes.

This catalog records current shapes at the level required for safe changes. Unless a row says otherwise, “current writer” means the Rust-first/default CLI, not the Python fallback. Exact serde/constructor behavior in source and golden tests remains authoritative; examples are not promises that every legacy record or fallback writer contains every field.

## Version Axes

| Axis | Rust-first/default writer | Compatibility note |
|---|---|---|
| Core model `VERSION` | `1` | Most core serialized structs carry `version: 1`. <!-- constant: VERSION=1 --> |
| Atom hash schema | scan/commit `AtomIndex` emits `2` | Rust init's empty Atom index and Python fallback currently omit it; missing value follows the explicit legacy/default path. <!-- constant: ATOM_HASH_SCHEMA_VERSION=2 --> |
| Repository layout | `repo.json.version = 1` | Migration validates unsupported values explicitly. |
| Commit root schema | Rust init/commit payloads emit `2` | Python fallback and legacy records may omit it; absent/default schema has fewer required roots. |
| Public object digest prefix | 24 hex | Legacy 12-hex IDs remain readable when full record hash agrees. <!-- constant: OBJECT_ID_DIGEST_HEX_LENGTH=24 --> |
| Automation envelope | Rust declared JSON commands use `codefire.command_result.v1` | Python fallback is primarily a human-text CLI and does not provide a shared `--json` surface or common envelope; isolated JSON-producing commands are separate contracts. Payload types are independently versioned inside Rust `data`. |

Product/package/SemVer is independent; see `strategy/release-and-format-roadmap.md`.

### Rust/Python Writer Boundary

| Surface | Rust-first/default CLI | Python fallback |
|---|---|---|
| New public Object ID | 24-hex digest prefix | 12-hex digest prefix |
| Atom hash schema | normal scan/commit emits 2; initial empty roots currently omit | omitted |
| Commit root schema | emits 2 | omitted |
| JSON command envelope | `codefire.command_result.v1` for declared JSON support | No shared `--json`/envelope surface; output is primarily human text, with isolated command-specific JSON outputs treated separately |

This is an observed compatibility gap, not a promise that either writer can safely rewrite every repository produced by the other. The release/format fixture matrix owns that decision.

## Core Domain Types

| Type | Essential fields / meaning |
|---|---|
| `Atom` | stable `atom_id`, kind/source location, content hash and metadata used for identity/trace. |
| `AtomIndex` | `version`, `hash_schema_version`, Atom collection, duplicate IDs; identity-producing snapshot. |
| `TraceLink` / `TraceGraph` | typed source/target relation; direct `current_trace_graph` validates syntax/shape but does not receive an Atom index, while batch mutation paths perform their own Atom-reference checks. |
| `TracePolicy` / `RequiredLinkRule` | link completeness rules and blocking behavior. |
| `VerificationPolicy` | commit policy and constrained verification commands. |
| `VerificationCommand` | `id`, command, cwd, timeout, output limit, environment/inheritance controls, and `allow_failure`; there is no `required` field on this type. |
| `Fire` | stable UID/display ID, source/target Atom refs, reason/status, trace path, creator/time, key, and optional resolution/obsolete fields. |
| `Resolution` / `ResolutionBasis` | decision/rationale/evidence refs plus source/target content, Trace Link, and policy basis. |
| `ScanResult` | current index/graph, changed Atoms, non-Atom changed files, tool migration, open Fires. |
| `Verification` | result plus open required Fires, failed checks, missing links, stale resolutions, missing evidence refs, duplicates. |

Commit eligibility must derive from one `Verification`, not independent recounts in renderers.

## CLI Boundary Types

`cli_model` owns shared options/results for init/import/open/status/branch/extinguish/commit/clone/close/diff/review/patch/serve/merge/lock. Focused modules own their specialized types, including context, doctor, evidence, explain, fire/link batch, migration, remote, storage, and view algorithms.

Options carry explicit path/JSON/dry-run/lock/idempotency fields where supported. Result types return domain identity plus an operation plan or data payload; rendering is separate.

## Object Record and Identity

Current object record:

```json
{
  "object_id": "CF-...-<24 hex>",
  "type": "<type tag>",
  "hash": "<full 64-hex SHA-256>",
  "payload": {}
}
```

Digest input is `type_tag`, one NUL byte, then the canonical JSON bytes of `payload`. `read_object_record` recomputes the full hash and current/legacy ID and validates the filename/record identity. `validate_sealed_commit` is the stronger consuming boundary that validates commit/root shapes and traverses allowed references.

## Object Kinds

| Type tag | Prefix | Subdirectory | Role |
|---|---|---|---|
| `blob` | `CF-BLOB` | `blobs` | file content. <!-- object-kind: blob --> |
| `content_manifest` | `CF-MANIFEST` | `content_manifests` | path → blob/metadata sealed snapshot. <!-- object-kind: content_manifest --> |
| `atom_index` | `CF-ATOMINDEX` | `atom_indexes` | serialized Atom index. <!-- object-kind: atom_index --> |
| `trace_graph` | `CF-TRACE` | `trace_graphs` | serialized Trace Graph. <!-- object-kind: trace_graph --> |
| `fire_ledger` | `CF-FIRELEDGER` | `fire_ledgers` | sealed Fire set/delta. <!-- object-kind: fire_ledger --> |
| `resolution_ledger` | `CF-RESOLUTION` | `resolution_ledgers` | sealed resolutions and evidence refs. <!-- object-kind: resolution_ledger --> |
| `verification` | `CF-VERIFY` | `verifications` | core verification result. <!-- object-kind: verification --> |
| `policy` | `CF-POLICY` | `policies` | policy used for the seal. <!-- object-kind: policy --> |
| `artifact_ref` | `CF-ARTIFACT` | `artifact_refs` | external/local artifact identity and metadata. <!-- object-kind: artifact_ref --> |
| `evidence` | `CF-EVIDENCE` | `evidence` | captured/referenced evidence subject. <!-- object-kind: evidence --> |
| `commit` | `CF-COMMIT` | `commits` | parent/root/certificate/history object. <!-- object-kind: commit --> |
| `branch` | `CF-BRANCH` | `branches` | object-store branch payload kind; repository branch heads also have active metadata records. <!-- object-kind: branch --> |

## Current Commit Payload

Current production `run_commit` writes:

```json
{
  "type": "commit",
  "version": 1,
  "root_schema_version": 2,
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
    "stale_resolutions": 0,
    "missing_evidence_refs": 0,
    "duplicate_atom_ids": 0
  },
  "changed_atoms": [],
  "extinguished_fires": [],
  "created_at": "...",
  "signature": {
    "type": "commit_signature",
    "version": 1,
    "algorithm": "hmac-sha256",
    "signer": "...",
    "key_id": "...",
    "signed_at": "...",
    "signature": "..."
  }
}
```

The store's sealed-commit validator checks required roots, root object types/shapes, certificate-versus-verification agreement, and parent references. Commit/request signature policy is enforced separately by the CLI `signatures` module at remote/security boundaries. The `signature` member is optional and, when present, is an object rather than a string.

## Repository Layout

```text
.codefire/
  repo.json
  objects/<object-kind-subdir>/<object-id>.json
  branches/<encoded-branch>.json
  opened/<encoded-branch>.json
  active/<open-instance>/
    state.json
    scan.json
    fires.json
    resolutions.json
    verification.json
    commit_transaction.json   # only while interrupted/in progress
  cache/
  locks/
  remotes/
  idempotency/
```

`repo_layout::REPO_DIRS` distinguishes required from auto-created directories. Object subdirectory requirements come from the store registry.

## Open Marker and Registry

Current `.codefire-open` writer shape:

```json
{
  "version": 1,
  "repository": {"path": "/repo/.codefire", "repository_id": "..."},
  "branch": {"name": "feature", "opened_from_commit": "CF-COMMIT-..."},
  "open": {"open_instance_id": "open_...", "opened_path": "/work", "opened_at": "..."}
}
```

Current repository-side opened record:

```json
{
  "version": 1,
  "branch": {"name": "feature"},
  "open": {
    "open_instance_id": "open_...",
    "path": "/work",
    "opened_from_commit": "CF-COMMIT-...",
    "current_base_commit": "CF-COMMIT-...",
    "active_state_path": "/repo/.codefire/active/open_..."
  },
  "state": {"last_known": "open-clean"}
}
```

`active/.../state.json` is currently `{"state":"open-clean|open-burning|open-consistent"}` with no `type` or `version`. Consumers must not invent fields from old documentation.

## Commit Transaction Marker

While committing, `active/.../commit_transaction.json` contains version, transaction ID, `pending` status, current phase, branch/open directory, parents, message, object IDs written so far, and timestamps. Success removes it. Doctor treats a surviving marker as interrupted work requiring diagnosis; it is not a sealed object.

## Automation Envelope

Every Rust command with declared JSON support uses this top-level shape:

```json
{
  "schema": "codefire.command_result.v1",
  "command": "scan",
  "ok": true,
  "exit_code": 0,
  "repo": "/repo-or-null",
  "data": {"type": "payload-specific", "metrics": "optional under data"},
  "diagnostics": [],
  "next_actions": []
}
```

There is no top-level payload `type`. `next_actions` are normalized and bounded; command-specific payloads/plans live under `data`.

## Compatibility Rules

- Missing/legacy Atom hash schema defaults only where source explicitly defines it. Rust scan/commit emits 2; Rust init and Python fallback currently omit it.
- Legacy 12-hex object IDs are accepted only when the record's full hash and payload recompute correctly.
- Rust current commits emit root schema 2; Python fallback and legacy root-schema behavior remain explicit compatibility work.
- Stage 2 required rule: an unknown required schema or capability must fail explicitly and must never downgrade a blocker to success.
- Current gap: version/schema rejection is not yet uniform, so some future version values can pass deserialization or default paths. That behavior is not a compatibility guarantee and must be closed and fixture-tested before claiming the Stage 2 rule.
- Rust/Python behavior and writable/readable combinations require real fixtures; see release/format roadmap.
- A source change to any marker in this file requires source-doc checker and focused product tests.
