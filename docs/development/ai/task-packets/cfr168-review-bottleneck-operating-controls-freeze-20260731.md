# CFR-168 Review-Bottleneck Operating Controls Freeze

Status: `DRAFT_FOR_INDEPENDENT_PREFLIGHT`

Authority: contract proposal only. This packet does not authorize implementation,
integration, a live recurrence arm, process adoption, or a claim that review
throughput improved.

Owner: CodeFire development manager.

Base revision: `f951f9a4ea0b208f08baef6c931d90284b46e1a6`.

Lifecycle position: Discover complete for the seven process-readiness questions;
Freeze and Plan are proposed here. Implement, Validate, Close, Install, Dogfood,
and Retrospect remain unauthorized.

## 1. Discover: evidence and disposition

The live-freeze hardening packet closes concrete recurrence-gate bypasses, but
does not by itself prove that the review bottleneck is solved. This packet keeps
that distinction explicit. It also does not treat every later
`REQUEST_CHANGES` as an initial-review miss.

The authoritative evidence inspected for this freeze is:

- `docs/development/ai/validation-report-template.md`;
- `docs/development/ai/review-recurrence/README.md`;
- `docs/development/ai/operating-protocol.md`;
- `docs/development/ai/development-flow-kgi-kpi.md`;
- `docs/development/ai/stage5-manual-control-runbook.md`;
- `docs/development/ai/schemas/development-flow-record.v1.schema.json`;
- `docs/development/ai/schemas/review-recurrence-control.v1.schema.json`;
- `docs/development/ai/schemas/review-recurrence-control.v2.schema.json`;
- `docs/development/ai/schemas/review-recurrence-repair-manifest.v1.schema.json`;
- `docs/development/ai/task-packets/cfr168-review-recurrence-v2-live-freeze-hardening-20260731.md`;
- `docs/development/ai/swarm-batches/cfr168-review-recurrence-v2-live-freeze-hardening-adjudication-20260731.md`;
- `docs/development/ai/swarm-batches/cfr168-development-bottleneck-causal-gate-20260731.md`; and
- `docs/development/ai/swarm-results/cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md`.

The accepted hardening packet requires a new closed v2 gate-result schema, but
that file is intentionally absent at this packet's `f951f9a` base. In-progress
implementation bytes were useful for the preliminary audit but are not treated
as accepted authority by this Freeze.

The seven readiness questions are dispositioned as follows. `PRESERVE` means an
existing control is already structurally strong, not that the whole bottleneck
is closed.

| ID | Question | Current evidence | Disposition | Owning phase |
|---|---|---|---|---|
| `RB-01` | requirement x change x risk x counterexample traceability | The Markdown template freezes acceptance criteria, boundaries, routes, anticipated regressions, and forbidden shortcuts. Manifest claims join to obligations and derivations, but the template's changed-path `Required by` column has no corresponding v2 `path_review` field. Risk/hazard joins also remain prose. | `CONTRACT_OMISSION`; require a P0 report matrix, then make the joins typed in P2. | P0 + P2 |
| `RB-02` | pre-repair failure-mode prediction | The current hardening packet gives every F1-F3 rule concrete required counterexamples. Generic packets still rely on manager and independent-preflight semantic completeness. | `P0_HUMAN_BOUNDARY`; preserve the current packet and bind each known hazard to evidence. Add typed hazard joins only in P2. | P0 + P2 |
| `RB-03` | finding ID to repair, validation, and rereview scope | Stable prior-obligation IDs, successor carry, closure validations, and same-generation new-finding closure prohibition exist. There is no direct obligation-to-changed-path join, and v2 no longer retains observation/effect-episode identity. | `CONTRACT_OMISSION`; enforce the complete P0 finding handoff and P2 joins. | P0 + P2 |
| `RB-04` | exact commit/tree fixation | Candidate revision/tree/patch/path identity, timed result identity, report/evidence chronology, and separate validator identity are already fail-closed controls. | `PRESERVE`; any weakening is a blocker. | P0-P2 |
| `RB-05` | KPI capture for repeated findings, first-pass acceptance, review rounds, elapsed/wait/repair time | Policies name some metrics and the manual runbook names the desired events, but the only committed flow record predates this run. Current schemas have no typed wait interval, review-lineage, effect mapping, or repair-interval model. | `CONTRACT_OMISSION`; implement P1 without inventing historical values. | P1 |
| `RB-06` | count-gaming prevention | v2 fixes `finding_target_policy` to `none`, closes its schemas, rejects named quota fields, derives exact set/path coverage, and keeps unavailable evidence visible. Semantic proxy detection remains a human judgement. | `PRESERVE`; add no quota, score, or saturation proxy. | P0-P2 |
| `RB-07` | fresh reviewer independence | Policy requires an independent report-only lane and topology proves a report-only commit, but reviewer independence is explicitly a human boundary. A boolean or Git author string cannot prove it. | `P0_HUMAN_BOUNDARY`; record fresh-session exclusions now and add trusted scheduler-level provenance in P2 without claiming cognitive independence. | P0 + P2 |

These are process-contract findings, not findings on the in-progress hardening
candidate. `RB-01`, `RB-03`, and `RB-05` are candidate classifications of
`contract_omission` relative to the broader review-bottleneck objective. They
must not be relabelled as hardening `implementation_nonconformance` unless an
exact code review proves that the implementation violates its already accepted
packet.

