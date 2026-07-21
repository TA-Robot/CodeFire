# CFR-168 Decision-Equivalent Work Budget Task Packet

Status: in progress

Typed packet: [`json/cfr-168-decision-work-budget.json`](json/cfr-168-decision-work-budget.json)

## Objective

Implement the frozen CFR-168 root across Core, Store, CLI, signatures, object transfer, HTTP,
metrics, and Python compatibility without changing semantic results or weakening validation.

## Scope

Core Trace evaluation, Store validated traversal, CLI ancestry/signature/object transfer, HTTP
request/response generation, aggregate metrics, Python compatibility, canonical product/source
documentation, and lifecycle evidence are in scope. Dogfood source code, devcontainer, CI, packaging,
and unrelated frozen roots are outside the write set.

## Non-Goals

- Do not adopt transitive Trace propagation.
- Do not build a persistent graph database or mtime/size-only cache.
- Do not add AI orchestration to CodeFire.
- Do not change the Algorithm dogfood implementation to make the gate pass.

## Parallel Lanes

| Lane | Scope | Write ownership | Gate |
|---|---|---|---|
| Manager/integration | task contract, issue/ADR ownership, CLI/Store integration, final gates | all integration files after lane handoff | full lifecycle |
| Core implementer | direct Trace impact and required-link indexed evaluator plus work counters | `crates/codefire-core/src/` only | core focused tests |
| ADR reviewer | adversarial semantic/resource/cache/compatibility review | read-only | ACCEPT or ranked BLOCK |
| Remote auditor | ancestry/signature/object/HTTP counterexamples and API map | read-only | exact fixtures and boundary map |

Only the manager may admit a same-root blocker, change the frozen issue scope, update authoritative
ledgers, integrate overlapping files, or close CFR-168.

## Invariants

- Existing result, ordering, Fire/object identity, and corruption diagnostics remain equivalent.
- Budget is charged before allocation, parsing, queueing, response publication, or authority writes.
- Budget exhaustion is not corruption and never becomes successful partial output.
- Metrics are privacy-safe aggregate evidence and cannot control semantic evaluation.
- Commit/signature/object traversal is iterative and command-scoped memoization is not a new
  persistent authority.

## Lifecycle Gate

Discover/Freeze/Plan require accepted ADR-020 and executable counterexamples. Implement requires
focused reference-equivalence and limit-boundary tests. Close requires full Rust/Python/docs gates.
Install/Dogfood require a fresh source-matching binary, installed reversal fixtures, Algorithm tests,
canonical status/scan/verify/Doctor, and an explicit Retrospect disposition for every new finding.

## Validation

Focused Core and CLI work-budget tests precede the complete Rust fmt/Clippy/workspace gate. Python
fallback tests, demo, both source/project documentation checkers, fresh isolated install, installed
limit reversals, Algorithm tests, and canonical read-only/mutating workflow checks are mandatory.

## Handoff

Each lane reports changed files, protected invariants, counterexamples, exact commands/results, and
residual risks. The manager integrates these into accepted ADR evidence, complexity/budget tables,
the CFR detail/map/history ledgers, installed runtime evidence, and the final Retrospect.
