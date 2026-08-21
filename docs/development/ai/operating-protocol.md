# AI Operating Protocol

This protocol is the single human-readable operating guide for AI task packets, review packets, and parallel lanes.

Use it with:

- `context-packs.json` for bounded reading;
- `schemas/` for the authoritative machine packet contracts;
- `role-routing-matrix.json` for machine-readable role and reasoning-effort selection;
- `provider-routing-matrix.json` for machine-readable Fugu/Claude/Codex selection and concurrency;
- `provider-routing-policy.md` for milestone routing, timebox, swarm, and integration semantics;
- `orchestration-protocol.md` for the Codex / Claude / Grok topology;
- `audit-cadence.md` for the 30-minute auditor;
- `model-routing-policy.md` for Medium/High/Ultra semantics and escalation;
- `task-packets/` for ready-to-delegate concrete work.

## Core Rule

Reduce ambiguity before increasing parallelism.

No AI lane should start until it has:

- objective;
- context pack;
- allowed and disallowed files;
- non-goals;
- invariants;
- validation commands;
- handoff expectations.

## Role Routing

Keep stable role semantics separate from provider and model runtime selection. The default external topology uses Codex `gpt-5.6-sol` as the orchestrator, Claude Opus 5 / Grok 4.6 as individual agents, and a Claude or Grok auditor every 30 minutes. CodeFire product semantics must not depend on any provider. See `orchestration-protocol.md`.

| Role | Default tier | Use for | Required output |
|---|---|---|---|
| Manager / orchestrator | Ultra | M1 admission, packets, lane assignment, parking, integration | task packets, milestone board, integration plan |
| Architect | Ultra | module boundaries, object/branch invariants, transaction/security design | alternatives, design note, falsification plan |
| Implementer | High | focused M1 code or docs changes | patch, focused tests, handoff |
| Reviewer | High | `block-m1` vs `park` vs `drop` only | block-m1 findings, parked remainder |
| Researcher | High | bounded exploration that M1 actually named | evidence, alternatives, decision candidates |
| Docs synchronizer | Medium | source-backed tables, links, current-state synchronization | docs diff, source evidence |
| Validation runner | Medium | the M1 validation list only | validation table |
| Dogfood runner | High | algorithm workflow needed for the active milestone | dogfood evidence, parked CFB candidates |
| Classifier | Medium | dirty tree grouping and declared ownership classification | classification table |
| Auditor | High | 30-minute plan vs progress, stall and scope-creep | `audit-latest.md` |

Provider routing is applied after role/risk selection:

| Selected tier | Default provider | Execution shape |
|---|---|---|
| Medium | Claude `claude-opus-5` (alternate Grok `grok-4.6`) | M1-scoped packet, isolated worktree or Grok Build subagent |
| High | Claude `claude-opus-5` (alternate Grok `grok-4.6`) | multi-hour worktree, checkpoint commits, independent review/audit |
| Ultra | Codex `gpt-5.6-sol` | explicit `--allow-codex`, one orchestrator lane, no product implementation |

No unavailable worker automatically falls back to Codex. Claude and Grok may substitute for each other. Auditor must not be Codex.

Escalate Implementer, Researcher, Dogfood, or Docs work when the task crosses an invariant, migration, security, deletion, or release-completion boundary. Apply the score and mandatory escalation rules from `model-routing-policy.md`.

`Ultra` includes both maximum available reasoning and proactive delegation where the surface supports it. Direct API `max` without an external manager is not equivalent. When a model or reasoning surface changes, update and benchmark the runtime adapter without silently changing the role taxonomy.

## Task Packet Shape

Machine execution should use `schemas/task-packet.v1.schema.json`. This Markdown shape is a human view:

~~~markdown
# AI Task Packet

Status:

Packet ID:
Schema version:
Base revision:
Risk score:
Reasoning tier:

## Objective

One sentence describing the exact result expected.

## Context

- Required context pack(s):
- Additional files:
- Related issue or observation:

## Scope

Allowed files:

- 

Disallowed files:

- 

## Non-Goals

- 

## Invariants

- object identity:
- sealed commit validation:
- branch head atomicity:
- remote mutation policy:
- JSON envelope stability:
- path/workspace safety:
- module boundaries:

## Validation

```bash

```

If a command cannot run, report the exact command, failure, environment reason, and review risk.

## Handoff

- files changed;
- design summary;
- validation result;
- risks;
- follow-up.
- task/packet digest and resulting patch digest when automation provides them.
~~~

## Review Packet Shape

Machine review records use `schemas/review-packet.v1.schema.json`. This is the human view:

Each validation evidence row must carry the task packet's `validation_id`, exact command, status, and summary. Approval additionally requires a log digest; post-validation requirements such as source-bound binary provenance require a repository-contained artifact path and its digest. Approval is invalid unless the task ID/digest/base revision match, canonical Git base/head resolve, the exact Git changed-file set stays inside the write set, the canonical patch digest matches, and every required validation has matching passed evidence. `block` and `request_changes` may record unavailable gates without fabricating approval-only provenance.

~~~markdown
# AI Review Packet

## Review Target

