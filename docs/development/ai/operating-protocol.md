# AI Operating Protocol

This protocol is the single human-readable operating guide for AI task packets, review packets, and parallel lanes.

Use it with:

- `context-packs.json` for bounded reading;
- `schemas/` for the authoritative machine packet contracts;
- `role-routing-matrix.json` for machine-readable role and reasoning-effort selection;
- `provider-routing-matrix.json` for machine-readable Fugu/Claude/Codex selection and concurrency;
- `provider-routing-policy.md` for Low-Codex fallback, timebox, swarm, and integration semantics;
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

Keep stable role semantics separate from provider and model runtime selection. The default external topology uses Fugu for Medium evidence, Claude for High implementation/review, and guarded Codex `gpt-5.6-sol` for Ultra integration/final decisions. CodeFire product semantics must not depend on any provider.

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

Provider routing is applied after role/risk selection:

| Selected tier | Default provider | Execution shape |
|---|---|---|
| Medium | Fugu `fugu-ultra-v1.1` | bounded packet, strong oracle, 2h default |
| High | Claude `claude-opus-5` | multi-hour worktree, checkpoint commits, independent review |
| Ultra | Codex `gpt-5.6-sol` | explicit `--allow-codex`, one control-plane lane, integration/final evidence |

No unavailable worker automatically falls back to Codex. Fugu-to-Claude rerouting is explicit; Claude-to-Codex requires a recorded Ultra reason and a new guarded lane.

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

For multi-hour batches, use the external `dev-swarm` scheduler. It binds the
base revision, task tickets, DAG, write sets, provider concurrency, expected
commits, and optional machine-readable review decision. Fugu 4, Claude 2,
and Codex 1 are initial provider windows; they do not add up to a global
session cap. External Fugu/Claude windows adapt one lane at a time from clean
completion or provider-backpressure evidence, while Codex remains one
cost-controlled manager lane. The manual portfolio separately runs
`check_swarm_admission.py` for host resources and conflict ownership. The
scheduler creates an integration branch but never merges main automatically.

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

## Successor Review Recurrence Gate

Use recurrence control v2 for the current successor-review candidate. The v1
schema and validator remain historical compatibility evidence only: passing a
v1 record to the normal checker always returns a structured `BLOCK`. Never
rewrite an accepted packet or historical review document merely to make it v2.

The gate addresses three different failure modes without treating them as one
undifferentiated `REQUEST_CHANGES` loop:

1. a prior obligation was never actually closed;
2. the repair introduced a new effect or review surface;
3. the earlier review, oracle, scope, or communication missed the effect.

Before repair starts, the manager commits a Markdown repair packet and
`review-recurrence-repair-manifest.v1` together. The packet explains each prior
obligation's acceptance criterion, authorized boundary, production routes,
anticipated regressions, and forbidden shortcuts. The manifest freezes the
exact obligation, parity/closure-claim, and required-validation sets. The gate
proves equality to those sets, but their semantic completeness remains a
manager and independent-preflight judgement. The implementer may not replace
them with self-authored closure claims.

The v2 record binds repair packet, manifest, candidate, review packet, report,
timed evidence, and terminal event as exact regular Git blobs at
role-compatible commits. Git topology separates the pre-repair packet commit,
candidate, review-control commit, report-only commit, and evidence commit. A
post-candidate prompt cannot masquerade as the pre-repair contract, and this
generation's review commits cannot self-author prior closure evidence.

The independent reviewer inspects every prior obligation, then every exact
changed path and its candidate-created surfaces. There is no no-surface or
out-of-scope escape: each changed path has at least one reviewed surface.
`referred_to_manager` binds an earlier routing decision and remains blocking.
Evidence is classified with the v2 evidence-state/next-action vocabulary;
`CLOSED_PROVEN` with `NONE` is the only potentially nonblocking pair and still
needs the separately derived closure proof. A new finding cannot be closed in
the generation that first records it.

There is deliberately no minimum or target finding count. Required validations
come only from the frozen manifest and must bind committed
`codefire.timed_gate.v2` results. Run them from the clean candidate with both
the live log and result outside the source worktree, then copy those bytes
unchanged into the evidence commit. Acceptance requires the exact candidate
revision/tree, clean state before and after, no index flags, complete log
bytes/digest, exact argv, unchanged post-command `HEAD`/tree, terminal exit,
stopped reviewer, zero child processes, and committed terminal-event evidence.
The index check is the union of its before/after observations, so clearing a
pre-existing flag inside the command does not hide it. Mutate-and-restore
behavior inside the command remains outside this proof.

Authority is progressive. A legacy-bootstrap v2 generation derives
predecessor completeness as `unavailable`, always blocks, and exists only to
seed the next generation. A later record must resolve the complete retained v2
record/result chain to the manager-fixed Cargo or KGI/KPI root. The current
gate replays every retained generation oldest-first using current schema,
artifact, manifest, timed-evidence, and decision semantics. Retained results
are comparison evidence, never trusted predecessor state; replay does not
claim that older gate implementations were semantically identical. For a live
arm, replay also reloads each generation's exact anchor blob from its historical
deciding-gate commit and requires that commit to remain in the current validator
ancestry; a retained pin cannot substitute for that rebind.

Run the decision from a clean committed validator checkout. `--git-root` is
the analysed candidate repository; `--validator-root` is the separate
manager-controlled source of the gate modules and schemas. Any dirty validator
state, uncommitted gate byte, index flag, redirected schema path, or validator
identity mismatch blocks. A manager-fixed chain also requires an explicit
matching `--live-arm cargo|kgi_kpi` and its committed validator-owned anchor.
No anchor is currently committed, so neither live arm can yet emit `ALLOW`.

After the reviewer has stopped, all child processes have ended, and the
report-only commit exists, the manager writes the final JSON record and runs:

```bash
python3 docs/development/tools/check_review_recurrence.py \
  --record <final-record.json> \
  --git-root <analysed-candidate-repository> \
  --validator-root <clean-validator-repository> \
  --live-arm cargo \
  --result <outside-both-repositories>/recurrence-result.json
```

Use `--live-arm kgi_kpi` for that arm. Only exit `0` with
`integration_decision: ALLOW` authorizes integration. v1, bootstrap generation,
`request_changes`, `block`, incomplete terminalization, unreviewed surfaces,
scope referrals, missing or in-tree timed evidence, missing live anchor,
mismatched Git identity, and `--no-git-verify` remain `BLOCK`.

The machine proves topology, bytes, declared set equality, legal state/action
pairs, and machine-visible closure predicates. Honest execution, semantic
oracle quality, nonzero selection, complete surface/symbol enumeration,
manifest completeness, authorship, and reviewer independence remain human
judgements. Machine `ALLOW` is required evidence, not a substitute for the
manager's semantic integration decision. Current status and promotion metrics
are in `review-recurrence/README.md`.

## Prompt Construction

Do not maintain separate role-specific prompt documents. Build prompts from:

1. the selected context pack;
2. the task or review packet;
3. this protocol;
4. `model-routing-policy.md` for risk and escalation;
5. `provider-routing-policy.md` and `provider-routing-matrix.json` for provider/concurrency/fallback;
6. `role-routing-matrix.json` for Codex-specific machine-readable reasoning routing.

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
