# CFR-168 Rust Storage Exact-Candidate Independent Rereview

Date: 2026-07-31
Reviewer lane: Fugu Ultra 1.1 (`fugu-ultra-v1.1`) report-only rereview
Decision: **REQUEST_CHANGES**

OVERALL_VERDICT: REQUEST_CHANGES

## 1. Exact review identity

The immutable identity checks passed before substantive review. This report reviews the cumulative two-commit repair range, not only the final commit.

| Item | Verified value | Result |
|---|---|---|
| Exact review subject | `cdcb9bdefe7d25a118a9dc14171864d75f5ad304` | Match |
| Subject tree | `1836a83aa42df026f578b134f4b455830b873a3f` | Match |
| Subject parent / durable checkpoint | `e9e42aae9eb3c5a0993ca07688ade5d4d6e0eed3` | Match |
| Frozen repair task revision | `13767c7f39eca82d4a9093a055abf568814cfef0` | Match |
| Repair-task tree | `d053752a48a8a1de15c7d3b16a8b2d7331920523` | Match |
| Rejected predecessor | `b3bc32a6bd4c792edc5a1302c61e16c624a9b648` | Match |
| Prior independent report commit | `4cfb607cfd6ac95d26d24a4542ac15ba56b428d6` | Match |
| Prior report integrated revision | `4380985d392d5c3f0866c35cf789fada9f872f58` | Match |
| Reviewer-start revision | `844e4a319e458e6be2507ab8b166640a74d98c85` | Match |
| Required final report commit parent | `844e4a319e458e6be2507ab8b166640a74d98c85` | Fixed before report commit |
| Exact range patch SHA-256 | `a6fab7cbb36d89be52dc4d400a2d84987351888642b8ce481fdf78b51b6487a4` | Match |
| Repair task Markdown SHA-256 | `b24faadc5293893fa74fd601ca019358d151a5df2654fdad27214d240b908b59` | Match |
| Prior report Markdown SHA-256 | `628a37f51cad4710eeccb8c9b34286b9e41332a59a421068a6a1aaa61ed4140a` | Match |
| Candidate commit-message SHA-256 | `3e58dbc35c9cc4a4dd24f8d8c3bbdcb95fdca372d9ebbe460d99184b6b15e07f` | Match |

The exact range is:

1. `e9e42aae9eb3c5a0993ca07688ade5d4d6e0eed3` — `wip(storage): repair F1-F8 storage decision-work residuals`;
2. `cdcb9bdefe7d25a118a9dc14171864d75f5ad304` — `fix(cfr168): close F1-F9 Rust storage zero-residual residuals`.

`git rev-list --count 13767c7f39eca82d4a9093a055abf568814cfef0..cdcb9bdefe7d25a118a9dc14171864d75f5ad304` returned `2`, and the frozen task revision is an ancestor of the candidate.

The range contains exactly eight changed paths and `2,972 insertions(+), 291 deletions(-)`:

| Path | Verified candidate blob |
|---|---|
| `crates/codefire-cli/src/main.rs` | `a1981193eace2e3bd21d3d122a4397bd9ab95916` |
| `crates/codefire-cli/src/secure_worktree.rs` | `dacdf244e3de78ba04411bd0ff2e79c4d9038341` |
| `crates/codefire-cli/src/storage.rs` | `c22ba1900feaea95bfe8be427a4b66c0a0578258` |
| `crates/codefire-cli/src/tests/storage_decision_work.rs` | `6f8b5a8c65ef3ff44230f81cc31bd65a7baf9a12` |
| `docs/development/change-playbook.md` | `1bdd04f102c14323e0e2ba44d03253e070e2033d` |
| `docs/development/command-source-trace.md` | `c7093219c449d304fbb63b67a403e0383bcc6068` |
| `docs/development/source-architecture.md` | `d632dd5cffb51e4b16e07eb3ee4ff7018637d4e3` |
| `docs/development/source-data-model-catalog.md` | `21feaeb9cecdc05d9405b4e49e096ed1b313bd25` |

The manager's clean-worktree and successful implementation-session observations establish handoff identity only. They are not correctness evidence.

## 2. Decision

**REQUEST_CHANGES.** The candidate materially improves the rejected predecessor: it introduces one storage command ledger, iterative capability traversal, pinned bounded reads, projection and response reservations, tracked publication state, focused boundary tests, and updated canonical documents. The independently rerun focused gates pass.

Acceptance is nevertheless prohibited because direct source evidence contradicts required closure properties:

- local traversal charges phantom `GraphNodes`, does not charge local `GraphEdges`, and omits live traversal container/authority owners;
- a visitor error can discard a distinct Store scanner/finalization result in release builds;
- publication failure is recorded as terminal `Complete`, and a failure after a completed write/flush is not wrapped as `PublishedOutputFailed`, permitting a second JSON document;
- path, projection, URL, and derived-path formulas remain partly platform-dependent, blanket, structurally mismatched, or non-vacuously unproved;
- pinned-path replacement/final parent authority, required terminal/writer sequences, independent reversals, and numeric 1x/3x/10x executions remain missing;
- two canonical documents overstate the implemented formula and precedence guarantees.

Passing tests and checkers do not override those contradictions. Category-3 implementation-session claims, including the claimed full binary run and claimed numeric curves, were not converted into acceptance evidence.

## 3. Severity-ordered findings

### R1 — High — Local traversal counters do not represent the traversed graph

**Location:** `crates/codefire-cli/src/storage.rs:1883`, `crates/codefire-cli/src/storage.rs:2026`, `crates/codefire-cli/src/storage.rs:2039`, `crates/codefire-cli/src/storage.rs:3002`.

**Production path:** `main::run_storage_report_command` -> `storage::prepare_storage_report` -> `prepare_storage_report_with_ledger` -> local area/object scan -> `scan_files` -> `scan_capability_files`.

**Counterexample:** `scan_files` charges the root once. Every traversal-loop iteration then charges one `GraphNodes` unit before `ReadDir::next`. The EOF iteration for every live directory therefore consumes a node even though `next()` returns `None`. A tree with `D` directories and `E` directory entries is charged as `1 + E + D`, not as the logical node set. Conversely, no local child edge is charged anywhere in `scan_capability_files`; `GraphEdges` is charged only once per selected remote URL at `storage.rs:3002`.

