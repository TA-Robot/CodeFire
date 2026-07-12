# Documentation Doubling Plan

This plan defines how to grow the algorithm evolution lab documentation from the current baseline to roughly twice its current size while keeping the documentation useful, navigable, and tied to actual research operations.

## Baseline

Measured on 2026-06-14 from `docs/**/*.md`:

| Metric | Current | 2x Target | Required Addition |
|---|---:|---:|---:|
| Markdown files | 19 | 32+ | 13+ files |
| Lines | 4,826 | 9,652 | 4,826 lines |
| Words | 38,303 | 76,606 | 38,303 words |

The primary completion target is useful coverage, not raw bulk. The line and word targets are guardrails that make the goal measurable.

## Scope

The doubling effort should expand these documentation families:

| Family | Current Role | Expansion Goal |
|---|---|---|
| `docs/operations/` | operator runbooks | become the main handbook for humans and agents |
| `docs/experiments/` | experiment log index | include templates, examples, and review-ready record patterns |
| `docs/development/` | implementation history and atlas | include maintenance plans, module ownership, and docs governance |
| `docs/spec/` | requirements | add process and documentation quality requirements when behavior changes |
| `docs/design/` | architecture and module design | add architecture narratives, lifecycle diagrams, and decision boundaries |

## Non-Goals

- Do not duplicate source code in prose.
- Do not create fake experiment results.
- Do not inflate docs with generic ML background that does not guide this project.
- Do not make SOTA claims without evidence.
- Do not replace CodeFire trace links with prose-only assertions.

## Batch Plan

| Batch | Target Files | Approx Lines | Purpose |
|---|---|---:|---|
| D1 | `documentation-doubling-plan.md`, `agent_execution_playbook.md`, `documentation_maintenance.md`, experiment examples | 700 | establish plan and first operational guides |
| D2 | `experiment_lifecycle.md`, `research_cycle_planning.md`, `evidence_and_claim_review.md` | 650 | deepen existing operations runbooks |
| D3 | `source-code-atlas.md` and module family guides | 700 | map modules to responsibilities, tests, and docs |
| D4 | `docs/design/*` | 800 | expand architecture narratives and design boundaries |
| D5 | `docs/spec/*` | 700 | add process-level requirements for evidence, docs, and branch workflow |
| D6 | `docs/experiments/` templates and sample records | 650 | make experiment logging repeatable |
| D7 | `docs/operations/` review checklists and troubleshooting | 650 | cover common failures and recovery paths |
| D8 | close audit across all docs | 500 | fix navigation, cross-links, and stale guidance |

Total planned addition: about 5,350 lines. That leaves slack over the minimum 4,826-line increase.

## Quality Bar

Every new document should satisfy these checks:

- It tells the reader when to use it.
- It names inputs, outputs, and completion criteria.
- It distinguishes required work from optional advice.
- It references the relevant source or operation family.
- It avoids claims that require evidence unless the evidence path is named.
- It can be reviewed as a deterministic Markdown diff.

## Navigation Rules

- `docs/operations/README.md` is the entry point for repeatable workflows.
- `docs/development/source-code-atlas.md` is the entry point for implementation orientation.
- `docs/experiments/README.md` is the entry point for recording results.
- `docs/development/implementation-roadmap.md` tracks long-horizon documentation hardening.
- New long-lived docs must be discoverable from at least one of those entry points.

## Batch D1 Work Items

This branch starts Batch D1:

- add this doubling plan
- add agent execution playbook
- add documentation maintenance guide
- add experiment record examples
- update operations index
- update roadmap, todo, and history

## Completion Criteria

The doubling initiative is complete when:

- docs line count is at least 9,652
- docs word count is at least 76,606
- operations index links all operator-facing guides
- source atlas maps every major module family to an operational or design document
- experiment logging has at least one complete example and one negative-result example
- CodeFire status is clean after every documentation batch
- full unittest suite still passes

## Completion Audit

Measured after the D2-D8 expansion on 2026-06-14:

| Metric | Required Target | Current Measurement | Status |
|---|---:|---:|---|
| Lines | 9,652 | 10,955 | complete |
| Words | 76,606 | 118,466 | complete |

The closeout batch added:

- `docs/operations/runbook_catalog.md`
- `docs/development/module_ownership_guide.md`
- `docs/design/experiment_automation_boundaries.md`
- `docs/spec/documentation_evidence_requirements.md`
- `docs/experiments/experiment_record_patterns.md`

The navigation updates link these files from the operations handbook, experiment log index, source atlas, roadmap, todo checklist, and history.
