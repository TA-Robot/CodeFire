# CFR-168 Rust Storage Review Finding Impact/Actionability Adjudication

Date: 2026-08-02
Lane: independent impact/actionability adjudicator (Claude Opus 5, `claude-opus-5`)
Stage: Plan / Preflight, report-only. Lifecycle stage 9
(`docs/development/project-management.md` cycle table, Impact/Actionability
Adjudication).

This artifact adjudicates the 23 open items (F1-F9 and R1-R14) bound by the
immutable `REQUEST_CHANGES` rereview. It does not relabel that report, does not
fabricate acceptance, does not grant repair authority, and does not widen the
authorized write set. It is an input to a manager freeze.

No Cargo, `rustc`, `rustfmt`, Clippy, install, Docker, network, dogfood, test
binary, or product mutation was executed. Every product judgement below is
read-only Git/source proof, and every such judgement is labelled as source
proof rather than runtime reproduction.

---

## 1. Exact identity binding

Every supplied identity was treated as a claim and independently recomputed in
this fresh auto-worktree before analysis. All of them reproduce.

### 1.1 Launch revision

| Item | Recomputed value | Claim | Result |
|---|---|---|---|
| Launch HEAD | `6db7944b26399c3bc43867f12ee9b3f260988621` | `6db7944b26399c3bc43867f12ee9b3f260988621` | Match |
| Launch HEAD tree | `69b9f7414ef0104a5782493f8731e1cd3636795f` | not supplied; recomputed and bound | Bound |
| Launch HEAD subject | `docs(cfr168): correct remaining composition stage authority labels` | — | Bound |
| Worktree porcelain at launch | empty (clean, including untracked) | clean canonical | Match |

### 1.2 Immutable rejected product candidate

| Item | Recomputed value | Claim | Result |
|---|---|---|---|
| Candidate `C` | `cdcb9bdefe7d25a118a9dc14171864d75f5ad304` (type `commit`) | same | Match |
| `C` tree | `1836a83aa42df026f578b134f4b455830b873a3f` | same | Match |
| `C` parent | `e9e42aae9eb3c5a0993ca07688ade5d4d6e0eed3` | same | Match |
| `C` commit-message SHA-256 | `3e58dbc35c9cc4a4dd24f8d8c3bbdcb95fdca372d9ebbe460d99184b6b15e07f` | report §1 | Match |

### 1.3 Independent report

| Item | Recomputed value | Claim | Result |
|---|---|---|---|
| Report commit `R` | `71222e1cec1b0dc10003845428abfb611ba8bf71` (type `commit`) | same | Match |
| `R` tree | `d43b5a1da31aba03943cdfe6c38c17fd5438511b` | repair packet | Match |
| `R` parent | `844e4a319e458e6be2507ab8b166640a74d98c85` | reviewer-start | Match |
| Report path | `docs/development/ai/swarm-results/cfr168-rust-storage-zero-residual-rereview-fugu-20260730/review.md` | same | Match |
| Report blob | `cd0ffc8458d8240ac7a0215a365cf32b5b91e7da` | same | Match |
| Report file SHA-256 | `6438df445b2d05510895fd23f73a4e71ba49238da15b00e86abf1a477b5ab960` | same | Match |
| Report lines / bytes | 562 lines / 54,661 bytes | 54,661 bytes (packet) | Match |
| Report verdict | `REQUEST_CHANGES` (`OVERALL_VERDICT: REQUEST_CHANGES`) | same | Match, preserved |

The identical report blob `cd0ffc84…` is also present at launch HEAD at the
same path, so the canonical lineage carries the immutable review bytes even
though commit `R` itself is not an ancestor of launch HEAD (§1.6).

### 1.4 Cumulative patch digests and exact changed path set

Two ranges exist and both are bound, because the prompt names the single-parent
range while the immutable report reviewed the two-commit cumulative range.

