# RVC-020: core domainのpurityとdeterminism境界を明確にする

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | architecture / testing / correctness / performance |
| Review kind | refine |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `codefire-core / filesystem extractors / policy parsers / domain evaluators` |
| Promotion target | future CFR |

## Executive Judgment

CLI→core/store/utilというdependency directionと、scan/verification/resolution builderに時刻等を引数で渡す設計は良い。しかし`codefire-core/src/lib.rs`はfilesystem traversal、config/YAML parsing、text extraction、domain evaluation、hashingを単一moduleで担い、純粋なdecisionをfixtureなしに再利用しにくい。crate分割を目的化せず、portsとpure values/functionsの境界をmodule単位で明示すべきである。

## Scope and Review Question

- In scope: filesystem/input adapters、extractor selection、canonical parsed values、scan/verification/freshness evaluation、clock/order/hash inputs
- Out of scope: crate数増加そのもの、store/CLI責務をcoreへ移すこと、AI inference
- Question: 同じcanonical inputからOS/path/traversal orderに依存せず同じdomain facts、IDs、diagnosticsを生成できるか

## Evidence

- `docs/development/source-architecture.md` — `Crate Boundaries` / `Core Domain Flow`: coreはAtom/Trace/Fire/resolution/verificationを所有し、filesystem repository layoutやCLI renderingを持たないとする。
- `crates/codefire-core/src/lib.rs` — `build_atom_index` / `rel_files` / `collect_rel_files`: coreが直接filesystemを再帰走査し、hard-coded directory除外、path sort、config matching、extractor実行を行う。
- `crates/codefire-core/src/lib.rs` — `current_trace_graph` / `parse_verification_policy`: core domain valuesを作る一方で、filesystem上のlimited YAMLを直接読むadapter責務も持つ。
- `crates/codefire-core/src/lib.rs` — `build_scan_result_with_options` / `build_verification` / `stale_resolutions`: canonical valuesと明示的`now`等から結果を作る比較的pureなfunctionsが既に存在する。
- `crates/codefire-cli/src/main.rs` — `compute_scan` / `compute_verify`: IO orchestrationとcore pure-ish buildersを接続するが、base loading/manifest/non-Atom計算はCLI側へ分散する。
- Test/fixture evidence: core unit testsはtempdir中心で、pure golden/property corpusとcross-platform path/order determinism corpusは限定的である。

## Strengths to Preserve

- core/storeがCLIへ依存せず、verification/buildersの主要入力がtyped valuesであること。
- BTreeMap/sort、canonical hash、明示時刻引数により多くの出力をdeterministicにしていること。

## Gap / Risk

### Observed gap

IO/parsing/selection/evaluationが3124行の単一`lib.rs`に混在し、domain functionの入力境界とalgorithmic complexityが見えにくい。hard-coded traversal ignoreやlossy text readがdomain結果へ暗黙影響する。

### Failure or stagnation mode

同じtreeでもplatform/path/encoding差でAtomIndexが変わる、adapter実験がfilesystemをmockできずcore semanticsを複製する、またはrefactorでIO side effectがread-only decisionへ混入する。

### Affected invariants

- deterministic object/Atom/Fire identity
- parse/validate/plan separation
- core/CLI/store dependency direction

## Improvement Direction

`source_input`（bytes/path metadata）、`extractor`、`policy_format`、`domain`（scan/verification/freshness）、`identity` modulesへ段階分離する。filesystem adapterはsorted normalized inputsを作り、pure functionsはそのvaluesだけを受ける。golden/property testsをpure layerに置き、OS adapterはconformance fixturesで照合する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: 直ちに多数crateへ分割 | 境界が見える可能性 | churnと循環/公開API固定化 | reject |
| B: same-crate module/ports-first refactor | 小さく証明しながら分離 | 一時的adapter重複 | adopt |

## Recommended Sequence

1. **Characterize**: path order、separator、encoding、ignored dirs、clock、map orderのdeterminism corpusを作る。
2. **Separate**: pure input structs/functionsを同crate moduleへ抽出し、filesystem adapterを薄くする。
3. **Implement**: Atom extraction/index assembly、policy parse/evaluateの順にportsへ移す。
4. **Prove**: old adapter/new pure pathのgolden equivalence、property tests、work countersを通す。
5. **Promote**: source architecture/module boundariesと必要なmaintainability CFRへ昇格する。

## Acceptance Signals

- canonical in-memory fixtureだけでAtom/Trace/scan/verification/freshnessの主要decisionを再現できる。
- filesystem adapterのentry order/path separator差でobject-producing valuesが変わらない。
- public behavior、diagnostic、complexityがrefactor前後でequivalentである。

## Verification Design

- Focused tests: pure extractor/index/evaluator、ordering、clock injection、invalid bytes/path normalization。
- Cross-boundary tests: filesystem adapter → canonical inputs → core values → store canonical IDs → CLI rendering。
- Scale or fault tests: deep/wide tree、symlink/permission race、mixed encoding、large source set、adapter interruption。
- Observability: files/bytes/Atoms/links visitedとadapter/evaluator phase countersを分ける。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-001/002/009はmain.rs集中、CFR-040はutility重複。本候補はcore内部のIO/pure-domain境界であり重複しない。module分割自体を成果にしない。
- Required predecessor: RVC-006/007/011のcanonical input/decision contractsとfocused regression corpus。
- Can combine with: RVC-015、RVC-018。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- crate数や抽象trait数を品質指標にせず、determinism/equivalenceで判断する。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
