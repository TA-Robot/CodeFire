# RVC-037: batch atomicityとpartial failureの意味を分離する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | correctness / architecture / testing |
| Review kind | redesign |
| Priority | P0 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 1 |
| Primary scope | `batch.rs` / fire, link, evidence batch workflows |
| Promotion target | future CFR |

## Executive Judgment

全itemを最初にvalidateし、最初のinvalid itemより前にwriteしない規則は維持すべきである。しかし「validation atomicity」「logical all-or-nothing」「crash consistency」「immutable orphan許容」がbatchごとに異なるのに、すべてbatch success/failureという一語で扱われる。mutable multi-file batchにはtransaction境界を、immutable evidence batchにはresume可能なpartial resultを設計するべきである。

## Scope and Review Question

- In scope: fire/extinguish/link/evidence batch、all-items validation、multi-write crash、partial immutable writes
- Out of scope: distributed remote batch、verification command自体のtransaction化
- Question: 失敗点ごとに何件が可視化され得るか、retryが重複しないか、operatorが安全に収束できるか

## Evidence

- `crates/codefire-cli/src/batch.rs` — `run_extinguish_batch`: lock下で全itemをresolve/validate後、fires.jsonとresolutions.jsonを別々のatomic writeで更新する。
- `crates/codefire-core/src/lib.rs` — `build_scan_result_with_options`: `obsolete`以外のFire keyは再生成を抑止する一方、verificationへ渡すのは`status == "open"`のFireだけなので、resolution write失敗後の`extinguished` Fireはblockerへ戻らない。
- `crates/codefire-store/src/lib.rs` — `validate_resolution_ledger_references`: Resolutionからevidenceへの参照は検査するが、extinguished Fireの`resolution_uid`がResolution ledgerに存在することはsealed validationで相互検査しない。
- `crates/codefire-cli/src/fire.rs` — `run_manual_fire_specs`: fires ledgerを書いた後、set_open_stateがregistry、active state、branch recordを順次更新する。
- `crates/codefire-cli/src/evidence/batch.rs` — `run_evidence_batch`: 全optionをvalidate後、itemごとにrun_evidence_addし、途中失敗までのimmutable evidence/object writeは残り得る。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs` — `extinguish_batch_validates_all_items_before_writing_ledgers`、`manual_fire_and_batch_validate_before_writing_fires`、`evidence_batch_validates_all_items_before_writing_objects` はpre-write validationを固定するが、各write/item間failpointは網羅しない。

## Strengths to Preserve

- invalid inputを全件適用前にrejectし、duplicate targetやevidence refも先に検査すること。
- immutable object storeではpartial objectが既存reachable historyを壊さないこと。

## Gap / Risk

### Observed gap

extinguishの2ledger、fireのledger/state群はprocess crashで片方だけ更新され得る。evidence batchはlogical resultがerrorでも一部objectが成功済みになり得るが、resume tokenやper-item statusを返せない。

### Failure or stagnation mode

Fireはextinguishedだがresolutionがない、stateはcleanだがopen fireがある、retryでevidence object/commandを再実行する。all-items-before-write testだけで「atomic batch」と誤認する。

### Affected invariants

- Fire-resolution agreement
- open state consistency
- idempotent retry
- evidence provenance

## Improvement Direction

batchごとにAtomicityClassを宣言する。mutable multi-file applyはpending markerまたはsingle authoritative ledger＋derived state rebuildでcrash recoveryする。immutable evidenceはitem identity、completed IDs、failed indexを持つBatchExecutionRecordでresumeし、外部commandの再実行可否を明示する。resultはapplied/partial/not_appliedとper-item dispositionを返す。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: atomicity class別のtransaction/resume | domainに合う失敗意味を正確に扱える | batchごとに異なる実装が必要 | adopt |
| B: すべて一時directoryで作ってrename | 一見共通化 | 複数file/directoryや外部commandを原子的にできない | reject |

## Recommended Sequence

1. **Characterize**: batch別write set、visibility point、orphan安全性、retry副作用を列挙する。
2. **Separate**: validation plan、apply journal、result dispositionを別型にする。
3. **Implement**: extinguishのledger agreementを最初のtransaction sliceにする。
4. **Prove**: item/write間failpoint、retry、doctor recoveryを全境界で試験する。
5. **Promote**: 再現したpartial inconsistencyだけをfuture CFRへ昇格する。

## Acceptance Signals

- mutable batchの全failpoint後にdomain invariantが成立するかpending/recoverableと明示される。
- partial evidence batchは成功item IDと再実行禁止/許可を返し、resumeで重複commandを避ける。
- invalid input時のwrite 0件という既存contractが維持される。

## Verification Design

- Focused tests: duplicate、unknown target、write1/write2 failpoint、resume token。
- Cross-boundary tests: batch→status/verify/doctor→retryのend-to-end。
- Scale or fault tests: large batch、disk full、permission change、command timeout、process kill。
- Observability: atomicity class、planned/applied/partial count、visibility phase、resume actionを出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-100はextinguish batchのlockと全件validationをfixed済み。本件はRVC-012が定義するFire/Resolution invariantをmulti-write failureで破るproducerと、crash consistency/partial resultを扱う。CFR-049等のdiagnostic completenessとは別。
- Required predecessor: RVC-024のtransaction model、RVC-026のguard、RVC-036のplan fidelity。
- Can combine with: RVC-012、RVC-028、RVC-056のfault/race harness。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 外部commandの副作用をfilesystem transactionでrollback可能とは主張しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
