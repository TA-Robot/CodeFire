# AI Task Packet

Status: completed

Packet ID: MS-adoption-wedge-contract

Schema version: human-v1

Risk score: 4

Reasoning tier: high

## Objective

Select the smallest second-repository or source-blind consumer task that can prove or falsify active
M1 `MS-adoption-wedge`, and freeze its exact public commands, fixture, preservation boundary,
comprehension questions, measurements, and stop rule before any experiment mutation.

## Scope

- active milestone board and CFR-171..173 public problem/acceptance text;
- public CLI help, public schemas/specs, and previously sealed/read-only evidence;
- read-only inspection of candidate repository shapes;
- this packet's Result only.

Start public-only. Do not read product implementation source or tests to compensate for an absent
consumer contract. Clearly distinguish an absent public contract from an implementation bug.

## Questions

1. Which exact second repository, isolated target, or source-blind consumer is the smallest credible
   subject, and why is it independent enough to test adoption rather than maintainer memory?
2. Is inspect or isolated import the smaller honest path? What exact public argv/JSON fields answer
   repository, branch, base/head, claim/proof kind, and missing-authority questions?
3. What before/after digests prove the non-target repositories were not mutated?
4. What setup-time, first-useful-result, misunderstanding, wrong-action, and maintenance measurements
   can be gathered without telemetry or private clarification?
5. What precommitted thresholds produce adopt, narrow, or stop, and what is the first missing contract?

## Validation

Set Status `completed` or `blocked`, classify every finding `block-m1` / `park` / `drop`, and run:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

## Non-Goals

- product/source/test/schema implementation;
- running an import or mutating any candidate repository;
- reading private implementation to invent undocumented flags or fields;
- broad interviews, telemetry, a Git bridge, an agent platform, release work, or M2–M5 work;
- editing the board, issues, another packet, or canonical dogfood state.

## Handoff

Return exact fixed HEAD, subject/consumer topology, public argv, expected JSON facts, preservation
manifest, comprehension/measurement rubric, adopt/narrow/stop thresholds, first blocker, issue mapping,
and next legal action. Commit only this packet.

## Result

Status **completed**. Verdict **one `block-m1`** (the first missing public contract, admitted as the
smallest CFR-171 slice), plus five `park` and three `drop`.

A credible, fully bounded, zero-mutation adoption subject exists: a **source-blind clean-room consumer
performing read-only sealed inspection of an isolated copy of the canonical CodeFire store**, using
five documented public commands and no open directory, no mutator, and no remote. Inspect is the
smaller honest path; isolated import is rejected below with reasons. The public contract was frozen
from `--help`, `capabilities --json`, `schemas/public/`, `schemas/conformance/manifest-v1.json`,
`docs/04_cli_spec.md`, `docs/automation_interface.md`, `docs/runbook.md`, `docs/02_core_concepts.md`,
and `README.md` before any repository data was inspected. No product implementation source
(`crates/**`), no product test (`tests/**`, `crates/**/tests.rs`), and no
`docs/11_internal_architecture.md` was read. No repository was mutated. No experiment was run.

### 0. Fixed HEAD, runner, and environment identity

```text
integration HEAD (exact)   f91c8ca429dff5f93b9d88c35890a20f72435e27  ("docs: close remote M1 and shift to adoption")
packet worktree branch     claude-agent/m1-adoption-contract-20260822
runner (admitted input)    /workspace/worktrees/ms-remote-happy-path-budget-20260821/target/release/codefire
runner sha256              70301c874d7220a71be988562df0e5188ede6a85c65c63dcbd3dcdee09f4706c
runner inode / bytes       1361045 / 14594080   (hardlink of codefire-rs, per audit-latest.md)
runner<->HEAD binding      git diff --quiet 510a185 HEAD -- Cargo.toml Cargo.lock crates .cargo rust-toolchain rust-toolchain.toml  -> exit 0
runner version banner      codefire 1
toolchain in this env      cargo: absent   rustc: absent   python3: 3.10.12
repo-root ./codefire       Python MVP v0.2; no --json on any command; no context/capabilities/explain/import/storage/migrate/patch/evidence/link
```