**Violated invariant:** graph nodes and edges must be input-derived logical work, exact at `L`, rejected at `L - 1`, and increased by one only when the subject adds one logical unit. An EOF probe is not a node, and a local parent-child relation is an edge.

**Why tests do not close it:** the focused boundary tests derive candidate values from the candidate snapshot and then replay those values. That proves internal self-consistency, not correctness of the graph oracle. The selected-remote `GraphEdges` test does not exercise local traversal edges.

**Required repair:** move node charging after `Some(entry)` is established, define whether the traversal root and each entry are nodes, charge every local child edge before opening/pushing the child, add an independent hand-derived flat/tree oracle, and test exact `L`, `L - 1`, and one additional subject node/edge at fixed `L`.

### R2 — High — Visitor cancellation can hide a distinct Store result in release builds

**Location:** `crates/codefire-cli/src/storage.rs:2560`, `crates/codefire-cli/src/storage.rs:3434`; shared callee behavior at `crates/codefire-store/src/lib.rs:3513`, `crates/codefire-store/src/lib.rs:3616`, and `crates/codefire-store/src/lib.rs:3672`.

**Production path:** full local or selected-remote object scan -> Store scanner callback -> callback stores a `CliError` and returns `StoreError::ValidationCancelled` -> Store performs finalization/revalidation/release -> storage receives `scan_result`.

**Counterexample:** when `visitor_error` is present, storage checks `scan_result` only with `debug_assert!` and unconditionally returns the visitor error. The Store scanner can return a different finalization, authority, or accounting error; Store release logic explicitly allows a release error to replace the scan result. In optimized builds the assertion is absent, so that distinct result is silently discarded.

**Violated invariant:** cleanup/finalization and primary-error precedence must be explicit, typed, deterministic, and identical in debug and release builds. A debug-only assertion is not a release contract.

**Required repair:** replace both `debug_assert!` branches with an exhaustive match over `(visitor_error, scan_result)`, define precedence for cancellation sentinel versus distinct Store errors, preserve both errors when necessary, and add injected Store-finalization/release-error tests for local and remote callbacks in release-equivalent behavior.

### R3 — High — The publication ledger reports `Complete` for failed publication

**Location:** `crates/codefire-cli/src/storage.rs:267`, `crates/codefire-cli/src/storage.rs:278`, `crates/codefire-cli/src/tests/storage_decision_work.rs:1919`.

**Production path:** `StorageReportRun::publish_to_writer` -> encode -> `seal_prepublication` -> `write_all_tracked` -> sink failure -> `abort`.

**Counterexample:** the committed partial-publication test asserts that the failed publication snapshot has `terminal_reason == Some(DecisionWorkTerminalReason::Complete)`. The external error is `PublishedOutputFailed`, but the single ledger observation says the command completed.

**Violated invariant:** publication/sink failure must have the publication-failure terminal class, one terminal observation, and balanced ownership. A completed decision-work terminal cannot simultaneously represent a failed externally visible outcome under the packet's required terminal matrix.

**Required repair:** defer the terminal `Complete` observation until publication and postpublication cleanup succeed, or introduce a sealed-but-not-terminal state that can still transition once to publication failure. Update tests to require the publication-failure reason rather than `Complete`.

### R4 — High — A post-write finish error can append a second JSON document

**Location:** `crates/codefire-cli/src/storage.rs:290`, `crates/codefire-cli/src/storage.rs:291`, `crates/codefire-cli/src/main.rs:204`.

**Production path:** all bytes accepted -> flush succeeds -> publication state becomes `Completed` -> `finish_complete_in_place()` fails -> error returns without `PublishedOutputFailed` -> `main::top_level_error_rendering` sees an ordinary error.

**Counterexample:** in JSON mode, a release/finish/accounting error after the complete success document was flushed is not wrapped. `top_level_error_rendering` therefore selects `JsonEnvelopeOnStdout` for the ordinary error and can append a second JSON document after the already visible success document.

**Violated invariant:** after any write attempt that may have become visible, top-level handling must never emit a second stdout document. `Completed` is more visible, not less visible, than `PartiallyProgressed`.

**Required repair:** wrap every error after the first attempted write, including post-flush release and ledger finish failures, in `CliError::PublishedOutputFailed`; add a deterministic finish-failure seam and process-level text/JSON channel assertions.

### R5 — High — Traversal container and authority owners remain outside the portable peak

**Location:** `crates/codefire-cli/src/storage.rs:1994`, `crates/codefire-cli/src/storage.rs:2080`, `crates/codefire-cli/src/storage.rs:2127`.

**Production path:** `scan_capability_files` -> `Vec<StorageTraversalFrame>` -> `push_traversal_frame` -> exhausted-frame identity fence.

**Counterexample:** the implementation charges iterator scratch, handle frames, display paths, and retained child names, but it does not charge the traversal stack's own `Vec` capacity or an explicit portable shell for each stored `StorageTraversalFrame`. The temporary descriptor/handle opened for the post-visit identity fence is also not represented as a distinct live authority owner before it is opened.

**Violated invariant:** every frame/frontier capacity, descriptor, identity owner, path, and per-frame state must be acquired before allocation/open and included in the simultaneous heap peak.

**Required repair:** define an ADR-backed portable frontier entry formula, acquire stack growth before `Vec` capacity expansion, acquire identity-reopen authority before opening, and add omission-sensitive tests that fail when each owner is removed.

### R6 — High — Live storage owners are allocated before the command ledger exists

**Location:** `crates/codefire-cli/src/storage.rs:1349`, `crates/codefire-cli/src/storage.rs:1366`, `crates/codefire-cli/src/storage.rs:1387`, `crates/codefire-cli/src/storage.rs:1416`, `crates/codefire-cli/src/storage.rs:1431`.

**Production path:** CLI dispatch -> `parse_storage_report_args` -> clone `--remote` strings and construct the path -> `prepare_storage_report` -> create ledger.

**Counterexample:** an arbitrarily long remote argument is cloned into `StorageReportOptions.remotes`, and a path is materialized as a `PathBuf`, before the storage decision-work ledger is created and before the later 16 KiB adapter rejection.

**Violated invariant:** every storage-owned input string/path allocation on the live command path must be covered before allocation. Moving a later bound in front of the shared URL parser does not cover the earlier CLI clone.