They are blockers within the already frozen CFR-168/AIA-047 control-plane work,
not admission of an unrelated Discovery batch. This draft creates no new issue
status; the manager decides issue mapping when adjudicating the contract.

## Objective

The objective is to make a review/repair lineage answer all of the following
without relying on report length or an agent's assertion:

1. what exact obligation and anticipated hazard required each candidate change;
2. what independent positive and falsifying evidence tests that obligation;
3. whether a later finding is a contract omission, implementation
   nonconformance, or repair-introduced regression;
4. what exact bytes were reviewed and what exact repair/rereview scope follows;
5. which reviewer session was operationally separate from implementation;
6. what raw events support lead, wait, review, repair, recurrence, and round
   measurements; and
7. what remains a human semantic judgement even after machine validation.

Delivery is split into three gates:

- **P0, immediate exact-review control:** report-level traceability,
  pre-mutation finding classification, exact repair/rereview scope, fresh-agent
  exclusions, and honest current-run time evidence. This may be applied to the
  current hardening candidate without changing its accepted packet or candidate
  bytes.
- **P1, typed measurement control:** raw review-process events, paired wait and
  repair intervals, stable review lineages/rounds, effect mappings, explicit
  denominators, and derived KPI output.
- **P2, recurrence v2.1 control:** direct machine joins for `required_by`,
  obligation hazards, negative controls, and trusted scheduler-level reviewer
  provenance.

Implementing P0-P2 is still not proof that the bottleneck improved. That claim
requires live eligible subjects, exact raw outcomes, quality non-inferiority,
and a dated `keep`, `change`, `stop`, or `experiment` decision. The current v2
control remains experimental until its own promotion conditions are met.

## Scope

The bounded scope is the P0 exact-review operating record, the P1 external
development-flow measurement contract, and the P2 pre-live recurrence v2.1
contract described below. It is development-management and AI-acceleration
infrastructure only; it includes no CodeFire product behavior.

## 3. Frozen invariants

### 3.1 Identity and chronology

- Every candidate, packet, report, validation, event record, result, and
  validator source is identified by full commit, tree where applicable, path,
  mode, and SHA-256. A branch name or mutable filename is never identity.
- A repair contract precedes the repair it governs. A report or post-candidate
  prompt cannot retroactively become its contract.
- P0 review begins only after the manager freezes one clean exact candidate.
- Findings are classified and their repair boundary is frozen before any repair
  mutation. If the accepted contract omitted a rule, the manager commits a new
  additive packet; the old packet is not rewritten.
- A repaired candidate receives a new exact identity and a rereview of the
  frozen repair delta plus every inherited obligation and affected surface.

### 3.2 Evidence and semantics

- Raw events precede aggregates. A derived metric without its eligible subjects,
  raw event/effect references, numerator, denominator, and unavailable count is
  invalid.
- A passing candidate-authored test is never the sole independent negative
  control for the behavior it implements.
- Observation, continuous effect episode, and later review opportunity are
  distinct units. One still-open effect reported repeatedly does not become
  several independent defects; a recurrence after proven closure is not folded
  into continuous-open restatement.
- Git proves bytes, paths, commits, and topology. It does not prove semantic
  completeness, honest command execution, reviewer cognition, effect identity,
  or the quality of an oracle.
- Human judgements are typed as such and carry evidence. They never become
  machine facts merely because they are in JSON.

### 3.3 Time and unavailable evidence

- Durations are derived only from ordered events in one monotonic clock domain
  and boot epoch. UTC or Git commit timestamps may establish order or a
  wall-clock observation, but never monotonic service time.
- A missing start/end, a cross-boot interval, an unmatched clock domain, a
  discontinuity, or absent trusted event is `unavailable`, never zero.
- Overlapping waits retain every immediate reason. Per-reason durations may
  overlap; total wait is the interval union, not their arithmetic sum.
- Repair elapsed time and evidence-producing repair service time are different.
  The latter is unavailable unless bounded active intervals exist.

### 3.4 Count-free review

- There is no minimum or target number of findings, observations, hazards,
  surfaces, fields, tests, mutations, report lines, reviewers, or review rounds.
- Exact set coverage, stable identities, terminal dispositions, and falsifiable
  evidence determine acceptance. A count cannot grant authority.
- Every unavailable item stays visible in its denominator and reason set.
- Required counterexamples below are responses to known observed failure
  classes. Their number is not a saturation target, and a reviewer may report
  any additional material finding.

## 4. P0: immediate exact-review operating control

P0 changes no candidate byte. The manager creates a review packet after the
candidate is clean and committed, then assigns a fresh report-only reviewer.

### 4.1 Exact traceability matrix

The final exact review contains one canonical matrix with these columns:

