# Experiment Catalog And KPI Scorecard

> **非正本 / exploration only**
>
> これはPhase 2仮説を反証するためのexperimentとmetric候補である。実行結果の正本ではない。採用されたexperimentはtest/eval/benchmark suiteへ、KPIはproject management/strategyへpromotionする。

## 1. Experiment Rules

各experimentは次を固定する。

- experiment ID and hypothesis
- immutable base revision or fixture digest
- Task/Context Packet version
- runtime/model/effective effort
- sandbox、approval、network、tool capabilities
- trial count and randomization/order
- deterministic assertions
- grader version
- command/log/evidence location
- abort and safety condition
- result、confidence、limitations

比較不能なrunを混ぜない。provider/model availabilityが変わった場合は新cohortとする。

## 2. Shared Fixture Matrix

| Class | Candidate fixture |
|---|---|
| deterministic docs | stale link/current-state contradiction |
| focused implementation | historical bounded CFB fix |
| architecture | module boundary/object format proposal |
| invariant bug | sealed commit/branch/path/JSON seeded bug |
| review | patch set with hidden correctness/security defects |
| dirty tree | mixed product/docs/dogfood changes |
| context | command/source/test mapping task |
| agent security | repository/MCP prompt injection |
| evidence | forged/missing/stale validation artifact |
| adoption | small Python、TypeScript、Rust Git repository |
| scale | large files/Atoms/Trace/branch history |
| recovery | corrupt object/pending transaction/stale lock |

Historical casesはfix commitやissue answerをmodel contextへ直接入れず、hidden evaluatorが保持する。

## 3. EXP-AI-001: Model Tier Routing

Hypothesis:

risk-based Medium/High/Ultra routingはalways Ultraと同等のcritical qualityを保ち、cost to trusted integrationを下げる。

Design:

- task classesごとに最低10 case。
- same Task/Context PacketをMedium、High、Ultraでrun。
- always Ultra、risk-routed、Medium-first-escalateを比較。
- implementer/reviewerを同一にせず、review laneもtier別比較。

Primary metrics:

- trusted integration success
- escaped hidden defect
- reviewer correction
- false completion
- wall time/token/tool calls
- rework and integration time

Falsification:

- routed cohortでcritical escapeがalways Ultraより増える。
- routing overhead plus reworkがcost savingを超える。

Decision gate:

- task class別にcheapest non-inferior tierを選ぶ。
- sample不足ならrouting policyを固定しない。

## 4. EXP-AI-002: Context Pack Ablation

Hypothesis:

graph-derived bounded contextはfull repo readingとstatic packより少ないtokenでcritical source recallを維持する。

Variants:

1. full repo orientation
2. current static context pack
3. changed-files only
4. graph-derived pack with authority/freshness/omission

Metrics:

- critical source recall/precision
- time to first correct edit
- redundant reads
- token estimate/actual
- context clarification
- missed invariant/test
- prompt cache hit

Falsification:

- graph-derivedでcritical missまたはreview reworkが増える。
- generation latencyがsavingを上回る。

## 5. EXP-AI-003: Typed Packet Versus Free Prompt

Hypothesis:

typed Task/Review Packetはscope violation、clarification、completion overclaim、review driftを減らす。

Variants:

- short natural-language prompt
- current Markdown packet
- typed packet without digest
- typed packet plus base/context/patch/review digest

Metrics:

- packet authoring time
- clarification count
- out-of-scope paths
- acceptance pass
- evidence completeness
- stale review detection
- total cycle time

Falsification:

- typed packet maintenanceがsaved review/reworkより高い。
- agentsがschemaを読んでもbehaviorが改善しない。

## 6. EXP-AI-004: Parallelism Boundary

Hypothesis:

read-heavy/disjoint workはparallel benefitがあり、overlapping writesはnegative speedupとdefectを生む。

Variants:

- sequential
- 2 and 4 read-only lanes
- 2 and 4 disjoint write lanes
- intentionally overlapping write lanes

Metrics:

- wall-clock speedup
- parallel efficiency
- duplicated context/tool cost
- conflict count
- semantic integration defect
- user/other lane overwrite attempt
- manager integration time

Safety:

- isolated worktrees/fixtures。
- no remote mutation。
- user dirty treeを対象にしない。

Decision:

- task class別max_threads and one-writer ruleへ反映。

## 7. EXP-AI-005: Independent Review

Hypothesis:

fresh Ultra/High reviewerはself-reviewよりseeded defect recallとcompletion calibrationを改善する。

Variants:

- implementer self-review
- fresh High reviewer
- fresh Ultra reviewer
- two specialized reviewers plus manager synthesis

Seed classes:

- objective miss
- invariant break
- JSON/schema regression
- missing test
- path/security issue
- maintainability/module boundary
- review target drift

Metrics:

- recall/precision/F1
- strongest issue rank
- false block
- review time/cost
- defects remaining after correction

Falsification:

- independent/Ultraでrecallが改善せずfalse positive/costだけ増える。

## 8. EXP-AI-006: Evidence Trust And Forgery

Hypothesis:

subject-bound captured/attested evidenceはagent self-reportよりforged、stale、wrong-base evidenceを検出する。

Cases:

- fake test pass text
- output from different base
- missing artifact
- changed patch after test
- expired URI
- compromised/untrusted producer
- redacted secret

Variants:

- asserted handoff
- local captured evidence
- environment-bound evidence
- CI-attested
- signed in-toto

Metrics:

- forged evidence acceptance
- valid evidence rejection
- verification latency
- metadata/storage overhead
- redaction leak

Guardrail:

- unauthorized evidence acceptance 0 for release/security profile。

## 9. EXP-AI-007: MCP Conformance And Red Team

Hypothesis:

read-only MCP profileはCLI semanticsを保ち、two clientsから安全にcontext/verifyを利用できる。

Tests:

- tools/resources/capabilities discovery
- input/output schema
- Roots/path escape
- cancellation/timeout/pagination
- repository prompt injection
- malicious tool description/output
- sensitive URL/data exfiltration
- mutation attempt

Metrics:

- CLI/MCP semantic equivalence
- tool selection accuracy
- unauthorized mutation/data disclosure
- approval count
- latency/output size
- independent client pass

Failure gate:

- read-only profileでpersistent writeまたはoutside-root readが一件でも起きる。

## 10. EXP-AI-008: Trace And Attestation Join

Hypothesis:

OTel trace、CodeFire evidence/certificate、in-toto/SLSA provenanceをsubject digestで結ぶと、workflow debugとartifact auditを両立できる。

Design:

- one Task Packetからpatch、test、review、seal、buildを実行。
- OTel trace IDsをcapture。
- CodeFire evidence/certificateを生成。
- in-toto statementとSLSA provenanceへlink。
- one missing/corrupt linkをfault inject。

Metrics:

- end-to-end traversal success
- semantic duplication
- broken link detection
- storage/latency
- sensitive field count/redaction

Falsification:

- identity mappingがambiguous。
- duplicate metadata driftが多い。
- standard exportでCodeFire semanticsが失われる。

## 11. EXP-AI-009: Adoption Funnel

Hypothesis:

read-only/advisory onboardingはnative-firstよりtime-to-first-valueとpilot retentionを改善する。

Cohorts:

- native-only quickstart
- snapshot read-only
- PR/SARIF advisory
- critical-flow trace assisted setup

Repositories:

- small Python
- medium TypeScript
- medium Rust
- docs/API-heavy

Metrics:

- install success
- first trusted signal time
- meaningful finding rate
- setup changes/files
- user explanation accuracy
- day/week retention
- transition to selected blocking

Falsification:

- advisoryではmeaningful signalが得られない。
- setup metadata taxがnativeと同等。
- userがclaim scopeを理解できない。

## 12. EXP-AI-010: Schema And Compatibility

Hypothesis:

versioned JSON Schema、capability negotiation、golden vectorsでmulti-version consumer breakageを事前検出できる。

Cases:

- optional field addition
- enum addition
- required field addition
- field meaning change
- unknown extension
- unsupported capability/downgrade
- malformed/oversized payload

Implementations:

- current Rust producer/consumer
- minimal independent script/client
- one older fixture version

Metrics:

- expected accept/reject accuracy
- silent semantic downgrade
- migration effort
- schema/code drift

Guardrail:

- unknown/downgradeがDENYをALLOWへ変えない。

## 13. EXP-AI-011: Standards Export Round Trip

Hypothesis:

SARIF、CDEvents、OTel、in-toto/SLSA exportsは独自consumerより低いintegration costでCodeFire semanticsを有用に運べる。

Mappings:

- diagnostic -> SARIF
- lifecycle occurrence -> CloudEvents/CDEvents
- operation/run -> OTel
- evidence/certificate -> in-toto
- build relation -> SLSA

Metrics:

- schema conformance
- required semantic preservation
- consumer rendering/usefulness
- custom extension count
- round-trip identity
- adapter LOC/maintenance

Falsification:

- critical semanticsの大半がopaque customDataへ落ちる。
- consumerがextensionを使えずvalueがない。

## 14. EXP-AI-012: Git Bridge Modes

Hypothesis:

read-only snapshot/PR bridgeはcore Git compatibilityなしにadoption valueを出せる。

Variants:

- snapshot assessment
- base/head PR assessment
- attested selected-blocking gate
- native import/seal

Cases:

- rename/copy
- submodule/external artifact
- generated file
- untracked working tree
- large binary
- merge commit
- missing base

Metrics:

- Atom/Fire accuracy
- subject identity ambiguity
- false stale
- setup/maintenance cost
- user claim comprehension
- core code coupling

Falsification:

- meaningful assessmentにGit-specific core changesが大量に必要。
- native/assessment certificate confusionが解消できない。

## 15. EXP-AI-013: Prompt Cache And Compaction

Hypothesis:

stable prefix plus typed variable packetはcache hitを改善し、explicit durable packetはcompaction後のconstraint lossを減らす。

Variants:

- ad hoc full prompt
- stable AGENTS/protocol prefix
- stable prefix plus typed packet
- long-running with/without explicit durable state

Metrics:

- cache read/write tokens
- latency/cost
- constraint recall
- stale assumption
- prompt size
- quality/acceptance

Falsification:

- cache optimizationでprompt clarity/qualityが下がる。
- packet redundancyがsavingを相殺する。

## 16. EXP-AI-014: Behavioral Reproducibility

Hypothesis:

same inputでbyte-identical outputを求めず、acceptance/evidence equivalenceを測る方がagent regression detectionに適する。

Design:

- same base/packet/profileを5回以上run。
- patch text、tool sequence、acceptance、evidence、review outcomeを比較。

Metrics:

- exact patch variance
- behavioral pass variance
- evidence completeness variance
- tool/action variance
- reviewer disagreement

Decision:

- stable propertyをrelease gate、unstable detailをdiagnosticに分類。

## 17. EXP-AI-015: Fire Fatigue And Policy Precision

Hypothesis:

severity、propagation depth、grouping、batch resolutionを調整すると、impact missを増やさずFire fatigueを減らせる。

Variants:

- current policy
- shallow propagation
- severity-selected blocking
- grouped Fire
- template/batch resolution

Metrics:

- valid Fire precision
- critical impact recall
- invalid-fire rate
- resolution time
- copied rationale similarity
- bypass/override
- reopened stale resolution

Falsification:

- fatigue改善と引き換えにcritical missが増える。

## 18. EXP-AI-016: IDE And SARIF UX

Hypothesis:

LSP contextual diagnostics and SARIF PR annotationはCLI-onlyよりFire理解とcorrect resolutionを速める。

Variants:

- CLI text
- CLI JSON wrapper
- SARIF PR annotation
- LSP diagnostics/hover/action

Metrics:

- diagnosis time
- correct next action
- incorrect resolution
- interaction count
- latency
- false-positive dismissal
- user confidence versus actual correctness

Falsification:

- convenienceは上がるがincorrect resolutionも増える。
- server latency/noiseでfeatureをdisableする。

## 19. KPI Hierarchy

### North Star

MET-AI-001 Trusted Integration Throughput

Definition:

~~~text
count(change packages)
where task packet is bound
and required deterministic gates pass
and review target matches integrated patch
and required evidence is complete/fresh
and no blocking Fire remains
~~~

Report per engineer-week、team-week、agent compute cohort。raw numberだけでseverity/task classを混ぜない。

### Quality

| ID | Metric | Definition |
|---|---|---|
| MET-AI-040 | escaped consistency defect | integration後に見つかったspec/design/code/test/ops mismatch |
| MET-AI-041 | critical invariant escape | identity/atomicity/security/user-data defect |
| MET-AI-042 | false completion claim | complete claim後にrequired evidence/gateが不足 |
| MET-AI-043 | reviewer correction rate | reviewerがmaterial changeを要求したtask比率 |
| MET-AI-044 | seeded defect recall | hidden defects detected / hidden defects |
| MET-AI-045 | Fire precision | human-confirmed valid obligations / generated Fires |
| MET-AI-046 | critical impact recall | known critical impacts detected / expected |
| MET-AI-047 | evidence completeness | required evidence resolved / required evidence |

### Speed

| ID | Metric |
|---|---|
| MET-AI-050 | issue to packet-ready |
| MET-AI-051 | packet to first correct patch |
| MET-AI-052 | patch to review decision |
| MET-AI-053 | total trusted integration cycle |
| MET-AI-054 | manager integration time |
| MET-AI-055 | validation queue time |
| MET-AI-056 | time to first trusted signal |

