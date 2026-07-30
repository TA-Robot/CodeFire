# CFR-168 Rust Runtime Composition Freeze Task Packet

Status: blocked

Role: manager only

Frozen canonical base:
`740ea18eb42630ba265a8aad83c950ceb96aacca`

Typed packet:
[`json/cfr-168-rust-runtime-composition-freeze-20260730.json`](json/cfr-168-rust-runtime-composition-freeze-20260730.json)

This packet freezes the only authorized composition order and the evidence
required before CodeFire's four Rust runtime candidates can be combined. It
does not authorize a worker launch, a `dev-swarm.v1` manifest, a product
change, or a canonical integration while its status is `blocked`.

## Objective

After every input lane has an exact final candidate and an independent report
whose decision is `ACCEPT`, compose the accepted Rust runtime changes on a
clean successor of canonical
`740ea18eb42630ba265a8aad83c950ceb96aacca` in this order:

1. file-remote ancestry;
2. Rust HTTP merge-request response closure;
3. Rust storage;
4. file-remote Doctor;
5. one shared synthesis of the four canonical source documents.

The manager must reconcile the shared `work_budget.rs` semantics, preserve one
ledger/sealer/terminal owner per operation, run the focused composition gate
ladder, and then request a new independent review of the exact composed commit
and tree. Candidate-local acceptance does not imply composed acceptance.

## Launch Blockers

All four rows must identify an exact final product commit and an exact
independent `ACCEPT` report commit plus report SHA-256 before this packet can
move from `blocked` to `ready`.

| Order | Surface | Frozen subject or repair base | Current decision state | Evidence still required |
|---:|---|---|---|---|
| 1 | File-remote ancestry | Candidate `02b0d63651703aaac88c249bdc437cd173dfa397` | Prior candidate review `0db912d58bc9c013dfccfaa175056dc463b177b0` requested changes; repaired candidate has no bound acceptance at this base. | Exact report commit, report SHA-256, literal `ACCEPT`, and confirmation that the report reviewed exactly `02b0d63…`. |
| 2 | Rust HTTP merge-request closure | Rejected candidate `71854e20689d2dd79246fa741e0e6da443d04d1a`; closure task base `7e4e685efbfc453a6d2209f37c99edb1ab46243d` | Review `b1905e9383e0201b841a6efd94578f46ffa98808` requested changes. The closure lane's final product commit is not yet known. | Final repair commit and parent/range, then an exact independent report commit, report SHA-256, and literal `ACCEPT`. |
| 3 | Rust storage | Candidate `cdcb9bdefe7d25a118a9dc14171864d75f5ad304` | Prior report `4cfb607cfd6ac95d26d24a4542ac15ba56b428d6` requested changes; repaired candidate has no bound acceptance at this base. | Exact report commit, report SHA-256, literal `ACCEPT`, and confirmation that the report reviewed exactly `cdcb9bd…`. |
| 4 | File-remote Doctor | Accepted predecessor `52a025ecda3fe2f5ad8b5b2a485562e7c16ab6de`; zero-residual task base `dbeaac1526acc8c2e102ee2b137e8f2a708890b0` | Review `ac725ae7a2bf3e419384ea599fc2f273130caf50` accepted the predecessor routing decision, but the required zero-residual repair's final product commit is not yet known. | Final repair commit and parent/range, then a fresh exact independent report commit, report SHA-256, and literal `ACCEPT`. |

The packet remains blocked if any hash is absent, abbreviated in the
authoritative evidence, points to the wrong parent/range, or has only worker
success without an independent decision.

If any new report returns `REQUEST_CHANGES`, do not salvage selected hunks,
reinterpret the decision, or continue composition. Produce a successor from
that exact rejected candidate, close every finding, obtain a fresh exact
review, and update both packet files with the new immutable hashes.

## Exact Inputs

When every blocker is cleared, the manager ports implementation history only.
Review, task, result, manifest, and recovery-control commits are evidence and
must not be merged as product history merely because they are ancestors of a
candidate.

### 1. File-remote ancestry

Port, in order:

