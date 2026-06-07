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

## REQ-AUTO-028: Experiment iteration plan

The system shall convert triage decisions into a bounded next-iteration plan that separates executable experiments, promotion or rejection review items, and deferred work while respecting remaining budget and active-run capacity.

## REQ-AUTO-029: Experiment risk register

The system shall maintain a risk register for experiment campaigns, prioritize open risks by likelihood, impact, and severity, and surface blocker risks that must be mitigated before promotion or external claims.

## REQ-AUTO-030: Experiment evidence pack

The system shall build a compact evidence pack for candidate promotion or external review, combining improvement signal, confidence, replication count, evidence freshness gaps, open blocker risks, missing artifacts, and mitigation checklist into a deterministic readiness decision.

## REQ-AUTO-031: Claim review queue

The system shall turn evidence packs and reviewer objections into a deterministic review queue that prioritizes blocked or revision-needed claims before approval-ready claims and explains the required next action for each claim.

## REQ-AUTO-032: Claim audit trail

The system shall generate a deterministic audit trail for candidate claims that records evidence readiness, review decisions, external-claim eligibility, and required follow-up so that promotion decisions can be reconstructed without reading raw logs.

## REQ-AUTO-033: Claim release gate

The system shall evaluate whether a candidate claim may be externally released by combining evidence readiness, review decision, audit terminal status, and required release artifacts into an allow, hold, or reject gate decision.

## REQ-AUTO-034: Research frontier map

The system shall synthesize a ranked research frontier map from baseline gaps, negative results, novelty reviews, open risks, budget pressure, and challenge objectives. Each frontier item shall identify the mechanism family to explore, the expected evidence gain, the primary risk, the next experiment action, and the reason it is worth pursuing before lower-ranked alternatives.

## REQ-AUTO-035: Frontier experiment plan drafting

The system shall convert executable research frontier items into bounded experiment plan drafts with a hypothesis, benchmark target, baseline, metric, runner command, artifact contract, and analysis criteria. Deferred, risk-mitigation, or redesign frontier items shall not become executable plans until their blocking reason is resolved.

## REQ-AUTO-036: Frontier feedback integration

The system shall integrate completed run outcomes back into research frontier signals. Feedback shall update baseline gap, expected information gain, novelty pressure, negative-result overlap, blocker risks, and rationale without mutating the original signal. The update shall be deterministic, bounded, and conservative so failed, regressed, over-budget, or blocked runs reduce executable opportunity while replicated improvements increase priority for follow-up frontier planning.

## REQ-AUTO-037: Frontier drift report

The system shall summarize how research frontier priorities drift across long campaigns. A drift report shall compare previous and current ranked frontier items, identify new, removed, rising, falling, and stable frontiers, expose score and action changes, and recommend follow-up actions for high-impact drift without inspecting raw experiment logs.

## REQ-AUTO-038: Research cycle retrospective

The system shall summarize completed research cycles into policy adjustment recommendations. A retrospective shall combine completed run counts, improvements, regressions, failures, blockers, budget pressure, frontier drift, and evidence-ready claims into deterministic recommendations for the next planning cycle. The retrospective shall distinguish whether the next cycle should increase exploration, consolidate promising candidates, reduce cost, mitigate risks, archive stale paths, or keep the current policy.

## REQ-AUTO-039: Retrospective planning summary

The system shall render a research cycle retrospective into a compact planning summary suitable for the next docs-first development cycle. The summary shall include cycle IDs, priority, key rates, recommendations, and rationale in deterministic order so the next planning document can be generated without rereading raw run logs.
