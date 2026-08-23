# CodeFire Strategy Hub

Status: canonical

Last reviewed: 2026-07-11

Owner: Product and Engineering Steward

Update trigger: a strategy decision changes status, a roadmap stage passes its exit gate, or evidence falsifies an accepted assumption.

This directory is the canonical bridge between non-authoritative exploration and executable product/development work. It contains only promoted decisions, explicitly bounded hypotheses, and named open decisions. Raw alternatives and experiment designs remain under `../exploration/`.

## Read Order

1. `engineering-principles.md` — rules that every implementation and review must preserve.
2. `product-strategy.md` — product core, market hypothesis, boundaries, and adoption ladder.
3. `modernization-roadmap.md` — ordered stages and evidence gates; this is the active strategic roadmap.
4. `release-and-format-roadmap.md` — independent version axes and public-release prerequisites.
5. `decision-register.md` — accepted, experimental, pending-owner, and rejected decisions with review triggers.

## Current Strategic Position

- CodeFire remains a consistency-first, version-control-style system with sealed evidence-bearing history. It is not Git-compatible and does not become an agent runner.
- The leading external category hypothesis is an AI-change consistency and evidence protocol/control plane. This is a positioning hypothesis until independent users validate it.
- AI orchestration stays outside the product core. The current Rust top-level JSON envelope, capability discovery, and evidence are the starting integration surface; stable public payload schemas and conformance profiles remain Stage 2 deliverables.
- Development uses GPT-5.6 Sol with project-level Medium/High/Ultra routing. Model selection is an explicit runtime policy, while role and risk semantics remain vendor-independent.
- Deterministic checks outrank model claims. A model can propose, implement, and review; it cannot self-certify a failed or unexecuted gate as passing.
- The immediate program is repository hygiene and contract/eval infrastructure. Execution is the five-ahead board in `../milestones.md`; remaining review-portfolio issues are parked, not an active burn-down.
- Codex orchestrates. Claude Opus 5 and Grok 4.6 do individual and 30-minute audit work.

## Authority and Promotion

| Material | Authority |
|---|---|
| This directory | Accepted strategy, bounded hypotheses, ordered gates. |
| `../exploration/` | Non-authoritative questions, alternatives, experiments, and falsifiers. |
| `../ai/` and `.codex/` | Executable AI-development policy and runtime configuration. |
| Product docs, ADRs, schemas, tests | Product behavior and compatibility contract. |
| `../README.md` and `../project-management.md` | Current development state and management process. |

A strategy decision does not silently change product behavior. Product-semantic decisions must also be promoted to the relevant product document, ADR, schema, and regression evidence in one reviewable change.

## Review Cadence

- Review accepted decisions when their stated trigger occurs, not on a calendar alone.
- Expire an experiment when its evidence deadline passes; do not leave it as implied policy.
- Remove rejected alternatives from active roadmaps while keeping the concise reason and reconsideration trigger in the decision register.
- Keep only one active roadmap: `modernization-roadmap.md`. Exploration backlogs are inputs, not competing task boards.
