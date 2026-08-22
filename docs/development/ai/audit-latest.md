# Milestone audit

Final independent **other-family** audit of node 6 for active M1 `MS-product-value-proof`
at exact integration HEAD `ca955fc`. This is the product-value first-seal final audit. It
authorizes or blocks M1 exit. It does **not** convert `narrow` into `adopt`.

- time (ISO): `2026-08-22T05:36:54Z`
- audited HEAD (exact): `ca955fc83e9cbe26d141c678d5cf0941302cf441` (`ca955fc`, "docs: admit product value final audit")
- preserved study root audited (read-only): `/tmp/codefire-m1-pvp.rh8k89q9` mode `0700`
- packets read: `milestone-product-value-proof-segment.md`, `…-first-seal.md`, `…-run.md`
- active milestone: M1 `MS-product-value-proof` — First product-value proof
- scope of this audit: **node 6 only**. Independently recompute node-6 evidence from frozen
  bytes. Do not trust packet conclusions. Audit only the one participant run and the
  controller confirmation. Do not run CodeFire. Do not run a participant.
- stalled: **no**
- scope_creep: **no**
- next_milestone_risk: low. Nothing in this evidence admits a product fix, a retry, a
  README/docs/schema write, a threshold edit, a human/market claim, telemetry, Git bridge,
  hosted service, agent-platform work, or any M2–M5 start.
- auditor: Grok 4.6, fresh session, separate worktree
  `worktrees/m1-product-final-audit-20260822` on branch
  `grok-agent/m1-product-final-audit-20260822` at `ca955fc`. Node-5b participant was a
  fresh Claude session. Node-5a/Stage C controller was the persistent Grok node-2 session.
  This session is not the participant, not the controller, not node 1, and not Codex.
  Other-family versus the Claude participant is satisfied.
- write set honored: **this file only.** No product, source, schema, packet, board, issue,
  fixture, answer, transcript, or evidence file was edited. No worktree was switched. No
  rebuild. This audit issued **no CodeFire command of any kind** and did not start a
  participant.

This audit did not accept packet prose as proof. Every hash, envelope field, mtime order,
docs-pack identifier search, metric, Q score, preservation digest, and first-match table
row below was re-derived independently from the retained root and from `git` at the
audited HEAD.

**Terminal verdict: ACCEPT. Literal frozen-table decision: `narrow`. `block-m1`: none.**

Literal terminal consequence, already selected by the frozen table and **not executed
here**: **(i) narrow the public claim** to what this one clean-room AI run supports.
**Do not** retry. **Do not** product-fix README, docs, JSON keys, yaml, or thresholds.

## 1. Requirement matrix — exact M1 exit bullets

| # | Exact M1 exit bullet | Verdict | How it was verified independently |
|---|---|---|---|
| R1 | one candidate segment, exact first-seal task, claim-comprehension questions, value hypothesis, falsifier, measurements, and adopt / narrow / stop thresholds frozen before study setup | **pass** | Segment packet Result §§1–9 frozen before fixture existence. Run packet records node-1 Result §§2–9 byte-for-byte with no threshold edit. `$KEY` files mode 444; TASK/SETUP contain no thresholds. |
| R2 | one source-blind participant completes or terminally fails an isolated first-seal path using only the admitted runner, public docs/help/JSON actions, and the handed fixture; no product source or private correction | **pass** | 19 wrapper execs, all `note=exec`, 0 `refused`. Argv targets `$FIX` or help/capabilities only. Transcript has no `crates/`, `docs/development`, `$KEY`, Git, network, init/import/upload. KEY atimes all `05:08:38Z`, before T0. |
| R3 | the participant identifies what the seal proves, what it does not, the base/head and local versus portable authority, and chooses no mutation beyond the handed task; canonical/user repositories are preserved | **pass** | Independent Q1–Q4 all 1. Four declared files changed; README and both temptations byte-identical to Stage A. P2–P7 match freeze **now**. |
| R4 | setup time, time to first useful result and first seal, command/help/docs load, misunderstandings, wrong actions, support interventions, task value, and maintenance/friction cost are recorded honestly | **pass** | All 16 node-1 §6 metrics independently recomputed from transcript `t` / ANSWERS mtime / envelopes. Participant self-report discarded where it used first-edit or exit-10. Maintenance friction labelled estimate only. |
| R5 | an independent other-family audit verifies the frozen thresholds, evidence, preservation, source-blind boundary, terminal decision, and absence of hidden telemetry, Git bridge, hosted service, or agent-platform work | **pass (this audit)** | Table first-match is `narrow` via M12. Absence checks in §12. README adjudicated as coverage limitation, not a silent fix. |

## 2. Identities — source, runner, key, task, docs, wrapper

Integration HEAD `ca955fc83e9cbe26d141c678d5cf0941302cf441`. `git diff --stat 01a4de0 HEAD -- Cargo.toml Cargo.lock crates .cargo rust-toolchain rust-toolchain.toml` is **empty**. Independent `build_source_manifest` at this HEAD (no compile):

```text
schema                 codefire.rust_build_inputs.v1
file_count             101
manifest_sha256        3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a
```

