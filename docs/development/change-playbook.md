# CodeFire Change Playbook

Status: canonical

Owner: Engineering Steward

Update trigger: a common change path, validation command, ownership boundary, or invariant checklist changes.

This is the task-oriented entry point for changing CodeFire. Architecture belongs in `source-architecture.md`, exact command routing in `command-source-trace.md`, and persisted/type contracts in `source-data-model-catalog.md`.

## Ten-Minute Orientation

1. Read `AGENTS.md`, the development hub, and the relevant strategy decision.
2. Run `git status --short --branch`; do not assume a clean tree.
3. Identify the command or domain symbol in `command-source-trace.md`.
4. Read the owning module and its focused tests before `main.rs` broadly.
5. Name the invariant and a regression that would fail if it broke.
6. Generate validation suggestions:

```bash
python3 docs/development/tools/suggest_validation.py <changed-path>...
python3 docs/development/tools/probe_validation_capabilities.py
```

7. If delegating, resolve a context pack and use a typed task packet.

## Read Routes

| Change | Read first | Then inspect |
|---|---|---|
| Atom extraction / Trace policy | `codefire-core` public structs and builders, `docs/09_index_trace_policy.md` | core tests, scan callers, hash-schema compatibility |
| Object identity / sealed commit | `codefire-store`, `docs/08_object_store.md` | commit construction, doctor/migration, compatibility fixtures |
| CLI option/help/capability | `command_registry`, `completion`, `cli_model` | parser/dispatch arm, docs tests, JSON failure path |
| State/scan/verify | `source-architecture.md`, core scan/verification | `main.rs` orchestration, `verification`, automation renderers |
| Open/clone/close/import | relevant trace row, open context helpers | idempotency module, path policy, branch/opened/active records |
| Fire/extinguish/link/evidence batch | focused module | all-items-before-write tests, operation plans, active-state update |
| Commit | commit transaction section in architecture | store validators, signatures, automation/certificate tests, doctor pending marker |
| Diff/show/review/patch/merge | `view` and its submodules | bounded output, object reads, merge/patch idempotency |
| Remote/HTTP/signatures | `remote`, `http`, `http_tls`, `signatures` | path/resource limits, nonce/key policy, structured remote errors |
| Doctor/migration/layout | `repo_layout`, `doctor`, `migration` | store object registry, recovery fixtures |
| AI development system | `ai/README.md`, schemas, model policy | strategy decision register and local AI gate |

## Recipe: Add or Change a Public Command

1. Decide whether it belongs in the product; keep AI orchestration external.
2. Add/update the shared option/result type in `cli_model` or a focused module.
3. Register the top-level capability in `command_registry`.
4. Add help/completion and ensure help returns before mutation/parser side effects.
5. Implement parse → validate → plan → apply → render; move domain logic out of dispatch.
6. Return `codefire.command_result.v1` for declared JSON support, including failures.
7. Add parser, help, text, JSON success/failure, dry-run/no-write, and path-context tests.
8. Update CLI spec, command trace, capability/source checker, and runbook if operational.

## Recipe: Change a Mutating Workflow

Write down before editing:

- validated base/head and expected state;
- lock/resource scope;
- idempotency payload and conflict key;
- full plan and no-write dry-run behavior;
- ordered writes and visibility point;
- interruption states and recovery/doctor output;
- JSON/text result and next action;
- focused failpoint/replay tests.

Never validate one batch item after earlier items have written. Never let rendering or dry-run take a weaker validation path.

## Recipe: Change Object or Format Semantics

This is mandatory Ultra work.

1. Add cross-runtime canonical/full-digest/public-ID vectors first.
2. State readable, writable, and rejected versions.
3. Preserve old field meaning; add a versioned extension or migration.
4. Test malformed payload, unknown version, legacy readability, collision, and filename/record mismatch.
5. Update store validator, doctor, migration, data catalog, object spec, release/format roadmap, and compatibility matrix together.
6. Do not infer release SemVer from an internal schema increment.

## Recipe: Optimize a Repository or Graph Command

1. Reproduce with a bounded fixture and record stable work counters.
2. Identify the repeated entity (object read, parse, hash, graph visit), not wall time alone.
3. Define the cache/memo key from every validity input.
4. Compare success and corruption diagnostics before/after.
5. Add scale points that distinguish expected complexity.
6. Reject an optimization that weakens sealed validation or hides omissions.

## Recipe: Synchronize Documentation

1. Classify the target as canonical, supporting, exploration, historical, generated, or superseded.
2. Update the one authority; do not copy current counts into stable entry pages.
3. For deletion, record unique-content disposition and inbound migration in the deletion manifest.
4. Run project docs and source docs checks.
5. Preserve old event facts in append-only history; do not leave old instructions live.

## Invariant Review Checklist

### Identity and storage

- canonical serialization and full digest unchanged or versioned;
- current and legacy public IDs handled explicitly;
- immutable records validated before use;
- commit roots and certificate agree with verification;
- no reachable object deletion or silent alias.

### Branch and workspace state

- branch head sealed;
- branch/open marker/opened registry/active state identities agree;
- open work is never silently discarded or rematerialized over;
- dry-run does not mutate;
- publish/replace is atomic enough for the supported platform contract.

### Automation

- success and failure envelopes use `schema`, `command`, `ok`, `exit_code`, `repo`, `data`, `diagnostics`, `next_actions`;
- payload-specific `type` stays inside `data`;
- metrics stay under `data.metrics`;
- bounded output reports truncation/omission;
- exit code and diagnostics describe the same failure.

### Security and recovery

- repository containment rechecked after path resolution;
- symlink/temporary-file/lock races considered;
- secrets are not captured as ordinary evidence;
- remote mutation authenticates the full target/operation and prevents replay;
- resource/command limits have cleanup behavior;
- pending/stale/corrupt states lead to safe, explicit recovery.

## Focused Validation Order

1. format/compile for the changed language;
2. the narrow regression protecting the invariant;
3. owning crate/module tests;
4. cross-crate/workspace or Python compatibility tests;
5. CLI/installer/dogfood smoke when the boundary changed;
6. docs/source/AI gates;
7. release gates only from a clean, source-bound environment.

When `cargo` is unavailable, report Rust source validation as blocked. A pre-existing binary may reproduce user behavior but is not evidence for modified Rust source.

## Handoff

Every implementation handoff names:

- objective and issue/task packet;
- files changed and intentionally untouched;
- invariant and design choice;
- focused and broad commands with exact result;
- blocked capabilities/environment;
- residual risk and next evidence.

For High/Ultra changes, use an independent reviewer bound to the task and patch subjects.
