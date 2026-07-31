# CFR-168 Rust Storage Exact-Rereview Closure Repair

Status: ready for exact original-implementer continuation after the shared
Rust source-binding guard and Cargo conflict preflight both pass

Provider/model: Claude Opus 5 (`claude-opus-5`)

Role: implementation repair

Resume only the original implementer session
`c28925df-d4bc-4994-b905-73aaaf2425e5` in its existing isolated worktree.
Do not start a replacement implementation lane while that exact session and
worktree remain recoverable.

## Exact immutable subject

| Item | Exact value |
|---|---|
| Rejected product candidate | `cdcb9bdefe7d25a118a9dc14171864d75f5ad304` |
| Candidate parent | `e9e42aa3c9916ca848ec736b1e31289d93f9acd2` |
| Candidate tree | `1836a83aa42df026f578b134f4b455830b873a3f` |
| Original repair task revision | `13767c7f39eca82d4a9093a055abf568814cfef0` |
| Independent-review task base | `844e4a319e458e6be2507ab8b166640a74d98c85` |
| Independent-review commit | `71222e1cec1b0dc10003845428abfb611ba8bf71` |
| Independent-review tree | `d43b5a1da31aba03943cdfe6c38c17fd5438511b` |
| Candidate-rooted report copy | `9b5f4ea` |
| Canonical report copy | `8ac6741` |
| Report Git blob | `cd0ffc8458d8240ac7a0215a365cf32b5b91e7da` |
| Report bytes | `54661` |
| Report SHA-256 | `6438df445b2d05510895fd23f73a4e71ba49238da15b00e86abf1a477b5ab960` |
| Review verdict | `REQUEST_CHANGES` |
| Original implementer session | `c28925df-d4bc-4994-b905-73aaaf2425e5` |

The immutable report is present on this product lineage at:

`docs/development/ai/swarm-results/cfr168-rust-storage-zero-residual-rereview-fugu-20260730/review.md`

Verify the candidate, report commit, tree, blob, byte count, SHA-256, and clean
state before changing product bytes. The candidate, report, and this packet
are immutable inputs. Add commits above this packet. Do not amend, rebase,
squash, reset, merge, replace, or rewrite them.

## Objective

Close R1 through R14 and all F1 through F9 dispositions without weakening the
useful behavior already present in `cdcb9bd`:

- one command-owned storage ledger across local and selected-remote work;
- iterative no-follow traversal with bounded depth and stable order;
- pin-before-read descriptor handling;
- bounded stable top-ten aggregation;
- prepublication encoding and explicit publication state;
- preserved legacy quick/full text and JSON behavior where no repaired
  boundary is crossed; and
- zero current retained/temporary debt and zero open permits on every
  supported terminal path.

The prior candidate's passing tests are regression inputs, not acceptance
evidence. Every repair below needs an independent production-reachable oracle
and a deliberate reversal that fails for the intended reason.

## Required closure

### R1 / F3 — exact local graph nodes and edges

1. Charge a traversal node only after `ReadDir::next` returns `Some(entry)`;
   EOF probes are not nodes.
2. Define explicitly whether the root is a node and charge exactly one local
   edge per parent-child relation before opening or pushing the child.
3. Keep stable visit order, depth-zero semantics, no-follow authority, and
   early-return cleanup unchanged.
4. Add hand-derived empty, flat, branching, and deep-tree node/edge oracles.
   For each, test exact `L`, `L - 1`, and one additional subject unit at fixed
   `L`.
5. Mutations that restore pre-`next` charging or remove local edge charging
   must fail the independent oracle rather than a candidate-derived snapshot.

### R2 / F7 — typed visitor-versus-Store result precedence

1. Replace both release-erased `debug_assert!` branches with an exhaustive
   typed match over `(visitor_error, scan_result)`.
2. Distinguish the expected cancellation sentinel from a different Store
   validation, finalization, authority, release, or accounting failure.
3. Define deterministic primary/secondary precedence. Preserve both errors
   where one outcome cannot safely erase the other.
