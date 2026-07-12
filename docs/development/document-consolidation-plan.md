# CodeFire Document Consolidation Plan

この文書は、CodeFireの文書を「増え続ける作業ログ」から「役割が明確で更新可能な知識体系」へ移行するための実行計画である。

## Problem Statement

現状には次の問題がある。

- current-state、plan、historyが複数fileに重複し、同じissue数やphaseが矛盾する。
- `current-cycle-burn-down-plan.md` がactive planと約2,800行の履歴を同時に持つ。
- v0.6からv1.0までのversion planが、完了済みissueを未来形で説明する。
- source navigation文書が多数あり、相互参照が増えた一方で現sourceへ追随していない。
- rootの旧MVP TODO、v0.2統合設計、現行Rust文書が並立する。
- AI acceleration master planが「探索を統合済み」と述べるが、検討過程と反証材料が存在しない。
- historyに高頻度dogfood cycleのほぼ同型の行が大量にあり、重要な設計判断を探しにくい。

Git historyは削除前の内容を保持する。したがって、現在の作業treeから古い文章を除去することと、監査可能性を失うことは同義ではない。

## Target Information Architecture

| Layer | Canonical location | Holds | Must not hold |
|---|---|---|---|
| Product truth | `README.md`, numbered product docs (`docs/00_*`–`docs/11_*`, `docs/13_*`), ADR, schemas | stable product semantics and user contract | live issue counts, daily work log |
| Development control | `docs/development/README.md`, `project-management.md`, active roadmap | current state, ownership, active gates | repeated historical cycles |
| Engineering map | consolidated source atlas, command trace, data catalog | source/data/navigation truth | product roadmap, issue diary |
| AI development system | `docs/development/ai/`, `.codex/` | role routing, packets, runtime config, evals | CodeFire product runtime orchestration |
| Exploration | `docs/development/exploration/` | hypotheses, alternatives, open questions | authoritative current state |
| Strategy | `docs/development/strategy/` | accepted principles, product direction, roadmap | raw brainstorm |
| Audit history | `history.md`, issue detail ledgers, Git history | what happened and evidence | mutable current-state claims |
| Archive | `docs/development/archive/` | superseded plans worth convenient lookup | active instructions |

## Classification Rules

Every document receives exactly one lifecycle state.

| State | Meaning | Action |
|---|---|---|
| `canonical` | authoritative and actively maintained | link from a hub; add validation |
| `supporting` | useful detail owned by a canonical doc | state owner and update trigger |
| `exploration` | non-authoritative working material | label claims and promotion target |
| `historical` | closed record still useful for audit | move to archive or keep in ledger |
| `generated` | reproducible output | document generator; avoid hand edits |
| `superseded` | replaced and not needed for daily work | remove after inbound links are migrated |

## Consolidation Batches

### Batch D1: Active State — Partial; destructive pruning deferred

Canonical hubs, issue counts, lifecycle labeling, and machine checks are in place. `current-cycle-burn-down-plan.md` and `todo-checklist.md` contained pre-existing uncommitted evidence when this cleanup began, so their destructive split/prune is deliberately deferred until an owner-reviewed checkpoint. The docs checker keeps this visible as a warning rather than deleting or rewriting evidence in place.

Keep canonical:

- `docs/development/README.md`
- `docs/development/project-management.md`
- `docs/development/history.md`
- `docs/development/bug-backlog.md` and detail ledgers

Change:

- Reduce `current-cycle-burn-down-plan.md` to the current bounded batch only.
- Replace table-internal historical cycles with a concise closed-batch summary and Git/history references.
- Replace `todo-checklist.md` as an active board with a small roadmap; retain completed task evidence in history or an archive.

Exit evidence:

- one current issue count;
- one active batch;
- no historical section that can be misread as current;
- machine-checked ledger counts.

### Batch D2: Version Plans — Completed 2026-07-11

Removed from the active worktree after audit:

- `v0.6-rust-rewrite-plan.md`
- `v0.7-issue-improvement-plan.md`
- `v0.8-cfb-burn-down-plan.md`
- `v0.9-contract-scale-plan.md`
- `v1.0-contract-validation-plan.md`
- `v0.6-readiness.md`
- `completion-plan.md`

