# AI Task Packet

Status: completed

Packet ID: MS-product-value-proof-first-seal

Schema version: human-v1

Risk score: 7

Reasoning tier: high

## Objective

Derive the smallest isolated first-seal fixture and exact public command/help/JSON path that a fresh
source-blind participant can complete without private correction. Classify runner/source identity,
preservation, evidence capture, friction points, and the first missing contract. Do not run the study.

## Scope

- read-only inspection of current public install/runbook/help/JSON/next-action surfaces and focused tests;
- one disposable source project and CodeFire repository lifecycle ending in a first sealed commit;
- controller/participant/evidence separation and exact preservation/falsifier plan;
- this packet only.

## Required Classification

1. Select the exact source-bound runner or fresh-build rule. Retained adoption/release artifacts are
   historical inputs, not automatically valid participant runners.
2. Select a small realistic source project task with visible value and deterministic focused validation.
3. Derive the minimum public sequence from acquisition/handoff through init/import/open, edit,
   scan/Fire handling, verify, commit, show/status confirmation, using help/JSON/next_actions only.
4. Identify which steps mutate only the isolated target, which fields/argv are public contracts, and
   where a participant could confuse local embedded proof with portable authority.
5. Define separate controller/key/participant roots, transcript/evidence format, exact before/after
   hashes for every non-target repository/install/artifact, and no-private-correction enforcement.
6. Define setup/friction timing points, command/refusal/intervention counters, terminal success, and
   crash/wrong-action/claim-misreading falsifiers.
7. Classify the first missing contract and any CFR/CFB overlap as `block-m1`, `park`, or `drop`; propose
   the smallest exact write set only if a reproduced product blocker is unavoidable.

## Exact Write Set

- `docs/development/ai/task-packets/milestone-product-value-proof-first-seal.md` only.

## Validation

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

## Non-Goals

- building, installing, invoking CodeFire, mutating a fixture, or running a participant;
- product/source/test/public-doc edits, Git bridge, telemetry, network, hosted service, or agent platform;
- canonical Algorithm or user-install mutation, broad pilot, pricing, or M2–M5 work;
- inventing a fix before the public path is shown to require it.

## Handoff

Return exact runner/source rule, fixture/task, public argv/help/JSON fields, mutation boundary,
controller/participant topology, evidence/preservation/falsifier plan, timing/friction points, first
missing contract, `block-m1` / `park` / `drop`, any smallest fix write set, validation, and commit.
Commit only this packet.

## Result

Status **completed**. Classification at exact HEAD
`d9481644f5dfd47d8cdff507df3c4215fa542fa6`. **No `block-m1` product write.** No participant ran.
No CodeFire process was invoked. No fixture was materialized. No historical runner was admitted.

Run by: Grok 4.6 individual agent, 2026-08-22, worktree
`m1-product-first-seal-20260822`. Public requirement was frozen from `codefire --help` /
subcommand help source, `docs/04_cli_spec.md`, `docs/automation_interface.md`, `docs/runbook.md`,
`README.md`, `docs/02_core_concepts.md`, `docs/03_workflow_spec.md`, `docs/09_index_trace_policy.md`,
`schemas/public/`, and the focused tests named below. Crate/test mapping was used only after that
freeze, and only to name exact argv/fields/next_actions already advertised by those public surfaces.

Active milestone: `MS-product-value-proof`. This packet is node 2, parallel to node 1's segment/value
contract. Write set: this packet only.

### 0. Method freeze

| Item | Value |
|---|---|
| Exact HEAD | `d9481644f5dfd47d8cdff507df3c4215fa542fa6` (`docs: close release M1 and shift to product value`) |
| Rust input manifest | `3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a` over 101 files (`build_source_manifest` at this HEAD; no compile) |
| Crates vs `01a4de0` | `git diff --stat 01a4de0 HEAD -- Cargo.toml Cargo.lock crates .cargo rust-toolchain rust-toolchain.toml` empty |
| Historical release runner | `/workspace/worktrees/m1-release-rehearsal-20260822/target/release/codefire-rs` exists; SHA-256 recorded as `90cb2895aadc45b559a96207cdd5ae3b08485f9a7e5a07014db8cca8b5b90ce9`; provenance bound to `01a4de0`; **not** a participant runner |
| `codefire` / `cargo` on PATH | absent |
| Python fallback | `./codefire` is not the study runner (`--json` / `--path` / `next_actions` / `codefire.action_execution.v1` are the Rust public contract) |
| This node invoked CodeFire? | no |
| This node mutated a fixture? | no |

