# CFR-168 Independent Review: `agent/cfr168-execution-evidence-accounting`

Reviewer lane: `claude-cfr168-review-20260729-r2` / task `review` (tier high).
Review date: 2026-07-29. Reviewer role: independent, read-only against the candidate.

**Decision: `REQUEST_CHANGES`.**

One Must-class implementation defect was reproduced in the lane's own new code, on exactly the
invariant this slice claims to establish (bounding execution-evidence work before it is performed).
Everything else the lane claims in `docs/development/history.md` was verified as actually present and
passing.

---

## 1. Revision landscape

| Item | Value |
|---|---|
| Declared base revision | `f1644122134d479333cf422fdb1d48b8fa92dd8e` ("Complete AI acceleration bootstrap") |
| `main` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` (identical to base) |
| Candidate branch | `agent/cfr168-execution-evidence-accounting` |
| Candidate tip | `aa5e57d8b1e22fb83792412c68f255ceaf603e39` ("Document planner function shape authority") |
| `git merge-base f164412 aa5e57d` | `f1644122134d479333cf422fdb1d48b8fa92dd8e` |
| Relationship | Candidate is a strict linear descendant of base; fast-forwardable, no divergence to reconcile |
| Unique commit range | `f164412..aa5e57d` = **93 commits**, 321 files, +217540/-10135 |
| Full-range patch sha256 | `9b773aa86d6f93e1f3bc9f8ada2f6ea91a25a89cb67afffe8509eb1f5a8b6a7b` |

The candidate is a **stacked** branch: it contains the work of many sibling `agent/cfr168-*` lanes.
The slice this ticket names ("execution evidence accounting") is the tail of the stack, on top of
`agent/cfr168-execution-evidence-phase-a` (`579867b0314ed017ace84be4e901822742719edf`, verified an
ancestor of the candidate with `git merge-base --is-ancestor`).

| Item | Value |
|---|---|
| Lane-specific range | `579867b..aa5e57d` = **15 commits**, 8 files, +1692/-31 |
| Lane-specific patch sha256 | `827578e92a981bbaf6a28c2c9fd8e1c050c1f273385e4c894b70f56d3e515e6e` |

Lane-specific commits (oldest first):

```
7668969 Add execution evidence accounting counterexample
388f345 Account execution evidence retained state
e2da01a Account execution evidence byte work
758ee8b Correct execution evidence accounting ownership
8da98f5 Pin execution accounting schema
9d68072 Make evidence upper planning scalar only
d54bb0d Pin execution accounting measurement authority
f6f0a8d Document execution evidence accounting checkpoint
f17be87 Extend accounting authority through reducer
741079c Preserve reducer authority registry fixtures
3b8897a Record reducer accounting authority correction
7725e6c Pin execution planner scalar inputs
e326f3f Record planner scalar authority correction
060285a Pin execution planner function shape
aa5e57d Document planner function shape authority
```

Lane-specific write set (`git diff --stat 579867b..aa5e57d`):

```
codefire                                                   1026 +++++-
docs/development/history.md                                   4 +
docs/development/known-limitations.md                         2 +-
docs/development/source-architecture.md                      16 +-
tests/test_python_existing_execution_evidence_accounting.py   48 +
tests/test_python_existing_recovery_authoritative_classifier.py 250 +++
tests/test_python_existing_recovery_proof.py                 215 ++-
tests/test_python_existing_reducer_recovery_integration.py   162 ++
```

No Rust, schema, ADR, or task-packet file is touched by the lane range.

## 2. Contracts read

Read at the candidate revision (`git show aa5e57d:<path>`):

- `docs/development/code-review-issues/cfr-168.md` (1230 lines) — root issue, Summary / Problem /
  Required Fix / Acceptance Criteria / Frozen Implementation Plan / Implementation Evidence.
- `docs/development/bug-issues/cfb-136.md` (79 lines) — same-root blocker, Status `investigating`.
- `adr/ADR-020-decision-equivalent-work-budget.md` (292 lines, Status Accepted) — work vocabulary,
  budget profiles, evaluators/memoization, failure contract, required proof before adoption.
- `docs/development/ai/task-packets/cfr-168-decision-work-budget.md` — objective, scope, non-goals,
  parallel lanes, invariants, lifecycle gate, validation, handoff.
- Canonical source docs touched by the lane: `docs/development/source-architecture.md:320-334`,
  `docs/development/known-limitations.md:183`, `docs/development/history.md:16`.
- Implementation: `codefire:28461-29065` (new accounting types/planners/authority),
  `codefire:29882-30133` (`_build_execution_evidence`), `codefire:30576-30677` (measurement, upper
  check, consume, shrink, release), `codefire:31398-31797` (factory reservation and record creation),
  plus the terminal/reducer validation mirrors at `codefire:30160-30230`, `codefire:30984-31070`.
- Tests: the four modules in the lane write set.

## 3. What the slice actually does (verified)

`_HTTPUploadExistingExecutionAccounting` (`codefire:28461`) is a seven-dimension record
(`retained_bytes`, `temporary_bytes`, `aggregate_bytes`, `object_records`, `files_read`,
`file_bytes_read`, `bytes_hashed`) embedded in the terminal-frozen
`_HTTPUploadExistingExecutionEvidence` (`codefire:28500`) that Phase B (the not-yet-implemented
filesystem reducer) will consume.

Six pure planners (`codefire:28648`, `:28675`, `:28704`, `:28759`, `:28800`, `:28841`) compute
pre-admission uppers from scalars and module-load fixed JSON fragments only. A 26-element singleton
authority (`codefire:28953`) pins the borrowed-ID walker, the two measurement helpers,
`sys.getsizeof`, the hash domain, the schema name-byte count, `U64_MAX`, all six planners, nine
size/base scalars, the component/path limits, `VERSION`, and the six immutable planner `__code__`
objects. The factory (`codefire:31490-31571`) rejects any drift of those 26 elements plus
`__defaults__`/`__kwdefaults__`/`__closure__` on every planner before the uppers are computed; the
terminal probe and the reducer's `begin_effect` / `mark_durable` / response edges repeat the same
check.

Ordering is as documented: uppers are computed pre-permit (`codefire:31572-31626`), reserved and
consumed on the `_StablePermitTransition` (`codefire:31724-31742`), the record is allocated once
after permit acquisition (`codefire:31743`) and the identical object is embedded in the evidence
(`codefire:30588`, `codefire:30131`). After capture, the evidence graph is measured with
identity-deduplication against predecessor aliases (`codefire:30591-30625`), compared to the reserved
uppers (`codefire:30626-30631`), the actual work is consumed (`codefire:30636`), unused headroom is
cancelled (`codefire:30666`), Retained is shrunk to the exact value (`codefire:30676`) and Temporary
is released (`codefire:30677`).

The `object_records = files_read = file_bytes_read = 0` claim is **correct**: every filesystem read
this slice performs goes through `_read_raw_file` / `_read_file`, which append to `self._files`
(`codefire:29423-29443`) and are therefore already charged by the pre-existing recovery dimensions
(`2 * len(files)`, `2 * physical` at `codefire:30656-30657`). Because `self._files` is passed as a
borrowed root, the raw bytes are correctly excluded from the evidence-specific Retained measurement
rather than double-charged.

I also confirmed every claim in the lane's `history.md` entries maps to real, passing fixtures:

| Claim in `history.md` | Where it actually is |
|---|---|
| fresh R/T exact/-1 | `tests/test_python_existing_recovery_proof.py:1215` (new) |
| receipt R/T exact/-1 | `tests/test_python_existing_recovery_proof.py:1132` (pre-existing, extended with `execution_retained`/`execution_temporary` and a new `delta=0` accept arm) |
| aggregate/hash exact/-1 | same module, the `charges`/`counters` loop, extended at the diff hunk adding `charges["aggregate_bytes"] += execution_aggregate` / `counters["bytes_hashed"] += execution_hashed` |
| 4 public variants, no post-`READY` increase | `tests/test_python_existing_reducer_recovery_integration.py`, the added `accounted_cumulative`/`accounted_counters` equality assertions inside the pre-existing 4-variant loop |
| helper/constant/planner drift | `tests/test_python_existing_recovery_authoritative_classifier.py:594,682,753,798` and `tests/test_python_existing_reducer_recovery_integration.py:205` |

Planner `__code__` rebinding (not only `__defaults__`) is genuinely covered, at both the pre-factory
and terminal edges.

## 4. Findings

### MUST-1 — Execution-accounting uppers are keyed to the request's write plan while the work is driven by the persisted pending document; the mismatch is only detected after the work is done

**Classification: implementation defect in the lane's new code.**

All six planners derive their bound from `write_count = len(publication._write_plan)`
(`codefire:28655`, `:28681`, `:28720`, `:28774`) — a request-side quantity. But for the `pending` and
`completed-quarantine` variants the evidence that is actually generated is driven by `records`, which
comes from the on-disk `pending_transaction.json`:

- `_typed_records` (`codefire:29496`) bounds the persisted `writes` list only by
  `not 1 <= len(records) <= 64` (`codefire:29497`). It never compares that length — or the record
  contents — to `publication._write_plan`.
- Nothing else closes the gap. The classifier validates `type`, `version`, `status`/`phase`,
  `transaction_id`, `operation`, `actor`, `idempotency_key`, `request_hash`, `object_roots`,
  `result`, and `created_at` against the request (`codefire:30359-30386`), but `request_hash` itself
  is computed from operation/actor/semantic/key/object_roots only
  (`remote_publication_request_hash`, `codefire:2419`) and therefore does not cover `writes`.
- The generation loops then scale with `len(records)`: `expected_names` (`codefire:29961-29985`),
  `selected_stages` (`codefire:30415-30460`), `authority_frontier` (`codefire:30461-30473`), and the
  completion envelopes (`codefire:30047-30091`).

Because each of the `len(records) + 2` envelopes embeds *all* `len(records)` records, generated
canonical bytes are Θ(n²) in the persisted record count, while the reservation is Θ(1) in it. In the
`_prepare` harness the reservation is dominated by `publication._retained_exact` (168 110 B), so
`generated_upper = (write_count + 2 + int(keyed)) * pending_upper = 3 * 168 967` is *constant*
regardless of how many records the on-disk document declares.

Reproduced against the candidate with a throwaway probe (a `pending_transaction.json` carrying
`n` writes plus matching stage files, against a 1-write request write plan):

| persisted records | envelopes | generated B | Retained exact B | Retained reserved B | aggregate actual B | aggregate reserved B | hashed actual B | hashed reserved B | outcome |
|---|---|---|---|---|---|---|---|---|---|
| 1 (matches plan) | 2 | 1 391 | — | 688 966 | 6 551 | 1 060 167 | 1 542 | 507 264 | accepted |
| 9 | 10 | 22 391 | 37 912 | 688 966 | 50 151 | 1 060 167 | 22 542 | 507 264 | accepted |
| 25 | 26 | 140 068 | 172 910 | 688 966 | 288 780 | 1 060 167 | 140 219 | 507 264 | accepted |
| 49 | 50 | 505 732 | 564 538 | 688 966 | 1 025 028 | 1 060 167 | 505 883 | 507 264 | accepted |
| 64 | — | — | — | 688 966 | — | 1 060 167 | — | 507 264 | `CFError: HTTP upload execution accounting upper was exceeded` |

Consequences, against the contracts:

1. **ADR-020 charge-before-work is violated.** "Budget is charged before allocation, parsing,
   queueing, response publication, or authority writes" (task packet, Invariants) and "A counter is
   charged before the corresponding allocation, queue insertion, parse, validation, or output append"
   (ADR-020, One work vocabulary). At 64 records the whole quadratic envelope generation and hashing
   completes and only then does the post-hoc comparison at `codefire:30626-30631` raise. The
   reservation taken at `codefire:31724` was never a bound on the work.
2. **Bounded-but-wrong admission inside the constant slack.** At 49 persisted records the path
   *succeeds* having done 156× the legitimate aggregate work (1 025 028 B vs 6 551 B) and 328× the
   legitimate hash work (505 883 B vs 1 542 B), consuming 96.7% / 99.7% of a reservation that was
   sized for one record. This is precisely the failure mode ADR-020's "one cumulative ledger proves
   that N heads/phases cannot each consume the full command budget" proof requirement exists to
   exclude.
3. **The Temporary derivation inherits the same defect.** `temporary_upper` returns
   `construction + 4 * retained_upper` (`codefire:28786-28797`) where `retained_upper` is the same
   constant, but the `seen` set inside `_measure_http_upload_existing_execution_retained`
   (`codefire:28877`) has one entry per evidence object and therefore scales with `len(records)`.
   Unlike Retained/aggregate/hash, Temporary is never measured — it is released wholesale at
   `codefire:30677` — so a Temporary under-reservation has **no runtime detector at all**.
4. **Integrity, not only accounting.** The module docstring calls this the "recovery-first
   *authoritative* classifier", and `source-architecture.md:320+` presents its terminal snapshot as
   the pre-effect authority. A tampered or corrupt `pending_transaction.json` that keeps the correct
   `request_hash` but declares different `writes` is accepted, and its extra authority records become
   terminal-frozen Phase B evidence. A legitimate resume of the same request must have the same
   writes, so binding `records` to `publication._write_plan` costs nothing and closes both the
   accounting and the integrity gap.

**Not currently live.** `_prepare_http_upload_existing_publication_recovery_proof`
(`codefire:31398`) has no production caller — `handle_http_upload` (`codefire:12058`) does not reach
it, and `known-limitations.md:183` documents the handler wiring as an open Freeze. So this is a
pre-wiring defect, not an exploitable production path. It must nevertheless be fixed before the
Freeze is lifted, because the slice's stated purpose is to install this bound.

Suggested fix: require `len(records) == len(publication._write_plan)` and per-record equality against
the write plan (path, authority bytes, `before_absent`, `before_hash`, `after_hash`) inside
`_typed_records` or immediately after it, before any evidence is generated. That makes the existing
`write_count`-derived planners exact and turns the post-hoc check at `codefire:30626` into a
redundant assertion rather than the only line of defence.

### SHOULD-1 — Accounting-upper breach is an untyped `CFError`, not the ADR-020 typed resource-budget outcome

`codefire:30631` raises `CFError("HTTP upload execution accounting upper was exceeded")`, and the
four new overflow pre-checks (`codefire:31659`, `:31682`, `:31706`, `:31715`) plus
`_measure_..._work` (`codefire:28949`) do the same. ADR-020's Failure contract requires a typed
blocking outcome and exit 60 carrying profile, dimension, observed value, limit, phase, command, and
continuation support — the shape `ResourceBudgetExceeded` already provides elsewhere in this file.
This follows a pre-existing pattern in the surrounding owner, so it is not a lane regression, but the
new code adds five more sites that will need conversion before the typed 413/exit-60 contract can be
claimed for this path.

### SHOULD-2 — The Temporary bound for the borrowed-ID set is asserted, not derived

The comment at `codefire:28794-28796` justifies `4 * retained_upper` as covering "two identity sets
and the largest pending list while the predecessor-ID set remains live". The stated derivation
("every retained evidence referent consumes at least one byte") bounds the *evidence* `seen` set only.
`_http_upload_existing_execution_borrowed_ids` (`codefire:28894`) walks the whole **predecessor**
graph (`_HTTP_UPLOAD_EXISTING_EXECUTION_SCHEMA`, the authority itself, `publication._write_plan`,
`semantic._copy_plan`/`_copy_bytes`/`_semantic_view`, `self._directories`, `self._files`,
`self._absence`, …), whose object count is not a function of evidence retained bytes. It is
numerically comfortable in the fixtures I ran, but the derivation does not establish the bound, and
Temporary has no measurement backstop.

### SHOULD-3 — `evidence.accounting` carries planning uppers, not the exact charges its name implies

The record built at `codefire:31743` stores `execution_retained`, `execution_temporary`,
`execution_aggregate`, `execution_hashed` — the pre-admission **uppers**. The exact values measured
at `codefire:30610-30625` are used for `shrink_retained_primary` / `consume` but never written back.
Measured gap in the harness: `accounting.retained_bytes = 688 966` vs exact `10 640` at the matched
record count (65×), and `accounting.aggregate_bytes = 1 060 167` vs actual `6 551` (162×).
`accounting.temporary_bytes` is stale by construction, since all Temporary is released at
`codefire:30677`. `source-architecture.md:323` calls this "a typed seven-dimension accounting record"
and then says Retained is shrunk "to exact ownership", which reads as though the record is exact.
Phase B must not consume these as measured work; either write the exact values back after the shrink
or rename the field (e.g. `reserved_upper`) and say so in the canonical docs.

### SHOULD-4 — CFR-168 root ledger carries no evidence for this slice (missing integration evidence, not a defect)

`docs/development/code-review-issues/cfr-168.md` at the candidate has **zero** occurrences of
"execution evidence" or "execution accounting" (`grep -i`). Its "Implementation Evidence (in
progress)" section ends at the reducer-external-authority entry, and the file was last modified by
`e87af3f` — before both `agent/cfr168-execution-evidence-phase-a` and this lane. The lane did update
`history.md`, `known-limitations.md`, and `source-architecture.md`. Per the task packet ("Only the
manager may … update authoritative ledgers"), the CFR detail/map/history ledger integration is the
manager's step; flagging it so it is not lost at integration time, since two consecutive slices are
now missing from the root issue.

### SHOULD-5 — No Rust counterpart and no disclosed parity residual (missing evidence, not a defect)

`grep -rn 'execution_evidence|ExecutionEvidence|execution-evidence' crates/ --include=*.rs` returns
nothing on the candidate; the lane touches no Rust. ADR-020 states "Rust default semantics are
authoritative. The Python fallback uses the same profile, charge points, ordering, diagnostic fields,
and limits", and its Required Proof list includes agreeing Rust/Python golden vectors. This slice
adds Python-only charge points. Both sides are production-unwired so nothing is broken today, but the
lane's residual list in `known-limitations.md:183` names only the 13-substate × 8-boundary failure
matrix, concurrent prepare serialization, and the legacy quarantine prefix-scan migration — it does
not disclose the cross-runtime parity gap. That residual should be recorded.

### SHOULD-6 — Candidate tip fails the AI acceleration docs gate (inherited from ancestor lanes)

`python3 docs/development/tools/check_ai_acceleration_docs.py` fails on the candidate with 23
`missing-task-section` errors across eight CFR-168 task packets
(`cfr-168-existing-preimage-accounting`, `-preimage-baseline`, `-recovery-proof`, `-response-bridge`,
`-shared-adoption`, `-typed-reducer`, `cfr-168-gate-relative-preimage-proof`,
`cfr-168-preimage-terminal-freeze`), which cascades into
`check_ai_development.py` → `FAIL acceleration_docs`. The same command **passes** on base
`f164412` and fails **identically (same 23 errors)** at the phase-a tip `579867b`, so this is
inherited from ancestor lanes in the stack and is not caused by the 15 lane commits. It will
nevertheless block the branch-level gate at integration.

### NICE-1 — "Identity-deduplicated" is only partly realized because of statement ordering

`borrowed_ids` is computed at `codefire:30591` passing `self._identity`, `self._fresh_plan`, and
`self._variant`, but those three attributes are not assigned until `codefire:30678-30679` — after the
measurement. At measurement time they still hold their bootstrap values, so the freshly built
`identity` / `fresh_plan` / `variant` state (including the `records` tuple) is not treated as
borrowed and its aliases are counted as evidence-owned. The direction is conservative (over-count,
never under-count), so it is not a safety problem, but it means the exact figure is not exact and the
`source-architecture.md:325` phrase "Recovery identity-deduplicates predecessor aliases" overstates
what happens for the state created in the same call.

### NICE-2 — The new test module is a thin smoke test and has a PEP 8 E305 slip

`tests/test_python_existing_execution_evidence_accounting.py` contains one test whose assertions are
`assertGreater(..., 0)` on four dimensions and `assertEqual(..., 0)` on three. All the substantive
exact/-1 and drift fixtures live in the three pre-existing modules, so a reader navigating by
filename will underestimate the coverage. Line 45 has a single blank line before
`if __name__ == "__main__":` where every sibling module in `tests/` uses two.

### NICE-3 — Accounting state now gates semantic admission; worth an explicit ADR note

ADR-020 says counters "describe correctness work and remain outside the authoritative domain result"
and that enabling/disabling metrics "cannot change the traversal, result, diagnostic, ordering, or
budget enforcement". This slice makes the accounting record and its 26-element authority part of the
pre-effect terminal subject: drift causes `begin_effect`, `mark_durable`, and
`_take_prepared_response` to raise (`codefire:30984-31070` and the reducer mirrors). Domain results
are unchanged and this reads as tamper-detection rather than metrics-controlled semantics, so I do
not treat it as a violation — but the manager should state the interpretation in ADR-020 rather than
leave it implicit.

## 5. Validation performed

The assigned worktree sits at base `f164412`, so the candidate was materialized read-only outside the
repository (`git archive aa5e57d | tar -x -C /tmp/cand`; a throwaway `git init` there only so the doc
checkers, which shell out to git, could run). No branch was checked out, merged, or modified; the
user's dirty main worktree was never inspected.

| validation_id | exact_command | status | result_summary |
|---|---|---|---|
| pycompile | `python3 -m py_compile codefire tests/test_codefire_cli.py` | passed | clean |
| focused-lane | `python3 -m unittest -v tests.test_python_existing_execution_evidence_accounting tests.test_python_existing_recovery_proof tests.test_python_existing_recovery_authoritative_classifier tests.test_python_existing_reducer_recovery_integration` | passed | **60/60** in 22.0 s — matches the lane's "focused 60" claim exactly |
| broader-existing | `python3 -m unittest discover -s tests -p 'test_python_existing_*.py'` | passed | **332/332** in 95.3 s |
| full-python | `python3 -m unittest discover -s tests` | failed (capability) | **860 tests, 2 failures**, both isolated (below) |
| full-python-failures | `python3 -m unittest -v tests.test_codefire_cli.CodeFireCliTests.test_install_script_installs_codefire tests.test_codefire_cli.CodeFireCliTests.test_completion_command_and_install_completion` | failed (capability) | Both fail with `AssertionError: 2 != 0 : error: cargo is required to build the Rust codefire binary; pass --binary PATH to use a prebuilt binary` (`tests/test_codefire_cli.py:13337` and `:13493`). `which cargo` / `which rustc` are both empty in this sandbox. **The same two tests fail identically on base `f164412`**, so this is an environment capability gap, not a candidate defect. The other four install/packaging/demo tests in that area pass. |
| source-docs | `python3 docs/development/tools/check_source_docs.py` | passed | 44 modules, 34 commands, 12 object kinds |
| project-docs | `python3 docs/development/tools/check_project_docs.py` | passed | 523 Markdown files inventoried |
| ai-contracts | `python3 docs/development/tools/check_ai_contracts.py` | passed | clean |
| ai-acceleration-docs | `python3 docs/development/tools/check_ai_acceleration_docs.py` | **failed** | 23 `missing-task-section` errors; passes on base `f164412`, fails identically at phase-a tip `579867b` → inherited (SHOULD-6) |
| ai-runtime | `python3 docs/development/tools/validate_ai_runtime.py` | failed (environment) | `missing-codex-version-adapter: codex_cli_0_146`; **also fails on base `f164412`** → local toolchain version, not a candidate issue |
| diff-check | `git diff --check 579867b aa5e57d` and `git diff --check f164412 aa5e57d` | passed | no whitespace errors; no added `codefire` line exceeds 100 columns in the lane range |
| reviewer-probe | throwaway `tests/test_probe_review_extra_writes.py` / `test_probe_review_work.py` in `/tmp/cand` | reproduced MUST-1 | table in §4; probe files were not added to the repository |

### Expensive gates deliberately skipped

- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` — **no Rust toolchain in this sandbox** (`which cargo` empty). The lane
  range touches no `crates/` file, so the lane cannot have regressed them, but the 93-commit branch
  does and the manager must run them.
