# AI Task Packet

Status: completed

Packet ID: MS-2026-08-control-restore

Schema version: human-v1

Base revision: current project worktree

Risk score: 3

Reasoning tier: high

Closed: 2026-08-21

This packet is **historical evidence** for the milestone that exited on 2026-08-21. It is not permission to work and it is not evidence about any later milestone. The live board is `../../milestones.md`; do not copy an old M1 name from this packet.

## Objective

Install milestone-based scheduling and the Codex-orchestrator / Claude-or-Grok-worker / 30-minute-auditor topology so unbounded tests and unbounded review response stop being the scheduler.

## Context

- Required context pack(s): pack-0-universal-orientation, pack-5-project-management-docs
- Additional files: `docs/development/milestones.md`, `docs/development/milestone-operating-model.md`, `docs/development/ai/orchestration-protocol.md`
- Related issue or observation: development stall from review-portfolio burn-down and extra gates

## Plan

This graph ran to completion. The live board `docs/development/milestones.md` no longer carries it; it now carries the graph for the milestone that followed. This section is the closed record.

```text
Critical path: classified-worktree snapshot eb4fd0c → board/packet plan completion → integration → M1 focused validation → final independent Grok or Claude audit → M1 exit and slot shift
Progress: nodes 1-6 done
Parallel now: none — the graph is closed
Blocked: nothing
```

Nodes 2-4 fanned in on a single edge into node 5. `Parallel now: none` at that stage was fan-in, not stall: both independent lanes had already run concurrently and landed.

Node states:

| # | Node | State | Evidence |
|---|---|---|---|
| 1 | Classified-worktree snapshot `eb4fd0c` | **done**, preservation-only | `eb4fd0c` |
| 2 | Board and packet plan completion | **done** | `a7b750e`, integrated byte-identically |
| 3 | Integration of M1 packet worktrees | **done** | `64b7c55` on `integration/ms-2026-08-control-restore`, 2026-08-21T19:14:51Z |
| 4 | M1 focused validation | **done** | five commands exited 0 at exact HEAD `64b7c55`; clean worktree before and after |
| 5 | Final independent audit (Grok 4.6 or Claude Opus 5, never Codex) | **done** | independent Grok 4.6 audit at 2026-08-21T19:22:30Z against exact HEAD `8b6c822`; `stalled=no`, `scope_creep=no`, no `block-m1` finding, all five M1-card commands re-run green at that exact HEAD; verdict "M1 exit column is true", advice "close M1 on this evidence" |
| 6 | M1 exit and slot shift | **done** | this commit: `history.md` exit row, this packet set to `completed`, board shifted one slot with `MS-local-sealed-loop` as the new M1 and `MS-release-operational-trust` named as the new M5 |

### Node 5 audit verdict (2026-08-21T19:22:30Z)

The final independent audit was Grok 4.6, a different family from the implementing Claude session and never Codex. It audited exact HEAD `8b6c822` and returned:

- `stalled: no`;
- `scope_creep: no`;
- no `block-m1` finding — the remaining `in_progress` packet label was classed as orchestrator close-out, not missing implementation;
- all five M1-card commands re-run green at `8b6c822` (534 Markdown files inventoried; AI docs pass; `bash -n /workspace/scripts/dev-agent`; `/workspace/scripts/test-dev-agent.sh` all 6 assertions; `run_milestone_audit.py` brief generated);
- ledger honesty confirmed: none of the eight open CFB/CFR items were closed as fixed;
- explicit authorization for node 6, and only node 6, as the next legal action.

It also named the risk that motivated this commit: leaving this packet `in_progress`, or the parked `cfr-168-decision-work-budget.json` `in_progress`, would read as live work to the next worker. Both labels are corrected in this commit.

That audit record lives in `docs/development/ai/audit-latest.md`, which is gitignored runtime evidence and is not committed.

### Slot shift performed by node 6

| Before | After |
|---|---|
| M1 `MS-2026-08-control-restore` | exited — this packet |
| M2 `MS-local-sealed-loop` | M1, active |
| M3 `MS-agent-local-contract` | M2 |
| M4 `MS-remote-happy-path` | M3 |
| M5 `MS-adoption-wedge` | M4 |
| — | M5 `MS-release-operational-trust`, a bounded slice of `strategy/modernization-roadmap.md` Stage 4 |

All eight open CFB/CFR items kept `investigating` and moved with their slot: CFB-133 / CFR-169 / CFR-170 → M2, CFB-136 / CFR-168 → M3, CFR-171 / CFR-172 / CFR-173 → M4. No issue was closed, reopened, or started.

Snapshot `eb4fd0c` is **preservation-only**: a 339-file, ~198,000-insertion commit of already-accumulated pre-M1 work, product code included, made so the classified worktree survived the session boundary. It is not a clean tree, not a release, not a validated baseline, and not evidence for any M1 exit criterion. No exit-gate command was run against it and it carries no `history.md` exit evidence.

