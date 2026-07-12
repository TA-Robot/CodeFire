# Experiment Automation Design Boundaries

This document expands the architecture narrative with explicit boundaries that keep the research automation system reviewable.
Use it when a proposed change could collapse two responsibilities into one module or when a review needs a crisp ownership rule.

## Boundary Review Questions

- Does the change move research judgment into code that lacks evidence or review records?
- Does the change make a claim stronger than the artifacts can support?
- Does the change blur planning, execution, ingestion, analysis, review, and release responsibilities?
- Does the change leave a future operator with a clear runbook and verification path?

## B001: Planning Versus Execution

- Boundary rule: candidate selection must not perform experiment execution.
- Applies when: a contributor edits modules, tests, or docs that touch `planning-versus-execution` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B002: Execution Versus Ingestion

- Boundary rule: runner capture must remain separate from result interpretation.
- Applies when: a contributor edits modules, tests, or docs that touch `execution-versus-ingestion` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B003: Ingestion Versus Analysis

- Boundary rule: artifact parsing must not decide promotion.
- Applies when: a contributor edits modules, tests, or docs that touch `ingestion-versus-analysis` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B004: Analysis Versus Claim

- Boundary rule: result summaries must not become external claims automatically.
- Applies when: a contributor edits modules, tests, or docs that touch `analysis-versus-claim` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B005: Claim Versus Release

- Boundary rule: a claim candidate still needs release gate evidence.
- Applies when: a contributor edits modules, tests, or docs that touch `claim-versus-release` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B006: Frontier Versus Queue

- Boundary rule: research opportunity ranking must remain separate from ready work ordering.
- Applies when: a contributor edits modules, tests, or docs that touch `frontier-versus-queue` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B007: Queue Versus Batch

- Boundary rule: priority ordering must not silently ignore cost and diversity constraints.
- Applies when: a contributor edits modules, tests, or docs that touch `queue-versus-batch` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B008: Budget Versus Policy

- Boundary rule: budget fit does not imply safety or evidence readiness.
- Applies when: a contributor edits modules, tests, or docs that touch `budget-versus-policy` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B009: Baseline Versus Benchmark

- Boundary rule: baseline choice must track benchmark identity and metric direction.
- Applies when: a contributor edits modules, tests, or docs that touch `baseline-versus-benchmark` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B010: Replication Versus Ablation

- Boundary rule: repeatability and mechanism tests answer different questions.
- Applies when: a contributor edits modules, tests, or docs that touch `replication-versus-ablation` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B011: Negative Result Versus Failed Run

- Boundary rule: invalid execution must not be archived as hypothesis evidence.
- Applies when: a contributor edits modules, tests, or docs that touch `negative-versus-failed-run` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B012: Freshness Versus Confidence

- Boundary rule: old evidence can be statistically strong and still unusable.
- Applies when: a contributor edits modules, tests, or docs that touch `freshness-versus-confidence` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B013: Review Versus Override

- Boundary rule: human override must preserve audit trail rather than erase objections.
- Applies when: a contributor edits modules, tests, or docs that touch `review-versus-override` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B014: Memory Versus History

- Boundary rule: searchable records and chronological project history serve different readers.
- Applies when: a contributor edits modules, tests, or docs that touch `memory-versus-history` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B015: Documentation Versus Trace Links

- Boundary rule: prose explains context while CodeFire links prove relationships.
- Applies when: a contributor edits modules, tests, or docs that touch `documentation-versus-trace` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B016: Implementation Names Versus Atom Identifiers

- Boundary rule: source names should stay readable while atoms stay traceable.
- Applies when: a contributor edits modules, tests, or docs that touch `naming-versus-atoms` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B017: Product Surface Versus Research Automation

- Boundary rule: features must support experiment automation before peripheral polish.
- Applies when: a contributor edits modules, tests, or docs that touch `product-versus-research` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B018: Sota Claim Versus Internal Gain

