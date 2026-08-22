# 04. CLI仕様

## 4.1 最小コマンドセット

```bash
codefire init
codefire import [path] [--branch main] [--dry-run] [--json]

codefire branch list [--json] [--metrics]
codefire branch show [branch] [--json]

codefire open <branch> <path>
codefire close <branch>
codefire close <branch> --discard

codefire clone <source-branch-or-url> <new-branch>

codefire status
codefire status --json [--metrics]
codefire scan [--wait-lock] [--lock-timeout <duration>]
codefire scan --json [--metrics] [--wait-lock] [--lock-timeout <duration>]
codefire fire <source-atom> --to <target-atom> --reason <text> [--dry-run] [--full] [--json]
codefire fire --batch <file> [--path <open-dir>] [--dry-run] [--full] [--json]
codefire extinguish <fire-id> --resolution <type> [--rationale <text>] [--evidence <ref>]
codefire verify [--wait-lock] [--lock-timeout <duration>]
codefire verify --json [--metrics] [--wait-lock] [--lock-timeout <duration>]
codefire context --changed --json
codefire context --atom <atom-id> --depth <n> --json
codefire context --fire <fire-id> --json
codefire link --batch <file> [--path <open-dir>] [--dry-run] [--json]
codefire commit -m <message>

codefire merge <source-branch> --into <target-branch>
codefire merge --continue [--path <open-dir>] [--accept-semantic-conflicts]

codefire upload <branch> <server-url> [--dry-run] [--json]
codefire request-merge <source-url> <target-url> [--dry-run] [--json]

codefire list [path|<server-url>] [--limit <1..1000>] [--cursor <cursor>] [--json]
codefire show <branch-or-url>
codefire diff [--algorithm myers|patience|histogram] [--context <lines>] [--rename-detection] [--atoms] [--trace] [--impact] [--json] <branch-or-url> <branch-or-url>
codefire request-list <server-url> [--limit <1..1000>] [--cursor <cursor>] [--json]

codefire discard <branch>
codefire doctor [path|<server-url>] [--quick|--full] [--json]
codefire storage report [path] [--json] [--large-threshold <bytes|KB|MB|GB>]
codefire evidence add [--path <repo-or-open>] (--artifact <path>|--from-command <command>|--from-argv <program> [--argv <arg>...]|--batch <file>) [--label <text>] [--dry-run] [--json]
codefire explain (fire <id>|atom <id>|verify-failure|storage-warning) [--path <path>] [--json]
codefire migrate check [path] [--json]
codefire migrate dry-run [path] [--target-format current|v0.6] [--json]
codefire migrate apply [path] [--target-format current|v0.6] [--full] [--json] [--wait-lock] [--lock-timeout <duration>]
codefire serve <storage-root> [--host <host>] [--port <port>] [--tls-cert <cert>] [--tls-key <key>] [--tls-client-ca <ca-pem>]
codefire completion <bash|zsh>
codefire capabilities [--json]
```

Implementation status note:

- Python v0.2 `codefire` supports the full HTTPS serve surface shown above.
- Rust v0.6 `codefire` supports `serve <storage-root> [--host] [--port] [--tls-cert] [--tls-key] [--tls-client-ca]` for `cf+http://` and `cf+https://`.
- Rust v0.6 supports `--signer`, `--key-id`, `--request-key-id`, commit signature verification, key rotation/revocation policy, request timestamp skew checks, and request nonce replay cache for file-backed and HTTP remote mutators.
- `install.sh` installs Rust v0.6 as the default `codefire` command and keeps Python v0.2 as `codefire-py`.
- After install, `install.sh` reports the active `codefire` path resolved from PATH. If PATH resolves a different binary than the installed target, it prints a warning; if it resolves the installed target, it verifies active help/version and reports `active codefire verified`.
- Python-only maintenance commands such as remote `gc` and token hash generation remain available through `codefire-py` / `./codefire` while Rust v0.6 is the default install CLI.
- `codefire capabilities --json` returns `codefire.capabilities.v1`, the command capability registry used by help/completion. It exposes target classes, output schema, pagination mode, `supports_json` / `supports_metrics`, and the generated `codefire.support.v1` compatibility manifest.

Local repository mutators that acquire the repository lock accept:

```bash
--wait-lock
--lock-timeout <duration>
```

仕様:

```text
対象: open, clone, scan, verify, link --batch, extinguish, extinguish --batch, commit, merge, patch import
--wait-lockはrepo.lockのkernel ownershipが解放されるまで待機する（persistent lock fileの存在だけではcontentionではない）
--lock-timeoutは待機上限を指定し、指定時は--wait-lockを暗黙に有効化する
durationは裸数または`s` suffixなら秒、`ms` suffixならミリ秒として扱う
timeout時はexit code 30で失敗し、no-follow/16 KiB上限で読める`repo.lock.owner.json` sidecarにpid/created_atがあれば人間向けdiagnosticに含める。sidecarがsymlink、non-regular、oversized、malformedでもfollow/trustせず、ownership判定はkernel lockだけで行う
local/import repository lockの`repo.lock` bytesはacquire時にも変更せず、regular fileかつfilesystem link数がexactly 1と証明できる場合だけ使う。取得後もcanonical pathnameのfile identity/link provenanceを再検証し、owner sidecarはlock取得後にatomic replaceする
remote mutatorのupload/request-merge/request-review/request-applyも同じoptionを受け付け、remote project resource lockに適用する
remote branch head更新はbranch resource lockで直列化し、remote generation採番はproject単位の`generation.lock`で直列化する。generation採番APIはlock guard経由でのみ呼び出す
remote mutatorで--json併用時にlock contentionが発生した場合はcodefire.command_result.v1 envelopeをstdoutに出し、diagnosticsにlock_contention、next_actionsにretry_with_wait_lockを含める
--jsonを持つ主要commandはcodefire.command_result.v1 envelopeを返す。旧来のoperation/merge plan payloadを返していたmutating commandは同じpayloadをdata.planへ入れ、data.type=codefire_plan_resultを付ける
HTTP clientはconnect/read/write timeoutを持ち、defaultは30000ms。`CODEFIRE_HTTP_TIMEOUT_MS` に正の整数を指定すると調整でき、不正値または0は設定エラーとして拒否する
HTTP client/serverはrequest/response bodyのContent-Lengthを既定上限で検査し、上限超過はbody読み取り前に拒否する
HTTP decision-workは`codefire.decision-work.v1`を使い、graph/object/byte/depth/frontier/retained workを一requestのatomic reservation ledgerへ累積する。超過はexit code 60、`resource_budget_exceeded`、profile/dimension/observed/limit/phase/command/continuationを返し、HTTPでは成功header前の413になる
HTTP JSON success bodyは16 MiB上限付きwriterで一度だけencodeする。limit exactは成功し、limit+1は`200`またはpartial bodyを一切送らない。atomic object bundleはcontinuationを返さない
HTTP serverはbounded per-connection handlerでrequestを処理し、server-side read/write timeoutを同じ値で設定する。cleartext HTTPで同時接続上限を超えた場合は503を返す
HTTP authorityはIPv4/hostnameとbracketed IPv6 (`[::1]:8080`) を受け付け、bracketなしIPv6 literalは拒否する
HTTP server requestはGET/POSTとHTTP/1.1だけを受け付け、invalid percent escapeやdecode後に`/`、`\`、`.`、`..`を含むpath componentを400で拒否する
```

