# AI Task Packet

Status: ready

Packet ID: MS-release-operational-trust-rehearsal

Schema version: human-v1

Risk score: 7

Reasoning tier: high

## Objective

Derive the smallest isolated, locally published release-candidate topology that a fresh operator can
verify, install into an isolated prefix, smoke, and roll back using recorded evidence only. Classify
the current build/provenance/install surfaces and stop at the first missing contract. Do not build,
install, publish, or mutate during this packet.

## Scope

- read-only classification of build, provenance, packaging, install, smoke, and rollback surfaces;
- one proposed local-only release root, isolated prefix, evidence boundary, and operator handoff;
- classification and handoff in this packet only.

## Authoritative Inputs

- `AGENTS.md` and `docs/development/milestones.md`;
- `install.sh`, package manifests, source-bound Rust build/provenance tooling, checksum/conformance
  schemas, and their focused tests;
- public version/help/capabilities surfaces and release/format strategy documents;
- retained B1 runner and provenance as historical evidence, not automatically the release candidate;
- the current intentional dirty main worktree and the integration branch, both read-only.

## Required Classification

1. Select the exact source revision and clean-worktree rule for a rehearsal candidate. State whether
   the existing source-bound runner can be reused or a fresh accepted-source build is required.
2. Define one `$RELEASE_ROOT`, immutable artifact staging area, manifest/checksum/provenance record,
   local publication pointer, isolated install prefix, prior-state/rollback target, and evidence root.
3. Name the exact operator commands for identity verification, checksum validation, source binding,
   install, `--version`/`--help`/`capabilities --json` smoke, and rollback. No command may touch the
   user's current install or canonical Algorithm repository.
4. Define before/after hashes for integration/main, canonical CodeFire store, current user install,
   artifact staging, install prefix, and rollback target.
5. Define crash/partial-publication and wrong-artifact falsifiers. Publication must be local and
   atomic or fail without presenting a mixed candidate.
6. Separate required M1 evidence from parked Stage 4 signing, SBOM, external registry, credential,
   support/security policy, backup, and two-person response work.
7. Classify the first missing contract as `block-m1`, `park`, or `drop`, and propose the smallest
   exact write set only if a reproduced blocker exists.

## Exact Write Set

- `docs/development/ai/task-packets/milestone-release-operational-trust-rehearsal.md` only.

## Validation

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

## Non-Goals

- running a build or installer, changing the user's PATH/prefix, or invoking CodeFire against canonical state;
- public GitHub/PyPI/crates.io upload, network publication, credentials, signing, or release naming;
- editing source, tests, product/public docs, package metadata, license files, or another task packet;
- treating a retained binary as source-bound without revalidating its recorded revision and manifest;
- broad Stage 4 completion, M2 product research, or M3–M5 ecosystem work.

## Handoff

Return the exact local topology, source/reuse decision, public commands and cwd, preservation manifest,
operator handoff, rollback/falsifier matrix, first missing contract, `block-m1` / `park` / `drop`, any
smallest proposed fix write set, validation results, and commit SHA. Commit only this packet.