- Boundary rule: internal score improvements are not external benchmark claims.
- Applies when: a contributor edits modules, tests, or docs that touch `sota-versus-internal-gain` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B019: Benchmark Domain Versus Metric

- Boundary rule: domain diversity and metric validity are different constraints.
- Applies when: a contributor edits modules, tests, or docs that touch `domain-versus-metric` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B020: Configuration Capture Versus Result

- Boundary rule: reproducibility settings must not be inferred from metrics.
- Applies when: a contributor edits modules, tests, or docs that touch `configuration-versus-result` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B021: Compute Accounting Versus Timeout Handling

- Boundary rule: failed attempts still consume budget.
- Applies when: a contributor edits modules, tests, or docs that touch `compute-versus-timeout` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B022: Manifest Versus Artifact

- Boundary rule: a listed artifact must still be checked for drift.
- Applies when: a contributor edits modules, tests, or docs that touch `manifest-versus-artifact` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B023: Handoff Bundle Versus Summary

- Boundary rule: handoff material must include audit state, not only prose summary.
- Applies when: a contributor edits modules, tests, or docs that touch `handoff-versus-summary` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B024: Plan Lint Versus Plan Quality

- Boundary rule: a syntactically valid plan can still be a poor research plan.
- Applies when: a contributor edits modules, tests, or docs that touch `plan-lint-versus-plan-quality` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B025: Policy Evaluation Versus Progress Meter

- Boundary rule: selection regret and overall program progress answer separate questions.
- Applies when: a contributor edits modules, tests, or docs that touch `policy-eval-versus-progress` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B026: Contamination Versus Overfitting

- Boundary rule: data overlap and validation probing need separate controls.
- Applies when: a contributor edits modules, tests, or docs that touch `contamination-versus-overfit` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B027: Leakage Versus Leaderboard Probing

- Boundary rule: local split issues and external probing are distinct risks.
- Applies when: a contributor edits modules, tests, or docs that touch `leakage-versus-leaderboard` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B028: Novelty Versus External Validity

- Boundary rule: new mechanisms still need transfer evidence.
- Applies when: a contributor edits modules, tests, or docs that touch `novelty-versus-external-validity` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B029: Stop Criteria Versus Deferral

- Boundary rule: stopping a path is stronger than parking it for capacity.
- Applies when: a contributor edits modules, tests, or docs that touch `stop-versus-defer` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B030: Retry Versus Revise

- Boundary rule: rerun only when the hypothesis remains valid and execution failed.
- Applies when: a contributor edits modules, tests, or docs that touch `retry-versus-revise` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B031: Component Role Versus Module Owner

- Boundary rule: extension roles do not replace file ownership.
- Applies when: a contributor edits modules, tests, or docs that touch `role-versus-owner` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B032: Requirements Versus Design

- Boundary rule: requirements state acceptance while design states responsibility.
- Applies when: a contributor edits modules, tests, or docs that touch `spec-versus-design` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B033: Design Versus Implementation

- Boundary rule: design boundaries should not depend on incidental class names.
- Applies when: a contributor edits modules, tests, or docs that touch `design-versus-implementation` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B034: Unit Tests Versus Research Evidence

- Boundary rule: passing tests do not prove benchmark improvement.
- Applies when: a contributor edits modules, tests, or docs that touch `test-versus-evidence` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B035: Documentation Example Versus Experiment Result

- Boundary rule: examples must be clearly marked when synthetic.
- Applies when: a contributor edits modules, tests, or docs that touch `example-versus-result` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B036: Codefire Branch Versus Filesystem Path

- Boundary rule: the open directory must match the intended branch.
- Applies when: a contributor edits modules, tests, or docs that touch `branch-versus-worktree` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B037: Codefire Verify Versus Unittest

- Boundary rule: trace checks and behavior tests cover different risks.
- Applies when: a contributor edits modules, tests, or docs that touch `verify-versus-test` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B038: Verification Warning Versus Blocker

