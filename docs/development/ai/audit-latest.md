# Milestone audit

Final independent other-family audit of active M1 `MS-remote-happy-path` (node 6 of the M1 execution graph).

- time (ISO): `2026-08-22T01:41:14Z`
- audited HEAD (exact): `74bbcd6df999f345f803696de6a3398ebeba977c` (`74bbcd6`, "docs: advance M1 remote path to final audit")
- evidence root: `/tmp/m1-remote-happy-path-a3.VbZTZ8` (preserved attempt-3 fixture/remote/evidence; left unmodified in domain state)
- active milestone: M1 `MS-remote-happy-path` — Remote happy path of sealed history
- stalled: **no**
- scope_creep: **no**
- next_milestone_risk: low. M2 `MS-adoption-wedge` remains queued. Nothing in this M1's evidence admits CFR-168, CFB-136, HTTP transport, scale, installer, or adoption work. `request-list` still omits `applied_head` from its per-MR JSON item; that is `park`, not an M1 blocker, because the apply and `show` envelopes independently identify the applied head.
- auditor: Grok 4.6, new session, separate worktree `worktrees/ms-remote-final-audit-20260822` on branch `grok-agent/m1-remote-final-audit-20260822`. Nodes 4/4b/5 were a persistent Claude session. This session is not the run executor and did not inspect implementer or executor uncommitted state. Independence per `orchestration-protocol.md` (fresh other-family Grok session) and `audit-cadence.md` is satisfied. Codex was not used.
- write set honored: this file only. `/workspace/project`, its CodeFire state, the preserved evidence root, and every product/dogfood file were left unmodified as domain authorities. This audit issued only read-only CodeFire commands (`status`, `list`, `show`, `request-list`, `doctor`). `milestones.md`, `history.md`, issue files, and packets were not edited. M1 was not exited and no slot was shifted.

This audit verified evidence against the environment. It did not accept packet prose as proof: every hash, envelope field, receipt, authority, and preservation claim below was re-derived independently at the audited HEAD.

## Exit criteria

Exact M1 exit from the card, in order.

| # | Exact exit criterion | Verdict | How it was verified independently |
|---|---|---|---|
| EC1 | an isolated fixture and exact public commands are selected before any remote mutation | **pass** | Probe packet selected the isolated copy recipe, slash-free `base`/`topic` aliases, `cf://` URLs, and the public command sequence before node 4. Attempt-3 evidence shows fixture copy of `repo.json`/`objects`/`branches` plus empty `opened`/`active`/`cache`/`idempotency`/`locks`/`remotes`, then `doctor --quick` and `branch list` before any `clone`/`upload`/`request-*`. Command 02: `ok true`, `blocking 0`, `opened 0`, seven non-blocking `legacy_branch_head_semantic_profile` warnings, **zero issue paths under `/workspace/project`**. No `server_policy.json`. No `--actor`, HTTP, or branch `main` in any recorded argv. |
| EC2 | already sealed local history completes `upload -> request-merge -> request-review(approve) -> request-apply` on the file-backed happy path | **pass** | Twenty node-4b commands plus four node-5 commands, all exit 0, cwd `$FIX`. Live mutators: upload base `CF-COMMIT-d068d2c3995f6da0a0959b98` then topic `CF-COMMIT-126555fdb84822a0c09c9e6a`; request-merge created `MR-748b80865110` from JSON `data.merge_requests[0].id` (also `data.plan.merge_request_id`); request-review `--decision approve --reviewer m1-reviewer`; request-apply. Replay argv is byte-identical to the live apply argv. No new local commit. Target `base` fast-forwarded from `d068d2c3…` to `126555fd…`; source `topic` unchanged at `126555fd…`. |
| EC3 | CodeFire's JSON envelopes identify the remote source/target/MR/applied head, and a separate read-only query confirms the terminal head/state | **pass** | See identity table and the `applied_head` adjudication below. Live apply `data.plan.head`, remote `list` `base`/`topic` heads, and `show` `commit` all equal `CF-COMMIT-126555fdb84822a0c09c9e6a`. `request-list` identifies MR `MR-748b80865110`, source/target URLs, and persisted status `applied`. This audit re-ran the four read-only commands; envelopes are byte-identical to node-5 evidence. |
| EC4 | an independent other-family audit verifies object closure, no partial remote mutation, preservation of canonical dogfood state, and no hidden retry/action loop | **pass** | Remote `doctor --json`: `ok true`, `checked_objects 2346`, all four error counters 0, `issues []`. All live mutators exited 0 with completed v2 receipts. Same-key apply replay returns `replayed: true`, `would_apply: false`, empty `operations`/`next_actions`, and does not rewrite domain authorities. Canonical preservation (probe §5, twelve checks) is `PRESERVE=OK` at audit time, including after the live read-only confirmation. |