**Required repair:** create/pass the command ledger before storage option ownership is materialized, or parse into borrowed spans and acquire exact owners before cloning.

### R7 — Medium — The logical owner schema is not the frozen ADR-020 contract and is not portable for paths

**Location:** `crates/codefire-cli/src/storage.rs:31`, `crates/codefire-cli/src/storage.rs:59`, `crates/codefire-cli/src/storage.rs:1132`, `docs/development/source-data-model-catalog.md:117`.

**Counterexample:** the candidate generalizes the ADR's `256 + utf8(node_id)` traversal/frontier unit into one 256-byte shell for every semantic Rust owner. Embedded scalar aggregates receive a frame while real collection capacity and some authority/container owners do not. Separately, `Path::as_encoded_bytes()` is platform encoding, not exact UTF-8 output. On Unix an invalid single byte has encoded length one, while lossy text display emits the three-byte UTF-8 replacement character and JSON path serialization may fail instead of matching the charged payload.

**Violated invariant:** formulas must be representation-independent, tied to the frozen logical contract, dominate every actual encoded owner, and avoid arbitrary false public-limit cliffs.

**Required repair:** define logical semantic records and collection/frontier entries in ADR-020 terms; use an explicit portable path-output contract for text and JSON; separately account failure diagnostics; and remove blanket application of a frame unit where the ADR does not define one.

### R8 — Medium — JSON projection has a structural field-count defect hidden by fixed slack

**Location:** `crates/codefire-cli/src/storage.rs:920`, `crates/codefire-cli/src/storage.rs:1000`, `crates/codefire-cli/src/storage.rs:3571`.

**Counterexample:** `object_file_json` emits four string fields: `object_id`, `type`, `type_confidence`, and `path`. `storage_json_projection_bytes` passes `string_fields = 3` while summing all four payloads. With the candidate constants, this omits one fixed string-field allowance (`66` bytes) per largest-object node, up to `660` bytes for the hard-bounded top ten. The global `64 * 128` fixed allowance happens to hide that error in current fixtures.

**Violated invariant:** the projection formula must structurally dominate field selection, formatting, escaping, and duplication; acceptance cannot depend on unrelated slack.

**Required repair:** derive field overhead from an independent schema/table with four string fields, replace broad fixed envelopes with reviewed structural counts, and add a mutation test that removes each field allowance and fails at the intended projection owner.

### R9 — Medium — URL accounting is a blanket multiplier and the 16 KiB proof is false

**Location:** `crates/codefire-cli/src/storage.rs:2954`.

**Production path:** selected remote -> `storage_remote_url_parse_bytes` -> acquire temporary -> unchanged `remote::parse_cf_project_url`.

**Counterexample:** the formula reserves `segment_vector + 12 * (256 + url_bytes + 16 KiB)` regardless of actual simultaneous lifetimes. Its justification says every project root is at least as long as the URL, but percent-decoding can reduce `%41` from three input bytes to one decoded byte. A URL can therefore exceed 16 KiB while producing a shorter project path that remains inside the path envelope. The oversized argument is returned as `InvalidRepository`, which classifies user input as subject corruption.

**Violated invariant:** parser ownership must be tied to actual simultaneously live owners, and a new public input limit must preserve the prior supported domain or be explicitly accepted as a product change.

**Required repair:** expose a shared parser plan/ledger contract or independently enumerate exact segment/vector, decoded bytes/strings, server/suffix, root, and returned-value lifetimes; remove the false length implication; classify bounded argument rejection as usage/business completion; and test long percent-encoded shrinking inputs plus exact/one-unit limits.

### R10 — Medium — Local/remote path-owner counts are not exact simultaneous lifetimes

**Location:** `crates/codefire-cli/src/storage.rs:109`, `crates/codefire-cli/src/storage.rs:3035`, `crates/codefire-cli/src/tests/storage_decision_work.rs:1599`.

**Counterexample:** the local constant claims five simultaneous derived paths, while source order retains `.codefire` and creates the area joins sequentially; the focused test explicitly acknowledges that removing the local reservation may not make the probe fail because another temporary owner dominates. On the remote route the broad URL-parse reservation remains live while the six-owner derived-path reservation is acquired, double-counting some project-root/join envelopes without proving exact overlap.

**Violated invariant:** simultaneous-owner peaks must follow actual lexical/live ownership, and tests must independently detect omission of each owner.

**Required repair:** shorten scopes or enumerate exact path values and clone/intermediate lifetimes, charge before each first join, and add omission-sensitive limits for local and remote owners separately.

### R11 — Medium — Typed terminal classification remains non-exhaustive and text-dependent

**Location:** `crates/codefire-cli/src/storage.rs:718`, `crates/codefire-cli/src/storage.rs:730`, `crates/codefire-cli/src/storage.rs:776`, `crates/codefire-cli/src/storage.rs:789`, `crates/codefire-cli/src/storage.rs:2690`.

**Counterexample:** `storage_terminal_class` has a wildcard arm that maps any future `CliError` according only to phase. Pinned subject change is recognized by searching I/O message text. Object semantic-validation classification still depends on `IncompleteRecord` context text rather than a dedicated Store variant.

**Violated invariant:** the terminal matrix must be typed and exhaustive; future variants and localized/contextual strings must not silently change corruption, subject-change, I/O, or publication semantics.

**Required repair:** remove the wildcard, use explicit typed variants for pinned changes and semantic validation, and make compilation fail when a new relevant error class is added.

### R12 — Medium — Pinned reads do not prove final path/parent authority

**Location:** `crates/codefire-cli/src/storage.rs:3485`, `crates/codefire-cli/src/secure_worktree.rs` (`PinnedBoundedFile`), `crates/codefire-cli/src/tests/storage_decision_work.rs:1473`.

**Counterexample:** pin-before-size/charge/read is established, and in-place growth is rejected. However, the replacement race test expects success by reading the old pinned inode after the directory entry is replaced. No committed focused test establishes truncation, parent replacement, or final descriptor-plus-parent/path-entry revalidation.

**Violated invariant:** the packet requires typed outcomes for in-place growth, inode replacement, truncation, parent replacement, and final descriptor/parent authority, not merely agreement between the pinned descriptor size and bytes read.

**Required repair:** define whether path replacement is a subject change, revalidate descriptor and parent/path binding accordingly, and add deterministic tests for truncation and parent replacement with exact owner cleanup.

