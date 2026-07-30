# Development-Flow KGI/KPI Framework — Independent Adversarial Review (Claude)

Review date: 2026-07-30
Reviewer: Claude Opus 5 (`claude-opus-5`), independent control-plane review
Subject commit: `bfa7f08` (`feat(ai): add development flow KGI and KPI records`), base `0be1039`
Canonical head at review time: `dd993cc` (clean)

**Decision: `REQUEST_CHANGES`.**

The framework is a serious, well-shaped measurement policy. It picks the right north star (trusted
integration, not activity), it separates KGI from diagnostic KPI, it names anti-metrics explicitly,
it refuses to invent a baseline for `KGI-DEV-005`, and its exemplar record is candid about its own
gaps in `limitations`. Nothing in it is dishonest.

It is nevertheless **not yet usable for bottleneck, throughput, or efficiency decisions, and it can
silently accept misleading records.** Three structural reasons:

1. **The contract is purely structural.** 55 hand-crafted semantic mutations of the shipped record —
   inverted intervals, junk timestamps, `measured` with no value, `unavailable` with value `0`,
   `denominator: 0`, fabricated commits, dangling cross-references, every KGI flipped to `achieved`
   with `lane_runs: []` — were **all accepted** by `check_ai_contracts.py`. 15 control mutations of
   *shape* were correctly rejected, so the harness works; the semantic layer simply does not exist.
2. **The framework's own reference record already violates the framework's own required-events
   rule, and the checker passes it.** `lane_start`, `trusted_signal`, `mutation`,
   `candidate_ready`, `review_decision`, `integration`, `reroute` and `decision` events are absent
   entirely; not one of the four lanes has a `lane_start` event.
3. **The headline decision rule (`KGI-DEV-005`) is not a decision rule.** n=5, median lead time and
   total cost, a 20% threshold, "non-inferior" quality on zero-target metrics, no cohort
   pre-registration, no censoring rule, no per-class minimum, no dispersion treatment. The
   80%/50% triggers built on the same n=5 flip on a single packet.

There is also one **presently misleading measurement in the shipped record** (F5): `MET-FLOW-004`
for the MR-list lane understates lane-start-to-first-checkpoint by 363.5 s (27.5%), because it is
measured from the resumed session start while the record's own `provider_error` event proves the
lane began ~5.7 minutes earlier.

The unrelated adapter work in `bfa7f08..dd993cc` (`role-routing-matrix.json`,
`model-routing-policy.md`, `run_ai_role.py`, `validate_ai_runtime.py`, `test_ai_tools.py`) is **not**
attributed to this framework. It is referenced once, in F-S9, only because it demonstrates a class
of drift (`0.144` vs `0.146` adapter selection) that a flow record structurally cannot express.

---

## 1. Exact subject and method

### 1.1 Subject

`git diff --stat 0be1039 bfa7f08` — 11 files, +1087/-3:

| File | Change |
|---|---|
| `docs/development/ai/development-flow-kgi-kpi.md` | new, 196 lines — the policy |
| `docs/development/ai/schemas/development-flow-record.v1.schema.json` | new, 268 lines — the contract |
| `docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json` | new, 552 lines — the exemplar record |
| `docs/development/tools/check_ai_contracts.py` | +31 — schema registration, `flow-metrics/*.json` existence + schema validation + duplicate `record_id` |
| `docs/development/tools/check_ai_acceleration_docs.py` | +2 — two required-file entries |
| `docs/development/project-management.md` | +25 — governance section |
| `docs/development/README.md`, `ai/README.md`, `ai/acceleration-master-plan.md`, `ai/schemas/README.md`, `history.md` | index/registration rows |

`git diff --stat bfa7f08 dd993cc` — 7 files, all Codex-adapter routing work; **zero** overlap with
the files above.

### 1.2 Sources read

`AGENTS.md`; `docs/development/ai/development-flow-kgi-kpi.md`;
`docs/development/ai/schemas/development-flow-record.v1.schema.json`;
`docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json`;
`docs/development/tools/check_ai_contracts.py`; `docs/development/tools/ai_contracts.py`
(`validate_json_schema`, lines 89–205); `docs/development/tools/check_ai_acceleration_docs.py`;
`docs/development/project-management.md` (§Cycle Lifecycle, §Development-System KGI/KPI);
`docs/development/README.md`; `docs/development/ai/README.md`;
`docs/development/ai/acceleration-master-plan.md`; `docs/development/history.md`;
`docs/development/exploration/phase-2/experiment-catalog-and-kpi-scorecard.md` (§19 KPI Hierarchy,
§20 Provisional Targets, §21 Anti-Metrics, §24 Promotion Rule, §25 Promotion Status);
`docs/development/tools/test_ai_tools.py`.

### 1.3 Method and commands actually run

All runs were bounded, local, network-free, with `PYTHONDONTWRITEBYTECODE=1` so no repository cache
was created. Fixtures were built in `/tmp/flowaudit/`, never in the repository.

| Step | Command | Result |
|---|---|---|
| Baseline contract gate | `python3 docs/development/tools/check_ai_contracts.py` | `AI contract check passed.` |
| Baseline docs gate | `python3 docs/development/tools/check_ai_acceleration_docs.py` | passed |
| Docs inventory | `python3 docs/development/tools/check_project_docs.py` | passed, 561 Markdown files |
| Mutation matrix | 55 in-memory mutations of the exemplar → `validate_json_schema(rec, schema)` | 55 ACCEPT, 0 REJECT |
| Control matrix | 15 shape mutations (wrong `const`, extra property, missing required, 63-hex sha, `kgi_status: []`, bad enums, `unique_artifact_count: -1`, …) | 15 REJECT — harness verified live |
| Checker-level tests | symlink overlay root in `/tmp` with a mutated `flow-metrics/` | see §5.2 |
| Arithmetic | independent recomputation of every duration and byte value in the record | see §4.3 |
| Reference resolution | `git cat-file -t` on all 8 commit refs; filesystem check on both `doc:` refs | all resolve |

`git status --short` was empty before and after every step until the single report commit.

### 1.4 Explicitly not done

No Cargo, build, install, demo, or full-suite runs. No network calls. No edits to the framework,
product source, tests, issue state, or existing reports. No repository caches. The four
`second-agent://` evidence prefixes were **not** resolved or hash-verified — they are not reachable
from the repository, which is itself finding F9.

---

## 2. Findings (severity-ordered)

All findings require repair. `MUST` blocks acceptance; `SHOULD` and `COULD` are still repair items,
per the review instruction.

### MUST

#### F1 — The record contract has no semantic layer; 55/55 adversarial mutations were accepted

`validate_json_schema` in `ai_contracts.py:89` is a deliberately small structural evaluator, and the
new block in `check_ai_contracts.py:143-171` adds exactly two things beyond it: "at least one
`*.json` exists" and "no two records share a `record_id`". There is no
`validate_development_flow_record` analogue to the existing `validate_task_packet` /
`validate_review_packet` / `validate_context_manifest` functions, which *do* carry semantic checks
(digest recomputation, patch identity, live-file verification).

Consequence: a record can be internally contradictory in every dimension the policy cares about and
still be reported as a passing contract. Full matrix in §5.1. The control set in §5.1b proves the
harness rejects malformed *shape*, so this is an absence of validation, not a test artifact.

