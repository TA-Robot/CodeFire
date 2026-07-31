# CFR-168 review recurrence gate v2 — implementation result

Status: **READY_FOR_INDEPENDENT_REREVIEW**. This is not live-arm authorization,
integration approval, permanent adoption, or evidence that the review
bottleneck is solved.

Date: 2026-07-31

## 1. Provider and execution truth

Claude Opus 5 produced the initial design and implementation on
`claude-agent/cfr168-review-recurrence-v2-repair-claude-20260731`. The Claude
organization then reached its monthly spend cap, so the manager continued the
same branch with Codex/GPT-5.6 and used three bounded Codex audit lanes for
documentation preparation, implementation-test coverage, and a read-only
adversarial audit. Fugu was unavailable because its quota window does not
reopen until 2026-08-03 00:00.

No Claude or Fugu work is claimed after those providers became unavailable.
No author, manager, or reviewer labor minutes are reported: this run did not
capture authoritative monotonic role timing. Commit timestamps are diagnostic
history, not labor evidence.

## 2. Frozen contract and candidate identity

| Item | Exact value |
|---|---|
| Accepted packet | `docs/development/ai/task-packets/cfr168-review-recurrence-gate-v2-repair-20260731.md` |
| Accepted packet commit | `41824afaace1d172ec12f26d98fa0ae306b4ddcd` |
| Accepted packet SHA-256 | `e08e736cbdb7f48fc131ed2b542db951fbff32ebe1b29d399bfe83172bdea6f6` |
| Manager design adjudication | `f7bfb3c` (`docs/development/ai/swarm-batches/cfr168-review-recurrence-v2-design-adjudication-20260731.md`) |
| Integration base | `1b5e06a3bfe28a83547facfff345d92666ab1bf5` |
| Implementation candidate | `9e50f04b3d6e53753e40aab57cfbc4a884de5bc8` |
| Candidate tree | `56677eb3142e93da650058e67ab2c0779b9e8632` |
| Canonical patch SHA-256 | `2c27dad81a8c6148722f161edc1bc4120db268dd3feb635cfed7f9f75f03540b` |
| Merge base | `1b5e06a3bfe28a83547facfff345d92666ab1bf5` |
| Candidate state at capture | clean; no merge commits; 12 commits; `READY_FOR_REVIEW` |
| Delta | 22 files, 9,382 insertions, 184 deletions |

The identity was recomputed with `capture_candidate_identity.py`, including its
hermetic canonical patch function. This result document is a later report-only
commit and is not part of the implementation subject above.

Implementation commits, oldest first:

1. `2c594ff` — schemas and isolated domain primitives;
2. `22902c6` — topology, authority, and evidence binding;
3. `0829a24` — synthetic v2 fixture;
4. `7189a0a` — fail-closed authority replay;
5. `cf6b51d` — initial adversarial coverage;
6. `fc8d823` — reproducible burden measurement;
7. `ee980d3` — v1/v2 operating documentation;
8. `5f0d65b` — independent-audit replay fixes;
9. `0bc54b3` — complete adversarial matrix;
10. `fcb11c1` — fail-closed replay documentation;
11. `8ff4e14` — development-gate timeout correction;
12. `9e50f04` — direct integration-seam regressions required by first rereview.

## 3. Changed paths and purpose

