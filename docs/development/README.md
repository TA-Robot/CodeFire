# CodeFire 開発ガイド

| 知りたいこと | 文書 |
|---|---|
| いま何をするか | [PLAN.md](PLAN.md) |
| 開いている課題 | [issues.md](issues.md) |
| 何が起きたか | [history.md](history.md) |
| Rustコードの構成 | [source-architecture.md](source-architecture.md)、[module-boundaries.md](module-boundaries.md) |
| commandごとの処理経路 | [command-source-trace.md](command-source-trace.md) |
| 型・object・永続化形式 | [source-data-model-catalog.md](source-data-model-catalog.md) |
| 変更種別ごとの手順 | [change-playbook.md](change-playbook.md) |
| 既知の制約 | [known-limitations.md](known-limitations.md) |

検証は `scripts/check.sh`。command・module・型・永続化境界を変えたら
`python3 docs/development/tools/check_source_docs.py` も実行する。
