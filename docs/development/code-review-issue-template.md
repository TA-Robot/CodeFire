# Code Review Issue Template

CodeFire本体のコードレビューで見つけた改善候補は、この形式で記録する。バグ確定ではないものも、再現条件・影響・完了条件まで明示してからtriageする。

## Detail File Template

```markdown
### CFR-000: <短いタイトル>

## Summary

| Field | Value |
|---|---|
| Status | open | investigating | fixed | wontfix |
| Severity | critical | high | medium | low |
| Type | bug | correctness-risk | performance | security | maintainability | test-gap | compatibility | ux |
| Area | <crate/module/command> |
| Source | `<file>:<line>` または `<file>:<line>-<line>` |

## Problem

<コード上の根拠。観測ログだけでなく、読み取れる実装事実を書く。>

## Why It Matters

<ユーザー、データ整合性、AI tool運用、保守性、速度、メモリへの影響を書く。>

## Failure Mode / Review Hypothesis

<どの条件で破綻するか、またはどのような保守リスクになるかを具体化する。>

## Code Evidence

- Primary source: `<file>:<line>`
- Observed implementation fact: <実装から読み取れる事実>

## Recommended Fix

<望ましい修正方針。実装単位に落とせる粒度で書く。>

## Implementation Notes

- Priority: <いつ潰すべきか>
- First step: <最初に追加するテスト、または最初の小さいrefactor>
- Boundary: <CodeFire本体の責務外に出ないための注意>
- Related issues: <関連CFR/CFB>

## Acceptance Criteria

- <完了条件1>
- <完了条件2>

## Test Plan

- <追加・更新するテスト>

## Triage Checklist

- [ ] 再現fixtureまたは現状固定テストを追加した
- [ ] 修正方針が既存ADR/仕様と矛盾しないことを確認した
- [ ] `docs/development/history.md` または関連仕様を更新した
- [ ] 必要な確認コマンドを実行した
```

## Review Rules

- `Severity` はデータ破壊・整合性破壊・security境界を最優先に上げる。
- `Source` は最低1箇所のコード根拠を必ず入れる。
- `Evidence` は「気がする」ではなく、現在の実装から読み取れる事実を書く。
- `Recommendation` はAI agent運用機能やmarketplaceをCodeFire本体へ入れない制約を守る。
- `Acceptance criteria` は実装完了を機械的に確認できる内容にする。
- indexは `docs/development/code-review-issues-2026-06-06.md`、詳細は `docs/development/code-review-issues/<id>.md` の1 issue 1 fileで管理する。
- 既存の `docs/development/bug-backlog.md` はdogfooding由来の不具合・UX issue、コードレビュー由来のまとまった棚卸しはCFR系列で管理する。