| Path | Purpose |
|---|---|
| `docs/development/ai/acceleration-master-plan.md` | Replace the rejected v1 recurrence workflow with the experimental v2 lifecycle and human boundary. |
| `docs/development/ai/operating-protocol.md` | Define progressive authority, historical replay, separate validator, timed evidence, and live-arm operation. |
| `docs/development/ai/review-recurrence/README.md` | Document normal v1 `BLOCK`, v2 evidence rules, historical anchor rebind, and operating sequence. |
| `docs/development/ai/schemas/README.md` | Register the human-readable purpose and status of the three v2 schemas. |
| `docs/development/ai/schemas/review-recurrence-control.v2.schema.json` | Count-free v2 review record contract. |
| `docs/development/ai/schemas/review-recurrence-repair-manifest.v1.schema.json` | Manager-frozen obligation, claim, and validation sets. |
| `docs/development/ai/schemas/timed-gate-result.v2.schema.json` | Candidate, log, clean-state, index-state, and post-command HEAD/tree evidence. |
| `docs/development/ai/validation-report-template.md` | Add v2 count-free stopping and evidence checklist. |
| `docs/development/tools/check_ai_contracts.py` | Register only the new schema IDs and paths, as frozen by the packet. |
| `docs/development/tools/check_ai_development.py` | Raise the per-check default from 30 to 60 seconds because the mandatory 107-test suite takes about 32 seconds. |
| `docs/development/tools/check_review_recurrence.py` | Select v1 fail-closed behavior or assemble the v2 verifier and decision; require separate validator and external result. |
| `docs/development/tools/measure_recurrence_v2_burden.py` | Reproduce required-leaf counts without defining an acceptance target. |
| `docs/development/tools/recurrence_v2_authority.py` | Resolve the complete predecessor authority chain to a manager-fixed root. |
| `docs/development/tools/recurrence_v2_decision.py` | Derive closure, state/action blocking, continuity, successor obligations, I1/I5 metrics, and integration decision. |
| `docs/development/tools/recurrence_v2_evidence.py` | Resolve role-bound blobs, manifest equality, timed results, canonical ports, and live/historical anchors. |
| `docs/development/tools/recurrence_v2_fixture.py` | Build real-Git synthetic positive and adversarial histories. |
| `docs/development/tools/recurrence_v2_git.py` | Provide bounded byte-safe Git derivation and pinned canonical patch bytes. |
| `docs/development/tools/recurrence_v2_history.py` | Replay every retained generation oldest-first under current semantics. |
| `docs/development/tools/recurrence_v2_model.py` | Parse the v2 domain model and define fixed vocabularies and provenance limits. |
| `docs/development/tools/recurrence_v2_topology.py` | Verify per-generation chronology and exact write sets. |
| `docs/development/tools/run_timed_gate.py` | Emit v2 monotonic, source-bound, external-log/result execution evidence. |
| `docs/development/tools/test_ai_tools.py` | Exercise the accepted 30-case matrix and independent-audit counterexamples. |

`check_ai_development.py` is the sole scope addition beyond the packet's
expected surface. It was necessary because the packet requires the default
`check_ai_development.py` command, while its expanded tool suite completed in
36.764 seconds and the old wrapper killed it at 30 seconds. The affected
consumer is only the local aggregate AI-development validation command; no
CodeFire product or provider execution timeout changes.

## 4. Implemented decision boundary

The normal entry point now treats v1 as historical compatibility input and
always returns a structured `BLOCK`. A v2 `ALLOW` is reachable only when all of
the following independently hold:

- the exact Git chronology and repair interval are valid;
- every authority link replays oldest-first to a manager-fixed root;
- retained result claims agree with current semantic replay;
- the frozen manifest sets equal the record's obligation, claim, and validation
  sets;
- every changed path has a reviewed surface and no scope referral remains;
- only derived `CLOSED_PROVEN` / `NONE` rows are nonblocking;
- timed v2 evidence binds exact candidate bytes, external complete logs,
  clean-before/after state, the union of before/after index flags, and unchanged
  post-command `HEAD` and tree;
- the deciding gate comes from a separate clean committed validator checkout;
- a live arm binds its current and every historical pin to exact committed
  validator-owned anchor blobs.

The first bootstrap generation is always `BLOCK`. A synthetic rooted second
generation reaches `ALLOW`; that verifies the decision path, not a live arm.

## 5. Independent implementation audit and repairs

The read-only audit inspected exact candidate `95ecb36` before the final repair
commits. Its seven production findings were repaired as follows.

| Severity / finding | Repair | Direct regression test |
|---|---|---|
| High: command could switch to another clean HEAD and still pass | Capture post-command revision/tree and require exact equality. | `test_v2_timed_runner_tracks_head_switch_and_transient_index_flags` |
| High: a pre-existing index flag could be cleared inside the command | Capture flags before and after; decide on their union. | `test_v2_timed_runner_tracks_head_switch_and_transient_index_flags` |
| High/medium: retained live-arm anchors were not replayed historically | Reopen the exact anchor blob at each retained deciding-gate commit and require that commit in current validator ancestry. | helper unit test `test_v2_retained_live_anchor_rebinds_historical_validator_blob`; integrated replay test `test_v2_history_replay_enforces_retained_live_anchor` |
| Medium: a retained result could contradict its replayed validity/findings/rows/metrics | Compare all authority-relevant result fields against current derivation. | `test_v2_history_rejects_self_contradictory_retained_result` |
| Medium: lone-surrogate authored paths could raise instead of block | Reject surrogate JSON text before schema fingerprinting and reject unsafe authored/Git paths before encoding. | `test_v2_malformed_control_path_returns_structured_block` |
| Low: the verified commit set lost the legacy root | Seed progressive replay with the verified root commit. | `test_v2_replayed_chain_keeps_legacy_root_for_prior_evidence_roles` |
| Low: migration text denied the semantic replay that actually occurred | Separate compatibility disposition from `current_gate_full_semantic_replay`. | `test_v2_gate_identity_is_git_verified_and_migration_is_explicit` |

