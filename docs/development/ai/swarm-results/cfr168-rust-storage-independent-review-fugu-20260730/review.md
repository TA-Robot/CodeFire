# CFR-168 Rust Storage Independent Adversarial Review

Date: 2026-07-30
Reviewer: Fugu independent review lane
Decision: **REQUEST_CHANGES**

## 1. Exact review identity

| Role | Commit |
|---|---|
| Subject candidate | `b3bc32a6bd4c792edc5a1302c61e16c624a9b648` |
| Recovery/baseline precursor | `40c095de250df2ee9e9e34526e1926069b476e42` |
| Focused finisher precursor | `f5bce305da7ebfd08a840a4b281e29b0476874e0` |
| Focused finisher range | `f5bce305da7ebfd08a840a4b281e29b0476874e0..b3bc32a6bd4c792edc5a1302c61e16c624a9b648` |
| Review-start worktree `HEAD` | `ab551fdcdf2cfa816b58313fab0d5a696fe40bc6` |

`b3bc32a6bd4c792edc5a1302c61e16c624a9b648` is a direct child of
`f5bce305da7ebfd08a840a4b281e29b0476874e0`; both recovery precursors are ancestors of the
subject. The review-start `HEAD` adds only
`docs/development/ai/swarm-results/cfr168-rust-storage-finisher-recovery-claude-20260730/result.md`
after the candidate. The reviewed product and focused-test blobs at `HEAD` exactly match the
candidate:

| Path | Candidate/`HEAD` blob |
|---|---|
| `crates/codefire-cli/src/main.rs` | `c71d0f5965d146f08624179d7693ff64c293490e` |
| `crates/codefire-cli/src/storage.rs` | `1a5ed9972cba6ccd282af817f0c1648f6dfbc6ae` |
| `crates/codefire-cli/src/tests.rs` | `57003cf20c954c55cdd0dfe62721cb7f5d776b5a` |
| `crates/codefire-cli/src/tests/storage_decision_work.rs` | `770801003e0115324f434b9ed3c90f940defdb89` |
| `docs/development/command-source-trace.md` | `e3e63af27c8e3aeaa8c5ddc999d6ce24af089e16` |

The full baseline range changes only:

- `crates/codefire-cli/src/main.rs`
- `crates/codefire-cli/src/storage.rs`
- `crates/codefire-cli/src/tests.rs`
- `crates/codefire-cli/src/tests/storage_decision_work.rs`

The focused finisher range changes only `crates/codefire-cli/src/storage.rs` and
`crates/codefire-cli/src/tests/storage_decision_work.rs`.

## 2. Review method and limits

This was an independent source/diff/test-design review. It read:

- `AGENTS.md`
- `docs/development/ai/swarm-batches/cfr168-rust-storage-ledger-20260729.md`
- `adr/ADR-020-decision-equivalent-work-budget.md`
- `docs/development/code-review-issues/cfr-168.md`
- `docs/development/bug-issues/cfb-136.md`
- the canonical source documents `docs/development/source-architecture.md`,
  `docs/development/command-source-trace.md`,
  `docs/development/source-data-model-catalog.md`, and
  `docs/development/change-playbook.md`
- the exact baseline and focused diffs, the complete candidate storage implementation, its live
  CLI route, the remote URL/layout helpers, the no-follow bounded reader, Explain integration, and
  the focused and legacy storage tests
- the separately recorded finisher execution evidence at
  `docs/development/ai/swarm-results/cfr168-rust-storage-finisher-recovery-claude-20260730/result.md`

The review traced quick/full, text/JSON, local/selected-remote, malformed/integrity, budget,
publication, sink-error, and internal-consumer paths. It audited ledger creation and propagation,
pre-allocation charging, retained/temporary ownership, terminal classification, cleanup/error
precedence, output publication, ordering, top-N behavior, and no-follow capability use.

Per the recovery constraint, this lane ran no Cargo, `rustc`, `rustfmt`, Clippy, test, build,
install, or demo command and created no target artifact. Validation was limited to non-building
Git and text inspection. Existing execution evidence is cited as separate evidence, not represented
as work rerun by this reviewer.

## 3. Decision

**REQUEST_CHANGES.** The candidate establishes a useful single-ledger owner and preserves important
legacy behavior, but it does not close the selected Rust storage/report decision-work surface under
the zero-residual rule. Four high-severity defects permit platform-dependent limits, allocations or
reads before adequate accounting, or unbounded call-stack failure. Four medium defects leave remote
parsing/path ownership, terminal semantics, and stdout publication outside the stated contract. The
canonical command trace is also stale. All findings below require repair before a replacement
candidate can be accepted.

