# CodeFire Milestone Board

Status: live execution board

Owner: Orchestrator (Codex)

Last audited: 2026-08-21

Update trigger: M1 exit, M1 blocker admission, or slot shift.

M1 state after the 2026-08-21 slot shift: `MS-agent-local-contract` exited after the independent Grok audit of exact HEAD `b9ea932`; its product evidence is sealed `CF-COMMIT-2792b54ad730e6d644668c38`. The active milestone is now `MS-remote-happy-path`. The first isolated-run attempt stopped before every mutator because the copied `opened/main.json` retained canonical absolute paths. Canonical state was preserved, no remote project was created, and a harness-only retry is admitted with `opened/` and `active/` empty in the copy.

This is the only live five-ahead execution board. Historical burn-down narrative stays in `current-cycle-burn-down-plan.md` and `history.md`. Those files are not permission to work.

## Active slots

| Slot | ID | Name | Exit (must all be true) | In-scope now |
|---|---|---|---|---|
| M1 | `MS-remote-happy-path` | Remote happy path of sealed history | Already sealed history completes documented `upload -> request-merge -> request-review(approve) -> request-apply` against an isolated file-backed remote; CodeFire reports the applied remote head and a read-only replay confirms it; only the smallest reproduced CFR-168/CFB-136 blocker may be fixed | **Yes** |
| M2 | `MS-adoption-wedge` | First adoption wedge | A second repository can inspect or import without becoming a Git bridge or an agent platform; remaining product experiments record adopt / narrow / stop | No |
| M3 | `MS-release-operational-trust` | Release and operational trust | A recorded rights/license decision for published artifacts and one independently verified release-and-rollback rehearsal; no public beta label while source gates or release identity are unresolved | No |
| M4 | `MS-product-value-proof` | First product-value proof | One candidate user segment completes a claims-comprehension and first-seal study with measured setup/friction and an explicit adopt / narrow / stop decision | No |
| M5 | `MS-ecosystem-conformance-seed` | Ecosystem interoperability seed | One maturity-labelled public profile plus extension rules is consumed by one independently authored adapter/client that passes its bounded conformance corpus; formal standardization and multi-organization targets remain in the roadmap | No |

M3–M5 are bounded slices of roadmap Stages 4–6, not copies of those stages. `strategy/modernization-roadmap.md` remains a stage/gate roadmap and must not become a second task board.

## M1 card — Remote happy path of sealed history

Objective: prove that CodeFire can move one already sealed history item through its documented file-backed collaboration path without admitting the full remote/scale backlog.

Exact M1 exit (nothing else counts):

- an isolated fixture and exact public commands are selected before any remote mutation;
- already sealed local history completes `upload -> request-merge -> request-review(approve) -> request-apply` on the file-backed happy path;
- CodeFire's JSON envelopes identify the remote source/target/MR/applied head, and a separate read-only query confirms the terminal head/state;
- an independent other-family audit verifies object closure, no partial remote mutation, preservation of canonical dogfood state, and no hidden retry/action loop.

Issue inputs:

- CFR-168: primary remote/graph/object decision-work root, admitted only if the bounded happy path actually hits it;
- CFB-136: admit only if command-scoped validation reuse is required by the exact path and the existing subject/final-revalidation contract fails a focused probe.

The classification packets may inspect existing source, tests, public docs, and historical evidence read-only. They may not run upload/request mutations, implement, or start a scale campaign.

