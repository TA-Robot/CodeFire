# AI Task Packet

Status: ready

Packet ID: MS-release-operational-trust-rights

Schema version: human-v1

Risk score: 6

Reasoning tier: high

## Objective

Classify the exact rights, license, source-identity, and version-axis facts for the smallest bounded
M1 release rehearsal. Produce a decision table that distinguishes what may be locally rehearsed,
what could later be publicly distributed, and what remains blocked. Do not infer permission from
repository access, package metadata, historical version labels, or a successful build.

## Scope

- read-only inventory of current rights, license, package, source-identity, and version-axis evidence;
- a per-artifact internal-rehearsal / later-publication / stop decision table;
- classification and handoff in this packet only.

## Authoritative Inputs

- `AGENTS.md` and `docs/development/milestones.md`;
- `README.md`, `Cargo.toml`, `Cargo.lock`, `pyproject.toml`, `setup.py`, `install.sh`;
- all checked-in license, notice, copyright, contribution, security, governance, maintainer, support,
  release, package, and artifact-identity documents that actually exist;
- `docs/development/strategy/release-and-format-roadmap.md` and `modernization-roadmap.md`;
- current Git source identity and the retained source-bound runner/provenance records, read-only.

Absence is evidence. Record missing files and undecidable ownership explicitly; do not create or
choose license text.

## Questions

1. Which exact Rust binary, Python package/fallback, public schemas/docs, source archive, checksum,
   provenance, and generated metadata are candidates for the bounded rehearsal?
2. For each candidate, what checked-in evidence identifies author/owner, applicable license,
   third-party obligations, generated-source relationship, and source revision?
3. Which candidates are safe only for an isolated internal rehearsal, which are supportable for later
   public distribution, and which must stop pending a human/legal decision?
4. What are the independent version axes today: product/package version, CLI-reported version,
   schema/object format versions, compatibility profiles, and historical internal labels?
5. What exact statement prevents the rehearsal from being described as public beta, SemVer release,
   signed release, or legal clearance?
6. Is any product/source change the smallest unavoidable M1 blocker, or can the rehearsal proceed
   truthfully with an internal-only/stop decision and a manifest of unresolved rights?

## Exact Write Set

- `docs/development/ai/task-packets/milestone-release-operational-trust-rights.md` only.

## Validation

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

## Non-Goals

- legal advice, license selection, copyright transfer, contributor outreach, or external publication;
- editing package metadata, source, tests, README, LICENSE/NOTICE, SECURITY, governance, or release docs;
- building, signing, installing, uploading, or deleting artifacts;
- completing the whole Stage 4 roadmap or starting M2–M5;
- filing a new issue unless one fact is a genuine `block-m1` that cannot be represented in the packet.

## Handoff

Return exact evidence paths/lines, the per-artifact decision table, version-axis matrix, missing-rights
list, public-claim prohibitions, `block-m1` / `park` / `drop` classification, the smallest admissible
rehearsal scope, validation results, and commit SHA. Commit only this packet.
