# RVC-019: consistency certificateのproof boundaryを明示する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | correctness / security / domain / evolution |
| Review kind | redesign |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `commit certificate / verification root / policy root / sealed validation` |
| Promotion target | ADR |

## Executive Judgment

certificate result/countをverification payloadと照合し、commit object identityが全rootsをbindする現行構造は強い。しかしcertificate単体はsummary projectionであり、product requirementが期待するsubject、policy hash、base、coverage、execution profileを持たない。「certificate」と呼ぶ範囲をsealed commit内summaryに限定するか、standalone検証可能なproof envelopeへ進化させるかをADRで決めるべきである。

## Scope and Review Question

- In scope: certificate fields、subject/policy/base binding、verification reference、signature relation、standalone/embedded semantics、legacy compatibility
- Out of scope: correctness保証、PKI/法的署名基盤の即時導入
- Question: certificateだけを取り出したconsumerが、何を検証し何が未検証かを安全に判定できるか

## Evidence

- `docs/01_product_definition.md` — `Verification and Certificate`: certificateに検証対象、policy hash、blocker count、result、base commitを含める要求がある。
- `crates/codefire-cli/src/main.rs` — `run_commit`: certificateはresultと6種countだけをinline生成し、subject/policy/base/referenceはcommitの別fields/rootsに置く。
- `crates/codefire-store/src/lib.rs` — `validate_sealed_commit`: required rootsとtypes/referencesを検査し、verification rootを読みcertificateとの一致を確認する。
- `crates/codefire-store/src/lib.rs` — `validate_certificate_matches_verification`: result aliasとpresent count fieldsを比較するが、certificateにないcountは照合を省略し、subject/policy digestは扱わない。
- `crates/codefire-cli/src/signatures.rs` — commit signing path: signatureはcommit payload境界で扱われ、certificate自体の独立署名contractではない。
- Test/fixture evidence: CFR-122/159 mismatch/legacy testsはfixedだが、certificate extraction、wrong subject/policy/base、missing countのstandalone conformance fixtureはない。

## Strengths to Preserve

- current commit ID/signatureがcertificateを含むroot graph全体へ暗号学的にbindすること。
- storeが見た目のpassedだけを信じず、verification factsとreachable rootsを検証すること。

## Gap / Risk

### Observed gap

embedded summaryとstandalone proofの意味が名称上分かれず、certificate JSONだけをexportした場合のverification procedureが定義されない。optional count照合とlegacy aliasも、どのschemaで何が必須かを弱くする。

### Failure or stagnation mode

adapter/reviewerがcertificate fragmentを完全なattestationとして流通させ、subject substitutionやpolicy誤認を起こす。逆に毎回commit graph全体が必要ならintegration costが高い。

### Affected invariants

- sealed subject/policy/verification binding
- no forged/stale evidence completion
- signature/claim boundary / compatibility

## Improvement Direction

まず`EmbeddedConsistencySummary`と`PortableConsistencyProof`を別型・claim typeにする。portable版はcommit/subject digest、verification object/full digest、policy/profile digest、base/parents、coverage、execution/evidence profile、schema criticalityを明示し、signatureはそのcanonical envelopeを対象にする。embedded-onlyを選ぶ場合はexport禁止/必要contextをschemaで明記する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: certificateをembedded summaryと正式限定 | 単純で誤用を減らす | external integrationはgraph取得が必要 | experiment |
| B: portable proof envelopeを追加 | 独立検証可能 | crypto/schema/key lifecycleが増える | experiment |

## Recommended Sequence

1. **Characterize**: embedded/portable consumer use casesとsubject substitution counterexamplesを作る。
2. **Separate**: decision facts、embedded summary、portable proof、commit/request signaturesを型分離する。
3. **Implement**: embedded summaryのschema/required fieldsを厳格化し、portable prototypeと比較する。
4. **Prove**: clean-room verifierでwrong subject/policy/base/evidence/profile/missing fieldを拒否する。
5. **Promote**: claim/certificate ADR、data model、public schema、RVC-060 clusterへ昇格する。

## Acceptance Signals

- certificate kindから必要contextと検証procedureを一意に選べる。
- subject/policy/base/verificationの差し替えが全supported profileで拒否される。
- legacy embedded commitsは保守的claimで読め、portable proofと誤認されない。

## Verification Design

- Focused tests: required fields、canonical bytes、legacy alias、wrong digest、unknown critical extension。
- Cross-boundary tests: core decision → commit roots → certificate/proof → signature → store/clean-room verifier。
- Scale or fault tests: missing object、partial graph、key rotation/replay、tampered counts、schema downgrade。
- Observability: proof kind、subject/policy/verification digests、coverage/trust profile、validation stepsを出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-121/122/123/159はroot/certificate shapeと一致の個別修正。本候補はembedded/portable proof境界を決める上位設計で重複しない。
- Required predecessor: RVC-001 claim descriptor、RVC-005/018 provenance、RVC-011 authoritative decision。
- Can combine with: RVC-001、RVC-011。RVC-060ではsingle predicate clusterのdownstream proof projectionとして統合する。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- certificateをsemantic correctnessやregulatory complianceの証明にしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