M1 validation is packet-scoped. Every admitted run/fix packet must name its exact public commands and focused tests. Standing management checks:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
```

M1 non-goals:

- HTTP transport, large graph/scale completion, all-command budget coverage, full installer, or Retrospect unless the exact file-backed path proves one is its smallest blocker;
- CFR-171..173 adoption/product work (M2);
- release, product-economics, or ecosystem work (M3–M5);
- reopening CFB-133, CFR-169, or CFR-170 residual work;
- mutating the canonical Algorithm project or treating `/workspace/project` as a general write set;
- serializing the two independent classification lanes.

## M1 execution graph (current)

```text
Critical path: corrected isolated file-backed remote run -> read-only confirmation -> independent audit -> M1 exit
Progress: nodes 1–3 completed; node 4 attempt 1 stopped safely at fixture admission and proved that the original whole-copy recipe was not isolated
Parallel now: node 4 retry only — continue the same Claude public-fixture session to retain topology, command, and preservation context
Blocked: nodes 5–7 wait on a successful node 4 retry
Next legal action: retry `milestone-remote-happy-path-run.md` with empty copied `opened/` and `active/`; do not implement product code
```

Nothing on this new M1 graph has run. The prior local sealed commit is an input candidate, not remote-path evidence.

### Nodes

| # | Node | State | Owner | Waits on | Packet | Write set | Evidence |
|---|---|---|---|---|---|---|---|
| 1 | Public remote fixture/command probe — derive the smallest isolated file-backed fixture and stop at the first missing contract | **completed** | Claude Opus 5 | — | `ai/task-packets/milestone-remote-happy-path-probe.md` | that packet only | `3504bd8` (integrated as `da3e691`); safe closed-branch copy plan; mandatory review gate; observed-field evidence required; CFR-168/CFB-136 park |
| 2 | CFR-168/CFB-136 residual classification — map current implemented state and decide which, if any, smallest slice lies on the exact happy path | **completed** | Grok 4.6 | — | `ai/task-packets/milestone-remote-happy-path-budget.md` | that packet only | `b8b0a1b`; both park; file-backed owners already exist; no product write unavoidable |
| 3 | Orchestrator fan-in, fixture admission, and run/fix graph | **completed** | Orchestrator | 1, 2 | this commit | board, history, run packet | four-command sequence admitted; no issue/product fix admitted |
| 4 | Isolated remote happy-path run | **ready (retry)** | persistent Claude node-1 session | 3 | `ai/task-packets/milestone-remote-happy-path-run.md` | isolated temp fixture/remote plus that packet | attempt 1 `3ce12d1` (integrated as `8f45917`) stopped before mutation; corrected harness admitted |
| 5 | Read-only terminal confirmation and evidence record | **blocked** | Individual agent, persistent if serially dependent | 4 | — | packet/evidence only | — |
| 6 | Final independent audit | **blocked** | Other-family audit agent | 5 | `ai/audit-latest.md` | audit file only | — |
| 7 | M1 exit and slot shift | **blocked** | Orchestrator | 6 | — | board/history/current-state docs | — |

Nodes 1 and 2 are parallel because fixture/public-contract selection does not depend on source ownership classification. After fan-in, keep serially dependent work with the agent that already owns the relevant context unless audit independence requires a fresh session.

## Open issue placement

All remain `investigating`; slot movement is not a status transition.

| Issue | Slot / role | Admission rule |
|---|---|---|
| CFR-168 | M1 input | Admit only the smallest reproduced blocker of the exact file-backed happy path |
| CFB-136 | M1 input | Admit only if validation-session/cache subject behavior blocks that path |
| CFR-171 | M2 parked | Machine-readable claim/proof descriptor for external consumers |
| CFR-172 | M2 parked | Read-only adoption experiment |
| CFR-173 | M2 parked | Product falsification / stop-rule experiment |
| CFB-133 | backlog parked | Migrate slice not reproduced by the completed local-contract M1 |
| CFR-169 | backlog parked | Architecture seam classified off the critical path |
| CFR-170 | backlog parked | Local-loop slice completed; all-command residual is not remote M1 permission |

`BATCH-2026-07-review-portfolio-completion` is frozen as history. Backlog-parked residuals do not enter an active slot without a later evidence-triggered board update.

## M2 preview (queued)

When M1 exits, M2 `MS-adoption-wedge` may classify CFR-171..173 and select at most one bounded read-only adoption experiment. It must not start early.

## Shift rule

On M1 close:

1. Record exit evidence in `history.md`.
2. Rename slots: old M2 becomes M1, old M3 becomes M2, and so on.
3. Name a new M5 from `strategy/modernization-roadmap.md` without turning that roadmap into a task board.
4. Refuse any in-flight parked work that tried to start early.
