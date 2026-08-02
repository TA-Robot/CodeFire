# CFR-168 File Doctor Composition Input — Impact/Actionability Adjudication

Adjudication date: 2026-08-02

Adjudicator: Claude Opus 5 (`claude-opus-5`), independent impact/actionability
adjudicator. Fresh session and fresh auto-worktree; not the implementer of
either candidate and not either prior File Doctor reviewer.

Batch: `cfr168-file-doctor-composition-input-adjudication-claude-20260802`

Mode: report-only manager adjudication artifact under
`docs/development/ai/validation-report-template.md` §"Manager adjudication".
It references immutable reviewer bytes and does not edit them.

Question: choose the File Doctor composition input for the CFR-168
accepted-input composition — accepted candidate `A` = `1a04490…`, or hardening
successor `H` = `cd130e5…`.

**Decision: `SELECT_ACCEPTED_A`.**

No current must-fix trigger is met by NOTE-1, NOTE-2 or NOTE-3. All three are
terminal `nonblocking_improvement`. `H` therefore has no repair authority
behind it, no fresh exact review, and no valid acceptance-profile evidence, and
is excluded from the current composition root.

---

## 1. Bound identities

### 1.1 Canonical launch base

| Field | Exact value |
|---|---|
| Launch base `HEAD` | `6db7944b26399c3bc43867f12ee9b3f260988621` |
| Launch base tree | `69b9f7414ef0104a5782493f8731e1cd3636795f` |
| Launch base subject | `docs(cfr168): correct remaining composition stage authority labels` |
| Worktree state at start | clean; `git status --porcelain` empty |
| Branch | `claude-agent/cfr168_file_doctor_composition_input_adjudication_20260802` |

This adjudication commit is a direct, non-merge child of that base.

### 1.2 Candidate A (accepted)

| Field | Exact value |
|---|---|
| Commit | `1a04490806b59f8c39b5c1cf03f42d06aa750a98` |
| Tree | `eafb2cbf20980944d16086a972100c321a8b56be` |
| Parent | `dbeaac1526acc8c2e102ee2b137e8f2a708890b0` |
| Subject | `Close every file Doctor residual on the accounted live route` |
| Accepted predecessor (ancestor) | `52a025ecda3fe2f5ad8b5b2a485562e7c16ab6de` |
| Changed paths vs `dbeaac1` | 5 (`crates/codefire-cli/src/remote/doctor.rs`, `docs/development/change-playbook.md`, `docs/development/command-source-trace.md`, `docs/development/source-architecture.md`, `docs/development/source-data-model-catalog.md`) |
| Numstat vs `dbeaac1` | `5 files changed, 2132 insertions(+), 266 deletions(-)` |
| `doctor.rs` blob / SHA-256 | `410d648f933ab796cd26e2822b3313a42014a73b` / `465ac999bb7eeb22775737a5f17ec30a8b8249aacefc50c8ab4e3e5fb6d2cf0f` |
| Patch digest `dbeaac1..A`, plain `git diff --binary` | `5f150dbe37b61b592fa5a52e9ce72a4367ba7c8de2c09eba79390a1c37427fac` |
| Patch digest `dbeaac1..A`, `ai_contracts.canonical_git_patch` | `671c3826e3b5e9883892b55cc9680ff87fdaa3e8ee6b8a8502f5169ea183f24d` |

The plain-`--binary` value reproduces the accepting review's recomputed digest
exactly. Both methods are recorded because the reviewer used the plain form and
`capture_candidate_identity.py` uses the canonical form; they are different
byte streams of the same delta, not a discrepancy.

`A` is **not** an ancestor of the launch base: no File Doctor product byte is
integrated.

### 1.3 Acceptance report

| Field | Exact value |
|---|---|
| Report path | `docs/development/ai/swarm-results/cfr168-rust-file-doctor-zero-residual-rereview-claude-20260730/review.md` |
| Report blob | `c6a052a3362bc35b8833c6a2689eaf82a04f3c94` |
| Report file SHA-256 | `f7c52b05e312cc131fdfc2e2c0d9d486f5ee372b5ec93a85fdb44a95fb3ce1b9` |
| Report bytes / lines | `32669` / `484` |
| Report commit on the candidate lineage | `1c555ff3f426d7fa6874db92ceb71515a8d908e6` (parent `1a04490…`, one added path, report-only) |
| Same bytes on the canonical lineage | `c22a78aa025e4fd2bf38d335f9665b781e4f4714` (parent `367aa7a3f3aa957ec9f0e71ff951324ad9668ecf`), **ancestor of the launch base** |
| Terminal verdict | `ACCEPT` — 0 MUST, 0 SHOULD, 3 NOTE, 1 contract-excluded item |

