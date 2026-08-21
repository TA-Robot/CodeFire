# AI Task Packet

Status: ready

Packet ID: MS-agent-local-contract-replay

Schema version: human-v1

Risk score: 5

Reasoning tier: high

## Objective

As the same source-blind Claude session that found the contract blocker, prove the corrected public contract by taking one new bounded Algorithm change through `status -> edit -> focused test -> scan -> extinguish -> verify -> commit -> status` without reading CodeFire product source or receiving a private command correction.

This is node 5 of active M1 `MS-agent-local-contract`. The implementation session must not perform this replay.

## Admitted runner

Use the public CLI name exactly as emitted by action argv:

```text
PATH prefix: /workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release
command: codefire
codefire hardlink: /workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release/codefire
provenance binary: /workspace/worktrees/ms-agent-local-contract-seam-20260821/target/release/codefire-rs
binary sha256: 207fae943c09c2f739611e37050cbc83f34d3b53318fcefaf9d7304ba03d708b
build HEAD: 4d0f755dfb6f7fc5421dfd27cd66f90adebd4396
Rust input manifest: 95ee643bfdc6d62bb1aeaf7c304c74fb4978be089e190de3b3e6debfe37f2066 (99 files)
provenance: /workspace/worktrees/ms-agent-local-contract-seam-20260821/target/codefire-ai/rust-build-provenance.json
provenance sha256: 3d4cc3a1ae06515d63b17facdf3d89dacab9dd900fd7bf2b969e58a7885a2feb
toolchain: cargo 1.97.0 / rustc 1.97.0
```

The hardlink and `codefire-rs` are the same inode and bytes. Prepend the directory to `PATH`; do not substitute another executable for action argv element 0. Stop if any identity above differs.

## Bounded dogfood change

Canonical open directory:

```text
/workspace/project/subprojects/algorithm-evolution-agent-lab
```

Two currently clean tracked files and their admitted pre-edit SHA-256 values:

```text
src/evoagent/statistics.py  8488cfd9656e8bf4047adebac0d5fb906438f6517949b0ef1228d9639c353994
tests/test_statistics.py    4844701e33489d670e324fcd27af97a73226317eb9d793ebf3eec767bbcec542
```

Implement one behavior: reject non-finite confidence-interval inputs instead of returning a NaN/inf report.

1. Immediately after `numeric_values = tuple(float(value) for value in values)`, add a guard using the already imported `math.isfinite`; raise `ValueError("values must be finite")` when any converted value is non-finite.
2. Add one focused unittest method to `ConfidenceIntervalEstimatorTests` that checks `nan`, positive infinity, and negative infinity with `subTest` and `assertRaisesRegex(ValueError, "values must be finite")`.

Caller-supplied action values:

```text
resolution: addressed
rationale: Reject non-finite confidence interval inputs
commit message: Reject non-finite confidence inputs
```

These are task inputs, not command syntax. Obtain flags, ordering, path binding, ready/parameterized state, and argv structure only from help/envelopes/action execution metadata.

## Scope

- the two exact dogfood files above;
- the existing CodeFire state under `/workspace/project/subprojects/.codefire` and marker as written by the named loop;
- this packet for the evidence Result.

Every other `/workspace/project` path is preservation-only. The already dirty `src/evoagent/external_validity.py` is the prior sealed change and must remain untouched.

## Non-Goals

- reading `crates/**`, the Python CodeFire implementation, or CodeFire tests;
- editing CodeFire product/schema/docs or repairing the action contract;
- using the compatibility `command` string as a shell command when `execution` exists;
- changing CFB-133, CFR-169, any other issue, or any later milestone;
- full Algorithm tests, Retrospect, remote commands, install, build, or workspace Rust gates;
- cleaning/staging/committing the surrounding `/workspace/project` Git tree.

## Procedure

1. Preflight runner/provenance hashes, the two clean-file hashes/status, `/workspace/project` HEAD/status counts, and the CodeFire marker/control-file hashes. Require `codefire status --path <dogfood> --json` to report `open-clean`, Fire 0, and base `CF-COMMIT-373468dbaf827a6cc2bae9a4` before the edit. Do not run `open`.
2. Apply only the bounded edit above.
3. Run the focused test:

   ```bash
   PYTHONPATH=src python3 -m unittest tests.test_statistics -v
   ```

4. Discover and run `scan --json` from public help. Require at least one real `extinguish_fire` action so the repaired boundary is exercised.
5. For every real scan-reported Fire, consume that Fire's `execution` object, not its `command` string:
   - require `schema=codefire.action_execution.v1` and `ready=false`;
   - materialize `argv_template` by filling `resolution=addressed` and adding the rationale input using the published `required_inputs[].cli_flag` and `required_input_groups` contract;
   - execute the resulting argv directly with no shell parsing and no path/flag supplied from this packet;
   - require each extinguish result's `verify` action to be `ready=true` with a complete argv containing the canonical `--path`.
6. After all real Fires are resolved, execute the last returned ready verify argv directly. Require a passed decision and a `commit` action with `ready=false`, a path-bound `argv_template`, and required input `message`.
7. Fill only the caller-authored message value above through the action's published input contract and execute the resulting argv directly.
8. Run public `status --json` after commit. Record the sealed identity, require `open-clean` / Fire 0, and compare preservation hashes/status. Only the two admitted dogfood paths may be newly dirty in `/workspace/project`.
9. Update only this packet with full evidence and commit it on the agent branch.

If an execution object is absent, claims the wrong readiness, loses the path, cannot be materialized from its own input metadata, or produces a loop after success, stop and record `block-m1`. Do not repair it.

## Validation

The required validation is exactly the identity preflight, focused 3-case module test, CodeFire action-driven loop, post-status/preservation comparison, plus:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

No wider gate is allowed.

## Handoff

- runner/provenance/source identity;
- pre-state envelope and preservation baseline;
- exact two-file edit and focused test count;
- scan Fire IDs and the structured actions actually consumed;
- materialized argv arrays (not just shell strings) for extinguish, verify, and commit;
- readiness/input/path assertions;
- sealed commit identity and post-state;
- preservation result and any `block-m1`;
- next legal action: independent other-family audit only.

## Result

Not run.
