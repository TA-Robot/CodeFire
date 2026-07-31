# AI Validation Report Template

This is a handoff shape, not passing evidence. Replace every placeholder; an omitted, skipped, or capability-blocked command must not be reported as passed.

```text
Task packet ID:
Base revision:
Head revision:
Changed-file digest:
Environment/capability artifact:

Validation:
- validation_id:
  exact_command:
  status: passed | failed | blocked
  exit_code:
  duration_ms:
  log_sha256:
  result_summary:
  capability_evidence_path:

Claims verified:
- claim:
  evidence_refs:

Known gaps:
-

Recommended decision: approve | request_changes | block
```

For approval, serialize the equivalent machine-readable record with `schemas/review-packet.v1.schema.json`. The reviewer must bind the exact task digest, committed base/head, canonical patch digest, changed-file set, and every required validation ID.

## Compact Exact Independent Review

Use this section for a report-only exact-candidate review. Replace every
placeholder and delete instructions. Prefer evidence references and compact
tables over repeating source narration. A required row may be `OPEN`,
`CLOSED`, or `ROUTED_OPEN`; routing is never closure.

```text
# <Exact review title>

Reviewed candidate: <full SHA>
Report mode: report-only

## 1. Immutable subject

| Field | Declared | Recomputed | Result |
|---|---|---|---|
| task/review base | <SHA> | <SHA> | MATCH / MISMATCH |
| candidate parent | <SHA> | <SHA> | MATCH / MISMATCH |
| candidate head | <SHA> | <SHA> | MATCH / MISMATCH |
| tree | <SHA> | <SHA> | MATCH / MISMATCH |
| merge base | <SHA> | <SHA> | MATCH / MISMATCH |
| additive commits / merges | <n> / 0 | <n> / <n> | MATCH / MISMATCH |
| binary patch SHA-256 | <digest> | <digest> | MATCH / MISMATCH |
| changed paths | <count/ref> | <count/ref> | MATCH / MISMATCH |
| clean tracked/untracked | true | true/false | MATCH / MISMATCH |

## 2. Scope and production reachability

- live entry point(s):
- reviewed owners/effects/fences:
- exact writable report path:
- out-of-scope surfaces:
- scope deviations: none / <list>

## 3. Required finding dispositions

| ID | Prior claim | Disposition | Production counterexample or proof | Deliberate mutation | Evidence |
|---|---|---|---|---|---|
| <ID> | <one sentence> | CLOSED / OPEN / ROUTED_OPEN | <live route and observed result> | <mutation and targeted failure> | <log/test/symbol> |

Every `CLOSED` row names a reachable owner, effect, fence, terminal snapshot,
and mutation that the exact production oracle kills. Every `OPEN` row names
impact and the smallest repair boundary. Every `ROUTED_OPEN` row names a
committed successor contract and stays excluded from local/global closure.

## 4. Owner/effect/fence summary

| Phase | Physical owner | Charge/reservation | First effect | Final fence | Terminal outcome |
|---|---|---|---|---|---|
| <phase> | <owner> | <dimension/site> | <effect or none> | <authority/deadline> | <status/bytes/debt/permits> |

## 5. Mutation matrix

| Mutation | Exact production edit | Focused oracle | Expected failure | Observed | Reverted/identity restored |
|---|---|---|---|---|---|
| <ID> | <symbol/change> | <test/command> | <reason> | KILLED / SURVIVED | yes/no |

## 6. Validation

| Class | Exact command or cache key | Tree/env/toolchain | Result/count | Duration | Log SHA-256 |
|---|---|---|---|---:|---|
| non-cacheable focused | <argv> | <identity> | <result> | <seconds> | <digest> |
| reusable broad | <argv or key> | <identity> | <result> | <seconds/avoided> | <digest> |

Failed, flaky, interrupted, truncated, different-tree, different-argv,
different-environment, or mutable-name-only evidence is never `PASS` or a
cache hit. Prefer `docs/development/tools/run_timed_gate.py`; report its
monotonic duration and wall-clock-discontinuity field rather than shell wall
time.

## 7. New findings and integration overlap

| Severity | Owner | Finding | Reproducer/evidence | Required successor |
|---|---|---|---|---|
| <severity> | <owner> | <finding or none> | <evidence> | <packet or none> |

## 8. Unavailable evidence

- <item and why unavailable>, or `none`

## 9. Decision

- local required rows closed: <n>/<n>
- routed-open contracts valid: <n>/<n>
- mutations killed: <n>/<n>
- required independent gates: <n>/<n>
- unowned material findings: <n>
- integration authorization: no

OVERALL_VERDICT: ACCEPT
```

