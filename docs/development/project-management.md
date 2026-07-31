# CodeFire Project Management Model

This document defines how CodeFire development should be managed over a long horizon. It is intentionally about project control, not product behavior.

## Goals

- Keep CodeFire product work, management documentation, and dogfood-project work separated.
- Make the current project state easy to identify without reading hundreds of historical cycle entries.
- Keep issue status, validation evidence, and implementation plans auditable.
- Prevent old "current" snapshots from becoming false sources of truth.
- Support repeated dogfooding cycles without turning the repository into an unstructured work log.

## Repository Boundaries

| Boundary | Includes | Does not include |
|---|---|---|
| CodeFire product | `crates/`, `codefire`, `tests/`, `install.sh`, product docs, schemas, ADRs | Algorithm research code or agent marketplace features. |
| Development management | `docs/development/`, root `README.md`, `AGENTS.md` | Product runtime behavior unless it is documenting decisions or workflow. |
| Dogfood target | `subprojects/algorithm-evolution-agent-lab/` | CodeFire CLI/store/core implementation. |
| Devcontainer base | `/workspace` outside `project/` | CodeFire product changes unless explicitly scoped to the base repository. |

Changes that cross a boundary must say why in `history.md`.

## Document Architecture

Use a small number of live documents and many append-only or historical documents.

| Layer | Document | Ownership |
|---|---|---|
| Stable entry | `README.md` | Product and repository overview. Avoid volatile counts. |
| Management hub | `docs/development/README.md` | Current state entry and source-of-truth map. |
| Governance | `docs/development/project-management.md` | This model. Update only when the model changes. |
| AI acceleration | `docs/development/ai/README.md` | External AI task packets, context packs, role routing, review packets, strategy notes, checks, and parallel-agent playbooks. |
| Development-flow KGI/KPI | `docs/development/ai/development-flow-kgi-kpi.md`, `docs/development/ai/flow-metrics/` | Trusted-integration outcomes, diagnostic metrics, incidents, and dated management decisions. |
| Strategy and roadmap | `docs/development/strategy/README.md` | Accepted principles, product direction, decision status, modernization gates, and release/format identity. |
| Exploration | `docs/development/exploration/README.md` | Non-authoritative hypotheses, alternatives, falsifiers, and experiments. |
| History | `docs/development/history.md` | Append-only work log. |
| Active batch | `docs/development/current-cycle-burn-down-plan.md` | Current burn-down batch only. Move old detail into history or mark it historical. |
| Issue ledgers | `bug-backlog.md`, `bug-issues/`, `code-review-issues-2026-06-06.md`, `code-review-issues/` | Issue facts, status, evidence, and closure. |
| Deletion audit | `docs/development/archive/deletion-manifest.md` | Destination and recovery record for superseded tracked prose. |
| Source architecture | `source-architecture.md`, `command-source-trace.md`, `source-data-model-catalog.md`, `change-playbook.md` | Module ownership, command routing, data contracts, and change recipes. |

This document alone owns the lifecycle order, issue admission, source
acceptance, and final Close meanings. The one-way AI governance map in
`docs/development/ai/README.md` assigns review/adjudication, metrics, report
shape, and machine-shape ownership beneath this lifecycle. Templates, schemas,
task packets, and historical cycle records instantiate or evidence policy; they
do not redefine it.

## State Model

Project state is the intersection of five facts:

1. Git state: branch, ahead/behind, dirty files, untracked files.
2. CodeFire product state: implementation status, issue ledgers, validation gates.
3. Dogfood CodeFire state: target repo branch state, open fires, sealed commit.
4. Validation state: which checks passed, failed, or could not run.
5. Documentation state: whether current-state documents agree.

Do not infer current project state from old cycle headings, old remaining counts, or historical issue IDs. Recompute from the ledgers and record the result in the management hub.

## Formal Issue Lifecycle

Use CFB for dogfood/product issues and CFR for source-review issues.

