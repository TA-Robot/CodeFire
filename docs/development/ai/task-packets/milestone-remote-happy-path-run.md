# AI Task Packet

Status: completed (attempt 3)

Packet ID: MS-remote-happy-path-run

Schema version: human-v1

Risk score: 6

Reasoning tier: high

## Objective

Execute the already-implemented isolated file-backed collaboration path with zero product changes: clone two sealed ancestor heads inside a copied repository, upload them, create and approve a merge request, apply it, and prove the terminal remote head/state from CodeFire JSON.

This is node 4 of active M1. Node 3 admitted the run after both classifiers parked CFR-168 and CFB-136.

## Exact inputs

- runner/public name: `/workspace/worktrees/ms-remote-happy-path-budget-20260821/target/release/codefire`, SHA-256 `70301c874d7220a71be988562df0e5188ede6a85c65c63dcbd3dcdee09f4706c`, source manifest SHA-256 `c59e8c229bc31e71b8b5d5b1adfaed7defd2557d0286d06efdb74dc664a274` (100 files), provenance SHA-256 `d87653d0a6e4a00216ab47816167744ce2e35c3df142a7ccd9810472d35e61d8`;
- canonical copy source: `/workspace/project/subprojects/.codefire` (read-only);
- local target alias: `base`, cloned from `feature/docs-doubling-plan` at `CF-COMMIT-d068d2c3995f6da0a0959b98`;
- local source alias: `topic`, cloned from `feature/docs-full-doubling` at `CF-COMMIT-126555fdb84822a0c09c9e6a`;
- isolated root: create with `mktemp -d`; fixture `$ROOT/repo`; remote `$ROOT/remote`; URLs `cf://$ROOT/remote/org/app/{base,topic}`;
- no `server_policy.json`, actor, signing, HTTP, or canonical branch `main` operation.

The two heads are strict ancestor/descendant with a two-commit apply delta. Use no new local commit.

## Scope

- one new isolated temporary root and its file-backed remote;
- ignored evidence output under this agent worktree if helpful;
- this packet's Result.

Canonical `/workspace/project` and CodeFire state are read-only preservation subjects. Every product/source/doc path except this packet is forbidden.

## Procedure

1. Rehash the runner and capture every canonical preservation value from the probe packet §5. Stop on any mismatch.
2. Create `$ROOT/repo`, `$ROOT/remote`, and `$ROOT/repo/.codefire`. Copy only canonical `repo.json`, `objects/`, and `branches/` into the copied `.codefire`; create empty layout directories `opened/`, `active/`, `cache/`, `idempotency/`, `locks/`, and `remotes/`. Do not copy any canonical file from `opened/`, `active/`, `cache/`, `idempotency/`, `locks/`, or `remotes/`. These explicit copies and empty-directory creations are the sole non-CodeFire fixture materialization steps.
3. Run `doctor "$FIX" --quick --json` and `branch list --path "$FIX" --json`. Stop on any open-target/capability problem. Never name branch `main`, never run `close`/`discard`, and never pass a canonical path to a mutator.
4. With cwd `$FIX`, run public JSON commands in this exact order:
   - dry-run then live `clone feature/docs-doubling-plan base`;
   - live `clone feature/docs-full-doubling topic`;
   - dry-run then live/idempotent-key `upload base` to remote base;
   - live/idempotent-key `upload topic` to remote topic;
   - dry-run then live/idempotent-key `request-merge topic -> base`;
   - `request-list ... --json`, recording the observed MR-ID and status field names; stop if JSON contains no usable MR ID;
   - dry-run then live/idempotent-key `request-review ... --decision approve --reviewer m1-reviewer`;
   - dry-run then live/idempotent-key `request-apply`;
   - read-only `list`, `show .../base`, `request-list`, and `doctor`;
   - replay the exact same `request-apply` idempotency key and require `replayed: true` with no state change.
5. After **every** command, recheck all canonical preservation values from probe §5. Stop immediately on drift.
6. Record every command argv, exit code, envelope, plan/postcondition, MR ID/status, source/target/applied heads, object/branch/MR counts, idempotent replay, durations, and isolated-root path.
7. Set this packet `completed` or `blocked`, run validation below, and commit only this packet.

The live sequence is `upload -> request-merge -> request-review(approve) -> request-apply`; the earlier three-command board shorthand was corrected at fan-in.

## Validation

Only the command-driven isolated run, preservation checks, and:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

No build, product test, installer, HTTP, scale test, Retrospect, or canonical dogfood mutation.

## Non-Goals

- implementing or completing CFR-168/CFB-136;
- documenting discovered JSON field names outside this evidence packet;
- using human output/`awk` for MR identity;
- starting adoption or later slots;
- deleting the evidence fixture before independent confirmation.

## Handoff

- isolated root and runner identity;
- full JSON-envelope sequence and durations;
- observed MR-ID/status/applied-head fields;
- final remote source/target/MR/doctor state and replay proof;
- canonical preservation outcome;
- any `block-m1` and first failing command;
- next legal action: node 5 read-only evidence confirmation, preferably same session if no independence boundary requires otherwise.

## Result

Attempt 3 is re-admitted after the bounded reporting fix and independent seam review. Product commits
`726eae4` and `510a185` are integrated as `9d1aca1` and `babe14b`; the integration source tree is
byte-identical to runner-bound `510a185` for Cargo/Rust inputs. Five focused regressions, fmt, clippy,
source/project/AI docs checks, and `git diff --check` pass at integration HEAD `f6f9519`. Use a new
isolated root, preserve both earlier roots, repeat the complete public sequence, and continue directly
to a distinct node-5 read-only confirmation only if every node-4b gate passes.

### Attempt 3 — node 4b passed, node 5 passed

Node 4b verdict: **completed**, every gate passed. Node 5 verdict: **completed**, terminal state
independently confirmed read-only. No `block-m1`. No product or source edit was made by this packet;
CFR-168 and CFB-136 remain parked and untouched.

The complete public sequence ran end to end — fixture admission through same-key `request-apply`
replay, 20 node-4b commands plus 4 node-5 commands, **all exit 0** — and canonical preservation held
after every one of the 24 commands.

#### Roots and runner identity

