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
Parallel now: board/packet plan-completion packet, independent baseline audit writing audit-latest.md
Blocked: final independent audit waits on integration; M1 exit and slot shift wait on the final audit and on every validation named on the M1 card
```

Node states:

| # | Node | State |
|---|---|---|
| 1 | Classified-worktree snapshot `eb4fd0c` | done, preservation-only |
| 2 | Board and packet plan completion | in progress |
| 3 | Integration of M1 packet worktrees | not started |
| 4 | M1 focused validation | not started |
| 5 | Final independent audit (Grok 4.6 or Claude Opus 5, never Codex) | not started |
| 6 | M1 exit and slot shift | not started |

Snapshot `eb4fd0c` is **preservation-only**: a 339-file, ~198,000-insertion commit of already-accumulated pre-M1 work, product code included, made so the classified worktree survived the session boundary. It is not a clean tree, not a release, not a validated baseline, and not evidence for any M1 exit criterion. No exit-gate command was run against it and it carries no `history.md` exit evidence.

Parallel lanes and their disjoint write sets:

| Lane | Write set |
|---|---|
| Board/packet plan completion (this lane) | `docs/development/milestones.md`, `docs/development/ai/task-packets/milestone-control-restore.md` |
| Independent baseline audit | `docs/development/ai/audit-latest.md` only |

The baseline audit is an observation lane, not node 5. It does not clear the M1 exit.

M1 remains **open**. This packet stays `in_progress`; completing node 2 is not an M1 close and does not shift slots.

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

Lane scoping for node 2 (docs-only, two files, no product code): this lane runs

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check -- docs/development/milestones.md docs/development/ai/task-packets/milestone-control-restore.md
```

`run_milestone_audit.py`, `bash -n /workspace/scripts/dev-agent`, and `/workspace/scripts/test-dev-agent.sh` are **not** run by this lane and remain unsatisfied for the packet as a whole. They belong to node 4 (M1 focused validation) after integration. The full list above is still the packet's exit contract; node 2 does not discharge it.

## Handoff

- files changed;
- design summary;
- validation result;
- risks;
- follow-up.
- task/packet digest and resulting patch digest when automation provides them.
