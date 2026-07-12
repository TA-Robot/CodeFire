# AI Task Packet: Branch List Timeout

Status: ready when a Rust-capable environment is available.

## Objective

Investigate and fix `target/release/codefire-rs branch list --path subprojects/algorithm-evolution-agent-lab --json` timing out with no output.

## Background

- Observed during the 2026-07-11 project cleanup audit.
- `timeout 10s target/release/codefire-rs branch list --path subprojects/algorithm-evolution-agent-lab --json` exited with timeout and wrote no stdout/stderr.
- `status --json --path subprojects/algorithm-evolution-agent-lab` returned `open-clean`, 0 open fires.
- Branch list is expected to be read-only, bounded, and automation-friendly.

## Context Pack

- Required: Pack 0, Pack 1, Pack 3.
- Read:
  - `docs/development/ai/README.md`
  - `docs/development/ai/context-packs.json`
  - `docs/development/worktree-cleanup-plan.md`
  - `docs/development/command-source-trace.md`
  - `docs/development/source-architecture.md`
  - `docs/development/change-playbook.md`
  - `crates/codefire-cli/src/main.rs`
  - branch-related tests in `crates/codefire-cli/src/tests.rs`

## Scope

Allowed files:

- `crates/codefire-cli/src/main.rs`
- `crates/codefire-cli/src/cli_model.rs`
- `crates/codefire-cli/src/tests.rs`
- `crates/codefire-cli/src/tests/docs.rs`
- `target/codefire-ai/rust-build-provenance.json` as generated validation evidence only
- `docs/development/bug-backlog.md` and `docs/development/bug-issues/` only if a new issue is needed
- `docs/development/history.md` only for final evidence

Disallowed files:

- `subprojects/algorithm-evolution-agent-lab/src/**`
- `subprojects/algorithm-evolution-agent-lab/docs/**`
- object format docs unless root cause proves object-format impact
- remote/security modules unless branch-list traversal reaches them unexpectedly

## Non-Goals

- Do not redesign branch lifecycle.
- Do not change sealed commit object format.
- Do not add async runtime or new dependency.
- Do not weaken sealed commit validation.
- Do not add AI-agent orchestration to CodeFire product code.

## Invariants

- Branch heads must still validate as sealed commits.
- `branch list --json` must return `codefire.command_result.v1`.
- Read-only branch list must not mutate repository state.
- Missing/corrupt branch heads must remain structured diagnostics.
- Path handling must preserve explicit `--path` context.

## Investigation Plan

1. Reproduce with timeout in a Rust-capable environment.
2. Identify whether the hang is object graph validation, branch discovery, path discovery, lock contention, or output rendering.
3. Add a focused regression test that fails before the fix.
4. Fix the smallest root cause.
5. Confirm large object directories do not cause unbounded branch-list traversal.

## Validation

Required:

```bash
cargo fmt --check
cargo test -p codefire-cli --bin codefire-rs branch_list
python3 docs/development/tools/build_source_bound_rust.py --output target/codefire-ai/rust-build-provenance.json
cargo clippy -p codefire-cli --all-targets -- -D warnings
timeout 10s target/release/codefire-rs branch list --path subprojects/algorithm-evolution-agent-lab --json
```

Run the source-bound build only after the implementation is committed and the Cargo/crates/`.cargo`/toolchain input set is clean against that HEAD. The wrapper uses Cargo `--locked` and refuses dirty or untracked build inputs.

## Handoff

Return:

- root cause;
- files changed;
- regression test name;
- timeout reproduction result;
- validation command table;
- remaining risk.
