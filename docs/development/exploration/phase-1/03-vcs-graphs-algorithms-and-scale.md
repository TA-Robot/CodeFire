# VCS, Graphs, Algorithms, And Scale

> **非正本 / exploration only**
>
> この文書はCodeFireのbranch/commit/Atom/Trace/Fire/diff/mergeをgraphとalgorithmとして捉える探索ノートである。採用内容はproduct specs、`docs/09_index_trace_policy.md`、`docs/10_merge_remote_server.md`、algorithm implementation、benchmark suiteへpromotionする。

## 1. Graphs Inside CodeFire

CodeFireには少なくとも次の異なるgraphがある。混同しないことが重要である。

| Graph | Vertices | Edges | Main operations |
|---|---|---|---|
| commit history | sealed commits | parent references | ancestry、merge base、fast-forward、GC reachability |
| object graph | typed objects | root/object references | sealed validation、copy、doctor、GC |
| Trace Graph | Atoms | typed Trace Links | completeness、impact propagation、explain |
| Fire provenance | Fire/Atom/change events | cause/target/path | dedup、stale、resolution basis |
| file/Atom diff relation | before/after entities | match/rename/copy/change | semantic diff、review pack |
| development dependency graph | source/docs/tests/invariants | owns/depends/verifies | context pack、task routing |

`decision-candidate`: graphごとにidentity、cycle policy、traversal limit、deterministic order、complexityを個別に定義する。

## 2. Scale Variables

性能主張は次の変数で記述する。

```text
F = managed files
S = total managed source bytes or lines
A = Atoms
E = Trace Links
R = Fires + Resolutions
O = stored objects
C = reachable commits
B = branches
M = merge requests/reviews
P = changed paths in one operation
D = maximum graph depth
```

fixtureの「小・中・大」は曖昧なので使わず、少なくとも上記の値とcontent sharing率を記録する。

## 3. Commit DAG And Ancestry

### Q-VCS-001

commit graphのcycle、multiple merge base、criss-cross、missing parent、deep historyをどう定義するか。

### Candidate invariants

| ID | Invariant |
|---|---|
| `INV-VCS-001` | parent referenceは存在し、validated commit objectを指す |
| `INV-VCS-002` | traversalはvisited setを持ち、malformed inputでも有限終了する |
| `INV-VCS-003` | fast-forward判定はcurrent remote headがnew headのancestorである場合だけtrue |
| `INV-VCS-004` | merge planのbase/source/target identityを結果へ明示する |
| `INV-VCS-005` | same validated DAGから同じancestor/base orderが得られる |

### HYP-VCS-001

`hypothesis`: ancestry queryを一回のcommand scopeでmemoizeし、object IDごとのvalidation/read resultを共有すると、安全性を落とさずbranch list、remote list、multi-head doctorを大幅に短縮できる。

反証条件:

- validation resultがbranch-specific policy/contextに依存する。
- memoized errorが後続branchのより有用なdiagnosticを隠す。
- peak memory増加がlatency改善を上回る。

指標:

- object record read/parse/hash回数。
- `B`と`C`に対するruntime curve。
- peak RSS。
- corruption diagnostic completeness。

対応実験: `EXP-PERF-001`。

## 4. Observed Branch List Cost

`observed`: local/remote branch listはbranch recordを列挙し、各headについてsealed commit graph validationをfreshに開始する。複数branchが長いhistoryを共有する通常形では、同じcommit/root objectを繰り返し読む。

概算:

\[
T_{current} \approx \sum_{b=1}^{B} reachable(head_b)
\]

共有historyが大半ならworst practical caseは `O(B·C)` に近い。command-level memoを共有できればcandidate costは次になる。

\[
T_{candidate} \approx O(C_{union} + B)
\]

これはobserved timeoutに直接つながるP0 research targetである。ただし、修正前にobject reads/countを計測し、推測を確定する。

## 5. Atom Identity And Refactoring

### Q-VCS-002

Atom identityはrename、file move、symbol rename、split、merge、copyで何を同一とみなすか。

explicit Atom IDは安定しやすい一方、derived IDはextractor/path/owner/nameの規則に依存する。CodeFireがsemantic impactを主張するなら、単にtext diffを改善するだけでなくidentity policyを説明する必要がある。

### HYP-VCS-002

