# RVC-031: command lifecycleをtyped stageとして揃える

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | architecture / correctness / testing |
| Review kind | refine |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `crates/codefire-cli/src/main.rs` / command modules |
| Promotion target | ADR |

## Executive Judgment

focused moduleが増え、help-before-mutation、operation plan、共通envelopeが導入された方向は正しい。一方、top-level runとmain.rs内workflowにはparse、context resolution、lock、plan、apply、renderの組合せがcommandごとに残り、共通lifecycleは文書上の約束に留まる。巨大frameworkへ一括移行せず、mutation riskの高いcommandからtyped stageと共通adapterへ段階収束させるべきである。

## Scope and Review Question

- In scope: argv routing、typed parse、validate、plan、apply、render、help/error path
- Out of scope: parser library導入そのもの、domain algorithm、public command追加
- Question: commandごとの例外を保ちながら、validation抜け・dry-run分岐・JSON failure迂回を共通構造で防げるか

## Evidence

- `crates/codefire-cli/src/main.rs` — `run` / `local_workflow_handler`: 一部local workflowはhandler tableへ移ったが、残る多数commandは大きなmatch armでparse/execution/renderを個別編成する。
- `crates/codefire-cli/src/main.rs` — `run_commit_command` / `run_commit`: dispatch helper、parser、transaction orchestration、rendererがmain.rs内の離れたsymbolへ分散する。
- `docs/development/command-source-trace.md` — Execution Contract: help→typed parse→context→validation/lock→plan→apply→renderという標準経路をcanonicalに定める。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs` — `command_help_routes_before_mutating_parsers` は重要な入口性質を固定するが、全commandのstage通過を同じcontract testで検査しない。

## Strengths to Preserve

- command固有domain logicをcontext、doctor、evidence、view等へ分けたmodule ownership。
- helpをmutation/parser side effectより先に返し、dry-runでplanを返す既存contract。

## Gap / Risk

### Observed gap

各commandがlifecycleを手動で組み、同じCliError/envelope helperを使うか、lockをどこで取るか、dry-runで何をskipするかがcall siteごとに決まる。

### Failure or stagnation mode

新commandやoption追加でJSON failure、metrics、help、lock、idempotencyの一つが漏れる。main.rsの変更衝突が増え、個別issueを直しても別commandで同じclassが再発する。

### Affected invariants

- validate-before-apply
- dry-run no-write
- automation envelope
- module ownership

## Improvement Direction

CommandLifecycleを巨大traitとして強制せず、Parsed<T>、Validated<T>、Plan<T>、Applied<T>の狭い内部型と、render adapterの共通contractを用意する。各focused moduleはparse/validate/plan/applyを所有し、main.rsはhelp/capability routingとadapter invocationへ限定する。read-only、plan-only、mutationの3profileを先に定義する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: profile＋typed stagesをcommand単位で段階導入 | invariantを明示し移行差分を小さくできる | 一時的に旧新adapterが併存する | adopt |
| B: macro/汎用CLI frameworkへ一括rewrite | 表面上統一しやすい | domain例外を隠し大差分になる | reject |

## Recommended Sequence

1. **Characterize**: 全commandをread-only/plan/mutation profileと現在stage ownerへ分類する。
2. **Separate**: lifecycle result型とrender adapterだけを小moduleへ抽出する。
3. **Implement**: 一つの単純read-only commandと一つのmutation commandでvertical sliceを作る。
4. **Prove**: help、parse failure、dry-run、apply failure、text/JSONをprofile contract test化する。
5. **Promote**: stage APIとmain.rsの責務上限をADR/module-boundariesへ昇格する。

## Acceptance Signals

- 移行commandではmain.rsにdomain validation/write/render分岐が残らない。
- profile contract testがhelp-before-side-effect、no-write dry-run、JSON failureを全件検査する。
- command固有diagnosticとhuman UXを失わず、変更行のowner moduleが明確になる。

## Verification Design

- Focused tests: 各stageの入力型、invalid transition、renderer純粋性。
- Cross-boundary tests: registry→help→parse→plan→apply→envelopeのtable-driven suite。
- Scale or fault tests: apply failpointとconcurrent lock pathをmigration前後で比較する。
- Observability: stage名、duration/work counter、failure stageをmetrics/diagnosticへ付与する。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-009、CFR-001/002/008はmodule抽出・shared parserをfixed済み。本件はそれを撤回せず、残るorchestration contractを次段へ進める候補。CFR-039のtest分割方針も前提にする。
- Required predecessor: 凍結中Stage 1 correctness batch。特にRVC-026のlock ownership。
- Can combine with: RVC-032、RVC-036、RVC-037、RVC-051/054のsource・test decomposition。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- framework導入を目的化せず、実証できない抽象化を増やさない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
