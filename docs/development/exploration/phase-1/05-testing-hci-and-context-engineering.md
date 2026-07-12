# Testing, HCI, And Context Engineering

> **非正本 / exploration only**
>
> この文書は「正しく作れたか」「人間とagentが正しく使えるか」「必要なcontextを十分かつ小さく渡せるか」を同じevaluation systemとして検討する。採用内容はtest strategy、automation interface、AI operating protocol、context packs、UX/product docsへpromotionする。

## 1. Evaluation Target

CodeFireの評価対象はbinaryのpass/failだけではない。

```text
semantic correctness
durability and recovery
security boundary
performance and boundedness
human comprehension and recovery
agent automation success
context retrieval quality
trusted integration throughput
```

「testが多い」「patchが速い」「agentが完了と言った」は、それぞれ単独では上記の証拠にならない。

## 2. Current Test Shape

2026-07-11 snapshot:

- `observed`: Rust sourceは約40,890行。
- `observed`: `crates/codefire-cli/src/tests.rs` は約8,457行、`main.rs` は約7,956行。
- `observed`: Rust workspace内の `#[test]` / `#[tokio::test]` annotationは約231件。
- `documented`: object identity、sealed validation、diff/merge、graph traversalはgolden/edge testsを重視する方針。
- `inferred`: test数より、不変条件coverage、failure oracle、fault injection、discoverabilityの管理が次の課題である。

この数値はcurrent-state正本ではなく、探索時点の構造理解だけに使う。

## 3. Test And Evidence Layers

| Layer | Purpose | Typical oracle |
|---|---|---|
| pure unit | parser、hash、policy、graph function | exact value/invariant |
| property/metamorphic | 広いinput空間 | relation/invariant |
| model-based | operation sequence/state transition | reference model |
| golden | schema、canonical bytes、JSON envelope | reviewed fixture |
| differential | Rust/Python、backend間compatibility | agreed subset |
| integration lifecycle | init→open→scan→verify→commit等 | persistent state + output |
| crash/fault injection | write ordering/recovery | old/new valid state |
| fuzz/security corpus | malformed/adversarial input | no escape/panic/hang |
| performance | scale/complexity regression | work counters + thresholds |
| dogfood/e2e | realistic task utility | workflow outcome/CFB |
| AI eval | implementation/review/context quality | hidden defect/task oracle |

### Q-TEST-001

feature/command coverageではなく、CodeFire固有の不変条件coverageをどう正本化するか。

### HYP-TEST-001

`hypothesis`: `INV-* -> owning module -> unit/property/integration/fault test -> evidence` matrixを作ると、test countやline coverageよりfalse confidenceを減らせる。

反証条件:

- invariantが抽象的すぎてtestとのmappingが主観的になる。
- matrix維持costが変更速度を大きく下げる。
- line/branch/mutation coverageと比べて追加defectを発見しない。

指標:

- P0 invariantのtest mapping率。
- mapped testが実際にinvariant違反でfailする率。
- escaped CFB/CFRのうち未mappingだった割合。
- matrix更新時間。

対応実験: `EXP-TEST-001`。

## 4. Property And Model-Based Testing

候補property:

- canonicalize/serialize/parse round-trip。
- same payload -> same full digest/object ID。
- different payload collisionをsilent reuseしない。
- dry-run前後でfilesystem snapshotが同じ。
- idempotency same request -> same result、different payload -> conflict。
- commit success -> head validates、active reset、no pending marker。
- any injected failure -> old/new valid stateまたはexplicit pending/corrupt diagnostic。
- merge plan deterministic。
- Trace propagation terminates and deduplicates。
- metrics on/offでsemantic output/stateが同じ。

### HYP-TEST-002

`hypothesis`: commandのrandom operation sequenceをpure state modelとfilesystem implementationの両方へ適用すると、単発testよりstate interaction bugを発見できる。

反証条件:

- fixture生成/縮小が遅すぎてfailureを理解できない。
- external command/filesystem nondeterminismによりflakyになる。

指標:

- unique minimized counterexample数。
- sequence lengthとshrink time。
- flaky reproduction率。
- found defect当たりmaintenance cost。

対応実験: `EXP-TEST-002`。

## 5. Historical Defect Corpus

### Q-EVAL-001

CFB/CFR historyを、将来のcode/model/reviewer/context変更を比較するstable evalへ変換できるか。

### HYP-EVAL-001

`hypothesis`: historical issueから「pre-fix fixture、expected detection、minimal patch scope、protected invariant」を抽出したhidden corpusを作れば、AI development accelerationを主観ではなく測定できる。

反証条件:

- issue docsに再現可能なpre-fix evidenceが不足する。
- current sourceへ答えが埋め込まれ、eval leakageが支配的になる。
- historical defectsがfuture task distributionを代表しない。

