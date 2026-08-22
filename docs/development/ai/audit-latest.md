# Milestone audit

Interim independent decision audit of the **first** adoption run of active M1 `MS-adoption-wedge`
(node 5b of the M1 execution graph). **This is not M1 exit authorization.**

- time (ISO): `2026-08-22T02:42:47Z`
- audited HEAD (exact): `11d0c8cc4721a5583fb8b657bbfc5692f31327f8` (`11d0c8c`, "docs: admit adoption B1 classification and decision audit")
- preserved roots audited (read-only): `/tmp/m1-adoption-root.d1YEhz` (handed consumer root), `/tmp/m1-adoption-ctrl.c23eQC` (controller), `/tmp/m1-adoption-key.JbqhPy` (sealed answer key)
- active milestone: M1 `MS-adoption-wedge` — First adoption wedge
- scope of this audit: first-run score, `undocumented_field_reliance`, the ordered `narrow` decision, the preservation boundary, and the B1 / single-rerun admission that node 5c is about to make
- stalled: **no** (see stall-risk flag F1 — the rerun as literally frozen is a guaranteed second `narrow`)
- scope_creep: **no** as executed (see scope flag F2 — the risk is at node 5d, not in what has run)
- next_milestone_risk: low. Nothing in this evidence admits CFR-168, CFB-136, CFB-133, CFR-169, CFR-170, a Git bridge, telemetry, an agent platform, or any M2–M5 work. Contract `park` P2 stands: the runner was handed to the consumer, so **M1 does not prove toolchain-free adoption** and no later slot may inherit that claim.
- auditor: Claude Opus 5, fresh session, separate worktree `worktrees/m1-adoption-decision-audit-20260822` on branch `claude-agent/m1-adoption-decision-audit-20260822`. Nodes 1, 4a, 4c and 5 were a persistent Claude session; node 4b was a fresh Grok 4.6 consumer; node 2 / node 5a are Grok. This session is not the run executor, did not use any prior agent conversation, and did not inspect any implementation worktree or executor uncommitted state. Provider routing per `provider-routing-policy.md` (individual and audit are Claude Opus 5 / Grok 4.6; Codex commands only) is satisfied. Codex was not used. Audit independence is same-family-as-controller but fresh-session; the *other-family* requirement in the board applies to node 6, which is still blocked.
- write set honored: **this file only.** No repository was mutated, no packet, board, issue, or product file was edited, no consumer rerun was performed, no threshold was changed, no implementation worktree was inspected, and no CodeFire command of any kind was issued by this audit — against the fixture, the canonical store, or anything else. All verification was filesystem digests, recorded envelopes, and `git`.

This audit did not accept packet prose as proof. Every hash, argv, envelope field, count, timestamp
bound, threshold clause, and preservation value below was re-derived independently from the preserved
roots and from `git` at the audited HEAD.

## 1. Requirement matrix — exact M1 exit bullets

The M1 card's five exit bullets, judged against the **first run only**. `n/a (interim)` means the
bullet cannot be satisfied by this node and is not claimed.

| # | Exact M1 exit bullet | Verdict | How it was verified independently |
|---|---|---|---|
| EC1 | consumer, exact public surface, task, falsifier, measurements, and stop rule selected **before any experiment mutation** | **pass** | Proven by `git`, not by assertion. The contract packet's Result — subject, C1–C6 table, preservation manifest, 7-item rubric, 10 metrics, ordered threshold table, falsifier — has exactly **one** version in history: commit `624f28c` at `02:03:14Z`. `git show f91c8ca:…contract.md` has no section 6 at all (empty-string digest `e3b0c442…`); `624f28c` introduced it (`cd880bff69435d78`) and no later commit touches the file. That precedes fixture creation (`$ROOT` mtime `02:06`), the answer key (`02:11:40Z`), the consumer's first command (`02:21:42Z`), and scoring (`7f7b86e`, `02:34:27Z`). |
| EC2 | one bounded inspect-or-import experiment completes, with before/after preservation hashes for every repository not explicitly admitted as an isolated target | **pass** | Five read-only invocations, all exit 0, all stderr 0 bytes. All 14 canonical values, all 3 prior evidence roots, and the fixture member digest reproduce exactly at audit time (§4). `$FIX` file count is still **2900** — not even a `locks/repo.lock` was created, so the C1–C6 path demonstrably holds no write authority. |
| EC3 | the consumer identifies repository, branch, base/head, claim/proof kind, and any missing authority **without source code or private clarification**, or the failure is retained as the terminal result | **pass on identification; the *contract-truthfulness* of the path is what B1 blocks** | All seven answers independently re-scored correct (§3). Source-blindness verified structurally (§2): the handed pack is exactly 12 files, and `crates`, `tests`, `docs/development`, `11_internal_architecture`, `08_object_store`, `09_index_trace_policy` have **0** path matches in it. No clarification was requested or given; 0 harness refusals during Stage B. But 7 of the 20 field paths carrying those answers are unpromised (§3.3), which is exactly what B1 admits. |
| EC4 | setup time, time to first useful result, misunderstanding/wrong-action count, and maintenance cost recorded with an explicit adopt / narrow / stop decision | **pass** | All ten metrics present and re-derived (§3.4). `misunderstanding_count 0`, `wrong_action_count 0`, `off_contract_invocations 0`. `maintenance_cost` is correctly labelled an **estimate, not a measurement**, proxied by `undocumented_field_reliance` + `docs_pack_files_read`. Decision `narrow` recomputed and confirmed (§3.5). |
| EC5 | an independent **other-family** audit verifies the evidence, preservation boundary, and absence of a hidden Git bridge, mutation authority, telemetry collection, or agent-orchestration dependency | **n/a (interim)** | This is node 6 and remains **blocked**. This audit is node 5b: same family as the controller, fresh session, and it does discharge the substantive absence checks (§5) — but it does **not** satisfy EC5's other-family requirement and does not claim to. EC5 can only be met after the rerun (node 5e), by a fresh other-family session. |