4. Remove message-text dependency for Store semantic validation by adding or
   using a typed Store error contract in the smallest necessary shared scope.
5. Exercise local and remote callbacks with injected cancellation plus
   distinct finalization/release failures in debug and release.
6. A mutation that restores the debug-only assertion or discards the Store
   result must fail in release.

### R3 / F8 — publication failure cannot be terminal `Complete`

1. Do not record terminal `Complete` before publication and postpublication
   cleanup have succeeded.
2. Use an explicit sealed/prepared but nonterminal state if the encoded result
   must remain frozen before the first write.
3. First-write rejection, partial progress, flush failure, and any later
   publication failure must receive the exact typed publication-failure
   terminal reason once.
4. Update the existing test that currently expects `Complete` after a failed
   publication and add an independent terminal-state oracle.

### R4 / F8 — no second document after any visible write

1. Wrap every error after the first attempted write, including successful
   flush followed by ledger/release/finish failure, as
   `CliError::PublishedOutputFailed` or an equally typed no-second-output
   contract.
2. Top-level text and JSON error rendering must never append a second document
   after storage bytes may have become visible.
3. Add a deterministic post-flush finish-failure seam plus process-level
   stdout/stderr assertions.
4. Preserve the committed response and primary outcome where cleanup fails
   after visibility; do not pretend rollback occurred.

### R5 / F1 / F3 — traversal stack and identity authority ownership

1. Define an ADR-backed portable logical formula for the traversal stack
   collection and each stored frontier/frame record.
2. Acquire collection growth before `Vec` capacity expansion.
3. Acquire the post-visit identity-reopen authority before the descriptor or
   capability is opened.
4. Include the stack, frame, path/name, iterator scratch, handle, and reopen
   authority in the simultaneous peak with exact scopes.
5. Add omission-sensitive exact/minus-one probes for collection growth,
   per-frame state, and reopen authority independently.

### R6 / F1 — create the command ledger before storage-owned CLI allocation

1. Move command-ledger creation before cloning `--remote` values or creating
   a storage-owned `PathBuf`, or parse borrowed spans and acquire exact owners
   before every clone/materialization.
2. Carry that same ledger into `prepare_storage_report`; do not mint a second
   storage ledger after argument ownership already exists.
3. Cover large remote arguments and paths at exact/minus-one limits and prove
   no pre-ledger allocation/read/open.
4. Preserve CLI diagnostic and exit semantics for ordinary valid and invalid
   arguments.

### R7 / F1 / F9 — portable ADR-020 logical owner schema

1. Do not apply the traversal/frontier `256 + utf8(id)` unit to every Rust
   semantic owner.
2. Define named portable logical records for actual retained, temporary,
   collection, frontier, descriptor/authority, projection, and publication
   owners.
3. Define one portable path-output contract. Platform-encoded path bytes,
   lossy text, JSON serialization failure, and diagnostic ownership must not
   be conflated.
4. Tie each formula to ADR-020 and the live output contract; remove arbitrary
   shells and false public-limit cliffs.
5. Update the canonical data-model and playbook truth only after production
   code implements the exact formulas and precedence.

### R8 / F2 — structural JSON projection

1. Derive emitted fields from an independent schema/table. `object_file_json`
   has four string fields: `object_id`, `type`, `type_confidence`, and `path`.
2. Remove reliance on unrelated fixed slack that currently hides the missing
   fourth field allowance.
3. Account field names, punctuation, escaping, values, collection growth, and
   simultaneously retained report/output owners structurally.
4. Add a mutation per field allowance and prove it reaches the intended
   projection owner.

### R9 / F5 — exact URL parser owners without a false 16 KiB cliff

1. Replace the blanket twelve-owner multiplier with actual simultaneously live
   segment/vector, decoded-byte/string, server/suffix, root, and returned-value
   owners.
2. Do not assume decoded/project-root length is at least input URL length;
   percent decoding can shrink it.
