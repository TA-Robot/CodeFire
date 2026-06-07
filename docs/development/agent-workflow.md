# Agent Workflow

この文書は、CodeFire本体を改修するagent向けの作業運用を定義する。品質基準は `agent-code-quality.md` を参照する。

## 対象

- CLI本体: `codefire`
- Rust workspace: `crates/`
- テスト: `tests/`
- インストール/包装: `install.sh`, `pyproject.toml`, `setup.py`
- ドキュメント: `README.md`, `docs/`, `adr/`, `schemas/`
- 開発管理: `docs/development/`

## 差分・マージ品質

Gitと比べて現在のCodeFireのdiff/mergeはまだ薄い。v0.6ではRust再実装と合わせて、以下を意識する。

- Myers / patience / histogram-style diffを比較可能にする。
- rename/copy detectionではsimilarity scoreを出す。
- binary fileはpayload diffではなくhash/size/kind summaryを出す。
- Atom diffを出す。
- TraceGraph diffを出す。
- Fire impact diffを出す。
- policy impact diffを出す。
- merge dry-runで書き込み前に影響を提示する。
- semantic conflictは候補検出までに留め、自動解決しない。
- review-packはfile diff、Atom diff、Trace diff、verification、next actionsをまとめる。

## Automation-Friendly Interface

CodeFireはAI agent運用機能を持たない。agentの起動、編成、分業、プロンプト管理、marketplaceは対象外である。

一方で、外部自動化ツール、人間のwrapper、CI、editor integrationが安全にCodeFireを叩けるよう、v0.6では以下を重視する。

- stable `--json`
- stable exit code taxonomy
- `codefire context`
- machine-readable `next_actions`
- dry-run / operation plan
- batch operation
- idempotency key
- `--wait-lock` / `--lock-timeout`
- evidence capture
- read-only `codefire explain`

実装時は、human outputよりmachine-readable contractの安定性を重視する。

## Rust v0.6方針

v0.6ではRust実装をdefault CLIへ移行する計画である。

守ること:

- Python版はreference implementation / fallbackとして残す。
- Rust本流実装では `main.rs` を薄く保ち、command、remote、signature、verification、diff/merge、renderingを責務単位で分ける。
- module/file分割の判断基準は `module-boundaries.md` を正とする。
- Rust write pathの前にcanonical JSON / object ID golden testを作る。
- Python-created repoをRustが読めることを先に保証する。
- Rust-created objectをPython版が最低限 `doctor` / `show` できる互換性を保つ。
- object format extensionはversioned extensionとして追加し、既存fieldの意味を変えない。
- installer切替はmigration checkが揃ってから行う。

参照:

- `v0.6-rust-rewrite-plan.md`
- `v0.6-readiness.md`
- `todo-checklist.md`
- `bug-backlog.md`

## テスト方針

変更範囲に応じて、最小ではなく十分なテストを走らせる。

`codefire` 本体を触ったら:

```bash
python3 -m py_compile codefire tests/test_codefire_cli.py
python3 -m unittest discover -s tests -v
./demo.sh
```

installやpackagingを触ったら:

```bash
bash -n install.sh
./install.sh --prefix /usr/local
codefire --help
```

Rust workspaceを触ったら:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

docs-only変更ではテスト不要。ただしMarkdownの表、リンク、TODO/historyの整合性は確認する。

## CodeFire dogfooding運用

反復運用の全体像は `codefire-improvement-cycle.md` を正とする。

CodeFire自身の不具合、バグに近いUX、改善点を見つけたら:

- `bug-backlog.md` に `CFB-*` として記録する。
- 実装タスクに落とせる場合は `todo-checklist.md` に `CF-*` として追加する。
- 方針や履歴は `history.md` に残す。

開発対象がCodeFire管理repoの場合は、原則として:

```bash
codefire scan
codefire verify --details
codefire extinguish ...
codefire commit -m "..."
```

Git管理側のCodeFire本体では、docs/code/testsの変更を小さく分けてgit commitする。

## 禁止・注意

- ユーザーの未コミット変更を勝手に戻さない。
- `git reset --hard` や `git checkout --` で破壊的に戻さない。
- 依存追加を軽く扱わない。
- 大きなrefactorを小さなbug fixに混ぜない。
- object format互換性を壊す変更を無計画に入れない。
- semantic conflictの自動解決をv0.6に入れない。
- AI agent運用機能やmarketplace機能をCodeFire本体に入れない。
