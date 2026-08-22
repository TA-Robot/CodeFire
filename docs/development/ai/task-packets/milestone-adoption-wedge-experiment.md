# AI Task Packet

Status: completed

Packet ID: MS-adoption-wedge-experiment

Schema version: human-v1

Risk score: 5

Reasoning tier: high

## Objective

Classify CFR-171, CFR-172, and CFR-173 against the active adoption-wedge exit, map the current
production/public owners, and select at most one bounded experiment plus only its smallest unavoidable
product blocker. Freeze the falsifier and terminal decision before any implementation or experiment.

## Scope

- active board and the three issue detail files;
- directly relevant public docs, Rust ownership/source docs, command routes, schemas, and existing tests;
- historical adoption/claim/metrics evidence read-only;
- this packet's Result only.

This lane may read source after fixing the public requirement, but it may not implement, run an import,
mutate a repository, or begin a pilot.

## Questions

1. Which existing inspect/import surfaces already satisfy the second-consumer task without a new core
   feature, and which facts remain genuinely unavailable?
2. Does CFR-171 block the smallest consumer, or can existing certificate/profile/JSON fields truthfully
   bound the claim while the full descriptor stays parked?
3. What is the smallest CFR-172 experiment topology and exact no-mutation proof?
4. Which CFR-173 measures and privacy/retention rules are necessary for this one experiment, and which
   broad pilot metrics remain parked?
5. What focused test or read-only command would falsify each proposed admission, and where are the
   parse/validate/plan/apply/render/write owners if a blocker is reproduced?

## Validation

