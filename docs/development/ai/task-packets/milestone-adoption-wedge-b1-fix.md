# AI Task Packet

Status: ready

Packet ID: MS-adoption-wedge-b1-fix

Schema version: human-v1

Risk score: 7

Reasoning tier: high

## Objective

Implement only the B1 correction proven by the first adoption experiment: provide stable public data
schemas for existing `branch list --json` and `show --json`, advertise them through capabilities, and
add a backward-compatible `show` claim projection for the certificate subject/profile/phase facts the
public consistency docs already promise. Preserve all persistence and command behavior.

## Scope

- two new public data schemas and their capabilities/support-manifest advertisement;
- additive `show` JSON claim projection while retaining the existing certificate result string;
- directly relevant Rust regression coverage and public/canonical source docs;
- this packet only.

## Required contract

- The shared `codefire.command_result.v1` envelope and each command's `output_schema` remain unchanged.
- Add optional `data_schema` to capabilities command entries; advertise it only for `branch` and
  `show`. Add both schema IDs/paths to `support.schemas`.
- `branch list` data schema v1 requires `type`, `version`, and branch entries with `name`, `head`, and
  persisted `state`; allow noncritical extra fields. Document that copied `state=open-clean` does not
  prove an `opened/` registration currently exists; doctor/opened state is the live authority.
- `show` data schema v1 requires current `show.target`, `commit`, `parents`, certificate result string,
  signature presence, and an additive claim projection. Do not rename or replace `certificate`.
- The additive claim projection must expose, from the already validated persisted certificate only:
  result, profile, phase, and subject kind/trust kind/immutability when present. Legacy omission is
  represented without inventing proof; do not synthesize stronger authority.
- Schema version 1 is required; a future version is denied by schema. Required comprehension fields
  must fail validation when missing; noncritical extra keys remain accepted.
- Actual branch/show resolution, sealed validation, Store objects, certificate bytes, signature
  authority, human output, and write behavior remain unchanged.

## Exact write set

- `schemas/public/branch-list-v1.schema.json`;
- `schemas/public/show-result-v1.schema.json`;
- `schemas/public/capabilities-v1.schema.json`;
- `crates/codefire-cli/src/compatibility.rs`;
- `crates/codefire-cli/src/command_registry.rs`;
- `crates/codefire-cli/src/view.rs`;
- focused tests under `crates/codefire-cli/src/tests.rs` or one responsibility-named test module;
- `docs/04_cli_spec.md`, `docs/automation_interface.md`;
- directly relevant rows/paragraphs in all four canonical source docs:
  `docs/development/source-architecture.md`, `command-source-trace.md`,
  `source-data-model-catalog.md`, `change-playbook.md`;
- this packet.

Any Store/core/object/persistence/conformance-fixture/Python/HTTP file requirement is a stop condition,
not silent scope expansion.

## Validation

Run the smallest focused tests covering capabilities, both data shapes, missing-required/future-version/
extra-key schema behavior, and the additive claim projection, then:

```bash
cargo fmt --check
cargo clippy -p codefire-cli --all-targets -- -D warnings
python3 docs/development/tools/check_source_docs.py
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_ai_acceleration_docs.py
git diff --check
```

Build a source-bound release runner only after the accepted source and tests are committed.

## Non-Goals

- full CFR-171 descriptor, cross-surface equality, Store validator, object-format change, tamper corpus;
- changing `certificate` from string to object, changing the branch-list envelope command name, or
  turning persisted branch state into live-open authority;
- Python/HTTP parity, toolchain acquisition, review-pack/import, Git bridge, telemetry, agent platform;
- altering first-run evidence, task, questions, rubric, thresholds, or decision;
- rerunning the consumer before a corrected source-bound runner exists;
- CFR-168/CFB-136 or M2–M5.

## Handoff

Return commit SHA, exact schemas/IDs/fields, additive projection semantics, compatibility argument,
changed files, focused test counts, all validation results, and source-bound runner/provenance identity.
Commit only the admitted write set.

## Result

Not implemented.
