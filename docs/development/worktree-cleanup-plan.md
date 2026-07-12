# CodeFire Worktree Cleanup Plan

This completed record documents how the mixed dirty tree was classified and saved before new product development resumes.

Last audited: 2026-07-12.

## Goal

- Separate CodeFire product changes from management-document changes and dogfood-project changes.
- Preserve all intentional work without destructive cleanup.
- Define reviewable save/commit packages.
- Record validation gaps explicitly before handoff.

## Current State

- Git branch: `main`.
- Remote relation: local `main` is ahead of `origin/main`; no push was performed by this organization pass.
- Dirty tree: clean after the purpose-specific organization checkpoint.
- Product issue state: the organization/source audit opened CFB-120..CFB-126 and the restored gate found/fixed CFB-127; CFB/CFR totals are 287 tracked, 280 closed, 7 open.
- Dogfood target state: `subprojects/algorithm-evolution-agent-lab/` is `open-clean` with 0 open fires through the local Rust binary.
- Rust validation: base `PATH` still has no cargo, but an isolated `/tmp` Rust 1.97.0 toolchain passed fmt, locked workspace tests, and warning-denying Clippy. A durable pinned toolchain and clean-HEAD provenance build remain pending.
- The managed sandbox denies socket creation. Five Python HTTP/HTTPS tests failed only there and passed when rerun outside that network restriction.
- Known probe: `target/release/codefire-rs branch list --path subprojects/algorithm-evolution-agent-lab --json` timed out after 10 seconds with no output during the 2026-07-11 audit.

## Classification

| Package | Files | Meaning | Validation |
|---|---|---|---|
| Management docs rebuild | `README.md`, `AGENTS.md`, `docs/development/README.md`, `docs/development/project-management.md`, `docs/development/current-cycle-burn-down-plan.md`, `docs/development/codefire-improvement-cycle.md`, `docs/development/todo-checklist.md`, `docs/development/history.md` | Rebuilt the long-term project-management entry points and corrected stale current-state claims. | Project docs checker; no full product test required. |
| Strategy and source architecture | `docs/development/exploration/`, `docs/development/strategy/`, four canonical source docs, `adr/ADR-008-object-id-digest-length.md` | Preserves the three exploration phases, promotes accepted principles/roadmaps, and replaces stale source maps with source-checked canonical documents. | Project/source docs checks and AI-development gate. |
| Document cleanup | deletion manifest/consolidation plan plus tracked removals from D2–D4 | Removes superseded version plans, source guides, duplicate design renders, completed MVP prose, and the old decision scratchpad after durable promotion. | Basename/inbound search, project/source docs checks, deletion manifest review. |
| AI development acceleration infrastructure | `.codex/`, `docs/development/ai/`, `docs/development/tools/` | Adds external AI context packs, typed task/context/review contracts, safe role execution, model routing, historical evals, and local gates/classification. | `python3 docs/development/tools/check_ai_development.py`; `python3 docs/development/tools/classify_worktree.py --json`. |
| CodeFire branch/workflow fixes | `crates/codefire-cli/src/*.rs`, `crates/codefire-core/src/lib.rs`, `docs/04_cli_spec.md`, `docs/development/known-limitations.md`, issue metadata | Product implementation and issue closure work for branch workflow, path-safe next actions, Markdown atom filtering, non-Atom visibility, remote clone/dry-run safety, and related tests. | Requires Rust fmt/clippy/test in a Rust-capable environment. |
| Algorithm nameaudit implementation | `subprojects/algorithm-evolution-agent-lab/src/evoagent/nameaudit.py`, `subprojects/algorithm-evolution-agent-lab/tests/test_nameaudit.py` | Adds identifier audit, report, CI, JSON, gate, and note-generation capabilities to the dogfood project. | `PYTHONPATH=src python3 -m unittest discover -s tests -v` passed with 281 tests. |
| Algorithm documentation expansion | `subprojects/algorithm-evolution-agent-lab/docs/operations/`, documentation-doubling docs, module ownership guide, experiment record patterns, documentation evidence requirements, experiment automation boundaries | Expands the dogfood project's operational and evidence documentation. | Docs consistency plus algorithm unittest if bundled with nameaudit implementation. |

## Completed Save Order

1. **Management docs rebuild** — `e8e993e`
   - Save the new project-management structure first.
   - This makes later commits easier to understand.
   - Include only management docs and root `README.md` / `AGENTS.md` changes.

2. **AI acceleration infrastructure** — `a20d7c6`
   - Saved project Codex routing, AI contracts, context packs, evals, and local tooling.

