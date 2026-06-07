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

v0.6以前に作られた先頭12 hexのobject IDは読み取り互換として残す。互換読み取りではrecord内 `object_id`、record `hash`、filenameが一致し、payloadから再計算したSHA-256 digestの先頭12 hexと一致する場合だけ有効とする。新規書き込みは常に24 hex IDを使う。

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
  "commit_id": "CF-COMMIT-8f3a91d0c2",
  "parents": ["CF-COMMIT-21ab77aa00"],
  "message": "Change session expiration policy",
  "roots": {
    "content_manifest": "CF-MANIFEST-aaa111",
    "atom_index": "CF-ATOMINDEX-ccc333",
    "trace_graph": "CF-TRACE-ddd444",
    "verification": "CF-VERIFY-fff666",
    "policy": "CF-POLICY-ggg777"
  },
  "certificate": {
    "result": "consistent",
    "open_required_fires": 0,
    "failed_checks": 0,
    "missing_required_links": 0,
    "stale_resolutions": 0,
    "duplicate_atom_ids": 0
  },
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

`signature` は任意である。署名対象はcommit payloadから `signature` フィールドを除いた内容と、`signature.signature` を除いた署名metadataである。これによりcommit object IDのcontent-addressed性を保ったまま、commit IDそのものへの循環署名を避ける。

## 8.7 object保存タイミング

未commitファイル内容はobject storeへ保存しない。

```text
scan時:
  Atom hashやfire metadataはactive stateに保存してよい
  作業中ファイル内容はblob化しない

commit時:
  初めて作業中ファイルをBlobObjectとして保存する
  manifest, atom index, trace graph, verification, commitを保存する
```

これはcheckpointなしの思想を内部実装でも守るための重要ルールである。
