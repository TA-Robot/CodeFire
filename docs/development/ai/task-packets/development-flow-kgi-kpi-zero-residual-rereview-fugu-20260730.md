# Development-Flow KGI/KPI Zero-Residual Independent Rereview

Status: Ready

## Objective

Perform an independent adversarial review of exact candidate
`0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1` and determine whether it truly
closes every MUST, SHOULD, and COULD finding in the bound independent review:

`docs/development/ai/swarm-results/development-flow-kgi-kpi-independent-review-claude-20260730/review.md`

Use Fugu 1.1 Ultra (`fugu-ultra-v1.1`). The candidate is not accepted merely
because its implementer reports 55/55 semantic mutations rejected, 15/15
structural controls rejected, and 79/79 tests passing. Trace the production
checker, schema, policy, exemplar, and tests independently and construct new
counterexamples.

The required output is an explicit `ACCEPT` or `REQUEST_CHANGES` decision
bound to the exact candidate SHA. Under the zero-residual rule, every Must,
Should, and Could issue assigned to this surface requires closure.

## Exact subject

- Original independent review commit:
  `b307b47f51f73b054b66444493c4590379518c4f`
- Review report SHA-256:
  `02a0d259c55121379f07c2c1d893d4a29dd546b16e6f237659a5081c60fd3336`
- Repair packet commit:
  `9dd0da4f2ae55742ee51c1671aab6308994de24c`
- Packet-section correction:
  `a6d41a9d7cdae2157cdcc5ca9f9008e4734c4e16`
- Exact repaired candidate:
  `0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1`
- Implementation diff:
  `a6d41a9d7cdae2157cdcc5ca9f9008e4734c4e16..0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1`

Review the committed tree only. Do not infer acceptance from uncommitted
state, transcripts, commit messages, or implementer summaries.

## Scope

Read:

- `AGENTS.md`
- the complete original independent review
- the complete repair packet Markdown and typed JSON
- the exact implementation diff and all 14 changed files
- the full resulting schema, semantic validator, directory validator,
  policy, active record, indexes, and tests
- the canonical Discover -> Freeze -> Plan -> Implement -> Close -> Install
  -> Dogfood -> Retrospect lifecycle authorities

Create only:

`docs/development/ai/swarm-results/development-flow-kgi-kpi-zero-residual-rereview-fugu-20260730/review.md`

### Required prior-finding disposition

For every original MUST F1-F14, SHOULD F-S1-F-S14, and COULD F-C1-F-C7:

1. identify the exact enforcing schema rule, semantic-validator branch,
   policy text, exemplar repair, and non-vacuous negative test;
2. construct or replay the smallest falsification case;
3. record `closed`, `partially closed`, `open`, or `regressed`;
4. reject the candidate if any item is not fully closed.

### Semantic-validator audit

Independently verify:

- strict RFC 3339 UTC parsing, leap/timezone handling, interval ordering, and
  schema/semantic agreement;
- ECMA-262 regex translation, end-anchor behavior, full-match assumptions,
  and rejection of trailing-newline or Unicode/escape bypasses;
- all metric IDs map to exactly one canonical unit and deprecated aliases
  cannot create ambiguity or silently preserve invalid records;
- status/value/due-at rules, ratio/percent arithmetic, positive
  denominators, precision, uncertainty, NaN/infinity/boolean rejection, and
  decimal comparison behavior;
- event/lane/window lifecycle completeness, unique IDs, endpoint binding,
  original/resumed start semantics, queue/review/integration/conflict
  intervals, candidate lineage, and lifecycle completion ordering;
- commit/document/evidence/artifact resolution is exact, repository-root
  confined, symlink-safe, batch-resolved, and resistant to Git option/path
  injection, ambiguous prefixes, duplicate subjects, and time-of-check/time-
  of-use changes;
- `record_sha256` and artifact/evidence digests are non-circular,
  deterministically canonicalized, and actually verified rather than merely
  shape-checked;
- cohort preregistration, censoring, dispersion, non-inferiority, provider
  coverage, unavailable-cost refusal, and KGI achievement cannot be gamed by
  omission, splitting, recommitting, supersession, or selective denominator
  membership;
- directory-wide rules find nested records, reject unexpected files, enforce
  one active record per batch, reject stale active/supersession graphs
  structurally, and do not rely on wall-clock-dependent CI outcomes;
- every required safety metric is present with honest status and an
  `achieved`/`failed` KGI has a measured, evidence-bound reference;
- error findings are stable, specific, deterministic, and do not crash or
  perform unbounded work on malformed-but-schema-valid input.

### Manager-confirmed launch counterexamples

The manager reproduced the following against exact candidate `0ae583e`.
Confirm each result through the production checker and include it in the
decision; do not spend the review rediscovering whether the probe exists.

1. An event at
   `2026-07-30T09:51:55.000000001Z` with `captured_at` at
   `2026-07-30T09:51:55Z` returns zero findings. Timestamp parsing converts
   nanosecond instants to binary `float`, so an event one nanosecond after the
   record boundary compares equal at 2026 epoch magnitude.