### 4.1.1 Active state capability contract

local workflowがopened registryのactive stateを使う場合、`open.active_state_path`がcanonical repositoryの`.codefire/active/<open_instance_id>`と完全一致することを検証し、そのinstance directoryとparent directoryをno-follow capabilityとして保持する。

- `state.json`、`fires.json`、`resolutions.json`、`scan.json`、`verification.json`、`commit_transaction.json`、`state_transaction.json`とtransaction archiveのread/write/removeは、検証済みinstance capabilityに対する1個のnormal relative componentとして行う。ambient absolute pathへ戻るfallbackは持たない。
- readはfinal symlinkをfollowせず、non-blocking regular fileだけをbounded readする。writeはcapability内のcreate-new tempへ完全書込み・syncしてからrelative renameし、directoryをsyncする。各operationはambient parent/instance pathが保持したdirectory identityを指すことをuse boundaryで再検証する。
- validation後にinstance nameがrenameされ、元nameがrepository外へのsymlinkへ差し替えられた場合、read/write/removeはrepository外へredirectせずfail closedする。JSON failureはexit code 20と`active_state_capability_invalid` diagnosticを返し、`operation`と`path`を含む。
- close/discardのactive-state tree削除はtransactionに記録したstable directory identityとdeterministic sibling quarantine nameを使う。移動はkernel atomic no-replace renameで行い、collision entryを上書きしない。削除開始後の失敗は部分削除treeを元nameへ戻さず、errorに示したquarantineをretry/recoveryのため保持する。
- `doctor`と`storage report`によるactive root/instanceの列挙・metadata readもno-follow directory capabilityを使い、active rootまたはchild symlinkをrepository外のreadへ使わない。

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
repo layoutはshared layout registryに基づき、objects/branches/opened/activeのrequired directory、cache/locks/remotes/idempotencyのauto-create directory、既知object subdirを検査する
auto-create可能なlayout不足はnon-blocking warningとして扱い、migrate dry-runのrepair planへ接続する
object recordはhash/id/filename/type directoryの整合性を検査する
--quickはobject store全件integrity走査をskipし、layout、branch head、opened registry、active state shape中心に検査する
JSON data.modeはquick/full、data.skipped_checksはquickで省略した検査名を返す
branch headとopened registry current_base_commitはsealed commit validationを実行する
opened registryのopen.path、active_state_path、.codefire-open marker、branch名、open_instance_idを相互検証する
active state filesはJSONとして読めるだけでなく、state/fires/resolutions/scan/verificationの最低限のshapeを検証する。active state directoryにstate.jsonがない場合はmissing_active_state_fileを出し、open-clean/open-consistentとnon-empty fires.jsonの矛盾はactive_state_invariant_violationを出す
validなpending `commit_transaction.json`は`pending_commit_transaction`としてmarker pathとmessage内のphase/stored role-entry件数/planned role-entry件数を返す。candidate stateやentryごとのrole/path/object type/object IDはmarker自体のfieldであり、doctor issueの独立したJSON fieldとしては返さない。legacy string entryはstoredとして数える
validなpending `state_transaction.json`は`pending_state_transaction`としてmarker pathとmessage内のoperation/phase/intended stateを返す。unknown operation/phaseやshape不正はpendingとみなさず`invalid_active_state_shape`にする
--jsonはcodefire.command_result.v1 envelopeを返し、破損時はok=false、exit_code=20、diagnosticsとnext_actionsを含める
doctor diagnosticsはseverity、category、repairable、blocking、kind、message、pathを含める
```

remote URLを指定した場合はlocal repository探索を行わず、file-backed remoteまたはHTTP(S)
`GET /v1/projects/{org}/{app}/doctor`をread-onlyで検査する。JSONの`data`は
`codefire.remote_doctor.v1`で、project/transport/layout、branch/MR/object/reference count、
bounded issue sample、quick時のskipped checkを返す。remote projectが存在しない場合もI/O例外ではなく
typed reportを返し、未知branch/MR versionやcritical extensionは意味fieldの利用前に拒否する。

## 4.3.1 Remote list pagination

remote `list` / `request-list`は`--limit`を指定したとき`codefire.page.v1`を返す。次ページは
`data.page.next_cursor`を同じcommand、collection、limitへ渡す。cursorは完全なordered resultのsnapshot
digestを束縛するため、途中でcollectionが変化した場合はstaleとして拒否し、先頭ページから再開する。
cursorの別collection流用、limit変更、future/malformed cursorも拒否する。pagination optionを省略した
既存呼び出しは従来どおり全件を返す。

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
targetはcanonical repository rootのparentをresolved workspace boundaryとし、`..`、既存symlink component、repositoryとのoverlap、既存open directoryとのnesting/overlapを拒否する
applyはworkspace boundaryからtargetまでをno-followで開いたpinned directory capabilityに対して全fileと`.codefire-open`をrelative writeする。materialization後、repository metadataをpublishする前にambient target pathが同じdirectory identityを指すことを再検証する
.codefire-openを生成する
open registryを更新する
--dry-runはtarget materialization、open marker、registry、branch stateを書き換えず、codefire_operation_planを返す
--idempotency-keyは成功したopen resultを.codefire/idempotency/open/へ記録する
同じ--idempotency-keyかつ同じopen request payloadは保存済みopen resultを返し、多重open/target既存checkより先にreplayする
同じ--idempotency-keyでbranch/path/branch_head/repoが異なるpayloadはexit code 33で拒否する
--json failureは`open_target_resolution_failed` diagnosticを返し、reason、repo、branch、requested_path、nullable resolved_path、workspace_boundaryを含める。reasonは`unsafe_path_component`、`outside_workspace_boundary`、`repository_overlap`、`open_directory_overlap`、`target_changed_before_use`、`identity_changed_during_use`、`materialization_write_failed`を区別し、next actionは安全なtargetでの`open ... --dry-run --json`を促す
```

