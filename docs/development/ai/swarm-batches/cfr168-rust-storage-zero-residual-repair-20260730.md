# CFR-168 Rust Storage Zero-Residual Repair

Repair the exact Rust storage candidate
`b3bc32a6bd4c792edc5a1302c61e16c624a9b648` against every finding in the
independent adversarial review committed as
`4cfb607cfd6ac95d26d24a4542ac15ba56b428d6`:

`docs/development/ai/swarm-results/cfr168-rust-storage-independent-review-fugu-20260730/review.md`

Use Claude Opus 5 (`claude-opus-5`). Work only in the assigned worktree and
write set. Add new commits on top of the frozen task revision; do not amend,
rewrite, or merge the reviewed candidate, its precursors, the independent
report, or unrelated recovery branches.

The verdict is `REQUEST_CHANGES`. F1 through F9, all non-vacuity gaps assigned
to this storage surface, and every newly discovered storage-local residual are
required closure work. Preserve the useful candidate properties: one
command-owned ledger, one ledger shared by local and selected-remote scans,
bounded stable top ten, restored legacy visit order, and balanced covered
success/error paths.

## Exact ancestry and scope

- Baseline recovery precursor:
  `40c095de250df2ee9e9e34526e1926069b476e42`
- Focused finisher precursor:
  `f5bce305da7ebfd08a840a4b281e29b0476874e0`
- Rejected reviewed candidate:
  `b3bc32a6bd4c792edc5a1302c61e16c624a9b648`
- Independent review:
  `4cfb607cfd6ac95d26d24a4542ac15ba56b428d6`

The repaired candidate must remain an auditable additive successor of the
task revision. Do not integrate it into canonical, change CFR-168/CFB-136
status, install it, or run dogfood from this lane.

## Required architecture and behavior

### F1 — portable logical accounting

1. Remove every decision-work charge derived from `size_of::<T>()`, allocator
   layout, pointer width, enum padding, or compiler representation.
2. Define explicit checked portable logical units for `StorageReport`,
   `StorageWarning`, `String`, `ObjectTypeStats`, `ObjectFileStats`,
   `RemoteStorageStats`, `RemoteGenerationStats`, and every other retained or
   temporary owner used by the command.
3. Keep formulas named, reviewable, overflow-checked, and tied to the frozen
   ADR-020 contract. Do not weaken the contract by choosing an arbitrarily
   large blanket reservation.
4. Add golden formula and exact/exact-plus-one tests that cannot consult Rust
   ABI layout.

### F2 — projection and response ownership before allocation

1. Do not construct a duplicate unbounded JSON `Value`/diagnostic/action
   envelope before acquiring a proven simultaneous upper.
2. Prefer borrowed or streaming projection. If a projected tree remains,
   compute and acquire a checked portable upper before each collection,
   string, evidence, diagnostic, action, and envelope allocation.
3. Acquire the final response-buffer owner before buffer construction, and
   account the simultaneous retained report, projection, encoding scratch,
   and output state without double counting or hidden gaps.
4. Cover many maximum-length warnings and paths at exact/exact-plus-one
   projection and output boundaries.

### F3 — iterative capability traversal and zero-based depth

1. Replace recursive `scan_capability_files` traversal with an iterative,
   explicitly charged heap frontier/frame owner.
2. Define the traversal root at logical depth zero. Reject a child before it
   is pushed/opened when its edge depth exceeds the limit.
3. Preserve descriptor-relative no-follow capability semantics, stable visit
   order, quick/full behavior, and cleanup on every early return.
4. Add empty-root depth-zero, exact/exact-plus-one, symlink, and very-deep
   non-recursive fixtures. A deep input must return a typed bounded outcome,
   never consume process call stack until panic/abort.

### F4 — pin, charge, then read the same optional JSON descriptor

1. Eliminate the ambient-metadata `P` to reopened-size `Q` gap in
   `read_optional_json_accounted`.
2. Open the no-follow file first, pin the exact descriptor/capability, derive
   the charged size from that descriptor, acquire buffer/read/parse owners,
   then read and revalidate that same descriptor. Reserving the complete
   supported maximum is acceptable only if the formula is exact, portable,
   and does not create a false public-limit cliff.
3. No `Q`-sized capacity allocation or byte read may occur before sufficient
   ownership exists.
4. Add a deterministic metadata-to-open/growth/replacement race seam with
   exact authority-byte and zero-debt assertions.

### F5/F6 — URL, decoded strings, and every simultaneous path owner

1. Bound and charge the selected-remote URL byte length and segment count
   before allocation. Do not collect an unbounded segment vector.
2. Account input-proportional URL parsing, suffix/server strings,
   percent-decoded byte/string owners, project root, and every intermediate
   or returned `PathBuf`.
3. Before the first `join`, reserve the true simultaneous local owner set for
   `.codefire`, `objects`, `active`, `idempotency`, and transient paths.
   Likewise account the remote root plus `objects`, `branches`,
   `merge_requests`, `locks`, `idempotency`, policy, GC, nonce, and record
   paths while each remains live.
4. Prefer a storage-owned prevalidation/accounting adapter over expanding the
   write set into shared remote parsing. If that cannot prove the bound, stop
   and record the exact shared-helper dependency instead of hiding it.
