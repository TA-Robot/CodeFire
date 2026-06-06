# Agent Architecture

## DES-AGENT-001: Evolution loop

The first implementation uses a deterministic loop:

1. Accept an `ExperimentGoal`.
2. Convert candidate ideas into `Hypothesis` records.
3. Create an `ExperimentPlan` for each hypothesis.
4. Score completed or simulated candidates.
5. Select the highest-ranked next candidate.

This is intentionally simple so later experiments can replace each stage with LLM, Bayesian optimization, population search, or benchmark runners independently.

## DES-SCORING-001: Candidate scoring

Candidate score is a weighted sum of quality, novelty, and confidence, penalized by estimated cost.

The score is not a SOTA claim. It is a scheduling priority for the next experiment.

## DES-SAFETY-001: SOTA claim gate

The agent separates experiment scheduling from external claims.

A candidate can be marked as SOTA-ready only after reproducible evidence, at least one baseline comparison, and at least one benchmark result are present.

## DES-PLANNER-001: Planner boundary

Hypothesis generation is behind a `Planner` interface.

The default planner is deterministic so tests and examples are stable. The Codex planner invokes `codex exec` with a JSON-only prompt, parses hypotheses, and returns the same domain model as the deterministic planner.

This keeps AI generation useful without coupling the evolution loop to one provider or to free-form text.
