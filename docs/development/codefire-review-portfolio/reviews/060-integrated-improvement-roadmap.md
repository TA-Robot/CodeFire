# RVC-060: Integrated improvement roadmap

## Review Summary

| Field | Value |
|---|---|
| Status | accepted_non_goal |
| Perspective | architecture / correctness / evolution / product |
| Review kind | refine |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 1 |
| Primary scope | `RVC-001〜059 discovery portfolio integration` |
| Promotion target | strategy experiment / none |

## Executive Judgment

RVC-001〜059は59本の独立実装laneではない。RVC-011/CFR-161とRVC-039のCFR-162 safety sliceはRetrospectまで完了し、その後のcurrent-source terminal auditは6 implemented、3 accepted non-goal、51 residualと判定した。51 residualはCFR-163..173へexactly once mappingされ、product/adoption、CLI workflow、architecture decompositionはsafety/contract rootの後段に置かれた。`modernization-roadmap.md`は唯一のactive roadmap、`review-dispositions.json`はterminal mappingの正本であり、本文書は実装initiativeや第二roadmapにしない補助mapとしてterminal化する。

## Scope and Review Question

- In scope: (A) product claim/adoption RVC-001〜010、(B) authoritative predicate/domain semantics RVC-011〜020、(C) identity/seal/transaction/recovery RVC-021〜030、(D) CLI/machine contract/workflow RVC-031〜040、(E) remote/security/scale RVC-041〜050、(F) architecture/testing/evolution RVC-051〜059のroot cause、依存順、Stage 1〜6への接続、promotion/kill条件
- Out of scope: active roadmapの置換、日付/人員commitment、59候補の一括issue化、現在の凍結batchへの割り込み
- Question: authoritative decisionとsafety rootを先に固定し、proof・public contract・adoption・operational trust・product proof・extensionをStage 1〜6で飛ばさず進める最小の順序は何か

## Evidence

