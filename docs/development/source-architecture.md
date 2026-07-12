# CodeFire Source Architecture

Status: canonical

Owner: Engineering Steward

Update trigger: crate/module ownership, command execution boundary, persistence layout, object kind, or a cross-cutting invariant changes.

This document explains stable source ownership and data flow using files and symbols, not line numbers. Source and tests remain authoritative for exact implementation. Run `python3 docs/development/tools/check_source_docs.py` after changing modules, command registration, or object kinds.

## Crate Boundaries

| Crate | Owns | Must not own |
|---|---|---|
| `codefire-core` | Atom extraction, Trace Graph and required-link policy, Fires, resolutions, scan and verification domain values | filesystem repository layout, CLI rendering, remote transport |
| `codefire-store` | canonical bytes, full digest/object ID, immutable object records, object lookup, sealed-commit validation | open-workflow state, command parsing, policy UX |
| `codefire-cli` | parse/validate/plan/apply/render orchestration, repository/open state, automation JSON, remote/HTTP, recovery tools | alternate object identity or duplicated core verification rules |
| `codefire-util` | small shared primitives such as SHA-256 | domain policy and command workflows |

Dependency direction is CLI → core/store/util and core/store → util. Core and store do not depend on CLI.

## Five Connected Machines

1. **Workspace machine**: working files → `AtomIndex` → `TraceGraph` → changed Atoms/Fires → resolutions → verification.
2. **Sealing machine**: verified workspace → immutable root objects → signed/unsigned commit object → branch-head publish. Sealed-root validation exists in the store, but the current local commit publish path does not re-run it after storing the new commit.
3. **Repository machine**: branch/opened/active/lock/idempotency metadata → lifecycle state and recovery diagnostics.
4. **Remote machine**: URL/auth/signature/policy → object graph transfer → branch or merge-request mutation → audit/result.
5. **Automation machine**: parser → validated options → operation plan → apply → text or `codefire.command_result.v1` envelope.

No machine may infer a stronger claim than its inputs. In particular, an automation envelope renders the domain result; it does not decide that result.

## CLI Module Ownership

`main.rs` owns top-level dispatch and workflows that have not yet moved behind focused modules. New responsibilities belong in a module unless coupling to the orchestration transaction is essential.

| Module | Stable responsibility |
|---|---|
| `automation` | `codefire.command_result.v1`, diagnostics, bounded `next_actions`, state/scan/verification JSON. | <!-- module: automation -->
| `batch` | extinguish batch parsing, validation, planning, and rendering. | <!-- module: batch -->
| `cli_model` | shared command option/result types; no filesystem behavior. | <!-- module: cli_model -->
| `command_registry` | declared public command capabilities used by help/completion/metrics/capability output. | <!-- module: command_registry -->
| `completion` | top-level help and shell completion rendering. | <!-- module: completion -->
| `context` | bounded read-only automation context packs. | <!-- module: context -->
| `doctor` | repository integrity diagnosis and repair guidance. | <!-- module: doctor -->
| `evidence` | evidence/artifact capture, batch validation, and result rendering. | <!-- module: evidence -->
| `exit_code` | stable error-to-exit classification. | <!-- module: exit_code -->
| `explain` | read-only explanation of Fire/Atom/verification/storage targets. | <!-- module: explain -->
| `extinguish_ux` | interactive and recent-evidence input assistance. | <!-- module: extinguish_ux -->
| `fire` | manual/batch Fire validation and plan/apply behavior. | <!-- module: fire -->
| `http` | HTTP client/server framing, route dispatch, size/time boundaries. | <!-- module: http -->
| `http_tls` | TLS transport configuration and key/certificate loading. | <!-- module: http_tls -->
| `idempotency` | common idempotency record identity/conflict helpers. | <!-- module: idempotency -->
| `import_workflow` | importing an existing directory into a new native open branch. | <!-- module: import_workflow -->
| `limited_yaml` | intentionally bounded YAML-like scalar/key parsing shared by batch inputs. | <!-- module: limited_yaml -->
| `link_batch` | Trace Link batch parsing, full validation, plan, and apply. | <!-- module: link_batch -->
| `merge_patch_idempotency` | merge and patch-import replay/conflict records. | <!-- module: merge_patch_idempotency -->
| `metrics` | phase/counter values and attachment under `data.metrics`. | <!-- module: metrics -->
| `migration` | compatibility inspection and migration planning. | <!-- module: migration -->
| `open_clone_idempotency` | open/clone replay and conflict records. | <!-- module: open_clone_idempotency -->
| `remote` | file/HTTP remote models, object transfer, merge-request operations. | <!-- module: remote -->
| `repo_layout` | required versus auto-created repository directories shared by doctor/migrate. | <!-- module: repo_layout -->
| `signatures` | commit/request signing, verification, key lifecycle, nonce/replay policy. | <!-- module: signatures -->
| `storage` | bounded local/remote storage report and warnings. | <!-- module: storage -->
| `tests` | CLI-wide regression module compiled only for tests; focused module tests should stay near their owners. | <!-- module: tests -->
| `verification` | verify argument parsing plus text rendering of a core `Verification`. | <!-- module: verification -->
| `view` | show/diff/review-pack/patch views, algorithms, bounded payload handling. | <!-- module: view -->