Required run artefacts for this slot:

| Item | Verdict | Independent verification |
|---|---|---|
| runner identity and HEAD binding | **pass** | `70301c874d7220a71be988562df0e5188ede6a85c65c63dcbd3dcdee09f4706c`, inode `1361045`, `14594080` bytes — all match. Provenance `d87653d0…`, schema `codefire.rust_build_provenance.v1`, `ok=true`, `base_revision 510a1855…`, `binary_sha256` matches the binary, 100 source files, manifest `c59e8c22…` (64 hex). `git diff --quiet 510a185 HEAD -- Cargo.toml Cargo.lock crates .cargo rust-toolchain rust-toolchain.toml` exits **0** at the audited HEAD, so the binary is the correct executable for `11d0c8c`. `cargo` and `rustc` are absent; no build ran. |
| label-only harness | **pass** | `bin/cf` = `83dc9d2581df5e6f4874a9670d6f934aa84beadce0e90ecb1c633d39bcc9dd32`, mode 555. Read in full: it exposes exactly `C1..C6` plus non-executing `labels`/`log`, refuses everything else with exit 64, and separates `PRESERVE=DRIFT` (71) from `PRESERVE=ERROR` (72). It contains **no** network primitive, **no** mutating CodeFire subcommand, and no path outside `$ROOT`/`$CTRL` in any executed argv. |
| answer key predates the answer | **pass** | Key `b5996cff…` mtime `02:11:40Z`, 7243 B; derivation bundle mtimes `02:10`. Consumer answer `187812d6…` mtime `02:25:54.995013949Z`, 5936 B. **0** files under `$KEY` newer than `02:17:00Z`. **0** files under `$CTRL` in the window `02:16:44Z`–`02:21:00Z`. `$CTRL/stagec/consumer-answer.frozen.json` is byte-identical **and** mtime-identical to the consumer answer (`cp -a`), so the frozen copy is authentic and untouched. |
| isolated run + distinct read-only confirmation | **pass** | Controller mirror is byte-identical to the handed transcript on all five envelopes. Node-5 confirmation (`02:30:44Z`, after scoring) is byte-identical to the consumer transcript for C2/C3/C4/`C5head`, and its `C5parent` matches the key's C5 (`2b0a2d71…`). Terminal facts are reproducible from the confirmation pass alone. |
| preservation of `/workspace/project` | **pass** | §4. No CodeFire command was ever issued against a canonical path, in any stage. |

## 2. Source-blind boundary

| Check | Expected | Observed | Verdict |
|---|---|---|---|
| handed docs pack | exactly the 12 frozen files, digest `218123042bb8b1a680deb7b06581afb13a225e7d7b85397fde693f2fa5896034` | all 12 per-file digests match the packet byte-for-byte; the sorted repo-relative rollup reproduces `21812304…` exactly | **pass** |
| product source / tests / internal docs in pack | 0 | `crates` 0, `tests` 0, `docs/development` 0, `11_internal_architecture` 0, `08_object_store` 0, `09_index_trace_policy` 0 | **pass** |
| answer leak in handed root | 0 | `TASK.md` read in full: seven questions, citation shape, public thresholds, **no answer**. No commit ID, no `codefire_show_result`, no `codefire_branch_list`, no `legacy-unversioned` anywhere in the handed pack | **pass** |
| key reachable from `$ROOT` | never | `$KEY` path appears nowhere in `$ROOT` or `$CTRL`; `setup-metadata.json` publishes only `answer_key_sha256` and states the path is withheld | **pass** |
| private clarification | none | none requested or given; harness has no channel to one | **pass** |
| harness refusals during Stage B | 0 | all **9** refusals in `$CTRL/evidence/refusals.log` are timestamped `02:09:37Z` (7, pre-handoff self-test) and `02:15:47Z` (2, smoke `verify` + second `C6`) — every one strictly before the `02:16:44Z` seal and ~5 min before the consumer's first command. No `refusals.log` exists in the handed transcript. | **pass** |
| read outside `$ROOT` | none evidenced | `seq=5`, no `HALT`, no `c6.used`. Every claim in the answer is reproducible from the five recorded envelopes alone. | **pass, with the packet's own honest limit upheld** |

**Boundary limit, restated rather than smoothed over:** `$CTRL` and `$KEY` share the consumer's uid, so
containment is *operational, not kernel-enforced*. The packet says this in §A.8/§C.2 and does not
overclaim. This audit re-checked the supporting evidence (key mtime/derivation bundle predate the run;
zero new files under `$KEY` after `02:17Z`; zero controller writes in the seal→start window; no `$KEY`
path reachable) and confirms it is **consistent with no outside read, with no evidence of one**. A
stronger claim is not available from this setup, and none is made here either.

## 3. Score, metrics, and decision — recomputed

### 3.1 C1–C5 exact argv / cwd / envelopes

Re-read from `$ROOT/evidence/transcript/*.argv` and `*.meta`:

| seq | label | argv after runner | cwd | start | dur | exit | stderr | stdout sha256 | vs frozen C-table |
|---|---|---|---|---|---|---|---|---|---|
| 1 | C2 | `doctor /tmp/m1-adoption-root.d1YEhz/repo --quick --json` | `$FIX` | 02:21:42Z | 1835ms | 0 | 0 B | `c9bfa51c…` | **exact** |
| 2 | C1 | `capabilities --json` | `$ROOT` | 02:22:00Z | 4ms | 0 | 0 B | `5d5245c3…` | **exact** |
| 3 | C3 | `branch list --path /tmp/m1-adoption-root.d1YEhz/repo --json` | `$FIX` | 02:22:00Z | 1644ms | 0 | 0 B | `9ba62a91…` | **exact** |
| 4 | C4 | `show main --json` | `$FIX` | 02:22:00Z | 1474ms | 0 | 0 B | `1c22cec2…` | **exact** |
| 5 | C5 | `show CF-COMMIT-2792b54ad730e6d644668c38 --json` | `$FIX` | 02:22:51Z | 1356ms | 0 | 0 B | `8ca6b540…` | **argument deviates** — see F4 |

`argv[0]` is `codefire` on every line; subcommands used are exactly `branch`, `capabilities`, `doctor`,
`show`; **0** occurrences of `/workspace/project` in any recorded argv. `C6` was never used. C1/C3/C4
share a start second because they were issued concurrently — nothing in the contract forbids it and all
three recorded independently.

Envelope facts re-read directly (not taken from the packet): `repo` is `/tmp/m1-adoption-root.d1YEhz/repo`
on C2/C3/C4/C5 and `null` on C1; C2 `data.type=codefire_doctor_report` v1, `mode=quick`,
`checked={objects:0,branches:8,opened:0,active_files:0}`, `skipped_checks=['object_store_integrity']`,
`issue_counts={blocking:0,total:7}`, and **all 7** `legacy_branch_head_semantic_profile` warnings are on
`feature%2F*` paths — `main` is excluded, which is the mixed state Q5/Q6 depend on; C3
`data.type=codefire_branch_list` v1 with 8 branches, `main` `state=open-clean`, 7 `feature/*` `closed`;
C4 `data.type=codefire_show_result` v1, `commit=CF-COMMIT-2792b54ad730e6d644668c38`,
`parents=["CF-COMMIT-373468dbaf827a6cc2bae9a4"]` (single), `certificate="passed"` (**bare string** — no
`subject`, no `profile`, no `phase`), `signature={present:false, details:null, summary:"(none)"}`;
C1 reports 12 formats, exactly the 5 checked-in `schemas/public` IDs, `unknown_version=deny`,
`unknown_critical_extension=deny`, `unknown_noncritical_extension=preserve`,
`semantic_validation_profile=sealed-snapshot-v1`, `independent-python-stdlib-v1` with
`read=true/write=false/role=read-only-conformance`, and `output_schema=codefire.command_result.v1` for
**all 34** commands including `show` and `branch`; the single `next_actions` entry
(`inspect_legacy_branch_profile`, on C2/C6 only) has keys `command, context, description, id, kind,
reason, target` and **no `execution` object**.

### 3.2 Answer / key / template / fixture hashes

| Artefact | Packet | Recomputed | Verdict |
|---|---|---|---|
| consumer answer | `187812d6d25d5699d6dc6ca847a474e3b73c6e86892ec28e8f915b38cfa6c1a3` (5936 B) | identical | **match** |
| frozen copy | same digest | identical digest **and** mtime | **match** |
| answer key | `b5996cff931da40afb272d535401accc1f312492a76788978cf45ae34b277392` | identical (7243 B) | **match** |
| derive script | `e1bd0822baac77197825335f52c055a4707764cfdaac5a29dd2ec06cf77e181b` | identical | **match** |
| answer template | `61916070ec5c04a9f1c0efdcdf6bbd7da91eff6e846e77fc1e3434b59b523ad0` | identical | **match** |
| task sheet | `89eb24aac6c7291078620d1c0e95b0335ad7dc403550b2844cf2a8dc68a0958f` | identical | **match** |
| harness | `83dc9d2581df5e6f4874a9670d6f934aa84beadce0e90ecb1c633d39bcc9dd32` | identical | **match** |
| setup metadata | `bcf40c0442a27d6d777434724597e55a1dba93a954310b9405ae26a548200c25` | identical | **match** |
| docs pack | `218123042bb8b1a680deb7b06581afb13a225e7d7b85397fde693f2fa5896034` (12 files) | identical | **match** |
| fixture members | `cccd8814a66dd3236c4a85e5752ab7bb6aa8ea09db2e74d35ab1a53998c07452` (2900 files / 114657290 B) | identical, and **byte-identical to the canonical members digest** | **match** |

Fixture shape re-verified: `opened`, `active`, `cache`, `idempotency`, `locks`, `remotes` all **0**
entries — a genuinely closed repository, and still 0 after five commands.

### 3.3 Q1–Q7 citations and score

Every cited path was resolved structurally against the recorded envelopes. **All resolve; none is
fabricated; every cited value matches what the consumer asserted.** Under the consumer's head-variant
C5, `C5 .data.show.commit` (cited for Q3) is the head and `C5 .data.show.parents` (cited for Q4) is
`["CF-COMMIT-373468db…"]`, so both Q3 and Q4 citations resolve correctly.

