# Development-Flow KGI/KPI Manager Repair Design

Date: 2026-07-30
Exact candidate: `0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1`
Candidate parent: `a6d41a9d7cdae2157cdcc5ca9f9008e4734c4e16`
Status: manager design; exact-candidate independent decision pending

## Purpose

The candidate passes its 79-test tool matrix and aggregate AI-development
gate, but manager probes found eight representation, lineage, lifecycle, and
repository-path inconsistencies. This document freezes the smallest shared
repair design so a later repair does not add eight unrelated special cases.

It does not accept, integrate, or modify the candidate.

## Four shared primitives

1. An integer-nanosecond `InstantNs` representation for every parsed
   timestamp.
2. A shared valid-JSON-number predicate requiring every numeric value to be
   finite and excluding booleans.
3. Iterative functional-graph validation for candidate and record
   supersession lineage.
4. Canonical repository path and provenance resolution shared by documents,
   locators, flow-record discovery, and final-record history.

## Repair map

### Exact instants

Current `parse_utc_timestamp` converts the validated fraction to `float`.
At 2026 epoch magnitude, instants one nanosecond apart can compare equal.

Required implementation:

- parse the calendar portion to integral epoch seconds;
- pad the optional fraction to exactly nine digits;
- return `epoch_seconds * 1_000_000_000 + fraction_ns`;
- keep ordering/equality in integer nanoseconds;
- convert an interval to seconds only at the measurement arithmetic boundary;
- express the one-second commit-time tolerance as `1_000_000_000` ns.

Apply the representation to record windows, lane endpoints, events,
checkpoints, candidates, evidence capture, artifact production, cohorts,
resume gaps, due dates, Git commit instants, and directory freshness.

Complexity remains O(timestamp length) per parse and O(1) per comparison.

### Finite JSON numbers

Python's JSON loader accepts non-standard `NaN` and infinities by default.
The schema type/minimum/maximum and scalar semantic paths can then accept
non-finite measured values.

Required implementation:

- make JSON loading reject non-standard numeric constants;
- make direct schema validation reject every non-finite float;
- make semantic numeric extraction reject non-finite values and booleans;
- make canonical serialization use `allow_nan=False`;
- preflight values before `uniqueItems` fingerprinting;
- avoid unchecked integer-to-float overflow.

This generic helper must protect all AI contracts, not only the flow record.

### Candidate and record lineage

The candidate checker rejects self-cycles and counts roots but does not detect
a two- or three-node cycle beside one valid root.

Required implementation:

- build `revision -> predecessor`;
- reject duplicates and dangling parents;
- require exactly one root;
- traverse every node iteratively with tri-color state;
- require every path to terminate at that root;
- require predecessor readiness not to follow child readiness.

Use the same algorithm for record supersession. Define the record node as an
exact version, not just `record_id`; current filename/revision documentation
otherwise conflicts with directory uniqueness.

Do not mutate an old final record to add a reverse link. Derive the reverse
relation from a newer version's `supersedes` edge so final-record immutability
remains possible.

Complexity is O(V+E) time and O(V) memory with no recursion.

### Operational due dates and batch isolation

`next_review_due_at` currently proves only that due follows capture. An active
record remains valid after the due instant. Directory freshness also compares
every active record with one global newest capture, coupling unrelated
batches.

Required implementation:

- capture `as_of_ns` once in the production checker and inject it in tests;
- retain structural `due_at > captured_at`;
- report an overdue active record when `as_of_ns > next_review_due_at`;
- report an overdue `in_progress` measurement when `as_of_ns > due_at`;
- exempt final and superseded records;
- compute `newest_by_batch`, never one global newest instant;
- retain the separate one-active-record-per-batch rule.

No test may depend on the wall clock; exact due−1 ns, due, and due+1 ns values
are injected.

### Repository documents and flow discovery

The `doc:` path uses lexical validation followed by `Path.is_file()`, which
follows a final symlink outside the repository. The safe artifact reader
already contains stronger but separate logic.

Required implementation:

- centralize a repository regular-file resolver;
- reject any symlink component;
- resolve beneath the resolved repository root;
- require a regular file;
- use the same resolver for `doc:`, context artifacts, and flow-record
  discovery;
- reject symlinked flow-record JSON files.

Complexity is O(path depth).

### Agent-log locators

The current character class admits `../`, `./`, doubled separators, and
absolute-like tails.

