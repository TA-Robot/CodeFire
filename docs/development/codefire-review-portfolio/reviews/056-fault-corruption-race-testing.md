# RVC-056: Fault, corruption, and race testing

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | testing / correctness / security |
| Review kind | redesign |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 1 |
| Primary scope | `mutation/recovery/store/remote fault verification` |
| Promotion target | future CFR / strategy experiment |

## Executive Judgment

doctor corruption fixtures、commit pending marker、atomic temp-path tests、generation concurrency testは強い土台である。しかしfailpointはcommitの一phaseへ偏り、raceは少数thread test、corruptionは手作業の点検で、Stage 1が求める全crash window・counterexample matrixをsystematicに生成できない。

## Scope and Review Question

- In scope: filesystem fault、partial write、process crash、corrupt object/state、lock/idempotency race、recovery diagnostics
- Out of scope: OS/kernel fault injection framework、chaos service運用、CI基盤
- Question: 各visibility point前後の失敗とconcurrent interleavingをdeterministicに再現し、安全state/recoveryを証明できるか

## Evidence

- `crates/codefire-cli/src/main.rs` — `begin_commit_transaction` / `update_commit_transaction` / `maybe_fail_commit_transaction_for_test`: pending markerはあるが注入点は実質`after_content_manifest`一箇所である。
- `crates/codefire-cli/src/tests.rs` — `commit_transaction_marker_is_clean_after_success_and_doctor_reports_pending`: 一つの中断phaseとdoctor検出を確認する。
- `crates/codefire-cli/src/tests.rs` — `doctor_reports_clean_repo_and_corrupted_object_record`: object hash改変を手作りして診断する。
- `crates/codefire-cli/src/tests.rs` — `remote_generation_allocation_is_locked_across_threads`: generation単体raceは`Barrier`同期のthread testで確認するが、resource交差や各write間の決定的interleaving schedulerは持たない。
- `crates/codefire-cli/src/http.rs` — connection limiter/slow request tests: transport concurrencyの局所behaviorはあるがdomain mutation raceへ接続しない。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs::commit_transaction_marker_is_clean_after_success_and_doctor_reports_pending`、`doctor_reports_clean_repo_and_corrupted_object_record`、`remote_generation_allocation_is_locked_across_threads`はそれぞれ単一phase/corruption/Barrier同期raceを覆うが、共通phase matrixやwrite単位schedulerではない。

## Strengths to Preserve

- pending transactionを正常objectと混同せずdoctor recovery対象にする。
- immutable object write、atomic JSON temp path、resource lockの局所testsがある。

## Gap / Risk

### Observed gap

filesystem operationを抽象化したfault seam、全phase一覧、barrier scheduler、corruption generatorがなく、production codeへ個別`#[cfg(test)]` hookを足す形になる。

### Failure or stagnation mode

branch publish後/registry前、MR apply中、idempotency record前、nonce cache中等の未検証windowが残り、実障害で初めてrepair不能stateが見つかる。flaky timing testではraceを再現できない。

### Affected invariants

- branch head and multi-file state atomicity
- immutable object integrity and recovery
- lock/idempotency/replay correctness

## Improvement Direction

named mutation phasesとinjectable filesystem/clock/barrierを持つtest-only fault harnessを作る。各workflowはwrite set/visibility point/recovery expectationをtable化し、corruption corpusとpairwise race scheduleを同じfixture builderで実行する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 共通deterministic fault harness | crash/race matrixを再利用可能 | IO seamの設計が必要 | adopt |
| B: bugごとのsleep/manual corruption | 小差分 | flakyでcoverage holeが見えない | reject |

## Recommended Sequence

1. **Characterize**: commit/open/scan/verify/remote/idempotencyのphase/write/visibility表を作る。
2. **Separate**: filesystem、clock、barrier、fault selectorをtest seamへ抽出する。
3. **Implement**: commit全phaseとCFB-124/126 raceからmatrix化する。
4. **Prove**: 各注入点でallowed stateまたはdoctor-guided recoveryだけが残ることを確認する。
5. **Promote**: 確認した実bugだけをCFB/CFRへ昇格し、Stage 1 gateへ接続する。

## Acceptance Signals

- 全declared mutation phaseにbefore/after failpointとexpected persistent stateがある。
- race testがsleepに依存せず同じinterleavingを再現する。
- corruption/fault後にsilent success、unsealed head、user work lossがない。

## Verification Design

- Focused tests: write/rename/fsync/remove/lock/time faultとstate validator
- Cross-boundary tests: commit/doctor/retry、HTTP remote/retry、Python/Rust fixture read
- Scale or fault tests: phase matrix、pairwise races、truncated JSON、disk-full、stale lock
- Observability: injected phase、writes completed、visibility reached、recovery class

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-120〜126、CFR-010〜012/119/124/130等の個別issueを置換せず、再現・proof基盤を提供する候補。新issueはharnessが示したcounterexampleだけ採番する。
- Required predecessor: frozen batchのinvariant/visibility point確定
- Can combine with: RVC-044、RVC-045、RVC-050、RVC-054、RVC-055

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- test harnessをproduction featureや一般chaos platformとして公開しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