1. After manager admission, create a detail file from the relevant template.
2. Add or update the index row.
3. Set `Status` to `open`, `investigating`, `fixed`, `fixed in v0.7`, or `wontfix`.
4. Link the issue to a root cause or version plan when the fix is not trivial.
5. Add regression evidence before marking fixed.
6. Update `history.md` with the closure and validation evidence.

The detail file status is authoritative for closure. Index files are navigational summaries and must be kept consistent with detail files.

For new work, `fixed`, `fixed in v0.7`, and `wontfix` are final `Close`
dispositions, not pre-Install source-acceptance markers. Before final Close,
retain the existing nonterminal issue status and record `source accepted` in the
issue's lifecycle-evidence prose/current cycle record; do not add an unchecked
status value just for this transition. Existing historical status values and
accepted closure records are not retroactively reinterpreted.

A review observation is not automatically an open issue or repair order. First
retain its evidence and complete the impact/actionability adjudication in
`docs/development/ai/operating-protocol.md`:

- `required_correction` maps to an existing admitted issue or a manager-admitted
  additive issue/amendment before current repair;
- `nonblocking_improvement` becomes an issue only if admitted during a later
  Discover/Freeze decision;
- `accepted_implementation_variance`, `duplicate`, and
  `invalid_or_unsubstantiated` remain decision records rather than artificial
  issues; and
- `out_of_scope` is retained/routed and cannot authorize current repair or hide
  an in-scope effect.

`wontfix` is a final issue disposition with its own rationale; it is not a
synonym for accepted implementation variance.

## Cycle Lifecycle

Use this lifecycle for long-running CodeFire improvement work:

For day-to-day orientation, group the formal events into seven macro phases:

1. **Discover / Freeze**: observe, admit, and bound the batch.
2. **Plan / Preflight**: design the root fix and challenge high-risk contracts.
3. **Implement / Validate / Candidate Freeze**: produce one exact review subject.
4. **Review / Adjudicate / Repair**: decide impact and correction necessity, then repair only what is required.
5. **Integrate**: establish source acceptance on the canonical revision.
6. **Install / Dogfood / Evaluate**: prove the applicable installed and real-workflow outcomes.
7. **Close / Retrospect**: synchronize terminal issue evidence and improve the development system.

The table below is the formal event order inside those macro phases:

| Order | Stage | Required work | Exit evidence |
|---:|---|---|---|
| 1 | **Discover** | Capture an observed problem, requirement, or improvement candidate with reproducer/evidence and a candidate impact surface. | Each observation has an owner and issue/intake reference; discovery volume is not an exit criterion. |
| 2 | **Freeze** | Select the bounded batch, non-goals, KGI/acceptance criteria, and admission rule. Stop unrelated exploration. | Every admitted issue maps exactly once to the batch or a recorded successor; scope changes require manager adjudication. |
| 3 | **Plan** | Group symptoms by root cause; define change boundaries, hazards, counterexamples, tests, dependencies, and rollback/recovery. | An implementer can act without inventing a requirement or authority boundary. |
| 4 | **Risk-based Preflight** | Before mutation, independently challenge contracts that cross invariants, public/persisted interfaces, state machines, authority/security boundaries, migrations, or difficult recovery. | No unresolved authorization-changing ambiguity, contradiction, reproducible fail-open/contract-escape behavior, or unowned test surface. Low-risk bounded work may record `not_required` with manager rationale. |
| 5 | **Implement** | Apply the bounded root fix in owned worktrees and keep durable checkpoints. | The allowed write set and frozen invariants are respected; focused self-validation passes or exact gaps are reported. |
| 6 | **Validate** | Run focused checks first, then the applicable broad checks with exact command/result evidence. | A clean committed candidate exists; failures, skips, and unavailable evidence remain explicit. |
| 7 | **Candidate Freeze** | Bind candidate commit/tree, patch and changed paths, environment/toolchain evidence, packet identity, and reviewer exclusion set. | The review subject is immutable; later byte changes create a new candidate. |
| 8 | **Independent Review** | A report-only reviewer inspects the exact candidate, frozen obligations, changed paths, hazards, evidence, and candidate-created effects. | Every observation is evidence-bound; reviewer count and review types follow risk, not a fixed quota. |
| 9 | **Impact/Actionability Adjudication** | Separate severity from correction necessity. Assess impact, apply must-fix triggers, and assign one actionability disposition before any repair. | Acceptance-critical unknowns are investigated; every observation is terminally dispositioned; only required corrections enter causal classification. |
| 10 | **Bounded Repair / Rereview** | Freeze the smallest repair and rereview scope for required corrections, implement it, revalidate, and review the affected delta plus inherited obligations. | No non-required observation creates repair work; a changed contract/effect expands review only through an explicit scope decision. |
| 11 | **Integrate** | Compose accepted packages into the manager-owned canonical candidate and rerun the relevant integration checks. | One clean exact integrated revision is source-accepted. |
| 12 | **Install** | Build/install or select the artifact intended for actual use and bind it to the accepted source. | Installed-artifact identity and smoke evidence match the integrated source. |
| 13 | **Dogfood / Evaluate** | Exercise the installed artifact against real workflows and replay batch reproducers; collect product and development-flow outcomes. | Acceptance evidence, regressions, new blockers, and unavailable observations are recorded without suppressing valid findings. |
| 14 | **Close** | Give every frozen issue a final terminal disposition and synchronize detail files, indexes, root-cause map, roadmap/cycle state, and history. | The frozen batch has no unresolved item and all closure claims bind the exact integrated and applicable installed/dogfooded evidence, or an explicit `not_applicable` decision. |
| 15 | **Retrospect** | Evaluate quality, speed, cost, continuity, and process controls. | Record a dated `keep`, `change`, `stop`, or `experiment` decision and any separately admitted next work. |

`Integrate` establishes **source acceptance**. It is not final issue closure.
Issue detail files may record implementation and validation evidence before
Install/Dogfood, but `Close` occurs only after the installed artifact and
required real workflow have been evaluated. Historical records that used the
older `Close -> Install -> Dogfood` wording remain historical; do not rewrite
their evidence or pretend that source acceptance proved dogfood completion.

Lifecycle stages are explicit but applicability is scope-dependent. Product,
runtime, packaging, CLI/JSON, persistence, UX, or workflow behavior changes
require Install and Dogfood/Evaluate before their issues can Close. A docs-only
or process-only batch, or a `wontfix` disposition with no product mutation, may
mark Install/Dogfood `not_applicable` only through a manager decision naming the
scope, evidence, and reason and making no runtime claim. `not_applicable` is not
an implicit skip and must never fabricate an artifact or dogfood result.

Review observations follow the actionability policy in
`docs/development/ai/operating-protocol.md`; the report shape is
`docs/development/ai/validation-report-template.md`. Severity expresses impact,
not an automatic repair instruction. A reviewer-preferred implementation is not
a required correction unless frozen evidence or a must-fix trigger makes it one.

Do not start a new discovery batch while the active frozen batch still has open issues.

## Change Package Types

Separate work into reviewable packages:

| Package | Examples | Validation |
|---|---|---|
| Product implementation | Rust CLI/core/store changes, Python fallback fixes | Rust/Python gates as applicable. |
| Product documentation | CLI spec, known limitations, architecture docs | Link and consistency checks. |
| Management documentation | README, history, issue ledgers, cycle plans | Source-of-truth and count consistency checks. |
| AI acceleration infrastructure | `docs/development/ai/`, `docs/development/tools/` | `python3 docs/development/tools/check_ai_acceleration_docs.py`; dirty-tree classification when packaging. |
| Dogfood implementation | `subprojects/algorithm-evolution-agent-lab/src`, tests | Algorithm unittest gate and CodeFire dogfood status. |
| Dogfood documentation | Algorithm specs, design, operations docs | Algorithm docs consistency and CodeFire status. |