| Field | Rule |
|---|---|
| `obligation_id` | Stable accepted-packet clause or inherited finding ID. No prose-only alias. |
| `origin_generation` | First review generation in which the obligation existed, or `0` for a manager control obligation introduced before the first live generation. |
| `packet_clause_ref` | Exact packet path, SHA-256, heading/row, and acceptance criterion. |
| `authorized_boundary` | Smallest permitted paths/symbols and explicitly forbidden expansion. |
| `hazard_ids` | Every anticipated regression relevant to the selected repair mechanism. |
| `changed_paths` | Exact Git-derived candidate paths required by this obligation. |
| `surface_ids` | Stable symbol/locus/behavior/oracle/claim surfaces on those paths. |
| `claim_ids` | Frozen predecessor-parity, closure, held-out, and negative-control claims. |
| `positive_validation_ids` | Exact clean-candidate exit-zero validations. |
| `negative_control_evidence` | Independent held-out/mutation/rejection evidence; never silently converted to a positive validation. |
| `result` | `closed_proven`, `open`, `routed_open`, or `unavailable`, with exact evidence and next action. |

Coverage is bidirectional:

- every prior obligation has exactly one matrix row or an explicit split with
  stable child IDs and a manager reason;
- every Git-derived changed path occurs in at least one row;
- every changed-path surface occurs in one or more rows;
- every path is `required_by` at least one pre-repair obligation; a path justified
  only by a newly discovered finding is unplanned and remains blocking;
- every frozen claim and required validation is referenced; and
- every known anticipated hazard has falsification evidence or an explicit
  blocking `unavailable` result.

P0 uses this Markdown matrix because the accepted v2 schema cannot encode the
direct path join. It does not pretend the machine already checks the matrix.

### 4.2 Finding classification before repair

Each post-candidate finding has a stable ID and exactly one primary class:

| Class | Meaning | Required manager action before mutation |
|---|---|---|
| `contract_omission` | The accepted packet did not specify the rule needed to close the finding. | Freeze an additive packet/amendment, its hazard, authorized boundary, oracle, and rereview scope. Do not edit the accepted packet. |
| `implementation_nonconformance` | The candidate violates an already frozen clause. | Bind the clause, smallest repair boundary, regression test, and exact delta review. No scope expansion. |
| `repair_introduced_regression` | The repair created a new reachable effect or invalidated an oracle/claim. | Create a successor obligation with production route, before/after evidence, hazard, falsifier, and inherited-surface rereview. |

The classification row also records:

- finding and effect-episode IDs;
- exact candidate commit/tree and evidence;
- affected paths, symbols, owners, effects, and fences;
- packet clause or explicit absence of one;
- proposed repair paths and forbidden paths;
- positive validation and independent negative-control requirements;
- whether a new public API, state machine, persisted shape, classifier, or
  authority boundary is proposed; and
- exact rereview scope, including transitive consumers and unchanged inherited
  obligations.

A disputed class remains `unavailable` and blocks mutation until manager
adjudication. The implementer does not choose a more convenient class.

### 4.3 Fresh-agent exclusion and provenance

The manager records, before review dispatch:

- every implementation, integration, test-authoring, contract-authoring, and
  pre-candidate-audit lane/session/agent ID;
- their worktrees, checkpoint/candidate commits, provider, exact model, and
  effective route;
- the exact candidate-ready event and review packet identity; and
- the set of identities excluded from fresh review.

The reviewer and any reviewer child must:

- have a lane/session/agent ID outside the exclusion set;
- start after candidate freeze;
- use a separate clean worktree at the exact candidate;
- make only one report-path change in the report commit;
- perform no candidate repair, conflict resolution, or test implementation; and
- stop, including all children, before the manager writes the final review
  control/result.

Provider/model diversity is recorded truthfully but is not synonymous with
independence. A new GPT-5.6 session may be operationally independent while
providing no provider diversity; a different provider that edited the candidate
is not independent. Machine checks in P2 will prove only the recorded
scheduler-level separation and topology. They will not claim to prove unbiased
reasoning or absence of knowledge outside the recorded system.

### 4.4 Honest current-run timestamps

From the moment this packet is accepted, the manager retains the following
events when they occur:

| Event | Required evidence |
|---|---|
| `candidate_ready` | exact clean commit/tree, manager observation, monotonic/UTC pair when available |
| `review_start` | fresh session dispatch/start event, candidate identity, clock domain |
| `review_decision` | reviewer's first terminal decision event, exact report subject |
| `report_commit` | report commit identity plus a same-domain event if verdict-to-report duration is claimed |
| `repair_start` / `repair_end` | repair packet and candidate identities, finding IDs, clock domain |
| `wait_start` / `wait_end` | interval ID, immediate reason, blocking artifact, release condition, and clock domain |

No event is backfilled as monotonic evidence from a commit timestamp, file mtime,
chat timestamp, or remembered estimate. Existing commit metadata may be recorded
only as `utc_observation_only`. An event that occurred without suitable evidence
is retained as `unavailable` with the exact reason.

### 4.5 P0 output and acceptance

P0 produces:

1. one exact review packet naming the subject and exclusion set;
2. one report-only fresh review with the traceability and finding matrices;
3. content-addressed event/provenance evidence for events that remain observable;
4. explicit unavailable rows for all earlier unmeasured intervals; and
5. a manager decision that either accepts the exact candidate or freezes the
   next repair packet before mutation.

P0 acceptance requires zero unresolved production, bypass, contract-conformance,
or unowned scope finding. It does not require any number of findings and does
not authorize a claim of faster review.

## 5. P1: typed KPI raw events and denominators

P1 introduces a separate non-product contract. Existing
`codefire.ai_development_flow_record.v1` remains readable and is not silently
reinterpreted.

### 5.1 New records

