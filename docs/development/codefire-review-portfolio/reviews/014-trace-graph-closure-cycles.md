# RVC-014: Trace Graphのclosure・cycle・propagation semanticsを定義する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | domain / correctness / performance |
| Review kind | experiment |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `TraceGraph / scan Fire propagation / context traversal` |
| Promotion target | ADR |

## Executive Judgment

現在のscanがdirect neighborだけへFireを出す単純さは予測可能で、context traversalもvisited setとdepth/limitで安全にboundedされている。この強みを保ちながら、productが期待する「変更影響」がdirect edgeだけなのか、typed transitive closureを含むのか、cycleをどう扱うのかを明文化すべきである。暗黙に探索深度を増やすのではなく、fixtureとactionability実験で決める。

## Scope and Review Question

- In scope: direct/transitive impact、link-type direction、cycle/self-loop、path identity、dedup、depth/budget
- Out of scope: arbitrary graph inference、全到達Atomへの無条件Fire生成
- Question: graph上のどこまでを確認責任として伝播し、複数path/cycleを一つの意味あるFireへ畳むか

## Evidence

- `crates/codefire-core/src/lib.rs` — `adjacent_atoms`: Trace Linksを双方向に1回走査し、direct endpointと2-node trace pathだけを返す。
- `crates/codefire-core/src/lib.rs` — `build_scan_result_with_options`: direct adjacencyごとにFireを作り、self targetをskipし、path hashをsemantic keyへ含める。
- `crates/codefire-cli/src/context.rs` — `atom_neighborhood` / `trace_adjacency`: visited set、depth上限、limit、omitted neighbor countを持つbounded BFSを提供する。
- `docs/01_product_definition.md` — `G-002 Traceable change impact` / `Trace Link`: Atom変更から関連Atomへの確認責任を提示するが「関連」のclosure semanticsは固定しない。
- Test/fixture evidence: context depth traversal testはあるが、diamond、cycle、複数typed pathでのFire propagation decision corpusはない。

## Strengths to Preserve

- direct-only生成がFire explosionを抑え、同一入力で決定的であること。
- read-only context traversalがcycleでloopせず、深度・件数・省略を説明すること。

## Gap / Risk

### Observed gap

scanはlink direction/typeに関係なく双方向direct impactとみなし、specはtransitive impactの有無を述べない。複数pathはpath hashが違えば別Fireになり得る一方、cycleのdomain validityも未定義である。

### Failure or stagnation mode

遠隔requirement/testへの本質的影響を見逃すか、closure追加でFire stormを起こす。repositoryごとに期待が異なるまま「CodeFireがimpactを追う」というclaimだけが強くなる。

### Affected invariants

- Fire completeness/actionability
- deterministic graph traversal
- bounded complexity / no false completion

## Improvement Direction

link typeごとにpropagation mode（none/direct/declared bounded transitive）、direction、max depth/path budgetをpolicy/ontologyで明示する実験を行う。cycleはvalid graph featureとしてSCC単位のdiagnostic/groupingを検討し、path explosionはcanonical minimal witnessとomitted pathsで抑える。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 全linkのunbounded closure | impact見逃しを減らす | exponential paths/alert fatigue | reject |
| B: declared typed bounded propagation | profile別に検証可能 | policyと説明が複雑 | experiment |

## Recommended Sequence

1. **Characterize**: chain、diamond、cycle、self-loop、mixed directionのexpected Fire corpusを作る。
2. **Separate**: graph validity、impact reachability、witness path selection、Fire groupingを分離する。
3. **Implement**: read-only impact previewでdirect vs candidate policyを比較する。
4. **Prove**: work counters、equivalence、boundedness、actionability auditで採否を決める。
5. **Promote**: 採用profileだけTrace/Fire specとontology ADRへ昇格する。

## Acceptance Signals

- 各link type/profileについて伝播方向・深度・budgetがmachine-readableである。
- cycle/diamondでも同じsemantic responsibilityを無制限に複製しない。
- budget超過時はUNKNOWN/omittedを示し、completeと表示しない。

## Verification Design

- Focused tests: chain/diamond/SCC/self-loop/multi-edge、canonical witness、dedup。
- Cross-boundary tests: graph parse → impact preview → scan Fire → context/explain → verification。
- Scale or fault tests: high-degree cyclic graph、path explosion、depth/budget boundary、malformed endpoint。
- Observability: visited nodes/edges、paths considered/omitted、SCC count、propagation profileを出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-123はendpoint integrity、CFR-024はrequired-link count性能。本候補はimpact closure semanticsで重複しない。
- Required predecessor: RVC-004 actionability thresholdとRVC-007 ontology direction。
- Can combine with: RVC-015。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- graph reachabilityをsemantic correctnessの証明とみなさない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
