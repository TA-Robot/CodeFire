# CFR-168 Review Root-Cause Reliability Blind Classification

Status: logically ready; launch held until Fugu usage reset and bounded health probe  
Owner: portfolio manager  
Provider/model: Fugu / `fugu-ultra-v1.1`  
Role: independent method classifier  
Codex fallback: forbidden  
Product/process mutation: forbidden

## Exact Task Base

| Item | Exact value |
|---|---|
| Canonical base | `f259e9488e9676af331b2fed45d15209feec0e7b` |
| Base tree | `143ff061ada99533d587b9eb77c4f8811282535d` |
| Reliability gate | `docs/development/ai/swarm-batches/cfr168-review-root-cause-reliability-gate-20260731.md` |
| Gate SHA-256 | `cdc07044dde70b33549c7708ad3a7c3635585b3895d649ba89d49bf0a59131da` |
| Existing provisional audit SHA-256 | `2392450e957c8437b323ded492fc7597dc77f5147b74f3056c97c0efd9329780` |
| Review-index SHA-256 | `890c5e0b128baa8b0e369afcf8e95079780dfb3315ea4a760b6fb2e7e45da855` |
| Recovery-ledger SHA-256 | `73ad5ce65606ae53189796a74babc6c5dccf7bda7588adea53440a5d66764beb` |

Use a fresh session. Do not resume any earlier Fugu review or audit session.
Those sessions are superseded and stopped on the separately recorded provider
usage limit.

## Objective

Independently determine what the observed successor-finding corpus supports
about:

- pre-existing behavior inside earlier declared review scope;
- repair-introduced behavior;
- exact repair obligations closed, weakened, changed, or left open;
- production effects versus evidence, injection, or policy-only findings;
- closure-oracle quality; and
- review routing.

Do not assume that review omission is the cause. Do not choose one narrative
primary cause. Do not propose a finding count, checklist, review-template
depth, or process change.

## Finite Corpus, Not A Quota

Reconstruct the exact calibration snapshot:

- 21 independent reports;
- 22 candidate generations; and
- 51 findings first observed in successor reports.

The 51 rows are the complete observed successor-finding census at the frozen
snapshot, not a requested output count and not a denominator for all defects.
Never split an effect to reach 51. Never merge effects with different
subjects/evidence/axis values merely to stay at 51. Any reconstruction
difference is reported and resolved from exact report locators before
aggregation.

## Contamination Boundary

Before opening any per-finding classification, cross-lineage total,
intervention ranking, or conclusion in
`cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md`:

1. reconstruct the report/candidate/repair chain from Git objects, the
   disposition index, packets, results, and original reports;
2. classify every calibration effect on all six axes in the reliability gate;
3. attach the complete analysis-row fields and evidence locators;
4. write the raw rows, rules, unavailable values, and aggregates to
   `/tmp/cfr168-review-root-cause-blind-freeze.md`;
5. compute and emit its byte count and SHA-256 in one event-log tool result;
   and
6. state explicitly that calibration content is frozen.

Only then read the provisional audit classifications and construct the
comparison. Do not edit the freeze. If it is lost, recover its exact bytes from
the event log; if its digest cannot be reproduced, restart the blind phase.

The temporal reports remain unopened until the original-audit comparison and
all calibration disagreements are frozen.

## Required Classification

Apply the reliability gate literally:

- A — code origin;
- B — prior-review detectability;
- C — repair-contract handling;
- D — effect evidence;
- E — closure-oracle quality; and
- F — review-routing state.

Each axis value has its own evidence field. A report's framing is not proof of
origin; a severity word is not proof of effect; a test name is not proof of
oracle strength. Use `UNAVAILABLE` rather than project-general prose.

For every row:

1. resolve exact predecessor/candidate/report objects;
2. locate the implicated path/symbol/effect at each relevant revision;
3. inspect the earlier report's declared scope;
4. compare exact repair wording with implementation/result/oracle;
5. reproduce safe Python effects when needed and feasible;
6. otherwise distinguish static production reachability, injection-only,
   claim-only, and unavailable evidence;
7. inspect or execute deliberate oracle falsification when feasible; and
8. record counterevidence that could falsify the chosen axis values.

No Cargo, Rust build, Install, Dogfood, Docker, product edit, or network call
other than the Fugu provider request is authorized.

## Post-Freeze Comparison

After the blind calibration freeze:

1. open the existing audit;
2. map its claims to the six axes without changing the blind values;
3. list every row/axis agreement, disagreement, and non-comparable claim;
4. explain whether disagreement comes from object identity, scope,
   obligation wording, effect evidence, oracle evidence, routing, or
   unavailable data;
5. flag every `REPRODUCED_PRODUCTION_EFFECT` row for Claude adjudication; and
6. flag every disagreement whose disposition could change an intervention.

Do not publish a generic agreement percentage without the numerator,
denominator, unavailable count, and per-axis confusion table. Do not call
absence of a later withdrawal precision.

## Temporal Check

Only after calibration/comparison freeze, apply unchanged rules to:

| Subject | Report commit | Candidate |
|---|---|---|
| Python Upload | `b3de0d7` | `02e04c7e6458035d9c7d58de4d30b4a12e7cd594` |
| KGI/KPI | `5760c98` | `0944bf3deba9e753132b3f71716b5c009a8fc30f` |
| Source-keyed Cargo gate | `b589864` | `aaa39b61b37df273fe4d57e274ed8e1321e788ad` |

Do not change a category or evidence threshold after seeing these rows.
Report stability and every `UNAVAILABLE` value.

## Authorized Write

Write exactly:

```text
docs/development/ai/swarm-results/cfr168-review-root-cause-reliability-classification-fugu-20260731/classification.md
```

Do not edit product, tests, packets, existing reports, ledgers, indexes,
policy, schemas, tools, issues, or lifecycle state.

## Required Report

The report contains:

1. exact task/gate/base/index/ledger identities and digests;
2. blind protocol and freeze byte count/SHA-256/event locator;
3. source reconstruction and any row-count discrepancy;
4. compact raw calibration rows using every required analysis field;
5. required detailed evidence blocks;
6. per-axis raw aggregates with numerator, denominator, and unavailable;
7. row/axis comparison with the provisional audit;
8. adjudication queue containing all disagreements and reproduced effects;
9. unchanged temporal-check rows and stability;
10. threats to validity and right-censoring;
11. supported conditional statements only; and
12. explicit non-decisions for template length, item count, checklist, and
    process adoption.

The report must not include a universal remedy. It may conclude that evidence
is insufficient.

## Validation And Handoff

Run:

```bash
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_contracts.py
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py
```

Commit only the classification report and leave the worktree clean. Return:

- report commit/tree/path/SHA-256;
- blind-freeze byte count/SHA-256/event locator;
- raw row and unavailable counts;
- per-axis disagreement counts;
- exact adjudication row set;
- temporal stability result; and
- every validation command and result.

The report is evidence for a later Claude disagreement adjudication. It
cannot adopt a review process or change any candidate disposition.

