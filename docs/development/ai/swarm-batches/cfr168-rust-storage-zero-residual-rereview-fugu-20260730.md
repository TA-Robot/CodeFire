# CFR-168 Rust Storage Exact-Candidate Independent Rereview

Independently rereview the exact Rust storage repair candidate
`cdcb9bdefe7d25a118a9dc14171864d75f5ad304` with Fugu Ultra 1.1
(`fugu-ultra-v1.1`). This is a report-only lane. Product code, tests,
canonical source documents, issue state, management ledgers, and prior reports
are read-only.

The candidate claims to close F1 through F9 from the prior independent report.
Do not infer acceptance from that claim, the implementation agent's successful
exit, test names, comments, commit messages, or session-log summaries. Establish
the result from the exact committed source, the exact two-commit range,
independent derivations, non-vacuous tests, and the constrained validation
allowed below.

## Exact identity and immutable scope

| Role | Commit or digest |
|---|---|
| Exact review subject | `cdcb9bdefe7d25a118a9dc14171864d75f5ad304` |
| Subject tree | `1836a83aa42df026f578b134f4b455830b873a3f` |
| Subject parent / durable checkpoint | `e9e42aae9eb3c5a0993ca07688ade5d4d6e0eed3` |
| Frozen repair task revision | `13767c7f39eca82d4a9093a055abf568814cfef0` |
| Repair-task tree | `d053752a48a8a1de15c7d3b16a8b2d7331920523` |
| Rejected predecessor candidate | `b3bc32a6bd4c792edc5a1302c61e16c624a9b648` |
| Prior independent report commit | `4cfb607cfd6ac95d26d24a4542ac15ba56b428d6` |
| Prior report as integrated into canonical | `4380985` |
| Exact range patch SHA-256 (`git diff --binary`) | `a6fab7cbb36d89be52dc4d400a2d84987351888642b8ce481fdf78b51b6487a4` |
| Repair task Markdown SHA-256 | `b24faadc5293893fa74fd601ca019358d151a5df2654fdad27214d240b908b59` |
| Prior report Markdown SHA-256 | `628a37f51cad4710eeccb8c9b34286b9e41332a59a421068a6a1aaa61ed4140a` |
| Final candidate commit-message SHA-256 | `3e58dbc35c9cc4a4dd24f8d8c3bbdcb95fdca372d9ebbe460d99184b6b15e07f` |

Verify all identities before substantive review. The repair range must be
exactly:

1. `e9e42aae9eb3c5a0993ca07688ade5d4d6e0eed3`
   (`wip(storage): repair F1-F8 storage decision-work residuals`);
2. `cdcb9bdefe7d25a118a9dc14171864d75f5ad304`
   (`fix(cfr168): close F1-F9 Rust storage zero-residual residuals`).

`git rev-list --count 13767c7..cdcb9bd` must be two, and
`13767c7` must be an ancestor of `cdcb9bd`. Review the cumulative range
`13767c7f39eca82d4a9093a055abf568814cfef0..cdcb9bdefe7d25a118a9dc14171864d75f5ad304`,
not only the final commit.

The cumulative range is expected to contain only these eight modified files
and to have a stat of 2,972 insertions and 291 deletions:

| Path | Final blob |
|---|---|
| `crates/codefire-cli/src/main.rs` | `a1981193eace2e3bd21d3d122a4397bd9ab95916` |
| `crates/codefire-cli/src/secure_worktree.rs` | `dacdf244e3de78ba04411bd0ff2e79c4d9038341` |
| `crates/codefire-cli/src/storage.rs` | `c22ba1900feaea95bfe8be427a4b66c0a0578258` |
| `crates/codefire-cli/src/tests/storage_decision_work.rs` | `6f8b5a8c65ef3ff44230f81cc31bd65a7baf9a12` |
| `docs/development/change-playbook.md` | `1bdd04f102c14323e0e2ba44d03253e070e2033d` |
| `docs/development/command-source-trace.md` | `c7093219c449d304fbb63b67a403e0383bcc6068` |
| `docs/development/source-architecture.md` | `d632dd5cffb51e4b16e07eb3ee4ff7018637d4e3` |
| `docs/development/source-data-model-catalog.md` | `21feaeb9cecdc05d9405b4e49e096ed1b313bd25` |

