# RVC-021: public Object ID と full digest の同一性境界

## Review Summary

| Field | Value |
|---|---|
| Status | implemented |
| Perspective | correctness / security / evolution |
| Review kind | refine |
| Priority | P0 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `crates/codefire-store/src/lib.rs / object identity` |
| Promotion target | existing CFB/CFR |

## Executive Judgment

ObjectRecord が64桁の full digest を保存し、読み取り時に payload から再計算する設計は維持すべきである。一方、24桁の public ID は検索キーであって同一性証明ではないため、既存pathを再利用する2経路で「既存recordと今回要求したrecordの full digest が同じ」を必須条件にする必要がある。この候補は新規issueではなく CFB-125 の設計上位説明として吸収する。

## Scope and Review Question

- In scope: immutable object の生成、既存object再利用、legacy 12桁ID読取、同一prefix衝突
- Out of scope: hash algorithm変更、暗号署名、remote認証
- Question: public prefix衝突が起きても、異なるpayloadが同一objectとして成功しないことを全write経路で証明できるか

## Evidence

- `crates/codefire-store/src/lib.rs` — ObjectRecord / object_record: public object_id と64桁 hash を同じpayloadから生成してrecordへ保持する。
- `crates/codefire-store/src/lib.rs` — validate_object_record: 保存record自身のpayload、hash、current/legacy ID、filenameの内部整合を再計算する。
- `crates/codefire-store/src/lib.rs` — store_object: path.exists 分岐と hard_link の AlreadyExists 分岐はいずれも既存recordを検証するが、新規要求record.hashとの比較を行わず object_id を返す。
- Test/fixture evidence: `crates/codefire-store/src/lib.rs` — store_object_writes_and_reuses_immutable_record は同一payload再利用を固定するが、人工的な同一prefix・異full-digest fixtureは CFB-125 の未完了範囲である。

## Strengths to Preserve

- full digest をrecordへ保存し、読取時にcanonical payloadから再計算する二重確認。
- legacy 12桁IDを、full hashが一致するrecordに限って読み取る明示的互換経路。

## Gap / Risk

### Observed gap

再利用判定がpublic object_idと既存recordの自己整合だけで完了し、同じprefixへ写像された別の新規payloadを区別しない。

### Failure or stagnation mode

人工的衝突だけでなく十分大きなstoreでprefix衝突が発生した場合、write呼出側は保存したつもりのpayloadとは異なる既存object IDを成功として受け取る。以後のsealed commit検証も「参照先record自体は正しい」ため意味の取り違えを検出できない。

### Affected invariants

- object identity
- sealed commit root identity
- no silent alias

## Improvement Direction

public IDはlocator、full digestはidentityという型・API上の区別を明示し、既存path再利用時には requested.hash == existing.hash と type/payload identity を比較する。衝突は新しいIDを黙って割り当てるのではなく、専用のcollision errorとして停止し、将来のID長変更は別のformat negotiationとして扱う。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 両reuse分岐でrequested/existing full digestを比較する | 現行formatのままsilent aliasを閉じる | collision errorとfixture追加が必要 | adopt |
| B: prefixをさらに延長するだけ | 発生確率を下げる | 同一性境界の欠陥は残りlegacy互換も複雑化する | reject |

## Recommended Sequence

1. **Characterize**: digest関数をtest seam化した人工的same-prefix/different-full-digest fixtureを作る。
2. **Separate**: reuse判定を validate_reused_object(requested, existing, path) へ集約する。
3. **Implement**: path.exists と AlreadyExists の双方でfull digest比較を必須化する。
4. **Prove**: 同一payload、異payload衝突、legacy ID、tamper、並行publishを決定的に試験する。
5. **Promote**: CFB-125へ実装・acceptance evidenceを追記し、本RVCを別CFRへ昇格しない。

## Acceptance Signals

- 人工的same-prefix/different-full-digestが両reuse分岐で専用errorになる。
- 同一payloadのidempotent storeとlegacy 12桁readは維持される。
- collision後に要求外payloadのobject IDを成功返却しない。

## Verification Design

- Focused tests: store_objectのexisting-path、hard-link race、legacy read、tampered record matrix。
- Cross-boundary tests: commit root保存とdoctor読取が同じcollision分類を伝播すること。
- Scale or fault tests: digest prefixをtest用に短縮したbirthday/collision corpus。
- Observability: errorにtype tag、public ID、既存/要求full digestを機密性を損なわない範囲で含める。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-125と直接重複するため同issueへ吸収する。CFR-016はIDを24桁へ延長済みだがfull-digest比較の代替ではない。
- Required predecessor: なし。凍結中Stage 1 batchのCFB-125を先に閉じる。
- Can combine with: RVC-022、RVC-023。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- SHA-256置換やpublic ID全面変更をこの候補へ混ぜない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