3. Preserve the previously supported input domain unless an explicit product
   limit is independently accepted. If a bound is crossed, classify it as the
   correct usage/business outcome, not subject corruption.
4. Add long shrinking percent-encoded inputs, many empty/nonempty segments,
   exact/minus-one limits, and omission-sensitive owner probes.
5. If the shared parser cannot expose a sound plan/ledger contract within this
   scope, record the exact shared-helper dependency instead of fabricating a
   storage-local proof.

### R10 / F6 — exact simultaneous local and remote path lifetimes

1. Derive path-owner peaks from lexical/live ownership, not fixed counts.
2. Acquire before the first `join`, shorten scopes where possible, and avoid
   double-counting a root already retained by URL parsing.
3. Cover local `.codefire`/area paths and all remote roots/records/policy/GC/
   nonce paths separately.
4. Each path owner must have an omission-sensitive exact/minus-one oracle.

### R11 / F7 — exhaustive typed terminal classification

1. Remove the wildcard arm from `storage_terminal_class`.
2. Match all relevant `CliError` classes explicitly so a new variant causes a
   compile-time or focused-table failure.
3. Replace message substring matching for pinned subject change and incomplete
   object semantics with typed variants.
4. Prove usage/business completion, ordinary I/O, parse/render failure,
   semantic corruption, subject change, resource exhaustion, cancellation,
   publication failure, and cleanup/finalization precedence independently.

### R12 / F4 — final descriptor, parent, and path-entry authority

1. Define the supported result for in-place growth, truncation, inode
   replacement, parent replacement, and path-entry replacement.
2. Revalidate the pinned descriptor and required parent/path binding at the
   final authority fence.
3. Do not silently succeed through the old inode when the contract requires
   the current path entry; do not reopen ambiently when descriptor stability is
   the contract.
4. Add deterministic replacement, truncation, and parent-swap seams with exact
   authority snapshots and zero ownership on every outcome.

### R13 / F8 — complete writer and channel sequence matrix

Add scripted writers and real top-level route assertions for:

- first-write rejection;
- prefix then error;
- short-write completion;
- `WriteZero`;
- one and repeated `Interrupted`;
- flush failure;
- successful flush then finish/release failure; and
- text and JSON stdout/stderr behavior with exactly one visible outcome.

Each sequence must assert publication state, terminal reason, primary/
secondary error precedence, response bytes, zero ledger debt, and zero
permits. A helper-only test without the top-level channel decision is
insufficient.

### R14 / F9 — canonical source truth

After implementation is complete, synchronize:

- `docs/development/source-architecture.md`;
- `docs/development/command-source-trace.md`;
- `docs/development/source-data-model-catalog.md`; and
- `docs/development/change-playbook.md`.

The documents must describe the live portable formulas, actual ledger
creation boundary, graph semantics, descriptor/parent authority, typed
visitor/Store precedence, terminal matrix, and publication state. Structural
checker success is not semantic evidence.

## Independent reversal requirements

Run and record a separate deliberate mutation for every R1 through R14
contract. At minimum:

1. restore phantom EOF node charging;
2. remove local edge charging;
3. restore release-only visitor result discard;
4. record `Complete` before publication;
5. unwrap a post-flush finish error;
6. omit stack capacity and then identity-reopen authority separately;
7. restore pre-ledger CLI allocation;
8. restore the generic 256-byte shell;
9. declare three JSON strings instead of four;
10. restore blanket URL multiplication or the false 16 KiB implication;
11. restore fixed local/remote path counts;
12. restore terminal wildcard or text matching;
13. accept inode/parent replacement without the declared final fence;
14. remove each writer sequence oracle; and
15. restore each canonical overclaim after implementation.

Record mutation patch SHA-256, exact command, expected failure, observed
assertion, monotonic duration/log digest, and proof the mutation was removed.
A test that imports only candidate symbols, reuses production projection
helpers as its oracle, or fails for compilation/name mismatch does not count.

## Required formula, matrix, and scale evidence

The durable result must contain:

- ordinary local, selected-remote, and internal Explain production call
  chains;
