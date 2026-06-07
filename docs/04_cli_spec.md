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
codefire status --json [--metrics]
codefire scan
codefire scan --json [--metrics]
codefire fire <source-atom> --to <target-atom> --reason <text> [--dry-run] [--json]
codefire fire --batch <file> [--path <open-dir>] [--dry-run] [--json]
codefire extinguish <fire-id> --resolution <type> [--rationale <text>] [--evidence <ref>]
codefire verify
codefire verify --json [--metrics]
codefire context --changed --json
codefire context --atom <atom-id> --depth <n> --json
codefire context --fire <fire-id> --json
codefire link --batch <file> [--path <open-dir>] [--dry-run] [--json]
codefire commit -m <message>

codefire merge <source-branch> --into <target-branch>

codefire upload <branch> <server-url> [--dry-run] [--json]
codefire request-merge <source-url> <target-url> [--dry-run] [--json]

codefire list <server-url>
codefire show <branch-or-url>
codefire diff [--algorithm myers|patience|histogram] [--context <lines>] [--rename-detection] [--atoms] [--trace] [--impact] [--json] <branch-or-url> <branch-or-url>
codefire request-list <server-url>

codefire discard <branch>
codefire doctor
codefire storage report [path] [--json] [--large-threshold <bytes|KB|MB|GB>]
codefire evidence add [--path <repo-or-open>] (--artifact <path>|--from-command <command>|--from-argv <program> [--argv <arg>...]|--batch <file>) [--label <text>] [--dry-run] [--json]
codefire explain (fire <id>|atom <id>|verify-failure|storage-warning) [--path <path>] [--json]
codefire migrate check [path] [--json]
codefire migrate dry-run [path] [--target-format v0.6] [--json]
codefire serve <storage-root> [--host <host>] [--port <port>] [--tls-cert <cert>] [--tls-key <key>]
codefire completion <bash|zsh>
```

Implementation status note:

- Python v0.2 `codefire` supports the full HTTPS serve surface shown above.
- Rust v0.6 `codefire` supports `serve <storage-root> [--host] [--port] [--tls-cert] [--tls-key]` for `cf+http://` and `cf+https://`.
- Rust v0.6 supports `--signer`, `--key-id`, `--request-key-id`, commit signature verification, key rotation/revocation policy, request timestamp skew checks, and request nonce replay cache for file-backed and HTTP remote mutators.
- `install.sh` installs Rust v0.6 as the default `codefire` command and keeps Python v0.2 as `codefire-py`.
- Python-only maintenance commands such as remote `gc` and token hash generation remain available through `codefire-py` / `./codefire` while Rust v0.6 is the default install CLI.

Local repository mutators that acquire the repository lock accept:

```bash
--wait-lock
--lock-timeout <duration>
```

仕様:

