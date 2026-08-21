# Current Cycle Burn-Down Plan

**This file is historical.** Live execution is `docs/development/milestones.md`. Do not implement remaining review-portfolio roots because they appear below.

この文書は、2026-07 の review-portfolio burn-down の記録と、ドキュメントチェッカーが要求する ledger 行を残す。新しい作業許可はここから出さない。

## Policy

この運用は 2026-08-21 に `milestone-operating-model.md` へ置き換えた。残作業は「全部消してから次」ではなく、次マイルストーンに必要なものだけを進め、残りは M2–M5 へ送る。

## Current Cycle State

Current known issue count and lifecycle position, audited 2026-07-19 after CFR-167 Retrospect:

- CFB/CFR product/review issues: 8 open/investigating.
- CFB detail ledger: 137 total; 135 `fixed`, 2 `investigating`.
- CFR detail ledger: 175 total; 159 `fixed`, 10 `fixed in v0.7`, 6 `investigating`.
- The CFR-167 Install gate produced `/tmp/codefire-cfr167-final.KWa8TC/bin/codefire`; source and installed SHA-256 are `1483874b4c1de4a78478595d5d158cf34a12e5ec3b05255dfc8819f511f00272`.
- Algorithm source tests pass 281/281. Installed schema 4 scan/verify removed the false fixture-string duplicates and sealed canonical bridge `CF-COMMIT-e5ca290905c2a4b5e2c05bd4` from legacy parent `CF-COMMIT-1bc27ac05b3a2f8e9d9c1de3`.
- `CFB-120`..`CFB-132` are fixed; final canonical state is `open-clean`, Fire 0, tool migration none. The separately configured repository's branch/open/active control hashes remain unchanged.
- RVC-011/CFR-161 completed Discover through Retrospect. Installed unsafe-policy replay fails before writes, both real read-only matrices pass without hash drift, and canonical trace-warning policy returns a typed passed decision with no blockers.
- CFR-162 completed Discover, Freeze, Plan, Implement, Close, Install, Dogfood, and Retrospect. All unresolved merge classes now block the authoritative decision until an explicit validated continue transition.
- The 60-review current-source audit classified 6 implemented, 3 accepted non-goals, and 51 residual reviews. The residuals remain mapped exactly once to CFR-163..173. Roots through CFR-167/CFB-134 completed the full lifecycle; open CFB-133/136 and CFR-168..173 remain `investigating`. After the second 2026-08-21 slot shift, CFR-170 is the active M1 contract input, CFB-133/CFR-169 await M1 classification, CFR-168/CFB-136 are parked on M2, and CFR-171..173 on M3. `BATCH-2026-07-review-portfolio-completion` is historical; live work is `milestones.md` M1 `MS-agent-local-contract`.
- Historical cycle sections were moved to `archive/completed-cycle-ledger.md`. Do not infer current issue counts from historical headings or remaining-count notes.
- The management hub `docs/development/README.md` is the current-state entry point; update it and append `history.md` when this section changes.

## Active Review-Portfolio Completion Batch

Batch ID: `BATCH-2026-07-review-portfolio-completion`

Lifecycle: historical. The first five ordered roots completed through Retrospect. Remaining roots are parked on `milestones.md`; after the 2026-08-21 slot shift that is CFR-169/170/CFB-133 → M2, CFR-168/CFB-136 → M3, and CFR-171..173 → M4. Do not resume CFR-168 as M1.

Frozen scope:

- Dogfood/admitted blockers: active `CFB-133` and CFR-168 same-root `CFB-136`, plus fixed `CFB-134`/`CFB-135`.
- Current-source review roots: fixed `CFR-163`..`CFR-167` and `CFR-174`/`CFR-175`; active `CFR-168`..`CFR-173`.
- RVC terminal audit: 60/60 mapped exactly once; 6 implemented, 3 accepted non-goals, 51 promoted to the eleven root owners.

Root execution order:

| Order | Root owner | Consumes | Required outcome | State |
|---:|---|---|---|---|
| 1 | CFR-163 + CFB-135 | RVC-012/024/027/028/037 | one verified local authority snapshot and replayable partial transitions, with Doctor accepting extinguished Fire history | fixed; 532-test gate, fresh Install, old/fresh fixture replay, real/canonical Dogfood, independent ACCEPT |
| 2 | CFR-164 | RVC-005/018/038/041/043/046 | complete request/execution subjects, key/nonce/secret/resource trust | fixed; 562 Rust / 94 Python gates, fresh Install, isolated/real/canonical Dogfood, independent ACCEPT |
| 3 | CFR-165 + CFR-174 + CFR-175 | RVC-026/030/044/050/056 | one contained remote lock/idempotency/MR/GC publication protocol whose apply head is the immutable reviewed source | fixed; 596 Rust / 115 Python gates, fresh Install, installed race reversal, real/canonical Dogfood, independent ACCEPT |
| 4 | CFR-166 + CFB-134 | RVC-006/007/013/017/020/022/032/033/049/053/055/057/059 | complete version/capability/schema/conformance contract and Rust remote doctor | fixed; 613 Rust / 117 Python gates, fresh Install, file/HTTP/future/cursor reversal, canonical Dogfood |
| 5 | CFR-167 | RVC-008/016 | typed effective-policy provenance and ChangeCoverage | fixed; 622 Rust / 117 Python gates, fresh Install, installed policy/coverage/seal reversal, canonical Dogfood |
| 6 | CFR-168 + CFB-136 | RVC-014/015/029/042/047/048/058 | bounded graph/object/HTTP work with stable counters and snapshot-safe validation reuse | parked on M3 |
| 7 | CFR-169 | RVC-031/051/052/054 | behavior-preserving lifecycle/Store/test proof seams | parked on M2 |
| 8 | CFR-170 + CFB-133 | RVC-034/035 | typed postcondition-driven outcomes and executable actions | parked on M2 |
| 9 | CFR-171 | RVC-001/019 | machine-readable claim/proof descriptor | parked on M4 |
| 10 | CFR-172 | RVC-002/009 | bounded read-only adoption/mental-model experiment | parked on M4 |
| 11 | CFR-173 | RVC-003/004/010 | privacy-preserving product falsification pilot and stop decision | parked on M4 |

Implemented/non-goal terminal rows:

- implemented: RVC-011, RVC-021, RVC-023, RVC-025, RVC-036, RVC-045;
- accepted non-goal: RVC-039 shared auto-resolution/assessment beyond the CFR-162 safety claim, RVC-040 exploratory Stage-5 help study, and RVC-060 as a non-authoritative auxiliary roadmap.

Batch rules (superseded 2026-08-21):

- remaining rows are parked, not a reason to keep the old burn-down running;
- new work follows `milestone-operating-model.md`;
- only `block-m1` review findings are required for the active milestone.

Completed CFR-163 root:

- `VerifiedOpenSnapshot` now owns branch/opened/marker/active identity and typed pending/divergence across all open consumers before writes;
- commit reset, state publication, and Fire/Resolution publication use exact-request durable reducers; released replay is read-only and close/discard reject pending authority before destructive transactions;
- Doctor counts only `status=open` Fires and reports pending commit/state/active-ledger authority with typed safe inspection and exact owning retry;
- Rust 532/532, source/project docs, fresh Install, original CFB-135 replay, fresh full lifecycle, both divergence directions with no authority hash drift, two real read-only matrices, Algorithm 281/281, canonical scan/verify, and independent ACCEPT all pass;
- no new blocker was found in Retrospect. The batch then advanced to CFR-164 subject trust.

Completed CFR-164 root:

- remote request signature v2 binds the complete canonical payload and versioned mutation subject across Rust/Python and file/HTTP; key windows and strict UTC skew precede nonce publication, while live nonce entries are never capacity-evicted;
- verification/evidence use bounded streaming execution, minimal or explicit inherited-snapshot environments, contained cwd, input/environment/worktree digests, process cleanup scope, and pre-persistence secret/URI/path redaction;
- live assessment subjects bind scan plus canonical worktree manifest and are rechecked through branch publication; context, explain, verify, certificate, and immutable review pairs project the applicable identity;
- Rust 562, Python 94, demo, Algorithm 281, source/project docs, fresh Install, isolated installed trust matrix, real read-only 12/12, canonical scan/verify, and independent ACCEPT all pass;
- independent review found and closed the Python default-skew/local-time mismatch before final acceptance. CFR-165, CFR-166/CFB-134, and CFR-167 subsequently completed; CFR-168 is the next authorized root.