If ancestry, tree, range, path set, or blobs differ, stop and return
`REQUEST_CHANGES` with the exact mismatch. Do not silently review a nearby
candidate.

## Evidence boundary

The committed evidence is the two repair commits, their source/test/doc blobs,
the inherited repair task, the inherited prior report, and their commit
messages. The manager also observed a clean implementation worktree and a
successful `dev-swarm` task state with two commits and final head `cdcb9bd`.
Those observations establish a handoff identity, not correctness.

The implementation session log separately claims:

- 33/33 focused `storage_decision_work` tests;
- 41/41 `storage`-filtered tests;
- 801/801 binary tests;
- formatting, strict package Clippy, both source/document checkers, and
  `git diff --check`;
- candidate-behavior reversals for several findings;
- 1x/3x/10x counter and owner curves.

There is no committed result artifact in the two-commit candidate range that
immutably records those executions. Treat the session-log claims as leads only.
The report must distinguish:

1. evidence independently rerun by this reviewer;
2. committed source or test proof inspected by this reviewer;
3. implementation-lane claims not independently reproduced.

Never convert category 3 into acceptance evidence.

## Required reading

Read the following before deciding:

- `AGENTS.md`;
- `docs/development/ai/swarm-batches/cfr168-rust-storage-zero-residual-repair-20260730.md`;
- `docs/development/ai/swarm-results/cfr168-rust-storage-independent-review-fugu-20260730/review.md`;
- `adr/ADR-020-decision-equivalent-work-budget.md`;
- `docs/development/code-review-issues/cfr-168.md`;
- `docs/development/bug-issues/cfb-136.md`;
- the four changed canonical source documents;
- the complete two-commit diff and all four changed Rust files;
- the live storage dispatch and error rendering in `main.rs`;
- the relevant shared ledger/profile definitions in `codefire-core`;
- the called remote URL/layout helpers in `remote.rs`;
- the Store scanner/finalization contract;
- the internal Explain storage-warning consumer.

Shared callees outside the write set are review inputs, not authorized edits.

## Decision rule

Return `ACCEPT` only if direct evidence establishes all F1-F9 closure
requirements, all required test/non-vacuity properties, the exact ownership and
complexity claims, and no newly introduced storage-local blocker. If a required
claim is contradicted, missing, indirect, or cannot be checked within the
allowed lane, return `REQUEST_CHANGES`. Do not narrow the packet around what
happens to pass.

An accepted cross-command handoff must name the exact symbol, owning module,
revision, and reason it is genuinely outside this storage candidate. A
storage-local gap cannot be relabeled as predecessor-gated.

## F1 — portable logical formulas

Independently verify all of the following:

1. No decision-work charge in the live storage path depends on
   `size_of::<T>()`, allocator capacity/layout, pointer width, enum padding,
   compiler representation, or a platform-only path representation.
2. `StorageLogicalShape` and every formula use checked arithmetic and match the
   actual logical fields and live owner lifetimes. Audit at least
   `StorageReport`, `StorageWarning`, `AreaStats`, `ObjectTypeStats`,
   `ObjectFileStats`, `RemoteStorageStats`, `RemoteGenerationStats`, strings,
   paths, maps, vectors, traversal frames, parser owners, projection state,
   response state, and publication state.
3. The claimed 256-byte frame and per-field widths are the frozen ADR-020
   logical contract rather than an arbitrary blanket reservation introduced
   merely to make tests pass.
4. Payload lengths include every exact UTF-8/encoded owner and every
   simultaneous duplicate. A conservative upper must remain portable,
   input-proportional, and free of a false public-limit cliff.
5. Golden tests derive expected formulas independently of the production
   helper and demonstrate exact-limit acceptance and one-unit-insufficient
   rejection.

The report must contain a formula table for every retained and temporary owner
encountered on local quick/full, selected-remote, text, JSON, success, and
failure routes. Show checked simultaneous-owner peak derivations rather than
listing isolated object sizes.

## F2 — projection, response, and encoding owners

Trace text and JSON from the retained report through projection, encoding,
prepublication sealing, response construction, publication, release, and
terminal finish.

Verify that:

1. a proved upper is acquired before the first allocation of every projected
   envelope, data field, warning, diagnostic, evidence string, next action,
   collection, and encoding scratch owner;
