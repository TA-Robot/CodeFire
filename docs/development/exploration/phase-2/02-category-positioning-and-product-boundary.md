# Category Positioning And Product Boundary

> **非正本 / exploration only**
>
> これはCodeFireのcategory、competitive differentiation、Git bridge、product non-goalを検討する研究ノートである。採用された結論はstrategy、docs/01_product_definition.md、README、ADR、roadmapへpromotionする。

## 1. Current Position

documented:

- CodeFireはconsistency-first version-control style CLIである。
- Git-compatibleではなく、Git上のlayerでもない。
- formal historyには整合済みsealed stateだけを残す。
- AI orchestration、agent marketplace、prompt management、GUIをproduct non-goalとする。
- external AI、CI、human workflowにはJSON、dry-run、next_actions、context、review-pack、evidenceを提供する。

この境界は思想的一貫性がある。一方、de facto adoptionを目標にすると、既存repository、PR、IDE、CIを全面置換する導入costが最大riskになる。

## 2. Candidate Categories

### Option A: Replacement VCS

Value:

- consistent-only historyを最も強く保証できる。
- branch/open directory/commit certificateを一つのmodelで制御できる。
- GitのWIP、partial commit、force操作に引きずられない。

Risk:

- Git hosting、PR ecosystem、IDE、CI、developer habitを同時に置換する必要がある。
- proof-of-value前のmigration costが高い。
- package managerやdeployment systemがGit commitをidentityとして使う場合にbridgeが必要。

### Option B: AI Agent Platform

Value:

- userに見えやすく、agent boomへ直接乗れる。
- task delegation、prompt、marketplaceを一製品へ統合できる。

Risk:

- current non-goalと正面衝突する。
- model/provider UIの変化速度へproduct scopeが支配される。
- CodeFire固有のmoatであるconsistency/evidenceが薄まる。
- orchestration competitorとfeature raceになる。

rejected candidate:

現時点ではOption Bを棄却する。再検討条件は、consistency kernel単独では外部orchestratorから必要なauthority/identityを得られないことが実験で示された場合だけとする。

### Option C: AI Change Consistency And Evidence Protocol

Value:

- agent、human、IDE、CI、SCMを問わず同じAtom/Trace/Fire/Evidence contractを使える。
- execution runtimeとtrust kernelを分離できる。
- MCP、SARIF、in-toto、OTelなど標準へ接続できる。
- sealed VCS modeを高保証profileとして残せる。

Risk:

- category説明が抽象的で、初回価値を見せにくい。
- protocolだけではUXが弱く、reference adaptersが必要。
- core VCSとadapter modeの保証差を明確にしないと誤解される。

### Option D: Git-Native Sidecar

Value:

- PR/CIへ最短で導入できる。
- advisory scanから開始できる。
- existing commit/branch identityを活用できる。

Risk:

- Git上位layerではないという現方針に抵触する。
- WIP/partial commitを許すGit historyとconsistent-only claimが混ざる。
- Git semanticsがcoreへ漏れるとarchitectureが二重化する。

## 3. Recommended Hypothesis

HYP-AI-020:

hypothesis: 外向きcategoryをOption Cとし、Option Dをexternal adoption adapter、Option Aをhigh-assurance native modeとして提供すると、core invariantを保ちながらadoption funnelを広げられる。

~~~text
Protocol category
  shared schemas, diagnostics, evidence, certificate, conformance

Adoption adapter
  Git/PR/CI read-only or advisory integration

Native high-assurance mode
  CodeFire repository and consistent-only sealed history
~~~

これは採用決定ではない。特にGit bridgeは現product boundaryを変える可能性があるためADRとexperimentを必要とする。

反証条件:

- adapter modeでnative modeとの差をuserが理解できず、certificateの意味を恒常的に誤認する。
- Git metadataとCodeFire object identityの同期costが高く、false staleやduplicate stateが増える。
- advisory modeではmeaningful Fireが得られず、全面migrationしない限り価値が出ない。
- bridge maintenanceがcore developmentを圧迫する。

対応実験: EXP-AI-009、EXP-AI-012。

## 4. Git Bridge Shapes

### Bridge 1: Read-Only Snapshot Scan

- Git working treeまたはcommit treeをinput snapshotとして読む。
- CodeFire repositoryへbranch mutationしない。
- diagnostics、context、SARIFだけを返す。
- certificateではなくassessment reportと明示する。

最も低riskなpilot candidate。

### Bridge 2: PR Advisory Gate

- base/head SHAをsubject identityとして固定する。
- changed files/Atoms/Trace/Firesを計算する。
- PR check、review-pack、SARIF、evidence summaryを出す。
- failureは最初はnon-blocking。

### Bridge 3: Attested CI Gate

- CI runnerがbase/head、workflow digest、test evidenceをcaptureする。
- in-toto/SLSA compatible attestationへCodeFire assessmentを結ぶ。
- selected path/policyだけblockingにする。

### Bridge 4: Import To Native Seal

- Git revisionをCodeFire Artifact/commit subjectとしてimportする。
- full CodeFire verify後にnative sealed commitを作る。
- Git history自体をconsistent-onlyとは主張しない。

