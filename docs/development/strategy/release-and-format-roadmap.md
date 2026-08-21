# Release and Format Roadmap

Status: canonical replacement for the v0.6–v1.0 planning series

Owner: Release Manager

Update trigger: product/package/format identity changes, a release rehearsal completes, or a compatibility decision is accepted.

## Why This Replaces Version Plans

The historical v0.6, v0.7, v0.8, v0.9, and v1.0 documents mixed completed implementation batches, future release claims, internal milestones, and format work. The repository currently exposes several independent identities, so a new feature checklist under another version number would deepen ambiguity.

## Observed Identity Baseline

As of 2026-07-12:

| Axis | Observed value | Meaning / limitation |
|---|---|---|
| Rust crate package | `0.1.0` in all workspace crates | Package metadata; not aligned with prior v0.6–v1.0 plan labels. |
| Python package | `0.2.0` | Reference/fallback package identity. |
| Rust CLI output | `codefire foundation 1` | Internal foundation identity, not SemVer or source provenance. |
| Git releases | no tags in the local repository | No repository-tag evidence for a public release line. |
| Repository layout | version `1` | Persistent repository compatibility axis. |
| Core object/model structs | version `1` | Domain serialization axis. |
| Atom hash schema | current Rust-default writes use `4`; lower authenticated values remain historical inputs | Identity/extractor-related schema axis; source-unchanged drift is reported as tool migration rather than ordinary source change. |
| Commit root schema | current Rust-default writes use `2`; legacy absence remains readable | Required-root compatibility axis; the Python fallback is not included in this writer claim. |
| Automation envelope | Rust-default declared JSON surfaces use `codefire.command_result.v1` | Machine API envelope axis; the Python fallback has no shared `--json`/envelope surface. |
| Remote/public support profile | `codefire.support.v1`; branch/MR version `1` | Capabilities advertise readable/current formats and checked public schema IDs; this is not product SemVer. |
| License metadata | Cargo workspace `MIT`; `setup.py` says `UNLICENSED`; `pyproject.toml` has no license field; no root `LICENSE` file | Rights and distribution terms are unresolved and inconsistent. |

This table is an observed worktree baseline, not a compatibility promise.

## Independent Version Axes

Maintain and expose these separately:

1. **Product SemVer** — user-visible CLI behavior and supported compatibility.
2. **Package version** — Cargo/Python distribution identity; normally aligned to product SemVer after release policy is adopted.
3. **Repository layout version** — directory and persistent metadata compatibility.
4. **Object/root schema versions** — serialized object meaning and required roots.
5. **Hash/extractor semantic version** — identity-producing extraction/canonicalization behavior.
6. **Automation/schema version** — JSON envelope and individual payload contracts.
7. **Evidence/conformance profile version** — what was checked and what a certificate may claim.

Changing one axis does not silently change another. Every release note states the exact tuple and migration implications.

## First Public Release Decision

Do not label the current tree v1.0 or public beta from the historical plan names. First complete:

- rights and license inventory;
- external-consumer and distribution inventory;
- current crate/Python/binary/version-axis reconciliation;
- compatibility and migration fixtures;
- cargo-based source validation from a clean environment;
- signed non-public release rehearsal and independent install verification;
- supported versions, security intake, and rollback policy.

After the audit, choose a SemVer based on external compatibility obligations. If no external contract exists, an explicit pre-1.0 line is the default candidate; this is not accepted until the Release Manager records the audit.

## Release Channels

Candidate channels, pending the release audit:

- `nightly` or development artifacts: no compatibility/support promise; source revision mandatory;
- `preview`: selected testers, explicit schema/profile tuple, migration may be required;
- `stable`: supported SemVer, signed artifacts, documented compatibility and security window.

Channel labels never replace version numbers or subject digests.

## Release Gate

Every distributable release record must bind:

- source revision and clean-build inputs;
- product/package/version-axis tuple;
- checksums and signature identity;
- SBOM and provenance;
- Rust/Python/installer tests with exact environment;
- compatibility and migration results;
- known limitations, supported deployment, and rollback;
- independent verifier and evidence digests.

No stale local binary counts as evidence for modified Rust sources, even when its timestamp is newer. A trusted build must bind the binary to the source revision and build inputs.

## Compatibility Matrix to Build