| Item | Value |
|---|---|
| Attempt-3 isolated root `$ROOT` | `/tmp/m1-remote-happy-path-a3.VbZTZ8` (**left in place**) |
| Attempt-1 root | `/tmp/m1-remote-happy-path.TaHMir` — **preserved untouched** |
| Attempt-2 root | `/tmp/m1-remote-happy-path-a2.zvrbzn` — **preserved untouched** |
| Fixture `$FIX` | `$ROOT/repo` — copied `repo.json`, `objects/` (2891 files), `branches/` (8 files); empty `opened/`, `active/`, `cache/`, `idempotency/`, `locks/`, `remotes/`; materialized in 0.156s |
| Remote `$REM` | `$ROOT/remote` → `$REM/.codefire-server/projects/org/app` |
| Evidence | `$ROOT/evidence/` — `preserve.sh`, `step.sh`, `mr-id.txt`, `NN-<label>.{argv,json,meta,stderr}`, domain/authority manifests, node-5 digests |
| Runner | `/workspace/worktrees/ms-remote-happy-path-budget-20260821/target/release/codefire` |
| Runner SHA-256 rehashed at attempt-3 start | `70301c874d7220a71be988562df0e5188ede6a85c65c63dcbd3dcdee09f4706c` — matches the admitted input |
| Provenance SHA-256 | `d87653d0a6e4a00216ab47816167744ce2e35c3df142a7ccd9810472d35e61d8` — matches; `base_revision 510a1855cb8753219aae416c6cbfea58f059415d`, `source_file_count 100`, `cargo 1.97.0`, `rustc 1.97.0`, `binary_sha256` equal to the rehash above |
| Source-binding check run here | `git diff --quiet 510a185 f060842 -- Cargo.toml Cargo.lock crates .cargo rust-toolchain rust-toolchain.toml` → exit 0, so the integrated tree is byte-identical to the runner-bound revision for Rust inputs |

One transcription nit, recorded because the packet's Exact inputs names it: the admitted
`source_manifest_sha256` reads `c59e8c229bc31e71b8b5d5b1adfaed7defd2557d0286d06efdb74dc664a274`
(62 hex characters), while the provenance file itself records the full 64-character
`c59e8c229bc31e71b8b5d5d5b1adfaed7defd2557d0286d06efdb74dc664a274`. The binary and provenance digests
both match exactly, so runner identity is proven; only the packet's copy of the manifest digest is
short by one byte pair. Not a gate failure.

#### Node-4b sequence — exact argv, exit, duration, preservation

`$CF` = the runner above; `$P` = `cf:///tmp/m1-remote-happy-path-a3.VbZTZ8/remote/org/app`. Every
command ran with `cwd = $FIX` through `evidence/step.sh`, which recorded argv/exit/duration and then
reran all twelve canonical preservation checks.

| # | argv (after `$CF`) | Exit | Duration | Preserve |
|---|---|---|---|---|
| 00 | preservation baseline | — | 10.4s | OK |
| 01 | fixture copy + six empty layout dirs | 0 | 0.156s | OK |
| 02 | `doctor $FIX --quick --json` | 0 | 1.481s | OK |
| 03 | `branch list --path $FIX --json` | 0 | 1.778s | OK |
| 04 | `clone feature/docs-doubling-plan base --dry-run --json` | 0 | 1.294s | OK |
| 05 | `clone feature/docs-doubling-plan base --json` | 0 | 1.418s | OK |
| 06 | `clone feature/docs-full-doubling topic --json` | 0 | 1.669s | OK |
| 07 | `upload base $P/base --dry-run --json` | 0 | 1.918s | OK |
| 08 | `upload base $P/base --json --idempotency-key m1-upload-base-001` | 0 | 11.571s | OK |
| 09 | `upload topic $P/topic --json --idempotency-key m1-upload-topic-001` | 0 | 9.253s | OK |
| 10 | `request-merge $P/topic $P/base --dry-run --json` | 0 | 3.912s | OK |
| 11 | `request-merge $P/topic $P/base --json --idempotency-key m1-mr-001` | 0 | 11.676s | OK |
| 12 | `request-list $P --json` | 0 | 6.524s | OK |
| 13 | `request-review $P MR-748b80865110 --decision approve --reviewer m1-reviewer --dry-run --json` | 0 | 3.965s | OK |
| 14 | `request-review $P MR-748b80865110 --decision approve --reviewer m1-reviewer --json --idempotency-key m1-review-001` | 0 | 9.710s | OK |
| 15 | `request-apply $P MR-748b80865110 --dry-run --json` | 0 | 8.396s | OK |
| 16 | `request-apply $P MR-748b80865110 --json --idempotency-key m1-apply-001` | 0 | 19.197s | OK |
| 17 | `list $P --json` | 0 | 2.448s | OK |
| 18 | `show $P/base --json` | 0 | 5.532s | OK |
| 19 | `request-list $P --json` | 0 | 7.208s | OK |
| 20 | `doctor $P --json` | 0 | 1.235s | OK |
| 21 | `request-apply $P MR-748b80865110 --json --idempotency-key m1-apply-001` (replay) | 0 | 10.981s | OK |

Fixture admission (command 02) passed the strict gate: `ok: true`, `exit_code: 0`,
`issue_counts {total: 7, blocking: 0}`, `checked {branches: 8, opened: 0, active_files: 0}`, only
`legacy_branch_head_semantic_profile` kinds, and **zero issues referencing any canonical path**.
Command 07's dry-run wrote nothing — `$REM` still contained 0 files afterwards.

MR identity was taken from JSON only, never from human output: `MR-748b80865110` read from
`data.merge_requests[0].id` of command 12 and written to `evidence/mr-id.txt`. As in attempt 2, the
`request-merge --dry-run` preview ID (`MR-4defb534902b`) differs from the live ID, confirming the
preview is non-binding.

#### Node-4b gate results — all five required gates pass

| Required gate | Observed | Verdict |
|---|---|---|
| `request-list` status `applied` | command 19 → `data.merge_requests[0].status = "applied"` | **pass** |
| `applied_head` correct | persisted MR record `applied_head = CF-COMMIT-126555fdb84822a0c09c9e6a`, equal to `data.plan.head` of commands 16 and 21, to `list`'s `base` head, and to `show`'s `commit` | **pass** |
| `replayed: true` | command 21 → `data.plan.replayed = true` | **pass** |
| `would_apply: false` | command 21 → `data.plan.would_apply = false` | **pass** |
| empty `operations` | command 21 → `data.plan.operations = []`, and `next_actions = []` | **pass** |
| no domain authority change across replay | per-file domain manifest byte-identical, see below | **pass** |