```text
対象: open, clone, link --batch, extinguish, extinguish --batch, commit, merge, patch import
--wait-lockはrepo.lockが解放されるまで待機する
--lock-timeoutは待機上限を指定し、指定時は--wait-lockを暗黙に有効化する
durationは裸数または`s` suffixなら秒、`ms` suffixならミリ秒として扱う
timeout時はexit code 30で失敗し、lock fileにpid/created_atがあれば人間向けdiagnosticに含める
remote mutatorのupload/request-merge/request-review/request-applyも同じoptionを受け付け、remote project resource lockに適用する
remote branch head更新はbranch resource lockで直列化し、remote generation採番はproject単位の`generation.lock`で直列化する。generation採番APIはlock guard経由でのみ呼び出す
remote mutatorで--json併用時にlock contentionが発生した場合はcodefire.command_result.v1 envelopeをstdoutに出し、diagnosticsにlock_contention、next_actionsにretry_with_wait_lockを含める
HTTP clientはconnect/read/write timeoutを持ち、defaultは30000ms。`CODEFIRE_HTTP_TIMEOUT_MS` に正の整数を指定すると調整できる
HTTP serverはbounded per-connection handlerでrequestを処理し、server-side read/write timeoutを同じ値で設定する。cleartext HTTPで同時接続上限を超えた場合は503を返す
HTTP authorityはIPv4/hostnameとbracketed IPv6 (`[::1]:8080`) を受け付け、bracketなしIPv6 literalは拒否する
HTTP server requestはGET/POSTとHTTP/1.1だけを受け付け、invalid percent escapeやdecode後に`/`、`\`、`.`、`..`を含むpath componentを400で拒否する
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

## 4.3 `doctor`

```bash
codefire doctor [path] [--quick|--full] [--json]
```

仕様:

```text
repository layout、object record integrity、branch head sealed commit、opened registry、active state JSONをread-onlyで検査する
repo layoutはobjects/branches/opened/active/cache/locks/remotes/idempotencyと既知object subdirを検査する
object recordはhash/id/filename/type directoryの整合性を検査する
--quickはobject store全件integrity走査をskipし、layout、branch head、opened registry、active state shape中心に検査する
JSON data.modeはquick/full、data.skipped_checksはquickで省略した検査名を返す
branch headとopened registry current_base_commitはsealed commit validationを実行する
opened registryのopen.path、active_state_path、.codefire-open marker、branch名、open_instance_idを相互検証する
active state filesはJSONとして読めるだけでなく、state/fires/resolutions/scan/verificationの最低限のshapeを検証する
--jsonはcodefire.command_result.v1 envelopeを返し、破損時はok=false、exit_code=20、diagnosticsとnext_actionsを含める
doctor diagnosticsはseverity、category、repairable、blocking、kind、message、pathを含める
```

## 4.4 `open`

```bash
codefire open main ./main
codefire open main ./main --dry-run --json
codefire open main ./main --idempotency-key request-open-001
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
--idempotency-keyは成功したopen resultを.codefire/idempotency/open/へ記録する
同じ--idempotency-keyかつ同じopen request payloadは保存済みopen resultを返し、多重open/target既存checkより先にreplayする
同じ--idempotency-keyでbranch/path/branch_head/repoが異なるpayloadはexit code 33で拒否する
```

## 4.5 `close`

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

## 4.6 `clone`

```bash
codefire clone main feature-login
codefire clone main feature-login --dry-run --json
codefire clone main feature-login --idempotency-key request-clone-001
```

仕様：

```text
source branchのsealed headをtarget branchのheadとして設定する
sourceがopen-burningの場合は拒否する
target branchはclosed状態で作成される
--dry-runはbranch recordやremote object graphを書き換えず、codefire_operation_planを返す
--idempotency-keyは成功したclone resultを.codefire/idempotency/clone/へ記録する
同じ--idempotency-keyかつ同じclone request payloadは保存済みclone resultを返し、target branch既存checkより先にreplayする
同じ--idempotency-keyでsource/new_branch/repoが異なるpayloadはexit code 33で拒否する
```

remoteから取得する場合：

```bash
codefire clone cf://server/org/app/main main-latest
codefire clone cf+http://127.0.0.1:8080/org/app/main main-latest
```

remote tracking branchは作らない。

## 4.7 `scan`

```bash
codefire scan
codefire scan --json
codefire scan --metrics
codefire scan --json --metrics
```

仕様：

```text
open directoryをindexする
Atom単位で変更を検出する
Trace Graphに基づきfireを生成する
scan生成fireのfire_uidはSHA-256由来のstable IDで、display_idもfire key由来のstable短縮IDになる
text出力のopen fire行はdisplay_idとfire_uidを併記する
obsolete fireを整理する
branch stateを更新する
--jsonはcodefire.command_result.v1 envelopeを出力し、data.changed_atoms、data.open_fires、diagnosticsを含める
--jsonはchanged atoms、open fires、verifyに進むためのmachine-readable next_actionsを含める
--metricsはtext出力ではCodeFire metrics block、JSON出力ではdata.metricsを追加する
```

## 4.7.1 Batch Limited YAML Rules

```text
fire --batch、extinguish --batch、link --batch、evidence add --batch の限定YAMLは共通のscalar/comment/key-value parserを使う
コメントはquote外の#から行末まで。double/single quoted scalarは外側quoteを外し、\" と \\ をunescapeする
unsupported section/keyやschema固有validationは各commandで判定し、error messageのcontextはcommandごとのbatch YAML名を使う
```

## 4.8 `fire`

```bash
codefire fire REQ-AUTH-001 --to DES-AUTH-001 --reason "仕様と設計が一致していない可能性がある"
codefire fire --batch fires-batch.yaml --dry-run --json
codefire fire --batch fires-batch.json --path ./main-open --json
```

仕様：

```text
manual fireを立てる。source/target Atomはどちらも現在のopen directoryに存在する必要がある
manual fireはrevertで自動消滅しない
extinguishが必要
--dry-runはfires.jsonやbranch stateを書き換えず、codefire_operation_planを返す
--batchはJSONまたは限定YAMLのversion/fires形式を読み、全fireのfrom/to/reason/severity、Atom存在、batch内重複、既存fire重複を事前検証してから書き込む
batch defaults.reason/defaults.severityを指定すると各fireの省略fieldへ適用する
適用時はrepository lockを取得し、.codefire/active/<branch>/fires.jsonへmanual fireを追記し、open stateをopen-burningへ更新する
--jsonはcodefire.command_result.v1 envelopeを出力し、単発はdata.type=codefire_fire_result、batchはdata.type=codefire_fire_batch_resultを含める
```

## 4.9 `extinguish`

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

codefire extinguish FIRE-001 \
  --resolution addressed \
  --rationale "修正済み" \
  --idempotency-key request-2026-06-05-002

codefire extinguish FIRE-001 \
  --resolution addressed \
  --evidence-ref CF-EVIDENCE-abc123

codefire extinguish FIRE-001 \
  --resolution addressed \
  --edit-rationale \
  --evidence-ref CF-EVIDENCE-abc123

codefire extinguish --interactive --json

codefire extinguish --all-matching "REQ-session -> DES-session" \
  --resolution addressed \
  --rationale "同じsource/targetのfireを同一根拠で確認した"

codefire extinguish --batch .codefire/fires-to-extinguish.yaml --dry-run --json
```

