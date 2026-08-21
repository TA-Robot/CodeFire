# AI Task Packet

Status: ready

Packet ID: MS-remote-happy-path-budget

Schema version: human-v1

Risk score: 5

Reasoning tier: high

## Objective

Classify the current implemented residuals of CFR-168 and CFB-136 against only the active M1 file-backed `upload -> request-merge -> request-apply` happy path, so the orchestrator admits no scale/HTTP/cache work that the path does not require.

This is node 2. It is read-only and runs in parallel with the public fixture/command probe.

## Scope

Read CFR-168, CFB-136, ADR-020, current source/architecture/command traces, existing focused tests, and historical implementation evidence only as needed to answer:

1. Which file-backed upload/MR/apply owners and decision-work/session paths are already implemented at current HEAD?
2. Does the smallest happy path traverse any unfinished CFR-168 counter/budget/closure surface?
3. Can CFB-136 same-inode mutation occur within one isolated happy-path command, and is its final revalidation already wired on that path?
4. What exact focused fixture/test can distinguish “path already works” from the smallest blocker?
5. Which unfinished HTTP, Python parity, scale, Doctor/storage, or broad graph work must remain parked?

## Non-Goals

- implementing, refactoring, running remote mutation, or widening tests;
- completing CFR-168 or changing issue status;
- treating “unfinished issue” as proof that the M1 happy path is blocked;
- reading or changing canonical dogfood state;
- starting adoption/release/product/ecosystem work.

## Validation

Set Status `completed` and classify each candidate slice `block-m1`, `park`, or `drop`. Run:

```bash
python3 docs/development/tools/check_project_docs.py
git diff --check
```

No build, test, install, upload/request command, or product write is allowed.

## Handoff

- exact production ownership/path map;
- current implemented-vs-residual boundary for CFR-168 and CFB-136;
- whether either lies on the M1 critical path;
- smallest exact write set and focused fixture only if unavoidable;
- conflict/dependency relationship with node 1;
- next legal action after fan-in.

Commit only this packet.

## Result

Not run.
