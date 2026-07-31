# CFR-168 Canonical Governance Manager Adjudication and Repair Freeze

Status: `REPAIR_AUTHORIZED`

Manager event: `MGR-GOV-20260731-001` at `2026-07-31T23:42:18Z`

This artifact adjudicates the three immutable reviews of governance candidate
`9f0d6bd292d0768d603b6120aae60437383a51e3`, tree
`1fac8e7f7b43bf8201db9c36f7fd007c66897b93`, packet SHA-256
`b5b74a6be1f6abf39eaf2cff4cdd3449b98a056687438cc07cab93e09fd8970c`.
It is separate from the reviewer reports and does not edit their proposals.
The reports were retained by merge commit
`c2f3b935cbcd4a3b5895ad09d348455cdf455c4e`, tree
`8a125b9563794597536a20a5d6f276b0f6ab5c3f`.

## Immutable reviewer inputs

| Review | Report commit | Report SHA-256 | Verdict |
|---|---|---|---|
| operability | `b32befa1d31fd14b04c48370b21ff7efca1312f1` | `95d688fa2f7de2010786806ac12d41743d4f2e1fecae29c4a45f07322766ea88` | `REQUEST_CHANGES` |
| semantic | `b55abcb51437ea67486eb2a0ee56983794d7d5fd` | `dbd162a3d8db6ec384c273065b4c86c13c9d4c26fee36aae1ade25fd1398d605` | `REQUEST_CHANGES` |
| compatibility/migration | `d4216c6a25693ae277812c7e1cd6f0f8d695f049` | `ebaf10045f099ff7f7bc7dd88d4486f9145a646d78609de4a3add28ef4466e0d` | `CHANGES_REQUESTED` |

The manager accepts the reports' exact-candidate identities and referenced
evidence. No acceptance-critical impact is unknown. Repeated observations are
mapped to one canonical decision below; a `duplicate` disposition preserves the
canonical correction and closure state instead of erasing it.

## Canonical decisions

### GOV-D01 — governance baseline inheritance

- Canonical observation: `GSEM-001`.
- Final impact: accept the reviewer's high/material authority and
  development-flow assessment by reference.
- Final must-fix: `MF-1 proved`; all other MF triggers rejected as separately
  unnecessary.
- Final actionability: `required_correction`; `correction_required=true`.
- Causal class: `implementation_nonconformance`. The candidate already names
  `3baa2af` and the one-way authority map as binding, but its inherited-rule
  clause falls back to the older implementation subject and produces a second
  authority interpretation.
- Repair: distinguish implementation subject from dispatch governance
  baseline. Future authorization inherits the declared governance baseline,
  its applicable `AGENTS.md`, and later manager amendments; the old product
  base owns product bytes, not governance semantics.

### GOV-D02 — compact impact-state vocabulary

- Canonical observation: `GSEM-002`.
- Final impact: accept the reviewer's high/material record-shape and authority
  assessment by reference.
- Final must-fix: `MF-1 proved`.
- Final actionability: `required_correction`; `correction_required=true`.
- Causal class: `implementation_nonconformance`. The candidate requires
  explicit non-neutral states but its compact vocabulary cannot represent
  `not_applicable` and conflates evidence availability with impact state.
- Repair: use `affected | unaffected | not_applicable | unknown` for compact
  impact state and keep `available | partial | unavailable` in a separate
  evidence-quality field. An unavailable observation never implies unaffected
  or not applicable.

### GOV-D03 — duplicate metric subjects

- Canonical observation: `GSEM-003`.
- Duplicate observations: `OP-002`, `GCC-003`.
- Final impact: accept the material metric/checker conflict; severity is high
  for the pre-implementation contract and does not itself create repair
  authority.
- Final must-fix: `MF-1 proved`.
- Final actionability: canonical row `required_correction`; the two restatements
  are `duplicate` and delegate to this decision. `correction_required=true` for
  the canonical effect and false for the duplicate rows themselves.
- Causal class: canonical row `implementation_nonconformance`; duplicate rows
  null.
- Repair: retain every duplicate observation in review workload, churn, and
  `MET-REVIEW-011`; create no second unique effect/finding/issue/repair or
  `MET-REVIEW-005` closure opportunity. Rewrite `P1-T17` as a two-sided test so
  both illegal dropping and illegal unique-unit duplication fail.

### GOV-D04 — accepted-variance reopen triggers

- Canonical observation: `GSEM-004`.
- Final impact: accept the high/material cross-generation authority assessment.
- Final must-fix: `MF-1 proved`.
- Final actionability: `required_correction`; `correction_required=true`.
- Causal class: `implementation_nonconformance`.
- Repair: the four canonical triggers are a fixed non-suppressible admissible
  set. A decision may record applicability/evidence but may not remove a later
  `frozen_requirement_changed`, `materially_new_evidence`,
  `new_counterexample`, or `prefrozen_metric_regression` route. Preference-only
  requests still remain rejected churn.

