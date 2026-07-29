# CFR-168 / CFB-136 Current-Snapshot Acceptance-Evidence Audit

## Verdict

**Not close-ready.** Snapshot `e2a218e015953f082245f36dbc6aee906b837e5a` contains substantial accepted
implementation slices and strong focused evidence for Trace equivalence, Store subject-change safety,
branch-list reuse, bounded HTTP failure, closure-only upload staging, and iterative sealed/signature
traversal. It does not yet satisfy the complete ADR-020/CFR-168 adoption gate. Two production ancestry
families and the Rust storage command remain outside the command/request ledger contract; the required
cross-surface boundary, scale, parity, metrics, installed-runtime, Dogfood, Close-review, and Retrospect
evidence is incomplete or not bound to this exact snapshot. The canonical `investigating` / “Implement in
progress” status is therefore consistent with the audited source.

## Snapshot and Method

- Audited immutable revision: `e2a218e015953f082245f36dbc6aee906b837e5a`.
- `git rev-parse HEAD` returned that exact revision and the worktree was clean before this report.
- Contract sources read: `adr/ADR-020-decision-equivalent-work-budget.md`,
  `docs/development/code-review-issues/cfr-168.md`, `docs/development/bug-issues/cfb-136.md`, and
  `docs/development/ai/task-packets/cfr-168-decision-work-budget.md`.
- Evidence was accepted only after reading test bodies and the production call paths they exercise. Test
  names alone were not treated as proof.
- Focused Rust tests were invoked with package/target-qualified Cargo commands and fully qualified harness
  names. Earlier exploratory commands that used a bare substring with `-- --exact` and ran zero tests are
  explicitly excluded. Two guessed Python method/class selectors also failed at unittest loading and are
  excluded; the corrected exact selectors below are the evidence.
- Broader source gates in the first matrix are manager-provided results for this immutable snapshot. This
  lane reran only bounded focused tests plus the capability probe.

## Gate Matrix