### Rejected Shape: Transparent Git Compatibility

Git command、object format、rebase、stash、partial commitを透過再現する案は、current philosophyとimplementation scopeを壊すため棄却候補。再検討には独立product line相当の根拠が必要。

## 5. Certificate Semantics By Mode

| Mode | Allowed claim |
|---|---|
| snapshot advisory | このsnapshotをこのpolicyで評価した結果 |
| PR gate | このbase/head diffのrequired checksとFire状態 |
| attested CI | このsubject digestをこのrunner/workflowで検証した結果 |
| native sealed | CodeFire repository policy下で整合済みとして封印されたstate |

全modeで同じconsistentという語を無条件に使わない。

DC-AI-020:

certificateにmode、subject type、subject digest、policy version、evidence trust levelを必須化する。

## 6. Competitive Differentiation

比較対象を単一product名ではなくproblem classで捉える。

| Existing class | Strength | CodeFire candidate differentiation |
|---|---|---|
| SCM/PR | diff、history、collaboration | cross-artifact obligationとconsistent-only seal |
| Agent runtime | code generation、tool use、orchestration | runtime-neutral verification and evidence |
| Static analyzer | language-specific finding | spec/design/code/test/opsを跨ぐTrace/Fire |
| Requirements tool | traceability、approval | source-integrated machine workflowとchange impact |
| Supply-chain attestation | build provenance | pre-build semantic consistency and resolution basis |
| CI platform | repeatable commands | why-change、what-must-be-reviewed、certificate |

Moat candidate:

1. accumulated Trace Graph and resolution basis
2. historical failure/eval corpus
3. policy and evidence trust semantics
4. conformance suite and public schemas
5. adapters across agent/IDE/CI/provenance ecosystems
6. safe native sealed workflow

model name、prompt template、agent countはdurable moatにしない。

## 7. Initial Wedges

candidate order:

1. SARIF/PR advisory: existing workflowでFireを見せる。
2. read-only MCP: agentがCodeFire contextとdiagnosticsを安全に取得する。
3. evidence/certificate export: CI/security/release ownerへ価値を示す。
4. IDE/LSP: edit直後にimpactを説明する。
5. native sealed mode: high-assurance projectへ拡張する。

選定基準:

- time-to-first-value
- existing workflow mutationの少なさ
- CodeFire固有価値が見えるか
- security boundaryの単純さ
- adapterを削除してもcoreが残るか

## 8. Adoption Risks

### Fire Fatigue

大量のFireが機械的に発生し、userがinvalid-fireまたは理由のcopyで閉じる。

metric:

- valid Fire率
- resolution再open率
- Fire per changed Atom
- median resolution time
- bulk override率

### Metadata Tax

Atom ID、Trace Link、policyの初期設定costが価値より先に来る。

mitigation hypothesis:

- one critical flowから始める。
- generated candidate IDsとexplicit IDsを併用する。
- backfillを必須にしない。
- policyをadvisoryから段階化する。

### False Assurance

certificateがabsolute correctnessやsecurity proofと誤解される。

mitigation:

- subject、policy、coverage、skipped gates、trust levelをcertificateへ含める。
- human outputでも保証範囲を省略しない。

### Ecosystem Isolation

Git/IDE/CIへ繋がらず、native modeを試す前に離脱する。

mitigation:

- standard exportとthin adapterをfirst-class roadmapにする。

## 9. Category Validation

Q-CAT-001:

userはCodeFireをVCS、CI gate、requirements trace tool、AI safety toolのどのcategoryとして理解すると最も早く価値へ到達するか。

Q-CAT-002:

native sealed historyがなくても、Fire/context/evidenceだけで継続利用に足る価値が出るか。

Q-CAT-003:

protocol categoryは購買者、maintainer、developerへ説明可能か。それとも具体的なPR/IDE product wedgeが先に必要か。

測定:

- installからfirst meaningful findingまでの時間
- explanation後に正しく言い換えられるuser比率
- advisoryからselected blockingへの移行率
- week 4/12 retention
- pilotで防げたactual consistency defect

## 10. Decision Candidates And Promotion

- DC-AI-021: categoryをAI Change Consistency and Evidence Protocolとする。
- DC-AI-022: native VCSをhigh-assurance mode、Git bridgeをexternal adapterとして分離する。
- DC-AI-023: snapshot/advisory結果とnative consistency certificateを別claim typeにする。
- DC-AI-024: initial wedgeをSARIF/PR advisoryまたはread-only MCPから選ぶ。
- DC-AI-025: transparent Git compatibilityは採用しない。

Promotion先:

- category statement: strategy、README、docs/01_product_definition.md
- Git boundary: ADR
- certificate mode: product spec、schema、tests
- adoption wedge: roadmap、integration docs

## 11. Promotion Status

- Status: not promoted
- Required evidence: EXP-AI-009、EXP-AI-012、real pilot interview/evidence
- Revisit when: read-only snapshot/PR prototypeでfirst-valueとguarantee comprehensionを測定したとき

