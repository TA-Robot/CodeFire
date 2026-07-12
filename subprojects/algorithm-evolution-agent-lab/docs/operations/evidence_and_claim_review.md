# Evidence And Claim Review Runbook

This runbook describes how to prepare evidence before promoting a candidate or making an external claim.

## Claim Discipline

Do not call a result "SOTA" until the release gate has enough evidence.

Use weaker labels when evidence is incomplete:

- hypothesis
- internal improvement
- benchmark-local improvement
- candidate for reproduction
- mature external claim

The label should match the evidence, not the desired conclusion.

## Evidence Pack Contents

An evidence pack should include:

- claim id
- candidate or algorithm id
- benchmark id
- baseline id
- run ids
- metric values and direction
- repeated run summary when needed
- configuration capture
- artifact manifest
- analysis note
- known limitations
- unresolved objections

Missing artifacts should block release, not be explained away in prose.

## Baseline Reproduction

Before comparing a candidate, confirm that the baseline is reproduced under compatible settings.

Check:

- baseline identity
- benchmark version
- dataset or fixture version
- metric direction
- repeated run requirements
- configuration parity

If the baseline cannot be reproduced, the candidate may still be useful internally, but it should not become an external claim.

## Objection Handling

Review objections should be explicit and durable.

Common objection types:

- missing evidence
- weak repeated-run support
- leakage risk
- contamination risk
- overfitting risk
- benchmark mismatch
- unsupported generalization
- missing limitation

Each objection should have an owner, severity, status, and follow-up.

## Release Gate

The release gate should pass only when:

- all required artifacts exist
- blocking objections are resolved
- baseline reproduction is present
- metric direction and normalization are clear
- limitation section is present
- external validity is acceptable for the claim type
- evidence freshness is acceptable

If any item fails, downgrade the claim or hold release.

## Negative Results

Negative results are research assets.

Archive:

- failed mechanism
- benchmark context
- failure mode
- avoid condition
- follow-up candidate if any
- budget consumed

The planner and novelty reviewer should use archived negative results before proposing similar work again.