`hypothesis`: explicit identityを優先し、derived identityについてはconfidence付きmatchをdiff viewに使ってもsealed identityや自動resolutionには使わない方がfalse equivalenceを抑えられる。

反証条件:

- explicit IDの運用costが高く、実projectの大半でtraceが維持されない。
- conservative matchによりfire/conflict false positiveが許容不能になる。
- confidence表現がhuman/agentを誤誘導する。

指標:

- rename/move/split/copy match precision・recall。
- refactor当たりgenerated Fire数。
- explicit ID付与率。
- false identityがcommit/reviewへ到達した件数。

対応実験: `EXP-VCS-001`。

## 6. Diff Layers

diffを次の独立layerへ分ける。

```text
L1 byte/file manifest diff
L2 bounded text/binary diff and rename/copy candidates
L3 Atom add/remove/change/move candidates
L4 Trace Link/policy/verification impact
L5 Fire/resolution/review consequence
```

### Invariants

- higher semantic layerはlower file truthを上書きしない。
- outputをtruncateしてもomitted countとscopeを示す。
- same inputs/optionsからsame order/resultを返す。
- binary/huge fileでunbounded memoryを使わない。
- rename/copyはidentity factではなくcandidateとして区別する。

### HYP-VCS-003

`hypothesis`: layer別resultとconfidence/provenanceを保持すると、単一の「semantic diff」へ混合するよりreviewerが誤推論を発見しやすい。

反証条件:

- output complexityが増え、human/agentのtask timeが悪化する。
- confidence calibrationが不安定で契約として使えない。

指標:

- seeded diff defect detection率。
- review time。
- false semantic conflict率。
- layerごとのoutput/token bytes。

対応実験: `EXP-VCS-002`。

## 7. Merge Semantics

`documented`: mergeはfile-level 3-wayをtarget open directoryへ適用し、正式historyを作らずopen-burningへ移す。semantic conflictはcandidate報告に留める。

### Q-VCS-003

multiple merge baseがある場合、base選択またはvirtual baseをどう扱うか。

### Q-VCS-004

file merge成功だがAtom/Trace/policy上のconflictがある場合、どのFireをどのbasisで立てるか。

### HYP-VCS-004

`hypothesis`: automatic semantic mergeを避け、deterministic file plan、semantic candidate、required Fire、verification recoveryを分離する現方針は、AI agent利用時にもtrust calibration上有利である。

反証条件:

- dogfoodでfalse-positive Fireが支配的となり、ほぼ全mergeがmanual no-change resolutionになる。
- semantic auto-resolutionを限定領域で使った方が、false acceptを増やさず大幅にlead timeを減らせる。

指標:

- merge当たりfile conflict/semantic candidate/Fire数。
- semantic candidate precision。
- mergeからconsistent commitまでの時間。
- `invalid-fire` / `no-change-required`比率。

対応実験: `EXP-VCS-003`。

## 8. Trace Graph And Fire Propagation

`documented`: MVP propagationはchanged Atomへ直接接続したAtomにrequired Fireを立てる。

### Q-GRAPH-001

propagation direction、edge type、depth、dedup、cycle、severityをpolicyとしてどう表すか。

### HYP-GRAPH-001

`hypothesis`: propagationをtyped directed graph queryとして定義し、`(change kind, link type, direction, max depth, required/advisory)` をexplicit policyにすると、fire生成を説明・測定できる。

反証条件:

- real projectのlink qualityが低く、policy細分化がnoiseを増やす。
- change kind classificationが不安定でdeterministic contractを作れない。
- policy理解costがFire削減効果を上回る。

指標:

- fire amplification factor = generated Fires / changed Atoms。
- reachable relevant Atom recall。
- irrelevant Fire率。
- graph diameter/fan-outとlatency。
- duplicate Fire suppression数。

対応実験: `EXP-GRAPH-001`。

### Candidate invariants

| ID | Invariant |
|---|---|
| `INV-GRAPH-001` | propagationはvisited keyを持ち有限終了する |
| `INV-GRAPH-002` | 同一cause/target/policyからduplicate required Fireを作らない |
| `INV-GRAPH-003` | Fireは生成reasonとtrace pathを説明できる |
| `INV-GRAPH-004` | policy変更は関連resolutionをstaleにする |
| `INV-GRAPH-005` | missing/duplicate Atom IDを正常edgeとして黙認しない |

## 9. Type And Data-Structure Questions

`inferred`: graph algorithm以前にstringly-typed identityが誤接続riskになる。

