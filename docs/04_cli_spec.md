# 04. CLI仕様

## 4.1 最小コマンドセット

```bash
codefire init

codefire branch list

codefire open <branch> <path>
codefire close <branch>
codefire close <branch> --discard

codefire clone <source-branch-or-url> <new-branch>

codefire status
codefire scan
codefire fire <atom-id> --reason <text>
codefire extinguish <fire-id> --resolution <type> [--rationale <text>] [--evidence <ref>]
codefire verify
codefire commit -m <message>

codefire merge <source-branch> --into <target-branch>

codefire upload <branch> <server-url>
codefire request-merge <source-url> <target-url>

codefire list <server-url>
codefire show <branch-or-url>
codefire diff [--algorithm myers|patience|histogram] <branch-or-url> <branch-or-url>
codefire request-list <server-url>

codefire discard <branch>
codefire doctor
codefire serve <storage-root> [--host <host>] [--port <port>] [--tls-cert <cert>] [--tls-key <key>]
codefire completion <bash|zsh>
```

## 4.2 作らないコマンド

```text
checkout
pull
fetch
rebase
stash
checkpoint
add
stage
reset --soft
force-upload
```

## 4.3 `open`

```bash
codefire open main ./main
```

仕様：

```text
branch headのsealed commitを指定pathに展開する
同一branchの多重openは禁止する
target pathは存在しないか空でなければならない
.codefire-openを生成する
open registryを更新する
```

## 4.4 `close`

```bash
codefire close feature-login
```

仕様：

```text
open-cleanの場合のみ実ディレクトリを削除する
open-burningまたはopen-consistentの場合は拒否する
```

破棄して閉じる場合：

```bash
codefire close feature-login --discard
```

## 4.5 `clone`

```bash
codefire clone main feature-login
```

仕様：

```text
source branchのsealed headをtarget branchのheadとして設定する
sourceがopen-burningの場合は拒否する
target branchはclosed状態で作成される
```

remoteから取得する場合：

```bash
codefire clone cf://server/org/app/main main-latest
codefire clone cf+http://127.0.0.1:8080/org/app/main main-latest
```

remote tracking branchは作らない。

## 4.6 `scan`

```bash
codefire scan
```

仕様：

```text
open directoryをindexする
Atom単位で変更を検出する
Trace Graphに基づきfireを生成する
obsolete fireを整理する
branch stateを更新する
```

## 4.7 `fire`

```bash
codefire fire REQ-AUTH-001 --reason "仕様と実装が一致していない可能性がある"
```

仕様：

```text
manual fireを立てる
manual fireはrevertで自動消滅しない
extinguishが必要
```

## 4.8 `extinguish`

```bash
codefire extinguish FIRE-001 \
  --resolution no-change-required \
  --rationale "外部仕様は変化していない"

codefire extinguish FIRE-001 \
  --resolution changed \
  --evidence "stale resolutionを再確認した" \
  --refresh
```

仕様：

```text
fireを解消する
basisとしてsource/target/link/policy hashを保存する
`--refresh` 指定時は、すでにextinguishedのfireについて現在のbasisでresolutionを更新する
必要なrationale/evidenceがない場合は拒否する
```

## 4.9 `verify`

```bash
codefire verify
codefire verify --details
```

仕様：

```text
scanを実行する
policy checkを実行する
required verification commandを実行する
verification objectをactive stateに保存する
`--details` 指定時は、失敗したdiagnosticsの代表例を最大5件ずつ出力する
```

## 4.10 `commit`

```bash
codefire commit -m "Implement login handler"
```

仕様：

```text
scanとverifyを再実行する
committable条件を満たす場合だけsealed commitを作る
branch headを更新する
active stateをresetする
open stateをopen-cleanにする
```

## 4.11 `merge`

```bash
codefire merge feature-login --into main
```

仕様：

```text
source sealed headをtarget open directoryへ統合する
targetはopen-cleanでなければならない
merge実行時点ではcommitを作らない
merge fireを生成し、targetをopen-burningにする
```

## 4.12 `upload`

```bash
codefire upload feature-login cf://server/alice/app/feature-login
codefire upload feature-login cf+http://127.0.0.1:8080/alice/app/feature-login
```

仕様：

```text
sealed branchだけuploadできる
open-burning branchはupload不可
server branchが進んでいたら拒否する
force uploadは存在しない
```

## 4.12A `show` / `diff`

```bash
codefire show feature-login
codefire diff main feature-login
codefire diff --algorithm patience main feature-login
codefire diff main feature-login --algorithm=histogram
```

仕様：

```text
local branch、sealed commit ID、cf:// URL、cf+http:// URLを参照できる
参照先のsealed commit妥当性を検証してから表示する
diffのdefault algorithmはmyers
--algorithmはmyers、patience、histogramを受け付ける
myersはexact text diff baselineとして使う
patienceはunique line anchorを優先してrefactor時の可読性を上げる
histogramは低頻度line anchorを優先して繰り返しの多いfileの差分を安定させる
```

## 4.13 `request-merge`

```bash
codefire request-merge \
  cf://server/alice/app/feature-login \
  cf://server/org/app/main

codefire request-merge \
  cf+http://127.0.0.1:8080/alice/app/feature-login \
  cf+http://127.0.0.1:8080/org/app/main
```

仕様：

```text
source sealed commitをtargetに統合してほしいという申請を作る
source/targetともsealed commitを指す
不整合状態はmerge request不可
```