Parallel lanes that ran, with disjoint write sets — kept as evidence that M1 was not a single serial queue:

| Lane | Write set | Outcome |
|---|---|---|
| Board/packet plan completion (this lane) | `docs/development/milestones.md`, `docs/development/ai/task-packets/milestone-control-restore.md` | `a7b750e` → integrated `64b7c55` |
| Independent baseline Grok 4.6 audit | `docs/development/ai/audit-latest.md` only | completed 2026-08-21T19:15:02Z; `stalled=yes` only from the pre-node-2 placeholders, `scope_creep=no`, four M1-card validations green |

The baseline audit was an observation lane, not node 5. It did not clear the M1 exit; node 5 did.

M1 is **closed**. What that close does and does not assert:

- it asserts the control plane — board, operating model, agent topology, provider routing, parked-issue classification — is installed and is the source of truth;
- it does **not** assert anything about CodeFire product behavior. No product code was changed by this milestone and none was validated by it. Every command in the exit gate targeted development docs and control-plane scripts;
- it does **not** clear any parked root. All eight open CFB/CFR items stayed `investigating` and were carried forward one slot.

## Scope

Allowed files:

- `docs/development/milestone-operating-model.md`
- `docs/development/milestones.md`
- `docs/development/project-management.md`
- `docs/development/README.md`
- `docs/development/current-cycle-burn-down-plan.md`
- `docs/development/codefire-improvement-cycle.md`
- `docs/development/history.md`
- `docs/development/ai/**`
- `docs/development/strategy/decision-register.md`
- `docs/development/strategy/README.md`
- `docs/development/strategy/modernization-roadmap.md`
- `AGENTS.md`
- open CFB/CFR detail files only to record the parked milestone
- `/workspace/scripts/dev-agent` and `/workspace/scripts/test-dev-agent.sh`
- `/workspace/docs/low-codex-multi-agent.md`
- `/workspace/README.md`
- `/workspace/AGENTS.md`

Disallowed files:

- `crates/**`
- `codefire`
- `tests/**`
- `subprojects/**`
- new CFB/CFR issues
- Grok `second-agent` protocol backend

## Non-Goals

- Finishing CFR-168 or any parked root.
- Adding a fourth second-agent protocol.
- Changing sealed-commit or object-identity semantics.
- Running workspace-wide Rust tests as an admission ticket.

## Invariants

- object identity: unchanged
- sealed commit validation: unchanged
- branch head atomicity: unchanged
- remote mutation policy: unchanged
- JSON envelope stability: unchanged
- path/workspace safety: workers stay in configured workspace
- module boundaries: product code is not edited

## Validation

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
python3 docs/development/tools/run_milestone_audit.py >/tmp/milestone-audit-brief.md
bash -n /workspace/scripts/dev-agent
/workspace/scripts/test-dev-agent.sh
```

If a command cannot run, report the exact command, failure, environment reason, and review risk.

### Node 4 result (full list, at HEAD `64b7c55`)

Run by a distinct Claude validation runner. All five commands exited 0:

| Command | Result |
|---|---|
| `check_project_docs.py` | passed, 534 Markdown files inventoried |
| `check_ai_acceleration_docs.py` | passed |
| `run_milestone_audit.py` | final audit brief generated, 123 lines |
| `bash -n /workspace/scripts/dev-agent` | passed |
| `/workspace/scripts/test-dev-agent.sh` | passed, all 6 assertions |

Worktree clean before and after. **No product tests were run**; Rust workspace tests, installer, and Algorithm dogfood Retrospect stay out of M1, so this is not evidence about product behavior.

### Bounds on that evidence

The node 4 result is pinned to tree `64b7c55`. The progress commit that followed it edited the same two docs files, producing the **later tree** `8b6c822`. Node 5 therefore audited `8b6c822`, not `64b7c55`, and re-ran all five commands there rather than inheriting node 4's results. That is the evidence the exit rests on.

Neither node 4 nor node 5 is evidence about product behavior. Both command lists cover development docs and `/workspace/scripts/` only.

### Bounds on the exit itself

This close-out commit (node 6) edits current-state docs — board, history, hub, cycle docs, roadmap paragraph, orchestration invocation, issue milestone labels, and the superseded CFR-168 json label. It re-runs the docs-scoped subset:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
python3 docs/development/tools/run_milestone_audit.py
git diff --check
```

`bash -n /workspace/scripts/dev-agent` and `/workspace/scripts/test-dev-agent.sh` target `/workspace/scripts/`, which this commit does not touch, so the node 5 results for them stand. No product test was run and none was required.

## Handoff

- files changed;
- design summary;
- validation result;
- risks;
- follow-up.
- task/packet digest and resulting patch digest when automation provides them.
