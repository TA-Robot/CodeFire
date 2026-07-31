# CFR-168 Review Root-Cause Reliability Blind Calibration

Status: logically ready; launch held until Fugu usage reset and bounded health probe  
Owner: portfolio manager  
Provider/model: Fugu / `fugu-ultra-v1.1`  
Role: independent method classifier  
Codex fallback: forbidden  
Product/process mutation: forbidden

## Exact Blind Base

| Item | Exact value |
|---|---|
| Calibration snapshot | `c258cbfc061b14905554c9d319b350d2da7f771b` |
| Snapshot tree | `75135efad20c5908884231b152cde72d0c9be965` |
| Snapshot committed at | `2026-07-31T08:37:14+00:00` |
| Review discovery | `git ls-tree -r --name-only HEAD docs/development/ai/swarm-results \| grep '/review\\.md$'` |
| Review files at snapshot | 21 |
| Expected successor-effect rows | deliberately unbound |

Run in the worktree created from the exact calibration snapshot. At that
snapshot:

- the provisional root-cause audit does not exist;
- the later Upload, KGI/KPI, and Cargo reports do not exist; and
- exactly 21 `review.md` files exist below the result tree.

Do not inspect another branch, tag, reflog entry, worktree, or commit newer
than the exact snapshot. Candidate and predecessor objects named by the
snapshot reports and their snapshot-visible packets may be inspected even
when they are not ancestors of the snapshot. No commit, path, or conclusion
learned from a newer management artifact may enter calibration.

Use a fresh session. Do not resume any earlier Fugu review or audit session.

## Objective

Reconstruct every independently testable effect first reported in a successor
review generation, then classify it on seven independent evidence axes:

1. code origin;
2. detectability in the immediately prior declared review scope;
3. handling of an exact repair obligation;
4. strength of effect evidence;
5. closure-oracle quality; and
6. review-routing state; and
7. pre-repair hazard coverage.

Do not assume that review omission is the cause. Do not choose one narrative
primary cause. Do not propose a finding count, checklist, review-template
depth, or process change.

## Census Rule — Count Is An Outcome

The report universe is finite, but the number of independently testable
successor effects is not supplied to this lane.

- Start from the 21 exact report paths found at `HEAD`.
- Reconstruct same-surface predecessor and successor generations from exact
  report, candidate, repair, and result identities.
- Include an effect only when it first appears in a successor report and has a
  stable report locator.
- One row represents one independently falsifiable product effect or one exact
  evidence/routing defect.
- Split a combined report item only when its effects have different subjects,
  evidence, or axis values.
- Do not split wording variants, severity restatements, or repeated evidence.
- Do not merge independently testable effects.
- Publish the observed row count and every inclusion/exclusion dispute.

There is no expected row count to hit. A later, different session will
reconcile row identities against another analyst's census. It will not edit
this report.

## Six-Axis Rubric

Assign every axis independently. An axis value and its evidence are separate
fields. Use `UNAVAILABLE` when exact evidence does not decide a value.

### A. Code origin

| Value | Meaning |
|---|---|
| `PREEXISTING_IN_PREDECESSOR` | The implicated behavior or code shape exists in the immediately preceding reviewed candidate. |
| `INTRODUCED_BY_REPAIR` | It is absent from that predecessor and introduced by the repair. |
| `NO_PRODUCT_DEFECT_ASSERTED` | The finding concerns evidence, documentation, policy, or a non-production-reachable claim rather than an asserted product defect. |
| `UNAVAILABLE` | Exact objects or evidence do not decide origin. |

### B. Prior-review detectability

| Value | Meaning |
|---|---|
| `EXPLICITLY_BOUND_BEFORE_REPAIR` | The predecessor report or repair packet already states the exact obligation. |
| `WITHIN_DECLARED_PRIOR_SCOPE` | The defect existed then and the prior review declares the implicated path or invariant in scope, but did not bind the exact obligation. |
| `OUTSIDE_DECLARED_PRIOR_SCOPE` | The prior review's declared scope excludes the implicated path or invariant. |
| `NOT_PRESENT_AT_PRIOR_REVIEW` | The repair introduced the implicated surface after the prior review. |
| `NOT_APPLICABLE` | No product-defect detectability claim is made. |
| `UNAVAILABLE` | Scope or temporal identity cannot be decided. |

Only `PREEXISTING_IN_PREDECESSOR` together with
`WITHIN_DECLARED_PRIOR_SCOPE` supports the bounded statement "observed prior
review scope miss". It does not establish absolute recall.

### C. Repair-contract handling

