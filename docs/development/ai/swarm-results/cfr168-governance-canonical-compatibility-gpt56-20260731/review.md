# CFR-168 Canonical Governance Compatibility and Migration Review

Review mode: independent, report-only, candidate mutation forbidden.

## 1. Immutable subject

| Field | Requested | Recomputed | Result |
|---|---|---|---|
| candidate commit | `9f0d6bd292d0768d603b6120aae60437383a51e3` | `9f0d6bd292d0768d603b6120aae60437383a51e3` | MATCH |
| candidate tree | `1fac8e7f7b43bf8201db9c36f7fd007c66897b93` | `1fac8e7f7b43bf8201db9c36f7fd007c66897b93` | MATCH |
| candidate parent / canonical-docs commit | `3baa2afbcfe2adaa6c4604d3ed632ba6a363ea73` | `3baa2afbcfe2adaa6c4604d3ed632ba6a363ea73` | MATCH |
| canonical-docs tree | packet-declared `b4831e9f81da2be677083395e8fa4b217db2c5bd` | `b4831e9f81da2be677083395e8fa4b217db2c5bd` | MATCH |
| task packet | `docs/development/ai/task-packets/cfr168-review-bottleneck-operating-controls-freeze-20260731.md` | same | MATCH |
| task-packet SHA-256 | `b5b74a6be1f6abf39eaf2cff4cdd3449b98a056687438cc07cab93e09fd8970c` | `b5b74a6be1f6abf39eaf2cff4cdd3449b98a056687438cc07cab93e09fd8970c` | MATCH |
| composed review range | `845f660757b9bc972eab0ddb5101a398b0e6343f..9f0d6bd292d0768d603b6120aae60437383a51e3` | 8 changed paths | MATCH |
| composed binary-diff SHA-256 | derived | `b1f39f5e77cc5ec8cc55e97e87ada418c36a7553ab7953a758bb26e43e13800a` | RECORDED |
| packet-alignment binary-diff SHA-256 | derived for `3baa2af..9f0d6bd` | `fc1bd9a7e79546422a2e843ee4f21ee7c6b769ecefc60cb069ed59bdfbd7be3d` | RECORDED |

The final commit changes only the packet. The composed range changes these eight
paths:

1. `docs/development/ai/README.md`
2. `docs/development/ai/development-flow-kgi-kpi.md`
3. `docs/development/ai/operating-protocol.md`
4. `docs/development/ai/review-recurrence/README.md`
5. `docs/development/ai/task-packets/cfr168-review-bottleneck-operating-controls-freeze-20260731.md`
6. `docs/development/ai/validation-report-template.md`
7. `docs/development/codefire-improvement-cycle.md`
8. `docs/development/project-management.md`

## 2. Compatibility result

The candidate successfully establishes a one-way authority map, separates
source acceptance from final Close in the canonical lifecycle documents,
separates immutable reviewer proposals from manager adjudication, makes
actionability precede the causal triad, preserves repaired
`required_correction` history, and describes the current v1/v2 limitations
honestly. In particular:

- `review-packet.v1` is accurately described as unable to prove the new impact,
  actionability, repair-authority, or reviewer-provenance semantics;
- recurrence v1 remains compatibility-only and the normal checker has a
  dedicated structured `BLOCK` path;
- recurrence v2 remains the current experimental candidate, has no committed
  live-arm anchor, and is not claimed to encode general actionability;
- the P1 review-process and P2 v2.1 schemas/checker are explicitly future work
  and are absent at this candidate; and
- `docs/development/history.md` is byte-identical to the pre-governance base
  (Git blob `964085ed12ca3c55c34c8f99bbac75c52e3428a2`), so accepted historical
  evidence was not rewritten.

Five evidence-bound compatibility defects remain. They are not a finding quota
and none is a style preference.

## 3. Observations

### GCC-001 — live lifecycle instructions still authorize the superseded order

Evidence:

- `docs/development/project-management.md:43-48` says that document alone owns
  lifecycle order, and `:115-146` fixes source acceptance at Integrate,
  Install/Dogfood/Evaluate before final Close, and Close-time synchronization.
