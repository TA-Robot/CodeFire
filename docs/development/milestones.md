# CodeFire Milestone Board

Status: live execution board

Owner: Orchestrator (Codex)

Last audited: 2026-08-21

Update trigger: M1 exit, M1 blocker admission, or slot shift.

M1 state at integration head `f52d489`: **blocked**. Nodes 1 and 2 ran in parallel and fanned in; node 1 recorded a `block-m1` runner blocker and node 2 completed its reading-only increment selection. No independent ready packet remains and no product behavior has been validated. M1 stays the only in-scope slot; being blocked does not admit M2–M5 or any parked root.

This is the only live five-ahead execution board. Historical burn-down narrative stays in `current-cycle-burn-down-plan.md` and `history.md`. Those files are not permission to work.

The previous milestone `MS-2026-08-control-restore` exited on 2026-08-21 and the board shifted one slot. Its exit evidence is in `history.md`; its execution record is in `ai/task-packets/milestone-control-restore.md`. Neither is evidence about the current M1.

## Active slots

| Slot | ID | Name | Exit (must all be true) | In-scope now |
|---|---|---|---|---|
| M1 | `MS-local-sealed-loop` | Daily local sealed-commit loop | One real change goes `open → edit → scan → extinguish → verify → commit` with only the loop's focused validation; no parked review root is started unless dogfood shows it currently blocks that loop | **Yes** |
| M2 | `MS-agent-local-contract` | Agent-consumable local contract | Local `status` / `scan` / `verify` / `extinguish` / `commit` JSON, `next_actions`, and help are enough for a Claude or Grok worker to finish an M1-style local sealed-loop change without reading product source | No |
| M3 | `MS-remote-happy-path` | Remote happy path of sealed history | `upload` → `request-merge` → `request-apply` of an already sealed commit works on the documented happy path; bounded HTTP/graph work is in scope only if it blocks that path | No |
| M4 | `MS-adoption-wedge` | First adoption wedge | A second repository can inspect or import without becoming a Git bridge or an agent platform; remaining product experiments record adopt / narrow / stop | No |
| M5 | `MS-release-operational-trust` | Release and operational trust | Derived from `strategy/modernization-roadmap.md` Stage 4, bounded to: a recorded rights/license decision for the artifacts CodeFire actually publishes, and **one** rehearsed release-and-rollback path a backup operator can execute from documentation only, with checksums and independent verification. No public beta label while source gates or release identity are unresolved. The rest of Stage 4 stays in the roadmap and is not admitted by this slot | No |

M5 is a bounded slice of Stage 4, not a copy of it. `strategy/modernization-roadmap.md` remains a stage/gate roadmap and must not be turned into a second task board.

## M1 card — Daily local sealed-commit loop

Objective: prove the local sealed-commit loop is usable for real day-to-day work by taking one small real change all the way through it.

Exact M1 exit (nothing else counts):

- one small real change goes `open → edit → scan → extinguish → verify → commit` in the canonical dogfood project;
- the resulting commit is sealed and the post-loop state is reported by CodeFire itself, not asserted by an agent;
- only the loop's focused validation was required to get there.

Required implementation (this slot only):

- resolve which `codefire` executable runs the loop, and record it;
- select the one small real change, by reading only, before the loop starts;
- capture the pre-loop state with `status --json`, then run the loop on that change;
- record the loop evidence and the sealed commit identity.

Capturing the baseline needs a working executable, so it belongs to the loop run, not to selection. Selection is a reading task and stays independent.

The first product increment is **one small real change** in `subprojects/algorithm-evolution-agent-lab`. If the canonical dogfood state is already `open-clean`, the increment is a new small change, not a replay of a parked root.

M1 validation (do not widen):

```bash
codefire status --path subprojects/algorithm-evolution-agent-lab --json
codefire scan --path subprojects/algorithm-evolution-agent-lab
codefire extinguish --path subprojects/algorithm-evolution-agent-lab
codefire verify --path subprojects/algorithm-evolution-agent-lab
codefire commit --path subprojects/algorithm-evolution-agent-lab
python3 docs/development/tools/check_project_docs.py
```

Plus the **focused** test of the single module the increment touches, inside `subprojects/algorithm-evolution-agent-lab`.

`cargo test --workspace`, `cargo clippy`, `cargo fmt`, the full installer, the full Algorithm suite, and dogfood Retrospect are **out of M1**. They belong to later slots if that slot names them.

M1 non-goals:

- starting CFB-133, CFR-169, or CFR-170 (now parked on M2) or any other parked root;
- building the agent-consumable JSON / `next_actions` contract (that is M2);
- remote `upload` / `request-merge` / `request-apply` work (M3);
- adoption or release work (M4, M5);
- refactors, cleanups, or test expansion that the loop does not require;
- treating the classified dirty tree in `/workspace/project` as a write set;
- serializing independent M1 slices "to keep the tree simple".

