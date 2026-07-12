# Agent And Tool Protocol Stack

> **非正本 / exploration only**
>
> これはagent、tool、IDE、CI、telemetry、attestationの相互運用層を検討する研究ノートである。採用された結論はschema、docs/automation_interface.md、adapter architecture、ADR、security policy、conformance suiteへpromotionする。

## 1. Scope

対象:

- MCP、A2A、LSP、SARIF、CloudEvents/CDEvents、OpenTelemetry
- in-toto、SLSA、JSON Schema、SPDX
- CodeFire CLI JSON、capabilities、resource/tool surface
- versioning、capability negotiation、security、conformance

対象外:

- 各標準の完全実装
- vendor-specific marketplace
- agent schedulerのproduct内実装
- remote protocolを直ちに置換する判断

## 2. Separation Of Responsibilities

| Standard/surface | Solves | Does not solve | CodeFire candidate use |
|---|---|---|---|
| CLI JSON | local command invocation and exact result | long-lived discovery/session | stable kernel and fallback |
| JSON Schema | payload shape and validation | transport、auth、semantics proof | canonical packet/envelope schemas |
| MCP | model-facing context/resources/tools | agent-to-agent task lifecycle | read-only context and bounded operations |
| A2A | remote agent discovery、Task、Message、Artifact lifecycle | IDE diagnostics、tool execution details | optional task/handoff adapter |
| LSP | editor/server diagnostics、navigation、actions | CI artifact or agent orchestration | IDE integration |
| SARIF | static-analysis result interchange | interactive action lifecycle | Fire/verification export |
| CloudEvents | portable event envelope | CI/CD vocabulary | transport-neutral event wrapper |
| CDEvents | CI/CD event vocabulary | detailed trace spans | scan/verify/seal lifecycle events |
| OpenTelemetry | traces、metrics、logs、context propagation | artifact attestation | agent/tool/validation observability |
| in-toto | subject-bound attestation statement/envelope | build level policy by itself | CodeFire evidence/certificate export |
| SLSA | verifiable build/source provenance model | semantic requirement consistency | link build provenance to sealed subject |
| SPDX | SBOM、build、AI/dataset/provenance metadata | interactive workflow | external supply-chain relation |

decision-candidate: CodeFireはこれらを一つの独自super-protocolへ再実装しない。CodeFire固有semanticsを小さなcanonical schemaとして定義し、各標準へadapterで写像する。

## 3. Protocol Kernel

candidate minimal kernel:

1. versioned JSON Schemas
2. stable IDs and URIs
3. capabilities document
4. read-only query semantics
5. dry-run plan semantics
6. mutation approval and idempotency semantics
7. evidence/certificate subject binding
8. error taxonomy and pagination
9. conformance vectors

Candidate URI forms:

~~~text
codefire://repository/{repo-id}
codefire://repository/{repo-id}/atom/{atom-id}
codefire://repository/{repo-id}/fire/{fire-uid}
codefire://repository/{repo-id}/commit/{object-id}
codefire://repository/{repo-id}/evidence/{object-id}
codefire://repository/{repo-id}/certificate/{object-id}
codefire://work/{packet-id}
~~~

URIはidentityであり、local pathを無条件に外部へ公開しない。path、remote URL、display IDとstable object identityを分ける。

## 4. JSON Schema

documented: JSON Schema 2020-12はCoreとValidationを分け、self-describing meta-schemaを持つ。

candidate schema family:

~~~text
codefire.command_result.v1
codefire.capabilities.v1
codefire.context_pack.v1
codefire.task_packet.v1
codefire.review_packet.v1
codefire.handoff.v1
codefire.evidence_statement.v1
codefire.consistency_certificate.v1
codefire.event.v1
~~~

Schema rule candidates:

- additionalPropertiesはcontractごとに明示する。
- extension用namespaceまたはextensions objectを用意する。
- unknown fieldを無条件errorにするstrict consumerと、forward-compatible consumerを区別する。
- enum追加、required field追加、meaning変更のcompatibility classを定義する。
- examplesではなくgolden valid/invalid vectorsを正本にする。
- schema ID、media type、semantic version、minimum reader versionを持つ。

Q-INT-001:

command_result.v1のhuman-compatible evolutionとstrict machine consumerをどう両立するか。

HYP-AI-030:

hypothesis: envelopeとdata subtypeを別schemaにし、top-level contractを狭く固定すると、command追加と互換性維持を両立できる。

反証条件:

- subtype discoveryが複雑になり、consumerが実質command名switchへ戻る。
- current outputがschemaへ適合せず、大規模breaking correctionが必要になる。

対応実験: EXP-AI-010。

## 5. MCP Adapter

documented: MCPはJSON-RPC、stateful session、capability negotiationを使い、serverがResources、Prompts、Toolsを、clientがRoots、Sampling、Elicitation等を提供できる。Hostはauthorization、consent、context aggregation、server isolationを担う。

### Candidate Resources

