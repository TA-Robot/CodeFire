# RVC-041: Remote mutation trust boundary

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | security / correctness / architecture |
| Review kind | redesign |
| Priority | P0 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `codefire-cli::remote/http/signatures` |
| Promotion target | future CFR / ADR |

## Executive Judgment

Remote mutationがsealed commit、resource lock、request signatureを組み合わせている点は維持すべきである。一方、現行request signatureはoperation/actor/target等の署名metadataを認証するだけで、upload head/object集合、review decision/comment、MR URL等のmutation payloadをdigestへ含めないため、mutation全体の認証境界としては不足している。

## Scope and Review Question

- In scope: file/HTTP remoteのupload、request-merge、review、applyにおける認証対象、検証順序、visibility point
- Out of scope: TLS製品選定、資格情報配布、remote GC、package運用
- Question: serverが適用する全ての意味的入力を、改ざん不能な一つのrequest subjectとして認証・再検証できるか

## Evidence

- `crates/codefire-cli/src/signatures.rs` — `sign_remote_request_with_key` / `request_signature_digest`: HMAC入力はsignature objectからdigest fieldを除いたmetadataであり、request body digestを持たない。
- `crates/codefire-cli/src/remote.rs` — `upload_branch`: `head`、`objects`、idempotency keyをpayloadへ入れた後に独立したrequest signatureを添付する。
- `crates/codefire-cli/src/http.rs` — `handle_http_upload` / `handle_http_request_review`: signature検証後に、署名入力外のobject recordsやdecision/commentを読み取りmutationへ使う。
- Test/fixture evidence: `crates/codefire-cli/src/signatures.rs::request_signature_rejects_old_timestamp_and_replayed_nonce` は時刻・replayを固定するが、署名後のbody改変counterexampleはない。

## Strengths to Preserve

- remote branch headはsealed commit validation、fast-forward、commit signature policyを通る。
- operation別target文字列、nonce、timestamp、constant-time digest比較、resource lockが既にある。

## Gap / Risk

### Observed gap

request signatureとidempotency payload hashが別々に構築され、前者はmutation bodyを束縛しない。TLSを使わないtransportや中間層の誤実装では、署名を維持したままbodyの意味を変えられる。

### Failure or stagnation mode

認証済みactorが意図したhead、branch、decisionとは異なるmutationが適用されても、署名検証は成功し得る。後からauditしても「誰がどの完全payloadへ同意したか」を一意に証明できない。

### Affected invariants

- remote mutation authentication
- sealed branch head publication
- idempotent replay identity

## Improvement Direction

versioned `RemoteMutationSubject` を導入し、operation、canonical target、actor、canonical payload hash、idempotency key、timestamp、nonceを一つの署名対象にする。serverはrouting後にsubjectを再構築し、signature、payload hash、policy、lock取得後のhead preconditionを順に検証する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: canonical request subjectへbody hashを含める | transport共通で完全な意味を認証できる | schema移行とcross-runtime fixtureが必要 | adopt |
| B: TLSだけを信頼し現行署名を維持 | 実装変更が小さい | file remoteや終端後の改変、audit subjectを解決しない | reject |

## Recommended Sequence

1. **Characterize**: 各mutatorで署名後に一fieldだけ改変するcounterexampleを作る。
2. **Separate**: transport非依存の`RemoteMutationSubject`とcanonical builderを抽出する。
3. **Implement**: client/server双方で同じpayload digestとpreconditionを署名・検証する。
4. **Prove**: file/HTTP、Rust/Python fixture、replay、stale head、body tamper matrixを通す。
5. **Promote**: security ADRと必要な正式CFRへ昇格する。

## Acceptance Signals

- mutation payloadの任意fieldまたはobject recordを変えるとsignature failureになり、writeは0件である。
- file/HTTPで同一semantic requestのsubject bytesとdigestが一致する。
- sealed validation、fast-forward、idempotency conflictの既存結果が維持される。

## Verification Design

- Focused tests: operation/target/actor/body hash/key/nonceそれぞれのtamper table
- Cross-boundary tests: signer → HTTP serialization → server reconstruction → remote record
- Scale or fault tests: 大きいupload object集合、lock待機中のhead変更、replay race
- Observability: payloadを露出せずsubject version/digestと拒否phaseをdiagnosticへ出す

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-035/036/067-070のlock・nonce・idempotency各論を置換せず、完全request subjectという未採番の上位境界を扱う。CFB-121のsealed publicationとも隣接するが別問題。
- Required predecessor: CFB-120〜126の凍結batch完了とremote mutation invariant再監査
- Can combine with: RVC-043、RVC-044、RVC-050、RVC-056

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 公開鍵/KMS導入そのものを先に決めず、まず署名subjectの意味を固定する。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
