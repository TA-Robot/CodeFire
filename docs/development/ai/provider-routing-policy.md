# Milestone Provider Routing Policy

Status: active development policy  
Effective: 2026-08-21  
Applies to: external AI development of CodeFire, not the CodeFire product runtime

## Decision

Provider selection happens before provider-specific model/effort selection.

- Codex `gpt-5.6-sol` is the **orchestrator only**: milestone admission, packets, lane assignment, integration.
- Individual agents (implement, review, docs, validation, dogfood) use the strongest available **Claude or Grok**: `claude-opus-5` or `grok-4.6`.
- The 30-minute **audit agent** also uses Claude or Grok, never Codex, and should be the other family from the current implementer.
- Isolated worktree execution through `dev-agent` defaults to Claude Opus 5 because that backend exists today. Grok 4.6 is the preferred Grok Build / `grok -m grok-4.6` worker.
- Fugu is overflow only. It is not a default individual agent and not the auditor.
- Every Codex execution uses `gpt-5.6-sol` and requires explicit `--allow-codex`.
- A worker failure never automatically falls back to Codex.

`provider-routing-matrix.json` is authoritative for provider, concurrency, timeout, and fallback automation.
`orchestration-protocol.md` is authoritative for the three-role topology.
`role-routing-matrix.json` and `model-routing-policy.md` remain authoritative for Codex reasoning semantics when the orchestrator lane itself runs.

## Why

Unbounded review burn-down and extra test gates stalled CodeFire. Codex also remains a scarce control plane. The replacement topology is small on purpose:

1. Codex orchestrates only the next milestone.
2. Claude Opus 5 and Grok 4.6 do the work.
3. An independent Claude or Grok auditor fires every 30 minutes to stop stall and scope creep.
4. Review may only require `block-m1` findings. Everything else is parked on M2–M5.
5. A manager/human inspects the integration branch. Automation never mutates main.

## Runtime Entry Points

The devcontainer base provides:

- `dev-agent`: one provider-routed task lane;
- `dev-swarm`: a long-running background DAG with provider concurrency and worktree isolation;
- `claude-second-agent`, `codex-second-agent`: isolated worktree adapters;
- Grok 4.6 via Grok Build subagents or `grok -m grok-4.6` (not a second-agent backend in this milestone);
- `fugu-second-agent`: overflow only.

Initialize all providers once:

```bash
dev-agent workspace init /workspace/project
dev-agent doctor
```

`doctor` reports availability without printing credentials. Claude `auth status` is advisory because an
interactive session or credential helper can differ from that report: if the CLI exists, the router attempts
the actual worker and treats its exit as authoritative. Fugu credential sources are `SAKANA_API_KEY`,
`FUGU_API_KEY_FILE`, or `<target-workspace>/fugu-api`.

## Timeboxes Are Runaway Limits

Default development timeboxes are deliberately measured in hours:

| Lane | Limit |
|---|---:|
| Claude / Grok implementation | 6h |
| Claude / Grok review/research | 4h |
| Audit slice | 15m |
| guarded Codex orchestrator | 3h |
| Fugu overflow (explicit only) | 4h |

A timeout does not authorize discarding a worktree. Long tasks must create small commit checkpoints. On timeout
or provider outage, preserve the session, logs, branch, and commits, then resume or create a successor packet.
Minute-scale timeouts are only for connectivity/contract health checks.

## Parallel Batch Contract

`dev-swarm` manifests bind:

- base revision;
- task role, risk tier, provider override, and timeout;
- ticket path;
- dependency DAG;
- declared write set;
- whether a commit is required;
- optional machine-readable review result and required decision;
- explicit Codex authorization.

The scheduler rejects dependency cycles, unknown dependencies, overlapping parallel write sets, and dirty
main paths that intersect a task write set. Each task gets a distinct provider branch, worktree, session, and
log. Dependency branches seed later worktrees. A failed review result blocks integration.

Default concurrency is Claude 2, Grok 2, Codex 1. Fugu overflow stays at 1 and is opt-in. Raising concurrency requires evidence that write sets remain disjoint
and the provider quota supports the increased rate.

## Escalation

A Claude or Grok worker stops and returns a checkpoint when the packet is done, when it hits a parked issue,
or when an Ultra invariant appears. It does not start later-milestone work.

Worker-to-Codex rerouting is not automatic and is not used for implementation. The orchestrator must:

1. record why Codex must **orchestrate** rather than a worker continuing;
2. narrow the remaining question to an M1 packet;
3. preserve existing branch/evidence;
4. keep implementation on Claude or Grok;
5. use `gpt-5.6-sol` only on the orchestrator lane with `--allow-codex`.

Provider availability, quota, or authentication failure is not evidence that the task itself failed.

## Integration

Implementation and review never share an agent/session. Review approval must be machine-readable when used as a
DAG gate. Successful agent exit alone is not approval.

The last automated output is an integration branch plus a collection manifest containing task status, provider,
branch, head revision, commit count, worktree, log, and message. Main merge remains manager/human-owned.