The audit also found incomplete coverage of the accepted validation matrix. The
test fixture and consolidated methods now cover all named gaps, including
manifest thinning, injection-only closure, canonical ports, retained export
tamper, symlink packet/evidence, byte-identical loci, and dirty-state cases.

The first independent rereview of exact candidate `8ff4e14` found zero
production or authorization bypasses and returned `REQUEST_CHANGES` for three
test-only gaps. Commit `9e50f04` closes them without changing production bytes:

- historical pin absence, digest forgery, and validator-ancestry divergence now
  pass through `revalidate_history`, not only the helper;
- both prior-closure and non-reachability roles now prove that the replayed
  manager root is accepted and an unrelated commit is rejected;
- migration now asserts `current_gate_full_semantic_replay` and the replayed
  generation summary directly.

This report is therefore a rereview handoff, not a claim that the first verdict
was acceptance.

## 6. F-1 through F-14 disposition

| ID | Implemented mechanism | Consolidated regression test |
|---|---|---|
| F-1 | Authority-resolved predecessor fixes the exact interval. | `test_v2_rejects_narrowed_interval_and_author_transcribed_shortcuts` |
| F-2 | Packet is the predecessor's direct child with exactly two paths. | `test_v2_real_git_topology_counterexamples_are_structured_blocks` (`late_packet`, `extra_packet_path`) |
| F-3 | Dispatch-attestation shortcuts are absent and rejected as extra properties. | `test_v2_rejects_narrowed_interval_and_author_transcribed_shortcuts` |
| F-4 | Result/log blobs, argv, revision, tree, and manifest-required validations are rebound. | `test_v2_timed_result_rejects_wrong_tree_and_v1_schema`, `test_v2_rejects_narrowed_interval_and_author_transcribed_shortcuts` |
| F-5 | Blocking derives from evidence state, next action, and closure proof; author outcomes are rejected. | `test_v2_routing_states_never_become_integration_authority` |
| F-6 | Obligation identity, origin, and post-closure recurrence derive from the predecessor. | `test_v2_relabelled_origin_and_recurrence_after_closure_are_derived` |
| F-7 | Surfaces are nested per exact path with `minItems: 1`; no escape field exists. | `test_v2_manifest_sets_surfaces_and_count_free_shape_are_exact` |
| F-8 | Review control may write only its exact review packet. | `test_v2_real_git_topology_counterexamples_are_structured_blocks` (`control_rewrites_gate`) |
| F-9 | Control, report, and evidence are a direct ordered chain. | `test_v2_real_git_topology_counterexamples_are_structured_blocks` (`evidence_out_of_order`) |
| F-10 | Invalid Git bytes are found before schema validation; unsafe authored Unicode blocks structurally. | `test_v2_real_git_topology_counterexamples_are_structured_blocks` (`invalid_utf8`), `test_v2_malformed_control_path_returns_structured_block` |
| F-11 | Every authority artifact must be a regular mode-`100644` blob. | `test_v2_real_git_topology_counterexamples_are_structured_blocks` (`symlink_packet`, `symlink_report`, `symlink_evidence`, `executable_report`) |
| F-12 | Each repair commit is inspected and packet rewrite/revert is rejected. | `test_v2_real_git_topology_counterexamples_are_structured_blocks` (`mutate_revert`, `nonlinear_repair`) |
| F-13 | `out_of_scope` is absent; unavailable evidence/ref combinations are checked. | `test_v2_manifest_sets_surfaces_and_count_free_shape_are_exact`, `test_v2_routing_states_never_become_integration_authority` |
| F-14 | Canonical diff flags and a deterministic fixture digest are pinned. | `test_v2_canonical_patch_flags_and_fixture_digest_are_pinned` |

## 7. C-1 through C-16 disposition

