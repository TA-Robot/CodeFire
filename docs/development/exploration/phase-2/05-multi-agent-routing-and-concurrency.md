# Multi-Agent Routing And Concurrency

> **非正本 / exploration only**
>
> これはGPT-5.6 Sol、Medium/High/Ultra、manager/worker/reviewer、parallel lane、runtime adapterを検討する研究ノートである。採用された結論はdocs/development/ai/model-routing-policy.md、role-routing-matrix.json、.codex/、orchestration wrapper、eval suiteへpromotionする。

## 1. Observed And Documented Runtime Facts

observed locally:

- global configはgpt-5.6-solとmodel_reasoning_effort ultraを指定する。
- project .codex/config.tomlもgpt-5.6-solをprimary modelとし、Medium/High/Ultra role configを持つ。
- Codex CLI versionは0.144.1。
- bundled GPT-5.6 Sol catalogはlow、medium、high、xhigh、max、ultraを列挙する。
- bundled catalogはultraをMaximum reasoning with automatic task delegationと説明する。
- bundled catalogはlocal context windowを372,000、effective percentを95と報告する。

documented publicly:

- GPT-5.6 Sol API model pageはreasoning levelsをnone、low、medium、high、xhigh、maxとする。
- public Codex config referenceはmodel_reasoning_effortをminimal、low、medium、high、xhighと記述する。
- Codex Subagents文書はUltraをeligible account/model向けのmaximum reasoning plus proactive delegationと説明する。
- OpenAI Agents SDKはhandoffとagents-as-toolsを別orchestration patternとして説明する。

重要:

provider API、Codex product intelligence level、local CLI enum、CodeFire project policyは別namespaceである。同じ文字列が同じbehaviorを保証しない。

## 2. Project-Level Tier Semantics

### Medium

目的:

- bounded、deterministic、strong oracleのworkを低いcoordination costで完了する。

Candidate tasks:

- exact validation and summary
- deterministic classification
- source-backed table/link sync
- prepared smoke/dogfood command
- schema-preserving mechanical transform

Prohibited decisions:

- object identity/format
- security/remote policy
- architecture boundary
- canonical doc family deletion
- release completion
- conflicting authority adjudication

### High

目的:

- objectiveは明確だがimplementation/synthesisに判断が必要なwork。

Candidate tasks:

- focused implementation
- root-cause analysis
- test design
- bounded doc consolidation
- performance investigation
- routine research/dogfood

Required:

- typed task packet
- focused then wide validation
- intentionally untouched files
- independent review for product/canonical policy

### Ultra

目的:

- plausible but wrongな判断が複数lane、user data、security、release claimを壊すwork。

Candidate tasks:

- cross-subsystem architecture
- object identity/seal/branch atomicity
- migration/recovery/security/auth/signature
- large canonical deletion/consolidation
- multi-agent global planning
- conflicting reviewers/sources
- final completion audit

Ultraは一agentが長く考えるだけではない。

Candidate execution shape:

1. Ultra manager/architect: question、alternatives、invariants、falsification
2. High implementer: bounded patch
3. Ultra reviewer: independent adversarial review
4. Medium validator: exact deterministic gates
5. Ultra manager: requirement-by-requirement completion audit

## 3. Runtime Adapter

Current string mapping is insufficient for Ultra.

Candidate semantic adapter:

~~~json
{
  "projectTier": "ultra",
  "intent": {
    "reasoning": "maximum-supported",
    "delegation": "proactive-when-independent",
    "managerOwnsFinal": true,
    "independentReview": true
  },
  "runtime": {
    "surface": "openai-responses-api",
    "model": "gpt-5.6-sol",
    "reasoningEffort": "max",
    "delegationImplementation": "external-orchestrator"
  }
}
~~~

| Surface | Medium | High | Ultra intent |
|---|---|---|---|
| local Codex 0.144.1 | medium | high | ultra |
| GPT-5.6 Sol Responses API | medium | high | max plus explicit proactive orchestrator |
| portable Codex without ultra | medium | high | xhigh plus explicit delegation, recorded fallback |
| runtime lacking model | stop | stop or explicit substitution | stop; no silent downgrade |

reasoning=maxだけをUltra semantic equivalentと呼ばない。

