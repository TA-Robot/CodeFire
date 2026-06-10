# CodeFire Improvement Cycle

この文書は、CodeFire本体をCodeFire自身と実験サブプロジェクトでdogfoodし、継続的に改善する反復運用を定義する。

この反復を **codefire改善サイクル** と呼ぶ。

## Purpose

codefire改善サイクルの目的は、CodeFireを机上の設計だけで強化するのではなく、実際の開発管理で使い、使いづらさ、バグ、automation contractの弱さ、設計不足を継続的に検出して次versionの計画へ戻すことである。

CodeFire本体にAI agent運用機能は入れない。ただし、外部自動化ツールや人間がCodeFireを安全に使えるよう、CLI、JSON、exit code、next_actions、dry-run、evidence、batch、diagnosticsを強くする。

## Repositories

| Area | Location | Role |
|---|---|---|
| CodeFire product | repository root | Rust CLI/core/store/docsを開発する |
| Algorithm dogfood project | `subprojects/algorithm-evolution-agent-lab/` | CodeFire管理対象として実開発を進め、UX/bugを検出する |
| CFB issue management | `docs/development/bug-backlog.md`, `docs/development/bug-issues/` | dogfoodingで見つけたCodeFire issueを管理する |
| Code review issue management | `docs/development/code-review-issues/` | ソースコードレビューで見つけた改善issueを管理する |
| Version planning | `docs/development/v*-*.md` | 次versionでissueを消すための計画と設計を残す |

## Cycle Overview

1. Discovery: Algorithm dogfood projectでCodeFireを使い、バグ、UX違和感、JSON contract不整合、速度、記憶領域、diagnostic不足を `CFB-*` として記録する。
2. Expansion: 必要ならCodeFireソースコードレビューを行い、同じbatchの `CFR-*` を追加する。
3. Freeze: そのcycleで修正するissue batchを確定する。この時点で次のissue探索を止める。
4. Burn-down planning: batch内issueをroot cause単位に束ね、全部消すための修正計画を作る。
5. CodeFire development: 計画に沿ってbatch内issueを全部fixedまたは明示wontfixにする。
6. Install: 開発したCodeFireをinstallし、実際の `codefire` commandとして使える状態にする。
7. Confirmation dogfood: Algorithm projectでbatch内issueの再現コマンドを再実行し、修正を確認する。
8. Close audit: backlog、detail issue、history、version plan、cycle planが一致していることを確認する。
9. Next discovery: batch内issueが全部消えてから、次のalgorithm計画と次のissue出しへ進む。

今後の基本形は **issueを出す -> 全部消す -> issueを出す** である。issueを出したまま次のissue discoveryへ進まない。

Burn-down中に新しい問題を見つけた場合:

- batch内issueの修正を妨げるblockerは、同じbatchへ追加してそのcycle内で消す。
- blockerでない周辺改善は、正式issue採番を次のDiscoveryまで待つ。必要なら `next discovery candidate` として短くメモする。
- 「issue数を増やすための追加探索」はBurn-down完了まで行わない。

## Phase A: CodeFire Development

Input:

- 最新のversion plan。
- open CFB/CFR issue。
- `docs/development/module-boundaries.md`。
- `docs/development/agent-code-quality.md`。

Work:

- root cause単位で修正する。issue単位の場当たり修正を避ける。
- `main.rs` へ責務を戻さず、parse、validate、plan、apply、renderを分ける。
- JSON contract、state reducer、diagnostics、layout definitionなど、複数issueを閉じる共通基盤を優先する。
- docs/spec/automation interface/history/todoを実装と同時に更新する。

Exit criteria:

- version planで対象にしたissueがfixedまたは明示的なwontfixになっている。
- `cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace` が通る。
- docs-onlyではない変更は、該当するtargeted testと全体testの両方を原則実行する。
- `docs/development/history.md` に結果と証跡を残す。

## Phase B: Install Next CodeFire

Input:

- Phase Aで通ったCodeFire build。

Work:

- `install.sh` で現在の開発版をinstallする。
- installed binary identity、version/help、completionが壊れていないか確認する。
- algorithm dogfood projectからinstalled `codefire` を呼んで、repo discoveryとJSON outputをsmokeする。

Expected commands:

```bash
cargo build --workspace
bash -n install.sh
./install.sh --prefix /home/devuser/.cargo
codefire --version
codefire --help
```

Exit criteria:

- installed `codefire` が新しいbinaryを指している。
- `codefire status --json`、`codefire scan --json`、`codefire verify --json` がdogfood projectで実行できる。

## Phase C: Algorithm Planning and Design

Input:

- algorithm projectの現状docs、todo、history、spec/design。
- 前回cycleで残ったalgorithm側の未完了task。
- CodeFire dogfoodingで特に試したいcommand surface。

Work:

- algorithm projectの次開発テーマを決める。
- 仕様、設計、実装タスク、テスト計画をdocs-firstで作る。
- CodeFireで追跡しやすいよう、REQ/DES/CODE/TEST AtomとTrace Linkを意識する。
- そのcycleで重点的に叩くCodeFire機能を決める。例: `context`, `evidence`, `doctor`, `storage`, `diff`, `branch`, `commit --dry-run`。

Exit criteria:

- algorithm側の計画、設計、todoが更新されている。
- `codefire scan` / `verify` で発生したfireの扱い方が明確になっている。

## Phase D: Algorithm Development and Dogfooding

Input:

- Phase Cのalgorithm設計。
- installed CodeFire。

Work:

- algorithm projectを設計に沿って実装する。
- CodeFireを通常運用として使う。
- 違和感や失敗はすぐにCFBとして記録する。記憶に頼って後でまとめない。
- issueは `bug-issue-template.md` に沿い、原則1 issue 1 fileで詳細を書く。

Dogfood checkpoints:

```bash
codefire status --json
codefire scan --json --metrics
codefire context --changed --json
codefire verify --details --blocking-only --json --metrics
codefire evidence add --dry-run --json ...
codefire doctor . --json
codefire migrate check . --json
codefire storage . --json
codefire diff main main --json
codefire commit -m "<message>" --dry-run --json
```

Exit criteria:

- algorithm側の設計対象が実装済み。
- algorithm側テストが通る。
- CodeFire commitまたはCodeFire管理上のclean stateまで到達している。
- dogfoodingで見つけたCFBをbatchとして確定している。
- batch確定後は、Burn-down完了まで追加探索へ進まない。

## Phase E: Code Review Issue Expansion

Input:

- dogfoodingで追加したCFB。
- 最新CodeFire source。

Work:

- CodeFire sourceをreviewし、追加で40件を目標に改善issueを出す。
- 重複issueは増やさず、既存issueへ証拠を追記する。
- 1 issue 1 fileを守る。
- issueは症状だけでなく、再現、影響、候補修正、acceptance criteriaまで書く。

Review focus:

- asymptotic complexity and memory use。
- object identity and hash invariants。
- branch/open state transition。
- JSON envelope compatibility。
- CLI option consistency。
- diagnostics severity/blocking semantics。
- evidence and artifact trust boundary。
- doctor/migrate/storage consistency。
- remote/idempotency/lock safety。
- module boundary and test placement。

Exit criteria:

- 追加40件目標を達成しているか、重複排除の結果として達成しない理由が明記されている。
- summary backlogとdetail filesが一致している。
- issue templateが使われている。
- 追加したCFRは現在batchに入れるか、次batch候補として明示的に分けている。

## Phase F: Next Version Planning and Design

Input:

- Phase D/Eで集めたCFB/CFR。
- 最新のCodeFire architecture。

Work:

- issueをroot causeごとに束ねる。
- 1つの修正で複数issueを閉じる設計を優先する。
- 次versionのscope、非scope、root fix map、phase plan、acceptance tests、dogfood verificationを文書化する。
- 実装前にmodule/file変更方針を決める。

Exit criteria:

- 次version planが `docs/development/v*-*.md` にある。
- 現在batch内の全issueが次version planまたはcycle burn-down plan内のどこかにmappingされている。
- 実装順序が、共通基盤、state/contract、command個別修正、docs/issue closeの順に整理されている。

## Issue Recording Rules

CFB:

- dogfoodingで発見したCodeFire bug、UX不具合、automation contract不整合。
- Summary table: `docs/development/bug-backlog.md`。
- Detail file: `docs/development/bug-issues/cfb-NNN.md`。
- Template: `docs/development/bug-issue-template.md`。

CFR:

- CodeFire source reviewで発見した改善issue。
- Detail file: `docs/development/code-review-issues/cfr-NNN.md`。
- Template: `docs/development/code-review-issue-template.md`。

Common rules:

- 既存issueと同じ根なら新規issueにせず、既存issueへEvidenceを追記する。
- 「不便」だけで終わらせず、再現、影響、workaround、候補修正、acceptance criteriaを書く。
- issue closeは実装、テスト、docs更新、履歴更新が揃ってから行う。

## Current Cycle State

Current named cycle:

- CodeFire次version系統: v1.0 Contract ValidationのPhase 1a以降を継続する。
- Current plan: `docs/development/v1.0-contract-validation-plan.md`。
- Current burn-down plan: `docs/development/current-cycle-burn-down-plan.md`。
- Current phase: cycle 36 dogfood pass完了。今回のalgorithm開発ではCodeFire本体の新規CFBは発見されず、既知CFB/CFR open issueは0件。
- Installed CodeFire: `/home/devuser/.local/bin/codefire`。
- Installed version output: `codefire foundation 1`。
- Git remote: `git@github.com:TA-Robot/CodeFire.git`。
- Latest pushed Git commits: current global burn-down commits through cycle 36 dogfood pass。
- Algorithm CodeFire state: `subprojects/algorithm-evolution-agent-lab/` は `open-clean`、open fires 0。
- Latest algorithm CodeFire base: `CF-COMMIT-03b681f9ef9c5844c3a1c639`。

Current global issue inventory:

| Series | Total tracked | Fixed / closed | Open |
|---|---:|---:|---:|
| CFB dogfood issues | 95 | 95 | 0 |
| CFR code review issues | 160 | 160 | 0 |
| Total | 255 | 255 | 0 |

Close audit note:

- 2026-06-10時点で残っていた146件は、過去の `BATCH-2026-06-global-open-backlog` の初期open数であり、現在のopen数ではない。同batchは同cycle内で全件fixedにした。
- 現在のCFB/CFR open issueは0件。`146` は履歴ログに出る開始件数であって、現残数ではない。
- 最後に残っていた `CFR-120` は、`docs/development/issue-root-cause-map.json` と `issue_root_cause_map_covers_open_issues` で、今後のopen issueをroot cause計画へ機械的に接続する形でcloseした。
- Cycle 4 dogfoodingでは `ResearchCyclePlanSynthesizer` を追加し、`CFB-093` を発見、同cycle内でfixedにした。
- Cycle 5 dogfoodingでは `ResearchCyclePlanMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-476c73ed58ac5e64be166e38` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 6 dogfoodingでは `ResearchCyclePlanLint` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-e9809fabb758032363e80407` でsealした。その後、非Atomのみ変更のstatus next_actions欠落として `CFB-094` を発見し、同cycle内でfixedにした。最終algorithm baseは履歴追記seal後の `CF-COMMIT-2de6ff5b70a1591e55786035`。
- Cycle 7 dogfoodingでは `ResearchCyclePlanLintMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-a1d8e9e852adbb15c0e1e532` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 8 dogfoodingでは `ResearchCyclePlanningPacketBuilder` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-7000b03b972c87a78bda0893` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 9 dogfoodingでは `ResearchCyclePlanningPacketMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-f16f1f0a0de14e661cbde9d0` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 10 dogfoodingでは `ResearchCyclePlanningPacketManifestBuilder` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-738987cf39675353564e60da` でsealした。その後、prediction上open fireがあるclean statusでnext_actionsが空になる `CFB-095` を発見し、同cycle内でfixedにした。test整形follow-upは `CF-COMMIT-a82560bcd3f29c8b06b7edf4` でsealした。並列mutating操作でlock contentionも観測したが、再実行で正常に進み、排他制御として期待動作と判断した。
- Cycle 11 dogfoodingでは `ResearchCyclePlanningPacketManifestMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-85c55142b38b449258674506` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 12 dogfoodingでは `ResearchCyclePlanningPacketManifestVerifier` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-3235c02fd7d24ef40b6cf9cd` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 13 dogfoodingでは `ResearchCyclePlanningPacketManifestVerificationMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-27ff328879a1d33f166509f1` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 14 dogfoodingでは `ResearchCyclePlanningHandoffBundleBuilder` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-a8a2133eebf4e2d25a6b5ff4` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 15 dogfoodingでは `ResearchCyclePlanningHandoffBundleMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-f28a264a7eb8ba11f09c6f69` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 16 dogfoodingでは `ResearchCyclePlanningHandoffBundleSummary` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-7e54b0cd74ec6147417adfae` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 17 dogfoodingでは `ResearchCyclePlanningHandoffReadinessGate` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-e339eafbdb954d1d3aa94199` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 18 dogfoodingでは `ResearchCyclePlanningHandoffReadinessMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-b6e595769abe0e97f721bcf6` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 19 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketBuilder` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-b1606907ae8c9c0ef9f8d86d` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 20 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-8bfb09c524a9502892cb6fcc` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 21 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactBuilder` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-5dd928c08fe5c0eeb673bb04` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 22 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactManifestBuilder` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-a301c378a9e91020c26669e1` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 23 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactManifestMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-bdf99515ee6fa2576e7b0acc` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 24 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactManifestVerifier` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-efcc72220a4aec9228629367` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 25 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactManifestVerificationMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-ec21ce3f2d608602068ca5b6` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 26 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveBuilder` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-11cd0738c4cc3e281c943b16` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 27 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummary` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-ef003ce1fbfaa856969d83ca` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 28 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-b0059f76d36e80505a0e7522` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 29 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdownGate` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-21c452eaaed817710d58b84f` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 30 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryMarkdownGateMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-50356d058653af136bbfe2fe` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 31 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactBuilder` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-f337eedec17e36c98916d3aa` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 32 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestBuilder` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-9042b8b63434edc218bd993b` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 33 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-7f483c5777bf8cb6da929246` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 34 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestVerifier` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-d8910ff757e08b1119b03d12` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 35 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactManifestVerificationMarkdown` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-b4d96c977703356c86890ec9` でsealした。CodeFire本体issueは新規発見なし。
- Cycle 36 dogfoodingでは `ResearchCyclePlanningHandoffReviewPacketArtifactArchiveSummaryArtifactArchiveBuilder` をdocs-firstで追加し、6件のCodeFire open fireを同cycle内でextinguishして `CF-COMMIT-03b681f9ef9c5844c3a1c639` でsealした。CodeFire本体issueは新規発見なし。
- 次のcycleでも、issue discovery後にこのmapまたは後継schemaへ全open issueを登録し、batchを全部消してから次のissue出しへ進む。

## Completed Two-Cycle Snapshot

直近の2周分の実行は、2026-06-07時点で完了している。

### Cycle 1

CodeFire側:

- Fixed CFB-091 by making Python explicit `cf-atom` spans stop at the marker-owned class/def/async def/decorated symbol block.
- Verified with `cargo test -p codefire-core`.
- Installed the updated CodeFire before dogfooding.

Algorithm dogfood側:

- Added `ResearchCycleRetrospective` docs-first.
- Covered trace from `REQ-AUTO-038` to `DES-AUTO-020`, `CODE-ResearchCycleRetrospective`, and `TEST-research-cycle-retrospective-recommends-policy-adjustments`.
- Verified algorithm test suite: 143 tests pass.
- Sealed algorithm work with CodeFire evidence `CF-EVIDENCE-d751bcd96366e2d8ac72c61f` and CodeFire commit `CF-COMMIT-c4e4c9adea4d17be6ddbaabd`.

Dogfoodで見つかったこと:

- CFB-092 was added after the extractor fix caused source-unchanged projects to produce many hash-change fires.
- This is not treated as a functional regression in the algorithm source. It is a CodeFire migration/rebaseline workflow gap.

Git結果:

- Pushed as `9b6dd07 Stabilize Python explicit atom spans` and `1d181bc Add research cycle retrospective`.

### Cycle 2

CodeFire側:

- Fixed CFB-086 by making extinguish JSON compute `has_evidence` from inline evidence or `evidence_refs`.
- Fixed CFB-088 by making evidence dry-run JSON return `created:false` and `evidence_id:null` instead of an empty-string ID.
- Added targeted tests for evidence JSON and evidence-ref extinguish behavior.
- Verified with:
  - `cargo fmt --check`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - targeted `evidence_add` tests
  - targeted `extinguish_evidence_ref_links_resolution_and_verify_detects_missing_ref` test
- Reinstalled CodeFire to `/home/devuser/.local/bin/codefire`.

Algorithm dogfood側:

- Added `RetrospectivePlanningSummary` docs-first.
- Covered trace from `REQ-AUTO-039` to `DES-AUTO-021`, `CODE-RetrospectivePlanningSummary`, and `TEST-retrospective-planning-summary-renders-markdown`.
- Verified algorithm test suite: 144 tests pass.
- Registered evidence `CF-EVIDENCE-4b6b2a0e93268addbc3371f9`.
- Sealed algorithm work through CodeFire commits:
  - `CF-COMMIT-4d8776fc22ae2f173890165e`
  - `CF-COMMIT-6558c5b7de3769164e36172a`

Dogfoodで確認したこと:

- `codefire evidence add --dry-run --json` now shows `created:false` and `evidence_id:null`.
- `codefire extinguish --evidence-ref ... --dry-run --json` now shows `has_evidence:true`.
- `codefire scan --path . --json --metrics` changed only the four intended new atoms for the planning summary work.
- Algorithm project ended at `open-clean` with open fires 0.

Git結果:

- Pushed as `6d8361d Fix evidence JSON contracts` and `8978dfc Add retrospective planning summary`.

## Completed Global Burn-down Snapshot

The 2026-06-09 global burn-down batch started with 146 open CFB/CFR issues and is now complete. The authoritative current count is `docs/development/current-cycle-burn-down-plan.md`, and `docs/development/issue-root-cause-map.json` now guards future open-issue root-cause mapping.

Global inventory as of 2026-06-10:

- CFB dogfood issues: 95 total, 95 fixed, 0 open.
- CFR code review issues: 160 total, 160 fixed/closed, 0 open.
- Combined tracked backlog: 255 total, 255 fixed/closed, 0 open.

Active batch closed in this burn-down:

- CFB-005: interactive/all-matching/batch-template extinguish flows reduce repeated rationale and evidence input.
- CFB-010: `status --json` reports read-only scan prediction counts.
- CFB-011: `extinguish --batch-template` generates a strict JSON batch wrapper from current open fires.
- CFB-018: batch-template and all-matching extinguish reduce repeated evidence entry across many related fires.
- CFB-020: extinguish text output includes source/target context and remaining open fire count.
- CFB-017/CFR-117: scan model and CLI outputs include non-Atom changed file metadata.
- CFR-092: unsupported branch subcommands return branch-specific JSON diagnostics and valid next_actions.
- CFR-111: invalid HTTP timeout env values fail explicitly instead of silently falling back.
- CFR-128: HTTP client response body limit rejects oversized Content-Length before reading.
- CFR-118: unknown CF-* object prefixes no longer trigger fallback subdir scans.
- CFR-121: sealed commit validation requires the resolution ledger root.
- CFR-081: uncaught `--json` failures now return the common command_result error envelope.
- CFR-082: command capability metadata is shared by help, completion, and capability JSON.
- CFR-083: read-only debug commands use `--path` and JSON envelopes.
- CFR-084: remote read commands expose branch/MR data through JSON envelopes.
- CFR-094: link help routing and JSON error contracts are parser-before-mutation and structured.
- CFR-095: `CliError::diagnostic()` maps error variants to typed automation diagnostics.
- CFR-150: metrics support is declared in the command capability registry and exposed by `capabilities --json`.
- CFB-007: performance metrics are available through `data.metrics` and capability metadata.
- CFB-008: large artifact payloads are represented by external artifact refs and storage summaries.
- CFR-113: upload dry-run top-level repo/next_actions are present in the command envelope.
- CFR-093: status JSON no longer swallows open context mismatch diagnostics.
- CFR-090: HTTP remote view temp object directories are unique per call and cleaned on drop.
- CFR-110: HTTP remote error responses preserve remote diagnostic kind, status, and exit code.
- CFR-116: VerificationCommand now carries timeout/output/env/allow_failure execution contract and local runner enforces it.
- CFR-122: sealed commit validation cross-checks certificate result/counts against the verification root payload.
- CFR-123: required roots now have payload shape validators and validator coverage tests.
- CFR-112: file remote upload dry-run validates existing remote layout and reports validation checks.
- CFR-129: HTTP request-merge dry-run resolves branch heads and fails missing branches instead of returning empty heads.
- CFB-009: new import workflow code lives in `import_workflow.rs`, keeping command-specific parsing/planning/apply logic out of `main.rs`.
- CFB-048: `codefire import` adopts existing non-empty directories with dry-run/apply/idempotency support.
- CFR-089: normal diff and patch export now use manifest metadata to avoid reading unchanged blob content.
- CFR-105: storage report JSON now exposes object scan metrics and quick/full JSON parse counts.
- CFR-119/CFR-130: commit now records a pending transaction marker during multi-object writes, successful commits clean it, and doctor reports interrupted commits.
- CFB-025: core policy parser accepts legacy `verification.required` command lists.
- CFB-026: context data exposes scan-compatible changed/open fire summary fields.
- CFB-045: diff JSON distinguishes requested options from actually included sections.
- CFB-037: failed evidence commands no longer create successful-looking evidence by default.
- CFB-032: doctor no longer marks auto-creatable layout gaps as blocking invalid repo state.
- CFB-033: doctor repairable layout warnings now point to migration plan review.
- CFB-034: doctor and migrate share the same layout severity vocabulary.
- CFB-035: migration planned actions are generated from the same layout registry as doctor findings.
- CFB-049: evidence command capture cwd resolution is explicit and returned in JSON.
- CFB-043: commit dry-run blockers return JSON commit result envelopes.
- CFB-051: commit dry-run changed atoms are bounded by default.
- CFB-053: scan JSON defaults to bounded counts/samples with explicit full mode.
- CFB-061: missing remote storage target is reported as invalid layout instead of silently disappearing.
- CFB-063: review-pack and patch export have JSON metadata automation surfaces.
- CFB-066: install reports the active `codefire` path and warns when PATH resolves a different binary than the installed target.
- CFB-067: commit JSON uses typed result data and non-null top-level repo.
- CFB-072: evidence dry-run plan reports resolved cwd and cwd source.
- CFB-077: manual fire identity uses scan-compatible digest IDs.
- CFB-078: manual fire cleanup without source changes returns to open-clean.
- CFB-027: scan/verify next_actions offer `extinguish --batch-template --json` for multiple open fires.
- CFB-054: context detects stale active scans and falls back to read-only preview data.
- CFB-082: storage quick mode reports inferred largest object types and type confidence.
- CFB-087: missing evidence-ref dry-run failures are returned as JSON envelopes.
- CFB-089: `atom-index` and `missing-links` handle shared `--path` / `--json` like normal commands.
- CFB-090: bounded context reports endpoint omission metadata for omitted trace link endpoints.
- CFB-092: extractor/hash schema changes are migration-aware and can be rebaselined.
- CFR-132: migrate target format defaults to `current` and reports supported targets.
- CFR-120: open issue root-cause mapping is enforced through `issue-root-cause-map.json` and `issue_root_cause_map_covers_open_issues`.
- CFB-093: status JSON next_actions now use scan prediction to recommend commit for pending changes with zero open fires.

Next cycle entry condition:

- Start new discovery only while known open CFB/CFR count is 0.
- Record the newly discovered batch with detail files, root-cause map entries, and a burn-down plan before implementation.
- Burn down the newly frozen batch to 0 before opening another discovery batch.
- CFR-133: repository layout is defined by a shared doctor/migrate registry.
- CFR-114: context compares active scan freshness against the current preview scan.
- CFR-137: branch context selector returns changed/open-fire atom context instead of an empty selection.
- CFR-124: manual fire apply reloads state after lock acquisition.
- CFR-145: fire dry-run plans are bounded by default.
- CFR-146: fire operation plan next_actions carry open_dir/repo path context.
- CFR-148: doctor reports missing active `state.json`.
- CFR-149: doctor reports active state/open fire invariant violations.
- CFR-156: migrate supports quick/full scan modes with skipped check metadata.
- CFR-158: manual fire semantic identity is independent of base commit.
- CFR-159: commit certificate result uses verification vocabulary with legacy compatibility.
- CFR-160: commit JSON envelope repo comes from execution context.

Important already-fixed issues from the latest two-cycle run:

- CFB-086: evidence-ref extinguish `has_evidence` contract.
- CFB-088: evidence dry-run `created` / nullable `evidence_id` contract.
- CFB-091: Python explicit atom span stability.

Next expected action:

- Known CFB/CFR open issueは0件のため、次は新しいalgorithm dogfood discoveryを開始できる。
- 新issueを見つけた場合はそのcycle batchへfreezeし、全件burn downしてから次のissue discoveryへ進む。

## Cycle Invariants

- CodeFire本体にAI agent運用機能を入れない。
- marketplaceを入れない。
- CodeFireは外部自動化ツールが使いやすいCLI/JSON契約を提供する。
- algorithm projectはdogfooding対象であり、CodeFire product codeと責務を混ぜない。
- issue数を増やすこと自体を目的にしない。重複せず、修正可能で、検証できるissueを増やす。
- 開発終了時には、issue、設計、実装、テスト、install、dogfood結果、次計画がつながっていること。
