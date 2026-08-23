# 05. 整合性モデル

## 5.1 整合の定義

Code Fireにおける整合は、絶対的な正しさではない。

Code Fireにおける整合とは、定義されたpolicyのもとで、未解消の確認責任がなく、必須リンクが存在し、検証に成功している状態である。

## 5.2 commit可能条件

作業状態を \( W \) とする。

変数の定義：

- \( W \)：現在のopen directory上の作業状態
- \( F_{open}(W) \)：状態 \( W \) に存在する未解消required fireの集合
- \( C_{fail}(W) \)：状態 \( W \) で失敗している検証の集合
- \( L_{missing}(W) \)：policy上blockingとなる、状態 \( W \) で欠落している必須Trace Linkの集合
- \( R_{stale}(W) \)：状態 \( W \) で古くなったresolutionの集合
- \( E_{missing}(W) \)：状態 \( W \) のactive resolutionが参照するmissing/invalid evidenceの集合
- \( D_{dup}(W) \)：状態 \( W \) で重複しているAtom IDの集合
- \( M_{unresolved}(W) \)：状態 \( W \) に記録された未解消text/binary/semantic merge conflictの集合

Code Fireでcommit可能である条件は以下である。

\[
committable(W) =
|M_{unresolved}(W)| = 0
\land |F_{open}(W)| = 0
\land |C_{fail}(W)| = 0
\land |L_{missing}(W)| = 0
\land |R_{stale}(W)| = 0
\land |E_{missing}(W)| = 0
\land |D_{dup}(W)| = 0
\]

これはworkspaceの整合性predicateである。`commit_policy.require_commit_signature=true`の場合、commit applyにはこれに加えて署名鍵が利用できることが必要である。現行writerはpending commit markerを作成した直後、最初のimmutable object writeより前にこの前提を確認する。

Rust defaultでは、unresolved text/binary/semantic merge conflict、open required Fire、failed required check、stale resolution、missing evidence、
duplicate Atom IDはpolicyで無効化できないsafety floorである。version 1 policy objectに残る
`require_no_required_fires`、`require_no_stale_resolutions`、`require_verification_success`、
`reject_duplicate_atom_ids`は互換fieldだが、working configで`false`を指定するとconfiguration error
になる。`require_trace_completeness=false`だけはmissing required linkをfactとして残したwarningへ
変更でき、そのときの \(L_{missing}(W)\) はpolicy上blockingなlinkだけを表す。

coreはraw factsから順序付き`CommittableDecision`を一度だけ構築する。process exit、JSON、diagnostic、
next action、human summary、open state、commit gate、certificate、sealed validationはこのdecisionを
projectionし、各surfaceでcommittableを再定義しない。理由優先順はunresolved merge conflict、open Fire、missing link、stale
resolution、missing evidence、duplicate Atom、failed checkである。一般decisionはADR-013、merge fact/continue transitionはADR-014を参照する。

## 5.3 open state

branchは以下の状態を持つ。

```text
closed
open-clean
open-burning
open-consistent
```

### closed

branchは存在するが、実ディレクトリは存在しない。

### open-clean

実ディレクトリが存在し、branch headと一致している。

### open-burning

実ディレクトリに変更または未解消fireがあり、commit不可である。

### open-consistent

変更はあるが、未解消fireがなく、検証に成功しており、commit可能である。

## 5.4 状態遷移

```text
closed
  │ open
  ↓
open-clean
  │ edit / scan / fire / merge
  ↓
open-burning
  │ extinguish / verify
  ↓
open-consistent
  │ commit
  ↓
open-clean
  │ close
  ↓
closed
```

## 5.5 stale resolution

resolutionは、fireを解消した時点のsource atom、target atom、trace link、policyに対する判断である。

以下のいずれかが変わった場合、resolutionはstaleになる。

```text
source atom hash
target atom hash
trace link hash
policy hash
```

stale resolutionが1つでもある場合、commit不可である。

## 5.6 commit certificate

commit時には、consistent certificateを生成する。

certificate objectは以下を含む。

```text
result
subject
profile
phase
open_required_fires
failed_checks
missing_required_links
stale_resolutions
missing_evidence_refs
duplicate_atom_ids
effective_policy_digest
change_coverage_digest
```

