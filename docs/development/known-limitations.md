# CodeFire Known Limitations

このファイルは、Python v0.2 reference implementationとRust v0.6 rewriteで意図的に残している制約、互換性差分、release前に詰めるべき項目を明示する。

## MVPで実装済みの範囲

- local repository
- local branch
- `init`, `branch list`, `open`, `close`, `clone`
- existing-directory `import` with durable provenance/retry
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
- sealed Trace Graph endpoint validation with bounded typed diagnostics
- schema-2 cross-root sealed snapshot validation through the authoritative core predicate
- explicit schema-1 sealed-read compatibility and fail-closed future commit/root versions
- descriptor-relative sealed-object traversal and root/subdirectory-bound command cache
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
- persistent kernel-backed local repository/import bootstrap locks
- capability-relative no-follow `open` materialization
- installer active PATH reporting

## 既知制約

実運用で見つかった不具合やバグに近いUXは `docs/development/bug-backlog.md` に分離して管理する。

### Rust default and Python fallback

Rust v0.6 is the default installed `codefire` CLI. Python v0.2 remains the reference/fallback as installed `codefire-py` or repository-local `./codefire`.

`install.sh` verifies the installed binary and reports which `codefire` command is active on PATH after installation. A successful install can still be shadowed by an earlier PATH entry; in that case the installer emits a warning and the operator must reorder PATH, install to the earlier prefix, or invoke the installed target directly.

Python fallback freeze policy:

- New mainline features must land in Rust first.
- Python fallback changes are limited to compatibility fixtures, critical security/bug fixes, installer fallback behavior, and narrowly-scoped Python/Rust parity tests.
- Python fallback must not gain a new user-facing command, flag, output schema, remote behavior, or repository format unless the Rust CLI already owns that behavior and the temporary fallback delta is documented here.
- Golden compatibility tests should cover only repository/object/remote formats that Python-created projects still need for migration or incident response.

The Python fallback still owns Python-only maintenance commands that are not part of the Rust default CLI surface yet:

- `gc`
- `token-hash`

Rust covers the release-critical local workflow including branch close/discard, file-backed/HTTP/HTTPS remote upload/list/clone/show/diff/MR operations, diagnostics, migration checks, signatures, automation JSON, storage/evidence, diff/merge intelligence, completion, and installer default behavior. Use Python fallback for remote GC and token hash generation until those maintenance commands are explicitly ported.

ADR-013's typed committable decision and non-relaxable safety-floor policy are Rust-default
contracts. The frozen Python fallback retains its historical boolean-policy behavior for legacy
fixtures and is not a current-profile consistency writer; do not use `codefire-py` to create a new
seal with any safety-floor field set to `false`. A cross-runtime policy corpus and fallback
retirement/write restriction belong to RVC-053/RVC-055 rather than silently extending the fallback.

Fallback削除条件:

- Rust CLIの `COMPLETION_COMMANDS` / help / shell completionに対象commandが追加される。
- 対象commandのRust実装、JSON出力、exit code、主要failure modeのテストが追加される。
- Python-created repository / remote projectでの互換性テストが必要範囲で通る。
- このfallback一覧から対象commandを削除し、`known_limitations_python_fallback_commands_stay_out_of_rust_cli` が通る。

### Config YAML subset

現在は標準ライブラリのみで動かすため、`codefire.yaml`, `codefire.links.yaml`, `codefire.policy.yaml` はMVPに必要な形だけを読む限定YAML subsetとして処理している。
`commit_policy.require_trace_completeness`、`require_commit_signature`はRust verify/commitへ反映される。`require_no_required_fires`、`require_no_stale_resolutions`、`require_verification_success`、`reject_duplicate_atom_ids`は互換fieldだがRustのsafety floorとして`true`固定で、working configの`false`は明示的に拒否する。
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

