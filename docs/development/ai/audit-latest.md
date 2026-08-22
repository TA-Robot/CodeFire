# Milestone audit

Final independent **other-family** audit of node 6 for active M1 `MS-adoption-wedge`
(post-B1 rerun). This is the EC5 audit. It authorizes or blocks M1 exit.

- time (ISO): `2026-08-22T03:41:30Z`
- audited HEAD (exact): `a805c222097fc25a812dd7accd0ad2b7645e1ada` (`a805c22`, "docs: score post-B1 adoption rerun as terminal adopt")
- preserved roots audited (read-only):
  - rerun handed `$ROOT2` `/tmp/m1-adopt2-root.N622xo`
  - rerun controller `$CTRL2` `/tmp/m1-adopt2-ctrl.hx1vCY`
  - rerun key `$KEY2` `/tmp/m1-adopt2-key.3Qj9MY`
  - first-run `$ROOT` `/tmp/m1-adoption-root.d1YEhz`, `$CTRL` `/tmp/m1-adoption-ctrl.c23eQC`, `$KEY` `/tmp/m1-adoption-key.JbqhPy`
  - remote-milestone roots `/tmp/m1-remote-happy-path.TaHMir`, `/tmp/m1-remote-happy-path-a2.zvrbzn`, `/tmp/m1-remote-happy-path-a3.VbZTZ8`
- active milestone: M1 `MS-adoption-wedge` — First adoption wedge
- scope of this audit: node 6 only. Post-B1 score vs key `cea8bee0…`, citation resolution,
  mechanical `undocumented_field_reliance` against the refreshed 14-file pack, ordered
  adopt/narrow/stop table without threshold change, one-rerun limit, C2–C5 byte-identity,
  runner/provenance binding to `c78d5e0e`, preservation including first-run digests, absence
  of Git bridge / mutation authority / telemetry / toolchain-free claim / agent-orchestration
  in the consumer path, and whether the two parked comprehension hazards or the Q5 reasoning
  imprecision are `block-m1`.
- stalled: **no**
- scope_creep: **no**
- next_milestone_risk: low. Nothing in this evidence admits a Git bridge, telemetry, an agent
  platform, full CFR-171, CFR-168/CFB-136/CFB-133/CFR-169/CFR-170, or any M2–M5 work. Contract
  `park` P2 stands: the runner was handed to the consumer, so **M1 does not prove toolchain-free
  adoption** and no later slot may inherit that claim.
- auditor: Grok 4.6, fresh session, separate worktree
  `worktrees/m1-adoption-final-audit-20260822` on branch
  `grok-agent/m1-adoption-final-audit-20260822`. The persistent fixture/controller lane is
  Claude; node 5b was Claude; both consumers were Grok. This session is not the run executor,
  did not use any prior agent conversation, and did not inspect any implementation worktree or
  executor uncommitted state. Provider routing is satisfied. Codex was not used. Other-family
  vs the Claude controller and vs the node-5b Claude auditor is satisfied.
- write set honored: **this file only.** No product, source, schema, packet, board, issue,
  fixture, or consumer-answer file was edited. No mutating CodeFire command was issued. This
  audit issued **no CodeFire command of any kind** — against the fixture, the canonical store,
  or anything else. All verification was filesystem digests, recorded envelopes, schema/docs
  greps, and `git`.

This audit did not accept packet prose as proof. Every hash, argv, envelope field, count,
timestamp bound, threshold clause, and preservation value below was re-derived independently
from the preserved roots and from `git` at the audited HEAD.

**Terminal verdict: ACCEPT. `block-m1`: none.**

## 1. Requirement matrix — exact M1 exit bullets

Judged against the **post-B1 rerun** plus retained first-run evidence. The first run remains
frozen history (`narrow`); this node scores the single authorized retry.

| # | Exact M1 exit bullet | Verdict | How it was verified independently |
|---|---|---|---|
| EC1 | consumer, exact public surface, task, falsifier, measurements, and stop rule selected **before any experiment mutation** | **pass** | Contract Result sections 1–6 have exactly one version in history: `624f28c`. No later commit touches `milestone-adoption-wedge-contract.md`. Rerun `TASK.md` hard rules, seven questions, citation shape, and public thresholds are **byte-identical** to the first-run task sheet; only the docs-pack listing changed (two added schema paths + sort), which the rerun amendment authorized. Thresholds were not edited after seeing either run's data. |
| EC2 | one bounded inspect-or-import experiment completes, with before/after preservation hashes for every repository not explicitly admitted as an isolated target | **pass** | Five read-only invocations, all exit 0, all stderr 0 bytes. All 14 canonical values, 3 remote roots, 4 first-run digests, and the fixture member digest reproduce at audit time (§5). `$FIX2` file count is still **2900** with empty `opened/active/cache/idempotency/locks/remotes` — not even a `locks/repo.lock` was created. |
| EC3 | the consumer identifies repository, branch, base/head, claim/proof kind, and any missing authority **without source code or private clarification**, or the failure is retained as the terminal result | **pass** | All seven answers independently re-scored correct against key `cea8bee0…` (§3). Source-blindness verified structurally (§2): the handed pack is exactly 14 files matching HEAD public bytes; `crates`/`tests`/`docs/development`/`11_internal_architecture`/`08_object_store`/`09_index_trace_policy` have **0** path matches in the pack. No clarification was requested or given; 0 harness refusals during Stage B. Mechanical `undocumented_field_reliance` is **0** against the 14-file pack, so the answers are contract-truthful, not only factually correct. |
| EC4 | setup time, time to first useful result, misunderstanding/wrong-action count, and maintenance cost recorded with an explicit adopt / narrow / stop decision | **pass** | All ten metrics present and re-derived (§3.4). `misunderstanding_count 0`, `wrong_action_count 0`, `off_contract_invocations 0`. `maintenance_cost` remains an estimate proxied by `undocumented_field_reliance=0` + `docs_pack_files_read=14`. Decision `adopt` recomputed on the original ordered table with **no threshold change or coercion** (§3.5). |
| EC5 | an independent **other-family** audit verifies the evidence, preservation boundary, and absence of a hidden Git bridge, mutation authority, telemetry collection, or agent-orchestration dependency | **pass (this audit)** | This is node 6. Other-family vs the Claude controller and the Claude node-5b auditor. Absence checks in §6. |