The last line appears exactly once and must be either
`OVERALL_VERDICT: ACCEPT` or `OVERALL_VERDICT: REQUEST_CHANGES`.
`ACCEPT_WITH_NOTES` is not permitted. Discussion may describe acceptance
criteria, but must not emit a second `OVERALL_VERDICT` line.

## Finding Impact and Actionability Record

Use the compact screen once for every canonical review observation before any
repair. Escalate to the expanded record only for a proposed
`required_correction`, accepted variance, disputed/reopened locus, high or
material impact, or acceptance-critical unknown. A duplicate may reference its
canonical expanded record. A clearly evidenced invalid, out-of-scope, or
nonblocking observation may terminate after the compact screen and manager
disposition. This two-depth shape reduces review service time without omitting
an axis or treating unknown as neutral.

When combined with the compact exact review above, place the reviewer blocks
before its Decision section and single `OVERALL_VERDICT`. The manager block
below is a later, separate artifact; it is not appended to the reviewer report.

Severity alone is never a repair instruction. If a review produces no
observation, write `none` and still provide the immutable subject,
changed-path/obligation coverage, hazards, and evidence that justify stopping;
an empty report is not coverage. Definitions and authority live in
`operating-protocol.md`; this template mirrors them and does not replace policy
or machine schemas. Future typed tooling may prefill Git-derived identity,
coverage, common frozen facts, and unchanged axes, but the reviewer/manager
remain responsible for their semantic truth.

