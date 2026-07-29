# 08. Object Store設計

## 8.1 基本方針

Code FireはGitを使わず、独自のcontent-addressed object storeを持つ。

objectはimmutableである。一度作成したobjectは更新しない。

## 8.2 Object ID

全objectは以下でIDを計算する。

\[
object\_id = sha256(type\_tag \parallel 0x00 \parallel canonical\_payload)
\]

変数の定義：

- \( object\_id \)：objectのID
- \( sha256 \)：SHA-256 hash関数
- \( type\_tag \)：`blob`, `commit`, `trace_graph` などのobject種別
- \( \parallel \)：バイト列の連結
- \( canonical\_payload \)：順序や空白差分を排除した正規化payload

保存されるobject ID文字列は、object種別prefixとSHA-256 digestの先頭24 hexを組み合わせる。

```text
CF-BLOB-<24 hex>
CF-COMMIT-<24 hex>
```

v0.6以前に作られた先頭12 hexのobject IDは読み取り互換として残す。互換読み取りではrecord内 `object_id`、record `hash`、filenameが一致し、payloadから再計算したSHA-256 digestの先頭12 hexと一致する場合だけ有効とする。新規object構築は常に24 hex IDを使う。一方、remote/migration transferは完全なrecordを再検証して、その認証済み12/24 hex public IDをexact-record publicationで保持する。transportの都合でIDと参照edgeを書き換えない。

24/12 hex部分はlookup keyであり完全同一性証明ではない。同じpublic IDのpathが既に存在する場合、既存record自身のhash/id/filenameを検証した後、今回要求したrecordとtype、full 64-hex digest、canonical payloadがすべて一致するときだけidempotent reuseする。どれかが異なる場合はtyped collision errorを返し、既存objectを置換しない。

object recordのserialized sizeはwriter/readerとも64 MiBを上限とする。object rootからsubdirectoryまでabsolute pathをcomponent-wise no-followで開き、readerはrecord名もno-follow/non-blockingで開いてregular fileであることを確認し、metadata sizeとbounded readの両方で上限超過を`ObjectRecordTooLarge` として拒否する。symlink root/subdirectory/record、FIFO/device/socketなどをobject JSONとして読まない。

writerはpinned root/subdirectory内へcreate-new tempを作り、64 MiB以内のrecordをwrite/syncして再読・identity検証した後、hard-linkでfinal名をno-replace publicationする。payloadからcurrent IDを作る通常writerと、validated complete recordのsupported IDを保持するtransfer writerは、この同じpublication pathを使う。既存pathとlink raceの双方でfull identity一致だけを再利用し、publication/reuse後にambient object rootとsubdirectoryが同じidentityであることを再検査する。sealed traversalとfull object scanもroot/subdirectory capabilityをpinし、最後のidentity recheckに失敗すればsnapshot全体を成功扱いしない。

複数recordのprepared publicationはobject rootごとの永続single-link lock
`.codefire-stage-v1.lock`をkernel exclusive lockとして保持し、stageを
`.codefire-stage-v1.<object-id>.<pid>.<counter>.tmp`へ限定する。cleanup失敗後に固定retry応答へ
移る場合、stageはobject authorityへ昇格させず残してaccounting ownerだけを明示的に解放できる。
次の同subdirectory batchは同じlock下でreserved namespaceをbounded/poll可能に列挙し、全候補を
ledgerへ計上してregular/single-link/size/name/object-id/hashを検証し終えるまで一件も削除しない。
unlink直前にもname inodeを再照合する。unknown/future/malformed/nonregular/hardlinked/content不一致、
または途中でname identityが変わった候補は保存する。

object recordのidentity外top-level metadataはcomplete documentとして扱う。標準`extensions` mapの
descriptorはnamespace/name/version/criticalと任意のreferences/payloadを持つ。未知のcritical extensionは
利用前に拒否し、未知のnoncritical extensionはRust/Python/file/HTTP transferでJSON valueをexactに保持する。
descriptorが宣言する`CF-*` referencesはsealed reachability、clone/upload/bundle、remote Doctor、GCの
object graphへ加える。既存immutable recordの再利用はpayloadだけでなくextension mapも完全一致する場合に
限り、暗黙のmergeやdropを行わない。

## 8.3 主要object

```text
BlobObject
ContentManifestObject
ArtifactObject
AtomIndexObject
TraceGraphObject
FireObject
ResolutionObject
VerificationObject
CommitObject
BranchObject
```

## 8.3.1 ArtifactRefObject