Required run artefacts:

| Item | Verdict | Independent verification |
|---|---|---|
| runner identity and HEAD binding | **pass** | Binary `/workspace/worktrees/ms-adoption-experiment-20260822/target/release/codefire`, sha256 `90cb2895aadc45b559a96207cdd5ae3b08485f9a7e5a07014db8cca8b5b90ce9`, 14590776 bytes, inode `1374942`, nlink 3. Provenance file sha256 `1731b8636330841f20dcad9265710eacc3830767e8c693b34ad75b5907688dea`, schema `codefire.rust_build_provenance.v1`, `ok=true`, `base_revision c78d5e0e5e151a1489744c21340716a439597e71`, `binary_sha256` matches the binary, 101 source files, manifest `3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a`. At audited HEAD: `git diff --quiet c78d5e0e HEAD -- Cargo.toml Cargo.lock crates .cargo rust-toolchain rust-toolchain.toml` exits **0**; same for `schemas/public`. `cargo` and `rustc` are absent; no build ran. |
| label-only harness | **pass** | `bin/cf` = `5c7558a901835aec0056f5558fbce86cef5c897a5f8076208f70faf61f3458cf`, mode 555. Read in full: exposes exactly `C1..C6` plus non-executing `labels`/`log`, refuses everything else with exit 64, separates `PRESERVE=DRIFT` (71) from `PRESERVE=ERROR` (72). Contains **no** network primitive, **no** mutating CodeFire subcommand, **zero** 64-hex digests, **zero** first-run root/key paths. The only `git` is a read-only preservation check against `/workspace/project`. |
| answer key predates the answer | **pass** | Key `cea8bee0ceeac21822cf6327dcf2b783450de4025d82af53be191c0616eff7b7`, 9028 B, mtime `03:12:13Z`, atime `03:16:53Z` (Stage A seal). Consumer answer `fea6223d2a4394e1822f8636d4649288e1b47fcd8d3e3420e75a90edc9247113`, 5229 B, mtime `03:24:22Z`. **0** files under `$KEY2` newer than `03:12:19Z`. `$CTRL2/stagec/consumer-answer.frozen.json` is byte-identical **and** mtime-identical to the consumer answer. Setup metadata publishes the same key digest and withholds the path. |
| isolated run + distinct read-only confirmation | **pass** | Controller mirror is byte-identical to the handed transcript on all five envelopes. Node-5 confirmation (`03:27:43Z`–`03:27:48Z`, after scoring) is byte-identical to consumer **and** key for C2/C3/C4/C5. Smoke self-test is byte-identical to the key for C1–C6. Terminal facts reproduce from the confirmation pass alone. |
| one-rerun limit | **pass** | Exactly one first-run score (`7f7b86e`, `narrow`) and exactly one post-B1 score (`a805c22`, `adopt`). Two consumer roots exist and no third. Git history of the run packet shows one rerun-setup (`d6716de`) and one rerun-score. No second consumer was launched. |
| preservation of `/workspace/project` | **pass** | §5. No CodeFire command was issued against a canonical path by this audit. Harness checkpoints after every consumer command recorded `PRESERVE=OK`. |

## 2. Source-blind boundary and rerun identity

