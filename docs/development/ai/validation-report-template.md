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
