# Current Snapshot and Candidate-Branch Reconciliation

- Batch: `current-state-closure-audit-20260729`
- Task: `candidate-reconciliation`
- Immutable snapshot: `e2a218e015953f082245f36dbc6aee906b837e5a`
- Snapshot parent / common candidate base: `f1644122134d479333cf422fdb1d48b8fa92dd8e`
- Audit date: 2026-07-29
- Scope: every `agent/cfr168-*` ref, the `cfr168-doctor-object-diagnostics` outlier, `agent/cfr169-ref-registry`, `agent/cfr170-migrate-actions`, `agent/cfr171-claim-descriptor`, `agent/cfr172-readonly-experiment`, and the two required deep frontiers.
- Safety: read-only provenance and conflict forecasting only. No existing ref, index, or worktree was merged, cherry-picked, rebased, reset, or otherwise mutated.

## Decision Summary

The snapshot is not an ancestor/merge integration of the candidate family. It is a one-commit safety checkpoint whose parent is the same `f164412...` base used by all candidate lines. Commit ancestry and `git cherry` therefore report every candidate commit as absent even when its content was manually or squash-integrated. Content-level evidence changes the conclusion materially:

1. **Rust CFR-168 implementation is contained.** The entire `crates/` tree at the snapshot is byte-identical to candidate integration commit `216521891bb81a92625d05b31016c4a4623f4591`. The deep Rust frontier `aa5e57d8b1e22fb83792412c68f255ceaf603e39` adds no Rust source after that snapshot; its later commits are Python, tests, task packets, and history/source-document work.
2. **Python is intentionally frozen at the accepted publication-request-identity boundary.** Snapshot `codefire`, `tests/test_python_existing_effect_input.py`, and `tests/test_python_existing_publication_request_identity.py` are byte-identical to `bde042ef3c6cf58a133e4d109cbd56a5b8b013d6`. The canonical current history records this boundary as independently accepted and explicitly leaves pinned preimage, response/effect terminal adoption, typed reducer, and production handler unwired/frozen.
3. **The deep Python signature/sealed frontier is not a missing integration head.** Its accepted signature/sealed/verification/ancestry material is represented through exact reverse-applicable patches, snapshot tests, and later consolidated source. No patch unique to `e5e6888cb409fdf7eb54dfacf6c407afcd3b9cd4` is classified `needed`.
4. **Response-bridge, typed-reducer, recovery, preimage, filesystem-reducer, and execution-evidence branches are alternative frozen designs, not merge candidates.** Their tests/task packets are intentionally absent at the snapshot and the current issue/history keeps the same downstream wiring frozen. Replaying them would reopen a design-authority decision rather than recover accidentally omitted accepted work.
5. **Doctor/accounting and Store handoff are represented or superseded.** Snapshot Doctor implementation blobs match the doctor outlier tip for `checks.rs`, `opened_registry.rs`, and `report.rs`; the Store handoff's terminal patch is exactly reverse-applicable to the snapshot. No Doctor or Store-handoff patch remains `needed`.
6. **CFR-169, CFR-171, and CFR-172 carry six material missing patch identities.** They remain gated by the active root order and are not authorized for integration here. CFR-170's implementation and focused regression are already byte-identical at the snapshot; its stale branch is superseded.
7. **No candidate-only patch is left `unknown`.** The 315 stable patch identities cover all 374 unique candidate commits. Of the 294 patch identities not represented exactly, 167 are `superseded`, 71 are `alternative-design`, 50 are `docs-only-history`, and 6 are `needed`.

## Inventory Frozen Before Analysis

The ref and commit inventory was created before content classification. The exact scope contains:

- 63 `agent/cfr168-*` refs;
- 1 non-namespaced Doctor outlier;
- 4 CFR-169..172 refs;
- 68 refs total, pointing to 67 distinct tips because `agent/cfr168-execution-evidence-phase-a` and `agent/cfr168-response-input` alias the same tip;
- 374 unique commits reachable from candidate tips and not from the immutable snapshot;
- 315 stable first-parent patch identities after collapsing cherry-picked/parallel duplicates;
- no candidate merge commits except the two explicit Python line merge commits `d64ab8da3246b12f8f9bf687aba5d2dc9414d1fb` and `e1171fbb0ba4ac850a35acc03e824933f1766621`.

Five independent candidate roots have parent `f1644122134d479333cf422fdb1d48b8fa92dd8e`:

| Root | Subject | Family |
|---|---|---|
| `216521891bb81a92625d05b31016c4a4623f4591` | chore: snapshot main CFR-168 integration audit | Rust integration / reducer / execution-evidence |
| `b7c79e623e337c50f1bd8d7dc26c750a0c88f2d0` | chore: snapshot CFR-168 Python HTTP base | Bootstrap / Doctor / Store handoff / CFR-169..172 |
| `552437da08feacd771af7787213543f6001f3f8a` | chore: snapshot CFR-168 Python HTTP base | Deep Python signature/sealed/verification |
| `be06655915b9008ea12e932613811683534c398a` | chore: checkpoint Python HTTP worktree | Standalone Python HTTP work |
| `1ee171119885be89514b31bd4a3e2833625ed1f7` | chore: snapshot CFR-168 integration base | Digest-owner side line |

The duplicated `snapshot CFR-168 Python HTTP base` subject belongs to different commits and different lines; subjects and dates were never used as identity evidence.

The two required deep frontiers resolve exactly as follows:

| Frontier | Ref | Parent | Candidate commits | Snapshot relationship |
|---|---|---|---:|---|
| `aa5e57d8b1e22fb83792412c68f255ceaf603e39` | `agent/cfr168-execution-evidence-accounting` | `060285a1935b7fce021b5a0470e546476507434f` | 93 | merge-base `f164412...`; neither side is ancestor of the other |
| `e5e6888cb409fdf7eb54dfacf6c407afcd3b9cd4` | `agent/cfr168-python-existing-consumer-coordinator` | `e0f3ead5b791539d716db3375c70c26b84537760` | 129 | merge-base `f164412...`; neither side is ancestor of the other |

## Mechanical Method

### Ancestry and `git cherry`

For every ref, the snapshot and candidate have merge-base `f164412...`. The snapshot contributes exactly one commit after that base; every candidate contributes its branch-specific range. No candidate tip is an ancestor of the snapshot, and the snapshot is not an ancestor of any candidate tip.

`git cherry <snapshot> <candidate>` reports only `+` lines for all 68 refs. This is expected, not evidence of wholesale omission: the snapshot's consolidated commit has a different patch identity from every individual candidate commit.

### Stable patch identities

Stable patch IDs were computed from each commit's first-parent delta:

```text
git diff --binary <commit>^1 <commit> | git patch-id --stable
```

This produced 315 identities from 374 commits. Duplicate identities correspond to cherry-picked or parallel copies of the same patch. No stable patch group crosses classification categories.

### Snapshot content applicability

Each first-parent patch was checked against a temporary index loaded from the immutable snapshot:

- **reverse-applicable:** exact patch content already exists at the snapshot;
- **forward-applicable:** patch is absent but can be replayed without textual conflict;
- **neither:** snapshot and patch context diverged, requiring tree/blob/test/issue evidence.

Results:

| Commit deltas | Count |
|---|---:|
| reverse-applicable / not forward-applicable | 31 |
| not reverse-applicable / forward-applicable | 21 |
| neither reverse- nor forward-applicable | 322 |
| total | 374 |

The 31 exact deltas collapse to 21 stable patch identities. The 21 cleanly forward-applicable deltas collapse to 13 identities; clean applicability alone does not imply that the patch is currently authorized or semantically required.

### Classification rules

Only patch identities not represented exactly require one of the ticket's five classifications:

- `needed`: material executable/implementation work absent from the snapshot, not superseded, and still relevant to an active gated root;
- `superseded`: behavior or regression already present through a later consolidated implementation, exact blob equality, current test contract, or a later accepted design;
- `alternative-design`: post-boundary frozen, reverted, competing, or production-unwired design work whose replay would reopen architecture/ownership decisions;
- `docs-only-history`: candidate-only narrative, task-log, or source-document history that does not carry missing executable behavior;
- `unknown`: insufficient evidence after ancestry, patch identity, tree, test-name, issue-ledger, and conflict review.

The audit also records `represented` for exact reverse-applicable identities; these are not candidate-only patches and therefore do not consume one of the five required classifications.

## Classification Totals

| Classification | Stable patch identities | Commit occurrences | Meaning at snapshot |
|---|---:|---:|---|
| `represented` | 21 | 31 | Exact candidate delta already exists |
| `superseded` | 167 | 198 | Later consolidated/current behavior replaces the candidate delta |
| `alternative-design` | 71 | 89 | Frozen/reverted/competing design; no mechanical replay |
| `docs-only-history` | 50 | 50 | Historical/narrative candidate-only delta |
| `needed` | 6 | 6 | Material missing work, gated for a later root |
| `unknown` | 0 | 0 | None |
| **Total** | **315** | **374** | Exhaustive |

Candidate-only totals are 294 stable patch identities / 343 commit occurrences.

## Frontier Reconciliation

| Frontier | represented | superseded | alternative-design | docs-only-history | needed | candidate-only total |
|---|---:|---:|---:|---:|---:|---:|
| `agent/cfr168-execution-evidence-accounting` | 1 | 25 | 56 | 11 | 0 | 92 |
| `agent/cfr168-python-existing-consumer-coordinator` | 15 | 82 | 0 | 28 | 0 | 110 |
| `agent/cfr168-response-bridge-owner-fix` | 1 | 25 | 17 | 1 | 0 | 43 |
| `agent/cfr168-typed-reducer-predecessor` | 1 | 25 | 33 | 1 | 0 | 59 |
| `cfr168-doctor-object-diagnostics` | 2 | 32 | 0 | 0 | 0 | 32 |
| `agent/cfr168-rust-store-handoff` | 2 | 19 | 0 | 0 | 0 | 19 |
| `agent/cfr169-ref-registry` | 1 | 4 | 0 | 0 | 1 | 5 |
| `agent/cfr170-migrate-actions` | 1 | 5 | 0 | 0 | 0 | 5 |
| `agent/cfr171-claim-descriptor` | 1 | 4 | 0 | 0 | 2 | 6 |
| `agent/cfr172-readonly-experiment` | 1 | 4 | 0 | 0 | 3 | 7 |

Counts are unique stable patch identities reachable from each frontier; shared roots appear in multiple frontier rows but only once in the global totals.

## High-Risk Family Findings

### Rust integration frontier `aa5e57d...`

- `git diff --quiet 216521891bb81a92625d05b31016c4a4623f4591 e2a218e... -- crates` succeeds: every Rust crate file is identical.
- The snapshot's later source documents differ from `2165218...`; those are current-state documentation, not missing Rust behavior.
- Snapshot `codefire` is byte-identical to `bde042e...`, a commit on the `aa5e57d...` line.
- The canonical current history states that `bde042e...`'s publication request identity slice passed candidate/main full Python comparison and independent re-review, while pinned preimage, response/effect terminal adoption, typed reducer, and production handler remain frozen.
- Therefore the 56 post-boundary alternative patch identities on this frontier are not `needed`. The 11 docs-only identities remain historical evidence; 25 earlier identities are superseded by the consolidated snapshot.

### Python signature / sealed / ancestry frontier `e5e6888...`

- Direct `git cherry` is misleading: all 127 non-merge first-parent deltas appear `+` against the snapshot because the line began from a different bulk root.
- Snapshot retains the focused test files `tests/test_python_signature_terminal_freeze.py`, `tests/test_python_sealed_signature_composite.py`, `tests/test_python_ancestry_borrow_handoff.py`, and `tests/test_python_existing_consumer_terminal_freeze.py`.
- Fifteen stable patch identities are exactly reverse-applicable, including one-shot permits, verification handoff, signature transition retirement, sealed-borrow fences, and ancestry deque/handoff corrections.
- The remaining implementation deltas are superseded by later consolidated Python source and accepted current tests; 28 patch identities are only branch-local documentation/history.
- No signature/sealed/ancestry patch is `needed`, and replaying the whole stale line would conflict in 11 paths including `codefire`, `tests/test_codefire_cli.py`, signature/sealed tests, and canonical source documents.

### Response bridge

- The standalone response bridge and its trunk copy share stable patch identities for the freeze and preacquired-body work.
- The trunk explicitly reverts the preacquired-body freezer before moving to different owner-first/recovery/reducer scaffolding.
- Snapshot omits `tests/test_python_existing_response_bridge_freeze.py`, `tests/test_python_response_bridge_predecessor.py`, and the response-bridge task packet.
- Current history keeps response/effect terminal-token adoption and typed reducer unwired after the accepted request-identity boundary.
- Classification: all candidate-only response-bridge patch identities are `alternative-design`; none are `needed`.

### Typed reducer / recovery / execution evidence

- The typed-reducer branch forks before a competing reducer chain on the deep Rust frontier. Identical subjects sometimes have different patch IDs; subject equality is not patch equivalence.
- Snapshot omits the typed-reducer, recovery, filesystem-reducer, and execution-evidence focused files/task packets.
- These lines are characterization, freeze, alternative predecessor, recovery-classifier, and accounting-authority designs after the accepted Python boundary. They are not production wiring accepted by the current snapshot.
- Classification: candidate-only implementation identities in this family are `alternative-design`; narrative-only deltas are `docs-only-history`.

### Python publication/signature boundary

- Snapshot `codefire` exactly equals `bde042ef3c6cf58a133e4d109cbd56a5b8b013d6`.
- Snapshot `tests/test_python_existing_effect_input.py` and `tests/test_python_existing_publication_request_identity.py` also exactly equal that commit.
- This pins the accepted current behavior more strongly than subject/date evidence and explains why later preimage/reducer branches must not be mechanically appended.

### Doctor and accounting

- The outlier ref is exactly `cfr168-doctor-object-diagnostics` at `902af8988afedba74fef75963aed925637626598`.
- Snapshot blobs for `crates/codefire-cli/src/doctor/checks.rs`, `doctor/opened_registry.rs`, and `doctor/report.rs` equal both `2165218...` and `902af89...` where those paths exist.
- The current CFR-168 record explicitly marks Doctor session, report, layout, marker, conflict, branch, and object-diagnostic slices accepted while retaining later active/open-summary/JSON residuals under CFR-168.
- Candidate Doctor patches are therefore represented or superseded, not missing. A whole-branch merge forecast reports nine conflict paths because the stale line carries older Store/tests/source-doc context.

### Store handoff

- The terminal patch `5397a1650818d70d24354e031d43f1eaf32c8fea` (`fix(store): preserve object conversion precedence`) is exactly reverse-applicable to the snapshot.
- Snapshot's full `crates/codefire-store/src/lib.rs` equals `2165218...`, which contains later consolidated Store work around that patch context.
- The other Store-handoff-line deltas are superseded by current HTTP/Store ownership code. No Store-handoff patch is `needed`.

### CFR-169

- Missing material patch: `25e5e3639b9854aac1f241253ae8e33f105b16e1`, stable patch ID `e92dfce0c6f9165c9c8f52a1e0d9228ea9ba66a0`.
- It moves one remote graph consumer to a Store-owned typed object-reference registry and adds focused golden tests/source-doc ownership.
- Snapshot still has CLI-owned `try_for_each_object_document_reference` / `object_document_references`; the Store-owned `ObjectReferenceShape`/`ObjectReferenceSource` registry is absent.
- The single tail patch is textually forward-applicable to the snapshot, but merging the stale full branch forecasts five conflicts. Classification: `needed`, gated behind CFR-168 and requiring replay/reimplementation of the tail patch, not branch merge.

### CFR-170

- Candidate `f83b53ee...` is superseded. Snapshot `crates/codefire-cli/src/migration.rs` and `crates/codefire-cli/src/tests/migration_apply.rs` are byte-identical to the branch tip.
- Snapshot retains `migration_next_actions_follow_mode_and_current_postcondition` and `retry_remaining_migration_actions`; current `docs/automation_interface.md` contains the same postcondition rule plus later documentation.
- Classification: no needed patch. Whole-branch forecast has only two stale documentation conflicts.