- `docs/development/current-cycle-burn-down-plan.md:3-16` describes itself as
  the current plan and claims to use the same order as project management, but
  still lists `Implement -> Close -> Install -> Dogfood -> Retrospect` and omits
  Validate, Candidate Freeze, review/adjudication, repair/rereview, Integrate,
  source acceptance, and Evaluate.
- This is not a historical-row exception: `project-management.md:38` assigns the
  file to the active batch, and `current-cycle-burn-down-plan.md:40` says its
  historical sections were moved to the archive.
- `docs/development/ai/README.md:99-108` likewise completes final Close and
  Retrospect at step 10, then runs the AI gate, records required events, and
  updates canonical docs/history at steps 11-13. That order conflicts with the
  Close exit evidence at `project-management.md:130`.

Compact impact screen:

- Frozen outcomes: affected; the sole canonical lifecycle and final-Close
  evidence contract are contradicted by two live operator entry points.
- Affected entities: users=developers/managers; workflows=current burn-down and
  AI fast path; surfaces=the two live instructions above; owners=development
  manager.
- Categories: correctness=affected; state=affected (premature terminal issue
  status); data=affected (closure evidence/history ordering);
  authority=affected; api_compatibility=unaffected; operations=affected;
  performance=unaffected; resource=unaffected; maintainability=affected;
  development_flow=affected.
- Reachability/likelihood: directly reachable and likely because both files are
  current operating entry points.
- Blast radius: cross-workflow and batch-wide; no product-runtime byte is
  directly changed.
- Severity/materiality: high/material for governance integrity.
- Reversibility/recovery: documentation-only and reversible, but any issue
  already closed under the wrong order requires an evidence audit rather than a
  prose-only relabel.
- Recurrence: observed in both the current-cycle policy and AI fast path.
- Evidence quality: available, deterministic line-level contradiction; no
  acceptance-critical unknown.
- Repair tradeoffs/hazards: preserve completed historical rows; do not rewrite
  old evidence to make it look as though the new order was used historically.

Actionability proposal:

- Disposition: `required_correction`.
- Must-fix basis: `MF-1` proved against the frozen lifecycle and Close exit
  contract.
- Causal-class proposal: `implementation_nonconformance`; manager adjudication
  remains authoritative.
- Smallest repair: replace the current-cycle top-level Policy with a reference
  to, or faithful summary of, the formal lifecycle; make AI event capture
  continuous and put required checks plus issue/map/docs/history synchronization
  inside the pre-Close/Close evidence path, before Retrospect. Keep explicitly
  historical completion rows immutable or label them historical.
- Rereview: exact changed paragraphs plus all live lifecycle links and final
  Close ordering.

### GCC-002 — current-state authority disagrees on whether L1.5 is complete

Evidence:

- `docs/development/ai/README.md:60-64` says later Cargo/KGI authority review
  reopened L1.5, that hardening is in progress, and no hardening implementation
  candidate is integrated on the canonical branch.
- The current management hub instead says L1.1-L1.5 are complete and L1.6 is
  next (`docs/development/README.md:7-21`, especially `:18`). It also says the
  hub wins when live phase claims disagree (`:38`).
- The current-cycle snapshot and control table repeat the old state:
  `docs/development/current-cycle-burn-down-plan.md:28-41` and `:76-82` say
  L1.1-L1.5 are complete, integration needs no further implementation
  authority, and L1.6 is next.
- The hub and current-cycle blobs are unchanged across the composed candidate
  (`e257b20999f791a056bf4e6a64e71fa3a61753f0` and
  `115a3f11ec4cd77c7c257e21cc8325b9d4d0cba2`, respectively). They are live
  state, not immutable history. The older history row may correctly remain an
  immutable record of the earlier experimental integration.
- `project-management.md:240-246` requires the management hub to be updated
  first when current documents disagree; `docs/development/README.md:21`
  requires a history append when its snapshot changes.

Compact impact screen:

- Frozen outcomes: affected; one exact next-authority decision cannot be
  derived from the live documents.
- Affected entities: users=manager/implementers/reviewers; workflows=AIA-047
  dispatch, live-arm creation, and L1.6 admission; surfaces=management hub,
  active cycle, AI checkpoint; owner=development manager.
- Categories: correctness=affected; state=affected; data=unaffected for product
  data; authority=affected; api_compatibility=unaffected; operations=affected;
  performance=unaffected; resource=potentially affected by launching the wrong
  work; maintainability=affected; development_flow=affected.
