# Phase 1 Experiment Catalog

> **非正本 / exploration only**
>
> このcatalogは未実施を含む実験案である。記載された結果をCodeFireの現在状態として扱ってはならない。再現されたevidenceだけをtests、benchmarks、ADR、strategy、issueへpromotionする。

## 1. Experiment Record Contract

各実験の実行記録は最低限次を持つ。

```text
experiment_id:
hypothesis_id:
repository_revision:
dirty_tree_summary:
environment:
fixture_generator_or_path:
exact_commands:
parameters:
oracle:
raw_result_paths:
metrics:
unexpected_observations:
conclusion: supported | falsified | inconclusive
scope_limit:
follow_up:
```

結果を手書き要約だけで残さない。可能ならfixture seed、machine-readable result、stdout/stderr、exit code、filesystem before/after hashを残す。

## 2. Safety Rules

- real user repositoryではなくtemporary fixtureを使う。
- crash、corruption、collision、path escape、malicious command実験はisolated environmentで行う。
- destructive experimentはfixture rootをassertしてから実行する。
- secretに似たtest valueを使い、本物のcredentialを使わない。
- benchmarkとcorrectness testを分ける。速くてもoracle違反ならfailureである。
- dirty worktree上のbinaryを使う場合はrevision/source対応を明示する。
- failed/inconclusive resultも保存する。

## 3. Experiment Index

| ID | Target | Input/fixture | Oracle | Primary metrics | Priority |
|---|---|---|---|---|---|
| `EXP-FORMAL-001` | commit predicate alignment | 6 blocker dimensions × policy cases | verify/commit/certificate/JSON decisions agree | mismatch count | P0 |
| `EXP-FORMAL-002` | derived state | inconsistent branch/opened/active combinations | recompute returns correct state or explicit corruption | disagreement、latency | P1 |
| `EXP-FORMAL-003` | executable state model | generated operation/interleaving traces | implementation trace refines model | counterexamples、false positives | P1 |
| `EXP-TXN-001` | commit crash consistency | failpoint at every transaction phase | old/new valid state or explicit pending, never partial normal success | invalid states、RTO、orphans | P0 |
| `EXP-TXN-002` | idempotent retry | same/different payload and crash windows | same result or explicit conflict, no duplicate effect | duplicate count | P1 |
| `EXP-TXN-003` | concurrent remote mutation | simultaneous upload/MR apply | serializable fast-forward result | lost updates、contention | P1 |
| `EXP-STORE-001` | canonical identity | cross-runtime JSON corpus | expected bytes/full digest/public ID | mismatch count | P0 |
| `EXP-STORE-002` | short-ID collision | injected small digest prefix | explicit collision, never silent alias | silent alias count | P0 |
| `EXP-STORE-003` | GC root explanation | generated object graph/all root categories | no reachable deletion, every retention explained | false deletion、coverage | P1 |
| `EXP-PROTO-001` | compatibility | client/server/schema version matrix | supported pair passes, unsupported fails explicitly | silent acceptance | P1 |
| `EXP-SEMVER-001` | extractor drift | same source under extractor variants | unsafe semantic drift makes resolution stale | false active/false stale | P0 |
| `EXP-PERF-001` | shared history validation | `B` branches sharing `C` commits | same validated result/diagnostics | reads、time、RSS | P0 |
| `EXP-PERF-002` | complexity/work counters | x10 scale matrix | expected asymptotic work | work multiplier、p95 | P1 |
| `EXP-VCS-001` | Atom identity under refactor | rename/move/split/copy corpus | reviewed identity relation | precision/recall、Fire count | P1 |
| `EXP-VCS-002` | layered diff | seeded file/Atom/Trace defects | reviewer finds target defects without false identity | recall、review time | P2 |
| `EXP-VCS-003` | semantic merge policy | real/synthetic merge corpus | no false accept; measured recovery cost | precision、lead time | P1 |
| `EXP-GRAPH-001` | Fire propagation | typed/cyclic/high-fanout Trace Graphs | finite deterministic expected Fire set | amplification、precision/recall | P1 |
| `EXP-GRAPH-002` | validated graph types | one pilot consumer refactor | invalid graph cannot reach consumer without explicit validation | boundary count、complexity | P2 |
| `EXP-SEC-001` | path/symlink safety | traversal, symlink swap, platform separator corpus | no root escape | escape/false reject | P0 |
| `EXP-SEC-002` | auth/key lifecycle | replay/skew/revoke/rotate matrix | policy-consistent accept/reject | false accept/reject | P1 |
| `EXP-SEC-003` | verification trust boundary | malicious commands in isolated server | behavior matches declared deployment boundary | access/cleanup evidence | P0 |
| `EXP-SEC-004` | resource budgets | large/deep/slow inputs | bounded rejection and cleanup | peak resource、orphans | P1 |
| `EXP-SEC-005` | release provenance | source/build/install artifacts | installed binary maps to signed manifest/revision | provenance coverage | P1 |
| `EXP-OBS-001` | diagnostic phase model | injected failures across phases | independent operator/agent identifies phase/action | diagnosis success、log bytes | P1 |
| `EXP-REC-001` | recovery decision table | pending/stale/corrupt fixtures | safe correct next action, no user work loss | RTO、unsafe action | P0 |
| `EXP-TEST-001` | invariant coverage | selected P0 invariants and seeded mutants | mapped tests fail on invariant violation | kill/mapping rate | P1 |
| `EXP-TEST-002` | random lifecycle | generated command sequences | model/implementation relation holds | minimized counterexamples | P1 |
| `EXP-TEST-003` | mutation testing | core/store/automation predicates | suite kills semantic mutants | mutation score、runtime | P2 |
| `EXP-EVAL-001` | historical defect corpus | reproducible CFB/CFR snapshots | hidden expected fix/review outcome | replay rate、defect recall | P1 |
| `EXP-HCI-001` | diagnostic disclosure | flat vs grouped/progressive output | correct action on seeded scenarios | task time、miss rate | P2 |
| `EXP-HCI-002` | structured next actions | command-only vs contextual actions | safe action succeeds in correct target | success、stale/wrong target | P1 |
| `EXP-HCI-003` | Fire explanation | minimal vs cause/path/basis explanation | correct resolution/rationale | blind/incorrect resolution | P1 |
| `EXP-CONTEXT-001` | context retrieval | historical tasks with oracle file sets | required context within budget | recall@tokens、defects | P1 |
| `EXP-CONTEXT-002` | provenance summary | full source vs plain/provenance summary | task succeeds; stale/omission detected | success、stale detection | P2 |