Required implementation / run for this slot:

| Item | Verdict | Independent verification |
|---|---|---|
| preselected isolated public commands | **pass** | Exact argv recorded for commands 02–21 and n5-01–n5-04; sequence matches the admitted run packet. |
| runner / provenance / binary / public-name identity | **pass** | See identity tables. Public name `codefire` is a hardlink of `codefire-rs` (inode `1361045`). Binary SHA-256 `70301c87…` matches provenance. Source manifest is **64 hex characters** `c59e8c229bc31e71b8b5d5d5b1adfaed7defd2557d0286d06efdb74dc664a274`, 100 files, recomputed at the audited HEAD. |
| bounded reporting fix (replay marker + applied list status) | **pass** | Product commits `726eae4`/`510a185` integrated as `9d1aca1`/`babe14b`. `git diff --quiet 510a185 HEAD -- Cargo.toml Cargo.lock crates .cargo rust-toolchain rust-toolchain.toml` exits 0. Replay plan is built only for `JsonPublicationReplayKind::CompletedReceipt`. `request-list` projects persisted `applied` and still assigns `stale` for non-terminal drift. |
| isolated attempt-3 run + distinct read-only confirmation | **pass** | Evidence root intact. 20/20 + 4/4 exit 0. This audit's live four-command confirmation matches node-5 JSON exactly. |
| preservation of `/workspace/project` | **pass** | All twelve probe-§5 values match at audit time. No `.codefire-server` under `/workspace/project`. Canonical `status --json` is `ok true` / `main` / `open-clean` / base `CF-COMMIT-2792b54ad730e6d644668c38` / `open_fires 0` / `next_actions []`. |

## Adjudication: `request-list` omitting `applied_head`

**Verdict: `park`, not `block-m1`.**

The M1 card requires that CodeFire's JSON envelopes identify the remote source/target/MR/applied head, and that a separate read-only query confirm the terminal head/state. It does not require that a single `request-list` item carry all four identities.

Independently observed public JSON:

| Surface | Fields that identify | Observed value |
|---|---|---|
| `request-apply` live/replay `data.plan.head` | applied head | `CF-COMMIT-126555fdb84822a0c09c9e6a` |
| `list` `data.branches[]` | terminal branch heads | `base` and `topic` both `CF-COMMIT-126555fdb84822a0c09c9e6a` |
| `show …/base` `data.show.commit` | sealed applied head | `CF-COMMIT-126555fdb84822a0c09c9e6a`, certificate `passed` |
| `request-list` `data.merge_requests[]` | MR / source / target / state | `id=MR-748b80865110`, `source_url=…/topic`, `target_url=…/base`, `status=applied` |
| persisted MR record (not a public envelope) | `applied_head` | `CF-COMMIT-126555fdb84822a0c09c9e6a` |

Live and audit-time `request-list` items have exactly the keys `id`, `source_url`, `status`, `target_url`. `MergeRequestListItem` has no `applied_head` field. The admitted reporting-fix / public contract (`docs/automation_interface.md`, `command-source-trace.md` `request-list` row) requires that persisted `applied` survive target-head drift; it does not require projecting `applied_head` on the list item.

Attempt 2's actual M1 hazard was `status: "stale"` after a successful apply, which could make a consumer retry. That is fixed: terminal `request-list` is `applied`, and completed-receipt replay is machine-distinguishable (`replayed: true`, `would_apply: false`, empty operations, empty `next_actions`). Omitting `applied_head` from the list item does not create a hidden retry/action loop and does not leave the applied head unidentified in CodeFire JSON.

Park as later list-projection completeness. Do not reopen product work before M1 exit.

## Adjudication: 64-character source manifest claim

**Verdict: the real digest is 64 hex characters and binds the runner. The run packet's first quoted "admitted" string is a 62-character truncation. `park` for citation hygiene, not `block-m1`.**

Independently recomputed at audited HEAD with `build_source_manifest()`:

```text
source_file_count          100
source_manifest_sha256     c59e8c229bc31e71b8b5d5d5b1adfaed7defd2557d0286d06efdb74dc664a274   (len 64)
provenance file            same digest; file sha256 d87653d0a6e4a00216ab47816167744ce2e35c3df142a7ccd9810472d35e61d8
binary sha256              70301c874d7220a71be988562df0e5188ede6a85c65c63dcbd3dcdee09f4706c
```

The run packet Exact inputs and Result first quote `c59e8c229bc31e71b8b5d5b1adfaed7defd2557d0286d06efdb74dc664a274` (len **62**) and call it 64 hex characters. That string is the real digest with one `d5` dropped from the `d5d5` run (`c59e8c229bc31e71b8b5d5` + `d5b1…` instead of `d5d5b1…`). The same Result paragraph then quotes the provenance value, which is the correct 64-character digest. Binary, provenance, and recomputed manifest agree. Node 7 must cite the 64-character value, not the truncated quote.

## Evidence

- git: audited HEAD `74bbcd6df999f345f803696de6a3398ebeba977c`. Runner-bound source is `510a1855cb8753219aae416c6cbfea58f059415d`, integrated as `9d1aca1` + `babe14b`. `git diff --quiet 510a185 HEAD -- Cargo.toml Cargo.lock crates .cargo rust-toolchain rust-toolchain.toml` exits 0. `510a185..HEAD` product/Rust inputs are empty; remaining delta is docs (`milestone-remote-happy-path-reporting-fix.md`, `…-reporting-review.md`, `…-run.md`, `milestones.md`). No `codefire` Python tree, no installer, no `subprojects/**`, and no CFB/CFR ledger file is in that range.
- packets in flight: probe/budget/reporting-fix/reporting-review/run all `completed`; node 6 (this audit) is the only ready node; node 7 remains blocked and is orchestrator work.
- evidence root `/tmp/m1-remote-happy-path-a3.VbZTZ8` read in place. Earlier roots `/tmp/m1-remote-happy-path.TaHMir` and `/tmp/m1-remote-happy-path-a2.zvrbzn` were not mutated.
- validation actually run **by this audit**:
  - `git rev-parse HEAD` → `74bbcd6df999f345f803696de6a3398ebeba977c`;
  - `build_source_manifest()` at the audited HEAD → **100 files**, `c59e8c229bc31e71b8b5d5d5b1adfaed7defd2557d0286d06efdb74dc664a274` (64 hex chars);
  - `git diff --quiet 510a185 HEAD -- Cargo.toml Cargo.lock crates .cargo rust-toolchain rust-toolchain.toml` → exit 0;
  - `sha256sum` / `stat` of admitted `codefire` and `codefire-rs` → identical inode `1361045`, 14594080 bytes, sha256 `70301c87…`;
  - provenance path sha256 `d87653d0…` and content (`base_revision 510a185`, 100 files, matching binary hash, cargo/rustc 1.97.0);
  - independent re-read of every attempt-3 `*.argv` / `*.json` / `*.meta`, MR record, branch records, apply receipt, and domain manifests;
  - `find … -type f ! -name '*.lock.owner.json' | sort | xargs sha256sum | sha256sum` on `$REM` → `a3db3bd4c91c0107a4acff98ad4cf81ab051a48cf93722433854693aeaba256f` (matches node-5 domain digest and the before/after replay manifests);
  - fixture all-files digest `73cf173cbcc9af60a6a926ca281da0ba388a783559c918156138bc7a5c0d7f2e` (matches node-5 fixture digest) both before and after this audit's read-only queries;
  - live `"$RUNNER" list|show|request-list|doctor --json` against the preserved remote, cwd `$FIX` → four envelopes **byte-identical** to `n5-01`…`n5-04`; domain digest unchanged across those calls;
  - `bash evidence/preserve.sh` → `PRESERVE=OK` before and after the live confirmation;
  - `cargo test -p codefire-cli --bin codefire-rs` with pinned rustc/cargo 1.97.0 and isolated `CARGO_TARGET_DIR`:
    - `tests::request_apply_replay::file_backed_request_apply_replay_and_applied_list_status` ok
    - `tests::request_apply_replay::file_backed_request_apply_pending_recovery_is_not_receipt_replay` ok
    - `tests::request_apply_replay::file_backed_request_list_keeps_stale_projection_for_open_mr` ok
    - `tests::request_apply_replay::file_backed_request_list_projects_stale_for_drifted_rejected_mr` ok
    - `tests::remote_mutator_idempotency_replays_same_payload_and_rejects_conflicts` ok
    - counts: 4 passed / 771 filtered, then 1 passed / 774 filtered;
  - `python3 docs/development/tools/check_project_docs.py`, `python3 docs/development/tools/check_ai_acceleration_docs.py`, and `git diff --check` for this audit's own docs change.