3. **Exploration and promoted strategy** — `0f9d4e0`
   - Saved all three exploration phases and the accepted strategy/roadmap family.

4. **Document cleanup** — `37e1e03`
   - Removed 20 superseded tracked artifacts with a recovery manifest and consolidation record.

5. **Issue ledger reconciliation** — `9023556`
   - Saved CFB-098..CFB-127 and synchronized the indexes/root-cause map.

6. **CodeFire product and canonical source docs** — `62eaa14`
   - Saved Rust implementation, tests, CLI spec, known limitations, ADR-008, and the four canonical source documents together.
   - Kept all dogfood project files out of this package.
   - Passed the required Rust gates before finalizing:
     - `cargo fmt --check`
     - `cargo clippy --workspace --all-targets -- -D warnings`
     - `cargo test --workspace`

7. **Algorithm nameaudit implementation** — `9417dcc`
   - Save dogfood source/test/spec/design/trace changes for `NameAudit` and related CI/report surfaces.
   - Confirm algorithm tests:
     - `PYTHONPATH=src python3 -m unittest discover -s tests -v`

8. **Algorithm documentation expansion** — `5302620`
   - Save the large docs-only expansion separately from executable dogfood code when possible.
   - Keep docs-only changes reviewable by topic: operations handbook, experiment records, ownership, evidence requirements.

9. **Final audit**
   - Check `git status --short --branch`.
   - Re-run CodeFire status for the dogfood target.
   - Confirm no current-state documents disagree.

## Non-Destructive Rules

- Do not run `git reset --hard`.
- Do not discard untracked files until they are classified as accidental.
- Do not combine CodeFire product fixes and algorithm dogfood expansion into one commit unless unavoidable.
- Do not treat historical cycle sections as current issue state.
- Do not start a new discovery batch while CFB-120..CFB-126 remain the frozen active batch.

## Validation Record

Known validation from the 2026-07-11 audit:

| Command | Result | Note |
|---|---|---|
| `PYTHONPATH=src python3 -m unittest discover -s tests -v` in `subprojects/algorithm-evolution-agent-lab/` | pass, 281 tests | Dogfood project is test-clean. |
| `target/release/codefire-rs status --json --path subprojects/algorithm-evolution-agent-lab` | pass | Returned `open-clean`, 0 open fires. |
| `python3 -m unittest discover -s tests -v` with the isolated cargo toolchain and socket permission | pass, 89 tests | Replaces the earlier split result: installer/completion and all HTTP/HTTPS tests pass in one run. |
| Focused HTTP/HTTPS Python tests outside socket-restricted sandbox | pass, 5 tests | Confirms the five network failures were environment restrictions, not observed product regressions. |
| `./install.sh --prefix /tmp/codefire-organize-smoke --binary target/release/codefire-rs`; installed Rust/Python help and Rust completion | pass | Smoke-only: the supplied binary is not bound to current Rust source. |
| `cargo fmt --check` | pass | Rust 1.97.0 isolated under `/tmp`; current worktree. |
| `cargo test --workspace --locked` | pass, 231 tests | Found and fixed date-dependent CFB-127 before the green rerun. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass | Removed four unused-assignment warnings and eight eager-null findings without behavior changes. |
| `PYTHONPATH=src python3 -m unittest discover -s tests -v` in the dogfood project | pass, 281 tests | Current rerun completed in 0.43 seconds. |
| `python3 docs/development/tools/check_ai_development.py --json` | pass | Docs/contracts/source/runtime/capability checks and 35 tool tests pass. The protected oversized-cycle warning was removed by the post-checkpoint archive split. |
| `python3 docs/development/tools/classify_worktree.py --json` | pass | Every dirty path is assigned; unknown list is empty. |
| `target/release/codefire-rs branch list --path subprojects/algorithm-evolution-agent-lab --json` | timeout after 10s | Needs product investigation before claiming branch list health. |

Run after this AI acceleration infrastructure package exists:

```bash
python3 docs/development/tools/check_ai_acceleration_docs.py
python3 docs/development/tools/classify_worktree.py --json
```

## Post-Checkpoint Cleanup

- Historical cycle detail was moved to `archive/completed-cycle-ledger.md`; the active cycle now contains only current state, the frozen batch, and its gate.
- `todo-checklist.md` remains a compact historical ledger and is below the active-document size threshold.
- `history.md` remains append-only and newest-first within the current work-log section; old entries are not rewritten for cosmetic normalization.
- Algorithm documentation was saved as one dedicated documentation commit, separate from executable nameaudit code.
- The root `/workspace` repository was not modified by the CodeFire checkpoint.
