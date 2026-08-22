# CodeFire Milestone Board

Status: live execution board

Owner: Orchestrator (Codex)

Last audited: 2026-08-22

Update trigger: M1 exit, M1 blocker admission, or slot shift.

`MS-product-value-proof` exited with literal `narrow`. One source-blind clean-room AI-assisted
maintainer completed the isolated first-seal study on participant base `CF-COMMIT-3456b0494fd47761a0522cac`,
sealed head `CF-COMMIT-d6bb8b1d159ad489dc67ec92`, scored Q1–Q7 7/7 with task value 3/3, and the fresh
final Grok node-6 audit `218ef8668d96d5e011f62586f54712d69da68c79` returned `ACCEPT` with no
`block-m1`. The frozen ordered decision table first-matched `narrow` on undocumented JSON identifier
reliance — the fresh auditor independently counted 6 where the controller recorded 4 — so the only
admitted consequence is to narrow the public claim. No retry and no product, README, docs, schema, or
threshold fix was performed, because the frozen table required `narrow`. The active milestone is now
`MS-ecosystem-conformance-seed`. No ecosystem work has run.

Standing product-value limitations carried forward: n=1 clean-room AI-assisted maintainer/operator
only, which is not human adoption and not market validation; timings are machine timings, with time
to first successful seal 402s; one genuine exit-10 refusal occurred and its six Fires were
subsequently cleared; no wrong action, off-task mutation, or private correction was observed; the study
fixture's own unconfigured `README.md`, outside that fixture's configured globs, can contradict the
sealed requirement without the first-seal loop noticing; and the handed public docs pack does not
enumerate every JSON identifier the envelopes return.

This is the only live five-ahead execution board. Historical burn-down narrative stays in
`current-cycle-burn-down-plan.md` and `history.md`; those files are not permission to work.

## Active slots

| Slot | ID | Name | Exit (must all be true) | In-scope now |
|---|---|---|---|---|
| M1 | `MS-ecosystem-conformance-seed` | Ecosystem interoperability seed | One maturity-labelled public profile plus extension rules is consumed by one independently authored adapter/client that passes its bounded conformance corpus | **Yes** |
| M2 | `MS-ecosystem-multi-implementation-proof` | Ecosystem multi-implementation proof | A second independent integration from a distinct organization consumes a compatibility-preserving profile update and passes the bounded conformance corpus; formal standards/governance remain roadmap work | No |
| M3 | `MS-ecosystem-third-organization-proof` | Ecosystem third-organization proof | A third distinct organization consumes the stable public profile without a compatibility exception and passes the bounded conformance corpus; release-cycle and formal-governance gates remain roadmap work | No |
| M4 | `MS-ecosystem-compatible-release-cycles` | Ecosystem compatible release cycles | Two successive compatibility-preserving profile releases pass the bounded conformance corpus without an emergency exception; funded-maintainer and formal-governance gates remain roadmap work | No |
| M5 | `MS-ecosystem-funded-maintainer-capacity` | Ecosystem funded-maintainer capacity | A named funded maintainer rotation independently handles one compatibility review and bounded conformance release rehearsal; formal standards-body activity remains roadmap work | No |

M1–M5 are now five successive bounded slices of roadmap Stage 6, not copies of that stage.
`strategy/modernization-roadmap.md` remains a stage/gate roadmap and must not become a second task board.

## M1 card — Ecosystem interoperability seed

Objective: prove or falsify that one bounded, maturity-labelled CodeFire public profile is precise
enough for an independently authored client to consume without product source, private clarification,
or a compatibility exception. Earn interoperability before governance or standardization claims.

Exact M1 exit (nothing else counts):

- one existing public profile, its maturity label, extension namespace/critical-extension behavior,
  version/unknown behavior, exact schemas/vectors, and bounded corpus are frozen before client work;
- one independently authored source-blind adapter/client consumes only the frozen profile and passes
  positive, negative, unknown-version, and critical/noncritical-extension cases without private clarification;
- the client preserves DENY/UNKNOWN versus ALLOW/CLEAN semantics, records unsupported capabilities,
  and does not obtain mutation authority or depend on a Git bridge, telemetry, hosted service, or agent platform;
- authoring time, docs/schema reliance, ambiguity, exception count, conformance results, and maintenance
  surface are measured with an explicit adopt / narrow / stop decision;
- an independent other-family audit verifies corpus/provenance, client independence, preservation,
  compatibility claims, and the absence of formal-standard/governance or multi-organization overclaim.

