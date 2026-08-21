# AI Task Packet

Status: blocked

Packet ID: MS-remote-happy-path-run

Schema version: human-v1

Risk score: 6

Reasoning tier: high

## Objective

Execute the already-implemented isolated file-backed collaboration path with zero product changes: clone two sealed ancestor heads inside a copied repository, upload them, create and approve a merge request, apply it, and prove the terminal remote head/state from CodeFire JSON.

This is node 4 of active M1. Node 3 admitted the run after both classifiers parked CFR-168 and CFB-136.

## Exact inputs

- runner/public name: `/workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release/codefire`, SHA-256 `207fae943c09c2f739611e37050cbc83f34d3b53318fcefaf9d7304ba03d708b`;
- canonical copy source: `/workspace/project/subprojects/.codefire` (read-only);
- local target alias: `base`, cloned from `feature/docs-doubling-plan` at `CF-COMMIT-d068d2c3995f6da0a0959b98`;
- local source alias: `topic`, cloned from `feature/docs-full-doubling` at `CF-COMMIT-126555fdb84822a0c09c9e6a`;
- isolated root: create with `mktemp -d`; fixture `$ROOT/repo`; remote `$ROOT/remote`; URLs `cf://$ROOT/remote/org/app/{base,topic}`;
- no `server_policy.json`, actor, signing, HTTP, or canonical branch `main` operation.

The two heads are strict ancestor/descendant with a two-commit apply delta. Use no new local commit.

## Scope

- one new isolated temporary root and its file-backed remote;
- ignored evidence output under this agent worktree if helpful;
- this packet's Result.

Canonical `/workspace/project` and CodeFire state are read-only preservation subjects. Every product/source/doc path except this packet is forbidden.

## Procedure

1. Rehash the runner and capture every canonical preservation value from the probe packet §5. Stop on any mismatch.
2. Create `$ROOT/repo` and `$ROOT/remote`; `cp -a` only canonical `subprojects/.codefire` to `$ROOT/repo/.codefire`. This is the sole non-CodeFire fixture materialization step.
3. Run `doctor "$FIX" --quick --json` and `branch list --path "$FIX" --json`. Stop on any open-target/capability problem. Never name branch `main`, never run `close`/`discard`, and never pass a canonical path to a mutator.
4. With cwd `$FIX`, run public JSON commands in this exact order:
   - dry-run then live `clone feature/docs-doubling-plan base`;
   - live `clone feature/docs-full-doubling topic`;
   - dry-run then live/idempotent-key `upload base` to remote base;
   - live/idempotent-key `upload topic` to remote topic;
   - dry-run then live/idempotent-key `request-merge topic -> base`;
   - `request-list ... --json`, recording the observed MR-ID and status field names; stop if JSON contains no usable MR ID;
   - dry-run then live/idempotent-key `request-review ... --decision approve --reviewer m1-reviewer`;
   - dry-run then live/idempotent-key `request-apply`;
   - read-only `list`, `show .../base`, `request-list`, and `doctor`;
   - replay the exact same `request-apply` idempotency key and require `replayed: true` with no state change.
5. After **every** command, recheck all canonical preservation values from probe §5. Stop immediately on drift.
6. Record every command argv, exit code, envelope, plan/postcondition, MR ID/status, source/target/applied heads, object/branch/MR counts, idempotent replay, durations, and isolated-root path.
7. Set this packet `completed` or `blocked`, run validation below, and commit only this packet.

The live sequence is `upload -> request-merge -> request-review(approve) -> request-apply`; the earlier three-command board shorthand was corrected at fan-in.

## Validation

Only the command-driven isolated run, preservation checks, and:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

No build, product test, installer, HTTP, scale test, Retrospect, or canonical dogfood mutation.

## Non-Goals

- implementing or completing CFR-168/CFB-136;
- documenting discovered JSON field names outside this evidence packet;
- using human output/`awk` for MR identity;
- starting adoption or later slots;
- deleting the evidence fixture before independent confirmation.