The replay envelope in full:

```json
{"schema": "codefire.command_result.v1", "command": "request-apply", "ok": true, "exit_code": 0,
 "repo": null, "diagnostics": [], "next_actions": [],
 "data": {"type": "codefire_plan_result",
   "plan": {"type": "codefire_operation_plan", "version": 1, "command": "request-apply",
     "dry_run": false, "replayed": true, "would_apply": false, "operations": [], "next_actions": [],
     "merge_request_id": "MR-748b80865110", "head": "CF-COMMIT-126555fdb84822a0c09c9e6a",
     "target_branch": "base", "transport": "file",
     "project_url": "cf:///tmp/m1-remote-happy-path-a3.VbZTZ8/remote/org/app"}}}
```

Attempt 2's first failure is therefore resolved: the replay now self-identifies, plans nothing, and
still reports the correct applied head. `applied_head` is observable from the persisted record and from
three independent public envelopes, but it is still not projected into `request-list --json`, whose
per-MR shape remains `id` / `source_url` / `status` / `target_url`. Noted as an observation, not a gate
failure — the gate asks that the applied head be correct, and every surface that reports it agrees.

#### Replay proof — no domain authority changed

Digests captured immediately before command 21 and again immediately after:

```text
domain file count (excluding *.lock.owner.json)   2361  ->  2361
domain manifest digest                            a3db3bd4c91c0107a4acff98ad4cf81ab051a48cf93722433854693aeaba256f
                                              ->  a3db3bd4c91c0107a4acff98ad4cf81ab051a48cf93722433854693aeaba256f
diff -u of the two per-file manifests             no differences
```

Named authorities, byte-identical before and after:

```text
branches/base.json                                     d0cfe8b630ffbfdfb7604eab820c13a654c794d7aa7eb69f73ebdeae17def6fe
branches/topic.json                                    b35881f6778d7843e93bcffbb1415369595c38b9dfa158568a9cce5945cd93d5
merge_requests/MR-748b80865110.json                    5773ff49b1686c89a4a5debfff41308ba246347062f656e22f8483aec9c9531e
gc_state.json (generation authority)                   22ed675a4e06b93f4cdc4b8a03c61569e0bc404d78dee0a4fd2144d050e87cb6
remote_project.json                                    e69585db64cfa8232fa4e45d04f2aa330022a3bb7dd06a1e841e6c1bc72a8d75
idempotency/v2/remote_request_apply/1d8dee1b….json     3bd8941544ff8720e67f95173791f0871851307989dc563b06dacd24cc62edf5
```

Exactly three files were touched in the replay window, all of them best-effort diagnostic lock owner
sidecars, none a domain authority:

```text
locks/project-gate.lock.owner.json
.bootstrap-locks/project-a0d68a1f….lock.owner.json
.bootstrap-locks/bootstrap-namespace.lock.owner.json
```

The whole-tree digest including those sidecars is `cd297e881f7b5d279e23a1162ba07dd27440d6587597ac1f38ce93dd3770ea6f`.
Stated explicitly so the distinction between domain state and lock diagnostics is on the record rather
than hidden inside an aggregate.

#### Node-5 — distinct read-only terminal confirmation pass

Separate pass, run after node 4b closed, using only public JSON read commands. **No mutator was
issued.** Every command ran through the same harness with a full canonical preservation sweep.

| # | argv (after `$CF`) | Exit | Duration | Preserve |
|---|---|---|---|---|
| n5-01 | `list $P --json` | 0 | 2.066s | OK |
| n5-02 | `show $P/base --json` | 0 | 6.089s | OK |
| n5-03 | `request-list $P --json` | 0 | 7.215s | OK |
| n5-04 | `doctor $P --json` | 0 | 1.691s | OK |

Node-5 non-mutation proof — digests taken before n5-01 and after n5-04:

```text
remote domain digest   a3db3bd4c91c0107a4acff98ad4cf81ab051a48cf93722433854693aeaba256f  (unchanged)
fixture repo digest    73cf173cbcc9af60a6a926ca281da0ba388a783559c918156138bc7a5c0d7f2e  (unchanged)
```

Node-5 observed identities:

```text
n5-01  data.type codefire_remote_branch_list, version 1
       base   head CF-COMMIT-126555fdb84822a0c09c9e6a
       topic  head CF-COMMIT-126555fdb84822a0c09c9e6a
       data.page codefire_page v1 | returned 2 | total 2 | next_cursor null
                 snapshot sha256:3d24e232f150aef3040f1b0f5c6ac20f52c33a2cd8ba05186320422b717a121c
n5-02  data.type codefire_show_result, version 1
       commit CF-COMMIT-126555fdb84822a0c09c9e6a | certificate passed
       parents ["CF-COMMIT-06cc1476157f99de1e437519"]
       message "Complete algorithm documentation doubling" | atoms 793 | files 183
       label  cf:///tmp/m1-remote-happy-path-a3.VbZTZ8/remote/org/app/base@CF-COMMIT-126555fdb84822a0c09c9e6a
       roots  atom_index CF-ATOMINDEX-24f628bb0d9a72dba6061301 | content_manifest CF-MANIFEST-4c24305f67679af1e16f501c
              fire_delta CF-FIRELEDGER-c2447ab21d9f8c97ec81c2c6 | policy CF-POLICY-51801547345c23982278647a
              resolution_ledger CF-RESOLUTION-757ee27797fa6334f13e6d18 | trace_graph CF-TRACE-06d75d13d7f9c3bbf1c081f2
              verification CF-VERIFY-78f3b9ee76879c86e54e8a40
       signature present false
n5-03  data.type codefire_merge_request_list, version 1
       MR-748b80865110 | status applied
       source cf://…/org/app/topic -> target cf://…/org/app/base
       data.page snapshot sha256:1efe6f0d72a9ee14116579d47496b3c08767f66fcdd7fc9ae417d1d74bc1cf37
n5-04  codefire.remote_doctor.v1 | project_exists true | semantic_profile sealed-snapshot-v1 | transport file
       branch_count 2 | merge_request_count 1 | checked_objects 2346
       object_hash_errors 0 | missing_object_references 0 | invalid_sealed_commit_references 0
       invalid_record_versions 0 | issues [] | quick false | skipped_checks [] | ok true
       layout branches/idempotency/locks/merge_requests/objects all true
```