Parked-issue admission rule for this slot: a parked root enters M1 **only** if running the loop produces dogfood evidence that the loop cannot complete without it, and then only the smallest slice that restores the loop. Anticipating a block is not evidence. CFB-133 (`migrate apply` next-action loop) is the most likely candidate and is **not** admitted today.

## M1 execution graph (current)

This is the live plan for `MS-local-sealed-loop`. It is a concrete graph, not a template. Every active M1 must carry one; the shape rule lives in `milestone-operating-model.md`. A single serial queue with no parallel set is not an M1 plan.

```text
Critical path: loop runner readiness → sealed-loop run of the selected increment (pre-loop baseline first) → loop evidence record → final independent audit → M1 exit and slot shift
Progress: nodes 1 and 2 ran concurrently and have fanned in at f52d489 — node 1 is blocked with a recorded block-m1 runner blocker (eab877f), node 2 is completed with the increment selection (f52d489); node 3 has not started and nodes 3-6 stay blocked
Parallel now: none — no independent ready packet remains. Node 2 is done, node 1 returned a blocker, and every other node depends on the unresolved node 1 result
Blocked: node 3 waits on 1 and 2 — node 2's input exists but node 1 supplied no runner, so node 3 cannot start; node 4 waits on 3; node 5 waits on 4; node 6 waits on 5
Next legal action: independent classification of the node 1 runner blocker, then orchestrator admission of the smallest resolution. The classification does not pick the resolution
```

Nodes 1 and 2 have run; no other node has. Node 1's result is a recorded blocker, not a runner. Node 2's result is a candidate one-line increment plus its focused test, chosen by reading only — selecting a candidate does not clear the runner blocker.

At `f52d489` no CodeFire command has run against the dogfood project, no product or dogfood file has been edited, no focused test has run, no product validation of any kind has run, there is no sealed commit, and there is no audit for this milestone. The two validations recorded so far are the child packets' docs checks, which say nothing about product behavior.

### Nodes

| # | Node | State | Owner | Waits on | Packet | Write set | Evidence |
|---|---|---|---|---|---|---|---|
| 1 | Loop runner readiness — decide which `codefire` executable runs the loop and record its path, provenance, and capability outcome, or record an exact blocker | **blocked** — ran, recorded an exact blocker (`block-m1`); no runner selected | Individual agent (Claude Opus 5) | — | `ai/task-packets/milestone-local-sealed-loop-runner.md` | that packet file only | `eab877f`; runner packet `## Result`: no in-scope executable can express the `--path` / `--json` gate. No `codefire`, `cargo`, or `rustc` on `PATH`; the recorded July `/tmp` binary is gone; the gate surface belongs to the Rust CLI (`crates/codefire-cli`, bin `codefire-rs`), unbuildable in scope; Python `./codefire` is durable and runnable but implements neither `--path` nor `--json`. Read-only help probes only; no CodeFire command was run against any managed repository |
| 2 | Increment selection — name the exact one-file small change, why it is the smallest real change, and which focused test covers it. Reading only: does **not** run or record CodeFire state | **completed** — one exact candidate and one focused test recorded | Individual agent | — | `ai/task-packets/milestone-local-sealed-loop-increment.md` | that packet file only | `f52d489`; increment packet `## Result`: one-line `improved_domains` change in `subprojects/algorithm-evolution-agent-lab/src/evoagent/external_validity.py`, focused test `PYTHONPATH=src python3 -m unittest tests.test_external_validity -v`. Selection is by reading only — the edit was **not** applied, and no CodeFire command and no test were run |
| 3 | Sealed-loop run — capture the pre-loop `status --json` baseline with the node 1 runner, then `open → edit → scan → extinguish → verify → commit` on the node 2 increment plus its focused test | **blocked** on 1 — not started; node 2's input exists, but node 1 supplied no runner for the first mandated step | Individual agent | 1, 2 | not written; blocked on the node 1 resolution | the one touched file under `subprojects/algorithm-evolution-agent-lab/`, and the CodeFire state that loop writes | — |
| 4 | Loop evidence record — sealed commit identity, before/after state, and which commands actually ran | **blocked** on 3 | Individual agent | 3 | parent packet | `docs/development/history.md`, `ai/task-packets/milestone-local-sealed-loop.md` | — |
| 5 | Final independent audit (Claude Opus 5 or Grok 4.6, never Codex, not the implementing session) | **blocked** on 4 | Audit agent | 4 | — | `docs/development/ai/audit-latest.md` only | — |
| 6 | M1 exit and slot shift | **blocked** on 5 | Orchestrator | 5 | — | board, history, packet, current-state docs | — |

