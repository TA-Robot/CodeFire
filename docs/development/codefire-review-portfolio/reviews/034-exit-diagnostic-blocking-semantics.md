# RVC-034: exit code・diagnostic・blockingを一つの意味論へ揃える

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | domain / correctness / UX / evolution |
| Review kind | redesign |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 2 |
| Primary scope | `exit_code.rs` / `CliError` / automation diagnostics |
| Promotion target | product spec |

## Executive Judgment

stable exit taxonomyとVerification由来のpriority orderは優れたautomation基盤である。ただしdiagnostic severityはblocking/error/warningの複数語彙を使い、custom lock envelopeなど共通CliError diagnosticを迂回する経路が残る。exit、ok、blocking、severity、repairabilityを一つのtyped outcomeから導出し、rendererによる意味の再判断をなくすべきである。

## Scope and Review Question

- In scope: domain blocker、usage/corruption/remote failure、exit code、ok、severity、blocking、repairability
- Out of scope: log verbosity、human文言の翻訳、process signal mapping
- Question: 同じfailureがtext/JSON/remote経路で同じblocking意味とexit classを持ち、矛盾する組合せを生成できないか

## Evidence

- `crates/codefire-cli/src/exit_code.rs` — `ExitCode` / `verification_exit_code`: public codeとdomain blockerの優先順を一箇所に定義する。
- `crates/codefire-cli/src/main.rs` — `CliError::exit_status` / `CliError::diagnostic`: error variantからexitとdiagnosticを別matchで導出し、多くをseverity=blocking、blocking=trueにする。
- `crates/codefire-cli/src/automation.rs` — `verification_diagnostics_json_with_filter`: domain diagnosticはseverity=error/warningとblocking boolを使う。
- Test/fixture evidence: `crates/codefire-cli/src/main.rs` — `lock_contention_envelope` はseverity=blockingを直接構築し、blocking/repairable/diagnostic exit_codeやrepoを共通mapperと同じ形では返さない。

## Strengths to Preserve

- verification.resultをtruthとしてexit codeを決め、non-blocking missing linkをwarningにできること。
- corruption、lock、remote、idempotency等を区別するstable numeric taxonomy。

## Gap / Risk

### Observed gap

severity vocabularyとfield presenceがproducerごとに異なり、ok/exit/diagnosticの整合をconstructorが保証しない。CliErrorのmessage prefix解析もkind/next action決定に使われる。

### Failure or stagnation mode

consumerがseverity文字列だけ、exit codeだけ、blocking boolだけを見て異なる判断をする。新error variantで一方のmatch更新を忘れ、ok=falseなのにnonblocking表示、またはwarningだけなのにfailure exitとなる。

### Affected invariants

- automation contract
- verification blocker truth
- fail-closed compatibility

## Improvement Direction

OutcomeClass、DiagnosticSeverity、BlockingDisposition、Repairabilityをenum化したDiagnosticSetをdomain/CLI境界で作り、exit codeとokをその集合から導出する。severityはerror/warning/info、blockingは独立bool/dispositionに統一する。message文字列解析はtyped contextへ置換し、remote diagnosticも同schemaをtransportする。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: typed outcomeからenvelope/exitを派生 | 矛盾する組合せをconstructorで防げる | error variantとremote変換の移行が必要 | adopt |
| B: JSON schemaでfield presenceだけ検査 | 表面driftを検出 | semantic矛盾とmessage parsingは残る | reject |

## Recommended Sequence

1. **Characterize**: 全ExitCode × CliError × Verification blockerのtruth tableを作る。
2. **Separate**: typed diagnostic modelとrender/transport DTOを分ける。
3. **Implement**: verification、usage、lock、corruptionの代表classから移行する。
4. **Prove**: property testでok、exit、blocking、severityの不正組合せを生成不能にする。
5. **Promote**: taxonomyとcompatibility ruleをpublic automation specへ昇格する。

## Acceptance Signals

- 全failure envelopeでdiagnostic schemaとseverity語彙が統一される。
- ok=trueかつblocking=true、ok=falseかつexit_code=0等の禁止組合せが0になる。
- text、local JSON、HTTP propagated errorが同じoutcome classとexit codeになる。

## Verification Design

- Focused tests: truth table、multi-blocker priority、warning-only pass、repairability。
- Cross-boundary tests: core Verification→CLI→HTTP remote→consumerのround-trip。
- Scale or fault tests: 多数diagnosticのbounded summaryと未知kind/version。
- Observability: primary outcome、all blocker classes、suppressed/omitted countを出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-095はCliError structured diagnosticをfixed、CFR-140はblocking-only payload filterをfixed済み。本件は複数producerの意味論統一で次段。CFB-003/028/069の個別UXは包含しない。
- Required predecessor: RVC-033のschema family。
- Can combine with: RVC-035、RVC-031。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 複数blockerを一つのexit codeへ完全可逆に符号化しようとしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