| Q | Key's accept clause | Consumer | Re-scored |
|---|---|---|---|
| Q1 | root exact, branches=8, opened=0; objects 0 from C2 (quick skips `object_store_integrity`) or 2891 from C6 | root exact, 8, 0, objects 0 with the quick-mode reason stated | **correct** |
| Q2 | all 8, no extras | all 8 decoded names, no extras | **correct** |
| Q3 | `CF-COMMIT-2792b54ad730e6d644668c38` | exact | **correct** |
| Q4 | `CF-COMMIT-373468dbaf827a6cc2bae9a4`, single parent | exact, single | **correct** |
| Q5 | `certificate=passed` **and** embedded/local rather than signed/portable, justified by `signature.present=false`; calling it signed/portable/legacy is wrong | `passed`; `embedded/local`; justified by `signature.present=false`, `details:null`, `summary:"(none)"`; additionally noted the 7 legacy warnings exclude `main` | **correct** |
| Q6 | any two distinct pool entries, each cited | two distinct pool entries (absent portable signed-commit authority; absent `sealed-snapshot-v1` proof on the 7 feature heads), each cited | **correct** |
| Q7 | must name `inspect_legacy_branch_profile` **and** state it is not ready / has no `execution` object | named it; `ready=false`; justified by the **absent `execution` object** rather than by a flag | **correct** |

**Score 7 / 7. Q5 and Q6 both correct. `misunderstanding_count 0`, `unanswerable_count 0`.** Confirmed.

The score is not a courtesy. The consumer's *semantic* claims were checked against the handed pack and
all are genuinely doc-grounded: `profile=local_verification_v1` / `phase=pre_commit`
(`05_consistency_model.md:135`, `04_cli_spec.md:529`), the `certificate.result` vocabulary and
"現行成功commitはpassedを返す" (`04_cli_spec.md:528`), and the `ready=true` / `codefire.action_execution.v1`
contract (`automation_interface.md:61,70`). It also did **not** claim a live open workspace, so the
`open-clean`-versus-`opened=0` trap recorded in the key did not fire. Its `claim_scope_explanation` is
accurate and appropriately narrow.

**`undocumented_field_reliance`, recomputed from scratch.** I enumerated every citation independently
and reproduce both of the packet's figures — they are two denominators of the same set, not a
contradiction:

```text
distinct label+path citation pairs                     25   (matches packet §C.3)
distinct label-agnostic field paths                    20   (matches packet §C.5 — this is the
                                                             contract's definition, "distinct JSON
                                                             field paths")
  documented in docs-pack or schemas/public            13   .repo; .data.checked.{objects,branches,
                                                             opened}; .data.mode; .data.skipped_checks;
                                                             .data.issues; .data.issues.0.kind;
                                                             .data.issue_counts.total; .next_actions;
                                                             .next_actions.0.{kind,command};
                                                             .next_actions.0.target.semantic_validation_profile
  UNDOCUMENTED, relied on for correct answers           7   .data.branches            (codefire_branch_list)
                                                            .data.show.commit         (codefire_show_result)
                                                            .data.show.parents
                                                            .data.show.certificate
                                                            .data.show.signature
                                                            .data.show.signature.present
                                                            .data.show.target
  undocumented payload families                         2   codefire_show_result, codefire_branch_list
```

Grep verification in the handed pack, run directly: `codefire_show_result` **0**,
`codefire_branch_list` **0**, `data.show` **0**, `data.branches` **0**, `show.commit`/`show.parents`/
`show.certificate`/`show.signature` **0** each. Controls confirm the scan works:
`codefire_doctor_report` 1, `codefire.capabilities.v1` 4, `next_actions` 29, `skipped_checks` 11,
`issue_counts` 1, `action_execution` 3, `semantic_validation_profile` 4, `command_result` 28.
`command-result-v1.schema.json` constrains `data` to exactly `{"type":"object"}`. The only `show --json`
sentence in the entire pack is `04_cli_spec.md:626` ("sealed commit summaryを返す"); `branch list --json`
is mentioned only to say `--metrics` adds `data.metrics` (`automation_interface.md:221`).

**The metric is decision-robust under every reading**: 7 (contract definition), 12 (label+path
denominator), or 2 (most generous family grouping). The threshold clause is `>= 2`. It fires in all
three. The consumer's self-reported **1** is an under-count — it grouped the whole `.data.show` family
as one entry and omitted `.data.branches` entirely despite citing it for Q2 and Q3 — and Stage C
correctly recorded that and did not use it. See F3.

### 3.4 Metrics and the observed-timing choice

| Metric | Packet | Re-derived | Threshold | Verdict |
|---|---|---|---|---|
| `setup_time_s` | 300 | 300 | — | consistent |
| `ttfur_s` | 300 | 300 | ≤ 900 | **pass** |
| `ttac_s` | 550 | 550 | ≤ 3600 | **pass** |
| `commands_issued` | 5 | 5 | — | consistent |
| `off_contract_invocations` | 0 | 0 | = 0 | **pass** |
| `wrong_action_count` | 0 | 0 | = 0 | **pass** |
| `misunderstanding_count` | 0 | 0 | — | consistent |
| `unanswerable_count` | 0 | 0 | — | consistent |
| `undocumented_field_reliance` | **7** | **7** | ≤ 1 | **FAIL** (decisive) |
| `docs_pack_files_read` | 12 (self-reported) | 12 listed, all 12 exist in the pack | — | consistent |

**The choice to score observed rather than self-reported timings is correct, and it cannot change the
decision.** The consumer's self-reported `started_utc 02:15:00Z` is 104 s *before* the handover seal and
its `submitted_utc 02:25:00Z` is 54 s *earlier* than the answer file's mtime — both impossible as
stated, so the observed values are the right ones to score. I also bounded the exposure rather than
trusting a single instant: the `02:16:44Z` seal is an `atime` and is **not** independently
re-derivable now, but durable mtimes bracket it to `[02:16:02Z, 02:21:42Z]` (last handed artefact
written `02:16:02Z`; last pre-handoff controller write `02:15:47Z`; first consumer command `02:21:42Z`;
zero writes anywhere in between). Across that entire bracket **and** under the self-report, `ttfur_s`
∈ [2, 404] ≤ 900 and `ttac_s` ∈ [252, 654] ≤ 3600. No timing clause in any row can flip. The timing
choice is therefore threshold-immune, and the decision does not rest on it.