| ID | Revised mechanism | Consolidated regression test/evidence |
|---|---|---|
| C-1 | Exact prior-v2 or fixed legacy-root authority; mandatory predecessor equality. | `test_v2_rejects_narrowed_interval_and_author_transcribed_shortcuts`, `test_v2_canonical_port_packet_identities_match_git_objects` |
| C-2 | Two-path packet commit; repair surface derived from Git. | `test_v2_real_git_topology_counterexamples_are_structured_blocks` |
| C-3 | Manifest owns validation set; timed v2 binds source revision/tree. | `test_v2_rejects_narrowed_interval_and_author_transcribed_shortcuts`, `test_v2_timed_result_rejects_wrong_tree_and_v1_schema` |
| C-4 | Only derived proven closure can be nonblocking. | `test_v2_routing_states_never_become_integration_authority`, `test_v2_closed_proven_rejects_missing_and_nonpassing_mapped_results` |
| C-5 | Every path owns a reviewed surface; diagnostic kind cannot grant authority. | `test_v2_manifest_sets_surfaces_and_count_free_shape_are_exact` |
| C-6 | Prior obligation set equals manifest and predecessor export; bootstrap completeness blocks. | `test_v2_bootstrap_blocks_and_rooted_second_generation_allows`, manifest-thinning variants |
| C-7 | Input record is digested; successor verifies exact two-file retention. | `test_v2_retention_commit_must_contain_exactly_record_and_result` |
| C-8 | Injection exemption derives only from the prior row and bound non-reachability evidence. | `test_v2_injection_only_empty_mapping_needs_prior_bound_boundary` |
| C-9 | I5 counts state/action rows and product-repair need separately. | Standard result assertions in `test_v2_routing_states_never_become_integration_authority` and `test_v2_bootstrap_blocks_and_rooted_second_generation_allows` |
| C-10 | I1 is manifest-claim scoped, including byte-identical loci; unavailable remains fail-closed. | `test_v2_byte_identical_locus_still_requires_i1_derivation`, manifest-thinning variants |
| C-11 | Emitted provenance limits restrict claims to the presented topology and machine evidence. | Full fixture/result assertions and documentation contract checks. |
| C-12 | Governing packet and adjudication are revision/digest pinned in section 2. | Candidate identity capture and `check_ai_contracts.py` |
| C-13 | Timed v2 schema/runner and schema registration are implemented. | timed-runner methods plus `check_ai_contracts.py` PASS |
| C-14 | Invalid Git paths are escaped structured findings before schema evaluation. | `test_v2_real_git_topology_counterexamples_are_structured_blocks` (`invalid_utf8`) |
| C-15 | Exact chronology, roles, state/action table, closure, and named quota rejection. | topology, routing, closure, and shortcut consolidated methods |
| C-16 | Normal v1 entry is always structured `BLOCK`; full development gate is required. | `test_v2_no_git_and_v1_normal_entry_are_always_blocks`; aggregate gate PASS |

## 8. R-1 through R-10 and N-1 through N-6 disposition

