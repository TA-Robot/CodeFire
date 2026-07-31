# CFR-168 Recurrence Gate v2 — Pre-Implementation Design Checkpoint

Implementer: Claude Opus 5 (`claude-opus-5`), implementation lane, auto-worktree
`claude-agent/cfr168-review-recurrence-v2-repair-claude-20260731`. Date: 2026-07-31.

This commit adds only this report. No schema, tool, test, fixture, product,
packet, or prior-report byte is changed by it.

## 1. Frozen dispatch identity, independently recomputed

| Item | Value | Verification |
|---|---|---|
| Implementation base / canonical dispatch revision | `1c8a95fcf46cd3d1cec91f83e1ede5761ceb36bc` | `git rev-parse HEAD` in this worktree → same |
| Accepted task packet commit | `41824afaace1d172ec12f26d98fa0ae306b4ddcd` | — |
| Packet path | `docs/development/ai/task-packets/cfr168-review-recurrence-gate-v2-repair-20260731.md` | — |
| Packet mode / blob | `100644` / `f5314904335f8fed4ed7ec65c6f82b590fef1a42` | `git ls-tree 41824af -- <path>` |
| Packet SHA-256 | `e08e736cbdb7f48fc131ed2b542db951fbff32ebe1b29d399bfe83172bdea6f6` | `git cat-file -p 41824af:<path> \| sha256sum` — **matches** |
| Final adjudication commit | `1c8a95fcf46cd3d1cec91f83e1ede5761ceb36bc` | — |
| Final-review path | `docs/development/ai/swarm-results/cfr168-review-recurrence-v2-contract-preflight-claude-20260731/final-review.md` | — |
| Final-review mode / SHA-256 | `100644` / `fdc0df8111da1bea50737563d5ed3dea1ed989f95c1c728f30fe406edfff336c` | `git cat-file -p 1c8a95f:<path> \| sha256sum` — **matches** |
| Verdict | `ACCEPT_FOR_IMPLEMENTATION` | final-review §10 |
| Both bootstrap authority roots | sole parent = pinned predecessor, write set = exactly the report path, mode `100644`, declared digest | re-verified here for both arms with `git rev-list --parents`, `git ls-tree`, `git diff --name-only`, `sha256sum` — all four conditions hold per arm |

Baseline gate state at this revision, measured before any edit:
`test_ai_tools` 76 tests OK (exit 0), `check_ai_contracts.py` 0,
`check_ai_acceleration_docs.py` 0, `check_project_docs.py` 0,
`check_ai_development.py` 0 (all seven components PASS), `git diff --check` 0.

## 2. Verdict of this checkpoint

**No stop condition. Implementation may proceed.** Details:

- **Not internally contradictory.** §11 records the four contradiction searches
  the packet demands (predecessor identity vs packet chronology vs report-only
  commits vs evidence retention) and how each resolves.
- **No shared or out-of-scope helper is required.** `ai_contracts.py` is read
  only through its existing public functions and is not modified; the v2 path
  needs byte-safe Git plumbing that `canonical_git_patch` cannot provide
  (it decodes paths with `surrogateescape`), so v2 gets its own plumbing inside
  the v2 boundary rather than widening the shared helper. See §4.
