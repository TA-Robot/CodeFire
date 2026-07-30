# CFR-168 Rust HTTP MR Preflight Review-Closure Repair

Repair the exact Rust HTTP merge-request create/review candidate
`71854e20689d2dd79246fa741e0e6da443d04d1a` against every finding and residual
risk in the independent review originally committed as
`b1905e9383e0201b841a6efd94578f46ffa98808`:

`docs/development/ai/swarm-results/cfr168-rust-http-mr-preflight-review-claude-20260730/review.md`

The report SHA-256 is
`f30c22aa34559829254b6ca305449abc874b653c8b3dbc767248fdbbc5fe0a5a`.
The report is present on this exact-candidate taskbase as additive commit
`185b7b7`. Use Claude Opus 5 (`claude-opus-5`). Work only in the assigned
worktree and write set. Add new commits; do not amend, rewrite, merge, or
replace the reviewed candidate or report.

The verdict is `REQUEST_CHANGES`. The required F1/F2 evidence fixes, F3/F4/F5
hardening, all four named residual risks, and every newly discovered route-
local residual are required closure work. Preserve the accepted product
properties: response pairs are prepared before mutation, exact wire bytes and
replay remain stable, the selected response buffer is published without a
second encoding, one request ledger owns the response work, and neighboring
upload/apply/list routes remain unchanged.

## Exact ancestry and scope

- Original task/baseline revision:
  `3c23e381f871724d31332bd179671db7cbb9d941`
- Reviewed product candidate:
  `71854e20689d2dd79246fa741e0e6da443d04d1a`
- Independent report:
  `b1905e9383e0201b841a6efd94578f46ffa98808`
- Candidate-rooted report copy:
  `185b7b7`
- Independent report SHA-256:
  `f30c22aa34559829254b6ca305449abc874b653c8b3dbc767248fdbbc5fe0a5a`

The final repair must remain an auditable additive successor of the frozen
task revision. This lane does not authorize canonical integration, issue
closure, installation, dogfood, or retrospective completion.

## Required closure work

### F1 — make the first-effect ordering evidence observable

1. Change the merge-create fixture so the target does not already contain the
   complete source closure. Use distinct source/target heads or omit at least
   one exact source object from the target.
2. Extend the authority snapshot with the complete target object byte-set and
   explicit directory state. Snapshot traversal errors must fail the test
   rather than silently return a partial snapshot.
3. For every failure that is specified as pre-effect, assert byte-identical
   objects, branches, generations, merge requests, nonce state, transaction/
   pending/recovery/receipt/audit state, directories, and response state.
4. The focused suite must fail if
   `MergeRequestResponsePreflight::prepare` is moved after
   `source_records.publish_with_work_and_poll`. Record that exact reversal
   against the reviewed candidate.
5. Resolve the publication-fence semantics explicitly:
   - if the fence promises no object effect, place it before the escrow;
   - if escrow is an accepted durable stage, assert and document only the
     exact post-escrow authority guarantee and prove exact retry/recovery.
   Do not label a post-escrow snapshot byte-identical when object bytes have
   already changed.

### F2 — prove exact residual ownership instead of discarding it

1. Assert merge-create success residual is exactly zero after the prepared
   body owner is released, matching merge-review.
2. Replace the stale `assert_no_open_work` comment with the actual invariant.
3. For each error path that intentionally hands branch-head frame owners to
   request termination, assert their named formula and then prove the outer
   publisher reaches zero current, zero open permits, and exactly one terminal
   reason.
4. A blanket terminal sweep must not be used as evidence that an unidentified
   route owner is legitimate. Name every transferred owner or release it
   before handoff.

### F3 — make the second response owner fail closed

1. `MergeRequestResponsePreflight::seal` must reject a second or externally
   completed prepublication seal, matching the incumbent transaction owner.
2. `Drop` must safely cancel/release every remaining response permit and frozen
   body in release builds. A debug-only assertion is insufficient.
3. Double select, double cancel, partial select, external seal, early return,
   and drop after each state transition must end with zero debt and zero open
   permits without replacing the primary outcome.
4. Update the corresponding `work_budget.rs` invariant so it names every
   authorized typed sealer and forbids simultaneous owners.
5. Prefer one shared prepared-response owner/state machine when that removes
   duplicated weaker lifecycle logic without changing route wire behavior.

### F4 — complete independent response boundaries

1. Add merge-review exact, exact-minus-one, and exact-plus-one cases for
   `response_bytes` and `response_header_bytes`, in addition to retained and
   temporary ownership.
2. Separate success-body and durable-retry-body evidence. The larger
   149-byte durable body must not make a success-body-only regression
   invisible.
3. Prove each boundary reaches the intended response-preflight owner rather
   than an earlier unrelated limit.
4. Cover create and review encoding failure, deadline, disconnect, replay, and
   cleanup paths with exact response/header/body bytes and full snapshots.

### F5 — remove stale staging/dead-code claims

