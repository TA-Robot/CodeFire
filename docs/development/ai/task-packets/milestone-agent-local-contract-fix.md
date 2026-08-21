# AI Task Packet

Status: completed

Packet ID: MS-agent-local-contract-fix

Schema version: human-v1

Risk score: 5

Reasoning tier: high

## Objective

Implement only the CFR-170 local-action contract slice reproduced by the clean-room probe, so a fresh worker can traverse `scan -> extinguish -> verify -> commit` from public machine-readable output plus caller-supplied resolution/rationale/message values, without source reading, shell parsing, or a private path correction.

This is node 4 of active M1 `MS-agent-local-contract`. Node 3 admitted this packet after node 1 classified CFR-170 `block-m1` and node 2 classified CFR-169 `park`.

## Reproduced blockers

1. `scan` emits `extinguish_fire` as an undifferentiated command string with `<type>` and `<text>`. Its public action does not say that the command is a template, which inputs remain, or what resolution contract the CLI accepts.
2. A successful single-fire `extinguish --json` emits `codefire verify --details --json` without `--path` and without `path` / `open_dir` / `repo_root` in the action target. From the implementing agent's external cwd this fails with `not_open`.
3. The local-loop actions provide no shell-free argv representation. This packet admits that representation only for the three transitions needed by the M1 replay, not an all-command CFR-170 matrix.

CFB-133 was not reproduced. Its migrate postcondition slice is not in this packet. CFR-169 is parked; no extraction or architecture cleanup is permitted.

## Scope

- the shared action builder/normalizer and local status/scan/verify reducers in `crates/codefire-cli/src/automation.rs`;
- the single-fire extinguish result/plan action and the smallest adjacent help text in `crates/codefire-cli/src/main.rs`;
- existing focused tests in `automation.rs` and `crates/codefire-cli/src/tests.rs`;
- `schemas/public/command-result-v1.schema.json` only as needed to make the new optional action execution metadata checkable while preserving v1 readers;
- public contract docs and canonical source docs only where the changed surface requires them.

Prefer an optional versioned execution object on the existing action, with one of:

- ready direct action: shell-free argv, explicit path/target binding, `ready: true`;
- caller-parameterized action: argv template plus named required inputs and `ready: false`.

The compatibility `command` string may remain, but on the admitted local-loop actions it must be a projection of the structured representation, not a separately authored source of truth. The representation must truthfully describe the current resolution input model; do not invent or enforce a closed enum if the implementation accepts an extensible string.

## Acceptance criteria

- `scan`'s `extinguish_fire` action is explicitly non-ready/parameterized, carries the fire/path binding, and exposes machine-readable required inputs for resolution and rationale/evidence. A consumer is not told that a placeholder string is executable.
- the single-fire extinguish action for `verify` is ready and has an exact argv containing `--path <open_dir>`; its compatibility command and target carry the same path.
- a successful `verify` action for `commit` is explicitly parameterized by the caller-authored message and preserves the open-directory path in its argv template/target.
- the three local-loop action shapes have deterministic tests, including a path containing spaces so no shell-token inference is required.
- public help/docs explain the resolution input and ready-vs-parameterized action contract sufficiently for a fresh worker.
- existing `codefire.command_result.v1` consumers remain compatible; do not rename/remove existing action fields or widen into every command.

## Non-Goals

- completing all of CFR-170 or its representative all-command decision matrix;
- changing migrate/CFB-133 behavior;
- CFR-169 module, Store, lifecycle, or test-owner extraction;
- adding a new agent/orchestration feature to CodeFire;
- remote, adoption, release, Retrospect, installer, full Algorithm, or workspace-wide Rust work;
- editing canonical dogfood source or CodeFire state in this implementation node.

## Validation

Use the pinned Rust toolchain already recorded for this source tree. Run only:

```bash
cargo fmt --check -p codefire-cli
cargo test -p codefire-cli automation::tests
cargo test -p codefire-cli extinguish_dry_run_returns_plan_without_writing_ledgers
cargo test -p codefire-cli extinguish_help_documents_batch_wrapper_schema
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

If a named test is split or renamed in the implementation, record the exact equivalent focused command. Do not replace the focused gates with `cargo test --workspace`, Clippy, build/install, or a dogfood mutation.

## Handoff

- exact files: `crates/codefire-cli/src/automation.rs`, `crates/codefire-cli/src/main.rs`, `crates/codefire-cli/src/tests.rs`, `schemas/public/command-result-v1.schema.json`, `docs/automation_interface.md`, `docs/04_cli_spec.md`, and canonical source docs for the changed automation surface. Action fields: optional `execution` (`schema`, `ready`, `argv` / `argv_template`, `required_inputs`, `required_input_groups`); existing `kind`/`id`/`command`/`reason`/`description`/`target`/`context` unchanged.
- before/after JSON: see Result.
- focused tests: `cargo test -p codefire-cli automation::tests` (24 passed); `extinguish_dry_run_returns_plan_without_writing_ledgers` (1); `extinguish_help_documents_batch_wrapper_schema` (1).
- schema: `command-result-v1` still additionalProperties-false at the envelope; `next_actions` items remain open objects with optional `$defs/actionExecution`. v1 readers keep working.
- CFB-133 and CFR-169 untouched. Board, probe packet, seam packet, issues, and dogfood state untouched.
- rust source manifest: 99 files, sha256 `95ee643bfdc6d62bb1aeaf7c304c74fb4978be089e190de3b3e6debfe37f2066`. Previous runner key `9c92559f…` is stale. Node 5 needs a source-matching rebuild at this commit.
- next legal action: orchestrator integrates this commit and admits a source-matching runner; node 5 stays blocked until then.

Commit the implementation, focused tests, and required public/source docs together. Do not edit the live board, issue status, the clean-room packet, or canonical dogfood state. Node 5 remains blocked until the orchestrator integrates this result and admits a source-matching runner.

## Result

Status **completed** at admission head `ab8b10b61060c2fba002866ac0f37b009d4c1126`. Implemented only the admitted CFR-170 local-loop action slice. CFB-133, CFR-169, board, probe/seam packets, issue files, and canonical dogfood state were not edited.

### Exact files

| File | Change |
|---|---|
| `crates/codefire-cli/src/automation.rs` | shared `execution` builder/normalizer; `scan` `extinguish_fire` parameterized; `verify` `commit` parameterized; `ready_verify_details_action` |
| `crates/codefire-cli/src/main.rs` | single-fire extinguish plan `verify` action; extinguish help for resolution + ready/parameterized contract |
| `crates/codefire-cli/src/tests.rs` | dry-run verify argv/path assertions; help assertions |
| `schemas/public/command-result-v1.schema.json` | optional `$defs/actionExecution` on `next_actions` items |
| `docs/automation_interface.md`, `docs/04_cli_spec.md` | public contract |
| `docs/development/source-architecture.md`, `command-source-trace.md`, `source-data-model-catalog.md` | canonical owners |

Action fields added (optional): `execution.schema=codefire.action_execution.v1`, `execution.ready`, `execution.argv` (ready) or `execution.argv_template` + `execution.required_inputs` (+ optional `required_input_groups`). Existing fields were not renamed or removed. `command` on the three admitted actions is a projection of argv/argv_template.

### Before / after (three admitted transitions)

**1. `scan` → `extinguish_fire`**

Before: `command` was `codefire extinguish FIRE-… --resolution <type> --rationale <text> --path …` with no execution metadata and no required-input list.

After (path with spaces):

```json
{
  "kind": "extinguish_fire",
  "command": "codefire extinguish FIRE-001 --resolution <resolution> --path '/tmp/open dir' --json",
  "target": {"display_id": "FIRE-001", "fire_uid": "fire_FIRE-001", "path": "/tmp/open dir"},
  "execution": {
    "schema": "codefire.action_execution.v1",
    "ready": false,
    "argv_template": ["codefire", "extinguish", "FIRE-001", "--resolution", {"input": "resolution"}, "--path", "/tmp/open dir", "--json"],
    "required_inputs": [
      {"name": "resolution", "cli_flag": "--resolution", "value_kind": "string", "required": true, "accepts": "extensible_string", "cli_default": "addressed", "examples": ["addressed", "changed", "no-change-required"]},
      {"name": "rationale", "cli_flag": "--rationale", "group": "rationale_or_evidence"},
      {"name": "evidence", "cli_flag": "--evidence", "group": "rationale_or_evidence"},
      {"name": "evidence_ref", "cli_flag": "--evidence-ref", "group": "rationale_or_evidence"}
    ],
    "required_input_groups": [{"name": "rationale_or_evidence", "min_selected": 1}]
  }
}
```

No closed resolution enum. A consumer is not told the template is executable (`ready: false`).

**2. single-fire `extinguish` → `verify`**

Before: `{"kind":"verify","command":"codefire verify --details --json","target":{"branch":"main"}}` — path dropped.

After:

```json
{
  "kind": "verify",
  "command": "codefire verify --details --json --path '/tmp/open dir'",
  "target": {"branch": "main", "path": "/tmp/open dir", "open_dir": "/tmp/open dir", "repo_root": "/tmp/repo"},
  "execution": {
    "schema": "codefire.action_execution.v1",
    "ready": true,
    "argv": ["codefire", "verify", "--details", "--json", "--path", "/tmp/open dir"]
  }
}
```

**3. `verify` → `commit`**

Before: `command` was `codefire commit -m <message> --path …` with no execution object.

After:

```json
{
  "kind": "commit",
  "command": "codefire commit -m <message> --path '/tmp/open dir' --json",
  "target": {"path": "/tmp/open dir", "pending_changes": true},
  "execution": {
    "schema": "codefire.action_execution.v1",
    "ready": false,
    "argv_template": ["codefire", "commit", "-m", {"input": "message"}, "--path", "/tmp/open dir", "--json"],
    "required_inputs": [{"name": "message", "cli_flag": "-m", "value_kind": "string", "required": true}]
  }
}
```

### Focused validation

Pinned toolchain: `rustc 1.97.0 (2d8144b78 2026-07-07)`, `cargo 1.97.0 (c980f4866 2026-06-30)`, `rustfmt 1.9.0-stable (2d8144b788 2026-07-07)` at `/home/devuser/.cache/codefire-rust-1.97/rustup/toolchains/1.97.0-x86_64-unknown-linux-gnu`.

| Command | Result |
|---|---|
| `cargo fmt --check -p codefire-cli` | pass |
| `cargo test -p codefire-cli automation::tests` | 24 passed, 0 failed (3 new local-loop execution tests) |
| `cargo test -p codefire-cli extinguish_dry_run_returns_plan_without_writing_ledgers` | 1 passed |
| `cargo test -p codefire-cli extinguish_help_documents_batch_wrapper_schema` | 1 passed |
| `python3 docs/development/tools/check_source_docs.py` | pass (44 modules, 34 commands, 12 object kinds) |
| `python3 docs/development/tools/check_project_docs.py` | pass (542 Markdown files) |
| `git diff --check` | pass |

No workspace-wide `cargo test`, Clippy, install, or dogfood mutation.

### Compatibility

`codefire.command_result.v1` envelope required fields unchanged. `next_actions` items remain objects; `execution` is optional and additional properties on the action object remain allowed. Legacy actions without `execution` still normalize as before.

### Source manifest and node 5 runner

Rust build inputs (`Cargo.toml`, `Cargo.lock`, `crates/**`): **99 files**, `manifest_sha256` `95ee643bfdc6d62bb1aeaf7c304c74fb4978be089e190de3b3e6debfe37f2066`.

The previously admitted runner `/workspace/.cache/cargo-target-shared/source-keys/9c92559f4f4d51f270d27bcc5fff888936bda373dcb9150435607e6d33c861c6/release/codefire-rs` (manifest `96737474cfec261cd9891d3d986e49f81601ef0ce4b7a25ce138526395801435`) does **not** match this source. Node 5 must not use it.

How node 5 obtains a source-matching runner, after this commit is the clean HEAD:

1. Confirm `git rev-parse HEAD` is this commit and `python3 -c "from pathlib import Path; import sys; sys.path.insert(0,'docs/development/tools'); from build_source_bound_rust import build_source_manifest; m=build_source_manifest(Path('.').resolve()); print(m['manifest_sha256'], len(m['files']))"` prints `95ee643bfdc6d62bb1aeaf7c304c74fb4978be089e190de3b3e6debfe37f2066 99`.
2. Build with the pinned 1.97.0 toolchain: `python3 docs/development/tools/build_source_bound_rust.py` (writes `target/codefire-ai/rust-build-provenance.json` bound to this HEAD), or the source-keyed cargo gate `cargo build --release --locked -p codefire-cli --bin codefire-rs` under `RUSTUP_TOOLCHAIN=1.97.0-x86_64-unknown-linux-gnu`.
3. Re-hash the binary before replay and stop if provenance/source-manifest diverge.

### Next legal action

Orchestrator integrates this packet commit and admits the new source-matching runner. Node 5 (fresh clean-room sealed-loop replay) remains blocked until that admission. This session must not start replay, rebuild for dogfood, or edit the board.
