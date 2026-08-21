# RVC-048: Graph and ancestry algorithms

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | performance / correctness / architecture |
| Review kind | redesign |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `commit DAG / object graph / signature history` |
| Promotion target | future CFR / ADR |

## Executive Judgment

visited setを用いるiterative object graph walkとdeterministic common-ancestor tie breakは維持すべきである。ただしancestry実装がmain/remote/signatureへ分散し、targeted ancestor queryまで全ancestor mapを作り、history signature検証は再帰・無制限であるため、検証を弱めない共通bounded graph engineへ寄せる価値が高い。

## Scope and Review Question

- In scope: common ancestor、is-ancestor、signed history、reachable object graphのalgorithmとcorruption semantics
- Out of scope: 新しいmerge意味論、分散graph DB、UI可視化
- Question: deep/wide/corrupt graphで決定的結果を保ちつつ、必要nodeだけをboundedに訪問できるか

## Evidence

- `crates/codefire-cli/src/main.rs` — `common_ancestor` / `ancestor_distances` / `nearest_common_ancestor_from_right`: 左全ancestor mapを作り、右BFSで距離和・generationを比較する。
- `crates/codefire-cli/src/remote.rs` — `is_ancestor_in_objects`: target一件の存在確認にも`ancestor_distances`全mapを構築する。
- `crates/codefire-cli/src/signatures.rs` — `verify_commit_policy_recursive`: `require_history`時に再帰で全parent historyを検証し、node/depth budgetを持たない。
- `crates/codefire-cli/src/remote.rs` — `copy_object_graph` / `collect_object_records`: iterative stackと`seen`でcycle/revisitを抑える良いprecedentがある。
- Test/fixture evidence: `common_ancestor_handles_deep_generation_graph`は80世代を確認するが、wide DAG、very-deep stack、missing/corrupt node、work counterは未固定。

## Strengths to Preserve

- common ancestorの選択順がdistance、最大片距離、generation、IDで決定的である。
- object graph walkがreferenceを辿り、visited setで重複copyを避ける。

## Gap / Risk

### Observed gap

graph reader、visit policy、termination、budget、diagnosticが用途ごとに重複し、generation metadataはtie-break以外のpruningに使われない。再帰pathは深いhistoryでstack riskを持つ。

### Failure or stagnation mode

長大/merge-heavy履歴でremote applyやsignature policyが全history I/Oを繰り返し、壊れたnodeの診断位置も経路でずれる。最適化を個別実装するとancestor意味論が分岐する。

### Affected invariants

- fast-forward and common-ancestor correctness
- sealed/signed history validation
- complete object graph transfer

## Improvement Direction

typed graph readerとiterative walkerを共有し、query別stop predicate、generation hint、node/depth/byte budget、visited/corruption reportingを持たせる。common ancestorの意味論はgolden vectorで先に固定し、cacheはobjects root+head+validation generationをkeyにする。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 共通iterative graph engine | 意味・counter・boundsを共有できる | abstraction設計を誤ると複雑化 | experiment then adopt |
| B: 各関数を局所最適化 | 小差分 | corruption/ordering semanticsが分岐する | reject as strategy |

## Recommended Sequence

1. **Characterize**: linear、diamond、wide merge、missing、cycle-like corruptionのgolden graphを作る。
2. **Separate**: object reader、walker、query/tie-break policyを分ける。
3. **Implement**: targeted is-ancestorとsignature historyをiterative engineへ移す。
4. **Prove**: result equivalence、node read counter、stack非依存を確認する。
5. **Promote**: algorithm ADRと残るhotspotの正式CFRを選ぶ。

## Acceptance Signals

- common ancestor/is-ancestor/signature historyのgolden resultが移行前後で一致する。
- near ancestor queryは全history mapを構築せず、visited countが距離に比例する。
- deep graphでstack overflowせず、budget超過とcorruptionを区別して返す。

## Verification Design

- Focused tests: tie-break、multi-parent、duplicate edge、generation有無、targeted early stop
- Cross-boundary tests: merge、upload、request-apply、commit signature history
- Scale or fault tests: 100k linear、wide DAG、missing parent、tampered record、budget±1
- Observability: nodes/edges/object reads/cache hits/stop reason

## Dependencies and Duplication Check

- Existing CFB/CFR: fixed CFR-029のcommon ancestor改善を保持し、remote targeted queryとsignature recursionまで含む上位再評価。新規CFRはcounterexample確認後に限定する。RVC-014/015はTraceGraphのclosure/propagationとrequired-link complexityをownerとし、本候補はpersisted commit DAG/object graph/signature historyに限定する。
- Required predecessor: RVC-047のwork budgetとsealed validation equivalence
- Can combine with: RVC-047、RVC-052、RVC-058

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- graph databaseや履歴意味論の全面変更を前提にしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