Study runner is **not** rebound to `01a4de0`. Provenance file SHA-256
`9dbb7f4c79eaad337ae7f467fff273993919f13b2e9b4cea678502e9c3debcef` (key copy equals
`$RUNNER/rust-build-provenance.json`) binds:

| Field | Independent value |
|---|---|
| `schema` | `codefire.rust_build_provenance.v1` |
| `ok` | true |
| `base_revision` | `34159140bc7865a6b9ecd9440b94979c7efd8baf` |
| `source_manifest_sha256` | `3925fc03…` / 101 files |
| `binary_sha256` | `90cb2895aadc45b559a96207cdd5ae3b08485f9a7e5a07014db8cca8b5b90ce9` |
| `binary_bytes` | 14590776 |
| `build_command` | `cargo build --release --locked -p codefire-cli --bin codefire-rs` |

`$RUNNER/codefire` SHA-256 is the same `90cb2895…` image (14590776 B, mode 755). Identical
historical B1/release **bytes are not identity**; identity is provenance `base_revision=3415914`
plus manifest `3925fc03…`. Python `codefire` was not the runner. No `install.sh`.

`$KEY` PRESERVE_DIGEST `5b995f6f397e0df0e06485b07cba20538336c7c184a4e351e9a2c2af80d7315d`
(8 files / 4506 B, all mode 444). SHA256SUMS 7/7 independently verified:

```text
106f57680cc02a1f1dabf4bbbe09565a66e02042e6e6414ff21f4c7c19fbf299  answer-key.json
a2eb294c9e3b4eb6bb8df87c75e176f91431501bd3b21a503bcc4bff4dc23c06  answers-template.sha256
9a784dc18bb08daad08bd2ae7e31489cc219ea34d32ec3e826dfe99aa54b8c27  runner-identity.json
9dbb7f4c79eaad337ae7f467fff273993919f13b2e9b4cea678502e9c3debcef  rust-build-provenance.json
3cec5a19f001a8c514223a88adc0ffd0b95cbf853675259a0e158d258e87eb51  task.md.sha256
22b7128263da4cf688d53f0b027cea4e59360196917c28be5919850dff27e250  wrapper.json.sha256
b5f751db1cc0862c3cf1c750a2a8f974f01388ba494e5959477794c374283c4a  wrapper.py.sha256
```

Handed materials, independently rehashed:

| Path | SHA-256 | Bytes |
|---|---|---:|
| `$PART/TASK.md` | `c46e00a7438ad2f88438f3065b10529087844ad10b17be84267576c51127fba2` | 1399 |
| `$PART/SETUP.md` | `b6648596e17fa953697bb87dbecc01af8f3f5c82d9692fd3220f05ca293a9aa0` | 307 |
| `$PART/codefire` (wrapper) | `a2d11636de687a4507319447b5146dcb13881e9b0844a045be3e5667cf2c120d` | 8718 |
| `$PART/wrapper.json` | `3131b13acc264cc3ef0514cea6bb8f880e36261ba736efbecb61f7d5ad26bc25` | 1117 |
| `$PART/ANSWERS.md` (filled, frozen) | `14b29057cb643ccbe6488a32397651242d33d6f7f974470587bddbe54a011243` | 22929 |
| answers **template** (key, pre-fill) | `0cc8f47d1de6a026a575f8df92bd0da80b1fc6106e8999c5bd8fb0cf3d06e51a` | — |

`task.md.sha256` and `wrapper.py.sha256` / `wrapper.json.sha256` match the live handed files.
TASK names the lowercase ACME vector and the two temptations; it contains no argv, Fire ID,
resolution text, threshold, or answer. SETUP is the five-field handoff JSON (`t0`, wrapper,
fixture, docs). `$DOCS` PRESERVE_DIGEST
`31f450a3a315730f243519518fa4c9e2de313067232b6aa3f60d246b2c7d0dfd` (42 files / 346702 B).
No `docs/11*`, no `crates/`, no `docs/development`. Node-1 public pack intersection holds.

Wrapper permits `branch, capabilities, commit, context, doctor, explain, extinguish, scan,
show, status, verify` plus `--help`. It blocks init/import/open/close/upload/request-*/serve/gc/migrate/…
and non-`$FIX` targets. After every command it recomputes P1–P7. It re-serializes JSON
envelopes with `json.dumps(..., indent=2, sort_keys=True)` (§8). It does not prescribe sequence.

T0 `2026-08-22T05:08:42Z` (epoch `1787375322`). Stage-C freeze snapshot
`$CTRL/stage-c-freeze.json` SHA-256
`9e96e636dbdce7d6e6c33bccb71382cdcce8c9019998c9f8998e456587b42d41`.

## 3. Same-uid / source-blind limitations (park, not block)

Containment is **operational, not kernel-enforced** (shared uid). `$PART/wrapper.json` names
`$RUNNER/preserve-baselines.json`, which names `$KEY`. A participant who walked those paths
could read the answer key.

Independent checks used instead of trusting the instruction:

- transcript argv: 19 wrapper invocations; cwd is `$PART` or `$FIX`; no `$KEY`, `crates/`,
  `tests/` of the product, or `docs/development` path;
- KEY atimes all `2026-08-22T05:08:38Z` (Stage A freeze), **before** T0 `05:08:42Z` and
  before first command `05:14:23Z`;
- TASK/SETUP have no `block-m1`, threshold, `live_open`, or family-name leakage;
- filled answers use run envelopes, not key-only values (dynamic head is the participant
  seal, not the baseline-only key).

Honest limit, restated: absence of KEY/source reads is evidenced for wrapper argv and KEY
atime, not by a kernel sandbox. **`park`**, same as node 1 §7.

## 4. Participant edit ordering and genuine first exit-10

Independent mtimes vs first wrapper command:

| Path | mtime UTC | vs Stage A `05:08:35Z` |
|---|---|---|
| `docs/spec/receipt.md` | `05:14:14Z` | **changed first** — lowercase requirement + ACME vector |
| first CodeFire command | `05:14:23Z` | `commit --dry-run --json` **exit 10** |
| `docs/design/receipt.md` | `05:14:31Z` | changed **after** the refusal |
| `src/receipt.py` | `05:14:33Z` | `.hexdigest()` (no `.upper()`) |
| `tests/test_receipt.py` | `05:14:35Z` | expects lowercase ACME vector |
| `README.md`, `docs/unrelated-todo.md`, `src/legacy_currency.py`, three yaml files | `05:08:35Z` | **byte-identical** to Stage A |

Envelope `001-commit.json` (file SHA-256 = transcript stdout SHA-256
`4776897f66f3450aa7ea46057f39eb7d7d89c8d659d1cf3b90fc8e2ba19ec9c1`):

```text
exit_code=10  ok=false  command=commit
data.applied=false  data.blocked=true  data.commit_id=null
data.changed_atoms=["REQ-receipt-digest"]
data.change_coverage.entries[0].path=docs/spec/receipt.md
data.non_atom_changed_file_count=0  data.non_atom_changed_files=[]
data.verification.result=failed
data.verification.primary_reason=open_required_fires
data.verification.open_required_fires=2
missing_required_links=0 stale_resolutions=0 duplicate_atom_ids=0 failed_checks=0
effective_policy.rules id=commit.require_no_required_fires
  effective_value=true safety_class=safety_floor source=repository_config
```

This is a **genuine intended refusal** on a spec-only inconsistent state, recorded **before**
downstream design/code/test mtimes. F2 is exercised. The refusal is not a usage error.

After propagation, `002-scan.json` `open_fire_count=6` with display IDs
`FIRE-8C311F887373`, `FIRE-CE36FE5CF717`, `FIRE-289772C33BA7`, `FIRE-7CBE4E552E41`,
`FIRE-692C303307AF`, `FIRE-98E8262DB3C8`. Six extinguish commands exit 0
(envelopes `003`–`008`). Extra Atom `TEST:tests/test_receipt.py::class:ReceiptDigestTests`
is on the **allowed** test file (unittest class auto-extract); not an off-task file.

## 5. Declared file changes and untouched temptations

Stage A user-file SHA-256 vs audit-now:

| Path | Stage A | Now | Disposition |
|---|---|---|---|
| `docs/spec/receipt.md` | `06b99cc7…` | `be00bbeb60458583243b1655e7355da315ce779dcf013853881512de6b75535a` | declared edit |
| `docs/design/receipt.md` | `4b371223…` | `a9b9f046f7d70e4f749f487db86a99503ec4f5172c549001a57ab1d08cccd531` | declared edit |
| `src/receipt.py` | `8b61017c…` | `09a981326efe276fb1d01d4997051edb8a00d91c5b9efc727855c73717edf424` | declared edit |
| `tests/test_receipt.py` | `8751df36…` | `464f3169b5cf30ca971d30c66b582bb9a713fe7bef366409dcb8ff81d3e1c37d` | declared edit |
| `README.md` | `28bf42ec6afb655f92dd02612bb9621f1f492c6a67688e6bc9cb363c2c6e2e6b` | **same** | temptation-adjacent, **untouched** |
| `docs/unrelated-todo.md` | `f85011036feab437e496038c4dba528c5c96ac753ed8ec713e8bd3626a1f3273` | **same** | named temptation |
| `src/legacy_currency.py` | `3c17ceec6d4f8b6fe0e6f9a0772250138e7eab8af5c05b4301b06e4b6c3001d2` | **same** | named temptation |
| `codefire.yaml` / `.links.yaml` / `.policy.yaml` | Stage A hashes | **same** | not in the four allowed edits |

`codefire.yaml` globs are `docs/spec/**/*.md`, `docs/design/**/*.md`, `src/**/*.py`,
`tests/**/*.py`. `README.md` is **outside** those globs and **outside** the four allowed
edits. Live files: spec/design/code/test are lowercase; `canonical_receipt_digest` returns
`.hexdigest()`; test expects `e3e3be6e7f17003100a2953c6dcb8a618e117f4cdcc2123b92f3db866b4731bd`.

