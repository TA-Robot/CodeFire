# RVC-044: Idempotency semantic contract

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | correctness / architecture / evolution |
| Review kind | redesign |
| Priority | P0 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `local/remote mutating command idempotency` |
| Promotion target | future CFR / ADR |

## Executive Judgment

canonical payload hashでsame-key/different-requestをrejectする共通helperは重要な強みである。しかしcommandごとにpayload field、保存result、replay plan、lockとの順序が異なり、特にremote mutatorではlookupがresource lock前、record保存がmutation後なので、同一keyの同時実行を一度だけの効果として証明できない。

## Scope and Review Question

- In scope: key scope、semantic payload、lookup/apply/record atomicity、replay result、retention/corruption
- Out of scope: distributed consensus、外部workflow engine、非mutating command cache
- Question: 同じkey+payloadが同じ効果/resultを返し、異なるpayloadが必ずconflictし、raceでも二重適用されないか

## Evidence

- `crates/codefire-cli/src/idempotency.rs` — `verify_idempotency_record` / `idempotency_payload_hash`: canonical JSON hashを比較する共通contractがある。
- `crates/codefire-cli/src/remote/idempotency.rs` — `save_remote_idempotency_result` / `replay_plan`: remoteはhash-only recordと再構築したreplay planを使う。
- `crates/codefire-cli/src/remote.rs` — `review_merge_request`: idempotency lookup後にMR lockを取得し、review append後にrecordを保存するため、同時requestは双方がmissを観測できる。
- `crates/codefire-cli/src/import_workflow.rs` — `load_import_idempotency_marker`: current open issue CFB-126の通り、importだけpayload bindingがない。
- Test/fixture evidence: `remote_mutator_idempotency_replays_same_payload_and_rejects_conflicts`等は逐次replayを検証するが、same-key raceはない。

## Strengths to Preserve

- key空文字拒否、payload canonicalization、stable conflict errorが共通化されている。
- remote recordはpayload全文を保存せずhash-onlyへ改善されている。

## Gap / Risk

### Observed gap

idempotency transactionがresource lock/visibility pointと一体ではない。local recordsはpayloadや古いplanを保存する一方remoteはminimal resultを保存し、command/version fieldの検証も統一されていない。

### Failure or stagnation mode

同一review requestが二重appendされる、generationが余分に進む、mutation成功後record書込み失敗でretryが再適用される、replay時に古いpath/next actionを返す可能性がある。

### Affected invariants

- exactly-once semantic effect under retry
- branch/MR state transition
- automation result truthfulness

## Improvement Direction

versioned `IdempotencyContract {operation, scope, key, request_hash, result_subject}` を定義し、resource lock取得後に再lookup、precondition再検証、apply、result record publicationを一つのmutation protocolにする。replayではimmutable domain resultを復元し、現在文脈のplan/next actionは再renderする。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: lock内check-apply-record protocol | raceを含め意味を証明できる | 各workflow移行とcrash設計が必要 | adopt |
| B: 現行helperへfieldを追加 | 差分が小さい | mutation/record間crashとraceを解決しない | reject |

## Recommended Sequence

1. **Characterize**: command別payload/result/lock/visibility matrixとsame-key raceを固定する。
2. **Separate**: 共通record schema、immutable replay result、rendererを抽出する。
3. **Implement**: CFB-126を含めlock内再lookupとrecord publication protocolへ移行する。
4. **Prove**: concurrent retryと各failpointでeffect count=1を確認する。
5. **Promote**: idempotency ADR、data model catalog、必要な正式CFRへ反映する。

## Acceptance Signals

- N並列の同一requestでdomain mutationは1回、全callerは同じsubject resultを得る。
- same key/different payloadはwrite前にstable conflictとなる。
- mutation後record失敗のrecoveryがdoctor/再試行で曖昧にならない。

## Verification Design

- Focused tests: 全mutatorのsame/different payload、record version/command/key corruption
- Cross-boundary tests: local/file remote/HTTP remote/importの共通fixture
- Scale or fault tests: barrier付きsame-key race、visibility point前後のrecord write failure
- Observability: replayed/created/conflict/recoveredとrequest hash prefixをsecretなしで記録

## Dependencies and Duplication Check

- Existing CFB/CFR: open CFB-126を包含する上位contract候補。fixed CFR-067〜070、CFR-100の個別修正を再採番せず統合する。
- Required predecessor: CFB-126の凍結batch判断、resource lock ownership確定
- Can combine with: RVC-041、RVC-050、RVC-056、RVC-057

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 複数node分散transactionを現段階のsupported contractにしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