#### F2 — The exemplar record violates the policy's own Required Events list, undetected

`development-flow-kgi-kpi.md` §Required Events mandates 11 event classes. The shipped record's 8
events cover `incident`(1), `recovery`(1), `provider_error`(1), `checkpoint`(2), `validation`(1),
`lane_stop`(1), `resource_sample`(1). Absent from the record entirely:

`lane_start`, `trusted_signal`, `mutation`, `candidate_ready`, `review_decision`, `integration`,
`reroute`, `decision`.

Concretely: four lanes are declared in `lane_runs`, and **not one has a `lane_start` event** —
required event #1. Lane `cfr168-rust-storage-independent-review-fugu-20260730` has **zero** events
of any kind. Nothing in the schema requires an event per lane, an event per required class, or any
`lane_runs` ↔ `events` correspondence. The reference implementation of the policy therefore already
demonstrates that the policy's central obligation is unenforced.

#### F3 — Three named flow metrics are underivable, and the stage vocabulary contradicts the canonical lifecycle

The event `kind` enum has a single `validation`, a single `integration`, and a `review_decision`
with no `review_start`. But:

- `MET-FLOW-005` "Validation queue time: **validation-ready** to actual gate start" — neither
  endpoint has an event kind or a lane field.
- `MET-FLOW-006` "Review lead time: review-ready exact revision to bound decision" — no review-start
  or review-ready marker exists.
- `MET-FLOW-007` "Manager integration time: accepted candidate **entering the integration queue** to
  focused post-integration gate completion" — `integration` is one undifferentiated kind; queue
  entry, conflict, and post-integration gate result are distinguishable only by free-text `summary`.

The corresponding triggers ("Validation queue exceeds 15% of trusted-integration lead time",
"Manager integration exceeds 25% of cycle time") are therefore inoperable as written.

Separately, `project-management.md` §Cycle Lifecycle defines **8** stages (Discover, Freeze, Plan,
Implement, Close, Install, Dogfood, Retrospect). The new schema's `stage` enum and `MET-FLOW-002`
define **11** (adding `review`, `integrate`, `operations`). `bfa7f08` edits
`project-management.md` — it adds the KGI/KPI governance section — but leaves the canonical
lifecycle at 8 stages. Two authoritative documents now describe different lifecycles, and
`MET-FLOW-002` measures entry/exit for stages that the governing document does not define, with no
entry/exit criteria published for any stage.

#### F4 — Timestamps are unvalidated strings and no interval ordering is enforced

Every temporal field is `{"type": "string", "minLength": 20, "maxLength": 64}`. There is no
`format: date-time` — and adding one would currently *break* the checker, because
`validate_json_schema` fails closed on unsupported keywords (`ai_contracts.py:112-125`).

Accepted (verified):

| Mutation | Result |
|---|---|
| `captured_at: "xxxxxxxxxxxxxxxxxxxx"` | ACCEPT |
| `captured_at: "2026-13-45T99:99:99Z!!!"` | ACCEPT |
| lane `ended_at` = `2026-07-30T07:00:00Z` (1h38m before its `started_at`) | ACCEPT |
| measurement `window_ended_at` = `2026-07-30T00:00:00Z` (before `window_started_at`) | ACCEPT |
| `first_checkpoint_at` before the lane's `started_at` | ACCEPT |
| `measurement_window.started_at` = `2027-01-01`, after `captured_at` | ACCEPT |
| event `at` = `1999-01-01T00:00:00Z` | ACCEPT |

Every duration KPI in the framework is a subtraction of two such strings. A negative or nonsensical
lead time is admissible today, and a negative value in a cohort median is a silent corruption of the
`KGI-DEV-005` decision.

Also: `pattern` is applied with `re.search`, and Python's `$` matches before a trailing newline, so
`"sha256": "<64 hex>\n"` and `"record_id": "cfr168-recovery-20260730\n"` are both accepted despite
anchored patterns.

#### F5 — A presently misleading duration in the shipped record: `MET-FLOW-004` understates by 27.5%

This is not hypothetical; it is in the committed exemplar.

- `MET-FLOW-004` for lane `cfr168-python-http-mr-list-zero-residual-repair-20260730`:
  `value: 956.367528` s, `scope: "Resumed lane start to first durable checkpoint"`,
  window `08:45:14.632472Z → 09:01:11Z`. The arithmetic is exact.
- But the definition in the policy is "**Lane start** to the first committed source/report
  checkpoint after work begins" — not *resumed* lane start.
- The record's own event `mr-list-claude-529-20260730-084456` fires at `08:44:56.252Z` and states
  "ten retries and **345.091 seconds** of provider API time". The original attempt therefore began
  at ≈`08:39:11.161Z`.
- True lane-start → first checkpoint = **1319.839 s**. Recorded: 956.367528 s. **Understated by
  363.472 s = 27.5%.**

Two further defects visible in the same pair:

- The `provider_error` event at `08:44:56.252Z` carries `lane_id` of a lane whose `started_at` is
  `08:45:14.632472Z` — the event **precedes its own lane's start by 18.380 s**. An ordering
  violation, accepted (F4).
- The schema has no field distinguishing an original lane start from a resumed one, and no
  `resumed_from`/`outage_seconds` field. A recovered lane is therefore *systematically* cheaper on
  every start-anchored metric (`MET-FLOW-003`, `-004`, `-001`) than a lane that never failed —
  which inverts the incentive `KGI-DEV-004` is trying to create.

#### F6 — `status` and `value` are independent; `in_progress` is an unbounded escape hatch

The schema requires `status` but never conditions `value` on it. Accepted (verified):

| Mutation | Why it matters |
|---|---|
| `status: "measured"`, `value` key deleted | A "measured" metric with no number passes the gate |
| `status: "unavailable"`, `value: 0` | Directly defeats the policy's "Unknown billing, cache, or token values are `unavailable`, never zero" |
| `status: "in_progress"`, `value: 42` | An in-flight metric can carry an authoritative-looking number |
| `value: -99999` seconds | Negative durations pass |

`in_progress` additionally has no expiry, no `due_at`, and no aging rule. The shipped record uses it
for `MET-FLOW-013` ("no pass or flake classification is claimed yet") — honestly, in this instance.
But `MET-FLOW-013`'s trigger is "Flaky gate rate above zero on release evidence → treat the gate as
unresolved". Leaving the classification `in_progress` indefinitely is the strictly cheaper move and
nothing ages it out. The same applies to `MET-FLOW-001` and `MET-FLOW-027`, both `in_progress` here.

#### F7 — Rate integrity: `denominator: 0`, `numerator > denominator`, value/ratio inconsistency, unit not bound to metric

Accepted (verified):

| Mutation | Result |
|---|---|
| `numerator: 1, denominator: 0, value: 1` | ACCEPT — a rate with a zero denominator |
| `numerator: 9, denominator: 1` on a `ratio` | ACCEPT |
| `MET-FLOW-012` `numerator: 0, denominator: 1` with `value: 1.0` | ACCEPT — value contradicts its own fraction |
| `MET-FLOW-012` `unit: "bytes"` | ACCEPT — a first-pass gate *rate* in bytes |
| `ratio` with `value: 500` | ACCEPT |

