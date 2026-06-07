# 01. 要求定義

この文書は、Code Fireの要求定義である。思想文書ではなく、実装・仕様変更・issue triage・release判定で参照する要求の正本として扱う。

## 1.1 Product Statement

Code Fireは、仕様・設計・コード・テスト・運用文書を単一のTrace Graphとして管理し、未解消の確認責任が残る状態を正式履歴へcommitできないようにする、整合性ファーストの仕様駆動VCSである。

短く言えば、次である。

> Code Fireは、整合済みシステム状態だけを封印できる、AI時代の仕様駆動バージョン管理システムである。

Code FireはGit互換ツールではない。Gitの上位レイヤでもない。履歴操作の自由度よりも、正式履歴に残る状態の意味的整合性を優先する。

## 1.2 Problem Definition

現代の開発では、ファイルが保存されているだけでは不十分である。実際に壊れるのは、成果物間の意味的な対応関係である。

Code Fireが解決する問題:

- コードだけが変わり、要求仕様が古いままになる。
- 要求仕様だけが変わり、設計・コード・テストへの影響確認が漏れる。
- 設計判断が変わり、ADRや運用Runbookが古くなる。
- API挙動が変わり、OpenAPI定義や互換性説明が追従しない。
- テストが変わり、どの要求を検証しているのか追えなくなる。
- AIや自動化ツールが大量変更を行い、人間が成果物間の整合を後追いで確認できなくなる。

Code Fireは、変更そのものを禁止しない。変更後に発生する確認責任を明示し、その責任が解消されるまで正式履歴への封印を拒否する。

## 1.3 Goals

Code Fireの主要目標は以下である。

| ID | Goal | Description |
|---|---|---|
| G-001 | Consistent-only history | branch headは常に整合済みsealed commitを指す |
| G-002 | Traceable change impact | Atom変更から関連Atomへの確認責任をfireとして提示する |
| G-003 | Evidence-backed resolution | fire解消には理由または証跡を残す |
| G-004 | Automation-safe CLI | 外部自動化ツールが安定したJSON、exit code、next_actionsで操作できる |
| G-005 | No hidden checkpoint | 未整合作業をrepository内に復元可能な形で保存しない |
| G-006 | Multi-branch workspace | branchを個別directoryとしてopenし、current branch概念を排除する |
| G-007 | Verifiable remote circulation | upload、merge request、applyでsealed commitの妥当性を検証する |
| G-008 | Local-first durability | local repositoryだけで主要workflowが完結し、remoteは流通経路として扱う |

## 1.4 Non-Goals

以下は意図的に作らない。

- Git互換性。
- Git repositoryとの透過的共存。
- checkout。
- current branch。
- staging area。
- partial commit。
- WIP commit。
- stash。
- checkpoint。
- pull/fetch/rebase。
- force upload。
- remote tracking branch。
- AI agent運用機能。
- agent marketplace。
- prompt orchestration。
- GUI。
- Code Fireによる真理判定。
- semantic conflictの完全自動解決。

これらは未実装項目ではなく、プロダクト境界である。追加要求が来た場合は、まずこの境界を変更するADRが必要である。

## 1.5 Users and Use Cases

| User | Need | Code Fire response |
|---|---|---|
| human developer | 変更影響を見落とさず小さく整合commitしたい | scan、fire、extinguish、verify、commit |
| reviewer | commitが何を根拠に整合済みとされたか確認したい | sealed commit、certificate、fire delta、evidence |
| release owner | serverへ流通するbranchを整合済みに限定したい | upload validation、merge request validation、server-side verification |
| automation tool | CLI結果を安定して機械処理したい | `--json` envelope、exit code taxonomy、next_actions |
| repository maintainer | object破損やlayout不整合を検出・修復したい | doctor、migrate、storage report |
| framework/tool author | Code Fire管理対象を拡張したい | codefire.yaml、Atom extractor、Trace Link、policy |

Code Fire本体はAI agentを起動・管理しない。ただし、AIやCIなどの外部自動化が安全にCode Fireを利用できる契約は一級要求である。

## 1.6 Domain Model Requirements

### Repository

Repositoryは `.codefire/` を正本とする保管領域である。

要求:

- repositoryはsealed commit、branch、object store、open registry、active state、policy、remote metadataを保持する。
- repositoryにはcurrent branchを持たない。
- repository構造はdoctor/migrate/storageが同じlayout定義から検査できる。
- cacheは派生データであり、破損しても正式データから再構築できなければならない。

### Branch

Branchはsealed commit列である。

要求:

- branch headは常にsealed commitを指す。
- open-burning状態のbranch headをremoteへuploadしてはならない。
- branch head更新はatomicでなければならない。
- slash入りbranch名は安全に保存・lock・remote URL化できなければならない。