### R13 — Medium — Publication tests do not exercise required writer/channel sequences

**Location:** `crates/codefire-cli/src/tests/storage_decision_work.rs:1881`, `crates/codefire-cli/src/tests/storage_decision_work.rs:1948`.

**Counterexample:** committed tests cover a prefix-then-error writer and a first-write rejection through an internal test API. They do not inject `WriteZero`, repeated `Interrupted`, short-write completion, flush failure, or post-flush finish failure, and they do not spawn the top-level route to observe actual stdout and stderr bytes.

**Violated invariant:** the one-visible-outcome claim must be non-vacuous at every Rust `Write` boundary and through the real top-level channel decision.

**Required repair:** add scripted writer traces for every sequence and process-level text/JSON tests that assert exact stdout/stderr contents and one document.

### R14 — Should fix, release-blocking under zero-residual rules — Canonical docs overstate implemented guarantees

**Location:** `docs/development/change-playbook.md:331`, `docs/development/source-data-model-catalog.md:117`.

**Counterexample:** the playbook describes primary-error/cleanup precedence as guaranteed although the visitor/Store result is debug-asserted and discarded in release builds. The data-model catalog presents the generalized 256-byte owner shell as the portable contract although ADR-020 freezes the frame unit for traversal/frontier semantics, not every Rust owner.

**Violated invariant:** canonical source documents must describe exact live behavior and must not convert intended invariants into current-state claims.

**Required repair:** correct the implementation first, then update both documents to the exact typed precedence and ADR-backed formulas. Structural checker success is not a semantic waiver.

## 4. F1-F9 disposition

| Finding | Disposition | Direct evidence | Evidence class | Blocking reason |
|---|---|---|---|---|
| F1 portable formulas | **OPEN** | Source inspection; portable-golden tests rerun | Independent source proof plus executed candidate tests | pre-ledger input owners, platform path encoding, generalized 256-byte shell, and omitted collection/authority owners |
| F2 projection/response | **OPEN** | Independent `reference_json_bytes` inspected; focused projection/response tests rerun | Executed candidate tests plus independent source proof | four emitted largest-object strings are declared as three; broad fixed slack hides mismatch |
| F3 traversal/depth/order | **OPEN** | root/depth/deep/symlink tests rerun; live loop inspected | Executed candidate tests plus independent source proof | phantom nodes, absent local edges, uncharged stack capacity/reopen authority |
| F4 pinned read | **OPEN** | race/growth/replacement/exact-temp tests rerun | Executed candidate tests plus independent source proof | replacement is accepted through old inode; truncation, parent replacement, final path authority unproved |
| F5 URL owners | **OPEN** | URL bound test rerun; adapter and shared parser inspected | Executed candidate test plus independent source proof | blanket 12-owner formula, false 16 KiB proof/cliff, corruption classification |
| F6 path owners | **OPEN** | local/remote owner tests rerun; scopes inspected | Executed candidate tests plus independent source proof | local probe is acknowledged non-omission-sensitive; counts do not match exact lifetimes |
| F7 terminal matrix | **OPEN** | terminal matrix test rerun; classifier and Store scanner inspected | Executed candidate test plus independent source proof | debug-only precedence, wildcard, message matching, Store semantic text dependency |
| F8 publication | **OPEN** | partial/reject/preflight tests rerun; top-level route inspected | Executed candidate tests plus independent source proof | failed publication terminal is `Complete`; completed-publication finish error can emit second JSON; required writer traces absent |
| F9 canonical truth | **OPEN** | both document checkers rerun; four docs compared to source | Executed structural checks plus independent source proof | playbook and data-model catalog overstate live guarantees |

No F1-F9 item has an executable predecessor failure or a reviewer-performed temporary reversal in this rereview. The required minimum reversal matrix was therefore not independently satisfied. This report does not relabel source proof as an executable reversal.

## 5. Production call paths and ownership boundaries

### Storage command

`main` dispatches storage at `crates/codefire-cli/src/main.rs:1617`:

`parse_storage_report_args` -> `prepare_storage_report` -> one `DecisionWorkLedger` -> `prepare_storage_report_with_ledger` -> local scans -> selected-remote scans using the same ledger -> one `StorageReportRun` -> encode/project -> reserve exact response -> seal -> tracked write/flush -> release report/response -> finish.

This is the correct intended command-owner shape. The findings concern missing owners, incorrect formulas/counters, and terminal/publication transitions inside that shape.

### Internal Explain consumer

`crates/codefire-cli/src/explain.rs:285` calls `run_storage_report`, which reaches `StorageReportRun::into_report_for_internal_consumer`. The storage ledger is released and finished before `explain_storage_warning` allocates its own `json!` data, diagnostics, next actions, and final Explain response. Explain's projection/publication is therefore not owned by the finished storage ledger.

### Shared callees reviewed but not writable in this lane

- `crates/codefire-store/src/lib.rs`: object-store scanner, finalization/revalidation, retained release, and `StoreError` taxonomy.
- `crates/codefire-cli/src/remote.rs`: `parse_cf_project_url` and `remote_dirs` allocation/layout behavior.
- `crates/codefire-core`: `DecisionWorkLedger`, profile, dimensions, counters, terminal reasons, sealing, and finish contracts.

## 6. Portable logical formulas

The following table reconstructs the candidate formula. It does not endorse every formula as correct.

Definitions:

- `F = 256` (`LOGICAL_FRAME_BASE_BYTES`);
- `U = 20` (widest `u64` text);
- `B = 5` (widest boolean text);
- `C = 2` (empty collection text);
- `s(x) = UTF-8 byte length of string x`;
- `p(x) = platform `as_encoded_bytes()` length of path x` (portability defect R7);
- `Str(x) = F + s(x)`;
- `Path(x) = F + p(x)`.

### Retained owners

