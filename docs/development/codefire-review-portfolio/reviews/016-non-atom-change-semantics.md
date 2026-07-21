# RVC-016: non-Atom changeの意味とseal policyを定義する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | domain / correctness / product / UX |
| Review kind | refine |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `ScanResult.non_atom_changed_files / manifest / verification / commit` |
| Promotion target | product spec |

## Executive Judgment

full content manifestとAtom indexを併用し、Atom外の変更をscan/commit summaryへ戻した修正は重要である。ただしnon-Atom changeは現在「報告されるpending change」であり、確認責任・policy・coverage上の意味が曖昧である。全変更をAtom化するのではなく、分類とseal decisionをexplicit policyにすべきである。

## Scope and Review Question

- In scope: Atom外span、Atomを持たないfile、binary/generated/config、manifest差分、Fire/verification/commit summary
- Out of scope: 全fileを自動Atom化すること、binary内容のsemantic解析
- Question: non-Atom changeを含むcommitが何を確認済みとしてsealされたか、利用者とconsumerが判別できるか

## Evidence

- `docs/01_product_definition.md` — `Artifact and Atom`: 非Atom file変更もscanで見落とさず報告する要求がある。
- `crates/codefire-cli/src/main.rs` — `non_atom_changed_files_since_base`: base/current content manifestとchanged Atom pathsを比較し、added/deleted/modified file差分を作る。
- `crates/codefire-core/src/lib.rs` — `ScanResult` / `NonAtomFileChange`: changed Atomsと別collectionでpath/status/blob identitiesを表す。
- `crates/codefire-core/src/lib.rs` — `build_verification`: non-Atom collectionをblocker/policy inputとして参照せず、Fireやchecks等だけでresultを決める。
- `crates/codefire-cli/src/main.rs` — `commit_operation_plan` / `run_commit`: non-Atom filesをbounded summaryに含めるが、passed verificationならmanifestへsealする。
- Test/fixture evidence: CFB-115のout-of-span visibility testと`non_atom_file_changes_are_reported_in_scan_context_and_commit`は存在するが、file class別のrequired action/coverage testはない。

## Strengths to Preserve

- whole-tree content manifestがAtom model外のbytesもcommit identityへ含めること。
- non-Atom changeをcleanから隠さず、path/status/blobでbounded報告すること。

## Gap / Risk

### Observed gap

`non_atom`は意味分類ではなく「changed Atom pathではない」という差集合である。同じfileでAtom spanと外spanが同時に変わる場合、fileはchanged Atom pathとして除外され、外span変更を独立に説明できない。policyもgenerated/binary/config/docs indexを区別しない。

### Failure or stagnation mode

利用者がnon-Atom changeを「verification不要」と誤解し、重要なconfig/ops changeをFire/evidenceなしでsealする。逆にすべてblockするとgenerated outputで過剰摩擦になる。

### Affected invariants

- whole-tree change visibility
- declared coverage / no false completion
- consistent sealed state semantics

## Improvement Direction

`ChangeCoverage` read modelを導入し、Atom-covered span、uncovered span、non-Atom file、generated/ignored/unknown classを分ける。policyはclass/pathごとにallow-with-disclosure、require-review/evidence、denyを宣言し、unknownはcleanへしない。同一fileのmixed coverageも保持する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 全non-Atomを一律block | 見逃しに強い | generated/ordinary filesで採用不能 | reject |
| B: typed coverage + declared policy | 明示性と実用性を両立 | span tracking/policy設計 | adopt |

## Recommended Sequence

1. **Characterize**: Atom外intro、mixed-span、binary、generated、rename/delete、configのdecision corpusを作る。
2. **Separate**: byte-level manifest change、Atom semantic change、coverage classification、policy evaluationを分離する。
3. **Implement**: read-only coverage reportとunknown diagnosticから始める。
4. **Prove**: 全changed bytesがcovered/uncovered/ignored-with-ruleのいずれかへ決定的に分類されるtestを作る。
5. **Promote**: product definition、scan/verification schema、policy specへ昇格する。

## Acceptance Signals

- commit対象の全changed file/spanにcoverage classとpolicy dispositionがある。
- mixed Atom/non-Atom editがAtom changeだけに丸められない。
- legacy behaviorを選ぶprofileでもnon-Atom count/unknown claimが消えない。

## Verification Design

- Focused tests: mixed span、pre-heading prose、explicit marker外、binary/generated、rename/delete。
- Cross-boundary tests: manifest → extraction → scan coverage → verify → commit/certificate claim。
- Scale or fault tests: 巨大file、many spans、encoding failure、extractor crash、path class overlap。
- Observability: covered/uncovered/unknown/ignored counts、omitted paths、policy dispositionをbounded表示する。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-017とCFB-115、CFR-117はvisibility/shapeを修正済み。本候補はvisibleになったnon-Atom changeのsemantic policyを定義するため重複しない。
- Required predecessor: RVC-006 extractor span contractとRVC-011 decision model。
- Can combine with: RVC-001、RVC-019。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- すべてのbytesへ意味Atomを強制作成しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