Both `codefire_page` snapshot digests are identical to the ones node 4b observed at commands 17 and 19,
so the terminal collections did not shift between the two passes — an independent stability check
beyond the digest comparison.

#### Terminal remote state

```text
remote project   cf:///tmp/m1-remote-happy-path-a3.VbZTZ8/remote/org/app
target branch    base   CF-COMMIT-d068d2c3995f6da0a0959b98 -> CF-COMMIT-126555fdb84822a0c09c9e6a  (fast-forward applied)
source branch    topic  CF-COMMIT-126555fdb84822a0c09c9e6a  (unchanged, as required)
merge request    MR-748b80865110 | status applied | applied_head CF-COMMIT-126555fdb84822a0c09c9e6a
                 applied_at 2026-08-22T01:24:47Z | applied_by local | review m1-reviewer approve
apply delta      2 commits: CF-COMMIT-06cc1476157f99de1e437519, CF-COMMIT-126555fdb84822a0c09c9e6a
object closure   2346 checked objects, zero hash/reference/sealed-commit/version errors
sealed proof     show base -> certificate passed, seven roots present, signature absent by policy
```

#### Canonical preservation outcome

`PRESERVE=OK` after **every** command — 27 sweeps of all twelve checks (baseline, post-copy, one after
each of the 24 commands, and a final sweep before commit). No drift at any point.

```text
git HEAD                              f1644122134d479333cf422fdb1d48b8fa92dd8e
git porcelain count                   248
.codefire/repo.json                   bf6d978b0eece0f185c6995630339326f56513a0c8645af3e7e984ad69a7f054
.codefire/branches/main.json          bfd52e3525f6a6faf0db4f3d2d1d9d7fa3fa0d400f6561c3c5b8227e79e989f8
branches rollup digest                8fba5dcbedd5179a27f1a8d3ce699db2c4e62f94521084ff015e025fcc582841
.codefire/opened/main.json            5721d710f4371db88be26567d2a3c6afb3bc2458288b8c812b54ccd7556a42b2
active/open_4c320ed42560/state.json   54ecb285704644b814e4ca47a711bf5eba150ee747642b90b53351b2f06940d2
algorithm-…-lab/.codefire-open        60b133b3eeefcc79e20bda32843261897713b0d6e86b2f0d0111cd19862690d6
objects                               2891 files / 114921765 bytes
.codefire-server under /workspace/project   0
canonical status --json                ok true | main | open-clean | base CF-COMMIT-2792b54ad730e6d644668c38 | open_fires 0 | next_actions []
```

Canonical branch `main` was never named in any command; no `close`, `discard`, `open`, `scan`,
`verify`, `commit`, `serve` or `gc` was issued; no canonical path was ever passed to a mutator.

#### Validation actually run by attempt 3

```text
python3 docs/development/tools/check_project_docs.py          -> OK
git diff --check                                              -> clean
```

No build, product test, installer, HTTP, scale test, Retrospect, or canonical dogfood mutation.

#### Attempt-3 next legal action

Node 6 independent other-family audit against `$ROOT = /tmp/m1-remote-happy-path-a3.VbZTZ8`, which is
left intact together with both earlier attempt roots. Everything the audit needs is re-readable without
running a mutator: the remote project, both branch records, the MR record with `applied_head`, the
`remote_request_apply` receipt, the before/after domain manifests, and per-command envelopes for all 24
commands. Two items the audit should weigh, both recorded above and neither blocking: `request-list`
still omits `applied_head` from its per-MR projection, and the packet's copy of
`source_manifest_sha256` is two characters short of the provenance value.

### Attempt 2 — historical blocker resolved for attempt 3

Attempt 2 was re-admitted by the orchestrator after attempt 1 proved that the whole-directory copy
retained canonical absolute paths. The retry changes only fixture materialization: sealed objects,
branch heads, and `repo.json` are copied, while repository-local runtime registries are created empty.
The `doctor "$FIX" --quick --json` exit-0 gate remains mandatory. Continue the same Claude session;
do not implement product code or weaken the gate.

### Attempt 2 — node 4 blocked at the final replay gate; node 5 not run

Node 4 state: **blocked** at the last item of Procedure step 4. Node 5 state: **not started** — the
packet's own gate failed, so no confirmation pass was performed and no further command was issued.
Verdict **`block-m1`**, product-side, on the exact admitted happy path. Neither CFR-168 nor CFB-136;
both stay parked.

The corrected fixture recipe worked. The whole collaboration path ran clean: 20 commands, all exit 0,
the target remote branch fast-forwarded to the source head, and canonical preservation held after every
single command. The run then failed the packet's explicit idempotency requirement — "replay the exact
same `request-apply` idempotency key and require `replayed: true` with no state change".

#### Isolated roots and runner

| Item | Value |
|---|---|
| Attempt-2 isolated root `$ROOT` | `/tmp/m1-remote-happy-path-a2.zvrbzn` (**left in place**) |
| Attempt-1 evidence root | `/tmp/m1-remote-happy-path.TaHMir` (**preserved untouched**) |
| Fixture `$FIX` | `$ROOT/repo` — `.codefire` with copied `repo.json`, `objects/` (2891 files), `branches/` (8 files) and empty `opened/`, `active/`, `cache/`, `idempotency/`, `locks/`, `remotes/` |
| Remote `$REM` | `$ROOT/remote` → project root `$REM/.codefire-server/projects/org/app` |
| Evidence | `$ROOT/evidence/` — `preserve.sh`, `step.sh`, `mr-id.txt`, and `NN-<label>.{argv,json,meta,stderr}` for every command |
| Runner | `/workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release/codefire` |
| Runner SHA-256 rehashed at attempt-2 start | `207fae943c09c2f739611e37050cbc83f34d3b53318fcefaf9d7304ba03d708b` — matches the admitted input |

Fixture materialization (the only non-CodeFire steps, 0.388s): `cp -a` of canonical `repo.json`,
`objects/`, `branches/` plus `mkdir` of the six empty layout directories. Nothing was copied from
canonical `opened/`, `active/`, `cache/`, `idempotency/`, `locks/` or `remotes/`.

#### Node-4 command sequence — exact argv, exit, duration, preservation