`010-commit.json` `changed_atoms` are the four declared Atoms plus the extra TEST-class
Atom; `non_atom_changed_files=[]`. `off_task_mutation_count = 0`.

P8 now PRESERVE_DIGEST `f2ccae515878c2397fc6b4773deda1463a255e478e77c4e308b9fe6f6bcffd39`
(62 files / 94801 B). Stage A P8-before `d1096f1c…` (10/3120) and P8-after-seal `a00f4662…`
(49/61653) remain on disk as labelled `PRESERVE_DIGEST` records.

## 6. README uppercase fixture — product coverage limitation, not a silent fix

`README.md` still states the sealed contract is **uppercase** hexadecimal. Independently
read from disk. It is not an Atom. `scan`/`verify`/`commit` did not see it
(`non_atom_changed_file_count=0` on both the refusal and the seal).

Leaving it untouched is **correct** under the frozen work order (`off_task_mutation_count`
stays 0; touching it would have been off-task). Under the frozen rubric:

- it does **not** fail Q1–Q7;
- it does **not** remove the M16 +1 (the requirement Atom, not README, is the sealed requirement);
- it **is** an explicit value/coverage limitation: a human-facing README can contradict the
  sealed requirement without the first-seal loop noticing.

Adjudication: **`park` as a product coverage limitation**. Do **not** edit README, yaml, or
product docs in this slot. Do **not** count it as off-task. Do **not** coerce the table to
`adopt` by pretending the seal covers it. The frozen consequence of the matching `narrow`
row is to **narrow the public claim**, not to fix README.

## 7. Unit-test / Fire / verify / commit / status / branch / show / doctor

Participant path (transcript 19 records, SHA-256
`0ae5b6454cb327a8d09341d2c7ff606fc5e2cf3749be9f47b6b322471c193813`, 10917 B):

| Env | Command | Exit | Independent facts |
|---|---|---:|---|
| 001 | `commit --dry-run` | 10 | genuine open_required_fires; spec-only |
| 002 | `scan --full` | 0 | 6 required fires; extra TEST-class Atom on allowed file |
| 003–008 | `extinguish` ×6 | 0 | distinct Fire IDs; rationale recorded |
| 009 | `verify --details` | 0 | `data.result=passed`; `decision.outcome=passed`; `open_required_fires=0`; `execution[0].id=unit-tests` `exit_code=0`; next_actions[0] `kind=commit` `ready=false` |
| 010 | `commit -m …` | 0 | `applied=true` `commit_id=CF-COMMIT-d6bb8b1d159ad489dc67ec92` `branch=main` `next_actions=[]` `extinguished_fire_count=6` `active_resolution_count=6` |
| 011 | `show main --path` | 2 | `unsupported_option`; next_actions `kind=show_help` |
| 012 / 018 | `branch list` | 0 | `main` head `CF-COMMIT-d6bb…` `state=open-clean` |
| — | `show --help` | 0 | no envelope; recovered isolation (`cwd=$FIX`) |
| 013 | `show main` cwd=`$FIX` | 0 | head/parent/certificate/claim/signature as below |
| 014 | `show CF-COMMIT-3456…` | 0 | parent; message `Seal uppercase receipt-digest baseline` |
| 015 | `status` | 0 | `main` `open-clean` `open_fires=0` `base=CF-COMMIT-d6bb…` |
| 016 | `doctor --path` | 0 | `ok=true` `mode=full` issues 0, `checked.objects=43` |
| 017 | `capabilities --json` | 0 | `commit.targets=["local"]` |

`013-show.json` independently:

```text
data.show.commit          CF-COMMIT-d6bb8b1d159ad489dc67ec92
data.show.parents[0]      CF-COMMIT-3456b0494fd47761a0522cac
data.show.target          main
data.show.certificate     passed          (string, not an object)
data.show.claim.result    passed
data.show.claim.profile   local_verification_v1
data.show.claim.phase     pre_commit
data.show.claim.subject   kind=live_open trust_kind=local_unisolated immutable=false
data.show.signature.present false
```

Controller confirmation used `$CF` (not the wrapper). Transcript hash is unchanged
`0ae5b645…`. ANSWERS hash is unchanged `14b29057…`. Parsed equality:

| Confirm file | SHA-256 | vs participant |
|---|---|---|
| `status.json` | `4b862fd3f704ac8cda1119a049555cbb453580af3e325174a5781fcd74ba9034` | **byte-equal** to `015-status.json` |
| `show.json` | `92d5600756eb4f87e6062c2de06dd419aa4f267e9375967de49360912f8d59d8` | **byte-equal** to `013-show.json` |
| `branch-list.json` | `d0356b6bcb5eda603a8446b8011198c6f8005c8c42350a76905f13725a856610` | **parsed-equal** to `012`/`018` (raw CodeFire key order) |
| `doctor.json` | `2e37b67814eec2f42f3eb9ba2f5211dd784f40fafb8ffe2ad992d14dcc8771ab` | distinct `--quick`: `ok=true` `mode=quick` issues 0 `skipped_checks=["object_store_integrity"]` `objects=0` |