| Check | Expected | Observed | Verdict |
|---|---|---|---|
| handed docs pack | 14 files, digest `813edc85203a97b58e5efc56129db467723a16d8f61fed8e792369badde6fd7a` | relative-path rollup reproduces `813edc85…`; all 14 per-file sha256s match both the packet **and** HEAD public files byte-for-byte | **pass** |
| new/refreshed files | two new schemas + three B1-refreshed docs/schemas; nine unchanged vs first-run 12-file pack | `branch-list-v1.schema.json` `8403a663…` NEW; `show-result-v1.schema.json` `1a7e283f…` NEW; `04_cli_spec.md` `b7142880…`, `automation_interface.md` `9c87a4c9…`, `capabilities-v1.schema.json` `d559c164…` REFRESHED; remaining nine unchanged vs first-run | **pass** |
| product source / tests / internal docs as pack **paths** | 0 | 0 path matches for `crates`, `tests`, `docs/development`, `11_internal_architecture`, `08_object_store`, `09_index_trace_policy` | **pass** |
| first-run evidence reachable from `$ROOT2` | 0 | 0 hits for `m1-adoption-key`, `m1-adoption-root.d1YEhz`, `m1-adoption-ctrl`, `m1-adopt2-key` in any handed non-repo file | **pass** |
| answer-value leak in controller-authored handed files | 0 repository-specific answers | `TASK.md` / template / harness / setup-metadata: 0 hits for `2792b54a…`, `373468db…`, feature-branch names, `inspect_legacy_branch_profile`. Token `main` is present because it is the frozen C4 command / Q3 wording (unchanged since contract freeze). `"passed"` occurs in public `automation_interface.md` as the documented certificate vocabulary, not as this repository's answer | **pass** |
| key reachable from `$ROOT2` | never | `$KEY2` path appears nowhere in `$ROOT2`; setup-metadata publishes only `answer_key_sha256` | **pass** |
| private clarification | none | none requested or given; harness has no channel to one | **pass** |
| harness refusals during Stage B | 0 | all 10 lines in `$CTRL2/evidence/refusals.log` are `03:14:38Z` (9) and `03:15:17Z` (second C6) — strictly before the `03:16:53Z` seal. No `refusals.log` in the handed transcript | **pass** |
| C1–C6 identity vs first run | argv/cwd table unchanged except `$ROOT2` path | recorded argv are exactly `capabilities --json`; `doctor $FIX2 --quick --json`; `branch list --path $FIX2 --json`; `show main --json`; `show CF-COMMIT-373468dbaf827a6cc2bae9a4 --json`. C6 unused | **pass** |
| read outside `$ROOT2` | none evidenced | `seq=5`, no `HALT`, no `c6.used`. Every claim in the answer reproduces from the five recorded envelopes alone | **pass, with the packet's own honest limit upheld** |

**Boundary limit, restated rather than smoothed over:** `$CTRL2` and `$KEY2` share the consumer's
uid, so containment is *operational, not kernel-enforced*. Supporting evidence (key mtime/atime
predating the first command; zero new files under `$KEY2` after `03:12:19Z`; no `$KEY2` path
reachable; first-run digests intact; answer fully reproducible from the five envelopes) is
**consistent with no outside read, with no evidence of one**. A stronger claim is not available
from this setup, and none is made.

## 3. Score, citations, reliance, and decision — recomputed

### 3.1 C1–C5 exact argv / cwd / envelopes

Re-read from `$ROOT2/evidence/transcript/*.argv` and `*.meta`:

| seq | label | argv after runner | cwd | start | dur | exit | stderr | stdout sha256 | vs frozen C-table |
|---|---|---|---|---|---|---|---|---|---|
| 1 | C1 | `capabilities --json` | `$ROOT2` | 03:21:00Z | 4ms | 0 | 0 B | `f3f18424d8aa5bcd4a22a8ced1a51b967892f25901dc23bdc42ec2c5712d9d65` | **exact** |
| 2 | C3 | `branch list --path /tmp/m1-adopt2-root.N622xo/repo --json` | `$FIX2` | 03:21:00Z | 1815ms | 0 | 0 B | `34b48004d25819ffa2bfca006279160681db6654cb88c870a1a6219009b386bd` | **exact** |
| 3 | C2 | `doctor /tmp/m1-adopt2-root.N622xo/repo --quick --json` | `$FIX2` | 03:21:00Z | 1849ms | 0 | 0 B | `74fed7e56d8c30384c4005067df3859d4406d7bf7325641af8d914df178b4980` | **exact** |
| 4 | C4 | `show main --json` | `$FIX2` | 03:21:00Z | 1683ms | 0 | 0 B | `43a6e72eb2d1b3746a544df59e35224be75f9ce59dc075ce5d431e8691286cdc` | **exact** |
| 5 | C5 | `show CF-COMMIT-373468dbaf827a6cc2bae9a4 --json` | `$FIX2` | 03:21:58Z | 1808ms | 0 | 0 B | `9881736f238dabe6cdb4891076e3a61086b6bfa4803821f93c2f9005d6126eee` | **exact** (parent, as the key used) |

`argv[0]` is `codefire` on every line. Subcommands used: `capabilities`, `branch list`, `doctor`,
`show`. Exact-token forbidden scan over all five `*.argv` for
`init import open close discard clone scan fire extinguish verify commit merge upload
request-merge request-review request-apply gc serve evidence link patch migrate
--idempotency-key /workspace/project` and any first-run root: **zero hits**. C1/C3/C2/C4 share a
start second (concurrent); each recorded independently. C6 unused.

Envelope facts re-read directly from the recorded JSON (not from the packet):

- `C2.repo` = `/tmp/m1-adopt2-root.N622xo/repo`; `data.type=codefire_doctor_report` v1;
  `mode=quick`; `checked={objects:0,branches:8,opened:0,active_files:0}`;
  `skipped_checks=['object_store_integrity']`; `issue_counts={blocking:0,total:7}`; all 7 issues
  are `legacy_branch_head_semantic_profile` on `feature%2F*` paths — `main` is excluded.
- `C3.data.type=codefire_branch_list` v1, 8 branches; `main` head
  `CF-COMMIT-2792b54ad730e6d644668c38` `state=open-clean`; seven `feature/*` `closed`.
