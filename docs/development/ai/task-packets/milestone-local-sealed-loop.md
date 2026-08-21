# AI Task Packet

Status: in_progress

Packet ID: MS-local-sealed-loop

Schema version: human-v1

Base revision: the commit that closed `MS-2026-08-control-restore` on 2026-08-21

Risk score: 5

Reasoning tier: high

This is the **parent packet** for the milestone. Nodes 1 and 2 have their own child packets, per `../../milestone-operating-model.md` and `../orchestration-protocol.md`: one packet per parallel slice.

- node 1 → `milestone-local-sealed-loop-runner.md` — completed after independent correction `f963345`; cached runner admitted
- node 2 → `milestone-local-sealed-loop-increment.md` — ran; Status `completed`, integrated at `f52d489`
- node 3 → `milestone-local-sealed-loop-run.md` — ran; Status `completed`, integrated at `c471b77`; sealed `CF-COMMIT-373468dbaf827a6cc2bae9a4`
- node 4 → this file plus `docs/development/history.md` — **completed**; loop evidence recorded below. Node 5 is ready. M1 has **not** exited.

Everything from `## Objective` through `## Handoff` is the original **plan** and carries no evidence — do not read those sections as observations. The exceptions are the `Progress` / `Parallel now` / `Blocked` lines and the `State` and `Outcome` cells in `## Plan`, which this node 4 record updates, plus `## Fan-in result`, `## Orchestrator admission after independent classification`, and `## Loop evidence record`. Node 4 does not edit `docs/development/milestones.md`; the live board remains for node 6 / the orchestrator.

## Objective

Prove the local sealed-commit loop is usable for real day-to-day work by taking **one small real change** in the canonical dogfood project all the way through `open → edit → scan → extinguish → verify → commit`, using only the loop's focused validation.

The deliverable is a sealed commit plus CodeFire's own before/after report of the project state. An agent asserting "the loop works" is not the deliverable.

## Context

- Required context pack(s): pack-0-universal-orientation, pack-6-dogfood-algorithm-project
- Additional files: `docs/development/milestones.md`, `docs/development/milestone-operating-model.md`, `docs/development/known-limitations.md`, `subprojects/algorithm-evolution-agent-lab/AGENTS.md`
- Related issue or observation: `MS-2026-08-control-restore` restored the scheduler but validated no product behavior. This is the first slot that touches product/dogfood behavior.
- Board state to respect: the canonical Algorithm project was last recorded `open-clean` at sealed bridge `CF-COMMIT-e5ca290905c2a4b5e2c05bd4` with zero Fires. That is a **prior record**, not a fact this packet has re-observed. Node 3 re-reads it with `status --json` before the loop; nodes 1 and 2 must not rely on it and must not run `status` to check it.

## Plan

The live graph is on `docs/development/milestones.md` under "M1 execution graph (current)". That board is authoritative; this section is the packet-local copy.

```text
Critical path: loop runner readiness → sealed-loop run of the selected increment (pre-loop baseline first) → loop evidence record → final independent audit → M1 exit and slot shift
Progress: nodes 1–4 completed; node 3 sealed CF-COMMIT-373468dbaf827a6cc2bae9a4 (packet c471b77); node 4 records that evidence here
Parallel now: node 5 only — final independent audit (Claude Opus 5 or Grok 4.6, never Codex, not the implementing session)
Blocked: node 6 waits on 5
Next legal action: run the final independent audit into docs/development/ai/audit-latest.md only. Do not exit M1.
```

