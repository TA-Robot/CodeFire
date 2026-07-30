# CFR-168 Rust File Ancestry Zero-Residual Fugu Rereview

Independently review the exact Rust file-remote ancestry repair candidate with Sakana Fugu
Ultra 1.1 (`fugu-ultra-v1.1`). This is a report-only lane. Do not modify product code, tests,
source-of-truth docs, issue state, management ledgers, or the candidate commit.

The previous independent review returned `REQUEST_CHANGES`. Under the active zero-residual
policy, this rereview may return `ACCEPT` only if every prior Must, Should, and Could finding is
closed on the exact candidate, every required check is independently reproduced, the evidence is
non-vacuous, and no new unresolved finding remains on the candidate's declared surface.

## Exact subject and provenance

| Role | Exact value |
|---|---|
| Rejected predecessor candidate | `e93ebc2394c8bd0398cfc23e8bf2d23d52e253b9` |
| Prior review commit | `0db912d58bc9c013dfccfaa175056dc463b177b0` |
| Canonical cherry-pick of the same review | `30bbb4609fd646ed193a69c12f906b7cb4073e9e` |
| Prior review content SHA-256 | `64a4831ea0a639d7f87fe0b08312677efcab8bfe014f0d5ba72806e9caceecf1` |
| Repair task/base revision | `86a0c2e42c15bb44df8a2ff299140fedbcff4aae` |
| Repair task content SHA-256 | `4385f51b061be3bc608e170c1f979e0fdeaac73215196378540db556c32b92b8` |
| Exact repair candidate | `02b0d63651703aaac88c249bdc437cd173dfa397` |
| Candidate parent | `86a0c2e42c15bb44df8a2ff299140fedbcff4aae` |
| Candidate tree | `51a683def1ec666d9baff802cbf898392bbd5c99` |
| Candidate patch SHA-256 | `9cf5bb60a6563eb111fb9a352fda2bb44334457ac48844465e3ef1197c8f9662` |
| Candidate branch | `claude-agent/cfr168-rust-file-ancestry-zero-residual-repair-r1-20260730-file-ancestry-zero-residual-repair` |

The patch digest above is the SHA-256 of:

```bash
git diff --binary --full-index \
  86a0c2e42c15bb44df8a2ff299140fedbcff4aae..\
02b0d63651703aaac88c249bdc437cd173dfa397
```

The candidate is exactly one commit on the repair base. Its cumulative changed-file set is:

```text
crates/codefire-cli/src/remote.rs
crates/codefire-core/src/work_budget.rs
docs/development/change-playbook.md
docs/development/command-source-trace.md
docs/development/source-architecture.md
docs/development/source-data-model-catalog.md
```

The recorded numstat is:

```text
2696  813  crates/codefire-cli/src/remote.rs
83      0  crates/codefire-core/src/work_budget.rs
42      0  docs/development/change-playbook.md
2       2  docs/development/command-source-trace.md
57      0  docs/development/source-architecture.md
20      0  docs/development/source-data-model-catalog.md
```

At packet preparation time both the implementation worktree and the candidate commit were clean,
and `git show --check 02b0d636...` passed. Reconfirm all identity and cleanliness facts rather than
assuming they remain true.

## Required reading

Read these before reaching a decision:

1. `AGENTS.md`
2. `docs/development/README.md`
3. `docs/development/project-management.md`
4. `docs/development/agent-code-quality.md`
5. `docs/development/agent-workflow.md`
6. `docs/development/ai/README.md`
7. `docs/development/ai/operating-protocol.md`
8. `docs/development/ai/provider-routing-policy.md`
9. `docs/development/ai/swarm-results/cfr168-rust-file-ancestry-review-claude-fallback-20260730/review.md`
10. `docs/development/ai/swarm-batches/cfr168-rust-file-ancestry-zero-residual-repair-20260730.md`
11. the complete exact diff `86a0c2e...02b0d636`
12. all six changed files at the exact candidate

The implementation handoff reported focused `21/21`, remote `129/129`, full-bin `789/789`, core
`91/91`, strict Clippy, format, source-doc, and diff checks. It also reported several deliberate
regression experiments. Those are implementer claims, not reviewer evidence. The Git commit contains
the implementation, tests, source docs, and detailed closure claims, but no independently trusted
validation artifact. Rerun and falsify the claims yourself.

## Scope and independence

Allowed write:

- `docs/development/ai/swarm-results/cfr168-rust-file-ancestry-zero-residual-rereview-fugu-20260730/review.md`

Disallowed writes:

- every product, test, source-of-truth, issue, strategy, and management path;
- the prior review and repair packet;
- any other report path.

Use temporary archives or scratch directories outside the Git worktree for deliberate regression
experiments. Leave no tracked scratch edit. Do not amend, rebase, merge, or cherry-pick the
candidate. Do not infer acceptance from a clean commit, test count, prior agent completion, or the
presence of comments claiming an invariant.