- `C4.data.type=codefire_show_result` v1; `commit=CF-COMMIT-2792b54ad730e6d644668c38`;
  `parents=["CF-COMMIT-373468dbaf827a6cc2bae9a4"]` (single); `certificate="passed"` (string);
  `signature={present:false, details:null, summary:"(none)"}`;
  `claim={result:passed, profile:local_verification_v1, phase:pre_commit,
  subject:{kind:live_open, trust_kind:local_unisolated, immutable:false}}`.
- `C5.show.commit=CF-COMMIT-373468dbaf827a6cc2bae9a4`; `signature.present=false`; claim present.
- `C1`: 34 commands; `branch.data_schema=codefire.branch_list.v1`;
  `show.data_schema=codefire.show_result.v1`; `output_schema=codefire.command_result.v1` for both.
  Capabilities schema `$defs.command` is `additionalProperties: false` and lists `data_schema`.
- Single `next_actions` entry, on C2 only: `kind=inspect_legacy_branch_profile`, keys
  `command, context, description, id, kind, reason, target`, **no `execution` object**.
  C1/C3/C4/C5 return `next_actions=[]`.

### 3.2 Artefact hashes (recomputed)

| Artefact | Packet | Recomputed | Verdict |
|---|---|---|---|
| consumer answer | `fea6223d2a4394e1822f8636d4649288e1b47fcd8d3e3420e75a90edc9247113` (5229 B) | identical | **match** |
| frozen copy | same digest | identical digest **and** mtime `03:24:22Z` | **match** |
| answer key | `cea8bee0ceeac21822cf6327dcf2b783450de4025d82af53be191c0616eff7b7` | identical (9028 B) | **match** |
| key sidecar `answer-key.sha256` | `cea8bee0…` | identical | **match** |
| derive script | `1a50db3492c0fb3ce617d6fdceb6797427201d354065126a9151915f21161c38` | identical | **match** |
| derive bundle (`derive/` relative) | `fb2a5cd86bea70821e158d69cd075e5b061ff7dbfb0557e5eb6aa724a9aa22a2` | identical | **match** |
| answer template | `e414421998f1a60fbf465ace98b565c08954599c19a7920496bcbf26bcae6e7c` | identical | **match** |
| task sheet | `776a9a45b9e1404ed9c895ddd9300e4eb22f4a0fc8238e684ab31e79e5122cd2` | identical | **match** |
| harness | `5c7558a901835aec0056f5558fbce86cef5c897a5f8076208f70faf61f3458cf` | identical | **match** |
| setup metadata | `62faafe21af6af1967ca52811c99d45057187208a3c319b9313f502656f9616c` | identical | **match** |
| docs pack | `813edc85203a97b58e5efc56129db467723a16d8f61fed8e792369badde6fd7a` (14 files) | identical | **match** |
| fixture members | `cccd8814a66dd3236c4a85e5752ab7bb6aa8ea09db2e74d35ab1a53998c07452` (2900 files / 114657290 B) | identical, and **byte-identical to the canonical members** | **match** |
| transcript bundle (21 files, relative) | `d74af2d40744300292a8f16f6b4cd809b4595d971ec7c1e8553ace26ba5305dc` | identical | **match** |
| confirm bundle (8 files, relative) | `41c445bf5b3ec70f7057ad4a01f19ca5efe6f97ea1798b4f47d4eedad73ad2f1` | identical | **match** |
| smoke bundle (relative) | `cfa30e5a6a466208c1da9f0d74039bebfb3a72110a8fd66fccc89e4c65ba2c4b` | identical | **match** |

### 3.3 C2–C5 byte-identity (key / consumer / controller)

Independently hashed. **All five consumer envelopes, including C1, are byte-identical to the
sealed key derivation.** Controller mirror (`$CTRL2/evidence/0N-Cx.json`) is byte-identical to
the consumer transcript. Node-5 confirmation (`$CTRL2/stagec/confirm/{C2,C3,C4,C5}.json`) is
byte-identical to both consumer and key. Smoke self-test is byte-identical to the key for
C1–C6. Confirm stderr files are 0 bytes.

This is stronger than the claimed C2–C5 three-way identity: C1 and the smoke C6 also match.

### 3.4 Q1–Q7 citations and score

Every citation was resolved structurally against the recorded envelopes. **25 citation
occurrences, 23 distinct label+path pairs, 0 unresolvable, 0 fabricated**, and every cited
value matches what the consumer asserted.

| Q | Key accept clause | Consumer | Re-scored |
|---|---|---|---|
| Q1 | root exact, branches=8, opened=0; objects 0 from C2 (quick skips `object_store_integrity`) or 2891 from C6 | root exact, 8, 0, objects 0 with the quick-mode reason stated | **correct** |
| Q2 | all 8, no extras | all 8 decoded names, no extras | **correct** |
| Q3 | `CF-COMMIT-2792b54ad730e6d644668c38` | exact, cited from `C4 .data.show.commit` and `C3 .data.branches` | **correct** |
| Q4 | `CF-COMMIT-373468dbaf827a6cc2bae9a4`, single parent | exact, single; C5 was the parent, so `C5 .data.show.commit` is the base | **correct** |
| Q5 | `certificate=passed` **and** embedded/local rather than signed/portable, justified by `signature.present=false`; post-B1 the documented `claim` object is an additional accepted route; calling it signed/portable/legacy is wrong | `passed`; `embedded/local`; justified by **both** `signature.present=false` / summary `(none)` **and** `claim.{result,profile,phase,subject}` | **correct** |
| Q6 | any two distinct pool entries, each cited | two distinct pool entries (absent signed/portable commit-signature authority; absent `sealed-snapshot-v1` proof on the 7 feature heads, `main` excluded), each cited | **correct** |
| Q7 | must name `inspect_legacy_branch_profile` **and** state it is not ready / has no `execution` object | named it; `ready_to_execute_as_is=false`; justified by the omitted `execution` object | **correct** |

