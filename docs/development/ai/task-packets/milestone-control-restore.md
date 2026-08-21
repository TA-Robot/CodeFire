# AI Task Packet

Status: in_progress

Packet ID: MS-2026-08-control-restore

Schema version: human-v1

Base revision: current project worktree

Risk score: 3

Reasoning tier: high

## Objective

Install milestone-based scheduling and the Codex-orchestrator / Claude-or-Grok-worker / 30-minute-auditor topology so unbounded tests and unbounded review response stop being the scheduler.

## Context

- Required context pack(s): pack-0-universal-orientation, pack-5-project-management-docs
- Additional files: `docs/development/milestones.md`, `docs/development/milestone-operating-model.md`, `docs/development/ai/orchestration-protocol.md`
- Related issue or observation: development stall from review-portfolio burn-down and extra gates

## Plan

The live graph is on `docs/development/milestones.md` under "M1 execution graph (current)". That board is authoritative; this section is the packet-local copy.

```text
Critical path: classified-worktree snapshot eb4fd0c → board/packet plan completion → integration → M1 focused validation → final independent Grok or Claude audit → M1 exit and slot shift
Progress: nodes 1-4 done; node 5 ready; node 6 blocked
Parallel now: none — node 5 is the sole runnable post-fan-in node and no independent M1 packet is idle
Blocked: M1 exit and slot shift wait on the final independent audit; the integration and validation edges that the final audit waited on are now cleared
```

`Parallel now: none` is fan-in, not stall: both independent lanes already ran concurrently and landed, and everything left is downstream of node 5 on one edge.

Node states:

| # | Node | State | Evidence |
|---|---|---|---|
| 1 | Classified-worktree snapshot `eb4fd0c` | **done**, preservation-only | `eb4fd0c` |
| 2 | Board and packet plan completion | **done** | `a7b750e`, integrated byte-identically |
| 3 | Integration of M1 packet worktrees | **done** | `64b7c55` on `integration/ms-2026-08-control-restore`, 2026-08-21T19:14:51Z |
| 4 | M1 focused validation | **done** | five commands exited 0 at exact HEAD `64b7c55`; clean worktree before and after |
| 5 | Final independent audit (Grok 4.6 or Claude Opus 5, never Codex) | **ready** — not started | — |
| 6 | M1 exit and slot shift | **blocked** on node 5 | — |

Snapshot `eb4fd0c` is **preservation-only**: a 339-file, ~198,000-insertion commit of already-accumulated pre-M1 work, product code included, made so the classified worktree survived the session boundary. It is not a clean tree, not a release, not a validated baseline, and not evidence for any M1 exit criterion. No exit-gate command was run against it and it carries no `history.md` exit evidence.

Parallel lanes that ran, with disjoint write sets — kept as evidence that M1 was not a single serial queue:

| Lane | Write set | Outcome |
|---|---|---|
| Board/packet plan completion (this lane) | `docs/development/milestones.md`, `docs/development/ai/task-packets/milestone-control-restore.md` | `a7b750e` → integrated `64b7c55` |
| Independent baseline Grok 4.6 audit | `docs/development/ai/audit-latest.md` only | completed 2026-08-21T19:15:02Z; `stalled=yes` only from the pre-node-2 placeholders, `scope_creep=no`, four M1-card validations green |

The baseline audit is an observation lane, not node 5. It does not clear the M1 exit.

M1 remains **open**. This packet stays `in_progress`: nodes 1-4 done plus five green commands are the inputs to the final independent audit, not a substitute for it. Node 5 has not run, node 6 has not run, and no slot has shifted.

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

The node 4 result is pinned to tree `64b7c55`. This progress commit edits the same two docs files, so it is a **later tree**. Node 5 must audit the resulting commit, not `64b7c55`.

For this progress commit the docs-only lane re-runs:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check -- docs/development/milestones.md docs/development/ai/task-packets/milestone-control-restore.md
```

`bash -n /workspace/scripts/dev-agent` and `/workspace/scripts/test-dev-agent.sh` target `/workspace/scripts/` and are unaffected by these two files, so the node 4 results carry over. `run_milestone_audit.py` reads this board and will render the updated graph; it is not re-run by this lane.

## Handoff

- files changed;
- design summary;
- validation result;
- risks;
- follow-up.
- task/packet digest and resulting patch digest when automation provides them.