The implementation adds three closed schemas:

- `codefire.review_process_run.v1`: one exact review/repair lineage and its raw
  evidence;
- `codefire.review_process_cohort.v1`: a manager-frozen eligible-subject list
  chosen before terminal outcomes; and
- `codefire.review_process_result.v1`: machine-derived intervals, raw counts,
  numerators, denominators, unavailable counts, and findings.

The run record contains exactly:

- run, batch, task packet, and review-lineage identities;
- candidate revision/tree/patch/changed paths;
- review round and separately typed policy-generation flag;
- participant roles and scheduler-level provenance refs;
- clock-domain declarations;
- raw events;
- wait-interval declarations paired by stable interval ID;
- repair-episode declarations bound to finding/effect IDs;
- observation-to-effect mappings and their human authority/evidence;
- referenced report/validation/event artifacts; and
- limitations/unavailable reasons.

No authored KPI value, acceptance percentage, average, or conclusion is accepted
in the run record.

### 5.2 Raw event shape

Every event has:

- unique `event_id` and exact `kind`;
- actor lane/session/role and exact subject revision;
- `review_lineage_id` and positive `review_round`;
- evidence ref and event source (`orchestrator`, `manager`, `reviewer`,
  `implementer`, `gate`, `git_commit`, or `provider`);
- time state: `same_boot_monotonic`, `utc_observation_only`, or `unavailable`;
- exact UTC microsecond text when observed;
- monotonic nanoseconds and clock-domain/boot-epoch ID only for
  `same_boot_monotonic`; and
- a closed unavailable-reason enum when time is unavailable.

Required event kinds are:

```text
packet_ready
candidate_ready
review_start
review_decision
report_commit
finding_observed
finding_classified
rereview_scope_frozen
repair_start
repair_active_start
repair_active_end
repair_end
wait_start
wait_end
integration_queue_enter
integration_focused_gate
```

The list is a data vocabulary, not a required event-count target. Events that do
not apply are absent; required lifecycle facts that occurred without observable
evidence have explicit unavailable entries.

Wait reasons are exactly the existing causal-gate vocabulary:

```text
provider_backoff
provider_window
resource_admission
conflict_domain
dependency
review_permit
build_control
manager
credential_or_preflight
unknown
```

Each `wait_start` freezes one immediate reason, release condition, and blocking
artifact. `wait_end` references the same interval ID. Overlapping reasons use
separate interval IDs and are retained rather than forced into one narrative.

### 5.3 Review lineage and rounds

A manager freezes `review_lineage_id` before the first candidate review. It
cannot be reset after a rejection merely because the packet is renamed, the
candidate is rebuilt, or a finding is reclassified. A lineage ends only with:

- the first accepted exact candidate and its integration disposition;
- explicit manager cancellation with all open findings retained; or
- a separately reviewed scope split that names the predecessor lineage and
  carries every inherited obligation to one or more successors.

Round 1 is the first terminal independent exact review after the lineage's
pre-repair contract is frozen. Each `REQUEST_CHANGES` followed by a repaired
candidate increments the round. Interrupted/provider-held attempts without a
terminal exact report are retained as attempts and cost/wait evidence but do
not fabricate a review decision. A generation driven only by a nonblocking
policy change is marked `policy_generation` and reported separately; it never
resets or improves first-pass acceptance.

### 5.4 Same-effect mapping

Each review observation records:

- `observation_id`, `finding_id`, and `effect_episode_id`;
- affected exact surface/invariant family;
- prior observation IDs, if any;
- continuity: `new_effect`, `continuous_open`,
  `recurred_after_proven_closure`, or `unknown`;
- production/claim/oracle evidence refs;
- mapping authority: fresh reviewer plus manager adjudication when disputed;
  and
- mapping state: `verified`, `disputed`, or `unavailable`.

The checker can verify identity continuity and referenced bytes but cannot infer
that two behaviors are semantically the same effect. A renamed finding without
an effect mapping remains `unavailable`; it does not improve a recurrence rate.

### 5.5 Derived KPI definitions

P1 derives and registers these metrics while preserving existing
`MET-FLOW-006`, `MET-FLOW-014`, and `MET-FLOW-015` semantics:

| ID | Definition | Numerator / denominator and unavailable rule |
|---|---|---|
| `MET-REVIEW-001` | first-pass exact acceptance | lineages whose round-1 terminal independent exact verdict is `ACCEPT` / every cohort-eligible lineage with a round-1 opportunity. Missing/untrusted verdicts stay in the denominator and unavailable count. |
| `MET-REVIEW-002` | review rounds per accepted lineage | sum of rounds through first `ACCEPT` / accepted lineages. Open/cancelled lineages are reported separately; denominator zero yields `unavailable`, not zero. |
| `MET-REVIEW-003` | third-review share | eligible lineages reaching round 3 / all eligible lineages. Open and unavailable subjects stay visible. |
| `MET-REVIEW-004` | same-effect repeat-observation rate | successor observations mapped to an earlier effect episode / all successor observations. Emit known lower bound, unavailable mappings, and possible upper bound; do not claim the target passed with unavailable mappings. |
| `MET-REVIEW-005` | recurrence-after-proven-closure rate | `(effect_episode_id, later_generation)` opportunities classified `recurred_after_proven_closure` / every carried `CLOSED_PROVEN` episode with a later eligible review opportunity. |
| `MET-REVIEW-006` | candidate-ready review wait | `candidate_ready` to `review_start` in one clock domain; otherwise unavailable. |
| `MET-FLOW-006` | review lead time | `review_start` to first terminal `review_decision`, same-domain only. |
| `MET-REVIEW-007` | verdict-to-report time | `review_decision` to `report_commit`, same-domain only. |
| `MET-REVIEW-008` | repair elapsed time | `repair_start` to `repair_end`; wait overlap reported separately. This is not active service time. |
| `MET-REVIEW-009` | evidence-producing repair service | union of paired `repair_active_start/end` intervals / repair episode. Missing active boundaries make it unavailable. |
| `MET-REVIEW-010` | wait time by reason and total union | per-reason interval union plus union across all reasons. Overlapping per-reason totals are never summed as total wait. |
| `MET-FLOW-014` | rework ratio | post-rejection evidence-producing repair/retest seconds / total evidence-producing seconds, only when both interval sets are complete; otherwise unavailable. |
| `MET-FLOW-015` | reviewer correction rate | cohort-eligible packets requiring a material correction / reviewed eligible packets, with classification breakdown and unavailable count. |

Every cohort result lists raw subject rows before aggregates. The cohort manifest
freezes inclusion/exclusion criteria and every eligible subject before outcomes.
Post-outcome exclusions are retained and make the aggregate invalid unless a
pre-frozen rule requires them.

### 5.6 Current-run rule

P1 may ingest P0 evidence, but it must not upgrade evidence quality. Events
known only from Git commit timestamps remain `utc_observation_only`; intervals
that began before trustworthy capture remain unavailable. The current run is a
valid first raw subject even when most historical durations are unavailable.
Its purpose is to prove honest capture and expose missing evidence, not create a
favorable baseline.

## 6. P2: recurrence v2.1 direct joins and reviewer provenance

P2 is a pre-live schema break. It creates new schema IDs and does not reinterpret
old v2 JSON bytes.

The new identities and paths are fixed as:

| Role | Schema identity | Path |
|---|---|---|
| control record | `codefire.review_recurrence_control.v2.1` | `docs/development/ai/schemas/review-recurrence-control.v2.1.schema.json` |
| repair manifest | `codefire.review_recurrence_repair_manifest.v2` | `docs/development/ai/schemas/review-recurrence-repair-manifest.v2.schema.json` |
| gate result | `codefire.review_recurrence_gate_result.v2.1` | `docs/development/ai/schemas/review-recurrence-gate-result.v2.1.schema.json` |
| reviewer provenance | `codefire.review_reviewer_provenance.v1` | `docs/development/ai/schemas/review-reviewer-provenance.v1.schema.json` |

### 6.1 Direct traceability joins

The v2.1 repair manifest adds:

- stable `hazards[]` with `hazard_id`, `obligation_id`, repair mechanism,
  observable failure, and `negative_control_claim_ids`;
- a claim evidence role distinguishing `predecessor_parity`, `closure`,
  `held_out_negative`, and `mutation_negative`; and
- exact joins from every obligation to positive validation IDs and from every
  hazard to one or more independent negative-control claim IDs.

The v2.1 control record adds:

- nonempty `required_by` obligation IDs on every planned changed path and
  surface;
- hazard and negative-control claim IDs on each affected surface;
- explicit `unplanned` disposition when a path is justified only by a finding
  first discovered after candidate creation;
- an observation/effect mapping section bound to the P1 run record; and
- a reviewer-provenance artifact reference.

The checker derives these rules:

- every `required_by` ID exists in the manifest or inherited successor set;
- every Git-derived changed path and surface has at least one join;
- a planned path cannot be justified only by a new finding;
- every hazard belongs to one obligation and has at least one negative-control
  claim with a negative evidence role;
- positive timed validations and negative controls remain distinct under the
  existing F3 boundary;
- every referenced claim/validation/hazard appears in exact result coverage;
- unavailable or contradicted joins route a blocking next action; and
- retained history replays the complete joins rather than trusting retained
  counts.

The gate still does not decide whether the manager enumerated every semantic
hazard or whether the reviewer mapped two effects correctly. Independent
preflight and review retain those judgements.

### 6.2 Trusted scheduler-level reviewer provenance

The v2.1 record binds a closed, manager-controlled provenance artifact that
contains:

- exact candidate-ready and review-dispatch events;
- implementation/test/integration/contract-author lane and session IDs;
- fresh-reviewer and reviewer-child lane/session IDs;
- provider, exact model, effective route, worktree, and role for each;
- exclusion-set membership;
- report-only write-set evidence and terminal child count; and
- artifact path, commit, mode, byte count, and SHA-256.

The checker requires reviewer identities to be disjoint from the exclusion set,
the review start to follow candidate freeze, the checkout to match the exact
candidate, the report commit to be report-only, and the reviewer tree to be
terminal. A self-authored `reviewer_independent: true`, Git author string, or
provider difference is insufficient.

This is deliberately called **scheduler-level provenance**. Local event bytes
can prove what the configured manager/orchestrator recorded; they cannot prove
human authorship, absence of off-system communication, lack of prior knowledge,
or unbiased judgement. Those remain explicit human acceptance statements.

### 6.3 Migration