- `docs/development/codefire-review-portfolio/README.md` — `Governance` / `Promotion Workflow`: 60件は60 taskではなく、同じroot causeは昇格前に統合し、`modernization-roadmap.md`を唯一のactive roadmapとする。
- `docs/development/strategy/modernization-roadmap.md` — `Stage 1 — Correctness and Recovery Foundation` / `Ordering Rules`: committable predicate、crash/idempotency、canonical corpus、extractor semantics、performance budget、trust boundary、recoveryを先行させ、P0 safetyがadoptionに優先する。
- `docs/development/strategy/modernization-roadmap.md` — `Stage 2`〜`Stage 6`: public schema/conformance、read-only adoption、operational trust、product proof、extension standardizationをevidence gateの順に置く。
- `crates/codefire-core/src/lib.rs` — `build_verification`: Fire、required link、stale resolution、evidence、check、duplicate Atomを一つのverification resultへ統合し、RVC-011をprojectionより前のdecision rootに置く実装根拠になる。
- `crates/codefire-store/src/lib.rs` — `validate_sealed_commit` / `required_commit_roots`: commit identity、parent、root type/payload、reachable referenceの消費境界を持ち、RVC-021〜026のidentity/publication rootが共有する不変条件を示す。
- `crates/codefire-cli/src/main.rs` — `run_commit` / `begin_commit_transaction` / `complete_commit_transaction`: verificationからobject write、branch publication、active reset、idempotency recordまで複数のvisibility pointを持ち、transaction/recovery/concurrency候補を一つのprotocolで扱う必要を示す。
- `crates/codefire-cli/src/http.rs` — HTTP upload/apply mutation handlers: request signature・idempotency lookupの後にresource lockを取り、branch/MR/idempotencyを更新するため、RVC-041/044/050はpayload・lock・publicationが結びつく同一rootである。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs` のcommit pending、corrupt object、remote generation concurrencyの局所fixtureとRVC-056の指摘は、個別fixより先に共通fault/race proof seamを置く必要を示す。

## Strengths to Preserve

- active roadmapが日付ではなくevidence gateとordering ruleを持ち、後段prototypeの早期実験と先行safety gateを両立させる。
- RVCが強み、gap、counterexample、acceptance、duplicateを同じ形式で持ち、機能数を改善と誤認しない。
- CFB-120〜132、CFR-161、CFR-162はInstall、Dogfood、Retrospectまで完了し、全RVCのterminal auditとCFR-163..173へのdeduplicationも完了した。

## Gap / Risk

### Observed gap

RVC-001〜059はA〜Fの観点別に読める一方、同じdecision、publication、subject、version、work-budget、test seamが複数群にまたがる。root ownerとstage gateなしでは、同一invariantをspec、CLI、remote、testで別々に昇格してしまう。

### Failure or stagnation mode

59 laneを並走させてactive roadmapと競合する、RVC-051/052/054のarchitecture cleanupがRVC-011/023〜026/044/050のcounterexampleより先行する、RVC-002/040のadoption UXがStage 1/2 gateを迂回する、RVC-033と57を別々のversion policyにする、または本候補が第二のactive roadmapになる。

### Affected invariants

- single authoritative committable decision and claim projection
- sealed identity, compare-and-publish, lock/idempotency/recovery ordering
- subject-bound evidence/request trust and no silent schema downgrade
- Stage ordering and single current-state owner

## Improvement Direction

本文書はactive roadmapを変更せず、59候補を六つのcoverage groupと7つのprimary root clusterへ写像する。A〜Fは「何を読むか」、root clusterは「何を一つのfix/ADR/spec/experimentとして昇格するか」の分類である。

| Coverage group | RVC | Stage 1〜6での役割 |
|---|---|---|
| A. Product claim / adoption | 001〜010 | Stage 2でclaim境界、Stage 3でread-only wedge、Stage 5で理解・actionability・価値を反証するdownstream consumer |
| B. Authoritative predicate / domain semantics | 011〜020 | Stage 1の最上位decision rootとdomain invariant、Stage 2のproof/claim projectionの入力 |
| C. Identity / seal / transaction / recovery | 021〜030 | Stage 1でidentity、validated read、visibility point、lock、recoveryを一つのpublication protocolにする |
| D. CLI / machine contract / workflow | 031〜040 | Stage 2でtyped lifecycle/schema/outcomeを消費可能にし、Stage 3/5でsafe workflowと理解性を検証する |
| E. Remote / security / scale | 041〜050 | Stage 1でremote mutation・resource・path・concurrencyを閉じ、Stage 2でbounded transport、Stage 4でsensitive evidence/credential drillへ接続する |
| F. Architecture / testing / evolution | 051〜059 | Stage 1の最小proof seam、Stage 2のconformance/version governance、Stage 6のextension ruleを供給する |

各RVCは下表のprimary clusterに一度だけ属する。cluster間の依存は残すが、formal promotion ownerを複数にしない。

| Primary root cluster | RVC-001〜059の割当 | Earliest gate / deduplication judgment |
|---|---|---|
| Authoritative decision root | **RVC-011**、RVC-008、RVC-016、RVC-034 | RVC-011を最上位のStage 1 decisionとし、policy・non-Atom・CLI outcomeを個別boolean/renderer fixにしない |
| Publication-concurrency root | **RVC-023、RVC-024、RVC-025、RVC-026、RVC-044、RVC-050**、RVC-012、RVC-021、RVC-027、RVC-028、RVC-036、RVC-037、RVC-045、RVC-056 | Stage 1でvalidated read、Fire/Resolution ledger agreement、full identity、compare-and-publish、lock、idempotency、path use、recoveryを一つのprotocolとfault/race proofに束ねる。RVC-012はdomain invariant/validator、RVC-037はそのmulti-write fault producerとして同じformal ownerへ統合する |
| Subject-trust root | **RVC-005、RVC-018、RVC-019、RVC-041、RVC-046**、RVC-038、RVC-043 | Stage 1でassessment/request subjectとtrust profileを固定し、Stage 4までsensitive capture・key/nonce・credential drillを別の「security feature」に分断しない |
| Version-semantics root | **RVC-006、RVC-007、RVC-013、RVC-022、RVC-033、RVC-055、RVC-057、RVC-059**、RVC-017、RVC-032、RVC-049、RVC-053 | Stage 1でextractor/canonical vectorsをseedし、Stage 2でsupport decision/corpusへ統合、Stage 6でcritical extensionへ進む。RVC-033はpublic command/input/output schema、RVC-057はpersisted object/repository/remote formatを担当する |
| Bounded graph/resource/performance root | RVC-014、RVC-015、RVC-029、RVC-030、RVC-039、RVC-042、RVC-047、RVC-048、RVC-058 | Stage 1でdecision-equivalent work budgetを固定する。TraceGraph semanticsはRVC-014/015、persisted commit/object/signature historyはRVC-048がownerとなり、Stage 2のpagination/output contractより先に正しい仕事量を証明する |
| Architecture/proof seam | RVC-020、RVC-031、RVC-051、RVC-052、RVC-054 | Stage 1 root fixに必要なIO/domain、lifecycle、fixture seamだけをbehavior-preservingに抽出し、file/crate分割自体をpromotion成果にしない |
| Product/adoption/workflow experiment | RVC-001、RVC-002、RVC-003、RVC-004、RVC-009、RVC-010、RVC-035、RVC-040 | Stage 2 claimとsafe action contractの後にStage 3で入口を試し、Stage 5で理解・Fire actionability・価値・stop ruleを検証する |

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 7 root clusterごとにproof→root fix→promotion | 001〜059の重複を抑え、Stage 1〜6のgateを守れる | triageで多くをmerge/defer/killする必要 | adopt |
| B: RVC-001〜059をそれぞれ正式issue化 | 件数と進捗は見えやすい | backlog膨張、重複fix、active roadmap競合、stage bypass | reject |

## Recommended Sequence

1. **Characterize**: source Close済みCFB-120〜126をInstall/Dogfood/Retrospectし、最初にRVC-011のdecision tableを再監査する。次にRVC-023〜026/044/050のpublication-concurrency、RVC-005/018/019/041/046のsubject-trust、RVC-006/007/013/022のversion-semanticsにseeded counterexampleを置き、RVC-056で再現する（Stage 1）。
2. **Separate**: RVC-056のfault/fixture seamとRVC-054のtest-suite ownershipを区別する。RVC-020/031/051/052/054はroot fixに必要な最小抽出だけ行い、RVC-014/015/029/030/042/047/048/058のwork counterを先に固定する（Stage 1）。
3. **Implement**: RVC-011のtyped decisionを一度だけ構築し、publication-concurrency、subject-trust/path、bounded resourceの順に一clusterずつroot fixする。RVC-001のclaim、RVC-034のoutcome、RVC-049のpublic paginationはこの時点で独立実装しない（Stage 1）。
4. **Prove**: RVC-022/055のcanonical・cross-runtime corpusを基盤に、RVC-006/007/013/033/057/059のversion support decisionを固定する。RVC-033はpublic command/input/output、RVC-057はpersisted object/repository/remoteの担当を守り、RVC-032〜039/049/053を独立minimal consumerへ通す。その後にRVC-001のclaim descriptorをpublic contractにする（Stage 2）。
5. **Promote**: Stage 3はRVC-002のread-only wedgeとRVC-009/035/038/040のsafe workflowから最大一つを試し、mutation authorityを広げない。Stage 4はRVC-005/018/019/041/043/046をsensitive-evidence・credential-compromise・backup/restore drillへ接続する。Stage 5はRVC-001/003/004/010のclaim理解、working-progress、Fire actionability、product stop ruleを検証する。Stage 6はRVC-007/032/055/059のnamespace/conformanceが複数の独立実装で実証された後だけ進める。

## Acceptance Signals

- RVC-001〜059の全IDがA〜Fの一groupと7つのprimary root clusterの一つへ漏れ/重複なくmappingされる。
- RVC-011は最初のsemantic decision rootとして完了し、残差はcurrent cycle planのlocal authority、subject trust、remote publication、version/policy、bounded work、contract/experiment順へ接続された。
- 現在batch完了前に新RVCをCFB/CFR採番せず、promotion後もactive roadmap正本は`modernization-roadmap.md`一つである。
- Stage 3〜6のprototypeがStage 1のsafety/identity/recoveryとStage 2のschema/conformance gateを迂回しない。

## Verification Design

- Focused tests: RVC-011 decision matrix、RVC-023〜026/044/050のvisibility/lock/idempotency matrix、RVC-005/018/019/041/046のsubject/trust vectors
- Cross-boundary tests: RVC-022/055 corpus、RVC-033/057の分担を跨ぐsupport decision、Stage 2 independent minimal consumer
- Scale or fault tests: RVC-056 phase/race harness、RVC-014/015/029/030/042/047/048/058 budget、RVC-045 path red team
- Observability: `RVC → coverage group → primary root → predecessor → formal artifact → exit evidence`を追跡し、RVC数や消化件数を進捗KPIにしない

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-120〜132はRetrospectまで完了した。CFR-120のroot-cause mapping方針と各RVCが示すfixed CFB/CFRの成果を再利用し、RVC-011のcharacterization後にだけ新しいformal open issue mapへ登録する。
- Required predecessor: 完了済み。凍結batch close audit、RVC-011 authoritative decision、RVC-001〜059 owner triageを経て、formal ownersは`review-dispositions.json`とcurrent cycle planへ固定した。
- Can combine with: RVC-001〜059の全候補。ただし同時実装ではなく、7 primary clusterごとに一つのformal ownerへpromotion/merge/killする。RVC-033/057は共通version-governance下でpublic schemaとpersisted formatを分担する。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- `modernization-roadmap.md`、release roadmap、正式CFB/CFR backlogを置換せず、RVC-060自体を実装initiativeや進捗KPIにしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
