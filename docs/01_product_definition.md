# 01. プロダクト定義

## 1.1 一文定義

Code Fireは、仕様・設計・コード・テスト・トレーサビリティを単一のシステムグラフとして管理し、未解消の確認責任が存在する状態を正式履歴にcommitすることを禁止する、整合性ファーストの仕様駆動VCSである。

短く言えば、次である。

> Code Fireは、整合済みシステム状態だけをcommitできる、AI時代の仕様駆動バージョン管理システムである。

## 1.2 プロダクト目的

Code Fireの目的は、以下の状態を防ぐことである。

```text
コードだけが変わったが、仕様は未確認
仕様は変わったが、テストは未更新
設計判断は変わったが、ADRが古い
API挙動は変わったが、OpenAPI定義が古い
テストは変わったが、要求仕様との関係が追えない
```

Code Fireは、成果物間の関係をTrace Graphとして管理し、変更時に関連成果物へfireを発火させる。fireがすべてextinguishされるまで、commitは拒否される。

## 1.3 非Git方針

Code FireはGitと共存しない。Gitを内包するわけでも、Git repositoryを裏側に使うわけでもない。

以下は提供しない。

```text
checkout
current branch
staging area
partial commit
stash
checkpoint
pull
fetch
rebase
force upload
remote tracking branch
WIP commit
```

この非提供は制約ではなく、プロダクト思想である。

## 1.4 提供する中核体験

ユーザ体験は以下である。

```text
1. sealed branchをcloneする
2. branchをopenし、実ディレクトリとして展開する
3. 仕様・設計・コード・テストを編集する
4. scanにより変更Atomを検出する
5. Trace Graphに基づきfireが立つ
6. fireをextinguishする
7. verifyする
8. consistentならcommitできる
9. 必要ならuploadまたはmerge requestする
10. closeすると実ディレクトリが消える
```

## 1.5 基本保証

Code Fireは以下を保証する。

```text
branch headは常にsealed commitを指す
sealed commitは常にconsistent certificateを持つ
serverにuploadできるのはsealed branchのみ
merge requestにできるのはsealed branchのみ
open-burning状態は正式履歴に入らない
open-burning状態はserverに流通しない
```

## 1.6 Code Fireが保証しないこと

Code Fireは、世界に対する絶対的な正しさを保証しない。

Code Fireが保証するのは、次である。

> 対象commitが、その時点のCode Fire policy上で、未解消の確認責任を持たず、必要な検証に通っていること。

したがって、policyが不十分なら、Code Fire上でconsistentでも実際のビジネス要求とずれる可能性はある。Code Fireは真理判定機ではなく、確認責任の管理システムである。

