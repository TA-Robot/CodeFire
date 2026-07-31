# CFR-168 Non-Review Throughput Bottleneck Audit

Status: manager operating analysis; not product acceptance evidence

Date: 2026-07-31

Scope owner: end-to-end development throughput **excluding** the review
process itself. Review recursion, reviewer independence, verdict vocabulary,
finding taxonomy, and zero-residual policy remain owned by the parallel
root-cause audit. Section 11 defines the interfaces between the two.

## 1. Exact snapshot

| Field | Value |
|---|---|
| Canonical head | `1409290b1df276936d522008fedc240a0fe085be` |
| Head committed at | `2026-07-31T07:34:28Z` |
| Audit captured at | `2026-07-31T08:27:10Z` |
| Immutable product baseline | `e2a218e015953f082245f36dbc6aee906b837e5a` (`2026-07-29T11:37:24Z`) |
| Recovery ledger SHA-256 at head | `cc60d9af5b91e01f3b1d4429fc9aa0a5d583f85660fa905c606408137795a14a` |
| Review index SHA-256 at head | `61bc3449ea3144c2bbb43a313433db5e76db0b4695301888d0bc3ff3c8238acd` |
| Open-execution audit subject | frozen at `664f907…`; superseded on three pending outputs |
| Measurement window | baseline → head: nominal `43.951 h` |
| Real elapsed in window | `≈ 39.1 h` (nominal minus the measured `≈ 4 h 50 m` wall-clock jump) |

This audit performed **read-only** repository, filesystem, and `git`
observation. No product byte, Cargo/Rust command, provider launch, network
call, or agent session was created. Its only write is this file.

### 1.1 Sources read

- `AGENTS.md`
- `docs/development/ai/swarm-batches/cfr168-recovery-execution-ledger-20260730.md` (1,759 lines, 63 commits since `2026-07-30T00:04:05Z`)
- `docs/development/ai/swarm-results/cfr168-review-disposition-index-20260730/index.md`
- `docs/development/ai/swarm-results/cfr168-open-execution-audit-20260731/audit.md`
- `docs/development/ai/development-flow-kgi-kpi.md`
- `docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json` (7 lane runs, 29 measurements, 15 events, 6 limitations)
- `docs/development/ai/acceleration-master-plan.md`
- `docs/development/ai/swarm-results/cfr168-trusted-throughput-audit-20260730/audit.md`
- `docs/development/ai/swarm-results/dev-swarm-throughput-design-20260730/design.md`
- `docs/development/ai/stage5-manual-control-runbook.md`
- `docs/development/ai/provider-routing-policy.md`, `provider-routing-matrix.json`
- six `*-candidate-ready-*/candidate.md` cards
- `docs/development/ai/swarm-results/cfr168-python-http-upload-review-closure-rereview-launch-20260731/incident.md`
- `docs/development/ai/task-packets/` (19 packets), `docs/development/issue-root-cause-map.json`
- `docs/development/code-review-issues/cfr-168.md`, `docs/development/bug-backlog.md`
- live read-only host samples (`/proc/meminfo`, `/proc/pressure/*`, `df`, `du`, `git`)

## 2. Measurement-integrity register

Every number in this audit carries one of four markers. Nothing is invented.

| Marker | Meaning |
|---|---|
| `measured` | recomputed by this audit, or recorded with exact precision in an immutable artifact |
| `derived` | arithmetic over `measured` values; formula shown |
| `estimated` | recorded as operator-observed or provider-reported, minute-precision or coarser |
| `unavailable` | no telemetry exists; explicitly **not** replaced by a guess |

Known contaminations, carried forward everywhere:

1. **Wall-clock discontinuity.** The container clock advanced by roughly
   `4 h 50 m` during active validation on 2026-07-30/31 while process-monotonic
   time stayed continuous. Concrete proof in the ledger: a 13-test unittest run
   reported `202.701 s` internally and `16,319.902 s` by `date +%s%3N`; a
   507-case broad gate reported `407.631 s` internally and
   `275 m 28.791 s` by shell `time`. **Every wall-derived interval that crosses
   that window is `unavailable`,** including the nominal `4.69 h` canonical
   commit gap between `0f1eee2` (`2026-07-30T20:55:56Z`) and `48338d1`
   (`2026-07-31T01:37:18Z`).
2. **Operator-observed instants.** The three host restarts (`2026-07-29T23:31Z`,
   `≈2026-07-30T02:58Z`, `≈2026-07-30T08:21Z`) and the Fugu hedge stop
   (`2026-07-30T09:21Z`) are minute-precision operator observations.
3. **Provider-reported durations.** Claude `duration_ms` and cost values are
   provider result events, retained rather than recomputed. Fugu cost is
   `unavailable`, so cross-provider totals stay incomplete.
4. **Mutable evidence prefixes.** Agent event logs are bound only by declared
   `prefix_bytes + sha256`; later appends are outside every hash in this audit.
5. **No typed stage event stream exists.** Provider-active, provider-wait,
   validation-queue, gate-active, manager-queue, collect, rollover, and
   integration time are not separately emitted anywhere. All stage
   decomposition below is reconstructed from ledger prose, commit timestamps,
   and the single flow-metrics record — which itself was captured at
   `2026-07-30T09:51:55Z` on source revision `dd993cc…` and covers only the
   first ~1.5 h of a ~44 h window.

## 3. What the system produced in the window

| Signal | Value | Marker |
|---|---:|---|
| Canonical commits since product baseline | 199 | measured |
| Files changed | 192 | measured |
| Lines inserted / deleted | 36,392 / 41 | measured |
| Product bytes changed (`codefire`, `crates/`, `tests/`, `Cargo.toml`, `Cargo.lock`) | **0** | measured (`git diff` is empty) |
| Trusted product integrations | **0** | measured |
| Exact candidate generations produced across 6 product surfaces | 22 | measured (§3.1) |
| Independent reports indexed at head | 21 (19 product, 2 control-plane) | measured |
| `ACCEPT` / `REQUEST_CHANGES` | 4 / 17 | measured |
| Raw indexed acceptance yield | 19.05% | derived (4 / 21) |
| Accepted **and integrated** deltas | 0 | measured |
| Open execution items (frozen audit at `664f907`) | 29 | measured |
| In-scope issues / repository-wide active issues | 3 / 9 | measured |
| Blocked task packets (no manifest) | 10 | measured |
| Ready/queued task packets | 7 | measured |
| Review reports total size | 11,411 lines | measured |

### 3.1 Candidate generations per surface (all unintegrated)

| Surface | Generations | Terminal state at head |
|---|---:|---|
| Python Upload / CFB-138 | 4 (`cb13da0`→`dece688`→`76baff8`→`467b0c2`) | `REQUEST_CHANGES` (HIGH-2, MEDIUM-4 open) |
| Python MR-list | 4 (`37f7427`→`38da898`→`4930d25`→`b164046`) | `ACCEPT`, not integrated |
| Python MR-write | 2 (`21fe751`→`6dd4527`) | rejected; successor blocked |
| Python Doctor+Branch | 2 (`01a7c86`/`fbef572`→`701469f`) | `ACCEPT` as composition input only |
| Rust Storage | 2 (`b3bc32a`→`cdcb9bd`) + WIP `f5bce30` | rejected, F1–F9 all open |
| Rust File Ancestry | 2 (`e93ebc2`→`02b0d63`) | review never launched |
| Rust File Doctor | 3 (`52a025e`→`1a04490`→`cd130e5`) | review launch `BLOCK` |
| Rust HTTP MR | 3 (`71854e2`→`73bc532`→`92fc810`) | review launch blocked |

**22 generations, 0 integrations.** This is the central non-review fact: the
system's implementation *service* capacity is not the constraint; everything
downstream of a finished candidate is.

## 4. Reconstructed timeline and coordinator-visibility gaps

Canonical commit timestamps are the only continuous durable clock the system
owns. Gaps longer than 30 minutes, `measured` from `git log`:

| From → to | Nominal gap | What the ledger says was happening |
|---|---:|---|
| 07-29 12:59:45 → 07-30 00:00:26 | 11.01 h | host restart #1 at 23:31; swarm recovery |
| 07-30 02:19:43 → 03:18:05 | 0.97 h | restart #2 (≈02:58) Git/object reconstruction |
| 07-30 03:28:06 → 08:26:26 | 4.97 h | heavy Rust lanes; ends after restart #3 (≈08:21). **Cargo targets grew to 91.8 GiB inside this unobserved window** |
| 07-30 08:44:15 → 09:32:11 | 0.80 h | post-restart relaunch |
| 07-30 11:42:54 → 18:21:53 | **6.65 h** | four external lanes running; zero durable canonical artifact |
| 07-30 20:55:56 → 07-31 01:37:18 | 4.69 h nominal | **`unavailable`** — contains the wall-clock jump |
| 07-31 03:51:05 → 07:30:10 | **3.65 h** | three long review lanes; four reports then integrated in one burst at 07:31:46 |

Derived: excluding the contaminated row, `28.05 h` of the `43.951 h` nominal
window (`63.8%`) produced no canonical durable artifact; against the
`≈39.1 h` real elapsed, `≥71.7%`. Four commits share the single second
`2026-07-31T07:31:46Z`, confirming burst integration rather than continuous
rollover.

This is not proof that the coordinator was idle — ledger entries at 10:49 and
11:24 document real coordinator work (freezing, mapping, registration) — but
it is proof that **coordinator output is not observable at the cadence its own
policy requires** (`development-flow-kgi-kpi.md` triggers at 20 min without a
trusted signal and 45 min without a unique artifact).

## 5. Stage-time decomposition

`measured` unless marked. Blank cells are genuine telemetry gaps.

### 5.1 Service time (implementation)

