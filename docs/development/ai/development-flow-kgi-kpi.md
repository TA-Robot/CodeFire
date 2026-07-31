# AI Development Flow KGI / KPI

Status: authoritative operating policy

Owner: development manager

Review cadence: task completion, active-batch daily checkpoint, every five comparable task packets,
resource/provider incident, and batch Retrospect

Machine records: `flow-metrics/*.json`

## Purpose

This policy makes development speed, quality, cost, continuity, and coordination inspectable without
treating agent count, patch size, or raw activity as progress. It promotes the useful parts of the
Phase 2 KPI scorecard into the operating system for CodeFire development.

The hierarchy is:

1. **KGI** states the outcome the development system must achieve.
2. **KPI** explains why that outcome is or is not being achieved.
3. **Events and evidence** provide immutable facts from which the KPI is derived.
4. **A management decision** records whether to keep, change, stop, or experiment with the current
   development arrangement.

Long-running work is not a failure by itself. A lane is healthy when it keeps producing unique,
trusted signals such as a reproduced counterexample, a bounded design decision, a durable
checkpoint, a validation result, or an independent finding. Elapsed time without such a signal is a
diagnostic, not an automatic timeout.

## KGI

| ID | Outcome | Target | Guardrail |
|---|---|---|---|
| `KGI-DEV-001` | Trusted frozen-batch completion | Every issue in the frozen batch reaches a terminal disposition with source acceptance, applicable Install and Dogfood/Evaluate, final Close, and Retrospect bound to one exact integrated revision and, when applicable, its installed artifact | No unresolved blocking finding or stale evidence may be inferred away; source acceptance is not final Close, and `not_applicable` requires an evidence-backed manager scope decision |
| `KGI-DEV-002` | Quality integrity | Zero escaped critical invariant defects, zero accepted false-completion claims, and 100% required evidence completeness | Speed and cost never compensate for identity, atomicity, security, user-data, or authority defects |
| `KGI-DEV-003` | Delivery continuity | Zero lost user/source work; every mutated lane has a durable recovery point; provider, host, memory, and disk incidents have measured recovery | A long task may continue for hours, but an uncheckpointed mutation or unbounded resource consumer may not |
| `KGI-DEV-004` | Codex-quota-independent operation | Zero development stoppages caused solely by Codex quota exhaustion; implementation and review remain resumable through Claude/Fugu with portable Git/log evidence | Codex stays the coordinator and guarded final authority where selected; silent model downgrade is forbidden |
| `KGI-DEV-005` | Improving cost to trusted integration | After the first five comparable packets establish a baseline, the next comparable cohort reduces median trusted-integration lead time and total cost by at least 20% with non-inferior `KGI-DEV-002` results | Do not compare unlike task/risk classes or optimize only first-patch latency |

`KGI-DEV-005` is an initial decision target, not a product promise. If five comparable packets do
not exist, its status is `in_progress`, and the correct action is to collect a cohort rather than
invent a baseline.

## KPI

### Flow and speed

| ID | Definition | Start / end |
|---|---|---|
| `MET-FLOW-001` | Trusted integration lead time | Packet ready to reviewed, validated, integration-ready exact revision |
| `MET-FLOW-002` | Lifecycle stage lead time | Entry to exit for Discover, Freeze, Plan, risk-based Preflight, Implement, Validate, Candidate Freeze, Independent Review, Impact/Actionability Adjudication, Repair/Rereview, Integrate, Install, Dogfood/Evaluate, final Close, or Retrospect |
| `MET-FLOW-003` | Time to first trusted signal | Lane start to first reproduced fact, bounded decision, validation result, or committed artifact |
| `MET-FLOW-004` | Time to first durable checkpoint | Lane start to the first committed source/report checkpoint after work begins |
| `MET-FLOW-005` | Validation queue time | Validation-ready to actual gate start |
| `MET-FLOW-006` | Review lead time | Review start to first terminal exact review decision in one monotonic clock/boot domain; otherwise unavailable |
| `MET-FLOW-007` | Manager integration time | Accepted candidate entering the integration queue to focused post-integration gate completion |
| `MET-FLOW-008` | Flow efficiency | Evidence-producing active seconds divided by total trusted-integration lead seconds |
| `MET-FLOW-009` | WIP age | Current time minus the last unique trusted signal for an unfinished lane |
| `MET-FLOW-010` | No-unique-artifact time | Lane time that produced no unique fact, patch, report, or gate result |
| `MET-FLOW-011` | Useful parallelism ratio | Useful lane-seconds on the critical batch divided by total agent lane-seconds |

