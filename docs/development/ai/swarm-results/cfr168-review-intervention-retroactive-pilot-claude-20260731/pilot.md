# CFR-168 Review-Intervention Retroactive Pilot

Status: report-only development-process experiment. **Not** a product acceptance
decision, **not** a review verdict, **not** an integration authorization, and
**not** an adoption of any intervention.

- Date: 2026-07-31
- Executing role: independent development-process researcher
- Provider/model: Claude / `claude-opus-5`, report-only lane
  `cfr168-review-intervention-retroactive-pilot-claude-20260731`

---

## 1. Exact task, audit, and corpus identities

| Field | Value |
|---|---|
| Canonical task base for this execution | `cdaee6ad3b5e497f6f872bfe76783d0e26e8df85` |
| Packet | `docs/development/ai/swarm-batches/cfr168-review-intervention-retroactive-pilot-20260731.md` |
| Packet's declared canonical task base | `abfee6ef25f27af35790138404ce6f144b3244da` |
| Root-cause audit | `docs/development/ai/swarm-results/cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md` |
| Audit SHA-256 (recomputed) | `2392450e957c8437b323ded492fc7597dc77f5147b74f3056c97c0efd9329780` — **matches** the packet `[P]` |
| Audit subject head | `c258cbfc061b14905554c9d319b350d2da7f771b` |
| Immutable product baseline | `e2a218e015953f082245f36dbc6aee906b837e5a` |
| Reports in the audit snapshot (recomputed at `c258cbf`) | **21** `review.md` files, **11,411** lines total `[P]` |
| Decisions (recomputed from each report's own verdict) | **4 `ACCEPT` / 17 `REQUEST_CHANGES`** `[P]` |
| Reports at this task base | 24 (21 + the 3 pre-registered temporal-holdout reports) `[P]` |
| Candidate commits resolved in this object database | 36 of 36 named revisions resolve as commits `[P]` |

The packet's declared base (`abfee6e`) is the commit that carried the audit; this
execution runs at `cdaee6a`, the commit that carried the packet. The difference is
recorded, not reconciled: it matters in exactly one place, the lineage-holdout
digest rule, and that rule is verified against `abfee6e` in §2.2 below.

**Write boundary.** This run's only write is
`docs/development/ai/swarm-results/cfr168-review-intervention-retroactive-pilot-claude-20260731/pilot.md`.
No product byte, policy, schema, tool, issue, ledger, index, task packet, candidate
card, or existing report was modified. No Codex, Fugu, subagent, network call,
Cargo/Rust command, install, or dogfood was used. Every executed probe ran against
scratch copies extracted from the object database into `/tmp`, and nothing was
written back.

---

## 2. Method and leakage controls

### 2.1 Recomputation, not aggregation

Every quantity below was recomputed from Git objects, report text, or executed
Python in this session. Where my recomputation disagrees with the audit's
published aggregate, **my number is reported and the disagreement is stated**
(§4.1, §4.7). Nothing was copied from the audit's tables.

Evidence markers, as the packet requires: `[P]` direct object/repository proof
recomputed here; `[R]` reproduced execution performed in this lane; `[I]`
inference; `[T]` task/packet/ticket claim; `[U]` unavailable.

### 2.2 The pre-registered lineage holdout, verified

`sha256("<canonical-task-base>|<lineage-name>")` reproduces **all seven** packet
digests exactly under base `abfee6ef25f27af35790138404ce6f144b3244da` and under
no other base I tried (`cdaee6ad…`, `c258cbf…`, the short form) `[P]`:

| family | lexicographic order of digests | selected (smallest) |
|---|---|---|
| Python | `38c16469…` `python-mr-write` < `521ae3e3…` `python-doctor-branch` < `9973258e…` `python-mr-list` < `d2193941…` `python-upload` | **`python-mr-write`** |
| Rust | `6a8542cf…` `rust-file-doctor` < `a3dcdb6e…` `rust-storage` < `b2f6f431…` `rust-http` | **`rust-file-doctor`** |

The packet's selection is therefore internally consistent and independently
reproducible. Eligibility (at least one successor report in the audit snapshot)
also holds for every listed lineage and excludes `rust-file-ancestry`, which has
none `[P]`.

### 2.3 Freeze order, made auditable

The packet requires the calibration analysis to be frozen before either holdout is
opened. The order actually executed was:

1. Calibration measurement completed (I3 sweep, I1 executions, I4 per-rule scoring,
   I5 relabelling) using only the seven non-held-out lineages.
2. The calibration text of §4 was written to a scratch file and hashed:
   **SHA-256 `4d57c323b7c20015c9cd388c4175c973631a67839d8e3d0cc427e58a7cec87d9`**,
   384 lines, at **2026-07-31T09:43:17Z**.
3. Only then were the lineage-holdout reports' finding/oracle sections and the
   three temporal-holdout `review.md` files opened and scored.

**§4 of this report is that frozen file, reproduced byte-for-byte.** It occupies
**lines 225–608 of this file**, from `### 4.1 I3 — recomputed successor deltas
(calibration)` to the final line of §4.8. `sed -n '225,608p' pilot.md | sha256sum`
returns the digest above; I verified this after assembly `[P]`. No sentence of §4 was edited after step 2; every holdout result
appears in §5 and §6 and every revision prompted by a holdout appears in §7
onward.

### 2.4 Leakage this pilot could not prevent, stated plainly

1. **The audit saw both held-out lineages while deriving its proposed controls.**
   §5.3 (MR-write) and §5.5 (File Doctor) of the audit are part of the material
   the packet requires me to read in full. The lineage holdout is therefore
   **not** a blind holdout; it is a holdout against *my* control derivation, not
   against the audit's. The packet records this and so do I. Its practical effect
   is that lineage-holdout capture figures below are still optimistically biased,
   just less so than calibration.
2. **The disposition index's rows for the three temporal subjects were visible**
   while I enumerated the corpus at step 1, because the index is a required corpus
   source and lists all 24 reports on one page. Those rows are one-paragraph
   dispositions, not findings, oracles, or deltas. No calibration choice in §4
   uses them; the delta rule, the boundary set, the E-rule scoring scheme, and the
   state vocabulary were all fixed by the packet or by the audit's §12, not by me.
   I record it as a partial leak rather than claiming a clean temporal blind.
3. **The three temporal reports' own `review.md` bodies were not opened until
   after the freeze.** That part of the temporal holdout is clean.

### 2.5 What was executed

Nine `codefire` revisions (`cb13da0`, `dece688`, `76baff8`, `467b0c2`, `7327e2a`,
`37f7427`, `38da898`, `4930d25`, `b164046`) plus `cab3126`, `21fe751`, `6dd4527`,
`7e87665`, `701469f`, `e2a218e`, `02e04c7` were extracted to `/tmp/pilot/trees`
and imported as modules; all imported cleanly `[R]`. Seven differential
reproducers were executed (§4.5 rows `R1`–`R5`, §5.2 row `G`). Rust and the
KGI/KPI Cargo-adjacent surfaces were analysed by object-database symbol
occurrence only — **no Cargo, `rustc`, Clippy, build, install, or dogfood command
was run**, and no Rust behavioural claim is made from text inspection.

### 2.6 Two probe artifacts, recorded rather than hidden

Two of my own probes produced results that are **not** defects, and the corpus's
own precedent (audit §6.5) is to say so:

- Constructing `_HTTPMergeRequestListOperation` by `__new__` at `4930d25` /
  `b164046` raised `AttributeError: … has no attribute '_cleanup_faults'`. That is
  my partial construction, not a product defect — and it incidentally proves the
  gen-3 repair added cleanup-fault recording `[R]`.
- The same technique on `_HTTPMergeRequestLane` at `6dd4527` raised
  `AttributeError: … has no attribute '_sealed'`. Same cause. The `21fe751` row of
  that experiment is unaffected and is the one the finding rests on.

---

## 3. Raw candidate and finding dataset

### 3.1 Corpus tuples

The packet's tuple is `(predecessor tree, repair ticket, implementer plan/result,
candidate tree, successor report, root-cause-audit classification)`. Reconstructed
for all 9 lineages, 24 candidate generations, 15 successor deltas. No tuple was
dropped; missing elements are marked `unavailable` and kept in every denominator.

| lineage | generations | candidates (in object order) | successor reports | tuple gaps |
|---|---:|---|---:|---|
| `python-upload` | 4 | `cb13da0` `dece688` `76baff8` `467b0c2` | 3 (`534f0983…`, `7632552e…`, `57d67921…`) | gen-1 report `985ca9e4…` is **unindexed**: `unavailable` |
| `python-mr-list` | 4 | `37f7427` `38da898` `4930d25` `b164046` | 4 | none |
| `python-mr-write` **(holdout)** | 2 | `21fe751` `6dd4527` | 2 | none |
| `python-doctor-branch` **(eligible, not held out)** | 2 | `01a7c86`/`fbef572` → `701469f` | 2 | none |
| `rust-file-doctor` **(holdout)** | 3 | `52a025e` `1a04490` `cd130e5` | 2 | gen-3 has **no report**: `unavailable` |
| `rust-http` | 3 | `71854e2` `73bc532` `92fc810` | 2 | gen-3 has **no report**: `unavailable` |
| `rust-storage` | 2 | `b3bc32a` `cdcb9bd` | 2 | none |
| `rust-file-ancestry` | 2 | `e93ebc2` `02b0d63` | 1 | gen-2 has **no report**: `unavailable` |
| `kgi-kpi` (control plane) | 2 | `bfa7f08` `0ae583e` | 2 | none |

I count **24 candidate generations across 9 surfaces and 15 successor deltas**,
where the audit records 22 and 14. The difference is enumerative, not factual: the
audit's 22-commit list includes the product baseline `e2a218e` and omits
`cb13da0`, `7327e2a`, `bfa7f08`, and the Doctor/Branch slices. My enumeration is
published above so the difference is checkable; every downstream denominator uses
mine.

### 3.2 Finding inventory

Recomputed by reading each report's own severity headings, not by adopting the
audit's hand count:

| report | gen | findings | report | gen | findings |
|---|---:|---:|---|---:|---:|
| root review `d41ae00c…` | — | 7 | `A1` `64a4831e…` | 1 | 8 |
| `D1` `5d76f8a5…` | 1 | 1 | `FD1` `c1b39ffc…` **(holdout)** | 1 | 8 |
| `D2` `c7a1665f…` | 2 | 0 | `FD2` `f7c52b05…` **(holdout)** | 2 | 3 |
| `M1` `95617c3c…` | 1 | 6 | `H1` `f30c22aa…` | 1 | 5 |
| `M2` `68400355…` | 2 | 7 | `H2` `8bb1a418…` | 2 | 4 |
| `M3` `dc4f66ba…` | 3 | 3 | `S1` `628a37f5…` | 1 | 9 |
| `M4` `5c952a9a…` | 4 | 0 | `S2` `6438df44…` | 2 | 14 |
| `W1` `925cbcdb…` **(holdout)** | 1 | 6 | `K1` `02a0d259…` | 1 | 35 |
| `W2` `214e42fc…` **(holdout)** | 2 | 7 | `K2` `268ec0b7…` | 2 | 7 |
| `U1` `534f0983…` | 2 | 4 | | | |
| `U2` `7632552e…` | 3 | 4 | | | |
| `U3` `57d67921…` | 4 | 2 open | | | |

**Total 140**, not the audit's 143. **Successor findings (generation ≥ 2) = 51**,
which reproduces the audit exactly `[P]`: 41 calibration + 10 lineage holdout. The
three-row difference in the grand total turns on whether `H1`/`H2`'s
reviewer-labelled residual risks and `M4`'s three closed `F`-items count as
findings. I publish my reading rather than adopting the audit's; the successor
subset, which every measurement below depends on, is identical either way.

### 3.3 Independent verification of the audit's decisive symbol tests

Twenty symbol-occurrence tests were re-run against the exact trees. **All twenty
reproduce the audit's stated result** `[P]`. The load-bearing ones:

| symbol | path | predecessor | candidate | audit claim | reproduced? |
|---|---|---|---|---|---|
| `_AccountedUploadSourceMemory` | `codefire` | `dece688` = 0 | `76baff8` = 6 | repair-created | yes |
| `RemoteJSONPublicationRootProof` | `codefire` | `dece688` = 0 | `76baff8` = 6 | repair-created | yes |
| `is_transport_os_error` | `codefire` | `7327e2a` = 0, `37f7427` = 0 | `38da898` = 3 | repair-created | yes |
| `revalidate_json_authority_capture` | `codefire` | `37f7427` = 0 | `38da898` = 2 | repair-created | yes |
| `_HTTPReadResponseSlot` / `require_publishable_state` | `codefire` | `38da898` = 0 | `4930d25` = 6 / 2 | repair-created | yes |
| `merge_request_response_work_is_flat_while_object_work_scales` | `http.rs` | `71854e2` = 0 | `73bc532` = 1, `92fc810` = **0** | created then removed | yes |
| `StorageTraversalFrame` | `storage.rs` | `b3bc32a` = 0 | `cdcb9bd` = 8 | repair-created | yes |
| `write_all_tracked` | `storage.rs` | `b3bc32a` = 0 | `cdcb9bd` = 2 | repair-created | yes |
| `StorageReportOptions` | `storage.rs` | `b3bc32a` = 7 | `cdcb9bd` = 7 | pre-existing | yes |
| `validate_sealed_commit(remote_dirs(project_root)["objects"], root_id)` | `codefire` | `e2a218e` = 1, `dece688` = 1 | `467b0c2` = **0** | pre-existing, later removed | yes |
| `read_json_accounted` | `codefire` | `7327e2a` L1435–1504 | `37f7427` L1435–1504 | **byte-identical** | yes |
| `_HTTPMergeRequestListOperation._release` | `codefire` | `37f7427` | `38da898` | **byte-identical** | yes |

The audit's object-level method is sound. Its *aggregates* are where my
recomputation diverges (§4.1, §4.7).

---

## 4. Frozen calibration analysis

> Everything from here to the end of §4.8 is the frozen artifact of §2.3,
> SHA-256 `4d57c323b7c20015c9cd388c4175c973631a67839d8e3d0cc427e58a7cec87d9`,
> reproduced verbatim. It was written before any holdout was opened.

### 4.1 I3 — recomputed successor deltas (calibration)

Delta rule, fixed before measurement: `D(g) = git diff <predecessor reviewed candidate>
<candidate> -- <reviewed implementation+test paths>`, insertions only.
Reviewed paths: Python `codefire` + `tests/`; Rust `crates/`; KGI/KPI control plane
`docs/development/tools/`, `docs/development/ai/schemas/`,
`docs/development/ai/flow-metrics/`, `development-flow-kgi-kpi.md`,
`role-routing-matrix.json`. Documentation-of-record and copied prior reports are
excluded from `D` and reported separately as `all-paths`.

| gen | surface | predecessor → candidate | `D` (ins) | del | all-paths ins | new prod symbols | new type-level surface |
|---|---|---|---:|---:|---:|---:|---:|
| UP-g2 | python-upload | `cb13da0` → `dece688` | 556 | 21 | 832 | 1 | 0 |
| UP-g3 | python-upload | `dece688` → `76baff8` | 2,157 | 117 | 2,834 | 52 | 11 |
| UP-g4 | python-upload | `76baff8` → `467b0c2` | 962 | 31 | 2,554 | 2 | 0 |
| ML-g2 | python-mr-list | `37f7427` → `38da898` | 1,555 | 105 | 2,319 | 13 | 8 |
| ML-g3 | python-mr-list | `38da898` → `4930d25` | 2,188 | 256 | 3,087 | 34 | 14 |
| ML-g4 | python-mr-list | `4930d25` → `b164046` | 731 | 110 | 1,598 | 12 | 3 |
| DB-g2 | python-doctor-branch | `7e87665` → `701469f` | 17 | 2 | 17 | 0 | 0 |
| RH-g2 | rust-http | `71854e2` → `73bc532` | 2,674 | 558 | 3,426 | 47 | 7 |
| RH-g3 | rust-http | `73bc532` → `92fc810` | 1,305 | 73 | 1,927 | 20 | 3 |
| ST-g2 | rust-storage | `b3bc32a` → `cdcb9bd` | 2,935 | 289 | 4,007 | 143 | 46 |
| AN-g2 | rust-file-ancestry | `e93ebc2` → `02b0d63` | 2,779 | 813 | 3,488 | 100 | 25 |
| KG-g2 | kgi-kpi | `bfa7f08` → `0ae583e` | 4,292 | 358 | 5,582 | 0 | 0 |

All rows `[P]`, recomputed from the object database. Calibration total 22,151
insertions across 12 successor generations; median `D` = 2,172.5; smallest 17,
largest 4,292.

**Divergence from the audit's §3.1, recorded rather than reconciled away.** The
audit's per-generation figures are not produced by one uniform rule: `+4,007/−291`
(Storage g2), `+3,488/−815` (Ancestry g2), `+3,426/−560` (Rust HTTP g2),
`+2,517/−291` (File Doctor g3) and `+1,927/−73` (Rust HTTP g3) reproduce my
*all-paths lineage* column `[P]`; `+2,132/−266` (File Doctor g2), `+4,011/−415`
(KGI g2), `+17/−2` (Doctor+Branch g2), `+2,718/−133` (MR-write g1) and
`+2,548/−18` (MR-list g1) reproduce the *commit-versus-Git-parent* diff `[P]`;
`+1,293/−40` (Upload g4) and `+813/−122` (MR-list g4) reproduce neither. The
audit's "median 2,180 insertions across the 14 successor deltas" is therefore not
reproducible as stated. I also count **15** successor deltas across 9 surfaces,
not 14 (12 calibration + 3 holdout); the enumeration is published above and in
§4.6 so the difference is checkable.

New-symbol sets were recomputed independently of the audit's names — by Python
AST symbol diff on `codefire` and by item-declaration diff on the changed Rust
`src` files. The recomputation *contains* all six symbols the audit named
(`_AccountedUploadSourceMemory`, `RemoteJSONPublicationRootProof`,
`_HTTPReadResponseSlot`, `route_filesystem_boundary`, `is_transport_os_error`,
`StorageTraversalFrame`, `write_all_tracked`) and adds 42 further new top-level
Python symbols and several hundred Rust items the audit did not name `[P]`.

Newly introduced surface by kind, recomputed (`[P]`, top-level only):

| gen | new public API | new state machine / owner type | new classifier | new dispatch protocol |
|---|---:|---:|---:|---:|
| UP-g3 | 6 (`canonical_json_stream`, `canonical_json_stream_digest`, `RemoteJSONPublicationIdentity`, `RemoteJSONPublicationRootProof`, + 2 module constants' accessors) | 3 (`_AccountedUploadSourceMemory`, `RemoteJSONPublicationRootProof`, `RemoteJSONPublicationIdentity`) | 0 | 0 |
| ML-g2 | 6 (`filesystem_authority_error`, `http_filesystem_authority_error`, `pinned_authority_changed_error`, `revalidate_json_authority_capture`, `validate_remote_project_descriptor_value`, `is_transport_os_error`) | 1 (capture protocol on `read_json_accounted`) | 1 (`is_transport_os_error`) | 0 |
| ML-g3 | 11 | 2 (`_HTTPReadResponseSlot`, `_RemoteProjectGateFence`) | 4 (`classify_route_os_error`, `is_filesystem_os_error`, `os_error_origin`, `mark_*_origin`) | 2 (`route_filesystem_boundary`, `transport_boundary`) |
| ML-g4 | 3 diagnostics | 0 (extends `_HTTPReadResponseSlot`) | 0 | 0 |
| RH-g2 | 1 (`pub fn consume_reserved_pair_exact_and_release`) | 2 (`MergeRequestPreflightState`, `PreparedSelectionFault`) | 1 (`http_deadline_phase_id`) | 1 (`HTTP_DEADLINE_PHASE_TABLE`) |
| ST-g2 | 0 `pub` | ≥4 (`StorageTraversalFrame`, `PinnedBoundedFile`, tracked-write path, terminal classifier) | 1 (`storage_terminal_class`) | 1 (`write_all_tracked`) |

`UP-g2`, `UP-g4`, `DB-g2`, `KG-g2` introduced **no** new top-level public surface
`[P]`.

### 4.2 I3 — where the later findings sat

For each measured calibration successor generation, the findings raised by the
report on that candidate, partitioned by whether the finding's cited locus is a
symbol **absent at the predecessor tree** (`new`) or present (`pre-existing`).
Every partition is `[P]` by `git show <tree>:<path> | grep -c <symbol>`.

| gen | findings on this candidate | locus new at this gen | locus pre-existing | other surface |
|---|---:|---:|---:|---:|
| UP-g2 (`dece688`, report `534f0983…`) | 4 | 1 | 3 | 0 |
| UP-g3 (`76baff8`, report `7632552e…`) | 4 | 3 | 1 | 0 |
| UP-g4 (`467b0c2`, report `57d67921…`) | 2 | 1 | 1 | 0 |
| ML-g2 (`38da898`, report `68400355…`) | 7 | 2 | 5 | 0 |
| ML-g3 (`4930d25`, report `dc4f66ba…`) | 3 | 3 | 0 | 0 |
| ML-g4 (`b164046`, report `5c952a9a…`) | 0 | 0 | 0 | 0 |
| DB-g2 (`701469f`, report `c7a1665f…`) | 0 | 0 | 0 | 0 |
| RH-g2 (`73bc532`, report `8bb1a418…`) | 4 | 3 | 1 | 0 |
| RH-g3 (`92fc810`) | **unavailable** — review launch blocked, no report | — | — | — |
| ST-g2 (`cdcb9bd`, report `6438df44…`) | 14 | 10 | 2 | 2 (`R13` test-only, `R14` doc-only) |
| AN-g2 (`02b0d63`) | **unavailable** — review never launched, no report | — | — | — |
| KG-g2 (`0ae583e`, report `268ec0b7…`) | 7 | 7 | 0 | 0 |

Totals: **45 findings / 30 new-locus / 13 pre-existing-locus / 2 other-surface**,
with **2 of 12 generations `unavailable`** and retained in the denominator.

### 4.3 I3 — the boundary sweep

Every distinct observed calibration insertion count is a candidate boundary, plus
`unbounded`. No round number was introduced. `capture(B)` is the count of
new-locus findings on generations whose delta exceeded `B`; it is an **upper
bound**, because it credits the boundary with preventing every defect in surface
the repair would not have been permitted to write. The corresponding lower bound
is 0 — a bounded repair may write the same defective surface one generation
later. Both bounds are reported; nothing between them is measured.

| `B` (insertions) | generations over `B` | new type-level surface refused | capture(B) (upper bound, /30) | pre-existing findings retained (/13) | provably infeasible generations | displaced lines | displaced share |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 17 | 11 | 117 | 30 | 13 | 5 | 21,947 | 0.991 |
| 556 | 10 | 117 | 29 | 13 | 1 | 16,018 | 0.723 |
| 731 | 9 | 114 | 29 | 13 | 0 | 14,268 | 0.644 |
| 962 | 8 | 114 | 28 | 13 | 0 | 12,189 | 0.550 |
| 1,305 | 7 | 111 | 28 | 13 | 0 | 9,445 | 0.426 |
| 1,555 | 6 | 103 | 26 | 13 | 0 | 7,695 | 0.347 |
| 2,157 | 5 | 92 | 23 | 13 | 0 | 4,083 | 0.184 |
| 2,188 | 4 | 78 | 20 | 13 | 0 | 3,928 | 0.177 |
| 2,674 | 3 | 71 | 17 | 13 | 0 | 1,984 | 0.090 |
| 2,779 | 2 | 46 | 17 | 13 | 0 | 1,669 | 0.075 |
| 2,935 | 1 | 0 | 7 | 13 | 0 | 1,357 | 0.061 |
| 4,292 | 0 | 0 | 0 | 13 | 0 | 0 | 0.000 |
| unbounded | 0 | 0 | 0 | 13 | 0 | 0 | 0.000 |

**"Provably infeasible"** counts generations where `B` is below the number of
production insertions that landed *inside functions or classes cited by that
generation's own bound findings* — a measured lower bound on the repair's
required size, not an estimate. Recomputed per generation `[P]`:

| gen | production insertions | inside bound-finding symbols (required) | discretionary | required share |
|---|---:|---:|---:|---:|
| UP-g3 | 1,151 | 637 | 514 | 0.55 |
| UP-g4 | 156 | 144 | 12 | 0.92 |
| ML-g2 | 473 | 290 | 183 | 0.61 |
| ML-g3 | 768 | 346 | 422 | 0.45 |
| ML-g4 | 265 | 227 | 38 | 0.86 |
| DB-g2 | 17 | 17 | 0 | 1.00 |
| UP-g2, RH-g2, RH-g3, ST-g2, AN-g2, KG-g2 | — | **unavailable** (no AST span mapping for Rust/JSON/doc surfaces in a non-build lane) | — | — |

Required-share is `unavailable` for 6 of 12 generations. The test-evidence half of
the requirement is `unavailable` for **all 12**: no ticket states the number of
test lines its required reversal matrix costs, and attributing inserted test lines
to individual bound findings is not mechanically decidable here.

**Reading of the curve.** Capture is almost flat from `unbounded` down to 2,779
(0 → 17 of 30) and then rises steeply: 17 at 2,674, 23 at 2,157, 28 at 962. It is
already 29 of 30 at `B` = 731 and gains only one more finding all the way down to
17, while provable infeasibility appears at 556 and reaches 5 of 6 measurable
generations at 17. Displacement rises monotonically the whole way: at 731,
64% of all calibration successor insertions must move to some later generation.

The region where capture is high and no generation is provably infeasible is
therefore **`B` ∈ [731, 1,555]**, and inside it the corpus offers exactly two
existence proofs that a closure of that size is achievable: `ML-g4` (`D`=731,
7 bound items closed, `ACCEPT`) and `DB-g2` (`D`=17, one bound item closed,
`ACCEPT`) — the two smallest calibration deltas and the only two terminal-accepted
ones `[P]`. Below 731 the corpus contains no example of a multi-finding closure at
all, and `UP-g3`'s production-only requirement (637) would already consume 87% of
a 731 budget before a single test line.

**This is a region, not a recommendation.** Three properties needed to convert it
into one are missing: displacement at 0.64 is not costed anywhere (no
generations-to-terminal measurement survives, §4.4); the capture figure is an
upper bound with an unmeasured lower bound of 0; and the test-evidence half of
every required-size denominator is `unavailable`.

### 4.4 I3 — generations-to-terminal

Measurable on 2 of 9 surfaces `[P]`: MR-list reached `ACCEPT` at generation 4
after deltas 2,548 / 1,555 / 2,188 / 731, and Doctor+Branch at generation 2 after
a 17-line delta. No surface in the corpus was repaired under a declared delta
budget, so the counterfactual "would a budget have added generations?" has
**no observation** in this corpus: `unavailable`, denominator 9 surfaces, 0
observed. This is the single largest gap in I3 and it is not closable
retroactively.

### 4.5 I1 — predecessor re-derivation (Python-first; no Cargo, no Rust build)

Two distinct obligations were scored separately, because the corpus separates
them sharply:

- **I1-A, differential**: run the finding's smallest reproducer against the
  predecessor tree and the candidate tree and compare. Captures only when the
  predecessor contains the implicated code *and* behaves differently.
- **I1-B, predecessor derivation**: derive the predecessor's behaviour from the
  predecessor tree (load/execute it) rather than from the candidate's
  reimplementation or the previous report's prose. Captures when that derivation
  exposes the fact before the candidate is frozen.

Nine `codefire` revisions were extracted to a scratch directory and imported
cleanly (`cb13da0`, `dece688`, `76baff8`, `467b0c2`, `7327e2a`, `37f7427`,
`38da898`, `4930d25`, `b164046`) `[R]`. No product byte was modified and nothing
was written into the repository.

**Executed differentials** (all `[R]`, this lane, this session):

| # | Subject | Predecessor result | Candidate result | I1-A | I1-B |
|---|---|---|---|---|---|
| R1 | `RemoteJSONPublicationRootProof` double consumption (`U2-H3`) | `dece688`: class **absent** | `76baff8`: attempt 1 accepted, attempt 2 accepted, `consumptions=2`; `467b0c2`: attempt 2 rejected "already consumed" | no subject | **no** |
| R2 | `_AccountedUploadSourceMemory.close` erase-before-fallible-release (`U3-H2`) | `dece688`: class **absent** | `76baff8` **and** `467b0c2`: injected `release` failure strands **11,456** retained bytes, `_closed=True`/`_acquired=False`, retry is a no-op | no differential | **yes at gen 4** (predecessor `76baff8` exhibits it identically) |
| R3 | `_HTTPMergeRequestListOperation._release` cleanup swallow (`M2-MUST3`) | `37f7427`: `setattr(...,0)` then `try: release except BaseException: pass` — swallowed, attribute zeroed | `38da898`: **byte-identical**, swallowed | no differential | **yes** |
| R4 | `read_json_accounted` parent-open fail-open arm (`M2-MUST1`) | `7327e2a` **and** `37f7427`: an `ELOOP` parent returns the caller's default silently, zero ledger events | `38da898`: identical; `4930d25`: raises typed `CFError: filesystem authority is unusable` | no differential | **yes** |
| R5 | Filesystem-origin `TimeoutError` classification (`M2-MUST2`) | `7327e2a`, `37f7427`: `is_transport_os_error` **absent** | `38da898`: returns `True` for `TimeoutError(110, …, "/storage/authority.json")`; `4930d25`: `False` | no subject | **no** |

`R3` and `R4` are the decisive pair. Both defects are **byte-identical** at
predecessor and candidate, so the differential mode sees nothing, while the
derivation mode sees them immediately — and `R4` directly contradicts the prose
the previous report published about that same function. The corpus's one
canonical I1 success is a *derivation* success, not a *differential* success.

**Per-finding capture** (universe: the 45 findings of §4.2):

| Class | n | I1-A capture | I1-B capture | not captured | unavailable |
|---|---:|---:|---:|---:|---:|
| locus pre-existing at the predecessor | 13 | 0 | **10** | 0 | 3 (`H2-F-4`, `S2-R2`, `S2-R6` — Rust, need execution) |
| locus new at the predecessor (repair-created surface) | 30 | 1 (`U1-H2`) | 0 | 29 | 0 |
| other surface (test/doc only) | 2 | 0 | 0 | 2 | 0 |
| **total** | **45** | **1** | **10** | **31** | **3** |

Combined I1 capture = **11 / 45**, unavailable **3**, i.e. 11 numerator, 45
denominator, 3 of the 45 undecidable. Restricted to the class I1 is aimed at —
`INITIAL_REVIEW_SCOPE_MISS`, findings whose locus predates the reviewed delta —
capture is **10 of 13 with 3 unavailable**, i.e. every Python instance and no
Rust instance.

Rust was scored by the one non-build method already proven in this corpus:
symbol-occurrence at the exact predecessor tree. Where that establishes the locus
is **absent** from the predecessor, "I1 cannot capture" is a `[P]` determination
requiring no build (15 Rust findings). Where the locus pre-exists, deciding
capture needs behavioural observation, and those 3 rows are `[U]`. No Rust
capture is claimed from text inspection.

**Cost.** For 10 of the 11 captures the obligation adds **no new executable
artifact** — it is a second invocation of a reproducer the reviewer already wrote,
against a second tree `[R]`. `R4`'s capture needed no execution at all: reading
the predecessor's `except` arms is sufficient. The one added artifact across the
whole calibration set is the fault-injection harness in `R2`, and the gen-4 ticket
already required it `[T]`. Author burden 0 new artifacts; reviewer burden 1 new
obligation per parity claim.

**Noise and false positives.** I1 emits one output per parity claim, and every
output is a fact about a committed tree. Calibration noise = **0 of 45**
(no output failed to match a later finding or the audit ledger, because the
obligation produces observations, not accusations). Duplication = **3 of 21
reports already discharged it** (`M1`, `M3`, `U3`) `[P]`, so on those three the
obligation adds nothing. False positives = **0 of 45**, all checked against the
object database.

### 4.6 I4 — oracle admissibility, `E1`–`E9` scored separately

**Universe.** A *historical closure oracle* is one row of a calibration report's
own explicit oracle / mutation / reversal / non-vacuity matrix — an artifact
offered as evidence that a bound finding or bound contract item is closed, with
that report's adjudication attached. Enumerated: `U2` 14 reversal rows, `U3` 9
required mutations, `M1` 1 precedence oracle, `M2` 8 adjudicated oracles, `M3` 2,
`M4` 5 candidate-regression rows, `H2` 6 deliberate regressions + 1 scaling
oracle, `S2` 9 `F1`–`F9` reversal rows + 6 further adjudicated oracles, `K2` 4,
`A1` 4, `D2` 1 = **66 rows**. The enumeration is the denominator; it is `[I]` as a
set boundary and `[P]`/`[R]` per row.

**Never collapsed into a checklist score.** Per rule:

| rule | applicable | rejects | capture | noise (admissible oracle rejected) | duplicates a bound ticket clause | mechanically decidable? |
|---|---:|---:|---:|---:|---:|---|
| `E1` zero-selected / unbound-artifact gate | **0** | 0 | 0 | 0 | 0 | yes — but calibration contains **no instance**; the corpus's only proven instance sits in the lineage holdout, so `E1` is scored in §5, not here |
| `E2` the asserted axis must observably move | 3 | 1 | **1** (`H2` `F-1`: `source_closure` 8/8/8 while the store grows 9→11→18) | 0 | 0 | yes — compare counters across multipliers |
| `E3` assertion fires at or before the boundary it names | 6 | 4 | **4** (`U3` `m1`, `U3` `m9`, `M2` final-fence oracle, `M3` ordering oracle) | **1** (`M4` "restore the proof inside the pinned gate" observes *after* the proof by design) | 2 | no — needs the named boundary located in production |
| `E4` a name/docstring claim must be entailed by its assertions | 66 | 6 | **6** (`M2` "one descriptor read", `M2` "every poll required", `H2` `F-1` doc comment, `U3` `m1`/`m9` test name, `S2` partial-publication test) | 0 | 1 | no — entailment is semantic |
| `E5` deleting the protected production line must change a result | 45 | 16 | **16** (`U2` rows 3, 5, 8, 10, 11, 12, 14; `S2` `F1`–`F9`) | **1** (`M4` "silent `_release_exact` restored" is a restoration, not a deletion, and is behaviourally decisive) | **23** | **yes** |
| `E6` reversal must fail behaviorally, not by missing symbol/fixture/line | 4 | 3 | **3** (`U2` old-target suite, `M4` legacy reversal errors, `S2` "category-3 leads only") | 0 | 2 | **yes** — error-vs-failure is a classification of existing output |
| `E7` a frozen counterexample corpus is not the invariant | 4 | 2 | **2** (`K2` 55/55 semantic, 15/15 structural — 7 new blocker classes, none in the 55) | **1** (`U3`'s nine required mutations are themselves a frozen corpus and were decisive) | 0 | no |
| `E8` an unreachable or non-production subject is not evidence | 8 | 6 | **6** (`H2` `F-1` unreachable objects, `A1` `SHOULD-1` fence cannot fire in a production build, `A1` `SHOULD-3` counters reach no production caller, `M3` `state_proved` has no production reader, `S2` internal-test-API publication, `S2` writer tests never spawn the top-level route) | 0 | 1 | partly — "no production caller" is a call-graph query; "unreachable by construction" is judgement |
| `E9` single-instance fixtures inadmissible for ordering properties | 4 | 1 | **1** (`M1` `test_legacy_cross_project_and_error_precedence_parity` injects exactly one fault per run) | 0 | 0 | **yes** — count injected faults per run |

Totals over the 66-row universe: **39 rejections, 39 captures, 3 noise, 0 false
positives, 29 duplications, 0 unavailable**. Rejections are not disjoint: 8 rows
are rejected by two or more rules (`H2` `F-1` by `E2`+`E4`+`E8`; `U3` `m1`/`m9` by
`E3`+`E4`; `S2` partial-publication by `E4`+`E8`; `U2` rows 8/11 by `E5` alone but
their successors by `E3`). Distinct rows rejected = **31 of 66**.

Signal is concentrated and unevenly:

- `E5` alone accounts for 16 of 39 captures, and **23 of its 45 applicable rows
  duplicate a clause the ticket already bound** `[T]` — it is mostly an
  enforcement problem, not a missing rule.
- `E8` (6) and `E4` (6) are the next largest and are almost duplication-free
  (1 each): they are the rules that add genuinely new obligations.
- `E2`, `E7`, `E9` each capture 1–2 rows on 3–4 applicable rows. They are
  high-precision and low-frequency: cheap to state, rarely triggered.
- `E1` has **zero calibration instances**. Its denominator here is 0 and it must
  not be credited with the File Doctor `0 selected / 794 filtered` case, which
  belongs to the lineage holdout.
- **`E3`, `E4` and `E7` are not mechanically decidable.** Any implementation that
  presents them as machine checks will either under-fire or produce the noise
  measured above.

**Rules the corpus needs and `E1`–`E9` does not contain.** Three calibration rows
were adjudicated defective and are rejected by **no** rule in the set: `M2`'s
"cleanup zero debt" oracle (vacuous because no release/close fault is injected at
all), `M2`'s "shared-route escaping generic `OSError`" oracle (the fixture never
enters the failure origin the property is about), and `K2`'s directory test (it
encodes the erroneous behaviour as the expected value). All three are the same
missing property: *the fixture must actually enter the state the invariant
concerns*. `E9` states that only for ordering. This is a measured gap of
**3 of 66**, not a proposal.

### 4.7 I5 — evidence state versus routing state

The 21 reports were re-counted from their own severity headings. My independent
count is **140** severity-listed findings, not 143: root review 7, `D1` 1, `D2` 0,
`M1` 6, `M2` 7, `M3` 3, `M4` 0, `W1` 6, `W2` 7, `U1` 4, `U2` 4, `U3` 2, `A1` 8,
`FD1` 8, `FD2` 3, `H1` 5, `H2` 4, `S1` 9, `S2` 14, `K1` 35, `K2` 7. The three-row
difference is attributable to whether `H1`/`H2`'s reviewer-labelled residual risks
and `M4`'s three closed `F`-items are counted as findings; both readings are
defensible and I publish mine rather than adopting the audit's. **Successor
findings (generation ≥ 2) = 51**, which reproduces the audit exactly `[P]`; of
those, **41 are calibration** and 10 are lineage holdout.

Re-labelled under the audit's proposed §12.2 states, preserving every original
verdict (calibration successor findings, n = 41):

| proposed state | n | representative instance |
|---|---:|---|
| `DEFECT-REPRODUCED` | 13 | `U2` `BLOCKER-1` (expired-deadline recovery publishes), `M2` `MUST-1`/`MUST-2`/`MUST-3`/`MUST-4`, `U3` `HIGH-2` (11,456 B stranded, `[R]` reproduced here) |
| `ORACLE-VACUOUS` | 12 | `H2` `F-1`, `U2` `MEDIUM-4`, `U3` `MEDIUM-4`, `M2` `COULD-1`/`COULD-2`, `S2` `R13` |
| `CLAIM-FALSE` | 6 | `H2` `F-2` (57 vs 60 bytes), `H2` `F-3`, `S2` `R14`, `M2`-adjacent catalog claims |
| `DEFECT-INJECTION-ONLY` | 4 | `M3` `F3` (guard raises, `200` publishes; the report states no production input leaks ownership past `close()`) |
| `SCOPE-BOUNDARY` | 3 | `H2` `F-4` (pre-existing, "hardening rather than a regression") |
| `REQUIREMENT-UNSATISFIABLE` | 1 | `M3` `F1` (`return` from inside `with remote_project_gate(...)`; teardown is unavoidable in the language) |
| `CLOSURE-UNPROVEN` | 0 among the 41 findings, but **9 among `S2`'s prior-finding dispositions** | `S2` `F1`–`F9`, all recorded `OPEN` with `not run` in both reversal columns |
| **no §12.2 state fits** | 2 | `S2` `R2`, `S2` `R6`: source-proven by a lane that executed nothing. They are neither reproduced, nor injection-only, nor claim-only, nor closure-unproven |

`CLOSURE-UNPROVEN` was **never** treated as accepted, and `ACCEPT_WITH_NOTES` was
not invented. The 9 `S2` `F1`–`F9` rows are the state's clearest instance and they
sit in the *disposition* table, not the finding table — which is precisely the
conflation `I5` names.

**Generation routing.** For each calibration successor generation, the state class
of the findings that made it mandatory:

| gen | driven by | only-routable? |
|---|---|---|
| UP-g2 | **unavailable** — its driving report `985ca9e4…` is not in the indexed snapshot | `[U]` |
| UP-g3 | mixture: `DEFECT-REPRODUCED` (`U1-B1`, `U1-H2`) + `ORACLE-VACUOUS` | no |
| UP-g4 | mixture: `DEFECT-REPRODUCED` ×3 + `ORACLE-VACUOUS` | no |
| ML-g2 | mixture: `DEFECT-REPRODUCED` ×2 + `ORACLE-VACUOUS` ×2 | no |
| ML-g3 | mixture: `DEFECT-REPRODUCED` ×4 + `DEFECT-INJECTION-ONLY` + `ORACLE-VACUOUS` ×2 | no |
| ML-g4 | mixture: `REQUIREMENT-UNSATISFIABLE` (`F1`) + `DEFECT-REPRODUCED` (`F2`) + `DEFECT-INJECTION-ONLY` (`F3`) | no |
| DB-g2 | `CLAIM-FALSE` only — the reviewer states it "is not evidence of a product regression"; the delta is 17 lines in one test file | **yes** |
| RH-g2 | `ORACLE-VACUOUS` + `CLAIM-FALSE` ×2 + `SCOPE-BOUNDARY` ×4 (`R1`–`R4` promoted); report: "The production change is sound. I found no product defect" | **yes** |
| RH-g3 | `ORACLE-VACUOUS` + `CLAIM-FALSE` ×2 + `SCOPE-BOUNDARY`; report: "I found no product defect" | **yes** |
| ST-g2 | 9 source-proven defects from a lane that executed nothing — no §12.2 state fits | no |
| AN-g2 | mixture: `DEFECT-REPRODUCED` (`MUST-1`) + contract-scope | no |
| KG-g2 | `DEFECT-REPRODUCED` on the control plane (55 semantic mutations accepted, 0 rejected) | no |

**Generations driven only by states the extended vocabulary would route rather
than block: 3 of 12, with 1 unavailable retained in the denominator** — `DB-g2`,
`RH-g2`, `RH-g3`. Numerator 3, denominator 12, unavailable 1. Expressed in
delta: those three generations account for 3,996 of 22,151 calibration successor
insertions (18.0%) and, by §4.2, for **7 of the 45** later findings.

The audit's §9 `I5` guess was "at least Rust HTTP gen 2, File Doctor gen 3, and
part of MR-list gen 3". Calibration confirms Rust HTTP gen 2, adds Rust HTTP gen 3
and Doctor+Branch gen 2, and **rejects** the MR-list gen 3 part: `ML-g3` was
driven by four `DEFECT-REPRODUCED` findings `[R]`. File Doctor gen 3 is holdout and
is not scored here.

### 4.8 Frozen calibration conclusions

1. **I3.** The capture/burden region is `B` ∈ [731, 1,555] insertions on the
   reviewed implementation+test surface, with capture 26–29 of 30 (upper bound),
   0 provably infeasible generations, and displacement 0.35–0.64 of all successor
   insertions. The region has two existence proofs inside it and none below it.
   Generations-to-terminal is `unavailable` on 9 of 9 surfaces.
2. **I1.** Capture 11/45 overall, 10/13 within its target class, 3 unavailable.
   The capture mechanism that works is **derivation**, not **differential**:
   2 of the 2 executed derivation cases captured, 1 of 5 executed differential
   cases captured. Cost is 0 new author artifacts, 1 reviewer obligation.
   3 of 21 reports already discharge it.
3. **I4.** 39 captures over 66 oracle rows, concentrated in `E5` (16, of which 23
   applicable rows already duplicate a bound clause), `E8` (6) and `E4` (6).
   `E2`/`E7`/`E9` are precise and rare. `E1` has no calibration instance. Noise 3,
   false positives 0. `E3`/`E4`/`E7` are not mechanically decidable. Three
   adjudicated defective oracles are caught by no rule in the set.
4. **I5.** 3 of 12 calibration successor generations were driven only by routable
   states; 1 unavailable. Two findings fit no proposed state.
5. **Baseline.** The historical target is "material reviewer correction on at most
   2 of 5 packets"; the observed corpus rate is 17 of 21 `REQUEST_CHANGES`
   `[P]`. On calibration, no intervention moves that packet-level ratio, because
   17 of the 21 reports would still have been written and 13 calibration successor
   findings are `DEFECT-REPRODUCED` — findings a review is supposed to raise. The
   only mechanism by which any intervention changes the ratio is by removing
   *generations*, and only I5 does that (3 of 12).

---

## 5. Lineage-holdout scores

Opened only after the §4 freeze. Held out: `python-mr-write` (`W1` `925cbcdb…`,
`W2` `214e42fc…`, candidates `21fe751` → `6dd4527`) and `rust-file-doctor`
(`FD1` `c1b39ffc…`, `FD2` `f7c52b05…`, candidates `52a025e` → `1a04490` →
`cd130e5`). Neither was used to choose the delta region, refine an E-rule, change
a state definition, or select an intervention.

### 5.1 I3 on the lineage holdout

| gen | predecessor → candidate | `D` (ins) | del | new prod symbols | new type-level surface | findings on this candidate | new-locus | pre-existing | other |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| MW-g2 | `21fe751` → `6dd4527` | 1,516 | 295 | 23 | 4 | 7 (`W2` `F1`–`F7`) | 1 (`F6`) | 5 | 1 (`F7`, doc) |
| FD-g2 | `52a025e` → `1a04490` | 2,078 | 265 | 54 | 8 | 3 (`FD2` `NOTE-1`–`3`) | 1 (`NOTE-1`) | 2 | 0 |
| FD-g3 | `1a04490` → `cd130e5` | 1,873 | 282 | 42 | 11 | **unavailable** — review launch `BLOCK`, no report | — | — | — |

Loci `[P]`: `AccountedRemoteCommitBorrow` 4 at `21fe751` and 4 at `6dd4527`
(pre-existing at the predecessor — `W2-F2` is an initial-review miss, not a
regression); `seal_prepublication` 1/1/1 (present and uncalled since `e2a218e`);
`_HTTPMergeRequestLane` 7 → 8; `remote_project_stable_id` 21 → 21;
`request_nonce_cache_postimage_from` 0 → 3 (`W2-F6`, the one new locus);
`panic!` in `doctor.rs` 3 → 9 → 18; `DoctorFenceSubject`/`UnavailableFault`
0 at `1a04490`, 14 at `cd130e5` (so `NOTE-3`'s gap is pre-existing and the fix
lands one generation later).

**Score against the frozen region `B` ∈ [731, 1,555].**

| `B` | generations over `B` | capture (new-locus findings avoided, upper bound) | pre-existing retained | provably infeasible | unavailable |
|---:|---:|---:|---:|---:|---:|
| 731 | 3 of 3 | 2 of 2 decidable | 7 | `[U]` (Rust/AST) | 1 gen (FD-g3) |
| 1,516 | 2 of 3 | 1 of 1 decidable | 7 | `[U]` | 1 gen |
| 1,555 (region top) | 2 of 3 | 1 of 1 decidable | 7 | `[U]` | 1 gen |

The decisive holdout observation is **MW-g2 at `D` = 1,516: it already sits inside
the frozen region**, so a budget anywhere in [731, 1,555] would not have split it —
and it still produced 7 findings, 5 of them in surface that pre-dated the repair
`[P]`. Capture from that generation at the region top is 0.

Required-size is `unavailable` for all three holdout generations: Rust has no AST
span mapping in a non-build lane, and MW-g2's bound findings (`W1` `F1`–`F6`) map
onto shared readers spread across the whole `codefire` file rather than onto
enclosing symbols.

### 5.2 I1 on the lineage holdout

**Executed differential `G`** (`[R]`, this lane): `_HTTPMergeRequestLane.close`
with an injected borrow-close failure.

| tree | outcome | `_borrows` after fault | retry |
|---|---|---|---|
| `21fe751` (predecessor) | `OSError` raised, `_closed = False` | `[]` — the failed borrow was **dropped** | second `close()` returns cleanly and re-releases nothing |
| `6dd4527` (candidate) | same erase-then-fail shape, plus four new scalar `setattr(…, 0)`-before-`release` sites `[P]` | `[]` | same |

The predecessor exhibits the exact ownership loss `W2-F5` names, and it is
observable before `6dd4527` was frozen. `W2-F5` is `I1-B` capture.

| class | n | I1-A capture | I1-B capture | not captured | unavailable |
|---|---:|---:|---:|---:|---:|
| MW-g2, locus pre-existing | 5 (`F1`, `F2`, `F3`, `F4`, `F5`) | 0 | **5** | 0 | 0 |
| MW-g2, locus new | 1 (`F6`) | 0 | 0 | 1 | 0 |
| MW-g2, doc surface | 1 (`F7`) | 0 | 0 | 1 | 0 |
| FD-g2, locus new | 1 (`NOTE-1`) | 0 | 0 | 1 | 0 |
| FD-g2, locus pre-existing | 2 (`NOTE-2`, `NOTE-3`) | 0 | 0 | 0 | **2** (Rust, needs execution) |
| FD-g3 | — | — | — | — | **unavailable** (no report) |
| **total** | **10** | **0** | **5** | **3** | **2** |

I1 capture on the lineage holdout is **5 of 10, unavailable 2**; within its target
class (pre-existing locus) it is **5 of 5 Python and 0 of 2 Rust (both
unavailable)**. Calibration was 10 of 13 in-class with 3 unavailable. The in-class
figure replicates. Noise 0, false positives 0, new author artifacts 0.

### 5.3 I4 on the lineage holdout

Holdout oracle universe, enumerated the same way as §4.6: `FD2` 8 deliberate
regressions + 2 adjudicated scope/shape rows, the File Doctor generation-3 launch
gate, `W2` 10 mutation/vacuity rows, `W1`'s `run_boundary` oracle = **22 rows**.

| rule | applicable | rejects | capture | noise | duplicates a bound clause | calibration comparison |
|---|---:|---:|---:|---:|---:|---|
| `E1` | **1** | 1 | **1** — `cargo test … --locked live_file_doctor`: exit 0 in 0.267 s, **0 selected / 794 filtered**, from a binary belonging to a different Rust worktree `[R]` (index row + audit §5.5) | 0 | 0 | 0 applicable in calibration; **this is the rule's only corpus instance and the pre-registered holdout is where it lives** |
| `E2` | 0 | 0 | 0 | 0 | 0 | 1/3 calibration; no holdout instance |
| `E3` | 1 | 1 | 0 | **1** — `FD2`: "the tests use mutation hooks placed after the last observation, **which is exactly the shape** [the contract requires]"; `E3` read literally rejects an oracle its own reviewer found correct | 0 | 4/6 calibration with 1 noise — **noise replicates** |
| `E4` | 22 | 1 | **1** — `W2`: `tests…:667` "makes the apply reread defect an expected value" while the production docstring at `:608` claims every read record is charged exactly once | 0 | 0 | 6/66 calibration |
| `E5` | 18 | 7 | **7** — `W2`: 2 of the 7 required mutation classes survive (policy-owner, nonce-owner omissions), and 5 named properties have **no** defending mutation (physical-read equality, prepublication sealing, copy reducer recovery, cleanup retry ownership, replay nonce atomicity) | 0 | **7** (`W2`'s packet already bound the seven classes `[T]`) | 16/45 calibration, 23 duplications — **replicates, including the duplication problem** |
| `E6` | 2 | 0 | 0 | 0 | 0 | 3/4 calibration. Here `W2`'s old-target reversal **passes** `E6` (23 failures vs 4 errors against `21fe751`; 33 vs 27 against `e2a218e`) where `U2`'s was rejected — the rule discriminates rather than always firing |
| `E7` | 1 | 1 | **1** — `W2`'s seven required mutation classes are a frozen list, and the five undefended properties lie outside it | 0 | 0 | 2/4 calibration — **replicates** |
| `E8` | 3 | 1 | **1** — `FD2` residual-4: gate wait/exclusion semantics never exercised, owner outside the write set | 0 | 0 | 6/8 calibration |
| `E9` | 0 | 0 | 0 | 0 | 0 | 1/4 calibration; no holdout instance |

Lineage-holdout totals: **22 rows, 12 rejections, 11 captures, 1 noise, 0 false
positives, 7 duplications, 0 unavailable rows**. Distinct rows rejected: 11 of 22.

`FD2`'s own eight deliberate regressions all produced targeted failures and are
rejected by no rule — correctly. That is 8 of 22 rows on which the whole rule set
is silent and should be.

### 5.4 I5 on the lineage holdout

| gen | driven by | only-routable? |
|---|---|---|
| MW-g2 | `W1` `F1` (2.3×–3.4× ledger escape, measured), `F3`/`F5` (permanent `503` with escrow retained), `F4`/`F6` — `DEFECT-REPRODUCED` ×4 plus `ORACLE-VACUOUS` | no |
| FD-g2 | `FD1`'s eight non-Must residuals on a candidate the same reviewer **accepted with zero Must findings**; three are explicitly "inherited from the previously accepted accounted implementation that the HTTP route already ships" — `ORACLE-VACUOUS` ×2, `SCOPE-BOUNDARY` ×6 | **yes** |
| FD-g3 | `FD2`'s three `NOTE`s, each declined as a defect by its own author: `NOTE-1` "Aborting is the safer of the two … a boundary the manager should be aware of **rather than a defect**"; `NOTE-2` "**mirrors the pre-existing accepted read path**", "No test can currently observe it"; `NOTE-3` "I read this as a **scope boundary** … rather than a defect" — `SCOPE-BOUNDARY` ×3 | **yes** |

**2 of 3 lineage-holdout successor generations were driven only by routable
states**, 0 unavailable. This is the audit's §9 `I5` guess about File Doctor
generation 3 confirmed **out of sample**, and it adds File Doctor generation 2,
which the audit did not name. Combined with calibration: **5 of 15, 1
unavailable**.

`CLOSURE-UNPROVEN` was not treated as accepted anywhere in the holdout, and
`ACCEPT_WITH_NOTES` was not invented.

---

## 6. Temporal-holdout scores

Opened last, after §4 was frozen and §5 was scored. These three reports did not
exist in the audit worktree at `c258cbf`.

| subject | report commit | report SHA-256 (recomputed) | exact candidate | verdict |
|---|---|---|---|---|
| Python Upload | `b3de0d7` | `3df1db92d6adcef9d318e23182fb9c62519f6eacb091266ced9616626f64cf1f` | `02e04c7e6458035d9c7d58de4d30b4a12e7cd594` | `REQUEST_CHANGES` |
| KGI/KPI | `5760c98` | `54a4244a8e563b48b2f086a51433ea858996c08b0104c0309f2c8445ce6706b2` | `0944bf3deba9e753132b3f71716b5c009a8fc30f` | `REQUEST_CHANGES` |
| Source-keyed Cargo gate | `b589864` | `212d391c8fdb23c9178e6f8ddbb68cfc7087a50950049f137d19d342a5e2e259` | `aaa39b61b37df273fe4d57e274ed8e1321e788ad` | `REQUEST_CHANGES` |

All three digests match the disposition index `[P]`.

### 6.1 The three deltas, recomputed

| subject | predecessor → candidate | `D` (reviewed surface, ins) | del | new top-level surface | position vs the frozen region [731, 1,555] |
|---|---|---:|---:|---:|---|
| Upload gen 5 | `467b0c2` → `02e04c7` | **455** (`codefire` 167 + tests 288) | 25 | **0** new top-level symbols; 9 new nested methods `[P]` | **below** the region floor |
| KGI/KPI gen 3 | `0ae583e` → `0944bf3` | **3,066** | 640 | `ai_contracts.py` 3,162 → 4,265 lines | **far above** the region |
| Cargo gate gen 2 | `bfbbc2b` → `aaa39b6` | **610** (5 tool files) | 63 | 3 new defs in `readonly_source_exec.py` | **below** the region floor |

### 6.2 I3 on the temporal holdout — the intervention's worst result

**Upload gen 5 is the decisive case, and it goes against I3.** `02e04c7` is
already exactly what I3 prescribes and what audit §10 change 1 asked for: a
bounded additive repair of 455 insertions that introduces **zero new top-level
public surface** `[P]`. It nevertheless returned `REQUEST_CHANGES` with `HIGH-2`
still open, and the finding's locus is `_AccountedUploadCommitSource`, which
occurs 3 times at `76baff8`, 3 at `467b0c2`, and 3 at `02e04c7` `[P]` — **code
that pre-dates the repair by two generations**. A delta budget and a
new-public-surface refusal have nothing to bite on. I3 capture: **0 of 1**.

The Cargo gate repeats the pattern at a smaller scale: 610 insertions, below the
region floor, three `MUST` findings. Two of the three (`MUST-2` recorded isolation
identity, `MUST-3` retention accounting) are in text and code the candidate itself
added, but neither would have been prevented by a size bound — the candidate was
already small. I3 capture: **0 of 3**.

KGI/KPI gen 3 is the one subject where I3 would have bitten: 3,066 insertions,
`ai_contracts.py` grown by another 1,103 net lines, and 7 of its 9 findings
(`R1`–`R7`) located in validator code created or rewritten at this generation
`[P]` (`superseded_by` 5 → 8, `artifact_subject` 0 → 3, `tukey` 0 → 1). I3 capture
at any `B` in the frozen region: **7 of 9** (upper bound).

Temporal-holdout I3 totals: **capture 7 of 13 findings across 3 subjects**, and
the capture is entirely concentrated in the one subject that was over budget.
**Two of three subjects were already inside or below the frozen region and
recurred anyway.** Generations-to-terminal remains `unavailable` (3 of 3 subjects
still open). Required-size remains `unavailable` (3 of 3).

### 6.3 I1 on the temporal holdout

**Upload gen 5 contains an executed predecessor differential run by the report
itself.** Mutation row `mB` — "restore the pre-repair erase-before-release
ordering in `_AccountedUploadSourceMemory.close`" — returns exit 1 with three
failures, one per public path `[R]`. That is I1-A performed live, and it detected.
Rows `mC`/`mD` — the two the prior review found vacuous — now fail
**pre**-allocation with "0 of 352 temporary_bytes proved".

The open `HIGH-2` is I1-B's case. `_AccountedUploadCommitSource.close`
(`codefire:7306`–`7328`) consumes `self._session = None; self._memory = None`
before anything fallible runs, and `_open`'s constructor unwind does the same at
`:7289`–`:7292`. The class is byte-present at the predecessor `467b0c2` `[P]`, so
deriving the predecessor's cleanup ordering exposes the loss before `02e04c7` was
frozen. The reviewer's own counterexamples show `{'retained_bytes': 11456}`
surviving retry and `gc.collect()` — **the same 11,456-byte owner my §4.5 row `R2`
reproduced two generations earlier**. I1 capture: **1 of 1**.

This is the **fourth independent instance** of the erase-before-fallible-release
family (`IV-06`), after MR-list `M2-MUST3`, MR-write `W2-F5` and Upload `U3-H2` —
and the first one that appears in a *wrapper* around the owner that the third
instance's repair fixed.

KGI/KPI gen 3: all 7 `R`-findings sit in validator code new or rewritten at this
generation `[P]`; there is no predecessor behaviour to derive. Capture **0 of 9**,
0 unavailable. Source-keyed Cargo gate: the predecessor `bfbbc2b` is a WIP
checkpoint, not an independently reviewed candidate, and no finding rests on a
behavioural-parity claim against it. **`not-applicable`**, with that reason,
rather than a manufactured score.

Temporal I1: **1 capture / 10 decidable findings / 1 subject `not-applicable` /
0 unavailable**. In-class (pre-existing locus) it is **1 of 1**.

### 6.4 I4 on the temporal holdout

| rule | applicable | rejects | capture | noise | evidence |
|---|---:|---:|---:|---:|---|
| `E1` | 3 | 0 | 0 | 0 | all three subjects ran non-zero-selected, source-bound gates |
| `E2` | 0 | 0 | 0 | 0 | no scaling-axis oracle adjudicated |
| `E3` | 2 | 2 | **2** | 0 | Upload `mA`: the removed-charge shape reported a **late** "memory claim underflow" instead of the boundary error; `mC`/`mD` now refuse **pre**-allocation. `E3` is the exact property the closure turned on |
| `E4` | 3 | 2 | **2** | 0 | Cargo gate `MUST-1`: the helper's comment "the isolation-tool descriptors are all closed first" and the doc's "retaining only the descriptors the child environment itself names" are **both false** `[R]`; Cargo gate `MUST-3`: the retention claim "asserts an exactness and a budget flow that neither exists nor holds" |
| `E5` | 3 | 0 | 0 | 0 | Upload's four-row matrix, KGI's 55+15 replay and the Cargo gate's six checkpoint controls all delete or restore a real production line and all change a result |
| `E6` | 1 | 0 | 0 | 0 | Upload's mutations restore byte-identical trees and fail behaviourally |
| `E7` | 2 | 2 | **2** | 0 | KGI/KPI: "Passing 108/108 again verifies the implemented contract, **not the missing contract**", with seven named uncovered classes. Cargo gate `MUST-1`: `test_readonly_helper_mounts_snapshot_and_drops_namespace_caps` asserts `len(closed) == 4` **over the four descriptors the patch enumerates**, so the assertion passes while the invariant is violated for at least four more — the reviewer's own words: "**A control that can only confirm the change already made is not a negative control for the property**" |
| `E8` | 2 | 1 | **1** | 0 | KGI/KPI: the cohort acceptance fixture itself contains the 120 s-lane / 1000 s-sample contradiction, so it cannot detect `R1b` |
| `E9` | 0 | 0 | 0 | 0 | no ordering-property fixture adjudicated |

Temporal-holdout totals: **7 captures, 0 noise, 0 false positives, 0
duplications**, over an oracle universe of 3 subjects' matrices (Upload 4 rows,
KGI 2 replay instruments + 7 gap rows, Cargo gate 6 checkpoint controls + 2
adjudicated controls = **21 rows**).

Two results matter more than the totals:

1. **`E7` fires twice, on two lanes that share no code with the calibration
   corpus**, and in the Cargo-gate case the reviewer states the rule in their own
   words without having read `E7`. That is the strongest out-of-sample evidence in
   this pilot for any single rule.
2. **The gap I measured in calibration reappears.** §4.6 recorded three
   adjudicated defective oracles that no `E1`–`E9` rule rejects, all of the form
   *the fixture never enters the state the invariant concerns*. KGI/KPI gen 3
   produces a fourth:
   `test_injected_validation_instant_refuses_overdue_claims` "asserts that a
   **backdated** instant is clean, which fixes the current behaviour in place
   rather than flagging it". The same report also records that the *previous*
   instance of this gap — the directory test that "encoded the erroneous
   global-newest behavior as expected" — was fixed. The class recurs; the rule set
   still has no member for it. Measured gap across all three sets: **4 of 175
   oracle rows**.

### 6.5 I5 on the temporal holdout

| subject | state labels | only-routable? |
|---|---|---|
| Upload gen 5 | `HIGH-2` = `DEFECT-REPRODUCED` — "one real `close(2)` failure strands retained debt and makes retry a no-op", reproduced without patching any `release` `[R]`; `MEDIUM-4` closed | **no** |
| KGI/KPI gen 3 | `R1`–`R7` with **13 executable falsifications**, `DEFECT-REPRODUCED` on the control plane | **no** |
| Source-keyed Cargo gate | `MUST-1` = `CLAIM-FALSE` + `ORACLE-VACUOUS` (the reviewer states practical severity is low: "No *capability* is gained that the documented non-hermetic boundary does not already concede. What is defective is **the claim and the control**"); `MUST-2` = `CLAIM-FALSE`; `MUST-3` = `CLAIM-FALSE`; `SHOULD-1`/`SHOULD-2` = `CLAIM-FALSE`; `NIT-2` = `ORACLE-VACUOUS` | **yes** |

**1 of 3 temporal-holdout generations was driven only by routable states.**

Corpus-wide, across calibration and both holdouts: **6 of 18 successor
generations (33.3%) were driven only by states the extended vocabulary would route
rather than block; 1 of 18 unavailable** (Upload gen 2, whose driving report is not
in the indexed snapshot). Numerator 6, denominator 18, unavailable 1.

The Cargo-gate row is the cleanest instance in the whole corpus: a `REQUEST_CHANGES`
in which the reviewer states in the verdict that "none of the ten reviewed
`MUST`/`CRITICAL` classes yields a privilege escalation or an evidence-forgery path
beyond the boundaries the document already states" and that "the gate's fail-closed
posture is intact", and blocks on three claim/control defects. Under the current
vocabulary that is indistinguishable from a reproduced production defect.

---

## 7. Common outcome-measure table

Every cell is numerator / denominator with the `unavailable` count carried. No
percentage appears without all three.

### 7.1 Later-finding capture

| intervention | calibration | lineage holdout | temporal holdout |
|---|---|---|---|
| **I3** (at the frozen region top, `B` = 1,555) | 26 / 30 new-locus, upper bound; 13 pre-existing findings untouched; **unavailable 2 of 12 generations** | 1 / 1 decidable; **unavailable 1 of 3 generations** | **7 / 13**, all from the single over-budget subject; 0 / 4 from the two subjects already inside the region; unavailable 0 |
| **I1** (derivation mode) | 11 / 45 overall, **10 / 13 in-class**; unavailable 3 | 5 / 10 overall, **5 / 5 in-class Python**; unavailable 2 | 1 / 10 overall, **1 / 1 in-class**; 1 subject `not-applicable`; unavailable 0 |
| **I4** (all rules, distinct rows) | 31 / 66 rows rejected, 39 rule-level captures; unavailable 0 | 11 / 22 rows rejected, 11 captures; unavailable 0 | 7 / 21 rows rejected, 7 captures; unavailable 0 |
| **I5** (generations routable) | 3 / 12; unavailable 1 | 2 / 3; unavailable 0 | 1 / 3; unavailable 0 |

### 7.2 Noise, duplication, burden, false positives

| measure | I3 | I1 | I4 | I5 |
|---|---|---|---|---|
| **noise** (outputs matching neither a later finding nor the audit ledger) | not separable from capture: I3 emits one number per repair, and its error is an unmeasured capture *lower bound* of 0, not a false output. Recorded as `[U]` | **0 / 55** — every output is a fact about a committed tree | **4 / 109** rule-level rejections of an oracle its own reviewer adjudicated admissible: `E3` ×2 (`M4` restore-the-proof; `FD2` mutation-hooks-after-last-observation), `E5` ×1 (`M4` restoration is not a deletion), `E7` ×1 (`U3`'s nine required mutations are themselves a frozen corpus and were decisive) | **0 / 18** — relabelling produces no new claim |
| **duplication** (already required by the bound report or ticket) | 0 — no ticket in the corpus declared a delta budget `[P]` | **3 / 21 reports** already discharge it (`M1`, `M3`, `U3`) `[P]` | **36 / 109** rule-level rejections restate a clause the ticket already bound; `E5` alone accounts for 30 of those | 0 — no report used the proposed vocabulary |
| **author burden** (added required evidence artifacts) | 1 per packet: a declared budget and a new-surface justification. No new executable artifact | **0** new executable artifacts; 10 of 11 calibration captures are a second invocation of a reproducer the reviewer already wrote `[R]` | **5 of 9 rules add a new artifact** (`E2` a multi-point counter table, `E5` one deletion run per protected line, `E7` an out-of-corpus case, `E8` a production call-graph proof, `E9` a multi-fault fixture); `E1`, `E3`, `E4`, `E6` add none | **0** — documentation-only |
| **reviewer burden** (added review obligations) | 2: check the budget, check the new-surface refusal | 1 per parity claim | 9, of which **3 are not mechanically decidable** (`E3`, `E4`, `E7`) and will need reviewer judgement in every application | 1 per finding: assign a state alongside the verdict |
| **false positives** (contradicted by the object database or an executed reproducer) | 0 / 18 generations | **0 / 55** findings, all checked against the object database | **0 / 109** rejections | 0 / 18 |
| **author/reviewer seconds** | `[U]` | `[U]` | `[U]` | `[U]` |

`MET-FLOW-006` (review lead time), `MET-FLOW-014` (rework ratio) and
`MET-FLOW-015` (reviewer correction rate) are defined at
`development-flow-kgi-kpi.md:56`, `:69`, `:70` and have **zero occurrences** in
`flow-metrics/cfr168-recovery-20260730.v1.json`, which records 29 measurements
`[P]`. No intervention's cost can be expressed in seconds by this corpus, and none
is guessed.

### 7.3 The packet-level baseline

The ledger's target is "material reviewer correction on at most 2 of 5 packets".
The observed corpus rate is **17 of 21 `REQUEST_CHANGES`** `[P]`, and it is **20 of
24** at this task base once the three temporal subjects are added — all three are
`REQUEST_CHANGES` `[P]`.

Precisely how each intervention would or would not change that packet-level result:

- **I3 does not change it.** Of the 45 calibration findings, 13 are
  `DEFECT-REPRODUCED` on pre-existing code that no size bound touches, and the
  temporal holdout shows two already-bounded repairs (`D` = 455 with zero new
  public surface; `D` = 610) both returning `REQUEST_CHANGES`. I3 changes the
  *content* of a report, not whether one is written.
- **I1 does not change it either, and is not meant to.** It moves 11 of 45
  calibration findings and 5 of 10 lineage-holdout findings *earlier*, from the
  successor report into the current one. The report count is unchanged; what
  changes is that the finding is bound before the candidate is frozen rather than
  after.
- **I4 does not change it.** Its 39 + 11 + 7 captures are all evidence-side; every
  one of them still produces a `REQUEST_CHANGES` unless the oracle is fixed
  pre-freeze, which is the same relocation I1 achieves.
- **I5 is the only intervention that changes it, and only by removing
  generations.** 6 of 18 successor generations were driven only by routable
  states. Had those six been routed rather than blocked, 6 fewer candidate
  generations — and therefore up to 6 fewer reports — would exist. Against the
  17-of-21 observation that is a change from 17/21 to at most 11/15; against the
  2-of-5 target it is still a failure. **No intervention in this pilot brings the
  corpus to the target.**

---

## 8. Threats to validity

1. **The lineage holdout is not blind.** The audit read `python-mr-write` and
   `rust-file-doctor` in full while deriving the controls this pilot scores
   (§2.4). Their capture figures are less biased than calibration's and more
   biased than the temporal holdout's. Where the three disagree, the temporal
   figure should be believed.
2. **The disposition index's rows for the three temporal subjects were visible
   before the freeze** (§2.4). No calibration choice uses them, but I cannot prove
   a negative about my own attention, so the temporal holdout is "clean at the
   report-body level", not "clean".
3. **I3's capture is an upper bound with a lower bound of 0, and nothing between
   them is measured.** Every I3 number in this report credits a boundary with
   preventing all defects in surface the repair would have been forbidden to
   write. A bounded repair that writes the same defect one generation later
   scores identically in this corpus and would score 0 in reality. This is the
   single largest weakness in the I3 result and it is not closable retroactively.
4. **Generations-to-terminal is unavailable on 9 of 9 surfaces.** No repair in the
   corpus ran under a declared budget, so the cost side of I3 — the displacement
   at 0.35–0.64 of all successor insertions — has no observation at all.
5. **Rust is measured negatively, never positively.** Symbol absence at a
   predecessor tree is a sound non-build proof that I1 *cannot* capture a finding.
   The converse needs execution, and 5 findings across calibration and the lineage
   holdout stay `[U]` for that reason. If Rust surfaces recur differently from
   Python ones, this pilot cannot see it — the same censoring the audit recorded.
6. **The I4 oracle universe is a set boundary I chose.** 66 + 22 + 21 = 109 rows,
   enumerated per report so the denominator is checkable, but a different reader
   drawing the boundary elsewhere would get different totals. The per-rule
   *ordering* (`E5` ≫ `E4` ≈ `E8` > `E7` > `E2` ≈ `E9` > `E1`) is robust to that
   choice; the absolute counts are not.
7. **My finding total is 140, the audit's is 143.** The successor subset, on which
   every measurement depends, is identical at 51. The disagreement is confined to
   first-generation accounting.
8. **My generation and successor-delta counts are 24 and 15, the audit's are 22
   and 14, and the audit's own §3.1 delta figures are not reproducible under any
   single rule** (§4.1). Anyone comparing this pilot to the audit must use one
   enumeration or the other throughout, not mix them.
9. **State assignment in §4.7 and §6.5 is my reading `[I]`** over `[P]`/`[R]` rows.
   Two calibration findings and nine `S2` dispositions fit no proposed state at
   all, which is a result rather than a threat, but it means the state
   distribution is not exhaustive.
10. **Everything here is observational.** No live lane ran under any intervention.
    The pilot answers "would the control have seen this?", never "would the lane
    have gone better?".

---

## 9. Intervention decisions

These are experiment-readiness labels, not adoption. A promising intervention
still needs a small live experiment and independent review before becoming policy.

### I3 — bounded repair delta with a new-public-surface refusal → `REVISE_AND_REPLAY`

Calibration produced a real, non-preselected region (`B` ∈ [731, 1,555]) with two
existence proofs inside it and none below it, and the two terminal-accepted
generations in the corpus are the two smallest deltas — reproduced with my own
numbers `[P]`. But the temporal holdout is decisive against adopting it as
specified: **Upload gen 5 was already a 455-insertion additive repair with zero
new top-level public surface and still returned `REQUEST_CHANGES`, on a locus two
generations old**; the Cargo gate did the same at 610 insertions. Capture there is
0 of 4. Add an unmeasured capture lower bound of 0, displacement of 0.35–0.64 with
no cost model, generations-to-terminal `unavailable` 9 of 9, and the test-evidence
half of every required-size denominator `unavailable` 18 of 18. The replay this
needs is a *prospective* one: bound a real repair and measure generations, not
findings.

### I1 — predecessor-behaviour re-derivation → `PROMISING_FOR_LIVE_EXPERIMENT`

The only intervention that survives both holdouts on its target class: 10 of 13
calibration, 5 of 5 lineage-holdout Python, 1 of 1 temporal. Noise 0 of 55, false
positives 0 of 55, new author artifacts 0. The pilot also **corrects the audit's
framing**: capture comes from *derivation*, not *differential*. The two decisive
calibration cases (`read_json_accounted`, `_HTTPMergeRequestListOperation._release`)
are byte-identical at predecessor and candidate `[P]`, so a differential sees
nothing while derivation sees the defect immediately — and in the `read_json_accounted`
case it directly contradicts the prose the previous report published about that
same function. The live experiment should therefore bind the *derivation*
obligation ("state the predecessor's behaviour from the predecessor tree"), not a
differential one. Caveat to carry in: 3 of 21 reports already do this, so the
experiment's control group is 18 lanes, and the Rust arm is blocked until a
predecessor build is available.

### I4 — oracle-admissibility rules `E1`–`E9`, scored separately → `REVISE_AND_REPLAY`

Per rule, never collapsed:

| rule | calibration | lineage holdout | temporal holdout | replicates? | mechanically decidable? |
|---|---|---|---|---|---|
| `E1` | 0 applicable | **1 / 1** | 0 applicable | single instance, captured | yes |
| `E2` | 1 / 3 | 0 applicable | 0 applicable | **no holdout instance** | yes |
| `E3` | 4 / 6, noise 1 | 0 / 1, **noise 1** | 2 / 2 | capture yes, **noise replicates** | **no** |
| `E4` | 6 / 66 | 1 / 22 | 2 / 3 | **yes** | **no** |
| `E5` | 16 / 45, dup 23 | 7 / 18, dup 7 | 0 / 3 | **yes, and so does the duplication** | yes |
| `E6` | 3 / 4 | 0 / 2 | 0 / 1 | discriminates rather than always firing | yes |
| `E7` | 2 / 4, noise 1 | 1 / 1 | **2 / 2** | **yes, strongest out-of-sample** | **no** |
| `E8` | 6 / 8 | 1 / 3 | 1 / 2 | **yes** | partly |
| `E9` | 1 / 4 | 0 applicable | 0 applicable | **no holdout instance** | yes |

`E4`, `E5`, `E7` and `E8` replicate across at least two of the three sets with
zero false positives. `E1` has exactly one corpus instance and the pre-registered
holdout happens to be where it lives — captured, but *n* = 1. `E2` and `E9` never
recur outside calibration. `E3` is noisy in two of three sets. `E5`, the single
largest contributor, duplicates an already-bound ticket clause on 30 of its 66
applicable rows — it is mostly an enforcement gap, not a missing rule. And the set
has a **measured coverage hole of 4 of 175 oracle rows** — oracles whose fixture
never enters the state the invariant concerns — that recurred in the temporal
holdout after being fixed once. Adopting `E1`–`E9` as a nine-item mechanical
checklist would institutionalise exactly the enumerated-corpus trap the Cargo-gate
reviewer independently named ("a control that can only confirm the change already
made is not a negative control for the property"). Revise the set, then replay.

### I5 — separate evidence state from routing state → `PROMISING_FOR_LIVE_EXPERIMENT`

The measurement replicates in both holdouts and is the only one that removes
generations: **6 of 18 successor generations (numerator 6, denominator 18,
unavailable 1) were driven only by states the extended vocabulary would route
rather than block** — Doctor+Branch gen 2, Rust HTTP gens 2 and 3, File Doctor
gens 2 and 3, and the source-keyed Cargo gate. Cost is documentation-only: 0 new
artifacts, 1 obligation per finding, 0 noise, 0 false positives. It confirms the
audit's File Doctor gen-3 guess out of sample and **rejects** its MR-list gen-3
guess (`ML-g3` was driven by four `DEFECT-REPRODUCED` findings `[R]`).

Two conditions the live experiment must carry, both measured here rather than
assumed. First, the proposed §12.2 vocabulary is **incomplete**: `S2` `R2`/`R6`
and the nine `S2` `F1`–`F9` dispositions are *source-proven by a lane that
executed nothing*, which is neither reproduced, nor injection-only, nor
claim-only, nor closure-unproven. A state for that is missing and must be added
before the vocabulary is used, or those rows will be forced into a wrong bucket.
Second, `CLOSURE-UNPROVEN` was never treated as accepted anywhere in this pilot
and `ACCEPT_WITH_NOTES` was not invented; the live experiment must preserve both
properties, because the whole benefit is a routing change and none of it may
become an integration-authority change.

---

## 10. Explicitly undecided template properties

The packet forbids deriving a review-item count, checklist length, template depth,
or field count unless all four prerequisites hold. Their status, measured:

| prerequisite | status |
|---|---|
| 1. a stable I3 capture/burden region on calibration **and both holdouts** | **not met.** The region is stable on calibration and survives the lineage holdout weakly, but the temporal holdout scores 0 of 4 on the two subjects already inside it, and the burden side is `unavailable` on 9 of 9 surfaces |
| 2. per-rule I4 capture/noise with no hidden aggregate | **met.** §4.6, §5.3, §6.4 and §9 publish every rule separately, with denominators; no total is substituted for a rule |
| 3. the I5 fraction of generations driven only by routable states | **met.** 6 / 18, unavailable 1 |
| 4. a field-level mapping from each retained field to a measured causal class | **not met.** No field set exists to map, and §4.6/§6.4 show 4 of 175 measured oracle defects that no proposed rule covers — so a field set derived today would be provably incomplete |

Prerequisites 1 and 4 are unavailable. Therefore, and stated as the packet
requires rather than inferred:

- **the number of items in any future review checklist is undecided;**
- **the depth and structure of any future review/repair template are undecided;**
- **the number of fields in any future detailed review/repair contract is
  undecided;**
- **no repair-delta limit is named.** §4.3 publishes a curve and a region
  `B` ∈ [731, 1,555]; §6.2 shows two temporal subjects inside it that recurred
  anyway. A manager may read the curve; this pilot does not select a point on it.

The one field-level statement the data does support, offered as evidence and not
as a schema: of the 51 successor findings, the single attribute that had to be
recomputed by hand from the object database for every measurement in this report
is **whether the finding's locus existed in the reviewed diff's predecessor tree**.
It is a one-command determination at candidate-freeze time, it partitions 43 of 45
calibration findings cleanly, and it is the input `I1`, `I3` and `I5` all require.
That is one measured causal class with one measured field — not four
prerequisites, and not a template.

---

## 11. Non-authorization and write boundary

This pilot does **not** accept, reject, or integrate any candidate; change any
recorded verdict, the disposition index, the recovery ledger, or any existing
report; adopt any intervention; fix any checklist size, template depth, or field
count; authorize CFR-168, CFB-136, CFB-138, Install, Dogfood, or release; launch
or stop any agent, provider, or scheduler; or modify Cargo, Rust, network, or
credential state.

`[P]` No Codex, Fugu, subagent, network call, Cargo/Rust command, install, or
dogfood was used. Product bytes at this task base remain byte-identical to
`e2a218e`. The commit carrying this report adds exactly one path:

```text
docs/development/ai/swarm-results/cfr168-review-intervention-retroactive-pilot-claude-20260731/pilot.md
```

Adopting `I1` or `I5` requires its own packet, a small live experiment, and an
independent review. `I3` and `I4` require revision and a further replay before
either is worth a live lane.
