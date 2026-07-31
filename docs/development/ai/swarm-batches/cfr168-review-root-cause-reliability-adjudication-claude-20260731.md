# CFR-168 Review Root-Cause Reliability Disagreement Adjudication

Status: blocked on committed blind-calibration and comparison reports
Owner: portfolio manager  
Provider/model: Claude / `claude-opus-5`  
Role: independent disagreement adjudicator  
Codex fallback: forbidden  
Product/process mutation: forbidden

## Admission Inputs

This packet is deliberately incomplete until the blind Fugu classifier and
fresh Fugu comparator commit:

```text
docs/development/ai/swarm-results/cfr168-review-root-cause-reliability-classification-fugu-20260731/classification.md
docs/development/ai/swarm-results/cfr168-review-root-cause-reliability-comparison-fugu-20260731/comparison.md
```

Before launch, the manager must update this ticket and its typed packet with:

- exact classifier report commit and tree;
- report byte count and SHA-256;
- calibration row-set digest or deterministic row identity list;
- exact comparator report commit, tree, byte count, and SHA-256;
- comparison-freeze byte count, SHA-256, and event locator;
- normalized row-map and adjudication-queue identity digests;
- all decision-relevant row/axis disagreements;
- all rows marked `REPRODUCED_PRODUCTION_EFFECT`;
- every decision-critical `UNAVAILABLE` row routed by the comparator;
- every policy/contradiction row and denominator-changing split/merge; and
- the comparator's temporal-holdout result.

Do not create a `dev-swarm.v1` manifest until those values are committed and
the updated packet passes the complete AI-development gate.

## Objective

Adjudicate only the finite queue produced by the fresh comparator:

1. every decision-relevant row/axis disagreement between the immutable blind
   report and provisional audit;
2. every row classified as a reproduced production effect; and
3. every comparator-routed unavailable, policy/contradiction, or
   denominator-changing split/merge whose resolution could change an
   intervention decision.

Resolve each item from exact Git objects, prior declared scope, repair wording,
candidate implementation, executed safe reproducer, and closure-oracle
falsification where feasible. Preserve all seven axes independently. Do not
choose one narrative primary cause and do not redo the entire corpus merely to
generate more observations.

## Independence Boundary

Use one fresh Claude Opus 5 session. Do not resume the provisional-audit
author, intervention-pilot, product review, or implementation sessions.

Claude produced the provisional audit, so model-family independence is
limited even with a fresh session. Record that threat explicitly. The blind
Fugu rows and comparison report remain immutable inputs: the adjudicator may
select a final value or `UNAVAILABLE`, but may not rewrite either Fugu report
or either freeze.

For each queued row:

1. restate the exact disagreement or reproduced-effect claim;
2. inspect both sides' cited evidence, not their conclusion text;
3. identify the earliest Git object at which the implicated behavior exists;
4. quote or locate the exact earlier scope and repair obligation;
5. reproduce the effect safely against exact candidate identities where
   feasible;
6. deliberately falsify the claimed closure oracle where feasible;
7. record evidence that would overturn the adjudication;
8. decide each disputed axis separately; and
9. state whether the decision changes any aggregate or intervention card.

If object identity, production reachability, scope, obligation wording, or an
oracle cannot be decided, return `UNAVAILABLE`. Project-general reasoning,
severity labels, test names, and report wording are not substitute evidence.

## No Volume Target

The queue size is determined entirely by the committed comparator output.
There is no requested finding count, minimum number of disagreements, or
credit for adding bullets. Do not reopen agreed, non-reproduced,
non-decision-critical rows unless exact evidence proves that the row identity
itself is corrupt; route such corruption separately without expanding the
adjudication opportunistically.

## Required Output

Write exactly:

```text
docs/development/ai/swarm-results/cfr168-review-root-cause-reliability-adjudication-claude-20260731/adjudication.md
```

The report contains:

1. exact task/gate/classifier/comparator/audit identities and SHA-256 values;
2. the immutable input queue and completeness proof;
3. one evidence block per queued row;
4. per-axis Fugu value, provisional-audit value, adjudicated value, and
   evidence;
5. reproducer/oracle command, exact subject, outcome, and counterevidence;
6. unresolved values with specific missing evidence;
7. corrected aggregates with numerator, denominator, and unavailable count;
8. sensitivity against both the Fugu and provisional-audit aggregates;
9. conditional causal statements and supported next experiments;
10. temporal-holdout implications without category retuning;
11. threats to validity, including same-model-family audit/adjudication; and
12. explicit non-decisions for template shape, item count, checklist,
    premortem requirement, severity/routing policy, and process adoption.

The report may conclude that the evidence is insufficient.

## Authorized Operations

Read the exact classifier, comparator, provisional audit, reliability gate,
object chains, packets, result reports, implicated Python source/tests, and
Git objects.
Safe Python reproductions and deliberate oracle falsifications may run in
temporary copies.

Do not run Cargo, Rust builds, Install, Dogfood, Docker, or network calls
other than the Claude provider request. Do not edit product, tests, packets,
existing reports, indexes, ledgers, schemas, tools, issue state, lifecycle
state, templates, or process policy.

## Decision Boundary

This report can:

- replace a disputed axis value for the later causal decision;
- mark a value unresolved;
- identify which intervention experiment is or is not supported; and
- require a sensitivity replay.

It cannot:

- accept or reject any product candidate;
- declare absolute review recall or precision;
- adopt a review/repair template, field, item count, checklist, or premortem
  requirement;
- change the frozen pilot;
- mandate one universal remedy; or
- close the review-reliability question by agent agreement alone.

Any proposed process field must still map one-to-one to an adjudicated failure
mode and pass a bounded live experiment for capture, noise, duplication,
false positives, author burden, reviewer burden, and trusted-integration
effect.

## Validation And Handoff

Run:

```bash
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_contracts.py
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py
```

Commit only the adjudication report and leave the worktree clean. Return:

- report commit/tree/path/byte count/SHA-256;
- exact input queue count and identity digest;
- per-axis resolved/disagreed/unavailable counts;
- reproduced-effect dispositions;
- aggregate sensitivity;
- supported/unsupported experiment cards;
- every threat to validity; and
- every validation command and result.