Set Status `completed` or `blocked`, classify every finding `block-m1` / `park` / `drop`, and run:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_source_docs.py
git diff --check
```

## Non-Goals

- implementing the full CFR-171 descriptor or changing sealed object formats;
- adding telemetry, hosted services, Git bridging, agent orchestration, or broad UX surfaces;
- running the adoption experiment, import, dogfood, release, product-value, or ecosystem work;
- CFR-168/CFB-136 and other backlog roots;
- editing the board, issues, source, tests, schemas, or any packet except this one.

## Handoff

Return exact fixed HEAD, issue-by-issue verdict, existing owner/source/test map, one recommended bounded
experiment, falsifier, preservation and measurement contract, exact candidate write/test set if and
only if a blocker is reproduced, stall/scope-creep risk, and next legal action. Commit only this packet.

## Result

Status **completed**. Classification at exact HEAD
`f91c8ca429dff5f93b9d88c35890a20f72435e27`: **no `block-m1` product write**.
**CFR-171 `park`**, **CFR-172 inspect-experiment admitted as the node-4 run (product residuals
`park`)**, **CFR-173 measurement/privacy/decision slice admitted as protocol only (broad pilot
`park`)**. **CFR-168 `park`**, **CFB-136 `park`**. Later slots stay parked. No import, no repository
mutation, no experiment execution.

Run by: Grok 4.6 individual agent, 2026-08-22, worktree
`ms-adoption-experiment-20260822`. Public requirement was fixed from `--help`,
`capabilities --json`, `schemas/public/`, `docs/04_cli_spec.md`,
`docs/automation_interface.md`, `docs/01_product_definition.md`,
`docs/05_consistency_model.md`, `docs/08_object_store.md`, `docs/runbook.md`,
and `docs/development/strategy/product-strategy.md` before any crate/test mapping.

Active milestone: `MS-adoption-wedge`.
M1 exit: one bounded inspect-or-import path by a second repository or clean-room consumer, with
preservation hashes, comprehension of repository/branch/base/head/claim-proof/missing-authority,
measured setup/TT FUR/misunderstanding/wrong-action/maintenance, adopt/narrow/stop, then independent
audit. This packet is node 2, parallel to node 1's public-consumer contract.
Write set: this packet only.

### 0. Runner and public-requirement freeze

| Item | Value |
|---|---|
| Exact HEAD | `f91c8ca429dff5f93b9d88c35890a20f72435e27` (`docs: close remote M1 and shift to adoption`) |
| Public-help runner | `/workspace/worktrees/ms-remote-happy-path-budget-20260821/target/release/codefire` |
| Runner sha256 | `70301c874d7220a71be988562df0e5188ede6a85c65c63dcbd3dcdee09f4706c` |
| Source identity | `git diff --quiet f91c8ca 552b17b -- Cargo.toml Cargo.lock crates` → identical; last crate-touching ancestor does not change this HEAD's crates |
| `codefire --version` | `codefire foundation 1` |
| Python fallback | present as reference; **not** the experiment runner (Rust is default CLI; envelope `--json` is the M1 evidence surface) |

Public inspect/import argv actually documented by default-CLI help:

```text
codefire import [path] [--branch <name>] [--dry-run] [--json] [--idempotency-key <key>]
codefire doctor [path] [--quick|--full] [--json]
codefire branch list [path|--path <repo-or-open>] [--json] [--metrics]
codefire list [path|<remote-project-url>] [--json] [--limit N] [--cursor CURSOR]
codefire show <branch-or-commit> [--json]
codefire status [path|--path <path>] [--json] [--metrics]
codefire review-pack <source> [--base <base>] [--output <path>] [--algorithm ...] [--no-rename-detection] [--json]
codefire explain (fire <id>|atom <id>|verify-failure|storage-warning) [--path <open-dir>] [--json]
codefire context (--changed|--atom <id>|--fire <id>|--branch <branch>) [--path <open-dir>] [--json]
codefire capabilities [--json]
codefire verify [path|--path <path>] [--details] [--blocking-only] [--json] [--metrics] [--wait-lock] [--lock-timeout <duration>]
```

`capabilities --json` (`codefire.capabilities.v1` inside `codefire.command_result.v1`): every
inspect command above except `import`/`verify` advertises `supports_json=true` and
`output_schema=codefire.command_result.v1`. `import` and `verify` also advertise JSON, but they are
**not** inspect: live `import` adopts a directory; live `verify` persists
`verification.json`/`state.json` through state-transaction v2 (`docs/04_cli_spec.md` §4.11).
`explain verify-failure` is the documented persist=false verify. `import --dry-run --json` is the
documented no-write **native-adoption plan**, not a labeled snapshot/advisory claim.
`show` and `review-pack` document **no `--path`**: working directory is the isolation boundary.
Checked public schemas are envelope/capabilities/page/remote-doctor only; there is no checked
public schema for certificate, verify `data`, show `data`, or a claim descriptor.

Public facts already available without a new core feature:

| M1 question | Existing public field |
|---|---|
| repository | envelope `repo`; assessment `subject.repository_id` when present |
| branch | `branch list` / `list`; `status.data.branch`; live subject `branch` |
| head | branch-list/list head; `show.data.show.commit` |
| base | `status.data.base`; review-pack `subject` / `base.commit`; live subject `base_commit`; closed-branch inspect uses head plus `show.data.show.parents` |
| claim/proof kind | `capabilities.support.semantic_validation_profile=sealed-snapshot-v1`; certificate `profile=local_verification_v1` + `phase`; assessment `kind` + `trust_kind` (`sealed_validated` vs `local_unisolated`); `certificate.result` / `verification.result` (`passed`/`failed`; `consistent` is the documented legacy alias of `passed`) |
| missing authority | `doctor` diagnostics (`severity`/`blocking`/`kind`); envelope `diagnostics` + `next_actions`; verify/explain blocker classes and exit 10–15/21/22 |

Genuinely unavailable as one lossless public descriptor (not required to *start* inspect):

- a versioned `claim_type` field and explicit non-claims projected identically from certificate,
  verify JSON, review-pack, show, and explain;
- `show --json` projecting the full certificate (it currently projects `certificate` as the
  **result string only**; review-pack `base`/`source` keep the certificate object);
- a checked public JSON Schema for those payloads;
- a Git-snapshot or plain-directory advisory profile whose claim type is not native import.

That gap is the CFR-171 residual. It is **not** a reproduced product blocker: M1 allows retaining
consumer failure as the terminal result, and review-pack + doctor + branch list + show already
expose enough fields to *attempt* the comprehension task.

### 1. Existing inspect/import surfaces (Question 1)

Inspect path already implemented and smaller than import:

- `doctor --quick --json`, `branch list --json`, `list --json`, `show --json`,
  `review-pack --json` with `--output` **outside** the subject, `capabilities --json`.
- Optional open-directory extras, **not** selected: `status --json`, `context --json`,
  `explain … --json` (need an open instance).
- `schemas/conformance/minimal_consumer.py` is a stdlib object-record/remote-branch reader, **not**
  the M1 consumer: it cannot answer repository/branch/base/head/claim/missing-authority.

Import path exists (`import_workflow::import_existing_project`) with dry-run proven not to create
`.codefire` (`tests.rs::import_existing_project_dry_run_and_apply_adopt_non_empty_directory`).
RVC-002 already records that this is native adoption, not a distinct read-only snapshot claim.
Selecting live import, or treating dry-run as inspect, would enlarge M1 and mix claim types.

Live `verify` / `scan` / `open` / `commit` are mutating and are **not** the inspect experiment.

### 2. CFR-171 (Question 2) — `park`

Current production certificate (canonical catalog + `docs/05_consistency_model.md` §5.6) already
binds `result`, `subject`, `profile`, `phase`, blocker counts, `effective_policy_digest`, and
`change_coverage_digest`. Store
`validate_certificate_matches_verification` / `validate_assessment_subject` fail-closes mismatch,
one-sided subject, and unknown critical versions. CLI `assessment_subject` shares live vs
`sealed_commit_pair` identities across context, explain, verify, review-pack, and the later
certificate (`tests/subject_trust.rs::sealed_commit_certificate_preserves_the_verified_subject_and_phase`).

The smallest consumer does **not** need the full cross-surface descriptor, tamper table, or
conformance fixture before the experiment. Existing fields truthfully bound *what was evaluated
under which profile/phase/trust_kind and whether it passed*. Full CFR-171 remains parked unless a
later isolated run reproduces that the selected public argv cannot answer claim/proof kind *and*
that failure is judged a product defect rather than the allowed terminal result.

`schemas/object_commit.example.json` still shows `certificate.result=consistent` without
subject/policy/coverage. That example drift is **`park`** (docs residual), not `block-m1`.

### 3. Smallest CFR-172 topology and no-mutation proof (Question 3)

**Selected experiment (at most one):** source-blind inspect of an isolated copy of an already-sealed
CodeFire object store. No import. No Git. No open/scan/verify write. No agent platform.

```text
isolated root     $ROOT     = mktemp -d, outside /workspace/project and this worktree's canonical tree
subject           $FIX      = $ROOT/subject          (copy of .codefire only; no open directory)
evidence          $EVIDENCE = $ROOT/evidence         (hashes, envelopes, rubric; not product telemetry)
non-targets       this worktree HEAD tree, /workspace/project if distinct, and the copy-source repo
consumer          clean-room operator: public help + numbered product docs + schemas/public only
runner            source-identical Rust CLI as admitted at node 3 (this probe used 70301c87…)
```

Copy source is a **second already-sealed repository** (dogfood
`subprojects/algorithm-evolution-agent-lab/.codefire` or another sealed store). It is a harness copy,
not a write to Algorithm or `/workspace/project`. Node 1 independently names the exact subject; this
lane requires inspect-only and refuses a live import target.

Exact public sequence (`cwd=$FIX` for `show` / `review-pack`; path args for the others):

```bash
"$CF" capabilities --json
"$CF" doctor "$FIX" --quick --json
"$CF" branch list --path "$FIX" --json
"$CF" list "$FIX" --json
"$CF" show <one-closed-branch-or-head> --json
"$CF" review-pack <that-branch> --output "$EVIDENCE/review-pack.json" --json
```

`<one-closed-branch-or-head>` is taken from the branch-list envelope, not from memory. `review-pack`
`--output` must be under `$EVIDENCE`, never under `$FIX`. Do not pass `--base` unless the chosen
commit is a documented root (spec requires `--base` only then).

**No-mutation proof (preservation contract):**

1. Before any CodeFire command: recursive sha256 manifest of `$FIX`, of the copy-source original,
   and of every non-target named above (path-sorted regular files; follow nothing).
2. Repeat the same manifests after the sequence.
3. `$FIX` and every non-target must be byte-identical. `$EVIDENCE` may grow.
4. Optional distinguisher: `doctor --quick --json` and `branch list --json` on the **original**
   copy-source before and after must match (read-only confirmation, node 5).
5. Protocol is invalid if any Git command is used as a bridge, if the consumer reads `crates/` or
   tests, if live `import`/`verify`/`scan`/`open` runs, or if network/telemetry leaves the isolated
   root.

Existing tests that already distinguish “inspect owners do not write” / “import dry-run does not
write” and must **not** be re-run from this packet:

- `crates/codefire-cli/src/tests.rs::import_existing_project_dry_run_and_apply_adopt_non_empty_directory`
- `crates/codefire-cli/src/tests.rs::doctor_quick_mode_skips_full_object_record_scan`
- `crates/codefire-cli/src/tests.rs::show_and_diff_local_branches`
- `crates/codefire-cli/src/tests.rs::review_pack_exports_file_atom_verification_and_next_actions`
- `crates/codefire-cli/src/tests.rs::explain_fire_atom_verify_and_storage_targets_return_actions`
- `crates/codefire-store/src/lib.rs::validate_sealed_commit_binds_certificate_to_the_verification_subject`

CFR-172 residuals **`park`**: Git snapshot/base-head adapter, canonical one-screen open-workspace
model, thin MCP/SARIF adapter, dual plain-repo + CodeFire user study, UX sugar.

### 4. CFR-173 measures, privacy, decision (Question 4)

Admit **only** this experiment's protocol. Do not add product metrics, telemetry, or a calculator.

Local evidence fields (JSON in `$EVIDENCE/decision.json`, no raw source, no secrets, no PII):

| Field | Definition |
|---|---|
| `setup_s` | wall time from isolated root creation through first successful `capabilities --json` |
| `ttfur_s` | wall time from first command to first envelope that names repository + branch + head |
| `misunderstanding_n` | wrong answers among the six comprehension questions |
| `wrong_action_n` | mutating commands attempted (`import` without dry-run, `scan`, live `verify`, `open`, `commit`, Git) |
| `maintenance` | extra product surface the consumer needed beyond the six argv above (`none` expected) |

Six comprehension questions, answered from captured JSON only:

1. What is the repository identity (`repo` and/or `subject.repository_id`)?
2. What branch was inspected?
3. What is the head commit ID?
4. What is the base (review-pack `base.commit` / parents / status `base` if an open dir were in
   scope — for this sealed-only topology: parent list plus declared pair base)?
5. What claim/proof kind was evaluated (`semantic_validation_profile`, certificate
   `profile`/`phase`, subject `kind`/`trust_kind`, `result`)? Explicitly: this is **not**
   semantic correctness, security, or compliance.
6. What authority is missing, if any (`doctor` blocking/repairable issues, empty vs present
   diagnostics)? “None visible on `--quick`” is a valid answer when `ok=true` and issues are empty.

Precommitted thresholds (frozen before node 4):

| Decision | All of |
|---|---|
| **adopt** | preservation holds; all six answers correct; `wrong_action_n=0`; `setup_s≤900`; `ttfur_s≤600`; `maintenance=none` |
| **narrow** | preservation holds; repo/branch/head correct; claim/proof or missing-authority incomplete **or** recovered only after review-pack (show result-string confused the consumer); `setup_s≤1800` and `ttfur_s≤1200`; `wrong_action_n≤1` and recovered via help/doctor without a write |
| **stop** | cannot identify repo **or** branch **or** head; treats `passed`/`consistent` as “the change is correct/secure”; requires import, Git, source, or private clarification; `setup_s>1800` or `ttfur_s>1200` with no useful envelope |

Missing data is **not** success. Failed/negative envelopes are retained in `$EVIDENCE`.

**Privacy / retention:** no product telemetry, no hosted service, no environment values, no file
bodies, no user identifiers. Evidence is local aggregates + command envelopes + hashes. Isolated
`$ROOT` is deleted after the independent M1 audit copies the decision record. Unlisted CFR-173
cohort metrics (checkpoint frequency, Fire actionability/lead time, support cost, multi-repo
pilots, metric-calculator productization) stay **`park`**. Telemetry in core is **`drop`** as M1
work.

### 5. Falsifiers, owners, and admission tests (Question 5)

Protocol falsifiers (experiment invalid; do not interpret as product stop):

- non-target or `$FIX` hash change after the inspect sequence;
- consumer reads implementation/tests or receives private clarification;
- Git used; live import/verify/scan/open; network exfil;
- `--output` of review-pack written inside `$FIX`.

Product falsifiers (use the adopt/narrow/stop table): comprehension failure, mutating wrong
action, claim-language confusion, cost overrun.

| Finding | Class | Why |
|---|---|---|
| CFR-171 full descriptor / lossless show / checked claim schema / tamper corpus | `park` | existing profile/subject/result/digests bound the inspect claim; full slice is not reproduced as a start blocker |
| CFR-171 use of current certificate/subject/trust_kind fields | not a write | sufficient to attempt M1 comprehension |
| CFR-172 selected sealed inspect experiment | admitted run, not `block-m1` | primary M1 input; no crate change |
| CFR-172 Git adapter, one-screen UX, import-as-first-signal, MCP/SARIF | `park` | larger than one inspect path; Git bridge is an M1 non-goal |
| CFR-173 this experiment's measures/privacy/stop | protocol only | no product metric surface |
| CFR-173 Fire/checkpoint/support/pilot/telemetry | `park` / telemetry `drop` | broad Stage 5 residual |
| CFR-168 / CFB-136 | `park` | remote-M1 did not reproduce; inspect path does not admit budget/cache writes |
| CFB-133 / CFR-169 / CFR-170 / request-list `applied_head` | `park` | later slots; node 1 may independently reproduce a public-contract miss, this lane did not |
| Live verify/scan/import as the inspect path | `drop` | mutating; contradicts preservation |
| Git bridge / agent platform / hosted telemetry | `drop` | M1 non-goals |
| Python fallback as experiment runner | `drop` | default CLI is Rust JSON |

**Existing owner map (inspect path only):**

| Command | Parse | Validate | Plan | Apply / write | Render | Focused evidence |
|---|---|---|---|---|---|---|
| `capabilities` | `command_registry` | registry/support | none | none | `capabilities_data_json` + envelope | `command_capabilities_are_shared_by_help_completion_and_metrics_parser` |
| `doctor --quick` | `doctor::parse_doctor_args` | layout/heads/opened via Store session | none | **none** (read-only) | `doctor::report` | `doctor_quick_mode_skips_full_object_record_scan` |
| `branch list` | `main` branch routing | `branch_refs` snapshots + sealed heads | none | stdout only | automation/metrics | `branch_list_reads_repo_from_open_marker_and_validates_heads` |
| `list` (local) | `main::parse_list_args` | sealed heads | none | none | list JSON | command-source-trace `list` |
| `show` | `main::parse_show_args` | `view` + `validate_sealed_commit` | none | none | `show_commitish_data` (`certificate` = result string) | `show_and_diff_local_branches` |
| `review-pack` | `main::parse_review_pack_args` | both sealed commits | none | optional `--output` file | `view::review_pack_with_options` + `sealed_pair_subject` | `review_pack_exports_file_atom_verification_and_next_actions` |
| `import --dry-run` | `import_workflow::parse_import_args` | preflight snapshot | operation plan | **skipped** | `import_result_data_json` | `import_existing_project_dry_run_and_apply_adopt_non_empty_directory` |
| certificate agreement | n/a | `codefire_store::validate_certificate_matches_verification` | n/a | sealed commit only | projected by show/review-pack | store certificate/subject tests |
| assessment subject | `assessment_subject::{live_open_subject,sealed_pair_subject}` | Store subject contract | n/a | commit certificate | context/explain/verify/review-pack | `tests/subject_trust.rs` |

Core: `Verification`, `CommittableDecision`, `ChangeCoverage`, effective-policy digest. Store:
sealed commit + certificate match. Docs owners: `docs/04_cli_spec.md`, `docs/automation_interface.md`,
`docs/05_consistency_model.md`, `docs/08_object_store.md`, canonical source-docs 4.

**Candidate write/test set if a blocker is reproduced:** **none**. No product blocker was
reproduced. Do not admit `view.rs` lossless-certificate show, sealed-format changes, or new tests
from this packet. If node 4 later reproduces that the selected argv cannot answer claim/proof kind
*and* fan-in treats that as a defect rather than stop/narrow, the smallest later candidate would be
`show_commitish_data` projecting the existing certificate object (parse/render only, no format
change), tests around `show_and_diff_local_branches`. That remains **unadmitted**.

### 6. Fan-in, stall risk, next action

Node 1 writes only `milestone-adoption-wedge-contract.md`. This packet writes only itself. Fan-in
(node 3) must:

- keep the experiment **inspect-only** unless node 1 independently proves inspect cannot name
  repository/branch/base/head from public argv — even then prefer `import --dry-run` on an isolated
  plain tree over live import;
- refuse Git, HTTP remote, live `verify`/`scan`, canonical Algorithm writes, and CFR-168/CFB-136;
- name the exact copy-source, isolated `$FIX`, runner digest, and preservation manifests;
- copy this packet's falsifier, measurement, privacy, and adopt/narrow/stop table into the admitted
  run packet without widening CFR-171/173.

Issue ledgers stay `investigating`. This packet does not change issue status, the board, or
canonical dogfood state.

Stall / scope-creep risks: implementing CFR-171 before the run; treating lossy `show.certificate`
as a pre-experiment `block-m1`; running import against dogfood or `/workspace/project`; using live
verify as “inspect”; admitting Fire-pilot metrics or telemetry; pulling CFR-168/CFB-136 because
doctor uses a Store session; waiting serially on node 1 before this classification (already
independent).

Validation actually run after this Result: `python3 docs/development/tools/check_project_docs.py`;
`python3 docs/development/tools/check_source_docs.py`; `git diff --check`. No cargo/python tests,
no `codefire import`, no product write.

Next legal action: wait for node 1 and orchestrator fan-in (node 3). Do not implement. Do not
start node 4 from this packet alone.