候補newtype:

```text
CommitId
ObjectId { kind, short_digest, full_digest? }
AtomId
LinkId
BranchName
RepoRelativePath
ValidatedCommitGraph
ValidatedTraceGraph
```

### HYP-GRAPH-002

`hypothesis`: parsed graphとvalidated graphを型で分けると、validation前のpayloadをancestry/merge/remote writerへ渡す経路を減らせる。

反証条件:

- validationがcontext依存で単一の`Validated*`型にできない。
- cloning/lifetime/serialization costが過大。

指標:

- graph consumerのvalidation重複数。
- raw `String`/`Value` identity boundary数。
- invalid graphを受け取れるpublic function数。

対応実験: `EXP-GRAPH-002`。

## 10. Complexity Budget

| Operation | Desired order | Main memory | Known uncertainty |
|---|---|---|---|
| Atom extraction | `O(S)` | bounded current file + `O(A)` index | parserごとのrepeat scan |
| duplicate detection | `O(A)` expected or `O(A log A)` deterministic | `O(A)` | deterministic ordering cost |
| required links | `O(A + E)` | `O(A + E)` indexes | rule count factor |
| direct propagation | `O(A + E)` upper bound | visited/dedup `O(A+E)` | policy depth expansion |
| sealed validation one graph | `O(O_reachable)` | visited `O(O_reachable)` | object IO dominates |
| branch list shared graph | `O(O_union + B)` candidate | shared memo | current repeated traversal |
| common ancestor | `O(C + parent edges)` | visited/frontier | multiple base semantics |
| file hash/commit manifest | `O(S)` | streaming preferred | repeated reads |
| doctor/storage/GC | `O(O)` | streaming/indexed | explanation/root reasons |
| diff | input-dependent | bounded lines/chunks | pathological repeated sequences |

### HYP-PERF-001

`hypothesis`: performance gateはwall timeだけでなくobjects/files/bytes parsed、peak RSS、output bytesを記録すると、hardware noiseを越えてalgorithm regressionを検出できる。

反証条件:

- instrumentation overheadが通常commandへ無視できない影響を与える。
- internal work counterがimplementation detailへ強く結合し維持不能。

指標:

- scaleを10倍にしたときのwork counter倍率。
- p50/p95/p99 latency。
- peak RSS。
- output omission率。

対応実験: `EXP-PERF-002`。

## 11. Adversarial Fixtures

- one million small files対one huge file
- many branches sharing one deep ancestry
- wide merge DAG
- missing parent near ancestry tail
- repeated-line text causing bad diff anchors
- high fan-out Trace Graph
- cyclic Trace Graph input
- duplicate Atom IDs across files
- large number of resolved/obsolete Fires
- renamed file with stable explicit Atom IDs
- same symbol copied to many paths
- binary artifact and invalid UTF-8

## 12. Risks

| ID | Risk | Impact |
|---|---|---|
| `RISK-PERF-001` | branchごとのshared history再検証 | read-only command timeout |
| `RISK-VCS-001` | derived Atom identityのrename fragility | false delete/add、Fire増加 |
| `RISK-VCS-002` | multiple merge base未定義 | wrong merge plan |
| `RISK-GRAPH-001` | propagation explosion | alert fatigue、memory/time増大 |
| `RISK-GRAPH-002` | semantic candidateのfalse confidence | incorrect automated review |
| `RISK-PERF-002` | outputだけboundedで内部計算はunbounded | hidden resource exhaustion |

## 13. Decision Candidates And Promotion

| Candidate | Promotion先 | Gate |
|---|---|---|
| `DC-PERF-001` command-scoped validation memo | store API、branch/remote readers | `EXP-PERF-001` |
| layered diff contract | diff product spec、automation schema | `EXP-VCS-002` |
| conservative semantic merge policy | merge spec/ADR | `EXP-VCS-003` |
| typed propagation policy | policy schema、core design | `EXP-GRAPH-001` |
| validated graph types | core/store API ADR | `EXP-GRAPH-002` |
| work-counter performance gate | benchmark/metrics docs | `EXP-PERF-002` |

## 14. Promotion Status

- 現時点でpromotion済みの項目: なし。
- owner候補: core graph/extractor、store validation、view/merge、performance maintainer。
- trigger: branch-list fix、semantic diff変更、propagation policy拡張、large-repo target定義。
