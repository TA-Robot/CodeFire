# Experiment Lifecycle Runbook

This runbook describes the normal path from a research objective to an archived experiment result.

## 1. Frame The Objective

Start with a narrow objective that can be measured in one benchmark or fixture.

Record:

- the target behavior or metric
- the baseline to compare against
- the expected gain or information value
- the budget limit
- the known safety, leakage, overfitting, or leaderboard risks

Avoid broad goals such as "make the agent better" unless they are decomposed into measurable subgoals.

## 2. Create Candidate Work

Use planner output, frontier rankings, or memory search to propose candidates.

Good candidate records include:

- stable candidate identity
- parent or lineage reference
- mechanism summary
- expected gain
- expected cost
- benchmark target
- reason this is not just a duplicate of prior negative results

## 3. Build An Experiment Plan

An experiment plan should be runnable without hidden context.

Minimum plan fields:

- plan id
- hypothesis id or candidate id
- command
- artifact contract
- benchmark and metric
- baseline reference
- expected runtime or cost
- required analysis note

If the command or artifact contract is missing, hold the plan before execution.

## 4. Execute Locally

Local execution should capture artifacts instead of relying on terminal scrollback.

Expected artifacts:

- stdout log
- stderr log
- metrics file
- configuration capture
- run identity record
- artifact manifest

Failed runs still matter. Archive the failure mode, budget cost, and follow-up decision.

## 5. Ingest And Analyze

After execution, ingest metrics into structured result objects and run the analyst layer.

The analysis should answer:

- Did the run complete?
- Did the metric improve against the selected baseline?
- Is the result noisy or underpowered?
- Is the failure caused by budget, benchmark, configuration, implementation, or hypothesis quality?
- What should be retried, revised, archived, or promoted?

## 6. Record Evidence

Evidence must link claims to concrete run artifacts.

Before promotion, check:

- baseline reproduction exists
- configuration fields are complete
- evidence freshness is acceptable
- repeated runs support the result when the metric is noisy
- leakage and contamination checks are clear

## 7. Review And Decide

Use review ledgers and claim gates to keep research claims disciplined.

Possible decisions:

- promote candidate for more evaluation
- run an ablation
- rerun with fixed configuration
- defer due to cost or risk
- archive as negative result
- block release due to missing evidence

## 8. Update Memory

Every completed experiment should update durable memory.

Record:

- accepted findings
- negative results and avoid conditions
- new risks
- changed frontier signals
- decision rationale
- next-cycle planning hints

## Completion Checklist

- The experiment has a stable identity.
- The benchmark and baseline are named.
- Artifacts are captured and referenced.
- Metrics are ingested into structured records.
- Failure modes are recorded, even for unsuccessful work.
- Evidence is connected to any claim or promotion decision.
- The next action is explicit.

