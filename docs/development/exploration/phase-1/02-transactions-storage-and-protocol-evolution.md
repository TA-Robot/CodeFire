# Transactions, Storage, And Protocol Evolution

> **非正本 / exploration only**
>
> この文書はlocal/remote transaction、content-addressed storage、schema/protocol evolutionの仮説集である。採用内容は `docs/05_consistency_model.md`、`docs/08_object_store.md`、`docs/10_merge_remote_server.md`、ADR、migration/compatibility testsへpromotionする。

## 1. Current Shape

`documented`:

- object store recordはimmutableで、object identityは `sha256(type_tag || 0x00 || canonical_payload)` に基づく。
- public object IDは現在digestの先頭24 hex、legacyは先頭12 hexを読む。
- objectはtemp fileへwrite/fsyncし、publish後にdirectory syncを試みる。
- commitは複数root objectを書いた後にbranch headを進める。
- commit中はactive directoryにpending markerを置き、途中failureはdoctorが報告する。
- remote upload/applyはsealed graph validation、policy、fast-forward条件を通す。

`observed`:

- mutable JSONはtemp file、file sync、rename、best-effort directory syncでpublishされる。
- commit成功にはobject、branch、opened registry、active state、transaction markerの複数pathが関わる。
- object pathが既に存在する場合、既存record自体を検証してreuseする。
- branch listなど複数head readerはheadごとにsealed graph validationを開始する。
- idempotency recordはcommand familyごとに別module/shapeを持つ。

## 2. Transaction Contract

### 2.1 Candidate phases

```text
P0 resolve and parse
P1 validate unlocked snapshot
P2 build dry-run-equivalent plan
P3 acquire lock
P4 revalidate mutable preconditions
P5 publish immutable objects
P6 publish mutable visibility pointer (branch head / MR state)
P7 update derived/opened/active metadata
P8 persist idempotency result/audit
P9 remove pending marker and release lock
```

### Q-TXN-001

各mutationのlinearization point、つまり他readerから「operationが起きた」と見える一瞬はどこか。

### HYP-TXN-001

`hypothesis`: commitではbranch head、remote mutationではbranch/MR state recordのatomic publishを唯一のlinearization pointと定義すれば、head前のorphan immutable objectsを許容しながらpartial commitの可視化を防げる。

反証条件:

- branch head公開前に他metadataが外部contractとして可視でなければならないoperationがある。
- head公開後にopened/active updateへ失敗すると、通常readerがnew headを安全に扱えない。
- filesystemごとにatomic publishの前提を満たせない。

指標:

- crash pointごとのobservable valid state率。
- old/newどちらにも分類できないpartial state数。
- orphan object数とbytes。
- recovery time、manual decision数。

対応実験: `EXP-TXN-001`。

### Q-TXN-002

idempotencyの契約はat-most-once、effectively-once、same-result replayのどれか。

### HYP-TXN-002

`hypothesis`: `(operation, scope, idempotency key, canonical payload hash)` をidentityにし、mutation resultをvisibility publishと同じtransaction protocolへ含めると、timeout後retryを安全にできる。

反証条件:

- result recordのatomic inclusionがfilesystem layout上実現できない。
- result planにsensitive dataまたは巨大payloadを保存する必要が生じる。
- commandごとのsemantic differenceが共通contractを破る。

指標:

- mutation済み/result未記録のcrash window数。
- retry後のduplicate effect数。
- same key/different payload conflict detection率。
- idempotency storage bytes。

対応実験: `EXP-TXN-002`。

## 3. Concurrency And Remote Consistency

### Candidate consistency level

`decision-candidate`: 一つのrepository/remote project内のmutable metadata mutationをsingle-writer serializableとして扱い、次を約束する。

- successful mutationの直後のsame-project readはnew headを見る。
- branch headは祖先関係を後退しない。ただし明示的な将来のadministrative recoveryは別protocolとする。
- concurrent writersの一方はlock/revalidation/fast-forwardで失敗する。
- object uploadは重複してもimmutable/idempotentである。

### Q-TXN-003

file-backed、HTTP、将来のmulti-node remoteで同じconsistency contractを維持できるか。

### HYP-TXN-003

`hypothesis`: transportとstorage backendを分離し、mutation contractをcompare-and-swap相当のexpected headで表現すれば、file-backedから他backendへ拡張してもfast-forward invariantを保持できる。

