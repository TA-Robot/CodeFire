# RVC-012: Fire lifecycleを明示的なstate machineにする

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | domain / correctness / architecture / evolution |
| Review kind | redesign |
| Priority | P0 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `codefire-core::Fire / Resolution / scan / extinguish / codefire-store ledger validation` |
| Promotion target | future CFR |

## Executive Judgment

Fireにstable UID、basis、resolution link、obsolete timestampを持たせた設計は履歴追跡に有効である。しかしlifecycleは`status: String`への直接代入で実装され、許可遷移、再発、supersede、manual/scan originの差が型で守られていない。eventとstate transitionをdomainへ集約すべきである。

## Scope and Review Question

- In scope: open/extinguished/obsolete、resolution active/superseded、refresh、再生成、origin、transition audit
- Out of scope: Fireを非blocking通知へ一般化すること、external issue tracker同期
- Question: 任意のscan/extinguish/refresh sequenceでFireとResolutionが矛盾しない状態へ決定的に収束するか

## Evidence

- `crates/codefire-core/src/lib.rs` — `Fire` / `Resolution`: 両方がstring `status`を持ち、Fireはoptional `obsolete_at`/`resolution_uid`を持つ。
- `crates/codefire-core/src/lib.rs` — `build_scan_result_with_options`: changed sourceに対応しないopen scan Fireを直接`obsolete`へ変更する。
- `crates/codefire-cli/src/main.rs` — `run_extinguish`: open確認後にFireを`extinguished`へし、refresh時は既存active Resolutionを`superseded`へする。
- `crates/codefire-cli/src/fire.rs` — manual/batch Fire planning: non-obsolete keyでduplicateを検査し、scanとは別経路でopen Fireを構築する。
- `crates/codefire-store/src/lib.rs` — `validate_fire_ledger_shape` / `validate_resolution_ledger_shape`: ledgerが配列であることだけを検査し、status語彙、extinguished FireのResolution存在、active Resolution一意性をsealed validationで拒否しない。
- Test/fixture evidence: obsolete、refresh、manual identity、batch validation testsはあるが、全状態×eventのtransition table/property testはない。

## Strengths to Preserve

- immutable sealed ledgersへFire/Resolution履歴を残し、current open setをscanで再構築できること。
- refreshが古いResolutionを消さずsupersededとして監査可能にすること。

## Gap / Risk

### Observed gap

string statusは不正値と不正組合せ（extinguishedなのにresolutionなし、obsoleteなのにtimestampなし等）を表現できる。scan/manual/mergeでidentity・obsoletion ruleが分散する。

### Failure or stagnation mode

Fireが閉じたように見えてactive resolutionがない、refresh後に複数active resolutionが残る、同じsemantic responsibilityが別originで二重発生し、verification countとledger historyがずれる。

### Affected invariants

- every extinguished Fire has valid active Resolution
- open required Fire blocks commit
- deterministic Fire identity / audit history

## Improvement Direction

`FireState`/`ResolutionState` enumと`FireEvent`（Detected、Resolved、Invalidated、Obsoleted、Reopened等）をdomain化し、transition functionが新stateとaudit eventを返す。persistenceはversioned enumを使い、legacy stringは厳格にmigrationする。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 各代入前にvalidationを追加 | 局所修正 | 経路追加でdrift再発 | reject |
| B: typed transition engine | 不正状態を集中排除 | migrationとorchestration変更 | adopt |

## Recommended Sequence

1. **Characterize**: origin×state×event×expected next state/side effectsの表を作る。
2. **Separate**: identity、occurrence、lifecycle event、resolution association、renderingを分離する。
3. **Implement**: pure transition functionを追加しscan/extinguish/manualを順次移す。
4. **Prove**: property testsでillegal transition、idempotent replay、single-active-resolutionを検証する。
5. **Promote**: domain spec/data modelと正式CFRへ昇格する。

## Acceptance Signals

- 永続化可能なFire/Resolution状態の全組合せがinvariant checkerを通る。
- 同じevent replayは同じ結果を返し、active resolutionが1件を超えない。
- legacy ledgersをlossなく読み、invalid legacy stateは明示diagnosticになる。

## Verification Design

- Focused tests: transition table、refresh、obsolete/reopen、manual/scan collision、invalid enum。
- Cross-boundary tests: scan → extinguish → edit → rescan → refresh → verify → seal/reopen。
- Scale or fault tests: batch中断、duplicate event、out-of-order event、corrupt ledger。
- Observability: previous/event/next、semantic key、occurrence、active resolution IDをbounded auditに出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-077/078とCFR-158はidentity/commit-readyの個別修正。本候補はlifecycle全体のdomain invariant/validatorを所有し、multi-write failureでこの不正状態を生成するRVC-037と一つのroot fixへ統合する。
- Required predecessor: RVC-011 decision modelと現行lock/transaction凍結batch。
- Can combine with: RVC-004、RVC-013、RVC-037。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- Fire lifecycleを外部task managerのworkflowへ同一化しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