Public inspect/import/seal argv actually documented by default-CLI help:

```text
codefire init [path] [--force]
codefire import [path] [--branch <name>] [--dry-run] [--json] [--idempotency-key <key>]
codefire open <branch> <path> [--dry-run] [--json] [--idempotency-key <key>]
codefire status [path|--path <path>] [--json] [--metrics]
codefire scan [path|--path <path>] [--json] [--metrics] [--full] [--wait-lock] [--lock-timeout <duration>]
codefire extinguish <fire-id> [--path <open-dir>] --resolution <type> (--rationale <text>|--evidence <text>|--evidence-ref <id>) [--dry-run] [--json]
codefire verify [path|--path <path>] [--details] [--blocking-only] [--json] [--metrics] [--wait-lock] [--lock-timeout <duration>]
codefire commit [path|--path <open-dir>] -m <message> [--dry-run] [--json] [--idempotency-key <key>]
codefire show <branch-or-commit> [--json]
codefire doctor [path] [--quick|--full] [--json]
codefire capabilities [--json]
```

`init` advertises neither `--json` nor `next_actions`. `show` advertises no `--path`; working
directory is the isolation boundary. Successful live `commit --json` returns `next_actions: []`.

### 1. Source-bound runner / fresh-build rule

**Selected rule:** the participant runner is a Rust `codefire` binary whose
`codefire.rust_build_inputs.v1` manifest equals this admitted study HEAD (or the later node-3
admitted HEAD if only docs/board commits land) and whose provenance file binds **that same
revision**. Retained adoption/release/local-loop binaries are historical inputs, not automatically
valid.

Exact admission (node 3 / 5a controller only; this node did not build):

1. Clean worktree at the admitted HEAD. `build_source_manifest` must print
   `3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a 101` unless a later admitted
   crate-touching commit updates it; if it drifts, stop and rehash.
2. Fresh-build with
   `python3 docs/development/tools/build_source_bound_rust.py --output target/codefire-ai/rust-build-provenance.json`
   using the isolated Rust 1.97.0 toolchain from the release-trust cache. Do not use PATH `cargo`
   (currently absent). Do not use `install.sh` without `--binary`.
3. Provenance `revision` must equal the admitted HEAD. Binary path is
   `target/release/codefire-rs` (public display name `codefire`). Record binary SHA-256 from the
   provenance file.
4. Optional isolated install: `install.sh --prefix "$RUNNER_PREFIX" --binary "$ADMITTED_RS"` with
   `$RUNNER_PREFIX` inside the study root. Do not export `PATH`. Do not write `$HOME/.local` or
   `/usr/local`.
5. Hand the participant only the absolute `$CF` path.

**Rebind exception (not automatic):** node 3 may reuse the historical
`90cb2895…` bytes only after recording all of: rust-input manifest equality with the admitted HEAD
(already true vs `01a4de0` at this freeze), binary SHA-256 match to that file, and an explicit
statement that provenance still names `01a4de0` so the artifact remains internal-only. Absence of
that written rebind means **fresh-build**. Python `codefire`, B1, local-sealed-loop cache keys, and
any binary whose manifest does not match are **`drop` as runners**.

### 2. Isolated source fixture and task

**Selected fixture name:** `receipt-digest`. A disposable Python receipt-canonicalization library
with four Atoms and three Trace Links. It is not Algorithm dogfood, not `/workspace/project`, and
not this worktree.

**Visible value:** implement `canonical_receipt_digest(merchant, date, total_cents) -> str` that
returns the lowercase SHA-256 hex digest of the UTF-8 string `"{merchant}|{date}|{total_cents}"`.
Example: `ACME|2026-08-22|1999` →
`e3e3be6e7f17003100a2953c6dcb8a618e117f4cdcc2123b92f3db866b4731bd`. That digest is a real
reconciliation/audit primitive, not a test-only stub.

