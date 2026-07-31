# CFR-168 Review Root-Cause Reliability Comparison And Temporal Check

Status: blocked on committed blind-calibration report and exact digest
Owner: portfolio manager
Provider/model: Fugu / `fugu-ultra-v1.1`
Role: independent corpus comparator
Codex fallback: forbidden
Product/process mutation: forbidden

## Admission Inputs

Do not launch until the manager binds all of these values:

- blind-calibration report commit and tree;
- report path, byte count, and SHA-256;
- generation-chain count and deterministic chain-identity digest;
- effect-episode count and deterministic episode-identity digest;
- blind observation-row count;
- deterministic blind row-identity digest;
- blind reproduced-effect row set;
- blind decision-critical-unavailable row and episode-identity sets; and
- the exact comparison base containing the immutable blind report.

Required blind report:

```text
docs/development/ai/swarm-results/cfr168-review-root-cause-reliability-classification-fugu-20260731/classification.md
```

Do not create a `dev-swarm.v1` manifest while any value is unbound.

## Objective

Using a fresh session, compare the immutable blind calibration with the
provisional audit without changing either source. Reconcile normalized
generation chains, continuous effect episodes, and observation-row identity
before comparing causal labels. Then content-freeze the comparison and apply
the unchanged seven-axis rubric and episode-identity rules to the three
temporal reports.

This task decides what evidence enters the later Claude adjudication queue. It
does not decide the final causal class or any process intervention.

## No Count-Based Pass Or Fail

The provisional audit reports 51 successor findings. The blind classifier was
not told that number.

A blind count different from 51 is neither automatically an omission nor
automatically a better decomposition. Reconcile every difference by exact
report locator and independently testable effect:

| Relation | Meaning |
|---|---|
| `EXACT_ROW_MATCH` | Same report locator, effect, and subject. |
| `SEMANTIC_ROW_MATCH` | Different stable wording or sublocator, but the same falsifiable effect and subject. |
| `BLIND_ONLY` | Blind row has no provisional counterpart. |
| `PROVISIONAL_ONLY` | Provisional row has no blind counterpart. |
| `SPLIT_DIFFERENCE` | One source separates effects that the other combines. |
| `MERGE_DIFFERENCE` | One source combines effects with different subjects, evidence, or axis values. |
| `NON_COMPARABLE` | The provisional narrative has no seven-axis observation-row equivalent. |
| `UNAVAILABLE` | Exact source locators do not decide the relation. |

For every non-exact relation, state:

- both exact source locators;
- the independently falsifiable effects;
- whether subjects, evidence, or axis values differ;
- the normalized episode/row identities used for aggregate sensitivity; and
- whether the difference can change a causal conclusion or intervention.

Do not add, split, delete, or relabel a blind row in its source report.

Separately reconcile effect episodes. Two observations belong to one episode
only when they assert the same falsifiable effect on the same subject and
exact object, source, or reproducer evidence establishes continuity. A repair
claim, unrelated passing gate, or verdict text is not closure evidence, but
absence of closure evidence is not persistence evidence. A recurrence after
proven closure is a new linked episode; the same invariant on another subject
is a different episode. Record `UNAVAILABLE` when neither continuity nor
closure is established rather than selecting whichever merge or split makes
an aggregate cleaner.

## Axis Comparison

For every exact or semantic row match:

1. map the provisional audit's evidence and taxonomy to axes A through G;
2. record `AGREE`, `DISAGREE`, `NON_COMPARABLE`, or `UNAVAILABLE` separately
   for every axis;
3. preserve both original values and exact evidence locators;
4. distinguish a taxonomy mismatch from an object/evidence disagreement; and
5. route every decision-relevant disagreement to adjudication.

The provisional audit's narrative "primary cause" never overrides the blind
orthogonal axes. Agreement between agents is not proof. A provisional class
with no seven-axis equivalent remains `NON_COMPARABLE`. In particular,
`PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED` must be mapped from exact
pre-repair artifacts to axis G; its hindsight wording is not evidence that a
blind premortem would have predicted the effect.

## Comparison Freeze

Before opening any temporal report:

1. finish the complete generation-chain, episode-identity, and row-identity
   reconciliation;
2. finish the per-axis comparison;
3. freeze the normalized episode/row map, comparison rules, disagreements,
   unavailable values, and observation-, episode-, and generation-level
   aggregate sensitivity in
   `/tmp/cfr168-review-root-cause-comparison-freeze.md`;
