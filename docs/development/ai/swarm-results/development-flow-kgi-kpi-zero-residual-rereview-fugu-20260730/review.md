# Development-Flow KGI/KPI Zero-Residual Independent Rereview

Review date: 2026-07-31

Reviewer: Fugu Ultra 1.1 independent reviewer

Exact candidate: `0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1`

Candidate parent: `a6d41a9d7cdae2157cdcc5ca9f9008e4734c4e16`

Candidate tree: `c4210c692a2cdf90c8073dbb899d9787eccdd448`

Implementation diff: `a6d41a9d7cdae2157cdcc5ca9f9008e4734c4e16..0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1`

**Decision: `REQUEST_CHANGES`.**

The candidate is a substantial repair: the committed CFR-168 record passes a new semantic
validator, all 55 frozen semantic mutations produce their exact expected finding, all 15 structural
controls still produce their exact expected finding, the canonical lifecycle and metric registry are
materially clearer, and the reference record is much more candid and arithmetically consistent than
the original. Those improvements do not satisfy the bound zero-residual rule.

Production-checker counterexamples still accept or crash on decision-critical states: nanosecond
ordering loss, non-finite numbers, multi-node lineage and supersession cycles, wall-clock-overdue
claims, cross-batch active-record interference, repository-escaping document symlinks, non-normalized
agent-log locators, forged evidence-prefix metadata, self-recomputable final-record digests,
fabricated or task-class-mismatched cohort members, arbitrary KGI metric bindings, and incomplete
lifecycle completion. At least 20 of the 35 bound findings are only partially closed. Any one is
blocking under the ticket; together they show that a passing checker still is not decision-grade.

This decision is bound only to exact candidate
`0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1`. It does not authorize integration, CFR-168 Close,
Install, Dogfood, or Retrospect.

---

## 1. Exact Subject And Review Bindings

### 1.1 Bound authorities

| Authority | Exact identity | Verification |
|---|---|---|
| Original independent-review commit | `b307b47f51f73b054b66444493c4590379518c4f` | Read complete report |
| Original report | `docs/development/ai/swarm-results/development-flow-kgi-kpi-independent-review-claude-20260730/review.md` | SHA-256 `02a0d259c55121379f07c2c1d893d4a29dd546b16e6f237659a5081c60fd3336`, exact match |
| Repair packet commit | `9dd0da4f2ae55742ee51c1671aab6308994de24c` | Markdown and typed packet read completely |
| Packet-section correction | `a6d41a9d7cdae2157cdcc5ca9f9008e4734c4e16` | Candidate parent and diff base |
| Repair packet Markdown | `docs/development/ai/task-packets/development-flow-kgi-kpi-zero-residual-repair-20260730.md` | SHA-256 `f5d413d56d711fb8faf7052a6d992f21f7a7ffb1e326fd3ca2c107fb73eb8e64` |
| Repair packet JSON | `docs/development/ai/task-packets/json/development-flow-kgi-kpi-zero-residual-repair-20260730.json` | SHA-256 `693390de55d4f5df02e5551f07d2b7c0835c068b63f38d602e38e2a1e30301e1` |
| Exact repaired candidate | `0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1` | Commit, parent, tree, and all changed blobs verified |
| Review worktree base revision | `110a38c80a1f89f3e8bb969c3e4273002047f07c` | Candidate is an ancestor; the 14 reviewed blobs match the candidate exactly |

The current worktree is a descendant because the manager-counterexample ticket was committed after
the candidate. No conclusion below is taken from descendant-only product content: for every file in
the implementation diff, `git rev-parse 0ae583e:<path>` equals `git hash-object <path>`.

### 1.2 Candidate changed files and reviewed blobs

| File | Candidate blob |
|---|---|
| `docs/development/README.md` | `d5026eae8a852c40d7747467ba7a2af8834a8ab5` |
| `docs/development/ai/README.md` | `d1a7e71bf4332b22e7aab381db3f79fc9c1ee2bb` |
| `docs/development/ai/acceleration-master-plan.md` | `37edc4970676ce961547e73f03d671d08e0e16a1` |
| `docs/development/ai/development-flow-kgi-kpi.md` | `673d299093bb775d78541be22dcca2ea6094b1c6` |
| `docs/development/ai/flow-metrics/README.md` | `d46022b66afe34bfefdaa467b759fa5bc68cacc3` |
| `docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json` | `87cf9506017588dfb177d1a44ea6ed01ca77eb7d` |
| `docs/development/ai/schemas/README.md` | `da921c0d7fa4cf9f3fc679812bdc094d660b352e` |
| `docs/development/ai/schemas/development-flow-record.v1.schema.json` | `6be66f71a9db3f83cb0522c644cfd61baf278880` |
| `docs/development/exploration/phase-2/experiment-catalog-and-kpi-scorecard.md` | `f0178f1d69f4432094acbe39b3386055d3d53d86` |
| `docs/development/history.md` | `433a3b61ae4ef38c408aa7dd548354df763e458f` |
| `docs/development/project-management.md` | `6680c504c5dad2c1afa63f75d427acca66fb7fac` |
| `docs/development/tools/ai_contracts.py` | `84895715fe52e847b1542c0b2efb0dda50b37f40` |
| `docs/development/tools/check_ai_contracts.py` | `1212a9cbe4af0c726aa5080f8a90d892c905fed2` |
| `docs/development/tools/test_ai_tools.py` | `93b579088b0e100fd969d791d3e42d2f5b645fd5` |

The candidate diff is 14 files, `+4011/-415`. The semantic validator grows by 1,957 lines and the
focused test file by 615 lines. The size is not used as acceptance evidence.

### 1.3 Additional authorities read

The rereview read `AGENTS.md`, the complete diff, the complete resulting schema, validator,
directory checker, policy, flow-metrics README, active record, indexes, Phase-2 promotion record,
history, and tests. Lifecycle authority was checked against
`docs/development/project-management.md:68`, `docs/development/agent-workflow.md`,
`docs/development/codefire-improvement-cycle.md`,
`docs/development/strategy/modernization-roadmap.md`, and
`docs/development/strategy/release-and-format-roadmap.md`.