2. the projection formula dominates actual output for worst-case escaping,
   pretty/canonical formatting, maximum numeric widths, repeated warning
   evidence, skipped checks, type/largest-object collections, remote status,
   generations, and maximum-length paths;
3. the exact response buffer is owned before `Vec`/string construction and
   remains simultaneously accounted with retained report and projection
   owners;
4. temporary and retained releases occur exactly once on success and every
   error;
5. many-warning and maximum-string tests place the limit at the intended
   projection/response owner, not at an earlier graph or record counter.

Inspect `reference_json_bytes` structurally. It may use general serialization
primitives, but it must not call production projection helpers, share their
field-selection/order implementation, or reproduce the same defect by copying
an unchecked production formula.

## F3 — iterative traversal, depth, and stable order

Prove from the live call graph that capability traversal is iterative. Search
for direct and indirect recursion and confirm that no depth-dependent process
stack remains.

Audit:

- root logical depth is zero;
- child edge depth and all frame/frontier capacity are checked and owned before
  the child is opened or pushed;
- frontier growth, pending entries, paths, directory handles, identities, and
  per-frame state are included in the portable owner peak;
- descriptor-relative no-follow behavior and parent/directory identity fences
  remain in force;
- quick/full traversal and historical visit order are unchanged;
- every early return unwinds frame and permit ownership exactly once;
- the largest-object state remains a stable, hard-bounded top ten.

Exercise or directly prove empty-root depth zero, exact depth, one-edge-too-deep,
symlink variants, and a deep fixture that demonstrates heap iteration without
approaching process stack failure. Explain the filesystem path-length limit and
why the chosen deep fixture is or is not decisive.

## F4 — pin, charge, read, and revalidate one descriptor

Trace `read_optional_json_accounted` through
`secure_worktree::PinnedBoundedFile`.

Verify:

1. the no-follow descriptor is opened and pinned before its size is used;
2. buffer, read, parse, and retained owners are acquired from that same pinned
   descriptor before any capacity allocation or byte read;
3. the reader cannot allocate or read beyond the charged `pinned + 1` bound,
   including overflow and supported-size-limit cases;
4. in-place growth, inode replacement, truncation, parent replacement, and
   final descriptor/parent revalidation produce the intended typed outcome;
5. all owner and authority-byte state returns to zero/closed on each window;
6. exact and one-unit-insufficient temporary limits fail at this owner rather
   than a predecessor.

The two race-window tests must prove allocation/read ordering, not only that a
later revalidation eventually notices change.

## F5 — URL parsing and decoded owners

Audit the storage-owned prevalidation adapter and the unchanged shared
`parse_cf_project_url` implementation together.

Determine whether the declared URL byte limit, segment count, segment-vector
upper, server/suffix copies, percent-decoded byte/string owners, project root,
and parser-returned values truly dominate every allocation before the shared
parser performs it. Check long URLs, many empty/non-empty segments,
percent-encoded maximum input, invalid decoding, overflow, and exact/one-unit
boundaries.

Explain whether the 16 KiB URL rejection preserves the prior supported public
domain or creates a new false cliff. Confirm that reserving a multiple of input
bytes is tied to actual simultaneous owners rather than a blanket constant.

## F6 — every simultaneous local and remote path owner

Reconstruct owner lifetimes before the first `join` and through every
subsequent policy, GC, nonce/idempotency, object, branch, merge-request, lock,
active, and transient record path.

Do not accept `LOCAL_PATH_OWNER_COUNT = 5` or
`REMOTE_PATH_OWNER_COUNT = 6` from their names. Enumerate the actual live
`PathBuf`/encoded-path owners, intermediate joins, clones, returned values, and
roots. Verify separator/suffix bounds and non-UTF-8/platform behavior.

The report must show local and selected-remote peak formulas and explain
whether tests independently detect omission of each owner. The implementation
handoff disclosed that removing the local reservation did not make its focused
probe fail; decide whether source-order proof plus the other test is sufficient
or whether this remains a blocking non-vacuity gap.

## F7 — typed terminal matrix and error precedence

Build an independent matrix for every relevant `CliError` class across
subject-work, encoding, and publication phases:

- bounded usage/business completion;
- ordinary I/O;
- parse/render failure;
- actual subject integrity failure;
- subject change;
- ledger budget exhaustion;
- host resource exhaustion;
- publication/sink failure.

Verify the mapping to terminal reason, exit status, diagnostic, output channel,
one terminal observation, zero retained bytes, zero temporary bytes, and zero
permits. Look for wildcard arms that silently absorb future variants.

Prove that cleanup/finalization cannot replace or hide the primary typed error.
The candidate uses a `debug_assert!` around visitor cancellation/finalization;
decide explicitly whether a debug-only assertion closes the release-build
precedence concern. Separately decide whether the string/context-based Store
semantic-validation recognition is safely outside this write set or remains a
storage-local correctness dependency.

## F8 — publication state and one externally visible outcome

Audit the complete text and JSON publication route, including
`StoragePublicationState`, `write_all_tracked`,
`CliError::PublishedOutputFailed`, `main::top_level_error_rendering`, display,
diagnostic, and exit-status delegation.

Verify:

1. all bytes are encoded, owned, and preflight-sealed before the first write;
2. the state distinguishes not attempted, attempted without confirmed progress,
   partial progress, and complete publication;
3. zero-byte/interrupt/retry/short-write/error/flush behavior follows the Rust
   `Write` contract and does not claim transactional rollback;
4. after any attempt that may have become visible, top-level handling emits no
   second stdout document in text or JSON mode;
5. a true prepublication failure still produces the one historical typed error
   envelope;
6. retained/temporary ownership and terminal observation remain balanced when
   publication fails at every boundary.

The partial and rejecting writers must be non-vacuous: confirm their exact call
sequence and the actual bytes/channels observed through the top-level route,
not only an internal enum value.

## F9 — canonical source truth

Compare all four changed canonical documents with the exact live route and
types. In particular, verify:

- dispatch -> prepare -> one `StorageReportRun` -> encode/seal/publish/release/
  finish;
- one command ledger shared by local and selected-remote scans;
- the actual portable formula and pinned-read contracts;
- terminal/publication behavior and the top-level channel decision;
- the internal Explain consumer boundary.

The documents must not imply that Explain's projection/publication is already
owned by the finished storage ledger. Run both document checkers. A stale or
overstated canonical document keeps F9 open.

## Independent oracle and candidate-reversal requirements

For each F1-F9 item, classify the evidence as one of:

1. executable failure against the rejected predecessor;
2. executable targeted reversal of the repaired behavior;
3. independent source proof where execution is genuinely impractical.

Do not claim an executable reversal unless this reviewer actually ran it.
Temporary mutations are allowed only for local review experiments, must never
be committed, and must be restored byte-for-byte before the report commit.
Prefer a detached temporary worktree or saved exact blobs over editing the
candidate worktree. Do not create large duplicate build directories; reuse the
constrained shared target directory.

At minimum, independently challenge:

| Finding | Candidate behavior to reverse or predecessor behavior to expose |
|---|---|
| F1 | ABI-derived `size_of` charge |
| F2 | retained bytes plus fixed scratch before JSON projection |
| F3 | recursive traversal and root depth one |
| F4 | ambient size charge followed by reopened read |
| F5 | fixed remote scratch independent of URL/segments |
| F6 | omitted local/remote simultaneous path owner |
| F7 | catch-all corruption mapping and cleanup precedence |
| F8 | untracked `write_all` followed by generic top-level output |
| F9 | stale pre-repair command trace |

The independent JSON oracle must be reviewed separately from the mutation
evidence. A test that shares production field selection, ordering, formulas, or
helpers is not independent merely because it has a different function name.

## Exact boundaries, curves, cleanup, and complexity

Produce an independently derived table at representative 1x, 3x, and 10x
fixtures covering quick and full behavior as applicable. Include:

- graph nodes and edges;
- logical depth and maximum frontier;
- aggregate bytes and object records;
- counter values/open permits;
- prepared and retained peaks;
- temporary/projection/response peaks;
- terminal reason and final retained/temporary/permit debt.

Do not copy the implementation session's numbers without reproduction. If a
curve cannot be executed within this lane, derive it from committed fixture
construction and formulas, label it as static evidence, and state the missing
execution.