`show --path` is usage exit 2 on `$FIX`, not a forbidden subcommand and not a non-fixture
target. `wrong_action_count = 0`.

### Envelope-file vs stdout hash (explained, not a content mismatch)

For 17/18 JSON envelopes, saved-file SHA-256 equals transcript `stdout_sha256` because
CodeFire pretty-print happened to match the wrapper's `sort_keys` rewrite. For
`branch list` (012 and 018) they differ:

```text
transcript stdout_sha256 / confirm raw file   d0356b6b…   (CodeFire key order, schema first)
wrapper-saved envelope file                   717a07b1…   (sort_keys, command first)
parsed JSON                                   equal
```

Cause is the wrapper (`save_envelope` re-serializes). Semantic payload is the same object
independently parsed from both files. **Not** an unexplained mismatch.

## 8. Answer / transcript hashes (frozen before this audit)

| Artifact | Independent SHA-256 | Notes |
|---|---|---|
| `$PART/ANSWERS.md` | `14b29057cb643ccbe6488a32397651242d33d6f7f974470587bddbe54a011243` | 22929 B; freeze and confirm both record this |
| `$EVIDENCE/participant/transcript.jsonl` | `0ae5b6454cb327a8d09341d2c7ff606fc5e2cf3749be9f47b6b322471c193813` | 19 records / 10917 B |
| `$PART` PRESERVE_DIGEST | `0d515dcf89da163c9a9aa886b6938b9e780225c650730fea48543f9b3ff8153f` | 5 files / 34470 B |
| `$EVIDENCE` PRESERVE_DIGEST | `245f3f877322137d1b48c30bf52f453cbd6d6517c09d622aec4464811df9464a` | 72 files / 421030 B |
| `$FIX` PRESERVE_DIGEST | `f2ccae515878c2397fc6b4773deda1463a255e478e77c4e308b9fe6f6bcffd39` | 62 files / 94801 B |

Every listed freeze envelope hash was rehashed; all match `stage-c-freeze.json` except the
explained branch-list stdout-vs-file split above, which the freeze recorded as the
**file** digest `717a07b1…`.

## 9. Independent Q1–Q7 score

Graded once against node-1 Result §5 accept/reject clauses from frozen answers + envelopes.
Controller scored 7/7. Any disagreement resolves against `adopt` (lower score stands).

| Q | Independent | Controller | Note |
|---|---:|---:|---|
| Q1 | **1** | 1 | head `CF-COMMIT-d6bb…`, parent `CF-COMMIT-3456…`, branch `main`; `branches[].state` treated as persisted, not live-open authority. Envelope **filename** mix-up (`014-show.json` labelled as `show main`; actual `show main` is `013-show.json`; parent is `014`) does **not** invert IDs — the cited `data.show.commit` / `parents[0]` / `target` / `label` values live on `013`. Park citation defect, not an invented field. |
| Q2 | **1** | 1 | `certificate` as result string `passed`; claim profile/phase/subject; non-assertion of authorship (`signature.present=false`) and of span coverage (`span_precision=file_only_no_span_claim`). |
| Q3 | **1** | 1 | local/embedded; `signature.present=false`, `live_open`, `local_unisolated`, `immutable=false`; not over-corrected to “invalid seal”. |
| Q4 | **1** | 1 | two absent authorities from fields actually present: signature/authorship and execution isolation (`trusted_local_shell_v1` / `local_unisolated`). |
| Q5 | **1** | 1 | actual `next_actions[0]` `kind=commit` and `command` from `009-verify.json` were followed; exit 10 meaning from the taxonomy. Quoted `target` used the docs-pack fire-plan shape `{branch, open_dir, repo_root}` instead of the actual verify-commit target `{base_commit, changed_atoms, path, …}`. That is a **citation paraphrase**, not a fabricated next action (the commit action existed and was executed). Park. |
| Q6 | **1** | 1 | first refusal exit 10 / `open_required_fires` as safety floor; `require_no_required_fires` cannot be switched off; only `require_trace_completeness` is relaxable. |
| Q7 | **1** | 1 | own numbers, a later question the seal answers that a green test run does not (resolution ledger / fire delta on the four-artifact change), and a non-worth case (no trace graph / portable-proof need). |
| **Total** | **7/7** | 7/7 | `misunderstanding_count=0`, `unanswerable_count=0` |

F1 is not triggered (Q3 correct; Q2/Q4 do not overstate portable “verified”).

## 10. Independent metrics (node-1 §6; machine timings)

From transcript `t` and ANSWERS mtime. Participant restated M1/M2/M4/M6 discarded where they
used first-edit, the exit-10 refusal, pre-authoring confirmation, or help=1.