### GOV-D05 — live lifecycle and AI fast-path order

- Canonical observation: `GCC-001`.
- Duplicate observations: `GSEM-005`, `OP-004`.
- Final impact: accept the high/material lifecycle, issue-state, authority, and
  false-completion assessment.
- Final must-fix: `MF-1 proved`; `MF-5 proved` for the contradictory live
  completion route.
- Final actionability: canonical row `required_correction`; duplicate rows
  `duplicate`. `correction_required=true` for the canonical effect.
- Causal class: canonical row `implementation_nonconformance`; duplicate rows
  null. The new lifecycle is already binding in the exact candidate, while two
  live operator routes fail to conform to it.
- Repair: synchronize only live policy/current guidance. Integrate establishes
  source acceptance; checks, applicable Install/Dogfood/Evaluate evidence, raw
  event/metric capture, and issue/current-doc/history synchronization precede
  final Close; Retrospect follows. Completed old-order rows remain labelled
  historical and byte-stable unless a separate evidence correction is proven.

### GOV-D06 — live AIA-047 checkpoint

- Canonical observation: `GSEM-006`.
- Duplicate observation: `GCC-002`.
- Final impact: accept the high/material current-state and next-authority
  assessment.
- Final must-fix: `MF-1 proved`; `MF-5 proved`.
- Final actionability: canonical row `required_correction`; restatement
  `duplicate`. `correction_required=true` for the canonical effect.
- Causal class: canonical row `implementation_nonconformance`; duplicate row
  null. The pre-existing management-hub precedence and update rule were
  binding, but the candidate updated only the lower AI checkpoint.
- Repair: preserve the historical fact that the experimental foundation was
  accepted at `00ad885`; update the live hub first, then active-cycle and live
  strategy summaries, to say authority hardening is reopened/in progress and
  L1.6 remains blocked. Append one new history row; do not rewrite the old
  integration row or immutable reports.

### GOV-D07 — compact versus expanded depth

- Canonical observation: `OP-001`.
- Duplicate observation: `GCC-005`.
- Final impact: accept the material review-burden and future checker conflict.
- Final must-fix: `MF-1 proved`.
- Final actionability: canonical row `required_correction`; restatement
  `duplicate`. `correction_required=true` for the canonical effect.
- Causal class: canonical row `implementation_nonconformance`; duplicate row
  null.
- Repair: every observation carries the complete compact screen. The expanded
  matrix exists exactly when the frozen expansion trigger is true, with the
  existing canonical-reference rule for a true duplicate. Remove every
  universal-full-matrix sentence that contradicts this rule.

### GOV-D08 — `MET-FLOW-014` raw derivation

- Canonical observation: `GCC-004`.
- Final impact: accept the high/material measurement and closed-schema design
  assessment.
- Final must-fix: `MF-1 proved`; `MF-5 proved`.
- Final actionability: `required_correction`; `correction_required=true`.
- Causal class: `implementation_nonconformance`.
- Repair: P1 must add paired same-boot evidence-active intervals and an
  activity class that can identify post-rejection repair/retest work. Service
  is the union within one lane followed by a sum of lane-seconds across lanes;
  it is not a cross-lane elapsed-time union. Missing, cross-clock, overlapping,
  and concurrent-lane cases must derive explicit unavailable or exact results.

### GOV-D09 — repair batching clarification

- Canonical observation: `OP-003`.
- Final impact: accept the moderate/nonmaterial development-flow observation.
- Final must-fix: none proved; no acceptance-critical unknown.
- Final actionability: `nonblocking_improvement`;
  `correction_required=false`; causal class and repair authority null.
- Manager decision: retain as a next-Discovery candidate named
  `multi-correction-repair-batching`. It is not admitted into this repair and
  cannot delay the required corrections. A later Freeze may define
  composition by dependency/write/hazard compatibility with per-correction
  closure evidence.

## Observation index