| Owner | Candidate formula | Simultaneous lifetime / audit verdict |
|---|---|---|
| `StorageReport` fixed fields | `F + 15U + B + 5C = 571` | retained from preparation through response release; generalized frame is disputed |
| report repository root | `Path(repo_root)` | retained with report; path-output dominance is not portable |
| object scan mode | `Str(mode)` | retained with report |
| skipped check | `Str(value)` each | quick route; retained with report and duplicated in JSON projection |
| warning | `(F + U) + Str(kind) + (F + message_bytes) + optional Path(path)` | retained to publication; each warning duplicated in JSON diagnostics |
| object type | `(F + 2U) + Str(type_tag) = 552 + s(type_tag)` | map/report owner; vector conversion can temporarily duplicate the logical entry |
| largest object | `(F + U) + Str(object_id) + Str(type_tag) + Path(path)` | up to ten retained; borrowed `type_confidence` is excluded from retained formula but emitted in projection |
| remote base | `F + 19U + B + 2C + Str(url) + Str(status) + Path(project_root)` | retained per selected remote |
| missing remote path | `Path(path)` each | retained in remote report |
| remote optional timestamp | `Str(value)` each for oldest/updated values | retained while present |
| remote generation | `F + 3U = 316` each | map entry retained; vector conversion duplicates entries temporarily |
| exact encoded response | `R = exact Vec/string bytes` | acquired as retained response ownership before construction; live with retained report until publication cleanup |

### Temporary and authority owners

| Owner | Candidate formula | Simultaneous peak / audit verdict |
|---|---|---|
| local derived paths | `5 * (F + p(codefire_root) + 32)` | acquired before joins; count is not actual simultaneous lifetime and local test is non-vacuous only for presence, not omission |
| URL parser adapter | `2 + 40*segments + 12*(F + url_bytes + 16,384)` | held across selected-remote parse and scan; blanket and false-cliff defects |
| remote derived paths | `6 * (F + p(project_root) + 32)` | overlaps URL reservation; exact overlap not derived |
| root traversal frame | handle `F` + display `Path(root)` + iterator `4,096` | stack `Vec` capacity and frame shell are omitted |
| child traversal frame | handle `F` + display `Path(child)` + iterator `4,096` + retained `Path(name)` | same omissions |
| directory-entry scratch | `(F + 4*255) + F = 1,532` | acquired before `next`; node charge incorrectly occurs at same point |
| child display path | `F + p(parent) + 1 + p(name)` | acquired before `PathBuf::with_capacity`; capped at 16 KiB |
| identity-reopen authority | intended handle `F` | descriptor/open owner is not explicitly acquired before reopen |
| pinned JSON buffer | `Q = pinned_bytes + 1` | acquired before same-descriptor read |
| pinned JSON parse peak | `2Q + min(Q, 65,536) + 4,096` | raw buffer + parser-sized owner + parser buffer + base |
| object-type vector duplicate | `C + count * ObjectTypeFormula` | acquired while ordered map still owns entries |
| generation vector duplicate | `C + count * 316` | acquired while ordered map still owns entries |
| text formatting | `4*(F + 32) + F + sum(s(skipped)+2)` | four formatted-size strings plus joined skipped checks |
| JSON node | `50 + 84*scalar_fields + 66*string_fields + 6*payload_bytes` | checked arithmetic; largest-object call underdeclares one string field |
| JSON fixed structure | `64 * 128 = 8,192` | broad allowance, not structural proof |
| JSON remote fixed structure | `32 * 128 = 4,096` each | broad allowance, not structural proof |
| JSON next action | `1,024` | fixed allowance when warnings exist |

### Simultaneous route peaks

Let `Rreport` be all retained report owners, `Twalk` the maximum live iterative traversal owners, `Tparse` the maximum pinned parse scratch, `Tlocal` the local derived-path reservation, `Turl` the selected URL adapter, `Tremote` the remote derived-path reservation, `J` the JSON projection upper, `H` the text formatter upper, and `E` exact response bytes.

| Route/phase | Retained current | Temporary current/peak | Verdict |
|---|---|---|---|
| local quick prepare | report fixed + repo + mode + skipped checks + warnings | `Tlocal + max(Twalk, other area scratch)` | quick skips full record detail; local path and traversal formulas remain open |
| local full prepare | quick base without skipped owners + object types + top ten + warnings | `Tlocal + max(Twalk, Tparse, vector duplicate)` | map/vector/top-ten retained lifetime is bounded; traversal formula is incomplete |
| selected remote prepare | local retained + accumulated remote reports/warnings/generations | `Turl + Tremote + max(remote walk, pinned parse, vector duplicate)` | both broad reservations overlap; not an exact owner peak |
| JSON projection | `Rreport` | `J` | projection is acquired before `json!`; structural upper is defective |
| JSON response construction | `Rreport + E` | projection owner remains part of the intended simultaneous proof | exact output reservation is tested; projection formula is not accepted |
| text response construction | `Rreport + E` | `H` | text formatter formula is explicit; non-UTF-8 path display can exceed `p(path)` |
| publication | `Rreport + E` until cleanup | projection/format scratch released | write failures release current bytes in focused tests |
| successful finish | `0` | `0` | focused tests assert byte debt zero; permit debt is not independently asserted |
| prepublication failure | cleanup intends `0` | cleanup intends `0` | normal JSON error envelope remains permitted |
| attempted/partial publication failure | cleanup tests reach `0` | cleanup tests reach `0` | terminal reason incorrectly remains `Complete` |
| completed publication then finish error | uncertain external outcome | cleanup failure path | second-document blocker R4 |

## 7. Representative 1x/3x/10x curves

No independent numeric 1x/3x/10x execution was performed in this report-only lane. The implementation-session numbers are not reviewer evidence. The table below is a static derivation from the committed flat-object fixture constructor and production formulas, with that limitation explicit.

Define one scale unit as one additional flat object record under the existing object root, with authority bytes `b`, stable type `blob`, object-id length `i`, and path encoded length `p`. Let `G0`, `A0`, `Q0`, `F0`, `T0`, `J0`, and `E0` denote the unchanged repository/base-route values. Define:

- `OT = 552 + len("blob")` for the one retained type entry;
- `OF = 1,044 + i + len("blob") + p` for one retained largest-object entry;
- `JO` as the corrected JSON projection contribution for one largest object (the candidate currently undercounts its fixed part by `66`);
- `K(n) = min(n, 10)`.

