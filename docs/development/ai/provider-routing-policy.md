# Low-Codex Provider Routing Policy

Status: active development policy  
Effective: 2026-07-29  
Applies to: external AI development of CodeFire, not the CodeFire product runtime

## Decision

Provider selection happens before provider-specific model/effort selection.

- Medium, bounded, strongly verifiable work defaults to Sakana Fugu.
- High implementation, debugging, dogfood, and independent review default to Claude.
- Codex is reserved for Ultra architecture/invariant decisions, integration, conflicting evidence, and final completion audits.
- Worker defaults are pinned to Fugu `fugu-ultra-v1.1` and Claude `claude-opus-5`.
- Every Codex execution uses `gpt-5.6-sol` and requires explicit `--allow-codex`.
- A worker failure or missing credential never automatically falls back to Codex.

`provider-routing-matrix.json` is authoritative for provider, concurrency, timeout, and fallback automation.
`role-routing-matrix.json` and `model-routing-policy.md` remain authoritative for the reasoning semantics used
when a task is deliberately routed to Codex.

## Why

Codex usage credit has repeatedly become a single point of development stoppage. Long implementation and
review sessions also make poor control-plane work: they consume the scarce manager context while independent
lanes wait.

The new topology keeps the expensive/scarce control plane small:

1. Codex or a human manager fixes task packets, write sets, dependency order, and acceptance evidence.
2. Fugu performs narrow, deterministic evidence and validation work in parallel.
3. Claude owns multi-hour implementation and independent review.
4. Fugu runs deterministic post-review gates.
5. Codex adjudicates only unresolved Ultra risk and prepares an integration branch.
6. A manager/human inspects evidence and merges; automation never mutates main directly.

## Runtime Entry Points

The devcontainer base provides:

- `dev-agent`: one provider-routed task lane;
- `dev-swarm`: a long-running background DAG with provider concurrency and worktree isolation;
- `fugu-second-agent`, `claude-second-agent`, `codex-second-agent`: low-level provider adapters.

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
| bounded Fugu evidence/validation | 2h |
| Fugu development | 4h |
| Claude implementation/debug/dogfood | 6h |
| Claude review/research | 4h |
| guarded Codex integration/final | 3h |

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

Default concurrency is Fugu 4, Claude 2, Codex 1. Raising it requires evidence that write sets remain disjoint
and the provider quota supports the increased rate.

## Escalation

Fugu stops and returns evidence when a deterministic packet becomes ambiguous. The manager may explicitly
reroute it to Claude. Claude stops and returns a checkpoint when an Ultra invariant, security, migration,
cross-subsystem, or release-completion decision appears.

Claude-to-Codex rerouting is not automatic. The manager must:

1. record the Ultra reason;
2. narrow the remaining question;
3. preserve existing branch/evidence;
4. create an explicit Codex lane with `--allow-codex`;
5. use `gpt-5.6-sol`.

Provider availability, quota, or authentication failure is not evidence that the task itself failed.

## Integration

Implementation and review never share an agent/session. Review approval must be machine-readable when used as a
DAG gate. Successful agent exit alone is not approval.

The last automated output is an integration branch plus a collection manifest containing task status, provider,
branch, head revision, commit count, worktree, log, and message. Main merge remains manager/human-owned.
