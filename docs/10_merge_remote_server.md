# 10. Merge / Remote / Server設計

## 10.1 mergeの意味

Code Fireのmergeは、source branchのsealed headをtarget branchのopen directoryへ統合し、必要なfireを立てる操作である。

merge実行時点では正式履歴にcommitを作らない。

```text
merge
  ↓
target open directoryがopen-burningになる
  ↓
text/binary conflictを解消し、semantic candidateを確認する
  ↓
merge --continueを実行する
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
3. 3-way mergeを行う（text fileは行単位、それ以外はfile単位）
4. Atom-level変更を計算する
5. TraceGraph変更をmergeする
6. 両側が異なる内容に変えた領域があれば、その領域にconflict markerを出す
7. semantic conflict候補を検出する
8. merge fireを作る
9. targetをopen-burningにする
```

text、binary、semantic conflict metadataはactive stateのauthoritative unresolved factであり、
Verificationへtyped projectionされる。markerを編集しただけ、またはsidecarを残したままでは
verify/commitはexit 15でblockする。

```bash
codefire merge --continue --path ../main-open --dry-run --json
codefire merge --continue --path ../main-open --accept-semantic-conflicts --json
```

continueはtext markerの除去、binary sidecarの除去、semantic candidateの明示承認を検証する。
成功後もpending source parentを保持し、通常のverify/commitが二親commitを作る。semantic conflictの
自動解決は行わない。

`--dry-run` では 1-7 だけを実行し、予測結果をoperation planとして返す。file書き込み、conflict marker書き込み、active state更新、branch state更新、fire生成は行わない。

dry-run planには以下を含める。

```text
source/target/base commit
target open directory
write_source / delete_target / write_conflict_markers のfile action
file conflict一覧
semantic conflict candidate一覧
machine-readable next_actions
```

semantic conflict candidateは、同一Atom IDがsourceとtargetの双方でbaseから変更され、source headとtarget headのcontent_hashが異なる場合に報告する。v0.6では候補報告に留め、自動semantic mergeや自動解決は行わない。

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

object graphは各recordのfull digest/payload/public IDを再検証してcopyする。validなlegacy
12-hex IDは12-hexのまま、current 24-hex IDは24-hexのまま保持し、clone transportがsealed
parent/root/reference edgeを書き換えることはない。

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

uploadできるのはsealed branchのみである。uploadもcloneと同じexact-record transferを使い、
認証済みlegacy/current public IDを再導出しない。

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

既存remote projectへのHTTP uploadは、request認証後にexclusive project gateを取り、publication-aware nonce
preflight、incoming overlayのsealed validation、branch ancestry/commit signature、object batch、branch/generation/
nonce/receipt、通常成功/固定durable-retry responseを一つのrequest ledgerとprepared mutation ownerへ閉じる。
completed receiptとmatching pendingではFresh用のancestry、policy、postimage、object batchを再構築しない。
Freshだけがそれらを計画し、object batch、JSON authority、Store proofの順で最終再検証して一度だけsealする。
新規projectの初回uploadは、同じownerがhidden bootstrap treeを完成させた後、directory renameを唯一の可視化操作に
しなければならない。既存project用routeが先にpublic layoutを作成してこの境界を迂回してはならない。

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

remote branch recordとmerge-request recordは現行`version: 1`を持つ。file/HTTP readerはversionを
semantic fieldより先に検査し、欠落またはfuture versionを暗黙に現行へ解釈しない。branch list responseも
versionを明示する。

## 10.8.1 Remote diagnostic and pagination contract

serverは`GET /v1/projects/{org}/{app}/doctor`で`codefire.remote_doctor.v1`を返す。missing project、
layout、branch/MR version、object integrity、extension reference、sealed rootをread-onlyで診断し、issue
sampleとrecord scanには上限を設ける。file-backedとHTTP(S)は同じreport shapeを使う。

branch/MR listのpagination cursorはoffset単体ではなく、collection identity、complete ordered resultの
snapshot digest、offset、limitを束縛する。mutation後のcursor replayは結果を混在させずstaleとして拒否する。

## 10.9 remote mutation trust subject

file-backed `cf://` とHTTP `cf+http://` のupload / request-merge / request-review / request-apply / gcは、
policyが要求する場合、version 2 HMAC request signatureを検証してから最初のremote writeへ進む。
`codefire_remote_mutation_subject` version 1はactor、operation、target、key ID、UTC timestamp、nonce、
idempotency keyと、`request_signature`だけを除いた完全canonical request payloadのSHA-256を束縛する。
head、object record内field、lock optionを含む任意のpayload変更はtransport共通verifierでpre-write failureとなる。

request key objectのstatus/signers/`not_before`/`not_after`はsigned timestampに対して評価する。nonce cacheは
actor/key単位でpayloadが異なる同一nonceもreplayとして拒否し、構造が正しいTTL-expired entryだけを除去する。
malformed cache/entryは再初期化せず保存したままfail closedし、`max_nonce_cache_entries`がすべてliveなら新規requestをfail closedする。
容量確保のためlive nonceをevictしないため、既知replayはTTL中忘れない。これはproject-local file-backed cacheであり、
複数server node間のdistributed replay protectionはclaimしない。canonical bytesはRust/Python/file/HTTPで共有fixtureを持つ（ADR-016）。

HTTP `request-apply`は、project gate前にrequest HMAC/key policyを検証し、gate取得後にpublication receiptと
pending requestを考慮したnonce preflightを行う。異なるpayloadのnonce再利用とmalformed/saturated cacheは、
branchのfast-forward判定より先に拒否する。一方、同じrequest hashのcompleted receiptまたはmatching pendingを
replay/recoveryするときは、後から変更されたMR policy・commit signature policy・authority postimageを再構築しない。
ancestry、commit signature、branch/generation/MR postimageの計画はFresh requestだけが行う。Fresh/replay/recoveryの
いずれもStore proof、nonce cache、authority JSON、通常成功responseと固定durable-retry responseを一つのrequest ledgerと
prepared mutation ownerへ閉じ、全fallible raw revalidation後に一度だけsealしてから選択した応答をwriterへ渡す。
HTTP branch bundleはshared project gate内でsealed headを検証し、reachable objectをLIFO reference scheduleで
収集して最終public ID順に並べる。node/edge/depth/frontier/object/aggregate/retained workは一request ledgerへ
累積し、limit超過はatomic bundleのpartial successではなくtyped 413/exit 60となる。JSON bodyも16 MiB上限を
header送信前に検査し、limit+1で`200` bytesを送らない。
Python fallbackもread-only bundle/clone GETに限り、handler ledgerをbranch authority、iterative sealed validation、
reachable chunked read/hash、canonical walk、snapshot fence、result owner、canonical responseまで同一identityで移管し、
同じdeadlineをresponse publicationまでpollする。ただしstdlib JSONのcontiguous parse/materializationはcallbackで
中断できず、full deadline supportは未成立である。mutation response preflightとconnection admission、および
request header/body・他routeのdeadlineは別sliceであり、この記述はそれらのsupport claimではない。