| Scale | Candidate local graph-node delta | Local edge delta | Depth/frontier | Aggregate delta | Full record delta | Quick prepared retained | Full prepared retained | JSON projection/response | Final debt |
|---|---:|---:|---|---:|---:|---|---|---|---|
| 1x | `+1` entry, with directory EOF phantom already in base | `0` (defect) | flat: unchanged | `+b` | `+1` | `Q0` apart from scalar-width changes/warnings | `F0 + OT + OF` | `J0 + JO`; `E0 + encoded object contribution` | expected `0/0`, not independently curve-executed |
| 3x | `+3` | `0` (defect) | flat: unchanged | `+3b` | `+3` | `Q0` apart from scalar-width changes/warnings | `F0 + OT + 3OF` | `J0 + 3JO`; linear encoded growth | expected `0/0`, not independently curve-executed |
| 10x | `+10` | `0` (defect) | flat: unchanged | `+10b` | `+10` | `Q0` apart from scalar-width changes/warnings | `F0 + OT + 10OF` | `J0 + 10JO`; linear encoded growth | expected `0/0`, not independently curve-executed |

For a selected-remote scale of `n` independent URLs, dispatch adds `n` candidate `GraphNodes` and `n` candidate `GraphEdges`; retained remote reports and JSON output grow linearly with `n`, while URL parse temporary is sequential per URL and should be a maximum, not a sum across URLs. Inside each remote, local directory traversal still has the phantom-node/missing-edge defects.

The missing independent numeric execution is itself acceptance-blocking under the packet. The static curve also demonstrates why copied implementation-session numbers would be misleading: candidate graph dimensions are not the intended graph dimensions.

## 8. Exact boundary matrix

| Dimension/owner | Work exactly at `L` | Same work at `L - 1` | One additional subject unit at fixed `L` | Independent verdict |
|---|---|---|---|---|
| `GraphNodes` | candidate-derived snapshot can pass | focused test can reject | not established against intended node oracle | **Open:** EOF is charged as node |
| `GraphEdges` local | zero/absent rather than logical edges | no meaningful local boundary | added local child does not charge edge | **Open** |
| `GraphEdges` selected URL dispatch | focused test passes candidate value | rejects at candidate value minus one | extra URL behavior not independently rerun | partial only |
| `LogicalDepth` | exact depth passes | a tree one edge deeper at fixed limit fails | executed | closed for depth semantics only |
| `MaximumFrontier` | deep fixture observes bounded heap frontier | constrained by same traversal profile | no independent width-growth fixture | partial |
| `AggregateBytes` | focused object test passes measured `L` | rejects at `L - 1` | extra-byte subject at fixed `L` not independently isolated | partial |
| `ObjectRecords` | focused full-scan test passes measured `L` | rejects at `L - 1` | one additional record not independently isolated | partial |
| counter values / response bytes | exact encoded count passes | `L - 1` rejects at response counter | one extra encoded byte not independently forced | partial |
| prepared retained peak | self-measured peak passes | peak minus one rejects | added-owner target can be masked by other peaks | **Open** formula/non-vacuity |
| JSON projection temporary | self-measured peak passes | peak minus one rejects | additional field/owner mutation not run | **Open** structural formula |
| exact response retained owner | response preflight exact value passes | one less rejects before publication | added response byte not independently forced | partial |
| pinned read temporary | `2Q + min(Q,65536) + 4096` passes | one less rejects | in-place growth is rejected after pin | ordering established; authority matrix open |
| URL public byte limit | 16 KiB accepted by focused helper test | not a ledger-owner boundary | 16 KiB + 1 rejected | **Open:** false public cliff and wrong class |
| local derived paths | formula-visible reservation passes | removing/count omission is not guaranteed to fail | not executed | **Open, vacuous omission test** |
| remote derived paths | six-owner candidate formula is observed | candidate test detects lower envelope | exact real simultaneous owner count not proved | **Open** |

All focused budget failures inspected by the test helper assert typed `ResourceBudget`, limit/observed values, exit code `60`, terminal `BudgetExceeded`, and zero retained/temporary current bytes. The helper does not assert zero permits.

## 9. Traversal, ordering, descriptor authority, and complexity

### Traversal and ordering

- The live capability traversal is iterative: `scan_capability_files` owns a heap `Vec<StorageTraversalFrame>` and has no direct or indirect recursive descent.
- Root logical depth is observed as zero. A child is checked at parent depth plus one before open/push.
- The focused empty-root, exact-depth, one-edge-too-deep, symlink, and 4,096-level deep tests passed.
- The deep fixture demonstrates heap iteration without process-stack recursion. It remains bounded by the 16 KiB display-path envelope; short components make the fixture feasible, so it does not prove behavior for paths beyond that product bound.
- Descriptor-relative no-follow opens and exhausted-directory identity fences remain present.
- DFS visit behavior and quick/full route selection are preserved by the iterative structure, but counter defects prevent acceptance of the claimed graph curve.
- Largest-object state remains hard-bounded at ten. The focused many-candidate test passes the committed stable-order/tie behavior. Type output remains ordered by `BTreeMap`.

### Descriptor and authority matrix

| Window | Candidate behavior | Reviewer verdict |
|---|---|---|
| no-follow open | `PinnedBoundedFile::open` occurs before size use | established by source and rerun race test |
| charge before allocation/read | `pinned + 1` buffer and parse scratch acquired before `read_pinned` | established |
| in-place growth | read cannot exceed charged bound; growth is typed subject change | established by rerun test |
| same-path inode replacement | old pinned descriptor is read and test expects success | does not establish final path binding |
| truncation | no dedicated committed focused result | missing |
| parent replacement | no dedicated committed focused result | missing |
| final descriptor/parent revalidation | not demonstrated for all windows | open |
| cleanup | exact-temp and growth tests assert zero retained/temporary current bytes | byte debt established for tested windows; permit debt unproved |

### Complexity

| Operation | Time | Heap | Audit note |
|---|---|---|---|
| iterative traversal | `O(entries + directories)` filesystem steps | `O(depth + live path bytes)` intended | stack capacity and identity-reopen owners are uncharged |
| area aggregation | `O(files)` | `O(1)` outside report owners | checked scalar accumulation |
| object type aggregation | `O(records log types)` | `O(types)` | map/vector duplicate formula exists |
| top ten | `O(records * K log K)` with fixed `K=10`, therefore linear | `O(K)` | stable bounded test passed |
| URL parse | `O(url bytes + segments)` | `O(url bytes + decoded/root owners)` | candidate charge is blanket, not exact lifetime |
| JSON projection/encoding | `O(report fields + encoded bytes)` | retained report + projection + response | largest-object fixed undercount |
| text encoding | `O(report fields + encoded bytes)` | retained report + formatter + response | lossy path expansion not dominated by encoded path length |
| publication | `O(response bytes + write calls)` | exact response buffer | post-write terminal/finish defect |

