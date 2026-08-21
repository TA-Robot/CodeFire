# Milestone audit

Final independent audit of active M1 `MS-local-sealed-loop` (node 5 of the M1 execution graph).

- time (ISO): `2026-08-21T22:36:37Z`
- audited HEAD (exact): `e7e792fcdce4598798597cc33e3bca8702758b56` (`e7e792f`, "docs: advance sealed-loop M1 to final audit")
- active milestone: M1 `MS-local-sealed-loop` — Daily local sealed-commit loop
- stalled: **no**
- scope_creep: **no**
- next_milestone_risk: low. M2 `MS-agent-local-contract` inherits one real, already-recorded gap — the Python `codefire` has no `--path`/`--json`, and `show` rejects `--path` (observed this audit) — which is exactly the M2 contract surface, not an M1 blocker.
- auditor: Claude Opus 5, new session, separate worktree `worktrees/m1-final-audit-20260821` on branch `claude-agent/m1-final-audit-20260821`. Nodes 3–4 were implemented by Grok 4.6. Independence per `orchestration-protocol.md` ("implementer Grok 4.6 → auditor Claude Opus 5") and `audit-cadence.md` is satisfied. Codex was not used.
- write set honored: this file only. `/workspace/project`, its CodeFire state, and every product/dogfood file were left unmodified; all project-side commands were read-only. `milestones.md`, `history.md`, and the parent packet were not edited. M1 was not exited and no slot was shifted.

This audit verified evidence against the environment. It did not accept packet prose as proof: every hash, envelope, and state claim below was re-derived independently at the audited HEAD.

## Exit criteria

Exact M1 exit from the card, in order.

| # | Exact exit criterion | Verdict | How it was verified independently |
|---|---|---|---|
| EC1 | one small real change goes `open → edit → scan → extinguish → verify → commit` in the canonical dogfood project | **pass** | Re-read the working-tree diff of the one target file (1 insertion, 1 deletion); confirmed the sealed commit object, its parent, the extinguished Fire, and the active resolution directly from the CodeFire object store. See EC1 detail below for the recorded `open` deviation. |
| EC2 | the resulting commit is sealed and the post-loop state is reported by CodeFire itself, not asserted by an agent | **pass** | Ran `show CF-COMMIT-373468dbaf827a6cc2bae9a4 --json` and `status --path "$DOGFOOD" --json` myself at audit time. CodeFire — not a packet — reported `certificate: "passed"`, `parents: ["CF-COMMIT-e5ca290905c2a4b5e2c05bd4"]`, and `state: "open-clean"` / `open_fires: 0` / `base: CF-COMMIT-373468dbaf827a6cc2bae9a4`. |
| EC3 | only the loop's focused validation was required to get there | **pass** | The recorded command set is exactly the card's list plus the one focused unittest plus the docs check. No `cargo test --workspace`, `cargo clippy`, `cargo fmt`, installer run, or dogfood Retrospect appears anywhere in the M1 record, and none was required to reach the seal. No Rust toolchain was invoked and nothing was built or installed. |

Required implementation for this slot, all four items:

| Item | Verdict | Independent verification |
|---|---|---|
| resolve which `codefire` executable runs the loop, and record it | **pass** | Runner exists at the recorded absolute path; `sha256sum` = `09d9fa2561de4b687200baa4b9fe54a4159e43def1a7a3cd28dc05c2741741e4`, matching the admitted hash exactly. |
| select the one small real change, by reading only, before the loop starts | **pass** | `milestone-local-sealed-loop-increment.md` is `completed` at `f52d489`, one commit before any runner admission or loop commit; the edit it specifies is byte-for-byte the edit later applied. Node 2 recorded no CodeFire state. |
| capture the pre-loop state with `status --json`, then run the loop on that change | **pass** | The verbatim pre-loop envelope (`base: CF-COMMIT-e5ca290905c2a4b5e2c05bd4`, `state: "open-clean"`, `open_fires: 0`, exit 0) is in the node 3 packet, and the parent (`e5ca2909…`) is confirmed by my own `show` of the sealed commit. Baseline correctly sat in node 3, after the fan-in, not in node 2. |
| record the loop evidence and the sealed commit identity | **pass** | Full envelopes for `status`/`scan`/`extinguish`/`verify`/`commit`/`status` with exit codes are in `milestone-local-sealed-loop-run.md`; the sealed identity and preservation outcome are in the parent packet and `history.md` row 1. |