### 1.4 Constraints honored

- Candidate source, schema, policy, exemplar, tests, issue state, and prior reports were not edited.
- New probe files and clone overlays were created only under `/tmp`.
- No Cargo, build, install, demo, dogfood, broad product suite, or network command was run.
- No uncommitted state or implementer narrative was treated as candidate evidence.
- Only this report is created in the worktree.

---

## 2. Method And Commands Actually Run

### 2.1 Method

1. Bound the exact commits, report digest, candidate tree, implementation diff, and all 14 blobs.
2. Read the original 35 findings and both repair packets in full.
3. Traced the production route
   `check_ai_contracts.py:63` → `validate_development_flow_record` →
   `_FlowRecordValidator.run` → `validate_development_flow_directory`.
4. Replayed the frozen 55 semantic mutations and 15 structural controls independently, asserting
   the exact expected code rather than accepting any rejection.
5. Re-ran all eight manager-confirmed launch counterexamples through the production validator and
   added adjacent representation, graph, path, lifecycle, cohort, numeric, and complexity probes.
6. Recomputed every interval/rate in the exemplar, every candidate/artifact commit timestamp, the
   token delta, checkpoint denominator, byte conversions, evidence-reference resolution, and KGI
   disposition.
7. Ran the ticket's bounded repository gates.

### 2.2 Executed validation and probe commands

`PYTHONDONTWRITEBYTECODE=1` was present for every Python execution below.

| Command | Result | Duration |
|---|---|---:|
| `python3 /tmp/replay_flow_matrix.py` | 55/55 semantic expected codes; 15/15 structural expected codes | 3.954989 s validator time; 5.388 s process |
| `python3 /tmp/kgi-kpi-rereview-probe.9JJ0tp/probe.py "$PWD" /tmp/kgi-kpi-rereview-fresh.npB5AZ` | Reproduced all eight manager probes and adjacent numeric/graph/path/digest cases | Part of 3.132 s combined run |
| `python3 /tmp/kgi-kpi-rereview-probe.9JJ0tp/cohort_probe.py "$PWD"` | Fabricated members, wrong task class, and mixed-class members accepted; uncensored open real member rejected | Part of 3.132 s combined run |
| `python3 /tmp/additional_flow_probes.py` | Reordering clean; leap second/offset/Unicode rejected; non-finite denominators and unbounded events exposed | 0.353 s process |
| `python3 /tmp/flow_record_toctou_probe.py /tmp/kgi-kpi-rereview-fresh.npB5AZ/candidate-clone` | On-disk record mutated after load; first checker pass returned zero findings, second pass observed the mutation | 0.284 s process |
| `python3 docs/development/tools/test_ai_tools.py` | `Ran 79 tests`; `OK` | 5.662837 s process; 5.125 s unittest |
| `python3 docs/development/tools/check_ai_contracts.py` | `AI contract check passed.` | 0.733522 s |
| `python3 docs/development/tools/check_ai_development.py --timeout 120` | All seven aggregate components `PASS` | 12.442835 s |
| `python3 docs/development/tools/check_project_docs.py` | `565 Markdown files inventoried`; passed | 1.421810 s |
| `git diff --check a6d41a9d7cdae2157cdcc5ca9f9008e4734c4e16..0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1` | Passed | 0.028414 s |

Additional direct validator calls exercised arbitrary KGI bindings, infinite rate denominators,
vacuous prose limitations, missing runtime versions, lifecycle completion without Install/Dogfood,
checkpoint-denominator omission, and same-evidence/different-kind artifacts. They are quoted in the
findings and matrix below.

### 2.3 Read-only identity and source-inspection commands

The following read-only commands were also run, with the displayed revisions and path sets rather
than aliases or uncommitted substitutes:

```text
git status --short
git rev-parse HEAD
git show -s --format=%T 0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1
git diff --name-only a6d41a9d7cdae2157cdcc5ca9f9008e4734c4e16..0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1
git diff --stat/--numstat/--unified=<n> a6d41a9..0ae583e -- <all 14 candidate files>
git log --oneline --no-merges a6d41a9..0ae583e
git show -s --format=<commit,parent,tree,date,subject> <all bound commits>
git merge-base HEAD 0ae583e
git merge-base --is-ancestor 0ae583e HEAD
git rev-parse 0ae583e:<path> and git hash-object <path> for all 14 changed files
sha256sum <original review, repair packets, candidate schema/record/checker/tests, probe artifacts>
find docs/development/ai/... and find /tmp/kgi-kpi-rereview-... for bound inputs and probe outputs
rg -n <finding IDs, policy terms, validator branches, test names> <bound docs/source/tests>
sed -n <ranges> <original review, packets, diff, source, schema, policy, record, tests>
nl -ba <candidate source/schema/policy/record/tests/lifecycle docs> | sed -n <ranges>
cat <complete repair packets and complete changed management/index files>
wc -l <review authorities and candidate surfaces>
```

Small `python3 - <<'PY' ... PY` read-only scripts were used to enumerate JSON sections, recompute
intervals/rates/bytes/token deltas, resolve all commit and document references, count event/metric
states, print candidate/current blob equality, and execute the direct counterexamples reported here.

### 2.4 Probe artifacts

| Artifact | SHA-256 |
|---|---|
| `/tmp/kgi-kpi-rereview-matrix.jsonl` | `1447fe49a35b3f08c82f281bcfc34dc1e490ad7f92c007f68b092831b77612f4` |
| `/tmp/kgi-kpi-rereview-fresh.npB5AZ/results.jsonl` | `6fc0b725502d89fce91e17296b1dc0414250f41b87fa081937bd75e32f0938df` |
| `/tmp/kgi-kpi-rereview-fresh.npB5AZ/cohort-results.jsonl` | `c5e2adef5436c4b62e96df7d8426b64a780cf78279cf514b762789a53323afaf` |
| `/tmp/kgi-kpi-rereview-additional.jsonl` | `3f7196cda93ccb3647a34be9fd4308787f91f16820524bced5f39008a76c6a2a` |
| `/tmp/kgi-kpi-rereview-toctou.jsonl` | `356b7de3212462b223efd903614541c7a4735be4ae7e95582dfbc860be5afc0a` |
| `/tmp/kgi-kpi-rereview-validations.json` | `59ea398a8560f9deeb871e5e84bd19cfe729a215f2ad541575389f045a7cf919` |