4. emit its byte count and SHA-256 in one event-log tool result; and
5. do not edit the freeze afterward.

If the digest cannot be reproduced, stop without opening the temporal reports.

## Temporal Check

Only after the comparison freeze, apply the unchanged blind rubric and
evidence thresholds to:

| Subject | Report commit | Candidate | Report path |
|---|---|---|---|
| Python Upload | `b3de0d7` | `02e04c7e6458035d9c7d58de4d30b4a12e7cd594` | `docs/development/ai/swarm-results/cfr168-python-http-upload-second-review-closure-rereview-claude-fallback-20260731/review.md` |
| KGI/KPI | `5760c98` | `0944bf3deba9e753132b3f71716b5c009a8fc30f` | `docs/development/ai/swarm-results/development-flow-kgi-kpi-r2-rereview-claude-fallback-20260731/review.md` |
| Source-keyed Cargo gate | `b589864` | `aaa39b61b37df273fe4d57e274ed8e1321e788ad` | `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-closure-rereview-claude-20260731/review.md` |

Do not change a category, episode rule, row rule, evidence threshold,
unavailable rule, or calibration aggregate after seeing a temporal effect.
Temporal episodes and rows use separate ID namespaces and denominators unless
exact pre-frozen identity rules prove continuation of a calibration episode.

Report:

- temporal raw rows;
- temporal effect episodes and any exact continuation links;
- every unavailable value;
- whether each calibration evidence pattern recurs, fails to recur, or is
  undecidable;
- any new axis combination;
- every rule that would have needed retuning; and
- right-censoring.

## Adjudication Queue

Queue exactly:

1. every decision-relevant row-identity disagreement;
2. every decision-relevant episode-identity or closure disagreement;
3. every multi-observation episode link, `PROVEN_CLOSED` claim,
   `NEW_EPISODE_AFTER_PROVEN_CLOSURE` relation, or episode-identity
   `UNAVAILABLE` that changes an episode denominator, even when the two source
   narratives appear consistent;
4. every decision-relevant axis disagreement;
5. every blind or temporal `REPRODUCED_PRODUCTION_EFFECT`;
6. every decision-critical `UNAVAILABLE`;
7. every `POLICY_PROMOTED_BLOCKER`;
8. every `UNSATISFIABLE_OR_CONTRADICTORY`; and
9. every other split/merge decision that changes an aggregate.

Do not queue agreed, non-reproduced, non-decision-critical rows merely to
increase volume. The episode items above are not a volume sample: they are the
complete set of semantic decisions that change the unique-episode
denominator. Publish the ordered queue and a deterministic identity digest.

## Authorized Output

Write exactly:

```text
docs/development/ai/swarm-results/cfr168-review-root-cause-reliability-comparison-fugu-20260731/comparison.md
```

Do not edit either source report, the provisional audit, product, tests,
packets, ledgers, indexes, policy, schemas, tools, issues, lifecycle state, or
the frozen pilot.

Safe Python reproductions may run in isolated temporary copies when needed to
decide row identity. Do not run Cargo, Rust builds, Install, Dogfood, Docker,
or network calls other than the Fugu provider request.

## Required Report

The report contains:

1. exact task, blind-report, provisional-audit, and temporal identities;
2. blind chain-, episode-, and row-identity digest verification;
3. complete generation-chain, effect-episode, and row-identity reconciliation;
4. per-row/per-axis comparison with both source values and evidence;
5. comparison-freeze byte count, SHA-256, and event locator;
6. raw observation-, episode-, and generation-level comparison aggregates
   with numerator, denominator, and unavailable;
7. temporal episodes and raw rows under the unchanged rules;
8. temporal stability and retuning-pressure results;
9. exact adjudication queue and identity digest;
10. threats to validity and censoring;
11. conditional evidence statements only; and
12. explicit non-decisions for template shape, detail count, finding count,
    checklist, premortem requirement, routing policy, and process adoption.

The report may conclude that the sources cannot be reconciled or that the
evidence is insufficient.

## Validation And Handoff

Run:

```bash
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_contracts.py
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py
```

Commit only the comparison report and leave the worktree clean. Return:

- report commit/tree/path/byte count/SHA-256;
- episode- and row-reconciliation counts by relation;
- per-axis agreement/disagreement/unavailable counts;
- comparison-freeze byte count/SHA-256/event locator;
- temporal episode/row counts and stability result;
- exact adjudication queue and identity digest;
- all threats to validity; and
- every validation result.
