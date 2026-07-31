# CFR-168 Review Root-Cause Reliability Gate

Status: ready; provider launch held by the recorded Claude and Fugu backoffs  
Owner: portfolio manager  
Execution role: independent method auditor, followed by disagreement-only adjudication  
Canonical task base: `f65459d54a9075c00494f05e4517c67bf3d55852`  
Product mutation: forbidden  
Policy adoption: forbidden

## Decision Question

Determine what the CFR-168 review-recursion evidence actually supports before
changing the review template, demanding a fixed number of findings, or adopting
any proposed intervention.

The current evidence does **not** establish absolute review recall or precision.
It contains a complete census of findings that appeared in known successor
reports, but it cannot contain defects that no reviewed generation discovered.
Several lineages are also right-censored before a successor review. Therefore
this gate may estimate the composition of **observed later findings**; it must
not rename that quantity "all review omissions" or "review recall".

## Why This Gate Exists

The root-cause audit at
`docs/development/ai/swarm-results/cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md`
is useful evidence, but it is one analyst's retrospective classification. Its
own threats-to-validity section records hand-derived counts, a same-corpus
intervention design, missing execution, and censored lineages.

Two additional limitations require explicit correction before a process
decision:

1. absence of a later withdrawal is not proof that a finding was a true
   positive; the corpus contains many findings that have not reached a
   terminal independently reproduced decision;
2. a successor-finding corpus can identify an observed predecessor miss, but
   cannot provide the denominator needed for absolute recall.

The existing preregistered intervention pilot remains frozen and may resume
unchanged. Its result is sensitivity evidence against the original labels, not
an adoption decision. If this gate changes any decision-relevant label, a
separate sensitivity replay must score the affected rows without rewriting the
frozen calibration.

## No Arbitrary Item Count

This task does not ask an agent to invent 100, or any other fixed number, of
review findings.

- The 51 successor findings in the audit snapshot are included because they
  are the complete observed successor-finding census at that snapshot.
- No reviewer receives credit for adding bullets.
- One finding is one row only when it has one stable report locator.
- Splitting a finding for volume is a protocol violation.
- Combining independently testable effects is also a protocol violation.
- Missing evidence is recorded as `UNAVAILABLE`; it is not replaced by prose.

A future template field or checklist item is justified only if a measured
causal class needs it and a holdout shows useful capture at acceptable burden.
This gate does not decide template length or shape.

## Frozen Source Boundary

### Calibration snapshot

Reconstruct the exact report/candidate universe used by the root-cause audit:

- 21 independent reports;
- 22 candidate generations;
- 51 findings that first appear in a successor report; and
- the exact predecessor, repair packet, implementer result, candidate, and
  successor report for each finding.

Use Git objects and the review disposition index to reconstruct identities.
Do not read the root-cause audit's per-finding classifications, cross-lineage
totals, intervention ranking, or the interrupted pilot's frozen calibration
until the blind classification is content-frozen.

### Temporal check

After blind calibration is frozen, apply the unchanged classification rules to
the reports that were absent from the audit snapshot:

| Subject | Report commit | Candidate |
|---|---|---|
| Python Upload | `b3de0d7` | `02e04c7e6458035d9c7d58de4d30b4a12e7cd594` |
| KGI/KPI | `5760c98` | `0944bf3deba9e753132b3f71716b5c009a8fc30f` |
| Source-keyed Cargo gate | `b589864` | `aaa39b61b37df273fe4d57e274ed8e1321e788ad` |

Do not use the temporal rows to redefine a category or repair a calibration
disagreement.

## Orthogonal Classification

Do not assign one narrative "primary cause". Record every axis independently.
Each non-`UNAVAILABLE` value requires an exact object, symbol, report locator,
or executed reproducer.

### A. Code origin

| Value | Meaning |
|---|---|
| `PREEXISTING_IN_PREDECESSOR` | The implicated behavior or code shape exists in the immediately preceding reviewed candidate. |
| `INTRODUCED_BY_REPAIR` | The implicated behavior or code shape is absent from that predecessor and introduced by the repair. |
| `NO_PRODUCT_DEFECT_ASSERTED` | The finding concerns evidence, documentation, policy, or a non-production-reachable claim rather than an asserted product defect. |
| `UNAVAILABLE` | The available objects or evidence do not decide origin. |

### B. Prior-review detectability

