# CFR-168 Source-Keyed Cargo Gate Canonical Port

Status: blocked  
Owner: portfolio manager  
Execution role: Claude Opus 5 implementer, fresh Claude Opus 5 reviewer  
Canonical packet base: `c273a0c6cb231d000d6289f6de5615fb9f1320be`  
Codex worker fallback: forbidden  
Product mutation: forbidden

## Objective

Port the independently accepted source-keyed Cargo-gate implementation onto a
fresh then-current canonical head, obtain an exact review of that port, and
integrate it without waiting for the independent KGI/KPI candidate.

An accepted canonical port releases the source-bound Rust validation critical
path. KGI/KPI is deliberately a later composition dependency, not a
prerequisite for this port.

## Scope

- Source range
  `893cbcad2b1f2a8455a5c256246c8110d3fcbdd6..4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912`.
- The exact 19 source-range paths plus one port result.
- Semantic port from a fresh canonical worktree, complete Python-only
  validation, and a fresh independent exact-port review.
- Preservation of all canonical provider incidents, review-root-cause gates,
  KGI/KPI work already present in canonical, and current routing policy.

## Non-Goals

- No KGI/KPI candidate port; it has its own later composition packet.
- No Cargo, Rust, CodeFire product, install, dogfood, packaging, capability, or
  issue-closure mutation.
- No whole-branch merge or complete-history cherry-pick.
- No use of an implementer result, worker gate, or clean tree as an acceptance
  verdict.
- No automatic Codex fallback.

## Admission Blocker

Exact candidate
`4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912` must first receive a fresh
independent `ACCEPT`.

Current reviewer session `d089c8ae-8364-49af-bcd1-b9bf4b741ea1` stopped on
Claude quota before writing a report or verdict. Resume that same reviewer
after verified capacity. If it returns `REQUEST_CHANGES`, this packet remains
blocked and must be rebound to the accepted successor.

## Why A Port Is Required

Candidate `4e5c0e7` is three commits beyond historical base
`893cbcad2b1f2a8455a5c256246c8110d3fcbdd6`:

```text
bfbbc2b2bdc22f88679e5e1dc794665d863304a1
aaa39b61b37df273fe4d57e274ed8e1321e788ad
4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912
```

The current canonical line has newer management, routing, provider-incident,
and review-method evidence. Cherry-picking or merging the stale history would
overwrite or conflict with that evidence and would not prove the combined
tree.

## Manager Preflight

Read-only evidence
`docs/development/ai/swarm-results/cfr168-source-keyed-cargo-canonical-port-preflight-20260731/evidence.md`
at SHA-256
`90449137903e2b15f0fd780cee7a3e9809cb54e7d1b7da18cce885282579ba34`
binds the source-range patch digest, path status, comparison-snapshot blob
collisions, and normalized test-name-set digests.

At canonical snapshot `9c1eefd`, only the File Doctor and Rust HTTP packet
paths had diverged from the historical base among the 19 source-range paths.
The source candidate has 104 test names absent from the historical base and
replaces one permissive shared-root test with separate fail-closed and admitted
source-keyed cases.

These are preflight observations, not frozen port outputs. Recompute every
path blob and test-name set from the then-current canonical base after
admission. A changed observation requires explicit disposition; it does not
authorize editing the preflight or copying a candidate blob wholesale.

## Exact Write Set

```text
docs/development/ai/README.md
docs/development/ai/acceleration-master-plan.md
docs/development/ai/failure-cookbook.md
docs/development/ai/source-keyed-cargo-gates.md
docs/development/ai/stage5-manual-control-runbook.md
docs/development/ai/swarm-batches/cfr168-rust-file-doctor-review-note-closure-rereview-claude-20260730.md
docs/development/ai/swarm-batches/cfr168-rust-http-mr-preflight-evidence-closure-rereview-claude-20260730.md
docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md
docs/development/history.md
docs/development/tools/ai_contracts.py
docs/development/tools/build_source_bound_rust.py
docs/development/tools/check_ai_acceleration_docs.py
docs/development/tools/check_rust_execution_profile.py
docs/development/tools/pdeath_exec.py
docs/development/tools/probe_validation_capabilities.py
docs/development/tools/readonly_source_exec.py
docs/development/tools/run_source_keyed_cargo_gate.py
docs/development/tools/run_timed_gate.py
docs/development/tools/test_ai_tools.py
```

The implementer may additionally create only:

```text
docs/development/ai/swarm-results/cfr168-source-keyed-cargo-canonical-port-claude-20260731/result.md
```

The reviewer may create only:

```text
docs/development/ai/swarm-results/cfr168-source-keyed-cargo-canonical-port-rereview-claude-20260731/review.md
```

## Port Contract

1. Start from the then-current clean canonical HEAD after the candidate
   acceptance report is integrated.
2. Use the exact source range as reference; do not cherry-pick it.
3. For canonical files already changed since the historical base, apply the
   smallest semantic delta and retain all newer unrelated content.
4. Mechanically extract every test method added by the accepted source range.
   Every name must exist exactly once and execute on the port.
5. Preserve all accepted fail-closed behavior:
   - exact source/worktree/dirty-state key;
   - pinned toolchain and executable identities;
   - private namespace and propagation proof;
   - complement-based descriptor closure;
   - nonzero exact test selection and bounded parser state;
   - source/candidate immutability across execution;
   - durable bounded result/log/artifact publication;
   - bounded persistent storage and crash cleanup;
   - no false hardlink-exact reclaimed-byte or charge-back claims;
   - finite timeout and process-tree termination; and
   - refusal of direct and known indirect unkeyed Rust execution.
6. Preserve canonical KGI/KPI files and tests already present. The later
   `753c8db` semantic KGI/KPI port remains separate.
7. Record the source range, canonical base, composed tree, and every manual
   conflict disposition.

## Validation

Run only Python/docs/control-plane gates:

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

Additional mandatory evidence:

1. accepted-source test-name set equals the canonical-port test-name set for
   every source-range addition;
2. source-gate tests execute separately with nonzero selection;
3. the accepted candidate's manager counterexamples pass on the port;
4. at least one deliberate descriptor-leak regression, one propagation-claim
   regression, and one hardlink-accounting overclaim are rejected;
5. a pre-existing canonical KGI/KPI test subset still runs and passes;
6. all gates bind the exact port HEAD with monotonic duration and complete-log
   digest; and
7. both source and port worktrees remain clean.

No Cargo or Rust process may start.

## Release After Acceptance

Only a fresh `ACCEPT` on the exact canonical-port commit authorizes manager
integration. After integration, the source-keyed validation scheduler may
admit Rust candidate gates independently of KGI/KPI review state, subject to
live resource admission and Rust-heavy serialization.

KGI/KPI then ports onto that Cargo-enabled canonical head through
`cfr168-control-plane-composition-20260731.md`.

## Handoff

The implementer returns:

- accepted candidate report identity and digest;
- source range, canonical base, port commit/tree, branch, and clean state;
- exact changed paths and per-file port disposition;
- source/port test-name-set comparison;
- counterexample and deliberate-regression evidence;
- every validation command, return code, monotonic duration, log digest, and
  unavailable row; and
- result commit and SHA-256.

The reviewer returns:

- exact port identity and result digest;
- one terminal `ACCEPT` or `REQUEST_CHANGES`;
- explicit disposition of every accepted source-gate invariant;
- severity-ordered findings with reproduced or object-level evidence; and
- report commit, report SHA-256, and clean-worktree proof.