### CFR-171

- Missing material patches:
  - `73025a22c47a6be88ef70cf9563e89ea0d9751a0`, stable patch ID `c86656a339586b538f94fb7c4851e4d5d28792ea`;
  - `a77e9ef992a55a916e0a47acadee22babe83bc7b`, stable patch ID `6de8de800656bb7ed5e5081aad127994c71a49b3`.
- Snapshot lacks `crates/codefire-core/src/claim_descriptor.rs`, the claim-generation fixtures, and `codefire_consistency_claim` symbols.
- The second patch closes the documented schema-generation downgrade; the pair is one inseparable candidate decision.
- Classification: both `needed`, but strictly gated by CFR-168/169/170 integration order. Whole-branch replay forecasts 11 conflicts across Python, Rust CLI/Store, tests, and source docs.

### CFR-172

- The three commits are executable experiment work despite living under `docs/development/`:
  - `fabfea6c4c4a58c758a9a37ff1025ad49036a408`;
  - `ef17eb16c26e29facea51b852d4f95d9a2f4304c`;
  - `91954879749114f8aefce267206e95aaf59cec9e`.
- Snapshot lacks `docs/development/experiments/cfr-172/readonly_adoption.py`, its 23-test harness, experiment schema, example observations, and checker.
- Classification: all three `needed` as one gated experiment checkpoint, not `docs-only-history`. The experiment remains blocked behind CFR-168..171 and is not a product-core merge authorization.

## Conflict Forecast

Forecasts use read-only three-argument `git merge-tree` from each line's true fork point. Counts identify paths requiring manual resolution; they do not prove semantic compatibility.

| Candidate delta | Conflict paths | Main conflicts |
|---|---:|---|
| `bde042e...` → `aa5e57d...` | 2 | CFR-168 issue/history only; executable alternative patches otherwise apply textually |
| deep Python signature/sealed tail | 11 | `codefire`, CLI test, signature/sealed tests, source docs |
| response-bridge owner-fix | 7 | `codefire`, response test, task packet, source docs |
| typed-reducer predecessor | 8 | `codefire`, reducer/effect/shared-adoption tests, task packet/docs |
| Doctor outlier | 9 | Store/CLI tests and canonical source docs |
| Store handoff | 10 | HTTP/Store, ADR-020, source docs |
| CFR-169 tail branch | 5 | CLI remote, Store, module/source docs |
| CFR-170 branch | 2 | automation and command-trace docs |
| CFR-171 branch | 11 | Python/Rust CLI/Store/tests/source docs |
| CFR-172 branch | 2 | development README/history; experiment files add cleanly |

The low textual conflict count for the post-`bde042e...` alternative line is not an integration recommendation. Its code applies cleanly precisely because the snapshot froze at its parent boundary; the canonical issue/history still rejects automatic continuation into those designs.

## Material Missing Patch Set

Exactly six stable patch identities are material, absent, and not superseded:

| Stable patch ID | Commits | Subject | Paths | Representative paths |
|---|---|---|---:|---|
| `e92dfce0c6f9165c9c8f52a1e0d9228ea9ba66a0` | `25e5e3639b9854aac1f241253ae8e33f105b16e1` | refactor(store): own typed object reference traversal | 7 | `crates/codefire-cli/src/remote.rs`, `crates/codefire-store/src/lib.rs`, `docs/development/code-review-issues/cfr-169.md`, `docs/development/module-boundaries.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `c86006dad23ff930b669d29db13ae84004f19a60` | `fabfea6c4c4a58c758a9a37ff1025ad49036a408` | experiment(cfr-172): prepare read-only adoption harness | 11 | `docs/development/README.md`, `docs/development/code-review-issues/cfr-172.md`, `docs/development/current-cycle-burn-down-plan.md`, `docs/development/experiments/cfr-172/README.md`, `docs/development/experiments/cfr-172/consumer-observations.example.json`, `docs/development/experiments/cfr-172/experiment.v1.json`, … |
| `aaae7f1b967a4570091e1318a31a41b7793434c4` | `ef17eb16c26e29facea51b852d4f95d9a2f4304c` | fix(cfr-172): bind hashes to canonical roots | 6 | `docs/development/code-review-issues/cfr-172.md`, `docs/development/experiments/cfr-172/README.md`, `docs/development/experiments/cfr-172/experiment.v1.json`, `docs/development/experiments/cfr-172/readonly_adoption.py`, `docs/development/experiments/cfr-172/test_readonly_adoption.py`, `docs/development/history.md` |
| `b67af6d6a23ca7fcd282c200fad7322a70bbd23e` | `91954879749114f8aefce267206e95aaf59cec9e` | fix(cfr-172): discover canonical plain roots | 6 | `docs/development/code-review-issues/cfr-172.md`, `docs/development/experiments/cfr-172/README.md`, `docs/development/experiments/cfr-172/experiment.v1.json`, `docs/development/experiments/cfr-172/readonly_adoption.py`, `docs/development/experiments/cfr-172/test_readonly_adoption.py`, `docs/development/history.md` |
| `c86656a339586b538f94fb7c4851e4d5d28792ea` | `73025a22c47a6be88ef70cf9563e89ea0d9751a0` | feat: bind verification claim descriptors | 13 | `crates/codefire-cli/src/automation.rs`, `crates/codefire-cli/src/exit_code.rs`, `crates/codefire-cli/src/main.rs`, `crates/codefire-cli/src/tests.rs`, `crates/codefire-cli/src/tests/subject_trust.rs`, `crates/codefire-core/src/claim_descriptor.rs`, … |
| `6de8de800656bb7ed5e5081aad127994c71a49b3` | `a77e9ef992a55a916e0a47acadee22babe83bc7b` | fix: prevent claim descriptor generation downgrade | 25 | `codefire`, `crates/codefire-cli/src/compatibility.rs`, `crates/codefire-cli/src/main.rs`, `crates/codefire-cli/src/tests/conformance.rs`, `crates/codefire-cli/src/tests/subject_trust.rs`, `crates/codefire-core/src/claim_descriptor.rs`, … |

These six patches reduce to three decisions: CFR-169 registry adoption, CFR-171 descriptor adoption (two-patch unit), and CFR-172 experiment preservation/execution (three-patch unit).

## Uncertainties and Boundaries

- Stable patch ID proves textual patch equivalence, not runtime equivalence. The audit therefore combines patch IDs with exact blob/tree equality, test-path/name checks, current issue/history state, and merge-tree forecasts.
- The snapshot is a safety squash, so zero snapshot-history patch-ID matches and all-`+` `git cherry` output are expected. Treating those results alone as omission would be incorrect.
- `git merge-tree` is Git 2.34.1's three-argument simulation. Custom merge drivers or future Git versions can change hunk presentation, but not the identified stale-line overlap risk.
- `superseded` does not assert that every historical test name remains verbatim. It means current consolidated code/tests/ledger represent a later accepted boundary or implementation of that work.
- `alternative-design` is intentionally conservative: these patches may contain useful experiments or counterexamples, but the current canonical record freezes their production connection. They require a new explicit design-authority task if revived.
- No product code was executed for this docs-only audit. Validation covers Git object provenance, exhaustive ledger integrity, Markdown/diff hygiene, and project/AI documentation checkers.

## Appendix A — Exact Ref Tips

The table below is the pre-analysis ref inventory. `Snapshot-only / candidate-only` is `git rev-list --left-right --count <snapshot>...<tip>`.

| Ref | Tip | Snapshot-only / candidate-only commits | Merge base | Tip subject |
|---|---|---:|---|---|
| `agent/cfr168-capability-metrics` | `e2a43381464936ab8ebcf8fce51aa88620195ddc` | `1/6` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | feat(capabilities): advertise decision work coverage |
| `agent/cfr168-consumer-owner` | `3fa4c7e6a0f0e2879f9c124eb0e94de73ccae2eb` | `1/8` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(python): require terminal parent pair bind |
| `agent/cfr168-digest-integration` | `08d26aba539b4297e99936877519922c085d365b` | `1/5` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(remote): remove unreachable accounted compatibility path |
| `agent/cfr168-digest-owner` | `cc75b7f30cacabb2f9bfb84a58561d692ca7ffb3` | `1/4` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(remote): remove unreachable accounted compatibility path |
| `agent/cfr168-doctor-branch-accounting` | `63dea5ec72b97a2fbfb3901a5972ae41b20f5b14` | `1/23` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Bound Doctor branch record reads |
| `agent/cfr168-doctor-conflict-accounting` | `aa94eed872d147f9906c3a0c75d518f8707bdd7b` | `1/22` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Clarify Doctor conflict report path accounting |
| `agent/cfr168-doctor-consumer` | `7545be29fea6a33bcb6eec814edefc7ed3822f4d` | `1/11` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Preserve standalone scan accounting |
| `agent/cfr168-doctor-report-accounting` | `7b6d71baf9f84654b9a7c4bd66e2a68fe7e3f3a5` | `1/20` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Order Doctor open marker candidate reads |
| `agent/cfr168-execution-evidence-accounting` | `aa5e57d8b1e22fb83792412c68f255ceaf603e39` | `1/93` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Document planner function shape authority |
| `agent/cfr168-execution-evidence-phase-a` | `579867b0314ed017ace84be4e901822742719edf` | `1/78` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Append execution evidence history |
| `agent/cfr168-existing-effect-input` | `882b55441cbbe71ecd90efe36447357b8eef7182` | `1/18` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Add existing upload effect input owner |
| `agent/cfr168-existing-filesystem-reducer-packet` | `7bc379b69407b62d4a98f84d1fd5bdac3875910c` | `1/72` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Clarify partial write recovery wording |
| `agent/cfr168-existing-reducer-freeze` | `51894c07876e1947345bce9c76c22c3ac23952a0` | `1/35` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Freeze typed existing upload reducer |
| `agent/cfr168-http-bootstrap` | `74a3eac077a9f500db42ae2d248ba21ba526990a` | `1/11` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(http): preflight missing upload footprint |
| `agent/cfr168-main-integration-audit` | `c71289ecc5b9058c156012c9f70a2aeac103e1f7` | `1/2` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | chore: mirror main integration whitespace fix |
| `agent/cfr168-missing-bootstrap-transaction` | `5910383ae980424460dd436b26d9f72a03515df9` | `1/17` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Harden missing bootstrap callback fences |
| `agent/cfr168-pair-semantic-carrier` | `864b27b665e6779e0ff4777102edfa156e296375` | `1/13` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | feat(python): add pair semantic carrier |
| `agent/cfr168-preimage-baseline-freeze` | `a8ec6d6d8f0547b4d9abe6fa9da7a22915325b8d` | `1/31` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | test: freeze sealed preimage predecessor |
| `agent/cfr168-preimage-owner` | `429173f16ff922270899fcd8a1e1c2c61701903b` | `1/27` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | test: freeze preimage accounting contract |
| `agent/cfr168-preimage-terminal-freeze` | `297aa57132db4259925f2fbae5fb4358b0ed2f3c` | `1/31` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Freeze preimage terminal probe contract |
| `agent/cfr168-preimage-tests` | `be4484caa0416ea852c43148dd21f5e2a39c1081` | `1/27` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Freeze gate-relative preimage proof contract |
| `agent/cfr168-python-admission` | `3681cb80e0ee326260a353307cec1bac810e3869` | `1/19` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(cfr-168): roll back failed HTTP registration |
| `agent/cfr168-python-ancestry-borrow-handoff` | `b6ff7055334525917e357d1989dd94f8b27b4abb` | `1/5` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(python): make ancestry handoff two-stage |
| `agent/cfr168-python-ancestry-owner` | `d952746007391de403810c515d1c6eb6ccbefbaf` | `1/72` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | docs: record ancestry handoff fault evidence |
| `agent/cfr168-python-existing-consumer-coordinator` | `e5e6888cb409fdf7eb54dfacf6c407afcd3b9cd4` | `1/129` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Freeze existing consumer terminal coordination |
| `agent/cfr168-python-external-production` | `4ab58a188c8ec39a3502e1b60a12bd8d285686d9` | `1/65` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | docs: record upload verification production wiring |
| `agent/cfr168-python-external-verification` | `6d76cd686f22fe54525ed0135babfd68ec69a050` | `1/62` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | docs: record verification final evidence |
| `agent/cfr168-python-http` | `e89ac53ea1651e1a619bd7210bca27e416b3c780` | `1/2` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | chore: checkpoint CFR-168 bootstrap integration |
| `agent/cfr168-python-http-work` | `73e2003339dc82326d195d8bcb2bf72a05fbf74f` | `1/12` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Clean HTTP owners on base exceptions |
| `agent/cfr168-python-integration` | `715907e099f188e21156801f6ccd886153ba8285` | `1/20` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Poll Python bundle hash work by deadline |
| `agent/cfr168-python-mutation-response-preflight` | `ffb902201bd55f9984ee3b51fb572532b5b446a3` | `1/20` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Preflight Python upload mutation responses |
| `agent/cfr168-python-object-digest-handoff` | `a32bf5c9ab0f8913780c8349591aaa083f291352` | `1/46` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | docs: close missing bootstrap gate handoff |
| `agent/cfr168-python-object-digest-owner` | `ffee5189da4cf514abf3ce57e7bfb1e48966fd0f` | `1/42` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(python): detach digest failure chains |
| `agent/cfr168-python-pair-detach-owner` | `e8545f192fa1f9b88bb70a47852b91e6e9c8d0c6` | `1/11` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix: pin pair detach subject across callbacks |
| `agent/cfr168-python-pre-effect-validation` | `8d5ffaddff938efb6b4a90037b2b6fe486fc729b` | `1/41` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(python): contain upload stage payloads |
| `agent/cfr168-python-sealed-borrow` | `42eb2c7a2ab207902c5e3e3cddfe551c4f5dff49` | `1/74` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | docs: record final detach close guard |
| `agent/cfr168-python-sealed-owner` | `aa8d55bdcb4cc576eaa3e19bb3620d400a1aa779` | `1/51` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Document single-use descriptor close policy |
| `agent/cfr168-python-sealed-production` | `0c83597bc12a1f7108f42669094b711652ebfb8b` | `1/54` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | docs: record sealed upload production wiring |
| `agent/cfr168-python-sealed-signature-composite` | `689477f5aa975783cb6219b762dadf488a7bcff9` | `1/122` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Add sealed signature verification composite |
| `agent/cfr168-python-signature-authority` | `64c9b6e1c8640956b7a169bce5c232451811e08a` | `1/54` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(python): sanitize signature policy parse failures |
| `agent/cfr168-python-signature-owner` | `56303e3080b65148367b4c0e503815f614dbb5e8` | `1/48` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | docs: record failed reservation retirement |
| `agent/cfr168-python-signature-terminal` | `708ae5bf0009706f21b6aad6ed32a43f7fb871a5` | `1/93` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | test(python): freeze signature terminal ordering |
| `agent/cfr168-python-upload-semantic-owner` | `bb426f809161fa8e39789504b779ca4e7601290c` | `1/5` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(python): account semantic fingerprint scratch |
| `agent/cfr168-python-verification-plan` | `6c06aa2a69d6a4f53c84c52f534a411256324748` | `1/121` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(python): guard full verification handoff |
| `agent/cfr168-recovery-authoritative-classifier` | `a79432490a23c46efd2087dd4fa01cbe3187c4c4` | `1/57` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | harden recovery terminal state snapshot |
| `agent/cfr168-recovery-composition-freeze` | `a816bc191d077455b8c84e5a8bcb56cd473704cb` | `1/54` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Correct recovery composition Freeze |
| `agent/cfr168-recovery-proof-freeze` | `3016656cbe043c9a799d41b39b397a6a2983d9b1` | `1/38` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | test: freeze existing upload recovery proof |
| `agent/cfr168-recovery-proof-impl` | `35f136757c0757cb38d496e2bac39cfae1433c0a` | `1/43` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Account terminal recovery scan overflow |
| `agent/cfr168-recovery-stage-proof-freeze` | `3a6fca39cd90b3b6309a953fe060982d30c4b6c9` | `1/52` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Freeze recovery effect evidence gaps |
| `agent/cfr168-reducer-current-integration` | `e87af3f49dcc1ab0906407de3610b1e18fffc4c3` | `1/69` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Pin reducer input authority externally |
| `agent/cfr168-response-bridge-freeze` | `8fff6719aa37eff9041255d86b97c877d5992e86` | `1/38` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Freeze existing upload response bridge |
| `agent/cfr168-response-bridge-impl` | `d7270dd7d33879e98d8c7ced5c6dec5d1f15eced` | `1/40` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Add preacquired response body freezer |
| `agent/cfr168-response-bridge-owner-fix` | `1c1b752d99723038de3f6670fdd8e9518915bd04` | `1/44` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Harden response predecessor retries |
| `agent/cfr168-response-composition-freeze` | `da778edddfae8d5ed9eccb53f2d31d8a71addd2c` | `1/19` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Freeze existing response composition boundary |
| `agent/cfr168-response-input` | `579867b0314ed017ace84be4e901822742719edf` | `1/78` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Append execution evidence history |
| `agent/cfr168-rust-request-body-owner` | `aeacd096f06a56572e29900161efe290669eca6f` | `1/12` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | refactor(http): bind parsed request body ownership |
| `agent/cfr168-rust-store-handoff` | `5397a1650818d70d24354e031d43f1eaf32c8fea` | `1/21` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(store): preserve object conversion precedence |
| `agent/cfr168-semantic-input-owner` | `e8545f192fa1f9b88bb70a47852b91e6e9c8d0c6` | `1/11` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix: pin pair detach subject across callbacks |
| `agent/cfr168-semantic-probe` | `52b1f3419c13489e3c260a601ea5740d3ab1bc89` | `1/12` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | feat(python): add callback-free semantic probe |
| `agent/cfr168-shared-adoption-freeze` | `9720b5efccca3e3f0b9bc3f116792df554f56bbe` | `1/34` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Freeze shared upload adoption boundary |
| `agent/cfr168-stable-identity-gate-owner` | `b6262f7a57e87db8f0a3069f2567856c29a6f951` | `1/15` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(python): preserve gate descriptor ownership |
| `agent/cfr168-typed-copy-plan` | `e119a7911ce1366449450aa4702759acbd26e9b1` | `1/39` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Account typed copy hashing once |
| `agent/cfr168-typed-reducer-predecessor` | `7c05e3a8a1da3eb822d638e54a1aaefbdefe7e3a` | `1/60` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | Linearize final response ownership |
| `cfr168-doctor-object-diagnostics` | `902af8988afedba74fef75963aed925637626598` | `1/34` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(doctor): preserve cleanup retry owner |
| `agent/cfr169-ref-registry` | `25e5e3639b9854aac1f241253ae8e33f105b16e1` | `1/6` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | refactor(store): own typed object reference traversal |
| `agent/cfr170-migrate-actions` | `f83b53ee1b226ae0d8da0b623e2706f3e2cb6aa1` | `1/6` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(migrate): derive next actions from postcondition |
| `agent/cfr171-claim-descriptor` | `a77e9ef992a55a916e0a47acadee22babe83bc7b` | `1/7` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix: prevent claim descriptor generation downgrade |
| `agent/cfr172-readonly-experiment` | `91954879749114f8aefce267206e95aaf59cec9e` | `1/8` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` | fix(cfr-172): discover canonical plain roots |

