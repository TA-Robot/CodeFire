# CFR-168 Source-keyed Cargo Gate — Third Closure Repair

Status: queued behind live resource admission

Provider/model: Claude Opus 5 (`claude-opus-5`)

Role: original-session repair implementer

Resume only original implementer session
`f7692f47-4af1-4f84-a186-d57ddd4b083e` in its existing clean worktree at
exact candidate `4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912`.

## Objective

Close the finite R1–R5 residual set from the exact second-closure review
without reopening the six proven closures, following a finding quota, or
absorbing an adjacent pre-existing issue.

## Immutable inputs

- reviewed candidate:
  `4e5c0e7f058cd406cb7a183ecda8b3a07f3ee912`
- candidate tree:
  `6417400cb7b5a3507f85f38d66c0573992ad4007`
- predecessor:
  `aaa39b61b37df273fe4d57e274ed8e1321e788ad`
- checkpoint:
  `bfbbc2b2bdc22f88679e5e1dc794665d863304a1`
- immutable second-closure review commit:
  `130878d321f6be166f554ac2c68a59e2b36afa88`
- immutable review path:
  `docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-second-closure-rereview-claude-20260731/review.md`
- immutable review SHA-256:
  `aad067b2b8feb80830c41db1608dd31f0e7e7d7ca1f37ab01d040336f788b4fd`

Recompute all identities before editing. Do not edit, replace, or amend the
review report.

## Why this is a bounded repair rather than a larger checklist

The review closed six of seven inherited findings and independently proved
their oracles non-vacuous. It rejected the candidate on one inherited
finding that the repair re-created, plus four candidate-created NITs. One
adjacent NIT is explicitly pre-existing and is not assigned to this repair.

This run has no requested finding count. Do not generate extra bullets to
meet a quota. Inspect the exact repair delta and the named properties below,
record any genuinely observed decision-relevant issue, and stop when the
finite owner table is either closed or explicitly unavailable.

This case also contains a reviewer correction that must be preserved. The
prior review at `b589864` counted 12 checkpoint failures from a non-Git
archive. The candidate correctly demonstrated 10 real failures in a Git
worktree; the other two were environment errors from `git ls-files`. Do not
blindly restore 12.

## Causal assessment to test, not silently assume

The blocker currently has mixed evidence:

- the prior review's checkpoint-count component contained a reproduction
  environment artifact, which the implementer correctly challenged;
- the prior review explicitly gave candidate count 159, and the repair
  itself later measured 168 after adding tests;
- nevertheless `history.md` was changed from the old wrong value 157 to a
  third wrong value 161, while both the commit message and repair result
  claimed the count was corrected.

The working classification is therefore
`BOUND_FINDING_NOT_CLOSED + REPAIR_INTRODUCED_REGRESSION`, with a missing
claim-to-oracle verification step. It is not currently classified as an
initial-review omission or primarily as ambiguous communication. Confirm or
refute that classification in the result with exact bytes and commands. Do
not generalize one episode into a project-wide root cause.

## Scope

## Finite owner table

### R1 — blocking inherited finding re-created by the repair

`docs/development/history.md` must describe exact Git objects rather than an
unnamed “candidate”:

- `bfbbc2b`: 148 tests, 10 genuine failures in a real Git worktree;
- non-Git archive observation: two additional provenance errors, not product
  failures;
- `aaa39b6`: 159 tests, all passing; and
- `4e5c0e7`: 168 tests, all passing.

Use only the identities necessary for the row's claim. The smallest valid
repair is to bind the existing closure row to `aaa39b6` and replace 161 with
159. If the row also discusses `4e5c0e7`, state 168 separately and name that
commit. Update the prior repair result so it no longer claims a fact the
committed history bytes do not contain.

Before editing, mechanically reproduce all four observations. After editing,
add a property-based consistency control that would fail on 157 and 161 and
does not become self-referential when this repair adds tests. Do not publish
the new repair HEAD's suite count before the final commit exists.

