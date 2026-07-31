# CFR-168 Review Root-Cause Reliability Disagreement Adjudication

Status: blocked on committed Fugu blind-calibration and comparison evidence

Typed packet:
[`json/cfr168-review-root-cause-reliability-adjudication-claude-20260731.json`](json/cfr168-review-root-cause-reliability-adjudication-claude-20260731.json)

Execution ticket:
[`../swarm-batches/cfr168-review-root-cause-reliability-adjudication-claude-20260731.md`](../swarm-batches/cfr168-review-root-cause-reliability-adjudication-claude-20260731.md)

Reliability gate:
[`../swarm-batches/cfr168-review-root-cause-reliability-gate-20260731.md`](../swarm-batches/cfr168-review-root-cause-reliability-gate-20260731.md)

Provider/model: Claude / `claude-opus-5`  
Role: independent reviewer/adjudicator  
Preparation base: `107293420d35daf861f8d7b751472f86e794c03a`

## Objective

After the blind classifier and fresh comparator commit, adjudicate the
comparator's exact decision-relevant episode identity/closure and row/axis
disagreement queue, all reproduced-production-effect rows, routed
decision-critical unavailable values, policy/contradiction rows, and
denominator-changing split/merge decisions. This includes every
multi-observation episode link and proven-closure/new-episode relation that
changes the episode denominator. Resolve each disputed episode link and axis
independently from exact objects and repeatable evidence without rerunning the
whole corpus for finding volume.

## Scope

The exact queue, classifier and comparator identities/digests, comparison
freeze, normalized episode/row map, and episode/row identities are currently
unavailable. The manager must bind them and every decision-relevant
identity/closure dispute into the execution ticket and typed packet before
changing status to `ready`.

The future only authorized write is:

```text
docs/development/ai/swarm-results/cfr168-review-root-cause-reliability-adjudication-claude-20260731/adjudication.md
```

No `dev-swarm.v1` manifest is authorized while this packet is blocked.

## Non-Goals

- Re-review every agreed row or generate a target number of observations.
- Modify the blind classification, provisional audit, or frozen pilot.
- Accept, reject, or repair a product candidate.
- Estimate absolute review recall or precision.
- Adopt a template, checklist, field count, severity policy, routing policy,
  or process change.
- Review development-environment infrastructure.

## Validation

Before admission, bind and recompute the exact classifier and comparator
commits/trees/report SHA-256 values, comparison-freeze SHA-256, normalized
episode/row-map digest, queue identity digest, and all required rows or
episode disputes. Run the complete AI-development gate after updating the
packet. The eventual
report-only worker must run `git diff --check`, `check_ai_contracts.py`, and
`check_project_docs.py`, commit only the authorized report, and leave a clean
worktree.

## Handoff

Return the report identity and digest, exact queue completeness proof,
per-axis adjudications and unavailable values, reproduced-effect dispositions,
episode identity/closure dispositions, observation-/episode-/generation-level
aggregate sensitivity, supported or unsupported experiment cards, temporal
implications, threats to validity, validation results, and explicit
non-decisions for every process/template property.