- Boundary rule: nonblocking warnings still need visible review.
- Applies when: a contributor edits modules, tests, or docs that touch `warning-versus-blocker` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B039: Merge Versus Commit

- Boundary rule: merged files still need the target branch sealed.
- Applies when: a contributor edits modules, tests, or docs that touch `merge-versus-commit` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B040: Branch Close Versus Discard

- Boundary rule: closing clean work must not erase main changes.
- Applies when: a contributor edits modules, tests, or docs that touch `close-versus-discard` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B041: Runbook Versus Tutorial

- Boundary rule: operators need executable steps more than background narrative.
- Applies when: a contributor edits modules, tests, or docs that touch `runbook-versus-tutorial` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B042: Source Atlas Versus Api Documentation

- Boundary rule: atlas maps ownership rather than restating every method.
- Applies when: a contributor edits modules, tests, or docs that touch `atlas-versus-api-docs` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B043: Roadmap Versus Todo

- Boundary rule: roadmap carries direction while todo carries checkable tasks.
- Applies when: a contributor edits modules, tests, or docs that touch `roadmap-versus-todo` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B044: History Versus Changelog

- Boundary rule: history preserves project decisions, not only file changes.
- Applies when: a contributor edits modules, tests, or docs that touch `history-versus-changelog` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B045: Readme Versus Deep Guide

- Boundary rule: README points readers while deep guides carry detail.
- Applies when: a contributor edits modules, tests, or docs that touch `readme-versus-deep-guide` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B046: Safety Gate Versus Iteration Speed

- Boundary rule: fast iteration cannot bypass evidence requirements.
- Applies when: a contributor edits modules, tests, or docs that touch `safety-versus-speed` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B047: Automation Versus Autonomy

- Boundary rule: automated agents still need explicit review boundaries.
- Applies when: a contributor edits modules, tests, or docs that touch `automation-versus-autonomy` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B048: Determinism Versus Adaptivity

- Boundary rule: adaptive planning must produce deterministic records.
- Applies when: a contributor edits modules, tests, or docs that touch `determinism-versus-adaptivity` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B049: Local Benchmark Versus External Benchmark

- Boundary rule: local evidence must not be overclaimed externally.
- Applies when: a contributor edits modules, tests, or docs that touch `local-versus-external` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B050: Candidate Record Versus Research Program

- Boundary rule: single candidate state must not overwrite program memory.
- Applies when: a contributor edits modules, tests, or docs that touch `candidate-versus-program` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B051: Objective Versus Hypothesis

- Boundary rule: objectives describe goals while hypotheses make testable predictions.
- Applies when: a contributor edits modules, tests, or docs that touch `objective-versus-hypothesis` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B052: Metric Versus Decision

- Boundary rule: metrics inform but do not replace governance decisions.
- Applies when: a contributor edits modules, tests, or docs that touch `metric-versus-decision` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

## B053: Artifact Versus Interpretation

- Boundary rule: raw files and analyst conclusions must remain separable.
- Applies when: a contributor edits modules, tests, or docs that touch `artifact-versus-interpretation` behavior.
- Keep on left side: input validation, local responsibility, and data that belongs to the earlier lifecycle stage.
- Keep on right side: downstream decision making, review interpretation, and artifacts owned by the later lifecycle stage.
- Allowed coupling: explicit dataclasses, stable identifiers, documented artifacts, and CodeFire trace links.
- Disallowed coupling: hidden global state, implicit benchmark assumptions, and prose claims without evidence refs.
- Design check: the boundary should be explainable from this document, the source atlas, and one focused test family.
- Test check: add a rejection case when the change could accidentally cross this boundary.
- Documentation check: update the relevant operation guide if an operator must make a new decision.
- Review question: what would go wrong if this responsibility moved to the other side of the boundary?
- Completion evidence: changed source or docs preserve the boundary and verification reports no blocking trace problem.