- repository summary
- Atom and neighboring Trace Graph
- open Fire and resolution basis
- verification report
- sealed commit/certificate metadata
- task/context/review packet schemas
- command capabilities

### Candidate Tools

read-only first:

- status
- scan dry-run
- verify
- context
- explain
- review-pack
- capabilities
- show/diff

mutation tools after trust experiments:

- evidence add
- fire
- extinguish
- commit dry-run
- commit/applyはstrong approvalとpolicyが整うまで非公開候補

### Tool Contract

各toolは次を持つ。

- stable name and version
- JSON input/output schema
- readOnly/destructive/idempotent annotations
- required root/workspace
- expected side effects
- approval requirement
- timeout/cancellation
- bounded output/pagination
- error taxonomy

### Security

- remote MCP tool descriptionをtrusted codeとみなさない。
- sensitive actionはdefault approval required。
- allowed_toolsを最小化する。
- Roots外のpathを拒否する。
- tool output内URLを無条件取得しない。
- prompt injectionを含むrepository contentとsystem policyを分離する。
- serverごとにconnection/contextを隔離する。
- tool call input/outputのsecret redactionを行う。

DC-AI-030:

最初のMCP profileをread-onlyとし、mutationはcapabilityとapproval policyが明示された別profileにする。

## 6. A2A Adapter

documented: A2AはAgent Card、Message、Task、Artifact、streaming、push notification、task stateを定義し、remote agent discoveryとlong-running workを扱う。

Candidate mapping:

| CodeFire external artifact | A2A object |
|---|---|
| agent capability/profile | Agent Card skill/extension |
| Task Packet | Task input Artifact or structured Part |
| clarification | Message |
| progress | TaskStatusUpdateEvent |
| patch/handoff/review | Artifact |
| blocked/failed/completed | Task state |
| packet family | contextId |

Boundary:

- A2A task stateはCodeFire product branch stateではない。
- A2A Message historyはdurable evidenceの正本にしない。
- critical outputはMessageではなくdigest-bound Artifactへ置く。
- CodeFire coreがA2A serverになる必要はない。external manager/adapterで十分かを先に検証する。

Q-INT-002:

A2A adoptionがない環境でもtyped packet contract単独に価値があるか。

HYP-AI-031:

hypothesis: A2Aをcanonical packet schemaのtransport adapterに留めれば、protocol lock-inなしにremote agent ecosystemへ接続できる。

反証条件:

- A2A Task semanticsとCodeFire WorkGraphが一致せず、大量のlossy fieldsが必要。
- A2A extensionなしではreview/evidence identityを表せない。

## 7. LSP And SARIF

### LSP

LSP candidate features:

- diagnostics: open Fire、missing link、stale resolution、duplicate Atom、missing evidence
- hover: Atom ID、kind、hash、neighbors、policy
- go to relation: requirement -> design -> code -> test
- code lens: Fire count、verification state
- code action: context表示、scan/verify、resolution template
- workspace command: review-pack、graph refresh
- progress/cancellation: large scan/verify

Mutation code actionはoperation planを表示し、explicit confirmation後に適用する。

### SARIF

SARIF export candidate:

- ruleId: CodeFire diagnostic kind or policy rule
- level: severity mapping
- location: artifact URI/range
- message: concise explanation
- relatedLocations: source/target Atom
- properties: fire UID、policy、subject digest
- fixes: safe command suggestionのみ。自動mutationは別contract。

責務分離:

- LSPはinteractive incremental feedback。
- SARIFはbatch/static-analysis result interchange。
- 同じinternal diagnostic modelから両方をrenderし、別々のtruthを作らない。

## 8. CloudEvents And CDEvents

Candidate CodeFire occurrences:

~~~text
scan.completed
fire.opened
fire.resolved
verification.completed
review.completed
certificate.sealed
remote.requested
remote.applied
~~~

CloudEventsはevent envelope、CDEventsはCI/CD vocabularyを提供する。CodeFire固有eventをすぐCDEvents namespaceへ入れず、custom extensionでprototypeし、一般性が実証された場合だけ標準提案する。

Event rule candidates:

- eventはoccurrenceであり、command instructionではない。
- source plus idをdedup identityにする。
- subject digest、time、schema、correlation/trace contextを持つ。
- secret、raw prompt、full patchをevent metadataへ入れない。
- at-least-once deliveryを前提にconsumerをidempotentにする。

## 9. OpenTelemetry

Candidate spans:

~~~text
codefire.scan
codefire.verify
codefire.context.build
codefire.review_pack
codefire.evidence.capture
codefire.seal
agent.task
agent.model_call
agent.tool_call
agent.handoff
validation.command
~~~

Candidate low-cardinality attributes:

- codefire.operation.name
- codefire.command.version
- codefire.policy.version
- codefire.result
- codefire.blocker.kind
- agent.role
- gen_ai.request.model
- gen_ai.response.model
- reasoning tier/effective effort
- tool name
- validation class

High-cardinality object IDs、path、prompt、tool outputはdefault metric attributeにしない。trace/eventでopt-in redacted captureする。

