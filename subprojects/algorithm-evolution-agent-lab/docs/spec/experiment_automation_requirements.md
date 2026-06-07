# Experiment Automation Requirements

## REQ-AUTO-001: Experiment plan schema

An experiment plan shall include hypothesis, benchmark, baseline, metric, expected outcome, budget, runner command, artifacts to collect, and analysis criteria.

## REQ-AUTO-002: Runner interface

The system shall expose a runner interface that can execute local commands first and later support containers, job queues, and remote accelerators.

## REQ-AUTO-003: Run identity

Each experiment run shall receive a stable run ID that is used across logs, metrics, artifacts, reports, and evidence links.

## REQ-AUTO-004: Configuration capture

The runner shall capture the exact experiment configuration, including hyperparameters, seed, dataset version, benchmark split, and code reference.

## REQ-AUTO-005: Artifact collection

The runner shall collect declared artifacts such as metrics JSON, stdout, stderr, model summaries, plots, failure traces, and environment metadata.

## REQ-AUTO-006: Metric normalization

The system shall normalize metric direction and scale so that different benchmarks can be compared by a common scoring layer without losing raw values.

## REQ-AUTO-007: Baseline registry

The system shall maintain a registry of baselines for each benchmark, including source, command, expected metric, and confidence level.

## REQ-AUTO-008: Ablation protocol

The system shall support ablation experiments that isolate which part of a candidate caused improvement or regression.

## REQ-AUTO-009: Replication protocol

The system shall require repeated runs before promoting noisy improvements above configurable thresholds.

## REQ-AUTO-010: Failure classification

The analyst shall classify failures into categories such as implementation error, invalid hypothesis, insufficient budget, benchmark mismatch, instability, or inconclusive result.

## REQ-AUTO-011: Experiment queue

The system shall maintain a queue of proposed experiments with priority, expected information gain, estimated cost, and dependency constraints.

## REQ-AUTO-012: Stop criteria

The runner shall support stop criteria based on timeout, maximum cost, metric plateau, invalid output, or safety policy violation.

## REQ-AUTO-013: Result ingestion

The system shall ingest completed run artifacts into structured `ExperimentResult` records.

## REQ-AUTO-014: Analysis note generation

The analyst shall produce a concise analysis note for each run, including observed result, comparison against baseline, confidence, caveats, and recommended next action.

## REQ-AUTO-015: Candidate mutation

The planner shall generate follow-up candidates by mutating promising hypotheses and by simplifying or ablation-testing uncertain hypotheses.

## REQ-AUTO-016: Portfolio balance

The scheduler shall balance exploitative follow-ups on strong candidates with exploratory trials of novel candidates.

## REQ-AUTO-017: Regression detection

The system shall detect when a new candidate regresses against previously accepted candidates on shared benchmarks.

## REQ-AUTO-018: Experiment dependency graph

The system shall model dependencies between runs, including prerequisites, repeated trials, ablations, and benchmark unlocks.

## REQ-AUTO-019: Research decision records

Major research decisions shall be recorded with evidence, rejected alternatives, and expected follow-up work.

## REQ-AUTO-020: Continuous benchmark refresh

The benchmark registry shall be refreshable as datasets, baselines, or evaluation protocols change.

## REQ-AUTO-021: Campaign execution state

The system shall track research campaign milestones, completion criteria, blocked milestones, and next milestone recommendations from runs, evidence, and results.

## REQ-AUTO-022: Budgeted experiment batch planning

The scheduler shall assemble ready experiments into executable batches that respect total cost, maximum batch size, and benchmark diversity constraints.

## REQ-AUTO-023: Retry and escalation planning

The analyst shall convert failed, unstable, or inconclusive runs into retry, mutate, archive, or human-review recommendations with explicit rationale.

## REQ-AUTO-024: Evidence freshness audit

The system shall detect when evidence references an older benchmark, dataset, or protocol version and require refresh before strong claims.

## REQ-AUTO-025: Learning curve diagnosis

The analyst shall summarize metric trajectories across repeated attempts and detect plateau, regression, or unstable progress.

## REQ-AUTO-026: Candidate promotion policy

The system shall decide whether a candidate should be promoted, replicated, mutated, rejected, or held based on improvement size, confidence, replication count, and open blockers.

## REQ-AUTO-027: Experiment triage board

The system shall combine queued experiment value, promotion policy, learning curve diagnosis, open blockers, and remaining budget into a ranked triage board of next research actions.