5. Add long-URL, many-segment, percent-decoding, near-limit local path,
   near-limit remote path, and exact/exact-plus-one cases.

### F7 — semantic terminal reasons

1. Replace the catch-all corruption mapping with a typed, phase-aware terminal
   matrix.
2. Distinguish bounded usage/business completion, ordinary I/O, parse/render
   failure, actual subject corruption, subject change, resource exhaustion,
   and publication/sink failure using the accepted ledger vocabulary.
3. Preserve the primary typed CLI error and legacy exit/output semantics;
   terminal observation must not reclassify or mask it.
4. Add an independent table-driven matrix for every class and cleanup path.

### F8 — publication state and no dual stdout

1. Make the supported storage publication boundary explicit. Pre-encode and
   preflight all bytes before the first attempted write.
2. Track whether publication was not attempted, attempted without confirmed
   progress, partially progressed, or completed. Propagate that state to the
   top-level error path so `main` never appends a second text/JSON document
   after any storage output may have become visible.
3. Do not claim arbitrary `Write::write_all` is transactional. If the product
   cannot guarantee rollback after a sink accepts a prefix, define and enforce
   the strongest supported atomic boundary and return the single correct
   terminal outcome without a second envelope.
4. Add writers that accept a prefix and then fail for text and JSON, plus a
   top-level JSON regression proving no truncated-success-plus-error stream.

### F9 — canonical source truth

1. Update `docs/development/command-source-trace.md` to the live path:
   storage dispatch -> `prepare_storage_report` -> one `StorageReportRun` ->
   encode/prepublication seal/publish/release/finish.
2. Document the internal Explain consumer and its exact ownership boundary.
3. Keep all four canonical source documents synchronized with any changed
   owner, data model, invariant, or validation route.

## Required non-vacuity and residual audit

- Replace the JSON legacy oracle that calls the same production projection
  helpers with an independent expected representation/byte oracle.
- Prove formula boundaries, depth boundaries, `P -> Q` descriptor races,
  URL/segment/decoding bounds, simultaneous path ownership, many-warning
  projection peaks, partial writers, symlink variants, terminal classes, and
  top-level dual-output behavior.
- Every new regression must fail against
  `b3bc32a6bd4c792edc5a1302c61e16c624a9b648` for its intended reason, or the
  handoff must explain why a source-proof rather than an executable
  counterexample is required.
- Preserve exact legacy quick/full text and JSON bytes, warning/type/top-ten
  ordering, malformed/integrity behavior, selected-remote behavior, and exit
  codes when no repaired boundary is crossed.
- Success and every failure must finish with zero retained bytes, zero
  temporary bytes, zero open permits, one terminal observation, and no hidden
  allocation/read outside a live owner.
- Audit the review's three unresolved questions:
  typed object semantic-validation signaling, visitor-versus-finalization
  error precedence, and Explain's combined ownership. Close storage-local
  defects in this lane. For a genuine predecessor-gated cross-command
  residual, provide an exact symbol/owner/revision handoff and do not claim it
  closed.
- Do not raise a public profile limit, weaken no-follow/sealed/integrity
  checks, reorder legacy output/work, suppress an error, hide an allocation
  from instrumentation, or convert a permanent failure into a retry.

## Complexity and ownership evidence

The final handoff must include:

- the exact live call chain and proof that only one storage ledger exists;
- a portable retained/temporary formula table for every owner;
- simultaneous-owner peak derivations for local paths, remote paths, parsing,
  projection, response, and publication state;
- discovery/record/byte/depth/retained/temporary counter curves at
  1x/3x/10x, with exact/exact-plus-one observations;
- time and memory complexity for every traversal and aggregation;
- proof of bounded heap depth and stable top-ten behavior;
- authority-byte snapshots and cleanup state on every injected race/failure;
- all intentionally untouched files and any exact residual owner.

## Authorized validation ladder

Use the shared constrained toolchain and target directory:

```bash
export RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup
export CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=2
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared
```

During implementation, run non-building checks and individual new tests
first. Create a durable WIP commit before the first long Cargo gate. Near
handoff, serialize:

```bash
cargo fmt --all -- --check
cargo test -p codefire-cli --locked storage_decision_work -- --nocapture
cargo test -p codefire-cli --locked storage -- --nocapture
cargo clippy -p codefire-cli --all-targets --locked -- -D warnings
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Also run the exact legacy storage, remote-storage, symlink, corrupted-record,
oversized-record, Explain-storage-warning, and top-level JSON cases affected
by the repair. Record commands, counts, durations, toolchain, and hashes.

Do not run Docker, install, `demo.sh`, unrestricted all-language discovery, or
the full Rust workspace in this repair lane. Those gates are
manager-serialized after accepted candidates are composed.

## Handoff

Commit only intended product, focused tests, and canonical source documents;
leave the worktree clean. Report:

- final commit and parent/task revision;
- exact changed and intentionally untouched files;
- F1 through F9 closure evidence;
- independent-oracle and candidate-failure evidence;
- exact formulas, limits, owner peaks, curves, commands, counts, durations,
  and hashes;
- zero-debt/zero-permit terminal evidence;
- unresolved cross-command handoffs, if any;
- conflicts expected when porting the repaired range;
- a request for fresh independent adversarial review of the exact commit.

This lane does not authorize integration, issue Close, Install, Dogfood, or
Retrospect.
