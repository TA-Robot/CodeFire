# Evolution Agent Blueprint

## DES-PRODUCT-001: Research program object model

The core system is organized around a `ResearchProgram` aggregate:

- objective
- benchmark registry
- baseline registry
- hypothesis graph
- experiment queue
- run ledger
- evidence ledger
- claim registry
- decision history

The aggregate is intentionally larger than the current Python MVP. The immediate code can evolve toward this model incrementally.

## DES-PRODUCT-002: Agent role architecture

The automation loop is split into roles:

- Planner: proposes hypotheses and experiments.
- Implementer: modifies code or configuration.
- Runner: executes experiments and collects artifacts.
- Analyst: interprets results.
- Critic: challenges claims and checks validity.
- Curator: updates memory, reports, and decision records.

Early versions may implement all roles in one process. The boundaries still matter because they define future replacement points for LLM agents, deterministic optimizers, and human review.

## DES-PRODUCT-003: Hypothesis graph

Hypotheses form a graph rather than a flat list.

Edges include:

- derived_from
- mutates
- combines
- ablates
- contradicts
- supersedes

This allows the agent to reason about algorithm evolution as lineage, not just isolated trials.

## DES-PRODUCT-004: Candidate selection policy

Candidate selection uses a portfolio policy:

- exploit high-confidence improvements
- explore high-novelty ideas
- run cheap probes to reduce uncertainty
- reserve budget for ablations
- penalize repeated failure modes

The current `score_candidate` function is the first deterministic seed for this policy.

## DES-PRODUCT-005: Agent self-improvement loop

The system stores enough planning and outcome data to evaluate the agent itself.

Examples:

- Which planner prompts produce useful hypotheses?
- Which scoring weights predict real improvement?
- Which critic checks catch false positives?
- Which benchmark choices lead to transferable gains?

The agent can then run meta-experiments on its own policy.

## DES-PRODUCT-006: CodeFire trace model

CodeFire is used as the consistency layer:

- `REQ-*` captures research and product requirements.
- `DES-*` captures architecture and experiment protocol design.
- `CODE-*` captures implementation atoms.
- `TEST-*` captures behavioral checks.
- experiment logs and claim reports are linked as evidence in future iterations.

During early specification expansion, trace completeness is not a blocker. Once the implementation stabilizes, missing trace links should become a blocker again.
