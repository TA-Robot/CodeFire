# RVC-033: public command JSON schema familyに互換性decisionを持たせる

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | evolution / correctness / architecture |
| Review kind | redesign |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 2 |
| Primary scope | `codefire.command_result.v1` / command inputs, data payloads, capabilities |
| Promotion target | product spec |

## Executive Judgment

top-level envelopeとpayload-specific type/versionを分離している方針は正しい。しかしpublic commandのbatch input、data payload、operation plan、capabilitiesにはschema ID、consumer profile、additive/required field規則を束ねるfamily manifestがない。RVC-033はCLI public contractに限定し、persisted object・repository・remote versionのdecode/migrate規則はRVC-057へ委ねる。

## Scope and Review Question

- In scope: command envelope、data.type/version、batch input、operation plan、capabilities、public consumer profile
- Out of scope: persisted object/repository/remote version gate（RVC-057）、product SemVer、canonical JSON bytes（RVC-022）
- Question: 独立consumerがcommand input/output schemaの追加・未知version・required capabilityを同じ規則でALLOW/DENY/UNKNOWN判定できるか

## Evidence

- `crates/codefire-cli/src/automation.rs` — `COMMAND_RESULT_SCHEMA` / `command_result_envelope`: top-level schemaはcodefire.command_result.v1へ統一するが、data payload schemaは個別type/versionである。
- `crates/codefire-cli/src/command_registry.rs` — `capabilities_data_json`: capabilities payloadはtype/versionを持つが、各commandのinput/output schema IDやrequired capabilityを列挙しない。
- `crates/codefire-cli/src/main.rs` — `commit_result_data_json` / `review_pack_result_data` / `plan_result_envelope`: public data payloadごとにtype/version/plan wrappingを個別構築する。
- Test/fixture evidence: `crates/codefire-cli/src/batch.rs`、`fire/batch.rs`、`evidence/batch.rs`、`link_batch.rs` — batch inputはversion != 1を明示拒否する良いprecedentがある。対照的にpersisted rootのfuture-version問題は `crates/codefire-store/src/lib.rs` の `required_commit_roots` に存在するが、解決ownerはRVC-057とする。

## Strengths to Preserve

- envelope schemaとpayload-specific type/versionを独立させ、top-level typeを増やさない設計。
- batch inputが未知versionをsilent defaultせず明示errorにし、capabilitiesを共通envelopeで取得できること。

## Gap / Risk

### Observed gap

public command payloadごとのowner、schema ID、current producer、consumer profile、required/optional field、unknown dispositionを列挙するmachine-readable sourceがない。

### Failure or stagnation mode

consumerが未知data payloadやrequired fieldを既知payloadとして処理し、diagnostic、omission、dry-run、action safetyの意味を落とす。逆に安全なadditive fieldでもconsumerが壊れ、public machine contractが停滞する。persisted objectを誤解するriskはRVC-057で扱う。

### Affected invariants

- automation contract
- capability negotiation
- command input validation
- blocker fail-closed semantics
- independent consumer interoperability

## Improvement Direction

public command schema manifestを作り、envelope、data payload、batch input、operation plan、capabilitiesのschema ID、version、owner、producer/consumer support、critical extension rule、unknown dispositionを定義する。JSON Schemaはshape、semantic validatorはok/exit/diagnosticやdry-run/plan等のcross-field invariantを担当する。persisted decode registryやmigrationはRVC-057のmanifestから参照し、同じregistryへ混在させない。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: public command schema manifest＋semantic conformance | consumer向けcompatibility decisionを統一 | schema maintenanceとproducer移行が必要 | adopt |
| B: payload type/versionの個別ifに任せる | 実装が局所的 | required fieldとconsumer profileのdriftが残る | reject |

## Recommended Sequence

1. **Characterize**: registered commandのinput/output/data/plan schemaを全列挙しproducer/consumer現状表を作る。
2. **Separate**: public JSON shape、semantic invariant、capability negotiationをpersisted decode/migrationから分ける。
3. **Implement**: command_result、capabilities、主要batch、plan/result payloadのschemaから公開する。
4. **Prove**: current/future/additive/critical-extension corpusを独立consumerで検査する。
5. **Promote**: public command schema familyとconsumer ruleをproduct specへ昇格する。

## Acceptance Signals

- 全declared JSON commandのinput/output/data payloadに一意schema ID/versionとconsumer dispositionがある。
- unknown required schema/capabilityがok=trueや安全に実行可能なactionへ変換されない。
- current producerのsuccess/failure/dry-run payloadを独立consumerがsource非参照で検証できる。

## Verification Design

- Focused tests: missing/wrong/future version、unknown required/optional field、envelope/data cross-field mismatch。
- Cross-boundary tests: parser/command→envelope→capability-aware independent consumer。
- Scale or fault tests: large bounded payload、schema bomb/deep nesting、partial/truncated JSON。
- Observability: detected schema/version、support disposition、failed semantic ruleをdiagnosticへ出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-071はenvelope schema文字列のsingle constant化をfixed済み。CFR-132はpersisted migration側でRVC-057の近接issue。public command schema familyに直接重複するopen CFB/CFRはない。
- Required predecessor: RVC-032のcapability negotiation。persisted subjectを参照するpayloadはRVC-057のversion decisionに依存する。
- Can combine with: RVC-034、RVC-038、RVC-055のcross-runtime conformance corpus、RVC-057のpersisted schema/object evolution、RVC-059のcritical extension policy。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- public JSON Schemaでpersisted object decode/migrationやsealed semantic validationを代替しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
