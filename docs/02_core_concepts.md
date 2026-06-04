# 02. 中核概念

## 2.1 Repository

Repositoryは、Code Fireが管理するプロジェクト全体の保管庫である。

Repositoryは以下を持つ。

```text
sealed commit群
branch群
object store
policy
open directory管理情報
server接続情報
cache
```

Repositoryにcurrent branchは存在しない。

## 2.2 Branch

Code Fireのbranchは、Git branchではない。

Code Fire branchは、整合済みcommitだけからなる開発線である。

```text
C1 consistent -> C2 consistent -> C3 consistent
```

branch headは常にsealed commitを指す。

## 2.3 Open Directory

branchは、`open` すると実ディレクトリとして展開される。

```bash
codefire open main ./main
codefire open feature-login ./feature-login
```

これにより、複数branchを同時に開ける。

```text
project/
  .codefire/
  main/
  feature-login/
```

Gitのcheckoutのように、1つの作業ディレクトリを切り替えることはしない。

## 2.4 Close

`close` はopenされたbranch directoryを削除する。

```bash
codefire close feature-login
```

cleanでないbranchは通常closeできない。

```bash
codefire close feature-login --discard
```

を使うと、未commit作業を破棄してcloseする。

## 2.5 Clone

Code Fireのcloneは、sealed branch headから新しいbranchを作る操作である。

```bash
codefire clone main feature-login
```

open directoryの未commit状態をコピーする操作ではない。

## 2.6 Commit

Code Fireのcommitは、整合済み状態の封印である。

commitには以下が含まれる。

```text
content manifest
artifact index
atom index
trace graph
fire delta
resolution records
verification result
policy
consistency certificate
```

未解消fireが残っている状態ではcommitできない。

## 2.7 Artifact

Artifactは、Code Fireが管理する成果物である。

例：

```text
仕様書
設計書
ADR
API仕様
DBスキーマ
コード
テスト
運用Runbook
```

## 2.8 Artifact Atom

Artifact Atomは、成果物内の意味的な最小管理単位である。

例：

```text
REQ-AUTH-001
DES-AUTH-001
CODE-SessionPolicy
TEST-session-expiration
API-login
```

Code Fireではファイル単位ではなくAtom単位で変更影響を管理する。

## 2.9 Trace Link

Trace LinkはArtifact Atom同士の関係である。

例：

```text
REQ-AUTH-001 refined_by DES-AUTH-001
DES-AUTH-001 implemented_by CODE-SessionPolicy
REQ-AUTH-001 verified_by TEST-session-expiration
```

トレーサビリティ表はTrace Linkから生成される派生ビューであり、手編集する一次情報ではない。

## 2.10 Fire

fireは未解消の確認責任である。

```text
CODE-SessionPolicy changed
  -> DES-AUTH-001 の確認が必要
  -> REQ-AUTH-001 の確認が必要
  -> TEST-session-expiration の確認が必要
```

fireはcommit blockerである。

## 2.11 Extinguish

extinguishはfireを解消する操作である。

解消方法：

```text
changed
verified
no-change-required
invalid-fire
```

`no-change-required` でも理由が必要である。