1. `e93ebc2394c8bd0398cfc23e8bf2d23d52e253b9`;
2. `02b0d63651703aaac88c249bdc437cd173dfa397`.

Expected implementation surfaces:

- `crates/codefire-cli/src/remote.rs`;
- `crates/codefire-core/src/work_budget.rs`.

Defer this lane's canonical source-document changes to the single synthesis
step.

### 2. Rust HTTP merge-request closure

Port:

1. `71854e20689d2dd79246fa741e0e6da443d04d1a`;
2. only the final accepted implementation commits after task base
   `7e4e685efbfc453a6d2209f37c99edb1ab46243d`.

Expected implementation surfaces:

- `crates/codefire-cli/src/http.rs`;
- `crates/codefire-core/src/work_budget.rs`.

The final repair range is a blocker until its exact commits are known. Defer
canonical source-document changes to the shared synthesis step.

### 3. Rust storage

Port, in order:

1. `40c095de250df2ee9e9e34526e1926069b476e42`;
2. `f5bce305da7ebfd08a840a4b281e29b0476874e0`;
3. `b3bc32a6bd4c792edc5a1302c61e16c624a9b648`;
4. `e9e42aae9eb3c5a0993ca07688ade5d4d6e0eed3`;
5. `cdcb9bdefe7d25a118a9dc14171864d75f5ad304`.

Expected implementation and focused-test surfaces:

- `crates/codefire-cli/src/main.rs`;
- `crates/codefire-cli/src/secure_worktree.rs`;
- `crates/codefire-cli/src/storage.rs`;
- `crates/codefire-cli/src/tests.rs`;
- `crates/codefire-cli/src/tests/storage_decision_work.rs`.

Defer canonical source-document changes from `cdcb9bd…` to the shared
synthesis step.

### 4. File-remote Doctor

Port:

1. `52a025ecda3fe2f5ad8b5b2a485562e7c16ab6de`;
2. only the final accepted implementation commits after task base
   `dbeaac1526acc8c2e102ee2b137e8f2a708890b0`.

Expected implementation surface:

- `crates/codefire-cli/src/remote/doctor.rs`.

The final repair range is a blocker until its exact commits are known. Defer
canonical source-document changes to the shared synthesis step.

### 5. Canonical source-document synthesis

After all code and focused tests are present, update exactly:

- `docs/development/source-architecture.md`;
- `docs/development/command-source-trace.md`;
- `docs/development/source-data-model-catalog.md`;
- `docs/development/change-playbook.md`.

Do not resolve these four files by choosing one candidate's blob or accepting
the last patch that applies. Derive one coherent description from the composed
source and commit the four-document synthesis as its own auditable step.

## Scope

The manager may create one clean composition branch from the frozen canonical
base, port only independently accepted implementation commits, manually
reconcile the shared core primitive, synthesize the four source documents, run
the focused gate ladder, and prepare one report-only independent review of the
exact composed tree.

Only the manager may:

- move this packet from `blocked` after binding all exact accepted hashes;
- select the clean composition base;
- port the accepted implementation ranges;
- resolve `work_budget.rs` and source-document overlap;
- decide whether the composed tree is ready for independent review;
- admit or reject the resulting review evidence.

Implementation and review agents may inspect or validate the manager's exact
tree only through later, separately frozen packets. They may not integrate
their own branch or update canonical.

## Manual `work_budget.rs` Reconciliation

Both accepted ancestry and the future accepted HTTP closure own
`crates/codefire-core/src/work_budget.rs`. A clean textual application is not
proof of semantic composition.

The ancestry candidate adds:

- `DecisionWorkProfile::FILE_REMOTE_OPERATION_DEADLINE_MS`;
- one-directional `DecisionWorkProfile::with_lowered_limit`;
- their exact scope and regression tests.

The HTTP closure is expected to add or revise:

- the request-ledger prepublication-sealer ownership contract;
- transactional prepared-response pair consumption and release;
- permit, selection, cancellation, fallback, and cleanup invariants.

The final blob must preserve both sets without widening a public profile,
tightening unrelated `local_v1()` commands, introducing a third request-ledger
terminal-response sealer, or making a local operation look like an HTTP
request. Candidate-local tests do not cover the manager's newly composed blob,
so the reconciled file is always part of the composed independent review.

