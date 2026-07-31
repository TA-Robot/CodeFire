# AI Development Acceleration System

This directory defines the external AI-assisted development system for CodeFire.

It is not a CodeFire product feature. CodeFire should not grow agent orchestration, marketplace, prompt management, or autonomous agent runtime logic. The goal is to make external AI agents, human reviewers, CI, and wrappers faster and safer when working on CodeFire.

## Objective

Maximize development speed without losing control of correctness, repository safety, or long-term maintainability.

The acceleration system has nine parts:

1. **Context packs**: minimal, task-specific reading lists so agents do not reread the whole repository.
2. **Task packets**: a standard unit of work with scope, constraints, files, validation, and handoff expectations.
3. **Parallel lanes**: safe ways to split work among manager, implementer, reviewer, docs, and dogfood agents.
4. **Review gates**: fast checks focused on invariants, JSON contracts, module boundaries, and validation evidence.
5. **Learning loop**: every friction point becomes either a CFB/CFR issue, a docs update, or an automation backlog item.
6. **Tooling**: small repository-local checks that make context packs, task packets, and dirty-tree classification executable.
7. **Provider and model routing**: Fugu/Claude/Codex are selected first; Codex `gpt-5.6-sol` Medium/High/Ultra remains the guarded high-assurance adapter.
8. **Exploration and evals**: falsifiable research notes and task corpora that prevent operating policy from becoming unsupported folklore.
9. **Development-flow measurement**: KGI/KPI, immutable event prefixes, resource samples, and
   management triggers that measure cost to trusted integration rather than activity.

## Core Rule

AI acceleration must reduce ambiguity before it increases parallelism.

Do not spawn or delegate implementation work until the task packet has:

- clear objective;
- bounded files or modules;
- explicit non-goals;
- invariant list;
- validation command list;
- expected handoff format.

## Operating Modes

| Mode | Purpose | Primary docs |
|---|---|---|
| Orientation | Get a new agent productive in under 10 minutes. | `context-packs.json`, `../README.md`, `../project-management.md` |
| Strategy | Choose what to build into the acceleration system. | `acceleration-master-plan.md` |
| Single task | Hand one focused issue to one agent. | `operating-protocol.md`, relevant context pack |
| Parallel batch | Split one batch across multiple safe lanes. | `operating-protocol.md` |
| Review | Check an implementation without redoing it. | `operating-protocol.md` |
| Provider routing | Keep bounded work on Fugu, implementation/review on Claude, and guarded final work on Codex. | `provider-routing-policy.md`, `provider-routing-matrix.json` |
| Codex model routing | Select GPT-5.6 Sol Medium/High/Ultra when Codex is explicitly selected. | `model-routing-policy.md`, `role-routing-matrix.json`, `.codex/` |
| Exploration | Keep hypotheses separate from accepted strategy and executable policy. | `../exploration/` |
| Retrospective | Convert friction into better systems. | `acceleration-master-plan.md`, `../bug-backlog.md`, `../history.md` |
| Flow measurement | Record KGI/KPI, lane timing, quality/rework, provider cost, and resource continuity. | `development-flow-kgi-kpi.md`, `flow-metrics/` |

## Fast Path

For a new task:

1. Pick a context pack from the index below.
2. Resolve it to a content-addressed manifest with `../tools/resolve_context_pack.py` when reproducible delegation matters.
3. Pick an execution role, score risk, and select the provider using `provider-routing-policy.md`.
4. Create or reuse a JSON task packet under `task-packets/json/`; validate it with `../tools/check_ai_contracts.py`.
5. Add the task to the active roadmap batch or issue detail.
6. Inspect the dry-run manifest from `../tools/run_ai_role.py` or a `dev-swarm plan`; resolve stale base, unavailable provider/capabilities, and dirty write-set intersections before granting writes.
7. Run an independent reviewer and bind the review to task/patch digests.
8. Run `python3 docs/development/tools/check_ai_development.py` when the AI system or strategy changed.
9. Record the lane/candidate/review/gate events required by `development-flow-kgi-kpi.md`.
10. Update canonical docs and history with the result.

For a batch:

1. Freeze the batch.
2. Group tasks by non-overlapping files or modules.
3. Assign one lane per group.
4. Keep one manager lane responsible for conflict control and final integration.
5. Run reviewers after implementation, not in the same lane.

The committed-state audit at
`swarm-batches/codefire-state-audit-20260729.json` is a concrete five-lane example: four disjoint Fugu
evidence lanes run with four-hour runaway limits, then a fifth Fugu lane merges only their report branches
and prepares a manager handoff. Its automation stops at that integration branch and never updates `main`.

## Context Pack Index

`context-packs.json` is the authoritative manifest for required files, likely files, and pack inheritance.

| Pack | Use when |
|---|---|
| `pack-0-universal-orientation` | Any CodeFire task. |
| `pack-1-rust-cli-command-work` | CLI parsing, command behavior, JSON envelope, dry-run, `next_actions`, branch/open/merge/status workflows. |
| `pack-2-core-semantics` | Atom extraction, Trace Graph, Fire propagation, verification, policy semantics. |
| `pack-3-store-object-commit-integrity` | Object identity, sealed commits, doctor/storage/migration health. |
| `pack-4-remote-http-security` | File remote, HTTP/HTTPS, auth, signatures, nonce replay, branch protection. |
| `pack-5-project-management-docs` | README, current-state docs, issue ledgers, cleanup plan, history. |
| `pack-6-dogfood-algorithm-project` | Work inside `subprojects/algorithm-evolution-agent-lab/`; do not mix with product code. |

## Ready Task Packets

Concrete packets live in `task-packets/`.

| Packet | Purpose |
|---|---|
| `task-packets/branch-list-timeout.md` | Investigate and fix the observed `branch list --path ... --json` timeout. |
| `task-packets/dirty-tree-split.md` | Split the current mixed dirty tree into reviewable packages. |
| `task-packets/json/branch-list-timeout.json` | Typed, risk-routed machine contract for the branch-list investigation. |

Keep packets small enough for one agent, include non-goals and validation, and mark whether each packet is ready, draft, or blocked.

## AI Work Product Requirements

Every AI-produced change must declare:

- what changed;
- why the selected design is correct;
- which invariants are protected;
- which files were intentionally not touched;
- which validation commands passed, failed, or could not run;
- what follow-up remains.

If the agent cannot state these, the work is not ready.

## Safety Constraints

- Never discard user changes.
- Never run destructive git operations.
- Do not combine CodeFire product fixes with dogfood project changes.
- Do not add AI orchestration features to CodeFire product code.
- Do not trust large generated diffs without a reviewer lane.
- Do not accept "tests not run" without an explicit environment or scope reason.

## Integration Points

| Existing doc | Relationship |
|---|---|
| `../agent-code-quality.md` | Code quality bar for all AI-generated implementation. |
| `../module-boundaries.md` | File/module boundary rules for implementation packets. |
| `../agent-workflow.md` | Baseline workflow and validation expectations. |
| `../project-management.md` | Governance, dirty tree, issue, and cycle lifecycle. |
| `../worktree-cleanup-plan.md` | Current mixed tree split plan before more development. |
| `../source-architecture.md`, `../command-source-trace.md`, `../source-data-model-catalog.md`, `../change-playbook.md` | Canonical source ownership, command routing, data contracts, and change recipes. |
| `../command-source-trace.md` | Command-to-source map for CLI work packets. |
| `acceleration-master-plan.md` | Selected implementation path and execution backlog for the acceleration system. |
| `role-routing-matrix.json` | Machine-readable role and reasoning-effort routing. |
| `provider-routing-matrix.json` | Machine-readable Fugu/Claude/Codex selection, concurrency, timeout, fallback, and integration rules. |
| `provider-routing-policy.md` | Low-Codex control-plane decision, escalation, long-running swarm, and recovery semantics. |
| `swarm-batches/codefire-state-audit-20260729.json` | Reproducible four-worker-plus-integration Fugu audit of the current CFR-168 branch family. |
| `model-routing-policy.md` | Risk scoring, Medium/High/Ultra semantics, escalation, fallback, and eval rules. |
| `schemas/` | Versioned task/context/review machine contracts. |
| `evals/` | Deterministic routing cases, redacted historical defect inputs, evaluator oracles, the dated live routing smoke, and future scored live-eval records. |
| `../exploration/` | Phase 1-3 non-authoritative research notes and experiment candidates. |
| `operating-protocol.md` | Task packet shape, review packet shape, parallel-lane rules, and handoff format. |
| `failure-cookbook.md` | Maps recurring environment, provenance, packet, timeout, JSON, and review failures to safe next actions. |
| `validation-report-template.md` | Human handoff shape corresponding to the typed review contract; placeholders are never evidence. |
| `development-flow-kgi-kpi.md`, `flow-metrics/` | Promoted KGI/KPI definitions, management triggers, and schema-checked active/final measurement records. |