仕様：

```text
fireを解消する
fire-idにはdisplay_idまたはfire_uidを指定できる
text成功出力はdisplay_idとfire_uidを併記する
basisとしてsource/target/link/policy hashを保存する
`--refresh` 指定時は、すでにextinguishedのfireについて現在のbasisでresolutionを更新する
必要なrationale/evidence/evidence-refがない場合は拒否する
--evidence-refは既存のevidence object IDを参照し、resolution.evidence_refsへ保存する
--evidence-refが存在しない、またはevidence以外のobjectを指す場合は拒否する
--edit-rationaleはCODEFIRE_EDITORまたはEDITORで一時draftを開き、保存後の本文をrationaleとして使う
--dry-runはfire/resolution ledgerを書き換えず、codefire_operation_planを返す
--idempotency-keyは成功したextinguish resultを.codefire/idempotency/extinguish/へ記録する
同じ--idempotency-keyかつ同じextinguish request payloadは保存済みextinguish resultを返し、新しいledger entryを作らない
同じ--idempotency-keyでfire/resolution/rationale/evidence/evidence-ref/refresh/open_dir/branchが異なるpayloadはexit code 33で拒否する
--interactiveはopen fire、直近evidence object候補、editor付きdraft commandを返す
--interactive --jsonはtype=codefire_extinguish_interactive_planを返す
--all-matching "SOURCE -> TARGET"はsource_atom/target_atomが一致するopen fireを同じresolution/rationale/evidence/evidence-refで一括解消する
--all-matching --dry-run --jsonはcommand=extinguish-all-matchingのcodefire_operation_planを返し、各fireの単発extinguish validation planをfiresに含める
--batchはversion/defaults/firesだけを持つstrict limited YAMLまたは同等JSONを受け取る
--batchは全fire itemを事前検証し、unknown fire、重複fire id、必須rationale/evidence/evidence-ref不足があればledgerを書き換えない
--batch --dry-run --jsonはcommand=extinguish-batchのcodefire_operation_planを返し、全itemの単発extinguish validation planをitemsに含める
--batchのYAML schema:
version: 1
defaults:
  resolution: addressed
  evidence: "cargo test --workspace: passed"
  evidence_ref: CF-EVIDENCE-abc123
fires:
  - id: FIRE-001
    rationale: "REQ/DES linkを確認した"
```

