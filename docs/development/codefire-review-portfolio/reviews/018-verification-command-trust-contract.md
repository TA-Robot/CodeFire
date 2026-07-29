# RVC-018: verification commandのtrust contractをexecution profileとして固定する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | security / correctness / architecture |
| Review kind | redesign |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 1 |
| Primary scope | `VerificationCommand / command runner / policy / evidence` |
| Promotion target | ADR |

## Executive Judgment

timeout、output cap、env control、allow-failureをcore policy型へ入れた修正は良い基礎である。ただし現行runnerは`sh -c`、joinされたcwd、inherited environment、同一OS権限で実行し、passがどのtrust profileを意味するかをverificationへ残さない。trusted local executionという境界を明記し、path/process/resource/subject contractを型と結果へ加えるべきである。

## Scope and Review Question

- In scope: shell/argv mode、cwd containment、env、timeout/process tree、resource bounds、subject snapshot、execution profile、result provenance
- Out of scope: 汎用sandbox/container orchestrator、untrusted multi-tenant serverの即時対応
- Question: verification passが再現可能な対象と実行境界にbindされ、policyからroot外command実行へ逸脱しないか

## Evidence

- `crates/codefire-core/src/lib.rs` — `VerificationCommand`: command/cwd、timeout、max output、inherit_env、env、allow_failureを持つ。
- `crates/codefire-cli/src/main.rs` — `run_verification_command`: `Command::new("sh").arg("-c")`で実行し、`open_dir.join(check.cwd)`をcwdとし、poll/kill後にbounded combined outputを作る。
- `crates/codefire-cli/src/main.rs` — `run_verification_commands`: 非zero/timeoutをFailedCheckへ変換するが、execution profile、environment/tool/subject digestを結果へ保持しない。
- `docs/development/source-architecture.md` — `Required Invariants and Current Gaps`: command executionはtimeout/output/environment policyに従うが、OS isolation欠如を明示的trust boundaryとする。
- Test/fixture evidence: CFR-116のtimeout/env/output testsはfixedだが、absolute/`..` cwd escape、child process tree、subject mutation、profile labelingのred-team matrixはない。

## Strengths to Preserve

- resource defaultsをcore型に持ち、timeout/output truncation/allow-failureをdeterministicに評価すること。
- command failure detailsをverification factsへ含め、renderer外でpass/failを決めること。

## Gap / Risk

### Observed gap

shell stringはpolicy authorの権限で任意実行でき、cwdがroot containmentを保証しない。timeout killはdirect child中心で、descendant/resource/FS/network境界はない。実行前後にworking subjectが変化しても同じverificationへbindされる可能性がある。

### Failure or stagnation mode

malicious/accidental policyがroot外を変更・読取し、childが残存する。または検証中にsubjectが変わり、別bytesへのpassをcommitへ使う。consumerがlocal-unisolated passをisolated attestationと誤解する。

### Affected invariants

- root/path containment
- verification subject freshness
- no unauthorized mutation / honest trust claim

## Improvement Direction

`ExecutionProfile`をversion化し、trusted-local-shell、trusted-local-argv、将来のisolated profileを明確に分ける。cwdはcanonical root-relative、argv modeを優先し、environment allowlist、process-group termination、resource/FS/network declarations、before/after subject digestを結果provenanceへ残す。unsupported isolation要求はfail closedにする。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 現行runnerへ注意書きだけ追加 | 低コスト | escape/replayを機械的に防げない | reject |
| B: typed profiles + subject/path enforcement | claimと実行を一致 | platform差、migration、runner complexity | adopt |

## Recommended Sequence

1. **Characterize**: cwd escape、symlink、env secret、process tree、timeout race、subject mutationのred-team fixturesを作る。
2. **Separate**: policy command spec、validated execution plan、runner、capture/provenance、decisionを分離する。
3. **Implement**: canonical cwd containment、argv profile、before/after subject check、profile labelから始める。
4. **Prove**: fault/process testsとsame-subject replay fixtureでpassの境界を固定する。
5. **Promote**: trust-boundary ADR、policy spec、certificate claimへ昇格する。

## Acceptance Signals

- absolute/`..`/symlinkでroot外cwdへ出られず、明示diagnosticになる。
- verification中にsubjectが変わればpassをcommitへ利用しない。
- consumerがunisolated/isolated/unsupportedをmachine-readableに区別できる。

## Verification Design

- Focused tests: shell/argv、cwd canonicalization、env allowlist、timeout/output、allow-failure。
- Cross-boundary tests: policy parse → execution plan → runner → Verification → evidence/certificate claim。
- Scale or fault tests: forked descendants、signal無視、large output、resource exhaustion、TOCTOU/symlink swap。
- Observability: profile、cwd、subject before/after、timeout/truncation/resource dispositionをsecret-freeに出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-041/042/043/116とCFB-049はtimeout/shell/cwd/typed fieldsの個別問題。本候補はverification gateのexecution trust profileとsubject bindingで重複しない。
- Required predecessor: Stage 1 root/path red team、RVC-005 provenance、RVC-011 decision model。
- Can combine with: RVC-005、RVC-019。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- isolation未実装のprofileをsecure sandboxと称さない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
