# Research Cycle Planning Runbook

This runbook describes how to move from retrospective findings to the next cycle plan.

## Inputs

Collect these inputs before planning:

- current frontier signals
- completed experiment results
- negative result archive entries
- open objections from claim review
- budget consumed and remaining
- compute failures and timeout patterns
- risk register updates
- benchmark or baseline drift
- CodeFire status for the repository

## Retrospective Pass

The retrospective layer turns cycle signals into policy recommendations.

Classify each recommendation as one of:

- increase exploration
- reduce risky work
- prioritize reproduction
- tighten claim gates
- spend budget on high-information runs
- archive or defer low-value ideas

The goal is not to maximize the number of active tasks. The goal is to choose the smallest set of tasks that can change the next decision.

## Planning Lanes

Use these lanes when building the next plan:

| Lane | Meaning |
|---|---|
| active | Work that fits current capacity and budget |
| review | Work needing human or reviewer decision before execution |
| mitigation | Risk, leakage, overfitting, or evidence hardening work |
| archive | Work that should be preserved as negative or completed context |
| deferred | Valid work that does not fit current capacity |

Active capacity is a hard limit. Overflow should move to deferred, not disappear.

## Budget Rules

- Do not schedule active work above remaining budget.
- Prefer high information gain per cost when budget is tight.
- Keep reproduction and validation budget separate from novelty-seeking budget.
- Count failed attempts against budget.
- Treat repeated leaderboard probing as a risk, not as free exploration.

## Plan Quality Checks

Before committing a cycle plan, check:

- each active item has a command or a concrete next action
- each item has a reason tied to a frontier, risk, or retrospective recommendation
- active item count is within capacity
- estimated cost is within remaining budget
- blocked work is not mixed into active work
- deferred work has a reason rather than being silently dropped

## Handoff Document

A good planning handoff contains:

- source cycle ids
- plan priority
- active capacity
- remaining budget
- lane-grouped items
- lint findings
- artifact manifest hashes
- readiness status

The handoff should be deterministic enough to review in CodeFire diffs.

