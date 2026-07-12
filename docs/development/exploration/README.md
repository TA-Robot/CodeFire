# CodeFire Exploration Workspace

このdirectoryは、CodeFireの開発方針を決定する前に、意図的に広く考えを書き出すための検討作業領域である。

探索文書は正本ではない。相互に矛盾する仮説、棄却予定の案、未検証の問いを含んでよい。採用した結論だけを `docs/development/strategy/`、`docs/development/ai/`、ADR、product docsへ蒸留する。

## 探索の目的

1. 既存実装と過去計画だけを前提にせず、CodeFireが解くべき問題を再定義する。
2. 「速くpatchを作る」ではなく「信頼できる変更を最短で統合する」開発システムを設計する。
3. AI-native developmentの一時的な製品名やUIに依存せず、長期に残る契約と測定方法を作る。
4. 探索、決定、実装、検証、履歴を別の情報層として管理する。

## Phase 1: Computer Science / Software Engineering

場所: `phase-1/`

対象:

- 計算モデル、型、状態機械、形式手法、不変条件
- object store、database、distributed systems、transaction、concurrency
- diff/merge、graph、index、complexity、memory、large repository
- security、trust boundary、supply chain、recovery
- testing、property-based testing、model checking、observability
- information retrieval、context compression、human factors

出口条件:

- 各観点から設計問い、仮説、反証条件、測定指標、実験案がある。
- CodeFire固有の不変条件と、まだ証明できていない主張が分離されている。
- strategyへ昇格させる候補がdecision candidateとして列挙されている。

## Phase 2: AI Development Standard

場所: `phase-2/`

対象:

- AI-native SDLC、agent/tool contract、context engineering
- multi-agent coordination、review、provenance、evidence、eval
- IDE/CLI/CI/MCP/remote interoperability
- adoption、migration、standardization、competitive differentiation
- GPT-5.6 SolとMedium/High/Ultra reasoning tierのrouting

出口条件:

- CodeFireが「AI agentそのもの」ではなく、複数agent/toolが共有できる整合性・証跡基盤になる道筋がある。
- model routingがtask risk、uncertainty、blast radius、verification costで決まる。
- benchmark、KPI、互換性contract、段階的adoption pathが定義されている。

## Phase 3: Other Perspectives

場所: `phase-3/`

対象:

- product strategy、OSS、license、community、governance
- operations、SRE、privacy、compliance、economics
- UX、education、documentation information architecture
- release、distribution、ecosystem、premortem

出口条件:

- 技術的に正しくても普及しない失敗を説明できる。
- owner、cost、policy、support、migrationを含む運用可能な戦略候補がある。
- 中止・縮小・方向転換の条件が明示されている。

## Evidence Labels

探索文書の重要な主張には、可能な限り次のlabelを付ける。

| Label | 意味 |
|---|---|
| `observed` | 現在のsource、test、runtime、dogfoodで直接確認した事実 |
| `documented` | 公式仕様やprojectの正本文書に根拠がある |
| `inferred` | 複数の事実から導いたが直接検証していない |
| `hypothesis` | 実験や反証が必要 |
| `decision-candidate` | strategy/ADRへ昇格を検討する選択肢 |
| `rejected` | 理由と再検討条件を残して棄却した案 |

## Promotion Rule

探索内容を正本へ移すときは、次を満たす。

1. 採用する主張を一文で表す。
2. 代替案と棄却理由を書く。
3. 守る不変条件とtrade-offを書く。
4. 検証方法、owner、見直し条件を書く。
5. 正本側から探索文書を根拠として参照し、探索文書側へ採用先を追記する。

探索文書を長期のcurrent-state sourceにしない。現在状態、実行手順、active backlogはそれぞれ管理ハブ、runbook、roadmapへ移す。
