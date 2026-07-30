# CFR-168 Final Complexity / Conformance Freeze

Status: blocked pending exact acceptance and integration of final
decision-work metrics projection

Typed packet:
[`json/cfr168-final-complexity-conformance-freeze-20260730.json`](json/cfr168-final-complexity-conformance-freeze-20260730.json)

Freeze base: `1ec72e70a5241b8506afad94636392ff240816b9`

Root finding: S3. This packet also owns the final shared Rust/Python
decision-work corpus required by the post-composition lifecycle.

No worker manifest is authorized while the metrics successor, final
production call graphs, counter formulas, profiles, or selected operations
remain unbound.

## Objective

Publish one canonical table for the actual selected operations' time,
retained-memory, temporary-memory, and physical-I/O orders, then bind every
row to exact production counters, boundaries, and a versioned shared
Rust/Python corpus.

This is deliberately separate from product implementation. If the table or
corpus exposes a missing counter, wrong formula, divergent runtime behavior,
or nonlinear production path, return the defect to the metrics/product
successor and remain `BLOCK`; do not patch product code here.

## Required launch bindings

The manager must bind:

1. exact accepted and integrated metrics-projection SHA/tree/report/digest;
2. exact accepted Python/Rust compositions and all selected successors;
3. the final selected-operation and production-entry-point inventory;
4. exact counter owners/formulas, budget profiles, typed phases, and units;
5. exact current Rust/Python conformance schema and corpus version; and
6. a clean base with a docs/corpus/test-only write-set lease.

## Scope

Read all accepted production call graphs, counters, profiles, exact reports,
existing Rust/Python conformance consumers, scale fixtures, canonical source
contracts, and the post-composition lifecycle packet.

The Fugu preflight may write only its report. The later Claude writer may
change only the manager-bound canonical order documentation, versioned
conformance corpus/schema, and direct corpus/fixture tests. `codefire`,
`crates/**`, capability declarations, issue/lifecycle state, devcontainer,
packaging, and unrelated documentation are read-only or out of scope.

## Canonical table

For each selected operation, record:

- production entry point and input scale variables;
- worst-case time order;
- retained- and temporary-memory order;
- physical read/open/write/publication order;
- exact witnessing counter formulas and units;
- budget owner, dimension, profile, and typed failure phase;
- exact, minus-one, and plus-one boundary fixture;
- 1x/3x/10x expected counters and slope;
- Rust/Python parity rule or explicit runtime-specific reason; and
- exact source/test/corpus references.

Every table row must have a production-path fixture. Every fixture must map
back to one row. Qualitative prose, helper-only microbenchmarks, or an `O(...)`
label without counters and boundaries is incomplete.

## Shared corpus

Create or extend one versioned, deterministic corpus consumed by both
runtimes. It must cover common ancestry and tie-breaks, required links,
sealed/signature/object validation, selected HTTP/file decisions, typed
budget boundaries, cache transitions, projection envelopes, and exact
1x/3x/10x counter expectations where applicable.

The corpus records expected semantics and counters, not implementation
internals. Rust and Python must either agree byte/field-for-field or name a
canonical, reviewed runtime-specific distinction.

## Mandatory counterexamples

Detect at least:

1. remove one canonical table row or its fixture;
2. change a counter formula without changing the documented order;
3. change an order without changing 1x/3x/10x expectations;
4. omit retained or temporary peak;
5. make scale pass through an unrelated earlier budget;
6. diverge one Rust/Python corpus outcome;
7. use a helper-only path instead of the production entry point;
8. accept a nonlinear 10x result under a claimed linear row; and
9. change a schema/corpus version without lineage and digest updates.

## Validation

One Claude Opus 5 docs/corpus writer builds the immutable candidate after a
short Fugu 1.1 Ultra `READY/BLOCK` mapping review. A fresh independent Fugu
session reviews the exact candidate.

Required gates are both runtime corpus consumers, production-path exact and
scale fixtures, schema/lineage/digest validation, canonical source-doc
checks, AI contract/doc checks, exact parent/head/tree/patch/write-set/log
digests, and clean status. Product source paths in the diff force `BLOCK` and
return to the preceding packet.

Broad product gates are reused from the unchanged accepted composition; run
only the corpus/source-contract gates affected by this docs/corpus candidate,
then one stable lifecycle gate after manager composition.

Record stage-ready/start/checkpoint/end, validation cache key, review
ready/start/decision, queue reason, and integration times. A healthy stage
must durably checkpoint within 45 minutes.

## Handoff

Provide exact input identities, preflight report/digest, canonical table,
table-to-fixture bijection, versioned corpus and lineage, Rust/Python results,
mutation matrix, exact candidate identity, independent decision, stable
composition binding, and the exact input for final capability
re-advertisement.

## Non-Goals

- Do not launch before the metrics successor is accepted and integrated.
- Do not modify product code or counter behavior.
- Do not advertise final capability.
- Do not review development-environment or packaging infrastructure.
