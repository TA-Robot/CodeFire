# Development Document Deletion Manifest

Status: canonical audit record

Owner: Documentation Owner

Last reviewed: 2026-07-11

This manifest records tracked development documents removed from the active worktree after their durable content was promoted, rejected, or preserved by more authoritative evidence. The original text remains recoverable from Git history at or before base revision `12ad19d9a0e4d1a1b2977c4c811dd748cbff44d8`.

## Batch D2-2026-07-11 — Version and Completion Plans

| Removed path | Lifecycle finding | Durable destination | Unique-content disposition |
|---|---|---|---|
| `docs/development/completion-plan.md` | Superseded pre-implementation MVP plan. | product definition, core/workflow docs, `../strategy/modernization-roadmap.md` | M0–M8 behavior is implemented/documented; first-sprint prose had no remaining requirement. |
| `docs/development/v0.6-rust-rewrite-plan.md` | Completed internal Rust migration plan mixed with untested candidates. | `../strategy/release-and-format-roadmap.md`, engineering principles, product/automation docs, issue/history evidence | Performance numbers not promoted; eight-crate sketch rejected; compatibility, sandbox, policy, extractor, release, and Python-retirement candidates explicitly dispositioned. |
| `docs/development/v0.6-readiness.md` | Dated 2026-06-05 milestone snapshot presented as current readiness. | `../strategy/release-and-format-roadmap.md`, development validation matrix, runbook, history | Exact snapshot summarized as historical and explicitly not reusable for current release evidence. |
| `docs/development/v0.7-issue-improvement-plan.md` | Closed CFR batch log with stale “partial/next” state. | CFR index/detail ledger, history, release milestone mapping | No durable product requirement unique to the plan. |
| `docs/development/v0.8-cfb-burn-down-plan.md` | Closed CFB planning/status log. | CFB ledger, root-cause map, history, release milestone mapping | Root-fix themes already exist in product contracts/issues/tests. |
| `docs/development/v0.9-contract-scale-plan.md` | Closed contract/scale issue plan and draft release notes. | Stage 1–2 roadmap, issue ledgers, product contracts, engineering principles | Focused-regression-before-refactor rule promoted. |
| `docs/development/v1.0-contract-validation-plan.md` | Closed issue plan incorrectly usable as a public v1.0/current target. | Stage 1–4 roadmap, product/automation docs, release/format roadmap | Partial CommandSpec, normalized plan, richer NextAction, and issue-evidence candidates explicitly dispositioned. |

## Deletion Gate Evidence

- All seven files were tracked and unmodified before this cleanup batch.
- Active hub/instruction/index references are migrated in the same change package.
- Historical backtick references in `history.md`, completed TODO evidence, and issue records intentionally describe files that existed at the time; they are not active links.
- The historical labels v0.6–v1.0 are internal milestones or planning labels, not proven public SemVer releases.
- Post-deletion checks must include basename reference search, Markdown link validation, development-doc checker, and AI-development gate.

## Batch D3-2026-07-11 — Source Navigation Consolidation

| Removed path | Durable destination | Unique-content disposition |
|---|---|---|
| `docs/development/source-code-map.md` | `../source-architecture.md`, `../change-playbook.md` | Module/tour/test/change navigation moved; stale fixed-line bands rejected. |
| `docs/development/source-code-blueprint.md` | `../source-architecture.md`, `../change-playbook.md` | Five-machine model, invariant and feature-change routes moved; completed debt claims removed. |
| `docs/development/source-code-mental-model.md` | `../source-architecture.md`, `../change-playbook.md` | Typed flow, workflow and review hotspots moved; obsolete commit/envelope/layout flows removed. |
| `docs/development/source-code-anatomy.md` | all four canonical source docs | Module/function ownership and recipes moved; incorrect layout/type examples discarded. |
| `docs/development/source-code-implementation-atlas.md` | all four canonical source docs | Command/object/module/review information rebuilt from current symbols; line snapshots discarded. |
| `docs/development/source-code-executable-mental-model.md` | `../source-architecture.md` | Compact call graph and data lifetime integrated; incorrect opened/commit flow removed. |
| `docs/development/source-code-reconstruction-guide.md` | `../source-architecture.md`, `../change-playbook.md` | Crossing points, recipes and reading checklist moved; stale skeleton/envelope claims removed. |

Retained paths `command-source-trace.md` and `source-data-model-catalog.md` were rewritten rather than appended. `check_source_docs.py` now fails when source modules, registered commands, object kinds, or selected version constants diverge from their markers.

## Batch D4-2026-07-11 — Legacy Product Material

| Removed path | Lifecycle finding | Durable destination | Unique-content disposition |
|---|---|---|---|
| `CodeFire_Design_Document_v0.2.md` | Superseded aggregate of the initial 00–13 product document set. | current numbered product docs (`docs/00_*`–`docs/11_*`, `docs/13_*`), strategy, canonical source docs | Current documents extend or replace every durable chapter; the completed chapter 12 plan moved to the ledger and the obsolete eight-crate sketch is explicitly rejected by STR-R09. |
| `CodeFire_Design_Document_v0.2.docx` | Generated rendering of the same v0.2 Markdown. | Git history and text-searchable current product docs | No unique text and no inbound/distribution reference. |
| `CodeFire_Design_Document_v0.2.pdf` | Generated rendering of the same v0.2 Markdown. | Git history and text-searchable current product docs | No unique text and no inbound/distribution reference. |
| `IMPLEMENTATION_TODO.md` | Completed initial checklist duplicated by the detailed ledger. | `../todo-checklist.md`, history | All 85 tasks are represented with status/dependency/evidence in the surviving ledger. |
| `docs/12_mvp_implementation_plan.md` | Duplicate of chapter 12 in the v0.2 aggregate and completed MVP prose. | `../todo-checklist.md`, modernization roadmap, current product docs | Success conditions and completion evidence survive; no active task packet depends on it. |
| `docs/decisions.md` | Bootstrap decision scratchpad/template with one durable project decision. | `../../../adr/ADR-008-object-id-digest-length.md`, `../../08_object_store.md` | The 12/16/24/64-hex alternatives, decision, and legacy-read compatibility moved to ADR-008. |

All six files were tracked and unmodified before removal. The Markdown design, rendered copies, TODO, and MVP plan are recoverable at revision `a3fafdd2c7b5fb79702f51813143ed1e091846d8`; the final decision scratchpad is recoverable at `3c197a9602ba75e08922cb6e141b9253e61f61f3`. Historical mentions inside exploration notes remain non-authoritative observations.

## Recovery

Use Git history for forensic lookup rather than restoring these files as active instructions. If a removed requirement becomes relevant, promote the smallest current requirement to a canonical product/strategy/ADR location and cite the historical revision; do not resurrect the entire plan.