commitの`roots` はcontent manifest、Atom index、Trace Graph、Fire/resolution ledger、verification、policyのobject IDを別に持つ。certificateはverification rootと同じblocker count/result、versioned assessment subject、effective-policy digest、change-coverage digestであることをsealed validatorで照合し、`profile=local_verification_v1` / `phase=pre_commit`として、そのcommitが参照policyのもとでどのsubject/coverageを評価したかを示す。policy rootとverification rootのeffective policyも完全一致しなければならない。live subjectはcanonical worktree manifest digestを含み、commitはverification後、manifest plan後、branch publish直前に再照合する。historical recordではpolicy/coverageを両root/certificateから省略した弱いclaimを許すが、片側だけ、variant fieldの矛盾、値の不一致、またはidentity digest不一致はfail closedする（ADR-016/019）。

現行Rust writerは`commit.version=1` / `root_schema_version=2` / `semantic_validation_profile="sealed-snapshot-v1"`とversion 1の7 rootを生成する。schema 2のsealed validationはrootをdomain typeへdecodeした後、`codefire-core::validate_sealed_snapshot_semantics_with_contract`を唯一のsnapshot predicateとして呼ぶ。このpredicateはVerificationのunresolved merge fact、Fire ledgerからopen required Fire数、Atom indexからduplicate ID、Atom/Trace/policyからmissing required link、Resolution basisからstale resolutionを用い、ADR-013/014のsafety floorとpolicy trace-completenessからtyped outcomeを再構築してverification resultを照合する。さらにeffective policyのprofile/order/ID/value/digestと両root一致、change coverageのpath/Atom relation/classification/summary/digest、Fire UID、active Resolution UID、Fire対active Resolutionのcardinalityと`resolution_uid`対応を検査する。storeはその結果に加えてcertificateのresult/count/policy/coverage digestをverificationへ照合するため、個々のrootがshape-validでも相互に矛盾するsealは受理しない。

`root_schema_version`欠落はlegacy schema 1として読み、`resolution_ledger`を除く旧required-root集合と緩和されたverification/certificate shapeを維持する。schema 1/2以外、`commit.version != 1`、またはroot objectの`version != 1`はfuture schemaを既知形式へ読み替えずfail closedする。schema 2でも互換上、`verification.missing_evidence_refs`と`verification.unresolved_merge_conflicts`は空なら省略可能である。前者のcertificate countもverification上の件数が0の場合だけ省略でき、後者はzero-only active factなのでcertificate countを追加しない。非空のblockerやcount不一致を省略で消すことはできない。

semantic profileはroot schemaと別軸である。`sealed-snapshot-v1`はAtom/Trace endpoint closureを
要求し、profile欠落はそのpredicate導入前の`legacy-unversioned` read contractを表す。
legacyでもobject identity、root/reference、schema-2 domain/certificate agreementは省略しない。
unknown profileはfail closedし、新writerはprofileを省略しない。strict childはlegacy parentを
保持できるが、legacy parentの成功をcurrent endpoint-integrity proofへ読み替えない（ADR-012）。

## 5.7 Durable Write And Commit Transaction

CodeFireの永続化は、immutable object writeとmutable metadata publishを分けて扱う。

local/import repository mutationはpersistent `repo.lock`のadvisory kernel ownershipを取得してから行う。lock inodeはregular/no-follow/exactly-one-linkでcontent不変とし、取得前後とmutation use boundaryでpathname identity/link provenanceを再検証する。`repo.lock.owner.json`はlock取得後にatomic replaceするbest-effort diagnosticであり、ownership authorityではない。unlockではどちらも削除しない。

- object store writeはcontent-addressed object recordをtemp fileへ書き、file sync後にpublishする。既存objectは再利用される。
- branch head、opened registry、active stateなどのmetadata writeはatomic JSON write helperを通す。
- commitは複数root objectを書いてからbranch headを進めるmulti-object operationである。

CLIのgeneric repository JSON helperはread/write双方を64 MiBで対称に制限する。writerはserialize後のsizeをtemp publication前に拒否する。readerはabsolute parentをcomponent-wise no-followで開き、final fileもno-follow/non-blocking regular-fileとしてmetadata上限とlimit-plus-one readの両方を検査するため、symlink、FIFO/device/socket、read中に上限を越えたfileをrepository stateとして採用しない。

commit適用時は `.codefire/active/<open_instance_id>/commit_transaction.json` をpending markerとして作成する。markerはtransaction_id、phase、parents、branch、message、ordered object-role entries、started_at、updated_atを持つ。署名必須policyの鍵検証はmarker作成直後に行い、失敗時は`phase=started` / `objects=[]`を診断証拠として残す。

writerは最初のstoreより前に、作業fileごとのmanifest blob、7 sealed roots、candidate commit、次branch snapshotのpayloadとobject IDを計算する。`objects` は次のordered entryで、初期stateはすべて`planned`である。

