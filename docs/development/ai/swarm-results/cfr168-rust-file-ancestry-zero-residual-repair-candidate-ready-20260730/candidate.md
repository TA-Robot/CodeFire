# CFR-168 Rust File Ancestry Zero-Residual Repair Candidate Card

Captured: 2026-07-31T02:40:55Z

Overall status: `BLOCK`

Candidate identity status: `READY_FOR_REVIEW`

The exact candidate is clean, linear, non-empty, and inside its implementation
write set. Review launch is nevertheless blocked by the physical Rust build
conflict domain. This card is manager-recomputed identity and worker-validation
evidence; it is not an acceptance decision or product-integration
authorization.

## Exact Git identity

| Field | Manager-recomputed value |
|---|---|
| Candidate worktree | `/workspace/worktrees/cfr168-rust-file-ancestry-zero-residual-repair-base/.claude-second-agent/cff7d4d1d8086b993f4dca2f5f3a557e09247f18e669dd4f62610caabcc600a8/worktrees/cfr168-rust-file-ancestry-zero-residual-repair-r1-20260730-file-ancestry-zero-residual-repair` |
| Branch | `claude-agent/cfr168-rust-file-ancestry-zero-residual-repair-r1-20260730-file-ancestry-zero-residual-repair` |
| Base / merge base | `86a0c2e42c15bb44df8a2ff299140fedbcff4aae` / `86a0c2e42c15bb44df8a2ff299140fedbcff4aae` |
| Parent | `86a0c2e42c15bb44df8a2ff299140fedbcff4aae` |
| Head / checkout HEAD | `02b0d63651703aaac88c249bdc437cd173dfa397` / `02b0d63651703aaac88c249bdc437cd173dfa397` |
| Tree | `51a683def1ec666d9baff802cbf898392bbd5c99` |
| Additive commits / merge commits | `1` / `0` |
| Binary patch SHA-256 | `9cf5bb60a6563eb111fb9a352fda2bb44334457ac48844465e3ef1197c8f9662` |
| Tracked/untracked state | clean; porcelain output empty |
| Identity-only decision | `READY_FOR_REVIEW` |

The identity was recomputed with
`docs/development/tools/capture_candidate_identity.py`, whose exact SHA-256 is
`c4606a4b48af8f844a9b42b32e681cbc028df8b884fe75a66dcb6fafc16557b9`.
The helper resolved the committed range and implementation manifest instead of
trusting a branch tip or worker prose.

```bash
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/development/tools/capture_candidate_identity.py \
  --repo "$CANDIDATE_WORKTREE" \
  --base 86a0c2e42c15bb44df8a2ff299140fedbcff4aae \
  --head 02b0d63651703aaac88c249bdc437cd173dfa397 \
  --manifest "$IMPLEMENTATION_MANIFEST" \
  --task-id file-ancestry-zero-residual-repair \
  --format markdown
```

## Exact changed paths

