# RVC-005: Evidenceのtrustとprovenanceをsubject-bound contractへ高める

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | security / domain / correctness / evolution |
| Review kind | redesign |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 1 |
| Primary scope | `evidence / artifact_ref / resolution_ledger / verification command` |
| Promotion target | ADR |

## Executive Judgment

Evidenceとartifactをcontent-addressed objectとしてsealed resolutionから到達可能にする構造は強い。一方、現在のevidence objectは「何を、誰が、どの環境で、どのsubjectに対して実行したか」のprovenanceを完全には束縛せず、captured outputの存在が検証の信頼性と混同され得る。信頼tierとsubject/environment bindingを型にし、captureとattestationを分離すべきである。

## Scope and Review Question

- In scope: command/artifact evidence、hash、cwd/env/tool identity、subject digest、proof status、resolution/commitへのbinding
- Out of scope: arbitrary commandの安全化そのもの、remote attestation authorityの即時導入、秘密情報の保管基盤
- Question: sealed commitのevidenceを第三者が再検証するとき、由来・対象・限界・改ざん有無を判定できるか

## Evidence

- `crates/codefire-cli/src/evidence.rs` — `run_evidence_add` / `CommandCapture::to_json`: command、cwd、exit、timeout、duration、bounded stdout/stderrと`proof_status`をevidence objectへ保存する。
- `crates/codefire-cli/src/evidence.rs` — `store_artifact_ref` / `artifact_location`: artifactをstreaming SHA-256、size、URI/path kindで記録し、repository外pathをredactする。
- `crates/codefire-core/src/lib.rs` — `Resolution` / `ResolutionBasis`: evidence refsとAtom/Trace/policy basisを保持するが、evidence object自体のsubject digestをresolution basisへ結合しない。
- `crates/codefire-store/src/lib.rs` — `validate_resolution_ledger_references` / `validate_evidence_references`: sealed graphでevidenceとartifact_refの型・到達性を検証するが、command結果の真実性やenvironment同一性は検証しない。
- Test/fixture evidence: evidence timeout/redaction/missing-ref testsは存在する一方、同じevidenceを異なるchange subjectへ再利用するcross-subject replay fixtureはない。

## Strengths to Preserve

- evidence/artifactがimmutable object identityを持ち、missing referenceをverification/store双方が検出すること。
- shell modeとargv mode、失敗capture、timeout、output truncation、path redactionを明示すること。

## Gap / Risk

### Observed gap

`created_at`とcommand captureはあるが、input tree/commit/Atom basis、policy、tool binary/config、selected environment variables、producer identity、trust tierの統一契約がない。`command-passed`はローカルprocessのexit successであり、隔離や再現性を意味しない。

### Failure or stagnation mode

古い、別subject向け、または信頼できない環境のevidenceが形式上有効なrefとしてresolutionへ使われ、false completionを生む。逆に過剰な環境captureはsecret leakageと非決定性を増やす。

### Affected invariants

- evidence-backed resolution / subject binding
- zero stale or forged evidence completion
- sealed object reachability / secret minimization

## Improvement Direction

Evidence envelopeを`capture`、`provenance`、`claim`へ分け、subject digest、policy/profile、producer/tool identity、declared environment fingerprint、trust tier（local-unisolated等）、redaction manifestを必須/critical-extension規則付きでversion化する。resolutionは期待subjectとevidence claimの一致をdomain predicateで検査する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 現行objectへ任意metadataを足す | 後方互換が容易 | trust判断がconsumer依存で曖昧 | reject |
| B: versioned provenance/claim envelope | replay・誤読を機械的に拒否 | schema/migrationとprivacy設計が必要 | adopt |

## Recommended Sequence

1. **Characterize**: cross-subject reuse、cwd drift、tool drift、env secret、artifact replacementのthreat fixturesを作る。
2. **Separate**: raw capture、artifact identity、provenance、evidence claim、resolution acceptance predicateを分離する。
3. **Implement**: local-unisolated profileからsubject bindingとredaction manifestを導入する。
4. **Prove**: replay/missing-critical-field/tamperを拒否し、same-subject legacy policyを明示的に扱う。
5. **Promote**: threat modelとprivacy decisionをADR、product spec、public schemaへ昇格する。

## Acceptance Signals

- 異なるsubject向けevidence refはverificationでstale/invalidとして拒否される。
- consumerがcapture成功とtrusted attestationを区別でき、unknown trust tierをpassへ昇格しない。
- repository外path/secretのredactionとexisting evidence reachability validationが回帰しない。

## Verification Design

- Focused tests: subject/policy/tool/env binding、legacy profile、redaction、failed/timeout capture。
- Cross-boundary tests: evidence add → extinguish → verify → commit → sealed graph validation。
- Scale or fault tests: artifact TOCTOU、object tamper、cross-repo replay、大output、process tree timeout。
- Observability: trust tier、subject digest、redacted fields、reproducibility statusをbounded diagnosticにする。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-037/086とCFR-042/044/109/116はexit、shell、path、runner contractの個別修正。本候補はprovenanceとsubject replay防止を上位設計するため重複しない。
- Required predecessor: Stage 1 path/trust-boundary red teamとRVC-011のauthoritative predicate。
- Can combine with: RVC-018、RVC-019。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- local command outputを自動的にsecure/compliant attestationへ格上げしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
