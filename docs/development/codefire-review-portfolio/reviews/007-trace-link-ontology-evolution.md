# RVC-007: Trace Link ontologyの進化規則を明示する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | domain / evolution / architecture |
| Review kind | redesign |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 2 |
| Primary scope | `TraceLink / TraceGraph / RequiredLinkRule / links schema` |
| Promotion target | ADR |

## Executive Judgment

typed Trace Linksから表ではなくgraphを正本化する設計は維持すべきである。しかし`link_type`とAtom `kind`は現在string vocabularyで、方向、inverse、transitivity、deprecation、namespace、unknown typeの扱いがversioned ontologyとして定義されていない。外部consumerや長期履歴の前に、最小ontology kernelと安全な拡張規則が必要である。

## Scope and Review Question

- In scope: link type/kind vocabulary、方向性、namespace、version、deprecation、unknown extension、policy参照
- Out of scope: 汎用knowledge graph、semantic inference engine、業界ontologyの内蔵
- Question: link vocabularyが増減・改名されても、旧commitと現行policyの意味を保守的に解釈できるか

## Evidence

- `docs/02_core_concepts.md` — `Trace Link`: `refined_by`、`implemented_by`、`verified_by`を例示し、Trace Linkを一次情報とする。
- `crates/codefire-core/src/lib.rs` — `TraceLink` / `TraceGraph`: `from`、`to`、string `link_type`、derived ID/hashをversion 1 graphに格納する。
- `crates/codefire-core/src/lib.rs` — `parse_links` / `TraceLinkInput::into_trace_link`: bounded YAMLから任意文字列typeを読み、構造とIDを作るがontology registryは参照しない。
- `crates/codefire-core/src/lib.rs` — `RequiredLinkRule` / `required_link_missing`: policyがsource kind、link type、target kindの文字列一致でminimumを評価する。
- Test/fixture evidence: trace graph parse/ID/required-link testsとCFB-123 endpoint integrityはあるが、unknown/deprecated/renamed typeのcross-version fixtureはない。

## Strengths to Preserve

- link ID/hashがendpointsとtypeから決まり、resolution basisへbindされること。
- policyがrepository固有のrequired relationshipを宣言でき、core vocabularyへ過度に固定されないこと。

## Gap / Risk

### Observed gap

type/kindはopen stringなのに、拡張namespaceやcriticalityがなく、typo・新type・意味変更を同じ形で受理する。方向を逆に記述したlinkもsyntax上有効で、policyが見つけられないだけになり得る。

### Failure or stagnation mode

ontology更新でrequired linkが突然missingになり、旧resolutionが大量staleになる。独立adapterが同名typeを別の意味で出し、silent under-coverageまたはfalse Fireを生む。

### Affected invariants

- Trace Graph semantic identity
- required-link completeness
- resolution freshness / schema evolution

## Improvement Direction

最小ontology descriptorをsealし、type URI/namespace、version、source/target kind constraints、direction、optional inverse、critical extension、deprecation/replacementを宣言する。inferenceは既定で行わず、unknown critical typeはfail、noncritical typeはpreserve+unknown diagnosticとする。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 組込みenumへ完全固定 | typoを防ぐ | domain拡張性を失う | reject |
| B: versioned registry + namespaced extension | 安全な拡張と旧履歴解釈 | schema・migrationが増える | adopt |

## Recommended Sequence

1. **Characterize**: typo、reverse edge、rename、deprecated type、unknown extensionのdecision tableを作る。
2. **Separate**: graph instanceとontology descriptor、policy requirementsを別object/typeにする。
3. **Implement**: built-in minimal vocabularyとrepository extension manifestのread-only validatorを作る。
4. **Prove**: legacy graph、unknown critical/noncritical、migration mappingのgolden vectorsを通す。
5. **Promote**: Trace Link spec、data model、public schema、extension ADRへ昇格する。

## Acceptance Signals

- typo/方向違反がlocation付きdiagnosticになり、silent missing-linkへ化けない。
- old/new ontology間の意味変更がexplicit migrationまたはUNKNOWNとして扱われる。
- custom domain typeをcore code変更なしでnamespace付き追加できる。

## Verification Design

- Focused tests: type constraint、direction、namespace、deprecation、unknown criticality。
- Cross-boundary tests: links parse → policy evaluation → Fire/resolution freshness → sealed validation。
- Scale or fault tests: 大規模vocabulary、duplicate alias、cycle/invalid endpoint、malformed manifest。
- Observability: ontology ID/version、unknown/deprecated counts、migration mappingをbounded diagnosticに出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-123はdangling endpoint、CFR-023/048はIDとYAML mutationの個別問題。本候補はvocabulary semanticsの進化規則で重複しない。
- Required predecessor: RVC-006のAtom kind producer contractとStage 2 schema critical-extension rule。
- Can combine with: RVC-013、RVC-014、RVC-015。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 自動推論で明示的なTrace Link確認責任を置き換えない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
