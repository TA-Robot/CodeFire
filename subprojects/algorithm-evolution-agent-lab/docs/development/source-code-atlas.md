# Algorithm Evolution Agent Source Code Atlas

この文書は、`algorithm-evolution-agent-lab` のsourceを開く前に、Python modulesの実装像を頭に描けるようにするためのsource atlasである。

このプロジェクトは「機械学習の新しいアルゴリズムでSOTAを狙うAIエージェント」そのものではなく、そのための実験自動化・評価・証跡・安全側の管理部品を小さな標準ライブラリ中心のPython moduleとして積み上げている。

## 1. Source Shape

```text
src/evoagent/
  models.py              core dataclasses for hypothesis, plan, run, result
  planner.py             planner interface
  static_planner.py      deterministic fallback planner
  codex_planner.py       Codex-backed JSON planner adapter
  loop.py                EvolutionAgent orchestration

  runner.py              local command execution and artifact capture
  ingestion.py           metrics artifact -> ExperimentResult
  analysis.py            run result -> analysis note
  reporting.py           living research report markdown
  report_bundle.py       SOTA claim bundle files

  challenge.py           challenge definition loader and validator
  baselines.py           baseline registry
  baseline_reproduction.py
  benchmark_domains.py
  benchmarks.py
  fixtures.py

  evolution.py           mutation/crossover/ablation operators
  scheduler.py           portfolio selection
  queueing.py            ready experiment queue
  batch_planning.py      budgeted batch builder
  budgeting.py           information per cost planner
  triage.py              action-ranked experiment triage board
  iteration.py           active/review/deferred iteration planner
  frontier.py            frontier ranking, plan draft, feedback, drift report
  retrospective.py       cycle-level policy adjustment recommendations and Markdown planning summary

  evidence.py
  evidence_pack.py
  evidence_freshness.py
  reproduction.py
  configuration.py
  artifacts.py
  run_identity.py

  safety.py
  claims.py
  claim_review_queue.py
  claim_audit.py
  claim_release.py
  review.py
  risk.py
  promotion.py
  stop_criteria.py

  memory.py
  negative_results.py
  novelty.py
  external_validity.py
  leakage.py
  contamination.py
  overfitting.py
  regression.py
  learning_curve.py
  failures.py
  limitations.py

  traceability.py
  automation_policy.py
  automation_surface.py
  roles.py
  dependencies.py
  compute.py
  metrics.py
  statistics.py
  policy_eval.py
  progress.py
  program.py
  sota_framing.py
```

基本設計は「巨大な万能Agent class」ではなく、immutable dataclassを入力し、deterministicなdecision objectを返す小さなサービス群である。外部I/Oを持つのは主にrunner、challenge loader、report writersで、その他は純粋関数に近い。

## 2. Core Data Spine

`models.py` が中心で、ほぼ全moduleは次のspineに接続する。

```text
CandidateAlgorithm
  -> ExperimentPlan
     -> ExperimentRun
        -> ExperimentResult
           -> analysis/evidence/report/decision
```

周辺moduleの役割:

| Layer | Modules | Input | Output |
|---|---|---|---|
| propose | `planner.py`, `static_planner.py`, `codex_planner.py`, `evolution.py` | objective, memory, previous candidates | `CandidateAlgorithm` |
| plan | `loop.py`, `scheduler.py`, `queueing.py`, `batch_planning.py`, `budgeting.py`, `frontier.py` | candidates, budget, risk, frontier signals | `ExperimentPlan` or plan draft |
| execute | `runner.py`, `fixtures.py` | plan command and artifact contract | `ExperimentRun` |
| ingest | `ingestion.py`, `metrics.py`, `statistics.py` | artifacts/results | normalized result summaries |
| judge | `analysis.py`, `promotion.py`, `triage.py`, `iteration.py`, `risk.py` | runs, results, evidence, blockers | decisions and next actions |
| learn | `frontier.py`, `policy_eval.py`, `retrospective.py` | ranked frontiers, policy outcomes, cycle summaries | policy adjustment recommendations |
| govern | `claims.py`, `safety.py`, `review.py`, `claim_*`, `reproduction.py` | claim/evidence/review inputs | claim readiness and release gates |
| remember | `memory.py`, `negative_results.py`, `evidence.py`, `run_identity.py`, `program.py` | events and records | searchable durable context |
| explain | `reporting.py`, `report_bundle.py`, `limitations.py`, `progress.py` | program state | markdown/report artifacts |