## 10. Terminal and publication matrices

### Typed terminal matrix

| Error class | Candidate terminal class/reason intent | Exit/diagnostic delegation | Top-level channel before write | Debt evidence | Verdict |
|---|---|---|---|---|---|
| bounded usage/business completion | `BoundedUsageCompletion` / complete bounded outcome | original typed usage/business exit and diagnostic | JSON envelope on stdout in JSON mode; stderr otherwise | focused matrix only | wildcard-free typing still required |
| ordinary I/O | `OrdinaryIo` | original I/O/generic exit | JSON envelope before write; stderr in text | focused matrix only | explicit variants required |
| parse/render failure | subject JSON -> corruption; encoding JSON -> `RenderFailure` | original render/JSON exit | prewrite normal top-level route | projection tests assert byte debt zero | phase split present |
| actual subject integrity failure | `SubjectCorrupt` / `CorruptSubject` | original repository/Store diagnostic | normal prewrite route | malformed tests assert byte debt zero | semantic text dependency remains |
| subject change | `SubjectChanged` | original subject-change diagnostic | normal prewrite route | growth test asserts byte debt zero | I/O message matching is not typed |
| ledger budget exhaustion | `LedgerBudgetExhausted` / `BudgetExceeded` | `ResourceBudget`, exit `60` | prewrite JSON envelope or stderr | helper asserts retained/temp zero | established for tested boundaries |
| host resource exhaustion | `HostResourceExhausted` | original host-resource exit | normal prewrite route | classifier test only | explicit list exists, wildcard remains |
| publication/sink failure | `PublicationFailure` intent | inner I/O exit delegated through `PublishedOutputFailed` | stderr only after attempted write | retained/temp zero in two writer tests | ledger actually remains terminal `Complete` |

`storage_terminal_class` ends in a wildcard, so this table is not compile-time exhaustive. Current tests prove selected examples, not future-variant safety. No focused assertion demonstrated zero open permits across the whole matrix.

### Publication-state and observable-output matrix

| State/window | Exact write sequence | Returned error/output policy | Evidence | Verdict |
|---|---|---|---|---|
| `NotAttempted` | encoding/preflight fails before first `write` | ordinary typed error; JSON top level may emit one error envelope | rerun preflight test, output Vec empty | established |
| `AttemptedWithoutProgress` | first writer call errors or accepts zero | must be `PublishedOutputFailed`; stderr only | first-write rejecting writer rerun | internal route only; `WriteZero` not injected |
| `PartiallyProgressed` | one or more bytes accepted, then error; or flush fails | `PublishedOutputFailed`; no second stdout document | prefix-then-error writer rerun | flush-failure and process channel missing |
| retry after `Interrupted` | writer returns `Interrupted`, loop retries | state remains possibly visible according to prior progress | source proof only | no executable trace |
| short writes to completion | repeated positive writes, then flush | complete document | source proof only | no scripted executable trace |
| flush failure | all bytes accepted, flush errors | partial/maybe visible and stderr only | source proof only | no executable trace |
| `Completed` + successful finish | all bytes and flush succeed; release/finish succeeds | one success document | ordinary focused success tests | byte debt zero in test snapshots |
| `Completed` + finish failure | all bytes and flush succeed; release/finish fails | candidate returns ordinary error | source proof | **blocker:** second JSON may be appended |

The partial and rejecting writer tests inspect accepted bytes and internal `top_level_error_rendering`, but they do not capture an actual process's stdout/stderr. Their publication snapshots also expect terminal `Complete`, contradicting the required publication-failure terminal reason.

## 11. Independent oracle and reversal evidence

### Independent JSON oracle

`reference_json_bytes` in `crates/codefire-cli/src/tests/storage_decision_work.rs:156` independently constructs the reference envelope and does not call production projection-size or envelope helpers. The quick/full JSON and text byte-preservation tests were rerun and passed. This is valid output-oracle evidence.

The projection boundary tests are not independent formula oracles: they call the production projection helper or use a candidate-observed peak. The independent serializer therefore proves current bytes, but it does not prove the production upper-bound formula or owner placement.

### F1-F9 reversal classification

| Item | Executable predecessor failure | Reviewer temporary reversal | Independent source proof | Result |
|---|---|---|---|---|
| F1 remove ABI/layout dependence | not run | not run | yes; current replacement still has portability/schema gaps | open |
| F2 reverse fixed scratch/projection | not run | not run | yes; four-vs-three string mismatch | open |
| F3 restore recursion/root-one behavior | not run | not run | iterative/root-zero tests executed; counter/owner contradiction inspected | open |
| F4 restore ambient-size/reopen read | not run | not run | pin ordering executed; final authority gaps inspected | open |
| F5 restore fixed remote scratch | not run | not run | input formula inspected and URL test executed | open |
| F6 omit local/remote path owner | not run | not run | source lifetime proof; local test admits omission may survive | open |
| F7 restore catch-all/precedence defect | not run | not run | wildcard/debug-assert source proof | open |
| F8 restore untracked write/top-level duplicate | not run | not run | tracked writer tests executed; completed-finish duplicate source proof | open |
| F9 restore stale trace | not run | not run | semantic docs/source comparison and checkers | open |

The required independent executable reversals were not performed. The implementation session's claimed reversals are category-3 leads only and are not reproduced evidence in this report.

## 12. Canonical source documents and prior unresolved questions

### Canonical-document verdict

| Document | Verdict |
|---|---|
| `docs/development/command-source-trace.md` | broadly matches dispatch -> prepare -> one run -> encode/seal/publish/release/finish and the top-level channel split |
| `docs/development/source-architecture.md` | broadly matches one local/remote ledger and the internal-consumer boundary |
| `docs/development/change-playbook.md` | **stale/overstated:** claims cleanup/error precedence not guaranteed in release behavior |
| `docs/development/source-data-model-catalog.md` | **stale/overstated:** presents the generalized 256-byte owner formula as canonical |

Both source/document checkers pass structurally. F9 remains open because semantic truth is stricter than inventory/link consistency.

### Three prior unresolved questions

