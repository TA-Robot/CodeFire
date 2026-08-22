# AI Task Packet

Status: consumer-ready

Packet ID: MS-adoption-wedge-run

Schema version: human-v1

Risk score: 6

Reasoning tier: high

## Objective

Run one source-blind, inspect-only adoption experiment against an isolated closed copy of already
sealed CodeFire history. Measure whether a fresh consumer can identify repository, branches, head,
base, claim/proof kind, missing authority, and a next action using only the frozen public docs and
read-only JSON commands. Produce the precommitted adopt / narrow / stop decision without product code.

## Scope

- one new controller root and one separately handed consumer root under `/tmp`;
- copied sealed `repo.json`, `objects/`, and `branches/`, with all runtime registries empty;
- the exact public docs/schema pack and C1–C6 command/rubric contract frozen in
  `milestone-adoption-wedge-contract.md` Result sections 1–6;
- local experiment evidence and this packet only.

Canonical `/workspace/project`, prior remote evidence roots, product source/tests/docs, and every
non-target repository are read-only preservation subjects.

## Fan-in adjudication

- Node 1's undocumented `show` / `branch list` payload finding is a **conditional CFR-171 candidate**,
  not a pre-run product write. The M1 exit explicitly accepts a retained terminal failure.
- Node 2 independently found the existing profile/subject/result/digest fields sufficient to attempt
  inspection and recommended no product write.
- Therefore run the frozen experiment first. Apply the section-6 table exactly: `adopt` or `narrow`
  may admit only the documented/schema-bound B1 slice and at most one identical rerun; `stop` records
  falsification and admits no fix.
- Review-pack/import/live verify/scan/open/Git are not added to the first experiment. CFR-168/CFB-136
  and later milestones remain parked.

## Exact inputs

- integration base at admission: record exact HEAD before setup;
- runner: `/workspace/worktrees/ms-remote-happy-path-budget-20260821/target/release/codefire`, SHA-256
  `70301c874d7220a71be988562df0e5188ede6a85c65c63dcbd3dcdee09f4706c`;
- canonical copy source: `/workspace/project/subprojects/.codefire`, read-only;
- consumer docs pack: exactly the files listed in contract Result section 1;
- commands, required/optional status, cwd, field questions, preservation manifest, metrics, seven-item
  rubric, falsifier, privacy rules, and ordered decision table: contract Result sections 2–6,
  unchanged;
- fresh consumer: Grok 4.6 session with no previous CodeFire context, no subagents, no web, no
  maintainer clarification, and no access authorization outside the handed consumer root.

## Procedure

### Stage A — persistent fixture owner

1. Rehash runner/provenance and every canonical preservation value. Stop on mismatch.
2. Create separate `$CTRL` and `$ROOT` with `mktemp -d`; do not reuse prior roots.
3. Materialize `$ROOT/repo/.codefire` by copying only `repo.json`, `objects/`, and `branches/`; create
   empty `opened/`, `active/`, `cache/`, `idempotency/`, `locks/`, `remotes/`.
4. Copy the frozen public docs pack into `$ROOT/docs-pack`. Create a task sheet containing only C1–C6,
   the seven questions, required citation shape, and the public thresholds; do not include an answer.
5. Create a controller-owned harness that exposes only C1–C6, records exact underlying argv/cwd/start/
   end/duration/exit/stdout/stderr, rechecks canonical and fixture preservation after every command,
   and refuses every other command label. Store the answer key outside `$ROOT`; put only its SHA-256
   in handed metadata.
6. Derive the answer key by running the same C1–C6 JSON commands controller-side before handoff.
   Optional C6 may run at most once. Stop on any nonzero required command, preservation drift, or
   answer-key ambiguity.
7. Record setup handoff, set this packet `consumer-ready`, validate, and commit only this packet.

### Stage B — fresh source-blind consumer