Because the Rust input tree is byte-identical between `510a185` and `f91c8ca`, the audited runner is
the correct executable for this HEAD. It is admitted as a **provided input**, not as something the
consumer acquires (see `park` P2/P3 — this is the honest scope limit of M1).

### 1. Q1 — selected subject and consumer, and why it is independent enough

**Subject S — isolated sealed copy of the canonical store.**

```text
S root            $FIX = $ROOT/repo                     (fresh mkdtemp root, e.g. /tmp/m1-adoption-wedge.XXXXXX)
S contents        $FIX/.codefire/repo.json               copied from /workspace/project/subprojects/.codefire/repo.json
                  $FIX/.codefire/objects/                copied (2891 files, 114921765 bytes)
                  $FIX/.codefire/branches/               copied (8 files)
                  $FIX/.codefire/{opened,active,cache,idempotency,locks,remotes}/   created empty
S shape           CLOSED repository: no open directory, no worktree, no active state, no remote
S branches        main.json plus 7 URL-encoded feature/* heads (feature%2F...)
```

`S` is a *closed* repository on purpose. That is what makes this the smallest credible surface: the
consumer never needs `open` (a mutator), so `status`, `scan`, `verify`, `context`, `fire`,
`extinguish`, and `commit` are all off the contract, and the whole live-open trust boundary,
repository-lock, `state_transaction.json`, and idempotency machinery is out of scope. Every question
M1 asks can be answered from sealed history plus the capability manifest.

**Consumer C — fresh source-blind agent session.** Other-family (Grok 4.6) preferred; the executing
owner is chosen at node 3 per `orchestration-protocol.md`. C receives exactly three things:

```text
1. the runner path (execute + read only)
2. $FIX                     (read-only; the ONLY admitted target)
3. $ROOT/docs-pack/         a copied, frozen public documentation set:
                              README.md
                              docs/02_core_concepts.md
                              docs/04_cli_spec.md
                              docs/05_consistency_model.md
                              docs/automation_interface.md
                              docs/runbook.md
                              schemas/public/{capabilities-v1,command-result-v1,page-v1,page-cursor-v1,remote-doctor-v1}.schema.json
                              schemas/conformance/manifest-v1.json
```

C receives **no** `crates/**`, no `tests/**`, no `docs/development/**`, no
`docs/11_internal_architecture.md`, no `docs/08_object_store.md`, no `docs/09_index_trace_policy.md`,
no issue text, no packet, and no maintainer clarification channel.

Why this is independent enough to test adoption rather than maintainer memory: the *artefact* is
maintainer-produced (unavoidable — only the maintainer owns a sealed CodeFire repository at this HEAD,
which is exactly why the milestone exists), but the *reader* has zero prior exposure to CodeFire's
model, source, naming, or internal docs. M1 exit bullet 3 is a comprehension test on a reader, not a
provenance test on an artefact, so reader independence is the property that matters. Enforcement is
by containment (`$ROOT` is the only tree handed over) plus a post-hoc argv/transcript scan for any
read outside `$ROOT`; the honest limit is that containment is operational, not kernel-enforced, so the
node-6 audit must re-check the transcript rather than trust the instruction.

**Rejected alternatives** (recorded as `drop` D1–D3 in section 8): the conformance-fixture-only
consumer, the prior milestone's preserved evidence root, and a freshly imported toy repository.

### 2. Q2 — inspect vs isolated import, and the exact public argv/JSON fields

**Inspect is the smaller honest path.** Isolated import would add: mutation authority, the
`.codefire-import.lock` bootstrap lock and durable import transaction, the five-phase
planned/initialized/opened/scanned/completed recovery contract, idempotency-key semantics and exit 33,
the 1 MiB operational-JSON and 47.25 MiB per-file caps, a repository lock, and a `scan` publication —
and it would still require `commit` before any certificate exists to reason about. None of that is
needed to answer M1's five identification questions, and every item of it is a fresh preservation
hazard. Import is therefore not selected; it stays available as a later slot if inspect adopts.

