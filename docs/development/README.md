# CodeFire Development Management Hub

This directory is the control plane for CodeFire development. Use it to understand current state, plan work, close issues, and keep the Rust implementation, Python fallback, dogfood project, and documentation aligned.

## Current Operating Snapshot

Last audited: 2026-09-05.

- Active execution is **MS-recovery-baseline** on [milestones](milestones.md), following the user's accepted recovery plan. The ecosystem sequence is parked until repeat use establishes a real interoperability need.
- Grok 4.6 is the primary worker. Fable 5.1 is scarce and explicit-only. All providers must use verified subscription allowance; metered billing, purchased credits, extra usage, and auto top-up are forbidden.
- The original `/workspace/project` main is ahead of origin by 92 commits and has a pre-existing cherry-pick sequencer with no current `CHERRY_PICK_HEAD`. It is preserved. Recovery runs on `recovery/subscription-first-20260905` in `.codex-worktrees/recovery-20260905`.
- Original worktree changes at recovery start were the prior-turn proposal and its exploration link. The earlier assertion that it held large uncommitted review/invariant packages was stale. A clean porcelain listing did not expose the outstanding sequencer; full `git status` did.
- Native current-session Codex subagents handled recovery environment/tooling and independent billing review. Grok billing preflight now verifies purchased credits/PAYG/auto-topup zero or off and included quota available; Grok is executing independent runtime review and the admitted product fix. No external Codex API fallback is enabled.
- Rust `codefire-rs` remains the default implementation; Python is reference/fallback. Rust 1.98.1 is installed locally and pinned; fmt, strict Clippy, build and help/version pass. The final CFB-138 snapshot passes CLI 782, separate integration 7, and Core 89 tests at four-way concurrency, including all original CLI failures. The previously unrun Store suite exposes a separate replacement-file deletion/abort and blocks M1; Util/doc-tests have not yet run. Historical binaries remain historical evidence only.
- Runtime repair REC-03 resolved 18 pre-existing model/role/config/CLI adapter errors. The combined AI development gate passed; independent Grok runtime review returned ACCEPT, with subsequent execute-refusal independently checked. Final integration reruns the gate after all tool changes.
- CFB detail ledger: 139 total; 135 `fixed`, 4 `investigating`.
- CFR detail ledger: 175 total; 159 `fixed`, 10 `fixed in v0.7`, 6 `investigating`.
- Slot changes do not close issue roots; partially implemented roots must be inspected before new work.
- The previous product-value study remains literal `narrow`: one AI-assisted maintainer, not human adoption or market validation. Public JSON identifier reliance and configured-glob claim limits are inputs to M2.
- Historical release evidence is internal-only. No public release identity, rights resolution, external publication, or funded ecosystem has been established by this recovery.
- Canonical Algorithm and user-installed binaries are not mutation targets of M1. Product-code changes need their own reproduced M1 admission.

Update this snapshot from actual evidence and append the reason to history; do not copy live state into additional documents.

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

Only the current [M1 board](milestones.md) authorizes implementation. Recover reproducible source validation,
coherent runtime roles, and subscription-only execution. The next increments are the usable first-seal path,
fast safe changes, repeat use, and bounded distribution. Historical review/experiment details are evidence,
not an additional work queue.

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

## Local Rust baseline

The repository pins Rust 1.98.1. For the checkout-local toolchain installed during recovery:

```bash
source docs/development/tools/rust_env.sh
RUST_TEST_THREADS=4 cargo test --workspace --locked
```

The helper only selects ignored local toolchain/cache/temp directories; it does not install anything or
change shell profiles. Setup, checksum verification, removal and initial failures are recorded in
[REC-02](ai/task-packets/recovery-rust-baseline.md). Four test-case workers bound suite contention while
retaining the explicit threads/barriers inside concurrency tests. This is a harness setting, not a claim
that the original full-concurrency import timeout has been fixed.

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