## 3. Main Orchestration

`EvolutionAgent` is intentionally small.

```text
EvolutionAgent.run_iteration(objective, budget)
  -> planner.propose(objective)
  -> score candidates
  -> choose best or schedule portfolio
  -> create ExperimentPlan
  -> runner executes when available
  -> result is recorded into ResearchProgram
```

The agent should not hide research governance. Promotion, release, claim review, risk, frontier drift, and evidence readiness remain separate modules so that CodeFire can trace requirement/design/code/test atoms independently.

## 4. Frontier Subsystem

`frontier.py` is the current high-value planning module. Its internal shape is:

```text
ResearchFrontierSignal
  raw compact signal:
    baseline gap
    expected information gain
    novelty risk
    blocker risk count
    budget estimate
    negative result overlap
    challenge objective match

ResearchFrontierMap.rank(signals, remaining_budget)
  -> validate each signal
  -> frontier_score(signal, remaining_budget)
  -> choose_frontier_action(signal, score, remaining_budget)
  -> ResearchFrontierItem
  -> sorted tuple by score and stable id

FrontierExperimentPlanner.draft_plans(frontiers, capacity, default_command)
  -> skip non-runnable actions
  -> create bounded FrontierPlanDraft
  -> include rationale and expected artifact contract

FrontierFeedbackIntegrator.apply(signals, feedback)
  -> validate feedback
  -> apply conservative score deltas
  -> clamp signal values
  -> return FrontierFeedbackUpdate

FrontierDriftReporter.report(previous, current)
  -> compare frontier_id maps
  -> classify new/removed/rising/falling/stable/action_changed
  -> assign severity
  -> emit recommendation and summary

ResearchCycleRetrospective.summarize(signals)
  -> aggregate cycle rates, budget pressure, and risk pressure
  -> emit policy recommendations and rationale

RetrospectivePlanningSummary.render_markdown(report)
  -> compact recommendation summary for docs-first planning

ResearchCyclePlanSynthesizer.synthesize(report, active_capacity, remaining_budget)
  -> map recommendations into mitigation/active/review/archive/deferred lanes
  -> cap active work by capacity
  -> preserve deferred overflow instead of dropping work

ResearchCyclePlanMarkdown.render(plan)
  -> write source cycles, priority, capacity, and budget
  -> group plan items by lane
  -> emit deterministic Markdown sections for review

ResearchCyclePlanLint.lint(plan)
  -> validate source cycle, capacity, budget, and item fields
  -> detect active lane and budget overcommitment
  -> return deterministic blocker/warning findings without mutating the plan

ResearchCyclePlanLintMarkdown.render(report)
  -> summarize status, blocker count, and warning count
  -> group findings into severity sections
  -> emit field path and message lines for review logs

ResearchCyclePlanningPacketBuilder.build(report, active_capacity, remaining_budget)
  -> synthesize the next-cycle plan
  -> lint the plan
  -> render plan Markdown and lint Markdown
  -> return a synchronized planning packet for docs-first handoff

ResearchCyclePlanningPacketMarkdown.render(packet)
  -> summarize packet-level source cycles, priority, status, and budget
  -> embed demoted plan Markdown
  -> embed demoted lint Markdown
  -> return one deterministic cycle handoff document

ResearchCyclePlanningPacketManifestBuilder.build(packet)
  -> render the combined planning packet Markdown
  -> hash combined packet, plan, and lint artifacts
  -> reject unsafe handoff paths
  -> return deterministic artifact metadata without writing files

ResearchCyclePlanningPacketManifestMarkdown.render(manifest)
  -> summarize source cycles, status, and artifact count
  -> preserve manifest entry order
  -> render artifact path, hash, and byte count as an audit table

ResearchCyclePlanningPacketManifestVerifier.verify(manifest, artifact_contents)
  -> compare manifest entries against provided UTF-8 artifact contents
  -> report missing artifact, hash mismatch, and byte count mismatch findings
  -> return deterministic verification state without reading or writing files

ResearchCyclePlanningPacketManifestVerificationMarkdown.render(verification)
  -> summarize verification status and finding count
  -> render explicit none line for clean verification
  -> render path-scoped drift findings for audit logs

ResearchCyclePlanningHandoffBundleBuilder.build(report, active_capacity, remaining_budget)
  -> build packet, artifact contents, manifest, manifest Markdown, verification, and verification Markdown
  -> validate artifact paths through manifest generation
  -> return one deterministic handoff object without filesystem writes

ResearchCyclePlanningHandoffBundleMarkdown.render(bundle)
  -> summarize packet, manifest, and verification status
  -> render artifact path and byte-size index
  -> expose included audit documents without opening each artifact

ResearchCyclePlanningHandoffBundleSummary.summarize(bundle)
  -> return packet, manifest, and verification status as plain data
  -> return artifact path, byte size, and SHA-256 digest for each artifact
  -> expose audit document availability and finding count for automation routing

ResearchCyclePlanningHandoffReadinessGate.evaluate(bundle)
  -> return ready/status/blocker/warning routing data for a handoff bundle
  -> block drifted, incomplete, empty, or missing-audit handoffs before storage
  -> preserve lint warnings as non-blocking routing metadata

ResearchCyclePlanningHandoffReadinessMarkdown.render(readiness)
  -> render the already-computed readiness decision for reviewers
  -> include ready/status, artifact count, finding count, blockers, and warnings
  -> avoid bundle inspection so automation and reviewer logs share one decision source

ResearchCyclePlanningHandoffReviewPacketBuilder.build(bundle)
  -> compose an existing handoff bundle with summary, readiness, and readiness Markdown
  -> keep the bundle as the single planning and verification source of truth
  -> avoid filesystem writes and avoid recomputing planning or manifest verification

ResearchCyclePlanningHandoffReviewPacketMarkdown.render(packet)
  -> render a final reviewer-facing Markdown document from the already-built review packet
  -> include source cycles, readiness, statuses, artifact digest table, blockers, warnings, and audit availability
  -> avoid bundle rebuilds, readiness recomputation, manifest verification, and filesystem reads

ResearchCyclePlanningHandoffReviewPacketArtifactBuilder.build(packet)
  -> package review packet Markdown, readiness Markdown, manifest Markdown, and verification Markdown as handoff artifacts
  -> validate all output paths with the shared relative POSIX manifest path rule
  -> avoid filesystem writes and copy already-computed audit Markdown from the packet and bundle

ResearchCyclePlanningHandoffReviewPacketArtifactManifestBuilder.build(packet, artifacts)
  -> build a storage verification manifest for packaged review packet artifacts
  -> preserve source cycles from the packet summary
  -> reuse shared manifest entry hashing and path validation without reading or writing files

ResearchCyclePlanningHandoffReviewPacketArtifactManifestMarkdown.render(manifest)
  -> render packaged review packet artifact manifests as reviewer-facing Markdown tables
  -> include source cycles, manifest status, artifact count, path, hash, and byte count
  -> reuse the shared manifest Markdown renderer without reading artifact contents
```

