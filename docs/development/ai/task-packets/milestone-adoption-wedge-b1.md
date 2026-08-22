# AI Task Packet

Status: ready (classification only)

Packet ID: MS-adoption-wedge-b1

Schema version: human-v1

Risk score: 6

Reasoning tier: high

## Objective

Classify the exact smallest public-contract correction justified by the first adoption experiment's
`narrow` decision: document and machine-bind only the existing `branch list --json` and `show --json`
payload facts the source-blind consumer relied on, without implementing the full CFR-171 descriptor.

## Scope

- integration evidence in `milestone-adoption-wedge-run.md` and preserved consumer/controller roots;
- current `branch list` / `show` parse, validation, typed data, render, capabilities, public schema,
  docs, and focused tests;
- this packet's Result only.

This is a read-only classification. Do not implement, edit another packet, run a consumer retry, or
mutate any repository.

## Questions

1. What exact current payload shapes/versions/types/fields did Q1–Q7 cite, and which are already typed
   versus ad-hoc JSON?
2. What is the smallest backward-compatible way for public schemas/capabilities to bind the two
   command-specific `data` shapes while retaining the shared command-result envelope?
3. Which fields are required for repo/branch/head/base/claim/missing-authority comprehension, and which
   unrelated branch/show fields must remain outside the admitted guarantee?
4. What exact product/docs/schema/test write set and focused validations prove the contract, including
   unknown/additional-field/version behavior?
5. Does any Store/object/certificate persistence change become necessary? If yes, classify `block-m1`
   and stop; the admitted slice is intended to be projection/schema/docs only.

## Validation

Set Status `completed` or `blocked`, classify findings `block-m1` / `park` / `drop`, and run:

```bash
python3 docs/development/tools/check_project_docs.py
python3 docs/development/tools/check_source_docs.py
git diff --check
```

## Non-Goals

- full CFR-171 lossless cross-surface descriptor, persisted format, Store validator, or tamper corpus;
- changing actual claim semantics, branch state, commit objects, certificate authority, or command
  behavior beyond machine-readable contract binding;
- Python/HTTP parity, toolchain acquisition, review-pack/import, Git bridge, telemetry, agent platform;
- rerunning or rescoring the consumer before a fix is integrated;
- CFR-168/CFB-136 or M2–M5.

## Handoff

Return fixed HEAD, exact current payload map, recommended schema/capability strategy, required and
excluded fields, exact write/test set, compatibility/invariant argument, issue verdict, and next legal
action. Commit only this packet.

## Result

Not run.
