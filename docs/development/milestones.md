# CodeFire Milestone Board

Status: live execution board

Owner: Orchestrator (Codex)

Last audited: 2026-08-22

Update trigger: M1 exit, M1 blocker admission, or slot shift.

`MS-remote-happy-path` exited after attempt 3 completed the isolated file-backed collaboration path,
a distinct read-only confirmation pass, and a fresh Grok audit. The audit authorized the exit at
integration HEAD `68a94b4`; its follow-up commit `51e3164` confirmed the docs-only manifest correction.
The active milestone is now `MS-adoption-wedge`. No adoption experiment or later-slot work has run.

This is the only live five-ahead execution board. Historical burn-down narrative stays in
`current-cycle-burn-down-plan.md` and `history.md`; those files are not permission to work.

## Active slots

| Slot | ID | Name | Exit (must all be true) | In-scope now |
|---|---|---|---|---|
| M1 | `MS-adoption-wedge` | First adoption wedge | A second repository or clean-room consumer completes one bounded inspect-or-import path without a Git bridge or agent platform; preservation and comprehension are measured; the experiment records adopt / narrow / stop | **Yes** |
| M2 | `MS-release-operational-trust` | Release and operational trust | A recorded rights/license decision for published artifacts and one independently verified release-and-rollback rehearsal; no public beta label while source gates or release identity are unresolved | No |
| M3 | `MS-product-value-proof` | First product-value proof | One candidate user segment completes a claims-comprehension and first-seal study with measured setup/friction and an explicit adopt / narrow / stop decision | No |
| M4 | `MS-ecosystem-conformance-seed` | Ecosystem interoperability seed | One maturity-labelled public profile plus extension rules is consumed by one independently authored adapter/client that passes its bounded conformance corpus | No |
| M5 | `MS-ecosystem-multi-implementation-proof` | Ecosystem multi-implementation proof | A second independent integration from a distinct organization consumes a compatibility-preserving profile update and passes the bounded conformance corpus; formal standards/governance remain roadmap work | No |

M2–M5 are bounded slices of roadmap Stages 4–6, not copies of those stages.
`strategy/modernization-roadmap.md` remains a stage/gate roadmap and must not become a second task board.

## M1 card — First adoption wedge

Objective: prove or falsify that a second repository or source-blind consumer can obtain a useful,
truthful CodeFire result with a small read-only or isolated-import surface, without turning CodeFire
into a Git bridge, telemetry service, or agent platform.

Exact M1 exit (nothing else counts):

- the second repository/consumer, exact public surface, task, falsifier, measurements, and stop rule
  are selected before any experiment mutation;
- one bounded inspect-or-import experiment completes, with before/after preservation hashes for every
  repository not explicitly admitted as an isolated target;
- the consumer identifies repository, branch, base/head, claim/proof kind, and any missing authority
  without source code or private clarification, or the failure is retained as the terminal result;
- setup time, time to first useful result, misunderstanding/wrong-action count, and maintenance cost
  are recorded with an explicit adopt / narrow / stop decision;
- an independent other-family audit verifies the evidence, preservation boundary, and absence of a
  hidden Git bridge, mutation authority, telemetry collection, or agent-orchestration dependency.

Issue inputs:

- CFR-171: admit only the smallest claim/proof descriptor slice actually required by the selected
  independent consumer;
- CFR-172: primary bounded read-only adoption/mental-model experiment input;
- CFR-173: admit only the measurement, falsifier, privacy, and terminal-decision slice required by
  this one experiment.

The two classification packets are read-only and may not implement, mutate canonical repositories,
start a broad pilot, or work on M2–M5.

