# AI Operating Protocol

This protocol is the single human-readable operating guide for AI task packets, review packets, and parallel lanes.

Use it with:

- `context-packs.json` for bounded reading;
- `schemas/` for the authoritative machine packet contracts;
- `role-routing-matrix.json` for machine-readable role and reasoning-effort selection;
- `provider-routing-matrix.json` for machine-readable Fugu/Claude/Codex selection and concurrency;
- `provider-routing-policy.md` for Low-Codex fallback, timebox, swarm, and integration semantics;
- `model-routing-policy.md` for Medium/High/Ultra semantics and escalation;
- `task-packets/` for ready-to-delegate concrete work.

## Core Rule

Reduce ambiguity before increasing parallelism.

No AI lane should start until it has:

- objective;
- context pack;
- allowed and disallowed files;
- non-goals;
- invariants;
- validation commands;
- handoff expectations.

A task packet inherits the applicable `AGENTS.md` instructions and canonical
repository invariants at its bound base revision even when it fails to repeat
them. Packet omission cannot turn a module-boundary, identity, state,
compatibility, authority, or user-data violation into accepted implementation
variance. If the missing rule is needed to authorize the repair, adjudicate
`contract_omission`, freeze an additive amendment, and block mutation rather
than silently rewriting the accepted packet.

## Role Routing

Keep stable role semantics separate from provider and model runtime selection. The default external topology uses Fugu for Medium evidence, Claude for High implementation/review, and guarded Codex `gpt-5.6-sol` for Ultra integration/final decisions. CodeFire product semantics must not depend on any provider.

| Role | Default tier | Use for | Required output |
|---|---|---|---|
| Manager | Ultra | batch planning, lane assignment, integration, conflicting evidence, completion audit | task packets, integration plan, completion audit |
| Architect | Ultra | module boundaries, object/branch invariants, transaction/security design | alternatives, design note, falsification plan |
| Implementer | High | focused code or docs changes with bounded scope | patch, focused tests, handoff |
| Reviewer | Ultra (current machine route) | adversarial correctness, invariant, contract, and evidence review | evidence-bound observations and approve/request/block proposal |
| Researcher | High | bounded exploration with evidence and competing hypotheses | evidence, alternatives, decision candidates |
| Docs synchronizer | Medium | source-backed tables, links, current-state synchronization | docs diff, source evidence |
| Validation runner | Medium | deterministic commands and exact log summaries | validation table |
| Dogfood runner | High | algorithm workflow, reproducible friction, CFB discovery | dogfood evidence |
| Classifier | Medium | dirty tree grouping and declared ownership classification | classification table |

Provider routing is applied after role/risk selection:

| Selected tier | Default provider | Execution shape |
|---|---|---|
| Medium | Fugu `fugu-ultra-v1.1` | bounded packet, strong oracle, 2h default |
| High | Claude `claude-opus-5` | multi-hour worktree, checkpoint commits, independent review |
| Ultra | Codex `gpt-5.6-sol` | explicit `--allow-codex`, one control-plane lane, integration/final evidence |

No unavailable worker automatically falls back to Codex. Fugu-to-Claude rerouting is explicit; Claude-to-Codex requires a recorded Ultra reason and a new guarded lane.

Escalate Implementer, Researcher, Dogfood, or Docs work when the task crosses an invariant, migration, security, deletion, or release-completion boundary. Apply the score and mandatory escalation rules from `model-routing-policy.md`.

`Ultra` includes both maximum available reasoning and proactive delegation where the surface supports it. Direct API `max` without an external manager is not equivalent. When a model or reasoning surface changes, update and benchmark the runtime adapter without silently changing the role taxonomy.

## Task Packet Shape

Machine execution should use `schemas/task-packet.v1.schema.json`. This Markdown shape is a human view:

~~~markdown
# AI Task Packet

Status:

Packet ID:
Schema version:
Base revision:
Risk score:
Reasoning tier:

## Objective

One sentence describing the exact result expected.

## Context

- Required context pack(s):
- Additional files:
- Related issue or observation:

## Scope

Allowed files:

- 

Disallowed files:

- 

## Non-Goals

- 

## Invariants

- object identity:
- sealed commit validation:
- branch head atomicity:
- remote mutation policy:
- JSON envelope stability:
- path/workspace safety:
- module boundaries:

## Validation

```bash

```

If a command cannot run, report the exact command, failure, environment reason, and review risk.

## Handoff

