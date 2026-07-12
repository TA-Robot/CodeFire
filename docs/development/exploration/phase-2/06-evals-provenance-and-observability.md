# Evals, Provenance, And Observability

> **非正本 / exploration only**
>
> これはAI workflow quality、deterministic eval、agent trace、evidence trust、attestation、privacyを検討する研究ノートである。採用された結論はtest/eval strategy、evidence spec、certificate schema、telemetry policy、CI、security ADRへpromotionする。

## 1. Scope

対象:

- historical defect/task corpus
- deterministic tests、model grader、human review
- trace grading and workflow observability
- evidence capture、trust levels、subject binding
- in-toto/SLSA/OpenTelemetry export
- reproducibility、privacy、retention

対象外:

- vendor dashboardをproject truthにすること
- chain-of-thought収集
- model benchmark marketing
- certificateをabsolute correctness proofとすること

## 2. Evaluation Pyramid

Candidate authority order:

1. schema/parser validation
2. CodeFire domain invariant checks
3. compiler/unit/property/integration tests
4. reproducible dogfood/acceptance scenario
5. static/security analysis
6. independent human/agent review
7. model/trace grader

lower layerはhigher authority failureをoverrideしない。model graderはfuzzy behavior、tool choice、handoff、instruction adherence、review qualityを測る補助である。

documented: OpenAI agent evalsはtrace、grader、dataset、eval runを使い、tool selection、handoff、instruction/policy violationを評価できる。

DC-AI-060:

deterministic repository gateをeval pyramidの上位authorityとし、model graderをregression signalへ限定する。

## 3. Eval Corpus

Candidate corpus sources:

- CFB product bugs
- CFR code-review findings
- branch/open/merge/remote historical failures
- JSON envelope regressions
- docs/current-state contradictions
- dirty-tree classification
- algorithm dogfood friction
- migration/recovery incidents
- seeded security and prompt injection cases

Each case:

~~~json
{
  "schema": "codefire.eval_case.v1",
  "id": "eval-case",
  "baseRevision": "immutable-ref",
  "taskPacket": "uri-or-inline",
  "seed": {
    "kind": "historical-or-synthetic",
    "expectedFailure": "description"
  },
  "allowedChanges": [],
  "hiddenAssertions": [],
  "requiredInvariants": [],
  "scoring": {},
  "provenance": {}
}
~~~

Corpus split:

- development: prompt/packet iteration
- validation: routing/model selection
- held-out: release/regression claim
- adversarial: injection、corruption、overlap、false evidence

historical issue detailをそのままanswer keyとしてmodel contextへ入れない。

## 4. Eval Dimensions

### Task Outcome

- objective satisfied
- acceptance pass
- patch applies
- no out-of-scope change
- required docs/tests synchronized

### Correctness

- invariant preservation
- edge/error behavior
- backward compatibility
- no user work loss

### Process

- correct tool/context selection
- escalation/delegation appropriateness
- validation completeness
- review independence
- evidence accuracy

### Efficiency

- wall time
- model tokens/cost
- tool-call count
- file/context reads
- retry/rework
- coordination overhead

### Claim Calibration

- complete only when proven
- uncertainty disclosed
- skipped/unavailable gate not counted as pass
- residual risk accurate

## 5. Grader Design

Deterministic graders:

- exact command exit/result
- file set and diff
- JSON Schema conformance
- property/invariant tests
- expected diagnostic IDs
- performance threshold
- evidence digest resolution

Model graders:

- task interpretation
- review concern quality
- explanation usefulness
- tool/handoff choice
- policy adherence

Human calibration:

- sample model grader decisions
- measure agreement and systematic bias
- adjudicate ambiguous acceptance

HYP-AI-060:

hypothesis: repository-specific historical corpus predicts trusted integration quality better than generic coding benchmark。

反証条件:

- corpusがcurrent implementationへoverfitし、新feature/taskへgeneralizeしない。
- issue recordsがincompleteでground truthを作れない。
- hidden testsを通してもhuman expertが重大defectを継続発見する。