OpenTelemetryはobservabilityであり、証跡のauthenticityを保証しない。trace IDをEvidenceから参照できるが、traceだけでverification passを証明しない。

## 10. in-toto, SLSA, And SPDX

### in-toto

Candidate statement:

~~~json
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [
    {
      "name": "codefire-change",
      "digest": {
        "sha256": "subject-digest"
      }
    }
  ],
  "predicateType": "https://codefire.dev/attestation/consistency/v1",
  "predicate": {
    "policy": "policy-id",
    "taskPacketDigest": "digest",
    "contextPackDigest": "digest",
    "verificationEvidence": [],
    "reviewPacketDigest": "digest",
    "certificateId": "id"
  }
}
~~~

in-toto envelope/signatureはauthentication and serialization、Statementはsubject binding、predicateはCodeFire semanticsを担う。

### SLSA

SLSA provenanceはartifactがwhere/when/how producedされたかを追跡する。CodeFire certificateはspec/design/code/test整合性を扱い、SLSA build provenanceを置換しない。

Candidate relation:

~~~text
CodeFire sealed subject
  -> source/build input
SLSA provenance
  -> produced binary/package
CodeFire evidence
  -> links both digests and policy results
~~~

### SPDX

SPDX 3系はsoftware、build、AI model、dataset、provenance、integrityのmetadataを表せる。CodeFireはSBOMを独自再実装せず、Artifact/AtomからSPDX elementへのrelation exportを検討する。

## 11. Capability Negotiation

Candidate capabilities document:

~~~json
{
  "schema": "codefire.capabilities.v1",
  "protocolVersions": ["1.0"],
  "commands": {
    "verify": {
      "json": true,
      "dryRun": false,
      "readOnly": true,
      "pagination": false
    }
  },
  "schemas": {},
  "adapters": {
    "mcp": {"profile": "read-only-v1"},
    "sarif": {"version": "2.1.0"}
  }
}
~~~

Rules:

- version stringだけでbehaviorを推測させずcapabilityをquery可能にする。
- capability未宣言featureをclientが使わない。
- serverはunsupportedをstructured errorで返す。
- downgradeでsecurity/approval semanticsを失う場合はfail closed。

## 12. Conformance Profiles

| Profile | Minimum requirements |
|---|---|
| Core Reader | envelope/schema、status/show/context、no mutation |
| Diagnostic Producer | verify diagnostics、SARIF mapping、stable IDs |
| MCP Read-Only | resource/tool schemas、Roots、approval metadata |
| Evidence Producer | subject digest、capture metadata、trust level |
| Attestation Producer | in-toto statement/envelope、signature policy |
| Native Sealer | full verification、certificate、sealed graph |

Conformanceはfeature countではなく、golden vectors、invalid input、compatibility、security behaviorで判定する。

## 13. Decision Candidates And Promotion

- DC-AI-031: JSON Schema familyをprotocol kernelの正本にする。
- DC-AI-032: MCP read-only profileを最初のagent integrationにする。
- DC-AI-033: A2AはTask/Artifact adapterに留め、core dependencyにしない。
- DC-AI-034: LSPとSARIFを同じdiagnostic modelからrenderする。
- DC-AI-035: eventはCloudEvents/CDEvents、telemetryはOTel、attestationはin-toto/SLSAへ分離する。
- DC-AI-036: public conformance profilesとgolden vectorsを公開する。

Promotion先:

- schemas/: JSON Schemas and test vectors
- docs/automation_interface.md: core envelope/capability semantics
- ADR: MCP/A2A/LSP adapter boundaries
- security docs: consent、approval、redaction
- roadmap: adapter order

## 14. Primary References

- [JSON Schema specification](https://json-schema.org/specification)
- [MCP specification](https://modelcontextprotocol.io/specification/2025-06-18)
- [MCP architecture](https://modelcontextprotocol.io/specification/2025-06-18/architecture)
- [A2A specification](https://a2a-protocol.org/latest/specification/)
- [LSP specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.18/specification/)
- [SARIF 2.1.0](https://www.oasis-open.org/standard/sarif-v2-1-0/)
- [CloudEvents specification](https://github.com/cloudevents/spec)
- [CDEvents specification docs](https://cdevents.dev/docs/)
- [OpenTelemetry overview](https://opentelemetry.io/docs/specs/otel/overview/)
- [OpenTelemetry semantic conventions](https://opentelemetry.io/docs/specs/semconv/)
- [in-toto attestation](https://github.com/in-toto/attestation/blob/main/spec/v1/README.md)
- [SLSA provenance](https://slsa.dev/spec/v1.2/provenance)
- [SPDX specification](https://spdx.dev/use/specifications/)

## 15. Promotion Status

- Status: not promoted
- Required evidence: EXP-AI-007、EXP-AI-010、EXP-AI-011
- Revisit when: schema prototype、read-only MCP、SARIF/in-toto golden mappingがconformance testを通ったとき

