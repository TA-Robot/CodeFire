# Current-State Closure Baseline — 2026-07-29

This record binds the manager's first executable baseline to the immutable safety snapshot used by
the parallel audit lanes.

## Subject

- ref: `snapshot/current-worktree-20260729`
- commit: `e2a218e015953f082245f36dbc6aee906b837e5a`
- tree: `9be2b8aca8a8538d12852b835b95aafb8601f4a4`
- parent: `f1644122134d479333cf422fdb1d48b8fa92dd8e`
- worktree: `/workspace/worktrees/current-state-baseline`
- snapshot method: temporary Git index plus `git commit-tree`; the main index and worktree were not
  changed or cleaned.

## Toolchain

- `rustc 1.97.1 (8bab26f4f 2026-07-14)`
- `cargo 1.97.1 (c980f4866 2026-06-30)`
- `clippy 0.1.97`
- `rustfmt 1.9.0-stable`
- temporary homes: `/tmp/codefire-rustup/{rustup,cargo}`

## Product gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass |
| `cargo test --workspace --locked` | pass: 1,063 tests |
| `python3 -m py_compile codefire tests/test_codefire_cli.py` | pass |
| `python3 -m unittest discover -s tests -v` without temporary Cargo on `PATH` | 647/649 pass; the two install tests failed only with `cargo is required` |
| the two failed install tests with `/tmp/codefire-rustup/cargo/bin` on `PATH` | 2/2 pass |
| effective Python result | 649/649 pass across the full run plus environment-corrected rerun |
| `./demo.sh` | pass; local lifecycle, remote upload/MR/apply, and remote Doctor completed |
| fresh install from `target/release/codefire-rs` | pass; installed Rust CLI and Python fallback under `/tmp/codefire-current-state-install.qBPoNe` |
| installed Rust CLI provenance | pass; source and installed binaries both have SHA-256 `6b375f6cbcdd28fcd3e774ad9c62d0ab6c53377daef601957c26e287183cfc5f` |
| installed CLI surface | pass; `codefire foundation 1`, and `codefire --help` exited 0 |
| Algorithm source suite | pass: 281/281 tests with `PYTHONPATH=src python3 -m unittest discover -s tests -v` |
| installed CLI against the snapshot's standalone Algorithm checkout | expected failure: typed `not_open` diagnostic, exit 2, and a path-preserving Doctor next action |
| installed CLI against the live Algorithm CodeFire open directory | pass: `open-clean`, 0 changed atoms/files, 0 open fires, exit 0 |
| read-only dogfood mutation check | pass; complete project Git status before/after had identical SHA-256 `263d1aa846ab00adeb0986749dc054bcabf6df956c0c850187762b45fd454cfd` |
| `python3 docs/development/tools/check_project_docs.py` | pass |
| `python3 docs/development/tools/check_source_docs.py` | pass: 44 modules, 34 commands, 12 object kinds |

Rust count:

- CLI unit: 768
- CLI integration: 7 (`doctor_migrate_json` 5, legacy profile 1, merge resolution 1)
- Core: 89
- Store: 196
- Util: 3

## Acceleration-control gate

The snapshot's first `check_ai_development.py` run failed only because installed Codex CLI 0.146.0
had no declared `codex_cli_0_146` adapter. This did not contradict product correctness. The main
management worktree subsequently added an explicit 0.146 adapter, retained the historical 0.144
adapter, added active-adapter validation, and passed:

- `validate_ai_runtime.py`;
- 40 AI tool unit tests;
- `check_project_docs.py`;
- the complete `check_ai_development.py` aggregate gate.

Those control-plane fixes are newer than this immutable product snapshot and require a later
integration checkpoint.

## Not yet proved

- CFR-168/CFB-136 acceptance completeness; green tests do not prove every frozen criterion is
  covered.
- Independent review of this snapshot.
- Closure of any of the eight `investigating` issues.

## Candidate disposition and canonical implementation base

The candidate-reconciliation lane exhaustively classified 315 stable patch identities across 374
candidate commits. It found 21 represented identities, 167 superseded, 71 alternative designs, 50
docs-only history identities, six later-root identities still needed, and zero unknowns.

In particular, the Python tail ending at `aa5e57d8b1e22fb83792412c68f255ceaf603e39`
is not the P0 integration base. A mechanical probe at `c37b3be82f31f4a54088e5cdbdd2e8d3012bda4b`
proved that its focused 21-file/243-test corpus is executable, but that probe also imports frozen,
production-unwired response/reducer/preimage alternatives. The manager stopped its broad Python
gate after this disposition and will not merge the probe wholesale.

Independent review commit `6482e07f7b0f5be39abf0e90e4125290da082b1c` additionally returned
`REQUEST_CHANGES` for the execution-evidence tail: persisted records can make generated evidence
quadratic while reservations remain derived from a one-write request. The reviewed path is not
production-wired, so the finding reinforces rejection of that alternative rather than reopening it
as the current production fix.

All P0 product implementation now starts from this record's immutable `e2a218e...` snapshot on
`integration/cfr168-p0-canonical`. Rust-only work may be prototyped on the earlier probe only where
`git diff --quiet e2a218e c37b3be -- crates` proves the entire Rust tree byte-identical. Python work
must be implemented or explicitly ported onto the canonical branch and rerun there.

The CFR-169 reference-registry and CFR-171 two-commit descriptor units have separate green
integration probes, but remain predecessor-gated preparatory evidence. They must not advance issue
status or enter the canonical P0 branch before CFR-168/CFB-136 Close.