- Reachability/likelihood: directly reachable; the documented intake order reads
  the hub before AI docs, so operators are told the stale state first.
- Blast radius: control-plane-wide and cross-workflow.
- Severity/materiality: high/material because the stale state can skip required
  hardening and attempt L1.6 against the wrong authority chain.
- Reversibility/recovery: docs-only before dispatch; after dispatch, the manager
  must audit any work authorized from the stale snapshot.
- Recurrence: current state is duplicated in at least three live loci.
- Evidence quality: available and exact; no critical unknown about the textual
  contradiction.
- Repair tradeoffs/hazards: append new history instead of altering the accepted
  `00ad885` historical entry; serialize edits to live global state files.

Actionability proposal:

- Disposition: `required_correction`.
- Must-fix basis: `MF-1` (source-of-truth/current-state rules) and `MF-5`
  (required state evidence is contradicted).
- Causal-class proposal: `implementation_nonconformance`; manager adjudication
  remains authoritative.
- Smallest repair: make one manager decision on the L1.5 state, update the
  management hub first, then the active-cycle checkpoint/table and AI README to
  that same decision. If the later audit is accepted, show L1.5 hardening as the
  next gate and block L1.6; append a new dated history row explaining the reopen
  without modifying the old integration row.
- Rereview: exact live-state paragraphs, next-authority ordering, and history
  append topology.

### GCC-003 — duplicate observations have contradictory denominator semantics

Evidence:

- Canonical policy retains a duplicate in observation-level workload/churn and
  `MET-REVIEW-011`, while preventing a second unique effect/repair/closure unit
  (`docs/development/ai/development-flow-kgi-kpi.md:125-129` and
  `operating-protocol.md:324-337`).
- The packet initially agrees: its disposition row retains duplicates for
  workload, churn, and actionability distribution (`task-packets/...freeze-20260731.md:368`).
- Its proposed P2 checker later says a duplicate “does not add a finding,
  episode, repair, or metric denominator unit” (`:895-896`), which is a global
  exclusion and would remove the required `MET-REVIEW-011` observation unit.
- `P1-T17` repeats the over-broad rule by making any new denominator unit for a
  duplicate a failure (`:1134`), while the same packet defines
  `MET-REVIEW-011` over all observations at `:734`.

Compact impact screen:

- Frozen outcomes: affected; canonical numerator/denominator and raw-workload
  rules cannot both be implemented from the packet.
- Affected entities: users=process evaluators/managers; workflows=P1 derivation,
  P2 checker, churn evaluation; surfaces=duplicate observations and metric
  results; owners=P1/P2 implementers and manager.
- Categories: correctness=affected; state=unaffected; data=affected (metric
  subjects/denominators); authority=affected (checker decision contract);
  api_compatibility=affected for the proposed schema/checker contract;
  operations=affected; performance=unaffected; resource=unaffected;
  maintainability=affected; development_flow=affected.
- Reachability/likelihood: certain if lines 895-896 or P1-T17 are implemented
  literally.
- Blast radius: every cohort containing duplicate review observations.
- Severity/materiality: moderate/material; it systematically under-reports
  review work and preference churn.
- Reversibility/recovery: schema/checker prose repair is cheap before P1/P2;
  post-implementation repair may require schema/result migration.
- Recurrence: two contradictory packet loci make reintroduction likely.
- Evidence quality: available and deterministic; no acceptance-critical unknown.
- Repair tradeoffs/hazards: avoid the opposite error of counting one duplicate
  as a second unique effect, issue, repair, or recurrence-after-closure
  opportunity.

Actionability proposal:

- Disposition: `required_correction`.
- Must-fix basis: `MF-1` against the canonical denominator rule.
- Causal-class proposal: `implementation_nonconformance`; manager adjudication
  remains authoritative.
- Smallest repair: replace the global “metric denominator unit” prohibition with
  per-metric semantics: retain the observation in workload/churn and
  `MET-REVIEW-011`; do not create another unique effect/finding/issue/repair or
  `MET-REVIEW-005` closure-opportunity unit; let every other metric definition
  state its subject. Rewrite P1-T17 to test both prohibited double-counting and
  prohibited observation-level dropping.
