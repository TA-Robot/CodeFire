# CFR-168 Review Recurrence Gate v2 Repair

Status: contract revision after independent rereview `e837404`; implementation
remains forbidden until the same contract reviewer accepts the exact revised
packet and the manager freezes its manifest/dispatch revision.

Provider/model for implementation: Claude Code / `claude-opus-5`.

## Objective

Replace the rejected AIA-047 v1 authorization surface with a smaller,
evidence-bound v2 pilot gate that:

1. cannot narrow the reviewed repair interval past the exact candidate bound by
   the predecessor generation's committed authority artifact;
2. proves that, in the presented Git topology, the repair packet/manifest
   commit is the direct parent of the first repair commit and carries no repair
   byte;
3. binds validation, report, routing/scope-decision, and terminal evidence to exact
   regular-file Git blobs;
4. forces every changed path to have at least one reviewed surface with an
   explicit scope disposition;
5. runs the two interventions supported by the historical replay as live
   experiments:
   - I1: predecessor-behaviour **derivation**, not merely a differential;
   - I5: evidence state and routing/next-action state recorded separately;
6. preserves the count-free rule and emits diagnostic outcomes instead of
   requiring authors to transcribe them;
7. remains an integration gate: a valid record, a routing decision, or
   `CLOSURE_UNPROVEN` never becomes `ACCEPT_WITH_NOTES`.

This task repairs the review-control mechanism. It does not review, repair, or
integrate the Cargo, KGI/KPI, Upload, Storage, or product candidates.

## Evidence that governs this repair

Read these exact repository artifacts before changing anything:

- rejected implementation:
  - candidate `1cae0225e488ff918389700a7de0f94e9fd067b3`;
  - base `a22fad0698b1623d8898c47699e7ba4e61294b5f`;
  - candidate tree `9c0a46a2c27ed3d54f3e1aa475df8ab655caad3a`;
- independent adversarial review:
  - `docs/development/ai/swarm-results/cfr168-review-recurrence-gate-independent-review-claude-20260731/review.md`;
  - canonical commit `2bec22419c25882a81d029046ccf4cd7ba6d8df0`,
    mode `100644`, SHA-256
    `02bc2030728fd2c471237dd1a5da8eca268c07836c7d5c494bb73b2050ffb05b`;
  - verdict `REQUEST_CHANGES`;
  - the report's exact probe tally is 39 `BLOCK` as expected and 12 `ALLOW`
    as expected, with `unexpected=0`; only the bypasses the report identifies
    by finding/reproducer are treated as false `ALLOW` cases;
- root-cause audit:
  - `docs/development/ai/swarm-results/cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md`;
  - report commit `abfee6ef25f27af35790138404ce6f144b3244da`,
    mode `100644`, SHA-256
    `2392450e957c8437b323ded492fc7597dc77f5147b74f3056c97c0efd9329780`;
- preregistered historical replay:
  - packet
    `docs/development/ai/swarm-batches/cfr168-review-intervention-retroactive-pilot-20260731.md`;
  - result
    `docs/development/ai/swarm-results/cfr168-review-intervention-retroactive-pilot-claude-20260731/pilot.md`;
  - canonical commit `94f2ab4b67a6c679434e72fb73d9ec9ffee6b1ba`,
    mode `100644`;
  - result SHA-256
    `527b800cd29384ff5f2a1e213e27226eb794e10d247329a088d85679994ab553`.
- independent contract preflight:
  - `docs/development/ai/swarm-results/cfr168-review-recurrence-v2-contract-preflight-claude-20260731/review.md`;
  - canonical commit `102fc1232b9a5140c72be94b8add51b5a58c5634`,
    mode `100644`, SHA-256
    `ccd5585c79acb19e7c412b336e40930dd984ea4e26217486f659a1132f6bec0b`;
  - verdict `REQUEST_CHANGES`; C-1 through C-16 are dispositioned in
    **Contract-preflight corrections**.
- contract rereview:
  - `docs/development/ai/swarm-results/cfr168-review-recurrence-v2-contract-preflight-claude-20260731/rereview.md`;
  - canonical commit `e8374040ed177f915b3c3debed20cd4ffa21b17a`,
    mode `100644`, SHA-256
    `af5dab388cd2926da22ae77914a26605e067d5e884dae350f263123ad16e3dfb`;
  - verdict `REQUEST_CHANGES`; R-1 through R-10 are dispositioned below.

The replay decisions are inputs, not optional suggestions:

| Intervention | Decision | Consequence here |
|---|---|---|
| I1 predecessor-behaviour re-derivation | `PROMISING_FOR_LIVE_EXPERIMENT` | Implement a bounded live-experiment obligation. Bind derivation from the predecessor tree. A byte differential is insufficient. |
| I5 evidence state vs routing state | `PROMISING_FOR_LIVE_EXPERIMENT` | Record the two states independently without changing integration authority. |
| I3 bounded repair delta | `REVISE_AND_REPLAY` | Do not add a line, file, surface, or repair-size limit. Preserve measured delta as output only. |
| I4 oracle rules E1–E9 | `REVISE_AND_REPLAY` | Do not install E1–E9 as a mechanical checklist. Bind concrete evidence without claiming this enumeration is complete. |