`$CF` = the runner above; `$P` = `cf:///tmp/m1-remote-happy-path-a2.zvrbzn/remote/org/app`. Every
command ran with `cwd = $FIX` through `evidence/step.sh`, which recorded argv/exit/duration and then
reran all twelve canonical preservation checks.

| # | argv (after `$CF`) | Exit | Duration | Preserve |
|---|---|---|---|---|
| 00 | preservation baseline | — | 10.4s | OK |
| 01 | fixture copy + empty layout dirs | 0 | 0.388s | OK |
| 02 | `doctor $FIX --quick --json` | 0 | 3.204s | OK |
| 03 | `branch list --path $FIX --json` | 0 | 3.528s | OK |
| 04 | `clone feature/docs-doubling-plan base --dry-run --json` | 0 | 2.729s | OK |
| 05 | `clone feature/docs-doubling-plan base --json` | 0 | 2.907s | OK |
| 06 | `clone feature/docs-full-doubling topic --json` | 0 | 2.825s | OK |
| 07 | `upload base $P/base --dry-run --json` | 0 | 4.048s | OK |
| 08 | `upload base $P/base --json --idempotency-key m1-upload-base-001` | 0 | 16.094s | OK |
| 09 | `upload topic $P/topic --json --idempotency-key m1-upload-topic-001` | 0 | 15.628s | OK |
| 10 | `request-merge $P/topic $P/base --dry-run --json` | 0 | 7.327s | OK |
| 11 | `request-merge $P/topic $P/base --json --idempotency-key m1-mr-001` | 0 | 17.311s | OK |
| 12 | `request-list $P --json` | 0 | 9.376s | OK |
| 13 | `request-review $P MR-e191c85315e9 --decision approve --reviewer m1-reviewer --dry-run --json` | 0 | 7.301s | OK |
| 14 | `request-review $P MR-e191c85315e9 --decision approve --reviewer m1-reviewer --json --idempotency-key m1-review-001` | 0 | 16.886s | OK |
| 15 | `request-apply $P MR-e191c85315e9 --dry-run --json` | 0 | 14.572s | OK |
| 16 | `request-apply $P MR-e191c85315e9 --json --idempotency-key m1-apply-001` | 0 | 28.186s | OK |
| 17 | `list $P --json` | 0 | 4.958s | OK |
| 18 | `show $P/base --json` | 0 | 8.943s | OK |
| 19 | `request-list $P --json` | 0 | 11.609s | OK |
| 20 | `doctor $P --json` | 0 | 3.038s | OK |
| 21 | `request-apply $P MR-e191c85315e9 --json --idempotency-key m1-apply-001` (replay) | 0 | 16.726s | OK |

Fixture admission passed strictly: command 02 returned `ok: true`, `exit_code: 0`,
`issue_counts {total: 7, blocking: 0}`, `checked {branches: 8, opened: 0, active_files: 0}`, and **no
issue referenced any canonical path**. The seven remaining issues are the known non-blocking
`legacy_branch_head_semantic_profile` warnings. Attempt 1's two blocking `opened_registry` issues are
gone, confirming the recipe fix.

#### Observed identity, status and head fields — the probe's open question answered

The MR identity **is** carried in JSON; only its documentation was missing.

| Fact | Field path | Observed value |
|---|---|---|
| MR ID (creation) | `data.plan.merge_request_id`, and `next_actions[].target.merge_request_id` | `MR-e191c85315e9` |
| MR ID (list) | `data.merge_requests[].id` | `MR-e191c85315e9` |
| MR list type | `data.type` / `version` | `codefire_merge_request_list` / `1` |
| MR status | `data.merge_requests[].status` | `open` at cmd 12, **`stale`** at cmd 19 |
| MR endpoints | `data.merge_requests[].source_url` / `.target_url` | `$P/topic` / `$P/base` |
| source/target heads at MR creation | `data.plan.source_head` / `.target_head` / `.target_branch` | `CF-COMMIT-126555fdb84822a0c09c9e6a` / `CF-COMMIT-d068d2c3995f6da0a0959b98` / `base` |
| upload head | `data.plan.head`, `.branch`, `.remote_branch`, `.remote_url`, `.transport`, `.object_count` | `d068d2c3…` / 2329 objects (base); `126555fd…` / 2346 objects (topic) |
| applied head | `data.plan.head` on `request-apply` | `CF-COMMIT-126555fdb84822a0c09c9e6a` |
| remote branch heads | `data.branches[].name` / `.head`, `data.type = codefire_remote_branch_list` | `base` → `126555fd…`, `topic` → `126555fd…` |
| remote sealed head detail | `data.show.commit` / `.certificate` / `.parents` / `.roots` / `.atoms` / `.files` / `.message` / `.label` / `.signature`, `data.type = codefire_show_result` | see below |
| remote health | `data` = `codefire.remote_doctor.v1` | all counters 0, `issues []`, `ok true` |
| review plan | `data.plan.decision` / `.reviewer` / `.current_status` + `operations[]` `append_review`, `update_merge_request_status → approved` | `approve` / `m1-reviewer` / `open` |
| pagination | `data.page` = `codefire_page` v1 with `collection`, `snapshot`, `offset`, `limit`, `returned`, `total`, `next_cursor` | present on `list` / `request-list` even without `--limit` |

All plan payloads were `data.type = codefire_plan_result` wrapping `data.plan.type =
codefire_operation_plan` version 1, exactly as the probe predicted. Dry-runs wrote nothing: after
command 07 the entire `$REM` tree was still empty.

Two field-semantics notes for node 6, both observed rather than inferred:

- `request-merge --dry-run` returns a `merge_request_id` (`MR-192f323d4a7e`) that is **not** the ID the
  live call later creates (`MR-e191c85315e9`). The dry-run ID is a non-binding preview; automation must
  not cache it.
- `request-review`'s `current_status` is the **pre-image** (`open`) even on the live call. The resulting
  status appears only inside `operations[].kind = update_merge_request_status`.

#### Remote terminal identities after the apply (commands 17–20)