- **Larger in one place, and named rather than hidden.** §10 measures the
  minimal v2 record at **109 required leaves against v1's measured 76**
  (+43 %) and one realistic v2 record at **≈301 against v1's ≈320** (−6 %),
  while the machine-verified share rises from roughly 30 % to roughly 97 %.
  The entire minimal-instance increase is the artifact table: 8 exactly-bound
  Git blobs (48 leaves) replacing v1's 5 free-text reference strings. That is
  the F-4/F-11 repair the contract mandates, and the contract-preflight §6
  anticipated the direction ("adds authored surface in two places and removes
  none... both replace judgment with a machine check"). I judge this **expected,
  not unexpectedly larger**, and I state the +43 % explicitly so the manager can
  disagree without reading the schema. No threshold, target, or quota is
  proposed anywhere in this design.

## 3. Frozen N-item decisions as implemented mechanisms

| Decision | Mechanism in this design | Where |
|---|---|---|
| N-1 | `prior_closure_evidence` and `non_reachability_evidence` get an explicit role→revision rule: admissible iff (a) the commit is the candidate or an ancestor of the candidate, or (b) the commit is in the *verified earlier-generation commit set* returned by the rooted chain walk; and in all cases the commit must not be this generation's review-control, report, or evidence commit. Bound by role, commit, path, mode `100644`, digest, chronology and reader like every other artifact. | §5 role table; `recurrence_v2_evidence.resolve_artifact_role` |
| N-2 | The empty closure-validation exemption is never authored. It is read from the predecessor v2 record reached by `resolve_predecessor_authority_chain`: the exemption exists only if that record's obligation row for the same ID has `evidence_state: DEFECT_INJECTION_ONLY`. There is no manifest or record field that can assert it. A bootstrap manifest has no predecessor v2 record, so every bootstrap obligation row must carry a non-empty closure-validation mapping and an empty mapping is a manifest schema/consistency failure. | §6.4; `recurrence_v2_decision.derive_closure` |
| N-3 | No Cargo/KGI manifest is fabricated. A fail-closed extension point is designed and implemented: `--live-arm {cargo,kgi_kpi}` requires a manager-owned anchor `docs/development/ai/review-recurrence/live-arm-anchor.v1.json` that does not exist; without it the run is `BLOCK` with `live-arm-anchor-unavailable`, and without the flag the result emits `live_arm_pin: "unavailable"`. §12 states what is unavailable now and analyses the second-generation repair surface and its sequencing blocker. | §12 |
| N-4 | The v2 result binds `deciding_gate`: validator source repository (distinct from the analysed `--git-root`), its `HEAD` revision, per-artifact committed blob digests for the gate script, the six v2 modules, and the three v2 schemas, plus a derived `state` that must be `clean_committed` for `ALLOW`. The chain walk reads each link's retained `deciding_gate` block, requires it to be present and well-formed, retains it in the result, and emits an explicit `validator_identity_divergence` entry per link whose identity differs from the deciding gate — never a claim that the current revision re-proves the older one. | §7; §9 |
| N-5 | `run_timed_gate.py --emit v2` derives, and the v2 result carries, `worktree.index_flagged_entries` from `git ls-files -v -z` (every entry whose tag is not `H`, which covers `assume-unchanged` lowercase tags and `skip-worktree` `S`) plus `index_flags_clean`. v2 acceptance requires empty clean-before, empty clean-after **and** empty index-flag set. The mutate/revert-during-command limit stays stated in the result's `limits` array and in the documentation. | §8 |
| N-6 | Validation(13) becomes two named tests: `test_out_of_scope_property_is_rejected` (schema `additionalProperties: false`) and `test_unavailable_state_must_match_observations` (an `unavailable` evidence/negative-control state with a non-empty reference list, and its converse, are both rejected). | §9 |
| Nit 1 | No uniqueness of candidate children is encoded anywhere. The record declares its own packet commit, review-control commit, report commit and evidence commit; the gate checks each declared commit's parent and write set. An abandoned sibling attempt is invisible to the proof. Documentation says so in those words. | §6.2 |
| Nit 2 | Retention integrity is verified only by the successor generation's authority resolution. `review-recurrence/README.md` states this in one sentence so a manager citing generation *N* before *N+1* runs knows retention is not yet machine-checked. | §13 |

## 4. Module boundary and justification

The v1 path is a single 1,300-line file whose top-level validator mixes parse,
Git derivation, cross-field verification, decision and rendering. Extending it
would make the rejected surface harder to reason about and would put v2's
authority rules inside the function whose bypasses F-1…F-14 describe. This
design therefore uses a clean v2 boundary: new modules, one responsibility
each, with the v1 bytes and meaning untouched.

| New module | Responsibility (single) | Must not |
|---|---|---|
| `recurrence_v2_model.py` | **Parse.** Vocabularies (9 evidence states, 8 next actions, artifact roles), frozen dataclasses, manager-fixed root/port tables, JSON→typed model with structured findings, `Finding` construction helper. | touch Git, filesystem, or decide anything |
| `recurrence_v2_git.py` | **Derive from Git.** Byte-safe plumbing: commit resolution, parents, per-commit write sets, `ls-tree` mode+blob lookup, blob bytes, ancestry, canonical patch bytes with hermetic diff configuration, numstat, invalid-UTF-8 detection and backslash-escaped rendering. | interpret record semantics |
| `recurrence_v2_topology.py` | **Verify chronology.** One generation's topology: predecessor→packet(2 paths)→per-commit repair series→candidate→review control(1 path)→report(1 path)→evidence(declared paths), plus retention when a successor asks. Bounded traversal with explicit cycle detection. | know about evidence states or decide integration |
| `recurrence_v2_authority.py` | **Verify authority.** `resolve_predecessor_authority_chain`: transitive rooted walk to a manager-fixed legacy root, per-link topology re-verification (calls the topology module — no duplicated rule), generation continuity, candidate identity, record/result digest equality, exported obligation-set re-derivation, link validator identity retention, cycle/depth bounds. Returns the verified earlier-generation commit set N-1(b) needs. | duplicate topology rules or render |
| `recurrence_v2_evidence.py` | **Verify evidence.** Artifact-role resolution (role→permitted revision), mode/digest binding, manifest load + schema + set equality against the record, timed-gate v2 result/log binding, live-arm anchor consumption. | derive blocking or emit the result |
| `recurrence_v2_decision.py` | **Decide.** I5 state/next-action routing table, derived blocking, derived closure proof, derived continuity, I1 derivation rules, derived metrics and counts, integration decision. | read Git or the filesystem |

Rendering stays in `check_review_recurrence.py`'s v2 entry point, which
assembles the result document from parsed model + verified facts + decision and
writes it atomically. Six modules is more files than the packet's surface list
names; the justification is exactly the isolation boundary the dispatch asked
for, and each module's name states the invariant it owns, per
`module-boundaries.md`. Cost/complexity: every traversal is bounded and
`O(commits + paths + artifacts)` with `dict`/`set` lookups; no path, blob, or
manifest is read or parsed twice; blobs are read once and kept as `bytes` with
digest+length only where the payload is not needed.

## 5. Artifact roles: chronology, mode, digest, reader

Every reference is a row in one record-level `artifacts` table:
`{ref_id, role, commit, path, sha256, mode}` with `mode` fixed at `100644`.
Everything else in the record references artifacts by `ref_id` only, so no
digest is transcribed twice, a dangling `ref_id` is detectable, and a
role-incompatible use is detectable. Resolution loads the blob with
`git ls-tree -z <commit> -- <path>`: the entry must exist, its mode must be
exactly `100644` (rejecting `120000` symlinks, `040000` trees and `160000`
gitlinks), and `sha256(blob) == sha256`.

| Role | Permitted revision (chronology) | Mode | Digest bound | Machine reader |
|---|---|---|---|---|
| `repair_packet` | exactly `subject.repair_packet_commit`, whose sole parent is the predecessor candidate | `100644` | yes | `topology.verify_packet_commit` + `evidence.resolve_artifact_role` |
| `repair_manifest` | same commit; the two-path write set binds it without a circular digest | `100644` | yes | same, then `evidence.load_manifest` |
| `candidate_artifact` | exactly `subject.candidate_revision` | `100644` | yes | `evidence.resolve_artifact_role` |
| `review_packet` | exactly `subject.review_control_commit` (direct child of candidate, single-path write set) | `100644` | yes | `topology.verify_review_chain` |
| `report` | exactly `subject.report_commit` (direct child of review control, single-path write set, `swarm-results/` namespace) | `100644` | yes | `topology.verify_review_chain` |
| `timed_result`, `timed_log`, `terminal_event_evidence` | exactly `subject.evidence_commit` (direct child of report; write set is exactly the declared evidence paths under `docs/development/ai/review-recurrence/evidence/<control_id>/`) | `100644` | yes | `topology.verify_review_chain`, then `evidence.bind_timed_validation` |
| `prior_v2_record`, `prior_v2_result` | exactly the prior generation's retention commit, and admissible **only** through the rooted chain walk | `100644` | yes | `authority.resolve_predecessor_authority_chain` |
| `legacy_bootstrap_report` | exactly the manager-fixed root commit of the declared arm (`130878d3…`/`12f9d0b5…`), whose sole parent is the pinned predecessor and whose write set is exactly the pinned report path | `100644` | yes | `authority.verify_legacy_root` |
| `routing_scope_decision` | candidate, or an ancestor of the candidate; never this generation's review-control, report or evidence commit | `100644` | yes | `evidence.resolve_artifact_role` |
| `prior_closure_evidence` | **N-1**: candidate-or-ancestor, **or** a commit in the chain walk's verified earlier-generation set; never this generation's review-control/report/evidence commit | `100644` | yes | `evidence.resolve_artifact_role` with the chain walk's commit set |
| `non_reachability_evidence` | **N-1**: identical rule to `prior_closure_evidence` | `100644` | yes | same |

The verified earlier-generation set is exactly: for each `prior_v2_record`
link, its review-control, report, evidence and retention commits (each already
re-verified by the walk), plus the chain's pinned legacy root commit (a
report-only commit whose parent, write set, mode and digest the manager pinned
and the walk re-verifies). Nothing else is in the set, and a bootstrap
generation's set therefore contains only its pinned legacy root.