The request-ledger statement about exactly two HTTP terminal-response owners
must remain scoped to request ledgers. `StorageReportRun`,
`FileRemoteOperation`, and the file-remote Doctor use distinct local-operation
ledgers; their use of `seal_prepublication` must not be misclassified as
additional simultaneous HTTP response owners.

## Shared Ledger, Sealer, and Terminal Ownership

The composed tree must preserve these route orders:

1. File-remote ancestry creates one `FileRemoteOperation` before the first
   sealed validation, uses its snapshot-bound session for validation,
   collection, policy, graph decision and final fence, stages object effects,
   publishes only after acceptance, then releases and terminalizes once.
2. HTTP merge-request create/review prepares the exact success and durable
   retry response pair before the first authoritative effect, selects the pair
   transactionally, preserves the selected result after commit, and hands one
   prepared response to the outer publisher without a second encoding.
3. Storage dispatch prepares one `StorageReportRun`, accounts projection and
   response ownership, seals, performs one tracked publication, releases, and
   finishes once. `PublishedOutputFailed` prevents a second stdout document
   after possible partial publication.
4. File-remote Doctor uses the exact production `local_v1()` /
   `DecisionWorkCommandKind::LocalGraph` factory, accounts each growth at its
   real phase, fences every consumed subject, encodes/seals/releases, and
   finishes once without cleanup replacing the primary error.

For every route, success and every injected failure must prove:

- exactly one operation/request ledger;
- no nested or silently replaced ledger;
- no simultaneous unauthorized sealer;
- no allocation, read, response selection, or authority effect before its
  owner is admitted;
- zero retained bytes, zero temporary bytes, and zero open permits at handoff;
- exactly one terminal observation;
- the primary typed outcome is not replaced by cleanup;
- no second response document after any possibly visible publication.

## Invariants

### Exact acceptance lineage

Every ported implementation commit belongs to the exact independently accepted
candidate range and every acceptance report binds that full hash and report
digest.

Falsification: substitute a parent, sibling, abbreviated-only subject, worker
success, or report for a different candidate. The packet must remain blocked.

### Ordered whole-candidate composition

The manager applies ancestry, HTTP closure, storage, Doctor, and source-doc
synthesis in the frozen order. A `REQUEST_CHANGES` candidate contributes no
partial implementation.

Falsification: omit, reorder, or selectively copy a rejected candidate hunk.
The composition inventory must reject the tree before any product gate.

### Semantic core reconciliation

The final `work_budget.rs` contains both accepted primitive families and
preserves their distinct local-operation and HTTP-request scopes.

Falsification: delete either family, widen a limit, add a request sealer,
change `local_v1()` deadline behavior, or make response-pair selection
partially consumable. Focused core and route tests must fail.

### One owner and terminal outcome

Each route has one ledger, one authorized prepublication owner at a time, one
terminal observation, and zero residual ownership.

Falsification: force double seal, partial selection, failed release,
session-discard failure, deadline, subject change, response failure, or partial
sink progress. The route must preserve the primary outcome and return to zero.

### Effect ordering

Every subject fence, response preflight, owner admission, and fallible cleanup
that must precede an authoritative effect still precedes it after composition.

Falsification: move ancestry copy, HTTP response preparation, storage response
ownership, or Doctor final subject fence across the first effect/publication.
The focused regression must fail with byte-identical pre-effect snapshots.

### Canonical source truth

The four source documents describe the exact composed call chains, owners,
data models, failure semantics, and remaining boundaries without selecting one
lane's stale wording.

Falsification: restore any pre-candidate row or omit one accepted owner. Both
document checkers or the independent source trace must reject it.

### Bounded closure claim

The composition decision covers only the exact accepted four-lane runtime
surface. It does not relabel separately owned residuals as closed.

Falsification: claim that this composition closes Explain combined ownership,
the typed Store semantic signal, or adjacent local merge, scan, clone,
storage-report, or GC graph-walk residuals without their own exact candidate
and accepted evidence. The composed review must return `REQUEST_CHANGES`.

## Non-Goals