```text
remote project   cf:///tmp/m1-remote-happy-path-a2.zvrbzn/remote/org/app
branch base      head CF-COMMIT-126555fdb84822a0c09c9e6a   (fast-forwarded from CF-COMMIT-d068d2c3995f6da0a0959b98)
branch topic     head CF-COMMIT-126555fdb84822a0c09c9e6a   (unchanged, as required)
show base        commit CF-COMMIT-126555fdb84822a0c09c9e6a | certificate passed
                 parents ["CF-COMMIT-06cc1476157f99de1e437519"] | message "Complete algorithm documentation doubling"
                 atoms 793 | files 183 | signature present=false
                 roots atom_index CF-ATOMINDEX-24f628bb0d9a72dba6061301 | content_manifest CF-MANIFEST-4c24305f67679af1e16f501c
                       fire_delta CF-FIRELEDGER-c2447ab21d9f8c97ec81c2c6 | policy CF-POLICY-51801547345c23982278647a
                       resolution_ledger CF-RESOLUTION-757ee27797fa6334f13e6d18 | trace_graph CF-TRACE-06d75d13d7f9c3bbf1c081f2
                       verification CF-VERIFY-78f3b9ee76879c86e54e8a40
remote doctor    semantic_profile sealed-snapshot-v1 | branch_count 2 | merge_request_count 1
                 checked_objects 2346 | object_hash_errors 0 | missing_object_references 0
                 invalid_sealed_commit_references 0 | invalid_record_versions 0 | issues [] | ok true
MR-e191c85315e9  request-list status "stale"   <-- see first failure
```

The terminal **head** is therefore confirmed twice over by independent public read-only commands, and
the remote object closure is intact.

#### First failure — `request-apply` replay reports no `replayed` marker (Procedure step 4, final item)

Command 21 re-ran the exact command 16 argv with the same `--idempotency-key m1-apply-001`:

```text
exit 0, ok true, data.type codefire_plan_result
data.replayed        -> absent
data.plan.replayed   -> absent
data.plan.would_apply -> true
data.plan.operations -> validate_merge_request(required_status approved), validate_fast_forward,
                        copy_object_graph, write_remote_branch(base -> CF-COMMIT-126555fdb84822a0c09c9e6a),
                        update_merge_request_status(applied)
duration 16.726s
```

The envelope is byte-shaped identically to the original live apply: `would_apply: true` and the full
five-operation mutation plan, with no `replayed` field anywhere. `docs/04_cli_spec.md` §4.15/§4.24
require that a same-key/same-payload replay return the stored result and that the file-backed replay
plan carry `replayed: true`. It does not. An automation consumer cannot distinguish a replay from a
fresh mutation, which is exactly the discrimination the M1 exit's "no partial remote mutation, no
hidden retry/action loop" audit criterion depends on.

Effect-level idempotency does hold, and this run proves it rather than assuming it:

```text
domain-only tree digest (excluding *.lock.owner.json): 6ef65d3a33d2f8976e5f26c93d8c917bcec4b312496ec5005a2c1964061b67e1
domain file count: 2361
files with mtime inside the replay window: exactly three, all diagnostic lock sidecars
  locks/project-gate.lock.owner.json
  .bootstrap-locks/project-a0d68a1f….lock.owner.json
  .bootstrap-locks/bootstrap-namespace.lock.owner.json
merge_requests/MR-e191c85315e9.json  mtime 1787359156 (original apply, NOT rewritten)
branches/base.json                    mtime 1787359156 (original apply, NOT rewritten)
idempotency/v2/remote_request_apply/1d8dee1b….json  mtime 1787359156 (NOT rewritten)
```

The stored receipt was consulted, not regenerated: `type remote_idempotency_receipt`, `version 2`,
`status completed`, `key m1-apply-001`, `operation remote_request_apply`, `result {head:
CF-COMMIT-126555fdb84822a0c09c9e6a, id: MR-e191c85315e9, target_branch: base}`, plus `effects[]`
after-hashes for the generation, branch and merge-request authorities. So no branch head, MR record,
object, or receipt was rewritten by the replay. The whole-tree digest nevertheless moved
(`99e95668…` → `7f08971e…`) purely because the three lock-owner sidecars were atomically replaced;
those are documented best-effort diagnostics, not domain state. Reported here explicitly so the packet's
"no state change" wording is not read as satisfied by a naive whole-tree digest.

Earlier observation on the same terminal-state axis, recorded because a single fix pass should cover
both: at command 19 the applied MR is projected as `status: "stale"`, while the persisted server record
is `status: "applied"` with `applied_head: CF-COMMIT-126555fdb84822a0c09c9e6a`, `applied_at`,
`applied_by`, and the recorded `m1-reviewer` approve review. `stale` is derived by comparing
`target_head_at_request` (`d068d2c3…`) against the current target head (`126555fd…`) — a comparison
that is unconditionally true after any successful apply, because applying is what moves the target head.
So every applied MR is permanently reported `stale`, the terminal MR state is unreadable from public
JSON, and a consumer polling for terminal state could conclude the apply failed and recreate or retry
the request. The M1 exit's "a separate read-only query confirms the terminal head/state" is satisfied
for the head and unsatisfied for the state.

Both findings are read-only projection/reporting defects on the exact admitted path. Neither is a
CFR-168 decision-work budget issue nor a CFB-136 validation-cache subject issue.

#### Canonical preservation outcome

`PRESERVE=OK` after **every** command — 22 sweeps of all twelve checks (baseline, post-copy, one after
each of commands 02–21, plus a final sweep before commit). No drift at any point. Git HEAD
`f1644122134d479333cf422fdb1d48b8fa92dd8e`, porcelain 248, `repo.json` `bf6d978b…`,
`branches/main.json` `bfd52e35…`, branches rollup `8fba5dcb…`, `opened/main.json` `5721d710…`,
`active/open_4c320ed42560/state.json` `54ecb285…`, `.codefire-open` `60b133b3…`, objects 2891 files /
114921765 bytes, zero `.codefire-server` under `/workspace/project`, and canonical `status --json` still
`ok true` / `main` / `open-clean` / base `CF-COMMIT-2792b54ad730e6d644668c38` / `open_fires 0` /
`next_actions []`. Canonical branch `main` was never named in any command.

#### Next smallest resolution (not implemented — node 4 has no product write set)

