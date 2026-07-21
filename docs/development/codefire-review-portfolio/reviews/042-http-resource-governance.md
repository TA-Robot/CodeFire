# RVC-042: HTTP resource governance

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | security / performance / architecture |
| Review kind | refine |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `codefire-cli::http` |
| Promotion target | future CFR / ADR |

## Executive Judgment

HTTP層はtimeout、header/body上限、接続数上限を既に持ち、過去の無制限状態から大きく改善している。この強みを統一resource policyへ引き上げ、request受信だけでなくresponse生成、object graph展開、thread/IO/memory、cleanupまで同じ予算で統治すべきである。

## Scope and Review Question

- In scope: client/serverのbytes、時間、接続、object数、temporary storage、応答生成の資源境界
- Out of scope: Web framework移行、公開サービスSLA、TLS鍵運用
- Question: 悪意ある/巨大/遅いrequestが、受信上限を迂回してCPU・memory・disk・threadを占有しないか

## Evidence

- `crates/codefire-cli/src/http.rs` — `MAX_HTTP_BODY_BYTES` / `http_timeout` / `read_http_request` / `read_http_response`: 16MiB body、64KiB header、正のtimeoutを検査する。
- `crates/codefire-cli/src/http.rs` — `ServerConnectionLimiter` / `serve_http`: 同時接続を32に制限し、connectionごとにthreadを起動する。
- `crates/codefire-cli/src/http.rs` — `handle_http_bundle` / `write_http_json`: reachable object全件をValueへ集め、response全体を`Vec<u8>`へserializeしてから送るが、server側response生成上限はない。
- Test/fixture evidence: `http_request_rejects_oversized_body_before_reading_body`、`http_response_rejects_oversized_body_before_reading_body`、`server_connection_limiter_caps_and_releases_active_connections` は個別上限を固定するが、複合resource exhaustionは未検証。

## Strengths to Preserve

- 不正timeout値を黙って無視せずusage errorにする。
- oversized/truncated framingをbody処理前に拒否し、slow connectionを全体逐次処理から分離する。

## Gap / Risk

### Observed gap

固定body/connection limitはあるが、object数、展開後bytes、response bytes、temporary copy、per-project公平性、全thread合計memoryを束ねるpolicyがない。upload requestはclientで全payloadをserializeし、bundle/list responseも全件materializeする。

### Failure or stagnation mode

32本の合法サイズrequestが同時にobject graph検証やdirectory copyを行い、body上限を守りながらmemory/disk/CPUを枯渇させる。巨大bundleはserverで生成後にclient側上限で拒否され、双方の仕事が無駄になる。

### Affected invariants

- remote availability and cleanup
- bounded automation output
- validated object graph transfer

## Improvement Direction

`HttpResourcePolicy`にheader/body/response/object/thread/temp/time予算を集約し、route plan時に早期見積り、実行中にwork counter、終了時にcleanupを適用する。上限超過はstable diagnosticとretry/partition guidanceを返す。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: route共通resource budget | 一貫したfail-fastと観測が可能 | policy型と各routeの計測が必要 | adopt |
| B: 個別constを追加し続ける | 局所修正が容易 | 複合予算・drift・説明可能性が弱い | reject |

## Recommended Sequence

1. **Characterize**: 32接続、large bundle、large upload、slow bodyの資源counter fixtureを作る。
2. **Separate**: HTTP framingとdomain routeへ共通budget/contextを渡す。
3. **Implement**: response/object/temp上限、早期413/503、確実なtemp cleanupを加える。
4. **Prove**: limit前後のdeterministic counterとcleanup assertionを通す。
5. **Promote**: remote protocol spec/ADRと必要なCFRへ昇格する。

## Acceptance Signals

- 全routeがrequest/response/object/time予算を機械可読に報告し、超過時にpartial mutationを残さない。
- N並列時のmemory/temp使用がpolicyから導ける上限内に収まる。
- 現行のtimeout、framing、sealed validation testsが退行しない。

## Verification Design

- Focused tests: header/body/response/object/temp/time各limitの境界値
- Cross-boundary tests: upload/bundle/MR routeとclient error envelope
- Scale or fault tests: 32並列、slowloris、disk-full temp、serialize超過
- Observability: route、rejected phase、bytes/objects、active/queued count、cleanup結果

## Dependencies and Duplication Check

- Existing CFB/CFR: fixed CFR-030/031/032/033/111/128を再openせず、その成果を統合budgetへ一般化する候補。CFR-088/089のpayload memory問題とも関連する。
- Required predecessor: RVC-041でremote subjectを壊さないresource context境界を確認
- Can combine with: RVC-047、RVC-049、RVC-050、RVC-058

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 公開internet向け分散serverへ拡張する判断は行わない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