These temporary files are review evidence only and are not committed.

---

## 3. Severity-Ordered Blocking Findings

### MUST-BLOCKER R1 — Terminal KGI and cohort decisions remain self-authorizable

The policy says a KGI becomes terminal only through resolved metrics and that `KGI-DEV-005`
requires a closed pre-registered cohort and measured cross-provider total
(`docs/development/ai/development-flow-kgi-kpi.md:54`, `:314`). The validator does not bind each KGI
to the metrics or thresholds that define it. `check_kgi` accepts any measured metric reference and
does not evaluate a target (`docs/development/tools/ai_contracts.py:2790`). Direct production calls
set each of `KGI-DEV-001` through `KGI-DEV-004` to `achieved` with only
`metric_refs: ["MET-FLOW-028"]`, whose exemplar value is measured **zero**
(`docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json:808`): all four returned zero
findings.

`KGI-DEV-005` is more severe. `check_cohort_decision` counts member strings and filters only lanes
whose `task_id` happens to match; it never requires every member to resolve, never verifies each
member's declared `task_class`, and never recomputes the promised median lead-time reduction,
20-percent cost reduction, dispersion, or non-inferiority result
(`docs/development/tools/ai_contracts.py:2854`). These production cases returned zero findings:

- five unique fabricated task IDs, all listed as censored;
- one real `independent_review` member while the cohort declares `task_class: "docs"`;
- two real members of different classes while the cohort declares `docs`;
- `KGI-DEV-005: achieved`, exemplar throughput `0`, an arbitrary measured cross-provider total,
  and no before/after lead-time, cost, dispersion, or quality arithmetic.

The schema merely requires prose for `censoring_rule`, `non_inferiority_margin`, and
`unavailable_cost_rule` (`docs/development/ai/schemas/development-flow-record.v1.schema.json:86`).
This is a direct false-achievement path for F11, F12, and F13.

The same self-authorization exists for review-quality counterweights. Setting `MET-AI-040`,
`MET-AI-041`, and `MET-AI-042` to measured zero with the policy document as their only evidence,
then adding a measured reviewer-correction row, returns zero findings. The validator requires a
repository-resolvable `commit:`, `doc:`, or `event:` reference, but does not require evidence that
actually observes the claimed zero (`docs/development/tools/ai_contracts.py:2525`).

### MUST-BLOCKER R2 — Numeric and temporal representations are not order- or total-safe

The schema's `number` accepts Python `float` values without finiteness checks
(`docs/development/tools/ai_contracts.py:114`), and minimum/maximum comparisons do not reject NaN or
positive infinity (`docs/development/tools/ai_contracts.py:243`). The semantic helper converts every
number to `float` (`docs/development/tools/ai_contracts.py:1716`). Consequences reproduced through
the production validator:

- `MET-FLOW-017.value = NaN` → zero findings;
- `MET-FLOW-017.value = Infinity` → zero findings;
- a generic rate with `denominator = Infinity`, `numerator = 0`, `value = 0` → zero findings;
- `uncertainty = Infinity` reaches `math.floor(math.log10(...))` and crashes with
  `OverflowError: cannot convert float infinity to integer`
  (`docs/development/tools/ai_contracts.py:2494`).

Timestamps are syntactically limited to nine fractional digits, but
`parse_utc_timestamp` converts the fraction and epoch to binary `float`
(`docs/development/tools/ai_contracts.py:1568`). At the 2026 epoch,
`2026-07-30T09:51:55.000000001Z` and `2026-07-30T09:51:55Z` both become
`1785405115.0`. An event one nanosecond after `captured_at` therefore returns zero findings even
though policy requires `every event ≤ captured_at`
(`docs/development/ai/development-flow-kgi-kpi.md:203`).

The syntax layer correctly rejects trailing-newline anchored-pattern bypasses, non-`Z` offsets, and
Unicode full-width identifiers; semantic parsing rejects impossible calendars and leap second 60.
Those closures do not repair the ordering and totality defects.

### MUST-BLOCKER R3 — Evidence, repository paths, and record seals are shape-checked, not verified

Policy claims that append-only evidence is bound by `prefix_bytes + sha256`
(`docs/development/ai/development-flow-kgi-kpi.md:255`). `check_evidence_prefix` validates locator
shape and capture time only (`docs/development/tools/ai_contracts.py:2983`); it never reads the
prefix bytes or recomputes SHA-256. Changing a prefix digest to 64 zeroes or changing
`prefix_bytes` by one returns zero findings. The exemplar itself says no digest was recomputed from
a repository-reachable file
(`docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json:1336`).

The module already has the symlink-safe `read_repository_artifact` primitive
(`docs/development/tools/ai_contracts.py:358`), but `doc:` resolution bypasses it and uses lexical
validation plus `(root / target).is_file()` (`docs/development/tools/ai_contracts.py:3046`). A final
path component symlink to `/etc/passwd` returns zero findings. Agent-log locator parsing accepts
`logs/../events.jsonl`, `logs/./events.jsonl`, and a doubled leading slash because the regex permits
any `[A-Za-z0-9._/-]+` path without normalized-component validation
(`docs/development/tools/ai_contracts.py:3071`).

`record_sha256` detects accidental edits only. It is recomputed over the mutable record itself
(`docs/development/tools/ai_contracts.py:1792`) and is not bound to the commit that first introduced
the final record, `source_revision`, or a sealed repository blob. Arbitrary edits followed by
recomputation return zero digest findings. This contradicts the published final-record immutability
claim (`docs/development/ai/development-flow-kgi-kpi.md:299`).