### R2 — marker hard-link accounting claim

Reproduce the review's wholly-inside-namespace hard link to the namespace
marker. Make the payload and its exactness wording agree for that case,
without loosening the 32 GiB budget or changing the already accepted
conservative per-entry convention. Add a reversal that fails on `4e5c0e7`
for the property claimed, not merely on a symbol name.

### R3 — descriptor-survivor boundary wording

Make step 9's survivor statement conditional on the descriptors declared in
`--pass-fds`. Do not weaken the implemented complement closure or claim a
general filesystem sandbox.

### R4 — unshare documentation oracle direction

Replace or strengthen the substring-only documentation oracle so it detects
the actual failure direction: launched argv, recorded identity, source key,
artifact key, result, and the documented sequence must remain equal and in
order. Demonstrate that the control fails against the predecessor behavior
or an isolated reversal of the property.

### R5 — forward-guard labelling

Keep the useful no-consumer control, but label it as a forward guard rather
than evidence that a predecessor regression was closed. Correct the prior
repair result's table and any matching claim.

### Routed, not assigned — pre-existing `mount_points` parser NIT

The review found that `mount_points` skips malformed lines and searches for
`-` without anchoring. The bytes are identical at `bfbbc2b`, `aaa39b6`, and
`4e5c0e7`. Record it as pre-existing and do not change
`readonly_source_exec.py` in this repair. It needs a separately admitted
owner if it is promoted.

## Non-Goals

- do not change the pre-existing `mount_points` parser;
- do not redesign the source-keyed Cargo gate or its accounting convention;
- do not run or integrate any Rust candidate;
- do not change project-wide review policy from this single episode; and
- do not create findings merely to reach a numeric target.

## Allowed write set

Only these paths may change:

```text
docs/development/ai/source-keyed-cargo-gates.md
docs/development/history.md
docs/development/tools/run_source_keyed_cargo_gate.py
docs/development/tools/test_ai_tools.py
docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-second-closure-repair-20260731/result.md
docs/development/ai/swarm-results/aia-source-keyed-cargo-gate-third-closure-repair-20260731/result.md
```

No Cargo/Rust command, product CLI suite, Codex, Fugu, subagent, scheduler,
canonical integration, rebase, amend, squash, or destructive Git operation.
Keep all six independently verified closures, all ten fail-closed classes,
the global execution lock, and cleanup/resource ordering unchanged.

## Required result structure

Write the new result as a detailed evidence table, not a shallow list. One
row per R1–R5 must contain:

| Field | Required content |
|---|---|
| identity | finding ID, source report, exact base and affected paths |
| prior property | the exact requirement before repair |
| pre-edit observation | command/probe, observed value, and whether it reproduces |
| causal attribution | inherited-not-closed, repair-introduced, reviewer artifact, oracle gap, communication ambiguity, or other with evidence |
| repair | exact semantic change and why it is the smallest sufficient boundary |
| oracle | asserted property, independent source of expected value, and scope |
| negative control | exact reversal/base on which the oracle fails |
| remaining boundary | closed, unavailable with reason, or separately routed |

Also record:

- changed paths and per-path purpose;
- claims considered but rejected;
- all commands, exit codes, selected-test counts, and durations;
- final commit, tree, clean status, report bytes, and SHA-256; and
- an explicit statement that this candidate is not accepted until a fresh
  independent report returns `ACCEPT`.

## Validation

Run, without Cargo/Rust:

```bash
python3 -m unittest test_ai_tools
PYTHONDONTWRITEBYTECODE=1 python3 -m py_compile docs/development/tools/*.py
python3 docs/development/tools/check_ai_development.py
python3 docs/development/tools/check_ai_contracts.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

## Handoff

Commit one additive successor, leave the worktree clean, and return the exact
commit/tree, changed paths, gate results, result path/bytes/SHA-256, and
remaining routed observations.