外部artifactのmetadataだけを保存し、payload本体はobject storeへコピーしない。新規artifact_refはdefaultで絶対local pathを保存しない。

```json
{
  "type": "artifact_ref",
  "version": 1,
  "label": "training-smoke",
  "path": "artifacts/model.bin",
  "path_kind": "repo_relative",
  "local_path_redacted": false,
  "uri": "repo://artifacts/model.bin",
  "hash_algorithm": "sha256",
  "content_hash": "sha256:aaa111",
  "hash_streaming": true,
  "hash_chunk_bytes": 65536,
  "large_artifact": false,
  "large_artifact_threshold_bytes": 1048576,
  "size_bytes": 1234,
  "captured_at": "2026-06-06T00:00:00Z"
}
```

repo外artifactはdefaultで `path: "<redacted>"` / `path_kind: "redacted"` / `local_path_redacted: true` として保存する。

## 8.4 ContentManifestObject

commit時点の全ファイル一覧を表す。

```json
{
  "type": "content_manifest",
  "version": 1,
  "entries": [
    {
      "path": "docs/spec/auth.md",
      "kind": "file",
      "mode": "100644",
      "blob": "CF-BLOB-222bbb"
    }
  ]
}
```

`.codefire-open` はmanifest対象外である。

## 8.5 AtomIndexObject

commit時点のArtifact Atom索引を表す。

```json
{
  "type": "atom_index",
  "version": 1,
  "atoms": [
    {
      "atom_id": "REQ-AUTH-001",
      "kind": "requirement",
      "artifact_path": "docs/spec/auth.md",
      "selector": {
        "type": "markdown_heading",
        "value": "REQ-AUTH-001"
      },
      "content_hash": "sha256:aaa111"
    }
  ]
}
```

## 8.6 CommitObject

sealed commitを表す。

```json
{
  "type": "commit",
  "version": 1,
  "root_schema_version": 2,
  "semantic_validation_profile": "sealed-snapshot-v1",
  "parents": ["CF-COMMIT-21ab77aa0011223344556677"],
  "message": "Change session expiration policy",
  "roots": {
    "content_manifest": "CF-MANIFEST-aaa111aaa111aaa111aaa111",
    "atom_index": "CF-ATOMINDEX-ccc333ccc333ccc333ccc333",
    "trace_graph": "CF-TRACE-ddd444ddd444ddd444ddd444",
    "fire_delta": "CF-FIRELEDGER-eee555eee555eee555eee555",
    "resolution_ledger": "CF-RESOLUTION-bbb222bbb222bbb222bbb222",
    "verification": "CF-VERIFY-fff666fff666fff666fff666",
    "policy": "CF-POLICY-999777999777999777999777"
  },
  "certificate": {
    "result": "passed",
    "profile": "local_verification_v1",
    "phase": "pre_commit",
    "subject": {
      "type": "codefire_assessment_subject",
      "version": 1,
      "kind": "live_open",
      "trust_kind": "local_unisolated",
      "subject_digest": "sha256:...",
      "content_digest": "sha256:...",
      "worktree_digest": "sha256:...",
      "immutable": false,
      "repository_id": "repo-...",
      "branch": "main",
      "open_instance_id": "open-...",
      "base_commit": "CF-COMMIT-..."
    },
    "open_required_fires": 0,
    "failed_checks": 0,
    "missing_required_links": 0,
    "stale_resolutions": 0,
    "missing_evidence_refs": 0,
    "duplicate_atom_ids": 0
  },
  "changed_atoms": ["REQ-AUTH-001"],
  "extinguished_fires": ["fire_sha256_..."],
  "created_at": "2026-06-04T00:00:00Z",
  "signature": {
    "type": "commit_signature",
    "version": 1,
    "algorithm": "hmac-sha256",
    "signer": "alice",
    "key_id": "alice",
    "signed_at": "2026-06-04T00:00:00Z",
    "signature": "..."
  }
}
```

commit payloadは自身の`commit_id`を持たない。object recordの`object_id`とbranch headがcommit identityの正本である。`signature` は任意である。署名対象はcommit payloadから `signature` フィールドを除いた内容と、`signature.signature` を除いた署名metadataである。これによりcommit object IDのcontent-addressed性を保ったまま、commit IDそのものへの循環署名を避ける。

