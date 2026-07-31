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
`schemas/review-recurrence-control.v1.schema.json` record. It is not a request
for a fixed number of findings. Do not infer thoroughness from report length
or fill a quota.

### Before the repair

The manager, not the implementer, freezes one closure contract for every prior
obligation in the content-addressed repair packet:

| Obligation ID | Exact acceptance criterion | Authorized boundary | Required live routes | Anticipated regressions | Forbidden shortcuts |
|---|---|---|---|---|---|
| <stable ID> | <externally observable result> | <smallest allowed symbols/paths> | <normal/error/unwind/composition routes> | <specific ways this repair can regress> | <test-only, claim-only, weakened oracle, new API, etc.> |

This table is a communication boundary. If the implementation needs a new
public API, state machine, classifier, persisted shape, or other unplanned
surface, stop for a manager scope decision; do not silently redefine the
obligation.

### Independent successor review

Review every prior obligation first, then review the candidate-created delta
as a distinct pass:

| Prior obligation | Observation ID | Effect episode ID | Origin generation | Before | After | Negative control | Disposition |
|---|---|---|---:|---|---|---|---|
| <ID> | <this report's observation> | <underlying effect> | <n> | reproduced / justified N/A | verified / open | killed / justified N/A | CLOSED / OPEN / ROUTED_OPEN / UNAVAILABLE |

| Actual repair surface | Required by | Planned? | Kind | Review evidence | Disposition |
|---|---|---|---|---|---|
| <path:symbol/claim> | <obligation ID> | yes + plan ref / no | public API / state machine / classifier / persisted shape / private symbol / oracle / claim | <counterexample, test, source> | REVIEWED / BLOCKED / UNREVIEWED |

| New observation | Effect episode | Candidate-created? | Causal class(es) | Blocking? | Evidence | Disposition/route |
|---|---|---|---|---|---|---|
| <ID> | <stable underlying effect> | yes / pre-existing / unknown | initial-scope miss / repair-introduced / bound-not-closed / oracle / communication / reviewer artifact / scope expansion / adjacent / composition | yes/no | <live reproducer> | OPEN / ROUTED_OPEN / UNAVAILABLE |

Do not count a reworded observation as a new effect episode. Multiple causal
classes may apply; do not use a generic `mixed` label. A routed item names an
owner, release condition, and exact manager authority, and it must be
nonblocking.

### Count-free stopping rule

Mark each condition independently. `ACCEPT` is invalid unless all are true:

- every prior obligation has a disposition;
- every declared review surface was checked;
- the actual repair delta, including unplanned surfaces, was checked;
- required production routes were exercised or explicitly unavailable;
- applicable negative controls were killed;
- unavailable evidence is explicit;
- required validation is terminal, exit-code-bound, and content-addressed;
- the report-only commit exists and the reviewer and all child processes have
  stopped.

After terminalization, the manager writes the JSON record and runs:

```bash
python3 docs/development/tools/check_review_recurrence.py \
  --record <final-record.json> \
  --git-root .
```

The manager records distinct observation/effect counts, candidate-created
findings, unplanned surfaces, repair time, review time, and
verdict-to-report time for later comparison. `--no-git-verify` is diagnostic
only and can never return integration `ALLOW`.