**Score 7 / 7. Q5 and Q6 both correct. `misunderstanding_count 0`, `unanswerable_count 0`.**
Falsifier (`Q3 AND Q4 AND Q5` not all correct) does **not** fire.

Q5 extra sentence ("not legacy/unproven because `claim.profile` / `phase` / `subject` are
present rather than omitted") is recorded in §7. It does not change the score: the accept
clause is independently satisfied by `signature.present=false` and by the claim object.

The `claim_scope_explanation` is accurate and narrow: local `local_verification_v1` at
`pre_commit` for the worktree that was committed; not a signed portable proof; not
`sealed_validated` / immutable subject trust; other branches not lifted; quick doctor did not
re-prove object hashes and did not prove a live open directory (`checked.opened` is 0).
Neither parked hazard fired.

### 3.5 `undocumented_field_reliance` — mechanical, against the 14-file pack

Contract definition: distinct JSON field paths cited in correct answers that are absent from
the docs pack and from `schemas/public/**`.

```text
citation occurrences                                   25
distinct label+path pairs                              23   (packet §S.3)
distinct label-agnostic field paths                    21
  documented / schema-bound                            21
UNDOCUMENTED relied on                                  0
undocumented payload families                           0
consumer self-reported                                  0   (agrees)
```

Classifier (independent, not copied from Stage C):

- Envelope paths `.repo`, `.diagnostics`, `.next_actions`, `.next_actions[0].{kind,command}`
  are named in `command-result-v1.schema.json`.
- `.data.show.{commit,parents,certificate,claim,claim.profile,claim.phase,claim.subject,signature.present,signature.summary}`
  are named in `show-result-v1.schema.json` and in `docs/04_cli_spec.md` / `automation_interface.md`.
- `.data.branches` is named in `branch-list-v1.schema.json` and in `automation_interface.md`.
- Doctor `.data.checked.{objects,branches,opened}`, `.data.issues`, `.data.issue_counts.total`
  appear in the `doctor --json` sample in `automation_interface.md`.
- `.next_actions[0].target.semantic_validation_profile`: `target` is an envelope object;
  the leaf name has 4 hits in the pack (`04_cli_spec.md`, `05_consistency_model.md`,
  `automation_interface.md`, `capabilities-v1.schema.json`). First-run treated this path as
  documented; this audit does too.

First-run's 7 undocumented paths are now schema-bound:

| path | first run | after B1 |
|---|---|---|
| `.data.branches` | undocumented | `codefire.branch_list.v1` |
| `.data.show.commit` | undocumented | `codefire.show_result.v1` |
| `.data.show.parents` | undocumented | `codefire.show_result.v1` |
| `.data.show.certificate` | undocumented | `codefire.show_result.v1` |
| `.data.show.signature` / `.present` | undocumented | `codefire.show_result.v1` |
| `.data.show.target` | undocumented (not cited this run) | `codefire.show_result.v1` |

Control greps in the handed pack: `codefire_show_result` 3, `codefire_branch_list` 3,
`codefire.show_result.v1` 3, `codefire.branch_list.v1` 3, `data_schema` 3,
`codefire_doctor_report` 1. The first-run zeros for those type names are gone because B1
landed, which is the point of the rerun.

**Mechanical count = 0. Threshold `<= 1` passes. The `>= 2` narrow clause does not fire.**
Counted controller-side; the self-report was recorded but not trusted as the scored value.

### 3.6 Metrics

Observed values come from harness records and filesystem timestamps.

| Metric | Packet | Re-derived | Threshold | Verdict |
|---|---|---|---|---|
| `setup_time_s` | 249 | 249 (seal `03:16:53Z` → first exit-0 end `03:21:02Z`) | — | consistent |
| `ttfur_s` | 249 | 249 (first useful = C2, answers Q1) | ≤ 900 | **pass** |
| `ttac_s` | 449 | 449 (→ answer mtime `03:24:22Z`) | ≤ 3600 | **pass** |
| `commands_issued` | 5 | 5 | — | consistent |
| `off_contract_invocations` | 0 | 0 | = 0 | **pass** |
| `wrong_action_count` | 0 | 0 | = 0 | **pass** |
| `misunderstanding_count` | 0 | 0 | — | consistent |
| `unanswerable_count` | 0 | 0 | — | consistent |
| `undocumented_field_reliance` | **0** | **0** | ≤ 1 | **pass** |
| `docs_pack_files_read` | 14 | 14 listed, all 14 exist in the pack | — | consistent |

Self-reported `started_utc 03:18:00Z` is 67 s after the seal; `submitted_utc 03:24:30Z` is 8 s
after mtime. Every timing clause passes under either reading. Observed values are the ones
scored.

### 3.7 Original frozen table, applied in order — first match wins

Contract section 6, unchanged since `624f28c`. No clause was reinterpreted after seeing data.

| Row | Decision | Clause evaluation (this audit) | Match |
|---|---|---|---|
| 1 | stop (terminal) | `PRESERVE=OK` at every checkpoint and at audit time; `wrong_action_count=0`; no private clarification; no evidenced read outside `$ROOT2` | **no** |
| 2 | stop (terminal) | score 7 > 3; `ttac_s` 449 ≤ 7200 | **no** |
| 3 | narrow | Q5 **and** Q6 both correct → clause 1 false; **`undocumented_field_reliance` 0, so `>= 2` is false**; `ttac_s` 449 ∉ (3600, 7200] → clause 3 false | **no** |
| 4 | **adopt** | score 7 ≥ 6, Q5 correct, Q6 correct, `ttfur_s` 249 ≤ 900, `ttac_s` 449 ≤ 3600, `off_contract_invocations` 0, `wrong_action_count` 0, `undocumented_field_reliance` 0 ≤ 1 | **MATCH** |
| 5 | narrow | fallback, not reached | — |

**Literal decision: `adopt`.** The single clause that forced `narrow` on the first run
(`undocumented_field_reliance` 7 ≥ 2) is now 0. Subject, commands, questions, accept clauses,
and thresholds are the same; the difference is the B1 schema binding.

## 4. One-rerun limit

The frozen `narrow` definition authorized exactly one identical retry after B1. Independently:

- First-run terminal decision is still `narrow` at integrated commit `7f7b86e`, with frozen
  answer `187812d6…` and key `b5996cff…`.
- B1 landed at source `c78d5e0e` / integrated `0f5e9f9` + provenance `6a80adb`.
- One rerun setup (`d6716de`) and one rerun score (`a805c22`).
- Exactly two consumer roots exist (`d1YEhz`, `N622xo`). No third `m1-adopt*` consumer root.
- Rerun amendment deviations actually applied, and only those: fresh roots; 14-file pack
  (12 + 2 new schemas, 3 refreshes); new key `cea8bee0…` with the same Q1–Q7 accept clauses;
  answer-value leak gate replacing the type-name gate; fresh Grok consumer with no first-run
  evidence.

No second consumer, no threshold edit, no extra rerun. The one-rerun limit was honored, so
this `adopt` is terminal.

## 5. Preservation and root integrity — re-derived at audit time

Methods recorded beside values so a later auditor cannot report false drift (first-run F6).

Canonical `/workspace/project/subprojects/.codefire` (harness methods):

```text
git_head                f1644122134d479333cf422fdb1d48b8fa92dd8e            match
git_porcelain_count     248                                                 match
canon_digest            e5f6f7c2ac192934a0528414b54f1c2e097dfb65ec8c60a4f95b2c73251c41a6
                        (find -type f ! -name '*.lock.owner.json' | sort | xargs sha256sum | sha256sum)
canon_count             2904                                                match
codefire_open_mark      60b133b3eeefcc79e20bda32843261897713b0d6e86b2f0d0111cd19862690d6   match
no_codefire_server      0                                                   match
repo_json               bf6d978b0eece0f185c6995630339326f56513a0c8645af3e7e984ad69a7f054   match
branches_main_json      bfd52e3525f6a6faf0db4f3d2d1d9d7fa3fa0d400f6561c3c5b8227e79e989f8   match
branches_rollup         8fba5dcbedd5179a27f1a8d3ce699db2c4e62f94521084ff015e025fcc582841
                        (absolute-path sha256sum rollup of branches/)
opened_main_json        5721d710f4371db88be26567d2a3c6afb3bc2458288b8c812b54ccd7556a42b2   match
active_state_json       54ecb285704644b814e4ca47a711bf5eba150ee747642b90b53351b2f06940d2
                        (active/open_4c320ed42560/state.json)
objects_file_count      2891                                                match
objects_bytes           114921765                                           (`du -sb` objects/)
store_file_count        2904                                                (! -name '*.lock.owner.json')
store_bytes             114968174                                           (`du -sb` store)
store_all_files_digest  e5f6f7c2ac192934a0528414b54f1c2e097dfb65ec8c60a4f95b2c73251c41a6
                        (same exclude as harness CANON_DIGEST)
```

One `locks/repo.lock.owner.json` exists on the canonical store (76 B, mtime **2026-08-21
23:18:10Z**, before this milestone's first command). The harness excludes `*.lock.owner.json`
by construction; first-run used the same exclude. **0 canonical files are newer than
2026-08-22 02:00Z.** This is not post-experiment drift.

Remote roots (absolute-path rollup):

```text
.TaHMir     877461024a3deae974ff3b415a3cab8e953112c87d98540a91636b1d31139f40   2919 files   match
a2.zvrbzn   d2ee53b301e0fc8514be108e6e287f2f498a1d5fc7b48307a7b011f53ff823be   5360 files   match
a3.VbZTZ8   3fdd2867655f8e455a9269d8f9104f127d65e49034fdae292f5f3d6780b1a4ad   5381 files   match
```

First-run four frozen digests:

```text
answer      187812d6d25d5699d6dc6ca847a474e3b73c6e86892ec28e8f915b38cfa6c1a3   mtime 02:25:54Z  atime 02:25:57Z   match
key         b5996cff931da40afb272d535401accc1f312492a76788978cf45ae34b277392   mtime 02:11:40Z  atime 02:16:44Z   match
transcript  83a3f9156b97babfeb8ae78eea30a64a868da37d37657fe082e06ddb1e5a1148   21 files, relative rollup           match
confirm     6e4f46498e8bc95663276da8a887cdc1a25fad3c8f2e5264dcd108aa04aa2204   10 files, relative rollup           match
three-root file count 2938+71+29 = 3038                                         match
```

Rerun fixture: members `cccd8814…`, 2900 files, 114657290 member bytes matching canonical
members, six empty registries, no lock file (the `*lock*` find hit is the empty `locks/`
directory). Docs pack 14 files `813edc85…`. Answer key `cea8bee0…`.

**`PRESERVE=OK` / `NODE5_PRESERVATION=PASS` confirmed at audit time.** This audit wrote
nothing to any of those roots.

## 6. Absence checks

| Prohibited thing | Verdict | Independent basis |
|---|---|---|
| Git bridge / Git compatibility writer | **absent from the consumer path** | No `git` in any CodeFire argv; no `clone`, `import`, `merge`, `upload`, `request-*`. The only `git` is the harness's read-only `rev-parse HEAD` / `status --porcelain` against `/workspace/project` as a preservation check. It writes nothing and is not a product feature. |
| mutation authority | **absent** | Exact-token argv scan: **zero hits**. Positive proof: `$FIX2` is still 2900 files with **0** entries in `locks/` after five commands. Harness has no mutator code path. |
| telemetry collection | **absent** | All measurement is from harness-written local files (`.argv/.json/.stderr/.meta/index.tsv`) and filesystem timestamps. No instrumentation was added to CodeFire; nothing leaves local disk. |
| network use | **absent** | Harness contains no network primitive (`curl`/`wget`/`http`/`https`/`socket`/`urllib`: 0). No `.codefire-server` under `/workspace/project` (0). |
| private clarification | **absent** | No channel existed; none requested; 0 Stage B refusals; answer fully reproducible from five envelopes. |
| source / test / internal-doc reads | **absent** | §2 path scan. Content hits for the tokens `crates`/`tests`/`docs/development` in public README/runbook/automation_interface are those public docs referring to the project layout, not handed source trees. |
| agent-platform / agent-orchestration dependency | **absent** | Consumer was one plain Grok session with no subagents, no web, and a shell script as its only tool. Nothing in CodeFire was changed to support agents. Harness token `agent` occurs only in the dogfood path `algorithm-evolution-agent-lab`. |
| toolchain-free adoption claim | **not made; P2 stands** | Runner was handed; `cargo`/`rustc` absent; no prebuilt artefact is published. Neither run proves toolchain-free adoption. |
| product code / schema / test change in the rerun | **absent** | `git diff --name-only c78d5e0e HEAD` is three development-docs files (B1-fix packet, run packet, board). `crates/` and `schemas/public` are byte-identical to `c78d5e0e`. |

## 7. Parked comprehension hazards and Q5 imprecision — `block-m1`?

Reviewed independently against the M1 exit bullets and the review rule (`block-m1` / `park` /
`drop` only; `park` is not now-work).

### H1 — `branch list` `main state=open-clean` while `doctor` reports `opened=0` / `active_files=0`

**Still true** in the post-B1 envelopes. **Did not fire:** Q1 reports `opened_checked=0`;
`claim_scope_explanation` states the run "did not prove any live open directory
(`checked.opened` is 0)". Public pack now documents the trap (`04_cli_spec.md:116`,
`automation_interface.md`: persisted `state` is not live `opened/` authority).

**Not `block-m1`.** It is a remaining footgun on a documented, non-critical field. Both
consumers avoided it. Promoting it into required work would convert a `park` finding into an
M1 slice the contract already classified as persisted-state documentation, which B1 landed.
**`park`.**

### H2 — `claim.subject.kind=live_open` / `immutable=false` on a sealed commit in a closed repository

**Still true** (`C4.show.claim.subject`). This is the historical commit-time assessment
identity copied from the persisted certificate (B1 projection), not current workspace state.
**Did not fire:** the consumer described the subject as "the worktree that was committed" and
separately reported `opened=0`. `04_cli_spec.md:638` states claim copies existing certificate
fields and that legacy omission does not invent stronger authority.

**Not `block-m1`.** Created by the admitted B1 projection; documented as certificate copy, not
live-open authority. An unfired footgun is not a failed identification of claim/proof kind.
**`park`.**

### Q5 reasoning imprecision

The consumer wrote that the claim "is not legacy/unproven because `claim.profile`,
`claim.phase`, and `claim.subject` are present rather than omitted." The first-run key treated
"legacy-unversioned" as the doctor warning-set discriminator (Q6), not claim-field presence.

Independently: the **required** Q5 accept clause is met twice (`signature.present=false` and
the documented claim object). The extra sentence is a non-required strengthening argument.
It is also consistent with B1's own documented semantics (`04_cli_spec.md:638`: legacy
omission is represented by **not adding** profile/phase/subject). Q6 separately got the
warning-set exclusion right. The claim class returned is `embedded/local`, not
`legacy/unproven`.

