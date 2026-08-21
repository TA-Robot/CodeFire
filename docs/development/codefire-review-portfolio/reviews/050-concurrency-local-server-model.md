# RVC-050: Concurrency and local server model

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | correctness / security / architecture |
| Review kind | redesign |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 1 |
| Primary scope | `local repository locks / file and HTTP remote mutation` |
| Promotion target | existing CFB/CFR / ADR |

## Executive Judgment

single-thread serverからbounded per-connection threadへ進化し、branch/MR/generation/nonceのresource lockを持つ点は強い。しかしlock ownership・順序・multi-resource transactionが仕様化されず、idempotency checkはlock外、applyはbranchとMRを同時更新しながらMR lockを持たないため、local serverのconcurrency modelを明文化してroot fixする必要がある。

## Scope and Review Question

- In scope: RepoLock/FileLock、HTTP worker、resource lock hierarchy、same-key retry、branch+MR publication、stale lock
- Out of scope: multi-node server、async runtime採用、公開service autoscaling
- Question: concurrent local/file/HTTP operationをどのresource単位で直列化し、複数recordのvisibilityをどう保証するか

## Evidence

- `crates/codefire-cli/src/http.rs` — `ServerConnectionLimiter` / `spawn_http_connection_handler`: 最大32 threadでconnectionを並行処理する。
- `crates/codefire-cli/src/main.rs` — `RepoLock` / `FileLock`: create-new lock fileとRAII removeを使うが、process crash後のlease/stale recoveryはない。
- `crates/codefire-cli/src/http.rs` — `handle_http_request_review` / `handle_http_request_apply`: reviewはMR lock、applyはbranch+generation lockを取り、apply後にMRも更新するためresource ownershipが交差する。
- `crates/codefire-cli/src/remote.rs` — `review_merge_request` / `apply_merge_request`: idempotency lookupはresource lock前、record保存はmutation後である。
- Test/fixture evidence: `remote_generation_allocation_is_locked_across_threads`とslow connection testはあるが、review-vs-apply、same-key race、crash-stale lockはない。CFB-124はdirect scan/verifyのRepoLock欠落を追跡中。

## Strengths to Preserve

- slow clientが次requestを全面blockせず、overloadをboundedに拒否する。
- lock wait/timeout/owner metadataとstructured retry actionを持つ。

## Gap / Risk

### Observed gap

operation→resource set→lock order→recheck→writes→visibility pointの表がない。複数resourceを触るapply/review/idempotency/generationでlock scopeが揃わず、lock file crash recoveryも明示policyがない。

### Failure or stagnation mode

approved MRとbranch head/MR statusが競合して食い違う、reviewが二重appendされる、scan/verifyがlocked mutationへ割り込む、stale lockが永続的に操作を止める。

### Affected invariants

- branch head atomicity
- MR state transition and approval
- repository open-state consistency
- idempotent retry

## Improvement Direction

commandごとの`MutationGuard`を定義し、ordered resource keys、lock取得後revalidation、write set、visibility point、recovery markerを所有させる。lock hierarchyをrepo/project→MR/branch→generation/idempotencyのように明示し、複数resource mutationは一つのguardで扱う。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 宣言的mutation guard/lock order | raceとdeadlockをsystematically検証可能 | workflow移行が必要 | adopt |
| B: 発見ごとにlockを追加 | 小差分 | order inversionと再検証漏れが増える | reject |

## Recommended Sequence

1. **Characterize**: 全mutatorのresource/write/visibility表とbarrier raceを作る。
2. **Separate**: lock acquisition、post-lock validation、idempotency、publicationをguardへ集約する。
3. **Implement**: CFB-124、review/apply、same-key remoteから順に移行する。
4. **Prove**: deterministic scheduler/barrierとcrash failpointでstate matrixを確認する。
5. **Promote**: concurrency ADRと確認された個別CFB/CFRへ昇格する。

## Acceptance Signals

- 全mutationが宣言されたresource set/orderを使い、lock後にpreconditionを再検証する。
- review/apply/same-key/scan-vs-commit race後にbranch/MR/open/idempotencyが一貫する。
- crash後lock/transactionをdoctorが安全に診断し、勝手にuser workを破棄しない。

## Verification Design

- Focused tests: lock order、timeout、stale owner、post-lock head/status change
- Cross-boundary tests: local/file remote/HTTPで同じmutation interleaving
- Scale or fault tests: barrier race、process crash、lock file partial write、32 worker contention
- Observability: resource keys、wait duration、revalidation result、visibility/recovery phase

## Dependencies and Duplication Check

- Existing CFB/CFR: open CFB-124を包含し、fixed CFR-010/011/033/035/047/100/119/124/130等の原子性・lock各論を上位modelへ束ねる。新規issueは再現race単位に限定する。
- Required predecessor: CFB-120〜126完了、RVC-044 idempotency protocol
- Can combine with: RVC-041、RVC-042、RVC-044、RVC-056

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 分散lock/consensusを現行local serverへ持ち込まない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