### Quality and rework

| ID | Definition |
|---|---|
| `MET-FLOW-012` | First-pass gate rate: required gates passing on their first exact-candidate run / required gates run |
| `MET-FLOW-013` | Flaky gate rate: gates whose result changes without an intended source/environment change / gates run |
| `MET-FLOW-014` | Rework ratio: post-rejection repair/retest lane-seconds / all evidence-producing lane-seconds. Both are derived only from complete paired same-boot evidence-active intervals: union overlaps within each lane, then sum the per-lane unions; concurrent lanes remain separate lane-seconds. Missing class/endpoints, cross-clock/boot pairs, or otherwise incomplete applicable intervals make the ratio unavailable. |
| `MET-FLOW-015` | Reviewer correction rate: cohort-eligible packets with at least one adjudicated `required_correction` / reviewed eligible packets; also report `contract_omission`, `implementation_nonconformance`, and `repair_introduced_regression` raw counts and unavailable causal adjudications |
| `MET-FLOW-016` | Integration conflict count and resolution seconds per accepted package |
| `MET-FLOW-027` | Required evidence completeness: resolved required evidence / required evidence |
| `MET-AI-040` | Escaped consistency defect after trusted integration |
| `MET-AI-041` | Escaped critical identity, atomicity, security, authority, or user-data defect |
| `MET-AI-042` | False completion claim accepted without the required evidence |

### Review actionability and recurrence

These metrics diagnose review/repair behavior; none is a finding target or a
reason to suppress valid observations. Every result emits raw subject rows,
numerator, denominator, unavailable/disputed count, and immutable evidence refs
before an aggregate:

This registry is not a twenty-item per-task checklist. Core cohort diagnostics
are `MET-REVIEW-001`, `002`, `004`, `006`, `008`/`009`, `015`, `016`, and
`017`, together with `MET-FLOW-014`/`015`; capture their underlying lifecycle
events when they occur and derive them at the scheduled cohort review. The
other `MET-REVIEW-*` entries are conditional diagnostics, derived only when the
lineage reaches the named round, closure opportunity, report event, wait,
reopen/conflict, impact investigation, or must-fix decision. A genuinely
non-applicable event is absent, not manufactured. An event that should have
occurred but lacks trusted evidence is unavailable and stays in the applicable
denominator. Do not require a human to hand-calculate every aggregate after
every packet.