1. **Typed object semantic-validation signaling — valid cross-command handoff, not globally closed.** Exact owner: `codefire_store::StoreError` and the object-record scanner in `crates/codefire-store/src/lib.rs`, at reviewed revision `cdcb9bdefe7d25a118a9dc14171864d75f5ad304`. Reason outside storage candidate: `codefire-store` is not one of the eight writable candidate paths. Storage still interprets `IncompleteRecord` context at `crates/codefire-cli/src/storage.rs:2690`; the shared Store owner must expose a dedicated typed semantic-validation signal before global CFR-168 closure can be claimed.
2. **Visitor error versus Store finalization/error precedence — storage-local blocker.** Exact owner: local and remote adapter branches in `crates/codefire-cli/src/storage.rs:2560` and `crates/codefire-cli/src/storage.rs:3434`. A debug-only assertion does not close release behavior.
3. **Explain combined storage-report/projection/publication ownership — valid cross-command handoff.** Exact owner: `explain::explain_storage_warning` in `crates/codefire-cli/src/explain.rs:285`, at revision `cdcb9bdefe7d25a118a9dc14171864d75f5ad304`. Reason outside the storage command ledger: `StorageReportRun::into_report_for_internal_consumer` releases/finishes storage ownership before Explain constructs and publishes its own result. This report accepts the boundary only; it does not claim Explain's combined ownership is solved.

## 13. Validation evidence, resources, and limitations

Pinned environment:

```text
RUSTUP_HOME=/home/devuser/.cache/codefire-rust-1.97/rustup
CARGO_HOME=/home/devuser/.cache/codefire-rust-1.97/cargo
CARGO_BUILD_JOBS=2
RUST_TEST_THREADS=2
CARGO_INCREMENTAL=0
CARGO_TARGET_DIR=/workspace/.cache/cargo-target-shared
rustc 1.97.0 (2d8144b78 2026-07-07)
cargo 1.97.0 (c980f4866 2026-06-30)
host x86_64-unknown-linux-gnu
```

| Command | Reviewer result | Counts / duration | Resource note |
|---|---|---|---|
| identity, ancestry, path/blob, digest, source, and initial `git diff --check` reads | PASS | exact values in section 1 | read-only |
| `cargo fmt --all -- --check` | PASS | about 5 s | serialized gate |
| `cargo test -p codefire-cli --locked storage_decision_work -- --nocapture` | PASS | 33 passed, 0 failed, 768 filtered; test body 3.05 s; reviewer command about 173 s including build/lock wait | serialized shared target |
| `cargo test -p codefire-cli --locked storage -- --nocapture` | PASS | 41 passed, 0 failed, 760 filtered; 2.45 s | serialized shared target |
| `python3 docs/development/tools/check_source_docs.py` | PASS | 44 modules, 34 commands, 12 object kinds | no Cargo |
| `python3 docs/development/tools/check_project_docs.py` | PASS | 538 Markdown files inventoried | no Cargo |
| package strict Clippy | PASS | `cargo clippy -p codefire-cli --all-targets --locked -- -D warnings`; 112 s reviewer duration, Cargo reported 1m49s | preflight about 11.97 GiB available, zero swap, 1.69 GiB shared target, 903 GiB free |
| candidate-tree `git diff --check` | PASS | no output | read-only |

Final recorded resource observation after the gates was approximately 12 GiB available memory, zero swap use, 0.79 GiB shared target size, and 904 GB free workspace disk. The values changed as caches were reclaimed; every heavy gate began above the packet thresholds.

Two early shell preflight attempts exited before Cargo because of reviewer command-construction mistakes. They are not validation failures and provided no correctness evidence.

Not run, by constraint:

- Docker, install, `demo.sh`, dogfood, network-dependent commands;
- unrestricted test discovery, full workspace tests, and full binary suite;
- implementation-session claimed 801/801 binary tests;
- reviewer predecessor execution, temporary reverse mutation, and numeric 1x/3x/10x execution;
- product commands outside the focused test harness.

Implementation-session claims of 33/33, 41/41, 801/801, reversals, and 1x/3x/10x curves were treated only as leads. Only the commands in the table are independent reviewer executions.

## 14. Debt evidence and integration overlap

The focused helper `assert_zero_concurrent_debt` checks current `RetainedBytes == 0` and `TemporaryBytes == 0`. It is used on the tested success, budget, malformed-record, pinned-growth, projection, and publication failure routes. These are real byte-debt checks.

The following remain unproved:

- zero permit debt across every terminal/publication window;
- zero debt for completed-publication finish failure, Store finalization conflict, truncation, parent replacement, flush failure, `WriteZero`, and repeated `Interrupted`;
- actual owner completeness where allocations were never acquired into the ledger.

Integration overlap is intentionally minimal:

- the candidate commits are ancestors of reviewer-start `844e4a319e458e6be2507ab8b166640a74d98c85`;
- the reviewer-start commit adds only the frozen rereview task;
- this lane adds only this report on top of that revision;
- no product, test, canonical source document, issue state, management ledger, manifest, or prior report is modified;
- no integration, Close, Install, Dogfood, or Retrospect action is authorized or performed.

## 15. Exact required next action

Keep CFR-168 open and do not integrate `cdcb9bdefe7d25a118a9dc14171864d75f5ad304` as accepted. Produce a new exact candidate that, at minimum:

1. corrects local graph node/edge charging and owns traversal stack capacity plus identity-reopen authorities;
2. creates the ledger before storage-owned CLI input allocation and replaces platform path accounting with an ADR-backed portable output contract;
3. replaces blanket projection/URL/path formulas with exact simultaneous-owner derivations and corrects the four-string largest-object projection;
4. defines and tests final descriptor/parent/path authority for replacement, truncation, and parent swaps;
5. implements typed, exhaustive terminal classification and explicit visitor-versus-Store precedence in release builds;
6. records publication failure as failure, wraps every post-attempt finish error, and adds `WriteZero`, `Interrupted`, short-write, flush, post-finish, and process-level channel tests;
7. adds independent omission-sensitive reversals and independently executed numeric 1x/3x/10x curves;
8. updates all four canonical documents to the repaired live truth and reruns the constrained gates;
9. carries the two genuine cross-command handoffs explicitly: typed Store semantic validation and Explain-owned projection/publication.

The next reviewer must verify a newly frozen identity and cumulative range; this report does not authorize repairing or amending the reviewed commit in place.