No `cargo test --workspace`, Clippy, installer, HTTP, scale test, or dogfood Retrospect was run. No CodeFire mutator (`upload` / `request-*` live, `clone`, `close`, `discard`, `scan`, `verify`, `commit`, `serve`, `gc`) was issued by this audit. Nothing was rebuilt or installed for the runner; tests used a separate `CARGO_TARGET_DIR`.

### Runner, provenance, binary, and public-name identity

Independently observed at audit time:

| Claim | Recorded | Independently observed | Match |
|---|---|---|---|
| public command name | `codefire` | argv[0] in every evidence `*.argv` is the admitted `…/target/release/codefire` | yes |
| PATH prefix | `/workspace/worktrees/ms-remote-happy-path-budget-20260821/target/release` | directory present; binaries at that prefix | yes |
| `codefire` hardlink | same inode/bytes as `codefire-rs` | inode `1361045`, nlink `3`, 14594080 bytes, mode `755` for both | yes |
| `codefire` sha256 | `70301c874d7220a71be988562df0e5188ede6a85c65c63dcbd3dcdee09f4706c` | identical | yes |
| `codefire-rs` sha256 | same | same | yes |
| third hardlink | (cargo layout) | `target/release/deps/codefire_rs-730042419a756626`, same inode | yes (cargo artifact, not a substitute runner) |
| provenance path | `.../target/codefire-ai/rust-build-provenance.json` | present | yes |
| provenance sha256 | `d87653d0a6e4a00216ab47816167744ce2e35c3df142a7ccd9810472d35e61d8` | identical | yes |
| provenance schema | `codefire.rust_build_provenance.v1` | same, `ok: true` | yes |
| build HEAD | `510a1855cb8753219aae416c6cbfea58f059415d` | `base_revision` identical | yes |
| Rust source manifest | 64 hex chars / 100 files | recomputed at audited HEAD: `c59e8c229bc31e71b8b5d5d5b1adfaed7defd2557d0286d06efdb74dc664a274`, 100 files | yes (not the 62-char packet typo) |
| build command | `cargo build --release --locked -p codefire-cli --bin codefire-rs` | provenance `build_command` identical | yes |
| toolchain | cargo 1.97.0 / rustc 1.97.0 | `cargo 1.97.0 (c980f4866 2026-06-30)`, `rustc 1.97.0 (2d8144b78 2026-07-07)` | yes |

Because the 100-file manifest recomputes identically at `74bbcd6` and the Rust input tree is byte-identical to `510a185`, **the binary that ran attempt 3 corresponds to the Rust source at the audited HEAD**.

### Isolated command sequence (node 4b + node 5)

`$CF` = admitted runner; `$P` = `cf:///tmp/m1-remote-happy-path-a3.VbZTZ8/remote/org/app`; cwd = `$FIX`. Every recorded meta has `exit=0` and `preserve=PRESERVE=OK`. Stderr files are empty.