1. Work only inside `$ROOT`; read the task sheet and docs pack, not this packet or any source tree.
2. Use only the provided harness labels C1–C6; do not invoke CodeFire directly.
3. Submit `$ROOT/evidence/consumer-answer.json` with Q1–Q7, exact command/JSON-field citations,
   `first_useful_command`, and a short claim-scope explanation. Do not read the controller answer key.
4. Return without product/repository edits or private clarification.

### Stage C — persistent fixture owner

1. Verify the consumer transcript/read set, forbidden-token scan, answer citations, preservation, and
   metrics. Score once against the pre-hashed key.
2. Apply the contract section-6 ordered decision table without changing thresholds after seeing data.
3. Run a distinct read-only confirmation of fixture/canonical hashes and the required C2–C5 terminal
   envelopes; no consumer answer changes.
4. Record result/decision, set this packet `completed` or `blocked`, validate, and commit only it.

## Validation

Only the experiment, preservation checks, and:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

No build, product test, import, Git bridge, live workspace command, dogfood mutation, or later-slot gate.

## Non-Goals

- implementing B1/CFR-171 before the first score;
- using review-pack, import, open, scan, verify, context, explain, commit, remote commands, or Git as
  part of the consumer path;
- reading source/tests/internal/development docs or asking the maintainer what fields mean;
- telemetry, network, external publication, broad pilot, release, product-value, or ecosystem work;
- deleting evidence before independent audit.

## Handoff

Return exact roots, runner/input hashes, harness identity, answer-key hash, consumer session identity,
all command evidence and durations, Q1–Q7 score/citations, field reliance, preservation result,
adopt/narrow/stop decision, issue admission/parking, stall/scope-creep risk, and next legal action.

## Result

### Stage A — completed. Packet status `consumer-ready`. Stage B has not run.

Every Stage A gate passed. The fixture, docs pack, label-only harness, task sheet, and sealed answer
key exist; the answer key was derived and hashed **before** handoff and is not reachable from the
handed root. No product source, test, or schema was edited. No repository was mutated. No consumer has
been launched, and nothing has been scored.

#### A.1 Roots (all new; no prior root reused)

```text
$CTRL  /tmp/m1-adoption-ctrl.c23eQC     controller: baselines, per-command mirror, harness self-test evidence   (mode 700)
$ROOT  /tmp/m1-adoption-root.d1YEhz     HANDED consumer root                                                    (mode 755)
$KEY   /tmp/m1-adoption-key.JbqhPy      answer key + derivation bundle; path disclosed nowhere in $ROOT          (mode 700)
$FIX   /tmp/m1-adoption-root.d1YEhz/repo   the only admitted target
```

`$KEY` is a third root created specifically so that the harness may reference `$CTRL` for its
authoritative log mirror without that reference leading to the key. Verified: `grep -rl 'm1-adoption-key'
$ROOT` → 0 files, and `grep -rl 'm1-adoption-key' $CTRL` → 0 files.

#### A.2 Runner and provenance, rehashed at integration HEAD `6f991fe`

```text
integration HEAD                6f991feb3dd3bcb6db73619ead744a3bb9342de1  (admission 3c0be0d merged)
runner                          /workspace/worktrees/ms-remote-happy-path-budget-20260821/target/release/codefire
runner sha256                   70301c874d7220a71be988562df0e5188ede6a85c65c63dcbd3dcdee09f4706c   ok
runner bytes / inode            14594080 / 1361045                                                  ok
provenance file                 .../target/codefire-ai/rust-build-provenance.json
provenance sha256               d87653d0a6e4a00216ab47816167744ce2e35c3df142a7ccd9810472d35e61d8   ok
provenance schema / ok          codefire.rust_build_provenance.v1 / true
provenance base_revision        510a1855cb8753219aae416c6cbfea58f059415d
provenance binary sha256        70301c874d7220a71be988562df0e5188ede6a85c65c63dcbd3dcdee09f4706c   matches binary
provenance source manifest      c59e8c229bc31e71b8b5d5d5b1adfaed7defd2557d0286d06efdb74dc664a274 (64 hex), 100 files
provenance build command        cargo build --release --locked -p codefire-cli --bin codefire-rs
provenance toolchain            cargo 1.97.0 (c980f4866 2026-06-30) / rustc 1.97.0 (2d8144b78 2026-07-07)
runner<->HEAD binding           git diff --quiet 510a185 HEAD -- Cargo.toml Cargo.lock crates .cargo rust-toolchain rust-toolchain.toml  -> exit 0
```

