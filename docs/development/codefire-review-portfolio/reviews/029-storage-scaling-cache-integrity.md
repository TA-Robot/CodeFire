# RVC-029: storage scalingをintegrity-preserving cacheで改善する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | performance / correctness / architecture |
| Review kind | experiment |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 1 |
| Primary scope | `storage report / sealed validation / .codefire/cache` |
| Promotion target | strategy experiment |

## Executive Judgment

quick/fullの保証範囲、work metrics、largest top-kのbounded保持は維持すべきである。現在はfull reportが毎回全object JSONを読み、一般のsealed graph validationも再読する一方、cache directoryはあるがproduct cache contractは未実装である。破損診断を弱めないkeyとinvalidationを先に実験し、wall timeだけでなくobject read/parse/hash回数で採否を決めるべきである。

## Scope and Review Question

- In scope: object inventory、typed metadata、sealed validation memo、cache key/invalidation、work counters
- Out of scope: remote CDN、OS page cache tuning、無検証のmtime-only trust
- Question: repeated status/storage/branch operationsを高速化しても、tamper・missing reference・schema changeの診断が非cache時と同値か

## Evidence

- `crates/codefire-cli/src/storage.rs` — scan_objects: full modeはcollectした全fileをread_jsonしObjectRecordへdeserializeし、quick modeも全file metadataを走査する。
- `crates/codefire-cli/src/storage.rs` — ObjectScanMetrics / storage_report_data_json: files_seen、bytes_seen、json_records_parsed、invalid_recordsを既に観測可能にする。
- `crates/codefire-cli/src/metrics.rs` — cache_unimplemented / attach_metrics: cacheはunavailableとして明示され、偽のhit/miss値を成功契約へ出さない。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs` — storage report testsとmetrics cache_status testsはquick/full coverageを固定するが、warm-cache work amplificationやtamper invalidation fixtureはない。

## Strengths to Preserve

- quick modeが「何を検証していないか」をcoverage/skipped_checksで正直に返すこと。
- largest objectをtop-kにtruncateし、scan metricsをpayloadへ含めること。

## Gap / Risk

### Observed gap

immutable objectの内容・型・reference検査結果を再利用するindexがなく、commandごとにread/parse/hashが繰り返される。一方、安易なcacheは外部tamperやformat support変更を隠す。

### Failure or stagnation mode

repository規模に比例した同じ仕事がbranch list、doctor、storage、showで反復されUXが劣化する。性能fixがmtime/sizeだけを信頼すると、内容差替えをclean cache hitとして見逃す。

### Affected invariants

- object integrity diagnostics
- sealed commit validation
- bounded performance
- cache transparency

## Improvement Direction

まずappend-only ObjectInventoryを実験し、keyをobject ID、full digest、record size、validator/profile version、必要ならfilesystem identityへ結ぶ。cacheはauthorityではなく再検証可能なacceleration artifactとし、不確実なchange tokenしか得られない環境ではhash再確認またはcache bypassを選ぶ。shared parent graphはvalidated subgraph memoで一command内の重複visitを削減する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: integrity-bound inventory＋command-scoped memo | 診断同値性を保ちworkを削減 | key/invalidationとplatform評価が必要 | experiment |
| B: path+mtime+sizeだけのpersistent cache | 実装が容易 | 同size/mtime tamperを隠し得る | reject |

## Recommended Sequence

1. **Characterize**: object数・shared history幅を変え、read/parse/hash/visit回数をbaseline化する。
2. **Separate**: inventory metadata、validation result、command-scoped graph memoを別層にする。
3. **Implement**: read-only storage reportでprototypeし、cache miss時の結果をauthorityにする。
4. **Prove**: success/corruption diagnostic equivalenceとcomplexity slopeを比較する。
5. **Promote**: accepted key/invalidationだけをADR/CFRへ昇格し、効果が小さければ削除する。

## Acceptance Signals

- warm runのobject parse/hash回数がaccepted budgetまで減り、report JSONはcache metadata以外同値である。
- payload、filename、hash、reference、validator versionの変化で必ずmiss/revalidationになる。
- cache削除・破損時は安全にcold pathへ戻り、repositoryをinvalid化しない。

## Verification Design

- Focused tests: key生成、validator version bump、cache corruption、cold fallback。
- Cross-boundary tests: storage/doctor/branch listで同じobject error kindがcache有無で一致する。
- Scale or fault tests: 10²〜10⁵ objects、shared DAG、same-size tamper、途中cache write crash。
- Observability: hit/miss/bypass/revalidated、object reads、hash bytes、graph visitsを出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-051/055/105/135は全走査の可視化・quick mode・top-kをfixed済み。CFR-062/CFB-031は未実装cacheを偽装しない表示をfixed済み。本件は実cacheの安全性実験で非重複。
- Required predecessor: RVC-021/022のidentity/profile、RVC-023のvalidated boundary。
- Can combine with: RVC-030、RVC-047/048のgraph work counter・scale review、RVC-058のcorrectness-aware observability。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- cache hitをsealed validityの新しいauthorityにしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