HTTPS transportは `codefire serve --tls-cert --tls-key` / `codefire-rs serve --tls-cert --tls-key` で有効化できる。Rust版は `--tls-client-ca <ca-pem>` を指定するとclient certificateを必須にする。server private key PEMはPKCS#8、RSA、SEC1 ECに対応する。通常のTLS証明書検証を使い、自署名証明書のローカル検証時のみ `CODEFIRE_TLS_INSECURE=1` で検証を無効化できる。

server-side verificationは `server_policy.json` に書いたcommandをremote project root配下で実行する。checkごとに `cwd`、`env`、`timeout_seconds` を指定でき、`cwd` はremote project root配下に制限される。親プロセス環境は丸ごと渡さず、最小環境、CodeFire remote metadata、checkごとの `env` だけを渡す。OS-level sandboxやコンテナ隔離はまだ持たない。

merge request applyはfast-forwardのみを許可する。serverはsemantic mergeを行わない。

全remote mutationとGCはpersistent project reader/writer gateを共有し、upload / request-merge /
request-review / request-applyはkey有無にかかわらずdurable publication transactionを使う。
compound readerはshared gate取得後にfixed pending authorityを確認する。MR reviewはversioned subjectへ
束縛され、cross-project source graphはtargetへescrowされるため、applyはcurrent source branchではなく
review済みheadだけを使う。これはlocal filesystem上のcooperating CodeFire writer contractであり、同じ
filesystem credentialを持つ非協調processに対するOS isolationやdistributed consensusではない。

permissionは `server_policy.json` の `permissions` で操作ごとにactor名を許可するfile-backed実装である。
branch protectionは `server_policy.json` の `branch_protection` でbranch patternごとに `upload` / `apply` のactorを制御するfile-backed実装である。
`auth.required` と `auth.tokens` を設定すると、mutating operationは `--token` または `CODEFIRE_TOKEN` の一致も要求する。
tokenは既存互換の平文文字列に加えて、`sha256:<hex>` 文字列またはsalt付きhash objectとして保存できる。これはremote policy内の平文token保存を避けるためのMVP機能である。
Python v0.2とRust v0.6ではcommitは `CODEFIRE_SIGNING_KEY` と `--signer` / `--key-id` でHMAC-SHA256署名を付与でき、remote policyの `commit_signatures.required` / `commit_signatures.keys` でupload/applyされるbranch headへの署名を要求できる。key objectの `not_before` / `not_after` / `status` / `signers` と `require_history` により、MVP範囲のkey rotationと失効検査を扱える。これは共有鍵署名であり、公開鍵署名と外部KMS連携はまだ持たない。
Python fallbackにはsame-ledger pure signature owner、pinned raw policy authority、sealed commit borrowのproduction前提があるが、handlerには未接続である。policy authorityはproduction-unwired combined surfaceでsignature child成功・close後に同一P0 bytesとduplicated capabilityをexternal verificationへ貸せる。二段階signature compositeはsealed ownerのexact preallocated handleをwrapper/clone/reopenなしで貸し、verification child構築前にguard下でreleaseし、失敗cleanupをborrow→policy→sealedのLIFOでretryするが、これもterminal coordinatorの代替ではない。二つのfinal fence間callback、missing-project bootstrap、external verification effect、gate内ancestryを一つのterminal coordinatorへ閉じるまでproduction wiringはFreezeである。
Python v0.2とRust v0.6ではremote mutating operationは `CODEFIRE_REQUEST_SIGNING_KEY` と `--request-key-id` でversion 2 HMAC-SHA256 request署名を付与でき、remote policyの `request_signatures.required` / `request_signatures.keys` / `max_skew_seconds` / `nonce_ttl_seconds` / `max_nonce_cache_entries` で完全payload subject、key window、timestampずれ、nonce replayを検証できる。cache容量到達時はlive nonceをevictせず新規requestをfail closedする。これは共有鍵request署名とproject-local file cacheであり、公開鍵署名、外部KMS連携、複数server node向けの分散nonce storeはまだ持たない。

