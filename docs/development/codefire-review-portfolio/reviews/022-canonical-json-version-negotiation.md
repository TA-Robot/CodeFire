# RVC-022: canonical JSON のversion negotiationとcross-runtime corpus

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | architecture / correctness / evolution / testing |
| Review kind | refine |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 2 |
| Primary scope | `codefire-store canonical_json / Rust-Python object compatibility` |
| Promotion target | product spec |

## Executive Judgment

RustとPythonが現在のgolden payloadで同じdigestを作る基礎は強い。しかしcanonicalization algorithm自体には名前・version・受理範囲がなく、serde/json実装詳細と少数goldenへ依存している。既存object IDを変えずに、現行algorithmを明文化したprofileとcross-runtime corpusを先に固定し、未知profileを明示拒否するformat negotiationへ進むべきである。

## Scope and Review Question

- In scope: canonical byte列、数値・Unicode・key順序、Rust/Python互換、algorithm profile
- Out of scope: command_result schema、暗号hash変更、product SemVer
- Question: runtimeやserializer更新後も同一logical payloadが同一full digestになり、未知canonical profileが黙って現行扱いされないか

## Evidence

- `crates/codefire-store/src/lib.rs` — canonical_json / object_digest: serde_json::to_vec の出力をtype tagとNULへ連結してdigest入力にする。
- `codefire` — canonical_json / object_digest: Pythonは ensure_ascii=false、sort_keys=true、compact separatorで同じ目的のbyte列を独立実装する。
- `crates/codefire-store/src/lib.rs` — canonical_json_sorts_keys_and_uses_compact_utf8 / object_digest_matches_python_golden_payloads: key順・UTF-8と3種payloadのdigestを固定する。
- Test/fixture evidence: `docs/development/source-data-model-catalog.md` — Version Axesにはcore、Atom hash、layout、commit root、public ID、envelopeがあるがcanonicalization profile軸は定義されていない。

## Strengths to Preserve

- type tag + NUL + canonical bytes で型間衝突を分離する簡潔なdomain separation。
- compact UTF-8、key sort、cross-language goldenを既に採用していること。

## Gap / Risk

### Observed gap

canonical byte仕様の受理可能JSON subset、数値表現、escape、重複key、将来profileの選択規則がserializer実装と暗黙のdefaultに残る。

### Failure or stagnation mode

runtime更新や第三実装の導入時に、一部payloadだけ別digestになりobject graphを共有できない。逆に未知profileを現行defaultへ落とすと、異なる意味のobjectを正当なlegacy objectとして読む危険がある。

### Affected invariants

- canonical object identity
- Rust/Python readability
- format evolution

## Improvement Direction

現行byte規則を Canonical JSON Profile 1 としてspec化し、既存recordは暗黙profile 1としてのみ読む。新profileが必要になった場合はrepository/format capabilityで明示し、digest対象payloadへ場当たり的fieldを追加して既存IDを変えない。writer/reader/unknownのdecision tableと、言語非依存vector corpusをsource of truthにする。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 現行profileを外部仕様＋vector corpusとして固定する | 既存IDを維持し独立実装を検証できる | corpusとconformance runnerの保守が必要 | adopt |
| B: 全payloadへcanonical_version fieldを即追加する | object単位で見える | 全object IDが変わりmigration境界を広げる | reject |

## Recommended Sequence

1. **Characterize**: Unicode、escape、整数境界、-0、nested object/array、parse拒否例のcross-runtime vectorを作る。
2. **Separate**: canonical profile、object schema、public ID長を独立version軸としてspecにする。
3. **Implement**: profile 1 conformance readerと未知profileの明示errorをformat inspectionへ追加する。
4. **Prove**: Rust/Python/独立最小consumerで全vectorのbytes、full digest、public IDを比較する。
5. **Promote**: Stage 2 public schema familyとrelease/format roadmapへprofile decisionを昇格する。

## Acceptance Signals

- 全vectorでruntime間のcanonical bytesと64桁digestがbyte-for-byte一致する。
- 未知required profileは明示的に失敗し、legacy profile 1へdowngradeしない。
- 既存fixtureのobject IDは一つも変わらない。

## Verification Design

- Focused tests: serializer edge-case tableとinvalid/non-finite input拒否。
- Cross-boundary tests: Python-created recordのRust read、Rust-created recordのPython doctor/show。
- Scale or fault tests: 深いnested JSONと大きなmapのdeterminism・allocation counter。
- Observability: migration/check出力にdetected profile、reader support、writer profileを表示する。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-125はprefix衝突で隣接するが、本件はcanonical byteのversion境界で非重複。CFR-016はpublic ID長のみ、CFR-071はcommand envelope schema定数のみ。
- Required predecessor: RVC-021/CFB-125のfull-digest同一性修正。
- Can combine with: RVC-033のJSON Schema family、RVC-023のsealed reader、RVC-055のcross-runtime corpus。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- canonical profileをproduct/package versionと同一視しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
