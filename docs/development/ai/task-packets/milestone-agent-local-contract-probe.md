# AI Task Packet

Status: completed

Packet ID: MS-agent-local-contract-probe

Schema version: human-v1

Risk score: 3

Reasoning tier: high

## Objective

Act as a fresh external worker and determine the first point where the public local CodeFire contract is insufficient to repeat the prior sealed-loop path without product-source knowledge or private command correction.

This was node 1 of completed milestone `MS-agent-local-contract`. It ran read-only in parallel with the CFR-169 seam classification.

## Allowed inputs

- `docs/development/milestones.md` and the completed `milestone-local-sealed-loop-run.md` transcript;
- public product docs (`README.md`, `docs/04_cli_spec.md`, `docs/automation_interface.md`, `docs/runbook.md`);
- the admitted runner's `--help`, subcommand help, `capabilities --json`, and read-only `status`/`show` output;
- CFR-170 and CFB-133 detail files as issue context after the clean-room pass identifies a matching gap.

Do not read `crates/**`, the Python `codefire` implementation, Rust/Python tests, or private architecture docs before recording the clean-room result. The point is to test the external contract, not reverse-engineer the implementation.

## Scope

Read-only public-contract evaluation plus this packet's Result. The canonical dogfood repository and CodeFire state are read-only. The only writable tracked file is this packet.

## Non-Goals

- implementing CFR-170, CFB-133, or any inferred fix;
- reading product source to compensate for a missing public contract;
- mutating or replaying the canonical sealed loop;
- testing remote, adoption, release, or product-economics surfaces;
- filing new work that the first reproduced M1 blocker does not require.

## Procedure

1. Starting from only public surfaces, reconstruct each choice needed for the prior `status → edit → scan → extinguish → verify → commit` path.
2. For each JSON `next_action`, decide whether its `command` is directly executable, safely parameterized, has an explicit postcondition, and cannot loop after success.
3. Use the completed prior transcript as a fixture: do not repeat its mutation. Read-only runner calls are allowed; canonical CodeFire state must not change.
4. Stop at the first missing/unsafe/cyclic contract and classify it `block-m1`, `park`, or `drop`. If no gap exists, say so and provide the complete public-only replay script.
5. Only after the clean-room verdict, map a reproduced gap to CFR-170 and/or CFB-133. Filing is not implementation permission.

## Validation

Change Status to `completed` or `blocked`, add exact evidence and the smallest candidate resolution, then run:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Do not edit product code, dogfood files/state, issue status, the board, or another packet. Do not run a full test suite, build, install, remote command, or parked work.

## Handoff

- first failing public-contract step, or complete public-only replay;
- exact help/envelope/action evidence;
- whether the action has complete required arguments and a machine-checkable postcondition;
- `block-m1` / `park` / `drop` classification;
- smallest candidate fix surface and focused validation, without implementing it;
- whether CFR-170 or CFB-133 is actually reproduced;
- independent packets still running and next legal action.

## Result

Status **completed**. The clean-room probe reproduced a public-contract gap. Classification: **`block-m1`** on the CFR-170 typed-executable-action slice for the local sealed loop. CFB-133 is **not** reproduced. No product, dogfood, or CodeFire state was mutated.

Run by: Claude Opus 5 individual agent, 2026-08-21, worktree `m1-contract-probe-20260821` at base `4185a5551230548da800c311fc60f67656178ec8`.

### Clean-room discipline actually held

Before recording this verdict the only inputs read were `docs/development/milestones.md`, the completed `milestone-local-sealed-loop-run.md` transcript as a fixture, `AGENTS.md`, and the four public product docs (`README.md`, `docs/04_cli_spec.md`, `docs/automation_interface.md`, `docs/runbook.md`), plus the checked public schema `schemas/public/command-result-v1.schema.json` that `automation_interface.md` advertises as the envelope contract. `crates/**`, the Python `codefire`, Rust/Python tests, and private architecture docs were not read. CFR-170 and CFB-133 were read only after the verdict below was fixed, as Procedure step 5 requires.