Important implementation details:

- Ranking is deterministic: ties are broken by stable frontier id.
- Budget checks are conservative: over-budget frontiers are held or deferred.
- Feedback updates are bounded: values are clamped and do not explode over repeated runs.
- Drift reporting compares snapshots, not raw logs, so long campaigns can be summarized cheaply.
- Cycle plan synthesis converts retrospective prose into lane-level work while keeping budget and active capacity explicit.
- Cycle plan Markdown rendering formats the bounded plan object; it does not re-run retrospective analysis or inspect raw logs.
- Cycle planning packet Markdown embeds the already-rendered plan and lint documents so reviewers inspect the same objects automation will execute.
- Cycle planning packet manifests hash the reviewed packet artifacts before any future writer persists them.
- Cycle planning packet manifest Markdown formats manifest metadata only; it does not recompute hashes.
- Cycle planning packet manifest verification compares persisted content to the manifest and remains independent from the storage adapter.
- Cycle planning packet manifest verification Markdown formats the computed drift findings without rerunning verification.
- Cycle planning handoff bundle building composes existing planning components and adds no new planning policy.
- Cycle planning handoff bundle Markdown indexes the built bundle; it does not rebuild or verify it.

## 5. Experiment Automation Surface

The automation pipeline can be imagined as:

```text
ExperimentPlan
  -> SafeAutomationPolicy.check
  -> LocalExperimentRunner.run
       subprocess.run(..., timeout)
       capture stdout/stderr
       capture declared artifacts
       create ExperimentRun
  -> ResultIngestor.ingest
       read metrics.json
       normalize metric direction/scale
       create ExperimentResult
  -> RunAnalyst.analyze
       classify success/failure
       propose next action
```

