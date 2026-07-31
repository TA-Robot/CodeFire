# Successor Review Recurrence Gate v2

This directory holds manager-owned recurrence records, retained gate results,
and committed evidence. The current authorization candidate is
`codefire.review_recurrence_control.v2`; it is still experimental and does not
by itself prove that the recurrence bottleneck is solved.

`codefire.review_recurrence_control.v1` remains readable only as a compatibility
input and historical artifact. The normal checker always returns a structured
`BLOCK` for v1, even when that record passed its historical validator. Do not
rewrite accepted packets or historical review documents to v2, and do not use
the retained v1 validation function as an integration path.

## Current live authorization status

The manager-fixed `cargo` and `kgi_kpi` authority arms require an explicit
`--live-arm` and a matching manager-owned anchor at
`live-arm-anchor.v1.json`. The anchor must be a regular blob in the clean,
committed validator source and must pin the arm's repair packet, repair
manifest, and predecessor authority identities.

No live-arm anchor is committed yet. Therefore the current v2 candidate can be
exercised and retained, but it cannot emit `ALLOW` for either live arm until a
separate manager task commits and independently reviews that anchor. This is a
deliberate fail-closed state, not an instruction to synthesize an anchor from
the candidate being reviewed.

## Two repositories, two authorities

The checker distinguishes:

- `--git-root`: the repository whose candidate, review topology, evidence, and
  retained authority chain are analysed;
- `--validator-root`: a separate manager-controlled checkout containing the
  checker, its v2 modules, and its schemas.

The validator root must be at a clean committed `HEAD`. Every gate source and
schema used for the decision must match its committed regular blob; dirty
worktree state, index flags, untracked gate code, or an on-disk/committed digest
mismatch blocks. Compatibility schema flags may repeat the canonical paths in
that validator root but cannot redirect schema authority. A live arm also
requires `--validator-root` to differ from `--git-root`.

## Generation and authority rules

1. A v2 bootstrap generation may cite exactly one manager-fixed legacy report.
   Legacy Markdown cannot establish complete predecessor obligations, so the
   derived completeness is `unavailable` and that generation always blocks.
   Its retained v2 record/result can seed the next generation.
2. A later generation cites the previous retained v2 record and result. The
   authority resolver must reach the manager-fixed Cargo or KGI/KPI root and
   prove the complete committed topology.
3. Every retained v2 generation is replayed oldest-first with the current
   schema, artifact binding, manifest equality, timed-evidence checks, and
   decision derivation. A retained result records what an older gate said; it
   is never trusted as the source of predecessor state. For a manager-fixed
   live arm, each retained pin is rebound to the exact anchor blob at that
   generation's historical deciding-gate commit, which must remain in the
   current validator ancestry.
4. Replay uses current semantics and does not claim that an older gate was
   semantically identical. Structural or retained-result replay mismatches
   block the current decision. Ordinary historical blockers, including the
   mandatory bootstrap `BLOCK`, remain visible without automatically
   invalidating a correctly repaired successor.

## Evidence rules

- The pre-repair Markdown packet and
  `review-recurrence-repair-manifest.v1` are committed together before the
  presented repair. The machine proves set equality to the frozen manifest; a
  manager and independent preflight remain responsible for whether its sets
  are semantically complete.
- Artifact references resolve to exact Git blobs at role-compatible commits.
  Symlinks, executables, trees, gitlinks, dead references, and self-authored
  closure evidence are rejected.
- Every changed path has at least one reviewed surface. A manager scope referral
  is evidence for successor routing and remains blocking; it is not an
  out-of-scope approval escape.
- Required commands use `codefire.timed_gate.v2`. Both the live log and result
  must be outside the source worktree, then copied byte-identically into the
  evidence commit. In-tree log or result paths block acceptance even when the
  checkout appears clean afterward.
- The timed result must bind the exact candidate revision/tree, clean states
  before and after execution, an unchanged post-command `HEAD`/tree, an empty
  before/after index-flag union, complete log bytes and digest, a terminal exit
  code, and the exact frozen argv.
- `CLOSED_PROVEN` with `NONE` is the only potentially nonblocking state/action
  pair, and only when its separately derived closure proof succeeds. Routing
  never grants integration authority.
- There is no minimum finding, observation, surface, or obligation count.

## Human semantic boundary

The gate derives topology, byte identity, set equality, state/action legality,
and machine-visible closure predicates. It does not prove honest command
execution, semantic oracle quality, nonzero test selection, complete symbol or
surface enumeration, the semantic adequacy of a closure oracle, the
completeness of the frozen manifest, human authorship, or reviewer
independence. Those remain explicit manager and independent-review judgements.
An `ALLOW` is therefore necessary machine evidence for integration, not a
replacement for human semantic acceptance.

## Operating sequence

1. The manager freezes and commits the repair packet plus manifest before the
   repair, without editing previously accepted packets or historical reviews.
2. The implementer returns one committed candidate. The manager recomputes its
   exact patch, tree, and changed paths.
3. An independent report-only reviewer reviews every path/surface, prior
   obligation, new finding, derivation, required validation, and terminal event
   without a count target.
4. Run required commands from the clean candidate while writing live evidence
   outside that worktree:

   ```bash
   python3 docs/development/tools/run_timed_gate.py \
     --emit v2 \
     --cwd <clean-candidate-worktree> \
     --log <outside-source>/validation.log \
     --result <outside-source>/validation.json \
     --timeout <seconds> \
     -- <exact argv>
   ```

5. After the reviewer and all children stop, commit byte-identical evidence,
   report, record, and retained result in the required topology. From a clean,
   committed and separate validator checkout, run:

   ```bash
   python3 <validator-root>/docs/development/tools/check_review_recurrence.py \
     --record <final-v2-record.json> \
     --git-root <analysed-candidate-repository> \
     --validator-root <clean-validator-repository> \
     --live-arm cargo \
     --result <outside-both-repositories>/recurrence-result.json
   ```

Use `--live-arm kgi_kpi` for that arm. Exit `0` with
`integration_decision: ALLOW` is the only machine authorization; exit `3`, v1,
bootstrap generation, `--no-git-verify`, an absent/mismatched anchor, or any
finding/blocker remains `BLOCK`.

## Evaluation and promotion

For comparable Cargo and KGI/KPI generations, retain recurrence capture,
candidate-created effects, unplanned surfaces, repair/review/report service
time, unavailable evidence, repeated observations, and operator cost. Promote
v2 only after live anchored runs and an independent blind causal audit show
useful recurrence prevention without false closure or disproportionate
integration delay. Revise or remove it if a known recurrence passes or its
burden exceeds avoided rework.
