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
| Live milestones | `docs/development/milestones.md`, `docs/development/milestone-operating-model.md` | Five-ahead execution board and admission rules. This is what may be worked now. |
| Historical batch | `docs/development/current-cycle-burn-down-plan.md` | Frozen review-portfolio narrative. Not permission to work. |
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

Long-running work is scheduled on the five-ahead board in `milestones.md`, governed by `milestone-operating-model.md`.

1. **Name five slots**: M1 is the next exit; M2–M5 are queued only.
2. **Admit**: an issue, review finding, or extra test enters M1 only if the M1 exit cannot be met without it.
3. **Park**: everything else is assigned M2–M5 (or `unscheduled`) and is not active work.
4. **Plan the graph**: name the critical path to the M1 exit and the parallel set of independent packets. Launch the parallel set immediately.
5. **Implement**: Codex orchestrates; Claude Opus 5 / Grok 4.6 individual agents do the assigned M1 packets.
6. **Audit**: a Claude or Grok auditor, never Codex, runs every 30 minutes against plan vs progress.
7. **Review**: only `block-m1` findings are required. Other comments are parked.
8. **Validate**: run the M1 card's command list, not the historical full catalog, unless the card names a wider gate.
9. **Exit and shift**: close or park every M1 item, record evidence in `history.md`, then shift slots.

Do not start M2 while M1 is open. Do not require every open CFB/CFR issue to close before the next milestone. The revoked rule was "discover, freeze, burn down every issue, then discover again".

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
- Active-milestone work runs only the validation list on that milestone card. Workspace-wide product gates wait until a later slot names them.

## Current-State Hygiene

Follow these rules to prevent stale docs:

- `README.md` should describe stable structure and commands, not live issue counts.
- `docs/development/README.md` may contain a current snapshot and must carry a `Last audited` line.
- `milestones.md` is the live execution board. `current-cycle-burn-down-plan.md` must not be treated as permission to work.
- `current-cycle-burn-down-plan.md` should not accumulate every historical dogfood pass as live current state.
- Historical numbers are allowed only when labeled as historical.
- If two files disagree, update the management hub first, then either fix or explicitly demote the stale text.

## Long-Term Roadmap Control

`docs/development/strategy/modernization-roadmap.md` is the only active strategic roadmap. It answers:

- What risk or product question does the stage resolve?
- Which invariant, experiment, or operational evidence defines completion?
- What is explicitly out of scope or blocked by an earlier gate?
- Which accepted decision or product contract receives the result?

Release and format identity belongs only in `docs/development/strategy/release-and-format-roadmap.md`. Do not create a new version-number plan as a task board. Active bounded execution belongs on `milestones.md`; outcomes belong in history and the relevant issue/evidence record. The burn-down plan is historical.

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
