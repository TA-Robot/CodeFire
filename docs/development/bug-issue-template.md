# CodeFire Dogfooding Issue Template

CodeFireを実プロジェクトで使って見つけた不具合、UX不整合、自動化I/Oの弱さは、この形式で詳細化する。

## Detail File Template

```markdown
# CFB-000: <short title>

## Summary

| Field | Value |
|---|---|
| Status | open | investigating | fixed | wontfix |
| Severity | critical | high | medium | low |
| Type | bug | compatibility | automation-json | ux | performance | storage | docs |
| Area | <command/module/workflow> |
| Observed while | <dogfooding context> |

## Observation

<実際に観測した挙動。コマンド、出力、JSON shape、状態遷移などを書く。>

## Reproduction

```bash
<command>
```

## Why It Matters

<AI agent運用、人間のレビュー、整合性、CI、性能、記憶領域への影響を書く。>

## Workaround

<現状の回避策。内部JSON直読み、別コマンド併用、手作業など。>

## Candidate Fix

<CodeFire本体として望ましい修正。AI agent運用機能そのものは入れず、toolとしてのI/Oや整合性を改善する。>

## Acceptance Criteria

- <完了条件>

## Evidence

- <観測コマンドやdogfooding対象>
```

## Rules

- `Observed while` には必ず実プロジェクト名または操作文脈を書く。
- `Reproduction` は可能な限り現在のCLIで再実行できるコマンドにする。
- `Why It Matters` はAI agentがCodeFireをtoolとして使う場合の影響を明示する。
- `Candidate Fix` はCodeFire本体の責務に閉じる。AI agent運用機能やmarketplaceは入れない。
- 一覧は `docs/development/bug-backlog.md`、詳細は `docs/development/bug-issues/cfb-000.md` の1 issue 1 fileで管理する。