For graph nodes, edges, depth, aggregate bytes, records, counter values,
retained bytes, temporary bytes, projection, response, and pinned-read scratch,
record the intended boundary semantics:

- work exactly at limit `L`;
- the same work with limit `L - 1`;
- a subject with one additional unit at fixed limit `L`.

Confirm which case succeeds, which typed limit fails, and that failure reaches
the intended owner. A predecessor failure is not evidence for the target
boundary.

State time and memory complexity for traversal, aggregation, sorting/top ten,
URL parsing, JSON projection, and publication. Prove bounded heap depth, stable
visit/output order, top-ten tie behavior, and zero debt on success and every
injected failure.

## Newly introduced residual audit

The repair is large. Review beyond the named prior findings for:

- unchecked arithmetic or lossy casts;
- allocations before owner acquisition;
- owner lifetime mismatch, double release, or cleanup masking;
- accidental public-limit changes;
- changed quick/full, local/remote, text/JSON, warning, diagnostic, ordering,
  or exit behavior;
- no-follow/capability or subject-revalidation regressions;
- new `main.rs` exhaustiveness/diagnostic gaps;
- test-only seams reachable in production;
- module-boundary violations or storage-specific logic placed in the wrong
  owner.

Record every newly discovered storage-local issue. Do not suppress a new
finding because it was absent from the prior report.

The prior report left three questions visible. Give each an explicit verdict:

1. typed object semantic-validation signaling;
2. visitor error versus Store finalization/error precedence;
3. Explain's combined storage-report/projection/publication ownership.

If any remains a valid cross-command handoff, name its exact owner and do not
claim global CFR-168 closure.

## Constrained validation

Use the shared pinned toolchain and target:

```bash
export RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup
export CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo
export PATH="$CARGO_HOME/bin:$PATH"
export CARGO_BUILD_JOBS=2
export RUST_TEST_THREADS=2
export CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared
```

First run read-only ancestry, exact path/blob, source, and `git diff --check`
checks. Then serialize these focused gates:

```bash
cargo fmt --all -- --check
cargo test -p codefire-cli --locked storage_decision_work -- --nocapture
cargo test -p codefire-cli --locked storage -- --nocapture
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
git diff --check
```

One package-scoped strict Clippy run is allowed after the focused tests if the
resource envelope remains healthy:

```bash
cargo clippy -p codefire-cli --all-targets --locked -- -D warnings
```

Before each Cargo gate, inspect memory, swap, target size, and free disk. Do not
start a new heavy gate if available memory is below 6 GiB, swap use exceeds
2 GiB, the shared target exceeds 20 GiB, or free workspace disk is below
100 GiB. Record a resource skip honestly; do not substitute old log claims.

Do not run Docker, install, `demo.sh`, unrestricted test discovery, a full Rust
workspace gate, a full binary suite, network-dependent commands, or dogfood.
Do not modify issue status or integrate the candidate. Avoid parallel Cargo
processes.

## Required report

Write exactly:

`docs/development/ai/swarm-results/cfr168-rust-storage-zero-residual-rereview-fugu-20260730/review.md`

The report must include:

1. exact candidate, tree, parent, two-commit range, prior report, patch digest,
   path set, blob verification, reviewer-start revision, and final report
   commit parent;
2. one unambiguous `ACCEPT` or `REQUEST_CHANGES` decision;
3. severity-ordered findings with exact file/line or symbol, concrete
   counterexample, violated invariant, and required repair;
4. an explicit F1-F9 disposition table;
5. the portable formula and simultaneous-owner peak tables;
6. the 1x/3x/10x counter/owner curves and exact boundary matrix;
7. traversal/ordering/complexity and descriptor/authority evidence;
8. the full terminal/publication matrix and observable-output evidence;
9. independent-oracle and candidate-reversal results, distinguishing execution
   from source proof;
10. source-document verdicts and all three prior unresolved-question verdicts;
11. commands, status/counts/durations, toolchain, resource observations, and
    every skipped or failed gate;
12. zero retained/temporary/permit debt evidence and any exact cross-command
    handoff;
13. the exact required next action.

Commit only that report and leave the reviewer worktree clean. The report lane
does not authorize product/test/doc edits, integration, issue Close, Install,
Dogfood, or Retrospect.
