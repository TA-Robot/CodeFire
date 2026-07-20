# RVC-054: Test architecture decomposition

## Review Summary

| Field | Value |
|---|---|
| Status | promoted |
| Perspective | testing / architecture / correctness |
| Review kind | redesign |
| Priority | P1 |
| Confidence | confirmed |
| Roadmap stage | Stage 1 |
| Primary scope | `Rust CLI and Python fallback tests` |
| Promotion target | future CFR / ADR |

## Executive Judgment

feature-local testsとdocs test分離は前進しているが、Rustの`tests.rs`は約8.4千行、Python testは約3.5千行で、domain fixture・CLI contract・remote・fault testが巨大fileに集中する。責務別suiteと共有fixture builderへ段階分解し、private `main.rs` internalsへの結合を減らすべきである。

## Scope and Review Question

- In scope: test module ownership、fixture builders、unit/integration/contract/fault層、発見性
- Out of scope: test runner/CI provider、coverage percentage競争、product codeの同時全面refactor
- Question: invariant変更時に必要なtest setとfixtureをowner moduleから即座に特定できるか

## Evidence

- `crates/codefire-cli/src/main.rs` — `#[cfg(test)] mod tests`: CLI-wide test moduleがbinary crateのprivate implementationへ直接接続する。
- `crates/codefire-cli/src/tests.rs` — `use super::*`: lifecycle、remote、evidence、merge、schema、performance helperを一つのnamespaceへ取り込む。
- `crates/codefire-cli/src/tests/docs.rs` — docs/help/fallback/root-cause testsをsubmoduleへ抽出した良いprecedentがある。
- `crates/codefire-cli/src/http.rs` / `signatures.rs` / `view.rs` — feature-local `mod tests`がparser/algorithm境界を近接検証する。
- Test/fixture evidence: `tests/test_codefire_cli.py::CodeFireCliTests`も単一class/fileにremote server、format、installerまで集まり、cross-runtime corpusとは分離されていない。

## Strengths to Preserve

- private helperを無理にpublic化せずunit testできるRust構成。
- domain fixtureとgolden diff/merge outputが多く、behavior regressionの資産が厚い。

## Gap / Risk

### Observed gap

test分類がfile構造に反映されず、shared setup helperが末尾へ集中する。unit/integration/contract/fault/scaleの目的と実行costが見分けにくい。

### Failure or stagnation mode

変更者が巨大suiteへtestを追加し続け、必要fixtureを複製する。main抽出時にprivate symbol依存が大量に壊れ、architecture改善を妨げる。

### Affected invariants

- regression evidence discoverability
- independent workflow verification
- safe module decomposition

## Improvement Direction

`tests/{local,workflow,remote,view,recovery,compatibility,performance}.rs`とfeature-local unit testへ段階移動し、shared fixture builderはdomain vocabularyを持つtest support moduleへ置く。process-level public CLI testsとprivate unit testsを明示分離する。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: touch-when-related段階移動 | 小差分で継続的に改善 | 一時的に旧新構成が併存 | adopt |
| B: 全test一括移動 | 早く見た目が整う | merge conflictとbehavior変更混入が大きい | reject |

## Recommended Sequence

1. **Characterize**: testをowner/domain/layer/cost/fixtureでinventory化する。
2. **Separate**: shared fixture builderと最初のremote/recovery suiteを抽出する。
3. **Implement**: feature変更のたびに近いsuiteへ移し、new testの配置ruleを適用する。
4. **Prove**: test count/name/resultとgolden checksumが移動前後で一致する。
5. **Promote**: module-boundariesのtest配置と必要なCFRへ反映する。

## Acceptance Signals

- 新規testのowner/layerがfile pathから分かり、巨大root fileへ直接追加されない。
- shared fixtureがsealed/corrupt/remote/open stateを一貫して生成する。
- main/module抽出で無関係testの大量修正が不要になる。

## Verification Design

- Focused tests: test support builder自身のvalid/corrupt fixture assertion
- Cross-boundary tests: public binary CLI suiteとmodule unit suiteの同一golden
- Scale or fault tests: recovery/performance suiteを明示tag/domainへ隔離
- Observability: suite別test count/durationは補助、invariant coverage mapを主とする

## Dependencies and Duplication Check

- Existing CFB/CFR: fixed CFR-039は巨大test集中を記録したが、現状はroot Rust/Python fileがなお巨大。本候補は再open断定ではなく段階decomposition designで、具体的阻害が確認された場合のみfuture CFR化する。
- Required predecessor: 凍結batchのtest追加完了、RVC-051抽出順序
- Can combine with: RVC-051、RVC-055、RVC-056

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- test名やfileを動かすだけでcoverageが上がったと見なさない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
