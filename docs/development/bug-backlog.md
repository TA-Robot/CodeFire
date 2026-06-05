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
| CFB-013 | open | UX / Scan Output | clean状態の `scan` が `Changed atoms:` と `Open fires:` の空見出しだけを表示し、明示的に「なし」と言わない | `algorithm-evolution-agent-lab` で `CF-COMMIT-c28f651ba771` 直後に `status` / `verify` / `scan` を実行したとき | 正常にcleanなのか、表示欠落なのかを人間が推測する必要がある | `status` の `Open fires: 0` と `verify` 成功を合わせて判断する | 空の場合は `Changed atoms: none` / `Open fires: none` または `No changed atoms. No open fires.` を表示する |
| CFB-014 | open | UX / State Label | `scan` でopen firesが作成された直後でも `status` が `open-consistent` / `Open fires: 4` を表示する場合がある | `algorithm-evolution-agent-lab` で `ConfigurationCapture` / `RunIdentityLedger` 追加後に `scan` で4件fireを作成し、直後に `status` を実行したとき | `open-consistent` という状態名と `Open fires: 4` が矛盾して見え、消火が必要か判断しづらい | `Open fires` の件数と `scan` 出力を優先して判断する | open fire数が1件以上なら状態名を `open-burning` などに統一し、状態算出をfire indexから一貫させる |
| CFB-015 | open | UX / State Label | clean状態で `verify --details --blocking-only` を実行すると、直後の `status` が `open-clean` から `open-consistent` に変わる | `algorithm-evolution-agent-lab` で `CF-COMMIT-1cf6caf75deb` 後に `status` -> `verify --details --blocking-only` -> `status` -> `scan` -> `status` を実行したとき | worktreeに変更やfireがないのに状態名だけが変わり、clean判定がscan依存に見える | `scan` を実行して `open-clean` に戻るか確認する | `verify` がclean stateを劣化させないようにする、または `open-clean` / `open-consistent` の意味を明確化し状態遷移を安定させる |
| CFB-016 | open | UX / State Label | `scan` でopen firesが作成された直後でも `status` が `open-clean` / `Open fires: 10` を表示する場合がある | `algorithm-evolution-agent-lab` で `LearningCurveAnalyzer` / `CandidatePromotionPolicy` 追加後に `scan` で10件fireを作成し、直後に `status` を実行したとき | `open-clean` は作業不要に見えるが、実際にはfire消火が必要で、状態名と件数の矛盾が非常に強い | `Open fires` の件数と `scan` 出力を優先して判断する | open fire数が1件以上なら `open-clean` を絶対に表示しない不変条件を追加する |
| CFB-017 | open | UX / Non-Atom Changes | README、todo、historyなど非Atomドキュメント変更が `scan` の `Changed atoms` に出ず、fire/evidence対象にもならない | `algorithm-evolution-agent-lab` で各機能追加時に `README.md`、`docs/development/todo-checklist.md`、`docs/development/history.md` を更新したが、`scan` はCODE/REQ/TEST Atomだけを表示したとき | ドキュメント変更が封印対象に含まれるのか、証跡不要な変更として扱われるのかが分かりにくく、docs-only変更のレビュー漏れにつながる | 人間が差分を別途確認する | `scan` に `Non-atom changed files` セクションを追加し、必要ならpolicyでfire化/警告化できるようにする |
| CFB-018 | open | UX / Fire Volume | 新規REQを追加すると、REQ->DES、REQ->CODE、REQ->TESTとCODE/TEST->REQの複数fireが同時に発生し、同じ証跡で何度も消火する必要がある | `algorithm-evolution-agent-lab` で `REQ-AUTO-021/022/023/024` や `REQ-SOTA-013/014` を追加したとき、2要求ごとに10件前後のfireが発生した | 実質的に1つの仕様追加レビューなのにedge単位で消火が増幅し、AI/人間の操作量が増える | shell loopで同じevidenceを流す | `fire group` / `trace change set` を導入し、同一REQ追加に伴うedge群をまとめて解消できるようにする |
| CFB-019 | open | UX / Fire Identity | `FIRE-001` などのfire IDがscanごとに再利用され、ログや会話で過去のfire参照と衝突しやすい | `algorithm-evolution-agent-lab` で複数の機能追加を繰り返し、毎回 `FIRE-001..010` のようなIDが再利用されたとき | サブエージェントや人間が「FIRE-001」を参照しても、どのscan世代のfireか曖昧になる | 直近のscan出力を同じ文脈に保持する | fire IDにscan epochやatom/link digestを含める、または `FIRE-001@scan-id` のような安定参照を表示する |
| CFB-020 | open | UX / Extinguish Output | `extinguish` 成功時の出力が `extinguished FIRE-001` だけで、対象AtomやTrace Linkを再表示しない | `algorithm-evolution-agent-lab` で10件単位のfireをshell loopで消火したとき | 消火ログだけを後で見ても、どの要求・実装・テストを解消したか追跡しづらい | 直前の `scan` 出力と照合する | 成功出力に `FIRE-001 CODE-X -> REQ-Y atom_changed` とevidence要約を表示する |
| CFB-021 | open | UX / Commit Summary | `codefire commit` の出力がcommit hash、branch、stateだけで、封印したAtom、fire解消数、証跡数を要約しない | `algorithm-evolution-agent-lab` で `CF-COMMIT-e38475672bec`、`CF-COMMIT-9872184cce2f`、`CF-COMMIT-f825fc13207f` などを作成したとき | 後からログを見ても、そのcommitが何を含んだのかCodeFire出力だけでは分からない | commit messageと直前のscan/verifyログを合わせて読む | commit出力に changed atom count、resolved fire count、non-atom changed files count、messageを表示する |
| CFB-022 | open | UX / Verify Output | `verify --details --blocking-only` 成功時の出力が `Verification passed.` だけで、open fire数、stale resolution数、base commitを示さない | `algorithm-evolution-agent-lab` で各消火後にblocking verifyを実行したとき | CIログやサブエージェントログ上で、何が0件だったのか確認しづらく、別途 `status` を実行する必要がある | `verify` 後に `status` を実行する | 成功時も `Open fires: 0`, `Stale resolutions: 0`, `Base: ...`, `Blocking checks: 0` を表示する |

