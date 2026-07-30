# CFR-168 Rust HTTP MR Preflight Zero-Residual Independent Rereview

Use Claude Opus 5 (`claude-opus-5`) as an independent reviewer. Review only:
do not modify product code, tests, source contracts, issue state, or manager
ledgers. Commit exactly one immutable review report.

## Exact subject

- original reviewed product candidate:
  `71854e20689d2dd79246fa741e0e6da443d04d1a`
- repair task/report base:
  `7e4e685efbfc453a6d2209f37c99edb1ab46243d`
- exact repaired candidate:
  `73bc5327ff6d750ce95f63333055ee39b693cb8c`
- candidate parent:
  `7e4e685efbfc453a6d2209f37c99edb1ab46243d`
- candidate tree:
  `430f2e2ccf9dca485575e7f9db0b639390d7e761`
- binary patch SHA-256:
  `216c12f8d9e0c6939bf6889856a999e8f45b56237eca9ff63cdef34105ef268d`
- prior independent-review report SHA-256:
  `f30c22aa34559829254b6ca305449abc874b653c8b3dbc767248fdbbc5fe0a5a`
- repair-contract SHA-256:
  `b1c8e65a2af80a776c58e550d24272753698fb337464729ee49880804ef95e1e`

Changed paths are exactly:

```text
crates/codefire-cli/src/http.rs
crates/codefire-core/src/work_budget.rs
docs/development/change-playbook.md
docs/development/command-source-trace.md
docs/development/source-architecture.md
docs/development/source-data-model-catalog.md
```

First recompute every identity, clean state, ancestry, changed path, and patch
digest. A mismatch is `REQUEST_CHANGES`; do not review a moving substitute.

The implementation lane produced temporary WIP commits and then delivered the
single exact additive candidate above. That final tree, parent, patch, and
evidence must be reviewed independently. Record the provenance/process
deviation from the repair contract's no-rewrite instruction, reconcile whether
any evidence or intended product bytes were lost, and do not treat a clean
single-commit history as proof that the process contract was followed.

## Required independent falsification

Re-audit every finding and residual in:

- `docs/development/ai/swarm-results/cfr168-rust-http-mr-preflight-review-claude-20260730/review.md`
- `docs/development/ai/swarm-batches/cfr168-rust-http-mr-preflight-review-closure-repair-20260730.md`

### F1 through F5

1. **F1 observable first-effect ordering:** use a non-vacuous source/target
   closure, complete fail-closed object/directory/authority snapshots, and an
   actual reversal that moves response preparation after escrow/publication.
   Distinguish the pre-escrow no-effect guarantee from the post-escrow
   publication-fence guarantee.
2. **F2 exact residual ownership:** prove create success ends at zero after the
   selected body owner is released. Every intentional handoff must have a
   named formula and the request publisher must close at zero current, zero
   open permits, and one terminal reason. A blanket sweep is not evidence of
   ownership.
3. **F3 fail-closed response owner:** independently exercise external/double
   seal, double/partial select, partial-transfer failure, cancel, early return,
   unwind/drop after every state, and primary-error preservation. Release
   builds must close both response bodies and permits.
4. **F4 independent response boundaries:** create and review must each prove
   exact, minus-one, and plus-one response/header/retained/temporary
   boundaries. Separate success-body evidence from the larger durable-retry
   body and prove each failure reaches the intended owner.
5. **F5 dead-code and owner cleanup:** strict Clippy must reject replacement
   suppressions; production-wired APIs and source contracts must not claim a
   staged or duplicate owner.

### R1 through R4

1. **R1 postcommit cleanup:** a release failure cannot replace a committed
   response. Inject before and after the first effect and prove exact authority,
   one byte-exact document, primary outcome preservation, zero debt/permits,
   and one terminal observation.
2. **R2 transactional fallback:** fail after each internal response/header/body
   transfer. Either selection commits completely or the original pair remains
   valid for cancel or durable fallback. Double failure must preserve the first
   outcome and close at zero.
3. **R3 directory-complete snapshots:** include files, directories, symlink
   kinds, exact bytes, and required metadata. `read_dir`, metadata, and
   file-read failures must fail closed. Deliberately create/remove/change a
   directory, symlink, and unreadable subject after the last observation.
4. **R4 typed unique deadline phases:** every create/review phase must be
   selected independently, public strings must remain stable, and a duplicate
   typed identity must fail the uniqueness oracle.

For every asserted oracle, inspect or run a deliberate regression that breaks
the exact property while preserving nearby helper calls. Reject deleted-helper,
constant-label, helper-counter, test-only stage-jump, or incomplete-snapshot
mutations as sufficient proof. Trace the live create/review call graph through
the production HTTP dispatcher.

## Additional product audit

Inspect the 2,763-insertion/560-deletion candidate for:

- a second ledger, response owner, sealer, cleanup owner, or terminalizer;
- hidden/unreserved allocation and unbounded retained state;
- permit consumption that can partially commit;
- panic/abort behavior reachable from malformed or resource-limited input;
- response-after-effect, replay, nonce, escrow, signature, authentication, and
  error-precedence drift;
- changed upload/apply/list/bundle/doctor behavior or wire bytes;
- source-document overclaim and test-only behavior represented as production;
- duplicate typed phases, unreachable compatibility code, and suppressed
  warnings.

Reconcile the implementation's claimed 126-byte create success, 57-byte review
success, and 149-byte/503 durable-retry response against actual encoded bytes.
Audit simultaneous peak formulas and 1x/3x/10x scaling, not only cumulative
charges.

## Validation boundary

Run the smallest independent focused probes first. Then, when the exact source
tree and shared target are clean, run the authorized Rust CLI/core/Clippy/source
document gates. Use:

```bash
export RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup
export CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=2
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared
```

Record exact commands, counts, durations, toolchain, target path, and any
unavailable evidence. Passing worker-reported gates are context, not
independent evidence. Do not modify the candidate to make a probe pass. Do not
run Docker, install, dogfood, or unrestricted all-language discovery.

## Decision and output

Write exactly:

`docs/development/ai/swarm-results/cfr168-rust-http-mr-preflight-zero-residual-rereview-claude-20260730/review.md`

The report must include:

- recomputed exact subject and clean-state evidence;
- one verdict for every F1-F5 and R1-R4 item;
- production reachability and deliberate-regression evidence;
- severity-ordered findings with concrete file/line and reproducer;
- command results, counts, durations, and unavailable evidence;
- the WIP-rewrite provenance reconciliation;
- exact integration/write-overlap map;
- `ACCEPT` only if every required item is closed and no material unowned
  finding remains in scope; otherwise `REQUEST_CHANGES`.

Commit only the report and leave the reviewer worktree clean.