| # | Public argv after `$CF` | Exit | Duration | Notes independently confirmed |
|---|---|---|---|---|
| 02 | `doctor $FIX --quick --json` | 0 | 1.481s | `blocking 0`, opened 0, no canonical paths |
| 03 | `branch list --path $FIX --json` | 0 | 1.778s | selected heads present and `closed` |
| 04 | `clone feature/docs-doubling-plan base --dry-run --json` | 0 | 1.294s | `dry_run true`, `would_apply false` |
| 05 | `clone feature/docs-doubling-plan base --json` | 0 | 1.418s | source head `d068d2c3…` |
| 06 | `clone feature/docs-full-doubling topic --json` | 0 | 1.669s | source head `126555fd…` |
| 07 | `upload base $P/base --dry-run --json` | 0 | 1.918s | `dry_run true`, `would_apply false` |
| 08 | `upload base $P/base --json --idempotency-key m1-upload-base-001` | 0 | 11.571s | head `d068d2c3…`, 2329 objects |
| 09 | `upload topic $P/topic --json --idempotency-key m1-upload-topic-001` | 0 | 9.253s | head `126555fd…`, 2346 objects |
| 10 | `request-merge $P/topic $P/base --dry-run --json` | 0 | 3.912s | preview id `MR-4defb534902b` (non-binding) |
| 11 | `request-merge $P/topic $P/base --json --idempotency-key m1-mr-001` | 0 | 11.676s | live id `MR-748b80865110` from JSON |
| 12 | `request-list $P --json` | 0 | 6.524s | `id` + `status=open`; this is the MR identity source |
| 13 | `request-review $P MR-748b80865110 --decision approve --reviewer m1-reviewer --dry-run --json` | 0 | 3.965s | `would_apply false` |
| 14 | `request-review … --json --idempotency-key m1-review-001` | 0 | 9.710s | ops include `append_review` + `update_merge_request_status=approved` |
| 15 | `request-apply $P MR-748b80865110 --dry-run --json` | 0 | 8.396s | `would_apply false`, five ops, head `126555fd…` |
| 16 | `request-apply … --json --idempotency-key m1-apply-001` | 0 | 19.197s | `would_apply true`, five ops, no `replayed` key |
| 17 | `list $P --json` | 0 | 2.448s | both branches `126555fd…`; page snapshot `3d24e232…` |
| 18 | `show $P/base --json` | 0 | 5.532s | commit `126555fd…`, certificate `passed` |
| 19 | `request-list $P --json` | 0 | 7.208s | `status=applied`; page snapshot `1efe6f0d…` |
| 20 | `doctor $P --json` | 0 | 1.235s | 2346 objects, zero integrity errors |
| 21 | same argv as 16 (replay) | 0 | 10.981s | `replayed true`, `would_apply false`, empty ops/actions |
| n5-01..04 | `list`, `show`, `request-list`, `doctor` | 0 | — | distinct pass; this audit's live rerun is byte-identical |

Forbidden-token scan of every `*.argv`: no `--actor`, `cf+http`, `cf+https`, `close`, `discard`, `open`, `scan`, `verify`, `commit`, `serve`, `gc`, or named branch `main`.

MR identity was taken from JSON (`data.merge_requests[0].id` / `data.plan.merge_request_id`), never from human output. `evidence/mr-id.txt` is `MR-748b80865110`.

### Terminal identities (authorities + envelopes)

Persisted remote project `cf:///tmp/m1-remote-happy-path-a3.VbZTZ8/remote/org/app`, re-read directly:

```text
branches/base.json     head CF-COMMIT-126555fdb84822a0c09c9e6a  sha256 d0cfe8b630ffbfdfb7604eab820c13a654c794d7aa7eb69f73ebdeae17def6fe
branches/topic.json    head CF-COMMIT-126555fdb84822a0c09c9e6a  sha256 b35881f6778d7843e93bcffbb1415369595c38b9dfa158568a9cce5945cd93d5
MR-748b80865110.json   status applied
                       applied_head CF-COMMIT-126555fdb84822a0c09c9e6a
                       source_head  CF-COMMIT-126555fdb84822a0c09c9e6a
                       target_head_at_request CF-COMMIT-d068d2c3995f6da0a0959b98
                       review m1-reviewer approve, subject_digest sha256:ffcdebfe…
                       sha256 5773ff49b1686c89a4a5debfff41308ba246347062f656e22f8483aec9c9531e
gc_state.json          current_generation 3  sha256 22ed675a4e06b93f4cdc4b8a03c61569e0bc404d78dee0a4fd2144d050e87cb6
remote_project.json    CF-PROJECT-32b7a10f805a28f33c63d279086a19f6  sha256 e69585db…
apply receipt          type remote_idempotency_receipt v2 status completed
                       key m1-apply-001 operation remote_request_apply
                       result {head: CF-COMMIT-126555fdb84822a0c09c9e6a, id: MR-748b80865110, target_branch: base}
                       sha256 3bd8941544ff8720e67f95173791f0871851307989dc563b06dacd24cc62edf5
server_policy.json     absent
object files           2346 (matches doctor checked_objects)
```

Apply delta is the documented two-commit fast-forward: `show` parents `["CF-COMMIT-06cc1476157f99de1e437519"]`. Fixture local `base` remains `d068d2c3…` `closed`; local `topic` remains `126555fd…` `closed`. The two extra fixture objects versus canonical `objects/` are the clone-created branch objects `CF-BRANCH-5a6ca272…` and `CF-BRANCH-eefc77d2…`.

