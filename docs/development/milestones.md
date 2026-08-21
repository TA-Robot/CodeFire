# CodeFire Milestone Board

Status: live execution board

Owner: Orchestrator (Codex)

Last audited: 2026-08-21

Update trigger: M1 exit, M1 blocker admission, or slot shift.

M1 state after replay evidence commit `9ee9485`: **final independent audit ready**. The source-blind Claude session consumed structured argv/actions, resolved two real Fires, and sealed `CF-COMMIT-2792b54ad730e6d644668c38` with post-state `open-clean` / Fire 0. No `block-m1` recurred. M1 has not exited.

This is the only live five-ahead execution board. Historical burn-down narrative stays in `current-cycle-burn-down-plan.md` and `history.md`. Those files are not permission to work.

The previous milestone `MS-2026-08-control-restore` exited on 2026-08-21 and the board shifted one slot. Its exit evidence is in `history.md`; its execution record is in `ai/task-packets/milestone-control-restore.md`. Neither is evidence about the current M1.

## Active slots

| Slot | ID | Name | Exit (must all be true) | In-scope now |
|---|---|---|---|---|
| M1 | `MS-agent-local-contract` | Agent-consumable local contract | A fresh Claude or Grok worker can take one bounded local change from status through sealed commit using public help, JSON envelopes, and `next_actions`, without reading CodeFire product source or receiving private command corrections | **Yes** |
| M2 | `MS-remote-happy-path` | Remote happy path of sealed history | `upload` → `request-merge` → `request-apply` of an already sealed commit works on the documented happy path; bounded HTTP/graph work is in scope only if it blocks that path | No |
| M3 | `MS-adoption-wedge` | First adoption wedge | A second repository can inspect or import without becoming a Git bridge or an agent platform; remaining product experiments record adopt / narrow / stop | No |
| M4 | `MS-release-operational-trust` | Release and operational trust | A recorded rights/license decision for published artifacts and one independently verified release-and-rollback rehearsal; no public beta label while source gates or release identity are unresolved | No |
| M5 | `MS-product-value-proof` | First product-value proof | One candidate user segment completes a claims-comprehension and first-seal study with measured setup/friction and an explicit adopt / narrow / stop decision; broad market discovery and a four-week pilot remain in the roadmap | No |

M4 and M5 are bounded slices of roadmap Stages 4 and 5, not copies of those stages. `strategy/modernization-roadmap.md` remains a stage/gate roadmap and must not be turned into a second task board.

## M1 card — Agent-consumable local contract

Objective: remove private source knowledge and orchestrator command correction from the local sealed-loop path.

Exact M1 exit (nothing else counts):

- a fresh Claude or Grok worker receives only the public local command contract plus a bounded change packet and reaches a sealed commit;
- every transition is chosen from command help, a versioned JSON envelope, or machine-readable `next_actions` with complete required arguments;
- the worker does not read `crates/**` or the Python implementation to discover command syntax or state semantics;
- an independent other-family audit reproduces the contract path and finds no unsafe or cyclic action.

Issue inputs admitted to M1:

- CFR-170: **admitted only for the reproduced local-loop action slice** — parameterized-vs-ready execution metadata, shell-free argv/template projection, and the lost single-extinguish path binding;
- CFB-133: **not admitted** — the clean-room probe did not reproduce the stale migrate action on this path;
- CFR-169: **parked** — node 2 found that existing owners contain the fix; architecture cleanup would lengthen the critical path.