## Appendix B — Exhaustive Stable Patch Ledger

Every one of the 374 candidate commits appears exactly once in the following stable-patch groups. The group row lists all full commit SHAs sharing that first-parent patch identity. Representative paths are bounded for readability; the numeric path count is exact.

### B.1 Needed

| Stable patch ID | Commits | Subject | Paths | Representative paths |
|---|---|---|---:|---|
| `e92dfce0c6f9165c9c8f52a1e0d9228ea9ba66a0` | `25e5e3639b9854aac1f241253ae8e33f105b16e1` | refactor(store): own typed object reference traversal | 7 | `crates/codefire-cli/src/remote.rs`, `crates/codefire-store/src/lib.rs`, `docs/development/code-review-issues/cfr-169.md`, `docs/development/module-boundaries.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `c86006dad23ff930b669d29db13ae84004f19a60` | `fabfea6c4c4a58c758a9a37ff1025ad49036a408` | experiment(cfr-172): prepare read-only adoption harness | 11 | `docs/development/README.md`, `docs/development/code-review-issues/cfr-172.md`, `docs/development/current-cycle-burn-down-plan.md`, `docs/development/experiments/cfr-172/README.md`, `docs/development/experiments/cfr-172/consumer-observations.example.json`, `docs/development/experiments/cfr-172/experiment.v1.json`, … |
| `aaae7f1b967a4570091e1318a31a41b7793434c4` | `ef17eb16c26e29facea51b852d4f95d9a2f4304c` | fix(cfr-172): bind hashes to canonical roots | 6 | `docs/development/code-review-issues/cfr-172.md`, `docs/development/experiments/cfr-172/README.md`, `docs/development/experiments/cfr-172/experiment.v1.json`, `docs/development/experiments/cfr-172/readonly_adoption.py`, `docs/development/experiments/cfr-172/test_readonly_adoption.py`, `docs/development/history.md` |
| `b67af6d6a23ca7fcd282c200fad7322a70bbd23e` | `91954879749114f8aefce267206e95aaf59cec9e` | fix(cfr-172): discover canonical plain roots | 6 | `docs/development/code-review-issues/cfr-172.md`, `docs/development/experiments/cfr-172/README.md`, `docs/development/experiments/cfr-172/experiment.v1.json`, `docs/development/experiments/cfr-172/readonly_adoption.py`, `docs/development/experiments/cfr-172/test_readonly_adoption.py`, `docs/development/history.md` |
| `c86656a339586b538f94fb7c4851e4d5d28792ea` | `73025a22c47a6be88ef70cf9563e89ea0d9751a0` | feat: bind verification claim descriptors | 13 | `crates/codefire-cli/src/automation.rs`, `crates/codefire-cli/src/exit_code.rs`, `crates/codefire-cli/src/main.rs`, `crates/codefire-cli/src/tests.rs`, `crates/codefire-cli/src/tests/subject_trust.rs`, `crates/codefire-core/src/claim_descriptor.rs`, … |
| `6de8de800656bb7ed5e5081aad127994c71a49b3` | `a77e9ef992a55a916e0a47acadee22babe83bc7b` | fix: prevent claim descriptor generation downgrade | 25 | `codefire`, `crates/codefire-cli/src/compatibility.rs`, `crates/codefire-cli/src/main.rs`, `crates/codefire-cli/src/tests/conformance.rs`, `crates/codefire-cli/src/tests/subject_trust.rs`, `crates/codefire-core/src/claim_descriptor.rs`, … |

### B.2 Alternative Design

| Stable patch ID | Commits | Subject | Paths | Representative paths |
|---|---|---|---:|---|
| `b39453d714da8f085ec2f77dd48f8777d412f5a5` | `be4484caa0416ea852c43148dd21f5e2a39c1081`, `3d4d514ba5b23b7d5f82ebf7ebe93da32955c4fe` | Freeze gate-relative preimage proof contract | 2 | `docs/development/ai/task-packets/cfr-168-gate-relative-preimage-proof.md`, `tests/test_python_existing_preimage_proof_freeze.py` |
| `a63e8959d0de3696f6e68212c29146329466c26a` | `429173f16ff922270899fcd8a1e1c2c61701903b`, `007ad1cf5ba15ed377005ed3f261552e956717a8` | test: freeze preimage accounting contract | 3 | `docs/development/ai/task-packets/cfr-168-existing-preimage-accounting.md`, `docs/development/ai/task-packets/json/cfr-168-existing-preimage-accounting.json`, `tests/test_python_existing_preimage_accounting.py` |
| `5cf095eb08afb7b6bd91950bcb93ca54f6f1d83b` | `b284da79292fa810665d879b799a5e8067c977b9` | Implement gate-relative publication preimage owner | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `6bd82c21af4da473557095bee188e1da18a40e4e` | `a8ec6d6d8f0547b4d9abe6fa9da7a22915325b8d`, `853a556e9f5010c6983e8972a7d89d3c9608d3e2` | test: freeze sealed preimage predecessor | 4 | `docs/development/ai/task-packets/cfr-168-existing-preimage-baseline.md`, `docs/development/ai/task-packets/json/cfr-168-existing-preimage-baseline.json`, `tests/test_python_existing_preimage_baseline_freeze.py`, `tests/test_python_response_baseline.py` |
| `318d824d82a6da339fdebe140aaa1362f6756d79` | `297aa57132db4259925f2fbae5fb4358b0ed2f3c`, `79ea7e2f03ae9aaa5c74a068fddc792a633af2c3` | Freeze preimage terminal probe contract | 2 | `docs/development/ai/task-packets/cfr-168-preimage-terminal-freeze.md`, `tests/test_python_existing_preimage_terminal_freeze.py` |
| `2d12b8f9b5a4fb921f3e37e564482202b1ee0aee` | `32a2cf734869abdd8466dd09598816e0348294d5` | Integrate publication preimage terminal proof | 11 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-preimage-baseline.md`, `docs/development/ai/task-packets/cfr-168-preimage-terminal-freeze.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `ad05f61d63adf9f9f8cc1ef9aa8cc7c83db857c1` | `9720b5efccca3e3f0b9bc3f116792df554f56bbe`, `874ed59ab1b8b9b589856ec260f5047549d89651` | Freeze shared upload adoption boundary | 2 | `docs/development/ai/task-packets/cfr-168-existing-shared-adoption.md`, `tests/test_python_existing_shared_adoption_freeze.py` |
| `55cc7c8f2771848ea45b0d1f58fb6f46b570afa9` | `51894c07876e1947345bce9c76c22c3ac23952a0`, `a98a2ed70c5ae6b9ede2524080f8984b825678c8` | Freeze typed existing upload reducer | 2 | `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `tests/test_python_existing_reducer_freeze.py` |
| `87b8a8b4a79e851e6f410a918ad906a13f22b4d2` | `14b5f7e46551ba0780c7a2d4a2ba7634cc32c857` | Integrate existing upload shared adoption | 10 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-shared-adoption.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, … |
| `1014c0622a5e56f5fe9fe329647b092f1d99aae6` | `05e4e694fa8b7886bffe8d5293e23ee7930bf05c` | Harden shared upload adoption retries | 5 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-shared-adoption.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `tests/test_python_existing_shared_adoption_freeze.py` |
| `6f16288b29a07ec3b4fa84c8c6c8ab8a352cdfc5` | `3016656cbe043c9a799d41b39b397a6a2983d9b1`, `abcb77e42eebbe91b1a5ccb8438f11b7a043f564` | test: freeze existing upload recovery proof | 2 | `docs/development/ai/task-packets/cfr-168-existing-recovery-proof.md`, `tests/test_python_existing_recovery_proof_freeze.py` |
| `8eec6f7acd0aa3baa459487bd4230d237aeb65de` | `8fff6719aa37eff9041255d86b97c877d5992e86`, `3411263703928cc4e9c99a544c7bf688b99c4536` | Freeze existing upload response bridge | 2 | `docs/development/ai/task-packets/cfr-168-existing-response-bridge.md`, `tests/test_python_existing_response_bridge_freeze.py` |
| `62c050f2c858873288baeffd81347ed30cda2905` | `d7270dd7d33879e98d8c7ced5c6dec5d1f15eced`, `d6fe855d0f2d7f0a4c4a7ec30dcc537e1161222f` | Add preacquired response body freezer | 8 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-response-bridge.md`, `docs/development/change-playbook.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `0e17b1eb8ca8e3a862cd6491d52616f1acd61b86` | `c40b81c27d5d5d3072fa4ca2eca5a319ea270e14` | Implement existing upload recovery proof predecessor | 8 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-recovery-proof.md`, `docs/development/change-playbook.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `91f3b54b992bc77191ee9a9e49cd8da157a4a869` | `727bb82a1f145c35d616dc19e458aa4af3e8f5ce`, `d460dd43eb5ffa5fad679a835c780447967d49d5` | Fix recovery proof capture races | 2 | `codefire`, `tests/test_python_existing_recovery_proof.py` |
| `df9519265329d60787b747ba98d028c9662d4d6a` | `926cf035937514eb7782ea54d1a6128801df7712` | Bound recovery scan and descriptor cleanup | 6 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-recovery-proof.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, `tests/test_python_existing_recovery_proof.py` |
| `18f84427804d36a2209f5a61ebfc8dfcdebed307` | `35f136757c0757cb38d496e2bac39cfae1433c0a`, `f0bcb899af13c4ca0862ee98215a265344e3342d` | Account terminal recovery scan overflow | 5 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-recovery-proof.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, `tests/test_python_existing_recovery_proof.py` |
| `d906c733160f9413cf3e7abe9e3111fd98343ec9` | `89932c162984366edd48a11c7ea7fbef10b4e294`, `c9510473941c5bce218be90722cfb3f876b0ffec` | Freeze typed existing upload copy plan | 10 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, … |
| `9ffc4e91f93c3dd19b9fafb03f8e1e86e9a4b4f7` | `e119a7911ce1366449450aa4702759acbd26e9b1`, `a8f85e0bea6d55bf13c8a87e73dd4e6856064b9b` | Account typed copy hashing once | 9 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, … |
| `9bc2f513d322fca87b1e616f96fb1b8b9e7bb44b` | `9c6a8ab6223d8b737ca77f4f11d71e020e147c08` | Revert "Add preacquired response body freezer" | 8 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-response-bridge.md`, `docs/development/change-playbook.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `6a0615673d556a12a192a57739767a3f4b7ff4eb` | `2f8003cdc83d563734e934bb18d305e1fbbb8a1c` | Add owner-first response body predecessor | 10 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-response-bridge.md`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, … |
| `7eb829fd2a524b0f93ac0305f52de833583cf070` | `4c3ef9b5ed1432aaa867ef4c5f8bad5fe969d1db` | Fix stable transition consume reconciliation | 5 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-response-bridge.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `tests/test_python_response_bridge_predecessor.py` |
| `42c1b015f17ab530e8c7ae859e0c44e0b1a934db` | `1c1b752d99723038de3f6670fdd8e9518915bd04` | Harden response predecessor retries | 4 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-response-bridge.md`, `docs/development/history.md`, `tests/test_python_response_bridge_predecessor.py` |
| `014c002e342c75910bd15328fbfc097091a9c47e` | `22482c120b531e7664f9ea3f18d21c2f5b6951f2` | Implement existing upload recovery proof predecessor | 8 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-recovery-proof.md`, `docs/development/change-playbook.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `3c0d0d46e5e2f080cd9639cd4f5c765e2c797a33` | `ddf530ec6404672d03a04e4734f472834231ff6a` | Bound recovery scan and descriptor cleanup | 6 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-recovery-proof.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, `tests/test_python_existing_recovery_proof.py` |
| `68cf69a36cd3f47a8fae893095ca9d22c2c2a3a2` | `ce4088cacc6a758f39a81e73e518d83093c98c75` | Add owner-first response body predecessor | 10 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-response-bridge.md`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, … |
| `a1f6cbd061e651c0da1755281a01b7c9ee6ab078` | `d78e0ae5079416540937b4a5e67b03b060cdbedc` | Fix stable transition consume reconciliation | 5 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-response-bridge.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `tests/test_python_response_bridge_predecessor.py` |
| `7c368e971ee4993222a9f6a93f11b679b8c58c39` | `b40859b1184b17b26b1e5c4898f5afa6383c98a8` | Harden response predecessor retries | 4 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-response-bridge.md`, `docs/development/history.md`, `tests/test_python_response_bridge_predecessor.py` |
| `6f43a2084416888367a827915746d15627b3b356` | `3a6fca39cd90b3b6309a953fe060982d30c4b6c9`, `f5738a2c4c3f915e18245adced8580f867a814d4` | Freeze recovery effect evidence gaps | 8 | `docs/development/ai/task-packets/cfr-168-existing-recovery-proof.md`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `13f643ada9fcd2706c2257d6f0d42476db92a749` | `d6afe7d8f4560f4b37118510fe112747baa05203` | Revert "Freeze recovery effect evidence gaps" | 8 | `docs/development/ai/task-packets/cfr-168-existing-recovery-proof.md`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `c572fa60cf4cabef9ffe7b20088413638c640a53` | `a816bc191d077455b8c84e5a8bcb56cd473704cb`, `87f55b7eeebbf9a90d8fa14096a3ae84dd1f50b1` | Correct recovery composition Freeze | 8 | `docs/development/ai/task-packets/cfr-168-existing-recovery-proof.md`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `13a895ace45761f3e8f151ca7f10a8c183ee0ed5` | `e3ff2aa678f7fd43e9680027c2f3a03452cff9b3`, `51b2616afa14c41121ab7c719a88df58c286634b` | Implement recovery-first publication classifier | 11 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-recovery-proof.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `725680ba8d64671eef09e62aaec2b3fa07b088b2` | `dd4683705c8e1baa404904b22f6d483af9863ce6`, `d6fd57edbafa1894b5eb1edd3a1900045c496933` | fix recovery classifier cleanup states | 10 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-recovery-proof.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `f27ab348d1c5be3379563d00ac5406972110397c` | `a79432490a23c46efd2087dd4fa01cbe3187c4c4`, `1b3334386c2a98b57bd20a8dfc7a55713add934e` | harden recovery terminal state snapshot | 7 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-recovery-proof.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-data-model-catalog.md`, … |
| `d370779dd4f349a6cf2bd176be1c426271fc4c09` | `d8a3525c66a7746f51762cc9ca8b1b14b5486e0a` | Add pure typed reducer predecessor | 7 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_effect_input.py`, … |
| `38b990a3d4f3623ad7ed2d17897358782517af95` | `e109b5b7fb4bc7592329d904d2729e21c1a4d498` | Fix typed reducer retry fences | 6 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `f437553c0580d6e11d032760e8059798eabf4975` | `76cec0659575fbb432d301c8991c50159d8933c3` | Fence reducer adoption before response selection | 6 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `235562783b6424180ab0ec6716c3c9448c8cee6d` | `dd62602f5fb21686c2b6da98b2f1525fc3d0e332` | Pin original reducer adoption anchors | 6 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `f947d5abb52b79c8c8da46b7308ac0e2b06103b2` | `ded812ca05a7533d1491755e22ec3e61e8ab7c26` | Externalize reducer authority anchors | 6 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `1860b47a04f4eb2cba82fcb23a5cc28d8d687cee` | `6da0a8af4eb6af906ad920f2c082ba31ae4c20b5` | Hide reducer authority registry capability | 6 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `16135e5f12f20e7e7eb9a9747e15e9345eb2817b` | `250e26d0752f3746ffe410f3c19606bf485836a2` | Bound reducer authority registry churn | 6 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `12cf2a70b0c462d2bec3b57fda40b02dc4dd1e40` | `35b35a30e2a20a7e40b762ba0364515f10317d49` | Linearize reducer state transitions | 6 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `b733a0663204d02ed24a7f218d9c8a2f7a393a46` | `4403c4c11c33c8767e7e90285c36517dec206b0e` | Reconcile claimed reducer cleanup retries | 6 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `8c267bf5be049c8f9c248e3f94a02d489883dc7a` | `7c05e3a8a1da3eb822d638e54a1aaefbdefe7e3a` | Linearize final response ownership | 9 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_publication_owner_freeze.py`, … |
| `2c55cce5141de160865d21c038453b52be3c940a` | `b4dea41a017194ae4389dac51b55fa8172c7c7fb` | Add pure typed reducer predecessor | 7 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_effect_input.py`, … |
| `2052772f0570841c6ba045a8ef1ac12bf759c30f` | `0542b63951e4c632d92b85c9a4eb6e58e2ae7cba` | Fix typed reducer retry fences | 6 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `c5b7305c953a9b6aba828efa6db6b760fe19062b` | `d8dcb1a197631f25cc26b055163ab1896d6b000f` | Fence reducer adoption before response selection | 6 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `fac75fe1738e3a8f648213161d34b685890bad3e` | `a702b943aa7f423e29f1c87ef583eca708a13f9e` | Pin original reducer adoption anchors | 5 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `181aa4ca2650c165a32b2044f7f7161cfe243302` | `bbf07326fb58da8507cdf3518a1815b5404c7390` | Externalize reducer authority anchors | 5 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `2195c7a5a024550d7d5e510bd98817be85c4aeb5` | `bc8343a6ff11a7fb8b1fbd6d9b340095f3b9a2c3` | Hide reducer authority registry capability | 5 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `41069cdea30ecb240b0119d5b6d9adde05041eb3` | `599968f95b7968152df0b49aa582d6ee5bfc2b61` | Bound reducer authority registry churn | 5 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `0cd403d5ac007a2217feb57ca11bb6cafcdf8c18` | `8e7bb255ae4b1fb76e68a8659c34371ae197a058` | Linearize reducer state transitions | 5 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `46a741ce5ae10a2cea75b34ac17f6eb03b059548` | `7456486f5092ad22dc06e92081c85303af0c21ef` | Reconcile claimed reducer cleanup retries | 5 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `e083ae282513fa44f445ac6e358dc1f69a48b63e` | `82f0fa50bff0999f322fd5020cbeade194f500a9` | Linearize final response ownership | 8 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_publication_owner_freeze.py`, `tests/test_python_existing_response_composition.py`, … |
| `9fe8c2777148725781e6c3bce42bd6bb7a71a400` | `068405fe558d8543acec25c46497cb189cfcab91` | Integrate reducer with recovery classifier | 8 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_effect_input.py`, … |
| `1824cc67a0b74dcc8f7433bdc66e2596b08698a7` | `e87af3f49dcc1ab0906407de3610b1e18fffc4c3` | Pin reducer input authority externally | 7 | `codefire`, `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_reducer_recovery_integration.py`, … |
| `e7c72078060c379e90318cc3b2970198f38fda0f` | `42883207ba368536552119fa223d171d2c021ed9` | Freeze existing upload execution evidence | 5 | `codefire`, `tests/test_python_existing_recovery_authoritative_classifier.py`, `tests/test_python_existing_recovery_proof.py`, `tests/test_python_existing_reducer_recovery_integration.py`, `tests/test_python_existing_typed_reducer_predecessor.py` |
| `b5c31d0eac4229cfefbe4337d8dd1dde38480007` | `05fd2a7ee918f57566b103e2fe64b08eaee31822` | Classify deterministic fresh staging state | 3 | `codefire`, `tests/test_python_existing_recovery_authoritative_classifier.py`, `tests/test_python_existing_reducer_recovery_integration.py` |
| `ae5be096f093f7a2dd54ae74e997a0ef8bb2ea45` | `5a6f5c573c2061dc2395f38c312ce3d234eb8ee6` | Harden frozen execution evidence | 3 | `codefire`, `tests/test_python_existing_recovery_authoritative_classifier.py`, `tests/test_python_existing_reducer_recovery_integration.py` |
| `a5581df129e377ad3c7a496e24efe81a31cd2d1e` | `785d000badf2f7b7d8f86de7a08cd1c4bd1a5f00` | Tighten fresh recovery names | 2 | `codefire`, `tests/test_python_existing_recovery_authoritative_classifier.py` |
| `c91abfb33d1ebe3a80bb37646cf0297da42dafc7` | `7668969c9eaa2e00a52472b9469cd37e148aa4b1` | Add execution evidence accounting counterexample | 1 | `tests/test_python_existing_execution_evidence_accounting.py` |
| `15cdd3adfe848d8df7790dbb9f843b262c0eadf2` | `388f3459dcd8808e6934675836e76b0dfed5613e` | Account execution evidence retained state | 2 | `codefire`, `tests/test_python_existing_execution_evidence_accounting.py` |
| `b5ef0931ed4194623565350e74a7934037b3ff54` | `e2da01a2d627fc65f654215506af9dd6fc9a76fd` | Account execution evidence byte work | 4 | `codefire`, `tests/test_python_existing_execution_evidence_accounting.py`, `tests/test_python_existing_recovery_proof.py`, `tests/test_python_existing_reducer_recovery_integration.py` |
| `5b57adca2dee19d1ff7484da8a68300127757006` | `758ee8b8ebe44d9160c34d3145f1538fafc722bb` | Correct execution evidence accounting ownership | 2 | `codefire`, `tests/test_python_existing_recovery_proof.py` |
| `3e0523d8c4b24d5a32a127e804fb9e982cf30d0a` | `8da98f567d1aa13c6d22e26a604d056832f27c6e` | Pin execution accounting schema | 1 | `tests/test_python_existing_recovery_authoritative_classifier.py` |
| `9cdb22066b68b3ec1bc1f2442d99cd2e41e9104e` | `9d68072ed38290538f8d53fe1c95412886622a2a` | Make evidence upper planning scalar only | 1 | `codefire` |
| `9e73378ffdd46877ff3dd9ba0707d5680cec1a72` | `d54bb0dd720e3391936900b78abfd4fb65e3cc8e` | Pin execution accounting measurement authority | 3 | `codefire`, `tests/test_python_existing_recovery_authoritative_classifier.py`, `tests/test_python_existing_recovery_proof.py` |
| `268793ea68b2cd8e3880f5678b74e1ed5cbbdffc` | `f17be87f933191d7f85885bb5a4db791f1e1ed17` | Extend accounting authority through reducer | 3 | `codefire`, `tests/test_python_existing_recovery_authoritative_classifier.py`, `tests/test_python_existing_reducer_recovery_integration.py` |
| `d04ae326d1767f5b37b919e4b575a3799ff5cfbc` | `741079c42b8a5d4b90f9c1fa47ce5ce0bc045f17` | Preserve reducer authority registry fixtures | 1 | `codefire` |
| `573db117ed48476fa5d031dff57026e1c6534bf0` | `7725e6cd7df2ba68fa75642814718d765ccc1140` | Pin execution planner scalar inputs | 3 | `codefire`, `tests/test_python_existing_recovery_authoritative_classifier.py`, `tests/test_python_existing_reducer_recovery_integration.py` |
| `eef9e9c34000a3cec9876c1724dabd91c9302abc` | `060285a1935b7fce021b5a0470e546476507434f` | Pin execution planner function shape | 4 | `codefire`, `tests/test_python_existing_recovery_authoritative_classifier.py`, `tests/test_python_existing_recovery_proof.py`, `tests/test_python_existing_reducer_recovery_integration.py` |