## Handoff

- isolated root and runner identity;
- full JSON-envelope sequence and durations;
- observed MR-ID/status/applied-head fields;
- final remote source/target/MR/doctor state and replay proof;
- canonical preservation outcome;
- any `block-m1` and first failing command;
- next legal action: node 5 read-only evidence confirmation, preferably same session if no independence boundary requires otherwise.

## Result

Status **blocked** at Procedure step 3. Verdict **`block-m1` against the admitted fixture
materialization recipe** — not against product code, and not CFR-168 or CFB-136.

The admitted `cp -a` fixture is not an isolated repository. CodeFire itself rejects it: the copied
repository's `opened` registry still points at the canonical Algorithm worktree, so
`doctor "$FIX" --quick --json` returns **exit 20** with two blocking `opened_registry` issues, one of
them `repairable: false`, both anchored on the canonical marker path
`/workspace/project/subprojects/algorithm-evolution-agent-lab/.codefire-open`. This is the exact
open-target/capability condition that step 3 and probe packet §4 defect 1 pre-committed to stopping on.

**No mutator ran.** No `clone`, `upload`, `request-merge`, `request-review`, `request-apply`, `close`,
`discard`, `scan`, `verify`, `commit`, `serve` or `gc` was issued, with or without `--dry-run`. No
remote project was created — `$ROOT/remote` is still empty. Canonical preservation held exactly across
every command.

### Isolated root and runner identity

| Item | Value |
|---|---|
| Isolated root `$ROOT` | `/tmp/m1-remote-happy-path.TaHMir` (**left in place for node 5**) |
| Fixture repo `$FIX` | `$ROOT/repo` (contains `.codefire` only; 2905 files, 116 MB) |
| Remote storage `$REM` | `$ROOT/remote` — **empty, never written** |
| Evidence | `$ROOT/evidence/` — `preserve.sh`, `step.sh`, and per-command `*.argv` / `*.json` / `*.meta` / `*.stderr` |
| Runner | `/workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release/codefire` |
| Runner SHA-256 rehashed at run time | `207fae943c09c2f739611e37050cbc83f34d3b53318fcefaf9d7304ba03d708b` — matches the admitted input |

### Command outcome, in order

Every command ran with `cwd = $FIX` through `$ROOT/evidence/step.sh`, which records argv, exit code and
duration, then reruns the full canonical preservation check and refuses to continue on drift.

| # | Command | Exit | Duration | Preservation |
|---|---|---|---|---|
| 00 | preservation baseline (12 checks) | 0 | 10.449s | `PRESERVE=OK` |
| 01 | `cp -a /workspace/project/subprojects/.codefire $FIX/.codefire` | 0 | 0.509s | `PRESERVE=OK` |
| 02 | `codefire doctor $FIX --quick --json` | **20** | 2.809s | `PRESERVE=OK` |
| 03 | `codefire branch list --path $FIX --json` | 0 | 2.756s | `PRESERVE=OK` |
| 04 | `codefire doctor /workspace/project/subprojects --quick --json` (read-only control) | 0 | 2.664s | `PRESERVE=OK` |

Step 4 of the Procedure was never entered. Total wall time of the run itself is ~19s of commands plus
five 10.4s preservation sweeps.

### The blocking failure (command 02)

`data.mode = quick`, `data.skipped_checks = ["object_store_integrity"]`,
`data.checked = {branches: 8, opened: 1, active_files: 1, objects: 0}`,
`data.issue_counts = {total: 9, blocking: 2}`, envelope `ok: false`, `exit_code: 20`,
`repo: /tmp/m1-remote-happy-path.TaHMir/repo`.

The two blocking issues, verbatim fields:

```json
{"blocking": true, "category": "opened_registry", "kind": "open_marker_repository_mismatch",
 "severity": "error", "repairable": true,
 "message": "open marker repository.path does not match registry repository",
 "path": "/workspace/project/subprojects/algorithm-evolution-agent-lab/.codefire-open"}

{"blocking": true, "category": "opened_registry", "kind": "open_snapshot_divergence",
 "severity": "error", "repairable": false, "branch": "main", "phase": "steady",
 "authority": "repository_path",
 "expected": "/tmp/m1-remote-happy-path.TaHMir/repo/.codefire",
 "expected_record": "/tmp/m1-remote-happy-path.TaHMir/repo/.codefire/repo.json",
 "actual": "/workspace/project/subprojects/.codefire",
 "actual_record": "/workspace/project/subprojects/algorithm-evolution-agent-lab/.codefire-open",
 "path": "/workspace/project/subprojects/algorithm-evolution-agent-lab/.codefire-open",
 "repo": "/tmp/m1-remote-happy-path.TaHMir/repo"}
```

`next_actions` offered `inspect_legacy_branch_profile`, `inspect_open_snapshot_divergence`, and
`reopen_branch_workspace` (`codefire open <branch> <path>`). None was executed: the first two are
diagnostics outside this packet's admitted command set, and `open` is a mutator that would resolve the
canonical worktree — exactly the isolation breach this run must not commit.

**The defect is copy-induced, proven by control command 04.** The same `doctor --quick --json` run
against the canonical repository returns exit `0`, `ok: true`, `blocking: 0`, `total: 7`. So the two
blocking issues do not pre-exist in canonical state; they are produced by the admitted fixture recipe.

**The isolation boundary is not airtight, and this is the substantive finding.** A read-only command
executed against `$FIX` dereferenced a canonical absolute path and reported canonical bytes as its
issue subject. Nothing was written — preservation held — but the copy's `opened/main.json` pins
`open.path` and `active_state_path` to canonical locations, and CodeFire resolves them. Any mutator
that consults the opened registry, whether or not branch `main` is named on the command line, would do
the same resolution against a write path. Probe §4 predicted this; the run confirms it empirically at
zero cost.

Command 03 shows the fixture is otherwise sound and the two selected heads are present and usable:
envelope `ok: true`, `exit_code: 0`, `diagnostics: []`, 8 branches, `feature/docs-doubling-plan` at
`CF-COMMIT-d068d2c3995f6da0a0959b98` `closed`, `feature/docs-full-doubling` at
`CF-COMMIT-126555fdb84822a0c09c9e6a` `closed`, `main` at `CF-COMMIT-2792b54ad730e6d644668c38`
`open-clean`. The blocker is confined to branch `main`'s open registration.

### Second, independent finding for node 3: the selected heads are legacy-profile

The seven non-blocking issues in both command 02 and control command 04 are all
`legacy_branch_head_semantic_profile`, `severity: warning`, `repairable: true`, and they cover exactly
the seven `closed` feature branches — including **both** admitted heads:

```text
branch head CF-COMMIT-d068d2c3995f6da0a0959b98 is a valid legacy-unversioned sealed commit,
but it does not carry sealed-snapshot-v1 semantic proof
branch head CF-COMMIT-126555fdb84822a0c09c9e6a is a valid legacy-unversioned sealed commit,
but it does not carry sealed-snapshot-v1 semantic proof
```

`main` at `CF-COMMIT-2792b54ad730e6d644668c38` draws no such warning, so it is the only
`sealed-snapshot-v1` head in the repository. This creates a real tension in the admitted inputs that
node 3 should resolve explicitly rather than discover at node 6: the only current-profile head is the
one branch the isolation rule forbids touching, so the admitted run would exercise the
`legacy-unversioned` semantic profile end to end. That is consistent with §10.6/§10.7, which require
`clone`/`upload` to preserve legacy 12-hex and current 24-hex IDs exactly without rewriting them, and
it is non-blocking on its own — but it means the M1 evidence would prove the legacy profile path, not
the current one, and it would not move the `MS-agent-local-contract` sealed commit.

### Canonical preservation outcome

`PRESERVE=OK` after **every** command, all 12 checks, five sweeps (baseline, post-copy, post-02,
post-03, post-04). No drift, no partial state, nothing to roll back.