Create one canonical release/format roadmap that separates:

- product version;
- crate/package version;
- repository format version;
- object/schema version;
- completed historical milestone;
- next release gate.

Canonical replacement: `strategy/release-and-format-roadmap.md`. It separates version axes, records historical milestone evidence, keeps unresolved candidates explicit, and does not claim current public release readiness. Removal evidence and recovery instructions are in `archive/deletion-manifest.md`.

### Batch D3: Source Documentation — Completed 2026-07-11

Removed overlapping family after source audit:

- `source-code-map.md`
- `source-code-blueprint.md`
- `source-code-mental-model.md`
- `source-code-anatomy.md`
- `source-code-implementation-atlas.md`
- `source-code-executable-mental-model.md`
- `source-code-reconstruction-guide.md`
- `command-source-trace.md`
- `source-data-model-catalog.md`

Canonical family:

1. `source-architecture.md`: crate/module ownership, invariants, data flow, persistence boundaries.
2. `command-source-trace.md`: generated or machine-checked command-to-parser/plan/apply/render map.
3. `source-data-model-catalog.md`: object and JSON types with compatibility rules.
4. `change-playbook.md`: task-oriented navigation and validation recipes.

The two retained paths were rebuilt from current source, active inbound links were migrated, and a source checker compares module, command, object-kind, and version markers against source. Recovery and unique-content disposition are recorded in `archive/deletion-manifest.md`.

### Batch D4: Legacy Product Material — Completed 2026-07-11

Removed after tracked/unmodified audit:

- `CodeFire_Design_Document_v0.2.md`, `.docx`, `.pdf`
- `IMPLEMENTATION_TODO.md`
- `docs/12_mvp_implementation_plan.md`
- `docs/decisions.md`

Durable destinations:

- current product semantics remain in the numbered product docs (`docs/00_*`–`docs/11_*`, `docs/13_*`);
- the completed task ledger remains in `todo-checklist.md`;
- the unique 12/16/24/64-hex Object ID tradeoff moved to `adr/ADR-008-object-id-digest-length.md`;
- the implemented four-crate boundary and rejection of the old eight-crate sketch are recorded in source architecture and `strategy/decision-register.md`;
- duplicate PDF/DOCX renderings had no inbound references or distribution role.

Recovery and per-file disposition are recorded in `archive/deletion-manifest.md`.

### Batch D5: AI Development System

Keep separate:

- exploratory reasoning under `exploration/`;
- accepted strategy under `strategy/`;
- executable operating system under `ai/`, `.codex/`, and checkers.

Replace the old rule “never hardcode a model name” with two layers:

- stable role/risk taxonomy independent of vendors;
- explicit, versioned runtime policy selecting the current model and reasoning adapters.

This lets the project intentionally use GPT-5.6 Sol without coupling product semantics to one model forever.

## Deletion Gate

A file can be deleted only when all conditions hold:

1. its lifecycle state is `superseded`;
2. unique requirements, decisions, commands, and evidence have a destination;
3. inbound links are updated or intentionally removed;
4. no active task packet requires it;
5. the docs checker passes after deletion;
6. Git history is sufficient for forensic recovery;
7. the deletion appears as its own reviewable change package.

## Required Tooling

- document inventory with title, line count, lifecycle state, owner, and inbound links;
- Markdown link checker;
- stale current-state phrase checker;
- issue/index/detail count checker;
- source-map freshness checker based on symbols rather than fixed line numbers;
- model/role runtime configuration checker;
- archive/deletion manifest.

## Completion Criteria

- A new contributor can find product truth, current state, architecture, AI workflow, and roadmap from two hub pages or fewer.
- No current count, active phase, or release claim has two authoritative owners.
- Closed version plans and repeated dogfood cycles do not dominate active documents.
- Every surviving development document declares purpose, owner, and update trigger directly or through its directory hub.
- Every deleted document has a recorded destination or explicit “no unique content” finding.
- All local documentation and AI-runtime checks pass.