**Deterministic focused validation:** `python3 -m unittest discover -s tests` as the single
`codefire.policy.yaml` verification command (`id: unit-tests`, `cwd: "."`, `inherit_env: false`,
`timeout_ms: 30000`, `max_output_bytes: 65536`). No workspace cargo tests. No Algorithm suite.

**Exact files node 5a materializes under `$FIX` (no `.codefire` yet):**

| Path | Content rule |
|---|---|
| `README.md` | states the digest contract and that the library is incomplete until the function returns the digest |
| `docs/spec/receipt.md` | heading `## REQ-receipt-digest:` plus the canonical-string requirement |
| `docs/design/receipt.md` | heading `## DES-receipt-digest:` plus SHA-256 / UTF-8 / lowercase-hex design |
| `src/receipt.py` | `# cf-atom: CODE-receipt-digest` immediately before `def canonical_receipt_digest(...)` that raises `NotImplementedError` |
| `tests/test_receipt.py` | `# cf-atom: TEST-receipt-digest` immediately before a test that asserts the ACME vector above |
| `codefire.yaml` | `version: 1` with `docs/spec/**/*.md` requirement, `docs/design/**/*.md` design, `src/**/*.py` code, `tests/**/*.py` tests only |
| `codefire.links.yaml` | `REQ-receipt-digest refined_by DES-receipt-digest`; `DES-receipt-digest implemented_by CODE-receipt-digest`; `REQ-receipt-digest verified_by TEST-receipt-digest` |
| `codefire.policy.yaml` | standard `commit_policy` safety-floor trues; `require_trace_completeness: true`; `required_links` as in `docs/09_index_trace_policy.md` §9.10; one `unit-tests` verification command |

Do not add ADR/OpenAPI/SQL trees. Do not pre-create `.codefire`. Do not pre-extinguish. If
controller setup scan (node 5a only) reports a second Python-derived `CODE:` path Atom, leave it
unlinked unless `duplicate_atom_ids` appears; then keep the explicit `CODE-receipt-digest` marker
on the symbol block and remove the colliding ID. This node does not materialize those bytes.

**Participant task card (no command recipe, no Fire IDs, no resolution text, no this packet):**

> The handed directory is an incomplete receipt-digest library. Make
> `canonical_receipt_digest` return the documented lowercase SHA-256 hex digest, then produce one
> first sealed CodeFire commit of that change. Use only the handed `codefire` binary, its `--help`,
> `--json` envelopes, and `next_actions`. After a seal exists, use read-only commands to say what
> the seal proves and what it does not. Do not install, do not use Git, and do not ask the
> controller for commands.

### 3. Public first-seal sequence

**Selected start:** `import`, not `init`+`open`. Import adopts the non-empty fixture as repository
**and** open directory (`open_dir == repo_root`, initial empty sealed base, files become pending
worktree, internal scan, `initial_state=open-burning`). `init` creates a closed empty repo with no
JSON/next_actions; `init` inside `$FIX` writes `.codefire` and then blocks `import`; `open` of empty
`main` into a sibling materializes the empty commit and leaves the source files outside the open
directory. Those are classified wrong-actions, not the path.

Participant `$CF` is the admitted absolute binary. Preferred isolation: pass `--path $FIX` on every
command that advertises it; for `import` use the positional path; for `show` set `cwd=$FIX`.

Exact public sequence. JSON only after the help probes. Stop on first envelope that cannot be
followed without private correction; do not work around.