No build ran. `cargo` and `rustc` remain absent in this environment, which is why the audited binary is
a provided input (contract `park` P2).

#### A.3 Canonical and prior-evidence baseline, rehashed before setup and re-verified after every phase

```text
git_head                f1644122134d479333cf422fdb1d48b8fa92dd8e
git_porcelain_count     248
repo_json               bf6d978b0eece0f185c6995630339326f56513a0c8645af3e7e984ad69a7f054
branches_main_json      bfd52e3525f6a6faf0db4f3d2d1d9d7fa3fa0d400f6561c3c5b8227e79e989f8
branches_rollup         8fba5dcbedd5179a27f1a8d3ce699db2c4e62f94521084ff015e025fcc582841
opened_main_json        5721d710f4371db88be26567d2a3c6afb3bc2458288b8c812b54ccd7556a42b2
active_state_json       54ecb285704644b814e4ca47a711bf5eba150ee747642b90b53351b2f06940d2
codefire_open_mark      60b133b3eeefcc79e20bda32843261897713b0d6e86b2f0d0111cd19862690d6
objects_file_count      2891
objects_bytes           114921765
no_codefire_server      0
store_all_files_digest  e5f6f7c2ac192934a0528414b54f1c2e097dfb65ec8c60a4f95b2c73251c41a6
store_file_count        2904
store_bytes             114968174
prior root .TaHMir      877461024a3deae974ff3b415a3cab8e953112c87d98540a91636b1d31139f40   (2919 files)
prior root a2.zvrbzn    d2ee53b301e0fc8514be108e6e287f2f498a1d5fc7b48307a7b011f53ff823be   (5360 files)
prior root a3.VbZTZ8    3fdd2867655f8e455a9269d8f9104f127d65e49034fdae292f5f3d6780b1a4ad   (5381 files)
```

Checkpoints, all `PASS`: before setup, after the fixture copy, after answer-key derivation, after each
of the six harness self-test commands, and at final handoff. **No CodeFire command was issued against
any canonical path at any point** — stricter than the remote milestone's script, which invoked a
canonical `status`.

#### A.4 Fixture materialization and copy-fidelity gate

`$FIX/.codefire` holds copies of `repo.json`, `objects/`, `branches/` plus six empty registries
(`opened active cache idempotency locks remotes`, 0 entries each).

```text
copied member files          2900   (1 repo.json + 2891 objects + 8 branches)
member content manifest      cccd8814a66dd3236c4a85e5752ab7bb6aa8ea09db2e74d35ab1a53998c07452
   canonical side            cccd8814a66dd3236c4a85e5752ab7bb6aa8ea09db2e74d35ab1a53998c07452   EXACT MATCH
member file bytes            114657290 both sides
fixture repo digest (pre)    15c2874218e07017cda0737e562b5480d01b6885cd427e1b7c6f9421d59e68b2
```

The manifest is taken over repo-relative paths so it is path-independent; a naive `find | xargs
sha256sum` rollup differs across roots because the digest covers the printed path, and `du -sb` differs
by directory apparent size only. File bytes and per-file digests are identical.

**All six read-only commands produced zero fixture side effects**: file count stayed 2900 and not even
a `locks/repo.lock` appeared. That is direct evidence the C1–C6 path holds no write authority.

#### A.5 Docs pack (exactly the frozen set, 12 files)