## 4.10 `verify`

```bash
codefire verify
codefire verify --details
codefire verify --details --blocking-only
codefire verify --json
codefire verify --details --blocking-only --json
codefire verify --metrics
codefire verify --details --blocking-only --json --metrics
```

仕様：

```text
scanを実行する
policy checkを実行する
required verification commandを実行する
verification objectをactive stateに保存する
resolution.evidence_refsが存在しないevidence objectを指す場合はmissing evidence refsとして失敗する
`--details` 指定時は、失敗したdiagnosticsの代表例を最大5件ずつ出力する
--blocking-onlyはblocking diagnosticsだけを表示/返すことを明示する。現行のverify diagnosticsはすべてblockingで、JSON data.diagnostic_filterはblocking_onlyになる
--jsonはcodefire.command_result.v1 envelopeを出力し、verification data、blocking diagnostics、next_actionsを含める
--jsonのexit_codeはprocess exit codeと一致する
verify blockerのexit codeはopen fires=10、missing links=11、stale resolutions=12、missing evidence refs=21、duplicate Atom IDs=13、failed checks=14の優先順で決まる
next_actionsはcontext_changed、context_atom、refresh_resolution、rerun_check、commitなどの安定action kindを返す
next_actionsは最大12件に制限され、超過時はnext_actions_omitted actionに省略件数とlimitを含める
--metricsはtext出力ではCodeFire metrics block、JSON出力ではdata.metricsを追加する
```

## 4.11 `context`

```bash
codefire context --branch --json
codefire context --changed --json
codefire context --atom REQ-AUTH-001 --depth 2 --json
codefire context --atom REQ-AUTH-001 --depth 2 --limit 25 --json
codefire context --fire FIRE-001 --json
```

仕様：

```text
open directoryから現在のAtomIndex、TraceGraph、baseとの差分、open firesを読み取りcontext packを返す
contextはactive stateを書き換えない
--jsonはcodefire.command_result.v1 envelopeを出力し、data.type=codefire_context_packを含める
selectorは--branch、--changed、--atom、--fireのいずれか1つ
--atomの--depthはTraceGraph上の近傍探索深さを指定する
--depthは最大8に制限され、--limitはatoms/trace_links/firesの出力上限を指定する
context packはlimitsとtruncatedを含め、preview scan由来のfire metadataはscan.fire_sourceで区別する
```

## 4.12 `commit`

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

## 4.13 `merge`

```bash
codefire merge feature-login --into main
codefire merge feature-login --into main --dry-run
codefire merge feature-login --into main --dry-run --json
codefire merge feature-login --into main --idempotency-key request-merge-local-001
```

仕様：

```text
source sealed headをtarget open directoryへ統合する
targetはopen-cleanでなければならない
merge実行時点ではcommitを作らない
merge fireを生成し、targetをopen-burningにする
--dry-runは共通祖先、file action、file conflict、semantic conflict candidateを予測し、target file / active state / branch stateを変更しない
--jsonはcodefire_merge_plan envelopeを出力し、file_actions、conflicts、binary_conflicts、semantic_conflicts、next_actionsを含める
--idempotency-keyは成功したmerge resultを.codefire/idempotency/merge/へ記録する
同じ--idempotency-keyかつ同じmerge request payloadは保存済みmerge resultを返し、target open-burning checkより先にreplayする
同じ--idempotency-keyでsource/target/source_head/target_head/base/target_dir/repoが異なるpayloadはexit code 33で拒否する
semantic conflict candidateは同一Atom IDがsource/target双方でbaseから変わり、最終content_hashが異なる場合に報告する
semantic conflict candidateは自動解決しない
text conflictはtarget fileへconflict markerを書き込む
binaryまたは非UTF-8 conflictはtarget fileへlossy markerを書かず、`.codefire-conflicts/<path>/target` と `.codefire-conflicts/<path>/source` に復元用bytesを保存し、binary_conflicts metadataへ記録する
```