```text
# Acquisition (controller, not participant)
#   fresh-build or written rebind; optional isolated --prefix/--binary; freeze hashes (§6)
# Handoff: $CF, $DOCS, $FIX, task card. Never this Result, never crates/, never docs/development/

"$CF" --help
"$CF" capabilities --json
"$CF" import --help
"$CF" import "$FIX" --dry-run --json          # no write; plan.operations + next_actions
"$CF" import "$FIX" --json                    # MUTATES $FIX only
# edit: implement canonical_receipt_digest in $FIX/src/receipt.py only
"$CF" scan --path "$FIX" --json               # follow extinguish_fire execution, not import's commit
# for each scan next_action kind=extinguish_fire with execution.ready=false:
#   fill required_inputs resolution (extensible string; examples addressed/changed)
#   plus at least one of rationale/evidence/evidence_ref; exec argv without a shell
"$CF" extinguish <display_id> --resolution <filled> --rationale <filled> --path "$FIX" --json
# ready=true verify action from extinguish envelope:
"$CF" verify --path "$FIX" --json             # or verify --details --json as the action argv says
# if ok=false: follow that envelope (failed_checks → edit+rescan; open fires → extinguish)
# when execution.kind=commit and ready=false: caller-authored -m, then:
"$CF" commit --path "$FIX" -m <message> --json
# read-only confirmation (NOT emitted as post-commit next_actions; discover from help/task card):
"$CF" status --path "$FIX" --json
# cwd=$FIX because show has no --path:
"$CF" show main --json
"$CF" doctor "$FIX" --quick --json            # optional; must not repair
```

Do not run `init`, `open`, `close`, `clone`, `merge`, `upload`, `serve`, `request-*`, `migrate apply`,
Git, `codefire-py`, or network.

**How JSON steers the loop (public fields only):**

| Step | Envelope `command` | Follow |
|---|---|---|
| import dry-run/apply | `import` | `data.type=codefire_import_result`; `data.plan.next_actions` also copied to envelope `next_actions`: scan / verify / canned `commit -m "Initial import"` **without** `execution` (§9) |
| scan | `scan` | `data.open_fire_count`, `data.open_fires[].display_id`; `next_actions` `extinguish_fire` with `execution.schema=codefire.action_execution.v1`, `ready=false` |
| extinguish | `extinguish` | `next_actions` ready `verify` argv including `--path` and `--json` |
| verify | `verify` | `data.result` / `data.decision.outcome`; `data.subject.kind` / `trust_kind` / `immutable`; `data.profile=local_verification_v1`; `data.phase=post_execution`; commit action `ready=false` requiring `message` when committable with pending changes; else blocker actions |
| commit success | `commit` | `ok=true`, `data.applied=true`, `data.commit_id`; **`next_actions` empty** |
| status | `status` | `data.branch`, `data.state`, `data.base`, `data.open_fires`; open-clean has no next_actions |
| show | `show` | `data.type=codefire_show_result`; `data.show.commit`, `data.show.parents`, `data.show.certificate` (result **string**), `data.show.claim` |

Import already scans once. The public spec still names `scan --json` after import; the selected
path **re-scans after the edit** so Fire IDs bind the edited worktree. Do not treat import's canned
commit command as executable.

### 4. Mutation boundary, public contracts, local-vs-portable hazards

**Mutates only `$FIX` (the isolated target):** live `import`, the one source edit, live `scan`,
live `extinguish`, live `verify` (persists `verification.json` / `state.json`), live `commit`.
`import --dry-run` must not create `.codefire` (focused test
`import_existing_project_dry_run_and_apply_adopt_non_empty_directory`).

**Read-only:** `--help`, `capabilities --json`, `status --json`, `show --json`,
`doctor --quick --json`. `explain` / `context` / `review-pack` are optional extras; if used,
`review-pack --output` must land under `$EVIDENCE`, never `$FIX`.

**Must not mutate:** this worktree, `/workspace/project`, canonical Algorithm,
user install prefixes, historical runner trees, `$KEY` after freeze, `$DOCS`, `$CTRL`.

**Public argv/fields the participant may rely on:**

- envelope: `schema=codefire.command_result.v1`, `command`, `ok`, `exit_code`, `repo`, `data`,
  `diagnostics`, `next_actions` (`schemas/public/command-result-v1.schema.json`)
- `next_actions[]`: `kind`, `command`, `reason`, `target`; optional `execution` with
  `schema=codefire.action_execution.v1`, `ready`, `argv` or `argv_template`, `required_inputs`,
  `required_input_groups`
- `--resolution` is an extensible string; parser default `addressed`; not a closed enum
- `show.certificate` is the result string; `consistent` is a legacy alias of `passed` on
  **stored certificates**, not a second proof kind
- `capabilities --json` `output_schema=codefire.command_result.v1`; `show` advertises
  `data_schema=codefire.show_result.v1`

