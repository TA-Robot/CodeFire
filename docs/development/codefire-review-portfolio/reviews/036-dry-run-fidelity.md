# RVC-036: dry-run fidelityをread-setとrevalidationで定義する

## Review Summary

| Field | Value |
|---|---|
| Status | implemented |
| Perspective | correctness / architecture / UX |
| Review kind | redesign |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `--dry-run` operation plans across mutating commands |
| Promotion target | future CFR / ADR |

## Executive Judgment

多くのmutation commandがno-write planとfocused regressionを持つことは大きな強みである。ただしdry-run時のlock、context reload、remote/layout validationはcommandごとに異なり、「同じ入力ならapplyと同じ判断」なのか「ある時点のadvisory preview」なのかがplanに表現されない。dry-runを弱い別実装にせず、共有plan builderとread-set fingerprint、apply時revalidationで契約化すべきである。

## Scope and Review Question

- In scope: dry-run validation、lock strategy、read-set freshness、plan/apply parity、no-write guarantee
- Out of scope: dry-run中のverification command実行を常に省略すること、UI preview
- Question: dry-run結果が何を検証し、どの状態まで有効で、apply前に何を再確認するかを機械的に説明できるか

## Evidence

- `crates/codefire-cli/src/main.rs` — `merge_branch` / `build_merge_result` / `refresh_open_clean_for_branch_operation`: dry-runでもtarget refreshが`compute_scan(open_dir, true)`へ進み、applyをskipする前にFire、active state、registryをpersistし得る。
- `crates/codefire-cli/src/fire.rs` — `run_manual_fire_specs`: dry-runはlockなしでcontextを読み、live pathはlock取得後にcontextを再loadして同じfire builderへ進む。
- `crates/codefire-cli/src/link_batch.rs` — `run_link_batch`: dry-runだけlockを省略するが、同じresolve/validate/planを使い、live invalidならwrite前に失敗する。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs` — `commit_dry_run_returns_plan_without_updating_branch`、`open_and_clone_dry_run_return_plans_without_structural_changes` 等はno-writeを固定するが、plan後の状態変化とapply revalidationの共通contractはない。

## Strengths to Preserve

- operation planにoperations、would_apply、対象path/branchを含めること。
- liveとdry-runが多くのcommandで同じvalidation/builderを共有すること。

## Gap / Risk

### Observed gap

lockを取るdry-runと取らないdry-runが混在し、planのfreshnessやvalidated checksが共通fieldで表されない。さらにmerge dry-runのtarget refreshはpersisting scanを呼び得るため、repository metadataを変更しないという契約自体にcurrent gapがある。

### Failure or stagnation mode

merge previewだけでFire/open state metadataが変わる、またはplan後にhead、open state、batch file、policyが変わっても古いactionを実行する。逆にread-only previewのために長時間lockを保持し、通常workflowを不必要に止める。

### Affected invariants

- dry-run no-write
- validate-before-apply
- stale plan rejection
- bounded lock liveness

## Improvement Direction

PlanEnvelopeにvalidated_checks、read_set、subject fingerprint、created_from_state、requires_revalidationを持たせる。plan builderはdry/live共通とし、applyはexpected head/base/open instance/input digestを再確認してからmutationする。dry-run lockはsnapshot consistencyに必要な最短区間だけ選び、lockなしならadvisory/staleableと明示する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: shared plan＋read-set fingerprint＋apply revalidation | no-writeとfidelityを両立する | plan schemaと各commandのread-set定義が必要 | adopt |
| B: 全dry-runでliveと同じlockを最後まで保持 | snapshotは安定する | 長時間previewでlivenessを悪化させる | reject |

## Recommended Sequence

1. **Characterize**: command別にvalidation、external effects、lock、read-set、apply差分を表にする。
2. **Separate**: plan buildとapply/revalidateをfocused moduleへ分ける。
3. **Implement**: commit、merge、batch一種でPlanStampをvertical sliceする。
4. **Prove**: plan後に各read-set要素を変え、stale applyがwrite前に止まるtestを作る。
5. **Promote**: dry-run claim、freshness、revalidationをADR/CLI specへ昇格する。

## Acceptance Signals

- dry-runはfilesystem/repository/remoteのdocumented writeを0件にする。
- dry/liveが同じinput snapshotなら同じvalidation outcomeとplanned operationsを返す。
- snapshot変化後のapplyはstale-plan diagnosticを返すか、明示的に再planして差分を提示する。

## Verification Design

- Focused tests: read-set digest、no-write watcher、stale base/head/policy/input。
- Cross-boundary tests: commit/merge/open/link/fire/extinguish/remote dry-run contract table。
- Scale or fault tests: plan中race、lock timeout、verification command timeout、remote state change。
- Observability: validation coverage、lock mode、fingerprint、revalidated fields、plan ageを出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-039/043/070/072/087/088等とCFR-108/112/129は個別dry-run不具合をfixed済み。本件はmerge target refreshの未採番counterexampleと横断fidelity contractを扱うため非重複。
- Required predecessor: RVC-026のlock model、RVC-031のlifecycle。
- Can combine with: RVC-035、RVC-037。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- dry-runを将来のapply成功の無条件予約として保証しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