Avoid mixing all package types in one commit. If a dirty tree already mixes them, classify first and commit second.

## AI-Assisted Development

AI-assisted work is managed outside the CodeFire product surface.

Use `docs/development/ai/README.md` for:

- context pack selection;
- task packet creation;
- role routing and reasoning-effort selection;
- parallel lane setup;
- reviewer packet creation;
- prompt construction from packets and role routing;
- AI acceleration doc/tool checks;
- acceleration backlog items.

AI agents may accelerate implementation, review, docs synchronization, dogfood, and validation, but they must still follow the same issue lifecycle, dirty-tree rules, module boundaries, and validation policy as human contributors.

Do not convert AI workflow convenience into CodeFire product features unless the feature belongs to CodeFire's consistency, evidence, or automation CLI contract.

## Development-System KGI / KPI

Development-system performance is governed by
`docs/development/ai/development-flow-kgi-kpi.md`. KGI states the required outcome; KPI diagnoses
flow, quality, cost, continuity, and resource behavior. Raw agent count, lines, commits, transcript
size, and model tier are not progress indicators.

The active batch keeps a schema-checked record under `docs/development/ai/flow-metrics/`. Record at
least lane start, first trusted signal, first mutation, first durable checkpoint, candidate/review
decision, gate results, integration, provider incidents/reroutes, resource incidents, and lifecycle
completion. Bind an active log by byte-prefix length and SHA-256 so later appends do not rewrite
history.

Review the measurements:

- when a task completes;
- daily while a frozen batch is active;
- after every provider/host/resource incident;
- after each five-packet comparable cohort;
- at batch Retrospect.

Every review records `keep`, `change`, `stop`, or `experiment`. Threshold changes require a dated
cohort and evidence; do not loosen them to make an active task appear healthy.

## Dirty Tree Triage

Before resuming feature work with a large dirty tree:

1. Run `git status --short --branch`.
2. Group files by package type.
3. Identify generated files and temporary artifacts.
4. Check whether untracked files are intentional deliverables.
5. Run the cheapest validation relevant to each group.
6. Record unresolved validation gaps in the handoff.
7. Commit or otherwise checkpoint each coherent package separately.

Never use destructive cleanup to make the tree look clean. Existing user changes are project state until explicitly classified.

## Validation Policy

Validation evidence must name both the command and the result.

- A passing command is evidence.
- A failing command is evidence only if the failure cause is understood and documented.
- A skipped command is not evidence; it is an environment limitation or pending task.
- If `cargo` is unavailable, Rust release readiness remains unverified in that environment.
- Docs-only changes can skip full tests, but not consistency checks.

## Current-State Hygiene

Follow these rules to prevent stale docs:

- `README.md` should describe stable structure and commands, not live issue counts.
- `docs/development/README.md` may contain a current snapshot and must carry a `Last audited` line.
- `current-cycle-burn-down-plan.md` should not accumulate every historical dogfood pass as live current state.
- Historical numbers are allowed only when labeled as historical.
- If two files disagree, update the management hub first, then either fix or explicitly demote the stale text.

## Long-Term Roadmap Control

`docs/development/strategy/modernization-roadmap.md` is the only active strategic roadmap. It answers:

- What risk or product question does the stage resolve?
- Which invariant, experiment, or operational evidence defines completion?
- What is explicitly out of scope or blocked by an earlier gate?
- Which accepted decision or product contract receives the result?

Release and format identity belongs only in `docs/development/strategy/release-and-format-roadmap.md`. Do not create a new version-number plan as a task board. Active bounded execution belongs in the current burn-down plan; outcomes belong in history and the relevant issue/evidence record.

## Handoff Template

Every substantial handoff should include:

```text
State:
- Git:
- CodeFire product:
- Dogfood target:
- Open issues:
- Validation:

Changed:
- Product:
- Management docs:
- Dogfood:

Risks:
- 

Next:
-
```

This keeps long-term development resilient even when work spans many sessions or agents.
