# Security, Observability, And Recovery

> **非正本 / exploration only**
>
> この文書はthreat model、trust boundary、diagnostics、recoveryを一体で検討する。採用内容はsecurity/transaction ADR、remote policy、doctor、runbook、automation interface、release processへpromotionする。

## 1. Why These Belong Together

security failureとreliability failureは、operatorから見ると「何が起きたか分からず、安全に進めない」という同じ形になる。CodeFireではrepository corruption、policy violation、authentication failure、lock contention、pending transactionを区別し、次を同時に満たす必要がある。

- unsafe continuationを止める。
- failure phaseとtrust boundaryを説明する。
- secretやsensitive pathを漏らさない。
- repair可能性とsafe next actionを示す。
- diagnostic自体がresource exhaustionを起こさない。

## 2. Assets And Threat Actors

### Assets

- sealed commit/object identity
- branch head/history integrity
- working treeと未commit user work
- policy、verification、evidence
- signing/authentication keysとtokens
- remote branch/MR/review/audit state
- local/remote filesystem confidentiality
- release binaryとinstaller provenance
- automation consumerが依存するJSON/exit-code contract

### Threat actors and failure sources

| Actor/source | Capability |
|---|---|
| malicious repository | crafted JSON/YAML/object graph/path/symlink/huge input |
| malicious remote client | replay、oversized request、invalid graph、auth probing |
| malicious/compromised server | false result、object substitution、credential capture |
| untrusted contributor | hostile verification command/policy、crafted source parser input |
| local co-tenant | lock/key/file race、permission misuse |
| compromised dependency/release | arbitrary code execution、binary replacement |
| accidental operator | insecure TLS flag、wrong path、stale key、destructive recovery |
| AI-generated patch | plausible invariant bypass、secret leakage、oversized change |

## 3. Trust Boundaries

```text
CLI arguments/env
  -> parser boundary
filesystem repository/open directory
  -> path and schema boundary
object record
  -> hash/type/graph boundary
policy and verification commands
  -> code-execution boundary
remote HTTP/TLS
  -> authentication/request boundary
remote project storage
  -> authorization/atomic mutation boundary
installer/release artifact
  -> supply-chain boundary
JSON diagnostics/evidence
  -> information disclosure boundary
```

`decision-candidate`: security reviewはcommand単位ではなく、上記boundaryをcrossするdata flow単位で行う。

## 4. Path And Filesystem Safety

### Q-SEC-001

lexical normalization、canonicalization、symlink resolution、write直前revalidationをどの組合せで使うか。

### HYP-SEC-001

`hypothesis`: repository/open/remote rootに対するcapability-like path typeを作り、root-relative component validationとwriter直前のsymlink-aware containment確認を共通化すると、commandごとのpath policy差を減らせる。

反証条件:

- non-existent write targetはcanonicalizeできず、race-freeな共通方式を作れない。
- platform filesystem差により同一contractを維持できない。
- strict symlink拒否が正当なrepository利用を広く壊す。

指標:

- path writerの共通safe-path API利用率。
- traversal/symlink/case-fold corpusのescape成功数。目標0。
- TOCTOU fault test coverage。
- legitimate symlink workflow rejection数。

対応実験: `EXP-SEC-001`。

### Candidate invariants

| ID | Invariant |
|---|---|
| `INV-SEC-001` | repo-relative writerはroot外へ出ない |
| `INV-SEC-002` | branch/object/user-controlled IDをraw path componentとして使わない |
| `INV-SEC-003` | write前validationとwrite targetのidentityをrace可能な範囲で再確認する |
| `INV-SEC-004` | install prefix/completion pathのunsafe symlinkを拒否または明示扱いする |

## 5. Remote Authentication And Signatures

`documented`: CodeFireはtoken、HMAC commit signature、HMAC request signature、timestamp skew、nonce replay、key status/time/signer policyを持つ。

### Q-SEC-002

HMAC shared keyが提供するauthenticityと、public-key signature/KMSが提供するidentity/non-repudiationの差をproductとしてどう説明するか。

### HYP-SEC-002

`hypothesis`: 現行HMACをsingle-team/trusted-operator MVP境界として明示し、multi-tenant/public deploymentのpromotion gateにasymmetric key、external key management、distributed replay protectionを置くべきである。

