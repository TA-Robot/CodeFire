# Development History

| Date | Area | Change | Evidence |
|---|---|---|---|
| 2026-06-04 | Project | 実験自動化AIエージェント用リポジトリを作成した | initial scaffold |
| 2026-06-04 | Specification | SOTA追求型の実験自動化AIエージェント仕様を拡張した | CodeFire-managed spec/design docs |
| 2026-06-04 | Agent | Codex-backed planner adapter とCLI切替を追加した | 7 tests pass; live `--planner codex` selected `Lightweight feature preprocessing ensemble for regularized tree and linear models` |
| 2026-06-04 | Runner | local experiment runner とartifact captureを追加した | 9 tests pass; `TEST-local-runner-captures-artifacts` |
| 2026-06-04 | Research Model | `ResearchProgram` aggregate と `EvidenceLedger` を追加した | 12 tests pass; `TEST-research-program-records-runs-and-evidence` |
| 2026-06-04 | Reports | `LivingResearchReport` を追加し、候補、run、evidence、risk、decisionをMarkdownへ集約できるようにした | 14 tests pass; `TEST-living-research-report-summarizes-program` |
| 2026-06-04 | Research Memory | `BaselineRegistry` を追加し、benchmark/metric別のbaseline登録とbest baseline選択を実装した | 17 tests pass; `TEST-baseline-registry-selects-best` |
| 2026-06-04 | Research Model | hypothesis / experiment plan に安定IDを追加し、`ResearchProgram` で仮説lineage edgeを記録できるようにした | 20 tests pass; `TEST-durable-research-identities-and-lineage` |
| 2026-06-04 | Ingestion | `ResultIngestor` を追加し、runnerが収集したmetrics artifactを `ExperimentResult` に変換できるようにした | 22 tests pass; `TEST-result-ingestor-parses-metrics-artifact` |
| 2026-06-04 | Analysis | `RunAnalyst` を追加し、成功/失敗runからstructured analysis noteと次アクションを生成できるようにした | 24 tests pass; `TEST-run-analyst-writes-structured-note` |
| 2026-06-05 | Benchmark | versioned benchmark registryとrepeated-run confidence summaryを追加した | `TEST-benchmark-registry-refreshes-versioned-definitions`; `TEST-repeated-run-summary-requires-signal-over-noise` |
| 2026-06-05 | Benchmark | 標準ライブラリだけで動く `toy-tabular` local benchmark fixture とmodule runnerを追加した | `TEST-local-benchmark-fixture-produces-artifacts`; `TEST-local-benchmark-fixture-runs-through-runner` |
| 2026-06-05 | Evolution | hypothesis mutation / crossover operators とlineage link生成を追加した | `TEST-candidate-mutation-preserves-lineage`; `TEST-candidate-crossover-combines-parent-mechanisms` |
| 2026-06-05 | Evolution | 候補仮説から制御されたablation follow-upを生成する `AblationGenerator` を追加した | `TEST-ablation-generator-creates-controlled-followups` |
| 2026-06-05 | Scheduling | exploit / novelty / cheap probeを混ぜて次実験候補を選ぶ `PortfolioScheduler` を追加した | `TEST-portfolio-scheduler-balances-exploit-explore-cost`; `TEST-evolution-agent-schedules-balanced-portfolio` |
| 2026-06-05 | Meta Evaluation | 選択ポリシーを実測上位候補と比較し、hit rate / regret / costを測る `PolicyEvaluator` を追加した | `TEST-policy-evaluator-measures-selection-regret` |
| 2026-06-05 | SOTA Challenge | JSON challenge定義を読み込み、baseline/compute/禁止shortcutを検証する `SotaChallengeDefinition` を追加した | `TEST-sota-challenge-loads-definition-file` |
| 2026-06-05 | SOTA Governance | evidence summaryからclaim readiness levelを段階判定し、未解決reviewer objectionで外部SOTA主張を止めるgateを追加した | `TEST-claim-readiness-promotes-with-evidence` |
| 2026-06-05 | SOTA Reporting | challenge report、再現手順、limitations、manifestを束ねる `SotaReportBundleGenerator` を追加した | `TEST-sota-report-bundle-writes-reproduction-files` |
| 2026-06-05 | Reproducibility | challenge baseline、candidate command、run artifact、反復run、禁止shortcut、compute budgetを確認する再現チェックリストを追加した | `TEST-reproduction-checklist-detects-missing-evidence` |
| 2026-06-05 | Evaluation | 共有benchmark/metricで新候補が既存accepted候補より悪化した場合に検出する `RegressionDetector` を追加した | `TEST-regression-detector-flags-shared-benchmark-drop` |
| 2026-06-05 | Scheduling | prerequisite、replication、ablation、benchmark unlockの依存関係を管理し、ready planを返す `ExperimentDependencyGraph` を追加した | `TEST-dependency-graph-unlocks-ready-plans` |
| 2026-06-05 | Review | claimに対するreviewer objectionをevidence参照つきで記録し、解決状態を追跡する `ReviewLedger` を追加した | `TEST-review-ledger-tracks-open-objections` |
| 2026-06-05 | Metrics | metric方向とbaseline scaleをそろえ、confidence込みの比較スコアを返す `MetricNormalizer` を追加した | `TEST-metric-normalizer-aligns-direction-and-scale` |
| 2026-06-05 | SOTA Governance | 成功・失敗runを含むcompute消費、残予算、予算超過を集計する `ComputeAccountant` を追加した | `TEST-compute-accountant-counts-failed-attempts` |
| 2026-06-05 | SOTA Governance | dataset split overlapとmetric misuseをblocking findingとして検出する `LeakageChecker` を追加した | `TEST-leakage-checker-blocks-split-overlap` |
| 2026-06-05 | SOTA Governance | 同一leaderboard/hidden targetへの過剰probeとvalidation control欠落を検出する `AntiOverfittingMonitor` を追加した | `TEST-anti-overfitting-monitor-limits-target-probes` |
| 2026-06-05 | SOTA Governance | 候補仮説と既知algorithm familyのkeyword overlapを検出し、novelty reviewを促す `NoveltyReviewer` を追加した | `TEST-novelty-reviewer-flags-known-family-overlap` |
| 2026-06-05 | SOTA Governance | 改善が複数benchmark/domainへ転移しているかを評価する `ExternalValidityEvaluator` を追加した | `TEST-external-validity-requires-transfer-across-domains` |
| 2026-06-05 | Planning | 広い研究目標をbenchmark/metric/target/budget/riskつきsubgoalへ分解する `ObjectiveDecomposer` を追加した | `TEST-objective-decomposer-creates-measurable-subgoals` |
| 2026-06-05 | Governance | 人間によるpriority override、freeze、reject、manual evidenceをaudit trailとして保持する `HumanOverrideLedger` を追加した | `TEST-human-override-ledger-preserves-audit-trail` |
| 2026-06-05 | Memory | 仮説、dataset/metric、failure mode、decision rationaleをtag/kind/textで検索する `ResearchMemoryIndex` を追加した | `TEST-research-memory-index-queries-by-tag-kind-and-text` |
| 2026-06-05 | Memory | 失敗・棄却・低信頼結果をavoid条件つきの再利用可能な知識として残す `NegativeResultArchive` を追加した | `TEST-negative-result-archive-retains-failed-paths` |
| 2026-06-05 | Scheduling | 残予算内で情報量/コスト比が高い候補を選択する `BudgetAwarePlanner` を追加した | `TEST-budget-aware-planner-prefers-cheap-informative-candidates` |
| 2026-06-05 | Claims | internal score、benchmark improvement、leaderboard improvement、external SOTA claimを分離する `ClaimDisciplineClassifier` を追加した | `TEST-claim-discipline-distinguishes-sota-from-internal-score` |
| 2026-06-05 | Safety | 実験コマンド、推定コスト、artifact contractを実行前に検査する `SafeAutomationPolicy` を追加した | `TEST-safe-automation-policy-blocks-disallowed-command` |
| 2026-06-05 | SOTA Governance | imported baselineとreproduced baselineを分け、claim前のbaseline再現状況を確認する `BaselineReproductionLedger` を追加した | `TEST-baseline-reproduction-ledger-requires-reproduced-baseline` |
| 2026-06-05 | Statistics | 反復runの平均と95% confidence intervalを計算する `ConfidenceIntervalEstimator` を追加した | `TEST-confidence-interval-estimator-summarizes-repeated-runs` |
| 2026-06-05 | SOTA Reporting | 失敗run、benchmark gap、reviewer objectionからhonest limitationを生成する `LimitationSectionGenerator` を追加した | `TEST-limitation-section-includes-failures-gaps-and-objections` |
| 2026-06-05 | Benchmark | tabular、sequence、optimizerなど複数domainのbenchmarkを中立的に登録する `BenchmarkDomainCatalog` を追加した | `TEST-benchmark-domain-catalog-supports-multiple-domains` |
| 2026-06-05 | Architecture | planner/runner/analyst/reviewerなどのcomponent境界とcapabilityを登録する `RoleRegistry` を追加した | `TEST-role-registry-registers-extensible-components` |
| 2026-06-05 | Analysis | failure classifierを追加し、実装エラー、budget不足、benchmark mismatch、instability、invalid hypothesis、inconclusive resultを分類できるようにした | `TEST-failure-classifier-categorizes-run-outcomes` |
| 2026-06-05 | Scheduling | priority、情報利得、推定cost、依存制約を保持してready experimentを返す `ExperimentQueue` を追加した | `TEST-experiment-queue-orders-ready-experiments` |
| 2026-06-05 | Runner | timeout、最大cost、metric plateau、invalid output、safety violationで継続停止を判定する `StopCriteriaEvaluator` を追加した | `TEST-stop-criteria-evaluator-blocks-run-continuation` |
| 2026-06-05 | Traceability | 要求からCODE/TEST Atomへ到達できるかを監査する `CodeFireTraceAudit` を追加した | `TEST-codefire-trace-audit-connects-requirements-code-tests` |
| 2026-06-05 | Governance | `DecisionRecord` にrejected alternativesとfollow-upを追加し、意思決定の必須項目を検査するvalidatorを追加した | `TEST-research-program-records-runs-and-evidence` |
| 2026-06-05 | Reproducibility | hyperparameters、seed、dataset version、benchmark split、code reference、environmentをfingerprint化する `ConfigurationCapture` を追加した | `TEST-configuration-capture-records-reproducible-settings` |
| 2026-06-05 | Run Ledger | run IDからconfig、logs、artifacts、evidence、reportを辿る `RunIdentityLedger` を追加した | `TEST-run-identity-ledger-links-run-logs-artifacts-evidence` |
| 2026-06-05 | Product Metrics | reproducible run、改善結果、証跡、意思決定、uncertainty reductionから研究進捗を測る `ResearchProgressMeter` を追加した | `TEST-research-progress-meter-scores-sustained-progress` |
| 2026-06-05 | SOTA Framing | 証跡が不足するSOTA志向を外部claimではなくhypothesisとして扱う `SotaPursuitFramer` を追加した | `TEST-sota-pursuit-framer-keeps-weak-evidence-as-hypothesis` |
| 2026-06-05 | Product Surface | experiment automation firstを保つため、work itemの優先度とplan欠落を検査する `ExperimentAutomationSurfaceGuard` を追加した | `TEST-experiment-automation-surface-guard-prioritizes-automation` |
| 2026-06-05 | Experiment Plan | 実験計画のschema必須要素を検査する `ExperimentPlanContractBuilder` を追加した | `TEST-experiment-plan-contract-validates-required-schema` |
| 2026-06-05 | Artifacts | declared artifact、stdout/stderr、environment metadata、missing required artifactを評価する `ArtifactCollectionContract` を追加した | `TEST-artifact-collection-contract-detects-missing-required-artifacts` |
| 2026-06-05 | Campaign | runs、evidence、resultsからcampaign milestoneの完了、blocked、next milestoneを判定する `ExperimentCampaignTracker` を追加した | `TEST-experiment-campaign-tracker-identifies-next-milestone` |
| 2026-06-05 | Scheduling | ready experimentsをcost、batch size、benchmark diversity制約内でbatch化する `BudgetedExperimentBatchPlanner` を追加した | `TEST-budgeted-experiment-batch-planner-respects-cost-and-diversity` |
| 2026-06-05 | Analysis | failure classificationからretry、mutate、archive、human reviewへ分岐する `RetryEscalationPlanner` を追加した | `TEST-retry-escalation-planner-routes-failures-to-next-actions` |
| 2026-06-05 | Evidence | benchmark/dataset/protocol version変更時に古い証跡を検出する `EvidenceFreshnessAudit` を追加した | `TEST-evidence-freshness-audit-flags-stale-versioned-evidence` |
| 2026-06-05 | Analysis | repeated attemptsのmetric trajectoryからplateau、regression、unstableを診断する `LearningCurveAnalyzer` を追加した | `TEST-learning-curve-analyzer-detects-plateau-and-instability` |
| 2026-06-05 | Promotion | improvement、confidence、replication、blocker、noveltyから候補のpromote/replicate/mutate/reject/holdを決める `CandidatePromotionPolicy` を追加した | `TEST-candidate-promotion-policy-gates-promotion` |
| 2026-06-05 | SOTA Governance | hidden leaderboard submissionの過剰probe、cooldown違反、local evidence不足を検出する `LeaderboardSubmissionLedger` を追加した | `TEST-leaderboard-submission-ledger-flags-excessive-probing` |
| 2026-06-05 | SOTA Governance | contamination sourceとdataset/artifact overlapを検出する `ContaminationSourceRegistry` を追加した | `TEST-contamination-source-registry-flags-dataset-and-artifact-overlap` |
| 2026-06-07 | CodeFire | v0.8途中版をinstallし、`algorithm-evolution-agent-lab` をCodeFire管理へ戻して初回importを封印した | `CF-COMMIT-176865705a2e2a5f63e88d2f`; 114 tests pass |
| 2026-06-07 | Planning | queue/promotion/learning curve/budgetを統合する `ExperimentTriageBoard` を追加した | `REQ-AUTO-027`; `DES-AUTO-009`; `TEST-experiment-triage-board-ranks-actions` |
