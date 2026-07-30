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
| `KGI-DEV-001` | Trusted frozen-batch completion | Every issue in the frozen batch reaches a terminal disposition on one exact integrated revision, with required review/gates, Close, Install, Dogfood, and Retrospect complete | No unresolved blocking finding or stale evidence may be inferred away |
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
| `MET-FLOW-002` | Lifecycle stage lead time | Entry to exit for Discover, Freeze, Plan, Implement, Review, Integrate, Close, Install, Dogfood, or Retrospect |
| `MET-FLOW-003` | Time to first trusted signal | Lane start to first reproduced fact, bounded decision, validation result, or committed artifact |
| `MET-FLOW-004` | Time to first durable checkpoint | Lane start to the first committed source/report checkpoint after work begins |
| `MET-FLOW-005` | Validation queue time | Validation-ready to actual gate start |
| `MET-FLOW-006` | Review lead time | Review-ready exact revision to bound decision |
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
| `MET-FLOW-014` | Rework ratio: active repair/retest seconds after a review or gate rejection / total evidence-producing seconds |
| `MET-FLOW-015` | Reviewer correction rate: packets requiring a material correction / packets reviewed |
| `MET-FLOW-016` | Integration conflict count and resolution seconds per accepted package |
| `MET-FLOW-027` | Required evidence completeness: resolved required evidence / required evidence |
| `MET-AI-040` | Escaped consistency defect after trusted integration |
| `MET-AI-041` | Escaped critical identity, atomicity, security, authority, or user-data defect |
| `MET-AI-042` | False completion claim accepted without the required evidence |

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
| `MET-FLOW-022` | Minimum host `MemAvailable` | Do not start heavy work below 6 GiB |
| `MET-FLOW-023` | Maximum swap used | Do not start heavy work above 2 GiB used |
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

## Required Events

The manager records these events in the active `flow-metrics/*.json` file:

1. packet ready and lane start;
2. first trusted signal;
3. first source/report mutation;
4. each durable checkpoint;
5. candidate ready;
6. focused/broad gate start and result, including first failure;
7. review start and decision;
8. provider retry storm, reroute, resume, or deliberate hedge stop;
9. integration queue entry, conflict, and integrated focused-gate result;
10. resource sample and every host/provider/resource incident;
11. Close, Install, Dogfood, and Retrospect completion.

Active append-only logs are bound by `prefix_bytes + sha256`; later appends do not invalidate an
earlier measurement record. A mutable filename or a transcript summary alone is not evidence.

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

The north-star result remains trusted integration, not activity.