- files changed;
- design summary;
- validation result;
- risks;
- follow-up.
- task/packet digest and resulting patch digest when automation provides them.
~~~

## Review Packet Shape

Machine review records use `schemas/review-packet.v1.schema.json`. This is the human view:

Each validation evidence row must carry the task packet's `validation_id`, exact command, status, and summary. Approval additionally requires a log digest; post-validation requirements such as source-bound binary provenance require a repository-contained artifact path and its digest. Approval is invalid unless the task ID/digest/base revision match, canonical Git base/head resolve, the exact Git changed-file set stays inside the write set, the canonical patch digest matches, and every required validation has matching passed evidence. `block` and `request_changes` may record unavailable gates without fabricating approval-only provenance.

~~~markdown
# AI Review Packet

## Review Target

- Diff or branch:
- Claimed objective:
- Related task packet:
- Task packet digest:
- Base/head/patch digest:
- Reviewer independence:

## Required Context

- Context pack(s):
- Files changed:
- Files to inspect first:

## Review Questions

- Does the diff solve the stated objective?
- Did it touch files outside scope?
- Did it mix product, management, and dogfood changes?
- Are object identity, sealed commits, branch atomicity, JSON envelopes, and path safety preserved?
- Are parse, validate, plan, apply, and render responsibilities still separated?
- Are skipped validations justified?

## Reviewer Output

- decision: `approve`, `request_changes`, or `block`;
- strongest correctness concern;
- strongest maintainability concern;
- validation assessment;
- required follow-up.
~~~

## Parallel Lane Rules

Parallelism is safe only when lanes have non-overlapping write sets or a manager owns final integration.

For multi-hour batches, use the external `dev-swarm` scheduler. It binds the
base revision, task tickets, DAG, write sets, provider concurrency, expected
commits, and optional machine-readable review decision. Fugu 4, Claude 2,
and Codex 1 are initial provider windows; they do not add up to a global
session cap. External Fugu/Claude windows adapt one lane at a time from clean
completion or provider-backpressure evidence, while Codex remains one
cost-controlled manager lane. The manual portfolio separately runs
`check_swarm_admission.py` for host resources and conflict ownership. The
scheduler creates an integration branch but never merges main automatically.

| Lane | Writes allowed | Conflict risk |
|---|---|---|
| Manager | Plans, management docs, final conflict fixes. | High for global files. |
| Researcher | Exploration files assigned in the packet; usually no canonical docs. | Low when each researcher owns a distinct note. |
| Implementer | Only files allowed by the task packet. | Depends on packet scope. |
| Reviewer | Usually none; review notes only. | Low. |
| Docs synchronizer | Docs listed in the packet. | Medium when current-state docs change. |
| Dogfood runner | Dogfood project files or CFB candidates only. | Medium. |
| Validation runner | Validation log or handoff notes only. | Low. |

Serialize edits to global state files:

- `docs/development/history.md`;
- `docs/development/current-cycle-burn-down-plan.md`;
- `subprojects/algorithm-evolution-agent-lab/codefire.links.yaml`;
- `crates/codefire-cli/src/main.rs`;
- `crates/codefire-cli/src/tests.rs`.

If multiple lanes need one of these files, the manager writes the final version.

## Review Governance

Review is a decision gate over an immutable subject. It does not create a
requirement by preference, and it does not authorize an implementer to expand
the repair. Every observation follows this order:

```text
observation and evidence
-> multi-axis impact assessment
-> must-fix and unknown-impact decision
-> actionability disposition
-> required-correction causal class, only when required
-> frozen repair and rereview scope, only when required
```

The durable report shape is in `validation-report-template.md`. The detailed
P0-P2 typed-control implementation plan is the review-bottleneck packet under
`task-packets/`; it refines this policy but does not replace it or grant itself
implementation authority.

### Role separation and review types

| Role | Owns | Must not do |
|---|---|---|
| Manager | frozen scope, candidate identity, reviewer exclusions, impact/actionability adjudication, repair scope, integration and final lifecycle decision | turn an unsupported preference into a frozen requirement or silently discard an observation |
| Implementer | bounded implementation, self-review, focused tests, repair candidate and handoff | independently approve the candidate or choose a favorable disposition/causal class for its own work |
| Validation runner | exact commands, environment identity, immutable logs/results and unavailable evidence | infer semantic acceptance from a green command |
| Independent reviewer | report-only examination of the exact candidate, obligations, changed paths, hazards, evidence and new effects | mutate the candidate, implement tests/fixes, resolve conflicts, or reuse an excluded implementation session |
| Dogfood/evaluation owner | installed-artifact workflow evidence and KGI/KPI outcome evaluation | reinterpret source acceptance as final issue closure |