Admitted runner, re-hashed before the first call:

```text
path: /workspace/.cache/cargo-target-shared/source-keys/9c92559f4f4d51f270d27bcc5fff888936bda373dcb9150435607e6d33c861c6/release/codefire-rs
sha256: 09d9fa2561de4b687200baa4b9fe54a4159e43def1a7a3cd28dc05c2741741e4  (matches the node 1 admission)
```

Runner calls made, all inside the admitted read-only set: `--help`, `-h`, `help extinguish`, `version`, `extinguish --help`, `status --help`, `scan --help`, `verify --help`, `commit --help`, `show --help`, `capabilities --json`, `status --path <dogfood> --json`, `status --json` from `/tmp`, `show main --json` from `/tmp`. No `open`, `scan`, `fire`, `extinguish`, `verify`, `commit`, `migrate`, or remote command was run, not even with `--dry-run`.

### Step-by-step public-only reconstruction

| # | Transition | Source available to a fresh worker | Directly executable | Complete required arguments | Explicit machine-checkable postcondition | Cyclic after success |
|---|---|---|---|---|---|---|
| 1 | → `status` | `status --help`: `codefire status [path\|--path <path>] [--json] [--metrics]` | yes | yes | yes — `data.state`, `data.branch`, `data.base`, `data.open_fires` | no |
| 2 | `status` → edit → `scan` | help only. At `open-clean` the live envelope returns `next_actions: []`, so no action carries this transition | yes (`scan --help`) | yes | yes — `data.branch_state`, `changed_atom_count`, `open_fire_count` | no |
| 3 | `scan` → `extinguish` | `next_actions[3]` `extinguish_fire` | **no** | **no** | no | no |
| 4 | `extinguish` → `verify` | `next_actions[0]` `verify` | **no** | **no** | no | no |
| 5 | `verify` → `commit` | `next_actions[0]` `commit` | no (free-text `-m <message>`) | yes for path/target; message is inherently caller-authored | yes — `data.result`, `decision.outcome`, `open_required_fires`, `blocking_reasons` | no |
| 6 | `commit` → post-`status` | `commit` returns `next_actions: []`; `status` at `open-clean` returns `next_actions: []` | yes (help) | yes | yes — `data.commit_id`, `applied`, `blocked` | no — the loop terminates |

No cyclic action exists anywhere on this path. Step 5's placeholder is only a commit message, which no contract could supply; it is not counted as a gap.

### First failing public-contract step — step 3

Verbatim from the fixture's `scan --json` envelope, the only machine-readable transition out of `open-burning`:

```json
{
  "command": "codefire extinguish FIRE-C7EE54F7CE66 --resolution <type> --rationale <text> --path /workspace/project/subprojects/algorithm-evolution-agent-lab",
  "id": "extinguish_fire",
  "kind": "extinguish_fire",
  "target": {
    "display_id": "FIRE-C7EE54F7CE66",
    "fire_uid": "fire_sha256_c7ee54f7ce66631db3f769dd8b953d04",
    "path": "/workspace/project/subprojects/algorithm-evolution-agent-lab"
  }
}
```

The action is a template presented in the same `command` string field as the executable actions, and the value vocabulary needed to fill `<type>` is published on no machine-readable public surface:

```text
extinguish --help  ->  usage: codefire extinguish <fire-id> [--path <open-dir>] --resolution <type>
                       (--rationale <text>|--evidence <text>|--evidence-ref <id>) [--dry-run] [--json]
```

Help names the argument and correctly discloses the rationale/evidence choice group, but never enumerates the admissible `--resolution` values. `capabilities --json` does not either: each of its 34 command entries carries only `name`, `help_key`, `output_schema`, `pagination`, `supports_json`, `supports_metrics`, and `targets` — there is no argument schema and no value vocabulary anywhere in `codefire.capabilities.v1` or in the generated `codefire.support.v1` manifest. The action `target` carries no `allowed_resolutions` and no required-argument list. The top-level `--help` does not list `extinguish` under `common:` at all, and `help extinguish` silently falls back to the top-level listing instead of the subcommand usage.