| Gate | Exact command / evidence | Result on `e2a218e` | Audit disposition |
|---|---|---:|---|
| Snapshot identity | `git rev-parse HEAD` | `e2a218e015953f082245f36dbc6aee906b837e5a` | Pass |
| Rust format | `PATH=/tmp/codefire-rustup/cargo/bin:$PATH cargo fmt --all -- --check` | Manager reports success | Source gate pass; not lifecycle proof |
| Rust Clippy | `PATH=/tmp/codefire-rustup/cargo/bin:$PATH cargo clippy --workspace --all-targets --locked -- -D warnings` | Manager reports success | Source gate pass |
| Rust workspace | `PATH=/tmp/codefire-rustup/cargo/bin:$PATH cargo test --workspace --locked` | Manager reports success | Broad regression gate pass |
| Project docs | `python3 docs/development/tools/check_project_docs.py` | Manager reports success | Pass |
| Source docs | `python3 docs/development/tools/check_source_docs.py` | Manager reports success | Pass |
| AI development aggregate | `python3 docs/development/tools/check_ai_development.py` | Manager reports failure only in `runtime` | Not a CFR-168 product-close failure; acceleration-control gap |
| AI runtime subgate | `python3 docs/development/tools/validate_ai_runtime.py --json` | Manager reports missing `codex_cli_0_146` adapter | Acceleration-control evidence only |
| Runtime capability | `PATH=/tmp/codefire-rustup/cargo/bin:$PATH python3 docs/development/tools/probe_validation_capabilities.py` | `READY rust_source`, `READY rust_binary_smoke`, `BLOCKED rust_current_source_binary`; installed `codefire` absent | Fresh Install/Dogfood cannot be credited |
| Core required-link equivalence | `cargo test -p codefire-core --lib tests::trace_index_required_links_match_reference_on_graph_edge_cases -- --exact` | 1 passed | Strong semantic/order evidence |
| Core indexed scan equivalence | `cargo test -p codefire-core --lib tests::indexed_scan_matches_reference_for_duplicate_self_cycle_and_diamond -- --exact` | 1 passed | Strong semantic/order evidence |
| Core Trace scale | `cargo test -p codefire-core --lib tests::trace_work_counters_have_linear_1x_3x_10x_slope -- --exact` | 1 passed | Explicit Rust 1x/3x/10x curve |
| Store mutation safety | `cargo test -p codefire-store --lib tests::sealed_validation_session_detects_extension_only_same_metadata_rewrite -- --exact` | 1 passed | Strong CFB-136 nucleus evidence; Unix fixture |
| Store shared DAG reuse | `cargo test -p codefire-store --lib tests::sealed_validation_session_reuses_many_head_shared_dag_and_revalidates_once -- --exact` | 1 passed | Strong fixed-size reuse/final-fence evidence |
| Store snapshot boundary | `cargo test -p codefire-store --lib tests::sealed_validation_snapshot_revalidation_budget_accepts_exact_and_stops_plus_one -- --exact` | 1 passed | Exact/+1 proven for final snapshot records |
| Branch command reuse/metrics | `cargo test -p codefire-cli --bin codefire-rs tests::branch_list_uses_one_session_for_many_head_shared_dag_and_reports_aggregate_work -- --exact` | 1 passed | Strong one-ledger, one-session, aggregate-counter evidence |
| Branch final tamper | `cargo test -p codefire-cli --bin codefire-rs tests::branch_list_finalization_rejects_same_inode_extension_only_tamper -- --exact` | 1 passed | Strong no-stale-hit evidence; Unix fixture |
| Doctor reuse | `cargo test -p codefire-cli --bin codefire-rs doctor::checks::tests::doctor_scan_and_candidates_share_one_accounted_recoverable_session -- --exact` | 1 passed | Strong but limited to the sealed object/candidate slice |
| HTTP no partial success | `cargo test -p codefire-cli --bin codefire-rs http::tests::http_response_wire_rejects_limit_plus_one_without_partial_200 -- --exact` | 1 passed | Typed failure and no partial `200` proven in memory |
| HTTP upload no mutation | `cargo test -p codefire-cli --bin codefire-rs http::tests::upload_budget_failure_is_typed_413_and_has_no_effects -- --exact` | 1 passed | Strong pre-effect authority evidence |
| HTTP closure-only staging | `cargo test -p codefire-cli --bin codefire-rs http::tests::upload_staging_ignores_unreachable_existing_sentinels -- --exact` | 1 passed | 512 unreachable objects excluded |
| Rust signature depth/order | `cargo test -p codefire-cli --bin codefire-rs signatures::tests::iterative_commit_policy_handles_deep_history_in_preorder -- --exact` and `cargo test -p codefire-cli --bin codefire-rs signatures::tests::iterative_commit_policy_keeps_parent_order_first_error -- --exact` | 2 passed | Iteration and first-error order proven |
| Rust generic common ancestor depth | `cargo test -p codefire-cli --bin codefire-rs tests::common_ancestor_handles_deep_generation_graph -- --exact` | 1 passed | Stack independence only; the production helper is unbudgeted |
| Rust Store depth/order | `cargo test -p codefire-store --lib tests::iterative_sealed_validation_handles_deep_linear_history -- --exact` and `cargo test -p codefire-store --lib tests::iterative_sealed_validation_preserves_first_parent_error -- --exact` | 2 passed | Iteration and first-parent error proven |
| Python Trace/ancestor/tamper/upload | `python3 -m unittest -v tests.test_codefire_cli.CodeFireCliTests.test_python_trace_index_1x_3x_10x_has_linear_counters tests.test_codefire_cli.CodeFireCliTests.test_python_common_ancestor_prefers_generation_before_public_id tests.test_codefire_cli.CodeFireCliTests.test_python_sealed_session_detects_same_size_same_mtime_tamper tests.test_codefire_cli.CodeFireCliTests.test_python_http_upload_stages_only_reachable_existing_closure` | 4 passed in 3.584s | Focused parity/safety evidence |
| Python HTTP/ordering/depth | `python3 -m unittest -v tests.test_codefire_cli.CodeFireCliTests.test_python_http_bundle_deep_shared_dag_is_iterative tests.test_codefire_cli.CodeFireCliTests.test_python_trace_index_duplicate_occurrence_order_matches_rust tests.test_codefire_cli.CodeFireCliTests.test_python_sealed_validation_is_iterative_for_deep_chain tests.test_codefire_cli.CodeFireCliTests.test_python_signature_history_is_iterative_for_deep_chain` | 4 passed in 9.861s | HTTP 1x/3x/10x, duplicate order, and iterative depth evidence |
| Python signature scale | `python3 -m unittest -v tests.test_python_signature_owner.AccountedCommitSignaturePolicyTests.test_one_three_and_ten_node_curves_charge_actual_work` | 1 passed in 0.056s | Explicit Python signature 1/3/10 curve, with loose byte upper bounds |

Focused rerun total: **17/17 Rust exact tests and 9/9 corrected Python exact tests passed**.

## Acceptance Boundary Audit