There is also no binding between `ratio` and `percent`. Every management trigger is stated in
percent ("below 80%", "above 50%", "exceeds 15%", "exceeds 25%") while the record stores rates as
`ratio` (`MET-FLOW-012` = `0`, `MET-FLOW-019` = `1`). A record storing `0.8` under `ratio` and a
reader applying an `80` threshold disagree by two orders of magnitude, and nothing detects it.

#### F8 — One metric ID carries incommensurable units, already in the shipped record

Three policy metrics bundle two quantities under a single ID (`MET-FLOW-016` "conflict count **and**
resolution seconds"; `MET-FLOW-017` "seconds **and** retry count"; `MET-AI-062` "bytes/tokens";
`MET-AI-066` "Ultra share **and** its marginal gain"), and `unit` is not constrained per `metric_id`.
The exemplar consequently contains:

| `metric_id` | Rows |
|---|---|
| `MET-FLOW-017` | `345.091 seconds` **and** `10 count` |
| `MET-FLOW-020` | `98 gibibytes` **and** `3074977792 bytes` |
| `MET-FLOW-021` | `91.8 gibibytes` **and** `167936 bytes` |

Any aggregator keyed on `metric_id` — which is the only key a cohort comparison has — sums
`98 + 3,074,977,792` for workspace bytes and `345.091 + 10` for provider-failure seconds. Nothing in
the contract prevents that; the distinction lives only in the human-readable `scope` string.

#### F9 — Evidence is self-declared and unverifiable from the repository

The framework's integrity story is "Active append-only logs are bound by `prefix_bytes + sha256`".
In practice:

- All four locators use the scheme `second-agent://…`, which nothing in the repository can resolve.
- **3 of 4** prefixes are `mutable_after_capture: true`.
- No checker recomputes any digest. Contrast `validate_context_manifest`, which verifies live files,
  and `canonical_git_patch`, which re-derives review-packet patch identity.
- `prefix_bytes: 0` with an arbitrary 64-hex `sha256` is accepted; a `sha256` with a trailing
  newline is accepted; `locator: "x"` is accepted.
- The single largest cost figure in the record — `MET-FLOW-024`, **994,060 tokens** — is backed by
  `manager-goal-snapshot:2026-07-30T09:24:13Z`: an ad-hoc reference scheme with no locator, no byte
  count, and no hash. The `evidence_refs` vocabulary is an unconstrained string; six distinct
  schemes already appear (`commit:`, `measurement:`, `prefix:`, `doc:`, `event:`,
  `manager-goal-snapshot:`) with no enum.

Phase-2 `MET-AI-073` "forged evidence accepted — zero for release/security" was **not** promoted
(F-S8), so the framework has no metric for the failure mode its own evidence model is most exposed
to.

#### F10 — Commit and revision references are not verified, unlike every other packet type in the repo

Accepted (verified): `source_revision: "deadbeef"`, `source_revision: "notasha!"`,
`checkpoint_commits: ["zzzzzzz"]`. `minLength: 7` is the only constraint, and there is no hex
pattern and no `git cat-file` check.

To be fair to the shipped record: **all eight** commit references in it resolve
(`4f04188…`, `0be1039…`, `29c421fc…`, `535515ca…`, `38da8988…`, `57607cc9…`, `3a20c85e…`,
`89dfcefe…`), and both `doc:` references exist on disk. The defect is the absent check, not the
current data. But `MET-FLOW-019` (checkpoint coverage) and `KGI-DEV-003` (delivery continuity) are
discharged entirely by commit references, so an unverified commit string is a direct path to a false
continuity claim.

#### F11 — KGI status is narrative; no measurement is required to back any KGI

`kgi_status` entries require `target`, `observed`, `evidence_refs` — all free strings — and nothing
ties a KGI to a `metric_id`, a threshold, or a computed value. Accepted (verified):

| Mutation | Result |
|---|---|
| All five KGI → `achieved`, `lane_runs: []`, `measurements: []` | ACCEPT |
| `KGI-DEV-005` → `achieved`, observed "20% better", no cohort | ACCEPT |
| `KGI-DEV-002` deleted entirely | ACCEPT |
| Duplicate `KGI-DEV-001` with contradictory `status: "achieved"` | ACCEPT (`uniqueItems` dedups whole objects only) |
| `kgi_status[0].evidence_refs: []` | ACCEPT (`minItems` is set on lane/measurement/event refs, but **not** on KGI refs) |
| `kgi_id: "KGI-DEV-999"` | ACCEPT |

A record asserting complete success with no lanes, no measurements and no evidence passes the gate.
That is the definition of "silently accepts a misleading record".

#### F12 — `KGI-DEV-005` is not a statistically usable decision rule

The rule: "After the first five comparable packets establish a baseline, the next comparable cohort
reduces median trusted-integration lead time and total cost by at least 20% with non-inferior
`KGI-DEV-002` results."

Unspecified or unsound:

- **Censoring.** No rule for packets still open at cohort close. Dropping them biases the median
  downward, so a 20% improvement is manufacturable by leaving the slow packets open. The exemplar
  already has three lanes with no `ended_at`.
- **Dispersion.** No variance, interval, or minimum-effect treatment. With n=5, the median is the
  3rd order statistic; the durations already in this record span 62 s (a focused gate) to 1975 s (a
  lane). A 20% median shift on n=5 is comfortably inside noise.
- **Cohort membership.** "Comparable" is defined only by the 7-value `task_class`, with no size,
  risk, or write-set normalization — the two `architecture_security_repair` lanes here are visibly
  different jobs. Membership is chosen post-hoc by the same manager the metric judges; there is no
  pre-registration requirement.
- **Cohort size.** "the next comparable cohort" has no declared n.
- **Non-inferiority on zero-target metrics.** `KGI-DEV-002` counts escapes; five packets with zero
  escapes in both arms is uninformative, and no margin is defined.
- **Trigger granularity.** "First-pass gate rate below 80% across five comparable packets" can only
  take the values 0, .2, .4, .6, .8, 1.0 — the threshold sits exactly on a lattice point and flips
  on one packet. "Reviewer correction rate above 50% across five" flips at 3/5.

#### F13 — Cost comparability contradicts itself and has no refusal rule

The policy says "Provider token units are not added to one another unless the provider definitions
are demonstrably comparable" — and then defines `MET-AI-060` as "**Total** model tokens/cost per
trusted integration", which is exactly a cross-provider sum. The `unit` enum has one undifferentiated
`tokens` with no provider-normalization field.

For USD there is no refusal rule either: nothing says a cohort score must be withheld when a
component is `unavailable`. Today the record has `MET-FLOW-025` = **2.29561 USD** for the Claude
MR-list lane (and only for its *failed* attempt), no Fugu cost at all, and no Codex cost. A
"total cost reduced 20%" claim computed on this shape would be Claude-only while Fugu lane cost
free-rides at zero. `MET-AI-060` is correctly marked `unavailable` here — but that is discipline,
not contract.

#### F14 — `unique_artifact_count` is an anti-metric wearing the framework's own uniform

`unique_artifact_count` is the only quantitative lane-health field in the schema. It is a
self-declared integer with no definition of "unique artifact", no per-artifact evidence binding, and
no upper sanity bound (`999999` is accepted). In both Claude lanes it equals the checkpoint-commit
count exactly (3 = 3) — i.e. it is *commit count*, which the policy's own Anti-Metrics section
forbids as progress.

