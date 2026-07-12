# System Model And Formal Invariants

> **非正本 / exploration only**
>
> これはCodeFireのstate、safety、livenessを検討する研究ノートであり、製品仕様ではない。採用された結論は `docs/05_consistency_model.md`、product docs、ADR、core/store testsへpromotionする。

## 1. Scope

対象:

- open directory lifecycle
- scan/fire/extinguish/verify/commit
- branch headとsealed commit graph
- local/remote mutationの抽象状態遷移
- dry-run、idempotency、lock、pending transaction
- human/JSON viewが観測する状態

対象外:

- CLI文言の最終決定
- network protocolのwire format詳細
- model checker/toolの即時採用判断

## 2. Candidate State Model

`hypothesis`: repositoryの観測可能状態を、単一の文字列ではなく次のtupleとして扱うと矛盾を発見しやすい。

\[
S = (H, O, W, A, F, R, V, T, L, I)
\]

| Symbol | Meaning |
|---|---|
| `H` | branch headsとsealed commit graph |
| `O` | opened registryとopen marker |
| `W` | working file snapshotまたはheadとの差分 |
| `A` | derived active scan/index/trace state |
| `F` | open/obsolete/resolved Fires |
| `R` | Resolutionsとbasis |
| `V` | Verification resultとblockers |
| `T` | pending transaction markerとphase |
| `L` | lock ownership/age |
| `I` | idempotency records |

`open-clean`、`open-burning`、`open-consistent` はこのtupleから導出されるprojectionと考える。保存済みstate labelそのものをtruthにすると、branch record、opened registry、active stateの複数copyが食い違う可能性がある。

## 3. Safety And Liveness

### 3.1 Safety

「悪い状態へ到達しない」性質。

- invalid/unsealed commitをbranch headから公開しない。
- blockerを持つworkspaceをconsistent commitとして封印しない。
- crashでold headとnew rootsの不整合を正常状態として見せない。
- dry-runで永続状態を変えない。
- remote policyを迂回してmutationしない。
- stale resolutionをactiveな根拠として再利用しない。

### 3.2 Liveness

「前提が満たされれば操作がいつか完了できる」性質。

- dead processのlockでrepositoryが永久停止しない。
- corrupt branchがあっても、診断commandが無限traversalしない。
- shared historyのbranch listが規模に対して実用的に完了する。
- retry可能なfailureからidempotentに進行できる。
- pending transactionを説明または修復できる。

CodeFireはsafety優先でよいが、livenessを未定義のままにすると「安全に永久停止する」systemになる。

## 4. Commit Predicate

`documented`: 現行consistency modelは次をcommit条件としている。

\[
committable(W) =
|F_{open}|=0 \land
|C_{fail}|=0 \land
|L_{missing}|=0 \land
|R_{stale}|=0 \land
|D_{dup}|=0
\]

`observed`: core実装の`build_verification`はmissing evidence referenceも常時blockする。また、各blockerがpolicyでrequiredかどうかにより判定が変わる。

探索上のcandidate predicate:

\[
committable_v(W,P) =
sealed(base(W)) \land
predicate_P(F,C,L,R,E,D) \land
transaction\_ready(W)
\]

ここで `E` はmissing evidence references、`P` はversioned verification policyである。

### Q-FORMAL-001

commit predicateの正本は仕様、`Verification`、commit runner、certificate validatorのどこに置くべきか。

### HYP-FORMAL-001

`hypothesis`: blocker collectionとpolicy evaluationを一つのpure domain functionへ集約し、そのresultをverification、commit、certificate、JSONが参照すれば、仕様・実装差を減らせる。

反証条件:

- commandごとのcontext差が大きく、一つのpredicateが多数の例外分岐を必要とする。
- shared result化によってsealed validationとlive workspace validationが不適切に結合する。
- property testで同じinputから異なるcommand decisionが正当と示される。

測定:

- blocker組合せごとのdecision一致率。
- predicateを重複実装しているcall site数。
- certificate fieldとverification fieldの差分数。

対応実験: `EXP-FORMAL-001`。

## 5. State Derivation

### Q-FORMAL-002

branch record、opened registry、active `state.json` のstate labelは、どれをauthoritativeにするか。

### HYP-FORMAL-002

`hypothesis`: state labelをcache/read modelと位置づけ、sealed base、working delta、open Fires、Verificationからrecompute可能にすると、矛盾回復とdoctor診断が単純になる。

反証条件:

- recompute costが通常の`status` latency budgetを満たさない。
- external verificationを再実行しない限り正しいstateを導出できない。
- cache invalidationに必要なmetadataが現行formatで不足する。

測定:

- 同一repositoryに存在するstate copy数。
- fault injection後のstate disagreement数。
- cached statusとrecomputed statusのlatency差。
- repairに必要なuser判断数。

対応実験: `EXP-FORMAL-002`。

## 6. State Transitions

候補となるnormal transition:

```text
closed
  -> open-clean
  -> open-burning
  -> open-consistent
  -> committing(objects-written)
  -> committing(head-published)
  -> open-clean
  -> closed
```

normal pathだけでなく、次のexception stateをmodelへ入れる。

```text
locked-by-live-owner
locked-by-dead-owner
pending-before-head
pending-after-head
orphan-objects-present
branch-head-invalid
opened-registry-mismatch
active-state-corrupt
verification-stale
remote-head-raced
```

### HYP-FORMAL-003

`hypothesis`: 上記状態とoperationを小さなexecutable modelで列挙すれば、hand-written integration testだけでは見つけにくいwrite ordering、retry、repairのcounterexampleを発見できる。

反証条件:

- abstractionが実装のfilesystem semanticsを捨てすぎ、発見したcounterexampleがほぼ偽陽性になる。
- model維持costがstate-changing featureの開発costを恒常的に上回る。

測定:

- modelが発見した実装bug/仕様欠落数。
- model operationと実commandのtrace一致率。
- model更新時間。

対応実験: `EXP-FORMAL-003`。

## 7. Candidate Invariants

| ID | Invariant | Evidence status | Main proof obligation |
|---|---|---|---|
| `INV-FORMAL-001` | branch headは完全検証済みsealed commitだけを指す | `documented`, partly tested | 全head writerと全head reader |
| `INV-FORMAL-002` | blocking predicateがtrueならcommit publish不可 | `documented`, predicate alignment未証明 | 仕様/core/runner/certificate一致 |
| `INV-FORMAL-003` | dry-runのpersistent write setは空 | `documented`, command別test | idempotency/lock/audit/temp fileを含む全writer |
| `INV-FORMAL-004` | resolution basisが変われば再利用不可 | `documented`, extractor evolution未定 | atom/link/policy/extractorのidentity |
| `INV-FORMAL-005` | failed mutation後に正常readがpartial stateを正常扱いしない | `inferred` | crash point全列挙 |
| `INV-FORMAL-006` | read-only commandはpersistent stateを変えない | `documented`, global coverage未証明 | status/list/show/diff/doctor/storage/context |
| `INV-FORMAL-007` | same input/version/policyから同じplanとdiagnostic orderが得られる | `hypothesis` | time/env/filesystem順序の隔離 |
| `INV-FORMAL-008` | stale/dead lockは安全な手続きで診断・回復可能 | `decision-candidate` | owner identity、PID reuse、timeout、manual confirmation |
| `INV-FORMAL-009` | corrupt/cyclic/deep graphでも全readは有限時間・有限memoryで終了する | `decision-candidate` | traversal limitとvisited set |

## 8. Formalization Options

### Option A: Rust reference state model

- production型とは分離したpure model。
- property-based operation sequenceを生成しやすい。
- CIへ入れやすい。
- filesystem/crash semanticsは別のfault testが必要。

### Option B: TLA+ or PlusCal

- concurrency、linearization、liveness、retryの表現に強い。
- 新toolingと学習costがある。
- modelとimplementationのdriftを自動検出しにくい。

### Option C: Alloy

- small-scopeでobject/commit/branch relationのcounterexampleを探しやすい。
- temporal filesystem operationには追加設計が必要。

### Option D: 手書きtransition tableのみ

- 導入costは低い。
- interleavingとstate explosionを人手で見落としやすい。

`decision-candidate`: 最初はOption Aとcrash failpointを実施し、concurrency contractが複雑化した時点でOption Bを再評価する。

## 9. Risks

| ID | Risk | Failure impact |
|---|---|---|
| `RISK-FORMAL-001` | 仕様とblocker実装の差 | 不整合commitまたは正当commitの拒否 |
| `RISK-FORMAL-002` | redundant state labelsの矛盾 | status誤表示、誤った次操作、repair困難 |
| `RISK-FORMAL-003` | safetyだけを重視しliveness未定義 | stale lock、無限/長時間traversalによる停止 |
| `RISK-FORMAL-004` | external verificationの非決定性 | certificateの再現不能、同一sourceの結果差 |
| `RISK-FORMAL-005` | modelが実装からdrift | 偽の安心感 |

## 10. Promotion Candidates

| Candidate | Promotion先 | 必要evidence |
|---|---|---|
| `DC-FORMAL-001` versioned commit predicate | `docs/05_consistency_model.md`, core API, certificate docs | `EXP-FORMAL-001` |
| derived state reducer | ADR、state module、source maps | `EXP-FORMAL-002`とlatency data |
| executable state model | test strategy、CI docs | `EXP-FORMAL-003`の有効counterexample |
| stale lock liveness contract | runbook、doctor/lock ADR | crash/recovery experiments |

## 11. Promotion Status

- 現時点でpromotion済みの項目: なし。
- owner候補: core/store maintainer、CLI state/recovery maintainer。
- review condition: P0 experiment完了または関連CFB発生時。
