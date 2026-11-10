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
| 4 | 架空プロジェクトで回す | Algorithm Lab をRust CLIへ取り込み直し、エージェントの変更を3回以上、手助けなしでsealできる | 次 |
| 5 | 触った所から分割 | 段階4で改修した領域のファイルが3,000行以下 | 未着手 |

段階4の作業:

- 済: `subprojects/algorithm-evolution-agent-lab` を `import` でRust CLIの管理下に入れた（`.codefire/` はgit管理外。新しい環境では `import` からやり直す）。
- エージェントに機能追加・仕様変更をさせ、CodeFireを通してsealさせる。詰まった所を `issues.md` に起こして直す。
- 試用には release build（`cargo build --release --locked -p codefire-cli` → `target/release/codefire-rs`）を使う。Algorithm Lab での実測: status 0.3秒 / scan 0.5秒 / verify 2.7秒（debug build は 2.5秒 / 3.5秒 / 10秒）。

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