It is load-bearing: `MET-FLOW-010` (no-unique-artifact time), `MET-FLOW-011` (useful parallelism),
and the trigger "no unique artifact for 45 minutes" all key off it. Splitting work into more commits
raises it, resets the aging trigger, and inflates the useful-parallelism numerator, with no
counter-check.

### SHOULD

#### F-S1 — Zero negative tests for the new contract path

`bfa7f08` adds a new validation branch to `check_ai_contracts.py` and touches **0** lines of
`docs/development/tools/test_ai_tools.py`. That file has 42 tests; **0** mention `flow`,
`flow-metrics`, or `development_flow`. By contrast, the adjacent adapter commit `dd993cc` *did* ship
tests (+33 lines) for a much smaller change. The one new mechanism whose entire purpose is to make
records trustworthy is the one shipped without a test.

#### F-S2 — The existence gate is permanently satisfied; staleness and active-snapshot ambiguity are undetected

`missing-development-flow-records` fires only when `flow-metrics/*.json` is empty. That condition is
now satisfied forever by the one committed file. Nothing checks:

- that `captured_at` is recent, or that an active batch has a current record;
- that at most one record is `active` for a given `batch_id`;
- that a `final` record has `measurement_window.ended_at`.

Verified at checker level: two records with different `record_id`, **both** `status: "active"`, the
same `batch_id`, one dated `2025-01-01T00:00:00Z` with all five KGI flipped to `achieved` — **zero
findings**. Verified: `status: "final"` with no `ended_at` and every measurement `in_progress` —
ACCEPT. Verified as correctly failing closed: an empty directory, and a directory containing only a
`.jsonl` file, both produce `missing-development-flow-records`.

#### F-S3 — No versioning or supersession semantics despite a `.v1` filename convention

`status` is only `active | final`. There is no `supersedes`, `revision`, or `superseded` state; no
binding between `record_id` and the filename stem (`record_id: "totally-different-id"` is accepted);
and nothing asserts that a `final` record's numbers are unchanged from the commit that first
introduced it. The policy's immutability claim ("later appends do not invalidate an earlier
measurement record") covers the *logs*, not the record files, which are ordinary mutable JSON.

#### F-S4 — Cross-references are unresolved

Accepted (verified): a measurement whose `lane_id` does not appear in `lane_runs`;
`evidence_refs: ["prefix:no-such-evidence"]`; `["measurement:MET-FLOW-404"]`; `["trust me"]`;
`evidence_prefixes: []` while `prefix:` refs remain. In the shipped record all four `prefix:` refs
do resolve to a declared `evidence_id`, and the `event:`/`measurement:` refs are coherent — again,
the gap is the missing check.

#### F-S5 — Metric and KGI ID vocabularies are unenumerated

`^MET-[A-Z0-9-]{3,63}$` and `^KGI-[A-Z0-9-]{3,63}$` accept anything. `MET-FLOW-999` (invented),
`MET-FLOW-04` (a plausible typo for `-004`), and `KGI-DEV-999` are all accepted. A typo silently
forks a metric series, and the resulting cohort comparison compares two different populations
without noticing.

#### F-S6 — The promoted namespace duplicates the Phase-2 namespace

`MET-FLOW-001` ≡ `MET-AI-053`; `MET-FLOW-003` ≡ `MET-AI-056`; `MET-FLOW-005` ≡ `MET-AI-055`;
`MET-FLOW-007` ≡ `MET-AI-054`; `MET-FLOW-015` ≡ `MET-AI-043`; `MET-FLOW-027` ≡ `MET-AI-047`. Both
namespaces validate against the same pattern and both are referenced by the new policy, so the same
fact can be recorded twice under two IDs and double-counted, and it is ambiguous which ID a cohort
query should read. No deprecation or aliasing table exists.

#### F-S7 — The promotion violates the Phase-2 promotion rules the policy invokes

The policy states it "promotes the useful parts of the Phase 2 KPI scorecard into the operating
system". The scorecard's §25 still reads **"Status: not promoted / Required next action: P0 fixture
を選び、最低各5 caseのpilot baselineを取る"** — `bfa7f08` does not touch `exploration/`, so two
documents now disagree about whether promotion happened. Against the scorecard's §24 Promotion Rule:

| Rule | Status |
|---|---|
| 1. fixture/command reproducible in-repo | **Unmet** — no fixture, no reproduction command for any KPI |
| 2. denominator, exclusions, task class per metric | **Unmet** for `MET-FLOW-008`, `-010`, `-011`, `-014`, `-018`, `MET-AI-040/041/042` |
| 3. negative/inconclusive results preserved | Met |
| 4. provider/runtime version recorded | **Unmet** — schema has `provider`+`model`, no runtime/CLI version |
| 5. thresholds derived from baseline and risk tolerance | **Unmet** — see below |
| 6. owner, cadence, failure action | Met (development manager; five cadences; trigger table) |
| 7. deletion condition if the metric becomes vanity | **Absent entirely** |

On rule 5, the scorecard §20 explicitly says "latency、Fire precision、retentionはbaselineなしに
固定しない". The new policy nevertheless fixes 20/45/60/10-minute and 80/50/15/25-percent thresholds
with, by its own admission, no cohort. The thresholds are stated as reviewable ("Changing a
threshold requires a dated decision with its cohort") but they are asserted as current operating
policy, which is the thing §20 forbids.

#### F-S8 — Three safety metrics were dropped in promotion

Not promoted from Phase 2: `MET-AI-073` (forged evidence accepted — zero for release/security),
`MET-AI-074` (secret/redaction leak — zero), `MET-AI-076` (schema security downgrade — zero). Also
dropped: `MET-AI-063` (prompt cache hit), on which the `MET-AI-060`/`-064` cost claims materially
depend. The three security-guardrail drops are the most consequential given F9 and F-C5.

#### F-S9 — Zero-loss and Codex-independence claims are narrower than the KGI they discharge

`MET-FLOW-026` = `0 count`, scope "Source/worktree changes lost after exact third-restart Git
reconstruction", evidence `commit:4f04188`. But the record's own `recovery` event states "**Twelve
zero-length loose objects were quarantined**". No `git fsck` output digest, no reachable-vs-
unreachable object count, and no enumeration of what was quarantined is bound to the record — so the
zero claim is not falsifiable from the record. `KGI-DEV-003` covers "Zero lost **user**/source work",
and `MET-AI-070` (lost user work) is not recorded at all; uncommitted user edits are outside the
scope actually measured.

`KGI-DEV-004`'s guardrail is "silent model downgrade is forbidden", but `MET-AI-075` (silent
model/tier downgrade, target zero) is **not recorded**, and the schema has no runtime/CLI-version
field. The adapter drift addressed by the out-of-scope commit `dd993cc` — CLI `0.144` vs `0.146`
selecting different registered adapters for the same nominal `ultra` tier — is precisely the shape
of event `MET-AI-075` exists to catch, and a flow record structurally cannot express it.

#### F-S10 — `MET-FLOW-019`'s denominator is chosen by the party the metric judges

Policy definition: "Mutated lanes with a durable checkpoint / **mutated lanes**", where required
event #3 is "first **source/report** mutation". The record's scope narrows this to "Active lanes with
product/test mutations at capture", yielding `2/2 = 1.0`.

