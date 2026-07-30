# AI Development Contracts

These JSON Schemas are the portable contract for AI-assisted CodeFire development. JSON is authoritative; Markdown packets are human views and may be generated from the same data.

| Schema | Purpose |
|---|---|
| `task-packet.v1.schema.json` | Bounded objective, context, read/write scope, risk route, invariants, validation, and handoff. |
| `context-manifest.v1.schema.json` | Resolved and content-addressed context inputs for reproducible delegation. |
| `review-packet.v1.schema.json` | Patch identity, claims, independent evidence, findings, and an explicit decision. |
| `development-flow-record.v1.schema.json` | Batch KGI status, lane timing, KPI measurements, incidents, and immutable active-log prefixes. |

Contract rules:

- Repository paths are relative to the repository root. A runtime validator must reject absolute paths and `..` traversal even where JSON Schema implementations cannot express that portably.
- Every risk dimension is scored `0..2`; the sum is `0..12`.
- A mandatory-Ultra reason overrides the numeric threshold.
- Packet and patch hashes use SHA-256 over canonical UTF-8 JSON or canonical patch bytes.
- A passing schema check proves shape, not truth. Semantic validators recompute task/context digests and bind reviews to task scope and required validation; repository evidence remains mandatory.
- Any compatibility-affecting shape change requires a new schema identifier and fixtures; do not silently widen a schema whose consumers reject additional properties.

Run `python3 docs/development/tools/check_ai_contracts.py` to validate repository packets, routing cases, and cross-file consistency without installing a JSON Schema library.
