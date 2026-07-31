# CFR-168 Control-Plane Composition

Status: blocked  
Owner: portfolio manager  
Execution role: Claude Opus 5 implementer, fresh Claude Opus 5 reviewer  
Canonical packet base: `f420eb11dc49a4c7e8f98adb8e7f4c21b20fe5be`  
Codex worker fallback: forbidden  
Product mutation: forbidden

## Objective

Port the independently accepted development-flow KGI/KPI implementation onto
an already accepted, integrated source-keyed Cargo-gate canonical head without
losing either contract.

This is a semantic port, not a branch merge. The historical Cargo and KGI/KPI
lineages diverged from different canonical bases and overlap in five files.
Acceptance of the Cargo canonical port and the KGI/KPI source candidate will
not constitute acceptance of their composition.

## Scope

- Port the accepted KGI/KPI source range onto the accepted Cargo-enabled
  canonical head.
- Resolve the KGI/KPI 17-path range and its five collisions with the
  already-integrated Cargo contract.
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
| Canonical source-keyed Cargo port | blocked packet `cfr168-source-keyed-cargo-canonical-port-20260731.md` | Exact port commit is integrated into canonical after its own fresh `ACCEPT`. Candidate `4e5c0e7` acceptance alone is insufficient. |
| Development-flow KGI/KPI | `753c8db103579d021795d3fe6a97c5d45d4a111a` | Exact round-3 review `12f9d0b` returned `REQUEST_CHANGES`; a bounded original-session successor and fresh exact `ACCEPT` are required. |

If the KGI/KPI candidate receives `REQUEST_CHANGES`, this packet stays blocked
and must be rebound to its accepted successor. If the Cargo canonical port is
rejected, repair and accept that port first. Do not compose rejected work for
convenience.

## Exact Source Ranges

After both admission rows are canonical, create a fresh worktree from the
then-current Cargo-enabled canonical head. Use the KGI/KPI range as port source
and the Cargo range only as a protected-contract reference:

```text
Protected Cargo reference: 893cbcad2b1f2a8455a5c256246c8110d3fcbdd6..4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912
KGI/KPI port source:       9dd0da4f2ae55742ee51c1671aab6308994de24c..753c8db103579d021795d3fe6a97c5d45d4a111a
```

Do not cherry-pick the KGI/KPI history or replace files with source blobs.
Its history contains stale management-document versions relative to canonical,
and a whole-branch merge would make unrelated ancestry look like reviewed
composition evidence.

## Manager Preflight

Read-only evidence
`docs/development/ai/swarm-results/cfr168-kgi-kpi-control-plane-composition-preflight-20260731/evidence.md`
at SHA-256
`07e7fb0eb43550c3f496a9c9427e65cb956edf695349f575fe5494b36ed0f910`
binds the KGI/KPI patch/path census, current canonical collisions, and
normalized test-name sets.

The KGI/KPI source adds 87 unique test names relative to its base; none
collides with the Cargo candidate's added set. The source-candidate union has
255 names at the preflight snapshot, but that number is not the composition
criterion. Preserve the accepted Cargo-port set, the 87-name KGI/KPI addition,
and later canonical tests exactly once, and execute both families.

The preflight predates the accepted Cargo canonical port. Recompute every blob
and name set after both admission rows are satisfied. Do not update the
preflight to fit the result or use its comparison snapshot as a port base.

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

These paths are protected by the earlier canonical Cargo port. Do not modify
them in this KGI/KPI composition:

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

Mechanically extract every test method added by the accepted KGI/KPI source
range and snapshot every Cargo source-gate test present on the accepted
canonical base. Every name must still exist exactly once after composition
and every method must execute. Do not use a total test-count threshold as a
substitute: the base counts differ, and a duplicated or silently dropped test
can leave a plausible total.

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

1. compare the mechanically extracted KGI/KPI test-name set and the protected
   canonical Cargo test-name snapshot with the composed tree;
2. run each accepted candidate's manager counterexample/probe set against the
   composition;
3. demonstrate at least one deliberate source-gate regression and one
   deliberate KGI/KPI semantic regression fail the composed gates;
4. prove all validation commands bind the exact composed HEAD; and
5. record monotonic duration, complete-log digest, return code, and
   `UNAVAILABLE` evidence for every gate.

No Cargo or Rust command is authorized by this composition task.

## Write Boundary

The implementer may change only the 17 KGI/KPI source-range paths (the five
overlap paths plus the twelve KGI/KPI-owned non-overlap paths) plus:

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

- Cargo-enabled canonical base, accepted Cargo-port identity, accepted KGI/KPI
  source identity, composed HEAD, branch, and clean-worktree proof;
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