The replay did **not** justify a review-item count, checklist length, template
depth, field count, or repair-delta threshold. None may be invented in this
repair.

## Compatibility and authority boundary

- Add a new schema identifier and file:
  `codefire.review_recurrence_control.v2` /
  `docs/development/ai/schemas/review-recurrence-control.v2.schema.json`.
- Add a compact pre-repair manifest:
  `codefire.review_recurrence_repair_manifest.v1` /
  `docs/development/ai/schemas/review-recurrence-repair-manifest.v1.schema.json`.
  It is the machine-readable anchor for predecessor authority, prior-obligation
  IDs, parity/closure claims, and required validation IDs/argv; prose remains
  in the Markdown repair packet.
- Extend the timed runner with a new, compatibility-distinct
  `codefire.timed_gate.v2` result that binds canonical source revision and tree.
  Keep v1 evidence readable as historical data, but v1 timed results cannot
  satisfy a v2 recurrence `accept`.
- Register all new schema `$id` values in
  `docs/development/tools/check_ai_contracts.py`; its only permitted change is
  the corresponding `SCHEMAS` entries and tests required for them.
- Keep the v1 schema as historical evidence. Do not silently widen or rewrite
  its shape.
- The normal gate entry point must authorize only v2. A v1 record must return a
  structured `BLOCK`; it must never retain the rejected v1 `ALLOW` path.
- A v2 result must use a distinct result schema identifier.
- The documentation must label v1 rejected/deprecated and v2 experimental.
- Neither schema validity nor record validity is integration authority.
  `integration_decision: ALLOW` is the sole machine authorization.
- `--no-git-verify`, missing evidence, unavailable required evidence,
  nonterminal execution, and every non-`accept` review decision remain
  `BLOCK`.

## Exact Git chronology

### Predecessor authority and candidate interval

There is no independently author-chosen `base_revision`. The subject identifies
`predecessor_candidate_revision`, `candidate_revision`, and one exact
`predecessor_authority` artifact.

The authority is one of:

1. a committed v2 record whose bound `candidate_revision` is the predecessor
   and whose generation is exactly `N-1`; or
2. for the bootstrap live runs only, a legacy report-only commit whose sole
   parent is the predecessor, whose write set is exactly the report path, and
   whose report blob matches the declared mode and SHA-256.

The tool resolves the authority before accepting the predecessor field and
requires exact equality. It then computes tree, patch, numstat, binary paths,
changed paths, and all coverage over exactly
`predecessor_candidate_revision..candidate_revision`. A later ancestor cannot
be substituted merely because it is on candidate ancestry.

A prior v2 record is not trusted as a standalone JSON blob. The named machine
reader is the `resolve_predecessor_authority_chain` path in
`check_review_recurrence.py`. It walks the record's `predecessor_authority`
chain transitively until one of the manager-fixed legacy roots below. At every
link it verifies generation continuity, candidate identity, packet/candidate/
control/report/evidence/retention topology, exact per-commit write sets, modes,
blob digests, record/result digest equality, and the exported
successor-obligation set. In particular, it verifies that the retention commit
is the direct child of the evidence commit and changes exactly the declared
record and result paths. Cycles, missing links, a discontinuous generation, or
a chain that does not terminate at a manager-fixed root produce `BLOCK`. Ref
existence is never a substitute for this walk.

The two allowed bootstrap bindings are manager-fixed here:

| Arm | Predecessor candidate | Legacy report commit | Report path | SHA-256 |
|---|---|---|---|---|
| Cargo | `4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912` | `130878d321f6be166f554ac2c68a59e2b36afa88` | `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-second-closure-rereview-claude-20260731/review.md` | `aad067b2b8feb80830c41db1608dd31f0e7e7d7ca1f37ab01d040336f788b4fd` |
| KGI/KPI | `753c8db103579d021795d3fe6a97c5d45d4a111a` | `12f9d0b5c30ce2695ec6c96e5876703efd7b0d90` | `docs/development/ai/swarm-results/development-flow-kgi-kpi-r3-rereview-claude-20260731/review.md` | `23b59e01825e3f66d2d38b754348e3d51457f3c465def578ac99ce52729a204f` |

Both report blobs must be mode `100644`. A bootstrap generation always records
`predecessor_obligation_completeness: unavailable` and therefore cannot
produce machine `ALLOW`; its retained v2 record seeds the second generation,
where `ALLOW` first becomes reachable. No other legacy binding may serve as a
root at all without a new manager contract and independent preflight.

### Repair packet and manifest precede the presented repair

The v2 `ALLOW` surface supports one topology only:

```text
predecessor candidate
  -> packet commit: exactly {Markdown repair packet, JSON repair manifest}
  -> one or more repair commits
  -> candidate
```

Steady state is an explicit fork. The predecessor candidate has one child on
its completed review chain and a sibling packet commit for the successor
repair. The manifest may therefore embed obligation data emitted later on the
sibling review chain; the prior record is verified through the rooted
authority-chain walk in **Predecessor authority and candidate interval**, not
by pretending it is a candidate ancestor.
The topology proves graph relationships only, not when the manifest was
authored.

