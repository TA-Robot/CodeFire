# AGENTS.md（CodeFire本体開発用）

この `AGENTS.md` は **CodeFire本体リポジトリ** を改修するための入口ガイドである。詳細ルールは `docs/development/` 配下に分割して管理する。

## まず読む

- `docs/development/agent-code-quality.md`: AtCoder red レベルのコード品質、計算量、メモリ、アルゴリズム、不変条件。
- `docs/development/agent-workflow.md`: 作業運用、Rust v0.6方針、diff/merge方針、テスト、禁止事項。
- `docs/development/v0.6-readiness.md`: v0.6を完成と判断するための残ゲート。
- `docs/development/todo-checklist.md`: 実装タスクと依存関係。
- `docs/development/bug-backlog.md`: CodeFire自身のバグ、改善点、UX違和感。
- `docs/development/history.md`: 開発履歴。

## 対象

- CLI本体: `codefire`
- Rust workspace: `crates/`
- テスト: `tests/`
- インストール/包装: `install.sh`, `pyproject.toml`, `setup.py`
- ドキュメント: `README.md`, `docs/`, `adr/`, `schemas/`
- 開発管理: `docs/development/`

## 最重要ルール

- 常にコードは **AtCoder red レベルの最高レベルのコードを作成すること** を心がける。
- 計算オーダとメモリ利用オーダを軽く扱わない。
- `main.rs` や単一ファイルへ機能を詰め込まない。責務単位でmodule/fileを分割する。
- parse、validate、plan、apply、renderを混ぜない。
- object identity、sealed commit、branch head atomicity、remote mutation policyの不変条件を壊さない。
- AI agent運用機能、agent marketplace、prompt orchestrationはCodeFire本体へ入れない。
- ユーザーの未コミット変更を勝手に戻さない。
- `git reset --hard` や `git checkout --` で破壊的に戻さない。

## v0.6開発方針

v0.6ではRust実装をdefault CLIへ移行する。Python版はreference implementation / fallbackとして残す。

現時点の完成判定は `docs/development/v0.6-readiness.md` を正とする。TODOや履歴を更新する場合は、関連する `todo-checklist.md` / `history.md` / `known-limitations.md` / `runbook.md` も合わせて整合させる。

## 確認コマンド

`codefire` Python本体を触ったら:

```bash
python3 -m py_compile codefire tests/test_codefire_cli.py
python3 -m unittest discover -s tests -v
./demo.sh
```

Rust workspaceを触ったら:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

installやpackagingを触ったら:

```bash
bash -n install.sh
./install.sh --prefix /usr/local
codefire --help
```

docs-only変更ではテスト不要。ただしMarkdownの表、リンク、TODO/historyの整合性は確認する。