Completed CFR-165/CFR-174/CFR-175 implementation plan:

| Phase | Owner boundary | Required evidence before advancing |
|---:|---|---|
| P0 | ADR-017 | independent design review accepts project capability, all-runtime gate, all-mutation journal, MR subject/escrow, and GC recovery; no code mutation before this freeze |
| P1 | `remote/project.rs`, `remote/locks.rs`, signatures | strict component golden matrix; pinned no-follow project bootstrap/descriptor; shared/exclusive persistent gate; nonce lock crash recovery; process/symlink/hard-link/deadline fixtures |
| P2 | `remote/publication.rs` | fixed pending authority; staged bounded postimages; exact pre/post reducer; keyed v2 receipt plus v1 adapter; unkeyed crash convergence; every write-before-progress fault |
| P3 | `remote/operations.rs` | file/HTTP shared upload/merge/review/apply planners; canonical object-set digest; versioned MR subject/review binding; target-local cross-project escrow; fixed reviewed apply head |
| P4 | remote readers and HTTP adapters | shared-read pending check for list/show/clone/bundle/MR/doctor paths; no partial branch/MR view; current `FileLock` and duplicated handler reducers removed |
| P5 | Python fallback and GC | Unix cross-runtime gate/journal parity or platform fail-closed; durable GC plan/delete progress/atomic audit; pending roots and unknown-version blocking; Python file-server writers participate |
| P6 | evaluation and close | race/fault/parity suites, full Rust/Python/docs gates, fresh isolated Install, old counterexample reversal, real read-only matrix, canonical Algorithm Dogfood, independent High Close review, issue/map/history/Retrospect synchronization |

Phase ordering is strict. P1 establishes contained authority and ownership before P2 writes a journal; P2 establishes
recovery before P3 moves mutators; P3 completes before readers claim coherent visibility; Python/GC parity must pass
before source Close. A blocker found inside these phases is admitted only under the same order-3 root.

All P0-P6 evidence passed. Final source counts are 596 Rust and 115 Python tests; fresh install
`/tmp/codefire-cfr165-final.ln9Oa7` matches source bytes, installed same-key review reversal produces
one effect/receipt with pre-write loser conflict, real read-only probes pass 12/12 without control
hash drift, Algorithm 281/281 plus installed scan/verify pass with canonical head unchanged, and the
independent final review reports no blocker. Retrospect found no new issue.

Completed CFR-167 root:

- Core owns stable effective-policy rules and typed file-relation coverage; both use deterministic normalized digests and preserve explicit default/repository provenance;
- scan, verify, diagnostic, explain, commit plan, sealed roots, and certificate project the same contracts or their bound digests, while Store rejects semantic or cross-root drift;
- coverage classifies pure non-Atom, deleted Atom, out-of-Atom, mixed unclassified-span, and source-unchanged tool migration without claiming line/byte precision or adding a new blocker;
- Rust 622, Python 117, demo, Algorithm 281, fmt/Clippy/source/project docs, fresh Install, installed reversal, sealed digest/Doctor checks, and canonical scan/verify all pass;
- recovery-shape and diagnostic-count regressions found during the full gate were fixed under the same root. Retrospect found no new blocker, so CFR-168 is authorized next.

## Completed Unresolved-Merge Sealing Batch

Batch ID: `BATCH-2026-07-unresolved-merge-sealing`

Lifecycle: Discover, Freeze, Plan, Implement, Close, Install, Dogfood, and Retrospect complete.

Frozen scope:

- `CFR-162`: unresolved text, binary, and semantic merge facts must block the authoritative decision until an explicit, validated resolution transition.

Deduplication:

- CFR-162 consumes ADR-013 rather than creating a second commit predicate.
- Only the sealing-safety slice of RVC-039 is promoted; its shared diff/merge assessment experiment remains candidate work.
- CFB-104 merge-Fire timing and CFB-118 dry-run next actions remain closed and are not reopened.

Counterexample:

- a three-way `README.md` conflict writes `<<<<<<< target` markers and non-empty `merge_conflicts`;
- Rust verify still returns a committable decision;
- focused evidence: `unresolved_text_merge_conflict_blocks_verification_and_commit_preview` fails at the committable assertion before the fix.

Required outcome:

| Order | Issue | Invariant | Close state |
|---:|---|---|---|
| 1 | CFR-162 | unresolved merge facts are typed decision blockers with exit 15, and only a validated `merge --continue` transition can clear them before a two-parent seal | fixed; ADR-014, 495-test gate, fresh Install, installed replay, real/canonical Dogfood pass |

Exit gate:

- text/binary/semantic regression matrix, malformed metadata and no-write blocked paths;
- total core/CLI/store projections and legacy empty-field compatibility;
- full source gate, fresh isolated Install, installed counterexample reversal, real-repository read-only probes, and Retrospect;
- no next review root promoted until CFR-162 is fixed or explicit wontfix. This gate is satisfied; RVC-027 is the next confirmed Discovery candidate.

Close evidence:

- focused text/binary/semantic, one-sidecar, human/explain, malformed-path, symlink, identity, nonpublication, and exact two-parent regressions pass;
- locked workspace gate passes 495 tests: 367 CLI/integration, 55 core, 70 store, and 3 util; fmt, warning-denying Clippy, source/project docs, and diff checks pass;
- fresh `/tmp/codefire-cfr162-closed.YMG1ed/bin/codefire` has SHA-256 `547e385dd5110c0d72ca9fa2e119de3f50684b810bb25fdbf22107e0df44e1d6`, 34 commands, working Rust/Python help, and bash/zsh completion;
- `/tmp/codefire-cfr162-closed-repro.0daNIh` proves all blocked process/envelope exits are 15, authority hashes remain unchanged, legitimate Markdown `=======` is accepted, and resolved commit `CF-COMMIT-f776a3c2ec9f1d90d3eff5f5` has the exact target/source parents;
- `/tmp/codefire-cfr162-closed-readonly.U9wREV` passes 12/12 with both real repositories' branches/opened/active hashes unchanged;
- canonical installed status/scan/verify and Algorithm 281/281 tests pass while head remains `CF-COMMIT-e5ca290905c2a4b5e2c05bd4`; configured state is not mutated.

## Completed Authoritative-Decision Batch

Batch ID: `BATCH-2026-07-authoritative-committable-decision`

Lifecycle: Discover, Freeze, Plan, Implement, Close, Install, Dogfood, and Retrospect complete.

Frozen scope:

- `CFR-161`: one typed committable decision, non-relaxable safety floor, and total core/CLI/store projections.

Deduplication:

- RVC-011 is the formal owner.
- RVC-008 policy applicability, RVC-016 non-Atom decision input, and RVC-034 exit/diagnostic semantics are consumers of CFR-161, not separate issues.
- unresolved merge semantics, migrate post-apply actions, and remote doctor URL routing remain outside this batch.

Runtime counterexample:

- `/tmp/codefire-rvc011-repro.LtKS5J` with `require_no_required_fires: false` and one required Fire;
- verify exit 0, `result=passed`, `ok=true`, one `blocking=true` diagnostic, next action `commit`;
- commit dry-run exit 0, `blocked=false`, six planned mutation operations;
- evidence `/tmp/codefire-rvc011-{verify,commit-dry}.json`.

Required outcome:

| Order | Issue | Invariant | Close state |
|---:|---|---|---|
| 1 | CFR-161 | required Fires, stale resolutions, missing evidence, duplicate Atom IDs, and required external checks can never be downgraded by policy; one typed decision owns outcome, reason priority, and every projection | fixed; ADR-013, 484-test gate, fresh Install, old-repro reversal, real/canonical Dogfood pass |

Exit gate:

- complete blocker/policy/multi-reason table in core;
- unsafe working config and strict sealed policy fail closed without writes;
- verify exit/envelope/diagnostics/actions/text/state, commit gate/certificate, and store validation agree;
- full source gate, fresh isolated Install, counterexample replay, canonical/configured read-only probes, Retrospect;
- no second review root promoted until CFR-161 is fixed or explicit wontfix.

Close evidence:

- 484 Rust tests pass: 358 CLI/integration, 54 core, 69 store, 3 util; fmt, check, warning-denying Clippy, source/project docs, and diff checks pass;
- installed source identity, 34-command capability, Python help, and bash/zsh completion pass at `/tmp/codefire-cfr161-final.jsIkTv`;
- installed verify/commit dry/live all reject the old counterexample with exit 2 and unchanged authority hashes;
- `/tmp/codefire-cfr161-readonly.Igi3yL` passes 12/12 with both real repositories unchanged;
- canonical `/tmp/codefire-cfr161-canonical-verify.json` reports passed, no primary/blockers, and only `missing_required_links` warnings while head remains `CF-COMMIT-e5ca290905c2a4b5e2c05bd4`.

## Completed Stage 1 Dogfood-Blocker Batch

Batch ID: `BATCH-2026-07-stage1-invariants-dogfood-blockers`

Frozen scope:

- `CFB-120`: scan sealed-base validation;
- `CFB-121`: pre-publication sealed-commit validation;
- `CFB-122`: resolved path/symlink containment;
- `CFB-123`: Trace Graph endpoint integrity.
- `CFB-124`: live scan/verify repository-state locking;
- `CFB-125`: short-ID collision full-digest comparison.
- `CFB-126`: import idempotency request binding.
- `CFB-128`: versioned legacy Trace/extractor semantic recovery;
- `CFB-129`: single-owner doctor/migrate JSON failure rendering.
- `CFB-130`: implemented, locked, idempotent migrate-apply transition.
- `CFB-131`: exact legacy/current public object identity across remote graph transfer.
- `CFB-132`: string-aware Python derived-symbol extraction and versioned hash semantics.

CFB-120..CFB-126 were the original source-audit scope. Required Install and Dogfood added only the
blocking CFB-128..132 issues to the same frozen batch. All twelve entries are now fixed, the final
installed binary passed both real-repository and isolated mutable workflows, and the canonical
history was bridged by a normal child commit without rewriting the legacy parent.

## Batch Execution Gate

| Order | Issue | Required outcome | Close state |
|---:|---|---|---|
| 1 | CFB-120 | `scan` rejects an unsealed or structurally invalid base commit before deriving active state. | fixed; pinned sealed-token regressions pass |
| 2 | CFB-121 | `commit` validates the complete candidate commit graph before publishing the branch head. | fixed; graph and local-signature regressions pass |
| 3 | CFB-122 | every user-controlled path is contained after canonicalization and symlink resolution. | fixed; capability/identity replacement regressions pass |
| 4 | CFB-123 | Trace Graph endpoints resolve to existing Atoms or return an explicit blocking diagnostic. | fixed; dangling-endpoint regression passes |
| 5 | CFB-124 | live `scan` and `verify` mutations reload state after acquiring the repository lock. | fixed; shared-lock and phase-recovery regressions pass |
| 6 | CFB-125 | abbreviated object lookup compares full digests and reports collisions deterministically. | fixed; existing-path and link-race collision regressions pass |
| 7 | CFB-126 | import idempotency binds the key to the complete normalized request. | fixed; exact-replay and request-conflict regressions pass |
| 8 | CFB-128 | authenticated legacy extractor history has an explicit semantic profile and a non-destructive bridge to a strict current commit. | fixed; bridge E2E and both real-repository read-only status probes pass |
| 9 | CFB-129 | unhealthy doctor and incompatible migrate JSON each emit one complete command-result document. | fixed; single-parse JSON integration passes for doctor and migrate check/dry-run/apply |
| 10 | CFB-130 | the documented migrate apply transition safely and idempotently executes a freshly locked recovery plan. | fixed; 12 lock/capability/idempotency/concurrency regressions pass |
| 11 | CFB-131 | remote graph transfer preserves every validated legacy/current public object ID exactly. | fixed; exact-record Store and legacy upload/clone/receive regressions pass |
| 12 | CFB-132 | Python derived extraction ignores declaration-like text inside strings/comments and remains versioned. | fixed; schema 4 regressions and canonical installed bridge pass |

All twelve issues are closed. This batch must remain immutable historical scope; new review work
starts with a new Discover/Freeze record.

## Required Validation

The 2026-07-12 source Close gate for CFB-120..126 passed:

```bash
PATH=/tmp/codefire-rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH cargo fmt --all -- --check
PATH=/tmp/codefire-rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH cargo clippy --workspace --all-targets --locked -- -D warnings
PATH=/tmp/codefire-rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH cargo test --workspace --locked
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_source_docs.py
```

The locked workspace test run passed 337 CLI, 41 core, 61 store, and 3 util tests. Clippy with warnings denied, formatting, and both document checkers also passed.

The first isolated install also passed, including source/installed binary identity, help/version,
capabilities, Rust/Python fallback, and shell completion. It is retained only as the failing Dogfood
reproduction binary.

The CFB-128..130 close gate passed: 354 CLI/integration, 50 core, 65 store, and 3 util tests (472
total), all 89 Python fallback tests, warning-denying Clippy, formatting, and both documentation
checkers. Source-tree read-only `status` also passes on both real repositories without changing the
configured repository's branch/open/active control-file hashes. Next, reinstall from the closed
source and execute the original read-only probes plus the isolated mutable Dogfood matrix. Do not
infer installed-binary recovery or compatibility from source-tree tests alone.

The fresh reinstall then passed source/installed identity, help, capability, Python fallback, and
bash/zsh completion checks. Installed read-only status/branch/doctor/migrate/storage probes passed
for both real repositories without control-state mutation. In the independent mutable sandbox,
`migrate apply` and exact replay passed, but the first legacy file-remote upload failed at CFB-131;
the same frozen batch therefore returned to Plan/Implement.

CFB-131 then closed with Store-owned exact-record publication and remote copy/receive integration.
The source gate now passes 355 CLI/integration, 50 core, 68 store, and 3 util tests (476 total), all
89 Python fallback tests, warning-denying Clippy, formatting, and both documentation checkers. The
original partial sandbox retries upload without identity drift, remote-clones the full legacy graph,
and creates a strict current-profile bridge. A fresh install is the next gate.

That fresh install completed the full independent mutable matrix, including invalid graph and lock
failure non-mutation, signature recovery, import replay, and two-parent branch merge. Canonical
Algorithm tests then passed 281/281 and scan reported source unchanged/Fire 0, but verify found two
false duplicate Atoms parsed from triple-quoted fixtures (CFB-132). The branch head was not changed;
the same batch returned to Plan/Implement.

CFB-132 closed with a linear string/comment-aware Python lexical view and Atom hash schema 4. The
final source gate passes 355 CLI/integration, 51 core, 68 store, and 3 util tests (477 total), all 89
Python fallback tests, warning-denying Clippy, formatting, and both documentation checkers. The
fresh installed binary passes source identity, 34-command capabilities, Python fallback, bash/zsh
completion, and the read-only 12-command matrix at `/tmp/codefire-cfb132-readonly.Z311yO`.

Canonical Algorithm tests pass 281/281. Final installed scan/verify reports schema 2 to 4 tool
migration, source changes 0, Fires 0, duplicate IDs 0, and blocking diagnostics 0. Live commit and
exact idempotent replay created `CF-COMMIT-e5ca290905c2a4b5e2c05bd4`; its parent is the unchanged
legacy head, root schema is 2, Atom hash schema is 4, and semantic profile is
`sealed-snapshot-v1`. Post-commit scan/verify/doctor/storage pass with `open-clean`, Fire 0, and no
remaining tool migration. The configured repository was never mutated.

Next discovery candidates, explicitly outside the closed batch:

- successful first `migrate apply` retains the pre-apply `planned_actions` and recommends
  `review_migration_plan` even when `remaining_actions` is empty;
- Rust `doctor cf://...` routes the remote URL through local path discovery and returns `io_error`,
  although the remote surface and documentation make the URL appear supported.

RVC-011 characterization produced the installed counterexample above and is now promoted as
CFR-161. RVC-008/RVC-016/RVC-034 are deduplicated into that one authoritative
committable-decision batch.

## Historical Evidence

Completed batches, the full historical issue inventory, root-fix groups, and dogfood Cycles 4–124 are preserved in `docs/development/archive/completed-cycle-ledger.md`. Later dogfood history remains append-only in `docs/development/history.md` and the dogfood project's own history.

Do not reintroduce completed cycle narratives into this active plan.