The Release Manager owns a real fixture matrix; old planned fixture names are not evidence. At minimum cover:

| Fixture class | Assertions |
|---|---|
| Python legacy empty/local commit/merge/remote/signed repositories | Rust read/doctor/show/migrate behavior and explicit unsupported cases. |
| Current Rust local commit/merge/remote/signed repositories | Current Rust read/write plus the documented Python emergency-read boundary. |
| Legacy 12-hex and current 24-hex object IDs | Full digest agreement, no silent prefix alias, deterministic migration. |
| Commit root schema 1 and 2 | Required-root validation and legacy readability. |
| Atom hash schema 1 through 4 | Stable identity vectors, extractor-semantic provenance, source-unchanged migration classification, and explicit resolution-staleness behavior. |
| Supported and unsupported automation/config/batch versions | Expected accept/reject result; unsupported never silently downgrades policy. |

Each fixture records canonical bytes, full digests, expected public IDs, writer version, readable/writable consumers, migration result, and checksum. The checked `schemas/conformance/manifest-v1.json` now provides the initial canonical JSON, remote-branch version, and object-extension criticality vectors, verified by Rust, Python fallback, and an independent stdlib consumer. This is a base corpus rather than completion of the full matrix; release readiness remains blocked until the remaining selected families run in CI or an equivalent clean environment.

## Historical Milestone Mapping

| Historical label | Durable outcome retained | Current destination |
|---|---|---|
| v0.6 Rust rewrite | Rust-first CLI, Python fallback, installer, automation/diff/remote surface | README, CLI spec, source architecture, issue/history evidence |
| v0.7 issue improvement | Source-review fixes and automation-context hardening | CFR ledger, history, tests |
| v0.8 CFB burn-down | Root-cause grouped product issue closure | CFB ledger, root-cause map, tests |
| v0.9 contract/scale | Contract, reducer, evidence, durability themes | Stage 1–2 roadmap and P0 experiment corpus |
| v1.0 contract validation | Completion and dogfood gate themes | Stage 1 correctness and Stage 4 release gate |

The old plan text is not an active release promise. Git history and the deletion manifest identify removed source files.

The v0.6 internal milestone had a dated validation snapshot on 2026-06-05 covering then-current fmt/clippy/workspace tests, 87 Python tests, demo, local/remote/signature/install smoke. It had no local Git tag and is not reusable as current source or public-release evidence.

## Disposition of Old Plan-Only Candidates

| Candidate from superseded plans | Disposition |
|---|---|
| Hard numeric command-latency targets such as sub-100ms status | Not promoted: no reproducible baseline or accepted fixture. Establish work counters and benchmark distributions before setting a gate. |
| Eight-crate target architecture | Rejected as a historical design sketch; the current four-crate responsibility split is authoritative until an evidence-backed module ADR changes it. |
| tree-sitter, explicit-only extraction, incremental index cache | Deferred product experiments; promote only with precision/recall, compatibility, and total-cost evidence. |
| policy profiles and per-branch/per-remote overlays | Deferred; require a policy/schema ADR and migration/precedence matrix. |
| server verification sandbox abstraction and mutation-audit schema | Stage 1 security target; current supported boundary remains trusted/local or single-tenant. |
| full `CommandSpec` registry with parser/runner/path/dry-run ownership | Architecture experiment; current partial registry remains implementation truth. |
| one normalized operation-plan schema | Contract experiment; do not assert uniform fields until current plan variants are inventoried and consumers tested. |
| richer `NextAction` fields (`args`, `safety`, `priority`, recovery omission) | Schema evolution candidate; current four-field contract remains authoritative until compatibility evidence exists. |
| mandatory commit hash on every issue closure | Not adopted as a universal rule; require subject-bound regression evidence, and include a commit/patch digest when automation supplies one. |
| keep Python for a fixed number of releases | Superseded by the evidence-based fallback retirement gate below. |

## Python Fallback

Python remains a reference/fallback until evidence shows:

- required compatibility/migration/incident uses are covered elsewhere;
- Rust behavior and golden vectors are authoritative and reproducible;
- users have a documented migration path;
- removing or internalizing the Python entrypoint reduces more maintenance risk than it creates.

Retirement is a separate compatibility decision, not a cleanup side effect.
