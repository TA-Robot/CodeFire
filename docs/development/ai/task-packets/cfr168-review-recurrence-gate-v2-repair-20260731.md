# CFR-168 Review Recurrence Gate v2 Repair

Status: pre-implementation contract draft; implementation is forbidden until
the manager records the independent contract-preflight disposition and freezes
the dispatch revision.

Provider/model for implementation: Claude Code / `claude-opus-5`.

## Objective

Replace the rejected AIA-047 v1 authorization surface with a smaller,
evidence-bound v2 pilot gate that:

1. cannot narrow the reviewed repair interval past the predecessor candidate;
2. proves that the repair packet existed before the first repair change;
3. binds validation, report, route-authority, and terminal evidence to exact
   regular-file Git blobs;
4. forces every changed path to have a reviewed surface or an explicit
   evidence-backed `no_surface_change` disposition;
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

## 2. Evidence that governs this repair

Read these exact repository artifacts before changing anything:

- rejected implementation:
  - candidate `1cae0225e488ff918389700a7de0f94e9fd067b3`;
  - base `a22fad0698b1623d8898c47699e7ba4e61294b5f`;
  - candidate tree `9c0a46a2c27ed3d54f3e1aa475df8ab655caad3a`;
- independent adversarial review:
  - `docs/development/ai/swarm-results/cfr168-review-recurrence-gate-independent-review-claude-20260731/review.md`;
  - verdict `REQUEST_CHANGES`;
  - 51 executable counterexamples, comprising 39 expected `BLOCK` results
    and 12 expected false `ALLOW` results, with no probe surprises;
- root-cause audit:
  - `docs/development/ai/swarm-results/cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md`;
- preregistered historical replay:
  - packet
    `docs/development/ai/swarm-batches/cfr168-review-intervention-retroactive-pilot-20260731.md`;
  - result
    `docs/development/ai/swarm-results/cfr168-review-intervention-retroactive-pilot-claude-20260731/pilot.md`;
  - result SHA-256
    `527b800cd29384ff5f2a1e213e27226eb794e10d247329a088d85679994ab553`.

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

## 3. Compatibility and authority boundary

- Add a new schema identifier and file:
  `codefire.review_recurrence_control.v2` /
  `docs/development/ai/schemas/review-recurrence-control.v2.schema.json`.
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

## 4. Exact Git chronology

### 4.1 Candidate interval

Do not retain an independently author-chosen `base_revision`.

The subject must identify:

- `predecessor_candidate_revision`: exact candidate reviewed in generation
  `N-1`;
- `candidate_revision`: exact candidate for generation `N`.

The tool must:

1. resolve both to canonical commits;
2. require the predecessor to be an ancestor of the candidate;
3. compute tree, patch, numstat, binary paths, changed paths, and every repair
   coverage obligation over exactly
   `predecessor_candidate_revision..candidate_revision`;
4. provide no second field that can silently narrow this interval.

### 4.2 Repair packet precedes repair

The v2 pilot supports only an on-ancestry, Git-verifiable repair packet.
Remove `dispatch_event_attested` from the v2 `ALLOW` surface. A branch-external
or timestamp-only assertion must produce `BLOCK`.

The tool must verify all of the following:

1. the repair packet is a mode `100644` blob in the task-packet namespace;
2. its declared SHA-256 equals the exact blob bytes;
3. its revision lies on candidate ancestry;
4. if the packet revision is at or before the predecessor, the same packet
   blob is present at the predecessor;
5. if the packet revision is after the predecessor, every commit through and
   including that packet revision changes only the exact repair packet path or
   explicitly enumerated manager-control paths;
6. the first candidate/product/test change occurs strictly after the packet
   revision;
7. a packet committed together with a repair byte fails.

The implementation may express this with an explicit
`first_repair_commit`/`repair_start_parent` identity or derive it from the
per-commit write sets. Whichever representation is chosen must be
machine-checked and covered by counterexamples.

### 4.3 Review and evidence ordering

Use one explicit, testable order:

```text
predecessor
  -> repair packet / repair commits
  -> candidate
  -> review-control commit
  -> report-only commit
  -> evidence commit
  -> manager record
```

- The review-control commit must be a direct child of the candidate and change
  exactly the review packet path.
- The report commit must be a direct child of review control and change exactly
  the report path.
- The evidence commit must be a direct child of the report and change exactly
  the evidence paths declared by the record.
- The manager record is evaluated after the evidence commit. It need not be
  present in an earlier commit to authorize its own creation.
- Repair packet, review packet, report, timed-gate result, timed-gate log,
  route authority, terminal event evidence, and any other authority-bearing
  artifact must be mode `100644`; symlinks and other modes fail.
- Do not allow review control to rewrite the gate, schema, shared contract
  helpers, tests, or other product/development bytes.
- Direct-parent rules are preferred to net-diff assertions. If any multi-commit
  interval remains, inspect every commit write set so mutate/revert history
  cannot disappear.

## 5. Evidence-bound validations

Every required validation must reference:

- a committed `codefire.timed_gate.v1` result JSON blob;
- the committed log blob for that result;
- exact result/log paths and SHA-256 digests;
- expected argv or a stable gate identifier plus argv;
- expected exit code and status;
- terminal/timeout state.

The tool must load the exact Git blobs at the evidence commit and require:

- result schema is `codefire.timed_gate.v1`;
- the result blob digest matches;
- its `status`, `exit_code`, `timed_out`, and `log.complete` agree with the
  recurrence record;
- its log digest and byte count equal the committed log blob;
- required passing validations have exit code zero, `status: passed`,
  `timed_out: false`, and complete logs;
- a fabricated digest string, missing blob, mutable worktree-only file,
  symlink, edited timed result, or edited log fails closed.

Do not claim that this proves a command executed honestly. It proves that the
manager record, timed result, and retained bytes agree. Document that boundary
exactly.

All other evidence and authority references must be structured Git artifact
references, not non-empty strings. At minimum bind exact revision, path,
SHA-256, and regular-file mode. The implementation may reuse one common
artifact-reference definition.

## 6. I1 live experiment: predecessor derivation

The historical replay showed that the decisive cases can be byte-identical
between predecessor and candidate. A diff-only check therefore does not
discharge I1.

For each reviewed repair surface whose locus existed in the predecessor, the
record must bind a reviewer-authored predecessor derivation:

- exact path and stable symbol/locus identifier;
- whether the locus existed in the predecessor tree;
- predecessor behaviour derived from that tree;
- candidate behaviour asserted for the same locus;
- exact evidence references or a recorded `unavailable` state;
- disposition/next action.

For a genuinely new locus, the record still states
`locus_existed_in_predecessor: false`; the tool must verify path existence
where Git can decide it and must not pretend it can mechanically verify a
symbol claim.

An `accept` record fails when:

- a reviewed existing locus has no predecessor derivation;
- an asserted predecessor path contradicts the predecessor tree;
- the derivation evidence is unavailable;
- a prior obligation or changed surface lacks the required derivation
  disposition.

Do not require a synthetic byte difference. Do not claim that prose semantics
were machine-proved. The live pilot must emit I1 coverage and unavailable
counts derived from rows.

## 7. I5 live experiment: evidence state vs next action

Do not overload a free `blocking` boolean. Use separate, enumerated fields for:

1. what the reviewer established (`evidence_state`);
2. what work should happen next (`next_action` or `routing_state`);
3. whether the item currently blocks integration, derived by the validator
   from state, closure, and exact authority rather than copied from an
   unchecked boolean.

The evidence vocabulary must represent at least:

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
- an exact closed/no-defect state where needed for prior obligations.

The next-action vocabulary must distinguish at least:

- product repair;
- evidence/reversal execution;
- oracle repair;
- claim/document repair;
- manager scope/requirement decision;
- no further action after proven closure.

Required authority rules:

- `DEFECT_REPRODUCED` and `DEFECT_SOURCE_PROVEN` block integration while open.
- `DEFECT_INJECTION_ONLY` blocks unless exact, independently reviewable
  reachability evidence closes it; a prose assertion is insufficient.
- `CLOSURE_UNPROVEN` is not a product defect, but never authorizes integration;
  it routes to executing the missing closure evidence.
- `ORACLE_VACUOUS` routes to oracle repair, not product repair, and remains
  blocking until the oracle is closed.
- `CLAIM_FALSE` routes to the exact claim repair and remains blocking until it
  is closed.
- `REQUIREMENT_UNSATISFIABLE` routes to a manager criterion decision and
  remains blocking until that exact decision is committed.
- `SCOPE_BOUNDARY` may be routed only through an exact committed manager
  authority artifact with owner and release condition.
- Do not add `ACCEPT_WITH_NOTES`.
- Classification and routing may reduce unnecessary *product-repair
  generations*; they must not reduce integration evidence.

The result must emit counts by evidence state and next action so the Cargo and
KGI live experiments can measure whether routing changes generations. Authors
must not transcribe those counts into the input.

## 8. Obligations, recurrence, and changed-path coverage

### 8.1 Prior obligations

Retain only fields needed to establish identity, generation, continuity,
before/after evidence, negative-control state where required, evidence state,
next action, owner/release/authority where routed, and closure.

- `recurred_after_proven_closure` must bind exact prior closure evidence and
  block while open.
- `unavailable` must constrain the associated observations consistently and
  cannot ride an `accept`.
- Move unverified closure/oracle prose to the repair packet or review report
  unless the tool consumes it.

### 8.2 Changed paths and surfaces

The declared changed-path set must equal Git exactly.

Each changed path must have exactly one path-level disposition:

- `reviewed`, with one or more linked surface rows for that same path; or
- `no_surface_change`, with a non-empty justification and exact evidence.

Every surface row belongs to exactly one changed path. A single surface may not
cover unrelated paths. Public API, state machine, classifier, persisted shape,
and other authority-relevant surface kinds require an exact scope
disposition. A self-selected kind may not bypass that disposition; add
cross-checkable signals where practical and state the human-judgement boundary
where not.

