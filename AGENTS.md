# AGENTS.md（CodeFire本体開発用）

この `AGENTS.md` は **CodeFire本体リポジトリ** を改修するための入口ガイドである。詳細ルールは `docs/development/` 配下に分割して管理する。

## まず読む

- `docs/development/milestones.md`: いま進めてよい唯一の作業。5段先までのボード。
- `docs/development/milestone-operating-model.md`: 次マイルストーンに必要なissueだけを進め、他は後段へ送る規則。
- `docs/development/ai/orchestration-protocol.md`: Codex指揮、Claude/Grok個別、30分ごとの監査エージェント。
- `docs/development/README.md`: 開発管理ハブ、現在状態、正本ドキュメント対応表。
- `docs/development/project-management.md`: 長期開発に耐えるproject管理モデル、issue/cycle/dirty tree運用。
- `docs/development/ai/README.md`: 外部AIで開発を加速するためのcontext pack、task packet、role routing、review、parallel lane、local tool運用。
- `docs/development/ai/provider-routing-policy.md`: Codexは指揮のみ。個別と監査は Claude Opus 5 / Grok 4.6。
- `docs/development/agent-code-quality.md`: AtCoder red レベルのコード品質、計算量、メモリ、アルゴリズム、不変条件。
- `docs/development/module-boundaries.md`: module/file分割、責務境界、`main.rs` 肥大化防止。
- `docs/development/agent-workflow.md`: 作業運用、Rust v0.6方針、diff/merge方針、テスト、禁止事項。
- `docs/development/codefire-improvement-cycle.md`: CodeFire実装、install、algorithm dogfooding、issue創出、次version計画を反復するcodefire改善サイクル。
- `docs/development/source-architecture.md`: crate/module責務、domain/store/workspace/commit transaction、永続化・security境界。
- `docs/development/command-source-trace.md`: public commandごとのparse/validate/plan/apply/render/write/test経路。
- `docs/development/source-data-model-catalog.md`: Rust domain型、object kind、persisted JSON、version/compatibility境界。
- `docs/development/change-playbook.md`: 変更種別ごとのreading route、recipe、invariant review、validation順序。
- `docs/development/strategy/README.md`: 採用済み原則、product戦略、active roadmap、release/format方針、decision register。
- `docs/development/strategy/release-and-format-roadmap.md`: 独立version軸、historical milestone、次release gate。
- `docs/development/todo-checklist.md`: 完了済み実装taskのhistorical ledger。新規戦略作業のactive roadmapには使わない。
- `docs/development/bug-backlog.md`: CodeFire自身のバグ、改善点、UX違和感。
- `docs/development/history.md`: 開発履歴。

## 対象

- CLI本体: `codefire`
- Rust workspace: `crates/`
- テスト: `tests/`
- インストール/包装: `install.sh`, `pyproject.toml`, `setup.py`
- ドキュメント: `README.md`, `docs/`, `adr/`, `schemas/`
- 開発管理: `docs/development/`
- 実験サブプロジェクト: `subprojects/`

## 最重要ルール

- 作業許可は `docs/development/milestones.md` の **M1 だけ**。M2–M5 と review の非ブロッキング指摘は実装しない。
- オーケストレータは M1 のクリティカルパスを切り、依存しない packet は空きスロットがある限りすぐ並列起動する。直列キューは計画として不合格。
- 指揮は Codex `gpt-5.6-sol`。個別実装・レビュー・30分監査は Claude Opus 5 または Grok 4.6。Codex に実装も監査もさせない。
- レビューは `block-m1` / `park` / `drop` のみ。`park` を今やる仕事にしない。
- テストは当該マイルストーンの exit が指名したコマンドだけを必須にする。カタログ全件を日次入場条件にしない。
- 常にコードは **AtCoder red レベルの最高レベルのコードを作成すること** を心がける。
- 計算オーダとメモリ利用オーダを軽く扱わない。
- `main.rs` や単一ファイルへ機能を詰め込まない。責務単位でmodule/fileを分割する。
- 新規機能で責務境界が増える場合は、実装前に `docs/development/module-boundaries.md` の分割判断を確認する。
- 主要command、module、型、永続化境界を変える場合は、実装と同じ変更でcanonical source docs 4文書の該当箇所を更新し、`python3 docs/development/tools/check_source_docs.py` を実行する。
- parse、validate、plan、apply、renderを混ぜない。
- object identity、sealed commit、branch head atomicity、remote mutation policyの不変条件を壊さない。
- AI agent運用機能、agent marketplace、prompt orchestrationはCodeFire本体へ入れない。
- `subprojects/` はCodeFire dogfooding用の対象プロジェクトとして扱い、CodeFire本体のCLI/Store/Core実装と責務を混ぜない。
- ユーザーの未コミット変更を勝手に戻さない。
- `git reset --hard` や `git checkout --` で破壊的に戻さない。

## 現行実装・release方針

Rust実装をdefault CLIとし、Python版はreference implementation / fallbackとして残す。

現時点ではpublic product release identityは未確定であり、旧v0.6–v1.0 plan名をrelease完成の根拠にしない。開発順序は `docs/development/strategy/modernization-roadmap.md`、release/format判定は `docs/development/strategy/release-and-format-roadmap.md` を正とする。TODOや履歴を更新する場合は、関連する `history.md` / `known-limitations.md` / `runbook.md` も合わせて整合させる。

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