The packet commit is the direct child of the predecessor. The two packet paths
are mode `100644` and are in `docs/development/ai/task-packets/`. The manifest
binds the exact Markdown packet digest; the direct-child two-path write set
binds the manifest to the same packet commit without a circular digest. The
admissible manager-control path set in this pre-repair window is therefore
exactly those two paths; it is not
`CONTROL_PATH_PREFIXES`, and it contains no candidate repair path.

The manifest is compact and `additionalProperties: false`. It contains exactly
the schema/manifest identity, generation, predecessor authority reference,
predecessor candidate revision, Markdown packet path/digest,
`finding_target_policy: "none"`, unique prior-obligation rows mapping each ID
to its required closure-validation IDs, unique parity/closure claim rows, and
unique required validation IDs with argv. Every referenced closure-validation
ID must exist in the required validation set. Except for an obligation whose
predecessor state is `DEFECT_INJECTION_ONLY`, every prior-obligation row has at
least one closure-validation ID. An injection-only row may have an empty list
only because a later `CLOSED_PROVEN` state must instead bind the independently
accepted non-reachability artifact in **I5 live experiment: evidence state vs
next action**. The recurrence record must be
set-equal to those three manifest collections; the manifest does not contain
author-transcribed outcomes or semantic closure prose.

The manifest is the remaining discretion boundary. Its parity/closure claims
and required validations are manager-reviewed and content-frozen with the
dispatch revision before an implementer starts. Set equality prevents later
weakening but cannot prove that the manager chose a complete set; independent
packet preflight judges that completeness without a count target.

For the two canonical ports, the Markdown packet is fixed exactly:

| Arm | Packet path | Source commit | SHA-256 | Predecessor state |
|---|---|---|---|---|
| Cargo | `docs/development/ai/task-packets/cfr168-source-keyed-cargo-third-closure-repair-20260731.md` | `8c51b712f1e3937fd462a95c70b8de79687b9289` | `7464f158cc04373af2cb5e0eacb37212233d920dc9233bf220e553361a43adc7` | absent at `4e5c0e7f` |
| KGI/KPI | `docs/development/ai/task-packets/cfr168-development-flow-kgi-kpi-r4-repair-20260731.md` | `a22fad0698b1623d8898c47699e7ba4e61294b5f` | `bbd6ef472903079238f24ad92bf567a9143fdbd1874975d5364ddd4eea8964ad` | absent at `753c8db1` |

Both source blobs are mode `100644`. The port copies their bytes to the same
paths; the new manifest is the second path. KGI/KPI's historical
`swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r3-20260731.md`
and `...-r4-20260731.md` remain byte-for-byte in the ported repair patch and
are ordinary reviewed repair surfaces, not precedence packets. The Cargo
candidate's own repair/result artifacts likewise remain repair surfaces.
The **Repair packet and manifest precede the presented repair** proof is scoped
to the one declared Markdown packet and its manifest; every other instruction
artifact remains visible through exact changed-path/surface coverage.

The Git-derived `repair_surface` is:

```text
changed_paths(predecessor, candidate)
  - {repair_packet_path, repair_manifest_path}
```

It must be non-empty. Every commit after the packet commit through the
candidate is inspected separately; it may not rewrite either packet artifact,
and its paths must be within the derived repair surface. This definition
applies equally to product, test, schema, tool, and documentation repairs.
There is no product/control taxonomy and no unspecified manager-control prefix.

`dispatch_event_attested`, branch-external packets, timestamp-only assertions,
and a packet committed with a repair byte cannot produce v2 `ALLOW`.

This proves topology in the history presented to the gate: the packet commit
is an ancestor and direct predecessor of the repair series. Git does not prove
when text was authored and cannot detect a rewritten history that presents a
different topology. Documentation and results must state that boundary.

The existing Cargo `74cd37d5` and KGI/KPI `9ca01d9a` candidates remain
immutable and off-lineage from their packets. After v2 is accepted, each live
arm requires a transparent canonical port: branch from the manager-fixed
predecessor above, commit the byte-identical historical repair packet plus a
new manifest, apply the already-reviewed repair patch without other byte
changes, and recompute all candidate/review identities. The port attests its
presented topology; it must not be described as proof of the original lane's
authorship time.

### Review, evidence, record, and result ordering

Use this exact order:

```text
candidate
  -> review-control commit
  -> report-only commit
  -> evidence commit
  -> evaluate uncommitted record and result
  -> record/result retention commit
```

- Review control is a direct child of the candidate and changes exactly one
  mode-`100644` review-packet path.
- The report is a direct child of review control and changes exactly one
  mode-`100644` report path.
- Evidence is a direct child of the report and changes exactly the
  mode-`100644` artifact-reference paths declared by the record under
  `docs/development/ai/review-recurrence/evidence/<control_id>/`.
- At evaluation time, `--record` is deliberately an uncommitted input. The gate
  emits its canonical record SHA-256 in the v2 result.
- Before a manager cites the decision, a direct child of the evidence commit
  must add exactly the byte-identical record and result under
  `docs/development/ai/review-recurrence/records/` and
  `docs/development/ai/review-recurrence/results/`. Both must be mode `100644`;
  the committed record digest must equal the digest emitted by the result.
