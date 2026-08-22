# CodeFire Milestone Board

Status: live execution board

Owner: Orchestrator (Codex)

Last audited: 2026-08-22

Update trigger: M1 exit, M1 blocker admission, or slot shift.

`MS-adoption-wedge` exited after the source-blind post-B1 rerun scored 7/7 with zero undocumented
field reliance and the fresh Grok node-6 audit returned `ACCEPT` with no `block-m1`. The active
milestone is now `MS-release-operational-trust`. No release rehearsal or later-slot work has run.

This is the only live five-ahead execution board. Historical burn-down narrative stays in
`current-cycle-burn-down-plan.md` and `history.md`; those files are not permission to work.

## Active slots

| Slot | ID | Name | Exit (must all be true) | In-scope now |
|---|---|---|---|---|
| M1 | `MS-release-operational-trust` | Release and operational trust | A recorded rights/license decision for rehearsed artifacts and one independently verified release-and-rollback rehearsal; no public beta label while source gates or release identity are unresolved | **Yes** |
| M2 | `MS-product-value-proof` | First product-value proof | One candidate user segment completes a claims-comprehension and first-seal study with measured setup/friction and an explicit adopt / narrow / stop decision | No |
| M3 | `MS-ecosystem-conformance-seed` | Ecosystem interoperability seed | One maturity-labelled public profile plus extension rules is consumed by one independently authored adapter/client that passes its bounded conformance corpus | No |
| M4 | `MS-ecosystem-multi-implementation-proof` | Ecosystem multi-implementation proof | A second independent integration from a distinct organization consumes a compatibility-preserving profile update and passes the bounded conformance corpus; formal standards/governance remain roadmap work | No |
| M5 | `MS-ecosystem-third-organization-proof` | Ecosystem third-organization proof | A third distinct organization consumes the stable public profile without a compatibility exception and passes the bounded conformance corpus; release-cycle and formal-governance gates remain roadmap work | No |

M1–M5 are bounded slices of roadmap Stages 4–6, not copies of those stages.
`strategy/modernization-roadmap.md` remains a stage/gate roadmap and must not become a second task board.

## M1 card — Release and operational trust

Objective: prove or falsify that one exact CodeFire artifact set can be identified, locally published,
independently verified, installed into an isolated prefix, and rolled back without pretending the
project already has public-release rights, SemVer, signing, or operational maturity it has not earned.

Exact M1 exit (nothing else counts):

- the rehearsed Rust/Python/docs artifacts have an explicit rights/license and source-identity decision;
  a missing right remains a public-distribution blocker rather than being inferred from repository access;
- one exact source revision, version-axis statement, artifact manifest, checksum/provenance set, local
  publication target, verification procedure, isolated install target, and rollback target are fixed
  before the rehearsal mutates its isolated release area;
- a separate operator follows only the recorded release/verify/rollback handoff, verifies identity and
  checksums, exercises the bounded artifact, and restores the declared prior state with before/after hashes;
- the result records publishable / internal-only / stop, unresolved signing/SBOM/support/security limits,
  and explicitly refuses a public beta label while source or release identity is unresolved;
- an independent other-family audit verifies the rights record, source binding, artifact closure,
  preservation boundary, operator independence, rollback result, and absence of hidden external publication.

Issue inputs:

- no open CFB/CFR root is pre-admitted;
- a root may enter only if one of the two read-only classifiers reproduces it as the smallest blocker
  of the exact rights decision or isolated release/verify/rollback path.

The two classification packets are read-only and may not publish externally, build or install, change
license text, mutate canonical repositories, or work on M2–M5.

