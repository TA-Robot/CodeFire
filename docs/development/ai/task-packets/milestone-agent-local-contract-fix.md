# AI Task Packet

Status: ready

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

- exact files and action fields changed;
- before/after JSON examples for the three admitted transitions;
- focused test counts and commands;
- compatibility/schema result;
- confirmation that CFB-133 and CFR-169 were untouched;
- new source manifest and how node 5 obtains a source-matching runner;
- commit hash and next legal action.

Commit the implementation, focused tests, and required public/source docs together. Do not edit the live board, issue status, the clean-room packet, or canonical dogfood state. Node 5 remains blocked until the orchestrator integrates this result and admits a source-matching runner.

## Result

Not run.
