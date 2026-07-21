# RVC-001: consistency claim の境界を製品契約として固定する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | product / correctness / evolution |
| Review kind | refine |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 2 |
| Primary scope | `product claim / verification / sealed commit / automation schema` |
| Promotion target | product spec |

## Executive Judgment

CodeFireが「宣言されたpolicyの下で整合したsealed subject」を示せる点は維持すべき最大の強みである。一方、思想文書の強い表現、verificationの判定、certificate、利用者向け表示の間で、証明対象・coverage・実行環境・非保証事項を一つの型として渡す契約はまだ弱い。Stage 2で claim descriptor を定義し、`correct/secure/compliant` への誤読を構造的に防ぐべきである。

## Scope and Review Question

- In scope: verification結果、policy、coverage、evidence subject、certificate、human/JSON表示のclaim境界
- Out of scope: 法規制認証、security audit代行、AIによる真理判定
- Question: sealed commitを受け取った第三者が、CodeFireが証明したことと証明していないことを追加説明なしで判別できるか

## Evidence

- `docs/development/strategy/product-strategy.md` — `Durable Product Core` / `Claim Language`: policy整合とevidence bindingは主張できるが、semantic correctness、security、complianceは証明しないと明記する。
- `docs/01_product_definition.md` — `Verification and Certificate` / `Product Invariants`: certificateへ検証対象、policy hash、blocker count、result、base commitを含める要求がある。
- `crates/codefire-core/src/lib.rs` — `Verification` / `build_verification`: 現在のdomain truthは結果とblocker collectionsであり、coverageやexecution environmentのclaim型は持たない。
- `crates/codefire-cli/src/main.rs` — `run_commit`: certificateはverificationのresultと件数projectionとして生成される。
- Test/fixture evidence: `crates/codefire-store/src/lib.rs` — `validate_certificate_matches_verification` testsはcount/result改ざんを拒否するが、claim comprehensionやcoverage unknownのfixtureはない。

## Strengths to Preserve

- certificateがverificationから導出され、storeで相互検証される現在の一方向性。
- unknown coverageをcleanへ昇格しない戦略原則と、sealed subjectをcontent-addressed rootsへ結び付ける構造。

## Gap / Risk

### Observed gap

claimの注意書きはstrategy proseにあり、persisted certificateやautomation payloadが「どのprofile、subject、coverage、環境についてのpassedか」を自己記述していない。強い製品コピーと狭い実装上のpredicateの間をconsumerが推測する必要がある。

### Failure or stagnation mode

利用者が`passed`を「変更は正しい」「全成果物がcovered」と解釈し、誤った完了判断をする。逆に説明を毎回人手で補う必要があると、独立consumerとadoption pilotが進まない。

### Affected invariants

- sealed commit / evidence subject binding
- verification truth / automation contract
- unknown coverage remains unknown

## Improvement Direction

versioned claim descriptorをverification/certificateの共通projectionとして定義し、`claim_type`、policy/profile identity、subject digest、included/excluded/unknown coverage、verification environment reference、explicit non-claimsを持たせる。human wordingは同じ型からrenderし、既存commitはlegacy claimとして保守的に解釈する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: proseだけを強化する | 低コスト | machine consumerが誤読を防げずdriftする | reject |
| B: versioned claim descriptorをdomain/schema化する | native/advisoryの区別と独立解釈が可能 | format migrationとUI設計が必要 | adopt |

## Recommended Sequence

1. **Characterize**: 現行certificateを見せるclaim-comprehension testと、誤解パターンのcounterexampleを作る。
2. **Separate**: verification resultと外部向けclaim projectionを型で分離する。
3. **Implement**: read-only JSONへdescriptorを追加し、legacy commitは`coverage=unknown`でrenderする。
4. **Prove**: schema/golden vectorsとcertificate改ざんfixtureでsubject・policy・coverage一致を検査する。
5. **Promote**: `docs/01_product_definition.md`、CLI spec、schema、必要ならADRへ昇格する。

## Acceptance Signals

- 独立consumerがprofile、subject、included/excluded/unknown、non-claimsをschemaだけで判定できる。
- claim-comprehension testで`passed=semantic correctness`という誤読がpredeclared threshold以下になる。
- 既存のcertificate/verification mismatch拒否とsealed commit互換性が維持される。

## Verification Design

- Focused tests: claim descriptor decision table、legacy/default、unknown coverage、語彙snapshot。
- Cross-boundary tests: core verification → commit certificate → store validation → CLI JSON/textの同値性。
- Scale or fault tests: descriptor field欠落・改ざん・unknown critical extensionのcorruption fixture。
- Observability: claim typeとcoverage countをbounded JSONに出し、省略時はomitted/unknownを明示する。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-122とCFR-159はcertificate/result整合の個別修正でfixed。本候補は外部claimの意味境界を追加するため重複しない。CFB-014〜016の状態表示修正も包含しない。
- Required predecessor: Stage 1 committable predicateとsealed validationの凍結batch完了。
- Can combine with: RVC-010、RVC-019。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- CodeFireがsemantic correctness、security、complianceを認証する機能にはしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