This audit's live read-only confirmation (`2026-08-22T01:37Z`):

```text
list          ok true  base/topic head CF-COMMIT-126555fdb84822a0c09c9e6a
              page snapshot sha256:3d24e232f150aef3040f1b0f5c6ac20f52c33a2cd8ba05186320422b717a121c
show base     commit CF-COMMIT-126555fdb84822a0c09c9e6a  certificate passed
              parents [CF-COMMIT-06cc1476157f99de1e437519]  atoms 793  files 183
              seven roots present; signature present false
request-list  MR-748b80865110 status applied  keys {id, source_url, status, target_url}
              page snapshot sha256:1efe6f0d72a9ee14116579d47496b3c08767f66fcdd7fc9ae417d1d74bc1cf37
doctor        remote_doctor.v1 ok true  project_exists true  transport file
              semantic_profile sealed-snapshot-v1  branches 2  mrs 1
              checked_objects 2346  all error counters 0  issues []
```

All four live envelopes are byte-identical to the node-5 evidence files.

### Completed-receipt replay, no domain authority change, no hidden loop

Command 21 argv equals command 16 argv (`request-apply $P MR-748b80865110 --json --idempotency-key m1-apply-001`).

Replay envelope:

```text
replayed true  would_apply false  operations []  next_actions []
head CF-COMMIT-126555fdb84822a0c09c9e6a  target_branch base  merge_request_id MR-748b80865110
```

Those `head` / `target_branch` values equal the completed receipt `result`. Live apply has `would_apply true`, five operations, and no `replayed` key. Source: `apply_merge_request` substitutes `request_apply_replay_plan` only for `JsonPublicationReplayKind::CompletedReceipt`; `Recovery` keeps the live plan. Focused tests cover that discriminant, including a pending-recovery case that must not report replay.

Domain manifests before and after replay are byte-identical (2361 files, digest `a3db3bd4…`). Named authorities listed above are still those exact bytes at audit time. `domain-diff-across-replay.txt` is empty. This audit's later read-only commands left the domain-excluding-`*.lock.owner.json` digest at `a3db3bd4…`. Lock-owner sidecars may move; they are documented diagnostics, not domain authorities.

Terminal envelopes have `next_actions []`. Replay does not re-propose apply. `request-list` status `applied` does not look like a failed apply. No hidden retry/action loop on this path.

### Preservation boundary

Independently observed at audit time (and again after the live four-command confirmation):

| Preservation claim | Independently observed | Verdict |
|---|---|---|
| `/workspace/project` Git HEAD `f1644122134d479333cf422fdb1d48b8fa92dd8e` | identical | pass |
| porcelain count `248` | 248 | pass |
| `.codefire/repo.json` `bf6d978b…` | identical | pass |
| `.codefire/branches/main.json` `bfd52e35…` | identical | pass |
| branches rollup `8fba5dcb…` | identical | pass |
| `.codefire/opened/main.json` `5721d710…` | identical | pass |
| `active/open_4c320ed42560/state.json` `54ecb285…` | identical | pass |
| `algorithm-evolution-agent-lab/.codefire-open` `60b133b3…` | identical | pass |
| objects 2891 files / 114921765 bytes | identical | pass |
| `.codefire-server` under `/workspace/project` | 0 | pass |
| canonical `status --json` `ok true` / `main` / `open-clean` / `CF-COMMIT-2792b54ad730e6d644668c38` / `open_fires 0` / `next_actions []` | identical | pass |

Canonical branch `main` was never named in a recorded mutator. No canonical path was passed to a mutator. Fixture `opened/` and `active/` remain empty. Fixture `locks/` contains only the copy-local `repo.lock` pair from fixture commands, not canonical paths.

### Scope / non-goals / issues

At the audited HEAD, open items remain `investigating` with board placement unchanged:

```text
CFR-168 investigating  M1 input, parked (not admitted; path did not reproduce a budget failure)
CFB-136 investigating  M1 input, parked (not admitted; path does not share a command-wide cache)
CFR-171 investigating  M2 parked
CFR-172 investigating  M2 parked
CFR-173 investigating  M2 parked
CFB-133 investigating  backlog parked
CFR-169 investigating  backlog parked
CFR-170 investigating  backlog parked
```

