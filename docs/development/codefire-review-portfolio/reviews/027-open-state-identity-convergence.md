# RVC-027: open workspace identityを一つの検証済みsnapshotへ収束させる

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | domain / architecture / correctness |
| Review kind | redesign |
| Priority | P0 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `.codefire-open / opened registry / active state / branch record` |
| Promotion target | future CFR |

## Executive Judgment

marker、repository-side registry、active directory、branch recordを分けることで、worktree側とauthority側の双方から復旧材料を持てる点は維持したい。一方、共通open_contextが照合するのはpathとopen_instance_idに限られ、残りのidentity/state agreementは個別commandとdoctorへ分散する。全workflowが同じ VerifiedOpenSnapshot を受け取る形へ収束させるべきである。

## Scope and Review Question

- In scope: repository ID、branch、opened/base commit、open instance/path、active path、branch/open state
- Out of scope: target path symlink containment（CFB-122）、working file contentのscan
- Question: 4つのmutable recordのどれかが古い・差替え・欠損でも、commandが一部だけをauthorityとして進まないか

## Evidence

- `crates/codefire-cli/src/main.rs` — open_branch_from: marker、opened registry、active state、branch stateを別々のatomic writeで作成する。
- `crates/codefire-cli/src/main.rs` — open_context: markerとregistryのopened path、open_instance_idだけを照合し、repository_id、branch head/base、active path ownership、state agreementは返却前に統合検証しない。
- `crates/codefire-cli/src/doctor/checks.rs` — check_opened_registry / check_open_marker / check_active_state_files: marker identity、base sealing、active shapeをより広く検査するがcommand pathとは別実装である。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs` — doctor_checks_layout_active_state_and_open_marker_shape と doctor_detects_invalid_branch_head_and_open_registry は個別corruptionを検出するが、全field組合せの共通command拒否matrixはない。

## Strengths to Preserve

- open markerだけを唯一authorityにせず、repository側registryを照合する防御。
- doctorがmarker、registry、active、branchを横断診断し、reopen guidanceを出すこと。

## Gap / Risk

### Observed gap

open identityの不変条件がwriter、open_context、status、lifecycle command、doctorに分散し、各consumerが必要なsubsetだけを再検査する。

### Failure or stagnation mode

corruption時にstatusは動くがcloseは拒否するなどcommand間の見解が割れる。古いbranch stateやbaseを信頼してworkをdiscard/materializeし、復旧可能なローカル変更を危険にさらす。

### Affected invariants

- branch/open marker/opened registry/active identity agreement
- sealed current base
- user work preservation

## Improvement Direction

OpenIdentityとOpenStateSnapshotを専用moduleへ置き、marker・repo.json・branch・registry・active stateを一度読み、fieldごとのauthorityと許容されるin-flight divergenceを検証する。consumerはVerifiedOpenSnapshotまたは明示的CorruptOpenStateを受け取り、state mutationはsnapshotのgeneration/identityをexpected条件として行う。doctorも同じvalidatorの詳細reportをrenderする。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 共通snapshot validator＋typed corruption | commandとdoctorの判断が一致する | transaction中の許容状態tableが必要 | adopt |
| B: open_contextへif文を継ぎ足す | 局所修正が早い | 診断・mutation・doctorの重複が残る | reject |

## Recommended Sequence

1. **Characterize**: identity field × record × authority × transaction phaseのdecision tableを作る。
2. **Separate**: open_state moduleへloader、validator、typed issueを抽出する。
3. **Implement**: read-only status/contextから導入し、close/commitへ段階移行する。
4. **Prove**: 1fieldずつtamperしたmatrixとin-flight transaction fixtureを通す。
5. **Promote**: 残る実装gapのみfuture CFRへ昇格し、doctor固有issueを増やさない。

## Acceptance Signals

- 同じcorrupt fixtureに対して全lifecycle commandとdoctorが同じissue kindを返す。
- valid openとdocumented in-flight stateは拒否されず、unknown divergenceだけがblockingになる。
- close/discard/open/commitはverified identityなしにworkを削除・base更新しない。

## Verification Design

- Focused tests: repository_id、branch、open_instance、path、base、active path、state各fieldのtamper。
- Cross-boundary tests: open→scan→commit/closeとdoctorのdiagnostic equivalence。
- Scale or fault tests: writer各stepのfailpoint、marker/registry同時更新race。
- Observability: authority、expected/actual、record path、repair safetyをstructured issueへ含める。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-053、CFR-148/149はdoctor検査をfixed済み。CFB-119はopened registry欠損時のopen判断をfixed済み。本件は共通validatorへの収束で非重複。
- Required predecessor: RVC-024のin-flight state定義、CFB-120/121のsealed base/publish。
- Can combine with: RVC-028、RVC-026。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- open-state metadataを一つの巨大JSONへ物理統合することを目的にしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
