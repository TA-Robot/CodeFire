# Code Fire Documentation v0.2

Code Fireは、AI時代の仕様駆動開発を前提にした、整合性ファーストのバージョン管理システムです。
Git互換ツールではありません。Gitの上位レイヤでもありません。

このドキュメントパックは、現時点の思想、仕様、内部設計、MVP計画、スキーマ例、サンプルプロジェクトをまとめたものです。

## このworkspaceでの配置メモ

この作業コピーでは、devcontainer基盤リポジトリのルールに合わせて `examples/` は展開していません。
開発管理用の文書は `docs/development/` に追加しています。
CLI実装は `project/codefire` と `project/crates/` にあります。

- `docs/development/todo-checklist.md` - 実装TODOチェックリスト
- `docs/development/history.md` - 開発履歴と判断ログ
- `docs/development/completion-plan.md` - MVP完成計画
- `docs/development/known-limitations.md` - 既知制約、Rust v0.6差分、将来課題
- `docs/development/bug-backlog.md` - 実運用で見つかった不具合/修正候補
- `docs/development/v0.6-readiness.md` - Rust v0.6完成判定と残ゲート
- `docs/development/source-code-map.md` - Rust実装のsource map
- `docs/development/source-code-implementation-atlas.md` - Rust実装のcrate/module/関数帯/主要型/永続化先/command ownershipを一枚で読む実装アトラス
- `docs/development/source-code-blueprint.md` - 要求、設計、実装、永続化、テストの対応表
- `docs/development/source-code-mental-model.md` - ソースを開く前に実装形状を想像するための詳細地図
- `docs/development/command-source-trace.md` - CLI commandからsource file/functionへの実装経路

## 現在の実装

このリポジトリには2つの実装系統があります。

- `./target/release/codefire-rs` / `./target/debug/codefire-rs`: Rust v0.6 CLI。`install.sh` の既定で `codefire` として配置されます。
- `./codefire`: Python v0.2 reference implementation。`install.sh` では fallback として `codefire-py` に配置されます。

v0.6の完成判定と検証項目は `docs/development/v0.6-readiness.md` を参照してください。

```bash
cargo build --workspace
./target/debug/codefire-rs --help
python3 -m unittest discover -s tests -v
./demo.sh
```

Rust v0.6 CLIの主要コマンド:

```text
init
branch list
open
clone
status
scan
fire
extinguish
verify
commit
merge
show
diff
upload
list
request-merge
request-list
request-review
request-apply
review-pack
patch export
patch import
storage report
evidence add
explain
context
migrate check
migrate dry-run
doctor
serve
completion
```

インストール例:

```bash
./install.sh --prefix "$HOME/.local"
codefire --help
codefire-py --help
```

`install.sh` はrelease Rust binaryをbuildして `codefire` として配置し、Python reference implementationを `codefire-py` として配置します。install前後にRust binaryの `--help` / `--version` smokeを行い、completionは事前生成してから配置します。事前確認だけ行う場合:

```bash
./install.sh --prefix "$HOME/.local" --completion bash --dry-run
```

prebuilt binaryを使う場合:

```bash
./install.sh --prefix "$HOME/.local" --binary ./target/release/codefire-rs
```

Python packageとしてreference implementationだけをインストールする場合:

```bash
python3 -m pip install .
codefire --help
```

shell completionを生成する場合:

```bash
codefire completion bash > ~/.local/share/bash-completion/completions/codefire
codefire completion zsh > ~/.local/share/zsh/site-functions/_codefire
```

remote token hashを生成する場合:

```bash
codefire-py token-hash my-token
codefire-py token-hash my-token --salt my-salt
```

一気通貫デモ:

```bash
./demo.sh
./demo.sh /tmp/codefire-demo
```

Python v0.2はPython標準ライブラリのみを使う単一ファイルCLIです。Rust v0.6は `Cargo.toml` / `crates/codefire-*` のworkspaceとして実装されています。

現時点の主要な制約と未完了領域:

- YAML parser はMVP用の限定パーサです。ただし `codefire.yaml` / `codefire.links.yaml` / `codefire.policy.yaml` の主要な必須項目不足は明示エラーにします。
- Markdown Atomは requirements / designs / ADR / ops runbook を抽出できます。OpenAPI JSON/YAML operation、SQL table/column/index/view/trigger/function/procedure/sequence/type、Python/JavaScript/TypeScript/Go/Java/C#/Rust/Kotlin/PHP/Ruby/Swift/C/C++ code symbol も `API-*` / `DB-*` / `CODE-*` Atomとして抽出できます。
- object IDはcontent-addressed payloadに基づくMVP実装です。表示IDと自己参照を含むcommit payloadの厳密仕様は整理が必要です。
- AI連携、GUI、semantic mergeはMVP対象外です。追加言語のAtom抽出は、標準ライブラリで実装した限定パーサの範囲で対応します。
- Python v0.2 remote server はローカルファイル-backed実装に加えて、`codefire-py serve` によるHTTP/HTTPS transportを持ちます。file-backedは `cf:///tmp/server/org/app/main`、HTTPは `cf+http://127.0.0.1:8080/org/app/main`、HTTPSは `cf+https://127.0.0.1:8443/org/app/main` のようなURLを使います。
- Rust v0.6 remoteはfile-backed、`cf+http://`、`cf+https://` を実装済みです。
- HTTP transportは現時点で `upload` / `list` / `clone` / `show` / `diff` / `request-merge` / `request-list` / `request-review` / `request-apply` / `doctor` / `gc` に対応します。
- upload時にsealed commitのobject hash、parents/roots/certificate構造、parent履歴、root object type、root payload shape、certificateとverification rootの一致を検証します。
- remote projectの `server_policy.json` により、upload時のserver-side verification commandを実行できます。server-side verificationは `cwd`、`env`、`timeout_seconds` を指定でき、`cwd` はremote project内に制限されます。
- `codefire.policy.yaml` の `commit_policy` で commit blocker とする検査を制御できます。
- `codefire.policy.yaml` の `verification.required` は複数commandを順に実行できます。各commandは `timeout_ms`、`max_output_bytes`、`inherit_env`、`env`、`allow_failure` を持てます。
- `codefire.policy.yaml` の `required_links` で Atom kind ごとの必須Trace Linkを設定できます。
- merge requestにはreview recordを追加できます。`request-list` / `request-review` / `request-apply` はMR内のsealed commit参照を検証します。
- approved merge requestはfast-forward条件を満たす場合にserver上でapplyできます。
- `show` / `diff` は対象commitishをsealed commitとして検証してから表示します。
- `branch list` / remote `list` はbranch headをsealed commitとして検証してから表示します。
- `status` / `scan` / `fire` / `extinguish` / `verify` / `commit` はopen registryのbase commitとbranch headをsealed commitとして検証してから処理します。
- `doctor` / `doctor cf://.../org/app` / `doctor cf+http://.../org/app` でobject recordのhash/id/filename不一致、構造化されたmissing object reference、branch/MRが指すsealed commit参照の不正を診断できます。
- `server_policy.json` の `permissions` で upload / request / review / apply / gc を制御できます。
- `server_policy.json` の `branch_protection` でbranch patternごとに upload / apply を制御できます。
- `server_policy.json` の `auth.required` と `auth.tokens` で remote のトークン認証を有効化できます。CLIは `--token` または `CODEFIRE_TOKEN` を読みます。tokenは平文互換に加えて、Python fallbackの `codefire-py token-hash` が生成する `sha256:<hex>` またはsalt付きhash objectで保存できます。
- Python v0.2とRust v0.6では `CODEFIRE_SIGNING_KEY=... codefire commit --signer alice --key-id alice-2026-06` / `codefire-py commit ...` でcommitにHMAC署名を付与できます。remote側は `server_policy.json` の `commit_signatures.required` と `commit_signatures.keys` で upload / apply されるbranch headの署名を要求できます。key objectの `not_before` / `not_after` / `status` / `signers` により鍵世代の並行運用と失効を扱えます。
- Python v0.2とRust v0.6では `CODEFIRE_REQUEST_SIGNING_KEY=...` と `--request-key-id` でremote mutating operationにHMAC request署名を付与できます。remote側は `request_signatures.required` / `request_signatures.keys` / `max_skew_seconds` / `nonce_ttl_seconds` で upload / request / review / apply の署名とnonce replayを検証できます。Python v0.2のremote GC request署名も同じpolicyで扱います。
- Python v0.2とRust v0.6では `codefire serve --tls-cert CERT --tls-key KEY` / `codefire-py serve --tls-cert CERT --tls-key KEY` でHTTPS transportを有効化できます。自署名証明書をローカル検証する場合のみ `CODEFIRE_TLS_INSECURE=1` を使えます。
- Python fallbackの `codefire-py gc cf://.../org/app` でremote object GCを実行できます。GC前にremoteのobject graphとsealed commit参照を検証し、`gc.retention_seconds` / `gc.retention_generations` により新しい到達不能objectを保護し、実行結果は `audit/gc.jsonl` に記録します。

