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
| Strategy and roadmap | `docs/development/strategy/README.md` | Accepted principles, product direction, decision status, modernization gates, and release/format identity. |
| Exploration | `docs/development/exploration/README.md` | Non-authoritative hypotheses, alternatives, falsifiers, and experiments. |
| History | `docs/development/history.md` | Append-only work log. |
| Active batch | `docs/development/current-cycle-burn-down-plan.md` | Current burn-down batch only. Move old detail into history or mark it historical. |
| Issue ledgers | `bug-backlog.md`, `bug-issues/`, `code-review-issues-2026-06-06.md`, `code-review-issues/` | Issue facts, status, evidence, and closure. |
| Deletion audit | `docs/development/archive/deletion-manifest.md` | Destination and recovery record for superseded tracked prose. |
| Source architecture | `source-architecture.md`, `command-source-trace.md`, `source-data-model-catalog.md`, `change-playbook.md` | Module ownership, command routing, data contracts, and change recipes. |

## State Model

Project state is the intersection of five facts:

1. Git state: branch, ahead/behind, dirty files, untracked files.
2. CodeFire product state: implementation status, issue ledgers, validation gates.
3. Dogfood CodeFire state: target repo branch state, open fires, sealed commit.
4. Validation state: which checks passed, failed, or could not run.
5. Documentation state: whether current-state documents agree.

Do not infer current project state from old cycle headings, old remaining counts, or historical issue IDs. Recompute from the ledgers and record the result in the management hub.

## Issue Lifecycle

Use CFB for dogfood/product issues and CFR for source-review issues.

1. Create a detail file from the relevant template.
2. Add or update the index row.
3. Set `Status` to `open`, `investigating`, `fixed`, `fixed in v0.7`, or `wontfix`.
4. Link the issue to a root cause or version plan when the fix is not trivial.
5. Add regression evidence before marking fixed.
6. Update `history.md` with the closure and validation evidence.

The detail file status is authoritative for closure. Index files are navigational summaries and must be kept consistent with detail files.

## Cycle Lifecycle

Use this lifecycle for long-running CodeFire improvement work:

1. **Discover**: dogfood or source review identifies issues.
2. **Freeze**: define a bounded batch and stop adding unrelated issues to it.
3. **Plan**: group issues by root fix, not by symptom.
4. **Implement**: fix the root cause with focused tests.
5. **Close**: update detail file, index, root-cause map, and history.
6. **Install**: install or select the built CodeFire binary intended for dogfood.
7. **Dogfood**: use the fixed CodeFire against the algorithm project.
8. **Retrospect**: record new issues or explicitly record that none were found.

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
