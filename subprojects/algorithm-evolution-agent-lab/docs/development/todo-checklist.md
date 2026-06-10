# Development TODO Checklist

| ID | Area | Task | Status | Evidence |
|---|---|---|---|---|
| AE-001 | Agent | 最小 evolution loop を実装する | done | `test_evolution_agent_selects_best_candidate` |
| AE-002 | Scoring | multi-objective candidate scoring を実装する | done | `test_candidate_scoring_uses_quality_and_cost` |
| AE-003 | Experiments | benchmark runner interface を追加する | done | `CODE-LocalExperimentRunner` |
| AE-004 | Research | baseline registry を追加する | done | `CODE-BaselineRegistry`; `test_baseline_registry_selects_best_for_metric_direction` |
| AE-005 | Agent | LLM planner adapter を追加する | done | `CODE-CodexPlanner`; `test_codex_planner_invokes_codex_exec` |
| AE-006 | Evaluation | SOTA claim gate を実装する | done | `CODE-SotaClaimGate`; `test_sota_claim_requires_evidence` |
| AE-007 | Spec | 研究プログラム、実験自動化、SOTA governance の詳細仕様を作る | done | `docs/spec/*`, `docs/design/*` |
| AE-008 | CodeFire | 仕様拡張を CodeFire commit として管理する | done | `codefire commit` |
| AE-009 | Model | `ResearchProgram` aggregate を実装する | done | `CODE-ResearchProgram`; `test_research_program_records_runs_and_evidence` |
| AE-010 | Runner | local experiment runner を実装する | done | `test_local_runner_captures_artifacts` |
| AE-011 | Evidence | evidence ledger を実装する | done | `CODE-EvidenceLedger`; `test_evidence_ledger_links_claims_to_runs` |
| AE-012 | Reports | living research report generator を実装する | done | `CODE-LivingResearchReport`; `test_living_research_report_summarizes_program` |
| AE-013 | Agent | Codex planner をCLIから選べるようにする | done | `--planner codex` |
| AE-014 | Model | hypothesis / experiment plan のdurable IDとlineage edgeを追加する | done | `CODE-HypothesisLineage`; `test_durable_research_identities_and_lineage` |
| AE-015 | Ingestion | metrics artifact を `ExperimentResult` に変換するresult ingestorを追加する | done | `CODE-ResultIngestor`; `test_result_ingestor_parses_metrics_artifact` |
| AE-016 | Analysis | run/resultからstructured analysis noteを生成するanalystを追加する | done | `CODE-RunAnalyst`; `test_run_analyst_writes_structured_note` |
| AE-017 | Benchmark | versioned benchmark schemaとrefresh可能なregistryを追加する | done | `CODE-BenchmarkRegistry`; `test_benchmark_registry_refreshes_versioned_definitions` |
| AE-018 | Evaluation | repeated-run confidence summaryを追加する | done | `summarize_repeated_runs`; `test_repeated_run_summary_requires_signal_over_noise` |
| AE-019 | Benchmark | small local benchmark fixtureを追加する | done | `CODE-LocalBenchmarkFixtures`; `test_local_benchmark_fixture_produces_artifacts`; `test_local_benchmark_fixture_runs_through_runner_and_ingestor` |
| AE-020 | Evolution | candidate mutation / crossover operatorsを追加する | done | `CODE-CandidateEvolutionOperators`; `test_candidate_mutation_preserves_lineage`; `test_candidate_crossover_combines_parent_mechanisms` |
| AE-021 | Evolution | ablation follow-up generatorを追加する | done | `CODE-AblationGenerator`; `test_ablation_generator_creates_controlled_followups` |
| AE-022 | Scheduling | portfolio schedulerを追加する | done | `CODE-PortfolioScheduler`; `test_portfolio_scheduler_balances_exploit_explore_cost`; `test_evolution_agent_schedules_balanced_portfolio` |
| AE-023 | Meta Evaluation | agent policy evaluatorを追加する | done | `CODE-PolicyEvaluator`; `test_policy_evaluator_measures_selection_regret` |
| AE-024 | SOTA Challenge | challenge definition loaderを追加する | done | `CODE-SotaChallengeDefinition`; `test_sota_challenge_loads_definition_file` |
| AE-025 | SOTA Governance | claim readiness promotion gateを追加する | done | `CODE-ClaimReadinessGate`; `test_claim_readiness_promotes_with_evidence` |
| AE-026 | SOTA Reporting | report bundle generatorを追加する | done | `CODE-SotaReportBundle`; `test_sota_report_bundle_writes_reproduction_files` |
| AE-027 | Reproducibility | reproduction checklist builderを追加する | done | `CODE-ReproductionChecklist`; `test_reproduction_checklist_detects_missing_evidence` |
| AE-028 | Evaluation | regression detectorを追加する | done | `CODE-RegressionDetector`; `test_regression_detector_flags_shared_benchmark_drop` |
| AE-029 | Scheduling | experiment dependency graphを追加する | done | `CODE-ExperimentDependencyGraph`; `test_dependency_graph_unlocks_ready_plans` |
| AE-030 | Review | review ledgerを追加する | done | `CODE-ReviewLedger`; `test_review_ledger_tracks_open_objections` |
| AE-031 | Metrics | metric normalizerを追加する | done | `CODE-MetricNormalizer`; `test_metric_normalizer_aligns_direction_and_scale` |
| AE-032 | SOTA Governance | compute accountantを追加する | done | `CODE-ComputeAccountant`; `test_compute_accountant_counts_failed_attempts` |
| AE-033 | SOTA Governance | leakage checkerを追加する | done | `CODE-LeakageChecker`; `test_leakage_checker_blocks_split_overlap` |
| AE-034 | SOTA Governance | anti-overfitting monitorを追加する | done | `CODE-AntiOverfittingMonitor`; `test_anti_overfitting_monitor_limits_target_probes` |
| AE-035 | SOTA Governance | novelty reviewerを追加する | done | `CODE-NoveltyReviewer`; `test_novelty_reviewer_flags_known_family_overlap` |
| AE-036 | SOTA Governance | external validity evaluatorを追加する | done | `CODE-ExternalValidityEvaluator`; `test_external_validity_requires_transfer_across_domains` |
| AE-037 | Planning | objective decomposerを追加する | done | `CODE-ObjectiveDecomposer`; `test_objective_decomposer_creates_measurable_subgoals` |
| AE-038 | Governance | human override ledgerを追加する | done | `CODE-HumanOverrideLedger`; `test_human_override_ledger_preserves_audit_trail` |
| AE-039 | Memory | research memory indexを追加する | done | `CODE-ResearchMemoryIndex`; `test_research_memory_index_queries_by_tag_kind_and_text` |
| AE-040 | Memory | negative result archiveを追加する | done | `CODE-NegativeResultArchive`; `test_negative_result_archive_retains_failed_paths` |
| AE-041 | Scheduling | budget-aware plannerを追加する | done | `CODE-BudgetAwarePlanner`; `test_budget_aware_planner_prefers_cheap_informative_candidates` |
| AE-042 | Claims | claim discipline classifierを追加する | done | `CODE-ClaimDisciplineClassifier`; `test_claim_discipline_distinguishes_sota_from_internal_score` |
| AE-043 | Safety | safe automation policyを追加する | done | `CODE-SafeAutomationPolicy`; `test_safe_automation_policy_blocks_disallowed_command` |
| AE-044 | SOTA Governance | baseline reproduction ledgerを追加する | done | `CODE-BaselineReproductionLedger`; `test_baseline_reproduction_ledger_requires_reproduced_baseline` |
| AE-045 | Statistics | confidence interval estimatorを追加する | done | `CODE-ConfidenceIntervalEstimator`; `test_confidence_interval_estimator_summarizes_repeated_runs` |
| AE-046 | SOTA Reporting | limitation section generatorを追加する | done | `CODE-LimitationSectionGenerator`; `test_limitation_section_includes_failures_gaps_and_objections` |
| AE-047 | Benchmark | benchmark domain catalogを追加する | done | `CODE-BenchmarkDomainCatalog`; `test_benchmark_domain_catalog_supports_multiple_domains` |
| AE-048 | Architecture | role registryを追加する | done | `CODE-RoleRegistry`; `test_role_registry_registers_extensible_components` |
| AE-049 | Analysis | failure classifierを追加し、失敗run/低信頼結果を研究判断カテゴリへ分類する | done | `CODE-FailureClassifier`; `test_failure_classifier_categorizes_run_outcomes` |
| AE-050 | Scheduling | priority、情報利得、推定cost、依存制約を持つexperiment queueを追加する | done | `CODE-ExperimentQueue`; `test_experiment_queue_orders_ready_experiments` |
| AE-051 | Runner | timeout、最大cost、plateau、invalid output、safety violationのstop criteria evaluatorを追加する | done | `CODE-StopCriteriaEvaluator`; `test_stop_criteria_evaluator_blocks_run_continuation` |
| AE-052 | Traceability | CodeFire trace linkの要求-code-test接続を監査するtrace auditを追加する | done | `CODE-CodeFireTraceAudit`; `test_codefire_trace_audit_connects_requirements_code_tests` |
| AE-053 | Governance | decision recordに棄却案とfollow-upを追加し、validatorで必須項目を検査する | done | `CODE-DecisionRecordValidator`; `test_research_program_records_runs_and_evidence` |
| AE-054 | Reproducibility | hyperparameters、seed、dataset version、split、code referenceを持つconfiguration captureを追加する | done | `CODE-ConfigurationCapture`; `test_configuration_capture_records_reproducible_settings` |
| AE-055 | Run Ledger | run IDからconfig、logs、artifacts、evidence、reportを辿るrun identity ledgerを追加する | done | `CODE-RunIdentityLedger`; `test_run_identity_ledger_links_run_logs_artifacts_evidence` |
| AE-056 | Product Metrics | reproducible run、改善結果、証跡、意思決定、uncertainty reductionから研究進捗を測るprogress meterを追加する | done | `CODE-ResearchProgressMeter`; `test_research_progress_meter_scores_sustained_progress` |
| AE-057 | SOTA Framing | 弱い証跡のSOTA志向をclaimではなくhypothesisとして表現するframerを追加する | done | `CODE-SotaPursuitFramer`; `test_sota_pursuit_framer_keeps_weak_evidence_as_hypothesis` |
| AE-058 | Product Surface | experiment automation firstを守るwork item priority guardを追加する | done | `CODE-ExperimentAutomationSurfaceGuard`; `test_experiment_automation_surface_guard_prioritizes_automation` |
| AE-059 | Experiment Plan | hypothesis、benchmark、baseline、metric、expected outcome、budget、command、artifact、analysis criteriaをまとめるplan contractを追加する | done | `CODE-ExperimentPlanContract`; `test_experiment_plan_contract_validates_required_schema` |
| AE-060 | Artifacts | declared artifact、stdout/stderr、environment metadata、missing required artifactを評価するartifact collection contractを追加する | done | `CODE-ArtifactCollectionContract`; `test_artifact_collection_contract_detects_missing_required_artifacts` |
| AE-061 | Campaign | runs、evidence、resultsからcampaign milestoneの完了/blocked/nextを判定するtrackerを追加する | done | `CODE-ExperimentCampaignTracker`; `test_experiment_campaign_tracker_identifies_next_milestone` |
| AE-062 | Scheduling | ready experimentsをcost、batch size、benchmark diversity制約内でbatch化するplannerを追加する | done | `CODE-BudgetedExperimentBatchPlanner`; `test_budgeted_experiment_batch_planner_respects_cost_and_diversity` |
| AE-063 | Analysis | failure classificationからretry/mutate/archive/human reviewへ分岐するretry escalation plannerを追加する | done | `CODE-RetryEscalationPlanner`; `test_retry_escalation_planner_routes_failures_to_next_actions` |
| AE-064 | Evidence | benchmark/dataset/protocol version変更時に古い証跡を検出するfreshness auditを追加する | done | `CODE-EvidenceFreshnessAudit`; `test_evidence_freshness_audit_flags_stale_versioned_evidence` |
| AE-065 | Analysis | repeated attemptsのmetric trajectoryからplateau/regression/unstableを診断するlearning curve analyzerを追加する | done | `CODE-LearningCurveAnalyzer`; `test_learning_curve_analyzer_detects_plateau_and_instability` |
| AE-066 | Promotion | improvement、confidence、replication、blocker、noveltyから候補のpromote/replicate/mutate/reject/holdを決めるpolicyを追加する | done | `CODE-CandidatePromotionPolicy`; `test_candidate_promotion_policy_gates_promotion` |
| AE-067 | SOTA Governance | hidden leaderboard submissionの過剰probe、cooldown違反、local evidence不足を検出するledgerを追加する | done | `CODE-LeaderboardSubmissionLedger`; `test_leaderboard_submission_ledger_flags_excessive_probing` |
| AE-068 | SOTA Governance | contamination sourceとdataset/artifact overlapを検出するregistryを追加する | done | `CODE-ContaminationSourceRegistry`; `test_contamination_source_registry_flags_dataset_and_artifact_overlap` |
| AE-069 | Planning | queue/promotion/learning curve/budgetを統合して次研究actionを順位付けするtriage boardを追加する | done | `CODE-ExperimentTriageBoard`; `TEST-experiment-triage-board-ranks-actions` |
| AE-070 | Planning | triage結果を次イテレーションのactive/review/deferred計画へ変換するplannerを追加する | done | `CODE-ExperimentIterationPlanner`; `TEST-experiment-iteration-planner-builds-bounded-plan` |
| AE-071 | Governance | 実験campaignのrisk registerを追加し、open/blocking riskとmitigation checklistを優先度順に返す | done | `CODE-ExperimentRiskRegister`; `TEST-experiment-risk-register-prioritizes-open-risks` |
| AE-072 | Evidence | promotion/review向けに改善、信頼度、replication、freshness、risk、artifact gapを束ねるevidence pack builderを追加する | done | `CODE-ExperimentEvidencePackBuilder`; `TEST-experiment-evidence-pack-builder-blocks-incomplete-claims` |
| AE-073 | Review | evidence packとreviewer objectionからclaim review queueを生成し、block/revise/approveを優先度順に返す | done | `CODE-ClaimReviewQueue`; `TEST-claim-review-queue-prioritizes-blocked-claims` |
| AE-074 | Audit | evidence readiness、review decision、external claim eligibility、follow-upをclaim audit trailへ記録する | done | `CODE-ClaimAuditTrailBuilder`; `TEST-claim-audit-trail-records-review-decision` |
| AE-075 | Release | evidence readiness、review decision、audit status、release artifactsを統合してexternal claim release gateを判定する | done | `CODE-ClaimReleaseGate`; `TEST-claim-release-gate-allows-complete-claims` |
| AE-076 | Planning | baseline gap、negative results、novelty、risk、budget、challenge objectiveから次に攻めるresearch frontierを順位付けする | done | `CODE-ResearchFrontierMap`; `TEST-research-frontier-map-ranks-opportunities` |
| AE-077 | Planning | runnable research frontierをbounded experiment plan draftへ変換し、blocked/deferred frontierを実行対象から外す | done | `CODE-FrontierExperimentPlanner`; `TEST-frontier-experiment-planner-builds-runnable-plans` |
| AE-078 | Planning | 完了runのoutcomeをresearch frontier signalへ戻し、次iterationのfrontier rankingを保守的に更新する | done | `CODE-FrontierFeedbackIntegrator`; `TEST-frontier-feedback-integrator-updates-frontier-signals`; 139 tests pass |
| AE-079 | Planning | 長期campaignでfrontier順位・score・actionがどう変化したかをdrift reportとして要約する | done | `CODE-FrontierDriftReporter`; `TEST-frontier-drift-reporter-summarizes-priority-shifts`; 141 tests pass |
| AE-080 | Planning | 複数research cycleの結果から次cycleの探索方針調整recommendationを生成する | done | `CODE-ResearchCycleRetrospective`; `TEST-research-cycle-retrospective-recommends-policy-adjustments`; 143 tests pass |
| AE-081 | Reporting | research cycle retrospectiveを次cycle planning用Markdown summaryへ整形する | done | `CODE-RetrospectivePlanningSummary`; `TEST-retrospective-planning-summary-renders-markdown`; 144 tests pass |
| AE-082 | Planning | retrospective recommendationをmitigation/active/review/archive/deferred laneを持つ次cycle planへ変換する | done | `CODE-ResearchCyclePlanSynthesizer`; `TEST-research-cycle-plan-synthesizer-builds-next-cycle-lanes`; 145 tests pass |
| AE-083 | Reporting | synthesized research cycle planをlane別Markdownへ整形する | done | `CODE-ResearchCyclePlanMarkdown`; `TEST-research-cycle-plan-markdown-renders-lanes`; 146 tests pass |
| AE-084 | Planning QA | research cycle planを実行前にlintし、capacity/budget/item欠落をblocker/warningとして返す | done | `CODE-ResearchCyclePlanLint`; `TEST-research-cycle-plan-lint-flags-invalid-plan`; 148 tests pass |
| AE-085 | Reporting | research cycle plan lint reportをレビュー用Markdownへ整形する | done | `CODE-ResearchCyclePlanLintMarkdown`; `TEST-research-cycle-plan-lint-markdown-renders-findings`; 149 tests pass |
| AE-086 | Planning QA | retrospectiveからplan/lint/Markdown一式をplanning packetとして生成する | done | `CODE-ResearchCyclePlanningPacketBuilder`; `TEST-research-cycle-planning-packet-builds-plan-lint-and-markdown`; 150 tests pass |
| AE-087 | Reporting | planning packetを単一のcycle handoff Markdown文書へ整形する | done | `CODE-ResearchCyclePlanningPacketMarkdown`; `TEST-research-cycle-planning-packet-markdown-renders-review-document`; 151 tests pass |
| AE-088 | Reporting | planning packet handoff artifactのpath/hash/byte manifestを生成する | done | `CODE-ResearchCyclePlanningPacketManifestBuilder`; `TEST-research-cycle-planning-packet-manifest-records-artifact-hashes`; 152 tests pass |
| AE-089 | Reporting | planning packet manifestを監査用Markdown tableへ整形する | done | `CODE-ResearchCyclePlanningPacketManifestMarkdown`; `TEST-research-cycle-planning-packet-manifest-markdown-renders-audit-table`; 153 tests pass |
| AE-090 | Planning QA | planning packet manifestと保存済みartifact内容のdriftを検出する | done | `CODE-ResearchCyclePlanningPacketManifestVerifier`; `TEST-research-cycle-planning-packet-manifest-verifier-detects-drift`; 154 tests pass |
| AE-091 | Reporting | planning packet manifest verification結果を監査用Markdownへ整形する | done | `CODE-ResearchCyclePlanningPacketManifestVerificationMarkdown`; `TEST-research-cycle-planning-packet-manifest-verification-markdown-renders-findings`; 155 tests pass |
| AE-092 | Planning QA | planning packet、artifact、manifest、verification、監査Markdownをhandoff bundleへまとめる | done | `CODE-ResearchCyclePlanningHandoffBundleBuilder`; `TEST-research-cycle-planning-handoff-bundle-builds-artifacts-and-audits`; 156 tests pass |