## 4.14 `upload`

```bash
codefire upload feature-login cf://server/alice/app/feature-login
codefire upload feature-login cf+http://127.0.0.1:8080/alice/app/feature-login
codefire upload feature-login cf://server/alice/app/feature-login --dry-run --json
codefire upload feature-login cf://server/alice/app/feature-login --idempotency-key request-upload-001
codefire upload feature-login cf://server/alice/app/feature-login --wait-lock --lock-timeout 2s
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
--idempotency-keyは成功したupload resultをremote projectのidempotency/remote_upload/へ記録する
同じ--idempotency-keyかつ同じupload payloadは保存済みupload resultを返し、remote branch fast-forward checkより先にreplayする
同じ--idempotency-keyでlocal branch/head/remote branch/project/object bundleが異なるpayloadはexit code 33で拒否する
remote idempotency recordはhash-onlyで保存し、payload全文や署名metadata、operation planは保存しない。file-backed replay planは`replayed: true`を持つ軽量metadataになり、HTTP replay responseにも`replayed: true`を含める
cf+http uploadはserver側remote projectのidempotency recordで同じ規則を適用する
cf+http uploadのserver側tmp object directoryはrequestごとに一意化され、cleanupは自分のrequest directoryだけを対象にする
```

## 4.15 `show` / `diff`

```bash
codefire show feature-login
codefire diff main feature-login
codefire diff --algorithm patience main feature-login
codefire diff main feature-login --algorithm=histogram
codefire diff --context 1 main feature-login
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
--contextはtext diff hunkに含める前後context行数を指定する。defaultは3
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
text diff payloadはunified hunk headerを持ち、file単位の最大出力byte数でboundedになり、超過時は省略したchanged line数を表示する
--rename-detectionはexact hash renameを先に検出し、candidate pair数が上限を超えるinexact similarity計算はwarning付きでskipする
```

## 4.16 `review-pack`

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

## 4.17 `storage report`

```bash
codefire storage report
codefire storage report /path/to/repo --json
codefire storage report --quick --json
codefire storage report --large-threshold 16MB
codefire storage report --remote cf:///srv/codefire/org/app --json
```

仕様：

```text
repository rootを探索し、.codefire/objects、.codefire/active、.codefire/idempotencyのfile数とbytesを集計する
full modeではobject storeをObjectRecord wrapperとしてparseし、record type別のfile数/bytesとlargest object上位を返す
--quickはobject JSON parseを省略し、file数/bytes/largest/large_object warningだけをmetadataから返す。data.mode=quick、data.skipped_checksに省略した解析名を含める
--large-threshold以上のobjectはlarge_object warningとしてdiagnosticsに出す
object JSONが読めない場合はinvalid_object_json warningとしてdiagnosticsに出し、report自体は継続する
ObjectRecordのrecord typeとpayload.typeが異なる場合はpayload_type_mismatch warningとしてdiagnosticsに出す
--remoteはfile-backed remote project URLを追加で集計し、remote側objects/branches/merge_requests/idempotencyのfile数とbytes、gc retention policy、current generation、object generation別容量を返す
remote `server_policy.json` の `gc.idempotency_retention_seconds` が正なら、storage reportはremote idempotency recordのexpired_files/expired_bytes/oldest_created_atを返し、期限超過recordをremote_idempotency_retention warningとして出す
--jsonはcodefire.command_result.v1 envelopeを出力し、data.type=codefire_storage_reportを含める
warningがある場合、next_actionsにinspect_storage_warningsを含める
external_artifactsはartifact_ref objectのrefs、referenced_bytes、payload_bytes_stored=0を返す
artifact_refは外部artifact本体を.codefire/objectsへコピーせず、URI/path/hash/size metadataだけを保存する。defaultでは絶対local pathを保存しない
```