## Closure matrix for every prior finding

The report must give each row an explicit `CLOSED` or `OPEN` decision, with concrete source/test
locations and independently observed evidence. Any `OPEN` row requires `REQUEST_CHANGES`.

### `MUST-1` — no rejected operation may retain a pre-decision object effect

Verify all of the following:

- Upload publication takes a genuine byte-for-byte pre-command snapshot of the complete remote
  surface, including the object store.
- `stage_object_graph_copy`, `StagedRemoteObjectGraph`, `settle_staged_object_graph`, and
  `FileRemoteOperation::close` remember exactly the records created by this operation and never
  rewrite or delete a pre-existing record.
- Newly created records are kept only when the primary outcome is accepted and the exact final
  subject fence passed. Non-fast-forward, budget, deadline/cancellation, subject replacement,
  encoding/preflight, copy, publication, cleanup, and replay failures leave objects, branches,
  generations, merge requests, idempotency receipts, and publication authority byte-identical to
  the genuine pre-call state.
- A partial object-copy failure cannot create a record before its ownership is durably represented
  for cleanup. Check error returns, process interruption boundaries, and visibility to concurrent
  readers; do not accept the word “staged” if files are already visible in the live object store
  without a recovery authority.
- The order around branch/project gates, object staging, stable-project identity, digest and plan
  allocation, replay lookup, ancestry, final revalidation, generation reads, publication
  preflight, publication, and close has no fallible post-effect gap that can violate the claimed
  byte-identical result.
- The former `include_objects` escape hatch and the already-mutated reference comparison are gone.

Credit the repair only if the committed tests would fail when the staged effect is made
unconditional, moved ahead of its guard, retained after a rejected result, or allowed to lose its
created-record ownership.

### `MUST-2` — one operation-scoped ledger and one snapshot-bound session

Derive the live call graph for:

1. file upload dry-run;
2. file upload publication;
3. file merge-request apply.

For each route prove:

- exactly one `FileRemoteOperation` opens before that route's first selected sealed validation;
- exactly one `DecisionWorkLedger` identity and one
  `SealedCommitValidationSession` own validation, pinning, record collection where applicable,
  signature-policy ancestry work, goal-directed ancestry, final raw-document revalidation,
  staged object work, publication handoff, terminal selection, and cleanup;
- both apply heads are validated with the same pinned session where the contract requires it;
- no route reaches an unaccounted `validate_sealed_commit`, `collect_object_records`,
  `copy_object_graph`, legacy signature-history walk, or equivalent hidden graph traversal;
- early failures during `begin`, session bind, proof conversion, replay, plan preparation,
  publication, and close still emit one terminal observation and release every owner/permit;
- corruption and legacy error precedence remain exact rather than being delayed merely to simplify
  ledger composition.

The `unaccounted_object_graph_passes` counter is evidence only if it is complete, production-path
reachable, and itself shown non-vacuous. Search independently for equivalent traversals that bypass
the named wrappers.

### `SHOULD-1` — the deadline/cancellation contract is real in a shipped build

Verify:

- `DecisionWorkProfile::FILE_REMOTE_OPERATION_DEADLINE_MS` is the sole authoritative file-remote
  limit and the public limit was not raised;
- production uses a monotonic clock and `elapsed >= limit`;
- only elapsed time/the clock is injected by tests, not the production outcome;
- diagnostics report the same authoritative nonzero limit;
- `local_v1().request_deadline_ms` and unrelated commands retain their documented behavior;
- every required stage is fenced and every potentially long pass polls from inside its loop,
  including sealed validation, object collection, signature policy, ancestry traversal, staged
  copy, final revalidation, and any publication work the source docs claim is covered;
- a timeout after an effect cannot return a false no-effect result, and removing each production
  poll makes a focused test fail for the intended reason.

Explicitly adjudicate the candidate's omission of `ObjectEffect` from `UPLOAD_STAGES`, the limited
in-pass poll test that names collection/traversal, and any unpolled policy, copy, response, replay,
or publication work. A test-only stage jump is not proof that wall-clock expiration inside the pass
is wired.

### `SHOULD-2` — fixture classes and every charged dimension

Independently confirm deterministic exact counters and typed failure behavior for:

- chains;
- a shared diamond with one visit per unique node and deterministic edge/revisit counts;
- a missing or dangling parent;
- a corrupt record;
- content-addressed cycle impossibility or a valid synthetic cycle oracle.

For all three production routes cover exact, exact minus one, and exact plus one for:

- `ObjectRecords`;
- `AggregateBytes`;
- `GraphNodes`;
- `GraphEdges`;
- `SnapshotRevalidationRecords`;
- `SnapshotRevalidationBytes`;
- `RetainedBytes`;
- `TemporaryBytes`;
- `LogicalDepth`;
- `MaximumFrontier`.