### B.3 Superseded

| Stable patch ID | Commits | Subject | Paths | Representative paths |
|---|---|---|---:|---|
| `027f05a61ddb98656e4b36218d0ec2831c3c760d` | `1ee171119885be89514b31bd4a3e2833625ed1f7` | chore: snapshot CFR-168 integration base | 260 | `Cargo.lock`, `README.md`, `adr/ADR-009-capability-relative-worktree-writes.md`, `adr/ADR-010-stable-kernel-backed-lock-files.md`, `adr/ADR-011-capability-bound-sealed-validation.md`, `adr/ADR-012-versioned-sealed-semantic-profile.md`, … |
| `ad15ea39b685ded73a8de2d1fce4d6bc9a5ecfed` | `7d01efc5d4c142e4129c72bd0e26a44a751903ca` | fix(remote): own accounted object-set digests | 1 | `crates/codefire-cli/src/remote.rs` |
| `b45674dbe40cd9ba6831e01c2a195f06c168be69` | `30f3063d00e5fa20a384a246d3f242a156a68ec4`, `d3db8979dd068c64808948578d485ca8b6a68a52`, `e49517ac0fb66ac59083df235c1ad953aefb321a` | fix(remote): preserve digest ownership through handoff | 2 | `crates/codefire-cli/src/http.rs`, `crates/codefire-cli/src/remote.rs` |
| `3485309ad55d8c3c90e323a12a3c5ce5f1b6436b` | `b7c79e623e337c50f1bd8d7dc26c750a0c88f2d0`, `be06655915b9008ea12e932613811683534c398a`, `552437da08feacd771af7787213543f6001f3f8a` | chore: snapshot CFR-168 Python HTTP base | 260 | `Cargo.lock`, `README.md`, `adr/ADR-009-capability-relative-worktree-writes.md`, `adr/ADR-010-stable-kernel-backed-lock-files.md`, `adr/ADR-011-capability-bound-sealed-validation.md`, `adr/ADR-012-versioned-sealed-semantic-profile.md`, … |
| `3fcd20008e5817e644422fc6288e1640d31b26cf` | `e89ac53ea1651e1a619bd7210bca27e416b3c780`, `d2db3e2dad30f9cdca1691ab812a9f8b252fbd32` | chore: checkpoint CFR-168 bootstrap integration | 4 | `crates/codefire-cli/src/http.rs`, `crates/codefire-cli/src/remote/locks.rs`, `crates/codefire-cli/src/remote/project.rs`, `docs/development/code-review-issues/cfr-168.md` |
| `753d2c0bf159bb8622f02db47ba8a367d343bc23` | `71e181126d03bdef169789d0cb9f8250144d19b1`, `a3f538907e6ae282fddbb4c10d8db575d1860f2d` | fix(remote): own accounted object-set digests | 1 | `crates/codefire-cli/src/remote.rs` |
| `bb47f610ee3ccacb4abad1fe3811c00167a00694` | `f83b53ee1b226ae0d8da0b623e2706f3e2cb6aa1` | fix(migrate): derive next actions from postcondition | 6 | `crates/codefire-cli/src/migration.rs`, `crates/codefire-cli/src/tests/migration_apply.rs`, `docs/automation_interface.md`, `docs/development/bug-issues/cfb-133.md`, `docs/development/code-review-issues/cfr-170.md`, `docs/development/command-source-trace.md` |
| `40a1b3ff5a9cbf85691e64644d1bbf97046cc3ad` | `43652d015a5e057633ad203270b22f727981fa1c`, `0d9d7f716cf1770219acb85560dc32d878fc7e21`, `64eecb47c8723b0d4cc75e9de3bf32e58ef6bfdb` | Bound Python HTTP transport work | 2 | `codefire`, `tests/test_codefire_cli.py` |
| `50e6ca1e6a3477ddce5b73a5022fcdf1cba7f1ed` | `4b63353115805af2fe7a260b3296f7a8cc32a402`, `d464d1ef467b3dab5f0da07ab76a34a4c34e388a`, `c544c2dac76f28c07b17e044b35ec0b010e7f9b9` | Tighten Python HTTP ownership preflight | 2 | `codefire`, `tests/test_codefire_cli.py` |
| `4e45c04f720797f6d246959011f5f9ad5dac859b` | `f5764f69026e516d91a8a40cd1b058f4a9f5cdf9`, `46749185cfc403b537308228767eeb13e762b016`, `b207c618d581297772b76ee815d38f6ba596c877` | Add bounded canonical JSON freezer | 2 | `codefire`, `tests/test_codefire_cli.py` |
| `048c407b536a7a56bae8435a9e97b699372f0312` | `345a509931783ee05e4dbd16d798096a4dce1ecb`, `d048175179fda16fa413cb6302bbcbdc577cc3d4`, `8105298be58ac873fd372e870ce7ebd845d7b4c7` | Prepare bounded HTTP responses before publish | 2 | `codefire`, `tests/test_codefire_cli.py` |
| `eaaffbd8c00ed315755e467a486253cc295599c7` | `a20ac0c6f9a5f9a3993bc506559cee34579b93ca`, `aa811b1e77e56cc8abd1ae60996406a2ee3889cf`, `1f939a3d2d56985e0b8e2a35462b7551034979db` | Clean prepared response owners on faults | 2 | `codefire`, `tests/test_codefire_cli.py` |
| `fc6672df341ffd306396fb6cb8ce2eb9f5c76c5b` | `7f44f406cf6bc42e2f0e36fc102d83dd287aedc8`, `2db7868e1790efd55bffb5e8fbc46bad9ab60097`, `fdbb7afce8dfca4a6cdce443ee3aeb387b84d39e` | Own prepared HTTP responses across cleanup | 2 | `codefire`, `tests/test_codefire_cli.py` |
| `ad9eee4975daa0c1251cced7704e61934e8dd5f3` | `704f91863b23a8d4998ba2a70e9789827b9a9983`, `cd4b6f4549eed5d511092272efd59ce62d895f25`, `a381ca789f7fbfc3318d468e826b365d1510ad76` | Harden bounded freezer allocation cleanup | 2 | `codefire`, `tests/test_codefire_cli.py` |
| `5ac77849cc06729f8e4cfe145823a01fb9cfd393` | `06dced7701135ed9f754d31f9496d28c591ed45f`, `9fecb27edce8598027b82ed59f35890202113515`, `880bad1e62e8b8e419dfe21543b1106d2dde69aa` | Close prepared owners before propagating hooks | 2 | `codefire`, `tests/test_codefire_cli.py` |
| `c305f9c0595bc1c37f7f2ca9a3f0a65c267ce74c` | `73e2003339dc82326d195d8bcb2bf72a05fbf74f`, `3e7b92a2bba52a1e19826a8790bf27d443260d7a`, `3a54fdd6360c906810c3a6f2edd1390d328aec1b` | Clean HTTP owners on base exceptions | 2 | `codefire`, `tests/test_codefire_cli.py` |
| `155a9cceff01018a30b5a4340971fc6925f7ac82` | `bb55cafdbfebf95dad58a2f1c976c065655eceb1` | fix(cfr-168): bound Python HTTP admission | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `d2f06c12eb0613747f66f049afae68777020a943` | `d053b7faea7b16d79b02d6f5f2fa3867374dc824` | fix(cfr-168): harden Python admission registry | 8 | `adr/ADR-020-decision-equivalent-work-budget.md`, `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `c219e41d409e46416266e1314475784e832a9376` | `3681cb80e0ee326260a353307cec1bac810e3869` | fix(cfr-168): roll back failed HTTP registration | 8 | `adr/ADR-020-decision-equivalent-work-budget.md`, `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `55cb530428cc5ce890e8072205f338906c0aff98` | `645d30dd14bb650d2839c55fefba7e9a4442e0c5` | Add diagnostic scan validation handoff | 1 | `crates/codefire-store/src/lib.rs` |
| `6432d065622833c052a9d221aaa3e4d3b7229927` | `36d1b5c99d95cfedb7d0ceae3fd8919572b5901b` | Guard diagnostic scan visitor ownership | 1 | `crates/codefire-store/src/lib.rs` |
| `204be1d48a39452b04bb22926a493fe59ff13770` | `460309a796931b2aa923a068b9687d7c33120cac` | Share Doctor object validation session | 1 | `crates/codefire-cli/src/doctor/checks.rs` |
| `0473f1f9efb1aa7b5bc37e225ac8bdf22891cb1f` | `5f8616a3644666c236674252f1ae134b4e3e4069` | Seed Doctor validation from diagnostic scans | 1 | `crates/codefire-store/src/lib.rs` |
| `453ac265aa03a966374beb8985d7afedc351bcf8` | `54d2f2aa0da362c3ffac8f783f268748d455d137` | Propagate fatal Doctor validation failures | 2 | `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-cli/src/tests.rs` |
| `9726c0d3e89ac954c8108f7d5f290d64e374cbc5` | `3cdfd3556487a149a59d7844cdb17dc69aed00ee` | Account Doctor report rendering work | 9 | `crates/codefire-cli/src/doctor.rs`, `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-cli/src/doctor/report.rs`, `crates/codefire-cli/src/doctor/report/accounting.rs`, `crates/codefire-cli/src/main.rs`, `docs/development/code-review-issues/cfr-168.md`, … |
| `a5d2d6a92e59299397c89bd09bf4c0c37b344e5e` | `d34bdb453169fd12646217992ed5e4dccda73096` | Close Doctor report owner escape paths | 6 | `crates/codefire-cli/src/doctor.rs`, `crates/codefire-cli/src/doctor/report.rs`, `crates/codefire-cli/src/doctor/report/accounting.rs`, `crates/codefire-cli/src/main.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/source-data-model-catalog.md` |
| `c27aa92c6fa495ce223d347067f92a9cca99c6d8` | `151786bd1c4a7f29681feae68795691ce9b0bff5` | Account Doctor layout issue collection | 8 | `crates/codefire-cli/src/doctor.rs`, `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-cli/src/doctor/report.rs`, `crates/codefire-cli/src/doctor/report/accounting.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, … |
| `45b32569dcf2b5a79615bbe4b6eefd4e8834ffef` | `ecd9983c2355975315478047da86736c986fe478` | Bind Doctor issue ownership to its ledger | 8 | `crates/codefire-cli/src/doctor.rs`, `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-cli/src/doctor/report.rs`, `crates/codefire-cli/src/doctor/report/accounting.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, … |
| `1a7e0119df1d9b241218ab3765e50a8f8c90bfb2` | `22dece7a24afa62702483a37b31ade61ab7cc50e` | Account Doctor close transaction markers | 7 | `crates/codefire-cli/src/close_transaction.rs`, `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-cli/src/secure_worktree.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/source-architecture.md`, … |
| `858e69c11757b012c22a76e861d5ac56a5709304` | `97ad562a1870d8682360290d0513a5b972c8d6ea` | Bind Doctor marker reads to parent authority | 8 | `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-cli/src/secure_worktree.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/module-boundaries.md`, … |
| `15291c8f1eeffd3601fd66b79f7ed3ecc3672469` | `6e839381e891fb333103ea6ececdf34317751362` | Account Doctor open transaction markers | 2 | `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-cli/src/open_transaction.rs` |
| `12c8645bfac93121a07a21e0eb18e0642c33c419` | `5b583ccf3d2d382ef9688621403df4d71d864ee3` | Preserve Doctor open marker conflict precedence | 7 | `crates/codefire-cli/src/doctor/checks.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/module-boundaries.md`, `docs/development/source-architecture.md`, … |
| `78a2218e80de69149625483838eb59fd387c2d16` | `7b6d71baf9f84654b9a7c4bd66e2a68fe7e3f3a5` | Order Doctor open marker candidate reads | 7 | `crates/codefire-cli/src/doctor/checks.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/module-boundaries.md`, `docs/development/source-architecture.md`, … |
| `133e8bf55f19f4d73702bb944d13d958ffc85213` | `c0f291d2be87df58b927aca4fe285e70c0eefd96` | Account Doctor transaction conflicts | 7 | `crates/codefire-cli/src/doctor/checks.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/module-boundaries.md`, `docs/development/source-architecture.md`, … |
| `19253e8444ed56c1cc26584c8465b8b013454785` | `51a0d69306186589dcd02a1f422f92d874bc63b5`, `1a934c38387d592d5779d02711acf6b5b73a55a6` | Account Python HTTP bundle graph work | 8 | `codefire`, `docs/10_merge_remote_server.md`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/source-architecture.md`, … |
| `1a5bafdd6435c1bec3d0aca8ae572c0ef626f7ab` | `99b5f654d7f481f1f2bdc6fca3f1294c477fb6e0`, `09e4d060bc2fe1e4a6cdae98e2194dfa702fd53d` | Preserve terminal HTTP bundle diagnostics | 7 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `672f8e9e5f33106fd140196ff2ee32ef855355c2` | `52e3ec673f4389cd4d8ffdb7edb69b4cb72a9b29`, `a28508f33d8b3d4e108a7b62331b74f124e940de` | Bound Python bundle requests by deadline | 7 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `ce4a400aa62d1dadce5bc7b32a06c01fcfb4dbed` | `715907e099f188e21156801f6ccd886153ba8285` | Poll Python bundle hash work by deadline | 9 | `codefire`, `docs/10_merge_remote_server.md`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, … |
| `07ecbd2e34e072d2293903634867e3c8e8d41c03` | `ffb902201bd55f9984ee3b51fb572532b5b446a3`, `bc88de2a241d9a83088c9f06a2179a84f9cd6c1c` | Preflight Python upload mutation responses | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `59d72783847d910de12bb252ffb7611d263be392` | `6ffee13f9dc9cfc55dfbed874722a84194b9017e` | feat(http): bootstrap missing upload project atomically | 4 | `crates/codefire-cli/src/http.rs`, `crates/codefire-cli/src/remote.rs`, `crates/codefire-cli/src/remote/locks.rs`, `crates/codefire-cli/src/remote/project.rs` |
| `ce3456072245ebc947c25b05eb024cc85516a46d` | `cb1ac0311567af5ddce12522125807f0eb2184e0` | fix(http): close bootstrap failure ownership | 2 | `crates/codefire-cli/src/http.rs`, `crates/codefire-cli/src/remote/project.rs` |
| `08a28a6ad744e63c9a4cf91027dd0fb19626fb1b` | `84ed73a36c8aba40375c8c1b6089dd93d4cab0db` | refactor(http): plan bounded bootstrap footprint | 3 | `crates/codefire-cli/src/http.rs`, `crates/codefire-cli/src/remote.rs`, `crates/codefire-cli/src/remote/publication.rs` |
| `2f690ab92ef7e07e904bfa6511e6f4526f0bf77b` | `1a7ec2bd5933734caa5d76a35b1d07a8de8e60af` | test(http): enforce bootstrap footprint boundaries | 3 | `crates/codefire-cli/src/http.rs`, `crates/codefire-cli/src/remote.rs`, `crates/codefire-cli/src/remote/project.rs` |
| `9188d21e316249a1184b00ca7e294d4876f1a6fd` | `146a672748477e9ee64a6174db4a1c7724325234` | fix(http): make bootstrap handoff post-rename infallible | 4 | `crates/codefire-cli/src/http.rs`, `crates/codefire-cli/src/remote.rs`, `crates/codefire-cli/src/remote/locks.rs`, `crates/codefire-cli/src/remote/project.rs` |
| `7715ff117d48856ce51d9f727a59d63c3aadefd3` | `74a3eac077a9f500db42ae2d248ba21ba526990a` | fix(http): preflight missing upload footprint | 9 | `adr/ADR-020-decision-equivalent-work-budget.md`, `crates/codefire-cli/src/http.rs`, `crates/codefire-cli/src/remote/locks.rs`, `crates/codefire-cli/src/remote/project.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, … |
| `0f048caabe19d76618267169d8129d33fabe9e27` | `aeacd096f06a56572e29900161efe290669eca6f` | refactor(http): bind parsed request body ownership | 8 | `adr/ADR-020-decision-equivalent-work-budget.md`, `crates/codefire-cli/src/http.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/module-boundaries.md`, … |
| `d12160e9eed179ccb42b945262486283104333ce` | `596c2a5d8e392f80678194a7035c4dc3515c394e` | fix(http): transfer mutation body ownership into Store | 10 | `crates/codefire-cli/src/http.rs`, `crates/codefire-cli/src/remote/locks.rs`, `crates/codefire-cli/src/remote/project.rs`, `crates/codefire-store/src/lib.rs`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, … |
| `3084b85ff1028ca9e5f9db6c7ff45dde86fd3e90` | `a688ac20ccafa21969d16b6251505075501d69ab` | fix(http): account derived mutation owners | 11 | `adr/ADR-020-decision-equivalent-work-budget.md`, `crates/codefire-cli/src/http.rs`, `crates/codefire-cli/src/remote.rs`, `crates/codefire-cli/src/remote/locks.rs`, `crates/codefire-cli/src/remote/project.rs`, `docs/development/change-playbook.md`, … |
| `8437e9d8eddda91c3af86bf181050d1f8d1f1fd9` | `1f579de236ad08f42ff0f741b807da77572a2339` | fix(http): account mutation routing owners | 8 | `adr/ADR-020-decision-equivalent-work-budget.md`, `crates/codefire-cli/src/http.rs`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, … |
| `9b9c188b50ada12ef1d6c53140a269253868605f` | `7bfda7ee0532c33198c328f6b7bf54d7363f40b3` | fix(store): move overlay documents into upload batch | 9 | `adr/ADR-020-decision-equivalent-work-budget.md`, `crates/codefire-cli/src/http.rs`, `crates/codefire-store/src/lib.rs`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, … |
| `29c7d50d154bd426f375682e99f6c7b3dced85ab` | `050e4a68c6d125669d607c3b2d134aa58837ec8d` | fix(store): order overlay batch validation | 1 | `crates/codefire-store/src/lib.rs` |
| `7d0d38fc52ad14338a025d57fca13409302df781` | `8e006b653882607c786552241a4ee3da7b8af364` | fix(http): move request objects into overlay validation | 12 | `adr/ADR-020-decision-equivalent-work-budget.md`, `crates/codefire-cli/src/http.rs`, `crates/codefire-cli/src/remote.rs`, `crates/codefire-cli/src/remote/publication.rs`, `crates/codefire-cli/src/signatures.rs`, `crates/codefire-store/src/lib.rs`, … |
| `9c1d1f1220893ef08299432efc3b0ad69fff6373` | `d55207a92e4e8750184e1dc06a374c7fb992c155` | fix(store): bound overlay duplicate index scratch | 5 | `crates/codefire-store/src/lib.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `0bb452a5bf9e6efb71961ea7faedec6ddfd04ce7` | `8690b3fcd0c45c698d68936c7fe488e022ea1240` | fix(store): move object extensions into overlay | 5 | `crates/codefire-store/src/lib.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `c6ac98d4752da00910a229943ede98a5da13fcef` | `f7f5d7ce8344b2c83392b356ce6356fdbf2c9253` | Account Doctor branch head validation | 9 | `crates/codefire-cli/src/doctor.rs`, `crates/codefire-cli/src/doctor/branch_heads.rs`, `crates/codefire-cli/src/doctor/checks.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, … |
| `f1d8e01b3e3503d8ab5ee80037f10feafdae69e1` | `63dea5ec72b97a2fbfb3901a5972ae41b20f5b14` | Bound Doctor branch record reads | 5 | `crates/codefire-cli/src/doctor/branch_heads.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `6a89a4706a1cfef1ce6b8c14d6951e9260fe23f8` | `28016c612481b3fac714e462443171054c9e0b5f` | Account Doctor object scan diagnostics | 8 | `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-store/src/lib.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/module-boundaries.md`, … |
| `f91b36b0a94919a70e3994fd3627368bd9379856` | `53d3182abd912dcbbf6384f09ccd35e7bbeaf8ad` | Seal Doctor diagnostic publication ownership | 7 | `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-store/src/lib.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `40aa763dd2655c80b74f7cd1b17554be0566a7df` | `e433508eb6d2899c46732ffe0e1b0b5244d7070e` | Account Doctor opened entry diagnostics | 4 | `crates/codefire-cli/src/doctor/checks.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md` |
| `2102eaf357937f4f5fa4b46cd54f58451189033b` | `c0828dc21781afa71969247c8cb020ebff69c44f` | Bind Doctor opened registry candidates | 8 | `crates/codefire-cli/src/doctor.rs`, `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-cli/src/doctor/opened_registry.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/module-boundaries.md`, … |
| `06ac0b94914ca9bc9085d1217c3ef189e40484c8` | `7321baf727fdb557d0cdb96f2d96673ee6333b80` | Account Doctor absolute open scratch | 6 | `crates/codefire-cli/src/doctor/opened_registry.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/module-boundaries.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `1d8c6b5bdc57ddde2da2ea30e74761d32536d33e` | `6742e577374304c7d350beccc524519a4b897591` | Bind Doctor opened registry payloads | 7 | `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-cli/src/doctor/opened_registry.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/module-boundaries.md`, `docs/development/source-architecture.md`, … |
| `ce7fe676500537005c1a473c7d46517c8aa3a950` | `c5067a95dc431cba7fb3697706e30bc76f776460` | Preserve Doctor registry link authority | 6 | `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-cli/src/doctor/opened_registry.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `c5ab6425f0ee2343b732844da46a8f1a6034e881` | `3b3232c279ce44e194404a1ec0866f1cf7792114` | Recheck Doctor registry links after reads | 5 | `crates/codefire-cli/src/doctor/opened_registry.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `14116563d00154b6d329bfe6745a0cbe6a6f6c99` | `e0c892c800e2f40788a89c50bf3df5f1081f104b` | Account Doctor registry JSON values | 9 | `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-cli/src/doctor/opened_registry.rs`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, … |
| `ebc5d6b275920bcdd2816f56e5c698ea0e75ea91` | `cfbb7c006280472dc632e9349859a4534b8809ed` | Fix Doctor parse transition rollback | 6 | `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-cli/src/doctor/opened_registry.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `03eaf02254133229694fc42c446cac53069c4162` | `902af8988afedba74fef75963aed925637626598` | fix(doctor): preserve cleanup retry owner | 6 | `crates/codefire-cli/src/doctor/checks.rs`, `crates/codefire-cli/src/doctor/opened_registry.rs`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `5f80c9c57f3bf465cc2f47011b23347d35c851bd` | `f7706bb820f7d6283123e027ed097dd9f531b165` | Poll Python bundle hash work by deadline | 9 | `codefire`, `docs/10_merge_remote_server.md`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, … |
| `d52fb68d531e55c1288565b483680ecdce8d93c1` | `759ed9cfda0046b3b3bb21d0930d88fc3ff668a9` | fix(cfr-168): bound Python HTTP admission | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `46b272658ca586a411d170e16ead020c5d4ee725` | `b421f237818447d36a891efce5804dce41f2a834` | fix(cfr-168): harden Python admission registry | 8 | `adr/ADR-020-decision-equivalent-work-budget.md`, `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `c8a7b1a4a22af7309da11f6bf32fa39496b3c6a1` | `e210ddd2b0dbc1a736ddc1471b39fdb02721dbe6` | fix(cfr-168): roll back failed HTTP registration | 8 | `adr/ADR-020-decision-equivalent-work-budget.md`, `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `ddf69b5f2b2621bd6ae817e29f37c96e737cf86b` | `691c5cc728dfea28722257790202d80950b4be69` | Fix Python upload pre-effect validation | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `94a23eb57f01cb0af15ee027ce630cf61a3b5808` | `b6ab04dc91eb13ab0026ab6f4957c28b82dc7fb3` | Fix direct upload nonce publication ordering | 6 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_codefire_cli.py` |
| `5f404078850fcd5f0276765aa998e0f10742a355` | `ba4c37b04e814ff0d8f6d5de192b1b66e0cd4043` | Reject surrogate upload identities safely | 6 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_codefire_cli.py` |
| `d250760660dc50445b3ecd1d1ccb8c914284b6fd` | `b295bfc91da8b102ec1285712920d3b5cc55bd22` | Preflight missing upload writes before layout | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `80da6b99a2adf4c3199450a475dcc5921ba98240` | `6b8655ddf63627cc2ccb87db065db85d6ac2c90c` | Restore publication validation precedence | 4 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `tests/test_codefire_cli.py` |
| `f5fbbe22e2516d169703327d84860e35f61cbbd2` | `cecbb3b2a783559297cafcd3646d55890677baf7` | Preserve publication write count precedence | 4 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `tests/test_codefire_cli.py` |
| `c47fb7d093616692e4d8d63fbc8da62637fc909a` | `96d54e7fc7ab6193ed050baec95d3b7daa4fd93e` | Account for HTTP upload closure planning | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `442e045dc81fb69f9171db7a81fda98cbb3495e9` | `3347b81c92ecd98bc55757e439da8f81762b53ff` | Reserve upload closure list bases | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `165e1a862fc825d22a2ac926ea2192afc249cd63` | `15a13c3341f74bd2446e242a900219e711bfb27b` | fix(python): account upload closure planner | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `fff17fa4ac5dc9a0f9eed41ac148a27f1c6aae67` | `2eb422299cc69ac82ac8752f2b5b58856d9a06a6` | fix(python): bound upload planner collections | 6 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, `tests/test_codefire_cli.py` |
| `3b9bbb54ec312a95c04f0ed21535c807d6d44c79` | `5ab5461b14a3dfa94d23c77bf20ef2cc30d327e6` | fix(python): reserve every planner rehash | 6 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, `tests/test_codefire_cli.py` |
| `f1354f889d13c993c4b87352ffccd1e85f7cce9c` | `a30893a9eab7b7dc0f0119d47fcc31bc744a1db7` | fix(python): reserve planner indices before enumeration | 6 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, `tests/test_codefire_cli.py` |
| `a871399cf8fd74bbb198f34cc0fd3ee1a8b438e6` | `b4f1e0fe2f8fac06dc6649962a909b25bc88a787` | fix(python): bound server verification processes | 8 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, … |
| `9f046eb9653e853c8b8d2c3ec039dad32a2e2fc5` | `3d4034c6f2cad0a147080e978541df8aeda82714` | fix(python): pin verification process capabilities | 2 | `codefire`, `tests/test_python_external_verification.py` |
| `3041710ca8cc6894f0de004601fa2090932a85e9` | `7e4cadf44a551a1a4784118b399680f48dc5ae36` | fix(python): bound verification policy failures | 8 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, … |
| `e00b2959247791506b0e04067464f2efb96f5874` | `308c6b88b9332a9c37e51f95c6789b629147d767` | fix(python): fence verification policy authority | 2 | `codefire`, `tests/test_python_external_verification.py` |
| `a73fd0d1a56f2c687591e82a9af3191ff0dbc1d6` | `24fca185ad57fc6e4b84f4ea843db1af01014878` | fix(python): pin verification project authority | 9 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, … |
| `7052148f58b38e84fe100c6f123b4ab9162c9ade` | `0d76418b4210769c8813ffa60d3d5f0e42b5a166` | fix(python): seal missing verification authority | 2 | `codefire`, `tests/test_python_external_verification.py` |
| `7c00f7bd548a4eeead6dfa287e0389bb8a68115d` | `cfaa9218265b693ebd53984060473bf039f6f884` | fix(python): detach verification failure ownership | 2 | `codefire`, `tests/test_python_external_verification.py` |
| `00fc9f394933def5424888a3a1320bd531f511ab` | `752d500709dad46dd2df516b5331fc5e9226424e` | fix(python): reconcile verification accounting transitions | 2 | `codefire`, `tests/test_python_external_verification.py` |
| `83f3d10e139a0afe1e7cb71a85a43950732e09b1` | `1e32249cdc8bc1d2ad1af0fb7195ea8e2d999783` | fix(python): stabilize verification capability cleanup | 2 | `codefire`, `tests/test_python_external_verification.py` |
| `ca5393e3de06e69f3dab1a75772166e58d2bf454` | `13ee4113f5f8405dc7754ee127746b3b369595f0`, `2d2ba6f49b9d54354b18e49fb3156e0cf3bff1b4`, `e4b7455602dbea91b8f01d7baa19056e810a83a3` | feat(python): register caller-owned work permits | 2 | `codefire`, `tests/test_codefire_cli.py` |
| `fd6634b69cca729ad2b4425c948dc1d74cf18fc4` | `3daabdea76cf7f7de6e72f179ddc8748d3ee3948` | fix(python): distinguish seeded verification authority | 2 | `codefire`, `tests/test_codefire_cli.py` |
| `aa9d484d02c47b4776d8bdc4c416143031ea4d6f` | `2a09a1782293664c4316223d8362714c3000ccc0` | fix(python): never retry consumed capability numbers | 2 | `codefire`, `tests/test_python_external_verification.py` |
| `b212305b0ce79f290e884a35d355d28605b8fe0c` | `98f05ee8cd29e280b81f36c2b52b7fdf2c32dd74` | feat(python): account upload ancestry traversal | 2 | `codefire`, `tests/test_python_ancestry_owner.py` |
| `4714c71c2b434a93aebc7f6d87e21af79eccef95` | `8a415f7c8ab33209425b1e6fe65887ce3386517a`, `20313f008d33fdf3c2b54552de61f567ab23213d` | fix(python): close invalid ancestry subjects | 2 | `codefire`, `tests/test_python_ancestry_owner.py` |
| `1ecf2a90fb75c76b8cdd10e2c32052871ff59bb0` | `04eb43aef7f86fd9b089b6497161b99f5147038a` | fix(python): gate ancestry transition payloads | 2 | `codefire`, `tests/test_python_ancestry_owner.py` |
| `03df431ef8b707aafc02554c117f2766d4a00e11` | `af8baf91443bcb98620860f9d2e1268045535306` | feat(python): account object-set digest work | 6 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, `tests/test_python_object_set_digest.py` |
| `1fbacacf079b8977e72f71f54b721714ab6f7564` | `baad8af84f2e46aab1d46b6c4ac1fa33e9d50ae7` | fix(python): bound object-set digest work | 6 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, `tests/test_python_object_set_digest.py` |
| `272b001aa591f6f24791d8a45d0f538611cb36b0` | `fcc3bb1c8c44b51b0323b5aba09e5fa97eb4f638` | fix(python): reject unbound digest work | 6 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, `tests/test_python_object_set_digest.py` |
| `0503aab3f10d5bcaaf99e04d7570a3c84cf0cec5` | `89743de48a68569b146a4d7682fb2232a6a40165` | fix(python): retain digest permit retry | 4 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `tests/test_python_object_set_digest.py` |
| `e8e129e38f0e54a017223ed7956402dd68e3d12a` | `ffee5189da4cf514abf3ce57e7bfb1e48966fd0f` | fix(python): detach digest failure chains | 3 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `tests/test_python_object_set_digest.py` |
| `ac904830bb6f62f1e4c526c83398a5c685e5bb0c` | `216521891bb81a92625d05b31016c4a4623f4591` | chore: snapshot main CFR-168 integration audit | 273 | `Cargo.lock`, `README.md`, `adr/ADR-009-capability-relative-worktree-writes.md`, `adr/ADR-010-stable-kernel-backed-lock-files.md`, `adr/ADR-011-capability-bound-sealed-validation.md`, `adr/ADR-012-versioned-sealed-semantic-profile.md`, … |
| `816184728d7274a688953f071604c0317b600032` | `42798a6752790403f8af7bc0a3c302e9aca70aed` | feat(python): bind sealed ancestry borrow owner | 8 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `ca62c1036ed38e180aade287ee97f4ab663f7866` | `cfd2c48fe9557c599c00594de86230f35f15e231` | fix(python): publish ancestry handoff destinations | 3 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `tests/test_python_ancestry_borrow_handoff.py` |
| `595c8a248f31a0ac18898cceab1ab568d0776756` | `b6ff7055334525917e357d1989dd94f8b27b4abb` | fix(python): make ancestry handoff two-stage | 8 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `9fd53097c093bdb2b87c91bf3965019f565653d1` | `93d83a482d645f7fef3223355b4c5b8a281e1d2b` | fix(python): account upload staging work | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `dc44e0cbb62b2df22efa1b89b00341bcf24f9c53` | `35e44cdf65b6d4dad873fccd2576d6e24965e35c` | fix(python): seal upload staging preflight | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `55750d134251872126bef455e062672f09b562c0` | `9cd6afa07e52492c18a54a8dd4226aa29369c9bf` | fix(python): freeze upload staging plan | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `851aaffb137eaa0c84afb9976e3a86d00ded1be3` | `8d5ffaddff938efb6b4a90037b2b6fe486fc729b` | fix(python): contain upload stage payloads | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `5b02bd6f298ad1e2597fe25ce53ddfb8614a5f9e` | `bb7626e92ac6df7f95e279a8a9768129af1625d8` | Add accounted Python sealed validation owner | 2 | `codefire`, `tests/test_python_sealed_owner.py` |
| `ecebfebb501acd0a9573918e33e723a8d84690f8` | `5e75e3e74607a0b12b722f141718502efc015b13` | Harden Python sealed validation cleanup | 2 | `codefire`, `tests/test_python_sealed_owner.py` |
| `569368b0c8b881985675bdbc693deb96bbd37b48` | `715a7f010fc97d589f91b6921d744e455ce12131` | Stabilize sealed validation descriptor cleanup | 2 | `codefire`, `tests/test_python_sealed_owner.py` |
| `e70e9108117f5db9591e73520759c56595dae313` | `c91daa17ee07427a55e01594c3599b6f87fb4d8e` | Make sealed descriptor close single-use | 2 | `codefire`, `tests/test_python_sealed_owner.py` |
| `10aa8c115f3eb8b9bf6e7de16e431134d43da949` | `0c5eb2500198edcc3d70f9632ea2190f5b929910` | feat(python): wire sealed upload validation | 1 | `codefire` |
| `8932f27361e104eb23caac12dfa8b2cac086020d` | `a90d736385d5cfdf6c0256768f2178617c3d2b3f` | feat(python): expose sealed commit borrow handles | 2 | `codefire`, `tests/test_python_sealed_borrow.py` |
| `4f33fc1379411cb7cbe10d9d81435d7d17cd3177` | `669c64c988cf23fe61382815793e0b920782efe3` | fix(python): seal retained commit borrows | 2 | `codefire`, `tests/test_python_sealed_borrow.py` |
| `55022daabebcc80e3d1972b44af91b8296c5bba9` | `63180aec91f55064d7a3f9e3df88371846d40bf8` | fix(python): serialize sealed borrow release | 2 | `codefire`, `tests/test_python_sealed_borrow.py` |
| `863a9e55efa2e369417903a865d6c828e454911a` | `de77f025816d63516869111d826eb431b4988583` | fix(python): defer teardown during borrow release | 2 | `codefire`, `tests/test_python_sealed_borrow.py` |
| `1f10c51dc7066147d33044243fd0e3e5d6b1c70f` | `c77c0a48d206166196a86af60569cadafa0631de` | fix(python): reserve sealed fingerprint storage | 1 | `codefire` |
| `6398ba32b3fc1c01babcbb56ee94ba14c9420f85` | `cec3fa60fdb7cc2a8ff9ac341b4da73d3156bc9f` | fix(python): abort ambiguous borrow activation | 2 | `codefire`, `tests/test_python_sealed_borrow.py` |
| `c85f342ac9c6868fe2de7fc64f7be32149e3827d` | `41514a4f6f1a82582412d93227884bbf08f0adcd` | fix(python): close final borrow mutation window | 2 | `codefire`, `tests/test_python_sealed_borrow.py` |
| `80395b3a900fb583b35b98d41ffc861c01f0ea27` | `09c3180c4db714805070bf99eab715fd5887b09b` | fix(python): serialize sealed borrow lookup | 2 | `codefire`, `tests/test_python_sealed_borrow.py` |
| `bbc020288606704d7f1970def07fafc2aa7f122e` | `174df6d391aa47099af3179de0f6602cde480944` | fix(python): defer teardown through sealed finish | 2 | `codefire`, `tests/test_python_sealed_borrow.py` |
| `42774c7eeeb19668970049f42742ccde812bf730` | `b7365e531586212963e23dd8d6e59c7bc23d6cb5` | feat(python): account commit signature policy work | 8 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, … |
| `5b42f1e26dcacf45af3639ecf9125b8e0e85d4b4` | `27335a173313907fb9993c3c42f8d70f93ff0bd7` | fix(python): reconcile signature ledger transitions | 2 | `codefire`, `tests/test_python_signature_owner.py` |
| `07a0a6b3bea3e9bbec50946330406283a091a292` | `0721bc5008a5a50ff2020afa382eefb11ac425af` | fix(python): retain signature scalar ownership in transitions | 2 | `codefire`, `tests/test_python_signature_owner.py` |
| `9bc194c5929ae7d6e180b12940c29ed10dbf9f59` | `00afa5b2d7fe414b8c32af393fdb82fedd851b0c` | feat(python): pin remote signature policy authority | 2 | `codefire`, `tests/test_python_signature_authority.py` |
| `4db51daeff686c530912f30b9396c132c1a4ab8a` | `ccb0b4a7fa8dfa747c73779052e13374a394e7df` | fix(python): repeat project fence after policy read | 4 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `tests/test_python_signature_authority.py` |
| `fd16a24af0d46ee4746e3a454b428c4d7e967441` | `6dfe8a19a14d9750ef6bee965890f3f37e0573ff` | fix(python): detach final policy payload on failure | 4 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `tests/test_python_signature_authority.py` |
| `e0cf57748df1f4dd315985c9f3add6ce49616e0a` | `bf4324de1f12882d6f77b68b71aefab792b0677b` | fix(python): close signature authority review gaps | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `595fafe20330136dcda5eff80ab5e82e3148beb0` | `64c9b6e1c8640956b7a169bce5c232451811e08a` | fix(python): sanitize signature policy parse failures | 6 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, `tests/test_python_signature_authority.py` |
| `453c5c39623ecb8ee6e7ee1bdbc954f27ecf1b73` | `d64ab8da3246b12f8f9bf687aba5d2dc9414d1fb` | merge: combine sealed borrow and signature authority | 8 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `0ba563df031399d1418f4163ba6488df92d19b6b` | `708ae5bf0009706f21b6aad6ed32a43f7fb871a5` | test(python): freeze signature terminal ordering | 5 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, `tests/test_python_signature_terminal_freeze.py` |
| `a1dd00e670f58f6aa72506661b39bfb77093df53` | `e1171fbb0ba4ac850a35acc03e824933f1766621` | merge: combine signature authority and verification owner | 10 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, … |
| `e0ba4a36e7af53c6a53e68e35522a111bb372f08` | `0205390a82e075cb506e6372da433754d60a72c8` | feat(python): lend signature policy to verification | 8 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, … |
| `3329ba909f856bf0ca0d7e03c82e7b1324f6dbb6` | `6c06aa2a69d6a4f53c84c52f534a411256324748` | fix(python): guard full verification handoff | 6 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, `tests/test_python_signature_verification_handoff.py` |
| `4ee7b17526d6f1502737b3a7bb98d5db04877606` | `689477f5aa975783cb6219b762dadf488a7bcff9` | Add sealed signature verification composite | 10 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `6b85f2f94bd5ebb58235d7815f5052f68639f966` | `815bf026ab63d7521af025bb7e8e9e475113ecb3` | feat(python): account upload ancestry traversal | 2 | `codefire`, `tests/test_python_ancestry_owner.py` |
| `81118b8052dbf73a320c669c0f1c00d025d125a7` | `84bad17dfe29dbdd938b04cbf0a6ebf4deddf5ee` | fix(python): gate ancestry transition payloads | 2 | `codefire`, `tests/test_python_ancestry_owner.py` |
| `c104e70a1e9c83447107448d96ab4a912c830495` | `e0f3ead5b791539d716db3375c70c26b84537760` | Integrate sealed ancestry handoff | 8 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `39f61e6e6dc38db0ec952a90fbb825c69617bee1` | `e5e6888cb409fdf7eb54dfacf6c407afcd3b9cd4` | Freeze existing consumer terminal coordination | 7 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, … |
| `59cd4cbcefcd9669749388b5857dbb8ebcc82f97` | `76b3468dfeb601391dc160bffc6a3152fc0c3de3` | feat(python): own upload semantic handoff | 8 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `a3a94e6e15bef56ac3705234c5388df61e3b926e` | `e0d8cf3dae39a5610b6979dfae15620a9068358d` | fix(python): reconcile upload semantic ownership | 8 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `9cbbea6e17fe45ce0edd584b37b23cb04aaa888d` | `bb426f809161fa8e39789504b779ca4e7601290c` | fix(python): account semantic fingerprint scratch | 8 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, … |
| `5c456b8479d040d24df2d473d42f3bb731e7bab2` | `0d3a9f38549ed1ec81198e6de6d185e3d33c363a` | chore: integrate CFR-168 consumer prerequisites | 12 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `1654214464043b586207d767c47eb3374528b159` | `3df9fc943ae149e58ec04e13a7b77be1764e9b52` | feat(python): own existing upload consumers | 11 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `acf9eeb6c27e19a216d3d4a91e7782a354b143e7` | `3fa4c7e6a0f0e2879f9c124eb0e94de73ccae2eb` | fix(python): require terminal parent pair bind | 4 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `tests/test_python_existing_terminal_owner.py` |
| `0f0fc7be66875f1cee69075f86288f8d65e9db4a` | `4d17c3a225e08785a2ccc108f54522d8a169c0ed` | feat: add existing upload pair detach prerequisite | 9 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `f82f8ba6c381f35ed3dad2329fc0ac38cf44a53b` | `57e0901238cac6d98504a783dec43381f03260b4` | fix: freeze pair detach critical functions | 9 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `fe360ab7670e7f5d47a0054031a20a612152d0cb` | `e8545f192fa1f9b88bb70a47852b91e6e9c8d0c6` | fix: pin pair detach subject across callbacks | 6 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `tests/test_python_existing_pair_detach_owner.py` |
| `9ae96b9e2404cbdf0ea3188a21895f8b6b803d4d` | `52b1f3419c13489e3c260a601ea5740d3ab1bc89` | feat(python): add callback-free semantic probe | 9 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `f32955ad5641dc869bfe48476dd05d4fbeabb5ac` | `864b27b665e6779e0ff4777102edfa156e296375` | feat(python): add pair semantic carrier | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`, … |
| `400d321d6735d4301c817fa57534310aa9023c6f` | `66f311a0a9f6f235c7af9b56526a3e81c26c1d86` | feat(python): add existing project gate owner | 9 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `381d2467931ddc388eaa3d6fd8a7ead76fe80774` | `b6262f7a57e87db8f0a3069f2567856c29a6f951` | fix(python): preserve gate descriptor ownership | 9 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `a8b292745247d5515613c8b582201fdb62bc0d94` | `34bc52e8dc866ea3677c739db69aa1d19d4e2cd5` | Add missing-project bootstrap transaction owner | 9 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `3c04e91e11cc0b102dc34d09686a57464c361ba9` | `5910383ae980424460dd436b26d9f72a03515df9` | Harden missing bootstrap callback fences | 4 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `tests/test_python_missing_bootstrap_transaction.py` |
| `ac3e9952059db29522d328a9f7b330361a9c61e0` | `882b55441cbbe71ecd90efe36447357b8eef7182` | Add existing upload effect input owner | 9 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `bca971a11a2c2fc30746f4e17fc9bd039e2a9698` | `da778edddfae8d5ed9eccb53f2d31d8a71addd2c` | Freeze existing response composition boundary | 7 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, … |
| `746fbe0991c2aea77532460192f46814103b7ddd` | `fbbb22c8705f888b44d8a80ccf03786ac583df17` | Add preparse upload response input | 9 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `aaa36337f8230064241cc563085301377d813a33` | `308982069c77a78d738b0214e315156b1e9ee358` | Adopt upload response baseline | 10 | `codefire`, `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, … |
| `8d78fe7cbec4f739023e5d648cc53117119bbb8d` | `16aa0a8ef116ea520ec878d79f212b2b301eb083` | Compose existing upload response ownership | 6 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `tests/test_python_existing_effect_input.py`, `tests/test_python_existing_response_composition.py`, `tests/test_python_response_baseline.py` |
| `706688f47a3df4ed9320f96c0e59cad03e43d61c` | `e99a89f2b1de4329e7c7c99a97b92267c834ce7d` | Freeze existing publication composition boundary | 3 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `tests/test_python_existing_publication_composition_freeze.py` |
| `8ebc2bb2f724730994168c531434fc7b87a88c7a` | `0f75cb31839409357679c824e1a3dc46c98874aa` | Preaccount existing upload publication input | 6 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-data-model-catalog.md`, `tests/test_python_existing_publication_input.py` |
| `0edad52a6de5c5ec7844c314adcf550b27bcdff0` | `d6894967ae1c2ccecc4f5d08723ccd9fad026eb4` | Freeze publication owner adoption boundary | 4 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `tests/test_python_existing_publication_owner_freeze.py` |
| `4d38f7e2815b2843b8c20341a4685fddd088108a` | `bde042ef3c6cf58a133e4d109cbd56a5b8b013d6` | Bind existing publication request identity | 7 | `codefire`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-data-model-catalog.md`, `tests/test_python_existing_effect_input.py`, … |

### B.4 Docs-Only History

| Stable patch ID | Commits | Subject | Paths | Representative paths |
|---|---|---|---:|---|
| `94b6ad865cf18dd816ae3c41b744779725571152` | `aa94eed872d147f9906c3a0c75d518f8707bdd7b` | Clarify Doctor conflict report path accounting | 2 | `docs/development/command-source-trace.md`, `docs/development/source-data-model-catalog.md` |
| `48f58799061db09a1af9bab5ce700de4294a42fb` | `e6d8fae71f1d1212d436863aef7d11935103240f` | docs: record verification authority fence | 6 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `6cdd50362af7f6a5867e6ce18681863e70e73b67` | `63d54a167852d786d0e11f466c2cb21cc801a269` | docs: record verification final authority audit | 6 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `7738d9a022b9503e32c4b343f360d35c367105dd` | `907a919e7353a5cdfd2a4ead57773600e31fa560` | docs: close verification traceback audit | 5 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `fd160535e8c41819f1b0ad36c743f140bedd1313` | `53010e9f1aa7f38fa315d4649a4384fa0b5b6afc` | docs: record complete exception cleanup | 5 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `db7ac1eb64fec20fc288c56c5c0788044abf2d03` | `1ec5ea32fef3ad5d9ef4715d8f8e445c19ef0e7f` | docs: record verification accounting transitions | 5 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `a4d25e56709a263d10547a5b9b9b5b0850dafb55` | `e6d37ac1cc9e4a9216bab7c682c4f790670a6350` | docs: record verification ownership final audit | 5 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `f53ea1abfcdef5f2aca87d7a028ffd685d8ca3f6` | `3fdbf95b5dc5cc1a422151fb0e58235ea40da016` | docs: record seeded verification correction | 4 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md` |
| `515edae335b10e49d6f86c0e2b476be5d5270413` | `3e571a2951cea2ee7059a5058c9aa5d5057812c3` | docs: record capability close ABA contract | 5 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `d6ed2d2c497502d778d80c59aa6bc4803d30385a` | `6d76cd686f22fe54525ed0135babfd68ec69a050` | docs: record verification final evidence | 2 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md` |
| `8d0feea2d35e56736e828c48f38f3c299b111974` | `7c742d7ef8bf9ba15e342d4569489c89653fcc13` | docs: record Python ancestry owner prerequisite | 6 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `5bede99fd6ff09b18af8dd4d98cb67d2e5a2094e` | `ad754a685a063118b56c7d692b324c9edde3ddb3` | docs: update ancestry focused evidence | 2 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md` |
| `a90cd272ca0df3b6c4eb86d965788de01b595e96` | `b7d05444ffc83b7cb50c12e1e3ebf6948aa1f296` | docs: record ancestry allocation gates | 5 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `bdfaaff665eeb3e65c1de6544a78e3608f0460e4` | `d8c0f5444082405030ed732d708b7481f6e76abf` | docs: note deque ancestry overlap upper | 3 | `docs/development/change-playbook.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `deac90facc3699cacd6140e0c2b6e988154ec425` | `d952746007391de403810c515d1c6eb6ccbefbaf` | docs: record ancestry handoff fault evidence | 2 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md` |
| `838b887147f593ca0da83114dd57dbb65250a1b2` | `4ab58a188c8ec39a3502e1b60a12bd8d285686d9` | docs: record upload verification production wiring | 6 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `ba5819039e7928dcf5d4982845d443a442dc6c85` | `31b7a001b82fe7013bb5c9a2dbd309a7fdacb79d` | docs: freeze Python digest production handoff | 6 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `07923561cd3ca531e596ee25d3a19abb49261ce7` | `bbff14909ea2382a4afc283cba5adbdb8af1c903` | docs: close digest terminal mutation window | 6 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `a3335860cb1fe1d7e829d47ee02981ad1d468e18` | `13ef4e205ece08e949d9b152a024d13beb122bb1` | docs: split digest terminal by project state | 6 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `428f5fc67e2921f7a2e0e7285a15d5db38b0dced` | `a32bf5c9ab0f8913780c8349591aaa083f291352` | docs: close missing bootstrap gate handoff | 6 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `29960ed72c3fa7139d88bd0b697a0bbc956b0f90` | `218f0a93b41bc58e032cde3eb0969bd3054eda67` | Document Python sealed validation prerequisite | 5 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `6a0e229aa1d0a54ae7b47481b7928414c8522825` | `796884d3f2d03f119a6675b15cb3d35d304a5d4e` | Document sealed validation cleanup guarantees | 5 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `f4932023742754b2923651807d12adb2f1f14b5b` | `843a983f1b8aa5f7759a1e039ba1e3799ebc60f8` | Document sealed descriptor ownership | 4 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `4a42dafdaf089b913837f279b702f036a9e5e8cf` | `aa8d55bdcb4cc576eaa3e19bb3620d400a1aa779` | Document single-use descriptor close policy | 4 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `2f7680acada1fc0fcfd2c0d4a9893ace9df0da85` | `0c83597bc12a1f7108f42669094b711652ebfb8b` | docs: record sealed upload production wiring | 6 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `f53e5562b841603ad24f88b554a3c3ed7e0d3864` | `428b45104d426cd14802c7d1781341f6c778a8a8` | docs: freeze sealed borrow production boundary | 6 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `72e4b2d3699c2fac22ce8093c6e43ed47c742b89` | `8bde551c856b9953bb0dbedecd4e402b365142c8` | docs: record sealed borrow mutation fence | 6 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `daddb1e2ffeba88aaba8e9bb9cf26fa44e13023c` | `14e8cbbd331ecae442c9f9a076bd36b9254cf8b1` | docs: record sealed borrow release fence | 3 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md` |
| `8c45e7cc7dc524f554272d20c787b8a32a630d7a` | `8764321392c0594a493e3c2c5e80407face6147f` | docs: clarify borrow release reentry fence | 2 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/source-architecture.md` |
| `9a72ae274b8e7a73736d86b1eb9531859c5ab245` | `ee75e1798e56347ce9c18597c99d58cd890cf68c` | docs: state fingerprint allocation upper | 2 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/source-architecture.md` |
| `cd9e3ee4c830b183da137733e732db27687c9d83` | `c9b11f1f8e49922612c5996b2c342afdc8ac0c2a` | docs: record final borrow authority detach | 5 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md` |
| `78f26743c1420b33ed44debbc7825d231a1962b9` | `b5fc4e3504927d2922e89640f7c2f394765b0478` | docs: record sealed lookup reentry fence | 4 | `docs/development/change-playbook.md`, `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md` |
| `eccc38547c415dabdbbb483e0009fb50be9ab89b` | `5257a96afe184330d9a1cab0bcb242ebe25d6f89` | docs: record revoked borrow traceback cleanup | 3 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md` |
| `08dd0787fd372c7e9854ebc1edafcc10c89426bb` | `42eb2c7a2ab207902c5e3e3cddfe551c4f5dff49` | docs: record final detach close guard | 3 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md` |
| `23c554f84fc2dbd8d7bbf72cbec1fa200ff3fa59` | `b83e0256c7e7258ff739daff81d67c1453164807` | docs: record stable signature ledger transitions | 6 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `d779fb953062cf6a324d5e81755eee176f0058f1` | `eada9dbf4a168914bf8bbaf3955a78fe54bb14d3` | docs: record durable signature scalar ownership | 6 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `deb9c44d340bb2d70f07a510e65e2cc2cb33b6f7` | `32e695cfc4ff9c184b0ca1d9cd0f9d470259d698` | docs: record immediate signature transition retirement | 6 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `6a022bce861771af594c93a2fe076ce05f51b50e` | `56303e3080b65148367b4c0e503815f614dbb5e8` | docs: record failed reservation retirement | 4 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `da4c4efd4202ed49f0ef6c3cbeaa2b5525ee67df` | `c41c6f0708913b16a5446ad40acbdbf452a38d84` | docs: record signature policy authority prerequisite | 5 | `docs/development/code-review-issues/cfr-168.md`, `docs/development/command-source-trace.md`, `docs/development/history.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `863cbe0155bdaa1160c814539cc8bab72a097a8c` | `e0b50fd18f1e6260236f06ae0b37dcec88fb9b99` | Freeze publication preimage proof boundary | 1 | `docs/development/code-review-issues/cfr-168.md` |
| `7acf8b393d77448c1523d0c54044bea4c51de72c` | `39cc2a10a96d416316a3b1b669c99be98fe4a800` | Align response retry source documentation | 4 | `docs/development/change-playbook.md`, `docs/development/command-source-trace.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `64e8c3d30d8c7162be7d199a25de1738d11ce2e7` | `ae10db838eeebdbeee6f42da0d13299cf36bda64` | Freeze existing upload filesystem reducer task | 1 | `docs/development/ai/task-packets/cfr-168-existing-filesystem-reducer.md` |
| `7f44da41de3dfcc25cc945a331dd7c5074cb1f14` | `0f63f4c6238a3142cfab202dca079e9745fd9010` | Harden filesystem reducer recovery contract | 1 | `docs/development/ai/task-packets/cfr-168-existing-filesystem-reducer.md` |
| `6d6d302cfc91bfbd27d7204fb39c5081c45d3a45` | `7bc379b69407b62d4a98f84d1fd5bdac3875910c` | Clarify partial write recovery wording | 1 | `docs/development/ai/task-packets/cfr-168-existing-filesystem-reducer.md` |
| `8506d81c92908805d9c41f941e470d1ff5bfd7d8` | `3386496a25adfe496125ba7b605a65c84dd72dc6` | Document execution evidence inventory | 5 | `docs/development/ai/task-packets/cfr-168-existing-typed-reducer.md`, `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md` |
| `832ec09792cc63a76e2cd5cf89a1377738c0d698` | `579867b0314ed017ace84be4e901822742719edf` | Append execution evidence history | 2 | `docs/development/history.md`, `docs/development/known-limitations.md` |
| `5a7c18789ef86d854d2776a4bc11b00019b05690` | `f6f0a8d3be3147a5d152d4d7c5f562e85f0c4bf6` | Document execution evidence accounting checkpoint | 3 | `docs/development/history.md`, `docs/development/known-limitations.md`, `docs/development/source-architecture.md` |
| `d6dded5b0c1deabceff50a5cc6390cecb4082b58` | `3b8897a4e8a0cafff1610b632a8234d4a28a88bc` | Record reducer accounting authority correction | 1 | `docs/development/history.md` |
| `d992c8ea9b45ca433119a3b316d64e7d3a8fae11` | `e326f3f69ea8452af683ee28ce334009f1f0d3de` | Record planner scalar authority correction | 1 | `docs/development/history.md` |
| `091c3d93d1441c63f2d8f86f17edbd81e30f2926` | `aa5e57d8b1e22fb83792412c68f255ceaf603e39` | Document planner function shape authority | 2 | `docs/development/history.md`, `docs/development/source-architecture.md` |