### EC1 detail — the `open` stage

`open` was not invoked. The pre-loop `status --json` reported branch `main` already `open-clean` under existing open instance `open_4c320ed42560` (opened `2026-06-07T04:45:32Z`, marker `.codefire-open` sha256 `60b133b3eeefcc79e20bda32843261897713b0d6e86b2f0d0111cd19862690d6`, unchanged across the loop).

This is not a skipped exit-gate command. The M1 card's own validation list begins at `status`, not `open`, and the node 3 packet explicitly directs "Do not invoke `open` again against an already-open branch." The `open` stage was satisfied by a live open instance rather than a new invocation, and I confirmed the marker bytes are identical before and after. Stated here explicitly rather than folded silently into a pass.

## Evidence

- git: audited HEAD `e7e792fcdce4598798597cc33e3bca8702758b56`. M1 commit range `eab877f..e7e792f` (7 commits) touched **only** `docs/development/milestones.md`, `docs/development/history.md`, and the four `milestone-local-sealed-loop*.md` packets. No `crates/**`, no `codefire`, no `tests/**`, no `install.sh`/`pyproject.toml`/`setup.py`, no `adr/**`, no `subprojects/**`, and no CFB/CFR ledger file is in that range.
- packets in flight: parent `milestone-local-sealed-loop.md` (`in_progress`), children `-runner.md` (`completed`), `-increment.md` (`completed`), `-run.md` (`completed`). Node 5 was the only ready node; node 6 remains blocked and is orchestrator work.
- validation actually run **by this audit** (all read-only):
  - `sha256sum` of the admitted runner → matches `09d9fa25…`;
  - `build_source_manifest()` recomputed at the audited HEAD → **99 files**, `96737474cfec261cd9891d3d986e49f81601ef0ce4b7a25ce138526395801435`; recomputed again over the `/workspace/project` working tree → identical;
  - `git diff --quiet 54028a13 e7e792f -- Cargo.toml Cargo.lock crates` → exit 0;
  - `"$RUNNER" status --path "$DOGFOOD" --json` → exit 0;
  - `"$RUNNER" show CF-COMMIT-373468dbaf827a6cc2bae9a4 --json` → exit 0;
  - `PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=src python3 -m unittest tests.test_external_validity -v` → exit 0, 2 tests, `OK`;
  - `git status --porcelain` and targeted `sha256sum` in `/workspace/project`;
  - `python3 docs/development/tools/check_project_docs.py` and `git diff --check` for this audit's own docs change.

### Runner identity and source binding

| Claim | Recorded | Independently observed | Match |
|---|---|---|---|
| runner path | `/workspace/.cache/cargo-target-shared/source-keys/9c92559f…c861c6/release/codefire-rs` | present, 14457880 bytes, mode `-rwxr-xr-x` | yes |
| runner sha256 | `09d9fa2561de4b687200baa4b9fe54a4159e43def1a7a3cd28dc05c2741741e4` | `09d9fa2561de4b687200baa4b9fe54a4159e43def1a7a3cd28dc05c2741741e4` | yes |
| Rust source manifest | `96737474cfec261cd9891d3d986e49f81601ef0ce4b7a25ce138526395801435`, 99 files | recomputed at audited HEAD: same hash, 99 files | yes |
| build gate decision | `PASS`, `blockers=[]` | gate JSON `decision = PASS`, `blockers = []` | yes |
| build command | `cargo build --release --locked -p codefire-cli --bin codefire-rs` | gate `argv` identical | yes |
| toolchain | rustc/cargo 1.97.0 | gate `toolchain = 1.97.0-x86_64-unknown-linux-gnu` | yes |