## 4. Findings

### F1 — High — Retained/temporary accounting depends on Rust ABI layout

**Affected code:** `storage_struct_bytes<T>` in `crates/codefire-cli/src/storage.rs:543`; live uses at
`crates/codefire-cli/src/storage.rs:549`, `crates/codefire-cli/src/storage.rs:1165`,
`crates/codefire-cli/src/storage.rs:1483`, `crates/codefire-cli/src/storage.rs:1680`,
`crates/codefire-cli/src/storage.rs:1873`, `crates/codefire-cli/src/storage.rs:1967`,
`crates/codefire-cli/src/storage.rs:2168`, `crates/codefire-cli/src/storage.rs:2444`, and
`crates/codefire-cli/src/storage.rs:2470`.

`storage_struct_bytes<T>` charges `size_of::<T>()`. Its callers therefore account
`StorageReport`, `StorageWarning`, `String`, `ObjectTypeStats`, `ObjectFileStats`,
`RemoteStorageStats`, and `RemoteGenerationStats` according to pointer width, padding, enum layout,
and compiler/runtime representation. ADR-020 defines portable logical retained bytes independent of
allocator and runtime object layout (`adr/ADR-020-decision-equivalent-work-budget.md:70`). This is a
direct contract violation, not merely a conservative approximation.

**Concrete counterexample:** run the same repository and immutable decision-work profile on a
32-bit and a 64-bit target, or across a representation-changing compiler update. A retained limit
placed between the two computed totals accepts on one target and returns exit 60 on the other even
though the command's logical subject and output are identical. Exact-limit observations and metrics
also drift without any domain change.

**Required repair:** replace every `size_of` charge with explicit, checked, portable logical units
derived from the frozen contract. Add golden tests that assert the declared logical byte formula for
each owner and exact/exact-plus-one behavior without consulting Rust layout.

### F2 — High — JSON preflight materializes unbounded duplicate state before output reservation

**Affected code:** `StorageReportRun::encode_json_output` in
`crates/codefire-cli/src/storage.rs:194`; `storage_json_encoding_temporary_bytes` in
`crates/codefire-cli/src/storage.rs:511`.

The only pre-materialization temporary acquisition is `retained_bytes + 4 KiB`
(`crates/codefire-cli/src/storage.rs:194` and `crates/codefire-cli/src/storage.rs:511`). The code then
constructs the complete data `Value`, diagnostics, next actions, and command envelope at
`crates/codefire-cli/src/storage.rs:205` before it counts serialized bytes. The output reservation is
not attempted until `crates/codefire-cli/src/storage.rs:223`. Those projection helpers allocate new
JSON maps/vectors/strings and duplicate warning evidence while the original `StorageReport` remains
retained. No proof makes `retained_bytes + 4 KiB` an upper bound for that simultaneous state; for
warnings represented in both report data and diagnostics, the projection can own multiple logical
copies in addition to the original.

**Concrete counterexample:** construct many maximum-length warning kinds, messages, and paths and
set `TemporaryBytes` headroom just above `retained_bytes + 4 KiB` but below the actual envelope
projection peak. The ledger accepts the preflight, then all projection allocations occur. Only after
that hidden peak does the output-byte reservation have a chance to fail. The current 96-record test
does not place a boundary around this projection and cannot refute the counterexample.

**Required repair:** either serialize from borrowed report state without building a duplicate
`Value` tree, or compute and acquire a checked portable upper before every projection allocation.
Reserve response ownership before constructing the response buffer. Add many-warning and
maximum-string exact/exact-plus-one tests for the simultaneous report/projection/output peak.

### F3 — High — Capability traversal is recursive and starts the root at logical depth one

**Affected code:** root invocation in `crates/codefire-cli/src/storage.rs:1264` and recursive
`scan_capability_files` call in `crates/codefire-cli/src/storage.rs:1363`.

The candidate calls `scan_capability_files(..., 1, ...)` for the traversal root and recursively calls
the same Rust function for every child directory. ADR-020 defines root logical depth as zero
(`adr/ADR-020-decision-equivalent-work-budget.md:70`). The pre-ledger reference traversal used an
explicit heap stack; the candidate reintroduces process call-stack dependence while advertising a
logical maximum depth of up to 100,000.