**Not `block-m1`.** A reasoning wobble inside a correct, accept-clause-satisfying answer is
not a failed Q5. Thresholds may not be changed after seeing data to penalise it.
**`park` (or drop); recorded so node 7 does not reopen it as a fix.**

None of H1, H2, or the Q5 wobble falsifies EC3, fires a frozen stop/narrow clause, or
introduces a Git bridge / mutator / telemetry / agent surface.

## 8. Findings

### `block-m1` (0)

None. B1 was the only `block-m1` on this path; it is landed at `c78d5e0e`/`0f5e9f9` and the
rerun measured it closed (`undocumented_field_reliance` 7 → 0 with no other experimental
change). H1, H2, and the Q5 wobble are not blockers.

### `park` (7) — real, not to be worked now

1. **H1** — copied `state=open-clean` vs live `opened=0`. Documented; unfired; stays parked.
2. **H2** — `claim.subject.kind=live_open` is historical certificate identity. Documented as
   projection; unfired; stays parked.
3. **Q5 reasoning wobble** — extra non-required discriminator for "not legacy". Score
   unaffected. Do not reopen as a product fix.
4. **P2** — M1 does not prove toolchain-free adoption. Runner was handed; `cargo`/`rustc`
   absent. Belongs to M2. **No later slot may inherit the stronger claim.**
