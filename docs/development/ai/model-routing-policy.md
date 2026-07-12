# GPT-5.6 Sol Model Routing Policy

Status: active development policy  
Last verified locally: 2026-07-11  
Applies to: external AI development of CodeFire, not the CodeFire product runtime

## Decision

CodeFire development uses `gpt-5.6-sol` as the explicit primary model and exposes three project-level reasoning tiers:

- `Medium`
- `High`
- `Ultra`

The tier is selected from task risk, uncertainty, blast radius, reversibility, and verification cost. It is not a prestige label and must not be chosen only because a task is large.

## Why The Policy Has An Adapter

The project-level word `Ultra` is a stable intent: use the strongest available reasoning configuration for work whose mistakes can invalidate many downstream decisions.

Runtime vocabularies currently differ:

| Surface | Medium | High | Strongest CodeFire tier |
|---|---|---|---|
| CodeFire policy | `Medium` | `High` | `Ultra` |
| Current local Codex CLI 0.144.1 | `medium` | `high` | `ultra` |
| Public GPT-5.6 Sol API model page | `medium` | `high` | `max` |
| Portable Codex fallback | `medium` | `high` | `xhigh` |

Local evidence:

- `codex doctor --json` reports the project configuration loaded with `gpt-5.6-sol`.
- the installed Codex 0.144.1 bundle contains the GPT-5.6 Sol catalog entry and the `Ultra` description “Maximum reasoning with automatic task delegation”.

Official references:

- GPT-5.6 Sol model: <https://developers.openai.com/api/docs/models/gpt-5.6-sol>
- Codex configuration reference: <https://developers.openai.com/codex/config-reference>
- Codex advanced configuration and profile layering: <https://developers.openai.com/codex/config-advanced>
- Codex subagents and Ultra delegation behavior: <https://developers.openai.com/codex/subagents>

Because the public Codex config reference currently documents through `xhigh` while the installed Codex catalog and official subagent guide expose `Ultra`, automation must validate the local capability and retain a fallback. A `--version` command is not capability proof because it does not necessarily validate override semantics. Never silently change an `Ultra` task to `High`.

## Routing Inputs

Score the task before delegation.

| Dimension | 0 | 1 | 2 |
|---|---|---|---|
| Blast radius | one isolated file | one module or doc family | cross-module, repository-wide, external contract |
| Reversibility | trivial rollback | moderate state cleanup | format, migration, remote, or user-data risk |
| Ambiguity | deterministic | alternatives need judgment | problem or success criterion is uncertain |
| Invariant density | no core invariant | one known invariant | identity, atomicity, security, consistency interactions |
| Verification cost | cheap exact check | integration test | incomplete oracle, long-running or production-like evidence |
| Downstream multiplier | no dependent lane | one dependent task | plan/review affects many lanes or releases |

Default routing:

- total `0..3`: Medium
- total `4..7`: High
- total `8..12`: Ultra

Mandatory escalation overrides the score.

## Medium

Use for bounded, deterministic work with a strong oracle.

Typical work:

- run validation and summarize exact output;
- classify files using declared ownership rules;
- update one source-backed table or link set;
- mechanical formatting or schema-preserving conversion;
- gather repository facts without making a design decision;
- execute a pre-approved dogfood smoke.

Required controls:

- explicit input paths;
- exact command/result evidence;
- no expansion outside the packet;
- escalate ambiguity instead of guessing.

Medium must not decide object format, security policy, architecture, deletion scope, release completion, or conflicting requirements.

## High

Use for non-trivial implementation and synthesis where the objective is known but the solution requires judgment.

Typical work:

- implement a focused feature or bug fix;
- perform root-cause analysis with a reproducible failure;
- consolidate a bounded document family;
- design focused tests and failure diagnostics;
- investigate performance and propose measured optimization;
- synchronize source, CLI contract, runbook, and tests after a known change;
- perform routine dogfood discovery.

Required controls:

- task packet with invariants and non-goals;
- focused tests before broad tests;
- handoff listing intentionally untouched files;
- independent review for product code or canonical policy changes.

## Ultra

Use for decisions and audits where a plausible but wrong answer can waste multiple lanes, corrupt state, weaken trust, or falsely claim completion.

Mandatory Ultra work:

- product architecture and module-boundary changes spanning multiple subsystems;
- canonical object identity, sealed commit, branch head atomicity, migration, remote mutation, authentication, or signature design;
- deletion or consolidation of large canonical document families;
- security and privacy threat-model decisions;
- concurrency, transaction, recovery, and corruption handling;
- release/readiness completion audit;
- multi-agent batch planning with overlapping global files;
- adjudicating conflicting reviewers or contradictory sources;
- deciding whether CodeFire's product boundary should change;
- changing this routing policy.

Ultra is not “one agent thinks longer.” The preferred execution shape is:

1. Ultra manager/architect defines the question, invariants, alternatives, and falsification tests.
2. High implementer performs bounded changes.
3. Ultra reviewer challenges the result independently.
4. Medium validation runner executes deterministic gates.
5. Ultra manager performs the requirement-by-requirement completion audit.

## Escalation And De-escalation

Escalate immediately when:

- a required source of truth conflicts with runtime behavior;
- the write set expands beyond the task packet;
- a hidden migration or compatibility boundary appears;
- test results lack a trustworthy oracle;
- the task would delete, rewrite, publish, migrate, or mutate remote state;
- two failed attempts indicate the problem model is wrong;
- a reviewer cannot explain why an invariant remains true.

De-escalate only after an Ultra/High decision has converted the remaining work into deterministic packets with exact acceptance tests. De-escalation changes execution cost, not the importance of the outcome.

## Parallelism Policy

- Ultra lanes own global decisions and integration files.
- High lanes may edit disjoint modules or document families.
- Medium lanes collect evidence, classify, and validate; they do not resolve conflicts.
- A shared file has one writer per batch.
- Review is independent from implementation for High and Ultra changes.
- Increasing thread count without disjoint write sets is treated as a correctness defect.

## Fallback Policy

Before an automated run:

1. Validate project `.codex/config.toml` with the installed Codex strict parser.
2. Verify that the installed Codex model catalog exposes GPT-5.6 Sol and Ultra delegation semantics.
3. Resolve `Ultra` to local `ultra` when the active surface supports it.
4. If `ultra` is unavailable, use `xhigh` for Codex only when explicitly allowed and record `runtime_fallback=true`.
5. For direct GPT-5.6 Sol API calls, map the reasoning component of `Ultra` to `max` and supply proactive delegation through an external manager. API `max` alone is not semantically equivalent to Codex Ultra.
6. If `gpt-5.6-sol` is unavailable to the account or surface, stop or use an explicitly configured fallback; never silently route to an unknown model.
7. Record model, effective effort, delegation mode, Codex version, task role, duration, and validation outcome in the handoff.

## Model Evaluation

Routing is provisional until measured. Maintain a small eval set covering:

- architecture decision quality;
- seeded invariant bugs;
- JSON/CLI contract review;
- branch/open/merge safety;
- document contradiction detection;
- dirty-tree classification;
- focused implementation;
- completion-claim false positives.

Compare tiers on:

- trusted integration success rate;
- escaped defect rate;
- reviewer correction rate;
- elapsed time;
- token/cost estimate when available;
- tool-call count;
- scope violations;
- unsupported completion claims.

Choose the cheapest tier that meets the required reliability for that task class. Do not optimize for first-patch latency alone.

## Repository Configuration

Project defaults and role layers live in:

- `.codex/config.toml`
- `.codex/agents/medium.toml`
- `.codex/agents/high.toml`
- `.codex/agents/ultra.toml`
- `docs/development/ai/role-routing-matrix.json`

The machine-readable matrix is authoritative for automation; this document is authoritative for routing semantics and escalation rules.