Under the policy's own definition the fugu storage-review lane is a candidate member of the
denominator: its `outcome` says four defects "had been identified in the live log; the formal
committed report was still pending at capture" — a report in progress with no durable checkpoint.
Counted, coverage is `2/3 = 0.67`, below any reasonable continuity bar. The metric that discharges
`KGI-DEV-003`'s checkpoint guardrail is the one whose denominator the recorder selects in free text,
with no rule and no check.

#### F-S11 — Equating a mutation with a trusted signal disarms the WIP-age trigger

Lane `…mr-list…` records `first_trusted_signal_at` == `first_mutation_at` == `2026-07-30T08:54:40.271Z`.
The policy's trusted-signal list is "a reproduced counterexample, a bounded design decision, a
durable checkpoint, a validation result, or an independent finding" — an *uncommitted* edit is not
on it. `MET-FLOW-009` (WIP age) and the 20-minute trigger key off the last trusted signal, so if any
file write counts, the aging clock can be reset indefinitely without producing anything.

The converse omission is present too: lane `…mr-mutation…` has **no** `first_trusted_signal_at` and
**no** `first_mutation_at`, while carrying three checkpoint commits that prove mutation occurred.
Required events 2 and 3 are simply missing for that lane, and the schema does not require
`first_mutation_at` when `checkpoint_commits` is non-empty.

#### F-S12 — First-pass gate rate is resettable

`MET-FLOW-012` counts "required gates passing on their **first exact-candidate run**". There is no
candidate-lineage field — no `supersedes_candidate`, no predecessor revision. Amending or
re-committing after a failure produces a new exact candidate whose first run is green, yielding
`1/1`. The shipped record honestly logs `0/1` after a real first-run failure; nothing in the
contract makes that the only possible behaviour, and the 80% trigger gates a parallelism decision.

#### F-S13 — Review and rework metrics have no counterweight

`MET-FLOW-015` (reviewer correction rate, trigger at >50%) penalizes reviewers for finding material
corrections. `MET-FLOW-014` counts repair seconds only "after a review or **gate rejection**",
rewarding informal fix loops that never reach `request_changes`. The intended counterweights —
`MET-AI-040`, `-041`, `-042` — are lagging indicators and are three of the 24 metrics with no
recorded value (§3). Until they are populated, the only measurable review metrics point toward
softer review.

#### F-S14 — Precision hygiene: false precision on estimated values, and `time_basis` is unchecked

`MET-FLOW-010` = `1974.668417` seconds with `status: "estimated"` and the note "End time is rounded
to the manager-observed minute" — i.e. a ±30 s endpoint reported to the microsecond. There is no
`precision`, `uncertainty_seconds`, or significant-figures field. `measurement_window.time_basis`
is a free enum with no consistency requirement: setting it to `"exact"` while the record's own
`limitations` say operator-observed minute precision is accepted.

### COULD

#### F-C1 — `model` is used as a free-text instrument label

The record carries `provider: "human", model: "operator-observed"`; `provider: "system",
model: "git-and-filesystem"`; `provider: "system", model: "linux-proc-du"`. These are instruments,
not models. Any per-model cost, quality, or tier aggregation keyed on `(provider, model)` produces
nonsense buckets. A separate `instrument`/`source` field, and a `runtime_version` field (F-S9),
would fix both.

#### F-C2 — A lane claims findings with no artifact, no event, and only a mutable prefix

Lane `cfr168-rust-storage-independent-review-fugu-20260730`: `unique_artifact_count: 0`, zero events,
one `mutable_after_capture: true` prefix — and an `outcome` asserting "Four candidate-owned
contract/race defects had been identified". The strongest claim in the record has the weakest binding.

#### F-C3 — Directory hygiene

`flow-metrics/` ships with no `README.md`, unlike `schemas/` and `evals/`. The glob is
non-recursive, so a record in a subdirectory is silently ignored; and once one valid `.json` exists,
a malformed `.jsonl`/`.yaml` sibling is silently ignored rather than flagged.

#### F-C4 — Ordering fragility in the checker

The flow-record block (`check_ai_contracts.py:143`) sits after `if matrix is None: return findings`
(line 130), so an unreadable `role-routing-matrix.json` skips flow validation entirely. Not
exploitable today — the missing matrix raises its own error, so the run still fails — but the new
gate should not be reachable only through an unrelated file's success.

#### F-C5 — Privacy and sensitive-cost exposure

Per-lane provider USD (`2.29561`), internal infrastructure locators (`second-agent://claude/…`), and
transcript sizes (90,461,986 bytes for one lane) are committed to the repository. The framework
further mandates binding full agent event logs as evidence with no redaction rule, while Phase-2's
`MET-AI-074` (secret/redaction leak) was not promoted (F-S8). Low impact as committed; the policy
should state a redaction rule before this scales to many batches.

#### F-C6 — Throughput itself was not promoted

Phase-2's north star is `MET-AI-001` "Trusted Integration Throughput". It has no `MET-FLOW`
equivalent. The new policy closes with "The north-star result remains trusted integration, not
activity" while providing lead-time and cost metrics only. A throughput question — the first one
named in this review's charter — currently has no metric ID to read.

#### F-C7 — The honesty mechanisms are opt-in

`limitations` is required and non-empty, which is good; but its content is unconstrained, so
`limitations: ["none"]` is accepted. `batch_id: " "` is accepted (`minLength: 1`). The exemplar's six
limitations are genuinely candid — that is author discipline, not contract.

---

## 3. KGI / KPI coverage table

KGI: 5 defined, 5 present in the record (100%), 0 backed by any measurement reference (F11).

| KGI | Outcome | Recorded status | Backing | Assessment |
|---|---|---|---|---|
| `KGI-DEV-001` | Trusted frozen-batch completion | `in_progress` | 1 doc + 1 commit | Honest. Lifecycle stages it names (`Review`, `Integrate`) are undefined in the canonical lifecycle (F3) |
| `KGI-DEV-002` | Quality integrity | `at_risk` | 2 mutable prefixes | **No measurement.** `MET-AI-040/041/042` and `MET-FLOW-027` (value) are all unrecorded — the guardrail metrics are absent, not zero (F11, F-S13) |
| `KGI-DEV-003` | Delivery continuity | `at_risk` | 1 doc, 1 commit, `MET-FLOW-019` | Only KGI with a measurement ref. That measurement's denominator is self-selected (F-S10); `MET-AI-070/071/072` unrecorded (F-S9) |
| `KGI-DEV-004` | Codex-quota-independent operation | `in_progress` | 2 prefixes, `MET-FLOW-024` | `MET-FLOW-024`'s value does not match its own definition (see below); `MET-AI-075` unrecorded; no runtime-version field (F-S9) |
| `KGI-DEV-005` | Improving cost to trusted integration | `in_progress` | 1 doc | Correctly refuses to claim a baseline. The rule it will eventually apply is unsound (F12) |

KPI: **40 defined** (27 `MET-FLOW-001…027`, 13 `MET-AI-*`), **16 recorded** (40%), **24 with no
record at all**.

