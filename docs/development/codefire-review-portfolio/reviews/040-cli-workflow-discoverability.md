# RVC-040: CLIをcommand一覧からtask-oriented workflowへ案内する

## Review Summary

| Field | Value |
|---|---|
| Status | accepted_non_goal |
| Perspective | product / UX / testing |
| Review kind | experiment |
| Priority | P2 |
| Confidence | exploratory |
| Roadmap stage | Stage 5 |
| Primary scope | `completion.rs` / `subcommand_help` / status and next actions |
| Promotion target | strategy experiment |

## Executive Judgment

全commandのhelp、shell completion、contextual next actionが揃いつつある点は維持したい。しかしtop-level helpは多数commandを一列に並べ、利用者が「既存projectを始める」「変更を検査する」「Fireを解消する」「seal/shareする」「復旧する」というtask flowを自力で組み立てる必要がある。command renameを急がず、task groupingとguided examplesをuser testで比較するべきである。

## Scope and Review Question

- In scope: top-level help、subcommand navigation、task grouping、examples、statusからの次手、completion discoverability
- Out of scope: install/PATH、GUI、agent orchestration、command自動実行
- Question: 初見利用者が内部domain用語を完全に知らなくても、正しいread-only確認とfirst sealまで迷わず到達できるか

## Evidence

- `crates/codefire-cli/src/completion.rs` — `help_text`: COMMAND_NAMESを空白区切りの一行へ並べ、common例は一部commandだけを列挙する。
- `crates/codefire-cli/src/main.rs` — `subcommand_help`: 各commandのusageと短い説明を大きなmatchで返すが、task間の前後関係は局所的である。
- `crates/codefire-cli/src/completion.rs` — bash/zsh completion: command別optionは発見できる一方、open→scan→verify→commit等のworkflow intentは表現しない。
- Test/fixture evidence: `crates/codefire-cli/src/tests/docs.rs` — `help_and_completion_scripts_cover_rust_default_cli` とhelp command set testはcoverageを固定するが、task completion/理解度は評価しない。

## Strengths to Preserve

- help-before-mutationにより、どのcommandでも安全にusageを確認できること。
- status/scan/verify/doctorがcontextual next_actionsを返し、単なる静的manualに閉じないこと。

## Gap / Risk

### Observed gap

command vocabularyがflatで、initとimport、openとclone、closeとdiscard、context/explain/review-pack、doctor/migrate/storageの使い分けをtask単位で比較できない。

### Failure or stagnation mode

利用者が誤った入口commandを選ぶ、必要なscan/verifyを飛ばす、recovery commandを通常操作として使う。機能が増えるほどhelp coverageは上がってもtime-to-first-valueが悪化する。

### Affected invariants

- safe workflow ordering
- explicit destructive intent
- product comprehensibility

## Improvement Direction

既存command名を保ちつつ、helpを Start / Assess / Resolve / Seal and Share / Inspect and Recover のtask groupへ整理し、各groupに最小safe pathと比較表を置く。status/doctorの次手と同じaction vocabularyをhelp例にも使う。read-only guided commandまたは help workflow は実験候補だが、mutationを自動連鎖しない。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: task grouping＋guided examplesをA/B user test | command互換を保って理解度を測れる | docs/help同期と調査が必要 | experiment |
| B: domain用語commandを即rename/alias | 表面を簡単にできる可能性 | compatibilityと概念一貫性を壊す | reject |

## Recommended Sequence

1. **Characterize**: onboarding、daily change、Fire resolution、recoveryのtask test baselineを取る。
2. **Separate**: command descriptorのtechnical metadataとtask-oriented presentationを分ける。
3. **Implement**: top-level helpのgrouped prototypeと4本のsafe workflow exampleを作る。
4. **Prove**: first correct command、first seal時間、unsafe attempt、help往復数を比較する。
5. **Promote**: measured改善があればCLI UX specへ昇格し、command renameは別decisionにする。

## Acceptance Signals

- 既存全commandがgroupまたはadvanced/debug分類から発見でき、coverage regressionがない。
- user testでfirst safe workflow成功率が上がり、destructive誤操作が増えない。
- help例とnext_actionsのcommand/action vocabularyが一致する。

## Verification Design

- Focused tests: group coverage、example parseability、help-before-side-effect、debug classification。
- Cross-boundary tests: registry/help/completion/next actionから同じworkflow linkを検査する。
- Scale or fault tests: narrow terminal、unknown command/option、corrupt repoからrecovery help。
- Observability: user studyでtask completion、help views、誤command、recovery成功を記録する。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-024/047/079/085/099とCFR-082/091/094/131は個別help routingをfixed済み。本件はtask-level discoverability実験で非重複。
- Required predecessor: RVC-032のhierarchical registry、RVC-035のsafe action mode。Stage 1/2 gateを迂回しない。
- Can combine with: RVC-038のcross-view navigation。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- guided workflowからmutationを自動実行しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
