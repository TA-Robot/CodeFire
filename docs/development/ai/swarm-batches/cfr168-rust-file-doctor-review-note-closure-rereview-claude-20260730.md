# CFR-168 Rust File Doctor Review-Note Closure Rereview

Status: exact candidate bound; review launch blocked pending
acceptance-profile revalidation

Provider/model when unblocked: fresh Claude Opus 5 (`claude-opus-5`)

Role: independent reviewer

Mode: report-only

A report-only manifest may be registered for this exact subject, but it is not
launch-authorized while the bound candidate card remains `BLOCK`. Do not run
`dev-swarm validate`, `plan`, or `start` until a later immutable
manager-revalidation record explicitly authorizes review launch.

## Objective

Independently review the File Doctor successor for NOTE-1 through NOTE-3.
Prove its typed/fail-closed terminal boundary, reserve-before-allocation
behavior, and unavailable-subject authority fence through the live production
route, while preserving every previously accepted File Doctor residual.

Use a new agent/session/worktree. Do not reuse the implementer or either
earlier reviewer session.

## Exact bound subject

| Item | Exact value |
|---|---|
| Accepted product predecessor | `52a025ecda3fe2f5ad8b5b2a485562e7c16ab6de` |
| Accepted eight-residual candidate | `1a04490806b59f8c39b5c1cf03f42d06aa750a98` |
| Earlier accepted report SHA-256 | `c1b39ffc6f25cf9f19007af3cd5336421400e467e6e3c28969de775b86ed8045` |
| Zero-residual rereview report commit | `1c555ff3f426d7fa6874db92ceb71515a8d908e6` |
| Zero-residual report SHA-256 | `f7c52b05e312cc131fdfc2e2c0d9d486f5ee372b5ec93a85fdb44a95fb3ce1b9` |
| NOTE repair task base | `1e5a6f9467b90251b656e240f8c9b4f4700468a8` |
| NOTE repair contract SHA-256 | `4b94d8f5e811ed968d9d9cefcbf5fd244b6847b09566cee59fc2bb36b03940c1` |
| Candidate SHA / parent / tree | `cd130e544af0476162be153db58a0a840b75b134` / `1e5a6f9467b90251b656e240f8c9b4f4700468a8` / `2cf767fa00337e167b8a54e8bd20b6af5558b8df` |
| Additive commit list/count | `cd130e544af0476162be153db58a0a840b75b134` / `1`; merge commits `0` |
| Cumulative binary patch SHA-256 | `84dbf05bced17d9b6aa27e90cf900c57e49338f1835340ae283256613ce0d02a` |
| Exact changed paths/blobs/numstat | five rows frozen below and in the candidate card |
| Clean tracked/untracked evidence | clean; porcelain empty at `2026-07-31T02:52:13Z` |
| Candidate card | `docs/development/ai/swarm-results/cfr168-rust-file-doctor-review-note-closure-repair-candidate-ready-20260731/candidate.md` |
| Identity helper SHA-256 | `c4606a4b48af8f844a9b42b32e681cbc028df8b884fe75a66dcb6fafc16557b9` |
| Overall candidate status | `BLOCK`; identity-only status `READY_FOR_REVIEW` |

Exact changed paths are:

| Status | Path | Added | Deleted | Base blob | Head blob |
|---|---|---:|---:|---|---|
| `M` | `crates/codefire-cli/src/remote/doctor.rs` | 1873 | 282 | `410d648f933ab796cd26e2822b3313a42014a73b` | `34fd05a8a1609f00bd2b6ff31baa623e8d952c45` |
| `M` | `docs/development/change-playbook.md` | 19 | 2 | `7fe98c6943650b071b85b5c5ac0c4943bb4bd1cc` | `c49f1d446ccc264770944e8e4cbe2897040ea7ad` |
| `M` | `docs/development/command-source-trace.md` | 1 | 1 | `f85d194aa1838e3ebbb76c6d56ea24e0a74e1186` | `87c875ac0d57949cf7fdb92017607d54f6ca9623` |
| `M` | `docs/development/source-architecture.md` | 23 | 5 | `d722bb26fc122d689c1c911c4fe169195a477a7a` | `12c0aefe8945609ca0529800bee2327bbd2f3b8e` |
| `M` | `docs/development/source-data-model-catalog.md` | 1 | 1 | `9fff73b5f348cb521d2d395b141f918a0ffbeb01` | `6367fed1c190bbe07373622c92049e07cee4fe7d` |

The reviewer recomputes all identities, ancestry from `1e5a6f9` and
`1a04490`, commit range, actual paths, patch digest, and clean state. Any
mismatch is `REQUEST_CHANGES`.

## Pre-launch acceptance-profile block

The implementer used `/tmp/codefire-rustup/{rustup,cargo}` and
worktree-local `target/`, `target/clippy-review`, and
`target/release-review`. Its committed-tree release gate never produced a
terminal result, and no `cargo test -p codefire-core --locked` invocation was
found. Worker test claims are contextual evidence only.

The manager then attempted the exact shared profile. Debug returned rc 0 but
ran 0 tests against a stale shared-target artifact (log SHA-256
`2bcf08a342fed5517c56f77fbb9621b4080793968c10c677dd4a8e8f4c205b20`).
Release compiled the exact candidate and passed 16 `live_file_doctor_*` tests
(log SHA-256
`b7cae22be6ce2a17c16e3ed747ee62f5d6d5a2611248c5ceb403427525012acc`),
but excluded the required N1 `file_remote_doctor_*` cases and the N2 exact
measurement case. The release timer also recorded wall-clock discontinuity;
its monotonic duration was 550.765473361 seconds. The manager stopped the
chain before gate 03, preserved both terminal records, and left the candidate
clean. Exact argv, result digests, counts, and provenance are frozen in the
candidate card. Neither process-level `passed` result authorizes review.

Before review launch, a manager-owned immutable revalidation record must bind
the exact candidate tree, exact shared profile below, complete terminal logs,
non-vacuous artifact provenance, and post-run clean identity for focused
debug/release N1/N2/N3, the Doctor and full CLI gates, core, strict Clippy,
format, source docs, and diff hygiene. That successor record must explicitly
authorize review launch. Never infer authorization from scheduler `passed`,
the two manager attempt statuses, this prepared manifest, or the
identity-only `READY_FOR_REVIEW` field.

## Scope

Read the exact candidate, prior reports, NOTE repair contract, live
`run_remote_doctor`/production factory path, focused tests, source contracts,
candidate card, and Rust composition packet.

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
export RUSTUP_TOOLCHAIN=1.97.0-x86_64-unknown-linux-gnu
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=2
export CARGO_INCREMENTAL=0
```

Do not invoke Cargo directly. Treat each Cargo command below as the logical
argv after `--` to `run_source_keyed_cargo_gate.py`, using clean exact HEAD
and dedicated external evidence paths as specified in
[`source-keyed-cargo-gates.md`](../source-keyed-cargo-gates.md). Do not create
a worktree-local target or another Rustup/Cargo home. Record
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

## Decision and handoff

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