5. **P1, P3, P4, P5 unchanged** — Python CLI has no `--json`; `context --branch` doc/help
   divergence; no `data` schema for commands outside the B1 slice; `request-list`
   `applied_head`. Off-path; not reproduced.
6. **Operational, not kernel, containment** of `$CTRL2`/`$KEY2` (shared uid). Evidenced
   absence of an outside read; a stronger claim is unavailable.
7. **Live board graph is stale** relative to HEAD `a805c22` (still prints node 5e ready /
   nodes 6–7 blocked). Board write is node 7's job. Not a product defect and not permission
   to rerun.

### `drop` (2)

8. Reopening D1–D3, widening CFR-171, or promoting P1/P3/P4/P5 into this slot. Nothing in
   either run supports them.
9. A further consumer, a second rerun, a threshold change, or coercing `narrow`/`stop` out of
   H1/H2/Q5. The frozen table already yielded literal `adopt`.

## 9. Authorization for node 7

**Authorized:**

1. Treat the post-B1 rerun decision as terminal **`adopt`**.
2. Exit M1 `MS-adoption-wedge` on this ACCEPT, then apply the board shift rule: record exit
   evidence in `history.md`; rename slots (old M2 becomes M1, and so on); name a new M5 from
   `strategy/modernization-roadmap.md` without turning that roadmap into a task board; refuse
   any in-flight parked work that tried to start early.