**Concrete counterexamples:**

1. With an injected maximum-depth limit of zero, even an empty root is rejected although it has
   zero graph edges from itself. Every exact boundary is shifted by one.
2. A sufficiently deep directory chain can overflow the Rust process stack well before the ledger's
   allowed depth is reached. That abort/panic is neither the typed `resource_budget_exceeded`
   diagnostic nor a balanced terminal transition.

**Required repair:** restore an iterative capability traversal with an explicitly charged heap-owned
frame/frontier, start the root at depth zero, and reject the first child whose edge depth would exceed
the limit before pushing/opening it. Add depth-zero, exact/exact-plus-one, and very-deep non-recursive
fixtures.

### F4 — High — Optional JSON reads charge stale ambient size but allocate/read reopened size

**Affected code:** `read_optional_json_accounted` in
`crates/codefire-cli/src/storage.rs:2491`; `read_file_nofollow_bounded` in
`crates/codefire-cli/src/secure_worktree.rs:1387`.

`read_optional_json_accounted` obtains ambient path metadata at
`crates/codefire-cli/src/storage.rs:2507`, charges record bytes and parse scratch from that size at
`crates/codefire-cli/src/storage.rs:2531`, and only then calls a helper that reopens the path at
`crates/codefire-cli/src/storage.rs:2555`. The helper obtains fresh descriptor metadata and allocates
`Vec::with_capacity(metadata.len() + 1)` at
`crates/codefire-cli/src/secure_worktree.rs:1420`. If ambient size `P` becomes reopened size `Q`, the
read and `Q`-sized allocation have already happened before the caller notices `bytes.len() != P` at
`crates/codefire-cli/src/storage.rs:2560`.

**Concrete counterexample:** expose a one-byte policy/GC/idempotency JSON file to the first metadata
call, then replace or grow it to several MiB (still below `REPOSITORY_JSON_BYTE_LIMIT`) before the
descriptor-relative open. The ledger charges one byte plus one-byte-derived scratch, while the
helper reserves and reads the multi-MiB `Q` buffer. The command eventually reports “changed during
read,” but bounded-before-effect has already been violated.

**Required repair:** open and pin the no-follow file first, derive the charged size from that exact
descriptor, acquire the buffer/read/parse upper, and then read and revalidate the same descriptor; or
reserve the full supported maximum before open/read. Add a deterministic metadata-to-open growth
race seam proving no `Q` allocation/read occurs without sufficient headroom.

### F5 — Medium — Fixed remote scratch does not bound URL parsing allocations

**Affected code:** `scan_remote_storage` in `crates/codefire-cli/src/storage.rs:2031`;
`parse_cf_project_url` in `crates/codefire-cli/src/remote.rs:2577`; `percent_decode` in
`crates/codefire-cli/src/remote.rs:2616`.

The storage owner acquires a fixed `5 * 16 KiB` path scratch at
`crates/codefire-cli/src/storage.rs:2038`. Parsing then collects every slash-separated segment into a
`Vec<&str>`, builds `suffix` and `server_raw` strings, constructs a `PathBuf`, allocates decoded byte
vectors/strings for organization and application, and joins the final project path
(`crates/codefire-cli/src/remote.rs:2581` through `crates/codefire-cli/src/remote.rs:2613`). The fixed
scratch is independent of URL byte length and segment count and therefore cannot be a valid upper
for these allocations.

**Concrete counterexample:** pass a syntactically valid `cf://` argument with thousands of server
path segments and ordinary final organization/application components. The collected segment vector
alone can exceed 80 KiB on a 64-bit target, before counting copied strings, decoded buffers, and
paths. No ledger check occurs at that growth point.

**Required repair:** impose and charge an explicit URL/segment byte bound before allocation, parse
without an unbounded collected vector, and reserve every decoded/path owner from checked input-sized
formulas. Add long-URL, many-segment, percent-decoding, exact, and exact-plus-one tests.

### F6 — Medium — Local and remote path-owner preallocation omits simultaneous `PathBuf` owners

**Affected code:** local path construction in `prepare_storage_report_with_ledger` at
`crates/codefire-cli/src/storage.rs:810`, `crates/codefire-cli/src/storage.rs:812`,
`crates/codefire-cli/src/storage.rs:824`, and `crates/codefire-cli/src/storage.rs:825`; remote path
construction in `scan_remote_storage` at `crates/codefire-cli/src/storage.rs:2036` and
`remote_dirs` at `crates/codefire-cli/src/remote.rs:2456`.