M1 validation is packet-scoped. Every admitted run/fix packet must name exact commands, subjects,
hashes, and focused tests. Standing management checks:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
```

M1 non-goals:

- a public GitHub/PyPI/crates.io release, public beta claim, external upload, or credential use;
- inventing legal permission, changing copyright/license ownership, or treating an audit as legal advice;
- completing every Stage 4 policy, production signing, two-person incident response, or hosted service;
- broad product research, ecosystem standardization, CFR-168..173 residual implementation, or new issue discovery;
- mutating canonical Algorithm or the user's current install, or treating `/workspace/project` as a general write set;
- serializing the two independent classification lanes.

## M1 execution graph (current)

```text
Critical path: independent operator verification/install/rollback -> final audit -> M1 exit
Progress: nodes 1–5a completed; fresh source-bound candidate is atomically published under the isolated release root with internal-only rights fields
Parallel now: none — node 5b is the independent fresh-operator lane
Blocked: nodes 6–7 wait on operator verification and rollback evidence
Next legal action: execute `milestone-release-operational-trust-run.md` Stage B once; no rebuild, external publication, or private correction
```

### Nodes

| # | Node | State | Owner | Waits on | Packet | Write set | Evidence |
|---|---|---|---|---|---|---|---|
| 1 | Rights/license and artifact-source decision — classify what may be rehearsed or published and what must remain blocked | **completed** | Claude Opus 5 | — | `ai/task-packets/milestone-release-operational-trust-rights.md` | that packet only | `131be1b` (integrated as `4a2cad4`); internal-only, nothing publishable, no product blocker |
| 2 | Release/verify/rollback topology — derive the smallest isolated artifact and independent-operator procedure | **completed** | Grok 4.6 | — | `ai/task-packets/milestone-release-operational-trust-rehearsal.md` | that packet only | `cfafb5f` (integrated as `b6adfc9`); fresh build required, local atomic pointer/isolated rollback, no product blocker |
| 3 | Orchestrator fan-in and bounded rehearsal/fix admission | **completed** | Orchestrator | 1, 2 | `ai/task-packets/milestone-release-operational-trust-run.md` | board plus run packet | internal-only exact artifact set admitted; internal strategy doc removed; no node-4 fix |
| 4 | Smallest reproduced release-contract fix, if any | **not required** | — | 3 | — | — | both lanes found none; no source/package/licence edit admitted |
| 5a | Source-bound artifact build and isolated local publication | **completed** | persistent Grok node-2 session | 3 | `ai/task-packets/milestone-release-operational-trust-run.md` | isolated release root plus run packet | `76975ea` (integrated as `34ec8f5`); source `01a4de0`, provenance `7998bcf6…`, atomic current pointer, internal-only candidate root `/tmp/codefire-m1-release.vT1Uzs` |
| 5b | Fresh operator verification, isolated install/smoke, and rollback | **ready** | fresh Claude Opus 5 operator | 5a | `ai/task-packets/milestone-release-operational-trust-run.md` | handed release root/evidence plus packet only | one pass; prefix/prior start empty |
| 6 | Final independent audit | **blocked** | fresh other-family audit agent | 5b | `ai/audit-latest.md` | audit file only | — |
| 7 | M1 exit and slot shift | **blocked** | Orchestrator | 6 | — | board/history/current-state docs | — |

Nodes 1 and 2 are independent: legal/source-identity evidence does not depend on the mechanical
release/rollback topology. After fan-in, keep serially dependent work with the context owner unless
operator or audit independence requires a fresh session.

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

The adoption audit also parked `open-clean` versus `opened=0`, historical `claim.subject.kind=live_open`,
and the remote `request-list.applied_head` omission. None is release-M1 permission.

`BATCH-2026-07-review-portfolio-completion` is frozen as history. Backlog-parked residuals do not
enter an active slot without a later evidence-triggered board update.

## M2 preview (queued)

When M1 exits, M2 `MS-product-value-proof` may select one candidate segment and one bounded
claims-comprehension/first-seal study. It must not start early.

## Shift rule

On M1 close:

1. Record exit evidence in `history.md`.
2. Rename slots: old M2 becomes M1, old M3 becomes M2, and so on.
3. Name a new M5 from `strategy/modernization-roadmap.md` without turning that roadmap into a task board.
4. Refuse any in-flight parked work that tried to start early.
