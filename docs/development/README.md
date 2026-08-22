# CodeFire Development Management Hub

This directory is the control plane for CodeFire development. Use it to understand current state, plan work, close issues, and keep the Rust implementation, Python fallback, dogfood project, and documentation aligned.

## Current Operating Snapshot

Last audited: 2026-08-22.

- Execution is milestone-based. The live board is `docs/development/milestones.md`. `MS-adoption-wedge` exited after the post-B1 source-blind rerun reached terminal `adopt` and a fresh Grok audit returned `ACCEPT`; M1 is now `MS-release-operational-trust`, beginning with two parallel read-only classification packets.
- Codex orchestrates. Individual and 30-minute audit agents are Claude Opus 5 or Grok 4.6. Codex does not implement or audit.
- Rust `codefire-rs` is the default CLI line; Python `codefire` remains the reference/fallback implementation.
- CFB detail ledger: 137 total; 135 `fixed`, 2 `investigating`. Open/investigating count: 2.
- CFR detail ledger: 175 total; 159 `fixed`, 10 `fixed in v0.7`, 6 `investigating`. Open/investigating count: 6.
- The latest accepted source-bound Rust runner is `/workspace/worktrees/ms-adoption-experiment-20260822/target/release/codefire-rs`, SHA-256 `90cb2895aadc45b559a96207cdd5ae3b08485f9a7e5a07014db8cca8b5b90ce9`; provenance `1731b863…` binds it to source `c78d5e0e`, manifest `3925fc03…` over 101 files. It is adoption evidence, not yet a public release candidate.
- The current Git worktree contains the intentional uncommitted review, invariant-fix, regression, ADR, and management-document packages. They remain classified project state and must not be destructively cleaned or presented as a clean checkpoint.
- CFB-120..132 and the complete Stage 1 Dogfood-blocker lifecycle are closed. Canonical Algorithm is sealed at `CF-COMMIT-2792b54ad730e6d644668c38`, `open-clean`, and zero Fires after the agent-local-contract milestone; all remote-M1 attempts preserved it exactly.
- RVC-011 / CFR-161 is closed through Install, installed counterexample replay, both real-repository read-only probes, canonical trace-warning verification, and Retrospect. ADR-013 now owns the typed committable decision and non-relaxable safety floor.
- RVC-039 / CFR-162 is closed through the full lifecycle. ADR-014 makes unresolved text, binary, and semantic merge facts authoritative blockers and makes `merge --continue` the explicit validated resolution boundary.
- `BATCH-2026-07-review-portfolio-completion` is historical. CFB-133, CFB-136, and CFR-168..173 remain open/investigating in the ledgers and are backlog-parked. The bounded CFR-171..173 adoption slices were measured sufficient, but their broader roots were neither closed nor admitted to release M1.
- The base environment still does not expose `cargo` on `PATH`; a durable pinned toolchain/CI identity remains release work.

If any line above changes, update this snapshot and append the reason to `docs/development/history.md`.

## Management Sources of Truth

| Question | Source of truth | Update rule |
|---|---|---|
| What is the project-management model? | `docs/development/project-management.md` | Update when the workflow or document ownership model changes. |
| What happened? | `docs/development/history.md` | Append-only chronological work log. Do not rewrite history to fix current state. |
| What is the active issue ledger? | `docs/development/bug-backlog.md`, `docs/development/bug-issues/` | Summary row and detail file must agree. |
| What source-review issues exist? | `docs/development/code-review-issues-2026-06-06.md`, `docs/development/code-review-issues/` | Index count and detail statuses must agree. |
| What may be worked now? | `docs/development/milestones.md`, `docs/development/milestone-operating-model.md` | Five-ahead board; only M1 is active. |
| What is the historical burn-down batch? | `docs/development/current-cycle-burn-down-plan.md` | Ledger counts for checkers. Not permission to work. |
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

The CFB-120..132 and CFR-161..167 roots are complete. Active work is **only** M1 on `milestones.md`: `MS-release-operational-trust`. Its rights/source-identity classification and isolated release/verify/rollback topology classification are the two ready read-only packets. Queued, not started:

1. M2: first product-value proof (bounded roadmap Stage 5 slice).
2. M3: ecosystem conformance seed (bounded roadmap Stage 6 slice).
3. M4: ecosystem multi-implementation proof (next bounded roadmap Stage 6 slice).
4. M5: ecosystem third-organization proof (next bounded roadmap Stage 6 slice).

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
| Management | `README.md`, `project-management.md`, `milestones.md`, `milestone-operating-model.md`, `history.md`, `current-cycle-burn-down-plan.md` | State, five-ahead execution, and historical cycle narrative. |
| Strategy | `strategy/README.md`, engineering principles, product strategy, modernization and release/format roadmaps, decision register | Promoted decisions, bounded hypotheses, and the only active strategic roadmap. |
| Exploration | `exploration/` | Non-authoritative alternatives, falsifiable hypotheses, and experiment designs from the three completed exploration phases. |
| CodeFire improvement review | [`codefire-review-portfolio/`](codefire-review-portfolio/README.md) | Evidence-backed product, domain, architecture, implementation, workflow, and evolution reviews kept as next-discovery candidates until promotion. |
| AI acceleration | `ai/README.md`, `ai/context-packs.json`, `ai/schemas/`, `ai/evals/`, role/runtime policy, task/review packets, and `tools/` | External AI task definition, typed contracts, context compression, review, model routing, validation selection, and parallel-lane operation. |
| Issues | `bug-backlog.md`, `bug-issues/`, `code-review-issues-2026-06-06.md`, `code-review-issues/`, `issue-root-cause-map.json` | Product and source-review issue tracking. |
| Historical deletion audit | `archive/deletion-manifest.md`, `history.md`, Git history | Destinations and recovery path for removed completion/version plans. |
| Source architecture | `source-architecture.md`, `command-source-trace.md`, `source-data-model-catalog.md`, `change-playbook.md` | Stable ownership, exact command routing, data contracts, and task-oriented navigation. |
| Engineering rules | `agent-code-quality.md`, `agent-workflow.md`, `module-boundaries.md` | Code quality, workflow, and module-boundary constraints. |

## Long-Term Rule

When in doubt, prefer fewer current-state documents and better append-only history. Current state should be easy to find, easy to update, and impossible to contradict accidentally.