Required implementation:

- parse provider, lane, and path separately;
- require a supported provider;
- split the raw path on `/`;
- reject empty, `.`, and `..` components;
- reject leading/trailing slash;
- require `PurePosixPath(path).as_posix() == path`;
- return a structured locator including the normalized path.

Apply equivalent normalization to repository-relative paths; parsing must not
silently normalize an invalid raw subject into a valid one.

### Final-record provenance

`record_sha256` is a content checksum. It is not an immutability anchor,
because an edit can recompute it.

Required implementation:

- at the production directory route, load raw record bytes;
- locate the first commit in current `HEAD` ancestry where that exact version
  appeared as `final`;
- require current raw bytes to equal that historical blob;
- fail closed for an uncommitted final record or unavailable required history;
- batch Git history/blob queries instead of spawning per reference;
- make a correction a new version while retaining the old final bytes.

An exported or shallow tree may expose a non-decision-grade
`provenance_unavailable` result, but it must not claim final decision-grade
evidence.

## Independent regression matrix

### Time

- post-capture event by one nanosecond must fail;
- pre-capture by one nanosecond must pass;
- a one-nanosecond interval recomputes to `1e-9`;
- reordered input arrays do not alter findings;
- invalid calendar dates, offsets, leap second, and ten fractional digits
  remain invalid;
- the oracle compares literal integer instants and does not call production
  timestamp parsing.

### Numbers

- `NaN`, `Infinity`, and `-Infinity` for scalar value, numerator,
  denominator, uncertainty, interruption seconds, and evidence-producing
  seconds;
- production JSON loading and direct dict validation;
- canonical hashing rejects non-finite input;
- each test requires the intended stable finding, not any unrelated error.

### Graphs

- two- and three-node cycles;
- self-cycle;
- dangling parent;
- duplicate node;
- child ready before parent;
- shuffled valid chain;
- large valid iterative chain;
- equivalent record-supersession cycles and cross-batch edges.

### Due dates and batch grouping

- `as_of` at due−1 ns, due, and due+1 ns;
- active and in-progress overdue findings are distinct;
- final records are unaffected;
- older active batch A plus newer active batch B remains valid;
- older active plus newer same-batch record is stale;
- two active records in one batch remain invalid.

### Paths and locators

- regular repository document passes;
- final and intermediate external symlinks, broken symlinks, and symlinked
  flow records fail;
- reject `../`, `./`, leading slash, doubled slash, and trailing slash;
- accept the canonical relative `logs/events.jsonl`;
- rerun all existing context/artifact callers after resolver consolidation.

### Final provenance

In a temporary Git repository:

- commit active v1, then its first final form; validation passes;
- edit and recompute the checksum in the worktree; validation fails;
- commit that rewrite; it still fails against the first final blob;
- add a v2 correction superseding unchanged v1; both validate;
- uncommitted final and unavailable required history fail closed.

This must execute the production directory route. A dict-only helper cannot
prove repository immutability.

## Compatibility and migration

- Changing timestamp return type requires all annotations, subtraction, and
  tolerance sites to move together.
- Strict JSON loading may reject previously tolerated non-standard files
  across all AI contracts; this is standards-aligned and must be documented.
- Stronger raw-path validation may reject existing `./` or doubled-separator
  references; inventory them before landing.
- Operational due-date enforcement intentionally makes records fail after
  their declared deadline. Refresh, finalize, or explicitly archive the
  factual record; never extend a date merely to make the gate pass.
- Historical final sealing requires committed history. Define the
  non-decision-grade behavior for shallow/exported trees.
- If record supersession reference semantics or persisted seal metadata
  change, use a v2 schema or a precise v1 migration rule.

## Safe commit partition

1. `fix(ai): harden shared JSON, instant, and repository path primitives`
2. `fix(ai): close flow lineage and operational lifecycle invariants`
3. `fix(ai): bind final flow records to committed provenance`
4. `docs(ai): align flow contract and migration guidance`

Each functional commit carries focused tests. The four commits form one
candidate; none is independently sufficient for a decision-grade claim.

## Validation boundary

The later implementer runs only AI tooling and documentation gates for this
surface. No CodeFire product source, Cargo, build, install, demo, or dogfood
gate belongs in this repair lane. A fresh independent review must bind its
decision to the final exact repair commit before integration.
