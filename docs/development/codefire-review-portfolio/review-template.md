# CodeFire Improvement Review Template

Template version: 1

このテンプレートは、CodeFire本体のコードとシステム設計を多面的に評価し、次の改善候補を残すための形式である。対象は製品思想、domain model、invariant、Rust/Python実装、CLI contract、永続化、性能、security、UX、testabilityであり、devcontainer、CI、install/package配布、AI agent運用などの周辺環境は対象外とする。

ここで作る `RVC-*` は正式な `CFB-*` / `CFR-*` issueではない。現在の凍結batchが閉じた後に、既存issueとの重複を除き、必要なものだけをCFR、product spec、ADR、strategy experimentへ昇格するための **next discovery candidate** である。

## Rating Rubric

- **P0**: 現在のsupported pathでobject identity、sealed decision、authenticated mutation、recovery、またはuser workを損ない得る直接のcode evidence、open issue、seeded counterexampleのいずれかがある。
- **P1**: correctness/security/product claimへ重大な影響があり得るが、counterexampleやimpactが未確定、または現在はsafe-sideで将来driftを防ぐ設計課題である。
- **P2**: performance、maintainability、UX、adoptionを実質的に改善するが、直ちにfalse completionやlossを生まない。
- **P3**: optional optimizationまたは観測後にのみ判断する低緊急度候補である。
- **confirmed**: 現在のcode path、実fixture、またはopen issueで記述した挙動を直接確認できる。単にtest/corpusが無いだけではconfirmedにしない。
- **strong-hypothesis**: source構造からfailureが具体的に導けるが、seeded counterexampleまたはimpact measurementが未完了である。
- **exploratory**: product/user/architecture仮説であり、実装判断より先に比較実験が必要である。

---

# RVC-NNN: <レビュー題名>

## Review Summary

| Field | Value |
|---|---|
| Status | candidate / implemented / falsified / accepted_non_goal / promoted |
| Perspective | product / domain / architecture / correctness / security / performance / UX / testing / evolution |
| Review kind | preserve / refine / redesign / experiment |
| Priority | P0 / P1 / P2 / P3 |
| Confidence | confirmed / strong-hypothesis / exploratory |
| Roadmap stage | Stage 1 / Stage 2 / Stage 3 / Stage 4 / Stage 5 / Stage 6 |
| Primary scope | `<crate/module/command/spec>` |
| Promotion target | existing CFB/CFR / future CFR / product spec / ADR / strategy experiment / none |

## Executive Judgment

<現在の設計に対する中心判断を2〜4文で書く。何を維持し、何を変えるべきかを先に示す。>

## Scope and Review Question

- In scope: <今回評価する責務、データフロー、ユーザー体験>
- Out of scope: <周辺環境や意図的に扱わない機能>
- Question: <このレビューで答える設計上の問い>

## Evidence

- `<path>` — `<symbol/section>`: <実装または仕様から確認できる事実>
- `<path>` — `<symbol/section>`: <別の境界から確認できる事実>
- Test/fixture evidence: `<path or missing coverage>`

行番号だけに依存せず、fileとsymbol、型、command、object kind、schema名で根拠を識別する。

## Strengths to Preserve

- <CodeFireの価値または不変条件を支える既存の強み>
- <改善時に失ってはいけない性質>

## Gap / Risk

### Observed gap

<現在の設計・実装・仕様の不足または緊張関係。>

### Failure or stagnation mode

<放置した場合に起きる破損、誤解、性能劣化、採用停滞、保守負債。>

### Affected invariants

- <object identity / sealed commit / branch head / Fire / evidence / automation contract 等>

## Improvement Direction

<推奨する到達状態。単なる機能追加ではなく、責務境界とユーザー価値を説明する。>

### Option comparison

| Option | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| A | <...> | <...> | adopt / reject / experiment |
| B | <...> | <...> | adopt / reject / experiment |

## Recommended Sequence

1. **Characterize**: <現状固定test、counterexample、利用者実験>
2. **Separate**: <抽出する型、module、policy、schema、read model>
3. **Implement**: <最小のroot fixまたはprototype>
4. **Prove**: <deterministic test、complexity counter、conformance fixture>
5. **Promote**: <更新するspec/ADR/roadmap/issue>

## Acceptance Signals

- <機械的に確認できる正しさ・性能・互換性の条件>
- <ユーザーが理解・利用できることを示す条件>
- <既存の強みが維持されたことを示す回帰条件>

## Verification Design

- Focused tests: <追加するtest matrix>
- Cross-boundary tests: <crate/CLI/format/remote間の検証>
- Scale or fault tests: <大規模fixture、failpoint、corruption、race>
- Observability: <work counter、diagnostic、bounded output>

## Dependencies and Duplication Check

- Existing CFB/CFR: <IDと重複・包含関係>
- Required predecessor: <先に閉じるinvariantまたはroadmap gate>
- Can combine with: <同じroot fixで束ねられるRVC>

## Boundary / Non-Goals

- CodeFire本体へAI orchestration、agent marketplace、prompt managementを入れない。
- devcontainer、CI、install/package環境の改善へ論点を逸らさない。
- <この候補固有の非目標>

## Promotion Checklist

- [ ] 現在の凍結batchが閉じている
- [ ] 既存CFB/CFRとの重複を確認した
- [ ] code evidenceまたは再現fixtureを再監査した
- [ ] priorityとroadmap stageをownerが承認した
- [ ] 必要なら正式CFR、ADR、spec、strategy experimentへ昇格した

## Terminal Disposition Rules

- `candidate` はDiscovery中だけ許可し、portfolio completionを表さない。
- `implemented` はcurrent code、regression、closed issue/ADRがAcceptance Signalsを満たす場合だけ使う。
- `falsified` はseeded counterexampleまたはcharacterizationがGap/Riskを成立させなかった証拠を持つ。
- `accepted_non_goal` はcurrent product claim/roadmapが実装しない境界とfalsifierを明示した場合だけ使う。
- `promoted` は残差をformal CFB/CFRへexactly once mappingした状態であり、formal issueのCloseまでは製品完了を意味しない。
- terminal status、formal artifact、evidence path、root group、rationaleは`review-dispositions.json`へ同期し、
  Promotion Checklistを全項目完了する。project docs checkerが60 reviewのcoverageと整合を検査する。