| Status | Path | Added | Deleted | Base blob | Head blob | Head bytes | Head SHA-256 |
|---|---|---:|---:|---|---|---:|---|
| `M` | `crates/codefire-cli/src/remote.rs` | 2696 | 813 | `1a91bf2562ac33c5512e9d70d523508642bf478e` | `a960ff02d693aad5780f4f1894959e039db74c21` | 300514 | `657a10a0db093b2a2fabeceacfb9788139a201da2edb2c36c0636ca18f2ca3de` |
| `M` | `crates/codefire-core/src/work_budget.rs` | 83 | 0 | `6ff079aa17156008298efde2cfb2a4470ad20e78` | `c2292688437ace3c94f6f08b4ee3ac8c5e4cdf96` | 88092 | `c30e0f5bcd079d5849aa2ba9a4d2d4581135c78c0cf2a6ffe087a663817aee68` |
| `M` | `docs/development/change-playbook.md` | 42 | 0 | `12d7fc6ab41177454a3f8c2b3cb0ee156055f83f` | `1d14ca1350aab174d304a9fd50768e6f4db8bf4d` | 36615 | `b8aaa376caca596c790b71be9f4076554d36778474a83cd8e7552072b83f0f72` |
| `M` | `docs/development/command-source-trace.md` | 2 | 2 | `5feeaff7784cea83ad5db9205ddc648fe89703f6` | `95e703d356f3041cf3c5fc359f2e7a8805e0145b` | 40007 | `f6f1ece381f322d39177457fabaa4cec1df746cc71f5b7d14994013ee5f7c52e` |
| `M` | `docs/development/source-architecture.md` | 57 | 0 | `0fe3dc8470ee2a477afae7fb874fe645071908e2` | `15e0bb9739791d34215c4f753d3d54426d6cc5c5` | 102109 | `077cdf9a066b3e99733d2a6833d1f113b124cd093386e2a2b4e5cc11ec198a3c` |
| `M` | `docs/development/source-data-model-catalog.md` | 20 | 0 | `77623025f184a89a87cbcfb6447fb082edc77733` | `5a57dbb1774d996264a3d953f5b4ddf5b7ae874d` | 91471 | `985a70c1259811265dda2fa189b30ef5869a76060864f6eadb92f836f30d45ec` |

All six paths exactly match the implementation manifest's declared write set.
There are no outside or unmatched declared paths.

## Task and control binding

| Evidence | Exact value |
|---|---|
| Implementation task packet | `docs/development/ai/swarm-batches/cfr168-rust-file-ancestry-zero-residual-repair-20260730.md` |
| Implementation task packet SHA-256 | `4385f51b061be3bc608e170c1f979e0fdeaac73215196378540db556c32b92b8` |
| Implementation manifest | `docs/development/ai/swarm-batches/cfr168-rust-file-ancestry-zero-residual-repair-r1-20260730.json` |
| Implementation manifest SHA-256 | `cc04bb6d5d6d99f1e427f36a8677a3bc2df03a0f96e8335628e380cbf115e82a` |
| Cargo lock SHA-256 | `26b989f38cf05a1a5fde5770afba3c006e589f8cfa3b25456a8487db65c20059` |
| Rereview ticket commit | `8882ead03efbeb07db79e25e0098c8e20f30458a` |
| Rereview ticket SHA-256 | `e39ef648f89bbe19ef1429530ff07d2f79e3ec16ab64f4fe24b1f36ec5737f99` |
| Initial rereview manifest registration | `37b0a02867e9e5d9c3dfa29c2e20f2c9a9abf068` |
| Initial rereview manifest SHA-256 | `1ecaa2e84b60f84be8ef7b68755a512794cc8bb29edb4adda2a73ad7148b3f89` |
| Canonical equivalent registrations | `ecefdf24282ee8e99f7c6cc478c1f2fa63dbc834`, `ab119b0ae8a14d480433d41b8ab039f7148738ad` |

The historical ticket, initial manifest registration, and canonical
registrations remain immutable. A later additive control commit rebinds the
review manifest's `base_revision` to the commit that contains this card and the
identity helper. The implementation task/manifest digests above are the
candidate evidence required by the candidate-card contract; the review
manifest's successor digest is not self-embedded in its own task base.

## Provider, scheduler, and durable logs