Re-derive the meaning and off-by-one convention for depth/frontier. Do not accept a limit-lowering
helper unless it can only narrow a published profile and the live route consumes that narrowed
limit.

### `SHOULD-3` — scale, production reachability, and bounded retained memory

Verify through actual upload dry-run, upload publication, and merge-request apply callers:

- exact 1x/3x/10x counter curves;
- a 600-deep history without recursion or duplicate visits;
- early goal stop without a second full-distance materialization;
- total work, not only a selectively reported query phase;
- concurrent retained peak against the legacy distance-map baseline;
- the documented affine upper `128 KiB + 6 KiB * n` and the claimed `40x` legacy multiple;
- no uncharged full-document owner, duplicate payload, sort scratch, path owner, or diagnostic/plan
  projection makes the measured peak incomplete;
- stable complexity and ordering for shared DAGs, missing/corrupt subjects, and unsuccessful
  ancestry queries.

The three prior helper-level counter tests survived restoration of the old production call sites.
The replacement evidence must fail when production routing is reverted. Report exact measured
curves and explain what each counter includes.

### `SHOULD-4` — effect-bearing failure injection

Exercise all three routes independently. Cover every relevant stage and at least:

- exact budget rejection;
- wall-clock/deadline cancellation before and inside long passes;
- same-path, same-inode, same-size, same-mtime content replacement;
- session discard and proof-conversion failures;
- partial staged-copy failure;
- publication failure;
- non-fast-forward rejection;
- replay and plan/encoding failure where reachable;
- cleanup failure after a primary typed outcome.

Each case must prove the exact diagnostic/exit contract, one observation, zero current debt, zero
open permits, correct terminal reason, and byte-identical semantic remote state. Demonstrate the
fault location was actually reached.

### `COULD-1` — ledger-loss paths terminalize and remain observable

Audit every `FileRemoteOperationState` transition, including `Validating`, `Pinned`, `Released`,
`Closed`, and `Lost`. Session-discard failure, proof-conversion failure, retry failure, double
cancellation, observer failure, and publication failure must:

- terminalize before any irreversible ledger handoff;
- emit exactly one observation;
- retain enough mirrored evidence to show the terminal reason;
- close all recoverable ownership;
- never silently drop an unfinished ledger.

`force_lost_command_ledger` is not sufficient alone. Prove that tests reach the same production
state transitions or add a finding explaining why the injected path is only a synthetic imitation.

### `COULD-2` — the primary typed outcome is never masked

Independently combine a primary budget, deadline, subject-change, usage, or store outcome with
cleanup, release, residual-ownership, ledger-finish, and observation failures. The caller must
receive the primary typed outcome, while cleanup debt remains observable and retryable where the
owner contract permits it. A scalar cleared before a fallible release, `mem::forget`, or an
unretryable cleanup owner is not zero-residual closure.

## Operation lifecycle and ordering audit

Produce a compact state/ownership table from route entry through close. At minimum bind:

- ledger identity;
- validation session/proof state;
- pinned roots and raw-document source;
- object-record owner;
- signature-policy owner;
- ancestry subject/frontier/seen owners;
- staged-record owner and recovery authority;
- final-fence state;
- publication result;
- cleanup order;
- terminal reason and observation count.

Check reservation-before-allocation/read/growth for every retained collection, raw document,
frontier/seen insertion, string/path clone, signature-policy input, plan/diagnostic/response
projection, staged-path list, and cleanup/recovery record. Distinguish work intentionally left to a
predecessor-gated successor from work falsely claimed to be inside this operation.

## Source truth and compatibility

Review all four changed canonical source docs against the implementation:

- `docs/development/source-architecture.md`
- `docs/development/command-source-trace.md`
- `docs/development/source-data-model-catalog.md`
- `docs/development/change-playbook.md`

In particular decide whether these claims are literally true:

- “whole command” versus the exact point where `FileRemoteOperation` begins;
- one ledger/session covers the listed validation, policy, ancestry, effect, publication, and
  cleanup work;
- every rejected operation leaves the complete remote byte-identical;
- the production deadline is polled on every claimed pass;
- no unaccounted validate/collect/copy pass remains on the three routes;
- `is_ancestor_in_objects` is intentionally test-only and no production/source doc still promises
  it as a callable compatibility surface;
- `local_v1` and advertised compatibility profiles remain unchanged;
- adjacent residuals are named as open rather than implied closed.

Any source-of-truth overclaim is a finding under the zero-residual policy even if the code's narrower
behavior is otherwise acceptable.

## Predecessor-gated residual handoff audit

The report must classify every row below as:

- `candidate-closed`;
- `valid predecessor-gated handoff`, with exact symbol/API and successor prerequisite;
- `candidate-surface blocker`;
- `new unowned residual`.

