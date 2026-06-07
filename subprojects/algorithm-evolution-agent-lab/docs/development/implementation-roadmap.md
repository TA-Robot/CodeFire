# Implementation Roadmap

## Phase 0: CodeFire-managed specification base

- Build rich requirements and design documents.
- Keep unit tests passing.
- Use CodeFire commits for every coherent research-management change.

## Phase 1: Durable research objects

- Add `ResearchProgram`.
- Add hypothesis IDs and lineage.
- Add experiment plan IDs.
- Add run and evidence ledgers.

## Phase 2: Local experiment runner

- Execute configured local commands.
- Capture logs and metrics.
- Persist run artifacts.
- Ingest results into structured records.

## Phase 3: Planner and analyst agents

- Add deterministic planner interface.
- Add LLM planner adapter behind an interface.
- Add analyst that writes structured run notes.
- Add critic that blocks premature claims.

## Phase 4: Benchmark registry

- Add benchmark schema. (done)
- Add baseline schema.
- Add small local benchmark fixtures. (done)
- Add repeated-run confidence handling. (done)

## Phase 5: Algorithm evolution strategies

- Add mutation and crossover operators for candidate algorithms. (done)
- Add ablation generator. (done)
- Add portfolio scheduler. (done)
- Add meta-evaluation of the agent policy. (done)

## Phase 6: SOTA challenge mode

- Add challenge definition files. (done)
- Add claim readiness promotion. (done)
- Add report bundle generation. (done)
- Add reproduction checklist. (done)

## Phase 7: Evaluation memory hardening

- Add regression detection against accepted candidates. (done)
- Add experiment dependency graph. (done)
- Add research decision review ledger. (done)
- Add metric normalization surface for cross-benchmark scoring. (done)
- Add compute accounting across failed and successful runs. (done)
- Add leakage and metric misuse checks. (done)
- Add anti-overfitting monitor for repeated leaderboard probes. (done)
- Add novelty review against known algorithm families. (done)
- Add external validity evaluator across benchmark domains. (done)
- Add objective decomposition into measurable subgoals. (done)
- Add human override audit ledger. (done)
- Add research memory index for tag/kind/text queries. (done)
- Add negative result archive with avoid-condition lookup. (done)
- Add budget-aware candidate planning. (done)

## Phase 8: Claim and safety hardening

- Add claim type discipline classifier. (done)
- Add safe automation policy for commands, cost, and artifacts. (done)
- Add baseline reproduction ledger for claim gates. (done)
- Add confidence interval estimator for repeated runs. (done)
- Add limitation section generator from failures, gaps, and objections. (done)
- Add benchmark domain catalog for benchmark neutrality. (done)
- Add role registry for component extensibility. (done)

## Phase 9: Frontier-driven research iteration

- Add research frontier map from baseline gaps, negative results, novelty, risks, budget, and challenge objectives. (done)
- Add frontier-to-experiment-plan conversion. (done)
- Add frontier feedback after run outcomes. (planned)
- Add frontier drift report for long campaigns. (planned)