| Value | Meaning |
|---|---|
| `CLOSED_AS_WRITTEN` | The exact bound obligation is satisfied. |
| `NOT_CLOSED_AS_WRITTEN` | The exact bound obligation remains false. |
| `WEAKENED_OR_RESTATED` | The implementation or evidence silently substitutes a weaker property. |
| `CRITERION_CHANGED_AFTER_REPAIR` | A later decision changes the obligation instead of proving the original wording. |
| `NOT_BOUND` | No earlier exact obligation exists. |
| `UNAVAILABLE` | Closure cannot be decided. |

### D. Effect evidence

| Value | Meaning |
|---|---|
| `REPRODUCED_PRODUCTION_EFFECT` | A minimal reproducer reaches production code and demonstrates an externally or durably observable wrong effect. |
| `STATIC_PRODUCTION_REACHABLE_PROOF` | Exact source/object evidence proves production reachability and the defective state without an executed reproducer. |
| `INJECTION_ONLY_EFFECT` | The effect requires a state or fault for which no production route is demonstrated. |
| `CLAIM_OR_EVIDENCE_ONLY` | Behavior may be correct; a test, document, or proof overclaims it. |
| `UNAVAILABLE` | No decision-grade effect evidence exists. |

### E. Closure-oracle quality

| Value | Meaning |
|---|---|
| `DETECTS_THE_BOUND_PROPERTY` | The oracle fails when the bound property is deliberately falsified. |
| `PASSES_DESPITE_FALSIFICATION` | A temporary reversal or counterexample survives the oracle. |
| `LATE_OBSERVATION` | The assertion occurs after the effect it claims to prevent. |
| `NO_RELEVANT_ORACLE` | No oracle was offered for the property. |
| `NOT_APPLICABLE` | The finding has no closure-oracle claim. |
| `UNAVAILABLE` | The oracle cannot be executed or inspected sufficiently. |

### F. Review-routing state

| Value | Meaning |
|---|---|
| `PRODUCT_DEFECT_BLOCKER` | A demonstrated or statically proven production defect blocks acceptance. |
| `CLOSURE_EVIDENCE_BLOCKER` | Product closure may be true, but required evidence is absent or invalid. |
| `POLICY_PROMOTED_BLOCKER` | A non-blocking observation became mandatory through policy or manager choice. |
| `NON_BLOCKING_FOLLOW_UP` | The report explicitly routes the item forward without changing the candidate verdict. |
| `UNSATISFIABLE_OR_CONTRADICTORY` | The criterion cannot be satisfied as written or conflicts with another bound requirement. |
| `UNAVAILABLE` | The report does not make the routing basis decidable. |

### G. Pre-repair hazard coverage

Apply this axis to an effect classified `INTRODUCED_BY_REPAIR`. It records
what the immutable pre-repair report, packet, plan, or required oracle actually
contained; it does not claim that an unnamed hazard was foreseeable.

| Value | Meaning |
|---|---|
| `EXACT_HAZARD_AND_ORACLE_BOUND` | Before implementation, the concrete failure mode and a falsifying oracle were both required. |
| `EXACT_HAZARD_BOUND_ORACLE_MISSING` | The concrete failure mode was named, but no adequate falsifying oracle was required. |
| `GOVERNING_INVARIANT_ONLY` | A governing invariant was bound, but this concrete repair hazard was not named. |
| `NO_RELEVANT_PRE_REPAIR_BOUND` | No relevant hazard, invariant, or oracle was bound before implementation. |
| `NOT_REPAIR_INTRODUCED` | Axis A is not `INTRODUCED_BY_REPAIR`. |
| `UNAVAILABLE` | The exact pre-repair artifacts or chronology do not decide coverage. |

`GOVERNING_INVARIANT_ONLY` and `NO_RELEVANT_PRE_REPAIR_BOUND` do not prove
that a premortem would have predicted the later effect. They only identify
eligible historical subjects for a later blind premortem comparison.

Do not infer origin from report framing, effect evidence from severity words,
oracle quality from a test name, or hazard foreseeability from hindsight.

## Required Analysis Row

Publish a normalized generation-chain table before effect rows so evidence
shared by several findings is recorded once rather than repeated as shallow
prose:

| Field | Required content |
|---|---|
| `chain_id` | stable surface/generation ID |
| `predecessor_identity` | exact candidate and independent report commit |
| `declared_review_scope` | exact prior scope locator |
| `prior_method_flags` | separate `yes`/`no`/`UNAVAILABLE` plus evidence for static inspection, existing-gate execution, targeted reproducer, deliberate oracle reversal, and verified production entry |
| `repair_contract` | exact packet/plan commit and bound obligation locators |
| `repair_delta` | exact range, changed product/test paths, and insertion/deletion counts; no size threshold |
| `implementer_validation` | exact gates, selected subjects, results, and provenance |
| `successor_identity` | exact candidate and independent report commit |
| `chain_censoring` | terminal decision or exact missing successor evidence |

Method flags and repair size are explanatory covariates. Do not call either a
cause from this retrospective corpus.

Then publish a compact effect-row table. Every row contains:

| Field | Required content |
|---|---|
| `row_id` | stable calibration ID, never reused |
| `chain_id` | exact normalized generation-chain row |
| `lineage_generation` | surface and predecessor/successor generation |
| `finding_locator` | exact successor report heading/item and effect sublocator when justified |
| `effect_statement` | one falsifiable wrong effect or exact evidence/routing defect |
| `subject_locator` | exact path/symbol/object and revision |
| `invariant_family` | stable family key plus exact violated-invariant wording/evidence; `UNAVAILABLE` is valid |
| `axis_a` through `axis_g` | one declared rubric value for every axis |
| `axis_evidence_a` through `axis_evidence_g` | exact object, report, packet, command, or reproducer supporting that axis |
| `counterevidence_checked` | exact evidence that could falsify the selected values |
| `reproducer` | safe command/outcome and candidate identity, or `UNAVAILABLE` with reason |
| `oracle_reversal` | deliberate falsification and result, or `NOT_APPLICABLE`/`UNAVAILABLE` |
| `censoring` | whether a later exact disposition exists at the snapshot and what remains unknown |
| `decision_relevance` | which conditional aggregate could consume the row; `none` is valid |

Add a detailed evidence block for:

- every `REPRODUCED_PRODUCTION_EFFECT`;
- every decision-changing `UNAVAILABLE`;
- every policy-promoted or contradictory row;
- every row whose split/merge choice changes a denominator; and
- every row where predecessor scope or origin is disputed by the source
  objects.

The form is for this causal analysis. It is not a proposed production review
template.

## Allowed Evidence And Execution

Use only:

- the exact snapshot tree;
- Git objects named by snapshot-visible reports, packets, and result cards;
- exact predecessor/candidate source and tests;
- safe Python reproductions in isolated temporary copies; and
- deliberate Python oracle falsifications where feasible.

Do not run Cargo, Rust builds, Install, Dogfood, Docker, or network calls other
than the Fugu provider request. Do not edit product, tests, packets, reports,
ledgers, indexes, policy, schemas, tools, issues, or lifecycle state.

If a claim requires newer evidence, a Rust execution, unavailable production
entry point, or an unrepeatable historical environment, record `UNAVAILABLE`.

## Required Report

Write exactly:

```text
docs/development/ai/swarm-results/cfr168-review-root-cause-reliability-classification-fugu-20260731/classification.md
```

The report contains:

1. exact snapshot/tree identity and proof that the excluded artifacts are
   absent at `HEAD`;
2. the 21 discovered report paths and content digests;
3. the normalized generation-chain table and method/delta evidence;
4. reconstruction rules and every row inclusion/exclusion dispute;
5. the complete raw effect-row table with every required field;
6. decision-critical detailed evidence blocks;
7. per-axis aggregates with numerator, denominator, and unavailable count;
8. prior-method and exact invariant-recurrence descriptive measures;
9. the exact set of reproduced-effect rows;
10. every decision-critical unavailable row;
11. threats to validity and right-censoring;
12. conditional statements supported by this calibration only; and
13. explicit non-decisions for template length, finding count, checklist,
    premortem requirement, and process adoption.

Do not open or compare a provisional audit. Do not inspect or classify a
temporal holdout. Those are separate-session tasks.

## Validation And Handoff

Run:

```bash
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_contracts.py
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py
```

If snapshot-era checkers fail because their schemas predate the current
report, record the exact failure; do not import newer checker or policy files.

Commit only the classification report and leave the worktree clean. Return:

- report commit/tree/path/byte count/SHA-256;
- generation-chain count and deterministic chain-identity digest;
- raw effect-row count and deterministic row-identity digest;
- per-axis unavailable counts;
- reproduced-effect row set;
- decision-critical unavailable row set;
- all inclusion/exclusion disputes;
- validation results; and
- confirmation that no provisional or temporal artifact was opened.

The report is immutable input for a later fresh-session comparison. It cannot
adopt a process or change a candidate disposition.
