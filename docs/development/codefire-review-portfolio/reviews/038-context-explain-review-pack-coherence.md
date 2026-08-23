# RVC-038: context・explain・review-packのsubject/provenanceを揃える

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | product / architecture / UX / evolution |
| Review kind | refine |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 2 |
| Primary scope | `context.rs` / `explain.rs` / `view::review_pack_with_options` |
| Promotion target | product spec |

## Executive Judgment

contextのbounded live snapshot、explainの目的別要約、review-packのsealed comparisonはそれぞれ有用で、同じpayloadへ無理に統合すべきではない。ただし三者が扱うsubject、base、freshness、provenanceの共通descriptorがなく、同じ「Fire/Atom/verification」がlive previewかactive snapshotかsealed commitかをconsumerが個別解釈する。共通AssessmentSubjectを導入し、用途固有payloadは維持するべきである。

## Scope and Review Question

- In scope: subject identity、live/sealed claim、base/head、freshness、limits、provenance、cross-command navigation
- Out of scope: prompt/context pack for AI運用、review-packの外部配信、diff algorithm詳細
- Question: 利用者が三つのviewを行き来しても、同じsubjectと異なるsnapshotを取り違えず、差の理由を説明できるか

## Evidence

- `crates/codefire-cli/src/context.rs` — `context_snapshot` / `context_data_json`: active scan fingerprintをcurrent previewと比較し、snapshot_source、active_scan_fresh、omission countsを明示する。
- `crates/codefire-cli/src/explain.rs` — `explain_fire` / `explain_verify_failure`: compute_scan/compute_verifyのread-only再計算結果を独自codefire_explain payloadへ変換する。
- `crates/codefire-cli/src/view.rs` — `review_pack_with_options`: sealed base/source commitからfile diff、semantic diff、verification summary、next actionsを一artifactへ組み立てる。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs` — context freshness/boundsと`review_pack_exports_file_atom_verification_and_next_actions` は個別shapeを固定するが、同一subjectを跨ぐprovenance equivalence fixtureはない。

## Strengths to Preserve

- contextがstale active scanを黙って返さずpreview recomputationとomissionを明示すること。
- review-packがsealed base/sourceとfile/Atom/Trace/verificationを一つのreview artifactへまとめること。

## Gap / Risk

### Observed gap

contextはopen branch/base、explainはtarget中心、review-packはsealed pair中心で、共通subject ID、claim kind、freshness、source roots、generation ruleがない。

### Failure or stagnation mode

consumerがlive explain結果をsealed evidenceのように保存する、または古いreview-packとcurrent contextを同じstateとして比較する。payload間の重複fieldが独立進化し数値やnext actionがずれる。

### Affected invariants

- claim provenance
- freshness honesty
- bounded automation context
- review reproducibility

## Improvement Direction

AssessmentSubjectにkind（live_open/active_snapshot/sealed_commit_pair）、repo/branch/open instance、base/source/head、content fingerprint、freshness、generated-from roots、limitsを定義する。各commandはこのdescriptorと用途固有bodyを返す。explainはcontext/verification snapshotを再利用し、review-packは完全sealed subjectとしてdescriptorを埋め込む。値が異なる場合はstale/recomputed/different-claim reasonを返す。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 共通subject/provenance descriptor | view間の比較と保存claimが明確になる | schema追加とproducer移行が必要 | adopt |
| B: 三commandを一つへ統合 | 重複を減らせるように見える | live操作とsealed reviewの目的差を壊す | reject |

## Recommended Sequence

1. **Characterize**: 各payloadのsubject、freshness、limits、next actionをfield map化する。
2. **Separate**: common descriptorとcommand固有bodyをschema上分ける。
3. **Implement**: context/explain atomで同一snapshot descriptorを共有する。
4. **Prove**: live edit、active stale、sealed pairのcross-command consistency fixtureを作る。
5. **Promote**: claim kindとprovenanceをStage 2 schema/product specへ昇格する。

## Acceptance Signals

- 三payloadがsubject kind、IDs、freshness/immutability、limitsを同じschemaで返す。
- 同じsnapshotを指すcontext/explainの主要countが一致し、不一致にはreasonがある。
- review-packはsealed rootsだけをsubjectとし、live previewと混同できない。

## Verification Design

- Focused tests: descriptor construction、fingerprint、stale reason、limit metadata。
- Cross-boundary tests: context↔explain↔review-pack navigationとschema conformance。
- Scale or fault tests: edit中snapshot、large omissions、missing active scan、corrupt sealed root。
- Observability: claim kind、source/fingerprint、freshness、omitted count、recompute reasonを出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-074/114/136/137はcontext再構築・freshness・omission・branch selectionをfixed済み。CFR-087はreview-pack JSON metadataをfixed済み。本件はcross-view provenanceで非重複。
- Required predecessor: RVC-033のschema family、RVC-034/035のdiagnostic/action semantics。
- Can combine with: RVC-039。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- context payloadを外部AI prompt orchestration formatにしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
