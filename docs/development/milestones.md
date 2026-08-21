# CodeFire Milestone Board

Status: live execution board

Owner: Orchestrator (Codex)

Last audited: 2026-08-21

Update trigger: M1 exit, M1 blocker admission, or slot shift.

This is the only live five-ahead execution board. Historical burn-down narrative stays in `current-cycle-burn-down-plan.md` and `history.md`. Those files are not permission to work.

The previous milestone `MS-2026-08-control-restore` exited on 2026-08-21 and the board shifted one slot. Its exit evidence is in `history.md`; its execution record is in `ai/task-packets/milestone-control-restore.md`. Neither is evidence about the current M1.

## Active slots

| Slot | ID | Name | Exit (must all be true) | In-scope now |
|---|---|---|---|---|
| M1 | `MS-local-sealed-loop` | Daily local sealed-commit loop | One real change goes `open → edit → scan → extinguish → verify → commit` with only the loop's focused validation; no parked review root is started unless dogfood shows it currently blocks that loop | **Yes** |
| M2 | `MS-agent-local-contract` | Agent-consumable local contract | Local `status` / `scan` / `verify` / `extinguish` / `commit` JSON, `next_actions`, and help are enough for a Claude or Grok worker to finish an M1-style local sealed-loop change without reading product source | No |
| M3 | `MS-remote-happy-path` | Remote happy path of sealed history | `upload` → `request-merge` → `request-apply` of an already sealed commit works on the documented happy path; bounded HTTP/graph work is in scope only if it blocks that path | No |
| M4 | `MS-adoption-wedge` | First adoption wedge | A second repository can inspect or import without becoming a Git bridge or an agent platform; remaining product experiments record adopt / narrow / stop | No |
| M5 | `MS-release-operational-trust` | Release and operational trust | Derived from `strategy/modernization-roadmap.md` Stage 4, bounded to: a recorded rights/license decision for the artifacts CodeFire actually publishes, and **one** rehearsed release-and-rollback path a backup operator can execute from documentation only, with checksums and independent verification. No public beta label while source gates or release identity are unresolved. The rest of Stage 4 stays in the roadmap and is not admitted by this slot | No |

M5 is a bounded slice of Stage 4, not a copy of it. `strategy/modernization-roadmap.md` remains a stage/gate roadmap and must not be turned into a second task board.

## M1 card — Daily local sealed-commit loop

Objective: prove the local sealed-commit loop is usable for real day-to-day work by taking one small real change all the way through it.

Exact M1 exit (nothing else counts):

- one small real change goes `open → edit → scan → extinguish → verify → commit` in the canonical dogfood project;
- the resulting commit is sealed and the post-loop state is reported by CodeFire itself, not asserted by an agent;
- only the loop's focused validation was required to get there.

Required implementation (this slot only):

- resolve which `codefire` executable runs the loop, and record it;
- select and record the one small real change before the loop starts;
- run the loop on that change and capture the CodeFire-reported before/after state;
- record the loop evidence and the sealed commit identity.

The first product increment is **one small real change** in `subprojects/algorithm-evolution-agent-lab`. If the canonical dogfood state is already `open-clean`, the increment is a new small change, not a replay of a parked root.

M1 validation (do not widen):

```bash
codefire status --path subprojects/algorithm-evolution-agent-lab --json
codefire scan --path subprojects/algorithm-evolution-agent-lab
codefire extinguish --path subprojects/algorithm-evolution-agent-lab
codefire verify --path subprojects/algorithm-evolution-agent-lab
codefire commit --path subprojects/algorithm-evolution-agent-lab
python3 docs/development/tools/check_project_docs.py
```

Plus the **focused** test of the single module the increment touches, inside `subprojects/algorithm-evolution-agent-lab`.

`cargo test --workspace`, `cargo clippy`, `cargo fmt`, the full installer, the full Algorithm suite, and dogfood Retrospect are **out of M1**. They belong to later slots if that slot names them.

M1 non-goals:

- starting CFB-133, CFR-169, or CFR-170 (now parked on M2) or any other parked root;
- building the agent-consumable JSON / `next_actions` contract (that is M2);
- remote `upload` / `request-merge` / `request-apply` work (M3);
- adoption or release work (M4, M5);
- refactors, cleanups, or test expansion that the loop does not require;
- treating the classified dirty tree in `/workspace/project` as a write set;
- serializing independent M1 slices "to keep the tree simple".

Parked-issue admission rule for this slot: a parked root enters M1 **only** if running the loop produces dogfood evidence that the loop cannot complete without it, and then only the smallest slice that restores the loop. Anticipating a block is not evidence. CFB-133 (`migrate apply` next-action loop) is the most likely candidate and is **not** admitted today.

## M1 execution graph (current)

This is the live plan for `MS-local-sealed-loop`. It is a concrete graph, not a template. Every active M1 must carry one; the shape rule lives in `milestone-operating-model.md`. A single serial queue with no parallel set is not an M1 plan.