M1 validation is packet-scoped. The clean-room probe and every admitted fix packet must name the exact public commands and focused tests it uses. The standing management checks are:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
```

Workspace-wide Rust tests, full installer, full Algorithm suite, Retrospect, remote mutation, and review-portfolio discovery are out of M1 unless a later admitted fix packet proves one is the narrowest deterministic gate for its touched surface.

M1 non-goals:

- CFR-168/CFB-136 graph, HTTP, cache, or remote-scale work (M2);
- CFR-171..173 adoption/claim/product experiments (M3);
- release or product-economics work (M4/M5);
- adding agent orchestration to the CodeFire product;
- using the large classified `/workspace/project` tree as a general write set;
- implementing a hypothetical gap before the clean-room probe reproduces it.

## M1 execution graph (current)

This is the initial live plan for `MS-agent-local-contract`. It is a concrete graph, not a template.

```text
Critical path: independent other-family audit → M1 exit and slot shift
Progress: nodes 1–5 completed; node 5 sealed CF-COMMIT-2792b54ad730e6d644668c38 through public execution metadata with no private command correction
Parallel now: node 6 only — fresh Grok 4.6 audit session, not the node-4 implementing session
Blocked: node 7 exit waits on node 6
Next legal action: audit exact integration HEAD into `ai/audit-latest.md` only; do not exit M1 inside the audit
```

The prior sealed-loop transcript is the probe fixture, not evidence that the contract passes. Nodes 1 and 2 ran in parallel from exact base `4185a55` with disjoint packet writes. Node 3 admits only what their evidence requires.

### Nodes

| # | Node | State | Owner | Waits on | Packet | Write set | Evidence |
|---|---|---|---|---|---|---|---|
| 1 | Clean-room public-contract probe — replay the prior M1 transcript from public help/JSON/next-actions only and identify the first missing, unsafe, or cyclic action | **completed** | Claude Opus 5 | — | `ai/task-packets/milestone-agent-local-contract-probe.md` | that packet only | `e80f84a` (integrated as `adf343f`); CFR-170 `block-m1`; missing resolution input contract and lost extinguish→verify path; CFB-133 not reproduced |
| 2 | CFR-169 ownership-seam classification — decide whether any smallest contract fix actually requires the parked architecture seam | **completed** | Grok 4.6 | — | `ai/task-packets/milestone-agent-local-contract-seam.md` | that packet only | `bcafb9e`; CFR-169 `park`; existing automation/migration/help owners are sufficient |
| 3 | Orchestrator admission and focused fix graph | **completed** | Orchestrator | 1, 2 | this fan-in commit | board, history, CFR-170 milestone label, new fix packet | only the reproduced CFR-170 local-loop slice admitted; CFB-133 not admitted; CFR-169 parked |
| 4 | Focused contract fix and tests | **completed** | persistent Grok 4.6 node-2 session | 3 | `ai/task-packets/milestone-agent-local-contract-fix.md` | packet-fixed product/test/schema/public-doc files | `4d0f755`; 24 automation + 1 extinguish + 1 help tests passed; Rust manifest `95ee643b…` |
| 5 | Fresh clean-room sealed-loop replay | **completed** | persistent Claude Opus 5 source-blind session, not the fix session | 4 + source-matching runner | `ai/task-packets/milestone-agent-local-contract-replay.md` | two exact clean dogfood files, CodeFire state, that packet | `9ee9485`; test 3 pass; Fires `FIRE-26974231BF82` / `FIRE-F530E483B5AF`; sealed `CF-COMMIT-2792b54ad730e6d644668c38`; post `open-clean`, Fire 0; no block-m1 |
| 6 | Final independent audit | **ready** | fresh Grok 4.6 audit session, not node 4 | 5 | `ai/audit-latest.md` | audit file only | audit exact integration HEAD; rederive runner, action, sealed-state, and preservation evidence |
| 7 | M1 exit and slot shift | **blocked** | Orchestrator | 6 | written after audit | board, history, packet, current-state docs | — |

Node 4 stayed with the Grok session that already mapped the relevant owners; Claude's source-blind session independently completed node 5. Node 6 deliberately requires a new Grok session: continuity is valuable across dependent implementation work, but reusing the implementing session would invalidate the final audit.

## Open issue placement

These remain `investigating`; the slot shift does not mark any issue fixed.

| Issue | Slot / role | Admission rule |
|---|---|---|
| CFB-133 | M1 input, not admitted | Probe did not reproduce migrate drift; keep open but do not implement |
| CFR-169 | M1 input, parked | Existing owners contain the fix; no architecture extraction in M1 |
| CFR-170 | M1 admitted slice | Only the reproduced local-loop action execution/path contract; the all-command matrix remains later issue work |
| CFR-168 | M2 parked | Graph/object/HTTP work budget belongs to the remote happy path |
| CFB-136 | M2 parked | Validation-cache reuse is the admitted CFR-168 blocker |
| CFR-171 | M3 parked | Machine-readable claim/proof descriptor for external consumers |
| CFR-172 | M3 parked | Read-only adoption experiment |
| CFR-173 | M3 parked | Product falsification / stop-rule experiment |

`BATCH-2026-07-review-portfolio-completion` is **frozen as history**. Only the three M1 inputs above may be classified now; M2–M3 roots stay parked.

## M2 preview (queued)

When M1 exits, M2 `MS-remote-happy-path` may admit CFR-168 and CFB-136 only to the extent required for one sealed `upload → request-merge → request-apply` happy path. It must not start early.

## Shift rule

On M1 close:

1. Record exit evidence in `history.md`.
2. Rename slots: old M2 becomes M1, old M3 becomes M2, and so on.
3. Name a new M5 from `strategy/modernization-roadmap.md` without turning that roadmap into a task board.
4. Refuse any in-flight parked work that tried to start early.
