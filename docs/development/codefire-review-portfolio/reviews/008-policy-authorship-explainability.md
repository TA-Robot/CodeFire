# RVC-008: policy authorshipと判定説明を一級domain contractにする

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | product / domain / UX / evolution |
| Review kind | refine |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 2 |
| Primary scope | `VerificationPolicy / codefire.policy.yaml / explain / certificate` |
| Promotion target | product spec |

## Executive Judgment

repositoryが宣言したpolicyをsealし、そのpolicyでcommit可否を判定する設計は強い。しかしpolicyの作者・意図・profile・継承元・変更理由と、各判定がどのruleから生じたかを説明する構造は薄い。policyを単なるboolean/YAML集合ではなく、review可能なversioned decision artifactとして扱うべきである。

## Scope and Review Question

- In scope: policy metadata、authorship、profile、rule identity、default/override provenance、explainability、change review
- Out of scope: 組織IAM、中央policy server、自然言語からのpolicy自動生成
- Question: 利用者とreviewerが「誰のどのruleが、なぜこのblocker/passを作ったか」をsealed dataから追えるか

## Evidence

- `crates/codefire-core/src/lib.rs` — `VerificationPolicy` / `RequiredLinkRule`: commit gateをboolean群、required-link map、verification commandsで表すが、rule ID、author、profile、rationaleはない。
- `crates/codefire-core/src/lib.rs` — `parse_verification_policy` / `default_verification_policy`: bounded YAMLとcore defaultsをmergeするが、各値のsource provenanceを結果型へ残さない。
- `crates/codefire-core/src/lib.rs` — `policy_hash` / `ResolutionBasis`: policy全体digestをfreshnessへ使うため、無関係rule変更でもresolutionがstaleになり得る。
- `crates/codefire-cli/src/explain.rs` — `parse_explain_args` / `run_explain`: Fire、Atom、verify failure、storageを説明するが、policy rule/profile自体をtargetにしない。
- Test/fixture evidence: parser/default/legacy wrapper testsはあるが、rule-level explanation、authorship、unrelated-rule freshnessのdecision fixtureはない。

## Strengths to Preserve

- policy bytesがcontent-addressed sealed objectとなり、後から判定条件を再読できること。
- safe defaultsとunsupported field rejectionをcore parserが担うこと。

## Gap / Risk

### Observed gap

policyは機械判定できてもreview artifactとして自己説明的でない。default由来か明示設定かを判別しにくく、全体hashによるinvalidatonは変更影響を過大化する。

### Failure or stagnation mode

teamがblocker理由を理解できずpolicyを緩める、reviewなしの変更でseal基準が変わる、または無関係なpolicy editで大量resolution refreshが発生する。

### Affected invariants

- declared-policy verification
- resolution freshness
- certificate explainability / deterministic defaults

## Improvement Direction

policy documentへstable rule IDs、profile/version、author/owner、rationale、effective-from、criticalityを加え、parsed policyにはvalue provenanceを残す。verification diagnosticはrule IDを参照し、`explain policy|rule`でeffective policyとdefault/overrideを表示する。freshnessは実際に依存したrule digestへ細分化する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: YAML commentとdocsで説明 | format変更なし | sealed object/JSONから説明不能 | reject |
| B: typed metadataとrule-level provenance | review・freshnessが精密 | schema migrationとauthor semantics設計 | adopt |

## Recommended Sequence

1. **Characterize**: default/override、無関係rule変更、profile変更の判定・stale matrixを作る。
2. **Separate**: source document、effective policy、rule provenance、evaluation traceを型分離する。
3. **Implement**: read-only effective-policy/explain viewとstable rule IDsから始める。
4. **Prove**: 同じeffective semanticsのcanonical hash、rule-level invalidation、legacy policy fixtureを検証する。
5. **Promote**: policy spec、schema、authoring guide、必要ならADRへ昇格する。

## Acceptance Signals

- blocker diagnosticからstable rule ID、source、effective value、rationaleを辿れる。
- 無関係rule変更が当該ruleに依存しないresolutionをstaleにしない。
- legacy policyは明示されたdefault provenanceで同じ判定を維持する。

## Verification Design

- Focused tests: default/override provenance、rule ID uniqueness、profile/version、semantic hashing。
- Cross-boundary tests: policy parse → verification diagnostic → explain → commit policy root → replay。
- Scale or fault tests: 大規模rule set、duplicate IDs、unknown critical field、partial migration。
- Observability: effective profile、changed/used rule IDs、defaulted fieldsをbounded reportに出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-003/025とCFR-022/116はwarning、legacy syntax、parser/runner fieldの個別問題。本候補はauthorship/provenance/evaluation traceで重複しない。
- Required predecessor: RVC-011 committable predicateとRVC-007 ontology IDs。
- Can combine with: RVC-013、RVC-019。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- policy authorを自動的に信頼済みauthorityと見なさない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
