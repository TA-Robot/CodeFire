# AI Task Packet

Status: completed

Packet ID: MS-remote-happy-path-probe

Schema version: human-v1

Risk score: 4

Reasoning tier: high

## Objective

Derive the smallest isolated file-backed fixture and public command sequence that could prove active M1 `MS-remote-happy-path`, and identify the first missing/unsafe/cyclic public-contract step before any mutation runs.

This is node 1. It runs read-only in parallel with the CFR-168/CFB-136 residual classification.

## Scope

- the active board and completed local-contract evidence;
- public CLI/help/capabilities and public remote/runbook docs;
- read-only inspection of existing fixture shapes and sealed identities;
- this packet's Result only.

Start public-only. Do not read implementation source/tests before fixing the public-contract verdict. After that verdict, existing test names may be listed only to identify a reusable isolated fixture; do not run it or mutate one.

## Questions

1. What exact local sealed branch/head, file-backed remote project(s), remote branches, actor/signing policy, and merge-request state are minimally required?
2. What exact `upload`, `request-merge`, `request-apply`, and terminal read-only commands does public help document?
3. Can the commands be run in one isolated temporary root without touching canonical Algorithm or an existing remote fixture?
4. Which JSON fields prove source upload, MR creation, apply, remote head, idempotency, and terminal state?
5. What is the first missing/unsafe/cyclic step, if any, and does it map to CFR-168/CFB-136 or neither?

## Non-Goals

- running upload/request mutations, even with dry-run;
- creating a remote project or temporary fixture;
- reading product source to compensate for a missing public contract;
- implementing CFR-168/CFB-136 or starting HTTP/scale work;
- editing the board, issues, another packet, or canonical dogfood state.

## Validation

Set Status `completed` or `blocked`, record exact help/docs/envelope evidence and `block-m1` / `park` / `drop`, then run:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

## Handoff

- exact isolated fixture topology and source sealed identity;
- exact public command sequence and expected JSON postconditions;
- first missing contract or complete runnable plan;
- smallest candidate resolution if blocked, without implementation;
- CFR-168/CFB-136 mapping;
- deterministic preservation checks and next legal action.

Commit only this packet.

## Result

Status **completed**. Verdict **`park` (no `block-m1`)**. A complete, fully isolated, non-degenerate
file-backed `upload -> request-merge -> request-review -> request-apply` plan exists using only
already-sealed commits and zero new local commits. Three defects were found; none requires a product
change before node 4, and none maps to CFR-168 or CFB-136.

Base of this probe: `aedce77ecc64c3c478d19a366e51b5425e7f6605`. No remote mutation, no dry-run
mutation, and no product source or test file was read. Public contract was fixed from `--help`,
`capabilities --json`, `schemas/public/`, `docs/04_cli_spec.md`, `docs/10_merge_remote_server.md`,
`docs/automation_interface.md`, and `docs/runbook.md` before any repository data was inspected.

### 0. Runner and environment identity

| Item | Value |
|---|---|
| Runner used for public help/capabilities | `/workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release/codefire` |
| Runner sha256 | `207fae943c09c2f739611e37050cbc83f34d3b53318fcefaf9d7304ba03d708b` (identical to the audited node-6 runner) |
| Runner source validity at this base | `git -C /workspace/project diff --quiet 4d0f755 aedce77 -- Cargo.toml Cargo.lock crates .cargo rust-toolchain rust-toolchain.toml` → exit 0, so the binary is source-identical to `aedce77` for the Rust CLI |
| `codefire --version` | `codefire foundation 1` |
| Rust toolchain in this worktree | **absent** — `cargo`, `rustc`, `rustup` all missing, no `~/.cargo`. `cargo build --workspace` cannot run here |
| Python reference CLI | `python3 codefire` present; its `upload` / `request-*` help exposes `--actor` / `--token` but **no `--json`** |

Environment consequence for node 4: the default Rust CLI cannot be rebuilt in this worktree. Node 4
must either reuse the audited `207fae94…` binary above (source-identical to this base) or provision a
toolchain. The Python CLI is not a substitute — it does not document `--json`, so it cannot satisfy the
M1 "JSON envelopes identify …" exit clause.

### 1. Minimal isolated fixture topology (Question 1, 3)

Local sealed material already exists in the canonical dogfood repository and needs no new commit. Read
read-only from `codefire branch list --path … --json` plus the sealed commit objects:

