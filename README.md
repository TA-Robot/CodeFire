# CodeFire

CodeFire is a consistency-first version-control style CLI for specification-driven development in the AI era.
It is not Git-compatible, and it is not a layer on top of Git.

CodeFire manages changes through explicit Atoms, Trace Links, Fires, Evidence, and sealed commits. The core product goal is to make it hard to record inconsistent work as complete.

## Current Repository Shape

This repository contains the CodeFire product, its reference implementation, and its dogfooding project.

| Area | Path | Role |
|---|---|---|
| Rust CLI | `crates/` | Main implementation. This is the default CodeFire CLI line. |
| Python fallback | `codefire`, `tests/` | Reference implementation and compatibility fallback. |
| Installer | `install.sh`, `pyproject.toml`, `setup.py` | Rust-first install path with Python fallback support. |
| Product docs | numbered product docs (`docs/00_*`–`docs/11_*`, `docs/13_*`), `adr/`, `schemas/` | Product definition, concepts, architecture, schemas, and decisions. The retired chapter 12 MVP plan is preserved through Git history and the completion ledger. |
| Development management | `docs/development/` | Project state, plans, issue ledgers, source maps, and long-term workflow. |
| Dogfood project | `subprojects/algorithm-evolution-agent-lab/` | A separate research project used to exercise CodeFire in real workflows. |

Do not treat `subprojects/algorithm-evolution-agent-lab/` as CodeFire product code. It is a managed target project and should stay separated from CodeFire's CLI, store, and core domains.

## Implementation Lines

CodeFire currently has two implementation lines:

- `crates/codefire-cli` builds `codefire-rs`, the Rust default CLI.
- `codefire` is the Python v0.2 reference implementation and fallback.

The Rust workspace is split by responsibility:

| Crate | Responsibility |
|---|---|
| `codefire-core` | Atom extraction, trace/fire/verification policy, scan model, domain rules. |
| `codefire-store` | Content-addressed object storage and sealed commit validation. |
| `codefire-cli` | CLI parsing, command orchestration, rendering, remote transport, automation JSON. |
| `codefire-util` | Shared utility code. |

Keep parsing, validation, planning, applying, and rendering separate. Do not grow `main.rs` with new domain logic when a focused module is more appropriate.

## Quick Start

Build and inspect the Rust CLI:

```bash
cargo build --workspace
./target/debug/codefire-rs --help
./target/debug/codefire-rs --version
```

Run the Python reference checks:

```bash
python3 -m py_compile codefire tests/test_codefire_cli.py
python3 -m unittest discover -s tests -v
./demo.sh
```

Install Rust as `codefire` and Python as `codefire-py`:

```bash
./install.sh --prefix "$HOME/.local"
codefire --help
codefire-py --help
```

Use a prebuilt Rust binary when `cargo` is not available:

```bash
./install.sh --prefix "$HOME/.local" --binary ./target/release/codefire-rs
```

Generate shell completion:

```bash
codefire completion bash > ~/.local/share/bash-completion/completions/codefire
codefire completion zsh > ~/.local/share/zsh/site-functions/_codefire
```

## Development Management

Start all project-management work from `docs/development/README.md`.

The durable management model is defined in `docs/development/project-management.md`. In short:

- `README.md` is the stable product and repository entry point.
- `docs/development/README.md` is the development-management hub and current state entry.
- `docs/development/project-management.md` defines the long-term governance model.
- `docs/development/ai/README.md` defines the external AI development acceleration system.
- `docs/development/strategy/README.md` holds promoted engineering/product decisions and the only active strategic roadmap.
- `docs/development/exploration/README.md` holds the three completed, non-authoritative exploration phases and their falsifiable experiments.
- `docs/development/history.md` is the append-only work log.
- `docs/development/bug-backlog.md` and `docs/development/bug-issues/` are the CFB product issue ledger.
- `docs/development/code-review-issues-2026-06-06.md` and `docs/development/code-review-issues/` are the CFR source-review issue ledger.
- `docs/development/current-cycle-burn-down-plan.md` is only for the active burn-down batch; old cycle details belong in history.

Avoid duplicating current issue counts or phase names across multiple files. If a current value changes, update the management hub and append the reason to `history.md`.

## Validation Gates

Use the narrowest relevant check first, then widen before handoff.

Rust product changes:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Python fallback changes:

```bash
python3 -m py_compile codefire tests/test_codefire_cli.py
python3 -m unittest discover -s tests -v
./demo.sh
```

Installer changes:

```bash
bash -n install.sh
./install.sh --prefix /tmp/codefire-install-smoke
/tmp/codefire-install-smoke/bin/codefire --help
/tmp/codefire-install-smoke/bin/codefire-py --help
```

Dogfood project changes:

```bash
cd subprojects/algorithm-evolution-agent-lab
PYTHONPATH=src python3 -m unittest discover -s tests -v
```

Docs-only changes do not require full test runs, but links, current-state claims, issue counts, and source-of-truth references must be checked.

## Product Reading Path

Read these first for product concepts:

1. `docs/00_philosophy.md`
2. `docs/01_product_definition.md`
3. `docs/02_core_concepts.md`
4. `docs/03_workflow_spec.md`
5. `docs/04_cli_spec.md`
6. `docs/11_internal_architecture.md`

Read these first for implementation work:

1. `docs/development/README.md`
2. `docs/development/project-management.md`
3. `docs/development/ai/README.md`
4. `docs/development/module-boundaries.md`
5. `docs/development/source-architecture.md`
6. `docs/development/command-source-trace.md`
7. `docs/development/source-data-model-catalog.md`
8. `docs/development/change-playbook.md`

## Feature Surface

The Rust CLI covers the core local and remote workflow:

```text
init
open
branch list
status
scan
fire
extinguish
verify
commit
clone
merge
merge --continue
show
diff
upload
list
request-merge
request-list
request-review
request-apply
review-pack
patch export
patch import
doctor
storage report
evidence add
explain
context
migrate check
migrate dry-run
migrate apply
serve
completion
capabilities
version
```

The major implemented areas include:

- content-addressed object storage and sealed commit validation;
- local branch open/clone/merge workflows;
- Atom extraction for Markdown, OpenAPI, SQL, and multiple programming languages;
- Trace Link validation, Fire generation, extinguish workflows, and evidence capture;
- structured JSON command envelopes, exit-code taxonomy, next actions, and context packs;
- machine-readable capability/support manifests, checked base schemas, checksum conformance vectors, and snapshot-bound remote pagination;
- file-backed, HTTP, and HTTPS remote flows;
- commit signatures plus payload-bound versioned request HMAC subjects, key windows, and fail-closed nonce replay protection;
- subject-bound verification/evidence execution with minimal environments, streaming output caps, redaction, and explicit trust profiles;
- local and remote storage/doctor diagnostics, migration, review-pack, patch import/export, and diff tooling.

Current limitations and compatibility notes are maintained in `docs/development/known-limitations.md`.

## Non-Goals

CodeFire intentionally does not aim to provide:

- Git compatibility;
- staging areas, stash, WIP commits, or partial commits;
- GUI or hosted service features;
- AI agent orchestration, marketplace, or prompt management inside the CodeFire product;
- semantic merge as a replacement for explicit review and evidence.

External AI, CI, and human workflows should use CodeFire as a consistency and evidence tool through its CLI, JSON envelopes, dry-run plans, and sealed history.