**Frozen command set (exactly these six; C6 optional).** `$CF` = admitted runner. All read-only.
`show` has no `--path` flag (`codefire show <branch-or-commit> [--json]`), so `show` **must** run with
`cwd=$FIX`; every invocation records `pwd` in its `.meta`.

| # | argv after `$CF` | cwd | required | documented at |
|---|---|---|---|---|
| C1 | `capabilities --json` | `$ROOT` | yes | `04_cli_spec.md` §4.1, `automation_interface.md` |
| C2 | `doctor $FIX --quick --json` | `$FIX` | yes | `04_cli_spec.md` §4.3 |
| C3 | `branch list --path $FIX --json` | `$FIX` | yes | `04_cli_spec.md` §4.1 |
| C4 | `show main --json` | `$FIX` | yes | `04_cli_spec.md` §4.16 |
| C5 | `show <parent-commit-id> --json` | `$FIX` | yes | `04_cli_spec.md` §4.16 |
| C6 | `doctor $FIX --json` | `$FIX` | optional, at most once | `04_cli_spec.md` §4.3 |

C1 is repo-free and was verified at this HEAD to return `ok=true`, `exit_code=0`, `repo=null`.
Anything outside C1–C6 is an off-contract invocation and is measured (section 4).

**Exact public fields that must answer each M1 identification question.**

| M1 question | Command | Exact field path | Documented shape? |
|---|---|---|---|
| repository | C2/C3/C4 envelope | `repo` | **yes** — `schemas/public/command-result-v1.schema.json` (required, `string\|null`) |
| repository | C2 | `data.type=codefire_doctor_report`, `data.checked.{objects,branches,opened,active_files}`, `data.mode`, `data.skipped_checks` | **yes** — `automation_interface.md` doctor block |
| branch (set) | C3 | `data` branch entries (name / state / head) | **NO** — no documented field list, no schema |
| head | C3 / C4 | branch head, and `data.show.commit` | **NO** for both |
| base | C5 / C4 | sealed parent commit ID(s) of the head | **NO** |
| claim/proof kind | C4 | certificate `result`, assessment `subject`, `profile=local_verification_v1`, `phase=pre_commit` | **NO** in JSON; only prose in `04_cli_spec.md` §4.13 and `runbook.md:127` |
| missing authority | C4 | commit signature presence | **NO** |
| missing authority | C2 | `legacy_branch_head_semantic_profile` warning vs current `sealed-snapshot-v1` head | partly — `runbook.md` names the warning; `doctor` issue block is documented |
| missing authority | C1 | `data.support.runtimes[]` (`independent-python-stdlib-v1`: `read=true`, `write=false`, `role=read-only-conformance`), `data.support.unknown_version=deny`, `unknown_critical_extension=deny`, `unknown_noncritical_extension=preserve`, `semantic_validation_profile=sealed-snapshot-v1`, `canonical_json_profile=serde-json-compact-v1`, `public_object_id.{reader_digest_hex_lengths=[12,24],writer_digest_hex_length=24}` | **yes** — `schemas/public/capabilities-v1.schema.json` |

Independently observed at this HEAD from C1: `data.support.formats` has 12 entries
(`object_record`, `commit_payload`, `commit_root_schema`, `remote_project_descriptor`,
`remote_branch`, `remote_merge_request`, `remote_publication_transaction`,
`remote_idempotency_receipt`, `command_result`, `capabilities`, `remote_doctor`,
`decision_work_profile`); `data.support.schemas` lists exactly the five checked-in
`schemas/public/*` IDs; and every one of `branch`, `show`, `doctor`, `status`, `verify`, `context`,
`list`, `storage`, `explain`, `capabilities` reports `output_schema=codefire.command_result.v1`.

**This is the first missing contract** (see `block-m1` B1): `output_schema` is the *envelope* ID for
every command, `command-result-v1.schema.json` declares `data` as an unconstrained
`{"type":"object"}`, and the two commands that carry branch/head/base and claim/proof kind — `show`
and `branch list` — have no documented `data` field list and no checked-in schema. `show --json` has
exactly one line of public documentation at `docs/04_cli_spec.md:626` ("sealed commit summaryを返す"),
and `branch list --json` has none at all (`automation_interface.md:221` documents only that
`--metrics` adds `data.metrics`).

