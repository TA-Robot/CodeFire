# Development-Flow KGI/KPI Round-2 Independent Rereview (Claude Fallback)

Review date: 2026-07-31

Reviewer: Claude Opus 5 (`claude-opus-5`), independent fallback reviewer

Exact candidate: `0944bf3deba9e753132b3f71716b5c009a8fc30f`

Candidate parent (rejected predecessor): `0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1`

Candidate tree: `3c938f2d3e3e6b996346f63c22dc4de256c67f21`

Implementation diff: `0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1..0944bf3deba9e753132b3f71716b5c009a8fc30f`

**Decision: `REQUEST_CHANGES`.**

The candidate is a large and mostly real repair. Independently replayed, all 55 frozen semantic
mutations and all 15 structural controls still return their exact expected code, all 17 declared
production probes reject with their expected stable code, the eight manager counterexamples from the
bound rereview are genuinely closed, the three positive acceptance cases (a two-batch directory, a
sealed final record bound to real Git provenance, one complete cohort) validate clean against
arithmetic I recomputed myself, and the committed CFR-168 record became materially more candid
(fourteen rows demoted from `measured` to `estimated`, two `measured` rows left, eleven guardrails
`absent`, cost `unavailable`, no cohort, no terminal KGI).

That is not the bound standard. The ticket's rule is that any partially-closed row independently
requires `REQUEST_CHANGES`, and the central R1 blocker — *terminal KGI and cohort decisions remain
self-authorizable* — is narrowed rather than closed. Through the production route, on this exact
candidate:

- `KGI-DEV-005` reaches `achieved` with **zero improvement** between identical arms, because the
  20-percent threshold the policy publishes is read from the record's own
  `required_improvement_ratio` (§3 R1a);
- the cohort's per-member lead times and costs are free-floating declarations — the candidate's own
  passing cohort fixture declares 1000–2000 s lead times for member lanes that each ran 120 s — so
  an arbitrary reduction ratio is recomputable-and-wrong (§3 R1b);
- `KGI-DEV-001`, `KGI-DEV-002` and `KGI-DEV-004` reach `achieved` with zero findings by declaring
  three trusted integrations, 7/7 evidence completeness and five measured zeroes, citing the policy
  document plus one unrelated in-window gate-*failure* event (§3 R1c);
- a finite, schema-valid `uncertainty` of `10**400` makes the production checker raise
  `OverflowError` instead of returning a finding (§3 R2);
- a sealed `final` record can be edited and resealed by a later commit, or sealed against a
  **dangling, unreachable** commit object, with zero findings (§3 R3);
- `superseded_by` is validated one-way: it may point across batches, need not be reciprocated, and a
  batch with **zero** unsuperseded heads passes, though the policy and README publish "exactly one
  head" and same-batch supersession (§3 R4);
- one extra harmless evidence reference defeats the new artifact subject digest, so one deliverable
  still becomes two artifacts (§3 R6);
- the injected validation instant is unbounded below: `--validation-instant 1970-01-01T00:00:00Z`
  passes silently and the gate output records nothing about which instant was used (§3 R5).

Six published claims are also stronger than the code (§11), including "every malformed value
produces a stable finding; nothing raises" and "a single record emits at most 512 findings".

This decision binds only exact candidate `0944bf3deba9e753132b3f71716b5c009a8fc30f`. It does not
authorize integration, CFR-168 Close, Install, Dogfood, or Retrospect.

The failed Fugu round-2 attempt is reported separately in §13 and had no influence on any product
finding above.

---

## 1. Exact Subject And Review Bindings

### 1.1 Bound authorities

| Authority | Exact identity | Verification |
|---|---|---|
| Exact candidate | `0944bf3deba9e753132b3f71716b5c009a8fc30f` | `git rev-parse HEAD`; subject `fix(ai): make the KGI/KPI contract exact, derived, and fail-closed`, committed `2026-07-31 08:06:13 +0000` |
| Rejected predecessor | `0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1` | `git rev-parse 0944bf3^` |
| Bound prior report | `d04ec99:docs/development/ai/swarm-results/development-flow-kgi-kpi-zero-residual-rereview-fugu-20260730/review.md` | SHA-256 `268ec0b768a334ad11dbdf57c81245d7b315d7699e34c511e1f782253c178b30`, recomputed here; matches the digest the R2 result packet declares |
| Prior decision | `REQUEST_CHANGES`, R1–R7, 15 closed / 20 partially closed / 0 open | §6 and §11 of that report |
| Original independent review | `b307b47f51f73b054b66444493c4590379518c4f` | read for F1–F14 / F-S1–F-S14 / F-C1–F-C7 identity |
| Round-2 result packet | `docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r2-20260731.md` | SHA-256 `9b224d37a8a2569743a5b655fcdadffb0f9b14476cac2c9fd25b6448a7afbe75` |

The prior report and the result packet were read, never edited.

### 1.2 Candidate changed files and reviewed blobs

| Added / removed | Blob at candidate | Path |
|---:|---|---|
| 1662 / 559 | `2cbb66380326` | `docs/development/tools/ai_contracts.py` |
| 982 / 9 | `09eb16768e25` | `docs/development/tools/test_ai_tools.py` |
| 154 / 17 | `44d34d297efa` | `docs/development/ai/development-flow-kgi-kpi.md` |
| 117 / 0 | `377d34413c38` | `docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r2-20260731.md` |
| 88 / 39 | `d6ba8b782ba5` | `docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json` |
| 79 / 4 | `eac312cf6fb6` | `docs/development/ai/schemas/development-flow-record.v1.schema.json` |
| 63 / 6 | `f6b7d224f9c9` | `docs/development/tools/check_ai_contracts.py` |
| 36 / 6 | `f2c57c1126e2` | `docs/development/ai/flow-metrics/README.md` |
| 14 / 4 | `8946fd4210e0` | `docs/development/project-management.md` |
| 5 / 4 | `e7760b086ff6` | `docs/development/exploration/phase-2/experiment-catalog-and-kpi-scorecard.md` |
| 2 / 0 | `35ba968050f0` | `docs/development/ai/schemas/README.md` |
| 1 / 0 | `de90936496f8` | `docs/development/history.md` |

