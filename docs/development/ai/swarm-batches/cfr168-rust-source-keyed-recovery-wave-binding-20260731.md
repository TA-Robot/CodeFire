# CFR-168 Rust Source-Keyed Recovery Wave Binding

Status: blocked until an accepted Cargo canonical port is integrated  
Owner: portfolio manager  
Product mutation: forbidden by this binding  
Codex worker fallback: forbidden

## Objective

Bind the four held Rust recovery lines to one accepted canonical
source-keyed Cargo entry point without rewriting their review/repair
requirements.

The existing packets were written before the source-keyed control existed.
Their direct shared-target Cargo command blocks are historical task intent,
not executable launch instructions. Their candidate identity, closure matrix,
test selection, deliberate regressions, scope, and decision rules remain in
force.

## Exact Consumers

| Line | Exact subject | Existing control | SHA-256 |
|---|---|---|---|
| File Ancestry rereview | candidate `02b0d63651703aaac88c249bdc437cd173dfa397`, tree `51a683def1ec666d9baff802cbf898392bbd5c99` | `cfr168-rust-file-ancestry-zero-residual-rereview-fugu-20260730.md` | `e39ef648f89bbe19ef1429530ff07d2f79e3ec16ab64f4fe24b1f36ec5737f99` |
| File Ancestry card | same candidate | `cfr168-rust-file-ancestry-zero-residual-repair-candidate-ready-20260730/candidate.md` | `d6148534713c75f1fd737530b2900805c987e9eac4f52f1039911c76d16824b1` |
| File Doctor rereview | candidate `cd130e544af0476162be153db58a0a840b75b134`, tree `2cf767fa00337e167b8a54e8bd20b6af5558b8df` | `cfr168-rust-file-doctor-review-note-closure-rereview-claude-20260730.md` | `42590c8930680c289e0b34b982ec3dd604aa6fc1e3636aa12e7ed99516482795` |
| File Doctor card | same candidate | `cfr168-rust-file-doctor-review-note-closure-repair-candidate-ready-20260731/candidate.md` | `6809216e1da16f700541ea12eba5e74ea6a03199fa102bfd98c27f0feac2ab25` |
| Rust HTTP rereview | candidate `92fc8103bb4de400eaf40e9db76075130dedbb35`, tree `53ebad73bb99dbead5207efd34c64126ba3bbb0a` | `cfr168-rust-http-mr-preflight-evidence-closure-rereview-claude-20260730.md` | `ab711e4d98636ba12c6704f3e8d1897da79daa8f59d2a0b2dc484849325c1370` |
| Rust HTTP card | same candidate | `cfr168-rust-http-mr-preflight-evidence-closure-repair-candidate-ready-20260730/candidate.md` | `80e14e1ddaf47d5eac91d5fa0ea307c2164903b733b56f30e620a2c3e5ab3d80` |
| Storage implementation resume | rejected input `cdcb9bdefe7d25a118a9dc14171864d75f5ad304`; successor not yet frozen | `cfr168-rust-storage-review-closure-repair-control-addendum-20260731.md` | `9a56e1ce681a05acfd8b1e8a4f13670587ec6bda8da3b73a00a8da0c71d8b113` |
| Storage resume preflight | original session `c28925df-d4bc-4994-b905-73aaaf2425e5` | `cfr168-rust-storage-review-closure-resume-preflight-20260731.md` | `294b9948c790b8e84ada9a422949db4567b0979cd7e0402abe306af4205d3084` |

The card and packet digests bind inputs only. A `BLOCK` card does not become
ready because this file exists.

## Admission

No line may execute Rust until all applicable rows pass:

1. source candidate `4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912`
   receives exact independent `ACCEPT`;
2. its behavior is semantically ported through
   `cfr168-source-keyed-cargo-canonical-port-20260731.md`;
3. the exact port receives a fresh independent `ACCEPT` and is integrated
   into canonical;
4. the manager captures the integrated port commit/tree and SHA-256 for the
   wrapper, policy modules, tests, and `source-keyed-cargo-gates.md`;
5. the target candidate/worktree identity is recomputed and clean;
6. live resource, provider, conflict-domain, and Rust-heavy admission passes;
7. no direct/unkeyed Cargo process is live; and
8. a new exact manifest binds this addendum, the original packet/card, the
   accepted port, candidate identity, output paths, and every gate argv.

Acceptance of `4e5c0e7` alone is insufficient. A candidate-local copy of a
rejected or unreviewed wrapper is insufficient. Worker-green or manager-green
gate evidence is not a review verdict.

## Superseded Launch Mechanics

For these four lines only, the accepted canonical port supersedes:

- direct `cargo`, `cargo-clippy`, `cargo-fmt`, or Rust tool invocation in an
  older packet;
- direct Rust dispatch through the generic timed runner;
- `/workspace/.cache/cargo-target-shared` as an executable provenance root;
- worktree-local or arbitrary `/tmp` targets;
- alternate Rustup/Cargo homes;
- a zero-selected or ignored-only green result;
- mutable/truncated logs without a complete digest; and
- any old scheduler manifest that does not bind the accepted canonical port.

The old commands still define package, target, filter, profile, and closure
intent. Replace only their dispatch/evidence mechanics. Do not silently remove
a focused case, release profile, Clippy, format, core, deliberate regression,
or source-doc gate.

## Canonical Invocation Contract

