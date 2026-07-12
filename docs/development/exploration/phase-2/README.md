# Phase 2: AI Development Standard Exploration

> **非正本 / exploration only**
>
> このdirectoryは、CodeFireをAI-native developmentの整合性・証跡基盤へ発展させるための検討材料である。未検証の仮説、競合する案、将来棄却する案を含む。現在状態、製品仕様、model availability、実行手順、採用済みarchitectureの正本として参照してはならない。

## 目的

このphaseでは、CodeFireをagent runtimeやprompt marketplaceへ変えるのではなく、複数のagent、human、IDE、CI、remote serviceが共有できるconsistency and evidence protocolとして成立させる道筋を検討する。

1. AI-native SDLCを、曖昧な会話ではなくtyped work contractと検証可能な状態遷移として表す。
2. context、task、review、handoff、evidence、certificateをdigestで結び、staleな判断を再利用しない。
3. GPT-5.6 SolのMedium、High、Ultraをtask riskと検証可能性でroutingする。
4. MCP、A2A、LSP、SARIF、CDEvents、OpenTelemetry、in-toto、SLSA、JSON Schemaの責務を混同せず接続する。
5. Git中心の既存teamが全面移行せず価値を試せるadoption pathを設計する。
6. feature数やpatch量ではなく、trusted integration throughputとescaped defectで成果を測る。

## Evidence Snapshot

- 調査日: 2026-07-11
- 対象: 現worktreeのCodeFire product docs、Rust CLI surface、development management、AI acceleration docs、project/global Codex config、Codex CLI 0.144.1 bundled model catalog
- 主要なlocal根拠:
  - README.md
  - docs/00_philosophy.md
  - docs/01_product_definition.md
  - docs/02_core_concepts.md
  - docs/05_consistency_model.md
  - docs/automation_interface.md
  - docs/11_internal_architecture.md
  - docs/development/ai/
  - .codex/
- 主要なexternal一次資料:
  - OpenAI GPT-5.6 Sol model page
  - OpenAI Codex configuration、Subagents、MCP、App Server、GitHub Action、non-interactive mode
  - OpenAI Agents SDK orchestration、tracing、agent evals
  - MCP、A2A、LSP、SARIF、CDEvents、OpenTelemetry、in-toto、SLSA、JSON Schema、SPDX各公式仕様
- 制約: 現worktreeはdirtyで、AI acceleration文書とproject Codex configは未追跡である。このsnapshotはrelease済みbaselineやprovider availabilityの保証ではない。

## Evidence Labels

親directoryの規則に従う。

| Label | このphaseでの扱い |
|---|---|
| observed | local source、runtime、config、commandで直接確認した事実 |
| documented | CodeFireまたは外部標準の公式文書に明記された主張 |
| inferred | 複数の観察から導いたが未実験の結論 |
| hypothesis | fixture、反証条件、metricを必要とする仮説 |
| decision-candidate | strategy、ADR、schema、AI protocolへの昇格候補 |
| rejected | 棄却理由と再検討条件を残した案 |

## 文書一覧

| 文書 | 主題 | 主なpromotion先 |
|---|---|---|
| 01-ai-native-sdlc-control-plane.md | AI-native SDLC、状態遷移、control plane | AI operating protocol、product strategy、automation interface |
| 02-category-positioning-and-product-boundary.md | category、Git bridge、競争優位、非目標 | strategy、product definition、ADR |
| 03-agent-tool-protocol-stack.md | MCP/A2A/LSP/SARIF/CDEvents/OTel/attestation | protocol specs、schema、adapter backlog |
| 04-context-packet-and-cache-engineering.md | typed context/task/review packet、cache、freshness | AI protocol、JSON Schema、packet generator |
| 05-multi-agent-routing-and-concurrency.md | Medium/High/Ultra、lane、ownership、fallback | model routing policy、Codex config、orchestrator wrapper |
| 06-evals-provenance-and-observability.md | eval、trace、evidence trust、provenance | eval suite、evidence spec、telemetry policy |
| 07-integration-adoption-and-standardization.md | IDE/CI/MCP、migration、governance、conformance | roadmap、ADR、release/distribution docs |
| experiment-catalog-and-kpi-scorecard.md | 反証実験、North Star、guardrail | benchmark/eval suite、management KPI |

## 共通System Model

このphaseでは、CodeFireを次の4層で考える。

~~~text
Agent and human execution layer
  Codex / other agents / humans / reviewers / CI runners

Interoperability adapter layer
  MCP / A2A / LSP / SARIF / CDEvents / OpenTelemetry / Git bridge

CodeFire consistency kernel
  Atoms / Trace Graph / Fires / Resolutions / Verification / Evidence

Trust and distribution layer
  sealed commits / certificates / in-toto / SLSA / signatures / remote policy
~~~

agent編成、prompt routing、marketplace、scheduleはexecution layerの責務である。CodeFire product coreは、外部実行者へstable contract、read-only context、dry-run plan、diagnostics、evidence、sealを提供する。

## 直接確認した重要観察