- Existing accepted packets, reviews, v1 records, and v2 records remain
  immutable.
- v1 remains compatibility input and always blocks the normal live decision.
- v2 remains readable as experimental historical evidence but cannot be
  silently treated as v2.1.
- No live v2 anchor or live v2 authority chain exists at this freeze, so no live
  authority is migrated.
- The first Cargo/KGI live manifests and anchors must target the independently
  accepted v2.1 contract if P2 completes before live-arm creation. If a manager
  chooses otherwise, that is a new explicit contract decision and cannot be
  inferred from this packet.
- Retained synthetic fixtures may be regenerated for tests; immutable historical
  report/packet bytes are never rewritten.

## 7. Plan: dependency order and ownership

| Order | Gate | Work | Exit evidence |
|---:|---|---|---|
| 0 | Contract preflight | Review this exact packet for semantic completeness, counterexample adequacy, feasibility, and conflicts with accepted recurrence contracts. | Manager acceptance of one exact packet commit/tree/SHA, or a revised packet and fresh preflight. |
| 1 | P0 | Freeze the final hardening candidate, start honest event capture, dispatch a fresh excluded reviewer, and use the two P0 matrices. | Exact report-only verdict, provenance/event evidence, classifications, and zero unresolved finding or a frozen successor repair packet. |
| 2 | P1 foundation | Add run/cohort/result schemas and total parsers/checkers. Do not aggregate yet. | Schema/adversarial tests for malformed, duplicate, missing, cross-clock, and unavailable evidence. |
| 3 | P1 derivation | Derive intervals, rounds, effect counts, denominators, and cohort metrics from raw records. | Deliberate-mutation matrix, one honest current-run record/result, and independent exact review. |
| 4 | P2 | Add v2.1 manifest/control/result joins, P1 record binding, provenance binding, and full retained replay. | Current/retained counterexamples, positive controls, full tests, and fresh exact report-only review. |
| 5 | Live experiment | Independently preflight Cargo/KGI manifests, commit manager-owned anchors, and run required bootstrap/successor generations. | Anchored exact results for both arms, raw P1 records, and no false closure. |
| 6 | Evaluation | Run blind causal/provider review and compare eligible cohorts with all unavailable values and costs. | Dated `keep`, `change`, `stop`, or `experiment` decision. |
| 7 | Lifecycle close | Synchronize issues/maps/history, then Close, Install/Dogfood where applicable, and Retrospect on one exact integrated revision. | Standard lifecycle evidence; no speed claim without the evaluation gate. |

P0 does not wait for P1 implementation. P1 and P2 use separate write owners and
compose only after focused review. Heavy full suites are serialized. No Cargo,
Rust, install, or product gate is required for these control-plane-only changes.

## 8. Allowed write set

### 8.1 This contract-draft commit

The Freeze/Plan commit that introduces this proposal may change exactly:

- `docs/development/ai/task-packets/cfr168-review-bottleneck-operating-controls-freeze-20260731.md`.

No other path is authorized in the contract-draft commit.

### 8.2 P0 artifacts after manager acceptance

The manager must replace placeholders with exact paths before dispatch. P0 may
add only:

- one review task packet under `docs/development/ai/task-packets/`;
- one report under one new
  `docs/development/ai/swarm-results/cfr168-review-bottleneck-p0-*/` directory;
- one event/provenance evidence namespace under
  `docs/development/ai/review-process/evidence/<run_id>/`; and
- one manager adjudication under `docs/development/ai/swarm-batches/`.

P0 may not change the accepted hardening packet, implementation candidate,
recurrence schemas/tools, product source, tests, history, issue status, live
anchor, or prior evidence.

### 8.3 P1 implementation

- `docs/development/ai/schemas/review-process-run.v1.schema.json`;
- `docs/development/ai/schemas/review-process-cohort.v1.schema.json`;
- `docs/development/ai/schemas/review-process-result.v1.schema.json`;
- `docs/development/ai/schemas/README.md`;
- `docs/development/ai/review-process/README.md`;
- `docs/development/ai/development-flow-kgi-kpi.md`;
- `docs/development/ai/stage5-manual-control-runbook.md`;
- `docs/development/ai/operating-protocol.md`;
- `docs/development/tools/check_review_process_run.py`;
- `docs/development/tools/summarize_review_process_cohort.py`;
- `docs/development/tools/check_ai_contracts.py` only for schema registration;
- `docs/development/tools/test_ai_tools.py`;
- one new current-run record/result/evidence namespace under
  `docs/development/ai/review-process/`; and
- one bounded implementation/preflight/result directory under
  `docs/development/ai/swarm-results/`.

### 8.4 P2 implementation

