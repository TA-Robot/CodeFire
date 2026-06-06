# AGENTS.md

このリポジトリは、機械学習アルゴリズムを進化させる実験自動化AIエージェントの研究用です。

## 方針

- 主対象は「実験自動化AIエージェント」であり、個別アルゴリズム実装はエージェント評価のための対象物として扱う。
- SOTA主張は、再現可能なbenchmark、baseline、統計的比較、失敗ログが揃うまで書かない。
- 実験は小さく、再実行可能にする。
- 結果が悪い実験も `docs/experiments/` に記録する。
- 研究判断は `docs/development/history.md` に残す。

## 最低確認

```bash
PYTHONPATH=src python3 -m unittest discover -s tests -v
```

## CodeFire運用

- 要求は `docs/spec/` に `REQ-*` Atomとして書く。
- 設計は `docs/design/` に `DES-*` Atomとして書く。
- 実装は `src/evoagent/` に `CODE-*` Atomとして書く。
- テストは `tests/` に `TEST-*` Atomとして書く。
- Trace Link は `codefire.links.yaml` を正本にする。
- このリポジトリは CodeFire dogfooding 対象である。CodeFire の不具合、バグに近いUX、改善点を見つけたら `/workspace/project/docs/development/bug-backlog.md` に `CFB-*` として記録し、必要なら `/workspace/project/docs/development/todo-checklist.md` に修正タスクを追加する。
- algorithm-evolution 側の開発は、原則として `codefire scan -> extinguish -> verify -> commit` で履歴化する。
- 仕様先行で未実装Atomが多い期間は `commit_policy.require_trace_completeness: false` を許容する。ただし実装済み要求には Trace Link を追加していく。
- Codex をAI plannerとして使う変更は、free-form出力をそのまま採用せず、構造化データへparseしてから既存の scoring / scheduling に渡す。
