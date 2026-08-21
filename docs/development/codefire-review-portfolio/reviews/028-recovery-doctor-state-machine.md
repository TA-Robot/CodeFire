# RVC-028: doctorを検出器からrecovery decision engineへ発展させる

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | UX / correctness / architecture |
| Review kind | refine |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `doctor checks/report / interrupted repository recovery` |
| Promotion target | product spec |

## Executive Judgment

doctorはlayout、object、branch、opened、activeを一度に診断し、blocking/repairableをmachine-readableに出す強い基盤になっている。しかしpending commitは検出されてもcategory、repairability、専用next actionへ接続されず、現行testもmarkerを手で消して先へ進む。自動修復を急がず、まず観測状態から安全なrecovery decisionを返す状態機械にするべきである。

## Scope and Review Question

- In scope: pending commit、open metadata divergence、stale lock、orphan object、doctor issue→safe action
- Out of scope: arbitrary corrupt payloadの自動再生成、working filesの無断削除
- Question: operatorが内部実装を知らなくても、保存すべきworkと安全な次の一手をdoctorだけで判別できるか

## Evidence

- `crates/codefire-cli/src/doctor/checks.rs` — run_checks: layout、object records、branch heads、opened registry、active stateを一reportへ集約する。
- `crates/codefire-cli/src/doctor/checks.rs` — check_active_state_files: pending_commit_transactionをphase/object数付きerrorとして検出する。
- `crates/codefire-cli/src/doctor/report.rs` — doctor_report_next_actions / issue_category / issue_repairable: 多くのissueをreopen/migrate等へ接続するがpending_commit_transactionを専用分類しない。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs` — commit_transaction_marker_is_clean_after_success_and_doctor_reports_pending はpending検出後にmarkerを手動removeし、doctorによるdecision/recoveryを検証しない。

## Strengths to Preserve

- doctorがread-onlyであり、診断実行だけでrepositoryを変更しないこと。
- issueごとのcategory、blocking、repairable、pathとautomation next actionを返すこと。

## Gap / Risk

### Observed gap

issue列挙からrecovery stateへの合成がなく、個々のwarning/errorが同時に存在したときの優先順位と禁止actionが分からない。

### Failure or stagnation mode

operatorが「markerを消す」「reopenする」など局所actionを選び、publish済みcommitの追跡情報やunsealed workを失う。doctorが問題数を増やしてもmean-time-to-recoveryが改善しない。

### Affected invariants

- recovery safety
- user work preservation
- branch/open convergence
- reachable object retention

## Improvement Direction

DoctorIssueの上に RecoveryAssessment を追加し、observed facts、state class、safe actions、forbidden actions、manual evidence requirementsを返す。repair commandはassessment IDとexpected state fingerprintを要求し、read後に状態が変われば再診断させる。pending transaction、stale lock、open divergenceを最初のdecision tableとする。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: read-only assessmentを先に実装しrepairを段階追加 | 誤修復を避けつつUXを改善 | state table整備が必要 | adopt |
| B: issue kindごとに即自動fix | 操作回数が少ない | 複合corruptionでwork lossを招く | reject |

## Recommended Sequence

1. **Characterize**: stale lock、pending phase、head/base divergence、orphanのみのrecovery tableを作る。
2. **Separate**: diagnosis fact、recovery classification、mutation repairを別module/型にする。
3. **Implement**: doctor JSONへassessmentとforbidden actionsを追加し、safe finalizeを一つだけprototypeする。
4. **Prove**: assessment fingerprint後のrace、再実行、manual interventionをfixture化する。
5. **Promote**: recovery UXとnon-destructive guaranteeをproduct specへ昇格する。

## Acceptance Signals

- pending transaction全phaseがsafe_resume/finalize/abort/manualのいずれかに根拠付き分類される。
- read-only doctorは一切writeせず、repairはstate fingerprint mismatchで停止する。
- どの自動actionもworking fileまたはreachable objectを無言で削除しない。

## Verification Design

- Focused tests: issue合成、priority、forbidden action、assessment fingerprint。
- Cross-boundary tests: commit/open/lock corruption→doctor→repair→doctor clean。
- Scale or fault tests: every-phase failpoint、複数同時issue、repair直前race。
- Observability: assessment ID、facts、decision reason、RTO step数、manual blockerを出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-033のrepair next actions、CFR-148/149のactive診断はfixed済み。本件はpending/複合状態のdecision engineで非重複。CFR-130のmarkerを前提にする。
- Required predecessor: RVC-024、RVC-027。
- Can combine with: RVC-026、RVC-030、RVC-056のfault/corruption matrix。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- doctor実行だけで自動repairを開始しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