| Lane | Service duration | Source |
|---|---:|---|
| MR-mutation zero-residual repair (`→6dd4527`) | 4,019.063 s (1 h 07 m) | provider `duration_ms` |
| MR-list zero-residual repair, resumed (`→38da898`) | 3,164.042 s (52.7 m) | provider `duration_ms` |
| Rust HTTP MR preflight closure (`→73bc532`) | 6,808.307 s (1 h 53 m) | candidate card, 81 turns |
| Upload review-closure repair (`→467b0c2`) | ≈ 1 h 05 m (02:42 → 03:47) | ledger instants, `estimated` |
| MR-list final-publication closure (`→b164046`) | ≈ 1 h 09 m (02:17 → 03:26) | ledger instants, `estimated` |
| Time to first durable checkpoint | 1,317.370 s and 956.368 s | `MET-FLOW-004` |

Median observed implementation service time is on the order of **1–2 h per
candidate generation**. Against a 22-generation, 0-integration record over
~39 h real elapsed, implementation service is roughly **10–20% of lane wall
time** and is *not* the throughput constraint.

### 5.2 Queue time (non-review causes only)

| Waiting item | Queue cause (non-review) | Measured wait |
|---|---|---:|
| Rust File Ancestry `02b0d63` review | Rust build provenance / source-keyed Cargo guard | **≥ 13 h 04 m** (candidate 07-30 18:27 → still unlaunched at head) |
| Rust Storage `cdcb9bd` review | single-Fugu queue + heavy permit | 7 h 25 m (packet 07-30 18:30 → launch 07-31 01:55) |
| Rust HTTP `92fc810` review | source-keyed profile evidence | ≥ 4 h 28 m (card `a5c69f1` 03:06 → head) |
| Rust File Doctor `cd130e5` review | vacuous/stale Cargo artifact; `BLOCK` | ≥ 4 h 21 m (card 03:12 → head) |
| Rust Storage R1–R14 repair resume | source-keyed guard + machine binding | ≥ 4 h 24 m (packet 03:10 → head) |
| Python MR-write repair launch | dependency: accepted Upload + MR-list + shared-primitives `READY` | **≥ 20 h 10 m** (packet 07-30 11:24 → head) |
| MR-list continuation resume | memory admission floor (projected 4,081,827,840 B < 6 GiB) | 7 m (02:10 → 02:17) |
| Candidate-ready → reviewer start (best cases) | none | 84.7 s; 105.6 s; "under five minutes" |

The contrast is decisive: when nothing blocks it, candidate-to-reviewer
handoff is **85–300 seconds**. When a *non-review* control blocks it, the same
transition costs **4–20 hours**. Handoff mechanics are already good; the
blockers are build provenance and dependency ordering.

### 5.3 Validation / gate time

Measured broad and full gate executions, **all on heads that are not
integrated and never will be integrated as-is**:

| Gate | Duration | Subject |
|---|---:|---|
| Python-pattern 463/463 | 296.500 s | probe `701469f` |
| Full suite 691/691 | 2,550.503 s | probe `701469f` |
| Python-pattern 481/481 | 432.026 s | `37f7427` |
| Python 507/507 | 380.500 s | `4930d25` candidate |
| Python 507/507 | 407.631 s | `4930d25` reviewer (second run of same scope) |
| Python-pattern 450/450 | 333.387 s | `467b0c2` |
| Python-pattern 512/512 | 463.329 s (monotonic 464.940) | `b164046` |
| Full CLI 228 cases | 1,347.107 s | `b164046` |
| Affected legacy CLI 115 cases | 854.600 s | `4930d25` |
| Selected legacy 13 cases | 169.748 s + 202.701 s | `467b0c2` candidate + reviewer |
| Affected CLI 16 cases | 31.276 s | `b164046` |
| Rust release `live_file_doctor` 16/775 filtered | 550.765 s | `cd130e5` (partial scope, unusable) |
| Rust debug `live_file_doctor` **0 selected / 794 filtered** | 0.267 s | `cd130e5` (vacuous, fail-closed) |

Derived total of gates with recorded durations: **8,020.3 s ≈ 2 h 14 m**. This
is a **lower bound**: at least ten further broad runs (`431/431`, `433/433`,
`434/434`, the 667-case run, and Rust full-binary runs at 775/776/778/779/787/
788/792 cases) have **no recorded duration** — `unavailable`.

Focused-gate repetition on the MR-list surface alone:

- `38da898`: 33-case module 62.057 s first run with one failure, then **12
  module reruns + 14 targeted stress runs** chasing a failure that never
  reproduced. Derived ≈ 806.7 s for the 13 module runs; the 14 stress runs are
  `unavailable`. The failing test's **name was lost to output truncation**, so
  `MET-FLOW-013` (flaky-gate rate) is permanently unscoreable for this event.
- `4930d25`: **10 consecutive** 44-case runs at 92.7–118.5 s = 927–1,185 s.
- `b164046`: final 49-case 127.089 s plus three stability runs
  (116.988 / 126.437 / 156.707 s) = 527.221 s.

Derived: ≈ **2,390 s (≈40 min)** of focused re-execution on one of six
surfaces, driven by a diagnostic gap rather than by a code change.

### 5.4 Rework time

`unavailable` as a duration. The system does not separate repair-and-retest
seconds from first-attempt seconds (`MET-FLOW-014` has no value in the
record). What *is* measured: 18 of 22 candidate generations exist only because
an earlier generation was rejected, and 17 of 21 reports are
`REQUEST_CHANGES`. Rework dominates by count; its cost in seconds is not
instrumented.

## 6. Non-review bottleneck register

Fifteen bottlenecks, ranked in §8. Each carries causal proof, affected lanes,
current mitigation, remaining gap, expected impact, and cost/risk.

---

### B01 — Rust build provenance and target-cache identity

**Causal proof.** At 2026-07-31 03:06 a manager revalidation of exact File
Doctor candidate `cd130e544af0476162be153db58a0a840b75b134` ran
`cargo test -p codefire-cli --bin codefire-rs --locked live_file_doctor`
against the shared `CARGO_TARGET_DIR` and returned **exit 0 in 0.267 s with 0
selected and 794 filtered tests**. The reused binary
`/workspace/.cache/cargo-target-shared/debug/deps/codefire_rs-2521b6556af2330d`
(mtime `2026-07-31T02:46:55.434223650Z`) belonged to a **different Rust HTTP
worktree** and did not contain the candidate's own
`live_file_doctor_uses_one_local_ledger_across_scanner_and_validation`
registration, although that test exists in the candidate source
(`doctor.rs` SHA-256 `3ebe81c9…`). A release rerun compiled correctly but took
**550.765 s** and still covered only 16 of the required NOTE-1/NOTE-2/NOTE-3
cases. One global `CARGO_TARGET_DIR` therefore produced a *fast green result
for the wrong source tree*.

**Affected lanes.** All four Rust product items — File Doctor `cd130e5`, Rust
HTTP `92fc810`, File Ancestry `02b0d63`, Storage `cdcb9bd` repair — i.e. 4 of
5 Rust execution items (80% of the Rust lane), plus every future Rust review.

**Current mitigation.** (a) A correctness quarantine: all four Rust launches
are held. (b) `check_rust_execution_profile.py` (AIA-044) blocks a Rust launch
whose live processes or durable commands use alternate toolchains, private
targets, excessive jobs/threads, or incremental builds. (c) The first
source-keyed guard was **held before commit** after an adversarial audit found
ten mandatory closure classes (mutate-build-restore TOCTOU, post-gate budget
growth, cleanup races, log paths able to hit linked-worktree Git metadata,
toolchain bound by name only, fakeable test-summary text, incomplete `BLOCK`
journaling, source-mutating `fmt`/`clippy --fix` forms, indirect dispatch
bypass, non-finite timeouts).

**Remaining gap.** The guard has **no canonical candidate at the snapshot**.
Its own audit additionally rejected permanent global one-lane execution as an
end state, identified a future lock-order cycle, and showed that a full-HEAD
namespace key would rebuild several GiB for docs-only changes. So the fix as
first drafted trades a correctness bug for a serialization bottleneck.

**Expected impact.** Directly recovers the largest measured queue in the
system: ≥13 h 04 m accumulated on File Ancestry alone, ≥4 h on each of three
other items, and restores up to four concurrent Rust review/repair lanes.
Eliminates a class of *false-green* acceptance that no amount of review effort
can catch, because the reviewer would be reading a correct source tree and a
foreign binary.

**Implementation cost / risk.** Highest single item in this audit. Requires:
content/version-bound tool binaries; Cargo executed inside an available
`unshare -Urnm` namespace with the candidate root bind-remounted read-only;
evidence on a dedicated root with bounded pinned descriptors; registry usage
measured from the filesystem under one lock order; atomic inode quarantine
before descriptor-relative deletion; post-gate budget and source-drift
invalidation; artifact identity separated from candidate evidence identity;
and **resource-token concurrency keyed by source key** rather than a global
lane. Risk: a lock-order cycle or an over-broad namespace key silently
serializing all Rust work — which is why §8 requires a concurrency proof, not
just a correctness proof, before the guard is accepted.

---

### B02 — Shared-primitive duplication, ordered *after* the surfaces it de-duplicates

**Causal proof.** The same seven lifecycle families — request ledger/deadline,
sealed snapshot, final authority fence, prepared response/response owner,
retryable cleanup, publication transaction, error-origin classification — were
independently rediscovered on Upload, MR-list, MR-write, Doctor/branch, and
(in Rust form) Storage:

- Upload `76baff8`: BLOCKER-1 ledger/deadline replacement, HIGH-2 ordered
  projection ownership, HIGH-3 reusable publication root proof, MEDIUM-4
  vacuous reversals. Successor `467b0c2` closed two and **kept HIGH-2 and
  MEDIUM-4 open**.