反証条件:

- current protocolがimplicit current headに依存し、expected versionを表現できない。
- MR review/applyの複数record更新がsingle CASでは表せない。
- nonce/idempotency/auditのconsistency要求がbranch headより強い。

指標:

- concurrent update lost-write数。
- stale request detection率。
- transport/backend間contract test一致率。
- lock hold durationとcontention率。

対応実験: `EXP-TXN-003`。

## 4. Crash Consistency

| Crash point | Expected durable facts | Allowed leftovers | Forbidden observation |
|---|---|---|---|
| before marker | old head/state | temp fileなし | new head |
| after marker, before objects | old head + pending marker | temp file | new head |
| after some objects | old head + pending marker | orphan immutable objects | partial rootsを指すhead |
| after all objects, before head | old head + pending marker | complete unreachable graph | new commitをnormal head扱い |
| after head, before active reset | new head + pending marker | stale active metadata | new headをinvalidと誤診断 |
| after active reset, before marker removal | new head + pending marker | redundant recovery marker | old headへ自動rollback |
| after marker removal | new head + clean active state | unreachable preexisting objects | pending transaction |

### Candidate invariants

| ID | Invariant |
|---|---|
| `INV-TXN-001` | 全referenced rootがdurableで検証可能になる前にheadをpublishしない |
| `INV-TXN-002` | lock取得後にhead/open marker/policy/idempotency preconditionを再検証する |
| `INV-TXN-003` | crash後のnormal readはpartial stateを正常成功として返さない |
| `INV-TXN-004` | pending markerはoperation ID、phase、old/new identity、written objectsを説明できる |
| `INV-TXN-005` | GCはpending transactionが参照するobjectを削除しない |
| `INV-TXN-006` | same key/same payload retryはsame semantic result、different payloadはconflict |

## 5. Content-Addressed Object Identity

### Q-STORE-001

canonical JSONはRust/Python/将来実装間でどのbyte sequenceを保証するか。

論点:

- object key order
- whitespace
- UTF-8 escapeとUnicode normalization
- integer/float表現
- negative zero、large integer、future numeric types
- absent fieldとexplicit null
- unknown field
- schema versionをidentityへ含める位置

### HYP-STORE-001

`hypothesis`: canonicalization corpusをlanguage-independentなinput JSON、expected canonical bytes、full digest、public IDの組としてversion管理すれば、implementation migrationによるidentity splitを防げる。

反証条件:

- current historical objectsの一部が単一規則で再現できない。
- canonical ruleを明文化すると既存reader compatibilityを壊す。
- JSON以外のpayload familyが必要になり共通contractが不適切になる。

指標:

- runtime/version間golden一致率。
- existing corpus再計算一致率。
- canonicalization-related migration件数。

対応実験: `EXP-STORE-001`。

### Q-STORE-002

short public ID collision時に何を保証するか。

### HYP-STORE-002

`hypothesis`: same short IDのpathが存在するとき、existing full digestとnew full digestの完全一致を必須にし、不一致ならdedicated collision errorにすべきである。

反証条件:

- current record formatからexisting full digestを信頼可能に取得できない。
- compatibility上、same short IDに複数recordを格納する必要がある。

指標:

- artificial collisionのsilent alias数。目標0。
- legacy 12 hex objectのcollision-aware read coverage。
- collision errorからのdiagnostic/repair completeness。

対応実験: `EXP-STORE-002`。

### Candidate invariants

| ID | Invariant |
|---|---|
| `INV-STORE-001` | same type/canonical payloadはsame full digest |
| `INV-STORE-002` | different full digestをsame objectとしてsilent reuseしない |
| `INV-STORE-003` | immutable object pathを上書きしない |
| `INV-STORE-004` | filename、record ID、record hash、recomputed hashが整合する |
| `INV-STORE-005` | unknown object kind/versionを黙ってtrusted graphへ入れない |

## 6. GC And Reachability

候補root set:

```text
local branch heads
remote branch heads
open directory base commits
merge request source/target commits
pending transaction written objects/new commit
retention generations/time window
explicit evidence/artifact references required by retained commits
administrative holds
```

### Q-STORE-003

idempotency result、audit、review pack、uncommitted evidenceはGC rootか、derived cacheか、期限付きholdか。

