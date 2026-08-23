# RVC-025: branch head publicationを単一のcompare-and-publish境界にする

## Review Summary

| Field | Value |
|---|---|
| Status | implemented |
| Perspective | architecture / correctness / evolution |
| Review kind | redesign |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 1 |
| Primary scope | `main.rs save_branch_record / local and remote head publication` |
| Promotion target | ADR |

## Executive Judgment

branch recordのatomic replaceとrepository lockは維持すべき土台である。ただし「新headはsealed」「期待した旧headからだけ更新」「mutable headが唯一のvisibility point」「open metadataはrecover可能」というpublication contractが一つのAPIに表現されていない。local commitとremote mutationが共有できるcompare-and-publish primitiveを設計し、CFB-121の局所fixを再発防止境界へ発展させる価値がある。

## Scope and Review Question

- In scope: branch record head/state更新、expected head、sealed precondition、publication receipt
- Out of scope: merge algorithm、remote authentication、branch naming UX
- Question: 全head mutationが同じpreconditionとvisibility semanticsを使い、stale writerやpartial follow-upを検出できるか

## Evidence

- `crates/codefire-cli/src/main.rs` — save_branch_record: mutable branch JSONを先にatomic replaceし、その後branch payloadをimmutable object storeへ保存する。
- `crates/codefire-cli/src/main.rs` — run_commit: commit object保存後にbranch head/stateを更新し、その後opened registryとactive stateを別writeで更新する。
- `crates/codefire-cli/src/remote.rs` — upload_branch / apply_merge_request: resource lockとgenerationを用いる独立publication pathを持ち、local helperとは別のmutation境界である。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs` — commit_transaction_marker_is_clean_after_success_and_doctor_reports_pending はpre-publication crashのみを扱い、expected-head CASやpost-head crashのpublication receiptを固定しない。

## Strengths to Preserve

- write_json_atomicによる一つのbranch record内のhead/state同時置換。
- local RepoLock、remote resource/generation lockによる通常のwriter直列化。

## Gap / Risk

### Observed gap

save_branch_recordはexpected old headもValidatedCommitも要求せず、branch object mirror保存との順序・失敗意味もAPI契約にない。commit callerはhead publish後のopen-state同期を個別に続ける。

### Failure or stagnation mode

将来の新commandがsealed validationやstale-head確認を省略する。headだけ新しくopen baseが古いcrash状態を通常状態と誤認し、retryが誤ったparentから進む。

### Affected invariants

- branch head sealed
- linearizable publication per branch
- commit parent/base agreement
- recovery visibility

## Improvement Direction

publish_branch_head(repo, branch, expected_head, validated_new_head, target_state, transaction_id) を唯一のmutation primitiveにし、locked contextまたはresource guardを型で要求する。成功時はold/new headとpublication epochをreceiptとして返し、follow-up open metadata同期はRVC-024のrecovery stateから再実行可能にする。immutable branch objectの役割もaudit snapshotか不要なduplicateかをADRで決める。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: validated CAS publication API | stale writerとvalidation抜けを一境界で防ぐ | local/remote adapterとmigrationが必要 | adopt |
| B: 各callerでload→compare→saveを継続 | 小差分 | policy driftとTOCTOUを再生産する | reject |

## Recommended Sequence

1. **Characterize**: 全head writer、lock scope、validation、generation、follow-up writeを表にする。
2. **Separate**: BranchHeadUpdate、ExpectedHead、PublicationReceiptを専用module型にする。
3. **Implement**: local commitを最初のadapterにし、remoteはsemantic差を保って接続する。
4. **Prove**: stale expected head、invalid sealed commit、concurrent writer、post-publish crashを試験する。
5. **Promote**: visibility pointとbranch object mirrorの決定をADRへ昇格する。

## Acceptance Signals

- invalid commitとstale expected headはbranch JSON write前に拒否される。
- 一branchの同時writerは高々一つだけ成功し、loserはactual headを含むstructured conflictになる。
- publish後crashをdoctorがreceipt/transactionから識別し、安全にopen metadataを収束できる。

## Verification Design

- Focused tests: CAS success/mismatch、sealed precondition、branch state transition table。
- Cross-boundary tests: local commit、clone、remote upload/applyが共通contract testを通る。
- Scale or fault tests: concurrent publishersとhead write前後failpoint。
- Observability: old/new/actual head、publication epoch、transaction IDをbounded diagnosticへ出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-121はlocal pre-publication validationの直接issue。本件はwriter全体のAPI/ADRであり、凍結中は新issue化しない。CFR-011はdurable atomic write自体をfixed済み。
- Required predecessor: CFB-121、RVC-023。
- Can combine with: RVC-024、RVC-026、RVC-028。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- branch publicationを分散consensus問題へ拡張しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