3. Record the read-only sealed-inspect surface as a proven adoption wedge.
4. Leave the landed B1 slice (`codefire.show_result.v1`, `codefire.branch_list.v1`,
   per-command `data_schema`) as sufficient. No follow-up product write is authorized by this
   result.

**Refused:**

1. Any further consumer or rerun.
2. Any change to the section-6 thresholds, rubric, C1–C6 set, or fixture topology.
3. A new command, adapter, thin client, Git bridge, telemetry, agent surface, store-validator
   or sealed-format change, or the full CFR-171 cross-surface descriptor.
4. Starting H1/H2, P1–P5, CFR-168, CFB-136, CFB-133, CFR-169, or CFR-170.
5. Treating either run as evidence of toolchain-free adoption (P2).
6. Deleting the preserved first-run or rerun roots.
7. Any product, source, schema, packet, or issue edit as part of "closing" M1 beyond the
   board/history/current-state docs that node 7 owns.

Issue placement after this audit:

| Issue | Disposition |
|---|---|
| CFR-171 | satisfied at the admitted B1 slice; nothing further admitted |
| CFR-172 | terminal `adopt` recorded |
| CFR-173 | bounded slice satisfied; otherwise parked |
| CFR-168, CFB-136, CFB-133, CFR-169, CFR-170 | unchanged, backlog parked |
| contract `park` P1–P5, H1, H2 | unchanged and still parked |

## 10. Validation run by this audit

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

Results recorded with the commit. No build, no product test, no CodeFire command of any kind,
no import, no Git bridge, no live workspace command, no dogfood mutation, no consumer rerun,
no threshold change, and no later-slot gate. Write set is this file only.

## 11. Verdict

**ACCEPT.**

The post-B1 source-blind consumer scored **7/7** against sealed key
`cea8bee0ceeac21822cf6327dcf2b783450de4025d82af53be191c0616eff7b7`. All 25 citations resolve.
Mechanical `undocumented_field_reliance` is **0** against the refreshed 14-file pack. The
original ordered table yields literal **`adopt`** without threshold change or coercion. The
one-rerun limit was honored. Key, consumer, controller-mirror, and node-5 confirmation C2–C5
outputs are byte-identical (C1 and smoke C6 also match the key). Runner and provenance bind to
`c78d5e0e`. All 14 canonical values, 3 remote roots, 4 first-run digests, and the rerun
fixture/pack/key hashes are intact. No Git bridge, mutation authority, telemetry,
toolchain-free claim, or agent-orchestration dependency entered the consumer path. H1, H2,
and the Q5 wobble are **`park`**, not `block-m1`.

`block-m1`: **none**.

Node 7 may exit M1 and shift slots. It must not start parked work, widen CFR-171, rerun the
consumer, or inherit a toolchain-free adoption claim.
