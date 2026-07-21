# RVC-057: Schema and object version evolution

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | evolution / correctness / architecture |
| Review kind | redesign |
| Priority | P0 |
| Confidence | confirmed |
| Roadmap stage | Stage 2 |
| Primary scope | `persisted objects / repository metadata / remote persisted formats` |
| Promotion target | product spec / ADR / future CFR |

## Executive Judgment

CodeFireはversion axesを分離し、legacy 12-hex/root schemaを明示的に読む強い基盤を持つ。ただしvalidatorはversion fieldの存在だけを見る箇所や、未知の`root_schema_version >= 2`をcurrent rootsとして受理する箇所があり、「unknown required schemaはfail closed」というStage 2原則を実装できていない。本候補はpersisted object/repository/remote formatを担当し、public command/input/output schemaはRVC-033に委ねる。

## Scope and Review Question

- In scope: object payload、commit root、repo metadata、active/idempotency/remote persisted formatのversion read/write/reject/migrate
- Out of scope: RVC-033が担当するpublic command/input/output/envelope JSON Schema、product SemVer、package version、無条件のformat migration
- Question: 各version axisでknown readable/writable/rejected rangeとunknown required behaviorが一意か

## Evidence

- `docs/development/source-data-model-catalog.md` — `Version Axes` / `Compatibility Rules`: schema axesと「unknown required schemaは明示失敗」をcanonicalに定める。
- `crates/codefire-store/src/lib.rs` — `required_commit_roots`: `root_schema_version >= 2`を全てcurrent root setとして扱い、未知future versionを区別しない。
- `crates/codefire-store/src/lib.rs` — `validate_root_payload`: root payloadの`version`をu64として要求するだけでsupported valueを検証しない。
- `crates/codefire-cli/src/migration.rs` — `check_repo_json`: repository version 1を受理し、`version != Some(1)`を明示rejectする一方、全object kind/versionのcompatibility decisionまでは持たない。
- `crates/codefire-cli/src/fire/batch.rs` / `link_batch.rs` — batch version != 1を明示rejectする良いprecedentがある。
- Test/fixture evidence: legacy root/read testsはあるが、future root/object/active/idempotency versionのfail-closed matrixはない。

## Strengths to Preserve

- product/package/repository/root/hash/envelope versionを同一数字として扱わない。
- legacy readabilityをfull digest agreementと明示default pathへ限定する。

## Gap / Risk

### Observed gap

version handlingがserde default、presence check、numeric comparison、module固有parserに分散し、reader/writer/migrator capabilityがregistry化されていない。

### Failure or stagnation mode

future required fieldを理解しない旧consumerがcommitをvalidと判断し、blockerを落としてread/writeする。逆に安全なoptional extensionまで不必要に拒否する。

### Affected invariants

- no silent schema downgrade
- sealed commit/root interpretation
- automation contract compatibility

## Improvement Direction

schema familyごとに`SchemaSupport {read, write, migrate, unknown_required}`をregistry化し、decode時にversion/capabilitiesを判定してからdomain型へ変換する。critical/optional extensionはRVC-059と共通規則にし、writerは常にcurrent explicit versionを出す。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: schema support registry + typed decode | 全readerのdecisionを統一できる | fixtureとmigration設計が必要 | adopt |
| B: validatorごとにversion ifを追加 | 早い | driftとdefault漏れが残る | reject as strategy |

## Recommended Sequence

1. **Characterize**: 全persisted/schema familyのwriter/read/default/reject表を作る。
2. **Separate**: version gateをshape/domain validationより前の共通decode boundaryへ置く。
3. **Implement**: root/object/repo/idempotency/remoteからfail-closed registryへ移す。
4. **Prove**: RVC-055 corpusでlegacy/current/future/optional/critical vectorsを通す。
5. **Promote**: persisted-format compatibility、migration policy、ADRへ反映し、public command schemaのdecisionはRVC-033へ引き渡す。

## Acceptance Signals

- 各schema familyにread/write/reject/migrate rangeとownerがある。
- unknown required version/capabilityはblockerを落とさずwrite前に失敗する。
- legacy supported fixtureはidentityを変えず読み、current writerは一意なversionを出す。

## Verification Design

- Focused tests: absent/0/1/max/future/non-number version、required/optional extension
- Cross-boundary tests: store→CLI→doctor/migrate→remote、Rust/Python/minimal consumer
- Scale or fault tests: mixed-version history、partial migration、future root with valid digest
- Observability: schema family/version/decision/migration actionをstable diagnosticにする

## Dependencies and Duplication Check

- Existing CFB/CFR: fixed CFR-071/132等はschema文字列/target format各論。本候補はcanonical catalogが既に認めるunknown-version gapであり、counterexample確認後に必要部分だけfuture CFR化する。RVC-033とは同じversion-governance rootを共有するが、本候補はpersisted object/repository/remote format、RVC-033はpublic command/input/output schemaを担当する。
- Required predecessor: CFB-125とRVC-055 corpus
- Can combine with: RVC-033、RVC-043、RVC-049、RVC-053、RVC-055、RVC-059

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- internal schema incrementからproduct SemVerを自動決定しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