Remove `out_of_scope` if it adds no checked guarantee. If retained, it must be
disjoint from changed/reviewed paths.

### 8.3 No quota and no self-attested saturation

- No minimum, target, or proxy target for findings, observations, surfaces,
  obligations, or checklist rows.
- Remove author-transcribed `outcomes`; derive them in output.
- Remove unchecked “all done” stopping booleans.
- Acceptance is derived from exact set coverage and terminal dispositions.

## 9. Mandatory independent-review repairs

The implementation and its tests must explicitly disposition every finding
from the independent report:

| Finding | Required disposition |
|---|---|
| F-1 base narrowing | Remove free base; exact predecessor interval. |
| F-2 post-repair packet | Verify packet precedes first repair change. |
| F-3 fake dispatch event | No dispatch-attested v2 `ALLOW`. |
| F-4 fake evidence/log digests | Exact committed result/log/artifact binding. |
| F-5 dead severity/blocking | Derive blocking; exact routing authority. |
| F-6 dead continuity | Enforce recurrence-after-proven-closure. |
| F-7 one surface covers many paths | Per-path surface or evidenced no-surface row. |
| F-8 control rewrites gate | Exact direct-child review-packet-only control. |
| F-9 impossible/undocumented ordering | Explicit report/evidence/record sequence. |
| F-10 invalid UTF-8 Git path | Structured fail-closed finding, not a codec traceback. |
| F-11 symlink report/packets | Require mode `100644` for all authority artifacts. |
| F-12 mutate/revert control chain | Direct child or per-commit write-set verification. |
| F-13 contradictory unavailable/out-of-scope | Enforce consistency or remove field. |
| F-14 self-referential patch fixture | Pin a known byte-level patch digest fixture. |

No finding may be marked “fixed” only because its field was renamed.

## Scope

Expected implementation surfaces:

- `docs/development/ai/schemas/review-recurrence-control.v2.schema.json`;
- `docs/development/tools/check_review_recurrence.py`;
- `docs/development/tools/test_ai_tools.py`;
- `docs/development/ai/schemas/README.md`;
- `docs/development/ai/review-recurrence/README.md`;
- `docs/development/ai/operating-protocol.md`;
- `docs/development/ai/validation-report-template.md`;
- `docs/development/ai/acceleration-master-plan.md`;
- directly related fixtures under `docs/development/ai/fixtures/` if needed.

The v1 schema may be documented as rejected/deprecated but its bytes remain
unchanged.

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

Start by turning the independent report's bypasses into failing tests. At
minimum, demonstrate old-v1 false behaviour or an equivalent fixture, then
show v2 blocks:

1. base moved forward past a risky commit;
2. repair packet committed at/after the repair;
3. branch-external fake dispatch digest;
4. fabricated timed result or log digest;
5. block/high defect routed with a free boolean or string authority;
6. recurrence-after-proven-closure without closure evidence;
7. two changed paths with only one path's surface;
8. review control that rewrites the gate/schema;
9. missing or out-of-order evidence commit;
10. invalid-UTF-8 Git path;
11. symlink packet/report/evidence;
12. mutate/revert intermediate control history;
13. contradictory unavailable/out-of-scope state;
14. drift in canonical patch flags caught by a pinned digest;
15. byte-identical predecessor/candidate locus lacking an I1 derivation;
16. source-proven-but-unexecuted evidence state represented without
    misclassifying it as reproduced;
17. `CLOSURE_UNPROVEN` cannot produce `ALLOW`;
18. author-transcribed outcome/count fields are rejected;
19. no finding-count target or proxy appears in schema, tool, or tests.

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

## 12. Pre-implementation self-review

Before editing implementation bytes, return a short written design table that
maps:

- each F-1 through F-14 finding to one mechanism and one counterexample;
- I1 and I5 to their exact schema rows, semantic rules, and emitted metrics;
- every input field to either a validator consumer or a stated
  human-judgement purpose;
- every authority-bearing artifact to its chronology, mode, digest, and
  reader;
- all fields removed from v1 and why.

Look specifically for contradictions between predecessor identity, packet
chronology, report-only commits, and evidence retention. Resolve them before
coding; do not defer them to the final independent review.

## Handoff

Return:

1. exact base revision, candidate revision/tree, and canonical patch SHA-256;
2. changed paths and per-path purpose;
3. the pre-implementation design table;
4. F-1 through F-14 disposition table with test names;
5. I1/I5 live-experiment fields and derived metrics;
6. minimal and one realistic v2 record required-leaf counts, reported as
   measurements only, with no target or acceptance threshold;
7. exact validation commands, exit codes, and retained log/result paths;
8. known human-judgement and provenance limits;
9. clean/dirty worktree status;
10. one commit containing only this repair.

Do not claim the review bottleneck is solved. The candidate must pass a separate
Claude Opus 5 adversarial review and then the Cargo/KGI live comparison before
the manager can decide permanent adoption.