```text
422174c3a419d846abce1844461e449c4cbb8178a0983aac2afc3911f355110e  README.md
c4511be600d8e7405ae7afd840b8c5bf5d2053499aa474961dc883cb92c8e55d  docs/02_core_concepts.md
909ac4c44ff3f1daa0d07235d27ec2d4b507405f84e1e0d2dcfd0a075c01d7d9  docs/04_cli_spec.md
6a70b6b7e83f07b261587e7231f66c6836516a67e5ccc0baaba9e92f2e14927b  docs/05_consistency_model.md
5a10f0121810eb9730d592c646eacb57c1c5715d50e666783e179ff37f3829c8  docs/automation_interface.md
bc8943c63109d238e7c27a7d418b45b29f1e8477618e606a3684c914df7e035b  docs/runbook.md
d67a1a21c49c9ec952444312801523b17e5a59a65fc88f0bad400738bb125793  schemas/conformance/manifest-v1.json
8e86aa215a11b58c0e43e5c50b0c51346be06bf7fd12976ca533ddebc93c86b5  schemas/public/capabilities-v1.schema.json
f7f00b62350367a561147e5a11aea1d4f74c4f0977d3c8396bc535199f4dc372  schemas/public/command-result-v1.schema.json
461da6adc376e8f29932d338adb4f1293da8fabe3ed22c76c892dc8fb9e72b56  schemas/public/page-cursor-v1.schema.json
fb93c97deb7382d7a4b55239644b9f2dc3aca337c7058e8e2465cdd2c0fb442e  schemas/public/page-v1.schema.json
981231921326e1eb40bd5c694792c7d8853388dbd99ecea15f9ecd4eae2559e3  schemas/public/remote-doctor-v1.schema.json

docs_pack_digest 218123042bb8b1a680deb7b06581afb13a225e7d7b85397fde693f2fa5896034
```

Absent from the pack, verified by path scan: `crates`, `tests`, `docs/development`,
`11_internal_architecture`, `08_object_store`, `09_index_trace_policy` — 0 matches each. Answer-value
leak scan across the pack (`2792b54a…`, `373468db…`, `codefire_show_result`, `codefire_branch_list`,
`inspect_legacy_branch_profile`) — 0 matches each, while `codefire_doctor_report`,
`codefire.capabilities.v1`, and `next_actions` are present as expected.

#### A.6 Label-only harness

```text
path             $ROOT/bin/cf                (mode 555)
sha256           83dc9d2581df5e6f4874a9670d6f934aa84beadce0e90ecb1c633d39bcc9dd32
allowed labels   C1 C2 C3 C4 C5 <CF-COMMIT-...> C6, plus the non-executing helpers `labels` and `log`
records per run  label, seq, runner, cwd, exact argv, start/end ISO, duration_ms, exit,
                 stdout/stderr bytes, stdout sha256, preservation verdict
writes to        $ROOT/evidence/transcript/NN-Cx.{json,stderr,meta,argv} + index.tsv,
                 mirrored to $CTRL/evidence/
enforcement      unknown label / extra args / bad C5 id / second C6 -> exit 64, logged to refusals.log;
                 PRESERVE=DRIFT -> HALT + exit 71; internal error -> HALT + exit 72; HALT is permanent
```

Underlying argv, exactly the frozen contract set:

```text
C1  codefire capabilities --json                                              cwd=$ROOT
C2  codefire doctor /tmp/m1-adoption-root.d1YEhz/repo --quick --json          cwd=$FIX
C3  codefire branch list --path /tmp/m1-adoption-root.d1YEhz/repo --json      cwd=$FIX
C4  codefire show main --json                                                 cwd=$FIX
C5  codefire show <CF-COMMIT-...> --json                                      cwd=$FIX
C6  codefire doctor /tmp/m1-adoption-root.d1YEhz/repo --json                  cwd=$FIX   (at most once)
```

Refusal self-test, all refused with exit 64: `cf status`, `cf C7`, `cf` (no label), `cf capabilities`,
`cf C1 --json`, `cf C5` (missing id), `cf C5 deadbeef`, `cf verify`, and a second `cf C6`.

