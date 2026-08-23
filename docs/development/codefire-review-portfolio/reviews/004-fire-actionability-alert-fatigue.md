# RVC-004: Fireのactionabilityを守りalert fatigueを測定する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | product / domain / UX / performance |
| Review kind | experiment |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 5 |
| Primary scope | `codefire-core::Fire / scan / extinguish / context` |
| Promotion target | strategy experiment |

## Executive Judgment

変更から確認責任をFireとして明示し、解消までsealを拒否するモデルはCodeFireの中心価値である。ただし現在の生成単位はdirect adjacencyごとのrequired Fireを基本とし、量・重複感・修正可能性をdomain outcomeとして測っていない。件数を隠すのではなく、actionability、false-positive、grouping安全性をpilotで定量化すべきである。

## Scope and Review Question

- In scope: Fire生成粒度、severity/reason、grouping、batch resolution、diagnostic優先順位、利用者が取れる次行動
- Out of scope: blockerを無条件にwarningへ下げること、AIによる自動解消判断
- Question: 1件のFireが明確な確認責任を表し、大量変更でも重要なblockerがnoiseに埋もれないか

## Evidence

- `docs/00_philosophy.md` — `AI時代の開発規律`: Fireを未解消の確認責任かつ明示的な整合回復taskとして位置付ける。
- `crates/codefire-core/src/lib.rs` — `adjacent_atoms` / `build_scan_result_with_options`: changed Atomの各direct Trace Linkについてrequired Fireを生成し、semantic keyで同一active Fireを抑止する。
- `crates/codefire-cli/src/extinguish_ux.rs` — `run_interactive_extinguish` / `run_all_matching_extinguish`: recent evidence候補、draft command、queryによる複数解消を提供する。
- `crates/codefire-cli/src/automation.rs` — `scan_next_actions` / `verification_next_actions`: open fire数に応じてbatch templateやextinguish actionをboundedに提示する。
- Test/fixture evidence: CFB-005/006/018/027とbatch testsは入力負荷を改善したが、Fireごとのactionability/false-positive率を判定するpilot rubricはない。

## Strengths to Preserve

- Fireがstable identity、source/target、reason、trace path、basisを持ち、resolutionへ追跡できること。
- batchでも全件validateしてからwriteし、単なるalert dismissalにしないこと。

## Gap / Risk

### Observed gap

全direct adjacencyを同じ`required` severityで提示するモデルは、fan-outの大きいAtomやontologyが粗いrepositoryでFire stormを起こしやすい。groupingは操作UXであり、同じ根拠を共有できる群か、重要度が異なるかをdomainとして表さない。

### Failure or stagnation mode

利用者が定型rationaleで一括解消し、evidenceの質とblockerの意味が低下する。重要な影響も大量noiseに埋もれ、CodeFireの採用価値が「作業を増やすだけ」と評価される。

### Affected invariants

- Fire as explicit review responsibility
- evidence-backed resolution
- no false completion / bounded automation output

## Improvement Direction

Fireを減らす前にactionability rubric（distinct decision、owner、expected action、basis、false-positive reason）とwork counterを定義する。安全に同一判断を共有できるFire groupをfirst-class read modelとして試し、blocking semanticsは個々のFireに保持する。policy/profile別のpropagation範囲はevidenceに基づき狭める。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 一律warning化または上限超過分を非表示 | 操作量が減る | blocker逃しとfalse completion | reject |
| B: actionability audit + explainable grouping | integrityを保ち摩擦原因を特定 | pilotと新read modelが必要 | experiment |

## Recommended Sequence

1. **Characterize**: dogfood以外を含むpilotでFire数、distinct decisions、dismiss/refresh率、解消時間を採取する。
2. **Separate**: Fire truthとpresentation group/actionability annotationを分ける。
3. **Implement**: group preview、shared-basis validation、top blocker reasonsのread-only prototypeを作る。
4. **Prove**: groupingしても全Fire identity/basisが残り、blocker decisionが変わらないequivalence testを行う。
5. **Promote**: thresholdを満たすprofileだけproduct spec/policyへ昇格し、失敗時はpropagationを狭める。

## Acceptance Signals

- predeclared pilot thresholdでactionable Fire率が達成され、false-positive理由を分類できる。
- grouping操作後も各Fireにresolution/evidence basisが追跡できる。
- open required Fireを省略・group化してもverification resultとcountが不変である。

## Verification Design

- Focused tests: fan-out、duplicate semantic key、obsolete/reopen、shared-basis grouping。
- Cross-boundary tests: scan → context/explain → batch extinguish → verify → certificate count。
- Scale or fault tests: 1万Atom/high-degree graphでbounded outputとgroup calculation work counterを測る。
- Observability: generated/grouped/actioned/invalidated Fire数、reason distribution、omitted countを出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-005/006/018/027は個別UX、CFR-145はplan size。本候補はそれらの修正を再提案せず、Fireモデルのactionability仮説を検証する。
- Required predecessor: RVC-006/007のAtom・Trace semantics安定性とStage 1 correctness gate。
- Can combine with: RVC-012、RVC-013、RVC-010。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- 件数を減らすためにrequired Fireを黙ってdropしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