| Local branch | Head | State |
|---|---|---|
| `main` | `CF-COMMIT-2792b54ad730e6d644668c38` | `open-clean` |
| `feature/docs-full-doubling` | `CF-COMMIT-126555fdb84822a0c09c9e6a` | `closed` |
| `feature/docs-doubling-plan` | `CF-COMMIT-d068d2c3995f6da0a0959b98` | `closed` |
| `feature/docs-ops-handbook` | `CF-COMMIT-4721c006b90935dc24ee28b0` | `closed` |
| `feature/name-ops-guide` | `CF-COMMIT-6a12ef01e53174cbf8f570a0` | `closed` |
| `feature/name-ci-note-section` | `CF-COMMIT-0969987e62dfcd8f1dfd97f5` | `closed` |
| `feature/name-ci-note-md` | `CF-COMMIT-284a45376df35f84f1f5f621` | `closed` |
| `feature/nameaudit-summary-line` | `CF-COMMIT-24531d7129f7746431c427d7` | `closed` |

Ancestry walked from the sealed commit objects (164 reachable commits from `main`): all seven closed
branch heads are strict ancestors of `main`'s head and form one total chain

```text
24531d71 ⊂ 284a4537 ⊂ 0969987e ⊂ 6a12ef01 ⊂ 4721c006 ⊂ d068d2c3 ⊂ 126555fd ⊂ 2792b54 (main)
```

This matters because `upload` requires \( R \in ancestors(U) \) (`docs/10_merge_remote_server.md` §10.7)
and `request-apply` requires the fast-forward condition on the target branch. Exactly one already
sealed commit cannot produce a non-degenerate apply: with one head the same commit would have to serve
as both remote source and remote target, and no public document defines whether `request-apply`
accepts an already-up-to-date fast-forward or what field distinguishes "applied" from "no-op". The
chain above avoids that entirely at zero product cost.

**Selected topology (smallest non-degenerate, zero contact with branch `main`):**

```text
isolated root      $ROOT            = mktemp -d, outside /workspace/project
fixture repo root  $FIX             = $ROOT/repo          (contains a copy of .codefire only)
remote storage     $REM             = $ROOT/remote        (created by the first upload)
remote project     org/app          (bootstrapped by the first upload; no pre-existing fixture reused)

local source branch  topic  @ CF-COMMIT-126555fdb84822a0c09c9e6a   (clone of feature/docs-full-doubling)
local target branch  base   @ CF-COMMIT-d068d2c3995f6da0a0959b98   (clone of feature/docs-doubling-plan)
remote branches      cf://$REM/org/app/base    (target, head must move)
                     cf://$REM/org/app/topic   (source, head must not move)
apply delta          exactly 2 commits: CF-COMMIT-06cc1476157f99de1e437519, CF-COMMIT-126555fdb84822a0c09c9e6a
merge-request state  created -> approved -> applied (terminal)
actor / signing      none required, and none may be configured
```

`base`/`topic` are slash-free `clone` aliases. This is required, not cosmetic: every already-sealed
ancestor branch name contains `/`, and the documented remote URL grammar
`cf://<storage-root>/<org>/<app>/<branch>` defines no escaping for `/` inside a branch name, nor
whether the remote branch name may differ from the local one — every public example in §4.15, §10.7
and the runbook uses identical local and remote names. Cloning to a slash-free alias keeps local name
== remote branch name and removes the ambiguity from the critical path.

Actor/signing policy: the fixture must contain **no** `.codefire-server/projects/org/app/server_policy.json`.
`permissions`, `branch_protection`, `commit_signatures.required` and `request_signatures.required` are
opt-in (`docs/runbook.md` §policy examples), and the default Rust CLI's help documents **no `--actor`**
flag for `upload`/`request-*` and no `--request-key-id` for `request-merge`/`request-review`/`request-apply`
(only `upload` documents `--request-key-id`). Any policy requiring an actor or a request signature is
therefore unsatisfiable from the default CLI's documented surface and must stay out of M1.

The fixture root must be a copy because the storage root is absolute inside the URL: with
`$REM=/tmp/xxx`, the URL renders `cf:///tmp/xxx/org/app/base` (three slashes), matching the
`cf:///srv/codefire/org/app` form in `docs/automation_interface.md`.

### 2. Exact public command sequence (Question 2)

Public help of the default Rust CLI, verbatim:

```text
codefire upload <branch> <remote-url> [--dry-run] [--json] [--idempotency-key <key>] [--request-key-id <key>]
codefire request-merge <source-url> <target-url> [--dry-run] [--json] [--idempotency-key <key>]
codefire request-review <remote-project-url> <mr-id> --decision approve|reject [--reviewer <name>] [--dry-run] [--json] [--idempotency-key <key>]
codefire request-apply <remote-project-url> <mr-id> [--dry-run] [--json] [--idempotency-key <key>]
codefire request-list <remote-project-url> [--json] [--limit N] [--cursor CURSOR]
codefire list [path|<remote-project-url>] [--json] [--limit N] [--cursor CURSOR]
codefire show <branch-or-commit> [--json]
codefire doctor [path] [--quick|--full] [--json]
codefire clone <source-branch> <new-branch> [--dry-run] [--json] [--idempotency-key <key>]
codefire branch list [path|--path <repo-or-open>] [--json] [--metrics]
```

`capabilities --json` confirms remote reach for the terminal read-only commands: `list`, `show`, and
`doctor` all advertise `targets: ["local","cf","cf+http","cf+https"]`; `request-list` advertises
`["cf","cf+http","cf+https"]` with `pagination.mode = snapshot_cursor`, `page_schema
codefire.page.v1`, `cursor_schema codefire.page_cursor.v1`. Every one of the ten commands advertises
`output_schema: codefire.command_result.v1` and `supports_json: true`.

`upload`, `request-merge`, `request-review` and `request-apply` document **no** `--wait-lock` /
`--lock-timeout` in Rust help even though `docs/04_cli_spec.md` §4.1 and §4.15 and
`docs/automation_interface.md` state remote mutators accept them. Single-agent M1 does not need them;
recorded here as a help/spec drift, not a path blocker.

Exact sequence, `cwd = $FIX` for every local/mutating step (none of these commands accepts `--path`,
so the working directory is itself the isolation boundary — no `--path` ever points at canonical state):

```bash
CF=/workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release/codefire
ROOT=$(mktemp -d); FIX=$ROOT/repo; REM=$ROOT/remote
mkdir -p "$FIX" "$REM"
cp -a /workspace/project/subprojects/.codefire "$FIX/.codefire"   # harness step, not a public command

# read-only fixture admission
"$CF" doctor "$FIX" --quick --json
"$CF" branch list --path "$FIX" --json

cd "$FIX"
# slash-free aliases of two already-sealed heads
"$CF" clone feature/docs-doubling-plan base  --dry-run --json
"$CF" clone feature/docs-doubling-plan base  --json
"$CF" clone feature/docs-full-doubling topic --json

# publish target then source
"$CF" upload base  "cf://$REM/org/app/base"  --dry-run --json
"$CF" upload base  "cf://$REM/org/app/base"  --json --idempotency-key m1-upload-base-001
"$CF" upload topic "cf://$REM/org/app/topic" --json --idempotency-key m1-upload-topic-001

# merge request
"$CF" request-merge "cf://$REM/org/app/topic" "cf://$REM/org/app/base" --dry-run --json
"$CF" request-merge "cf://$REM/org/app/topic" "cf://$REM/org/app/base" --json --idempotency-key m1-mr-001
"$CF" request-list  "cf://$REM/org/app" --json          # obtain <MR> — see defect 2

# mandatory approval gate, then apply
"$CF" request-review "cf://$REM/org/app" "<MR>" --decision approve --reviewer m1-reviewer --dry-run --json
"$CF" request-review "cf://$REM/org/app" "<MR>" --decision approve --reviewer m1-reviewer --json --idempotency-key m1-review-001
"$CF" request-apply  "cf://$REM/org/app" "<MR>" --dry-run --json
"$CF" request-apply  "cf://$REM/org/app" "<MR>" --json --idempotency-key m1-apply-001

# terminal read-only confirmation, separate process, no mutator in between
"$CF" list         "cf://$REM/org/app" --json
"$CF" show         "cf://$REM/org/app/base" --json
"$CF" request-list "cf://$REM/org/app" --json
"$CF" doctor       "cf://$REM/org/app" --json
# idempotent replay, must not mutate
"$CF" request-apply "cf://$REM/org/app" "<MR>" --json --idempotency-key m1-apply-001
```

`request-review --decision approve` is **not optional**: `docs/04_cli_spec.md` §4.24 states
`request-apply` validates `approved status` before the stale and fast-forward checks. The board's M1
exit and this packet's Objective both write the path as `upload -> request-merge -> request-apply`,
omitting the approval gate. The three-command wording is unsatisfiable; the four-command sequence
above is the documented happy path. Node 3 should restate the exit sequence accordingly — this is a
wording correction, not new product scope.