- Diff or branch:
- Claimed objective:
- Related task packet:
- Task packet digest:
- Base/head/patch digest:
- Reviewer independence:

## Required Context

- Context pack(s):
- Files changed:
- Files to inspect first:

## Review Questions

- Does the diff solve the stated objective?
- Did it touch files outside scope?
- Did it mix product, management, and dogfood changes?
- Are object identity, sealed commits, branch atomicity, JSON envelopes, and path safety preserved?
- Are parse, validate, plan, apply, and render responsibilities still separated?
- Are skipped validations justified?

## Reviewer Output

- decision: `approve`, `request_changes`, or `block`;
- strongest correctness concern;
- strongest maintainability concern;
- validation assessment;
- required follow-up.
~~~

## Parallel Lane Rules

The orchestrator must plan M1 as a critical path plus a parallel set. Independent M1 packets start together. Parallelism is then safe only when lanes have non-overlapping write sets or a manager owns final integration. Serializing independent M1 work "to keep it simple" is a planning defect.

For multi-hour batches, use the external `dev-swarm` scheduler. It binds the base revision, task tickets, DAG,
write sets, provider concurrency, expected commits, and optional machine-readable review decision. Its default
limits are Fugu 4, Claude 2, Codex 1. It creates an integration branch but never merges main automatically.

| Lane | Writes allowed | Conflict risk |
|---|---|---|
| Manager | Plans, management docs, final conflict fixes. | High for global files. |
| Researcher | Exploration files assigned in the packet; usually no canonical docs. | Low when each researcher owns a distinct note. |
| Implementer | Only files allowed by the task packet. | Depends on packet scope. |
| Reviewer | Usually none; review notes only. | Low. |
| Docs synchronizer | Docs listed in the packet. | Medium when current-state docs change. |
| Dogfood runner | Dogfood project files or CFB candidates only. | Medium. |
| Validation runner | Validation log or handoff notes only. | Low. |

Serialize edits to global state files:

- `docs/development/history.md`;
- `docs/development/current-cycle-burn-down-plan.md`;
- `subprojects/algorithm-evolution-agent-lab/codefire.links.yaml`;
- `crates/codefire-cli/src/main.rs`;
- `crates/codefire-cli/src/tests.rs`.

If multiple lanes need one of these files, the manager writes the final version.

## Review Severity

| Severity | Meaning | Action |
|---|---|---|
| Block | Can corrupt repo state, bypass sealed validation, break object identity, or lose user work. | Do not merge. |
| High | Breaks CLI contract, JSON envelope, path safety, or core workflow. | Fix before merge. |
| Medium | Maintainability, docs mismatch, missing focused test, unclear diagnostics. | Usually fix before merge. |
| Low | Wording, small cleanup, optional extra test. | Track or fix opportunistically. |

## Handoff Format

Every AI lane returns:

```text
Role:
Objective:
Files changed:
Validation:
Risks:
Next:
```

Manager-owned integration then records final validation and history updates once.

## Prompt Construction

Do not maintain separate role-specific prompt documents. Build prompts from:

1. the selected context pack;
2. the task or review packet;
3. this protocol;
4. `model-routing-policy.md` for risk and escalation;
5. `provider-routing-policy.md` and `provider-routing-matrix.json` for provider/concurrency/fallback;
6. `orchestration-protocol.md` and `audit-cadence.md` for Codex orchestrator vs Claude/Grok workers;
7. `role-routing-matrix.json` for Codex-specific machine-readable reasoning routing.

Keep common instructions and stable context at the beginning so prompt caching can reuse a stable prefix. Put task-specific packet data and volatile repository evidence at the end.

Resolve inherited context files and bind their current content with:

```bash
python3 docs/development/tools/resolve_context_pack.py pack-0-universal-orientation
```

Validate and dry-run a machine packet before granting write access:

```bash
python3 docs/development/tools/check_ai_contracts.py
python3 docs/development/tools/run_ai_role.py \
  --packet docs/development/ai/task-packets/json/branch-list-timeout.json
```

`run_ai_role.py` is a Codex-specific low-level adapter. It defaults to a non-executing manifest and read-only sandbox; every real run additionally requires `--allow-codex` under the Low-Codex policy. Execution and `workspace-write` are separate explicit choices. The Codex repository sandbox cannot enforce an individual packet's write-set, so write execution also requires explicit acknowledgement of that limitation and independent diff review. Routine Fugu/Claude work uses `dev-agent` or `dev-swarm` instead.

The runner reconstructs the packet's content-addressed context from its exact packs, additional files, and all currently existing explicit read/write inputs. A declared manifest must match that reconstruction exactly. It binds the digest into the run manifest and verifies live file bytes before execution; missing read inputs block execution and remain visible in dry-run output. Typed execution rejects an extra `--prompt`, a stale `base_revision`, and any existing worktree change intersecting a workspace-write packet's declared write set. There is no broad dirty-write override: first split/save the existing work or create a future packet contract that binds each inherited file by digest.

Validation requirements have two phases. Executable, compound, and derived capabilities are preflight requirements. Evidence capabilities such as a Rust binary bound to current source are post-validation requirements: they do not prevent investigation from starting, but their missing evidence prevents review approval.