| ID | Named reader / mechanism | Consolidated regression test |
|---|---|---|
| R-1 | `resolve_predecessor_authority_chain` walks to a fixed root with cycle/depth/generation checks. | `test_v2_authority_generation_skip_and_unrooted_chain_block`, `test_v2_authority_cycle_and_64_65_link_boundary` |
| R-2 | `derive_closure` requires every mapped timed result to pass. | `test_v2_closed_proven_rejects_missing_and_nonpassing_mapped_results` |
| R-3 | `resolve_artifacts` enforces role-compatible commits, modes, and digests; refs are irrelevant. | `test_v2_references_reject_dangling_and_role_incompatible_artifacts`, `test_v2_prior_export_tamper_blocks_even_with_local_ref` |
| R-4 | Topology and the chain walk accept the explicit review/repair fork without pretending the prior record is a candidate ancestor. | `test_v2_bootstrap_blocks_and_rooted_second_generation_allows` |
| R-5 | `bind_timed_validations` requires clean before/after, no flags, and unchanged HEAD/tree. | all three timed-runner/dirty-evidence methods |
| R-6 | `verify_legacy_root` admits only the manager-fixed roots; bootstrap remains `BLOCK`. | `test_v2_bootstrap_blocks_and_rooted_second_generation_allows`, canonical-port identity method |
| R-7 | Successor authority verifies retention parent, exact two-file write set, record/result digests, and exports. | retention, history replay, and export-tamper methods |
| R-8 | `load_manifest` set-compares all three frozen collections. | three manifest-thinning variants in `test_v2_real_git_topology_counterexamples_are_structured_blocks` |
| R-9 | Manifest bytes come from the committed packet artifact; completeness remains independent judgement. | standard positive fixture plus manifest-thinning counterexamples |
| R-10 | `verify_canonical_port_packet` pins both source triples and KGI repair surfaces. | `test_v2_canonical_port_packet_identities_match_git_objects` |
| N-1 | Closure and non-reachability artifacts use explicit role/revision rules and exclude self-authored current evidence. | `test_v2_references_reject_dangling_and_role_incompatible_artifacts`, injection method |
| N-2 | Empty closure mapping is allowed only from an immediate verified `DEFECT_INJECTION_ONLY` predecessor. | `test_v2_injection_only_empty_mapping_needs_prior_bound_boundary` |
| N-3 | Live arms require an absent-by-default manager-owned anchor; no manifest is fabricated. | `test_v2_live_anchor_requires_exact_separate_clean_validator_pin` |
| N-4 | Current and retained deciding-gate identities and anchors are Git verified. | `test_v2_gate_identity_is_git_verified_and_migration_is_explicit`, `test_v2_history_replay_enforces_retained_live_anchor` |
| N-5 | Before/after status, before/after flag union, external outputs, and stable HEAD/tree are mandatory. | timed-runner and dirty-evidence methods |
| N-6 | No out-of-scope escape; unavailable observations must be consistent. | surface/count-free and routing-state methods |

## 9. Accepted Validation 1–30 coverage

| Validation | Consolidated method |
|---|---|
| 1 narrowed interval | `test_v2_rejects_narrowed_interval_and_author_transcribed_shortcuts` |
| 2 late packet | real-Git topology counterexamples |
| 3 fake dispatch | narrowed/shortcut method |
| 4 fabricated/wrong/omitted timed evidence | timed-result method plus narrowed/shortcut method |
| 5 routed non-closure | routing-state method |
| 6 omitted/relabelled/recurred obligation | manifest-set and recurrence methods |
| 7 missing surface / escape property | manifest-set method |
| 8 control rewrites gate | real-Git topology counterexamples |
| 9 ordering/retention mismatch | topology and retention methods |
| 10 invalid path bytes | topology invalid-UTF8 and malformed-control-path methods |
| 11 symlink artifacts | real-Git topology counterexamples |
| 12 mutate/revert | real-Git topology counterexamples |
| 13 unavailable/out-of-scope contradiction | manifest-set and routing-state methods |
| 14 patch flag drift | canonical-patch method |
| 15 byte-identical locus | byte-identical I1 method |
| 16 source proven but unexecuted | routing-state method's `DEFECT_SOURCE_PROVEN` case |
| 17 closure unproven | routing/closure methods |
| 18 author outcomes | narrowed/shortcut method |
| 19 count target / quota fields | manifest-set and narrowed/shortcut methods |
| 20 v1/foreign tree timed result | timed-result method |
| 21 bootstrap cannot allow | bootstrap/second-generation method |
| 22 dangling/role-incompatible artifact | reference method |
| 23 injection-only non-reachability | injection method |
| 24 forged prior chain/export | authority, cycle, retention, and export methods |
| 25 vacuous closure mapping | injection and closure-result methods |
| 26 exact bootstrap role binding without ref authority | canonical-port and reference methods |
| 27 dirty state and mutate/revert boundary | timed-runner and dirty-evidence methods |
| 28 bootstrap then rooted second generation | bootstrap/second-generation method |
| 29 Cargo/KGI exact triples and KGI in-patch paths | canonical-port method |
| 30 thinning any frozen collection | three real-Git manifest-thinning variants |

## 10. I1 and I5 emitted evidence

I1 uses one `derivations[]` row for every manifest parity/closure claim. Each
row binds the predecessor path/locus, predecessor and candidate behavior,
evidence availability, disposition, and next action. Emitted metrics separate
manifest-claim count, derivation-row count, unavailable rows, and changed-path
surface coverage. Git verifies path/topology identity; it does not decide the
semantic quality of the locus or behavior prose.

I5 keeps three facts separate: authored enumerated `evidence_state`, authored
enumerated `next_action`, and machine-derived blocking. Results emit evidence
state counts, next-action counts, blocking rows, unavailable rows,
`product_repair_required`, and successor obligation IDs. No authored outcome
or finding-count field can grant integration authority.

