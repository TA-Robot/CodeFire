# AI-Native SDLC Control Plane

> **非正本 / exploration only**
>
> これはAI-assisted developmentの状態機械、work contract、control pointを検討する研究ノートである。採用された結論はdocs/development/ai/、docs/automation_interface.md、strategy、ADR、schema、test/evalへpromotionする。

## 1. Scope

対象:

- requirement/issue intakeからtrusted integrationまでのlifecycle
- human、manager、implementer、reviewer、CI、CodeFireの責務
- Task、Context、Patch、Evidence、Review、Certificateの結合
- deterministic gateとmodel judgmentの境界
- long-running task、retry、cancellation、stale work、parallel lane

対象外:

- 特定provider SDKの実装詳細
- prompt文面の最終形
- CodeFire product内のagent runtime、scheduler、marketplace
- GUIの最終interaction design

## 2. Problem Statement

documented: CodeFireは、AIによる変更速度が仕様、設計、test、operationsの更新速度を超え、意味的な乖離が正式履歴へ入る問題を対象にしている。

inferred: AI-native SDLCの中心問題はpatch generation速度ではない。次の問いへ再定義した方がよい。

> どの変更が、何を根拠に、誰の責任で、どの不変条件を満たし、どのevidenceとreviewを経て統合されたか。

North Star candidate:

MET-AI-001 Trusted Integration Throughput

一定期間に、typed taskへ結び付き、required deterministic gatesを通り、独立reviewとcomplete evidenceを持って統合されたchange package数。

raw commit数、generated LOC、agent数、tool call数はNorth Starにしない。

## 3. Actors And Trust Boundaries

| Actor | Primary responsibility | Trust assumption |
|---|---|---|
| Human requester | objective、authority、business acceptance | intentのsourceだがtechnical correctnessは未保証 |
| Manager agent | decomposition、risk routing、ownership、integration | planはuntrusted until checked |
| Implementer agent | bounded patch、focused tests、handoff | patchとself-reportはuntrusted |
| Reviewer agent | independent challenge、claim/evidence audit | model judgmentは補助evidence |
| Validation runner | exact command executionとresult capture | environment identityが必要 |
| CI | isolated repeatable gate、signed evidence | runner identityとworkflow digestが必要 |
| CodeFire kernel | graph、Fire、verification、seal | product invariantとobject validationへ依存 |
| External MCP/A2A service | context/tool/task execution | capability、consent、auth、output validationが必要 |
| Human integrator | final exception、risk acceptance | exceptionはrationaleとauthorityを残す |

原則:

- agent outputは提案であり、evidenceではない。
- command実行報告は、command、cwd、environment、exit、output digestを持って初めてevidence candidateになる。
- reviewer approvalだけでdeterministic blockerを上書きしない。
- CodeFire certificateは絶対的正しさではなく、versioned policy下の整合性を示す。

## 4. Candidate Lifecycle

~~~text
observed need
  -> proposed work
  -> packet-ready
  -> context-bound
  -> executing
  -> patch-produced
  -> validation-pending
  -> review-pending
  -> integration-ready
  -> sealed
  -> distributed
  -> evaluated
~~~

例外状態:

~~~text
blocked-authority
blocked-environment
stale-base
write-set-conflict
validation-failed
evidence-incomplete
review-requested-changes
policy-rejected
cancelled
superseded
~~~

各transitionは自然言語だけでなくtyped preconditionを持つ。

| Transition | Required input | Required output/evidence |
|---|---|---|
| proposed -> packet-ready | objective、authority、scope | Task Packet digest |
| packet-ready -> context-bound | base revision、selectors | Context Pack digestとcoverage report |
| context-bound -> executing | owner、lane、runtime profile | run ID、effective model/effort、sandbox |
| executing -> patch-produced | bounded write set | patch digest、changed paths、handoff |
| patch-produced -> validation-pending | patch/base binding | validation plan |
| validation-pending -> review-pending | required gates complete | command evidence、failure/skip record |
| review-pending -> integration-ready | independent review | Review Packet digest、decision |
| integration-ready -> sealed | no blocker、fresh inputs | consistency certificate |
| sealed -> distributed | verified subject digest | remote/PR/artifact reference |
| distributed -> evaluated | outcome telemetry | regression/eval record |

## 5. Work Graph

candidate representation:

~~~text
WorkGraph = (Tasks, Dependencies, Ownership, Artifacts, Gates)
~~~

Task edge types:

- requires: predecessor outputが必要
- validates: deterministic or review gate
- reviews: independent review relation
- blocks: unresolved issue or authority
- supersedes: old packetを無効化
- produces: patch/evidence/certificate

HYP-AI-001:

hypothesis: task packetとartifactをDAGへ置き、base digestとdependency digestを固定すると、長期taskのhidden dependencyとstale workを減らせる。

反証条件:

- 日常taskの大半でgraph maintenanceが実装時間を上回る。
- dependency宣言が実際のfile/module dependencyを捉えず、false confidenceを増やす。
- stale detectionが頻発し、safe rebaseよりtask再作成を強制してthroughputが低下する。

測定:

- undeclared dependency件数。
- stale-baseで破棄されたwork時間。
- packet作成時間。
- write-set conflictとintegration conflictの件数。

対応実験: EXP-AI-003、EXP-AI-004。

## 6. Control Plane Versus Execution Plane

decision-candidate: CodeFireはexecution planeを持たず、control/evidence contractだけを提供する。

Control plane candidate:

- Task/Context/Review schema
- capability discovery
- Fire/diagnostic/context retrieval
- validation/evidence ingestion
- certificate and policy evaluation
- event/trace/provenance export

Execution plane:

- model invocation
- subagent scheduling
- prompt assembly
- worktree/process lifecycle
- retry/backoff
- human approval UI

境界の利点:

- provider lock-inを避ける。
- product invariantとagent behaviorを独立versioningできる。
- Codex、他社agent、human-only workflowが同じcertificate contractを共有できる。
- orchestration failureがCodeFire repository semanticsを直接壊しにくい。

RISK-AI-001:

control planeの名目でagent marketplace、prompt library、schedulerをcoreへ取り込み、product scopeを再拡大する。

mitigation candidate:

- core command追加はAtom/Trace/Fire/Evidence/Verification/Sealのいずれかへ直接属することを要求する。
- orchestration convenienceはadapterまたはdocs/development/ai/へ置く。

## 7. Deterministic And Judgment Gates

候補となる評価順序:

1. parse/schema validation
2. path/workspace/permission validation
3. CodeFire invariants and policy
4. compiler/test/static analyzer
5. reproducible integration/dogfood
6. independent human/agent review
7. model grader/trace grader

上位gateが失敗したとき、下位のreviewやgraderはoverrideしない。

HYP-AI-002:

hypothesis: model reviewをdeterministic gateの後に置くと、reviewer contextをsemantic riskへ集中でき、token costとfalse approvalを減らせる。

反証条件:

- reviewerが先にarchitecture defectを見つけた方が不要なfull validationを大幅に減らす。
- deterministic gate自体が高costで、明白なscope errorをreview前に検出できない。

実験ではcheap structural review、focused gate、deep review、wide gateの順序も比較する。対応実験: EXP-AI-005。

## 8. Completion Claim

completion candidate:

~~~text
Complete(Task) =
  objective_satisfied
  and acceptance_criteria_all_proven
  and patch_digest_matches_review
  and required_gates_fresh
  and required_evidence_resolves
  and no_blocking_fire
  and no_unowned_followup
~~~

次はcompletion evidenceにならない。

- agentの「完了しました」という文章。
- tests not runを含む曖昧なhandoff。
- patch作成前のreview。
- base/headが不明なreview。
- stale Context Packを前提にしたvalidation。
- target外fixtureだけのpass。

DC-AI-010:

release completion auditとtask completion auditへ、claim-to-evidence matrixを必須化する。

## 9. Learning Loop

各失敗を次のいずれかへ変換する。

| Failure | Durable destination |
|---|---|
| product defect | CFB/CFR issue、regression test |
| packet ambiguity | Task Packet schema/template |
| context miss | selector/context pack rule |
| invalid routing | model routing policy/eval case |
| tool misuse | MCP permission/tool contract |
| flaky gate | validation policy/environment probe |
| review miss | seeded eval corpus/grader criteria |
| adoption friction | runbook、installer、adapter backlog |

traceや会話全体を永久保存するのではなく、reusable rule、minimal failing fixture、decision evidenceへ圧縮する。

## 10. Failure Modes

| Failure mode | Detection candidate |
|---|---|
| plausible patch, wrong objective | acceptance assertionとhidden eval |
| correct patch, stale base | base/patch digest mismatch |
| tests pass, wrong scope | write-set and changed-path check |
| reviewer saw different patch | review target digest |
| evidence forged by agent | capture trust level、CI signature |
| parallel lane overwrites user work | lane ownership、pre-apply diff、serialized integration |
| context omits critical invariant | coverage manifest、reviewer context challenge |
| long thread forgets constraint | explicit durable packet、compaction-independent state |
| certificate overclaims truth | policy/version/subject明記 |

## 11. Decision Candidates And Promotion

- DC-AI-011: AI-native SDLCをtyped transitionとartifact digestで定義する。
- DC-AI-012: manager/agent conversationではなくTask Packetをwork identityの正本にする。
- DC-AI-013: deterministic gateをreview/model graderより上位のauthorityにする。
- DC-AI-014: completion claimへclaim-to-evidence matrixを要求する。
- DC-AI-015: failure learningをraw transcript保存ではなくfixture、rule、issueへ圧縮する。

Promotion先:

- lifecycle/state contract: docs/development/ai/operating-protocol.md
- machine contract: schemas/、docs/automation_interface.md
- product invariant: docs/01_product_definition.md、docs/05_consistency_model.md
- architecture boundary: ADR
- eval: dedicated eval corpus and CI

## 12. Promotion Status

- Status: not promoted
- Required evidence: EXP-AI-003、EXP-AI-005、EXP-AI-006
- Revisit when: typed packet prototypeとhistorical task replayが完了したとき

