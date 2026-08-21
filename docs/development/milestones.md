# CodeFire Milestone Board

Status: live execution board

Owner: Orchestrator (Codex)

Last audited: 2026-08-21

Update trigger: M1 exit, M1 blocker admission, or slot shift.

This is the only live five-ahead execution board. Historical burn-down narrative stays in `current-cycle-burn-down-plan.md` and `history.md`. Those files are not permission to work.

## Active slots

| Slot | ID | Name | Exit (must all be true) | In-scope now |
|---|---|---|---|---|
| M1 | `MS-2026-08-control-restore` | Restore milestone-driven development | Operating model, board, agent topology, and routing are the source of truth; remaining review-portfolio issues are parked on M2–M5; no agent is required to continue CFR-168 or sibling roots to make progress | **Yes** |
| M2 | `MS-local-sealed-loop` | Daily local sealed-commit loop | One real change goes `open → edit → scan → extinguish → verify → commit` with only the loop's focused validation; no parked review root is started unless it currently blocks that loop on dogfood | No |
| M3 | `MS-agent-local-contract` | Agent-consumable local contract | Local `status` / `scan` / `verify` / `extinguish` / `commit` JSON, `next_actions`, and help are enough for a Claude or Grok worker to finish an M2-style change without reading product source | No |
| M4 | `MS-remote-happy-path` | Remote happy path of sealed history | `upload` → `request-merge` → `request-apply` of an already sealed commit works on the documented happy path; bounded HTTP/graph work is in scope only if it blocks that path | No |
| M5 | `MS-adoption-wedge` | First adoption wedge | A second repository can inspect or import without becoming a Git bridge or an agent platform; remaining product experiments record adopt / narrow / stop | No |

## M1 card — Restore milestone-driven development

Objective: stop unbounded testing and unbounded review response from being the scheduler.

Required implementation (this slot only):

- publish this board and `milestone-operating-model.md`;
- install the orchestrator / individual / audit topology;
- route Codex to orchestration only;
- route individual and audit work to Claude Opus 5 or Grok 4.6;
- classify every currently open CFB/CFR issue onto M2–M5;
- keep ledger counts honest; do not close parked issues as fixed.

M1 validation (do not widen):

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
bash -n /workspace/scripts/dev-agent
/workspace/scripts/test-dev-agent.sh
```

Rust workspace tests, installer, and Algorithm dogfood Retrospect are **out of M1**. They belong to later slots if that slot names them.

M1 non-goals:

- finishing CFR-168 remaining surfaces;
- admitting new review roots;
- adding a fourth second-agent protocol for Grok;
- product CLI behavior changes.
- serializing independent M1 slices "to keep the tree simple"

## M1 execution graph (current)

This is the live plan for `MS-2026-08-control-restore`. It is a concrete graph, not a template. Every active M1 must carry one, including later slots after a shift; the shape rule lives in `milestone-operating-model.md`. A single serial queue with no parallel set is not an M1 plan.

```text
Critical path: classified-worktree snapshot eb4fd0c → board/packet plan completion → integration → M1 focused validation → final independent Grok or Claude audit → M1 exit and slot shift
Parallel now: board/packet plan-completion packet, independent baseline audit writing audit-latest.md
Blocked: final independent audit waits on integration; M1 exit and slot shift wait on the final audit and on every validation named on this card
```

### Nodes on the critical path

| # | Node | State | Owner | Waits on |
|---|---|---|---|---|
| 1 | Classified-worktree snapshot `eb4fd0c` | done, preservation-only | Orchestrator | — |
| 2 | Board and packet plan completion | in progress | Individual agent (Claude Opus 5) | 1 |
| 3 | Integration of M1 packet worktrees | not started | Orchestrator (Codex, integration branch only) | 2 |
| 4 | M1 focused validation (the command list above, no wider) | not started | Validation lane | 3 |
| 5 | Final independent audit (Grok 4.6 or Claude Opus 5, never Codex, not the implementing session) | not started | Audit agent | 3, and 4 for its evidence |
| 6 | M1 exit and slot shift | not started | Orchestrator | 4 and 5 |

### Parallel now

Two lanes are live with disjoint write sets:

| Lane | Write set | Why it is independent |
|---|---|---|
| Board/packet plan completion | `milestones.md`, `ai/task-packets/milestone-control-restore.md` | Records the graph; needs no audit result to exist |
| Independent baseline audit | `ai/audit-latest.md` only | Read-only over plan vs progress; needs no board edit to observe stall |

Neither lane may enter the other's write set. The baseline audit is **not** node 5: it observes the current state, it does not clear the M1 exit.

### About snapshot `eb4fd0c`

`eb4fd0c` (`snapshot: preserve classified pre-M1 worktree`) is **preservation-only**. It committed 339 files and roughly 198,000 inserted lines of already-accumulated pre-M1 work, product code included, so the classified worktree could not be lost between sessions.

It is explicitly **not**:

- a clean-tree checkpoint;
- a release or a validated baseline;
- evidence that any M1 exit criterion is met;
- permission to continue the parked work it happens to contain.

No M1 exit-gate command was run against it and it records no exit evidence in `history.md`. It is the starting point of this graph and nothing more.

### Not yet true

M1 is **open**. The exit criteria are not met, the final independent audit has not run, and no slot has shifted. Do not read node 1 or node 2 as an M1 close.

## Parked open issues

These remain `investigating` in the ledgers. They are not M1 work.

| Issue | Parked on | Why not M1 |
|---|---|---|
| CFR-168 | M4 | Graph/object/HTTP work budget is for remote/scale, not for restoring the scheduler |
| CFB-136 | M4 | Validation-cache reuse is the admitted CFR-168 blocker |
| CFR-169 | M3 | Module/test ownership seams; not required to restore planning |
| CFR-170 | M3 | Typed executable outcomes; needed once agents consume local contracts |
| CFB-133 | M3 | `migrate apply` next-action loop; automation contract, not scheduler restore |
| CFR-171 | M5 | Machine-readable claim/proof descriptor for external consumers |
| CFR-172 | M5 | Read-only adoption experiment |
| CFR-173 | M5 | Product falsification / stop-rule experiment |

`BATCH-2026-07-review-portfolio-completion` is **frozen as history**. Remaining roots are parked as above. They do not gate M1 or M2 unless dogfood evidence shows the local sealed loop is actually unusable.

## M2 preview (queued)

When M1 exits, M2 becomes active and the first allowed product increment is:

- take one bounded dogfood or docs/code change through the local sealed loop;
- run only `status` / `scan` / `extinguish` / `verify` / `commit` plus the focused tests of that change;
- if the loop is already `open-clean` on canonical Algorithm, the increment is a new small change, not a replay of parked roots.

Do not pull CFR-168 into M2 because it is unfinished.

## Shift rule

On M1 close:

1. Record exit evidence in `history.md`.
2. Rename slots: old M2 becomes M1, old M3 becomes M2, and so on.
3. Name a new M5 from `strategy/modernization-roadmap.md` without turning that roadmap into a task board.
4. Refuse any in-flight parked work that tried to start early.
