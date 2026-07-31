# AI Validation Report Template

This is a handoff shape, not passing evidence. Replace every placeholder; an omitted, skipped, or capability-blocked command must not be reported as passed.

```text
Task packet ID:
Base revision:
Head revision:
Changed-file digest:
Environment/capability artifact:

Validation:
- validation_id:
  exact_command:
  status: passed | failed | blocked
  exit_code:
  duration_ms:
  log_sha256:
  result_summary:
  capability_evidence_path:

Claims verified:
- claim:
  evidence_refs:

Known gaps:
-

Recommended decision: approve | request_changes | block
```

For approval, serialize the equivalent machine-readable record with `schemas/review-packet.v1.schema.json`. The reviewer must bind the exact task digest, committed base/head, canonical patch digest, changed-file set, and every required validation ID.

## Compact Exact Independent Review

Use this section for a report-only exact-candidate review. Replace every
placeholder and delete instructions. Prefer evidence references and compact
tables over repeating source narration. A required row may be `OPEN`,
`CLOSED`, or `ROUTED_OPEN`; routing is never closure.

```text
# <Exact review title>

Reviewed candidate: <full SHA>
Report mode: report-only

## 1. Immutable subject

| Field | Declared | Recomputed | Result |
|---|---|---|---|
| task/review base | <SHA> | <SHA> | MATCH / MISMATCH |
| candidate parent | <SHA> | <SHA> | MATCH / MISMATCH |
| candidate head | <SHA> | <SHA> | MATCH / MISMATCH |
| tree | <SHA> | <SHA> | MATCH / MISMATCH |
| merge base | <SHA> | <SHA> | MATCH / MISMATCH |
| additive commits / merges | <n> / 0 | <n> / <n> | MATCH / MISMATCH |
| binary patch SHA-256 | <digest> | <digest> | MATCH / MISMATCH |
| changed paths | <count/ref> | <count/ref> | MATCH / MISMATCH |
| clean tracked/untracked | true | true/false | MATCH / MISMATCH |

## 2. Scope and production reachability

- live entry point(s):
- reviewed owners/effects/fences:
- exact writable report path:
- out-of-scope surfaces:
- scope deviations: none / <list>

## 3. Required finding dispositions

| ID | Prior claim | Disposition | Production counterexample or proof | Deliberate mutation | Evidence |
|---|---|---|---|---|---|
| <ID> | <one sentence> | CLOSED / OPEN / ROUTED_OPEN | <live route and observed result> | <mutation and targeted failure> | <log/test/symbol> |

Every `CLOSED` row names a reachable owner, effect, fence, terminal snapshot,
and mutation that the exact production oracle kills. Every `OPEN` row names
impact and the smallest repair boundary. Every `ROUTED_OPEN` row names a
committed successor contract and stays excluded from local/global closure.

## 4. Owner/effect/fence summary

| Phase | Physical owner | Charge/reservation | First effect | Final fence | Terminal outcome |
|---|---|---|---|---|---|
| <phase> | <owner> | <dimension/site> | <effect or none> | <authority/deadline> | <status/bytes/debt/permits> |

## 5. Mutation matrix

| Mutation | Exact production edit | Focused oracle | Expected failure | Observed | Reverted/identity restored |
|---|---|---|---|---|---|
| <ID> | <symbol/change> | <test/command> | <reason> | KILLED / SURVIVED | yes/no |

## 6. Validation

| Class | Exact command or cache key | Tree/env/toolchain | Result/count | Duration | Log SHA-256 |
|---|---|---|---|---:|---|
| non-cacheable focused | <argv> | <identity> | <result> | <seconds> | <digest> |
| reusable broad | <argv or key> | <identity> | <result> | <seconds/avoided> | <digest> |

Failed, flaky, interrupted, truncated, different-tree, different-argv,
different-environment, or mutable-name-only evidence is never `PASS` or a
cache hit. Prefer `docs/development/tools/run_timed_gate.py`; report its
monotonic duration and wall-clock-discontinuity field rather than shell wall
time.

## 7. New findings and integration overlap

| Severity | Owner | Finding | Reproducer/evidence | Required successor |
|---|---|---|---|---|
| <severity> | <owner> | <finding or none> | <evidence> | <packet or none> |

## 8. Unavailable evidence

- <item and why unavailable>, or `none`

## 9. Decision

- local required rows closed: <n>/<n>
- routed-open contracts valid: <n>/<n>
- mutations killed: <n>/<n>
- required independent gates: <n>/<n>
- unowned material findings: <n>
- integration authorization: no

OVERALL_VERDICT: ACCEPT
```

