# CFR-168 Review Root-Cause Reliability Blind Classification

Status: ready as a report-only task; provider-held  

Typed packet:
[`json/cfr168-review-root-cause-reliability-classification-fugu-20260731.json`](json/cfr168-review-root-cause-reliability-classification-fugu-20260731.json)

Execution ticket:
[`../swarm-batches/cfr168-review-root-cause-reliability-classification-fugu-20260731.md`](../swarm-batches/cfr168-review-root-cause-reliability-classification-fugu-20260731.md)

Reliability gate:
[`../swarm-batches/cfr168-review-root-cause-reliability-gate-20260731.md`](../swarm-batches/cfr168-review-root-cause-reliability-gate-20260731.md)

Provider/model: Fugu / `fugu-ultra-v1.1`  
Role: independent reviewer/classifier  
Base: `f259e9488e9676af331b2fed45d15209feec0e7b`

## Objective

Classify the finite observed successor-finding corpus on six independent
evidence axes before opening the provisional causal labels, freeze that blind
classification by byte count and SHA-256, compare all disagreements, and then
apply unchanged rules to the temporal holdout.

This task tests whether the evidence supports review-scope omission,
repair-introduced defects, unmet or weakened repair obligations, weak closure
oracles, evidence-only blockers, routing problems, some combination of these,
or no decision. It must not presume that review omission is the cause.

## Scope

Read the exact predecessor/candidate/report/repair object chains bound by the
typed packet and execution ticket. Reconstruct the calibration corpus without
opening the provisional causal labels, preserve a digest-bound blind freeze,
compare only after that freeze, and keep the temporal reports hidden until the
calibration comparison is complete.

The only authorized write is:

```text
docs/development/ai/swarm-results/cfr168-review-root-cause-reliability-classification-fugu-20260731/classification.md
```

No product, test, issue, lifecycle, packet, policy, existing report, or
development-environment file may change.

## Non-Goals

- Accept, reject, or repair a product candidate.
- Estimate absolute review recall or precision without an independent defect
  denominator.
- Select one narrative primary cause when several axes can hold.
- Complete or reinterpret the frozen intervention pilot.
- Review the development environment.
- Adopt a review or repair process.

## Non-Adoption

The 51 observed successor findings describe the frozen census; they are not a
requested output count. The classifier receives no credit for extra bullets
and may not split one effect for volume.

The analysis row form in the reliability gate is evidence capture for this
causal study only. This task cannot adopt a review template, checklist,
finding quota, minimum detail level, severity policy, routing policy, or
candidate disposition. Any later process proposal must map to an adjudicated
failure mode and survive a bounded live experiment measuring capture, noise,
duplication, false positives, and author/reviewer burden.

## Validation

Recompute every bound Git identity and evidence digest before classification.
Before reading the provisional labels, emit the blind-freeze byte count and
SHA-256 in one event-log result. Before handoff run:

```bash
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_contracts.py
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py
```

The committed diff and final worktree must contain only the authorized
classification report.

## Handoff

Return the exact report commit, tree, path, and SHA-256; blind-freeze byte
count, SHA-256, and event locator; calibration row and unavailable counts;
per-axis aggregates and disagreements; the exact adjudication queue; temporal
stability; all threats to validity and censoring; every validation result; and
explicit non-decisions for template length, finding count, checklist, and
process adoption.

## Launch Hold

Do not launch until the recorded Fugu usage-reset window has passed and one
bounded health probe succeeds. Use a fresh Fugu session. Codex worker fallback,
Claude fallback within this prepared lane, and a competing classifier are
forbidden.
