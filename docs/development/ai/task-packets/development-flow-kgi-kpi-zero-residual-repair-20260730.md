# Development-Flow KGI/KPI Zero-Residual Repair Task Packet

Status: cancelled — implementation produced exact candidate
`0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1`, but eight manager
counterexamples make that candidate non-integratable

Successor:
[`development-flow-kgi-kpi-manager-counterexample-repair-20260730.md`](development-flow-kgi-kpi-manager-counterexample-repair-20260730.md)
is blocked pending the exact-candidate Fugu rereview.

Typed packet:
[`json/development-flow-kgi-kpi-zero-residual-repair-20260730.json`](json/development-flow-kgi-kpi-zero-residual-repair-20260730.json)

Base revision: `b307b47f51f73b054b66444493c4590379518c4f`

Independent review:
[`development-flow-kgi-kpi-independent-review-claude-20260730/review.md`](../swarm-results/development-flow-kgi-kpi-independent-review-claude-20260730/review.md)
at SHA-256 `02a0d259c55121379f07c2c1d893d4a29dd546b16e6f237659a5081c60fd3336`.

## Objective

Turn the development-flow KGI/KPI framework from a structurally valid manager note into a
decision-grade, semantically validated measurement contract. Close every `MUST`, `SHOULD`, and
`COULD` item in the bound independent review in one candidate, with negative tests for every
review mutation and with the active CFR-168 recovery record repaired to satisfy the resulting
contract.

The KGI remains trusted frozen-batch completion with quality, continuity, Codex-quota independence,
and improving cost-to-trusted-integration as guardrails. Activity counts, commit counts, agent
runtime, and cheap incomplete cohorts must not become success signals.

## Scope

The policy, flow-record schema and exemplar, semantic contract checker, focused negative tests,
canonical lifecycle documentation, Phase-2 promotion record, and their indexes/history are in
scope. The independent review is immutable input evidence. The typed packet's exact `write_set`
is authoritative.

## Non-Goals

- Do not change CodeFire product behavior or any CFR-168 product candidate.
- Do not review or alter the devcontainer, CI, packaging, host-resource controls, or agent wrappers.
- Do not fabricate a five-packet baseline, evidence, provider cost, or KGI achievement.
- Do not run release, install, dogfood, Cargo, build, demo, or broad product test gates.
- Do not redesign unrelated task, review, context-manifest, or routing contracts.

## Required Closure

### 1. Temporal integrity

- Validate every temporal value as RFC 3339 UTC and reject trailing characters.
- Enforce record, lane, event, checkpoint, candidate, measurement-window, and final-record ordering.
- Represent original start and resume boundaries explicitly; all start-anchored metrics use the
  original start.
- Correct MR-list `MET-FLOW-004` to `1319.839` seconds or encode an equivalently explicit,
  non-understating interval.
- Require declared precision or uncertainty for estimated measurements.

### 2. Status, value, rate, and unit semantics

- `measured` and `estimated` require a value; `unavailable` and `in_progress` forbid one.
- Rates require a positive denominator, bounded numerator, and a value arithmetically consistent
  with ratio/percent units.
- Bind every metric ID to allowed units and split every dual-quantity metric into independently
  aggregatable IDs.
- Require `due_at` for `in_progress` and reject stale active claims.

### 3. Reference and evidence integrity

- Enumerate and resolve every evidence-reference scheme.
- Resolve lanes, prefixes, measurements, events, repository documents, commits, source revisions,
  and checkpoint commits; require `prefix_bytes > 0`.
- Replace the unbound manager snapshot or downgrade its measurement honestly.
- Promote forged-evidence, redaction-leak, and schema-security-downgrade zero-target metrics and
  publish a transcript redaction rule.

### 4. Events, lifecycle, and completeness

- Enforce lane starts and every policy-required event class, including derivable queue, review,
  integration, conflict, and lifecycle endpoints.
- Reconcile the schema with the canonical eight-stage
  Discover→Freeze→Plan→Implement→Close→Install→Dogfood→Retrospect lifecycle, and publish entry/exit
  criteria.
- Require the unique complete `KGI-DEV-001` through `KGI-DEV-005` set and decision-capable
  measurement/evidence bindings.
- Require mutation timestamps for mutated lanes, define the `MET-FLOW-019` denominator, and
  distinguish absent safety evidence from measured zero.

### 5. Decision and comparability

- Pre-register comparison cohorts with task-class sample sizes, censoring, dispersion,
  non-inferiority, and unavailable-cost refusal rules.
- Add trusted-integration throughput.
- Publish metric aliases/deprecations and eliminate duplicate-series ambiguity.
- Reconcile Phase-2 promotion status and mark unbaselined thresholds provisional rather than
  evidence-derived.

### 6. Record lifecycle and gaming resistance

- Enforce one active record per batch, active-record freshness, final `ended_at`, filename/record-ID
  binding, supersession, and final-record immutability semantics.
- Replace self-declared artifact counts with evidence-bound artifact identity.
- Preserve candidate lineage so first-pass results cannot be reset by recommitting.
- Pair review/rework measures with quality and escape guardrails.
- Separate model, instrument, source, and runtime version.
- Run flow validation independently of the role-routing matrix and add a `flow-metrics/README.md`.
- Add negative tests for all 55 semantic mutations, all checker-level lifecycle cases, and the new
  positive reference record.

## Write Ownership

Only the files listed in the typed packet's `write_set` may change. The independent report is
read-only evidence. Do not edit CodeFire product source, tests outside the AI tooling test, issue
state, devcontainer files, agent wrappers, or external-session logs.

## Validation

Run bounded, repository-local Python and documentation checks only:

1. `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/test_ai_tools.py`
2. `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_contracts.py`
3. `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_development.py --timeout 120`
4. `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py`
5. `git diff --check`

No Cargo, build, install, demo, broad product Python suite, or network access belongs to this lane.

## Handoff

Commit the complete repair as one clean candidate. Report the exact base/head, changed files,
review-finding closure matrix, semantic mutation results, focused gate results, residual risks, and
whether the active metric record is now suitable for routing/cohort decisions. Any unresolved review
item means `REQUEST_CHANGES`; do not label a partial semantic validator decision-grade.
