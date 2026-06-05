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
codefire status --json
codefire scan
codefire scan --json
codefire fire <atom-id> --reason <text>
codefire extinguish <fire-id> --resolution <type> [--rationale <text>] [--evidence <ref>]
codefire verify
codefire verify --json
codefire context --changed --json
codefire context --atom <atom-id> --depth <n> --json
codefire context --fire <fire-id> --json
codefire commit -m <message>

codefire merge <source-branch> --into <target-branch>

codefire upload <branch> <server-url> [--dry-run] [--json]
codefire request-merge <source-url> <target-url> [--dry-run] [--json]

codefire list <server-url>
codefire show <branch-or-url>
codefire diff [--algorithm myers|patience|histogram] [--rename-detection] [--atoms] [--trace] [--impact] [--json] <branch-or-url> <branch-or-url>
codefire request-list <server-url>

codefire discard <branch>
codefire doctor
codefire serve <storage-root> [--host <host>] [--port <port>] [--tls-cert <cert>] [--tls-key <key>]
codefire completion <bash|zsh>
```

Local repository mutators that acquire the repository lock accept:

```bash
--wait-lock
--lock-timeout <duration>
```

仕様:

```text
対象: open, clone, extinguish, extinguish --batch, commit, merge, patch import
--wait-lockはrepo.lockが解放されるまで待機する
--lock-timeoutは待機上限を指定し、指定時は--wait-lockを暗黙に有効化する
durationは裸数または`s` suffixなら秒、`ms` suffixならミリ秒として扱う
timeout時はexit code 30で失敗し、lock fileにpid/created_atがあれば人間向けdiagnosticに含める
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
codefire open main ./main --dry-run --json
codefire open main ./main --wait-lock --lock-timeout 2s
```

仕様：

```text
branch headのsealed commitを指定pathに展開する
同一branchの多重openは禁止する
target pathは存在しないか空でなければならない
.codefire-openを生成する
open registryを更新する
--dry-runはtarget materialization、open marker、registry、branch stateを書き換えず、codefire_operation_planを返す
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
codefire clone main feature-login --dry-run --json
```

仕様：

```text
source branchのsealed headをtarget branchのheadとして設定する
sourceがopen-burningの場合は拒否する
target branchはclosed状態で作成される
--dry-runはbranch recordやremote object graphを書き換えず、codefire_operation_planを返す
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
codefire scan --json
```

仕様：

```text
open directoryをindexする
Atom単位で変更を検出する
Trace Graphに基づきfireを生成する
obsolete fireを整理する
branch stateを更新する
--jsonはcodefire.command_result.v1 envelopeを出力し、data.changed_atoms、data.open_fires、diagnosticsを含める
--jsonはchanged atoms、open fires、verifyに進むためのmachine-readable next_actionsを含める
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

codefire extinguish FIRE-001 \
  --resolution addressed \
  --rationale "修正済み" \
  --dry-run --json

codefire extinguish --batch .codefire/fires-to-extinguish.yaml --dry-run --json
```

仕様：

```text
fireを解消する
basisとしてsource/target/link/policy hashを保存する
`--refresh` 指定時は、すでにextinguishedのfireについて現在のbasisでresolutionを更新する
必要なrationale/evidenceがない場合は拒否する
--dry-runはfire/resolution ledgerを書き換えず、codefire_operation_planを返す
--batchはversion/defaults/firesだけを持つstrict limited YAMLまたは同等JSONを受け取る
--batchは全fire itemを事前検証し、unknown fire、重複fire id、必須rationale/evidence不足があればledgerを書き換えない
--batch --dry-run --jsonはcommand=extinguish-batchのcodefire_operation_planを返し、全itemの単発extinguish validation planをitemsに含める
--batchのYAML schema:
version: 1
defaults:
  resolution: addressed
  evidence: "cargo test --workspace: passed"
fires:
  - id: FIRE-001
    rationale: "REQ/DES linkを確認した"
