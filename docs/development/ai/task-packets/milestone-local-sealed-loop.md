# AI Task Packet

Status: ready

Packet ID: MS-local-sealed-loop

Schema version: human-v1

Base revision: the commit that closed `MS-2026-08-control-restore` on 2026-08-21

Risk score: 5

Reasoning tier: high

This packet is a **plan**. Nothing in it has run. It carries no product evidence and no validation results; the `Evidence` cells on the live board are empty for a reason. Do not read any statement here as an observation.

## Objective

Prove the local sealed-commit loop is usable for real day-to-day work by taking **one small real change** in the canonical dogfood project all the way through `open → edit → scan → extinguish → verify → commit`, using only the loop's focused validation.

The deliverable is a sealed commit plus CodeFire's own before/after report of the project state. An agent asserting "the loop works" is not the deliverable.

## Context

- Required context pack(s): pack-0-universal-orientation, pack-6-dogfood-algorithm-project
- Additional files: `docs/development/milestones.md`, `docs/development/milestone-operating-model.md`, `docs/development/known-limitations.md`, `subprojects/algorithm-evolution-agent-lab/AGENTS.md`
- Related issue or observation: `MS-2026-08-control-restore` restored the scheduler but validated no product behavior. This is the first slot that touches product/dogfood behavior.
- Board state to respect: the canonical Algorithm project was last recorded `open-clean` at sealed bridge `CF-COMMIT-e5ca290905c2a4b5e2c05bd4` with zero Fires. That is a **prior record**, not a fact this packet has re-observed. Re-read it with `status` before assuming it.

## Plan

The live graph is on `docs/development/milestones.md` under "M1 execution graph (current)". That board is authoritative; this section is the packet-local copy.

```text
Critical path: loop runner readiness → sealed-loop run of the selected increment → loop evidence record → final independent audit → M1 exit and slot shift
Progress: nothing has run; nodes 1 and 2 are ready, nodes 3-6 are blocked
Parallel now: node 1 (loop runner readiness) and node 2 (increment selection and pre-loop baseline) — independent, disjoint write sets, both launchable immediately
Blocked: node 3 waits on 1 and 2; node 4 waits on 3; node 5 waits on 4; node 6 waits on 5
```

| # | Node | State | Waits on | Write set |
|---|---|---|---|---|
| 1 | Loop runner readiness | **ready** — not started | — | `docs/development/known-limitations.md` |
| 2 | Increment selection and pre-loop baseline | **ready** — not started | — | this file |
| 3 | Sealed-loop run | **blocked** | 1, 2 | one file under `subprojects/algorithm-evolution-agent-lab/`, plus the CodeFire state that loop writes |
| 4 | Loop evidence record | **blocked** | 3 | `docs/development/history.md`, this file |
| 5 | Final independent audit | **blocked** | 4 | `docs/development/ai/audit-latest.md` only |
| 6 | M1 exit and slot shift | **blocked** | 5 | board, history, packet, current-state docs |

Nodes 1 and 2 are independent: node 1 settles *what executes the loop*, node 2 settles *what the loop carries*. Neither reads the other's answer, and their write sets are disjoint. Starting only one of them is a planning defect. Nodes 5 and 6 are named here so the closing shape is fixed in advance; they are not this packet's implementation work.

### Node 1 — loop runner readiness

Decide, and record in `known-limitations.md`, which `codefire` executable runs this loop. The base environment exposes no `cargo` on `PATH`, and the only recorded source-matching binary is the July artifact `/tmp/codefire-cfr167-final.KWa8TC/bin/codefire`, which is not durable across sessions. Acceptable outcomes:

- a named, reachable executable with its provenance recorded; or
- an explicit recorded blocker if no such executable can be obtained without leaving M1 scope.

Recording a blocker is a legitimate node 1 result. Inventing a build pipeline, pinning a toolchain, or fixing the installer is **not** in scope — those are release-slot concerns.

### Node 2 — increment selection and pre-loop baseline

Pick the exact change before touching anything, and record here:

- the single file and the one-line-scale edit;
- which focused test covers that file;
- CodeFire's reported project state before the loop, captured verbatim from `status --json`.

