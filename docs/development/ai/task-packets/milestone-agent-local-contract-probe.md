# AI Task Packet

Status: ready

Packet ID: MS-agent-local-contract-probe

Schema version: human-v1

Risk score: 3

Reasoning tier: high

## Objective

Act as a fresh external worker and determine the first point where the public local CodeFire contract is insufficient to repeat the prior sealed-loop path without product-source knowledge or private command correction.

This is node 1 of active M1 `MS-agent-local-contract`. It is read-only and runs in parallel with the CFR-169 seam classification.

## Allowed inputs

- `docs/development/milestones.md` and the completed `milestone-local-sealed-loop-run.md` transcript;
- public product docs (`README.md`, `docs/04_cli_spec.md`, `docs/automation_interface.md`, `docs/runbook.md`);
- the admitted runner's `--help`, subcommand help, `capabilities --json`, and read-only `status`/`show` output;
- CFR-170 and CFB-133 detail files as issue context after the clean-room pass identifies a matching gap.

Do not read `crates/**`, the Python `codefire` implementation, Rust/Python tests, or private architecture docs before recording the clean-room result. The point is to test the external contract, not reverse-engineer the implementation.

## Scope

Read-only public-contract evaluation plus this packet's Result. The canonical dogfood repository and CodeFire state are read-only. The only writable tracked file is this packet.

## Non-Goals

- implementing CFR-170, CFB-133, or any inferred fix;
- reading product source to compensate for a missing public contract;
- mutating or replaying the canonical sealed loop;
- testing remote, adoption, release, or product-economics surfaces;
- filing new work that the first reproduced M1 blocker does not require.

## Procedure

1. Starting from only public surfaces, reconstruct each choice needed for the prior `status → edit → scan → extinguish → verify → commit` path.
2. For each JSON `next_action`, decide whether its `command` is directly executable, safely parameterized, has an explicit postcondition, and cannot loop after success.
3. Use the completed prior transcript as a fixture: do not repeat its mutation. Read-only runner calls are allowed; canonical CodeFire state must not change.
4. Stop at the first missing/unsafe/cyclic contract and classify it `block-m1`, `park`, or `drop`. If no gap exists, say so and provide the complete public-only replay script.
5. Only after the clean-room verdict, map a reproduced gap to CFR-170 and/or CFB-133. Filing is not implementation permission.

## Validation

Change Status to `completed` or `blocked`, add exact evidence and the smallest candidate resolution, then run:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

Do not edit product code, dogfood files/state, issue status, the board, or another packet. Do not run a full test suite, build, install, remote command, or parked work.

## Handoff

- first failing public-contract step, or complete public-only replay;
- exact help/envelope/action evidence;
- whether the action has complete required arguments and a machine-checkable postcondition;
- `block-m1` / `park` / `drop` classification;
- smallest candidate fix surface and focused validation, without implementing it;
- whether CFR-170 or CFB-133 is actually reproduced;
- independent packets still running and next legal action.

## Result

Not run.
