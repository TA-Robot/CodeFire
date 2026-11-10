# AGENTS.md（CodeFire本体開発用）

## まず読む

1. `docs/development/PLAN.md`: ゴール、いまの段階、凍結している範囲。
2. `docs/development/issues.md`: 開いている課題。
3. 触る領域に応じて `docs/development/README.md` の表から設計文書を選ぶ。

製品の概念は `README.md` と `docs/00`–`docs/11`。

## ルール

- 作業は `PLAN.md` の現在の段階に効くものだけ。凍結範囲は、利用で必要になるまで改修しない。
- 不具合は直す前に再現する。直したら、その条件を固定するテストを足す。
- 引き渡し前に `scripts/check.sh` を通す。失敗を範囲外として扱わない。テストを緩めない。
- parse、validate、plan、apply、render を混ぜない。`main.rs` や巨大ファイルへ新しい責務を足さず、moduleを分ける。
- object identity、sealed commit、branch head の原子性、remote mutation policy の不変条件を壊さない。
- command・module・型・永続化境界を変えたら、同じ変更で設計文書を更新し `python3 docs/development/tools/check_source_docs.py` を通す。
- 計算量とメモリ使用量を軽く扱わない。
- AIエージェント運用機能を製品本体へ入れない。
- `subprojects/` は適用先の架空プロジェクト。CodeFire本体の実装と混ぜない。
- 状態や予定を書く場所は `PLAN.md`・`issues.md`・`history.md` だけ。新しい管理文書を増やさない。
- 他人の未コミット変更を戻さない。`git reset --hard` や `git checkout --` で破壊的に戻さない。
- モデルと課金経路は開発コンテナ側の `AGENTS.md` に従う。このリポジトリには書かない。

## 確認コマンド

```bash
scripts/check.sh                                      # 整形、lint、全Rustテスト
python3 docs/development/tools/check_source_docs.py   # 設計文書とソースの照合
```

Rustは `rust-toolchain.toml` の版を使う。