## Core Domain Flow

`codefire-core` exposes the domain stages:

```text
build_atom_index(open_dir)
  -> AtomIndex { version, hash_schema_version, atoms, duplicates }

current_trace_graph(open_dir)
  -> parse link syntax and construct typed links
  -> TraceGraph

build_scan_result_with_options(current, base, trace, policy, prior fires, ...)
  -> changed_atoms + non_atom_changed_files + tool_migration + open_fires

build_verification(scan, resolutions, policy, failed commands, evidence refs)
  -> Verification { result, blocker collections }

build_resolution(request, current atoms/links/policy)
  -> Resolution + content/link/policy basis
```

`Verification.result` and its blocker collections are the domain truth. CLI exit codes, certificate counts, diagnostics, and JSON must be derived from the same value.

## Store and Identity Flow

```text
payload
  -> canonical_json(payload)
  -> SHA-256(type_tag || NUL || canonical_bytes)
  -> full digest
  -> public object ID using the current 24-hex prefix
  -> ObjectRecord { object_id, type, hash, payload }
  -> immutable object subdirectory
```

Ordinary object readers validate each stored record's internal agreement among payload, full digest, public/legacy ID, and filename. `validate_sealed_commit` adds commit/root type, shape, certificate, and reachable-reference checks when a sealed subject is consumed. The required rule is that a public prefix never replaces comparison of the existing and requested full digests. Current `store_object` reuse validates only the existing record and does not compare it with a different newly requested record that shares the prefix; CFB-125 tracks both reuse branches.

## Open Workspace Flow

`open` validates the sealed branch head and closed branch state, materializes the commit, then creates:

- `.codefire-open` in the opened directory with repository, branch, and open-instance identity;
- `.codefire/opened/<encoded-branch>.json` with path/base/active-state linkage;
- `.codefire/active/<open-instance>/state.json` and later scan/fire/resolution/verification state;
- branch state `open-clean`.

The common `open_context` currently cross-checks the marker's opened path and open-instance ID against the opened registry. Individual lifecycle commands add branch-state or sealed-head checks. Full branch/opened/marker/active identity agreement is a required Stage 1 invariant, not one centralized check in the current implementation; never treat one file as sufficient authority after corruption.

## Scan and Verify Flow