| ID | Definition and denominator rule |
|---|---|
| `MET-REVIEW-001` | First-pass exact acceptance: eligible lineages with round-1 terminal independent `ACCEPT` / every frozen cohort lineage with a round-1 opportunity. Missing or untrusted decisions remain unavailable in the denominator. |
| `MET-REVIEW-002` | Review rounds through first acceptance / accepted lineage; open and cancelled lineages remain separate and a zero denominator is unavailable. |
| `MET-REVIEW-003` | Eligible lineages reaching round 3 / all eligible lineages; no favorable exclusion after outcome. |
| `MET-REVIEW-004` | Successor observations mapped to an earlier same-effect episode / all successor observations. Report a known lower bound, unavailable mappings, and possible upper bound. |
| `MET-REVIEW-005` | Effects recurring after proven closure / every carried proven-closed effect with a later eligible review opportunity; continuous-open observations are not recurrence. |
| `MET-REVIEW-006` | Candidate-ready to review-start wait from same-clock events; otherwise unavailable. |
| `MET-REVIEW-007` | Terminal review decision to committed report from same-clock events; otherwise unavailable. |
| `MET-REVIEW-008` | Repair start to repair end, with overlapping wait reported separately; this is not active service time. |
| `MET-REVIEW-009` | Union of paired evidence-producing repair-active intervals / repair episode; incomplete boundaries are unavailable. |
| `MET-REVIEW-010` | Wait interval union by immediate reason and total union. Overlapping reason totals are never arithmetically summed. |
| `MET-REVIEW-011` | Raw distribution of the six actionability dispositions / all observations, with pending impact, disputed and unavailable adjudications separate. No distribution is a success target. |
| `MET-REVIEW-012` | Accepted-variance loci reopened without one admissible trigger plus exact new evidence / all admitted reopens. Target zero; raw zero denominator is unavailable. |
| `MET-REVIEW-013` | Reopen requests lacking an allowed trigger/evidence / all reopen requests. Rejected requests remain visible as churn. |
| `MET-REVIEW-014` | Active manager decisions reversed without allowed trigger/evidence / all supersessions. Evidence-backed supersessions are reported separately. |
| `MET-REVIEW-015` | Contrary blocking proposals without a valid trigger / later review opportunities for active accepted-variance loci. Unknown materiality remains unavailable. |
| `MET-REVIEW-016` | Repair episodes begun for a non-`required_correction` disposition / all repair episodes, plus their available service / total repair service. Both target zero. |
| `MET-REVIEW-017` | Accepted/integrated candidates with acceptance-critical unknown/unavailable impact and no completed investigation / all accepted/integrated eligible candidates. Target zero. |
| `MET-REVIEW-018` | High-impact, disputed, or variance-reversal impact proposals materially changed by distinct manager adjudication / all such observations with both events. Diagnostic only; missing adjudications stay separate. |
| `MET-REVIEW-019` | Impact-investigation request-to-completion intervals plus open/unavailable investigations / all requested investigations. No missing end becomes zero. |
| `MET-REVIEW-020` | Attempts to offset a proved must-fix trigger using cost, workaround, score, vote, or neutral-axis count / all proved trigger adjudications. Target zero; rejected attempts remain counted. |

`MET-FLOW-015` supplies the required-correction causal breakdown. The three
classes are mutually exclusive only after actionability is
`required_correction`; accepted variance, nonblocking improvement, out of
scope, duplicate, and invalid/unsubstantiated observations never receive a
causal class and never count as corrections.

A `duplicate` stays in observation-level workload/churn and
`MET-REVIEW-011` denominators because the review work occurred. Its canonical
reference prevents a new unique finding/effect/issue/repair unit and prevents a
second `MET-REVIEW-005` closure-opportunity unit. Metric definitions, not one
global deduplication rule, determine the denominator.

### Cost, provider, and coordination

| ID | Definition |
|---|---|
| `MET-FLOW-017` | Provider unavailable/retry seconds and retry count, separated by provider/model/status |
| `MET-FLOW-024` | Codex coordinator token delta per measurement window and per trusted package |
| `MET-FLOW-025` | Provider-reported model cost per lane and per trusted package |
| `MET-AI-060` | Total model tokens/cost per trusted integration |
| `MET-AI-061` | Tool calls per trusted integration |
| `MET-AI-062` | Context bytes/tokens read |
| `MET-AI-064` | Retry and rework cost |
| `MET-AI-065` | Manager/reviewer minutes |
| `MET-AI-066` | Ultra share and its measured marginal quality/lead-time gain |

Unknown billing, cache, or token values are `unavailable`, never zero. Provider token units are not
added to one another unless the provider definitions are demonstrably comparable.

### Reliability and resources

