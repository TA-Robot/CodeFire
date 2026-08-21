# RVC-032: capability registryを階層的なpublic contractへ育てる

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | architecture / UX / evolution / testing |
| Review kind | refine |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 2 |
| Primary scope | `crates/codefire-cli/src/command_registry.rs` / help / completion |
| Promotion target | product spec |

## Executive Judgment

COMMAND_CAPABILITIESをhelp、completion、metrics判定へ接続したことは、drift防止の良い出発点である。しかし現在のentryはtop-level name、JSON、metrics、help keyだけで、subcommand、dry-run、mutation、schema、option surfaceは表せない。Stage 2では独立consumerがsourceを読まずに安全なprofileを選べる階層registryへ拡張すべきである。

## Scope and Review Question

- In scope: top-level/subcommand capability、JSON/metrics/dry-run、side effect、schema/help/completion関係
- Out of scope: plugin marketplace、dynamic command loading、権限認可
- Question: consumerがcapabilities出力だけでsupported operationと安全な呼び方を判定でき、実parserとのdriftを検出できるか

## Evidence

- `crates/codefire-cli/src/command_registry.rs` — `CommandCapability` / `COMMAND_CAPABILITIES`: name、supports_json、supports_metrics、help_keyのみを宣言する。
- `crates/codefire-cli/src/command_registry.rs` — `COMMAND_NAMES` / `supports_metrics`: command名を別配列で重複し、branch listだけをspecial caseで扱う。
- `crates/codefire-cli/src/completion.rs` — `bash_completion_script` / `zsh_completion_script`: subcommandとoption集合を大きな手書きcaseで再定義する。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs` — `command_capabilities_are_shared_by_help_completion_and_metrics_parser` は代表例を検査するが、全subcommand/optionのparser conformanceは検査しない。

## Strengths to Preserve

- capabilitiesをstable command_result envelopeで機械取得できること。
- debug commandをpublic registry外と明示した現在の分類意識。

## Gap / Risk

### Observed gap

top-level branchがsupports_metrics=trueでも実際はbranch listのみなど、entry粒度と実parser粒度が一致しない。dry-run、path、idempotency、lock、output schema、read/write性をconsumerが発見できない。

### Failure or stagnation mode

wrapperがhelp文字列をparseするか、危険側に推測する。registry、help、completion、dispatchの一つだけ更新され、capability negotiationが誤った安全性を伝える。

### Affected invariants

- public automation contract
- capability negotiation
- safe command discovery

## Improvement Direction

CommandDescriptorをcommand treeとして表し、profile、subcommands、supported modes、side_effect class、input/output schema ID、common option capabilitiesを宣言する。生成を目的化せず、parser conformance testとhelp/completion rendererが同descriptorを読む範囲から始める。未知required capabilityはunsupportedとして明示し、falseを「不明」と混同しない。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: hierarchical descriptor＋parser conformance | consumerとCLI surfaceの共通authorityになる | descriptor設計と移行が必要 | adopt |
| B: 現行bool entryへfieldを継ぎ足す | 小変更 | subcommand粒度とunknown semanticsが解決しない | reject |

## Recommended Sequence

1. **Characterize**: command/subcommand × mode × side effect × schemaの実装表を生成する。
2. **Separate**: internal parser metadataとpublic capability viewを別型にする。
3. **Implement**: branch、evidence、patchの階層commandからdescriptor化する。
4. **Prove**: declared optionをparserが受理し、undeclared modeを明示拒否するcontract testを作る。
5. **Promote**: capability schemaとunknown behaviorをStage 2 product specへ昇格する。

## Acceptance Signals

- 全registered command/subcommandにprofile、JSON、dry-run、mutation、schema dispositionがある。
- help/completion/capabilities/parserのcommand tree差分がtestで0になる。
- 独立consumerがsource非参照でread-only profileだけを選択できる。

## Verification Design

- Focused tests: descriptor uniqueness、hierarchy、unknown/unsupported distinction。
- Cross-boundary tests: registry↔parser↔help↔completion↔command_result conformance。
- Scale or fault tests: future/unknown capabilityを含むfixtureと旧consumer negotiation。
- Observability: capabilities payloadにregistry schema versionとomitted/experimental markerを出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-082はhelp metadata、CFR-150はmetrics capabilityをfixed済み。本件は階層・negotiation・schemaを追加する次段で非重複。CFR-131のhelp routing fixを維持する。
- Required predecessor: RVC-031のcommand profile、RVC-033のschema family。
- Can combine with: RVC-040、RVC-035、RVC-059のcritical extension/capability policy。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- registryから外部codeをdynamic loadしない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