Issue inputs:

- no open CFB/CFR root is pre-admitted;
- a root may enter only if one classifier or the frozen client run reproduces it as the smallest
  blocker of the exact profile/corpus/client path.

The two classification packets are read-only and may not author the client, change schemas/product,
mutate a repository, claim an external organization, or work on M2–M5.

M1 validation is packet-scoped. Every admitted run/fix packet must name exact commands, subjects,
hashes, and focused tests. Standing management checks:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
```

M1 non-goals:

- a second implementation/organization, third-organization claim, compatible release cycles, or funded capacity;
- formal standards body, IPR/trademark/governance process, public beta, or external registry publication;
- broad schema redesign, all-command descriptor, Git bridge, telemetry, hosted service, or agent platform;
- CFR-168..173 residual implementation unless the exact client reproduces the smallest unavoidable blocker;
- canonical Algorithm or user-install mutation, external network dependence, or hidden private corpus;
- changing the profile/corpus/thresholds after client output or serializing independent classification lanes.

## M1 execution graph (current)

```text
Critical path: parallel profile/corpus and client-topology classification -> orchestrator fan-in -> frozen independent-client run -> confirmation -> final audit -> M1 exit
Progress: product-value M1 closed with terminal `narrow`; ecosystem nodes 1–2 are ready and no client/profile work has run
Parallel now: node 1 public profile/maturity/extensions classification + node 2 independent client/corpus/falsifier topology
Blocked: nodes 3–7 wait on both read-only classifiers
Next legal action: dispatch both ready packets concurrently; do not author a client or change schemas before node 3 admission
```

### Nodes

| # | Node | State | Owner | Waits on | Packet | Write set | Evidence |
|---|---|---|---|---|---|---|---|
| 1 | Public profile, maturity label, version/extension rules, and bounded corpus classification | **ready** | Claude Opus 5 | — | `ai/task-packets/milestone-ecosystem-conformance-profile.md` | that packet only | — |
| 2 | Independent client topology, source-blind handoff, falsifiers, and conformance measurements | **ready** | Grok 4.6 | — | `ai/task-packets/milestone-ecosystem-conformance-client.md` | that packet only | — |
| 3 | Orchestrator fan-in and bounded profile/client/fix admission | **blocked** | Orchestrator | 1, 2 | — | board plus admitted run/fix packet | — |
| 4 | Smallest reproduced conformance-contract fix, if any | **blocked** | persistent source-context owner | 3 | — | exact admitted source/docs/test set | — |
| 5a | Frozen corpus/client handoff | **blocked** | persistent topology owner | 3, optional 4 | — | isolated corpus/key/evidence plus run packet | — |
| 5b | Fresh independent client authoring/run and distinct confirmation | **blocked** | fresh other-family client author + persistent controller | 5a | — | handed root/evidence plus packet only | — |
| 6 | Final independent audit | **blocked** | fresh other-family audit agent | 5b | `ai/audit-latest.md` | audit file only | — |
| 7 | M1 exit and slot shift | **blocked** | Orchestrator | 6 | — | board/history/current-state docs | — |

Nodes 1 and 2 are independent: profile/maturity/extension selection does not depend on client
implementation topology. After fan-in, keep serially dependent work with the context owner unless
client-author or audit independence requires a fresh session.

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

Product-value M1 added two parked facts: six cited JSON keys were absent as identifiers from its handed
public pack (controller recorded four of them), and an unconfigured README could contradict the sealed
requirement. The historical adoption milestone still ended terminal `adopt`, and the release milestone
still ended internal-only `ACCEPT`; neither is restated by this close. Earlier adoption/release parks
remain. None is ecosystem-M1 permission without reproduction on the selected profile/client.

`BATCH-2026-07-review-portfolio-completion` is frozen as history. Backlog-parked residuals do not
enter an active slot without a later evidence-triggered board update.

## M2 preview (queued)

When M1 exits, M2 `MS-ecosystem-multi-implementation-proof` may select one second independent
integration and one compatibility-preserving profile update. It must not start early.

## Shift rule

On M1 close:

1. Record exit evidence in `history.md`.
2. Rename slots: old M2 becomes M1, old M3 becomes M2, and so on.
3. Name a new M5 from `strategy/modernization-roadmap.md` without turning that roadmap into a task board.
4. Refuse any in-flight parked work that tried to start early.
