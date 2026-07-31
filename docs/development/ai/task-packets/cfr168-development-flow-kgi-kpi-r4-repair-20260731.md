# CFR-168 Development-Flow KGI/KPI — Round-4 Bounded Repair

Status: queued behind live resource/provider admission

Provider/model: Claude Opus 5 (`claude-opus-5`)

Role: original-session repair implementer

Resume only original implementer session
`34cdf037-2326-480e-8353-621713838feb` in its existing clean worktree at
exact candidate `753c8db103579d021795d3fe6a97c5d45d4a111a`.

## Objective

Close the decision-critical residuals from the exact round-3 review while
preserving all 13 independently reproduced round-2 closures. Keep the repair
inside existing semantic owners and tests; do not answer a review-cycle
problem by adding another general framework, public API, state machine,
classifier, or fixed checklist.

## Immutable Inputs

- reviewed candidate:
  `753c8db103579d021795d3fe6a97c5d45d4a111a`
- candidate tree:
  `213e16a033aeb2e8eb3599be4ffe725b1d7ad344`
- predecessor:
  `0944bf3deba9e753132b3f71716b5c009a8fc30f`
- immutable round-3 review commit:
  `12f9d0b5c30ce2695ec6c96e5876703efd7b0d90`
- immutable review path:
  `docs/development/ai/swarm-results/development-flow-kgi-kpi-r3-rereview-claude-20260731/review.md`
- immutable review bytes:
  `63,420`
- immutable review SHA-256:
  `23b59e01825e3f66d2d38b754348e3d51457f3c465def578ac99ce52729a204f`

Recompute these identities before editing. Do not modify any independent
review report.

## Scope

### What the evidence says

The candidate did not simply ignore the prior review. The round-3 reviewer
rebuilt every one of the 13 executable round-2 residuals against the
predecessor tree, then showed that all reject on the candidate with stable
codes. The frozen 55 semantic / 15 structural code tables are byte-identical,
so those closures are not goalpost movement.

The new decision has a mixed causal shape:

- one inherited terminal-decision path remains partially closed
  (`BOUND_FINDING_NOT_CLOSED`);
- several other inherited SHOULD rows were narrowed but not closed;
- the repair created an honest-positive rejection and a diagnostic-budget
  suppression (`REPAIR_INTRODUCED_REGRESSION`);
- some published claims exceed enforcement; and
- several candidate tests share production helpers or inert repository
  context (`VACUOUS_OR_LATE_TEST_ORACLE` risk).

Do not summarize this as “review omission”, “bad communication”, or “bad
tests” without the row-level evidence. Do not infer a project-wide dominant
cause from this episode.

### Structural repair boundary

There is deliberately no arbitrary line-count or finding-count target.
Before editing, list every symbol and data shape the proposed repair would
add. If closing A1–A8 requires a new public API, new persisted schema concept,
new state machine, new general classifier, or a second decision engine, stop
and return a design blocker instead of implementing it. Extending existing
private checks and fixtures is allowed.

Every changed or new private symbol is fresh review surface. The result must
state which A-row requires it and its independent negative control.

### A1 — terminal guardrails must not be discharged by a name

Reproduce the exact round-3 counterexample in which:

- `MET-AI-075` and `MET-FLOW-037` are declared `measured 0`;
- one unrelated in-window validation event cites
  `measurement:<metric>`;
- `KGI-DEV-004` is set to `achieved`; and
- the checker returns zero findings.

Make a zero-target guardrail depend on evidence about that guardrail, not on
an author-written back-reference alone. Require the missing runtime-version
fact before `MET-AI-075` can be measured. Preserve an honest positive case
whose evidence independently identifies the metric/guardrail subject.

The acceptance property is behavioral: adding arbitrary event names,
`measurement:` strings, or unrelated repository citations must not make the
KGI terminal. Do not satisfy it with a frozen list of known event IDs.

### A2 — required-evidence completeness needs a non-author-chosen denominator

Reproduce `MET-FLOW-027 == 12/12 == 1.0` after deleting the non-verifiable
citations. Either derive a required evidence set from the record's actual
claims/deliverables or narrow the metric and KGI target language so it no
longer claims complete required evidence. The author must not improve the
metric by omitting a required citation.

Test both directions:

- removing required evidence cannot improve the metric; and
- adding an optional, unverifiable reference cannot make an otherwise honest
  record fail a stronger claim it never made.

### A3 — artifact subject must resist padding without collapsing deliverables

Close all four observed shapes with one property:

1. the round-2 lane-padding case remains rejected;
2. a commit-bearing twin padded with another declared `commit:` cannot mint a
   second subject;
3. a commit-free artifact padded with `event:` cannot mint a second subject;
4. two genuinely distinct deliverables in one commit remain valid.

Derive identity from the deliverable, not from its citation list. Do not use
the production digest helper to build the expected digest in the central
negative test.

### A4 — lifecycle stages require evidence of the stage

Reproduce a `final` record whose five closing events cite only the policy
document and otherwise produce no lifecycle finding. Make each closing stage
bind evidence of that stage, not merely any repository-verifiable reference.
Keep the ordering and prior-stage constraints already closed.