The last line appears exactly once and must be either
`OVERALL_VERDICT: ACCEPT` or `OVERALL_VERDICT: REQUEST_CHANGES`.
`ACCEPT_WITH_NOTES` is not permitted. Discussion may describe acceptance
criteria, but must not emit a second `OVERALL_VERDICT` line.

## Successor Review Recurrence Addendum

For review generation 2 or later, use this addendum with the compact exact
review and produce a final
`schemas/review-recurrence-control.v2.schema.json` record. v2 is the only
current authorization candidate. A v1 record is compatibility/historical input
and the normal checker always returns `BLOCK`; do not rewrite accepted packets
or historical review documents to change their schema. This addendum is not a
request for a fixed number of findings. Do not infer thoroughness from report
length or fill a quota.

### Before the repair

The manager, not the implementer, freezes one closure contract for every prior
obligation in the Markdown repair packet and commits it together with a
`review-recurrence-repair-manifest.v1`. The manifest freezes exact obligation,
parity/closure-claim, and required-validation sets; the final v2 record must be
set-equal to them. Git topology must establish that this packet/manifest commit
preceded the repair:

| Obligation ID | Exact acceptance criterion | Authorized boundary | Required live routes | Anticipated regressions | Forbidden shortcuts |
|---|---|---|---|---|---|
| <stable ID> | <externally observable result> | <smallest allowed symbols/paths> | <normal/error/unwind/composition routes> | <specific ways this repair can regress> | <test-only, claim-only, weakened oracle, new API, etc.> |

This table is a communication boundary. If the implementation needs a new
public API, state machine, classifier, persisted shape, or other unplanned
surface, stop for a manager scope decision; do not silently redefine the
obligation. The machine can prove equality to the frozen collections, but a
manager and independent preflight must judge whether those collections are
semantically complete.

Before any live decision, identify a separate clean committed validator source.
The checker and all v2 schemas/modules must match regular blobs at its `HEAD`.
For a manager-fixed Cargo or KGI/KPI chain, the validator source must also hold
the independently committed live-arm anchor. No such anchor is currently
committed, so current live-arm records necessarily remain `BLOCK` until that
separate manager task is completed.

### Independent successor review

Review every prior obligation first, then every exact changed path and its
candidate-created surfaces as a distinct pass. Use the schema's exact
evidence-state and next-action vocabulary; routing controls successor work and
does not authorize integration:

| Prior obligation | Origin generation | Evidence state | Next action | Before/after evidence | Negative control | Independent closure/non-reachability evidence |
|---|---:|---|---|---|---|---|
| <ID> | <n> | DEFECT_REPRODUCED / DEFECT_SOURCE_PROVEN / DEFECT_INJECTION_ONLY / CLOSURE_UNPROVEN / ORACLE_VACUOUS / CLAIM_FALSE / SCOPE_BOUNDARY / REQUIREMENT_UNSATISFIABLE / CLOSED_PROVEN | PRODUCT_REPAIR / RUN_CLOSURE_EVIDENCE / REPAIR_ORACLE / REPAIR_CLAIM / DOCUMENT_NON_REACHABILITY / MANAGER_SCOPE_DECISION / MANAGER_REQUIREMENT_DECISION / NONE | <artifact refs> | killed / survived / not_applicable / unavailable | <artifact ref or null> |

