# CodeFire Bug Backlog

このファイルは、CodeFire自身を実プロジェクト管理に使いながら見つかった不具合、バグに近いUX、修正候補を管理する。

## Status

- `open`: 未修正
- `investigating`: 再現条件や修正方針を確認中
- `fixed`: 修正済み
- `wontfix`: 現時点では仕様として扱う

## Bugs

| ID | Status | Area | Summary | Observed while | Impact | Workaround | Candidate fix |
|---|---|---|---|---|---|---|---|
| CFB-001 | fixed | Extinguish / stale resolution | extinguished済みfireのresolutionがstaleになった後、通常CLIで同じfireを再extinguishできない | `algorithm-evolution-agent-lab` で `StaticPlanner` を別moduleへ移動した後の `codefire verify` | `Open fires: 0` でも `Stale resolutions: N` でcommit不能になり、通常操作だけでは復旧しづらい | `codefire extinguish <fire> --refresh ...` で現在のbasisへ更新する | `--refresh` で既存resolutionを `superseded` にし、新しいresolution basisを保存する |
| CFB-002 | fixed | Python Atom extraction | Python methodの派生Atom IDがownerなしの `CODE:path::def:<method>` になり、異なるclassの同名methodでduplicate Atom IDになる | `CodexPlanner.propose` と `StaticPlanner.propose` を同一fileに置いたとき | 正当なPython設計でもduplicate Atom IDでverify/commitが止まる | 明示 `# cf-atom:` でも回避可能 | Python extractorでclass ownerを追跡し、method派生IDを `method:Class.method` にした |
| CFB-003 | fixed | Diagnostics | `verify` は `Missing required links` の件数を表示するが、`commit_policy.require_trace_completeness: false` の場合でも失敗原因に見えやすい | 仕様先行フェーズでtrace completenessを緩和した状態 | 実際のblockerがstale/duplicateでも、missing linkが主原因に見える | `Blocking checks:` 行を見る | verify出力にpolicy上のblockerだけを列挙する `Blocking checks:` を追加した |
| CFB-004 | fixed | Diagnostics | `verify` の失敗件数から対象Atomやstale resolutionを直接確認できない | duplicate Atom IDやmissing linkの原因を調査するとき | active JSONやscan結果を読まないと次の操作対象を特定しづらい | `.codefire-open/state/active_verify.json` を読む | `codefire verify --details` で失敗diagnosticの代表例を表示する |
| CFB-005 | open | UX / Extinguish | fire解消時の `--rationale` / `--evidence` 入力が長く、dogfooding中の人間操作コストが高い | `algorithm-evolution-agent-lab` で複数fireを解消しながらCodeFire commitしたとき | CodeFire処理時間より説明文入力の負担が支配的になり、連続作業の速度を落とす | shell historyや手書きテンプレートを使う | `codefire extinguish --interactive`、複数fire一括解消、直近テスト結果の自動evidence添付を検討する |
| CFB-006 | open | UX / Trace completeness | 仕様先行フェーズでmissing required linksが多く、実際のblocker確認時にノイズになる | `algorithm-evolution-agent-lab` で `require_trace_completeness: false` の状態で `verify --details` を使ったとき | open fireは0でもmissing link詳細が大量に出て、次に見るべき問題の優先度が分かりにくい | `Blocking checks:` を見る | `verify --details --blocking-only` またはnon-blocking diagnosticsの折りたたみ/優先度表示を追加する |
| CFB-007 | open | Performance / Observability | `status` / `scan` / `verify` の速度は現規模では軽いが、継続的に測るCLIがない | `algorithm-evolution-agent-lab` 現規模で `status` 約0.4s、`scan` 約0.45s、`verify --details` 約1.2sを手動計測したとき | repo拡大時に遅くなっても、どの処理がボトルネックか追跡しづらい | shellの `time` で手動測定する | `codefire doctor --metrics` または `codefire perf` でAtom数、object数、各phase時間を出す |
| CFB-008 | open | Storage / Artifacts | sealed object storeは小規模では軽いが、ML実験artifactを入れると肥大化し得る | `algorithm-evolution-agent-lab` で `.codefire` 約2.1MB、objects約2.0MB、198 filesを確認したとき | 大きいmetrics/log/model artifactをCodeFire objectに直接入れる運用だとrepositoryが急増する | 重いartifactは外部パスや要約メタデータだけ管理する | artifact retention policy、object store size report、large artifact warning、external artifact reference仕様を追加する |
| CFB-009 | open | Architecture / Maintainability | Rust CLIの機能追加時に `main.rs` へ責務が集まりやすく、module境界が曖昧になり得る | v0.6 Rust rewriteのinstaller/completion作業前レビュー | command dispatch、domain logic、rendering、policyが近接すると不変条件の所在が読みづらくなる | 新規作業前に `module-boundaries.md` を確認し、必要なら先に抽出commitを作る | `main.rs` のdispatch薄型化を継続し、commandごとのplanning/renderingを専用moduleへ分離する |
| CFB-010 | open | UX / Status vs Scan | `status` が `open-consistent` でも、直後の `scan` で多数の `atom_changed` fireが発生する可能性を事前に示さない | `algorithm-evolution-agent-lab` で `status` は `open-consistent` / open fires 0 だったが、`scan` 後に33件のfireが出た | 人間には「今commit可能そう」に見え、scan後に大量fire処理へ進むため作業見積もりがずれる | commit前に必ず `scan` を走らせる | `status --staleness`、`status --predict-scan`、または `status` に `last_scan_base/current_worktree_dirty_atoms` の要約を追加する |
| CFB-011 | open | UX / Extinguish Batch | `verify --details` が出したfire一覧から `extinguish --batch` 用テンプレートを直接生成できない | `algorithm-evolution-agent-lab` で33件のfireを同じテスト証跡で解消したとき | shell loopで `FIRE-001..033` を組み立てる必要があり、ID範囲ミスや証跡入力漏れが起きやすい | shell loopまたは手書きbatch fileを使う | `verify --json` のopen fire一覧から `codefire extinguish --batch-template --evidence-from-command <cmd>` を生成する |
| CFB-012 | open | UX / State Label | 全fireをextinguishし `verify --details --blocking-only` が通った後でも、commit前の `status` が `open-burning` / `Open fires: 0` を表示する | `algorithm-evolution-agent-lab` で `MetricNormalizer`、`LeakageChecker`、`NoveltyReviewer` などを消火・verifyした直後 | blockingなしでcommit可能な状態なのに「burning」と読めるため、まだ未処理fireが残っているように見える | `Open fires: 0` と `verify --details --blocking-only` を優先して判断する | fire数0かつverify blockingなしの場合は `open-consistent` など非burning状態名にする、または `pending commit after extinguish` のような別状態を表示する |

