# AI Task Packet

Status: ready

Packet ID: MS-adoption-wedge-experiment

Schema version: human-v1

Risk score: 5

Reasoning tier: high

## Objective

Classify CFR-171, CFR-172, and CFR-173 against the active adoption-wedge exit, map the current
production/public owners, and select at most one bounded experiment plus only its smallest unavoidable
product blocker. Freeze the falsifier and terminal decision before any implementation or experiment.

## Scope

- active board and the three issue detail files;
- directly relevant public docs, Rust ownership/source docs, command routes, schemas, and existing tests;
- historical adoption/claim/metrics evidence read-only;
- this packet's Result only.

This lane may read source after fixing the public requirement, but it may not implement, run an import,
mutate a repository, or begin a pilot.

## Questions

1. Which existing inspect/import surfaces already satisfy the second-consumer task without a new core
   feature, and which facts remain genuinely unavailable?
2. Does CFR-171 block the smallest consumer, or can existing certificate/profile/JSON fields truthfully
   bound the claim while the full descriptor stays parked?
3. What is the smallest CFR-172 experiment topology and exact no-mutation proof?
4. Which CFR-173 measures and privacy/retention rules are necessary for this one experiment, and which
   broad pilot metrics remain parked?
5. What focused test or read-only command would falsify each proposed admission, and where are the
   parse/validate/plan/apply/render/write owners if a blocker is reproduced?

## Validation

Set Status `completed` or `blocked`, classify every finding `block-m1` / `park` / `drop`, and run:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_source_docs.py
git diff --check
```

## Non-Goals

- implementing the full CFR-171 descriptor or changing sealed object formats;
- adding telemetry, hosted services, Git bridging, agent orchestration, or broad UX surfaces;
- running the adoption experiment, import, dogfood, release, product-value, or ecosystem work;
- CFR-168/CFB-136 and other backlog roots;
- editing the board, issues, source, tests, schemas, or any packet except this one.

## Handoff

Return exact fixed HEAD, issue-by-issue verdict, existing owner/source/test map, one recommended bounded
experiment, falsifier, preservation and measurement contract, exact candidate write/test set if and
only if a blocker is reproduced, stall/scope-creep risk, and next legal action. Commit only this packet.

## Result

Not run.