Selection rules: one file, smallest real change with a reason to exist, and not a replay of any parked root. If the project is already `open-clean`, the increment is a new small change — `open-clean` is not permission to skip the edit.

### Node 3 — sealed-loop run

Run the loop in order on the selected increment: `open → edit → scan → extinguish → verify → commit`, then the one focused test. Stop and report on the first command that does not behave as documented; do not work around it and do not widen validation to diagnose it.

## Scope

Allowed files:

- `docs/development/known-limitations.md` (node 1)
- `docs/development/ai/task-packets/milestone-local-sealed-loop.md` (nodes 2, 4)
- exactly one file under `subprojects/algorithm-evolution-agent-lab/` (node 3), plus the CodeFire project state the loop itself writes
- `docs/development/history.md` (node 4)
- `docs/development/milestones.md` (node 6, orchestrator only)

Disallowed files:

- `crates/**`
- `codefire`
- `tests/**`
- `install.sh`, `pyproject.toml`, `setup.py`
- `adr/**`
- any second file under `subprojects/algorithm-evolution-agent-lab/`
- new CFB/CFR issues
- the classified dirty tree in `/workspace/project`, which is preserved pre-milestone state and not a write set

## Non-Goals

- Starting CFB-133, CFR-169, or CFR-170 (parked on M2) or any other parked root.
- Building the agent-consumable JSON / `next_actions` contract; that is M2.
- Remote `upload` / `request-merge` / `request-apply`; that is M3.
- Adoption or release work; those are M4 and M5.
- Changing sealed-commit, object-identity, or branch-head semantics.
- Running workspace-wide Rust tests, the full installer, the full Algorithm suite, or dogfood Retrospect as an admission ticket.
- Refactors or test expansion the loop does not require.

Parked-issue admission: a parked root enters this milestone only if node 3 produces dogfood evidence that the loop cannot complete without it, and then only the smallest slice that restores the loop. CFB-133 is the most likely candidate and is **not** admitted. Anticipating a block is not evidence.

## Invariants

Product invariants this packet must not violate:

- object identity: unchanged — the loop must not produce a new identity for unchanged content, and must not alias distinct content
- sealed commit validation: unchanged — the commit produced by node 3 must be sealed under the existing rules; a commit that only appears sealed is a failure, not a pass
- branch head atomicity: unchanged — the loop either advances the branch head completely or not at all
- Fire/Resolution semantics: `extinguish` must not clear a Fire that `scan` still has grounds to report, and `verify` must not pass over an unextinguished Fire
- remote mutation policy: unchanged — this milestone performs no remote mutation
- JSON envelope stability: unchanged — reading `status --json` must not require changing it
- path/workspace safety: all work stays inside the configured dogfood workspace; the separately configured legacy repository must not be mutated
- module boundaries: no product module is added, split, or moved by this milestone

If running the loop appears to require breaking one of these, that is a `block-m1` finding to report, not a thing to work around.

## Validation

```bash
codefire status --path subprojects/algorithm-evolution-agent-lab --json
codefire scan --path subprojects/algorithm-evolution-agent-lab
codefire extinguish --path subprojects/algorithm-evolution-agent-lab
codefire verify --path subprojects/algorithm-evolution-agent-lab
codefire commit --path subprojects/algorithm-evolution-agent-lab
python3 docs/development/tools/check_project_docs.py
```

Plus the **focused** test of the single module the increment touches, run inside `subprojects/algorithm-evolution-agent-lab`.

Do not widen. `cargo test --workspace`, `cargo clippy`, `cargo fmt`, the installer, the full Algorithm suite, and Retrospect are out of this milestone.

If a command cannot run, report the exact command, failure, environment reason, and review risk. A skipped exit-gate command is not an exit.

## Handoff

- files changed;
- the selected increment and why it is the smallest real change;
- which executable ran the loop, and its provenance;
- CodeFire-reported state before and after, verbatim;
- the sealed commit identity;
- validation actually run, command by command, with results;
- parked instead of done;
- independent packets still unstarted;
- stall risk;
- next allowed action.