- `./demo.sh`.
- Lifecycle gate items from the task packet: fresh isolated install, installed limit-reversal
  fixtures, Algorithm dogfood tests, canonical read-only/mutating workflow checks
  (status/scan/verify/Doctor), and the explicit Retrospect disposition. None were run.
- CFB-136's own acceptance criteria (`SealedCommitValidationCache` same-inode fixtures, Store/CLI
  cross-runtime mutation fixtures) were not exercised: the lane touches neither
  `crates/codefire-store/` nor the Python sealed-validation cache, so it neither advances nor
  regresses CFB-136, whose Status correctly remains `investigating`.

### Gaps in my own evidence

- MUST-1 was reproduced through the existing `_prepare` fixture, whose
  `publication._retained_exact = 168 110` is what makes the reservation constant. I did not sweep
  other request shapes, so the exact 156×/328× ratios are fixture-specific; the Θ(n²)-actual /
  Θ(1)-reserved shape is structural and does not depend on the fixture.
- I did not attempt to exhibit an overflow of the Temporary dimension (SHOULD-2/MUST-1 item 3),
  because Temporary is never measured and therefore has no observable.
- The 13-substate × 8-boundary failure matrix, concurrent-prepare serialization, and `resume(token)`
  filesystem-effect behaviour are declared residual by the lane; I confirmed they are absent but did
  not attempt to write them.