- Direct-parent relations above are mandatory. Every other multi-commit
  interval, including the repair series, is checked commit by commit; a net
  diff alone is insufficient.

The common artifact-reference definition applies structurally to predecessor
authority, repair packet/manifest, review packet, report, timed result/log,
terminal event evidence, prior-closure evidence, non-reachability evidence,
and routing/scope decision artifacts. Each such referenced blob must be mode
`100644`; symlinks and all other modes fail. Review control cannot rewrite the
gate, schema, helpers, tests, or any path other than its exact packet.

## Evidence-bound validations

Every required validation must reference:

- a committed `codefire.timed_gate.v2` result JSON blob;
- the committed log blob for that result;
- exact result/log paths and SHA-256 digests;
- the stable validation ID and exact argv frozen in the pre-repair manifest;
- expected exit code and status;
- terminal/timeout state.

`run_timed_gate.py` must derive, rather than accept as an unchecked string, the
Git root containing its cwd, canonical `HEAD` revision, and `HEAD^{tree}` and
emit them in v2. Required acceptance validations run against a clean checkout
of the exact candidate; the recurrence tool requires the timed result's
revision/tree to equal the bound candidate.

The runner captures `git status --porcelain=v1 -z --untracked-files=all`
immediately before and after the command, emits both cleanliness states, and
v2 acceptance requires both empty. Timed logs/results are first written
outside the source worktree and later copied byte-identically into the evidence
commit, so they do not make the checkout dirty. This rejects a result from a
different revision/tree or an ordinarily dirty checkout. A command that mutates
and restores bytes during its own execution remains outside the provenance
proof and is stated as such.

The generic runner also does not prove that a test selector chose a nonzero
corpus; validation-specific selection claims remain report evidence unless a
dedicated parser verifies them, and v2 must not claim otherwise.

The tool must load the exact Git blobs at the evidence commit and require:

- result schema is `codefire.timed_gate.v2`;
- the result blob digest matches;
- its source revision/tree equal the candidate;
- validation IDs and argv are set-equal to the required validations in the
  Git-bound repair manifest; neither the record nor reviewer may add, omit, or
  weaken a required validation;
- its `status`, `exit_code`, `timed_out`, and `log.complete` agree with the
  recurrence record;
- its log digest and byte count equal the committed log blob;
- required passing validations have exit code zero, `status: passed`,
  `timed_out: false`, and complete logs;
- a fabricated digest string, missing blob, mutable worktree-only file,
  symlink, edited timed result, or edited log fails closed.

`cwd` and `log.path` remain machine-specific absolute paths; they are
diagnostic only. The committed log is linked by its digest and byte count.
Do not claim that the result proves honest execution, semantic oracle quality,
nonzero test selection, or absence of mutate/revert activity inside the
command. It proves that the record, manifest, committed candidate identity,
clean-before/after states, timed result, and retained bytes agree.

All other evidence and authority references must be structured Git artifact
references, not non-empty strings. The common definition binds role, exact
revision, path, SHA-256, and mode `100644`. The validator derives the permitted
revision from the role:

- packet/manifest evidence resolves at the packet commit on candidate ancestry;
- candidate evidence resolves at the candidate;
- review/report/evidence artifacts resolve at their exact
  **Review, evidence, record, and result ordering** commit;
- a prior v2 record resolves at its retained record commit and is accepted only
  by the rooted transitive chain walk in
  **Predecessor authority and candidate interval**;
- the two legacy bootstrap reports resolve only at the exact commits fixed in
  **Predecessor authority and candidate interval**;
- routing or scope-decision artifacts resolve at or before the candidate on
  its ancestry and never inside review control, report, or evidence commits.

There is no global "reachable from any ref" predicate. Contract-pinned
bootstrap artifacts are verified by their exact commit/write set/mode/digest;
packet, candidate, review, report, and evidence roles are verified by their
declared topology; prior records use the rooted chain walk; routing/scope roles
use candidate ancestry. A local branch or ref pointing at an object is never
evidence, and a role-incompatible or unrooted commit fails even when
`git cat-file` can read it.

## I1 live experiment: predecessor derivation

The historical replay showed that the decisive cases can be byte-identical
between predecessor and candidate. A diff-only check therefore does not
discharge I1.

The pre-repair manifest freezes the exact parity/closure claims to which I1
applies. The record's derivation rows must be set-equal to those claim IDs;
surface count does not create additional I1 obligations. For each claim, the
record binds a reviewer-authored predecessor derivation:

- exact path and stable symbol/locus identifier;
- whether the locus existed in the predecessor tree;
- predecessor behaviour derived from that tree;
- candidate behaviour asserted for the same locus;
- exact evidence references or a recorded `unavailable` state;
- disposition and `next_action`.

For a genuinely new locus, the record still states
`locus_existed_in_predecessor: false`; the tool must verify path existence
where Git can decide it and must not pretend it can mechanically verify a
symbol claim.

An `accept` record fails when:

- a manifest parity/closure claim has no predecessor derivation;
- an asserted predecessor path contradicts the predecessor tree;
- the derivation evidence is unavailable;
- a prior obligation that declares a parity/closure claim lacks its derivation
  disposition.

