# Integration, Adoption, And Standardization

> **非正本 / exploration only**
>
> これはIDE、CLI、CI、MCP、Git bridge、migration、developer experience、open standard化を検討する研究ノートである。採用された結論はstrategy、roadmap、ADR、schema、release/distribution docs、integration packagesへpromotionする。

## 1. Goal

de factoになるには、優れたcore semanticsだけでなく次が必要である。

1. existing workflowへ10分単位で入り込める。
2. first useful diagnosticがmetadata整備より先に得られる。
3. advisoryからblockingへ安全に強化できる。
4. agent/IDE/CI/providerを変えてもcontractが残る。
5. data、evidence、certificateをexportできる。
6. independent implementationがconformanceを検証できる。
7. breaking evolutionとextension governanceが予測可能である。

## 2. Integration Architecture

~~~text
Canonical kernel
  JSON schemas
  CLI JSON envelope
  capabilities
  Atom/Trace/Fire/Verification/Evidence/Certificate

Thin local adapters
  MCP server
  LSP server
  SARIF renderer
  Git snapshot/PR bridge
  CI action/wrapper

External protocol adapters
  A2A task/artifact
  CloudEvents/CDEvents
  OpenTelemetry
  in-toto/SLSA/SPDX
~~~

Rule:

- adapterはcore domain logicをduplicateしない。
- same internal diagnosticをCLI、LSP、SARIF、MCPへrenderする。
- same evidence/certificate subjectをnative、PR、CIで別claimとして表す。
- adapter failureがrepository stateを破損しない。

## 3. CLI As Compatibility Kernel

現行CLI JSONを最小fallbackとする利点:

- shell/CI/IDE/providerを問わない。
- process isolationが明確。
- existing command and testsを再利用できる。
- MCP/LSPが壊れてもdiagnosticを取得できる。

Required evolution:

- formal JSON Schemas
- stable pagination/cancellation/progress
- capabilities discovery
- bounded diagnostics/next_actions
- subject/base/path identity
- machine-readable operation plan
- structured version/compatibility error
- deterministic output ordering

Q-ADOPT-001:

long-running serverを追加する前にCLI latencyとstartup costをどこまで改善できるか。

LSP/MCP serverはperformanceとsession discoveryのために必要になりうるが、CLI contractを迂回して別semanticsを作らない。

## 4. MCP Integration

Phase candidate:

### MCP-0: Schema And Discovery

- capabilities
- schema/resources
- no repository data

### MCP-1: Read-Only Context

- status、context、explain、show、diff
- Roots enforcement
- bounded result/pagination

### MCP-2: Read-Only Analysis

- scan dry-run、verify、review-pack
- progress/cancellation
- sensitive output redaction

### MCP-3: Evidence Mutation

- evidence add
- explicit approval
- operation plan and subject preview

### MCP-4: Repository Mutation

- fire/extinguish/commit/apply
- separate capability profile
- strong auth/approval/idempotency
- security/adoption evidenceが揃うまでdefer candidate

MCP success criteria:

- two independent clientsでsame schema/semantics。
- prompt injection fixtureでunauthorized mutation 0。
- CLI JSONとMCP resultのsemantic conformance。
- read-only profileでpersistent write 0。

## 5. IDE And LSP

Candidate user journey:

1. editor opens managed/advisory workspace。
2. background indexはcached/incrementalに更新。
3. changed AtomへFire count/code lensを表示。
4. hoverでsource/target/reason/policyを説明。
5. code actionはcontext、verify、resolution templateを提示。
6. mutation前にplanとrequired evidenceを表示。
7. save/CI時にwider verify。

UX principles:

- diagnostic messageだけでwhy、subject、next safe actionが分かる。
- blocking/non-blocking、fresh/staleを視覚的に分ける。
- generated IDsやinternal object pathをuserへ過剰露出しない。
- graph全体を常時描画せず、change-centered neighborhoodを優先する。
- scan latency中もprogress/cancellationを提供する。
- LSP server crashでworkspace editを失わない。

Provisional latency hypotheses:

- cached status P95 under 200 ms
- changed-file context P95 under 1 s
- incremental scan P95 under 2 s for pilot repository
- full verifyはasync/progress

これらはtarget決定ではなくEXP-AI-016でbaselineを取る。

## 6. CI And SARIF

Generic CI flow:

~~~text
checkout
  -> CodeFire capability/version check
  -> snapshot/base/head binding
  -> scan/verify
  -> evidence capture
  -> SARIF and human summary
  -> optional in-toto attestation
  -> PR/check/artifact publish
~~~

Modes:

- observe: metrics only
- advisory: finding visible、non-blocking
- selected blocking: critical policy/pathだけblock
- full blocking: all required Fire/evidence
- native seal: CodeFire certificate required

CI security:

- repository-controlled codeとprovider API keyを同じuntrusted environmentへ広く置かない。
- least privilege token。
- action/wrapper versionをpin。
- workflow digestとrunner identityをevidenceへ入れる。
- fork PRとtrusted branchでpolicyを分ける。
- artifact upload前にsecret redaction。

SARIFはdiagnostic exportであり、CodeFire state mutationやcertificateを表す正本ではない。

## 7. Git Bridge Adoption Hypothesis

### Subject Identity

Candidate subject:

~~~text
repository remote identity
+ base commit SHA
+ head commit SHA
+ normalized tree/patch digest
+ CodeFire policy/schema version
~~~

Git SHAだけでworking tree、submodule、LFS/external artifact、generated evidenceを全て表せるとは限らない。coverageを明示する。

### Bridge Modes

| Mode | Repository mutation | Claim | Initial use |
|---|---|---|---|
| snapshot | none | assessment of one tree | local trial |
| PR diff | none | assessment of base/head change | advisory CI |
| attested gate | evidence/CI artifact only | verified checks for subject | selected blocking |
| import/native | CodeFire objects/branch | native consistency seal | high-assurance migration |

### Boundary

- coreはGit command semanticsを実装しない。
- bridgeはGit tree/diffをCodeFire Artifact/change inputへ変換する。
- Git commitをCodeFire sealed commitと呼ばない。
- bridge failureをnative repository corruptionと混同しない。
- bridge packageは独立version/owner/testを持つ。

HYP-AI-070:

hypothesis: read-only Git/PR bridgeはnative-only onboardingよりfirst-valueとpilot retentionを改善し、core invariant defectを増やさない。

反証条件:

- meaningful Trace/Fireを得るために結局full CodeFire metadata/migrationが必要。
- assessment/native claimの混同が多数発生。
- bridge compatibility maintenanceがcore feature deliveryを恒常的に遅らせる。
- Git diff semanticsとAtom change semanticsの差でfalse diagnosticが多発。

対応実験: EXP-AI-009、EXP-AI-012。

## 8. Adoption Ladder

### Stage 0: Inspect

- install and capabilities
- read-only repository/snapshot assessment
- no policy blocking
- no backfill required

Exit:

- first meaningful findingまたはclean evidenceを10分以内に得られるか。

### Stage 1: Advisory

- one CI job
- SARIF/PR summary
- selected artifact classes
- Fire quality feedback

Exit:

- findingsがhuman reviewへvalueを足す。

### Stage 2: Trace One Critical Flow

- requirement -> design/API -> code -> test/ops
- explicit Atom/links
- critical path policy

Exit:

- actual impact missまたはreview time reductionを観測。

### Stage 3: Selected Blocking

- security/API/migration/release-critical areas
- required evidence trust level
- exception workflow

Exit:

- bypass/false block rateが許容範囲。

### Stage 4: Broad Blocking

- wider project policy
- IDE context/actions
- attested CI
- schema/version governance

### Stage 5: Native Seal

- project/branchをCodeFire native repositoryへ移行。
- consistent-only formal history。
- remote distribution and certificate。

全userがStage 5へ進む必要はない。各stage単独のvalueとexport pathを持つ。

## 9. Migration

Principles:

- full history backfillをdefault requirementにしない。
- current trusted baselineから開始可能にする。
- unknown/untraced areaをcleanとみなさずcoverage unknownと表示する。
- generated Atom IDからexplicit IDへのmigrationをversionedにする。
- policy強化前にimpact preview/dry-runを行う。
- rollbackはCodeFire metadata/advisory integrationを外せる。
- native sealへの移行はsource/subject mappingとcertificate semanticsをADR化する。

Candidate migration phases:

1. inventory artifacts and formats
2. generate candidate atoms/links
3. human review of critical graph
4. baseline assessment with coverage gaps
5. advisory run period
6. selected policy enablement
7. optional native import/seal

## 10. Developer Experience

First-run questions:

- 何をinstallするか。
- repositoryへ何を書き込むか。
- Git/CIに影響するか。
- findingはなぜ発生したか。
- false positiveをどう報告/解消するか。
- dataはどこへ送られるか。
- uninstall/export/rollbackできるか。

DX requirements candidate:

- one command health/capability check
- read-only default trial
- explicit mutation preview
- exact next actions
- copied open directory/path confusionの明確なdiagnostic
- supported artifact/Atom coverage report
- progressive configuration
- sample policyではなくproject-specific generated proposal
- all data export

