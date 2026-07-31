# Development-Flow KGI/KPI Round-3 Independent Rereview (Claude)

Review date: 2026-07-31

Reviewer: Claude Opus 5 (`claude-opus-5`), fresh independent reviewer

Exact candidate: `753c8db103579d021795d3fe6a97c5d45d4a111a`

Candidate tree: `213e16a033aeb2e8eb3599be4ffe725b1d7ad344`

Rejected predecessor (base): `0944bf3deba9e753132b3f71716b5c009a8fc30f`

Reviewed diff: `0944bf3deba9e753132b3f71716b5c009a8fc30f..753c8db103579d021795d3fe6a97c5d45d4a111a`
(11 files, **+1366 / −189**; the repair is `d003a666bcbf7cbec804efcf4e321f0a0ae8e1dd`, the head is a
metadata follow-up that records that hash)

**Decision: `REQUEST_CHANGES`.**

This is the strongest candidate in this lineage and most of it is real. I reproduced **every** round-2
executable falsification on the predecessor tree itself, with inputs I built rather than the
candidate's, and every one of them now rejects with a specific stable code. The frozen 55/15 tables
are byte-identical to the predecessor, so nothing was closed by moving a goalpost. The cohort
derivation in particular is genuinely load-bearing: my own held-out cohort — my own arm values, my own
median and Tukey-hinge arithmetic, never importing the candidate's test module — validates clean, and
eight independent attacks on it (duplicate provider rows, unit substitution, member/sample
permutation, arm omission, sub-floor threshold, cross-lane cost re-attribution, padded cross-provider
total) all reject.

It is still not the bound standard. Round 2 made R1a–c, R2, R3 and R4 `MUST`, and the ticket's rule is
that any partially-closed row independently requires `REQUEST_CHANGES`. **R1c is not closed.** On this
exact candidate, through the production entry point:

- `KGI-DEV-004` reaches **`achieved` with zero findings** by declaring `MET-AI-075` and
  `MET-FLOW-037` as `measured 0` and appending two author-written strings —
  `measurement:MET-AI-075` and `measurement:MET-FLOW-037` — to one unrelated in-window `validation`
  event whose own summary reports a 33-test module run (§3 R1c-a). The candidate's own policy text
  asserts the opposite outcome for this exact case;
- `MET-FLOW-027` reaches **`1.0`** by *deleting* citations rather than producing evidence: the
  derivation is verifiable references over all distinct references, both chosen by the author
  (12/38 on the committed record, 12/12 after stripping) (§3 R1c-b);
- artifact identity is still splittable: padding a commit-bearing artifact with a second
  already-declared **commit** reference, or padding any commit-free artifact kind with an
  already-declared `event:` reference, yields zero findings (§3 R6);
- lifecycle completion's closing half is still discharged by five synthetic events citing only the
  policy document; the published claim that this is refused as
  `lifecycle-stage-evidence-unverifiable` is false (§3 L);