## Triage Notes

- CFB-001は `test_stale_resolution_blocks_verify` でrefresh導線を確認する。
- CFB-002は `test_python_methods_include_class_owner_in_derived_atom_id` で同名methodの派生ID衝突回避を確認する。
- CFB-003は既存verify失敗系テストで `Blocking checks:` を確認する。
- CFB-004は `test_verify_details_reports_missing_required_links` とduplicate Atom ID詳細表示で確認する。
- CFB-005からCFB-008は、`algorithm-evolution-agent-lab` dogfoodingでの使いやすさ、速度、記憶領域観察から登録した改善issue。
- CFB-009は、v0.6以降のRust本流で単一ファイル肥大化を防ぐための開発プロセス改善issue。
- CFB-010とCFB-011は、v0.6 Rust defaultをインストールした後の `algorithm-evolution-agent-lab` Phase 4作業で再確認したdogfooding issue。
- CFB-012は、連続して新Atomを追加し、各fireを消火してからcommitする運用で再現した。`open-burning` は内部的には「未sealed変更あり」を含む可能性があるが、fire数0のときは人間には未消火に見えやすい。
- CFB-005の具体例として、同一evidenceで多数fireを解消する場合は `verify` からbatch templateを生成できると操作量が大きく減る。
- CFB-006の具体例として、`require_trace_completeness: false` でもmissing link件数が表示されるため、`--blocking-only` とnon-blocking診断の優先度分離は引き続き重要である。
