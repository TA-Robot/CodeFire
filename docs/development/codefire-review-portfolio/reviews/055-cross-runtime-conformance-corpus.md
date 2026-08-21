# RVC-055: Cross-runtime conformance corpus

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | testing / evolution / correctness |
| Review kind | redesign |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 2 |
| Primary scope | `canonical JSON/object/repository/remote compatibility fixtures` |
| Promotion target | product spec / strategy experiment |

## Executive Judgment

Rust storeのhard-coded Python golden digestとlegacy-read testsは価値があるが、actual writer同士を同一corpusへ通していない。release roadmapが要求するfixture matrixを、checked-in artifact+manifest+checksum+expected decisionからなるruntime-independent conformance corpusとして実装する必要がある。

## Scope and Review Question

- In scope: canonical bytes/full digest/public ID、root/Atom schema、repository/open/remote/signature、read/write/migrate/reject
- Out of scope: package installation、全CLI output parity、Pythonへの新feature実装
- Question: Rust/Python/将来consumerが同じartifactに対し同じidentityと明示support decisionを返すことを再現できるか

## Evidence

- `crates/codefire-store/src/lib.rs` — `object_digest_matches_python_golden_payloads`: Python由来とされる期待digestをRust test内へ手書きするが、Python実行で再生成しない。
- `crates/codefire-store/src/lib.rs` — `read_object_accepts_legacy_twelve_hex_object_ids`: legacy ID一件のreadabilityを局所fixtureで確認する。
- `crates/codefire-cli/src/tests.rs` — `init_creates_python_compatible_repo_layout` / `status_reads_python_compatible_open_directory`: compatibility shapeをRust testが手作りする。
- `tests/test_codefire_cli.py` — `object_digest` / `remote_object_id`: Python test側にもcanonical digest/12-hex期待の独立helperがある。
- `docs/development/strategy/release-and-format-roadmap.md` — `Compatibility Matrix to Build`: writer、canonical bytes、digest、IDs、read/write consumer、migration、checksumを持つ実fixtureを未完gateとして要求する。
- Test/fixture evidence: `crates/codefire-store/src/lib.rs::object_digest_matches_python_golden_payloads`と`crates/codefire-cli/src/tests.rs::status_reads_python_compatible_open_directory`は局所goldenを持つが、runtime-independentなartifact/manifest/checksum corpusはまだない。

## Strengths to Preserve

- canonical bytes/full digestをpublic prefixより正とする原則が明確である。
- legacy 12-hex、root schema、Atom hash schema等のversion axesがcatalog化されている。

## Gap / Risk

### Observed gap

期待値がruntime別test codeへ重複し、fixture provenance/writer version/checksumがない。repository/remote/signed historyの複合artifactとnegative/unknown-version vectorが不足する。

### Failure or stagnation mode

両runtimeが同じ誤りへ同時にdriftする、手作りfixtureがactual writerと違う、migrationがidentityを変えても検出できない。独立consumer gateを満たせない。

### Affected invariants

- canonical identity across runtimes
- legacy readability and no silent downgrade
- release/format compatibility claims

## Improvement Direction

言語非依存manifestでinput artifact、writer/source revision、schema axes、canonical bytes、full digest、public/legacy IDs、allowed operations、expected diagnostics、checksumを記録する。各runtimeは同じcorpus readerを実装し、actual writer artifactはimmutable fixtureとして保存する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: checked-in immutable corpus + independent runner | drift/provenance/negative casesを一括証明 | fixture更新reviewが必要 | adopt |
| B: runtime内hard-coded vectorsを増やす | 導入が容易 | provenanceと複合互換性が弱い | reject |

## Recommended Sequence

1. **Characterize**: roadmap matrixの最小fixture classとnegative vectorsを凍結する。
2. **Separate**: corpus manifest schemaとruntime adapterを定義する。
3. **Implement**: canonical/object vectorsからrepository/remote/signed fixtureへ拡張する。
4. **Prove**: Rust、Python、minimal independent readerで同じexpected resultを得る。
5. **Promote**: Stage 2 public schema/release compatibility gateへ接続する。

## Acceptance Signals

- 全fixtureにwriter provenance、checksum、schema axes、expected operation matrixがある。
- Rust/Python/minimal consumerが同じcanonical/full-digest decisionを返す。
- unknown required schema、collision、corruptionは全consumerで明示失敗する。

## Verification Design

- Focused tests: corpus manifest/schema/checksum、canonical edge cases、legacy/current ID
- Cross-boundary tests: writer→reader→migrate→reader、local↔remote signed graph
- Scale or fault tests: artificial prefix collision、truncated/missing root、mixed-version history
- Observability: fixture ID、writer、consumer、decision、digest。環境依存path/timeを除外

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-038/080やCFB-125は個別runtime/identity問題。本候補はroadmap既記載の未実装corpusで、正式issueを置換しない。
- Required predecessor: CFB-125完了とRVC-053/057のsupport/version decision
- Can combine with: RVC-022（canonical JSON profile/version negotiationを定義し、本候補が同じrootをcross-runtime corpusで証明する）、RVC-053、RVC-054、RVC-056、RVC-057

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- Rust/Pythonの全human textを一致させるparity suiteにはしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