`510a185..HEAD` does not touch `docs/development/bug-issues/**` or `docs/development/code-review-issues/**`. No HTTP/`cf+http` command ran. No installer, scale, Retrospect, or Python default-CLI substitution. Reporting-fix write set stayed on file-backed JSON projection plus focused tests and the admitted docs. This audit did not start parked work.

### Validation not widened

The card's out-of-M1 list was respected. Workspace-wide `cargo test`, Clippy, full installer, and dogfood Retrospect were not used as gates. This audit reran only the packet-named focused product tests plus the standing docs checks for its own file.

## Findings

Classified only as `block-m1`, `park`, or `drop`, per `milestone-operating-model.md`.

**`block-m1`: none.** No exit criterion is false at the audited HEAD against the preserved attempt-3 root.

`park` — valid, not required for M1 exit:

1. **`park` — `request-list` omits `applied_head`.** Per-MR JSON is still `id` / `source_url` / `status` / `target_url`. Apply `plan.head`, `list` branch heads, and `show` commit independently identify `CF-COMMIT-126555fdb84822a0c09c9e6a`; `request-list` identifies MR/source/target and terminal `applied`. Public contract after the reporting fix requires persisted `applied` status, not this extra field. Do not implement before M1 exit. **Retained after the `68a94b4` docs-only follow-up.**
2. **`superseded` — run-packet 62-character source-manifest truncation.** Corrected at integration `68a94b4` (`docs: correct M1 runner manifest citation`). The run packet now quotes `c59e8c229bc31e71b8b5d5d5b1adfaed7defd2557d0286d06efdb74dc664a274` (64 hex characters) in all three claim sites. No longer a park finding.
3. **`park` → node 7 board refresh.** `milestones.md` still has node 4 as "blocked at final reporting gate", "Last audited: 2026-08-21", and a leftover sentence that nothing on the graph has run. Fold node-6 completion, this follow-up HEAD, and the attempt-3 evidence root into the same exit/slot-shift edit. Do not open a separate packet.
4. **`park` — file-backed `upload` / `request-merge` / `request-review` replay still omit `replayed: true`.** Review P1/P2. Not exercised by attempt-3's apply replay, which is the admitted path.
5. **`park` — HTTP and Python `request-list` still overwrite applied with `stale`.** Review P3. Deliberate `cf://` vs `cf+http://`/Python divergence. Out of this file-backed M1.
6. **`park` — no dedicated T6 regression that corrupts the current target head under an `applied` MR and expects `request-list` to fail closed.** Source still validates the current target sealed head unconditionally (`remote.rs` list path). Live `request-list` succeeded against a valid head. Missing test does not falsify this run.

`drop` — out of boundary, duplicate, or noise; not filed:

7. **`drop`.** `request-merge --dry-run` preview id `MR-4defb534902b` differs from live `MR-748b80865110`. Real automation hazard, already recorded; attempt 3 took the live JSON id.
8. **`drop`.** `request-review` `current_status` is the pre-image `open` on the live call. Documented plan contract.
9. **`drop`.** Lock-owner sidecar mtimes / whole-tree digest including owners. Domain digest excluding those files is stable. This audit's shared-lock read-only commands may rewrite owner sidecars; fixture all-files digest stayed `73cf173c…`.
10. **`drop`.** Local fixture `doctor --quick` warns that the two admitted heads are `legacy-unversioned`; remote `doctor` reports `semantic_profile sealed-snapshot-v1` and `show` certificate `passed`. Probe already parked this. M1 proves the selected already-sealed path, not a rewrite onto `main`'s current-profile head.
11. **`drop`.** `show` `signature.present false` is expected without `commit_signatures.required`.
12. **`drop`.** `codefire`/`codefire-rs` nlink is 3 because cargo also hardlinks `deps/codefire_rs-730042419a756626`. Same inode and bytes; not a substitute runner.
13. **`drop`.** `docs/development/ai/audit-latest.md` is gitignored (`.gitignore:25`). This is the auditor write set behaving as designed; the commit is force-added, as prior milestone audits were.

## Advice (at most five bullets)

