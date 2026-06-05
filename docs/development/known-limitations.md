# CodeFire Known Limitations

このファイルは、Python v0.2 reference implementationとRust v0.6 rewriteで意図的に残している制約、互換性差分、release前に詰めるべき項目を明示する。

## MVPで実装済みの範囲

- local repository
- local branch
- `init`, `branch list`, `open`, `close`, `clone`
- `discard`
- Markdown requirement/design/ADR/ops / OpenAPI JSON/YAML / DB schema / Python / JavaScript / TypeScript / Go / Java / C# / Rust / Kotlin / PHP / Ruby / Swift / C/C++ / pytest Atom extraction
- Trace Link parsing
- required link policy
- `scan`, auto fire, manual fire, obsolete fire
- `extinguish`, resolution basis, stale resolution
- `verify`
- `commit`
- common ancestor search
- file-level 3-way merge
- merge fire and merge commit parents
- local `show`
- local `diff`
- file-backed remote server
- remote `clone`
- `upload`
- `list`
- `request-merge`
- `request-list`
- `request-review`
- `request-apply`
- `gc`
- remote `show`
- remote `diff`
- server-side sealed commit validation
- recursive sealed commit parent history validation
- sealed commit parents / roots / certificate shape validation
- sealed commit root object type validation
- server-side verification command execution
- merge request review record
- merge request application on server
- remote `doctor`
- `install.sh --prefix`
- file-backed remote permission policy
- file-backed remote branch protection policy
- remote object garbage collection
- file-backed remote token authentication
- remote GC retention policy
- generation-aware remote GC
- remote GC audit log
- HTTP remote upload/list/clone protocol
- HTTP merge request create/list/review/apply protocol
- HTTP remote show/diff
- HTTP remote doctor/gc
- optional HTTPS transport with `cf+https://`
- server-side verification cwd/env/timeout restrictions
- shell completion generation and install
- `doctor`
- local / remote missing object reference diagnosis
- local / remote sealed commit reference validation in `doctor`
- sealed branch head validation before local open / clone / merge
- sealed base/head validation for commands executed inside an open directory
- sealed commitish validation before local / remote show / diff
- sealed branch head validation before local branch list / remote list
- sealed commit reference validation before request-list / request-review / request-apply
- remote GC health preflight before deleting unreachable objects
- object record hash / id / filename mismatch diagnosis
- repo / branch / object-store lock の最小実装

## 既知制約

実運用で見つかった不具合やバグに近いUXは `docs/development/bug-backlog.md` に分離して管理する。

### Rust default and Python fallback

Rust v0.6 is the default installed `codefire` CLI. Python v0.2 remains the reference/fallback as installed `codefire-py` or repository-local `./codefire`.

The Python fallback still owns Python-only maintenance commands that are not part of the Rust v0.6 default CLI surface yet:

- `close`
- `discard`
- `gc`
- `token-hash`

Rust v0.6 covers the release-critical local workflow, file-backed/HTTP/HTTPS remote upload/list/clone/show/diff/MR operations, diagnostics, migration checks, signatures, automation JSON, storage/evidence, diff/merge intelligence, completion, and installer default behavior. Use Python fallback for remote GC and token hash generation until those maintenance commands are explicitly ported.

### Config YAML subset

現在は標準ライブラリのみで動かすため、`codefire.yaml`, `codefire.links.yaml`, `codefire.policy.yaml` はMVPに必要な形だけを読む限定YAML subsetとして処理している。
`commit_policy` のboolean項目は verify/commit と sealed commit validation に反映される。
`extinguish_policy.no-change-required.requires_rationale` は `extinguish` 判定に反映される。
`codefire.policy.yaml` の `verification.required` は複数commandに対応しているが、一般的なYAML機能全体を実装しているわけではない。
`required_links` はAtom kindごとの `type`, `target_kind`, `min` を読める。
限定subset内の必須項目不足は、設定ミスを黙って無視せず `invalid codefire.*.yaml` として失敗させる。現在は artifact entry の `path`、Trace Link の `from` / `to` / `type`、verification entry の `command`、required link rule の `type` / `target_kind` / 整数 `min` を検証する。