local verification/evidence commandはstreaming cap、minimal environment、digest/redaction、cwd classification、Unix process-group cleanupを持つが、OS-level sandboxではない。`local_unisolated` resultは実行subjectを説明するだけで、host filesystem/networkから隔離されたproofへ昇格しない。live subjectのcanonical worktree digestはverification開始、各command、commit planning、branch publication境界で再照合するが、外部processによる変更をfilesystem snapshotとして原子的に禁止するものではない。verificationで生成物をopen worktree内へ書くcommandはsubject mutationとしてblockingになるため、tool cache/outputは明示的にworktree外へredirectする必要がある。非Unixではtimeout cleanupの記録scopeは`direct_child`であり、子孫全体のterminationをclaimしない。

GCは削除前にobject hash、missing object reference、sealed commit参照を検査する。健全なremoteでは到達不能objectを削除し、`gc.retention_seconds` と `gc.retention_generations` により新しい到達不能objectを保護できる。
remote projectは `gc_state.json` に現在世代を保存し、branch head更新ごとに世代を進める。世代保持metadataはcontent-addressed object recordを変更せず、bounded/versioned `retention_index.json` に保存する。
GCはpolicy/root/inventory/retention/plan fingerprint、plan-owned quarantine、delete progressを持つdurable transactionである。exact `gc/GC-*.json` がaudit authorityで、`audit/gc.jsonl` はそこから決定的に再構築されるderived viewである。

### Local Filesystem and Recovery Boundary

Python external/sealed capability owners detach a numeric file descriptor before invoking one module-captured builtin close and
never retry that number after an exception. Linux can reuse the same descriptor number for the same inode, so EBADF or inode
identity cannot prove that a retry still targets the original authority. This deliberately accepts a possible descriptor leak under
an injected pre-call close failure rather than risk closing an unrelated ambient capability. Remaining payload/accounting cleanup is
retryable without touching that number. The sealed owner is currently prerequisite-only and is not yet production-wired.

Rust CLIのlocal `RepoLock`、import bootstrap lock、import中のrepository lock、remote project gate、request nonce lockは、persistent regular file上のadvisory kernel lockである。repository `repo.lock`はno-followかつexactly one filesystem linkを証明できる場合だけ使い、contentをtruncate/writeせず、取得前後とmutation use boundaryでidentity/link provenanceを再検証する。process終了でownershipは解放されるが、lock inodeは残り、best-effort owner情報の`.owner.json` sidecarも残る。sidecarはownershipのauthorityではなく、contention時にno-follow/16 KiB上限で安全に読めた場合だけdiagnosticへ使う。advisory lockを無視する別processのwriteをOSが強制的に防ぐものではなく、validation、transaction recovery、atomic publicationは依然必要である。

`open`のworktree write、import subject/repository-lock identity、sealed-object cacheのstable identityはUnixでdevice/inode、Windowsでvolume/file indexを使う。これらを証明できないplatformでは、open/import identity-sensitive operationはfail closedし、sealed validationはpinned handleによる1 traversal内reuseのみを行ってcacheをcall間で再利用しない。

sealed-validation cacheはobject-rootと各object subdirectoryのidentity/absenceを主体に含める。built-in 12 subdirectoryに加えてbindできるextension subdirectoryは128までで、129個目は`InvalidSealedCommit`にする。cacheが保持するはcurrent store subject 1つだけで、別root/identity setへ移ると旧pin/resultをevictするため、後で戻れば再検証になる。一方、公開済みobject fileを同一inode上で不正に書き換える行為をcache hit時に再検知するものではない。object file immutabilityは依然cooperative store contractである。

serialized object recordはwriter/readerとも64 MiB上限である。readerはsymlinkをfollowせずnon-blocking regular-file openとbounded readを使うため、FIFO/device/socketや上限超過recordは正当なobjectとしては読めない。これは信頼境界の意図的なproduct limitである。

