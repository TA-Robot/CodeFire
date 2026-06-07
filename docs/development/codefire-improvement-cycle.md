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

1. CodeFireの次versionを設計に沿って開発する。
2. 開発したCodeFireをinstallし、実際の `codefire` commandとして使える状態にする。
3. Algorithm dogfood projectの次の開発計画と設計を作る。
4. その設計に基づいてalgorithm開発を進める。
5. algorithm開発中にCodeFireを使い、バグ、UX違和感、JSON contract不整合、速度、記憶領域、diagnostic不足を `CFB-*` として記録する。
6. algorithm開発終了時に、dogfoodingで十分なissueが見つかったか確認する。目標は新規20件。
7. 20件に届かない場合は、algorithmの次の開発計画と設計からもう一度進める。
8. dogfooding issueが十分集まったら、CodeFireソースコードレビューを行い、追加で40件を目標にissueを洗い出す。
9. dogfooding issueとcode review issueを重複排除し、全issueを解決する次versionの開発計画と設計を作る。
10. 次versionのCodeFire開発へ戻る。

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
- dogfoodingで新規CFBが20件以上見つかった、または20件未満ならPhase Cへ戻る判断が明記されている。

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
- 全open issueが次version plan内のどこかにmappingされている。
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

- CodeFire next version: v0.8 CFB burn-down。
- Current plan: `docs/development/v0.8-cfb-burn-down-plan.md`。
- Current phase: Phase D。v0.8途中版をinstallし、`algorithm-evolution-agent-lab` をCodeFire管理へ再取り込み、docs-first実装でdogfooding中。
- Latest algorithm work: Experiment Triage BoardとExperiment Iteration Plannerを要求、設計、Trace Link、実装、テスト、CodeFire evidence、CodeFire commitまで通した。
- Latest dogfood findings: CFB-046..CFB-056を追加。新規11件であり、目標20件には未達のため、次はPhase Cへ戻ってalgorithm側の次設計を立てる。
- Next expected action: algorithm側で次のdocs-first開発テーマを決め、`context`、`doctor`、`migrate`、`diff`、`commit --dry-run`、large JSON surfaceを重点的に叩く。

## Cycle Invariants

- CodeFire本体にAI agent運用機能を入れない。
- marketplaceを入れない。
- CodeFireは外部自動化ツールが使いやすいCLI/JSON契約を提供する。
- algorithm projectはdogfooding対象であり、CodeFire product codeと責務を混ぜない。
- issue数を増やすこと自体を目的にしない。重複せず、修正可能で、検証できるissueを増やす。
- 開発終了時には、issue、設計、実装、テスト、install、dogfood結果、次計画がつながっていること。
