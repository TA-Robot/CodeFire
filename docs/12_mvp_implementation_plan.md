# 12. MVP実装計画

## 12.1 MVPの目的

MVPの目的は、次の中核価値を証明することである。

> コードや仕様を変更すると関連成果物へfireが立ち、fireが残っている状態ではcommitできず、すべてextinguishしてverifyに通った場合だけsealed commitを作れる。

## 12.2 MVP対象

```text
Local repository
Local branch
open / close / clone
Markdown仕様書
Markdown設計書
Pythonコード
pytestテスト
Trace Link YAML
Fire ledger
Extinguish
Verify
Commit
Merge basic
```

## 12.3 MVP対象外

```text
Remote server
Merge request
AI連携
多言語対応
OpenAPI
DB schema
高度なsemantic merge
権限管理
Review workflow
warning fire
GUI
```

## 12.4 Phase 1: Object store

作るもの：

```text
object_id計算
blob store
manifest object
commit object
branch object
init
```

成功条件：

```text
repositoryをinitできる
空のmain branchを作れる
manifestを保存できる
commit objectを保存できる
branch headを更新できる
```

## 12.5 Phase 2: open / close / clone

作るもの：

```text
open
close
close --discard
clone
open registry
.codefire-open
copied directory検出
```

成功条件：

```text
mainをopenできる
mainからfeatureをcloneできる
featureをopenできる
同一branch多重openを拒否できる
closeで実ディレクトリを消せる
```

## 12.6 Phase 3: Indexer

作るもの：

```text
codefire.yaml parser
Markdown Atom抽出
Python Atom抽出
pytest Atom抽出
AtomIndex生成
duplicate atom id検出
```

成功条件：

```text
REQ/DES/CODE/TESTを抽出できる
Atom hashを計算できる
変更Atomを検出できる
```

## 12.7 Phase 4: Trace Graph

作るもの：

```text
codefire.links.yaml parser
TraceGraphObject
link参照検証
required link policy
```

成功条件：

```text
REQ -> DES -> CODE
REQ -> TEST
の関係を読める
link欠落でcommit拒否できる
```

## 12.8 Phase 5: Fire Engine

作るもの：

```text
scan
changed atom detection
fire propagation
active fires
manual fire
obsolete fire
status表示
```

成功条件：

```text
コード変更で設計・仕様・テストにfireが立つ
fireが残るとcommit不可
revertするとauto fireがobsoleteになる
manual fireは残る
```

## 12.9 Phase 6: Extinguish / stale

作るもの：

```text
extinguish
resolution basis
no-change-required rationale必須
stale resolution判定
```

成功条件：

```text
fireをextinguishできる
理由なしno-change-requiredは拒否
extinguish後に対象Atomを変えるとstaleになる
staleがあるとcommit不可
```

## 12.10 Phase 7: Verify / Commit

作るもの：

```text
verify
external command runner
commit sealer
certificate
branch head更新
active state reset
```

成功条件：

```text
open fireなし
staleなし
link欠落なし
pytest成功
の場合だけcommitできる
commit後branch headが進む
open stateがopen-cleanになる
```

## 12.11 Phase 8: Merge

作るもの：

```text
common ancestor探索
file-level 3-way merge
atom-level diff
merge fire
merge commit parents
```

成功条件：

```text
featureをmainにmergeできる
merge後mainがopen-burningになる
fire解消後にmerge commitできる
merge commit parentsが2つになる
```

## 12.12 MVPデモシナリオ

```text
1. codefire init
2. mainをopen
3. サンプル仕様・設計・コード・テストを書く
4. linkを定義する
5. verifyしてcommit
6. mainからfeatureをclone
7. featureをopen
8. コードを30分から15分へ変更
9. scanでREQ/DES/TESTにfireが立つ
10. commitが拒否される
11. 仕様・設計・テストを更新する
12. fireをextinguishする
13. verifyする
14. commitできる
```

