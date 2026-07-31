# CodeFire Development Management Hub

This directory is the control plane for CodeFire development. Use it to understand current state, plan work, close issues, and keep the Rust implementation, Python fallback, dogfood project, and documentation aligned.

## Current Operating Snapshot

Last audited: 2026-07-31 after the experimental AIA-047 recurrence-v2 integration.

- Rust `codefire-rs` is the default CLI line; Python `codefire` remains the reference/fallback implementation.
- CFB detail ledger: 138 total; 135 `fixed`, 3 `investigating`. Open/investigating count: 3.
- CFR detail ledger: 175 total; 159 `fixed`, 10 `fixed in v0.7`, 6 `investigating`. Open/investigating count: 6.
- Final installed `/tmp/codefire-cfr167-final.KWa8TC/bin/codefire` has source-matching SHA-256 `1483874b4c1de4a78478595d5d158cf34a12e5ec3b05255dfc8819f511f00272`; policy/coverage reversal, sealed digest binding, and canonical Dogfood pass.
- The current Git worktree contains the intentional uncommitted review, invariant-fix, regression, ADR, and management-document packages. They remain classified project state and must not be destructively cleaned or presented as a clean checkpoint.
- CFB-120..132 and the complete Stage 1 Dogfood-blocker lifecycle are closed. Canonical Algorithm is now sealed at `CF-COMMIT-e5ca290905c2a4b5e2c05bd4` with Atom hash schema 4, `sealed-snapshot-v1`, `open-clean`, and zero Fires; the configured legacy repository was not mutated.
- RVC-011 / CFR-161 is closed through Install, installed counterexample replay, both real-repository read-only probes, canonical trace-warning verification, and Retrospect. ADR-013 now owns the typed committable decision and non-relaxable safety floor.
- RVC-039 / CFR-162 is closed through the full lifecycle. ADR-014 makes unresolved text, binary, and semantic merge facts authoritative blockers and makes `merge --continue` the explicit validated resolution boundary.
- `BATCH-2026-07-review-portfolio-completion` remains frozen. Roots through CFR-167/CFB-134 completed through Retrospect; CFB-133, admitted CFR-168 blockers CFB-136/138, and CFR-168..173 are the 9 active issues. No unrelated Discovery is admitted until the batch completes.
- AIA-047 recurrence-v2 layers L1.1-L1.5 are complete on the experimental control-plane line integrated at `00ad88562068a6101a7530b84d5f52b0cff559f0`. The independent L1.5 decision used the recorded GPT-5.6/Codex fallback because Claude and Fugu were unavailable. This changes no CodeFire product bytes and proves no live reduction in review recurrence. L1.6 (manager-owned Cargo/KGI manifests plus a committed live-arm anchor in a separate clean validator source) is the next gate; v1 always returns `BLOCK`, and the Fugu blind branch cannot begin before the recorded 2026-08-03 provider window.
- The base environment still does not expose `cargo` on `PATH`; a durable pinned toolchain/CI identity remains release work.

If any line above changes, update this snapshot and append the reason to `docs/development/history.md`.

## Management Sources of Truth

| Question | Source of truth | Update rule |
|---|---|---|
| What is the project-management model? | `docs/development/project-management.md` | Update when the workflow or document ownership model changes. |
| How is development speed/quality/cost measured? | `docs/development/ai/development-flow-kgi-kpi.md`, `docs/development/ai/flow-metrics/` | Record lane events continuously; review per task, incident, five comparable packets, and Retrospect. |
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

The CFB-120..132, CFR-161, CFR-162, CFR-163/CFB-135, CFR-164, CFR-165/CFR-174/CFR-175, CFR-166/CFB-134, and CFR-167 roots are complete. The active constrained work is:

1. Complete AIA-047 L1.6, then run the required bootstrap `BLOCK` and rooted successor generations; do not claim review-bottleneck improvement before the anchored Cargo/KGI comparison and blind causal audit.
2. Implement CFR-168 bounded graph/object/HTTP work and stable counters without changing decision semantics.
3. Burn down architecture, automation, and claim roots in the frozen dependency order; CFB-133 closes with CFR-170.
4. Run CFR-172/173 experiments only after their safety/contract predecessors, record adopt/narrow/stop outcomes, then perform one portfolio-wide completion audit.

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
| CodeFire improvement review | [`codefire-review-portfolio/`](codefire-review-portfolio/README.md) | Evidence-backed product, domain, architecture, implementation, workflow, and evolution reviews kept as next-discovery candidates until promotion. |
| AI acceleration | `ai/README.md`, `ai/development-flow-kgi-kpi.md`, `ai/flow-metrics/`, `ai/context-packs.json`, `ai/schemas/`, `ai/evals/`, role/runtime policy, task/review packets, and `tools/` | External AI task definition, typed contracts, context compression, review, model routing, validation selection, parallel-lane operation, and KGI/KPI measurement. |
| Issues | `bug-backlog.md`, `bug-issues/`, `code-review-issues-2026-06-06.md`, `code-review-issues/`, `issue-root-cause-map.json` | Product and source-review issue tracking. |
| Historical deletion audit | `archive/deletion-manifest.md`, `history.md`, Git history | Destinations and recovery path for removed completion/version plans. |
| Source architecture | `source-architecture.md`, `command-source-trace.md`, `source-data-model-catalog.md`, `change-playbook.md` | Stable ownership, exact command routing, data contracts, and task-oriented navigation. |
| Engineering rules | `agent-code-quality.md`, `agent-workflow.md`, `module-boundaries.md` | Code quality, workflow, and module-boundary constraints. |

## Long-Term Rule

When in doubt, prefer fewer current-state documents and better append-only history. Current state should be easy to find, easy to update, and impossible to contradict accidentally.
