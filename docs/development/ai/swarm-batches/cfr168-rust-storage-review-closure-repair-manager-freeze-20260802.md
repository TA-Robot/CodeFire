# CFR-168 Rust Storage Review Closure Repair — Manager Freeze

Date: 2026-08-02

Status: repair scope frozen; implementation resume is authorized only after the
AIA-048 source-keyed live-sampler repair is accepted and available on the
implementation lineage.

Role: manager repair authority. This document supersedes conflicting repair
requirements but does not modify or relabel any immutable review artifact.

## Bound inputs

| Input | Exact identity |
|---|---|
| Rejected product candidate | `cdcb9bdefe7d25a118a9dc14171864d75f5ad304` |
| Candidate tree | `1836a83aa42df026f578b134f4b455830b873a3f` |
| Candidate parent | `e9e42aae9eb3c5a0993ca07688ade5d4d6e0eed3` |
| Immutable review | `71222e1cec1b0dc10003845428abfb611ba8bf71`; verdict `REQUEST_CHANGES` |
| Immutable review file SHA-256 | `6438df445b2d05510895fd23f73a4e71ba49238da15b00e86abf1a477b5ab960` |
| Original repair packet | `docs/development/ai/swarm-batches/cfr168-rust-storage-review-closure-repair-20260731.md` |
| Original packet file SHA-256 | `119ec9cf3055706e456c5a6090f32235e3db19260ee35f06e5814280643bec87` |
| Impact/actionability adjudication | `docs/development/ai/swarm-results/cfr168-rust-storage-review-finding-adjudication-claude-20260802/adjudication.md` |
| Adjudication file SHA-256 | `e87764f6fb57b72a2581aa87c631a13c20c32a78e2c379a5161c819f89527dc6` |
| Original implementer session | `c28925df-d4bc-4994-b905-73aaaf2425e5` |

The original packet and adjudication now both exist on the canonical lineage.
The control-integrity gap recorded by adjudication section 7.5 is therefore
closed additively; neither input was rewritten.

## Manager decision

`AUTHORIZE_BOUNDED_REPAIR`, limited to the reduced root-cause and evidence sets
below. The immutable `REQUEST_CHANGES` verdict remains valid. Observations that
do not prove a must-fix trigger do not create additional repair authority.

### Correction of original packet R3.3

Original packet R3.3 is superseded. A failed Storage publication must continue
to produce the external `PublishedOutputFailed` outcome, exactly one terminal
observation, and no terminal observation before publication. Within the frozen
`codefire.decision-work.v1` vocabulary, `DecisionWorkTerminalReason::Complete`
is the ADR-020-conformant terminal value. This lane must not add a seventh
terminal-reason variant, edit `codefire-core`, edit ADR-020, or change another
consumer to satisfy the superseded literal wording.

This resolves R3 for this lane as an accepted implementation variance plus a
requirement correction. It authorizes zero Storage product bytes. Any future
proposal to extend the advertised terminal vocabulary is a separate
cross-surface design change and is not acceptance-critical for CFR-168 Storage.

## Frozen required repair set

The implementer must close these five product root causes exactly once:

1. **G1 — traversal graph accounting and frontier ownership.** Correct node,
   edge, stack-capacity, per-frame-shell and identity-reopen accounting in the
   single iterative traversal lifecycle.
2. **G2 — publication output transaction.** Preserve the one-document contract
   when a finish/accounting error occurs after output bytes may be visible.
3. **G3 — typed error-origin classification and precedence.** Replace
   message-text and debug-only classification with typed, exhaustive behaviour,
   including Store semantic validation and pinned-subject change.
4. **G4 — charge before allocation.** Establish command-ledger ownership before
   the four Storage dispatch paths allocate command argument owners.
5. **G5 — portable formula and output dominance.** Correct path-output,
   four-string JSON projection and exactly enumerated remote-URL owner bounds.

The required evidence set is **B1**, **B2**, **B3** and the single shared
**B-REV** residual from adjudication section 4.B. Reversals must fail for the
intended behaviour, not because a symbol is missing. One reversal residual is
recorded once rather than duplicated across F1-F9.