| ID | Definition | Current safety rule |
|---|---|---|
| `MET-FLOW-018` | Incident recovery time | Incident observed to safe coordination restored |
| `MET-FLOW-019` | Checkpoint coverage | Mutated lanes with a durable checkpoint / mutated lanes |
| `MET-FLOW-020` | Workspace bytes | Measure before/after every heavy gate |
| `MET-FLOW-021` | Cargo target bytes | One shared target; reproducible cache only |
| `MET-FLOW-022` | Minimum projected effective `MemAvailable` after candidate reservation | Do not launch any lane below 6 GiB |
| `MET-FLOW-023` | Maximum swap used | Do not launch any lane above 2 GiB used |
| `MET-FLOW-026` | Lost user/source work | Target zero |
| `MET-AI-070` | Lost user work | Target zero |
| `MET-AI-071` | Unauthorized mutation | Target zero |
| `MET-AI-072` | Outside-root access | Target zero |
| `MET-AI-075` | Silent model/tier downgrade | Target zero |

Heavy work also remains blocked when `/workspace` exceeds 20 GiB or the mounted filesystem reports
less than 100 GiB available. Windows C-drive free space is an explicit `unavailable` measurement
when the container cannot observe it.

## Measurement Dimensions

Every duration, count, rate, and cost must carry enough dimensions to remain comparable:

- batch and immutable source revision;
- task packet and task class;
- lifecycle stage and role;
- provider, exact model, and effective route;
- lane ID and candidate/review identity;
- focused, broad, install, or dogfood validation class;
- measured, estimated, unavailable, or still in progress;
- evidence locator or immutable log prefix.

At minimum, cohorts separate docs, bounded implementation, architecture/security repair, independent
review, validation, integration, and recovery work. A 40-minute adversarial review is not compared to
a 40-minute mechanical docs edit.

## Time, Lineage, and Cohort Rules

- Derive a duration only from ordered events in the same monotonic clock domain
  and boot epoch. UTC timestamps and Git commit time may establish order or a
  wall-clock observation but never service time.
- A missing endpoint, cross-boot/domain pair, clock discontinuity, or absent
  trusted event makes the duration unavailable, never zero.
- Pair wait and active-repair endpoints by stable interval ID. Per-reason wait
  unions may overlap; total wait is the union across intervals, not a sum of
  reason totals.
- For `MET-FLOW-014`, every evidence-active interval has a stable pair ID, lane,
  same monotonic clock/boot identity, and an activity class that distinguishes
  post-rejection repair/retest from other evidence production. Compute the
  denominator by unioning all evidence-active intervals within each lane and
  summing those lane durations. Compute the numerator the same way after
  selecting post-rejection repair/retest intervals. Overlap inside one lane is
  never double-counted; simultaneous work in distinct lanes is summed as
  lane-seconds, never collapsed into a cross-lane elapsed-time union. A missing
  endpoint/class, cross-clock or cross-boot pair, or incomplete applicable lane
  makes the metric unavailable rather than zero or a partial favorable ratio.
- Freeze `review_lineage_id` before round 1 and preserve it across rejection,
  repair, packet rename, and rebuilt candidates. A separately reviewed scope
  split must carry inherited obligations and the predecessor lineage.
- The manager freezes cohort eligibility, inclusion/exclusion rules, and every
  eligible subject before terminal outcomes. An outcome-dependent exclusion is
  retained and invalidates the aggregate unless a pre-frozen rule requires it.
- Emit raw subject rows and unavailable counts before a rate or median. Zero
  denominator yields unavailable, never a favorable zero or 100 percent.

## Required Events

The manager records the applicable events in the active evidence set. Put fields
supported by v1 in `flow-metrics/*.json`; until the migration below, bind newer
review events in a content-addressed supplemental record rather than forcing
them into the old schema:

1. packet ready and lane start;
2. first trusted signal;
3. first source/report mutation;
4. each durable checkpoint;
5. candidate ready and immutable candidate/reviewer-exclusion identity;
6. focused/broad gate start and result, including first failure;
7. review start, observation, impact proposal/adjudication, impact investigation,
   actionability proposal/adjudication, required-correction causal decision,
   rereview-scope freeze, terminal decision, and report commit as applicable;
8. provider retry storm, reroute, resume, or deliberate hedge stop;
9. integration queue entry, conflict, and integrated focused-gate result;
10. resource sample and every host/provider/resource incident;
11. source acceptance, Install, Dogfood/Evaluate, final Close, and Retrospect completion;
12. decision-locus conflict, accepted-variance decision, reopen request,
    admitted/rejected trigger, supersession, non-required repair attempt, and
    must-fix offset/score attempt as applicable.