指標:

- reproducible historical issue率。
- implementer fix success/first-pass acceptance。
- reviewer seeded defect recall/precision。
- context retrieval oracle recall。
- eval leakage label coverage。

対応実験: `EXP-EVAL-001`。

## 6. Mutation And Differential Testing

### HYP-TEST-003

`hypothesis`: store/core/automationのcritical predicateへ限定したmutation testingは、巨大integration suiteが実際にinvariantを検出するかを測れる。

反証条件:

- runtimeがfull validation budgetを大幅に超える。
- equivalent mutantのtriage costが高すぎる。

指標:

- critical module mutation score。
- surviving mutantのinvariant category。
- runtime。

対応実験: `EXP-TEST-003`。

Rust/Python differentialは「全feature parity」ではなく、repository/object/remote formatの必要compatibility subsetだけを対象にする。意図的な差をfailure扱いしないmanifestが必要である。

## 7. Human-Computer Interaction

### Personas

- 初めてCodeFireを使うdeveloper
- repository maintainer
- CI operator
- external AI agent
- reviewer
- corruption/recoveryを扱うincident responder

### Core HCI questions

| ID | Question |
|---|---|
| `Q-HCI-001` | `open-clean/open-burning/open-consistent`から次行動を誤解なく選べるか |
| `Q-HCI-002` | Fireが責任を明確化するか、alert fatigueを生むか |
| `Q-HCI-003` | `no-change-required/invalid-fire`を安易に選ばずbasis/evidenceを理解できるか |
| `Q-HCI-004` | truncated outputで重要blockerを見落とさないか |
| `Q-HCI-005` | next_actionsは現在contextで安全に実行可能か |

### HYP-HCI-001

`hypothesis`: summary、blocker group、basis/explanation、detailsのprogressive disclosureは、flatな全件表示よりhuman/agentのtime-to-correct-actionを短縮する。

反証条件:

- extra navigation/commandsで単純taskの時間が増える。
- groupingがrare/high severity blockerを隠す。
- humanとagentで最適表示が大きく異なる。

指標:

- time-to-first-correct-action。
- task completion率。
- retry/誤command数。
- blocker見落とし率。
- output/token bytes。

対応実験: `EXP-HCI-001`。

### HYP-HCI-002

`hypothesis`: next actionにcommandだけでなくtarget path、precondition、mutation/read-only、expected resultを持たせるとautomation loopの安全性が上がる。

反証条件:

- payload肥大化が大きい。
- stale preconditionによりnext actionの失敗が増える。
- consumerがcommand stringしか使わない。

指標:

- suggested command成功率。
- wrong target path率。
- destructive action誤実行数。
- next action生成から実行までのstale率。

対応実験: `EXP-HCI-002`。

### HCI invariants

| ID | Invariant |
|---|---|
| `INV-HCI-001` | human textとJSONは同じsemantic blocker/resultを表す |
| `INV-HCI-002` | omitted details/countを明示する |
| `INV-HCI-003` | destructive operationをdefault next actionにしない |
| `INV-HCI-004` | diagnosticsは対象repo/path/branch contextを曖昧にしない |
| `INV-HCI-005` | blockerとwarningを視覚・machine schemaの両方で区別する |

## 8. Fire Fatigue And Trust Calibration

### HYP-HCI-003

`hypothesis`: generated Fireごとにcause、trace path、basis hash、why-blocking、resolution choicesを説明すると、blind extinguishを減らせる。

反証条件:

- explanationが長すぎて確認されない。
- trace dataの品質が低く、explanationがfalse confidenceを増やす。

指標:

- Fire amplification factor。
- `invalid-fire` / `no-change-required`率。
- rationale/evidence completeness。
- 後でstale/incorrectと判明したresolution率。

対応実験: `EXP-HCI-003`。

## 9. Context Engineering As Information Retrieval

現行context packはtask-specific reading listとして有用だが、qualityを測るoracleがない。

### Retrieval unit

```text
task
  -> command/domain
  -> source owners
  -> persistent state
  -> invariants/security boundaries
  -> focused tests
  -> current issue/evidence
```

### Q-CONTEXT-001

static pack、source map graph、lexical search、semantic retrieval、full repositoryのどれが最小tokenで必要contextを満たすか。

### HYP-CONTEXT-001

`hypothesis`: command/source/persistence/invariant/testのtyped dependency graphをstatic context packへ重ねると、現行manual packよりoracle file recall/tokenを改善できる。

反証条件:

- source mapのstalenessがretrieval benefitを相殺する。
- historical taskでstatic packと差がない。
- graph maintenance costがtaskごとのmanual searchより高い。