No product source, issue state, devcontainer file, agent wrapper, or Cargo/Rust artifact is touched
by the candidate, and none was touched by this review.

### 1.3 Constraints honored

No implementation edit, no Cargo or Rust command, no install, dogfood or demo, no Codex, no
subordinate agent, no network command. Every probe is a small deterministic Python or Git command in
`/tmp`. The only repository file this review writes is its own `review.md`.

---

## 2. Method And Commands Actually Run

### 2.1 Method

1. Read the R2 result packet, then the full candidate diff surface: the semantic validator, the
   production checker route, the schema, the policy, the flow-metrics README, `project-management.md`,
   the Phase-2 scorecard row, and the exemplar record.
2. Re-ran every repository gate.
3. Independently replayed the frozen 55/15 matrix and the 17 declared production probes with my own
   driver against the production validator, checking exact returned codes rather than mere rejection.
   `P6` (symlink escape) and `P9` (forged prefix digest), which carry no mutation callable in the
   candidate's table, were rebuilt here from scratch.
4. Adjudicated each of the seven hypotheses handed to me, each through the real file boundary or the
   production entry point rather than through a helper in isolation.
5. Re-disposed the 20 partially-closed rows and regression-checked the 15 closed rows.
6. Verified the three positive acceptance cases with statistics and digests recomputed locally
   (`Fraction` medians and Tukey hinges, my own canonical-JSON SHA-256), never through
   `median_of`, `dispersion_of`, or `sha256_json`.

### 2.2 Repository gates

| Command | Result |
|---|---|
| `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/test_ai_tools.py` | `Ran 108 tests`, `OK` (3.663 s) |
| `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_contracts.py` | `AI contract check passed.` |
| `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_project_docs.py` | passed; 565 Markdown files inventoried |
| `PYTHONDONTWRITEBYTECODE=1 python3 docs/development/tools/check_ai_development.py --timeout 180` | `PASS` for all seven components (`acceleration_docs`, `contracts`, `project_docs`, `source_docs`, `runtime`, `tool_tests`, `validation_capabilities`) |
| `git diff --check` | clean |
| `check_ai_contracts.py --validation-instant 2026-08-05T00:00:00Z` | 1 × `active-record-review-overdue`, 3 × `in-progress-measurement-overdue` |
| `check_ai_contracts.py --validation-instant 1970-01-01T00:00:00Z` | `AI contract check passed.` (see §3 R5) |
| `CODEFIRE_FLOW_VALIDATION_INSTANT=2026-07-30T09:51:56Z check_ai_contracts.py` | `AI contract check passed.` |

### 2.3 Probe artifacts

The constraint permits writing only this report, so the probe drivers live outside the repository.
891 lines total, all deterministic and re-runnable against this exact candidate:

| SHA-256 | Driver | Covers |
|---|---|---|
| `f0038829961c84884bd118632f1f40ec170a76cb1599e1e1ffe83a843ebf19a2` | `/tmp/kgi-r2-probe/replay.py` | baseline, 55 semantic, 15 structural, 17 production probes incl. rebuilt P6/P9 |
| `8161545b86c1b9f45e259616e544dbcbc5beccf34f8b719d07f251a2c060c929` | `/tmp/kgi-r2-probe/hypo_numeric.py` | raw-JSON numerics through the real file boundary |
| `71221bdc51e6eb20b4279a9dcaebe283ec272d2e572f185b8b340f8a82b2a733` | `/tmp/kgi-r2-probe/hypo_crash_route.py` | the same crash through `check_development_flow_records` |
| `fd8c2f8906a7ffcee5d105cfc15d678902e983ae400781afc29649c230ca70ba` | `/tmp/kgi-r2-probe/hypo_dir_art.py` | injected instant, supersession graph, artifact subject, diagnostics bound |
| `91f58981ccc5f31da1bc4dbf65ab8e833ae9ba3fea273661ab24c1fe803ed7cb` | `/tmp/kgi-r2-probe/hypo_cohort.py` | cohort threshold and arm derivation, independent statistics |
| `6ee0ec61e078707c9683da57c5a04815d43c03c8de78faf6f59f128281be1230` | `/tmp/kgi-r2-probe/hypo_seal.py` | real-Git-history seal, reseal, dangling-commit seal |
| `98cf89785470d007ca743f8508a2f3285b7d3b18a8017de8170d3f6a1d041773` | `/tmp/kgi-r2-probe/hypo_kgi.py` | non-cohort terminal KGI by declaration |
| `0c54977597091ff8a3a5cec0823b93d6c9d2270c3abf34de37dd0b85d27c1b0e` | `/tmp/kgi-r2-probe/hypo_misc.py` | array bounds, lifecycle completion by declaration |

---

## 3. Severity-Ordered Blocking Findings

Each finding is stated as the smallest falsification, run on the production route, and mapped back to
the R1 residual owner it belongs to.

### MUST-BLOCKER R1 — Terminal KGI and cohort decisions are still self-authorizable

The per-KGI vocabulary and target registry is a genuine improvement: `kgi-metric-not-allowed`,
`kgi-required-metric-missing`, `kgi-achieved-without-measurement`, `kgi-achieved-with-open-metric`,
`kgi-target-not-met` and `kgi-failure-without-violation` all fire (`ai_contracts.py:3526`). What the
targets are evaluated *against* is still author-controlled.

**R1a — `KGI-DEV-005` accepts a self-declared improvement threshold below the published 20 percent.**
The policy fixes the target twice: "reduces median trusted-integration lead time and total cost by at
least 20%" (`development-flow-kgi-kpi.md:52`) and "a `required_improvement_ratio` (0.20 for the
published target)" (`:439`). The checker instead reads the threshold out of the record
(`ai_contracts.py:3841`) and only requires `expected >= required` (`:3868`); the schema allows any
value in `[0, 1]` (`development-flow-record.v1.schema.json:172`). Starting from the candidate's own
valid cohort fixture:

| Case | `required_improvement_ratio` | Recomputed reduction | Findings | `KGI-DEV-005` |
|---|---:|---|---|---|
| honest fixture | 0.2 | lead 0.2667, cost 0.4 | none | `achieved` |
| weakened threshold | 0.005 | lead 0.0100, cost 0.0100 | **none** | `achieved` |
| identical arms | 0.0 | lead 0.0000, cost 0.0000 | **none** | `achieved` |

A cohort whose two arms are byte-identical therefore publishes `KGI-DEV-005: achieved` and passes.
This is the direct false-achievement path F11/F12/F13 was opened for.

**R1b — cohort arm samples and costs are not derived from the resolved member lanes or the record's
own cost rows.** Membership resolution is real (`cohort-member-unresolved`,
`cohort-member-ambiguous`, `cohort-member-class-mismatch` all fire), but `_cohort_arm`
(`ai_contracts.py:3733`) takes `lead_time_seconds` and `costs_usd` as free lists. The candidate's own
accepted fixture demonstrates the gap:

- each member lane runs `08:mm:00Z → 08:mm+2:00Z`, i.e. **120 s**, while the baseline arm declares
  1000/1200/1500/1800/2000 s and the comparison arm 700/800/1100/1300/1400 s;
- `MET-FLOW-001` is `measured` **120.0 s** for `cohort-baseline-member-0-20260730`, whose declared
  arm sample is 1000.0 s — the two are never compared;
- the record's `MET-FLOW-025` provider rows sum to **92.11071 USD** while the arms declare 75.0 and
  45.0 USD; `check_metric_completeness` only requires that each lane provider has *some* measured
  cost row (`ai_contracts.py:3505`), never that `MET-AI-060` equals their sum.

Multiplying every baseline sample by ten and keeping every declared median, dispersion, total and
ratio self-consistent yields a fabricated **92.67 percent** lead-time and **94.00 percent** cost
reduction, zero findings, `KGI-DEV-005: achieved`. `MET-AI-060` must equal the comparison total, but
the comparison total is also author-chosen, so the binding is circular.

**R1c — the other four KGIs are reachable by declaration.** Two new gates do fire in isolation
(`measured-without-repository-evidence` for operator-only evidence, `zero-target-audit-event-required`
for a citation-only guardrail), but both are satisfied by *any* resolvable subject and *any* in-window
event of a broad kind set (`ai_contracts.py:3246`, `:3261`). On the committed exemplar, declaring

- `MET-FLOW-028 = 3` and `MET-FLOW-027 = 7/7` (both free scalars/rates) with the policy document as
  their only evidence, and
- `MET-AI-040/041/042/075` and `MET-FLOW-037` as `measured 0`, citing the policy document plus
  `event:mr-list-focused-first-pass-failure-20260730-091908` — a *gate-failure* validation event that
  observes nothing about escaped defects, quota stoppages or silent downgrades

produces **zero findings** and `KGI-DEV-001: achieved`, `KGI-DEV-002: achieved`,
`KGI-DEV-004: achieved`. `KGI-DEV-003` is the one materially harder case, because `MET-FLOW-019` is
recomputed from evidence-bounded lane state.

### MUST-BLOCKER R2 — A finite, schema-valid number still crashes validation

The non-finite closure is real and complete: `parse_constant` refuses `NaN`/`Infinity` tokens at the
file boundary, an overflowing literal such as `1e400` is caught by both `schema-non-finite-number`
and the whole-document `non-finite-number` scan, and `parse_utc_nanos` genuinely separates
`...:55.000000001Z` from `...:55Z`. The published promise, however, is that "every malformed value
produces a stable finding; nothing raises" (`development-flow-kgi-kpi.md`, *Exact representation*).

`check_measurement` still converts an exact `Decimal` back to a binary float for the precision rule:
`allowed = max(0, -math.floor(math.log10(float(uncertainty))))` (`ai_contracts.py:3168`). A JSON
**integer** of 401 digits is finite, is decoded exactly by Python, and survives every non-finite
guard, but `float(Decimal(10**400))` is `inf`:

```
uncertainty = 1000…0 (401 digits) on the operator_observed MET-FLOW-010 row
  validate_development_flow_record       -> OverflowError: cannot convert float infinity to integer
  check_development_flow_records (route) -> OverflowError: cannot convert float infinity to integer
```

The same value written as the float literal `1e400` is handled correctly
(`non-finite-number`, `schema-non-finite-number`), which is exactly why the integer path was missed.
This is R2's own residual owner requirement — "turn all malformed values into stable findings" —
still open, and it is a hard gate crash rather than a rejection.

Secondary, same owner: the file boundary is still lossy in the underflow direction. `1e-400` decodes
to `0.0`, so `MET-FLOW-026` (target zero) can declare a **nonzero** loss in the file and be treated as
an exact measured zero with zero findings. `exact_decimal` is exact with respect to the decoded
*float*, not to the literal the reviewer reads, so the policy's "quantities as **exact decimals**"
overstates what survives parsing.

### MUST-BLOCKER R3 — A sealed final record can be edited and resealed, or sealed against nothing

`check_final_provenance` (`ai_contracts.py:2133`) is a real improvement: an in-place edit with a
recomputed `record_sha256` now returns `final-record-provenance-mismatch`, and an unknown sealing
commit returns `final-record-provenance-unresolved`. Both were confirmed in a temporary repository
with real commits. Two holes remain, and both defeat the published immutability claim.

| Probe (real Git history, temp repo) | Result |
|---|---|
| H4a honest two-step seal (commit content, then name the commit) | zero findings — positive case holds |
| H4b in-place edit plus recomputed `record_sha256` | `final-record-provenance-mismatch` |
| **H4c edit the sealed record, commit it, then move `sealed_in_commit` to that new commit** | **zero findings** |
| **H4d seal against a `commit-tree` object reachable from no ref, no branch, not an ancestor of `HEAD`, absent from `git log`** | **zero findings** |

