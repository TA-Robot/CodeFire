# Milestone Operating Model

Status: canonical execution policy

Owner: Engineering Manager / Orchestrator

Update trigger: a milestone exits, a next-milestone blocker is admitted, or this control-plane policy itself changes.

This document replaces unbounded review burn-down and portfolio-wide testing as the way CodeFire development is scheduled. Strategy remains in `strategy/`. Issue facts remain in the CFB/CFR ledgers. **What may be worked now** is decided only here and on the live board `milestones.md`.

## Why this model exists

Recent work stalled because every review finding and every extra test gate could expand the active batch. The frozen review-portfolio batch required remaining roots to finish before new product motion. That is the failure mode this model forbids.

The operating rule is:

> Keep five milestones visible. Do only the work that the **next** milestone needs. Park everything else.
> For that next milestone, find the critical path and run every independent slice in parallel.

## Five-ahead board

The live board is `docs/development/milestones.md`. It always shows exactly five future (or current-plus-four) milestones:

| Slot | Meaning |
|---|---|
| M1 | The next exit. The only slot that may receive implementation, focused tests, or blocking review. |
| M2 | The following increment. Issues may be **queued**, not started. |
| M3–M5 | Queued increments. Capture deferred review and non-blocking issues. Do not implement. |

When M1 exits, every slot shifts forward one step and a new M5 is named. Do not keep a sixth live execution target.

## What may enter M1

An issue, review finding, test expansion, or docs change enters M1 only if **all** of the following are true:

1. M1's exit criteria cannot be met without it.
2. It is the smallest change that restores that exit criterion.
3. The orchestrator records it on `milestones.md` before work starts.

Examples that **do not** enter M1:

- a review comment that would be valuable later but does not block the M1 exit;
- a missing test that does not protect an M1 invariant;
- a performance, HTTP, remote, or architecture root that M1 does not name;
- a new CFB/CFR discovery that is not an M1 blocker;
- "while we are here" cleanups.

## Critical path and parallel plan

M1 admission is not a to-do list. The orchestrator must turn the M1 card into an execution graph before launching agents.

1. **Name the critical path.** The shortest dependency chain from "now" to the M1 exit. Only work that later M1 work waits on belongs on this path.
2. **Split the rest.** Every M1 item that does not wait on another unfinished M1 item is a parallel packet. Launch those immediately, up to provider concurrency, with disjoint write sets.
3. **Do not serialize by habit.** One agent finishing A before another starts independent B is a planning defect, not caution.
4. **Do not parallelize parked work.** Extra lanes are for M1 slices only. M2–M5 stay queued.
5. **Keep the path short.** If two packets share a file, either merge them, give the shared file to the orchestrator, or sequence only that edge — not the whole milestone.

The live board must show, for the active M1:

```text
Critical path: packet-a → packet-c → M1 exit
Parallel now: packet-b, packet-d
Blocked: packet-c waits on packet-a
```

A plan that lists M1 work as a single queue with no parallel set is incomplete. The auditor treats idle capacity plus an unstarted independent M1 packet as stall.

## Parking rule

Work that is real but not M1-blocking is parked with an explicit later slot:

```text
status: parked
milestone: M4
reason: needed for remote happy path, not for the local sealed loop
```

Parked items stay in the CFB/CFR ledgers. They are not `wontfix`. They are not active work. Agents must not implement parked items, must not expand tests for them, and must not treat them as review blockers of M1.

## Review rule

Reviewers may emit three classes of finding:

| Class | Meaning | Action |
|---|---|---|
| `block-m1` | M1 exit is false if this stands | Must fix before M1 close |
| `park` | Valid issue, not required for M1 | Assign M2–M5 and stop |
| `drop` | Out of product boundary, duplicate, or noise | Do not file |

A review that lists more than the M1-blocking findings as required work is itself a process violation. Nice-to-have comments are parked in one line each, not turned into a new batch.

Independent review remains required for product-code M1 work. The reviewer may not implement. The reviewer may not reopen parked roots.

## Validation rule

Validation is milestone-scoped.

- During M1: run the **narrowest command list named on the M1 card**. Focused tests for the touched surface are required. Workspace-wide `cargo test`, full Install, and dogfood Retrospect are **not** required unless the M1 card names them as the exit gate.
- At M1 exit: run only the exit-gate commands on that card.
- A skipped command is allowed when the M1 card says it is out of scope; record the skip. A skipped M1 exit-gate command is not an exit.
- Deterministic gates that **are** on the card still outrank model claims (`strategy/decision-register.md` STR-005).

This does not weaken sealed-commit, object-identity, or branch-head invariants. It stops using the entire historical test catalog as a daily admission ticket.

## Issue intake

1. Anyone may file a CFB/CFR detail.
2. The orchestrator classifies it onto M1 or M2–M5 within the same working day, or parks it as `unscheduled` if it is beyond M5.
3. Filing is not permission to implement.
4. No new Discover batch may start while M1 is open, except to admit an M1 blocker.
5. The old rule "close every open issue before the next discovery" is revoked. The replacement is "close or park every M1 item before starting M2".

## Agent topology

See `docs/development/ai/orchestration-protocol.md`.

| Role | Provider | Allowed work |
|---|---|---|
| Orchestrator | Codex `gpt-5.6-sol` only | Choose M1 work, draw the critical path, fan out independent packets, refuse parked work, integrate |
| Individual agents | strongest available Claude or Grok | Implement, focused test, docs sync for the assigned M1 packet only |
| Audit agent | strongest available Claude or Grok, never Codex | Every 30 minutes, compare plan vs progress, detect stall and scope creep |

Codex does not implement M1 product changes and does not act as the 30-minute auditor. Individual and audit agents never silently fall back to Codex.

## Handoff

Every M1 packet and every audit note must answer:

```text
Active milestone:
M1 exit criteria:
Critical path:
This packet is on the path / parallel to the path:
This change is required because:
Parked instead of done:
Independent packets still unstarted:
Validation actually run:
Stall risk:
Next allowed action:
```