反証条件:

- target deploymentが恒久的にsingle operator/local networkに限定される。
- external identity provider/token-only設計の方が署名機能より単純かつ安全である。

指標:

- deployment classごとのrequired control coverage。
- replay/skew/revoked key rejection率。
- shared key exposure blast radius。
- key rotation recovery time。

対応実験: `EXP-SEC-002`。

### Candidate invariants

- mutating requestはconfigured auth/signature/permission/branch protectionをすべて通る。
- serverはclient-provided verification/certificateだけを信頼しない。
- nonce replay windowを越えて同一signed requestを受理しない。
- secret key/tokenをobject、audit、idempotency、diagnosticへ保存しない。
- `CODEFIRE_TLS_INSECURE=1` は明示warningを出し、production evidenceとして扱わない。

## 6. Server-Side Verification As Code Execution

`documented`: verification commandはremote project root配下のcwd、限定env、timeout、output capを持つが、OS-level sandbox/container isolationはない。

### Q-SEC-003

remote repository contributorがverification policy/sourceを変更できる場合、server operator権限でどこまで実行されるか。

### HYP-SEC-003

`hypothesis`: sandboxなしserver verificationをtrusted policy/admin-controlled commandに限定し、untrusted public MR executionはsupport外またはsandbox必須と明記しなければならない。

反証条件:

- verification policyがserver operatorだけにより完全固定され、project contentからcommandが変更できない。
- server process自体が使い捨ての強隔離environmentで常に実行される。

指標:

- command source/ownerの明示率。
- filesystem/network/process/secret access test結果。
- timeout/output/process-tree cleanup成功率。
- supported deploymentごとのisolation evidence。

対応実験: `EXP-SEC-003`。

## 7. Resource Exhaustion

攻撃/事故入力:

- huge HTTP body、JSON、diagnostics、verification output
- very deep/wide commit/object/Trace graph
- many branches sharing history
- many files/Atoms/Fires
- parser pathological input
- slow/never-ending verification commandとchild process
- repeated failed auth/nonce requests

### HYP-SEC-004

`hypothesis`: input bytes、entity counts、depth、output bytes、wall time、memoryの独立budgetをboundaryごとに定義すると、単一timeoutだけよりDoS耐性を説明できる。

反証条件:

- hard limitがlegitimate large repoを不必要に拒否する。
- memoryをportableに測れず、契約化できない。

指標:

- budget別reject coverage。
- limit到達時のpeak resource。
- false reject率。
- cleanup後のorphan process/temp file数。

対応実験: `EXP-SEC-004`。

## 8. Supply Chain

### Q-SEC-004

installerが配置したRust binary、prebuilt binary、Python fallback、completion、schema/docsが同じrelease identityに属することをどう証明するか。

### HYP-SEC-005

`hypothesis`: reproducible build metadata、artifact digest/signature、SBOM、source revision、CLI `version --json`を結ぶrelease manifestが必要である。

反証条件:

- distributionがsource buildだけに限定され、prebuilt artifactを提供しない。
- target platform差でreproducible byte identityが現実的でない。ただしprovenanceはなお必要。

指標:

- release artifact signature/provenance coverage。
- source revisionを特定できるinstalled binary率。
- dependency inventory coverage。
- compromised artifact drillのdetection time。

対応実験: `EXP-SEC-005`。

## 9. Observability Model

### Candidate event shape

```json
{
  "operation_id": "...",
  "transaction_id": "...",
  "command": "commit",
  "phase": "locked_revalidate",
  "scope": {"repo_id": "redacted-or-stable-id", "branch": "main"},
  "counts": {"objects": 0, "files": 0, "fires": 0},
  "duration_ms": 0,
  "result": "ok|blocked|failed",
  "diagnostic_kinds": [],
  "sensitive_fields_omitted": true
}
```

これはwire schema案ではなく、必要なcorrelation情報の探索例である。

### Q-OBS-001

parse、validate、plan、lock、revalidate、write、renderを全commandで同じphase taxonomyにできるか。

### HYP-OBS-001

`hypothesis`: phase、operation/transaction ID、bounded work counters、diagnostic kindを共通化すると、raw contentを記録せずMTTRとperformance investigationを改善できる。