The live cross-arm outcome remains unavailable. No product-repair generation
was run under Cargo or KGI/KPI, so no KGI claim is made.

## 11. Authoring-burden measurement

`measure_recurrence_v2_burden.py` counts required scalar leaves and defines no
minimum, target, or acceptance threshold.

| Shape | Record | Manifest | Combined | v1 comparison |
|---|---:|---:|---:|---:|
| Minimal | 110 | 18 | 128 | v1 record 76 |
| Published realistic shape | 312 | 43 | 355 | v1 record approximately 320 |

The earlier design checkpoint's provisional 109/301 figures omitted the
required nullable surface routing reference and did not add the separate
manifest burden. The executable measurement above supersedes them. A larger or
smaller count is descriptive only; semantic completeness is not inferred from
row count.

## 12. Exact validation evidence

| Command | Result |
|---|---|
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest test_ai_tools.ReviewRecurrenceTests` from `docs/development/tools` | exit 0; 43 tests; 35.587 s |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest test_ai_tools` from `docs/development/tools` | exit 0; 109 tests; 36.764 s |
| `python3 docs/development/tools/check_ai_contracts.py` | exit 0; pass |
| `python3 docs/development/tools/check_ai_acceleration_docs.py` | exit 0; pass |
| `python3 docs/development/tools/check_project_docs.py` | exit 0; 664 Markdown files inventoried |
| `python3 docs/development/tools/check_ai_development.py` | exit 0; all seven components pass |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile ...` for all v2 tools/fixture/tests | exit 0 |
| `git diff --check integration/cfr168-p0-canonical..HEAD` | exit 0; no output |
| `python3 docs/development/tools/measure_recurrence_v2_burden.py` | exit 0; 128 minimal / 355 realistic combined leaves |

The aggregate development gate was then rerun through the candidate's timed-v2
runner against exact candidate `9e50f04`:

| Evidence | Value |
|---|---|
| Command | `python3 docs/development/tools/check_ai_development.py` |
| Status / exit | `passed` / `0` |
| Monotonic duration | 39.646953314 s |
| Candidate revision/tree | `9e50f04b3d6e53753e40aab57cfbc4a884de5bc8` / `56677eb3142e93da650058e67ab2c0779b9e8632` |
| Worktree | clean before/after; no flags; unchanged HEAD/tree; outputs outside worktree |
| Log | `/workspace/.codefire-validation/cfr168-review-recurrence-v2-20260731/ai-development.log` — 159 bytes, SHA-256 `4fbf60b5e5556dbc658be8648b5dd0749710f4e01d3c6993ae77f5852a461ec6` |
| Result | `/workspace/.codefire-validation/cfr168-review-recurrence-v2-20260731/ai-development.result.json` — 2,617 bytes, SHA-256 `bb10eeb23f55ecb639c7bc3d2ef4ea672bf809fc66299d03975f1fd6a310bd74` |

These external files are candidate-validation evidence, not committed live-arm
recurrence evidence. The focused and standalone full-suite outputs were
observed directly but were not separately retained as timed result files.

## 13. Verification layers and remaining limits

The implementation intentionally separates:

1. schema and structural consistency;
2. Git topology, blob, digest, validator, anchor, and timed-evidence identity;
3. human semantic judgement.

It does not machine-prove manifest completeness, surface/symbol completeness,
oracle quality, honest command execution, nonzero test selection, human
authorship, reviewer independence, or the semantic adequacy of a
non-reachability boundary. Mutate-and-restore behavior entirely inside a command
also remains outside the clean-before/after proof.

No live-arm anchor is committed. Therefore neither Cargo nor KGI/KPI can emit a
live `ALLOW`, and the required two-generation comparison has not run. Permanent
adoption requires, in order:

1. an independent adversarial rereview of exact candidate `9e50f04` and this
   report-only handoff;
2. manager-authored and independently preflighted exact Cargo/KGI manifests and
   validator-owned anchors;
3. live generation 1 (`BLOCK` by construction), retention, a real remaining
   repair, and rooted generation 2;
4. comparison of recurrence rate, product-repair generations, review/rework
   time, and authoring burden before deciding whether the bottleneck improved.

Until those steps complete, the accurate claim is: the v2 candidate and its
adversarial test matrix are ready for independent review; the review
bottleneck itself is not yet demonstrated as solved.