object writer/reader、sealed traversal、full object scanはabsolute object root/subdirectoryをcomponent-wise no-followで開く。writerはpublication/reuse後、sealed/full scanはsnapshot末尾にroot/subdirectory identityを再検査するため、symlink rootや差し替えsubjectを成功として継続しない。ただし同一inode上の公開済みrecord書換えをcache hitごとに検知するものではない。

現行Rust sealはcommit/root object version 1、root schema 2、semantic profile `sealed-snapshot-v1`を書き、schema 2のcore predicateとcurrent profileのAtom/Trace endpoint closureを必須とする。root schema欠落はlegacy schema 1、semantic profile欠落は独立した`legacy-unversioned` cross-root claimとして読めるが、unknown profile、schema 1/2以外、future commit/root versionはfail closedする。legacy profileでもobject/root/reference/schema-2 certificate semanticsは省略しない。`missing_evidence_refs`の省略互換は空list/countだけである。remote branch/MR、object extension、公開base schema familyは`codefire.support.v1`とchecksum conformance corpusへ登録したが、全command固有payloadのpublic schema化は未完了でありincrementalに行う。

object recordの未知noncritical extensionはexact transferし、宣言されたobject referenceをreachabilityへ含める。未知critical extensionはfail closedする。ただし現行descriptor registryが意味を理解するcritical extensionはまだなく、extension payloadのdomain validationと全既存top-level metadataのnamespace移行は将来作業である。

remote `list` / `request-list`のcursorは最大1000件/page、最大4096文字で、完全ordered resultのdigestを毎回計算してsnapshot driftを拒否する。これは結果混在を防ぐcontractであり、巨大collectionでのserver-side streaming/index cursorではない。remote Doctorはissue sampleと報告record数をboundするが、CFR-168で予定する完全なI/O work budget/streaming inventoryを置き換えない。

effective policyはstable rule/source/value/applicability/safety classを保持し、ChangeCoverageは全manifest差分を`non_atom_file`、`deleted_atom`、`out_of_atom_change`、`atom_change_with_unclassified_spans`へ分類する。ただし現行Atom indexはline/byte ownership spanを保持しないため、coverage precisionは`file_only_no_span_claim`であり、mixed fileのどのbyteがAtom内外かを証明しない。現行coverage ruleは`allow_with_disclosure`で、分類自体は新しいcommit blockerではない。path/class別review必須化やdenyは、span/evidence workflowと明示的なpolicy/schema profile evolutionを伴う将来作業である。

generic repository JSONはread/writeとも64 MiB上限である。readerはabsolute parentとfinal fileをno-followで開き、finalをnon-blocking regular fileとしてmetadataとlimit-plus-oneの両方で制限する。各transaction/import/owner recordのより小さい上限は別途優先される。

Rust CLIのopened active stateはparent/instance directory capabilityへbindし、active file名を1個のnormal relative componentに制限する。read/write/removeはこのhandleに相対で、operation境界にambient identityを再検証するため、validation後のinstance renameとoutside symlink差替えは`active_state_capability_invalid`としてfail closedする。これは同じdirectory/file inodeへadvisory lockを無視して直接書き込む別processをOSが禁止する保証ではない。また、active stateをnested directory schemaへ拡張する場合は現在のflat-file capability APIを先に明示的に拡張する必要がある。

close/discardはpending marker、5 phase、worktree/active/openedのtransaction固有deterministic quarantine、object-first branch publish、fixed completed archiveでmulti-record deletionをretry可能にする。atomic no-replace renameまたはstable target/parent identityを証明できないplatformではfail closedする。削除開始後に失敗したquarantineは自動で元nameへ戻さず、exact request retryが同じidentity/nameから再開するため保持する。pending/completed markerやerrorが示すquarantineを一般的な一時fileとして手動削除してはならない。これはcooperative repository lockを無視したprocessによる同一inodeへの直接書換えをOSが禁止する保証ではなく、background recovery daemonも持たない。