### 3.5 Ordered decision table, re-applied

Contract section 6, first match wins, evaluated independently:

| Row | Decision | Clause evaluation (mine) | Match |
|---|---|---|---|
| 1 | stop (terminal) | preservation `PRESERVE=OK` at every checkpoint and at audit time; `wrong_action_count = 0`; no private clarification; no evidenced read outside `$ROOT` | **no** |
| 2 | stop (terminal) | score 7 > 3; `ttac_s` 550 ≤ 7200 (and ≤ 7200 across the whole timing bracket) | **no** |
| 3 | **narrow** | clause 1 false (Q5 **and** Q6 both correct); **clause 2 TRUE — `undocumented_field_reliance` 7 ≥ 2**; clause 3 false (550 ∉ (3600, 7200]) | **MATCH** |
| 4 | adopt | not reached. Every other clause would have passed — score 7/7, Q5+Q6 correct, `ttfur_s` 300 ≤ 900, `ttac_s` 550 ≤ 3600, `off_contract 0`, `wrong_action 0` — but `undocumented_field_reliance` 7 > 1 fails it independently | not reached |
| 5 | narrow | fallback, not reached | — |

**Decision `narrow` — confirmed.** The thresholds were provably frozen before any data existed (EC1),
so this is not a post-hoc fit. A perfect 7/7 correctly does **not** clear the bar: five of the seven
answers, including both milestone-critical ones, rest on payloads the project has never promised to
keep true. That is the honest result rather than the flattering one, and the precommitted metric — not
the auditor's judgement — is what resolved it against `adopt`.

## 4. Preservation and root integrity — re-derived at audit time

All values recomputed now, at HEAD `11d0c8c`:

```text
git_head                f1644122134d479333cf422fdb1d48b8fa92dd8e            match
git_porcelain_count     248                                                 match
repo_json               bf6d978b0eece0f185c6995630339326f56513a0c8645af3e7e984ad69a7f054   match
branches_main_json      bfd52e3525f6a6faf0db4f3d2d1d9d7fa3fa0d400f6561c3c5b8227e79e989f8   match
branches_rollup         8fba5dcbedd5179a27f1a8d3ce699db2c4e62f94521084ff015e025fcc582841   match
opened_main_json        5721d710f4371db88be26567d2a3c6afb3bc2458288b8c812b54ccd7556a42b2   match
active_state_json       54ecb285704644b814e4ca47a711bf5eba150ee747642b90b53351b2f06940d2   match
codefire_open_mark      60b133b3eeefcc79e20bda32843261897713b0d6e86b2f0d0111cd19862690d6   match
objects_file_count      2891                                                match
objects_bytes           114921765                                           match
no_codefire_server      0                                                   match
store_all_files_digest  e5f6f7c2ac192934a0528414b54f1c2e097dfb65ec8c60a4f95b2c73251c41a6   match
store_file_count        2904                                                match
store_bytes             114968174                                           match  (du -sb; see F6)
prior root .TaHMir      877461024a3deae974ff3b415a3cab8e953112c87d98540a91636b1d31139f40   match (2919 files)
prior root a2.zvrbzn    d2ee53b301e0fc8514be108e6e287f2f498a1d5fc7b48307a7b011f53ff823be   match (5360 files)
prior root a3.VbZTZ8    3fdd2867655f8e455a9269d8f9104f127d65e49034fdae292f5f3d6780b1a4ad   match (5381 files)
fixture_members         cccd8814a66dd3236c4a85e5752ab7bb6aa8ea09db2e74d35ab1a53998c07452   match (2900 files)
```

**`PRESERVE=OK`.** Canonical `/workspace/project`, the open dogfood worktree, all three prior-milestone
evidence roots, and the fixture are byte-identical. This audit wrote nothing to any of them. The
stricter-than-remote rule held: **no CodeFire command was issued against a canonical path at any point**,
in any stage, by anyone — the run's preservation checks are pure `find`/`sha256sum`/`git`, which is why
no repository lock was taken and no `repo.lock.owner.json` was republished.

## 5. Absence checks

| Prohibited thing | Verdict | Independent basis |
|---|---|---|
| Git bridge / Git compatibility writer | **absent** | No `git` in any CodeFire argv; no `clone`, `import`, `merge`, `upload`, `request-*`. The only `git` in the whole setup is the harness's own read-only `rev-parse HEAD` / `status --porcelain` against `/workspace/project` as a *preservation check* — a controller-side digest, not a product feature, not a bridge, and it writes nothing. |
| mutation authority | **absent** | Exact-token scan over all five `*.argv` for `init import open close discard clone scan fire extinguish verify commit merge upload request-merge request-review request-apply gc serve evidence link patch migrate --idempotency-key`: **zero hits**. Positive proof, not just argv absence: `$FIX` is still 2900 files with **0** entries in `locks/` after five commands. The harness has no code path that runs a mutator. |
| telemetry collection | **absent** | All measurement comes from harness-written local files (`.argv/.json/.stderr/.meta/index.tsv`) and filesystem timestamps. No instrumentation was added to CodeFire, no counters are transmitted, nothing leaves local disk. |
| network use | **absent** | The harness contains no network primitive of any kind (read in full). No remote, no `cf://`/`cf+http://`, no `serve`, no `.codefire-server` under `/workspace/project` (0). |
| private clarification | **absent** | No channel existed; none requested; 0 refusals during Stage B; the answer is fully reproducible from the five recorded envelopes. |
| source / test / internal-doc reads | **absent** | §2 path scan: 0 matches for `crates`, `tests`, `docs/development`, `11_internal_architecture`, `08_object_store`, `09_index_trace_policy` in the handed pack. |
| agent-platform / agent-orchestration dependency | **absent** | The consumer was one plain session with no subagents, no web, and a shell script as its only tool. Nothing in CodeFire was changed to support agents; no orchestration surface, marketplace, or prompt machinery is involved in the measured path. |
| product code / schema / test change in this run | **absent** | The run packet, the contract packet, and the B1 packet are the only files touched by nodes 1–5; `git log` for each confirms it. No `crates/**`, `tests/**`, or `schemas/**` change exists in `3c0be0d..11d0c8c`. |