The gate record `rust-release-build-ddcfad8a1d7b4b94fa4daf94-0.json` independently binds artifact to source: its `candidate` block records `rust_source_manifest_sha256 = 96737474…` over `rust_source_file_count = 99` on a clean tree (`dirty = False`, `untracked_file_count = 0`) at `head_revision 54028a136943a9d5f9608e19d3855c0edae46adf`, its `artifact_export.sha256` and `source_reread_sha256` both equal the admitted runner hash, and its `target_artifact_path` is exactly the admitted runner path. Since the same manifest recomputes identically at the audited HEAD and `git diff --quiet 54028a13 e7e792f -- Cargo.toml Cargo.lock crates` exits 0, **the binary that ran the loop corresponds to the Rust source at the audited HEAD**. Nothing was rebuilt, installed, or fetched for this audit.

### Loop envelopes, exit codes, Fire resolution

All six CodeFire commands are recorded verbatim with `schema: codefire.command_result.v1`, `ok: true`, `exit_code: 0`, empty stderr.

1. **pre-loop `status`** — exit 0, `base: CF-COMMIT-e5ca290905c2a4b5e2c05bd4`, `state: "open-clean"`, `open_fire_count: 0`.
2. **`scan`** — exit 0, `branch_state: "open-burning"`, `open_fire_count: 1`. Exactly one Fire, `FIRE-C7EE54F7CE66` (`fire_uid fire_sha256_c7ee54f7ce66631db3f769dd8b953d04`, `reason: atom_changed`, `severity: required`, `CODE-ExternalValidityEvaluator` → `REQ-SOTA-007`). `next_actions` were `context_changed`, `verify`, `context_fire`, `extinguish_fire` — none demanded a product change or a parked issue, so no stop rule fired.
3. **`extinguish`** — exit 0, on the exact scan-reported identifier, `--resolution addressed --rationale "External-validity implementation corrected"`, resolution `res_5b39746c46d0`.
4. **`verify`** — exit 0, `result: "passed"`, `decision.outcome: "passed"`, `open_required_fires: 0`, `blocking_reasons: []`. Seven `missing_required_links` are `severity: warning`, `blocking: false`, non-blocking under `trace_completeness_required: false` (`commit.require_trace_completeness` is `false` from `repository_config`). Next action `commit` only.
5. **`commit`** — exit 0, `applied: true`, `blocked: false`, `commit_id: CF-COMMIT-373468dbaf827a6cc2bae9a4`, `parents: [CF-COMMIT-e5ca290905c2a4b5e2c05bd4]`, `extinguished_fire_count: 1`, `changed_atoms: ["CODE-ExternalValidityEvaluator"]`.
6. **post-loop `status`** — exit 0, `base: CF-COMMIT-373468dbaf827a6cc2bae9a4`, `state: "open-clean"`, `open_fires: 0`.

Fire/Resolution semantics verified against the sealed objects themselves, not the envelopes:

- `objects/fire_ledgers/CF-FIRELEDGER-2f013d97ce80bcbfe3d3ab48.json` — `FIRE-C7EE54F7CE66` `status: "extinguished"`, `resolution_uid: res_5b39746c46d0`, source `content_hash_at_fire sha256:edb455a5…`.
- `objects/resolution_ledgers/CF-RESOLUTION-fa1c86ab6ecc03a45a879506.json` — `resolution_type: "addressed"` (**not** `no-change-required`), `rationale: "External-validity implementation corrected"`, `status: "active"`, basis pinning both source and target atom content hashes plus the trace link.