Use the review type that answers the current risk question:

| Review type | When required | Subject and exit |
|---|---|---|
| Contract preflight | Before mutation for new/cross-cutting invariants, public or persisted contracts, state machines, migrations, security/authority boundaries, or costly recovery | Exact frozen contract; zero unresolved authorization-changing choice, contradiction, reproducible fail-open/contract-escape behavior, or unowned falsifier |
| Exact candidate review | After Candidate Freeze for every integration candidate | Exact commit/tree/patch/path set; every observation evidence-bound and adjudicated |
| Repair rereview | After a required correction | Exact repair delta, inherited obligations/effects, named hazards and regression evidence; expand only when the repair creates a new effect or contract |
| Integration review | When independently accepted packages compose across shared owners/contracts | Exact composed revision and overlap; focused post-integration evidence |
| Installed dogfood/evaluation | When the batch requires runtime, packaging, UX, or development-flow proof | Source-bound installed artifact and frozen workflows/KGI/KPI; feeds final Close |

Reviewer count is risk-based, never a universal number. One fresh exact
reviewer is normally sufficient for a bounded change with one established
contract and deterministic evidence. Use distinct contract, feasibility,
security/authority, data/migration, or adversarial perspectives when the change
crosses those risks or when evidence conflicts. Each additional reviewer gets a
different bounded question; repeated generic reviews and finding-count targets
are not coverage. Provider diversity is recorded but does not prove reviewer
independence. Preflight does not substitute for exact-candidate review.

The current `role-routing-matrix.json` keeps the reviewer role on Ultra even
when one bounded reviewer is sufficient; `review-packet.v1` permits High or
Ultra. This policy changes review coverage and count, not that machine route.
Any risk-based tier change requires a separate bounded routing/schema/eval
migration rather than a prose-only downgrade.

### Impact is not actionability

Severity records the magnitude of an observed effect as `critical`, `high`,
`moderate`, `low`, `none`, or `unknown`. It never directly says "fix". Before
actionability, record all of these axes with exact evidence or an explicit
`unknown`, `unavailable`, or `not_applicable` rationale:

- effects on frozen requirements, invariants, acceptance criteria, KGI/KPI and
  binding quality guardrails;
- affected users, workflows, paths/symbols/surfaces and accountable owners;
- `correctness`, `state`, `data`, `authority`, `api_compatibility`,
  `operations`, `performance`, `resource`, `maintainability`, and
  `development_flow` impact categories;
- reachability, likelihood, blast radius, severity/materiality,
  reversibility/recovery, and recurrence likelihood;
- evidence confidence, reproducibility and availability; and
- repair cost, alternatives, repair-regression hazards and recovery tradeoffs.

Use risk-proportional depth. Every observation gets a compact screen covering
every axis as affected, unaffected, or unknown with evidence. Expand the full
typed values/rationales only for a proposed required correction, accepted
variance, disputed/reopened locus, high or material impact, or
acceptance-critical unknown. A true duplicate may reference the canonical full
record; an evidenced invalid, wholly out-of-scope, or nonblocking observation
may terminate after the complete screen and manager disposition. Axis omission
and treating unknown as neutral are forbidden at either depth.

Do not combine the axes into a weighted score, severity sum, vote, field-count
percentage, or checklist cutoff. Repair difficulty may change containment,
ordering, mechanism, or rereview scope; it cannot cancel a must-fix trigger.

The evidence-bound must-fix triggers are:

| ID | Trigger |
|---|---|
| `MF-1` | An exact frozen requirement, invariant, acceptance criterion, or binding quality guardrail is violated. |
| `MF-2` | A reachable material correctness, state, data-integrity, or authority-boundary failure prohibited by accepted scope is observed or reproducible. |
| `MF-3` | A binding public API, persisted data/state, or cross-generation compatibility contract is broken. |
| `MF-4` | A frozen operations, performance, resource, availability, or recovery bound is breached. |
| `MF-5` | Evidence explicitly required for acceptance is absent, contradicted, or invalid. |
| `MF-6` | A repair introduced a reachable material regression or reopened an obligation/effect previously proven closed. |

