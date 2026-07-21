# RVC-011: single committable predicateをtyped decisionとして完成させる

## Review Summary

| Field | Value |
|---|---|
| Status | implemented |
| Perspective | correctness / domain / architecture / testing |
| Review kind | redesign |
| Priority | P0 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `codefire-core::Verification / commit / exit_code / automation` |
| Promotion target | CFR-161 / ADR-013 |

## Executive Judgment

`build_verification`をdomain truthとしてcommitが`result`を参照する流れは正しい。しかしexit priority、blocking diagnostic、human summary、next action、certificate projectionがcollectionsを再解釈しており、policy variant追加時のdrift余地が残る。bool/string resultではなく、reason-coded `CommittableDecision`を一度だけ構築して全surfaceがprojectionする形へ完成させるべきである。

## Scope and Review Question

- In scope: blocker classification、policy applicability、decision reason、exit code priority、diagnostics、certificate facts、commit gate
- Out of scope: policy内容そのもの、command rendererの見た目だけの統一
- Question: 同じ入力に対しcore、CLI、JSON、certificate、storeが一つのauthoritative decisionを矛盾なく表現するか

## Evidence

- `docs/development/strategy/engineering-principles.md` — `One Authoritative Predicate per Decision`: committable predicateを最初のproduct-correctness targetとする。
- `crates/codefire-core/src/lib.rs` — `build_verification`: policy booleansとblocker collectionsから`passed/failed`を一度計算する。
- `crates/codefire-cli/src/exit_code.rs` — `verification_exit_code`: resultと各collectionをpriority順に再検査してprocess exit classを決める。
- `crates/codefire-cli/src/automation.rs` — `verification_diagnostics_json_with_filter` / `verification_next_actions`: blocking/severity/actionを再びfieldごとの条件で構築する。
- `crates/codefire-cli/src/main.rs` — `run_commit`: `verification.result != "passed"`をgateにし、同じobjectからcertificate countを再projectionする。
- Test/fixture evidence: exit priority、diagnostic、certificate mismatchの個別testsはあるが、全policy×blocker combinationを全surfaceで照合する単一decision tableは未完成。

## Strengths to Preserve

- commitがfresh verificationを実行し、rendererではなくcore resultで拒否すること。
- certificateをverificationから導出し、storeがresult/count一致を再検証すること。

## Gap / Risk

### Observed gap

`Verification`はraw factsとdecisionを同居させるが、どのfactsがblockingか、primary reasonは何かをtypedに持たない。各consumerが`trace_completeness_required`等を知って再判定する。

Installed runtime characterizationでこのdriftは仮説ではなくsupported-path bypassになった。
`require_no_required_fires: false` とrequired Fire 1件の組合せは、coreでは`passed`、process exit 0、
envelope `ok=true`、next action `commit`、commit dry-run `blocked=false`になる一方、同じverify envelopeは
`blocking=true` diagnosticも返す。証跡は`/tmp/codefire-rvc011-{verify,commit-dry}.json`、正式issueは
[CFR-161](../../code-review-issues/cfr-161.md)である。

### Failure or stagnation mode

新blockerやwarning policy追加時に、commitは拒否するがJSONはwarning、exit codeはgeneric、certificate countは欠落というfalse/opaque completionが再発する。

### Affected invariants

- commit blocks on every applicable blocker
- process exit = envelope exit / certificate truth
- rendering never decides validity

## Improvement Direction

raw `VerificationFacts`と、policyを適用した`CommittableDecision { outcome, blockers: Vec<TypedReason>, warnings, primary_reason }`を分ける。exit、diagnostics、next actions、state、certificateはreason registryのtotal mappingから生成し、unknown reasonをsuccessへfallbackしない。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: consumerごとのtestsを追加 | 小変更 | decision重複は残る | reject |
| B: typed decision + total projections | semantic driftを構造的に防ぐ | format/migrationとrefactor | adopt |

## Recommended Sequence

1. **Characterize**: blocker presence×policy×multiple blockers×unknown reasonのcomplete decision tableを作る。
2. **Separate**: facts収集、policy evaluation、decision、surface projectionを型/module分離する。
3. **Implement**: reason enum/registryとexit/diagnostic/certificate projectionを一つずつ置換する。
4. **Prove**: property/table testsで全surfaceのoutcome/count/reason同値性を検査する。
5. **Promote**: STR-X07、product invariants、source docs、必要な正式CFRへ昇格する。

## Acceptance Signals

- 全decision-table rowでcommit、process exit、JSON `ok/exit_code`、state、certificateが一致する。
- 新reason追加時に未実装projectionがcompile/test failureとなり、successへ黙ってfallbackしない。
- warning-only policyでfactsを失わずpassedを正しく表現する。

## Verification Design

- Focused tests: blocker/policy Cartesian table、primary priority、unknown version、warning-only。
- Cross-boundary tests: core decision → verify CLI → commit dry-run/apply → certificate → store validation。
- Scale or fault tests: 多数diagnostics、corrupt/missing fact、stale cached verification、concurrent mutation。
- Observability: stable reason code、blocking flag、policy rule ID、total/omitted countsを共通projectionで出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-122/127/140/159とCFB-003/014/028は個別drift修正。STR-X07が本候補と直接対応し、installed counterexampleをCFR-161へ昇格した。RVC-008/016/034は別issueにせず同じformal rootへ統合する。
- Required predecessor: 凍結中CFB-120..126のsealed/lock invariant完了。
- Can combine with: RVC-008、RVC-019。portfolio横断ではRVC-060「統合改善ロードマップ」が本候補を最上位root-cause clusterとして統合し、個別renderer/exit/certificate修正より先にpredicate境界を昇格判断する。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- rendererやexit codeへ独自のvalidity ruleを残さない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] CFR-161とADR-013へ昇格し、Install/Dogfood/Retrospectまで完了した