The local path creates `.codefire`, `objects`, `active`, and `idempotency` `PathBuf`s through `join`
before any exact owner acquisition for those allocations. On the remote side, the storage comment
claims “project root plus four derived layout directories,” but `remote_dirs` constructs five
derived paths (`objects`, `branches`, `merge_requests`, `locks`, and `idempotency`) while the parsed
`project_root` remains live. Policy, GC-state, and nonce/idempotency record joins add further
overlapping paths. The fixed count does not describe the live owner peak.

**Concrete counterexample:** with paths near the declared 16 KiB envelope, the parsed remote root
plus five `remote_dirs` values alone can require roughly six path envelopes while only five are
reserved; later policy/GC/record joins widen the gap. A tight temporary limit accepts the fixed
scratch and allocations then exceed the claimed peak. Local joins similarly allocate even if the
next charged scan fails immediately.

**Required repair:** construct each path in a precharged owner with an exact checked capacity, or
reserve the true simultaneous owner set before the first `join`. Audit intermediate and cloned paths,
not only returned paths. Add near-limit local-root and selected-remote path fixtures.

### F7 — Medium — Terminal reason collapses usage, ordinary I/O, and sink failure into corruption

**Affected code:** `storage_terminal_reason` in `crates/codefire-cli/src/storage.rs:405`; sink override
in `StorageReportRun::publish_to_writer` at `crates/codefire-cli/src/storage.rs:127`.

The mapping recognizes only resource exhaustion and sealed-subject change. Every other error becomes
`CorruptSubject`. That includes invalid remote syntax (`CliError::Usage`), missing/permission I/O,
bounded business rejection, JSON/render plumbing errors, and output sink failure. ADR-020 explicitly
states that a business-policy rejection which completes bounded decision work is not corruption
(`adr/ADR-020-decision-equivalent-work-budget.md:38`). Existing HTTP decision-work handling already
distinguishes bounded completion from corrupt-subject outcomes.

**Concrete counterexample:** `storage --remote not-a-cf-url` performs a bounded parse and returns a
usage diagnostic, but its ledger snapshot says `corrupt_subject`. A broken stdout pipe is also
recorded as corruption of the repository subject.

**Required repair:** define and test a typed terminal mapping by semantic error class and phase.
Reserve `CorruptSubject` for actual subject-integrity failures; map bounded usage/business completion,
subject change, budget exhaustion, and publication/sink outcomes consistently with the accepted
vocabulary and existing request handlers.

### F8 — Medium — `write_all` can publish a prefix, after which `main` can append a second JSON error

**Affected code:** `StorageReportRun::publish_to_writer` at
`crates/codefire-cli/src/storage.rs:103`; top-level error rendering at
`crates/codefire-cli/src/main.rs:186`.

The response is fully encoded before publication, but `writer.write_all(&bytes)` at
`crates/codefire-cli/src/storage.rs:125` is not transactional: the `Write` contract allows a writer
to accept a prefix and fail on a later call. The owner returns `CliError::Io`. In JSON mode,
`main` then calls `print_cli_error_json` at `crates/codefire-cli/src/main.rs:193`, with no state saying
stdout was already touched. The stream may therefore contain a truncated success envelope followed
by a generic error envelope, or merely a truncated success prefix. Neither is valid JSON and both
violate the packet's no-partial-stdout requirement.

**Concrete counterexample:** a writer accepts the first `N` bytes, returns an error on the next
write, and accepts later writes. `write_all` leaves the first `N` bytes visible; the command returns
an error; a later generic JSON error is appended. The focused test exercises a rejecting writer but
does not exercise a writer that first makes partial progress.

**Required repair:** make the supported publication boundary explicitly atomic or otherwise satisfy
the no-partial contract, and propagate publication state so top-level rendering cannot emit a second
stdout document after any storage bytes were attempted. Add a partial-progress failing writer test
for text and JSON plus a top-level JSON dual-output test.

### F9 — Should fix, release-blocking under zero-residual rules — Canonical command trace is stale

**Affected document:** storage row in `docs/development/command-source-trace.md:58`.

The canonical trace still describes production as `run_storage_report` followed by separate text or
JSON projection/printing helpers. The live route is
`main.rs` storage dispatch -> `prepare_storage_report(...)` -> one `StorageReportRun` ->
`publish(...)`, including encode, prepublication seal, write, release, and finish. `run_storage_report`
and several projection helpers are now retained primarily for tests/internal compatibility. The
stale row obscures the new ledger owner and publication boundary and violates the repository rule
that canonical source docs change with major command ownership changes.

