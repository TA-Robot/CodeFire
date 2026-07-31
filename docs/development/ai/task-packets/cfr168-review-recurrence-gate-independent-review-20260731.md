# CFR-168 AIA-047 Successor Review Recurrence Gate — Independent Review

Status: Ready

## Objective

Adversarially review the count-free successor-review recurrence gate as an
exact candidate. Determine whether it materially addresses the observed
review-loop causes (initial review miss, bound finding not closed,
repair-introduced regression, weak oracle, and ambiguous repair
communication), or merely adds report volume and another self-attested
approval path.

This is a report-only independent review. Do not edit the candidate, schema,
tool, tests, or canonical management documents.

## Immutable Subject

- Base revision:
  `a22fad0698b1623d8898c47699e7ba4e61294b5f`
- Candidate revision:
  `1cae0225e488ff918389700a7de0f94e9fd067b3`
- Candidate tree:
  `9c0a46a2c27ed3d54f3e1aa475df8ab655caad3a`
- Canonical binary patch SHA-256:
  `9b52caec0a67e3c74b431a68486cb01abb4a38e04df71e7934a06e468a6a06a4`
- Candidate commits:
  `5fa48c9`, `bf284b7`, `1cae022`
- Exact changed-path count: 11
- Required report path:
  `docs/development/ai/swarm-results/cfr168-review-recurrence-gate-independent-review-claude-20260731/review.md`

Recompute every identity before analysis. The report commit must be the sole
tree change over the review-control revision and the final worktree must be
clean.

## Scope

Read and review all 11 changed paths:

- `docs/development/ai/README.md`
- `docs/development/ai/acceleration-master-plan.md`
- `docs/development/ai/operating-protocol.md`
- `docs/development/ai/review-recurrence/README.md`
- `docs/development/ai/schemas/README.md`
- `docs/development/ai/schemas/review-recurrence-control.v1.schema.json`
- `docs/development/ai/validation-report-template.md`
- `docs/development/tools/check_ai_acceleration_docs.py`
- `docs/development/tools/check_ai_contracts.py`
- `docs/development/tools/check_review_recurrence.py`
- `docs/development/tools/test_ai_tools.py`

Inspect shared `ai_contracts.py` behavior where the new validator relies on
it. You may create temporary repositories and records outside the committed
tree for executable counterexamples.

## Required Review Passes

### 1. Root-cause fit and operator burden

- Does the mechanism distinguish observation rows, underlying effect
  episodes, obligations, and review generations without rewarding duplicate
  wording?
- Is the absence of a finding quota real, including indirect count proxies?
- Does the stopping rule require coverage saturation, or can an author set
  booleans and omit the difficult surface?
- Are repair communication and anticipated failure modes genuinely frozen
  before implementation?
- Is the nearly 3,000-line schema/tool/test addition proportionate, usable,
  and less costly than the rework it targets? Identify a materially smaller
  design if equivalent guarantees exist.

### 2. Fail-closed integration semantics

Attempt executable counterexamples for at least:

- `--no-git-verify` or a non-`accept` decision yielding `ALLOW`;
- a missing, extra, unsafe, or mismatched candidate changed path;
- an omitted changed-path coverage row or surface;
- a blocking, open, unavailable, or unauthorized routed finding accepted;
- an unreviewed or blocked repair path/surface accepted;
- a passed but live/truncated/nonzero/logless validation accepted;
- a live reviewer, active child, uncommitted report, or missing event log
  accepted;
- a report/control commit with extra product changes accepted;
- a mismatched repair/review packet, report, patch, tree, numstat, or digest
  accepted;
- a post-candidate repair contract masquerading as a pre-repair contract.

Separate a valid `REQUEST_CHANGES` evidence record from integration
authorization: it should remain record-valid where internally consistent but
must return `BLOCK`.

### 3. Completeness and anti-self-attestation

- Can the manager/reviewer omit a changed symbol, production route,
  candidate-created finding, causal class, or negative control and still
  manufacture `ALLOW`?
- Do Git-derived path/numstat identities meaningfully constrain the
  human-authored surface inventory?
- Does `dispatch_event_attested` prove the repair packet preceded work, and
  are its trust limits stated precisely?
- Are `not_applicable`, `unavailable`, nonblocking routing, scope expansion,
  and pre-existing-adjacent classifications bounded tightly enough?
- Can report/control ancestry or path-prefix handling be confused by merge
  commits, symlinks, binary paths, unusual Git names, or intermediate
  mutation commits?

### 4. Contract/tool/test/document agreement

- Schema and semantic validator must agree on every disposition and required
  field.
- Tests must fail for the dangerous mutations they claim to cover and must
  not pass only through shared implementation helpers.
- Documentation must not claim automation, chronology, completeness, or
  measured speedup that the code does not establish.
- The Cargo and KGI successor pilots must be operationally possible without
  retroactively fabricating evidence.

## Non-Goals

- Do not review or modify CodeFire product behavior.
- Do not start the Cargo or KGI successor reviews.
- Do not promote AIA-047 to permanent policy.
- Do not invent a target number of findings.
- Do not rerun Cargo/Rust/Docker or unrelated broad product suites.

## Validation

Run focused counterexamples before broad checks. At minimum:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  test_ai_tools.ReviewRecurrenceTests
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest test_ai_tools
python3 docs/development/tools/check_ai_contracts.py
python3 docs/development/tools/check_ai_acceleration_docs.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_development.py
git diff --check
```

Run the unittest commands from `docs/development/tools`. Record complete exit
status and distinguish a real terminal result from background progress.

## Handoff

The report must include:

- recomputed immutable identity;
- strongest root-cause-fit and strongest bypass/false-closure concern;
- prior guarantees that are actually established;
- every finding with severity, exact reproducer/evidence, and smallest repair
  boundary;
- explicit unavailable evidence;
- operator-burden assessment and smallest credible alternative;
- one final line only:
  `OVERALL_VERDICT: ACCEPT` or
  `OVERALL_VERDICT: REQUEST_CHANGES`.

Commit only the required report path. Acceptance authorizes no product or
process promotion; the manager owns the pilot decision.
