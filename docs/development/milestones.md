# CodeFire Milestone Board

Status: live execution board

Owner: Orchestrator (Codex)

Last audited: 2026-08-22

Update trigger: M1 exit, M1 blocker admission, or slot shift.

`MS-release-operational-trust` exited after a fresh source-bound internal-only candidate was atomically
published, independently verified/installed/smoked/rolled back, and the fresh Grok node-6 audit
returned `ACCEPT` with no `block-m1`. The active milestone is now `MS-product-value-proof`. No
product-value study or later-slot work has run.

This is the only live five-ahead execution board. Historical burn-down narrative stays in
`current-cycle-burn-down-plan.md` and `history.md`; those files are not permission to work.

## Active slots

| Slot | ID | Name | Exit (must all be true) | In-scope now |
|---|---|---|---|---|
| M1 | `MS-product-value-proof` | First product-value proof | One candidate user segment completes a claims-comprehension and first-seal study with measured setup/friction and an explicit adopt / narrow / stop decision | **Yes** |
| M2 | `MS-ecosystem-conformance-seed` | Ecosystem interoperability seed | One maturity-labelled public profile plus extension rules is consumed by one independently authored adapter/client that passes its bounded conformance corpus | No |
| M3 | `MS-ecosystem-multi-implementation-proof` | Ecosystem multi-implementation proof | A second independent integration from a distinct organization consumes a compatibility-preserving profile update and passes the bounded conformance corpus; formal standards/governance remain roadmap work | No |
| M4 | `MS-ecosystem-third-organization-proof` | Ecosystem third-organization proof | A third distinct organization consumes the stable public profile without a compatibility exception and passes the bounded conformance corpus; release-cycle and formal-governance gates remain roadmap work | No |
| M5 | `MS-ecosystem-compatible-release-cycles` | Ecosystem compatible release cycles | Two successive compatibility-preserving profile releases pass the bounded conformance corpus without an emergency exception; funded-maintainer and formal-governance gates remain roadmap work | No |

M1–M5 are bounded slices of roadmap Stages 4–6, not copies of those stages.
`strategy/modernization-roadmap.md` remains a stage/gate roadmap and must not become a second task board.

## M1 card — First product-value proof

Objective: prove or falsify that one concrete CodeFire user segment can understand the product's
claims and complete one useful first-seal workflow with less setup, confusion, and wrong action than
the value of the verified result warrants. Do not turn one study into broad market validation.

Exact M1 exit (nothing else counts):

- one candidate segment, exact first-seal task, claim-comprehension questions, value hypothesis,
  falsifier, measurements, and adopt / narrow / stop thresholds are frozen before the study setup;
- one source-blind participant completes or terminally fails an isolated first-seal path using only the
  admitted runner, public docs/help/JSON actions, and the handed fixture; no product source or private correction;
- the participant identifies what the seal proves, what it does not prove, the base/head and local versus
  portable authority, and chooses no mutation beyond the handed task; canonical/user repositories are preserved;
- setup time, time to first useful result and first seal, command/help/docs load, misunderstandings,
  wrong actions, support interventions, task value, and maintenance/friction cost are recorded honestly;
- an independent other-family audit verifies the frozen thresholds, evidence, preservation, source-blind
  boundary, terminal decision, and absence of hidden telemetry, Git bridge, hosted service, or agent-platform work.

Issue inputs:

- no open CFB/CFR root is pre-admitted;
- a root may enter only if one of the two read-only classifiers or the frozen study reproduces it as
  the smallest blocker of the exact first-seal/value path.

The two classification packets are read-only and may not run a participant, implement, mutate any
repository, collect telemetry, contact external users, or work on M2–M5.