Do not require a synthetic byte difference. Do not claim that prose semantics
were machine-proved. The live pilot emits:

- derivation-row coverage over manifest claims;
- unavailable-row count retained in the denominator;
- changed-surface coverage as a separate diagnostic.

These measure row presence and artifact binding, not derivation quality or
historical capture. `unavailable` remains fail-closed for v2 `ALLOW`. The
Cargo/KGI live arms are Python-side control tooling; no Rust arm may use v2
`ALLOW` until a predecessor build is available. That is the explicit
fail-closed consequence of the replay's five unavailable Rust rows, not an
inference that I1 was validated on Rust.

## I5 live experiment: evidence state vs next action

Do not overload a free `blocking` boolean. Use separate, enumerated fields for:

1. what the reviewer established (`evidence_state`);
2. what work should happen next (`next_action`);
3. whether the item currently blocks integration, derived by the validator
   from those exact enums and proven closure rather than copied from an
   unchecked boolean.

The evidence vocabulary is exactly:

- `DEFECT_REPRODUCED`;
- `DEFECT_SOURCE_PROVEN` for source-proven findings from a lane that executed
  no reproducer (the historical replay proved the earlier vocabulary lacked
  this state);
- `DEFECT_INJECTION_ONLY`;
- `CLOSURE_UNPROVEN`;
- `ORACLE_VACUOUS`;
- `CLAIM_FALSE`;
- `SCOPE_BOUNDARY`;
- `REQUIREMENT_UNSATISFIABLE`;
- `CLOSED_PROVEN`.

The `next_action` vocabulary is exactly:

- `PRODUCT_REPAIR`;
- `RUN_CLOSURE_EVIDENCE`;
- `REPAIR_ORACLE`;
- `REPAIR_CLAIM`;
- `DOCUMENT_NON_REACHABILITY`;
- `MANAGER_SCOPE_DECISION`;
- `MANAGER_REQUIREMENT_DECISION`;
- `NONE`.

Required authority rules:

- `CLOSED_PROVEN` with `next_action: NONE` is the only nonblocking state.
- Every other evidence state blocks machine `ALLOW`; routing changes the kind of
  successor work, never integration authority.
- `CLOSED_PROVEN` is not a free label. A prior obligation may use it only when
  it binds every closure-validation ID frozen for that obligation in the
  manifest, that mapping is non-empty, and every corresponding committed timed
  v2 result passes. An item whose predecessor state is
  `DEFECT_INJECTION_ONLY` may instead use an empty validation mapping only when
  it binds the independently accepted non-reachability boundary below. A new
  finding with no pre-repair manifest mapping cannot be closed in the same
  generation. The tool verifies the identity, set equality, and passing
  results; it does not claim those oracles semantically prove closure.
- `DEFECT_REPRODUCED` and `DEFECT_SOURCE_PROVEN` route to product repair.
- `DEFECT_INJECTION_ONLY` blocks unless an exact, independently reviewable
  **non-reachability** boundary is committed and bound. The validator proves
  artifact identity only, not the semantic non-existence claim; after an
  independent reviewer accepts that boundary, the item is recorded as
  `CLOSED_PROVEN`, not silently de-blocked while retaining the defect state.
- `CLOSURE_UNPROVEN` is not a product defect, but never authorizes integration;
  it routes to executing the missing closure evidence.
- `ORACLE_VACUOUS` routes to oracle repair, not product repair, and remains
  blocking until the oracle is closed.
- `CLAIM_FALSE` routes to the exact claim repair and remains blocking until it
  is closed.
- `REQUIREMENT_UNSATISFIABLE` routes to a manager criterion decision and
  remains blocking until a later reviewed criterion produces
  `CLOSED_PROVEN`.
- `SCOPE_BOUNDARY` routes to a manager decision and remains blocking at this
  machine gate. An owner, release condition, or pre-candidate authority
  artifact is retained for routing evidence only and cannot change the v2
  blocking derivation. Git binds its bytes and ancestry but cannot attest
  human authorship.
- Do not add `ACCEPT_WITH_NOTES`.
- Classification and routing may reduce unnecessary *product-repair
  generations*; they must not reduce integration evidence.

The result emits counts by evidence state and `next_action`, plus a derived
record-level `product_repair_required` boolean. Across the Cargo/KGI live arms,
the manager measures **product-repair generations avoided**: generations whose
denominator includes every reviewed generation, whose numerator has no
`PRODUCT_REPAIR` action, and whose unavailable count is reported separately.
This is deliberately not the replay's `6 / 18` candidate-generation-removal
metric and must not be compared as though it were. The gate preserves
integration evidence even when it avoids sending claim/oracle/scope work back
through a product-repair lane. Authors transcribe none of these outcomes.

## Obligations, recurrence, and changed-path coverage

### Prior obligations

Retain only fields needed to establish identity, generation, continuity,
before/after evidence, negative-control state where required, evidence state,
next action, owner/release/authority where routed, and closure.

- The pre-repair manifest's prior-obligation IDs must be set-equal to the
  predecessor v2 record's emitted `successor_obligation_ids`: every prior
  obligation plus every new finding from that record, including items then
  recorded closed so a later recurrence remains visible. Omission, addition,
  or an extra ID fails. Continuity is derived: an item that was
  `CLOSED_PROVEN` and is nonclosed now is
  `recurred_after_proven_closure`; an item that remained nonclosed is
  `continuous_open`. The author cannot choose a weaker label.