`run_scan_command`/`compute_scan` resolve the open context, read the base commit and Atom index, build the current index/graph, compare the base, and derive Fires and non-Atom changes. Unlike `status` and several lifecycle paths, `compute_scan` does not currently call `validate_sealed_commit` for that base. Live scan writes `fires.json`, `scan.json`, opened registry state, active state, and branch state through individually atomic replacements; the sequence is not a multi-file transaction. Direct production scan does not acquire `RepoLock`. `run_verify_command` → `compute_verify` builds on scan, loads resolutions, executes constrained verification commands, builds one core `Verification`, persists its result, and updates derived state; direct production verify also lacks that lock. Nested commit/batch paths may already hold it, so Stage 1 must make one lock owner and a separate read-only compute boundary explicit (CFB-124).

Read-only preview paths must not write scan state. Dry-run and live mutations must share validation/plan semantics.

## Commit Transaction Flow

Production `run_commit` currently constructs the commit payload directly; it does not call the store test/helper `commit_payload`.

```text
open_context + scan + verification + active fires/resolutions + policy
  -> operation plan
  -> reject or dry-run if not applicable
  -> commit_transaction.json(status=pending, phase=started)
  -> store content_manifest
  -> store atom_index, trace_graph, fire_ledger, resolution_ledger
  -> store verification and policy
  -> construct root_schema_version=2 commit + certificate
  -> optional signature
  -> store commit object (current path does not call validate_sealed_commit here)
  -> publish branch head and state
  -> update opened registry base/state
  -> reset active state
  -> remove commit_transaction.json
  -> persist idempotency result
```

Immutable objects written before publication may be orphaned safely, but a pending marker must let doctor/recovery explain interrupted work. Branch-head publish is the candidate visibility point; Stage 1 must add/confirm publish-time sealed validation and use failpoints to establish every crash window before this is treated as a proven atomic transaction.

## Repository Layout Ownership

`repo_layout::REPO_DIRS` is the source of truth shared by doctor and migration:

- required: `objects`, `branches`, `opened`, `active`;
- auto-created: `cache`, `locks`, `remotes`, `idempotency`;
- object subdirectories come from the store registry; `artifact_refs` and `evidence` are auto-created while other current object subdirectories are required.

Additional state includes `repo.json`, encoded branch/opened records, active state files, locks, remotes, and idempotency records. Exact shapes belong in `source-data-model-catalog.md`.

## Required Invariants and Current Gaps

- Canonical bytes/full digest/public ID must agree across every writer and reader. Current object-store reuse lacks the existing-versus-requested full-digest comparison for an artificial same-prefix collision (CFB-125).
- A branch head must name a valid sealed commit and remote mutation must not weaken this. Many read/lifecycle/remote paths enforce it; scan base loading and local post-store commit publication still need the explicit Stage 1 proof/fix described above.
- Validation happens before plan application; batch inputs validate all items before the first write.
- Branch/opened/open-marker/active identities must agree or the command must return structured corruption/recovery guidance. Current checks are distributed and do not yet prove this complete matrix.
- Mutable metadata and open-state mutations must acquire the relevant lock and use collision-resistant temporary writes plus replace/sync behavior. Direct live scan/verify currently persist several open-state records without `RepoLock` (CFB-124). Some immutable content-addressed evidence writes intentionally rely on store publication rather than a repository-wide lock.
- Idempotency keys must replay the same payload/result and reject a different payload. Current import markers bind only key/result and can replay a different branch request without conflict (CFB-126); other inspected mutating paths use payload-bound helpers.
- JSON success/failure uses the stable envelope and exit taxonomy; bounded output reports omission.
- Root/path/symlink containment must be rechecked at the use boundary. Current `absolute_path` and open-target checks are lexical and do not resolve every existing symlink/`..` case; Stage 1 path red-team and a canonical resolver are required.
- Verification command execution obeys timeout/output/environment policy; lack of OS isolation remains an explicit trust boundary.
- AI orchestration remains outside these modules.

## Review Hotspots

Treat changes to store identity, `run_commit`, branch/open lifecycle, remote mutation, signatures, path helpers, automation envelopes, or repository layout as Ultra review. Require focused regression evidence before refactoring a semantic path. Use `change-playbook.md` for read sets and validation selection.
