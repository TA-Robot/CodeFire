# CodeFire Development History

このファイルは、CodeFire 実装へ向けた作業履歴、判断、未解決事項を残すためのログです。

## 記録ルール

- 新しい作業単位を始めたら `Work Log` に追記する。
- 設計判断は、恒久的なら `adr/` に昇格する。暫定判断はこのファイルに残す。
- 完了したタスクは `docs/development/todo-checklist.md` の `Status` と `Evidence` も更新する。
- 仕様差分が出た場合は、該当する `docs/*.md` とこの履歴を同時に更新する。

## Work Log

| Date | Area | Summary | Artifacts | Notes |
|---|---|---|---|---|
| 2026-06-03 | Bootstrap | `codefire_docs_v0.2.zip` を `project/` 直下に展開した | `README.md`, `docs/`, `adr/`, `schemas/`, `diagrams/`, design document | この基盤リポジトリのルールに合わせ、archive内の `examples/` は展開対象から除外した |
| 2026-06-03 | Planning | 開発管理用TODO、履歴、完成計画を追加した | `docs/development/todo-checklist.md`, `docs/development/history.md`, `docs/development/completion-plan.md` | 実装は未開始。MVP計画を実行可能なタスクへ分解した |
| 2026-06-03 | Implementation | Python標準ライブラリのみでCodeFire MVP CLIを追加した | `codefire`, `tests/test_codefire_cli.py` | init/open/scan/fire/extinguish/verify/commit/clone/merge/doctor のE2Eが通る |
| 2026-06-03 | Merge | common ancestor探索とfile-level 3-way mergeを追加した | `codefire`, `tests/test_codefire_cli.py` | merge conflict は conflict marker を書き、verify/commit blocker になる |
| 2026-06-03 | Hardening | close/discard、obsolete fire、manual fire、stale resolution のE2Eを追加した | `tests/test_codefire_cli.py` | 9 tests pass |
| 2026-06-03 | Hardening | repo/branch/object-store lock、duplicate Atom ID E2E、既知制約文書を追加した | `codefire`, `tests/test_codefire_cli.py`, `docs/development/known-limitations.md` | 11 tests pass |
| 2026-06-03 | CLI | local `show`, local `diff`, `discard` commandを追加した | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/development/known-limitations.md` | 13 tests pass |
| 2026-06-03 | Remote | file-backed remote server、upload/list/remote clone/request-merge/request-list/remote show/remote diffを追加した | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/development/known-limitations.md` | 15 tests pass |
| 2026-06-03 | Remote | repo外からのremote show/diffとmerge request stale検出を検証した | `codefire`, `tests/test_codefire_cli.py` | 15 tests pass |
| 2026-06-03 | Decisions | object ID自己参照なし、限定YAML subset方針をADR化した | `adr/ADR-006-object-id-no-self-reference.md`, `adr/ADR-007-limited-config-yaml-subset.md` | 未決事項をMVP制約として確定 |
| 2026-06-03 | Remote | upload時のsealed commit validationとmerge request review recordを追加した | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, `docs/development/known-limitations.md` | 16 tests pass |
| 2026-06-03 | Remote | file-backed remote projectのserver-side verification commandを追加した | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, `docs/development/known-limitations.md` | 17 tests pass |
| 2026-06-03 | Remote | approved merge requestのserver-side fast-forward applyを追加した | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, `docs/development/known-limitations.md` | 17 tests pass |
| 2026-06-03 | Packaging | remote doctor、install.sh、pyproject metadataを追加した | `codefire`, `install.sh`, `pyproject.toml`, `tests/test_codefire_cli.py`, docs | 18 tests pass |
| 2026-06-03 | Remote | file-backed remote permission policyとremote object GCを追加した | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, `docs/development/known-limitations.md` | 20 tests pass |
| 2026-06-03 | Remote | file-backed remote token authenticationとGC retention policyを追加した | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, `docs/development/known-limitations.md` | 22 tests pass |
| 2026-06-03 | Release | local/remoteを通す一気通貫デモスクリプトを追加した | `demo.sh`, `README.md`, `docs/runbook.md`, `docs/development/todo-checklist.md` | `bash -n demo.sh`; `./demo.sh` |
| 2026-06-03 | Verify | `codefire.policy.yaml` の複数verification command実行に対応した | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, docs | 23 tests pass |
| 2026-06-03 | Trace | `codefire.policy.yaml` のcustom required_linksに対応した | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, docs | 24 tests pass |
| 2026-06-03 | Verify | `commit_policy` booleanをverify/commit/server validationに反映した | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, docs | 25 tests pass |
| 2026-06-03 | Hardening | `doctor` がlocal/remoteのmissing object referenceを検出するようにした | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, docs | 26 tests pass |
| 2026-06-03 | Extinguish | 全resolutionでrationaleまたはevidenceを必須にした | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, docs | 27 tests pass |
| 2026-06-03 | Hardening | object recordのhash/id/filename整合性検証を共通化しdoctorで検出するようにした | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, docs | 28 tests pass |
| 2026-06-03 | Branch | open-burning source branchからのclone/mergeを拒否するようにした | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, docs | 29 tests pass |
| 2026-06-03 | Branch | slash入りbranch名をURLエンコードした内部ファイル名で保存し、open directoryからregistryを辿れるようにした | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, docs | 30 tests pass |
| 2026-06-03 | Extinguish | `extinguish_policy.no-change-required.requires_rationale` を実装した | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, docs | 31 tests pass |
| 2026-06-03 | Remote | slash入りremote branch名をURLエンコードしたURLで扱い、MR applyまで通せるようにした | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, docs | 32 tests pass |
| 2026-06-03 | Object store | object graph traversalを型別参照抽出にし、通常文字列の `CF-*` 誤検出を防いだ | `codefire`, `tests/test_codefire_cli.py`, `README.md`, `docs/runbook.md`, docs | 33 tests pass |
| 2026-06-03 | Config | 限定YAML subsetで必須項目不足を明示エラーにする診断を追加した | `codefire`, `tests/test_codefire_cli.py`, docs | 37 tests pass; `./demo.sh` |
| 2026-06-03 | Branch | local/remote branch lock名をURLエンコードに統一し、slash名と二重underscore名の衝突を防いだ | `codefire`, `tests/test_codefire_cli.py`, docs | 39 tests pass; `./demo.sh` |
| 2026-06-03 | Doctor | local branch headとremote branch/MR参照がsealed commitとして妥当か診断するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 41 tests pass; `./demo.sh` |
| 2026-06-03 | Validation | sealed commit rootが期待するobject typeを指すことをupload/doctorで検証するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 42 tests pass; `./demo.sh` |
| 2026-06-03 | Validation | sealed commitのparents/roots/certificate構造を検証し、壊れたpayloadでもdoctorが診断できるようobject reference抽出を堅牢化した | `codefire`, `tests/test_codefire_cli.py`, docs | 43 tests pass; `./demo.sh` |
| 2026-06-03 | Validation | sealed commit validationをparent履歴へ再帰適用し、壊れた祖先commitをupload/doctorで検出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 44 tests pass; `./demo.sh` |
| 2026-06-03 | Branch | local open/clone/merge前にbranch headのsealed commit妥当性を検証し、壊れた履歴を通常操作へ広げないようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 46 tests pass; `./demo.sh` |
| 2026-06-03 | CLI | local/remote show/diff前にcommitishのsealed commit妥当性を検証し、壊れた履歴を表示しないようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 48 tests pass; `./demo.sh` |
| 2026-06-03 | Remote | request-list/review/apply前にMR内のsealed commit参照を検証し、壊れたMRを表示・承認しないようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 49 tests pass; `./demo.sh` |
| 2026-06-03 | CLI | local branch listとremote list前にbranch headのsealed commit妥当性を検証し、壊れた履歴を一覧表示しないようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 51 tests pass; `./demo.sh` |
| 2026-06-03 | GC | remote GC前にobject hash、missing reference、sealed commit参照を検証し、異常remoteでは削除しないようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 53 tests pass; `./demo.sh` |
| 2026-06-03 | Open directory | open registryのbase commitとbranch headをopen directory command入口でsealed commit検証するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 55 tests pass; `./demo.sh` |
| 2026-06-03 | GC | remote GCのdry-run/削除結果を `audit/gc.jsonl` にJSONL監査ログとして記録するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 56 tests pass; `./demo.sh` |
| 2026-06-03 | Remote | `server_policy.json` の `branch_protection` でbranch別にupload/apply actorを制御できるようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 57 tests pass; `./demo.sh` |
| 2026-06-04 | GC | remote branch head更新ごとに世代を進め、`gc.retention_generations` で直近世代の到達不能objectを保護するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 58 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | `codefire.yaml` の `adrs` / `ops` セクションを読み、Markdown ADR/ops Atomを抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 59 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | `codefire.yaml` の `apis` セクションを読み、OpenAPI JSON operationを `API-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 60 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | `codefire.yaml` の `db` セクションを読み、SQL `CREATE TABLE` を `DB-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 61 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | JavaScript/TypeScriptのclass/function/arrow functionを `CODE-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 62 tests pass; `./demo.sh` |
| 2026-06-04 | Packaging | `codefire completion bash|zsh` と `install.sh --completion` でshell completionを生成・配置できるようにした | `codefire`, `install.sh`, `tests/test_codefire_cli.py`, docs | 63 tests pass; `./demo.sh` |
| 2026-06-04 | Packaging | `pyproject.toml` / `setup.py` をsetuptools script installに切り替え、`pip install .` で `codefire` を配置できるようにした | `pyproject.toml`, `setup.py`, `tests/test_codefire_cli.py`, docs | 64 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | OpenAPI YAMLの `paths` 配下operationを `API-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 65 tests pass; `./demo.sh` |
| 2026-06-04 | Remote | `codefire serve` と `cf+http://` URLを追加し、HTTP経由のremote upload/list/cloneを実装した | `codefire`, `tests/test_codefire_cli.py`, docs | 66 tests pass; `./demo.sh` |
| 2026-06-04 | Remote | HTTP経由のmerge request create/list/review/applyを実装し、server側でsealed commit参照とfast-forward applyを検証するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 66 tests pass; `./demo.sh` |
| 2026-06-04 | Remote | HTTP remote branch bundleを一時object storeへ展開し、`show` / `diff` をsealed commit validation付きで実行できるようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 66 tests pass; `./demo.sh` |
| 2026-06-04 | Remote | HTTP remote `doctor` / `gc` を追加し、診断結果とGC auditをHTTP越しに確認できるようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 66 tests pass; `./demo.sh` |
| 2026-06-04 | Remote | `codefire serve --tls-cert --tls-key` と `cf+https://` URLを追加し、HTTPS経由のremote upload/list/show/cloneを検証した | `codefire`, `tests/test_codefire_cli.py`, docs | 67 tests pass; `./demo.sh` |
| 2026-06-04 | Remote | server-side verificationに `cwd` / `env` / `timeout_seconds` を追加し、cwd脱出、親環境漏れ、無期限実行を防ぐようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 68 tests pass; `./demo.sh` |
| 2026-06-04 | Remote | `server_policy.json` のtokenを `sha256:<hex>` またはsalt付きhash objectで保存できるようにし、`codefire token-hash` を追加した | `codefire`, `tests/test_codefire_cli.py`, docs | 69 tests pass; `./demo.sh` |
| 2026-06-04 | Remote | commit HMAC署名を追加し、remote policyでupload/applyされるbranch headの署名を必須化できるようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 72 tests pass; `./demo.sh` |
| 2026-06-04 | Remote | commit署名に `--key-id` とkey object policyを追加し、key rotation / revoked key / history検査を実装した | `codefire`, `tests/test_codefire_cli.py`, docs | 73 tests pass; `./demo.sh` |
| 2026-06-04 | Remote | remote mutating operationにHMAC request署名を追加し、upload/request/review/apply/gcで署名必須policyを検証できるようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 75 tests pass; `./demo.sh` |
| 2026-06-04 | Remote | HMAC request署名のnonce replay cacheを追加し、同一署名requestの再利用を拒否できるようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 76 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | SQL `CREATE TABLE` のcolumn定義を `DB-<table>.<column>` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 76 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | SQL `CREATE INDEX` を `DB-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 76 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | SQL `CREATE VIEW` / `CREATE TRIGGER` を `DB-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 76 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | SQL `CREATE FUNCTION` / `CREATE PROCEDURE` を `DB-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 76 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | SQL `CREATE SEQUENCE` / `CREATE MATERIALIZED VIEW` / `CREATE TYPE` を `DB-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 76 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | Go type/function/methodを `CODE-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 77 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | Java type/methodを `CODE-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 78 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | C# type/methodを `CODE-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 79 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | Rust type/function/methodを `CODE-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 80 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | Kotlin type/function/methodを `CODE-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 81 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | PHP type/function/methodを `CODE-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 82 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | Ruby type/function/methodを `CODE-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 83 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | Swift type/function/methodを `CODE-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 84 tests pass; `./demo.sh` |
| 2026-06-04 | Indexer | C/C++ type/function/methodを `CODE-*` Atomとして抽出するようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 85 tests pass; `./demo.sh` |
| 2026-06-04 | Release | completion planと既知制約をv0.2実装済み範囲に合わせて更新した | `README.md`, docs | TODO全件done、85 tests pass; `./demo.sh` |
| 2026-06-04 | Dogfood | CodeFire実運用で見つかったstale resolution復旧UX、Python method派生ID重複、verify診断表示の課題をbug backlog化した | `docs/development/bug-backlog.md`, docs | `algorithm-evolution-agent-lab` dogfooding |
| 2026-06-04 | Dogfood | stale resolution refresh、Python method owner派生ID、verify blocker表示を実装した | `codefire`, `tests/test_codefire_cli.py`, docs | targeted dogfood regression tests pass |
| 2026-06-04 | Diagnostics | `codefire verify --details` を追加し、missing link、stale resolution、duplicate Atom ID、failed checkの対象をCLI上で確認できるようにした | `codefire`, `tests/test_codefire_cli.py`, docs | 87 tests pass; `./demo.sh`; install smoke |
| 2026-06-04 | Dogfood | CodeFireの使いやすさ、診断ノイズ、性能可観測性、object store肥大化リスクをissue化した | `docs/development/bug-backlog.md`, `docs/development/todo-checklist.md` | CFB-005..CFB-008 |