### Open Directory

Open Directoryはbranchを実ファイルとして編集するための作業directoryである。

要求:

- 同一branchの多重openは禁止する。
- open directoryには `.codefire-open` markerを置く。
- marker、open registry、active state、branch head、base commitは相互検証される。
- OSコピーされたopen directoryではCode Fire mutationを拒否する。
- active stateには作業中ファイル内容、patch、snapshotを保存しない。

### Artifact and Atom

Artifactは管理対象成果物であり、Atomは成果物内の意味的な最小管理単位である。

要求:

- Atom IDはrepository内で一意でなければならない。
- Atom extractorは言語・形式ごとに限定実装でよいが、誤検出時のdiagnosticを出せなければならない。
- Atom hashは意味的単位の変更を反映しなければならない。
- 非Atomファイル変更もscanで見落とさないように報告する。

### Trace Link

Trace LinkはAtom間の意味的関係である。

要求:

- Trace Graphは要求、設計、コード、テスト、API、DB、ADR、opsの関係を表現できる。
- 必須Trace Link policyにより、欠落リンクをcommit blockerまたはwarningとして扱える。
- Trace Link IDは安定し、stale resolution判定に使える。
- Trace Linkの追加・変更はbatch操作とdry-runで検証できる。

### Fire

Fireは未解消の確認責任である。

要求:

- Atom変更、merge、manual operationによりfireを生成できる。
- fireにはsource atom、target atom、reason、severity、stable UID、display IDを持たせる。
- 同じ原因のfireを重複生成しない。
- obsolete fireはscanで整理される。
- fire数が多い場合はgroupingまたはbatch templateにより操作量を抑える。

### Resolution

Resolutionはfireに対する解消判断である。

要求:

- resolutionには解消種別、理由、証跡、basisを持たせる。
- `no-change-required` でも理由または証跡を要求する。
- basisが古くなったresolutionはstaleとしてcommit blockerになる。
- stale resolutionは通常CLIでrefreshまたは再解消できる。

### Verification and Certificate

Verificationはinternal policy checkとexternal command checkをまとめた検証結果である。

要求:

- verifyはopen fires、missing required links、stale resolutions、missing evidence refs、duplicate Atom IDs、external command failuresを判定する。
- policyによりblocking/non-blockingを分離できる。
- commit certificateには、検証対象、policy hash、各blocker count、verification result、base commitを含める。
- successful verifyでも、何が0件だったのか人間とmachine-readable JSONの両方に出す。

## 1.7 Functional Requirements

| ID | Requirement | Acceptance |
|---|---|---|
| FR-001 | `init` でrepositoryを作成する | `.codefire/` layout、main branch、initial sealed commitが作成される |
| FR-002 | `open` でbranchをdirectoryへ展開する | marker、open registry、active stateが一致する |
| FR-003 | `close` でclean open directoryを削除する | dirty/burning状態では拒否し、`--discard` のみ破棄を許す |
| FR-004 | `clone` でsealed branchから新branchを作る | open-burning sourceからcloneできない |
| FR-005 | `scan` でchanged atomsとopen firesを生成する | Atom変更、non-Atom変更、open fire countが表示・JSON出力される |
| FR-006 | `fire` でmanual fireを作成する | source/target Atom存在、重複、dry-runが検証される |
| FR-007 | `extinguish` でfireを解消する | resolution basis、rationale/evidence、stale判定が保存される |
| FR-008 | `verify` でcommit可否を判定する | blocker種別ごとのexit codeとJSON diagnosticが一致する |
| FR-009 | `commit` でconsistent stateをsealed commit化する | open fireなし、blocking verificationなし、branch head atomic update |
| FR-010 | `merge` でsealed branch同士を統合する | conflictやsemantic impactはtarget open directoryのfireになる |
| FR-011 | `show` / `diff` でsealed commitを表示比較する | 対象commitishを検証し、JSONはenvelopeで返す |
| FR-012 | `upload` / MR commandsでremote流通する | sealed commit検証、policy、signature、permissionを守る |
| FR-013 | `doctor` でrepo/remote healthを診断する | repairable/blocking/severity付きdiagnosticを返す |
| FR-014 | `migrate` でlayout/policyの移行計画を出す | check/dry-run/applyがdoctorと同じlayout定義を使う |
| FR-015 | `storage` でobject store容量とartifact参照を観測する | large object、external artifact、remote storageを報告する |
| FR-016 | `evidence` で証跡を保存する | command/artifact/URI、dry-run、失敗command扱いを明示する |
| FR-017 | `context` で変更・Atom・fire周辺情報を取得する | automationがscan/verify後に次操作を決められる |
| FR-018 | `explain` で診断の意味を説明する | fire、Atom、verify failure、storage warningをread-onlyで説明する |
| FR-019 | `completion` とinstallで利用可能にする | installed binary identity、completion生成、Python fallbackを検証する |

