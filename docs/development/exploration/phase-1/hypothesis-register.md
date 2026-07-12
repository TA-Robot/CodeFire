# Phase 1 Hypothesis Register

> **非正本 / exploration only**
>
> このregisterはPhase 1探索内の仮説を追跡するための一時正本であり、CodeFire製品のcurrent-stateや採用済み方針の正本ではない。支持された主張は各行のpromotion先へ移し、このregisterにはevidenceと移動先だけを残す。

## Status

| Status | Meaning |
|---|---|
| `proposed` | 問いから作った未実験仮説 |
| `testable` | fixture、oracle、metricが定義済み |
| `running` | experiment実行中 |
| `supported` | 指定範囲のevidenceが仮説を支持したが一般証明ではない |
| `falsified` | 反証条件を満たした |
| `inconclusive` | evidenceが弱い、矛盾、sample不足 |
| `promoted` | 正本へ採用済み |
| `retired` | 不要/重複。理由を残して終了 |

## Priority

- `P0`: repository/history/securityを壊す、またはCodeFireの中心的correctness claimを誤る可能性。
- `P1`: scale、recovery、false confidence、主要workflowの信頼性へ大きく影響。
- `P2`: 最適化、使いやすさ、将来architectureの判断材料。

## Register

| ID | Claim | Evidence now | Priority | Primary falsifier | Metric | Experiment | Status | Promotion target |
|---|---|---|---|---|---|---|---|---|
| `HYP-FORMAL-001` | blocker collection/policy evaluationを一つのdomain predicateへ集約すると仕様・commit・certificate・JSON差を減らせる | implementation/spec差を`observed` | P0 | commandごとに正当なdecision差があり共通predicateが例外化 | 64 blocker statesのdecision一致率 | `EXP-FORMAL-001` | testable | consistency model、core API、certificate spec |
| `HYP-FORMAL-002` | state labelをfactsから導出可能なread modelにするとmetadata矛盾を回復しやすい | 複数state copyを`observed` | P1 | recompute不能またはstatus latencyが許容外 | state disagreement、latency | `EXP-FORMAL-002` | proposed | state ADR/module、doctor |
| `HYP-FORMAL-003` | executable state modelがcross-file/interleaving bugを発見する | `inferred` | P1 | model counterexampleがほぼ偽陽性、維持cost過大 | unique valid counterexamples | `EXP-FORMAL-003` | proposed | test strategy/CI |
| `HYP-TXN-001` | branch/MR pointer publishを唯一のlinearization pointにできる | transaction orderingを`documented` | P0 | head後failureがnormal readを安全に成立させない | invalid partial states | `EXP-TXN-001` | testable | transaction ADR、consistency model |
| `HYP-TXN-002` | operation/scope/key/payload hashの共通idempotency contractでretryを安全化できる | module分散を`observed` | P1 | result persistenceをmutationと整合させられない | duplicate effect、conflict detection | `EXP-TXN-002` | proposed | automation/remote protocol |
| `HYP-TXN-003` | expected-head CAS相当contractでbackend間fast-forward invariantを維持できる | fast-forwardを`documented` | P1 | MR/nonce/audit consistencyをCASで表せない | lost update、backend contract parity | `EXP-TXN-003` | proposed | remote protocol/ADR |
| `HYP-STORE-001` | language-independent canonical corpusがruntime migrationのidentity splitを防ぐ | canonical identityを`documented` | P0 | historical objectsを一規則で再現不能 | runtime間golden一致率 | `EXP-STORE-001` | testable | object-store spec、compat fixtures |
| `HYP-STORE-002` | same short IDでもfull digest不一致をcollision errorにすべき | short ID/reuse pathを`observed` | P0 | current formatでfull digestを信頼不能 | artificial collision silent alias数 | `EXP-STORE-002` | testable | store implementation/spec/tests |
| `HYP-STORE-003` | GCがretention reasonを説明できればsafe deletionと理解を両立できる | GC root complexityを`inferred` | P1 | explanation costが実用不能 | reason coverage、GC work | `EXP-STORE-003` | proposed | GC/storage/doctor design |
| `HYP-PROTO-001` | compatibility matrixとcapability advertisementがsilent misreadを防ぐ | multiple schemas/transportsを`documented` | P1 | single-version rejectionの方が低cost | version-pair test coverage | `EXP-PROTO-001` | proposed | protocol/schema docs |
| `HYP-SEMVER-001` | extractor version identityをAtomIndex/resolution validityへ反映すべき | basis fieldsを`observed` | P0 | existing hashes/Atom setで必ず安全にstale化 | false active/false stale resolutions | `EXP-SEMVER-001` | testable | AtomIndex schema、migration ADR |
| `HYP-VCS-001` | command-scoped sealed validation memoがmulti-head readを安全に高速化する | repeated traversalを`observed` | P0 | policy/context依存またはmemory/diagnostic悪化 | object reads、runtime、RSS | `EXP-PERF-001` | testable | store API、branch/remote readers |
| `HYP-VCS-002` | derived identity matchはcandidateに限定しsealed identityへ使わない方が安全 | extractor/Atom modelを`documented` | P1 | conservative false positive costが許容不能 | rename precision/recall、Fire数 | `EXP-VCS-001` | proposed | Atom/diff product spec |
| `HYP-VCS-003` | file/Atom/Trace/Fire diffのlayerとprovenanceを分けるとreview精度が上がる | layered modulesを`observed` | P2 | output complexityでreviewが悪化 | defect detection、review time | `EXP-VCS-002` | proposed | diff/automation spec |
| `HYP-VCS-004` | semantic auto-mergeを避け候補/Fire/verifyへ分ける現方針はtrust上有利 | current policyを`documented` | P1 |限定auto-resolutionがfalse acceptなしで大幅短縮 | semantic precision、merge lead time | `EXP-VCS-003` | proposed | merge ADR/spec |
| `HYP-GRAPH-001` | typed propagation policyがFire生成を説明・制御可能にする | direct-only MVPを`documented` | P1 | link/change qualityが低くnoise増大 | Fire amplification、precision/recall | `EXP-GRAPH-001` | proposed | policy schema/core |
| `HYP-GRAPH-002` | parsed graphとvalidated graphを型で分けるとvalidation bypassを減らせる | string/Value boundaryを`observed` | P2 | context-dependent validationで型が形骸化 | invalid graph consumer数 | `EXP-GRAPH-002` | proposed | core/store API ADR |
| `HYP-PERF-001` | work countersはwall timeよりalgorithm regressionを安定検出する | complexity riskを`inferred` | P1 | instrumentation cost/driftが過大 | x10 scale時work倍率 | `EXP-PERF-002` | proposed | benchmark/metrics strategy |
| `HYP-SEC-001` | root-scoped safe path typeとwrite直前確認がpath policy差を減らす | path boundaryを`documented` | P0 | platform差またはlegitimate symlink破壊 | escape success、false reject | `EXP-SEC-001` | testable | filesystem/path ADR |
| `HYP-SEC-002` | HMACをtrusted single-team境界としpublic/multi-tenantへ強いkey/replay基盤を要求すべき | HMAC limitationを`documented` | P1 | deploymentが恒久的にsingle operator | control coverage、rotation RTO | `EXP-SEC-002` | proposed | remote security docs |
| `HYP-SEC-003` | sandboxなしserver verificationはtrusted admin policyに限定すべき | no OS sandboxを`documented` | P0 | policyが完全server-ownedまたはprocessが常時強隔離 | escape capability、deployment evidence | `EXP-SEC-003` | testable | remote spec、runbook/security ADR |
| `HYP-SEC-004` | bytes/count/depth/time/memoryの独立budgetがDoS耐性を改善する | bounded output/timeoutを`observed` | P1 | hard limitのfalse rejectが支配的 | limit coverage、peak resource | `EXP-SEC-004` | proposed | protocol/config/CLI docs |
| `HYP-SEC-005` | release manifestがsource revision、artifact、signature、SBOM、versionを結ぶべき | packaging limitationを`documented` | P1 | source-only distributionへ限定 | provenance coverage | `EXP-SEC-005` | proposed | release/installer strategy |
| `HYP-OBS-001` | common phase/correlation/work-counter eventがraw contentなしでMTTRを下げる | phase separationを`documented` | P1 | event overhead/volume/semantic intrusionが過大 | diagnosis success、log bytes | `EXP-OBS-001` | proposed | automation/metrics docs |
| `HYP-REC-001` | phase別recovery decision tableがdoctorのsafe next actionを改善する | pending/stale lock gapを`observed` | P0 | marker情報だけでは安全な判断不能 | correct recovery choice、data loss | `EXP-REC-001` | testable | doctor/runbook/ADR |
| `HYP-TEST-001` | invariant-to-test matrixがtest countよりfalse confidenceを減らす | test concentrationを`observed` | P1 | mappingが主観的で追加defectを発見しない | mapped mutation kill/escaped defect | `EXP-TEST-001` | proposed | test strategy/gates |
| `HYP-TEST-002` | random operation sequenceとreference modelがstate interaction bugを発見する | `inferred` | P1 | shrink/reproductionがflakyまたは高cost | minimized counterexamples | `EXP-TEST-002` | proposed | core/CLI tests |
| `HYP-TEST-003` | critical predicate限定mutation testingがsuiteの実効性を測れる | `inferred` | P2 | equivalent mutant/runtime cost過大 | mutation score、triage time | `EXP-TEST-003` | proposed | validation strategy |
| `HYP-EVAL-001` | historical CFB/CFRをhidden corpus化するとAI/context/reviewを比較できる | issue corpusを`observed` | P1 | reproducibility不足またはanswer leakage | replay率、defect recall | `EXP-EVAL-001` | proposed | AI eval/test fixtures |
| `HYP-HCI-001` | progressive diagnostic disclosureがcorrect actionを速める | output boundingを`observed` | P2 | navigationが遅くcritical blockerを隠す | task time、miss率 | `EXP-HCI-001` | proposed | CLI/automation UX spec |
| `HYP-HCI-002` | target/precondition/mutation/result付きnext actionがautomation safetyを高める | next_actionsを`documented` | P1 | stale化/payload costで成功率低下 | suggestion success、wrong target | `EXP-HCI-002` | proposed | automation schema |
| `HYP-HCI-003` | Fire cause/path/basis説明がblind extinguishを減らす | explain/evidence surfaceを`documented` | P1 | explanationがfalse confidence/noiseになる | incorrect resolution、rationale quality | `EXP-HCI-003` | proposed | Fire/explain product docs |
| `HYP-CONTEXT-001` | typed source/persistence/invariant/test graphがcontext recall/tokenを改善する | static packsを`observed` | P1 | staleness/maintenance costがbenefit超過 | oracle recall@tokens | `EXP-CONTEXT-001` | proposed | AI context tooling |
| `HYP-CONTEXT-002` | revision/path/checksum/omission付きsummaryが安全なcompressionを可能にする | `inferred` | P2 | summaryが即staleまたはnegative constraintを落とす | stale detection、task success | `EXP-CONTEXT-002` | proposed | AI operating protocol |
| `HYP-EVAL-002` | model/tierはtrusted integration lead time/rework/defectで比較すべき | `decision-candidate` | P1 | human oracle/task varianceで測定不能 | accepted change当たりtime/cost | Phase 2へpromotion | proposed | Phase 2 routing/eval docs |

## Immediate Order

次の順序を初期実験順とする。P0でもfixture/tooling依存により並行可能である。

1. `HYP-FORMAL-001`
2. `HYP-TXN-001`
3. `HYP-STORE-002`
4. `HYP-VCS-001`
5. `HYP-SEMVER-001`
6. `HYP-SEC-003`
7. `HYP-SEC-001`
8. `HYP-REC-001`
9. `HYP-STORE-001`
10. P1/P2をrisk、dependency、available environmentで選ぶ。

## Update Rule

仮説を更新するときは次を同時に記録する。

- status変更日。
- 実行した `EXP-*` とevidence path。
- 支持範囲。全称命題として誇張しない。
- falsifierを満たしたか。
- alternative explanation。
- promotion、retest、retireの次行動。

## Promotion Status

- `promoted`: 0
- `supported`: 0
- すべて探索開始時点では未採用。
- 集計値はこの探索registerだけの状態であり、product progressを表さない。