The directory checker also validates an in-memory snapshot without re-reading or retaining an
immutable file identity. In an isolated candidate clone, the probe replaced `captured_at` on disk
with `not-a-timestamp` after `read_object` returned but before semantic validation. That checker run
returned zero findings; an immediate second run returned `bad-timestamp`, `schema-min-length`, and
`schema-pattern`. A record can therefore change between load and verdict without invalidating the
verdict.

### MUST-BLOCKER R4 — Candidate, record, and lifecycle graphs are not validated as graphs

Candidate lineage counts roots, rejects self-loops, and resolves predecessor names, but never walks
the graph (`docs/development/tools/ai_contracts.py:2249`). One valid root plus a disjoint two-node
cycle `b1c29fc… → 6dd4527… → b1c29fc…` returns zero findings. This is not a rooted lineage even
though `candidate-lineage-not-rooted` is absent.

Directory validation checks only that `supersedes` and `superseded_by` names exist
(`docs/development/tools/ai_contracts.py:3104`). A two-node record supersession cycle returns zero
findings; reciprocal direction, batch identity, revision monotonicity, acyclicity, and exactly one
terminal head are not checked.

Lifecycle completion is also incomplete. `check_lifecycle` requires only a preceding
`review_decision` and `integration` before `lifecycle_complete`
(`docs/development/tools/ai_contracts.py:2912`). An active record with those three synthetic events
and no Install or Dogfood evidence returns zero findings, despite the canonical eight-stage
authority requiring Close → Install → Dogfood → Retrospect
(`docs/development/project-management.md:68`).

### MUST-BLOCKER R5 — Operational staleness and directory independence are wrong

For active records and `in_progress` metrics, the validator checks only that the due time follows
the record's historical `captured_at` (`docs/development/tools/ai_contracts.py:1807`, `:2507`). It
does not compare against the validation instant. At validation time
`2026-07-31T03:40:49Z`, both an active `next_review_due_at` and a metric `due_at` of
`2026-07-30T09:51:56Z` returned zero findings. The field is therefore an ordering annotation, not
the promised stale-claim control.

Conversely, directory validation calculates one globally newest capture and marks every older
active record superseded (`docs/development/tools/ai_contracts.py:3145`). Two legitimate active
records in different batches are not independent: the older one receives
`superseded-active-development-flow-record`. Staleness must be evaluated per batch and through an
explicit, clock-independent policy input; the current behavior is both under- and over-rejecting.

### SHOULD-BLOCKER R6 — Anti-gaming state remains declarative

`MET-FLOW-019` is recomputed from lane `mutation_status`, and the exemplar honestly reports `2/6`
(`docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json:533`). That closes the original
free-text denominator. It does not bind mutation state to evidence. Changing the four review lanes
from `uncommitted`/`unverified` to `none`, deleting their optional mutation timestamps, and changing
the metric to `2/2 = 1.0` returns zero findings. The denominator can still be improved by declaration.

Artifact counting is improved but remains split-able. The validator allows at most one artifact per
kind (`docs/development/tools/ai_contracts.py:2201`). Adding the same exact evidence and timestamp as
an unused different kind (`counterexample` instead of `source_patch`) returns zero findings. The
same deliverable can therefore become multiple unique artifacts without a unique subject digest.

### SHOULD-BLOCKER R7 — Malformed-but-schema-valid work is not bounded

The principal arrays `lane_runs`, `measurements`, `events`, `evidence_prefixes`, `limitations`,
cohort members, candidates, artifacts, and checkpoint commits have no `maxItems`
(`docs/development/ai/schemas/development-flow-record.v1.schema.json:150`, `:268`, `:305`, `:355`).
A 2,001-event record receives no schema size finding. Validation appends diagnostics per item and
revisits events/references through several passes; JSON `uniqueItems` also materializes canonical
bytes for every item (`docs/development/tools/ai_contracts.py:218`). The current exemplar is small,
but attacker-controlled memory, CPU, and diagnostic growth are not bounded as required by the
ticket. Git commit resolution is appropriately batched, which avoids per-reference subprocess
amplification (`docs/development/tools/ai_contracts.py:1597`).

---

## 4. Manager-Confirmed Launch Counterexamples

All eight manager probes were re-run, not merely inspected. The fresh production output is
`/tmp/kgi-kpi-rereview-fresh.npB5AZ/results.jsonl` at the digest listed in §2.4.

| # | Counterexample | Fresh production result | Severity / affected closure |
|---:|---|---|---|
| 1 | Event at `captured_at + 1 ns` | **0 findings**; both epochs `1785405115.0` | MUST; F1/F4 |
| 2 | `MET-FLOW-017 = NaN` or `Infinity` | **0 findings** for each | MUST; F1/F6/F7 |
| 3 | One root plus disjoint two-node candidate cycle | **0 findings** | MUST; F-S12 and lineage invariant |
| 4 | Active review/metric due one second after capture but overdue at validation | **0 findings** for each | MUST/SHOULD; F6/F-S2 |
| 5 | Two active records in different batches | Older record incorrectly gets `superseded-active-development-flow-record` | SHOULD; F-S2 regression |
| 6 | `doc:` final component symlink to `/etc/passwd` | **0 findings**; `is_file=true`, resolved target `/etc/passwd` | MUST; F9/F-S4 |
| 7 | Agent locator with `../`, `./`, or doubled slash | **0 findings** for all three | MUST; F9/F-S4 |
| 8 | Arbitrary record edit followed by recomputed `record_sha256` | **0 digest findings** | MUST/SHOULD; F9/F-S3 |

---

## 5. Frozen Mutation Matrix And Test Non-Vacuity

### 5.1 Independent replay

- Semantic mutations: **55 total, 55 exact expected codes present**.
- Structural controls: **15 total, 15 exact expected codes present**.
- Baseline reference record: zero findings.
- The replay records every returned code, not just process rejection.

