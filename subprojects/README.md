# CodeFire Subprojects

このディレクトリは CodeFire を実プロジェクトで dogfooding するためのサブプロジェクト置き場である。

## algorithm-evolution-agent-lab

`algorithm-evolution-agent-lab` は、機械学習アルゴリズム探索の実験自動化AIエージェントを題材にした検証プロジェクトである。

CodeFire本体との関係は次の通り。

- CodeFire本体の実装は `crates/`、仕様と開発管理は `docs/` に置く。
- サブプロジェクトのソース、仕様、テスト、CodeFire設定は `subprojects/algorithm-evolution-agent-lab/` に置く。
- `.codefire/` と `.codefire-open` は実行時メタデータなのでGit管理対象にしない。
- サブプロジェクトで見つかったCodeFire本体のバグや改善点は `docs/development/bug-backlog.md` または `docs/development/code-review-issues/` に戻す。