The required documentation set is **C1-C4**. C1, C2 and C4 are unconditional.
C3 is required only if the final G1/G4 implementation changes the documented
graph semantics or ledger-creation boundary.

## Frozen decisions and routed boundaries

- **D1:** closed for this lane by the R3 decision above. No Core or ADR edit.
- **D2:** the Explain-owned projection/publication boundary is accepted and
  requires no Storage repair.
- **D3:** adapter-side URL-owner enumeration is presently feasible. If the
  implementer proves otherwise, stop only that part and report the exact owner;
  do not edit `remote.rs` or widen scope.
- **D4:** the cross-surface publication-transaction family remains owned by its
  separate shared-primitives successor. It creates no additional Storage path.
- **E1:** R3's one-terminal, no-prepublication-terminal and external outcome
  subclaims are accepted as satisfied and must remain regression-protected.
- **E2:** the generalized 256-byte owner shell is accepted conservative product
  behaviour; only its false canonical-contract wording is corrected by C1.
- **E3:** blanket URL over-reservation is nonblocking by itself, but its
  same-symbol exactness correction ships with G5.
- **E4:** wildcard removal is nonblocking by itself, but its same-symbol typed
  exhaustiveness correction ships with G3.
- **E5:** R14 remains a documentation correction because its claims are false,
  not because old zero-residual policy independently authorizes repair.

## Exact write set

Only these eleven paths are authorized:

1. `crates/codefire-cli/src/storage.rs`
2. `crates/codefire-cli/src/main.rs`
3. `crates/codefire-cli/src/secure_worktree.rs`
4. `crates/codefire-cli/src/tests/storage_decision_work.rs`
5. `crates/codefire-cli/tests/storage_report_publication.rs` (create)
6. `crates/codefire-store/src/lib.rs`
7. `docs/development/source-architecture.md` (C3 only, conditional)
8. `docs/development/command-source-trace.md` (C3 only, conditional)
9. `docs/development/source-data-model-catalog.md`
10. `docs/development/change-playbook.md`
11. `docs/development/ai/swarm-results/cfr168-rust-storage-review-closure-repair-20260731/result.md` (create)

Paths 1-6 are product/test implementation paths, paths 7-10 are canonical
documentation paths, and path 11 is result-only. No other path is implied by a
directory named in the old packet. In particular, `remote.rs`, `codefire-core`,
ADR files, lifecycle ledgers, install surfaces and unrelated commands remain
outside this lane.

## Resume and validation controls

1. Resume the recoverable original implementation session
   `c28925df-d4bc-4994-b905-73aaaf2425e5`; do not start a competing Storage
   writer while that session remains usable.
2. Before any Cargo gate, require the accepted AIA-048 implementation on the
   candidate lineage and rerun the source-keyed admission preflight.
3. Retain the original packet's serialized, source-keyed Cargo controls,
   toolchain, nonzero test-selection requirement, exact debug/release coverage,
   fixed-limit axes and 1x/3x/10x evidence.
4. Replace only the superseded R3 oracle: assert the manager decision above and
   the preserved external/one-terminal/no-early-terminal behaviour; do not
   expect a seventh enum value.
5. Implement and checkpoint the five G groups before broad gates. Do not count
   a gate whose exact source key, exit status, selected-test count, result/log
   hashes and artifact provenance are unavailable.
6. Commit product/tests/docs first, then add the durable result in a separate
   result-only commit. Do not amend, squash, rebase, merge or rewrite the bound
   candidate and review inputs.
7. Send the immutable review, original packet, adjudication, this manager
   freeze, implementation commits and exact evidence to a fresh independent
   reviewer. Only a fresh acceptance proposal can return the candidate for
   integration.

## Stop conditions

Stop the affected part without widening scope if AIA-048 has not been accepted,
the source-keyed gate cannot bind exact source, a required owner lies outside
the eleven paths, the candidate identity differs, another writer owns a
serialized path, or a reversal cannot be made behaviourally non-vacuous.

MANAGER FREEZE: `AUTHORIZE_BOUNDED_REPAIR_AFTER_AIA048`
