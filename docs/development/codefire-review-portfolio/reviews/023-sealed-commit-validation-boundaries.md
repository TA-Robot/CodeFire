# RVC-023: sealed commit validationを消費境界へ強制する

## Review Summary

| Field | Value |
|---|---|
| Status | implemented |
| Perspective | architecture / correctness / security |
| Review kind | redesign |
| Priority | P0 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `codefire-store validate_sealed_commit / CLI commit consumers` |
| Promotion target | existing CFB/CFR |

## Executive Judgment

validate_sealed_commit はroot type、payload shape、certificate、reachable referenceまで検査する強い実装であり、これを維持する。問題は検査能力ではなく、raw read_object と強いsealed readを呼出側が選べるため境界が抜けることである。CFB-120/121を閉じた後は、sealed commitを要求するAPIが未検証payloadを型として返せない構造へ寄せるべきである。

## Scope and Review Question

- In scope: branch head/base/parentをconsume・publishする境界、sealed validatorのAPI形
- Out of scope: commit signature policy、root schema拡張そのもの、remote transport
- Question: 「sealedであること」を要求する全pathで、callerの注意ではなくAPI構造により検証を強制できるか

## Evidence

- `crates/codefire-store/src/lib.rs` — validate_sealed_commit / validate_sealed_commit_inner: parent、required roots、root shape/reference、certificate-verification一致を再帰検証する。
- `crates/codefire-cli/src/main.rs` — compute_scan / load_base_atom_index: opened registryのcurrent_base_commitをraw object readしてAtom indexへ進み、sealed validationを先行させない。
- `crates/codefire-cli/src/main.rs` — run_commit: 新commitをstore_objectした後、validate_sealed_commitを呼ばずbranch headを書き換える。
- Test/fixture evidence: `crates/codefire-store/src/lib.rs` — validate_sealed_commit_rejects_* testsはvalidator単体を厚く固定する一方、production scan/publish callerの必須通過はCFB-120/121で未完了である。

## Strengths to Preserve

- validatorがcertificateとverificationを同じdomain truthへ結び、root参照も型付きで辿ること。
- legacy root schemaを明示的に読みつつcurrent schemaのresolution ledgerを必須化したこと。

## Gap / Risk

### Observed gap

read_object は正しいimmutable recordであることしか保証せず、commitとしてsealedである保証を返さない。CLI callerは同じString IDとValueを使うため、強い境界を省略してもcompile時に区別されない。

### Failure or stagnation mode

corruptまたはshape不正のbaseからscan/verifyが進む、あるいは新commitがpublishされた後に初めてinvalidと判明する。validatorを強化しても全consumerが通らなければ安全性の主張が増えない。

### Affected invariants

- sealed commit
- branch head validity
- scan base integrity
- certificate truth

## Improvement Direction

storeに read_sealed_commit と ValidatedCommit/ValidatedCommitId の狭いAPIを置き、branch/open/view/scanがsealed subjectを必要とするときはこの型だけを受け取る。publication APIもvalidated commitを入力に要求し、raw object readはroot traversalなど内部用途へ限定する。再帰検証コストはRVC-029の安全なmemoizationと別に最適化する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: validated wrapperをstore API境界に導入する | 抜けをcompile/reviewで見つけやすい | caller移行とlifetime/ownership設計が必要 | adopt |
| B: caller checklistと追加grep testだけにする | 小変更 | 新consumer追加時に再発する | reject |

## Recommended Sequence

1. **Characterize**: scan baseとpublish直前へmalformed-but-hash-valid commit fixtureを入れる。
2. **Separate**: raw ObjectRecord、typed commit payload、ValidatedCommitを別型にする。
3. **Implement**: CFB-120/121のcallerを先にsealed APIへ移す。
4. **Prove**: branch/open/scan/show/diff/remoteのconsumer matrixで未検証commit経路をなくす。
5. **Promote**: CFB-120/121へclose evidenceを集約し、残るAPI再設計だけを必要ならADR化する。

## Acceptance Signals

- scan baseとlocal publishがmalformed sealed fixtureをstate mutation前に拒否する。
- branch headをconsumeするproduction symbolの一覧がvalidated APIへ収束する。
- fixed済みroot/certificate/reference regressionが維持される。

## Verification Design

- Focused tests: wrong root type、missing reference、certificate mismatch、future root schema。
- Cross-boundary tests: open→scan→verify→commit→show、file/HTTP remoteのsealed subject matrix。
- Scale or fault tests: deep/shared parent graphでvisit回数を計測しcycleを安全に止める。
- Observability: errorにsubject ID、validation stage、root/reference contextを構造化して返す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-120とCFB-121へ直接包含される。CFR-014、CFR-121〜123はvalidator能力を修正済みで、本件はcaller強制境界という残差。
- Required predecessor: 凍結中CFB-120/121を先に閉じる。
- Can combine with: RVC-024、RVC-025、RVC-029。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- signature認可とcontent sealingを一つのvalidatorへ混同しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