Honest scope limit carried forward, not buried: contract `park` **P2**. The audited binary was *handed*
to the consumer; `cargo`/`rustc` are absent in this environment and no prebuilt artefact is published,
so **M1 does not prove toolchain-free adoption.** That belongs to M2 `MS-release-operational-trust` and
no later slot may inherit the stronger claim.

## 6. Adjudication: is B1 genuinely the only admissible next slice?

**Verdict: yes.** B1 — *document and machine-bind the sealed claim/proof and branch-head fields that
`show --json` and `branch list --json` already emit* — is the only admissible next slice, and it is
`block-m1`. Five independent grounds:

1. **It is the measured blocker, not a preferred one.** The only failing clause in the entire threshold
   table is `undocumented_field_reliance`, and every one of the 7 paths belongs to exactly these two
   payload families. Nothing else in the run failed anything.
2. **Both lanes converged on it before the data existed.** Node 1 raised it from documentation absence
   and explicitly recorded the counter-argument for adjudication rather than burying it. Node 2 —
   reading source, independently — named the *same* smallest candidate (`show_commitish_data`
   projecting the existing certificate object, parse/render only, no format change) and correctly left
   it **unadmitted**, listing "treating lossy `show.certificate` as a pre-experiment `block-m1`" as a
   scope-creep risk. The measurement is what promoted it. That is the right order.
3. **`branch list` cannot be dropped from the slice.** I tested whether Q2 could be answered from
   documented carriers alone: it cannot. C2's warnings name only the 7 legacy `feature/*` branches (not
   `main`), and `checked.branches=8` gives only a count. The sole complete branch-name carrier is
   `C3 .data.branches`, and reconstructing the set otherwise still requires the undocumented
   `.data.show.target`. So the two-command scope is minimal **and** sufficient — not padded.
4. **The slice needs no persistence, format, or semantic change**, which is what keeps it admissible
   under the B1 packet's own Q5 stop-condition. The *semantics* are already public prose:
   `04_cli_spec.md:528-529` and `05_consistency_model.md:135` state that a commit's certificate carries
   the assessment `subject`, `profile=local_verification_v1`, and `phase=pre_commit`, enforced by the
   sealed validator. The *values* are already sealed in the objects. What is missing is only the
   machine-readable binding of one to the other. This audit adds the sharper form of the finding:
   **`show --json` — the only public read-only window onto a sealed commit on this path — lossily
   projects that documented certificate object down to the bare string `"passed"`.** So a consumer who
   reads the documented model correctly still cannot retrieve the documented fields through the only
   surface it has. That elevates B1 from "no promise was made" to "a promise was made in prose and is
   not kept in the JSON projection" — and it is still fixable as projection/schema/docs only, by either
   projecting `subject`/`profile`/`phase` in `show_commitish_data` or explicitly documenting that
   `show` returns `certificate.result` alone and naming the surface that returns the full object.
   Node 5a must pick one; both stay inside the slice.