One proved trigger requires correction. If acceptance-critical impact or
evidence is unknown/unavailable, leave actionability undecided and block on an
owned impact investigation; do not call the observation neutral or accept a
variance. A noncritical unknown may be routed only when evidence and manager
rationale prove that no current acceptance criterion can be affected, and its
investigation remains separately owned.

### Actionability and causal classification

After the impact gate is complete, assign exactly one disposition:

| Disposition | Meaning and current-batch action |
|---|---|
| `required_correction` | A must-fix trigger or equivalent frozen acceptance violation is proved. Block acceptance and freeze repair/rereview scope. |
| `accepted_implementation_variance` | Different implementation, but equivalent frozen behavior/invariants and required evidence are affirmatively proved. Bind a manager decision at the stable locus; authorize no repair. |
| `nonblocking_improvement` | Concrete value exists but no current frozen criterion makes it mandatory. Admit separately in a future batch if chosen; do not repair inside this one. |
| `out_of_scope` | The observation is outside the frozen candidate/review scope and proves no in-scope or cross-cutting violation. Retain and route it; do not use it to hide a candidate-created effect. |
| `duplicate` | The same effect, obligation, or decision locus already has a canonical observation/decision. Reference that ID; create no new unique effect/finding/issue/repair or recurrence-after-closure unit. Retain the observation in review-workload, churn, and actionability-distribution evidence. |
| `invalid_or_unsubstantiated` | The exact claim is contradicted, non-reproducible, evidence-free, or too underspecified to decide against the candidate. Retain the reason/evidence; materially new evidence may create a later observation. |

Apply the dispositions in this precedence so labels cannot hide a must-fix
effect: an acceptance-critical pending impact leaves disposition null; a true
restatement with no distinct effect/evidence delegates to the canonical row as
`duplicate`; a disproved/underspecified claim is
`invalid_or_unsubstantiated`; an observation wholly outside scope with no
in-scope or cross-cutting effect is `out_of_scope`; a proved must-fix trigger is
`required_correction`; affirmatively proved equivalent implementation is
`accepted_implementation_variance`; otherwise a concrete optional benefit is
`nonblocking_improvement`. A purported duplicate with materially distinct
evidence, effect, or decision need is not a duplicate. Duplicate disposition
never erases the canonical effect's correction/closure state.

The recurrence gate's statement that every **changed path** must have a reviewed
surface and has no `out_of_scope` approval escape remains absolute. The
`out_of_scope` disposition above applies to a review **observation** only. It
cannot remove a changed path, applicable invariant, cross-cutting effect, or
distinct finding from coverage.

Only `required_correction` receives exactly one causal class:

| Causal class | Required next step |
|---|---|
| `contract_omission` | Freeze an additive contract/amendment, hazard, oracle, authorized boundary and rereview scope before mutation; never rewrite accepted history. |
| `implementation_nonconformance` | Bind the already-frozen clause, smallest repair boundary, regression test and exact delta review; do not expand scope. |
| `repair_introduced_regression` | Create a successor obligation with before/after production evidence, hazard, falsifier and inherited-surface rereview. |

These are governance-origin classes, not an exhaustive technical root cause.
Choose exactly one in this precedence: if no pre-candidate binding rule covered
the required behavior, use `contract_omission`; otherwise, if the newly failing
effect was created by the preceding repair delta, use
`repair_introduced_regression` even though that delta also violates the frozen
rule; otherwise use `implementation_nonconformance` for the initial candidate
or a non-repair-origin violation of an existing clause.

All other dispositions have a null causal class and authorize no repair.
Readability, naming, file splitting, abstraction, layering, and style are
maintainability observations; they block only when tied to a frozen criterion,
a pre-candidate measurable violation, or a concrete reachable failure or
regression.

Transition rule: `review-packet.v1` still requires a finding
`required_action`, but under this policy that field is the reviewer's proposed
follow-up, not manager repair authority and not proof that correction is
required. Its legacy severity vocabulary also does not encode the impact axes.
Legacy `block` is a gate/decision label, not impact severity; a legacy `medium`
observation maps to human impact `moderate` only for the accompanying report and
does not rewrite the v1 JSON vocabulary.
Likewise, a self-asserted `reviewer_independent` value is not fresh-session
proof. Until a new schema/checker version enforces the complete adjudication and
provenance contract, retain the immutable reviewer proposal and a separate
manager impact/actionability/repair-authority artifact that references the
report commit/digest, plus the exclusion set and exact Git topology. The
reviewer's terminal verdict is a proposal, not integration authority. Do not
silently reinterpret or rewrite accepted v1 bytes.

