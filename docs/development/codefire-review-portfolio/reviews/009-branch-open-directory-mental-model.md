# RVC-009: branchとopen directoryのmental modelを利用者実験で磨く

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | product / UX / domain |
| Review kind | experiment |
| Priority | P2 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 5 |
| Primary scope | `branch / open / close / status / repository layout` |
| Promotion target | strategy experiment |

## Executive Judgment

current branchを廃し、branchごとに独立directoryを開く設計は同時作業とauthorityを明示する強みである。一方branch record、opened registry、marker、active stateという複数の真実が利用者の単純な「branch=folder」理解を超え、状態名もsealed headとworking stateを混ぜやすい。内部不変条件は維持し、外部mental modelとcommand languageを検証すべきである。

## Scope and Review Question

- In scope: repository/branch/open instance/base/head/work stateの説明、open/clone/close/list/status UX
- Out of scope: checkout/current branchの導入、folder copyを正式branch操作にすること
- Question: 初見利用者がbranch historyとworking directoryの別物性、破棄・seal・cloneの意味を誤らず操作できるか

## Evidence

- `docs/01_product_definition.md` — `G-006 Multi-branch workspace` / `Open Directory`: current branchを持たず、marker・registry・active state・head・baseの相互検証を要求する。
- `crates/codefire-cli/src/main.rs` — `open_branch_from`: sealed headをmaterializeし、marker、opened registry、active state、branch stateを作る。
- `crates/codefire-cli/src/main.rs` — `open_context`: markerを入口にregistry pathとopen instanceを照合するが、branch/head/activeの完全matrixは個別commandへ分散する。
- `crates/codefire-cli/src/main.rs` — `close_branch` / `list_branches` / `read_status`: closeはopen workを除去し、listはsealed headsを検証し、statusはopen contextを前提にworking stateを返す。
- Test/fixture evidence: open/close/copy/nested-path lifecycle testsは多数あるが、用語理解・task completion・誤操作率の外部user testはない。

## Strengths to Preserve

- branch headとworking filesを同じdirectory stateに暗黙合成せず、multiple branchを同時に開けること。
- copied/stale open directoryをidentity mismatchとして拒否し、明示discardなしにdirty workを捨てないこと。

## Gap / Risk

### Observed gap

`clone`がbranch作成、`open`がmaterialization、`close`がdirectory除去を意味し、一般的VCS語彙とずれる。`open-clean/open-burning/open-consistent/closed`はbranch recordにもopen workにも現れ、sealed head自体の整合性と作業状態を混同しやすい。

### Failure or stagnation mode

利用者がfolder copyをbranch cloneとみなす、closeをhistory削除と恐れる、open-consistentを既にsealedと誤解する、またはtarget path/repository rootを取り違える。

### Affected invariants

- branch head sealed-only
- open identity agreement
- zero user-work loss / explicit lifecycle

## Improvement Direction

`sealed branch line`、`open instance`、`working state`を別概念としてtask-based user testする。結果に基づき、read-only `branch show`/`status`へhead/base/open path/next transitionの統一viewと用語説明を追加する。内部4-record modelを一つに潰さず、projectionを一つにする。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: Git用語へ寄せcheckout/currentを追加 | 見慣れている | product invariantと同時openを損なう | reject |
| B: lifecycle projectionとuser test | authorityを保ち理解を改善 | UX調査とrenderer変更 | experiment |

## Recommended Sequence

1. **Characterize**: clone/open/close/discard/mergeのtask testで誤解箇所を記録する。
2. **Separate**: sealed line、open instance、working state、filesystem pathのview modelを定義する。
3. **Implement**: branch/status JSONとtextに共通lifecycle projectionをprototype追加する。
4. **Prove**: corruption時にもprojectionが強いclaimを作らずUNKNOWN/recoveryを示すtestを行う。
5. **Promote**: workflow/CLI specと用語集へ採用結果を反映する。

## Acceptance Signals

- user testでhead/base/open path、close/discardの結果を正しく予測できる。
- lifecycle projectionがmarker/registry/branch/active不一致をcleanへ丸めない。
- current branchやhidden checkout stateを追加せず複数openの利点を維持する。

## Verification Design

- Focused tests: closed/open-clean/burning/consistent、copied marker、stale base、keep-dir。
- Cross-boundary tests: clone → open → edit/scan → commit → close → reopenのprojection snapshot。
- Scale or fault tests: 多数open branches、missing record、partial open/close crash、nested/symlink path。
- Observability: head/base/open instance/path/state sourceとrecovery actionをbounded表示する。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-101/105/108/110/111/114/119はlifecycleの個別bug。本候補は修正済みguardを保ちながらmental modelを測るため重複しない。
- Required predecessor: Stage 1 branch/open identity matrixとrecovery gate。
- Can combine with: RVC-002、RVC-003、RVC-010。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- Git checkout/current branch semanticsを復活させない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