- `docs/development/ai/schemas/review-recurrence-control.v2.1.schema.json`;
- `docs/development/ai/schemas/review-recurrence-repair-manifest.v2.schema.json`;
- `docs/development/ai/schemas/review-recurrence-gate-result.v2.1.schema.json`;
- `docs/development/ai/schemas/review-reviewer-provenance.v1.schema.json`;
- `docs/development/ai/schemas/README.md`;
- `docs/development/ai/review-recurrence/README.md`;
- `docs/development/ai/validation-report-template.md`;
- `docs/development/ai/operating-protocol.md`;
- `docs/development/tools/check_review_recurrence.py`;
- `docs/development/tools/recurrence_v2_model.py`;
- `docs/development/tools/recurrence_v2_evidence.py`;
- `docs/development/tools/recurrence_v2_decision.py`;
- `docs/development/tools/recurrence_v2_authority.py`;
- `docs/development/tools/recurrence_v2_history.py`;
- `docs/development/tools/recurrence_v2_topology.py`;
- `docs/development/tools/recurrence_v2_fixture.py`;
- new `docs/development/tools/recurrence_v21_model.py`,
  `docs/development/tools/recurrence_v21_evidence.py`,
  `docs/development/tools/recurrence_v21_decision.py`,
  `docs/development/tools/recurrence_v21_history.py`, and
  `docs/development/tools/recurrence_v21_fixture.py` only if the independently
  preflighted implementation packet chooses the separate-model compatibility
  design;
- `docs/development/tools/check_ai_contracts.py` only for schema registration;
- `docs/development/tools/test_ai_tools.py`; and
- one bounded implementation/preflight/result directory under
  `docs/development/ai/swarm-results/`.

Any new shared helper, event source, schema path, production path, history/issue
edit, or live authority artifact requires a manager scope decision before
mutation. A broad prefix in this section is not permission to edit every file
under that prefix; the implementation packet must list exact files.

## 9. Required counterexamples and positive controls

The following cases are mandatory because they correspond to the seven observed
readiness gaps. They are not a finding or test quota.

### 9.1 P0 report/process cases

| ID | Mutation or failure | Required result |
|---|---|---|
| `P0-T1` | Omit one Git-changed path from the traceability matrix. | Review cannot accept. |
| `P0-T2` | Include a path/surface with no prior obligation, hazard, or newly blocking finding. | Mark unplanned and block; never infer `required_by`. |
| `P0-T3` | Claim one negative-control mutation only through a passing candidate-authored test. | Evidence remains incomplete. |
| `P0-T4` | Begin repair before classifying the finding and freezing its repair/rereview scope. | Stop mutation and return to Freeze. |
| `P0-T5` | Label an absent accepted rule as implementation nonconformance. | Reclassify as `contract_omission`; freeze an additive packet. |
| `P0-T6` | Reuse an implementer, test author, integrator, contract author, or pre-candidate auditor as the fresh reviewer. | Fresh-review provenance fails; dispatch a different session. |
| `P0-T7` | Use a new provider but a session that edited the candidate. | Independence fails despite provider diversity. |
| `P0-T8` | Derive service time from Git commit timestamps or remembered wall time. | Record `utc_observation_only` or `unavailable`; no duration. |
| `P0-T9` | Add a 100-finding target or report-length proxy. | Reject as count gaming. |
| `P0-P1` | Exact matrix covers every path/obligation/claim/hazard with independent evidence and zero unresolved row. | P0 may accept the exact candidate subject, subject to all other gates. |

### 9.2 P1 event/KPI cases

| ID | Mutation or failure | Required machine result |
|---|---|---|
| `P1-T1` | Duplicate event, interval, lineage, observation, or effect ID with different content. | Structured failure before dictionary conversion. |
| `P1-T2` | `review_start` precedes `candidate_ready`, or uses another candidate tree. | Invalid run; no KPI output. |
| `P1-T3` | Pair interval endpoints from different clock domains/boot epochs. | Duration unavailable, never a wall-time substitute. |
| `P1-T4` | Omit a wait end, report commit, review decision, or repair end. | Retain in-progress/unavailable state; never zero. |
| `P1-T5` | Sum overlapping per-reason waits as total wait. | Recompute interval union and expose overlap. |
| `P1-T6` | Label repair elapsed including waits as active repair service. | Reject the service metric unless active intervals exist. |
| `P1-T7` | Rename a continuous-open finding and omit its earlier effect mapping. | Mapping unavailable; recurrence lower/upper bounds expose it. |
| `P1-T8` | Label a continuous-open restatement as recurrence after proven closure. | Contradicted continuity finding. |
| `P1-T9` | Reset a rejected lineage to round 1 by renaming the packet/surface. | Lineage continuity failure. |
| `P1-T10` | Remove an unavailable/failed first review from a denominator after its outcome is known. | Cohort result invalid. |
| `P1-T11` | Use denominator zero to emit rate zero or 100%. | Emit unavailable with raw zero denominator. |
| `P1-T12` | Author a favorable KPI value inconsistent with raw events. | Authored aggregate rejected; checker-derived result prevails. |
| `P1-P1` | One same-boot complete run with deliberate overlapping waits and known same-effect continuity. | Exact intervals, unions, raw counts, and denominators are derived correctly. |

### 9.3 P2 join/provenance cases

