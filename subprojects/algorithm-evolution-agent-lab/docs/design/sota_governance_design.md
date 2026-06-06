# SOTA Governance Design

## DES-SOTA-001: Claim readiness state machine

Claim readiness uses explicit states:

```text
idea -> local_improvement -> reproduced_improvement -> benchmark_candidate -> paper_ready -> external_sota_claim
```

Promotion requires evidence checks. Regression or reviewer objection can demote a candidate.

## DES-SOTA-002: Evidence requirements

Evidence requirements increase with claim readiness:

- local improvement: one valid run against local baseline
- reproduced improvement: repeated runs or independent seed set
- benchmark candidate: benchmark protocol satisfied
- paper-ready: ablations and limitations included
- external SOTA claim: external baseline comparison and reproduction bundle

## DES-SOTA-003: Critic checks

The critic runs checks before promotion:

- leakage risk
- metric misuse
- cherry-picking risk
- compute unfairness
- missing ablation
- weak baseline
- non-transferability
- excessive benchmark probing

## DES-SOTA-004: Anti-overclaim interface

The UI and reports shall phrase uncertain progress as hypotheses, candidates, or local improvements until the claim gate is satisfied.

## DES-SOTA-005: Report bundle generation

The report bundle contains:

- candidate summary
- benchmark definition
- baseline table
- run table
- artifact references
- statistical treatment
- ablations
- limitations
- reproduction instructions

## DES-SOTA-006: Review ledger

Reviewer objections are stored as evidence-linked records.

An objection can be resolved by new evidence, scoped limitation, rejected claim, or accepted risk.
