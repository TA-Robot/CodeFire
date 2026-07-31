# AI Development Contracts

These JSON Schemas are the portable contract for AI-assisted CodeFire development. JSON is authoritative; Markdown packets are human views and may be generated from the same data.

| Schema | Purpose |
|---|---|
| `task-packet.v1.schema.json` | Bounded objective, context, read/write scope, risk route, invariants, validation, and handoff. |
| `context-manifest.v1.schema.json` | Resolved and content-addressed context inputs for reproducible delegation. |
| `review-packet.v1.schema.json` | Patch identity, claims, independent evidence, findings, and an explicit decision. |
| `review-recurrence-control.v1.schema.json` | Rejected historical recurrence shape, retained byte-for-byte for compatibility evidence. Normal gate input always produces `BLOCK`; it is never current authorization. |
| `review-recurrence-control.v2.schema.json` | Current experimental recurrence candidate: Git-bound authority generations, exact artifact roles, path/surface coverage, derived evidence state and next action, committed timed evidence, and terminalization. |
| `review-recurrence-repair-manifest.v1.schema.json` | Pre-repair manager anchor for exact obligation, parity/closure-claim, and required-validation sets. The machine checks equality, not semantic completeness. |
| `timed-gate-result.v2.schema.json` | Candidate revision/tree, clean-before/after and index-flag state, monotonic execution, and complete log identity for recurrence evidence. |
| `development-flow-record.v1.schema.json` | Batch KGI status, lane timing, KPI measurements, incidents, and immutable active-log prefixes. |

Contract rules:

- Repository paths are relative to the repository root. A runtime validator must reject absolute paths and `..` traversal even where JSON Schema implementations cannot express that portably.
- Every risk dimension is scored `0..2`; the sum is `0..12`.
- A mandatory-Ultra reason overrides the numeric threshold.
- Packet and patch hashes use SHA-256 over canonical UTF-8 JSON or canonical patch bytes.
- A passing schema check proves shape, not truth. Semantic validators recompute task/context digests and bind reviews to task scope and required validation; repository evidence remains mandatory.
- A recurrence record is valid management evidence, not automatically an approval. `check_review_recurrence.py` always blocks v1 compatibility input and only considers v2 for `ALLOW`; schema-only or `--no-git-verify` mode always blocks integration.
- Recurrence v2 schemas and gate modules are loaded from the identity-bound `--validator-root`, not from the analysed candidate. The validator source must match a clean committed revision. Compatibility schema flags may only repeat those canonical validator-owned paths.
- Manager-fixed Cargo/KGI authority chains require an explicit matching `--live-arm` and a committed manager-owned anchor in a separate validator root. Until that anchor exists, a live run is intentionally unable to emit `ALLOW`.
- Every retained v2 generation is semantically replayed under the current gate. Retained results are compared evidence, not trusted predecessor state; replay does not claim historical gate semantics were identical.
- Recurrence timed logs and result files must be created outside the source worktree and copied byte-identically into the evidence commit. An in-tree log or result blocks acceptance.
- Git/schema checks do not decide semantic completeness, oracle adequacy, honest execution, nonzero test selection, complete surface enumeration, authorship, or reviewer independence. Those remain manager and independent-review judgements.
- Any compatibility-affecting shape change requires a new schema identifier and fixtures; do not silently widen a schema whose consumers reject additional properties.

Run `python3 docs/development/tools/check_ai_contracts.py` to validate repository packets, routing cases, and cross-file consistency without installing a JSON Schema library.