| ID | Mutation or failure | Required machine result |
|---|---|---|
| `P2-T1` | `required_by` names an absent obligation or an obligation from another authority chain. | Structured block. |
| `P2-T2` | A changed path or surface has an empty join. | Structured block. |
| `P2-T3` | A path is marked planned but is justified only by a same-generation new finding. | Unplanned/blocking. |
| `P2-T4` | A hazard has no negative-control claim, or cites a positive-only claim. | Structured block or unavailable next action. |
| `P2-T5` | Move an expected-failure mutation into required positive timed validations. | F3 boundary violation and block. |
| `P2-T6` | Change a retained required-by/hazard/control join while preserving counts. | Full-history replay blocks. |
| `P2-T7` | Set `reviewer_independent: true` without trusted provenance. | No integration authority. |
| `P2-T8` | Reviewer lane/session intersects the implementation exclusion set. | Provenance block. |
| `P2-T9` | Reviewer starts before candidate freeze or reviews another tree. | Provenance block. |
| `P2-T10` | Reviewer report commit changes a test/tool/product byte. | Report-only topology block. |
| `P2-T11` | Add an unknown provenance/result member or duplicate JSON key. | Closed-schema/strict-decode block. |
| `P2-P1` | Disjoint post-freeze reviewer, report-only commit, complete joins, independent negatives, and unchanged retained replay. | Structural provenance/join controls pass while human-boundary text remains explicit. |

## Validation

### 10.1 This docs-only Freeze/Plan candidate

From its clean worktree:

```bash
python3 docs/development/tools/check_ai_acceleration_docs.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_development.py
git diff --check
```

Verify that every cited existing evidence/source path exists at the candidate,
that paths explicitly declared as future additions are currently absent or
unowned, and that the commit changes exactly this packet. No product, Cargo,
install, or dogfood test is required for this docs-only contract proposal.

### 10.2 P1/P2 implementation candidates

Each later implementation packet must include, at minimum:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  docs.development.tools.test_ai_tools.ReviewProcessTests
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  docs.development.tools.test_ai_tools.ReviewRecurrenceTests
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  docs.development.tools.test_ai_tools
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile docs/development/tools/*.py
python3 docs/development/tools/check_ai_contracts.py
python3 docs/development/tools/check_ai_acceleration_docs.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_development.py
git diff --check
```

If `ReviewProcessTests` is introduced under a different exact unittest symbol,
the manager must revise and preflight that implementation packet before dispatch;
the implementer may not silently skip it. Test counts are measured from the
actual run, never copied from history. Broad suites are serialized to protect
memory and disk.

Every P0/P1/P2 counterexample above must have an exact test or report evidence
row. A passing positive fixture does not substitute for a deliberate mutation.

## Non-Goals

- Do not modify the accepted live-freeze hardening packet, its adjudication, or
  immutable preflight/review reports.
- Do not modify or integrate the in-progress hardening candidate under this
  contract-draft branch.
- Do not create Cargo/KGI manifests, ports, live anchors, live records, synthetic
  authority generations, or product code.
- Do not adopt a permanent template shape, finding count, checklist length,
  repair-size threshold, reviewer count, or universal severity policy.
- Do not claim machine proof of semantic completeness, honest execution,
  effect equivalence, authorship, or cognitive reviewer independence.
- Do not use provider diversity, agent count, files, lines, commits, tests, or
  report length as throughput success.
- Do not backfill unknown timestamps/costs from estimates or treat unavailable
  as zero.
- Do not claim the review bottleneck is solved from a green unit suite, one
  accepted candidate, or a v2.1 structural `ALLOW`.

## 12. Completion evidence and stopping rule

| Gate | Evidence required before complete |
|---|---|
| Contract Freeze | Exact packet commit/tree/SHA; primary semantic review, adversarial contradiction/counterexample review, and feasibility review all target those bytes; every item is mapped, truthfully excluded as a human/non-goal boundary, or unresolved. Manager records the decision. |
| P0 | Exact candidate identity; complete matrices; fresh excluded reviewer provenance; honest raw time evidence/unavailable rows; zero unresolved material findings, or a pre-repair successor packet. |
| P1 | Closed schemas and total checker; all P1 mutations killed; current-run record/result; frozen cohort denominator; independent exact review; no authored aggregate accepted. |
| P2 | New schema identities and migration decision; all direct joins/provenance checks; current and retained mutations killed; positive controls; independent exact review; old bytes unchanged. |
| Live experiment | Cargo and KGI/KPI manager-owned manifests/anchors, required bootstrap and successor generations, raw P1 records, no false closure, and complete unavailable/cost reporting. |
| Bottleneck outcome | Comparable preregistered cohort, exact first-pass/round/recurrence/wait/repair results, quality guardrails, blind causal/provider evaluation, and dated process decision. |

There is no fixed unresolved-finding search count. Contract preflight stops when
the three bounded roles examine the same exact bytes, all raised items have a
disposition, and all three report zero unresolved authorization-changing choice,
contradiction, bypass class, or unowned test surface. A later finding is first
classified as `contract_omission`, `implementation_nonconformance`, or
`repair_introduced_regression`; it does not trigger an unbounded generic review
loop.

The manager may accept this packet for implementation only in a separate
adjudication commit. Until then:

```text
IMPLEMENTATION_AUTHORIZATION: BLOCKED
PROCESS_ADOPTION: BLOCKED
REVIEW_BOTTLENECK_SOLVED_CLAIM: BLOCKED
```

## Handoff

The contract author returns one clean docs-only commit with the full commit,
tree, packet path, packet SHA-256, changed-path list, and exact validation
results. The manager then assigns bounded primary, adversarial, and feasibility
preflights to fresh report-only lanes. Their reports must bind those exact bytes;
they may not implement P0, P1, or P2. Any correction produces a new packet
commit/tree/SHA and invalidates earlier acceptance claims until the required
preflights target the new bytes.
