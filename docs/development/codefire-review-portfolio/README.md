# CodeFire Improvement Review Portfolio

Status: terminal audit complete; residual root batch frozen

Review baseline: Git `f164412` (`main`, reviewed 2026-07-12)

このポートフォリオは、CodeFire本体を「今あるバグ」だけでなく、product claim、domain model、system architecture、implementation、user workflow、evolutionの観点から再評価した60件のレビューである。devcontainer、CI、install/package、外部AI運用などの周辺環境は対象に含めない。Stage 1 Retrospect後、全60件をcurrent sourceへ再監査し、6 implemented、3 accepted non-goal、51 promotedへterminal分類した。正本は[review-dispositions.json](review-dispositions.json)であり、51 residualはCFR-163..173へexactly once mappingされている。

## Governance

- 各 `RVC-*` は分析単位であり、正式な `CFB-*` / `CFR-*` issueではない。
- 60件は60個の実装taskを意味しない。同じroot causeを共有する候補は、昇格前に一つのfix/ADR/experimentへ統合する。
- 各reviewのPriorityは「記述したriskが成立した場合の重大度」であり、そのまま実装順やbatch件数を表さない。実行順は既存roadmap gate、counterexample、root-cause deduplicationで決める。
- `BATCH-2026-07-stage1-invariants-dogfood-blockers`、CFR-161、CFR-162、CFR-163/CFB-135、CFR-164は2026-07-12にInstall、Dogfood、Retrospectまで完了した。RVC-027のlocal-authority residualとRVC-005/018/038/041/043/046のsubject-trust residualは各root ownerのfixed evidenceとして保持し、terminal disposition `promoted`は書き換えない。`BATCH-2026-07-review-portfolio-completion`の残り11 formal issuesがRetrospectするまで新Discoveryを停止する。
- 実装へ進めるときは、[review-template.md](review-template.md) のPromotion Checklistに従い、既存issueとの重複を除いたうえでCFR、product spec、ADR、strategy experimentへ昇格する。
- `modernization-roadmap.md` が唯一のactive roadmapであり、RVC-060もそれを置き換えない。
- `review-dispositions.json` がRVC status、root owner、formal artifact、current evidenceのmachine-readable正本である。project docs checkerは60/60 coverage、terminal status、review header、checklist、artifact/evidence existenceを検査する。
- Reviewは「強みを維持する判断」と「変える判断」を両方記録する。機能数を増やすこと自体を改善とみなさない。