2. Setting measured scalar `MET-FLOW-017` to Python JSON `NaN` or `Infinity`
   returns zero findings. The schema's number check and semantic comparisons
   do not require finite numbers.
3. One candidate root plus a two-node cycle
   `b1c29fc… -> 6dd4527… -> b1c29fc…` returns zero findings. The validator
   rejects only self-cycles and counts roots; it does not traverse the
   candidate graph.
4. Setting an active record's `next_review_due_at` to one second after
   `captured_at`, but before the actual validation instant, returns zero
   findings. The field has ordering semantics but no operational overdue
   semantics.
5. Two active records from different batches are not independent: the
   directory validator marks the older one superseded because it compares
   every active record with the globally newest capture, rather than the
   newest record in the same batch.
6. `doc:` resolution accepts a repository path whose final component is a
   symlink to `/etc/passwd`: lexical path validation and `Path.is_file()`
   both pass while the resolved target is outside the repository.
7. Agent-log locator parsing accepts `../`, `./`, and a doubled leading slash
   in the path component because the parser validates characters but not
   normalized relative-path components.
8. A final record's `record_sha256` is self-recomputable after arbitrary
   edits. No rule binds the sealed bytes to `source_revision` or the first
   commit that introduced the final record, so the claimed final-record
   immutability is not yet independently enforced.

These are candidate defects or explicit contract-policy conflicts, not
authorization to edit in the review lane. Verify their exact severity and
look for the same representation, graph, path, and lifecycle mistakes on
adjacent fields.

### Non-vacuity and mutation audit

- Reproduce all 55 semantic mutations and 15 structural controls against the
  exact candidate, checking the expected finding rather than any rejection.
- Inspect whether tests call the same helper as the implementation under
  test, derive expected values from candidate output, accept unrelated
  findings, or omit the production directory/checker route.
- Add independent adversarial cases beyond the frozen set: duplicate and
  reordered objects, numeric boundary types, Unicode identifiers, cyclic
  supersession, deeply nested/large arrays, dangling local references,
  ambiguous Git revisions, symlinked evidence/documents, modified record
  during validation, clock-independent stale records, cross-batch lane
  locators, and forged digest metadata.
- Audit time and memory complexity. Identify any attacker-controlled
  quadratic traversal, repeated Git subprocess amplification, recursive
  structure risk, or unbounded diagnostic accumulation.
- Recompute every exemplar interval, ratio, percent, count, byte conversion,
  checkpoint denominator, evidence binding, and KGI disposition independently.

### Decision-quality audit

Distinguish:

1. whether the contract is mechanically valid;
2. whether the current CFR-168 record is factually honest;
3. which KPIs are decision-capable now;
4. which remain unavailable, absent, estimated, or lack a preregistered
   cohort;
5. whether policy text could still authorize a development-process change
   from incomplete evidence.

An honest "not decision-capable yet" result is acceptable. Fabricated
completeness, zero-valued missing safety data, unverifiable evidence, or a
claim whose denominator can be selected is blocking.

## Non-Goals

- Do not edit product source, tests, schemas, policy, exemplar, canonical
  issue state, existing reports, or the reviewed commit.
- Do not fix findings in the review lane.
- Do not review devcontainer implementation, CI, packaging, host resources,
  or unrelated provider wrappers.
- Do not run Cargo, Rust builds, install, demo, dogfood, broad product Python
  suites, or network calls other than the Fugu provider request.
- Do not authorize product integration, CFR-168 Close, Install, Dogfood, or
  Retrospect.
- Do not accept based on test count, commit size, provider time, or
  implementer confidence.

## Validation

Bounded read-only execution is allowed:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/test_ai_tools.py
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_contracts.py
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_development.py --timeout 120
PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py
git diff --check a6d41a9d7cdae2157cdcc5ca9f9008e4734c4e16..0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1
```

Prefer direct validator calls and temporary files outside the worktree for
new adversarial probes. Do not add tests or mutate the candidate. Record every
command, exact count, duration when available, and result. Clearly separate
commands actually executed from implementer evidence merely inspected.

Before committing the report:

```bash
git diff --check
git diff --name-only
git status --short
```

## Handoff

The single report must include:

- exact subject, baselines, report digest, reviewed blobs, and changed files;
- method, constraints, and every command actually run;
- severity-ordered findings with exact file:line references and executable or
  source-level counterexamples;
- a row-by-row disposition of F1-F14, F-S1-F-S14, and F-C1-F-C7;
- schema/semantic/policy/exemplar/test alignment;
- mutation non-vacuity and new adversarial-case results;
- arithmetic, reference, digest, lineage, lifecycle, cohort, KGI/KPI, and
  complexity audits;
- an explicit `ACCEPT` or `REQUEST_CHANGES`;
- required next action and exact residual owners;
- report commit, report path, and clean-worktree confirmation.

Commit only the report. An `ACCEPT` authorizes a manager to consider the exact
candidate for integration; it does not itself integrate, Close, Install,
Dogfood, or Retrospect.