| Group | Defined | Recorded | Unrecorded |
|---|---|---|---|
| Flow and speed (`MET-FLOW-001…011`) | 11 | 3 (`-001` in_progress, `-004` ×2, `-010` estimated) | `-002`, `-003`, `-005`, `-006`, `-007`, `-008`, `-009`, `-011` |
| Quality and rework (`-012…-016`, `-027`, `MET-AI-040/041/042`) | 9 | 3 (`-012`, `-013` in_progress, `-027` in_progress) | `-014`, `-015`, `-016`, `MET-AI-040`, `-041`, `-042` |
| Cost/provider/coordination (`-017`, `-024`, `-025`, `MET-AI-060…066`) | 9 | 4 (`-017` ×2, `-024`, `-025`, `MET-AI-060` unavailable) | `MET-AI-061`, `-062`, `-064`, `-065`, `-066` |
| Reliability and resources (`-018…-023`, `-026`, `MET-AI-070/071/072/075`) | 11 | 6 (`-019`…`-023`, `-026`) | `-018`, `MET-AI-070`, `-071`, `-072`, `-075` |

Two structural observations from this table:

1. **Every metric needed for a bottleneck decision is unrecorded.** `MET-FLOW-002` (stage lead
   time), `-005` (queue), `-006` (review), `-007` (integration), `-008` (flow efficiency), `-011`
   (useful parallelism) — the six metrics that would localize a bottleneck — have no value, and
   three of them are underivable from the event vocabulary (F3). `-008`, `-010`, `-011`, `-014` all
   require "evidence-producing active seconds", for which no field exists anywhere in the schema.
2. **Every zero-target safety metric is unrecorded, and absence is indistinguishable from zero.**
   `MET-AI-041`, `-042`, `-070`, `-071`, `-072`, `-075` have no rows. The policy insists that unknown
   *cost* is `unavailable`, never zero — it imposes no such rule on unknown *safety*, which is the
   more dangerous direction.

Definition/value mismatches among the 16 recorded:

- `MET-FLOW-024` is defined as "Codex coordinator token delta **per measurement window and per
  trusted package**". The recorded value 994,060 is a *cumulative thread total* — neither a
  window delta (the thread predates the `08:21:00Z` window) nor a per-package figure (no package
  exists). The `note` says so; the `status` is nonetheless `measured`, and a cohort reader comparing
  `MET-FLOW-024` across records would compare a cumulative total against a delta.
- `MET-FLOW-004`: understated by 27.5% (F5).
- `MET-FLOW-019`: denominator self-selected (F-S10).
- `MET-FLOW-020/021`: dual units under one ID (F8).

---

## 4. Baseline and evidence audit

### 4.1 Commit and document references — all resolve

| Reference | `git cat-file -t` / filesystem |
|---|---|
| `commit:0be103950120a36dabd258301fb090b89cc9ba5d` (also `source_revision`) | commit — and is exactly `bfa7f08^`, correct |
| `commit:4f0418848b9b6551c9a2202a2465da91a63870c6` | commit |
| `commit:29c421fc…`, `535515ca…`, `38da8988…` (MR-list checkpoints) | commit ×3 |
| `commit:57607cc9…`, `3a20c85e…`, `89dfcefe…` (MR-mutation checkpoints) | commit ×3 |
| `doc:docs/development/ai/swarm-batches/cfr168-recovery-execution-ledger-20260730.md` | present |
| `doc:docs/development/ai/development-flow-kgi-kpi.md` | present |

Internal cross-references are also coherent as committed: all four `prefix:` refs resolve to a
declared `evidence_id`; `event:resource-sample-20260730-092413` and the six `measurement:` refs are
consistent. None of this is *checked* (F-S4, F10) — it was verified by hand here.

### 4.2 Evidence prefixes — unverifiable by construction

| `evidence_id` | `prefix_bytes` | `mutable_after_capture` | Verifiable from repo? |
|---|---|---|---|
| `mr-list-claude-20260730` | 35,627,395 | **true** | No |
| `mr-mutation-claude-20260730` | 90,461,986 | **true** | No |
| `mr-list-fugu-hedge-20260730` | 315,528 | false | No |
| `storage-fugu-review-20260730` | 2,858,205 | **true** | No |
| `manager-goal-snapshot:2026-07-30T09:24:13Z` | — | — | No locator, no bytes, no hash |

### 4.3 Arithmetic — recomputed independently

| Claim | Independent recomputation | Verdict |
|---|---|---|
| `MET-FLOW-004` mutation lane `1317.369787` s (`08:39:24.630213Z`→`09:01:22Z`) | `1317.369787` | Exact |
| `MET-FLOW-004` MR-list lane `956.367528` s (`08:45:14.632472Z`→`09:01:11Z`) | `956.367528` | Exact **as stated**, but the stated window is the wrong start point — see F5 |
| MR-list true lane start (`08:44:56.252Z` − `345.091` s) | `08:39:11.161Z` → checkpoint at `09:01:11Z` = **1319.839 s** | Recorded value understates by 363.472 s (27.5%) |
| `MET-FLOW-010` hedge `1974.668417` s (`08:48:05.331583Z`→`09:21:00Z`) | `1974.668417` | Exact, but ±30 s endpoint reported to the microsecond (F-S14) |
| `MET-FLOW-022` MemAvailable `16,816,082,944` bytes | 15.66 GiB, above the 6 GiB floor | Consistent |
| `MET-FLOW-020` point sample `3,074,977,792` bytes | 2.86 GiB, below the 20 GiB ceiling | Consistent |
| `MET-FLOW-023` swap `0` bytes | below the 2 GiB ceiling | Consistent |
| `MET-FLOW-019` `value: 1`, `2/2` | Arithmetically consistent; denominator selection is the issue (F-S10) |
| `MET-FLOW-012` `value: 0`, `0/1` | Consistent; "32 passes and one failure" correctly scored as a failed first pass |
| Event ordering | `provider_error` at `08:44:56.252Z` precedes its lane's `started_at` `08:45:14.632472Z` by **18.380 s** | Ordering violation (F4, F5) |

### 4.4 Baseline status

`KGI-DEV-005` has **no baseline**, correctly and explicitly. But the framework simultaneously ships
hard thresholds (20 min, 45 min, 60 min, 10 min; 80%, 50%, 15%, 25%) with no baseline, which is what
Phase-2 §20 forbids (F-S7). The record's final limitation — "No five-packet comparable cohort or
trusted current-wave integration existed at capture, so throughput, flow efficiency, useful
parallelism, and the 20 percent KGI target remain unscored" — is accurate and is the single best
line in the artifact.

---

## 5. Schema / checker mutation matrix

### 5.1 Semantic mutations — 55 crafted, 55 accepted, 0 rejected

Method: deep-copy the shipped record, apply one mutation, call
`ai_contracts.validate_json_schema(record, development-flow-record.v1.schema.json)`. Fixtures in
`/tmp/flowaudit/`; nothing written to the repository.

