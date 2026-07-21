# RVC-002: 非Git互換を守るadoption bridgeの契約を検証する

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | product / architecture / evolution |
| Review kind | experiment |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 3 |
| Primary scope | `read-only snapshot assessment / import / public automation contract` |
| Promotion target | strategy experiment |

## Executive Judgment

Git互換やGit上位レイヤにならない境界は、consistent-only historyの意味を守るため維持すべきである。ただしnative移行だけを価値体験の入口にすると、未検証のproduct仮説に高い移行コストを先払いさせる。coreへGit semanticsを入れず、read-only snapshot/base-head assessmentを別claim typeとして実験すべきである。

## Scope and Review Question

- In scope: 既存directory/snapshotのread-only評価、native importとの区別、subject identity、claim labeling
- Out of scope: Git command互換、transparent coexistence、staging/rebase/force-push semantics
- Question: native sealed historyを弱めず、既存repositoryで最初の有用なsignalを得られるか

## Evidence

- `docs/01_product_definition.md` — `Non-Goals`: Git互換、Git repositoryとの透過共存、staging、WIP commitを明示的に対象外とする。
- `docs/development/strategy/product-strategy.md` — `Adoption Ladder` / `First Integration Experiments`: inspect/advisoryから始め、Git bridgeはread-only snapshot/base-head assessmentに限定する。
- `docs/development/strategy/decision-register.md` — `STR-009` / `STR-X04`: core非互換をacceptedとし、bridge価値とidentity ambiguityを実験対象にする。
- `crates/codefire-cli/src/import_workflow.rs` — `import_existing_project`: 既存directoryをnative `.codefire` repositoryへ変換し、open-burning stateとscanを生成するmutating adoption pathである。
- Test/fixture evidence: importのdry-run/idempotency testsはnative adoptionを扱うが、外部snapshotをmutationなしで評価するclaim-separation fixtureは未整備。

## Strengths to Preserve

- branch headがsealed commitだけを指すnative authorityと、WIP/checkpointを保存しない原則。
- automation envelope、dry-run、bounded diagnosticsというadapter向けkernel。

## Gap / Risk

### Observed gap

製品戦略にはread-only bridgeがあるが、実装の入口はnative import/open flowが中心である。外部snapshotをCodeFire commitと誤認させずに評価するsubject model、coverage表示、capability profileがまだproduct contractになっていない。

### Failure or stagnation mode

利用者が価値検証前にrepository移行を拒むか、急いだadapterがGit-specific identityをcoreへ持ち込みnative certificateとadvisory reportを混同させる。

### Affected invariants

- native sealed commit authority
- subject identity / claim type
- reversible adoption / unknown coverage

## Improvement Direction

coreの入力を「content snapshot + declared base/head + explicit coverage」に限定したread-only assessment profileをprototype化する。Gitの収集・path mappingは外部adapterが担い、CodeFire側はopaque subject digestとprofileを受けてadvisory claimを返す。native importは別の明示的なmutation stepとして残す。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: native importを唯一の入口にする | semanticsが単純 | time-to-valueが遅く仮説検証を阻害 | reject |
| B: read-only adapter profileを競争実験する | 低リスクで独立価値を測れる | subject/coverage契約が必要 | experiment |

## Recommended Sequence

1. **Characterize**: 3種類の既存repositoryでsetup時間、signal、identity ambiguityを事前定義する。
2. **Separate**: native commit subjectとexternal snapshot subjectをschema上別unionにする。
3. **Implement**: filesystem-neutral fixture inputを読むread-only prototypeと薄いGit collectorを作る。
4. **Prove**: adapterなしのconformance vectorと、native/advisory混同を拒否するconsumer testを通す。
5. **Promote**: STR-X04の結果をproduct strategyまたは停止判断へ反映する。

## Acceptance Signals

- assessmentが対象repositoryを変更せず、native certificateを発行しない。
- 参加者がadvisoryとnative sealの違いを説明でき、first signalまでのコストを測定できる。
- adapterを削除してもcoreにGit ref/index/working-tree固有型が残らない。

## Verification Design

- Focused tests: external subject digest、coverage unknown、base/head mismatch、native/advisory claim type。
- Cross-boundary tests: adapter fixture → public input schema → CLI envelope → independent consumer。
- Scale or fault tests: rename、submodule相当、binary、大規模tree、unstable snapshotを明示的にUNKNOWNへするfixture。
- Observability: assessed files/Atoms、excluded/unknown、snapshot digest、adapter versionをbounded reportに出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFB-048は既存directoryのnative import UX、CFB-126はimport idempotencyの個別欠陥。本候補はread-only adoption claimの実験で重複しない。
- Required predecessor: Stage 2 public schema/conformance kernelとRVC-001のclaim区別。
- Can combine with: RVC-009、RVC-010。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- Git互換CLI、Git repository mutation、transparent dual-authorityを実装しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