- For the two bootstrap arms, all three manifest collections are manager-derived
  from the exact frozen packets below: prior-obligation rows from their frozen
  row inventories; parity/closure claims from their closure-contract and
  predecessor-failure sections; required validation IDs/argv from their
  validation and independent-negative-control tables.
  The exact packets are:
  - Cargo packet commit
    `0304c0e1ee835336d779e16bd6a4c96096662242`, SHA-256
    `db0d58cc45ae214306f325f1047dfa1783be7aba21ce067b4a8840b252ad999c`;
  - KGI/KPI packet commit
    `8437a8e006465576a288c6db167f4a0c5f5c30e0`, SHA-256
    `4b5e6603ccc759510d6ed7f482c8b6a5f42fa19340742d3feccdd82f38bab83c`.
- Legacy Markdown completeness is not machine-decidable. A bootstrap record
  carries `predecessor_obligation_completeness: unavailable`, retains it in the
  denominator, and cannot produce machine `ALLOW`; it is a live measurement
  and a machine-readable seed for the next generation. This exception is not
  silently treated as completeness.
- The gate verifies the frozen manifest bytes and downstream set equality, not
  the manager's semantic extraction from Markdown. Independent packet preflight
  must approve that extraction before the manifest/dispatch revision is frozen.
- `recurred_after_proven_closure` must bind exact prior closure evidence and
  block while open.
- `unavailable` must constrain the associated observations consistently and
  cannot ride an `accept`.
- Move unverified closure/oracle prose to the repair packet or review report
  unless the tool consumes it.

### Changed paths and surfaces

The declared changed-path set must equal Git exactly.

Every changed path, including packet, test, schema, tool, and documentation
paths, has disposition `reviewed` and one or more linked surface rows for that
same path. There is no `no_surface_change` escape. Every surface row belongs
to exactly one path, and a surface cannot cover another path.

Every surface row, regardless of its self-declared kind, must carry the same
exact scope disposition and evidence fields. `kind` is diagnostic only and
cannot change validation or blocking. Git verifies path coverage and
path-to-surface linkage; complete semantic symbol enumeration remains a human
review judgement and must not be described as machine-proved.

The v2 schema has no `out_of_scope` field; `additionalProperties: false`
rejects it. Scope routing is per surface and cannot contradict exact
changed-path coverage.

### No quota and no self-attested saturation

- The schema requires `finding_target_policy: "none"` and rejects unknown
  properties. It explicitly prohibits
  `target_finding_count`, `minimum_findings`, `expected_findings`,
  `target_observations`, `minimum_surfaces`, and `minimum_obligations`.
  Semantic proxy detection beyond those named fields remains an independent
  review judgement.
- There is no minimum or target for findings, observations, surfaces,
  obligations, or checklist rows. The one-or-more-surface-per-changed-path
  relation is exact Git coverage, not a quota.
- Remove author-transcribed `outcomes`; derive them in output.
- Remove unchecked “all done” stopping booleans.
- Acceptance is derived from exact set coverage and terminal dispositions.

## Mandatory independent-review repairs

The implementation and its tests must explicitly disposition every finding
from the independent report:

| Finding | Required disposition |
|---|---|
| F-1 base narrowing | Resolve predecessor from prior authority; exact interval equality. |
| F-2 post-repair packet | Direct-parent packet/manifest topology; state Git rewrite boundary. |
| F-3 fake dispatch event | No dispatch-attested v2 `ALLOW`. |
| F-4 fake evidence/log digests | Candidate-bound timed v2 result plus exact committed result/log/artifact bytes. |
| F-5 dead severity/blocking | `CLOSED_PROVEN` is the only nonblocking state; routing never authorizes. |
| F-6 dead continuity | Prior-obligation set equality and recurrence-after-proven-closure evidence. |
| F-7 one surface covers many paths | At least one scope-dispositioned surface for every exact changed path; no no-surface escape. |
| F-8 control rewrites gate | Exact direct-child review-packet-only control. |
| F-9 impossible/undocumented ordering | Exact review/report/evidence/evaluate/retention sequence and namespaces. |
| F-10 invalid UTF-8 Git path | Detect before schema validation and JSON-escape offending bytes in a structured finding. |
| F-11 symlink report/packets | Require mode `100644` for all authority artifacts. |
| F-12 mutate/revert control chain | Direct children for control chain; mandatory per-commit inspection for every other interval. |
| F-13 contradictory unavailable/out-of-scope | Enforce consistency or remove field. |
| F-14 self-referential patch fixture | Pin a known byte-level patch digest fixture. |

No finding may be marked “fixed” only because its field was renamed.

## Contract-preflight corrections

The implementation design and tests must preserve these dispositions from
`docs/development/ai/swarm-results/cfr168-review-recurrence-v2-contract-preflight-claude-20260731/review.md`:

| Finding | Revised contract mechanism |
|---|---|
| C-1 | Predecessor comes from a prior v2 record or one of two exact legacy report-only commits; equality is mandatory. |
| C-2 | Packet commit changes exactly two named artifacts; admissible extra manager-control set is empty; repair surface is Git-derived. |
| C-3 | Timed v2 derives candidate revision/tree; validation set comes from the manifest; artifact roles enforce reachability. |
| C-4 | Only `CLOSED_PROVEN`/`NONE` is nonblocking; no routing or review-chain authority can create machine `ALLOW`. |
| C-5 | `no_surface_change` is removed; every changed path has a scope-dispositioned surface, with `kind` diagnostic only. |
| C-6 | Prior-obligation IDs are set-equal to prior v2 authority; legacy completeness remains explicit unavailable and blocks bootstrap `ALLOW`. |
| C-7 | Uncommitted evaluation input is digested; record/result retention commit and three namespaces are explicit. |
| C-8 | Injection-only closure requires an independently accepted non-reachability boundary; the tool attests only its bytes/role. |
| C-9 | I5 measures product-repair generations avoided and is explicitly incomparable to replay `6 / 18`. |
| C-10 | I1 is manifest parity/closure-claim scoped; Rust unavailable is fail-closed; output is row coverage. |
| C-11 | Claims are limited to topology in presented history; rewritten authorship chronology is outside evidence. |
| C-12 | Governing review/audit/replay artifacts have exact revisions, modes, and digests; probe tally is quoted verbatim. |
| C-13 | Timed v2 schema/runner and `check_ai_contracts.py` registration are in scope. |
| C-14 | Invalid Git path bytes are rejected before schema validation and JSON-escaped in the result. |
| C-15 | Direct-parent/per-commit rules, artifact roles, `next_action`, `CLOSED_PROVEN`, and named quota fields are exact. |
| C-16 | v1 before evidence is pinned to `1cae0225`; the full AI development gate remains required. |

## Contract-rereview corrections

| Finding | Revised contract mechanism |
|---|---|
| R-1 | Every prior-v2 authority chain is walked transitively to one of the manager-fixed roots; each topology/write-set/mode/digest/set link is checked. |
| R-2 | `CLOSED_PROVEN` binds all manifest-mapped closure validations, or the independently accepted injection non-reachability boundary. |
| R-3 | Artifact validity is role-specific; pinned commits, topology, rooted chains, and candidate ancestry replace an undefined any-ref reachability rule. |
| R-4 | The sibling review-chain/successor-packet fork is explicit, and off-lineage predecessor records are verified by the rooted role. |
| R-5 | Timed v2 records clean-before/after status; acceptance requires both clean and retains the mutate/revert provenance limit. |
| R-6 | Bootstrap records always block and seed a second generation; no other legacy authority root is admitted. |
| R-7 | Successor predecessor-authority resolution is the named reader of retention parentage, write sets, modes, and record/result digest equality. |
| R-8 | Frozen bootstrap packets seed obligation, parity/closure, and validation collections explicitly. |
| R-9 | The manifest is named as the manager discretion boundary and is independently reviewed/frozen before dispatch. |
| R-10 | Cargo and KGI/KPI port packets are fixed by path/commit/digest; historical in-patch dispatch artifacts stay reviewed repair surfaces. |

## Scope

Expected implementation surfaces:

- `docs/development/ai/schemas/review-recurrence-control.v2.schema.json`;
- `docs/development/ai/schemas/review-recurrence-repair-manifest.v1.schema.json`;
- `docs/development/ai/schemas/timed-gate-result.v2.schema.json`;
- `docs/development/tools/check_review_recurrence.py`;
- `docs/development/tools/run_timed_gate.py`;
- `docs/development/tools/check_ai_contracts.py`, limited to schema registration;
- `docs/development/tools/test_ai_tools.py`;
- `docs/development/ai/schemas/README.md`;
- `docs/development/ai/review-recurrence/README.md`;
- `docs/development/ai/operating-protocol.md`;
- `docs/development/ai/validation-report-template.md`;
- `docs/development/ai/acceleration-master-plan.md`;
- directly related fixtures under `docs/development/ai/fixtures/` if needed.

The recurrence-control v1 schema and historical timed-gate v1 result shape may
be documented as rejected/deprecated inputs but their existing bytes and
meaning remain unchanged.

## Non-Goals

Do not change:

- CodeFire product code, Rust crates, Python CLI, storage, HTTP, Cargo, KGI/KPI
  product/control candidates, install, release, credentials, provider routing,
  or unrelated development docs;
- historical review reports, replay result, prior packets, or recorded verdicts;
- Fugu schedules or provider windows.

If a shared helper change is genuinely necessary, stop and return the exact
reason and affected consumers instead of widening scope silently.

## Validation

Start by turning the independent report's bypasses into failing tests.
Demonstrate v1's before behaviour at exact rejected revision
`1cae0225e488ff918389700a7de0f94e9fd067b3` or in an explicitly labelled
synthetic v1 fixture; deleting v1 authorization at HEAD is not evidence of its
old behaviour. Then show v2 blocks:

1. an interval narrowed to a later ancestor that does not equal the candidate
   resolved from predecessor authority;
2. repair packet committed at/after the repair;
3. branch-external fake dispatch digest;
4. fabricated timed result/log, wrong candidate revision/tree, or omitted
   manifest-required validation;