There is no "reachable from any ref" predicate anywhere. A local branch or ref
pointing at an object is never evidence.

## 6. Record, manifest and their consumers

### 6.1 Every input field and its consumer

`M` = machine-verified. `H` = human judgement retained deliberately.
`D` = derived by the tool and cross-checked against the authored value.

| Field | Consumer |
|---|---|
| `schema` | M `const codefire.review_recurrence_control.v2`; v1 or unknown → structured `BLOCK` at the entry point |
| `control_id` | M evidence-namespace prefix `…/evidence/<control_id>/` |
| `generation` | M chain continuity (`link.generation == generation-1`), manifest equality |
| `finding_target_policy` | M `const "none"` |
| `predecessor_obligation_completeness` | D `unavailable` iff authority kind is `legacy_bootstrap_report`, `verified` iff `prior_v2_record`; `unavailable` blocks `ALLOW` and stays in the denominator |
| `review_decision` | M any non-`accept` → `BLOCK` |
| `subject.predecessor_candidate_revision` | M must equal the candidate bound by the resolved predecessor authority; all interval derivation uses exactly this bound |
| `subject.candidate_revision` / `candidate_tree` / `patch_sha256` | M resolved commit, `^{tree}`, `sha256(canonical patch bytes)` |
| `subject.changed_paths` | M exact set equality with `git diff --name-only predecessor..candidate`, byte-safe |
| `subject.repair_packet_commit` | M sole parent = predecessor; write set = exactly the two packet paths |
| `subject.review_control_commit` / `report_commit` / `evidence_commit` | M direct-parent chain and exact write sets (§6.2) |
| `subject.repair_packet_ref` / `repair_manifest_ref` / `review_packet_ref` / `report_ref` | M must resolve in `artifacts` with the matching role and commit |
| `subject.predecessor_authority.kind` | M selects the authority reader |
| `subject.predecessor_authority.bootstrap_arm` | M non-null iff kind is `legacy_bootstrap_report`; selects the pinned root row |
| `subject.predecessor_authority.commit` | M retention commit (prior record) or pinned root commit (bootstrap) |
| `subject.predecessor_authority.artifact_refs` | M role set is exactly `{prior_v2_record, prior_v2_result}` or `{legacy_bootstrap_report}` |
| `artifacts[].ref_id` | M unique; every entry must be referenced somewhere; every reference must resolve |
| `artifacts[].role` / `.commit` / `.path` / `.sha256` / `.mode` | M §5 |
| `path_reviews[].path` | M set-equal to `subject.changed_paths` and therefore to Git |
| `path_reviews[].disposition` | M `const "reviewed"` — there is no `no_surface_change` escape and no `out_of_scope` property |
| `path_reviews[].surfaces[]` | M `minItems: 1` per path; a surface is nested inside its path, so it structurally cannot cover another path |
| `…surfaces[].surface_id` | M globally unique across all paths |
| `…surfaces[].kind` | H diagnostic only; cannot change validation or blocking |
| `…surfaces[].scope_disposition` | M `reviewed_in_scope` \| `referred_to_manager`; `referred_to_manager` requires a bound `routing_scope_decision` artifact **and** a linked obligation/finding row in `SCOPE_BOUNDARY`, which blocks |
| `…surfaces[].evidence_ref_ids` | M `minItems: 1`, each resolving to a role-compatible artifact |
| `prior_obligations[].id` | M set-equal to the manifest's prior-obligation IDs **and** to the predecessor's exported successor-obligation set |
| `prior_obligations[].origin_generation` | M `< generation`; for generation ≥ 2 cross-checked against the predecessor record's row |
| `prior_obligations[].evidence_state` | M one of the nine; drives derived blocking |
| `prior_obligations[].next_action` | M must be in the fixed routing set for that state |
| `prior_obligations[].closure_evidence_ref` | M non-null iff derived continuity is `recurred_after_proven_closure`; role `prior_closure_evidence` |
| `prior_obligations[].non_reachability_ref` | M non-null only for the injection-only closure alternative; role `non_reachability_evidence` |
| `prior_obligations[].routing_decision_ref` | M non-null iff `next_action ∈ {MANAGER_SCOPE_DECISION, MANAGER_REQUIREMENT_DECISION}`; role `routing_scope_decision`; retained for routing only and cannot change blocking |
| `…observations.before_ref_ids` / `after_ref_ids` | M non-empty for `CLOSED_PROVEN`; each role-resolved |
| `…observations.negative_control` | M `killed` \| `not_applicable` for `CLOSED_PROVEN`; `unavailable` requires an empty reference list |
| `…observations.negative_control_ref_ids` | M non-empty unless the state is `unavailable`, which requires empty |
| `new_findings[]` | M same row shape minus `origin_generation`/`closure_evidence_ref`; `CLOSED_PROVEN` is barred in the same generation; IDs join the derived successor set |
| `derivations[].claim_id` | M set-equal to the manifest's parity/closure claim IDs |
| `derivations[].path` | M existence in the predecessor tree is checked by Git |
| `derivations[].locus` | H stable symbol/locus identifier; Git cannot decide a symbol claim and the tool does not pretend to |
| `derivations[].locus_existed_in_predecessor` | M contradiction against the predecessor tree is rejected |
| `derivations[].predecessor_behaviour` / `candidate_behaviour` | H reviewer-authored derivation prose, required by I1; never described as machine-proved |
| `derivations[].evidence.state` | M `available` \| `unavailable`; `unavailable` is fail-closed for `ALLOW` and retained in the denominator |
| `derivations[].evidence.ref_ids` | M non-empty iff `available`, empty iff `unavailable` |
| `derivations[].disposition` / `.next_action` | M enumerated; a claim mapped to an obligation must have a disposition |
| `validations[].id` | M set-equal to the manifest's required validation IDs |
| `validations[].result_ref` / `.log_ref` | M roles `timed_result`/`timed_log` at the evidence commit; the result's `log.sha256` and `log.bytes` must equal the committed log blob |
| `validations[].status` / `.exit_code` / `.timed_out` / `.log_complete` | D must agree with the committed timed v2 result blob |
| `terminalization.state` / `.worker_alive` / `.child_processes_active` | M `complete`, `false`, `0` required for `ALLOW` |
| `terminalization.event_evidence_ref` | M role `terminal_event_evidence` at the evidence commit |