## 6. Distinguishing defects from missing evidence

| Finding | Kind |
|---|---|
| MUST-1 | implementation defect in the lane's new code |
| SHOULD-1 | implementation gap against ADR-020's failure contract (extends a pre-existing pattern) |
| SHOULD-2 | unsound derivation in the lane's new code, no observed failure |
| SHOULD-3 | contract/naming defect between the lane's record and its canonical documentation |
| SHOULD-4 | missing ledger integration evidence — manager-owned step |
| SHOULD-5 | missing cross-runtime parity evidence and an undisclosed residual |
| SHOULD-6 | missing/failing branch gate inherited from ancestor lanes, not this lane |
| NICE-1..3 | precision, coverage-legibility, and ADR-wording items |

## 7. Confidence

- **High** on the revision landscape, the write set, and every claim-to-fixture mapping in §3: all
  checked directly against the candidate blob and re-run locally.
- **High** on the MUST-1 mechanism: reproduced end to end, with the reservation formula's
  `write_count` dependency and the persisted-record-driven generation loops both read at exact line
  numbers, and the accept/reject boundary observed at 49 vs 64 records.
- **Medium-high** on MUST-1's severity ranking. It is a real defect on the slice's central invariant,
  which is why the decision is `REQUEST_CHANGES` rather than accept-with-notes; but the path is
  production-unwired, so it is a Freeze-lift blocker rather than a live integrity incident.
- **Medium** on SHOULD-2 (derivation is unsound; practical impact unproven and unobservable) and on
  the completeness of my gate coverage, since the Rust, install, dogfood, and Retrospect gates were
  not run at all.

**Recommended decision: `REQUEST_CHANGES`.**

Minimum to reach `ACCEPT_FOR_INTEGRATION_REVIEW`: fix MUST-1 (bind the persisted `writes` to the
request-derived write plan before any evidence generation) with an executable counterexample at the
`len(records) > write_count` boundary, and resolve SHOULD-3 so Phase B cannot mistake reserved uppers
for measured work. SHOULD-1/2/5 and the NICE items can be tracked as recorded residuals. SHOULD-4 and
SHOULD-6 are manager-owned integration steps and do not block this lane's code.