## Local Tools and Evidence

| Tool | Purpose |
|---|---|
| `../tools/check_ai_acceleration_docs.py` | Validates AI acceleration JSON manifests, required docs, task packet shape, and stale active-state phrases. |
| `../tools/check_ai_contracts.py` | Validates schemas, typed packet instances, risk decisions, and routing-eval coverage. |
| `../tools/check_project_docs.py` | Checks links, issue ledgers, active references, strategy metadata, and produces a document inventory. |
| `../tools/check_ai_development.py` | Runs the complete local non-mutating AI/docs/runtime/tool gate. |
| `../tools/classify_worktree.py` | Classifies the current dirty tree into reviewable save packages. |
| `../tools/validate_ai_runtime.py` | Strict-parses project routing, checks every role layer and CLI-version adapter, and validates GPT-5.6 Sol Medium/High/Ultra from the bundled model JSON. |
| `../tools/resolve_context_pack.py` | Resolves pack inheritance and hashes every selected context file. |
| `../tools/suggest_validation.py` | Maps changed paths to deduplicated focused validation commands. |
| `../tools/probe_validation_capabilities.py` | Reports available validators and distinguishes binary presence from a revalidated clean-HEAD provenance artifact. |
| `../tools/build_source_bound_rust.py` | Builds the release Rust CLI and writes canonical provenance only when Cargo/crates/`.cargo`/toolchain inputs are clean against HEAD and unchanged across the build. |
| `../tools/run_dogfood_smoke.py` | Runs bounded dogfood status checks read-only by default; `scan` and `verify` require explicit `--allow-mutation`. |
| `../tools/check_swarm_admission.py` | Admits one additional external lane from live memory/swap/disk/PSI/PID pressure, a per-candidate reservation, provider congestion, and conflict ownership without a fixed total-agent cap. |
| `../tools/check_rust_execution_profile.py` | Audits live Rust processes and agent event logs for private targets, alternate toolchains, excessive jobs/threads, or incremental builds; a violation blocks another Rust launch and candidate freeze. |
| `../tools/check_review_terminalization.py` | Distinguishes healthy long-running review progress from repeated terminal-verdict/report-ready signals without an immutable report and emits an evidence-preserving intervention decision. |
| `../tools/run_timed_gate.py` | Runs one argv-only gate with monotonic duration, process-group timeout, complete log digest, and wall-clock-discontinuity evidence. |
| `../tools/run_ai_role.py` | Codex-specific low-level adapter; validates a typed packet and binds context/base/dirty-state. Dry-run is default and real use requires explicit `--allow-codex`. |
| `../tools/prepare_historical_eval.py` | Materializes one deterministic historical CFB/CFR candidate input while withholding the evaluator oracle. |

Dogfood findings start with `task-packets/cfb-intake.md`; only the manager may promote them into the authoritative issue ledgers or amend the frozen batch.
| `evals/live-routing-smoke-2026-07-11.v1.json` | Records successful live GPT-5.6 Sol Medium/High/Ultra minimal invocations and explicitly bounds what the smoke proves. |

The Rust provenance wrapper binds one local clean-HEAD build to its source inputs, toolchain report, build-log digest, and current binary digest. It is not a trusted or reproducible builder and does not provide dependency supply-chain attestation.

## Definition of a Strong AI Task

A strong AI task is small enough to finish independently, but rich enough that the agent does not need to guess.

Good:

- "Fix `branch list --json` timeout by inspecting branch-list traversal and object validation. Do not touch merge/open code. Preserve sealed commit validation. Add focused regression or benchmark-style test."

Bad:

- "Improve branch workflow."
