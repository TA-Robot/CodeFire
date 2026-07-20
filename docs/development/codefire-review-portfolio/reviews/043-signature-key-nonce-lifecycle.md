# RVC-043: Signature, key, and nonce lifecycle

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | security / evolution / correctness |
| Review kind | refine |
| Priority | P0 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `codefire-cli::signatures` / `server_policy.json` |
| Promotion target | ADR / future CFR |

## Executive Judgment

HMAC key ID、status、signer制約、commit有効期間、request skew、nonce replay cacheはMVPとして筋が良い。ただしcommit/requestでkey lifecycleの意味が揃わず、nonce cache上限時に有効TTL内entryを追い出すため、rotation・revocation・replay防止のfail-open/fail-closed規則を明文化して証明する必要がある。

## Scope and Review Question

- In scope: commit/request key selection、rotation、revocation、有効時刻、nonce TTL/capacity/persistence
- Out of scope: KMS製品、公開鍵algorithm選定、組織のcredential配布
- Question: 時刻境界、rotation、cache飽和、再起動を跨いでも署名policyが一意かつfail-closedか

## Evidence

- `crates/codefire-cli/src/signatures.rs` — `commit_signature_key_secret`: object形式keyではstatus/signersと`not_before`/`not_after`をsigned_atへ適用する。
- `crates/codefire-cli/src/signatures.rs` — `hmac_policy_secret`: request keyはstatus/signersを検査するが、key有効期間を評価しない。
- `crates/codefire-cli/src/signatures.rs` — `record_request_nonce`: TTL prune後、capacity超過時は最古entryを期限内でも削除するため、そのnonceがskew内に再送される余地がある。
- Test/fixture evidence: `commit_signature_verifies_key_policy_and_rotation_bounds`、`request_signature_rejects_old_timestamp_and_replayed_nonce`、`request_nonce_cache_prunes_to_policy_limit_and_reports_metadata` はあるが、capacity eviction後replayとrequest key rotationのmatrixはない。

## Strengths to Preserve

- nonce生成はOS randomness、digest比較はconstant-time、cache更新は専用lock配下でatomic writeされる。
- wall clock依存testは`sign_commit_payload_at`でdeterministicに修正済みである。

## Gap / Risk

### Observed gap

plain string keyとobject keyでpolicy能力が異なり、commit/requestでも期間評価が異なる。nonce capacityはavailabilityを優先してreplay memoryを失うが、そのsecurity trade-offがpolicy名やdiagnosticに表れない。

### Failure or stagnation mode

高頻度の正規/攻撃requestでcacheを押し出した後、まだ受理可能時刻内の署名が再利用される。rotation時にcommitは拒否、requestは許可など、同じkey recordへの期待が経路で変わる。

### Affected invariants

- request replay prevention
- authenticated remote mutation
- signed commit history policy

## Improvement Direction

共通`KeyPolicy`と明示的clockを導入し、用途別許可、validity、revocation semanticsを統一する。nonce cacheは期限内entryを静かにevictせず、capacity到達時は新規requestをfail-closedで拒否するか、署名時刻bucketを含むbounded persistent setへ移行する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 共通key policy + fail-closed nonce capacity | 意味が一貫しreplay proofが可能 | overload時に明示拒否が増える | adopt |
| B: oldest evictionを維持 | 可用性が高い | replay保証がcapacity依存で崩れる | reject |

## Recommended Sequence

1. **Characterize**: key形式/時刻/status/用途、nonce restart/TTL/capacityのdecision tableを作る。
2. **Separate**: `Clock`、`KeyPolicy`、`ReplayStore` interfaceを抽出する。
3. **Implement**: request期間評価とfail-closed capacity/diagnosticを加える。
4. **Prove**: deterministic fake clock、concurrent replay、crash/restart fixtureで確認する。
5. **Promote**: signature policy ADRとserver policy schemaへ反映する。

## Acceptance Signals

- commit/request双方でkey status・用途・期間のdecision tableが一致する。
- TTL内nonceは負荷・再起動・並行送信でも二度受理されない。
- capacity超過はsilent evictionでなく明示的なsecurity diagnosticになる。

## Verification Design

- Focused tests: before/at/after validity、revoked/disabled、plain legacy key、capacity±1
- Cross-boundary tests: file/HTTP request、storage report nonce metadata、policy parsing
- Scale or fault tests: concurrent duplicate nonce、cache write interruption、clock skew、restart
- Observability: entry/max/expired/rejected countsのみを出しsecret/nonce本体は出さない

## Dependencies and Duplication Check

- Existing CFB/CFR: fixed CFR-036/037とCFB-127の成果を保ちつつ、capacity evictionとcommit/request lifecycle差という未採番論点を扱う。
- Required predecessor: RVC-041のversioned request subject
- Can combine with: RVC-041、RVC-042、RVC-056、RVC-057

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- HMACから公開鍵へ直ちに置換する提案ではない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
