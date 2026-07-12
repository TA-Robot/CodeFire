# Phase 1: Computer Science / Software Engineering Exploration

> **非正本 / exploration only**
>
> このdirectoryは、CodeFireの設計方針を決める前の検討材料である。未検証の仮説、相互に競合する案、棄却予定の案を含む。現在状態、実行手順、製品仕様、採用済みarchitectureの正本として参照してはならない。

## 目的

CodeFireを「AI時代の整合性優先version-control style CLI」として発展させる前に、computer scienceとsoftware engineeringの観点から次を明確にする。

1. すでに守れている不変条件と、文書上だけ主張していて未証明の性質を分ける。
2. correctness、durability、security、scale、usabilityを反証可能な仮説へ変える。
3. 実験で支持された結論だけをstrategy、ADR、product docs、test/evalへ昇格する。
4. 探索文書自体が新しい恒久的な文書負債にならないよう、promotionまたは削除の出口を持つ。

## Evidence Snapshot

- 調査日: 2026-07-11
- 対象: このworktree上のRust default CLI、product docs、development docs、AI acceleration docs
- 主な根拠:
  - `docs/02_core_concepts.md`
  - `docs/05_consistency_model.md`
  - `docs/08_object_store.md`
  - `docs/09_index_trace_policy.md`
  - `docs/10_merge_remote_server.md`
  - `docs/development/source-architecture.md`
  - `docs/development/source-data-model-catalog.md`
  - `docs/development/known-limitations.md`
  - `crates/codefire-core/src/lib.rs`
  - `crates/codefire-store/src/lib.rs`
  - `crates/codefire-cli/src/main.rs`
- 制約: 現在のworktreeには未保存の変更があるため、この探索はrelease済みbaselineの証明ではない。

## Evidence Labels

親directoryの規則に従い、重要な主張へ次のlabelを付ける。

| Label | このphaseでの扱い |
|---|---|
| `observed` | source、test、runtime、worktreeで直接確認した事実 |
| `documented` | CodeFireのproduct/development文書に明記された主張 |
| `inferred` | 複数の観察から導いたが未実験の結論 |
| `hypothesis` | 反証条件と実験が必要な主張 |
| `decision-candidate` | strategy、ADR、仕様への昇格候補 |
| `rejected` | 棄却理由と再検討条件を持つ案 |

## 文書一覧

| 文書 | 主題 | 主なpromotion先 |
|---|---|---|
| `01-system-model-and-formal-invariants.md` | 状態機械、安全性、活性、形式化 | `docs/05_consistency_model.md`, ADR, core tests |
| `02-transactions-storage-and-protocol-evolution.md` | transaction、CAS、schema、remote consistency | `docs/08_object_store.md`, `docs/10_merge_remote_server.md`, ADR |
| `03-vcs-graphs-algorithms-and-scale.md` | commit DAG、diff/merge、Trace Graph、計算量 | product specs, algorithm docs, benchmarks |
| `04-security-observability-and-recovery.md` | threat model、diagnostics、repair、supply chain | security ADR, runbook, doctor/remote design |
| `05-testing-hci-and-context-engineering.md` | property/model test、eval、HCI、context retrieval | test strategy, AI protocol, automation interface |
| `hypothesis-register.md` | 仮説、反証条件、evidence、promotion状態 | strategy/ADR/backlogへの入口 |
| `experiment-catalog.md` | 再現可能な実験計画 | tests, benchmark/eval suites, evidence records |

## CodeFireを考える共通モデル

このphaseでは、CodeFireを次の5つのmachineが接続したsystemとして扱う。

```text
Workspace machine
  working files -> Atoms -> Trace Graph -> Fires -> Resolutions -> Verification

Sealing machine
  verified workspace -> immutable root objects -> sealed commit -> branch-head publish

Repository machine
  branch/opened/active/idempotency/lock metadata -> state derivation and recovery

Remote machine
  auth/signature/policy -> object transfer -> fast-forward/MR mutation -> audit

Automation machine
  parse -> validate -> plan -> apply -> render -> JSON envelope/next_actions/metrics
```

この分解は採用済みarchitectureではなく、問いと実験を漏れなく配置するための探索上のモデルである。

## 直接確認した重要観察

1. `documented`: branch headはsealed commitだけを指し、object storeはimmutableである。
2. `documented`: commit可能条件はopen required fire、failed check、missing required link、stale resolution、duplicate Atom IDが0であることとして記述される。
3. `observed`: core実装はmissing evidence referenceも常にverification blockerにする。仕様述語、certificate、実装、automation contractの完全一致はまだ証明されていない。
4. `documented`: commitはimmutable objectsの保存後、branch head、opened registry、active stateを更新するmulti-file operationで、pending markerを持つ。
5. `observed`: branch listはbranchごとにsealed commit graphをfresh traversalし、共有historyを再読込しうる。
6. `observed`: public object IDはSHA-256 digestの先頭24 hexを使い、recordにはfull digestを持つ。
7. `inferred`: short ID collision時に既存full digestと新規full digestが異なる場合の明示的なerror contractを、保存経路全体で確認する必要がある。
8. `observed`: resolution basisはsource/target content hash、関連Trace Link hash、policy hashを持つが、extractor versionは直接含まない。
9. `documented`: server-side verificationはcwd/env/timeout/outputを制約するが、OS-level sandboxはない。
10. `observed`: repository lockはlock fileのcreate-newとDrop時削除に依存し、process crash後のstale lock recoveryは明文化されていない。