```text
Critical path: loop runner readiness → sealed-loop run of the selected increment → loop evidence record → final independent audit → M1 exit and slot shift
Progress: nothing has run; nodes 1 and 2 are ready, nodes 3-6 are blocked
Parallel now: node 1 (loop runner readiness) and node 2 (increment selection and pre-loop baseline) — independent, disjoint write sets, both launchable immediately
Blocked: node 3 waits on 1 and 2; node 4 waits on 3; node 5 waits on 4; node 6 waits on 5
```

No node on this graph has run. There is no validation evidence, no sealed commit, and no audit for this milestone yet.

### Nodes

| # | Node | State | Owner | Waits on | Write set | Evidence |
|---|---|---|---|---|---|---|
| 1 | Loop runner readiness — decide which `codefire` executable runs the loop and record it, or record the blocker | **ready** — not started | Individual agent (Claude Opus 5 or Grok 4.6) | — | `docs/development/known-limitations.md` | — |
| 2 | Increment selection and pre-loop baseline — name the exact one small change and record the CodeFire-reported state before the loop | **ready** — not started | Individual agent (the other family) | — | `docs/development/ai/task-packets/milestone-local-sealed-loop.md` | — |
| 3 | Sealed-loop run — `open → edit → scan → extinguish → verify → commit` on the selected increment plus its focused test | **blocked** on 1, 2 | Individual agent | 1, 2 | the one touched file under `subprojects/algorithm-evolution-agent-lab/`, and the CodeFire state that loop writes | — |
| 4 | Loop evidence record — sealed commit identity, before/after state, and which commands actually ran | **blocked** on 3 | Individual agent | 3 | `docs/development/history.md`, `docs/development/ai/task-packets/milestone-local-sealed-loop.md` | — |
| 5 | Final independent audit (Claude Opus 5 or Grok 4.6, never Codex, not the implementing session) | **blocked** on 4 | Audit agent | 4 | `docs/development/ai/audit-latest.md` only | — |
| 6 | M1 exit and slot shift | **blocked** on 5 | Orchestrator | 5 | board, history, packet, current-state docs | — |

### Why nodes 1 and 2 are genuinely independent

Node 1 answers *what runs the loop*. The base environment exposes no `cargo` on `PATH`, and the only recorded source-matching binary is the July `/tmp/codefire-cfr167-final.KWa8TC/bin/codefire` artifact, which is not a durable runner. That question must be settled before the loop can execute.

Node 2 answers *what the loop carries*. Choosing the one small change and recording the pre-loop state reads the dogfood project and the board; it does not depend on which executable will later run.

Their write sets are disjoint (`known-limitations.md` versus the M1 packet file), so both may run concurrently from the start. Launching only one of them is a stall, not caution.

### Not yet true

M1 is **open** and nothing on this graph has run. Do not read the previous milestone's green validations as evidence here: those covered docs and control-plane scripts at `8b6c822` and said nothing about product behavior. This milestone is the first slot that touches product/dogfood behavior, so its exit needs its own loop evidence and its own independent audit.

## Parked open issues

These remain `investigating` in the ledgers. They are not M1 work.

| Issue | Parked on | Why not M1 |
|---|---|---|
| CFB-133 | M2 | `migrate apply` next-action loop; an automation contract concern, not required to run the loop by hand |
| CFR-169 | M2 | Module/test ownership seams; not required to run the loop |
| CFR-170 | M2 | Typed executable outcomes; needed once agents consume local contracts |
| CFR-168 | M3 | Graph/object/HTTP work budget is for remote/scale, not for the local loop |
| CFB-136 | M3 | Validation-cache reuse is the admitted CFR-168 blocker |
| CFR-171 | M4 | Machine-readable claim/proof descriptor for external consumers |
| CFR-172 | M4 | Read-only adoption experiment |
| CFR-173 | M4 | Product falsification / stop-rule experiment |

`BATCH-2026-07-review-portfolio-completion` is **frozen as history**. Remaining roots are parked as above. They do not gate M1 or M2 unless dogfood evidence shows the local sealed loop is actually unusable without them.

## M2 preview (queued)

When M1 exits, M2 `MS-agent-local-contract` becomes active and the first allowed increment is:

- make the local `status` / `scan` / `verify` / `extinguish` / `commit` JSON and `next_actions` sufficient for a worker to repeat an M1-style loop change without reading product source;
- admit CFR-170 or CFB-133 only to the extent that contract requires them.

Do not pull CFR-168 into M2 because it is unfinished.

## Shift rule

On M1 close:

1. Record exit evidence in `history.md`.
2. Rename slots: old M2 becomes M1, old M3 becomes M2, and so on.
3. Name a new M5 from `strategy/modernization-roadmap.md` without turning that roadmap into a task board.
4. Refuse any in-flight parked work that tried to start early.