So `extinguish` did not clear a Fire that `scan` still had grounds to report — the resolution is bound to the post-edit content hash, and the post-loop scan prediction independently reports `changed_count: 0` / `open_fire_count: 0`. `verify` did not pass over an unextinguished Fire (`open_required_fires: 0` with the Fire already extinguished). No Fire was manufactured; exactly the one real scan-reported Fire was resolved.

### Sealed commit, independently confirmed at audit time

`"$RUNNER" show CF-COMMIT-373468dbaf827a6cc2bae9a4 --json` (exit 0) returned:

```text
commit:      CF-COMMIT-373468dbaf827a6cc2bae9a4
parents:     ["CF-COMMIT-e5ca290905c2a4b5e2c05bd4"]
message:     Fix unmapped benchmark transferability
certificate: passed
atoms: 1644   files: 183
roots: atom_index CF-ATOMINDEX-b71a5e64ec142eccd7a1a2d9 | content_manifest CF-MANIFEST-9a751af38cbb879f86212010
       fire_delta CF-FIRELEDGER-2f013d97ce80bcbfe3d3ab48 | policy CF-POLICY-c0e6d23d61488392f3589d70
       resolution_ledger CF-RESOLUTION-fa1c86ab6ecc03a45a879506 | trace_graph CF-TRACE-3ab0570e29aba10036a883f9
       verification CF-VERIFY-67bc19276962cf9f0b70351e
signature: (none)
```

The commit is genuinely sealed, not merely reported as sealed: it carries a full object root set including a verification certificate that reads `passed`, the parent is the pre-loop base, and the message is exactly the card's mandated fixed message. Branch head atomicity holds — `branches/main.json` reads `head: CF-COMMIT-373468dbaf827a6cc2bae9a4`, `state: "open-clean"`, with no partial-advance residue and no commit-transaction marker left behind. `signature: present false` is expected: `commit.require_commit_signature` is `false` (`built_in_default`, `optional_hardening`).

### Focused test

Reproduced by this audit, from the dogfood directory, with bytecode writing disabled so the audit left no artifact:

```text
test_external_validity_rejects_single_domain_gain ... ok
test_external_validity_requires_transfer_across_domains ... ok
Ran 2 tests in 0.001s — OK   (exit 0)
```

Identical to the recorded node 3 result. This is the focused test of the single module the increment touches, inside `subprojects/algorithm-evolution-agent-lab` — not the 281-test suite.

### Exact target-file content, and that no other tracked path changed

```text
working-tree sha256: 873aa601542dec4337f17d4f05779d86e26ebc36ae286dc76cc2d420fe10c2e7  (= recorded post-edit)
HEAD blob   sha256: 1e42de7df500e00f4be8d071de0a775dd3a2c2d018e927539106e23bab2c9df3  (= recorded admission, Git-clean precondition)
git diff --numstat: 1  1  subprojects/algorithm-evolution-agent-lab/src/evoagent/external_validity.py
```

The diff is exactly the increment node 2 selected by reading, one line replaced:

```diff
-        improved_domains = tuple(sorted({domain_by_benchmark.get(result.plan.benchmark, "unknown") for result in improvements}))
+        improved_domains = tuple(sorted({domain_by_benchmark[result.plan.benchmark] for result in improvements if result.plan.benchmark in domain_by_benchmark}))
```

`git status --porcelain -- subprojects/` at audit time returns exactly one line — that file. With `--untracked-files=all`, **zero** untracked paths exist under `subprojects/`. No second dogfood file, no new test, no stray artifact.

### Preservation boundary