| Range | Commits | Digest of `git diff` | Shortstat |
|---|---:|---|---|
| `e9e42aa..cdcb9bd` (prompt's `C` parent → `C`) | 1 | `09bd6b8942d3b3ac52e22ccc24b36110e193b7d42a894c70f0134614b50c81f1` | 8 files, 1,536 insertions(+), 64 deletions(-) |
| `13767c7..cdcb9bd` (frozen repair-task revision → `C`; the reviewed range) | 2 | `a6fab7cbb36d89be52dc4d400a2d84987351888642b8ce481fdf78b51b6487a4` | 8 files, 2,972 insertions(+), 291 deletions(-) |

The reviewed-range digest `a6fab7cb…` reproduces the report §1 value exactly,
and `git rev-list --count 13767c7..cdcb9bd` returns `2`. The report's identity
section is therefore independently verified, not accepted on assertion.
Pre-range base tree `13767c7^{tree}` = `d053752a48a8a1de15c7d3b16a8b2d7331920523`.

Both ranges touch the **same exact eight paths**, with these candidate blobs:

| Path | Blob at `C` | Class |
|---|---|---|
| `crates/codefire-cli/src/main.rs` | `a1981193eace2e3bd21d3d122a4397bd9ab95916` | product |
| `crates/codefire-cli/src/secure_worktree.rs` | `dacdf244e3de78ba04411bd0ff2e79c4d9038341` | test seam only (`#[cfg(test)]`, +244 lines, pure addition) |
| `crates/codefire-cli/src/storage.rs` | `c22ba1900feaea95bfe8be427a4b66c0a0578258` | product |
| `crates/codefire-cli/src/tests/storage_decision_work.rs` | `6f8b5a8c65ef3ff44230f81cc31bd65a7baf9a12` | test |
| `docs/development/change-playbook.md` | `1bdd04f102c14323e0e2ba44d03253e070e2033d` | canonical doc |
| `docs/development/command-source-trace.md` | `c7093219c449d304fbb63b67a403e0383bcc6068` | canonical doc |
| `docs/development/source-architecture.md` | `d632dd5cffb51e4b16e07eb3ee4ff7018637d4e3` | canonical doc |
| `docs/development/source-data-model-catalog.md` | `21feaeb9cecdc05d9405b4e49e096ed1b313bd25` | canonical doc |

`crates/codefire-cli/tests/storage_report_publication.rs` does **not** exist at
`C`; `crates/codefire-cli/tests/` does exist (four other files), so the granted
new integration test is an ordinary auto-discovered addition.

### 1.5 Authorities read (complete, with binding hashes at launch HEAD)

| Authority | Blob at HEAD | File SHA-256 |
|---|---|---|
| `/workspace/worktrees/cfr168-p0-canonical/AGENTS.md` | `769bb13ce0fe2be6633a3f3e28593abf3356ff1b` | `df3412709c777399f0dc7a4927190e0e47c03c23930f4e81ab593dfdd2a7e8c1` |
| `docs/development/ai/operating-protocol.md` | `b5397d0c683a069673f687c5847dab9135c3c43c` | `0257b12aab81ad39df238075465a2750b4fa2973f6ab2d5a52c0124d06f9bc2b` |
| `docs/development/ai/validation-report-template.md` | `71c56ba0cc784124ef9ae4ad232186471c064ddf` | `2b6e1f38b499c1393a4cfb6c157a98bd7447da867046c7098ef875fa8bbf4ce0` |
| `docs/development/project-management.md` | `0a311b7dc3bf98b3a675ce578cd53d0a64df0771` | `fcee0efd3cea618ce6d119749e15792f85758edc7ddadb2df0078c467712ec3c` |
| `…/swarm-batches/cfr168-rust-storage-review-closure-repair-control-addendum-20260731.md` | `13bed7264431dd8b5c307efe5c3a0bfe7c7e231a` | `9a56e1ce681a05acfd8b1e8a4f13670587ec6bda8da3b73a00a8da0c71d8b113` |
| `…/swarm-batches/cfr168-rust-storage-review-closure-resume-preflight-20260731.md` | `e6238e18f9636b1c7403fee3db8973a4075dfb88` | `294b9948c790b8e84ada9a422949db4567b0979cd7e0402abe306af4205d3084` |
| `…/swarm-results/cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md` | `783a3def680140076a36c74d6b8659355cbf9fbd` | `2392450e957c8437b323ded492fc7597dc77f5147b74f3056c97c0efd9329780` |
| `…/swarm-results/cfr168-non-review-throughput-bottleneck-audit-claude-20260731/audit.md` | `100980f0e604764c873e0853e3431a35f2e98d76` | `f4e0f00014233f52ec7792accbc1dae61a6f0e7a888866d62e3be3b6405e4b3b` |
| `…/swarm-results/cfr168-rust-storage-zero-residual-rereview-fugu-20260730/review.md` (immutable report) | `cd0ffc8458d8240ac7a0215a365cf32b5b91e7da` | `6438df445b2d05510895fd23f73a4e71ba49238da15b00e86abf1a477b5ab960` |
| `adr/ADR-020-decision-equivalent-work-budget.md` (frozen product contract) | `9140c13cc9255bd1254f28fffeb3e6f806ea481b` | `4566de104181bc94f7d7fd41ae1b59b8c93b267665898f2629a3ff6c58ce5ab9` |

**Original repair packet.** The prompt named
`docs/development/ai/swarm-batches/cfr168-rust-storage-review-closure-repair-20260731.md`.
That path does **not** exist at launch HEAD. It was located in the object
database on the divergent packet lineage at commit
`926b179f2dcad8d4fabbeebaf8547f34a50b0c1b`, blob
`c5a7bca3ad56d1b9643d78c3db503caf761abea3`, SHA-256
`119ec9cf3055706e456c5a6090f32235e3db19260ee35f06e5814280643bec87` — which
reproduces the value the control addendum records for it. It was read in full
(387 lines) from that blob. `926b179` is **not** an ancestor of launch HEAD.
This is recorded as a live control-integrity gap in §8.

Candidate/parent/base source was read directly from the object database:
`storage.rs`, `main.rs`, `secure_worktree.rs`,
`tests/storage_decision_work.rs`, `crates/codefire-store/src/lib.rs`,
`crates/codefire-core/src/work_budget.rs`, `crates/codefire-cli/src/remote.rs`
at `C`; `storage.rs` at `e9e42aa` and at `13767c7` for causal classification.

### 1.6 Production-reachability baseline — a decisive, previously unrecorded fact

None of `C`, `R`, or the repair packet commit is an ancestor of launch HEAD.

```
merge-base(HEAD, C) = e2a218e015953f082245f36dbc6aee906b837e5a
  e2a218e..C     =  81 commits
  e2a218e..HEAD  = 331 commits
  commits touching crates/codefire-cli/src/storage.rs on e2a218e..HEAD = 0
```

Consequently:

- `HEAD:crates/codefire-cli/src/storage.rs` = `ea747173c6fd6b58150edd15b4ac063a2d014f06`,
  which differs from both `C` (`c22ba190…`) and `C`'s parent (`aeb4d605…`);
- `crates/codefire-cli/src/tests/storage_decision_work.rs` does not exist at
  launch HEAD at all;
- the canonical mainline storage command therefore predates the entire CFR-168
  storage decision-work lineage.

**What this changes for adjudication.** No R-finding is currently damaging the
canonical shipped product, because the candidate is an unintegrated proposal on
agent branches (`claude-agent/cfr168-rust-storage-zero-residual-repair-…`,
`fugu-agent/…`, `task-base/…`). "Live production path" in every row below means
*the live path of the storage command as `C` would ship it if integrated* —
which is the correct subject for an integration-candidate adjudication. It does
**not** mean an outage exists today. This is stated explicitly because reading
the R-severities as current production incidents would misprice the whole set.

---

## 2. Governing policy and vocabulary mapping

`docs/development/project-management.md` (lines 82-97, 148-152) and
`docs/development/ai/operating-protocol.md` §"Impact is not actionability" and
§"Actionability and causal classification" own this stage. The controlling
rules applied here:

- severity records magnitude and never says "fix";
- one of six protocol dispositions is assigned after the impact gate;
- the six evidence-bound must-fix triggers are `MF-1`…`MF-6`;
- "A reviewer-preferred implementation is not a required correction unless
  frozen evidence or a must-fix trigger makes it one";
- repair difficulty may change ordering/mechanism but cannot cancel a trigger;
- `out_of_scope` is retained and routed and may not hide a candidate-created
  effect;
- only `required_correction` receives a causal class.

This lane was asked for a finer terminal vocabulary. It maps onto the protocol
without inventing authority:

| This artifact's disposition | Protocol disposition | Repair authority in this batch |
|---|---|---|
| `required_product_correction` | `required_correction` | candidate product bytes |
| `required_test_or_review_evidence_only` | `required_correction` (`MF-5` only) | test/evidence bytes only, zero product bytes |
| `required_documentation_correction` | `required_correction` (`MF-1` via AGENTS.md canonical-truth rule) | canonical doc bytes only |
| `accepted_implementation_variance` | `accepted_implementation_variance` | none |
| `nonblocking_improvement_successor` | `nonblocking_improvement` | none |
| `duplicate_subsumed` | `duplicate` | none; delegates to the canonical row |
| `invalid_or_unsubstantiated` | `invalid_or_unsubstantiated` | none |
| `out_of_scope_routed_dependency` | `out_of_scope` | none; named dependency routed |

The recursion audit's §12.2 severity states are used as the *evidence-class*
vocabulary the immutable report lacked (`DEFECT-REPRODUCED`,
`DEFECT-INJECTION-ONLY`, `CLOSURE-UNPROVEN`, `ORACLE-VACUOUS`, `CLAIM-FALSE`,
`SCOPE-BOUNDARY`, `REQUIREMENT-UNSATISFIABLE`) and its §12.3 oracle rules
(`E1`…`E9`) as the admissibility test. That audit's §10 item 5 is the direct
mandate for this lane: *"Before the Storage repair resumes, re-partition its 23
open items (F1-F9 + R1-R14) into (a) reproduced defects, (b)
behavior-closed-but-reversal-not-run, (c) claim-only."*

### 2.1 The four mandatory distinctions, applied as stated

1. **Source proof of a live invariant violation can suffice without a runtime
   reproducer.** It does so here when the violating code path is
   unconditional and the violated clause is exact and frozen — e.g. R1's node
   charge sits before `ReadDir::next` on every loop iteration with no guard, and
   ADR-020 line 63 names "unique node discovery" as the charged unit. Nothing
   about that conclusion depends on execution; a reproducer would only restate
   the control flow. Where reachability *is* conditional (R2, R4), the row says
   so and uses `DEFECT-INJECTION-ONLY`, which §12.2 still treats as blocking
   absent a proof of unreachability.
2. **A missing reversal is missing acceptance evidence, not proof of a broken
   product.** All nine F rows and R10/R12/R13 turn on this. They receive
   `MF-5` only, never `MF-2`.
3. **A candidate test that asserts the defect is strong reproduced evidence.**
   Two such tests exist and both were read: the terminal-`Complete`-on-failed-
   publication assertion (R3) and the `ExitCode::RepositoryCorruption`
   assertion for an over-long user `--remote` argument (R9). Both are recorded
   as reproduced *behaviour*. Whether reproduced behaviour is a *defect* is a
   separate question answered against a frozen authority, and the two resolve
   oppositely — see R3 and R9.
4. **Reviewer preference, theoretical completeness, and zero-residual language
   are not triggers.** This retires R14's "release-blocking under zero-residual
   rules" promotion as a trigger (it survives on a different, valid basis) and
   caps R7(a), R9(a), R10 and R11(a) at their evidenced parts.

---

## 3. Master disposition table — all 23 items, exactly once

Columns: (1) original exact claim and evidence; (2) evidence class on `C`; (3)
live production path and realistic impact; (4) severity independent of
actionability; (5) must-fix trigger; (6) causal class; (7) terminal
disposition; (8) smallest next authorized action and exact writable
symbol/path; (9) 11-path grant adequacy.

`Loc` references are line numbers in the candidate blobs named in §1.4.

### 3.1 R1-R14 (canonical loci — adjudicated first because every F row delegates to them)

| # | (1) Claim and evidence | (2) Evidence class on `C` | (3) Live path and realistic impact | (4) Severity | (5) Must-fix trigger | (6) Causal class | (7) Disposition | (8) Smallest next action / symbol | (9) 11-path grant |
|---|---|---|---|---|---|---|---|---|---|
| **R1** | Local traversal counters do not represent the traversed graph: phantom `GraphNodes` per EOF probe, zero local `GraphEdges`. Report loc `storage.rs:1883/2026/2039/3002`. | **Source-proven, unconditional.** Verified: `scan_capability_files` charges `GraphNodes 1` at `storage.rs:2026-2037` *before* `stack[top].iterator.next()` at `2039`; the `Ok(None)` arm at `2042` releases only `TemporaryBytes` and pops — the cumulative node charge is not reversible. `grep GraphEdges storage.rs` returns exactly **one** site, `3002`, on the selected-remote URL dispatch; `process_traversal_entry` (`2193`) charges none. Root charged separately at `1898`. Tree with `D` dirs / `E` entries charges `1 + E + D`. | `main::run_storage_report_command` → `prepare_storage_report` → `scan_files` → `scan_capability_files`, on **every** `codefire storage` invocation. Nodes over-charged by exactly one per live directory, so the declared 100,000-node cap trips early and reports a false `BudgetExceeded`; the 400,000-edge cap never enforces locally at all. Both are wrong enforcement, not cosmetics. | **High** — resource/correctness on the primary command path. | `MF-1` proved (ADR-020 line 63: "unique node discovery … edge occurrence"; line 36 "graph nodes and edges discovered"). `MF-6` proved (see col. 6). | **candidate-introduced** (`repair_introduced_regression`): `StorageTraversalFrame` count at base `13767c7` = 0, at `C` = 8; the iterative traversal that miscounts was created by the F3 repair. Corroborates recursion audit §5.7 S2-R1. | `required_product_correction` | `crates/codefire-cli/src/storage.rs`: move the `GraphNodes` charge into the `Ok(Some(entry))` arm of `scan_capability_files`; charge one `GraphEdges` per parent→child relation in `process_traversal_entry` before `push_traversal_frame`; state root-node semantics once. Oracle in `crates/codefire-cli/src/tests/storage_decision_work.rs`. | **Sufficient** (paths 1, 4). |
| **R2** | A visitor error can discard a distinct Store scanner/finalization result in release builds. Report loc `storage.rs:2560/3434`, `codefire-store/src/lib.rs:3513/3616/3672`. | **Source-proven, conditional reachability.** Verified: `storage.rs:2567` and `3438` guard `scan_result` with `debug_assert!` only, then `return Err(error)` unconditionally. Independently confirmed the callee genuinely can return a non-sentinel result: `codefire-store/src/lib.rs` `match (scan_result, release_binding, release_root) { … (_, Err(e), _) \| (_, _, Err(e)) => Err(e) }` **replaces** the cancellation sentinel with a release error. Base `13767c7` had `return Err(error)` with no check at all, so `C` strictly improved this without closing it. | Full local object scan and selected-remote object scan. In the shipped (release) binary a Store finalization / identity-changed / retained-accounting fault is silently dropped while the visitor's error is surfaced. The user still gets *an* error, so no wrong success; the harm is that `object_store_identity_changed` and ledger-accounting faults are masked, i.e. a subject change can be reported as an unrelated visitor error. | **High** — authority / error-origin (`IV-08`). Reachability is conditional (needs both faults in one scan) and was not runtime-demonstrated; §12.2 `DEFECT-INJECTION-ONLY` remains blocking absent an unreachability proof, and none exists. | `MF-1` proved (packet §R2.1; ADR-020 typed-diagnostic contract). `MF-2` conditional. | **pre-existing** (`implementation_nonconformance`): the discard predates the range; `C` added a debug-only mitigation. Matches audit §5.7 `INITIAL_REVIEW_SCOPE_MISS`. | `required_product_correction` | `crates/codefire-cli/src/storage.rs`: replace both `debug_assert!` sites with an exhaustive `match (visitor_error, scan_result)` defining sentinel-vs-distinct precedence and preserving both errors where neither may erase the other. | **Sufficient** (path 1; injected-fault tests on path 4). |
| **R3** | The publication ledger reports terminal `Complete` for failed publication. Report loc `storage.rs:267/278`, test `…storage_decision_work.rs:1919`. | **Behaviour reproduced by a candidate test, but *not* a defect against the frozen authority.** Verified: `storage.rs:697-700` maps `StorageTerminalClass::PublicationFailure` → `DecisionWorkTerminalReason::Complete`, and the committed test asserts `terminal_reason == Some(Complete)` on a failed publish. **However** `codefire_core::DecisionWorkTerminalReason` (`crates/codefire-core/src/work_budget.rs:137`) has exactly six variants and **no publication-failure value**, and ADR-020 lines 38-40 freeze that vocabulary for the advertised capability profile `codefire.decision-work.v1`. `Complete` is also the cross-surface convention (`http.rs` 55 sites, `doctor/report/accounting.rs`, `remote/publication.rs`, `codefire-store`). The report's other three R3 sub-claims are affirmatively **satisfied**: exactly one terminal observation (`abort` guards on `terminal_reason.is_none()`, `storage.rs:541`), no terminal recorded before publication, and the external outcome is `PublishedOutputFailed`. | Publication failure on `codefire storage`. Realistic impact is confined to decision-work telemetry fidelity: a consumer reading `terminal_reason: complete` alongside a nonzero exit and a `PublishedOutputFailed` diagnostic. No wrong user outcome, no state or data effect, no exit-code error. | **Moderate** — observability/diagnostic fidelity, cross-surface. | **None proved.** `MF-1` **rejected**: ADR-020, the frozen authority, is satisfied; the violated clause is packet §R3.3, a repair-task requirement, not a product contract. `MF-3` would be *created*, not cured, by adding a seventh variant to an advertised profile. §12.2 class: `REQUIREMENT-UNSATISFIABLE` → "rewrite the criterion, not the candidate". | **requirement/scope expansion** | `out_of_scope_routed_dependency` | Route to the Core/ADR owner: `crates/codefire-core/src/work_budget.rs:137` (`DecisionWorkTerminalReason`) + `adr/ADR-020-decision-equivalent-work-budget.md` §"One work vocabulary" + the `codefire.decision-work.v1` profile declaration. Manager to record the decision that `Complete` is the ADR-conformant value and to correct packet clause §R3.3. **Change zero storage bytes for this item.** | **Insufficient for the packet's literal wording** — neither `crates/codefire-core/**` nor `adr/**` is granted, and the change is cross-surface. **Not needed at all** under the ADR-conformant reading. |
| **R4** | A post-write finish error can append a second JSON document. Report loc `storage.rs:290/291`, `main.rs:204`. | **Source-proven, conditional reachability.** Verified: `publish_to_writer` sets `self.publication = Completed` at `storage.rs:290` then returns `self.finish_complete_in_place()` at `291` **unwrapped**; that function (`530-536`) can fail via `release_retained()` or `ledger.finish()`. `main.rs:204-219` `top_level_error_rendering` wraps only `PublishedOutputFailed` to `StderrOnly`, so in JSON mode an unwrapped accounting error selects `JsonEnvelopeOnStdout` after a flushed success document. Contradicts `C`'s **own** declared contract at `main.rs:199-203` and its **own** canonical claim in `source-data-model-catalog.md` ("anything past `not_attempted` returns `CliError::PublishedOutputFailed`"). | `codefire storage --json` success path followed by a release/finish accounting fault. Two JSON documents on stdout — unparsable by every JSON consumer of this command. Requires an accounting-invariant breach to trigger, so likelihood is low; the contract violated is absolute and stated in three places. | **High** — output-transaction contract (`IV-07`); blast radius is the public stdout stream. | `MF-1` proved (violates the candidate's own frozen output contract *and* its own canonical document at the same tree). | **candidate-introduced** (`repair_introduced_regression`): `write_all_tracked` and `StoragePublicationState` are absent at base `13767c7`. | `required_product_correction` | `crates/codefire-cli/src/storage.rs`, `StorageReportRun::publish_to_writer`: wrap the `finish_complete_in_place()` error in `CliError::PublishedOutputFailed` when `self.publication.may_be_visible()`. Add a deterministic post-flush finish-failure seam consumed by the R13 process test. | **Sufficient** (paths 1, 4, 5). |
| **R5** | Traversal container and authority owners remain outside the portable peak. Report loc `storage.rs:1994/2080/2127`. | **Source-proven, unconditional, and the ADR is explicit here.** Verified: `Vec::<StorageTraversalFrame>::new()` at `1994` and `stack.push(…)` at `2115` with no charge for the vector's own capacity growth and no per-frame shell; `push_traversal_frame` charges only iterator scratch (`4,096`) plus the retained name. `pop_traversal_frame` opens a descriptor via `parent.directory.open_dir_nofollow(name)` at `2138` with no prior authority acquisition. ADR-020 line 72 mandates the frame unit **precisely** for this case: "plus `256 + utf8(node_id).len()` bytes for each retained traversal or frontier frame". | Every local and selected-remote directory walk. The concurrently-retained/temporary peak is **under**-charged, so the declared 1 GiB retained and temporary caps do not bound actual frontier cost; a deep or wide tree consumes more than the ledger believes. | **High** — resource accounting under-bound (`IV-02`/`IV-03`) on the primary path. | `MF-1` proved (ADR-020 line 72 verbatim; the one place the ADR *does* define the frame unit is the one place `C` omits it). `MF-6` proved. | **candidate-introduced** (`repair_introduced_regression`): same new traversal as R1. | `required_product_correction` | `crates/codefire-cli/src/storage.rs`: `scan_capability_files` / `push_traversal_frame` / `pop_traversal_frame` — acquire stack growth before `Vec` capacity expansion, charge the ADR-020 frame unit per stored frame, acquire the identity-reopen authority before `open_dir_nofollow`. **Same symbol set as R1 — one implementation, not two.** | **Sufficient** (paths 1, 4). |
| **R6** | Live storage owners are allocated before the command ledger exists. Report loc `storage.rs:1349/1366/1387/1416/1431`. | **Source-proven, unconditional.** Verified: `remotes.push(value.to_string())` at `1366` and `1387`, `PathBuf::from(value)` at `1416`, and `std::env::current_dir()?` at `1402` all execute before `DecisionWorkLedger::new` at `1431`. Four `parse_storage_report_args` call sites in `main.rs` (`1098/1109/1120/1131`). | Every `codefire storage` invocation with `--remote` or a path argument. The uncovered allocation is bounded by the OS `ARG_MAX` envelope, so it cannot be driven unbounded; the violation is of the exact charge-before-allocation clause, not of a practical memory bound. | **Moderate** — exact-invariant violation, low materiality (`IV-01`/`IV-02`). | `MF-1` proved (ADR-020 line 42 "A counter is charged before the corresponding allocation" and line 50 "One mutable `DecisionWorkLedger` is created for the complete command"; packet §R6). | **pre-existing** (`implementation_nonconformance`): identical at base `13767c7:705/726` and at parent. Matches audit §5.7 `INITIAL_REVIEW_SCOPE_MISS`. | `required_product_correction` | `crates/codefire-cli/src/main.rs` (the four storage dispatch sites) + `crates/codefire-cli/src/storage.rs` (`parse_storage_report_args`, `prepare_storage_report`): create the ledger at dispatch and pass it in, or parse borrowed spans and acquire exact owners before each clone. | **Sufficient** (paths 1, 2). Note `main.rs` is an operating-protocol serialize-listed global file — the manager must own its final version. |
| **R7** | The logical owner schema is not the frozen ADR-020 contract and is not portable for paths. Report loc `storage.rs:31/59/1132`, `source-data-model-catalog.md:117`. | **Two separable sub-claims, resolving differently.** **(a) Generalized 256-byte shell** — verified at `storage.rs:38-43` and `StorageLogicalShape::logical_bytes` (`76-83`): every semantic owner gets `LOGICAL_FRAME_BASE_BYTES` (=256, `work_budget.rs:11`), whereas ADR-020 line 72 scopes that unit to traversal/frontier frames. This is a **conservative over-charge**, not an under-bound; no reachable failure follows. **(b) Platform path encoding** — verified `storage_encoded_path_bytes` (`1132-1135`) uses `Path::as_os_str().as_encoded_bytes().len()`. On Unix one invalid byte charges 1, while `path.display()` emits a 3-byte U+FFFD, and `serde_json` cannot encode invalid UTF-8 at all. That is a genuine **under**-bound of the text formatter and JSON projection payload. | (a) all routes, harmless over-charge. (b) any repository containing a non-UTF-8 path component — creatable by any user on Linux — reached through text rendering and JSON projection. The reserved output envelope can be exceeded by the bytes actually produced. | **Moderate** for (b) — accounting under-bound on a reachable input. **Low/none** for (a). | `MF-1` proved **for (b)** (ADR-020 line 42: charged before "output append"; exactness). `MF-1` **rejected for (a)** — over-charging violates no clause; the false part of (a) is the *documentation*, carried by R14. | **candidate-introduced** for both (the whole formula layer is new in-range). | `required_product_correction` — carried by (b). (a) is recorded as substantively `accepted_implementation_variance` whose documentation correction is R14; it creates **no** product work. | `crates/codefire-cli/src/storage.rs`: `storage_encoded_path_bytes` / `storage_path_bytes` — define one portable path-output contract that dominates both lossy `display()` expansion and JSON escaping, and account JSON-serialization failure separately. | **Sufficient** (paths 1, 4, and doc paths for the (a) wording). An ADR-020 amendment is needed **only** if the manager prefers to bless (a) in the ADR instead of correcting the docs — routed, not required. |
| **R8** | JSON projection has a structural field-count defect hidden by fixed slack. Report loc `storage.rs:920/1000/3571`. | **Source-proven, arithmetic checked independently.** Verified: `object_file_json` (`3571-3579`) emits four string fields (`object_id`, `type`, `type_confidence`, `path`) plus one scalar (`bytes`); the projection call at `1003-1012` passes `scalar_fields = 1, string_fields = 3` while summing **all four** payloads. `STORAGE_JSON_STRING_FIELD_BYTES = 40 + 2 + 24 = 66`, so the omission is exactly 66 bytes per largest-object node, ≤ 660 for the hard-bounded top ten, currently absorbed by the unrelated `STORAGE_JSON_FIXED_STRUCTURE_BYTES = 64*128 = 8,192`. | `codefire storage --json` full route. The projection reservation is an upper bound used for budget enforcement; under-declaring it makes the bound non-dominating, so at a tightened limit the check passes when it should reject. No current fixture fails, because the slack hides it — which is itself the defect the packet names. | **Moderate** — accounting exactness; no user-visible effect at current constants. | `MF-1` proved (ADR-020 exactness; the formula's own stated contract at `storage.rs:914-918` is that it dominates the emitted document; packet §R8.1 names the four fields explicitly). | **candidate-introduced** (`repair_introduced_regression`): `storage_json_projection_bytes` absent at base. | `required_product_correction` | `crates/codefire-cli/src/storage.rs`, `storage_json_projection_bytes` largest-object arm: declare four string fields, and derive the counts from an independent schema table rather than a literal. Per-field mutation oracle on path 4. | **Sufficient** (paths 1, 4). |
| **R9** | URL accounting is a blanket multiplier and the 16 KiB proof is false. Report loc `storage.rs:2954`. | **Three sub-claims; two are proven defects, one is imprecision.** **(a) Blanket 12× multiplier** (`STORAGE_REMOTE_PARSE_URL_SIZED_OWNERS = 12` at `2972`): a conservative **over**-charge, not an under-bound. **(b) False justification** at `2957-2959` and `124-129`: "Every accepted project root is at least as long as its URL". Independently disproved by reading the shared parser — `remote.rs:2616-2632` `percent_decode` builds `Vec::with_capacity(bytes.len())` and emits one byte per `%XX` triple, so the decoded root can be a third of the URL length. A >16 KiB URL can yield a short, legal project root. This also creates a **new public input cliff**. **(c) Misclassification**: `2960` returns `CliError::InvalidRepository`, which `storage_terminal_class:755` maps to `SubjectCorrupt` → `CorruptSubject`. The candidate's **own test** (`…storage_decision_work.rs`, `formula::remote_url_parse_bytes` case) asserts `error.exit_status() == ExitCode::RepositoryCorruption` for an over-long **user argument**. The shared parser itself classifies bad URL input as `CliError::Usage` (`remote.rs:2579/2588`), so `C` is also internally inconsistent. | (c) is reachable by any user typing `codefire storage --remote cf://<16 KiB+>`: they receive a repository-corruption exit code and a `corrupt_subject` terminal reason for a plain argument-length rejection. (b) narrows the previously supported input domain. (a) only over-reserves. | **High** for (c) — user-visible wrong outcome class on a live input path. **Moderate** for (b). **Low** for (a). | `MF-1` proved for (c): ADR-020 lines 39-40 verbatim — "Business-policy rejection that completes its bounded decision work is not mislabeled as corruption." `MF-3` proved for (b): the supported input domain is narrowed without an accepted product-limit decision. | **candidate-introduced** (`repair_introduced_regression`): `storage_remote_url_parse_bytes` absent at base. | `required_product_correction` | `crates/codefire-cli/src/storage.rs`, `storage_remote_url_parse_bytes`: return `CliError::Usage` (→ `BoundedUsageCompletion`) for the length rejection; delete the false implication at `2957-2959` and `124-129`; either preserve the prior domain or record an explicit accepted limit; tighten (a) by enumerating the parser's actual owners. **`remote.rs` need not be edited** — see col. 9. | **Sufficient.** The preflight's R9 rule offers two branches; branch (a) — "close R9 **without editing the shared parser**" — is satisfiable: `remote::parse_cf_project_url` (`remote.rs:2577-2614`) was read and its owner set is finite and exactly enumerable (`Vec<&str>` segments, `suffix`, `server_raw`, `absolute_path` root, two `percent_decode` `Vec<u8>`+`String` pairs, two validated components, four `join` intermediates, returned `project_root`), all bounded by URL length plus the cwd envelope. The routed-handoff branch is therefore **not** triggered, and R9 is **not** an acceptance blocker. |
| **R10** | Local/remote path-owner counts are not exact simultaneous lifetimes. Report loc `storage.rs:109/3035`, test `:1599`. | **Product defect not proven; evidence defect proven by the candidate's own words.** Verified fixed counts `LOCAL_DERIVED_PATH_OWNER_COUNT = 5` (`113`) and `REMOTE_DERIVED_PATH_OWNER_COUNT = 6` (`119`). Both directions cited by the report — local retaining `.codefire` with sequential joins, and the remote reservation overlapping the live URL-parse reservation — are **over**-counts, i.e. conservative; no under-bound is demonstrated. The committed test `storage_local_derived_path_owners_are_reserved_before_the_first_join` **states in its own docstring** that "removing it does not necessarily push the measured peak below this expectation". That is a direct §12.3 `E5` violation. | Local and selected-remote prepare phases. Realistic product impact: over-reservation only, i.e. the budget is stricter than necessary. Realistic evidence impact: the omission oracle cannot detect removal of the owner it exists to protect. | **Low** (product) / **Moderate** (evidence). §12.2 class: `ORACLE-VACUOUS`. | `MF-5` proved (packet §R10.4 requires an omission-sensitive exact/minus-one oracle per owner; absent and self-admitted). `MF-1`/`MF-2` **rejected** — no under-bound, no reachable failure. | **evidence gap** | `required_test_or_review_evidence_only` | `crates/codefire-cli/src/tests/storage_decision_work.rs`: isolate the local reservation (dedicated dimension limit or shortened measured scope) so its removal fails, and give local and remote owners separate probes. **Zero candidate product bytes.** | **Sufficient** (path 4). Path 1 needed only if the manager elects scope-shortening over test isolation. |
| **R11** | Typed terminal classification remains non-exhaustive and text-dependent. Report loc `storage.rs:718/730/776/789/2690`. | **Two sub-claims.** **(a) Wildcard** at `776`: real, but a future-variant safety property; every *existing* `CliError` arm was inspected and classifies correctly today. **(b) Text dependence**: `storage_io_error_is_subject_change` (`789-797`) discriminates a subject change by `message.contains("changed size while reading" \| "changed during read")`, and `object_record_failed_semantic_validation` (`2690-2703`) discriminates semantic validation by `*context == path.display().to_string()`. Both are current, reachable, cross-crate string couplings with no compile-time guard: any wording change in `secure_worktree.rs:1434/1671/1682` or in the Store's `IncompleteRecord` context silently reclassifies a subject change as ordinary I/O, or corruption as I/O. | Every scan that produces a pinned-read or Store record error. Misclassification changes the ADR-020 terminal reason (`subject_changed` ↔ `complete` ↔ `corrupt_subject`) and the user-facing diagnostic. | **Moderate** — error-origin classification (`IV-08`). | `MF-1` proved for (b) (ADR-020 typed-diagnostic contract; packet §R11.3). For (a) the trigger rests on packet §R11.1 alone; it is cheap and is folded into the same repair, but it does **not** independently block. | **candidate-introduced** surface (`storage_terminal_class` absent at base) over a **pre-existing shared-owner gap** (the Store has no typed semantic-validation variant). Governance class for the required correction: `repair_introduced_regression`. | `required_product_correction` | `crates/codefire-store/src/lib.rs`: add the typed semantic-validation variant. `crates/codefire-cli/src/secure_worktree.rs`: add a typed pinned-subject-change error in place of the marker strings. `crates/codefire-cli/src/storage.rs`: consume both, remove the wildcard from `storage_terminal_class`. | **Sufficient** (paths 1, 3, 6). This **supersedes** the report §12 question 1 rationale, which said `codefire-store` is "not one of the eight writable candidate paths" — true of `C`'s eight *changed* paths, but the addendum's eleven-path **grant** includes `crates/codefire-store/src/lib.rs`, so this handoff is in scope here. |
| **R12** | Pinned reads do not prove final path/parent authority. Report loc `storage.rs:3485`, `secure_worktree.rs` (`PinnedBoundedFile`), test `:1473`. | **No production defect proven; a contract is undefined and evidence is absent.** Verified the entire in-range `secure_worktree.rs` delta is `+244` lines, **100 % `#[cfg(test)]`** race-injection infrastructure (`PinnedFileRaceWindow::{BeforeOpen, AfterPin}`); `PinnedBoundedFile`'s product authority semantics are untouched and pre-existing. Reading the pinned inode after a directory-entry replacement is the *intended* behaviour of a pin-before-read design, not a violation — packet §R12.1 asks to "**define** the supported result", i.e. the contract is undefined, not broken. Confirmed no committed focused result exists for truncation, parent replacement, or final descriptor-plus-parent revalidation. | The optional-JSON reads on the local and remote report routes. `codefire storage` is a **read-only reporting** command: a stale-inode read yields a stale report value, with no state, data, or authority mutation. Blast radius is one reported field. | **Low-moderate.** Authority axis is `unknown`, not proved `affected`; recorded as `unknown` with the read-only-command evidence, **not** treated as neutral. | `MF-5` proved (packet §R12.4 requires deterministic replacement/truncation/parent-swap seams; absent). `MF-1`/`MF-2` **rejected** — no declared contract violated, no reachable material failure on a read-only path. **Not** acceptance-critical: the read-only blast radius is the evidence for that, per the protocol's requirement that `not_applicable`/bounded claims carry an axis-specific reason. | **evidence gap** + **requirement/scope expansion** (§R12.1 is a new clause). | `required_test_or_review_evidence_only` | `crates/codefire-cli/src/secure_worktree.rs` (`#[cfg(test)]` only): extend `PinnedFileRaceWindow` with truncation and parent-replacement windows. `crates/codefire-cli/src/tests/storage_decision_work.rs`: deterministic outcomes with exact owner cleanup. The one-sentence contract statement rides the R14 doc sync. **Zero product bytes.** | **Sufficient** (paths 3, 4, and a doc path for the contract sentence). |
| **R13** | Publication tests do not exercise required writer/channel sequences. Report loc `…storage_decision_work.rs:1881/1948`. | **Pure acceptance-evidence gap; production is correct on the named sequences.** Verified only two scripted writers exist (`PrefixThenFailWriter`, `RejectingWriter`) and no process-level test; `crates/codefire-cli/tests/storage_report_publication.rs` does not exist at `C`. **Production already handles** the missing sequences: `write_all_tracked` (`299-323`) returns a typed `WriteZero` on `Ok(0)` at `308-313`, retries `Interrupted` at `315`, loops short writes, and flushes at `322` with `PartiallyProgressed` already latched. So the absent traces are missing *proof*, not evidence of breakage. The single sequence whose production behaviour **is** defective — post-flush finish failure — is R4, not R13. | Publication on every route. No production impact from R13 itself. | **Moderate** — evidence-side only. §12.2 class: `CLOSURE-UNPROVEN` for the handled sequences. | `MF-5` proved (the resume preflight §"Required process publication gates" names the exact file and both profiles as a *named acceptance requirement*, with nonzero executed count and exact stdout/stderr byte assertions). `MF-1`/`MF-2` **rejected**. | **evidence gap** | `required_test_or_review_evidence_only` | Create `crates/codefire-cli/tests/storage_report_publication.rs` (granted path 5) with `WriteZero`, single and repeated `Interrupted`, short-write completion, flush failure, post-flush finish failure, and real stdout/stderr byte assertions in text and JSON; extend `crates/codefire-cli/src/tests/storage_decision_work.rs`. **Zero product bytes** beyond the R4 seam. | **Sufficient** (paths 4, 5). Depends on the R4 fix existing to make the post-flush case meaningful. |
| **R14** | Canonical docs overstate implemented guarantees. Report loc `change-playbook.md:331`, `source-data-model-catalog.md:117`. | **`CLAIM-FALSE`, verified verbatim at `C` — four distinct false current-state claims.** (i) `change-playbook.md`: "never let cleanup accounting mask the primary typed error" — false in release builds (R2). (ii) `change-playbook.md` **and** `source-data-model-catalog.md`: "one 256-byte frame per owner" presented as the portable ADR-020 formula — ADR-020 line 72 scopes that unit to traversal/frontier frames only (R7a). (iii) `source-data-model-catalog.md`: "anything past `not_attempted` returns `CliError::PublishedOutputFailed`" — false for `Completed` + finish failure (R4). (iv) `change-playbook.md`: "every simultaneous local and remote `PathBuf` before the first `join`" — overstated given R6 and R10. Both structural checkers pass, which is why the report correctly notes semantic truth is stricter than inventory consistency. | Canonical source documents are the reading route for every future change to this surface (AGENTS.md line 18). A false current-state claim mis-guides the next implementer and the next reviewer — precisely the recurrence this batch exists to stop. No runtime impact. | **Moderate** — development-flow / maintainability, with a proven recurrence history. | `MF-1` proved — but **on a different basis than the report used**. The report's stated basis, "Should fix, **release-blocking under zero-residual rules**", is explicitly **rejected** here: old zero-residual language is not a must-fix trigger under current policy. The surviving basis is AGENTS.md line 41 (canonical source docs must be updated in the same change as the implementation) plus §12.2 `CLAIM-FALSE`. | **documentation claim** | `required_documentation_correction` | `docs/development/change-playbook.md` (storage paragraph, claims i/ii/iv) and `docs/development/source-data-model-catalog.md` (`StorageReportRun` paragraph, claims ii/iii). Sequenced **after** the product repairs land, per packet §R14. Conditional sync of `source-architecture.md` and `command-source-trace.md` only if R6/R1 move a documented boundary. **Zero product bytes.** | **Sufficient** (paths 7-10). |

**Separation of R14 truth correction from product behaviour, as required.** Claims
(i), (iii) and (iv) become true once R2, R4 and R6/R10 are repaired, so their
correction is a *synchronisation* obligation that rides the repair. Claim (ii)
is independent: R7(a) is adjudicated as an accepted conservative over-charge in
the code, so the documents must be corrected to describe it as a storage-local
conservative convention rather than as the ADR-020 contract — **regardless of
whether any product byte changes**. That single claim is the irreducible
docs-only residual and it is not contingent on any repair.

### 3.2 F1-F9

Independent verification of the report's §4 and §11 tables confirms two facts.
First, every F row's stated blocking reason is, without residue, one or more
R rows — the mapping is 1:1 with no orphan content. Second, the report's §11
reversal table records `not run` in **both** the "executable predecessor
failure" and "reviewer temporary reversal" columns for **all nine** items, and
the report states this plainly rather than disguising it. That is one shared
evidence residual, not nine.

Per the protocol's duplicate rule ("Reference that ID; create no new unique
effect/finding/issue/repair … Retain the observation in review-workload, churn,
and actionability-distribution evidence"), each F row is dispositioned
`duplicate_subsumed` against its named canonical R rows, and the single shared
reversal residual is carried once as evidence item **B-REV** in §4.B. All nine
rows are retained; none is dropped.

| # | (1) Claim and evidence | (2) Evidence class on `C` | (3) Live path and realistic impact | (4) Severity | (5) Must-fix trigger | (6) Causal class | (7) Disposition | (8) Smallest next action | (9) 11-path grant |
|---|---|---|---|---|---|---|---|---|---|
| **F1** | Portable formulas. Report §4: "pre-ledger input owners, platform path encoding, generalized 256-byte shell, and omitted collection/authority owners". Evidence: source inspection + rerun portable-golden tests. | `duplicate` of **R6** (pre-ledger owners), **R7** (path encoding, 256-byte shell), **R5** (collection/authority owners). No distinct effect. Reversal `not run` → shares B-REV. | Delegated to R5/R6/R7. | Delegated (max **High**, R5). | Delegated. Distinct residual: `MF-5` via B-REV. | delegated; `evidence gap` for the reversal share | `duplicate_subsumed` → R5, R6, R7 | None of its own. Repair via G1/G4/G5; reversal via B-REV. | Delegated; **sufficient**. |
| **F2** | Projection/response. Report §4: "four emitted largest-object strings are declared as three; broad fixed slack hides mismatch". Evidence: independent `reference_json_bytes` inspected + focused tests rerun. | `duplicate` of **R8**, exactly. Reversal `not run` → shares B-REV. Note the report's own §11 finding that `reference_json_bytes` **is** an admissible independent output oracle is retained as positive evidence. | Delegated to R8. | Delegated (**Moderate**). | Delegated; distinct residual `MF-5` via B-REV. | delegated | `duplicate_subsumed` → R8 | None of its own. Repair via G5. | Delegated; **sufficient**. |
| **F3** | Traversal/depth/order. Report §4: "phantom nodes, absent local edges, uncharged stack capacity/reopen authority". Evidence: root/depth/deep/symlink tests rerun + live loop inspected. | `duplicate` of **R1** and **R5**. The report's own positive findings — iterative heap traversal, root depth zero, child rejected before open, 4,096-level fixture passing — are retained as satisfied obligations. Reversal `not run` → shares B-REV. | Delegated to R1/R5. | Delegated (**High**). | Delegated; distinct residual `MF-5` via B-REV. | delegated | `duplicate_subsumed` → R1, R5 | None of its own. Repair via G1. | Delegated; **sufficient**. |
| **F4** | Pinned read. Report §4: "replacement is accepted through old inode; truncation, parent replacement, final path authority unproved". Evidence: race/growth/replacement/exact-temp tests rerun. | `duplicate` of **R12**. Positive obligations verified satisfied: pin-before-size/charge/read established, in-place growth rejected. Reversal `not run` → shares B-REV. | Delegated to R12; read-only report command. | Delegated (**Low-moderate**). | Delegated `MF-5`. | delegated (`evidence gap`) | `duplicate_subsumed` → R12 | None of its own. Evidence via B2. | Delegated; **sufficient**. |
| **F5** | URL owners. Report §4: "blanket 12-owner formula, false 16 KiB proof/cliff, corruption classification". Evidence: URL bound test rerun + adapter and shared parser inspected. | `duplicate` of **R9** (a), (b), (c). Reversal `not run` → shares B-REV. | Delegated to R9. | Delegated (**High**, via R9c). | Delegated; distinct residual `MF-5` via B-REV. | delegated | `duplicate_subsumed` → R9 | None of its own. Repair via G3/G5. | Delegated; **sufficient** (see R9 col. 9 — the shared-parser branch is not triggered). |
| **F6** | Path owners. Report §4: "local probe is acknowledged non-omission-sensitive; counts do not match exact lifetimes". Evidence: local/remote owner tests rerun + scopes inspected. | `duplicate` of **R10**. Reversal `not run` → shares B-REV. | Delegated to R10; over-reservation only. | Delegated (**Low** product / **Moderate** evidence). | Delegated `MF-5`. | delegated (`evidence gap`) | `duplicate_subsumed` → R10 | None of its own. Evidence via B1. | Delegated; **sufficient**. |
| **F7** | Terminal matrix. Report §4: "debug-only precedence, wildcard, message matching, Store semantic text dependency". Evidence: terminal matrix test rerun + classifier and Store scanner inspected. | `duplicate` of **R2** and **R11**. Reversal `not run` → shares B-REV. | Delegated to R2/R11. | Delegated (**High**, R2). | Delegated; distinct residual `MF-5` via B-REV. | delegated | `duplicate_subsumed` → R2, R11 | None of its own. Repair via G3. | Delegated; **sufficient**. |
| **F8** | Publication. Report §4: "failed publication terminal is `Complete`; completed-publication finish error can emit second JSON; required writer traces absent". Evidence: partial/reject/preflight tests rerun + top-level route inspected. | `duplicate` of **R3** (routed), **R4** (required product correction), **R13** (evidence). This is the one F row whose canonical children split across three dispositions — it is retained whole and delegates to each. Reversal `not run` → shares B-REV. | Delegated to R3/R4/R13. | Delegated (**High**, R4). | Delegated; distinct residual `MF-5` via B-REV. | delegated | `duplicate_subsumed` → R3, R4, R13 | None of its own. Repair via G2; evidence via B3; routing via D1. | Delegated; **sufficient for R4/R13, insufficient for R3's literal clause** — see R3 col. 9. |
| **F9** | Canonical truth. Report §4: "playbook and data-model catalog overstate live guarantees". Evidence: both document checkers rerun + four docs compared to source. | `duplicate` of **R14**. Reversal `not run` → shares B-REV. The report's verdict that `command-source-trace.md` and `source-architecture.md` "broadly match" is retained. | Delegated to R14. | Delegated (**Moderate**). | Delegated `MF-1` (on the AGENTS.md basis, not the zero-residual basis). | delegated (`documentation claim`) | `duplicate_subsumed` → R14 | None of its own. Correction via set C. | Delegated; **sufficient**. |

### 3.3 Precision note on the immutable report

Not one of the 23 rows is dispositioned `invalid_or_unsubstantiated`. Every
claim inspected was accurate as a statement of fact about the candidate source;
where this adjudication departs from the report it is on the *consequence* of a
fact (R3, R7a, R9a, R10, R11a, R12, R13, R14's basis), never on the fact
itself. The report's identity block reproduced digit-for-digit, and it
explicitly declined to relabel source proof as executed reversal. The defect
the recursion audit identified is confirmed exactly: the failure was a
**vocabulary** with no state for "not proven closed", not fabrication. The
`REQUEST_CHANGES` verdict is preserved unaltered and remains correct — R1, R2,
R4, R5, R6, R7(b), R8, R9(b,c) and R11(b) independently justify it.

---

## 4. Synthesis

### A. Required product repair set, grouped by root cause and symbol

Five groups. Grouping is by root cause and writable symbol so that repeated
symptoms produce **one** implementation, per the recursion audit's §10 item 4
rule against surface-local duplication.

| Group | Root cause | Covers | Exact writable symbols | Non-duplication note |
|---|---|---|---|---|
| **G1** | Traversal graph accounting and frontier ownership — the new iterative walk charges the wrong units and omits ADR-020's own frame unit | R1, R5 (and F1-part, F3) | `crates/codefire-cli/src/storage.rs`: `scan_capability_files`, `process_traversal_entry`, `push_traversal_frame`, `pop_traversal_frame`, `scan_files` | R1 and R5 are the same loop and the same frame lifecycle. One patch, one oracle family. |
| **G2** | Publication output transaction — an error after bytes may be visible escapes the no-second-document contract | R4 (and F8-part) | `crates/codefire-cli/src/storage.rs`: `StorageReportRun::publish_to_writer`, `finish_complete_in_place` | Distinct from G3: this is output visibility, not error classification. |
| **G3** | Typed error-origin classification and precedence — outcome class is inferred from text, from a shared sentinel, or from a debug-only assertion | R2, R11, **and the classification half of R9(c)** | `crates/codefire-cli/src/storage.rs`: the two `(visitor_error, scan_result)` sites, `storage_terminal_class`, `storage_io_error_is_subject_change`, `object_record_failed_semantic_validation`; `crates/codefire-cli/src/secure_worktree.rs`: typed pinned-subject-change error replacing the marker strings; `crates/codefire-store/src/lib.rs`: typed semantic-validation variant | R9(c) is the *same* root cause (`IV-08`, ADR-020 lines 39-40) applied at a different symbol. The **rule** is defined once in G3; G5 applies it at `storage_remote_url_parse_bytes` so that one function is edited by one group. |
| **G4** | Charge-before-allocation at the command entry boundary — owners exist before the ledger | R6 (and F1-part) | `crates/codefire-cli/src/main.rs`: the four storage dispatch sites (`1098`, `1109`, `1120`, `1131`); `crates/codefire-cli/src/storage.rs`: `parse_storage_report_args`, `prepare_storage_report` | The only group touching `main.rs`; manager owns the final version of that serialize-listed file. |
| **G5** | Portable formula exactness and output dominance — formulas under-bound the bytes actually produced, or rest on a false implication | R7(b), R8, R9(a)(b)(c-at-symbol) | `crates/codefire-cli/src/storage.rs`: `storage_encoded_path_bytes`/`storage_path_bytes` (portable path-output contract), `storage_json_projection_bytes` (four string fields, schema-derived), `storage_remote_url_parse_bytes` (exact enumerated owners, false implication removed, `CliError::Usage` per G3's rule), `STORAGE_REMOTE_URL_BYTE_LIMIT` doc comment | Three symptoms, one root cause: a bound that does not dominate the thing it claims to bound. |

Product paths touched by A: `storage.rs`, `main.rs`, `codefire-store/src/lib.rs`,
and `secure_worktree.rs` (G3's typed error only). **Four** of the eleven.

### B. Evidence-only set — requires no candidate product byte change

| ID | Item | Covers | Exact path | Note |
|---|---|---|---|---|
| **B1** | Omission-sensitive local and remote derived-path probes that actually fail when the owner is removed (§12.3 `E5`) | R10, F6 | `crates/codefire-cli/src/tests/storage_decision_work.rs` | The candidate's own docstring is the admission of vacuity. |
| **B2** | Deterministic truncation and parent-replacement seams plus final descriptor/parent revalidation outcomes | R12, F4 | `crates/codefire-cli/src/secure_worktree.rs` (`#[cfg(test)]` windows), `crates/codefire-cli/src/tests/storage_decision_work.rs` | The declared contract sentence rides set C, not a product change. |
| **B3** | Complete writer and channel sequence matrix plus the two-profile real-process gate | R13, F8-part | **create** `crates/codefire-cli/tests/storage_report_publication.rs`; extend `crates/codefire-cli/src/tests/storage_decision_work.rs` | Preflight names this file and both `--locked` profiles explicitly, with nonzero executed count and exact stdout/stderr bytes. Consumes G2's finish-failure seam. |
| **B-REV** | The **single** shared reversal residual: one executable predecessor failure or reviewer temporary reversal per contract, or an explicit `CLOSURE-UNPROVEN` mark | F1-F9 (all nine), plus one reversal per G1-G5 contract | `crates/codefire-cli/src/tests/storage_decision_work.rs` and the durable result | Recursion audit §10 items 5 and 6. Recorded **once**, not nine times, so nine `OPEN` labels do not become nine repair units. §12.3 `E6`: a reversal must fail behaviourally, not through a missing symbol. |

### C. Documentation-only correction set

| ID | Item | Covers | Exact path | Contingency |
|---|---|---|---|---|
| **C1** | Correct "one 256-byte frame per owner" from an asserted ADR-020 contract to a storage-local conservative convention, in both documents | R14(ii), R7(a), F9 | `docs/development/change-playbook.md`, `docs/development/source-data-model-catalog.md` | **Unconditional** — required whether or not any product byte changes. |
| **C2** | Re-synchronise the precedence, publication-state and path-owner claims to the repaired live behaviour | R14(i)(iii)(iv), F9 | same two documents | Rides G2/G3/G4; sequenced **after** them, per packet §R14. |
| **C3** | Conditional sync of the ledger-creation boundary and graph semantics | R14 follow-on | `docs/development/source-architecture.md`, `docs/development/command-source-trace.md` | **Only if** G4 moves the documented ledger boundary or G1 changes documented graph semantics. The report found both documents currently "broadly match". |
| **C4** | State the pinned descriptor/parent/path-entry supported outcomes as a one-sentence contract | R12 contract half | `docs/development/source-data-model-catalog.md` | Rides B2. |

### D. Routed shared-owner dependencies

Named exactly. This lane does not pretend to close any of them.

| ID | Dependency | Exact owner symbol/path | Why this lane cannot close it | Acceptance-critical? |
|---|---|---|---|---|
| **D1** | A publication-failure terminal reason | `crates/codefire-core/src/work_budget.rs:137` `DecisionWorkTerminalReason` (six frozen variants) + `adr/ADR-020-decision-equivalent-work-budget.md` §"One work vocabulary" lines 38-40 + the advertised `codefire.decision-work.v1` profile | Neither `crates/codefire-core/**` nor `adr/**` is in the eleven-path grant, and the vocabulary is consumed cross-surface (`http.rs` 55 sites, `doctor`, `remote/publication.rs`, `codefire-store`). Adding a seventh variant is an advertised-profile change, i.e. an `MF-3`-class decision of its own. | **No** — no must-fix trigger is proved against the frozen authority (see R3). Requires a manager decision plus a correction to packet clause §R3.3. |
| **D2** | Explain-owned combined projection/publication | `crates/codefire-cli/src/explain.rs:285` `explain_storage_warning` | Not in the grant. `StorageReportRun::into_report_for_internal_consumer` releases and finishes storage ownership before Explain constructs its own result, so the boundary is coherent. | **No** — the report accepts the boundary and does not claim Explain's ownership is solved. |
| **D3** | Exact shared-parser owner plan, *if* in-adapter enumeration proves infeasible | `crates/codefire-cli/src/remote.rs:2577` `parse_cf_project_url` | Not in the grant. | **No, and currently not triggered.** The parser was read and its owner set is finite and exactly enumerable from outside, satisfying the preflight's first R9 branch. Recorded as a contingency only; if the implementer finds an owner that cannot be bounded without editing `remote.rs`, they must stop that part and record it rather than widen scope. |
| **D4** | Cross-surface recurrence of the publication-transaction family (`IV-07`) | The shared-primitives successor tracked by the non-review audit §"Causal proof" (Upload `76baff8`, MR-list `4930d25`, MR-write `6dd4527`, Storage `cdcb9bd`) | Outside this lane entirely. | **No** for this candidate; relevant because it reinforces that D1 belongs in a shared owner, not in a storage-local patch. |

### E. Nonblocking / variance / invalid set

| ID | Item | Class | Basis |
|---|---|---|---|
| **E1** | R3's other three sub-claims — exactly one terminal observation, no terminal recorded before publication, external `PublishedOutputFailed` | `accepted_implementation_variance` | Affirmatively proved satisfied at source (`storage.rs:541` `is_none()` guard; `abort` only on failure; `284-288` wrapping). Bind a manager decision at locus `StorageReportRun::abort`. Authorises no repair. |
| **E2** | R7(a) generalized 256-byte owner shell | `accepted_implementation_variance` (product) + `required_documentation_correction` (docs, = C1) | A conservative over-charge violating no clause. The false part is the documentation, not the code. |
| **E3** | R9(a) blanket twelve-owner multiplier, beyond the G5 tightening | `nonblocking_improvement_successor` | Over-reservation only; no under-bound. The tightening is folded into G5 because the same function is already being edited, not because it independently blocks. |
| **E4** | R11(a) wildcard removal as future-variant compile-time safety | `nonblocking_improvement_successor`, folded into G3 | Today's arms classify correctly; the value is future-variant safety. Cheap and same-symbol, so it ships with G3, but it is **not** an independent blocker. |
| **E5** | The report's promotion of R14 as "release-blocking under zero-residual rules" | `invalid_or_unsubstantiated` **as a trigger only** — the observation itself stands | `project-management.md:148-152` and `operating-protocol.md`: old zero-residual language and reviewer preference are not must-fix triggers. R14 survives on AGENTS.md line 41 and §12.2 `CLAIM-FALSE`. This is the one place where the prompt's "old zero-residual policy" caution changes an outcome's **basis** without changing its disposition. |

Nothing in the 23 is `invalid_or_unsubstantiated` at row level (§3.3).

### F. Corrected minimal candidate write set and test/reversal matrix

#### F.1 Minimal write set derived bottom-up from sets A-C

| # | Granted path | Needed? | Why, and by which set |
|---:|---|---|---|
| 1 | `crates/codefire-cli/src/storage.rs` | **yes** | G1, G2, G3, G4, G5 — the dominant repair surface. |
| 2 | `crates/codefire-cli/src/main.rs` | **yes** | G4 only; four dispatch sites. Serialize-listed global file. |
| 3 | `crates/codefire-cli/src/secure_worktree.rs` | **yes, narrowly** | G3 typed pinned-subject-change error (product) + B2 `#[cfg(test)]` windows. **Not** for a new product authority contract — the addendum authorises it for "the R12 final descriptor, parent, and path-entry authority contract", and R12 is adjudicated evidence-only, so the product use here is G3's, which is narrower than the grant contemplated. |
| 4 | `crates/codefire-cli/src/tests/storage_decision_work.rs` | **yes** | B1, B2, B3, B-REV and every G-oracle. |
| 5 | `crates/codefire-cli/tests/storage_report_publication.rs` *(create)* | **yes** | B3; named verbatim by the resume preflight. Directory already exists at `C`. |
| 6 | `crates/codefire-store/src/lib.rs` | **yes** | G3 typed semantic-validation variant. Supersedes the report §12 Q1 "outside scope" rationale. |
| 7 | `docs/development/source-architecture.md` | **conditional** | C3 only — required iff G4 moves the documented ledger boundary. |
| 8 | `docs/development/command-source-trace.md` | **conditional** | C3 only — required iff G1/G4 change documented graph or dispatch semantics. |
| 9 | `docs/development/source-data-model-catalog.md` | **yes** | C1, C2, C4. |
| 10 | `docs/development/change-playbook.md` | **yes** | C1, C2. |
| 11 | `…/swarm-results/cfr168-rust-storage-review-closure-repair-20260731/result.md` *(create)* | **yes** | Durable result. |

**Proof that the eleven-path set remains exact.** Every one of the eleven is
required by at least one item in sets A, B or C (paths 7 and 8 conditionally,
under C3). No granted path is unnecessary. Conversely, no item in sets A, B or
C requires a path outside the eleven: G1/G2/G4/G5 are confined to `storage.rs`
and `main.rs`; G3's cross-crate reach lands exactly on the two granted
non-`storage.rs` product files; B1-B3 land on the two granted test paths; C1-C4
land on the granted canonical documents. **The set is exact and is not to be
widened.**

**Where it is insufficient.** Exactly one place: **R3**. The packet's literal
clause §R3.3 ("must receive the exact typed publication-failure terminal reason
once") cannot be satisfied by any combination of the eleven paths, because the
enum lives at `crates/codefire-core/src/work_budget.rs:137` and its vocabulary
is frozen by `adr/ADR-020` lines 38-40 as an advertised capability profile.
This is a `REQUIREMENT-UNSATISFIABLE` condition: the correct response is to
rewrite the criterion, not the candidate. Because R3 carries no proved must-fix
trigger, this insufficiency **does not block acceptance** — it blocks only the
packet clause. Routed as D1.

One further contingency: if the implementer discovers a
`remote::parse_cf_project_url` owner that cannot be bounded from the adapter,
R9's exactness half becomes blocked on the ungranted `remote.rs` (D3). Reading
the parser gives no present reason to expect this.

#### F.2 Test and reversal matrix

Each row states the oracle, the deliberate reversal that must fail
behaviourally (§12.3 `E6`), and the admissibility rules it must satisfy. No
row may use a candidate-observed snapshot as its own expectation (`E7`).

| Contract | Independent oracle | Required reversal (must fail behaviourally) | Rules |
|---|---|---|---|
| G1 nodes | Hand-derived empty / flat / branching / deep node counts; exact `L`, `L-1`, and one added subject node at fixed `L` | Restore the pre-`next` charge | `E2`, `E5`, `E7` |
| G1 edges | Hand-derived parent-child edge counts on the same fixtures; exact `L`, `L-1`, one added child at fixed `L` | Remove local edge charging | `E2`, `E5`, `E7` |
| G1 frontier ownership | Separate omission probes for stack capacity, per-frame shell, and identity-reopen authority | Omit each of the three independently | `E5`, `E9` |
| G2 output transaction | Process-level text and JSON stdout/stderr byte assertions, exactly one document | Unwrap the post-flush finish error | `E1`, `E3`, `E8` |
| G3 precedence | Injected Store finalization/release failure concurrent with a visitor error, in **both** debug and release | Restore the `debug_assert!`-only branch | `E8`, `E9` |
| G3 typed classification | Terminal-matrix table over every `CliError` class; compile-time failure on a new variant | Restore the wildcard; restore each message-text discriminator | `E4`, `E6` |
| G3 argument outcome class | Over-long `--remote` yields usage/business exit and terminal reason, not corruption | Restore `InvalidRepository` | `E3` |
| G4 charge-before-allocation | Large `--remote` and path arguments at exact/minus-one limits; assert no pre-ledger allocation | Restore pre-ledger CLI allocation | `E3`, `E5` |
| G5 path portability | Non-UTF-8 path component: charged bytes dominate both `display()` output and JSON handling | Revert to `as_encoded_bytes()` for the output bound | `E5`, `E8` |
| G5 projection | Schema-derived field table; one mutation per field allowance reaching the intended owner | Declare three strings instead of four | `E5`, `E7` |
| G5 URL owners | Long percent-encoded shrinking input; exact/minus-one; per-owner omission probes | Restore the blanket multiplier and the false length implication | `E5`, `E7` |
| B1 path owners | Local and remote probes isolated so removal fails | Omit each path owner separately | `E5` |
| B2 pinned authority | Deterministic truncation and parent-replacement outcomes with zero ownership on every branch | Accept replacement without the declared final fence | `E6`, `E9` |
| B3 writer matrix | `WriteZero`, single and repeated `Interrupted`, short-write completion, flush failure, post-flush finish failure; real process channels, both `--locked` profiles, nonzero executed count | Remove each sequence oracle | `E1`, `E8`, `E9` |
| C1-C4 canonical truth | Semantic comparison of each corrected claim against the repaired source | Restore each overclaim after implementation | `E4` |

Also still owed from the resume preflight, unchanged by this adjudication: the
five fixed-limit subject axes (`AggregateBytes`, `ObjectRecords`, response
bytes and counter, `MaximumFrontier`, selected-URL `GraphEdges`) each proved at
`L`, rejected at `L-1`, and rejected with one added subject unit at unchanged
`L`; and independently executed 1x/3x/10x curves, which the report correctly
recorded as not performed in a report-only lane.

### G. Recommendation

**AUTHORIZE_BOUNDED_REPAIR**, after the manager freezes the reduced root-cause
set above.

Reasoning, against the three alternatives:

- Not `REVIEW_CANDIDATE_WITH_EVIDENCE_ONLY`: nine items carry a proved must-fix
  trigger requiring candidate product bytes — R1, R2, R4, R5, R6, R7(b), R8,
  R9(b)(c), R11(b) — grouped into five root causes across four product files.
  R1, R5 and R9(c) in particular are exact ADR-020 clause violations on the
  live command path, and R4 violates a contract the candidate itself declares
  in three places.
- Not `BLOCK_ON_SHARED_OWNER`: the two candidates for an acceptance-critical
  external dependency both fail the test. **R3** carries no proved must-fix
  trigger — ADR-020, the frozen authority, is satisfied, and the demanded
  seventh terminal reason is a cross-surface advertised-profile change that
  policy resolves by rewriting the criterion. **R9's** shared-parser branch is
  not triggered, because `parse_cf_project_url` was read and its owner set is
  exactly enumerable from the storage adapter, satisfying the preflight's own
  first branch. D2 and D4 are boundaries the report already accepts.
- Therefore `AUTHORIZE_BOUNDED_REPAIR`, conditioned on the manager first
  freezing: the five product groups G1-G5; the four evidence items B1-B3 and
  B-REV; the four documentation corrections C1-C4; the four routed dependencies
  D1-D4; the five decision records E1-E5; the unchanged eleven-path write set;
  and a correction to packet clause §R3.3.

**Scale of the reduction.** 23 open items resolve to **5** product repair
groups, **4** evidence items, **4** documentation corrections, **4** routed
dependencies, and **5** decision records. Nine `OPEN` F-labels become **zero**
independent repair units and **one** shared reversal obligation. The
zero-residual reading would have produced 23 work units, at least nine of them
duplicating an R row and at least one (R3) unsatisfiable within the granted
scope.

---

## 5. Exclusions

Not performed, by constraint: Cargo, `rustc`, `rustfmt`, Clippy, install,
Docker, network, dogfood, test binaries, product mutation, and any predecessor
build or executed reversal. No packet, existing report, issue, lifecycle
document, management ledger, candidate path, or canonical source document was
altered. No integration, Close, Install, Dogfood, or Retrospect action was
taken or authorised. The immutable `REQUEST_CHANGES` report is unmodified and
its verdict is preserved.

This artifact assigns dispositions as an independent adjudicator's proposal for
the manager freeze. Per `operating-protocol.md`, the manager owns the final
impact/actionability adjudication and repair authority; nothing here grants
integration authority or approves the candidate.

---

## 6. Exact next action

1. Manager reviews and freezes the reduced set: G1-G5, B1-B3 + B-REV, C1-C4,
   D1-D4, E1-E5, and the unchanged eleven-path write set.
2. Manager records the R3 decision — `Complete` is the ADR-020-conformant
   terminal reason for a publication failure — corrects packet clause §R3.3,
   and opens the D1 successor against `codefire-core` and ADR-020.
3. Manager records the E1, E2 and E5 decision loci and the D2/D3/D4 routings.
4. Only then does the bounded repair resume, under the existing resume
   preflight controls (source-keyed Cargo gate, count-free admission, original
   session `c28925df-d4bc-4994-b905-73aaaf2425e5`), which this report-only lane
   neither satisfies nor waives.
5. The fresh independent rereview receives this adjudication alongside the
   immutable report, the original packet, the control addendum and the resume
   preflight.

---

## 7. Remaining unknowns

1. **No runtime reproduction anywhere.** Every product judgement is source
   proof. R2's and R4's inner faults (a Store release failure; a ledger
   accounting failure) were shown to be *expressible* by the code, not shown to
   occur. Both are classified `DEFECT-INJECTION-ONLY`, which remains blocking
   absent an unreachability proof — but neither has an observed production
   trigger, and that gap is not closed here.
2. **R7(b) severity is bounded but not measured.** That `display()` expands an
   invalid byte to three is certain; by how much a real repository's paths
   exceed the reservation is not quantified without execution.
3. **C3's conditionality is unresolved.** Whether G1/G4 move a boundary
   documented in `source-architecture.md` or `command-source-trace.md` depends
   on the repair's final shape. The report found both currently accurate.
4. **D3 remains a contingency.** The adapter-side enumeration was judged
   feasible from reading `parse_cf_project_url`; an implementer may still find
   an owner that resists bounding. The instruction stands: stop that part and
   record the dependency rather than widen scope.
5. **Control-integrity gap.** The original repair packet
   `…/cfr168-rust-storage-review-closure-repair-20260731.md` is **absent from
   the launch-HEAD lineage** and exists only at `926b179`, which is not an
   ancestor of launch HEAD — as are `C` (`cdcb9bd`) and the report commit `R`.
   The report *blob* is present at HEAD, but the packet is not. A resume
   binding that names the packet by path will not resolve it on this lineage.
   The manager must decide whether the packet is ported additively or bound by
   blob SHA `c5a7bca3ad56d1b9643d78c3db503caf761abea3` / file SHA-256
   `119ec9cf…`. Not adjudicated here; flagged as blocking for the resume
   envelope, not for this adjudication.
6. **Whether the manager accepts the R3 reading.** If the manager instead
   affirms the packet's literal clause, R3 converts from
   `out_of_scope_routed_dependency` to a blocked dependency and the
   recommendation would become `BLOCK_ON_SHARED_OWNER`. That single decision is
   the only input that changes the recommendation.

ADJUDICATION: AUTHORIZE_BOUNDED_REPAIR