Every Rust command is dispatched by the accepted absolute canonical tool path
with the exact isolated Python flags:

```text
/usr/bin/python3.10 -I -S
<accepted-canonical>/docs/development/tools/run_source_keyed_cargo_gate.py
--repo <exact-clean-candidate-worktree>
--timeout <finite-command-timeout>
--lock-timeout 21600
--log <new-direct-child-of-dedicated-evidence-root>
--result <new-direct-child-of-dedicated-evidence-root>
--
<exact-cargo-or-rustup-component-argv>
```

The launch environment binds:

```text
RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup
CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo
RUSTUP_TOOLCHAIN=1.97.0-x86_64-unknown-linux-gnu
CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared
CARGO_BUILD_JOBS=2
RUST_TEST_THREADS=2
CARGO_INCREMENTAL=0
```

The wrapper replaces the allocation root with the source-keyed artifact
namespace. Do not pass `--target-dir`, `--manifest-path`, `--config`, an
unfrozen lock override, or a shell/interpreter code wrapper.

Log and result names are reserved before launch, never reused, and identify
line, candidate, gate, profile, and attempt. The result must bind source,
artifact, execution, toolchain, argv, nonzero selection, monotonic duration,
complete log, terminal state, and unchanged candidate identities.

## Long-Run And Wait Contract

Do not inherit a blanket 30- or 40-minute timeout from an older packet.

- Each regenerated manifest gives every command its own finite process
  timeout.
- Use at least twice the longest verified duration for the same scope on the
  same resource profile, with a two-hour floor when no trustworthy comparable
  run exists.
- A genuinely heavy full/release/Clippy gate may receive a six-hour process
  timeout when the manifest records why.
- The separate six-hour lock timeout permits waiting behind a healthy
  serialized Rust gate; it is not charged as execution evidence.
- The wrapper, not a polling manager, owns process-group termination and the
  terminal `BLOCK` record.
- Clock-discontinuous or boot-crossing durations are `UNAVAILABLE`.

An agent may continue source inspection, call-graph analysis, report
preparation, or non-conflicting Python/docs work while another line owns the
Cargo lock. It must not start a second unkeyed build or claim a queued gate
passed.

## Deliberate Regression Contract

The accepted wrapper is clean-only, including with `--ephemeral`. Therefore a
reviewer must not edit its exact candidate and ask the gate to trust a dirty
tree.

For a Rust deliberate regression:

1. create one manager-authorized scratch worktree/branch from the exact
   candidate;
2. apply one declared mutation and commit it as a clean temporary subject;
3. record the temporary commit/tree and binary diff digest;
4. run only the focused oracle through the source-keyed gate with
   `--ephemeral`;
5. record the expected and observed failure;
6. use a normal revert commit to restore the exact candidate tree before the
   next mutation; and
7. retain the scratch evidence until the immutable review report is committed.

Reuse one scratch worktree serially; do not create one long-lived target or
one worktree per mutation. A compile failure unrelated to the intended oracle
does not kill the mutation. Temporary mutation commits do not become candidate
or product history.

## Scheduling

Provider/model ownership remains:

- File Ancestry rereview: Fugu Ultra 1.1;
- File Doctor and Rust HTTP rereviews: Claude Opus 5;
- Storage repair resume: the exact original Claude Opus 5 session.

There is no fixed total agent cap. Admit conflict-free static/report work up to
the live provider/resource window. The accepted source-keyed gate currently
uses a global Rust execution lock, so Rust command execution remains
serialized until a separately accepted bounded-concurrency control proves
otherwise.

When more than one gate is ready, choose the next owner from current
downstream release and gate readiness, and record the tie-break. Do not keep an
agent consuming provider tokens merely to wait on the lock; preserve its exact
session and resume when its next gate can advance.

## Per-Line Release

### File Ancestry

Regenerate the rereview manifest against exact `02b0d63`. Preserve the complete
closure matrix and mutation list from the Fugu packet. The direct shared-target
commands are replaced by source-keyed invocations; no prior 789-test or other
worker count is reusable by prose.

### File Doctor

The current card stays `BLOCK` until source-keyed debug and release evidence
executes the full declared NOTE-1/NOTE-2/NOTE-3 subject with nonzero selection,
plus every required adjacent/core/static gate. The old zero-selected debug
green and partial 16-case release run remain invalid counterevidence.

### Rust HTTP

The old post-cache-clear shared-profile pass is contextual only. Regenerate
the exact rereview gates and all eleven mutations through source-keyed
subjects. Do not repeat arbitrary `/tmp` target probes.

### Storage

Resume only after the existing preflight's exact-session and closed-write-set
checks pass. The implementation may not mutate product before the accepted
canonical wrapper identity is bound. Every subsequent Rust gate and both
`storage_report_publication` profiles use source-keyed dispatch. A fresh
candidate still needs a separate exact review.

## Handoff

For each released line, the manager records:

- accepted canonical port commit/tree and control-file digests;
- exact original packet/card/addendum digests;
- candidate or resume base HEAD/tree/clean state;
- regenerated manifest digest;
- every source-keyed result/log path and digest;
- selected/passed/failed/ignored counts;
- process and lock-wait monotonic durations separately;
- scratch mutation identities and dispositions;
- provider/session continuity; and
- exact next decision or blocker.

This binding accepts no Rust candidate, closes no finding, and starts no
agent. Its only authorized repository write is this file.

