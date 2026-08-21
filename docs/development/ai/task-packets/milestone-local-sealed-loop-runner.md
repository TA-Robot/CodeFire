# AI Task Packet

Status: ready

Packet ID: MS-local-sealed-loop-runner

Schema version: human-v1

Base revision: the commit that closed `MS-2026-08-control-restore` on 2026-08-21

Risk score: 3

Reasoning tier: high

Parent packet: `milestone-local-sealed-loop.md`. This is node 1 of that graph — one of two parallel ready slices.

Nothing here has run. No runner has been chosen. Do not read any statement below as an observation.

## Objective

Decide which `codefire` executable runs the M1 local sealed loop, and record it in this file: path, provenance, and capability outcome. If no such executable can be obtained inside M1 scope, record an exact blocker instead.

This node answers *what runs the loop*. It does not answer what the loop carries — that is node 2, running concurrently.

## Context

- Required context pack(s): pack-0-universal-orientation
- Additional files: `docs/development/milestones.md`, `docs/development/known-limitations.md`, `docs/development/README.md`
- Known constraints to check, not to assume: the base environment has been recorded as exposing no `cargo` on `PATH`, and the only recorded source-matching binary is the July artifact `/tmp/codefire-cfr167-final.KWa8TC/bin/codefire` with SHA-256 `1483874b4c1de4a78478595d5d158cf34a12e5ec3b05255dfc8819f511f00272`. A `/tmp` path is not durable across sessions. Verify current reality; do not restate these as findings.

## Plan

1. Determine which `codefire` executables are actually reachable now.
2. For each candidate, record path and provenance — how it was produced and whether its identity is known.
3. Record the capability outcome: can it run `status` / `scan` / `extinguish` / `verify` / `commit` against the dogfood project at all, on a read-only basis where possible.
4. Name the one runner node 3 should use, or record an exact blocker.

A recorded blocker is a legitimate and complete result for this node. Report it as `block-m1` and stop.

## Scope

Allowed files:

- `docs/development/ai/task-packets/milestone-local-sealed-loop-runner.md` — this file, and nothing else

Disallowed files:

- `crates/**`, `codefire`, `tests/**`, `install.sh`, `pyproject.toml`, `setup.py`, `adr/**`
- `subprojects/**` — this node makes no product or dogfood edit
- CodeFire project state; this node must not mutate any repository CodeFire manages
- `docs/development/ai/task-packets/milestone-local-sealed-loop-increment.md` — node 2 owns it
- `docs/development/milestones.md` — the orchestrator owns the board
- the classified dirty tree in `/workspace/project`

## Non-Goals

- Building a release pipeline, pinning a toolchain, or establishing CI identity; those are M5 release-slot concerns.
- Fixing `install.sh` or packaging.
- Compiling the workspace to manufacture a binary if `cargo` is genuinely unavailable — record the blocker instead.
- Selecting the increment, editing product code, or running the loop.
- Starting any parked root.

## Invariants

- no product or dogfood file is modified by this node;
- no CodeFire-managed repository is mutated — probing is read-only, and the separately configured legacy repository is not touched at all;
- path/workspace safety: work stays inside the configured workspace;
- an executable whose provenance cannot be established is recorded as unknown provenance, not silently accepted as the runner.

## Validation

This node changes one planning file, so it runs only:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Any `codefire` invocation here is a read-only capability probe, not milestone validation. The M1 exit-gate list stays on the M1 card and is node 3's to run. Do not widen.

## Handoff

- runner path, or the exact blocker;
- provenance: how the binary was produced, and its identity if known;
- capability outcome per loop command probed, read-only;
- whether the runner is durable across sessions;
- risks node 3 inherits;
- next allowed action.

## Result

Not started. Empty by design — this section is filled by the node 1 agent.
