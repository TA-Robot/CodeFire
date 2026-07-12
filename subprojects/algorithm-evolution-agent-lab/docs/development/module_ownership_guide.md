# Module Ownership Guide

This guide expands the source atlas with module-level ownership notes.
It is written for contributors who need to choose the correct file, test family, and documentation surface before editing source code.

## Ownership Rules

- Change the nearest module that already owns the behavior instead of creating a parallel abstraction.
- Add or update tests in the matching test family before broadening shared behavior.
- Update operations documentation when a human workflow changes.
- Update design documentation when a boundary or responsibility changes.

## M001: `loop.py` - Main Orchestration

- Responsibility: this module coordinates planning, scoring, and experiment execution boundaries.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M002: `planner.py` - Candidate Planning

- Responsibility: this module turns objectives into candidate algorithms and hypotheses.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M003: `static_planner.py` - Static Candidate Planning

- Responsibility: this module provides deterministic proposal behavior for tests and baselines.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M004: `codex_planner.py` - Codex-Backed Planning

- Responsibility: this module parses external planner output into structured candidate data.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M005: `evolution.py` - Candidate Evolution

- Responsibility: this module mutates, crosses, and ablates candidate mechanisms.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M006: `scheduler.py` - Portfolio Scheduling

- Responsibility: this module balances exploitation, exploration, and estimated cost.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M007: `queueing.py` - Experiment Queueing

- Responsibility: this module orders ready work by value, constraints, and dependency state.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M008: `batch_planning.py` - Batch Planning

- Responsibility: this module groups ready experiments within cost and diversity limits.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M009: `budgeting.py` - Budget Aware Planning

- Responsibility: this module keeps experiment work inside bounded compute and time budgets.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M010: `frontier.py` - Research Frontier Planning

- Responsibility: this module ranks frontier signals and drafts runnable plans.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M011: `dependencies.py` - Dependency Graph

- Responsibility: this module unlocks prerequisite, replication, ablation, and benchmark work.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M012: `runner.py` - Local Runner

- Responsibility: this module executes experiment commands and captures artifacts.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M013: `fixtures.py` - Benchmark Fixtures

- Responsibility: this module provides local benchmark contracts for repeatable tests.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M014: `ingestion.py` - Result Ingestion

- Responsibility: this module turns metrics artifacts into normalized experiment results.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M015: `metrics.py` - Metric Normalization

- Responsibility: this module aligns metric direction, scale, and tolerance.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M016: `statistics.py` - Statistical Summaries

- Responsibility: this module summarizes repeated runs and confidence intervals.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M017: `analysis.py` - Run Analysis

- Responsibility: this module classifies run outcomes and writes structured notes.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M018: `failures.py` - Failure Classification

- Responsibility: this module distinguishes invalid hypotheses, invalid runs, timeouts, and budget failures.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M019: `promotion.py` - Promotion Policy

- Responsibility: this module gates candidate promotion on evidence and risk.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M020: `triage.py` - Triage Board

- Responsibility: this module ranks active, review, blocked, and deferred work.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M021: `iteration.py` - Iteration Planning

- Responsibility: this module builds bounded next-iteration plans from triage signals.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M022: `risk.py` - Risk Register

- Responsibility: this module prioritizes open risks and mitigation work.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M023: `evidence.py` - Evidence Ledger

- Responsibility: this module links claims, runs, artifacts, and confidence statements.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M024: `evidence_pack.py` - Evidence Packs

- Responsibility: this module bundles promotion and review evidence.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M025: `evidence_freshness.py` - Evidence Freshness

- Responsibility: this module flags outdated benchmark and artifact evidence.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M026: `claims.py` - Claim Discipline

- Responsibility: this module separates hypotheses, internal gains, and external SOTA claims.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M027: `claim_review_queue.py` - Claim Review Queue

- Responsibility: this module orders claims by objections and blocking status.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M028: `claim_audit.py` - Claim Audit Trail

- Responsibility: this module records claim review decisions and follow-ups.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M029: `claim_release.py` - Claim Release Gate

- Responsibility: this module blocks release when artifacts or audit outcomes are incomplete.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M030: `safety.py` - Sota Safety Gate

- Responsibility: this module prevents unsupported claim promotion.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M031: `sota_framing.py` - Sota Framing

- Responsibility: this module keeps weak evidence framed as hypothesis.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M032: `challenge.py` - Challenge Definitions

- Responsibility: this module loads target benchmark and metric definitions.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M033: `baselines.py` - Baseline Registry

- Responsibility: this module selects comparison baselines and rejects duplicates.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M034: `baseline_reproduction.py` - Baseline Reproduction Ledger