対応実験: EXP-AI-001、EXP-AI-005。

## 6. Trace Model

Candidate trace:

~~~text
workflow trace
  task packet span
  context build span
  model call spans
  tool call spans
  handoff spans
  validation command spans
  review span
  seal span
~~~

documented: OpenAI Agents SDK tracingはmodel call、tool call、handoff、guardrail、custom spanを記録できる。OpenTelemetryはtrace、metric、log、context propagationとsemantic conventionsを提供する。

Candidate correlation fields:

- workflow ID
- task packet digest
- base/patch/head digest
- agent role
- effective model/effort
- tool name
- validation class
- CodeFire certificate/evidence ID
- trace/span ID

Trace is not evidence authenticity:

- trace backend may be mutable/retained temporarily。
- trace content may be redacted。
- model/tool output may be wrong。
- trace IDはEvidenceから参照できるが、pass assertionはcaptured command evidenceで証明する。

## 7. Metrics And Cardinality

Low-cardinality metrics:

- task class
- role
- reasoning tier
- result class
- blocker kind
- validation class
- model family/version bucket

High-cardinality trace fields:

- packet ID
- object ID
- path
- Fire UID
- trace ID

Never-default content:

- raw prompt/completion
- secret/auth token
- full source/patch
- command output containing credentials
- chain-of-thought

Sampling:

- failure、security、completion false positiveはhigh retention。
- routine successful validationはsample/aggregate可能。
- evidence required runはtrace samplingと独立してcaptureする。

## 8. Evidence Trust Levels

Candidate levels:

| Level | Meaning | Example |
|---|---|---|
| asserted | agent/humanが結果を記述 | handoff text |
| locally-captured | CodeFire/wrapperがcommand/outputをcapture | local test run |
| environment-bound | cwd、env、binary/worktree identityを含む | validation runner |
| CI-attested | trusted CI identity/workflow digestを持つ | CI gate |
| signed-verified | signature、subject、issuer policyを検証 | in-toto/remote |

Policy candidate:

- low-risk docs taskはlocally-capturedで足りる。
- release/security/migrationはCI-attestedまたはsigned-verifiedを要求する。
- asserted evidenceだけでblocking Fireを解消しない。

Q-EVAL-001:

evidence trust levelをCodeFire object formatへ入れるか、external attestation relationで表すか。

Trade-off:

- core field: policy evaluationが単純だがformat coupling。
- external relation: standard interoperabilityが高いがmissing/availability handlingが複雑。

## 9. Provenance Chain

Candidate chain:

~~~text
human/issue authority
  -> Task Packet digest
  -> Context Pack digest
  -> agent run identity
  -> base + patch digest
  -> validation evidence objects
  -> Review Packet digest
  -> CodeFire consistency certificate
  -> in-toto attestation
  -> SLSA build provenance
  -> released artifact digest
~~~

Each edge should answer:

- subjectは何か
- producerは誰/何か
- policy/schema versionは何か
- input digestは何か
- signature/trustは何か
- verification可能か

## 10. in-toto Candidate

in-toto layers:

- Envelope: authentication and serialization
- Statement: subject and predicate type
- Predicate: CodeFire consistency details
- Bundle: related attestations

Candidate CodeFire predicate:

- assessment/native mode
- task/context/review digests
- base/head/patch/commit subject
- CodeFire policy and schema versions
- blocker counts
- verification evidence refs
- skipped/unavailable gates
- certificate object ID
- producer/runtime metadata

monotonic policy candidate:

missing/unknown evidenceでALLOWへ変わらない。unknown fieldを無視するforward compatibilityがDENYをALLOWへ反転しないよう設計する。

## 11. SLSA Relation

documented: SLSA provenanceはsoftware artifactがwhere、when、how producedされたかを追跡する。

CodeFire candidate:

- source consistency assessmentはSLSA build provenanceを置換しない。
- CodeFire sealed commit/assessment digestをbuild input subjectへ結ぶ。
- build artifactからSLSA provenanceを辿り、CodeFire certificateへ到達可能にする。
- builder identity、source revision、workflow digestをduplicateせずreferenceする。

Q-EVAL-002:

CodeFire consistency predicateを独自in-toto predicateとして始めるか、既存SLSA verification summaryへrelationするか。

まず独自prototypeを作り、一般性とconsumer demandを検証するcandidate。

## 12. Reproducibility

LLM outputのbyte-for-byte reproducibilityを要求しない。

Candidate reproducibility levels:

| Level | Requirement |
|---|---|
| input reproducibility | same base、packet、context、runtime profile |
| action replayability | tool/command sequence and inputs recorded |
| build reproducibility | same artifact digest where applicable |
| behavioral reproducibility | acceptance/invariant outcomes equivalent |
| decision auditability | rationale/evidence relation inspectable |

HYP-AI-061:

hypothesis: agent output textの一致ではなくbehavioral and evidence equivalenceを評価すると、model varianceを許容しつつquality regressionを検出できる。

対応実験: EXP-AI-014。

## 13. Privacy And Retention

Rules:

- chain-of-thoughtを要求・保存しない。
- prompt/completion contentはdefault off、必要時にredacted opt-in。
- secret scannerをevidence/trace write前に通す。
- external provider、region、retention、ZDR policyをrun metadataへ記録する。
- personal dataをAtom/trace metadataへ複製しない。
- deletion/retention policyとcertificate durabilityを分離する。
- artifact/evidence URIがexpiredした場合のverification behaviorを定義する。

Risk:

- provenanceを増やすほどsensitive metadataも増える。
- observability backendが新しいdata exfiltration pathになる。
- raw logsをevidenceとして永続化するとsecretをsealする可能性がある。

## 14. Adversarial Evals

- repository document内prompt injection
- malicious MCP tool description/output
- test output forgery
- skipped testをpassと主張
- patch review後のsilent edit
- stale base and stale context
- overlapping lane overwrite
- path traversal/outside workspace
- corrupt object/branch head
- huge log/context exhaustion
- flaky test and timeout
- compromised CI artifact
- model downgrade hidden
- false completion claim

Security evalはauthorized fixture内で実行し、remote/external mutationを行わない。

## 15. Decision Candidates And Promotion

- DC-AI-060: historical CodeFire defect/task corpusをrouting/release evalの中心にする。
- DC-AI-061: deterministic gateをmodel graderより上位に置く。
- DC-AI-062: evidenceへtrust level、subject digest、producer/environment identityを持たせる。
- DC-AI-063: OTel traceをobservability、in-toto/SLSAをattestation/provenanceとして分離する。
- DC-AI-064: raw chain-of-thoughtを収集せずdecision/evidenceだけを保存する。
- DC-AI-065: byte-identical outputでなくbehavioral/evidence reproducibilityを測る。

Promotion先:

- tests/evals corpus and CI
- evidence/certificate schema
- telemetry semantic convention
- security/privacy ADR
- release acceptance

## 16. Primary References

- [OpenAI Agent Evals](https://developers.openai.com/api/docs/guides/agent-evals)
- [OpenAI Agents integrations and observability](https://developers.openai.com/api/docs/guides/agents/integrations-observability)
- [OpenTelemetry overview](https://opentelemetry.io/docs/specs/otel/overview/)
- [OpenTelemetry semantic conventions](https://opentelemetry.io/docs/specs/semconv/)
- [in-toto attestation](https://github.com/in-toto/attestation/blob/main/spec/v1/README.md)
- [SLSA provenance](https://slsa.dev/spec/v1.2/provenance)

## 17. Promotion Status

- Status: not promoted
- Required evidence: EXP-AI-005、EXP-AI-006、EXP-AI-008、EXP-AI-014
- Revisit when: historical corpus、trust-level prototype、OTel/in-toto exportが同一workflowで結合されたとき

