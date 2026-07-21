# RVC-046: Evidence command and secret boundary

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | security / domain / UX |
| Review kind | redesign |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 4 |
| Primary scope | `codefire-cli::evidence` / evidence objects |
| Promotion target | future CFR / product spec / ADR |

## Executive Judgment

evidenceが外部artifact本体をcopyせず、path redaction、streaming hash、timeout、output summaryを備える点は強い。一方、command/argv、absolute cwd、stdout/stderrをsealed evidence objectへ保存し、childはcaller環境を継承するため、secret capture・path disclosure・process containmentを明示したsensitive-evidence boundaryが必要である。

## Scope and Review Question

- In scope: evidence command実行環境、argv/cwd/outputの保存、redaction/classification、timeout/child cleanup、remote搬送
- Out of scope: 任意commandのsandbox実装、外部artifact storage、AI prompt/log運用
- Question: evidenceとしてseal/remote transferする前に、secretとlocal-only metadataを確実に除外・明示承認できるか

## Evidence

- `crates/codefire-cli/src/evidence.rs` — `capture_shell_command` / `capture_argv_command`: `Command`はenvをclear/allowlistせずcaller環境を継承し、shell modeも残る。
- `crates/codefire-cli/src/evidence.rs` — `CommandCapture::to_json` / `run_evidence_add`: command/argv、absolute cwd、bounded後のstdout/stderrをimmutable evidence payloadへ保存する。
- `crates/codefire-cli/src/evidence.rs` — `artifact_location` / `sha256_file`: repo外artifact pathはredactし、payloadをcopyせずstreaming hashする強いprecedentがある。
- `crates/codefire-cli/src/remote.rs` — `object_references`: resolution → evidence → artifact_refをremote graphへ含めるため、captured command metadata/outputもremoteへ搬送される。
- Test/fixture evidence: `evidence_argv_command_capture_avoids_shell_expansion`、timeout/output truncation、artifact path redaction testsはあるが、env secret/argv secret/output redactionとdescendant process cleanupはない。

## Strengths to Preserve

- argv mode、明示timeout、failed-command policy、output truncation metadataを持つ。
- repo外artifact pathをredactし、large artifactはbytesを格納せずhash/sizeだけ扱う。

## Gap / Risk

### Observed gap

command captureにはartifactと同等のclassification/redaction policyがない。output limitは`wait_with_output`後に適用されるためpipe/memoryの実予算ではなく、timeout killも直接childだけでprocess group全体を保証しない。

### Failure or stagnation mode

tokenを含むenv dump、argv、stderr、home pathが永続objectとremoteへ流出する。大量outputでpipeが詰まり、子孫processが残ってtimeout/cleanup契約が崩れる。

### Affected invariants

- evidence provenance and confidentiality
- bounded command execution
- remote object graph privacy

## Improvement Direction

`EvidenceCapturePolicy`を導入し、env default-deny/allowlist、argv/cwd field classification、streaming bounded capture、redaction detectors、local-only/portable evidence区分、explicit unsafe overrideを定義する。remote transfer前にもportable classificationを検査する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: safe-by-default capture profile + explicit unsafe mode | secret境界とautomationが明確 | 既存commandの一部で追加指定が必要 | adopt |
| B: warning/documentationのみ | 互換性が高い | sealed後の回収不能な漏えいを防げない | reject |

## Recommended Sequence

1. **Characterize**: env/argv/cwd/stdout/stderrのsecret corpusとpipe/descendant counterexampleを作る。
2. **Separate**: executor、bounded collector、redactor、evidence serializerを分割する。
3. **Implement**: env allowlist、streaming cap、process-group cleanup、portable classificationを導入する。
4. **Prove**: remote round-tripしてもsecret corpusが保存/搬送されないことを確認する。
5. **Promote**: evidence product spec、privacy ADR、必要なCFRへ昇格する。

## Acceptance Signals

- default captureで代表的token/env/path/credential patternがobjectにもdiagnosticにも残らない。
- output memoryは設定上限の定数倍、timeout後にchild/descendantとpipeが確実に終了する。
- redaction内容、truncation、portable/local-only区分が機械可読である。

## Verification Design

- Focused tests: env inheritance、argv/output redaction、UTF-8境界、shell/argv mode、cwd classification
- Cross-boundary tests: evidence object → resolution ledger → upload/clone → readback
- Scale or fault tests: infinite output、grandchild、timeout、broken pipe、artifact swap
- Observability: redacted field/count、bytes seen/stored、termination reason。secret値自体は出さない

## Dependencies and Duplication Check

- Existing CFB/CFR: fixed CFR-041〜045、CFB-037〜039/049/072/088は個別UX・timeout・path改善。本候補は残るsecret/portable evidence境界を扱い、それらを再採番しない。
- Required predecessor: RVC-045のpath classification、Stage 4 sensitive-evidence red team
- Can combine with: RVC-042、RVC-045、RVC-056、RVC-057

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- CodeFireを汎用sandbox/secret scanner製品へ拡張しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