This confirms real closure of many original examples: bad calendars, reversed ordinary intervals,
trailing-newline anchors, status/value mismatch, finite rate arithmetic, unit aliases, duplicate
IDs, missing ordinary references, exact 40-hex commits, required events, repaired original start,
KGI set completeness, filename identity, and basic precision semantics.

### 5.2 Non-vacuity limitations

The repository test uses the production validator itself as the test subject and checks exact
finding codes (`docs/development/tools/test_ai_tools.py:1071`), which is stronger than checking only
non-empty output. However:

- the positive fixture and all mutations are derived from the one exemplar;
- expected arithmetic is not independently implemented;
- the sealed-record test constructs its valid digest with implementation helper `sha256_json`, then
  edits without recomputing (`docs/development/tools/test_ai_tools.py:1238`); it does not model a
  malicious rewrite;
- the directory test encodes the erroneous global-newest behavior as expected
  (`docs/development/tools/test_ai_tools.py:1349`);
- no frozen case covers nanosecond precision, non-finite values or crashes, multi-node cycles,
  actual-clock overdue state, cross-batch independence, symlink escape, normalized locator paths,
  digest-byte recomputation, provenance-bound seals, fabricated cohort members, task-class mismatch,
  per-KGI metric/threshold binding, full lifecycle stage completion, or array bounds.

Passing 79/79 therefore verifies the implemented contract, not the missing contract.

---

## 6. Prior-Finding Disposition

The disposition standard is the ticket's: `closed` only when schema, semantic branch, policy,
exemplar, and a non-vacuous negative test jointly close the smallest falsification case. Any
`partially closed`, `open`, or `regressed` row independently requires `REQUEST_CHANGES`.

### 6.1 MUST F1–F14

| ID | Enforcing rule, repair, and test | Smallest independent falsification | Disposition |
|---|---|---|---|
| F1 | Semantic entry exists at `ai_contracts.py:3087`; production route at `check_ai_contracts.py:63`; 55-code matrix at `test_ai_tools.py:865`; exemplar passes. | NaN/Infinity accepted, `uncertainty=Infinity` crashes, nanosecond ordering lost, graph/path/cohort defects accepted. | **partially closed** |
| F2 | Event enum/minimum in schema `:305`; conditional obligations in policy `:229`; per-lane enforcement `ai_contracts.py:2383`; exemplar has seven lane starts at record `:959` onward; M37 removes all events. | Removing one lane start yields `missing-lane-start-event`; no residual for the original active-record event obligation. | **closed** |
| F3 | Canonical stages in schema `:14` and project management `:68`; queue/review/integration kinds in schema `:315`; event-window binding `ai_contracts.py:2658`; targeted queue test `test_ai_tools.py:1185`. | Missing queue endpoint and unknown `review` stage reject. The separate lifecycle-completion gap is recorded under F1/F-S3, not the original derivability contradiction. | **closed** |
| F4 | Timestamp syntax schema `:9`; strict full match and calendar parse `ai_contracts.py:1568`; ordinary ordering throughout `:1807`, `:1992`, `:2310`, `:2555`; M01–M09. | `captured_at + 1 ns` compares equal after float conversion and is accepted. | **partially closed** |
| F5 | Original/resume fields schema `:150`; policy original-start rule `:203`; start-anchor check `ai_contracts.py:2622`; exemplar MR-list `1319.839 s` at record `:429`; resume tests `test_ai_tools.py:1093`. | Repaired value and uncertainty recompute exactly; resumed-start substitution rejects. | **closed** |
| F6 | Status/value branch `ai_contracts.py:2463`; due fields schema `:275`; policy `:299`; M10–M13 and stale-capture test `test_ai_tools.py:1162`. | Due times already past at validation return zero; NaN/Infinity measured scalar returns zero. | **partially closed** |
| F7 | Metric registry units `ai_contracts.py:1341`; rate checks `:2592`; schema excludes booleans via validator type helper; M14–M18. | Generic rate with infinite denominator and `0/∞ = 0` returns zero; NaN paths are inconsistent rather than fail-closed. | **partially closed** |
| F8 | Dual quantities split into `MET-FLOW-029/030/031...`; exact registry/policy sync at `check_ai_contracts.py:111`; aliases refused; exemplar uses separate IDs at record `:498` and `:516`. | Reusing a deprecated alias or wrong unit rejects. | **closed** |
| F9 | Evidence schemes/policy `development-flow-kgi-kpi.md:255`; prefix shape at schema `:355`; reference resolution `ai_contracts.py:3030`; M25/M27/M28/M33/M34. | Forged prefix SHA/length, traversal locators, and recomputed record digest all return zero; prefix bytes are never read. | **partially closed** |
| F10 | Exact 40-hex schema `:11`; batched commit resolution `ai_contracts.py:1597`; candidate/artifact epoch checks `:2249`; M30–M32. | All ten exemplar commits resolve; non-hex, missing, and abbreviated revisions reject. | **closed** |
| F11 | Five exact KGI schema `:126`; KGI set/metric-status checks `ai_contracts.py:2790`; exemplar binds metrics and stays non-terminal at record `:14`; M21/M22/M35/M43–M47. | Each of KGI-DEV-001..004 becomes `achieved` with only measured-zero throughput and returns zero findings. | **partially closed** |
| F12 | Cohort schema `:86`; policy `:314`; closure/sample/preregistration/open-member/cost checks `ai_contracts.py:2854`; targeted test `test_ai_tools.py:1310`. | Fabricated members, wrong task class, and no 20%/dispersion/non-inferiority arithmetic return zero. | **partially closed** |
| F13 | Provider-required cost registry; missing-component refusal `ai_contracts.py:2767`; policy unavailable-cost rule `:314`; exemplar `MET-AI-060: unavailable` at record `:821`; targeted component test `test_ai_tools.py:1175`. | Arbitrary measured total is never recomputed from provider components and can support fabricated KGI-DEV-005. | **partially closed** |
| F14 | Self-declared integer removed; artifact schema `:237`; policy definition `:279`; per-kind/commit/timestamp checks `ai_contracts.py:2201`; split-commit test `test_ai_tools.py:1132`. | Same evidence-bound deliverable relabeled under an unused artifact kind is accepted as another artifact. | **partially closed** |