5. any non-`CLOSED_PROVEN` state, including `SCOPE_BOUNDARY`, routed with a free
   boolean/string/self-authored review-chain authority;
6. omitted prior obligation, relabelled continuity, or recurrence after proven
   closure without closure evidence;
7. two changed paths with only one path's surface, or an attempted
   `no_surface_change` property;
8. review control that rewrites the gate/schema;
9. missing/out-of-order evidence or a record/result retention mismatch;
10. invalid-UTF-8 Git path detected before schema validation and rendered as an
    escaped structured finding;
11. symlink packet/report/evidence;
12. mutate/revert intermediate control history;
13. contradictory unavailable/out-of-scope state;
14. drift in canonical patch flags caught by a pinned digest;
15. byte-identical predecessor/candidate locus lacking an I1 derivation;
16. source-proven-but-unexecuted evidence state represented without
    misclassifying it as reproduced;
17. `CLOSURE_UNPROVEN` cannot produce `ALLOW`;
18. author-transcribed outcome/count fields are rejected;
19. `finding_target_policy` is exactly `none`, named quota fields are rejected,
    and the independent reviewer searches for semantic proxies;
20. a v1 timed result or a v2 result from another tree is rejected;
21. a bootstrap record with unavailable predecessor-obligation completeness
    cannot produce `ALLOW`;
22. an artifact in a dangling or role-incompatible commit is rejected;
23. `DEFECT_INJECTION_ONLY` cannot close on reachability evidence; only a bound,
    independently accepted non-reachability boundary can lead to a later
    `CLOSED_PROVEN` row.
24. a fabricated prior-v2 record whose chain is unrooted, cyclic, skips a
    generation, has a malformed retention commit, or changes its exported
    successor obligations is rejected even if a local ref points at it;
25. `CLOSED_PROVEN` with an empty closure-validation mapping, a missing mapped
    result, or a nonpassing mapped result is rejected, with the sole
    injection-only/non-reachability alternative tested separately;
26. role-specific artifact resolution accepts each exact pinned bootstrap
    binding without depending on local refs, while rejecting an object that
    exists at a role-incompatible commit;
27. a timed v2 acceptance run with nonempty clean-before or clean-after status
    is rejected, and the test names the unproved mutate/revert-during-command
    boundary;
28. the first bootstrap generation blocks by construction, its retained
    authority seeds the next generation, and only that rooted second
    generation can exercise the `ALLOW` path;
29. both canonical-port fixtures bind the exact Cargo/KGI packet
    path/source-commit/digest triples, and KGI's historical in-patch
    `swarm-batches` artifacts remain covered repair surfaces rather than packet
    authority;
30. all three bootstrap manifest collections are set-equal to the independently
    preflighted manager freeze; thinning any one collection is rejected.

Run:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  test_ai_tools.ReviewRecurrenceTests
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest test_ai_tools
python3 docs/development/tools/check_ai_contracts.py
python3 docs/development/tools/check_ai_acceleration_docs.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_development.py
git diff --check
```

The unittest commands run from `docs/development/tools`; the remaining commands
run from the repository root.

## Pre-implementation self-review

Before editing implementation bytes, return a short written design table that
maps:

- each F-1 through F-14 finding to one mechanism and one counterexample;
- each contract-preflight C-1 through C-16 finding to its exact revised clause
  and implementation mechanism;
- each contract-rereview R-1 through R-10 finding to its exact revised clause,
  named machine reader, and adversarial counterexample;
- I1 and I5 to their exact schema rows, semantic rules, and emitted metrics;
- the repair manifest's predecessor, obligation, parity-claim, and validation
  set anchors;
- every input field to either a validator consumer or a stated
  human-judgement purpose;
- every authority-bearing artifact to its chronology, mode, digest, and
  reader;
- all fields removed from v1 and why.

Before implementation, estimate the minimal and one realistic v2 required-leaf
counts, compare them with v1's measured 76-minimal / approximately-320-realistic
shape, and separate machine-verified from human-judgement leaves. These are
measurements with no acceptance threshold; if the design is unexpectedly
larger, stop for manager adjudication rather than hiding the burden until
handoff.

Look specifically for contradictions between predecessor identity, packet
chronology, report-only commits, and evidence retention. Resolve them before
coding; do not defer them to the final independent review.

## Handoff

Return:

1. exact base revision, candidate revision/tree, and canonical patch SHA-256;
2. changed paths and per-path purpose;
3. the pre-implementation design table;
4. F-1 through F-14, C-1 through C-16, and R-1 through R-10 disposition tables
   with test names;
5. I1/I5 live-experiment fields and derived metrics;
6. minimal and one realistic v2 record required-leaf counts, reported as
   measurements only, with no target or acceptance threshold;
7. exact validation commands, exit codes, and retained log/result paths;
8. known human-judgement and provenance limits;
9. clean/dirty worktree status;
10. small, bounded commits separating schema/tool, tests, and documentation,
    with no unrelated change and a clean final worktree.

Do not claim the review bottleneck is solved. The candidate must pass a separate
Claude Opus 5 adversarial review and then the Cargo/KGI live comparison before
the manager can decide permanent adoption.