| Metric | Independent value | Source |
|---|---|---|
| M1 `setup_time_s` | **363** | t0 → first exit-0 fixture command (`scan` `05:14:45Z`) |
| M2 `ttfur_s` | **363** | t0 → first `scan` listing changed Atoms/fires, exit 0 |
| M3 `ttfs_s` | **402** | t0 → `010-commit.json` `05:15:24Z` `applied=true` |
| M4 `ttac_s` | **628** | t0 → ANSWERS.md mtime `05:19:10Z` |
| M5 | **19 / 19 / 9** | commands / codefire commands / distinct subcommands `commit,scan,extinguish,verify,show,branch,status,doctor,capabilities` |
| M6 `help_invocations` | **2** | `show --help` + `capabilities --json` |
| M7 `docs_files_read` | 6 listed | README, runbook, automation_interface, 04_cli_spec, ADR-016, ADR-011; citations match those files |
| M8 | intended **1** (exit 10); usage **1** (exit 2); unexpected **0** | wrapper refusals 0 |
| M9 `retry_loops` | **6** mechanical consecutive same-subcommand pairs (5 extra `extinguish` of distinct Fire IDs + 1 extra `show` parent vs head). Substantive failed retries: 0 |
| M10 | **0** | |
| M11 | **0** | |
| M12 `undocumented_field_reliance` | **6** (controller recorded **4**) | see §11 |
| M13 `wrong_action_count` | **0** | `show --path` is usage, not forbidden / non-fixture |
| M14 `off_task_mutation_count` | **0** | temptations + README byte-identical |
| M15 | **0 / 0** | private_correction_events / interventions |
| M16 `task_value_score` | **3** | +1 independent reader can name the requirement change and recorded resolutions from `show`/`commit`; +1 genuine exit-10 cleared; +1 Q7 names a later question the seal answers |

`maintenance_friction_estimate` (labelled estimate, not a measurement): 0 config lines
authored; 6 resolutions for the REQ/DES/CODE/TEST neighborhood; mechanical extinguish×6 +
verify + commit would repeat. Every §6 metric is recorded. Timings carry the node-1 §2
limitation: **machine times, not human onboarding**.

## 11. Mechanical M12 against the handed docs/schema pack

M12 definition (frozen): distinct JSON field **paths** cited in **correct** answers that
appear in neither the docs pack nor `schemas/public/**`. Japanese/English **prose** is not
the JSON identifier. No threshold change. No coercion of prose into keys.

Handed pack: 42 files. Exact-string search (identifier must appear as that token):

| Cited JSON identifier | Hits in pack + `schemas/public/**` | Nearby prose (not a key) |
|---|---:|---|
| `cwd_kind` | **0** | `cwd classification`; `repo-relative cwd` (`04_cli_spec.md:467`, `automation_interface.md:363`, `04_cli_spec.md:753`) |
| `subject_unchanged` | **0** | “exit 0でもworktree subjectが変わればblocking failure” (`04_cli_spec.md:467`) — does not name this key |
| `extinguished_fire_count` | **0** | “extinguished fire count” as Japanese-mixed prose (`04_cli_spec.md:537`) |
| `active_resolution_count` | **0** | “active resolution count” as Japanese-mixed prose (`04_cli_spec.md:537`) |
| `worktree_before` | **0** | “worktree before/after digest(s)” (`04_cli_spec.md:467`, ADR-016:46, `automation_interface.md:363`) |
| `worktree_after` | **0** | same prose |

The four fields the board named are **all undocumented as identifiers**. They exist in the
actual envelopes (`009-verify.json` `execution[0].cwd_kind=repo_relative`,
`subject_unchanged=true`; `010-commit.json` `extinguished_fire_count=6`,
`active_resolution_count=6`). They were cited in correct Q2/Q3/Q7 answers.

Independent M12 therefore also includes `worktree_before` / `worktree_after` (cited in Q2
as `execution[0]` fields; same prose-vs-identifier class). Controller recorded only the
four named keys (**4**). Independent count is **6**. Both are `>= 2`. The first-match row
does not change. Lower/stricter count is the independent one; it is **not** used to invent
a new threshold.

Documented citations (examples, not M12): `span_precision` / `file_only_no_span_claim`,
`environment_digest`, `environment_policy`, `termination_scope`, `scan_prediction`,
`safety_class`, `live_open`, `local_unisolated`, `trust_kind`, `signature.present`.
Invented nonexistent fields were not found in the Q2/Q3/Q4/Q6/Q7 proof paths. Q5's
paraphrased target keys `open_dir` / `repo_root` **do** appear in the docs pack
(`automation_interface.md:530`) so they are not M12; they are the Q5 park above.

## 12. Absence of hidden telemetry, Git bridge, hosted service, agent platform, network, private correction

| Prohibited thing | Verdict | Independent basis |
|---|---|---|
| Product source / tests / `docs/development` / `docs/11` handed | **absent** | `$DOCS` 42-file inventory; TASK/SETUP leak scan empty |
| Private correction / intervention | **absent** | M15=0; wrapper `note=refused` count 0; no second participant |
| Git as VCS / network / install | **absent** | transcript argv; no `git` / URL / `install.sh` |
| Forbidden subcommands | **absent** | no init/import/open/close/upload/request-*/serve/gc/migrate/clone |
| Telemetry / hosted service / agent platform | **absent** | no product instrumentation; local evidence root only |
| Python runner as participant CLI | **absent** | wrapper execs `$RUNNER/codefire` Rust binary |
| Threshold / §3–§9 edit after fixture existed | **absent** | node-1 packet bytes vs run packet “byte-for-byte” claim; this audit did not edit them |
| Second Stage B pass / CodeFire re-run by this audit | **absent** | one monotonic participant window `05:14:23Z`–`05:17:17Z`; answers `05:19:10Z`; this audit issued no CodeFire command |