| Residual surface | Required handoff/predecessor |
|---|---|
| local `ancestor_distances` / `nearest_common_ancestor_from_right` | Rust storage releases the overlapping `main.rs` write set |
| file clone graph walk | accepted reusable file-ancestry operation plus storage integration |
| storage-report graph walk | accepted reusable file-ancestry operation plus storage integration |
| file GC graph walk | accepted reusable file-ancestry operation plus storage integration |
| local `cf scan` validation | Rust storage releases overlapping `main.rs`/metrics work |
| file transport branch/generation/plan authority reads and response encoding | an explicit successor using the reusable operation/accounted authority APIs |
| HTTP client leg of upload | the HTTP request-ledger/server composition successor |
| `remote::publication::validate_object_roots` | a publication-composition successor that preserves preflight ordering |

Confirm that the reusable API list in `source-architecture.md` exists with compatible visibility and
ownership semantics:

```text
FileRemoteOperation::{
  begin,
  validate_sealed_subject,
  pin_subject,
  collect_object_records,
  enforce_commit_signature_policy,
  is_fast_forward,
  revalidate_and_fence,
  stage_object_graph_copy,
  close
}
collect_object_records_with_owned_work_and_poll
is_ancestor_in_provisional_subject_with_poll
signatures::enforce_remote_commit_signature_policy_in_provisional_subject_with_poll
```

A residual may be handed off only when it is outside this packet's frozen surface, explicitly
tracked, and the candidate does not use it to support a zero-residual claim. If a live route already
executes the residual inside the claimed operation, treat it as a candidate blocker rather than
deferring it by name.

## Required deliberate-regression evidence

In disposable copies of the exact candidate, make focused tests fail non-vacuously when you:

1. restore the predecessor's unaccounted sealed-validation call site;
2. bypass operation-ledger composition for object collection;
3. bypass operation-ledger composition for signature policy;
4. commit staged objects regardless of outcome/fence;
5. remove or move the final raw-document fence;
6. remove each required in-pass cancellation poll;
7. make the production deadline comparison unable to expire at the exact limit;
8. let cleanup/residual failure replace a primary typed outcome;
9. weaken complete-remote snapshots so object changes are excluded;
10. route the 1x/3x/10x evidence through a helper instead of a production caller.

Record the exact scratch change, exact command, expected failure, observed failure, and why it proves
the intended guard rather than an unrelated compile/test break. If an experiment cannot be made
non-vacuous, return `REQUEST_CHANGES` and state the missing oracle.

## Independent validation

Run without competing Cargo builds:

```bash
export RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup
export CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=2
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared

cargo test -p codefire-cli --bin codefire-rs \
  remote::file_remote_ancestry_tests --locked
cargo test -p codefire-cli --bin codefire-rs remote --locked
cargo test -p codefire-cli --bin codefire-rs --locked
cargo test -p codefire-core --locked
cargo clippy -p codefire-cli --all-targets --locked -- -D warnings
cargo fmt --all -- --check
python3 docs/development/tools/check_source_docs.py
git diff --check \
  86a0c2e42c15bb44df8a2ff299140fedbcff4aae..\
02b0d63651703aaac88c249bdc437cd173dfa397
git show --check 02b0d63651703aaac88c249bdc437cd173dfa397
```

Also run focused legacy file upload/apply/signature/replay/publication cases selected from the exact
call graph. Record exact counts, durations, toolchain versions, and every failure or unavailable
gate. A rerun that changes without an intended source/environment change is unresolved flaky
evidence, not a pass.

## Decision rule and output

Write exactly:

`docs/development/ai/swarm-results/cfr168-rust-file-ancestry-zero-residual-rereview-fugu-20260730/review.md`

The report must include:

1. reviewer/model/date and independence statement;
2. exact base, candidate, parent, tree, patch digest, changed-file set, and clean-state result;
3. `ACCEPT` or `REQUEST_CHANGES`;
4. severity-ordered new findings with concrete file/line and executable evidence;
5. an explicit closure table for `MUST-1`, `MUST-2`, `SHOULD-1..4`, and `COULD-1..2`;
6. route-by-route operation/session/owner lifecycle;
7. exact/boundary/scale/cancellation/cleanup and deliberate-regression results;
8. source-truth and compatibility verdicts;
9. predecessor residual handoff table with exact symbols and prerequisites;
10. commands, results, counts, durations, and environment limitations;
11. residual risk and exact next action.

`ACCEPT` is valid only when all eight prior rows are closed, required evidence is complete and
non-vacuous, source docs are exact, and no newly discovered unresolved finding remains on the
candidate's surface. Otherwise return `REQUEST_CHANGES`; do not soften a missing oracle into an
approval note.

Commit only the report and leave the reviewer worktree clean.
