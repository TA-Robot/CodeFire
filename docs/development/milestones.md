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
Progress: nodes 1-4 done; node 5 ready; node 6 blocked
Parallel now: none — node 5 is the sole runnable post-fan-in node and no independent M1 packet is idle
Blocked: M1 exit and slot shift wait on the final independent audit; the integration and validation edges that the final audit waited on are now cleared
```

`Parallel now: none` here is fan-in, not stall. The two independent lanes already ran concurrently and both landed (see "Parallel lanes that ran"). Every remaining M1 item is downstream of node 5 on a single edge, so there is no idle independent packet to launch and no free slot being wasted. Idle capacity would only be a stall signal if an unstarted independent M1 packet existed; none does.

### Nodes on the critical path

| # | Node | State | Owner | Waits on | Evidence |
|---|---|---|---|---|---|
| 1 | Classified-worktree snapshot `eb4fd0c` | **done**, preservation-only | Orchestrator | — | `eb4fd0c` `snapshot: preserve classified pre-M1 worktree` |
| 2 | Board and packet plan completion | **done** | Individual agent (Claude Opus 5) | 1 | `a7b750e` `docs: make M1 execution graph concrete`, integrated byte-identically |
| 3 | Integration of M1 packet worktrees | **done** | Orchestrator (Codex, integration branch only) | 2 | `64b7c55` on `integration/ms-2026-08-control-restore`, 2026-08-21T19:14:51Z; both file blobs identical to `a7b750e` |
| 4 | M1 focused validation (the command list above, no wider) | **done** | Validation lane (distinct Claude session) | 3 | All five packet commands exited 0 at exact HEAD `64b7c55`; worktree clean before and after |
| 5 | Final independent audit (Grok 4.6 or Claude Opus 5, never Codex, not the implementing session) | **ready** — not started | Audit agent | 3, 4 (both cleared) | — |
| 6 | M1 exit and slot shift | **blocked** on node 5 | Orchestrator | 5 | — |

### Parallel lanes that ran

Both lanes ran concurrently with disjoint write sets and both completed. This record is kept as evidence that M1 was not planned as a single serial queue:

| Lane | Write set | Outcome |
|---|---|---|
| Board/packet plan completion | `milestones.md`, `ai/task-packets/milestone-control-restore.md` | Landed as `a7b750e`, integrated at `64b7c55` |
| Independent baseline Grok 4.6 audit | `ai/audit-latest.md` only | Completed 2026-08-21T19:15:02Z |

Neither lane entered the other's write set. The baseline audit was **not** node 5: it observed plan-vs-progress and did not clear the M1 exit.

Baseline audit result, recorded honestly: it reported `stalled=yes` **solely** because the board still carried plan placeholders at the time it read them. Node 2 (`a7b750e`) resolved that specific finding. It reported `scope_creep=no` and all four M1-card validations green. The stall verdict is therefore closed by node 2, not overridden or dismissed.

### Node 4 validation evidence

Run by a distinct Claude validation runner at exact HEAD `64b7c55`. All five packet commands exited 0:

| Command | Result |
|---|---|
| `check_project_docs.py` | passed, 534 Markdown files inventoried |
| `check_ai_acceleration_docs.py` | passed |
| `run_milestone_audit.py` | final audit brief generated, 123 lines |
| `bash -n /workspace/scripts/dev-agent` | passed |
| `/workspace/scripts/test-dev-agent.sh` | passed, all 6 assertions |

Worktree clean before and after. **No product tests were run** — Rust workspace tests, installer, and Algorithm dogfood Retrospect stay out of M1 per the card above, so node 4 is not evidence about product behavior.

That evidence is pinned to tree `64b7c55`. This progress commit edits the same two docs files, so node 5 must run at the resulting commit, not at `64b7c55`. The two doc checkers are re-run here; `bash -n` and `test-dev-agent.sh` target `/workspace/scripts/` and are unaffected by these two files, while `run_milestone_audit.py` reads this board and will render the updated graph.

### About snapshot `eb4fd0c`

`eb4fd0c` (`snapshot: preserve classified pre-M1 worktree`) is **preservation-only**. It committed 339 files and roughly 198,000 inserted lines of already-accumulated pre-M1 work, product code included, so the classified worktree could not be lost between sessions.

It is explicitly **not**:

- a clean-tree checkpoint;
- a release or a validated baseline;
- evidence that any M1 exit criterion is met;
- permission to continue the parked work it happens to contain.

No M1 exit-gate command was run against it and it records no exit evidence in `history.md`. It is the starting point of this graph and nothing more.

### Not yet true

M1 is **open**. Nodes 1-4 being done is not an exit: the final independent audit (node 5) has not run, node 6 has not run, and no slot has shifted. Four green validations plus a clean integration are the *inputs* to the final audit, not a substitute for it. Do not read this board as an M1 close and do not pre-emptively rename slots.

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
