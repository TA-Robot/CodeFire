# RVC-047: Large repository performance budget

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | performance / correctness / product |
| Review kind | refine |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `scan/status/commit/view/storage/remote graph workloads` |
| Promotion target | strategy experiment / future CFR |

## Executive Judgment

CodeFireは個別hotspotでindex、bounded sample、changed-blob限定読込みを導入してきたが、supported repository規模とcommand別work budgetがない。wall time目標を先に置かず、files/bytes/Atoms/links/objects/ancestryごとのdeterministic work counterとcomplexity envelopeをStage 1 gateへ置くべきである。

## Scope and Review Question

- In scope: local repositoryのscan/status/verify/commit、diff、storage、remote graph transferの支配的計算量とmemory
- Out of scope: hardware別SLA、build/CI時間、installer、外部benchmark競争
- Question: repository各次元が増えたとき、どのcommandが何を何回読むかを予算として説明・回帰検出できるか

## Evidence

- `crates/codefire-core/src/lib.rs` — `build_atom_index` / `rel_files`: tree全体を列挙・sortし、対象fileを抽出ごとに読む。
- `crates/codefire-core/src/lib.rs` — `build_scan_result_with_options` / `adjacent_atoms`: changed AtomごとにTrace Link全件を走査する経路が残る。
- `crates/codefire-cli/src/main.rs` — `build_manifest` / `build_manifest_fingerprint`: open treeを再列挙し、全fileをread/base64化してobject IDを求める。
- `crates/codefire-cli/src/remote.rs` — `collect_object_records`: reachable graph全件をValue vectorへ保持し、最後にsortする。
- Test/fixture evidence: `codefire-core::required_link_missing_counts_large_fixture_with_link_index`等の局所scale testはあるが、command横断の複数scale point/work budgetはない。

## Strengths to Preserve

- required-link count index、unknown object prefixの早期拒否、diffのunchanged blob skipなど、実測hotspotを局所改善している。
- canonical validationやcorruption diagnosticを性能のために省略しない方針がcanonical playbookにある。

## Gap / Risk

### Observed gap

「large」の軸、上限、expected complexity、memory ownershipがcommandごとに未定義で、metricsもdomain count中心で実際のfile/object read回数を持たない。

### Failure or stagnation mode

小fixtureでは通るO(changed_atoms × links)、重複tree scan、全graph materializationが実repoで支配的になり、最適化時にvalidationを弱める誘惑が生じる。

### Affected invariants

- complete scan/verification
- sealed object validation equivalence
- bounded resource use and diagnostics

## Improvement Direction

代表scenarioごとにinput dimensions、expected Big-O、peak-memory model、work counters、accepted scale pointsを定義する。最初はwall timeではなく`file_reads`、`bytes_hashed`、`object_reads/validations`、`graph_visits`、`sort_items`で回帰を固定する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: deterministic work-budget suite | machine差に強くroot causeが分かる | instrumentationとfixture生成が必要 | adopt |
| B: elapsed-time thresholdのみ | 導入が速い | noiseが大きくvalidation skipを検出しにくい | reject |

## Recommended Sequence

1. **Characterize**: files/Atoms/links/objects/historyを独立に増やすscale fixtureを作る。
2. **Separate**: command phaseと共通work counter interfaceを定義する。
3. **Implement**: repeated scan/adjacency/object materializationの最大hotspotから直す。
4. **Prove**: 3点以上のscale curveとdiagnostic equivalenceを固定する。
5. **Promote**: accepted budgetだけをstrategy experiment/正式CFRへ昇格する。

## Acceptance Signals

- 主要commandごとにinput dimension、expected complexity、peak memory、counter ceilingがある。
- 10倍scaleでcounter増分が合意したorderを超えず、corruption診断は同値である。
- optimization前後でobject identity、sealed validation、output omissionが一致する。

## Verification Design

- Focused tests: adjacency、manifest、object lookup、top-N、ancestry各counter
- Cross-boundary tests: status→scan→verify→commit、diff、remote uploadの同一fixture
- Scale or fault tests: sparse/dense graph、many-small/large-binary、deep/wide ancestry、corrupt object
- Observability: stable work countersとphase outcome。wall timeは補助値に限定

## Dependencies and Duplication Check

- Existing CFB/CFR: fixed CFB-007/008、CFR-013/024-026/051/055/061/089/105/118/135/156等の個別改善を再openせず、予算と回帰検知の上位枠を作る。
- Required predecessor: correctness-equivalence fixtureとStage 1 frozen batch完了
- Can combine with: RVC-048、RVC-049、RVC-058

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 根拠のないsub-100ms等のmarketing latency目標を置かない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