| # | Mutation | Result | Finding |
|---|---|---|---|
| M00 | baseline, unmutated | ACCEPT (control) | — |
| M01 | `captured_at: "xxxxxxxxxxxxxxxxxxxx"` | **ACCEPT** | F4 |
| M02 | `captured_at: "2026-13-45T99:99:99Z!!!"` | **ACCEPT** | F4 |
| M03 | lane `ended_at` 1h38m before `started_at` | **ACCEPT** | F4 |
| M04 | measurement `window_ended_at` before `window_started_at` | **ACCEPT** | F4 |
| M05 | `measurement_window.started_at` after `captured_at` | **ACCEPT** | F4 |
| M06 | event `at` in 1999 | **ACCEPT** | F4 |
| M07 | `first_checkpoint_at` before lane `started_at` | **ACCEPT** | F4 |
| M08 | `sha256` = 64 hex + trailing `\n` | **ACCEPT** | F4, F9 |
| M09 | `record_id` + trailing `\n` | **ACCEPT** | F4 |
| M10 | `status: "measured"`, `value` removed | **ACCEPT** | F6 |
| M11 | `status: "unavailable"`, `value: 0` | **ACCEPT** | F6 |
| M12 | `status: "in_progress"`, `value: 42` | **ACCEPT** | F6 |
| M13 | `value: -99999` seconds | **ACCEPT** | F6 |
| M14 | `unit: "ratio"`, `value: 500` | **ACCEPT** | F7 |
| M15 | `numerator: 1`, `denominator: 0` | **ACCEPT** | F7 |
| M16 | `0/1` with `value: 1.0` | **ACCEPT** | F7 |
| M17 | `numerator: 9`, `denominator: 1` | **ACCEPT** | F7 |
| M18 | first-pass gate rate with `unit: "bytes"` | **ACCEPT** | F7, F8 |
| M19 | `metric_id: "MET-FLOW-999"` | **ACCEPT** | F-S5 |
| M20 | `metric_id: "MET-FLOW-04"` (typo) | **ACCEPT** | F-S5 |
| M21 | `kgi_id: "KGI-DEV-999"` | **ACCEPT** | F-S5 |
| M22 | duplicate `KGI-DEV-001`, contradictory `status` | **ACCEPT** | F11 |
| M23 | duplicate `lane_id` (two lanes, same id) | **ACCEPT** | F-S4 |
| M24 | duplicate `event_id`, different content | **ACCEPT** | F-S4 |
| M25 | duplicate `evidence_id`, different `sha256` | **ACCEPT** | F9 |
| M26 | measurement `lane_id` absent from `lane_runs` | **ACCEPT** | F-S4 |
| M27 | `evidence_refs: ["prefix:no-such-evidence"]` | **ACCEPT** | F-S4 |
| M28 | `evidence_refs: ["trust me"]` | **ACCEPT** | F9, F-S4 |
| M29 | `evidence_refs: ["measurement:MET-FLOW-404"]` | **ACCEPT** | F-S4 |
| M30 | `source_revision: "deadbeef"` (not in repo) | **ACCEPT** | F10 |
| M31 | `source_revision: "notasha!"` | **ACCEPT** | F10 |
| M32 | `checkpoint_commits: ["zzzzzzz"]` | **ACCEPT** | F10 |
| M33 | `prefix_bytes: 0` with unchanged `sha256` | **ACCEPT** | F9 |
| M34 | `locator: "x"` | **ACCEPT** | F9 |
| M35 | `lane_runs: []` + all KGI `achieved` | **ACCEPT** | F11 |
| M36 | `measurements: []` | **ACCEPT** | F11 |
| M37 | `events: []` | **ACCEPT** | F2 |
| M38 | `evidence_prefixes: []` while `prefix:` refs remain | **ACCEPT** | F-S4 |
| M39 | `status: "final"`, no `ended_at`, all metrics `in_progress` | **ACCEPT** | F-S2 |
| M40 | `status: "checkpointed"`, `checkpoint_commits: []` | **ACCEPT** | F6 |
| M41 | lane `status: "accepted"`, 0 artifacts, no `ended_at` | **ACCEPT** | F6 |
| M42 | `unique_artifact_count: 999999` | **ACCEPT** | F14 |
| M43 | all KGI `achieved`, `observed: "done"` | **ACCEPT** | F11 |
| M44 | `kgi_status[0].evidence_refs: []` | **ACCEPT** | F11 |
| M45 | `KGI-DEV-005: achieved`, "20% better", no cohort | **ACCEPT** | F11, F12 |
| M46 | `KGI-DEV-002` deleted | **ACCEPT** | F11 |
| M47 | only one KGI entry total | **ACCEPT** | F11 |
| M48 | mutated lane with no `first_mutation_at` (**as shipped**) | **ACCEPT** | F-S11 |
| M49 | duplicated identical measurement (double count) | **ACCEPT** | F-S4 |
| M50 | `provider` swapped `claude`→`fugu` on a Claude lane | **ACCEPT** | F-C1 |
| M51 | `model: "gpt-3.5"` on a `claude-opus-5` lane | **ACCEPT** | F-S9 |
| M52 | `record_id` ≠ filename stem | **ACCEPT** | F-S3 |
| M53 | `batch_id: " "` | **ACCEPT** | F-C7 |
| M54 | `time_basis: "exact"` contradicting `limitations` | **ACCEPT** | F-S14 |
| M55 | `limitations: ["none"]` | **ACCEPT** | F-C7 |

### 5.1b Control mutations — 15 crafted, 15 rejected (harness verified)

`schema` const change → `schema-const`; extra top-level property → `schema-additional-property`;
`limitations` removed → `schema-required`; 63-char `sha256` → `schema-pattern`; uppercase
`record_id` → `schema-pattern`; `kgi_status: []` → `schema-min-items`; `limitations: []` →
`schema-min-items`; `unit: "minutes"` → `schema-enum`; lane `evidence_refs: []` →
`schema-min-items`; event `kind: "note"` → `schema-enum`; measurement `status: "partial"` →
`schema-enum`; 19-char `captured_at` → `schema-min-length`; `unique_artifact_count: -1` →
`schema-minimum`; exact-duplicate `kgi_status` entry → `schema-unique-items`;
`task_class: "refactor"` → `schema-enum`.

The structural layer is genuinely solid. That is what makes §5.1 a statement about the design and
not about the tooling.

### 5.2 Checker-level tests (symlink overlay root under `/tmp`)

| # | Scenario | Result |
|---|---|---|
| T1 | Same `record_id` in two files (`.v1.json` + `.v2.json`) | **REJECT** — `duplicate-development-flow-record-id`. The one new semantic check works |
| T2 | Two records, different `record_id`, **both `active`**, same `batch_id`, one dated `2025-01-01`, all KGI `achieved` | **ACCEPT — zero findings** (F-S2) |
| T3 | Empty `flow-metrics/` | **REJECT** — `missing-development-flow-records`. Fails closed correctly |
| T4 | Only a `rec.jsonl` present (no `.json`) | **REJECT** — same code. Fails closed correctly |

### 5.3 Negative-test coverage in the repository

`docs/development/tools/test_ai_tools.py`: 42 tests, **0** referencing `flow`, `flow-metrics`, or
`development_flow`. `bfa7f08` adds no test file and modifies no test. Every row in §5.1 and §5.2 is
currently unguarded by regression tests (F-S1).

---

## 6. Decision

**`REQUEST_CHANGES`.**

The policy document is the strongest part of the change and mostly deserves to survive: the KGI
hierarchy, the anti-metrics list, the "long-running is not failure" framing, the refusal to invent a
baseline, and the honest `limitations` block are all correct instincts, and the exemplar record was
authored in good faith. But the question this review was asked — *is the framework genuinely usable
for later bottleneck/throughput/efficiency decisions, and can it silently accept misleading
records?* — has two clear answers:

- **Usable for those decisions: not yet.** All six bottleneck-localizing metrics are unrecorded,
  three of them are underivable from the event vocabulary the schema provides (F3), throughput has
  no metric ID at all (F-C6), and the efficiency decision rule is statistically unsound (F12).
- **Can it silently accept misleading records: yes.** 55/55 semantic mutations pass, including "all
  KGI achieved with zero lanes and zero measurements" (F11) and "`unavailable` with value 0" (F6);
  and the shipped exemplar already contains one 27.5% understated duration (F5), one event that
  precedes its own lane's start (F5), one metric whose value contradicts its definition
  (`MET-FLOW-024`, §3), and eight missing required event classes (F2).

---

## 7. Required zero-residual next action

**One action, with an enumerated closure list. Do not split it across batches — a partial semantic
validator recreates the current problem with a stronger claim attached to it.**

Add `validate_development_flow_record(record, location, *, root)` to
`docs/development/tools/ai_contracts.py`, call it from the flow block in `check_ai_contracts.py`
alongside the existing structural pass, and ship it together with a negative-test suite in
`test_ai_tools.py` that pins every row of §5.1, §5.2 and §5.3 as an expected `REJECT`. Repair the
policy and the exemplar in the same change so no residual remains:

1. **Temporal integrity (F4, F5, F-S14).** Enforce RFC 3339 UTC on every temporal field (as an
   explicit checker rule, since `format` would trip the fail-closed keyword guard); enforce
   `started_at ≤ first_mutation_at ≤ first_checkpoint_at ≤ candidate_ready_at ≤ ended_at`,
   `window_started_at ≤ window_ended_at`, `measurement_window.started_at ≤ every event.at ≤
   captured_at`, and `event.at ≥ its lane's started_at`. Add `original_started_at` /
   `resumed_from_at` to `lane_runs`, re-anchor `MET-FLOW-003`/`-004`/`-001` on the original start,
   and **correct the MR-list `MET-FLOW-004` row to 1319.839 s** (or split it into two rows with the
   outage explicit). Add a `precision`/`uncertainty_seconds` field and require it whenever
   `status: "estimated"`.
2. **Status/value/rate semantics (F6, F7, F8).** `measured`/`estimated` ⇒ `value` present;
   `unavailable`/`in_progress` ⇒ `value` absent; `denominator > 0`; `0 ≤ numerator ≤ denominator`;
   `value == numerator/denominator` within tolerance for `ratio`/`percent`; a `metric_id → allowed
   units` table so `MET-FLOW-017`/`-020`/`-021` rows are unit-typed; split the dual-quantity IDs
   (`MET-FLOW-016`, `-017`, `MET-AI-062`, `-066`) into one quantity per ID; give `in_progress` a
   required `due_at` and flag it when stale.
3. **Reference and evidence integrity (F9, F10, F-S4).** Enumerate the `evidence_refs` scheme
   vocabulary; resolve `prefix:` → `evidence_prefixes`, `measurement:` → a recorded `metric_id`,
   `event:` → an `event_id`, `lane_id` → `lane_runs`, `doc:` → an existing file, and `commit:`/
   `source_revision`/`checkpoint_commits` → `git cat-file -t` (matching the rigour
   `validate_review_packet` already applies). Require `prefix_bytes > 0`. Replace
   `manager-goal-snapshot:` with a hashed prefix or downgrade `MET-FLOW-024` to `estimated`. Promote
   Phase-2 `MET-AI-073`/`-074`/`-076` and state a redaction rule for bound transcripts (F-S8, F-C5).
4. **Required events, lifecycle, and completeness (F2, F3, F11, F-S10, F-S11).** Enforce one
   `lane_start` per lane and the full 11-class required-event list; add the missing event kinds
   (`packet_ready`, `validation_ready`, `gate_start`, `review_start`, `integration_queue_entry`,
   `integration_conflict`, `lifecycle_complete`) so `MET-FLOW-002`/`-005`/`-006`/`-007` become
   derivable; reconcile the 11-value `stage` enum with the 8-stage canonical lifecycle in
   `project-management.md` and publish entry/exit criteria per stage; require every `kgi_status`
   entry to reference at least one `metric_id` with a non-empty `evidence_refs`; make `kgi_id`
   unique and require the full `KGI-DEV-001…005` set; add the missing `lane_runs` fields
   (`first_mutation_at` required when `checkpoint_commits` is non-empty) and fix the exemplar's
   `…mr-mutation…` lane; publish an explicit denominator rule for `MET-FLOW-019` (report mutations
   included) and restate the exemplar value accordingly; and require an explicit
   `absent | measured_zero` distinction for the zero-target safety metrics `MET-AI-041/042/070/071/
   072/075`.
5. **Decision rule and comparability (F12, F13, F-S6, F-S7, F-C6).** Replace the bare "20% median
   over five packets" with a pre-registered cohort (declared before the cohort runs, stored in the
   record), a declared n per task class, an explicit censoring rule for open packets, a dispersion
   statement, and a refusal rule when any cost component is `unavailable`. Add a throughput metric
   to close the `MET-AI-001` gap. Publish a `MET-FLOW-*` ↔ `MET-AI-*` alias/deprecation table.
   Update `exploration/phase-2/…§25` from "not promoted", state the promotion rules that are waived
   and why (§24.1/2/4/5/7), and mark the currently baseline-free thresholds as provisional.
6. **Record lifecycle and gaming resistance (F14, F-S1, F-S2, F-S3, F-S12, F-S13, F-C1..C4, F-C7).**
   Enforce at most one `active` record per `batch_id`, `captured_at` freshness for an active batch,
   `ended_at` required when `status: "final"`, `record_id` bound to the filename stem, and a
   `supersedes` field with an immutability check on `final` records. Define "unique artifact" with
   per-artifact evidence binding so `unique_artifact_count` stops being commit count; add a
   candidate-lineage field so `MET-FLOW-012` cannot be reset by re-committing; require `MET-AI-040/
   041/042` alongside `MET-FLOW-014`/`-015` so review-softening is visible; separate `model` from an
   `instrument` field and add `runtime_version`; move the flow block above the `matrix is None`
   early return; add `flow-metrics/README.md`.

Until items 1–6 land together with their negative tests, the correct operating posture is to treat
`flow-metrics/*.json` as **manager notes with a schema**, not as decision-grade measurement, and to
make no cohort, parallelism, or routing decision from them.

---

## 8. Reviewer limitations

- The four `second-agent://` evidence prefixes were not resolvable from this worktree; no digest was
  recomputed and no claim sourced solely from a prefix (`MET-FLOW-017`, `-025`, the `KGI-DEV-002`
  and `KGI-DEV-004` observations) was independently confirmed. F5's understatement is derived only
  from figures internal to the record, so it stands without the logs.
- No Cargo, build, install, demo, or full-suite run; those gates are neither claimed nor disclaimed.
- `MET-FLOW-019`'s denominator (F-S10) turns on whether the fugu storage-review lane had an
  uncommitted report at capture. The record's own `outcome` text ("the formal committed report was
  still pending") is the sole basis; the lane worktree was not inspected.
- The mutation matrix targets the record contract. `validate_json_schema` itself, and the task/
  review/context packet validators, were read but not re-audited — they are out of scope for
  `bfa7f08`.