| Check | Expected | Observed |
|---|---|---|
| `git -C /workspace/project rev-parse HEAD` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | match |
| `git status --porcelain \| wc -l` | `248` | match |
| `.codefire/repo.json` | `bf6d978b…` | match |
| `.codefire/branches/main.json` | `bfd52e35…` | match |
| all-branches rollup digest | `8fba5dcb…` | match |
| `.codefire/opened/main.json` | `5721d710…` | match |
| `.codefire/active/open_4c320ed42560/state.json` | `54ecb285…` | match |
| `algorithm-evolution-agent-lab/.codefire-open` | `60b133b3…` | match |
| objects file count | `2891` | match |
| objects bytes | `114921765` | match |
| `.codefire-server` under `/workspace/project` | `0` | match |
| `status --path <dogfood> --json` | `ok true`, exit 0, `main`, `open-clean`, base `CF-COMMIT-2792b54ad730e6d644668c38`, `open_fires 0`, `next_actions []` | match |

### Not obtained, because step 4 never ran

MR ID, MR status field names, applied remote head, remote source/target heads, object/branch/MR counts,
idempotent `replayed: true` proof, and remote `doctor` terminal report are all **unobserved**. The
probe packet's open question about which JSON field carries the merge-request identity therefore
remains unanswered; no `awk`/human-output substitute was used, per Non-Goals.

### `block-m1` and smallest candidate resolutions (no implementation performed)

`block-m1`: the node 3 admission fixes a fixture recipe that yields a repository CodeFire declares
corrupt (exit 20) and whose registry dereferences canonical paths. The run cannot proceed under that
input. Node 3 must re-admit a corrected fixture; ranked options, none of which touches product code:

1. **Preferred — copy without the open registration.** Materialize `$FIX/.codefire` from canonical but
   leave `opened/` and `active/` empty, keeping both as the layout-required directories they are
   (`docs/04_cli_spec.md` §4.3 lists `objects`/`branches`/`opened`/`active` as required, and `main`
   simply stops being an open branch in the copy). This removes every canonical absolute back-reference
   instead of tolerating it, and it should return `doctor` to the canonical control's clean exit 0 with
   only the seven legacy warnings. Re-admission must keep `doctor "$FIX" --quick --json` exit 0 as the
   gate.
2. **Explicit, recorded exception.** Keep the current copy and narrow the admission gate to "no blocking
   issue outside `category: opened_registry` for branch `main`", plus the standing rule never to name
   `main`. This is weaker: it leaves a live canonical back-reference in the fixture, and whether
   `clone`/`upload` consult the opened registry is unknown without a further probe. If node 3 chooses
   this, the first admitted command should be `clone … --dry-run --json` with a preservation sweep
   immediately after.
3. **Rejected — copy the worktree too.** Copying `algorithm-evolution-agent-lab` alongside `.codefire`
   does not help: `open.path` and `active_state_path` are stored absolute and would still resolve to
   canonical, and the only public commands that could re-point them (`open`, `close`, `discard`) resolve
   the canonical target first.

Option 1 is the smallest change and is a harness edit, not a product edit. CFR-168 and CFB-136 stay
parked; nothing observed in this run touches decision-work budgeting or validation-cache subject reuse.

### Validation actually run by this packet

```text
python3 docs/development/tools/check_project_docs.py          -> OK
git diff --check                                              -> clean
```

No build, product test, installer, HTTP, scale test, Retrospect, or canonical dogfood mutation. The
only canonical commands issued were read-only: `status --path … --json` (twelve times, inside the
preservation sweeps) and the single `doctor … --quick --json` control.

### Next legal action

Node 3 re-admission of the fixture recipe, then a fresh node 4 attempt. Node 5 read-only confirmation
cannot proceed yet — there is no remote terminal state to confirm. `$ROOT` is left intact so node 3 or
node 5 can inspect commands 02–04 and reproduce the exit-20 admission failure without rerunning
anything.
