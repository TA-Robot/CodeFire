# AI Task Packet

Status: ready

Packet ID: MS-agent-local-contract-seam

Schema version: human-v1

Risk score: 3

Reasoning tier: high

## Objective

Classify whether the smallest plausible M1 local-contract fix actually requires a CFR-169 module/test ownership seam, or whether CFR-169 must remain parked while CFR-170/CFB-133 are handled in existing owners.

This is node 2 of active M1 `MS-agent-local-contract`. It is read-only and runs in parallel with the clean-room public-contract probe.

## Inputs and questions

Read CFR-169, CFR-170, CFB-133, `module-boundaries.md`, `source-architecture.md`, `command-source-trace.md`, the completed prior loop packets, and only the source/test files necessary to locate current ownership.

Answer:

1. Which modules own JSON envelopes, `next_actions`, help rendering, migrate postconditions, and command argument completion?
2. Can the likely smallest contract correction stay inside existing owners with focused tests?
3. Would touching CFR-169 now shorten the M1 critical path, or create an architecture/refactor detour?
4. If a seam is unavoidable, what is the smallest exact file boundary and deterministic focused test? If it is avoidable, classify CFR-169 `park` and stop.

Do not design a broad refactor, implement code, expand tests, or promote a parked remote/adoption/release root.

## Scope

Read-only ownership mapping plus this packet's Result. Source reading is limited to files needed to locate the existing contract owners. The only writable tracked file is this packet.

## Non-Goals

- implementing or designing a broad CFR-169 refactor;
- changing issue status or admitting CFR-169 before evidence;
- evaluating remote, adoption, release, or product-economics work;
- running build/test gates for unchanged product code;
- duplicating the clean-room public-contract probe.

## Validation

Change Status to `completed` and record `block-m1`, `park`, or `drop` with evidence. Run:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

No product/dogfood/state write, build, install, full test suite, or issue-status change is allowed.

## Handoff

- ownership map for the likely contract fix;
- CFR-169 classification and why it does or does not sit on the critical path;
- smallest exact write set if unavoidable;
- focused tests already owning that surface;
- integration/conflict risk with the clean-room probe's likely result;
- next allowed action after fan-in.

## Result

Not run.