## 4.18 `link --batch`

```bash
codefire link --batch links-batch.yaml --dry-run --json
codefire link --batch links-batch.json --path ./main-open --json
```

仕様：

```text
open directoryのcodefire.links.yamlへTrace Linkを追記する
--batchはJSONまたは限定YAMLのversion/links形式を読み、全linkのfrom/to/type、Atom存在、batch内重複、既存link重複を事前検証してから書き込む
--batch --dry-runはcodefire.links.yamlを書き換えず、codefire_operation_planを返す
batch defaults.typeを指定すると各linkのtype省略時に使う
適用時はrepository lockを取得し、codefire.links.yamlへquoted scalar形式で追記する
既存codefire.links.yamlにroot `links:` sectionがある場合は、そのsection内で次のroot keyより前へ追記する。空ファイルは`version: 1`と`links:`を作成する。root `links:` がなく別root keyがある構造やroot `links:` 重複はunsupported errorにする
validationは全itemを確認し、dry-runではdata.valid=falseとitem_index付きdiagnosticsを返す。非dry-runではdiagnosticsを集約したerrorで書き込み前に止める
--jsonはcodefire.command_result.v1 envelopeを出力し、data.type=codefire_link_batch_result、data.valid、data.diagnosticsを含める
```

## 4.19 `evidence add`

```bash
codefire evidence add --artifact runs/model.bin --label "best checkpoint" --json
codefire evidence add --from-command "cargo test --workspace" --json
codefire evidence add --from-argv cargo --argv test --argv --workspace --json
codefire evidence add --artifact runs/model.bin --from-command "python eval.py" --timeout 30s --max-output-bytes 65536
codefire evidence add --batch evidence-batch.yaml --dry-run --json
codefire evidence add --batch evidence-batch.json --json
```

仕様：

