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

## Handoff

- files changed;
- design summary;
- validation result;
- risks;
- follow-up.
- task/packet digest and resulting patch digest when automation provides them.