| ADR-020 / CFR-168 boundary | Coverage found | Classification |
|---|---|---|
| Indexed required-link and Trace semantic equivalence | Rust reference/indexed tests cover duplicate IDs, duplicate links, self-loop, cycle, and diagnostic/order equality; Python duplicate occurrence order matches the Rust vector | **Satisfied for the focused Trace slice** |
| 1x/3x/10x counter order | Explicit curves exist for Rust Trace, Python Trace, Python HTTP bundle, and Python signature policy. No literal three-point curve was found for Store final revalidation, branch-list many-head reuse, Doctor, Rust signature, Rust object transfer, or generic Rust/Python ancestry | **Partial** |
| Limit-minus-one / exact / plus-one | Many ledger, memory, parser, HTTP, Store, branch, Doctor, and publication fixtures exist; focused proof confirms Store final snapshot exact/+1 and HTTP response +1. The source does not present a systematic matrix proving all three points for every advertised dimension and every selected production surface. `graph_edges`, logical depth/frontier, aggregate/file bytes, and final snapshot bytes are especially distributed across generic or isolated tests | **Partial** |
| Cache/result/diagnostic equivalence | Core reference comparisons are exact. Branch metrics-on/off preserve domain output, and Store/branch shared-DAG tests show reuse. There is no one common cross-surface cold/warm corpus covering branch, storage, Doctor, and ancestry together | **Partial** |
| Store subject mutation invalidates reuse | Same-path/same-inode/same-size/same-mtime digest mutation is detected by Store and local branch finalization; Python same-size/same-mtime finalization is also covered | **Satisfied for audited Store/branch/Python sessions** |
| Immutable shared DAG validates once | Store 32-head session and local branch many-head fixture demonstrate unique read reuse plus one final revalidation | **Satisfied for Store and local branch list** |
| Branch / storage / Doctor / ancestry share a command-scoped validated subject | Local branch list and the audited Doctor object-scan/candidate slice do. Rust storage still performs independent unaccounted scans. Generic ancestry helpers remain outside the session/ledger path | **Implementation gap** |
| Iterative traversal and first-error ordering | Rust Store and signature tests cover 20,000-deep histories and parent order; Python sealed/signature tests cover 1,100-deep histories; Python HTTP bundle covers a deep shared DAG | **Strong but not sufficient for all ancestry paths** |
| HTTP no-partial / no-mutation | Rust response overflow yields typed 413 without a partial `200`; upload budget failure leaves nonce, branch, object, transaction, and GC authorities absent; closure-only staging ignores 512 unreachable existing records. Python has matching bounded-bundle and closure fixtures | **Satisfied for focused routes** |
| Typed local/HTTP diagnostic parity | Stable Rust diagnostic JSON and Python typed exit-60 tests exist, and HTTP typed 413 paths are covered. No single shared corpus compares every required field for the same local and HTTP failure across both runtimes | **Partial** |
| Rust/Python semantic parity | Matching vectors exist for generation tie-break, Trace duplicate order, sealed/signature depth, and HTTP work. No single versioned corpus proves common ancestor, required-link, sealed/signature, object transfer, and HTTP together | **Partial** |
| Stable counters and metrics independent of semantics | Core profile/snapshot JSON shapes and branch-list metric projection are tested. CFR counters are not projected across every selected command; storage still exposes elapsed/count metrics outside `DecisionWorkLedger`. Wall-clock values are intrinsically variable | **Partial** |
| Installed reversal, full compatibility, Dogfood, Retrospect | No installed command or valid current-source provenance artifact is present. Tracked review batch files target older revisions, not this snapshot. No exact-snapshot Install/Dogfood/Retrospect artifact was found | **Missing runtime/lifecycle evidence** |

## Implementation Defects and Contract Gaps

These are source findings, not merely absent test artifacts.

1. **Rust storage scanning is outside the decision-work ledger/session contract.**
   `crates/codefire-cli/src/storage.rs:412` builds the full object scan with local counters and elapsed
   time; the non-quick path calls unaccounted
   `codefire_store::scan_object_store_record_documents` at `crates/codefire-cli/src/storage.rs:466`.
   Remote generation scanning repeats that API at `crates/codefire-cli/src/storage.rs:924`. The file has
   no `DecisionWorkLedger` or sealed validation-session owner. This contradicts ADR-020's command-scoped
   reuse requirement for branch/storage/Doctor/ancestry and leaves storage object reads, validation,
   retained ownership, final subject revalidation, and typed budget exhaustion outside the common
   vocabulary.

2. **Generic Rust ancestry is still unbudgeted and materializes the reachable map.**
   `crates/codefire-cli/src/remote.rs:3721` implements `is_ancestor_in_objects` as a full
   `ancestor_distances` map lookup. It is used by file-remote paths including
   `crates/codefire-cli/src/remote.rs:311`, `crates/codefire-cli/src/remote.rs:372`, and
   `crates/codefire-cli/src/remote.rs:2379`. The separate local merge helper
   `common_ancestor` at `crates/codefire-cli/src/main.rs:10756` builds a full left-side map through
   `ancestor_distances` at `crates/codefire-cli/src/main.rs:10767`. The deep common-ancestor test proves
   iteration, but not command-ledger accounting, exact boundaries, early bounded failure, or Store-session
   reuse for these production callers.