1. documented: CodeFireはAI agent orchestration、marketplace、prompt managementをproduct non-goalとしている。
2. documented: external AI、CI、human wrapperはJSON envelope、dry-run、next_actions、context、review-pack、evidenceを通じてCodeFireを利用する想定である。
3. observed: current context packはrequired_filesとlikely_filesが中心で、base revision、digest、freshness、token estimate、selection reasonを持たない。
4. observed: current task packetはMarkdown contractで、packet schema、packet digest、base digest、write-set conflict checkを持たない。
5. observed: current command contractはcodefire.command_result.v1を持つが、schemas/に対応する正式JSON Schemaがない。
6. observed: local Codex 0.144.1 bundled catalogはGPT-5.6 Solにmedium、high、xhigh、max、ultraを列挙し、ultraをmaximum reasoning with automatic task delegationと説明する。
7. documented: public GPT-5.6 Sol API model surfaceはreasoning effortとしてnone、low、medium、high、xhigh、maxを列挙する。
8. documented: public Codex config referenceはmodel_reasoning_effortをxhighまで列挙する一方、Codex Subagents文書はUltraをmaximum reasoningとproactive delegationを組み合わせたintelligence levelとして説明する。
9. inferred: Responses APIでUltraをmaxへ文字列変換するだけではdelegation semanticsが失われる。
10. documented: CodeFireはGit互換でもGit上位layerでもない。この境界は高保証設計を単純にするが、既存teamのadoption funnelを狭める。
11. inferred: de facto化の主要なmoatはmodel固有promptではなく、cross-artifact consistency graph、evidence chain、conformance suite、adapter ecosystemである。

## Identifier Rule

| Prefix | Meaning |
|---|---|
| Q-AI-* | AI-native SDLCまたはcontrol planeの設計問い |
| Q-CAT-* | category、positioning、product boundaryの問い |
| Q-INT-* | protocol、integration、interoperabilityの問い |
| Q-CTX-* | context、packet、cacheの問い |
| Q-ORCH-* | multi-agent、routing、concurrencyの問い |
| Q-EVAL-* | eval、provenance、observabilityの問い |
| Q-ADOPT-* | adoption、migration、standardizationの問い |
| HYP-AI-* | phase 2横断の反証可能な仮説 |
| MET-AI-* | KPIまたは測定定義 |
| EXP-AI-* | experiment catalogの実験 |
| DC-AI-* | promotion候補のdecision |

実験の正本は、このexploration範囲ではexperiment-catalog-and-kpi-scorecard.mdとする。product current-stateの正本ではない。

## Phase 2 Decision Candidates

- DC-AI-001: CodeFireの対外categoryをAI change consistency and evidence protocolとして定義する。
- DC-AI-002: agent orchestrationをproduct外へ維持し、MCP、CLI JSON、schema、capability discoveryだけをcore contractにする。
- DC-AI-003: Context Pack、Task Packet、Review Packetをversioned typed contractにし、Markdownをgenerated viewへ寄せる。
- DC-AI-004: task、context、patch、validation、review、certificateをdigest chainで結ぶ。
- DC-AI-005: Ultraをreasoning valueではなくreasoning plus delegation policyとして定義する。
- DC-AI-006: Git bridgeをcore Git compatibilityではなく、外部adoption adapterとして実験する。
- DC-AI-007: deterministic gateをmodel graderより上位に置くeval pyramidをrelease policyにする。
- DC-AI-008: SARIF、in-toto/SLSA、OpenTelemetry、CDEventsへのexportを独自形式追加より優先する。
- DC-AI-009: public schema、conformance vectors、capability negotiation、extension registryを標準化の最小核にする。

いずれもこの時点では未採用である。

## Promotion Rule

decision candidateを昇格するときは、親directoryの規則に加え次を満たす。

1. 対応するEXP-AIを同じbase revisionとfixtureで再現する。
2. provider固有挙動とprovider-neutral contractを分離する。
3. security、privacy、user consent、secret redactionを評価する。
4. backward compatibility、migration、schema negotiationを定義する。
5. owner、cost、failure budget、見直し日を定める。
6. promotion先のschema、test、runbook、ADRを同じchange packageで更新する。

| Finding | Promotion先 |
|---|---|
| category / product boundary | docs/development/strategy/、docs/01_product_definition.md、ADR |
| AI execution rule | docs/development/ai/ |
| machine contract | schemas/、docs/automation_interface.md |
| product command or object | product docs、ADR、Rust tests |
| adapter | roadmap、独立crate/package、integration docs |
| eval / KPI | eval suite、benchmark、project management |
| security / privacy | security ADR、runbook、managed policy |
| rejected hypothesis | phase 2最終synthesisへ理由だけを残す |

## Exit Criteria

- AI-native SDLCの各transitionにinput、output、owner、evidence、failure stateがある。
- MCP/A2A/LSP/SARIF/CDEvents/OTel/in-toto/SLSA/JSON Schemaの責務が重複なく説明される。
- typed packet contractとdigest bindingのcandidateがある。
- Medium/High/Ultraのrouting、fallback、runtime adapter、反証実験がある。
- Git bridgeを含む段階的adoption pathと中止条件がある。
- eval corpus、North Star、quality/security guardrailが定義される。
- P0 experimentを実行できるfixture、command、metricがある。
- 採用結果だけが正本へpromotionされ、残りのscratchをarchiveまたは削除できる。

## Promotion Status

- Status: not promoted
- Promotion candidates: strategy、docs/development/ai/、schemas/、ADR、product docs、test/eval、integration roadmap
- Required next step: P0 experimentsのfixtureとbaselineを作り、decision candidateごとにsupported、falsified、inconclusiveを記録する