The only public sources of the three values `addressed` / `changed` / `no-change-required` are prose examples in `docs/04_cli_spec.md:349,353,358` and `docs/runbook.md:69,403`, plus the `extinguish_policy: no-change-required:` policy key at `docs/runbook.md:84`. Those are examples, never declared as the complete admissible set. Under the M1 exit rule that every transition come from command help, a versioned JSON envelope, or machine-readable `next_actions` with complete required arguments, an example in a prose document is none of the three, and a worker cannot confirm a guessed value without probing a mutating command.

### Strictly harder failure in the same root — step 4

Collected from the same fixture, no additional runner call needed. The fixture's `extinguish --json` envelope offers exactly one action, and its inner `data.plan.next_actions[0]` is identical:

```json
{
  "command": "codefire verify --details --json",
  "id": "verify",
  "kind": "verify",
  "context": {"branch": "main"},
  "target": {"branch": "main"}
}
```

The required open-directory binding is **dropped, not merely unfilled**. All four `scan` actions carry `--path /workspace/project/subprojects/algorithm-evolution-agent-lab` and `target.path`, and `verify`'s own `commit` action carries `--path`; `extinguish`'s action carries neither the flag nor any `path` / `open_dir` / `repo_root` field in `target` or `context`. A consumer that executes the envelope's own action verbatim therefore cannot reach the open directory, and cannot repair the command from the envelope. `docs/04_cli_spec.md:341` guarantees this binding for the sibling command (`fire planのnext_actionsは--path <open_dir>付きcommandとbranch/open_dir/repo_root targetを含める`); `extinguish` has no equivalent guarantee and does not honour it.

Reproduced with an admitted read-only command that uses the same repository-discovery mechanism, run from a cwd outside the open directory exactly as the prior loop's worker cwd was:

```text
$ cd /tmp && "$RUNNER" status --json      # no --path, mirroring the offered pathless action
exit_code=2, ok=false
diagnostics[0] = {"kind":"not_open","severity":"blocking","blocking":true,"exit_code":2,
                  "message":"not inside an open CodeFire branch directory: /tmp","path":"/tmp","repairable":true}
next_actions[0].command = "codefire doctor --path /tmp --json"
```

The prior sealed loop only succeeded because its operator re-supplied `--path "$DOGFOOD"` from the packet's fixed inputs rather than following the envelope. That substitution is precisely the private command correction M1 exists to remove.

### Systemic root — no action carries a postcondition or precondition

Every `next_action` on the whole path has the shape `{id, kind, command, description, reason, context, target}`. None declares an expected postcondition, an expected precondition, a mutation class, dry-run/idempotency capability, or an argv form. `docs/automation_interface.md:43-61` documents only `kind` / `command` / `reason` / `target`, and the checked public schema types the array as unstructured:

```json
"next_actions": {"type": "array", "items": {"type": "object"}}
```

So the envelope cannot be validated for action executability at all, and a consumer must infer each action's effect from prose. Secondary, off-path observation: `show --help` exposes no `--path`, and its `invalid_usage` error returns `next_actions: []`, giving automation a dead end.

### Classification

**`block-m1`.** Steps 3 and 4 are on the exact critical path named in the M1 exit and both violate "every transition is chosen from command help, a versioned JSON envelope, or machine-readable `next_actions` with complete required arguments". Step 4 additionally forces the "private command correction" the exit forbids. This is not `park`: no architecture cleanup is required to reach the exit, and the gap is reproduced rather than hypothesised. It is not `drop`: the fixture shows the prior loop passed only because a human re-supplied the missing argument.

### Smallest candidate resolution — not implemented

Narrowest fix that clears the reproduced blocker, scoped to the local sealed-loop actions only:

1. Publish the admissible `--resolution` value set on a machine-readable surface — the `extinguish` help usage line and the `extinguish_fire` action `target` (for example `target.allowed_resolutions`) — and list `extinguish` under the top-level `common:` usage so subcommand help is discoverable.
2. Make `extinguish`'s emitted `next_actions` carry `--path <open_dir>` plus `open_dir` / `repo_root` in `target`, matching the binding `docs/04_cli_spec.md:341` already guarantees for `fire` plans, and applying to both the envelope action and `data.plan.next_actions`.

Deliberately excluded from this candidate: the full typed argv/mutation-class/postcondition action schema and the all-command decision matrix from CFR-170's Required Fix. Those are wider than the reproduced blocker and are not admitted by this probe.

Focused validation for that candidate, to be named by the fix packet rather than run here: the `extinguish` and `verify` next-action projection tests plus the existing command-envelope automation regression, and `python3 docs/development/tools/check_source_docs.py` if the command contract docs change. No workspace-wide Rust gate, installer, full Algorithm suite, Retrospect, or remote command is implied.

### Issue mapping

- **CFR-170 — reproduced.** Its Problem statement names `next actionのstring command/placeholder` and `precondition/mutation authority`; its Acceptance Criteria require that placeholder commands not be presented as machine-executable and that actions be usable as argv without shell parsing. Steps 3 and 4 are exactly that, on the M1 path. Its own Implementation Progress already records that only the CFB-133 migration slice closed and that "the shared typed executable-action schema and representative all-command decision matrix remain open", which matches this probe independently.
- **CFB-133 — not reproduced.** `migrate` is not on the `status → edit → scan → extinguish → verify → commit` path, and no action on that path returns a consumed-plan or otherwise stale recommendation. `docs/automation_interface.md:289-292` documents the postcondition-derived migrate behaviour, and both issue files record the slice as implemented. Per the board's admission rule, CFB-133 must not be admitted to M1 on this evidence.
- **CFR-169 — not classified here.** That is node 2's write set and the other provider family's decision.

Filing this mapping is not implementation permission.

### Preservation evidence — nothing mutated

Canonical CodeFire state after the probe, identical to the fixture's post-loop record:

```text
.codefire-open  60b133b3eeefcc79e20bda32843261897713b0d6e86b2f0d0111cd19862690d6  (unchanged)
repo.json       bf6d978b0eece0f185c6995630339326f56513a0c8645af3e7e984ad69a7f054  (unchanged)
opened/main     f217d0cfabb97e3ed58938e0931decde41a85a65e8cdbdd1f715938f7e038c5c  (unchanged)
branches/main   7781697ea2a08a41f8bae9a9cf2217f154261d67e1fa4f94653f410a519fa339  (unchanged)
active/open_4c320ed42560  contains only state.json  (unchanged)
```

Read-only `status --path <dogfood> --json` reports `state: "open-clean"`, `branch: "main"`, `base: "CF-COMMIT-373468dbaf827a6cc2bae9a4"`, `open_fires: 0`, `next_actions: []` — the sealed identity from `MS-local-sealed-loop` is intact. `/workspace/project` Git is unchanged at `HEAD=f1644122134d479333cf422fdb1d48b8fa92dd8e` with 246 porcelain lines (`114 M`, `132 ??`), matching the prior packet's post-loop count. The only tracked write in this session is this packet.

### Handoff

- first failing public-contract step: step 3, `scan`'s `extinguish_fire` action;
- no complete public-only replay script exists, so none is provided;
- classification `block-m1`, root CFR-170;
- CFB-133 not reproduced; CFR-169 left to node 2;
- independent packet still running: node 2 `ai/task-packets/milestone-agent-local-contract-seam.md`, CFR-169 ownership-seam classification, other provider family;
- next legal action: node 3 orchestrator admission after node 1 and node 2 fan in. No implementation, no board edit, and no issue-status change from this packet.
