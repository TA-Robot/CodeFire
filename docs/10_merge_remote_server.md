# 10. Merge / Remote / Server設計

## 10.1 mergeの意味

Code Fireのmergeは、source branchのsealed headをtarget branchのopen directoryへ統合し、必要なfireを立てる操作である。

merge実行時点では正式履歴にcommitを作らない。

```text
merge
  ↓
target open directoryがopen-burningになる
  ↓
fireを解消する
  ↓
verifyする
  ↓
commitする
```

## 10.2 merge前提

```text
source branch:
  sealed headを持つ
  open-burningではない

target branch:
  openされている
  open-cleanである
```

未完了変更があるtargetにはmergeできない。

## 10.3 merge inputs

```text
B = common ancestor commit
S = source head commit
T = target head commit
W = target open directory
```

変数の定義：

- \( B \)：sourceとtargetの共通祖先commit
- \( S \)：source branchのhead commit
- \( T \)：target branchのhead commit
- \( W \)：target branchのopen directory

## 10.4 merge処理

```text
1. B, S, TのContentManifestを読む
2. B, S, TのAtomIndexを読む
3. file-level 3-way mergeを行う
4. Atom-level変更を計算する
5. TraceGraph変更をmergeする
6. conflictがあればconflict markerを出す
7. semantic conflict候補を検出する
8. merge fireを作る
9. targetをopen-burningにする
```

## 10.5 server方針

serverは不整合状態を保存しない。

保存するもの：

```text
sealed commit
sealed branch
merge request
review record
policy
permission
```

保存しないもの：

```text
burning workspace
checkpoint
stash
unverified WIP state
```

## 10.6 remote clone

```bash
codefire clone cf://server/org/app/main main
codefire clone cf+http://127.0.0.1:8080/org/app/main main
```

remote branchをlocal branchとして取得する。remote tracking branchは作らない。

最新版が必要な場合は、再度cloneする。

```bash
codefire clone cf://server/org/app/main main-latest
codefire clone cf+http://127.0.0.1:8080/org/app/main main-latest
```

その後、必要ならmergeする。

## 10.7 upload

```bash
codefire upload feature-login cf://server/alice/app/feature-login
codefire upload feature-login cf+http://127.0.0.1:8080/alice/app/feature-login
```

uploadできるのはsealed branchのみである。

server branchの現在headを \( R \)、uploadしようとするcommitを \( U \) とする。

upload可能条件：

\[
R \in ancestors(U)
\]

変数の定義：

- \( R \)：server上の現在branch head
- \( U \)：clientがuploadしようとしているcommit
- \( ancestors(U) \)：commit \( U \) の祖先commit集合

つまり、fast-forward可能な場合だけuploadできる。

## 10.8 merge request

```bash
codefire request-merge \
  cf://server/alice/app/feature-login \
  cf://server/org/app/main

codefire request-merge \
  cf+http://127.0.0.1:8080/alice/app/feature-login \
  cf+http://127.0.0.1:8080/org/app/main
```

merge requestは、source sealed commitをtarget branchへ統合してほしいという申請である。

MR作成時target commitを \( T_0 \)、現在target headを \( T_1 \) とする。

stale条件：

\[
T_0 \ne T_1
\]

targetが進んだら、MRはstaleである。

serverは分岐したcommitを勝手にsemantic mergeしない。source側で最新targetをcloneし、local mergeし、整合commitを作り直す。