## 4.4.1 `import`

```bash
codefire import .
codefire import ./existing-project --branch main --dry-run --json
codefire import ./existing-project --idempotency-key initial-adopt
```

既存の非空directoryをCodeFire repositoryかつopen directoryとして採用する。`init` + `open` と異なり、targetが既存fileを含んでいても拒否しない。採用後は初期empty commitをbaseとして `.codefire-open`、opened registry、active stateを作成し、既存fileを現在のworktree変更として扱う。

- targetが存在しない、directoryではない、既に `.codefire` または `.codefire-open` を持つ場合は拒否する。
- `--dry-run --json` はrepo metadata、open marker、opened registry、active state、初回scanのplanと `initial_files` を返し、filesystemへ書かない。
- 適用時は初期repository metadataを可視化する前にrepository lockを取得し、`repo.json`、sealed object graph、branch refの順で初期repositoryをpublishしてからopen metadataを生成する。最後のscan publicationは通常scanと同じoperation-scoped `state_transaction.json` phase列を検証し、branch stateを実際のworktree差分に合わせる。
- 適用前にpayload-bound bootstrap lock/transactionを作り、planned/initialized/opened/scanned/completedの各durable phaseから同一requestを再開できる。`.codefire-import.lock`と`.codefire/locks/repo.lock`は安定したinode上のkernel advisory lockとして残し、pathの存在だけをcontentionと扱わない。repository lockのbytesは変更せずexactly-one-link provenanceを取得前後に検証し、owner metadataは共通の`.codefire/locks/repo.lock.owner.json`へlock取得後にatomic publishする。異なるrequest、malformed/foreign published provenance、legacy result-only idempotency markerはfail closed。
- `.codefire/repo.json` はimport transactionとdeterministic open identityのexact provenanceを保持し、retryが同じtransactionのpartial initだけを再開する。通常の`codefire init` repositoryをimport recoveryとして採用しない。
- imported rootでは`.codefire-import.lock.bootstrap-*.part`を未公開staging、`.codefire-import.lock.bootstrap-*.tmp`をfully-written bootstrap publication candidateとしてreserveする。`.part`はauthorityとしてparseせず、canonical lockと同一file identityと証明できるもの以外は保存したままscanから除外する。malformed `.tmp`は保存してfail closed、異なるfully-formed contenderは勝者が勝手にunlinkしない。
- rootのatomic replacementは`.codefire-import.write-<transaction-id>-*.tmp`を使う。retryは同じtransaction IDのwrite tempだけを回収し、foreign IDは保存してfail closedする。上記のexact root names/patternsはAtom/content scanから除外するが、nested pathや似ただけのnameは通常のuser contentとして追跡する。dry-runはこれらreserved root pathが既存すれば書き込まずfail closedする。
- importのbootstrap/transaction/idempotencyなどcontrol fileは1 MiB、staged/current `.codefire/**` repository fileは64 MiBをread上限とする。user fileはbase64 blob record全体をobject-storeの64 MiB上限へ収めるため49,545,216 bytes（47.25 MiB）までとし、readerはno-follow/non-blocking regular-file open、metadata size check、limit+1 bounded readで特殊fileとread中のgrowthを拒否する。
- `--branch` は初期branch名を指定する。省略時は `main`。
- 初期import後の通常導線は `codefire scan --json`、`codefire verify --json`、`codefire commit -m "Initial import"`。

## 4.5 `close`

```bash
codefire close feature-login
```

仕様：

```text
open-cleanの場合のみ実ディレクトリを削除する
open-burningまたはopen-consistentの場合は拒否する
repository root自体は削除しない
--keep-dirは実ディレクトリを残し、.codefire-open markerだけを外す
```

破棄して閉じる場合：