5. **Every alternative is strictly larger or off-path.** P4 (no command's `data` has a schema) is the
   general form and is correctly parked. P1 (Python has no `--json`), P3 (`context --branch`
   divergence), and P5 (`request-list` `applied_head`) are not on the selected path and were not
   reproduced by this run. D1–D3 were rejected as subjects with reasons at node 1 and nothing here
   reopens them. The full CFR-171 cross-surface descriptor, a new command, an adapter, a store-validator
   change, and a broadened rubric all remain **out**.

## 7. Adjudication: is a single identical rerun after B1 authorized?

**Verdict: authorized in principle — exactly one — but node 5c may NOT admit it as byte-identical.**

The contract's frozen `narrow` definition authorizes precisely: *admit only B1, re-run this identical
experiment once with a fresh consumer, then decide adopt/stop on the same table. No second consumer, no
new command, no broadened rubric.* That authorization stands and this audit does not touch it.

But the rerun as **literally** frozen is unsatisfiable, and this is the single most important thing
node 5c must resolve. `undocumented_field_reliance` is defined against **the handed docs pack**. If the
pack is reproduced byte-for-byte (`218123042bb8…`, the same 12 filenames), then the 7 relied-on paths
are still absent from it *by construction*, the metric is still ≥ 2, row 3 matches again, and the rerun
returns `narrow` a second time **regardless of how good the B1 fix is**. That is not an experiment; it
is a loop, and it is the concrete mechanism by which M1 would stall (F1).

Three deviations are therefore *necessary*, and node 5c must record each one explicitly as the price of
the rerun:

1. **The docs pack must be re-copied from the post-B1 HEAD**, and its `docs_pack_digest` re-derived.
   The new digest must be recorded as a deliberate, admitted change — not as drift.
2. **The frozen 12-file pack list must be extended by exactly the new public schema file(s)** the B1 fix
   adds (e.g. a `schemas/public/*show*`/`*branch*` schema). A schema the consumer is never handed cannot
   lower a metric defined over the handed pack; conversely, if the metric is instead read against the
   repository's `schemas/public/**`, the consumer would be scored against a document it was forbidden to
   see. Node 5c must state which reading governs — the coherent one is the handed pack.
3. **Stage A's answer-leak gate must be re-specified.** The first run gated on **0** hits for
   `codefire_show_result` / `codefire_branch_list`-style type names in the pack. After B1 those tokens
   *must* appear — that is the entire point of the fix — so the gate must narrow to the answer *values*
   (`2792b54a…`, `373468db…`, the branch-name set, `inspect_legacy_branch_profile`) and drop the type
   names. Left as-is, Stage A's own gate fails and the rerun cannot even launch.

Everything else must stay frozen and byte-for-byte: the fixture members digest `cccd8814…`, the C1–C6
argv/cwd table, the seven questions and citation shape, the key-derivation method, the ten metric
definitions, the ordered threshold table, the preservation manifest, and the falsifier. Additionally:
the rerun consumer should be a **fresh Grok 4.6** session — the same family as the first consumer, so
that the only changed variable is the documentation. Using a different family would confound the B1
effect with a model change and would make the before/after uninterpretable. ("Other-family" in the
board's node-5e row means other-family *from the persistent Claude controller*, which resolves to Grok;
node 5c should say so explicitly rather than leave it ambiguous.) Node 6's other-family audit
requirement is separate and unaffected.

## 8. Findings

### `block-m1` (2)

1. **B1 — `show --json` and `branch list --json` `data` payloads are undocumented and unschema'd, and
   they are the only public carriers of head, base, claim/proof kind, and signature authority on the
   selected path.** Verdict **upheld** as the standing and only admissible slice (§6). Strengthened by
   this audit: it is not merely an absence of documentation but a **lossy projection of a documented
   contract** — `04_cli_spec.md:528-529` and `05_consistency_model.md:135` promise a certificate
   carrying `subject`/`profile=local_verification_v1`/`phase=pre_commit`, while `show --json` emits
   `certificate` as the bare string `"passed"`. Fix stays projection/schema/docs only. Not the full
   CFR-171 descriptor, not a new command, not a store-validator or format change.

2. **F1 — the frozen "identical rerun" is unsatisfiable as written and would deterministically return a
   second `narrow`, stalling M1.** `undocumented_field_reliance` is defined over the handed docs pack;
   reproducing that pack byte-for-byte guarantees the metric stays ≥ 2 no matter how good the B1 fix is,
   and Stage A's own leak-scan gate (0 hits for `codefire_show_result`) would additionally fail once B1
   lands. `block-m1` because it blocks the node-5c admission itself, not the completed run. Resolution
   is the three recorded deviations in §7 — refresh the pack, extend the file list by the new
   schema(s), re-specify the leak gate — with everything else frozen. **Node 5c must not admit a
   byte-identical rerun.**

### `park` (5) — real, not to be worked now

3. **F2 — B1's write-set boundary on `branch list` `.state` is undecided, and deciding it wrong turns
   B1 into a behavior change.** `C3` reports `main` `state=open-clean` while `C2`/`C6` report `opened=0`
   and `active_files=0`, and `doctor` does not flag the combination. This consumer avoided the trap; the
   next one may not. If B1 documents `.state`, it must be documented as *the persisted branch-record
   state, which is not live-open truth*, or excluded from the admitted guarantee. If node 5a instead
   proposes changing the value, that is a behavior change and the B1 packet's own Q5 requires it be
   classified `block-m1` and stopped. Park the fix; **node 5c must record which of the three it chose
   before node 5d writes anything.** The B1 packet's Q3 already has the hook for this.

4. **F3 — the deciding metric's self-report was elicited from a subject that had been told the cutoff.**
   `TASK.md` §Thresholds discloses "at most 1 undocumented field relied on" (adopt) and "≥ 2" (narrow),
   and the consumer self-reported exactly **1** — the only value preserving `adopt` — by collapsing the
   whole `.data.show` family into one entry and omitting `.data.branches`, which it had cited for Q2 and
   Q3. Mechanically it is 7. **No impact on this decision**: Stage C computed the metric controller-side
   and explicitly refused the self-report. But the disclosure cannot be removed from `TASK.md` without
   breaking rerun identity, so the mitigation must instead be *mandatory and repeated*: the rerun's
   `undocumented_field_reliance` must again be computed mechanically controller-side, and the
   self-reported figure must again be recorded but not scored.

5. **F4 — C5 as executed did not exercise the "base" surface it was frozen to exercise.** The frozen
   table specifies `show <parent-commit-id> --json`; the consumer passed `main`'s head, which the
   harness permits (it validates only `^CF-COMMIT-[0-9a-f]{12,24}$`), making C5 informationally
   redundant with C4. Correctly recorded as not off-contract and correctly not penalised — and
   **decision-neutral**: Q4 resolves from `C4 .data.show.parents` alone, and had it been counted as an
   off-contract invocation it would only have failed row 4, which was already failed. For the rerun,
   prefer accepting the redundancy (tightening the harness would itself break identity) but state
   explicitly that C5's independent value is optional.

6. **F5 — P2 unchanged: M1 does not prove toolchain-free adoption.** The runner was handed to the
   consumer; no prebuilt artefact is published and this environment has no `cargo`/`rustc`. Belongs to
   M2. No later slot may inherit the stronger claim. (Also unchanged and still parked: P1 Python has no
   `--json` on any command, P3 `context --branch` doc/help divergence, P4 the general no-`data`-schema
   form of B1, P5 `request-list` omitting `applied_head`.)

7. **F6 — digest *methods* are not recorded beside the digests, which invites false drift reports.**
   `store_bytes 114968174` is `du -sb` (apparent size, directories included), not a sum of file sizes
   (114660898); the prior-root rollups require **absolute**-path `sha256sum` input, and re-deriving them
   with repo-relative paths yields entirely different digests. Both reproduce exactly once the method
   matches, and the harness hardcodes the correct form internally — but the numbers as published in the
   packet are method-dependent and a later auditor can easily read spurious drift. Record the exact
   command next to each digest in the rerun's preservation manifest.

### `drop` (2)

8. **The packet's "25 distinct cited field paths" (§C.3) vs "20" (§C.5) is not a discrepancy.** They are
   distinct label+path pairs (25) and distinct label-agnostic paths (20). I reproduced both exactly. The
   contract's metric definition ("distinct JSON field paths") is the 20 denominator, giving 7
   undocumented. Recorded only so node 5c does not read it as a contradiction. No action.

9. **Reopening D1–D3 (conformance-fixture-only consumer, prior evidence root as subject, freshly
   imported toy repository) or promoting P4/P1/P3/P5 into this slot.** Nothing in this run's evidence
   supports any of them, and each is strictly larger than or off the measured path. Drop.

## 9. Authorization for node 5c

**Authorized:**

1. Admit **B1** as scoped in §6 — document and machine-bind the claim/proof and branch-head fields
   `show --json` and `branch list --json` already emit, as projection / schema / docs only, and decide
   §6 ground 4 (project `subject`/`profile`/`phase`, or explicitly document that `show` returns
   `certificate.result` alone and name the surface carrying the full object).
2. Admit **exactly one** identical fresh-consumer rerun — **conditional on recording the three
   necessary deviations in §7** (refreshed docs pack + re-derived digest; pack file list extended by
   exactly the new public schema(s); Stage A leak gate re-specified to answer *values*, not type names),
   with the fixture digest, C1–C6 argv/cwd, seven questions, key-derivation method, ten metric
   definitions, ordered threshold table, preservation manifest, and falsifier all unchanged.
3. Record F2's choice for `branch list` `.state` **before** node 5d writes anything.
4. Name the node-5d context owner and the node-5e consumer (fresh **Grok 4.6**, same family as the first
   consumer so the only changed variable is the documentation) and controller.
5. Carry F3's mandatory mitigation (controller-side mechanical recount of
   `undocumented_field_reliance`) into the rerun packet.