- MR-list `4930d25`: F1 post-fence fallible teardown, F2 filesystem error
  discarded behind an older prepared `200`, F3 inert publishability guard.
- MR-write `6dd4527`: F1–F7 covering whole-request ownership, sealed-snapshot
  reuse, prepublication sealing, retryable cleanup, replay nonce.
- Storage `cdcb9bd`: publication failure recorded as `Complete`, omitted
  traversal/authority/pre-ledger owners.

The accepted MR-list report `ee6517a` then routes four *more* observations —
including "inherited POST displacement asymmetry weaker than GET" — into a
shared-primitives successor.

**The ordering defect.** The `cfr168-python-http-shared-primitives-preflight`
packet (frozen `ba8dd25`, 2026-07-30 20:01) is **blocked until exact accepted
Upload and MR-list identities are bound**. Upload is still `REQUEST_CHANGES`.
So the one control designed to stop this recurrence is scheduled *behind* the
surface whose recurrence it exists to prevent. MR-write has been blocked on it
for ≥20 h 10 m.

**Affected lanes.** Python Upload, MR-list, MR-write, Doctor+Branch, the
Python HTTP composition packet `53cf720`, and — through the same invariant
families in Rust — Storage and Rust HTTP.

**Current mitigation.** Ledger control 2 ("primitive before surface", 07-30
19:10) states the rule; the trusted-throughput audit lists it as root
bottleneck 3; AIA-035 defers extraction to post-composition so move-only work
cannot invalidate active reviews.

**Remaining gap.** The rule exists; the *schedule* contradicts it. The
preflight is report-only, writes no product byte, and needs a *specification*
of the seven primitives — which the four immutable rejecting reports already
supply. Binding it to an *accepted* Upload SHA is stricter than its own
deliverable requires.

**Expected impact.** Each avoided recurrence saves one full generation:
1–2 h implementation service, one review cycle, one rollover, and one
candidate-card/ticket pair. Two avoided generations on MR-write plus one on
Upload is a derived **≈4–6 h of provider service time and 3 review cycles**,
and removes the ≥20 h MR-write dependency wait.

**Implementation cost / risk.** Low, documentation-and-scheduling only:
re-bind the preflight's inputs to (i) accepted MR-list `b164046`, (ii) the
four immutable Upload/MR-write/Storage reports as *specifications*, and (iii)
an explicit statement that the resulting `READY` binds primitive contracts,
not Upload's product bytes. Risk: the preflight could ossify a contract around
a still-defective Upload behavior — mitigated by requiring the preflight to
cite each finding by report SHA-256 and to mark any primitive whose
final shape depends on the open HIGH-2 residual.

---

### B03 — Manual terminalization (verdict settled, artifact absent)

**Causal proof.** Two exact, independently recorded occurrences:

- Upload Fugu reviewer: ran ≈ **3 h 15 m** and emitted repeated decisive /
  "report ready" signals for **more than 20 minutes** while its exact report
  path stayed absent and its worktree stayed clean. Downstream repair cannot
  be bound until an immutable report commit exists.
- Storage Fugu reviewer: `check_review_terminalization.py` counted **179
  agent-text events, 149 after the first ready signal, 135 after the first
  verdict signal, 7 ready signals, 5 verdict signals**, a live worker, and no
  report path → `INTERVENE`. The manager stopped only the looping process
  group and resumed the same session `019fb5e4-…` in report-only mode, which
  then committed the 54,661-byte report.

The same class appears on the implementation side: the File Doctor worker's
terminal result text says the committed-tree release gate "was still
compiling" and two background tasks were then stopped — a provider-terminal
run with **no terminal gate evidence**.

**Affected lanes.** Every long adversarial lane; measured on Upload review,
Storage review, File Doctor implementation.

**Current mitigation.** AIA-045 `check_review_terminalization.py` classifies
`WAIT` / `INTERVENE` / `COMPLETE` from agent-authored signals, later event
count, worker liveness, and exact Git report state; manual intervention
preserves the session and resumes report-only rather than starting a competing
reviewer. AIA-043 makes decision-settled → report-commit a separately measured
interval.

**Remaining gap.** Detection is manual and polled; there is no automatic
scheduler checkpoint or nudge, and **the interval itself is still not
measured** — the ledger records event *counts*, not the seconds between first
verdict signal and report commit. `MET-NRT-011` in §9 fixes this.

**Expected impact.** Measured floor of 20 minutes on one occurrence and an
unbounded loop on another; with two occurrences in ~39 h, a derived
**≥40–120 min per day** of dead critical-path time, plus the provider cost of
149 post-decision events.

**Implementation cost / risk.** Low-to-medium. The detector exists; it needs
(a) periodic invocation from the scheduler rather than the manager, (b) a
`progress.verdict_settled` event, and (c) a bounded nudge that asks the exact
session to commit its report before intervention. Risk: nudging a reviewer
mid-analysis could truncate findings — mitigated by nudging only after the
agent has itself emitted a verdict signal, which is the current
`INTERVENE` precondition.

---

### B04 — Coordinator serialization and burst integration

**Causal proof.** §4: `28.05 h` of `43.951 h` nominal (`63.8%`; `≥71.7%` of
real elapsed) with no canonical durable artifact, including a **6.65 h** block
on 07-30 while four lanes ran and a **3.65 h** block on 07-31 while three
review lanes ran. Four report integrations then share the single second
`2026-07-31T07:31:46Z`. The ledger's own rule requires a checkpoint after 45
minutes without a unique artifact.

Structurally, one Codex coordinator lane owns: freeze, conflict-domain
decisions, candidate identity recomputation, report integration, packet
registration, admission evaluation, resource sampling, cleanup, and ledger
maintenance. Every stage transition in the mandatory flow passes through it.

**Affected lanes.** All. Every rollover, every integration, every admission.

**Current mitigation.** `capture_candidate_identity.py` (AIA-038) moved
identity recomputation off the critical path — measured effect: candidate-card
capture to verified reviewer start "under five minutes", and the 84.7 s /
105.6 s handoffs in the flow record. Prebuilt blocked review tickets (07-30
20:34) moved review specification off the candidate-ready path at zero
provider cost. `check_swarm_admission.py` (AIA-041) automated admission
evaluation.

**Remaining gap.** Report integration, ledger maintenance, packet rebinding,
and cleanup remain serialized behind one lane with no emitted heartbeat. There
is no `coordinator.action` event, so idle time cannot be distinguished from
un-emitted work — which is exactly why this audit must present the gap as
*visibility*, not proven idleness.

**Expected impact.** Bounding coordinator burst latency to one 45-minute
checkpoint would convert the 6.65 h and 3.65 h blocks into ≤9 and ≤5
checkpoints respectively, each carrying a rollover opportunity. Derived upper
bound on recoverable latency: the portion of those 10.3 h during which a
completed candidate or settled verdict existed but was not rolled over —
currently `unavailable`, and the first thing §10 instruments.

**Implementation cost / risk.** Low: one append-only `coordinator.action`
event per durable manager action plus a mandatory 45-minute ledger heartbeat.
Risk: heartbeat-for-its-own-sake noise; mitigated by requiring each heartbeat
to name the artifact it produced or the exact blocker it is waiting on.

---

### B05 — Repeated broad gates and test redundancy

**Causal proof.** §5.3: at least **11 distinct broad/full runs**, ≥ 8,020.3 s
of measured gate time, **zero of them on an integrated composition head**. The
same 507-case scope ran twice (candidate 380.5 s, reviewer 407.631 s) on the
same candidate. The full 691-case suite ran once at **2,550.503 s** on a probe
that is explicitly "validation input, not an integration candidate". Ten
consecutive 44-case runs and 12+14 reruns chasing an unreproducible failure
whose name was lost to truncation.

**Affected lanes.** Every candidate; heaviest on Python (33,455-line
`codefire`, 13,602-line `tests/test_codefire_cli.py`) and Rust
(~790-case binary target requiring a full compile).

