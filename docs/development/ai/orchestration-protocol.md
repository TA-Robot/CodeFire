# Orchestration Protocol

Status: canonical agent topology for CodeFire development

Owner: Orchestrator

Update trigger: a provider pin, role, or invocation path changes.

This protocol is an **external** development control plane. It is not a CodeFire product feature. Do not add agent scheduling, marketplaces, or prompt runtimes to `crates/` or `codefire`.

Companion docs:

- `../milestone-operating-model.md` — what may be worked;
- `../milestones.md` — the live five-ahead board;
- `audit-cadence.md` — the 30-minute auditor;
- `provider-routing-policy.md` — machine routing.

## Topology

```text
Codex gpt-5.6-sol
  orchestrator (one lane)
    ├─ individual agents: Claude Opus 5 and/or Grok 4.6
    │     implement, focused test, docs for one M1 packet each
    └─ audit agent: Claude Opus 5 or Grok 4.6, never Codex
          every 30 minutes, read-only advice, no implementation
```

Three roles, no others with write permission:

| Role | Provider | Model pin | Writes | Cadence |
|---|---|---|---|---|
| Orchestrator | Codex only | `gpt-5.6-sol` | packets, board, integration branch, refusal of parked work | continuous while M1 is open |
| Individual agent | Claude or Grok | `claude-opus-5` or `grok-4.6` | the packet write set only | until the packet exits or timebox |
| Audit agent | Claude or Grok, not Codex | `claude-opus-5` or `grok-4.6` | `audit-latest.md` only | every 30 minutes |

Fugu is overflow only. It is never the default individual agent, never the auditor, and never the orchestrator. Automatic Codex fallback remains forbidden.

## Strongest-model rule

Pick the strongest currently available model from **each** allowed family:

- Claude family: `claude-opus-5`
- Grok family: `grok-4.6`

Do not use weaker aliases (`claude-yolo`, `grok-4.5`, `grok-yolo`) for these roles.

When both families are available, split them for independence:

| If implementer is | Then reviewer and auditor are |
|---|---|
| Grok 4.6 | Claude Opus 5 |
| Claude Opus 5 | Grok 4.6 |

If only one family is available, use it for individual work **and** audit, but in a **different session and worktree**. Same-session self-audit is invalid.

## Orchestrator duties

The Codex orchestrator:

1. Reads `milestones.md` and names the M1 exit.
2. Draws the **critical path** to that exit and the **parallel set** of independent M1 packets. A single serial queue is not a plan.
3. Refuses any request whose files or issue IDs are parked.
4. Writes one task packet per parallel slice, each with a disjoint write set, non-goals, M1-scoped validation, and handoff shape.
5. Launches every independent packet now, up to Claude 2 / Grok 2. Never waits for the critical path to finish before starting work that does not depend on it. Never implements the product change itself.
6. Calls the audit agent at least every 30 minutes while any individual lane is running.
7. Accepts only `block-m1` review findings as required follow-up.
8. Prepares an integration branch. Never merges to `main`.

Codex `--allow-codex` is required for this lane. Using Codex as an implementer, reviewer, or auditor is a policy miss.

## Individual-agent duties

An individual agent:

- reads only the packet and its context pack;
- stays inside the write set;
- runs only the validation list on the packet;
- does not file new CFB/CFR issues unless they are M1 blockers, and even then files rather than expands scope;
- parks extra findings in the handoff instead of fixing them;
- stops when the packet objective is met.

## Invocation

### Orchestrator (Codex)

```bash
dev-agent workspace init /workspace/project
dev-agent --role manager --tier ultra --allow-codex --agent orchestrator-m1 \
  "Drive only MS-agent-local-contract. Launch the two ready read-only packets, fan them in, and admit no product fix before a reproduced M1 blocker. Call the auditor every 30 minutes."
```

### Individual agent — Claude (isolated worktree)

```bash
dev-agent --role implementer --tier high --agent impl-m1 "<packet prompt>"
dev-agent --role reviewer --tier high --agent rev-m1 "<review prompt>"
```

### Individual agent — Grok 4.6

Grok is not a `second-agent` backend in this milestone (adding a fourth protocol is parked). Launch it as a Grok Build subagent with model `grok-4.6`, or:

```bash
grok -m grok-4.6 --always-approve --cwd /workspace/project "<packet prompt>"
```

The packet prompt must include the write set, M1 non-goals, and the validation list. A Grok worker that starts extra review-portfolio work is out of policy.

### Audit agent

See `audit-cadence.md`. Preferred independent auditor is the family **not** used by the current implementer.

```bash
python3 docs/development/tools/run_milestone_audit.py
dev-agent --role auditor --tier high --agent audit-m1 "$(python3 docs/development/tools/run_milestone_audit.py --prompt)"
```

When the auditor is Grok, pass the same generated prompt to a Grok 4.6 subagent or `grok -m grok-4.6 --always-approve`.

## Timeboxes

Timeboxes are runaway limits, not estimates.

| Lane | Limit |
|---|---:|
| Codex orchestrator | 3h per session, then resume |
| Claude / Grok implementation | 6h |
| Claude / Grok review | 4h |
| Audit slice | 15 minutes; overtime is a stall signal |

On timeout: preserve the worktree and commits. Do not discard. Do not escalate to Codex to "just finish it".

## Concurrency

Default: Codex 1, Claude 2, Grok 2. Overlapping write sets are forbidden. Implementation and review never share a session.

Concurrency is a ceiling, not a reason to run one lane. If two M1 packets are independent and two worker slots are free, starting only one of them is a stall. If a third independent packet is ready and both slots are full, it waits — that is capacity, not a serial plan.

## Failures

| Event | Action |
|---|---|
| Claude unavailable | Use Grok 4.6 if available; otherwise stop the worker lane |
| Grok unavailable | Use Claude Opus 5 if available; otherwise stop the worker lane |
| Codex unavailable | Workers that already have packets may continue; no new M1 scope; no Codex fallback from workers |
| Auditor unavailable | Orchestrator records `audit-missed` and still must not expand scope |
| Auditor reports stall | Orchestrator cuts scope to the M1 exit, parks the rest, restarts one worker |
| Auditor reports scope creep | Orchestrator stops the offending lane immediately |