**Claim-misreading hazards (measure; do not pre-fix):**

1. `verify` / first-seal certificate `subject.kind=live_open`, `trust_kind=local_unisolated`,
   `immutable=false`, `profile=local_verification_v1`, `phase=pre_commit` on the sealed commit.
   A participant may read the seal as portable or still-live. Public docs: this is local policy
   consistency on an unisolated live subject captured at commit time, not semantic correctness,
   security, compliance, or remote/portable authority.
2. `show.certificate` is a string; the certificate **object** is not in `show` JSON.
   `review-pack` keeps the object. Treating `passed`/`consistent` as “the change is correct” is a
   product-strategy forbidden claim.
3. `status.data.state=open-clean` is persisted branch/open state, not proof that `opened/` is
   empty, not portable authority, and not “no files exist”. Copied repos can still say
   `open-clean` (CFR-171/adoption parked finding).
4. Import `next_actions` commit command looks ready and uses a canned message; executing it before
   Fire handling is a wrong-action, not a seal.
5. Top-level help lists `init` first. `init $FIX` poisons the import target.
6. Empty post-commit `next_actions` plus `show` lacking `--path` can look like “the CLI is done /
   has no confirmation surface”.

### 5. Controller / key / participant topology

One filesystem, mode 700 root. Names are exact.

```text
$ROOT       = mktemp -d /tmp/codefire-m1-pvp.XXXXXX
$CTRL       = $ROOT/controller     controller-only: this packet, answer-key, hash worksheets
$KEY        = $ROOT/key            frozen SHA256SUMS + runner provenance copy; chmod a-w after freeze
$RUNNER     = $ROOT/runner         admitted binary and optional isolated prefix
$DOCS       = $ROOT/docs           public docs only (below)
$FIX        = $ROOT/fixture/receipt-digest
$PART       = $ROOT/participant    participant cwd / transcript; no product source
$EVIDENCE   = $ROOT/evidence       envelopes, JSONL transcript, timings, counters
$CF         = admitted absolute binary under $RUNNER
```

`$DOCS` may contain only: `README.md`, numbered product chapters `docs/00_*`–`docs/11_*` and
`docs/13_*`, `docs/automation_interface.md`, `docs/runbook.md`, `schemas/public/*`. It must not
contain `docs/development/**`, `crates/**`, `tests/**`, this packet, or the segment packet.

Roles:

| Root | Who | Sees | Writes |
|---|---|---|---|
| `$CTRL` / `$KEY` | persistent controller | everything | hashes, admission notes; never during the participant session |
| `$FIX` | participant | fixture + after-import `.codefire` | the first-seal path only |
| `$PART` / `$EVIDENCE` | participant + controller collector | envelopes they produce | transcript/envelopes |
| `$DOCS` / `$CF` | participant | public docs + binary | none |
| this worktree / `/workspace/project` / user install | neither as write | controller hashes only | **none** |

The participant session is other-family and source-blind. The controller must not remain in the
terminal, must not paste argv, and must not hand this Result.

### 6. Transcript, evidence, preservation, no-private-correction

**Transcript** (`$EVIDENCE/transcript.jsonl`), one object per invocation:

```text
t, cwd, argv[], exit_code, duration_ms, stdout_sha256, stderr_sha256, envelope_path|null, note
```

Save every `--json` stdout as `$EVIDENCE/envelopes/NNN-<command>.json`. Save help text hashes, not
bodies larger than 64 KiB twice. No environment dumps, no file bodies from `$FIX` except the
pre-declared edit path hash, no PII, no secrets, no product telemetry, no network.

**Preservation subjects** — recursive path-sorted SHA-256 of regular files, no symlink follow,
immediately **before controller setup** and **after participant terminal**:

| ID | Subject |
|---|---|
| P1 | this worktree at admitted HEAD (`git rev-parse HEAD`, `git status --porcelain`, tracked tree hash) |
| P2 | `/workspace/project` if distinct from P1 |
| P3 | `subprojects/algorithm-evolution-agent-lab` inside P1/P2 (canonical dogfood) |
| P4 | user-install prefixes if present (`$HOME/.local`, `/usr/local`) |
| P5 | historical runner file `…/m1-release-rehearsal-20260822/target/release/codefire-rs` if still a regular file |
| P6 | `$KEY` after freeze (must be identical after the run) |
| P7 | `$DOCS` |
| P8 | `$FIX` **before import** (baseline) and **after** (expected change; record both) |