```text
repository rootを探索し、evidence objectを.codefire/objects/evidenceへ保存する
--artifactはartifact_ref objectを.codefire/objects/artifact_refsへ保存する
artifact_refはpath/path_kind/local_path_redacted/uri/hash_algorithm/content_hash/hash_streaming/hash_chunk_bytes/large_artifact/large_artifact_threshold_bytes/size_bytes/captured_atを持ち、payload本体は保存しない
--artifact-uri指定時はartifact_ref.uriへ保存する。未指定時、repo内artifactはrepo相対pathと`repo://<relative-path>`を保存し、repo外artifactはpathを`<redacted>`、uriを`artifact://redacted/<hash-prefix>`にする
1MiB以上のartifactはlarge_artifact warningをdiagnosticsへ出し、SHA-256は64KiB chunkのstreaming hashとして計算する
--from-commandはshell commandのstdout/stderr/exit_code/success/timed_out/timeout_ms/duration_ms/cwd/mode/shellを保存する
--from-argvはshellを使わずprogramと--argvで指定した引数配列を直接実行し、stdout/stderr/exit_code/success/timed_out/timeout_ms/duration_ms/cwd/mode/shell/argvを保存する
--from-commandと--from-argvは同時指定できない。--from-commandは`mode: "shell"` / `shell: true`、--from-argvは`mode: "argv"` / `shell: false`として保存する
--timeoutまたは--timeout-msはcommand captureの待機上限を指定する。未指定時は300000ms
--cwd未指定時のcommand cwdは探索されたrepository root
stdout/stderrは--max-output-bytesでそれぞれtruncateされ、truncated flagを保存する
commandがnon-zeroまたはtimeoutでもevidence capture自体は成功し、command_exit_codeとcommand_timed_outを返す
--jsonはcodefire.command_result.v1 envelopeを出力し、data.type=codefire_evidence_add_resultを含める
--batchはJSONまたは限定YAMLのversion/items形式を読み、全itemのartifact path/cwd/必須fieldを事前検証してからevidence objectを作る
--batch --dry-runはartifact_ref/evidence objectを書き込まず、codefire_operation_planを返す
batch itemはartifact、artifact_uri、from_command、from_argv、cwd、timeout_ms、label、max_output_bytesを持てる。JSON batchのfrom_argvは文字列配列、YAML batchのfrom_argvはinline JSON string arrayで指定する
resolutionから参照されたevidence objectと、そのevidenceが参照するartifact_refはremote object graphに含めてupload/clone/request workflowで搬送する
legacy artifact_refが絶対local pathを含む場合、remote upload planはartifact_path_sensitive warningを返す
```

## 4.20 `explain`

```bash
codefire explain fire FIRE-001 --path ./main-open --json
codefire explain atom REQ-session --path ./main-open --depth 2 --json
codefire explain verify-failure --path ./main-open --json
codefire explain storage-warning --path ./repo --large-threshold 16MB --json
```

仕様：

```text
read-only診断コマンドで、open directoryまたはrepositoryを読み取り、active stateやobject storeを書き換えない
fireはopen fireのsource/target/reason/trace_pathを説明し、context_fireとextinguish_fire next_actionsを返す
atomはcontext packを内包し、近傍Atom、TraceLink、関連fireを説明する
verify-failureはverifyをpersist=falseで実行し、blocker diagnosticsとverification next_actionsを返す
storage-warningはstorage reportを実行し、large object/invalid JSON warning、largest objects、external artifact summaryを返す
--jsonはcodefire.command_result.v1 envelopeを出力し、data.type=codefire_explainを含める
```

## 4.21 `migrate`

```bash
codefire migrate check
codefire migrate check /path/to/repo --json
codefire migrate dry-run /path/to/repo --target-format v0.6 --json
```

仕様：

```text
repository rootを探索し、repo.json、object record、branch head sealed commitを検証する
checkは互換性blockerとwarningを返し、互換性がない場合はexit code 40で終了する
dry-runはtarget format v0.6へ向けたplanned_actionsを返すが、file systemを書き換えない
object recordはobject_id/hash/filenameとpayload canonical digestを再検証する
branch recordはheadがvalid sealed commit graphを指していることを検証する
v0.6で必要なmissing directoryはplanned_actions=create_directoryとして返す
--jsonはcodefire.command_result.v1 envelopeを出力し、data.type=codefire_migration_reportを含める
```

## 4.22 `patch`

```bash
codefire patch export feature-login --base main --output feature-login.cfpatch.json
codefire patch export feature-login
codefire patch import feature-login.cfpatch.json --dry-run --json
codefire patch import feature-login.cfpatch.json
codefire patch import feature-login.cfpatch.json --idempotency-key request-patch-import-001
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
patch import --idempotency-keyは成功したimport resultを.codefire/idempotency/patch_import/へ記録する
同じ--idempotency-keyかつ同じpatch import payloadは保存済みimport resultを返し、file / active state / branch stateを再変更しない
同じ--idempotency-keyでpatch file/content/base/open_dir/branch/repoが異なるpayloadはexit code 33で拒否する
patch pathはmanifest pathと同じく相対pathだけを許可し、open directory外へescapeするpathを拒否する
```

## 4.23 `request-merge`

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
codefire request-merge cf://server/alice/app/feature-login cf://server/org/app/main --idempotency-key request-mr-001
codefire request-review cf://server/org/app MR-abc123 --reviewer alice --decision approve --idempotency-key request-review-001
codefire request-apply cf://server/org/app MR-abc123 --idempotency-key request-apply-001
codefire request-apply cf://server/org/app MR-abc123 --wait-lock --lock-timeout 2s
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
--idempotency-keyは成功したrequest-merge/review/apply resultをremote projectのidempotency/remote_request_*/へ記録する
同じ--idempotency-keyかつ同じremote request payloadは保存済みresultを返し、MR status/stale/approved checkより先にreplayする
同じ--idempotency-keyでsource/target/MR/reviewer/decision/comment/projectが異なるpayloadはexit code 33で拒否する
remote request idempotency recordもhash-only保存で、payload全文やoperation planを保存しない。replay responseは`replayed: true`で識別できる
cf+http request-* はserver側remote projectのidempotency recordで同じ規則を適用する
```
