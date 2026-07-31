# CFR-168 Open Execution Audit

Status: `OPEN`

This is a durable, exact-snapshot inventory of the remaining CFR-168,
CFB-136, and CFB-138 recovery and final-composition work. The machine-readable
source of truth is
[`open-items.v1.json`](open-items.v1.json).

## Exact snapshot

| Field | Value |
|---|---|
| Canonical head | `664f907062c9877f753b056902990916833f3b91` |
| Canonical tree | `55256458b83a5db16fff6a944b47d27318e2b62e` |
| Captured at | `2026-07-31T03:56:46Z` |
| Product baseline | `e2a218e015953f082245f36dbc6aee906b837e5a` |
| Recovery ledger SHA-256 | `8362062938df520a36dd4fca6c3353ddc9cda415422dc739418ee2f5b5553f76` |
| Review index SHA-256 | `5739e784c4b34a6cf5a4470ac1fc066e038f92671bf09e8c4c6f38158e87478a` |

The canonical branch advanced while the source audit was running. This
artifact deliberately freezes `664f907`; it does not silently absorb a later
candidate, report, or disposition.

## No-acceptance claim

This audit does **not**:

- accept or integrate a product candidate;
- infer a verdict from a candidate card, worker gate, active session, or
  missing report;
- authorize CFR-168, CFB-136, or CFB-138 Close;
- authorize Install, Dogfood, release, or capability re-advertisement; or
- replace the immutable recovery ledger or independent-review index.

Its only purpose is to make current ownership, exact subjects, blockers, and
next required development steps explicit.

## How to read the JSON

The JSON uses a fixed document-order serialization:

1. schema, version, and key-order policy;
2. exact snapshot and source-document hashes;
3. scope and counts;
4. the three issue rows;
5. 29 unique open execution items;
6. all 18 committed independent-review dispositions;
7. six expected-but-absent report outputs;
8. stale/superseded caveats; and
9. explicit non-authorization claims.

For execution state, use `open_execution_items`. For immutable historical
review evidence, use `review_dispositions`. A `SUPERSEDED_*` row remains
evidence but must not be scheduled or integrated. A pending report path is an
absence fact, not a provisional decision.

## Counts

| Measure | Count |
|---|---:|
| Repository-wide active issues | 9 |
| Issues in this scope | 3 |
| Indexed independent reports | 18 |
| Product reports | 17 |
| Control-plane reports | 1 |
| `ACCEPT` reports | 3 |
| `REQUEST_CHANGES` reports | 15 |
| Candidate-ready cards | 6 |
| Current candidate-ready cards | 5 |
| Superseded candidate-ready cards | 1 |
| Open execution items | 29 |
| Pending report outputs | 6 |

The repository-wide count also includes CFB-133 and CFR-169 through CFR-173;
those later roots are outside this artifact.

## Current critical path

1. Collect exact independent decisions for Python Upload `467b0c2` and
   MR-list `b164046`.
2. If both are accepted, run the shared-primitives Fugu preflight. Only a
   committed `READY` can release the MR-write `6dd4527` repair.
3. In parallel, close and independently accept the source-keyed Cargo guard.
   It gates File Ancestry `02b0d63`, File Doctor `cd130e5`, Rust HTTP
   `92fc810`, and the Storage `cdcb9bd` repair.
4. Produce exact accepted Python and Rust runtime compositions, then implement
   the eight predecessor-gated local/GC/clone/storage-report successor
   surfaces.
5. Complete decision-work metrics projection and the canonical
   complexity/cross-runtime corpus.
6. Freeze one integrated source-close SHA; run full gates, independent High
   review, reproducible build, isolated Install, installed reversals, and
   Algorithm/real-repository Dogfood.
7. Apply and independently accept the final declaration-only capability
   mutation, rerun the whole terminal lifecycle, synchronize issue/map/
   history/index/ledger/KGI state, and Retrospect.

## Exact candidates awaiting fresh review

| Owner | Exact candidate | Current state |
|---|---|---|
| Python Upload / CFB-138 | `467b0c27715e424ae18ffb5664da053b55915e2d` | fresh Fugu review active; report absent |
| Python MR-list | `b16404676639336f705feffd9af4857d4a1fcc69` | fresh Claude review active; report absent |
| Rust File Ancestry | `02b0d63651703aaac88c249bdc437cd173dfa397` | source-keyed validation blocked |
| Rust File Doctor | `cd130e544af0476162be153db58a0a840b75b134` | source-keyed nonzero NOTE gates blocked |
| Rust HTTP MR | `92fc8103bb4de400eaf40e9db76075130dedbb35` | source-keyed profile evidence blocked |