Use an honest complete lifecycle positive case and a policy-citation
negative case. Avoid an allowlist of the exact five historical event IDs.

### A5 — validation instant applies consistently

Apply the `captured_at` lower bound to active, final, and superseded records
on both record and directory routes. Echo the effective instant in both human
and JSON output. Preserve the existing valid-instant behavior.

### A6 — semantic diagnostics survive structural floods

Reserve diagnostic capacity so a structural flood cannot suppress all
semantic findings. The total remains bounded. Demonstrate a payload with more
than the structural allowance that still emits:

- the terminal `finding-limit-reached` marker; and
- at least one independently constructed decision-critical semantic finding.

Remove `cohort-cost-total-not-bound` or make it reachable with a negative
control. Do not add a second independent limit implementation.

### A7 — published claims must match enforcement

Correct only claims tied to A1–A6 and the exact precision mismatches in the
round-3 report:

- unrelated audit events and zero-target guardrails;
- lifecycle policy citations;
- active-only versus all-status instant behavior and JSON echo;
- 512 versus 513 emitted findings;
- working-tree/sealing-commit blob comparison;
- full-clone requirement versus shallow-clone behavior; and
- the measured project-document count.

Do not claim clone/rename/merge guarantees that this bounded repair does not
implement.

### A8 — closure evidence must be independent of the mechanism

For every A1–A7 negative control:

- construct expected values independently of the production helper being
  tested;
- run against `753c8db` first and record the failure;
- run against the repair and record the stable rejection;
- show one honest positive case;
- make the asserted axis move; and
- state the exact boundary at which the assertion fires.

Give the cohort positive case a real repository root rather than `root=None`.
Deleting the protected production check must change a test result.

## Explicitly Routed, Not Hidden

These observations remain recorded but are not authorized to enlarge this
repair:

- `R1c-c`: authored `integration` events are not yet trusted integration.
  Automatic event provenance belongs to the separately owned event/telemetry
  foundation; this repair must not invent a second trust system.
- `N3` and pre-existing `P1`: identical introductions across merge topology,
  rename/relocation, and shallow-clone seal behavior require one explicit Git
  history-support policy. Do not patch one topology locally.

The new result must preserve these as routed observations with owner and
release condition. A fresh reviewer may verify the routing but must not treat
their absence from this write set as an unreported closure claim.

## Non-Goals

- no project-wide review-template, finding-quota, or checklist policy;
- no KGI/KPI schema version replacement;
- no new telemetry/event trust architecture;
- no Git history traversal redesign for merge/rename/shallow clones;
- no product, Cargo/Rust, install, dogfood, or canonical integration work;
- no change to the frozen 55/15 code tables; and
- no correction of an immutable review report.

## Allowed Write Set

Only these paths may change:

```text
docs/development/ai/development-flow-kgi-kpi.md
docs/development/ai/flow-metrics/README.md
docs/development/ai/flow-metrics/cfr168-recovery-20260730.v1.json
docs/development/ai/schemas/README.md
docs/development/ai/schemas/development-flow-record.v1.schema.json
docs/development/history.md
docs/development/project-management.md
docs/development/tools/ai_contracts.py
docs/development/tools/check_ai_contracts.py
docs/development/tools/test_ai_tools.py
docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r3-20260731.md
docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r4-20260731.md
```

If the minimal repair needs another path, stop and report the exact reason
before writing it.

## Required Result Structure

Write
`docs/development/ai/swarm-batches/development-flow-kgi-kpi-zero-residual-repair-r4-20260731.md`
as a detailed evidence matrix. One row per A1–A8 must contain:

| Field | Required content |
|---|---|
| identity | A-row, originating review rows, exact base, affected paths |
| property | invariant stated without implementation names |
| pre-edit reproduction | independent fixture/probe, observed codes/value |
| causal attribution | inherited partial, repair-introduced, claim gap, oracle gap, routed, or unavailable with evidence |
| design boundary | why the change stays inside existing owners |
| repair | semantic change; every new symbol mapped back to this row |
| positive control | independently built honest case |
| negative/reversal | exact old candidate or mutation on which it fails |
| remaining state | closed, unavailable, or routed with owner/release condition |

Also include:

- preservation replay for all 13 previously closed residuals;
- complete changed/new-symbol inventory and per-symbol review;
- every claim considered but rejected;
- changed paths with per-path purpose;
- commands, exit codes, selected-test counts, and durations;
- final commit/tree and clean status;
- result bytes and SHA-256; and
- explicit non-acceptance pending a fresh independent report.

## Validation

Run:

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

Also replay the frozen 55 semantic / 15 structural codes and all independently
rebuilt A1–A8 controls. No Cargo/Rust command.

## Handoff

Commit one additive successor and leave the worktree clean. Return exact
commit/tree, changed paths, new-symbol inventory, gate results, result
path/bytes/SHA-256, and the unchanged routed observations. Do not claim
acceptance or authorize a process decision.