## Current Decisions

- v0.2は local repository / local branch / open-close-clone / scan / fire / extinguish / verify / commit / merge / file-backed remote / merge request review/apply / remote GC / HTTP upload-list-clone / HTTP show-diff / HTTP merge request review/apply / HTTP doctor-gc / HTTPS transport / server verification cwd-env-timeout restrictions / hashed token storage / HMAC commit signatures / key rotation policy / HMAC request signatures / nonce replay cache までを実装範囲にする。
- AI連携、GUI、semantic merge、hosted server isolationはv0.2対象外にする。多言語Atom抽出は標準ライブラリの限定パーサで扱う。
- `examples/` はこの devcontainer 基盤リポジトリには展開しない。必要なら CodeFire 実装用の別 repository または `project/` 内の明示的な target workspace で扱う。
- 当面の実装言語はPython標準ライブラリのみ。Rust workspace構成案は将来の再構成候補として残す。

## Open Questions

- 限定Atom extractorをどこまで各言語の完全な構文解析へ近づけるか。
- OS-level sandbox / process isolationをどう設計するか。
- canonical JSON の仕様を RFC 8785 準拠に寄せるか、CodeFire独自の最小仕様にするか。
- fire解消UX、non-blocking diagnostics表示、performance metrics、large artifact retentionをどの順で実装するか。

## Change Template

```text
## YYYY-MM-DD

- Area:
- Task ID:
- Summary:
- Changed files:
- Evidence:
- Decision / Follow-up:
```
