# Development-Flow KGI/KPI Manager-Counterexample Repair Task Packet

Status: blocked pending exact-candidate independent rereview

Typed packet:
[`json/development-flow-kgi-kpi-manager-counterexample-repair-20260730.json`](json/development-flow-kgi-kpi-manager-counterexample-repair-20260730.json)

Exact rejected candidate:
`0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1`

Candidate parent:
`a6d41a9d7cdae2157cdcc5ca9f9008e4734c4e16`

Manager repair design:
[`development-flow-kgi-kpi-manager-repair-design-20260730/design.md`](../swarm-results/development-flow-kgi-kpi-manager-repair-design-20260730/design.md)
at SHA-256
`fdd218b61e933d36a1e4dea647e8d105aab55b9dd7a940693f5128da0dce842c`.

The candidate passes 79/79 focused tool tests and all aggregate
AI-development/documentation gates, but eight manager counterexamples are
reproducible. Those passing gates are therefore insufficient for integration.
Do not launch this repair until the Fugu Ultra 1.1 rereview of the exact
candidate has committed its report and this packet has been updated to bind
that report's exact commit, SHA-256, decision, and any additional findings.

## Objective

Repair the KGI/KPI development-flow control plane through four shared
primitives rather than eight independent special cases:

1. integer-nanosecond instants;
2. finite JSON-number enforcement;
3. iterative functional-graph validation;
4. canonical symlink-safe repository/provenance resolution.

Close all eight manager counterexamples, the adjacent representations named
in the bound design, and every later independent-review finding in one
decision-grade candidate. Preserve the original 55-mutation semantic closure
and the canonical
Discover→Freeze→Plan→Implement→Close→Install→Dogfood→Retrospect lifecycle.

## Launch blocker

The exact-candidate independent rereview is prepared at:

`docs/development/ai/task-packets/development-flow-kgi-kpi-zero-residual-rereview-fugu-20260730.md`

The repair may move from `blocked` to `ready` only after:

- the rereview report is committed on the exact candidate;
- the report and its SHA-256 are copied into this packet;
- all new findings are added to Required Closure and the regression matrix;
- the typed packet is updated consistently;
- a `dev-swarm.v1` manifest is validated against the final packet revision.

No passing manager gate or design note substitutes for that independent
decision.

## Non-Goals

- Do not change CodeFire product behavior, product candidates, issue state,
  devcontainer files, or agent wrappers.
- Do not fabricate a baseline, KGI achievement, cost total, final provenance,
  or refreshed lifecycle date.
- Do not make shallow/exported history decision-grade merely to preserve a
  passing gate.
- Do not run Cargo, product build, install, demo, dogfood, broad product
  Python suites, or network calls.
- Do not redesign unrelated task, review, context-manifest, or routing
  contracts.

## Required Closure

### F1 — exact integer-nanosecond instants

1. Replace float timestamps with an `InstantNs` integer representation:
   validated integral epoch seconds multiplied by `1_000_000_000`, plus a
   right-padded nine-digit fractional component.
2. Compare and order instants only as integers. Convert intervals to seconds
   only at measurement arithmetic boundaries.
3. Express commit-time tolerance as integer nanoseconds.
4. Apply the representation to every record, lane, event, checkpoint,
   candidate, evidence, artifact, cohort, resume, due-date, Git-commit, and
   directory-freshness instant.
5. A post-capture event by one nanosecond must fail; a pre-capture event by one
   nanosecond must remain ordered; a one-nanosecond duration must recompute to
   `1e-9`.

### F2 — valid finite JSON numbers everywhere

1. Reject `NaN`, `Infinity`, and `-Infinity` during JSON loading.
2. Reject non-finite floats and booleans at schema-number and semantic-number
   validation boundaries.
3. Make canonical serialization use `allow_nan=False`.
4. Reject non-finite elements before `uniqueItems` fingerprinting.
5. Avoid unchecked integer-to-float overflow.
6. Cover scalar values, numerator, denominator, uncertainty, interruption
   seconds, evidence-producing seconds, and every schema numeric field in
   both production JSON loading and direct-object validation.

### F3 — candidate and record lineage are complete acyclic graphs

1. Build exact node-to-predecessor maps; reject duplicate nodes and dangling
   predecessors.
2. Require exactly one root and use iterative tri-color traversal so every
   candidate path terminates at that root.
3. Require predecessor `ready_at` not to follow child `ready_at`.
4. Apply the same non-recursive algorithm to record supersession.
5. Define record nodes by exact version, not only `record_id`; keep filename,
   revision, and supersession semantics consistent.
6. Derive reverse supersession from the successor edge. Never edit an old
   final record merely to add `superseded_by`.
7. Cover two- and three-node cycles beside a valid root, self-cycle, dangling
   parent, duplicate node, reversed time, shuffled valid chain, and a large
   iterative chain.

### F4 — operational due dates use one injected validation instant

1. Capture `as_of_ns` exactly once in the production route and inject it in
   tests.
2. Retain the structural rule `due_at > captured_at`.
3. Reject an active record only when `as_of_ns > next_review_due_at`.
4. Reject an `in_progress` measurement only when `as_of_ns > due_at`.
5. Exempt final and superseded records.
6. Test due−1 ns, due, and due+1 ns without consulting wall time.
7. Factual records must be refreshed, finalized, or honestly archived when
   overdue; never extend a due date solely to make the gate pass.

