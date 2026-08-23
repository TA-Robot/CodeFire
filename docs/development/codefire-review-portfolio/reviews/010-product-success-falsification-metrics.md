# RVC-010: product successを反証可能なmetricsで管理する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | product / testing / evolution |
| Review kind | experiment |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 5 |
| Primary scope | `product metrics / pilot protocol / local workflow counters` |
| Promotion target | strategy experiment |

## Executive Judgment

現在のstrategyはcost to trusted integration、claim comprehension、Fire actionabilityなど妥当な指標とstop ruleを持つ点が強い。しかしCLI metricsは処理時間・entity countであり、product successとの測定bridgeはない。機能追加より先に、privacy-preservingなpilot protocolと反証閾値を固定すべきである。

## Scope and Review Question

- In scope: first trusted signal/seal、Fire actionability、claim comprehension、setup/maintenance cost、retention、guardrail incidents
- Out of scope: covert telemetry、個人評価、vanity metrics、外部analytics基盤の実装
- Question: CodeFireが価値を生んだか、摩擦を増やしただけかを事前定義した証拠で判定できるか

## Evidence

- `docs/development/strategy/product-strategy.md` — `Metrics` / `Pivot and Stop Rules`: trusted integration cost、first signal/seal、Fire actionability、retentionとzero-harm guardrailsを定義する。
- `docs/01_product_definition.md` — `Success Metrics`: missed impact検出、automation、state consistency、evidence readability、health agreement、performance visibilityを挙げる。
- `crates/codefire-cli/src/metrics.rs` — `scan_metrics` / `verification_metrics`: elapsed timeとAtoms/links/fires/blockersを返すoperational metricsで、user outcomeは扱わない。
- `docs/development/strategy/decision-register.md` — `STR-X01..X04` / `STR-P06`: market/adoption仮説には外部evidenceが必要で、telemetry/privacy policyはowner未決定である。
- Test/fixture evidence: metrics JSON unit testsはshape/countを検証するが、pilot event definition、survey rubric、falsifier thresholdのreplayable fixtureはない。

## Strengths to Preserve

- guardrail targetをzeroとし、速度やadoptionのためにfalse completion/user-work lossを許容しないこと。
- wall-clockだけでなくwork counterを重視し、market仮説を仮説と明記すること。

## Gap / Risk

### Observed gap

戦略metricは定義されても、開始/終了event、denominator、失敗/abandon、手作業時間、sampling、privacy、thresholdが実験packetに固定されていない。内部dogfood件数を外部価値のproxyにしやすい。

### Failure or stagnation mode

issue数やFire数の増減を成功と誤認し、反証されないままsurfaceを増やす。逆に局所的な速度改善がclaim理解や作業コストを悪化させても検知できない。

### Affected invariants

- no false completion / zero user-work loss
- reversible adoption
- evidence-based roadmap ordering

## Improvement Direction

Stage 5 experiment packetを作り、event semantics、cohort、baseline、manual time log、survey rubric、guardrail incident、stop threshold、data retentionを事前登録する。CLIは必要最小のlocal exportだけを提供し、送信はexplicit opt-in/offline collectionとする。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: usage count/Fire countを成功指標にする | 収集容易 | outcomeと逆相関し得る | reject |
| B: pre-registered mixed-method pilot | 価値と摩擦を反証可能 | 実施コストとprivacy review | experiment |

## Recommended Sequence

1. **Characterize**: metric dictionary、baseline、threshold、abandon/incident定義をpilot前に凍結する。
2. **Separate**: operational performance、product outcome、safety guardrail、research notesを分ける。
3. **Implement**: local bounded session summary/exportとmanual rubricを最小prototype化する。
4. **Prove**: synthetic sessionでevent reconstruction、missing data、privacy redactionを検査する。
5. **Promote**: pilot結果でSTR-X01..X04をcontinue/narrow/stopへ更新する。

## Acceptance Signals

- pilot開始前に各hypothesisのpass/fail/stop thresholdとdenominatorが公開される。
- abandon、manual workaround、false positive、guardrail incidentが成功サンプルから消えない。
- source content、command output、個人識別情報を収集せず結果を再計算できる。

## Verification Design

- Focused tests: event semantics、session boundaries、missing/duplicate events、redaction、opt-in。
- Cross-boundary tests: CLI local summary → pilot worksheet → metric calculationのgolden case。
- Scale or fault tests: interrupted session、offline use、clock skew、partial export、zero-event cohort。
- Observability: operational countersとproduct study fieldsを別schema/labelで出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-007とCFR-061/062はcommand performance observability。本候補はproduct outcome/falsificationであり重複しない。
- Required predecessor: STR-P06 privacy/retention owner、RVC-001 claim metric、Stage 2 stable contract。
- Can combine with: RVC-002、RVC-003、RVC-004、RVC-009。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- opt-out困難なtelemetryや個人の生産性scoreを導入しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