## 4. P0 Protocol: Commit Predicate Alignment

### EXP-FORMAL-001

Linked hypothesis: `HYP-FORMAL-001`.

Goal:

仕様、core verification、commit rejection、sealed certificate、automation JSONが同じblocker semanticsを持つか確認する。

Dimensions:

```text
open required Fire
failed verification check
missing required Trace Link
stale Resolution
missing Evidence reference
duplicate Atom ID
```

Minimum matrix:

- 各dimension単独on/off。
- 全64組合せ。
- policy flagが存在するdimensionはrequired/disabledの両方。
- clean unchanged、changed consistent、corrupt baseをseparate controlにする。

Oracle:

1. domain verification resultとblocker detailsがpolicy通り。
2. commit decisionがverification resultと一致。
3. success certificate countersがverification inputと一致。
4. failure JSONの`blocking`、diagnostic kind、exit codeが一致。
5. text outputとJSONでsemantic blocker setが同じ。

Failure criteria:

- 一つでもverify passedなのにcommitがblock、またはverify failedなのにpublish。
- certificateがzero/nonzero counterを誤る。
- missing evidence等が仕様に存在せず実装だけでblockし、promotion先が未定。

Promotion:

- confirmed predicateを`docs/05_consistency_model.md`へ。
- generated matrixをcore/CLI regression testsへ。

## 5. P0 Protocol: Commit Crash Matrix

### EXP-TXN-001

Linked hypothesis: `HYP-TXN-001`.

Failure points:

```text
before pending marker
after pending marker
after each root object family
after commit object
before branch head publish
after branch head publish
after opened registry update
after active reset
before marker removal
after marker removal
```

For each point:

1. fixtureをsealed old head + commit-ready workspaceにする。
2. exact phaseでI/O errorまたはprocess terminationを注入する。
3. processを完全に再起動する。
4. branch record、opened registry、active files、marker、objectsをsnapshotする。
5. `doctor`、`status`、`branch list`、再commitを実行する。

Oracle:

- headはold valid commitまたはfully valid new commit。
- root missingのnew headは存在しない。
- partial stateをnormal successとして返さない。
- old headのままならnew objectsはorphanとして説明可能。
- new headならactive stateのstalenessを検出/回復可能。
- user working filesは失われない。

Metrics:

- invalid visible states。
- pending stateからdiagnosisまでのcommands/time。
- orphan count/bytes。
- retry success/idempotency。
- manual decisions。

## 6. P0 Protocol: Canonical Identity

### EXP-STORE-001

Linked hypothesis: `HYP-STORE-001`.

Corpus categories:

- object key permutation。
- nested arrays/maps。
- empty/null/absent。
- UTF-8、escape、non-ASCII、normalization candidates。
- integer boundaries and invalid/nonportable numeric forms。
- type/version changes。
- Python-created historical records and Rust-created records。

Each vector:

```text
logical payload
type tag
expected canonical bytes in hex
expected full sha256
expected current public ID
expected legacy ID where applicable
reader compatibility expectation
```

Oracle: all supported runtimes reproduce the exact expected sequence and IDs.

### EXP-STORE-002

Linked hypothesis: `HYP-STORE-002`.

Method:

- production SHA-256を破ろうとせず、test-only object ID prefix lengthまたはdigest providerを注入する。
- 1～2 hex prefixで異なるpayloadのsame short IDを生成する。
- first record保存後、second payloadを同じshort pathへ保存する。
- local store、remote copy、legacy readの各pathを実行する。

Oracle:

- explicit collision diagnostic。
- existing objectは不変。
- second payloadをfirst payloadとして成功返却しない。
- branch/commit graphへambiguous IDをpublishしない。

Failure severity: silent aliasはP0 defect候補として即CFBへpromotionする。

## 7. P0 Protocol: Shared Ancestry Performance

### EXP-PERF-001

Linked hypothesis: `HYP-VCS-001`.

Fixture matrix:

```text
B = 1, 10, 100, 1000 branches
C = 10, 100, 1000, 10000 commits where feasible
sharing = 0%, 50%, 99% shared ancestry
corruption = none, shared-tail missing, one unique-head invalid
```

Measure:

- elapsed time。
- object files opened/read。
- object JSON parses and hash validations。
- peak RSS。
- diagnostics and first reported failing branch/object。

Oracle:

- optimized and baseline versions return semantically identical branch rows/errors。
- corruption is never skipped because of memoization。
- shared case trends towardunion reachable objects rather than `B × C` reads。

Gate proposal must be based on measured hardware/environment; do not invent a universal millisecond threshold before baseline data exists.

## 8. P0 Protocol: Extractor Semantic Drift

### EXP-SEMVER-001

Linked hypothesis: `HYP-SEMVER-001`.

Variants:

- same content hash but selector/kind changes。
- derived Atom ID rule changes。
- parser begins recognizing a previously ignored symbol。
- parser bug fix changes nested owner。
- language/config version changes required links。

Procedure:

1. vN semanticsでFireをresolveする。
2. source textを変えずvN+1 semanticsでscan/verifyする。
3. resolution status、AtomIndex、Trace Graph、missing linksを比較する。

Oracle:

- review basisを変えるsemantic driftはactive resolutionをsilent reuseしない。
- irrelevant extractor implementation changeだけで全resolutionをfalse staleにしない。

Metric:

- false active、false stale、required manual re-review数。

## 9. P0 Protocol: Path Safety

### EXP-SEC-001

Linked hypothesis: `HYP-SEC-001`.

Corpus:

- `..` and repeated separators。
- absolute path and drive/prefix forms。
- URL percent encoding/double encoding。
- symlink to outside root。
- parent directory symlink swap between validation/write。
- dangling symlink and non-existent target。
- case-insensitive collision where environment supports it。
- branch/ref names that could become path components。

Run against:

- open/materialization。
- branch/opened/idempotency/lock records。
- remote org/app/branch paths。
- artifact/evidence refs。
- installer/completion paths。

Oracle:

- no read/write/delete outside fixture root unless an explicit external-artifact read contract permits read-only access。
- errors are path/config diagnostics, not panic。
- legitimate supported symlink cases are documented and tested separately。

## 10. P0 Protocol: Verification Trust Boundary

### EXP-SEC-003

Linked hypothesis: `HYP-SEC-003`.

Run only in disposable container/VM.

Malicious command attempts:

- read environment secrets represented by fake sentinel values。
- read/write outside project root。
- open network connection to local test listener。
- spawn child process surviving timeout。
- produce unlimited stdout/stderr。
- modify remote project state concurrently。

The experiment does not assume all attempts must currently fail. Its first oracle is that documented deployment/trust claims exactly match observed capability.

Decision outcomes:

- trusted-admin-only and clearly documented。
- isolated server process/container is mandatory。
- built-in sandbox/capability execution is required before a deployment class is supported。

Immediate security issue condition:

- observed capability exceeds documented trust boundary or exposes real secrets/user paths in normal deployment.

## 11. P0 Protocol: Recovery Decision Table

### EXP-REC-001

Linked hypothesis: `HYP-REC-001`.

Fixtures:

- pending before/after head publish。
- orphan immutable objects。
- active state missing/corrupt/stale。
- opened registry/marker mismatch。
- live lock、dead-owner lock、malformed lock、PID reuse candidate。
- missing root/parent object。
- valid object with wrong filename/record ID/hash。

Independent evaluator receives onlynormal diagnostics and documented recovery guidance, not fixture construction details.

Oracle:

- correctly classifies safe/unsafe/ambiguous。
- chooses no destructive automatic action for ambiguous cases。
- preserves working files and reachable objects。
- identifies required backup/validation before repair。

Metrics:

- diagnosis accuracy。
- correct next-action rate。
- commands/steps and elapsed time。
- unsafe action/data loss count。

## 12. P1/P2 Protocol Notes

### EXP-FORMAL-002

Generate all meaningful combinations of branch state, opened registry state, active state, working delta, Fire count, verification result. Compare stored labels with a candidate reducer. Measure both mismatch detection and clean-repo status latency.

### EXP-FORMAL-003

Model operations `open/edit/scan/extinguish/verify/commit/crash/retry/close`. Generate bounded interleavings, replay them against a filesystem fixture, and minimize divergence.

### EXP-TXN-002

Inject failure before/after effect and before/after idempotency result. Repeat same key/payload and same key/different payload. Assert no duplicate semantic effect.

### EXP-TXN-003

Start concurrent writers from the same expected head. Compare file-backed and HTTP behavior. Exactly one incompatible head mutation may succeed; all accepted histories must remain fast-forward valid.

### EXP-STORE-003

Generate typed object graphs with each candidate root category and pending marker. Compare GC mark result with an independent graph oracle and require retention reasons.

### EXP-PROTO-001

Build an explicit supported/unsupported client/server/object/schema matrix. Unsupported versions must fail with structured migration/capability diagnostics, not generic corruption or silent downgrade.

### EXP-PERF-002

Scale one variable at a time by approximately 10 while holding others fixed. Record both wall time and work counters; classify linear, near-linear, superlinear, or bounded-by-limit behavior.

### EXP-VCS-001

Create reviewed refactor pairs for rename, move, split, merge, copy, generated code, explicit and derived IDs. Score candidate identity separately from sealed identity.

### EXP-VCS-002

Seed file-only, Atom-only, Trace-only, and mixed defects. Compare flat and layered review packs with human/agent reviewers while hiding defect labels.

### EXP-VCS-003

