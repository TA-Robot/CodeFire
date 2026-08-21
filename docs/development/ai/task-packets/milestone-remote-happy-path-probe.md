# AI Task Packet

Status: ready

Packet ID: MS-remote-happy-path-probe

Schema version: human-v1

Risk score: 4

Reasoning tier: high

## Objective

Derive the smallest isolated file-backed fixture and public command sequence that could prove active M1 `MS-remote-happy-path`, and identify the first missing/unsafe/cyclic public-contract step before any mutation runs.

This is node 1. It runs read-only in parallel with the CFR-168/CFB-136 residual classification.

## Scope

- the active board and completed local-contract evidence;
- public CLI/help/capabilities and public remote/runbook docs;
- read-only inspection of existing fixture shapes and sealed identities;
- this packet's Result only.

Start public-only. Do not read implementation source/tests before fixing the public-contract verdict. After that verdict, existing test names may be listed only to identify a reusable isolated fixture; do not run it or mutate one.

## Questions

1. What exact local sealed branch/head, file-backed remote project(s), remote branches, actor/signing policy, and merge-request state are minimally required?
2. What exact `upload`, `request-merge`, `request-apply`, and terminal read-only commands does public help document?
3. Can the commands be run in one isolated temporary root without touching canonical Algorithm or an existing remote fixture?
4. Which JSON fields prove source upload, MR creation, apply, remote head, idempotency, and terminal state?
5. What is the first missing/unsafe/cyclic step, if any, and does it map to CFR-168/CFB-136 or neither?

## Non-Goals

- running upload/request mutations, even with dry-run;
- creating a remote project or temporary fixture;
- reading product source to compensate for a missing public contract;
- implementing CFR-168/CFB-136 or starting HTTP/scale work;
- editing the board, issues, another packet, or canonical dogfood state.

## Validation

Set Status `completed` or `blocked`, record exact help/docs/envelope evidence and `block-m1` / `park` / `drop`, then run:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

## Handoff

- exact isolated fixture topology and source sealed identity;
- exact public command sequence and expected JSON postconditions;
- first missing contract or complete runnable plan;
- smallest candidate resolution if blocked, without implementation;
- CFR-168/CFB-136 mapping;
- deterministic preservation checks and next legal action.

Commit only this packet.

## Result

Not run.