**A harness defect was found and fixed before handoff, not after.** The first happy-path self-test
exited 71 and self-halted while the underlying command had exited 0. Cause: in
`local tag="$1" f=0 out="$CTRL_LOG/preserve-$tag.txt"`, bash expands the whole `local` word list before
assigning, so under `set -u` `$tag` was unbound, the preservation file was never created, and an empty
verdict was misread as drift. Fixed by splitting the declarations and by separating a genuine
`PRESERVE=DRIFT` (exit 71) from a `PRESERVE=ERROR` internal fault (exit 72), so a harness bug can never
again be reported as a preservation failure. Both self-test generations are retained in
`$CTRL/harness/`.

Post-fix self-test ran all six labels: every exit 0, every stderr empty, every recorded verdict
`PRESERVE=OK`, durations 5 / 1339 / 2256 / 1927 / 1875 / 3020 ms. Handed transcript, `seq`, `c6.used`,
and `HALT` were then reset; the smoke bundle is retained controller-side
(`smoke_bundle_digest 47966024081784a9ec91345fc038f9e97d807a0609e72060401cf449a2691b08`).

#### A.7 Answer key — derived, sealed, and unreachable from the handed root

Derivation ran the identical argv/cwd controller-side, writing only into `$KEY/derive`, so the handed
transcript started empty.

```text
derive script            $KEY/derive-key.sh   sha256 e1bd0822baac77197825335f52c055a4707764cfdaac5a29dd2ec06cf77e181b
derive bundle digest     1f40a99ca2221cbf1ac9f3249096a58667f35a015004236062ce251ec9359ac2
answer key               $KEY/answer-key.json sha256 b5996cff931da40afb272d535401accc1f312492a76788978cf45ae34b277392
```

| label | exit | duration | stdout sha256 | harness self-test |
|---|---|---|---|---|
| C1 | 0 | 3ms | `5d5245c3f33d706eb32b9e510f5173541752df912357717093686ada76a2d3b0` | identical |
| C2 | 0 | 1384ms | `c9bfa51cdba9394a5974fa43308974a10a7bcaf4bd840766ada2002df2930968` | identical |
| C3 | 0 | 1052ms | `9ba62a916e28860d7f0669f57ff959dcc135cbc4640e30e91c94dd3d0d42999d` | identical |
| C4 | 0 | 1003ms | `1c22cec24e4a3992aa641c56da8b19a78809c21d54d86be1ba2dcbb27dfd309c` | identical |
| C5 | 0 | 983ms | `2b0a2d718bf8c35fd60668862ab398553a21f5210f351d11e714a388f47f2aa5` | identical |
| C6 | 0 | 2033ms | `610185f89914cdc3ba75038f84e9037799b83f823c6a0edf0a39d73f1f88d1df` | identical |

All stderr empty. `C5` argument `CF-COMMIT-373468dbaf827a6cc2bae9a4`, taken from `C4
.data.show.parents[0]`, which is a single-element list — no key ambiguity. The key records, per
question, the correct value, the accepted answer forms, and the citable field paths, so Stage C scoring
is mechanical and cannot drift after seeing the consumer's answer.

Three Stage A observations are recorded in the key for Stage C, and none of them is a Stage A gate
failure:

1. `C3` reports branch `main` `state=open-clean` while `C2`/`C6` report `opened=0` and
   `active_files=0`. The branch record carries the state copied from the source repository; the fixture
   has an empty `opened/` registry, and `doctor` does not flag the combination. A consumer that reports
   a live open workspace has misread the repository.
2. `show --json` returns `data.type=codefire_show_result` v1 with `certificate` as the **bare string**
   `"passed"` and **no** assessment `subject`, `profile`, or `phase`. Contract finding B1 is therefore
   confirmed empirically, not just from documentation absence: Q5's claim-class answer must be composed
   from `signature.present=false` plus the doctor semantic-profile warnings.
3. `C1` reports `output_schema=codefire.command_result.v1` for both `show` and `branch`, so
   `capabilities` cannot distinguish their `data` payloads.

Also confirmed as expected from prior read-only evidence: `C2`/`C6` return exactly 7 non-blocking
`legacy_branch_head_semantic_profile` warnings covering the 7 `feature/*` heads, while `main`'s head
carries current `sealed-snapshot-v1` proof — the mixed state Q5/Q6 depend on.

