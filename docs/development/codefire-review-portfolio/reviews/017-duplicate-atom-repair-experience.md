# RVC-017: duplicate Atom IDをlocation-awareに修復可能にする

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | UX / correctness / domain |
| Review kind | refine |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `AtomIndex duplicates / verification diagnostics / context/explain` |
| Promotion target | future CFR |

## Executive Judgment

duplicate Atom IDをdeterministicに検出し、default policyでcommit blockerにする設計は正しい。しかしdiagnostic payloadは重複IDだけで、宣言location群、selector、rename impact、Trace Link修復planを直接返さない。blockerの強さを維持したまま、衝突集合をfirst-class diagnosticにすべきである。

## Scope and Review Question

- In scope: duplicate declaration locations、selector、kind/hash差、Trace/Fire references、rename planning、bounded UX
- Out of scope: 勝手なID rename、重複をlast-winsで許容すること
- Question: 利用者が内部JSONを読まず、全衝突箇所と安全な修復影響を一回で把握できるか

## Evidence

- `crates/codefire-core/src/lib.rs` — `build_atom_index`: AtomをID/path順にsortしcountするが、`duplicate_atom_ids`にはID文字列だけを保存する。
- `crates/codefire-core/src/lib.rs` — `build_verification`: policyがrejectする場合duplicate ID listをfailed decisionへ含める。
- `crates/codefire-cli/src/verification.rs` — `render_blocking_verification_details`: duplicate detailとして先頭5件のAtom IDだけを表示する。
- `crates/codefire-cli/src/automation.rs` — `verification_next_actions`: `context --atom <id>`を提案するが、targetにはIDのみでconflicting locationsがない。
- `crates/codefire-cli/src/context.rs` — `atom_by_id`: duplicateを`BTreeMap<id, Atom>`へcollectするため、Atom selector viewでは一つの宣言だけが残る。
- Test/fixture evidence: `build_atom_index_uses_defaults_config_and_detects_duplicates`はID検出を固定するが、multi-location repair plan/ambiguous context testはない。

## Strengths to Preserve

- duplicateをsilent overwriteせず、sealed commit前のblocking factとして保持すること。
- Atom list自体にはartifact pathとselectorが残り、修復情報を再構築可能であること。

## Gap / Risk

### Observed gap

診断とnext actionが衝突宣言集合を運ばないため、提案された`context --atom`自体がambiguous mapで一件を隠す。rename後にどのTrace Links、Fires、resolutionsが影響を受けるかもpreviewできない。

### Failure or stagnation mode

利用者が誤った宣言をrename/deleteし、trace graphを壊すか、duplicate blockerから復旧できずpolicyを緩める。大量duplicateでは反復scanが必要になる。

### Affected invariants

- repository-wide Atom ID uniqueness
- Trace Link endpoint identity
- actionable blocker / no silent overwrite

## Improvement Direction

`DuplicateAtomConflict { atom_id, declarations[] }`をdomain diagnosticとし、各declarationにpath、selector、kind、content hash、reference countsを含める。`explain duplicate`またはread-only repair planでrename候補とaffected links/resolutionsを提示し、applyは既存batch validationを使う別段階にする。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: messageへpathを連結 | 早い | machine repair/limit/impactに弱い | reject |
| B: typed conflict + read-only repair plan | 完全でautomation-safe | payload/schemaとimpact計算 | adopt |

## Recommended Sequence

1. **Characterize**: same/different path、kind/hash、3件以上、linked/unlinked duplicate fixtureを作る。
2. **Separate**: uniqueness detection、conflict diagnostic、reference impact、mutation planを分離する。
3. **Implement**: location-aware diagnosticとambiguous context rejectionを先に入れる。
4. **Prove**: repair dry-runが全referencesを列挙し、部分適用なしで再scan後uniqueになるtestを作る。
5. **Promote**: 正式UX/correctness CFRとautomation schemaへ昇格する。

## Acceptance Signals

- 各duplicate IDについて全宣言locationとselectorがJSON/textで得られる。
- ambiguous `context --atom`が一件を黙って選ばず、conflict diagnosticを返す。
- repair後にdangling linksやstale basisが黙って残らない。

## Verification Design

- Focused tests: 2/3+ declarations、same path、mixed kind/hash、bounded conflict list。
- Cross-boundary tests: index → verify → context/explain → repair dry-run → link batch → rescan。
- Scale or fault tests: 1万duplicate groups、path rename失敗、concurrent edit、partial batch rejection。
- Observability: conflict/declaration/reference totalとomitted countを返す。

## Dependencies and Duplication Check

- Existing CFB/CFR: duplicate blocker自体は既存要求で、CFR-020はID validation経路差。location-aware repairを扱う既存CFB/CFRは確認できず、新規候補である。
- Required predecessor: CFB-123 endpoint integrityとRVC-007 ontology/schema。
- Can combine with: RVC-006、RVC-013。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- rename先を推測して無確認でsourceを編集しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