**Refused:**

1. **M1 exit or any slot shift.** The decision is `narrow`, not terminal. Only the post-B1 rerun can
   produce `adopt`/`stop`, and node 6's other-family audit (EC5) has not run.
2. **A byte-identical rerun admission** — F1. It would guarantee a second `narrow` and stall M1.
3. **Any change to the section-6 thresholds, the rubric, the metric definitions, the C1–C6 set, or the
   fixture topology.** They were provably frozen at `624f28c` before any data existed; that proof is
   the run's main defence and must not be spent.
4. **A second consumer, a broadened rubric, a new command, an adapter, a Git bridge, telemetry, an agent
   platform, a store-validator or sealed-format change, or the full CFR-171 cross-surface descriptor.**
5. **Any product write beyond the admitted B1 slice**, and any behavior change to `branch list` `.state`
   or to claim semantics (that would be `block-m1` and stop, per the B1 packet's Q5).
6. **Reopening D1–D3 or promoting P1/P3/P4/P5** into this slot.
7. **Deleting any preserved root** before node 6 has copied the decision record.

Issue placement is unchanged by this audit: CFR-171 admitted at the narrowest slice (B1) only; CFR-172
experiment executed with a non-terminal `narrow`; CFR-173 bounded slice satisfied for this run;
CFR-168, CFB-136, CFB-133, CFR-169, CFR-170 unchanged and backlog-parked. All ledgers stay
`investigating`.

## 10. Validation run by this audit

```bash
python3 docs/development/tools/check_project_docs.py     # passed; 552 Markdown files inventoried
python3 docs/development/tools/check_ai_acceleration_docs.py  # passed
git diff --check                                         # clean
```

No build, no product test, no CodeFire command of any kind, no import, no Git bridge, no live workspace
command, no dogfood mutation, no consumer rerun, no threshold change, and no later-slot gate. Write set
is this file only.

## 11. Verdict

**Interim verdict: the first adoption run is sound and its `narrow` decision is confirmed. M1 is NOT
authorized to exit.**

The source-blind boundary held, the preservation boundary held byte-for-byte across the canonical store,
the dogfood worktree, three prior evidence roots and the fixture, the answer key provably predates the
answer, the thresholds provably predate the data, the score is genuinely 7/7, and
`undocumented_field_reliance` is genuinely 7 under the contract's own definition and ≥ 2 under every
alternative reading. Row 3 of the frozen table is the correct and only match. There is no Git bridge, no
mutation authority, no telemetry, no network, no private clarification, no source/test/internal-doc
read, and no agent-platform dependency anywhere in the measured path.

Two things block node 5c from proceeding as currently framed. **B1 is confirmed as the only admissible
next slice** — and it is a stronger finding than first stated, because `show --json` lossily projects a
certificate contract the public docs already promise. **The rerun cannot be admitted byte-identically**
— doing so would return `narrow` a second time by construction and stall M1. Resolve F1 with the three
recorded deviations, record F2's `.state` choice before writing code, and carry F3's mechanical recount
forward; then the bounded fix and the single rerun are authorized.