Nothing checks that `sealed_in_commit` is the commit that *first introduced* the content — the
docstring says it does — nor that the commit is reachable, nor that the recorded blob equals the
working-tree blob (the docstring claims "must equal both the declared blob id and the working-tree
blob id"; the working-tree blob id is never computed). Because `sealed_content_digest` excludes the
sealing fields, resealing is a two-commit operation with no residue the checker can see. H4d is worse
than H4c: a dangling commit leaves no history trace at all, and it would not survive a clone or `gc`,
so the seal is also not reproducible for anyone else.

### MUST-BLOCKER R4 — The supersession graph is enforced from one side only

`_validate_flow_batch` (`ai_contracts.py:4141`) validates `supersedes` thoroughly: cross-batch,
reciprocity, predecessor status, revision monotonicity, and cycles all fire, and per-batch
independence is genuinely fixed (two active records in different batches now return zero findings,
closing the R1 regression). `superseded_by` is checked only for resolvability, and only globally
(`:4234`).

| Probe (directory route) | Result |
|---|---|
| two independent active batches | zero findings — R1 regression closed |
| honest reciprocal in-batch chain | zero findings |
| non-reciprocal `supersedes` | `supersession-not-reciprocal`, `unresolved-supersedes-record` |
| **one-way `superseded_by` naming a record in another batch, never reciprocated, on a record that stays `status: active`** | **zero findings** |
| **a batch in which every record points out of the batch, leaving zero unsuperseded heads** | **zero findings** |

The head check only fires for `len(heads) > 1` (`:4257`), so "exactly one unsuperseded head"
(`flow-metrics/README.md`, `development-flow-kgi-kpi.md` *Record Provenance And Supersession*) is
enforced as "at most one". A batch can therefore have no current authority at all, and a record can
simultaneously claim to be superseded and to be active. `status == "superseded"` requires
`superseded_by` (`:2112`) but not the converse.

### SHOULD-BLOCKER R5 — The injected validation instant is unbounded and untraceable

Injection was the option R1 explicitly allowed, and it works: at `2026-08-05T00:00:00Z` the exemplar
reports `active-record-review-overdue` and three `in-progress-measurement-overdue`. But the instant
is accepted with no lower bound — not even the record's own `captured_at` — and the CLI records
nothing about it:

```
check_ai_contracts.py --validation-instant 1970-01-01T00:00:00Z   -> AI contract check passed.
CODEFIRE_FLOW_VALIDATION_INSTANT=2026-07-30T09:51:56Z ...          -> AI contract check passed.
```

A checkpoint that is *supposed* to refuse stale claims can therefore be satisfied by an instant 56
years before the record was captured, and the passing output is indistinguishable from an honest run.
The candidate's own test encodes a backdated instant as expected-clean
(`test_ai_tools.py:1743`). One line — refuse an instant earlier than `captured_at`, and echo the
instant used — would close this.

### SHOULD-BLOCKER R6 — Artifact identity is defeated by one byte of padding

`subject_sha256` is recomputed over the artifact's sorted `evidence_refs`
(`ai_contracts.py:2814`), and the literal R1 falsification — relabel the same artifact under an
unused kind — is now `duplicate-artifact-subject`. Adding one already-declared, harmless reference to
the twin changes the subject:

```
artifact A: ["commit:29c421fc…"]                                        kind source_patch
artifact B: ["commit:29c421fc…", "lane:cfr168-python-http-mr-list-…"]    kind counterexample
-> zero findings; the lane now declares 4 artifacts for 3 deliverables
```

Both artifacts bind the same commit and the same deliverable. The published claim — "Two artifacts
with the same evidence therefore have the same subject and the second is refused, so one deliverable
cannot become several unique artifacts by being relabelled" — holds only for a byte-identical
evidence set. Subject identity needs to be derived from the deliverable (its commit/path set), not
from the citation list the author chooses.

### SHOULD-BLOCKER R7 — Diagnostics are bounded in the semantic layer only

Practical `maxItems` now exist on every principal array and fire (`2100` events →
`schema-max-items`), and the semantic layer stops at 512 with `finding-limit-reached`. The structural
layer is not bounded: 622 schema-valid events with one unparseable instant each produced **1713
findings** (`bad-timestamp`, `schema-min-length`, `schema-pattern`, plus the semantic cap). The claim
"a single record emits at most 512 findings before reporting `finding-limit-reached`"
(`development-flow-kgi-kpi.md`) and "cannot make validation unbounded in memory, CPU, or diagnostics"
(`schemas/README.md`) are therefore both inaccurate as written, although `maxItems` does bound the
worst case to roughly ten thousand findings rather than to nothing. This is a claim/enforcement
mismatch, not a denial-of-service.

### SHOULD — Lifecycle completion is satisfiable by six synthetic events, on an active record

`check_lifecycle` (`ai_contracts.py:3910`) genuinely requires `close`, `install`, `dogfood` and
`retrospect` stage evidence at or before the earliest completion, and the terminal completion must
carry `stage: retrospect`; P13 confirms the negative case. The evidence may be any in-window event of
an allowed kind. Appending six `validation`/`review_decision`/`integration`/`lifecycle_complete`
events at `2026-07-30T09:51:55Z`, each with the policy document as its only reference, returns **zero
findings** — and the record remains `status: active`, although the published obligation table says
lifecycle completion requires "The record is `final`". The conditional obligation is enforced in one
direction only (a `final` record needs the event; the event does not need a `final` record).

### COULD — Two small exactness slips in the candidate's own reporting

- The R2 result packet and the commit message both say "**Thirteen** provider and instrument rows drop
  from measured to estimated". The exemplar diff shows **fourteen**: `MET-FLOW-012`, `MET-FLOW-017`,
  `MET-FLOW-030`, `MET-FLOW-025` ×3, and `MET-FLOW-020/021/022/023` ×2 each.
- `unresolved-supersedes-record` is emitted for a dangling `superseded_by` as well as a dangling
  `supersedes`, so the code does not identify which direction failed.

---

## 4. Manager Counterexamples, Re-Run On The Production Route

All eight are genuinely closed. Every row below was executed, not inspected.

| # | Counterexample | Fresh result on this candidate | Prior result |
|---:|---|---|---|
| 1 | Event at `captured_at + 1 ns` | `event-outside-window` | 0 findings |
| 2 | `MET-FLOW-017 = NaN` / `= Infinity`; `denominator = Infinity`; `uncertainty = Infinity` | `non-finite-number` + `schema-non-finite-number` for each; no crash | 0 findings / `OverflowError` |
| 3 | One root plus a disjoint two-node candidate cycle | `candidate-lineage-cycle` | 0 findings |
| 4 | Review and metric due dates past at validation | `active-record-review-overdue`, `in-progress-measurement-overdue` (with an instant injected) | 0 findings |
| 5 | Two active records in different batches | 0 findings — batches independent | wrong `superseded-active-development-flow-record` |
| 6 | `doc:` final component symlinked to `/etc/passwd` | `missing-repository-document` | 0 findings |
| 7 | Agent locator with `../`, `./`, doubled slash | `bad-evidence-locator` for all three | 0 findings |
| 8 | Arbitrary edit plus recomputed `record_sha256` | `final-record-provenance-mismatch` | 0 digest findings |

Counterexamples 4, 5 and 8 are closed only in the exact shapes tested; §3 R5, R4 and R3 record what
their neighbourhoods still accept.

### 4.1 The 17 declared production probes

Independently replayed with my own driver; the two probes that carry no mutation callable in the
candidate's table were rebuilt from scratch in temporary repositories.

| Probe | Expected code | Observed |
|---|---|---|
| P1 nanosecond event | `event-outside-window` | PASS |
| P2a/P2b/P2c/P2d non-finite value / value / denominator / uncertainty | `non-finite-number` | PASS (4/4) |
| P3 disjoint candidate cycle | `candidate-lineage-cycle` | PASS |
| P6 `doc:` symlink escape (rebuilt here) | `missing-repository-document` | PASS |
| P7a/P7b/P7c locator traversal / dot / doubled slash | `bad-evidence-locator` | PASS (3/3) |
| P9 forged `repo://` prefix digest (rebuilt here) | `evidence-prefix-digest-mismatch` | PASS; honest control clean; over-long `prefix_bytes` → `evidence-prefix-bytes-mismatch` |
| P10 measured value on operator-only evidence | `measured-without-repository-evidence` | PASS |
| P11 relabelled artifact, identical evidence | `duplicate-artifact-subject` | PASS (see §3 R6 for the padded variant) |
| P12 review lanes reclassified to `none` | `lane-mutation-none-unproven` | PASS |
| P13 lifecycle completion without Install/Dogfood | `lifecycle-complete-missing-stage` | PASS |
| P14/P15 KGI vocabulary / required metric | `kgi-metric-not-allowed` / `kgi-required-metric-missing` | PASS |

---

## 5. Frozen Matrix Replay And Test Non-Vacuity

### 5.1 Independent replay

- Semantic mutations: **55 total, 55 exact expected codes present**, no crashes.
- Structural controls: **15 total, 15 exact expected codes present**, no crashes.
- Baseline reference record: **zero findings**, both through the record validator and the directory
  validator.
- Repository suite: **108 tests, OK**.

### 5.2 Non-vacuity, improved but still incomplete

Real improvements over the previous candidate: the sealed-record tests build actual Git history with
real commits, the cohort test recomputes its own median and Tukey hinge with independent helpers, the
directory test now asserts batch **independence** instead of encoding the old global-newest bug, and
`independent_canonical_digest` deliberately avoids `sha256_json`. Remaining gaps, all of which this
review exercised and the suite does not:

- no case covers a finite-but-huge numeric (§3 R2) — the non-finite cases pass precisely because they
  take the other branch;
- no case covers resealing in a later commit or sealing against an unreachable object (§3 R3);
- no case covers a one-way or cross-batch `superseded_by`, or a zero-head batch (§3 R4);
- no case covers a padded artifact subject (§3 R6);
- no case asserts that the cohort's declared threshold is at least the published 0.20, or that arm
  samples relate to member lanes (§3 R1a/R1b);
- `test_injected_validation_instant_refuses_overdue_claims` asserts that a **backdated** instant is
  clean, which fixes the current behaviour in place rather than flagging it (§3 R5);
- the cohort acceptance fixture itself contains the 120 s-lane/1000 s-sample contradiction, so it
  cannot detect R1b.

Passing 108/108 again verifies the implemented contract, not the missing contract.

---

## 6. Prior-Finding Disposition

Standard as bound: `closed` only when schema, semantic branch, policy, exemplar and a non-vacuous
negative test jointly close the smallest falsification case.

### 6.1 The 20 previously partially-closed rows

| ID | What the candidate added | Smallest surviving falsification | Disposition |
|---|---|---|---|
| F1 | Semantic layer extended with per-KGI targets, cohort recomputation, evidence verification, graph traversal | Huge finite `uncertainty` crashes; four KGIs still declarable (§3 R1, R2) | **partially closed** |
| F4 | `parse_utc_nanos` integer nanoseconds throughout | none found — nanosecond ordering, leap seconds, offsets, and reordering all behave | **closed** |
| F6 | Injected validation instant; status/value branch unchanged | Backdated instant silently clean (§3 R5) | **partially closed** |
| F7 | `exact_decimal`, whole-document non-finite scan, fail-closed helpers | `float()` re-entry crashes on a finite huge integer; `1e-400` becomes an exact zero (§3 R2) | **partially closed** |
| F9 | `doc:` through the symlink-safe reader, normalized locators, recomputed `repo://` bytes/digest, provenance seal | Reseal and dangling-commit seal (§3 R3); no existing prefix is `repo://`, so nothing in the record is byte-verified (disclosed) | **partially closed** |
| F11 | `FLOW_KGI_REGISTRY` vocabulary plus recomputed targets | `KGI-DEV-001/002/004` achieved by declaration (§3 R1c) | **partially closed** |
| F12 | Member resolution, class check, arm partition, recomputed median/dispersion/ratios/non-inferiority | Self-declared threshold and free arm samples (§3 R1a/R1b) | **partially closed** |
| F13 | `MET-AI-060` must equal the comparison arm total; component presence required | Comparison total is itself declared; component rows are never summed (§3 R1b) | **partially closed** |
| F14 | `subject_sha256` over the sorted evidence set | One padding reference splits a deliverable (§3 R6) | **partially closed** |
| F-S2 | Per-batch freshness, active uniqueness; injected instant | Backdated instant (§3 R5); the cross-batch regression itself is fixed | **partially closed** |
| F-S3 | Provenance seal, reciprocal monotone acyclic supersession | Reseal (§3 R3); one-way/cross-batch `superseded_by`, zero heads (§3 R4) | **partially closed** |
| F-S4 | Symlink-safe `doc:`, normalized locators, verified prefixes, resolved cohort members | Only `repo://` prefixes are verifiable and the record has none (disclosed); §3 R3 seal gap | **partially closed** |
| F-S7 | Scorecard rows updated; cohort class enforced; prefix verifiability published | `runtime_version` still optional, so `MET-AI-075`/`MET-FLOW-037` stay `absent` (waived, disclosed) | **partially closed (waived)** |
| F-S9 | `MET-AI-075` and new `MET-FLOW-037` required for `KGI-DEV-004` | Both are declarable measured zeroes (§3 R1c) | **partially closed** |
| F-S10 | `mutation_status` bounded below by lane evidence; `none` needs a terminated, artifact-free, zero-second lane | none found — P12 and the coverage recomputation hold | **closed** |
| F-S12 | Candidate lineage traversed for cycles and connectivity | none found — P3 and the disconnected case both reject | **closed** |
| F-S13 | Zero-target rows need an in-window audit event and repository evidence | Any in-window event of a broad kind set qualifies; a gate-failure event backs three escape zeroes (§3 R1c) | **partially closed** |
| F-S14 | Non-finite uncertainty is a stable finding | Finite huge integer crashes (§3 R2) | **partially closed** |
| F-C2 | Every prefix declares `verifiability`; the storage lane's claim is explicitly unverifiable | The lane still has `artifacts: []`, one operator-only prefix, no trusted-signal event, and an outcome claiming "six or more" blockers | **partially closed (disclosed)** |
| F-C7 | Vacuous-limitation blacklist and patterns retained; 14 limitations recorded | Honesty is still prose-pattern based; an evasively-worded limitation passes | **partially closed (inherent)** |

Totals for these 20 rows: **3 now closed** (F4, F-S10, F-S12), **17 still partially closed**, of
which 4 are disclosed/waived/inherent limitations and **13 carry an executable falsification proved
above**.

### 6.2 Regression check on the 15 previously closed rows

F2, F3, F5, F8, F10, F-S1, F-S5, F-S6, F-S8, F-S11, F-C1, F-C3, F-C4, F-C5, F-C6 were re-exercised
through the 55/15 replay, the 17 probes, and the exemplar audit. **No regression.** One qualification
on F3: the canonical stage reconciliation holds, but lifecycle completion is now accepted on an
`active` record contrary to the published obligation (§3, *Lifecycle completion*), so F3's neighbour
obligation has weakened even though its original falsification stays closed.

### 6.3 R1–R7 owner-level disposition

| Owner | Required residual closure | Status |
|---|---|---|
| Semantic validator (R2) | Exact instants/quantities; refuse every non-finite; all malformed values become stable findings | **partially closed** — crash on a finite huge integer; underflow to exact zero |
| KGI / cohort (R1) | Per-KGI metrics and target arithmetic; members resolve once, match class, arms bound, recompute median/dispersion/20-percent/cost/non-inferiority | **partially closed** — threshold self-declared, arm samples and totals undeclared-derived, four KGIs declarable |
| Evidence / references (R3) | Symlink-safe `doc:`, normalized locators, recomputed bytes/digests, refuse unverifiable evidence for decisions | **closed for references**, **partially closed for seals** (reseal, dangling commit) |
| Record lifecycle (R3, R5) | Bind sealed bytes to an immutable subject; reciprocal same-batch monotone acyclic supersession with one head; per-batch clock-independent freshness | **partially closed** — reseal, one-way `superseded_by`, zero heads, unbounded instant |
| Lineage / lifecycle (R4) | Traverse candidate lineage; require Close/Install/Dogfood/Retrospect | **closed for lineage**, **partially closed for completion** (declarable, allowed on an active record) |
| Anti-gaming (R6) | Mutation state bound to evidence; artifacts identified by immutable subject; evidence-derived denominator | **closed for mutation state and denominator**, **partially closed for artifacts** (padding) |
| Schema / tests (R7) | Finite-number semantics, practical `maxItems`, all probes on the production route, no cross-batch interference encoded as expected | **mostly closed** — bounds and probes real; diagnostics bound applies to the semantic layer only; a backdated instant is encoded as expected |
| Docs / policy | Publish no claim the checker does not enforce | **partially closed** — six overstatements (§11) |

---

## 7. Adjudication Of The Seven Handed-Over Hypotheses

Each was tested independently, on the production route or through the real file boundary.

| # | Hypothesis | Verdict | Evidence |
|---:|---|---|---|
| 1 | Raw JSON numerics may still collapse through binary float (underflow / threshold precision) | **Verified, in a different shape than expected** | Threshold precision is largely safe: `Decimal(repr(x))` keeps `0.19999999999999999` distinct from `0.2`. But `1e-400` silently becomes an exact `0.0` guardrail, and a finite 401-digit **integer** `uncertainty` re-enters `float()` and crashes the gate with `OverflowError` (§3 R2) |
| 2 | The injected validation instant may be arbitrarily backdated | **Verified** | `--validation-instant 1970-01-01T00:00:00Z` and the env-var form both pass silently; no lower bound, no echo of the instant used (§3 R5) |
| 3 | `superseded_by` may be asymmetric / cross-batch / one-way | **Verified** | One-way cross-batch `superseded_by` on an `active` record: zero findings; a batch with zero heads: zero findings; the `supersedes` direction is fully checked (§3 R4) |
| 4 | A later commit may edit and reseal a final record | **Verified, and worse** | Reseal against a later commit: zero findings; seal against an unreachable dangling `commit-tree` object: zero findings (§3 R3) |
| 5 | `KGI-DEV-005` may accept a self-declared improvement threshold below the fixed 20 percent policy | **Verified** | `required_improvement_ratio` 0.005 with a 1 percent reduction, and 0.0 with identical arms, both `achieved` with zero findings (§3 R1a) |
| 6 | Cohort arm samples/costs may not derive from resolved member lanes / provider rows | **Verified** | The candidate's own fixture declares 1000 s samples for 120 s lanes; a ×10 baseline inflation yields a fabricated 92.67 percent reduction with zero findings; provider rows sum to 92.11 USD against declared 75/45 (§3 R1b) |
| 7 | Artifact subject padding and bounded diagnostics may remain gameable | **Verified for padding; partly refuted for diagnostics** | One padding reference splits a deliverable into two artifacts (§3 R6). Array `maxItems` are real and fire; the 512-finding bound covers the semantic layer only — 1713 findings observed (§3 R7) |

No hypothesis was refuted outright. Two — 1 and 7 — were confirmed in a materially different form
than the hypothesis text suggested, which is recorded above rather than smoothed over.

---

## 8. Positive Cases Verified Without Sharing Production Helpers

All three acceptance cases the bound report demanded hold. Every expected value below was recomputed
locally (`Fraction` arithmetic, my own canonical-JSON SHA-256, my own Git commands).

| Case | Construction | Result |
|---|---|---|
| Committed exemplar | as committed | zero findings, record route and directory route |
| Valid two-batch directory | second batch built here (`second-batch-20260730`, distinct `batch_id`), both `active` | zero findings; batches independent |
| Valid reciprocal supersession chain | predecessor `status: superseded` + `superseded_by`, successor `revision: 2` + `supersedes` | zero findings |
| Sealed final record on real Git provenance | temp repo, real `git init`/`add`/`commit`, two-step seal, `record_sha256` from my own canonical digest | zero findings; every tampering variant rejects with its specific code |
| One complete cohort | candidate's fixture, arithmetic recomputed here | zero findings; independent median 1500.0 / 1100.0, Tukey IQR 800.0 / 600.0, totals 75.0 / 45.0, lead reduction 0.266667, cost reduction 0.4 — all match the declared values |

The cohort's internal arithmetic is genuinely correct and independently reproducible. What it is
arithmetic *about* is the finding in §3 R1b.

---

## 9. Exemplar Honesty Audit

The committed CFR-168 record is the strongest part of this candidate and became more candid, not less:

- 44 measurement rows: **2 `measured`** (`MET-FLOW-019` = 2/6, `MET-FLOW-028` = 0), 27 `estimated`,
  11 `absent`, 3 `in_progress`, 1 `unavailable`;
- fourteen rows were demoted `measured → estimated` because operator-only logs cannot back a
  measurement; none was promoted;
- all seven evidence prefixes declare `operator_only_unverifiable`, and the record says so;
- `MET-AI-060` stays `unavailable`; all twelve zero-target guardrails are `absent` or estimated, none
  a false zero;
- `MET-FLOW-037` was added as `absent` so `KGI-DEV-004` is evaluable at all, with the honest note that
  the absence of a quota incident is not an audit;
- all five KGIs remain `in_progress` or `at_risk`; no cohort is registered;
- 14 limitations, none vacuous;
- the storage review lane still carries no artifact and an unverifiable "six or more blockers" outcome
  (F-C2), now explicitly marked unverifiable.

An honest "not decision-capable yet" record is exactly right. The blocking problem is unchanged from
R1 in kind: the contract cannot enforce this honesty on the next record.

---

## 10. Decision Capability Audit

| Question | Assessment |
|---|---|
| Is the contract mechanically valid? | **No for decision authority.** The exemplar passes, and every gate passes, but production counterexamples still fabricate all five KGI outcomes and one crashes the gate outright. |
| Is the committed record factually honest? | **Yes, materially.** Two measured rows, everything unverifiable demoted or declared absent, no terminal claim. |
| Which KPIs are usable now? | Finite interval diagnostics, the recomputed `2/6` checkpoint coverage, measured zero throughput, the estimated resource and cost rows read as estimates. |
| Which are not decision-capable? | Trusted-integration lead time, evidence completeness, flake classification, cross-provider cost, every safety guardrail, flow efficiency, useful parallelism, and any cohort quantity. |
| Can `KGI-DEV-005` be evaluated now? | **No**, and the checker still permits a fabricated `achieved` — including with two identical arms. |
| Can policy authorize a process change from this contract? | **Mechanically yes, incorrectly.** The policy still treats a passing semantic check as the decision boundary while the checker accepts declared thresholds, declared arm samples, and declared guardrail zeroes. |
| Is the gate itself safe to run? | **Not fully.** One schema-valid record makes it raise `OverflowError` instead of failing closed. |

---

## 11. Published Claims Versus Enforcement

Every row below is a claim added by this candidate that the code does not fully keep. Each must be
either implemented or narrowed.

| Claim | Where | Reality |
|---|---|---|
| "Every malformed value produces a stable finding; nothing raises." | `development-flow-kgi-kpi.md`, *Exact representation* | A finite 401-digit integer `uncertainty` raises `OverflowError` (§3 R2) |
| "quantities as **exact decimals**" | same section | Exact with respect to the decoded float, not the file literal; `1e-400` becomes `0.0` |
| "a single record emits at most 512 findings before reporting `finding-limit-reached`" | same section; `schemas/README.md` | 1713 findings observed; only the semantic layer is capped (§3 R7) |
| "`sealed_in_commit` names the commit that introduced the sealed content" / "a sealed record cannot be edited in place" | *Record Provenance And Supersession*; `flow-metrics/README.md` | Any commit, including an unreachable one, may be named after the fact (§3 R3) |
| "reciprocal … same-batch … it must leave **exactly one** unsuperseded head" | same section; `flow-metrics/README.md` | Enforced from the `supersedes` side only; zero heads accepted (§3 R4) |
| "Both reductions must reach `required_improvement_ratio`" alongside "at least 20%" as the KGI definition | *Cohort* table and `:52` | The record chooses its own threshold, including 0.0 (§3 R1a) |
| "one deliverable cannot become several unique artifacts by being relabelled" | *Unique Artifacts* | True only for a byte-identical evidence set (§3 R6) |
| "Lifecycle completion … The record is `final`" | *Event obligations* table | A `lifecycle_complete` event is accepted on an `active` record |

`project-management.md` and the Phase-2 scorecard row are accurate and appropriately hedged; the
scorecard's Rule 1 and Rule 4 waivers correctly disclose that no prefix is repository-verifiable and
that `runtime_version` was never captured.

---

## 12. Required Next Action And Residual Owners

Do not integrate this candidate. A successor must close every row below and add a production-route
negative test for each.

| Owner | Required closure |
|---|---|
| Semantic-validator owner | Remove the `float()` re-entry in the precision rule (compare exponents on the `Decimal`), or bound `uncertainty`/`value` magnitude in the schema **and** keep the semantic path exact; make every remaining conversion crash-free. Decide and document what an underflowing literal means — refuse it, or state that quantities are exact only to double precision. |
| KGI/cohort owner | Enforce the published 0.20 as a floor on `required_improvement_ratio`. Derive each arm's `lead_time_seconds` and `costs_usd` from the member lane's own instants and the record's measured provider rows, and require `MET-AI-060` to equal the summed component rows. Require a zero-target guardrail's audit event to be an audit *of that guardrail*, not any in-window event, and require throughput/completeness values to be derived from the record's own integration events and evidence set rather than declared. |
| Record-lifecycle owner | Require `sealed_in_commit` to be the earliest commit whose blob for that path carries the sealed content, require it to be reachable from `HEAD`, and compare the working-tree blob id as the docstring already claims. Validate `superseded_by` symmetrically (in-batch, reciprocal, monotone) and require exactly one head, not at most one. Refuse `superseded_by` on an `active` record. |
| Freshness owner | Refuse a validation instant earlier than the record's `captured_at`, echo the instant used in the checker's output, and record in the policy that an omitted instant means the freshness control did not run. |
| Anti-gaming owner | Derive artifact subject identity from the deliverable (commit/path set), not from the author-chosen citation list, so padding cannot split one deliverable. |
| Lifecycle owner | Require `status: final` for any `lifecycle_complete` event, and bind each closing-stage event to evidence of that stage rather than to any in-window event. |
| Schema/test owner | Bound structural diagnostics as well, or correct the published bound. Add negative tests for: huge finite numerics, reseal, dangling-commit seal, one-way/cross-batch `superseded_by`, zero-head batch, padded artifact subject, sub-20-percent cohort threshold, arm samples contradicting member lanes, and a backdated validation instant. Replace the assertion that a backdated instant is clean. |
| Docs/policy owner | Narrow or implement each of the eight claims in §11. Correct "thirteen" to "fourteen" demoted rows. |

Required focused acceptance evidence for the next candidate: everything the bound report already
required, plus (9) no schema-valid record can raise; (10) a resealed or dangling-sealed record
rejects; (11) `superseded_by` violations reject from either direction and a zero-head batch rejects;
(12) a cohort below the published 20 percent, or whose arm samples contradict its member lanes,
rejects; (13) a padded artifact subject rejects; (14) a backdated validation instant rejects.

---

## 13. Incident: The Failed Fugu Round-2 Attempt (Not A Product Finding)

Reported to me by the manager and recorded here for completeness. It is kept strictly separate
because it produced no verdict and had no influence on §§1–12.

| Item | Statement |
|---|---|
| What was attempted | An independent Fugu Ultra rereview of the same exact candidate `0944bf3deba9e753132b3f71716b5c009a8fc30f`. |
| Reported failure | Repeated exec / model-refresh infrastructure errors before any report was written or committed. |
| Verdict produced | **None.** No `ACCEPT`, no `REQUEST_CHANGES`, no partial finding list. |
| Repository effect | None observable. No `development-flow-kgi-kpi-r2-rereview-fugu-*` directory, batch file, incident file, or commit exists on any ref in this clone; `git log --all` shows nothing for that attempt. |
| What I verified | Only the absence of artifacts. The failure mode itself is outside this repository, so I can neither confirm nor refute the reported cause. |
| Effect on this review | None. This review is a fresh independent adjudication; the seven hypotheses were supplied as questions and were each tested from scratch, and none was accepted on authority. |
| Recommended follow-up | If the manager wants the incident on the record, publish it under a `…-launch-…/incident.md` path in the established convention with the exact failing invocation and timestamps. That is a manager action; this reviewer did not fabricate one. |

---

## 14. Handoff

- **Decision:** `REQUEST_CHANGES` on exact candidate `0944bf3deba9e753132b3f71716b5c009a8fc30f`.
- **Files changed by this reviewer:** only
  `docs/development/ai/swarm-results/development-flow-kgi-kpi-r2-rereview-claude-fallback-20260731/review.md`.
- **Validation:** 108/108 repository tests, 55/55 semantic codes, 15/15 structural codes, 17/17
  declared probes, all seven `check_ai_development` components, contracts, project docs and
  `git diff --check` all pass on this candidate. The adversarial probes in §3 nevertheless fabricate
  five terminal KGI outcomes, reseal an immutable record, and crash the gate.
- **What genuinely closed:** all eight manager counterexamples, nanosecond ordering, non-finite
  numbers via every float path, `doc:` symlink escape, locator normalization, `repo://` byte and
  digest recomputation, candidate lineage as a graph, evidence-derived mutation state and checkpoint
  denominator, per-batch independence, array bounds, and a materially more candid exemplar.
- **What blocks:** §3 R1 (a–c), R2, R3, R4 as MUST; R5, R6, R7 and the lifecycle-completion gap as
  SHOULD; 17 of the 20 bound partially-closed rows remain partially closed, 13 of them with an
  executable falsification.
- **Risks of accepting anyway:** the contract would publish that a passing semantic check makes a
  record decision-grade while still admitting a fabricated `KGI-DEV-005`, a resealed final record, a
  batch with no authoritative head, and one record shape that makes the gate raise instead of fail
  closed.
- **Next action:** return the ticket to the residual owners in §12. Do not integrate, Close, Install,
  Dogfood, or Retrospect this candidate.