### 3. Q3 — preservation manifest

`$FIX` is the only admitted target. The canonical CodeFire store
`/workspace/project/subprojects/.codefire` and its open dogfood worktree
`/workspace/project/subprojects/algorithm-evolution-agent-lab` are non-target and must be
byte-identical before and after. **No CodeFire command may be issued against a canonical path** —
not even a read-only one. This is stricter than the remote M1 preservation script on purpose: it
drops the `status --path $DOG --json` check, because that acquires the shared repository lock and
republishes `.codefire/locks/repo.lock.owner.json`, and it is not needed when the store digest is
taken directly.

Recompute-and-compare, before the copy, after the copy, after every command, and at audit time:

```bash
C=/workspace/project/subprojects/.codefire
DOG=/workspace/project/subprojects/algorithm-evolution-agent-lab
git -C /workspace/project rev-parse HEAD
git -C /workspace/project status --porcelain | wc -l
sha256sum "$C/repo.json"
sha256sum "$C/branches/main.json"
find "$C/branches" -type f | sort | xargs sha256sum | sha256sum
sha256sum "$C/opened/main.json"
sha256sum "$C/active/open_4c320ed42560/state.json"
sha256sum "$DOG/.codefire-open"
find "$C/objects" -type f | wc -l
du -sb "$C/objects" | cut -f1
find "$C" -type f ! -name '*.lock.owner.json' | sort | xargs sha256sum | sha256sum
find "$C" -type f ! -name '*.lock.owner.json' | wc -l
find /workspace/project -maxdepth 6 -name .codefire-server 2>/dev/null | wc -l
```

