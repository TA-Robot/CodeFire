# Experiment Record Patterns

This guide expands the experiment log with reusable record patterns.
Each pattern is a checklist for a real experiment note; examples in this file are templates, not benchmark outcomes.

## Pattern Rules

- Use one experiment note per run, batch, audit, or review packet.
- Keep observed result, interpretation, failure mode, and next action in separate sections.
- Use `archive-negative` for useful failures that should prevent repeated dead-end work.
- Use `block-claim` when evidence is too weak for an external claim.

## P001: Baseline Reproduction Smoke

- Use when: the experiment or audit needs to confirm a baseline can be reproduced before comparing candidates.
- Suggested file name: `YYYY-MM-DD-baseline-reproduction.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P002: Candidate Ablation

- Use when: the experiment or audit needs to isolate which mechanism caused a measured change.
- Suggested file name: `YYYY-MM-DD-candidate-ablation.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P003: Replication Pack

- Use when: the experiment or audit needs to repeat promising results before promotion.
- Suggested file name: `YYYY-MM-DD-replication-pack.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P004: Negative Archive

- Use when: the experiment or audit needs to record a failed hypothesis that should not be retried unchanged.
- Suggested file name: `YYYY-MM-DD-negative-archive.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P005: Timeout Rerun

- Use when: the experiment or audit needs to separate infrastructure timeout from hypothesis failure.
- Suggested file name: `YYYY-MM-DD-timeout-rerun.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P006: Budget Overrun

- Use when: the experiment or audit needs to record compute overrun and decide whether to revise or stop.
- Suggested file name: `YYYY-MM-DD-budget-overrun.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P007: Metric Direction Audit

- Use when: the experiment or audit needs to verify direction and tolerance before interpreting improvement.
- Suggested file name: `YYYY-MM-DD-metric-direction.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P008: Contamination Review

- Use when: the experiment or audit needs to record overlap risks before accepting result evidence.
- Suggested file name: `YYYY-MM-DD-contamination-review.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P009: Leakage Review

- Use when: the experiment or audit needs to capture split or metric leakage checks.
- Suggested file name: `YYYY-MM-DD-leakage-review.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P010: Overfitting Review

- Use when: the experiment or audit needs to track validation probing and missing controls.
- Suggested file name: `YYYY-MM-DD-overfit-review.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P011: Leaderboard Ledger Update

- Use when: the experiment or audit needs to record external submission limits and probing risk.
- Suggested file name: `YYYY-MM-DD-leaderboard-ledger.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P012: External Validity Pass

- Use when: the experiment or audit needs to compare transfer across benchmark domains.
- Suggested file name: `YYYY-MM-DD-external-validity.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P013: Novelty Review Note

- Use when: the experiment or audit needs to compare candidate family overlap before novelty claims.
- Suggested file name: `YYYY-MM-DD-novelty-review.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P014: Learning Curve Review

- Use when: the experiment or audit needs to decide whether additional compute is justified.
- Suggested file name: `YYYY-MM-DD-learning-curve.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P015: Regression Check

- Use when: the experiment or audit needs to detect shared benchmark regressions.
- Suggested file name: `YYYY-MM-DD-regression-check.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P016: Policy Evaluation Run

- Use when: the experiment or audit needs to measure selection regret for planning policy.
- Suggested file name: `YYYY-MM-DD-policy-eval.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P017: Frontier Feedback Run

- Use when: the experiment or audit needs to adjust frontier priorities from observed outcomes.
- Suggested file name: `YYYY-MM-DD-frontier-feedback.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P018: Cycle Retrospective

- Use when: the experiment or audit needs to summarize completed cycle evidence and next lanes.
- Suggested file name: `YYYY-MM-DD-cycle-retrospective.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P019: Handoff Readiness

- Use when: the experiment or audit needs to verify next-cycle packet artifacts.
- Suggested file name: `YYYY-MM-DD-handoff-readiness.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P020: Manifest Drift Audit

- Use when: the experiment or audit needs to compare listed artifacts against current files.
- Suggested file name: `YYYY-MM-DD-manifest-drift.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P021: Claim Review Packet

- Use when: the experiment or audit needs to bundle evidence, objections, and limitations for claim review.
- Suggested file name: `YYYY-MM-DD-claim-review.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P022: Release Gate Review

- Use when: the experiment or audit needs to decide whether a claim can be externally released.
- Suggested file name: `YYYY-MM-DD-release-gate.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P023: Limitation Drafting

- Use when: the experiment or audit needs to state failures and evidence gaps.
- Suggested file name: `YYYY-MM-DD-limitation-section.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P024: Configuration Capture

- Use when: the experiment or audit needs to record reproducibility settings for a run.
- Suggested file name: `YYYY-MM-DD-configuration-capture.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P025: Run Identity Capture

- Use when: the experiment or audit needs to link logs, artifacts, and evidence to candidate identity.
- Suggested file name: `YYYY-MM-DD-run-identity.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P026: Compute Accounting Note

- Use when: the experiment or audit needs to charge failed attempts to budget.
- Suggested file name: `YYYY-MM-DD-compute-accounting.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P027: Review Objection Tracking

