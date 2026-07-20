# RVC-051: `main.rs` responsibility decomposition

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | architecture / correctness / testing |
| Review kind | redesign |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `crates/codefire-cli/src/main.rs` |
| Promotion target | future CFR / ADR |

## Executive Judgment

remote、HTTP、evidence、doctor、view等をfocused moduleへ分けた方向は正しい。しかし`main.rs`は約8千行で、dispatch/parser/renderingだけでなくopen/clone/merge/commit transaction、filesystem、locks、ancestry、repository recordsまで所有しており、Ultra invariant変更の影響範囲を狭められていない。

## Scope and Review Question

- In scope: `main.rs`が持つproduction責務、抽出順序、public/private API、orchestration kernel
- Out of scope: command追加、CLI UX変更、crate全面rewrite
- Question: top-level dispatchを薄くしつつ、transaction invariantを分断せずfocused moduleへ移せるか

## Evidence

- `crates/codefire-cli/src/main.rs` — `run`: help/capability routingから全top-level command dispatchとoutput分岐を担う。
- `crates/codefire-cli/src/main.rs` — `open_branch_from` / `clone_branch` / `merge_branch`: lifecycle validation、plan、lock、mutation、resultを同一fileに置く。
- `crates/codefire-cli/src/main.rs` — `run_commit` / commit transaction helpers: verification、multi-object write、branch publication、active reset、idempotencyを所有する。
- `crates/codefire-cli/src/main.rs` — `absolute_path` / `write_json_atomic` / `RepoLock` / `common_ancestor` / `materialize_commit`: filesystem・lock・algorithmの低層責務も混在する。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs`が`use super::*`でprivate main internalsへ強く結合し、抽出境界を固定するcontract testが不足する。

## Strengths to Preserve

- module ownership表と`command_registry`があり、新規責務をfocused moduleへ置く原則は明文化済みである。
- commit等のtransaction codeは現状一箇所にまとまり、処理順を追うこと自体は可能である。

## Gap / Risk

### Observed gap

過去CFR-001をfixedとした後も、未抽出workflowと共通utilityがmainへ蓄積した。抽出先ごとのinvariant/APIと完了条件がなく、局所module化だけではfile責務が減らない。

### Failure or stagnation mode

異なるcommand修正が同じfile/testへ集中し、parse/validate/plan/apply/render境界やlock順序を壊す。private helper依存のためfocused testを置きづらく、refactorが大差分になる。

### Affected invariants

- branch/open identity and transaction ordering
- sealed commit publication
- parse/validate/plan/apply/render separation

## Improvement Direction

行数ではなく責務で分割し、`main`をbootstrap/dispatch/error boundaryへ限定する。第一候補は`repository_io`、`locking`、`open_lifecycle`、`branch_lifecycle`、`commit_workflow`、`merge_workflow`、`ancestry`で、各moduleはtyped plan/resultと狭いcontext interfaceを持つ。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: behavior-preserving seam-by-seam extraction | regressionを抑え責務を明確化 | 複数小batchが必要 | adopt |
| B: main.rs全面rewrite | 一度に整う可能性 | invariant regressionとreview不能な大差分 | reject |

## Recommended Sequence

1. **Characterize**: symbol→責務→invariant→testのownership mapを作る。
2. **Separate**: pure helper/typed model、次にIO adapter、最後にworkflowを小さく抽出する。
3. **Implement**: lifecycle/commitを薄いdispatchからfocused moduleへ委譲する。
4. **Prove**: CLI golden、state snapshot、failpoint結果が抽出前後で一致する。
5. **Promote**: module ownership docsと必要な限定CFRへ反映する。

## Acceptance Signals

- `main.rs`がbootstrap、top-level route、final render/error境界以外のdomain mutationを持たない。
- 各mutating workflowのvalidation/plan/applyとowner testが同じmoduleから追える。
- 抽出commitごとにoutput/state/fault goldenが完全一致する。

## Verification Design

- Focused tests: 新moduleのpure plan、precondition、write ordering
- Cross-boundary tests: top-level dispatch→module→envelope/state snapshot
- Scale or fault tests: commit/open/mergeの既存failpoint/corruption fixtureを再利用
- Observability: module/phase名は安定させ、抽出でmetrics意味を変えない

## Dependencies and Duplication Check

- Existing CFB/CFR: fixed CFR-001/002/008/040とCFB-009の再発防止を評価する候補であり、同じ「fileが大きい」issueを再採番しない。具体的seamが確定した場合だけfuture CFRへ分割する。
- Required predecessor: RVC-054/056のcharacterization test、凍結batch完了
- Can combine with: RVC-031（command lifecycle contractをownerとし、本候補はそのorchestration seamを`main.rs`から抽出するenabler）、RVC-052、RVC-054、RVC-060

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- file行数だけをKPIにした機械分割をしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
