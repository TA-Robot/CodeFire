# RVC-003: sealed historyと長期working progressの緊張を再検証する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | product / domain / UX |
| Review kind | experiment |
| Priority | P2 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 5 |
| Primary scope | `open directory / close-discard / commit workflow / no hidden checkpoint` |
| Promotion target | strategy experiment |

## Executive Judgment

正式履歴をconsistent sealed statesだけに限定し、repository内へ復元可能なWIP snapshotを隠さない原則は維持すべきである。一方「長い作業は変更が大きすぎる」という前提が現実の調査・移行・外部依存待ちでも成立するかは独立利用者で未検証である。checkpoint追加を先に決めず、作業分割支援と損失リスクを測るべきである。

## Scope and Review Question

- In scope: unsealed files、open directory寿命、close/discard、最小consistent commitへの分割、利用者の復旧理解
- Out of scope: stash/WIP commit/checkpointの即時追加、OS backupやcloud syncの実装
- Question: consistent-only historyを保ったまま、長期作業を安全かつ理解可能に運用できるか

## Evidence

- `docs/00_philosophy.md` — `Git的自由からの離脱` / `履歴の意味を変える`: WIP commit、stash、partial commitを意図的に提供せず、commitをconsistent state sealと定義する。
- `docs/03_workflow_spec.md` — `長い作業の扱い`: checkpointはなく、大変更を複数の整合可能なcommitへ分けるよう求める。
- `docs/01_product_definition.md` — `G-005 No hidden checkpoint` / `Open Directory`: active stateへ作業中内容、patch、snapshotを保存しない。
- `crates/codefire-cli/src/main.rs` — `close_branch` / `remove_open_dir_marker_or_tree`: cleanでなければcloseを拒否し、`--discard`はopen directoryとactive stateを明示的に除去する。
- Test/fixture evidence: close/discard safety testsは破棄guardを固定するが、数日規模のfirst-seal成功率や作業分割の理解度はproduct evidenceとして存在しない。

## Strengths to Preserve

- official historyに未検証状態が混入せず、branch headの意味が単純である。
- discardが明示操作で、通常close前にfresh scan相当のdirty確認を行うこと。

## Gap / Risk

### Observed gap

仕様は作業を小さく分けられると仮定するが、分割不能なschema移行、探索的修正、外部承認待ちをどう安全に保持するかのUX契約は薄い。open directoryが唯一のWIP本体であることと、どのデータがCodeFireに保護されないかも初見では伝わりにくい。

### Failure or stagnation mode

first sealまで到達できない、open directoryをbackup済みと誤解する、または制約回避のため別VCSへ秘密のcheckpointを作り、CodeFireのauthorityが形骸化する。

### Affected invariants

- consistent-only sealed history
- no hidden checkpoint
- zero user-work loss / explicit discard

## Improvement Direction

まず長期作業のfailure taxonomyと分割可能性をpilotで測る。改善はread-onlyの「次のseal可能境界」診断、未保護WIPの明示、safe close/discard preview、外部backupとの責務説明に限定し、repository内WIP object追加は実験が原則を反証した場合のみADRで検討する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: WIP commit/checkpointを追加 | 長期作業を保存しやすい | historyの意味とG-005を破壊 | reject |
| B: 分割診断・保護境界表示を実験 | 原則を保ち摩擦を測れる | すべての作業を救えない | experiment |

## Recommended Sequence

1. **Characterize**: 代表的な長期変更で時間、open期間、外部checkpoint利用、work-loss near missを記録する。
2. **Separate**: sealed authority、active metadata、unprotected filesystem WIPの責務をUX modelにする。
3. **Implement**: status/contextへWIP durability warningとnext seal boundaryのread-only prototypeを追加する。
4. **Prove**: discard/close fault testsとuser comprehension testで誤破棄・誤認を測る。
5. **Promote**: 原則維持ならworkflow spec、反証ならproduct-boundary ADRへ送る。

## Acceptance Signals

- pilot利用者が「未commit fileはCodeFire object storeに保護されない」と正しく説明できる。
- first sealまでの時間と中断復帰失敗がbaselineより改善する。
- branch headとsealed commitへWIP contentが混入しない回帰testが維持される。

## Verification Design

- Focused tests: dirty close拒否、explicit discard、keep-dir marker removal、no active content snapshot。
- Cross-boundary tests: status/scan/close/dry-runのfreshnessとnext action整合。
- Scale or fault tests: 長期open、process crash、copied directory、pending transaction、disk failure simulation。
- Observability: open duration、pending change class、unprotected-work warningをlocal/boundedに表示し、telemetry送信はしない。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-101/105/114はclose/discardの個別欠陥でfixed。本候補はno-checkpoint product仮説の検証であり再修正ではない。
- Required predecessor: branch/open identityとrecoveryのStage 1 gate。
- Can combine with: RVC-009、RVC-010。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 実験前にstash、partial commit、WIP objectを導入しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