現行Rust writerの契約は`commit.version=1` / `root_schema_version=2` / `semantic_validation_profile="sealed-snapshot-v1"`で、上記7 rootsもすべて`version=1`である。ADR-019以降のwriterはpolicy/verification rootへ同一`effective-verification-policy-v1`、verificationへ`file-relation-v1` coverage、certificateへ両digestを追加する。これらはdisclosure-only additive claimで、historicalな両方欠落はclaimなしとして読むが、一方だけのroot/certificate、rule/value/digest、coverage classification/summary/digest driftはfail closedする。`root_schema_version`欠落はlegacy schema 1として読み、`resolution_ledger`以外の旧6 rootsを必須とする。schema 1/2以外、commit version 1以外、root version 1以外は未知future dataを既知形式へdowngradeせずfail closedする。schema 2の`verification.missing_evidence_refs`は空の場合だけ省略可能で、certificate側の同countもverification countが0の場合だけ省略できる。

sealed commit validatorはrequired root/type/shape、reachable reference、parent履歴を全profileで検証する。`sealed-snapshot-v1`はAtom indexとTrace Graph endpointの一意参照も要求する。profile欠落は`legacy-unversioned`として、当時存在しなかったendpoint cross-root predicateだけをclaimしない。schema 2ではprofileにかかわらずAtom/Trace/Fire/Resolution/verification/policyをdomain typeへdecodeし、`codefire-core::validate_sealed_snapshot_semantics`を呼ぶ。coreはopen required Fire数、duplicate Atom ID、missing required link、stale Resolution、trace-completeness、verification result、Fire/active ResolutionのUID/cardinality/identityを再計算・照合し、storeが最後にcertificate result/countとassessment subjectをverificationへ照合する。subjectが存在する場合はtype/version、digest label、kindごとのtrust/immutability/required field、未知field不在、identity fieldsからのsubject digestも再計算する。`live_open`はworktree digestを必須とし、`sealed_commit_pair`はmutable worktreeをclaimしない。unknown/non-string profileはfail closedし、parentは各commit自身のprofileで読む。branch headやscan base、commit candidateとして消費する前にこのvalidatorを通す。暗号学的な署名policy検証はstore validatorではなくCLI `signatures` 境界の責務である。詳細はADR-012/016。

sealed validationはobjects rootと12 built-in object subdirectoryのidentity/absence、および現在存在するextension subdirectoryをpinned capabilityとしてbindし、recordをsubdirectory handleからrelative/no-follow readする。extensionは128 directoryまでで、超過はhandleをcacheへ残す前に`InvalidSealedCommit`でfail closedする。command-scoped cacheはrootとordered child identity/missing setをsubjectに含め、current subject 1つだけを保持する。別subjectへ移ると旧pin/resultをevictし、後で戻れば再検証する。validation後もcommit/rootをconsumeまたはmutable authorityをpublishするcallerは`ValidatedSealedCommit`を保持する。tokenはvalidationと同じroot/subdirectory handle setを所有し、commit/root/reachable objectをdescriptor-relativeに読み、consume前後およびpublication直前/直後にambient identityを再検査する。同一contentを同じpathへ置き直してもdirectory identityが異なればfail closedする。ただし、同一file inode上のin-place rewriteは検知対象ではなく、published objectのimmutabilityはcooperative store contractである。

mutable authorityをpublicationする経路では、成功した未finalize sessionをpublication rootへbindして
`PrepublicationSealedValidationProof`へmoveする。planning viewは同じsubjectで検証・保持したrootとその
commit祖先だけをborrowし、別subjectのcache entryや未検証commitを返さない。このviewから導いた結果はまだ
publish可能ではない。proofがbound root membership、pinned store identity、全retained raw documentのsize/digest、
closing identityを再検証し、同じ`DecisionWorkLedger`を一度だけprepublication sealした後に限って、objectと
mutable authorityのcommitへ進める。これによりplanning用の再読を最終検証後へ置いてtamper gapを作らず、
中間祖先を含むsemanticsとpublication時のraw snapshotを同じ証明へ束縛する。

## 8.7 object保存タイミング

未commitファイル内容はobject storeへ保存しない。

```text
scan時:
  Atom hashやfire metadataはactive stateに保存してよい
  作業中ファイル内容はblob化しない

commit時:
  作業中ファイルとderived payloadから全object IDを先に計算する
  manifest blob, 7 sealed roots, candidate commit, branch snapshotの
  ordered role entryを最初のstore前にplannedとしてtransaction markerへ書く
  各objectを初めて保存し、planned IDと一致すればentryをstoredにする
  candidate sealed validation後にbranch snapshotを保存し、その後にmutable branch refをpublishする
```

これはcheckpointなしの思想を内部実装でも守るための重要ルールである。