| # | Node | State | Waits on | Packet | Write set | Outcome / current evidence |
|---|---|---|---|---|---|---|
| 1 | Loop runner readiness | **completed after independent correction** | — | `milestone-local-sealed-loop-runner.md` | that packet file only | cached source-matching runner admitted; original search miss preserved and classified at `f963345` |
| 2 | Increment selection (reading only) | **completed** | — | `milestone-local-sealed-loop-increment.md` | that packet file only | one candidate file, one one-line edit, one focused unittest recorded; edit not applied, no command run. Integrated at `f52d489` |
| 3 | Sealed-loop run, pre-loop `status --json` baseline first | **completed** — integrated at `c471b77` | 1, 2 | `milestone-local-sealed-loop-run.md` | exact clean file in canonical open directory plus loop-written CodeFire state | sealed `CF-COMMIT-373468dbaf827a6cc2bae9a4`; full envelopes in the child packet Result |
| 4 | Loop evidence record | **completed** | 3 | this file | `docs/development/history.md`, this file | `## Loop evidence record` below; M1 not exited |
| 5 | Final independent audit | **ready** — not started | 4 | — | `docs/development/ai/audit-latest.md` only | — |
| 6 | M1 exit and slot shift | **blocked** — not started | 5 | — | board, history, packet, current-state docs | — |

Nodes 1 and 2 were independent and ran as such: node 1 settled *what executes the loop*, node 2 settled *what the loop carries*. Neither read the other's answer, and their write sets stayed disjoint — each child packet wrote only itself. That concurrency remains evidence of how those two lanes ran, not a standing second writer. Nodes 3 and 4 then ran serially. Node 5 is now the only ready work. Node 6 stays blocked on 5 and is orchestrator work; this record does not exit M1.

The pre-loop `status --json` baseline is **node 3 work**. It cannot run until a runnable executable exists, so placing it in node 2 would have given node 2 a hidden dependency on node 1 and destroyed the parallelism this graph claims. Node 2 selected by reading only, and node 1's blocker is exactly why that baseline still has no executable.

### Node 1 — loop runner readiness

Owned by `milestone-local-sealed-loop-runner.md`. Decide which `codefire` executable runs this loop and record its path, provenance, and capability outcome in that packet — or record an exact blocker there. Acceptable outcomes:

- a named, reachable executable with recorded provenance; or
- an explicit recorded blocker if no such executable can be obtained without leaving M1 scope.

Recording a blocker is a legitimate node 1 result. Inventing a build pipeline, pinning a toolchain, or fixing the installer is **not** in scope — those are release-slot concerns.

### Node 2 — increment selection

Owned by `milestone-local-sealed-loop-increment.md`. Pick the exact change by reading, and record in that packet the candidate file, the minimal edit, the reason it is worth making, and the focused test that covers it. Node 2 runs no CodeFire command, records no CodeFire state, and edits no product file.

Selection rules: one file, smallest real change with a reason to exist, and not a replay of any parked root. A prior record that the project is `open-clean` is not permission to skip the edit; the increment is a new small change either way.

### Node 3 — sealed-loop run

Created only after nodes 1 and 2 fan in, because its write set depends on their results. It captures the pre-loop `status --json` baseline verbatim using the node 1 runner, then runs the loop in order on the node 2 increment: `open → edit → scan → extinguish → verify → commit`, then the one focused test. Stop and report on the first command that does not behave as documented; do not work around it and do not widen validation to diagnose it.

## Scope

Allowed files, per node:

- `docs/development/ai/task-packets/milestone-local-sealed-loop-runner.md` (node 1, and nothing else)
- `docs/development/ai/task-packets/milestone-local-sealed-loop-increment.md` (node 2, and nothing else)
- exactly one file under `subprojects/algorithm-evolution-agent-lab/` (node 3), plus the CodeFire project state the loop itself writes
- `docs/development/history.md` and this file (node 4)
- `docs/development/milestones.md` (node 6, orchestrator only)

Nodes 1 and 2 write only their own packet. Neither may touch product code or CodeFire state.

Disallowed files:

- `crates/**`
- `codefire`
- `tests/**`
- `install.sh`, `pyproject.toml`, `setup.py`
- `adr/**`
- any second file under `subprojects/algorithm-evolution-agent-lab/`
- new CFB/CFR issues
- the classified dirty tree in `/workspace/project`, which is preserved pre-milestone state and not a write set

## Non-Goals

