# RVC-052: Crate boundary and domain leakage

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | architecture / domain / evolution |
| Review kind | refine |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 1 |
| Primary scope | `codefire-core / codefire-store / codefire-cli boundaries` |
| Promotion target | ADR / future CFR |

## Executive Judgment

CLI→core/store/utilという依存方向とstoreのidentity ownershipは維持すべきである。一方、domain値がstring/`serde_json::Value`中心で、CLIがobject reference graph・commit payload・repository IOを再構築し、coreもfilesystem parsingとpure policyを同居させるため、不変条件のownerを型/APIとしてさらに明確にする余地がある。

## Scope and Review Question

- In scope: crate ownership、public API、typed domain/persisted model、reference traversal、workspace adapter
- Out of scope: crate数を増やすこと自体、外部plugin API、network library選定
- Question: identity・sealed graph・domain decisionを一つのownerへ集約し、CLIをorchestrationへ限定できるか

## Evidence

- `docs/development/source-architecture.md` — `Crate Boundaries`: core/store/CLIの所有・禁止責務と依存方向をcanonicalに定義する。
- `crates/codefire-core/src/lib.rs` — `Fire` / `Verification` / `Resolution`: status、severity、type tag等を`String`で持ち、invalid stateを型で排除しない。
- `crates/codefire-core/src/lib.rs` — `build_atom_index` / `current_trace_graph` / `parse_verification_policy`: filesystem traversal/parsingとdomain constructionを同じcrate APIで行う。
- `crates/codefire-cli/src/remote.rs` — `object_references` / `collect_object_records`: store validatorとは別にobject kind別reference graphをCLIが知る。
- `crates/codefire-cli/src/main.rs` — `run_commit`: storeの`commit_payload` helperを使わずproduction commit JSONを直接構築することがcanonical sourceにも記録される。
- Test/fixture evidence: crate-local testsはあるが、dependency boundaryや「CLIにidentity ruleを複製しない」ことを検出するarchitecture testはない。

## Strengths to Preserve

- core/storeはCLIへ依存せず、canonical bytes/full digest/sealed validationはstoreへ集約されている。
- focused CLI modulesと`codefire-util`の小さなprimitive ownerが既にある。

## Gap / Risk

### Observed gap

型とreference registryが境界を十分に表さず、JSON field名・object kind・status語彙を複数crate/moduleが共有知識として持つ。production pathとtest/helper pathが異なる場合もある。

### Failure or stagnation mode

新object rootやstatus追加時にstore validator、remote copy、commit builder、doctor/migrationの一部だけ更新され、sealedだが搬送不能・診断不能なstateが生じる。

### Affected invariants

- canonical object identity and reachability
- verification/certificate agreement
- crate dependency and policy ownership

## Improvement Direction

まずobject kind/reference schemaとcommit builder/validatorをstoreのtyped registryへ統合する。次にcoreのpure domain inputとfilesystem workspace adapterを分け、critical status/versionはenum/newtype化する。crate分割はownerが明確になる場合だけ行う。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: typed registry/APIを既存crate内で強化 | 小さいdependency変化でownerを明確化 | 段階的migrationが必要 | adopt |
| B: 新crateを大量追加 | 見た目の分離 | API増加と循環的mappingが増える | reject |

## Recommended Sequence

1. **Characterize**: object kind/status/version/referenceの重複owner mapを作る。
2. **Separate**: store registryとpure core inputを先に抽出する。
3. **Implement**: remote/commit/doctorをsingle registry/builderへ移す。
4. **Prove**: new synthetic object kindとinvalid stateで更新漏れを検出する。
5. **Promote**: crate boundary ADRとsource architectureを更新する。

## Acceptance Signals

- object kind追加が一つのregistry変更でvalidator/path/reference traversalへ伝播する。
- critical status/versionはinvalid stringをconstruction時に拒否する。
- CLIにcanonical identityやsealed graph ruleの代替実装が残らない。

## Verification Design

- Focused tests: typed enum/version、object registry、commit builder/validator round-trip
- Cross-boundary tests: core domain→store object→remote copy→doctor/read
- Scale or fault tests: unknown kind/version、missing reference、registry update omission
- Observability: owner moduleとrejected invariantをstable diagnosticにする

## Dependencies and Duplication Check

- Existing CFB/CFR: fixed CFR-015/040/121-123/133等のregistry・validator重複を再openせず、crate APIの上位原因を評価する。CFB-123/125の凍結invariantにも依存する。
- Required predecessor: CFB-120〜125完了、RVC-055/057のcompatibility vectors
- Can combine with: RVC-020（core内のIO/pure-domain境界を担当し、本候補はその同じowner漏れをcross-crate APIで閉じる）、RVC-048、RVC-051、RVC-057、RVC-059

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- crate数や型数の増加を品質指標にしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