### 6.2 SHOULD F-S1–F-S14

| ID | Enforcing rule, repair, and test | Smallest independent falsification | Disposition |
|---|---|---|---|
| F-S1 | 615 test lines added; exact 55 semantic and 15 structural cases; positive record and directory controls. | Independent replay matched all 70 expected codes. Missing adjacent cases are residual findings, not absence of the requested matrix. | **closed** |
| F-S2 | Recursive existence gate `check_ai_contracts.py:63`; one active per batch `ai_contracts.py:3135`; final-window rule `:1823`; review-due field. | Actual-clock overdue claims pass; different batches interfere through globally newest capture. | **partially closed** |
| F-S3 | `revision`, `supersedes`, `superseded_by`, filename binding, final digest in schema `:64`; identity/digest checks `ai_contracts.py:1763`; policy `:299`. | Two-node supersession cycle passes; digest is self-recomputable and not provenance-bound. | **partially closed** |
| F-S4 | Enumerated schemes and internal indexes `ai_contracts.py:1275`, `:1847`, `:3004`; M26–M29/M38. | `doc:` symlink escapes root; prefix digest is unresolved; fabricated cohort member IDs need not resolve. | **partially closed** |
| F-S5 | Exact metric registry `ai_contracts.py:1341`, KGI IDs `:1263`, unknown/alias refusal `:2433` and `:2814`; M19–M21. | Invented/typo IDs reject. | **closed** |
| F-S6 | Alias/deprecation table `ai_contracts.py:1551`; published mapping `development-flow-kgi-kpi.md:151`; policy-registry test `test_ai_tools.py:1402`. | Deprecated aliases reject and cannot create duplicate recordable series. | **closed** |
| F-S7 | Phase-2 status updated at scorecard `:727`; fixture/command, provisional threshold, deletion condition, and waivers documented. | Runtime version remains optional; cohort task-class membership is not enforced; evidence fixture digests are not reproducible. | **partially closed** |
| F-S8 | `MET-AI-073/074/076` in registry `ai_contracts.py:1534`; policy table `:120`; required-zero completeness `:2744`; exemplar marks all absent at record `:891`. | Required rows cannot be omitted and absent is distinguished from measured zero. | **closed** |
| F-S9 | Runtime/instrument fields in schema `:184`; model/instrument checks `ai_contracts.py:2096` and `:2342`; safety rows explicit absent in exemplar. | Removing every agent `runtime_version` returns zero; KGI-DEV-003/004 can be achieved via unrelated zero throughput. | **partially closed** |
| F-S10 | Mutation-state denominator recomputed `ai_contracts.py:2727`; policy defines inclusion; exemplar repairs `2/6` at record `:533`; test narrows declared states. | Reclassifying review lanes to `mutation_status: none` and changing `2/6` to `2/2` returns zero. | **partially closed** |
| F-S11 | Policy distinguishes mutation/trusted signal `:250`; equality rejection `ai_contracts.py:2174`; event binding `:2400`; M48 and targeted equality test. | Uncommitted edit equality rejects; checkpointed mutation requires timestamp/event. | **closed** |
| F-S12 | Candidate schema `:222`; lineage/root checks `ai_contracts.py:2249`; first-pass bound `:2710`; recommit test `test_ai_tools.py:1109`. | A root plus a disjoint two-node cycle returns zero. | **partially closed** |
| F-S13 | Review/rework metrics require measured safety guardrails `ai_contracts.py:2753`; all guardrail rows are required; policy retains anti-softening intent. | Arbitrary measured zeroes for 040/041/042, each backed only by the policy document, allow a measured reviewer-correction row with zero findings. | **partially closed** |
| F-S14 | Precision/uncertainty schema `:280`; decimal hygiene `ai_contracts.py:2483`; time-basis consistency `:2946`; exemplar rounds F010 to `1975 ±30 s` at record `:448`. | Non-finite uncertainty is accepted by schema and can crash validation. | **partially closed** |

### 6.3 COULD F-C1–F-C7

| ID | Enforcing rule, repair, and test | Smallest independent falsification | Disposition |
|---|---|---|---|
| F-C1 | Separate `model`, `instrument`, `runtime_version` fields; provider/model and instrument rules `ai_contracts.py:2096`, `:2342`; exemplar system/human events use instruments; test `test_ai_tools.py:1271`. | Moving `linux-proc-du` into `model` rejects. Runtime optionality remains F-S9. | **closed** |
| F-C2 | Lane-start event now exists for storage lane at record `:959`; evidence locator/provider/lane binding is checked. | Storage review lane still has `artifacts: []`, only a mutable unverified prefix, no trusted-signal event, and an outcome claiming six or more blockers at record `:98`. | **partially closed** |
| F-C3 | Directory README added; recursive `rglob` and unexpected-file rejection `check_ai_contracts.py:63`; tests `test_ai_tools.py:1372`. | Nested malformed record is found; `.jsonl` sibling rejects. | **closed** |
| F-C4 | Flow validation is before the unrelated matrix early return `check_ai_contracts.py:239`; comment and route are explicit. | Unreadable/missing matrix no longer skips flow validation. | **closed** |
| F-C5 | Redaction rule published at policy `:269`; prefix redaction enum schema `:355`; safety metric 074 required; exemplar explicitly says operator-only/not scanned. | The requested rule now exists and the absence is not fabricated as zero. | **closed** |
| F-C6 | `MET-FLOW-028` throughput added in registry `ai_contracts.py:1521`, policy `:131`, exemplar measured zero at record `:808`, and alias `MET-AI-001` documented. | Throughput is now a recordable north-star metric. | **closed** |
| F-C7 | Batch ID pattern schema `:68`; limitations min length and blacklist `ai_contracts.py:2946`; M53/M55. | `limitations: ["No known limitations were identified."]` returns zero; honesty remains prose-pattern based. | **partially closed** |