- Starting CFB-133, CFR-169, or CFR-170 (parked on M2) or any other parked root.
- Building the agent-consumable JSON / `next_actions` contract; that is M2.
- Remote `upload` / `request-merge` / `request-apply`; that is M3.
- Adoption or release work; those are M4 and M5.
- Changing sealed-commit, object-identity, or branch-head semantics.
- Running workspace-wide Rust tests, the full installer, the full Algorithm suite, or dogfood Retrospect as an admission ticket.
- Refactors or test expansion the loop does not require.

Parked-issue admission: a parked root enters this milestone only if node 3 produces dogfood evidence that the loop cannot complete without it, and then only the smallest slice that restores the loop. CFB-133 is the most likely candidate and is **not** admitted. Anticipating a block is not evidence.

## Invariants

Product invariants this packet must not violate:

- object identity: unchanged — the loop must not produce a new identity for unchanged content, and must not alias distinct content
- sealed commit validation: unchanged — the commit produced by node 3 must be sealed under the existing rules; a commit that only appears sealed is a failure, not a pass
- branch head atomicity: unchanged — the loop either advances the branch head completely or not at all
- Fire/Resolution semantics: `extinguish` must not clear a Fire that `scan` still has grounds to report, and `verify` must not pass over an unextinguished Fire
- remote mutation policy: unchanged — this milestone performs no remote mutation
- JSON envelope stability: unchanged — reading `status --json` must not require changing it
- path/workspace safety: all work stays inside the configured dogfood workspace; the separately configured legacy repository must not be mutated
- module boundaries: no product module is added, split, or moved by this milestone

If running the loop appears to require breaking one of these, that is a `block-m1` finding to report, not a thing to work around.

## Validation

```bash
RUNNER=/workspace/.cache/cargo-target-shared/source-keys/9c92559f4f4d51f270d27bcc5fff888936bda373dcb9150435607e6d33c861c6/release/codefire-rs
DOGFOOD=/workspace/project/subprojects/algorithm-evolution-agent-lab
"$RUNNER" status --path "$DOGFOOD" --json
"$RUNNER" scan --path "$DOGFOOD" --json
"$RUNNER" extinguish <each-scan-reported-fire-id> --path "$DOGFOOD" --resolution addressed --rationale "External-validity implementation corrected" --json
"$RUNNER" verify --path "$DOGFOOD" --details --json
"$RUNNER" commit --path "$DOGFOOD" -m "Fix unmapped benchmark transferability" --json
python3 docs/development/tools/check_project_docs.py
```

Plus the **focused** test of the single module the increment touches, run inside `subprojects/algorithm-evolution-agent-lab`.

Do not widen. `cargo test --workspace`, `cargo clippy`, `cargo fmt`, the installer, the full Algorithm suite, and Retrospect are out of this milestone.

If a command cannot run, report the exact command, failure, environment reason, and review risk. A skipped exit-gate command is not an exit.

## Handoff

- files changed;
- the selected increment and why it is the smallest real change;
- which executable ran the loop, and its provenance;
- CodeFire-reported state before and after, verbatim;
- the sealed commit identity;
- validation actually run, command by command, with results;
- parked instead of done;
- independent packets still unstarted;
- stall risk;
- next allowed action.

## Fan-in result

Observed at integration head `f52d489`. Nodes 1 and 2 ran in parallel and have both returned; this section records their results and nothing beyond them.

### What the two lanes returned

- **Node 1 — blocked (`block-m1`), integrated at `eab877f`.** `milestone-local-sealed-loop-runner.md` is Status `blocked`. It establishes that no in-scope executable can run the M1 exit-gate loop: the `--path` / `--json` surface the gate names belongs to the Rust CLI (`crates/codefire-cli`, bin `codefire-rs`), and no `codefire`, `cargo`, or `rustc` is on `PATH`, no build output or toolchain exists anywhere reachable, and the previously recorded July `/tmp` binary is gone. The Python `./codefire` at the repository root is durable, identity-known, and runnable, but implements neither `--path` nor `--json`, so it cannot express the gate or emit the JSON state evidence M1 exits on. Every probe there was read-only (`--help`, `ls`, `find`, `grep`, `git`, `sha256sum`, `pip show`); no CodeFire command ran against the dogfood project or any managed repository.
- **Node 2 — completed, integrated at `f52d489`.** `milestone-local-sealed-loop-increment.md` is Status `completed`. It selected exactly one candidate — a one-line `improved_domains` change in `subprojects/algorithm-evolution-agent-lab/src/evoagent/external_validity.py` — with its reason (an unmapped benchmark currently mints a synthetic `"unknown"` domain and can make `transferable=True` without a second real domain, against REQ-SOTA-007) and one focused unittest, `PYTHONPATH=src python3 -m unittest tests.test_external_validity -v`. Selection was by reading only: **the edit was not applied**, no CodeFire command ran, and no test ran. Alternates are recorded as fallbacks and are not to be applied.