### Cost

| ID | Metric |
|---|---|
| MET-AI-060 | model tokens/cost per trusted integration |
| MET-AI-061 | tool calls per trusted integration |
| MET-AI-062 | context bytes/tokens read |
| MET-AI-063 | prompt cache hit |
| MET-AI-064 | retry/rework cost |
| MET-AI-065 | reviewer/human minutes |
| MET-AI-066 | Ultra share and marginal gain |

### Reliability And Security

| ID | Metric | Candidate guardrail |
|---|---|---|
| MET-AI-070 | lost user work | zero |
| MET-AI-071 | unauthorized mutation | zero |
| MET-AI-072 | outside-root access | zero |
| MET-AI-073 | forged evidence accepted | zero for release/security |
| MET-AI-074 | secret/redaction leak | zero |
| MET-AI-075 | silent model/tier downgrade | zero |
| MET-AI-076 | schema security downgrade | zero |

### Adoption And Ecosystem

| ID | Metric |
|---|---|
| MET-AI-080 | install/capability success |
| MET-AI-081 | first meaningful finding |
| MET-AI-082 | advisory to selected-blocking conversion |
| MET-AI-083 | week 4/12 active repository retention |
| MET-AI-084 | independent client/adapter count |
| MET-AI-085 | conformance profile pass |
| MET-AI-086 | schema migration success |
| MET-AI-087 | export consumer success |

## 20. Provisional Targets

次はbaseline取得前のdecision thresholdsであり、current achievementではない。

- lost user work、unauthorized mutation、outside-root access、secret leak: 0
- required evidence completeness for integration-ready: 100 percent
- unsupported completion claim for release audit: 0
- packet creation median: under 10 minutes for prepared issue class
- patch to review packet median: under 5 minutes after gates for bounded task
- first trusted signal median: under 10 minutes for supported pilot repository
- schema/conformance golden suite: 100 percent
- critical seeded defect recall: task class別baseline後にthreshold決定
- Medium/High/Ultra: cheapest non-inferior routeを採用

latency、Fire precision、retentionはbaselineなしに固定しない。

## 21. Anti-Metrics

単独ではprogress/qualityとみなさない。

- generated LOC
- commits or patches per day
- spawned agent count
- reasoning tierの高さ
- tool-call volume
- closed Fire count
- document page count
- passしたmodel grader数
- context window utilization

## 22. Evidence Record Template

~~~yaml
experiment_id: EXP-AI-NNN
run_id: unique
date: YYYY-MM-DD
base_revision: immutable
fixture_digest: sha256
task_packet_digest: sha256
context_pack_digest: sha256
runtime:
  surface:
  model:
  requested_tier:
  effective_effort:
  delegation:
  cli_version:
environment:
  sandbox:
  approval:
  network:
result:
  deterministic_assertions:
  grader_scores:
  metrics:
  failures:
evidence:
  command_records:
  patch_digest:
  review_digest:
  trace_id:
limitations:
decision:
  supported:
  falsified:
  inconclusive:
~~~

## 23. Priority

P0:

1. EXP-AI-003 typed packet
2. EXP-AI-001 model routing
3. EXP-AI-005 reviewer
4. EXP-AI-006 evidence trust
5. EXP-AI-010 schema compatibility

P1:

1. EXP-AI-002 context ablation
2. EXP-AI-004 parallelism
3. EXP-AI-007 MCP
4. EXP-AI-009 adoption
5. EXP-AI-012 Git bridge
6. EXP-AI-015 Fire fatigue

P2:

1. EXP-AI-008 trace/attestation join
2. EXP-AI-011 standards export
3. EXP-AI-013 cache/compaction
4. EXP-AI-014 reproducibility
5. EXP-AI-016 IDE UX

## 24. Promotion Rule

experiment/KPIを正本へ昇格する条件:

1. fixtureとcommandがrepository内で再現可能。
2. metric definitionに分母、除外、task classがある。
3. negative/inconclusive resultも保存する。
4. provider/runtime versionを記録する。
5. thresholdはbaselineとrisk toleranceから決める。
6. owner、run cadence、failure actionを定める。
7. vanity metric化した場合の削除条件を持つ。

## 25. Promotion Status

- Status: not promoted
- Required next action: P0 fixtureを選び、最低各5 caseのpilot baselineを取る
- Promotion destinations: tests/evals、benchmark、CI、docs/development/ai/、project management KPI
