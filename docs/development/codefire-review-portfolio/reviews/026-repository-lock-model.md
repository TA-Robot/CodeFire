# RVC-026: repository lock modelをmutation capabilityとして明文化する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | architecture / correctness / performance |
| Review kind | redesign |
| Priority | P0 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `RepoLock / FileLock / mutable repository workflows` |
| Promotion target | ADR |

## Executive Judgment

create_newによる排他、bounded wait、owner metadata、RAII cleanupは良い最小実装である。ただしlock scopeと取得順序がcommandごとの慣習に残り、RepoLockとFileLockが重複し、live scan/verifyという既知の非取得経路もある。CFB-124を先に閉じた後、lockを単なるoptionではなくmutation capabilityとして表現する設計へまとめるべきである。

## Scope and Review Question

- In scope: local repository metadata/open state、resource lock、wait/timeout、stale lock診断、lock ordering
- Out of scope: OS分散lock、remote database、immutable object publishのみのlock不要経路
- Question: 各mutable operationが必要なguardを保持し、nested operationとstale lockを一貫して扱うことを構造的に証明できるか

## Evidence

- `crates/codefire-cli/src/main.rs` — RepoLock::acquire_with_options / Drop: repo.lockをcreate_newしpid/created_atを書き、wait/timeout後にguard dropでunlinkする。
- `crates/codefire-cli/src/main.rs` — FileLock::acquire_with_options: ほぼ同じalgorithmをresource path向けに重複実装する。
- `crates/codefire-cli/src/main.rs` — compute_scan / compute_verify: persist=trueで複数open-state recordを更新できるが、関数自体はRepoLock guardを要求しない。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs` — repo_lock_wait_timeout_returns_lock_contention_with_owner_metadata はcontentionとowner表示を固定するが、stale owner、guard ownership token、全mutation coverageは固定しない。

## Strengths to Preserve

- default fail-fastと明示的なwait/timeoutを両立し、無期限hangを避けること。
- lock contentionをstable exit codeとnext actionへ変換するautomation UX。

## Gap / Risk

### Observed gap

どの関数がguardを必要とするか型に現れず、同じcompute関数がread-only previewとpersisting applyをboolで切り替える。lock fileにはprocess生存性やownership tokenがなく、stale判定・安全なbreak手順もない。

### Failure or stagnation mode

新しいmutation callerがlockを省略する、nested callerが二重取得する、crash後のstale lockをoperatorが根拠なく削除する。広いrepo lockを無思慮に増やすとread-only処理まで直列化し性能が停滞する。

### Affected invariants

- mutable state serialization
- branch/open/active convergence
- idempotency replay
- bounded liveness

## Improvement Direction

locking moduleへLockScope、LockOwnerToken、RepositoryMutationGuard、ResourceMutationGuardを集約し、write helperやapply関数がguard参照を要求する。scan/verifyはread-only computeとlocked persistを分離する。lock hierarchyとstale診断をADR化し、自動breakはpidだけで判断せず、owner token・process check・transaction stateを組み合わせて安全性が証明できる場合だけ許可する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: typed guard＋compute/apply分離 | lock抜けとnested ownershipをreview/compileで発見しやすい | caller移行が必要 | adopt |
| B: 全command冒頭で常にrepo lock | 単純 | read-only性能低下、remote resource lockとの順序不明 | reject |

## Recommended Sequence

1. **Characterize**: 全write symbol、現在lock、nested caller、idempotency順序のmatrixを作る。
2. **Separate**: locking.rsとread-only compute / locked apply境界を抽出する。
3. **Implement**: CFB-124のscan/verify persistenceを同じguard contractへ移す。
4. **Prove**: concurrent mutation、timeout、stale metadata、lock orderingを決定的testにする。
5. **Promote**: lock hierarchy、stale recovery、immutable store例外をADRへ昇格する。

## Acceptance Signals

- mutable repository writeのproduction callerがguardなしで呼べない、または一括auditでゼロになる。
- scan/verify previewはwriteせず、live applyだけが一度lockを取得する。
- stale lock diagnosticがowner evidenceと安全な次手を返し、根拠なくunlinkしない。

## Verification Design

- Focused tests: acquire/drop、timeout、owner token mismatch、stale metadata parse。
- Cross-boundary tests: fire/extinguish/link/scan/verify/commit/mergeのlock contract table。
- Scale or fault tests: thread/process contention、kill後stale lock、nested workflow deadlock。
- Observability: scope、owner、age、waited_ms、timeout、transaction関係をdiagnosticへ出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-124がlive scan/verifyの直接修正。CFR-124はmanual fireのlock前読取をfixed済み、CFR-035はremote generation lockをfixed済み。本件は共通modelの残差で、凍結batch中は実装しない。
- Required predecessor: CFB-124。
- Can combine with: RVC-024、RVC-025、RVC-037、RVC-050/056のconcurrency・race review。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- network filesystem上の万能distributed lockを約束しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