MET-AI-030:

Time To First Trusted Signal =

install開始から、userがmeaningfulと確認したdiagnosticまたはverified-clean assessmentを得るまで。

## 11. Standardization Strategy

### Step 1: Publish Schemas And Semantics

- stable schema IDs
- media types/URIs
- valid/invalid examples
- compatibility policy
- security considerations

### Step 2: Conformance Kit

- golden vectors
- command/result fixtures
- adapter mapping fixtures
- property/invariant tests
- version/downgrade cases
- malicious inputs

### Step 3: Reference Implementations

- Rust CLI
- read-only MCP
- SARIF renderer
- generic CI wrapper
- in-toto exporter

### Step 4: Extension Registry

- Atom extractor kinds
- diagnostic rule IDs
- evidence predicate types
- adapter profiles
- vendor namespaces

### Step 5: Open Governance

- specification repository/process
- proposal/RFC or SEP
- maintainers and decision rights
- release cadence
- deprecation window
- security disclosure
- independent implementation feedback

### Step 6: External Standard Contribution

CodeFire固有fieldをすぐ標準提案しない。複数implementation/use caseで一般性が示された概念だけをMCP/A2A/CDEvents/OTel/in-toto ecosystemへ提案する。

## 12. Compatibility Policy Candidate

| Change | Compatibility |
|---|---|
| optional field addition | backward compatible if ignored safely |
| enum addition | consumer-dependent; capability/version required |
| required field addition | breaking major |
| field meaning change | breaking major |
| diagnostic wording | non-contract |
| rule ID meaning change | breaking or new ID |
| new adapter capability | negotiated |
| security default relaxation | never silent |

Monotonic trust:

unknown schema/field/evidenceでDENYからALLOWへ変えない。

## 13. Governance Risks

| Risk | Mitigation candidate |
|---|---|
| spec and implementation drift | generated schemas/tests and release gate |
| single-vendor standard | independent conformance and governance |
| extension fragmentation | namespace/registry and promotion criteria |
| too-early standardization | prototype adapters before freeze |
| backward compatibility paralysis | explicit major versions/deprecation |
| marketing claim ahead of evidence | public KPI and limitation |
| adapter maintenance explosion | thin adapters and support tiers |
| security default divergence | mandatory conformance cases |

## 14. De Facto Flywheel Hypothesis

~~~text
easy advisory adoption
  -> more real Trace/Fire/evidence cases
  -> better eval corpus and policy precision
  -> fewer false positives and stronger certificates
  -> more CI/IDE/agent adapters
  -> more independent consumers
  -> stronger schema/conformance standard
  -> lower adoption risk
~~~

HYP-AI-071:

hypothesis: open schemas and reference adapters create a stronger durable flywheel than proprietary orchestration features。

反証条件:

- independent consumer/adapterが増えない。
- schemasがCodeFire implementation detailに過度に依存する。
- governance costがcore adoptionを上回る。

## 15. Decision Candidates And Promotion

- DC-AI-070: CLI JSONをcompatibility kernel、MCP/LSP/CIをthin adapterとする。
- DC-AI-071: adoptionをInspect -> Advisory -> Critical Trace -> Selected Blocking -> Native Sealで段階化する。
- DC-AI-072: Git bridgeをread-only snapshot/PRから実験し、transparent compatibilityを作らない。
- DC-AI-073: public JSON Schemas、conformance kit、reference adaptersをstandardizationの最小核にする。
- DC-AI-074: unknown coverageをcleanと表示しない。
- DC-AI-075: first wedgeをread-only MCPまたはSARIF/CI advisoryで比較する。

Promotion先:

- strategy and roadmap
- Git boundary ADR
- integration crates/packages
- schemas and conformance CI
- installer/runbook
- governance document

## 16. Primary References

- [Codex MCP](https://developers.openai.com/codex/extend/mcp)
- [Codex GitHub Action](https://developers.openai.com/codex/github-action)
- [Codex non-interactive mode](https://developers.openai.com/codex/non-interactive-mode)
- [MCP specification](https://modelcontextprotocol.io/specification/2025-06-18)
- [LSP specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/)
- [SARIF 2.1.0](https://www.oasis-open.org/standard/sarif-v2-1-0/)
- [CDEvents](https://cdevents.dev/docs/)

## 17. Promotion Status

- Status: not promoted
- Required evidence: EXP-AI-007、EXP-AI-009、EXP-AI-010、EXP-AI-012、EXP-AI-016
- Revisit when: one read-only adapterとone advisory CI pilotでfirst-value、precision、security、maintenance costを測定したとき