Everything else the record used to carry is derived and emitted, not authored:
counts by evidence state and next action, `product_repair_required`,
continuity, blocking, `successor_obligation_ids`, repair-surface path count,
inserted/deleted lines, binary paths, coverage ratios, and the record digest.

### 6.2 Exact chronology proved

```text
predecessor candidate
  ├─ packet commit           sole parent = predecessor; write set = exactly
  │                          {repair packet .md, repair manifest .json},
  │                          both mode 100644 under docs/development/ai/task-packets/
  │   └─ repair commits      single-parent chain, inspected commit by commit;
  │                          may not rewrite either packet artifact; every path
  │                          ⊆ repair_surface = changed_paths − the two packet paths
  │       └─ candidate       tree/patch/changed-path identity
  │           └─ review control   sole parent = candidate; write set = exactly one
  │               │               mode-100644 review-packet path
  │               └─ report       sole parent = review control; write set = exactly one
  │                   │           mode-100644 path under swarm-results/
  │                   └─ evidence sole parent = report; write set = exactly the declared
  │                               evidence artifact paths under
  │                               review-recurrence/evidence/<control_id>/
  │                       ⇒ evaluate uncommitted --record; emit record SHA-256
  │                           └─ retention  sole parent = evidence; write set = exactly
  │                                         {record path, result path} under
  │                                         review-recurrence/records|results/
  └─ (sibling) the predecessor's own completed review chain
```

Steady state is an explicit fork and the design does not encode how many
children the predecessor candidate has: the record declares each commit and the
gate checks that commit's parent and write set. An abandoned attempt is
therefore invisible to the proof rather than a failure. `repair_surface` must be
non-empty. The retention commit is verified only later, by the successor
generation's authority resolution.

This proves topology **in the history presented to the gate**. Git does not
prove when text was authored and cannot detect a rewritten history that
presents a conforming topology. Both statements appear in the emitted `limits`
array and in the documentation.

### 6.3 Manifest anchors

`codefire.review_recurrence_repair_manifest.v1`, `additionalProperties: false`,
loaded from the Git blob at the packet commit (never from the worktree):

| Manifest content | Anchor / consumer |
|---|---|
| `schema`, `manifest_id` | identity |
| `generation` | must equal the record's |
| `predecessor_candidate_revision` | must equal the record's |
| `predecessor_authority` `{kind, bootstrap_arm, commit}` | must equal the record's authority reference |
| `repair_packet_path`, `repair_packet_sha256` | must equal the record's `repair_packet` artifact and the actual blob at the packet commit |
| `finding_target_policy: "none"` | `const` |
| `prior_obligations[] {obligation_id, closure_validation_ids[]}` | unique IDs; set-equal to the record's obligation IDs and to the predecessor's exported successor set; every referenced closure-validation ID must exist in `required_validations`; empty list admissible only under N-2 |
| `parity_closure_claims[] {claim_id, kind, obligation_id, locus_path}` | unique; set-equal to the record's derivation claim IDs; a non-null `obligation_id` must exist in `prior_obligations` and forces that claim's derivation disposition |
| `required_validations[] {validation_id, argv[]}` | unique; set-equal to the record's validation IDs; `argv` must equal the committed timed v2 result's `argv` exactly |

The manifest is the manager's discretion boundary. Set equality prevents later
weakening; it cannot prove the manager chose a complete set, and independent
packet preflight owns that judgement without a count target.

### 6.4 Derived closure (R-2 + N-2)

`CLOSED_PROVEN` is never a label the author can assert:

1. look up the manifest mapping `M(id)` for that obligation;
2. if `M(id)` is non-empty: every ID in `M(id)` must appear in
   `validations`, and each must bind a committed `codefire.timed_gate.v2`
   result with `status: passed`, `exit_code: 0`, `timed_out: false`,
   `log.complete: true`, empty clean-before, empty clean-after, empty index-flag
   set, and candidate revision/tree equal to the bound candidate;
3. if `M(id)` is empty: admissible only when the predecessor v2 record reached
   by the rooted chain has that obligation in `DEFECT_INJECTION_ONLY` **and**
   the row binds a role-resolved `non_reachability_evidence` artifact. No
   predecessor v2 record (bootstrap) ⇒ never admissible;
4. a new finding of this generation can never be `CLOSED_PROVEN`;
5. otherwise the row is blocking with a structured finding.

The tool proves identity, set equality and passing results. It does not claim
those oracles semantically prove closure, and it does not claim the
non-reachability boundary's semantic content — only its bytes, role and
chronology.

## 7. I1 and I5: schema rows, rules, emitted metrics

### I1 predecessor derivation

Rows: `derivations[]` as tabled in §6.1, set-equal to the manifest's
parity/closure claim IDs — surface count creates no additional I1 obligation.
An `accept` fails when a manifest claim has no derivation row, an asserted
predecessor path contradicts the predecessor tree, the derivation evidence is
`unavailable`, or a claim mapped to a prior obligation lacks its disposition.
No synthetic byte difference is required, and the byte-identical case is
exactly the case the derivation must cover.

Emitted: `i1.manifest_claims`, `i1.derivation_rows`,
`i1.rows_unavailable` (retained in the denominator),
`i1.changed_surface_coverage {changed_paths, paths_with_surface, surfaces}` as a
separate diagnostic. These measure row presence and artifact binding, not
derivation quality or historical capture. Rust arms stay fail-closed: an
`unavailable` derivation row cannot ride an `ALLOW`, which is the explicit
consequence of the replay's five unavailable Rust rows and not a claim that I1
was validated on Rust.

### I5 evidence state vs next action

Three separate concerns, third derived:

| Concern | Field | Source |
|---|---|---|
| what the reviewer established | `evidence_state` (9 values) | authored, enumerated |
| what should happen next | `next_action` (8 values) | authored, enumerated, checked against the routing table |
| whether it blocks integration | not a field | derived from the pair plus proven closure |

