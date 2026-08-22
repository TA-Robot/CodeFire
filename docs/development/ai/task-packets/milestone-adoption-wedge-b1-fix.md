# AI Task Packet

Status: completed

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

Status **completed** at admission HEAD `1710251f78a84487adfcb109f86cfd213581580a`
(merge of `0cee171`). Implemented only the admitted B1 slice. No Store/core/object/
Python/HTTP/conformance-fixture change. Envelope `output_schema`, certificate result
string, branch/show resolution, sealed validation, and write behavior are unchanged.

### Schemas and IDs

| ID | Path | Advertiser |
|---|---|---|
| `codefire.branch_list.v1` | `schemas/public/branch-list-v1.schema.json` | `capabilities.commands[name=branch].data_schema` |
| `codefire.show_result.v1` | `schemas/public/show-result-v1.schema.json` | `capabilities.commands[name=show].data_schema` |

`commands[].output_schema` remains `codefire.command_result.v1`. Other commands omit
`data_schema`. Both IDs/paths are in `support.schemas`. `data.version` other than `1`
is denied by schema; required comprehension fields fail when missing; noncritical extra
keys are accepted (`additionalProperties: true`).

Required branch-list fields: `type`, `version`, `branches[]{name,head,state}`.
`state` is persisted branch-record state, not live `opened/` authority.

Required show fields: `type`, `version`, `show.{target,commit,parents,certificate,signature,claim}`;
`signature.present`; `claim.result`. `certificate` stays a string.

### Additive claim projection

`show.claim` copies the already-validated persisted certificate: always `result`;
`profile` / `phase` / subject `{kind,trust_kind,immutable}` only when those fields are
present. Init/legacy certificates therefore project `{result}` only and do not invent
`local_verification_v1`. Current `run_commit` certificates project profile/phase/subject
kind facts. Human text still prints `Certificate: <result>` only.

### Focused tests

`crates/codefire-cli/src/tests/show_branch_list_contract.rs` — 3 tests, all passed:

- `capabilities_advertise_branch_and_show_data_schemas_only`
- `branch_list_and_show_payloads_match_public_data_schemas`
- `show_claim_projects_persisted_certificate_fields_without_inventing_proof`

### Validation

```text
cargo test -p codefire-cli --bin codefire-rs -- show_branch_list_contract
  3 passed; 0 failed; 775 filtered out
cargo fmt --check
cargo clippy -p codefire-cli --all-targets -- -D warnings
python3 docs/development/tools/check_source_docs.py
  Source docs check passed: 44 modules, 34 commands, 12 object kinds
python3 docs/development/tools/check_project_docs.py
  Project docs check passed; 553 Markdown files inventoried
python3 docs/development/tools/check_ai_acceleration_docs.py
  AI acceleration docs check passed
git diff --check
```

No `block-m1` beyond the admitted B1 slice. Full CFR-171, Store validator, Python/HTTP,
and consumer rerun remain parked.

### Source-bound runner

Recorded in the follow-up commit after `python3 docs/development/tools/build_source_bound_rust.py`
on the accepted source commit.
