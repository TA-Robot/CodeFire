# Agent Requirements

## REQ-AGENT-001: Research loop automation

The agent shall maintain a loop that converts a research goal into hypotheses, experiment plans, results, and ranked next candidates.

## REQ-SCORING-001: Multi-objective candidate scoring

The agent shall rank candidate algorithm ideas using quality, novelty, confidence, and cost instead of a single raw metric.

## REQ-SAFETY-001: SOTA claim discipline

The agent shall not mark a candidate as SOTA without benchmark definition, baseline comparison, and reproducible evidence.

## REQ-PLANNER-001: Codex-backed hypothesis generation

The agent shall support Codex as an AI planning backend that proposes machine learning algorithm hypotheses from a structured research goal.

Codex output shall be parsed into normal `Hypothesis` records so downstream scoring, scheduling, and evidence tracking remain provider-independent.