## Triage Notes

- CFB-001は `test_stale_resolution_blocks_verify` でrefresh導線を確認する。
- CFB-002は `test_python_methods_include_class_owner_in_derived_atom_id` で同名methodの派生ID衝突回避を確認する。
- CFB-003は既存verify失敗系テストで `Blocking checks:` を確認する。
- CFB-004は `test_verify_details_reports_missing_required_links` とduplicate Atom ID詳細表示で確認する。
- CFB-005からCFB-008は、`algorithm-evolution-agent-lab` dogfoodingでの使いやすさ、速度、記憶領域観察から登録した改善issue。
- CFB-009は、v0.6以降のRust本流で単一ファイル肥大化を防ぐための開発プロセス改善issue。
- CFB-010とCFB-011は、v0.6 Rust defaultをインストールした後の `algorithm-evolution-agent-lab` Phase 4作業で再確認したdogfooding issue。
- CFB-012は、連続して新Atomを追加し、各fireを消火してからcommitする運用で再現した。`open-burning` は内部的には「未sealed変更あり」を含む可能性があるが、fire数0のときは人間には未消火に見えやすい。
- CFB-013は、clean branchの確認時に再現した。成功状態の出力ほど明示的なempty stateを持つ方が、サブエージェント運用時のログ解釈も安定する。
- CFB-014は、CFB-012とは逆向きの状態ラベル不整合として記録した。fire数が正ならラベルも未消火状態を示すべきである。
- CFB-015は、`verify` 自体が成功しているにもかかわらず状態名が変化する問題として記録した。`scan` 後には `open-clean` に戻るため、状態算出の入力がコマンドごとに揺れている可能性がある。
- CFB-016は、CFB-014より強い矛盾として記録した。`open-clean` とopen fire数正の組み合わせは、状態名の意味を壊すためblocking相当のUX不具合として扱う。
- CFB-017は、Atom中心モデル自体は維持しつつ、Atom外の実ファイル差分も人間が見落とさないようにするためのdogfooding改善issueである。
- CFB-018からCFB-022は、同じ操作を繰り返したことで見えたAI運用向けのログ・ID・集約性改善issueである。特にfire grouping、安定ID、成功時summaryはサブエージェントにCodeFireを使わせる場合の誤操作を減らす。
- CFB-005の具体例として、同一evidenceで多数fireを解消する場合は `verify` からbatch templateを生成できると操作量が大きく減る。
- CFB-006の具体例として、`require_trace_completeness: false` でもmissing link件数が表示されるため、`--blocking-only` とnon-blocking診断の優先度分離は引き続き重要である。