**Current mitigation.** The gate ladder (ledger §"Gate ladder and evidence
posture") already limits broad gates to stable runtime-group heads. The two
prepared Rust reviews separate non-cacheable independent falsification
(identity, live routes, focused debug/release, every deliberate mutation) from
deterministic broad gates reusable only under a manager-generated key binding
tree, argv, environment, toolchain, Cargo lock/config, fixtures, terminal
pass, and complete log digest. `run_timed_gate.py` (AIA-040) gives shell-free
argv execution, process-group timeout, monotonic duration, complete log
bytes/SHA-256, atomic JSON, and explicit wall-drift.

**Remaining gap.** The content-addressed cache is **designed, not
implemented** (AIA-031, blocked with the foundation). The ladder is prose, not
enforced: nothing prevented the reviewer's duplicate 507-case run. Truncated
test output silently destroyed a flake classification.

**Expected impact.** Under the design's cache key, the two identical 507-case
runs collapse to one (−407.6 s), the 691-case probe suite is not repeated on
composition (−2,550.5 s each avoided repeat), and the ten-run stability
sequence becomes one run plus nine cache hits (−≈950 s). Derived
**≥ 3,900 s (≈1 h 05 m) per equivalent cohort**, before counting the Rust
recompiles in B01.

**Implementation cost / risk.** Medium; depends on B01 (a cache key is
worthless if artifacts can come from another worktree) and on the foundation
(B15). Risk: caching a gate that is not actually deterministic. Mitigated by
the design's explicit non-cacheable classes — failed, flaky, interrupted,
time-sensitive, install, dogfood, and final release gates are never promoted
from cache, and a packet must *opt in* per gate.

---

### B06 — Worktree, disk, and memory pressure

**Causal proof.** Measured incident chain: 49 per-worktree Cargo `target`
directories reached **91.8 GiB**; `/workspace` reached **≈98 GiB**;
`/workspace/worktrees` 67 GiB and `/workspace/project` 28 GiB; the host
restarted (#3, ≈08:21) after operator-observed memory pressure and Windows
C-drive exhaustion. Recovery cost: 12 zero-length loose Git objects quarantined
and five exact objects reconstructed (after 19 objects in the #2 incident),
two zero-length working copies restored, `fstrim`, and a ~10-minute
reconstruction window (08:21 → 08:31:20). Post-cleanup `/workspace` measured
2.678 GiB.

Later single-lane targets reclaimed after termination: 3,204,719,352 B
(ancestry, 19:06), 2,327,617,292 B (File Doctor, 19:21), 1,590,727,568 B
(reviewer, 19:48) — **≈7.12 GB in 42 minutes**, each representing one full
from-scratch compile that will be paid again. A duplicate toolchain home at
`/tmp/codefire-rustup` held ≈727 MiB.

Current live state (`measured`, 2026-07-31T08:27:10Z): `/workspace/worktrees`
**3,594,457,783 B** across **103** directories; `/workspace/project`
397,915,001 B; shared Cargo target **1,055,172,631 B**; 53 durable `dev-swarm`
state files; `MemAvailable` 10,350,788 kB; swap used ≈2,060 kB; workspace
filesystem 841 GiB free of 1,007 GiB; CPU some PSI avg10 0.00, memory/I-O full
PSI avg10 0.00; 14 CPUs.

Derived growth since the 2026-07-30 19:55 trusted-throughput snapshot
(97 dirs, 3,037,378,168 B, 49 state files): **+6 worktrees, +557,079,615 B
(+18.3%), +4 state files in 11.65 h** — i.e. ≈47.8 MB/h of retained state
against zero delivered product.

**Affected lanes.** All Rust lanes directly; all lanes indirectly through the
three restarts, each of which invalidated every live PID.

**Current mitigation.** Shared target with `CARGO_INCREMENTAL=0`,
`CARGO_BUILD_JOBS=2`, `RUST_TEST_THREADS=2`; measured floors (no launch below
6 GiB projected `MemAvailable`, above 2 GiB swap, above 20 GiB `/workspace`,
below 100 GiB free); terminal-cache rule reclaiming only inactive reproducible
targets while never touching worktrees, `.git`, packets, reports, state, or
logs; PSI/PID checks in `check_swarm_admission.py`.

**Remaining gap.** No worktree garbage collection: 103 directories persist,
including terminal, superseded, and rejected-candidate lanes. Cleanup is
manual and event-triggered by a manager observation, not by lane termination.
Windows C-drive free space remains structurally `unavailable` from the
container, so the actual failure signal for the worst incident cannot be
monitored at all.

**Expected impact.** Preventing one restart preserves the ~10-minute Git
reconstruction plus the loss of every live lane's process state — in the #3
incident that meant re-establishing four lanes. Automatic terminal-target
reclamation at lane exit prevents the 91.8 GiB accumulation pattern by
construction.

**Implementation cost / risk.** Low-to-medium: reclaim the private target at
task exit inside `dev-swarm` (already specified in the design's required
tests: "terminal cleanup reclaims an inactive private target but never an
active or shared target"), plus a retention policy for terminal worktrees
after their evidence prefix is bound. Risk: deleting a worktree still
referenced by an unbound evidence prefix — mitigated by the existing rule that
cleanup selects only reproducible build artifacts and only after the
authoritative log/result is committed.

---

### B07 — Provider readiness, admission, and capacity seams

**Causal proof.** Four distinct measured failure modes, none of them capacity:

1. **Overload storm.** Claude Opus 5 MR-list initial turn: 10 consecutive 529
   retries, **345.091 s** of provider API time, **$2.29561** spent, zero
   artifacts; the exact session was then resumed (17.520932 s… i.e. $17.52 for
   the successful lane).
2. **Model-list incompatibility.** Four Fugu independent-review sessions ended
   before producing any report after repeated Sakana model-list response
   incompatibility. Four review lanes lost; partial logs non-authoritative.
3. **Credential inheritance.** Storage review's first scheduler launch created
   no commit and exited in **5 seconds** because the manager environment did
   not inherit `SAKANA_API_KEY`, although a live Fugu lane already had it.
   Then, on 2026-07-31 03:46, the Upload rereview launch stopped **before
   provider dispatch with return code 3** because the launch environment did
   not name `/workspace/project/fugu-api` via `FUGU_API_KEY_FILE`; zero task
   commits, clean worktree, preserved state/log digests.
4. **Admission holds.** One measured 7-minute hold (02:10 → 02:17) when a
   2 GiB MR-list reservation projected 4,081,827,840 B against the 6 GiB
   floor; at 02:02 the next light launch projected 5,715,308,544 B.

Against this, capacity itself was repeatedly proven healthy: at 01:55 the host
showed 14 CPUs with no quota, no cgroup memory ceiling, 13,540,003,840 B
available, zero swap, CPU some PSI 0.36, memory/I-O full PSI 0.00,
903,601,012,736 B free, 23,168 PID headroom — while running five external
lanes.

**Affected lanes.** Storage review, Upload rereview, MR-list implementation,
and the four lost Fugu reviews.

**Current mitigation.** `check_swarm_admission.py` replaced the fixed
four-session / two-heavy caps with per-candidate reservations, projected
memory, swap/disk/PSI/PID pressure, adaptive provider windows with backoff,
and explicit conflict ownership — no total-session input. Outstanding
reservations prevent burst launches from double-counting the same free memory.
The Upload retry exported only `FUGU_API_KEY_FILE` and never copied a secret
into a worktree or log. Incident evidence is durable
(`…rereview-launch-20260731/incident.md`, canonical `c3a6cb3`/`8597451`).

**Remaining gap.** AIA-042 is explicitly unimplemented and blocked on the
foundation: no secret-preserving provider-readiness inheritance, no typed
preflight-versus-worker failure distinction, no automatic retry of a
zero-commit clean launch, and no adoption of a manually resumed exact session.
Every one of the four failures above required manual manager action.

**Expected impact.** Each credential failure costs one manager rollover
interval and one batch/task identity. Automatic typed-preflight retry converts
those into a self-healing 5-second event. The 529 case argues for provider
backoff *before* spending 345 s and $2.30.

**Implementation cost / risk.** Medium; blocked on B15. Risk: a retry loop
that leaks credentials or masks a genuine authentication failure — mitigated
by the existing rule that readiness inheritance must never log secrets and
that provider failure is never evidence the task failed.

---

### B08 — Codex quota discontinuity (latent, not yet triggered)

**Causal proof.** No Codex quota stoppage occurred in this window: no external
Codex implementation lane was active, two Claude Opus 5 implementation lanes
and four independent reviews ran without one, and `KGI-DEV-004` is
`in_progress` rather than `at_risk`. The exposure is structural:

- routing fixes `codex: 1` as a deliberate cost-control lane, and *all* of
  manager, architect, integrator, and final_auditor roles default to Codex;
- policy records that "Codex usage credit has repeatedly become a single point
  of development stoppage" (historical);
- the only measured consumption signal is a **cumulative coordinator-thread
  token sample**: 994,060 → 1,382,958 tokens between `09:24:13Z` and
  `09:51:55Z`. Derived: **+388,898 tokens in 1,662 s = 234 tokens/s ≈ 842k
  tokens/h** of coordinator thread growth.

**Affected lanes.** Every stage transition, because §B04 shows the coordinator
is the serialization point.

**Current mitigation.** Low-Codex routing (explicit `--allow-codex`, no
automatic fallback, `claude_to_codex: forbidden`); Claude/Fugu carry all
implementation and review; portable Git/log evidence means a coordinator loss
does not lose work; `KGI-DEV-004` and the KPI trigger "one Codex quota
incident stops the batch" are already written.

**Remaining gap.** `MET-FLOW-024` records a cumulative thread total, not a
per-window delta or a per-trusted-package figure — and the KGI/KPI independent
review already flagged that "`MET-FLOW-024`'s value does not match its own
definition". `MET-AI-060` is `unavailable` because Fugu cost is unavailable and
no package reached trusted integration. There is **no measured continuity
drill**: nobody has demonstrated a coordinator handoff to a non-Codex manager
on portable evidence.

**Expected impact.** Zero today; catastrophic and unbounded on the day it
triggers, because the coordinator owns every rollover. A single measured drill
converts an untested assumption into evidence.

**Implementation cost / risk.** Low. Define `MET-NRT-013` as a per-window
delta (§9), record Fugu cost as explicitly `unavailable` rather than absent,
and run one bounded continuity rehearsal: reconstruct the current portfolio
board from ledger + index + `flow-metrics` + `git` alone, and record the
reconstruction time. Risk: none to product; the drill is read-only.

---

### B09 — Implementation service time (measured, and *not* the constraint)

**Causal proof.** §5.1: 4,019.063 s, 3,164.042 s, 6,808.307 s of provider
service produced three candidates; first durable checkpoints landed at
1,317.370 s and 956.368 s (both inside the 30-minute p95 target). Meanwhile the
MR-write lane those candidates feed has been blocked for ≥20 h 10 m and
`0` product bytes have been integrated in ~39 h real elapsed. Implementation
is roughly 10–20% of lane wall time.

**Affected lanes.** All, but as a *non*-bottleneck.

**Current mitigation.** Checkpoint rule (45 min), 12-hour envelopes,
counterexample-first packets, exact-session continuation instead of new
implementation orientation.

**Remaining gap.** One real inefficiency exists inside service time: the
Rust HTTP lane created intermediate WIP commits and then **rewrote them into a
single delivered commit despite the packet's additive/no-rewrite
instruction**, forcing the reviewer to reconcile whether product bytes or
evidence disappeared. And the MR-list reviewer "first spent time discovering
that the new identity helper was not present in the older candidate-derived
taskbase" — a packaging cost now fixed by carrying the manager-vetted helper
blob (digest `ef548dd9…`).

**Expected impact.** Explicitly negative: **adding implementation concurrency
now increases review debt and composition work.** This entry exists to prevent
the reflex of scaling the wrong stage. Enforcing no-rewrite mechanically saves
one reviewer reconciliation per occurrence.

**Implementation cost / risk.** Trivial: `capture_candidate_identity.py`
already resolves additive-versus-rewritten commit lists and emits `BLOCK` on a
merge or wrong base; extend the same check to reject a delivered range whose
reflog shows a rewrite of committed WIP when the packet declares additive-only.

---

### B10 — Evidence and documentation overhead

**Causal proof.** 36,392 insertions and 41 deletions across 192 files, **0 of
them product**, in 199 commits. The recovery ledger alone is 1,759 lines and
63 commits since `2026-07-30T00:04:05Z`. Review reports total 11,411 lines
across 21 files. On-disk: `swarm-results` 1,431,427 B, `swarm-batches`
500,672 B, `task-packets` 269,600 B.

**Rebinding churn is measurable.** Three different recovery-ledger hashes are
simultaneously bound by live artifacts:

| Artifact | Ledger SHA-256 it binds | Index SHA-256 it binds |
|---|---|---|
| `cfr168-post-composition-lifecycle-freeze-20260730` (blocked) | `af8d5dcc…` | `76de456e…` |
| `cfr168-open-execution-audit-20260731` (frozen at `664f907`) | `83620629…` | `5739e784…` |
| Actual at head `1409290` | `cc60d9af…` | `61bc3449…` |

Every one of those bindings must be recomputed and re-audited before its
packet can move from `blocked` to `ready` — the open-execution audit says so
explicitly ("The blocked lifecycle packet binds older ledger/index hashes and
must be rebound before it becomes ready").

**Affected lanes.** Coordinator (authoring), every packet that binds a hash,
every reviewer that must recompute identity.

**Current mitigation.** The compact exact-review form in
`validation-report-template.md` (AIA-039) targets report synthesis and manager
parsing time without weakening evidence. `capture_candidate_identity.py`
removes hand transcription. AIA-038 report mode verifies report-only write
sets mechanically.

**Remaining gap.** The ledger is append-only prose with no machine index, so
answering "what is blocked on what" requires reading 1,759 lines — this audit
did exactly that. Hash rebinding is manual and has no staleness detector: an
artifact binding `af8d5dcc…` is not flagged anywhere as stale.

**Expected impact.** A generated portfolio view (AIA-033) plus a
`check_bound_digests.py` staleness gate converts a full-ledger read into a
table lookup and turns silent staleness into a failing check. This is the
difference between the next audit costing hours and costing minutes.

**Implementation cost / risk.** Low: emit the portfolio board as JSON
alongside the ledger prose, and add a checker that recomputes every declared
`SHA-256` binding in `task-packets/` and `swarm-results/` and fails on drift.
Risk: none — it is additive and read-only. Note the deliberate tension: this
overhead is also the system's *safety property*; the goal is to make it
machine-derived, not to reduce evidence.

---

### B11 — Issue-state drift

**Causal proof.** All three in-scope issues (`CFR-168`, `CFB-136`, `CFB-138`)
remain `investigating` while 22 candidate generations and 21 reports exist.
`issue-root-cause-map.json` carries `updated_at: 2026-07-30` while head is
2026-07-31 with three newly terminal reviews. `CFR-168`'s `owner_module` is the
single value `crates/codefire-cli/src/metrics.rs`, while its actual execution
topology spans `codefire` (33,455 lines), `crates/codefire-cli/src/http.rs`
(18,639), `main.rs` (12,647), `remote.rs` (4,101), `storage.rs`,
`remote/doctor.rs`, `work_budget.rs`, `secure_worktree.rs`, and four canonical
source documents. The open-execution audit states this directly: "Single-module
`owner_module` values in the issue map are insufficient for the current
cross-runtime execution topology."

**Affected lanes.** Planning and dependency resolution for all 29 open
execution items; anyone using the issue map to find an owner.

**Current mitigation.** `check_project_docs.py` reconciles CFB/CFR detail
ledgers and counts; the recovery ledger and disposition index carry the real
state; the open-execution audit's `open-items.v1.json` is the current
machine-readable execution inventory.

**Remaining gap.** Two parallel state systems (issue map vs. execution
inventory) with no reconciliation check, and a schema that cannot express a
cross-runtime owner set. Status transitions are correct but *coarse*: nothing
distinguishes "investigating with 0 candidates" from "investigating with 22
candidates and 4 accepts".

**Expected impact.** Prevents a future audit from mis-scoping work and
prevents an owner-based scheduler from ever being correct. Low throughput
impact today, high impact once portfolio scheduling (AIA-033) is automated.

**Implementation cost / risk.** Low: widen `owner_module` to an ordered
`owner_modules` array (additive, with the existing key retained for
compatibility), and add a checker asserting that every in-scope issue's owner
set covers the union of write sets declared by its live packets. Risk: touching
`crates/codefire-cli/src/tests/docs.rs::issue_root_cause_map_covers_open_issues`
is a product change and is therefore **out of scope for this audit** — it must
be scheduled as a normal reviewed product delta after composition.

---

### B12 — Excessive WIP and critical-path dilution

**Causal proof.** At head: 29 open execution items, 10 blocked packets, 7 ready
packets, 103 worktrees, 53 `dev-swarm` state files, 8 predecessor-gated
successor surfaces, 3 serially chained final-evidence packets
(metrics → complexity → capability), and a 7-step critical path in which every
step needs an exact acceptance. Peak concurrency was five external lanes
(2026-07-31 02:17).

Two measured dilution events:

- The Fugu MR-list outage hedge ran **1,974.668 s with `unique_artifact_count:
  0`** and was deliberately stopped once the primary had a clean candidate;
  its slot was reclaimed for review.
- Four Fugu reviews were lost entirely to the model-list incompatibility.

Fan-in concentration: one *unbuilt* control artifact (the source-keyed Cargo
guard) gates 4 of 5 Rust items; one *unaccepted* Upload candidate gates
MR-write, the shared-primitives preflight, and the whole Python composition.

**Affected lanes.** The whole portfolio.

**Current mitigation.** "Review debt before new surface WIP" (ledger control
3); one writer per conflict domain; review-waiting candidates bounded by
reviewer permits; the hedge-stop rule ("primary candidate is clean while a
hedge has no unique artifact → stop the duplicate"); the three blocked
final-evidence packets deliberately carry **no manifest**, so they add no live
WIP or provider cost.

**Remaining gap.** No WIP-age metric is computed (`MET-FLOW-009` has no value
in the record), no critical-path scheduler exists (AIA-033 blocked), and there
is no rule bounding the *number of simultaneously blocked packets* — 10
blocked packets each carry rebinding and audit debt that must be repaid before
they can launch.

**Expected impact.** Concentrating on the two fan-in nodes (B01, B02) rather
than distributing effort across 29 items is the single highest-leverage
scheduling change available; it converts an 80%-blocked Rust lane and a
20 h-blocked Python lane into runnable work.

**Implementation cost / risk.** Low (policy) to medium (automation). Risk:
under-utilization anxiety driving new lanes; explicitly rebutted by the ledger
19:10 decision and by §B09.

---

### B13 — Install / Dogfood sequencing forces a double terminal run

**Causal proof.** The open-execution audit's critical path, steps 6 and 7:
freeze one integrated source-close SHA and run full gates, independent High
review, reproducible build, isolated Install, installed reversals, and
Algorithm/real-repository Dogfood — **then** "apply and independently accept
the final declaration-only capability mutation, **rerun the whole terminal
lifecycle**, synchronize state, and Retrospect." The lifecycle packet confirms
the loop rule: "A release-binary change returns it to Install. A
dogfood-subject or command change returns it to the affected Dogfood entry."

Cost anchor for one terminal run: the Python full suite alone measured
**2,550.503 s (42.5 min)**, the Rust binary target runs ~790 cases requiring a
full compile (one partial release compile measured 550.765 s), plus two
isolated release builds that must agree, installed reversal matrices,
canonical Algorithm Dogfood, and real-repository read-only matrices with
before/after authority hashing. A full terminal lifecycle is measured in
**hours**, and it is currently scheduled to run **twice** — the second time to
validate a mutation that is, by its own packet, *declaration-only*.

**Affected lanes.** The final lifecycle: `FINAL-SOURCE-CLOSE-PRECAP`,
`FINAL-PRECAP-BUILD-INSTALL`, `FINAL-PRECAP-REVERSAL-DOGFOOD`,
`FINAL-CAPABILITY-READVERTISEMENT`, `FINAL-POSTCAP-TERMINAL-RERUN`,
`FINAL-SYNC-RETROSPECT`.

**Current mitigation.** Isolation is correct and deliberate: the capability
change is separated so it cannot smuggle behavior, and the rerun rule prevents
a stale-evidence Close. The lifecycle packet already forbids reusing
pre-mutation lifecycle evidence.

**Remaining gap.** No *bounded rerun-scope contract* exists. Nothing
distinguishes "the capability payload digest and the declaration bytes
changed, and nothing reachable from any installed command path changed" from a
general release-binary change. The conservative default is a full rerun.

**Expected impact.** A proven bounded rerun — capability payload digest,
version/help fingerprints, installed hashes/permissions, and the exact subset
of installed commands that read the declaration — would replace hours of
duplicate Install/Dogfood with a targeted differential. Derived saving: on the
order of one full terminal lifecycle, the single most expensive stage in the
plan.

**Implementation cost / risk.** Medium, and **it is a review-policy boundary
question**: whether a bounded differential can discharge post-mutation
lifecycle evidence is an acceptance-criteria decision. This audit therefore
*proposes the measurement* and routes the *decision* to the review root-cause
audit and the manager (see §11). Risk: a bounded rerun that misses a
capability-driven behavior path — mitigated by requiring the differential to
prove, on the exact installed binary, that no command's normalized output
digest changes except the declared capability payload.

---

### B14 — Wall-clock discontinuity destroys lead-time telemetry

**Causal proof.** §2.1. The clock advanced ≈4 h 50 m during active validation;
`date +%s%3N` reported 16,319.902 s for a run whose internal duration was
202.701 s; shell `time` reported 275 m 28.791 s for a 407.631 s gate.
`timeout`-controlled processes survived because they use monotonic time. One
Rust gate additionally recorded `wall_clock_discontinuity=true` with wall
exceeding monotonic by 9.364021547 s.

**Affected lanes.** Every KPI that is a wall-clock interval — which is most of
`MET-FLOW-001` through `MET-FLOW-011`.

**Current mitigation.** `run_timed_gate.py` (AIA-040) executes one argv without
a shell, uses a new process group with monotonic timeout and duration, binds
complete log bytes/SHA-256, writes JSON atomically, and reports wall-minus-
monotonic drift with a 2-second threshold. Correctly, **no gate was rerun
merely to make its wall duration look plausible**, and affected intervals were
marked `unavailable` rather than repaired.

**Remaining gap.** Only *gates* are monotonic. Stage transitions, queue
entries, candidate-ready, verdict-settled, and rollover instants are still
recorded as wall timestamps in prose and commit metadata, so the next audit
will hit the same hole the moment another discontinuity occurs.

**Expected impact.** This is the enabling fix for every other measurement in
§9: without a monotonic, drift-annotated stage clock, first-pass acceptance
and queue-versus-service decomposition cannot be computed at all.

**Implementation cost / risk.** Low: every event carries `(wall_ts,
monotonic_ns, boot_id, drift_flag)`; intervals are computed from monotonic
within a boot epoch and marked `unavailable` across epochs. Risk: none.

---

### B15 — Absent foundation Git provenance (root enabler)

**Causal proof.** `measured` at audit time: `/workspace/.git` is an **empty
directory**; `/workspace/scripts/dev-swarm` is a 40,326-byte unversioned
executable (mtime 2026-07-30 00:05); the ledger records that no foundation
remote identity is present and GitHub CLI is unauthenticated. The manager
explicitly refuses to initialize an unrelated history or mutate unversioned
scheduler code, because that would weaken rollback and review.

**What it blocks.** AIA-028 (atomic `second-agent.run.v1` state and
adopt/resume), AIA-029 (global execution profiles and adaptive admission
*in the scheduler*), AIA-030 (review rollover and repair continuation),
AIA-031 (content-addressed validation and generated evidence cards), AIA-032
(shadow integration trains), AIA-033 (portfolio event/KGI telemetry), AIA-037
(fail-closed `dev-swarm.v2`), AIA-042 (provider-ready retry and adoption).
That is the mitigation for **B03, B04, B05, B07, and most of §10**.

**Affected lanes.** All future automation; today it forces every Stage 5
control to be a manual runbook procedure
(`stage5-manual-control-runbook.md`).

**Current mitigation.** The manual runbook implements the design's semantics by
hand: one portfolio board, manager-recomputed candidate cards, prepared
blocked review manifests, exact continuation, deterministic gate keys, shadow
composition, stage events, and resource floors. The CodeFire-side AI checker
rejects Stage 5 keys at any nesting depth in `dev-swarm.v1` manifests so that
no inert field can look enforced (48-test AI-tool suite passing).

**Remaining gap.** Restoring the actual foundation repository metadata — or
binding its remote and intended base — is a prerequisite the project cannot
satisfy from inside this repository. It requires an operator action.

**Expected impact.** Unblocks the entire automation order. Until then, every
throughput control costs manager attention per occurrence rather than per
implementation.

**Implementation cost / risk.** Operator action, not engineering. Risk of the
wrong action is high and already correctly identified: initializing an
unrelated history would destroy rollback and reviewability. **Recommended
framing: this is the single highest-leverage item that no agent can perform.**

---

## 7. Bottleneck-to-lane matrix

| Bottleneck | Python Upload | Python MR-list | Python MR-write | Python composition | Rust Storage | Rust Ancestry | Rust Doctor | Rust HTTP | Final lifecycle | Control plane |
|---|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|:-:|
| B01 build provenance | | | | | ● | ● | ● | ● | ● | |
| B02 shared primitives | ● | ● | ● | ● | ○ | | | ○ | | |
| B03 terminalization | ● | ○ | | | ● | | ● | | | ● |
| B04 coordinator burst | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |
| B05 gate redundancy | ● | ● | ○ | ● | ● | ● | ● | ● | ● | |
| B06 disk/memory | ○ | ○ | | | ● | ● | ● | ● | ● | |
| B07 provider seams | ● | ● | | | ● | ○ | | | | ● |
| B08 Codex quota | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ● |
| B09 service time | ○ | ○ | | | ○ | | | ● | | |
| B10 evidence overhead | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |
| B11 issue drift | | | | ○ | | | | | ● | ● |
| B12 WIP dilution | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |
| B13 install/dogfood | | | | | | | | | ● | |
| B14 clock/telemetry | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |
| B15 foundation Git | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |

● direct measured effect  ○ indirect or predicted effect

## 8. Prioritized removal plan

Ordering rule: unblock the largest measured queue first, then remove the cause
of recurrence, then make the next audit decision-grade, then reduce cost.
Every wave states its exit evidence.

### Wave 0 — Make the next audit measurable (no provider cost, no product risk)

| # | Action | Owner | Exit evidence |
|---:|---|---|---|
| 0.1 | Adopt the monotonic stage clock: every recorded instant carries `(wall_ts, monotonic_ns, boot_id, drift_flag)`; intervals crossing boot epochs are `unavailable` | coordinator | one `flow-metrics` record whose intervals are boot-epoch scoped |
| 0.2 | Open one append-only `events.jsonl` per active batch and emit the §10 event set **by hand** until the scheduler can (the design already fixes the schema) | coordinator | ≥1 full lane with a complete stage/queue event chain |
| 0.3 | Emit `coordinator.action` per durable manager action and enforce the existing 45-minute artifact rule on the coordinator itself | coordinator | no canonical gap >45 min without a heartbeat naming its blocker |
| 0.4 | Add a bound-digest staleness checker over every declared SHA-256 in `task-packets/` and `swarm-results/` | tools | the three-way ledger-hash divergence in §B10 becomes a failing check |

Cost: hours of coordinator time. Risk: none. **Do this first — every other
wave's impact claim is currently unverifiable without it.**

### Wave 1 — Unblock the Rust lane (largest measured queue: ≥13 h)

| # | Action | Exit evidence |
|---:|---|---|
| 1.1 | Complete the source-keyed Cargo guard against all ten audited closure classes | committed guard + passing Python adversarial suite |
| 1.2 | **Prove concurrency, not just correctness**: distinct source keys must run concurrently under resource tokens; publish the lock order and show it is acyclic | a two-lane concurrent Rust gate run with distinct source keys and no lock-order violation |
| 1.3 | Key the namespace on the *Cargo-relevant* input set, not full HEAD, so docs-only changes do not rebuild GiB | a docs-only change producing a cache hit |
| 1.4 | Reject zero-selected-test results fail-closed; bind source key, argv, toolchain digest, nonzero count, duration, and complete log digest to every reusable result | the `0 selected / 794 filtered` case reproduced and rejected |
| 1.5 | Re-run File Doctor `cd130e5` debug+release focused N1/N2/N3 evidence, then release the four held Rust reviews in dependency order | four Rust review launches admitted |

Cost: highest in this plan. Risk: serialization regression — 1.2 is the gate
that prevents it.

### Wave 2 — Stop the recurrence (removes the cause of ~18 of 22 generations)

| # | Action | Exit evidence |
|---:|---|---|
| 2.1 | Re-bind the shared-primitives Fugu preflight to accepted MR-list `b164046` plus the four immutable rejecting reports **as specifications**, and remove the accepted-Upload precondition | preflight packet moves `blocked` → `ready` with no product byte bound |
| 2.2 | Require the preflight `READY` to enumerate the seven primitive contracts with a production entry point, an independent oracle, and a deliberate-regression mutation each | committed `READY`/`BLOCK` report |
| 2.3 | Release the MR-write repair only on that committed `READY` (unchanged rule) | MR-write lane launches after ≥20 h block |
| 2.4 | Route the four accepted-MR-list observations and the inherited POST/GET displacement asymmetry into the same successor | one successor packet, not four surface-local patches |

Cost: low (scheduling and documentation). Risk: ossifying a contract around an
open Upload residual — mitigated by per-finding report-SHA citation.

### Wave 3 — Close the terminalization and handoff seams

| # | Action | Exit evidence |
|---:|---|---|
| 3.1 | Invoke `check_review_terminalization.py` on a timer from the scheduler; emit `progress.verdict_settled` and `progress.report_committed` | measured verdict-to-report latency for ≥3 lanes |
| 3.2 | Bounded nudge: after a verdict signal, ask the exact session to commit its report before any intervention | ≥1 lane terminalized without manager intervention |
| 3.3 | Implement AIA-042 typed provider readiness: inherit credentials without logging, distinguish preflight from worker failure, auto-retry a zero-commit clean launch, adopt a manually resumed exact session | the `rc 3` and 5-second failures self-heal |
| 3.4 | Extend `capture_candidate_identity.py` to `BLOCK` a delivered range that rewrote committed WIP when the packet declares additive-only | the Rust HTTP rewrite case reproduced and blocked |

Cost: medium; 3.3 depends on B15. Risk: premature nudging — gated on a
self-emitted verdict signal.

### Wave 4 — Cut validation cost

| # | Action | Exit evidence |
|---:|---|---|
| 4.1 | Implement the content-addressed gate cache with the design's non-cacheable classes and opt-in per gate | duplicate 507-case run becomes a recorded cache hit |
| 4.2 | Enforce "one broad gate per stable composition revision" mechanically, not in prose | zero duplicate broad scopes per head |
| 4.3 | Never truncate test output that carries a failing test identity; bind complete log bytes/SHA-256 to every focused gate | the lost-flake case becomes classifiable |
| 4.4 | Reclaim a private Cargo target at task exit; add a terminal-worktree retention policy after evidence prefixes are bound | worktree count trends down from 103 |

Cost: medium; 4.1 depends on Wave 1 and B15. Risk: caching a non-deterministic
gate — the opt-in + non-cacheable class list is the control.

### Wave 5 — Restore the foundation (operator action)

Restore the actual `/workspace` foundation Git metadata or bind its remote and
intended base. **Do not initialize an unrelated history.** Then implement
AIA-037 → AIA-028 → AIA-029 → AIA-030..032 → AIA-033 in that order, each as an
independently reviewable black-box partition. This converts Waves 0, 3, and 4
from manual runbook procedures into enforced scheduler behavior.

### Wave 6 — Structural and hygiene items (post-composition)

| # | Action | Precondition |
|---:|---|---|
| 6.1 | Bounded rerun-scope contract for the declaration-only capability mutation (B13) | manager + review-audit decision on acceptance criteria |
| 6.2 | Widen `owner_module` → `owner_modules`, add issue-map ↔ execution-inventory reconciliation (B11) | normal reviewed product delta after composition |
| 6.3 | AIA-035 hot-file decomposition: extract request/session/authority/response/cleanup/publication primitives from the 33,455-line `codefire`, `http.rs` (18,639), `main.rs` (12,647), `remote.rs` (4,101) | accepted Python and Rust composition heads frozen |
| 6.4 | Generated portfolio JSON view alongside the ledger prose (B10) | Wave 0.2 event stream |

## 9. KGI / KPI design

This section **extends** `development-flow-kgi-kpi.md`; it does not replace
it. New IDs use the `MET-NRT-` (non-review throughput) namespace so they cannot
collide with `MET-FLOW-` / `MET-AI-`. Every metric names its source event,
because a metric with no event is a wish.

### 9.1 New KGIs

| ID | Outcome | Target | Guardrail |
|---|---|---|---|
| `KGI-DEV-006` | **Flow efficiency at trusted integration** | Evidence-producing active time ≥ 40% of trusted-integration lead time for the next comparable cohort, measured from typed events only | Never improve the ratio by deleting wait states from the record or by widening "active"; queue time must remain individually attributed |
| `KGI-DEV-007` | **Structural blocker elimination** | Zero critical-path items blocked >4 h by a *control-plane artifact that does not yet exist*; zero fan-in nodes gating >50% of a runtime's items | An item may be blocked indefinitely by a *decision*; it may not be blocked by an unbuilt tool with no owner and no ETA |

`KGI-DEV-006` is currently **unscoreable**: no typed event stream exists.
`KGI-DEV-007` is currently **failing** on two measured counts: the source-keyed
Cargo guard (no canonical candidate, gates 4 of 5 Rust items, ≥13 h) and the
shared-primitives preflight (blocked behind the surface it de-duplicates,
≥20 h on MR-write).

### 9.2 KPI family

All metrics carry the existing measurement dimensions (batch, immutable
revision, packet, task class, stage, role, provider/model, lane, candidate/
review identity, validation class, status marker, evidence locator) plus a new
mandatory `boot_id` and `queue_reason`.

#### Flow efficiency and time decomposition

| ID | Definition / formula | Unit | Source event | Target | Current |
|---|---|---|---|---|---|
| `MET-NRT-001` | **Flow efficiency** = Σ(service seconds) / (trusted-integration lead seconds), per package | ratio | `stage.enter`/`stage.exit` | ≥0.40 | `unavailable` (0 integrations) |
| `MET-NRT-002` | **Queue time by cause** = Σ(`queue.exit`−`queue.enter`) grouped by `queue_reason` ∈ {provider_window, provider_backoff, resource_admission, conflict_domain, dependency, review_permit, manager, build_control, credential} | seconds | `queue.*` | build_control = 0; dependency ≤20% of lead | measured floors only: 13 h 04 m build_control, 20 h 10 m dependency, 7 m resource_admission |
| `MET-NRT-003` | **Service time by stage** = Σ provider-active seconds per stage | seconds | provider result events | context | 4,019.1 / 3,164.0 / 6,808.3 s |
| `MET-NRT-004` | **Idle-coordinator time** = Σ intervals >45 min with no `coordinator.action` while ≥1 lane is terminal-or-blocked | seconds | `coordinator.action` | 0 | ≥10.3 h derived (6.65 + 3.65) as a *visibility* bound, not proven idleness |
| `MET-NRT-005` | **Handoff latency** = `review.start` − `candidate.ready` (non-blocked cases only) | seconds | `candidate.ready`, `lane_start` | p50 ≤300 s, p95 ≤1,800 s | 84.7 s, 105.6 s, "<5 min" — currently **passing** |

#### Rework, recurrence, and acceptance

| ID | Definition / formula | Unit | Source | Target | Current |
|---|---|---|---|---|---|
| `MET-NRT-006` | **First-pass exact acceptance** = candidates accepted on their first independent decision / candidates reviewed | ratio | `review.decision` | ≥0.60 next cohort | 19.05% raw indexed; terminal trusted yield 0 |
| `MET-NRT-007` | **Review cycles per surface** = independent decisions until an accepted terminal candidate | count | `review.decision` | mean ≤1.5 | measured: Upload ≥3, MR-list 3, Doctor 2, HTTP 2, Storage 2, Ancestry 1 |
| `MET-NRT-008` | **Same-root recurrence rate** = findings whose `invariant_family` already appeared in an earlier accepted-or-rejected report on a *different* surface / total findings | ratio | `review.finding` (family owned by the review audit — §11) | ≤0.20 | `unavailable`; qualitatively dominant across 7 primitive families |
| `MET-NRT-009` | **Generations per integrated delta** = candidate generations / integrated deltas | ratio | `candidate.ready`, `integration.*` | ≤2.0 | **22 / 0 = undefined** — the single clearest failure signal in this window |
| `MET-NRT-010` | **Rework seconds** = Σ service seconds in generations ≥2 / Σ all service seconds | ratio | `stage.*` + `candidate.ready` lineage | ≤0.20 | `unavailable` (no per-generation service attribution) |

#### Terminalization, handoff, and continuity

| ID | Definition / formula | Unit | Source | Target | Current |
|---|---|---|---|---|---|
| `MET-NRT-011` | **Terminalization lag** = `report.committed` − `verdict.settled` (first self-emitted verdict signal) | seconds | `progress.verdict_settled`, `report.committed` | p95 ≤600 s | ≥1,200 s on Upload (measured floor); Storage `unavailable` but 135 post-verdict events |
| `MET-NRT-012` | **Session-handoff cost** = seconds from a rejecting report commit to the exact original implementer session resuming | seconds | `report.committed`, `lane_start` with `continuation_of` | p95 ≤1,800 s | 02:07→02:17 = 600 s (MR-list); 02:33→02:42 = 540 s (Upload); Storage ≥4 h 24 m (blocked by B01) |
| `MET-NRT-013` | **Coordinator token delta per measurement window** and per trusted package (replaces the mis-defined cumulative `MET-FLOW-024`) | tokens | coordinator thread sample deltas | trend down between cohorts | 388,898 tokens / 1,662 s = 234 tokens/s ≈ 842k/h; per-package `unavailable` |
| `MET-NRT-014` | **Quota stoppage** = batch-blocking incidents attributable solely to Codex quota | count | `provider.error{kind:quota}` | 0 | **0 measured** in this window; latent per §B08 |
| `MET-NRT-015` | **Coordinator continuity readiness** = seconds to reconstruct the full portfolio board from durable evidence alone, measured by drill | seconds | drill record | ≤1,800 s | `unavailable` — never drilled |

#### Validation economics

| ID | Definition / formula | Unit | Source | Target | Current |
|---|---|---|---|---|---|
| `MET-NRT-016` | **Build-cache reuse** = cache-hit gates / cacheable gates; and rebuilt bytes per candidate | ratio, bytes | `gate.result{cache_decision}` | ≥0.60 hits on stable scopes | `unavailable` (cache unimplemented); measured counter-evidence: 3.20 GB + 2.33 GB + 1.59 GB targets discarded in 42 min |
| `MET-NRT-017` | **Test redundancy** = duplicate (scope, tree) gate executions / total gate executions | ratio | `gate.result` | 0 duplicates per stable head | ≥1 measured duplicate (507-case scope twice on one candidate); 10-run and 13-run repetition sequences |
| `MET-NRT-018` | **Broad-gate placement** = broad gates on non-integrated heads / all broad gates | ratio | `gate.result{class:broad}` | ≤0.50, then ≤0.25 | **11 / 11 = 1.00** |
| `MET-NRT-019` | **Vacuous-gate rate** = gates returning success with zero selected tests or unreachable-subject fixtures / gates run | ratio | `gate.result{selected,filtered}` | 0 | ≥2 measured (Cargo `0 selected/794 filtered`; the vacuous 1x/3x/10x reachable-object axis in the Rust HTTP report) |
| `MET-NRT-020` | **Evidence overhead** = control/evidence lines committed per integrated product line | ratio | `git` | reported, not targeted | **36,392 / 0 = undefined**; report as an absolute until the denominator is nonzero |

#### WIP, resources, and drift

| ID | Definition / formula | Unit | Source | Target | Current |
|---|---|---|---|---|---|
| `MET-NRT-021` | **WIP age** = now − last unique trusted signal, per unfinished item; report p50/p95/max | seconds | `wip.snapshot` | p95 ≤4 h | measured max ≥20 h 10 m (MR-write), ≥13 h 04 m (Ancestry) |
| `MET-NRT-022` | **Blocked-packet debt** = packets in `blocked` with ≥1 stale bound digest | count | bound-digest checker | 0 stale | ≥1 measured (lifecycle packet binds `af8d5dcc…` vs `cc60d9af…`) |
| `MET-NRT-023` | **Fan-in concentration** = max share of a runtime's open items gated by one artifact | ratio | execution inventory | ≤0.50 | **0.80** (Cargo guard gates 4/5 Rust items) |
| `MET-NRT-024` | **Retained state growth** = Δ`/workspace/worktrees` bytes and directory count per window, and per integrated delta | bytes, count | resource sample | ≤0 per window with 0 integrations | +557,079,615 B, +6 dirs, +4 state files in 11.65 h |
| `MET-NRT-025` | **Restart cost** = incidents × (recovery seconds + lanes lost) | count, seconds | `incident`/`recovery` events | 0 incidents per cohort | 3 restarts in the window; ~10 min Git reconstruction (#3); 19 + 12 zero-length objects repaired |
| `MET-NRT-026` | **Issue-state drift** = in-scope issues whose declared owner set does not cover the union of live-packet write sets | count | issue map + packets | 0 | ≥1 measured (`CFR-168`) |

### 9.3 Anti-gaming rules (additive to the existing anti-metrics list)

1. A metric may never improve because an event stopped being emitted. Every
   ratio publishes its denominator and its `unavailable` count alongside it.
2. `MET-NRT-001` flow efficiency may not be improved by reclassifying queue
   time as service time. "Active" means a provider process or a gate process
   was running with a bound log.
3. `MET-NRT-006` first-pass acceptance is per *surface generation*, not per
   report; reviewing the same candidate twice does not create a second chance.
4. `MET-NRT-018` counts scopes, not commands: renaming a gate does not make it
   a new scope.
5. Thresholds change only by dated decision with cohort and evidence, per the
   existing policy. This audit changes none.
6. Anti-metrics stay anti-metrics: worktree count, agent count, commit count,
   ledger length, and token volume are context only. Note especially that
   `MET-NRT-020` and `MET-NRT-024` are *diagnostics*, not targets — shrinking
   evidence to improve them would violate `KGI-DEV-002`.

## 10. Instrumentation required to make the next audit decision-grade

This audit could not compute flow efficiency, rework seconds, per-cause queue
totals, terminalization lag on one of two occurrences, cache reuse, or any
cross-provider cost. The following is the minimum to close those gaps. The
schema is already designed (`dev-swarm.event.v1`); what is missing is emission.

### 10.1 Event set (append-only, one stream per batch)

Every event carries: `schema`, `event_id`, `batch`, `task`, `lane`, `role`,
`provider`, `model`, `stage`, `wall_ts`, `monotonic_ns`, `boot_id`,
`drift_flag`, and an evidence locator with `prefix_bytes + sha256`.

| Event | Fields beyond the common set | Closes |
|---|---|---|
| `stage.enter` / `stage.exit` | `stage` ∈ {discover, freeze, preflight, implement, checkpoint, candidate_freeze, validate, review_ready, review, verdict_settled, report_commit, rollover, integrate, compose, close, install, dogfood, retrospect} | `MET-NRT-001/003` |
| `queue.enter` / `queue.exit` | `queue_reason`, `blocking_artifact` (path or SHA), `expected_release_condition` | `MET-NRT-002`, `KGI-DEV-007` |
| `candidate.ready` | base, head, tree, parent, patch digest, changed paths, write-set conformance, clean state, generation index, `supersedes` | `MET-NRT-005/009` |
| `gate.start` / `gate.result` | validation ID, exact argv, cwd, execution profile, **source key**, toolchain digest, fixture version, selected/filtered/pass/fail counts, monotonic duration, complete log bytes+SHA-256, `cache_decision` | `MET-NRT-016/017/018/019` |
| `progress.verdict_settled` | first self-emitted verdict signal instant, signal count | `MET-NRT-011` |
| `report.committed` | report path, blob SHA-256, patch SHA-256, terminal verdict line | `MET-NRT-011` |
| `rollover.bound` | successor packet, `continuation_of` session, admission decision | `MET-NRT-012` |
| `coordinator.action` | action kind, produced artifact or exact blocker | `MET-NRT-004` |
| `provider.error` | typed kind ∈ {429, 529, quota, auth, credential, model_list, preflight, worker}, retries, provider seconds, cost | `MET-NRT-014`, B07 |
| `resource.sample` | `/workspace` bytes, per-target bytes, `MemAvailable`, swap, PSI triple, PID headroom, filesystem free | `MET-NRT-024/025` |
| `wip.snapshot` | per-item last trusted signal, age, blocking artifact | `MET-NRT-021` |
| `clock.discontinuity` | wall-minus-monotonic delta, affected interval set | `MET-NRT-*` validity |
| `integration.*` | accepted delta, composition head, post-integration focused gate result | `MET-NRT-001/009` |

### 10.2 Derived-value rules

- Any interval spanning two `boot_id` values, or any interval whose endpoints
  straddle a `clock.discontinuity`, is emitted as `unavailable` — never
  estimated.
- Cross-provider cost is `unavailable` until Fugu reports cost; Claude-only
  totals are published as Claude-only and never summed with a missing term.
- Ratios publish numerator, denominator, and `unavailable` count.

### 10.3 Minimum viable subset if only three things can be built

1. `gate.result` with source key, selected/filtered counts, monotonic duration,
   and complete log digest — closes B01's false-green class **and**
   `MET-NRT-016/017/018/019` at once.
2. `queue.enter` / `queue.exit` with `queue_reason` and `blocking_artifact` —
   the only way to prove where the 13 h and 20 h waits actually went.
3. `progress.verdict_settled` + `report.committed` — closes the
   terminalization loop that already consumed two manual interventions.

## 11. Interfaces with the parallel review root-cause audit

**Ownership boundary.** The review root-cause audit owns: reviewer
independence, verdict vocabulary and semantics, `ACCEPT` sufficiency under the
zero-residual objective, finding severity, the invariant/root taxonomy, review
depth and counterexample requirements, and the review-process redesign itself.
This audit owns: *when and where* reviews are scheduled, how fast candidates
and reports become durable, how validation evidence is produced and reused,
and how resources and dependencies gate all of it. Neither may silently change
the other's rules.

**What this audit provides to the review audit.**

| Interface | Content | Contract |
|---|---|---|
| `IF-1` event stream | The §10 event set, including `review.start`, `progress.verdict_settled`, `report.committed`, `rollover.bound` | The review audit may add `review.finding` events to the same stream; it must not redefine existing fields |
| `IF-2` queue attribution | Per-cause queue time (`MET-NRT-002`), so a "slow review" can be split into review service versus build_control / dependency / permit wait | A review-duration claim must cite `MET-NRT-002` before attributing delay to review |
| `IF-3` terminalization telemetry | `MET-NRT-011` verdict-to-report latency and post-verdict event counts | This audit measures the lag; the review audit decides whether nudging or intervention is permissible without weakening independence |
| `IF-4` gate integrity | `MET-NRT-019` vacuous-gate rate and the source-key contract | A report may not treat a zero-selected or foreign-artifact gate as evidence; this is a *measurement* fact the review audit converts into an acceptance rule |
| `IF-5` recurrence counting | `MET-NRT-008` computation | This audit supplies the counting; the **review audit owns the `invariant_family` taxonomy** — the metric is unscoreable until that taxonomy is published |

**What this audit needs from the review audit.**

1. A published, stable `invariant_family` enumeration (at minimum: request
   ledger/deadline, sealed snapshot, authority fence, prepared response,
   retryable cleanup, publication transaction, error origin, allocation
   accounting, evidence non-vacuity) so `MET-NRT-008` becomes computable.
2. A definition of "first-pass" that survives successive zero-residual
   candidates on one surface, so `MET-NRT-006` is not distorted by the
   deliberate multi-generation policy.
3. A decision on B13: whether a bounded, proven differential can discharge
   post-mutation Install/Dogfood evidence, or whether a full terminal rerun is
   mandatory. This audit measures the cost; it does not decide the criterion.
4. A decision on whether the shared-primitives preflight may bind *rejecting*
   reports as specifications (Wave 2.1). This audit's position is that it can,
   because the preflight is report-only and binds contracts, not product bytes
   — but the acceptance-criteria implication belongs to the review audit.

**Explicitly out of this audit's scope.** Review recursion depth, the
zero-residual rule, reviewer model selection, whether `ACCEPT` should imply
integration authority, and any change to the disposition index's semantics.

## 12. Non-authorization and write boundary

This audit does **not**:

- accept, reject, or integrate any product candidate;
- infer a verdict from a candidate card, worker gate, scheduler `passed`, or
  active session;
- authorize CFR-168, CFB-136, or CFB-138 Close;
- authorize Install, Dogfood, release, or capability re-advertisement;
- change the recovery ledger, review disposition index, issue state, KGI/KPI
  policy thresholds, task packets, manifests, or any product source;
- launch or stop any agent, provider, or scheduler process; or
- modify Cargo, Rust, network, or credential state.

The commit carrying this audit adds exactly one path:

```text
docs/development/ai/swarm-results/cfr168-non-review-throughput-bottleneck-audit-claude-20260731/audit.md
```

Adopting any recommendation in §8 requires its own packet, its own exact
review where the existing rules demand one, and — for §6.2 and §6.3 — a normal
reviewed product delta after the accepted compositions are frozen.