- Responsibility: this module records reproduced baseline evidence.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M035: `benchmarks.py` - Benchmark Registry

- Responsibility: this module refreshes versioned benchmark definitions.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M036: `benchmark_domains.py` - Benchmark Domains

- Responsibility: this module groups benchmarks by domain for diversity checks.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M037: `contamination.py` - Contamination Registry

- Responsibility: this module records dataset and artifact overlap risks.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M038: `leakage.py` - Leakage Checks

- Responsibility: this module blocks metric misuse and split overlap.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M039: `overfitting.py` - Overfitting Monitor

- Responsibility: this module limits validation probing and missing controls.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M040: `leaderboard.py` - Leaderboard Ledger

- Responsibility: this module tracks external submission probing risk.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M041: `external_validity.py` - External Validity

- Responsibility: this module requires transfer checks across benchmark domains.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M042: `novelty.py` - Novelty Review

- Responsibility: this module detects overlap with known algorithm families.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M043: `negative_results.py` - Negative Result Archive

- Responsibility: this module retains failed paths to prevent repeated dead ends.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M044: `memory.py` - Research Memory

- Responsibility: this module indexes durable records by tag, kind, and text.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M045: `program.py` - Research Program

- Responsibility: this module records runs, evidence, decisions, and lineage.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M046: `identity.py` - Durable Identity

- Responsibility: this module keeps hypotheses and plans addressable over time.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M047: `run_identity.py` - Run Identity Ledger

- Responsibility: this module links logs, artifacts, evidence, and candidate identity.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M048: `configuration.py` - Configuration Capture

- Responsibility: this module records settings needed for reproducibility.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M049: `compute.py` - Compute Accounting

- Responsibility: this module counts failed attempts and budget overruns.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M050: `retrospective.py` - Cycle Retrospective

- Responsibility: this module builds planning packets, manifests, readiness gates, and archive audits.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M051: `policy_eval.py` - Policy Evaluation

- Responsibility: this module measures selection regret and policy changes.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M052: `progress.py` - Progress Meter

- Responsibility: this module scores sustained research progress.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M053: `reporting.py` - Living Report

- Responsibility: this module summarizes program state in Markdown.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M054: `report_bundle.py` - Report Bundle

- Responsibility: this module writes reproduction-ready report artifacts.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M055: `limitations.py` - Limitation Section

- Responsibility: this module documents failures, gaps, and objections.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M056: `reproduction.py` - Reproduction Checklist

- Responsibility: this module checks repeated artifact evidence.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M057: `regression.py` - Regression Detector

- Responsibility: this module flags benchmark drops introduced by changes.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M058: `review.py` - Review Ledger

- Responsibility: this module tracks objections and review severity.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M059: `overrides.py` - Override Ledger

- Responsibility: this module preserves audit trail for human overrides.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M060: `automation_policy.py` - Automation Policy

- Responsibility: this module blocks unsafe commands and missing artifact contracts.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M061: `automation_surface.py` - Automation Surface Guard

- Responsibility: this module keeps product work focused on experiment automation.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M062: `artifacts.py` - Artifact Contracts

- Responsibility: this module detects missing required experiment artifacts.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M063: `plan_contract.py` - Plan Contract

- Responsibility: this module validates hypothesis, benchmark, metric, command, artifacts, and analysis criteria.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M064: `traceability.py` - Trace Audit

- Responsibility: this module checks requirement, code, and test connectivity.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M065: `roles.py` - Role Registry

- Responsibility: this module registers extensible component roles.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

## M066: `nameaudit.py` - Identifier Naming Audit

- Responsibility: this module keeps implementation names shorter than trace atoms.
- Primary owner surface: keep changes close to the module and its matching tests unless the behavior crosses a lifecycle boundary.
- Inputs to inspect: dataclasses, public methods, nearby tests, CodeFire links, and operation guides named in the source atlas.
- Outputs to protect: stable return types, deterministic ordering, recorded evidence, and reviewable error messages.
- Preferred test shape: one positive case, one rejection or blocking case, and one edge case when the module owns policy logic.
- Documentation trigger: update docs when the module changes an operator decision, artifact contract, claim gate, or review workflow.
- Trace trigger: add or adjust links when a requirement, design atom, code atom, or test atom changes ownership.
- Risk note: avoid hiding research judgment inside helper code that cannot be inspected by reviewers.
- Review question: can a future contributor understand why this module owns the behavior without opening unrelated files?
- Completion check: tests pass, CodeFire reports no open fires, and the source atlas still points to the right guide.
- Handoff note: summarize any new failure mode in history or the relevant operation document.