```json
{
  "role": "manifest_blob|content_manifest|atom_index|trace_graph|fire_ledger|resolution_ledger|verification|policy|candidate_commit|branch_snapshot",
  "path": "repo-relative path or null",
  "object_type": "blob|content_manifest|...",
  "object_id": "CF-...",
  "state": "planned|stored"
}
```

同一contentを持つ複数pathが同じblob IDになっても、復旧時のpath/role数を失わないようentryは個別に持つ。各store後に実IDとplanned IDを照合し、そのentryだけを`stored`にする。blobとroots、candidateを保存したcandidate sealed validationに成功した後、branch snapshotも保存してからmutable branch refをpublishする。

正常commitでは、branch headとopened registryを更新し、active stateをresetした後にtransaction markerを削除する。したがって成功後のactive stateには `commit_transaction.json` が残らない。

途中失敗時はmarkerが残り、doctorが `pending_commit_transaction` としてphase、stored role-entry数、planned role-entry数を報告する。legacy stringのobjects entryはstoredとして数える。残ったobjectはimmutableであり、branch headから参照されていない限り未完了commit由来の残骸として扱う。将来GCはこのmarkerを見て未完了operationを説明できる必要がある。

現行regressionは、manifest blob、sealed root、candidate、branch snapshotそれぞれのstore後かつ`stored`更新前、candidate validation直前の中断と、certificate/root type/missing root/parent不正によるpre-publication rejectionを固定している。store-gap fixtureはstored objectがplanned entry IDから発見でき、branch/opened headが不変であることも確認する。これは全write phaseのcrash matrixを網羅したというclaimではない。

## 5.8 Scan / Verify State Transaction

live `scan` / `verify` はrepository kernel lockの下で、最初のderived-state writeより前に `.codefire/active/<open_instance_id>/state_transaction.json` を作成する。markerはversion、pending status、operation、現在phase、intended state、started_at、updated_atを持つ。

phase列は以下である。

```text
scan:   started -> verification_invalidated -> fires_written -> scan_written
        -> registry_written -> active_state_written -> branch_written
verify: started -> verification_written -> registry_written
        -> active_state_written -> branch_written
```

writerのphase updateはこの隣接transitionだけを認める。doctorとretryは保存された現在phaseがoperationの既知集合に属することは検証するが、markerが過去に隣接transitionを通ったことまで復元して証明するものではない。

retryは既存markerのshapeとoperation-phase対応を最初のrecovery writeより前に検証し、supported markerであれば既存のpartial derived stateを信頼せず再計算する。scanは旧verificationを最初のphaseでinvalidateする。最終phaseではregistry/active state/branchのstate文字列だけでなく、persistしたfires/scan/verificationが今回計算したauthorityと一致すること、`scan.open_fires`がfiresのopen subsetであること、scan base・registry base・branch headが一致すること、persisted authorityから導出したstateがintended stateと一致することを検証してからmarkerを削除する。このプロトコルはpartial writeをrollbackするものではなく、中断を診断可能にしてretryでhealする。

現行regressionはscan/verifyそれぞれについて全phase直後の中断とretry、pending verifyをscanが引き継ぐ場合、unknown/cross-operation phaseを最初のwrite前に拒否する場合、最終authority divergenceでmarkerを保持する場合を固定する。

## 5.9 Close / Discard Transaction

live `close` / `discard` はrepository lockの下で、worktreeを含む最初の削除より前に `.codefire/close_transaction.json` をno-replaceでpublishする。markerはrequest、repository/branch/open-instance、branch before/after、worktree marker、active state、opened registryのexact preimage、target/parent filesystem identity、transaction-derived quarantine、resultを保持する。

phase列は以下である。

```text
prepared -> worktree_applied -> active_state_removed
         -> registry_removed -> branch_written
```

各applyは `before -> quarantined -> removed` をidentity-boundに進め、その後でだけphaseを隣接更新する。retryはphaseとartifactの組合せを照合し、apply後phase前のgapだけを受け入れる。branchはimmutable postimage objectを先に保存/検証してからmutable referenceを置換するため、そのobjectの先行存在はprepublication phaseの正当な証拠である一方、postimage referenceがobjectなしで存在する状態はcorruptionである。成功はfixed completed archiveを先にpublishし、その記録が束縛するpending markerだけをquarantine/unlinkする。

pending closeは他のmutationより優先されるretry authorityであり、同じbranch/command/discard/keep-dir requestだけがresumeできる。pending openとの同時存在は自動で順序を推測せずconflictとしてfail closedする。dry-runはpending evidenceを検証・報告するがphase進行、cleanup、archive retirementを行わない。