Routing table (authored `next_action` must be in the row's set):

| `evidence_state` | permitted `next_action` | blocking |
|---|---|---|
| `DEFECT_REPRODUCED` | `PRODUCT_REPAIR` | yes |
| `DEFECT_SOURCE_PROVEN` | `PRODUCT_REPAIR` | yes |
| `DEFECT_INJECTION_ONLY` | `DOCUMENT_NON_REACHABILITY`, `PRODUCT_REPAIR` | yes |
| `CLOSURE_UNPROVEN` | `RUN_CLOSURE_EVIDENCE` | yes |
| `ORACLE_VACUOUS` | `REPAIR_ORACLE` | yes |
| `CLAIM_FALSE` | `REPAIR_CLAIM` | yes |
| `SCOPE_BOUNDARY` | `MANAGER_SCOPE_DECISION` | yes |
| `REQUIREMENT_UNSATISFIABLE` | `MANAGER_REQUIREMENT_DECISION` | yes |
| `CLOSED_PROVEN` | `NONE` | no, **and only** when §6.4 proves closure |

`ACCEPT_WITH_NOTES` does not exist. Routing changes the kind of successor work,
never integration authority. `SCOPE_BOUNDARY`'s retained owner/release/authority
artifact is routing evidence only; Git binds its bytes and ancestry and cannot
attest human authorship.

Emitted: `i5.evidence_state_counts`, `i5.next_action_counts`,
`i5.blocking_rows`, `i5.product_repair_required` (derived record-level boolean),
`i5.unavailable_rows` reported separately, and `successor_obligation_ids`. The
manager's cross-arm measure is **product-repair generations avoided**
(denominator: every reviewed generation; numerator: generations with no
`PRODUCT_REPAIR` action; unavailable reported separately). It is deliberately
not the replay's `6 / 18` candidate-generation-removal metric and must not be
compared as though it were. Authors transcribe none of these outcomes.

## 8. Timed gate v2

`run_timed_gate.py` keeps `run_gate` (v1) byte-compatible and adds
`run_gate_v2` behind `--emit v2`. v2 derives, rather than accepts as a string:

- the Git root containing its `cwd` (`rev-parse --show-toplevel`), canonical
  `HEAD` and `HEAD^{tree}`, emitted as `source {git_root, revision, tree}`;
- `git status --porcelain=v1 -z --untracked-files=all` immediately before and
  after the command, both emitted, both required empty for v2 acceptance;
- `git ls-files -v -z` entries whose tag is not `H`, emitted as
  `index_flagged_entries` with `index_flags_clean`; required empty (N-5);
- `log_inside_source_worktree`, so the documented practice of writing logs
  outside the source worktree and copying them into the evidence commit is
  visible rather than assumed;
- `limits[]`, including the exact sentence that a command which mutates and
  restores bytes during its own execution is outside this proof.

The recurrence gate requires the result schema to be `codefire.timed_gate.v2`,
its `source.revision`/`source.tree` to equal the bound candidate, its digest to
match the committed blob, its `argv` to equal the manifest's frozen argv, and
its log digest **and byte count** to equal the committed log blob. `cwd` and
`log.path` remain machine-specific absolute paths, diagnostic only. A v1 result,
a v2 result from another tree, a fabricated digest, a missing blob, a symlink, a
worktree-only file, an edited result or an edited log all fail closed. The gate
does not claim honest execution, semantic oracle quality, nonzero test
selection, or absence of mutate/revert inside the command; the generic runner
cannot prove a selector chose a nonzero corpus and v2 says so.

## 9. Disposition tables with mechanism, counterexample and test

All tests are named; none is chosen to reach a count. Tests use fixtures and
synthetic histories only; the real Cargo/KGI histories are never mutated.

### F-1 … F-14

| Finding | Mechanism | Adversarial counterexample | Test |
|---|---|---|---|
| F-1 base narrowing | predecessor resolved from the authority artifact; exact interval equality | interval narrowed to a later ancestor on candidate ancestry | `test_narrowed_interval_is_rejected` |
| F-2 post-repair packet | packet commit's sole parent is the predecessor; two-path write set; no repair byte | packet committed after the first repair commit | `test_packet_after_repair_is_rejected` |
| F-3 fake dispatch event | no timing/dispatch field exists in v2; `additionalProperties: false` | v1's `dispatch_event_attested` triple re-added | `test_dispatch_attestation_has_no_v2_field` |
| F-4 fake evidence/log digests | candidate-bound timed v2 plus exact committed result/log bytes | fabricated 64-hex digest and hand-edited log | `test_fabricated_timed_evidence_is_rejected` |
| F-5 dead severity/blocking | blocking derived from the enum pair plus proven closure; no `blocking`/`severity` field | `SCOPE_BOUNDARY` with `blocking: false` | `test_only_proven_closure_is_nonblocking` |
| F-6 dead continuity | obligation-ID set equality with the predecessor's exported set; continuity derived | one prior obligation silently dropped; `continuous_open` claimed after proven closure | `test_thinned_obligation_set_is_rejected`, `test_recurrence_after_closure_needs_evidence` |
| F-7 one surface covers many paths | surfaces nested under their path; `minItems: 1` per path; no escape property | two changed paths, one surface row | `test_every_changed_path_needs_a_surface` |
| F-8 control rewrites gate | review control's write set is exactly its one review-packet path | control commit that also edits the gate script | `test_control_commit_may_only_write_its_packet` |
| F-9 impossible ordering | exact direct-parent chain and three namespaces | evidence commit before the report commit | `test_review_chain_order_is_exact` |
| F-10 invalid UTF-8 Git path | byte-safe plumbing detects it **before** schema validation and renders backslash-escaped | changed path containing `\xff` | `test_invalid_utf8_path_precedes_schema_check` |
| F-11 symlink report/packets | `ls-tree` mode must be exactly `100644` | report path committed as a `120000` symlink | `test_symlink_artifact_is_rejected` |
| F-12 mutate/revert control chain | direct children for the control chain; per-commit inspection elsewhere | repair series that edits then restores the packet | `test_mutate_revert_repair_series_is_rejected` |
| F-13 contradictory unavailable/out-of-scope | no `out_of_scope` property; `unavailable` ⇔ empty reference list | `out_of_scope` added; `unavailable` with references | `test_out_of_scope_property_is_rejected`, `test_unavailable_state_must_match_observations` |
| F-14 self-referential patch fixture | pinned byte-level canonical patch digest over a committed fixture pair, with hermetic diff configuration | a changed diff flag | `test_canonical_patch_digest_fixture_is_pinned` |

No finding is marked fixed because a field was renamed: F-3, F-5, F-13 are
closed by **removing** the field and rejecting it structurally.

### C-1 … C-16

| Finding | Revised clause → mechanism | Test |
|---|---|---|
| C-1 | predecessor from a prior v2 record or one of two exact legacy report-only commits; equality mandatory | `test_narrowed_interval_is_rejected`, `test_legacy_root_binding_is_exact` |
| C-2 | packet commit changes exactly two named artifacts; admissible extra manager-control set is empty; repair surface Git-derived | `test_packet_write_set_is_exactly_two_paths`, `test_repair_commit_outside_surface_is_rejected` |
| C-3 | timed v2 derives candidate revision/tree; validation set comes from the manifest; roles enforce reachability | `test_foreign_tree_timed_result_is_rejected`, `test_manifest_required_validation_cannot_be_omitted` |
| C-4 | only `CLOSED_PROVEN`/`NONE` is nonblocking | `test_only_proven_closure_is_nonblocking`, `test_review_chain_cannot_self_authorize` |
| C-5 | `no_surface_change` removed; `kind` diagnostic only | `test_every_changed_path_needs_a_surface`, `test_surface_kind_does_not_change_blocking` |
| C-6 | prior-obligation IDs set-equal to prior authority; bootstrap completeness `unavailable` blocks | `test_thinned_obligation_set_is_rejected`, `test_bootstrap_generation_blocks_and_seeds_successor` |
| C-7 | uncommitted evaluation input digested; retention commit and three namespaces explicit | `test_result_emits_record_digest`, `test_retention_commit_is_verified_by_successor` |
| C-8 | injection-only closure needs an independently accepted non-reachability boundary; bytes/role only | `test_injection_only_cannot_close_on_reachability`, `test_non_reachability_role_revision_rule` |
| C-9 | I5 measures product-repair generations avoided; incomparable to `6 / 18` | `test_result_emits_i5_metrics` + documentation sentence |
| C-10 | I1 manifest-claim scoped; Rust unavailable fail-closed; row coverage output | `test_derivation_claims_are_manifest_scoped`, `test_unavailable_derivation_blocks_allow` |
| C-11 | claims limited to presented-history topology | `limits[]` assertion in `test_result_emits_provenance_limits` |
| C-12 | governing artifacts pinned by revision/mode/digest | verified in §1 of this report and in `result.md` |
| C-13 | timed v2 runner/schema and `check_ai_contracts.py` registration in scope | `test_timed_gate_v2_binds_source_and_worktree_state`, `test_v2_schema_ids_are_registered` |
| C-14 | invalid Git path bytes rejected before schema validation and JSON-escaped | `test_invalid_utf8_path_precedes_schema_check` |
| C-15 | direct-parent/per-commit rules, roles, `next_action`, `CLOSED_PROVEN`, named quota fields exact | `test_review_chain_order_is_exact`, `test_named_quota_fields_are_rejected` |
| C-16 | v1 before evidence pinned to `1cae0225`; full AI development gate still required | `test_v1_synthetic_fixture_shows_old_allow`, `test_v1_record_returns_structured_block` |

### R-1 … R-10

| Finding | Named machine reader | Adversarial counterexample | Test |
|---|---|---|---|
| R-1 | `resolve_predecessor_authority_chain` | chain that never reaches a pinned root; cyclic chain; skipped generation | `test_unrooted_authority_chain_is_rejected`, `test_cyclic_authority_chain_is_rejected`, `test_generation_skip_in_chain_is_rejected` |
| R-2 | `recurrence_v2_decision.derive_closure` | `CLOSED_PROVEN` with empty mapping, missing mapped result, or failing mapped result | `test_closed_proven_needs_mapped_passing_results` |
| R-3 | `recurrence_v2_evidence.resolve_artifact_role` | object that exists at a role-incompatible commit; local ref pointing at a dangling commit | `test_role_incompatible_commit_is_rejected`, `test_local_ref_is_not_evidence`, `test_dangling_artifact_ref_is_rejected` |
| R-4 | `topology.verify_packet_commit` + the chain walk | predecessor record treated as a candidate ancestor | `test_sibling_fork_authority_is_accepted_off_lineage` |
| R-5 | `evidence.bind_timed_validation` | non-empty clean-before/after; index bits hiding a modification | `test_dirty_before_or_after_blocks_v2_accept`, `test_index_flags_block_v2_accept` |
| R-6 | `authority.verify_legacy_root` | second bootstrap-like legacy root invented | `test_bootstrap_generation_blocks_and_seeds_successor`, `test_unpinned_legacy_root_is_rejected` |
| R-7 | successor generation's authority resolution | retention commit with a wrong parent or a third path; record/result digest mismatch | `test_malformed_retention_commit_is_rejected`, `test_retention_digest_mismatch_is_rejected` |
| R-8 | `evidence.load_manifest` set equality | any one of the three collections thinned | `test_manifest_collection_thinning_is_rejected` |
| R-9 | manifest is loaded from the packet-commit blob only | manifest edited in the worktree after freeze | `test_manifest_is_read_from_git_not_worktree` |
| R-10 | port fixtures bind path/source-commit/digest triples | wrong port digest; in-patch `swarm-batches` artifact treated as packet authority | `test_port_packet_fixtures_bind_exact_triples`, `test_in_patch_dispatch_artifact_is_a_repair_surface` |

### N-1 … N-6 and the packet's Validation(1…30)

| Item | Test |
|---|---|
| N-1 | `test_closure_evidence_role_revision_rule`, `test_non_reachability_role_revision_rule`, `test_self_authored_boundary_in_own_evidence_commit_is_rejected` |
| N-2 | `test_bootstrap_manifest_requires_nonempty_mappings`, `test_exemption_comes_from_predecessor_record` |
| N-3 | `test_live_arm_pin_is_unavailable_and_fails_closed` |
| N-4 | `test_result_binds_validator_identity`, `test_dirty_validator_blocks_allow`, `test_chain_retains_link_validator_identity` |
| N-5 | `test_index_flags_block_v2_accept` |
| N-6 | `test_out_of_scope_property_is_rejected`, `test_unavailable_state_must_match_observations` |
| V1 narrowed interval | `test_narrowed_interval_is_rejected` |
| V2 late packet | `test_packet_after_repair_is_rejected` |
| V3 fake dispatch | `test_dispatch_attestation_has_no_v2_field` |
| V4 fabricated timed result / wrong tree / omitted validation | `test_fabricated_timed_evidence_is_rejected`, `test_foreign_tree_timed_result_is_rejected`, `test_manifest_required_validation_cannot_be_omitted` |
| V5 free-boolean routing of a non-closed state | `test_only_proven_closure_is_nonblocking`, `test_review_chain_cannot_self_authorize` |
| V6 omitted obligation / relabelled continuity / recurrence without evidence | `test_thinned_obligation_set_is_rejected`, `test_recurrence_after_closure_needs_evidence` |
| V7 missing surface / `no_surface_change` | `test_every_changed_path_needs_a_surface`, `test_no_surface_change_property_is_rejected` |
| V8 control rewrites gate/schema | `test_control_commit_may_only_write_its_packet` |
| V9 out-of-order evidence / retention mismatch | `test_review_chain_order_is_exact`, `test_retention_digest_mismatch_is_rejected` |
| V10 invalid UTF-8 path | `test_invalid_utf8_path_precedes_schema_check` |
| V11 symlink artifacts | `test_symlink_artifact_is_rejected` |
| V12 mutate/revert history | `test_mutate_revert_repair_series_is_rejected` |
| V13 (restated per N-6) | `test_out_of_scope_property_is_rejected`, `test_unavailable_state_must_match_observations` |
| V14 canonical patch flag drift | `test_canonical_patch_digest_fixture_is_pinned` |
| V15 byte-identical locus without derivation | `test_byte_identical_locus_needs_derivation` |
| V16 source-proven but unexecuted | `test_source_proven_is_not_reproduced` |
| V17 `CLOSURE_UNPROVEN` | `test_closure_unproven_never_allows` |
| V18 author-transcribed outcomes | `test_author_transcribed_outcomes_are_rejected` |
| V19 `finding_target_policy` and named quota fields | `test_named_quota_fields_are_rejected` |
| V20 v1 or foreign-tree timed result | `test_v1_timed_result_is_rejected`, `test_foreign_tree_timed_result_is_rejected` |
| V21 bootstrap completeness | `test_bootstrap_generation_blocks_and_seeds_successor` |
| V22 dangling / role-incompatible commit | `test_dangling_artifact_ref_is_rejected`, `test_role_incompatible_commit_is_rejected` |
| V23 injection-only closure | `test_injection_only_cannot_close_on_reachability` |
| V24 forged prior record chain | `test_unrooted_authority_chain_is_rejected`, `test_cyclic_authority_chain_is_rejected`, `test_generation_skip_in_chain_is_rejected`, `test_malformed_retention_commit_is_rejected`, `test_chain_obligation_export_tamper_is_rejected` |
| V25 vacuous `CLOSED_PROVEN` | `test_closed_proven_needs_mapped_passing_results`, `test_injection_only_cannot_close_on_reachability` |
| V26 pinned bootstrap without local refs | `test_legacy_root_binding_is_exact`, `test_local_ref_is_not_evidence` |
| V27 clean-before/after with the stated limit | `test_dirty_before_or_after_blocks_v2_accept` |
| V28 first generation blocks, second is eligible | `test_bootstrap_generation_blocks_and_seeds_successor`, `test_second_generation_can_reach_allow` |
| V29 canonical-port triples and in-patch artifacts | `test_port_packet_fixtures_bind_exact_triples`, `test_in_patch_dispatch_artifact_is_a_repair_surface` |
| V30 manifest collection thinning | `test_manifest_collection_thinning_is_rejected` |

v1's before behaviour is demonstrated with an explicitly labelled synthetic v1
fixture (`test_v1_synthetic_fixture_shows_old_allow`): the rejected v1
authorization rule is reproduced locally under that label and shown to yield
`ALLOW` for a record that the v2 entry point returns as a structured `BLOCK`
(`test_v1_record_returns_structured_block`). Deleting v1 authorization at HEAD
is not offered as evidence of its old behaviour, and `validate_recurrence_control`
plus its existing tests keep v1's historical meaning byte-intact.

## 10. Required-leaf measurement (no threshold, no target)

Counting rule, applied identically to v1 and v2: one leaf per required scalar
in the schema's `required` closure; a required array contributes
`(number of items in the instance) × (leaves of its item schema)`; nested
required objects expand. v1's published figures use the independent review's own
rule (§6 of `…independent-review…/review.md`: 76 for the smallest possible
record, ≈320 for a record describing that candidate — 11 changed paths, 3 prior
obligations, 6 surfaces, 5 validations), and they are quoted, not recomputed.

Minimal v2 record (1 changed path, 1 surface, 1 prior obligation, 0 new
findings, 1 derivation claim, 1 validation, bootstrap authority):

| Block | Leaves | Machine | Human |
|---|---:|---:|---:|
| top-level scalars | 6 | 6 | 0 |
| `subject` (15 scalars + 1 changed path + 1 authority ref) | 17 | 17 | 0 |
| `artifacts` — 8 bound blobs × 6 | 48 | 48 | 0 |
| `path_reviews` — 1 path × (2 + surface(3 + 1)) | 6 | 5 | 1 (`kind`) |
| `prior_obligations` — 1 row | 11 | 11 | 0 |
| `derivations` — 1 row | 10 | 7 | 3 (`locus`, two behaviour statements) |
| `validations` — 1 row | 7 | 7 | 0 |
| `terminalization` | 4 | 4 | 0 |
| **total** | **109** | **105** | **4** |

One realistic v2 record, matching the v1 realistic shape (11 changed paths, 11
surfaces — one per path, 3 prior obligations, 1 new finding, 2 derivation
claims, 5 validations, 17 artifacts):

| Block | Leaves | Machine | Human |
|---|---:|---:|---:|
| top-level scalars | 6 | 6 | 0 |
| `subject` | 27 | 27 | 0 |
| `artifacts` — 17 × 6 | 102 | 102 | 0 |
| `path_reviews` — 11 × 6 | 66 | 55 | 11 (`kind`) |
| `prior_obligations` — 3 × 11 | 33 | 33 | 0 |
| `new_findings` — 1 × 8 | 8 | 8 | 0 |
| `derivations` — 2 × 10 | 20 | 14 | 6 |
| `validations` — 5 × 7 | 35 | 35 | 0 |
| `terminalization` | 4 | 4 | 0 |
| **total** | **301** | **284** | **17** |

Comparison, stated as measurement only:

| Shape | v1 (review's measure) | v2 (this design) | Machine-verified share |
|---|---:|---:|---|
| minimal | 76 | 109 (+43 %) | v1 ≈ 30 %, v2 ≈ 96 % |
| realistic | ≈320 | ≈301 (−6 %) | v1 ≈ 30 %, v2 ≈ 94 % |

The v1 machine-verified share is the independent review's own accounting
(8 Git identities, 4 blob digests, coverage set equality, the numstat triple,
control confinement, report parentage/write set and the cross-field rules,
against ~130 explicitly unverified leaves). In v2 the only leaves no machine
checks are one diagnostic `kind` per surface and, per derivation row, the locus
identifier and the two behaviour statements I1 requires. Everything else is
either compared against Git bytes, against the frozen manifest, against the
committed timed result, or derived and emitted rather than authored. There is
no acceptance threshold, no minimum, and no target anywhere in this design.

## 11. Contradiction search

| Pair examined | Result |
|---|---|
| predecessor identity vs packet chronology | Consistent only because the manifest is bound by the packet commit's two-path write set rather than by a digest of itself, and because the prior record is reached by the rooted chain instead of being asserted as a candidate ancestor. Both are implemented that way. |
| packet chronology vs report-only commits | Disjoint write sets: the packet commit writes exactly the two `task-packets/` paths, review control exactly one review-packet path, the report exactly one `swarm-results/` path. A commit satisfying two roles at once is impossible, so role resolution is unambiguous even under the explicit sibling fork. |
| report-only commits vs evidence retention | The record is evaluated uncommitted, so the current generation cannot bind its own retention commit. Retention is verified by the successor generation's authority resolution. This is a real, stated ordering consequence, not a contradiction; documentation carries the sentence (nit 2). |
| evidence retention vs predecessor identity | The successor's manifest must set-equal the predecessor's exported successor-obligation set, which the walk **re-derives from the retained record** and cross-checks against the retained result. A tampered result blob is caught; a tampered record blob breaks the retained digest equality. |
| bootstrap `unavailable` vs `ALLOW` reachability | First generation always blocks; its retained record/result seed generation 2, whose authority is that record and whose root is the pinned legacy commit. No contradiction, at the disclosed cost of two generations. |

## 12. N-3: live arms are not authorized, and what is unavailable now

**Nothing here authorizes a live arm.** No Cargo or KGI manifest is fabricated,
no live manifest path/commit/digest is invented, and no live-arm result is
produced or claimed.

Unavailable now, stated plainly:

- neither arm's repair manifest exists, so no manifest identity can be pinned
  and the packet's Validation(30) has no live subject; the test exercises
  collection thinning against synthetic manifests instead;
- therefore a *rooted but thin* bootstrap manifest remains possible and only
  independent human preflight bars it — the gate cannot decide manifest
  completeness, and this design does not pretend otherwise;
- no canonical port branch exists, so no ported candidate identity, patch
  digest, timed result, or clean-checkout evidence exists;
- Rust predecessor builds remain unavailable, so no Rust arm can reach v2
  `ALLOW`; this is a fail-closed limit, not a validation of I1 on Rust.

**Extension point, designed and implemented as fail-closed.** A future
manager-owned anchor, not written by this lane:

```text
docs/development/ai/review-recurrence/live-arm-anchor.v1.json
schema: codefire.review_recurrence_live_arm_anchor.v1
arms[]: { arm: "cargo" | "kgi_kpi",
          generation: <int>,
          predecessor_authority_commit: <sha1>,
          repair_packet:   { commit, path, sha256, mode: "100644" },
          repair_manifest: { commit, path, sha256, mode: "100644" } }
```

Consumption: `check_review_recurrence.py --live-arm <arm>` loads the anchor
through `recurrence_v2_evidence.load_live_arm_anchor` and requires the record's
`repair_packet` and `repair_manifest` artifacts and its predecessor-authority
commit to equal that arm's pinned row exactly. The anchor is absent today, so
the flag can only produce `BLOCK` with `live-arm-anchor-unavailable`; without
the flag the result emits `live_arm_pin: "unavailable"` and makes no live-arm
claim. A separate manager task must pin the anchor before any live-arm
dispatch.

**Second-generation repair surface, analysed rather than invented.** Both
pinned bootstrap roots are `REQUEST_CHANGES` reports with concrete residual
obligations (Cargo: `SHOULD-2` not closed plus NIT-1/3/4/5 named as one-to-four
line changes in files that commit already touches; KGI/KPI: eight named owner
rows plus five required acceptance items). So a legitimate non-empty repair
surface plausibly exists for each arm's second generation. It is not
*establishable* yet, and the sequencing blocker is structural, in this order:

1. the bootstrap generation cannot `ALLOW` by construction, so its purpose is
   to retain a machine-readable record — this is a cost, not a defect;
2. the second generation's manifest cannot be authored until the bootstrap
   generation's record/result are retained, because its three collections must
   set-equal the bootstrap record's exported successor-obligation set;
3. the second generation's `repair_surface` must be non-empty and must consist
   of real Cargo/KGI work, which is outside this lane entirely; if the
   bootstrap generation left nothing genuinely open, the correct outcome is to
   report that no second generation is due — **not** to manufacture a byte
   change so the topology passes.

I therefore report a sequencing blocker for both live arms and invent no repair.

## 13. Documentation plan

| Path | Change |
|---|---|
| `docs/development/ai/schemas/README.md` | three new rows; v1 recurrence-control row relabelled rejected/deprecated; v2 labelled experimental |
| `docs/development/ai/review-recurrence/README.md` | v2 mechanism, exact chronology and namespaces, roles, derived outcomes, v1 rejected, retention-verified-only-by-successor sentence, provenance limits, live-arm blocker |
| `docs/development/ai/operating-protocol.md` | v2 sequence and command; v1 no longer authorizes |
| `docs/development/ai/validation-report-template.md` | v2 addendum replacing the v1 tables that encode removed fields |
| `docs/development/ai/acceleration-master-plan.md` | AIA-047 status: v1 rejected, v2 experimental, live arms not authorized |

## 14. Commit plan

1. this design report only;
2. v2 schemas plus `recurrence_v2_model.py` (isolated parse/domain primitives)
   and the `check_ai_contracts.py` schema registration;
3. `recurrence_v2_git.py`, `recurrence_v2_topology.py`,
   `recurrence_v2_authority.py`, `recurrence_v2_evidence.py`,
   `recurrence_v2_decision.py`, the `check_review_recurrence.py` v2 entry with
   v1 structured `BLOCK`, and `run_timed_gate.py --emit v2`;
4. adversarial tests and fixtures;
5. documentation and the handoff result report.

Commits 2 and 3 may merge if splitting them would leave a broken intermediate
state. No unrelated change is included in any commit.

## 15. Explicitly out of scope in this lane

No Cargo, KGI/KPI, Upload, Storage, Rust, Python product, provider-routing,
credential, install or release byte is ported, repaired, reviewed or
integrated. `ai_contracts.py` and every other shared helper is unmodified. v1's
schema, validator function, bytes and historical meaning are preserved, and v1
does not authorize through the normal gate.