### B.5 Exact Represented Patches

These identities are included for completeness and are excluded from the candidate-only classification totals.

| Stable patch ID | Commits | Subject | Paths | Representative paths |
|---|---|---|---:|---|
| `77a51df68147d5ba015cfb18fb288ad4d994a23d` | `cc75b7f30cacabb2f9bfb84a58561d692ca7ffb3`, `08d26aba539b4297e99936877519922c085d365b`, `5ac817074db44d0fa286e758942a9b593225baa8` | fix(remote): remove unreachable accounted compatibility path | 1 | `crates/codefire-cli/src/remote.rs` |
| `5fd2a90bc8c03e4fe0adad5b84f85e89dea3f027` | `fd28dad991f812c5fd065183948d1d68b141deb6`, `a1b6d19adbd00f06a14704862e63be5d32b0ff13`, `beb2385dd381e55d988c5c0d9beeb1df6075c0ce` | Exercise close failures through exact payload freeze | 1 | `tests/test_codefire_cli.py` |
| `073d6eac70c8b37c5c59a70f81a7485ad4f8bba0` | `e351b4f128a058224819e6c1c54f17c8e9c5dd47`, `a41a0c85467a3086228d9e631d920e08d2226e10`, `38597206ecc02c21082c7f495136971cb2779c75` | Preserve integer subclass rejection diagnostic | 1 | `codefire` |
| `0666918985ad4f1b987820403d1e70940de354e0` | `e2a43381464936ab8ebcf8fce51aa88620195ddc` | feat(capabilities): advertise decision work coverage | 4 | `crates/codefire-cli/src/compatibility.rs`, `crates/codefire-cli/src/tests.rs`, `docs/automation_interface.md`, `docs/development/source-data-model-catalog.md` |
| `7b3a5fcc0f867c1f5739bb3fd35c04e770938781` | `7545be29fea6a33bcb6eec814edefc7ed3822f4d` | Preserve standalone scan accounting | 1 | `crates/codefire-store/src/lib.rs` |
| `1cd2740801d5579766518b4c24cc9a8c29740ba5` | `5397a1650818d70d24354e031d43f1eaf32c8fea` | fix(store): preserve object conversion precedence | 1 | `crates/codefire-store/src/lib.rs` |
| `64e07ae3989c3c9242267242b2c729f1470087c1` | `f847569724dc34c81a85278217e0f787ea19fc75` | test(python): lock planner index ownership order | 1 | `tests/test_codefire_cli.py` |
| `19674f6bf8255f6ddffb94857aa66c5228daa8fd` | `45dd621d2bba827d085eebcf2b66fd47277fbdac` | fix(python): exhaust verification exception chains | 2 | `codefire`, `tests/test_python_external_verification.py` |
| `d231d321da647ee93cced1dba5c53c3722ad3a7e` | `f165cf857f266b9d9b93ce01388b69a1a1af6b08`, `66b4bc9b6362ebcca21be147c9b18e085eb5b95b`, `e589d3299583d0af5287f271715573d1dde69837` | fix(python): make caller-owned permits one-shot | 2 | `codefire`, `tests/test_codefire_cli.py` |
| `aa2ae6684df50c786561b7831dec79734c375129` | `ec7c7b8c27bf5323b78ad454bb328a071f673265` | fix(python): publish verification permits before reserve | 2 | `codefire`, `tests/test_python_external_verification.py` |
| `d26a56ce0f152856e8513a40993c21fa9a5209cd` | `9a3f2f9c9e392e83218146bc3e9b1831e28d43b9` | fix(python): drop popped plan before capability close | 2 | `codefire`, `tests/test_python_external_verification.py` |
| `e88567dc3e90e6ae6fdea5db3f7ab3a597e78e4b` | `7be4d2168b448ac638719521fdee4411d4d16650`, `77ce92ead06c705f6850bef0abb7e84f2b67220c` | fix(python): cover deque ancestry churn | 2 | `codefire`, `tests/test_python_ancestry_owner.py` |
| `fda23f1895ddb55ab1072f80feef08d375e88008` | `1e84cc039eea27c8ef3eafecb51310d6a94f2ac9`, `fd407f0458e937d4ebeee217f90283e484f80c1b` | fix(python): reconcile partial ancestry handoff | 2 | `codefire`, `tests/test_python_ancestry_owner.py` |
| `d7af8306661bdb5cb9908b25e3efff2be7b50102` | `08637b82d7fbb020a4fa89a588bf7913aec16436` | feat(python): wire upload verification accounting | 1 | `codefire` |
| `af739764b632929e53b9594c230ee2c883ec088c` | `92bd5a2a71d7e53641be953c99b2f4f7023fc95e` | test(python): cover upload verification handoff | 1 | `tests/test_python_external_production.py` |
| `e23ff09916a10d2412a37362de5eca407b63bf11` | `c71289ecc5b9058c156012c9f70a2aeac103e1f7` | chore: mirror main integration whitespace fix | 3 | `adr/ADR-014-authoritative-unresolved-merge-state.md`, `tests/test_python_ancestry_owner.py`, `tests/test_python_sealed_production.py` |
| `4f8e2c64fc48974b5dd5d4359acf01f962352822` | `1807bf168f8c68cc973de760c46b808ff74f569f` | test(python): cover sealed upload production handoff | 1 | `tests/test_python_sealed_production.py` |
| `99c2c2897346024df5ea0501737c2a7efc07f2cb` | `70b44f8ebbd7fa04ae32b88d23b0dc0def4a314e` | test(python): drop mutated borrow before final fence | 1 | `tests/test_python_sealed_borrow.py` |
| `23c545733901c9ba9f4d5a4b2cef09bcf908c69f` | `bcd411f7d2bfbd9c84c1f463d06f7d4994dc4601` | fix(python): detach revoked borrow payload frames | 2 | `codefire`, `tests/test_python_sealed_borrow.py` |
| `7bc56231d6ae6157f10cf116133ab13b32a096ea` | `222ecc24d6be2ad7bdd64670abf7dd39ff0c48c2` | fix(python): retire released signature transitions | 2 | `codefire`, `tests/test_python_signature_owner.py` |
| `9bad5eeab57cb061a899925e32e2ba8752d72c55` | `4cd574ea3517191801cb60522fce53c157eacfe3` | fix(python): retire failed signature reservations | 2 | `codefire`, `tests/test_python_signature_owner.py` |