```

## 4.9 `verify`

```bash
codefire verify
codefire verify --details
codefire verify --json
codefire verify --details --json
```

仕様：

```text
scanを実行する
policy checkを実行する
required verification commandを実行する
verification objectをactive stateに保存する
`--details` 指定時は、失敗したdiagnosticsの代表例を最大5件ずつ出力する
--jsonはcodefire.command_result.v1 envelopeを出力し、verification data、blocking diagnostics、next_actionsを含める
--jsonのexit_codeはprocess exit codeと一致する
verify blockerのexit codeはopen fires=10、missing links=11、stale resolutions=12、duplicate Atom IDs=13、failed checks=14の優先順で決まる
next_actionsはcontext_changed、context_atom、refresh_resolution、rerun_check、commitなどの安定action kindを返す
```

## 4.10 `context`

```bash
codefire context --branch --json
codefire context --changed --json
codefire context --atom REQ-AUTH-001 --depth 2 --json
codefire context --fire FIRE-001 --json
```

仕様：

```text
open directoryから現在のAtomIndex、TraceGraph、baseとの差分、open firesを読み取りcontext packを返す
contextはactive stateを書き換えない
--jsonはcodefire.command_result.v1 envelopeを出力し、data.type=codefire_context_packを含める
selectorは--branch、--changed、--atom、--fireのいずれか1つ
--atomの--depthはTraceGraph上の近傍探索深さを指定する
```

## 4.11 `commit`

```bash
codefire commit -m "Implement login handler"
codefire commit --dry-run --json -m "Implement login handler"
codefire commit -m "Implement login handler" --idempotency-key request-2026-06-05-001
```

仕様：

```text
scanとverifyを再実行する
committable条件を満たす場合だけsealed commitを作る
branch headを更新する
active stateをresetする
open stateをopen-cleanにする
--dry-runはsealed object、branch head、open registry、active stateを書き換えず、codefire_operation_planを返す
--idempotency-keyは成功したcommit resultを.codefire/idempotency/commit/へ記録する
同じ--idempotency-keyかつ同じcommit request payloadは保存済みcommit resultを返し、新しいcommitを作らない
同じ--idempotency-keyでmessage/open_dir/branchが異なるpayloadはexit code 33で拒否する
```

## 4.12 `merge`

```bash
codefire merge feature-login --into main
codefire merge feature-login --into main --dry-run
codefire merge feature-login --into main --dry-run --json
```

仕様：

```text
source sealed headをtarget open directoryへ統合する
targetはopen-cleanでなければならない
merge実行時点ではcommitを作らない
merge fireを生成し、targetをopen-burningにする
--dry-runは共通祖先、file action、file conflict、semantic conflict candidateを予測し、target file / active state / branch stateを変更しない
--jsonはcodefire_merge_plan envelopeを出力し、file_actions、conflicts、semantic_conflicts、next_actionsを含める
semantic conflict candidateは同一Atom IDがsource/target双方でbaseから変わり、最終content_hashが異なる場合に報告する
semantic conflict candidateは自動解決しない
```

## 4.13 `upload`

```bash
codefire upload feature-login cf://server/alice/app/feature-login
codefire upload feature-login cf+http://127.0.0.1:8080/alice/app/feature-login
codefire upload feature-login cf://server/alice/app/feature-login --dry-run --json
```

仕様：

```text
sealed branchだけuploadできる
open-burning branchはupload不可
server branchが進んでいたら拒否する
force uploadは存在しない
--dry-runはlocal branch、sealed commit、remote fast-forward条件を検証し、remote layout、object graph、branch record、generation stateを書き換えない
--json併用時はcodefire_operation_planを返し、copy_object_graphとwrite_remote_branchの予定をoperationsに含める
cf+http dry-runはHTTP write requestを送らず、local object graph収集まででoperation planを返す
```

## 4.14 `show` / `diff`

```bash
codefire show feature-login
codefire diff main feature-login
codefire diff --algorithm patience main feature-login
codefire diff main feature-login --algorithm=histogram
codefire diff --rename-detection main feature-login
codefire diff --atoms --trace main feature-login
codefire diff --impact main feature-login
codefire diff --impact --json main feature-login
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
--rename-detectionは削除/追加fileのsimilarityからrenameを検出する
--rename-detectionは既存fileと追加fileのsimilarityからcopyも検出する
binary fileはpayload diffを出さず、sizeとsha256 prefixのsummaryだけを表示する
--atomsはsealed commit内のAtomIndexを比較し、Atom IDのadded/removed/changedを表示する
--traceはsealed commit内のTraceGraphを比較し、TraceLink IDのadded/removed/changedを表示する
--impactはrequired-link policy上のmissing link増減と、changed Atomから予測されるfire impactを表示する
--jsonはdiff結果をJSON objectとして出力し、impact有効時はmachine-readable next_actionsを含める
```

## 4.15 `review-pack`

```bash
codefire review-pack feature-login
codefire review-pack feature-login --base main --output review-pack.json
codefire review-pack feature-login --base main --algorithm patience --no-rename-detection
```

仕様：

```text
source sealed commitのreview-pack JSONを出力する
--base省略時はsource commitのfirst parentをbaseにする
root commitでは--baseが必須
出力typeはcodefire_review_pack、versionは1
file_diff.textにunified text diffを含める
semantic_diffにAtom diff、TraceGraph diff、impact diff、next_actionsを含める
verificationにbase/sourceそれぞれのverification summaryを含める
--output指定時は指定fileへ書き込み、未指定時はstdoutへ出力する
review-packはsealed commitから再現可能な情報だけで構成し、生成時刻などの非決定的値は含めない
```

## 4.16 `patch`

```bash
codefire patch export feature-login --base main --output feature-login.cfpatch.json
codefire patch export feature-login
codefire patch import feature-login.cfpatch.json --dry-run --json
codefire patch import feature-login.cfpatch.json
```

仕様：

```text
patch exportはbase/source sealed commit間のmanifest deltaをcodefire_patch JSONとして出力する
--base省略時はsource commitのfirst parentをbaseにする
patch entriesはwrite/delete actionを持つ
write entryはbase64 contentとbyte数を持つ
delete entryはpathだけを持つ
patch importはopen directory内で実行する
patch importはpatch base commitとopen directoryのcurrent_base_commitが一致しない場合に拒否する
patch import --dry-runはpath検証とbase照合だけを行い、file / active state / branch stateを変更しない
patch importは適用後にopen stateをopen-burningへ更新し、pending_patch_base、pending_patch_source、patch_pathsをactive stateへ記録する
patch pathはmanifest pathと同じく相対pathだけを許可し、open directory外へescapeするpathを拒否する
```

## 4.17 `request-merge`

```bash
codefire request-merge \
  cf://server/alice/app/feature-login \
  cf://server/org/app/main

codefire request-merge \
  cf+http://127.0.0.1:8080/alice/app/feature-login \
  cf+http://127.0.0.1:8080/org/app/main

codefire request-merge \
  cf://server/alice/app/feature-login \
  cf://server/org/app/main \
  --dry-run --json

codefire request-review cf://server/org/app MR-abc123 --reviewer alice --decision approve --dry-run --json
codefire request-apply cf://server/org/app MR-abc123 --dry-run --json
```

仕様：

```text
source sealed commitをtargetに統合してほしいという申請を作る
source/targetともsealed commitを指す
不整合状態はmerge request不可
request-merge --dry-runはsource/target remote branchとsealed commitを検証し、merge request recordを作成しない
request-review --dry-runはmerge request、stale判定、decision値を検証し、reviews配列とstatusを書き換えない
request-apply --dry-runはapproved status、stale判定、fast-forward条件、source/target object graphを検証し、target branch、object graph、merge request statusを書き換えない
cf+http request-* dry-runはHTTP write requestを送らず、transport/URL/operation planだけを返す
--json併用時は各commandのcodefire_operation_planを返し、write_merge_request、append_review、write_remote_branchなどの予定をoperationsに含める
```
