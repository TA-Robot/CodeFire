# AI Task Packet

Status: blocked

Packet ID: MS-local-sealed-loop

Schema version: human-v1

Base revision: the commit that closed `MS-2026-08-control-restore` on 2026-08-21

Risk score: 5

Reasoning tier: high

This is the **parent packet** for the milestone. Nodes 1 and 2 have their own child packets, per `../../milestone-operating-model.md` and `../orchestration-protocol.md`: one packet per parallel slice.

- node 1 → `milestone-local-sealed-loop-runner.md` — ran; Status `blocked` (`block-m1`), integrated at `eab877f`
- node 2 → `milestone-local-sealed-loop-increment.md` — ran; Status `completed`, integrated at `f52d489`
- node 3 → still not written. The fan-in happened, but node 1 produced no runner, so node 3's first mandated step and write set are undetermined

Everything from `## Objective` through `## Handoff` is the original **plan** and carries no evidence — do not read those sections as observations. The exceptions are the `Progress` / `Parallel now` / `Blocked` lines and the `State` and `Outcome` cells in `## Plan`, which mirror the live board, plus `## Fan-in result`, which records what the two child packets returned and nothing more. This packet carries **no product evidence**: no CodeFire command, no dogfood edit, and no focused test has run in this milestone.

## Objective

Prove the local sealed-commit loop is usable for real day-to-day work by taking **one small real change** in the canonical dogfood project all the way through `open → edit → scan → extinguish → verify → commit`, using only the loop's focused validation.

The deliverable is a sealed commit plus CodeFire's own before/after report of the project state. An agent asserting "the loop works" is not the deliverable.

## Context

- Required context pack(s): pack-0-universal-orientation, pack-6-dogfood-algorithm-project
- Additional files: `docs/development/milestones.md`, `docs/development/milestone-operating-model.md`, `docs/development/known-limitations.md`, `subprojects/algorithm-evolution-agent-lab/AGENTS.md`
- Related issue or observation: `MS-2026-08-control-restore` restored the scheduler but validated no product behavior. This is the first slot that touches product/dogfood behavior.
- Board state to respect: the canonical Algorithm project was last recorded `open-clean` at sealed bridge `CF-COMMIT-e5ca290905c2a4b5e2c05bd4` with zero Fires. That is a **prior record**, not a fact this packet has re-observed. Node 3 re-reads it with `status --json` before the loop; nodes 1 and 2 must not rely on it and must not run `status` to check it.

## Plan

The live graph is on `docs/development/milestones.md` under "M1 execution graph (current)". That board is authoritative; this section is the packet-local copy.

```text
Critical path: loop runner readiness → sealed-loop run of the selected increment (pre-loop baseline first) → loop evidence record → final independent audit → M1 exit and slot shift
Progress: nodes 1 and 2 ran concurrently and have fanned in at f52d489 — node 1 blocked with a recorded block-m1 runner blocker (eab877f), node 2 completed its reading-only increment selection (f52d489); node 3 has not started and nodes 3-6 stay blocked
Parallel now: none — no independent ready packet remains
Blocked: node 3 waits on 1 and 2 — node 2's input exists but node 1 supplied no runner; node 4 waits on 3; node 5 waits on 4; node 6 waits on 5
Next legal action: independent classification of the node 1 runner blocker, then orchestrator admission of the smallest resolution
```

| # | Node | State | Waits on | Packet | Write set | Outcome at `f52d489` |
|---|---|---|---|---|---|---|
| 1 | Loop runner readiness | **blocked** — ran, recorded an exact blocker | — | `milestone-local-sealed-loop-runner.md` | that packet file only | `block-m1`: no in-scope executable can express the `--path` / `--json` gate. Integrated at `eab877f` |
| 2 | Increment selection (reading only) | **completed** | — | `milestone-local-sealed-loop-increment.md` | that packet file only | one candidate file, one one-line edit, one focused unittest recorded; edit not applied, no command run. Integrated at `f52d489` |
| 3 | Sealed-loop run, pre-loop `status --json` baseline first | **blocked** — not started | 1, 2 | not written; blocked on the node 1 resolution | one file under `subprojects/algorithm-evolution-agent-lab/`, plus the CodeFire state that loop writes | — |
| 4 | Loop evidence record | **blocked** — not started | 3 | this file | `docs/development/history.md`, this file | — |
| 5 | Final independent audit | **blocked** — not started | 4 | — | `docs/development/ai/audit-latest.md` only | — |
| 6 | M1 exit and slot shift | **blocked** — not started | 5 | — | board, history, packet, current-state docs | — |

Nodes 1 and 2 were independent and ran as such: node 1 settled *what executes the loop*, node 2 settled *what the loop carries*. Neither read the other's answer, and their write sets stayed disjoint — each child packet wrote only itself. That concurrency is now **evidence of how the two lanes ran**, not a standing second lane: node 2 is finished and node 1 returned a blocker, so there is no independent ready packet left to launch. Nodes 5 and 6 remain named so the closing shape stays fixed in advance; they are not this packet's implementation work.

The pre-loop `status --json` baseline is **node 3 work**. It cannot run until a runnable executable exists, so placing it in node 2 would have given node 2 a hidden dependency on node 1 and destroyed the parallelism this graph claims. Node 2 selected by reading only, and node 1's blocker is exactly why that baseline still has no executable.

### Node 1 — loop runner readiness

