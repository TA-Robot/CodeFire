# CodeFire 開発計画

Updated: 2026-11-10

現行の計画はこの1枚だけ。ほかの文書に「いま何をするか」を書かない。

## ゴール

開発者本人とAIエージェントが、手元のプロジェクトで毎日使えるローカルCLIにする。
外部公開、他組織との連携、標準化は、手元で使い物になってから別途判断する。

## 進め方

1. 架空プロジェクト（`subprojects/algorithm-evolution-agent-lab`）にCodeFireを適用して開発を回す。
2. 詰まった所・誤った挙動を `issues.md` に起こす。
3. 上から直し、`scripts/check.sh` を通し、もう一度回す。

文書だけのコミットが続いたら止めて、コードか利用に戻る。

## 段階

| # | 段階 | 終わりの条件 | 状態 |
|---|---|---|---|
| 1 | 土台を固定 | `scripts/check.sh` 1本で整形・lint・全テストが再現できる | 完了（2026-11-10） |
| 2 | 管理を作り直す | 開発管理の文書が本書・`issues.md`・`history.md`・ソース設計文書だけになる | 完了（2026-11-10） |
| 3 | 最初の10分を通す | 初期化→変更→Fire検出→解消→検証→commit をRust CLIで通す自動テストがあり、CLIの出力だけでcommitまで到達できる（CFB-140） | 完了（2026-11-10） |
| 4 | 架空プロジェクトで回す | Algorithm Lab をRust CLIへ取り込み直し、エージェントの変更を3回以上、手助けなしでsealできる | 完了（2026-11-10、4周）。以後も同じやり方で回し続ける |
| 5 | 触った所から分割 | 段階4で改修した領域のファイルが3,000行以下 | 完了（2026-11-10）。`codefire-cli/src/main.rs` 12,866 → 2,334行、`codefire-core/src/lib.rs` 6,092 → 2,480行 |

## これから

段階1–5は完了した。以後は「進め方」の繰り返しだけを続ける。

- Algorithm Lab での試用を続ける（release build: `cargo build --release --locked -p codefire-cli` → `target/release/codefire-rs`）。新しい環境では `.codefire/` がないので `import` からやり直す。
- 試用で詰まった所を `issues.md` に起こし、上から直す。
- 3,000行を超えるファイルのうち凍結範囲外のもの（`codefire-store/src/lib.rs` 約20,500行、`codefire-cli/src/tests.rs` 約13,100行）は、改修で触るときに分割する。

## 凍結している範囲

消さない。テストは通し続ける。新規の改修と不具合修正の対象にしない。
解凍するのは、段階4の利用で必要になったときだけ。

| 範囲 | 場所 |
|---|---|
| ネットワーク越しのサーバーと通信（`serve`、HTTP/HTTPS、request署名） | `http.rs`、`http_tls.rs`、`signatures.rs` |
| 旧形式からの移行（`migrate`） | `migration.rs` |
| patch export/import、review-pack | `import_workflow.rs` ほか |
| Python版（reference/fallback） | `codefire`、`tests/`、`demo.sh` のPython経路 |

フォルダを置き場にするremote（`cf://` の upload / request-merge / request-apply）は凍結しない。

## やらないこと

- エコシステム連携、複数組織での実証、互換リリース周期、資金付き保守体制（旧M1–M5）。
- 読み取り専用の分類packet、定時監査、凍結した判定表による進行管理。
- 製品本体へのAIエージェント運用機能の追加。

## 過去の資料

2026-11-10 より前の管理文書（milestone board、task packet、exploration、review portfolio、
完了済みissueの詳細）は tag `archive/pre-rebuild-20261110` にある。
未統合の枝（`recovery/*`、`snapshot/*`、`archive/python-http-picks-*`）は退避のみで、精査せずにmergeしない。
