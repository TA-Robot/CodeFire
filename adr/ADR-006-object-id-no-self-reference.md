# ADR-006: Object IDは自己参照フィールドをpayloadに含めない

## Status

Accepted

## Context

CodeFire object IDは `sha256(type_tag || 0x00 || canonical_payload)` で計算する。

commit objectのpayloadに自身の `commit_id` を含めると、ID計算が循環する。

## Decision

Object IDはobject recordの `object_id` として保持し、content-addressed payloadには自身のIDを含めない。

branch head、manifest entry、commit roots、remote branch headなどの外部参照は `CF-*` object IDを指す。

commit表示では、object recordの `object_id` をcommit IDとして表示する。

## Consequences

- object ID計算が一意で単純になる。
- commit payloadは自己参照を持たない。
- 設計例にある `commit_id` フィールドは表示上の説明であり、正本はobject recordの `object_id` である。