### Stable decisions, repair and rereview

Every disputed implementation choice has a stable `decision_locus_id` and at
most one active manager decision for the subject/scope. An
`accepted_implementation_variance` has no wall-clock expiry and may reopen only
for `frozen_requirement_changed`, `materially_new_evidence`,
`new_counterexample`, or `prefrozen_metric_regression`, each with exact
evidence. A later reviewer, model, wording, or preference alone is retained as
preference churn and cannot reopen it. A valid trigger creates an explicit new
adjudication and supersession; it does not silently overwrite the earlier
decision.

`materially_new_evidence` means new to the active decision record, not
necessarily created later in wall time. Pre-existing evidence qualifies only
when the manager proves that the prior adjudication omitted or misbound that
exact evidence and that it can change a frozen outcome; rewording or a different
interpretation without that proof remains preference churn.

Before repair, the manager freezes the correction IDs, authorized and forbidden
paths/symbols, required positive evidence, negative controls/hazards, and exact
rereview surface. Rereview covers the delta, every inherited obligation/effect
touched by it, and named regression surfaces. It does not reopen unaffected
accepted loci. A newly discovered effect, unplanned public/state/persisted
surface, or contract omission returns to Freeze/Plan; it is not smuggled into a
repair. Any candidate byte change creates a new exact subject.

Do not rewrite a historical `required_correction` into a non-correction after a
repair. Preserve the original observation/disposition and record separate
repair and successor-candidate closure status. Acceptance requires zero
currently open/unclosed required correction for the exact candidate, not zero
historical required-correction observations in the lineage.

## Handoff Format

Every AI lane returns:

```text
Role:
Objective:
Files changed:
Validation:
Risks:
Next:
```

Manager-owned integration then records final validation and history updates once.

## Successor Review Recurrence Gate

Use recurrence control v2 for the current successor-review candidate. The v1
schema and validator remain historical compatibility evidence only: passing a
v1 record to the normal checker always returns a structured `BLOCK`. Never
rewrite an accepted packet or historical review document merely to make it v2.

The gate addresses three different failure modes without treating them as one
undifferentiated `REQUEST_CHANGES` loop:

1. a prior obligation was never actually closed;
2. the repair introduced a new effect or review surface;
3. the earlier review, oracle, scope, or communication missed the effect.

Before repair starts, the manager commits a Markdown repair packet and
`review-recurrence-repair-manifest.v1` together. The packet explains each prior
obligation's acceptance criterion, authorized boundary, production routes,
anticipated regressions, and forbidden shortcuts. The manifest freezes the
exact obligation, parity/closure-claim, and required-validation sets. The gate
proves equality to those sets, but their semantic completeness remains a
manager and independent-preflight judgement. The implementer may not replace
them with self-authored closure claims.

The v2 record binds repair packet, manifest, candidate, review packet, report,
timed evidence, and terminal event as exact regular Git blobs at
role-compatible commits. Git topology separates the pre-repair packet commit,
candidate, review-control commit, report-only commit, and evidence commit. A
post-candidate prompt cannot masquerade as the pre-repair contract, and this
generation's review commits cannot self-author prior closure evidence.

The independent reviewer inspects every prior obligation, then every exact
changed path and its candidate-created surfaces. There is no no-surface or
out-of-scope escape: each changed path has at least one reviewed surface.
`referred_to_manager` binds an earlier routing decision and remains blocking.
Evidence is classified with the v2 evidence-state/next-action vocabulary;
`CLOSED_PROVEN` with `NONE` is the only potentially nonblocking pair and still
needs the separately derived closure proof. A new finding cannot be closed in
the generation that first records it.

There is deliberately no minimum or target finding count. Required validations
come only from the frozen manifest and must bind committed
`codefire.timed_gate.v2` results. Run them from the clean candidate with both
the live log and result outside the source worktree, then copy those bytes
unchanged into the evidence commit. Acceptance requires the exact candidate
revision/tree, clean state before and after, no index flags, complete log
bytes/digest, exact argv, unchanged post-command `HEAD`/tree, terminal exit,
stopped reviewer, zero child processes, and committed terminal-event evidence.
The index check is the union of its before/after observations, so clearing a
pre-existing flag inside the command does not hide it. Mutate-and-restore
behavior inside the command remains outside this proof.

