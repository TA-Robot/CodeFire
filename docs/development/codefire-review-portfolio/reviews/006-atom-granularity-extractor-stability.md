# RVC-006: Atom granularityとextractor semanticsをversioned contractにする

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | domain / correctness / evolution / UX |
| Review kind | redesign |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 1 |
| Primary scope | `codefire-core::AtomIndex / extractors / hash schema` |
| Promotion target | ADR |

## Executive Judgment

ファイルではなく意味単位を追うAtomはCodeFireの差別化そのものだが、意味単位の境界はextractor実装に依存する。`hash_schema_version`でmass fireを回避した進展は維持しつつ、granularity、selector semantics、extractor identity、移行互換性を一つのversioned contractへ引き上げるべきである。

## Scope and Review Question

- In scope: Atom境界、ID、selector、content hash、language extractor、schema migration、granularity診断
- Out of scope: 完全なcompiler frontend、全言語のsemantic parser、AIによるAtom推定
- Question: tool upgradeや記法差で、利用者が意図しないAtom identity/change responsibilityが発生しないか

## Evidence

- `docs/01_product_definition.md` — `Artifact and Atom`: Atomを意味的な最小管理単位とし、extractor誤検出diagnosticとsemantic changeを反映するhashを要求する。
- `crates/codefire-core/src/lib.rs` — `build_atom_index`: path patternからformat別extractorを選び、全Atomをsortしてduplicate IDsを構築する。
- `crates/codefire-core/src/lib.rs` — `extract_markdown_atoms` / `extract_explicit_cf_atoms` / language extractor群: heading scopeやmarker/symbol spanという異なる境界規則でcontent hashを作る。
- `crates/codefire-core/src/lib.rs` — `AtomIndex.hash_schema_version` / `build_scan_result_with_options`: source不変のhash schema driftを`tool_migration`へ分離する。
- Test/fixture evidence: `explicit_cf_atom_body_changes_update_content_hash_*`、`python_explicit_cf_atom_hash_is_stable_when_later_marker_is_added`、CFB-092 fixtureが局所安定性とmigrationを固定するが、全extractorのcross-version corpusはない。

## Strengths to Preserve

- stable Atom IDとcontent hashを分離し、duplicateをblockerとして明示すること。
- extractor-only driftをsource changeと区別し、ordinary Fire stormを抑えるmigration model。

## Gap / Risk

### Observed gap

単一のhash schema numberだけでは、言語、selector/span algorithm、normalization、configごとのsemantic versionを説明できない。Atomが粗すぎる/細かすぎることもdiagnosticではなく利用者のFire量として初めて現れる。

### Failure or stagnation mode

upgradeで大量stale resolution、silent missed change、または過剰Fireが起きる。adapter/pluginを増やすほど同じAtomIndex versionでもproducer semanticsが異なり、sealed history比較が曖昧になる。

### Affected invariants

- deterministic Atom identity/hash
- stale-resolution correctness
- Fire actionability / format compatibility

## Improvement Direction

AtomIndexへextractor manifest（format、implementation ID、semantic version、normalization、config digest）をsealし、selector kindごとのgolden corpusとcompatibility ruleを定義する。granularity lintはread-only recommendationとし、identity変更はexplicit migration planでのみ行う。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: global hash schemaだけを継続 | 単純 | producer差と局所migrationを表せない | reject |
| B: extractor manifest + corpus | drift原因と互換性を特定可能 | object/schemaサイズと移行設計 | adopt |

## Recommended Sequence

1. **Characterize**: 全formatについてboundary edit、rename、formatter、unrelated edit、version drift corpusを作る。
2. **Separate**: filesystem discovery、extractor selection、pure extraction、normalization、index assemblyを分離する。
3. **Implement**: extractor manifestとmigration classificationをAtomIndexへ追加するprototypeを作る。
4. **Prove**: cross-version golden vectorsとactual-source-vs-tool-drift decision matrixを通す。
5. **Promote**: Atom/hash schema ADR、data-model catalog、migration specへ昇格する。

## Acceptance Signals

- sealed AtomIndexから各Atomのproducer semanticsを一意に説明できる。
- unrelated edit/formatter/tool upgradeでのidentity・hash期待値が全format corpusで固定される。
- unknown extractor semantic versionをordinary clean/passedとして扱わない。

## Verification Design

- Focused tests: format別span/normalization/duplicate/rename/version matrix。
- Cross-boundary tests: extractor → scan Fire → stale resolution → commit object → legacy reader。
- Scale or fault tests: 巨大file、大量marker、malformed/binary/mixed encoding、extractor upgrade。
- Observability: extractor ID/version、affected Atom count、migration reason、granularity warningをbounded表示する。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-091/092/112/115、CFR-019/021は個別extractor defectとmigration修正。本候補は全extractorのversion contractを統合するため重複しない。
- Required predecessor: 現行hash schema migrationの互換fixture再監査。
- Can combine with: RVC-004、RVC-013、RVC-020。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 初期段階で全言語のAST完全性を保証しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