この方針は `adr/ADR-007-limited-config-yaml-subset.md` に記録している。

将来的に一般YAML parserへ移行する余地は残す。

### Object ID and Commit ID

object IDは `sha256(type_tag || 0x00 || canonical_payload)` をベースにしている。

commit payload内に自身の `commit_id` を含めると循環参照になるため、現在の実装ではobject record側の `object_id` とbranch headを正本にしている。

この方針は `adr/ADR-006-object-id-no-self-reference.md` に記録している。

### Remote Features

remote機能は、Python v0.2とRust v0.6 rewriteの両方でローカルファイル-backed server、HTTP server、HTTPS serverとして実装している。

URL例:

```text
cf:///tmp/codefire-server/org/app/main
cf+http://127.0.0.1:8080/org/app/main
cf+https://127.0.0.1:8443/org/app/main
```

server側は `.codefire-server/projects/<org>/<app>/` にsealed object、branch、merge requestを保存する。Python v0.2の `codefire serve <storage-root>` とRust v0.6の `codefire-rs serve <storage-root>` は同じ保存形式をHTTP/HTTPS越しに公開する。

Python v0.2とRust v0.6のHTTP/HTTPS transportで実装済み:

- `upload`
- `list`
- `clone`
- `show`
- `diff`
- `request-merge`
- `request-list`
- `request-review`
- `request-apply`
- `doctor`
- `gc`

HTTPS transportは `codefire serve --tls-cert --tls-key` / `codefire-rs serve --tls-cert --tls-key` で有効化できる。通常のTLS証明書検証を使い、自署名証明書のローカル検証時のみ `CODEFIRE_TLS_INSECURE=1` で検証を無効化できる。

server-side verificationは `server_policy.json` に書いたcommandをremote project root配下で実行する。checkごとに `cwd`、`env`、`timeout_seconds` を指定でき、`cwd` はremote project root配下に制限される。親プロセス環境は丸ごと渡さず、最小環境、CodeFire remote metadata、checkごとの `env` だけを渡す。OS-level sandboxやコンテナ隔離はまだ持たない。

merge request applyはfast-forwardのみを許可する。serverはsemantic mergeを行わない。

permissionは `server_policy.json` の `permissions` で操作ごとにactor名を許可するfile-backed実装である。
branch protectionは `server_policy.json` の `branch_protection` でbranch patternごとに `upload` / `apply` のactorを制御するfile-backed実装である。
`auth.required` と `auth.tokens` を設定すると、mutating operationは `--token` または `CODEFIRE_TOKEN` の一致も要求する。
tokenは既存互換の平文文字列に加えて、`sha256:<hex>` 文字列またはsalt付きhash objectとして保存できる。これはremote policy内の平文token保存を避けるためのMVP機能である。
Python v0.2とRust v0.6ではcommitは `CODEFIRE_SIGNING_KEY` と `--signer` / `--key-id` でHMAC-SHA256署名を付与でき、remote policyの `commit_signatures.required` / `commit_signatures.keys` でupload/applyされるbranch headへの署名を要求できる。key objectの `not_before` / `not_after` / `status` / `signers` と `require_history` により、MVP範囲のkey rotationと失効検査を扱える。これは共有鍵署名であり、公開鍵署名と外部KMS連携はまだ持たない。
Python v0.2とRust v0.6ではremote mutating operationは `CODEFIRE_REQUEST_SIGNING_KEY` と `--request-key-id` でHMAC-SHA256 request署名を付与でき、remote policyの `request_signatures.required` / `request_signatures.keys` / `max_skew_seconds` / `nonce_ttl_seconds` で署名、timestampずれ、nonce replayを検証できる。これは共有鍵request署名であり、公開鍵署名、外部KMS連携、分散remote向けの共有nonce storeはまだ持たない。

