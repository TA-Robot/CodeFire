# Module Boundary Standard

この文書は、CodeFire本体を改修する際のmodule/file分割基準を定義する。

CodeFireはobject identity、sealed commit、branch mutation、verification、remote policyを扱うため、単一ファイルへ機能を集約すると不変条件の所在が不明確になりやすい。v0.6以降のRust実装では、責務境界をコード構造として表現することを必須とする。

## 原則

- `main.rs` はCLI entrypointとdispatchに限定する。
- command parsing、domain validation、operation planning、state mutation、renderingを同じ関数へ混ぜない。
- shared invariantを守る処理は専用moduleへ切り出し、呼び出し側に重複実装しない。
- file-backed remote、HTTP remote、signature、TLS、idempotency、diff、merge、verificationは、それぞれ独立したmodule境界を持つ。
- human outputとJSON/machine-readable outputを同じ生成関数で兼用しない。
- test helperとproduction pathを混ぜない。

## 分割判断

以下のどれかに該当したら、同じファイルへ追記せずmodule分割を検討する。

- 1つの関数がparse、validate、write、renderのうち2種類以上を担っている。
- 1つのファイルに複数commandのdomain logicが混在している。
- 同じ不変条件のチェックが2箇所以上に複製されている。
- 新しいpolicy、signature、lock、remote mutation、object write pathを追加する。
- テストで内部関数を直接検証したいが、`main.rs` に閉じているため呼べない。
- 変更差分を読んだときに、どの不変条件を守る変更かがファイル名から分からない。

## 推奨構成

Rust CLIでは、以下の責務分離を標準形とする。

| Responsibility | Preferred location | Notes |
|---|---|---|
| CLI dispatch / argument routing | `main.rs` | 薄く保つ。command固有ロジックを置かない |
| CLI option/result/context model | `crates/codefire-cli/src/cli_model.rs` | command modulesが共有する型を集約し、fieldsは`pub(crate)`に限定する |
| command-specific planning | `crates/codefire-cli/src/<command>.rs` | parse済み入力からplanを作る |
| domain/Trace invariants | `codefire-core` | Atom/Trace endpoint、Fire、resolution、verification predicateを集中管理する。effective policyは`effective_policy.rs`、manifest/Atom relationは`change_coverage.rs`、indexed direct Trace evaluationは`trace_evaluation.rs`をownerとする |
| deterministic work contract | `codefire-core/work_budget.rs` | profile/dimension/counter/phase/terminalとatomic reservation ledgerを所有する。HTTP/Store/CLIは独自limit語彙やfork可能なper-head ledgerを作らない |
| object/sealed invariants | `codefire-store` | canonical JSON、full digest/object ID、collision、descriptor-relative sealed validation/cache subjectを集中管理する |
| open/close/import filesystem capability | `secure_worktree.rs`, `close_worktree.rs`, `active_state.rs`, `import_workflow.rs` | stable identity/no-replace primitiveは共有し、open staging、close subject、active instance、import recoveryのpolicyは各ownerへ分ける |
| close/discard publication | `crates/codefire-cli/src/close_transaction.rs` | request/evidence/phase/archiveとordered applyを所有し、`main.rs`はdispatchだけにする |
| remote project authority | `remote/project.rs` | percent-decode後component、stable project descriptor、ledger-bound hidden bootstrap、bounded recovery/quarantine、legacy-adoption境界を所有する |
| remote project locking | `remote/locks.rs`, `stable_lock.rs` | persistent inode、shared/exclusive gate、stable-ID順の複数project取得、単一deadline、pending read guardを所有する |
| remote mutation policy | `remote/` | file-backed/HTTPで共通policyを共有し、任意pathをpublication APIへ渡さない |
| request/commit signatures | `signatures.rs` | signing、verification、policy evaluation、iterative history walk、nonce validation/publication境界を集約する |
| rendering / view | `view/` | human outputとstructured outputを分ける |
| diagnostics | feature module or `doctor/` | exit codeとmachine-readable detailsを明確にする |
| idempotency / lock handling | dedicated module | mutation commandから共通利用する。persistent kernel lockはcontent-immutable inode、exactly-one-link identity、atomic owner sidecarを一契約にし、legacy create-new lockと同じ型に見せかけない |

