# SOTA Challenge Protocol

## REQ-SOTA-001: Challenge definition

A SOTA challenge shall define task, dataset, metric, split, baseline set, target improvement, allowed compute, and disallowed shortcuts.

## REQ-SOTA-002: Baseline reproduction

The system shall reproduce or import trusted baseline numbers before claiming improvement.

## REQ-SOTA-003: Leakage check

The system shall check for dataset leakage, benchmark contamination, train/test overlap, and accidental metric misuse.

## REQ-SOTA-004: Statistical confidence

The system shall estimate uncertainty using repeated runs, confidence intervals, bootstrap analysis, or another benchmark-appropriate method.

## REQ-SOTA-005: Compute accounting

The system shall report compute used to achieve the result, including failed attempts when relevant to the claim.

## REQ-SOTA-006: Novelty review

The system shall require a novelty review that compares the candidate against known algorithm families and published techniques.

## REQ-SOTA-007: External validity

The system shall evaluate whether gains transfer beyond one narrow benchmark before elevating a candidate as broadly important.

## REQ-SOTA-008: Claim readiness levels

The system shall represent claim readiness as levels: idea, local improvement, reproduced improvement, benchmark candidate, paper-ready claim, and external SOTA claim.

## REQ-SOTA-009: Reviewer agent

A critic or reviewer role shall attempt to invalidate strong claims before they are promoted.

## REQ-SOTA-010: Report bundle

A SOTA-ready candidate shall produce a report bundle containing experiment plan, code reference, run artifacts, analysis, limitations, and reproduction instructions.

## REQ-SOTA-011: Anti-overfitting protocol

The agent shall avoid repeatedly optimizing against a hidden test set or leaderboard without controls.

## REQ-SOTA-012: Honest limitation section

Every promoted claim shall include a limitation section generated from failed experiments, benchmark gaps, and reviewer objections.

## REQ-SOTA-013: Leaderboard submission control

The system shall track hidden-test or leaderboard submissions and flag excessive probing, cooldown violations, and submissions without sufficient local evidence.

## REQ-SOTA-014: Contamination source registry

The system shall maintain known contamination sources and flag candidate datasets or benchmark artifacts that overlap with prohibited sources.