| Preservation claim | Independently observed at audit time | Verdict |
|---|---|---|
| `/workspace/project` Git HEAD unchanged at `f1644122134d479333cf422fdb1d48b8fa92dd8e` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | pass |
| porcelain 245 → 246 lines (`113 M`/`132 ??` → `114 M`/`132 ??`) | 246 lines: `114 M`, `132 ??` | pass |
| only newly dirty tracked path is the exact target file | `git status --porcelain -- subprojects/` = that one path | pass |
| `.codefire-open` marker bytes unchanged | `60b133b3eeefcc79e20bda32843261897713b0d6e86b2f0d0111cd19862690d6` | pass |
| `repo.json` unchanged | `bf6d978b0eece0f185c6995630339326f56513a0c8645af3e7e984ad69a7f054` | pass |
| `opened/main.json` advanced to the sealed head | `f217d0cfabb97e3ed58938e0931decde41a85a65e8cdbdd1f715938f7e038c5c` | pass |
| `branches/main.json` head is the sealed commit | `7781697ea2a08a41f8bae9a9cf2217f154261d67e1fa4f94653f410a519fa339`, `head: CF-COMMIT-373468dbaf827a6cc2bae9a4` | pass |
| active dir reset to `state.json` only, still `open-clean` | only `state.json`, `{"state": "open-clean"}` | pass |
| all seven closed feature-branch files unchanged | all seven sha256 identical to the recorded list | pass |
| `.git/sequencer` leftovers untouched | `abort-safety` Aug 3, `head`/`todo` Jul 20 — all predate the loop | pass |
| stale `/workspace/algorithm-evolution-agent-lab` unused | no file newer than `2026-08-21 22:00`; loop ran 22:21–22:22 | pass |

The classified dirty tree was preserved, not adopted as a write set. The narrow exception the orchestrator admitted — one Git-clean, byte-matched dogfood file plus loop-written CodeFire state — is exactly what was used, and nothing more. This audit's own read-only commands left the same state: post-audit `status` still reports `open-clean` / `open_fires: 0` / `base CF-COMMIT-373468dbaf827a6cc2bae9a4`, porcelain still 246, target hash still `873aa601…`.

### Parked issues untouched

At the audited HEAD, all eight parked items are still `investigating`, with milestone labels matching the board exactly:

```text
CFB-133 investigating  M2      CFR-169 investigating  M2 (parked)   CFR-170 investigating  M2 (parked)
CFB-136 investigating  M3      CFR-168 investigating  M3 (parked)
CFR-171 investigating  M4 (parked)   CFR-172 investigating  M4 (parked)   CFR-173 investigating  M4 (parked)
```

No ledger file, index, or issue JSON packet was modified anywhere in `eab877f..e7e792f`. No new CFB/CFR was filed. None of CFB-133, CFB-136, or CFR-168..173 was started, expanded, or treated as an M1 blocker, and node 3 produced no dogfood evidence that would admit one: scan and verify next actions asked only for `context`, `verify`, `extinguish`, and `commit`. CFB-133 remains correctly not admitted.

### Validation not widened

The card's out-of-M1 list was respected in full. `cargo test --workspace`, `cargo clippy`, `cargo fmt`, the full installer, and dogfood Retrospect do not appear in any M1 packet, commit, or evidence record — and could not have run silently, since no `cargo`/`rustc` is on `PATH` and the loop used a pre-existing cached artifact with no build step. Node 1's and node 2's validation was the two-command docs check; node 3 ran the card's command list plus the one focused unittest; node 4 was docs-only. Nothing wider than the M1 card blocked or gated any step.

One transparency note, recorded as `drop` below: CodeFire's own `verify` executed the dogfood repository's configured verification command `PYTHONPATH=src python3 -m unittest discover -s tests -v` (`verification.command.unit-tests`, `source: repository_config`) as part of the card-named `verify` step, with `exit_code: 0`, `success: true`, and `subject_unchanged: true`. That is the product's internal gate behavior for a command the card names, not an agent widening validation, and it passed.

## Findings

Classified only as `block-m1`, `park`, or `drop`, per `milestone-operating-model.md`.

**`block-m1`: none.** No exit criterion is false at the audited HEAD.