Disposition totals: **15 closed, 20 partially closed, 0 fully open, 0 separately regressed**. The
cross-batch newest-record behavior is a regression within F-S2, so that row cannot close even though
its top-level disposition is `partially closed`.

---

## 7. Schema / Semantic / Policy / Exemplar / Test Alignment

| Surface | Aligned strengths | Residual disagreement |
|---|---|---|
| Timestamp contract | Schema allows 1–9 fractional digits; semantic parser requires UTC `Z`; policy says RFC 3339 UTC. | Parser returns float, so nine-digit syntax overstates comparison precision. |
| Number contract | Booleans are rejected; finite ordinary values are checked against units/rates/intervals. | Schema and semantics accept NaN/Infinity; uncertainty Infinity crashes. |
| Evidence contract | Schemes and local identities resolve; exact commits are batched and verified. | Policy says digest-bound evidence, but bytes/digest are never resolved; `doc:` skips existing safe reader. |
| Record lifecycle | Filename/revision/status/digest fields and one-active-per-batch rule exist. | Global cross-batch freshness, supersession cycles, actual overdue time, and immutable provenance are not enforced. |
| KGI/cohort | Exact KGI set, metric status, cohort field presence, sample count, censoring, and missing cost are checked. | No per-KGI metric/threshold semantics; cohort member identity/class and outcome arithmetic are absent. |
| Canonical lifecycle | Eight stages and review/integrate gates are reconciled. | `lifecycle_complete` needs only review and integration, not Install/Dogfood or all stage transitions. |
| Tests | Exact expected-code matrix is real and passes through production helper. | It codifies global-newest behavior and omits every blocking representation/graph/path/cohort case. |

---

## 8. Exemplar Arithmetic, Reference, And Honesty Audit

### 8.1 Arithmetic recomputation

All 43 measurement rows were enumerated. Every row with defined interval or rate arithmetic was
recomputed independently:

| Measurement | Recorded | Independent result | Assessment |
|---|---:|---:|---|
| `MET-FLOW-002` mutation lane | `3964.9522 s` | exact window span `3964.9522 s` | Consistent, estimated ±54.1108 s |
| `MET-FLOW-002` MR-list lane | `3479.743294 s` | exact window span `3479.743294 s` | Consistent, estimated ±47.770178 s |
| `MET-FLOW-003/-004` mutation lane | `1317.369787 s` | exact original-start span | Consistent |
| `MET-FLOW-003/-004` MR-list lane | `1319.839 s` | `08:39:11.161Z → 09:01:11Z` | Corrects original F5; consistent |
| MR-list resume | `18.380472 s` interruption | `08:44:56.252Z → 08:45:14.632472Z` | Consistent |
| `MET-FLOW-010` hedge | `1975 s`, uncertainty 30 | exact span `1974.668417 s`; delta `0.331583 s` | Correct precision treatment |
| `MET-FLOW-012` | `0`, `0/1` | `0` | Consistent |
| `MET-FLOW-017` | `345.091 s` | exact declared interval | Consistent for finite value |
| `MET-FLOW-030` | `10 count` | bound event states ten retries | Internally consistent; prefix not independently verified |
| `MET-FLOW-019` | `0.333333`, `2/6` | `0.333333333...`; delta `-3.33e-7` | Inside tolerance; denominator states honest as committed |
| `MET-FLOW-020` restart estimate | `105226698752 bytes` | exactly `98 GiB` | Consistent |
| `MET-FLOW-021` restart estimate | `98569499443 bytes` | rounded `91.8 GiB = 98569499443.2` | Consistent rounding |
| `MET-FLOW-020/021/022/023` samples | raw byte values | point-sample windows are equal; 0 swap | Internally consistent |
| `MET-FLOW-024` samples | `994060`, `1382958 tokens` | delta `388898` | `MET-FLOW-031` exactly matches delta |
| `MET-FLOW-025` costs | `2.29561`, `17.520932`, `27.294168 USD` | provider-prefix claims only | Not repository-verifiable because prefix digests are not checked |
| `MET-FLOW-026` | estimated `0 count` | limited to committed source/worktree reconstruction | Honest scope; not evidence for lost user work |
| `MET-FLOW-028` | measured `0 count` | no integration event in record | Consistent |

No percent-valued row exists in the exemplar. The ratio-to-percent branch was exercised in the
frozen mutation matrix.

### 8.2 Reference recomputation

- All ten distinct commit references resolve as full 40-character commit objects.
- Candidate and artifact timestamps match commit instants within the validator's one-second rule.
- Both ordinary `doc:` references exist in the repository.
- All same-record `measurement:`, `event:`, `lane:`, `artifact:`, and `prefix:` names resolve.
- Seven agent prefixes have syntactically valid provider/lane locators and positive byte counts.
- None of the seven prefix digests was independently recomputable from the repository; the current
  record discloses this limitation.

### 8.3 Current record honesty

The committed active record is materially honest:

- all five KGIs are `in_progress` or `at_risk`, none `achieved`;
- ten zero-target safety metrics are `absent`, not false zeroes;
- cross-provider cost is `unavailable`;
- checkpoint coverage is `2/6`, not the old selected `2/2`;
- throughput is measured zero;
- no cohort or 20-percent result is claimed;
- MR-list start and Fugu stop uncertainty are stated explicitly.

The honest exemplar does not cure the executable contract. The checker can transform equivalent
malicious records into terminal success, and its own policy says a passing semantic check makes a
record usable for decisions.

---

## 9. Decision Capability Audit