指標:

- oracle file recall@token budget。
- precision、context tokens、latency。
- included fileの実際のcitation/use率。
- context不足由来のreview defect率。
- stale edge/reference率。

対応実験: `EXP-CONTEXT-001`。

### HYP-CONTEXT-002

`hypothesis`: compressed summaryへsource revision、path、section、checksum、omitted scopeを付ければ、agentはsummaryを使いつつ必要時に原文へ戻れる。

反証条件:

- checksum/revision metadataがcontext budgetを浪費する。
- source更新頻度が高くsummaryの大半が即staleになる。
- summaryが重要なnegative constraintを落とす。

指標:

- stale summary detection率。
- 原文fallback率。
- negative constraint omission数。
- summary有無のtask成功差。

対応実験: `EXP-CONTEXT-002`。

### Context invariants

| ID | Invariant |
|---|---|
| `INV-CONTEXT-001` | required file/referenceが存在し、task revisionと対応する |
| `INV-CONTEXT-002` | repository safety/invariant docsをscoreだけで除外しない |
| `INV-CONTEXT-003` | CodeFire productとdogfood projectのboundaryを混ぜない |
| `INV-CONTEXT-004` | secret、irrelevant user data、他project contextを含めない |
| `INV-CONTEXT-005` | compressionで省略したscopeを明示する |

## 10. Context Benchmark Design

historical taskごとに次を用意する。

```text
task statement
repository revision before fix
oracle required files
useful but optional files
forbidden/irrelevant files
protected invariants
expected validation
known minimal fix scope
hidden review defects
```

比較lane:

1. full repository context
2. current static context pack
3. `rg`/source map assisted retrieval
4. typed dependency graph retrieval
5. compressed summary + on-demand source

重要なのはpatch passだけでなく、context cost、out-of-scope edits、invariant defect、reviewer発見率を測ることである。

## 11. AI Role Evaluation Boundary

concrete model/tier routingはPhase 2の対象である。Phase 1ではmodel名に依存しないtask/evidence contractだけを定義する。

測るrole:

- manager: task decompositionとwrite-set conflict
- architect: invariant/risk completeness
- implementer: correct bounded patch
- reviewer: seeded defect detection
- docs synchronizer: source/docs consistency
- validation runner: command/evidence fidelity

### HYP-EVAL-002

`hypothesis`: model/tier比較は「first patch速度」ではなく、trusted integrationまでのlead time、rework、escaped defect、context tokenで評価すべきである。

反証条件:

- trusted integrationの人手oracleが不安定で比較不能。
- task heterogeneityが大きくsample sizeを確保できない。

指標:

- issue/task packet→trusted integration時間。
- first-pass acceptance。
- rework/conflict率。
- escaped defect severity。
- token/cost per accepted change。

対応実験はPhase 2へpromotionする。

## 12. Risks

| ID | Risk | Impact |
|---|---|---|
| `RISK-TEST-001` | test count/line coverageを品質と誤認 | invariant false confidence |
| `RISK-TEST-002` | giant integration test concentration | discoverability、ownership、runtime悪化 |
| `RISK-EVAL-001` | historical answer leakage | AI/context evalの無効化 |
| `RISK-HCI-001` | Fire/diagnostic fatigue | blind extinguish、tool abandonment |
| `RISK-HCI-002` | truncated outputでcritical blocker omission | unsafe automation decision |
| `RISK-CONTEXT-001` | critical context omission | confident wrong patch |
| `RISK-CONTEXT-002` | context pack肥大化 | latency/cost、attention dilution |
| `RISK-CONTEXT-003` | stale source map/summary | wrong file/contract selection |

## 13. Decision Candidates And Promotion

| Candidate | Promotion先 | Gate |
|---|---|---|
| `DC-EVAL-001` invariant coverage matrix | test strategy、development gates | `EXP-TEST-001` |
| property/model sequence suite | core/CLI tests | `EXP-TEST-002` |
| historical defect corpus | AI eval/test fixtures | `EXP-EVAL-001` |
| progressive diagnostic disclosure | automation/CLI UX spec | `EXP-HCI-001` |
| structured next action context | automation schema | `EXP-HCI-002` |
| Fire explanation contract | Fire/explain product docs | `EXP-HCI-003` |
| `DC-CONTEXT-001` typed context dependency graph | AI context packs/tools | `EXP-CONTEXT-001` |
| provenance-bearing summaries | AI operating protocol | `EXP-CONTEXT-002` |

## 14. Promotion Status

- 現時点でpromotion済みの項目: なし。
- owner候補: test/eval、CLI automation/HCI、AI context tooling maintainer。
- trigger: context pack generator、role routing benchmark、diagnostic schema変更、tests.rs分割。
