# CodeFire Milestone Board

Status: live execution board

Owner: Orchestrator (Codex)

Last audited: 2026-08-21

Update trigger: M1 exit, M1 blocker admission, or slot shift.

M1 state after the 2026-08-21 slot shift: **planning packets ready**. `MS-local-sealed-loop` exited on the independent audit of exact HEAD `e7e792f`; its sealed identity is `CF-COMMIT-373468dbaf827a6cc2bae9a4`. The active milestone is now `MS-agent-local-contract`. No product fix has started in this new M1.

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

- CFR-170: typed executable outcomes and postcondition-driven actions — primary contract root;
- CFB-133: admit only the stale `migrate apply` postcondition/action slice if the clean-room probe reproduces it;
- CFR-169: admit only a module/test ownership seam that the smallest contract fix cannot avoid. Architecture cleanup by itself is not work permission.

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
Critical path: clean-room public-contract probe → orchestrator admission of the smallest reproduced blocker → focused fix → clean-room sealed-loop replay → independent audit → M1 exit
Parallel now: node 1 clean-room contract probe; node 2 CFR-169 ownership-seam classification. Both are read-only and have disjoint packet write sets
Blocked: node 3 admission waits on 1 and 2; later implementation/replay/audit nodes wait on node 3
Next legal action: launch both ready read-only packets; do not implement before fan-in
```

Nothing on this new graph has run. The prior sealed-loop evidence is an input fixture, not evidence that the agent-local contract already passes.

### Nodes

| # | Node | State | Owner | Waits on | Packet | Write set | Evidence |
|---|---|---|---|---|---|---|---|
| 1 | Clean-room public-contract probe — replay the prior M1 transcript from public help/JSON/next-actions only and identify the first missing, unsafe, or cyclic action | **ready** | Claude Opus 5 or Grok 4.6 | — | `ai/task-packets/milestone-agent-local-contract-probe.md` | that packet only | — |
| 2 | CFR-169 ownership-seam classification — decide whether any smallest contract fix actually requires the parked architecture seam | **ready** | the other provider family | — | `ai/task-packets/milestone-agent-local-contract-seam.md` | that packet only | — |
| 3 | Orchestrator admission and focused fix graph | **blocked** | Orchestrator | 1, 2 | written after fan-in | board and new fix packets only | — |
| 4 | Focused contract fix and tests | **blocked** | Individual agent | 3 | — | fixed after admission | — |
| 5 | Fresh clean-room sealed-loop replay | **blocked** | Individual agent, not the fix session | 4 | — | one bounded dogfood change plus CodeFire state | — |
| 6 | Final independent audit and M1 exit | **blocked** | Audit agent then orchestrator | 5 | — | audit file, board/history/current-state docs | — |

The two ready packets are deliberately read-only. Their fan-in decides whether CFR-170, CFB-133, and/or the narrowest CFR-169 seam is necessary. Filing or anticipating a gap is not permission to implement it.

## Open issue placement

These remain `investigating`; the slot shift does not mark any issue fixed.

| Issue | Slot / role | Admission rule |
|---|---|---|
| CFB-133 | M1 input | Admit only if the clean-room probe reproduces the stale postcondition/action loop |
| CFR-169 | M1 input | Classification only until a smallest contract fix proves an ownership seam necessary |
| CFR-170 | M1 input | Primary typed executable outcome/action root |
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