Owned by `milestone-local-sealed-loop-runner.md`. Decide which `codefire` executable runs this loop and record its path, provenance, and capability outcome in that packet — or record an exact blocker there. Acceptable outcomes:

- a named, reachable executable with recorded provenance; or
- an explicit recorded blocker if no such executable can be obtained without leaving M1 scope.

Recording a blocker is a legitimate node 1 result. Inventing a build pipeline, pinning a toolchain, or fixing the installer is **not** in scope — those are release-slot concerns.

### Node 2 — increment selection

Owned by `milestone-local-sealed-loop-increment.md`. Pick the exact change by reading, and record in that packet the candidate file, the minimal edit, the reason it is worth making, and the focused test that covers it. Node 2 runs no CodeFire command, records no CodeFire state, and edits no product file.

Selection rules: one file, smallest real change with a reason to exist, and not a replay of any parked root. A prior record that the project is `open-clean` is not permission to skip the edit; the increment is a new small change either way.

### Node 3 — sealed-loop run

Created only after nodes 1 and 2 fan in, because its write set depends on their results. It captures the pre-loop `status --json` baseline verbatim using the node 1 runner, then runs the loop in order on the node 2 increment: `open → edit → scan → extinguish → verify → commit`, then the one focused test. Stop and report on the first command that does not behave as documented; do not work around it and do not widen validation to diagnose it.

## Scope

Allowed files, per node:

- `docs/development/ai/task-packets/milestone-local-sealed-loop-runner.md` (node 1, and nothing else)
- `docs/development/ai/task-packets/milestone-local-sealed-loop-increment.md` (node 2, and nothing else)
- exactly one file under `subprojects/algorithm-evolution-agent-lab/` (node 3), plus the CodeFire project state the loop itself writes
- `docs/development/history.md` and this file (node 4)
- `docs/development/milestones.md` (node 6, orchestrator only)

Nodes 1 and 2 write only their own packet. Neither may touch product code or CodeFire state.

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

## Fan-in result

Observed at integration head `f52d489`. Nodes 1 and 2 ran in parallel and have both returned; this section records their results and nothing beyond them.

### What the two lanes returned

- **Node 1 — blocked (`block-m1`), integrated at `eab877f`.** `milestone-local-sealed-loop-runner.md` is Status `blocked`. It establishes that no in-scope executable can run the M1 exit-gate loop: the `--path` / `--json` surface the gate names belongs to the Rust CLI (`crates/codefire-cli`, bin `codefire-rs`), and no `codefire`, `cargo`, or `rustc` is on `PATH`, no build output or toolchain exists anywhere reachable, and the previously recorded July `/tmp` binary is gone. The Python `./codefire` at the repository root is durable, identity-known, and runnable, but implements neither `--path` nor `--json`, so it cannot express the gate or emit the JSON state evidence M1 exits on. Every probe there was read-only (`--help`, `ls`, `find`, `grep`, `git`, `sha256sum`, `pip show`); no CodeFire command ran against the dogfood project or any managed repository.
- **Node 2 — completed, integrated at `f52d489`.** `milestone-local-sealed-loop-increment.md` is Status `completed`. It selected exactly one candidate — a one-line `improved_domains` change in `subprojects/algorithm-evolution-agent-lab/src/evoagent/external_validity.py` — with its reason (an unmapped benchmark currently mints a synthetic `"unknown"` domain and can make `transferable=True` without a second real domain, against REQ-SOTA-007) and one focused unittest, `PYTHONPATH=src python3 -m unittest tests.test_external_validity -v`. Selection was by reading only: **the edit was not applied**, no CodeFire command ran, and no test ran. Alternates are recorded as fallbacks and are not to be applied.

### What is therefore true, and what is not

- Node 3 has not started. No product file, no dogfood file, and no CodeFire-managed state changed in this milestone.
- No product validation has run. The only commands recorded across both lanes are the two child packets' docs checks (`check_project_docs.py`, `git diff --check`) plus node 1's read-only help probes. Those say nothing about product behavior.
- There is no runner, no loop evidence, no sealed commit, and no independent audit for this milestone.
- **A recorded candidate is not a cleared blocker.** Node 2's completion does not resolve node 1: knowing which line to change does not supply an executable that can run `status --json`, `scan`, `extinguish`, `verify`, or `commit`. The milestone is blocked on node 1 alone.
- The exact M1 exit on the board is unchanged by this fan-in, and so is the parked-work list. Nothing here admits CFB-133, CFB-136, or CFR-168..173, and node 1's blocker is not evidence for admitting any of them.

### Next legal action

1. **Independent classification of the node 1 runner blocker** by an agent that did not run node 1: confirm or refute the `block-m1` class and enumerate the smallest candidate resolutions. Classification records options only.
2. **Orchestrator admission** of the smallest resolution, recorded before any further node runs.

This packet does **not** choose the resolution. Provisioning a Rust toolchain or a build, amending the M1 exit gate so its evidence no longer requires `--json`, and adding `--path` / `--json` to the Python implementation are all out of this packet's write set: the first two change milestone scope, the third is product code and belongs to the M2 `MS-agent-local-contract` surface. Node 3 stays unwritten and unstarted until admission.

**Stall risk:** no independent M1 packet remains, so there is no legal parallel lane right now. Idle capacity must not be filled by inventing work, by pre-writing the node 3 packet, or by starting a parked root. Waiting on admission is the correct state, not a stall to be worked around.