反証条件:

- instrumentationがcommandごとの自然なphaseを歪める。
- correlation ID自体がobject identityやdeterministic outputへ混入する。
- event volumeがCLI toolとして過大。

指標:

- incidentでfailure phaseを特定できる率。
- mean time to diagnosis/recovery。
- unknown/internal diagnostic比率。
- log bytes/operation。
- redaction failure数。

対応実験: `EXP-OBS-001`。

### Observability invariants

| ID | Invariant |
|---|---|
| `INV-OBS-001` | metrics/logging有無でsemantic resultとpersistent writesが変わらない |
| `INV-OBS-002` | output/logはboundedでomitted countを示す |
| `INV-OBS-003` | secret、raw key、token、unbounded command outputを記録しない |
| `INV-OBS-004` | diagnostic kind、blocking flag、exit codeが矛盾しない |
| `INV-OBS-005` | mutation resultをoperation/transaction/auditへ相関できる |

## 10. Recovery Model

recoveryを三段階へ分ける。

```text
detect: corruption/pending/stale/mismatchを見つける
explain: affected state、safe/unsafe、likely phaseを示す
repair: explicit plan、backup、precondition、dry-runを伴って変更する
```

`decision-candidate`: doctorはdefault read-onlyを維持し、将来repairが必要ならseparate explicit subcommand/planとする。

### Q-REC-001

pending commitはresume、finalize、rollback、orphan cleanupのどれを許すか。

### Q-REC-002

stale lockをPID、timestamp、process start identityなしで安全に自動削除できるか。

### HYP-REC-001

`hypothesis`: recovery decision tableとfailure cookbookをpending phase別に定義すると、doctorが単に問題を報告するだけでなくsafe next actionを生成できる。

反証条件:

- marker情報だけではold/new stateを一意に判断できない。
- platform/process identity差によりstale lockを安全に判定できない。
- automatic recommendationがuser work loss riskを持つ。

指標:

- injected failureからcorrect next actionを選べる率。
- data-loss/unsafe repair数。目標0。
- manual forensic steps数。
- RTO、orphan cleanup成功率。

対応実験: `EXP-REC-001`。

### Recovery invariants

- repair前にaffected files/objectsとbackup planを示す。
- ambiguous stateで自動削除/rollbackしない。
- reachable immutable objectをrepairで消さない。
- user working filesをdefaultで変更しない。
- repair後にfull relevant validationを要求する。

## 11. Risks

| ID | Risk | Impact |
|---|---|---|
| `RISK-SEC-001` | sandboxなしverification | server compromise |
| `RISK-SEC-002` | shared HMAC key | impersonation、large blast radius |
| `RISK-SEC-003` | path/symlink race | root外read/write |
| `RISK-SEC-004` | unsigned/untraceable binary | supply-chain compromise |
| `RISK-OBS-001` | diagnostic/log secret leakage | credential/data exposure |
| `RISK-OBS-002` | unbounded observability | DoS、disk exhaustion |
| `RISK-REC-001` | ambiguous automatic repair | user work/history loss |
| `RISK-REC-002` | stale lockを回復不能 | permanent liveness failure |

## 12. Decision Candidates And Promotion

| Candidate | Promotion先 | Gate |
|---|---|---|
| safe path capability | filesystem/path ADR、shared helper | `EXP-SEC-001` |
| HMAC deployment boundary | security/remote docs | `EXP-SEC-002` |
| `DC-SEC-001` trusted verification/sandbox policy | remote spec、runbook | `EXP-SEC-003` |
| resource budgets | protocol/config/CLI docs | `EXP-SEC-004` |
| release provenance manifest | release strategy、installer | `EXP-SEC-005` |
| common phase/event taxonomy | automation/metrics docs | `EXP-OBS-001` |
| explicit recovery decision table | doctor/runbook/ADR | `EXP-REC-001` |

## 13. Promotion Status

- 現時点でpromotion済みの項目: なし。
- owner候補: security/remote、CLI path/repository、doctor/recovery、release maintainer。
- immediate escalation condition: root escape、secret leakage、untrusted command execution、reachable object lossを再現した場合は探索ではなくCFB/security responseへ移す。