`ai/task-packets/milestone-local-sealed-loop.md` is the parent packet for this milestone. Nodes 1 and 2 each had their own child packet, per `milestone-operating-model.md` and `ai/orchestration-protocol.md`: one packet per parallel slice. Both have now fanned in, but the node 3 implementation packet is still **not** written: node 1 returned a blocker instead of a runner, so node 3's first mandated step has no executable and its write set cannot be fixed. It must not be pre-written.

### Two-lane concurrency, as executed (evidence, not currently ready work)

This section records what actually ran in parallel. It is not a standing invitation to launch a second lane now — no independent ready packet remains.

Node 1 answered *what runs the loop*, and answered it with a blocker: no in-scope executable can express the M1 gate. Node 2 answered *what the loop carries*, by **reading** the dogfood project only — no CodeFire invocation, no product edit, no state capture — which is why it did not wait on node 1 and completed while node 1 was failing to find a runner.

The pre-loop `status --json` baseline stayed **node 3 work, not node 2 work**: it requires a runnable executable, so it sits after the fan-in and immediately before `open`. Putting it in node 2 would have given node 2 a hidden dependency on node 1, and the two lanes would not have been parallel at all. The disjoint write sets held — each child packet wrote only itself — and the two lanes ran concurrently to completion, which is the evidence that the split was real.

### Not yet true

M1 is **open** and blocked. Nodes 1 and 2 have run; nodes 3-6 have not. There is no runner, no product or dogfood edit, no focused-test run, no loop evidence, no sealed commit, and no independent audit for this milestone.

Node 2's completion is **not** progress on the loop: a recorded candidate is not an applied change, and it does not clear the node 1 runner blocker. Do not read the previous milestone's green validations as evidence here either: those covered docs and control-plane scripts at `8b6c822` and said nothing about product behavior. The two child-packet doc checks at `eab877f` and `f52d489` are the same kind of evidence — docs-only. This milestone is the first slot that touches product/dogfood behavior, so its exit needs its own loop evidence and its own independent audit.

### Next legal action

1. **Independent classification of the node 1 runner blocker** by an agent that did not run node 1 — is it `block-m1`, `park`, or `drop`, and what is the smallest set of candidate resolutions? Classification records options; it does not pick one and does not implement one.
2. **Orchestrator admission** of the smallest resolution, on the record, before any further node runs.

Not decided here, and not to be started by an agent: provisioning a Rust toolchain or a build, amending the M1 exit gate's `--json` requirement, or adding `--path` / `--json` to the Python implementation. Each changes milestone scope or product code and is the orchestrator's call at admission. Node 3 stays unstarted and unwritten until then. Idle capacity here is not a stall to be filled with invented parallel work; there is no independent M1 packet left to launch, and manufacturing one would be a process violation.

## Parked open issues

These remain `investigating` in the ledgers. They are not M1 work.

| Issue | Parked on | Why not M1 |
|---|---|---|
| CFB-133 | M2 | `migrate apply` next-action loop; an automation contract concern, not required to run the loop by hand |
| CFR-169 | M2 | Module/test ownership seams; not required to run the loop |
| CFR-170 | M2 | Typed executable outcomes; needed once agents consume local contracts |
| CFR-168 | M3 | Graph/object/HTTP work budget is for remote/scale, not for the local loop |
| CFB-136 | M3 | Validation-cache reuse is the admitted CFR-168 blocker |
| CFR-171 | M4 | Machine-readable claim/proof descriptor for external consumers |
| CFR-172 | M4 | Read-only adoption experiment |
| CFR-173 | M4 | Product falsification / stop-rule experiment |

`BATCH-2026-07-review-portfolio-completion` is **frozen as history**. Remaining roots are parked as above. They do not gate M1 or M2 unless dogfood evidence shows the local sealed loop is actually unusable without them.

## M2 preview (queued)

When M1 exits, M2 `MS-agent-local-contract` becomes active and the first allowed increment is:

- make the local `status` / `scan` / `verify` / `extinguish` / `commit` JSON and `next_actions` sufficient for a worker to repeat an M1-style loop change without reading product source;
- admit CFR-170 or CFB-133 only to the extent that contract requires them.

Do not pull CFR-168 into M2 because it is unfinished.

## Shift rule

On M1 close:

1. Record exit evidence in `history.md`.
2. Rename slots: old M2 becomes M1, old M3 becomes M2, and so on.
3. Name a new M5 from `strategy/modernization-roadmap.md` without turning that roadmap into a task board.
4. Refuse any in-flight parked work that tried to start early.