### HYP-STORE-003

`hypothesis`: root categoryとretention reasonをmachine-readableに列挙し、GC dry-runが各retained objectの一つ以上の理由を説明できれば、safe deletionとstorage理解を両立できる。

反証条件:

- explanation生成の追加traversalが大規模repoで実用不能。
- root categoryがbackendごとに大きく異なり共通schemaが形骸化する。

指標:

- reachable object誤削除0。
- objectごとのretention explanation coverage。
- orphan/reachable bytes ratio。
- GC runtime/peak memory。

対応実験: `EXP-STORE-003`。

## 7. Schema And Protocol Evolution

### Compatibility dimensions

| Dimension | Questions |
|---|---|
| object schema | older/newer objectを誰が読めるか |
| repository layout | migrate check/dry-run/apply、rollbackはどうするか |
| JSON envelope | field addition、diagnostic kind、exit code、schema ID |
| remote protocol | client/server version negotiation、capabilities |
| policy/config | limited YAML subset、unknown fields、defaults |
| extractor semantics | same sourceから生成されるAtom/selector/hash |
| Python/Rust | reference/fallbackのcompatibility surface |

### Q-PROTO-001

schema versionとcapability negotiationをどこまで明示すべきか。

### HYP-PROTO-001

`hypothesis`: reader/writer compatibility matrixとcapability advertisementを正本化すれば、unknown versionを「best effort」で誤読するより安全に段階移行できる。

反証条件:

- productが単一versionのみを明確にsupportする方が運用costを大幅に下げる。
- capability細分化がtest matrixを維持不能にする。

指標:

- supported version pairごとのcontract test coverage。
- silent downgrade/unknown field acceptance件数。
- migration failure/recovery率。

対応実験: `EXP-PROTO-001`。

### Q-SEMVER-001

Atom extractorの挙動変更はparser bug fixか、repository semantic format changeか。

### HYP-SEMVER-001

`hypothesis`: AtomIndexにextractor family/versionを記録し、resolution basisまたはverificationでsemantic version driftを検出すべきである。

反証条件:

- content hash/selector/Atom setの変化だけで必ず安全にstale化できる。
- versionを記録すると小さなbug fixでも全resolutionが不要にstaleになる。
- languageごとのversion管理が複雑すぎる。

指標:

- same sourceに対するversion間Atom差。
- semantic change後もactiveなまま残るresolution数。
- false stale resolution数。

対応実験: `EXP-SEMVER-001`。

## 8. Risks

| ID | Risk | Impact |
|---|---|---|
| `RISK-TXN-001` | mutation済みだがidempotency result未保存 | retryでduplicateまたはconflict |
| `RISK-TXN-002` | multi-file publish中断 | state disagreement、manual recovery |
| `RISK-TXN-003` | stale lock | safeだが永久に進行不能 |
| `RISK-STORE-001` | short ID collisionのsilent alias | wrong payloadを正当objectとして参照 |
| `RISK-STORE-002` | canonicalization差 | runtime間でidentity split |
| `RISK-STORE-003` | root set漏れ | GC data loss |
| `RISK-PROTO-001` | unknown versionの黙認 | corrupt/incorrect interpretation |
| `RISK-SEMVER-001` | extractor changeのidentity不在 | stale判断のfalse negative |

## 9. Decision Candidates And Promotion

| Candidate | Promotion先 | Gate |
|---|---|---|
| `DC-TXN-001` head publish linearization | consistency model、transaction ADR | `EXP-TXN-001` |
| unified idempotency contract | automation/remote spec、ADR | `EXP-TXN-002` |
| expected-head CAS remote contract | remote spec、protocol schema | `EXP-TXN-003` |
| `DC-STORE-001` full-digest collision rejection | object-store spec、store tests | `EXP-STORE-002` |
| canonical corpus | schemas/fixtures、compatibility tests | `EXP-STORE-001` |
| explicit GC root reasons | storage/GC design、doctor | `EXP-STORE-003` |
| extractor semantic version | AtomIndex schema、migration ADR | `EXP-SEMVER-001` |

## 10. Promotion Status

- 現時点でpromotion済みの項目: なし。
- owner候補: store、remote、migration、core extractor maintainer。
- 再検討trigger: repository format変更、new remote backend、GC拡張、extractor behavior変更。