- Use when: the experiment or audit needs to record reviewer concerns and next action.
- Suggested file name: `YYYY-MM-DD-review-objection.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P028: Override Ledger Entry

- Use when: the experiment or audit needs to document human override rationale.
- Suggested file name: `YYYY-MM-DD-override-ledger.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P029: Stop Criteria Review

- Use when: the experiment or audit needs to decide whether to stop, defer, revise, or rerun.
- Suggested file name: `YYYY-MM-DD-stop-criteria.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P030: Retry Escalation

- Use when: the experiment or audit needs to route recoverable failures.
- Suggested file name: `YYYY-MM-DD-retry-escalation.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P031: Artifact Contract Check

- Use when: the experiment or audit needs to verify required output files.
- Suggested file name: `YYYY-MM-DD-artifact-contract.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P032: Automation Policy Check

- Use when: the experiment or audit needs to verify command safety and artifact contract.
- Suggested file name: `YYYY-MM-DD-safe-policy.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P033: Queue Readiness Note

- Use when: the experiment or audit needs to show why an experiment was ready or blocked.
- Suggested file name: `YYYY-MM-DD-queue-readiness.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P034: Batch Diversity Note

- Use when: the experiment or audit needs to show budget and benchmark diversity constraints.
- Suggested file name: `YYYY-MM-DD-batch-diversity.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P035: Dependency Unlock Note

- Use when: the experiment or audit needs to record why prerequisites are satisfied.
- Suggested file name: `YYYY-MM-DD-dependency-unlock.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P036: Triage Board Snapshot

- Use when: the experiment or audit needs to capture active, review, and deferred lanes.
- Suggested file name: `YYYY-MM-DD-triage-board.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P037: Risk Register Note

- Use when: the experiment or audit needs to rank open risks and mitigations.
- Suggested file name: `YYYY-MM-DD-risk-register.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P038: Evidence Freshness Note

- Use when: the experiment or audit needs to mark stale benchmark or artifact evidence.
- Suggested file name: `YYYY-MM-DD-evidence-freshness.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P039: Report Bundle Note

- Use when: the experiment or audit needs to record report artifacts for review.
- Suggested file name: `YYYY-MM-DD-report-bundle.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P040: Memory Index Note

- Use when: the experiment or audit needs to add tags and searchable text for future retrieval.
- Suggested file name: `YYYY-MM-DD-memory-index.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P041: Identifier Naming Audit

- Use when: the experiment or audit needs to capture naming findings before review.
- Suggested file name: `YYYY-MM-DD-name-audit.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P042: Documentation Count Audit

- Use when: the experiment or audit needs to record line and word count progress.
- Suggested file name: `YYYY-MM-DD-docs-count.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P043: Documentation Staleness Audit

- Use when: the experiment or audit needs to record guidance that needs updating.
- Suggested file name: `YYYY-MM-DD-docs-staleness.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P044: Documentation Crosslink Audit

- Use when: the experiment or audit needs to confirm every long-lived guide is reachable.
- Suggested file name: `YYYY-MM-DD-docs-crosslink.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P045: Source Atlas Update Record

- Use when: the experiment or audit needs to state module ownership changes.
- Suggested file name: `YYYY-MM-DD-source-atlas.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P046: Roadmap Shift Record

- Use when: the experiment or audit needs to explain changed long-term direction.
- Suggested file name: `YYYY-MM-DD-roadmap-shift.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P047: Todo Closeout Record

- Use when: the experiment or audit needs to show why an item is done.
- Suggested file name: `YYYY-MM-DD-todo-closeout.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P048: History Entry Record

- Use when: the experiment or audit needs to summarize durable project decision.
- Suggested file name: `YYYY-MM-DD-history-entry.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

## P049: Branch Closeout Record

- Use when: the experiment or audit needs to summarize commit, merge, verify, and close state.
- Suggested file name: `YYYY-MM-DD-branch-closeout.md`.
- Required goal section: state the concrete question and why the program needs this evidence now.
- Required hypothesis section: describe the prediction in falsifiable terms or state that the record is an audit rather than a hypothesis test.
- Required benchmark section: name benchmark, dataset, version, metric, direction, and any known contamination risk.
- Required command section: record the command, environment, budget limit, and artifact contract.
- Required artifact section: list logs, metrics, configs, manifests, review packets, and generated reports with stable paths.
- Required result section: report observed numbers or audit findings without upgrading them into claims.
- Required analysis section: separate interpretation from the raw result and call out uncertainty explicitly.
- Required failure section: classify timeout, invalid run, invalid hypothesis, budget overrun, regression, or inconclusive result.
- Required evidence section: link runs, repeated artifacts, baselines, freshness checks, and review objections.
- Required next action section: choose promote, rerun, ablate, revise, defer, archive-negative, or block-claim.
- Completion check: another contributor can decide the next action without rerunning the experiment immediately.