- exact command-ledger creation and transfer;
- every retained/temporary/frontier/collection/path/descriptor/publication
  owner with checked formula, acquisition point, release point, and
  simultaneous peak;
- hand-derived node/edge/depth and JSON schema formulas;
- complete descriptor/parent/path authority matrix;
- complete terminal/publication/output-channel matrix;
- 1x/3x/10x curves with independently constructed subjects and fixed-axis
  expectations;
- exact/minus-one observations for every owner dimension;
- zero current retained/temporary bytes, zero permits, and exactly one
  terminal observation on every injected success/failure path; and
- exact residual handoffs for genuine shared-parser, Store, Explain, or
  file-authority dependencies. Routing is not closure.

## Scope

Authorized implementation paths:

- `crates/codefire-cli/src/storage.rs`;
- the minimum necessary dispatch/error-rendering portion of
  `crates/codefire-cli/src/main.rs`;
- storage-focused tests under `crates/codefire-cli/src/tests/`;
- the minimum necessary typed Store error contract and direct tests under
  `crates/codefire-store/src/`;
- the four canonical source documents listed under R14; and
- one new durable result under
  `docs/development/ai/swarm-results/cfr168-rust-storage-review-closure-repair-20260731/`.

Do not edit this packet, the immutable review, unrelated HTTP/file/Doctor
product, issue/lifecycle state, capability claims, management ledgers,
devcontainer, packaging, install, or subprojects. If a required shared change
falls outside the authorized paths, stop that part and record the exact
dependency rather than widening the write set silently.

## Execution and validation controls

Before the first Cargo command:

1. manager admission must report no active Cargo conflict owner;
2. the exact Rust toolchain must be `1.97.0-x86_64-unknown-linux-gnu`;
3. the project-side source-binding guard introduced for AIA-046 must bind the
   candidate tree plus tracked/untracked Cargo/crate/config/toolchain inputs to
   the shared target; and
4. any target provenance mismatch must fail closed before a test result can be
   counted.

Use:

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

Do not create a worktree-local/private target, alternate Cargo/Rustup home, or
parallel Cargo stream. Use the monotonic gate runner for every counted gate
and require nonzero expected test selection; exit code zero with zero selected
tests is failure evidence.

Run individual new regressions and mutations first. Make a durable product/
test checkpoint before the first broad gate. Near handoff, serialize:

```bash
cargo fmt --all -- --check
cargo test -p codefire-cli --locked storage_decision_work -- --nocapture
cargo test -p codefire-cli --locked storage -- --nocapture
cargo test -p codefire-store --locked
cargo test -p codefire-cli --bin codefire-rs --locked
cargo clippy -p codefire-cli --all-targets --locked -- -D warnings
cargo clippy -p codefire-store --all-targets --locked -- -D warnings
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Also run exact debug and release tests for R2, R3, R4, and R13; legacy local/
remote quick/full text and JSON cases; symlink/replacement/truncation cases;
corrupted/oversized record cases; Explain storage warnings; and process-level
single-output cases. Record argv, nonzero selected counts, monotonic durations,
toolchain, candidate tree, source key, result/log hashes, and exit status.

Do not run Docker, Install, Dogfood, unrestricted all-language discovery, or
the full Rust workspace in this lane.

## Handoff

Commit only intended product, focused tests, source documents, and the durable
result. Leave the worktree clean. Report:

- final commit, parent, tree, cumulative patch SHA-256, exact path/blob/
  numstat identity, and clean state;
- R1 through R14 and F1 through F9 closure evidence;
- every mutation and independent oracle result;
- exact formulas, matrices, limits, peaks, curves, commands, counts,
  monotonic durations, and hashes;
- source-binding/profile audit and zero-test rejection evidence;
- zero-debt/zero-permit/one-terminal evidence;
- exact unresolved shared-owner handoffs, if any; and
- a request for fresh independent adversarial review of the exact candidate.

Do not claim acceptance, integrate product bytes, Close CFR-168, Install,
Dogfood, or Retrospect from this lane.
