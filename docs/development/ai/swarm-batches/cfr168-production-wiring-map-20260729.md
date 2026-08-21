# CFR-168 Production-Wiring Map

Inspect immutable current-state snapshot `e2a218e015953f082245f36dbc6aee906b837e5a`.

## Objective

Map the exact remaining path from accepted but production-unwired CFR-168 primitives to the live
Python and Rust command/HTTP handlers. This is mechanical source tracing for implementation packet
design; do not select a new architecture and do not edit product code.

## Required work

- Start from the latest CFR-168 and CFB-136 evidence, not old REJECT entries already superseded by
  later acceptance.
- Enumerate every symbol explicitly described as production-unwired, frozen, dead-code allowed, or
  missing handler adoption.
- For each symbol, identify:
  - constructor and ownership token;
  - required predecessor and successor;
  - current production call site that bypasses it;
  - semantic/diagnostic/order contract that must remain byte-identical;
  - focused tests that cover the primitive;
  - missing production-path test.
- Trace both existing-project and missing-project upload/publication paths, response selection,
  request deadline, signature/verification/ancestry, object digest, stable gate, bootstrap
  transaction, effect input, publication input/preimage, reducer, and handler dispatch.
- Separately trace remaining Rust CFB-136 consumers: branch-ref/JSON accounting,
  Doctor/storage/ancestry session reuse, capability/metrics exposure.
- Use `rg`, call-site inspection, module ownership docs, and test-to-production path comparison.
- Propose the smallest ordered implementation packets. Declare exact write sets and serialize every
  packet that touches the Python `codefire` monolith or the same Rust module.

## Deliverable

Create only:

`docs/development/ai/swarm-results/cfr168-production-wiring-map-20260729/map.md`

The report must end with an ordered critical path, acceptance commands, and a list of questions
that genuinely require Claude architecture judgment. Commit the report and leave the worktree
clean. Do not merge, change issue status, or modify any existing branch.
