# RVC-049: Bounded output, pagination, and backpressure

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | performance / UX / evolution |
| Review kind | redesign |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 2 |
| Primary scope | `automation JSON / remote HTTP read surfaces` |
| Promotion target | product spec / future CFR |

## Executive Judgment

scan、context、batch、next_actionsがsample limitとomitted countを返す設計は優れている。ただし各commandのbounded表現が個別で、全件取得は`--full`や巨大単一HTTP responseに戻り、stable pagination/cursor/backpressure contractがないため、Stage 2のpublic machine contractとして統一すべきである。

## Scope and Review Question

- In scope: scan/context/plan/list/bundle/storage/diffのcollection output、cursor、omission、streaming pressure
- Out of scope: terminal pager、network protocol全面置換、無制限debug dumpの保証
- Question: defaultをboundedに保ちながら、consumerが漏れなく決定的に続きを取得できるか

## Evidence

- `crates/codefire-cli/src/automation.rs` — `MAX_SCAN_JSON_SAMPLE` / `bounded_next_actions`: sample、omitted count/kind、recovery commandを返す。
- `crates/codefire-cli/src/context.rs` — `context_data_json` / `OmissionCounts`: total/returned/omittedとendpoint omissionを返すが、続きを指すcursorはない。
- `crates/codefire-cli/src/fire.rs` — `fire_operation_plan`: default sampleと`full_output`を切り替え、fullは全itemを一度にmaterializeする。
- `crates/codefire-cli/src/http.rs` — `handle_http_request_list` / `handle_http_bundle` / `write_http_json`: branch/MR/object graphを全件Value/bytesへ構築し単一responseにする。
- Test/fixture evidence: bounded next-actions/context/scanのomission testsは豊富だが、page連結の完全性・重複なし・snapshot drift fixtureはない。

## Strengths to Preserve

- default responseがsummary/sample中心で、truncationを隠さずtotal/omittedを明示する。
- next actionが追加取得方法を機械可読に返す。

## Gap / Risk

### Observed gap

limit field名・omission shape・full-output escape hatchがmoduleごとに異なり、remote list/bundleにはlimit自体がない。producerはcollection/JSON全体をmemoryへ載せ、consumer速度へのbackpressureもない。

### Failure or stagnation mode

consumerが省略分を確実に取得できず、`--full`でmemory/log/token上限を超える。pagination追加をcommand別に行うとcursor安定性とschemaが分裂する。

### Affected invariants

- automation contract completeness
- omission truthfulness
- bounded memory and remote availability

## Improvement Direction

共通`Page<T> {items, page_size, returned, total?, omitted?, next_cursor, snapshot}`をversioned schemaとして定義する。cursorはsort keyとimmutable subject/headを束縛し、mutable collectionのdriftは明示conflict/restartにする。大payload routeはincremental writerまたはchunk protocolを実験する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 共通page/cursor schema | consumerが漏れなく取得できる | Stage 2 schema・snapshot設計が必要 | adopt |
| B: command別limit/fullを維持 | 互換性維持が容易 | complete retrievalとbackpressureを解決しない | reject as final |

## Recommended Sequence

1. **Characterize**: collection surfaceと現在のsort/snapshot/omission shapeをinventory化する。
2. **Separate**: shared page metadataとcursor codecをautomation boundaryへ置く。
3. **Implement**: read-only list/contextから導入し、bundle/large plansを後続実験する。
4. **Prove**: pages結合=full golden、重複/欠落0、mutation drift明示を確認する。
5. **Promote**: public JSON Schema、capability、CLI specへ昇格する。

## Acceptance Signals

- 全bounded collectionがreturned/omittedまたはnext_cursorを同じ語彙で返す。
- cursor traversalは固定snapshotで全件を重複・欠落なく再構成できる。
- default/full/pageでdomain resultとcorruption diagnosticsが一致する。

## Verification Design

- Focused tests: 0/1/N/N+1、invalid/stale cursor、stable ordering
- Cross-boundary tests: CLI envelope、HTTP list/bundle、independent minimal consumer
- Scale or fault tests: million-entry synthetic list、slow writer/client disconnect、mid-page mutation
- Observability: page bytes/items、cursor restart、producer peak buffer、omitted reason

## Dependencies and Duplication Check

- Existing CFB/CFR: fixed CFB-050/051/053/073/075、CFR-072/076/088/101/135/136/145/157のbounded各論を再openせず、取得継続contractを追加する上位候補。
- Required predecessor: Stage 2 schema/version方針とRVC-047 resource budget
- Can combine with: RVC-042、RVC-047、RVC-057、RVC-059

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 全commandをstreaming protocolへ一括置換しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
