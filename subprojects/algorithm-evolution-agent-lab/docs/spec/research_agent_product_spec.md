# Research Agent Product Specification

## REQ-PRODUCT-001: North-star objective

The system shall optimize for sustained research progress toward stronger machine learning algorithms, not for one-shot code generation.

Progress is measured by reproducible experiments, improved baselines, better hypotheses, reduced uncertainty, and a growing body of reusable research knowledge.

## REQ-PRODUCT-002: SOTA pursuit framing

The system shall support ambitious SOTA-seeking programs while treating SOTA as an evidence-backed claim rather than a prompt objective.

The agent may propose SOTA-seeking strategies, but it must label them as hypotheses until benchmark results, baselines, and reproducibility evidence are present.

## REQ-PRODUCT-003: Research program model

The system shall represent a research program as a long-lived object containing goals, constraints, datasets, baselines, hypotheses, experiments, results, open questions, and decision history.

## REQ-PRODUCT-004: Objective decomposition

The agent shall decompose a broad research objective into measurable subgoals, including benchmark choice, metric choice, target improvement, resource budget, and acceptable risk.

## REQ-PRODUCT-005: Hypothesis lifecycle

The system shall track each hypothesis from proposal through planned experiment, execution, analysis, promotion, rejection, mutation, or archival.

## REQ-PRODUCT-006: Experiment automation first

The primary product surface shall be experiment automation. Model architecture ideas, optimization tricks, and benchmark runners are subordinate to the automation loop.

## REQ-PRODUCT-007: Evidence ledger

The system shall maintain an evidence ledger that links claims to experiment runs, datasets, code versions, metrics, logs, and analysis notes.

## REQ-PRODUCT-008: Negative result retention

The system shall retain failed experiments and negative results as first-class knowledge to prevent repeated dead ends.

## REQ-PRODUCT-009: Reproducibility by default

The system shall make every experiment rerunnable by recording configuration, seeds, environment assumptions, dataset references, code version, and command line.

## REQ-PRODUCT-010: Human override

The system shall allow a human researcher to override priorities, freeze a hypothesis, reject a direction, or manually add evidence without corrupting the audit trail.

## REQ-PRODUCT-011: Research memory

The system shall maintain structured research memory that can be queried by hypothesis, dataset, metric, failure mode, algorithm family, and decision rationale.

## REQ-PRODUCT-012: Budget-aware autonomy

The system shall optimize within compute, time, memory, and cost budgets, and it shall prefer cheap uncertainty-reducing experiments before expensive runs.

## REQ-PRODUCT-013: Multi-agent extensibility

The architecture shall allow planner, implementer, runner, analyst, reviewer, and critic roles to be implemented by separate agents or deterministic components.

## REQ-PRODUCT-014: CodeFire-native traceability

The project shall use CodeFire Atom IDs and Trace Links to connect requirements, designs, implementation, tests, experiment logs, and claim evidence.

## REQ-PRODUCT-015: Benchmark neutrality

The system shall support multiple benchmark domains rather than hard-coding one ML task family.

Initial targets may include small tabular datasets, synthetic sequence tasks, optimizer benchmarks, and low-budget neural architecture experiments.

## REQ-PRODUCT-016: Algorithm family tracking

The system shall track algorithm families and variants so that mutations, crossovers, ablations, and regressions can be reasoned about over time.

## REQ-PRODUCT-017: Claim discipline

The system shall distinguish between internal score, benchmark improvement, leaderboard improvement, and external SOTA claim.

## REQ-PRODUCT-018: Research report generation

The system shall generate living research reports summarizing current best candidates, rejected paths, benchmark status, unresolved risks, and recommended next experiments.

## REQ-PRODUCT-019: Safe automation envelope

The system shall execute experiments only through configured runners with explicit resource limits and observable logs.

## REQ-PRODUCT-020: Continual improvement of the agent

The system shall treat the research agent itself as an experimental subject whose planning, scoring, and selection policies can be measured and improved.