- the new `validation-instant-before-capture` lower bound applies to **active records only**; a
  backdated instant on a `final` or `superseded` record passes with no diagnostic (§3 R5').

The candidate diff also introduced four defects of its own: two honestly distinct deliverables inside
one commit are now rejected as duplicates (§4 N1), a structural finding flood now suppresses the
entire semantic layer (§4 N2), an honest seal fails on one merge topology (§4 N3), and five published
claims are stronger than the code (§11).

This decision binds only exact candidate `753c8db103579d021795d3fe6a97c5d45d4a111a`. It does not
authorize integration, CFR-168 Close, Install, Dogfood, or Retrospect.

---

## 1. Exact Subject And Bindings

### 1.1 Bound authorities

| Authority | Exact identity | Verification |
|---|---|---|
| Exact candidate | `753c8db103579d021795d3fe6a97c5d45d4a111a` | `git rev-parse HEAD`; subject `docs(ai): record the round-3 repair commit in its result report`, committed `2026-07-31 09:20:55 +0000`; tree `213e16a033aeb2e8eb3599be4ffe725b1d7ad344` |
| Round-3 repair commit | `d003a666bcbf7cbec804efcf4e321f0a0ae8e1dd` | `git rev-parse 753c8db^`; subject `fix(ai): derive KGI/KPI decisions and seals instead of accepting declarations`; carries 1365 of the 1366 added lines |
| Rejected predecessor | `0944bf3deba9e753132b3f71716b5c009a8fc30f` | `git rev-parse d003a66^` |
| Bound round-2 report | `5760c98:docs/development/ai/swarm-results/development-flow-kgi-kpi-r2-rereview-claude-fallback-20260731/review.md` | SHA-256 `54a4244a8e563b48b2f086a51433ea858996c08b0104c0309f2c8445ce6706b2`, recomputed here; matches the digest the round-3 packet declares |
| Round-2 decision | `REQUEST_CHANGES`; R1a–c, R2, R3, R4 `MUST`; R5, R6, R7 and lifecycle `SHOULD`; 3 newly closed, 17 still partial, 13 with an executable falsification | §3, §6, §12 of that report |
| Round-3 result report | `docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r3-20260731.md` | SHA-256 `eedcf6931ca702223678273cad215b0d0afd4b53760f56906cfe7e31cc328fd5` |
| Root-cause audit | `abfee6e:docs/development/ai/swarm-results/cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md` | Read for the enumerated-corpus trap (§5 K2, §6.2), I1 predecessor re-derivation, I2 diff-scoped new surface, I4 oracle admissibility |

The round-2 report, the round-3 packet and the audit were read with `git show` and never edited.

### 1.2 Candidate blobs reviewed

| Path | Blob at candidate | Added / removed |
|---|---|---:|
| `docs/development/tools/test_ai_tools.py` | `c52633013e05` | 610 |
| `docs/development/tools/ai_contracts.py` | `2c135b63e60c` | 591 |
| `docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r3-20260731.md` | (new) | 155 |
| `docs/development/ai/development-flow-kgi-kpi.md` | `127a335e95e9` | 103 |
| `docs/development/ai/flow-metrics/README.md` | `449814c…` | 35 |
| `docs/development/ai/schemas/development-flow-record.v1.schema.json` | `fa83fb2fd4f6` | 22 |
| `docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json` | `05a92c9186b5` | 16 |
| `docs/development/tools/check_ai_contracts.py` | `918cac8f3ae9` | 14 |
| `docs/development/ai/schemas/README.md` | `47f11c9…` | 4 |
| `docs/development/project-management.md` | `a3ca5a8…` | 4 |
| `docs/development/history.md` | — | 1 |

No product source, Rust or Cargo artifact, issue state, devcontainer file, agent wrapper, install,
dogfood or demo path is touched by the candidate, and none was touched by this review.

### 1.3 Constraints honored

No implementation edit; no Cargo or Rust command; no install, dogfood or demo; no Codex; no Fugu; no
subordinate agent or fallback; no network command. Every probe is a deterministic Python or Git
command under `/tmp`. The predecessor was materialised as a read-only `git clone --shared` at
`/tmp/r3rev/predrepo` (HEAD `0944bf3`) so its behaviour could be derived from its own tree rather than
from the candidate's account of it. The only repository file this review writes is its own `review.md`.

### 1.4 Probe artifacts

1,748 lines of drivers, all outside the repository and all re-runnable against this exact candidate.

| SHA-256 | Driver | Covers |
|---|---|---|
| `14891ad15dcdd381facd2d96182c0e4bc3f354c55e3a24924574f1612b26a62c` | `/tmp/r3rev/replay.py` | baseline, 55 semantic, 15 structural, 23 probes, rebuilt P6/P9 |
| `0a925b00455863ba77e5c206826173d47ec50023f8e9091f63d05699af5eae0b` | `/tmp/r3rev/resid.py` | all 13 round-2 residuals, both trees; my own held-out cohort fixture |
| `71bceb3b2371505d2180526a23c21b2a99d4fef5843dcf59c99f538b00cef016` | `/tmp/r3rev/seal.py` | 11 seal probes on real Git histories, both trees |
| `40b0af778f76d3aa7e605ed60625ae213de342b4738af87f221d090967e56acf` | `/tmp/r3rev/adv.py` | numeric boundary, guardrails, throughput, artifacts, lifecycle, instants, diagnostics, 8 cohort attacks |
| `a8292473f209e66382562282a3be2d3386380dbbb9a7e06223b093466029c8ef` | `/tmp/r3rev/adv2.py` | composite KGI declaration, completeness by omission, worst-case bound |
| `49ca71ad39560e09a800410ccf3dde73de590030c911cd29aefe2d452830f71a` | `/tmp/r3rev/adv3.py` | KGI-DEV-004 declaration, commit-free artifact padding, ordered lifecycle, head counting |

---

## 2. Method And Commands Actually Run

### 2.1 Method

The root-cause audit `abfee6e` names three obligations this lineage failed before. I discharged each
explicitly rather than treating the 13 green counterexamples as the invariant.

1. **I1 — predecessor re-derivation.** The predecessor's behaviour is derived by importing
   `ai_contracts` **from the `0944bf3` tree** and running it against a real repository whose `HEAD` is
   `0944bf3`. No predecessor behaviour in this report is taken from the candidate's tests, from the
   round-3 packet, or from the round-2 report.
2. **I2 — diff-scoped new surface.** I computed the symbol set present in the candidate and absent
   from the predecessor: 13 new functions in `ai_contracts.py` (`_parse_json_float`,
   `_parse_json_int`, `parse_repository_json`, `unrepresentable_number_paths`, `git_hash_object`,
   `git_commit_is_reachable`, `git_path_history`, `earliest_final_publication`,
   `artifact_subject_digest`, `evidence_completeness_counts`, `_reference_is_repository_verifiable`,
   `lane_elapsed_seconds`, `lane_measured_cost`) and one removal (`non_finite_paths`), plus one
   removal in `check_ai_contracts.py` (`_reject_flow_constant`). **Every one of the 13 carries its own
   probe in §4/§5**, independent of finding closure. Coverage is stated per symbol in §4.4.
3. **I4 — oracle admissibility.** The candidate's suite is treated as evidence about the implemented
   contract only. The 55/15 corpus is replayed but is explicitly *not* the invariant; every
   substantive conclusion below rests on a probe I wrote, and the cohort, seal and artifact positives
   are reconstructed with my own values, my own median/Tukey-hinge arithmetic and my own
   canonical-JSON SHA-256 rather than through `median_of`, `dispersion_of`, `sha256_json` or the
   candidate's fixtures.
4. Read the full diff surface, then every repository gate, then the frozen replay, then the 13
   residuals on both trees, then five held-out adversarial batteries, then the exemplar and the
   published claims.

### 2.2 Repository gates

| Command | Result |
|---|---|
| `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/test_ai_tools.py` | `Ran 129 tests`, `OK` (4.979 s) |
| `python3 docs/development/tools/check_ai_contracts.py` | `AI contract check passed.` + `- flow validation instant: not injected; the freshness control did not run` |
| `... --validation-instant 2026-08-05T00:00:00Z` | 4 errors: 1 × `active-record-review-overdue`, 3 × `in-progress-measurement-overdue`; instant echoed |
| `... --validation-instant 1970-01-01T00:00:00Z` | 1 × `validation-instant-before-capture`; instant echoed |
| `CODEFIRE_FLOW_VALIDATION_INSTANT=1970-01-01T00:00:00Z ...` | same refusal through the environment form |
| `CODEFIRE_FLOW_VALIDATION_INSTANT=2026-07-30T09:51:56Z ...` | passed; instant echoed |
| `... --json --validation-instant 2026-08-05T00:00:00Z` | `{"findings": …, "ok": …}` — **no instant key, no echo** (§11) |
| `python3 docs/development/tools/check_project_docs.py` | passed; **566** Markdown files (the packet says 565 — §12 C4) |
| `python3 docs/development/tools/check_ai_development.py --timeout 180` | `PASS` for all seven components (`acceleration_docs`, `contracts`, `project_docs`, `source_docs`, `runtime`, `tool_tests`, `validation_capabilities`) |
| `git diff --check` | clean |
| `git status --porcelain` | clean before and after the review, apart from this report |

### 2.3 Frozen-corpus integrity

Before replaying anything I diffed the frozen tables between the two trees:

```
git show 0944bf3:…/test_ai_tools.py  vs  git show 753c8db:…/test_ai_tools.py
  FLOW_SEMANTIC_MUTATIONS  -> byte-identical (55 entries)
  FLOW_STRUCTURAL_CONTROLS -> byte-identical (15 entries)
```

**No frozen expectation was weakened, reworded or removed to make the repair pass.** The probe table
grew from 17 to 23 (`P18`–`P23` added). This is the single most important integrity check in a
lineage whose primary root cause was the enumerated-corpus trap, and the candidate passes it.

---

## 3. Round-2 Residuals: Predecessor Re-Derivation And Current State

Every row was executed twice with the **same** input: once against `ai_contracts` imported from the
`0944bf3` tree with `HEAD=0944bf3`, once against the candidate. Where the round-2 report used the
candidate's own cohort fixture, I substituted my own (§5.3) so neither result depends on a fixture the
implementer controls.

### 3.1 The 13 executable residuals

| Residual | On predecessor `0944bf3` (re-derived here) | On candidate `753c8db` |
|---|---|---|
| R1a identical arms, self-declared threshold `0.0` | **0 findings** | `cohort-improvement-floor-violated`, `cohort-improvement-unmet`, `schema-minimum`, `kgi-target-not-met` |
| R1b ×10 inflated baseline samples, every aggregate kept consistent | **0 findings** | `cohort-lead-time-not-derived`, `cohort-cost-not-derived`, `kgi-target-not-met` |
| R1c-i throughput declared as 3 | **0 findings** | `throughput-not-derived` |
| R1c-ii evidence completeness declared 7/7 | **0 findings** | `evidence-completeness-not-derived` |
| R1c-iii guardrail zero on a foreign gate-failure event | **0 findings** | `zero-target-audit-event-subject-mismatch` |
| R2 finite 401-digit integer `uncertainty`, record route | **`CRASH: OverflowError`** | `numeric-magnitude-out-of-range`, `schema-numeric-magnitude` |
| R2 `1e-400` at the file boundary | **decoded to exact `0.0`** | refused (`numeric literal underflows to zero`) → `invalid-json` |
| R2 `1e400` at the file boundary | **decoded to `inf`** | refused (`numeric literal is not finite`) |
| R2 `10**400` integer literal at the file boundary | **decoded exactly** | refused (`exceeds the documented boundary`) |
| R3 edit a sealed record, commit, move `sealed_in_commit` | **0 provenance findings** | `final-record-provenance-mismatch` |
| R3 seal against a dangling `commit-tree` | **0 provenance findings** | `final-record-provenance-unreachable` (ancestor-of-`HEAD` rc=1 confirmed) |
| R4 one-way cross-batch `superseded_by` on an active record | **0 findings** | `supersession-cross-batch`, `superseded-active-record-status`, `supersession-no-head` |
| R4 batch with zero unsuperseded heads | **0 findings** | `supersession-no-head` |
| R5 `--validation-instant 1970-01-01T00:00:00Z` on the active record | **0 findings** | `validation-instant-before-capture` |
| R6 artifact padded with one already-declared **lane** reference | **0 findings** | `duplicate-artifact-subject` |
| R7 700 malformed events | **1913 findings** against a published 512 bound | **513** findings incl. `finding-limit-reached` |
| Lifecycle completion on an active record | **0 findings** | `lifecycle-complete-requires-final-record` |

All 13 (17 rows, several residuals carrying more than one falsification) reproduce on the predecessor
and reject on the candidate. Two further round-2 details are also fixed: a dangling `superseded_by`
now returns the direction-specific `unresolved-superseded-by-record` where the predecessor returned
`unresolved-supersedes-record` for both directions, and a formatting-only uncommitted edit to a sealed
record is now `final-record-uncommitted-edit` where the predecessor returned nothing.

### 3.2 What is still open — severity-ordered

#### MUST-BLOCKER R1c — terminal KGI outcomes are still reachable by declaration

**R1c-a. `KGI-DEV-004` reaches `achieved` with zero findings.** The new gate requires a zero-target
row's audit event to name `measurement:<that metric>` in its own `evidence_refs`
(`ai_contracts.py:3489`). That name is a string the author writes. Starting from the committed record:

| Step | Edit |
|---|---|
| 1 | `MET-AI-075` → `status: measured`, `value: 0`, window `08:21:00Z–09:51:55Z`, `evidence_refs: [policy doc, event:mr-list-focused-first-pass-failure-20260730-091908]` |
| 2 | `MET-FLOW-037` → the same treatment |
| 3 | append `measurement:MET-AI-075` and `measurement:MET-FLOW-037` to that event's `evidence_refs` |
| 4 | `KGI-DEV-004` → `status: achieved` |

```
codes added relative to the clean baseline: NONE
KGI-DEV-004 status in the record: achieved
the cited event: kind=validation stage=implement
  summary "The exact committed-tree focused module ran 33 tests in 62.057 seconds and repor…"
control: drop the event reference -> zero-target-audit-event-required
```

A module test run is not an audit of silent model downgrades or Codex quota stoppages. The control
proves the gate is live and that the bypass costs exactly one string per guardrail. The `measurement:`
scheme is validated only for existence (`ai_contracts.py:4446`: the target must be a metric id in the
record), so nothing constrains which event may name which metric. This is the same false-achievement
path F11/F-S9/F-S13 were opened for, one indirection further out.

The candidate's own policy text asserts the opposite for this exact input
(`development-flow-kgi-kpi.md`): *"An unrelated in-window gate failure observes nothing about escaped
defects, quota stoppages, or silent downgrades, so citing one is
`zero-target-audit-event-subject-mismatch`."* Citing one **and naming the metric on it** is clean.

**R1c-b. `MET-FLOW-027` reaches `1.0` by omission.** The derivation is the record's
repository-verifiable references over its distinct references (`evidence_completeness_counts`,
`ai_contracts.py:3718`). Both numbers are author-chosen, because the denominator is *which references
the author decided to write*, not a required set:

```
derived on the committed exemplar:                    12/38
after deleting every non-commit:/doc: citation:        12/12  -> declared 1.0, accepted
```

`event:`, `lane:`, `prefix:`, `measurement:`, `artifact:` and `cohort:` references are never
verifiable, so citing fewer internal references raises "required evidence completeness". The metric
table row was honestly rewritten to say what the code does, but the `KGI-DEV-001` and `KGI-DEV-002`
target statement still reads *"complete required evidence"*, and `flow-metrics/README.md` still
presents completeness as one of the values that are *"recomputed, never declared"*.

**Honest qualification, which lowers this from a full false-achievement path to a narrowing:** the two
mechanisms interfere. Any zero-target audit needs an `event:` reference on the row and a
`measurement:` reference on the event, both unverifiable, so a record cannot simultaneously hold
`MET-FLOW-027 == 1` and a discharged guardrail. I therefore could **not** drive `KGI-DEV-001` or
`KGI-DEV-002` to `achieved` on this candidate — they fail with `kgi-target-not-met` on
`MET-FLOW-027=0.2857… fails == 1`. That interference is accidental, not designed, and it does not
protect `KGI-DEV-004`, which does not require `MET-FLOW-027`.

**R1c-c. Throughput derives from events, which are also authored.** `MET-FLOW-028` must equal the
in-window `integration` events the record records. Three synthetic `integration` events whose only
evidence is the policy document, with `value: 3`, produce **zero findings**. The derivation is real —
the number can no longer float free of the event list — but "trusted integration" is not verified:
no gate result, commit, or trusted-signal binding is required of the counted events. This is a genuine
tightening (the value is now bound to a structure the reader can audit) and a genuine remaining gap.

#### SHOULD-BLOCKER R6 — artifact identity is still splittable, two ways

`artifact_subject_digest` (`ai_contracts.py:2084`) digests the sorted commit set, "falling back to its
full reference set only when it binds no commit". Both halves of that rule leak.

| Probe | Result |
|---|---|
| R6 round-2 case: pad the twin with an already-declared **lane** reference | `duplicate-artifact-subject` — **closed** |
| Control: two artifacts with identical reference sets, different kinds | `duplicate-artifact-subject` — closed |
| **D1: pad a commit-bearing twin with a second already-declared `commit:` reference** | **zero findings** |
| **D5: two commit-free artifacts (`counterexample`, `report`), one padded with an already-declared `event:` reference** | **zero findings** |

D1 leaves commit `29c421fc…` bound by two artifacts of different kinds in the same lane. D5 works for
every kind outside `ARTIFACT_COMMIT_KINDS = {source_patch, test_patch, docs}` — that is `report`,
`decision`, `counterexample`, `validation_result` — because for those the subject is the citation list
again, which is exactly the input round 2 said identity must not depend on. The published claim
enumerates "lane, event, or prefix" and silently omits `commit:`, the one scheme that still works for
patch artifacts.

#### SHOULD-BLOCKER L — the lifecycle closing half is still synthetic-satisfiable

`check_lifecycle` now requires `status: "final"`, an allowed event *kind* per closing stage
(`LIFECYCLE_STAGE_EVENT_KINDS`), and repository-verifiable evidence on the qualifying event. On the
committed record, five appended events — `review_decision`/implement, `integration`/close,
`validation`/install, `validation`/dogfood, `lifecycle_complete`/retrospect, each citing **only**
`doc:docs/development/ai/development-flow-kgi-kpi.md` — produce exactly one finding:
`lifecycle-complete-requires-final-record`. No `lifecycle-complete-missing-stage`, no
`lifecycle-stage-evidence-unverifiable`. On a `final` record that last barrier is gone.

The cause is that a `doc:` reference **is** repository-verifiable by
`_reference_is_repository_verifiable`, so the policy document satisfies the very check whose message
says *"must bind repository-verifiable evidence, not a policy citation"*, and whose policy row says
*"binding repository-verifiable evidence rather than a policy citation"*. The branch is not vacuous —
switching those citations to `event:` references does raise
`lifecycle-stage-evidence-unverifiable` — but it does not do what it is published to do.

Two real constraints did land and I confirmed them: `decision` is an allowed kind for every closing
stage, yet a separate rule still requires a `review_decision` **and** an `integration` event at or
before the earliest completion, so four `decision` events alone raise
`lifecycle-complete-before-prior-stage`.

#### SHOULD R5' — the validation-instant lower bound is status-conditional

`validation-instant-before-capture` lives on the active-record freshness path
(`ai_contracts.py:2438`). It therefore does not exist for any other status:

| Record status | `--validation-instant 1970-01-01T00:00:00Z` |
|---|---|
| `active` | `validation-instant-before-capture` |
| `final` | **no instant finding** (only the record's own unrelated shape findings) |
| `superseded` | **no instant finding** |
| directory route, either status | clean — the bound is a record-route check only |

The policy publishes the rule without that scope: *"an instant earlier than the record's own
`captured_at` cannot evaluate freshness at all and is refused"*. The echo mitigates this in the human
output, and freshness matters least for a sealed record, which is why this is a `SHOULD` and not a
`MUST` — but the claim as written is broader than the code.

---

## 4. Diff-Scoped Review Of The New Surface (audit obligation I2)

These are findings about code the candidate introduced, scored independently of whether any bound
finding closed. **All four were created by this candidate diff.**

### N1 — Two honest deliverables inside one commit are now rejected

```
two artifacts in one lane, different kinds, both citing the same single commit
  -> duplicate-artifact-subject
```

A repair that lands a source patch and its counterexample in one commit can no longer declare both as
artifacts. Round 2 asked for identity "derived from the deliverable (its commit/path set)"; the
implementation uses the commit set only, so commit granularity is now forced to equal deliverable
granularity. This is a false positive on honest records, and it is the mirror image of §3 R6: the same
rule is simultaneously too strict (N1) and too loose (D1/D5).

### N2 — A structural finding flood suppresses the entire semantic layer

`validate_development_flow_record` now truncates and **returns** when the structural layer alone
exceeds `MAX_RECORD_FINDINGS` (`ai_contracts.py:4481`).

```
600 malformed events + MET-FLOW-028 declared as 3
  candidate: 3 distinct codes, finding-limit-reached present, throughput-not-derived ABSENT
  predecessor: semantic layer ran; 1913 findings reported
```

The record is still rejected, so this is not a false-accept, and the bound itself is the round-2 R7
requirement. But the semantic finding is now invisible, so a record can hide *which* rule it breaks
behind structural noise. Bounding after the semantic layer, or reserving a slice of the budget for it,
would keep both properties.

### N3 — An honest seal fails on one merge topology

`earliest_final_publication` walks `git rev-list --reverse HEAD -- <path>`, which applies history
simplification. When the same record content is introduced independently on two branches and merged,
one introducing commit is simplified away:

```
side=00d13a60  main=5171092d  rev-list history=[5171092d, merge]
seal names the side commit (a true ancestor of HEAD, publishing identical bytes)
  -> final-record-provenance-mismatch
```

It fails closed, and the topology is unusual, so this is a `COULD`. The straightforward
record-introduced-on-a-side-branch-then-merged case is handled correctly (§5.4 S5).

### N4 — `cohort-cost-total-not-bound` appears shadowed

The new `cross-provider-total-not-summed` branch precedes it and fires in every case I could
construct, including the one designed to reach the old branch (an unresolved comparison member whose
declared cost is therefore unchecked, with `MET-AI-060` reduced to the resolvable components). The
code is absent from the frozen 55, so the replay does not cover it either. Not a correctness defect;
a diagnostic that can no longer be emitted should be removed or made reachable.

### 4.4 New-symbol coverage

| New symbol | Probe that exercises it | Verdict |
|---|---|---|
| `parse_repository_json`, `_parse_json_float`, `_parse_json_int` | §5.2 A11–A24 (14 literals) | correct; no crash |
| `unrepresentable_number_paths` | A2, A3, A9 | correct; bools skipped; exact integers seen |
| `git_hash_object` | S8 formatting-only dirty tree | correct |
| `git_commit_is_reachable` | S4 dangling commit (rc=1 confirmed) | correct |
| `git_path_history` | S5 merge, S6 evil-twin merge, S7 rename | correct except N3 |
| `earliest_final_publication` | S2, S3, S5, S7, S10, S11 | correct except N3 |
| `artifact_subject_digest` | D1, D3, D5, D6, exemplar recomputation | **N1 and §3 R6** |
| `evidence_completeness_counts` | C2, C2′ | **§3 R1c-b** |
| `_reference_is_repository_verifiable` | C2′, E4, E5 | correct, but see §3 L |
| `lane_elapsed_seconds` | R1b, H3 | correct |
| `lane_measured_cost` | H1, H2, H7, H8 | correct |
| `MAX_NUMERIC_MAGNITUDE`, `PUBLISHED_IMPROVEMENT_FLOOR`, `LIFECYCLE_STAGE_EVENT_KINDS` | A1–A10, H5, H6, E4 | correct |
| removed `non_finite_paths`, `_reject_flow_constant` | replay 55/15, boundary literals | no regression |

---

## 5. Held-Out Verification

### 5.1 Independent replay

Driven by my own harness against the production validator, checking exact returned codes:

- baseline reference record: **zero findings** on the record route **and** the directory route;
- semantic mutations: **55/55** exact expected codes, no crash;
- structural controls: **15/15** exact expected codes, no crash;
- declared probes: **21/21** of the 23 that carry a mutation callable, exact codes;
- `P6` rebuilt from scratch in a temporary repository: `read_repository_artifact` refuses a final-
  component symlink to `/etc/passwd` with `ValueError`, and the record route returns
  `missing-repository-document`;
- `P9` rebuilt from scratch against a real `repo://` prefix over the policy document: honest control
  **clean**, forged digest → `evidence-prefix-digest-mismatch`, over-long `prefix_bytes` →
  `evidence-prefix-bytes-mismatch`.

Passing 129/129 and 55/15/21 verifies the implemented contract. It is not the invariant, which is why
everything in §3.2 and §4 comes from probes outside that corpus.

### 5.2 Numeric parsing and the `10**15` policy boundary

Through the real file boundary (`parse_repository_json`) and through the record route:

| Literal / value | Boundary result | Record-route result |
|---|---|---|
| `1000000000000000` (= `10**15`) | decoded exactly | accepted |
| `1000000000000001` | refused | `numeric-magnitude-out-of-range` + `schema-numeric-magnitude` |
| `-(10**15)-1` | — | both codes |
| `1e15`, `1E+15` | decoded `1000000000000000.0` | accepted |
| `1.0000000000000001e15` | refused | both codes |
| `-0.0` | decoded `-0.0`, `exact_decimal` → `-0.0` | accepted, no spurious negative-value finding |
| `0e0` | decoded `0.0` | accepted |
| `1e-320` (subnormal, non-zero) | decoded `1e-320` | accepted |
| `0.1` | decoded `0.1`, exact `0.1` | accepted |
| `1e-400`, `1e-1000000000` | refused: underflows to zero | — |
| `1e400`, `-1e400`, `1e309`, `1e1000000000` | refused: not finite | — |
| `10**400` as an integer literal | refused: exceeds the boundary | `numeric-magnitude-out-of-range` |
| `10**400` as an in-memory `uncertainty` | — | both codes, **no `OverflowError`** |

No input produced a crash on any path I tried, structural or semantic. The precision rule uses
`Decimal.adjusted()`, so the round-2 `float()` re-entry is gone. `1e-400` is refused rather than
silently becoming an exact zero, which is what makes a target-zero guardrail unable to hide a nonzero
loss at the file boundary. The boundary is a policy choice and the candidate says so; `10**15` is
below `2**53`, so every admissible magnitude is exactly representable.

### 5.3 Cohort derivation, against a fixture the implementer does not control

I built my own cohort: my own ten member lanes, my own arm values
(baseline `900/1200/1500/1800/2100 s`, `4/6/8/10/12 USD`; comparison `600/800/1000/1200/1400 s`,
`3/4.5/6/7.5/9 USD`), my own median and Tukey-hinge implementations, and the honest reductions I
computed myself (lead `1/3`, cost `1/4`). It never imports the candidate's test module.

| Case | Result |
|---|---|
| **honest held-out cohort** | **zero findings** — the positive case holds against an independent fixture |
| threshold exactly `0.2` with the honest reduction | zero findings |
| threshold `0.19999999999999998` | `cohort-improvement-floor-violated` + `schema-minimum` + `kgi-target-not-met` |
| identical arms, threshold `0.0` | `cohort-improvement-floor-violated`, `cohort-improvement-unmet`, `schema-minimum`, `kgi-target-not-met` |
| ×10 inflated baseline samples, all aggregates consistent | `cohort-lead-time-not-derived`, `cohort-cost-not-derived`, `kgi-target-not-met` |
| **duplicate** provider rows to double the baseline cost | `duplicate-measurement-series` |
| a baseline cost row's **unit** changed to `tokens` | `metric-unit-not-allowed` |
| two members' declared lead times **swapped** | `cohort-lead-time-not-derived` |
| one comparison member **omitted** from its arm | `cohort-arm-partition-mismatch` + five recomputation codes |
| a comparison member's cost row **re-attributed** to a baseline lane | `cohort-cost-not-derived`, `cross-provider-total-not-summed` |
| `MET-AI-060` **padded** above the summed comparison rows | `cross-provider-total-not-summed` |
| arm sample lists padded or short | `cohort-arm-shape-mismatch` (length equality is enforced) |

This is the strongest part of the candidate. Duplicates, omissions, permutation, unit substitution,
cross-lane substitution and author-declared mirrors are all refused, and an honest record still
passes. `MET-AI-060` is now bound to the summed component rows **and** to the arm total, so the
round-2 circularity is gone.

### 5.4 Sealing against real Git histories

Eleven probes, each in a fresh repository with real commits. The honest control is scored first;
`final-record-window-open` and digest noise from my deliberately minimal record are filtered out.

| Probe | Predecessor | Candidate |
|---|---|---|
| S1 honest two-step seal (commit the final record, then name that commit) | clean | **clean** |
| S2 edit the sealed record, commit, move `sealed_in_commit` to the new commit | **0** | `final-record-provenance-mismatch` |
| S3 edit unsealed content, commit, seal the newest commit | **0** | `final-record-provenance-mismatch` |
| S4 seal against a dangling `commit-tree` (ancestor-of-`HEAD` rc=1) | **0** | `final-record-provenance-unreachable` |
| S5 record introduced on a side branch, then merged into `main` | clean (vacuously) | **clean** — earliest reachable introduction found across the merge |
| S6 identical content on both branches, merged, seal names the side commit | 0 | `final-record-provenance-mismatch` (**N3**) |
| S7 sealed record relocated to a new path | `…-unresolved` | `…-unresolved` (**pre-existing**) |
| S8 formatting-only uncommitted edit | **0** | `final-record-uncommitted-edit` |
| S9 content-changing uncommitted edit | `…-mismatch` | `…-mismatch` |
| S10 shallow (`--depth 1`) clone of the sealing history | `…-unresolved` | `…-unresolved` (**pre-existing**) |
| S11 supersede by a new revision at a new path (the documented escape hatch) | clean | **clean** |

S7 and S10 reproduce identically on the predecessor, so **the candidate diff did not create them**;
they are pre-existing operational fragilities of binding a seal to a path plus a commit. They matter
here only because the candidate added the parenthetical *"(so the seal survives a clone or a `gc`)"*,
which is not true of a shallow clone — a common CI checkout.

One scope note the packet does not make: **no record in this repository is `final`**, so the entire
seal path has no production instance and is exercised only by tests and by probes like these.

### 5.5 Supersession, both directions

| Case | Result |
|---|---|
| honest reciprocal in-batch chain (`superseded` + `superseded_by` / `revision: 2` + `supersedes`) | **clean** |
| two independent active batches | **clean** — per-batch independence preserved |
| single-record batch, `active` or `final` | clean (exactly one head) |
| one-way cross-batch `superseded_by` on an active record | `supersession-cross-batch`, `superseded-active-record-status`, `supersession-no-head` |
| zero unsuperseded heads | `supersession-no-head` |
| non-monotone successor revision, from the `superseded_by` side | `supersession-not-monotone` |
| dangling `superseded_by` | `unresolved-superseded-by-record` (direction named) |

### 5.6 Diagnostics bound

| Input | Findings | `finding-limit-reached` |
|---|---:|---|
| 700 malformed events | 513 | yes |
| 2000 malformed events | 513 | yes |
| 600 malformed events + one semantic falsification | 3 | yes — semantic finding suppressed (**N2**) |

The combined bound holds. `schemas/README.md` says the cap is "at 512 findings"; the observed worst
case is 513 (512 plus the limit finding). The policy's own wording — "at most 512 findings … before
reporting `finding-limit-reached`" — is consistent; the schema README's is not (§11).

---

## 6. Prior-Finding Disposition

Standard as bound: `closed` only when schema, semantic branch, policy, exemplar and a non-vacuous
negative test jointly close the smallest falsification case.

### 6.1 The 17 rows round 2 left partially closed

| ID | What round 3 added | Smallest surviving falsification | Disposition |
|---|---|---|---|
| F1 | Boundary + magnitude scan; derived throughput, completeness, guardrail subject, cohort samples | `KGI-DEV-004` declarable (§3 R1c-a) | **partially closed** |
| F6 | `validation-instant-before-capture`; instant echoed | Bound is active-record-only; no echo in `--json` (§3 R5′) | **partially closed** |
| F7 | `Decimal.adjusted()`, boundary refusal of underflow/overflow, exact-integer scan | none found — no crash on any literal or in-memory value tried | **closed** |
| F9 | Reachability, earliest-publication comparison, working-tree blob | Seal correct; no `repo://` prefix exists in the record (disclosed); N3 topology | **partially closed (disclosed)** |
| F11 | Per-KGI vocabulary and recomputed targets, subject-bound guardrails | `KGI-DEV-004` achieved by declaration (§3 R1c-a) | **partially closed** |
| F12 | Per-member lead time and cost derived from lane instants and measured rows; `0.20` floor | none found — ten independent attacks all reject (§5.3) | **closed** |
| F13 | `MET-AI-060` bound to summed component rows *and* the arm total | none found — the circularity is gone | **closed** |
| F14 | `artifact_subject_digest` over the commit set | Commit-padding (D1) and commit-free padding (D5); over-strict N1 | **partially closed** |
| F-S2 | Per-batch freshness with the new lower bound | Backdated instant on a non-active record (§3 R5′) | **partially closed** |
| F-S3 | Earliest-publication seal; both-direction supersession; exactly one head | Seal and supersession closed; N3 remains | **closed for supersession**, **partially closed for seals** |
| F-S4 | Unchanged reference verification | Only `repo://` prefixes are verifiable and the record has none (disclosed) | **partially closed (disclosed)** |
| F-S7 | Unchanged | `runtime_version` still optional (waived, disclosed) | **partially closed (waived)** |
| F-S9 | `MET-AI-075` / `MET-FLOW-037` bound to a metric-naming audit event | Both declarable with one string each (§3 R1c-a) | **partially closed** |
| F-S13 | `zero-target-audit-event-subject-mismatch` | The subject name is author-supplied (§3 R1c-a) | **partially closed** |
| F-S14 | Boundary + scan replace the crash | none found | **closed** |
| F-C2 | Storage lane marked unverifiable | Still no artifact, one operator-only prefix, no trusted-signal event, "six or more" outcome | **partially closed (disclosed)** |
| F-C7 | One limitation added | Honesty is still prose-pattern based | **partially closed (inherent)** |

**Totals for these 17: 4 newly closed** (F7, F12, F13, F-S14), **13 still partially closed**, of which
4 are disclosed/waived/inherent (F9, F-S4, F-S7, F-C2, F-C7 — five rows, four distinct causes) and
**6 carry an executable falsification proved above** (F1, F6, F11, F14, F-S2, F-S9/F-S13 as one
mechanism), plus F-S3's seal half.

### 6.2 Regression check on the 18 previously closed rows

The 15 closed at round 1 (F2, F3, F5, F8, F10, F-S1, F-S5, F-S6, F-S8, F-S11, F-C1, F-C3, F-C4, F-C5,
F-C6) and the three closed at round 2 (F4 nanosecond ordering, F-S10 evidence-derived mutation state,
F-S12 candidate lineage as a graph) were re-exercised through my replay of all 55 semantic mutations,
all 15 structural controls and all 21 executable probes, plus the exemplar audit.

**No regression on any of the 18.** F4 is confirmed by `P1` (`event-outside-window` at
`captured_at + 1 ns`), F-S10 by `P12` (`lane-mutation-none-unproven`), F-S12 by `P3`
(`candidate-lineage-cycle`). One qualification carried forward and now sharpened: F3's neighbour
obligation (canonical stage reconciliation) is enforced in the `status` direction but the closing
stages themselves remain synthetic-satisfiable (§3 L).

### 6.3 R1–R7 owner-level disposition

| Owner | Status on this candidate |
|---|---|
| Semantic validator (R2) | **closed** — documented boundary, three-layer enforcement, exact `Decimal` comparison, no crash on any input tried |
| KGI / cohort (R1) | **closed for the cohort (R1a, R1b)**; **partially closed for R1c** — `KGI-DEV-004` declarable, completeness reachable by omission, counted integrations unverified |
| Evidence / references (R3) | **closed for references**; **closed for seals** except the N3 merge topology and the pre-existing path/shallow-clone fragility |
| Record lifecycle (R3, R5) | **partially closed** — seal and supersession closed; the instant lower bound is active-record-only |
| Lineage / lifecycle (R4) | **closed for lineage and for supersession**; **partially closed for completion** (§3 L) |
| Anti-gaming (R6) | **partially closed** — mutation state, denominator and cohort closed; artifact identity both too loose (D1, D5) and too strict (N1) |
| Schema / tests (R7) | **closed** for the bound and the new negative tests; N2 is the cost, N4 a shadowed code |
| Docs / policy | **partially closed** — three of the round-2 eight fully implemented, three narrowed correctly, two replaced by new overstatements (§11) |

---

## 7. Positive Cases Verified Without Sharing Production Helpers

| Case | Construction | Result |
|---|---|---|
| Committed exemplar | as committed | zero findings, record route and directory route |
| Valid two-batch directory | second batch built here, distinct `batch_id`, both `active` | zero findings; batches independent |
| Valid reciprocal supersession chain | predecessor `superseded` + `superseded_by`; successor `revision: 2` + `supersedes` | zero findings |
| Sealed final record on real Git provenance | temp repo, real `git init`/`add`/`commit`, honest two-step seal | zero findings; every tampering variant rejects with its own code |
| Sealed record introduced on a side branch and merged | real merge commit | zero findings |
| Supersession by a new revision at a new path | the documented escape hatch | zero findings |
| **One complete cohort, my own fixture and arithmetic** | ten member lanes I wrote, my median/Tukey hinge, my reductions | **zero findings**; declared `1/3` lead and `1/4` cost reductions match my recomputation |
| Cohort threshold exactly at the published floor | `required_improvement_ratio = 0.2` | zero findings |
| Honest `repo://` evidence prefix | real bytes, my own SHA-256 over the first 64 bytes | zero findings |
| All five exemplar artifact subjects | my own canonical-JSON SHA-256 over the commit sets | **all five reproduced exactly** |

The contract admits honest positive records. Nothing in §3 or §4 is a false-rejection of an honest
record except N1.

---

## 8. Oracle Admissibility (audit obligation I4)

Applying the audit's eight admissibility conditions to the oracles the candidate offers as closure
evidence:

| Condition | Verdict on this candidate |
|---|---|
| A frozen counterexample corpus is not the invariant | **Respected in form, still the main risk.** The 55/15 tables are unchanged and six new probes were added, but §3.2 and §4 are all outside the 23-probe corpus, and five of the six new probes are the round-2 report's own sentences turned into cases. The corpus grew along the axis the last review named. |
| The asserted axis must observably move | Met. Every new negative test asserts a code that its own mutation causes; I re-derived the predecessor behaviour for all 13 and the axis moves in every one. |
| Deleting the production line must change a result | Met where I checked: stripping `measurement:MET-FLOW-026` from the committed record's recovery event turns it from clean to `zero-target-audit-event-subject-mismatch`. |
| A test name/docstring claim must be entailed by its assertions | **Two violations.** `check_final_provenance`'s docstring claims the blob "must equal both the declared blob id and the working-tree blob id" — the working-tree comparison is against `HEAD`, not against the sealing commit. `lifecycle-stage-evidence-unverifiable`'s message claims a policy citation does not qualify; it does. |
| Reversal must be behavioral, not symbol-absence | Met. |
| The assertion must fire at or before the boundary it names | Met. |
| Multi-fault fixtures where the property is about ordering | Met for seals (real multi-commit histories) and supersession (real multi-record batches). |
| A zero-selected or foreign-artifact gate is never evidence | **One weakness.** The cohort acceptance test runs with `root=None`, so every repository-verification branch is inert for the candidate's central positive case. My held-out cohort inherits that limitation; I compensated by verifying repository resolution separately on the committed record. |
| — added here: a test must not share the helper it validates | **One violation.** `_pad_artifact_subject` calls the production `artifact_subject_digest` to build the subject it then asserts on. The assertion (`duplicate-artifact-subject`) still has content, but the fixture cannot detect a wrong digest rule — which is exactly the rule D1/D5/N1 show is wrong. |

---

## 9. Exemplar Honesty Audit

The committed CFR-168 record remains the most candid part of this lineage, and round 3 changed it in
exactly two substantive ways.

- **Five artifact `subject_sha256` values recomputed.** All five reproduce exactly under my own
  canonical-JSON SHA-256 over the sorted commit sets. No artifact was added, removed or relabelled.
- **`measurement:MET-FLOW-026` added to the `disk-git-recovery-20260730-083120` recovery event**, so
  the `estimated 0` for lost source work is bound to an audit that names that guardrail. I checked
  the substance: that event genuinely is the recovery audit whose outcome states no product source
  loss was observed. The claim is materially true, the row stays `estimated` with
  `precision: operator_observed`, and stripping the reference reproduces
  `zero-target-audit-event-subject-mismatch`, so the binding is non-vacuous.
- **One limitation added**, disclosing that the other eleven guardrails remain absent because no
  audit named them. 15 limitations total, none vacuous.

Everything else is unchanged and still honest: two `measured` rows, all seven evidence prefixes
`operator_only_unverifiable`, `MET-AI-060` `unavailable`, no cohort, all five KGIs `in_progress` or
`at_risk`, no terminal claim, the storage lane still explicitly unverifiable.

**The honesty problem is structural, not local.** The exemplar's own new binding — one `measurement:`
string appended to an existing event — is byte-for-byte the mechanism §3 R1c-a uses to fabricate
`KGI-DEV-004: achieved`. The contract cannot distinguish the honest use from the dishonest one, so
this record's candour is still the author's choice rather than the contract's guarantee.

---

## 10. Decision Capability Audit

| Question | Assessment |
|---|---|
| Is the contract mechanically valid? | **Yes for shape, arithmetic and provenance; no for one terminal decision.** `KGI-DEV-004` is still fabricable with zero findings. |
| Is the gate itself safe to run? | **Yes.** No input I tried raises; the round-2 `OverflowError` is closed and the diagnostics bound holds at 513 for 2000 malformed events. |
| Is `KGI-DEV-005` now trustworthy? | **Materially yes.** Ten independent attacks on the cohort all reject, an independent honest cohort passes, and both reductions plus the cross-provider total derive from typed lane facts. This is the round-2 blocker genuinely closed. |
| Is the committed record factually honest? | **Yes**, and slightly more so than at round 2. |
| Which KPIs are decision-capable now? | Finite interval diagnostics, checkpoint coverage, throughput as an event count, cohort lead time and cost where member lanes are closed and priced, evidence completeness read as "verifiable share of my own citations". |
| Which are not? | Every zero-target guardrail (author-nameable audit), "required evidence completeness" read as completeness, trusted-integration *trust*, all provider-reported costs (no `repo://` prefix exists), `runtime_version`-dependent rows. |
| Can policy authorize a process change from this contract? | **Not yet.** The policy still treats a passing semantic check as the decision boundary while `KGI-DEV-004` remains declarable and two published sentences describe refusals the code does not make. |

---

## 11. Published Claims Versus Enforcement

Round 2 listed eight over-strong claims. Three are now fully implemented, three are correctly
narrowed, and two were replaced by new overstatements. Five rows remain.

| # | Claim | Where | Reality |
|---|---|---|---|
| 1 | "An unrelated in-window gate failure observes nothing about escaped defects, quota stoppages, or silent downgrades, so citing one is `zero-target-audit-event-subject-mismatch`." | `development-flow-kgi-kpi.md` | Citing one **and naming the metric on it** is clean; `KGI-DEV-004` reaches `achieved` (§3 R1c-a) |
| 2 | "The values that define a KGI are recomputed, never declared" / `KGI-DEV-001`–`002` target "complete required evidence" | `flow-metrics/README.md`; policy KGI table | `MET-FLOW-027 == 1` is reachable by deleting citations (§3 R1c-b) |
| 3 | "neither relabelling under an unused kind nor padding the list with an already-declared lane, event, or prefix reference can split one deliverable into several artifacts" | `development-flow-kgi-kpi.md`, *Unique Artifacts*; `flow-metrics/README.md` rule 11 | Padding with an already-declared **commit** splits a patch artifact (D1); padding any commit-free kind with an `event:` reference splits it (D5) |
| 4 | closing stages need "repository-verifiable evidence rather than a policy citation (`lifecycle-stage-evidence-unverifiable`)" | policy *Event obligations* table; the code's own message | A `doc:` policy citation **is** repository-verifiable and satisfies the check (§3 L) |
| 5 | "an instant earlier than the record's own `captured_at` … is refused"; "the checker **echoes the instant it used** on every run" | policy *Freshness*; `flow-metrics/README.md`; `project-management.md` | The refusal exists only for `active` records; `--json` emits no echo (§3 R5′) |
| — | "so the seal survives a clone or a `gc`" | policy *Record Provenance* | True of `gc` and of a full clone; a `--depth 1` clone makes an honest seal `final-record-provenance-unresolved` (pre-existing behaviour, new claim) |
| — | "caps its combined structural and semantic diagnostics at 512 findings" | `schemas/README.md` | Worst case observed is 513 (512 + `finding-limit-reached`). The policy's own wording is consistent; this one is not |
| — | docstring: the blob "must equal both the declared blob id and the working-tree blob id" | `check_final_provenance` | The working-tree blob is compared against `HEAD`'s, not against the sealing commit's |

Correctly implemented or narrowed by this candidate, verified here: the exactness statement (now
scoped to the documented `10**15` boundary and true), "no input raises" (no crash found), the seal
wording (earliest reachable final publication — verified), the exactly-one-head wording (verified),
the `0.20` floor (verified at the schema and semantic layers), the cross-provider total wording
(verified), the both-directions supersession wording (verified), the MET-FLOW-027 and MET-FLOW-028
metric-table rows (now describe what the code actually computes), the omitted-instant disclosure, and
the 512-finding bound in the policy (true of both layers).

`project-management.md` and the Phase-2 scorecard row remain accurate and appropriately hedged.

---

## 12. Findings Classified, With Diff Attribution

Every finding, its class, and whether **this candidate diff created it**.

| ID | Severity | Finding | Class | Created by this diff? |
|---|---|---|---|---|
| R1c-a | **MUST** | `KGI-DEV-004` reaches `achieved` with zero findings via one author-written `measurement:` string on an unrelated event | `BOUND_FINDING_NOT_CLOSED` | **No** — inherited from round 2 R1c and narrowed. The false published sentence about it **is** new |
| R1c-b | SHOULD | `MET-FLOW-027 == 1.0` by deleting citations; the completeness denominator is author-chosen | `BOUND_FINDING_NOT_CLOSED` + `PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED` | **Partly** — the derivation and its omission incentive are new; the declarability is inherited |
| R1c-c | COULD | Counted `integration` events need no gate, commit or trusted-signal binding | `BOUND_FINDING_NOT_CLOSED` (narrowed) | Partly — the count is new and is a real tightening |
| R6-D1 | SHOULD | Padding a patch artifact with a second already-declared **commit** splits the deliverable | `BOUND_FINDING_NOT_CLOSED` | **No** — inherited R6; the new rule closed only the non-commit shape |
| R6-D5 | SHOULD | Padding any commit-free artifact kind with an `event:` reference splits it | `BOUND_FINDING_NOT_CLOSED` | **No** — inherited R6, preserved by the documented fallback |
| L | SHOULD | Five synthetic policy-doc events discharge the whole closing half on a `final` record | `BOUND_FINDING_NOT_CLOSED` | **No** — inherited round-2 `SHOULD`; the contradicting claim **is** new |
| R5′ | SHOULD | The instant lower bound exists for `active` records only; `--json` omits the echo | `INITIAL_REVIEW_SCOPE_MISS` | **Yes** — the check and the claim are both new |
| N1 | SHOULD | Two honest deliverables in one commit are rejected as `duplicate-artifact-subject` | `REPAIR_INTRODUCED_REGRESSION` | **Yes** |
| N2 | SHOULD | A >512-finding structural flood suppresses the semantic layer entirely | `REPAIR_INTRODUCED_REGRESSION` | **Yes** |
| N3 | COULD | Honest seal fails when identical content is introduced on two branches and merged | `REPAIR_INTRODUCED_REGRESSION` | **Yes** |
| N4 | COULD | `cohort-cost-total-not-bound` appears shadowed by `cross-provider-total-not-summed` | `REPAIR_INTRODUCED_REGRESSION` (dead diagnostic) | **Yes** |
| C1 | COULD | Docstring claims a working-tree/sealing-commit blob comparison that is not made | `VACUOUS_OR_LATE_TEST_ORACLE` (claim/oracle) | **Yes** |
| C2 | COULD | `schemas/README.md` "at 512 findings" vs an observed 513 | doc precision | **Yes** |
| C3 | COULD | "so the seal survives a clone or a `gc`" is false for a shallow clone | doc precision | **Yes** (claim); the behaviour is pre-existing |
| C4 | COULD | The round-3 packet reports `check_project_docs.py` "565 Markdown files"; the candidate reports **566** | `EVIDENCE_DOCUMENTATION_ONLY` | **Yes** |
| C5 | COULD | `_pad_artifact_subject` builds its fixture with the production `artifact_subject_digest` it is meant to validate | `VACUOUS_OR_LATE_TEST_ORACLE` | **Yes** |
| C6 | COULD | The cohort acceptance test runs with `root=None`, so repository verification is inert for the central positive case | `VACUOUS_OR_LATE_TEST_ORACLE` | No — inherited fixture convention |
| P1 | — | Relocating a sealed record, or validating in a shallow clone, makes an honest seal unresolvable | pre-existing | **No** — reproduces identically on `0944bf3` |

Interaction, not a single cause, dominates again: 7 of the 18 rows were created by this diff, 8 are
bound findings the diff narrowed without closing, and 1 is pre-existing — which matches the audit's
own cross-lineage result and is why §4 exists as a separate section from §6.

---

## 13. Adjudication Of The Packet's Eight Declared Limitations

Adjudicated under the zero-residual objective, without expanding scope.

| # | Packet limitation | Adjudication |
|---|---|---|
| 1 | No evidence prefix in the committed record is repository-verifiable; all seven `second-agent://` locators are unreachable | **Accepted as disclosed.** I verified the mechanism works on a real `repo://` prefix I built (honest clean, forged digest and over-long byte count both rejected), so the capability is real and the gap is the record's, not the contract's. It stays a residual because `MET-FLOW-027`'s numerator and every `measured` cost row depend on it |
| 2 | `runtime_version` was never captured, so `MET-AI-075` and `MET-FLOW-037` stay `absent` | **Accepted as waived — but now consequential.** These are exactly the two metrics §3 R1c-a drives to a fabricated `measured 0`. Making `runtime_version` required for a `measured` `MET-AI-075` would close half of R1c-a |
| 3 | The freshness control only runs when an instant is injected; the echo makes that visible | **Accepted, incomplete.** The echo is real and correct in the human output, absent in `--json`, and the lower bound it advertises does not apply to non-active records (§3 R5′) |
| 4 | Limitation honesty remains prose-pattern based | **Accepted as inherent.** Unchanged and correctly stated |
| 5 | The storage review lane (F-C2) is unverifiable and counts against coverage | **Accepted as disclosed.** Verified unchanged |
| 6 | The `10**15` magnitude boundary is a policy choice | **Accepted.** Documented, exactly representable, enforced three times, and I could not find a legitimate quantity in this domain near it. This is the right way to close R2 |
| 7 | Cohort lead time derives from the lane interval, not packet-ready to integration-ready | **Accepted, and it is the honest reading.** `MET-FLOW-001`'s published definition is trusted-integration lead time, so the derived quantity is narrower than the metric name. The policy's cohort table now says which interval is used, which is the minimum acceptable disclosure. A stricter derivation needs `packet_ready` events the records do not carry |
| 8 | `P6` and `P9` carry no mutation callable because they need a temporary repository | **Accepted.** I rebuilt both from scratch and both hold, including an honest control for `P9`. Two of 23 probes remaining non-executable in the table is a documentation gap, not a coverage gap |

None of the eight is a reason to block. Items 2 and 3 are load-bearing for §3 and are recorded there.

---

## 14. Required Next Action And Residual Owners

Do not integrate. A successor must close every row below and add a production-route negative test for
each.

| Owner | Required closure |
|---|---|
| KGI/guardrail owner | A zero-target guardrail's audit event must be an audit **of that guardrail** by something other than an author-chosen name — e.g. require the event's `kind`/`stage` to be an audit of that metric's own subject, require the row's `precision` to be at least `instrument_sampled`, or require a distinct evidence artifact per guardrail. `measurement:` back-references must not by themselves discharge a claim. Also make `runtime_version` required before `MET-AI-075` may be `measured`. |
| KGI/completeness owner | Either enumerate the *required* evidence set so `MET-FLOW-027`'s denominator is not the author's citation list, or rename the metric and correct the `KGI-DEV-001`/`KGI-DEV-002` target statement so "complete required evidence" no longer means "every citation I chose to make is verifiable". |
| Anti-gaming owner | Derive artifact subject identity from the deliverable in a way that is neither splittable nor over-strict: include the commit's touched-path set so two deliverables in one commit are distinguishable (N1) while padding with another commit or with any citation cannot mint a new subject (D1, D5). Give commit-free kinds a subject that is not their citation list. |
| Lifecycle owner | Bind each closing-stage event to evidence *of that stage* — not to any repository-verifiable reference, and specifically not to the policy document — or correct the published claim and the finding message. |
| Freshness owner | Apply the `captured_at` lower bound to every record status, and echo the instant in `--json` as well. |
| Schema/test owner | Reserve part of the 512-finding budget for the semantic layer so a structural flood cannot suppress it (N2). Remove or make reachable `cohort-cost-total-not-bound` (N4). Rebuild `_pad_artifact_subject` without the production digest helper, and give the cohort acceptance case a real repository root. Add negative tests for: commit-padded and commit-free-padded artifact subjects, two deliverables in one commit, a backdated instant on a `final` record, a synthetic policy-doc lifecycle closure on a `final` record, and `KGI-DEV-004` declared from a foreign audit event. |
| Record-lifecycle owner | Decide the merge/rename/shallow-clone policy for seals: follow renames, tolerate a simplified-away identical introduction (N3), and either handle a shallow clone or state that a full clone is required. Correct the `check_final_provenance` docstring. |
| Docs/policy owner | Implement or narrow each of the five remaining §11 rows plus the three precision rows. Correct the packet's "565 Markdown files" to 566. |

**Required focused acceptance evidence for the next candidate:** everything the bound reports already
require, plus (15) no terminal KGI is reachable without a fact the author cannot mint by naming it;
(16) an artifact subject that resists both padding and commit-level aggregation, with an honest
two-deliverables-one-commit positive case; (17) a semantic finding still reported in the presence of a
structural flood; (18) the instant lower bound on every status and in every output mode; (19) a
lifecycle closure that a policy citation cannot satisfy.

---

## 15. Handoff

- **Decision:** `REQUEST_CHANGES` on exact candidate `753c8db103579d021795d3fe6a97c5d45d4a111a`
  (tree `213e16a033aeb2e8eb3599be4ffe725b1d7ad344`).
- **Files changed by this reviewer:** only
  `docs/development/ai/swarm-results/development-flow-kgi-kpi-r3-rereview-claude-20260731/review.md`.
- **Validation on this candidate:** 129/129 repository tests; 55/55 semantic codes; 15/15 structural
  codes; 21/21 executable probes plus `P6`/`P9` rebuilt from scratch; all seven
  `check_ai_development` components `PASS`; contracts, project docs and `git diff --check` clean; the
  frozen 55/15 tables byte-identical to the predecessor; eleven seal probes, seven supersession
  cases, twelve cohort cases and twenty-four numeric literals executed here.
- **What genuinely closed:** all 13 round-2 executable residuals, each reproduced first on the
  predecessor tree itself — the `OverflowError`, the underflow-to-zero, the self-declared cohort
  threshold, the free-floating arm samples and costs, the circular cross-provider total, the
  edit-then-reseal, the dangling seal, the one-way and cross-batch `superseded_by`, the zero-head
  batch, the backdated instant on an active record, the lane-padded artifact subject, the unbounded
  structural diagnostics, and lifecycle completion on an active record. R2 and the cohort half of R1
  are closed outright, and the cohort survived ten attacks it was not built for.
- **What blocks:** §3 R1c as `MUST` — `KGI-DEV-004` still reaches `achieved` with zero findings from
  one unrelated event and two author-written strings, and the policy publishes a sentence my probe
  refutes. `SHOULD`: R1c-b completeness by omission, R6 (D1, D5), lifecycle closure by policy
  citation, the active-only instant bound, and the two candidate-introduced regressions N1 and N2.
  Five published claims are stronger than the code.
- **Risks of accepting anyway:** the contract would publish that a passing semantic check makes a
  record decision-grade while still admitting a fabricated `KGI-DEV-004`, an artifact count inflated
  by one padding commit, a lifecycle closure evidenced only by the policy document that describes it,
  and a record whose semantic defects are hidden behind structural noise — and it would reject an
  honest record that ships two deliverables in one commit.
- **Next action:** return the ticket to the residual owners in §14. Do not integrate, Close, Install,
  Dogfood, or Retrospect this candidate.