Both report commits carry the identical blob `c6a052a3…`. The acceptance
evidence for `A` is therefore already durably published in canonical history,
even though `A`'s product bytes are not. `1c555ff` satisfies the report-only
topology requirement (report-only commit, direct child of the exact candidate,
write set disjoint from the candidate's).

### 1.4 Hardening successor H (not accepted)

| Field | Exact value |
|---|---|
| Commit | `cd130e544af0476162be153db58a0a840b75b134` |
| Tree | `2cf767fa00337e167b8a54e8bd20b6af5558b8df` |
| Parent / repair base | `1e5a6f9467b90251b656e240f8c9b4f4700468a8` (`docs(cfr168): freeze File Doctor review-note closure`) |
| Subject | `Close the three File Doctor review notes on the live route` |
| Ancestry | `A` is an ancestor of `H`; range `A..H` = `1c555ff`, `1e5a6f9`, `cd130e5` |
| `doctor.rs` blob / SHA-256 | `34fd05a8a1609f00bd2b6ff31baa623e8d952c45` / `3ebe81c90075ca21db7ca540acf6995a07f593d366eb1fbd124f6b2455558c65` |
| H's own changed paths | the same 5 paths as `A`; `1873` added / `282` deleted |
| Canonical patch digest `1e5a6f9..H` | `84dbf05bced17d9b6aa27e90cf900c57e49338f1835340ae283256613ce0d02a` (recomputed; matches the frozen candidate card) |
| Review status | **none** — no report exists for `H` |
| Candidate-card status | `BLOCK` overall; `READY_FOR_REVIEW` identity-only |

`H` is **not** an ancestor of the launch base either.

### 1.5 A..H delta

Canonical patch digest (`ai_contracts.canonical_git_patch`, i.e.
`git diff --no-ext-diff --no-textconv --no-renames --binary --full-index --src-prefix=a/ --dst-prefix=b/ A H --`):

```text
603488eed14a1f5d4e3249855f312d8b968bda9d811bf9e89c8ae685d1936629
```

Plain `git diff --binary A H | sha256sum`:

```text
dd378748e79d9f1b40d236ab2d5a4ee57629cd51c2bd70aa19c332702f259bb5
```

Numstat: `7 files changed, 2517 insertions(+), 291 deletions(-)`.

Exact `A..H` path set (7 paths; 5 modified by `H` itself, 2 added by the
intermediate report-only and packet-freeze control commits):

| Status | Path | Origin commit in `A..H` |
|---|---|---|
| `M` | `crates/codefire-cli/src/remote/doctor.rs` | `cd130e5` |
| `M` | `docs/development/change-playbook.md` | `cd130e5` |
| `M` | `docs/development/command-source-trace.md` | `cd130e5` |
| `M` | `docs/development/source-architecture.md` | `cd130e5` |
| `M` | `docs/development/source-data-model-catalog.md` | `cd130e5` |
| `A` | `docs/development/ai/swarm-results/cfr168-rust-file-doctor-zero-residual-rereview-claude-20260730/review.md` | `1c555ff` |
| `A` | `docs/development/ai/swarm-batches/cfr168-rust-file-doctor-review-note-closure-repair-20260730.md` | `1e5a6f9` |

`H` writes no product path that `A` does not already write. There is no
product-scope expansion in the path set; the expansion is inside `doctor.rs`.

---

## 2. Evidence inspected

All digests below were recomputed in this worktree.

| Evidence | Path | Commit / blob | SHA-256 |
|---|---|---|---|
| Accepted rereview report (complete, 484 lines read) | `…/cfr168-rust-file-doctor-zero-residual-rereview-claude-20260730/review.md` | `1c555ff` and `c22a78a` / `c6a052a` | `f7c52b05…` |
| Zero-residual repair contract for `A` | `…/swarm-batches/cfr168-rust-file-doctor-zero-residual-repair-20260730.md` | in launch base | `732f070322f5452619c8a492b34174399a2aa9d06935440cbefbc57abe6c7375` |
| Manager packet authorizing `H` | `…/swarm-batches/cfr168-rust-file-doctor-review-note-closure-repair-20260730.md` | added by `1e5a6f9`; blob `e27d4db` at base | `4b94d8f5e811ed968d9d9cefcbf5fd244b6847b09566cee59fc2bb36b03940c1` |
| `H` implementation manifest | `…/swarm-batches/cfr168-rust-file-doctor-review-note-closure-repair-20260730.json` | in launch base | `120e4226bf30f28a3e4fb6d492c8721d434e7f4ab89cf093aebeb338814cf7c3` |
| Candidate-ready evidence card (`BLOCK`) | `…/swarm-results/cfr168-rust-file-doctor-review-note-closure-repair-candidate-ready-20260731/candidate.md` | added `8d5e729` / blob `44ba833` | `6809216e1da16f700541ea12eba5e74ea6a03199fa102bfd98c27f0feac2ab25` |
| Blocked rereview packet for `H` | `…/swarm-batches/cfr168-rust-file-doctor-review-note-closure-rereview-claude-20260730.md` | added `d73a908` / blob `f7bc263` | `4452739f6d9b3eb06b0d9948775feb3802db6cd8faf3916f19981bd16db0c19e` |
| Review-recursion root-cause audit | `…/swarm-results/cfr168-review-recursion-root-cause-audit-claude-20260731/audit.md` | added `abfee6e` / blob `783a3de` | `2392450e957c8437b323ded492fc7597dc77f5147b74f3056c97c0efd9329780` |
| Non-review throughput bottleneck audit | `…/swarm-results/cfr168-non-review-throughput-bottleneck-audit-claude-20260731/audit.md` | added `c258cbf` / blob `100980f` | `f4e0f00014233f52ec7792accbc1dae61a6f0e7a888866d62e3be3b6405e4b3b` |
| Review-bottleneck open-item audit | `…/swarm-results/cfr168-review-bottleneck-open-item-audit-claude-20260731/audit.md` | added `1b5e06a` / blob `c354c59` | `7b68f57526513a3a9d4c9b5ca0d72163b66fb47002fdeb020d86c580324639d4` |
| Review disposition index | `…/swarm-results/cfr168-review-disposition-index-20260730/index.md` | added `7a93abe` / blob `a0bdfe2` | `8057ab1a85f98d25290fabbc086cb1e7abc76102a38c2a07384f5fcc54d2904a` |
| Open-execution item ledger | `…/swarm-results/cfr168-open-execution-audit-20260731/open-items.v1.json` | in launch base | row `RUST-FILE-DOCTOR`, verdict `REVIEW_BLOCKED` |
| Current CFR-168 closure checkpoint | `docs/development/current-cycle-burn-down-plan.md` | blob `e95e9fc` at base | item 4 names this adjudication |
| Current actionability policy | `docs/development/ai/operating-protocol.md` §"Review Governance" | in launch base | MF-1…MF-6, six dispositions, precedence |
| Current report shape | `docs/development/ai/validation-report-template.md` | in launch base | compact screen + manager adjudication block |
| Source-keyed Cargo gate policy | `docs/development/ai/source-keyed-cargo-gates.md` | in launch base | names `cd130e5` as the demonstrating failure |
| Production source at `A` | `git show 1a04490:crates/codefire-cli/src/remote/doctor.rs` | blob `410d648` | traced, not executed |
| Production source at `H` | `git show cd130e5:crates/codefire-cli/src/remote/doctor.rs` | blob `34fd05a` | traced, not executed |

No Cargo, rustc, rustfmt, Clippy, install, Docker, or heavy validation was run.
No product, test, contract, packet, ledger, lifecycle doc, or existing report
was mutated. This adjudication adds exactly one file.

### 2.1 Governing policy actually in force

`operating-protocol.md` §"Impact is not actionability" fixes six evidence-bound
must-fix triggers `MF-1`…`MF-6` and six terminal dispositions, and states that
repair difficulty, reviewer preference, and maintainability preference cannot
create a requirement. §"Review Governance" states that the reviewer's terminal
verdict is a **proposal, not integration authority**, and that a separate
manager impact/actionability artifact is required.

The string `zero-residual` does not appear anywhere in
`operating-protocol.md`, `validation-report-template.md`,
`project-management.md`, or `current-cycle-burn-down-plan.md`. It survives only
in the 2026-07-30 batch tickets and in audit prose *about* those tickets. The
review-recursion root-cause audit classifies the File Doctor generation-3
driver as `REQUIREMENT_OR_SCOPE_EXPANSION`:

> an `ACCEPT` with zero Must findings nevertheless generates a mandatory
> successor generation under the zero-residual rule

and records that

> Every one of the six non-Must items that drove generation 3 was
> characterized by its own reviewer as a scope boundary, an inherited property,
> or safer-than-before.

The zero-residual mandate is therefore **not** current policy and is not, by
itself, a must-fix trigger. I did not treat the audit's diagnosis as
dispositive either; every note below is adjudicated against `MF-1`…`MF-6` on
exact evidence read at `A`.

### 2.2 Validity of A's acceptance evidence (checked, not assumed)

The false-green class identified for `H` is *cross-worktree artifact reuse from
the shared Cargo target root*: `0 selected / 794 filtered` in 0.267 s from
`/workspace/.cache/cargo-target-shared/debug/deps/codefire_rs-2521b6556af2330d`,
a binary whose dep-info named a different Rust worktree.

`A`'s accepting review did not use the shared root. It used the reviewer
worktree's own `target/`, plus `target/clippy-review` and `target/release-review`,
and reported non-vacuous counts: `remote::doctor` 20/20, `doctor` 131/131, full
CLI binary 787/787, release `remote::doctor` 20/20 (6 m 13 s including build).
Decisively, it applied **eight** single-line production regressions (R1–R8) and
observed a targeted failure for each, then reverted and re-verified the patch
digest, `HEAD`, and clean state. A mutation that changes production bytes and
changes the observed test result is positive proof that the executed binary
contained the candidate's source — the exact property the false-green gate
lacked. `MF-5` is therefore **rejected** for `A`.

Recorded evidence boundaries of that review, carried forward unchanged and not
treated as neutral: workspace-wide `cargo test` beyond the `codefire-cli`
binary target, the Python suites, and cross-runtime parity were not run; gate
wait/exclusion concurrency (`remote/locks.rs`, `stable_lock.rs`, byte-unchanged
by `A`) was not exercised. Both are `partial` evidence availability on surfaces
`A` does not modify.

---

## 3. Production trace at A (static, worktree-independent reads)

Reachability facts established by reading `1a04490:crates/codefire-cli/src/remote/doctor.rs`:

- the `#[cfg(test)] mod tests` boundary is at line **3228**; every construct
  cited below as production is above it;
- the live route is `main.rs:1115` → `run_remote_doctor` → `cf://` branch →
  `run_file_remote_doctor` → `run_file_remote_doctor_with_factory_and_observer(
  new_file_remote_doctor_ledger, InertFileRemoteDoctorObserver)`;
- `.reserve(` occurs exactly **once** in the whole module, at line `5013`,
  inside `mod tests`. No production code on this route opens a
  `DecisionWorkPermit`;
- `ledger.release(` occurs in production at 365, 398, 1030, 1548 and 2889 —
  all cleanup/drain sites, none of which takes a permit.

---

## 4. Per-note adjudication

Line citations are line numbers in `crates/codefire-cli/src/remote/doctor.rs`
at the stated revision.

### NOTE-1 — new production `panic!` on the failure-cleanup edge

**Exact original observation and evidence** (report `f7c52b05…`, §NOTE-1,
verbatim substance): `observe_file_remote_doctor_failure` (`A`:1082) aborts the
process when the drain cannot restore zero ownership. Ordering is
terminalize → drain → notify observer → abort. The reviewer's own
characterisation: "Aborting is the safer of the two given the invariant, so I
record this as a boundary the manager should be aware of **rather than a
defect**." Reproducer named: `file_remote_doctor_failure_cleanup_rejects_unrepairable_ownership`
(`A`:5009), a `#[should_panic]` test that injects a permit.

**Confirmed independently at A.** `A`:1069–1083 — `finish_file_remote_doctor_failure`,
then `enforce_file_remote_doctor_unowned`, then the `Failure` stage observation,
then `panic!("file remote doctor failure cleanup could not restore zero
ownership while handling {error}: {violation}")`. `enforce_file_remote_doctor_unowned`
(`A`:1008–1042) drains `RetainedBytes`/`TemporaryBytes` above the entry baseline
and rechecks with `ensure_file_remote_doctor_unowned` (`A`:976–998), which also
compares `open_permit_count()`.

**Was a production defect actually reproduced on A?** **No.** Three residual
paths to the panic exist and each is an internal-accounting-bug class, not a
user input class:

1. `open_permit_count() != baseline` — unreachable: the baseline is taken from a
   freshly built command-scoped ledger (so `0`), and no production code in the
   module reserves. Only a Store hand-off returning a ledger with an open permit
   could raise it, which is a Store invariant violation.
2. `ledger.release(dimension, residual)` fails, leaving a residual — an
   accounting inconsistency inside the ledger.
3. `current < baseline` (`checked_sub` → `None` → `continue`) — the route
   released *below* its own entry baseline, i.e. it over-released.

None is reachable from any `cf … doctor cf://…` invocation, malformed
repository, permission state, or concurrent mutation. The only exhibited
trigger is the test-only `reserve` at `A`:5013.

**Realistic impact and affected workflow.** Workflow: `cf doctor cf://…`
(read-only diagnosis), `remote::doctor` surface. If the internal invariant were
ever violated, a release build aborts instead of silently continuing —
`operations` adverse (process abort, no structured error), everything else
unaffected: nothing is published on that edge in either design, the project tree
stays byte-identical, and the ledger has already been terminalized. Debug-build
behavior is **unchanged from the accepted predecessor**, whose `debug_assert!`
also panicked; only the release build changed, from silently continuing to
aborting.

**Current must-fix triggers.** `MF-1` rejected — no frozen requirement,
invariant, acceptance criterion or binding guardrail forbids a production
`panic!` on an unreachable internal-invariant edge; no such clause exists in
`AGENTS.md`, `operating-protocol.md`, `change-playbook.md`, or the `A` repair
contract `732f0703…`, and the accepting review PASSed residual 8 on this exact
mechanism. `MF-2` rejected — not reachable, not reproduced on any production
route. `MF-3` rejected — no public API, persisted state, or cross-generation
contract touched; the JSON/text envelope is frozen against golden literals and
unchanged. `MF-4` rejected — `module-boundaries.md:47` asks diagnostics to
"clarify exit code and machine-readable details", which is a module-boundary
guideline, not a frozen exit-code bound, and the path is unreachable. `MF-5`
rejected — the release-build oracle was executed by the reviewer (`--release`
20/20) and R6 proved the drain non-vacuous. `MF-6` rejected — the predecessor
also panicked in debug; the release-build change is strictly fail-closed
tightening, not a reachable material regression.

**Severity independent of actionability:** `low`; materiality `nonmaterial`.
Blast radius `local`. Reachability `unreachable` on the production route.
Evidence availability `available`.

**Terminal actionability disposition:** `nonblocking_improvement` (successor).
A typed terminal invariant failure is genuinely better than an abort for CLI
ergonomics, but no current frozen criterion makes it mandatory. Causal class
`null`; no repair authority.

**H's change classification: contract change.** `H`:1494 introduces
`FileRemoteDoctorOwnershipViolation` and `H`:1667 `fail_file_remote_doctor_edge`,
which returns `CliError::InvalidRepository(…)` **in place of the primary cause**
instead of aborting, identically in debug and release. This is not
behavior-preserving: it changes the debug build (which previously panicked),
changes the release build's outcome class, and changes what the failure edge
returns on the `OpenPermits` class from "abort, guaranteed no continuation" to
"return a typed error and drop the command-scoped ledger". H's own doc
comment carries the safety argument — a stale `DecisionWorkPermit` is "two
private integers bound to a ledger id" and dropping the ledger destroys the
reservation. That argument is plausible on the source I read but is a
correctness claim about `codefire_core` ledger drop semantics that no executed
evidence in this repository currently supports for this route.

---

### NOTE-2 — one transient unreserved allocation per fenced record

**Exact original observation and evidence** (report `f7c52b05…`, §NOTE-2):
`read_doctor_json` allocates the 64-byte hex digest at `A`:1781–1783 before
`DoctorRecordSubjectFence::record` reserves its retained frame at `A`:751;
likewise `path.display().to_string()` at `A`:737 is allocated inside the
measurement call. "The unreserved window holds exactly one such value at a time
and it is charged immediately afterward via `logical_retained_frame_bytes`, so
nothing accumulates. This **mirrors the pre-existing accepted read path** rather
than being introduced by this packet." Reproducer: source reading only — "No
test can currently observe it because the amount is fixed and immediately
charged."

**Confirmed independently at A.** `A`:1779–1783 computes
`pinned_digest = fence.is_enabled().then(|| codefire_util::sha256_hex(&bytes))`
before any claim. `A`:736–746 computes the frame as
`logical_retained_frame_bytes(&path.display().to_string())` +
`logical_retained_frame_bytes(&digest)`, i.e. it measures the projections that
already exist, then applies the reservation at `A`:751–762 to
`DecisionWorkPhase::RecordRead`.

**Was a production defect actually reproduced on A?** **No.** There is no
inaccuracy: because `A` measures the real allocated projection, the charged
frame is exact, not an estimate. The window is one value wide, fixed size, and
closed on the next statement; the fence's `Vec<DoctorRecordSubject>` is bounded
both by the 1 GiB `RetainedBytes` limit and by the 100,000 cumulative
`ObjectRecords` limit that every record read charges. No budget can be exceeded
without being observed, and no state escapes.

**Realistic impact and affected workflow.** Workflow: `cf doctor cf://…` record
scan. Impact categories: `resource` at most `low` — a transient sub-kilobyte
heap value per record document that is charged before the next statement;
`correctness`, `state`, `data`, `authority`, `api_compatibility`, `operations`,
`performance` unaffected. `maintainability` is the real axis: strict
claim-before-allocate is the pattern used elsewhere in the accounted codebase,
and `A` does not follow it here.

**Current must-fix triggers.** `MF-1` rejected — the `A` repair contract
`732f0703…` froze no claim-before-allocate clause for this fence; the pattern
appears in `source-architecture.md` for the Python HTTP Upload lane and the
*local* `doctor::` owners, not for `remote::doctor`'s record fence. `MF-2`
rejected — no reachable failure; accounting is exact. `MF-3`, `MF-4` rejected —
no contract or frozen bound touched. `MF-5` rejected — the reviewer stated
plainly that no test can observe it; that is a recorded observability limit on
an item with no acceptance criterion behind it, not absent acceptance-required
evidence. `MF-6` rejected — the reviewer's own finding is that this mirrors the
pre-existing accepted read path, so it is not repair-introduced.

**Severity independent of actionability:** `none` to `low`; materiality
`nonmaterial`. Blast radius `local`. Reachability `reachable` (every fenced
record) but with no adverse effect. Evidence availability `available`.

I considered `duplicate` — the reviewer's "pre-existing accepted read path"
language points at an earlier accepted locus — but I found no canonical
observation ID for that locus to delegate to, and per the protocol a purported
duplicate without a canonical row is not a duplicate. I also considered
`accepted_implementation_variance`; that disposition requires a frozen clause to
be variant from, and none exists here.

**Terminal actionability disposition:** `nonblocking_improvement` (successor).
Causal class `null`; no repair authority.

**H's change classification: behavior-preserving hardening with a small
risk-expanding accounting addition.** `H`:316 `doctor_display_frame_bytes`
measures `Path::display` into a `ByteLengthWriter` counting sink and `H`:890
`DoctorSubjectFrameClaim::claim` reserves before allocating; the phase is still
`RecordRead` (via `DoctorGrowthSite::RecordScan.phase()`), so phase attribution
is preserved. Two candidate-created effects are *not* purely preserving:
`H`:949 adds a production `DecisionWorkCounter::BytesHashed` count that does not
exist anywhere at `A` (`grep -c BytesHashed` at `A` = 0), and `H`:630 adds a new
production observation stage `FileRemoteDoctorStage::RecordSubjectClaim`. The
`BytesHashed` count is bounded by the profile's `CounterValues` maximum, which
`local_v1` sets to `u64::MAX - 1` (`crates/codefire-core/src/work_budget.rs:269-270`),
so the new typed failure mode is practically unreachable — but it is a new
ledger-snapshot effect on the live route that no report has reviewed.

---

### NOTE-3 — a document unreadable at scan time is not fenced

**Exact original observation and evidence** (report `f7c52b05…`, §NOTE-3):
`read_doctor_json` returns `DoctorJsonRead::Invalid` without recording a fence
subject when `fs::symlink_metadata` fails (`A`:1729–1731) or when the bounded
read fails (`A`:1767–1777); only the *unparseable* case is fenced
(`A`:1797–1799). "A document that was unreadable during the scan and becomes
readable before publication would therefore leave its 'invalid'/absent
conclusion unrevalidated." Proposed reproducer: make `branches/main.json`
unreadable, run the live route, restore permissions before the `RecordFence`
stage; the report still publishes. Reviewer's own characterisation: "There are
no parsed bytes to pin in this case, so I read this as a **scope boundary** of a
content-digest fence rather than a defect."

**Confirmed independently at A.** `A`:1729–1731 returns
`Ok(DoctorJsonRead::Invalid(CliError::Io(error)))` directly from a failed
`symlink_metadata`, before any reservation and with no fence interaction.
`A`:1765–1777 releases the scratch and returns `Invalid` on a failed
`read_file_nofollow_bounded_with_identity`. `A`:1797–1799 is the only branch
that calls `fence.record(...)`, and only when `pinned_digest` exists — i.e.
only when bytes were read.

**Was a production defect actually reproduced on A?** **No.** The scenario is
not a wrong answer on a stable repository; it requires an external actor to
change a document's readability *during* a read-only diagnosis run. What `A`
publishes is a truthful report of what the scan observed. The named reproducer
was described but not executed by the accepting reviewer, and I did not execute
it (heavy validation is out of scope here); reproducibility is therefore
`not_attempted` and is recorded as such rather than as `unaffected`. That
unknown is not acceptance-critical, because the *effect* — a diagnostic that is
accurate as of scan time and stale as of publication time under concurrent
mutation — is established from the source regardless of whether the race is
exhibited.

**Realistic impact and affected workflow.** Workflow: `cf doctor cf://…` in
quick, full, or unreachable-root mode, on a project whose branch or
merge-request document changes readability mid-run. Effect: an
"invalid record" diagnostic may be published for a document that is readable by
publication time. `correctness` adverse but only in the TOCTOU sense;
`data`/`state` unaffected — the command is read-only, publishes no repository
byte, and the accepted tree-identity oracles cover every failure edge;
`authority` unaffected — denial is still surfaced as a diagnostic, not silently
dropped; `api_compatibility` unaffected — the envelope is unchanged. A user
acting on the stale diagnostic would re-run the read-only command and see the
corrected state.

**Current must-fix triggers.** `MF-1` rejected — the frozen fence obligation in
the `A` contract is over documents "the report consumed"; an unreadable document
supplies no read length, identity-of-bytes, or digest to pin, and the accepting
review PASSed residual 5 on exactly that basis. I checked the source-document
claims `A` published for over-claim: `command-source-trace.md` and
`source-data-model-catalog.md` at `A` both scope the fence to documents "the
report consumed" by "read length, filesystem identity, and content digest", so
`A`'s canonical docs do not assert coverage they lack. `MF-2` rejected — no
reachable *material* correctness, state, data-integrity, or authority failure:
a read-only diagnostic that is accurate at observation time and stale under
concurrent mutation is not a prohibited material failure, and no counterexample
was produced. `MF-3` rejected. `MF-4` rejected. `MF-5` rejected — no acceptance
criterion required unavailable-subject evidence. `MF-6` rejected — this gap
predates `A` in kind (the fence itself is new in `A` and closed the branch/
merge-request COULD-1 gap; it did not reopen a previously proven-closed
obligation).

**Severity independent of actionability:** `low`; materiality `nonmaterial`.
Blast radius `local` to `cross_workflow` (only the diagnostic consumer).
Reachability `conditional` (requires concurrent readability change).
Likelihood `unlikely`. Evidence availability `partial` — source-proved,
race not exhibited.

**Terminal actionability disposition:** `nonblocking_improvement` (successor).
This is the one note with real forward value; the accepting reviewer said the
same ("the only one worth carrying forward as a one-line entry if a future
packet widens the fence to unreadable subjects"). Causal class `null`; no repair
authority.

**H's change classification: risk-expanding repair / contract change.**
`H`:793 `DoctorUnavailableState` and `H`:802 `DoctorUnavailableSubject` add a
second fence-subject kind, pin the parent's no-follow identity, leaf name,
observed leaf kind/length/identity, failed boundary and typed `io::ErrorKind`,
and the recheck reperforms the bounded operation at fence time. This creates a
**new production failure mode**: a run that previously published a diagnostic
now terminalizes `SubjectChanged` and publishes nothing when the subject becomes
readable, is created, disappears, is symlink- or kind-replaced, has a replaced
parent, a changed length, or a different failure class. It also adds fence-time
filesystem syscalls per unavailable subject and a new fixed precedence order.
`H`'s own `command-source-trace.md` and `source-architecture.md` edits state
this as new behavior. This is not behavior-preserving hardening; it changes what
a real user-visible `cf doctor` run outputs in a class of situations that
previously succeeded.

---

### Contract-excluded item (recorded, not re-adjudicated)

The broader CLI envelope / text-output accounting item (`main.rs:1124-1135`)
remains owned by the command-wide response-projection packet in both `A` and
`H`. It is `out_of_scope` for this adjudication, is not closed by either
candidate, and neither candidate may be described as closing it.

---

## 5. Per-note index

| Note | Reproduced production defect on A | Frozen ref | Proved MF IDs | Severity / materiality | Terminal actionability | Causal class | H's change class |
|---|---|---|---|---|---|---|---|
| NOTE-1 production panic on the failure-cleanup edge | no — unreachable; only a `#[cfg(test)]` injected permit reaches it (`A`:5013, tests start `A`:3228) | none | none | `low` / `nonmaterial` | `nonblocking_improvement` (successor) | `null` | contract change (typed error replaces abort; debug and release both change; open-permit class settled by ledger drop) |
| NOTE-2 transient unreserved allocation per fenced record | no — charge is exact and immediate; one value wide | none | none | `none`–`low` / `nonmaterial` | `nonblocking_improvement` (successor) | `null` | behavior-preserving hardening **plus** a new production `BytesHashed` count and a new `RecordSubjectClaim` observation stage |
| NOTE-3 unreadable document not fenced | no — source-proved TOCTOU window; race not exhibited; read-only command, truthful at scan time | none | none | `low` / `nonmaterial` | `nonblocking_improvement` (successor) | `null` | risk-expanding repair / contract change (new terminalization class; previously-publishing runs can now fail) |

Impact gate for all three: `complete`. No acceptance-critical unknown remains.
No disposition is `null_pending_impact`. Repair authority for all three:
`null`.

---

## 6. Manager adjudication

### Subject and chronology

- reviewer report path / commit / SHA-256:
  `docs/development/ai/swarm-results/cfr168-rust-file-doctor-zero-residual-rereview-claude-20260730/review.md`
  / `1c555ff3f426d7fa6874db92ceb71515a8d908e6` (same bytes at canonical
  `c22a78aa025e4fd2bf38d335f9665b781e4f4714`) / `f7c52b05…`
- decision_locus_id: `cfr168.file-doctor.composition-input`
- observation IDs adjudicated: `FD2-NOTE1`, `FD2-NOTE2`, `FD2-NOTE3`
- adjudicator actor / date / evidence: Claude Opus 5, 2026-08-02, this document

### Final impact and actionability

- impact_gate: `complete` for all three observations
- final MF-1…MF-6: `rejected` for every trigger on every note, with the
  per-note evidence in §4
- final actionability_disposition: `nonblocking_improvement` × 3
- correction_required: `false`
- required_correction_causal_class: `null`
- authorized correction IDs / allowed paths / forbidden paths / rereview
  scope: `null` — this adjudication authorizes no repair

### Stable decision

- active decision: `SELECT_ACCEPTED_A` — `1a04490806b59f8c39b5c1cf03f42d06aa750a98`
  (tree `eafb2cbf20980944d16086a972100c321a8b56be`) is the File Doctor
  composition input for the CFR-168 accepted-input composition
- basis: `A` holds a fresh exact independent `ACCEPT` on the exact candidate
  with mutation-proved, non-vacuous, source-keyed-by-construction evidence;
  `H` holds none
- permitted reopen triggers (fixed, non-suppressible, none removed):
  `frozen_requirement_changed`, `materially_new_evidence`, `new_counterexample`,
  `prefrozen_metric_regression`
- reopen evidence at this time: none. An `ACCEPT` is not eternal — a proved
  reachable must-fix defect on `A`, arriving later through any of the four
  triggers, reopens this decision. Reviewer preference, theoretical
  completeness, and the retired zero-residual mandate do not.
- supersedes_decision_ids: none
- reopen result: `not_requested`

### Why not `REQUIRE_H_REVIEW`

No note meets a current must-fix trigger, so nothing forces `H` into the
critical path. Requiring `H`'s gates and review now would spend an
acceptance-profile revalidation, a source-keyed Cargo port dependency, and an
Ultra reviewer slot on three items each characterised by their own reviewer as
a scope boundary, an inherited property, or safer-than-before — the exact
`REQUIREMENT_OR_SCOPE_EXPANSION` mechanism the review-recursion audit proved for
this lineage.

### Why not `BLOCK_FOR_MISSING_EVIDENCE`

Nothing acceptance-critical is missing **for the choice between A and H**.
`A`'s subject identity, changed-path set, patch digest, report bytes, report
topology, and mutation-proved gate evidence are all recomputed and consistent.
`H`'s evidence is missing — but that is precisely what makes `H` unselectable,
not what makes the choice undecidable.

---

## 7. Selection, exclusions, and required evidence for H

### Selected

| Field | Value |
|---|---|
| Composition input | `1a04490806b59f8c39b5c1cf03f42d06aa750a98` |
| Tree | `eafb2cbf20980944d16086a972100c321a8b56be` |
| Authorizing report | `f7c52b05…` at `1c555ff` / canonical `c22a78a` |
| Product paths contributed | `crates/codefire-cli/src/remote/doctor.rs` plus the four canonical source documents |

### Excluded from the current composition root

- `cd130e544af0476162be153db58a0a840b75b134` and its tree
  `2cf767fa00337e167b8a54e8bd20b6af5558b8df` — no report, therefore no
  integration authority, per `operating-protocol.md` §"Review Governance"
  ("The reviewer's terminal verdict is a proposal, not integration authority";
  no report at all is strictly weaker than a proposal).
- The three `H` mechanisms individually: the typed ownership-violation
  substitution, the `BytesHashed`/`RecordSubjectClaim` accounting additions, and
  the unavailable-subject fence. None may be cherry-picked into the composition
  without its own admitted candidate and review.
- The `H` worker's self-reported gates (25/136/792, R1–R17) — private
  Rustup/Cargo home and private target directories; contextual only, per the
  frozen candidate card.
- The 2026-07-31 manager revalidation gates `01-live-debug`
  (`0 selected / 794 filtered`, fail-closed invalid) and `02-live-release`
  (16 of the required NOTE cases; `wall_clock_discontinuity=true`) — neither is
  acceptance evidence.
- The retired zero-residual completion mandate as a source of obligation.
- No third product candidate is created, proposed, or implied. Nothing is
  repaired here.

### Routing of the useful residue

NOTE-3 carries the only concrete forward value and is routed as a **separately
admitted successor**, not as a correction and not inside this composition:
widen the record fence to unavailable subjects, with its own packet, its own
frozen hazard set (new terminalization class; fence-time syscall cost;
multi-fault precedence), its own oracles, and its own exact review. NOTE-1 and
NOTE-2 may be admitted alongside it if a manager chooses; neither is required.
`cd130e5` is available as *reference material* for that successor's design; it
is not its base and confers no inherited evidence.

### Evidence required before H could be selected

All four levels, in order — nothing below is optional and no level substitutes
for a later one:

1. **Source-keyed acceptance-profile gates.** Run through
   `docs/development/tools/run_source_keyed_cargo_gate.py` per
   `docs/development/ai/source-keyed-cargo-gates.md`, under the shared
   low-memory profile, with the source key bound and a nonzero selected count
   recorded for each gate: focused N1/N2/N3 cases in **both** debug and release
   (the release run must cover `file_remote_doctor_*` and
   `doctor_subject_frame_measurement_is_exact_for_every_path_shape`, not only
   the 16 `live_file_doctor_*` cases); `remote::doctor`; the `doctor` group;
   the full `codefire-cli` binary target; `cargo test -p codefire-core --locked`;
   strict all-target `codefire-cli` Clippy; `cargo fmt --check`;
   `check_source_docs.py`; `git diff --check`. Each with exact argv,
   environment, toolchain digest, artifact provenance, complete log bytes and
   digest, monotonic duration, and post-run identity plus clean state.
   That port is itself an open dependency (`RUST-FILE-DOCTOR` in
   `open-items.v1.json`).
2. **An immutable successor manager record authorizing review launch.** The
   candidate card `6809216e…` is frozen at `BLOCK` and must not be amended; a
   new commit must supersede it and explicitly release
   `dev-swarm validate/plan/start` for the rereview manifest.
3. **A fresh exact independent review** in a new agent/session/worktree, reusing
   neither the implementer nor either earlier File Doctor reviewer, covering:
   every one of `H`'s five changed paths with at least one reviewed surface; all
   eight inherited `A` residuals re-proved non-vacuous (R1–R8 still fail as
   intended); and the three candidate-created surfaces adjudicated on their own
   evidence — (a) the typed ownership-violation substitution, including executed
   proof of the `DecisionWorkLedger` drop claim for the `OpenPermits` class and
   of the changed debug-build behavior, (b) the new production `BytesHashed`
   count and `RecordSubjectClaim` stage against the accepted growth-attribution
   and inert-observer contracts, and (c) the unavailable-subject fence's new
   terminalization class, its multi-fault precedence, and its effect on runs
   that previously published. Golden JSON/text literals are byte-identical
   between `A` and `H`, which is necessary but not sufficient: the goldens must
   be *executed* at `H`, not merely compared.
4. **A terminal `ACCEPT` plus a separate manager impact/actionability
   adjudication**, after which `H` — not `A` — could be bound as the composition
   input by a new decision that supersedes this one.

Until all four exist, `H` is unselectable regardless of the technical merit of
its three mechanisms.

---

## 8. Next authorized action

The manager binds `1a04490806b59f8c39b5c1cf03f42d06aa750a98` (tree
`eafb2cbf…`) as the File Doctor accepted composition input in the CFR-168
accepted-input composition graph, satisfying item 4 of the closure checkpoint in
`docs/development/current-cycle-burn-down-plan.md`, and proceeds with the
composition's own predecessors.

This document claims **no** CFR-168 integration, no merge to `main`, no Install,
no Dogfood, no Close, and no issue closure. It does not change the state of
`RUST-FILE-DOCTOR` in `open-items.v1.json`, the review disposition index, or any
lifecycle document; those are separate manager writes. Selecting `A` as a
composition input is a composition-scope decision; the composed candidate must
still earn its own fresh exact `ACCEPT` before anything is integrated.

---

ADJUDICATION: SELECT_ACCEPTED_A
