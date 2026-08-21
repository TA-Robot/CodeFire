# AI Task Packet

Status: ready

Packet ID: MS-local-sealed-loop-run

Schema version: human-v1

Base revision: integration head containing this packet

Risk score: 6

Reasoning tier: high

Parent packet: `milestone-local-sealed-loop.md`. This is node 3. Nodes 1 and 2 are complete; this is the only ready M1 lane.

## Objective

Take the selected one-line external-validity fix through the canonical CodeFire local loop and return CodeFire's own before/after evidence plus the sealed commit identity.

## Fixed inputs

Runner:

```text
/workspace/.cache/cargo-target-shared/source-keys/9c92559f4f4d51f270d27bcc5fff888936bda373dcb9150435607e6d33c861c6/release/codefire-rs
sha256 09d9fa2561de4b687200baa4b9fe54a4159e43def1a7a3cd28dc05c2741741e4
```

Canonical open directory:

```text
/workspace/project/subprojects/algorithm-evolution-agent-lab
```

Exact tracked write:

```text
/workspace/project/subprojects/algorithm-evolution-agent-lab/src/evoagent/external_validity.py
admission sha256 1e42de7df500e00f4be8d071de0a775dd3a2c2d018e927539106e23bab2c9df3
```

Exact edit: replace the `improved_domains` comprehension with the membership-filtered indexing expression recorded in `milestone-local-sealed-loop-increment.md`. Do not re-decide or add a test.

## Safety preflight

Before any CodeFire command or write:

1. Re-hash the runner and stop `block-m1` unless it equals the fixed SHA-256.
2. Confirm the exact target file has no Git diff/status in `/workspace/project` and still equals the admission SHA-256. Stop if not.
3. Capture `/workspace/project` Git status and the branch/opened/active CodeFire control-file hashes. These are preservation evidence, not permission to read or edit other tracked files.
4. Confirm `/workspace/algorithm-evolution-agent-lab` is not used; it is a stale separate repository.

## Allowed writes

- the exact target file above;
- CodeFire state under `/workspace/project/subprojects/.codefire` and the existing `.codefire-open` marker only as written by the named loop commands;
- this task packet for the result and evidence;
- ignored command-output evidence under this agent worktree if needed.

Every other tracked path in `/workspace/project` is forbidden. Do not stage, commit, format, or repair the surrounding Git dirty tree. Do not edit `crates/**`, the Python `codefire`, tests, links, specs, docs, or another dogfood file.

## Procedure

Use JSON where available and preserve each envelope verbatim in the Result.

1. Run the pre-loop baseline:

   ```bash
   "$RUNNER" status --path "$DOGFOOD" --json
   ```

   It must report the existing branch as open. A prior document saying `open-clean` is not evidence. Do not invoke `open` again against an already-open branch.
2. Apply the one-line edit only.
3. Run the focused test from the dogfood directory:

   ```bash
   PYTHONPATH=src python3 -m unittest tests.test_external_validity -v
   ```
4. Run scan with JSON:

   ```bash
   "$RUNNER" scan --path "$DOGFOOD" --json
   ```
5. For every Fire actually reported by scan, use its exact identifier and run:

   ```bash
   "$RUNNER" extinguish FIRE_ID --path "$DOGFOOD" --resolution addressed --rationale "External-validity implementation corrected" --json
   ```

   If scan reports zero Fires, record that fact and run no extinguish command. Never manufacture a Fire to satisfy a diagram.
6. Verify:

   ```bash
   "$RUNNER" verify --path "$DOGFOOD" --details --json
   ```
7. Commit:

   ```bash
   "$RUNNER" commit --path "$DOGFOOD" -m "Fix unmapped benchmark transferability" --json
   ```
8. Run post-loop `status --json`, record the sealed commit identity, and compare the preservation hashes/status. The only newly changed Git path may be the exact target file; CodeFire ignored state may change according to the loop.
9. In this packet, change Status to `completed` on success or `blocked` on the first failed precondition/command, and write the full Result. Commit only this packet in the agent branch. The external dogfood file and CodeFire state are not part of that Git commit.

## Stop rules

- Stop on the first command failure; do not diagnose by widening tests.
- Stop if status cannot read the legacy Python-created state; that is dogfood evidence for a smallest M1 blocker.
- Stop if scan/verify next actions demand product changes or a parked issue.
- Stop if extinguish would require a false rationale or if verify remains blocking.
- Do not run workspace Rust gates, full Algorithm tests, installer, Retrospect, remote commands, or discovery.

## Handoff

- runner path/hash/provenance recheck;
- safety preflight and preservation hashes;
- before/scan/extinguish/verify/commit/after envelopes and exit codes;
- exact file edit and focused-test result;
- sealed commit identity;
- any `block-m1`, with the first failing command and no workaround;
- parked instead of done;
- next allowed action: evidence record on success, orchestrator classification on block.

## Result

Not run.