Python MR-write `6dd4527…` and Rust Storage `cdcb9bd…` remain rejected
repair subjects, not review-ready successors. Doctor+Branch `701469f…` is an
accepted exact scaffold but is not integration-authorized.

## Open execution groups

Every ID below has one full owner/subject/verdict/blocker/next/evidence row in
the JSON.

- Validation control: `CTRL-SOURCE-KEYED-RUST-GATE`
- Python critical path:
  `PY-DOCTOR-BRANCH-COMPOSED-INPUT`, `PY-UPLOAD-CFB138`,
  `PY-MR-LIST`, `PY-SHARED-PRIMITIVES-PREFLIGHT`, `PY-MR-WRITE`,
  `PY-HTTP-COMPOSITION`
- Rust critical path:
  `RUST-FILE-ANCESTRY`, `RUST-FILE-DOCTOR`, `RUST-HTTP-MR`,
  `RUST-STORAGE`, `RUST-RUNTIME-COMPOSITION`
- Predecessor-gated successors:
  `PY-POST-GC`, `PY-LOCAL-ANCESTRY`, `RUST-LOCAL-MERGE`,
  `RUST-SCAN`, `RUST-LOCAL-DOCTOR`, `RUST-FILE-CLONE`,
  `RUST-STORAGE-REPORT-GRAPH`, `RUST-FILE-GC`
- Final evidence and lifecycle:
  `FINAL-METRICS-PROJECTION`, `FINAL-COMPLEXITY-CONFORMANCE`,
  `FINAL-SOURCE-CLOSE-PRECAP`, `FINAL-PRECAP-BUILD-INSTALL`,
  `FINAL-PRECAP-REVERSAL-DOGFOOD`,
  `FINAL-CAPABILITY-READVERTISEMENT`,
  `FINAL-POSTCAP-TERMINAL-RERUN`, `FINAL-SYNC-RETROSPECT`
- Measurement control: `CTRL-KGI-KPI`

## Review disposition interpretation

The 18 immutable reports contain three `ACCEPT` decisions and fifteen
`REQUEST_CHANGES` decisions. The accepted reports are:

- Doctor+Branch `701469f…`: accepted as a scoped composition input;
- File Doctor `52a025e…`: accepted predecessor evidence; and
- File Doctor `1a0449…`: accepted eight-residual evidence.

None is a complete integrated CFR-168 acceptance. The later File Doctor
candidate `cd130e5…` still requires its own exact review.

The review index also contains one development-flow KGI/KPI report. It is
control-plane evidence and must not inflate the product review count.

## Pending outputs

The snapshot expects six reports that do not yet exist:

1. Upload `467b0c2…` Fugu rereview;
2. MR-list `b164046…` Claude rereview;
3. Rust File Ancestry `02b0d63…` Fugu rereview;
4. Rust File Doctor `cd130e5…` Claude rereview;
5. Rust HTTP `92fc810…` Claude rereview; and
6. development-flow KGI/KPI `0ae583e…` Fugu rereview.

The JSON records each exact path and the required terminal decision
vocabulary.

## Precedence and stale evidence

- The old MR-list `4930d251…` candidate card is immutable but superseded by
  rejecting report `909978c` and candidate `b164046`.
- Some Rust cards retain earlier Cargo-conflict wording; the later
  source-target quarantine in the recovery ledger is current.
- The flow-metrics record is a historical `dd993cc…` capture. Its `running`
  values are not current lane state.
- The blocked lifecycle packet binds older ledger/index hashes and must be
  rebound before it becomes ready.
- Single-module `owner_module` values in the issue map are insufficient for
  the current cross-runtime execution topology.
- The source-keyed Cargo guard has no canonical candidate at this snapshot.
- CFR-168 and CFB-138 detail status is correct but does not directly bind the
  latest Upload rereview state.

The JSON contains the complete caveat rows and their evidence paths.

## Write boundary

This audit commit may add only:

- `docs/development/ai/swarm-results/cfr168-open-execution-audit-20260731/audit.md`
- `docs/development/ai/swarm-results/cfr168-open-execution-audit-20260731/open-items.v1.json`

The recovery ledger, review index, issues, product source, tests, manifests,
and candidate/report evidence remain unchanged.

## Validation

The committed artifact must pass:

- strict JSON parsing and fixed key-order assertions;
- exact count assertions;
- duplicate ID and owner-reference checks;
- existence checks for every declared evidence path;
- absence checks for all six pending outputs;
- `check_ai_contracts.py`;
- `check_project_docs.py`;
- `git diff --check`; and
- a final two-path write-scope audit.