### What is therefore true, and what is not

- Node 3 has not started. No product file, no dogfood file, and no CodeFire-managed state changed in this milestone.
- No product validation has run. The only commands recorded across both lanes are the two child packets' docs checks (`check_project_docs.py`, `git diff --check`) plus node 1's read-only help probes. Those say nothing about product behavior.
- There is no runner, no loop evidence, no sealed commit, and no independent audit for this milestone.
- **A recorded candidate did not clear the original blocker by itself.** The later independent classification and orchestrator admission cleared it by supplying the missed cached executable; neither event is product-loop evidence.
- The exact M1 exit on the board is unchanged by this fan-in, and so is the parked-work list. Nothing here admits CFB-133, CFB-136, or CFR-168..173, and node 1's blocker is not evidence for admitting any of them.

### Next legal action

1. **Independent classification of the node 1 runner blocker** by an agent that did not run node 1: confirm or refute the `block-m1` class and enumerate the smallest candidate resolutions. Classification records options only.
2. **Orchestrator admission** of the smallest resolution, recorded before any further node runs.

This packet does **not** choose the resolution. Provisioning a Rust toolchain or a build, amending the M1 exit gate so its evidence no longer requires `--json`, and adding `--path` / `--json` to the Python implementation are all out of this packet's write set: the first two change milestone scope, the third is product code and belongs to the M2 `MS-agent-local-contract` surface. Node 3 stays unwritten and unstarted until admission.

**Stall risk:** no independent M1 packet remains, so there is no legal parallel lane right now. Idle capacity must not be filled by inventing work, by pre-writing the node 3 packet, or by starting a parked root. Waiting on admission is the correct state, not a stall to be worked around.

## Orchestrator admission after independent classification

Independent classifier `f963345` classified node 1's original environment complaint as `park`, not `block-m1`, because a source-matching and provenance-recorded cached `codefire-rs` exists. The orchestrator selected R1: use it by absolute path, verify SHA-256 immediately before use, and perform no build, install, network, container, or product-code change.

The same classification proved that the canonical CodeFire open state exists only at `/workspace/project/subprojects/algorithm-evolution-agent-lab` with its store at `/workspace/project/subprojects/.codefire`. This is why ordinary Git-worktree isolation cannot run the real loop. The narrow exception is now explicit: node 3 may write only `/workspace/project/subprojects/algorithm-evolution-agent-lab/src/evoagent/external_validity.py` (confirmed Git-clean and byte-equal to the integration copy at admission) and the CodeFire state written by the loop. Every other `/workspace/project` tracked path remains forbidden. Node 3 records before/after Git status and stops if the exact file precondition changed.

The M1 command list is also made executable: extinguish applies only to actual scan-reported Fires and includes resolution/rationale; commit includes the fixed message. Zero Fires means a recorded no-op for the extinguish stage, not an invented Fire. The ready implementation packet is `milestone-local-sealed-loop-run.md`.

## Loop evidence record

Observed from completed node 3 packet `milestone-local-sealed-loop-run.md` at integration commit `c471b778737d40081d89021a3bc7990717ccd6e8`. Node 4 copies CodeFire's own reports; it did not re-run the loop, did not touch `/workspace/project`, and did not edit `docs/development/milestones.md`. M1 remains **open**. Final independent audit (node 5) has **not** run. This is not an M1 exit.