1. Remove stale `#[allow(dead_code)]` attributes from production-wired response
   preparation APIs.
2. Correct the comment that still calls the API staged before production
   wiring.
3. Either production-wire the unused owner through the common lifecycle or
   delete it and its test-only surface. Do not retain two nearly identical
   owners without an exact consumer/invariant.
4. Strict Clippy must pass without adding replacement suppression attributes.

## Required residual-risk closure

### R1 — post-commit release failure cannot select a new error response

The review identified post-closure `release` calls in merge-review and a
destination ownership release in merge-create that can fail after durable
publication. Repair the ordering/state machine so:

- all fallible accounting releases that can be completed precommit are
  completed before the first authoritative effect;
- any cleanup failure after commit preserves the already frozen success or
  durable-retry response and records the supported terminal condition;
- one HTTP document is written, never a committed success followed by a newly
  selected error document;
- injected release failure before and after commit proves exact authority,
  response bytes, zero debt, zero permits, and one terminal observation.

### R2 — durable fallback survives a real partial selection failure

1. Make prepared-response selection transactional: either the selected body's
   exact permits/owners transfer completely or the pair remains valid for
   cancel/fallback.
2. Add deterministic failure points after each internal selection transition,
   including after response/header permit consumption.
3. Prove a partially failed success selection can still choose the frozen
   durable-retry response without a missing-permit error or half-consumed pair.
4. Double-failure cleanup must preserve the primary outcome and end at zero.

### R3 — snapshots include directory effects and fail closed

1. Record files, directories, symlink kinds, and exact bytes/metadata needed by
   the contract.
2. Any `read_dir`, metadata, or file-read error must fail the snapshot rather
   than silently omit a subtree.
3. Add created/removed directory, unreadable-subtree, and symlink fixtures so
   pre-effect assertions are non-vacuous.

### R4 — deadline phases have typed, unique identity

1. Replace test injection keyed only by repeated string values with a typed
   phase identity or an independently validated uniqueness table.
2. Preserve the public diagnostic phase strings.
3. Prove each create/review phase can be selected independently and that adding
   a duplicate identity fails a regression check.

## Compatibility and invariants

- Preserve authentication, signature, nonce, escrow, review materialization,
  replay, error precedence, post-commit retry, and cross-runtime semantics.
- Preserve exact raw HTTP success and durable-retry bytes, including the
  149-byte durable body and its 503 status.
- No route may encode the selected response twice or reserve the same response
  bytes twice.
- No public profile limit may be raised and no prepublication/deadline/
  integrity check may be weakened.
- Neighboring upload, apply, list, and shared encoder behavior must remain
  byte-identical when the repaired boundary is not crossed.
- Success and every failure must finish with zero retained bytes, zero
  temporary bytes, zero open permits, and exactly one terminal observation.

## Required evidence

Provide:

- exact create/review call chains and response-owner state transitions;
- retained/temporary/response/header formula tables and simultaneous peaks;
- exact/minus-one/plus-one observations for every charged dimension;
- target object, directory, authority, transaction, response, and terminal
  snapshots before/after every injected failure;
- deliberate-regression results against exact candidate
  `71854e20689d2dd79246fa741e0e6da443d04d1a`;
- exact 1x/3x/10x counter curves where work scales with object or response
  count;
- full lists of changed and intentionally untouched routes/files.

## Authorized validation ladder

Use the shared constrained Rust toolchain and target:

```bash
export RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup
export CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=2
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared
```

Run narrow source checks and individual new tests first. Create a durable WIP
commit before long Cargo gates. Near handoff, serialize:

```bash
cargo fmt --all -- --check
cargo test -p codefire-cli --bin codefire-rs --locked merge_request -- --nocapture
cargo test -p codefire-cli --bin codefire-rs --locked http::tests -- --nocapture
cargo test -p codefire-cli --bin codefire-rs --locked
cargo clippy -p codefire-cli --bin codefire-rs --locked -- -D warnings
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Also run the exact signature/nonce/replay/escrow/branch-protection, publication
failure, apply/upload neighbor, and cross-runtime cases named in the report.
Do not run Docker, install, dogfood, unrestricted all-language discovery, or
the full Rust workspace in this repair lane. The manager serializes those
after accepted candidates are composed.

## Handoff

Commit only intended product, focused tests, and canonical source documents;
leave the worktree clean. Report:

- final commit and exact parent/task revision;
- F1 through F5 and R1 through R4 closure evidence;
- formulas, peaks, boundaries, state transitions, and snapshot hashes;
- deliberate-regression commands/results and independent-oracle results;
- commands, counts, durations, toolchain, and exact hashes;
- any exact predecessor-gated residual with symbol/owner/revision;
- expected conflicts with active `http.rs` or `work_budget.rs` ranges;
- a request for fresh independent review of the exact repaired commit.

Do not claim complete closure solely from focused test success. Acceptance
requires the fresh review and manager composition gates.