Authority is progressive. A legacy-bootstrap v2 generation derives
predecessor completeness as `unavailable`, always blocks, and exists only to
seed the next generation. A later record must resolve the complete retained v2
record/result chain to the manager-fixed Cargo or KGI/KPI root. The current
gate replays every retained generation oldest-first using current schema,
artifact, manifest, timed-evidence, and decision semantics. Retained results
are comparison evidence, never trusted predecessor state; replay does not
claim that older gate implementations were semantically identical. For a live
arm, replay also reloads each generation's exact anchor blob from its historical
deciding-gate commit and requires that commit to remain in the current validator
ancestry; a retained pin cannot substitute for that rebind.

Run the decision from a clean committed validator checkout. `--git-root` is
the analysed candidate repository; `--validator-root` is the separate
manager-controlled source of the gate modules and schemas. Any dirty validator
state, uncommitted gate byte, index flag, redirected schema path, or validator
identity mismatch blocks. A manager-fixed chain also requires an explicit
matching `--live-arm cargo|kgi_kpi` and its committed validator-owned anchor.
No anchor is currently committed, so neither live arm can yet emit `ALLOW`.

After the reviewer has stopped, all child processes have ended, and the
report-only commit exists, the manager writes the final JSON record and runs:

```bash
python3 docs/development/tools/check_review_recurrence.py \
  --record <final-record.json> \
  --git-root <analysed-candidate-repository> \
  --validator-root <clean-validator-repository> \
  --live-arm cargo \
  --result <outside-both-repositories>/recurrence-result.json
```

Use `--live-arm kgi_kpi` for that arm. Only exit `0` with
`integration_decision: ALLOW` authorizes integration. v1, bootstrap generation,
`request_changes`, `block`, incomplete terminalization, unreviewed surfaces,
scope referrals, missing or in-tree timed evidence, missing live anchor,
mismatched Git identity, and `--no-git-verify` remain `BLOCK`.

The machine proves topology, bytes, declared set equality, legal state/action
pairs, and machine-visible closure predicates. Honest execution, semantic
oracle quality, nonzero selection, complete surface/symbol enumeration,
manifest completeness, authorship, and reviewer independence remain human
judgements. Machine `ALLOW` is required evidence, not a substitute for the
manager's semantic integration decision. Current status and promotion metrics
are in `review-recurrence/README.md`.

## Prompt Construction

Do not maintain separate role-specific prompt documents. Build prompts from:

1. the selected context pack;
2. the task or review packet;
3. this protocol;
4. `model-routing-policy.md` for risk and escalation;
5. `provider-routing-policy.md` and `provider-routing-matrix.json` for provider/concurrency/fallback;
6. `role-routing-matrix.json` for Codex-specific machine-readable reasoning routing.

Keep common instructions and stable context at the beginning so prompt caching can reuse a stable prefix. Put task-specific packet data and volatile repository evidence at the end.

Resolve inherited context files and bind their current content with:

```bash
python3 docs/development/tools/resolve_context_pack.py pack-0-universal-orientation
```

Validate and dry-run a machine packet before granting write access:

```bash
python3 docs/development/tools/check_ai_contracts.py
python3 docs/development/tools/run_ai_role.py \
  --packet docs/development/ai/task-packets/json/branch-list-timeout.json
```

`run_ai_role.py` is a Codex-specific low-level adapter. It defaults to a non-executing manifest and read-only sandbox; every real run additionally requires `--allow-codex` under the Low-Codex policy. Execution and `workspace-write` are separate explicit choices. The Codex repository sandbox cannot enforce an individual packet's write-set, so write execution also requires explicit acknowledgement of that limitation and independent diff review. Routine Fugu/Claude work uses `dev-agent` or `dev-swarm` instead.

The runner reconstructs the packet's content-addressed context from its exact packs, additional files, and all currently existing explicit read/write inputs. A declared manifest must match that reconstruction exactly. It binds the digest into the run manifest and verifies live file bytes before execution; missing read inputs block execution and remain visible in dry-run output. Typed execution rejects an extra `--prompt`, a stale `base_revision`, and any existing worktree change intersecting a workspace-write packet's declared write set. There is no broad dirty-write override: first split/save the existing work or create a future packet contract that binds each inherited file by digest.

Validation requirements have two phases. Executable, compound, and derived capabilities are preflight requirements. Evidence capabilities such as a Rust binary bound to current source are post-validation requirements: they do not prevent investigation from starting, but their missing evidence prevents review approval.