| Value | Meaning |
|---|---|
| `EXPLICITLY_BOUND_BEFORE_REPAIR` | The predecessor report or repair packet already states the exact obligation. |
| `WITHIN_DECLARED_PRIOR_SCOPE` | Not explicitly bound, but the prior review declares the implicated path/invariant in scope and the defect existed then. |
| `OUTSIDE_DECLARED_PRIOR_SCOPE` | The prior review's declared scope excludes the implicated path/invariant. |
| `NOT_PRESENT_AT_PRIOR_REVIEW` | The repair introduced the implicated surface after the prior review. |
| `NOT_APPLICABLE` | No product-defect detectability claim is made. |
| `UNAVAILABLE` | Scope or temporal identity cannot be decided. |

Only `PREEXISTING_IN_PREDECESSOR` plus
`WITHIN_DECLARED_PRIOR_SCOPE` supports the bounded statement "observed prior
review scope miss". It still does not establish absolute recall.

### C. Repair-contract handling

| Value | Meaning |
|---|---|
| `CLOSED_AS_WRITTEN` | The exact bound obligation is satisfied. |
| `NOT_CLOSED_AS_WRITTEN` | The exact obligation remains false. |
| `WEAKENED_OR_RESTATED` | The implementation or evidence silently substitutes a weaker property. |
| `CRITERION_CHANGED_AFTER_REPAIR` | A later decision changes the obligation rather than proving the original wording. |
| `NOT_BOUND` | No earlier exact obligation exists. |
| `UNAVAILABLE` | Closure cannot be decided. |

This axis distinguishes implementation quality from communication failure.

### D. Effect evidence

| Value | Meaning |
|---|---|
| `REPRODUCED_PRODUCTION_EFFECT` | A minimal reproducer reaches production code and demonstrates an externally or durably observable wrong effect. |
| `STATIC_PRODUCTION_REACHABLE_PROOF` | Production reachability and the defective state follow from exact source/object evidence, without an executed reproducer. |
| `INJECTION_ONLY_EFFECT` | The effect requires a state or fault for which no production route is demonstrated. |
| `CLAIM_OR_EVIDENCE_ONLY` | Behavior may be correct; a test, document, or proof overclaims it. |
| `UNAVAILABLE` | No decision-grade effect evidence exists. |

Do not infer this value from severity words such as MUST, HIGH, or BLOCKER.

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

## Analysis Row Form

This is an audit data form, not a proposed review or repair template. Its
purpose is to prevent a classifier from satisfying the task with a long list
of unsupported bullets.

Publish one indexed row per independently testable reported effect. A combined
report item may use stable sublocators such as `F2/effect-a` and
`F2/effect-b` only when the effects have different subjects, evidence, or axis
values. Splitting one effect into wording variants to increase volume is a
protocol violation.

Every row contains:

| Field | Required content |
|---|---|
| `row_id` | stable audit ID; never reused after content freeze |
| `cohort` | `calibration` or `temporal`; no holdout value is visible during calibration rule formation |
| `lineage_generation` | surface and predecessor/successor generation |
| `object_chain` | predecessor candidate/report, repair packet/result, successor candidate/report commits |
| `finding_locator` | exact report heading/item plus an effect sublocator only when justified above |
| `effect_statement` | one falsifiable wrong effect or one exact evidence/routing defect; no severity rhetoric |
| `subject_locator` | exact path/symbol/object and revision |
| `axis_a` through `axis_f` | one declared value for each orthogonal axis |
| `axis_evidence_a` through `axis_evidence_f` | exact object, report, packet, command, or reproducer locator supporting each value |
| `counterevidence_checked` | predecessor/candidate locations or executions that could falsify the selected values |
| `reproducer` | exact safe command/outcome and candidate identity, or `UNAVAILABLE` with reason |
| `oracle_reversal` | exact deliberate falsification and result, or `NOT_APPLICABLE`/`UNAVAILABLE` |
| `censoring` | whether a later exact disposition exists and what remains unknown |
| `decision_relevance` | which aggregate or intervention card can consume the row; `none` is valid |

An axis value and its evidence must be separate fields. Repeating a report's
conclusion is not origin proof, a severity word is not effect proof, and an
unexecuted imagined reproducer is not a reproducer. If one exact locator does
not decide an axis, record `UNAVAILABLE`; do not fill the cell with general
project context.

The classification report must first publish a compact raw-row table. It then
adds a detailed evidence block for:

- every row with a reproduced production effect;
- every row with any `UNAVAILABLE` value that changes an aggregate;
- every row that disagrees with the original audit;
- every unsatisfiable/contradictory or policy-promoted blocker; and
- every row where splitting or combining effects could change a denominator.

The detailed blocks explain the object comparison or execution sufficiently
for the adjudicator to repeat it. Requiring detail for decision-critical rows
does not create credit for producing more rows.

## Blind Analysis Protocol

1. Reconstruct the 51-row census without opening the existing cause labels.
2. Classify all six axes and attach evidence locators.
3. Content-freeze the row set, rules, counts, and unavailable values.
4. Only then open the existing audit and build an axis-by-axis comparison.
5. Report every disagreement; do not hide it inside an aggregate.
6. A second agent adjudicates only disagreements and all rows marked
   `REPRODUCED_PRODUCTION_EFFECT`.
7. Apply the frozen axes to the temporal check.

Preferred low-Codex routing after provider recovery:

- blind classifier: Fugu `fugu-ultra-v1.1`;
- disagreement and reproduced-effect adjudicator: fresh Claude
  `claude-opus-5` session;
- Codex: no worker fallback; manager records the final process decision only
  if evidence conflicts after adjudication.

If Fugu remains unavailable, a fresh Claude session may perform the blind
classification, but the shared-model limitation must be recorded and no
same-session author/reviewer pairing is allowed.

## Required Measures

Publish raw rows before aggregates. For each aggregate include numerator,
denominator, and `UNAVAILABLE`.

1. observed prior-review scope misses;
2. repair-introduced defects;
3. exact bound obligations not closed;
4. weakened/restated obligations;
5. criteria changed after repair;
6. reproduced production effects by code origin and detectability;
7. evidence-only or injection-only blockers;
8. oracle failures by effect-evidence state;
9. policy-promoted or unsatisfiable successor generations;
10. agreement and disagreement with the original audit, per axis;
11. temporal-check stability.

Do not publish a "review precision" percentage from withdrawal text. A
precision claim requires independent disposition of sampled findings as true
positive, false positive, or unavailable. Do not publish absolute recall
without a candidate defect denominator produced independently of the reviewed
reports.

## Causal Decision Map

The report must state conditional conclusions, not jump from one aggregate to
one universal remedy.

| Evidence pattern | Supported next experiment |
|---|---|
| Severe reproduced effects are mostly pre-existing and within prior scope | Compare review methods on hidden historical candidates. |
| Severe reproduced effects are mostly introduced by repair | Compare bounded repair/design-review strategies and diff-scoped review. |
| Exact bound obligations are commonly weakened or left open | Test requirement-to-plan-to-test traceability and communication handoff. |
| Evidence-only or policy-promoted blockers drive whole generations | Test evidence/routing vocabulary without relaxing integration authority. |
| Oracles pass deliberate falsifications | Test only the implicated oracle properties, individually. |
| One invariant recurs across surfaces | Test shared-contract review before another surface-local repair. |
| Findings remain mostly `UNAVAILABLE` | Improve evidence capture first; do not redesign review from anecdotes. |

Multiple patterns may hold. The output may recommend multiple bounded
experiments; it may not collapse them into "make review more exhaustive".

## Process-Change Gate

No review template, mandatory finding count, repair-delta threshold, severity
vocabulary, or checklist is adopted until:

1. every decision-critical disagreement and every reproduced-effect row is
   adjudicated or explicitly `UNAVAILABLE`;
2. the temporal check is reported without retuning;
3. the existing intervention pilot completes or is explicitly retired;
4. any changed root-cause labels receive a sensitivity replay;
5. proposed fields map one-to-one to a measured failure mode; and
6. a small live experiment reports capture, noise, duplication, author burden,
   reviewer burden, and false positives.

The result may legitimately be "insufficient evidence to change the process".

## Authorized Output

The blind classifier may write only:

```text
docs/development/ai/swarm-results/cfr168-review-root-cause-reliability-classification-fugu-20260731/classification.md
```

The adjudicator may write only:

```text
docs/development/ai/swarm-results/cfr168-review-root-cause-reliability-adjudication-claude-20260731/adjudication.md
```

Both lanes are report-only, must commit only their authorized report, and must
leave their worktrees clean. They may run Python reproductions in isolated
temporary copies. They must not run Cargo/Rust builds, mutate product files,
edit the audit or frozen pilot, use network calls, or change any project
policy.