PriorityとConfidenceは[review-template.md](review-template.md#rating-rubric)のrubricに従う。P0はdocument数を増やすためのラベルではなく、現在のsupported pathに直接のinvariant riskが確認できる場合へ限定する。

## Reading Order

1. [RVC-001](reviews/001-consistency-claim-boundary.md) でCodeFireが何を証明するかを確認する。
2. RVC-011、RVC-019、RVC-023、RVC-024でconsistencyとsealの中核を読む。
3. RVC-031、RVC-033、RVC-034、RVC-040でCLI/machine contractを読む。
4. RVC-041、RVC-045、RVC-046、RVC-050でtrust boundaryを読む。
5. [RVC-060](reviews/060-integrated-improvement-roadmap.md) でroot-cause clusterと昇格順序を確認する。

## Portfolio Judgment

このbaselineでP0に残したのは12件である。件数ではなく、次の3つの直接riskへdeduplicateする。

| Direct risk | RVC | Judgment |
|---|---|---|
| Fire/Resolution、object、seal、publication、lock、open state、batch recovery | 012, 021, 023, 024, 026, 027, 037 | CFB-120〜126で導入したvalidated readとmulti-record publication/recoveryをDogfoodし、同じprotocolとして次候補へ再利用する。 |
| authenticated remote mutation、nonce replay、idempotency、path use boundary | 041, 043, 044, 045 | body-bound request subject、fail-closed replay store、lock内idempotency、use-time path validationを別々のsecurity featureに分断しない。 |
| unknown persisted schemaのfail-closed解釈 | 057 | public command schemaのRVC-033と分担しつつ、一つのversion-governance decisionへ統合する。 |

P0の[RVC-011](reviews/011-single-committable-predicate.md)はCFR-161/ADR-013として完了し、全surface driftを防ぐsemantic rootになった。architecture/test分割やUX/adoption候補は、このpredicate、publication、subject、versionのcounterexampleとcontractを支える範囲に限定する。RVC-060は補助cluster map、terminal dispositionは`review-dispositions.json`、実装順はcurrent cycle planを正とする。

## Terminal Audit Result

| Disposition | Count | Meaning |
|---|---:|---|
| `implemented` | 6 | current implementation、regression、closed issue/ADRがacceptanceを満たす |
| `accepted_non_goal` | 3 | current product claim/roadmapで実装taskにしない境界を明示した |
| `promoted` | 51 | residual acceptanceをCFR-163..173の一つへexactly once mappingした |
| Total | 60 | unchecked/candidate reviewは0 |

## A. Product and System Purpose

| ID | Review |
|---|---|
| RVC-001 | [CodeFireのconsistency claim境界](reviews/001-consistency-claim-boundary.md) |
| RVC-002 | [Git非互換性とadoption bridge](reviews/002-git-noncompatibility-adoption-bridge.md) |
| RVC-003 | [sealed historyとworking progress](reviews/003-sealed-history-working-progress.md) |
| RVC-004 | [Fireのactionabilityとalert fatigue](reviews/004-fire-actionability-alert-fatigue.md) |
| RVC-005 | [evidenceのtrust levelとprovenance](reviews/005-evidence-trust-provenance.md) |
| RVC-006 | [Atom粒度とextractor semantic stability](reviews/006-atom-granularity-extractor-stability.md) |
| RVC-007 | [Trace Link ontologyの進化](reviews/007-trace-link-ontology-evolution.md) |
| RVC-008 | [policy authorshipとexplainability](reviews/008-policy-authorship-explainability.md) |
| RVC-009 | [branch/open directoryのmental model](reviews/009-branch-open-directory-mental-model.md) |
| RVC-010 | [product successとfalsification metrics](reviews/010-product-success-falsification-metrics.md) |

## B. Core Consistency Domain

| ID | Review |
|---|---|
| RVC-011 | [single committable predicate](reviews/011-single-committable-predicate.md) |
| RVC-012 | [Fire lifecycle state machine](reviews/012-fire-lifecycle-state-machine.md) |
| RVC-013 | [Resolution freshnessとinvalidation](reviews/013-resolution-freshness-invalidation.md) |
| RVC-014 | [Trace Graph closureとcycle](reviews/014-trace-graph-closure-cycles.md) |
| RVC-015 | [required-link evaluationの計算量](reviews/015-required-link-evaluation-complexity.md) |
| RVC-016 | [non-Atom change semantics](reviews/016-non-atom-change-semantics.md) |
| RVC-017 | [duplicate Atomのrepair experience](reviews/017-duplicate-atom-repair-experience.md) |
| RVC-018 | [verification command trust contract](reviews/018-verification-command-trust-contract.md) |
| RVC-019 | [consistency certificateのproof境界](reviews/019-consistency-certificate-proof-boundary.md) |
| RVC-020 | [core domainのpurityとdeterminism](reviews/020-core-domain-purity-determinism.md) |

## C. Store, Repository, and Transactions

| ID | Review |
|---|---|
| RVC-021 | [object identityとfull digest](reviews/021-object-identity-full-digest.md) |
| RVC-022 | [canonical JSONとversion negotiation](reviews/022-canonical-json-version-negotiation.md) |
| RVC-023 | [sealed commit validation boundary](reviews/023-sealed-commit-validation-boundaries.md) |
| RVC-024 | [commit transactionのcrash consistency](reviews/024-commit-transaction-crash-consistency.md) |
| RVC-025 | [branch headのatomic publication](reviews/025-branch-head-atomic-publication.md) |
| RVC-026 | [repository lock model](reviews/026-repository-lock-model.md) |
| RVC-027 | [open state identity convergence](reviews/027-open-state-identity-convergence.md) |
| RVC-028 | [recovery/doctor state machine](reviews/028-recovery-doctor-state-machine.md) |
| RVC-029 | [storage scalingとcache integrity](reviews/029-storage-scaling-cache-integrity.md) |
| RVC-030 | [garbage collectionとreachability](reviews/030-garbage-collection-reachability.md) |

## D. CLI, Automation, and User Workflow

| ID | Review |
|---|---|
| RVC-031 | [command lifecycle architecture](reviews/031-command-lifecycle-architecture.md) |
| RVC-032 | [capability registry contract](reviews/032-capability-registry-contract.md) |
| RVC-033 | [JSON Schema compatibility family](reviews/033-json-schema-compatibility-family.md) |
| RVC-034 | [exit/diagnostic/blocking semantics](reviews/034-exit-diagnostic-blocking-semantics.md) |
| RVC-035 | [next_actionsのsafe executability](reviews/035-next-actions-safe-executability.md) |
| RVC-036 | [dry-run fidelity](reviews/036-dry-run-fidelity.md) |
| RVC-037 | [batch atomicityとpartial failure](reviews/037-batch-atomicity-partial-failure.md) |
| RVC-038 | [context/explain/review-packのcoherence](reviews/038-context-explain-review-pack-coherence.md) |
| RVC-039 | [diff/merge semantic review](reviews/039-diff-merge-semantic-review.md) |
| RVC-040 | [CLI workflow discoverability](reviews/040-cli-workflow-discoverability.md) |

## E. Remote, Security, and Scale

| ID | Review |
|---|---|
| RVC-041 | [remote mutation trust boundary](reviews/041-remote-mutation-trust-boundary.md) |
| RVC-042 | [HTTP resource governance](reviews/042-http-resource-governance.md) |
| RVC-043 | [signature/key/nonce lifecycle](reviews/043-signature-key-nonce-lifecycle.md) |
| RVC-044 | [idempotency semantic contract](reviews/044-idempotency-semantic-contract.md) |
| RVC-045 | [path/symlink use boundary](reviews/045-path-symlink-use-boundary.md) |
| RVC-046 | [evidence commandとsecret boundary](reviews/046-evidence-command-secret-boundary.md) |
| RVC-047 | [large repository performance budget](reviews/047-large-repository-performance-budget.md) |
| RVC-048 | [graph/ancestry algorithm](reviews/048-graph-ancestry-algorithms.md) |
| RVC-049 | [bounded output/pagination/backpressure](reviews/049-bounded-output-pagination-backpressure.md) |
| RVC-050 | [local/server concurrency model](reviews/050-concurrency-local-server-model.md) |

## F. Codebase Evolution and Verification

| ID | Review |
|---|---|
| RVC-051 | [`main.rs` responsibility decomposition](reviews/051-main-rs-responsibility-decomposition.md) |
| RVC-052 | [crate boundaryとdomain leakage](reviews/052-crate-boundary-domain-leakage.md) |
| RVC-053 | [Python fallback compatibility contract](reviews/053-python-fallback-compatibility-contract.md) |
| RVC-054 | [test architecture decomposition](reviews/054-test-architecture-decomposition.md) |
| RVC-055 | [cross-runtime conformance corpus](reviews/055-cross-runtime-conformance-corpus.md) |
| RVC-056 | [fault/corruption/race testing](reviews/056-fault-corruption-race-testing.md) |
| RVC-057 | [schema/object version evolution](reviews/057-schema-object-version-evolution.md) |
| RVC-058 | [correctness/performance observability](reviews/058-correctness-performance-observability.md) |
| RVC-059 | [extension boundaryとscope control](reviews/059-extension-boundary-scope-control.md) |
| RVC-060 | [統合改善ロードマップ](reviews/060-integrated-improvement-roadmap.md) |

## Terminal Disposition Workflow

1. 完了済みStage 1/CFR-161/CFR-162のRetrospect証跡を確認した。
2. 60件をexisting issue/ADR/current sourceへ再照合し、同一root causeでdeduplicateした。
3. current gapはformal issueへ、満たしたacceptanceはimplementedへ、明示境界はaccepted non-goalへ分類した。
4. `review-dispositions.json`と各review header/checklistを同期し、project docs checkerへ組み込んだ。
5. residual 51件を11 rootへFreezeした。active batchを全件Closeするまでstatusをcandidateへ戻さず、新Discoveryを追加しない。
