# CFR-168 Review-Recursion Root-Cause Audit

Status: manager operating analysis. **Not** product acceptance evidence, not a
review decision, not an integration authorization.

Date: 2026-07-31

Reviewer/analyst: Claude Opus 5 (`claude-opus-5`), report-only lane
`cfr168-review-recursion-root-cause-audit-claude-fallback`.

Scope owner: **why CFR-168 review cycles recur** — review method, finding
taxonomy, repair quality and communication, repair strategy, test/oracle
quality, validation provenance as it enters an acceptance decision, scope
change, and the interactions between them. Scheduling, queueing, resource, and
provider mechanics are owned by the parallel non-review audit at this same head
(§11).

The audit began from the explicit instruction **not** to assume review omission
and **not** to preselect a checklist size or template. Both conclusions below
are therefore derived, not assumed: omission is one of five roughly comparable
causes rather than the dominant one (§8), and no template depth is proposed —
§9 specifies the pilot that must produce it.

---

## 1. Exact snapshot

| Field | Value |
|---|---|
| Canonical head (audit subject) | `c258cbfc061b14905554c9d319b350d2da7f771b` |
| Head committed at | `2026-07-31T08:37:14Z` |
| Audit captured at | `2026-07-31T08:57:12Z` |
| Immutable product baseline | `e2a218e015953f082245f36dbc6aee906b837e5a` |
| Canonical commits since baseline | `200` (recomputed) |
| Product bytes changed since baseline (`codefire`, `crates/`, `tests/`, `Cargo.toml`, `Cargo.lock`) | **0** (`git diff` empty; recomputed) |
| Recovery ledger SHA-256 at head | `cc60d9af5b91e01f3b1d4429fc9aa0a5d583f85660fa905c606408137795a14a` |
| Review index SHA-256 at head | `61bc3449ea3144c2bbb43a313433db5e76db0b4695301888d0bc3ff3c8238acd` |
| Open-execution audit SHA-256 at head | `61a1f145d5a296918c5ae5943d853cdc080d9e0073ec46d70268eb56a3074a2f` (frozen subject `664f907…`) |
| Non-review throughput audit SHA-256 at head | `f4e0f00014233f52ec7792accbc1dae61a6f0e7a888866d62e3be3b6405e4b3b` |
| Indexed independent reports | `21` (19 product-surface, 2 control-plane) |
| Indexed decisions | `4 ACCEPT` / `17 REQUEST_CHANGES` (recomputed from each report's own verdict line) |
| Total report volume | `11,411` lines |

This audit performed read-only repository and `git` observation plus
**object-level verification of candidate diffs**. No product byte, Cargo/Rust
command, provider launch, network call, or agent session was created or
modified. Its only write is this file.

### 1.1 Sources read

- `AGENTS.md`
- all **21** indexed independent reports under
  `docs/development/ai/swarm-results/*/review.md` (digests recomputed; §1.3)
- `docs/development/ai/swarm-results/cfr168-review-disposition-index-20260730/index.md`
- `docs/development/ai/swarm-batches/cfr168-recovery-execution-ledger-20260730.md`
  (1,759 lines)
- `docs/development/ai/swarm-results/cfr168-open-execution-audit-20260731/audit.md`
  and `open-items.v1.json`
- `docs/development/ai/swarm-results/cfr168-non-review-throughput-bottleneck-audit-claude-20260731/audit.md`
  (1,290 lines, read in full including §11)
- the six `*-candidate-ready-*/candidate.md` manager cards
- repair packets / tickets in `docs/development/ai/swarm-batches/` for Upload
  (scale, zero-residual, review-closure), MR-list (zero-residual, closure,
  final-publication), MR-write (zero-residual), File Doctor (zero-residual,
  note-closure), Rust HTTP (review-closure, evidence-closure), Storage
  (zero-residual, control addendum, resume preflight), File Ancestry
- `docs/development/ai/task-packets/` (19 packets), including the KGI/KPI repair
  and manager-counterexample packets and the shared-primitives preflight
- `docs/development/ai/swarm-results/current-state-closure-audit-20260729/04-cfr168-review.md`
  (the root review carrying M1–M6 / S1–S5)
- `docs/development/ai/development-flow-kgi-kpi.md`,
  `docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json`
  (7 lane runs, 29 measurements, 15 events, 6 limitations)
- `docs/development/ai/swarm-results/cfr168-trusted-throughput-audit-20260730/audit.md`,
  `dev-swarm-throughput-design-20260730/design.md`,
  `stage5-manual-control-runbook.md`, `operating-protocol.md`,
  `validation-report-template.md`, `provider-routing-policy.md`,
  `model-routing-policy.md`, `failure-cookbook.md`
- `docs/development/ai/swarm-results/cfr168-python-http-upload-review-closure-rereview-launch-20260731/incident.md`
- **candidate objects**: every candidate commit named in the index was resolved
  in this worktree's object database and inspected directly (§1.2)

### 1.2 Candidate-diff availability — the decisive method advantage

All 22 candidate generations named in the disposition index and the non-review
audit resolve as commits in this worktree:

```text
e2a218e dece688 76baff8 467b0c2 37f7427 38da898 4930d25 b164046
21fe751 6dd4527 701469f 52a025e 1a04490 cd130e5 71854e2 73bc532
92fc810 b3bc32a cdcb9bd e93ebc2 02b0d63 0ae583e   (all: git cat-file -t → commit)
```

This is what makes the central attribution question decidable rather than
narrative. For every later finding, the two competing hypotheses —

- **pre-existing and missed** (`INITIAL_REVIEW_SCOPE_MISS`), versus
- **absent before, introduced by the repair diff** (`REPAIR_INTRODUCED_REGRESSION`)

— were tested by resolving the finding's own cited symbol against the exact
predecessor and successor trees, not by trusting either report's framing. §5
marks every such determination `[P]`. Where a symbol test was decisive it
sometimes **contradicted** the natural reading of the report (for example
MR-list MUST-1 at §5.2: the report presents a fail-open arm as though the
repair introduced it; the arm is present at the legacy baseline `7327e2a`).

### 1.3 Report digest integrity

All 21 report SHA-256 digests were recomputed and every one matches the value
declared in the disposition index. The index is **not** stale on report
identity, which means the immutable-evidence chain this audit reasons over is
sound. (This is a narrower claim than the index being current on state; the
open-execution audit already records that some packets bind older ledger/index
hashes.)

---

## 2. Measurement-integrity register

Every claim in this audit carries one marker. Nothing is invented, and no
inference is dressed as proof.

| Marker | Meaning |
|---|---|
| `[P]` | **Proven by this audit at `c258cbf`** — recomputed by me from the object database, a digest, or a count. Independent of any report's framing. |
| `[R]` | **Proven by an immutable report's own executed reproducer**, quoted with its locator. I did not re-execute it (no Cargo/provider/product execution in this lane); I verified the report's internal consistency and, where a symbol or line was cited, its existence at the named revision. |
| `[T]` | **Proven from a ticket/packet/card text** at a named path — a requirement-versus-delivery comparison, which is a documentary fact. |
| `[I]` | **Inference by me** from `[P]`/`[R]`/`[T]` facts, with the reasoning stated. Never presented as measurement. |
| `[U]` | **Unavailable** — no telemetry or artifact exists. Explicitly not replaced by an estimate. |

Contaminations carried forward from the non-review audit and independently
relevant here:

1. **Wall-clock discontinuity.** The container clock advanced ≈`4 h 50 m`
   during active validation on 2026-07-30/31. Every wall-derived review
   interval crossing that window is `[U]`. This is why §7 can state review
   *counts* but almost no review *durations*.
2. **No `review.finding` event stream exists.** Findings live only as prose in
   21 reports. Severity, invariant family, and closure state are not machine
   readable anywhere. Every aggregate in §6 was hand-derived by me from the
   reports' own severity headings, and is therefore `[I]` at the aggregate
   level even where each row is `[P]`/`[R]`.
3. **`MET-FLOW-006` (review lead time) and `MET-FLOW-015` (reviewer correction
   rate) are defined in `development-flow-kgi-kpi.md:56` and `:70` and have
   **no recorded value** in `flow-metrics/cfr168-recovery-20260730.v1.json`
   `[P]`. The one flow record was captured at `2026-07-30T09:51:55Z` on source
   revision `dd993cc…` and predates 17 of the 21 reports.
4. **Two reviewer lanes ran with no execution at all.** The Storage initial
   review states "this lane ran no Cargo, `rustc`, `rustfmt`, Clippy, test,
   build, install, or demo command and created no target artifact. Validation
   was limited to non-building Git and text inspection" `[R]`. Its findings are
   sound source proofs but carry no oracle. This is a *review-method* variable,
   not a telemetry gap, and it is scored in §6.4.

---

## 3. What the review system actually produced

| Signal | Value | Marker |
|---|---:|---|
| Independent reports | 21 | `[P]` |
| `ACCEPT` / `REQUEST_CHANGES` | 4 / 17 | `[P]` |
| Raw indexed acceptance yield | 19.05% | `[P]` derived (4/21) |
| Accepted **and integrated** deltas | 0 | `[P]` |
| Candidate generations across 6 product surfaces + 1 control plane | 22 | `[P]` (index + non-review audit §3.1, cross-checked against the object database) |
| Enumerated severity-listed findings across the 21 reports | **143** | `[I]` hand-count from each report's own severity headings |
| Plus recorded non-blocking observations | 7 | `[I]` |
| Findings in **successor** reports (generation ≥ 2) — this audit's classification universe | **51** | `[I]` |
| Reviewer findings later withdrawn, reversed, or shown to be probe artefacts | **4** | `[P]` grep over all 21 reports (§6.5) |

### 3.1 Generation and diff-size series (all `[P]`, recomputed)

| Surface | Generations | Per-generation product+test delta | Terminal state at head |
|---|---:|---|---|
| Python Upload / CFB-138 | 4 | `+2,105/−26` → `+2,227/−120` → `+1,293/−40` | `REQUEST_CHANGES` (HIGH-2, MEDIUM-4) |
| Python MR-list | 4 | `+2,548/−18` → `+1,667/−105` → `+2,323/−287` → `+813/−122` | `ACCEPT`, not integrated |
| Python MR-write | 2 | `+2,718/−133` → `+2,094/−295` | rejected; successor blocked |
| Python Doctor+Branch | 2 | Doctor slice `+854/−41`, Branch slice `+1,316/−32` → **`+17/−2`, one file** | `ACCEPT` as composition input |
| Rust Storage | 2 | `+2,310/−498` (`40c095d..b3bc32a`) → `+4,007/−291` | rejected, F1–F9 all open |
| Rust File Ancestry | 2 | `+1,751/−3` → `+3,488/−815` | review never launched |
| Rust File Doctor | 3 | `+1,272/−32` → `+2,132/−266` → `+2,517/−291` | review launch `BLOCK` |
| Rust HTTP MR | 3 | `+1,403/−47` → `+3,426/−560` → `+1,927/−73` | review launch blocked |
| KGI/KPI (control plane) | 2 | `+1,087/−3` → `+4,011/−415` | `REQUEST_CHANGES` (20/35 partial) |

Two facts in this table do more causal work than any other quantity in the
audit:

- **Repairs are not smaller than the work they repair.** Across the 14
  successor-generation deltas, median insertions are **2,180**; eleven exceed
  1,600 insertions and four exceed 3,400 `[P]`. The Storage repair
  (`+4,007/−291`) responded to 9 findings and produced 14 new ones. The Rust
  HTTP repair (`+3,426/−560`) responded to one blocking *test-coverage* finding
  plus four low items and four non-blocking residual risks, and produced four
  new evidence findings.
- **The one lineage that converged did so by shrinking.** MR-list is the only
  product surface that reached `ACCEPT`, and its accepted generation
  (`+813/−122`) is the smallest of its four `[P]`. The other convergent lineage
  (Doctor+Branch) converged on `+17/−2` in one file `[P]` — a two-assertion
  correction. The two smallest deltas in the entire corpus are the only two
  accepted-and-terminal ones.

---

## 4. Causal taxonomy and the attribution protocol

The ten classes were supplied. Two of them require a stated reference point,
because "pre-existing" and "introduced" are only meaningful relative to a
specific tree. This audit fixes that reference point as follows and applies it
uniformly.

| Class | Applied meaning in this audit | Decision test |
|---|---|---|
| `INITIAL_REVIEW_SCOPE_MISS` | The defect was present in the candidate reviewed by the **immediately preceding report in the same lineage**, lay inside that report's own declared scope, and was not reported. | resolve the cited symbol/line at the predecessor tree; read the predecessor report's declared scope and item list |
| `BOUND_FINDING_NOT_CLOSED` | A finding or requirement **explicitly bound by a report or repair ticket** was still open on the successor. | ticket/report text vs successor report disposition row |
| `REPAIR_INTRODUCED_REGRESSION` | The defect is in code **absent from the predecessor tree** and present in the successor, i.e. created by the repair diff. | symbol/occurrence count at predecessor vs successor |
| `PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED` | The failure mode is the *foreseeable* consequence of the prescribed repair (relocating a bound, widening a catch, adding a state machine) and no ticket clause, premortem, or oracle required it to be falsified. | ticket text: is the hazard named? |
| `VACUOUS_OR_LATE_TEST_ORACLE` | An oracle passes while the property it names is false: mutation survives, the assertion fires after the event it is meant to prevent, the axis never moves, or the docstring claims more than the assertion. | the report's own mutation result or oracle reading |
| `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` | The same invariant family is defective in a shared helper/dispatcher, or across a GET/POST or runtime pair, and was repaired surface-locally. | recurrence of the family across surfaces |
| `VALIDATION_PROVENANCE_FALSE_GREEN` | An acceptance-relevant gate returned success while not exercising the candidate: zero selected tests, foreign artifact, unreachable subject. | gate record |
| `REQUIREMENT_OR_SCOPE_EXPANSION` | The blocking criterion was created, widened, contradicted, or made unsatisfiable by policy or by a reviewer-authored closure clause rather than by a demonstrated defect. | criterion text vs demonstrated effect |
| `EVIDENCE_DOCUMENTATION_ONLY` | The blocking defect is a claim (commit message, docstring, canonical source doc) rather than shipped behavior. | is behavior wrong, or only the claim? |
| `ATTRIBUTION_UNAVAILABLE` | No successor report, no diff, or no evidence permits a determination. | — |

**Multiple causes are assigned where each is independently provable.** A
primary cause is the one whose removal would most plausibly have prevented the
cycle; contributing causes are provable co-factors. Where I could not prove a
class I say so rather than rounding it into a neighbour.

**What I deliberately did not do.** I did not treat a report's own framing of a
finding as its cause. Three times the object database contradicted the
framing (§5.2 M2-MUST1, §5.1 U1-H2, §5.3 W2-F2), and once a later report
narrowed a criterion without saying that the earlier criterion had been
unsatisfiable (§5.2 M3-F1).

---

## 5. Lineage finding ledger

Every row below names the finding, the exact evidence, and the classification.
`gen` numbers count candidate generations on that surface.

### 5.1 Python Upload / CFB-138 — `cb13da0 → dece688 → 76baff8 → 467b0c2`

Reports: `534f0983…` (gen 2, Claude), `7632552e…` (gen 3, Fugu),
`57d67921…` (gen 4, Fugu). Terminal: `REQUEST_CHANGES`, 2 residuals open.
Note the lineage has an **unindexed** predecessor report `985ca9e4…` referenced
as evidence-only by the gen-2 report.

| ID | Finding | Evidence | Primary | Contributing |
|---|---|---|---|---|
| U1-B1 | Publication-root validation runs two request-bypassing sealed traversals; an expired deadline still publishes | `[R]` `/tmp/probe_deadline.py`: 1.0 s elapsed vs 100 ms limit → `200`, branch moved, object durable, `expired_phase` never set. `[P]` all `codefire` below line 21912 is byte-identical to `cb13da0`, so the finding is untouched by the CFB-138 diff | `BOUND_FINDING_NOT_CLOSED` — reported at `985ca9e4…`, still open on the next reviewed candidate | `REQUIREMENT_OR_SCOPE_EXPANSION` — the `dece688` ticket was the *scale* packet; the two lanes were never serialized, so a bound blocker survived an unrelated repair by construction |
| U1-H2 | Reservation still charges a per-record 128× heap upper: the default profile refuses any bundle holding one record above ~127 KiB | `[R]` bisect: `≤126,976 B` admitted, `≥131,072 B` refused `temporary_bytes`, `≥262,144 B` refused `retained_bytes`; `33554432/128 = 262144` reproduces the ceiling exactly. `[P]` `RETAINED_BYTES_MULTIPLIER` occurs 2× at `cb13da0` (applied to `aggregate_bytes`, line 22396) and 3× at `dece688` — the per-record term `+ RETAINED_BYTES_MULTIPLIER * maximum_record_bytes` is an **insertion of the repair** | `REPAIR_INTRODUCED_REGRESSION` — proven; the report's own words: "the same root cause CFB-138 named, relocated from record *count* to record *size*" | `PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED` — the ticket named the "all records simultaneously live" heap factor as the root cause; relocating the same factor onto a different axis is the foreseeable failure and no clause required the size axis to be falsified. `VACUOUS_OR_LATE_TEST_ORACLE` — the repair's 1x/3x/10x curve pins record *count* (127/367/1,207); no test named a size boundary (repair step 5(d) had to create one) |
| U1-H3 | Traversal commit source builds a validation session with no memory owner | `[R]` 24 instrumented sites, `memory_owner` omitted at `codefire:6653`. `[P]` code byte-identical to `cb13da0` | `BOUND_FINDING_NOT_CLOSED` | — |
| U1-M4 | `_fd_owners` grows one never-removed descriptor owner per read; `finalize` allocates an unclaimed per-record id list | `[R]` 78 appends measured on a 27-record upload; `_pair_plan_upper` budgets a small constant | `INITIAL_REVIEW_SCOPE_MISS` — pre-existing (`[P]` `_fd_owners` present at `cb13da0`), squarely inside the charge-before-allocate rule, unreported by `985ca9e4…` | `VACUOUS_OR_LATE_TEST_ORACLE` — `tests…:1093` covers only the ancestry allocators |
| U2-B1 | Completed-quarantine upload recovery drops the request ledger and deadline | `[R]` `/tmp/cfr168_recovery_probe.py`: fresh `sealed validation`/`local_graph` ledger minted, `poll_count` frozen at 2,350 after the clock advance, exact retry returns `200` and deletes the quarantine. `[P]` **the unthreaded `validate_sealed_commit(remote_dirs(project_root)["objects"], root_id)` inside `recover_remote_pending_quarantine` is present at `e2a218e` and at `dece688`** | `INITIAL_REVIEW_SCOPE_MISS` — pre-existing, upload-reachable (`handle_http_upload → execute_remote_json_publication → recover_remote_pending_quarantine`), inside U1's declared route scope. U1 counted 27 call sites and named six non-upload ones; it did not name `codefire:2999`. U2's AST inventory found 28 and identified exactly four HTTP-reachable owners | `BOUND_FINDING_NOT_CLOSED` — U1 item 1 ("one exact request ledger/deadline owns body → cleanup") was `FAIL`; the repair closed only the ordinary route. `PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED` — U1's residual-risk section flagged the 27-site blast radius, and the repair ticket did not convert that into a required exhaustive upload-reachability enumeration |
| U2-H2 | The newly advertised caller-owned sealed-validation API cannot complete: the source owner reserves no `temporary_bytes` for `finalize`'s ordered projection | `[R]` all three public paths raise `CFError: upload commit source exceeded its prepared temporary_bytes reservation`; prepared temporary upper `0`, empty projection needs `56`. `[P]` `_AccountedUploadSourceMemory` absent at `dece688`, 6 occurrences at `76baff8` | `REPAIR_INTRODUCED_REGRESSION` — proven | `REQUIREMENT_OR_SCOPE_EXPANSION` — U1 repair step 3 asked for a memory owner at `codefire:6653`; the candidate delivered a **public reusable API** advertised in `source-architecture.md:391`, i.e. new public surface created inside a repair. `VACUOUS_OR_LATE_TEST_ORACLE` — no direct test of the three public paths existed; the reviewer had to author `f3_api.py`/`f3_real.py` |
| U2-H3 | `RemoteJSONPublicationRootProof` accepts repeated consumption before revocation | `[R]` `attempt=1 accepted … attempt=2 accepted … consumptions=2`. `[P]` the class is absent at `dece688`, present at `76baff8` | `REPAIR_INTRODUCED_REGRESSION` — proven | `BOUND_FINDING_NOT_CLOSED` — the F1 state-machine condition required single-use *and* revocable; only revocable was delivered. `VACUOUS_OR_LATE_TEST_ORACLE` |
| U2-M4 | Mandatory reversal coverage vacuous for two accounting invariants | `[R]` `m6_no_projection_claim` → focused 22/22 PASS **and** broad 33/33 PASS; `m8_disable_coverage` → 22/22 and 33/33 PASS; forced-raise proves execution only | `VACUOUS_OR_LATE_TEST_ORACLE` — proven by surviving mutation | `BOUND_FINDING_NOT_CLOSED` — the ticket required deletion-detecting reversals |
| U3-H2 | `_AccountedUploadSourceMemory`/`SealedCommitValidationSession` erase claim/owner state before a fallible `DecisionWorkLedger.release`; a pre-effect release failure strands `11,456` retained or `432` temporary bytes irrecoverably | `[R]` four fault-injection rows, all with zero open permits and `terminal_reason = None` — a **silent permanent** leak. `[P]` at `76baff8` the class's `close()` sets `_closed = True` and `_acquired = False` **before** `self._ledger.release("retained_bytes", self._control_upper)` — i.e. the defect is in the *rejected predecessor* and was not reported by U2. `[T]` the gen-4 ticket items 3 and 5 require "retryable release … cannot double-charge or detach debt" and "release failure and retry" | `REPAIR_INTRODUCED_REGRESSION` — the owner class was created by the `76baff8` diff | `INITIAL_REVIEW_SCOPE_MISS` — present at `76baff8`, inside U2's stated closure scope ("keep acquisition publication and retryable release ordering"), unreported. `BOUND_FINDING_NOT_CLOSED` — ticket items 3 and 5. `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` — the identical invariant is MR-list `MUST-3` and MR-write `F5` (§6.3) |
| U3-M4 | The ordered-projection oracle observes ledger state at the **sort**, after `list(self.documents)` has already made the unowned allocation | `[R]` `m1` and `m9` both fail at `tests…:3002` *after* the allocation; the report: "the ticket explicitly rejects an oracle that fails after the unowned allocation". `[T]` ticket: "The projection oracle must assert acquisition-before-allocation"; the implementer's own `result.md` at `467b0c2` states the oracle "asserts acquisition before the sort" | `VACUOUS_OR_LATE_TEST_ORACLE` — a textbook *late* oracle | `BOUND_FINDING_NOT_CLOSED` — proven by ticket-vs-result text. The test is named `test_ordered_projection_claim_is_exact_and_precedes_its_allocation`, so the **name** asserts the property the oracle does not check |

Upload lineage summary — 10 findings across three reports, by primary class:
**4** in code a repair created (`U1-H2`, `U2-H2`, `U2-H3`, `U3-H2`, all `[P]`
by symbol-occurrence test), **2** pre-existing-and-missed (`U1-M4`, `U2-B1`),
**2** bound findings not closed (`U1-B1`, `U1-H3`), **2** oracle defects
(`U2-M4`, `U3-M4`). Oracle defects additionally *contribute* to five of the ten.
No single class explains the lineage, and the two open residuals at head are one
of each severe kind: a repair-created leak and a late oracle.

### 5.2 Python MR-list — `7327e2a → 37f7427 → 38da898 → 4930d25 → b164046`

Reports: `95617c3c…` (Claude), `68400355…` (Fugu), `dc4f66ba…` (Claude),
`5c952a9a…` (Claude, `ACCEPT`). This is the only product surface that reached
`ACCEPT`, and therefore the most informative lineage in the corpus.

| ID | Finding | Evidence | Primary | Contributing |
|---|---|---|---|---|
| M1-MUST1 | Whole-snapshot authority capture reorders observable first-error precedence | `[R]` 10 multi-fault fixtures driven through the live socket **and** through the real pre-fix function imported from `7327e2a`; 8/10 diverge; controls agree, confirming mechanism not artefact | `REPAIR_INTRODUCED_REGRESSION` (relative to the pre-fix route) | `REQUIREMENT_OR_SCOPE_EXPANSION` — the contract demanded a pinned authority snapshot **and** preserved error precedence; the report itself records "the two requirements are in genuine tension… restoring interleaved reads would defeat the pinned snapshot". A contradictory acceptance criterion guarantees at least one cycle. `VACUOUS_OR_LATE_TEST_ORACLE` — `test_legacy_cross_project_and_error_precedence_parity` injects exactly one fault per run, so it passes while the property its name asserts is false |
| M1-MUST2 | A symlinked authority record yields a zero-byte reply where the pre-fix route returned `200 OK` | `[R]` `MR record is a symlink → <EMPTY WIRE>`; same on `/branches` and `/doctor` | `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` — the report states it plainly: "a shared-dispatcher gap, not an MR-list-only coding error… pre-existing on Doctor and branch-list and **newly reachable** on `/merge-requests`" | `REPAIR_INTRODUCED_REGRESSION` (new reachability with a `200 → empty` status regression). `REQUIREMENT_OR_SCOPE_EXPANSION` — the correction belongs outside the MR-list write set, so it cannot be closed in the lane that found it |
| M1-SHOULD1 | Escrow object-set digest recomputed per merge request | `[R]` slope table 1→80 MRs; `bytes_hashed` ≈ 12.5 KB per added MR while physical reads stay constant; target is *faster* than pre-fix at default budgets | `PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED` — the same diff newly *enforces* the 30 s deadline, converting a previously-slow-but-`200` listing into a typed `503` at a few hundred MRs | — |
| M1-SHOULD2 | The suite certifies precedence parity it does not test and never tests symlinked authority | `[R]` fixture reading at `tests…:1025-1123` | `VACUOUS_OR_LATE_TEST_ORACLE` | — |
| M1-COULD1 | `stable_project_id` charges from a pre-read `stat`, then re-opens the descriptor | `[R]` source reading; the double physical read is noted as pre-existing | `REPAIR_INTRODUCED_REGRESSION` (the charge/read divergence window is new) | — |
| M1-COULD2 | Deadline-poll density (1,496,274 polls at 200 MRs) | `[R]` | `PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED` | — |
| M2-MUST1 | Parent-directory filesystem failure recorded as absence; silently corrupts `stale` status | `[R]` live route with `branches/` replaced by a directory symlink: `200 OK` with `stale → open`, zero debt, `terminal = complete`. **`[P]` the fail-open arm `except OSError: return default, 0` on the parent open is present at the legacy baseline `7327e2a` and at `37f7427`** — it is *not* a gen-2 insertion | `INITIAL_REVIEW_SCOPE_MISS` — M1 read this exact function for MUST-2 and characterized it as "opens with `O_NOFOLLOW` and only maps `FileNotFoundError` to the default (`codefire@7327e2a:1454-1460`)", which `[P]` is provably incomplete: the broader parent-open arm sits ten lines earlier in the same function | `REPAIR_INTRODUCED_REGRESSION` — the gen-1 slice routed the consulted-branch read through `read_json_accounted`, so the pre-existing fail-open arm newly changes a legacy `stale` answer (`[R]` dual-route status comparison). `BOUND_FINDING_NOT_CLOSED` — the gen-2 ticket required one typed filesystem-authority response for every filesystem failure |
| M2-MUST2 | Type-only transport classification drops filesystem `TimeoutError` on GET and POST | `[R]` injected `TimeoutError(110, …, "/storage/authority.json")` from both dispatchers → `wire bytes = 0`. `[P]` `is_transport_os_error` is absent at `7327e2a` and `37f7427`, present at `38da898` | `REPAIR_INTRODUCED_REGRESSION` — proven | `PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED` — M1's required correction said "add an `except OSError` arm … that publishes a typed terminal diagnostic" and said nothing about *origin*; discriminating filesystem from transport by an exception class shared by both is the foreseeable error. `VACUOUS_OR_LATE_TEST_ORACLE` — the candidate's dispatcher mutation injects only generic `OSError(ELOOP)` and `ConnectionResetError` |
| M2-MUST3 | Cleanup failures swallowed after ownership is locally erased | `[R]` injected single `_sessions_retained` release failure → `200 OK`, `1296` wire bytes, `retained_bytes: 209` permanent, `terminal_reason = None`. **`[P]` at `37f7427`, `_release` is exactly: `setattr(self, attribute, 0)` … `try: self.ledger.release(...) except BaseException: pass`** | `INITIAL_REVIEW_SCOPE_MISS` — pre-existing in gen-1 code, inside M1's item-2 scope; M1's own words were "Closure to zero was confirmed independently on **every path this review exercised**", and no cleanup fault was among them | `VACUOUS_OR_LATE_TEST_ORACLE` — the candidate's cleanup test exercises budget/deadline/subject-change/validation failures and injects no cleanup fault. `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` — identical invariant to Upload `U3-H2` and MR-write `W2-F5` |
| M2-MUST4 | Mode and link-count changes after the final read's last metadata check pass the fence | `[R]` mode `0600→0644` and hardlink added from inside `_sha256_hexdigest_polling` → `200 OK`, `terminal = complete`. `[P]` `revalidate_json_authority_capture` is absent at `37f7427`, present at `38da898` — the metadata-reprove protocol is new | `REPAIR_INTRODUCED_REGRESSION` — an incomplete new control | `EVIDENCE_DOCUMENTATION_ONLY` — the same diff added `source-data-model-catalog.md:481-490` asserting the fence reproves mode and link count. `VACUOUS_OR_LATE_TEST_ORACLE` — existing tests mutate *before* `finalize_authority` begins |
| M2-SHOULD1 | Authority can change after all fences but before response preparation/publication | `[R]` MR record mutated from inside `prepare_json` → `200 OK`; **the report also records "The mutation was not observed to enter the already-built four-field projection"** | `INITIAL_REVIEW_SCOPE_MISS` — the fence-then-return-then-`send_json` order is gen-1 structure, and M1 asserted "the projection is prepared in full before the final fence and before publication" | `REQUIREMENT_OR_SCOPE_EXPANSION` — with no demonstrated semantic effect, this is blocking only because "Under this batch's zero-residual rule, this Should remains a blocker". `EVIDENCE_DOCUMENTATION_ONLY` — `source-architecture.md:727-731` says the fence is "before publication" |
| M2-COULD1 | Descriptor "read once" oracle measures one helper; whole-route reads are 5 | `[R]` instrumented `os.open` = 5, `read_json_accounted` = 1 | `VACUOUS_OR_LATE_TEST_ORACLE` (overclaimed oracle) | `EVIDENCE_DOCUMENTATION_ONLY` |
| M2-COULD2 | Poll mutation replaces a whole phase helper, so it proves helper ownership, not per-poll necessity | `[R]` `tests…:2388-2449` | `VACUOUS_OR_LATE_TEST_ORACLE` | — |
| M3-F1 | Five fallible operations run between the terminal fence and the first response byte | `[R]` instrumented `fcntl.flock`/`os.close`: `LOCK_UN` + 4 × `close` strictly after the publishable proof. `[T]` M2's own minimal closure demanded "the last authority/session/project fence immediately before the first response byte, **with no fallible or authority-dependent work between fence and publication**". `[P]` `_HTTPReadResponseSlot` / `require_publishable_state` are absent at `38da898` and present at `4930d25`. **`[R]` the gen-4 `ACCEPT` closes this item while recording "Five fallible teardown operations follow the fence"** — i.e. under the narrowed criterion "zero fallible operations *after the proof*" | `REQUIREMENT_OR_SCOPE_EXPANSION` — proven: the reviewer-authored criterion is unsatisfiable in place, because `handle_http_request_list` returns from inside `with remote_project_gate(...)` and Python must run the context-manager teardown; it was later replaced by a different, satisfiable criterion | `EVIDENCE_DOCUMENTATION_ONLY` — the blocking artifacts are three claims the candidate itself added (`codefire:13393`, `source-architecture.md:740`, `change-playbook.md:351`); the *authority-dependent* half of each claim is true. `VACUOUS_OR_LATE_TEST_ORACLE` — the candidate's ordering oracle instruments only named methods and can never see gate/lock teardown |
| M3-F2 | A real filesystem-origin `OSError` in that window is silently swallowed; the prepared `200` publishes | `[R]` injected `OSError(EIO)` at the production `fcntl.flock(fd, LOCK_UN)` → `200 OK`, `terminal_reason = complete`, `handler logs = []` | `REPAIR_INTRODUCED_REGRESSION` — the unconditional `read_slot.take()` publish after the `except` arms is new gen-3 code | `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` — `do_POST`'s `except BaseException` arm already closed `prepared` before re-preparing; the new read path was built without that shape, so GET and POST diverged inside one dispatcher |
| M3-F3 | `require_publishable_state` detects unaccounted work and the `200` publishes anyway | `[R]` injected residual owner → guard raises, `do_GET`'s `except CFError` computes a `400`, `take()` publishes the `200` with `4096` live retained bytes; `state_proved` has no production reader. **The report states honestly: "the reviewer could not exhibit a production input that leaks ownership past a successful `close()`"** and control probe P3 shows the legacy no-slot route behaves identically | `REPAIR_INTRODUCED_REGRESSION` — a new guard that cannot enforce its documented guarantee | `EVIDENCE_DOCUMENTATION_ONLY` — "a *dead safety net plus a false documented guarantee*, not a demonstrated wrong answer on clean inputs" (the report's own words). `REQUIREMENT_OR_SCOPE_EXPANSION` — blocking under zero-residual despite no production-reachable failure |
| M4 | `ACCEPT`; 0 blocking; 4 non-blocking observations | `[R]` §7 of `5c952a9a…`; observation 3 (GET/POST displacement asymmetry) is `[R]` "byte-identical at base and candidate" | — | `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` routed to a successor, correctly and explicitly |

