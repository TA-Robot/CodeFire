# RVC-013: Resolution freshnessをdependency-aware invalidationへ精密化する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | domain / correctness / evolution / UX |
| Review kind | redesign |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 1 |
| Primary scope | `ResolutionBasis / stale_resolutions / extractor-policy-trace evolution` |
| Promotion target | future CFR / ADR |

## Executive Judgment

Resolutionへsource/target content、Trace Link、policy basisを保存し、変化時にstale blockerへする設計はfalse completionを防ぐ重要な強みである。一方policy全体hashと現在のAtom contentだけでは、無関係変更による過剰invalidatonとextractor semantic driftの過少/過剰判定を同時に抱える。実際に依存したtyped inputsを列挙するbasisへ進化させるべきである。

## Scope and Review Question

- In scope: Atom/Trace/policy/evidence/extractor dependency、stale reason、refresh、legacy basis
- Out of scope: staleを自動passにすること、rationaleの意味的正しさをAI判定すること
- Question: Resolutionの判断前提が変わった場合だけ確実にinvalidになり、変わっていない場合は不要な再作業を生まないか

## Evidence

- `crates/codefire-core/src/lib.rs` — `ResolutionBasis`: source/target Atom hash、direct Trace Link ID/hash列、policy hashを保持する。
- `crates/codefire-core/src/lib.rs` — `stale_resolutions`: source、target、policy全体、trace linksを順序付きに比較し、最初の不一致理由を返す。
- `crates/codefire-core/src/lib.rs` — `build_resolution`: Fire endpoints間のdirect linksだけをbasisへ収集し、current policy全体hashを保存する。
- `crates/codefire-cli/src/main.rs` — `run_extinguish`: `--refresh`で新basisを作り、旧active Resolutionをsupersededへする。
- Test/fixture evidence: CFB-001 refreshとCFB-092 tool migration testsはあるが、rule-level policy、ontology/extractor version、evidence subjectを含む依存行列はない。

## Strengths to Preserve

- stale Resolutionをcommit blockerとし、古い判断を黙って再利用しないこと。
- refresh時に旧判断を削除せず、superseded historyとして残すこと。

## Gap / Risk

### Observed gap

policyの無関係field変更でも全resolutionがstaleになる一方、extractor implementationが同じhashを偶然生成した場合やontology semantics変更はbasisに現れない。複数原因があっても一つ目だけ返すため修復計画も不完全になり得る。

### Failure or stagnation mode

大量の不要refreshがalert fatigueを生むか、意味前提が変わったresolutionをfreshとしてsealする。どちらもevidence-backed resolutionへの信頼を損なう。

### Affected invariants

- stale resolution always blocks
- deterministic evidence basis
- extractor/policy/ontology evolution safety

## Improvement Direction

Resolution basisをversioned dependency setへし、Atom content+extractor descriptor、使用rule digest、Trace Link+ontology descriptor、evidence subject/trust profileをtyped entriesとしてcanonicalizeする。freshness evaluatorは全reason codeを返し、refresh planが変更dependencyを説明する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: policy全体hashを維持 | 保守的で単純 | 大量false stale | reject |
| B: dependency-aware basis | 精度と説明性 | schema/migration・依存収集が複雑 | adopt |

## Recommended Sequence

1. **Characterize**: source/target/link/rule/extractor/evidence各変更のfresh/stale期待matrixを作る。
2. **Separate**: dependency capture、canonical digest、freshness evaluation、refresh UXを分離する。
3. **Implement**: stable reason enumとrule/extractor dependencyから段階導入する。
4. **Prove**: mutation testingで各dependency変更が該当resolutionだけをinvalidateすることを確認する。
5. **Promote**: CFB-001後継root issue、data model、policy/Atom ADRへ昇格する。

## Acceptance Signals

- 各basis dependencyの単独mutationで該当stale reasonが決定的に出る。
- 無関係policy/link/Atom変更でresolutionがfreshを維持する。
- unknown legacy/critical dependencyはfreshへdowngradeされない。

## Verification Design

- Focused tests: dependency Cartesian matrix、複数stale reasons、canonical ordering、legacy basis。
- Cross-boundary tests: extinguish → policy/Atom/link/evidence change → verify → refresh → commit。
- Scale or fault tests: 10万resolutionのindexed invalidation、missing dependency object、tool version rollback。
- Observability: changed dependency IDs、reason codes、affected/omitted resolution countをbounded表示する。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-001はrefresh workflow、CFB-092はextractor migration。本候補は両者を包含するfreshness dependency modelとして既存issueへの追加discovery候補である。
- Required predecessor: RVC-006、RVC-007、RVC-008のversion/rule identities。
- Can combine with: RVC-004、RVC-012。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- rationale本文の意味が依然妥当かを自動推論しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
