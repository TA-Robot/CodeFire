# 11. 内部アーキテクチャ設計

## 11.1 レイヤ構成

```text
CLI Layer
  ユーザコマンドを受ける

Command Service Layer
  open, close, clone, scan, fire, extinguish, verify, commit, mergeを実行する

Repository Store
  sealed object, branch, policy, commit graphを保存する

Open Directory Manager
  branchを実ディレクトリへ展開し、close/discardする

Indexer
  ファイルからArtifact / Atomを抽出する

Trace Graph Engine
  Atom間の関係グラフを管理する

Fire Engine
  変更に基づいてfireを生成・伝播・重複排除する

Policy Engine
  commit可能性、link欠落、fire未解消、stale resolutionを判定する

Verifier
  テスト、静的解析、生成物鮮度、trace completenessを検証する

Commit Sealer
  consistent状態をsealed commitとしてobject storeへ保存する

Merge Engine
  sealed branch同士を統合し、target open directoryをburningにする

Remote Client / Server
  clone, upload, merge requestを扱う
```

## 11.2 Rust module構成案

```text
crates/
  codefire-cli/
    src/main.rs

  cf-core/
    src/types.rs
    src/errors.rs

  cf-store/
    src/object_store.rs
    src/commit_store.rs
    src/branch_store.rs
    src/hash.rs
    src/canonical.rs

  cf-workspace/
    src/open.rs
    src/close.rs
    src/materialize.rs
    src/discard.rs
    src/identity.rs

  cf-index/
    src/indexer.rs
    src/markdown.rs
    src/python.rs
    src/pytest.rs
    src/atom_hash.rs

  cf-trace/
    src/graph.rs
    src/links.rs
    src/policy.rs

  cf-fire/
    src/fire_engine.rs
    src/propagation.rs
    src/resolution.rs
    src/stale.rs

  cf-policy/
    src/commit_policy.rs
    src/link_policy.rs
    src/extinguish_policy.rs

  cf-verify/
    src/verifier.rs
    src/external_command.rs
    src/internal_checks.rs

  cf-commit/
    src/sealer.rs
    src/certificate.rs

  cf-merge/
    src/merge_engine.rs
    src/file_merge.rs
    src/semantic_fire.rs

  cf-remote/
    src/client.rs
    src/protocol.rs
    src/upload.rs
    src/clone.rs
    src/merge_request.rs
```

## 11.3 scan処理

```text
1. open context確認
2. codefire.yaml読み込み
3. indexer実行
4. current AtomIndex生成
5. base AtomIndexと比較
6. changed_atoms算出
7. TraceGraph読み込み
8. FireEngine実行
9. active fires更新
10. obsolete fire整理
11. status更新
```

## 11.4 commit処理

```text
1. repo lock
2. branch lock
3. open identity確認
4. copied directoryでないことを確認
5. scan
6. verify
7. committable判定
8. blob保存
9. manifest保存
10. atom index保存
11. trace graph保存
12. fire delta保存
13. verification保存
14. commit object作成
15. branch head更新
16. active state reset
17. open registry更新
```

## 11.5 lock設計

```text
repo.lock
  repository全体の構造変更

branch-<name>.lock
  branch head/open state変更

object-store.lock
  object書き込み

active-<open_instance_id>.lock
  active state変更
```

branch head更新はatomic renameで行う。

## 11.6 recovery

クラッシュ対応として `codefire doctor` を用意する。

```text
orphan tmp削除
object hash再検証
branch head参照検証
opened registryと.codefire-openの整合確認
active state破損確認
cache再構築
```

cacheは正式データではないため、壊れたら再構築する。

## 11.7 重要な実装判断

未commitファイル内容をrepositoryに保存しない。

scan時に作業中ファイルをblob化すると、実質checkpointになる。したがって、BlobObject保存はcommit sealing時だけ行う。

active stateにはfire/resolution/verification metadataを保存してよいが、それだけでは作業中ファイルを復元できないようにする。