The code intentionally uses the standard library where possible. That keeps dogfooding fast and isolates CodeFire issues from dependency noise.

## 6. Evidence And Claim Governance

Claim flow is deliberately stricter than experiment flow.

```text
ExperimentRun + artifacts
  -> EvidenceLedger
  -> ExperimentEvidencePackBuilder
  -> ClaimReviewQueue
  -> ClaimAuditTrailBuilder
  -> ClaimReleaseGate
  -> SotaReportBundleGenerator
```

The source implementation should remain conservative:

- weak evidence becomes review or blocked, not an external SOTA claim;
- stale benchmark/dataset/protocol evidence is flagged before release;
- reviewer objections remain visible until explicitly resolved;
- release requires evidence, review, audit trail, and required artifacts.

## 7. Evaluation Memory

Memory modules prevent the agent from repeating low-value work.

| Module | Stores or detects |
|---|---|
| `ResearchMemoryIndex` | tagged memory records searchable by kind, tag, and text |
| `NegativeResultArchive` | failed paths and avoid conditions |
| `RegressionDetector` | candidate worse than accepted predecessor |
| `LearningCurveAnalyzer` | plateau, regression, instability over attempts |
| `NoveltyReviewer` | overlap with known algorithm families |
| `ExternalValidityEvaluator` | improvement transfer across benchmark domains |
| `LeakageChecker` | split overlap and metric misuse |
| `AntiOverfittingMonitor` | repeated hidden leaderboard probing |
| `ContaminationSourceRegistry` | dataset/artifact contamination overlap |

These modules are small by design. Each should be readable as:

```text
dataclass input records
  -> validate
  -> deterministic score/finding
  -> sorted tuple output
```

## 8. Tests As Source Index

Every implemented module has a matching `tests/test_*.py` file. The tests are the best executable index:

| Source | Test |
|---|---|
| `frontier.py` | `tests/test_frontier.py` |
| `claim_release.py` | `tests/test_claim_release.py` |
| `claim_audit.py` | `tests/test_claim_audit.py` |
| `claim_review_queue.py` | `tests/test_claim_review_queue.py` |
| `evidence_pack.py` | `tests/test_evidence_pack.py` |
| `triage.py` | `tests/test_triage.py` |
| `iteration.py` | `tests/test_iteration.py` |
| `runner.py` | `tests/test_runner.py` |
| `program.py` | `tests/test_program.py` |
| `traceability.py` | `tests/test_traceability.py` |

When adding a module, add its test in the same naming pattern. When adding a CodeFire atom, keep requirement, design, code, and test IDs connected in `codefire.links.yaml`.

## 9. Change Recipes

### Add a new planning primitive

1. Add requirement in `docs/spec/experiment_automation_requirements.md`.
2. Add design in `docs/design/experiment_platform_design.md`.
3. Add source module or extend the nearest planning module.
4. Add a focused `tests/test_*.py` case.
5. Add CodeFire links for requirement/design/code/test atoms.
6. Run the full unittest suite and commit through CodeFire.

### Add a new governance gate

1. Define the exact blocker/review/approve states in spec.
2. Keep the gate input as a frozen dataclass.
3. Return a result object with decision, reasons, and checklist.
4. Keep ordering deterministic.
5. Test negative and positive paths.

### Add external execution

Do not hide it inside the planner. Add a runner abstraction or policy-gated adapter, then make the execution boundary explicit in docs and tests.

## 10. What The Source Should Feel Like

Before opening code, a reader should expect:

- one module per research-management concern;
- frozen dataclasses for records and result objects;
- deterministic sort order for every queue, ranking, and checklist;
- no hidden global mutable state;
- limited filesystem/subprocess I/O, concentrated in runner/report/challenge modules;
- tests that describe the behavior at module boundary level;
- CodeFire trace links that connect requirements, design, code, and tests.

If an implementation no longer matches this source shape, update this atlas before extending the feature.
