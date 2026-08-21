# RVC-053: Python fallback compatibility contract

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | evolution / correctness / product |
| Review kind | refine |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 2 |
| Primary scope | `Python reference/fallback ↔ Rust-default boundary` |
| Promotion target | product spec / ADR |

## Executive Judgment

Rust-first、Python freeze、fallback-only command一覧、evidence-based retirement gateは適切である。しかし「reference」「fallback」「emergency read」の保証がfixture profileとして固定されず、writer差が文書上のobserved gapに留まるため、support matrixを実行可能contractへ変える必要がある。

## Scope and Review Question

- In scope: read/write/migrate/incident support、object/repository/remote/signature format、fallback-only surface
- Out of scope: Pythonへの新機能追加、package/install経路、Python実装の全面refactor
- Question: どのPython-produced stateをRustが読み/変更でき、逆方向はどこまで保証し、何を明示拒否するか

## Evidence

- `docs/development/source-data-model-catalog.md` — `Rust/Python Writer Boundary`: 24/12-hex ID、Atom hash schema、commit root schema、JSON envelopeの差をcanonicalに列挙する。
- `docs/development/known-limitations.md` — `Rust default and Python fallback`: Rust-first freeze、fallback-only `gc`/`token-hash`、削除条件を定義する。
- `codefire` — `canonical_json` / `object_id` / `build_atom_index` / `build_parser`: Pythonは独立writer/reader/CLIとして大きなsemantic surfaceを保持する。
- `crates/codefire-store/src/lib.rs` — `legacy_object_id` / `read_object_accepts_legacy_twelve_hex_object_ids`: Rustは一部legacy readを明示実装する。
- Test/fixture evidence: `init_creates_python_compatible_repo_layout`と`status_reads_python_compatible_open_directory`は手作りshapeを確認するが、実Python binaryが生成したrepositoryをRustへ渡す双方向fixtureではない。

## Strengths to Preserve

- Pythonへmainline featureを足さず、critical compatibility/security fixへ限定するfreeze policyがある。
- fallback retirementを日付やversion数でなくevidence gateにしている。

## Gap / Risk

### Observed gap

command名のdoc consistencyはtestされるが、formatごとのreader/writer/mutator support levelと拒否結果がmachine-readableではない。両実装がremote/signature等を重複所有し続ける。

### Failure or stagnation mode

fallbackが「読めるが書くとRust schemaを落とす」repositoryを静かにrewriteする、緊急時に必要なcommandが実fixtureで動かない、retirement判断が印象論になる。

### Affected invariants

- canonical identity and legacy readability
- schema downgrade prevention
- explicit support/retirement boundary

## Improvement Direction

compatibility profileを`read`, `diagnose`, `migrate`, `write-preserving`, `unsupported`で定義し、writer/runtime/versionごとのfixture manifestへする。Pythonはprofile外のunknown required schemaをfail closedし、retirementはcorpus結果とincident use coverageで判断する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: executable compatibility profiles | support claimとretirement根拠が明確 | fixture/corpus maintenanceが必要 | adopt |
| B: docs-only freezeを維持 | 管理が軽い | semantic driftを実行時に検出できない | reject |

## Recommended Sequence

1. **Characterize**: actual Python/Rust writer artifactと操作matrixを収集する。
2. **Separate**: product support profileとfallback command inventoryをversioned manifest化する。
3. **Implement**: unsupported write/downgradeを両runtimeで明示拒否する。
4. **Prove**: RVC-055 corpusでread/write/migrate/reject resultを再現する。
5. **Promote**: product spec、release/format roadmap、retirement ADRへ反映する。

## Acceptance Signals

- 各fixtureにwriter、schema axes、allowed consumers/actions、expected resultがある。
- profile外writeはsilent downgradeせずstable unsupported diagnosticになる。
- fallback retirement/維持判断を一つのmatrix結果から説明できる。

## Verification Design

- Focused tests: fallback command inventory、unknown version、legacy ID/root/hash schema
- Cross-boundary tests: Python write→Rust read/migrate、Rust write→Python emergency-read/reject
- Scale or fault tests: signed remote、corrupt legacy record、partial migration、mixed IDs
- Observability: runtime/writer/profile/schema axisとdecisionを出し、release identityと混同しない

## Dependencies and Duplication Check

- Existing CFB/CFR: fixed CFR-038/080は二重実装とcommand inventoryを扱った。本候補は実行可能support profileであり重複issueではない。
- Required predecessor: RVC-055 corpus、RVC-057 unknown-version policy
- Can combine with: RVC-055、RVC-057、RVC-059

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- Python fallbackへRust未所有の新command/formatを追加しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
