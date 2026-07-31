# CFR-168 Control-Plane Composition

Status: blocked  
Owner: portfolio manager  
Execution role: Claude Opus 5 implementer, fresh Claude Opus 5 reviewer  
Canonical packet base: `f420eb11dc49a4c7e8f98adb8e7f4c21b20fe5be`  
Codex worker fallback: forbidden  
Product mutation: forbidden

## Objective

Compose the independently accepted source-keyed Cargo-gate implementation and
the independently accepted development-flow KGI/KPI implementation onto one
fresh canonical control-plane head without losing either contract.

This is a semantic port, not a branch merge. The two candidate lineages
diverged from different historical canonical bases and overlap in five files.
Separate acceptance of both candidates will not constitute acceptance of their
composition.

## Scope

- Port only the accepted control-plane behavior in the two exact source
  ranges.
- Resolve the 31-path union and the five overlapping paths from a fresh
  canonical worktree.
- Preserve both executable contract families, their tests, current canonical
  incident evidence, and current routing policy.
- Produce one exact composition candidate, one result, and one independent
  composition review.

## Non-Goals

- No CodeFire product, Rust, packaging, install, dogfood, or capability
  mutation.
- No repair of either input candidate after a rejecting review.
- No wholesale branch merge, history rewrite, or candidate-report edit.
- No inference that two separately accepted inputs make their composition
  accepted.
- No fixed combined test-count target in place of test-name and behavior
  preservation.

## Admission Blockers

Do not launch this task until both rows are satisfied:

| Input | Exact candidate | Required evidence |
|---|---|---|
| Source-keyed Cargo gate | `4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912` | Fresh exact independent `ACCEPT`; current reviewer session `d089c8ae-8364-49af-bcd1-b9bf4b741ea1` has no report because Claude quota interrupted it. |
| Development-flow KGI/KPI | `753c8db103579d021795d3fe6a97c5d45d4a111a` | Fresh exact independent `ACCEPT`; current reviewer session `60548555-ff29-4fd9-9360-2820a663ee54` has no report because Claude quota interrupted it. |

If either candidate receives `REQUEST_CHANGES`, this packet stays blocked and
must be rebound to its accepted successor. Do not compose a rejected candidate
for convenience.

## Exact Source Ranges

After both acceptance reports are canonical, create a fresh worktree from the
then-current canonical head and use these ranges only as source material:

```text
Cargo gate: 893cbcad2b1f2a8455a5c256246c8110d3fcbdd6..4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912
KGI/KPI:    9dd0da4f2ae55742ee51c1671aab6308994de24c..753c8db103579d021795d3fe6a97c5d45d4a111a
```

Do not cherry-pick either complete history. Both histories contain stale
management-document versions relative to canonical, and a whole-branch merge
would make unrelated ancestry look like reviewed composition evidence.

## Collision Inventory

The Cargo range changes 19 paths. The KGI/KPI range changes 17 paths. Their
31-path union has these five real overlaps:

```text
docs/development/ai/README.md
docs/development/ai/acceleration-master-plan.md
docs/development/history.md
docs/development/tools/ai_contracts.py
docs/development/tools/test_ai_tools.py
```

An exploratory three-tree merge already produces conflicts in documentation,
`ai_contracts.py`, and `test_ai_tools.py`. Resolving only textual conflict
markers is not sufficient.

### Cargo-owned non-overlap

Port the accepted Cargo-gate behavior from:

```text
docs/development/ai/failure-cookbook.md
docs/development/ai/source-keyed-cargo-gates.md
docs/development/ai/stage5-manual-control-runbook.md
docs/development/ai/swarm-batches/cfr168-rust-file-doctor-review-note-closure-rereview-claude-20260730.md
docs/development/ai/swarm-batches/cfr168-rust-http-mr-preflight-evidence-closure-rereview-claude-20260730.md
docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md
docs/development/tools/build_source_bound_rust.py
docs/development/tools/check_ai_acceleration_docs.py
docs/development/tools/check_rust_execution_profile.py
docs/development/tools/pdeath_exec.py
docs/development/tools/probe_validation_capabilities.py
docs/development/tools/readonly_source_exec.py
docs/development/tools/run_source_keyed_cargo_gate.py
docs/development/tools/run_timed_gate.py
```

### KGI/KPI-owned non-overlap

Port the accepted KGI/KPI behavior from:

```text
docs/development/README.md
docs/development/ai/development-flow-kgi-kpi.md
docs/development/ai/flow-metrics/README.md
docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json
docs/development/ai/schemas/README.md
docs/development/ai/schemas/development-flow-record.v1.schema.json
docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r2-20260731.md
docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r3-20260731.md
docs/development/ai/task-packets/development-flow-kgi-kpi-zero-residual-repair-20260730.md
docs/development/exploration/phase-2/experiment-catalog-and-kpi-scorecard.md
docs/development/project-management.md
docs/development/tools/check_ai_contracts.py
```

Canonical incident/review/ledger files added after either source range remain
canonical. Never replace them with the candidate branch's older copy.

## Overlap Resolution Contract

### `ai_contracts.py`

Preserve both independently reviewed contract families:

- Cargo/source-gate environment, routing, provenance, and bounded-evidence
  validation; and
- exact finite-number parsing, KGI/cohort derivation, sealing provenance,
  candidate/supersession graph validation, evidence-prefix verification,
  lifecycle validation, and anti-gaming semantics.

Resolve at function and invariant level. A last-writer-wins file copy is
forbidden.

### `test_ai_tools.py`

Mechanically extract every test method added by each accepted source range.
Every name must still exist exactly once after composition and every method
must execute. Do not use a total test-count threshold as a substitute: the
base counts differ, and a duplicated or silently dropped test can leave a
plausible total.

Additionally run the source-gate test classes and the development-flow
contract test classes separately so a zero-selection or discovery omission
cannot hide behind the combined suite.

### Policy and history documents

Start from canonical text. Add only claims that the composed executable
contracts enforce. Preserve:

- low-Codex routing and provider-incident corrections;
- the review-root-cause reliability gate;
- source-keyed Cargo provenance and cleanup limitations;
- exact KGI/KPI derivation and sealing limitations; and
- the complete chronological history of both candidate reviews.

Do not restore a candidate branch's stale current-state language or superseded
provider-cause attribution.

## Composition Invariants

1. No file under `codefire`, `crates/`, `tests/`, product schemas, packaging,
   or dogfood projects changes.
2. Direct or indirect unkeyed Rust execution remains fail-closed.
3. Source, toolchain, test-selection, artifact, cleanup, and result identities
   remain bound to the accepted source-gate contract.
4. Schema-valid KGI/KPI records cannot self-authorize decisions, crash numeric
   validation, reseal edited content, form invalid supersession graphs, or
   declare lifecycle completion without evidence.
5. The composed test suite contains every accepted candidate test exactly once.
6. Current canonical incidents, review reports, and routing decisions are not
   rewritten by candidate history.
7. A green composition gate is evidence only for the exact composed commit.

## Validation

Run from the clean composed worktree:

```bash
python3 -m py_compile docs/development/tools/*.py
python3 docs/development/tools/test_ai_tools.py
python3 docs/development/tools/check_ai_contracts.py
python3 docs/development/tools/check_ai_acceleration_docs.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_ai_development.py --timeout 180
git diff --check
```

Also:

1. compare the mechanically extracted test-name sets from both accepted ranges
   with the composed tree;
2. run each accepted candidate's manager counterexample/probe set against the
   composition;
3. demonstrate at least one deliberate source-gate regression and one
   deliberate KGI/KPI semantic regression fail the composed gates;
4. prove all validation commands bind the exact composed HEAD; and
5. record monotonic duration, complete-log digest, return code, and
   `UNAVAILABLE` evidence for every gate.

No Cargo or Rust command is authorized by this composition task.

## Write Boundary

The implementer may change only the 31-path union above plus:

```text
docs/development/ai/swarm-results/cfr168-control-plane-composition-claude-20260731/result.md
```

The fresh reviewer may add only:

```text
docs/development/ai/swarm-results/cfr168-control-plane-composition-rereview-claude-20260731/review.md
```

The implementer and reviewer must use separate sessions/worktrees. The
implementer commits one coherent composition and leaves a clean worktree.
Only the exact reviewer verdict `ACCEPT` authorizes manager integration.

## Handoff

The implementer returns:

- canonical base, both accepted source identities, composed HEAD, branch, and
  clean-worktree proof;
- exact changed paths and a five-file overlap disposition;
- mechanically compared source-test-name and composed-test-name sets;
- every validation command, return code, monotonic duration, complete-log
  digest, and unavailable row;
- both deliberate-regression results; and
- the result commit and report SHA-256.

The reviewer returns:

- exact composed HEAD and result digest;
- one terminal `ACCEPT` or `REQUEST_CHANGES`;
- severity-ordered findings with reproduced or object-level evidence;
- explicit disposition of both accepted input contract families and all five
  overlap files; and
- report commit, report SHA-256, and clean-worktree proof.