importはparsed operational/control JSONを1 MiB、repository byte copyを64 MiB、通常のuser fileを49,545,216 bytes（47.25 MiB）に制限する。user file上限はbase64 expansionと1 MiBのrecord overhead reserveを含めてobject recordの64 MiB上限内へ収めるためのproduct limitであり、巨大file importをstreamingで扱う機能はまだない。

scan/verifyのstate transactionはpartial multi-file writeをrollbackせず、pending markerとretry再計算でhealする。現行fixtureはscan/verifyの全phase直後、cross-operation takeover、最終authority divergenceを固定し、成功時はfires/scan/verificationとregistry/active/branchのauthority整合を再検証してからmarkerを削除する。ただしadvisory lockを無視して同じinodeへ直接書く外部processをOSが禁止するものではない。commitはmanifest-blob/root/candidate/branch-snapshotのstore-to-marker gap、pre-validation、主要candidate corruptionを固定しているが、全entry/全phaseのfailpointではない。doctorはpending commitのrole/path/type/ID/stateを独立fieldで返さず、marker pathとphaseおよびstored/planned role-entry countのmessageを返す。still-planned entryのIDがstore-to-marker gapで既に存在する可能性は意図したrecovery contractである。

importのroot `.codefire-import.lock.bootstrap-*.part` は未公開scratchであり、authorityとしてparseせずscanから除外し、canonical lockとのidentityが証明できない限り保存する。crashed contenderの`.part`が残ることは安全性優先の意図的な制約である。fully-written bootstrap `.tmp`とtransaction-bound write `.tmp`は別のrecovery ruleを持つため、一括して手動削除してはならない。

### Parser Coverage

Atom抽出はMVP対象に限定している。

下記はCodeFire全体（Rust defaultとPython fallbackの和）のcoverageである。Rust defaultの
hash schema 4はMarkdown、明示marker、Python derived class/top-level def/class-owned methodを
実装する。その他言語のderived Atomは現在Python fallbackのみであり、Rust defaultでは
明示`cf-atom`を要求する。fallback coverageをRustのcurrent writer claimへ読み替えない。
Python経路はstring/comment-awareな線形lexical viewを使い、fixture/docstring内の宣言風textや
`cf-atom` textをAtomとして扱わない。

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

semantic merge、Atom-level自動解決、AI提案は未実装。Rust defaultではtext/binary/semantic競合はtyped verify/commit blockerとなり、text markerとbinary sidecarを手動解消し、semantic candidateを明示確認した後に`merge --continue`でresolution transitionを記録する。CodeFireは競合内容そのものを自動選択しない。Python fallbackは既存text-conflict blockerを維持するが、Rustのcontinue writerは持たない。

### Packaging

`install.sh --prefix PATH` はRust v0.6 binaryを `PATH/bin/codefire` に配置し、Python v0.2 reference implementationを `PATH/bin/codefire-py` に配置する。`--dry-run` はbuild/mkdir/installを行わず配置計画だけを表示する。通常installはRust release binaryをbuildし、`--binary PATH` 指定時だけprebuilt binaryを使う。いずれも `--help` / `--version` smokeでCodeFire identityを検証し、completionは一時fileに事前生成してから配置する。prefix/completion dirは絶対pathを要求し、symlink prefix/bin/completion dirは拒否する。`codefire completion bash|zsh` でRust CLIのshell completionを生成でき、`install.sh --completion bash|zsh` で配置できる。`pyproject.toml` / `setup.py` はPython reference implementationのsetuptools script installに対応しており、`python3 -m pip install .` はPython版 `codefire` entrypointを配置するため、Rust default installとは別経路として扱う。

Rust workspace自体は `Cargo.toml` / `crates/codefire-*` として作成済みで、installer defaultはRustへ切替済みである。ただしpackage indexへの公開、署名付きrelease artifact、OS packageは未実装である。
