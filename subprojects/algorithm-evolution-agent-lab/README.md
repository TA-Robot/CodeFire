# Algorithm Evolution Agent Lab

機械学習の新しいアルゴリズム候補を、仮説生成、実験計画、実行、評価、次候補選定のループで進化させる実験自動化AIエージェントの研究用リポジトリです。

主目的は「単発のモデル実装」ではなく、目的関数に対して研究行動を継続的に改善するエージェント基盤を作ることです。

## 初期スコープ

- 研究目標を `ExperimentGoal` として構造化する
- 仮説を `Hypothesis` として管理する
- 実験計画を `ExperimentPlan` として生成する
- 結果を `ExperimentResult` として記録する
- novelty / quality / confidence / cost を使って候補を順位付けする
- 次に試すべき候補を選ぶ最小の evolution loop を実装する
- `ResearchProgressMeter` でreproducible run、改善結果、証跡、意思決定、uncertainty reductionから研究進捗を測る
- `SotaPursuitFramer` で証跡が不足するSOTA志向を外部claimではなくhypothesisとして表現する
- `ExperimentAutomationSurfaceGuard` でexperiment automation firstのproduct priorityを保つ
- local command runnerで実験を実行し、stdout/stderr/artifactを構造化して捕捉する
- `ResearchProgram` と `EvidenceLedger` で仮説、候補、run、証跡、意思決定を集約する
- `Hypothesis` / `ExperimentPlan` は安定IDを持ち、`ResearchProgram` は仮説lineageを記録できる
- `ConfigurationCapture` でhyperparameters、seed、dataset version、split、code reference、environmentをfingerprint化する
- `RunIdentityLedger` でrun IDからconfig、logs、artifacts、evidence、reportを辿れるようにする
- `ExperimentPlanContractBuilder` で実験計画に必要なschema要素を検査する
- `ArtifactCollectionContract` でdeclared artifact、stdout/stderr、environment metadata、missing required artifactを評価する
- `ExperimentCampaignTracker` でcampaign milestoneの完了、blocked、next milestoneを判定する
- `BudgetedExperimentBatchPlanner` でready experimentsをcost、batch size、benchmark diversity制約内でbatch化する
- `RetryEscalationPlanner` で失敗分類からretry、mutate、archive、human reviewへ分岐する
- `EvidenceFreshnessAudit` でbenchmark/dataset/protocol version変更時に古い証跡を検出する
- `LearningCurveAnalyzer` でrepeated attemptsのplateau、regression、unstableを診断する
- `CandidatePromotionPolicy` で候補のpromote、replicate、mutate、reject、holdを判定する
- `LeaderboardSubmissionLedger` でhidden leaderboardの過剰probe、cooldown違反、local evidence不足を検出する
- `ContaminationSourceRegistry` でdataset/artifact overlapによるcontamination riskを検出する
- `CandidateEvolutionOperators` でmutation / crossover由来の仮説を作り、lineageを残す
- `AblationGenerator` で候補仮説から制御されたablation follow-upを生成する
- `PortfolioScheduler` で高スコア候補、探索候補、低コストprobeを混ぜて次実験を選ぶ
- `PolicyEvaluator` で選択ポリシーのhit rate、regret、実験コストを実測結果から評価する
- `BaselineRegistry` でbenchmark/metric別のbaseline値と証跡参照を管理する
- `BenchmarkRegistry` でversioned benchmark定義、metric方向、validation protocolを管理する
- `ToyTabularFixture` で標準ライブラリだけの小さな再実行可能benchmarkを提供する
- `ResultIngestor` で `metrics.json` artifact を構造化された `ExperimentResult` に変換する
- `RunAnalyst` でrun/resultから分析note、失敗分類、次アクション、claim readiness更新を生成する
- `FailureClassifier` で実装エラー、budget不足、benchmark mismatch、instability、invalid hypothesis、inconclusive resultを分類する
- `ExperimentQueue` でpriority、情報利得、推定cost、依存制約つきの実験候補を管理する
- `StopCriteriaEvaluator` でtimeout、最大cost、metric plateau、invalid output、safety violationによる停止判定を行う
- repeated-run summaryで反復実験の平均、標準誤差、信頼度、promotion可否を判定する
- `LivingResearchReport` で候補、benchmark status、証跡、リスク、次実験をMarkdownへまとめる
- `SotaChallengeDefinition` でSOTA challengeのtask/dataset/metric/baseline/compute/禁止shortcutをファイル管理する
- `SotaReportBundleGenerator` でchallenge report、再現手順、limitations、manifestをbundle出力する
- `ReproductionChecklistBuilder` でchallenge/candidate/run artifact/repeated runの再現性条件を確認する
- `claim_readiness` で evidence summary から claim readiness level を段階判定する
- `RegressionDetector` で共有benchmark/metric上の既存候補からの悪化を検出する
- `ExperimentDependencyGraph` で前提実験、replication、ablation、benchmark unlockの依存関係を管理する
- `ReviewLedger` でclaimに対するreviewer objectionと解決履歴を管理する
- `MetricNormalizer` でmetric方向とbaseline scaleをそろえた比較用スコアを生成する
- `ComputeAccountant` で成功・失敗runを含むcompute消費と予算超過を集計する
- `LeakageChecker` でdataset split overlapとmetric misuseをblocking findingとして検出する
- `AntiOverfittingMonitor` で同一leaderboard/hidden targetへの過剰probeとvalidation control欠落を検出する
- `NoveltyReviewer` で既知algorithm familyとのkeyword overlapを検出し、novelty reviewを促す
- `ExternalValidityEvaluator` で改善が複数benchmark/domainへ転移しているかを評価する
- `ObjectiveDecomposer` で広い研究目標をbenchmark/metric/target/budget/riskつきsubgoalへ分解する
- `HumanOverrideLedger` で人間によるpriority override、freeze、reject、manual evidenceをaudit trailとして保持する
- `ResearchMemoryIndex` で仮説、dataset/metric、failure mode、decision rationaleをtag/kind/textで検索する
- `DecisionRecordValidator` で意思決定にevidence、棄却案、follow-upが残っているかを確認する
- `NegativeResultArchive` で失敗・棄却・低信頼結果をavoid条件つきの再利用可能な知識として残す
- `BudgetAwarePlanner` で残予算内の情報量/コスト比が高い候補を選択する
- `ClaimDisciplineClassifier` でinternal score、benchmark improvement、leaderboard improvement、external SOTA claimを分離する
- `SafeAutomationPolicy` で実験コマンド、推定コスト、artifact contractを実行前に検査する
- `BaselineReproductionLedger` でimported baselineとreproduced baselineを分け、claim前のbaseline再現状況を確認する
- `ConfidenceIntervalEstimator` で反復runの平均と95% confidence intervalを計算する
- `LimitationSectionGenerator` で失敗run、benchmark gap、reviewer objectionからhonest limitationを生成する
- `BenchmarkDomainCatalog` でtabular、sequence、optimizerなど複数domainのbenchmarkを中立的に登録する
- `RoleRegistry` でplanner/runner/analyst/reviewerなどのcomponent境界とcapabilityを登録する
- `CodeFireTraceAudit` で要求、設計、CODE Atom、TEST Atomの接続状態を監査する

## 実行

```bash
PYTHONPATH=src python3 -m unittest discover -s tests -v
PYTHONPATH=src python3 -m evoagent.cli --goal "Improve sample efficiency on small tabular datasets"
PYTHONPATH=src python3 -m evoagent.cli --goal "Improve sample efficiency on small tabular datasets" --planner codex --ideas 3
PYTHONPATH=src python3 -m evoagent.fixture_runner --fixture toy-tabular --candidate linear-threshold --seed 1
```

SOTA challenge定義の例は `challenges/toy_tabular_sample_efficiency.json` にあります。

## CodeFire

このリポジトリは CodeFire で追跡しやすいように、要求、設計、コード、テストの Atom と Trace Link を含みます。

```bash
codefire.yaml
codefire.links.yaml
codefire.policy.yaml
```

実験の履歴は `docs/experiments/` に追記します。