- Node 7 may proceed on integration HEAD `68a94b4`: run the M1 exit, record exit evidence in `history.md`, shift every slot forward one step, and name a new M5 from `strategy/modernization-roadmap.md` without turning that roadmap into a second board.
- Fold remaining `park` finding 3 into that same node 7 board/history edit. Cite the 64-character source-manifest digest `c59e8c229bc31e71b8b5d5d5b1adfaed7defd2557d0286d06efdb74dc664a274`, the attempt-3 root `/tmp/m1-remote-happy-path-a3.VbZTZ8`, and integration HEAD `68a94b4`.
- Keep `park` finding 1 (`request-list` `applied_head`) parked. Do not admit a follow-up product packet before the slot shift.
- Keep CFR-168 and CFB-136 `investigating` and unadmitted. Keep HTTP, Python parity, upload/merge/review replay markers, and adoption (CFR-171..173) parked on later slots.
- Do not widen validation at exit. The standing docs checks plus this audit's already-run focused product tests are the gate; workspace-wide Rust, installer, and Retrospect stay out.

## Close authorization

**Authorized.** M1 `MS-remote-happy-path` may be closed by node 7 on the evidence at exact HEAD `74bbcd6df999f345f803696de6a3398ebeba977c` with preserved evidence root `/tmp/m1-remote-happy-path-a3.VbZTZ8`.

All four exact exit criteria are true, and each was verified against the environment rather than from packet prose: an isolated fixture and preselected public commands completed sealed `upload -> request-merge -> request-review(approve) -> request-apply` on file-backed `cf://`; JSON envelopes identify source/target/MR and the applied head `CF-COMMIT-126555fdb84822a0c09c9e6a`; a separate read-only query (node 5 and this audit) confirms terminal head/state `applied`; object closure is intact; completed-receipt replay is truthful and does not rewrite domain authorities; canonical dogfood preservation holds; runner SHA-256 `70301c87…` is bound to 64-character source manifest `c59e8c229bc31e71b8b5d5d5b1adfaed7defd2557d0286d06efdb74dc664a274` at Rust tree `510a185`, byte-identical to this HEAD.

There is no `block-m1` finding. `request-list` omitting `applied_head` is `park`. The 62-character manifest typo is superseded by `68a94b4`; see Follow-up.

Scope of this authorization: node 7 only — M1 exit, exit evidence in `history.md`, and the slot shift. It does not authorize starting any parked root, widening validation, projecting `applied_head` onto `request-list`, or beginning M2 implementation before the shift is recorded. This audit itself did not exit M1, did not shift any slot, did not edit the board, `history.md`, issues, or packets, and did not start parked work.

## Follow-up — docs-only delta at integration `68a94b4`

- time (ISO): `2026-08-22T01:47:02Z`
- integration HEAD (exact): `68a94b4bc9e1519a83ef5ada9575e92348f33cc8` (`68a94b4`, "docs: correct M1 runner manifest citation")
- this worktree merge: `fba21b19f5754785f1cbfcec4a089ab9d3fd150c`
- original audit blob: `35c9bff` tree-identical to integration `370f230`; `docs/development/ai/audit-latest.md` blob `a93a8014012d8019d015b4c2637a9400c6cb76ea` is unchanged through `68a94b4`

`git diff 74bbcd6..68a94b4` is three docs files only (audit integration, two run-packet hash quotes, node 6/7 board rows). No `crates/`, tests, schemas, Python CLI, evidence root, or canonical dogfood change. No exit fact moved. No broad re-audit.

Verified:

1. **Audit integrated unchanged.** `git diff --quiet 35c9bff 370f230` (full trees) and the `audit-latest.md` blob is identical at `35c9bff`, `370f230`, and `68a94b4`.
2. **Run packet manifest citation.** All three `c59e8c22…` claims in `milestone-remote-happy-path-run.md` at `68a94b4` are the exact 64-character provenance digest `c59e8c229bc31e71b8b5d5d5b1adfaed7defd2557d0286d06efdb74dc664a274`. Zero remaining 62-character truncations in that packet.
3. **Board nodes 6/7 only.** Graph and table advance node 6 `ready` → `completed` (`35c9bff` integrated as `370f230`) and node 7 `blocked` → `ready`. Nodes 1–5 rows are untouched in this delta. Remaining stale node-4 / "Last audited" / leftover graph-prose lines are pre-existing `park` finding 3, not new scope.

**`block-m1`: none.** Finding 2 is **superseded**. Finding 1 (`request-list` omits `applied_head`) is **retained `park`**.

**Authorized.** Node 7 may close M1 `MS-remote-happy-path` on integration HEAD `68a94b4bc9e1519a83ef5ada9575e92348f33cc8` with preserved evidence root `/tmp/m1-remote-happy-path-a3.VbZTZ8`. Same scope as the original close: slot shift and `history.md` only; no parked product work.