### Admitted runner

```text
path: /workspace/.cache/cargo-target-shared/source-keys/9c92559f4f4d51f270d27bcc5fff888936bda373dcb9150435607e6d33c861c6/release/codefire-rs
sha256: 09d9fa2561de4b687200baa4b9fe54a4159e43def1a7a3cd28dc05c2741741e4
```

Re-hashed before the first CodeFire command and again after commit. Match. No rebuild, no install. Provenance: classifier `f963345` plus orchestrator admission of the cached source-matching Rust binary.

Canonical open directory: `/workspace/project/subprojects/algorithm-evolution-agent-lab`. `open` was **not** invoked: pre-loop `status --json` already reported branch `main` as `open-clean`.

### Pre-loop state (CodeFire, verbatim)

Command: `"$RUNNER" status --path "$DOGFOOD" --json` — exit 0

```json
{
  "command": "status",
  "data": {
    "base": "CF-COMMIT-e5ca290905c2a4b5e2c05bd4",
    "branch": "main",
    "open_fires": 0,
    "scan_prediction": {
      "base_commit": "CF-COMMIT-e5ca290905c2a4b5e2c05bd4",
      "branch_state": "open-clean",
      "change_coverage": {
        "digest": "2beb6d95c37df1f42add132f40f75367226b8c47aedae881ce3b22796b74aed0",
        "entries": [],
        "entries_omitted": 0,
        "entries_truncated": false,
        "profile": "file-relation-v1",
        "span_precision": "file_only_no_span_claim",
        "summary": {
          "atom_files_with_unclassified_spans": 0,
          "changed_files": 0,
          "deleted_atom_files": 0,
          "non_atom_files": 0,
          "out_of_atom_files": 0,
          "tool_migrations": 0
        },
        "tool_migration": null,
        "type": "change_coverage",
        "version": 1
      },
      "changed_atom_count": 0,
      "changed_count": 0,
      "non_atom_changed_file_count": 0,
      "open_fire_count": 0,
      "source": "preview_recomputed",
      "tool_migration": null
    },
    "state": "open-clean"
  },
  "diagnostics": [],
  "exit_code": 0,
  "next_actions": [],
  "ok": true,
  "repo": "/workspace/project/subprojects",
  "schema": "codefire.command_result.v1"
}
```

### Exact one-line change

File: `/workspace/project/subprojects/algorithm-evolution-agent-lab/src/evoagent/external_validity.py` (`CODE-ExternalValidityEvaluator`). Admission sha256 `1e42de7df500e00f4be8d071de0a775dd3a2c2d018e927539106e23bab2c9df3` (Git-clean). Post-edit sha256 `873aa601542dec4337f17d4f05779d86e26ebc36ae286dc76cc2d420fe10c2e7`.

```diff
-        improved_domains = tuple(sorted({domain_by_benchmark.get(result.plan.benchmark, "unknown") for result in improvements}))
+        improved_domains = tuple(sorted({domain_by_benchmark[result.plan.benchmark] for result in improvements if result.plan.benchmark in domain_by_benchmark}))
```

No other dogfood or product file was edited. No test was added.

### Focused test

From the dogfood directory: `PYTHONPATH=src python3 -m unittest tests.test_external_validity -v` — exit 0. Two tests, both `ok`: `test_external_validity_rejects_single_domain_gain`, `test_external_validity_requires_transfer_across_domains`. Ran 2 tests in 0.001s, `OK`.

### Scan, extinguish, verify, commit

All CodeFire envelopes `schema: codefire.command_result.v1`, stderr empty, exit 0. Full JSON is in the node 3 packet Result.