- Rereview: P1 denominator fixtures, P2 duplicate joins, and all duplicate
  references in canonical and packet prose.

### GCC-004 — P1 cannot derive `MET-FLOW-014` from its closed raw-event contract

Evidence:

- Canonical `MET-FLOW-014` is post-rejection evidence-producing repair/retest
  seconds divided by total evidence-producing seconds, and is unavailable unless
  both paired interval sets are complete
  (`development-flow-kgi-kpi.md:69`). Same-clock/boot pairing and raw-subject
  rules are mandatory at `:185-202`.
- P1 promises closed run/cohort/result schemas with machine-derived intervals,
  numerators, denominators, and unavailable counts and rejects authored KPI
  values (`task-packets/...freeze-20260731.md:551-580`).
- The closed event vocabulary at `:598-634` contains
  `repair_active_start/end`, but contains no general evidence-producing-active
  interval and no retest-active interval. Thus it can at most derive part of the
  numerator and cannot derive the total evidence-producing-seconds denominator.
- Nevertheless, P1 claims to preserve and derive `MET-FLOW-014` at `:718-745`.
  No aggregation rule says whether concurrent lane intervals are summed as
  lane-seconds or unioned as elapsed time.

Compact impact screen:

- Frozen outcomes: affected; the promised raw-first, checker-derived rework KPI
  has no complete source model.
- Affected entities: users=cohort evaluators/managers; workflows=P1 capture and
  KGI/KPI decision; surfaces=run schema, result schema, checker, cohort result;
  owners=P1 implementer/manager.
- Categories: correctness=affected; state=unaffected; data=affected (missing
  numerator/denominator subjects); authority=affected if a checker emits a value;
  api_compatibility=affected for the proposed closed schema; operations=affected;
  performance=affected because the metric is meant to measure rework service;
  resource=unaffected; maintainability=affected; development_flow=affected.
- Reachability/likelihood: certain for any P1 run that tries to emit a measured
  `MET-FLOW-014`; otherwise the metric remains permanently unavailable despite
  the packet's derivation claim.
- Blast radius: all P1 cohorts and the bottleneck outcome evaluation.
- Severity/materiality: high/material for the stated review-bottleneck KGI/KPI
  objective, without a product-runtime effect.
- Reversibility/recovery: cheap before schema Freeze; costly after records are
  accepted because closed-schema migration would be required.
- Recurrence: likely unless interval identity, classification, clocks, overlap,
  and cohort aggregation are all frozen together.
- Evidence quality: available for the missing vocabulary; no unknown is needed
  to establish the contract gap.
- Repair tradeoffs/hazards: do not infer active service from elapsed phase spans,
  UTC/Git time, or overlapping wait intervals.

Actionability proposal:

- Disposition: `required_correction`.
- Must-fix basis: `MF-1` and `MF-5` against the frozen raw-first KPI contract.
- Causal-class proposal: `implementation_nonconformance`; manager adjudication
  remains authoritative.
- Smallest repair: before P1 implementation, either (a) add stable paired
  same-boot evidence-active intervals covering first-attempt, repair, retest,
  validation, review, and other denominator work, with an explicit lane-seconds
  versus elapsed-union rule and unavailable handling, or (b) remove the claim
  that P1 derives `MET-FLOW-014` and keep it unavailable until such a source is
  separately frozen. Add missing/cross-clock/overlap/concurrent-lane mutations.
- Rereview: exact event enum, interval declarations, run/result derivation, and
  `MET-FLOW-014` raw subject/numerator/denominator fixtures.

### GCC-005 — the packet re-expands every observation after freezing two-depth review

Evidence:

- The packet's risk-proportional rule says every observation gets the complete
  compact screen and only required-correction, accepted-variance,
  disputed/reopened, high/material, or acceptance-critical-unknown observations
  get the expanded row (`task-packets/...freeze-20260731.md:300-311`).
- The proposed v2.1 contract repeats the same conditional expansion at
  `:824-845`.
- The intervening mandatory-row list says every observation/adjudication row
  records “the full impact matrix above” (`:409-420`). In this section, “full”
  names the expanded matrix introduced at `:313`; an implementer cannot satisfy
  that sentence and the compact-terminal rule simultaneously.
- The canonical template is unambiguous: compact always, expanded conditionally
  (`docs/development/ai/validation-report-template.md:132-190`).