```text
### Observation <observation_id>

Identity and evidence:
- finding_id: <stable ID>
- effect_episode_id: <stable ID or unavailable + reason>
- decision_locus_id: <stable path/symbol/conceptual decision locus>
- exact candidate commit/tree: <SHA> / <SHA>
- changed path/symbol or conceptual scope: <value>
- frozen requirement/invariant/acceptance refs: <exact refs or none + evidence>
- observed fact: <falsifiable statement>
- production route/precondition: <route or unreachable/unknown + evidence>
- reproducer/counterexample: <ref, not_attempted, unavailable, or not_applicable + reason>
- evidence refs: <immutable refs>

Compact impact screen (required for every observation):
- frozen outcomes: <affected/unaffected/unknown + exact refs/evidence>
- affected entities: users=<affected/unaffected/unknown>; workflows=<...>; surfaces=<...>; owners=<...> — <refs/reason>
- impact categories: correctness=<affected/unaffected/unknown>; state=<...>; data=<...>; authority=<...>; api_compatibility=<...>; operations=<...>; performance=<...>; resource=<...>; maintainability=<...>; development_flow=<...> — <refs/reasons>
- reachability/likelihood: <affected/unaffected/unknown + route/basis>
- blast radius: <affected/unaffected/unknown + boundaries>
- severity/materiality: <affected/unaffected/unknown + preliminary value/criterion>
- reversibility/recovery: <affected/unaffected/unknown + owner/steps>
- recurrence: <affected/unaffected/unknown + prior IDs>
- evidence quality: <available/partial/unavailable + reproducibility/limitations>
- repair tradeoffs/hazards: <affected/unaffected/unknown + refs; cannot offset must-fix>
- must_fix_candidates: <MF IDs/none/unknown + evidence>
- acceptance_critical_unknown: <yes/no + evidence>
- proposed disposition: <six values or null_pending_impact + evidence>
- full_record_required: <yes/no + triggering condition or compact-terminal rationale>
- canonical full-record ref: <required for duplicate when applicable, otherwise null>

If `full_record_required` is `no`, delete the expanded block below and leave the
manager the compact proposal, canonical/referral ID, and evidence. Otherwise
complete every expanded row:

Expanded impact gate (conditional):
| Axis | Assessment | Rationale / criterion | Evidence |
|---|---|---|---|
| frozen outcomes | <violated/satisfied/unaffected/unknown for each requirement, invariant, KGI/KPI, quality guardrail> | <why each is or is not binding> | <refs> |
| affected entities | users=<set/none/unknown>; workflows=<set/none/unknown>; surfaces=<set/none/unknown>; owners=<set/none/unknown> | <explicit empty/unknown reason> | <refs> |
| correctness | adverse/beneficial/neutral/not_applicable/unknown: <effect> | <criterion> | <refs> |
| state | adverse/beneficial/neutral/not_applicable/unknown: <effect> | <criterion> | <refs> |
| data | adverse/beneficial/neutral/not_applicable/unknown: <effect> | <criterion> | <refs> |
| authority | adverse/beneficial/neutral/not_applicable/unknown: <effect> | <criterion> | <refs> |
| api_compatibility | adverse/beneficial/neutral/not_applicable/unknown: <effect> | <criterion> | <refs> |
| operations | adverse/beneficial/neutral/not_applicable/unknown: <effect> | <criterion> | <refs> |
| performance | adverse/beneficial/neutral/not_applicable/unknown: <effect> | <criterion> | <refs> |
| resource | adverse/beneficial/neutral/not_applicable/unknown: <effect> | <criterion> | <refs> |
| maintainability | adverse/beneficial/neutral/not_applicable/unknown: <effect> | <frozen/pre-measured criterion or preference boundary> | <refs> |
| development_flow | adverse/beneficial/neutral/not_applicable/unknown: <effect> | <criterion> | <refs> |
| reachability / likelihood | <observed/reachable/conditional/unreachable/unknown> / <observed/reproducible/probable/possible/unlikely/unknown> | <route and basis> | <refs> |
| blast radius | <local/cross_module/cross_workflow/persisted/public/cross_generation/system_wide/none/unknown> | <boundaries> | <refs> |
| severity / materiality | <critical/high/moderate/low/none/unknown> / <material/nonmaterial/unknown> | <criterion, not a score> | <refs> |
| reversibility / recovery | <reversible_in_place/rollback_required/data_or_state_recovery_required/irreversible/unknown> | <steps, owner, cost evidence> | <refs> |
| recurrence likelihood | <observed_recurrence/likely/possible/unlikely/none_proven/unknown> | <prior effect/decision IDs> | <refs> |
| evidence quality | confidence=<high/medium/low/unknown>; reproducibility=<reproduced/observed_once/not_reproduced/not_attempted/unavailable>; availability=<available/partial/unavailable> | <limitations> | <refs> |
| repair tradeoffs | cost=<basis/unavailable>; hazards=<IDs>; alternatives=<list>; workaround=<limits> | <may select scope/order/mechanism, never offset must-fix> | <refs> |

Reviewer must-fix and actionability proposal:
- MF-1 frozen requirement/invariant/guardrail violation: proved/rejected/unknown — <evidence>
- MF-2 reachable material correctness/state/data/authority failure: proved/rejected/unknown — <evidence>
- MF-3 public/persisted/cross-generation compatibility break: proved/rejected/unknown — <evidence>
- MF-4 frozen operations/performance/resource/availability/recovery breach: proved/rejected/unknown — <evidence>
- MF-5 required acceptance evidence absent/contradicted/invalid: proved/rejected/unknown — <evidence>
- MF-6 repair-introduced regression or reopened proven effect: proved/rejected/unknown — <evidence>
- proposed impact_gate: complete | blocked_pending_impact_investigation | routed_noncritical_investigation
- proposed impact investigation objective: <when any relevant axis is unknown/unavailable>
- manager impact adjudication required: yes/no — <high/material boundary, dispute, variance reversal, or reason>
- reviewer proposal: required_correction | accepted_implementation_variance | nonblocking_improvement | out_of_scope | duplicate | invalid_or_unsubstantiated | null_pending_impact
- canonical duplicate/referral: <ID or null>
- proposed smallest repair/rereview boundary: <proposal only, or null>
- reviewer actor/event/evidence/rationale: <identity and refs>
```

Commit the reviewer report now. Do not append manager decisions to that
report-only commit. `OVERALL_VERDICT` is the reviewer's terminal proposal; it is
not integration authority.

After the report commit and reviewer/children stop, the manager writes a
separate adjudication/repair-authority artifact. It references immutable
reviewer bytes rather than editing them:

```text
### Manager adjudication <decision_id>

Subject and chronology:
- reviewer report path / commit / SHA-256: <exact values>
- observation_id / finding_id / decision_locus_id: <values>
- manager actor / event / timestamp / evidence: <distinct identity and refs>

Final impact and actionability:
- final or changed impact axes: <reviewer rows accepted by ref, plus every changed axis/evidence/rationale>
- impact_gate: complete | blocked_pending_impact_investigation | routed_noncritical_investigation
- impact investigation owner/ref/result: <required when relevant>
- final MF-1..MF-6 decisions: <proved/rejected/unknown + evidence>
- final actionability_disposition: required_correction | accepted_implementation_variance | nonblocking_improvement | out_of_scope | duplicate | invalid_or_unsubstantiated | null_pending_impact
- correction_required: true | false | undecided
- canonical duplicate/referral: <ID or null>

Required-correction causal class:
- required_correction_causal_class: contract_omission | implementation_nonconformance | repair_introduced_regression | null
- governance-origin precedence: <no pre-candidate rule / effect created by repair delta / existing rule and non-repair-origin violation>
- frozen-clause or additive-amendment ref: <required only for correction>
- causal decision evidence: <required only for correction>

Repair and rereview authority:
- authorized correction IDs: <required-correction IDs or null>
- allowed paths/symbols: <set or null>
- forbidden paths/surfaces: <set or null>
- required positive validations: <IDs/commands or equivalence evidence>
- required negative controls/hazards: <IDs/evidence>
- exact rereview scope: <delta + inherited obligations/effects + affected surfaces, or null>
- scope expansion/re-Freeze trigger: <new effect/contract/public/state/persisted surface or null>
- repair episode / successor candidate closure: <refs and open/closed/unavailable; never rewrite the original disposition>

Stable decision locus:
- active decision_id / subject scope / effective candidate: <values>
- decision: <disposition and supported equivalence/correction basis>
- permitted reopen triggers: <subset of frozen_requirement_changed, materially_new_evidence, new_counterexample, prefrozen_metric_regression>
- reopen evidence/ref: <exact evidence or null; materially new means new to the active decision record, with proved prior omission/misbinding if the artifact predates it>
- supersedes_decision_ids: <IDs or empty>
- reopen result: not_requested | rejected_preference_churn | admitted_for_manager_adjudication
```

Finish the manager artifact with an index proving that every observation occurs
exactly once and that no acceptance-critical impact remains undecided:

| Observation | Impact gate | Proved MF IDs | Actionability | Causal class or null | Active decision | Repair/rereview ref or null | Reopen trigger/result |
|---|---|---|---|---|---|---|---|
| <ID> | <state> | <IDs/none> | <disposition/null> | <class/null> | <ID> | <ref/null> | <trigger/result> |

For a non-correction disposition, causal class and repair authority are null.
For `required_correction`, all three are present before mutation. Observation
`out_of_scope` must never be used as the recurrence gate's forbidden
changed-path `out_of_scope` escape: every Git-changed path still needs an
in-scope reviewed surface and obligation/effect coverage.

Use the protocol's disposition precedence. In particular, a duplicate delegates
to the canonical row and cannot erase its correction/closure state; a proved
must-fix trigger outranks variance/nonblocking labels. Preserve an original
`required_correction` after repair and close it through the separate successor
candidate/repair evidence instead of changing history.

## Successor Review Recurrence Addendum

For review generation 2 or later, use this addendum with the compact exact
review and produce a final
`schemas/review-recurrence-control.v2.schema.json` record. v2 is the only
current authorization candidate. A v1 record is compatibility/historical input
and the normal checker always returns `BLOCK`; do not rewrite accepted packets
or historical review documents to change their schema. This addendum is not a
request for a fixed number of findings. Do not infer thoroughness from report
length or fill a quota.

### Before the repair

The manager, not the implementer, freezes one closure contract for every prior
obligation in the Markdown repair packet and commits it together with a
`review-recurrence-repair-manifest.v1`. The manifest freezes exact obligation,
parity/closure-claim, and required-validation sets; the final v2 record must be
set-equal to them. Git topology must establish that this packet/manifest commit
preceded the repair:

| Obligation ID | Exact acceptance criterion | Authorized boundary | Required live routes | Anticipated regressions | Forbidden shortcuts |
|---|---|---|---|---|---|
| <stable ID> | <externally observable result> | <smallest allowed symbols/paths> | <normal/error/unwind/composition routes> | <specific ways this repair can regress> | <test-only, claim-only, weakened oracle, new API, etc.> |

This table is a communication boundary. If the implementation needs a new
public API, state machine, classifier, persisted shape, or other unplanned
surface, stop for a manager scope decision; do not silently redefine the
obligation. The machine can prove equality to the frozen collections, but a
manager and independent preflight must judge whether those collections are
semantically complete.

Before any live decision, identify a separate clean committed validator source.
The checker and all v2 schemas/modules must match regular blobs at its `HEAD`.
For a manager-fixed Cargo or KGI/KPI chain, the validator source must also hold
the independently committed live-arm anchor. No such anchor is currently
committed, so current live-arm records necessarily remain `BLOCK` until that
separate manager task is completed.

### Independent successor review

Review every prior obligation first, then every exact changed path and its
candidate-created surfaces as a distinct pass. Use the schema's exact
evidence-state and next-action vocabulary; routing controls successor work and
does not authorize integration:

| Prior obligation | Origin generation | Evidence state | Next action | Before/after evidence | Negative control | Independent closure/non-reachability evidence |
|---|---:|---|---|---|---|---|
| <ID> | <n> | DEFECT_REPRODUCED / DEFECT_SOURCE_PROVEN / DEFECT_INJECTION_ONLY / CLOSURE_UNPROVEN / ORACLE_VACUOUS / CLAIM_FALSE / SCOPE_BOUNDARY / REQUIREMENT_UNSATISFIABLE / CLOSED_PROVEN | PRODUCT_REPAIR / RUN_CLOSURE_EVIDENCE / REPAIR_ORACLE / REPAIR_CLAIM / DOCUMENT_NON_REACHABILITY / MANAGER_SCOPE_DECISION / MANAGER_REQUIREMENT_DECISION / NONE | <artifact refs> | killed / survived / not_applicable / unavailable | <artifact ref or null> |

| Exact changed path | Required by | Review focus | Evidence | Disposition |
|---|---|---|---|---|
| <Git-derived path> | <one or more obligation IDs> | <behavior, oracle, claim, boundary> | <artifact refs> | reviewed |

The changed-path rows must equal the canonical Git changed-path set. Every row
must have at least one surface; v2 has no `no_surface_change`, `out_of_scope`,
or unreviewed-path approval escape.

| Exact path | Surface ID | Kind | Scope disposition | Review evidence | Routing decision |
|---|---|---|---|---|---|
| <Git-derived path> | <stable ID> | <diagnostic kind> | reviewed_in_scope / referred_to_manager | <artifact refs> | <routing_scope_decision ref or null> |

`referred_to_manager` requires an earlier bound routing decision and remains
blocking. It is not a silent scope exclusion.

| New finding | Evidence state | Next action | Before/after evidence | Negative control | Routing decision |
|---|---|---|---|---|---|
| <ID> | <v2 state other than CLOSED_PROVEN> | <legal v2 next action> | <artifact refs> | killed / survived / not_applicable / unavailable | <artifact ref or null> |

A new finding cannot become `CLOSED_PROVEN` in the generation that first
records it. `CLOSED_PROVEN` with `NONE` is the only potentially nonblocking
state/action pair, and only when the validator independently derives its
closure proof. All other pairs remain successor obligations.

For every frozen parity or closure claim, record the exact path/locus,
predecessor behavior, candidate behavior, artifact evidence, and disposition.
Git proves path existence, not semantic symbol identity; locus selection and
behavioral interpretation remain reviewer judgements.

### Count-free stopping rule

Mark each condition independently. `ACCEPT` is invalid unless all are true:

- the v2 authority chain roots at the manager-fixed arm and every retained v2
  generation has passed current-gate semantic replay, including rebind of its
  live-arm pin to the exact anchor blob at its historical deciding-gate commit;
- every prior obligation has a disposition;
- every exact changed path has one or more reviewed surfaces;
- no surface is referred to manager;
- every frozen derivation is complete and evidence-bound;
- applicable negative controls were killed;
- unavailable evidence is explicit;
- all manifest-required validations are passing, terminal, exit-code-bound,
  and content-addressed `codefire.timed_gate.v2` evidence;
- each validation retains the same `HEAD` revision/tree after execution and has
  no index flag before or after the command;
- each timed log and result was written outside the source worktree and then
  copied byte-identically into the evidence commit;
- the report-only commit exists and the reviewer and all child processes have
  stopped;
- the deciding gate comes from a separate clean committed validator root, and
  the explicit live arm matches a committed validator-owned anchor.

Generate each required timed result from the clean candidate without placing
the live log or result inside that source worktree:

```bash
python3 docs/development/tools/run_timed_gate.py \
  --emit v2 \
  --cwd <clean-candidate-worktree> \
  --log <outside-source>/validation.log \
  --result <outside-source>/validation.json \
  --timeout <seconds> \
  -- <exact frozen argv>
```

After terminalization, the manager writes the JSON record and runs:

```bash
python3 docs/development/tools/check_review_recurrence.py \
  --record <final-record.json> \
  --git-root <analysed-candidate-repository> \
  --validator-root <clean-validator-repository> \
  --live-arm cargo \
  --result <outside-both-repositories>/recurrence-result.json
```

Use `--live-arm kgi_kpi` for that arm. v1, a legacy-bootstrap v2 generation,
an absent/mismatched anchor, or `--no-git-verify` can never return integration
`ALLOW`.

The manager records candidate-created findings, surfaces, repair time, review
time, and verdict-to-report time for later comparison. Machine `ALLOW` proves
only the implemented topology, byte, set, and derived-state predicates. Honest
execution, semantic oracle quality, nonzero selector behavior, complete
surface/symbol enumeration, manifest completeness, authorship, and reviewer
independence remain human semantic judgements and must be stated in the review.