#### A.8 Handed root — final inventory and integrity

```text
$ROOT/TASK.md                                  sha256 89eb24aac6c7291078620d1c0e95b0335ad7dc403550b2844cf2a8dc68a0958f  (mode 444)
$ROOT/bin/cf                                   sha256 83dc9d2581df5e6f4874a9670d6f934aa84beadce0e90ecb1c633d39bcc9dd32  (mode 555)
$ROOT/evidence/setup-metadata.json             sha256 bcf40c0442a27d6d777434724597e55a1dba93a954310b9405ae26a548200c25  (mode 444)
$ROOT/evidence/consumer-answer.template.json   sha256 61916070ec5c04a9f1c0efdcdf6bbd7da91eff6e846e77fc1e3434b59b523ad0  (mode 444)
$ROOT/docs-pack/                               12 files, digest 218123042bb8b1a680deb7b06581afb13a225e7d7b85397fde693f2fa5896034
$ROOT/repo/                                    2900 files, members digest cccd8814a66dd3236c4a85e5752ab7bb6aa8ea09db2e74d35ab1a53998c07452
$ROOT/evidence/transcript/                     absent until the consumer's first command
```

`setup-metadata.json` publishes `answer_key_sha256 b5996cff…` and states the key is outside the handed
directory with its path deliberately withheld, so the auditor can prove the key predates the answer
without the consumer being able to reach it. The task sheet contains C1–C6, the seven questions, the
required citation shape, and the public thresholds, and **no answer**: leak scan for `2792b54a…`,
`373468db…`, `codefire_show_result`, `inspect_legacy_branch_profile`, `legacy-unversioned` returns 0
matches across `TASK.md`, `evidence/`, and `bin/`.

Honest limit, for the node-6 audit rather than buried: `$CTRL` and `$KEY` are the same uid as the
consumer, so containment is operational, not kernel-enforced. `$CTRL` is mode 700 and referenced by the
harness; `$KEY` is mode 700 and referenced nowhere reachable. Stage C must therefore scan the consumer
transcript and read set for any access outside `$ROOT`, per the contract's `wrong_action_count`
falsifier, instead of trusting the instruction.

#### A.9 Consumer launch instructions (Stage B — not started)

Launch a fresh Grok 4.6 session with no previous CodeFire context, no subagents, no web, and no
maintainer channel, authorized to read and write only under
`/tmp/m1-adoption-root.d1YEhz`. Hand it exactly this:

```text
Your entire working directory is /tmp/m1-adoption-root.d1YEhz.
Read TASK.md first and follow it exactly.
Run commands only through ./bin/cf using the labels C1-C6. Do not invoke any other binary.
Do not read, list, or search anything outside this directory. Do not use the network.
Write exactly one file: evidence/consumer-answer.json, following
evidence/consumer-answer.template.json.
Record your start time before your first command and your submit time when you write the answer.
When the answer file is written, stop.
```

Nothing else is disclosed: not this packet, not the contract packet, not `$CTRL`, not `$KEY`, not the
answer key digest's location, not the repository's provenance.

#### A.10 Validation and preservation outcome

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

All three pass; results recorded with the commit. No build, no product test, no import, no Git bridge,
no live workspace command, no dogfood mutation, no later-slot gate. Write set is this packet only.

Preservation outcome: **`PRESERVE=OK` at every checkpoint.** All 14 canonical values, all three prior
evidence roots, and the fixture member digest are unchanged at final handoff. No canonical path was
ever passed to a CodeFire command.

#### A.11 Next legal action

Stage B, node 4b: launch the fresh source-blind Grok 4.6 consumer with the §A.9 text and nothing more.
Do not score, do not open the answer key, and do not touch the contract's thresholds. Stage C (node 4c)
scores once against key `b5996cff931da40afb272d535401accc1f312492a76788978cf45ae34b277392` and applies
the contract section-6 ordered decision table unchanged. B1/CFR-171 stays unimplemented until that
score exists.
