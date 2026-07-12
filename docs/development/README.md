# CodeFire Development Management Hub

This directory is the control plane for CodeFire development. Use it to understand current state, plan work, close issues, and keep the Rust implementation, Python fallback, dogfood project, and documentation aligned.

## Current Operating Snapshot

Last audited: 2026-07-11.

- Rust `codefire-rs` is the default CLI line; Python `codefire` remains the reference/fallback implementation.
- CFB detail ledger: 127 total; 120 `fixed`, 7 `open`.
- CFR detail ledger: 160 total; 150 `fixed`, 10 `fixed in v0.7`.
- `subprojects/algorithm-evolution-agent-lab/` reports CodeFire `open-clean` with 0 open fires when checked via the local Rust binary.
- The repository has a large dirty tree that mixes CodeFire product changes, management docs, and algorithm dogfood work. Split this before further product development.
- The base environment does not expose `cargo` on `PATH`. The 2026-07-11 audit used an isolated Rust 1.97.0 toolchain under `/tmp` and passed fmt, locked workspace tests, and warning-denying Clippy; a durable pinned toolchain/CI identity is still release work.

If any line above changes, update this snapshot and append the reason to `docs/development/history.md`.

## Management Sources of Truth

| Question | Source of truth | Update rule |
|---|---|---|
| What is the project-management model? | `docs/development/project-management.md` | Update when the workflow or document ownership model changes. |
| What happened? | `docs/development/history.md` | Append-only chronological work log. Do not rewrite history to fix current state. |
| What is the active issue ledger? | `docs/development/bug-backlog.md`, `docs/development/bug-issues/` | Summary row and detail file must agree. |
| What source-review issues exist? | `docs/development/code-review-issues-2026-06-06.md`, `docs/development/code-review-issues/` | Index count and detail statuses must agree. |
| What is the active burn-down batch? | `docs/development/current-cycle-burn-down-plan.md` | Keep only active batch state at the top. Historical cycle detail belongs in `history.md`. |
| What is the active strategic roadmap? | `docs/development/strategy/modernization-roadmap.md` | One stage/gate roadmap; exploration and old version labels are not task boards. |
| What is the release/format identity? | `docs/development/strategy/release-and-format-roadmap.md` | Keep product/package/repository/object/schema versions separate. |
| How is the Rust code organized? | `source-architecture.md`, `command-source-trace.md`, `source-data-model-catalog.md`, `change-playbook.md` | Update with command/module/type/persistence boundary changes; run the source-doc checker. |
| What are known constraints? | `docs/development/known-limitations.md` | Update when behavior is intentionally limited or deferred. |

Do not maintain live project state in more than one place. When a current count or phase name appears in multiple documents, this hub wins until the duplicate is removed or marked historical.

## Work Intake Order

Before starting implementation:

1. Read `AGENTS.md`.
2. Read this file.
3. Read `project-management.md`.
4. Read `ai/README.md` when the work will be delegated to or reviewed by an AI agent.
5. Check `git status --short --branch`.
6. Check the relevant issue detail file or create one from the template.
7. Decide whether the change touches CodeFire product code, project-management docs, or the dogfood subproject.
8. Choose the validation gate before editing.

## Current Cleanup Priorities

These are organization tasks that should happen before new feature development:

1. Use `docs/development/worktree-cleanup-plan.md` to split the dirty tree into reviewable groups: CodeFire product, CodeFire management docs, algorithm dogfood code, algorithm dogfood docs.
2. Keep CFB-120..CFB-126 frozen as the first Stage 1 correctness batch; do not claim zero open product issues.
3. Save the management, strategy/source, document-cleanup, and AI-infrastructure packages in the declared order before product implementation.
4. Preserve the now-green Rust source gate in a durable pinned toolchain/CI environment; the local release binary is current-worktree smoke evidence but remains unbound until a clean checkpoint build.
5. After the organizational save gate, resolve sealed-base/publish, path-containment, Trace Graph endpoint, mutable-state locking, full-digest collision, and import-idempotency invariants before discretionary feature work.

## Validation Matrix

| Change type | Required checks |
|---|---|
| Rust CLI/core/store | `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace` |
| Python fallback | `python3 -m py_compile codefire tests/test_codefire_cli.py`; `python3 -m unittest discover -s tests -v`; `./demo.sh` |
| Installer | `bash -n install.sh`; temp-prefix install smoke; `codefire --help`; `codefire-py --help` |
| Development docs only | `python3 docs/development/tools/check_project_docs.py`; source navigation changes also run `python3 docs/development/tools/check_source_docs.py`. Full product tests are not required. For AI/strategy changes, run the complete AI-development gate. |
| Algorithm dogfood | `PYTHONPATH=src python3 -m unittest discover -s tests -v` inside `subprojects/algorithm-evolution-agent-lab/` |
| CodeFire dogfood state | `codefire status --path subprojects/algorithm-evolution-agent-lab --json` or local binary equivalent |

If an environment cannot run a required gate, record that limitation explicitly in the handoff.

## Document Families

| Family | Files | Purpose |
|---|---|---|
| Management | `README.md`, `project-management.md`, `history.md`, `current-cycle-burn-down-plan.md` | State, workflow, cycle execution, and handoff control. |
| Strategy | `strategy/README.md`, engineering principles, product strategy, modernization and release/format roadmaps, decision register | Promoted decisions, bounded hypotheses, and the only active strategic roadmap. |
| Exploration | `exploration/` | Non-authoritative alternatives, falsifiable hypotheses, and experiment designs from the three completed exploration phases. |
| AI acceleration | `ai/README.md`, `ai/context-packs.json`, `ai/schemas/`, `ai/evals/`, role/runtime policy, task/review packets, and `tools/` | External AI task definition, typed contracts, context compression, review, model routing, validation selection, and parallel-lane operation. |
| Issues | `bug-backlog.md`, `bug-issues/`, `code-review-issues-2026-06-06.md`, `code-review-issues/`, `issue-root-cause-map.json` | Product and source-review issue tracking. |
| Historical deletion audit | `archive/deletion-manifest.md`, `history.md`, Git history | Destinations and recovery path for removed completion/version plans. |
| Source architecture | `source-architecture.md`, `command-source-trace.md`, `source-data-model-catalog.md`, `change-playbook.md` | Stable ownership, exact command routing, data contracts, and task-oriented navigation. |
| Engineering rules | `agent-code-quality.md`, `agent-workflow.md`, `module-boundaries.md` | Code quality, workflow, and module-boundary constraints. |

## Long-Term Rule

When in doubt, prefer fewer current-state documents and better append-only history. Current state should be easy to find, easy to update, and impossible to contradict accidentally.
