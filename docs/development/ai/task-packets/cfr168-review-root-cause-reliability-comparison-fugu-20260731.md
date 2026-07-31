# CFR-168 Review Root-Cause Reliability Comparison

Status: blocked on committed blind-calibration evidence

Typed packet:
[`json/cfr168-review-root-cause-reliability-comparison-fugu-20260731.json`](json/cfr168-review-root-cause-reliability-comparison-fugu-20260731.json)

Execution ticket:
[`../swarm-batches/cfr168-review-root-cause-reliability-comparison-fugu-20260731.md`](../swarm-batches/cfr168-review-root-cause-reliability-comparison-fugu-20260731.md)

Reliability gate:
[`../swarm-batches/cfr168-review-root-cause-reliability-gate-20260731.md`](../swarm-batches/cfr168-review-root-cause-reliability-gate-20260731.md)

Provider/model: Fugu / `fugu-ultra-v1.1`  
Role: independent corpus comparator

## Objective

After the blind calibration is committed and digest-bound, use a fresh session
to reconcile its immutable effect rows with the provisional audit, compare all
six axes, content-freeze that comparison, and only then apply the unchanged
rubric to three later reports.

The provisional audit's 51-row census is comparison input, not a target. A
different blind count must be explained by exact effect identity rather than
graded by numerical equality.

## Scope

The blind report identity, row count, row-identity digest, reproduced-effect
set, decision-critical-unavailable set, and comparison base are currently
unavailable. The manager must bind them into the execution ticket and typed
packet before changing status to `ready`.

The future only authorized write is:

```text
docs/development/ai/swarm-results/cfr168-review-root-cause-reliability-comparison-fugu-20260731/comparison.md
```

No `dev-swarm.v1` manifest is authorized while this packet is blocked.

## Non-Goals

- Rewrite the blind calibration or provisional audit.
- Treat row-count equality as coverage proof.
- Choose a narrative primary cause.
- Adjudicate conflicting evidence.
- Accept, reject, or repair a product candidate.
- Estimate absolute review recall or precision.
- Adopt a review template, checklist, field count, routing rule, or process
  change.

## Validation

Before admission, bind and recompute the blind report commit/tree/path,
byte count, SHA-256, row count, row-identity digest, reproduced-effect set,
decision-critical-unavailable set, and exact comparison base. The eventual
report-only worker must run `git diff --check`, `check_ai_contracts.py`, and
`check_project_docs.py`, commit only the authorized report, and leave a clean
worktree.

## Handoff

Return exact report identity and digest, full row reconciliation, per-axis
comparison, comparison-freeze identity, unchanged temporal results, exact
adjudication queue and digest, unavailable values, threats to validity,
validation results, and explicit non-decisions for all process/template
properties.