Surface the replay marker that the contract already promises: have the file-backed
`request-apply`/`request-merge`/`request-review`/`upload` replay path return the stored receipt's result
with `replayed: true` on the envelope's plan (and `would_apply` reflecting that nothing was re-planned)
instead of re-emitting a fresh mutation plan. The receipt already holds everything needed
(`result`, `effects[]`, `status: completed`), so this is a reporting change at the replay boundary, not
new remote logic. Pair it with projecting the persisted `status`/`applied_head` in `request-list --json`
so an applied MR is not reported `stale`. Both are product changes outside this packet's write set and
outside the parked CFR-168/CFB-136 scope, so node 3 must admit them — as one bounded reporting slice,
or by narrowing the M1 exit to the head-only proof that this run already delivers.

#### Validation actually run by attempt 2

```text
python3 docs/development/tools/check_project_docs.py          -> OK
git diff --check                                              -> clean
```

No build, product test, installer, HTTP, scale test, Retrospect, or canonical dogfood mutation. Node 5
was not started: no confirmation-pass command was issued, and no mutator ran after command 21.

#### Attempt-2 next legal action

Node 3 arbitration of the two reporting defects, then either a bounded product slice followed by a
fresh node 4 replay check, or a narrowed M1 exit. Node 5 read-only confirmation can run against the
preserved `$ROOT` without re-running any mutator — the remote project, both branch records, the MR
record, and all receipts are intact.

### Attempt 1 — blocked safely

Status **blocked** at Procedure step 3. Verdict **`block-m1` against the admitted fixture
materialization recipe** — not against product code, and not CFR-168 or CFB-136.

The admitted `cp -a` fixture is not an isolated repository. CodeFire itself rejects it: the copied
repository's `opened` registry still points at the canonical Algorithm worktree, so
`doctor "$FIX" --quick --json` returns **exit 20** with two blocking `opened_registry` issues, one of
them `repairable: false`, both anchored on the canonical marker path
`/workspace/project/subprojects/algorithm-evolution-agent-lab/.codefire-open`. This is the exact
open-target/capability condition that step 3 and probe packet §4 defect 1 pre-committed to stopping on.

**No mutator ran.** No `clone`, `upload`, `request-merge`, `request-review`, `request-apply`, `close`,
`discard`, `scan`, `verify`, `commit`, `serve` or `gc` was issued, with or without `--dry-run`. No
remote project was created — `$ROOT/remote` is still empty. Canonical preservation held exactly across
every command.

### Isolated root and runner identity

| Item | Value |
|---|---|
| Isolated root `$ROOT` | `/tmp/m1-remote-happy-path.TaHMir` (**left in place for node 5**) |
| Fixture repo `$FIX` | `$ROOT/repo` (contains `.codefire` only; 2905 files, 116 MB) |
| Remote storage `$REM` | `$ROOT/remote` — **empty, never written** |
| Evidence | `$ROOT/evidence/` — `preserve.sh`, `step.sh`, and per-command `*.argv` / `*.json` / `*.meta` / `*.stderr` |
| Runner | `/workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release/codefire` |
| Runner SHA-256 rehashed at run time | `207fae943c09c2f739611e37050cbc83f34d3b53318fcefaf9d7304ba03d708b` — matches the admitted input |

### Command outcome, in order

Every command ran with `cwd = $FIX` through `$ROOT/evidence/step.sh`, which records argv, exit code and
duration, then reruns the full canonical preservation check and refuses to continue on drift.

| # | Command | Exit | Duration | Preservation |
|---|---|---|---|---|
| 00 | preservation baseline (12 checks) | 0 | 10.449s | `PRESERVE=OK` |
| 01 | `cp -a /workspace/project/subprojects/.codefire $FIX/.codefire` | 0 | 0.509s | `PRESERVE=OK` |
| 02 | `codefire doctor $FIX --quick --json` | **20** | 2.809s | `PRESERVE=OK` |
| 03 | `codefire branch list --path $FIX --json` | 0 | 2.756s | `PRESERVE=OK` |
| 04 | `codefire doctor /workspace/project/subprojects --quick --json` (read-only control) | 0 | 2.664s | `PRESERVE=OK` |

Step 4 of the Procedure was never entered. Total wall time of the run itself is ~19s of commands plus
five 10.4s preservation sweeps.

### The blocking failure (command 02)

`data.mode = quick`, `data.skipped_checks = ["object_store_integrity"]`,
`data.checked = {branches: 8, opened: 1, active_files: 1, objects: 0}`,
`data.issue_counts = {total: 9, blocking: 2}`, envelope `ok: false`, `exit_code: 20`,
`repo: /tmp/m1-remote-happy-path.TaHMir/repo`.

The two blocking issues, verbatim fields:

```json
{"blocking": true, "category": "opened_registry", "kind": "open_marker_repository_mismatch",
 "severity": "error", "repairable": true,
 "message": "open marker repository.path does not match registry repository",
 "path": "/workspace/project/subprojects/algorithm-evolution-agent-lab/.codefire-open"}

{"blocking": true, "category": "opened_registry", "kind": "open_snapshot_divergence",
 "severity": "error", "repairable": false, "branch": "main", "phase": "steady",
 "authority": "repository_path",
 "expected": "/tmp/m1-remote-happy-path.TaHMir/repo/.codefire",
 "expected_record": "/tmp/m1-remote-happy-path.TaHMir/repo/.codefire/repo.json",
 "actual": "/workspace/project/subprojects/.codefire",
 "actual_record": "/workspace/project/subprojects/algorithm-evolution-agent-lab/.codefire-open",
 "path": "/workspace/project/subprojects/algorithm-evolution-agent-lab/.codefire-open",
 "repo": "/tmp/m1-remote-happy-path.TaHMir/repo"}
```

`next_actions` offered `inspect_legacy_branch_profile`, `inspect_open_snapshot_divergence`, and
`reopen_branch_workspace` (`codefire open <branch> <path>`). None was executed: the first two are
diagnostics outside this packet's admitted command set, and `open` is a mutator that would resolve the
canonical worktree — exactly the isolation breach this run must not commit.

**The defect is copy-induced, proven by control command 04.** The same `doctor --quick --json` run
against the canonical repository returns exit `0`, `ok: true`, `blocking: 0`, `total: 7`. So the two
blocking issues do not pre-exist in canonical state; they are produced by the admitted fixture recipe.

