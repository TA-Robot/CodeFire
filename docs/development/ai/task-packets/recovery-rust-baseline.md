# REC-02: Recover the current Rust source baseline

Status: handoff complete; M1 blocked by existing source test failures

Milestone: MS-recovery-baseline

## Objective

Install/recover a named toolchain without changing user shell defaults, pin it, provide a small reusable environment helper, and run current-source Rust validation plus CLI smoke.

## Scope

Owner: current-session Codex bootstrap worker, explicitly admitted by the 2026-09-05 recovery policy.
Write set: rust-toolchain.toml; docs/development/tools/rust_env.sh; this packet; ignored local toolchain/build/log directories. Do not modify product source, dependencies, governance, or runtime routing.

## Non-Goals

Product fixes, public release, metered APIs, global shell/profile changes, devcontainer changes.

Invariants: preserve original worktree/sequencer; source results must name actual toolchain and HEAD; no stale binary evidence.

## Handoff

Files, rationale/pin alternatives/removal, exact commands/results, toolchain path, source/head and blocker if any. No commits: integrator owns shared branch commits.

## Local Rust environment

The recovery pins Rust **1.98.1**, the exact stable release returned by the official
[Rust distribution manifest](https://static.rust-lang.org/dist/channel-rust-stable.toml)
dated 2026-09-03. The downloaded manifest is retained at
`target/recovery-rust/bootstrap/channel-rust-stable.toml`.
`rustc --version --verbose` reports commit
`48a229ceaefd4985c50990b14116b6d856af0985` (2026-09-01), host
`x86_64-unknown-linux-gnu`, LLVM 22.1.8. Cargo reports
`cargo 1.98.1 (797e8a9bc 2026-08-05)`.

From this checkout, in Bash:

```bash
source docs/development/tools/rust_env.sh
rustc --version --verbose
cargo --version
```

The helper sets `CARGO_HOME=build/local-rust/cargo`,
`RUSTUP_HOME=build/local-rust/rustup`, `TMPDIR=build/local-rust/tmp`, and
`CARGO_TARGET_DIR=target`, all as absolute paths anchored to this checkout.
It does not install anything or edit shell profiles. Toolchains, registry caches,
temporary files and build logs stay in existing ignored directories.

For a fresh x86_64 Linux checkout with no local Rust installation:

```bash
(
set -e
source docs/development/tools/rust_env.sh
curl --fail --location --proto '=https' --tlsv1.2 \
  https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init \
  -o build/local-rust/rustup-init
curl --fail --location --proto '=https' --tlsv1.2 \
  https://static.rust-lang.org/rustup/dist/x86_64-unknown-linux-gnu/rustup-init.sha256 \
  -o build/local-rust/rustup-init.sha256
(cd build/local-rust && sha256sum --check rustup-init.sha256)
chmod +x build/local-rust/rustup-init
build/local-rust/rustup-init -y --no-modify-path --default-toolchain none --profile minimal
rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy
)
```

The setup subshell stops at the first failure; never execute an installer whose
checksum check failed. Source the helper again in the caller before building. Recovery verified
installer SHA-256 `dda7234360b7f578ca8b0ddcb80145646fa61a67c1720a5abc7051b35c9fcb71`.
The toolchain installer also verifies component checksums. Other host platforms
need their matching official rustup installer.

Rationale: a numbered toolchain makes formatting, lint and compiler behavior
repeatable. The existing `rust-version = "1.75"` workspace declaration is unchanged;
this recovery does **not** establish that declared minimum as a tested MSRV.
Alternatives are a moving `stable` channel (less repeatable) or a container pin
(outside REC-02). No product dependency or lockfile is changed. To remove the local
installation, stop Rust processes and remove this checkout's `build/local-rust`
directory; remove `target/recovery-rust` for the recovery logs, or `target` for all
build artifacts. Removing the tracked pin/helper should be an intentional later
toolchain-policy change. A fresh shell restores the caller's original environment.

## Validation

Run every command below and record failures without concealing them.

Baseline source HEAD: `83b452208d97108369c757b754057bdc2a5f6ae7`.
Product sources and `Cargo.lock` are unchanged; the build input set additionally
contains the new `rust-toolchain.toml`. This is checkout-bound pre-integration
evidence, not a claim that the pin is already in that commit.

Commands after sourcing the helper:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
./target/debug/codefire-rs --help
./target/debug/codefire-rs --version
```

Logs and individual exit codes are written to `target/recovery-rust/`.
`source-manifest-before.json` hashes every Rust build input;
`baseline-results.json` records versions, HEAD, environment, command results,
source stability and the built binary digest. The orchestrator must bind the final
integration evidence to the committed pin before closing M1.

### Recovery run result, 2026-09-05

| Command | Exit | Result |
|---|---:|---|
| `cargo fmt --check` | 0 | Pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | Pass, 199.33 seconds including initial dependency work |
| `cargo test --workspace --locked` | 101 | CLI: 771 passed, 7 failed; subsequent crate suites were not run by Cargo |
| `cargo build --workspace --locked` | 0 | Pass |
| `./target/debug/codefire-rs --help` | 0 | Pass |
| `./target/debug/codefire-rs --version` | 0 | Pass |

HEAD and the full build-input manifest remained unchanged throughout all six
commands. Fresh debug binary SHA-256:
`08313015ff4cdb00e9016f7aefc888edfef18b3640b64c1c59b7d67a815e4781`.
No product file, dependency, lockfile or original worktree/sequencer was modified.
Helper Bash syntax, repeated sourcing under `set -eu`, `git diff --check`, and
`check_ai_acceleration_docs.py` also passed.

The six cleanup-related failures are:

- `http::tests::missing_bootstrap_cleanup_follows_route_intent_owner_cancellation`
- `remote::locks::tests::ledger_bootstrap_cancel_consumes_permits_and_releases_current`
- `remote::locks::tests::footprint_reservation_does_not_charge_unrealized_cumulative_work`
- `remote::locks::tests::ledger_cleanup_failure_quarantines_for_same_ledger_bounded_recovery`
- `remote::locks::tests::ledger_bootstrap_rejects_foreign_ledger_without_losing_owner`
- `remote::project::tests::bootstrap_build_cleanup_failures_close_ledger_and_remain_recoverable`

Five report `remote bootstrap recovery byte limit exceeded (45732)` or `(47136)`;
the cleanup-injection case fails its expected-error assertion. The HTTP case and
the first locks case reproduce when run individually in the same environment.
Example exact reproduction after sourcing the helper:

```bash
cargo test --locked -p codefire-cli --bin codefire-rs \
  http::tests::missing_bootstrap_cleanup_follows_route_intent_owner_cancellation \
  -- --exact --nocapture
```

The seventh failure,
`import_workflow::tests::concurrent_exact_import_with_wait_replays_the_winner`,
reports a 5000ms import-lock timeout in the parallel suite. The same exact test
passes alone in 1.95 seconds; this is evidence of an environment/load-sensitive
failure, not proof that the original failure is resolved. Focused commands,
exit codes and full traces are in `target/recovery-rust/focused-results.json` and
`focused-0.log` through `focused-2.log`.

These failures block M1. REC-02 does not authorize fixing product source; the
orchestrator must admit the exact defect and write set into a successor packet.