M1 validation is packet-scoped. Every admitted run/fix packet must name exact commands, subjects,
hashes, and focused tests. Standing management checks:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
```

M1 non-goals:

- broad interviews, external recruitment, a three-repository pilot, a four-week team pilot, or a market-size claim;
- Git bridge, hosted service, telemetry collection, MCP/SARIF/LSP promotion, or agent-platform code;
- public release, licence resolution, ecosystem standardization, or later-slot adapter work;
- CFR-168..173 residual implementation unless the exact study reproduces the smallest unavoidable blocker;
- mutating canonical Algorithm, the user's install, or any non-isolated repository;
- changing thresholds after participant output or serializing the two independent classification lanes.

## M1 execution graph (current)

```text
Critical path: controller baseline/setup -> source-blind participant first seal -> score/confirmation -> final audit -> M1 exit
Progress: nodes 1–3 completed; both classifiers found no product blocker, and fan-in moved import to controller setup so the participant's first seal has a retrievable pre-change base
Parallel now: none — node 5a is the persistent Grok controller/setup lane
Blocked: nodes 5b–7 wait on a frozen participant-ready handoff
Next legal action: execute `milestone-product-value-proof-run.md` Stage A; no participant, product fix, or threshold change yet
```

### Nodes

| # | Node | State | Owner | Waits on | Packet | Write set | Evidence |
|---|---|---|---|---|---|---|---|
| 1 | Segment, claim-comprehension, value hypothesis, and frozen decision contract | **completed** | Claude Opus 5 | — | `ai/task-packets/milestone-product-value-proof-segment.md` | that packet only | `80899a3` (integrated as `aa95b98`); AI-assisted solo maintainer, Q1–Q7, 16 metrics, frozen ordered table, no blocker |
| 2 | Isolated first-seal fixture, command path, evidence, and friction classification | **completed** | Grok 4.6 | — | `ai/task-packets/milestone-product-value-proof-first-seal.md` | that packet only | `ca02b27` (integrated as `d968f52`); receipt-digest, public live loop, roots/preservation/falsifiers, no blocker |
| 3 | Orchestrator fan-in and bounded study/fix admission | **completed** | Orchestrator | 1, 2 | `ai/task-packets/milestone-product-value-proof-run.md` | board plus run packet | controller-sealed uppercase base + participant lowercase change; no node-4 fix |
| 4 | Smallest reproduced product-contract fix, if any | **not required** | — | 3 | — | — | import action gap parked in controller setup; no source/docs change admitted |
| 5a | Controller setup and source-blind participant handoff | **ready** | persistent Grok node-2 session | 3 | `ai/task-packets/milestone-product-value-proof-run.md` | isolated fixture/key/evidence plus run packet | — |
| 5b | Fresh participant first-seal run, score, and distinct confirmation | **blocked** | fresh other-family participant + persistent controller | 5a | — | handed root/evidence plus packet only | — |
| 6 | Final independent audit | **blocked** | fresh other-family audit agent | 5b | `ai/audit-latest.md` | audit file only | — |
| 7 | M1 exit and slot shift | **blocked** | Orchestrator | 6 | — | board/history/current-state docs | — |

Nodes 1 and 2 are independent: segment/value/falsifier selection does not depend on fixture/source
ownership. After fan-in, keep serially dependent work with the context owner unless participant or
audit independence requires a fresh session.

## Open issue placement

All remain `investigating`; slot movement is not a status transition.

| Issue | Slot / role | Admission rule |
|---|---|---|
| CFR-171 | backlog parked | Bounded branch/show data-schema slice measured sufficient; full descriptor unadmitted |
| CFR-172 | backlog parked | Read-only sealed-inspect experiment ended terminal `adopt`; broader adapter work unadmitted |
| CFR-173 | backlog parked | Bounded metric/falsifier slice satisfied; cohort, telemetry, and pilot residuals unadmitted |
| CFR-168 | backlog parked | Remote M1 did not reproduce decision-work budgeting as its blocker |
| CFB-136 | backlog parked | Remote M1 did not reproduce validation-cache subject reuse as its blocker |
| CFB-133 | backlog parked | Migrate slice not reproduced by completed local milestones |
| CFR-169 | backlog parked | Architecture seam classified off the critical path |
| CFR-170 | backlog parked | Local-loop slice completed; all-command residual is not adoption permission |

The adoption audit parked `open-clean` versus `opened=0`, historical `claim.subject.kind=live_open`,
and the remote `request-list.applied_head` omission. Release audit parked its legacy directory-digest
method gap. None is product-M1 permission without reproduction on the selected study.

`BATCH-2026-07-review-portfolio-completion` is frozen as history. Backlog-parked residuals do not
enter an active slot without a later evidence-triggered board update.

## M2 preview (queued)

When M1 exits, M2 `MS-ecosystem-conformance-seed` may select one maturity-labelled public profile,
extension rules, and one independently authored adapter/client. It must not start early.

## Shift rule

On M1 close:

1. Record exit evidence in `history.md`.
2. Rename slots: old M2 becomes M1, old M3 becomes M2, and so on.
3. Name a new M5 from `strategy/modernization-roadmap.md` without turning that roadmap into a task board.
4. Refuse any in-flight parked work that tried to start early.