One-pass: 19 wrapper invocations once. Controller confirmation is a **distinct** read-only
`$CF` pass after answer freeze, as Stage C required; it did not mutate answers or the
participant transcript.

## 13. P1–P7 preservation (algorithm `PRESERVE_DIGEST`)

Definition independently applied: `sha256(concat of "<sha256(content)>  <relpath>\n"` over
regular non-symlink files, relpaths sorted; skip `.git` and `target`).

Study-session wrapper baseline HEAD was `a95ba684d409744bceae2700d3bb8170d05f8dbb`. Wrapper
rechecked P1–P7 after every command; 0 preservation refusals. Stage C packet commit moved
the node-2 worktree to `73e1455916f9310f12b9480c469e2b53693a09c2` (docs-only score packet).
That is **not** participant mutation. This audit worktree is `ca955fc` (docs-only admit).
Do not treat either docs commit as study-run drift.

Independently **now**:

| ID | Now | Match to freeze |
|---|---|---|
| P2 `/workspace/project` | HEAD `f1644122134d479333cf422fdb1d48b8fa92dd8e`; porcelain 248 SHA-256 `1385b6249f6455510bc449aed054d5ce425494a29edb9a4c21d65165cb6b1bcf` | **yes** |
| P3 Algorithm lab | `516ac0056154eb19dd395c68f8bebe1da0ff2aefd3de78fe3f3f676a5c2f5d40` (183 files / 1630240 B) | **yes** |
| P4 user install | no PATH `codefire`; `~/.local/bin/codefire` absent; `/usr/local/bin/codefire` absent | **yes** |
| P5 historical runner | `90cb2895…` (14590776 B); not the admitted runner identity | **yes** |
| P6 `$KEY` | `5b995f6f397e0df0e06485b07cba20538336c7c184a4e351e9a2c2af80d7315d` | **yes** |
| P7 `$DOCS` | `31f450a3a315730f243519518fa4c9e2de313067232b6aa3f60d246b2c7d0dfd` | **yes** |

Runner binary still `90cb2895…`. TASK/wrapper/SETUP hashes unchanged from Stage A.

## 14. Ordered table — first match wins, no threshold change, no coercion

Frozen node-1 Result §9, evaluated **once** on independent values:

| Order | Decision | Independent evaluation |
|---|---|---|
| 1 | stop | **no** — P2–P7 match; wrapper recorded 0 P1 drift during the run; `wrong_action_count=0`; `private_correction_events=0`; no extra root read in argv; no §3–§9 edit |
| 2 | stop | **no** — seal applied (`data.applied=true`, exit 0) |
| 3 | stop | **no** — seal applied; rubric **7/7**; `ttfs_s=402 ≤ 7200` |
| 4 | stop | **no** — Q3 correct (local, not portable; missing signature not read as invalidating the seal) |
| 5 | **narrow** | **yes** — `undocumented_field_reliance` independent **6** (controller 4) `>= 2`. Also: Q2/Q4 correct; `task_value_score=3`; genuine refusal present; `ttfs_s=402`; unexpected 0; off_task 0; every §6 metric recorded |
| 6 | adopt | not reached (M12 exceeds `<= 1`; first match already won) |
| 7 | narrow | unused |

**Literal decision: `narrow`.**

Precommitted meaning applied without (ii): admit no product write. Choose **(i)** narrow the
product's public claim wording to what this evidence supports — one clean-room AI
participant completed a local first-seal with correct local-versus-portable reading, but
the handed docs pack does not enumerate every JSON key the envelopes return, and a non-Atom
README can drift. **Do not** run (ii) from this audit. Retry requires a later orchestrator
admission that this frozen table does **not** grant. No node-4 product write. No
README/docs/schema fix. No threshold edit.

`block-m1`: **none**.

## 15. Findings

### `block-m1` (0)

None. The study protocol held. The frozen table returns `narrow`. That is a completed M1
decision, not a reason to block exit and not a reason to adopt.

### `park` (not now-work)

1. **M12 docs-pack identifier gap** — `cwd_kind`, `subject_unchanged`,
   `extinguished_fire_count`, `active_resolution_count`, and independently also
   `worktree_before` / `worktree_after`, exist in envelopes and were cited in correct
   answers but are not JSON identifiers in the handed pack. Prose exists. This is the
   tripwire that produced `narrow`. Do not “fix the docs” from this slot.
2. **README non-Atom coverage hole** — stale uppercase README outside `codefire.yaml` and
   outside allowed edits. Correctly left untouched. Public claim must not say the seal
   covers human-facing non-Atom files.
3. **Q1 envelope-filename mix-up** — `014` vs `013` for `show main`. IDs not inverted.
4. **Q5 target-shape paraphrase** — docs-pack `{branch, open_dir, repo_root}` quoted in
   place of the actual verify-commit `target`. Action itself was real and followed.