P1–P7 after-hash must equal before-hash. P8 after-hash must differ only by the import metadata, the
one edited source file, CodeFire state under `$FIX/.codefire`, and `.codefire-open`. Any other P8
path change is off-task mutation.

**No-private-correction:** `intervention_n` must remain 0. Protocol is invalid if the participant
reads `crates/`, `tests/`, `docs/development/`, this Result, or receives spoken/pasted argv, Fire
IDs, resolution text, or “use import not init”. Task-card restatement of the digest contract is
allowed; command coaching is not.

### 7. Timing, friction, counters, terminal success, falsifiers

Clock starts at participant receipt of `$CF`+`$DOCS`+`$FIX`+task card (`T0`).

| Mark | Definition |
|---|---|
| `setup_s` | T0 → first successful `"$CF" --help` or `capabilities --json` |
| `ttfur_s` | T0 → first envelope that names `repo` + `branch` + `base_commit` (expected: `import --json`) |
| `edit_s` | T0 → first save of `src/receipt.py` that is not the stub |
| `scan_s` | T0 → first post-edit `scan --json` with `ok=true` |
| `fire_clear_s` | T0 → last successful extinguish that leaves a later verify with `open_required_fires=0` |
| `verify_s` | T0 → first `verify --json` with committable decision and `unit-tests` passed |
| `seal_s` | T0 → first live `commit --json` with `applied=true` and non-null `commit_id` |
| `confirm_s` | T0 → `status --json` `state=open-clean` and `open_fires=0` **and** `show main --json` `show.commit` equals that `commit_id` |

Counters: `command_n`, `help_n`, `docs_file_n`, `json_n`, `refusal_n` (non-zero exits),
`wrong_action_n`, `intervention_n`, `fire_n` (distinct display_id seen), `extinguish_n`.

**Terminal success (all required):**

- one sealed `data.commit_id`;
- `status` `branch=main`, `state=open-clean`, `open_fires=0`, `base` equals that commit;
- `show main` `show.commit` equals that commit; `show.parents` is the import empty base;
- `show.claim.result` is `passed` or documented legacy `consistent`;
- verify execution record for `unit-tests` passed (first useful product result, not the seal itself);
- P1–P7 preserved; P8 only expected paths changed;
- `intervention_n=0`; no Git/network/user-install.

**Protocol falsifiers (study invalid; do not score as product stop):** private correction; participant
read of product source/tests/internal docs; this Result used as answer key; P1–P7 drift; user-install
write; canonical Algorithm write; Python runner; un-rebound historical runner; Git as VCS bridge;
network; telemetry; `review-pack --output` inside `$FIX`.

**Product/path falsifiers:** crash; `init` of `$FIX` not recovered from public help/JSON; executing
import's canned commit as the seal; treating `live_open`/`local_unisolated` as portable; treating
`passed` as correctness/security; cannot name base vs head; mutation beyond `src/receipt.py` plus
CodeFire state; cannot complete a seal; cannot run confirmation without coaching.

Node 1 owns adopt/narrow/stop numeric thresholds. This node only names the marks and falsifiers
the fixture/path can observe.

### 8. First missing contract

**First missing contract on the selected path:** `import --json` `next_actions` are a pre-CFR-170
command-string triple

```text
scan --json
verify --json
commit -m "Initial import"
```

with `target.path` but **no** `codefire.action_execution.v1`, **no** Fire handling, **no** `--path`
flag, and a canned commit message that looks executable. The live scan → extinguish → verify →
commit loop **does** publish `execution.ready` / `argv` / `argv_template`. There is no public rule
that says which contract governs first-seal, or that import's commit command is not executable
until fires are gone and verify is committable.

Secondary holes, not selected as the first: `init` has no JSON; top-level help lists `init` first;
successful `commit --json` returns empty `next_actions` so status/show confirmation is not
machine-prompted; `show` has no `--path`.