### F5 — freshness is isolated by batch

1. Compute `newest_by_batch[batch_id]`; never compare an active record to one
   global newest capture from another batch.
2. Preserve the one-active-record-per-batch rule.
3. Cover older active batch A plus newer batch B, older active plus newer
   same-batch record, duplicate active records, and equal capture instants.

### F6 — repository references and flow discovery are symlink-safe

1. Centralize one canonical repository regular-file resolver.
2. Reject any symlink component, require the resolved target beneath the
   resolved repository root, and require a regular file.
3. Use the same resolver for `doc:`, context artifacts, repository artifacts,
   and flow-record discovery.
4. Reject final-component, intermediate, broken, and flow-record symlinks.
5. Snapshot/resolve errors must fail closed rather than silently omit a
   subject.

### F7 — raw locator and repository paths are normalized before acceptance

1. Parse provider, lane, and path separately and require a supported provider.
2. Reject empty, `.`, and `..` path components, leading/trailing slash,
   doubled slash, and any raw path whose canonical POSIX spelling differs.
3. Apply the same raw-subject rule to repository-relative paths; parsing must
   not normalize an invalid input into a valid one.
4. Preserve the canonical `logs/events.jsonl` locator.
5. Inventory all repository references before enabling the stricter rule and
   document any factual migration.

### F8 — final records are bound to committed provenance

1. Treat `record_sha256` as a content checksum, not an immutability anchor.
2. In the production directory route, load the raw record bytes and locate the
   first commit in current `HEAD` ancestry where that exact version appeared
   as `final`.
3. Require current bytes to equal that historical blob.
4. Batch Git history and blob queries; do not spawn one process per record.
5. Fail closed for an uncommitted final record or unavailable required
   history. An explicit shallow/exported-tree mode may report
   `provenance_unavailable`, but cannot claim decision-grade finality.
6. A correction is a new version that supersedes unchanged prior final bytes.
7. In a temporary Git repository, prove active→final success, worktree rewrite
   failure after checksum recomputation, committed rewrite failure, valid v2
   correction, uncommitted-final failure, and unavailable-history failure.

## Adjacent invariant closure

- Exact instants and finite numbers must be shared helpers used by all AI
  contracts, not flow-record-only wrappers.
- `uniqueItems`, canonical digests, and schema validation must return stable
  findings rather than crash on malformed numeric input.
- Candidate and record graph validation must remain O(V+E) time, O(V) memory,
  and non-recursive.
- Repository path validation, context-artifact reading, evidence resolution,
  and flow-record discovery must not drift into separate policies.
- Version identity and immutable-final correction semantics must be reconciled
  in schema, policy, README, exemplar, history, and tests. If persisted v1
  meaning changes, define a precise v1 migration or introduce v2.
- Operational overdue enforcement must keep tests deterministic and make
  stale records honestly non-decision-grade.
- All 55 original semantic reversals must remain rejected for their intended
  stable findings.

## Non-vacuous regression matrix

Use independent literal oracles; do not call the production helper being
tested to compute expected outcomes.

- Time: ±1 ns capture boundaries, 1 ns interval, shuffled order, invalid date,
  offset, leap second, and ten fractional digits.
- Numbers: all three non-finite constants across every numeric position,
  direct dict and JSON routes, canonical hashing, schema arrays.
- Graphs: candidate and record cycles, dangling/duplicate nodes, cross-batch
  supersession, reversed readiness, shuffled and large valid chains.
- Lifecycle: injected due−1/due/due+1 for active, in-progress, final, and
  superseded states.
- Grouping: same-batch and cross-batch freshness with deterministic ties.
- Paths: regular file; final/intermediate/broken/external symlink; symlinked
  record; `../`, `./`, leading, doubled, and trailing separators.
- Provenance: production directory validation in temporary full and
  unavailable-history Git repositories.
- Reversal: each of the eight manager probes must fail against exact candidate
  `0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1` for its intended reason.

## Scope

Only the typed packet's write set may change. CodeFire product source, Cargo,
product tests, issue state, devcontainer files, agent wrappers, external logs,
independent reports, task packets, and result/design evidence are read-only.

## Validation

Run bounded repository-local Python and documentation checks only:

1. `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/test_ai_tools.py`
2. `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_contracts.py`
3. `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_development.py --timeout 120`
4. `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py`
5. `git diff --check`

The final candidate must also rerun every aggregate AI-development check used
by the manager's 79-test baseline. No Cargo, product build, install, demo,
dogfood, broad product Python suite, or network access belongs to this lane.

## Handoff

Commit the complete repair as one reviewable candidate, using the design's
four safe implementation partitions but treating their union as the only
acceptable result. Report:

- exact task base, final head, and commit partition;
- exact changed and intentionally untouched files;
- F1 through F8 plus later rereview-finding closure;
- shared helper APIs and complexity;
- full regression/reversal matrix with stable finding codes;
- temporary-Git provenance evidence;
- command counts, durations, and hashes;
- migration and shallow/exported-tree behavior;
- whether the active record is actually decision-grade;
- any exact residual and a request for fresh independent review.

Any unresolved manager or independent-review finding means
`REQUEST_CHANGES`. Do not label a passing structural gate, recomputed digest,
or refreshed due date as decision-grade evidence by itself.
