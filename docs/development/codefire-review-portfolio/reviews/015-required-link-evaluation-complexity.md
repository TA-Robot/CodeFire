# RVC-015: required-link評価のcomplexity contractをsemantic evolutionまで拡張する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | performance / correctness / evolution |
| Review kind | preserve |
| Priority | P2 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `required_link_missing / TracePolicy / verification metrics` |
| Promotion target | future CFR |

## Executive Judgment

CFR-024で全link再走査をcount indexへ置き換え、概ね`O(A + L + A*R)`へ改善した現行実装は維持すべきである。今後ontology constraint、namespace、filtered profileを追加すると再びhidden traversalが入り得るため、performance fixを「実装詳細」ではなくdecision-equivalent complexity contractとして固定する候補である。

## Scope and Review Question

- In scope: required link count index、Atom/rule/link cardinality、memory、diagnostic order、incremental/profile evaluation
- Out of scope: benchmark値だけの最適化、semantic ruleの削減
- Question: repository規模とpolicy表現力が増えても、同じ欠落判定とdiagnosticを予測可能なwork budget内で返せるか

## Evidence

- `crates/codefire-core/src/lib.rs` — `required_link_missing`: Atom kind mapと`(from,type,target_kind)` count mapをlink一巡で構築し、Atomごとのrulesをlookupする。
- `crates/codefire-core/src/lib.rs` — `RequiredLinkRule` / `TracePolicy`: 現行ruleはsource kind別のtype、target kind、minimumに限定される。
- `crates/codefire-core/src/lib.rs` — `required_link_missing_counts_large_fixture_with_link_index`: 256組fixtureでindex経路の結果を固定する。
- `crates/codefire-cli/src/metrics.rs` — `verification_metrics`: missing countは出すが、visited links/rules/lookupsやmemory budgetは出さない。
- Test/fixture evidence: CFR-024はfixedで計算量を改善済みだが、work counter assertion、adversarial cardinality、semantic extensionのequivalence corpusはない。

## Strengths to Preserve

- deterministic BTreeMap indexとsorted Atom indexにより結果順序が安定すること。
- missing diagnosticが`min`と`found`を保持し、性能改善後も意味を失わないこと。

## Gap / Risk

### Observed gap

complexity expectationはissue記述にあり、public/source contractやcounterにない。新しいrule predicateを安易にper-Atom graph scanで実装しても回帰testが結果しか見なければ通る。

### Failure or stagnation mode

大規模repositoryでverifyが突然二次・三次時間になり、timeout回避のためtrace completenessを無効化する誘因が生まれる。optimizationがdiagnostic order/腐敗検出を変える危険もある。

### Affected invariants

- required trace completeness decision
- complexity is a contract
- optimized/unoptimized diagnostic equivalence

## Improvement Direction

reference evaluatorとindexed evaluatorのequivalence corpus、stable work counters（atoms/rules/links/index entries/lookups）、accepted asymptotic/memory budgetを定義する。rule language拡張はcompile-to-index可能性とbudget impactを設計審査する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: wall-clock benchmarkのみ | 実体感がある | environment依存でsemantic driftを検出しにくい | reject |
| B: reference equivalence + work counters | 決定と計算量を同時に固定 | instrumentation維持 | adopt |

## Recommended Sequence

1. **Characterize**: A/R/L、high cardinality、missing endpoint、duplicate edgeのfixture familyを作る。
2. **Separate**: rule compilation/index build/evaluation/diagnostic renderingを分離する。
3. **Implement**: test-visible work countersとslow reference evaluatorを追加する。
4. **Prove**: property/equivalence testsとasymptotic counter assertionsを行う。
5. **Promote**: source architecture complexity sectionと必要時のperformance CFRへ昇格する。

## Acceptance Signals

- counterが`links_scanned=L`相当を超えず、rule lookupが宣言budget内にある。
- reference/indexed evaluatorが結果・順序・corruption diagnosticで一致する。
- ontology/rule extensionがbudgetを超える場合、explicit unsupported/plan diagnosticになる。

## Verification Design

- Focused tests: zero/duplicate/missing links、min>1、many kinds/rules、deterministic order。
- Cross-boundary tests: parser → compiled policy → evaluator → verify JSON/text/certificate count。
- Scale or fault tests: 10万Atoms/Links、high-degree node、adversarial strings、memory cap。
- Observability: atoms/links/rules/index entries/lookupsとtruncationなしのdecision countをmetricsへ出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-024は同じ旧問題を修正済み。本候補は再修正ではなく、fixを将来のrule evolutionに対する契約へ昇格する。
- Required predecessor: RVC-007 ontology rule shapeとRVC-014 propagation semanticsの決定。
- Can combine with: RVC-014、RVC-020。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- performanceのためmissing-link diagnosticやcorruption checkを省略しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
