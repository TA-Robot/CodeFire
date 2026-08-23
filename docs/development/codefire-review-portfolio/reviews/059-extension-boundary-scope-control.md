# RVC-059: Extension boundary and scope control

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | evolution / architecture / security |
| Review kind | redesign |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 2 |
| Primary scope | `object/schema/capability extension policy` |
| Promotion target | product spec / ADR / strategy experiment |

## Executive Judgment

unknown CodeFire object prefixのscanを抑え、batch version/unknown optionをfail closedにする現行姿勢は良い。一方、unknown root/field/object referenceの扱いとcommand capabilityは統一extension modelを持たず、将来のadapter/standardizationで「無視可能」と「理解必須」を区別できないため、今のうちに狭いextension boundaryを定義すべきである。

## Scope and Review Question

- In scope: object kind、commit root、JSON field、batch/config、command capability、adapter profileのnamespace/criticality
- Out of scope: runtime plugin loading、agent marketplace、prompt extension、第三者code execution
- Question: extensionを認識しないconsumerが安全に無視できるものと、必ず拒否すべきものを判別できるか

## Evidence

- `crates/codefire-store/src/lib.rs` — `OBJECT_KINDS` / `object_prefix` / `allows_unknown_subdir_lookup`: known kindはregistry化し、unknown writerにはgeneric prefixを返す一方、lookup policyはID形状で分岐する。
- `crates/codefire-store/src/lib.rs` — `validate_sealed_commit` / `expected_root_type` / `validate_root_payload`: rootsを全件readするがunknown root名にはtype/shape/reference semanticsがなく、criticalityを判定しない。
- `crates/codefire-cli/src/remote.rs` — `object_references`: known commit/manifest/resolution/evidence typeだけを辿るため、unknown extension referenceはremote transferから脱落し得る。
- `crates/codefire-cli/src/command_registry.rs` — `CommandCapability`: JSON/metrics booleanだけでschema/profile/version/required feature negotiationを持たない。
- `crates/codefire-cli/src/fire/batch.rs` / `link_batch.rs` — unsupported version/optionを明示rejectする良いfail-closed precedentがある。
- Test/fixture evidence: unknown object type/IDの局所testはあるが、critical/noncritical extensionを旧consumerへ渡すconformance testはない。

## Strengths to Preserve

- object identityとsealed validationをextension convenienceより優先する。
- capability commandとobject kind registryという拡張点の種が既にある。

## Gap / Risk

### Observed gap

namespace、owner、version、criticality、reference traversal、read/write preservationの共通contractがない。serde/Value経路ではunknown fieldを保持する場合と落とす場合も一貫しない。

### Failure or stagnation mode

旧consumerがcritical blockerを無視してcommitをpassed扱いする、unknown referenced objectをremote copyせずgraphを欠損させる、逆に無害なadvisory extensionで全操作を止める。

### Affected invariants

- unknown-required never becomes allow/clean
- complete sealed/reference graph
- bounded product scope and interoperability

## Improvement Direction

`ExtensionDescriptor {namespace, name, version, critical, references, preservation}`とcapability profileを定義する。critical unknownはfail closed、noncritical unknownはidentity bytesを保持してread-only forwarding可能、writerは理解しないcritical subjectをrewriteしない。RVC-057 schema gateと一体で設計する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: namespaced critical-extension contract | 将来interopとfail-closedを両立 | 初期schemaとregistry実装が必要 | experiment then adopt |
| B: unknown field/objectを一律許可/拒否 | 単純 | safetyかevolutionのどちらかを失う | reject |

## Recommended Sequence

1. **Characterize**: 現在のunknown field/root/kind/config/capability behaviorをmatrix化する。
2. **Separate**: extension descriptorとconsumer capability decisionをschema gateへ置く。
3. **Implement**: synthetic critical/noncritical object/rootでread/copy/rewrite policyを試す。
4. **Prove**: old/new/minimal consumer conformanceとremote graph completenessを確認する。
5. **Promote**: Stage 2 schema、Stage 3 adapter profile、Stage 6 extension namespace規則へ反映する。

## Acceptance Signals

- unknown critical extensionは全consumerで明示拒否され、blockerを落とさない。
- noncritical extensionはidentityを変えず保持/転送され、reference graphが欠けない。
- capability negotiationがschema/profile/versionとsupport decisionを返す。

## Verification Design

- Focused tests: unknown root/kind/field、critical flag、namespace/version collision
- Cross-boundary tests: store→remote copy→doctor/migrate→minimal consumer
- Scale or fault tests: extension graph cycle/missing reference、mixed-capability history
- Observability: namespace/version/critical/support decision。payload本体は不要に露出しない

## Dependencies and Duplication Check

- Existing CFB/CFR: 直接同じextension policyを扱う正式issueは確認できない。CFR-071/082/121-123やCFB-123は隣接するが、個別schema/validator問題として置換しない。
- Required predecessor: RVC-055 corpus、RVC-057 schema support registry
- Can combine with: RVC-032（public command capability registryを担当し、本候補は同じgovernance rootにcritical/optional extension規則を与える）、RVC-049、RVC-052、RVC-053、RVC-057

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 任意codeを読み込むplugin systemや無制限product surfaceを作らない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