**Required repair:** update the canonical trace to the exact live owner chain, including the
internal Explain consumer and its residual boundary. Run the canonical source-doc checker in the
repair lane.

## 5. Missing or vacuous tests

1. `legacy_json_bytes` at
   `crates/codefire-cli/src/tests/storage_decision_work.rs:148` calls the current production
   `storage_report_data_json`, `storage_report_diagnostics_json`, and
   `storage_report_next_actions`. A projection regression shared by production and the oracle passes
   both sides. The text oracle is independently implemented; the JSON oracle is not.
2. No test freezes portable logical byte formulas independently of `size_of::<T>()` or proves the
   same exact limit across ABI/compiler layouts.
3. No depth-zero or very-deep fixture detects the root off-by-one or recursive stack dependence.
4. No deterministic race changes a JSON file from ambient size `P` to reopened size `Q` and checks
   that `Q` is reserved before allocation/read.
5. No long/many-segment remote URL or percent-decoding boundary test proves parser allocations fit
   the remote reservation.
6. No near-limit local or remote path fixture measures all simultaneous `PathBuf` owners.
7. The many-candidate coverage beginning at
   `crates/codefire-cli/src/tests/storage_decision_work.rs:525` uses only 96 records and merely checks
   that peak retained state stays below 1 GiB. It does not put an exact boundary around duplicated
   JSON warning projections or path/parser peaks.
8. No writer accepts a prefix and then fails. The current publication test therefore proves only
   pre-write budget failure and an immediate writer failure, not absence of partial/dual stdout.
9. Symlink regression tests cover active-state behavior, but there is no focused quick-object or
   selected-remote symlink/capability fixture for the newly rewritten paths.
10. No test distinguishes a visitor-side error from a simultaneous Store finalization/cleanup error,
    and no terminal-reason matrix covers usage, ordinary I/O, business rejection, parse failure,
    subject corruption, subject change, budget failure, and sink failure.
11. The focused owner tests bypass top-level `main`, so they cannot observe the second JSON error
    envelope described in F8.

## 6. Ownership, counters, cleanup, and complexity audit

| Surface | Static result |
|---|---|
| Command owner | Accepted structure: the live route at `crates/codefire-cli/src/main.rs:1583` creates one `StorageReportRun`; the run owns one `DecisionWorkLedger`, report state, retained count, encoding, seal, publication, release, and terminal transition. |
| Nested ledgers | No nested production storage ledger was found. Local object scanning and selected-remote object scanning receive the same mutable command ledger, including Store scanner callbacks. |
| Counter charging | Discovery/read/parse/output counters are broadly present, but F1, F4, F5, and F6 leave platform-dependent or uncharged bytes/owners. The counters therefore cannot yet be treated as complete decision-equivalent observations. |
| Retained/temporary cleanup | Normal success and the covered budget/malformed failure paths generally release report, map/vector duplication, parse scratch, and output ownership to zero. No confirmed scalar debt leak was found. Hidden allocations in F2/F4/F5/F6 are outside those scalar balances, so a zero snapshot is not proof of zero actual debt. |
| Error precedence | Confirmed terminal-classification defect is F7. Visitor errors are returned before Store scan finalization errors at `crates/codefire-cli/src/storage.rs:1673` and `crates/codefire-cli/src/storage.rs:2461`; masking of a real production cleanup/accounting failure remains unresolved rather than confirmed. |
| Publication | Covered budget failures occur before the first write. F2 shows projection allocations before adequate preflight, and F8 shows that sink failure can still expose partial/dual output. |
| Largest-object top-N | Accepted: production retains at most ten largest objects and uses stable comparison/tie behavior. No `O(n)` largest-object retained list or legacy tie-order drift was found. |
| Other aggregation | Type/generation maps are proportional to distinct output groups and are converted with explicit duplicate-vector accounting, but the accounting unit is ABI-dependent under F1. Warning output remains proportional to reported warnings by legacy contract. |
| Traversal complexity | File work is linear in visited entries, but F3 adds `O(depth)` process call-stack memory and permits stack failure before the advertised 100,000 logical-depth budget. Remote URL parsing is linear in input size but its `O(segments + bytes)` allocation is not input-proportionally reserved (F5). |
| Legacy order | Accepted: the candidate restores the true pre-ledger visit order `objects -> remotes -> active -> idempotency`. `40c095de...` itself contains an intermediate ordering regression; the candidate does not preserve that intermediate mistake. Warning order, object-type order, and top-N tie order otherwise remain stable in reviewed normal paths. |
| Quick/full behavior | No intentional quick/full semantic drift was found. Focused and legacy evidence cover ordinary quick/full bytes, but the missing symlink, deep-tree, race, and parser/path boundaries prevent closure. |
| Capability/symlink safety | The no-follow descriptor-relative direction is defensible and no direct symlink-follow regression was confirmed. Coverage is incomplete for quick object and selected-remote paths, so this is an accepted implementation property with a required test gap, not a full safety authorization. |