### 3. Expected JSON postconditions (Question 4)

Every step: `schema = codefire.command_result.v1`, `ok = true`, `exit_code = 0`, `diagnostics = []`
(`schemas/public/command-result-v1.schema.json`, `docs/automation_interface.md`).

Dry-runs of `upload` / `request-merge` / `request-review` / `request-apply` are explicitly covered by
the public operation-plan contract: `data.type = codefire_plan_result` wrapping
`data.plan.type = codefire_operation_plan`, `version 1`, `command`, `dry_run = true`,
`would_apply = false`, `operations`, `next_actions`. Documented planned operations are
`copy_object_graph` + `write_remote_branch` (upload, §4.15), `write_merge_request` (request-merge),
`append_review` (request-review), `write_remote_branch` (request-apply) (§4.24). A `cf://` dry-run must
leave remote layout, object graph, branch record, generation state, MR record, `reviews` array and MR
status unwritten.

Idempotent replay is the one live postcondition with a named field: the second
`request-apply --idempotency-key m1-apply-001` must return `replayed: true` (§4.15, §4.24) with no new
mutation; a same-key/different-payload call must return exit code `33`.

Terminal remote health is the other schema-backed proof: `doctor cf://$REM/org/app --json` returns
`codefire.remote_doctor.v1` in `data`, and `schemas/public/remote-doctor-v1.schema.json` requires
`project_exists`, `layout`, `branch_count`, `merge_request_count`, `object_hash_errors`,
`missing_object_references`, `invalid_sealed_commit_references`, `invalid_record_versions`, `issues`,
`ok`. Expected: `project_exists true`, `branch_count 2`, `merge_request_count 1`, all four error
counters `0`, `issues []`, `ok true`.

What the public contract does **not** name — the material gap against the M1 exit clause "CodeFire's
JSON envelopes identify the remote source/target/MR/applied head":

| Proof obligation | Publicly named field | Note |
|---|---|---|
| source upload succeeded | none beyond `ok` / `exit_code` / plan `operations` | no documented uploaded-head field |
| MR created, its identity | **none** | see defect 2 |
| apply performed | none beyond `ok` / `exit_code` | no documented applied/no-op discriminator |
| applied remote head | **none** | `remote_doctor.v1` is `additionalProperties: false` and carries counts only, never heads; remote `list --json` without pagination is documented only as "the historical full-result shape" |
| idempotency | `replayed: true`, exit `33` on conflict | fully documented |
| terminal MR state | **none** | no public MR status vocabulary; `approved` appears only inside prose |

`show cf://$REM/org/app/base --json` is the best documented instrument for the applied head —
`capabilities --json` confirms `show` accepts `cf` targets and §4.16 says `show --json` returns a
sealed commit summary in the envelope — but its `data` field names are not enumerated in any public
document either. So the applied head is *observable* but not *specified*. Node 4 must record the
actual field names it observes; they become the evidence, and node 3 should treat documenting them as
follow-up docs work rather than M1 product work.

### 4. First missing / unsafe / cyclic step, in path order (Question 5)

**Defect 1 — first overall, at fixture materialization. Unsafe: the isolated copy retains absolute
back-references into canonical dogfood state.** No public command produces an isolated copy of an
existing sealed repository: `init` creates an empty repo, `import` ingests a file tree into a *new*
repo (no sealed history), and `clone <source-branch> <new-branch>` stays inside one repository. So the
fixture can only be materialized by a filesystem copy — and the copied `opened/main.json` pins
absolute canonical paths, observed read-only:

```json
"open": {
  "active_state_path": "/workspace/project/subprojects/.codefire/active/open_4c320ed42560",
  "path": "/workspace/project/subprojects/algorithm-evolution-agent-lab",
  "current_base_commit": "CF-COMMIT-2792b54ad730e6d644668c38"
}
```

Branch `main` in the copy therefore resolves its open directory and active-state capability back to the
canonical Algorithm worktree, and `docs/04_cli_spec.md` §4.1.1 evaluates `active_state_path` against
the *canonical repository of that instance*, which a copy no longer is. No public command neutralizes
this safely: `close` and `discard` both resolve the open target, i.e. the canonical path, and
`discard` is destructive.