| Evidence | Exact value |
|---|---|
| Scheduler batch / task | `cfr168-rust-file-ancestry-zero-residual-repair-r1-20260730` / `file-ancestry-zero-residual-repair` |
| Scheduler collection | task `passed`, return code `0`, head `02b0d636…`, commit count `1`, message `completed` |
| Provider / model / role / tier | Claude / `claude-opus-5` / implementer / high |
| Agent | `cfr168-rust-file-ancestry-zero-residual-repair-r1-20260730-file-ancestry-zero-residual-repair` |
| Session | `fab21822-8688-46a3-a0e0-a5aaded8e47f` |
| Run state | `.dev-swarm/cfr168-rust-file-ancestry-zero-residual-repair-r1-20260730/state.json` |
| Run-state SHA-256 | `80767e9e86b2c5b89acda6935be4c59feb9793164daedacf6b46cab19b1f4546` |
| Manager log | `.dev-swarm/cfr168-rust-file-ancestry-zero-residual-repair-r1-20260730/manager.log` |
| Manager-log SHA-256 | `7269a540d6dfbf399c72b99bcf833d215fa430fc891bd468f9b85e5adc38c753` |
| Task log | `.dev-swarm/cfr168-rust-file-ancestry-zero-residual-repair-r1-20260730/logs/file-ancestry-zero-residual-repair.log` |
| Task-log SHA-256 | `dcc04c75fbc44a8352c295bd22b0243c84f01b53b4ae3b51f0b5f28c6049e0da` |
| Event log | `.claude-second-agent/cff7d4d1d8086b993f4dca2f5f3a557e09247f18e669dd4f62610caabcc600a8/agents/cfr168-rust-file-ancestry-zero-residual-repair-r1-20260730-file-ancestry-zero-residual-repair/logs/events.jsonl` |
| Event-log bytes / SHA-256 | `12839089` / `21cb0a696e2e89a7a263d38f6d1cac4aa21fee2e3e3ae380f01fe6f73c448903` |
| Transcript | `.claude-second-agent/cff7d4d1d8086b993f4dca2f5f3a557e09247f18e669dd4f62610caabcc600a8/agents/cfr168-rust-file-ancestry-zero-residual-repair-r1-20260730-file-ancestry-zero-residual-repair/logs/transcript.jsonl` |
| Transcript bytes / SHA-256 | `12389` / `def1702e4f53cabda2329507ccea5a49cdabeb59e0c8bd54067fd420c3f09aef` |

The run is terminal, so these are full-file digests rather than prefixes that
may still grow.

## Worker validation evidence

All stable final commands used:

```bash
export RUSTUP_HOME=/tmp/codefire-rustup/rustup
export CARGO_HOME=/tmp/codefire-rustup/cargo
export PATH="$CARGO_HOME/bin:$PATH"
unset CARGO_TARGET_DIR
```

| Validation ID | Event tool-use ID | Exact command | Terminal result |
|---|---|---|---|
| `impl-focused-file-ancestry` | `toolu_019jg9vAjmCCX6myjbjjp5VJ` | `cargo test -p codefire-cli --bin codefire-rs remote::file_remote_ancestry_tests --locked` | 21 passed, 0 failed, 768 filtered |
| `impl-remote-suite` | `toolu_019jg9vAjmCCX6myjbjjp5VJ` | `cargo test -p codefire-cli --bin codefire-rs remote --locked` | 129 passed, 0 failed, 660 filtered |
| `impl-full-bin` | `toolu_019jg9vAjmCCX6myjbjjp5VJ` | `cargo test -p codefire-cli --bin codefire-rs --locked` | 789 passed, 0 failed |
| `impl-clippy-cli` | `toolu_015uWhv9EEzDYvq11UDB2KqR` | `cargo clippy -p codefire-cli --all-targets --locked -- -D warnings` | clean |
| `impl-clippy-workspace` | `toolu_015uWhv9EEzDYvq11UDB2KqR` | `cargo clippy --workspace --all-targets --locked -- -D warnings` | clean |
| `impl-format` | `toolu_015uWhv9EEzDYvq11UDB2KqR` | `cargo fmt --all -- --check` | clean |
| `impl-core` | `toolu_01Ft1dQvrATif9NuKN4isDMJ` | `cargo test -p codefire-core --locked` | 91 passed, 0 failed |
| `impl-source-docs` | `toolu_015uWhv9EEzDYvq11UDB2KqR` | `python3 docs/development/tools/check_source_docs.py` | 44 modules, 34 commands, 12 object kinds; pass |
| `impl-patch-hygiene` | `toolu_015uWhv9EEzDYvq11UDB2KqR` | `git diff --check` | clean |
| `impl-terminal-recheck` | `toolu_01HUzJWU4ASxjkUMTn9mTYrj` | full-bin test, format check, and porcelain count | 789 passed; format clean; porcelain count 0 |