詳細は `docs/development/known-limitations.md` を参照してください。

## 読み方

最初に以下を読むと全体像がつかめます。

1. `docs/00_philosophy.md` - 思想背景
2. `docs/01_product_definition.md` - 要求定義
3. `docs/02_core_concepts.md` - 中核概念
4. `docs/03_workflow_spec.md` - 開発フロー仕様
5. `docs/11_internal_architecture.md` - 内部アーキテクチャ設計
6. `docs/12_mvp_implementation_plan.md` - MVP実装計画

実装を読む場合は、上記に続けて `docs/development/source-code-map.md`、`docs/development/source-code-implementation-atlas.md`、`docs/development/source-code-blueprint.md`、`docs/development/source-code-mental-model.md`、`docs/development/command-source-trace.md` を読むと、要求、設計、command、module、主要関数、データ流、persistent state、テスト配置の対応を把握できます。特に `source-code-implementation-atlas.md` は、sourceを開く前にRust workspaceの実装配置、`main.rs` の関数帯、command ownership、主要型、保存先、変更recipeを復元するための入口です。

単一ファイルで読みたい場合は、`CodeFire_Design_Document_v0.2.md` または `CodeFire_Design_Document_v0.2.docx` を参照してください。

## このパックに含まれるもの

```text
README.md
CodeFire_Design_Document_v0.2.md
CodeFire_Design_Document_v0.2.docx

docs/
  00_philosophy.md
  01_product_definition.md
  02_core_concepts.md
  03_workflow_spec.md
  04_cli_spec.md
  05_consistency_model.md
  06_fire_extinguish.md
  07_local_repository_model.md
  08_object_store.md
  09_index_trace_policy.md
  10_merge_remote_server.md
  11_internal_architecture.md
  12_mvp_implementation_plan.md
  13_risks_and_non_goals.md

adr/
  ADR-001-no-git-compatibility.md
  ADR-002-no-checkpoint-stash.md
  ADR-003-open-directory-branch.md
  ADR-004-consistent-only-commit.md
  ADR-005-active-state-metadata-only.md

schemas/
  codefire.yaml
  codefire.links.yaml
  codefire.policy.yaml
  object_commit.example.json
  object_fire.example.json
  object_resolution.example.json
  open_directory.example.yaml
  repository_layout.txt

examples/minimal-auth/
  （この作業コピーでは未展開）

diagrams/
  overview.mmd
  branch_state_machine.mmd
  commit_flow.mmd
  object_model.mmd
  merge_flow.mmd

docs/development/
  todo-checklist.md
  history.md
  completion-plan.md
  known-limitations.md
  source-code-map.md
  source-code-implementation-atlas.md
  source-code-blueprint.md
  source-code-mental-model.md
  command-source-trace.md
  bug-backlog.md
```

## v0.2での確定方針

- Gitとは共存しない。
- Git互換を目指さない。
- commitできるのは整合済み状態のみ。
- checkpoint、stash、WIP commit、partial commit、staging areaは存在しない。
- checkout、pull、fetch、rebase、force upload、remote tracking branchは存在しない。
- branchはopenすると実ディレクトリになる。
- 同時に複数branchをopenできる。
- `/` を含むbranch名は内部ファイル名でURLエンコードして保存する。
- remote URLで `/` を含むbranch名を指す場合は `feature%2Fsession` のようにURLエンコードする。
- closeすると実ディレクトリは削除される。
- cloneはsealed branch headから新branchを作る操作である。
- open / clone / merge は対象branch headがsealed commitとして妥当な場合だけ進む。
- open directory上の status / scan / fire / extinguish / verify / commit はregistry base commitとbranch headがsealed commitとして妥当な場合だけ進む。
- show / diff は対象commitishがsealed commitとして妥当な場合だけ表示する。
- branch list / remote list はbranch headがsealed commitとして妥当な場合だけ表示する。
- clone / merge は source branch が open-burning の場合は拒否する。
- mergeはtarget branchのopen directoryをburningにし、整合回復を要求する操作である。
- serverにはsealed commitだけが流通する。
- fireは未解消の確認責任である。
- extinguishには理由または証跡が必要である。`no-change-required` のrationale必須可否は `extinguish_policy` で制御できる。
- Trace Linkを一次情報とし、トレーサビリティ表は生成物とする。