```bash
codefire close feature-login --discard
codefire discard feature-login
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
local sourceがopen-burningまたはopen-consistentの場合は拒否する
local sourceがclosedまたはopen-cleanの場合のみsealed headを利用する
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

## 4.7 `status`

```bash
codefire status [path|--path <open-dir>] [--json] [--metrics]
```

仕様：

```text
open registryとactive fires ledgerから現在のbranch/state/open fire countを返す
branch head current_base_commitはsealed commit validationを実行する
open fire数が正の場合、raw stateがopen-clean/open-consistentでもstatus.stateはopen-burningとして返す
--jsonはcodefire.command_result.v1 envelopeを返す
--json data.scan_predictionはactive stateを書き換えずに現在worktreeをpreview scanし、branch_state、changed_count、open_fire_count、tool_migration、base_commitを返す
--metricsはtext出力ではCodeFire metrics block、JSON出力ではdata.metricsを追加する
```

## 4.8 `scan`

```bash
codefire scan
codefire scan --json
codefire scan --metrics
codefire scan --json --metrics
codefire scan --json --full
codefire scan --wait-lock --lock-timeout 2s --json
```

仕様：

```text
open directoryをindexする
live scanはrepository lock取得後にopen contextを再読込し、current baseのcomplete sealed validationとTrace endpoint validationをwrite前に実行する
Atom単位で変更を検出する
full manifest差分をbase/current Atom indexと照合し、`data.change_coverage`で`non_atom_file`、`deleted_atom`、`out_of_atom_change`、`atom_change_with_unclassified_spans`へ全changed pathを分類する
Atom selectorは永続spanを持たないため`span_precision=file_only_no_span_claim`とし、changed Atomがあるfile全体をcoveredとはclaimしない
互換`non_atom_changed_files`はnon-Atom、deleted-Atom、out-of-Atom entryを投影する
Trace Graphに基づきfireを生成する
scan生成fireのfire_uidはSHA-256由来のstable IDで、display_idもfire key由来のstable短縮IDになる
text出力のopen fire行はdisplay_idとfire_uidを併記する
obsolete fireを整理する
branch stateを更新する
live write sequenceはactive `state_transaction.json` を先に作り、旧verification.jsonをinvalidateしてからfires/scanを更新する。最終phaseではregistry/active/branchのstate、今回計算したfires/scan authority、scan baseとregistry base/branch head、persisted authorityから導出したstateを照合した後だけmarkerを削除する。retryは既存markerのversion/status/operation/phase/intended_state/timestamp shapeとoperation-phase対応を最初のrecovery writeより前に検証する。writerだけが各phase updateを隣接transitionに制限し、doctor/retryは保存された1つのphaseが既知かどうかを検証する。pending verifyをscanで引き継ぐ場合は全stateを再計算し、stale verificationを残さずhealする
--jsonはcodefire.command_result.v1 envelopeを出力し、既定ではchanged atoms/open firesをbounded sampleとして返す
data.changed_count、data.changed_atom_count、data.non_atom_changed_file_count、data.open_fire_count、data.changed_atoms_omitted、data.non_atom_changed_files_omitted、data.open_fires_omittedで全体件数と省略件数を返す
data.changed_atom_idsとdata.changed_atomsは後方互換のID sample配列、data.changed_atoms_sampleはpath/kind/selector/content_hashを含むobject sample
data.non_atom_changed_filesはpath/status/base_blob/current_blobを含むbounded sampleを返す
data.change_coverageはsummary、optional tool_migration、digestとbounded entry sampleを返し、entries_omitted/entries_truncatedで省略を示す。`--full`は全entryを返す。永続verification/commit certificateはsampleではなく同じ完全contract/digestへ束縛する
--full --jsonはchanged atom IDとopen fireを全量返す
--jsonはchanged atoms、open fires、verifyに進むためのmachine-readable next_actionsを含める
scanの`extinguish_fire` next_actionはready=falseのparameterized executionであり、fire/path bindingとresolutionおよびrationale/evidenceのnamed required_inputsを持つ。resolutionはclosed enumではなくextensible stringである。commandはargv_templateのprojectionであり、placeholderを実行可能とは表示しない
--metricsはtext出力ではCodeFire metrics block、JSON出力ではdata.metricsを追加する
metricsは `total_ms` / `scan_pipeline_ms` / `atom_extraction_ms` / `trace_parse_ms` / `fire_build_ms` の安定phase key、`phases[].measured`、cache status metadataを含める
```

## 4.7.1 Batch Limited YAML Rules

```text
fire --batch、extinguish --batch、link --batch、evidence add --batch の限定YAMLは共通のscalar/comment/key-value parserを使う
コメントはquote外の#から行末まで。double/single quoted scalarは外側quoteを外し、\" と \\ をunescapeする
unsupported section/keyやschema固有validationは各commandで判定し、error messageのcontextはcommandごとのbatch YAML名を使う
```

## 4.9 `fire`

```bash
codefire fire REQ-AUTH-001 --to DES-AUTH-001 --reason "仕様と設計が一致していない可能性がある"
codefire fire --batch fires-batch.yaml --dry-run --json
codefire fire --batch fires-batch.json --path ./main-open --full --json
```

仕様：

```text
manual fireを立てる。source/target Atomはどちらも現在のopen directoryに存在する必要がある
manual fireのdisplay_idはFIRE-<12 uppercase hex>、fire_uidはfire_sha256_<32 lowercase hex>で、scan fireと同じdigest-derived familyを使う
manual fireのsemantic keyはsource atom、target atom、reason、severity、trace path hashから作り、base commitはUIDに含めずplan.current_base_commitに記録する
manual fireはrevertで自動消滅しない
extinguishが必要
--dry-runはfires.jsonやbranch stateを書き換えず、codefire_operation_planを返す。planはitem_count、sample_items、sample_limit、items_omitted、truncatedを含み、defaultでは20件sampleに制限する
--fullはfire operation planへ全item詳細を含める
--batchはJSONまたは限定YAMLのversion/fires形式を読み、全fireのfrom/to/reason/severity、Atom存在、batch内重複、既存fire重複を事前検証してから書き込む
batch defaults.reason/defaults.severityを指定すると各fireの省略fieldへ適用する
適用時はrepository lockを取得後にactive stateを再読込してmanual fireを再計算し、.codefire/active/<branch>/fires.jsonへmanual fireを追記し、open stateをopen-burningへ更新する
fire planのnext_actionsは--path <open_dir>付きcommandとbranch/open_dir/repo_root targetを含める
--jsonはcodefire.command_result.v1 envelopeを出力し、単発はdata.type=codefire_fire_result、batchはdata.type=codefire_fire_batch_resultを含める
```

## 4.10 `extinguish`

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

codefire extinguish --batch-template \
  --resolution addressed \
  --evidence "cargo test --workspace: passed" \
  --output .codefire/fires-to-extinguish.json

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
単発extinguish --jsonのnext_actionsはready=trueのverify actionで、argv/command/targetが同じ`--path <open_dir>`を持つ。targetはpath/open_dir/repo_rootを含む
--resolutionはextensible stringでありclosed enumではない。省略時のparser defaultはaddressed。例: addressed, changed, no-change-required
--idempotency-keyは成功したextinguish resultを.codefire/idempotency/extinguish/へ記録する
同じ--idempotency-keyかつ同じextinguish request payloadは保存済みextinguish resultを返し、新しいledger entryを作らない
同じ--idempotency-keyでfire/resolution/rationale/evidence/evidence-ref/refresh/open_dir/branchが異なるpayloadはexit code 33で拒否する
--interactiveはopen fire、直近evidence object候補、editor付きdraft commandを返す
--interactive --jsonはtype=codefire_extinguish_interactive_planを返す
--batch-templateは現在のopen firesから`extinguish --batch` が受け取れるstrict JSON wrapperを生成する
--batch-template --output <file>はtemplateを書き込み、未指定時はtemplateをstdoutへ出す
--batch-template --jsonはcommand_result envelopeでtemplate本文、item_count、next_actionsを返す
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

## 4.11 `verify`

```bash
codefire verify
codefire verify --details
codefire verify --details --blocking-only
codefire verify --json
codefire verify --details --blocking-only --json
codefire verify --metrics
codefire verify --details --blocking-only --json --metrics
codefire verify --wait-lock --lock-timeout 2s --json
```

仕様：

```text
scanを実行する
live verifyはrepository lock取得後のvalidated scan subjectを使い、read-only preview pathとは分離する
policy checkを実行する
required verification commandを実行する
verification commandのcwdは実行前にopen directory配下へcanonical containmentを確認し、symlink/絶対path/`..`によるescapeはactive publication前に拒否する
verification commandは既定で固定PATH/LANG/LC_ALL/TZとpolicy `env`だけを使う。`inherit_env: true`は明示的なweaker modeで、実行時Unicode environment snapshotを固定してdigestへ束縛する
verification policy rootはsealed historyへ保存されるため、command inline metadataまたは明示`env`にsecret-like値を含むcheckはscan/active publication前にfield値を再表示せず拒否する。secretが必要な実行はsecret-free helperと明示`inherit_env: true`を使い、output redactionを維持する
verification JSON/textは`effective-verification-policy-v1`を返す。各ruleはstable `id`、`source=built_in_default|repository_config|cli_override`、`effective_value`、`applicability`、`safety_class`を持ち、decisionはpolicy/coverage digestを持つ。現行CLIは一般policy override flagを追加せず、model上のCLI precedenceだけを固定する
stdout/stderrはprocess実行中からstreaming drainし、各streamの保存量を`max_output_bytes`へ制限する。timeout時はUnixではprocess group、その他ではdirect childを停止し、scopeを記録する
各execution recordはprofile/phase/trust kind、assessment subject、input/environment digest、repo-relative cwd、worktree before/after digest、resource counters、truncation、timeout/exit、redaction countを持つ。live subject自身もcanonical worktree manifest digestを持ち、実行開始時に一致しなければcommandを起動しない。exit 0でもworktree subjectが変わればblocking failureにする
verification objectをactive stateに保存する
verification/state publicationもactive `state_transaction.json` を使い、最終phaseではpersistしたverificationが今回計算したauthorityと一致し、そのsummary/stateがpersisted scan/firesからの導出と一致する場合だけmarkerを削除する。retryは既存markerを最初のrecovery writeより前に検証し、中断時はdoctor診断と再verifyでhealする
verifyがpersistするopen directory stateは `.codefire/active/<open_instance_id>/scan.json`, `fires.json`, `verification.json`, `state.json` である
verification passed時のopen stateは`open-consistent`、failed時のopen stateは`open-burning`になる
clean状態でverifyしてもopen fireは増えず、statusは`open-consistent`へ進む
resolution.evidence_refsが存在しないevidence objectを指す場合はmissing evidence refsとして失敗する
`--details` 指定時は、失敗したdiagnosticsの代表例を最大5件ずつ出力する
--blocking-onlyはblocking diagnosticsだけを表示/返すことを明示する。JSON data.diagnostic_filterはblocking_onlyになる
require_trace_completeness=falseのmissing_required_linkはseverity=warning/blocking=falseになり、--blocking-only diagnosticsから除外される
--jsonはcodefire.command_result.v1 envelopeを出力し、verification data、policy-aware diagnostics、next_actionsを含める
committableかつpending changesがある場合のcommit next_actionはready=falseで、caller-authored messageをrequired inputとし、open directory pathをargv_templateとtargetに保持する
verification dataは`profile=local_verification_v1`、`phase=post_execution`、versioned `subject`、bounded execution metadataを落とさず返す
verification dataの`decision`は`outcome`、`primary_reason`、順序付き`blocking_reasons` / `warning_reasons`を持ち、既存`result`はoutcomeの互換projectionである
--jsonのexit_codeはprocess exit codeと一致する
verify blockerのexit codeはopen fires=10、missing links=11、stale resolutions=12、missing evidence refs=21、duplicate Atom IDs=13、failed checks=14の優先順で決まる
`require_no_required_fires`、`require_no_stale_resolutions`、`require_verification_success`、`reject_duplicate_atom_ids`の`false`はexit 2のconfiguration errorで、verify scan stateやcommit object/headを変更しない。`require_trace_completeness=false`のwarningだけが例外である
next_actionsはcontext_changed、context_atom、refresh_resolution、rerun_check、commitなどの安定action kindを返す
next_actionsは最大12件に制限され、超過時はnext_actions_omitted actionに省略件数とlimitを含める
--metricsはtext出力ではCodeFire metrics block、JSON出力ではdata.metricsを追加する
metricsは `total_ms` / `verification_pipeline_ms` / `scan_pipeline_ms` / `verification_commands_ms` / `policy_evaluation_ms` の安定phase key、`phases[].measured`、cache status metadataを含める
```

## 4.12 `context`

```bash
codefire context --branch --json
codefire context --changed --json
codefire context --atom REQ-AUTH-001 --depth 2 --json
codefire context --atom REQ-AUTH-001 --depth 2 --limit 25 --json
codefire context --fire FIRE-001 --json
```

仕様：

```text
contextはactive scanが存在する場合も現在worktreeのread-only preview scanを作り、active scan fingerprintと比較する
active scanがfreshならactive stateのscan.jsonを使い、staleならactive stateを書き換えずpreview scanをcontext packに使う
contextはactive stateを書き換えない
--jsonはcodefire.command_result.v1 envelopeを出力し、data.type=codefire_context_packを含める
selectorは--branch、--changed、--atom、--fireのいずれか1つ
--branchはchanged atomsとopen fire endpointsをbranch context selectionとして返す
--atomの--depthはTraceGraph上の近傍探索深さを指定する
--depthは最大8に制限され、--limitはatoms/trace_links/firesの出力上限を指定する
context packはtop-level changed_atoms、open_fires、changed_count、open_fire_countを返し、scan --jsonと同じ主要summaryを読める
context packはlimitsとtruncatedを含め、scan.snapshot_source、scan.active_scan_path、scan.preview_recomputed、scan.active_scan_fresh、scan.active_scan_stale_reason、scan.fire_sourceでactive scan/preview scanの由来を区別する
context packはrepository/branch/open instance/base、scan content、canonical worktree manifest digestを束縛したversion 1 assessment subjectを返し、同じworktreeに対するexplain/verifyとsubject digestを共有する
```

## 4.13 `commit`

```bash
codefire commit -m "Implement login handler"
codefire commit --dry-run --json -m "Implement login handler"
codefire commit --path ./main-open --dry-run --full --json -m "Implement login handler"
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
--dry-run --jsonはcodefire_commit_resultを返し、blockedの場合もok=falseのcommand_result envelopeでverification summaryとnext_actionsを返す
live commitがverification blockerに当たる場合も同じcodefire_commit_resultを一度だけ返し、typed blocker exit、blocked=true、applied=falseとなり、immutable objectやbranch headをpublishしない
commit JSONはtop-level repo、data.open_dir、data.branch、data.changed_atom_countを返す
commit text/JSONはmessage、changed atom count、non-Atom changed file count、typed change coverage、effective policy、extinguished fire count、active resolution count、evidence ref countを要約として返す
changed_atomsはdefaultで50件sampleに制限し、changed_atoms_omittedとchanged_atoms_truncatedを返す。--fullで全件を返す
non_atom_changed_filesもdefaultで50件sampleに制限し、non_atom_changed_files_omittedとnon_atom_changed_files_truncatedを返す。--fullで全件を返す
certificate.resultはverification.resultと同じ語彙を使い、現行成功commitはpassedを返す
certificateはverification rootと同じassessment `subject`、`profile=local_verification_v1`、`phase=pre_commit`を持つ。live commitはverification後、manifest plan後、branch publish直前にsubjectのworktree digestを再照合する。subjectが片側だけにある、variant fieldが不正、異なる、またはidentity digestが不正なsealは拒否する
適用時はactive stateに `commit_transaction.json` をpending markerとして作り、immutable objectごとのphaseとplanned/stored stateを記録する
署名policyがcommit署名をrequireする場合、pending marker作成直後かつ最初のimmutable object plan/storeより前に署名鍵を取得する。鍵が無ければphase=started、objects=[]のmarkerを残し、candidateやbranch headはpublishしない
作業fileをまだstoreせずpure planとして読み、manifest blobごとのpayload/ID、7 sealed roots、candidate commit、candidateを指す次branch snapshotのpayload/IDをすべて計算する。最初のstoreより前にordered `objects` entryを`{role,path,object_type,object_id,state:"planned"}`として一括publishし、candidate aliasも`candidate_commit={object_id,state:"planned"}`とする。同一contentのblob IDが重複してもpathごとのrole entryを残す
各storeは実IDがplanned IDと一致することを確認し、対応entryを`stored`にする。blob/root/candidateをこの順にstoreし、candidateのcomplete sealed validation後にbranch snapshotもstore/markする。その後だけmutable branch refをpublishし、opened registryとactive stateを更新する
中断fixtureは`after_manifest_blob_store_before_marker`、`after_root_store_before_marker`、`after_candidate_store_before_marker`、`after_branch_snapshot_store_before_marker`を持ち、各store済みobjectがまだ`planned`のentry IDから発見でき、branch/opened headが旧値のままであることを固定する
正常完了後は `commit_transaction.json` を削除する。途中失敗で残ったmarkerはdoctorが `pending_commit_transaction` として診断する
--idempotency-keyは成功したcommit resultを.codefire/idempotency/commit/へ記録する
同じ--idempotency-keyかつ同じcommit request payloadは保存済みcommit resultを返し、新しいcommitを作らない
同じ--idempotency-keyでmessage/open_dir/branchが異なるpayloadはexit code 33で拒否する
```

## 4.14 `merge`

```bash
codefire merge feature-login --into main
codefire merge feature-login --into main --dry-run
codefire merge feature-login --into main --dry-run --json
codefire merge feature-login --into main --idempotency-key request-merge-local-001
codefire merge --continue --path ../main-open --dry-run --json
codefire merge --continue --path ../main-open --accept-semantic-conflicts --json
```

仕様：

```text
source sealed headをtarget open directoryへ統合する
source branchはclosedまたはopen-cleanでなければならない
targetはopen-cleanでなければならない
merge実行時点ではcommitを作らない
merge fireを生成してactive stateへpersistし、targetをopen-burningにする
--dry-runは共通祖先、file action、file conflict、semantic conflict candidateを予測し、target file / active state / branch stateを変更しない
--jsonはcodefire_merge_plan envelopeを出力し、repo、file_actions、conflicts、binary_conflicts、semantic_conflicts、next_actionsを含める
--idempotency-keyは成功したmerge resultを.codefire/idempotency/merge/へ記録する
同じ--idempotency-keyかつ同じmerge request payloadは保存済みmerge resultを返し、target open-burning checkより先にreplayする
同じ--idempotency-keyでsource/target/source_head/target_head/base/target_dir/repoが異なるpayloadはexit code 33で拒否する
semantic conflict candidateは同一Atom IDがsource/target双方でbaseから変わり、最終content_hashが異なる場合に報告する
semantic conflict candidateは自動解決しない
text conflictはtarget fileへconflict markerを書き込む
binaryまたは非UTF-8 conflictはtarget fileへlossy markerを書かず、`.codefire-conflicts/<path>/target` と `.codefire-conflicts/<path>/source` に復元用bytesを保存し、binary_conflicts metadataへ記録する
apply後のtext、binary、semantic conflict metadataはVerification.unresolved_merge_conflictsへtyped projectionされ、verify/commitをexit code 15でblockする
text markerを編集で除去しただけではactive conflict authorityを消さない
merge --continueはpending mergeを要求し、text markerがないこと、全binary sidecarが除去されたこと、semantic candidateが--accept-semantic-conflictsで明示承認されたことを検証する
merge --continue --dry-runとblocked resultはfile/active/branch/object authorityを変更しない
live continueはrepository lockとpinned no-follow worktreeの下で検証を反復し、旧verificationをinvalidateしてversion 1 merge_resolution acknowledgementを記録し、conflict arraysを空にしてlocked scanを再実行する
pending_merge_parent/baseはcontinue後も保持し、通常のverify/commitが二親merge commitをsealする
--continue --jsonはdata.type=codefire_merge_continue_result、blocked/applied/already_resolved/conflicts/blockersを持つ単一command_result envelopeを返す
```

## 4.15 `upload`

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

## 4.16 `show` / `diff`

```bash
codefire show feature-login
codefire show feature-login --json
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
show --jsonは `codefire.command_result.v1` envelopeでsealed commit summaryを返す
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
diff --jsonはdiff結果を `codefire.command_result.v1` envelopeで出力し、既存の `codefire_diff` payloadを `data` に保持する。impact有効時はmachine-readable next_actionsを含める
text diff payloadはunified hunk headerを持ち、file単位の最大出力byte数でboundedになり、超過時は省略したchanged line数を表示する
--rename-detectionはexact hash renameを先に検出し、candidate pair数が上限を超えるinexact similarity計算はwarning付きでskipする
```

## 4.17 `review-pack`

```bash
codefire review-pack feature-login
codefire review-pack feature-login --base main --output review-pack.json
codefire review-pack feature-login --base main --algorithm patience --no-rename-detection
codefire review-pack feature-login --base main --output review-pack.json --json
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
subjectはbase/source commit pairを束縛するversion 1 `sealed_commit_pair`で、`trust_kind=sealed_validated`かつimmutableである
--output指定時は指定fileへ書き込み、未指定時はstdoutへ出力する
--json併用時はpayload本体ではなくcodefire.command_result.v1 envelopeでmetadataを返す。data.type=codefire_review_pack_result、output_path、bytes、base/source/options、included_sections、payload_in_envelope=falseを含める
review-packはsealed commitから再現可能な情報だけで構成し、生成時刻などの非決定的値は含めない
```

## 4.18 `storage report`

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
--quickはobject JSON parseを省略し、file数/bytes/largest/large_object warningだけをmetadataから返す。largest objectのtypeはobject ID prefixまたはobject subdirから推定し、`type_confidence` を返す
JSON dataは `mode`、`coverage`、`skipped_checks`、`objects.largest_limit`、`objects.scan` を返し、quick/fullの保証範囲、object scan所要時間、files/bytes、JSON parse件数、invalid record件数をmachine-readableにする
--large-threshold以上のobjectはlarge_object warningとしてdiagnosticsに出す
object JSONが読めない場合はinvalid_object_json warningとしてdiagnosticsに出し、report自体は継続する
ObjectRecordのrecord typeとpayload.typeが異なる場合はpayload_type_mismatch warningとしてdiagnosticsに出す
--remoteはfile-backed remote project URLを追加で集計し、remote側objects/branches/merge_requests/idempotencyのfile数とbytes、gc retention policy、current generation、object generation別容量を返す
requested remoteがmissingまたはrequired layout不足の場合も `remotes[]` に `status=invalid_layout` と `missing_required_paths` を返し、`remote_layout_invalid` warningを出す
remote `server_policy.json` の `gc.idempotency_retention_seconds` が正なら、storage reportはremote idempotency recordのexpired_files/expired_bytes/oldest_created_atを返し、期限超過recordをremote_idempotency_retention warningとして出す
--jsonはcodefire.command_result.v1 envelopeを出力し、data.type=codefire_storage_reportを含める
warningがある場合、next_actionsにinspect_storage_warningsを含める
external_artifactsはartifact_ref objectのrefs、referenced_bytes、payload_bytes_stored=0を返す
artifact_refは外部artifact本体を.codefire/objectsへコピーせず、URI/path/hash/size metadataだけを保存する。defaultでは絶対local pathを保存しない
```