3. **Python production ancestry parity remains incomplete.**
   `codefire:11189` implements unaccounted `is_ancestor_from_objects`, and the existing-project HTTP upload
   fast-forward path calls it at `codefire:12205`. Accounted ancestry owners exist elsewhere, but this
   production decision bypasses them. Python local `common_ancestor` at `codefire:31698` likewise builds two
   unaccounted full ancestor maps. The generation tie-break vector establishes semantic parity only; it does
   not establish ADR-020 resource/budget parity.

4. **Metrics coverage is not command-complete.**
   `crates/codefire-cli/src/metrics.rs:61` projects detailed decision-work counters for `branch-list`, but
   `crates/codefire-cli/src/storage.rs:559` records storage elapsed/count metrics from the unaccounted scan.
   Stable profile/snapshot JSON is covered in `crates/codefire-core/src/work_budget.rs:2296`; that does not
   prove every selected public command emits the same aggregate counter contract or that all work is charged.

## Missing Runtime and Lifecycle Evidence

These gaps do not by themselves prove a product-code defect, but they block Close under the frozen task
packet and ADR.

- The capability probe reports Rust source testing and a smoke binary, but
  `rust_current_source_binary` is blocked because no clean-HEAD provenance artifact binds a release binary
  to `e2a218e`. `command -v codefire` is absent.
- No fresh isolated install, installed exact/+1 reversal, installed HTTP no-partial/no-mutation run, or
  installed 1x/3x/10x profile is recorded for this snapshot.
- No canonical Algorithm Dogfood, real-repository read-only matrix, or mutating workflow evidence is bound
  to this snapshot.
- The tracked Claude review batches use base revision
  `f1644122134d479333cf422fdb1d48b8fa92dd8e` and review candidate
  `aa5e57d8b1e22fb83792412c68f255ceaf603e39`; they are not approval evidence for `e2a218e`.
  `docs/development/ai/review-packets/` contains only its README, so no final Close review packet is tracked.
- No exact-snapshot Retrospect disposition was found. Canonical sources intentionally retain CFR-168 and
  CFB-136 as `investigating` and list capability/metrics, Install, Dogfood, and Retrospect as residual work.
- The `codex_cli_0_146` runtime-adapter failure belongs to the AI acceleration gate. It should be repaired for
  that control plane, but it must not be misreported as CFR-168 product-close evidence.

## Weak, Flaky, or Environment-Dependent Evidence

- Same-inode/same-metadata mutation fixtures are `#[cfg(unix)]` and depend on Unix inode and timestamp
  behavior. They are valuable counterexamples, not portable runtime proof.
- Several Doctor tests assert only that memo/revisit or work counters are nonzero/at least a threshold rather
  than fixing exact deltas. They prove sharing but are weaker as stable-counter goldens.
- The Rust HTTP no-partial fixture uses an in-memory `Cursor`. It proves rejection before encoded-body
  publication, but not reversal under a real socket short write or peer disconnect.
- Deadline and connection-admission suites depend on clocks, threads, and sockets. Their semantic checks are
  useful, but they remain more environment-sensitive than deterministic ledger limits.
- Deep fixed-size tests prove stack independence and ordering; absent a 1/3/10 delta assertion, they do not
  prove the accepted asymptotic counter slope.
- Python signature's 1/3/10 byte assertions use broad `<4x` and `<12x` upper bounds. Node and edge counts are
  exact, but the byte-slope proof is deliberately loose.
- Actual `elapsed_ms` metrics are nondeterministic. Stable shape tests that inject fixed durations validate
  serialization, not production timing stability.

## Close-Readiness Decision

The focused gates are strong enough to accept the implemented **slices**, not the root. Close requires, at
minimum:

1. account and session-bind Rust storage plus every generic Rust/Python production ancestry call;
2. add a mechanically complete dimension-by-surface limit-minus-one/exact/plus-one matrix;
3. add explicit 1x/3x/10x counter curves for the remaining selected Store, branch, Doctor, Rust signature,
   object-transfer, and ancestry surfaces, or document a narrower adopted surface set with owner approval;
4. run one shared Rust/Python conformance corpus for ancestor selection, required-link diagnostics, sealed and
   signature traversal, object transfer, and HTTP typed failure fields;
5. project stable CFR counters across every selected command and demonstrate metrics on/off semantic identity;
6. build a clean-HEAD provenance-bound binary for `e2a218e`, then complete fresh Install, installed reversal,
   Dogfood, independent High Close review, and Retrospect.

Until those conditions are met, the conservative acceptance decision is **REQUEST CHANGES / insufficient
evidence for CFR-168 and CFB-136 Close**.