Mitigation, fully inside the public contract and already reflected in the sequence above: the selected
topology **never names branch `main`**, uses only `closed` branches with no open directory and no
active state, runs every command with `cwd = $FIX` and no `--path`, admits the copy with
`doctor "$FIX" --quick --json` first, and re-verifies the canonical digests in §5 after **every**
command rather than only at the end. If `doctor "$FIX"` reports `open_target_resolution_failed`,
`active_state_capability_invalid`, or exits `20`, node 4 must stop and hand back to node 3 rather than
work around it. Optional environment hardening if the sandbox allows it: bind-mount
`/workspace/project` read-only for the node 4 session.

**Defect 2 — first missing step inside the public command sequence, at `request-merge` →
`request-apply`.** `request-apply` and `request-review` both require an `<mr-id>` positional, but no
public document names a JSON field carrying the created merge-request ID. `capabilities --json`
advertises only the generic `codefire.command_result.v1` for `request-merge` and `request-list`;
`docs/automation_interface.md` documents `data` shapes for `status`, `scan`, `verify`, `context`,
`doctor`, `storage report`, `explain`, `fire`, `link --batch`, `evidence add`, `review-pack`,
`patch export` and `commit`, and for the four remote mutators documents only the legacy plan wrapper.
There is no checked-in schema for a remote branch list or MR list `data` in `schemas/public/`. The only
end-to-end procedure that obtains an MR ID is `docs/runbook.md` line 148, which parses the
human-readable `request-list` output:

```bash
mr_id=$(codefire request-list "cf://$server/org/app" | awk 'NR==1 {print $1}')
```

`docs/automation_interface.md` states plainly that "Human output may change between releases", so the
documented acquisition path for the mandatory `<mr-id>` argument is the one output surface the
automation contract refuses to stabilise. This is a genuine hole in the machine-checkable happy path,
and it is exactly the M1 exit's "JSON envelopes identify … MR" clause.

Smallest candidate resolution, **no implementation**: node 4 runs `request-list "cf://$REM/org/app" --json`
immediately after the live `request-merge` and records the observed `data` field that carries the MR
ID (and the field carrying MR status) as packet evidence, then feeds that literal value to
`request-review` / `request-apply`. If — and only if — no JSON field carries the MR ID at all, the path
is blocked by an absent public field and node 3 must admit a bounded docs+field slice; that judgement
belongs to node 3 after fan-in, not to this read-only probe. Recording the observed field names in
`docs/automation_interface.md` afterwards is `park`-level docs work, not M1 product work.

**Defect 3 — the exit sequence omits the mandatory approval gate** (see §2). `request-apply` validates
`approved status`, so `upload -> request-merge -> request-apply` cannot terminate; the path is
`upload -> request-merge -> request-review --decision approve -> request-apply`. Board wording fix at
node 3.

No cyclic step was found: nothing in the documented sequence re-proposes a consumed action, and the
only repeated call is the deliberate idempotent replay, which must answer `replayed: true` instead of
re-mutating.

Also noted, not on the critical path (`park`): Rust help omits `--actor` for all remote mutators while
`docs/runbook.md` shows `upload … --actor alice`; Rust help omits `--request-key-id` for
`request-merge` / `request-review` / `request-apply` while §10.9 and the runbook require version-2
request signatures for those operations under policy; Rust help omits `--wait-lock` /
`--lock-timeout` for all four remote mutators while §4.1/§4.15 and the automation contract grant them;
Rust `request-review` help makes `--reviewer` optional while the Python reference CLI requires it;
Rust `show` help says `<branch-or-commit>` while `capabilities --json` and §4.16 both grant `cf://`
targets. All are help/spec drift on a path that does not need any of these flags in M1.

### 5. Preservation plan — deterministic checks

Baselines captured read-only at this base. Re-check **after every command** of the sequence; any
mismatch aborts node 4 immediately.

