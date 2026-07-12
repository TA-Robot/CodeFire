# ADR-008: 新規Object IDはSHA-256 digestの先頭24 hexを使う

## Status

Accepted

## Context

CodeFire object recordは完全なSHA-256 hashを保持し、外部参照とfilenameには読みやすいprefix付きObject IDを使う。初期実装の12 hex（48 bit）表示は小規模運用では短く扱いやすい一方、大量のartifact、evidence、commitを同じrepositoryで扱うとbirthday bound上の衝突余裕が小さい。

検討した選択肢は次のとおり。

- 12 hexを維持する: 表示は最短だが、大規模自動化へ安全余裕を増やせない。
- 16 hexへ拡張する: 移行コストは24 hexと同種なのに、安全余裕は64 bitに留まる。
- 24 hexへ拡張する: 96 bitの表示digestとなり、可読性を保ちながら衝突余裕を大きくできる。
- 完全な64 hexを表示する: 最大の識別余裕は得られるが、CLI、ログ、文書、手入力での扱いやすさを大きく損なう。

表示prefixだけを完全性の根拠にはできない。readerはrecordの完全hash、payloadからの再計算結果、record内Object ID、filenameを整合させる必要がある。

## Decision

新規object writerは、種別prefixとSHA-256 digestの先頭24 hexからObject IDを生成する。

例:

```text
CF-BLOB-<24 hex>
CF-COMMIT-<24 hex>
```

既存の12 hex Object IDは読み取り互換として残す。ただし、次のすべてが一致するときだけ受理する。

- recordが保持する完全hash;
- canonical payloadとtype tagから再計算した完全SHA-256;
- 完全hashの先頭12 hexから再構成したlegacy Object ID;
- record内 `object_id`、filename、期待するobject格納先。

mixed repositoryでは既存参照文字列を書き換えず、新規objectだけを24 hexで書く。曖昧な短縮ID探索やprefix一致だけのfallbackは導入しない。

## Consequences

- 新規Object IDの表示digestは96 bitになる。
- legacy 12-hex repositoryをin-placeで強制移行せず読める。
- readerはcurrent/legacy双方について完全hash照合を続ける必要がある。
- ID長はrepository formatと互換性に関わるため、将来の変更には新しいADR、fixture、migration/recovery方針が必要になる。
- 現行値とlegacy互換は `docs/08_object_store.md`、source catalog、store testsで機械的・実装的に確認する。
