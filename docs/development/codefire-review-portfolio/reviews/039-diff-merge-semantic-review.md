# RVC-039: diffとmergeで共有するsemantic review modelを設計する

## Review Summary

| Field | Value |
|---|---|
| Status | accepted_non_goal |
| Perspective | product / domain / architecture / performance |
| Review kind | experiment |
| Priority | P1 |
| Confidence | strong-hypothesis |
| Roadmap stage | Stage 3 |
| Primary scope | `view/semantic_diff.rs` / `main.rs::build_merge_result` |
| Promotion target | strategy experiment |

## Executive Judgment

file、Atom、Trace、policy impact、binary、semantic conflict candidateを分け、自動semantic解決をしない現在方針は維持すべきである。一方、two-way diff impactとthree-way merge conflictが別modelで、同じAtom/Trace/policy changeを異なる粒度で説明する。共通ChangeAssessmentを実験し、merge判断とreview-packが同じ根拠・confidence・omissionを再利用できるか測るべきである。

## Scope and Review Question

- In scope: two-way/three-way semantic delta、Atom/Trace/policy/Fire impact、conflict candidate、review evidence
- Out of scope: semantic conflictの自動解決、Git完全互換、language-specific AST merge
- Question: diffで示した意味的影響がmerge dry-runでも同じ分類・根拠になり、false confidenceを増やさないか

## Evidence

- `crates/codefire-cli/src/view/semantic_diff.rs` — `diff_impact`: left/right policyのmissing linksを比較し、synthetic scanからexpected Firesとnext actionsを作る。
- `crates/codefire-cli/src/main.rs` — `semantic_conflict_candidates`: baseに対しsource/target双方で同じAtom IDが変わり、content hashが異なる場合だけcandidateにする。
- `crates/codefire-cli/src/main.rs` — `build_merge_result` / `apply_merge_result`: file/binary conflictとsemantic candidateを計画し、apply後にstateへ保存してopen-burning scanへ進める。
- Test/fixture evidence: `crates/codefire-cli/src/tests.rs` と `view/semantic_diff.rs` testsはAtom/Trace diff、review-pack、merge candidateを個別検査するが、diff assessmentとmerge assessmentのmetamorphic一致を検査しない。

## Strengths to Preserve

- semantic conflictをcandidateとして提示し、人間の判断を越えて自動解決しないこと。
- diff algorithm、rename/binary bounds、Atom/Trace/policy impactを明示的option/sectionに分けること。

## Gap / Risk

### Observed gap

diff impactはtwo-way synthetic scan、merge candidateはthree-way overlapping Atom hashという別taxonomyで、Trace Linkやpolicyのconcurrent conflict、confidence、coverage/omissionを共通表現しない。

### Failure or stagnation mode

review-packでは安全そうに見えたchangeがmerge dry-runで別分類になる、またはsemantic candidateがないことをsemantic safetyと誤解する。新しいsemantic ruleを二箇所へ実装してdriftする。

### Affected invariants

- review claim honesty
- Fire/policy impact consistency
- merge safety
- bounded semantic analysis

## Improvement Direction

ChangeAssessmentをfile delta、Atom delta、Trace delta、policy delta、predicted blockers、three-way conflict candidates、confidence/coverageへ分ける。two-way diffとthree-way mergeは同じprimitive deltaを使い、mergeだけbase/source/target relationを追加する。各candidateはevidence rootsとrule IDを持ち、unknown/omittedをcleanへ変換しない。

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A: shared assessment primitivesをprototype | review/mergeの説明とrule再利用を改善 | model複雑化とfalse-positive評価が必要 | experiment |
| B: mergeへdiff JSONをそのまま埋め込む | 実装が早い | two-wayとthree-way意味差を隠す | reject |

## Recommended Sequence

1. **Characterize**: rename、concurrent Atom、Trace/policy change、binary、missing rootのgolden corpusを作る。
2. **Separate**: primitive delta、impact prediction、conflict classification、renderを分ける。
3. **Implement**: read-only review-pack/merge dry-runでprototypeしapply semanticsは変えない。
4. **Prove**: metamorphic test、Git fixture比較、precision/recall、work countersで評価する。
5. **Promote**: accepted taxonomyだけをproduct spec/CFRへ昇格し、自動解決は非目標のままにする。

## Acceptance Signals

- 同じbase/source pairのdiffとmerge assessmentが共通deltaで一致する。
- unknown/omitted analysisは明示され、no candidateをno conflict保証として表示しない。
- representative corpusでaccepted precision/recallとbounded work budgetを満たす。

## Verification Design

- Focused tests: Atom add/remove/change、Trace/policy concurrent delta、confidence/coverage。
- Cross-boundary tests: diff→review-pack→merge dry-runのmetamorphic/golden suite。
- Scale or fault tests: large manifests/shared history、rename candidate cap、corrupt/missing root。
- Observability: rule ID、candidate count、omitted sections、object reads、graph visitsを出す。

## Dependencies and Duplication Check

- Existing CFB/CFR: CFR-025〜029、088/089はdiff/merge algorithm・boundsをfixed済み。CFB-118はsemantic candidate時のapply suggestionをfixed済み。本件は共通semantic assessmentの実験で非重複。
- 2026-07-12 safety split: active merge conflict metadataがRust committable decisionへ届かないconfirmed bypassだけをCFR-162/ADR-014へ昇格した。shared ChangeAssessment、precision/recall、diff/merge taxonomyは本RVCのStage 3 candidateとして残し、P0 fixへ混ぜない。
- Required predecessor: Stage 1 sealed/read performance gate、RVC-038のsubject provenance。
- Can combine with: RVC-035のaction mode、RVC-047/048のgraph scale review、RVC-058のcorrectness work counters。

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- semantic conflictを自動解決・自動承認しない。

## Promotion Checklist

- [x] 現在の凍結batchが閉じている
- [x] 既存CFB/CFRとの重複を確認した
- [x] code evidenceまたは再現fixtureを再監査した
- [x] priorityとroadmap stageをownerが承認した
- [x] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した