## Identifier Rule

| Prefix | Meaning |
|---|---|
| `Q-*` | 設計問い |
| `INV-*` | 守るべき安全性・活性・契約 |
| `HYP-*` | 反証可能な仮説 |
| `RISK-*` | failure modeまたは未管理のrisk |
| `MET-*` | 測定指標 |
| `EXP-*` | 再現可能な実験 |
| `DC-*` | decision candidate |

IDは文書間で一意にする。仮説の正本は `hypothesis-register.md`、実験の正本はこの探索範囲では `experiment-catalog.md` とする。ただし、どちらもproduct current-stateの正本ではない。

## Cross-Cutting Metric Dictionary

個別実験の詳細指標に加え、分野をまたいで比較する指標を次のIDで扱う。target値は実測baselineとrisk判断を経てpromotion先で決める。

| ID | Metric | Interpretation |
|---|---|---|
| `MET-CORRECT-001` | false accepted invalid states | blocker、corruption、policy違反を正常成功として受理した件数。最重要のsafety metric |
| `MET-CORRECT-002` | false blocked valid states | 正当なworkspace/operationを誤って拒否した件数 |
| `MET-TXN-001` | invalid observable crash states | failure後にold/new/pending/corruptのいずれにも安全分類できない状態数 |
| `MET-REC-001` | safe recovery success and RTO | user work/reachable objectを失わず復旧できた割合と時間 |
| `MET-STORE-001` | identity agreement | runtime/version間でcanonical bytes、full digest、public IDが一致した割合 |
| `MET-PERF-001` | work amplification | 入力規模またはunique reachable entityに対するread/parse/hash/traversal回数 |
| `MET-SEC-001` | boundary escape or secret exposure | root外access、policy bypass、secret disclosureの成功件数 |
| `MET-EVAL-001` | invariant detection effectiveness | 対応test/reviewerがseeded invariant violationを検出した割合 |
| `MET-HCI-001` | time to first correct action | diagnostic受領から安全で正しい次操作を選ぶまでの時間 |
| `MET-CONTEXT-001` | oracle recall at token budget | 固定context budget内に必要file/invariantを含めた割合 |
| `MET-FLOW-001` | trusted integration lead time | task定義からreview/validation済み統合可能状態までの時間 |

## Phase 1 Decision Candidates

- `DC-FORMAL-001`: commit可否を一つのversioned predicateとして仕様・core・certificate・JSONへ共有する。
- `DC-TXN-001`: branch head publishをcommitの唯一のvisibility pointとして明文化する。
- `DC-STORE-001`: short object IDが一致してもfull digestが異なればcollisionとして拒否する。
- `DC-SEMVER-001`: Atom extraction semanticsへversion identityを与え、resolution validityへ反映する。
- `DC-PERF-001`: 一回のcommand内ではsealed graph validation結果をobject ID単位で共有する。
- `DC-SEC-001`: server verificationのtrusted deployment境界とsandbox requirementを明示する。
- `DC-EVAL-001`: invariant coverageとhistorical defect corpusをtest/evalの主要指標にする。
- `DC-CONTEXT-001`: context packをcommand/source/persistence/invariant/test graphから検証可能にする。

いずれもこの時点では未採用である。

## Promotion Rule

各decision candidateを昇格するときは、親directoryの規則に加えて次を満たす。

1. `hypothesis-register.md` の反証条件とevidenceを更新する。
2. 対応する `EXP-*` を実行し、command、fixture、environment、結果を残す。
3. alternative、trade-off、compatibility/migration impactを記録する。
4. promotion先の正本文書とtestを同じchange packageで更新する。
5. 探索文書へpromotion先、採用日、再検討条件を追記する。

Promotion先の原則:

| Finding | Promotion先 |
|---|---|
| product invariant / state semantics | product spec、`docs/05_consistency_model.md` |
| architecture decision | `adr/` |
| implementation boundary | `docs/development/module-boundaries.md`、source maps |
| defect | CFB/CFR issue ledger |
| repeatable test | Rust/Python test suite |
| performance claim | benchmark suiteとthreshold |
| AI workflow rule | `docs/development/ai/` |
| operational recovery | runbook / doctor docs |
| rejected idea | `synthesis.md`相当の最終要約だけに理由を残す |

## Exit Criteria

Phase 1を終了できるのは、次をすべて満たしたときである。

- 各分野に設計問い、仮説、反証条件、不変条件、risk、metric、experimentがある。
- P0仮説の実験可能性が確認されている。
- 「証明済み」「observed」「documented」「未証明」が混同されていない。
- decision candidateごとにpromotion先とowner候補がある。
- 採用しないscratchは削除または棄却理由だけへ圧縮されている。
- このdirectoryがactive backlogやcurrent-state sourceになっていない。