DC-AI-050:

role-routing-matrixのadapter valueをstringからeffective reasoning、delegation、review、fallbackを持つobjectへ変える。

## 4. Capability Probe

Before run:

1. installed CLI/runtime versionを記録する。
2. model catalog/capabilitiesをqueryする。
3. requested modelとeffortを確認する。
4. project configがload可能か確認する。
5. multi-agent capabilityとthread/depth limitを確認する。
6. sandbox、approval、network、MCP availabilityを記録する。
7. effective routeをhandoff/evidenceへ残す。

Evidence nuance:

- codex --strict-config with a version-only commandは、unknown overrideでもversion表示へ到達しうるためenum validationの十分なproofではない。
- existing configがstrict doctorでloadされた事実と、bundled catalogのsupported levelを組み合わせる。
- actual sampling/runのeffective model/effort eventが得られるならそれを最も強いruntime evidenceとする。

Failure policy:

- requested model unavailable: fail closed by default。
- Ultra unavailable: automatic High downgradeをしない。
- explicitly allowed fallback: effective effortとlost semanticsを記録する。
- delegation unavailable: max/xhigh reasoningだけをUltraと表示しない。

## 5. Risk Routing

Candidate score dimensions:

| Dimension | 0 | 1 | 2 |
|---|---|---|---|
| blast radius | one isolated artifact | one module/family | cross-system/external contract |
| reversibility | trivial | cleanup needed | format/data/remote risk |
| ambiguity | deterministic | alternatives | success/problem uncertain |
| invariant density | none | one known | identity/atomicity/security interaction |
| verification cost | exact cheap | integration | incomplete/expensive oracle |
| downstream multiplier | no dependent lane | one dependent | plan/review controls many lanes |

Candidate thresholds:

- 0..3: Medium
- 4..7: High
- 8..12: Ultra

Mandatory Ultra override:

- object identity or format
- seal/branch atomicity
- auth/signature/remote mutation
- migration/recovery/user data
- cross-subsystem architecture
- canonical family deletion
- release completion
- conflicting authority

HYP-AI-050:

hypothesis: risk-based routingはalways Ultraと同等のcritical qualityを保ち、trusted integrationあたりのlatency/tokenを減らす。

反証条件:

- Medium/Highからのescaped critical defectがalways Ultraより有意に高い。
- routing misclassification correction costがsavingを上回る。
- task score作成がtask executionより高costになる。

対応実験: EXP-AI-001。

## 6. Orchestration Patterns

### Handoff

specialistが次branchのownershipを引き継ぐ。

Use when:

- instructions、tools、policyが明確に異なる。
- specialistが次response/resultの責任者になる。

Risk:

- final ownershipが曖昧になる。
- history/context filteringでcritical constraintが落ちる。

### Agents As Tools

managerがfinal ownershipを維持し、bounded specialistを呼ぶ。

Use when:

- classification、research、test/log analysisなどbounded capability。
- one stable outer workflowが必要。

CodeFire candidate default:

- manager owns task packet、global files、integration、completion claim。
- specialists return typed Artifacts。

### Parallel Research

read-heavy、independent explorationに適する。raw transcriptではなくdistilled evidenceを返す。

### Parallel Writes

disjoint write set、separate worktree、serialized integrationが証明できる場合だけ使う。

## 7. Lane And Ownership Model

Candidate lane record:

~~~json
{
  "laneId": "lane-1",
  "taskPacketDigest": "sha256",
  "ownerRole": "implementer",
  "baseDigest": "sha256",
  "readSet": [],
  "writeSet": [],
  "globalFiles": [],
  "lease": {
    "createdAt": "time",
    "expiresAt": "time"
  },
  "status": "active"
}
~~~

Rules:

- one shared file has one writer per batch。
- manager owns global state/integration files。
- agents must not revert other lane/user changes。
- apply前にbase and write-set preconditionを再確認する。
- overlap discovered後は自動mergeせずmanagerへ戻す。
- reviewerはimplementation laneからindependentにする。
- validation runnerは原則read-only。

## 8. Optimistic Concurrency

Candidate preconditions:

~~~text
packet.baseDigest == current.allowedBase
and changedPaths subsetOf packet.writeSet
and no active lane owns overlapping path
and patch applies without hidden overwrite
~~~

Integration:

1. patch digest verify
2. base freshness check
3. changed path and ownership check
4. apply/merge in serialized queue
5. focused validation
6. review freshness check
7. wider gate

HYP-AI-051:

hypothesis: exploration/test/log analysisは4 laneまでparallel benefitがあり、overlapping write laneはcoordination costでnegative speedupになる。

反証条件:

- read-heavy taskでもstartup/context duplication costがdominant。
- disjoint file writeでもsemantic dependencyによりintegration defectが増える。

対応実験: EXP-AI-004。

## 9. Reviewer Independence

Independent means:

- implementation transcriptをblindly inheritしない。
- claimed designをsource/testと照合する。
- task/patch digestを確認する。
- strongest correctness and maintainability concernを必ず出す。
- skipped validationをriskとして扱う。
- approve理由をclaim/evidenceへ結ぶ。

Review profiles:

| Profile | Focus |
|---|---|
| correctness | objective、edge、state、algorithm |
| invariant/security | identity、atomicity、auth、path、remote |
| contract | CLI/JSON/schema/backward compatibility |
| maintainability | module boundary、complexity、docs/test |
| completion | requirement-by-requirement evidence |

Ultra reviewを全small docs editへ使わず、risk scoreとcanonicalityで選ぶ。

## 10. Cost Controls

- task decomposition前にparallelizableか判断する。
- same sourceを各agentへ重複投入しないcontext packを作る。
- Medium validator/classifierにdesign judgmentをさせない。
- High implementationをUltra managerがmicro-manageしない。
- retryは同じprompt反復ではなくfailure modelを更新する。
- two failed attemptsまたはconflicting evidenceでescalateする。
- costはtokenだけでなくreview/rework/integrationを含める。

MET-AI-020:

Cost To Trusted Integration =

~~~text
model cost
+ human review time
+ validation compute
+ coordination time
+ conflict/rework time
~~~

## 11. Failure Modes

| Failure | Mitigation candidate |
|---|---|
| Ultra everywhere | tier eval and budget |
| automatic silent downgrade | effective route evidence and fail closed |
| agent count as progress | trusted artifact and integration KPI |
| overlapping writes | typed write set and one writer |
| manager bottleneck | delegate bounded analysis, retain only global decisions |
| groupthink review | independent reviewer context |
| stale long task | base/packet digest and heartbeat/revalidation |
| subagent prompt injection | bounded context, tool policy, output validation |
| raw transcript pollution | typed distilled artifact |
| model availability drift | capability probe and dated binding |

## 12. Decision Candidates And Promotion

- DC-AI-050: Ultraをmaximum reasoning plus proactive delegation intentとして定義する。
- DC-AI-051: runtime adapterをcompound capability mappingにする。
- DC-AI-052: risk scoreとmandatory escalationでtierを選ぶ。
- DC-AI-053: manager-owned final、High implementation、Ultra independent review、Medium validationをdefault shapeにする。
- DC-AI-054: parallel writesへtyped write set、one-writer、serialized integrationを要求する。
- DC-AI-055: local/runtime capability probeとeffective route evidenceをrun前後に残す。

Promotion先:

- docs/development/ai/model-routing-policy.md
- docs/development/ai/role-routing-matrix.json
- .codex/config.toml and agent profiles
- second-agent/orchestrator wrapper
- routing eval suite

## 13. Primary References

- [GPT-5.6 Sol model](https://developers.openai.com/api/docs/models/gpt-5.6-sol)
- [Codex configuration reference](https://developers.openai.com/codex/config-reference)
- [Codex Subagents](https://developers.openai.com/codex/agent-configuration/subagents)
- [OpenAI Agents orchestration](https://developers.openai.com/api/docs/guides/agents/orchestration)
- [Codex MCP server with Agents SDK](https://developers.openai.com/codex/mcp-server)

## 14. Promotion Status

- Status: partly represented in current AI routing docs, but phase 2 decisions are not promoted
- Required evidence: EXP-AI-001、EXP-AI-004、EXP-AI-005
- Revisit when: historical task corpusでtier and parallelism comparisonが完了したとき