The stable validation cache key is the tuple:

```text
candidate-tree=51a683def1ec666d9baff802cbf898392bbd5c99
Cargo.lock.sha256=26b989f38cf05a1a5fde5770afba3c006e589f8cfa3b25456a8487db65c20059
cargo=1.97.1-c980f4866-2026-06-30
rustc=1.97.1-8bab26f4f-2026-07-14
target=<candidate-worktree>/target
profile=debug
```

The implementer also reported mutation/reversal experiments and exact counter
curves. Those claims remain reviewer inputs, not independent review evidence.
The rereviewer must reproduce and falsify them as required by the bound ticket.

## Checkpoint and process history

1. The first scheduler run was interrupted before a task commit and archived
   with head equal to the launch revision, commit count `0`, and state digest
   `2c0e483046d383fc0f89dbbf7f667519f9a400f0487a1a1bb242d81a823aefb6`.
2. A restart then failed before launch because that old worktree still
   existed. Its archived state digest is
   `a0989171154fd29f196fcd77df45e0303007f779fd295a684356ea7f8b0ab3c7`.
3. The `r1` run used a new agent/worktree, completed normally, and created the
   sole candidate commit
   `02b0d63651703aaac88c249bdc437cd173dfa397` (`fix(remote): make file-remote
   operations one accounted zero-residual unit`).
4. The candidate was not amended, rebased, merged, or rewritten. The two
   archived attempts contributed no commit to the candidate range.

The worker installed and used the ticket-authorized private toolchain under
`/tmp/codefire-rustup` and allowed Cargo's target directory to default inside
the private candidate worktree. That differs from the later shared
low-memory execution profile
(`/home/devuser/.cache/codefire-rust-1.97` plus
`/workspace/.cache/cargo-target-shared`). The results are retained as worker
evidence, but they do not waive the rereview ticket's independent gates or
current Rust admission policy.

## Physical admission snapshot

The candidate is identity-ready, but the rereview itself is Cargo-heavy and
is classified `rust-heavy` with a 4 GiB bootstrap reservation. At
2026-07-31T02:40:55.477486Z, the admission calculation produced:

| Check | Observed | Limit / disposition |
|---|---:|---|
| Projected effective memory after reservation | 7,497,486,336 bytes | at least 6,442,450,944; pass |
| Swap used | 0 bytes | at most 2,147,483,648; pass |
| Workspace available | 903,169,585,152 bytes | at least 107,374,182,400; pass |
| Memory full / I/O full / CPU some PSI avg10 | 0.0 / 0.0 / 0.76 | 0.5 / 5 / 25; pass |
| Fugu active / window / backoff | 1 / 4 / false | pass |
| Rust build conflict domain | active File Doctor and Rust HTTP Cargo/rustc work | `BLOCK` |

The conservative admission command used zero outstanding reservation; any
newly admitted reservation not yet visible in resource pressure must be added
on the next attempt. The active builds included private File Doctor Cargo/rustc
work and shared-target Rust HTTP Cargo/rustc work. No Cargo process was
stopped, and no Fugu rereviewer was launched.

## Manager disposition

`BLOCK`

The exact candidate identity is `READY_FOR_REVIEW`, but Stage 5 does not permit
a candidate-ready launch claim while another Rust build owns the physical
conflict domain. Keep the rereview queued. After the competing Cargo/rustc
processes finish, rerun live admission with current outstanding reservations,
provider occupancy, PSI, disk, and process-tree evidence. Only a fresh
conflict-free `PASS` may replace this `BLOCK` and authorize
`dev-swarm validate`, `plan`, and `start`.