Use historical/dogfood merges where available plus synthetic cases. Compare current conservative flow with any candidate limited auto-resolution; false accept is a hard failure.

### EXP-GRAPH-001

Generate direct, cyclic, high-fanout, high-depth, duplicate-edge, missing-node graphs. Compare Fire set/order/path to a small independent oracle.

### EXP-GRAPH-002

Pilot one graph consumer with `Parsed`/`Validated` types. Count removedruntime checks, compile-time rejected paths, added complexity, serialization friction.

### EXP-SEC-002

Test valid/expired/not-yet-valid/revoked keys, signer constraints, skew boundaries, nonce replay, rotation overlap, wrong actor/branch permissions for file and HTTP remotes.

### EXP-SEC-004

Independently drive request bytes, JSON depth, graph depth/entities, output bytes,wall time, and child processes beyond proposed budgets. Verify bounded cleanup and structured errors.

### EXP-SEC-005

Build/install from source and prebuilt candidates. Attempt to recover source revision, manifest digest, signing identity, SBOM, CLI version from the installed artifact.

### EXP-OBS-001

Inject known failures at each parse/validate/plan/lock/revalidate/write/render phase. Give bounded diagnostics to an independent operator/agent and score phase/cause/next-action accuracy.

### EXP-TEST-001

Select P0 invariants, map existing tests, then seed one semantic mutant per invariant. A mapping is credible only if at least one mapped test kills the relevant mutant.

### EXP-TEST-002

Generate lifecycle sequences with fixed seeds, compare against reference model, require automatic shrinking and ten consecutive reproduction attempts before accepting a counterexample.

### EXP-TEST-003

Limit first pilot to pure critical functions in core/store/automation. Record equivalent-mutant triage time separately from mutation score.

### EXP-EVAL-001

Select historical issues with reproducible evidence. Reconstruct pre-fix snapshots without exposing solution text to the evaluated lane. Record leakage risk and representativeness.

### EXP-HCI-001

Use the same seeded blocker scenarios with flat versus progressive/grouped diagnostics. Randomize order and measure action correctness before preference.

### EXP-HCI-002

Compare command-string-only next actions with target/precondition/mutation/expected-result actions. Include stale state between suggestion and execution.

### EXP-HCI-003

Give users/agents Fires with minimal versus cause/path/basis explanation. Judge resolution type, rationale, evidence, and later stale/incorrect status.

### EXP-CONTEXT-001

For historical tasks, define oracle required/optional/forbidden files. Compare full repo, current static packs, lexical/source-map retrieval, and typed dependency graph under equal token budgets.

### EXP-CONTEXT-002

Compare full source, plain summary, and provenance-bearing summary. Mutate a source after summary generation to test stale detection and negative-constraint preservation.

## 13. Evidence Acceptance

一回の成功で一般化しない。

- correctness property: deterministic fixtureで再現し、regression test化する。
- performance claim: scale curve、environment、varianceを残す。
- HCI/AI claim: task/sample/persona/model条件を限定する。
- security claim: threat modelとdeployment classを限定する。
- compatibility claim: version pairとfixture corpusを列挙する。

`supported`へ変更できる最低条件:

1. oracleが実験前に定義されている。
2. exact input/command/seedを再現できる。
3. alternative explanationを一つ以上検討している。
4. failure/inconclusive runを隠していない。
5. scope limitが明記されている。

## 14. Promotion

- regressionを発見した場合: CFB/CFR + focused test packet。
- architecture decisionを支持した場合: ADR + relevant product/development docs。
- performance claimを支持した場合: repeatable benchmark + threshold ownership。
- HCI/context claimを支持した場合: AI/automation protocol + versioned eval result。
- security boundary violationを発見した場合: exploration完了を待たずsecurity responseへ移す。

## 15. Execution Status

- この文書作成時点で上記実験は未実施。
- existing testが一部propertyを覆っていても、このcatalogの全oracleを満たす確認はまだしていない。
- 実施結果はこのstatusを更新するだけでなく、machine-readable/raw evidenceへのpathを必ず記録する。