## 1.8 Automation Requirements

Automation-friendly interfaceは中核要求である。

要求:

- JSON対応commandは常に `codefire.command_result.v1` envelopeを返す。
- 成功時だけでなく失敗時もJSON envelopeを返す。
- `ok`、`exit_code`、`schema`、`command`、`repo`、`data`、`diagnostics`、`next_actions` の意味を崩さない。
- exit codeは `docs/automation_interface.md` のtaxonomyに従う。
- next_actionsは元のpath、branch、fire ID、Atom IDなど実行に必要なtargetをmachine-readableに含める。
- human outputは改善してよいが、JSON contractは後方互換を維持する。
- `--dry-run` は副作用なしでoperation planを返す。
- batch operationは全件validateしてからwriteする。

## 1.9 Non-Functional Requirements

| ID | Requirement | Target |
|---|---|---|
| NFR-001 | Determinism | 同じ入力から同じobject ID、Atom ID、Trace Link ID、fire UIDを生成する |
| NFR-002 | Atomicity | branch head、object record、active state更新はpartial writeを残さない |
| NFR-003 | Durability | object store writeはhash/id/filename整合性を検証可能にする |
| NFR-004 | Performance | scan/verify/statusはphase metricsを出し、large repoで支配的処理を観測できる |
| NFR-005 | Memory discipline | diff/merge/indexerは巨大fileや大量Atomで無制限メモリを使わない |
| NFR-006 | Concurrency safety | local/remote mutationはlockとidempotencyで競合を扱う |
| NFR-007 | Security | remote mutationはpermission、token、signature、nonce replay policyを持つ |
| NFR-008 | Recoverability | doctor/migrateで破損、欠落、旧layoutを説明・修復できる |
| NFR-009 | Compatibility | Python referenceで作ったrepoをRustが読め、Rust object extensionはversionedである |
| NFR-010 | Testability | command flow、object invariants、diagnostics、JSON contractを自動テスト可能にする |

## 1.10 Product Invariants

以下は実装が常に守る不変条件である。

- branch headはsealed commitだけを指す。
- sealed commitはcontent manifest、Atom index、Trace Graph、fire delta、verification、policy、certificateを参照する。
- open fireが1件以上ある状態を `open-clean` または `open-consistent` と表示しない。
- commitはblocking diagnosticが1つでもあれば失敗する。
- active stateは作業中ファイル内容を復元可能な形で保存しない。
- object IDはcanonical payloadから計算され、record内のhash/id/filenameと一致する。
- remote upload/applyはsealed commit graphを検証してから受け入れる。
- JSON envelopeの `exit_code` は実際のprocess exit codeと一致する。
- stale resolutionはcommit blockerである。
- duplicate Atom IDはcommit blockerである。

## 1.11 Release Acceptance

releaseは以下を満たさなければならない。

- 計画対象issueがfixed/wontfixとして整理されている。
- `cargo fmt --check` が通る。
- `cargo clippy --workspace --all-targets -- -D warnings` が通る。
- `cargo test --workspace` が通る。
- installer smokeが通る。
- dogfood projectでstatus、scan、verify、context、doctor、migrate、storage、evidence、diff、commit dry-runを確認している。
- 要求、設計、CLI仕様、automation interface、history、todoが実装と矛盾していない。

## 1.12 Requirement Change Control

要求変更は以下の手順で扱う。

1. 既存要求に合うbug fixか、要求そのものを変えるproposalかを分ける。
2. product boundaryを変える場合はADRを作る。
3. functional requirementを追加する場合は、CLI/API、object model、policy、test、migrationへの影響を記録する。
4. non-goalをgoalへ変える場合は、既存思想との衝突を明示する。
5. 実装後、要求定義と設計書を同じcommitまたは連続commitで更新する。

## 1.13 Success Metrics

Code Fireの成功は機能数ではなく、整合性管理の強さで測る。

- dogfood中に、変更影響の見落としがfireまたはdiagnosticとして検出される。
- AI/CIなどの外部自動化がJSONをtext parseせずに次操作を決められる。
- clean/consistent/burning状態の表示が件数と矛盾しない。
- fire解消の証跡が後から読める。
- doctor/migrate/storageが同じ健康状態を説明する。
- large repoでscan/verifyのボトルネックをmetricsから把握できる。
- issueが増えても、次version計画でroot cause単位に束ねられる。
