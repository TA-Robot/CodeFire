# RVC-024: commit transactionの全crash windowを状態機械で証明する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | correctness / architecture / testing |
| Review kind | redesign |
| Priority | P0 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `main.rs run_commit / commit_transaction.json / doctor` |
| Promotion target | future CFR |

## Executive Judgment

pending marker、phase、stored object一覧を導入した現行transactionは、無印の連続writeより大幅に良い。しかし実証済みfailpointは最初のroot保存後に偏り、doctorはpendingを発見するだけで安全なresume/abort判定を持たない。branch headをvisibility pointと定義した状態機械と全phase failpoint matrixへ完成させるべきである。

## Scope and Review Question

- In scope: commit marker、immutable object write、head publish、opened/active reset、retry/recovery
- Out of scope: distributed transaction、remote MR apply、filesystem一般の完全なACID保証
- Question: 任意のwrite直前/直後で停止しても、old headかvalid new headのどちらかに収束し、workを失わず説明・再試行できるか

## Evidence

- `crates/codefire-cli/src/main.rs` — begin_commit_transaction / update_commit_transaction: pending markerへphaseと保存済みobject IDをdurable atomic writeする。
- `crates/codefire-cli/src/main.rs` — run_commit: root群、commit、branch head、opened registry、active reset、marker削除を順次実行するがmulti-file rollback/resumeは実装しない。
- `crates/codefire-cli/src/doctor/checks.rs` — check_active_state_files: pending markerのphaseとobject数をerrorとして報告するが、headとの関係からrecovery stateを分類しない。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs` — commit_transaction_marker_is_clean_after_success_and_doctor_reports_pending は after_content_manifest の1failpointを確認し、test自身がmarkerを手動削除してから再commitする。

## Strengths to Preserve

- immutable objectはpublish前に書け、orphanになっても既存sealed historyを壊さない設計。
- marker自体をatomic/durable writeし、phaseとobject IDを残す診断可能性。

## Gap / Risk

### Observed gap

pre-head、post-head/pre-registry、post-registry/pre-active-reset、post-reset/pre-marker-deleteのcrash状態と再実行規則がコード・test・doctorで閉じていない。

### Failure or stagnation mode

operatorがpending markerを削除すべきかresumeすべきか判断できず、branch headとopen baseが食い違う。安易なreopen/resetは未commit作業を失い、安易なretryは二重commitや異なるparentを作り得る。

### Affected invariants

- branch head atomicity
- open work preservation
- idempotent commit retry
- recovery explainability

## Improvement Direction

transaction recordへexpected old head、candidate new head、open instance、publication stateを持たせ、phaseを列挙型として扱う。recovery classifierはmarker、branch head、registry base、active stateをread-onlyに照合し、safe_resume、safe_finalize、safe_abort_orphan_only、manual_interventionへ分類する。自動repairは分類とfixtureが証明された遷移だけに限定する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: explicit phase/state machine＋idempotent recovery | 全crash windowを同じ規則で説明できる | marker schemaとrecovery実装が増える | adopt |
| B: 失敗時は常にmarker削除して再commit | 実装が簡単 | post-publishでhead/base divergenceや二重commitを起こす | reject |

## Recommended Sequence

1. **Characterize**: 各durable writeの前後へfailpointを列挙しstate tableを作る。
2. **Separate**: CommitTransaction、ObservedCommitState、RecoveryDecisionを専用moduleへ移す。
3. **Implement**: read-only classifier、次にsafe finalize/resumeを一遷移ずつ追加する。
4. **Prove**: 全failpointでreachable work、head validity、retry convergenceを検査する。
5. **Promote**: CFR-130の残差ではなく新しいrecovery CFR/ADRとして、凍結batch後に昇格判断する。

## Acceptance Signals

- 全phase failpoint後にdoctorが一意のrecovery classと根拠を返す。
- invalid/new未検証commitがheadにならず、old/new両方のreachable workを失わない。
- 同じmessage/idempotency keyのretryが同じ最終状態へ収束する。

## Verification Design

- Focused tests: marker schema、phase transition、unexpected transaction ID、candidate mismatch。
- Cross-boundary tests: commit→doctor→recover→status/showのend-to-end matrix。
- Scale or fault tests: every-write failpoints、process kill、stale marker、partial JSON/corrupt object。
- Observability: phase、expected/current/candidate head、recommended action、manual理由をJSONへ出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-130はpending marker導入でfixed。本件は全crash windowとrecovery completionという未包含の次段階。CFB-121のpublish前sealed validationが前提。
- Required predecessor: CFB-121、RVC-023。
- Can combine with: RVC-025、RVC-028、RVC-030、RVC-056のsystematic failpoint testing。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 複数machine間のdistributed consensusを導入しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