Active append-only logs are bound by `prefix_bytes + sha256`; later appends do not invalidate an
earlier measurement record. A mutable filename or a transcript summary alone is not evidence.

### Current machine-contract limitation

`codefire.ai_development_flow_record.v1` cannot fully represent review lineage,
paired wait/active-repair intervals, effect/locus continuity, impact and
actionability adjudication, frozen eligible cohorts, or their denominators. It
also permits authored measurement values. Therefore v1 shape validation is not
proof of any `MET-REVIEW-*` result. For current runs, retain the raw supplemental
events and subject rows as content-addressed human evidence, mark non-derivable
values unavailable, and let the manager report diagnostics without claiming
machine enforcement. The P1 section of the review-bottleneck implementation
packet proposes new closed run/cohort/result schemas and a checker; those
contracts remain future work until separately accepted, implemented, and
reviewed. Never backfill old events or silently reinterpret accepted v1 bytes.

## Interpretation and Management Triggers

These are review triggers, not task-killing timeouts:

| Signal | Manager action |
|---|---|
| No trusted signal for 20 minutes, or no unique artifact for 45 minutes | Inspect whether the lane is still reducing uncertainty; narrow the packet, request a checkpoint, or replace duplicated work |
| First mutation exists for 60 minutes without a durable checkpoint | Require a WIP commit before more long validation or design expansion |
| Three consecutive provider failures or 10 minutes of provider-only waiting | Resume the exact session when possible; otherwise reroute with the same immutable packet and record the failure cost |
| Primary candidate is clean while a hedge has no unique artifact | Stop the duplicate hedge and redeploy the slot to independent review or the next dependency |
| First-pass gate rate below 80% across five comparable packets | Improve focused tests, preflight, and packet acceptance criteria before increasing parallelism |
| Reviewer correction rate above 50% across five comparable packets | Improve task/context packets and require earlier adversarial design review |
| Flaky gate rate above zero on release evidence | Treat the gate as unresolved until the flake is fixed or deterministically classified |
| Validation queue exceeds 15% of trusted-integration lead time | Reorder light/heavy gates and keep the single heavy slot fed without overlapping full suites |
| Manager integration exceeds 25% of cycle time or a packet causes repeated conflicts | Reduce write-set overlap, add a composition packet, or move shared primitives earlier |
| Workspace/resource guardrail is crossed | Stop new heavy work, checkpoint, measure, clean only reproducible caches, and record recovery |
| One Codex quota incident stops the batch, or coordinator tokens per trusted package rise between cohorts | Move more bounded implementation/review to Claude/Fugu, improve context packs, and retain Codex for global decisions/final assurance |

Changing a threshold requires a dated decision with its cohort and evidence. It must not be loosened
silently to make a current run look healthy.

## Review Cadence

- **Per lane:** record start, trusted signal, checkpoint, candidate/review result, and termination.
- **Daily while a batch is active:** inspect aged WIP, queue time, resources, provider failures, and
  whether every lane still lies on the dependency path.
- **After every five comparable packets:** compare lead time, first-pass quality, rework, cost, and
  useful parallelism; decide `keep`, `change`, `stop`, or `experiment`.
- **After every incident:** record lost work, recovery time, root cause, and a preventive control.
- **At Retrospect:** close the KGI status, explain misses, and promote only changes supported by the
  cohort. Negative and inconclusive results remain in the record.

## Anti-Metrics

The following are context only and never count as progress on their own:

- generated lines or files;
- commit count;
- spawned or simultaneously active agent count;
- reasoning tier;
- tool-call volume;
- raw transcript size;
- time spent without a trusted signal;
- number of model approvals;
- context-window utilization.
- finding, observation, reviewer, review-round, checklist-item, test, mutation,
  or report-line count;
- weighted impact/actionability score, severity sum, vote count, completed-field
  percentage, or a favorable disposition distribution.

The north-star result remains trusted integration, not activity.
