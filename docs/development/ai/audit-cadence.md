# 30-Minute Audit Cadence

Status: canonical

Owner: Orchestrator

Update trigger: cadence, auditor provider, or stall definition changes.

The audit agent exists to **prevent stall**, not to create more work. It never implements. It never files a new review portfolio. It never promotes parked issues into M1.

## Cadence

While an M1 individual lane is running, the Codex orchestrator must invoke the audit agent at least once every 30 minutes. The clock starts at the first individual-agent launch.

If the orchestrator session cannot host a timer, a human or Grok Build scheduler may fire the same prompt. The auditor is still not Codex.

A missed tick is recorded as `audit-missed`. Two missed ticks in a row is a stall.

## Who runs it

| Preference | Auditor |
|---|---|
| Implementer is Grok 4.6 | Claude Opus 5 |
| Implementer is Claude Opus 5 | Grok 4.6 |
| Only one family available | That family, new session, read-only |

Codex is forbidden as auditor.

## What the auditor reads

Generate the brief with:

```bash
python3 docs/development/tools/run_milestone_audit.py
```

The auditor then reads, and does not expand beyond:

- `docs/development/milestones.md`
- `docs/development/milestone-operating-model.md`
- `docs/development/ai/orchestration-protocol.md`
- `git status --short --branch` at `/workspace/project`
- the active task packet, if named on the board
- the previous `docs/development/ai/audit-latest.md` if present

Do not reread the CFB/CFR corpus. Do not reopen CFR-168.

## Required output

Write `docs/development/ai/audit-latest.md` with this shape:

```markdown
# Milestone audit

- time (ISO, if known):
- active milestone:
- stalled: yes/no
- scope_creep: yes/no
- next_milestone_risk:

## Evidence

- git:
- packets in flight:
- validation actually run:

## Advice (at most five bullets)

- 
```

Advice must be about **how to reach M1**, not new product work. Legal advice classes:

- cut the current packet to the exit criterion;
- park a finding that is not `block-m1`;
- start an independent M1 packet that is sitting idle while a worker slot is free;
- restart one worker with a narrower write set;
- stop a lane that is implementing a parked issue;
- run the M1 validation list and close M1 if already met.

Forbidden advice:

- "finish the remaining review roots first";
- "run the full workspace test suite before continuing";
- "keep Codex implementing";
- adding M2–M5 work to the current lane.

## Stall definition

The work is stalled when any of these hold:

- more than 30 minutes of individual-agent time with no commit, packet update, or explicit blocker;
- the active lane is implementing a parked issue;
- an independent M1 packet is unstarted while a worker slot is free;
- M1 work is queued as one serial list with no critical path / parallel set on the board;
- review output treats `park` findings as required;
- validation wider than the M1 card is blocking merge;
- two consecutive audit ticks were missed.

The orchestrator must respond to stall by reducing scope, not by adding agents.
