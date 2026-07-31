# CFR-168 Review Root-Cause Reliability Blind Calibration

Status: ready as a report-only task; provider-held  

Typed packet:
[`json/cfr168-review-root-cause-reliability-classification-fugu-20260731.json`](json/cfr168-review-root-cause-reliability-classification-fugu-20260731.json)

Execution ticket:
[`../swarm-batches/cfr168-review-root-cause-reliability-classification-fugu-20260731.md`](../swarm-batches/cfr168-review-root-cause-reliability-classification-fugu-20260731.md)

Reliability gate:
[`../swarm-batches/cfr168-review-root-cause-reliability-gate-20260731.md`](../swarm-batches/cfr168-review-root-cause-reliability-gate-20260731.md)

Provider/model: Fugu / `fugu-ultra-v1.1`  
Role: independent reviewer/classifier  
Blind base: `c258cbfc061b14905554c9d319b350d2da7f771b`

## Objective

At the exact pre-audit snapshot, reconstruct the finite observed
successor-finding corpus and classify it on seven independent evidence axes.
The number of effect rows is an outcome, not an input. Publish one immutable
blind report before a different session is allowed to see the provisional
audit or temporal reports.

This task tests whether snapshot-visible evidence supports review-scope
omission, repair-introduced defects, unmet or weakened repair obligations,
weak closure oracles, evidence-only blockers, routing problems, some
combination of these, or no decision. It must not presume that review omission
is the cause.

## Scope

Run from snapshot `c258cbf…`, where exactly 21 review reports exist and the
provisional audit plus three temporal reports are absent. Read the exact
predecessor/candidate/report/repair object chains named by snapshot-visible
sources. Do not inspect newer refs, branches, worktrees, or commits.

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

No expected successor-effect count is supplied to the classifier. It receives
no credit for extra bullets and may not split one effect for volume. A later
fresh session will reconcile stable row identities without editing this
report.

The analysis row form in the reliability gate is evidence capture for this
causal study only. This task cannot adopt a review template, checklist,
finding quota, minimum detail level, premortem requirement, severity policy,
routing policy, or candidate disposition. Any later process proposal must map
to an adjudicated failure mode and survive a bounded live experiment measuring
capture, noise, duplication, false positives, and author/reviewer burden.

## Validation

Recompute every bound Git identity and evidence digest before classification.
Before handoff run:

```bash
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_contracts.py
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py
```

The committed diff and final worktree must contain only the authorized
classification report.

## Handoff

Return the exact report commit, tree, path, byte count, and SHA-256;
generation-chain count/digest and method flags; raw effect-row count and
row-identity digest; per-axis aggregates and unavailable counts; exact
invariant recurrence; reproduced-effect and decision-critical-unavailable row
sets; all inclusion/exclusion disputes; all threats to validity and censoring;
every validation result; and explicit non-decisions for template length,
finding count, checklist, premortem, and process adoption.

## Launch Hold

Do not launch until the recorded Fugu usage-reset window has passed and one
bounded health probe succeeds. Use a fresh Fugu session. Codex worker fallback,
Claude fallback within this prepared lane, and a competing classifier are
forbidden.