| Check | Expected value |
|---|---|
| `git -C /workspace/project rev-parse HEAD` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` |
| `git -C /workspace/project status --porcelain \| wc -l` | `248` |
| `sha256sum …/subprojects/.codefire/repo.json` | `bf6d978b0eece0f185c6995630339326f56513a0c8645af3e7e984ad69a7f054` |
| `sha256sum …/subprojects/.codefire/branches/main.json` | `bfd52e3525f6a6faf0db4f3d2d1d9d7fa3fa0d400f6561c3c5b8227e79e989f8` |
| `find …/.codefire/branches -type f \| sort \| xargs sha256sum \| sha256sum` | `8fba5dcbedd5179a27f1a8d3ce699db2c4e62f94521084ff015e025fcc582841` |
| `sha256sum …/.codefire/opened/main.json` | `5721d710f4371db88be26567d2a3c6afb3bc2458288b8c812b54ccd7556a42b2` |
| `sha256sum …/.codefire/active/open_4c320ed42560/state.json` | `54ecb285704644b814e4ca47a711bf5eba150ee747642b90b53351b2f06940d2` |
| `sha256sum …/algorithm-evolution-agent-lab/.codefire-open` | `60b133b3eeefcc79e20bda32843261897713b0d6e86b2f0d0111cd19862690d6` |
| `find …/.codefire/objects -type f \| wc -l` | `2891` |
| `du -sb …/.codefire/objects` | `114921765` |
| `find /workspace/project -maxdepth 6 -name .codefire-server \| wc -l` | `0` before and after — no remote project may ever appear under `/workspace/project` |
| `codefire status --path …/algorithm-evolution-agent-lab --json` | `ok true`, `exit_code 0`, `repo /workspace/project/subprojects`, `branch main`, `state open-clean`, `base CF-COMMIT-2792b54ad730e6d644668c38`, `open_fires 0`, `next_actions []` |

Cost note for node 4: the first `upload` copies the reachable closure of a 164-commit history out of a
2891-file / ~115 MB object store, so `$ROOT` needs headroom and the first upload is the long step.

### 6. CFR-168 / CFB-136 mapping

**Neither issue is admitted by this probe, and none of the three defects maps to either.** Defects 1–3
are fixture-provenance/isolation and public-documentation gaps, not remote decision-work budgeting and
not validation-cache subject reuse.

- **CFR-168** (no decision-equivalent budget for graph/object/HTTP work): the exact path does touch its
  concern area — the first `upload` performs an unbudgeted full-closure object graph copy — but
  `docs/automation_interface.md` scopes exit-60 `codefire.decision-work.v1` coverage to the listed
  metric commands and the Rust HTTP request/response boundary, and this path is file-backed with no
  HTTP transport. The predicted failure mode is latency/memory, not a wrong result or a typed budget
  rejection. Classification from the public side: **park** unless node 4 actually observes exit `60` /
  `resource_budget_exceeded`. Node 2 owns the implementation-side confirmation.
- **CFB-136** (command cache reuses a same-inode object rewrite as a validated hit): the precondition is
  an out-of-lock in-place rewrite of an already-validated record concurrent with a cached command. The
  selected fixture is a single-agent, single-process sequence on a freshly copied store with no
  concurrent writer and no cross-head cache reuse requirement. The path does not reach the TOCTOU
  window. Classification from the public side: **park**. Node 2 owns the final call on whether the
  subject/final-revalidation contract is already wired on this path.

Anything HTTP/`cf+http`/`cf+https`, `serve`, `gc`, pagination limits, remote GC retention, scale, and
Python parity stays parked — the file-backed path needs none of it.

### 7. Validation actually run by this packet

```text
python3 docs/development/tools/check_project_docs.py          -> OK
python3 docs/development/tools/check_ai_acceleration_docs.py  -> OK
git diff --check                                              -> clean
```

Read-only public/inspection commands used: `codefire --version`, `codefire --help`, per-command
`--help` for `upload` / `request-merge` / `request-review` / `request-apply` / `request-list` / `list` /
`show` / `doctor` / `clone` / `branch`, `codefire capabilities --json`,
`codefire branch list --path <dogfood> --json`, `codefire status --path <dogfood> --json`,
`python3 codefire <cmd> --help`, plus `sha256sum` / `find` / `du` / `git` on canonical state and direct
JSON reads of sealed commit objects. No `upload`, `request-*`, `clone`, `close`, `discard`, `scan`,
`extinguish`, `verify`, `commit`, `serve` or `gc` was run, with or without `--dry-run`. No temporary
fixture or remote project was created. No product source or test file was read.

### 8. Next legal action

Node 3 orchestrator fan-in with node 2. Node 3 should (a) restate the M1 exit sequence to include
`request-review --decision approve`, (b) admit the §2 sequence and §1 topology as the node 4 write set
with the §5 preservation gate, (c) require node 4 to record the observed MR-ID / MR-status / applied-head
JSON field names as evidence, and (d) keep CFR-168 and CFB-136 unadmitted pending node 2. No mutation,
fix, or audit node may start before that fan-in.
