# CFR-168 Review-Intervention Retroactive Pilot

Status: ready  
Owner: portfolio manager  
Execution role: independent development-process researcher  
Provider/model: Claude / `claude-opus-5`  
Canonical task base: `abfee6ef25f27af35790138404ce6f144b3244da`  
Root-cause audit:
`docs/development/ai/swarm-results/cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md`
at canonical commit `abfee6e`, SHA-256
`2392450e957c8437b323ded492fc7597dc77f5147b74f3056c97c0efd9329780`

## Objective

Measure whether the first four proposed review-recursion interventions would
have exposed later findings before historical candidates were frozen, and
measure their noise, duplication, and evidence burden. This pilot produces
evidence for a later process decision. It does not adopt an intervention or
choose a checklist/template size.

The four interventions are:

1. I3 — bounded repair-delta sweep and new-public-surface refusal;
2. I1 — predecessor-behavior re-derivation;
3. I4 — oracle-admissibility properties E1–E9, scored separately; and
4. I5 — separate evidence/closure state from verdict/routing state.

I2, I6, and I7 are not evaluated here. Their cost or utility depends on this
pilot's result.

## Frozen Questions

The researcher must answer all of these without changing them after seeing
the scores:

1. Which repair-delta regions trade later-finding capture against inability to
   close an already-bound finding? Report a curve, not one preselected limit.
2. Does predecessor re-derivation capture initial misses or repair regressions
   that candidate-local tests miss?
3. Which individual oracle-admissibility properties add signal, and which add
   noise or duplicate an existing obligation?
4. How many historical successor generations were driven only by
   behavior-closed/unproven, claim-only, non-production-reachable,
   requirement-unsatisfiable, or manager-routable states?
5. Do results survive both a lineage holdout and a later-in-time holdout?
6. What information fields, if any, are justified for a future detailed
   review/repair contract? Do not decide field count or depth unless the
   denominators below support it.

## Frozen Corpus

The calibration corpus is the exact 21-report / 22-candidate snapshot used by
the root-cause audit. Reconstruct every tuple from repository objects:

```text
(predecessor tree,
 repair ticket,
 implementer plan/result,
 candidate tree,
 successor report,
 root-cause-audit finding classification)
```

Do not silently drop a tuple. Mark a missing element `unavailable` and keep it
in the denominator.

### Pre-registered lineage holdout

The manager selected one Python and one Rust lineage without looking at pilot
outcomes. Selection used:

```text
sha256("<canonical-task-base>|<lineage-name>")
```

and chose the lexicographically smallest digest in each eligible family.
Eligibility requires at least one successor report in the audit snapshot.

Python candidates and digests:

| Lineage | SHA-256 |
|---|---|
| `python-mr-write` | `38c16469381a2738348ce80bf0b6d1ddab10c9498026040a0d8a06e1dd38ee11` |
| `python-doctor-branch` | `521ae3e326da4f96d1ec10b3c11c3548df5f596a48f004fd82bb776e6c69bf66` |
| `python-mr-list` | `9973258e32573881076ea29fcda6492c7b17d62a970f3c015744f285218e24eb` |
| `python-upload` | `d2193941a4597ee2e6dfd60255e710e2f3b0d33f44f7ef003c836e0e70bcb15b` |

Rust candidates and digests:

| Lineage | SHA-256 |
|---|---|
| `rust-file-doctor` | `6a8542cf8c85b6be17eea50e8799bc333d16e12f9f510d4033d0324b8dc7f497` |
| `rust-storage` | `a3dcdb6e967d6ccfb18ebd98113080227efbf516c491c3d52751e1ecc7702165` |
| `rust-http` | `b2f6f431b970a493e8644e449737613c94f06e2dfa68215e1a6098589f29e0d5` |

Hold out:

- `python-mr-write`;
- `rust-file-doctor`.

Do not use either lineage to choose a delta region, refine an oracle rule,
change a state definition, or select an intervention. Score them only after
the calibration analysis is frozen in the report.

The root-cause audit itself saw these lineages while deriving its proposed
controls. This prevents a fully blind lineage holdout. Record that limitation;
do not hide it.

### Pre-registered temporal holdout

The following reports did not exist in the audit worktree at task base
`c258cbf`. They are stronger later-in-time checks and must not influence any
calibration choice:

| Subject | Report commit | Exact candidate | Report path |
|---|---|---|---|
| Python Upload | `b3de0d7` | `02e04c7e6458035d9c7d58de4d30b4a12e7cd594` | `docs/development/ai/swarm-results/cfr168-python-http-upload-second-review-closure-rereview-claude-fallback-20260731/review.md` |
| KGI/KPI | `5760c98` | `0944bf3deba9e753132b3f71716b5c009a8fc30f` | `docs/development/ai/swarm-results/development-flow-kgi-kpi-r2-rereview-claude-fallback-20260731/review.md` |
| Source-keyed Cargo gate | `b589864` | `aaa39b61b37df273fe4d57e274ed8e1321e788ad` | `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-closure-rereview-claude-20260731/review.md` |

Freeze the calibration conclusions first in the report, then score these
three. If a temporal subject is not comparable to an intervention, mark it
`not-applicable` with a reason rather than manufacturing a score.

## Common Outcome Measures

Every intervention must report:

| Measure | Required definition |
|---|---|
| later-finding capture | later findings the control would have surfaced before freeze / eligible later findings in its causal class |
| noise | control outputs matching neither a later report finding nor the audit finding ledger |
| duplication | control outputs already required by the bound report or repair ticket |
| author burden | additional required evidence artifacts or fields, not guessed seconds |
| reviewer burden | additional review obligations, not guessed seconds |
| false positives | outputs contradicted by an object-database comparison or executed reproducer |
| unavailable | eligible rows that cannot be decided, retained in the denominator |

Publish raw counts and denominators. Do not report percentages without the
corresponding numerator, denominator, and unavailable count.

Use `[P]` for direct object/repository proof, `[R]` for reproduced execution,
`[I]` for inference, `[T]` for task/packet claims, and `[U]` for unavailable.
Separate calibration, lineage-holdout, and temporal-holdout scores.

The baseline to beat is the historical material-correction target of at most
2 of 5 packets; the audit observed correction on 17 of 21. Explain precisely
how each intervention would or would not change that packet-level result.

## I3 Protocol — Delta-Budget Sweep

1. Recompute all 14 successor deltas from Git objects.
2. Identify newly introduced public APIs, state machines, classifiers,
   persistence owners, and dispatch protocols without relying only on names
   listed in the audit.
3. Sweep every distinct observed insertion count as a candidate boundary, plus
   `unbounded`. Do not choose round-number thresholds first.
4. For every boundary report:
   - later findings located in surface excluded by that boundary;
   - bound findings that could not be closed inside it;
   - work that would merely move to another successor;
   - generations-to-terminal where observable;
   - new-public-surface count and changed-symbol count.
5. Produce a Pareto curve. A boundary is not recommended merely because it
   maximizes capture; it must not make necessary repairs impossible or split
   one invariant across unreviewed successors.
6. Freeze the calibration curve and any selection rule before scoring either
   holdout.

## I1 Protocol — Predecessor Re-derivation

Run Python-first. Do not start Cargo or Rust builds.

For every eligible Python successor:

1. load or extract the implicated predecessor implementation from its exact
   tree;
2. run the successor report's smallest reproducer against predecessor and
   candidate where safe;
3. distinguish a pre-existing miss from a repair-introduced behavior change;
4. record whether this obligation adds a new executable artifact or only a
   second invocation of an existing artifact; and
5. score capture only when the difference could have been observed before
   candidate freeze.

Do not claim Rust coverage from text inspection. Score Rust as unavailable
unless a safe, already-proven non-build method establishes the fact.

## I4 Protocol — Oracle Admissibility

Score E1–E9 from audit section 12.3 separately. Never collapse them into one
checklist score.

For each historical closure oracle:

1. bind its exact production subject and candidate;
2. decide each applicable E-rule;
3. identify the later finding it would catch;
4. record admissible oracles the rule would incorrectly reject;
5. record duplication with an already-bound ticket clause; and
6. record whether the rule is mechanically decidable or needs reviewer
   judgement.

Deleting a production line, changing an axis, or injecting a fault may be done
only in temporary copies. Do not commit mutations.

## I5 Protocol — Evidence State vs Routing State

Re-label the audit snapshot's 143 findings using the proposed evidence states
in audit section 12.2. Preserve each original verdict.

For every successor generation, report whether it was required by:

- a reproduced production defect;
- behavior closed but closure evidence unavailable;
- a claim/enforcement mismatch;
- a defect not production-reachable without injection;
- a manager-routable non-blocking risk;
- an unsatisfiable requirement; or
- a mixture.

Compute how many generations were driven only by states that could have been
routed without changing integration authority. `CLOSURE-UNPROVEN` must never
be treated as accepted, and this pilot must not invent `ACCEPT_WITH_NOTES`.

## Decision Rules

For each intervention return one of:

- `PROMISING_FOR_LIVE_EXPERIMENT`;
- `REVISE_AND_REPLAY`;
- `NO_EVIDENCE_TO_ADOPT`.

This is not an adoption decision. A promising intervention still needs a
small live experiment and independent review before becoming policy.

Do not derive a review-item count, checklist length, or template depth unless
all of these are available:

1. a stable I3 capture/burden region on calibration and both holdouts;
2. per-rule I4 capture/noise with no hidden aggregate;
3. the I5 fraction of generations driven only by routable states; and
4. a field-level mapping from each retained field to a measured causal class.

If any prerequisite is unavailable, say so and leave template depth
undecided.

## Write Boundary And Validation

The worker may write only:

```text
docs/development/ai/swarm-results/cfr168-review-intervention-retroactive-pilot-claude-20260731/pilot.md
```

Required report sections:

1. exact task/audit/corpus identities;
2. method and leakage controls;
3. raw candidate/finding dataset;
4. I3 calibration curve and holdout scores;
5. I1 calibration and holdout scores;
6. I4 per-rule calibration and holdout scores;
7. I5 relabelling and generation-routing result;
8. common outcome-measure table;
9. threats to validity;
10. intervention decisions;
11. explicitly undecided template properties.

The worker must:

- use no Codex, Fugu, subagents, network calls, Cargo, Rust, install, or
  dogfood;
- make no product, policy, schema, tool, issue, ledger, index, or existing
  report edit;
- commit only the report;
- leave the worktree clean;
- run `git diff --check`,
  `python3 docs/development/tools/check_ai_contracts.py`, and
  `python3 docs/development/tools/check_project_docs.py`; and
- return the report commit, report SHA-256, exact decision labels, and every
  unavailable denominator.