- **scan** — `branch_state: "open-burning"`, `open_fire_count: 1`. The one scan-reported Fire is **`FIRE-C7EE54F7CE66`** (`fire_uid: fire_sha256_c7ee54f7ce66631db3f769dd8b953d04`, `reason: atom_changed`, `CODE-ExternalValidityEvaluator` → `REQ-SOTA-007`). `next_actions` were `context_changed`, `verify`, `context_fire`, `extinguish_fire`. None demanded a product change or a parked issue.
- **extinguish** — `"$RUNNER" extinguish FIRE-C7EE54F7CE66 --path "$DOGFOOD" --resolution addressed --rationale "External-validity implementation corrected" --json`. Applied. Resolution `res_5b39746c46d0`. Next action: verify only.
- **verify** — `"$RUNNER" verify --path "$DOGFOOD" --details --json`. `result: "passed"`, `decision.outcome: "passed"`, `open_required_fires: 0`, `blocking_reasons: []`. Seven `missing_required_links` warnings, non-blocking (`trace_completeness_required: false`). Next action: commit only.
- **commit** — `"$RUNNER" commit --path "$DOGFOOD" -m "Fix unmapped benchmark transferability" --json`. `applied: true`, `blocked: false`. Sealed identity **`CF-COMMIT-373468dbaf827a6cc2bae9a4`**, parent `CF-COMMIT-e5ca290905c2a4b5e2c05bd4`, `changed_atoms: ["CODE-ExternalValidityEvaluator"]`, `extinguished_fire_count: 1`.

Parked roots CFB-133 / CFB-136 / CFR-168..173 were not started. No `block-m1`.

### Post-loop state (CodeFire, verbatim)

Command: `"$RUNNER" status --path "$DOGFOOD" --json` — exit 0

```json
{
  "command": "status",
  "data": {
    "base": "CF-COMMIT-373468dbaf827a6cc2bae9a4",
    "branch": "main",
    "open_fires": 0,
    "scan_prediction": {
      "base_commit": "CF-COMMIT-373468dbaf827a6cc2bae9a4",
      "branch_state": "open-clean",
      "change_coverage": {
        "digest": "2beb6d95c37df1f42add132f40f75367226b8c47aedae881ce3b22796b74aed0",
        "entries": [],
        "entries_omitted": 0,
        "entries_truncated": false,
        "profile": "file-relation-v1",
        "span_precision": "file_only_no_span_claim",
        "summary": {
          "atom_files_with_unclassified_spans": 0,
          "changed_files": 0,
          "deleted_atom_files": 0,
          "non_atom_files": 0,
          "out_of_atom_files": 0,
          "tool_migrations": 0
        },
        "tool_migration": null,
        "type": "change_coverage",
        "version": 1
      },
      "changed_atom_count": 0,
      "changed_count": 0,
      "non_atom_changed_file_count": 0,
      "open_fire_count": 0,
      "source": "preview_recomputed",
      "tool_migration": null
    },
    "state": "open-clean"
  },
  "diagnostics": [],
  "exit_code": 0,
  "next_actions": [],
  "ok": true,
  "repo": "/workspace/project/subprojects",
  "schema": "codefire.command_result.v1"
}
```

Post-state: `open-clean`, `open_fires: 0`, base **`CF-COMMIT-373468dbaf827a6cc2bae9a4`**.

### Preservation outcome

`/workspace/project` Git `HEAD` stayed `f1644122134d479333cf422fdb1d48b8fa92dd8e`. Porcelain went 245 → 246 lines (`113 M`/`132 ??` → `114 M`/`132 ??`). The only newly dirty tracked path was the exact target file. Closed CodeFire feature-branch files unchanged. `.codefire-open` sha256 `60b133b3eeefcc79e20bda32843261897713b0d6e86b2f0d0111cd19862690d6` unchanged. `repo.json` unchanged. `opened/main.json` and `branches/main.json` advanced to the sealed head. Active `fires.json` / `scan.json` / `verification.json` were removed by commit `reset_active_state`; `active/state.json` remained `open-clean`. Stale `/workspace/algorithm-evolution-agent-lab` was unused. Sequencer leftovers under `/workspace/project/.git/sequencer` were untouched.

### What remains

Node 5 — final independent audit — is **ready** and has not started. Write set: `docs/development/ai/audit-latest.md` only. Auditor must not be Codex and must not be the implementing session. Node 6 (M1 exit and slot shift) stays **blocked**. Parked work stays parked. This packet Status remains `in_progress`.