Compact impact screen:

- Frozen outcomes: affected; the packet's stated burden-control and closed P2
  shape have two incompatible implementation requirements.
- Affected entities: users=reviewers/managers/implementers; workflows=P0 report
  authoring and P2 schema implementation; surfaces=every observation record;
  owners=packet/P0/P2 owners.
- Categories: correctness=affected (contract ambiguity); state=unaffected;
  data=affected (record shape); authority=affected; api_compatibility=affected
  for the proposed schema; operations=affected; performance=affected (review
  service time); resource=affected (unnecessary authoring/validation load);
  maintainability=affected; development_flow=affected.
- Reachability/likelihood: certain at implementation unless one clause is chosen
  over the other without authority.
- Blast radius: all observations, especially compact-terminal invalid,
  out-of-scope, nonblocking, and duplicate rows.
- Severity/materiality: moderate/material for the stated bottleneck-control
  objective.
- Reversibility/recovery: one prose correction before implementation; a later
  correction may force schema/data migration.
- Recurrence: likely because the obsolete universal-full sentence sits in the
  normative “Every row records” list.
- Evidence quality: available and line-exact; no acceptance-critical unknown.
- Repair tradeoffs/hazards: retain every compact axis and explicit unknown; do
  not reduce burden by permitting omitted axes.

Actionability proposal:

- Disposition: `required_correction`.
- Must-fix basis: `MF-1` against the frozen two-depth packet and canonical
  template contract.
- Causal-class proposal: `implementation_nonconformance`; manager adjudication
  remains authoritative.
- Smallest repair: change `:415` to require the complete compact screen for every
  observation and the expanded matrix exactly when `full_record_required` is
  true, with the existing canonical-reference exception for a true duplicate.
  Align nearby “complete impact row” wording to the same two-depth terms.
- Rereview: all normative P0/P2 row-shape clauses and compact-terminal
  counterexamples.

## 4. Unresolved must-fix and critical unknowns

| ID | State | Blocking reason |
|---|---|---|
| `GCC-001` | unresolved must-fix | live lifecycle order contradicts the sole canonical owner |
| `GCC-002` | unresolved must-fix | live current-state authorities disagree on L1.5 and next work |
| `GCC-003` | unresolved must-fix | duplicate denominator behavior is self-contradictory |
| `GCC-004` | unresolved must-fix | `MET-FLOW-014` has no complete raw derivation source |
| `GCC-005` | unresolved must-fix | compact/expanded impact depth is internally contradictory |

Critical unknowns: none. The five defects are established from current bytes;
they do not need speculative evidence to remain blocking.

## 5. Validation and migration checks

| Check | Result |
|---|---|
| `sha256sum docs/development/ai/task-packets/cfr168-review-bottleneck-operating-controls-freeze-20260731.md` | PASS; exact requested digest |
| exact commit/tree/parent/canonical-tree recomputation | PASS |
| `git diff --check 845f660757b9bc972eab0ddb5101a398b0e6343f 9f0d6bd292d0768d603b6120aae60437383a51e3` | PASS; no output |
| `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py` | PASS; 673 Markdown files inventoried |
| `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_acceleration_docs.py` | PASS |
| `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_contracts.py` | PASS |
| `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_development.py` | PASS; acceleration docs, contracts, project docs, source docs, runtime, tool tests, and capability probe all passed |
| future P1/P2 schema/checker path presence check | PASS for honesty; all packet-declared future paths inspected are absent |
| historical `history.md` blob comparison, base versus candidate | PASS; byte-identical |
| report-only post-check: `git diff --cached --check`, project docs, AI acceleration docs, and AI contracts | PASS; one staged report path and 674 Markdown files inventoried |

The green checks establish syntax, links, registered current schemas/tools, and
existing tests. They do not resolve the five semantic compatibility defects
above, which are outside the current machine contracts the candidate itself
describes as incomplete.

## 6. Decision

The candidate is not ready for manager acceptance or implementation dispatch.
A new exact candidate must close `GCC-001` through `GCC-005`, preserve existing
historical bytes, append rather than rewrite new history, and receive a fresh
report-only compatibility review against its new commit/tree/packet digest.

FINAL_VERDICT: CHANGES_REQUESTED
