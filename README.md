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
| Development management | `docs/development/` | The plan, open issues, history, and source maps. |
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

- `docs/development/PLAN.md` is the only current plan: goal, stage, and frozen scope.
- `docs/development/issues.md` lists open issues only.
- `docs/development/history.md` is the append-only work log.
- `AGENTS.md` holds the working rules.

Management documents from before 2026-11-10 are preserved at tag `archive/pre-rebuild-20261110`.

## Validation Gates

```bash
scripts/check.sh                                      # fmt, clippy, all Rust tests
python3 docs/development/tools/check_source_docs.py   # when commands/modules/types change
```

The Rust toolchain is pinned in `rust-toolchain.toml`. The Python reference line is frozen
(see `PLAN.md`); its checks are `python3 -m unittest discover -s tests -v` and `./demo.sh`.

## Product Reading Path

Read these first for product concepts:

1. `docs/00_philosophy.md`
2. `docs/01_product_definition.md`
3. `docs/02_core_concepts.md`
4. `docs/03_workflow_spec.md`
5. `docs/04_cli_spec.md`
6. `docs/11_internal_architecture.md`

Read these first for implementation work:

1. `docs/development/PLAN.md`
2. `docs/development/module-boundaries.md`
3. `docs/development/source-architecture.md`
4. `docs/development/command-source-trace.md`
5. `docs/development/source-data-model-catalog.md`
6. `docs/development/change-playbook.md`

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