These are **not** a reproduced product blocker. A participant who executes the first import action
(`scan --json`) and then only `execution`-bearing actions can still attempt the path. Terminal
failure remains an allowed M1 result. **No write set is admitted.**

If node 5b later reproduces that source-blind participants cannot complete first-seal without
private correction **because of this mixed import contract**, the smallest later candidate (still
unadmitted) would be parse/render only in `import_operation_plan`: attach
`codefire.action_execution.v1` to import next_actions, drop or `ready=false` the canned commit
until verify-committable, and bind `--path`. Tests around
`import_existing_project_dry_run_and_apply_adopt_non_empty_directory`. No sealed-format change.

### 9. CFR/CFB overlap

| Item | Class | Why |
|---|---|---|
| CFR-171 full claim descriptor / lossless show certificate object / checked claim schema | `park` | existing `profile` / `phase` / `subject.kind` / `trust_kind` / `immutable` / `result` are enough to *attempt* local-vs-portable comprehension; full descriptor is not a start blocker |
| `claim.subject.kind=live_open` on a sealed `show` | `park` | comprehension hazard to measure (§4); not pre-admitted as a product write |
| historical `open-clean` vs `opened=0` | `park` | doctor opened/active is the live-open authority; not required to start first-seal |
| CFR-172 Git adapter / one-screen UX / inspect-as-this-study | `park` | this study is native first-seal of a disposable import, not the parked inspect experiment |
| CFR-173 Fire-cohort / telemetry / multi-repo pilot / metric calculator | `park`; telemetry in core `drop` | only the local timing/friction fields in §7 are protocol for this study |
| CFR-168 / CFB-136 | `park` | not reproduced on this tiny fixture path; no budget/cache write |
| CFR-169 / CFB-133 / CFB-136 residuals | `park` | off the first-seal critical path |
| CFR-170 all-command execution residual | `park` | local scan/extinguish/verify/commit already carry execution; import gap is §8 and stays unadmitted |
| request-list `applied_head` omission | `drop` | no remote commands on this path |
| legacy directory-digest method gap | `park` | not used by the selected argv |
| Python fallback as participant runner | `drop` | Rust JSON is the evidence surface |
| Git bridge / hosted service / agent platform / user-install mutation / Algorithm mutation | `drop` | M1 non-goals |

**`block-m1`:** none.

**Smallest write set:** none. Only this packet.

Focused tests already distinguishing the selected surfaces (do not re-run from this node):

- `crates/codefire-cli/src/tests.rs::import_existing_project_dry_run_and_apply_adopt_non_empty_directory`
- `crates/codefire-cli/src/tests.rs::command_capabilities_are_shared_by_help_completion_and_metrics_parser`
- `crates/codefire-cli/src/automation.rs` tests: `verification_commit_action_is_parameterized_by_message_and_preserves_path`, `ready_verify_details_action_binds_path_in_argv_without_a_shell`, scan path-preserving next_actions
- `crates/codefire-cli/src/tests/show_branch_list_contract.rs` show `data_schema` / claim projection

### 10. Exact node-3 handoff

Fan-in with node 1. Do not run a participant. Do not implement §8. Do not admit a product fix.
Do not hand this Result to the participant.

Node 3 must:

1. Keep HEAD crate identity `3925fc035690296fa5e132b630dab65a7fbd5ddb1a55f0fb1226e6efd295d97a` / 101
   files, or rehash if a crate-touching commit is later admitted.
2. Admit a runner only by the §1 fresh-build rule, or by a **written** historical rebind of
   `90cb2895…` that records the `01a4de0` provenance remainder.
3. Admit the `$ROOT` topology, `$FIX=receipt-digest` fixture bytes per §2, public sequence per §3,
   preservation P1–P8, and falsifiers per §6–§7.
4. Combine node 1's segment/questions/thresholds with this path. If they conflict on the task,
   prefer this fixture's digest implementation as the sealed work and node 1's questions as the
   comprehension rubric; do not enlarge the fixture.
5. Write the run packet and optional isolated fixture materialization **only after** that admission.
   Node 4 stays blocked unless a later run reproduces §8 as a product defect rather than a
   terminal study result.

Next legal action: orchestrator fan-in (board node 3). Not node 4. Not node 5a.

