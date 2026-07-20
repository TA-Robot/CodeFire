# RVC-058: Correctness-aware performance observability

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | performance / correctness / UX |
| Review kind | refine |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `command metrics and invariant work counters` |
| Promotion target | product spec / strategy experiment |

## Executive Judgment

metricsがautomation envelope内に入り、measured/unmeasuredを区別する点は維持すべきである。現状はtotalとdomain result countが中心で、object validation、filesystem reads、graph visits、cache validity等を示さないため、速さだけでなく「必要な正しさの仕事を何件実行したか」を観測する契約へ進めるべきである。

## Scope and Review Question

- In scope: scan/status/verify/branch/storage/view/remoteのphase outcome、work counter、validation/cache evidence
- Out of scope: external telemetry backend、利用者追跡、wall-clock SLA、AI運用metrics
- Question: performance改善がvalidation省略や結果弱化ではないことをmetricsから検証できるか

## Evidence

- `crates/codefire-cli/src/metrics.rs` — `scan_metrics` / `verification_metrics`: total pipelineをmeasuredとし、atom extraction等のsubphaseはunmeasuredである。
- `crates/codefire-cli/src/metrics.rs` — `cache_unimplemented`: cache情報は実装なしとしてoutputから省略/ unavailableになり、validity inputやhit/missを持たない。
- `crates/codefire-cli/src/metrics.rs` — `attach_metrics`: metricsを`data.metrics`へ置き、domain envelopeから分離する正しいboundaryがある。
- `crates/codefire-store/src/lib.rs` — `read_object_record` / `validate_sealed_commit`: correctness上重要なhash/root/reference検証を行うがwork counterへ露出しない。
- Test/fixture evidence: metrics shape/count testsはあるが、performance fix前後でdiagnostic/resultとvalidation countが同値というtestはない。

## Strengths to Preserve

- unmeasured phaseを0msと偽らずnull/`measured:false`で表す。
- metricsをdomain resultやtop-level envelope truthから分離する。

## Gap / Risk

### Observed gap

経過時間とresult collection sizeは分かるが、file/object/bytes/graph/validation workが分からない。cacheを導入した場合のkey validityとcorruption fallbackも観測契約がない。

### Failure or stagnation mode

速くなった理由がmemoizationかvalidation skipか判断できず、wall time noiseで回帰を見逃す。metricsを増やすほど機微path/IDやhigh-cardinality dataを漏らす危険もある。

### Affected invariants

- validation completeness
- diagnostic equivalence under optimization
- bounded, non-sensitive automation metrics

## Improvement Direction

`WorkMetrics`をphase outcomeとstable countersへ分け、`objects_read/validated`、`bytes_hashed`、`files_walked`、`graph_nodes/edges`、`cache_hit/miss/invalidated`、`omitted_items`を持たせる。counterはprivacy-safe aggregateに限定し、各optimization testがresult+diagnostic+work deltaを同時assertする。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: invariant-aware work counters | machine非依存で正しさと性能を結べる | instrumentation propagationが必要 | adopt |
| B: phase wall timeだけ詳細化 | UIは分かりやすい | noiseとvalidation skipを識別できない | reject as primary |

## Recommended Sequence

1. **Characterize**: RVC-047 scale scenariosで必要counterとprivacy分類を決める。
2. **Separate**: domain resultから独立したmetrics sink/contextを定義する。
3. **Implement**: branch list、scan、sealed validation、ancestryからinstrumentする。
4. **Prove**: optimization前後のresult/diagnostic同値とcounter curveを固定する。
5. **Promote**: metrics schema/capabilityとperformance gateへ反映する。

## Acceptance Signals

- 主要commandで支配的I/O/graph/validation workをaggregate counterとして説明できる。
- performance changeは同一domain result/diagnosticと期待counter改善を同時に示す。
- metrics disabled時のsemantic resultが同一で、secret/path/high-cardinality IDを出さない。

## Verification Design

- Focused tests: counter increment、phase outcome、cache invalidation、metrics disabled
- Cross-boundary tests: core/store/CLI countersと`data.metrics` schema
- Scale or fault tests: 3点scale、corrupt cache/object、partial traversal、budget rejection
- Observability: stable aggregate counter、schema version、measured/estimated/unavailable区分

## Dependencies and Duplication Check

- Existing CFB/CFR: fixed CFB-007/030/031、CFR-061/062/150はmetrics個別shapeを改善済み。本候補はcorrectness workとの結合であり再openではない。
- Required predecessor: RVC-047 budget、RVC-048 graph work model
- Can combine with: RVC-042、RVC-047、RVC-048、RVC-060

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 外部telemetry収集や個人/組織利用追跡を導入しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