M1 validation is packet-scoped. Every admitted run/fix packet must name exact commands, subjects,
hashes, and focused tests. Standing management checks:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
```

M1 non-goals:

- a Git compatibility writer/bridge, background telemetry, hosted service, or agent marketplace;
- full CFR-171 cross-surface descriptor implementation unless the selected consumer reproduces it as
  the smallest unavoidable blocker;
- broad user research, multi-repository/team pilots, release work, product-economics work, or
  ecosystem standardization;
- CFR-168/CFB-136 residuals, CFB-133, CFR-169, or CFR-170;
- mutating canonical Algorithm or treating `/workspace/project` as a general write set;
- serializing the two independent classification lanes.

## M1 execution graph (current)

```text
Critical path: parallel B1 classification/decision audit -> bounded contract fix -> one identical fresh-consumer rerun -> final audit -> M1 exit
Progress: nodes 1–5 completed; first consumer scored 7/7 but the frozen table returned `narrow` because seven relied-on fields span two undocumented/unschema'd payload families
Parallel now: node 5a Grok B1 source/schema classification + node 5b fresh Claude score/preservation/decision audit
Blocked: nodes 5c–7 wait on both lanes; no fix or rerun is admitted yet
Next legal action: dispatch both ready packets concurrently; keep thresholds and first-run evidence frozen
```

### Nodes

| # | Node | State | Owner | Waits on | Packet | Write set | Evidence |
|---|---|---|---|---|---|---|---|
| 1 | Public consumer/fixture contract — select the smallest second-repository or clean-room task and exact public commands | **completed** | Claude Opus 5 | — | `ai/task-packets/milestone-adoption-wedge-contract.md` | that packet only | `0b8d69e` (integrated as `624f28c`); frozen C1–C6, preservation, rubric, thresholds; conditional B1 |
| 2 | CFR-171..173 experiment/falsifier classification — map existing owners and decide the smallest admissible slice | **completed** | Grok 4.6 | — | `ai/task-packets/milestone-adoption-wedge-experiment.md` | that packet only | `3173a91` (integrated as `ef4aa46`); same sealed inspect topology; no product blocker/write |
| 3 | Orchestrator fan-in and experiment/fix admission | **completed** | Orchestrator | 1, 2 | `ai/task-packets/milestone-adoption-wedge-run.md` | board plus run packet | first experiment admitted without product change; B1 depends on measured result |
| 4a | Isolated fixture, controller evidence, and consumer handoff | **completed** | persistent Claude node-1 session | 3 | `ai/task-packets/milestone-adoption-wedge-run.md` | isolated roots plus packet | `420bd88` (integrated as `507dda1`); consumer-ready root and sealed external key |
| 4b | Fresh source-blind consumer comprehension run | **completed** | fresh Grok 4.6 session | 4a | handed task/docs only | consumer-root answer/evidence only | C1–C5 once, answer SHA `187812d6…`, no off-contract command |
| 4c | Score and terminal adopt/narrow/stop decision | **completed: narrow** | persistent Claude node-1 session | 4b | run packet | packet/evidence only | `2207b88` (integrated as `7f7b86e`); 7/7, Q5/Q6 correct, field reliance 7 → narrow |
| 5 | Read-only preservation and comprehension confirmation | **completed** | persistent Claude node-1 session | 4c | run packet | packet/evidence only | distinct C2–C5 pass; answer/fixture/canonical hashes frozen |
| 5a | B1 source/schema/write-set classification | **ready** | persistent Grok node-2 session | 5 | `ai/task-packets/milestone-adoption-wedge-b1.md` | that packet only | — |
| 5b | Independent first-run score/preservation/decision audit | **ready** | fresh Claude Opus 5 audit session | 5 | `ai/audit-latest.md` | audit file only | — |
| 5c | Orchestrator fan-in and bounded fix/rerun admission | **blocked** | Orchestrator | 5a, 5b | — | board plus fix/rerun packet | — |
| 5d | Bounded B1 contract fix | **blocked** | context owner selected at 5c | 5c | — | exact admitted product/docs/test set | — |
| 5e | One identical fresh-consumer rerun and decision | **blocked** | fresh other-family consumer + persistent controller | 5d | run packet | isolated evidence plus packet | — |
| 6 | Final independent audit | **blocked** | fresh other-family audit agent | 5e | `ai/audit-latest.md` | audit file only | — |
| 7 | M1 exit and slot shift | **blocked** | Orchestrator | 6 | — | board/history/current-state docs | — |

Nodes 1 and 2 are independent: consumer/public-contract selection does not depend on source ownership
or experiment-metric classification. After fan-in, keep serially dependent work with the context owner
unless audit independence requires a fresh session.

## Open issue placement

All remain `investigating`; slot movement is not a status transition.

| Issue | Slot / role | Admission rule |
|---|---|---|
| CFR-171 | M1 input | Only the smallest reproduced claim/proof descriptor blocker |
| CFR-172 | M1 input | Primary read-only adoption and mental-model experiment |
| CFR-173 | M1 input | Only bounded measurement/falsifier/privacy/decision evidence |
| CFR-168 | backlog parked | Remote M1 did not reproduce decision-work budgeting as its blocker |
| CFB-136 | backlog parked | Remote M1 did not reproduce validation-cache subject reuse as its blocker |
| CFB-133 | backlog parked | Migrate slice not reproduced by completed local milestones |
| CFR-169 | backlog parked | Architecture seam classified off the critical path |
| CFR-170 | backlog parked | Local-loop slice completed; all-command residual is not adoption permission |

The remote-M1 finding that `request-list` omits `applied_head` is parked as a future contract
candidate, not active adoption work unless node 1 independently reproduces it as the smallest blocker.

`BATCH-2026-07-review-portfolio-completion` is frozen as history. Backlog-parked residuals do not
enter an active slot without a later evidence-triggered board update.

## M2 preview (queued)

When M1 exits, M2 `MS-release-operational-trust` may select the bounded rights/license decision and
one release/verify/rollback rehearsal. It must not start early.

## Shift rule

On M1 close:

1. Record exit evidence in `history.md`.
2. Rename slots: old M2 becomes M1, old M3 becomes M2, and so on.
3. Name a new M5 from `strategy/modernization-roadmap.md` without turning that roadmap into a task board.
4. Refuse any in-flight parked work that tried to start early.