## 7. Accepted properties and evidence that remains useful

- The candidate introduces a coherent owned `StorageReportRun` rather than scattered release calls.
- Production local and remote scanners share one command ledger; no reset-per-remote or reset-per-phase
  ledger multiplication was found.
- Bounded largest-object state remains top ten with stable comparator behavior.
- The reviewed order matches the actual pre-ledger command order, not the intermediate recovery
  precursor's drift.
- Covered resource failures occur before normal output publication, and covered success/error paths
  generally reach a terminal snapshot with zero concurrent retained/temporary counters.
- The separately recorded finisher evidence reports 779/779 binary tests, 83/83 remote tests,
  133/133 HTTP tests, 11/11 focused storage decision-work tests, four exact legacy storage tests,
  source-doc validation, formatting, strict Clippy, and diff checks on the recorded toolchains. See
  `docs/development/ai/swarm-results/cfr168-rust-storage-finisher-recovery-claude-20260730/result.md:105`.
- That evidence record explicitly marks the final combined remote/HTTP/source-doc/build command as
  interrupted and not evidence
  (`docs/development/ai/swarm-results/cfr168-rust-storage-finisher-recovery-claude-20260730/result.md:165`).
  Passing execution evidence does not exercise or disprove the static counterexamples above.

## 8. Explicitly unresolved questions

These are not promoted to candidate findings without a proven live failure, but they should remain
visible for the repair review:

1. `object_record_failed_semantic_validation` in
   `crates/codefire-cli/src/storage.rs:1782` recognizes a semantic-validation outcome through error
   context string equality. Current Store producers appear to use the expected text, but a typed
   Store signal would be more robust and would prevent a wording change from altering storage
   warning/failure semantics.
2. Object and remote visitor errors are returned before `scan_result` at
   `crates/codefire-cli/src/storage.rs:1673` and `crates/codefire-cli/src/storage.rs:2461`. This can
   theoretically mask a Store finalization, rollback, or accounting error. No production-triggerable
   release failure was proven statically, so this remains an error-precedence question rather than a
   confirmed defect.
3. `explain storage-warning` finishes the storage ledger before Explain projection/publication at
   `crates/codefire-cli/src/explain.rs:286`. That is a predecessor-gated cross-command combined-owner
   residual, not a defect assigned to this narrowly selected storage/report publication candidate.

## 9. Residual and ownership boundary

This decision covers only the selected Rust `storage` / `storage report` decision-work surface and
its exact owner/callee/test changes. It does not close or authorize:

- CFB-136 session/root-proof closure
- Python parity
- HTTP, Doctor, ancestry, or other command adoption
- capability advertisement or all-command metrics coverage
- integration, issue Close, Install, Dogfood, or Retrospect

Those predecessor-gated or separately owned residuals must remain mapped to their canonical owners.
Conversely, F1 through F9 are defects or required documentation/test repairs in this candidate's own
surface and must not be deferred to those residuals.

## 10. Review-only validation

The independent lane used only read-only Git/text inspection before writing this report: commit and
ancestor verification, exact blob comparison, baseline/focused name and stat diffs, and targeted
`rg`/`sed`/`nl` source and document reads. It intentionally did not execute the product.

The report commit is validated only with:

- `git diff --check`
- `git diff --name-only`
- `git status --short`

No Cargo/build/install/demo evidence is created or claimed by this review lane.

## 11. Required next action

Produce a repaired Rust storage candidate that resolves F1 through F9 and adds the non-vacuous
boundary/race/publication tests listed in section 5. Then run the authorized implementation gate in
that repair lane and request a fresh independent adversarial review against the exact repaired commit.

This report **does not authorize integration, Close, Install, or Dogfood**.