GCは削除前にobject hash、missing object reference、sealed commit参照を検査する。健全なremoteでは到達不能objectを削除し、`gc.retention_seconds` と `gc.retention_generations` により新しい到達不能objectを保護できる。
remote projectは `gc_state.json` に現在世代を保存し、branch head更新ごとに世代を進める。remote object recordには `remote.first_seen_generation` / `remote.last_seen_generation` を付与する。
dry-runと実削除の結果は `audit/gc.jsonl` にJSONL形式で記録される。

### Parser Coverage

Atom抽出はMVP対象に限定している。

- Markdown heading Atom
- Markdown ADR Atom
- Markdown ops/runbook Atom
- OpenAPI JSON/YAML operation Atom
- SQL `CREATE TABLE` table/column, `CREATE INDEX`, `CREATE VIEW`, `CREATE MATERIALIZED VIEW`, `CREATE TRIGGER`, `CREATE FUNCTION`, `CREATE PROCEDURE`, `CREATE SEQUENCE`, and `CREATE TYPE` DB schema Atom
- `# cf-atom:` 付きPython class/function/test
- `// cf-atom:` 付きJavaScript/TypeScript class/function
- `// cf-atom:` 付きGo type/function/method
- `// cf-atom:` 付きJava type/method
- `// cf-atom:` 付きC# type/method
- `// cf-atom:` 付きRust type/function/method
- `// cf-atom:` 付きKotlin type/function/method
- `// cf-atom:` 付きPHP type/function/method
- `# cf-atom:` 付きRuby type/function/method
- `// cf-atom:` 付きSwift type/function/method
- `// cf-atom:` 付きC/C++ type/function/method
- 明示IDがないPython class/functionの派生ID
- 明示IDがないJavaScript/TypeScript class/function/arrow functionの派生ID
- 明示IDがないGo type/function/methodの派生ID
- 明示IDがないJava type/methodの派生ID
- 明示IDがないC# type/methodの派生ID
- 明示IDがないRust type/function/methodの派生ID
- 明示IDがないKotlin type/function/methodの派生ID
- 明示IDがないPHP type/function/methodの派生ID
- 明示IDがないRuby type/function/methodの派生ID
- 明示IDがないSwift type/function/methodの派生ID
- 明示IDがないC/C++ type/function/methodの派生ID

DB schema parserは主要DDLのAtom抽出に対応しているが、dialect固有の全DDL、複雑なblock構文、各DDL内部要素の細分化までは未実装。Go/Java/C#/Rust/Kotlin/PHP/Ruby/Swift/C/C++以外の追加言語の詳細抽出も未実装。

Python extractorはclass methodの派生IDにclass ownerを含める。より複雑なdecorator、dynamic class生成、入れ子class、metaprogrammingの完全解析は対象外である。

### Merge Semantics

mergeはfile-level 3-way mergeである。

semantic merge、Atom-level自動解決、AI提案は未実装。競合時はconflict markerを生成し、verify/commit blockerにする。

### Packaging

`install.sh --prefix PATH` はRust v0.6 binaryを `PATH/bin/codefire` に配置し、Python v0.2 reference implementationを `PATH/bin/codefire-py` に配置する。`codefire completion bash|zsh` でRust CLIのshell completionを生成でき、`install.sh --completion bash|zsh` で配置できる。`pyproject.toml` / `setup.py` はPython reference implementationのsetuptools script installに対応しており、`python3 -m pip install .` はPython版 `codefire` entrypointを配置するため、Rust default installとは別経路として扱う。

Rust workspace自体は `Cargo.toml` / `crates/codefire-*` として作成済みで、installer defaultはRustへ切替済みである。ただしpackage indexへの公開、署名付きrelease artifact、OS packageは未実装である。