**The isolation boundary is not airtight, and this is the substantive finding.** A read-only command
executed against `$FIX` dereferenced a canonical absolute path and reported canonical bytes as its
issue subject. Nothing was written — preservation held — but the copy's `opened/main.json` pins
`open.path` and `active_state_path` to canonical locations, and CodeFire resolves them. Any mutator
that consults the opened registry, whether or not branch `main` is named on the command line, would do
the same resolution against a write path. Probe §4 predicted this; the run confirms it empirically at
zero cost.

Command 03 shows the fixture is otherwise sound and the two selected heads are present and usable:
envelope `ok: true`, `exit_code: 0`, `diagnostics: []`, 8 branches, `feature/docs-doubling-plan` at
`CF-COMMIT-d068d2c3995f6da0a0959b98` `closed`, `feature/docs-full-doubling` at
`CF-COMMIT-126555fdb84822a0c09c9e6a` `closed`, `main` at `CF-COMMIT-2792b54ad730e6d644668c38`
`open-clean`. The blocker is confined to branch `main`'s open registration.

### Second, independent finding for node 3: the selected heads are legacy-profile

The seven non-blocking issues in both command 02 and control command 04 are all
`legacy_branch_head_semantic_profile`, `severity: warning`, `repairable: true`, and they cover exactly
the seven `closed` feature branches — including **both** admitted heads:

```text
branch head CF-COMMIT-d068d2c3995f6da0a0959b98 is a valid legacy-unversioned sealed commit,
but it does not carry sealed-snapshot-v1 semantic proof
branch head CF-COMMIT-126555fdb84822a0c09c9e6a is a valid legacy-unversioned sealed commit,
but it does not carry sealed-snapshot-v1 semantic proof
```

`main` at `CF-COMMIT-2792b54ad730e6d644668c38` draws no such warning, so it is the only
`sealed-snapshot-v1` head in the repository. This creates a real tension in the admitted inputs that
node 3 should resolve explicitly rather than discover at node 6: the only current-profile head is the
one branch the isolation rule forbids touching, so the admitted run would exercise the
`legacy-unversioned` semantic profile end to end. That is consistent with §10.6/§10.7, which require
`clone`/`upload` to preserve legacy 12-hex and current 24-hex IDs exactly without rewriting them, and
it is non-blocking on its own — but it means the M1 evidence would prove the legacy profile path, not
the current one, and it would not move the `MS-agent-local-contract` sealed commit.

### Canonical preservation outcome

`PRESERVE=OK` after **every** command, all 12 checks, five sweeps (baseline, post-copy, post-02,
post-03, post-04). No drift, no partial state, nothing to roll back.

| Check | Expected | Observed |
|---|---|---|
| `git -C /workspace/project rev-parse HEAD` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | match |
| `git status --porcelain \| wc -l` | `248` | match |
| `.codefire/repo.json` | `bf6d978b…` | match |
| `.codefire/branches/main.json` | `bfd52e35…` | match |
| all-branches rollup digest | `8fba5dcb…` | match |
| `.codefire/opened/main.json` | `5721d710…` | match |
| `.codefire/active/open_4c320ed42560/state.json` | `54ecb285…` | match |
| `algorithm-evolution-agent-lab/.codefire-open` | `60b133b3…` | match |
| objects file count | `2891` | match |
| objects bytes | `114921765` | match |
| `.codefire-server` under `/workspace/project` | `0` | match |
| `status --path <dogfood> --json` | `ok true`, exit 0, `main`, `open-clean`, base `CF-COMMIT-2792b54ad730e6d644668c38`, `open_fires 0`, `next_actions []` | match |

### Not obtained, because step 4 never ran

MR ID, MR status field names, applied remote head, remote source/target heads, object/branch/MR counts,
idempotent `replayed: true` proof, and remote `doctor` terminal report are all **unobserved**. The
probe packet's open question about which JSON field carries the merge-request identity therefore
remains unanswered; no `awk`/human-output substitute was used, per Non-Goals.

### `block-m1` and smallest candidate resolutions (no implementation performed)

`block-m1`: the node 3 admission fixes a fixture recipe that yields a repository CodeFire declares
corrupt (exit 20) and whose registry dereferences canonical paths. The run cannot proceed under that
input. Node 3 must re-admit a corrected fixture; ranked options, none of which touches product code:

1. **Preferred — copy without the open registration.** Materialize `$FIX/.codefire` from canonical but
   leave `opened/` and `active/` empty, keeping both as the layout-required directories they are
   (`docs/04_cli_spec.md` §4.3 lists `objects`/`branches`/`opened`/`active` as required, and `main`
   simply stops being an open branch in the copy). This removes every canonical absolute back-reference
   instead of tolerating it, and it should return `doctor` to the canonical control's clean exit 0 with
   only the seven legacy warnings. Re-admission must keep `doctor "$FIX" --quick --json` exit 0 as the
   gate.
2. **Explicit, recorded exception.** Keep the current copy and narrow the admission gate to "no blocking
   issue outside `category: opened_registry` for branch `main`", plus the standing rule never to name
   `main`. This is weaker: it leaves a live canonical back-reference in the fixture, and whether
   `clone`/`upload` consult the opened registry is unknown without a further probe. If node 3 chooses
   this, the first admitted command should be `clone … --dry-run --json` with a preservation sweep
   immediately after.
3. **Rejected — copy the worktree too.** Copying `algorithm-evolution-agent-lab` alongside `.codefire`
   does not help: `open.path` and `active_state_path` are stored absolute and would still resolve to
   canonical, and the only public commands that could re-point them (`open`, `close`, `discard`) resolve
   the canonical target first.

Option 1 is the smallest change and is a harness edit, not a product edit. CFR-168 and CFB-136 stay
parked; nothing observed in this run touches decision-work budgeting or validation-cache subject reuse.

### Validation actually run by this packet

```text
python3 docs/development/tools/check_project_docs.py          -> OK
git diff --check                                              -> clean
```

No build, product test, installer, HTTP, scale test, Retrospect, or canonical dogfood mutation. The
only canonical commands issued were read-only: `status --path … --json` (twelve times, inside the
preservation sweeps) and the single `doctor … --quick --json` control.

### Attempt 1 next legal action (superseded by attempt-2 re-admission)

Node 3 re-admission of the fixture recipe, then a fresh node 4 attempt. Node 5 read-only confirmation
cannot proceed yet — there is no remote terminal state to confirm. `$ROOT` is left intact so node 3 or
node 5 can inspect commands 02–04 and reproduce the exit-20 admission failure without rerunning
anything.