Stage 1の現行所有では、`secure_worktree` がstable filesystem identity、atomic no-replace rename、open targetとrepository-lock pathのno-follow primitiveを所有する。`close_worktree` はmarker/worktree removal、`active_state` はactive-instance relative I/O/removal、`close_transaction` はmulti-record publication/recoveryを所有する。`import_workflow` はbootstrap provenance/recoveryとpinned import subject、`codefire-store` はsealed traversalのobject-root/subdirectory capabilityを所有する。同じcapability crateを使っても、open staging、close policy、active state、import transaction、immutable-object read/cacheを一つの巨大filesystem moduleへ統合しない。

Python reference implementationは歴史的に単一ファイルだが、Rust v0.6 default化後は互換性確認用referenceとして扱う。新規の本流機能はRust側でmodule分割して実装する。

## テスト配置

Rust CLIの横断integration-style testsは、`crates/codefire-cli/src/tests.rs` を入口にしつつ、目的別のsubmoduleへ分割する。

| Test domain | Preferred location | Notes |
|---|---|---|
| docs/help/completion/release-doc consistency | `crates/codefire-cli/src/tests/docs.rs` | CLI surfaceと文書の整合を固定する |
| doctor/storage/state diagnostics | `crates/codefire-cli/src/tests/doctor.rs` or feature-local module | 破損fixtureとdiagnostic schemaをまとめる |
| local branch/open/clone/status flows | `crates/codefire-cli/src/tests/local.rs` | repository lifecycle helpersを共有する |
| diff/merge/review-pack | `crates/codefire-cli/src/tests/diff_merge.rs` or `view/*` module tests | golden fixturesとnormalizerを近くに置く |
| remote/idempotency/http flows | `crates/codefire-cli/src/tests/remote.rs` or feature-local module | remote fixture setupを共有する |
| evidence/fire/extinguish/verify | `crates/codefire-cli/src/tests/workflow.rs` | traceability workflowのE2Eをまとめる |

新規Rust CLIテストを追加するときは、既存の巨大な `tests.rs` へ直接足す前に、上記domain submoduleまたはfeature-local `#[cfg(test)] mod tests` を優先する。既存テストは、関連featureを触るたびに近いdomain fileへ段階移動する。

## 実装フロー

1. 変更前に、追加する責務が既存moduleに属するかを確認する。
2. 既存moduleが肥大化している場合は、先に小さな抽出commitを作る。
3. public APIは狭くする。module外へ出す型や関数は、呼び出し側の責務に必要なものだけにする。
4. mutation前にはplan型を作り、検証とwriteを分離する。
5. outputは最後にrenderする。domain logic内で直接printしない。
6. testsはmodule境界に沿って置き、private helperを無理に公開しない。横断CLI testsは `src/tests/<domain>.rs` へ置く。

## レビュー観点

- 新しい責務が `main.rs` に増えていないか。
- file nameとmodule nameから責務が推測できるか。
- invariant checkが重複していないか。
- parse error、policy error、repository corruption、internal invariant violationが区別されているか。
- 大規模repoで支配的になる処理が、不要な全件読み込みや再parseをしていないか。
- machine-readable contractがhuman outputの都合で揺れていないか。

## 禁止パターン

- commandごとの長大なmatch armにdomain logicを直接書く。
- signatureやpolicy verificationをremote実装ごとにコピーする。
- JSON serializationとhuman message生成を同じ文字列組み立てで行う。
- filesystem write中に追加検証を挟み、失敗時のrollback方針を曖昧にする。
- 一時的な互換shimを期限や削除条件なしに置く。

## 記録ルール

module分割が必要なのに時間都合で後回しにする場合は、必ず `docs/development/bug-backlog.md` または `docs/development/todo-checklist.md` に記録する。記録には、対象file、肥大化している責務、想定される分割先、放置した場合のリスクを書く。
