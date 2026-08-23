# RVC-035: next_actionsに安全な実行可能性を明示する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | UX / security / evolution |
| Review kind | redesign |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 2 |
| Primary scope | `automation::next_action` / command recommendations |
| Promotion target | product spec |

## Executive Judgment

next_actionsを共通shape、上限、omission summary、path contextへ揃えた改善は維持すべきである。しかしcommand fieldには即実行可能なcommand、placeholder template、元policy由来の任意shell commandが同じ文字列型で入る。consumerが安全に扱えるよう executable/template/informational とconfirmation/preconditionを明示し、argv表現を追加するべきである。

## Scope and Review Question

- In scope: next action schema、shell quoting、placeholder、safe auto-execution、precondition、confirmation
- Out of scope: CodeFire自身によるagent実行、shell sandbox、UI固有button設計
- Question: 外部consumerがsourceや文言を推測せず「表示だけ」「編集して実行」「そのまま安全に実行」を区別できるか

## Evidence

- `crates/codefire-cli/src/automation.rs` — `next_action` / `normalize_next_action`: kind、command、reason、targetを共通化するが、実行可能性・argv・confirmation fieldを持たない。
- `crates/codefire-cli/src/automation.rs` — `verification_next_actions`: failed checkのcheck.commandをそのままrerun_checkのcommandへ入れる。
- `crates/codefire-cli/src/main.rs` — `commit_operation_plan`: apply actionにmessage placeholderを含むcommand文字列を返し、pathはdisplay formatで組み立てる。
- Test/fixture evidence: `crates/codefire-cli/src/main.rs` — lock contention next actionはcommand内に省略記号を含む。既存common next-action testsはshapeを検査するが、placeholder/shell safety classを検査しない。

## Strengths to Preserve

- actionのkind、reason、targetとpath contextをmachine-readableに返すこと。
- MAX_NEXT_ACTIONSとomitted_by_kindにより巨大workflowでも出力をboundedにすること。

## Gap / Risk

### Observed gap

command stringが「copy-paste可能」という暗黙の印象を与える一方、message placeholder、省略記号、shell command、確認が必要なmutationを区別しない。

### Failure or stagnation mode

wrapperがtemplateをそのままshellへ渡す、未quote値を解釈する、危険なmutationを確認なしで実行する。安全側consumerは全actionを表示専用にし、next_actionsのautomation価値が伸びない。

### Affected invariants

- safe automation contract
- path/argument integrity
- explicit mutation consent

## Improvement Direction

ActionModeを executable_argv、editable_template、informational に分け、executableにはprogram/argv、working context、read_only、requires_confirmation、preconditions、expected subject fingerprintを持たせる。human向けcommand stringは表示用に残す。policy由来shell rerunは既定でtemplate/confirmation-requiredとし、CodeFireが安全性を保証しないことを明示する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: typed action mode＋argv＋precondition | consumerが安全なsubsetを自動実行できる | schema versionとproducer移行が必要 | adopt |
| B: command stringのquoteだけ改善 | copy-paste事故を減らす | placeholder、shell、consentを区別できない | reject |

## Recommended Sequence

1. **Characterize**: 全actionをread-only/mutation/template/external-shellへ分類する。
2. **Separate**: display commandとexecution specを別field/typeにする。
3. **Implement**: status/context/doctor等read-only actionからexecutable_argv化する。
4. **Prove**: round-trip quoting、placeholder拒否、precondition stale、confirmation policyを試験する。
5. **Promote**: action modeとsafe consumer ruleをpublic schema/specへ昇格する。

## Acceptance Signals

- 全next actionにmodeがあり、templateをexecutableとしてparseできない。
- executable actionはshell文字列再parseなしにargvで実行でき、path/branchのround-tripが一致する。
- mutation/external commandはconfirmationとsubject preconditionを明示する。

## Verification Design

- Focused tests: spaces/quotes/newlines/leading dash、placeholder、empty argv、action mode validation。
- Cross-boundary tests: scan/verify/doctor/merge/remote error actionを独立consumerが分類する。
- Scale or fault tests: bounded omission後のrecovery actionとstale precondition。
- Observability: action mode別count、unsafe/template reason、precondition subjectを出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-103はbranch/path shell quoting、CFR-142/143/154はbounded/schema/envelope昇格をfixed済み。本件は安全なexecutability semanticsで非重複。
- Required predecessor: RVC-032/033のcapability・schema、RVC-034のoutcome model。
- Can combine with: RVC-040、RVC-036。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- CodeFire本体がnext actionを自律実行する機能は追加しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
