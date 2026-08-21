# RVC-045: Path and symlink use boundary

## Review Summary

| Field | Value |
|---|---|
| Status | implemented |
| Perspective | security / correctness / architecture |
| Review kind | redesign |
| Priority | P0 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `repository/open/materialization/output path handling` |
| Promotion target | existing CFB/CFR / ADR |

## Executive Judgment

manifest pathのabsolute/parent component拒否、repo discoveryのcanonicalize、evidence artifact pathのcanonicalizeは維持すべきである。ただし`absolute_path`とopen containmentはlexicalであり、materialize/mergeの最終write先も既存symlinkやcheck-use raceを再検査しないため、CFB-122を個別open修正ではなく共通use-boundary resolverで閉じるべきである。

## Scope and Review Question

- In scope: repo/open/output/artifact/remote storage pathのresolve、containment、symlink、TOCTOU
- Out of scope: installer prefix、platform全般のsandbox、ユーザー任意artifact URIの内容検証
- Question: pathを受理した時点ではなくread/write/remove直前に、同じauthority root内であることを保証できるか

## Evidence

- `crates/codefire-cli/src/main.rs` — `absolute_path` / `ensure_open_target_allowed`: 相対pathをcwdへjoinし、`starts_with`でworkspace/nestingを判定するがcanonicalizeしない。
- `crates/codefire-cli/src/main.rs` — `safe_manifest_output_path` / `materialize_commit` / `write_merged_file`: `..`等は拒否するが、target配下の既存symlink componentやwrite直前のcontainmentを確認しない。
- `crates/codefire-cli/src/evidence.rs` — `absolute_existing_path` / `artifact_location`: 既存pathはcanonicalizeし、repo外artifact pathをredactするため、より強い局所precedentがある。
- `crates/codefire-cli/src/main.rs` — `find_repo_root` / `find_open_marker`: discovery開始点はcanonicalizeしているが、そのpolicyがmutating outputへ共有されていない。
- Test/fixture evidence: CFB-122がsymlink/`..` counterexampleを要求している。現行Rust testsにはwrite直前symlink swap matrixがない。

## Strengths to Preserve

- manifest entryのabsolute、parent、root/prefix componentをfail closedで拒否する。
- diagnostics/planへ元のrepo/open path文脈を保持する。

## Gap / Risk

### Observed gap

path policyがdiscovery、open、manifest、evidence、remoteごとに分散し、lexical path、canonical existing path、未作成leafの扱いが統一されていない。

### Failure or stagnation mode

allowed-lookingsymlinkや`..`でworkspace外へmaterializeし、検証後にparentをsymlinkへ差し替えてmerge/patch writeを逸脱させる。cleanup時のauthorityも曖昧になる。

### Affected invariants

- workspace/repository containment
- user work preservation
- deterministic open identity and cleanup

## Improvement Direction

`ResolvedPathIntent`を導入し、authority root、operation(read/create/write/remove)、canonical existing ancestor、relative leaf、symlink policyを一度表現する。mutation直前にancestor identityとcontainmentを再検査し、可能なplatformではdirectory handle相対操作へ寄せる。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: intent-aware共通resolver + use-boundary recheck | 全path種別に一貫した証明ができる | platform差と段階移行が必要 | adopt |
| B: openだけcanonicalize | CFB-122の一部を早く閉じる | merge/patch/materialize raceが残る | reject as final |

## Recommended Sequence

1. **Characterize**: `..`、existing/non-existing leaf、symlink ancestor、swap raceの表を作る。
2. **Separate**: pure lexical parserとfilesystem-backed resolver/use tokenを分ける。
3. **Implement**: open/materialize/merge/patchから共通resolverへ移す。
4. **Prove**: disposable filesystemでwrite/remove先がroot外へ出ないことを確認する。
5. **Promote**: CFB-122 root fix、path policy ADR、canonical docsへ反映する。

## Acceptance Signals

- 全mutating pathでescape/symlink/race counterexampleがwrite 0件で拒否される。
- validなrepo-relative、sibling open、legacy registry pathは明示policyどおり動く。
- error envelopeがoriginal path、authority root、失敗phaseを安全に示す。

## Verification Design

- Focused tests: path component・symlink種別・存在状態のtable test
- Cross-boundary tests: open/materialize/merge/patch/evidence/file remote
- Scale or fault tests: barrier付きsymlink swap、rename race、permission change、partial parent creation
- Observability: lexical/resolved/use-recheck phaseを示し、機微な外部pathはredactする

## Dependencies and Duplication Check

- Existing CFB/CFR: open CFB-122を主対象とし、fixed CFB-108/110、CFR-043/044の成果を共通境界へ統合する。新しい個別CFBを先に増やさない。
- Required predecessor: CFB-122の凍結batch完了条件
- Can combine with: RVC-046、RVC-050、RVC-056

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- OS-level sandboxや任意外部pathへの一般権限管理は提供しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