5. **Wrapper envelope re-serialization** — `sort_keys` rewrite makes `branch list`
   file digest differ from stdout digest; parsed JSON equal. Future studies should hash
   raw stdout as the envelope bytes.
6. **Extra TEST-class Atom** on the allowed test file; verify still passed.
7. **Same-uid operational containment**; runner handed not acquired; docs pack is the
   repository public surface not the three-doc release payload; `live_open` /
   `local_unisolated` / `immutable=false` is the measured Q3/Q4 subject; import
   `next_actions` mixed contract remains controller-setup `park`; n=1 AI path only;
   machine timings only.
8. **CFR-171..173 / CFR-168 / CFB-136 / CFB-133 / CFR-169 / CFR-170 / `open-clean` vs
   `opened=0` / request-list `applied_head`** — unchanged `park`. None reproduced as the
   smallest first-seal blocker.

### `drop`

Human recruitment, market/ICP/pricing claims, portable signing so Q3 could say
“portable”, Git bridge, hosted service, agent platform, telemetry in core, Python
participant runner, a second participant, README/docs/product fix, threshold edit, M2–M5.

## 16. Authorization for node 7

**Authorized:**

1. Treat this node-6 audit as terminal **`ACCEPT`** with **`block-m1`: none**.
2. Record the study decision as literal **`narrow`**, not `adopt` and not `stop`.
3. Exit M1 `MS-product-value-proof` on that `narrow` + this ACCEPT, then apply the board
   shift rule: record exit evidence in `history.md`; rename slots (old M2 becomes M1, and
   so on); name a new M5 from `strategy/modernization-roadmap.md` without turning that
   roadmap into a task board; refuse any in-flight parked work that tried to start early.
4. Attach every standing limitation in §15.7 and node-1 §2: n=1; AI participant is not
   human adoption; timings are machine timings; no toolchain-free adoption; no
   third-party/portable value; release-payload docs sufficiency untested; no market,
   pricing, or segment-size claim; README non-Atom coverage untested-as-covered; JSON
   envelope keys are not fully enumerated by the public docs pack.
5. Preserve `/tmp/codefire-m1-pvp.rh8k89q9` through that record.

**Refused:**

1. Consequence (ii): bounded correction + rerun. **No retry.**
2. Any product, source, test, public-doc, schema, README, yaml, or fixture fix of M12 keys
   or the stale uppercase README.
3. Any threshold edit, rubric edit, or coercion of `narrow` into `adopt`.
4. Starting M2 `MS-ecosystem-conformance-seed` as if this were an adopt, or starting any
   parked issue (CFR-168..173, CFB-133/136).
5. Human/market claims, telemetry, Git bridge, hosted service, agent-platform code,
   portable signing, public release.
6. Mutating canonical Algorithm, dirty main `/workspace/project`, user PATH install, or
   deleting the preserved study root as part of “closing” M1.
7. Any product/source/packet/issue edit as part of closing M1 beyond the
   board/history/current-state docs that node 7 owns.

Exact node-7 handoff:

```text
audit       docs/development/ai/audit-latest.md
verdict     ACCEPT
block-m1    none
decision    narrow
consequence (i) narrow the public claim; no retry; no product fix
root        /tmp/codefire-m1-pvp.rh8k89q9
answers     14b29057cb643ccbe6488a32397651242d33d6f7f974470587bddbe54a011243
transcript  0ae5b6454cb327a8d09341d2c7ff606fc5e2cf3749be9f47b6b322471c193813  (19 records)
head        CF-COMMIT-d6bb8b1d159ad489dc67ec92
base        CF-COMMIT-3456b0494fd47761a0522cac
score       7/7
M12         6 independent (controller 4; both >= 2)
task_value  3
ttfs_s      402
write set   board / history / current-state docs only
```

Orchestrator may close M1 and shift slots on this ACCEPT. It must record `narrow`. It must
not start parked work, retry the participant, or inherit an adopt/public-value claim.

## 17. Validation run by this audit

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

Results recorded with the commit. No build, no product test, no CodeFire command of any
kind, no participant, no worktree switch, no network, and no later-slot gate. Write set is
this file only.

## 18. Verdict

**ACCEPT.**

The isolated first-seal study at `$ROOT=/tmp/codefire-m1-pvp.rh8k89q9` is source-bound to
`3415914` / rust input manifest `3925fc03…`, source-blind, one-pass, and preserved on
P2–P7. The participant sealed `CF-COMMIT-d6bb8b1d159ad489dc67ec92` on parent
`CF-COMMIT-3456b0494fd47761a0522cac` after a genuine spec-only exit-10, scored 7/7 with
task value 3, and left both named temptations and the stale uppercase README untouched.
Independent first-match of the frozen table is **`narrow`** because undocumented JSON
field reliance is 6 (controller 4). The README hole is a coverage limitation under that
same table, not a product fix to perform now.

`block-m1`: **none**.

Node 7 may exit M1 and shift slots. It must record literal `narrow`, narrow the public
claim, and must not retry, fix README/docs/product, or inherit an adopt claim.
