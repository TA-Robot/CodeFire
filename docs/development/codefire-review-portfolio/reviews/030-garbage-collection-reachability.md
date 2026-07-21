# RVC-030: GC reachabilityをtyped root registryとretention reasonで証明する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | correctness / architecture / security / UX |
| Review kind | redesign |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 1 |
| Primary scope | `object graph roots / Python fallback remote GC / future Rust GC` |
| Promotion target | strategy experiment |

## Executive Judgment

現行Python GCが削除前にhash、missing reference、sealed ref、signatureを検査し、dry-run・retention・auditを持つことは重要な強みである。しかしroot categoryとreference traversalがPython関数に手書きされ、Rust store validatorやpending transactionのobject一覧と共有されない。削除機能を増やす前に、全rootと各retention reasonをmachine-readableに説明するtyped reachability engineを検証すべきである。

## Scope and Review Question

- In scope: branch/MR/pending transaction等のroot、typed references、retention理由、dry-run/apply同値性
- Out of scope: secure erase、既にcloneされたcopyの消去、retention日数のproduct pricing
- Question: どのreachable objectも削除せず、保持・削除候補の全objectに監査可能な理由を付けられるか

## Evidence

- `codefire` — reachable_remote_objects / remote_graph_roots: remote branch headとMRのsource/target/applied headをrootにし、object_referencesをstack traversalする。
- `codefire` — remote_gc_payload: integrity preflight後にreachable set、mtime retention、generation retentionを評価し、removed/protected objectsをauditへ書く。
- `crates/codefire-store/src/lib.rs` — validate_sealed_commit / validate_root_references: Rust側にはtyped root/reference検査があるが、汎用root registryやmark result APIとしては公開されない。
- Test/fixture evidence: `tests/test_codefire_cli.py` — test_remote_gc_removes_unreachable_objects、blocks_on_unhealthy_object_graph、respects_retention_policy/generation は主要安全性を固定するが、root category網羅と各retained objectのreason coverageは固定しない。

## Strengths to Preserve

- unhealthy graphでは削除を全面停止するfail-closed preflight。
- dry-run、time/generation retention、actor付きJSONL auditを持つこと。

## Gap / Risk

### Observed gap

root categoryとreference抽出がruntime/機能ごとに独立し、pending commit markerが列挙する未publish objectなど、将来local GCで必要なrootを自動的に取り込む契約がない。protected objectのaudit itemにも「timeかgenerationか」の個別reasonが残らない。

### Failure or stagnation mode

新しいmutable recordやobject kind追加時にGC root/reference更新を忘れ、reachable workをorphan扱いする。逆に過剰保持の理由が説明できず、storage growthを安全に改善できない。

### Affected invariants

- no reachable object deletion
- pending transaction recovery
- sealed graph integrity
- retention auditability

## Improvement Direction

store/domain側に ObjectReferenceRegistry と RootProvider を定義し、mark結果を object_id → one-or-more ReachabilityReason として返す。rootにはbranch、MR、opened/base、pending transaction、policy上の保留を明示し、未知required object kind/referenceは削除禁止にする。sweepはstable snapshot/lock下でdry-run planのfingerprintを再確認してからだけ実行する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: typed root/reference registry＋reason-bearing mark | 安全性とstorage説明を同時に得る | registry更新gateとproperty testが必要 | experiment |
| B: 現行Python traversalをRustへそのまま移植 | parityは早い | root driftと理由欠落も複製する | reject |

## Recommended Sequence

1. **Characterize**: 全root category/object kind/pending stateを含む生成graph oracleを作る。
2. **Separate**: integrity validation、mark/reason、retention、sweep/auditを別phaseにする。
3. **Implement**: read-only reachability reportをRust storage/doctorから先に試す。
4. **Prove**: oracle/property testでfalse deletion 0とreason coverage 100%を要求する。
5. **Promote**: 実delete commandはStage 1 gate後に別CFR/product decisionとして昇格する。

## Acceptance Signals

- generated graphでreachable object誤削除が0、全retained objectに少なくとも1reasonがある。
- unknown root/reference/schemaはdelete候補ではなくblocking unknownになる。
- stable snapshotではdry-run候補集合とapply削除集合が一致し、変化時は再planを要求する。

## Verification Design

- Focused tests: root provider、reference registry、cycle、missing/unknown reference、reason composition。
- Cross-boundary tests: Python GC fixtureとRust read-only mark reportのparity。
- Scale or fault tests: shared DAG、pending commit各phase、plan/apply間head mutation、audit write failure。
- Observability: retained/deletable bytes、reason別count、orphan age/generation、work visitsを出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-069はidempotency retention visibilityをfixed済み。既存CFB/CFRにtyped root registryとreason coverageの直接issueはない。Python fallback GCの単純portとは区別する。
- Required predecessor: RVC-024のpending transaction state、RVC-021/023のidentity/sealed boundary。
- Can combine with: RVC-028、RVC-029、RVC-048の共通bounded graph engine。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- content-addressed copyの物理的・法的完全消去を保証しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
