# CFR-168 Rust File Doctor Review-Note Closure Rereview Prep

Status: blocked pending exact candidate binding

Provider/model when unblocked: fresh Claude Opus 5 (`claude-opus-5`)

Role: independent reviewer

Mode: report-only

No runnable manifest exists while any candidate field remains `UNBOUND`.

## Objective

Independently review the File Doctor successor for NOTE-1 through NOTE-3.
Prove its typed/fail-closed terminal boundary, reserve-before-allocation
behavior, and unavailable-subject authority fence through the live production
route, while preserving every previously accepted File Doctor residual.

Use a new agent/session/worktree. Do not reuse the implementer or either
earlier reviewer session.

## Exact blocked subject

| Item | Exact value |
|---|---|
| Accepted product predecessor | `52a025ecda3fe2f5ad8b5b2a485562e7c16ab6de` |
| Accepted eight-residual candidate | `1a04490806b59f8c39b5c1cf03f42d06aa750a98` |
| Earlier accepted report SHA-256 | `c1b39ffc6f25cf9f19007af3cd5336421400e467e6e3c28969de775b86ed8045` |
| Zero-residual rereview report commit | `1c555ff3f426d7fa6874db92ceb71515a8d908e6` |
| Zero-residual report SHA-256 | `f7c52b05e312cc131fdfc2e2c0d9d486f5ee372b5ec93a85fdb44a95fb3ce1b9` |
| NOTE repair task base | `1e5a6f9467b90251b656e240f8c9b4f4700468a8` |
| NOTE repair contract SHA-256 | `4b94d8f5e811ed968d9d9cefcbf5fd244b6847b09566cee59fc2bb36b03940c1` |
| Candidate SHA / parent / tree | `UNBOUND` / `UNBOUND` / `UNBOUND` |
| Additive commit list/count | `UNBOUND` |
| Cumulative binary patch SHA-256 | `UNBOUND` |
| Exact changed paths/blobs/numstat | `UNBOUND` |
| Clean tracked/untracked evidence | `UNBOUND` |

Before launch, the manager atomically binds every value and generates a
candidate-ready card plus fresh report-only manifest. The reviewer recomputes
all identities, ancestry from `1e5a6f9` and `1a04490`, commit range, actual
paths, patch digest, and clean state. Any mismatch is `REQUEST_CHANGES`.

## Scope

Read the exact candidate, prior reports, NOTE repair contract, live
`run_remote_doctor`/production factory path, focused tests, source contracts,
and Rust composition packet.

The only authorized write is:

`docs/development/ai/swarm-results/cfr168-rust-file-doctor-review-note-closure-rereview-claude-20260730/review.md`

Product, tests, source docs, packets, existing reports, issues, ledgers,
composition/lifecycle state, devcontainer, packaging, Install, and Dogfood are
read-only or out of scope.

## Required independent falsification

### N1: terminal invariant boundary

Require one explicit disposition.

- If typed terminal failure is chosen, drive the real production route in
  debug and release. Prove typed propagation, primary error/reason/cause
  retention, one ordered observer notification, no report, zero debt, and no
  permit. Regress by continuing, returning only the primary error, or notifying
  before drain.
- If abort is chosen, require one named invariant boundary, proof that all
  production Store/Doctor paths prevent open-permit reachability, child-
  process debug/release evidence, and why typed return cannot preserve
  ownership. Reject a renamed generic panic or test-only held permit.

### N2: reserve before digest/path allocation

Observe conservative size calculation without allocating the covered digest
or path. Reserve the correct dimension and real `DoctorGrowthSite` before each
production allocation, then retain/release symmetrically. Test digest and path
separately and together at required-1/exact/required+1, maximum path, and Unix
non-UTF-8 path. Regress reserve-after-allocation, undercharge by one, and
overreserve that rejects an admissible exact case.

### N3: unavailable-subject authority

Bind pinned parent capability, leaf, observed kind/identity when available,
operation, and typed failure origin/class. The final `RecordFence` repeats the
same bounded no-follow authority operation.

Cover metadata/read failure, deterministic denial, unchanged unavailability,
unavailable-to-readable, missing-to-created, parent replacement, symlink,
kind/identity change, failure-origin change, readable-byte change,
disappearance, and multi-fault precedence. Denial/I/O never becomes absence.
Every authority change fails as `subject_changed` before publication with
byte-identical semantic tree, zero debt/permits, and one terminal observation.
Delete the unavailable record, skip the final check, or collapse failure
origin and require targeted failure.

### Inherited closure

Independently rerun the prior 20 focused cases and all earlier R1--R8
production regressions: factory profile, growth phase, shared gate, fence
disable, stray abort file, drain bypass, label, and ObjectRecords charge.
Preserve golden quick/full JSON/text bytes, HTTP delegation, production
factory/profile/label, gate fields, typed cap, cleanup precedence, zero-debt
enforcement, and source truth. Command-wide CLI envelope/text accounting
remains explicitly excluded.

## Validation

Use this exact shared low-memory Rust profile for every Cargo command:

```bash
export RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup
export CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=2
export CARGO_INCREMENTAL=0
```

Do not create a worktree-local target or another Rustup/Cargo home. Record
exact command, counts, duration, toolchain, target, log digest, and
clean-state result for:

- focused N1/N2/N3 tests in debug and release;
- `cargo test -p codefire-cli --bin codefire-rs remote::doctor --locked`;
- `cargo test -p codefire-cli --bin codefire-rs doctor --locked`;
- the full CLI binary target;
- `cargo test -p codefire-core --locked`;
- strict all-target `codefire-cli` Clippy;
- `cargo fmt --all -- --check`;
- source-document checker; and
- exact-base `git diff --check`.

Identity, N1/N2/N3 focused debug/release tests, every deliberate regression,
and live production-route probes are non-cacheable and must be run by this
reviewer. The full CLI target, core target, strict Clippy, and format check may
be reused only from a manager-generated gate-evidence index whose key matches
the exact candidate tree, argv, environment profile above, Rust toolchain,
Cargo lock/config inputs, fixture/corpus versions, terminal pass result, and
complete log digest. Cite the key and digest when reused; otherwise run the
gate. A worker summary, mutable log name, different tree, incomplete log, or
failed/flaky/interrupted result is never reusable.

Each deliberate regression changes production code, produces the intended
targeted failure, is reverted, and is followed by recomputation of candidate
HEAD/tree/patch/clean state. Run broad gates once on the stable exact subject.

## Decision and Handoff

Commit exactly the declared report. Include exact identity, N1/N2/N3
dispositions, inherited residual matrix, production reachability, mutation
evidence, commands/results, debug/release parity, unavailable evidence,
integration overlap, and one literal verdict.

Use the compact exact-independent-review section in
`docs/development/ai/validation-report-template.md`; add detail only when an
evidence row cannot preserve the required meaning.

`ACCEPT` requires exact identity/scope, production-reachable non-vacuous proof
for N1--N3, all eight inherited residuals still closed, required independent
gates green, truthful docs, and zero new same-root Must/Should/Note. Otherwise
return `REQUEST_CHANGES`; no notes-based acceptance is permitted.

The reviewer does not integrate the candidate or mutate issue, ledger,
disposition, composition, or lifecycle state.