| Question | Assessment |
|---|---|
| Is the contract mechanically valid? | **No for decision authority.** The current fixture passes schema and semantics, but production counterexamples bypass or crash core invariants. |
| Is the current CFR-168 record factually honest? | **Mostly yes.** Its arithmetic, ordinary references, absent/unavailable distinctions, and non-terminal KGI narrative are candid. Prefix-backed provider claims remain independently unverifiable. |
| Which KPIs are usable now? | Finite interval diagnostics, recorded resource point samples, the finite `0/1` first-pass row, declared `2/6` checkpoint state, and measured zero throughput are internally usable as manager notes. |
| Which are not decision-capable? | Trusted-integration lead time is in progress; completeness is in progress; flake classification is in progress; cross-provider cost is unavailable; every safety metric is absent; no cohort exists; flow efficiency/useful parallelism are absent; provider cost prefixes are not verified. |
| Can `KGI-DEV-005` be evaluated now? | **No.** The exemplar correctly refuses it, and the validator nevertheless permits a fabricated terminal decision. |
| Can policy authorize a process change from incomplete evidence? | **Mechanically yes, incorrectly.** The policy treats a passing semantic validator as the decision-grade boundary, while the validator accepts fabricated cohort/KGI records and false evidence bindings. |

An honest “not decision-capable yet” record is acceptable. The candidate's current record largely
does that. The blocking defect is that the contract cannot enforce the same honesty on the next
record.

---

## 10. Complexity And Stability Audit

### 10.1 Positive properties

- Commit-kind resolution uses one `git cat-file --batch-check` subprocess for unique revisions.
- Commit timestamps are requested in one `git show` call and cached.
- Major indexes use sets/dictionaries; ordinary validation is approximately linear in record size.
- Findings are deterministic for the frozen 70-case matrix.
- Reordered top-level arrays return zero findings, so semantics do not depend on record ordering.

### 10.2 Residual risks

- Principal arrays are unbounded; diagnostics and canonicalized `uniqueItems` bytes can grow without
  a contract limit.
- `check_all_evidence_refs` builds another holder list over already indexed content.
- Candidate and record supersession are not traversed, so graph work is incomplete rather than
  safely bounded.
- One malformed-but-schema-valid uncertainty crashes instead of returning a stable finding.
- Global caches key by root/revision and can preserve a stale negative/epoch result across mutable
  repository state in a long-lived process; commits are immutable, but repository reachability and
  the requested time-of-check/time-of-use model are not documented.
- `doc:` checks are TOCTOU- and symlink-unsafe because no bytes or stable descriptor are retained.
- Flow-record files themselves are TOCTOU-unsafe: mutation after load is invisible to the current
  checker invocation.

---

## 11. Required Next Action And Residual Owners

The exact candidate must not be integrated. A new repair candidate must close all residual rows and
add production-route tests for every case below.

| Owner | Required residual closure |
|---|---|
| Semantic-validator owner | Replace float timestamps with exact integer nanoseconds or exact decimal tuples; reject every non-finite numeric field before arithmetic; turn all malformed values into stable findings. |
| KGI/cohort owner | Publish and enforce per-KGI allowed metrics and target arithmetic; require every cohort member to resolve exactly once, match the declared class, bind baseline/comparison arms, and recompute median, dispersion, 20-percent improvement, cost total, and non-inferiority. |
| Evidence/reference owner | Reuse `read_repository_artifact` for `doc:`; normalize agent locator paths; resolve accessible evidence bytes and recompute `prefix_bytes`/SHA-256; explicitly refuse unverifiable prefixes for decision evidence. |
| Record-lifecycle owner | Bind final-record bytes to the first introducing commit/blob or another immutable repository subject; validate reciprocal, same-batch, monotone, acyclic supersession with one head; make active freshness per-batch and clock-independent or inject an explicit validation instant. |
| Lineage/lifecycle owner | Traverse candidate lineage for acyclicity/connectivity; require lifecycle completion through canonical Close, Install, Dogfood, and Retrospect evidence, not only review/integration. |
| Anti-gaming owner | Bind mutation state to events/artifacts; identify artifacts by immutable subject digest so one deliverable cannot be relabeled by kind; make checkpoint denominator evidence-derived. |
| Schema/test owner | Add finite-number semantics and practical `maxItems` bounds; add all manager and adjacent probes to the production checker/directory route; do not encode cross-batch interference as expected behavior. |
| Docs/policy owner | Remove or narrow every claim that evidence prefixes, final immutability, cohort class separation, operational overdue handling, or lifecycle completeness are enforced until the executable checks exist. |

Required focused acceptance evidence for the next candidate:

1. every frozen 55/15 case still matches its exact code;
2. all eight manager probes reject with stable specific codes;
3. all adjacent probes in §§3–4 reject without crashes;
4. a valid two-batch directory passes independently;
5. a valid sealed final record is tied to immutable repository provenance;
6. one complete valid cohort produces independently recomputed KGI-DEV-005 arithmetic, while
   fabricated/mixed/unresolved members and arbitrary totals reject;
7. required repository gates pass.

---

## 12. Handoff

- **Decision:** `REQUEST_CHANGES` on exact candidate
  `0ae583ef8b4cfd26e10ebc1f13564d32a7f5e7e1`.
- **Files changed by reviewer:** only
  `docs/development/ai/swarm-results/development-flow-kgi-kpi-zero-residual-rereview-fugu-20260730/review.md`.
- **Substantive report commit:** `5082a295fa898d24d9f17602505e5eea806d2601`
  (`docs(ai): add KGI KPI zero-residual rereview`).
- **Clean-worktree confirmation:** immediately after that report-only commit,
  `git status --short`, `git diff --check`, and `git diff --name-only` produced no output. This
  metadata/whitespace follow-up is also required to finish with an empty `git status --short`.
- **Validation:** 55/55 semantic codes, 15/15 structural codes, 79/79 repository tests, contracts,
  AI-development aggregate, project-docs, and candidate diff check all passed; adversarial production
  probes above still fail the contract.
- **Risks:** no product source was changed; the only integration risk is accepting the candidate
  despite mechanically passing but decision-unsafe validation.
- **Next action:** manager returns the ticket to the repair owners above; do not integrate, Close,
  Install, Dogfood, or Retrospect this candidate.