| Observation | Impact gate | Proved MF IDs | Final actionability | Causal class or null | Active decision | Repair ref or null | Reopen result |
|---|---|---|---|---|---|---|---|
| `GSEM-001` | complete | MF-1 | required_correction | implementation_nonconformance | GOV-D01 | REPAIR-GOV-01 | not_requested |
| `GSEM-002` | complete | MF-1 | required_correction | implementation_nonconformance | GOV-D02 | REPAIR-GOV-01 | not_requested |
| `GSEM-003` | complete | MF-1 | required_correction | implementation_nonconformance | GOV-D03 | REPAIR-GOV-01 | not_requested |
| `GSEM-004` | complete | MF-1 | required_correction | implementation_nonconformance | GOV-D04 | REPAIR-GOV-01 | not_requested |
| `GSEM-005` | complete | inherited from GCC-001 | duplicate | null | GOV-D05 | GCC-001 | not_requested |
| `GSEM-006` | complete | MF-1, MF-5 | required_correction | implementation_nonconformance | GOV-D06 | REPAIR-GOV-01 | not_requested |
| `OP-001` | complete | MF-1 | required_correction | implementation_nonconformance | GOV-D07 | REPAIR-GOV-01 | not_requested |
| `OP-002` | complete | inherited from GSEM-003 | duplicate | null | GOV-D03 | GSEM-003 | not_requested |
| `OP-003` | complete | none | nonblocking_improvement | null | GOV-D09 | null | not_requested |
| `OP-004` | complete | inherited from GCC-001 | duplicate | null | GOV-D05 | GCC-001 | not_requested |
| `GCC-001` | complete | MF-1, MF-5 | required_correction | implementation_nonconformance | GOV-D05 | REPAIR-GOV-01 | not_requested |
| `GCC-002` | complete | inherited from GSEM-006 | duplicate | null | GOV-D06 | GSEM-006 | not_requested |
| `GCC-003` | complete | inherited from GSEM-003 | duplicate | null | GOV-D03 | GSEM-003 | not_requested |
| `GCC-004` | complete | MF-1, MF-5 | required_correction | implementation_nonconformance | GOV-D08 | REPAIR-GOV-01 | not_requested |
| `GCC-005` | complete | inherited from OP-001 | duplicate | null | GOV-D07 | OP-001 | not_requested |

Every reviewer observation occurs exactly once. Duplicate rows remain in
workload/churn/actionability evidence but add no unique correction, issue,
repair, or recurrence-after-closure unit. There is no undecided impact or
actionability state.

## Frozen repair and rereview authority

Repair episode: `REPAIR-GOV-01`.

Authorized correction decisions: `GOV-D01` through `GOV-D08` only.

Allowed paths:

- `docs/development/README.md`
- `docs/development/current-cycle-burn-down-plan.md`
- `docs/development/history.md`
- `docs/development/ai/README.md`
- `docs/development/ai/acceleration-master-plan.md`
- `docs/development/ai/development-flow-kgi-kpi.md`
- `docs/development/ai/operating-protocol.md`
- `docs/development/ai/validation-report-template.md`
- `docs/development/ai/task-packets/cfr168-review-bottleneck-operating-controls-freeze-20260731.md`

Forbidden paths and surfaces:

- every product, test, issue-ledger, schema, checker, runtime, provider-routing,
  role-routing, historical review-report, and prior-history-row byte;
- changing current issue status or claiming P1/P2 machine enforcement;
- editing a reviewer report or erasing its original requested-change verdict;
- admitting `GOV-D09` into the repair; and
- rewriting old lifecycle evidence as though the new order had been followed.

Required positive validation:

1. `python3 docs/development/tools/check_ai_acceleration_docs.py`
2. `python3 docs/development/tools/check_project_docs.py`
3. `python3 docs/development/tools/check_ai_contracts.py`
4. `python3 docs/development/tools/check_ai_development.py`
5. `git diff --check`
6. counterexample walkthroughs for old live Close order, compact N/A and
   unavailable evidence, duplicate observation/unique-effect denominators,
   all four accepted-variance reopen triggers, conditional expansion, and
   complete `MET-FLOW-014` interval derivation.

Required negative controls and hazards:

- the old `00ad885` history row and all three reports remain text-identical;
- completed historical lifecycle rows remain evidence-identical;
- no unavailable impact is normalized to unaffected/not-applicable;
- no manager-selected trigger subset can suppress material new evidence;
- no duplicate is dropped from `MET-REVIEW-011` or double-counted as a unique
  effect/closure opportunity;
- cross-clock or incomplete intervals never emit a duration; and
- current-state synchronization does not claim hardening integration, live
  `ALLOW`, effectiveness, or permanent adoption.

Repair structure:

1. update canonical/live governance and current-state owners, then commit;
2. align the subordinate task packet to that exact governance commit, then
   commit; and
3. freeze the new exact candidate and run one fresh Ultra exact rereview that
   covers the repair delta, all eight inherited corrections, the two duplicate
   joins, the nonblocking exclusion, and the affected lifecycle/metric/schema
   surfaces.

Any new public/persisted/product surface, schema/checker implementation, issue
status change, distinct effect, or need to edit a forbidden path returns to a
manager scope decision before mutation. The original required-correction
decisions remain immutable; a successful repair will close them only through
the successor candidate and rereview evidence.