`park` — valid, not required for M1 exit; both are board hygiene inside the orchestrator's own write set, which this audit may not edit:

1. **`park` → node 6 board refresh.** `milestones.md` line 96 states "At `f52d489` no CodeFire command has run against the dogfood project, no product or dogfood file has been edited, no focused test has run… there is no sealed commit". The sentence is true as scoped to `f52d489`, but it now sits inside the *current* execution-graph section directly under the progress lines, where it reads as a live claim that contradicts the sealed loop three lines above. Resolve during the node 6 board update.
2. **`park` → node 6 board refresh.** `milestones.md` line 11 anchors M1 state to "integration head `011d961`" while the audited head is `e7e792f`. Harmless (a board cannot cite its own commit), but the anchor should advance when node 6 writes the exit.

`drop` — out of boundary, duplicate, or noise; not filed:

3. **`drop`.** `verify` running the dogfood project's full configured unittest suite (see above). Product behavior of a card-named command, exit 0, not agent scope creep.
4. **`drop`.** Classifier commit `f963345` is cited on the board and in `history.md` but is not an ancestor of `e7e792f` — it branched from `1affb1f` and wrote only `docs/development/ai/audit-latest.md`, which `.gitignore:25` ignores. This is the auditor write set behaving exactly as designed, the commit is present and inspectable in the repository, its substance is restated at HEAD in `milestone-local-sealed-loop-runner.md`, and this audit independently re-derived its load-bearing claims (runner hash, 99-file manifest `96737474…`, gate `PASS`, toolchain 1.97.0). No defect.
5. **`drop`.** `show` rejects `--path` (`unsupported_option`, exit 2), so sealed-object inspection requires `cd` into the open directory. Real, but it is the M2 agent-local-contract surface, and the M1 card never names `show`.

## Advice (at most five bullets)

- Node 6 may proceed: run the M1 exit, record exit evidence in `history.md`, shift every slot forward one step, and name a new M5 from `strategy/modernization-roadmap.md` without turning that roadmap into a second board.
- Fold `park` findings 1 and 2 into that same node 6 board edit; do not open a separate packet for them.
- When M1 exits, admit only the M2 first increment already queued (`status`/`scan`/`verify`/`extinguish`/`commit` JSON, `next_actions`, and help sufficient for a worker), and admit CFR-170 or CFB-133 only to the extent that contract requires. Do not pull CFR-168 into M2.
- Keep all eight parked items `investigating` and move them with their slots; nothing in this M1's evidence admits any of them.
- Carry the two observed local-contract gaps — Python `codefire` lacking `--path`/`--json`, and `show` rejecting `--path` — into the M2 scoping conversation as inputs, not as M1 follow-up work.

## Close authorization

**Authorized.** M1 `MS-local-sealed-loop` may be closed by node 6 on the evidence at exact HEAD `e7e792fcdce4598798597cc33e3bca8702758b56`.

All three exact exit criteria are true, and each was verified against the environment rather than from packet prose: one small real change went through the loop in the canonical dogfood project; the resulting commit `CF-COMMIT-373468dbaf827a6cc2bae9a4` is sealed with a `passed` certificate and the correct parent, and CodeFire itself — re-queried by this auditor at `2026-08-21T22:36:37Z` — reports the post-loop state as `open-clean` with zero Fires; and only the loop's focused validation was required, with nothing built, installed, or widened. All four required-implementation items are recorded. The preservation boundary held, the parked list is untouched, and no product code outside the one admitted dogfood line changed.

There is no `block-m1` finding. The two `park` findings do not gate the exit and belong to the same node 6 board edit.

Scope of this authorization: node 6 only — M1 exit, exit evidence in `history.md`, and the slot shift. It does not authorize starting any parked root, widening validation, or beginning M2 implementation before the shift is recorded. This audit itself did not exit M1, did not shift any slot, did not edit the board, `history.md`, or the parent packet, and did not start parked work.