## Smallest Unresolved Candidate Decisions

1. **CFR-169 registry slice:** after CFR-168 is certified, decide whether to replay/reimplement `25e5e3639b9854aac1f241253ae8e33f105b16e1` on the certified base. Do not merge the stale branch wholesale.
2. **CFR-171 claim descriptor:** after CFR-168/169/170 gates, decide the two-patch unit `73025a22c47a6be88ef70cf9563e89ea0d9751a0` + `a77e9ef992a55a916e0a47acadee22babe83bc7b`. The downgrade fix is mandatory if the descriptor slice is adopted.
3. **CFR-172 bounded experiment:** after CFR-168..171 gates, preserve and run the three-commit experiment checkpoint or record an explicit stop decision. Do not misclassify its executable Python/test/schema files as narrative documentation.

No immediate decision is required for response-bridge, typed-reducer, recovery, preimage, execution-evidence, Doctor, Store handoff, CFR-170, or the deep Python signature/sealed frontier. They are represented, superseded, or frozen alternatives under the current snapshot.

## Exact Read-Only Verification Commands

Run from a clean checkout at the immutable snapshot. These commands do not update refs or the worktree.

```bash
S=e2a218e015953f082245f36dbc6aee906b837e5a
B=f1644122134d479333cf422fdb1d48b8fa92dd8e
E=aa5e57d8b1e22fb83792412c68f255ceaf603e39
PY=e5e6888cb409fdf7eb54dfacf6c407afcd3b9cd4

# Snapshot identity and shape.
git show -s --format='%H %P %T %s' "$S"
git rev-list --count "$B..$S"                       # expect 1
git status --porcelain                              # expect empty

# Exact 68-ref scope and immutable tips.
{
  git for-each-ref --sort=refname --format='%(refname:short) %(objectname)' \
    'refs/heads/agent/cfr168-*'
  git rev-parse cfr168-doctor-object-diagnostics
  git rev-parse agent/cfr169-ref-registry
  git rev-parse agent/cfr170-migrate-actions
  git rev-parse agent/cfr171-claim-descriptor
  git rev-parse agent/cfr172-readonly-experiment
}

git rev-parse agent/cfr168-execution-evidence-accounting  # expect aa5e57d...
git rev-parse agent/cfr168-python-existing-consumer-coordinator  # expect e5e6888...

# Ancestry: all candidate lines and snapshot share B, but neither contains the other.
git merge-base "$S" "$E"                         # expect B
git merge-base "$S" "$PY"                        # expect B
git merge-base --is-ancestor "$E" "$S" && echo BAD || echo expected-not-ancestor
git merge-base --is-ancestor "$S" "$E" && echo BAD || echo expected-not-ancestor

# `git cherry` limitation caused by the squash snapshot.
git cherry "$S" "$E" | awk '{count[$1]++} END {print count["+"]+0, count["-"]+0}'
git cherry "$S" "$PY" | awk '{count[$1]++} END {print count["+"]+0, count["-"]+0}'
# expect E: 93 0; PY: 127 0

# Exact content boundaries.
git diff --quiet 216521891bb81a92625d05b31016c4a4623f4591 "$S" -- crates
git diff --quiet bde042ef3c6cf58a133e4d109cbd56a5b8b013d6 "$S" -- codefire
git diff --quiet bde042ef3c6cf58a133e4d109cbd56a5b8b013d6 "$S" -- \
  tests/test_python_existing_effect_input.py \
  tests/test_python_existing_publication_request_identity.py
git diff --quiet f83b53ee1b226ae0d8da0b623e2706f3e2cb6aa1 "$S" -- \
  crates/codefire-cli/src/migration.rs \
  crates/codefire-cli/src/tests/migration_apply.rs

# Required missing artifacts.
git cat-file -e "$S:crates/codefire-core/src/claim_descriptor.rs" 2>/dev/null \
  && echo unexpected-present || echo expected-absent
git cat-file -e "$S:docs/development/experiments/cfr-172/readonly_adoption.py" 2>/dev/null \
  && echo unexpected-present || echo expected-absent
git grep -n 'ObjectReferenceShape' "$S" -- crates/codefire-store/src/lib.rs \
  || echo expected-CFR169-registry-absent

# Stable patch IDs for the six needed patches.
for c in \
  25e5e3639b9854aac1f241253ae8e33f105b16e1 \
  73025a22c47a6be88ef70cf9563e89ea0d9751a0 \
  a77e9ef992a55a916e0a47acadee22babe83bc7b \
  fabfea6c4c4a58c758a9a37ff1025ad49036a408 \
  ef17eb16c26e29facea51b852d4f95d9a2f4304c \
  91954879749114f8aefce267206e95aaf59cec9e
do
  git diff --binary "$c^1" "$c" | git patch-id --stable
done
# expect e92dfce..., c86656a..., 6de8de8..., c86006d..., aaae7f1..., b67af6d...

# Exact represented Store handoff patch (reverse check only; no application).
git diff --binary \
  5397a1650818d70d24354e031d43f1eaf32c8fea^ \
  5397a1650818d70d24354e031d43f1eaf32c8fea \
  | git apply --check --reverse

# Read-only conflict forecasts.
git merge-tree 9c6a8ab6223d8b737ca77f4f11d71e020e147c08 \
  "$S" 1c1b752d99723038de3f6670fdd8e9518915bd04
git merge-tree b40859b1184b17b26b1e5c4898f5afa6383c98a8 \
  "$S" 7c05e3a8a1da3eb822d638e54a1aaefbdefe7e3a
git merge-tree 08d26aba539b4297e99936877519922c085d365b \
  "$S" 25e5e3639b9854aac1f241253ae8e33f105b16e1
git merge-tree 08d26aba539b4297e99936877519922c085d365b \
  "$S" a77e9ef992a55a916e0a47acadee22babe83bc7b
git merge-tree 08d26aba539b4297e99936877519922c085d365b \
  "$S" 91954879749114f8aefce267206e95aaf59cec9e
```