MR-list is the corpus's natural experiment. Its four cycles decompose as:
cycle 1 caused by a **contradictory bound requirement** plus a single-fault
oracle; cycle 2 caused by **two pre-existing misses in shared readers** plus
**two over-broad new controls**; cycle 3 caused by an **unsatisfiable
reviewer-authored criterion** plus **three defects in the guard machinery
written to satisfy it**; cycle 4 accepted after the smallest delta in the
lineage. Nothing about cycles 2 and 3 required a longer checklist. Cycle 3 in
particular was caused by the *closure clause*, not by the candidate.

### 5.3 Python MR-write — `e2a218e → 21fe751 → 6dd4527`

Reports: `925cbcdb…` (Claude), `214e42fc…` (Fugu). Terminal: rejected;
successor blocked ≥20 h on the shared-primitives preflight.

| ID | Finding | Evidence | Primary | Contributing |
|---|---|---|---|---|
| W1-F1 | Material input-scaling decision work escapes the request ledger and deadline | `[R]` live measurement: hidden `local_v1` ledgers read **2.3×–3.4×** the object bytes the request charges, and the gap grows with input size (1x/3x/10x table) | `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` — every named escape is a shared helper (`validate_sealed_commit`, `collect_object_records`, `copy_object_graph`, `read_json`) | `INITIAL_REVIEW_SCOPE_MISS` relative to the root review's M3 |
| W1-F2 | Exact/−1/+1 boundaries do not bound the request | `[R]` | `VACUOUS_OR_LATE_TEST_ORACLE` | — |
| W1-F3/F4/F5/F6 | Legacy subject-less review turns a deterministic typed failure into a permanent `503` with escrow bytes retained; replay accepts and discards a fresh nonce; grow-tamper misreported; POST nonce timing inconsistent | `[R]` | `REPAIR_INTRODUCED_REGRESSION` (F3, F5 — in the gen-1 lane's new code) / `INITIAL_REVIEW_SCOPE_MISS` (F4, F6 — pre-existing legacy paths) | — |
| W2-F1 | Request accounting **still** excludes authority, identity, publication, canonicalization and projection work | `[R]` six further escape families enumerated with line cites (`remote_project_stable_id`, `read_remote_branch_record`/`read_remote_merge_request_path` → plain `read_json`, `peek_remote_publication_recovery_result`, publication preimage validation, three canonical encodings, the nonce postimage built before its charge) | `INITIAL_REVIEW_SCOPE_MISS` — W1 enumerated four escape mechanisms; W2 found at least six more on **pre-existing shared readers** in the same route | `BOUND_FINDING_NOT_CLOSED` (W1-F1 partially closed). `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` |
| W2-F2 | Apply rereads sealed commits outside the one sealed session per store | `[R]` with `object_records` limited to the measured exact value `10`, apply returned `200` while instrumentation observed `record_reads = 13`; the apply gap grows 9→13→27 across 1x/3x/10x. `[P]` `AccountedRemoteCommitBorrow` is absent at `e2a218e` and present at `21fe751` | `INITIAL_REVIEW_SCOPE_MISS` — present in the gen-1 candidate, unreported by W1 | `VACUOUS_OR_LATE_TEST_ORACLE` — proven, and unusually explicit: "The focused test **explicitly accepts** the mismatch" (`tests…:660`, `:667`) while the production docstring at `:608` claims every physically read record is charged exactly once |
| W2-F3 | Exact/−1/+1 and mutation non-vacuity remain incomplete | `[R]` `run_boundary` advertises exact/−1/+1 but "almost every caller passes only `(0, -1)`"; only response-body bytes exercise `+1` | `VACUOUS_OR_LATE_TEST_ORACLE` | `BOUND_FINDING_NOT_CLOSED` (W1-F2) |
| W2-F4 | The ledger is never prepublication-sealed and escrow copy sits outside reducer recovery | `[R]` direct probe: `ledger._prepublication_sealed == False` after effect start and an ordinary `apply` still succeeded; injected failure on the second `write_json` left one of four objects in the target store with **no pending marker and no transaction directory**. `[P]` `seal_prepublication` exists at `e2a218e` and is never called by the lane | `INITIAL_REVIEW_SCOPE_MISS` — a pre-existing, unused primitive in the gen-1 lane | `BOUND_FINDING_NOT_CLOSED` — W1-F3 "closed narrowly; zero-residual effect requirement not closed" |
| W2-F5 | Cleanup loses retry ownership and can suppress a durable response | `[R]` injected release failure on a lane retaining `37` bytes left debt `37` and the second `close()` marked the lane closed; live case: MR authority durably published and **no HTTP response reached the client** | `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` — third independent instance of erase-before-fallible-release | `INITIAL_REVIEW_SCOPE_MISS` — W1 injected no cleanup fault |
| W2-F6 | Replay nonce publication is durable but not atomic with replay confirmation | `[R]`; the disposition row reads "Closed narrowly; **replaced by** F6" | `REPAIR_INTRODUCED_REGRESSION` — the repair added persistence and the non-atomicity with it | `PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED` |
| W2-F7 | Canonical source documentation is stale | `[R]` | `EVIDENCE_DOCUMENTATION_ONLY` | `REQUIREMENT_OR_SCOPE_EXPANSION` — the report's own heading is "Low, **mandatory under zero-residual rules**" |

MR-write shows the corpus's clearest instance of a distinct recursion
mechanism, visible in the disposition table's own vocabulary: **"closed
narrowly … replaced by"** (F3→F4, F4→F6). The repair satisfies the literal
finding while the underlying invariant migrates to an adjacent boundary. That
is not omission and not regression; it is a finding-granularity defect.

### 5.4 Python Doctor + Branch — `775366e → 01a7c86 / fbef572 → 701469f`

Reports: `5d76f8a5…` (Fugu, `REQUEST_CHANGES`), `c7a1665f…` (Fugu, `ACCEPT`).
**One finding, one cycle, closed.** The cheapest lineage in the corpus.

| ID | Finding | Evidence | Primary | Contributing |
|---|---|---|---|---|
| D1-MUST1 | The canonical CLI test at `tests/test_codefire_cli.py:6273-6280` still asserts the pre-fix Branch behavior (`200 OK`, zero deadline polls); the completed target correctly returns the typed `503` | `[R]` the method fails at line 6279 on the target; the report classifies it explicitly: "This is a legitimate contract update… **It is not evidence of a product regression**" | `REQUIREMENT_OR_SCOPE_EXPANSION` — proven: "Because that file was outside the Branch write set, a separate corrective commit is mandatory before integration." The write set made a required contract update undeliverable inside the lane | — |
| D2 | `ACCEPT`; prior MUST-1 closed exactly | `[R]` | — | — |

This lineage is the control case. The finding was (a) singular, (b) precisely
localized to two assertions, (c) explicitly separated from product defect, and
(d) closed in one cycle. Every property that made it cheap is a *finding-shape*
property, not a checklist-depth property.

### 5.5 Rust File Doctor — `ea8f2a6 → 52a025e → 1a04490 → cd130e5`

Reports: `c1b39ffc…` (`ACCEPT`, 0 MUST / 4 SHOULD / 4 COULD),
`f7c52b05…` (`ACCEPT`, 0 MUST / 0 SHOULD / 3 NOTE). Generation 3 has **no
report**: review launch is `BLOCK`.

| ID | Finding | Evidence | Primary | Contributing |
|---|---|---|---|---|
| FD1-S1..S4, C1..C4 | Eight non-Must residuals on an accepted candidate; three "inherited from the previously accepted accounted implementation that the HTTP route already ships" | `[R]` | `REQUIREMENT_OR_SCOPE_EXPANSION` — an `ACCEPT` with zero Must findings nevertheless generates a mandatory successor generation under the zero-residual rule | `VACUOUS_OR_LATE_TEST_ORACLE` (S1: the production ledger's profile/kind/label asserted by no test; S2: no failure injection asserts "no semantic repository mutation") |
| FD2-NOTE1 | New production `panic!` on the failure-cleanup edge; release builds changed from silently continuing to aborting | `[R]` the reviewer's assessment: "Aborting is the safer of the two given the invariant, so I record this as a boundary the manager should be aware of **rather than a defect**" | `REPAIR_INTRODUCED_REGRESSION` in the literal sense (absent before, introduced by closing residual 8) | `PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED` — converting a `debug_assert!` into a release invariant necessarily changes release behavior; the ticket did not require that consequence to be premortemed |
| FD2-NOTE2 | One transient unreserved allocation per fenced record | `[R]` "This **mirrors the pre-existing accepted read path** rather than being introduced by this packet"; "No test can currently observe it" | `EVIDENCE_DOCUMENTATION_ONLY` | `INITIAL_REVIEW_SCOPE_MISS` (pre-existing, accepted twice) |
| FD2-NOTE3 | A document unreadable at scan time is not fenced | `[R]` "I read this as a **scope boundary** of a content-digest fence rather than a defect" | `REQUIREMENT_OR_SCOPE_EXPANSION` | — |
| FD3 (outside the 51: no report exists) | Generation 3 (`cd130e5`) review never launched | `[R]` per the non-review audit §B01: `cargo test … --locked live_file_doctor` returned **exit 0 in 0.267 s with 0 selected and 794 filtered tests** from a binary belonging to a *different* Rust worktree, although the candidate's own registration exists in its source; the release rerun compiled correctly, took `550.765 s`, and still covered only 16 of the required NOTE-1/NOTE-2/NOTE-3 cases | `VALIDATION_PROVENANCE_FALSE_GREEN` — the corpus's only proven instance in a review-relevant gate, and the reason the index records "No File Doctor product byte is integration-authorized" | `ATTRIBUTION_UNAVAILABLE` for any generation-3 finding: no report exists |

File Doctor is the corpus's proof that **`ACCEPT` does not terminate a
lineage.** Two consecutive `ACCEPT`s, zero Must findings ever, and three
generations — the third blocked on a false-green gate. Every one of the six
non-Must items that drove generation 3 was characterized by its own reviewer as
a scope boundary, an inherited property, or safer-than-before.

### 5.6 Rust HTTP MR create/review preflight — `3c23e38 → 71854e2 → 73bc532 → 92fc810`

Reports: `f30c22aa…` (Claude), `8bb1a418…` (Claude). Generation 3 has **no
report**: source-keyed profile evidence blocked.

| ID | Finding | Evidence | Primary | Contributing |
|---|---|---|---|---|
| H1-F1 | Moving `MergeRequestResponsePreflight::prepare` to *after* `source_records.publish_with_work_and_poll` — reintroducing the exact defect class the packet exists to close — leaves **all eight new tests green** | `[R]` mutation: 19/19 `merge_request` tests pass, 775/776 full target. **The report states: "The production change is sound. I found no product defect… The blocker is confined to the test evidence"** | `VACUOUS_OR_LATE_TEST_ORACLE` — proven by mutation | — |
| H1-F2..F5 | Four LOW/MEDIUM claim-accuracy, robustness and quality items | `[R]` | `EVIDENCE_DOCUMENTATION_ONLY` (F2, F5) / `REPAIR_INTRODUCED_REGRESSION` (F3) | — |
| H1-R1..R4 | Four **residual risks**, labelled by the reviewer "owned, not blocking", promoted into mandatory closure work by the repair packet §"Required residual-risk closure" | `[T]` `cfr168-rust-http-mr-preflight-review-closure-repair-20260730.md:119-162` | `REQUIREMENT_OR_SCOPE_EXPANSION` — proven: non-blocking residual risks became mandatory items, and the resulting repair was `+3,426/−560` `[P]` | — |
| H2-F-1 | The required 1x/3x/10x scaling oracle is vacuous: the loop adds *unreachable* source objects, so the escrowed closure and every counter are byte-identical at 1x, 3x and 10x; the doc comment claims the opposite of what the assertion says | `[R]` measured: `source_closure` stays 8 while the store grows 9→11→18; `object_records` 32, `aggregate_bytes` 13852, `response_bytes` 126, retained peak 41943 identical across all three. **`[P]` `merge_request_response_work_is_flat_while_object_work_scales` is absent at `71854e2`, present at `73bc532`, absent again at `92fc810`** | `REPAIR_INTRODUCED_REGRESSION` — proven, and specifically in the *evidence layer*: the repair wrote the vacuous oracle that blocked it | `VACUOUS_OR_LATE_TEST_ORACLE`. `BOUND_FINDING_NOT_CLOSED` — the packet's required evidence item was "exact 1x/3x/10x counter curves where work scales with object or response count" |
| H2-F-2 | The candidate's only durable evidence artifact (its commit message) states a review success body of 57 bytes; measured 60 | `[R]` | `EVIDENCE_DOCUMENTATION_ONLY` | `REQUIREMENT_OR_SCOPE_EXPANSION` — blocking because "this lane's only durable evidence artifact is the commit message"; the lane produced no result document |
| H2-F-3, F-4 | Phase-table comment claims a property the typed identity does not deliver; block-level `#[allow(dead_code)]` on 28 methods where the comment claims three | `[R]`; F-4 is explicitly "pre-existing, so this is hardening rather than a regression this candidate introduced" | `EVIDENCE_DOCUMENTATION_ONLY` | — |
| H2 verdict | **"All nine required closure items — F1–F5 and R1–R4 — are closed, and I found no product defect."** | `[R]` | — | — |

Rust HTTP is the corpus's decisive counter-example to the omission hypothesis.
**Both** of its cycles were blocked with an explicit statement that no product
defect exists. Two review cycles, two blocked launches, and `+4,829` changed
lines of repair were spent entirely on oracle quality and claim accuracy.

### 5.7 Rust Storage — `f5bce30/40c095d → b3bc32a → cdcb9bd`

Reports: `628a37f5…` (Fugu, F1–F9), `6438df44…` (Fugu, R1–R14 **and all F1–F9
`OPEN`**). Terminal: rejected; repair resume blocked.

| ID | Finding | Evidence | Primary | Contributing |
|---|---|---|---|---|
| S1-F1..F9 | Nine findings (4 High, 4 Medium, 1 release-blocking doc) from a lane that ran **no** build, test, or gate | `[R]` "this lane ran no Cargo, `rustc`, `rustfmt`, Clippy, test, build, install, or demo command… Validation was limited to non-building Git and text inspection" | `INITIAL_REVIEW_SCOPE_MISS` risk maximal by construction: no oracle existed to bound the finding set | — |
| S2-R1 | Local traversal counters do not represent the traversed graph: phantom `GraphNodes` on every EOF iteration, and **no** local child edge is ever charged | `[R]` source proof at `storage.rs:1883/2026/2039/3002`. `[P]` `StorageTraversalFrame` is absent at `b3bc32a` and appears 8× at `cdcb9bd` — the iterative traversal that miscounts was created by the F3 repair | `REPAIR_INTRODUCED_REGRESSION` — proven | `VACUOUS_OR_LATE_TEST_ORACLE` — "the focused boundary tests derive candidate values from the candidate snapshot and then replay those values. That proves internal self-consistency, not correctness of the graph oracle" |
| S2-R5 | Traversal container and authority owners remain outside the portable peak | `[R]`; same new traversal | `REPAIR_INTRODUCED_REGRESSION` | `BOUND_FINDING_NOT_CLOSED` (F1) |
| S2-R3, R4 | Publication ledger reports terminal `Complete` for failed publication; a post-write finish error can append a second JSON document | `[R]` the committed partial-publication test **asserts** `terminal_reason == Complete` while the external error is `PublishedOutputFailed`. `[P]` `write_all_tracked` is absent at `b3bc32a`, present at `cdcb9bd` | `REPAIR_INTRODUCED_REGRESSION` (the new tracked-publication path) | `BOUND_FINDING_NOT_CLOSED` (F8 survives in a narrower `Completed` case). `VACUOUS_OR_LATE_TEST_ORACLE` — a *committed test asserts the defect* |
| S2-R6 | Live storage owners allocated before the command ledger exists (CLI `--remote` clone and `PathBuf` before `prepare_storage_report`) | `[R]`; `[P]` `StorageReportOptions` present at both trees | `INITIAL_REVIEW_SCOPE_MISS` — pre-existing, inside S1-F6's own subject ("path-owner preallocation omits simultaneous `PathBuf` owners") | `BOUND_FINDING_NOT_CLOSED` |
| S2-R2, R7..R14 | Release-build precedence via `debug_assert!`; non-portable owner schema and path encoding; JSON field-count defect hidden by slack; blanket URL multiplier with a false 16 KiB proof; inexact path-owner lifetimes; non-exhaustive terminal classification; pinned reads not proving final path authority; publication tests missing required writer sequences; canonical docs overstating guarantees | `[R]` per-finding source proofs | mixed: `REPAIR_INTRODUCED_REGRESSION` (R7 generalized 256-byte shell, R8 projection), `INITIAL_REVIEW_SCOPE_MISS` (R2, R10, R12), `VACUOUS_OR_LATE_TEST_ORACLE` (R13), `EVIDENCE_DOCUMENTATION_ONLY` (R14) | `REQUIREMENT_OR_SCOPE_EXPANSION` for R14 ("Should fix, **release-blocking under zero-residual rules**") |
| S2 — the F1–F9 disposition itself (an audit-level observation, **not** one of the 51 counted rows) | All nine prior findings marked `OPEN` | `[R]` **"No F1-F9 item has an executable predecessor failure or a reviewer-performed temporary reversal in this rereview. The required minimum reversal matrix was therefore not independently satisfied."** The per-item reversal table shows `not run` in both the "executable predecessor failure" and "reviewer temporary reversal" columns for all nine | `REQUIREMENT_OR_SCOPE_EXPANSION` — proven and important: the disposition vocabulary has no state for *"not proven closed"*, so nine items whose closure the reviewer did not execute a reversal for were recorded with the same word (`OPEN`) as items with reproduced counterexamples | `VACUOUS_OR_LATE_TEST_ORACLE` |

Storage is the corpus's worst-conditioned lineage and the two facts are
connected `[I]`: an initial review with **no** executable oracle produced nine
findings whose closure the next review could not verify by reversal, so the
verdict collapsed to "all nine open" — while simultaneously producing 14 new
findings, six of them in the 4,007-line repair's own new code.

### 5.8 Rust File Ancestry — `fe54e60 → e93ebc2 → 02b0d63`

Report: `64a4831e…` (Claude). Generation 2 has **no report**: `≥13 h 04 m`
blocked on Rust build provenance (non-review audit §B01, §5.2).

| ID | Finding | Evidence | Primary | Contributing |
|---|---|---|---|---|
| A1-MUST1 | A pre-decision object effect remains on the live upload publish route, and the no-effect evidence compares against an already-mutated reference | `[R]` | `VACUOUS_OR_LATE_TEST_ORACLE` (the reference is already mutated) | `INITIAL_REVIEW_SCOPE_MISS` |
| A1-MUST2 | The ledger is decision-scoped, not command-scoped; duplicate unaccounted full traversals remain on every live route | `[R]`; the report's framing: "Review contract item 6 makes either of these `REQUEST_CHANGES` on its own, 'even if the new nucleus itself is correct' — and the new nucleus **is** correct. The blockers are **contract-scope and evidence-sufficiency defects**, not defects in the decision" | `REQUIREMENT_OR_SCOPE_EXPANSION` — the implementation was scoped to the decision; the contract's item 6 scoped acceptance to the command | `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` |
| A1-SHOULD1 | The deadline fence cannot fire in a production build, so the four-fence deadline evidence is vacuous with respect to shipped behavior | `[R]` | `VACUOUS_OR_LATE_TEST_ORACLE` | — |
| A1-SHOULD2..4, COULD1..2 | Incomplete fixture/dimension coverage; goal-directed claim holds only at query scope and its counters never reach a production caller; effect-bearing routes least covered by failure injection; two ledger-loss paths without terminal reason; residual-ownership error masks the typed outcome | `[R]` | `VACUOUS_OR_LATE_TEST_ORACLE` (S2, S3, S4) / `REPAIR_INTRODUCED_REGRESSION` (C1, C2) | — |
| A2 | Generation-2 findings | — | `ATTRIBUTION_UNAVAILABLE` — no report exists; the successor is `+3,488/−815` `[P]` and unreviewed | — |

### 5.9 KGI / KPI control plane — `0be1039 → bfa7f08 → 0ae583e`

Reports: `02a0d259…` (Claude, 35 bound findings: 14 MUST + 14 SHOULD + 7
COULD), `268ec0b7…` (Fugu, `REQUEST_CHANGES`: 15/35 closed, 20 partial,
R1–R7).

| ID | Finding | Evidence | Primary | Contributing |
|---|---|---|---|---|
| K1-F1 | The record contract has no semantic layer: **55 hand-crafted semantic mutations, 55 accepted, 0 rejected**; 15 control shape mutations correctly rejected | `[R]` | `VACUOUS_OR_LATE_TEST_ORACLE` — the strongest single instance in the corpus | — |
| K1-F2..F14, F-S1..S14, F-C1..C7 | 34 further findings on the same framework | `[R]` | mixed, dominated by `VACUOUS_OR_LATE_TEST_ORACLE` and `EVIDENCE_DOCUMENTATION_ONLY` | — |
| K2-R1..R7 | Terminal KGI and cohort decisions remain self-authorizable; numeric/temporal representations not order- or total-safe; evidence and repository paths shape-checked not verified; candidate/record/lifecycle graphs not validated as graphs; operational staleness and directory independence wrong; anti-gaming state declarative; malformed-but-schema-valid work unbounded | `[R]` "all 55 frozen semantic mutations produce their exact expected finding" **and** seven new blocker classes, none of which was in the 55; `[R]` 20 of 35 bound rows only partially closed; `[R]` R1's evidence is that the **new** `check_kgi` at `ai_contracts.py:2790` accepts any measured metric reference and `check_cohort_decision` at `:2854` counts member strings | `BOUND_FINDING_NOT_CLOSED` ×7 — the bound finding was "the record contract has **no semantic layer**"; a semantic layer that accepts seven decision-critical classes has not closed it. Chosen over `REPAIR_INTRODUCED_REGRESSION` because the 1,957-line validator is new code but the *accepting behavior* is unchanged from the predecessor, and over `INITIAL_REVIEW_SCOPE_MISS` because the initial finding was general, not enumerative | `INITIAL_REVIEW_SCOPE_MISS` ×7 — the initial review's evidence *was* a 55-case corpus, which under-specified the invariant. `VACUOUS_OR_LATE_TEST_ORACLE` ×7 — rule `E7`: an enumerated corpus, once frozen, becomes the specification, and passing it proves nothing about unenumerated classes |
| K2 context | "First repair candidate `0ae583e` **passes its official gates** but fails eight manager counterexamples" | `[T]` index row; `[R]` §4 of the rereview | `VALIDATION_PROVENANCE_FALSE_GREEN` in the weak sense — the gates are honest but not decision-grade; the index correctly instructs "do not use its currently green official gates as decision evidence" | — |

KGI/KPI is the purest demonstration of the **enumerated-corpus trap**: an
adversarial review that expresses its finding as *N* concrete counterexamples
converts the invariant into a list, and the repair optimizes the list. This is
the mechanism a fixed-size checklist would institutionalize — which is why §9
refuses to name a checklist size before the pilot.

---

## 6. Cross-lineage aggregation

### 6.1 Cause frequency over the 51 successor findings

Each finding contributes 1 to its primary cause and 1 to each proven
contributing cause; totals therefore exceed 51. Row-level evidence is `[P]`/
`[R]`/`[T]`; the aggregate is `[I]`.

The 51 rows are: Upload U2 (4) + U3 (2), MR-list M2 (7) + M3 (3) + M4 (0),
MR-write W2 (7), Doctor D2 (0), File Doctor FD2 (3), Rust HTTP H2 (4),
Storage S2 (14), KGI/KPI K2 (7). Primaries sum to exactly 51; the two
no-report rows (Ancestry gen 2, File Doctor gen 3) sit outside the 51 and are
counted separately.

| Causal class | Primary | Contributing | Total appearances | Share of the 51 as primary |
|---|---:|---:|---:|---:|
| `REPAIR_INTRODUCED_REGRESSION` | **16** | 1 | 17 | **31.4%** |
| `VACUOUS_OR_LATE_TEST_ORACLE` | 6 | 18 | **24** | 11.8% |
| `INITIAL_REVIEW_SCOPE_MISS` | **12** | 10 | 22 | **23.5%** |
| `BOUND_FINDING_NOT_CLOSED` | 8 | 14 | 22 | 15.7% |
| `EVIDENCE_DOCUMENTATION_ONLY` | 6 | 5 | 11 | 11.8% |
| `REQUIREMENT_OR_SCOPE_EXPANSION` | 2 | 6 | 8 | 3.9% |
| `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` | 1 | 4 | 5 | 2.0% |
| `PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED` | 0 | 4 | 4 | 0% |
| `VALIDATION_PROVENANCE_FALSE_GREEN` | 0 (1 outside the 51) | 0 | 1 | 0% |
| `ATTRIBUTION_UNAVAILABLE` | 0 (2 outside the 51) | 0 | 2 | 0% |

Read carefully, because the primary/contributing split matters:

- **No class is primary for more than a third of successor findings.** The
  largest, `REPAIR_INTRODUCED_REGRESSION`, is 16/51 (31.4%). There is no single
  root cause, and any intervention aimed at one class addresses at most about a
  third of the recurrence.
- **`VACUOUS_OR_LATE_TEST_ORACLE` appears in 47% of all successor findings**
  (24/51) once contributions are counted — the highest of any class — while
  being primary for only 12%. It is rarely the *sole* reason a cycle happened
  and very often a reason the cycle *could* happen.
- **`BOUND_FINDING_NOT_CLOSED` is primary for 8 (all but one of them the
  KGI/KPI control plane) and appears in 43%.** Product-side repairs do address
  what they are told to address; straightforward non-compliance is not the
  product problem.
- **`PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED` is never primary but appears 4
  times inside the 51** (`U2-B1`, `M2-MUST2`, `W2-F6`, `FD2-NOTE1`) and twice
  more on first-generation findings (`U1-H2`, `M1-SHOULD1`). Every instance is a
  *foreseeable* consequence of the prescribed repair: not exhaustively
  enumerating a flagged blast radius, widening an exception catch, adding nonce
  persistence, converting a `debug_assert!` into a release invariant, relocating
  a bound onto a second axis, and newly enforcing a deadline that turns latency
  into a `503`.
- **`VALIDATION_PROVENANCE_FALSE_GREEN` is essentially absent from review
  decisions.** Its one proven instance (File Doctor generation 3) blocked a
  *launch*, not an acceptance. This audit therefore does **not** support a
  claim that false-green validation is corrupting review verdicts today; it
  supports the non-review audit's claim that it is preventing them from being
  produced.
- **`SHARED_PRIMITIVE_OR_COMPOSITION_MISS` is under-counted by design**, at 5.
  Per §4 it is assigned only where a report itself identified the shared-helper
  scope. §6.3 shows the *family* recurrence is far broader; the two views must
  be read together, and §6.3 is the one that should drive I6.

### 6.2 Severity weighting

The reports use six incompatible severity vocabularies: `BLOCKER/HIGH/MEDIUM`,
`MUST/SHOULD/COULD`, `F1..Fn` with prose severity, `MUST-BLOCKER/SHOULD-BLOCKER`,
`NOTE`, and `R1..Rn` with `High/Medium`. Any weighted total is therefore an
`[I]` mapping, and I state the mapping rather than hiding it: blocking-with-
reproduced-production-effect = 3, blocking-by-policy-or-evidence-only = 2,
non-blocking = 1.

| Primary class | Weighted-3 | Weighted-2 | Weighted-1 | Weighted total |
|---|---:|---:|---:|---:|
| `REPAIR_INTRODUCED_REGRESSION` (16) | 7 | 8 | 1 | **38** |
| `INITIAL_REVIEW_SCOPE_MISS` (12) | 7 | 5 | 0 | **31** |
| `BOUND_FINDING_NOT_CLOSED` (8) | 2 | 6 | 0 | **18** |
| `VACUOUS_OR_LATE_TEST_ORACLE` (6) | **0** | 4 | 2 | **10** |
| `EVIDENCE_DOCUMENTATION_ONLY` (6) | **0** | 1 | 5 | **7** |
| `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` (1) | 1 | 0 | 0 | **3** |
| `REQUIREMENT_OR_SCOPE_EXPANSION` (2) | **0** | 1 | 1 | **3** |
| Total (51) | 17 | 25 | 9 | **110** |

The severity ranking inverts the frequency ranking, and this is the audit's
most decision-relevant quantitative result:

> **Oracle defects are the most *frequent* cause of a repeated cycle
> (24/51 appearances) but not one of the severe ones.** Of the 17 findings with
> a reproduced production effect, 14 are primary
> `REPAIR_INTRODUCED_REGRESSION` or `INITIAL_REVIEW_SCOPE_MISS`, one more is
> `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` (an ISM variant scoped to a shared
> helper), and the remaining two are control-plane `BOUND_FINDING_NOT_CLOSED`.
> **Zero** are primary `VACUOUS_OR_LATE_TEST_ORACLE`,
> `EVIDENCE_DOCUMENTATION_ONLY`, or `REQUIREMENT_OR_SCOPE_EXPANSION`. The Rust
> HTTP lineage states the same thing twice in its own words: "I found no
> product defect."

An intervention that only strengthens oracles will reduce cycle *count* while
leaving the severe classes untouched. An intervention that only front-loads
review depth will reduce misses (31 weighted points) while leaving the largest
severe class (38 weighted points) untouched, because those defects did not
exist when the initial review ran.

### 6.3 Invariant-family recurrence — the `MET-NRT-008` denominator the non-review audit needs

`[I]` derived by grouping every finding in the 21 reports by the invariant it
violates. This is the taxonomy the parallel audit's `IF-5` requires; §12
publishes it as a stable enumeration.

Note on counting: this table is a *family* view, so a finding that violates two
families is listed under both and the column does not sum to 143. §6.1's
class view assigns exactly one primary per finding and does sum. The two views
answer different questions and must not be added together.

| Invariant family | Surfaces where independently rediscovered | Distinct findings |
|---|---|---:|
| **Request ledger / deadline ownership** | Upload (U1-B1, U2-B1), MR-list (M1 item 1), MR-write (W1-F1, W2-F1), Doctor (root M2), Ancestry (A1-MUST2), Storage (S2-R6), root M1/M3 | 9 |
| **Retryable release / owner state erased before fallible release** | **Upload U3-H2, MR-list M2-MUST3, MR-write W2-F5**, Storage R-cleanup, Rust HTTP R1 | 5 |
| **Charge / reserve before allocation** | Upload U1-H3, U1-M4, U2-H2, MR-list M1 item 2, MR-write W2-F1, Storage S1-F6/R5/R6, File Doctor NOTE-2 | 9 |
| **Authority fence completeness and final-fence placement** | MR-list M2-MUST4, M2-SHOULD1, M3-F1, MR-write W1-F5/W2-F4, Storage R12, Upload U1-B1 | 7 |
| **Prepared response / response-owner displacement** | MR-list M3-F2, M3-F3, M4-obs3, MR-write W2-F5, Rust HTTP H1-F1/F3/R1/R2, Storage R3/R4 | 10 |
| **Publication transaction / durable recovery** | MR-write W2-F4/F6, Storage R3/R4, Upload U2-H3 | 5 |
| **Error-origin classification / typed diagnostics** | MR-list M1-MUST2, M2-MUST1, M2-MUST2, M3-F2, Storage R11, MR-write W1-F3 | 6 |
| **Evidence non-vacuity** | every lineage; 32 appearances (§6.1) | 32 |
| **Allocation / counter accounting exactness** | Upload U1-H2, Storage R1/R7/R8/R9/R10, Rust HTTP H2-F-1, MR-write W2-F2 | 8 |

Two rows are decisive.

- **Erase-before-fallible-release recurs on three independent surfaces with
  three independent reviewers**, and `[P]` I confirmed the *identical* code
  shape on two of them from the object database (`MR-list _release` at
  `37f7427`; `_AccountedUploadSourceMemory.close` at `76baff8`). This is a
  shared-primitive defect that was repaired three times surface-locally.
- **Prepared-response displacement recurs 10 times across Python GET, Python
  POST and Rust**, and the MR-list `ACCEPT` report explicitly routes it forward
  rather than claiming closure.

The shared-primitives preflight packet exists to stop exactly this, and it is
`[T]` blocked behind an accepted Upload identity — i.e. behind the surface
whose recurrence it exists to prevent. The non-review audit reaches the same
conclusion from the scheduling side (§B02). **This audit answers its §11
question 4: yes, the preflight may bind *rejecting* reports as specifications**
— see §11.

### 6.4 Review-method variance across the 21 reports

`[R]` from each report's own method section. This is the variable the corpus
most under-controls.

All counts `[P]` by grep over the 21 reports.

| Method property | With it | Without |
|---|---:|---:|
| Executed at least one gate on the exact candidate (`unittest` / `py_compile` / `cargo test` / a `check_*.py`) | 20 | **1** — Storage initial: "no Cargo, rustc… non-building Git and text inspection" |
| Ran deliberate regressions / mutations on production code | 11 | **10** |
| Authored reviewer-side production probes | 11 | 10 |
| Recomputed candidate identity in a dedicated section | 4 | 17 (11 recompute at least one declared digest somewhere) |
| Independently re-derived the *predecessor's* behavior from the predecessor tree | **3** | 18 |
| Published an explicit "what I did not verify" section | 10 | 11 |

The two most consequential rows:

- **Only 3 of 21 reports independently re-derived predecessor behavior**, and
  the three are MR-list M1, MR-list M3 and Upload U3 `[P]`. MR-list M1 states
  the method explicitly: findings were verified "against the **actual** pre-fix
  `handle_http_request_list` loaded from baseline `7327e2a…`, not against the
  test file's reimplementation". Upload U3 did it structurally, with a
  "detached predecessor scratch" at `76baff8`, and every one of its decisive
  results is a candidate-versus-predecessor row. Those three reports sit on the
  only converging product lineage (MR-list, ×2) and on the Upload generation
  that closed the most inherited findings (2 of 4). `[I]` This is the strongest
  method-to-outcome correlation in the corpus — and with *n*=2 lineages it is a
  hypothesis for §9's pilot, not a finding.
- **Mutation/reversal execution splits the corpus almost exactly in half
  (11/10)**, and the half without it contains both lineages with the largest
  unbounded finding sets (Storage: 9 then 14; Ancestry: 8 with no successor
  review). `[I]` Correlational; both are also the most build-blocked surfaces,
  which is a plausible common cause the pilot must control for.

Provider assignment shows no signal `[I]`: Claude authored 12 reports and Fugu
9; `ACCEPT` came from Claude twice and Fugu twice; both providers produced
reports that later reviewers found had missed pre-existing defects (Claude:
Upload U1, MR-list M1; Fugu: Upload U2, Storage S1). **Reviewer independence
and model selection are not implicated by this corpus.**

### 6.5 Review precision — the false-positive question

`[P]` A grep over all 21 reports for withdrawal language found exactly **four**
instances in 143 findings:

| Instance | Nature |
|---|---|
| Upload U1 reverses earlier item-4 `FAIL` → `PASS` | Correct: the 512 MiB override the FAIL rested on was removed |
| Upload U1 reverses earlier item-6 `FAIL` → resolved | Same basis |
| Upload U2 rejects its own earlier probe conclusion: "The earlier `f1_proof.py` line that printed an accepted expired proof **is not evidence of a defect**: that script advanced its clock before constructing the deadline" | A genuine reviewer probe error, self-caught within the same report |
| MR-list M1 records a prior harness note as "Not reproduced" | Prior report artefact |
| File Doctor NOTE-2 | Reviewer explicitly declines to call a pre-existing property a defect |

**The reviews are high-precision and low-recall-per-pass.** ≤3% of findings
were later withdrawn, and no finding in the corpus was accepted-then-shown-
wrong on the product. This reframes the whole problem: the system does not
suffer from reviewers inventing work. It suffers from each pass seeing a
different true subset, and from each repair enlarging the surface the next pass
must see.

---

## 7. Quantification — what the telemetry supports and what it does not

Per the parallel audit's `IF-2`, a review-duration claim must cite
`MET-NRT-002` before attributing delay to review. That metric does not exist
yet, so this section attributes almost nothing to review duration.

### 7.1 Supported

| Quantity | Value | Marker |
|---|---:|---|
| Review cycles to a terminal accepted candidate, per surface | MR-list 4, Doctor+Branch 2, Upload ≥4 (open), MR-write ≥2 (open), Storage ≥2 (open), Rust HTTP ≥3 (blocked), File Doctor ≥3 (blocked), Ancestry ≥2 (unreviewed), KGI/KPI ≥2 (open) | `[P]` |
| Mean indexed decisions per product surface | 2.1 (19 product reports / 9 surfaces) | `[P]` |
| Reports that are second-or-later on their surface | 11 of 21 (52.4%) | `[P]` |
| Successor findings attributable to code the repair created (primary) | **16 of 51 (31.4%)** | `[I]` over `[P]` rows |
| Successor findings with a proven oracle defect (any role) | **24 of 51 (47.1%)** | `[I]` over `[R]` rows |
| Successor findings with a reproduced production effect | **17 of 51 (33.3%)** | `[I]` over `[R]` rows |
| Successor findings blocking with **no reproduced production defect** | **34 of 51 (66.7%)** | `[I]`; includes all four Rust HTTP items, MR-list M2-SHOULD1 / M3-F1 / M3-F3, all three File Doctor NOTEs, MR-write W2-F7, Storage R5–R14 |
| Repair delta size, median across the 14 successor-generation deltas | 2,180 insertions | `[P]` |
| Ratio of largest repair to the finding count it answers (Storage) | 4,007 lines / 9 findings | `[P]` |

### 7.2 Cycle-time impact

| Quantity | Value | Marker |
|---|---|---|
| Candidate-ready → reviewer start, unblocked | 84.7 s, 105.6 s, "under five minutes" | `[R]` non-review audit §5.2; **review handoff is not a bottleneck** |
| Rejecting report → exact implementer session resumes | 600 s (MR-list), 540 s (Upload) | `[R]` `MET-NRT-012`; **review rollover is not a bottleneck** |
| Implementation service time per generation | 4,019.063 s / 3,164.042 s / 6,808.307 s measured; ≈1 h 05 m and ≈1 h 09 m estimated | `[R]` |
| **Provider service time spent on generations that exist only because an earlier generation was rejected** | 18 of 22 generations; at the measured median ≈1–2 h each, `[I]` **≈18–36 h of provider service** | `[I]` over `[R]` |
| Review lead time (`MET-FLOW-006`) | **`[U]`** — defined at `development-flow-kgi-kpi.md:56`, no recorded value | `[P]` |
| Reviewer correction rate (`MET-FLOW-015`) | **`[U]`** — defined at `:70`, no recorded value; the ledger's own 19:10 experiment target was "at most 2 of 5 packets" | `[P]`/`[T]` |
| Rework seconds (`MET-FLOW-014`) | **`[U]`** | `[P]` |
| Terminalization lag on review lanes | ≥1,200 s on Upload; Storage `[U]` but 135 post-verdict events | `[R]` non-review audit §B03 |
| Time cost of the four Fugu reviews lost to model-list incompatibility | `[U]` | `[R]` |
| Any interval crossing 2026-07-30T20:55 → 07-31T01:37 | **`[U]`** — wall-clock discontinuity | `[R]` |

### 7.3 Explicitly not quantified

- **Severity-weighted defect escape.** Zero deltas are integrated `[P]`, so
  there is no post-integration escape denominator. `KGI-DEV-002` cannot be
  scored for or against the review process by this corpus.
- **Cost per review cycle.** Fugu cost is `[U]`; cross-provider totals are
  incomplete, so no review-cost-per-finding figure is offered.
- **Whether more review depth would have prevented cycle *N*.** Not
  determinable from observational data. §9 exists because this is the question
  a pilot must answer.

---

## 8. Dominance determination

The prompt asks whether review method, repair quality/communication, repair
strategy, test quality, validation infrastructure, scope change, or
**interaction** dominates. The evidence answers this without ambiguity.

**Interaction dominates, and the interaction is specific and nameable.**

Not one of the seven candidates dominates alone:

| Candidate | Best case for it | Why it does not dominate |
|---|---|---|
| Review method (depth/omission) | 12/51 primary misses, incl. two proven by object-database contradiction; 31 weighted points | 16/51 successor findings are in code that did not exist at review time; deeper review of generation *N* cannot see generation *N+1*'s code |
| Repair quality | 16/51 primary regressions and the highest weighted total (38); 7 of the 17 reproduced-effect findings | Two lineages (Rust HTTP, File Doctor) recurred with **no product defect at all** |
| Repair communication | Real: the Rust HTTP WIP rewrite against an additive-only packet; the 57-vs-60-byte commit-message claim as the lane's only durable artifact; the implementer restating the ticket's "before allocation" as "before the sort" | Explains ~4 findings |
| Repair strategy | Strong: median successor delta 2,180 insertions; the only two accepted-and-terminal generations are the two smallest in the corpus (`+813/−122`; `+17/−2`) | Not a cause of the initial misses |
| Test quality | 24/51 appearances, the highest of any class | **Zero** oracle findings were reproduced production defects; alone it explains cycle count, not cycle severity |
| Validation infrastructure | The File Doctor `0 selected / 794 filtered` false green is real and severe | Zero appearances inside the 51; it currently blocks launches rather than corrupting verdicts |
| Scope change | 8 appearances, 2 primary, but two entire cycles (Rust HTTP gen 2, File Doctor gen 3) exist because non-blocking items became mandatory | Not a cause of the severe classes; low count, high leverage |

The dominant structure is a **three-step loop**, and every step is proven:

1. **A review pass has bounded recall on a shared, monolithic surface.** The
   surfaces are `codefire` (33,455 lines), `http.rs` (18,639), `main.rs`
   (12,647). Every initial review that later turned out to have missed
   something had *examined the containing function*: MR-list M1 read
   `read_json_accounted` and mis-stated its error mapping `[P]`; MR-list M1
   verified closure-to-zero on "every path this review exercised" `[P]`;
   Upload U1 enumerated 27 `validate_sealed_commit` call sites and missed the
   one that is upload-reachable `[P]`. This is not carelessness; it is what
   bounded recall looks like on a 33k-line file.

2. **The repair enlarges the reviewed surface, usually by more than the finding
   requires.** `[P]` Median repair 2,132 lines. New public APIs
   (`_AccountedUploadSourceMemory`), new state machines
   (`RemoteJSONPublicationRootProof`), new dispatch protocols
   (`_HTTPReadResponseSlot`, `route_filesystem_boundary`), new classifiers
   (`is_transport_os_error`), new traversals (`StorageTraversalFrame`), new
   publication paths (`write_all_tracked`) — **all six of those symbols are
   absent from their predecessor tree and each is the direct locus of at least
   one next-cycle blocking finding** `[P]`. The enlargement is partly caused by
   step 3.

3. **The acceptance criterion is set by the previous report's prose and by a
   zero-residual policy, so it expands monotonically and sometimes becomes
   unsatisfiable.** `[T]`/`[R]` Rust HTTP's four non-blocking residual risks
   became mandatory closure work and produced a 3,426-line repair that wrote
   its own vacuous oracle. MR-list M2's closure clause demanded "no fallible
   work between fence and publication", which the language's context-manager
   teardown makes impossible in place; the gen-4 `ACCEPT` closed the item under
   a **different, narrowed** criterion while recording that five fallible ops
   still follow the fence. Storage's disposition vocabulary had no state for
   "not proven closed", so nine items were marked `OPEN` with `not run` in
   every reversal column.

The loop closes because step 2's new surface has never been reviewed and step
3 guarantees the next review has more to prove. **Oracle vacuity is the
lubricant** — 24 of 51 successor findings (47%), the most frequent class, and
what lets each generation ship believing it closed the invariant — but it is
not the engine. The engine is surface growth under a monotonically expanding
criterion.

Two corollaries follow directly, and both cut against the intuitive fixes:

- **A longer initial checklist attacks the smaller of the two severe classes.**
  It cannot touch the 16 repair-introduced primaries (38 weighted points, the
  largest single block), and it lengthens step 3.
- **Making `ACCEPT` stricter makes the loop worse.** File Doctor got two
  `ACCEPT`s with zero Must findings and still needed a third generation `[P]`.

---

## 9. Candidate interventions, each with its causal class and a retroactive pilot

**No intervention below is adopted by this audit, and none names a checklist
size, template depth, or item count.** Each states the causal class it
addresses, the mechanism, the cost, the risk, and a *retroactive pilot on
historical candidates* — because the corpus is unusually well suited to
retroactive measurement: 22 candidate commits are in the object database `[P]`
and 21 reports are immutable with verified digests `[P]`, so a proposed control
can be replayed against known ground truth without spending a provider cycle
on a live lane.

### 9.0 The common pilot instrument

All pilots share one instrument, which must be built first because every
measurement below depends on it.

**The historical replay corpus.** For each of the 9 lineages, the tuple
(predecessor tree, repair ticket, implementer plan/result, candidate tree,
successor report, per-finding classification from §5). Ground truth for a
retroactive pilot is: *the set of findings the next report actually raised,
partitioned by §5's classification.* A control is scored on whether it would
have surfaced each finding **before** the candidate was frozen.

Required outcome measures, identical across pilots so results are comparable:

| Measure | Definition | Why |
|---|---|---|
| **Later-finding capture** | later findings the control would have surfaced pre-freeze / later findings in that class | the primary benefit |
| **Noise** | control outputs that match no later finding and no §5 finding | the primary cost |
| **Duplication** | control outputs already covered by an existing bound finding or ticket clause | measures whether the control adds anything |
| **Author time** | added implementer work, measured as the delta in required evidence artifacts, not as a guess at seconds | cost to the lane |
| **Reviewer time** | added reviewer obligations, same basis | cost to the review |
| **False positives** | control outputs asserting a defect that the object database or an executed reproducer refutes | credibility cost |

Every pilot must publish its denominators and its `unavailable` count, per the
existing anti-gaming rules and `MET-NRT` rule 1.

**Baseline to beat.** The ledger's own 2026-07-30 19:10 experiment already
set one: "material reviewer correction — at most 2 of 5 packets" `[T]`. The
observed corpus rate is 17/21 `[P]`. A control that does not move that ratio on
replay should not be adopted live.

---

### I1 — Predecessor-behavior re-derivation as a review obligation

**Causal class addressed:** `INITIAL_REVIEW_SCOPE_MISS` (primary — 12/51, 31
weighted points), `REPAIR_INTRODUCED_REGRESSION` (secondary — a behavioral diff
catches new regressions too), `VACUOUS_OR_LATE_TEST_ORACLE` (a candidate test
that reimplements the predecessor cannot be a parity oracle).

**Mechanism.** A review that asserts behavioral parity with a predecessor must
derive the predecessor's behavior from the predecessor tree, not from the
candidate's own reimplementation of it. `[P]` Only 3 of 21 reports did this
(MR-list M1, MR-list M3, Upload U3), and they sit on the only converging
product lineage and on the Upload generation that closed the most inherited
findings.

**Cost/risk.** Low cost — the predecessor tree is always available `[P]`. Risk:
on Rust surfaces it requires a build of the predecessor, which collides with
the non-review audit's `B01`; the pilot must therefore run Python-first.

**Retroactive pilot.** For the 6 lineages with ≥2 reports, replay: import each
predecessor's implicated function from its exact tree and run the successor
report's own reproducers against both. Ground truth is the 17
`INITIAL_REVIEW_SCOPE_MISS` appearances. Capture is measured directly.
Specifically decidable cases: would loading `read_json_accounted` from
`7327e2a` and enumerating its `except` arms have surfaced MR-list M2-MUST1
(`[P]` yes — I did exactly this in one command); would importing
`_AccountedUploadSourceMemory.close` from `76baff8` and fault-injecting
`ledger.release` have surfaced Upload U3-H2 (`[P]` the ordering is visible by
reading; the leak needs one injection).

### I2 — Diff-scoped re-review of repair-introduced surface

**Causal class addressed:** `REPAIR_INTRODUCED_REGRESSION` (primary — the
largest class by both count (16/51) and severity weight (38)),
`PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED`.

**Mechanism.** Treat every symbol that is **absent from the predecessor tree
and present in the candidate** as unreviewed surface with its own review
obligation, separate from finding closure. `[P]` Six such symbols account for
at least one blocking next-cycle finding each. The set is mechanically
computable — I computed it for six lineages with `git show <tree>:<path> | grep
-c <symbol>` — so this is a checkable precondition, not a judgement call.

**Cost/risk.** Medium: it adds review obligations exactly where repairs are
largest, so it interacts with I3. Risk: it could double the review surface of a
large repair; I3 is the control that prevents that.

**Retroactive pilot.** For each of the 14 measured successor deltas, compute the
new-symbol set and score what fraction of the next report's findings land
inside it. `[P]` Preliminary, from the six sets I already computed: Upload gen 3
→ 2 of 4 findings inside the new-symbol set; MR-list gen 2 → 2 of 7; MR-list
gen 3 → 3 of 3; Storage gen 2 → ≥4 of 14; Rust HTTP gen 2 → 1 of 4 (the
vacuous oracle is itself a new symbol). Duplication and noise are then scored
against the same sets. If capture on new-symbol surface is high while noise is
low, the obligation is cheap; if the new-symbol set is routinely thousands of
lines, the finding is that **I3 must precede I2**.

### I3 — Bounded repair delta with an explicit new-public-surface refusal

**Causal class addressed:** `REPAIR_INTRODUCED_REGRESSION`,
`REQUIREMENT_OR_SCOPE_EXPANSION`, and step 2 of §8's loop.

**Mechanism.** A repair packet declares a delta budget and an explicit "no new
public API, no new state machine, no new classifier unless a bound finding
requires it" clause. `[P]` The correlation this targets: accepted generations
were the two smallest (`+813/−122`; `+17/−2`), while the four largest
repairs (3,426 / 3,488 / 4,007 / 4,011 lines) produced 4, unreviewed, 14 and 7
next-cycle findings respectively.

**Cost/risk.** Low cost to write; real risk of *fragmenting* a repair that
genuinely needs one coherent change, and of pushing work into unreviewed
successors. That risk is why the pilot must measure generations-to-terminal,
not just findings-per-generation.

**Retroactive pilot.** Partition the 14 successor deltas at a set of candidate budget
thresholds (do **not** fix one in advance; sweep and report the curve).
Score, per threshold: how many next-cycle findings lay in surface a budget
would have excluded; how many bound findings could not have been closed within
it (Upload U2-H2 is the key test — the ticket asked for a memory owner at one
line and the delivered public API was ≈10× that); and whether the excluded work
appears later as a successor packet anyway. Report the curve and let the
manager pick the point. **This is the pilot that must run first**, because I2's
cost and the entire template question depend on its answer.

### I4 — Oracle admissibility rules, mechanically checked

**Causal class addressed:** `VACUOUS_OR_LATE_TEST_ORACLE` (24 appearances, the
most frequent class; note §6.2 — it carries only 10 weighted points as a
primary, so this intervention should be expected to cut cycle *count*, not
cycle severity).

**Mechanism.** An oracle offered as closure evidence must satisfy checkable
admissibility conditions drawn from the corpus's own failure modes, each of
which is a proven historical instance rather than a hypothesis:

| Admissibility condition | Historical instance it would have caught |
|---|---|
| The asserted axis must observably move | Rust HTTP H2-F-1 `[P]` (`source_closure` constant at 1x/3x/10x) |
| The assertion must fire at or before the boundary it names, not after | Upload U3-M4 `[P]` (`tests…:3002` fires after `list(self.documents)`) |
| A test name/docstring claim must be entailed by its assertions | MR-list M1-SHOULD2, M2-COULD1, MR-write W2-F2, Rust HTTP H2-F-1 |
| Deleting the production line must change a result | Upload U2-M4 `[P]` (22/22 and 33/33 survive both mutations) |
| Reversal must be behavioral, not symbol-absence | Upload U2 `[R]`, Storage S2 `[R]` |
| A frozen counterexample corpus is not the invariant | KGI/KPI K2 `[R]` (55/55 closed, 7 new classes) |
| Multi-fault / multi-instance fixtures where the property is about ordering | MR-list M1-MUST1 `[R]` |
| A zero-selected or foreign-artifact gate is never evidence | File Doctor FD3 `[R]` (`0 selected / 794 filtered`) |

**Cost/risk.** Low to medium. Real risk: this list is itself an enumerated
corpus and could become the next KGI/KPI trap — a repair that satisfies eight
conditions and nothing more. The mitigation is structural: the conditions must
be stated as *properties of the oracle*, and the pilot must include a
held-out set (§9.1).

**Retroactive pilot.** Apply each condition to every oracle cited as closure
evidence in the 21 reports. `[P]` decidable retroactively for at least the 11
instances above. Score capture per condition, noise (admissible oracles the
condition would have rejected), and duplication. **Report per-condition
capture, not a total**, so the manager can see which conditions carry the
signal — this is the measurement that must precede any decision about template
depth.

### I5 — Separate the verdict vocabulary from the evidence vocabulary

**Causal class addressed:** `REQUIREMENT_OR_SCOPE_EXPANSION` (8 appearances but
2 entire cycles), `BOUND_FINDING_NOT_CLOSED` (22 appearances; it currently
absorbs three distinct states), `EVIDENCE_DOCUMENTATION_ONLY` (11).

**Mechanism.** The corpus proves the current vocabulary conflates states that
have different next actions:

| Distinct state | Proven instance | Currently recorded as |
|---|---|---|
| reproduced production defect | Upload U3-H2 `[R]` | `OPEN` |
| closed in behavior, closure not independently proven | Storage F1–F9 `[R]` ("not run" in every reversal column) | `OPEN` |
| behavior correct, claim false | Rust HTTP H2-F-2 `[R]` (57 vs 60 bytes) | blocking finding |
| behavior correct, oracle vacuous | Rust HTTP H1-F1, H2-F-1 `[R]` ("I found no product defect") | blocking finding |
| defect exists but is not production-reachable without injection | MR-list M3-F3 `[R]` | blocking finding |
| scope boundary, reviewer declines to call it a defect | File Doctor NOTE-2/3 `[R]` | mandatory successor |
| requirement unsatisfiable as written | MR-list M3-F1 `[P]`/`[R]` | blocking finding |

**Cost/risk.** Low, documentation-only. Risk — and it is a serious one — that
adding an "unproven closure" state becomes a route to accept unproven work.
Mitigation: the new states change *routing and next action*, never integration
authority; `ACCEPT_WITH_NOTES` stays forbidden.

**Retroactive pilot.** Re-label all 143 findings under the extended vocabulary
and recompute how many mandatory successor generations each lineage would have
carried. `[I]` decidable retroactively from the reports alone. The measurement
that matters: how many of the 22 generations were driven **only** by states
that the extended vocabulary would route rather than block. My §5 reading says
at least Rust HTTP gen 2, File Doctor gen 3, and part of MR-list gen 3 — but
that is exactly the number the pilot must produce rather than accept from me.

### I6 — Shared-primitive contracts specified from rejecting reports

**Causal class addressed:** `SHARED_PRIMITIVE_OR_COMPOSITION_MISS` (primary),
`INITIAL_REVIEW_SCOPE_MISS` (a family preflight raises recall on every
consumer at once).

**Mechanism.** Publish the §12 invariant-family enumeration and let the
shared-primitives preflight bind **rejecting** reports as specifications,
removing its accepted-Upload precondition. `[P]`/`[R]` The erase-before-release
family was independently rediscovered on three surfaces and repaired
surface-locally three times; the preflight designed to prevent that is `[T]`
scheduled behind one of the three.

**Cost/risk.** Low; report-only, writes no product byte. Risk: ossifying a
contract around Upload's still-open HIGH-2. Mitigation: cite each primitive by
report SHA-256 and mark any primitive whose final shape depends on an open
residual. This is the non-review audit's Wave 2.1 and this audit's answer to
its question 4 (§11).

**Retroactive pilot.** For each of the 9 families, check whether a contract
derived **only** from reports available at the time would have pre-empted the
later surface's finding. `[P]` The erase-before-release case is decidable now:
MR-list M2-MUST3 (2026-07-30) precedes both MR-write W2-F5 and Upload U3-H2, so
a family contract published after MR-list gen 2 would have been available to
both later repairs. Score capture per family and, critically, **noise**: how
many contract clauses would have been irrelevant to a given consumer.

### I7 — Repair-hazard premortem bound to the repair mechanism

**Causal class addressed:** `PREDICTABLE_REPAIR_HAZARD_NOT_PREMORTEMED` (9
appearances, never primary — which is why this is a low-priority intervention
despite being cheap).

**Mechanism.** When a ticket prescribes a mechanism whose hazard class is known
from the corpus — relocate a bound, widen a catch, add a state machine, promote
an assertion to a release invariant, add persistence, newly enforce a limit —
the packet must name the hazard and require one falsification of it.

**Cost/risk.** Trivial to write. Risk: hazard lists grow without bound; must be
derived from observed instances only.

**Retroactive pilot.** Six historical mechanism→hazard pairs are already
proven (§6.1). Score whether naming the hazard in the ticket would have
produced the missing oracle, using the implementer result documents as the test
of whether a named hazard actually changes delivered evidence — `[T]` the Upload
gen-4 result document is the decisive case, because the hazard *was* named in
the ticket ("acquisition-before-allocation") and the implementer still
delivered "before the sort".

**That last fact is the most important caveat in this section.** One historical
case shows a ticket clause being restated into a weaker form by the
implementer. Any intervention that works by adding ticket clauses must be
piloted for exactly that failure mode, which is why §9.1 exists and why no
template is proposed here.

### 9.1 Pilot sequencing, and the explicit refusal to fix a template now

Sequencing follows from §8: attack the loop's engine before its lubricant.

| Order | Pilot | Rationale |
|---:|---|---|
| 1 | **I3** delta-budget sweep | Everything else's cost depends on repair size. Cheapest to replay: 14 diffs, all `[P]` available. |
| 2 | **I1** predecessor re-derivation (Python lineages only) | Highest-confidence method correlation; Rust blocked on `B01`. |
| 3 | **I4** oracle admissibility, **per-condition** | Highest frequency; must be reported per condition, not as a total. |
| 4 | **I5** vocabulary re-labelling | Pure re-analysis; no lane cost. |
| 5 | **I6** family contracts | Unblocks MR-write; needs §12 published first. |
| 6 | **I2** new-symbol re-review | Cost depends on I3's answer. |
| 7 | **I7** hazard premortem | Lowest leverage; run last and only if I4 shows ticket clauses change delivered evidence. |

**Held-out set, mandatory.** Because six of the seven controls are derived from
the same 21 reports they will be scored against, any capture figure is
optimistically biased. Two lineages — chosen by the manager, not by this audit,
and one of them Rust — must be withheld from control derivation and used only
for scoring. Without a held-out set the pilot will reproduce the KGI/KPI trap:
a control that perfectly explains its own training corpus.

**Why no template depth is named.** The three quantities that would determine
it are all currently `[U]`: per-condition oracle capture (I4), the
delta-budget/capture curve (I3), and the fraction of generations driven only by
routable states (I5). Naming a count now would be the same error as the frozen
55-mutation corpus: it would convert an invariant into a list, and the next
repair would optimize the list. The template's depth and structure are outputs
of pilots 1–4, and the pilot report should state the count it derived **and its
denominator**.

---

## 10. Immediate changes for active lanes

These follow from findings already proven above and require no pilot. None
changes a verdict, integration authority, or the disposition index's semantics.

| # | Change | Active lane it affects | Basis |
|---:|---|---|---|
| 1 | Return Upload `467b0c2` for HIGH-2 and MEDIUM-4 **as a bounded additive repair with a declared delta budget and no new public surface**, and require the projection oracle's precondition to be checked at the production allocation boundary rather than at the sort | Upload; the corpus's only open reproduced-production residual | §5.1 U3-H2/U3-M4 `[R]`/`[T]`; §9 I3/I4 |
| 2 | Carry the erase-before-fallible-release contract to the Upload repair as a **cross-surface** clause citing MR-list `68400355…` MUST-3 and MR-write `214e42fc…` F5 by report SHA-256, so the third instance is repaired once | Upload, MR-write | §6.3 `[P]`/`[R]` |
| 3 | Re-bind the shared-primitives preflight to accepted MR-list `b164046` **plus the four rejecting reports as specifications**, and drop the accepted-Upload precondition | MR-write, blocked ≥20 h | §6.3, §11 answer 4 |
| 4 | Record the four MR-list `ACCEPT` observations and the inherited GET/POST displacement asymmetry as **one** successor scoped to the shared response-owner primitive, not four surface-local patches | MR-list successor | §5.2 M4 `[R]` |
| 5 | Before the Storage repair resumes, re-partition its 23 open items (F1–F9 + R1–R14) into (a) reproduced defects, (b) behavior-closed-but-reversal-not-run, (c) claim-only. `[R]` The rereview's own reversal table shows `not run` in every column for all nine F-items, so (b) is provably non-empty | Storage, the largest open finding set | §5.7 `[R]`; §9 I5 |
| 6 | Require the next Storage and Ancestry reviews to execute at least one reversal per finding, or to mark the finding's closure state explicitly unproven. `[R]` The Storage initial review ran no build at all | Storage, Ancestry | §6.4 `[R]` |
| 7 | When File Doctor `cd130e5` is admitted, treat the `0 selected / 794 filtered` result as a **fail-closed** review input, and require the release evidence to cover the full NOTE-1/NOTE-2/NOTE-3 set rather than 16 cases | File Doctor | §5.5 FD3 `[R]`; non-review audit §B01 |
| 8 | When Rust HTTP `92fc810` is admitted, note `[P]` that the vacuous scaling oracle `merge_request_response_work_is_flat_while_object_work_scales` is **absent** at `92fc810`, and require its replacement to move the escrowed-object axis observably | Rust HTTP | §5.6 H2-F-1 `[P]` |
| 9 | Add one line to each future rejecting report: for every finding, whether it is in code the reviewed diff created. `[P]` This is a one-command determination and it is the input every intervention in §9 needs | all lanes | §1.2, §9.0 |
| 10 | Stop promoting reviewer-labelled non-blocking residual risks into mandatory closure work without a manager decision recorded per item. `[T]` Rust HTTP's R1–R4 promotion produced a 3,426-line repair whose own new oracle was the next blocker | all lanes | §5.6 `[T]`/`[P]` |

---

## 11. Interfaces with the non-review audit at `c258cbf`

The ownership boundary in that audit's §11 is accepted as written and is not
renegotiated here. This section discharges its four explicit asks and states
what this audit needs back.

### 11.1 Answers to its four requests

**1. A published, stable `invariant_family` enumeration so `MET-NRT-008`
becomes computable.** Published in §12. It contains the nine families that
audit anticipated (request ledger/deadline, sealed snapshot, authority fence,
prepared response, retryable cleanup, publication transaction, error origin,
allocation accounting, evidence non-vacuity), with `sealed snapshot` folded into
`authority fence` and `charge-before-allocation` separated from
`allocation-accounting exactness`, because §6.3 shows those two behave
differently across surfaces. `[I]` `MET-NRT-008`'s value on this corpus, under
the counting rule *"a finding whose family had already appeared in an earlier
committed report on a different surface"*, is **≥0.51** — at least 26 of the 51
successor findings, concentrated in `IV-01`, `IV-02`, `IV-05` and `IV-06`.
Against that metric's `≤0.20` target it fails by more than 2×. The exact value
requires the `review.finding` event stream (§11.2); the hand count is a floor
because I resolved families conservatively and assigned only one family per
finding.

**2. A definition of "first-pass" that survives successive zero-residual
candidates on one surface.** Proposed: *first-pass acceptance* is measured per
**surface**, on the first candidate reviewed after all findings bound by the
surface's most recent rejecting report have been declared closed by the
implementer. Successive zero-residual generations driven by items that were
never blocking under §12's severity vocabulary are counted separately as
`policy_generations` and reported alongside — never netted into first-pass. On
this corpus that yields `[I]` first-pass = 0/9 surfaces, with ≥3
`policy_generations` (Rust HTTP gen 2, File Doctor gen 3, KGI/KPI gen 2).
Publishing both numbers prevents the metric from being improved by relaxing the
policy or by reclassifying a blocker.

**3. Decision on B13 — whether a bounded, proven differential can discharge
post-mutation Install/Dogfood evidence.** This audit's position, stated as an
acceptance-criteria judgement since that is this audit's ownership: **yes,
conditionally**, and the conditions are drawn from proven corpus failures
rather than invented. A bounded differential may discharge the post-capability
rerun only if it proves, **on the exact installed binary**, that (a) the
capability payload digest and declaration bytes are the only changed bytes; (b)
every installed command's normalized output digest is unchanged except the
declared payload; (c) the differential's own gate is non-vacuous by §12's
`E1` and `E8` rules — specifically that it selects a nonzero test count and
binds a source key, because `[R]` the corpus already contains a `0 selected /
794 filtered` success; and (d) the differential is itself independently
reviewed against a deliberate regression that changes one command's behavior
behind an unchanged declaration. If any condition is unmet, the full terminal
rerun stands. **Caveat stated plainly:** this criterion has no historical
instance in the corpus to validate it against, so it is a judgement, not an
evidence-backed finding, and it should be revisited when the first bounded
differential is actually authored.

**4. Whether the shared-primitives preflight may bind *rejecting* reports as
specifications.** **Yes.** Three independent grounds, all proven: the preflight
is report-only and writes no product byte `[T]`; a rejecting report is
immutable evidence whose reproduced counterexamples are exactly the contract's
negative cases, and this corpus's rejecting reports are high-precision (≤3%
withdrawal rate, §6.5 `[P]`); and the family it is meant to de-duplicate has
already been rediscovered three times while it waited `[P]`. Two conditions:
each primitive clause cites its source report by SHA-256, and any primitive
whose final shape depends on an open residual (Upload HIGH-2 today) is marked
`shape-pending` rather than frozen. This matches that audit's own stated
position and its Wave 2.1.

### 11.2 What this audit provides back

| Interface | Content |
|---|---|
| `IF-5` fulfilled | §12's `invariant_family` enumeration, stable IDs, ready to key `review.finding` events |
| `IF-3` input | Terminalization is a review-process concern only insofar as nudging affects independence. Position: **a bounded nudge after the reviewer's own verdict signal does not weaken independence**, because the verdict is already the reviewer's; what must never be nudged is the analysis before that signal, and a competing reviewer must never be started. This is exactly the existing `INTERVENE` precondition, so no rule change is required |
| `IF-4` converted to an acceptance rule | A zero-selected-test gate, a gate whose artifact cannot be bound to the candidate's source key, or a gate whose subject is unreachable **is not evidence** and a report may not cite it. §12 rules `E1`/`E8`. `[R]` The corpus has two instances |
| `IF-2` respected | This audit attributes **no** delay to review duration. §7.2 records instead that review handoff (85–300 s) and review rollover (540–600 s) are measured and fast, and that `MET-FLOW-006` is `[U]` |
| New request | `review.finding` events keyed by `(family_id, severity_state, introduced_by_reviewed_diff: bool)`. The third field is the one this audit had to recompute by hand from the object database 20+ times, and it is a one-command determination at candidate-freeze time |

### 11.3 One correction to a shared framing

The non-review audit's §5.4 states "17 of 21 reports are `REQUEST_CHANGES`" and
treats rework as dominant by count. Both are `[P]` correct. This audit adds the
qualification that changes what follows from it: **34 of the 51 successor
findings (67%) were blocking with no reproduced production defect** (§7.1), and two
entire lineages recurred with an explicit reviewer statement that no product
defect exists (§5.6). Rework count therefore overstates product-quality rework
and understates evidence-and-criterion rework. `MET-NRT-006` and `MET-NRT-010`
will both be misread if that split is not published alongside them — which is
what §11.1 answer 2's `policy_generations` term is for.

---

## 12. Published `invariant_family` and severity-state enumerations

Published so `MET-NRT-008` becomes computable and so §9's pilots have a stable
key. Stable IDs; additive-only; a new family requires a dated decision.

### 12.1 Invariant families

| ID | Family | Definition | Corpus instances (§6.3) |
|---|---|---|---:|
| `IV-01` | Request ledger / deadline ownership | One exact request-scoped ledger and deadline owns every input-proportional unit of work from body to cleanup; no helper mints its own ledger or runs unpolled | 9 |
| `IV-02` | Charge before allocation | Every owner reserves its exact capacity before the allocation, open, parse, or insertion it covers | 9 |
| `IV-03` | Allocation-accounting exactness | Charged counters equal the logical work performed: no phantom units, no omitted edges, no platform-dependent or blanket formula, no cliff below a declared public limit | 8 |
| `IV-04` | Authority fence and sealed snapshot | Pinned authority identity, metadata and content are reproved through the last observation before the terminal decision; the fence's placement matches its documented placement | 7 |
| `IV-05` | Prepared response and response-owner displacement | At most one response is prepared, proven and published; a displaced response is disposed before another is selected; a detected failure cannot be discarded by an unconditional publish | 10 |
| `IV-06` | Retryable cleanup and release | Owner state is never erased before its release succeeds; cleanup is idempotent and retryable; a cleanup failure neither strands debt nor suppresses the primary outcome | 5 |
| `IV-07` | Publication transaction and durable recovery | Every partially visible effect has a durable record an exact retry can resume; a failed publication is never recorded as complete; no second document follows a visible one | 5 |
| `IV-08` | Error-origin classification and typed diagnostics | Failure origin is marked at its boundary, not inferred from a shared exception class; every in-scope failure produces one typed terminal diagnostic | 6 |
| `IV-09` | Evidence non-vacuity | Every oracle offered as closure evidence satisfies §12.3 | 32 |

### 12.2 Severity states (replacing the six incompatible vocabularies)

Routing only. **None of these grants integration authority**, and
`ACCEPT_WITH_NOTES` remains forbidden.

| State | Meaning | Blocking? | Next action |
|---|---|---|---|
| `DEFECT-REPRODUCED` | Production-reachable wrong behavior with an executed counterexample | yes | bounded additive repair |
| `DEFECT-INJECTION-ONLY` | Wrong behavior reachable only by injection; no production input exhibited | yes, unless the reviewer states reachability is impossible | repair or documented reachability boundary |
| `CLOSURE-UNPROVEN` | Behavior appears correct; the required reversal was not executed | **not** a defect | execute the reversal; do not re-repair |
| `ORACLE-VACUOUS` | Behavior correct, evidence cannot detect the regression | yes, evidence-side only | fix the oracle; no product change |
| `CLAIM-FALSE` | Behavior correct, a durable claim is wrong | yes, claim-side only | correct the claim |
| `SCOPE-BOUNDARY` | Reviewer declines to call it a defect; outside the contract's scope | no | manager routing decision, recorded per item |
| `REQUIREMENT-UNSATISFIABLE` | The closure clause cannot be met as written | no | rewrite the criterion, not the candidate |

`[R]` Every state has at least one proven corpus instance (§9 I5's table).
`CLOSURE-UNPROVEN` and `REQUIREMENT-UNSATISFIABLE` are the two the current
vocabulary lacks, and they account for Storage's nine `OPEN` F-items and
MR-list's gen-3 F1 respectively.

### 12.3 Oracle admissibility rules

Each is a proven corpus failure, not a hypothesis. `[R]` locators in §9 I4.

| ID | Rule |
|---|---|
| `E1` | A gate selecting zero tests, or whose artifact is not bound to the candidate's source key, is not evidence |
| `E2` | The axis a curve claims to vary must observably vary |
| `E3` | An assertion must fire at or before the boundary it names |
| `E4` | A name or docstring claim must be entailed by the assertions below it |
| `E5` | Deleting the production line the oracle protects must change a result |
| `E6` | A reversal must fail behaviorally, not through a missing symbol, fixture or line number |
| `E7` | A frozen counterexample corpus is not the invariant; closure requires the property, not the list |
| `E8` | An unreachable or non-production subject is not evidence for a production claim |
| `E9` | Where the property concerns ordering or precedence, single-instance fixtures are inadmissible |

---

## 13. Threats to this audit's own validity

Stated because §6.5 shows the corpus rewards this and because every finding
above is only as good as its method.

1. **Six of seven controls in §9 are derived from the same corpus they will be
   scored against.** Capture figures will be optimistically biased. §9.1's
   held-out set is mandatory, not advisory.
2. **The severity mapping in §6.2 is mine `[I]`.** The reports use six
   vocabularies. A different mapping changes the weighted totals but not the
   qualitative result, which rests on a categorical fact: no oracle finding in
   the corpus was a reproduced production defect.
3. **`INITIAL_REVIEW_SCOPE_MISS` is reference-point sensitive.** I fixed the
   reference at the immediately preceding report in the lineage (§4). A reader
   who fixes it at the root review would move several rows from "missed" to
   "never in scope".
4. **The MR-list convergence correlation is *n*=1 lineage.** §6.4's
   predecessor-re-derivation observation is the audit's most actionable
   hypothesis and its least statistically supported one. It is routed to a
   pilot for exactly that reason.
5. **I executed no product test, gate, or Cargo command.** Every `[R]` rests on
   a report's own executed reproducer. I verified internal consistency and
   symbol/line existence at named revisions; I did not re-run a single
   reproducer.
6. **Aggregate counts are hand-derived `[I]`.** No `review.finding` event stream
   exists. A different reader counting the 143 findings from the same headings
   would plausibly differ by a few percent.
7. **Two lineages have no successor report** (Ancestry, File Doctor gen 3), so
   their later findings are `ATTRIBUTION_UNAVAILABLE` and the corpus is
   censored in the direction of the two most build-blocked surfaces. `[I]` If
   Rust surfaces recur differently from Python ones, this audit cannot see it.

---

## 14. Non-authorization and write boundary

This audit does **not**:

- accept, reject, or integrate any product candidate, or change any recorded
  verdict;
- infer a verdict from a candidate card, worker gate, scheduler `passed`
  status, active session, or missing report;
- authorize CFR-168, CFB-136, or CFB-138 Close;
- authorize Install, Dogfood, release, or capability re-advertisement;
- change the recovery ledger, review disposition index, open-execution audit,
  issue state, KGI/KPI policy thresholds, task packets, manifests, candidate
  cards, existing reports, or any product source;
- adopt any intervention in §9, or fix any template depth, checklist size, or
  item count;
- change the disposition index's semantics, the zero-residual rule, reviewer
  model selection, or whether `ACCEPT` implies integration authority — §12's
  enumerations are *proposed* vocabularies pending the §9 pilots and a manager
  decision;
- launch or stop any agent, provider, or scheduler process; or
- modify Cargo, Rust, network, or credential state.

`[P]` No Cargo/Rust command, provider launch, network call, or agent session was
created. Product bytes at head remain byte-identical to `e2a218e`.

The commit carrying this audit adds exactly one path:

```text
docs/development/ai/swarm-results/cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md
```

Adopting any §9 intervention requires its own packet and, where the existing
rules demand one, its own exact independent review. §10's immediate changes are
packet/ticket-scoping and evidence-standard changes; each still requires the
manager's routing decision, and none of them authorizes an integration.