| Exact changed path | Required by | Review focus | Evidence | Disposition |
|---|---|---|---|---|
| <Git-derived path> | <one or more obligation IDs> | <behavior, oracle, claim, boundary> | <artifact refs> | reviewed |

The changed-path rows must equal the canonical Git changed-path set. Every row
must have at least one surface; v2 has no `no_surface_change`, `out_of_scope`,
or unreviewed-path approval escape.

| Exact path | Surface ID | Kind | Scope disposition | Review evidence | Routing decision |
|---|---|---|---|---|---|
| <Git-derived path> | <stable ID> | <diagnostic kind> | reviewed_in_scope / referred_to_manager | <artifact refs> | <routing_scope_decision ref or null> |

`referred_to_manager` requires an earlier bound routing decision and remains
blocking. It is not a silent scope exclusion.

| New finding | Evidence state | Next action | Before/after evidence | Negative control | Routing decision |
|---|---|---|---|---|---|
| <ID> | <v2 state other than CLOSED_PROVEN> | <legal v2 next action> | <artifact refs> | killed / survived / not_applicable / unavailable | <artifact ref or null> |

A new finding cannot become `CLOSED_PROVEN` in the generation that first
records it. `CLOSED_PROVEN` with `NONE` is the only potentially nonblocking
state/action pair, and only when the validator independently derives its
closure proof. All other pairs remain successor obligations.

For every frozen parity or closure claim, record the exact path/locus,
predecessor behavior, candidate behavior, artifact evidence, and disposition.
Git proves path existence, not semantic symbol identity; locus selection and
behavioral interpretation remain reviewer judgements.

### Count-free stopping rule

Mark each condition independently. `ACCEPT` is invalid unless all are true:

- the v2 authority chain roots at the manager-fixed arm and every retained v2
  generation has passed current-gate semantic replay;
- every prior obligation has a disposition;
- every exact changed path has one or more reviewed surfaces;
- no surface is referred to manager;
- every frozen derivation is complete and evidence-bound;
- applicable negative controls were killed;
- unavailable evidence is explicit;
- all manifest-required validations are passing, terminal, exit-code-bound,
  and content-addressed `codefire.timed_gate.v2` evidence;
- each timed log and result was written outside the source worktree and then
  copied byte-identically into the evidence commit;
- the report-only commit exists and the reviewer and all child processes have
  stopped;
- the deciding gate comes from a separate clean committed validator root, and
  the explicit live arm matches a committed validator-owned anchor.

Generate each required timed result from the clean candidate without placing
the live log or result inside that source worktree:

```bash
python3 docs/development/tools/run_timed_gate.py \
  --emit v2 \
  --cwd <clean-candidate-worktree> \
  --log <outside-source>/validation.log \
  --result <outside-source>/validation.json \
  --timeout <seconds> \
  -- <exact frozen argv>
```

After terminalization, the manager writes the JSON record and runs:

```bash
python3 docs/development/tools/check_review_recurrence.py \
  --record <final-record.json> \
  --git-root <analysed-candidate-repository> \
  --validator-root <clean-validator-repository> \
  --live-arm cargo \
  --result <outside-analysed-source>/recurrence-result.json
```

Use `--live-arm kgi_kpi` for that arm. v1, a legacy-bootstrap v2 generation,
an absent/mismatched anchor, or `--no-git-verify` can never return integration
`ALLOW`.

The manager records candidate-created findings, surfaces, repair time, review
time, and verdict-to-report time for later comparison. Machine `ALLOW` proves
only the implemented topology, byte, set, and derived-state predicates. Honest
execution, semantic oracle quality, nonzero selector behavior, complete
surface/symbol enumeration, manifest completeness, authorship, and reviewer
independence remain human semantic judgements and must be stated in the review.