Expected values, independently recomputed by this packet at HEAD `f91c8ca` (the first eleven are
unchanged from the values `audit-latest.md` recorded for the remote milestone; the last three are new
here):

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
```

Additional preservation obligations:

```text
- $ROOT lives under /tmp (8.2G free at contract time; the copy needs ~115MB). Not under /workspace/project.
- The prior milestone's evidence roots /tmp/m1-remote-happy-path{,-a2.zvrbzn,-a3.VbZTZ8} are read-only history and must not be written.
- $FIX/.codefire/locks/ may gain a copy-local repo.lock pair from read commands; that is fixture-local and expected.
- Fixture digest: take find "$FIX" -type f ! -name '*.lock.owner.json' | sort | xargs sha256sum | sha256sum before C1 and after C6; a read-only path must not change it.
- Forbidden-token scan over every recorded *.argv: init, import, open, close, discard, clone, scan, fire, extinguish, verify, commit, merge, upload, request-merge, request-review, request-apply, gc, serve, evidence, link, patch, migrate, --idempotency-key, /workspace/project.
```

### 4. Q4 — measurements obtainable without telemetry or private clarification

All measurements come from local run artefacts the harness writes itself, exactly as the remote
milestone did (`NN-name.argv` / `.json` / `.stderr` / `.meta` with start, end, duration, exit, pwd).
There is no network, no product telemetry, no instrumentation added to CodeFire, and no interview.

| Metric | Definition | Source |
|---|---|---|
| `setup_time_s` | `$ROOT` handover → first C1–C6 invocation exiting 0 | `.meta` timestamps |
| `ttfur_s` | handover → first command whose JSON answers ≥1 rubric item | `.meta` + answer sheet citation |
| `ttac_s` | handover → submitted answer sheet | `.meta` |
| `commands_issued` | count of runner invocations | `*.argv` |
| `off_contract_invocations` | invocations whose argv is not in C1–C6 | `*.argv` |
| `wrong_action_count` | invocations of any mutating command, or any argv naming a path outside `$ROOT` | forbidden-token scan |
| `misunderstanding_count` | rubric items answered incorrectly | answer sheet vs sealed key |
| `unanswerable_count` | rubric items C marks "not answerable from the docs pack", with the field looked for | answer sheet |
| `undocumented_field_reliance` | distinct JSON field paths cited in correct answers that are absent from the docs pack and from `schemas/public/**` | answer sheet vs docs pack |
| `docs_pack_files_read` | files C self-reports reading | answer sheet |

`maintenance_cost` is an **estimate, not a measurement**, and must be labelled that way: this
experiment changes no code, so the honest proxy is `undocumented_field_reliance` (each undocumented
field a consumer depends on is a future breakage the project has not promised not to cause) plus
`docs_pack_files_read`. Missing data is never scored as success.

Privacy/retention: `$ROOT` contains only the maintainer's own already-committed repository content and
public docs; no third-party data, no secrets, no network egress. `$ROOT` is retained on local disk as
node-4 evidence and is not published.

### 5. Comprehension rubric and answer key

Seven items, scored 0/1, answered from public output only, each answer required to cite the exact
command and JSON field path it came from. The key is derived from the same C1–C6 outputs by the node-4
run owner **before** `$ROOT` is handed to C, and its `sha256` is recorded in the run evidence; node 1
deliberately does not pre-bake values, so no answer can drift between contract and run.

```text
Q1 repository      absolute repository root, plus doctor checked object/branch/opened counts
Q2 branch set      every branch name present, including the URL-encoded feature/* forms
Q3 head            sealed head commit ID of branch main
Q4 base            immediate sealed parent commit ID of that head
Q5 claim kind      what CodeFire claims about that head: certificate result, the assessment
                   profile/phase, and whether the claim is embedded/local, signed/portable, or
                   legacy/unproven  (CFR-171 required-fix bullet 3)
Q6 missing auth.   name two absent authorities from the list the output actually supports, e.g.
                   commit signature absent; no open directory so no live status/verify/context;
                   independent-python-stdlib-v1 is read-only (write=false); unknown_version=deny
Q7 next action     one next_actions entry from any of C1-C6, and whether it is ready=true
```

Q5/Q6 carry the milestone. Per prior read-only evidence (`audit-latest.md`), the fixture is expected
to contain **both** a current-`sealed-snapshot-v1` head and legacy-unversioned heads, so C2 should
emit `legacy_branch_head_semantic_profile` warnings for the feature branches while `main` reports a
current profile. That mixed state is re-derived at node-4 setup, not assumed here; if it turns out
`main` is also legacy, Q5/Q6 keep their wording and the key records the actual profiles.

### 6. Q5 — precommitted adopt / narrow / stop thresholds

Frozen before any experiment mutation. Evaluated once, in this order; the first matching row wins.

| Order | Decision | Condition (all clauses) |
|---|---|---|
| 1 | **stop (terminal)** | preservation `DRIFT` at any checkpoint, OR `wrong_action_count >= 1`, OR C required private clarification, OR C read outside `$ROOT` |
| 2 | **stop (terminal)** | rubric score `<= 3/7`, OR `ttac_s > 7200` (120 min) |
| 3 | **narrow** | `Q5` or `Q6` incorrect/unanswerable while score `>= 4/7`, OR `undocumented_field_reliance >= 2`, OR `3600 < ttac_s <= 7200` |
| 4 | **adopt** | score `>= 6/7` with **both** `Q5` and `Q6` correct, AND `ttfur_s <= 900`, AND `ttac_s <= 3600`, AND `off_contract_invocations == 0`, AND `wrong_action_count == 0`, AND `undocumented_field_reliance <= 1` |
| 5 | **narrow** | anything else |

Meaning of each decision, precommitted:

```text
adopt   the read-only sealed-inspect surface is a real adoption wedge. Record it, and admit ONLY B1
        (document + schema-bind the show/branch-list claim fields). Do not promote a new command,
        adapter, Git bridge, or telemetry. Do not widen to the full CFR-171 cross-surface descriptor.
narrow  admit ONLY B1, re-run this identical experiment ONCE with a fresh consumer, then decide
        adopt/stop on the same table. No second consumer, no new command, no broadened rubric.
stop    record a terminal stop decision in CFR-172, keep CFR-171 and CFR-173 parked, promote nothing
        to core, and do not retry. Falsification is the result.
```

**Falsifier**, stated plainly: the M1 hypothesis is falsified if a source-blind consumer with public
docs and read-only commands cannot correctly state the head, its base, and the claim/proof kind of a
sealed CodeFire branch — i.e. if `Q3 AND Q4 AND Q5` are not all correct.

### 7. First missing contract — `block-m1`

**B1 (`block-m1`) — `show --json` and `branch list --json` `data` payloads are undocumented and
unschema'd, and they are the only public carriers of head, base, claim/proof kind, and signature
authority on the selected path.**

Evidence, public only, at HEAD `f91c8ca`:

```text
docs/04_cli_spec.md:626        "show --jsonは codefire.command_result.v1 envelopeでsealed commit summaryを返す"  (no field list)
docs/automation_interface.md   documents data shapes for status, scan, verify, doctor, storage report, review-pack,
                               patch export, evidence add, link batch, commit, fire, explain, migrate, context,
                               operation plan -- and NOT for show or branch list
docs/automation_interface.md:221  only mentions branch list to say --metrics adds data.metrics
schemas/public/               5 schemas; none describes any command's data payload
command-result-v1.schema.json  "data": {"type": "object"}   (unconstrained)
capabilities --json           output_schema == codefire.command_result.v1 for every command, so it cannot
                              distinguish per-command data contracts
```

This is CFR-171's gap reproduced from the public surface alone, and it is the smallest admissible
slice of it: **document and schema-bind the sealed claim/proof fields these two commands already
emit** (commit ID, parents, certificate result, assessment subject/profile/phase, signature presence,
branch name/state/head, semantic profile class). It is explicitly *not* the full CFR-171 cross-surface
descriptor, not a new command, and not a store validator change.

The counter-argument, recorded for node 3/node 6 to adjudicate rather than buried: reading a command's
own JSON output is not "source code or private clarification", so C may well answer Q3–Q6 correctly
from undocumented fields, and M1 exit bullet 3 could then be read as satisfied. This packet still
classifies B1 `block-m1` because exit bullet 3 asks for a *contract-truthful* identification, and an
answer that depends on fields the project has never promised is not one — it is a guess that happens
to be right today. `undocumented_field_reliance` in section 4 is the measurement that decides this
empirically, and threshold row 3/4 in section 6 is where it lands. **This packet does not authorize
implementing B1**; node 3 owns admission.

### 8. Findings

`block-m1` (1):

1. **B1** — `show --json` / `branch list --json` `data` payloads undocumented and unschema'd; first
   missing contract; smallest CFR-171 slice. Section 7.

`park` (5) — real, not required before node 3, not to be worked now:

2. **P1** — Repo-root `./codefire` (Python v0.2 reference/fallback) accepts `--json` on **no** command
   (`./codefire status --json` → `error: unrecognized arguments: --json`) and has no `context`,
   `capabilities`, `explain`, `import`, `storage`, `migrate`, `patch`, `evidence`, or `link`. The
   entire `docs/automation_interface.md` contract is Rust-only. `04_cli_spec.md:61` documents that
   `install.sh` installs Rust as default `codefire` and Python as `codefire-py`, so this is labelled,
   not silently broken — but a consumer who lands on the Python CLI gets none of the automation
   interface. Park to M2 `MS-release-operational-trust` (release identity / published artefacts).
3. **P2** — There is no consumer-obtainable runner acquisition path independent of a Rust toolchain.
   `README.md:41-70` offers `cargo build --workspace` or `./install.sh --binary ./target/release/codefire-rs`;
   the second still presupposes someone built it, and no prebuilt artefact is published. This
   environment has neither `cargo` nor `rustc`. Consequence, stated as an explicit scope limit rather
   than hidden: **M1 hands the audited binary to the consumer and therefore does not prove
   toolchain-free adoption.** Park to M2.
4. **P3** — Doc/help divergence on `context --branch`: `docs/04_cli_spec.md` §4.12 and
   `docs/automation_interface.md:171` both document `codefire context --branch --json` (bare flag),
   while `codefire context --help` documents `--branch <branch>` (value-taking). Not reproduced as a
   parse failure — the observed failure of `context --branch --json` outside a repository is
   `kind=not_open`, `exit_code=2`, which is repository discovery, not argv parsing, so the divergence
   is a documentation-consistency finding only. Off the selected path (`context` needs an open
   directory). Park.
5. **P4** — No command's `data` payload has a checked-in schema anywhere in `schemas/public/`;
   `capabilities` reports the envelope ID as `output_schema` for all ten JSON commands inspected. This
   is the general form of B1 and is deliberately *not* admitted; B1 is the two-command slice the
   selected consumer actually needs. Park.
6. **P5** — `request-list` omitting `applied_head`, inherited `park` from the remote milestone
   (`milestones.md`, `audit-latest.md` finding 1). Node 1 did **not** independently reproduce it as
   the smallest blocker for adoption; it is not on the selected path (no remote, no `request-*`).
   Stays parked.

`drop` (3) — alternatives considered and rejected as the subject; not filed as work:

7. **D1** — Conformance-fixture-only consumer (`schemas/conformance/manifest-v1.json`, six vectors,
   `independent-python-stdlib-v1`). Strictly smaller and needs no CLI, but it tests canonical-JSON /
   remote-branch / object-record accept-deny boundaries, **not** "repository, branch, base/head,
   claim/proof kind, missing authority", so it cannot satisfy M1 exit bullet 3. Available as a
   zero-cost optional pre-check at node 4; not the subject.
8. **D2** — Reusing `/tmp/m1-remote-happy-path-a3.VbZTZ8` as the subject. Dropped: it is the closed
   milestone's preserved evidence and must stay unmodified, and a fresh copy costs ~115MB.
9. **D3** — Freshly `import`ed toy repository as a true "second repository". Dropped for this slot: it
   requires `init`/`import`/`scan`/`commit` mutators before any certificate exists to reason about,
   and a three-file repository makes Q4/Q5 degenerate. Reconsider only if inspect adopts.

### 9. Issue mapping

| Issue | Node-1 disposition | Admitted slice |
|---|---|---|
| CFR-171 | **reproduced** as B1 | document + schema-bind the sealed claim/proof and branch-head fields `show --json` / `branch list --json` already emit. Not the full cross-surface descriptor, not a store-validator change. |
| CFR-172 | **primary owner** of node 4 | the read-only sealed-inspect experiment in sections 1–2, with the section 3 preservation boundary and the section 6 terminal decision. No thin adapter, no new surface, no Git bridge. |
| CFR-173 | **bounded slice only** | the ten metrics in section 4, the seven-item rubric in section 5, the thresholds in section 6. No cohort replay, no Fire-actionability protocol, no metric calculator, no telemetry. |
| CFR-168, CFB-136, CFB-133, CFR-169, CFR-170 | unchanged | backlog parked; not reproduced by this contract. |

### 10. Validation run by this packet

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Both pass; results are recorded with the commit. No product test, no `cargo` command (unavailable),
no installer, and no CodeFire mutator was run. The only CodeFire invocations were
`./codefire --help` / subcommand `--help` (Python, repo root), and against the admitted Rust runner
`--help`, `capabilities --json`, and one `context --branch --json` outside any repository — all
read-only, none touching `/workspace/project`. Canonical preservation was re-verified by direct
digest (section 3) and is `OK`.

### 11. Next legal action

Node 1 is complete. **Do not implement B1 and do not create `$ROOT`.** Node 3 (orchestrator fan-in)
is the next legal step, and it needs node 2 (`milestone-adoption-wedge-experiment.md`, Grok) first;
nodes 1 and 2 are independent, so node 2 may still be running. At fan-in, node 3 should:

1. admit or reject B1 as the CFR-171 slice, on this packet's evidence plus node 2's owner mapping;
2. admit sections 1–6 as the node-4 run packet, byte-for-byte, or reject them — the subject, argv,
   preservation manifest, rubric, metrics, and thresholds are frozen here and must not be edited
   after `$ROOT` exists;
3. name the node-4 persistent context owner and the node-6 fresh other-family auditor;
4. record in the board that M1 does not prove toolchain-free adoption (P2), so no later slot inherits
   that claim.