- Do not implement new fixes while composing. A conflict that needs behavior
  not already accepted becomes a new repair candidate and review.
- Do not close Explain's combined storage-report projection/publication
  ownership or edit `crates/codefire-cli/src/explain.rs`.
- Do not close the typed Store semantic-validation signal or edit
  `crates/codefire-store/src/**`.
- Do not claim closure of separately owned local merge, scan, clone,
  storage-report, or GC graph-walk adoption.
- Do not change Python fallback, packaging, devcontainer, CI, agent wrappers,
  dogfood project code, issue state, release status, or historical reports.
- Do not raise public limits, weaken sealed/object/signature/subject checks, or
  reinterpret a required failure as successful partial work.
- Do not run Install, Dogfood, Close, or Retrospect from this packet.
- Do not merge review/task/manifest/control-plane branch history merely because
  it is ancestral to a product candidate.

## Validation

No product validation runs while this packet is blocked. After all four exact
`ACCEPT` reports are bound and the manager has committed one clean composition
candidate, use the shared constrained toolchain and serialize every Cargo
command:

```bash
export RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup
export CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=2
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared
```

Run in this order:

```bash
git diff --check
cargo fmt --all -- --check
cargo test -p codefire-core --locked work_budget -- --nocapture
cargo test -p codefire-cli --bin codefire-rs --locked remote::file_remote_ancestry_tests -- --nocapture
cargo test -p codefire-cli --bin codefire-rs --locked merge_request -- --nocapture
cargo test -p codefire-cli --bin codefire-rs --locked http::tests -- --nocapture
cargo test -p codefire-cli --bin codefire-rs --locked storage_decision_work -- --nocapture
cargo test -p codefire-cli --bin codefire-rs --locked storage -- --nocapture
cargo test -p codefire-cli --bin codefire-rs --locked remote::doctor -- --nocapture
cargo test -p codefire-cli --bin codefire-rs --locked doctor -- --nocapture
cargo test -p codefire-cli --bin codefire-rs --locked
cargo clippy -p codefire-cli --all-targets --locked -- -D warnings
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Also run the exact neighboring upload/apply/replay/Explain cases named by the
four accepted reports. Record commands, counts, durations, toolchain, resource
observations, and exact candidate/tree hashes. Do not parallelize Cargo or
substitute earlier per-lane results for composed execution.

## Composed Independent Review

Only after the complete focused ladder and source-document synthesis pass may
the manager freeze a separate report-only review packet for the exact composed
commit.

That review must independently decide:

- whether the accepted code survived porting without omitted or extra hunks;
- whether `work_budget.rs` reconciliation preserves both primitive families;
- whether every route still has the intended ledger, sealer, publication, and
  terminal order;
- whether `CliError::PublishedOutputFailed` remains exhaustive across the
  composed CLI;
- whether `remote.rs` and `remote/doctor.rs` remain one compatible module;
- whether all four source documents match the composed source;
- whether any new cross-lane counterexample or residual was introduced;
- whether the closure claim excludes all separately owned residuals.

The report must bind the exact composed commit, tree, patch digest, validation
evidence digest, and literal `ACCEPT` or `REQUEST_CHANGES`. A request for
changes returns this composition to blocked. It does not authorize partial
integration.

## Handoff

The manager handoff must provide:

- frozen canonical base and clean composition head/tree;
- all four candidate and acceptance-report commits and report SHA-256 values;
- exact ported implementation commit list and excluded control-plane commits;
- before/after file inventory and candidate-to-composed blob map;
- manual `work_budget.rs` reconciliation proof;
- source-document synthesis commit and trace matrix;
- shared ledger/sealer/terminal ownership matrix;
- focused validation commands, counts, durations, hashes, and resource state;
- exact composed review packet/report commit, digest, and decision;
- explicit disposition of every excluded residual;
- conflicts or new behavior that require a separately reviewed successor;
- a statement that canonical integration, workspace-wide release gates,
  Install, Dogfood, Close, and Retrospect remain unauthorized by this packet.

This packet's successful end state is an independently accepted composition
candidate and manager evidence bundle, not canonical merge or CFR-168 closure.
