# AI Operating Protocol

This protocol is the single human-readable operating guide for AI task packets, review packets, and parallel lanes.

Use it with:

- `context-packs.json` for bounded reading;
- `schemas/` for the authoritative machine packet contracts;
- `role-routing-matrix.json` for machine-readable role and reasoning-effort selection;
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

Keep stable role semantics separate from the versioned runtime selection. The current runtime policy explicitly selects `gpt-5.6-sol`; CodeFire product semantics must not depend on that model.

| Role | Default tier | Use for | Required output |
|---|---|---|---|
| Manager | Ultra | batch planning, lane assignment, integration, conflicting evidence, completion audit | task packets, integration plan, completion audit |
| Architect | Ultra | module boundaries, object/branch invariants, transaction/security design | alternatives, design note, falsification plan |
| Implementer | High | focused code or docs changes with bounded scope | patch, focused tests, handoff |
| Reviewer | Ultra | adversarial correctness, invariant, contract, and evidence review | approve/request/block |
| Researcher | High | bounded exploration with evidence and competing hypotheses | evidence, alternatives, decision candidates |
| Docs synchronizer | Medium | source-backed tables, links, current-state synchronization | docs diff, source evidence |
| Validation runner | Medium | deterministic commands and exact log summaries | validation table |
| Dogfood runner | High | algorithm workflow, reproducible friction, CFB discovery | dogfood evidence |
| Classifier | Medium | dirty tree grouping and declared ownership classification | classification table |

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

Parallelism is safe only when lanes have non-overlapping write sets or a manager owns final integration.

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
5. `role-routing-matrix.json` for machine-readable routing.

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

`run_ai_role.py` defaults to a non-executing manifest and read-only sandbox. Execution and `workspace-write` are separate explicit choices. The Codex repository sandbox cannot enforce an individual packet's write-set, so write execution also requires explicit acknowledgement of that limitation and independent diff review.

The runner reconstructs the packet's content-addressed context from its exact packs, additional files, and all currently existing explicit read/write inputs. A declared manifest must match that reconstruction exactly. It binds the digest into the run manifest and verifies live file bytes before execution; missing read inputs block execution and remain visible in dry-run output. Typed execution rejects an extra `--prompt`, a stale `base_revision`, and any existing worktree change intersecting a workspace-write packet's declared write set. There is no broad dirty-write override: first split/save the existing work or create a future packet contract that binds each inherited file by digest.

Validation requirements have two phases. Executable, compound, and derived capabilities are preflight requirements. Evidence capabilities such as a Rust binary bound to current source are post-validation requirements: they do not prevent investigation from starting, but their missing evidence prevents review approval.