## 4.19 `link --batch`

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
Trace endpointはshared validatorで検証し、`missing_atom`と`ambiguous_atom_id`を区別する。structured endpoint issueはlink_index/link_id/endpoint/atom_id/kind/atom_occurrencesと各sampleのtruncation/omitted-byte fieldを持つ
diagnostic detailは最大100件、link_id/atom_id/from/to/type sampleはUTF-8 boundaryを保って1 field最大256 bytesに制限する。超過時は`diagnostics_omitted`のsummaryとdata.diagnostic_total/data.diagnostics_omitted、各sampleの`*_truncated`/`*_omitted_bytes`で省略を明示する
operation planは全件数を`plan.item_count`、最大100件のprojectionを`plan.items`、残りを`plan.items_omitted`で返す。live successも全追加件数を`data.added_link_count`、最大100件のprojectionを`data.added_links`、残りを`data.added_links_omitted`で返す。どちらのprojectionもfrom/to/typeを256-byte UTF-8-safe sampleと`*_truncated`/`*_omitted_bytes`に変換し、batch原文を無制限に再出力しない
--jsonはcodefire.command_result.v1 envelopeを出力し、dry-run validation resultはdata.type=codefire_link_batch_result、data.valid、data.diagnostics、data.diagnostic_total、data.diagnostics_omittedを含める。既存graphやlive applyのendpoint integrity failureはexit code 21の`trace_graph_endpoint_integrity` diagnosticとしてrepo/path/total_issues/issues_omitted/bounded issuesを返す
```

## 4.20 `evidence add`

```bash
codefire evidence add --artifact runs/model.bin --label "best checkpoint" --json
codefire evidence add --from-command "cargo test --workspace" --json
codefire evidence add --from-argv cargo --argv test --argv --workspace --json
codefire evidence add --artifact runs/model.bin --from-command "python eval.py" --timeout 30s --max-output-bytes 65536
codefire evidence add --from-command "python debug_failure.py" --allow-failed-command --json
codefire evidence add --batch evidence-batch.yaml --dry-run --json
codefire evidence add --batch evidence-batch.json --json
```

仕様：

```text
repository rootを探索し、evidence objectを.codefire/objects/evidenceへ保存する
--artifactはartifact_ref objectを.codefire/objects/artifact_refsへ保存する
artifact_refはpath/path_kind/local_path_redacted/uri/uri_secret_redacted/uri_redacted_value_count/hash_algorithm/content_hash/hash_streaming/hash_chunk_bytes/large_artifact/large_artifact_threshold_bytes/size_bytes/captured_atを持ち、payload本体は保存しない
--artifact-uri指定時はartifact_ref.uriへ保存する。未指定時、repo内artifactはrepo相対pathと`repo://<relative-path>`を保存し、repo外artifactはpathを`<redacted>`、uriを`artifact://redacted/<hash-prefix>`にする
1MiB以上のartifactはlarge_artifact warningをdiagnosticsへ出し、SHA-256は64KiB chunkのstreaming hashとして計算する
--from-commandはshell commandのredacted command/stdout/stderr、exit/timeout/resource metadata、cwd classification、profile/phase/trust/portability、versioned execution subjectを保存する
--from-argvはshellを使わずprogramと--argvで指定した引数配列を直接実行し、同じmetadataとredacted argvを保存する
--from-commandと--from-argvは同時指定できない。--from-commandは`mode: "shell"` / `shell: true`、--from-argvは`mode: "argv"` / `shell: false`として保存する
--timeoutまたは--timeout-msはcommand captureの待機上限を指定する。未指定時は300000ms
--cwd未指定時のcommand cwdは呼び出しprocessのcurrent working directory。--cwd指定時はそのdirectoryをcanonicalizeして使う。repo外cwdはobject/diagnostic/dry-runで`<redacted>`となり`local_only_redacted`、repo相対cwdは`portable_redacted`に分類する
dry-run planとapply resultはredacted/relative command cwd、cwd_source、repo_relative_cwdを返し、repo外artifact/batch pathもdry-runでは`<redacted>`にする。単体/batch planとstored artifact_refはいずれもURI userinfoおよびsecret-like query valueをredactする
command environmentは固定PATH/LANG/LC_ALL/TZ allowlistだけで、caller secret environmentを継承しない。command/argv/stdout/stderrのsecret-like値とexternal/home path literalを保存前にredactし、件数を記録する
stdout/stderrはprocess実行中から読み、--max-output-bytesで各streamの保存量を制限する。bytes seen/storedとtruncated flagを保存し、timeout時は記録されたtermination scopeでchild treeを停止する
execution subject/input digestはraw inputとeffective environment digest、cwd classification、timeout/output capを束縛するが、raw environment値は保存しない
commandがnon-zeroまたはtimeoutになった場合、デフォルトではevidence objectを作らずok=falseの`evidence_command_failed` diagnosticを返す
失敗ログを意図的に証跡化する場合だけ--allow-failed-commandを指定する。この場合はwarning diagnosticと`proof_status=failed-command-captured`を保存する
--jsonはcodefire.command_result.v1 envelopeを出力し、data.type=codefire_evidence_add_resultを含める
--batchはJSONまたは限定YAMLのversion/items形式を読み、全itemのartifact path/cwd/必須fieldを事前検証してからevidence objectを作る
--batch --dry-runはartifact_ref/evidence objectを書き込まず、codefire_operation_planを返す
batch itemはartifact、artifact_uri、from_command、from_argv、cwd、timeout_ms、label、max_output_bytesを持てる。JSON batchのfrom_argvは文字列配列、YAML batchのfrom_argvはinline JSON string arrayで指定する
resolutionから参照されたevidence objectと、そのevidenceが参照するartifact_refはremote object graphに含めてupload/clone/request workflowで搬送する
legacy artifact_refが絶対local pathを含む場合、remote upload planはartifact_path_sensitive warningを返す
```

## 4.21 `explain`

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

## 4.22 `migrate`

```bash
codefire migrate check
codefire migrate check /path/to/repo --json
codefire migrate dry-run /path/to/repo --target-format current --json
codefire migrate check /path/to/repo --quick --json
codefire migrate apply /path/to/repo --target-format current --full --json
```

仕様：

```text
repository rootを探索し、repo.json、object record、branch head sealed commitを検証する
checkは互換性blockerとwarningを返し、互換性がない場合はexit code 40で終了する
target formatはcurrentがdefaultで、accepted compatibility profile listにはcurrentとv0.6を含める。unsupported targetはsupported list付きblockerを返す
target-formatは互換性profileの選択であり、check/dry-run/applyのいずれもrepo.jsonやobject schemaをcurrent/v0.6へ書き換えない。JSONはtarget_format_semantics=compatibility_profile_only、format_write_supported=false、format_write_performed=falseを返す
dry-runはtarget formatへ向けたplanned_actionsを返すが、file systemを書き換えない
applyはfull compatibility scanだけを許可し、blockerがあればlock bootstrapを含むrepository writeを行わない
applyはclean preflight後に.codefire capabilityからmissing locks directoryだけをno-followでbootstrapし、共通のpersistent repository lockを取得してplanを再計算する。repo.lockとowner sidecarはoperational lock stateでありmigrationのdid_write/applied_countには数えない
applyが実行できるのはshared layout registryでAutoCreateとされたrepo-contained create_directoryだけで、required directory、任意path、format/object rewriteは実行しない
locked plan後のdirectory出現、plan差分、repository/.codefire/objects parent identity差分、symlink/non-directoryはfail closedとし、retained no-follow parent capabilityに対してcreateする。2回目のexact applyはapplied_count=0、did_write=falseの成功になる
object recordはobject_id/hash/filenameとpayload canonical digestを再検証する
branch recordはheadがvalid sealed commit graphを指していることを検証する
validated branch head自身にsemantic_validation_profileがない場合はlegacy_semantic_validation_profile warningを返す。current profile bridge headのparentがlegacyでもhead warningは返さない
shared layout registry上のmissing auto-create directoryとobject subdirはplanned_actions=create_directoryとして返す
--quickはobject record integrity scanをskipし、data.scan_mode=quick、data.skipped_checksにobject_record_integrityを返す
--jsonはcodefire.command_result.v1 envelopeを出力し、data.type=codefire_migration_report、current_format、supported_target_formats、scan_mode、skipped_checks、planned/applied/skipped/remaining actionsとcount、would_write、did_writeを含める
```

## 4.23 `patch`

```bash
codefire patch export feature-login --base main --output feature-login.cfpatch.json
codefire patch export feature-login
codefire patch export feature-login --json --max-file-bytes 1048576 --max-payload-bytes 4194304
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
既定ではpatch exportは大きなwrite contentをboundedにし、max_file_bytesまたはmax_payload_bytesを超えるcontentはomit entryへ落とす
omit entryはpath、bytes、sha256、reason、max_file_bytes、max_payload_bytesを持ち、contentを含まない
--include-large-filesを明示した場合だけlarge contentをpayloadへ含める
patch export JSON payloadはsummary、omissions、next_actionsを含む。--json併用時のcommand_result dataはcodefire_patch_export_result metadataであり、payload_in_envelope=false
patch importはopen directory内で実行する
patch importはpatch base commitとopen directoryのcurrent_base_commitが一致しない場合に拒否する
patch import --dry-runはpath検証とbase照合だけを行い、file / active state / branch stateを変更しない
patch importは適用後にopen stateをopen-burningへ更新し、pending_patch_base、pending_patch_source、patch_pathsをactive stateへ記録する
patch import --idempotency-keyは成功したimport resultを.codefire/idempotency/patch_import/へ記録する
同じ--idempotency-keyかつ同じpatch import payloadは保存済みimport resultを返し、file / active state / branch stateを再変更しない
同じ--idempotency-keyでpatch file/content/base/open_dir/branch/repoが異なるpayloadはexit code 33で拒否する
patch pathはmanifest pathと同じく相対pathだけを許可し、open directory外へescapeするpathを拒否する
```

## 4.24 `request-merge`

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
remote request idempotency recordもhash-only保存で、payload全文やoperation planを保存しない。file-backed request-apply は検証済みcompleted receiptのreplayだけ`replayed: true`、`would_apply: false`、空の`operations`を返す。pending/quarantine recoveryはlive mutation planのままであり、receiptなしでも到達できる
request-listのfile-backed JSON `status`は、persistされた`applied`以外でtarget headが`target_head_at_request`から進んだとき`stale`を投影する。`rejected`は再review可能なのでdrift時も`stale`になる
cf+http request-* はserver側remote projectのidempotency recordで同じ規則を適用する
```
